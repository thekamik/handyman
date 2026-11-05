//! READING GUIDE, step 5 of 15 (previous: core/bus.rs, next: core/proxy.rs).
//!
//! Answers one question for the core: "a client connected; where did it want to go?".
//! `StaticAcceptor` always answers "the upstream from the command line". A future transparent
//! acceptor (gateway with iptables TPROXY / `SO_ORIGINAL_DST`) will read the real destination
//! from the kernel instead, and nothing else in the core will have to change.

use std::future::Future;
use std::io;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

/// A connection handed to the core, together with where the client actually wanted to go.
pub struct Accepted {
    pub stream: TcpStream,
    pub client: SocketAddr,
    pub original_dst: SocketAddr,
}

/// Source of incoming connections; the only part of the core that knows how traffic was intercepted.
pub trait Acceptor {
    /// Written as `-> impl Future + Send` instead of `async fn` so the returned future is guaranteed
    /// to be `Send`, which `tokio::spawn` requires to move it between worker threads.
    fn accept(&mut self) -> impl Future<Output = io::Result<Accepted>> + Send;
    fn local_addr(&self) -> io::Result<SocketAddr>;
}

/// Forwards every connection to one fixed upstream; meant for development without a gateway.
pub struct StaticAcceptor {
    listener: TcpListener,
    upstream: SocketAddr,
}

impl StaticAcceptor {
    pub async fn bind(listen: impl ToSocketAddrs, upstream: SocketAddr) -> io::Result<Self> {
        let listener = TcpListener::bind(listen).await?;
        Ok(Self { listener, upstream })
    }
}

impl Acceptor for StaticAcceptor {
    // An implementation may use plain `async fn`; it satisfies the trait's `impl Future` signature
    async fn accept(&mut self) -> io::Result<Accepted> {
        let (stream, client) = self.listener.accept().await?;
        Ok(Accepted { stream, client, original_dst: self.upstream })
    }

    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }
}
