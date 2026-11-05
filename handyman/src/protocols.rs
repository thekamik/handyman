//! Protocol decoders; each one plugs into the core through `ProtocolDecoder` and a `DecoderFactory`.

pub mod http;
pub mod mqtt;

use crate::core::decoder::DecoderRegistry;

pub fn default_registry() -> DecoderRegistry {
    let mut registry = DecoderRegistry::default();
    registry.register(http::FACTORY);
    registry.register(mqtt::FACTORY);
    registry
}
