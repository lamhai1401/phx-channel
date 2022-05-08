use super::errors::PhxError;
use serde::de::{Deserialize, Deserializer, Error, Unexpected, Visitor};
use serde::ser::{Serialize, Serializer};
use std::fmt;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub enum EventKind {
    Close,
    Error,
    Join,
    Leave,
    Reply,
}
