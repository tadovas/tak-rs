use rcgen::BasicConstraints::Unconstrained;
use rcgen::DnType::CommonName;
use rcgen::ExtendedKeyUsagePurpose::{ClientAuth, ServerAuth};
use rcgen::{
    Certificate, CertificateParams, DistinguishedName, ExtendedKeyUsagePurpose, IsCa,
    KeyUsagePurpose, SanType,
};

fn main() -> anyhow::Result<()> {
    let ca_cert = build_ca("test CA")?;
    std::fs::write("tests/certs/ca.crt", ca_cert.serialize_pem()?.as_bytes())?;

    let server_cert = build_entity(
        "test server",
        &[SanType::DnsName("localhost".to_string())],
        ServerAuth,
    )?;

    write_key_and_cert("tests/certs", "server", &server_cert, &ca_cert)?;

    let client_a_cert = build_entity("test client A", &[], ClientAuth)?;

    write_key_and_cert("tests/certs", "client_a", &client_a_cert, &ca_cert)?;

    let client_b_cert = build_entity("test client B", &[], ClientAuth)?;

    write_key_and_cert("tests/certs", "client_b", &client_b_cert, &ca_cert)?;

    Ok(())
}

fn build_ca(cn: &str) -> Result<Certificate, rcgen::Error> {
    let mut ca_params = CertificateParams::default();
    ca_params.is_ca = IsCa::Ca(Unconstrained);
    ca_params.key_usages.push(KeyUsagePurpose::DigitalSignature);
    ca_params.key_usages.push(KeyUsagePurpose::KeyCertSign);
    ca_params.key_usages.push(KeyUsagePurpose::CrlSign);
    let mut dn = DistinguishedName::new();
    dn.push(CommonName, cn);

    ca_params.distinguished_name = dn;

    Certificate::from_params(ca_params)
}

fn build_entity(
    cn: &str,
    sans: &[SanType],
    purpose: ExtendedKeyUsagePurpose,
) -> Result<Certificate, rcgen::Error> {
    let mut params = CertificateParams::default();
    params.is_ca = IsCa::NoCa;
    params.use_authority_key_identifier_extension = true;
    params.key_usages.push(KeyUsagePurpose::DigitalSignature);
    let mut dn = DistinguishedName::new();
    dn.push(CommonName, cn);

    params.distinguished_name = dn;
    params.subject_alt_names.extend_from_slice(sans);
    params.extended_key_usages.push(purpose);

    Certificate::from_params(params)
}

fn write_key_and_cert(
    path: &str,
    name: &str,
    cert: &Certificate,
    signer: &Certificate,
) -> anyhow::Result<()> {
    std::fs::write(
        format!("{path}/{name}.crt"),
        cert.serialize_pem_with_signer(signer)?.as_bytes(),
    )?;
    std::fs::write(
        format!("{path}/{name}.key"),
        cert.serialize_private_key_pem().as_bytes(),
    )?;
    Ok(())
}
