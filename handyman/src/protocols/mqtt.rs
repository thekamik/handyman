//! MQTT 3.1.1 / 5.0 decoder: frames packets by their remaining-length header and extracts key fields.

use bytes::Bytes;

use crate::core::decoder::{Confidence, DecodeOutput, DecoderFactory, Field, Message, ProtocolDecoder};
use crate::core::event::Direction;

const NAME: &str = "mqtt";

pub const FACTORY: DecoderFactory = DecoderFactory { name: NAME, ports: &[1883], probe, create };

fn probe(first_bytes: &[u8]) -> Confidence {
    // A client must open with CONNECT (type 1) naming the protocol "MQTT" (3.1.1/5.0) or "MQIsdp" (3.1)
    if first_bytes.first() != Some(&0x10) {
        return Confidence::No;
    }
    match remaining_length(&first_bytes[1..]) {
        Ok(Some((_, used))) => {
            let rest = &first_bytes[1 + used..];
            if rest.starts_with(b"\x00\x04MQTT") || rest.starts_with(b"\x00\x06MQIsdp") {
                Confidence::Certain
            } else if rest.len() < 8 {
                Confidence::Maybe
            } else {
                Confidence::No
            }
        }
        Ok(None) => Confidence::Maybe,
        Err(_) => Confidence::No,
    }
}

fn create() -> Box<dyn ProtocolDecoder> {
    Box::new(MqttDecoder::default())
}

/// Decodes the variable-length integer used for "remaining length" and property lengths.
/// Returns `(value, bytes used)`, or `None` if more bytes are needed.
fn remaining_length(data: &[u8]) -> Result<Option<(usize, usize)>, String> {
    let mut value = 0;
    for (i, &byte) in data.iter().take(4).enumerate() {
        value |= usize::from(byte & 0x7f) << (7 * i);
        if byte & 0x80 == 0 {
            return Ok(Some((value, i + 1)));
        }
    }
    if data.len() >= 4 {
        Err("variable length integer longer than 4 bytes".into())
    } else {
        Ok(None)
    }
}

#[derive(Debug, Default)]
struct Side {
    buf: Vec<u8>,
    /// Stream offset of `buf[0]`.
    start: u64,
    broken: bool,
}

#[derive(Debug, Default)]
pub struct MqttDecoder {
    client: Side,
    server: Side,
    /// Protocol level from CONNECT (4 = 3.1.1, 5 = 5.0); 5.0 adds property blocks to several packets.
    level: u8,
}

impl ProtocolDecoder for MqttDecoder {
    fn name(&self) -> &'static str {
        NAME
    }

    fn feed(&mut self, direction: Direction, offset: u64, data: &[u8]) -> Vec<DecodeOutput> {
        let side = match direction {
            Direction::ClientToServer => &mut self.client,
            Direction::ServerToClient => &mut self.server,
        };
        if side.broken {
            return Vec::new();
        }
        if side.buf.is_empty() {
            side.start = offset;
        }
        side.buf.extend_from_slice(data);

        let mut out = Vec::new();
        while !side.buf.is_empty() {
            let (remaining, used) = match remaining_length(&side.buf[1..]) {
                Ok(Some(length)) => length,
                Ok(None) => break,
                Err(reason) => {
                    // Framing is lost, so nothing after this point can be trusted
                    let span = side.start..side.start + side.buf.len() as u64;
                    side.buf.clear();
                    side.broken = true;
                    out.push(DecodeOutput::Malformed { protocol: NAME, direction, span, reason });
                    break;
                }
            };
            let total = 1 + used + remaining;
            if side.buf.len() < total {
                break;
            }

            let span = side.start..side.start + total as u64;
            let output = match decode_packet(side.buf[0], &side.buf[1 + used..total], &mut self.level) {
                Ok(packet) => DecodeOutput::Message(Message {
                    protocol: NAME,
                    direction,
                    span,
                    // Packet identifiers pair PUBLISH/PUBACK, SUBSCRIBE/SUBACK etc.
                    exchange: packet.packet_id.map(u64::from),
                    summary: packet.summary,
                    fields: packet.fields,
                }),
                // Framing is still intact, so only this packet is lost and decoding continues
                Err(reason) => DecodeOutput::Malformed { protocol: NAME, direction, span, reason },
            };
            out.push(output);
            side.buf.drain(..total);
            side.start += total as u64;
        }
        out
    }

    fn reset(&mut self, direction: Direction) {
        let side = match direction {
            Direction::ClientToServer => &mut self.client,
            Direction::ServerToClient => &mut self.server,
        };
        side.buf.clear();
        side.broken = false;
    }

    fn finish(&mut self) -> Vec<DecodeOutput> {
        let mut out = Vec::new();
        for (direction, side) in [(Direction::ClientToServer, &mut self.client), (Direction::ServerToClient, &mut self.server)] {
            if !side.buf.is_empty() && !side.broken {
                out.push(DecodeOutput::Malformed {
                    protocol: NAME,
                    direction,
                    span: side.start..side.start + side.buf.len() as u64,
                    reason: "connection closed in the middle of a packet".into(),
                });
                side.buf.clear();
            }
        }
        out
    }
}

