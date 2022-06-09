use serde_json::{json, Value};
use slog;

use websocket::sender::Writer;
use websocket::stream::sync::TcpStream;
use websocket::OwnedMessage;

use super::{
    counter::{AtomicCounter, RelaxedCounter},
    cst::{CHAN_JOIN, CHAN_LEAVE, HEART_BEAT},
    errors::PhxError,
    message::PhoenixMessage,
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

    // join return join ref
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

    // heartbeat return msg ref
    pub fn heartbeat(&mut self) -> PhxError<usize> {
        let count = self.message_ref.inc();
        let phx_message = json![PhoenixMessage::new(
            HEART_BEAT,
            "phoenix".to_string(),
            count.clone(),
            Value::Null,
        )]
        .to_string();

        debug!(self.logger, "heartbeat()"; "payload" => &phx_message);

        let message = OwnedMessage::Text(phx_message);

        self.writer.send_message(&message)?;
        Ok(count)
    }

    pub fn leave(&mut self, channel: &str) -> PhxError<usize> {
        let count = self.message_ref.inc();
        let phx_message = json![PhoenixMessage::new(
            CHAN_LEAVE,
            channel.to_owned(),
            count.clone(),
            Value::Null,
        )]
        .to_string();

        debug!(self.logger, "leave()"; "payload" => &phx_message);

        let message = OwnedMessage::Text(phx_message);

        self.writer.send_message(&message)?;
        Ok(count)
    }
}
