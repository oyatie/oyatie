//! The cluster certificate authority and certificate-signing-request flow.
//!
//! `trustd` holds the cluster CA private key and signs CSRs presented by nodes
//! (after they pass join-token auth). This module models:
//!
//! * [`CertificateSigningRequest`] — a parsed CSR with the subject, requested
//!   SANs/usage, and the requester's public key.
//! * [`IssuancePolicy`] — the rules trustd applies before signing (TTL caps,
//!   whether non-CA leaves may request CA usage, required CN, etc.).
//! * [`CertificateAuthority`] — the self-signed CA plus a monotonic serial
//!   counter and a [`SigningBackend`].

use crate::certificate::{CertUsage, Certificate, IssuedIdentity};
use crate::error::{Result, TrustError};
use crate::signer::SigningBackend;
use crate::x509::{DistinguishedName, KeyPair, PEMEncoded, PEMLabel, SubjectAltNames, Validity};

/// A certificate signing request submitted by a node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertificateSigningRequest {
    /// Requested subject DN (CN = node name, O = requested roles).
    pub subject: DistinguishedName,
    /// Requested usage. Talos rejects leaf requests for CA usage.
    pub usage: CertUsage,
    /// Requested subject alternative names.
    pub sans: SubjectAltNames,
    /// The requester's public key DER (the CA never sees the private key).
    pub public_key_der: Vec<u8>,
    /// Requested TTL in seconds.
    pub ttl_secs: u64,
}

impl CertificateSigningRequest {
    /// Build a CSR for a node identity from its key pair.
    pub fn for_node(
        name: impl Into<String>,
        keypair: &KeyPair,
        usage: CertUsage,
        ttl_secs: u64,
    ) -> Self {
        CertificateSigningRequest {
            subject: DistinguishedName::common(name),
            usage,
            sans: SubjectAltNames::default(),
            public_key_der: keypair.public_der().to_vec(),
            ttl_secs,
        }
    }

    /// Add a requested role (encoded as an `os:<role>` organizational unit).
    pub fn requesting_role(mut self, ou: impl Into<String>) -> Self {
        self.subject.organizations.push(ou.into());
        self
    }

    /// Add a DNS SAN.
    pub fn with_dns(mut self, name: impl Into<String>) -> Self {
        self.sans.dns_names.push(name.into());
        self
    }

    /// PEM envelope for the CSR (stand-in body; see [`Certificate::tbs_bytes`]).
    pub fn to_pem(&self) -> PEMEncoded {
        let mut body = Vec::new();
        body.extend_from_slice(self.subject.to_rfc().as_bytes());
        body.push(self.usage as u8);
        body.extend_from_slice(&self.public_key_der);
        PEMEncoded::new(PEMLabel::CertificateRequest, body)
    }
}

/// Policy applied by trustd before signing a CSR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IssuancePolicy {
    /// Maximum TTL (seconds) the CA will grant a leaf certificate.
    pub max_ttl_secs: u64,
    /// Whether leaf requests may ask for CA (signing) usage. Talos: never.
    pub allow_ca_requests: bool,
    /// Whether an empty CN is allowed (never, for node identities).
    pub require_common_name: bool,
    /// Maximum number of SANs (DNS + IP) the CA will embed; guards against an
    /// abusive CSR requesting an unbounded SAN list.
    pub max_sans: usize,
    /// Whether the CA validates SAN syntax (rejecting malformed DNS/IP).
    pub validate_sans: bool,
}

impl Default for IssuancePolicy {
    fn default() -> Self {
        // Talos default node cert TTL is short (~24h); cap requests accordingly.
        IssuancePolicy {
            max_ttl_secs: 24 * 60 * 60,
            allow_ca_requests: false,
            require_common_name: true,
            max_sans: 16,
            validate_sans: true,
        }
    }
}

impl IssuancePolicy {
    /// Validate a CSR against this policy, returning the *effective* TTL the CA
    /// will grant (clamped to `max_ttl_secs`).
    pub fn approve(&self, csr: &CertificateSigningRequest) -> Result<u64> {
        if self.require_common_name && csr.subject.common_name.trim().is_empty() {
            return Err(TrustError::csr_rejected("CSR has empty common name"));
        }
        if csr.public_key_der.is_empty() {
            return Err(TrustError::csr_rejected("CSR has no public key"));
        }
        if csr.usage.can_sign() && !self.allow_ca_requests {
            return Err(TrustError::csr_rejected(
                "leaf may not request certificate-authority usage",
            ));
        }
        if csr.ttl_secs == 0 {
            return Err(TrustError::csr_rejected("CSR requested zero TTL"));
        }
        let san_count = csr.sans.dns_names.len() + csr.sans.ip_addresses.len();
        if san_count > self.max_sans {
            return Err(TrustError::csr_rejected(format!(
                "CSR requests {san_count} SANs, exceeding the limit of {}",
                self.max_sans
            )));
        }
        if self.validate_sans {
            csr.sans
                .validate()
                .map_err(|e| TrustError::csr_rejected(e.to_string()))?;
        }
        Ok(csr.ttl_secs.min(self.max_ttl_secs))
    }
}

