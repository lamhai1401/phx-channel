use super::cst::*;
use serde::de::{Deserialize, Deserializer, Error, Unexpected, Visitor};
use serde::ser::{Serialize, Serializer};
use std::fmt;

#[derive(Debug, Clone)]
pub enum EventKind {
    Close,
    Error,
    Join,
    Leave,
    Reply,
    HeartBeat,
}

impl Serialize for EventKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let kind = match *self {
            EventKind::Close => CHAN_CLOSE,
            EventKind::Error => CHAN_ERROR,
            EventKind::Join => CHAN_JOIN,
            EventKind::Leave => CHAN_LEAVE,
            EventKind::Reply => CHAN_REPLY,
            EventKind::HeartBeat => HEART_BEAT,
        };

        serializer.serialize_str(kind)
    }
}

impl<'de> Deserialize<'de> for EventKind {
    fn deserialize<D>(deserializer: D) -> Result<EventKind, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FieldVisitor;

        impl<'de> Visitor<'de> for FieldVisitor {
            type Value = EventKind;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a string with a value of [phx_close, phx_error, phx_join, phx_leave, phx_reply]")
            }

            fn visit_str<E>(self, value: &str) -> Result<EventKind, E>
            where
                E: Error,
            {
                match value {
                    CHAN_CLOSE => Ok(EventKind::Close),
                    CHAN_ERROR => Ok(EventKind::Error),
                    CHAN_JOIN => Ok(EventKind::Join),
                    CHAN_LEAVE => Ok(EventKind::Leave),
                    CHAN_REPLY => Ok(EventKind::Reply),
                    HEART_BEAT => Ok(EventKind::HeartBeat),
                    s => Err(E::invalid_value(Unexpected::Str(s), &self)),
                }
            }
        }
        deserializer.deserialize_str(FieldVisitor)
    }
}
