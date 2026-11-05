//! HTTP/1.x decoder: splits each direction into requests/responses and pairs them by order.
//!
//! READING GUIDE, step 11 of 15, continued (previous: protocols.rs, next: protocols/mqtt.rs).
//!
//! Read in the order data flows through it:
//! 1. `FACTORY`, `probe`: how the registry recognises HTTP.
//! 2. `HttpDecoder` + `Side`: one buffer per direction plus the queue of unanswered request methods.
//! 3. `feed` → `drain` → `next_message`: append a chunk, then cut out every complete message.
//! 4. `request_framing` / `response_framing` / `chunked_len`: HTTP's rules for "where does the body end".
//! 5. `build_message`: turning a parsed message into generic `Field`s and pairing responses with requests.
//! 6. The tests at the bottom: each one is a small, readable example of input and expected output.

use std::collections::VecDeque;

use bytes::Bytes;

use crate::core::decoder::{Confidence, DecodeOutput, DecoderFactory, Field, Message, ProtocolDecoder};
use crate::core::event::Direction;

// What one direction of an HTTP/1.1 connection looks like on the wire:
//
//   GET /index.html HTTP/1.1\r\n        ← start line (method, target, version)
//   Host: example.com\r\n               ← headers, one per line
//   Content-Length: 5\r\n
//   \r\n                                ← empty line: end of the "head"
//   hello                               ← body; its length comes from the headers ("framing")
//   GET /next HTTP/1.1\r\n               ← the next request may follow immediately (keep-alive)
//
// The decoder therefore works in two steps: find "\r\n\r\n" to get the head, then use the headers
// to know how many body bytes to wait for. See `Framing` for the three possibilities.

const NAME: &str = "http";
/// Header sections larger than this are reported as malformed instead of being buffered forever.
const MAX_HEAD_LEN: usize = 64 * 1024;
const METHODS: [&[u8]; 9] = [
    b"GET ", b"POST ", b"PUT ", b"HEAD ", b"DELETE ", b"OPTIONS ", b"PATCH ", b"CONNECT ", b"TRACE ",
];

pub const FACTORY: DecoderFactory = DecoderFactory { name: NAME, ports: &[80, 8000, 8080], probe, create };

fn probe(first_bytes: &[u8]) -> Confidence {
    if METHODS.iter().any(|m| first_bytes.starts_with(m)) {
        Confidence::Certain
    } else if first_bytes.starts_with(b"HTTP/1.") {
        Confidence::Maybe
    } else {
        Confidence::No
    }
}

fn create() -> Box<dyn ProtocolDecoder> {
    Box::new(HttpDecoder::default())
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum SideState {
    #[default]
    Active,
    /// Unparseable data seen; ignore the rest of this direction until a reset.
    Broken,
    /// After `101 Switching Protocols` or `CONNECT`, the stream is no longer HTTP.
    Tunnel,
}

/// Parsing state of one direction.
#[derive(Debug, Default)]
struct Side {
    buf: Vec<u8>,
    /// Stream offset of `buf[0]`.
    start: u64,
    state: SideState,
    /// Final messages seen so far; doubles as the exchange number of the next one.
    messages: u64,
}

#[derive(Debug, Default)]
pub struct HttpDecoder {
    requests: Side,
    responses: Side,
    /// Methods of requests still waiting for a response; HTTP/1.1 answers strictly in order.
    pending_methods: VecDeque<String>,
}

struct Head {
    /// Request: method, target, version. Response: version, status, reason.
    start_line: [String; 3],
    headers: Vec<(String, String)>,
}

/// How the end of a body is found.
enum Framing {
    /// `Content-Length: N` (or no body at all: N = 0).
    Fixed(usize),
    /// `Transfer-Encoding: chunked`: the body is a series of `<hex size>\r\n<data>\r\n`, ending with size 0.
    /// Used when the sender does not know the length in advance (streamed or generated content).
    Chunked,
    /// Body runs until the server closes the connection.
    UntilClose,
}

/// Result of one attempt to cut a message from the front of a buffer.
enum Step {
    Complete(Message),
    NeedMore,
    Invalid(String),
}

impl ProtocolDecoder for HttpDecoder {
    fn name(&self) -> &'static str {
        NAME
    }

    fn feed(&mut self, direction: Direction, offset: u64, data: &[u8]) -> Vec<DecodeOutput> {
        let side = self.side_mut(direction);
        if side.state != SideState::Active {
            return Vec::new();
        }
        if side.buf.is_empty() {
            side.start = offset;
        }
        side.buf.extend_from_slice(data);
        self.drain(direction, false)
    }

    fn reset(&mut self, direction: Direction) {
        let side = self.side_mut(direction);
        side.buf.clear();
        if side.state == SideState::Broken {
            side.state = SideState::Active;
        }
        // Request/response pairing cannot be trusted after a gap
        self.pending_methods.clear();
    }

    fn finish(&mut self) -> Vec<DecodeOutput> {
        let mut out = Vec::new();
        for direction in [Direction::ClientToServer, Direction::ServerToClient] {
            if self.side_mut(direction).state != SideState::Active {
                continue;
            }
            out.extend(self.drain(direction, true));
            let side = self.side_mut(direction);
            if !side.buf.is_empty() {
                let span = side.start..side.start + side.buf.len() as u64;
                side.buf.clear();
                out.push(DecodeOutput::Malformed {
                    protocol: NAME,
                    direction,
                    span,
                    reason: "connection closed in the middle of a message".into(),
                });
            }
        }
        out
    }
}

