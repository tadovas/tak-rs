use crate::{
    client::{self, CotConnection},
    protocol::Message,
    tls,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tracing::info;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Too many clients")]
    TooManyClients,
}

pub type RouterResult<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct Router {
    max_connections: usize,
    cn_counter_map: Arc<Mutex<HashMap<String, u64>>>,
    connection_map: Arc<Mutex<HashMap<String, ()>>>,
}

impl Router {
    pub fn new(max_connections: usize) -> Self {
        Self {
            cn_counter_map: Default::default(),
            connection_map: Default::default(),
            max_connections,
        }
    }

    pub fn new_cot_connection<T>(
        &self,
        stream: T,
        tls_info: tls::Info,
    ) -> RouterResult<client::CotConnection<T>> {
        let connection_id = {
            let cn_name = tls_info.common_name.as_deref().unwrap_or("unknown");
            let mut cn_map = self.cn_counter_map.lock().expect("cn counters locked");

            let counter = cn_map.entry(cn_name.to_string()).or_default();
            *counter += 1;
            format!("{cn_name}-{counter}")
        };
        info!("Connection: {connection_id}");

        let mut connections = self.connection_map.lock().expect("connections locked");
        if connections.len() == self.max_connections {
            return Err(Error::TooManyClients);
        }

        connections.insert(connection_id.clone(), ());

        Ok(CotConnection::new(stream, connection_id, self.clone()))
    }

    pub fn cot_packet_received(
        &self,
        connection_id: &String,
        message: Message,
    ) -> RouterResult<()> {
        info!("Conn: {connection_id} sent: ${message:#?}");
        Ok(())
    }

    pub fn connection_dropped(&self, connection_id: &String) {
        info!("Connection closed: {connection_id}")
    }
}
