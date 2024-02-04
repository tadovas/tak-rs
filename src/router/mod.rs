pub mod command;

use crate::router::command::Registration;

enum RouterMessage {
    Connected,
    Disconnected,
}

enum ClientMessage {}

pub struct Router {
    message_queue: tokio::sync::mpsc::Receiver<RouterMessage>,
    sender: tokio::sync::mpsc::Sender<RouterMessage>,
}

impl Router {
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(1_000);
        Self {
            sender: tx,
            message_queue: rx,
        }
    }

    pub fn new_registration(&self) -> Registration {
        Registration {
            router_queue: self.sender.clone(),
        }
    }
}
