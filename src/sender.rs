use serde_json::{json, Value};
use slog;

use std::sync::atomic::AtomicUsize;
use websocket::sender::Writer;
use websocket::stream::sync::TcpStream;
use websocket::OwnedMessage;

use super::{
    counter::{AtomicCounter, RelaxedCounter},
    cst::CHAN_JOIN,
    errors::{Error, PhxError},
    message::{Message, PhoenixMessage},
};

pub struct Sender {
    logger: slog::Logger,
    writer: Writer<TcpStream>,
    join_ref: u32,
    message_ref: RelaxedCounter,
}

impl Sender {
    pub fn new(writer: Writer<TcpStream>, logger: slog::Logger) -> Sender {
        Sender {
            logger: logger,
            writer: writer,
            join_ref: 0,
            message_ref: RelaxedCounter::new(0),
        }
    }

    pub fn join(&mut self, channel: &str) -> PhxError<u32> {
        self.join_ref += 1;
        let phx_message = json![PhoenixMessage::new(
            CHAN_JOIN,
            channel.to_owned(),
            self.message_ref.inc(),
            Value::Null,
        )]
        .to_string();

        debug!(self.logger, "join()"; "payload" => &phx_message);
        let message = OwnedMessage::Text(phx_message);
        self.writer.send_message(&message)?;
        return Ok(self.join_ref);
    }
}
