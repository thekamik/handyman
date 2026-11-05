//! READING GUIDE, step 8 of 15 (previous: core/flow.rs, next: core/decoder.rs).
//!
//! We now switch from the forwarding half to the observation half.
//! A `Stack` is what a consumer builds for one flow by collecting its events from the bus.
//! It is filled by `core/flow_table.rs` (step 10); `decoded` is filled by the decoders (step 9).

use crate::core::decoder::DecodeOutput;
use crate::core::event::{Direction, Event, EventKind, FlowId, FlowMeta};

/// All events of a single flow, in arrival order, built by a consumer from the event channel.
///
/// `events` are the source of truth; `decoded` is derived from them and can always be rebuilt
/// by feeding the events through a decoder again.
#[derive(Debug, Clone)]
pub struct Stack {
    pub flow_id: FlowId,
    pub events: Vec<Event>,
    /// Name of the decoder chosen for this flow, if any.
    pub protocol: Option<&'static str>,
    /// Messages (and malformed byte ranges) reconstructed from the raw `Data` events.
    pub decoded: Vec<DecodeOutput>,
}

impl Stack {
    pub fn new(flow_id: FlowId) -> Self {
        Self { flow_id, events: Vec::new(), protocol: None, decoded: Vec::new() }
    }

    /// Appends an event; returns it back as an error if it belongs to another flow.
    ///
    /// Returning the rejected event (instead of dropping it) lets the caller decide what to do with it.
    pub fn push(&mut self, event: Event) -> Result<(), Event> {
        if event.flow_id != self.flow_id {
            return Err(event);
        }
        self.events.push(event);
        Ok(())
    }

    /// Flow metadata from the `Started` event, if it has been received.
    pub fn meta(&self) -> Option<&FlowMeta> {
        self.events.iter().find_map(|event| match &event.kind {
            EventKind::Started(meta) => Some(meta),
            _ => None,
        })
    }

    /// True once the `Ended` event has been received.
    pub fn is_finished(&self) -> bool {
        self.events
            .iter()
            .any(|event| matches!(event.kind, EventKind::Ended { .. }))
    }

    /// Payload chunks sent in one direction, in order.
    /// Lazy: nothing is copied, the iterator walks `events` only when consumed,
    /// e.g. `stack.data(Direction::ClientToServer).map(|chunk| chunk.len()).sum::<usize>()`.
    pub fn data(&self, direction: Direction) -> impl Iterator<Item = &[u8]> {
        self.events.iter().filter_map(move |event| match &event.kind {
            EventKind::Data { direction: d, payload, .. } if *d == direction => Some(&payload[..]),
            _ => None,
        })
    }
}
