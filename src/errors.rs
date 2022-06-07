use serde_json::Error as SerdeErr;
use snafu::*;
use std::io::Error as IOErr;
use websocket::client::ParseError;
use websocket::result::WebSocketError;

use super::client::ClientError;
// PhxError to handle all wss error
pub type PhxError<T> = Result<T, Error>;

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum Error {
    #[snafu(display("Wss parsing err {}", details))]
    #[non_exhaustive]
    ParseErr { details: String },

    #[snafu(display("Wss err {}", details))]
    #[non_exhaustive]
    WssErr { details: String },

    #[snafu(display("Msg err {}", details))]
    #[non_exhaustive]
    MsgErr { details: String },

    #[snafu(display("Serde err {}", details))]
    #[non_exhaustive]
    SerdeErr { details: String },

    #[snafu(display("IO err {}", details))]
    #[non_exhaustive]
    IOErr { details: String },

    #[snafu(display("Client err {}", details))]
    #[non_exhaustive]
    ClientErr { details: String },
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Error {
        Error::ParseErr {
            details: err.to_string(),
        }
    }
}

impl From<WebSocketError> for Error {
    fn from(err: WebSocketError) -> Error {
        Error::WssErr {
            details: err.to_string(),
        }
    }
}

impl From<SerdeErr> for Error {
    fn from(err: SerdeErr) -> Error {
        Error::SerdeErr {
            details: err.to_string(),
        }
    }
}

impl From<IOErr> for Error {
    fn from(err: IOErr) -> Error {
        Error::IOErr {
            details: err.to_string(),
        }
    }
}

impl From<ClientError> for Error {
    fn from(err: ClientError) -> Error {
        Error::ClientErr {
            details: err.to_string(),
        }
    }
}
