use anyhow::Context;
use std::future::Future;
use std::sync::Arc;
use tak_rs::client::client_loop;
use tak_rs::tls;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tracing::{error, info, info_span, Instrument};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tak_rs::tracing::init()?;
    let tls_config = Arc::new(tls::setup_tls()?);
    let tls_acceptor = TlsAcceptor::from(tls_config);

    let listener = TcpListener::bind(("0.0.0.0", 8089)).await?;

    listener_loop(listener, tls_acceptor).await?;

    Ok(())
}

async fn listener_loop(listener: TcpListener, tls_acceptor: TlsAcceptor) -> anyhow::Result<()> {
    info!("Listening: {listener:?}");
    loop {
        let (stream, socket) = listener.accept().await?;
        info!("Connection from: {socket:?}");
        let tls_acceptor = tls_acceptor.clone();
        let client_span = info_span!("client_conn", remote_sock = ?socket);
        tokio::spawn(
            check_for_error(async move {
                let stream = tls_acceptor.accept(stream).await.context("TLS accept")?;
                client_loop(stream).await?;
                Ok(())
            })
            .instrument(client_span),
        );
    }
}

async fn check_for_error(fut: impl Future<Output = anyhow::Result<()>>) {
    match fut.await {
        Err(err) => error!("Client conn error: {err:?}"),
        _ => (),
    }
}