/// The cluster certificate authority held by trustd.
pub struct CertificateAuthority<S: SigningBackend> {
    cert: Certificate,
    keypair: KeyPair,
    signer: S,
    policy: IssuancePolicy,
    next_serial: u64,
}

impl<S: SigningBackend> CertificateAuthority<S> {
    /// Bootstrap a self-signed CA valid from `now` for `ttl_secs`.
    pub fn bootstrap(
        name: impl Into<String>,
        keypair: KeyPair,
        signer: S,
        now: u64,
        ttl_secs: u64,
    ) -> Result<Self> {
        let subject = DistinguishedName::common(name);
        let validity = Validity::from_duration(now, ttl_secs)?;
        let mut cert = Certificate {
            serial: 1,
            subject: subject.clone(),
            issuer: subject,
            validity,
            usage: CertUsage::CertificateAuthority,
            sans: SubjectAltNames::default(),
            public_key_der: keypair.public_der().to_vec(),
            signature: Vec::new(),
        };
        cert.signature = signer.sign(&cert.tbs_bytes());
        cert.validate()?;
        Ok(CertificateAuthority {
            cert,
            keypair,
            signer,
            policy: IssuancePolicy::default(),
            next_serial: 2,
        })
    }

    /// Rehydrate a CA from already-parsed durable state.
    pub(crate) fn from_persisted_parts(
        cert: Certificate,
        keypair: KeyPair,
        signer: S,
        policy: IssuancePolicy,
        next_serial: u64,
    ) -> Result<Self> {
        if !cert.is_ca() {
            return Err(TrustError::invalid("persisted trust anchor is not a CA"));
        }
        cert.validate()?;
        if !keypair.matches_public(&cert.public_key_der) {
            return Err(TrustError::verification_failed(
                "persisted CA private key does not match CA certificate public key",
            ));
        }
        if !signer.verify(&cert.tbs_bytes(), &cert.signature) {
            return Err(TrustError::verification_failed(
                "persisted CA certificate was not signed by the restored signer",
            ));
        }
        if next_serial <= cert.serial {
            return Err(TrustError::invalid(
                "persisted CA serial counter is not ahead of the CA certificate serial",
            ));
        }
        Ok(CertificateAuthority {
            cert,
            keypair,
            signer,
            policy,
            next_serial,
        })
    }

    /// Override the issuance policy.
    pub fn with_policy(mut self, policy: IssuancePolicy) -> Self {
        self.policy = policy;
        self
    }

    /// The CA's own certificate.
    pub fn certificate(&self) -> &Certificate {
        &self.cert
    }

    /// PEM of the CA certificate, for chain distribution.
    pub fn ca_pem(&self) -> PEMEncoded {
        self.cert.to_pem()
    }

    /// The serial that will be assigned to the next issued certificate.
    pub fn peek_serial(&self) -> u64 {
        self.next_serial
    }

    /// Sign a CSR into a leaf [`Certificate`], applying policy and assigning a
    /// fresh serial. `now` is the issuance time.
    pub fn sign_csr(&mut self, csr: &CertificateSigningRequest, now: u64) -> Result<Certificate> {
        if self.cert.validity.is_expired(now) {
            return Err(TrustError::expired("issuing CA certificate has expired"));
        }
        let ttl = self.policy.approve(csr)?;
        let validity = Validity::from_duration(now, ttl)?;
        let serial = self.next_serial;
        let mut leaf = Certificate {
            serial,
            subject: csr.subject.clone(),
            issuer: self.cert.subject.clone(),
            validity,
            usage: csr.usage,
            sans: csr.sans.clone(),
            public_key_der: csr.public_key_der.clone(),
            signature: Vec::new(),
        };
        leaf.signature = self.signer.sign(&leaf.tbs_bytes());
        leaf.validate()?;
        self.next_serial = self
            .next_serial
            .checked_add(1)
            .ok_or_else(|| TrustError::Other("serial counter overflow".into()))?;
        Ok(leaf)
    }

