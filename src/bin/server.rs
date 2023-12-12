use anyhow::Context;
use std::future::Future;
use std::sync::Arc;
use tak_rs::tls;
use tokio::io::AsyncReadExt;
use tokio_rustls::TlsAcceptor;
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tak_rs::tracing::init()?;
    let tls_config = Arc::new(tls::setup_tls()?);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", 8089)).await?;
    info!("Listening: {listener:?}");

    while let (stream, socket) = listener.accept().await? {
        let tls_config = tls_config.clone();
        tokio::spawn(check_for_error(async move {
            info!("Connection from: {socket:?}");
            let acceptor = TlsAcceptor::from(tls_config);
            let mut stream = acceptor.accept(stream).await.context("TLS accept")?;

            let mut buff = [0u8; 1024];
            loop {
                let readed = stream.read(&mut buff[..]).await.context("TLS read")?;
                if readed == 0 {
                    break;
                }
                let readed_buff = &buff[..readed];
                info!("Readed: {readed}");
                info!("Data: {:?}", readed_buff);
                info!("ASCII: {}", String::from_utf8_lossy(readed_buff));
            }
            info!("end of stream - aborting: {socket:?}");
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
