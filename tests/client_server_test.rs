mod test_client;

use std::time::Duration;
use tak_rs::protocol::Message;
use tak_rs::server::{Config, Server};
use tak_rs::tls;
use tracing::info;
use tracing::metadata::LevelFilter;

const TEST_PORT: u16 = 13000;
#[tokio::test]
async fn test_client_sends_message_to_server() -> anyhow::Result<()> {
    tak_rs::tracing::init(LevelFilter::INFO)?;

    let _server_task = tokio::spawn(async {
        let server = Server::new(Config {
            listen_port: TEST_PORT,
            tls: tls::Config {
                ca: "tests/certs/ca.crt".to_string(),
                cert: "tests/certs/server.crt".to_string(),
                key: "tests/certs/server.key".to_string(),
            },
        })?;

        server.run().await
    });

    let mut client_a = test_client::TestClient::setup("client_a", "localhost", TEST_PORT).await?;

    let mut client_b = test_client::TestClient::setup("client_b", "localhost", TEST_PORT).await?;

    client_a
        .send_raw(b"<event><abc>From client A</abc></event>")
        .await?;

    client_b
        .send(Message::from_raw_xml(
            "<event><abc>From client B</abc></event>",
        )?)
        .await?;

    // no implementation yet
    // let _msg = client_b.expect_message().await?;

    client_a.shutdown().await?;
    client_b.shutdown().await?;
    info!("we done - waiting for connections to close");
    tokio::time::sleep(Duration::from_secs(1)).await;
    Ok(())
}
