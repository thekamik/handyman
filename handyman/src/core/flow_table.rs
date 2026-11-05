use std::collections::HashMap;

use tokio::sync::broadcast::{self, error::RecvError};
use tracing::warn;

use crate::core::decoder::{DecoderRegistry, DecodingSession};
use crate::core::event::{Event, EventKind, FlowId};
use crate::core::stack::Stack;

/// Builds one `Stack` per active flow from the event stream and decodes its data on the way.
pub struct FlowTable {
    registry: DecoderRegistry,
    flows: HashMap<FlowId, (Stack, DecodingSession)>,
}

impl FlowTable {
    pub fn new(registry: DecoderRegistry) -> Self {
        Self { registry, flows: HashMap::new() }
    }

    pub fn active(&self) -> usize {
        self.flows.len()
    }

    /// Applies an event; returns the flow's stack once it has ended, removing it from the table.
    pub fn apply(&mut self, event: Event) -> Option<Stack> {
        let id = event.flow_id;
        let (stack, session) = self.flows.entry(id).or_insert_with(|| {
            // A missing `Started` (lost event) still gets a stack, just without port-based decoder choice
            let port = match &event.kind {
                EventKind::Started(meta) => meta.original_dst.port(),
                _ => 0,
            };
            (Stack::new(id), DecodingSession::new(&self.registry, port))
        });

        let decoded = match &event.kind {
            EventKind::Data { direction, offset, payload, .. } => {
                session.on_data(&self.registry, *direction, *offset, payload)
            }
            EventKind::Ended { .. } => session.finish(),
            EventKind::Started(_) => Vec::new(),
        };
        stack.decoded.extend(decoded);
        stack.protocol = session.protocol();
        let ended = matches!(event.kind, EventKind::Ended { .. });
        let _ = stack.push(event); // cannot fail: the stack was looked up by this event's flow_id

        if ended {
            return self.flows.remove(&id).map(|(stack, _)| stack);
        }
        None
    }
}

/// Feeds bus events into the table and hands every finished stack to `on_finished`.
pub async fn run(mut events: broadcast::Receiver<Event>, mut table: FlowTable, mut on_finished: impl FnMut(Stack)) {
    loop {
        match events.recv().await {
            Ok(event) => {
                if let Some(stack) = table.apply(event) {
                    on_finished(stack);
                }
            }
            Err(RecvError::Lagged(missed)) => {
                warn!(missed, active = table.active(), "flow table fell behind; decoders will see gaps");
            }
            Err(RecvError::Closed) => break,
        }
    }
}
