use snafu::*;
use websocket::client::ParseError;
use websocket::result::WebSocketError;

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum Error {
    #[snafu(display("Wss parsing err {}", details))]
    #[non_exhaustive]
    ParseErr { details: String },

    #[snafu(display("Wss err {}", details))]
    #[non_exhaustive]
    WssErr { details: String },
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
