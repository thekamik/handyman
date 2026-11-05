//! Protocol-agnostic proxy core: knows bytes, flows and events, never a specific protocol.
//!
//! READING GUIDE, step 2 of 15 (previous: main.rs, next: core/event.rs).
//!
//! The core is split into two halves that only meet through the event bus:
//!
//! - forwarding (hot path, must never wait for analysis): acceptor → proxy → flow, publishing to bus
//! - observation (consumers, may be slow): flow_table → stack + decoder
//!
//! `event.rs` is the shared vocabulary of both halves, so it comes first.
//! `handler.rs` is the bridge planned for active mode, where forwarding will ask a decoder before passing bytes on.

pub mod acceptor;
pub mod bus;
pub mod decoder;
pub mod event;
pub mod flow;
pub mod flow_table;
pub mod handler;
pub mod proxy;
pub mod stack;
