use crate::server::client_conn::client_loop;
use crate::tls;
use anyhow::{anyhow, Context};
use std::future::Future;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tracing::{debug, error, info, info_span, Instrument};
use x509_parser::nom::AsBytes;
use x509_parser::prelude::FromDer;

pub mod client_conn;

async fn listener_loop(listener: TcpListener, tls_acceptor: TlsAcceptor) -> anyhow::Result<()> {
    info!("Listening: {listener:?}");
    loop {
        let (stream, socket) = listener.accept().await?;
        info!("Connection from: {socket:?}");
        let tls_acceptor = tls_acceptor.clone();
        let conn_span = info_span!("client_conn", remote_sock = ?socket);
        tokio::spawn(
            check_for_error(async move {
                let stream = tls_acceptor.accept(stream).await.context("TLS accept")?;
                let (_, server_conn) = stream.get_ref();

                // accept future completion means peer certificates should be filled
                let peer_cert_chain = server_conn
                    .peer_certificates()
                    .ok_or_else(|| anyhow!("client cert chain expected"))?;
                let peer_cert = peer_cert_chain
                    .first()
                    .ok_or_else(|| anyhow!("at least 1 client cert expected"))?;

                let (_, peer_x509_cert) =
                    x509_parser::certificate::X509Certificate::from_der(peer_cert.as_bytes())?;

                let secured_conn_span = info_span!(
                    "tls",
                    subject = peer_x509_cert.subject().to_string(),
                    serial = peer_x509_cert.tbs_certificate.serial.to_string()
                );
                debug!(
                    parent: &secured_conn_span,
                    "Peer certificate: {peer_x509_cert:#?}"
                );
                client_loop(stream).instrument(secured_conn_span).await?;
                Ok(())
            })
            .instrument(conn_span),
        );
    }
}

async fn check_for_error(fut: impl Future<Output = anyhow::Result<()>>) {
    if let Err(err) = fut.await {
        error!("Client conn error: {err:?}")
    }
}

pub struct Config {
    pub listen_port: u16,
    pub tls: tls::Config,
}
pub async fn server_run(config: Config) -> anyhow::Result<()> {
    let tls_config = Arc::new(tls::setup_server_tls(config.tls)?);
    let tls_acceptor = TlsAcceptor::from(tls_config);

    let listener = TcpListener::bind(("0.0.0.0", config.listen_port)).await?;

    listener_loop(listener, tls_acceptor).await?;

    Ok(())
}