impl HttpDecoder {
    fn side_mut(&mut self, direction: Direction) -> &mut Side {
        match direction {
            Direction::ClientToServer => &mut self.requests,
            Direction::ServerToClient => &mut self.responses,
        }
    }

    /// Extracts every complete message from the buffer; `eof` allows bodies delimited by close.
    fn drain(&mut self, direction: Direction, eof: bool) -> Vec<DecodeOutput> {
        let mut out = Vec::new();
        loop {
            match self.next_message(direction, eof) {
                Step::NeedMore => break,
                Step::Complete(message) => out.push(DecodeOutput::Message(message)),
                Step::Invalid(reason) => {
                    let side = self.side_mut(direction);
                    let span = side.start..side.start + side.buf.len() as u64;
                    side.start = span.end;
                    side.buf.clear();
                    side.state = SideState::Broken;
                    out.push(DecodeOutput::Malformed { protocol: NAME, direction, span, reason });
                    break;
                }
            }
            if self.side_mut(direction).state != SideState::Active {
                break;
            }
        }
        out
    }

    /// Tries to cut exactly one message from the front of `direction`'s buffer.
    /// Nothing is removed from the buffer unless a whole message is there (`NeedMore` keeps it intact).
    fn next_message(&mut self, direction: Direction, eof: bool) -> Step {
        let side = match direction {
            Direction::ClientToServer => &self.requests,
            Direction::ServerToClient => &self.responses,
        };
        if side.buf.is_empty() {
            return Step::NeedMore;
        }

        // Step 1: the head is complete once the empty line has arrived
        let Some(head_len) = find(&side.buf, b"\r\n\r\n").map(|pos| pos + 4) else {
            return if side.buf.len() > MAX_HEAD_LEN {
                Step::Invalid("header section too large".into())
            } else {
                Step::NeedMore
            };
        };
        let head = match parse_head(&side.buf[..head_len]) {
            Ok(head) => head,
            Err(reason) => return Step::Invalid(reason),
        };
        // Step 2: headers tell where the body ends; a response also depends on its request (HEAD has no body)
        let framing = match direction {
            Direction::ClientToServer => request_framing(&head),
            Direction::ServerToClient => response_framing(&head, self.pending_methods.front()),
        };
        let framing = match framing {
            Ok(framing) => framing,
            Err(reason) => return Step::Invalid(reason),
        };

        // Step 3: is the whole body here yet?
        let body = &side.buf[head_len..];
        let body_len = match framing {
            Framing::Fixed(len) => (body.len() >= len).then_some(len),
            Framing::Chunked => match chunked_len(body) {
                Ok(len) => len,
                Err(reason) => return Step::Invalid(reason),
            },
            Framing::UntilClose => eof.then_some(body.len()),
        };
        let Some(body_len) = body_len else {
            return Step::NeedMore;
        };

        // Step 4: build the message, then remove its bytes from the buffer and advance the offset
        let total = head_len + body_len;
        let span = side.start..side.start + total as u64;
        let body = Bytes::copy_from_slice(&side.buf[head_len..total]);
        let message = self.build_message(direction, head, body, span);

        let side = self.side_mut(direction);
        side.buf.drain(..total);
        side.start += total as u64;
        Step::Complete(message)
    }

