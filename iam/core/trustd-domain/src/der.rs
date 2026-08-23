//! Real ASN.1 X.509 DER issuance (G002 slice-1b-i; ADR-0561 D5 promotion).
//!
//! The trustd domain models a [`Certificate`] as an in-memory value with a
//! custom `tbs_bytes` serialization. This module bridges that shape model to a
//! REAL X.509 DER certificate using `rcgen` on the `aws-lc-rs` backend (ADR-0506,
//! NO ring): it builds a real `TBSCertificate` from the trustd `Certificate`'s
//! load-bearing fields — distinguished name, validity window, key usage, and the
//! Subject Alternative Names INCLUDING the SPIFFE URI SAN as a real
//! `uniformResourceIdentifier` `GeneralName` — and signs it with the issuing CA
//! key. Because rcgen signs the whole `TBSCertificate`, the URI SAN is
//! cryptographically bound into the signature exactly as the shape model bound it
//! into `tbs_bytes`: tampering with the SPIFFE id after issuance breaks the real
//! signature, not a keyed-hash MAC.
//!
//! The DER produced here is what a peer presents and what the
//! `identity-workload-svid-trustd-adapter` parses with `x509-parser`. A real
//! rustls transport handing that adapter the post-handshake leaf DER is the
//! SEPARATE slice-1b-ii (the PDP mTLS wiring), not this module.
//!
//! ADR-0083 Tier-3: production code is panic-free; every fallible step returns a
//! [`TrustError`].

use rcgen::string::Ia5String;
use rcgen::{
    CertificateParams, DistinguishedName as RcgenDn, DnType, ExtendedKeyUsagePurpose, IsCa, Issuer,
    KeyUsagePurpose, SanType, SerialNumber,
};
use time::OffsetDateTime;

use crate::certificate::{CertUsage, Certificate};
use crate::error::{Result, TrustError};
use crate::signer::EcdsaP256Signer;

/// A real ECDSA P-256 workload key pair plus its issued leaf DER. The private key
/// is PKCS#8 DER (PEM-encodable by the caller); the leaf is real X.509 DER.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IssuedDer {
    /// The DER bytes of the issued X.509 leaf certificate (real ASN.1, real sig).
    pub leaf_der: Vec<u8>,
    /// The DER bytes of the issuing CA certificate, for chain building/anchoring.
    pub ca_der: Vec<u8>,
}

/// Build the real X.509 DER for a CA's own self-signed certificate, signed by the
/// CA's [`EcdsaP256Signer`]. The `ca_cert` provides the subject DN + validity; the
/// signer provides the real key. Used to materialise a trust anchor's real DER.
///
/// # Errors
/// [`TrustError`] if any field is unrepresentable (e.g. a non-ASCII SAN) or rcgen
/// fails to self-sign.
pub fn encode_ca_der(ca_cert: &Certificate, ca_signer: &EcdsaP256Signer) -> Result<Vec<u8>> {
    let params = params_from_certificate(ca_cert)?;
    let cert = params
        .self_signed(ca_signer.key_pair())
        .map_err(|e| TrustError::Other(format!("CA self-sign failed: {e}")))?;
    Ok(cert.der().as_ref().to_vec())
}

/// Build the real X.509 DER for a leaf certificate from its trustd [`Certificate`]
/// metadata, signed by the CA. The subject's public key is taken from the
/// subject's [`EcdsaP256Signer`] (the workload holds the matching private key).
///
/// The SPIFFE URI SAN carried in `leaf_cert.sans.uris` becomes a real
/// `uniformResourceIdentifier` `GeneralName`, signed into the certificate — the
/// signature-bound-URI invariant the shape model enforced in `tbs_bytes`.
///
/// # Errors
/// [`TrustError`] on an unrepresentable field (non-ASCII URI/DNS SAN, bad IP) or
/// an rcgen signing failure.
pub fn encode_leaf_der(
    leaf_cert: &Certificate,
    subject_signer: &EcdsaP256Signer,
    ca_cert: &Certificate,
    ca_signer: &EcdsaP256Signer,
) -> Result<Vec<u8>> {
    let leaf_params = params_from_certificate(leaf_cert)?;
    let issuer_params = params_from_certificate(ca_cert)?;
    let issuer = Issuer::new(issuer_params, ca_signer.key_pair());
    let cert = leaf_params
        .signed_by(subject_signer.key_pair(), &issuer)
        .map_err(|e| TrustError::Other(format!("leaf signing failed: {e}")))?;
    Ok(cert.der().as_ref().to_vec())
}