    /// Sign a CSR and bundle the result with the requester's key PEM and the CA
    /// chain, matching what trustd returns over its gRPC API.
    pub fn issue_identity(
        &mut self,
        csr: &CertificateSigningRequest,
        requester_key: &KeyPair,
        now: u64,
    ) -> Result<IssuedIdentity> {
        if !requester_key.matches_public(&csr.public_key_der) {
            return Err(TrustError::csr_rejected(
                "CSR public key does not match requester key pair",
            ));
        }
        let cert = self.sign_csr(csr, now)?;
        Ok(IssuedIdentity {
            cert_pem: cert.to_pem(),
            key_pem: requester_key.private_pem(),
            ca_pem: self.ca_pem(),
            certificate: cert,
        })
    }

    /// Verify that `cert` was signed by this CA and is valid at `now`. Used by
    /// peers to validate a presented certificate against the trusted root.
    pub fn verify(&self, cert: &Certificate, now: u64) -> Result<()> {
        cert.validate()?;
        if cert.issuer != self.cert.subject {
            return Err(TrustError::verification_failed("issuer is not this CA"));
        }
        if !cert.is_valid_at(now) {
            return Err(TrustError::expired(
                "certificate is not valid at the given time",
            ));
        }
        if !self.signer.verify(&cert.tbs_bytes(), &cert.signature) {
            return Err(TrustError::verification_failed(
                "signature does not match CA key",
            ));
        }
        Ok(())
    }

    /// The CA's signing key pair (held only by trustd).
    pub fn keypair(&self) -> &KeyPair {
        &self.keypair
    }

    /// The signing backend, exposed only to sealed durable persistence code.
    pub(crate) fn signing_backend(&self) -> &S {
        &self.signer
    }

    /// The currently-configured issuance policy.
    pub fn policy(&self) -> &IssuancePolicy {
        &self.policy
    }

    /// Whether this CA's own certificate is still valid at `now`.
    pub fn is_valid_at(&self, now: u64) -> bool {
        self.cert.is_valid_at(now)
    }

    /// Build (but do not sign) a CSR that re-keys this CA's identity, used when
    /// starting a CA rotation: a fresh key pair under the same (or a new) name.
    pub fn rotation_request(
        name: impl Into<String>,
        new_keypair: &KeyPair,
        ttl_secs: u64,
    ) -> CertificateSigningRequest {
        CertificateSigningRequest {
            subject: DistinguishedName::common(name),
            usage: CertUsage::CertificateAuthority,
            sans: SubjectAltNames::default(),
            public_key_der: new_keypair.public_der().to_vec(),
            ttl_secs,
        }
    }

    /// Rotate the CA: mint a brand-new self-signed CA generation with a fresh key
    /// pair and signer, starting its serial counter at 1. The old CA value is
    /// consumed; callers that must keep trusting the old generation should add it
    /// to a [`crate::bundle::TrustBundle`] before rotating. Returns the new CA.
    pub fn rotate(
        name: impl Into<String>,
        new_keypair: KeyPair,
        new_signer: S,
        now: u64,
        ttl_secs: u64,
    ) -> Result<Self> {
        Self::bootstrap(name, new_keypair, new_signer, now, ttl_secs)
    }