    fn build_message(&mut self, direction: Direction, head: Head, body: Bytes, span: std::ops::Range<u64>) -> Message {
        let [first, second, third] = head.start_line;
        let mut fields = Vec::new();

        let (summary, exchange) = match direction {
            Direction::ClientToServer => {
                let exchange = self.requests.messages;
                self.requests.messages += 1;
                self.pending_methods.push_back(first.clone());
                let summary = format!("{first} {second} {third}");
                fields.push(Field::text("http.method", first));
                fields.push(Field::text("http.target", second));
                fields.push(Field::text("http.version", third));
                (summary, exchange)
            }
            Direction::ServerToClient => {
                let status: u16 = second.parse().unwrap_or(0); // validated in response_framing
                let exchange = self.responses.messages;
                // Interim responses (100 Continue...) precede the final one of the same exchange
                if !(100..200).contains(&status) || status == 101 {
                    self.responses.messages += 1;
                    let method = self.pending_methods.pop_front();
                    let tunnel = status == 101 || (method.as_deref() == Some("CONNECT") && (200..300).contains(&status));
                    if tunnel {
                        self.requests.state = SideState::Tunnel;
                        self.responses.state = SideState::Tunnel;
                    }
                }
                let summary = format!("{first} {second} {third}").trim_end().to_string();
                fields.push(Field::text("http.version", first));
                fields.push(Field::number("http.status", status.into()));
                fields.push(Field::text("http.reason", third));
                (summary, exchange)
            }
        };

        for (name, value) in head.headers {
            fields.push(Field::text("http.header", format!("{name}: {value}")));
        }
        fields.push(Field::number("http.body_len", body.len() as u64));
        if !body.is_empty() {
            // Body as it was on the wire (still chunked/compressed if it was sent that way)
            fields.push(Field::bytes("http.body", body));
        }

        Message { protocol: NAME, direction, span, exchange: Some(exchange), summary, fields }
    }
}

/// Position of the first occurrence of `needle`; a simple linear scan is fine for headers.
fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

/// Splits a complete head into the start line and header pairs. Lossy UTF-8: odd bytes become '�'
/// instead of failing, because a proxy must tolerate whatever devices send.
fn parse_head(raw: &[u8]) -> Result<Head, String> {
    let text = String::from_utf8_lossy(raw);
    let mut lines = text.split("\r\n");
    let first = lines.next().unwrap_or_default();
    let mut parts = first.splitn(3, ' ');
    let start_line = [(); 3].map(|_| parts.next().unwrap_or_default().to_string());
    if start_line[0].is_empty() || start_line[1].is_empty() {
        return Err(format!("invalid start line {first:?}"));
    }

    let mut headers = Vec::new();
    for line in lines.filter(|line| !line.is_empty()) {
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| format!("invalid header line {line:?}"))?;
        // Original name casing and order are kept on purpose
        headers.push((name.to_string(), value.trim().to_string()));
    }
    Ok(Head { start_line, headers })
}

