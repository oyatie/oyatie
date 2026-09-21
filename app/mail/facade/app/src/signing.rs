//! The signature this server puts on the mail it sends.
//!
//! Receivers at the large providers treat an unsigned message from an unknown
//! host as suspect, so mail that leaves here unsigned is mail that arrives in
//! a spam folder. Signing happens once, on the way out of the queue, so both
//! transports carry it and the bytes still pass the wire validation every
//! outbound message is held to.
use mail_auth::{
    common::{
        crypto::{RsaKey, Sha256},
        headers::HeaderWriter,
    },
    dkim::{Canonicalization, DkimSigner, Done},
};
use rustls::pki_types::{PrivateKeyDer, pem::PemObject};

/// The headers covered by the signature. `From` is required by RFC 6376; the
/// rest are the ones a receiver weighs and a relay must not be free to alter.
/// A header named here but absent from the message is simply not signed, so
/// this list does not force any of them to exist.
const SIGNED: &[&str] = &[
    "From",
    "To",
    "Cc",
    "Subject",
    "Date",
    "Message-ID",
    "MIME-Version",
    "Content-Type",
    "Content-Transfer-Encoding",
];

/// All three settings, or none. `Err` names the one that is missing, so an
/// operator who configured two of three is told which, rather than finding
/// out from a receiver that their mail arrived unsigned.
fn complete(named: [&str; 3], given: [Option<String>; 3]) -> Result<Option<[String; 3]>, String> {
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

// Written by hand rather than derived: this holds a private key, and a
// derived Debug would put it wherever a diagnostic goes.
impl std::fmt::Debug for Signer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Signer")
    }
}

impl Signer {
    /// `None` when the operator has configured no key. Naming any one of the
    /// three settings requires all three: a half-configured signer would send
    /// unsigned mail while the operator believed otherwise, and believing
    /// your mail is signed when it is not is worse than knowing it is not.
    pub(super) fn configured() -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let named = ["MAIL_DKIM_KEY", "MAIL_DKIM_DOMAIN", "MAIL_DKIM_SELECTOR"];
        let given = named.map(|name| std::env::var(name).ok());
        let Some([key, domain, selector]) = complete(named, given)? else {
            return Ok(None);
        };
        Ok(Some(Self::from_key_file(&key, domain, selector)?))
    }

    fn from_key_file(
        key: &str,
        domain: String,
        selector: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // The key never appears in a diagnostic: a failure names the file, not
        // what was read out of it.
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
                .headers(SIGNED.iter().copied())
                // `simple` over the headers keeps the signature verifiable
                // when a relay reflows nothing; `relaxed` over the body
                // survives the whitespace changes transport still makes.
                .header_canonicalization(Canonicalization::Simple)
                .body_canonicalization(Canonicalization::Relaxed),
        ))
    }

    /// The message with its signature, or the reason it could not be signed.
    /// The header is prepended, so it covers the message as stored and as
    /// transmitted — those are the same bytes on this path.
    pub(super) fn signed(&self, raw: &[u8]) -> Result<Vec<u8>, mail_auth::Error> {
        let signature = self.0.sign(raw)?;
        let mut signed = signature.to_header().into_bytes();
        signed.extend_from_slice(raw);
        Ok(signed)
    }
}

#[cfg(test)]
mod tests;
