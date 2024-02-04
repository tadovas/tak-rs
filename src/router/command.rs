use crate::router::RouterMessage;
use crate::tls;

pub struct Registration {
    pub(super) router_queue: tokio::sync::mpsc::Sender<RouterMessage>,
}

impl Registration {
    pub async fn register_new_connection(self, _info: tls::Info) -> anyhow::Result<CommandQueue> {
        // TODO - do all the magic with sending router command, receiving response etc etc.
        Ok(CommandQueue {})
    }
}

pub trait Commands {}

pub struct CommandQueue {}

impl Commands for CommandQueue {}
