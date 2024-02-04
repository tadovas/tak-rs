use anyhow::{anyhow, Context};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tak_rs::protocol::xml::CotLegacyCodec;
use tak_rs::protocol::Message;
use tak_rs::tls;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::TlsConnector;
use tokio_util::codec::Framed;

pub struct TestClient {
    frames: Framed<TlsStream<TcpStream>, CotLegacyCodec>,
}

impl TestClient {
    pub async fn setup(name: &str, host: &'static str, port: u16) -> anyhow::Result<Self> {
        let tls_config = tls::setup_client_tls(tls::Config {
            ca: "tests/certs/ca.crt".to_string(),
            cert: format!("tests/certs/{name}.crt"),
            key: format!("tests/certs/{name}.key"),
        })?;
        let tls_connector = TlsConnector::from(Arc::new(tls_config));

        tokio::time::sleep(Duration::from_secs(1)).await;
        let client = TcpStream::connect((host, port)).await?;
        let conn = tls_connector.connect(host.try_into()?, client).await?;
        Ok(Self {
            frames: Framed::new(conn, CotLegacyCodec::new(10 * 1024)),
        })
    }

    pub async fn send_raw(&mut self, data: &[u8]) -> anyhow::Result<()> {
        self.frames.get_mut().write(data).await?;
        self.frames.flush().await?;
        Ok(())
    }

    pub async fn send(&mut self, msg: Message) -> anyhow::Result<()> {
        self.frames.send(msg).await?;
        Ok(())
    }

    pub async fn _expect_message(&mut self) -> anyhow::Result<Message> {
        let msg_res = tokio::select! {
            msg = self.frames.next() => msg,
            _ = tokio::time::sleep(Duration::from_millis(100)) => return Err(anyhow!("timeout waiting for message"))
        };

        let msg = msg_res.ok_or_else(|| anyhow!("expected at least one message"))?;
        let msg = msg.with_context(|| "msg parse")?;
        Ok(msg)
    }

    pub async fn shutdown(mut self) -> anyhow::Result<()>
    where
        Self: Unpin,
    {
        self.frames.get_mut().shutdown().await?;
        Ok(())
    }
}
