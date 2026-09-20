use mail_api::MailTransport;
use mail_smtp_relay::{MxConfig, MxTransport, Relay, RelayConfig};
use rustls::{
    ClientConfig, RootCertStore,
    pki_types::{CertificateDer, pem::PemObject},
};
use std::sync::Arc;

/// The outbound transport, and what inbound sessions check about their peer.
/// Both are returned together because message authentication resolves against
/// the same servers direct delivery does, from the same resolver.
pub(super) struct Outbound {
    pub(super) transport: Option<Arc<dyn MailTransport>>,
    pub(super) authentication: mail_protocol_imap::Authentication,
}

pub(super) fn configured() -> Result<Outbound, Box<dyn std::error::Error>> {
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
        let transport = MxTransport::new(
            MxConfig {
                helo,
                dns_servers,
                require_tls,
                port: super::setting("MAIL_MX_PORT", "25")?.parse()?,
            },
            trust("MAIL_MX_CA")?,
        )?;
        let authentication = authentication(Some(transport.resolver()))?;
        return Ok(Outbound {
            transport: Some(Arc::new(transport)),
            authentication,
        });
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
            return Ok(Outbound {
                transport: None,
                authentication: authentication(None)?,
            });
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
    Ok(Outbound {
        transport: Some(Arc::new(Relay::new(
            RelayConfig {
                host,
                port,
                helo,
                implicit_tls,
                credentials,
            },
            trust,
        )?)),
        authentication: authentication(None)?,
    })
}

/// What inbound sessions check about their peer.
///
/// Every check is off unless an operator names a policy. A policy needs a
/// resolver, and the only one this process builds belongs to direct delivery,
/// so enabling a check without `MAIL_MX_DNS_SERVERS` is refused here rather
/// than silently passing every message.
fn authentication(
    resolver: Option<mail_smtp_relay::TokioResolver>,
) -> Result<mail_protocol_imap::Authentication, Box<dyn std::error::Error>> {
    let resolvable = resolver.is_some();
    let policy = |name: &str| -> Result<mail_protocol_imap::Verify, Box<dyn std::error::Error>> {
        let value = super::setting(name, "disable")?;
        let policy = mail_protocol_imap::Verify::parse(&value)
            .ok_or_else(|| format!("{name} must be disable, relaxed or strict"))?;
        if policy != mail_protocol_imap::Verify::Disabled && !resolvable {
            return Err(format!(
                "{name} needs a resolver: set MAIL_MX_DNS_SERVERS, which relay delivery does not use"
            )
            .into());
        }
        Ok(policy)
    };
    Ok(mail_protocol_imap::Authentication {
        verifier: resolver.map(|resolver| {
            mail_protocol_imap::Verifier(Arc::new(mail_auth::MessageAuthenticator(resolver)))
        }),
        // Open: a miss falls through to the resolver. Only a conformance run
        // seals the table.
        dns: Some(Arc::new(mail_protocol_imap::MailDns::default())),
        spf_ehlo: policy("MAIL_SPF_EHLO")?,
        spf_mail_from: policy("MAIL_SPF_MAIL_FROM")?,
        iprev: policy("MAIL_IPREV")?,
        log: None,
    })
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
