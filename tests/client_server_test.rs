use std::sync::Arc;
use std::time::Duration;
use tak_rs::server::{server_run, Config};
use tak_rs::tls;
use tokio::io::AsyncWriteExt;
use tokio_rustls::TlsConnector;
use tracing::info;

const TEST_PORT: u16 = 13000;
#[tokio::test]
async fn test_client_sends_message_to_server() -> anyhow::Result<()> {
    tak_rs::tracing::init()?;

    let _server_task = tokio::spawn(async {
        server_run(Config {
            listen_port: TEST_PORT,
            tls: tls::Config {
                ca: "certs/ca.crt".to_string(),
                cert: "certs/server.crt".to_string(),
                key: "certs/server.key".to_string(),
            },
        })
        .await
    });

    let tls_config = tls::setup_client_tls(tls::Config {
        ca: "certs/ca.crt".to_string(),
        cert: "certs/client.crt".to_string(),
        key: "certs/client.key".to_string(),
    })?;
    let tls_connector = TlsConnector::from(Arc::new(tls_config));

    tokio::time::sleep(Duration::from_secs(1)).await;
    let client = tokio::net::TcpStream::connect(("127.0.0.1", TEST_PORT)).await?;
    let mut client = tls_connector
        .connect("192.168.1.110".try_into()?, client)
        .await?;

    client.write(b"<event><abc></abc></event>").await?;
    client.flush().await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    client.shutdown().await?;
    info!("we done");
    Ok(())
}
