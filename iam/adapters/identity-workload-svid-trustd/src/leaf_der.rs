//! Real X.509 leaf-DER parsing + verification (G002 slice-1b-i; ADR-0561).
//!
//! Replaces the TSV1 stand-in codec: a presented peer leaf is now REAL ASN.1
//! X.509 DER (minted by `os-trustd-domain::der` via rcgen), and this
//! module parses + verifies it with `x509-parser` on the `aws-lc-rs` backend
//! (`verify-aws`; NO ring, ADR-0506). It is the body the kernel
//! [`SvidVerifier::verify_peer`] port delegates to.
//!
//! Verification is total and fail-closed. A leaf is trusted ONLY when ALL hold:
//!   1. it parses as a single well-formed X.509 certificate (no trailing bytes);
//!   2. its real signature verifies under ONE of the trust bundle's CA public
//!      keys (AWS-LC ECDSA verify — a forged/rogue-CA leaf fails here);
//!   3. it is within its validity window at `now` (an expired leaf is a distinct
//!      DENY);
//!   4. it is a WORKLOAD leaf, not a CA: its basicConstraints `cA` is FALSE (a
//!      CA-capable leaf must never authenticate as a workload — defence in depth
//!      against a mis-issued or compromised CA presenting itself as a caller);
//!   5. it carries the `clientAuth` Extended Key Usage (a caller leaf is a TLS
//!      client; a leaf without clientAuth is not authorized for caller auth);
//!   6. it carries EXACTLY ONE SPIFFE URI SAN (zero → unauthenticated, more than
//!      one → ambiguous identity).
//!
//! The extracted URI is then handed to the kernel `SpiffeId` parser.
//!
//! [`SvidVerifier::verify_peer`]: iam_identity_workload_svid_kernel::SvidVerifier::verify_peer

use x509_parser::certificate::X509Certificate;
use x509_parser::extensions::GeneralName;
use x509_parser::prelude::FromDer;

/// The outcome classes of parsing+verifying a real leaf DER, mapped 1:1 by the
/// adapter onto the kernel `VerifyError` variants (kept here so the parse module
/// owns no kernel dependency in its return surface beyond the URI string).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeafVerifyError {
    /// The bytes did not parse as a single well-formed X.509 certificate.
    Undecodable(String),
    /// The signature did not verify under any trusted CA public key.
    UntrustedIssuer(String),
    /// The leaf was outside its validity window at the verification instant.
    Expired,
    /// The leaf is CA-capable (basicConstraints `cA` TRUE) — a CA must never
    /// authenticate as a workload caller (defence in depth).
    CaCapableLeaf,
    /// The leaf does not carry the `clientAuth` Extended Key Usage required of a
    /// caller leaf (no EKU extension, or an EKU set without clientAuth).
    MissingClientAuthEku,
    /// No SPIFFE URI SAN was present.
    NoUriSan,
    /// More than one URI SAN was present (a SVID carries exactly one).
    AmbiguousUriSan,
}

