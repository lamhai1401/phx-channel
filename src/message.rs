use serde_json;
use websocket::result::WebSocketResult;
use websocket::OwnedMessage;

use super::{cst::*, errors::PhxError, event::EventKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoenixMessage {
    #[serde(alias = "join", alias = "join_ref")]
    #[serde(rename = "join")]
    join_ref: Option<u32>,

    #[serde(alias = "message", alias = "message_ref")]
    #[serde(rename = "ref")]
    message_ref: usize,

    #[serde(alias = "topic", alias = "topic")]
    #[serde(rename = "topic")]
    topic: String,

    #[serde(alias = "event", alias = "event")]
    #[serde(rename = "event")]
    event: EventKind,

    #[serde(alias = "payload", alias = "payload")]
    #[serde(rename = "payload")]
    payload: serde_json::Value,
}

impl PhoenixMessage {
    pub fn new(event: &str, topic: String, message_ref: usize, payload: serde_json::Value) -> Self {
        let evt = match event {
            CHAN_REPLY => EventKind::Reply,
            CHAN_LEAVE => EventKind::Leave,
            CHAN_JOIN => EventKind::Join,
            CHAN_CLOSE => EventKind::Close,
            _ => EventKind::Error,
        };
        PhoenixMessage {
            join_ref: None,
            event: evt,
            topic,
            message_ref,
            payload,
        }
    }
}

#[derive(Debug)]
pub enum Message {
    Json(PhoenixMessage),
    Binary,
    Close,
    Ping,
    Pong,
}

impl Message {
    pub fn from_owned(owned: OwnedMessage) -> PhxError<Self> {
        let message = match owned {
            OwnedMessage::Text(text) => Message::Json(serde_json::from_str(&text)?),
            OwnedMessage::Binary(_) => Message::Binary,
            OwnedMessage::Close(_) => Message::Close,
            OwnedMessage::Ping(_) => Message::Ping,
            OwnedMessage::Pong(_) => Message::Pong,
        };
        Ok(message)
    }

    pub fn from_result(result: WebSocketResult<OwnedMessage>) -> PhxError<Self> {
        return Message::from_owned(result?);
    }
}
