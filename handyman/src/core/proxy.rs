use std::io;
use tracing::{info, warn};

use crate::core::acceptor::Acceptor;
use crate::core::bus::EventBus;
use crate::core::event::FlowId;
use crate::core::flow::{FlowConfig, run_flow};

/// Accept loop: gives every connection a `FlowId` and runs it as a separate task.
///
/// Runs until the surrounding future is dropped (e.g. on Ctrl+C).
pub async fn run<A: Acceptor>(mut acceptor: A, bus: EventBus, config: FlowConfig) -> io::Result<()> {
    info!(addr = %acceptor.local_addr()?, "proxy listening");

    let mut next_id = 0;
    loop {
        let accepted = match acceptor.accept().await {
            Ok(accepted) => accepted,
            Err(e) => {
                // A failed accept affects only one client, so keep serving others
                warn!("failed to accept connection: {e}");
                continue;
            }
        };

        next_id += 1;
        tokio::spawn(run_flow(FlowId(next_id), accepted, bus.clone(), config.clone()));
    }
}
