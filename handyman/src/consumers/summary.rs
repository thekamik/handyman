use tracing::{info, warn};

use crate::core::decoder::DecodeOutput;
use crate::core::event::EventKind;
use crate::core::stack::Stack;

/// Logs a finished flow: endpoints, totals and every decoded message in order.
pub fn print_stack(stack: &Stack) {
    let flow = stack.flow_id;
    let (client, dst) = stack
        .meta()
        .map(|meta| (meta.client.to_string(), meta.original_dst.to_string()))
        .unwrap_or_default();
    let (reason, bytes_c2s, bytes_s2c) = stack
        .events
        .iter()
        .find_map(|event| match &event.kind {
            EventKind::Ended { reason, bytes_c2s, bytes_s2c } => Some((format!("{reason:?}"), *bytes_c2s, *bytes_s2c)),
            _ => None,
        })
        .unwrap_or_default();

    info!(
        %flow, %client, %dst,
        protocol = stack.protocol.unwrap_or("unknown"),
        events = stack.events.len(),
        bytes_c2s, bytes_s2c, %reason,
        "flow summary"
    );
    for output in &stack.decoded {
        match output {
            DecodeOutput::Message(message) => {
                info!(%flow, direction = %message.direction, exchange = ?message.exchange, span = ?message.span, "  {}", message.summary);
            }
            DecodeOutput::Malformed { direction, span, reason, .. } => {
                warn!(%flow, %direction, ?span, "  malformed: {reason}");
            }
        }
    }
}
