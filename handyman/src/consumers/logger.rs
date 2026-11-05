use tokio::sync::broadcast::{self, error::RecvError};
use tracing::{debug, info, warn};

use crate::core::event::{Event, EventKind};

const PREVIEW_LEN: usize = 48;

/// Logs every event as it happens; `Data` chunks only at debug level (`RUST_LOG=debug`).
pub async fn run(mut events: broadcast::Receiver<Event>) {
    loop {
        match events.recv().await {
            Ok(event) => log_event(&event),
            Err(RecvError::Lagged(missed)) => warn!(missed, "logger fell behind"),
            Err(RecvError::Closed) => break,
        }
    }
}

fn log_event(event: &Event) {
    let flow = event.flow_id;
    match &event.kind {
        EventKind::Started(meta) => {
            info!(%flow, client = %meta.client, dst = %meta.original_dst, "flow started");
        }
        EventKind::Data { leg, direction, offset, payload } => {
            debug!(%flow, ?leg, %direction, offset, len = payload.len(), preview = %preview(payload), "data");
        }
        EventKind::Ended { reason, bytes_c2s, bytes_s2c } => {
            info!(%flow, ?reason, bytes_c2s, bytes_s2c, "flow ended");
        }
    }
}

/// First bytes of a chunk with non-printable bytes shown as '.', like a hexdump's text column.
fn preview(bytes: &[u8]) -> String {
    bytes
        .iter()
        .take(PREVIEW_LEN)
        .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' })
        .collect()
}