/// Parse `leaf_der`, verify its real signature against `trusted_ca_spki_ders`,
/// check validity at `now` (unix seconds), and return the SINGLE URI SAN string.
///
/// Order of checks is fail-closed and deterministic: decode → signature/chain →
/// validity → basicConstraints (not a CA) → clientAuth EKU → URI-SAN cardinality.
/// The returned `String` is the raw URI the caller parses into a `SpiffeId`.
///
/// # Errors
/// [`LeafVerifyError`] for every reject path.
pub fn verify_leaf_der(
    leaf_der: &[u8],
    trusted_ca_spki_ders: &[Vec<u8>],
    now: u64,
) -> Result<String, LeafVerifyError> {
    // 1. Parse exactly one certificate; reject trailing bytes (a real leaf is a
    //    single DER SEQUENCE — extra bytes are malformed input).
    let (rest, cert) = X509Certificate::from_der(leaf_der)
        .map_err(|e| LeafVerifyError::Undecodable(format!("DER parse failed: {e}")))?;
    if !rest.is_empty() {
        return Err(LeafVerifyError::Undecodable(format!(
            "{} trailing bytes after the leaf certificate",
            rest.len()
        )));
    }

    // 2. Verify the real signature against each trusted CA SubjectPublicKeyInfo.
    //    A leaf from a CA not in the bundle (forged/rogue) never verifies here.
    let verified = trusted_ca_spki_ders.iter().any(|spki| {
        match x509_parser::x509::SubjectPublicKeyInfo::from_der(spki) {
            // A well-formed anchor SPKI is a single DER object with no trailing
            // bytes; reject a malformed anchor rather than verifying against it.
            Ok((spki_rest, ca_spki)) => {
                spki_rest.is_empty() && cert.verify_signature(Some(&ca_spki)).is_ok()
            }
            Err(_) => false,
        }
    });
    if !verified {
        return Err(LeafVerifyError::UntrustedIssuer(
            "leaf signature did not verify under any trusted CA".to_string(),
        ));
    }

    // 3. Validity window: not_before <= now < not_after (unix seconds).
    let not_before = cert.validity().not_before.timestamp();
    let not_after = cert.validity().not_after.timestamp();
    let now_i = i64::try_from(now).unwrap_or(i64::MAX);
    if now_i < not_before || now_i >= not_after {
        return Err(LeafVerifyError::Expired);
    }

    // 4. basicConstraints: a CALLER leaf must NOT be a CA. A `cA == true` leaf
    //    (or a leaf whose basicConstraints extension is malformed/duplicated) is
    //    rejected — a CA-capable certificate must never authenticate as a
    //    workload caller, even if its chain + validity check out (defence in
    //    depth against a mis-issued or compromised CA). Absence of the extension
    //    means `cA` is FALSE (RFC 5280), which is acceptable for a leaf.
    match cert.basic_constraints() {
        Ok(Some(bc)) if bc.value.ca => return Err(LeafVerifyError::CaCapableLeaf),
        Ok(_) => {}
        Err(_) => return Err(LeafVerifyError::CaCapableLeaf),
    }

    // 5. Extended Key Usage: a caller leaf is a TLS client, so it MUST carry the
    //    `clientAuth` EKU. A missing EKU extension, an EKU set without clientAuth,
    //    or a malformed/duplicated EKU extension is rejected (fail-closed).
    match cert.extended_key_usage() {
        Ok(Some(eku)) if eku.value.client_auth => {}
        Ok(_) => return Err(LeafVerifyError::MissingClientAuthEku),
        Err(_) => return Err(LeafVerifyError::MissingClientAuthEku),
    }

    // 6. Extract the SINGLE URI SAN (the SPIFFE id carrier).
    let uris = uri_sans(&cert)?;
    match uris.as_slice() {
        [] => Err(LeafVerifyError::NoUriSan),
        [single] => Ok(single.clone()),
        _ => Err(LeafVerifyError::AmbiguousUriSan),
    }
}

/// Extract the SINGLE SPIFFE URI SAN from a leaf DER WITHOUT verifying its chain
/// or validity. This is for material the caller already trusts and only needs to
/// READ the identity of — specifically the PDP reading its OWN server leaf to
/// derive its cell authority (the server leaf was already validated by rustls at
/// boot). It is NOT a verification path: never use it on a presented peer leaf
/// (use [`verify_leaf_der`] for that).
///
/// # Errors
/// [`LeafVerifyError::Undecodable`] when the bytes do not parse as a single
/// certificate; [`LeafVerifyError::NoUriSan`] / [`LeafVerifyError::AmbiguousUriSan`]
/// when the leaf carries zero / more than one URI SAN.
pub fn extract_single_uri_san(leaf_der: &[u8]) -> Result<String, LeafVerifyError> {
    let (rest, cert) = X509Certificate::from_der(leaf_der)
        .map_err(|e| LeafVerifyError::Undecodable(format!("DER parse failed: {e}")))?;
    if !rest.is_empty() {
        return Err(LeafVerifyError::Undecodable(format!(
            "{} trailing bytes after the certificate",
            rest.len()
        )));
    }
    match uri_sans(&cert)?.as_slice() {
        [] => Err(LeafVerifyError::NoUriSan),
        [single] => Ok(single.clone()),
        _ => Err(LeafVerifyError::AmbiguousUriSan),
    }
}

