use tak_rs::server::{server_run, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tak_rs::tracing::init()?;
    server_run(Config { listen_port: 8089 }).await
}
