use slog;
use slog::Drain;
use slog_stdlog;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use tokio::{
    task,
    time::{sleep, Duration},
};

use websocket::client::ClientBuilder;

use super::{
    errors::{Error, PhxError},
    message::Message,
    receiver::Receiver,
    sender::Sender,
};

#[derive(Debug)]
pub enum ClientError {
    // Connect(ConnectError),
    // Join(JoinError),
    Thread(String),
}

impl ToString for ClientError {
    fn to_string(&self) -> std::string::String {
        String::from(format!("{:?}", self))
    }
}

// type MessageResult = Result<Message, ClientError>;

pub fn connect(
    url: &str,
    params: Vec<(&str, &str)>,
    logger: Option<slog::Logger>,
) -> PhxError<(Sender, Receiver)> {
    let logger = logger.unwrap_or(slog::Logger::root(slog_stdlog::StdLog.fuse(), o!()));

    // convert the params to a uri component string
    let mut params_uri: String = "".to_owned();
    // for (k, v) in params {
    //     params_uri.push_str(&format!("&{}={}", k, v));
    // }

    let addr = format!("{}{}", url, params_uri);
    let mut client_builder = ClientBuilder::new(&addr)?;

    let socket_client = client_builder.connect_insecure()?;
    let (reader, writer) = socket_client.split()?;
    let sender = Sender::new(writer, logger.new(o!("type" => "sender")));
    let receiver = Receiver::new(reader, logger.new(o!("type" => "receiver")));

    Ok((sender, receiver))
}

pub struct Client {
    logger: slog::Logger,
    sender_ref: Arc<Mutex<Sender>>,
    heartbeat_handle: task::JoinHandle<()>,
    message_processor_handle: task::JoinHandle<()>,
}

impl Client {
    pub fn new(
        url: &str,
        params: Vec<(&str, &str)>,
        logger: Option<slog::Logger>,
    ) -> PhxError<(Client, mpsc::Receiver<PhxError<Message>>)> {
        let logger = logger.unwrap_or(slog::Logger::root(slog_stdlog::StdLog.fuse(), o!()));
        debug!(logger, "creating client"; "url" => url);

        let (sender, receiver) = connect(url, params, Some(logger.clone()))?;

        let (tx, rx) = mpsc::channel();

        let sender_ref = Arc::new(Mutex::new(sender));
        let heartbeat = Client::keepalive(Arc::clone(&sender_ref));
        let message_processor = Client::process_messages(receiver, tx);

        let client = Client {
            logger: logger,
            sender_ref: sender_ref,
            heartbeat_handle: heartbeat,
            message_processor_handle: message_processor,
        };

        Ok((client, rx))
    }

    fn keepalive(sender_ref: Arc<Mutex<Sender>>) -> task::JoinHandle<()> {
        task::spawn(async move {
            'v1: loop {
                sleep(Duration::from_millis(5000)).await;
                // if the mutex is poisoned then the whole thread wont work
                let mut sender = sender_ref.lock().unwrap();
                match sender.heartbeat() {
                    Ok(_) => continue,
                    Err(_) => break 'v1,
                }
            }
            println!("keepalive was closed!");
        })
    }

    fn process_messages(
        receiver: Receiver,
        sender: mpsc::Sender<PhxError<Message>>,
    ) -> task::JoinHandle<()> {
        task::spawn(async move {
            for message in MessageIterator::new(receiver) {
                let result = sender.send(message);

                // exit the thread cleanly if the channel is closed
                if result.is_err() {
                    break;
                }
            }

            println!("process_messages was closed!");
        })
    }

    pub fn join(&self, channel: &str) -> PhxError<u32> {
        match self.sender_ref.lock() {
            Ok(mut sender) => Ok(sender.join(channel)?),
            Err(_) => Err(Error::from(ClientError::Thread(String::from(
                "Cannot join as sender mutex has been poisoned",
            )))),
        }
    }

    pub fn leave(&self, channel: &str) -> PhxError<usize> {
        match self.sender_ref.lock() {
            Ok(mut sender) => Ok(sender.leave(channel)?),
            Err(_) => Err(Error::from(ClientError::Thread(String::from(
                "Cannot leave as sender mutex has been poisoned",
            )))),
        }
    }
}

pub struct MessageIterator {
    receiver: Receiver,
}

impl MessageIterator {
    pub fn new(receiver: Receiver) -> Self {
        MessageIterator { receiver: receiver }
    }
}

impl Iterator for MessageIterator {
    type Item = PhxError<Message>;

    fn next(&mut self) -> Option<Self::Item> {
        self.receiver.next()
    }
}