/// Build both the leaf DER and its issuing CA DER in one call (the artifact pair a
/// verifier needs: the leaf to present and the CA to anchor against).
///
/// # Errors
/// [`TrustError`] as for [`encode_leaf_der`] / [`encode_ca_der`].
pub fn issue_der(
    leaf_cert: &Certificate,
    subject_signer: &EcdsaP256Signer,
    ca_cert: &Certificate,
    ca_signer: &EcdsaP256Signer,
) -> Result<IssuedDer> {
    Ok(IssuedDer {
        leaf_der: encode_leaf_der(leaf_cert, subject_signer, ca_cert, ca_signer)?,
        ca_der: encode_ca_der(ca_cert, ca_signer)?,
    })
}

/// Translate a trustd [`Certificate`]'s load-bearing fields into rcgen
/// [`CertificateParams`]. Validity is the unix-seconds window; the DN carries the
/// common name + organizational units; the SANs (DNS/IP/URI) are mapped to their
/// real `GeneralName` slots; the usage drives `is_ca` + extended key usage.
fn params_from_certificate(cert: &Certificate) -> Result<CertificateParams> {
    let mut params = CertificateParams::new(Vec::<String>::new())
        .map_err(|e| TrustError::Other(format!("rcgen params init failed: {e}")))?;

    // Distinguished name: CN + organizations (the RBAC `os:<role>` OUs).
    let mut dn = RcgenDn::new();
    dn.push(DnType::CommonName, cert.subject.common_name.as_str());
    for org in &cert.subject.organizations {
        dn.push(DnType::OrganizationName, org.as_str());
    }
    params.distinguished_name = dn;

    // Validity window (unix seconds -> OffsetDateTime).
    params.not_before = offset_from_unix(cert.validity.not_before)?;
    params.not_after = offset_from_unix(cert.validity.not_after)?;

    // Serial number (monotonic per CA).
    params.serial_number = Some(SerialNumber::from(cert.serial));

    // Subject Alternative Names: DNS, IP, and the SPIFFE URI (signature-bound).
    for dns in &cert.sans.dns_names {
        params.subject_alt_names.push(SanType::DnsName(ia5(dns)?));
    }
    for ip in &cert.sans.ip_addresses {
        let addr = ip
            .parse::<std::net::IpAddr>()
            .map_err(|_| TrustError::invalid(format!("invalid IP SAN '{ip}'")))?;
        params.subject_alt_names.push(SanType::IpAddress(addr));
    }
    for uri in &cert.sans.uris {
        params.subject_alt_names.push(SanType::URI(ia5(uri)?));
    }

    // Usage: CA (basicConstraints cA:TRUE) vs leaf (cA:FALSE) + the matching
    // extended key usage. A workload SVID is a ClientAuth leaf; the shape model's
    // IssuancePolicy already guarantees a workload leaf is never CA-capable.
    match cert.usage {
        CertUsage::CertificateAuthority => {
            params.is_ca = IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
            params.key_usages = vec![
                KeyUsagePurpose::KeyCertSign,
                KeyUsagePurpose::CrlSign,
                KeyUsagePurpose::DigitalSignature,
            ];
        }
        CertUsage::ServerAuth => {
            params.is_ca = IsCa::NoCa;
            params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
            params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
        }
        CertUsage::ClientAuth => {
            params.is_ca = IsCa::NoCa;
            params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
            params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ClientAuth];
        }
    }

    Ok(params)
}

/// Convert an `Ia5String` from a `&str`, erroring (never panicking) on a
/// non-IA5/ASCII character (a SPIFFE id is ASCII by construction).
fn ia5(s: &str) -> Result<Ia5String> {
    Ia5String::try_from(s.to_string())
        .map_err(|_| TrustError::invalid(format!("value '{s}' is not a valid IA5 string")))
}

