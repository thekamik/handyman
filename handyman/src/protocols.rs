//! Protocol decoders; each one plugs into the core through `ProtocolDecoder` and a `DecoderFactory`.
//!
//! READING GUIDE, step 11 of 15 (previous: core/flow_table.rs, next: protocols/http.rs).
//!
//! Adding a protocol = one new file with a `FACTORY` and a `ProtocolDecoder` impl,
//! plus one `register` line here. The core does not change.

pub mod http;
pub mod mqtt;

use crate::core::decoder::DecoderRegistry;

pub fn default_registry() -> DecoderRegistry {
    let mut registry = DecoderRegistry::default();
    registry.register(http::FACTORY);
    registry.register(mqtt::FACTORY);
    registry
}
