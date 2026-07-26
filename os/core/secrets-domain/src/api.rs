//! The Talos machine-API (`apid`) certificate controller.
//!
//! Mirrors `internal/app/machined/pkg/controllers/secrets/api.go`. The `apid`
//! service terminates the Talos gRPC API. Its serving certificate is signed by
//! the **OS** root CA and carries the node's SANs (hostname + addresses +
//! `localhost`/loopback). The certificate is short-lived and rotated
//! aggressively, so the renewal policy here is tighter than the Kubernetes one.
//!
//! On workers, `apid` does not hold the OS CA private key; it obtains its serving
//! certificate from `trustd` via a CSR. This controller models the control-plane
//! path (CA available locally); the CSR path is modeled in [`crate::trustd`].

use crate::bundle::{CaKind, CertUsage, Certificate, KeyPair, SecretsBundle, Subject};
use crate::certsans::{CertSans, San};
use crate::rotation::{CertState, RenewalPolicy, evaluate};
use os_kernel::error::{Error, Result};
use os_kernel::{Hostname, NodeAddress};

/// Default TTL for the apid serving certificate: short-lived (~24h) so it is
/// rotated frequently, matching Talos's node-cert lifetime.
pub const API_CERT_TTL_SECS: u64 = 24 * 60 * 60;

/// The `apid` serving-certificate controller for a node.
#[derive(Debug, Clone)]
pub struct ApiCertController {
    hostname: Hostname,
    policy: RenewalPolicy,
    ttl_secs: u64,
    sans: CertSans,
    cert: Option<Certificate>,
    fingerprint: u64,
}

impl ApiCertController {
    /// Construct for a node given its hostname and addresses. `localhost` and
    /// loopback are always present so local clients (talosctl over the local
    /// socket bridge) work.
    pub fn new(hostname: Hostname, addresses: &[NodeAddress]) -> Result<Self> {
        let mut sans = CertSans::new();
        sans.append_dns(hostname.as_str())?;
        sans.append_dns("localhost")?;
        sans.append_ip(NodeAddress::parse("127.0.0.1")?);
        for a in addresses {
            sans.append_ip(*a);
        }
        Ok(ApiCertController {
            hostname,
            // apid certs renew earlier: at 1/2 of a 24h life.
            policy: RenewalPolicy::default(),
            ttl_secs: API_CERT_TTL_SECS,
            sans,
            cert: None,
            fingerprint: 0,
        })
    }

    /// Override the renewal policy.
    pub fn with_policy(mut self, policy: RenewalPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// The SAN set.
    pub fn sans(&self) -> &CertSans {
        &self.sans
    }

    /// Add an extra user-supplied `certSAN`.
    pub fn add_san(&mut self, raw: &str) -> Result<()> {
        self.sans.append(raw)
    }

    fn desired_fingerprint(&self, bundle: &SecretsBundle) -> u64 {
        let ca_id = bundle.ca(CaKind::Os).identity_fingerprint();
        self.sans.fingerprint() ^ ca_id.wrapping_mul(0x9e3779b97f4a7c15)
    }

    /// Evaluate whether the serving cert needs (re)issue.
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

    /// Issue (or re-issue) the serving certificate from the OS CA.
    pub fn issue(&mut self, bundle: &mut SecretsBundle, now: u64) -> Result<()> {
        let leaf_key = KeyPair::from_seed(&format!("apid:{}", self.hostname.as_str()));
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

    /// Reconcile: issue the cert if needed. Returns whether it was (re)issued.
    pub fn reconcile(&mut self, bundle: &mut SecretsBundle, now: u64) -> Result<bool> {
        if self.state(bundle, now).should_rotate() {
            self.issue(bundle, now)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// The current serving certificate.
    pub fn certificate(&self) -> Option<&Certificate> {
        self.cert.as_ref()
    }

    /// Verify the serving cert against the OS CA at `now`.
    pub fn verify(&self, bundle: &SecretsBundle, now: u64) -> Result<()> {
        let cert = self
            .cert
            .as_ref()
            .ok_or_else(|| Error::not_found("apid cert not issued"))?;
        bundle.ca(CaKind::Os).verify(cert, now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundle() -> SecretsBundle {
        SecretsBundle::generate("api-cluster", 1000).unwrap()
    }

    fn controller() -> ApiCertController {
        ApiCertController::new(
            Hostname::new("cp-1").unwrap(),
            &[NodeAddress::parse("10.0.0.5").unwrap()],
        )
        .unwrap()
    }

    #[test]
    fn issues_server_cert_with_localhost_and_node_sans() {
        let mut b = bundle();
        let mut c = controller();
        assert_eq!(c.state(&b, 1000), CertState::Missing);
        assert!(c.reconcile(&mut b, 1000).unwrap());
        let cert = c.certificate().unwrap();
        assert!(cert.usage.server_auth());
        assert!(cert.covers_dns("cp-1"));
        assert!(cert.covers_dns("localhost"));
        c.verify(&b, 2000).unwrap();
    }

    #[test]
    fn second_reconcile_is_noop() {
        let mut b = bundle();
        let mut c = controller();
        assert!(c.reconcile(&mut b, 1000).unwrap());
        assert!(!c.reconcile(&mut b, 1000).unwrap());
        assert_eq!(c.state(&b, 1000), CertState::Valid);
    }

    #[test]
    fn short_lived_cert_expires_and_reissues() {
        let mut b = bundle();
        let mut c = controller();
        c.reconcile(&mut b, 1000).unwrap();
        let after_expiry = 1000 + API_CERT_TTL_SECS + 1;
        assert_eq!(c.state(&b, after_expiry), CertState::Expired);
        assert!(c.reconcile(&mut b, after_expiry).unwrap());
        assert_eq!(c.state(&b, after_expiry), CertState::Valid);
    }

    #[test]
    fn adding_san_marks_stale() {
        let mut b = bundle();
        let mut c = controller();
        c.reconcile(&mut b, 1000).unwrap();
        c.add_san("extra.example").unwrap();
        assert_eq!(c.state(&b, 1000), CertState::Stale);
        c.reconcile(&mut b, 1000).unwrap();
        assert!(c.certificate().unwrap().covers_dns("extra.example"));
    }

    #[test]
    fn signed_by_os_ca_not_kubernetes() {
        let mut b = bundle();
        let mut c = controller();
        c.reconcile(&mut b, 1000).unwrap();
        let cert = c.certificate().unwrap();
        assert!(b.ca(CaKind::Os).verify(cert, 2000).is_ok());
        assert!(b.ca(CaKind::Kubernetes).verify(cert, 2000).is_err());
    }
}
