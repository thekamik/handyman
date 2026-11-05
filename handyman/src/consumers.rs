//! Event bus consumers: everything that observes traffic without being part of forwarding.
//!
//! READING GUIDE, step 13 of 15 (previous: protocols/mqtt.rs, next: consumers/logger.rs).
//!
//! Two kinds of consumers exist: live ones that react to each event (`logger`) and ones that
//! work on complete flows handed over by the flow table (`summary`). Future recorders, exporters
//! and UIs belong here.

pub mod logger;
pub mod summary;
