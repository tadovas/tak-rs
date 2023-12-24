use anyhow::Context;
use std::future::Future;
use std::sync::Arc;
use tak_rs::client::client_loop;
use tak_rs::tls;
use tokio::io::AsyncReadExt;
use tokio_rustls::TlsAcceptor;
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tak_rs::tracing::init()?;
    let tls_config = Arc::new(tls::setup_tls()?);
    let tls_acceptor = TlsAcceptor::from(tls_config);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", 8089)).await?;
    info!("Listening: {listener:?}");

    loop {
        let (stream, socket) = listener.accept().await?;
        let tls_acceptor = tls_acceptor.clone();
        tokio::spawn(check_for_error(async move {
            info!("Connection from: {socket:?}");
            let stream = tls_acceptor.accept(stream).await.context("TLS accept")?;
            client_loop(stream).await?;
            Ok(())
        }));
    }
    Ok(())
}

async fn check_for_error(fut: impl Future<Output = anyhow::Result<()>>) {
    match fut.await {
        Err(err) => error!("Client conn error: {err:?}"),
        _ => (),
    }
}
