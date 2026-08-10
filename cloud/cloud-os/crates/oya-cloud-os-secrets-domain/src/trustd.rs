//! The `trustd` certificate path and its serving certificate.
//!
//! Mirrors `internal/app/machined/pkg/controllers/secrets/trustd.go` plus the
//! `trustd` CSR-signing surface. `trustd` runs on control-plane nodes and holds
//! the OS root CA private key. It does two things modeled here:
//!
//! 1. It has its own **serving certificate** (signed by the OS CA), used to
//!    terminate the gRPC endpoint that workers call. Tracked/rotated like the
//!    apid cert.
//! 2. It signs **CSRs** from nodes that present the cluster join token, so a
//!    worker's `apid` can obtain a serving cert without holding the CA key.
//!    This is the [`TrustdSigner`] flow with its [`IssuancePolicy`].

use crate::bundle::{CaKind, CertUsage, Certificate, KeyPair, SecretsBundle, Subject, Token};
use crate::certsans::{CertSans, San};
use crate::rotation::{CertState, RenewalPolicy, evaluate};
use os_kernel::error::{Error, Result};
use os_kernel::{Hostname, NodeAddress};

/// Default TTL for the trustd serving certificate (~24h, short-lived).
pub const TRUSTD_CERT_TTL_SECS: u64 = 24 * 60 * 60;

/// The trustd serving-certificate controller for a control-plane node.
#[derive(Debug, Clone)]
pub struct TrustdCertController {
    hostname: Hostname,
    policy: RenewalPolicy,
    ttl_secs: u64,
    sans: CertSans,
    cert: Option<Certificate>,
    fingerprint: u64,
}

impl TrustdCertController {
    /// Construct for a node from its hostname and addresses.
    pub fn new(hostname: Hostname, addresses: &[NodeAddress]) -> Result<Self> {
        let mut sans = CertSans::new();
        sans.append_dns(hostname.as_str())?;
        sans.append_ip(NodeAddress::parse("127.0.0.1")?);
        for a in addresses {
            sans.append_ip(*a);
        }
        Ok(TrustdCertController {
            hostname,
            policy: RenewalPolicy::default(),
            ttl_secs: TRUSTD_CERT_TTL_SECS,
            sans,
            cert: None,
            fingerprint: 0,
        })
    }

    /// The SAN set.
    pub fn sans(&self) -> &CertSans {
        &self.sans
    }

    fn desired_fingerprint(&self, bundle: &SecretsBundle) -> u64 {
        let ca_id = bundle.ca(CaKind::Os).identity_fingerprint();
        self.sans.fingerprint() ^ ca_id.wrapping_mul(0x9e3779b97f4a7c15)
    }

    /// Evaluate the serving cert state.
    pub fn state(&self, bundle: &SecretsBundle, now: u64) -> CertState {
        let desired = self.desired_fingerprint(bundle);
        evaluate(
            self.cert.as_ref(),
            desired,
            self.fingerprint,
            &self.policy,
            now,
        )
    }

    /// Issue (or re-issue) the trustd serving cert from the OS CA.
    pub fn issue(&mut self, bundle: &mut SecretsBundle, now: u64) -> Result<()> {
        let leaf_key = KeyPair::from_seed(&format!("trustd:{}", self.hostname.as_str()));
        let subject = Subject::common(self.hostname.as_str());
        let sans: Vec<San> = self.sans.all();
        let desired = self.desired_fingerprint(bundle);
        let ca = bundle.ca_mut(CaKind::Os);
        let cert = ca.issue(
            subject,
            leaf_key.public_der().to_vec(),
            CertUsage::ServerAuth,
            sans,
            now,
            self.ttl_secs,
        )?;
        self.cert = Some(cert);
        self.fingerprint = desired;
        Ok(())
    }

