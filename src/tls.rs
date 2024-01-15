use anyhow::Context;
use rustls_pemfile::Item;
use std::path::Path;
use std::sync::Arc;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivatePkcs1KeyDer};
use tokio_rustls::rustls::server::WebPkiClientVerifier;
use tokio_rustls::rustls::{ClientConfig, RootCertStore, ServerConfig};

pub fn setup_server_tls() -> anyhow::Result<ServerConfig> {
    let ca_cert: CertificateDer = read_certificate("certs/ca.crt").context("CA")?;

    let server_cert = read_certificate("certs/server.crt").context("Server cert")?;

    let key = read_private_key("certs/server.key").context("server key")?;

    let mut roots = RootCertStore::empty();
    roots.add(ca_cert).context("roots setup")?;

    ServerConfig::builder()
        .with_client_cert_verifier(
            WebPkiClientVerifier::builder(Arc::new(roots))
                .build()
                .context("verifier setup")?,
        )
        .with_single_cert(vec![server_cert], key.into())
        .context("tls config setup")
}

pub fn setup_client_tls() -> anyhow::Result<ClientConfig> {
    let ca_cert: CertificateDer = read_certificate("certs/ca.crt").context("CA")?;

    let client_cert = read_certificate("certs/server.crt").context("Server cert")?;

    let key = read_private_key("certs/server.key").context("server key")?;

    let mut roots = RootCertStore::empty();
    roots.add(ca_cert).context("roots setup")?;

    ClientConfig::builder()
        .with_root_certificates(Arc::new(roots))
        .with_client_auth_cert(vec![client_cert], key.into())
        .context("client cert setup")
}

fn read_certificate(path: impl AsRef<Path>) -> anyhow::Result<CertificateDer<'static>> {
    let item = read_pem(path)?;
    if let Item::X509Certificate(cert) = item {
        Ok(cert)
    } else {
        Err(anyhow::anyhow!("unexpected item :( {:?}", item))
    }
}

fn read_private_key(path: impl AsRef<Path>) -> anyhow::Result<PrivatePkcs1KeyDer<'static>> {
    let item = read_pem(path)?;
    if let Item::Pkcs1Key(key) = item {
        Ok(key)
    } else {
        Err(anyhow::anyhow!("unexpected item :( {:?}", item))
    }
}

fn read_pem(path: impl AsRef<Path>) -> anyhow::Result<Item> {
    let ca_cert = std::fs::read(path).context("pem read")?;

    let (item, _) = rustls_pemfile::read_one_from_slice(&ca_cert)
        .map_err(|e| anyhow::anyhow!("pem parse error: {e:?}"))?
        .expect("should be present");
    Ok(item)
}
