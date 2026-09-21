//! The signature this server puts on the mail it sends.
//!
//! Signing happens once, on the way out of the queue, so both transports carry
//! it and the signed bytes are the bytes outbound wire validation checks.
use mail_auth::{
    common::{
        crypto::{RsaKey, Sha256},
        headers::HeaderWriter,
    },
    dkim::{Canonicalization, DkimSigner, Done},
};
use rustls::pki_types::{PrivateKeyDer, pem::PemObject};

/// Each name twice, as RFC 6376 section 8.15 recommends. `mail-auth` writes a
/// name into `h=` whether or not the message carries it, so the first listing
/// covers a header added where there was none; the second covers a relay
/// adding a *second* `Cc` or `Subject` above the sender's, which a verifier
/// reading each name once from the bottom would otherwise accept.
fn signed_headers() -> impl Iterator<Item = &'static str> {
    mail_kernel::SIGNED_HEADERS
        .iter()
        .flat_map(|name| [*name, *name])
}

/// The selector and the domain go verbatim into the signature's first line,
/// which `mail-auth` never folds: a value long enough to push that line past
/// the 1000-byte outbound limit refuses *all* mail, not one message. Bounded
/// at startup to RFC 6376's sub-domain syntax.
fn sub_domain(name: &str, value: &str) -> Result<(), String> {
    let refuse = || {
        format!(
            "{name} must be dot-separated labels of letters, digits and hyphens, \
             each at most 63 bytes and at most 253 in total: {value}"
        )
    };
    if value.is_empty() || value.len() > 253 {
        return Err(refuse());
    }
    value
        .split('.')
        .all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
        .then_some(())
        .ok_or_else(refuse)
}

/// `Err` names the setting that is missing, so an operator who configured two
/// of three is told which, rather than hearing it from a receiver.
fn all_or_none(
    named: [&str; 3],
    given: [Option<String>; 3],
) -> Result<Option<[String; 3]>, String> {
    if given.iter().all(Option::is_none) {
        return Ok(None);
    }
    let mut all = Vec::new();
    for (name, value) in named.iter().zip(given) {
        all.push(value.ok_or_else(|| {
            format!(
                "{name} is required to sign outbound mail; all of MAIL_DKIM_KEY, \
                 MAIL_DKIM_DOMAIN and MAIL_DKIM_SELECTOR must be set together"
            )
        })?);
    }
    Ok(Some(<[String; 3]>::try_from(all).expect("three settings")))
}

pub(super) struct Signer(DkimSigner<RsaKey<Sha256>, Done>);

// Hand-written: this holds a private key, and a derived `Debug` would put it
// wherever a diagnostic goes.
impl std::fmt::Debug for Signer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Signer")
    }
}

impl Signer {
    /// `None` when nothing is configured. Naming one of the three settings
    /// requires all three: believing your mail is signed when it is not is
    /// worse than knowing it is not.
    pub(super) fn configured() -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let named = ["MAIL_DKIM_KEY", "MAIL_DKIM_DOMAIN", "MAIL_DKIM_SELECTOR"];
        let given = named.map(|name| std::env::var(name).ok());
        let Some([key, domain, selector]) = all_or_none(named, given)? else {
            return Ok(None);
        };
        // Before the file is read, so a bad selector is not reported as a bad key.
        sub_domain("MAIL_DKIM_DOMAIN", &domain)?;
        sub_domain("MAIL_DKIM_SELECTOR", &selector)?;
        Ok(Some(Self::from_key_file(&key, domain, selector)?))
    }

    fn from_key_file(
        key: &str,
        domain: String,
        selector: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // A failure names the file, never what was read out of it.
        let material = PrivateKeyDer::from_pem_file(key)
            .map_err(|_| format!("MAIL_DKIM_KEY is not a readable PEM private key: {key}"))?;
        let signing = RsaKey::<Sha256>::from_key_der(material).map_err(|_| {
            format!(
                "MAIL_DKIM_KEY is not an RSA key this build can sign with (2048 bits or more): {key}"
            )
        })?;
        Ok(Self(
            DkimSigner::from_key(signing)
                .domain(domain)
                .selector(selector)
                .headers(signed_headers())
                // `simple` headers stay verifiable when a relay reflows
                // nothing; `relaxed` body survives the whitespace that
                // transport still changes.
                .header_canonicalization(Canonicalization::Simple)
                .body_canonicalization(Canonicalization::Relaxed),
        ))
    }

    /// Prepended, so the signature covers the message as stored and as
    /// transmitted — the same bytes on this path.
    pub(super) fn signed(&self, raw: &[u8]) -> Result<Vec<u8>, mail_auth::Error> {
        let signature = self.0.sign(raw)?;
        let mut signed = signature.to_header().into_bytes();
        signed.extend_from_slice(raw);
        Ok(signed)
    }
}

#[cfg(test)]
mod tests;
