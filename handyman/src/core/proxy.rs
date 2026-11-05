//! READING GUIDE, step 6 of 15 (previous: core/acceptor.rs, next: core/flow.rs).
//!
//! Short on purpose: take a connection, number it, start a task for it, repeat.
//! Each task is independent, so one slow or hung device never blocks the others.

use std::io;
use tracing::{info, warn};

use crate::core::acceptor::Acceptor;
use crate::core::bus::EventBus;
use crate::core::event::FlowId;
use crate::core::flow::{FlowConfig, run_flow};

/// Accept loop: gives every connection a `FlowId` and runs it as a separate task.
///
/// Runs until the surrounding future is dropped (e.g. on Ctrl+C).
///
/// Generic over `A`: the compiler creates a separate copy of this function for each acceptor type
/// (static dispatch), so the abstraction costs nothing at runtime.
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
        // `spawn` starts the flow as an independent task and returns at once; the loop goes back to
        // `accept`. The task owns its arguments, hence the clones (cheap: the bus is a handle).
        tokio::spawn(run_flow(FlowId(next_id), accepted, bus.clone(), config.clone()));
    }
}
