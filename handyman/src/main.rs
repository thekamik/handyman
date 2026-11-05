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

    let bus = EventBus::new(EVENT_BUS_CAPACITY);
    // Consumers subscribe before the first connection, so they see every flow from its start
    tokio::spawn(consumers::logger::run(bus.subscribe()));
    tokio::spawn(core::flow_table::run(
        bus.subscribe(),
        FlowTable::new(protocols::default_registry()),
        |stack| consumers::summary::print_stack(&stack),
    ));

    let acceptor = StaticAcceptor::bind(listen, upstream).await?;
    info!(%upstream, "forwarding all connections");

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
