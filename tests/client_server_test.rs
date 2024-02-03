use std::sync::Arc;
use std::time::Duration;
use tak_rs::server::{server_run, Config};
use tak_rs::tls;
use tokio::io::AsyncWriteExt;
use tokio_rustls::TlsConnector;
use tracing::info;
use tracing::metadata::LevelFilter;

const TEST_PORT: u16 = 13000;
#[tokio::test]
async fn test_client_sends_message_to_server() -> anyhow::Result<()> {
    tak_rs::tracing::init(LevelFilter::INFO)?;

    let _server_task = tokio::spawn(async {
        server_run(Config {
            listen_port: TEST_PORT,
            tls: tls::Config {
                ca: "tests/certs/ca.crt".to_string(),
                cert: "tests/certs/server.crt".to_string(),
                key: "tests/certs/server.key".to_string(),
            },
        })
        .await
    });

    let tls_config = tls::setup_client_tls(tls::Config {
        ca: "tests/certs/ca.crt".to_string(),
        cert: "tests/certs/client_a.crt".to_string(),
        key: "tests/certs/client_a.key".to_string(),
    })?;
    let tls_connector = TlsConnector::from(Arc::new(tls_config));

    tokio::time::sleep(Duration::from_secs(1)).await;
    let client = tokio::net::TcpStream::connect(("localhost", TEST_PORT)).await?;
    let mut client = tls_connector
        .connect("localhost".try_into()?, client)
        .await?;

    client.write(b"<event><abc></abc></event>").await?;
    client.flush().await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    client.shutdown().await?;
    info!("we done");
    Ok(())
}
