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
    let Accepted { stream: client, client: client_addr, original_dst } = accepted;
    bus.publish(Event::now(
        id,
        EventKind::Started(FlowMeta { client: client_addr, original_dst, transport: Transport::Tcp }),
    ));

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

    let (client_rd, client_wr) = client.into_split();
    let (upstream_rd, upstream_wr) = upstream.into_split();
    let bytes_c2s = AtomicU64::new(0);
    let bytes_s2c = AtomicU64::new(0);
    let activity = Activity::new();

    let c2s = Pump { id, leg: Leg::Downstream, direction: Direction::ClientToServer, bus: &bus, bytes: &bytes_c2s, activity: &activity };
    let s2c = Pump { id, leg: Leg::Upstream, direction: Direction::ServerToClient, bus: &bus, bytes: &bytes_s2c, activity: &activity };

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
            let n = reader.read(&mut buf).await?;
            if n == 0 {
                // Half-close: pass the FIN on, but let the opposite direction keep running
                let _ = writer.shutdown().await;
                return Ok(());
            }

            let payload = Bytes::copy_from_slice(&buf[..n]);
            let offset = self.bytes.fetch_add(n as u64, Ordering::Relaxed);
            self.activity.touch();
            self.bus.publish(Event::now(
                self.id,
                EventKind::Data { leg: self.leg, direction: self.direction, offset, payload: payload.clone() },
            ));
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
