//! One proxied TCP connection, from connect to close.
//!
//! READING GUIDE, step 7 of 15 (previous: core/proxy.rs, next: core/stack.rs).
//!
//! The most important file of the forwarding half. Read in this order:
//! 1. `run_flow`: the life of a connection, split into numbered phases below.
//! 2. `Pump::run`: the loop that copies one direction and publishes a `Data` event per chunk.
//! 3. `Activity`: the shared "last seen traffic" clock behind the idle timeout.
//!
//! Every byte the device or the server sends goes through `Pump::run` exactly once.

use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use bytes::Bytes;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use crate::core::acceptor::Accepted;
use crate::core::bus::EventBus;
use crate::core::event::{CloseReason, Direction, Event, EventKind, FlowId, FlowMeta, Leg, Transport};

#[derive(Debug, Clone)]
pub struct FlowConfig {
    pub connect_timeout: Duration,
    /// Must be longer than protocol keep-alives (MQTT commonly uses 60 s), or idle devices get cut off.
    pub idle_timeout: Duration,
    pub buffer_size: usize,
}

impl Default for FlowConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(300),
            buffer_size: 16 * 1024,
        }
    }
}

/// Runs one proxied connection: connects upstream, forwards both directions and reports everything as events.
///
/// Decoding happens in bus consumers, not here, so forwarding never waits for analysis.
/// An active mode (a `Handler` deciding per message) would move decoding onto this path.
pub async fn run_flow(id: FlowId, accepted: Accepted, bus: EventBus, config: FlowConfig) {
    // Phase 1: announce the flow, even before we know whether the server is reachable
    let Accepted { stream: client, client: client_addr, original_dst } = accepted;
    bus.publish(Event::now(
        id,
        EventKind::Started(FlowMeta { client: client_addr, original_dst, transport: Transport::Tcp }),
    ));

    // Phase 2: open our own connection to the real server (the proxy is a client here)
    let upstream = match tokio::time::timeout(config.connect_timeout, TcpStream::connect(original_dst)).await {
        Ok(Ok(stream)) => stream,
        Ok(Err(_)) | Err(_) => {
            bus.publish(Event::now(
                id,
                EventKind::Ended { reason: CloseReason::UpstreamUnreachable, bytes_c2s: 0, bytes_s2c: 0 },
            ));
            return;
        }
    };

    // Phase 3: split both sockets into read and write halves, so each direction can run on its own:
    //   c2s pump: client_rd   → upstream_wr
    //   s2c pump: upstream_rd → client_wr
    let (client_rd, client_wr) = client.into_split();
    let (upstream_rd, upstream_wr) = upstream.into_split();
    // Counters live on this function's stack and the pumps borrow them (`&`); this is allowed because
    // `try_join!` runs both pumps inside this function, so they cannot outlive it. Atomics allow
    // updating a value through a shared reference (`&`), which is all a pump holds; `Relaxed` ordering
    // is enough for plain counters that do not guard other data.
    let bytes_c2s = AtomicU64::new(0);
    let bytes_s2c = AtomicU64::new(0);
    let activity = Activity::new();

    let c2s = Pump { id, leg: Leg::Downstream, direction: Direction::ClientToServer, bus: &bus, bytes: &bytes_c2s, activity: &activity };
    let s2c = Pump { id, leg: Leg::Upstream, direction: Direction::ServerToClient, bus: &bus, bytes: &bytes_s2c, activity: &activity };

    // Phase 4: run both pumps concurrently (same task, no threads) until both sides close,
    // one fails, or nothing moves for `idle_timeout`; whichever happens first decides the reason
    let reason = tokio::select! {
        // try_join stops at the first error, so a reset on one side tears down the other immediately
        result = async {
            tokio::try_join!(
                c2s.run(client_rd, upstream_wr, config.buffer_size),
                s2c.run(upstream_rd, client_wr, config.buffer_size),
            )
        } => match result {
            Ok(_) => CloseReason::Closed,
            Err(e) if matches!(e.kind(), io::ErrorKind::ConnectionReset | io::ErrorKind::BrokenPipe) => CloseReason::Reset,
            Err(e) => CloseReason::Error(e.to_string()),
        },
        _ = activity.wait_idle(config.idle_timeout) => CloseReason::IdleTimeout,
    };

    // Phase 5: announce the end; consumers use it to finish decoding and release the flow's Stack
    bus.publish(Event::now(
        id,
        EventKind::Ended {
            reason,
            bytes_c2s: bytes_c2s.load(Ordering::Relaxed),
            bytes_s2c: bytes_s2c.load(Ordering::Relaxed),
        },
    ));
}

/// Copies one direction of a flow and emits a `Data` event per chunk.
///
/// `'a` is the lifetime of the borrowed bus, counter and clock: a `Pump` can never live longer
/// than the `run_flow` call that owns them.
struct Pump<'a> {
    id: FlowId,
    leg: Leg,
    direction: Direction,
    bus: &'a EventBus,
    /// Bytes seen so far in this direction; the value before a chunk is that chunk's offset.
    bytes: &'a AtomicU64,
    activity: &'a Activity,
}

impl Pump<'_> {
    async fn run(&self, mut reader: OwnedReadHalf, mut writer: OwnedWriteHalf, buffer_size: usize) -> io::Result<()> {
        let mut buf = vec![0u8; buffer_size];
        loop {
            // Waits until the socket has at least one byte; returns whatever is available right now
            // (1..=buffer_size bytes), with no relation to message boundaries. `?` ends the pump on error.
            let n = reader.read(&mut buf).await?;
            if n == 0 {
                // Half-close: pass the FIN on, but let the opposite direction keep running
                let _ = writer.shutdown().await;
                return Ok(());
            }

            // `Bytes` is reference counted: the event and the write below share one copy of the data
            let payload = Bytes::copy_from_slice(&buf[..n]);
            let offset = self.bytes.fetch_add(n as u64, Ordering::Relaxed);
            self.activity.touch();
            self.bus.publish(Event::now(
                self.id,
                EventKind::Data { leg: self.leg, direction: self.direction, offset, payload: payload.clone() },
            ));
            // In active mode this is where a Handler (core/handler.rs) would decide what gets written
            writer.write_all(&payload).await?;
        }
    }
}

/// Last time any direction of a flow carried data; shared so that one-way silence is not "idle".
struct Activity {
    start: Instant,
    last_ms: AtomicU64,
}

impl Activity {
    fn new() -> Self {
        Self { start: Instant::now(), last_ms: AtomicU64::new(0) }
    }

    fn touch(&self) {
        self.last_ms.store(self.start.elapsed().as_millis() as u64, Ordering::Relaxed);
    }

    /// Completes once no data moved for `timeout`. Sleeps until the earliest moment that could be idle,
    /// re-checks (traffic may have arrived meanwhile) and sleeps again for the remainder.
    async fn wait_idle(&self, timeout: Duration) {
        loop {
            let last = Duration::from_millis(self.last_ms.load(Ordering::Relaxed));
            let idle = self.start.elapsed().saturating_sub(last);
            if idle >= timeout {
                return;
            }
            tokio::time::sleep(timeout - idle).await;
        }
    }
}