fn header_values<'a>(head: &'a Head, name: &'a str) -> impl Iterator<Item = &'a str> {
    head.headers
        .iter()
        .filter(move |(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

fn content_length(head: &Head) -> Result<Option<usize>, String> {
    let mut result = None;
    for value in header_values(head, "Content-Length") {
        let len = value
            .parse::<usize>()
            .map_err(|_| format!("invalid Content-Length {value:?}"))?;
        // Differing duplicates are a classic request smuggling vector, so refuse to guess
        if result.is_some_and(|previous| previous != len) {
            return Err("conflicting Content-Length headers".into());
        }
        result = Some(len);
    }
    Ok(result)
}

fn is_chunked(head: &Head) -> bool {
    header_values(head, "Transfer-Encoding").any(|v| v.to_ascii_lowercase().contains("chunked"))
}

fn request_framing(head: &Head) -> Result<Framing, String> {
    if !head.start_line[2].starts_with("HTTP/") {
        return Err("not an HTTP request line".into());
    }
    if is_chunked(head) {
        return Ok(Framing::Chunked);
    }
    Ok(Framing::Fixed(content_length(head)?.unwrap_or(0)))
}

/// Response bodies have more rules than request bodies: some statuses never have one, a response
/// to HEAD never has one (even with Content-Length), and without any length the body runs until close.
fn response_framing(head: &Head, request_method: Option<&String>) -> Result<Framing, String> {
    if !head.start_line[0].starts_with("HTTP/") {
        return Err("not an HTTP status line".into());
    }
    let status: u16 = head.start_line[1]
        .parse()
        .map_err(|_| format!("invalid status code {:?}", head.start_line[1]))?;
    let bodiless = (100..200).contains(&status)
        || status == 204
        || status == 304
        || request_method.is_some_and(|m| m == "HEAD");
    if bodiless {
        return Ok(Framing::Fixed(0));
    }
    if is_chunked(head) {
        return Ok(Framing::Chunked);
    }
    Ok(match content_length(head)? {
        Some(len) => Framing::Fixed(len),
        None => Framing::UntilClose,
    })
}

/// Wire length of a chunked body, or `None` if it is not complete yet.
///
/// Example: `5\r\nhello\r\n0\r\n\r\n` has wire length 15 and carries the 5-byte body "hello".
/// Returns `Ok(None)` while incomplete, so the caller waits for more data.
fn chunked_len(body: &[u8]) -> Result<Option<usize>, String> {
    let mut pos = 0;
    loop {
        let Some(line_len) = find(&body[pos..], b"\r\n") else {
            return Ok(None);
        };
        let line = std::str::from_utf8(&body[pos..pos + line_len]).map_err(|_| "invalid chunk size line".to_string())?;
        let size_text = line.split(';').next().unwrap_or_default().trim();
        let size = usize::from_str_radix(size_text, 16).map_err(|_| format!("invalid chunk size {size_text:?}"))?;
        pos += line_len + 2;

        if size == 0 {
            // Optional trailer headers, then an empty line
            if body.len() < pos + 2 {
                return Ok(None);
            }
            if &body[pos..pos + 2] == b"\r\n" {
                return Ok(Some(pos + 2));
            }
            return Ok(find(&body[pos..], b"\r\n\r\n").map(|end| pos + end + 4));
        }

        let end = pos.checked_add(size).ok_or("chunk size overflow")?;
        if body.len() < end + 2 {
            return Ok(None);
        }
        if &body[end..end + 2] != b"\r\n" {
            return Err("chunk not terminated by CRLF".into());
        }
        pos = end + 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::decoder::FieldValue;

    fn messages(outputs: Vec<DecodeOutput>) -> Vec<Message> {
        outputs
            .into_iter()
            .map(|o| match o {
                DecodeOutput::Message(m) => m,
                DecodeOutput::Malformed { reason, .. } => panic!("unexpected malformed: {reason}"),
            })
            .collect()
    }

    #[test]
    fn request_split_byte_by_byte() {
        let raw = b"POST /form HTTP/1.1\r\nHost: x\r\nContent-Length: 5\r\n\r\nhello";
        let mut decoder = HttpDecoder::default();
        let mut outputs = Vec::new();
        for (i, byte) in raw.iter().enumerate() {
            outputs.extend(decoder.feed(Direction::ClientToServer, i as u64, &[*byte]));
        }
        let msgs = messages(outputs);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].summary, "POST /form HTTP/1.1");
        assert_eq!(msgs[0].span, 0..raw.len() as u64);
        assert_eq!(msgs[0].field("http.body"), Some(&FieldValue::Bytes(Bytes::from_static(b"hello"))));
    }

    #[test]
    fn two_pipelined_requests_in_one_chunk() {
        let raw = b"GET /a HTTP/1.1\r\n\r\nGET /b HTTP/1.1\r\n\r\n";
        let mut decoder = HttpDecoder::default();
        let msgs = messages(decoder.feed(Direction::ClientToServer, 0, raw));
        assert_eq!(msgs.len(), 2);
        assert_eq!((msgs[0].exchange, msgs[1].exchange), (Some(0), Some(1)));
        assert_eq!(msgs[1].span.start, 19);
    }

    #[test]
    fn chunked_response_and_head_without_body() {
        let mut decoder = HttpDecoder::default();
        decoder.feed(Direction::ClientToServer, 0, b"HEAD / HTTP/1.1\r\n\r\nGET / HTTP/1.1\r\n\r\n");
        let raw = b"HTTP/1.1 200 OK\r\nContent-Length: 99\r\n\r\n\
                    HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabc\r\n0\r\n\r\n";
        let msgs = messages(decoder.feed(Direction::ServerToClient, 0, raw));
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].field("http.body_len"), Some(&FieldValue::Number(0)));
        // "3\r\nabc\r\n0\r\n\r\n": the wire length, chunk framing included
        assert_eq!(msgs[1].field("http.body_len"), Some(&FieldValue::Number(13)));
    }

    #[test]
    fn conflicting_content_length_is_malformed() {
        let raw = b"POST / HTTP/1.1\r\nContent-Length: 1\r\nContent-Length: 2\r\n\r\nab";
        let outputs = HttpDecoder::default().feed(Direction::ClientToServer, 0, raw);
        assert!(matches!(&outputs[..], [DecodeOutput::Malformed { .. }]));
    }

    #[test]
    fn body_until_close_is_flushed_on_finish() {
        let mut decoder = HttpDecoder::default();
        decoder.feed(Direction::ClientToServer, 0, b"GET / HTTP/1.0\r\n\r\n");
        assert!(decoder.feed(Direction::ServerToClient, 0, b"HTTP/1.0 200 OK\r\n\r\nabc").is_empty());
        let msgs = messages(decoder.finish());
        assert_eq!(msgs[0].field("http.body_len"), Some(&FieldValue::Number(3)));
    }
}