/// Convert unix seconds to `time::OffsetDateTime`, erroring on an out-of-range
/// value rather than panicking.
fn offset_from_unix(secs: u64) -> Result<OffsetDateTime> {
    let s = i64::try_from(secs)
        .map_err(|_| TrustError::invalid(format!("timestamp {secs} out of range")))?;
    OffsetDateTime::from_unix_timestamp(s)
        .map_err(|_| TrustError::invalid(format!("timestamp {secs} out of range")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ca::{CertificateAuthority, CertificateSigningRequest};
    use crate::x509::KeyPair;
    use x509_parser::prelude::FromDer;

    const SPIFFE_URI: &str = "spiffe://oyatie.cell-7/tenant/ten_acme/secrets-sync";

    /// Bootstrap a CA whose `SigningBackend` is a REAL ECDSA signer, plus a
    /// matching trustd `Certificate` whose `public_key_der` is the CA's real SPKI.
    fn real_ca() -> (CertificateAuthority<EcdsaP256Signer>, EcdsaP256Signer) {
        let signer = EcdsaP256Signer::generate().unwrap();
        let ca_keypair = KeyPair::new(signer.private_key_der(), signer.public_key_spki_der());
        let ca = CertificateAuthority::bootstrap(
            "oyatie-cell-7-ca",
            ca_keypair,
            signer.clone(),
            1_000,
            10_000_000,
        )
        .unwrap();
        (ca, signer)
    }

    #[test]
    fn leaf_der_parses_and_carries_the_signed_spiffe_uri() {
        let (mut ca, ca_signer) = real_ca();
        let wl_signer = EcdsaP256Signer::generate().unwrap();
        let wl_key = KeyPair::new(wl_signer.private_key_der(), wl_signer.public_key_spki_der());
        let csr =
            CertificateSigningRequest::for_workload("secrets-sync", SPIFFE_URI, &wl_key, 3_600);
        let leaf = ca.sign_csr(&csr, 2_000).unwrap();

        let der = encode_leaf_der(&leaf, &wl_signer, ca.certificate(), &ca_signer).unwrap();

        // It is REAL DER: x509-parser parses it and recovers the URI SAN.
        let (_, parsed) = x509_parser::certificate::X509Certificate::from_der(&der).unwrap();
        let san = parsed.subject_alternative_name().unwrap().unwrap();
        let uris: Vec<String> = san
            .value
            .general_names
            .iter()
            .filter_map(|gn| match gn {
                x509_parser::extensions::GeneralName::URI(u) => Some((*u).to_string()),
                _ => None,
            })
            .collect();
        assert_eq!(uris, vec![SPIFFE_URI.to_string()]);
    }

    #[test]
    fn leaf_signature_verifies_against_real_ca_and_tamper_breaks_it() {
        let (mut ca, ca_signer) = real_ca();
        let wl_signer = EcdsaP256Signer::generate().unwrap();
        let wl_key = KeyPair::new(wl_signer.private_key_der(), wl_signer.public_key_spki_der());
        let csr =
            CertificateSigningRequest::for_workload("secrets-sync", SPIFFE_URI, &wl_key, 3_600);
        let leaf = ca.sign_csr(&csr, 2_000).unwrap();

        let issued = issue_der(&leaf, &wl_signer, ca.certificate(), &ca_signer).unwrap();
        let (_, leaf_parsed) =
            x509_parser::certificate::X509Certificate::from_der(&issued.leaf_der).unwrap();
        let (_, ca_parsed) =
            x509_parser::certificate::X509Certificate::from_der(&issued.ca_der).unwrap();

        // The real leaf signature verifies against the real CA public key.
        assert!(
            leaf_parsed
                .verify_signature(Some(ca_parsed.public_key()))
                .is_ok()
        );

        // Flip a byte in the DER -> real signature no longer verifies (not a MAC).
        let mut tampered = issued.leaf_der.clone();
        let mid = tampered.len() / 2;
        tampered[mid] ^= 0xFF;
        // Either the DER no longer parses, or its signature fails — both are DENY.
        let broke = match x509_parser::certificate::X509Certificate::from_der(&tampered) {
            Err(_) => true,
            Ok((_, t)) => t.verify_signature(Some(ca_parsed.public_key())).is_err(),
        };
        assert!(broke, "tampering a real-DER leaf must break verification");
    }
}