struct Packet {
    summary: String,
    fields: Vec<Field>,
    packet_id: Option<u16>,
}

fn packet_name(kind: u8) -> &'static str {
    match kind {
        1 => "CONNECT",
        2 => "CONNACK",
        3 => "PUBLISH",
        4 => "PUBACK",
        5 => "PUBREC",
        6 => "PUBREL",
        7 => "PUBCOMP",
        8 => "SUBSCRIBE",
        9 => "SUBACK",
        10 => "UNSUBSCRIBE",
        11 => "UNSUBACK",
        12 => "PINGREQ",
        13 => "PINGRESP",
        14 => "DISCONNECT",
        15 => "AUTH",
        _ => "RESERVED",
    }
}

/// Decodes one complete packet: `header` is the fixed-header byte, `body` everything after the length.
fn decode_packet(header: u8, body: &[u8], level: &mut u8) -> Result<Packet, String> {
    let kind = header >> 4;
    let name = packet_name(kind);
    let mut r = Reader::new(body);
    let mut fields = vec![Field::text("mqtt.type", name)];
    let mut packet_id = None;

    let summary = match kind {
        1 => {
            let protocol = r.string()?;
            let protocol_level = r.u8()?;
            let flags = r.u8()?;
            let keep_alive = r.u16()?;
            *level = protocol_level;
            if protocol_level >= 5 {
                r.skip_properties()?;
            }
            let client_id = r.string()?;
            fields.push(Field::text("mqtt.protocol", protocol));
            fields.push(Field::number("mqtt.level", protocol_level.into()));
            fields.push(Field::number("mqtt.keep_alive", keep_alive.into()));
            fields.push(Field::flag("mqtt.clean_session", flags & 0x02 != 0));
            let mut summary = format!("CONNECT client_id={client_id:?}");
            fields.push(Field::text("mqtt.client_id", client_id));

            if flags & 0x04 != 0 {
                if protocol_level >= 5 {
                    r.skip_properties()?;
                }
                fields.push(Field::text("mqtt.will_topic", r.string()?));
                fields.push(Field::bytes("mqtt.will_payload", r.binary()?));
            }
            if flags & 0x80 != 0 {
                let username = r.string()?;
                summary.push_str(&format!(" username={username:?}"));
                fields.push(Field::text("mqtt.username", username));
            }
            if flags & 0x40 != 0 {
                // Credentials in a plaintext CONNECT are a finding in themselves (ETSI EN 303 645, 5.1 / 5.5)
                summary.push_str(" password=<present>");
                fields.push(Field::bytes("mqtt.password", r.binary()?));
            }
            summary
        }
        2 => {
            let _ack_flags = r.u8()?;
            let code = r.u8()?;
            fields.push(Field::number("mqtt.return_code", code.into()));
            format!("CONNACK code={code}")
        }
        3 => {
            let qos = (header >> 1) & 0x03;
            let topic = r.string()?;
            if qos > 0 {
                packet_id = Some(r.u16()?);
            }
            if *level >= 5 {
                r.skip_properties()?;
            }
            let payload = r.rest();
            let summary = format!("PUBLISH topic={topic:?} qos={qos} payload={} B", payload.len());
            fields.push(Field::text("mqtt.topic", topic));
            fields.push(Field::number("mqtt.qos", qos.into()));
            fields.push(Field::flag("mqtt.retain", header & 0x01 != 0));
            fields.push(Field::flag("mqtt.dup", header & 0x08 != 0));
            fields.push(Field::bytes("mqtt.payload", payload));
            summary
        }
        4..=7 | 9 | 11 => {
            let id = r.u16()?;
            packet_id = Some(id);
            format!("{name} id={id}")
        }
        8 | 10 => {
            let id = r.u16()?;
            packet_id = Some(id);
            if *level >= 5 {
                r.skip_properties()?;
            }
            let mut topics = Vec::new();
            while !r.is_empty() {
                let topic = r.string()?;
                if kind == 8 {
                    let _options = r.u8()?;
                }
                fields.push(Field::text("mqtt.topic", topic.clone()));
                topics.push(topic);
            }
            format!("{name} id={id} topics={topics:?}")
        }
        12..=15 => name.to_string(),
        _ => return Err(format!("reserved packet type {kind}")),
    };

    if let Some(id) = packet_id {
        fields.push(Field::number("mqtt.packet_id", id.into()));
    }
    Ok(Packet { summary, fields, packet_id })
}

