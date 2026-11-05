use crate::model::event::{Direction, Event, EventKind, FlowId, FlowMeta};

/// All events of a single flow, in arrival order, built by a consumer from the event channel.
#[derive(Debug, Clone)]
pub struct Stack {
    pub flow_id: FlowId,
    pub events: Vec<Event>,
}

impl Stack {
    pub fn new(flow_id: FlowId) -> Self {
        Self { flow_id, events: Vec::new() }
    }

    /// Appends an event; returns it back as an error if it belongs to another flow.
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
    pub fn data(&self, direction: Direction) -> impl Iterator<Item = &[u8]> {
        self.events.iter().filter_map(move |event| match &event.kind {
            EventKind::Data { direction: d, payload, .. } if *d == direction => Some(&payload[..]),
            _ => None,
        })
    }
}