    /// Re-issue (renew) the CA's *own* certificate under the same key pair,
    /// extending its validity window without changing the public key — the
    /// in-place renewal Talos performs as a CA approaches expiry. Keeps the
    /// serial counter untouched so issued leaves remain monotonic.
    pub fn renew_self(&mut self, now: u64, ttl_secs: u64) -> Result<()> {
        let validity = Validity::from_duration(now, ttl_secs)?;
        let mut cert = Certificate {
            serial: self.cert.serial,
            subject: self.cert.subject.clone(),
            issuer: self.cert.subject.clone(),
            validity,
            usage: CertUsage::CertificateAuthority,
            sans: SubjectAltNames::default(),
            public_key_der: self.keypair.public_der().to_vec(),
            signature: Vec::new(),
        };
        cert.signature = self.signer.sign(&cert.tbs_bytes());
        cert.validate()?;
        self.cert = cert;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::StaticKeySealer;
    use crate::signer::InMemorySigner;
    use std::fs;
    use std::path::PathBuf;

    fn ca() -> CertificateAuthority<InMemorySigner> {
        CertificateAuthority::bootstrap(
            "talos-ca",
            KeyPair::from_seed(b"ca-seed"),
            InMemorySigner::from_seed("ca-seed"),
            1000,
            1_000_000,
        )
        .unwrap()
    }

    fn temp_ca_state_path(test_name: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "oya-trustd-{test_name}-{}-{nonce}.state",
            std::process::id()
        ))
    }

    #[test]
    fn bootstrap_produces_self_signed_ca() {
        let ca = ca();
        assert!(ca.certificate().is_ca());
        assert_eq!(ca.certificate().issuer, ca.certificate().subject);
        assert!(ca.verify(ca.certificate(), 2000).is_ok());
    }

    #[test]
    fn signs_leaf_and_verifies() {
        let mut ca = ca();
        let node_key = KeyPair::from_seed(b"node-1");
        let csr =
            CertificateSigningRequest::for_node("node-1", &node_key, CertUsage::ClientAuth, 3600)
                .requesting_role("os:reader");
        let id = ca.issue_identity(&csr, &node_key, 2000).unwrap();
        assert_eq!(id.name(), "node-1");
        assert!(id.certificate.roles().can_read());
        assert!(ca.verify(&id.certificate, 2500).is_ok());
    }

    #[test]
    fn serials_are_monotonic() {
        let mut ca = ca();
        let node_key = KeyPair::from_seed(b"node-1");
        let csr = CertificateSigningRequest::for_node("n", &node_key, CertUsage::ClientAuth, 3600);
        let a = ca.sign_csr(&csr, 2000).unwrap();
        let b = ca.sign_csr(&csr, 2000).unwrap();
        assert_eq!(a.serial + 1, b.serial);
    }

    #[test]
    fn policy_rejects_ca_request_and_caps_ttl() {
        let mut ca = ca();
        let node_key = KeyPair::from_seed(b"node-1");
        let ca_req = CertificateSigningRequest::for_node(
            "evil",
            &node_key,
            CertUsage::CertificateAuthority,
            3600,
        );
        assert_eq!(
            ca.sign_csr(&ca_req, 2000).unwrap_err().kind(),
            "csr_rejected"
        );

        // request more than the 24h cap; effective TTL is clamped.
        let long = CertificateSigningRequest::for_node(
            "node",
            &node_key,
            CertUsage::ClientAuth,
            10 * 24 * 3600,
        );
        let cert = ca.sign_csr(&long, 2000).unwrap();
        assert_eq!(
            cert.validity.not_after - cert.validity.not_before,
            24 * 3600
        );
    }

    #[test]
    fn mismatched_key_rejected() {
        let mut ca = ca();
        let node_key = KeyPair::from_seed(b"node-1");
        let other_key = KeyPair::from_seed(b"other");
        let csr =
            CertificateSigningRequest::for_node("node-1", &node_key, CertUsage::ClientAuth, 3600);
        assert_eq!(
            ca.issue_identity(&csr, &other_key, 2000)
                .unwrap_err()
                .kind(),
            "csr_rejected"
        );
    }

    #[test]
    fn foreign_cert_fails_verification() {
        let ca = ca();
        let mut rogue = CertificateAuthority::bootstrap(
            "rogue-ca",
            KeyPair::from_seed(b"rogue"),
            InMemorySigner::from_seed("rogue"),
            1000,
            1_000_000,
        )
        .unwrap();
        let node_key = KeyPair::from_seed(b"node-1");
        let csr = CertificateSigningRequest::for_node("n", &node_key, CertUsage::ClientAuth, 3600);
        let rogue_cert = rogue.sign_csr(&csr, 2000).unwrap();
        assert!(ca.verify(&rogue_cert, 2500).is_err());
    }

    #[test]
    fn policy_rejects_too_many_sans() {
        let mut ca = ca().with_policy(IssuancePolicy {
            max_sans: 2,
            ..IssuancePolicy::default()
        });
        let key = KeyPair::from_seed(b"node");
        let mut csr =
            CertificateSigningRequest::for_node("node", &key, CertUsage::ClientAuth, 3600);
        csr.sans.dns_names = vec![
            "a.example.com".into(),
            "b.example.com".into(),
            "c.example.com".into(),
        ];
        assert_eq!(ca.sign_csr(&csr, 2000).unwrap_err().kind(), "csr_rejected");
    }

    #[test]
    fn policy_rejects_malformed_san() {
        let mut ca = ca();
        let key = KeyPair::from_seed(b"node");
        let mut csr =
            CertificateSigningRequest::for_node("node", &key, CertUsage::ClientAuth, 3600);
        csr.sans.dns_names = vec!["_invalid_".into()];
        assert_eq!(ca.sign_csr(&csr, 2000).unwrap_err().kind(), "csr_rejected");
    }

    #[test]
    fn renew_self_extends_validity_keeps_key() {
        let mut ca = ca();
        let old_key = ca.keypair().public_der().to_vec();
        let old_after = ca.certificate().validity.not_after;
        ca.renew_self(500_000, 2_000_000).unwrap();
        assert_eq!(ca.keypair().public_der(), old_key.as_slice());
        assert!(ca.certificate().validity.not_after > old_after);
        // a leaf signed after renewal still verifies under the same CA
        let key = KeyPair::from_seed(b"node");
        let csr = CertificateSigningRequest::for_node("node", &key, CertUsage::ClientAuth, 3600);
        let leaf = ca.sign_csr(&csr, 500_001).unwrap();
        assert!(ca.verify(&leaf, 500_500).is_ok());
    }

    #[test]
    fn rotate_starts_fresh_generation() {
        let mut old = ca();
        let key = KeyPair::from_seed(b"node");
        let csr = CertificateSigningRequest::for_node("node", &key, CertUsage::ClientAuth, 3600);
        let old_leaf = old.sign_csr(&csr, 2000).unwrap();

        let new = CertificateAuthority::rotate(
            "talos-ca",
            KeyPair::from_seed(b"new-ca-seed"),
            InMemorySigner::from_seed("new-ca-seed"),
            3000,
            1_000_000,
        )
        .unwrap();
        // old leaf does not verify under the new generation
        assert!(new.verify(&old_leaf, 3500).is_err());
        assert!(new.is_valid_at(3500));
    }

    #[test]
    fn rotation_request_carries_new_key() {
        let new_key = KeyPair::from_seed(b"new-ca-key");
        let req = CertificateAuthority::<InMemorySigner>::rotation_request(
            "talos-ca", &new_key, 1_000_000,
        );
        assert_eq!(req.public_key_der, new_key.public_der());
        assert!(req.usage.can_sign());
    }

    #[test]
    fn sealed_state_round_trips_without_plaintext_root_key() {
        let path = temp_ca_state_path("roundtrip");
        let sealer = StaticKeySealer::new(b"unit-test-kms-root".to_vec()).unwrap();
        let mut ca = ca();
        let node_key = KeyPair::from_seed(b"node-persisted");
        let csr = CertificateSigningRequest::for_node(
            "node-persisted",
            &node_key,
            CertUsage::ClientAuth,
            3600,
        );
        let before_restart = ca.sign_csr(&csr, 2000).unwrap();
        let next_serial = ca.peek_serial();

        ca.save_sealed_state(&path, &sealer).unwrap();
        let persisted = fs::read_to_string(&path).unwrap();
        assert!(persisted.contains("sealed.keypair_private_der="));
        assert!(persisted.contains("sealed.signer_private_key="));
        assert!(!persisted.contains("ca-seed"));
        assert!(!persisted.contains(&crate::x509::hex_encode(ca.keypair().private_der())));

        let mut restored =
            CertificateAuthority::<InMemorySigner>::load_sealed_state(&path, &sealer).unwrap();
        assert_eq!(restored.peek_serial(), next_serial);
        assert!(restored.verify(&before_restart, 2500).is_ok());
        let after_restart = restored.sign_csr(&csr, 2500).unwrap();
        assert_eq!(after_restart.serial, next_serial);
        assert!(restored.verify(&after_restart, 2600).is_ok());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn sealed_state_rejects_wrong_sealer_and_tampering() {
        let path = temp_ca_state_path("tamper");
        let sealer = StaticKeySealer::new(b"unit-test-kms-root".to_vec()).unwrap();
        ca().save_sealed_state(&path, &sealer).unwrap();

        let wrong = StaticKeySealer::new(b"wrong-kms-root".to_vec()).unwrap();
        assert_eq!(
            CertificateAuthority::<InMemorySigner>::load_sealed_state(&path, &wrong)
                .err()
                .unwrap()
                .kind(),
            "verification_failed"
        );

        let tampered = fs::read_to_string(&path).unwrap().replacen(
            "sealed.signer_private_key=",
            "sealed.signer_private_key=00",
            1,
        );
        fs::write(&path, tampered).unwrap();
        assert_eq!(
            CertificateAuthority::<InMemorySigner>::load_sealed_state(&path, &sealer)
                .err()
                .unwrap()
                .kind(),
            "verification_failed"
        );

        let _ = fs::remove_file(path);
    }
}
