//! handyman: a multi-protocol L4 proxy for researching IoT traffic.
//!
//! READING GUIDE, step 1 of 15. Read the files in this order; every file's header names the next one.
//!
//!  1. main.rs                  wiring: what gets created and who talks to whom
//!  2. core.rs                  map of the core
//!  3. core/event.rs            the data model: everything the proxy observes is an `Event`
//!  4. core/bus.rs              how events travel from the forwarding code to the analysers
//!  5. core/acceptor.rs         where connections come from
//!  6. core/proxy.rs            accept loop: one task per connection
//!  7. core/flow.rs             forwarding bytes and emitting `Data` events (the hot path)
//!  8. core/stack.rs            all events of one flow kept together
//!  9. core/decoder.rs          the plug-in interface every protocol implements
//! 10. core/flow_table.rs       consumer that builds stacks and runs the decoders
//! 11. protocols.rs, protocols/http.rs   a complete text-protocol decoder
//! 12. protocols/mqtt.rs        a binary protocol, for comparison
//! 13. consumers/logger.rs, consumers/summary.rs   what is done with events and finished stacks
//! 14. core/handler.rs          the future active mode (defined, not wired yet)
//! 15. http_server/server.rs    test upstream only, not part of the proxy; read whenever you like
//!
//! Life of one chunk of bytes in `proxy` mode:
//!
//! ```text
//!  device ──TCP──▶ StaticAcceptor ──▶ proxy::run spawns flow::run_flow (one task per connection)
//!                                        │
//!                                        ├─ Pump reads a chunk from one socket
//!                                        ├─ publishes Event::Data on the EventBus   (never waits)
//!                                        └─ writes the same chunk to the other socket
//!
//!  EventBus ──▶ consumers::logger         prints every event
//!           └─▶ core::flow_table          appends the event to its flow's Stack,
//!                                         feeds Data to the protocol decoder (http, mqtt, ...),
//!                                         and on Ended hands the finished Stack to consumers::summary
//! ```
//!
//! Start below at `main`, then follow `proxy()`. Next file: core.rs

pub mod consumers;
pub mod core;
pub mod http_server;
pub mod protocols;

use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;

use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use crate::core::acceptor::StaticAcceptor;
use crate::core::bus::EventBus;
use crate::core::flow::FlowConfig;
use crate::core::flow_table::FlowTable;

const USAGE: &str = "\
usage:
  handyman serve [ROOT_DIR] [HOST:PORT]   serve static files      (default: ../test_page 127.0.0.1:8080)
  handyman proxy [LISTEN] [UPSTREAM]      forward and decode TCP  (default: 127.0.0.1:8081 -> 127.0.0.1:8080)

Set RUST_LOG=debug to also log every data chunk.";

const EVENT_BUS_CAPACITY: usize = 4096;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("serve") => serve(&args[1..]).await,
        Some("proxy") => proxy(&args[1..]).await,
        _ => {
            eprintln!("{USAGE}");
            std::process::exit(2);
        }
    };
    if let Err(e) = result {
        error!("{e}");
        std::process::exit(1);
    }
}

async fn serve(args: &[String]) -> io::Result<()> {
    let root_dir = PathBuf::from(args.first().map_or("../test_page", String::as_str));
    let addr = resolve(args.get(1), "127.0.0.1:8080").await?;
    let host = addr.ip().to_string();

    tokio::select! {
        result = http_server::server::create_server(&host, addr.port(), root_dir) => result,
        _ = tokio::signal::ctrl_c() => Ok(()),
    }
}

async fn proxy(args: &[String]) -> io::Result<()> {
    let listen = resolve(args.first(), "127.0.0.1:8081").await?;
    let upstream = resolve(args.get(1), "127.0.0.1:8080").await?;

    // (1) The bus: the forwarding code publishes into it, analysers read from it (core/bus.rs)
    let bus = EventBus::new(EVENT_BUS_CAPACITY);

    // (2) Observation half, each consumer as its own task with its own receiver.
    // Consumers subscribe before the first connection, so they see every flow from its start
    tokio::spawn(consumers::logger::run(bus.subscribe()));
    tokio::spawn(core::flow_table::run(
        bus.subscribe(),
        // Which protocols can be recognised and decoded (protocols.rs)
        FlowTable::new(protocols::default_registry()),
        // Called once per finished flow with its complete Stack (consumers/summary.rs)
        |stack| consumers::summary::print_stack(&stack),
    ));

    // (3) Forwarding half: where connections come from (core/acceptor.rs)...
    let acceptor = StaticAcceptor::bind(listen, upstream).await?;
    info!(%upstream, "forwarding all connections");

    // (4) ...and the accept loop that spawns one flow task per connection (core/proxy.rs → core/flow.rs)
    tokio::select! {
        result = core::proxy::run(acceptor, bus, FlowConfig::default()) => result,
        _ = tokio::signal::ctrl_c() => {
            info!("shutting down");
            Ok(())
        }
    }
}

/// Resolves `HOST:PORT` (names like `test.mosquitto.org:1883` work too), falling back to `default`.
async fn resolve(arg: Option<&String>, default: &str) -> io::Result<SocketAddr> {
    let text = arg.map_or(default, String::as_str);
    tokio::net::lookup_host(text)
        .await?
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, format!("cannot resolve {text:?}")))
}
