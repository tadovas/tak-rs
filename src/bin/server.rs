use tak_rs::server::{server_run, Config};
use tak_rs::tls;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tak_rs::tracing::init()?;
    server_run(Config {
        listen_port: 8089,
        tls: tls::Config {
            ca: "certs/ca.crt".to_string(),
            cert: "certs/server.crt".to_string(),
            key: "certs/key.crt".to_string(),
        },
    })
    .await
}
