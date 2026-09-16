use mail_api::MailTransport;
use mail_smtp_relay::{MxConfig, MxTransport, Relay, RelayConfig};
use rustls::{
    ClientConfig, RootCertStore,
    pki_types::{CertificateDer, pem::PemObject},
};
use std::sync::Arc;

pub(super) fn configured() -> Result<Option<Arc<dyn MailTransport>>, Box<dyn std::error::Error>> {
    if [
        "MAIL_MX_DNS_SERVERS",
        "MAIL_MX_HELO",
        "MAIL_MX_PORT",
        "MAIL_MX_CA",
        "MAIL_MX_REQUIRE_TLS",
    ]
    .iter()
    .any(|name| std::env::var_os(name).is_some())
    {
        let servers = std::env::var("MAIL_MX_DNS_SERVERS")
            .map_err(|_| "MAIL_MX_DNS_SERVERS is required for direct delivery")?;
        if [
            "MAIL_RELAY_HOST",
            "MAIL_RELAY_PORT",
            "MAIL_RELAY_HELO",
            "MAIL_RELAY_CA",
            "MAIL_RELAY_USERNAME",
            "MAIL_RELAY_PASSWORD",
            "MAIL_RELAY_STARTTLS",
        ]
        .iter()
        .any(|name| std::env::var_os(name).is_some())
        {
            return Err("MX and relay settings are mutually exclusive".into());
        }
        let dns_servers = servers
            .split(',')
            .map(|s| s.trim().parse())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "MAIL_MX_DNS_SERVERS contains an invalid socket address")?;
        let helo = std::env::var("MAIL_MX_HELO")
            .map_err(|_| "MAIL_MX_HELO is required for direct delivery")?;
        let require_tls = match super::setting("MAIL_MX_REQUIRE_TLS", "true")?.as_str() {
            "true" => true,
            "false" => false,
            _ => return Err("MAIL_MX_REQUIRE_TLS must be true or false".into()),
        };
        return Ok(Some(Arc::new(MxTransport::new(
            MxConfig {
                helo,
                dns_servers,
                require_tls,
                port: super::setting("MAIL_MX_PORT", "25")?.parse()?,
            },
            trust("MAIL_MX_CA")?,
        )?)));
    }
    let host = match std::env::var("MAIL_RELAY_HOST") {
        Ok(host) => host,
        Err(std::env::VarError::NotPresent) => {
            if [
                "MAIL_RELAY_PORT",
                "MAIL_RELAY_HELO",
                "MAIL_RELAY_CA",
                "MAIL_RELAY_USERNAME",
                "MAIL_RELAY_PASSWORD",
                "MAIL_RELAY_STARTTLS",
            ]
            .iter()
            .any(|name| std::env::var_os(name).is_some())
            {
                return Err("MAIL_RELAY_HOST is required when relay settings are supplied".into());
            }
            return Ok(None);
        }
        Err(error) => return Err(error.into()),
    };
    let implicit_tls = match super::setting("MAIL_RELAY_STARTTLS", "false")?.as_str() {
        "true" => false,
        "false" => true,
        _ => return Err("MAIL_RELAY_STARTTLS must be true or false".into()),
    };
    let port =
        super::setting("MAIL_RELAY_PORT", if implicit_tls { "465" } else { "587" })?.parse()?;
    let helo = super::setting("MAIL_RELAY_HELO", "mail.localhost.invalid")?;
    let credentials = match (
        std::env::var("MAIL_RELAY_USERNAME"),
        std::env::var("MAIL_RELAY_PASSWORD"),
    ) {
        (Ok(user), Ok(pass)) => Some((user, pass)),
        (Err(std::env::VarError::NotPresent), Err(std::env::VarError::NotPresent)) => None,
        _ => {
            return Err("MAIL_RELAY_USERNAME and MAIL_RELAY_PASSWORD must both be supplied".into());
        }
    };
    let trust = trust("MAIL_RELAY_CA")?;
    Ok(Some(Arc::new(Relay::new(
        RelayConfig {
            host,
            port,
            helo,
            implicit_tls,
            credentials,
        },
        trust,
    )?)))
}

fn trust(name: &str) -> Result<Arc<ClientConfig>, Box<dyn std::error::Error>> {
    let ca = match std::env::var(name) {
        Ok(path) => path,
        Err(std::env::VarError::NotPresent) => [
            "/etc/ssl/certs/ca-certificates.crt",
            "/etc/ssl/cert.pem",
            "/etc/pki/tls/certs/ca-bundle.crt",
        ]
        .iter()
        .find(|p| std::path::Path::new(p).is_file())
        .ok_or_else(|| format!("{name} must name a trusted PEM CA bundle"))?
        .to_string(),
        Err(error) => return Err(error.into()),
    };
    let mut roots = RootCertStore::empty();
    for certificate in CertificateDer::pem_file_iter(ca)? {
        roots.add(certificate?)?;
    }
    if roots.is_empty() {
        return Err(format!("{name} contains no certificates").into());
    }
    Ok(Arc::new(
        ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth(),
    ))
}