/// Cursor over a packet body; every read fails cleanly on truncated data instead of panicking.
struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn is_empty(&self) -> bool {
        self.pos >= self.data.len()
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self
            .pos
            .checked_add(n)
            .filter(|&end| end <= self.data.len())
            .ok_or_else(|| "packet truncated".to_string())?;
        let bytes = &self.data[self.pos..end];
        self.pos = end;
        Ok(bytes)
    }

    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, String> {
        let bytes = self.take(2)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    /// Length-prefixed binary data.
    fn binary(&mut self) -> Result<Bytes, String> {
        let len = self.u16()?;
        Ok(Bytes::copy_from_slice(self.take(len.into())?))
    }

    /// Length-prefixed UTF-8 string; invalid sequences are replaced rather than rejected.
    fn string(&mut self) -> Result<String, String> {
        let len = self.u16()?;
        Ok(String::from_utf8_lossy(self.take(len.into())?).into_owned())
    }

    /// Skips an MQTT 5.0 property block (variable-length size followed by the properties).
    fn skip_properties(&mut self) -> Result<(), String> {
        match remaining_length(&self.data[self.pos..])? {
            Some((len, used)) => {
                self.pos += used;
                self.take(len)?;
                Ok(())
            }
            None => Err("property length truncated".into()),
        }
    }

    fn rest(&mut self) -> Bytes {
        let bytes = Bytes::copy_from_slice(&self.data[self.pos..]);
        self.pos = self.data.len();
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::decoder::FieldValue;

    /// CONNECT, MQTT 3.1.1, client id "dev1", username "admin", password "secret".
    const CONNECT: &[u8] = b"\x10\x1f\x00\x04MQTT\x04\xc2\x00\x3c\x00\x04dev1\x00\x05admin\x00\x06secret";

    #[test]
    fn probe_recognises_connect() {
        assert_eq!(probe(CONNECT), Confidence::Certain);
        assert_eq!(probe(b"GET / HTTP/1.1"), Confidence::No);
    }

    #[test]
    fn connect_with_credentials_split_across_chunks() {
        let mut decoder = MqttDecoder::default();
        assert!(decoder.feed(Direction::ClientToServer, 0, &CONNECT[..10]).is_empty());
        let outputs = decoder.feed(Direction::ClientToServer, 10, &CONNECT[10..]);
        let [DecodeOutput::Message(message)] = &outputs[..] else { panic!("expected one message: {outputs:?}") };
        assert_eq!(message.span, 0..CONNECT.len() as u64);
        assert_eq!(message.field("mqtt.username"), Some(&FieldValue::Text("admin".into())));
        assert_eq!(message.field("mqtt.password"), Some(&FieldValue::Bytes(Bytes::from_static(b"secret"))));
    }

    #[test]
    fn publish_qos1_and_puback_share_exchange() {
        let mut decoder = MqttDecoder::default();
        decoder.feed(Direction::ClientToServer, 0, CONNECT);
        let publish = b"\x32\x0c\x00\x05a/b/c\x00\x07hi!";
        let outputs = decoder.feed(Direction::ClientToServer, CONNECT.len() as u64, publish);
        let [DecodeOutput::Message(publish)] = &outputs[..] else { panic!("{outputs:?}") };
        assert_eq!(publish.field("mqtt.topic"), Some(&FieldValue::Text("a/b/c".into())));
        assert_eq!(publish.field("mqtt.payload"), Some(&FieldValue::Bytes(Bytes::from_static(b"hi!"))));

        let outputs = decoder.feed(Direction::ServerToClient, 0, b"\x40\x02\x00\x07");
        let [DecodeOutput::Message(puback)] = &outputs[..] else { panic!("{outputs:?}") };
        assert_eq!((publish.exchange, puback.exchange), (Some(7), Some(7)));
    }

    #[test]
    fn truncated_packet_body_is_malformed_but_decoding_continues() {
        // CONNACK claiming 1 byte of body (needs 2), followed by a valid PINGRESP
        let outputs = MqttDecoder::default().feed(Direction::ServerToClient, 0, b"\x20\x01\x00\xd0\x00");
        assert!(matches!(outputs[0], DecodeOutput::Malformed { .. }));
        assert!(matches!(&outputs[1], DecodeOutput::Message(m) if m.summary == "PINGRESP"));
    }
}
