use bytes::Bytes;

use crate::core::decoder::Message;
use crate::core::event::FlowMeta;

/// What to do with a decoded message in active mode.
#[derive(Debug, Clone)]
pub enum Action {
    /// Pass the original bytes on unchanged.
    Forward,
    /// Swallow the message; the other side never sees it.
    Drop,
    /// Send these bytes onward instead of the original message.
    Replace(Bytes),
    /// Answer the sender directly with these bytes, without contacting the other side (e.g. cloud emulation).
    Respond(Bytes),
}

/// Behaviour hook, the counterpart of mitmproxy addons. Kept separate from the observation model:
/// events record what happened, handlers decide what to do.
///
/// Not wired into `Flow` yet: deciding per message requires decoding inline, before forwarding.
pub trait Handler: Send + Sync {
    fn on_message(&self, flow: &FlowMeta, message: &Message) -> Action;
}

/// Passive mode: forwards everything.
pub struct PassThrough;

impl Handler for PassThrough {
    fn on_message(&self, _flow: &FlowMeta, _message: &Message) -> Action {
        Action::Forward
    }
}