    /// Reconcile: issue if needed.
    pub fn reconcile(&mut self, bundle: &mut SecretsBundle, now: u64) -> Result<bool> {
        if self.state(bundle, now).should_rotate() {
            self.issue(bundle, now)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// The current serving cert.
    pub fn certificate(&self) -> Option<&Certificate> {
        self.cert.as_ref()
    }

    /// Verify the serving cert against the OS CA.
    pub fn verify(&self, bundle: &SecretsBundle, now: u64) -> Result<()> {
        let cert = self
            .cert
            .as_ref()
            .ok_or_else(|| Error::not_found("trustd cert not issued"))?;
        bundle.ca(CaKind::Os).verify(cert, now)
    }
}

// ---------------------------------------------------------------------------
// CSR signing surface
// ---------------------------------------------------------------------------

/// A certificate signing request a worker presents to trustd to obtain an
/// `apid` serving certificate without holding the OS CA key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertificateSigningRequest {
    /// Subject CN (the requesting node's hostname).
    pub subject: Subject,
    /// Requested SANs.
    pub sans: Vec<San>,
    /// The requester's public key.
    pub public_key_der: Vec<u8>,
    /// Requested TTL.
    pub ttl_secs: u64,
}

impl CertificateSigningRequest {
    /// Build a CSR for a node's apid serving cert.
    pub fn for_node(hostname: &Hostname, key: &KeyPair, sans: Vec<San>, ttl_secs: u64) -> Self {
        CertificateSigningRequest {
            subject: Subject::common(hostname.as_str()),
            sans,
            public_key_der: key.public_der().to_vec(),
            ttl_secs,
        }
    }
}

/// The policy trustd applies before signing a CSR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IssuancePolicy {
    /// Maximum granted TTL (seconds); requests are clamped to this.
    pub max_ttl_secs: u64,
}

impl Default for IssuancePolicy {
    fn default() -> Self {
        IssuancePolicy {
            max_ttl_secs: TRUSTD_CERT_TTL_SECS,
        }
    }
}

impl IssuancePolicy {
    /// Validate a CSR and return the effective (clamped) TTL.
    pub fn approve(&self, csr: &CertificateSigningRequest) -> Result<u64> {
        if csr.subject.common_name.trim().is_empty() {
            return Err(Error::invalid("CSR has empty common name"));
        }
        if csr.public_key_der.is_empty() {
            return Err(Error::invalid("CSR has no public key"));
        }
        if csr.ttl_secs == 0 {
            return Err(Error::invalid("CSR requested zero TTL"));
        }
        Ok(csr.ttl_secs.min(self.max_ttl_secs))
    }
}

/// The trustd signer: gates CSRs on the cluster join token, then signs them
/// with the OS CA from the bundle.
pub struct TrustdSigner {
    join_token: Token,
    policy: IssuancePolicy,
}

impl TrustdSigner {
    /// Create a signer that accepts the given join token.
    pub fn new(join_token: Token) -> Self {
        TrustdSigner {
            join_token,
            policy: IssuancePolicy::default(),
        }
    }

