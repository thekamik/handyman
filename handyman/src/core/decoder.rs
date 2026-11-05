use std::ops::Range;

use bytes::Bytes;

use crate::core::event::Direction;

/// How sure a decoder is that a flow speaks its protocol, judging by the first bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    No,
    Maybe,
    Certain,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    Text(String),
    Number(u64),
    Flag(bool),
    Bytes(Bytes),
}

/// One named protocol field; names are global like in Wireshark ("http.method", "mqtt.topic"),
/// so rules can work across protocols without knowing their structures.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: &'static str,
    pub value: FieldValue,
}

impl Field {
    pub fn text(name: &'static str, value: impl Into<String>) -> Self {
        Self { name, value: FieldValue::Text(value.into()) }
    }

    pub fn number(name: &'static str, value: u64) -> Self {
        Self { name, value: FieldValue::Number(value) }
    }

    pub fn flag(name: &'static str, value: bool) -> Self {
        Self { name, value: FieldValue::Flag(value) }
    }

    pub fn bytes(name: &'static str, value: Bytes) -> Self {
        Self { name, value: FieldValue::Bytes(value) }
    }
}

/// A complete protocol message reconstructed from raw chunks.
///
/// The core keeps messages generic (a field list) so it never depends on concrete protocols;
/// raw bytes stay available through `span` and the flow's `Data` events.
#[derive(Debug, Clone)]
pub struct Message {
    pub protocol: &'static str,
    pub direction: Direction,
    /// Stream bytes (in this direction) the message was decoded from.
    pub span: Range<u64>,
    /// Groups a request with its response; how it is assigned is protocol-specific.
    pub exchange: Option<u64>,
    pub summary: String,
    /// Fields in wire order; a name may repeat (e.g. several "http.header").
    pub fields: Vec<Field>,
}

impl Message {
    pub fn field(&self, name: &str) -> Option<&FieldValue> {
        self.fields.iter().find(|f| f.name == name).map(|f| &f.value)
    }

    pub fn fields_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a FieldValue> {
        self.fields.iter().filter(move |f| f.name == name).map(|f| &f.value)
    }
}

#[derive(Debug, Clone)]
pub enum DecodeOutput {
    Message(Message),
    /// Bytes that do not follow the protocol; kept because odd traffic is often the interesting part.
    Malformed {
        protocol: &'static str,
        direction: Direction,
        span: Range<u64>,
        reason: String,
    },
}

/// Turns the raw byte streams of one flow into protocol messages; one instance per flow.
pub trait ProtocolDecoder: Send {
    fn name(&self) -> &'static str;

    /// Feeds the next chunk of one direction; `offset` is the stream position of `data[0]`.
    /// Returns every message this chunk completed (often none, sometimes several).
    fn feed(&mut self, direction: Direction, offset: u64, data: &[u8]) -> Vec<DecodeOutput>;

    /// Drops buffered state of one direction, e.g. after a gap caused by lost events.
    fn reset(&mut self, direction: Direction);

    /// Called when the flow ends, for messages delimited by connection close and leftover bytes.
    fn finish(&mut self) -> Vec<DecodeOutput> {
        Vec::new()
    }
}

/// Registration entry for a protocol: how to recognise it and how to create its decoder.
#[derive(Clone, Copy)]
pub struct DecoderFactory {
    pub name: &'static str,
    /// Well-known server ports; checked before probing.
    pub ports: &'static [u16],
    pub probe: fn(&[u8]) -> Confidence,
    pub create: fn() -> Box<dyn ProtocolDecoder>,
}

#[derive(Clone, Default)]
pub struct DecoderRegistry {
    factories: Vec<DecoderFactory>,
}

impl DecoderRegistry {
    pub fn register(&mut self, factory: DecoderFactory) {
        self.factories.push(factory);
    }

    pub fn by_port(&self, port: u16) -> Option<&DecoderFactory> {
        self.factories.iter().find(|f| f.ports.contains(&port))
    }

    /// The most confident decoder for these first bytes, if any is at least `Maybe`.
    pub fn probe(&self, first_bytes: &[u8]) -> Option<&DecoderFactory> {
        self.factories
            .iter()
            .map(|f| ((f.probe)(first_bytes), f))
            .filter(|(confidence, _)| *confidence >= Confidence::Maybe)
            .max_by_key(|(confidence, _)| *confidence)
            .map(|(_, f)| f)
    }
}

/// Decoding state of one flow: picks a decoder, detects gaps and feeds chunks to it.
pub struct DecodingSession {
    decoder: Option<Box<dyn ProtocolDecoder>>,
    probed: bool,
    /// Next expected offset per direction (index 0 = c2s, 1 = s2c).
    expected: [u64; 2],
}

impl DecodingSession {
    pub fn new(registry: &DecoderRegistry, server_port: u16) -> Self {
        let decoder = registry.by_port(server_port).map(|f| (f.create)());
        Self { probed: decoder.is_some(), decoder, expected: [0, 0] }
    }

    pub fn protocol(&self) -> Option<&'static str> {
        self.decoder.as_ref().map(|d| d.name())
    }

    pub fn on_data(&mut self, registry: &DecoderRegistry, direction: Direction, offset: u64, data: &[u8]) -> Vec<DecodeOutput> {
        if !self.probed {
            // No port match: let the first bytes of the flow decide, once
            self.probed = true;
            self.decoder = registry.probe(data).map(|f| (f.create)());
        }
        let Some(decoder) = self.decoder.as_mut() else {
            return Vec::new();
        };

        let slot = match direction {
            Direction::ClientToServer => 0,
            Direction::ServerToClient => 1,
        };
        if offset != self.expected[slot] {
            // Bytes were lost; continuing would glue unrelated data together and decode garbage
            decoder.reset(direction);
        }
        self.expected[slot] = offset + data.len() as u64;
        decoder.feed(direction, offset, data)
    }

    pub fn finish(&mut self) -> Vec<DecodeOutput> {
        self.decoder.as_mut().map(|d| d.finish()).unwrap_or_default()
    }
}
