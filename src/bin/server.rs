use tak_rs::server::{Config, Server};
use tak_rs::tls;
use tracing::metadata::LevelFilter;

use tikv_jemallocator::Jemalloc;
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tak_rs::tracing::init(LevelFilter::DEBUG)?;
    let server = Server::new(Config {
        listen_port: 8089,
        tls: tls::Config {
            ca: "certs/ca.crt".to_string(),
            cert: "certs/server.crt".to_string(),
            key: "certs/server.key".to_string(),
        },
    })?;

    server.run().await
}