    /// Override the issuance policy.
    pub fn with_policy(mut self, policy: IssuancePolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Whether a presented token authenticates as the cluster join token.
    pub fn authenticate(&self, presented: &Token) -> bool {
        presented == &self.join_token
    }

    /// Sign a CSR after authenticating the join token. The resulting cert is a
    /// server-auth leaf under the OS CA. `now` is the issuance time.
    pub fn sign(
        &self,
        presented_token: &Token,
        csr: &CertificateSigningRequest,
        bundle: &mut SecretsBundle,
        now: u64,
    ) -> Result<Certificate> {
        if !self.authenticate(presented_token) {
            return Err(Error::permission_denied("invalid join token"));
        }
        let ttl = self.policy.approve(csr)?;
        let ca = bundle.ca_mut(CaKind::Os);
        ca.issue(
            csr.subject.clone(),
            csr.public_key_der.clone(),
            CertUsage::ServerAuth,
            csr.sans.clone(),
            now,
            ttl,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundle() -> SecretsBundle {
        SecretsBundle::generate("trustd-cluster", 1000).unwrap()
    }

    #[test]
    fn serving_cert_issued_and_verifies() {
        let mut b = bundle();
        let mut c = TrustdCertController::new(
            Hostname::new("cp-1").unwrap(),
            &[NodeAddress::parse("10.0.0.5").unwrap()],
        )
        .unwrap();
        assert!(c.reconcile(&mut b, 1000).unwrap());
        assert!(c.certificate().unwrap().usage.server_auth());
        c.verify(&b, 2000).unwrap();
        assert!(!c.reconcile(&mut b, 1000).unwrap());
    }

    #[test]
    fn worker_csr_signed_with_valid_join_token() {
        let mut b = bundle();
        let join = b.join_token().clone();
        let signer = TrustdSigner::new(join.clone());

        let worker_host = Hostname::new("worker-1").unwrap();
        let worker_key = KeyPair::from_seed("worker-1-apid");
        let mut sans = CertSans::new();
        sans.append_dns("worker-1").unwrap();
        let csr = CertificateSigningRequest::for_node(&worker_host, &worker_key, sans.all(), 3600);

        let cert = signer.sign(&join, &csr, &mut b, 1500).unwrap();
        assert!(cert.usage.server_auth());
        assert!(cert.covers_dns("worker-1"));
        // The signed cert verifies against the OS CA.
        b.ca(CaKind::Os).verify(&cert, 2000).unwrap();
    }

    #[test]
    fn csr_rejected_with_wrong_token() {
        let mut b = bundle();
        let join = b.join_token().clone();
        let signer = TrustdSigner::new(join);
        let worker_key = KeyPair::from_seed("w");
        let csr = CertificateSigningRequest::for_node(
            &Hostname::new("w").unwrap(),
            &worker_key,
            vec![],
            3600,
        );
        let wrong = Token::from_seed("attacker");
        let err = signer.sign(&wrong, &csr, &mut b, 1500).unwrap_err();
        assert_eq!(err.kind(), "permission_denied");
    }

    #[test]
    fn issuance_policy_clamps_ttl() {
        let mut b = bundle();
        let join = b.join_token().clone();
        let signer = TrustdSigner::new(join.clone());
        let worker_key = KeyPair::from_seed("w");
        let csr = CertificateSigningRequest::for_node(
            &Hostname::new("w").unwrap(),
            &worker_key,
            vec![],
            10 * TRUSTD_CERT_TTL_SECS,
        );
        let cert = signer.sign(&join, &csr, &mut b, 1500).unwrap();
        assert_eq!(cert.validity.total(), TRUSTD_CERT_TTL_SECS);
    }

    #[test]
    fn empty_cn_csr_rejected() {
        let policy = IssuancePolicy::default();
        let csr = CertificateSigningRequest {
            subject: Subject::common("  "),
            sans: vec![],
            public_key_der: vec![1],
            ttl_secs: 3600,
        };
        assert!(policy.approve(&csr).is_err());
    }

    #[test]
    fn trustd_worker_csr_rejects_bad_source_contracts() {
        let policy = IssuancePolicy::default();
        let good_subject = Subject::common("worker-1");

        let missing_key = CertificateSigningRequest {
            subject: good_subject.clone(),
            sans: vec![],
            public_key_der: Vec::new(),
            ttl_secs: 3600,
        };
        let missing_key_err = policy.approve(&missing_key).unwrap_err();
        assert_eq!(missing_key_err.kind(), "invalid");
        assert!(missing_key_err.to_string().contains("no public key"));

        let zero_ttl = CertificateSigningRequest {
            subject: good_subject,
            sans: vec![],
            public_key_der: vec![1],
            ttl_secs: 0,
        };
        let zero_ttl_err = policy.approve(&zero_ttl).unwrap_err();
        assert_eq!(zero_ttl_err.kind(), "invalid");
        assert!(zero_ttl_err.to_string().contains("zero TTL"));
    }
}
