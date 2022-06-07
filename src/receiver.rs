use super::{errors::PhxError, message::Message};
use slog;
use websocket::receiver::Reader;
use websocket::stream::sync::TcpStream;

pub struct Receiver {
    logger: slog::Logger,
    reader: Reader<TcpStream>,
}

impl Receiver {
    pub fn new(reader: Reader<TcpStream>, logger: slog::Logger) -> Receiver {
        Receiver {
            logger: logger,
            reader: reader,
        }
    }
}

impl Iterator for Receiver {
    type Item = PhxError<Message>;

    fn next(&mut self) -> Option<Self::Item> {
        // convert all messages to a phoenix parsed message
        // and pass through any errors or non-json data along
        let result = self.reader.incoming_messages().next()?;
        return Some(Message::from_result(result));
    }
}
