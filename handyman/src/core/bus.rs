//! READING GUIDE, step 4 of 15 (previous: core/event.rs, next: core/acceptor.rs).
//!
//! The only link between forwarding and analysis. The forwarding side calls `publish`;
//! every consumer calls `subscribe` once and loops on `recv()` (see `consumers/logger.rs::run`).

use tokio::sync::broadcast;

use crate::core::event::Event;

/// Fan-out channel for events: every subscriber receives every event.
///
/// Publishing never blocks, so slow consumers cannot slow down proxied traffic.
/// A subscriber that falls behind gets `RecvError::Lagged(n)` with the number of missed events,
/// which makes losses explicit instead of silent.
#[derive(Debug, Clone)]
pub struct EventBus {
    sender: broadcast::Sender<Event>,
}

impl EventBus {
    /// `capacity` is how many events may wait for the slowest subscriber before it starts losing them.
    pub fn new(capacity: usize) -> Self {
        // The channel also returns a first receiver; we drop it (`_`) and create receivers on demand
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn publish(&self, event: Event) {
        // An error only means nobody is subscribed right now, which is not a failure
        let _ = self.sender.send(event);
    }

    /// Subscribers only see events published after this call, so subscribe before starting the proxy.
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }
}