/// Collect every `uniformResourceIdentifier` GeneralName from the leaf's
/// SubjectAlternativeName extension. A leaf with no SAN extension yields an empty
/// vec (→ `NoUriSan` upstream); a malformed/duplicated SAN extension is treated
/// as undecodable.
fn uri_sans(cert: &X509Certificate<'_>) -> Result<Vec<String>, LeafVerifyError> {
    let san = cert
        .subject_alternative_name()
        .map_err(|e| LeafVerifyError::Undecodable(format!("malformed SAN extension: {e}")))?;
    let Some(san) = san else {
        return Ok(Vec::new());
    };
    Ok(san
        .value
        .general_names
        .iter()
        .filter_map(|gn| match gn {
            GeneralName::URI(uri) => Some((*uri).to_string()),
            _ => None,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use os_trustd_domain::ca::{CertificateAuthority, CertificateSigningRequest};
    use os_trustd_domain::der;
    use os_trustd_domain::signer::EcdsaP256Signer;
    use os_trustd_domain::x509::KeyPair;

    const URI: &str = "spiffe://oyatie.cell-7/tenant/ten_acme/secrets-sync";

    fn real_ca() -> (CertificateAuthority<EcdsaP256Signer>, EcdsaP256Signer) {
        let signer = EcdsaP256Signer::generate().unwrap();
        let ca_key = KeyPair::new(signer.private_key_der(), signer.public_key_spki_der());
        let ca = CertificateAuthority::bootstrap(
            "oyatie-cell-7-ca",
            ca_key,
            signer.clone(),
            1_000,
            10_000_000,
        )
        .unwrap();
        (ca, signer)
    }

    fn issue(ca: &mut CertificateAuthority<EcdsaP256Signer>, ca_signer: &EcdsaP256Signer, uri: &str)
        -> (Vec<u8>, Vec<u8>)
    {
        let wl = EcdsaP256Signer::generate().unwrap();
        let wl_key = KeyPair::new(wl.private_key_der(), wl.public_key_spki_der());
        let csr = CertificateSigningRequest::for_workload("wl", uri, &wl_key, 3_600);
        let leaf = ca.sign_csr(&csr, 2_000).unwrap();
        let der = der::encode_leaf_der(&leaf, &wl, ca.certificate(), ca_signer).unwrap();
        let ca_spki = ca.certificate().public_key_der.clone();
        (der, ca_spki)
    }

    #[test]
    fn real_leaf_verifies_and_yields_uri() {
        let (mut ca, sgn) = real_ca();
        let (leaf, ca_spki) = issue(&mut ca, &sgn, URI);
        let got = verify_leaf_der(&leaf, &[ca_spki], 2_500).unwrap();
        assert_eq!(got, URI);
    }

    #[test]
    fn forged_leaf_from_untrusted_ca_fails_signature() {
        let (mut rogue, rogue_sgn) = real_ca();
        let (forged, _rogue_spki) = issue(&mut rogue, &rogue_sgn, URI);
        // Trust only a DIFFERENT real CA.
        let (good, _g) = real_ca();
        let good_spki = good.certificate().public_key_der.clone();
        let err = verify_leaf_der(&forged, &[good_spki], 2_500).unwrap_err();
        assert!(matches!(err, LeafVerifyError::UntrustedIssuer(_)));
    }

    #[test]
    fn expired_leaf_is_distinct_deny() {
        let (mut ca, sgn) = real_ca();
        let (leaf, ca_spki) = issue(&mut ca, &sgn, URI); // valid [2000,5600)
        assert_eq!(
            verify_leaf_der(&leaf, &[ca_spki], 6_000).unwrap_err(),
            LeafVerifyError::Expired
        );
    }

    #[test]
    fn post_signature_uri_tamper_breaks_verification() {
        let (mut ca, sgn) = real_ca();
        let (leaf, ca_spki) = issue(&mut ca, &sgn, URI);
        // Flip a byte inside the DER (which covers the signed TBS incl. the URI
        // SAN). A REAL signature — not a MAC — fails, or the DER no longer parses.
        let mut tampered = leaf.clone();
        let i = tampered.len() / 3;
        tampered[i] ^= 0xFF;
        let res = verify_leaf_der(&tampered, &[ca_spki], 2_500);
        assert!(res.is_err(), "tampered real-DER leaf must not verify");
    }

    #[test]
    fn undecodable_bytes_are_rejected() {
        let err = verify_leaf_der(b"not-a-der-cert", &[vec![1, 2, 3]], 2_500).unwrap_err();
        assert!(matches!(err, LeafVerifyError::Undecodable(_)));
    }

    #[test]
    fn workload_leaf_carries_clientauth_and_is_not_ca() {
        // Non-regression: a normal workload SVID leaf (for_workload ⇒ ClientAuth,
        // basicConstraints cA FALSE) passes the new basicConstraints + EKU gates.
        let (mut ca, sgn) = real_ca();
        let (leaf, ca_spki) = issue(&mut ca, &sgn, URI);
        assert_eq!(verify_leaf_der(&leaf, &[ca_spki], 2_500).unwrap(), URI);
    }

    #[test]
    fn ca_capable_leaf_is_rejected() {
        // A CA-capable cert (basicConstraints cA TRUE) signed by the TRUSTED CA,
        // carrying a valid SPIFFE URI SAN, must NOT authenticate as a workload —
        // defence in depth: a CA must never act as a caller.
        use os_trustd_domain::certificate::{CertUsage, Certificate};
        use os_trustd_domain::x509::{
            DistinguishedName, SubjectAltNames, Validity,
        };
        let (ca, sgn) = real_ca();
        let leaf_signer = EcdsaP256Signer::generate().unwrap();
        let ca_leaf = Certificate {
            serial: 42,
            subject: DistinguishedName::common("rogue-sub-ca"),
            issuer: DistinguishedName::common("oyatie-cell-7-ca"),
            validity: Validity { not_before: 1_000, not_after: 9_000 },
            usage: CertUsage::CertificateAuthority,
            sans: SubjectAltNames { uris: vec![URI.to_string()], ..Default::default() },
            public_key_der: leaf_signer.public_key_spki_der(),
            signature: vec![0x01],
        };
        let der_bytes =
            der::encode_leaf_der(&ca_leaf, &leaf_signer, ca.certificate(), &sgn).unwrap();
        let ca_spki = ca.certificate().public_key_der.clone();
        assert_eq!(
            verify_leaf_der(&der_bytes, &[ca_spki], 2_500).unwrap_err(),
            LeafVerifyError::CaCapableLeaf
        );
    }

    #[test]
    fn leaf_without_clientauth_eku_is_rejected() {
        // A serverAuth-only leaf (for_node ServerAuth ⇒ no clientAuth EKU) from the
        // TRUSTED CA, carrying a SPIFFE URI SAN, must NOT authenticate as a caller.
        use os_trustd_domain::certificate::{CertUsage, Certificate};
        use os_trustd_domain::x509::{
            DistinguishedName, SubjectAltNames, Validity,
        };
        let (ca, sgn) = real_ca();
        let leaf_signer = EcdsaP256Signer::generate().unwrap();
        let server_leaf = Certificate {
            serial: 43,
            subject: DistinguishedName::common("oya-cloud-iam-pdp"),
            issuer: DistinguishedName::common("oyatie-cell-7-ca"),
            validity: Validity { not_before: 1_000, not_after: 9_000 },
            usage: CertUsage::ServerAuth,
            sans: SubjectAltNames { uris: vec![URI.to_string()], ..Default::default() },
            public_key_der: leaf_signer.public_key_spki_der(),
            signature: vec![0x01],
        };
        let der_bytes =
            der::encode_leaf_der(&server_leaf, &leaf_signer, ca.certificate(), &sgn).unwrap();
        let ca_spki = ca.certificate().public_key_der.clone();
        assert_eq!(
            verify_leaf_der(&der_bytes, &[ca_spki], 2_500).unwrap_err(),
            LeafVerifyError::MissingClientAuthEku
        );
    }

    #[test]
    fn extract_single_uri_san_reads_identity_without_verifying() {
        // The cell-derivation read path: extract the URI SAN from a leaf without a
        // chain/validity check (the PDP reading its own server leaf).
        let (mut ca, sgn) = real_ca();
        let (leaf, _ca_spki) = issue(&mut ca, &sgn, URI);
        assert_eq!(extract_single_uri_san(&leaf).unwrap(), URI);
        assert!(matches!(
            extract_single_uri_san(b"not-a-der").unwrap_err(),
            LeafVerifyError::Undecodable(_)
        ));
    }

    #[test]
    fn node_leaf_without_uri_san_is_no_uri() {
        // A node cert (DNS SAN only, no URI) from the TRUSTED CA still has no
        // SPIFFE identity.
        let (mut ca, sgn) = real_ca();
        use os_trustd_domain::certificate::CertUsage;
        let wl = EcdsaP256Signer::generate().unwrap();
        let wl_key = KeyPair::new(wl.private_key_der(), wl.public_key_spki_der());
        let csr = CertificateSigningRequest::for_node("node-1", &wl_key, CertUsage::ClientAuth, 3_600)
            .with_dns("node-1.cluster.local");
        let leaf = ca.sign_csr(&csr, 2_000).unwrap();
        let der_bytes = der::encode_leaf_der(&leaf, &wl, ca.certificate(), &sgn).unwrap();
        let ca_spki = ca.certificate().public_key_der.clone();
        assert_eq!(
            verify_leaf_der(&der_bytes, &[ca_spki], 2_500).unwrap_err(),
            LeafVerifyError::NoUriSan
        );
    }
}
