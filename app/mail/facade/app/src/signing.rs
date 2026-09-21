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
mod tests {
    use super::*;

    /// 2048 bits, because aws-lc-rs refuses anything smaller and a shorter key
    /// would fail only at signing time. Fixed rather than generated: a keygen
    /// dependency costs about thirty crates to produce a value that never
    /// varies. Test-only; it signs nothing that leaves here.
    const TEST_KEY: &str = "-----BEGIN PRIVATE KEY-----
MIIEuwIBADANBgkqhkiG9w0BAQEFAASCBKUwggShAgEAAoIBAQDWJonSt4EOCvf7
1QbnWZ2Wa3umpCda/Ydha+uuvQ3wY4hJwBKbtctGwqTR+nM7h9VoKFTHftbQVsTm
irQ3MbsyYRiL1L9qGYBVTBGbgnXBaW4A8LyeKfUppDOj4pdaOeuMXOB9D2W2gNjO
7mEK8K1yMeMJ+j9ACgp2CqAI098Op3IOVGOrwXwujKWhIKrqb8EI82oOICbNAIEl
ePlP7JOwDHOMjxP9ICJ1m2uG0Wx/dzy23KccHkPJ6jwsyJfMWdXMFVpG5CYP8k3/
zTzINvYx5b5n5CdF3s1Ariyqy9dwZJp0xinV0+2wnuymZCqTUVSqRAxIpiGTVvpv
mNz2RR+BAgMBAAECggEADwhgh2azTAGQG+D93ZKoYc1EvlNqodQHQ1r4jekeh9/s
ysSNadnOnbZ/LHBI04Z7ABdIsEJioPheVRKqiO9YRTaUqwgxsah9nj87Qy/axUt5
2d4MV4v5dkVdDKWU21QSiWVhqtXAXZnY3lnUfRidDFWdKu+irgOmbVfcmRIKZ/RP
STNS+ho2tXkdon9xPGppmHkXijgli0ceFz+YZVtyeFTnwl1CYQirGzHccJnd+aKX
AO+dNXcBf58J1KNnGPzR+EtbAT3XX8Gaes/GktZWneV6OGU5joW1RC9uX9qlGS6H
gV1Tu6AKm4KzrWODXIzuXX5bhwIX+rfzIHKxLMj34wKBgQDr++pwyXOl8bWwZjRU
tfB4Yg0uGs92gRgJC92bKzpNti8a7EwqU5GZXH5Uf54b/yYjKGCMV5lctxgfY4yF
HGsbQSaFQZCRrfc1UML4TFtXwL5Qs/Tyo6/UIL66ICGKTuaKSwrXYj4sEPURS9ZZ
+JvmwSZ00I2DwfUsAXoxgYmILwKBgQDoUIlhAwNrsVg3xtAx0gXrQZzrcOXquqmm
5+eRLGl/cWZCKGYKK9JcQUOeDjj8cpiuTBenviK+sfd0JAIh215wziKmWytQLtfj
GewO3Mxf2tk9Pf7YRmiF4zbky4wr63Pz9NJTigz+OCwdLZTP9fraQf91SDS37SdG
duwGYg43TwKBgQCqoUVb9h3cAFDKUqxGECPnN6amDpax7hgN+nlrCC+pHzEiO3e7
Jx/hDyL9QCV3wt61jy14bKKkinMzxwxE905ur4YF1mmNHIfiEhpX5QYrBl/WBLj5
dOfe7ypZdIAr/G7v0eDt6chgnoZE0lwURBGeIU0ILAAXI/h9sDfFcg+a2wJ/LHc9
1FO/U92eBQ8IyoBooZ7taiMx3rvbvRamPCNEDiCmcgNJhKjemsnjJ12RisBkePgX
jwPVoqptss0xm0lhyjWqbC0HHVHaAJ31kOKyO6an7hDvtnXDi9zxpNlQ+xcWGpvB
pjvGQOJ/jxYqZaOvYBdBzjED5jB+U/5vLsmV7wKBgDI77NR1opuU6SpnejJK5fNs
jLzns+/wBP1r3B0q8dPL3eSuMoQhm3JWocFjXFD2YsEnMY9A0Z/M7ZY8h5ee0cK0
GULFnT1mkgy2BI04XYufq3OXwnslTSCno7HyOy5f3vr42sQ36aF+gCi7wMD3CrZb
uOv74OKArzXsO86qqdfO
-----END PRIVATE KEY-----";

    const NAMED: [&str; 3] = ["MAIL_DKIM_KEY", "MAIL_DKIM_DOMAIN", "MAIL_DKIM_SELECTOR"];

    fn key_file(name: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("mail-dkim-{name}.pem"));
        std::fs::write(&path, TEST_KEY).unwrap();
        path
    }

    #[test]
    fn the_signature_names_this_domain_and_leaves_the_message_unaltered() {
        let file = key_file("shape");
        let signer = Signer::from_key_file(
            &file.display().to_string(),
            "example.org".to_owned(),
            "default".to_owned(),
        )
        .expect("a 2048-bit key is a signer");
        let raw =
            b"From: alice@example.org\r\nTo: bob@example.net\r\nSubject: signed\r\n\r\nbody\r\n";
        let signed = signer.signed(raw).expect("a 2048-bit key signs");
        let text = String::from_utf8(signed.clone()).unwrap();
        assert!(text.starts_with("DKIM-Signature: "), "{text}");
        for field in [
            "v=1",
            "a=rsa-sha256",
            "d=example.org",
            "s=default",
            "c=simple/relaxed",
        ] {
            assert!(text.contains(field), "missing {field}: {text}");
        }
        // The header is prepended, so the bytes a receiver verifies are the
        // bytes that were signed.
        assert!(signed.ends_with(raw), "{text}");
        // Folding included, every line ends CRLF and stays inside the
        // 1000-byte limit the outbound path enforces.
        for line in text.split_inclusive("\r\n") {
            assert!(line.ends_with("\r\n"), "unterminated line: {line:?}");
            assert!(line.len() <= 1000, "line too long: {}", line.len());
        }
    }

    #[test]
    fn a_half_configured_signer_is_refused_rather_than_sending_unsigned() {
        // Believing mail is signed when it is not is worse than knowing it is
        // not, so naming one setting demands all three -- and the refusal says
        // which one is missing.
        let refusal = complete(NAMED, [Some("/key.pem".to_owned()), None, None])
            .expect_err("a key without a domain is not a signer");
        assert!(refusal.contains("MAIL_DKIM_DOMAIN"), "{refusal}");
        let refusal = complete(
            NAMED,
            [
                Some("/key.pem".to_owned()),
                Some("example.org".to_owned()),
                None,
            ],
        )
        .expect_err("a key and domain without a selector is not a signer");
        assert!(refusal.contains("MAIL_DKIM_SELECTOR"), "{refusal}");
    }

    #[test]
    fn nothing_configured_signs_nothing() {
        assert_eq!(complete(NAMED, [None, None, None]), Ok(None));
    }

    #[test]
    fn an_unreadable_key_names_the_file_and_not_its_contents() {
        let refusal = Signer::from_key_file(
            "/nonexistent/dkim.pem",
            "example.org".to_owned(),
            "default".to_owned(),
        )
        .expect_err("a missing key is not a signer")
        .to_string();
        assert!(refusal.contains("/nonexistent/dkim.pem"), "{refusal}");
        assert!(!refusal.contains("BEGIN"), "{refusal}");
    }
}
