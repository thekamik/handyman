use bytes::Bytes;
use std::fmt;
use std::net::SocketAddr;
use std::time::SystemTime;

/// Unique identifier of a single conversation (device ↔ server).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FlowId(pub u64);

impl fmt::Display for FlowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

/// A single observation emitted by the proxy core and consumed by analysers.
#[derive(Debug, Clone)]
pub struct Event {
    pub flow_id: FlowId,
    pub timestamp: SystemTime,
    pub kind: EventKind,
}

impl Event {
    pub fn now(flow_id: FlowId, kind: EventKind) -> Self {
        Self { flow_id, timestamp: SystemTime::now(), kind }
    }
}

#[derive(Debug, Clone)]
pub enum EventKind {
    /// Sent once per flow; later events refer to it only by `flow_id`.
    Started(FlowMeta),
    /// A chunk of raw bytes; chunk boundaries are arbitrary, not protocol messages.
    Data {
        leg: Leg,
        direction: Direction,
        /// Position of the first payload byte in this direction's stream; reveals gaps if events were dropped.
        offset: u64,
        payload: Bytes,
    },
    Ended {
        reason: CloseReason,
        bytes_c2s: u64,
        bytes_s2c: u64,
    },
}

#[derive(Debug, Clone)]
pub struct FlowMeta {
    /// Address of the connecting side (at L4 we only know its IP, not MAC).
    pub client: SocketAddr,
    /// Where the client wanted to connect, before any redirection.
    pub original_dst: SocketAddr,
    pub transport: Transport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    ClientToServer,
    ServerToClient,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Direction::ClientToServer => "c2s",
            Direction::ServerToClient => "s2c",
        })
    }
}

/// Which of the proxy's two sockets the bytes were read from.
///
/// While the proxy only forwards, client bytes are read on `Downstream` and server bytes on `Upstream`.
/// Once it modifies traffic, both legs are recorded and their difference shows what the proxy changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Leg {
    /// Socket between the device and the proxy.
    Downstream,
    /// Socket between the proxy and the real server.
    Upstream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloseReason {
    /// Orderly shutdown (FIN) from both sides.
    Closed,
    /// Connection reset (RST).
    Reset,
    /// No traffic for longer than the idle timeout.
    IdleTimeout,
    /// The original destination could not be reached; still useful as "who the device tried to talk to".
    UpstreamUnreachable,
    Error(String),
}
