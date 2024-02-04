use crate::router::Router;
use crate::server::client_conn::ClientConnection;
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

async fn check_for_error(fut: impl Future<Output = anyhow::Result<()>>) {
    if let Err(err) = fut.await {
        error!("Client conn error: {err:?}")
    }
}

pub struct Config {
    pub listen_port: u16,
    pub tls: tls::Config,
}

pub struct Server {
    tls_acceptor: TlsAcceptor,
    socket_addr: (&'static str, u16),
    router: Router,
}

impl Server {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let tls_config = Arc::new(tls::setup_server_tls(config.tls)?);
        let tls_acceptor = TlsAcceptor::from(tls_config);
        Ok(Self {
            tls_acceptor,
            socket_addr: ("0.0.0.0", config.listen_port),
            router: Router::new(),
        })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(self.socket_addr).await?;

        self.listener_loop(listener).await?;

        Ok(())
    }

    async fn listener_loop(&self, listener: TcpListener) -> anyhow::Result<()> {
        info!("Listening: {listener:?}");
        loop {
            let (stream, socket) = listener.accept().await?;
            info!("Connection from: {socket:?}");
            let tls_acceptor = self.tls_acceptor.clone();
            let conn_span = info_span!("client_conn", remote_sock = ?socket);

            let new_registration = self.router.new_registration();

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

                    debug!("Peer certificate: {peer_x509_cert:#?}");

                    let peer: tls::Info = peer_x509_cert.into();
                    let secured_conn_span =
                        info_span!("tls", subject = peer.common_name, serial = peer.serial);

                    let command_queue = new_registration.register_new_connection(peer).await?;

                    ClientConnection::new(stream, command_queue)
                        .conn_loop()
                        .instrument(secured_conn_span)
                        .await?;
                    Ok(())
                })
                .instrument(conn_span),
            );
        }
    }
}
