//! The etcd secrets controller.
//!
//! Mirrors `internal/app/machined/pkg/controllers/secrets/etcd.go`. From the
//! etcd root CA it derives the per-node etcd certificates:
//!
//! * the **peer** certificate (server+client auth, SANs = node hostname + all
//!   node addresses + loopback) used for etcd-to-etcd traffic;
//! * the **server** certificate (server auth) used for client->etcd traffic;
//! * the **client** (apiserver etcd client) certificate used by the API server
//!   to talk to etcd.
//!
//! etcd certs are per-node, so the controller is seeded with the node's name
//! and addresses, and re-issues when those change.

use crate::bundle::{CaKind, CertUsage, Certificate, KeyPair, SecretsBundle, Subject};
use crate::certsans::{CertSans, San};
use crate::rotation::{CertState, RenewalPolicy, evaluate};
use os_kernel::NodeAddress;
use os_kernel::error::{Error, Result};
use std::collections::BTreeMap;

/// Default TTL for etcd leaf certificates (1 year).
pub const ETCD_CERT_TTL_SECS: u64 = 365 * 24 * 60 * 60;

/// The etcd certificates this controller owns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EtcdCert {
    /// etcd peer cert (server + client auth), carries node SANs.
    Peer,
    /// etcd server cert (server auth), carries node SANs.
    Server,
    /// apiserver -> etcd client cert (client auth), no SANs.
    ApiServerClient,
}

impl EtcdCert {
    /// All managed etcd certs.
    pub fn all() -> [EtcdCert; 3] {
        [EtcdCert::Peer, EtcdCert::Server, EtcdCert::ApiServerClient]
    }

    /// The certificate usage.
    pub fn usage(self) -> CertUsage {
        match self {
            EtcdCert::Peer => CertUsage::ServerAndClientAuth,
            EtcdCert::Server => CertUsage::ServerAuth,
            EtcdCert::ApiServerClient => CertUsage::ClientAuth,
        }
    }

    /// Whether this cert carries the node SANs.
    pub fn uses_sans(self) -> bool {
        matches!(self, EtcdCert::Peer | EtcdCert::Server)
    }

    /// The subject common name for a given node.
    pub fn common_name(self, node_name: &str) -> String {
        match self {
            EtcdCert::Peer | EtcdCert::Server => node_name.to_string(),
            EtcdCert::ApiServerClient => "kube-apiserver-etcd-client".to_string(),
        }
    }

    /// Deterministic model key pair for this node-scoped etcd certificate.
    ///
    /// The secrets projection and issuance path both use this helper so
    /// rendered keys match issued certs.
    pub fn keypair(self, node_name: &str) -> KeyPair {
        KeyPair::from_seed(&format!("etcd-leaf:{node_name}:{self:?}"))
    }
}

/// The etcd secrets controller for a single node.
#[derive(Debug, Clone)]
pub struct EtcdController {
    node_name: String,
    policy: RenewalPolicy,
    ttl_secs: u64,
    sans: CertSans,
    certs: BTreeMap<EtcdCert, Certificate>,
    fingerprints: BTreeMap<EtcdCert, u64>,
}

impl EtcdController {
    /// Construct for a node, given its name and its addresses. Loopback is
    /// always added so local etcd clients can connect over `127.0.0.1`.
    pub fn new(node_name: impl Into<String>, addresses: &[NodeAddress]) -> Result<Self> {
        let node_name = node_name.into();
        if node_name.trim().is_empty() {
            return Err(Error::invalid("etcd controller needs a node name"));
        }
        let mut sans = CertSans::new();
        sans.append_dns(&node_name)?;
        sans.append_ip(NodeAddress::parse("127.0.0.1")?);
        for a in addresses {
            sans.append_ip(*a);
        }
        Ok(EtcdController {
            node_name,
            policy: RenewalPolicy::default(),
            ttl_secs: ETCD_CERT_TTL_SECS,
            sans,
            certs: BTreeMap::new(),
            fingerprints: BTreeMap::new(),
        })
    }

    /// Override the renewal policy.
    pub fn with_policy(mut self, policy: RenewalPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// The node's SAN set.
    pub fn sans(&self) -> &CertSans {
        &self.sans
    }

    /// The node name this controller is issuing etcd certificates for.
    pub fn node_name(&self) -> &str {
        &self.node_name
    }

    /// Update the node's addresses (e.g. a new IP was assigned). Re-derives the
    /// SAN set, which will mark the peer/server certs stale.
    pub fn set_addresses(&mut self, addresses: &[NodeAddress]) -> Result<()> {
        let mut sans = CertSans::new();
        sans.append_dns(&self.node_name)?;
        sans.append_ip(NodeAddress::parse("127.0.0.1")?);
        for a in addresses {
            sans.append_ip(*a);
        }
        self.sans = sans;
        Ok(())
    }

    fn desired_fingerprint(&self, which: EtcdCert, bundle: &SecretsBundle) -> u64 {
        let ca_id = bundle.ca(CaKind::Etcd).identity_fingerprint();
        let san_fp = if which.uses_sans() {
            self.sans.fingerprint()
        } else {
            0
        };
        san_fp ^ ca_id.wrapping_mul(0x9e3779b97f4a7c15)
    }

    /// Evaluate the state of one managed etcd cert.
    pub fn state(&self, which: EtcdCert, bundle: &SecretsBundle, now: u64) -> CertState {
        let desired = self.desired_fingerprint(which, bundle);
        let current_fp = self.fingerprints.get(&which).copied().unwrap_or(0);
        evaluate(
            self.certs.get(&which),
            desired,
            current_fp,
            &self.policy,
            now,
        )
    }

    fn sans_for(&self, which: EtcdCert) -> Vec<San> {
        if which.uses_sans() {
            self.sans.all()
        } else {
            Vec::new()
        }
    }

    /// Issue (or re-issue) one etcd cert.
    pub fn issue(&mut self, which: EtcdCert, bundle: &mut SecretsBundle, now: u64) -> Result<()> {
        let leaf_key = which.keypair(&self.node_name);
        let subject = Subject::common(which.common_name(&self.node_name));
        let sans = self.sans_for(which);
        let desired = self.desired_fingerprint(which, bundle);
        let ca = bundle.ca_mut(CaKind::Etcd);
        let cert = ca.issue(
            subject,
            leaf_key.public_der().to_vec(),
            which.usage(),
            sans,
            now,
            self.ttl_secs,
        )?;
        self.certs.insert(which, cert);
        self.fingerprints.insert(which, desired);
        Ok(())
    }

    /// Reconcile all etcd certs, issuing those that need it.
    pub fn reconcile(&mut self, bundle: &mut SecretsBundle, now: u64) -> Result<Vec<EtcdCert>> {
        let mut issued = Vec::new();
        for which in EtcdCert::all() {
            if self.state(which, bundle, now).should_rotate() {
                self.issue(which, bundle, now)?;
                issued.push(which);
            }
        }
        Ok(issued)
    }

    /// Borrow an issued cert.
    pub fn certificate(&self, which: EtcdCert) -> Option<&Certificate> {
        self.certs.get(&which)
    }

    /// Verify an issued cert against the etcd CA at `now`.
    pub fn verify(&self, which: EtcdCert, bundle: &SecretsBundle, now: u64) -> Result<()> {
        let cert = self
            .certs
            .get(&which)
            .ok_or_else(|| Error::not_found("etcd cert not issued"))?;
        bundle.ca(CaKind::Etcd).verify(cert, now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundle() -> SecretsBundle {
        SecretsBundle::generate("etcd-cluster", 1000).unwrap()
    }

    fn controller() -> EtcdController {
        EtcdController::new("cp-1", &[NodeAddress::parse("10.0.0.5").unwrap()]).unwrap()
    }

    #[test]
    fn empty_node_name_rejected() {
        assert!(EtcdController::new("  ", &[]).is_err());
    }

    #[test]
    fn reconcile_issues_three_certs_with_correct_usage() {
        let mut b = bundle();
        let mut c = controller();
        let issued = c.reconcile(&mut b, 1000).unwrap();
        assert_eq!(issued.len(), 3);
        assert!(c.certificate(EtcdCert::Peer).unwrap().usage.client_auth());
        assert!(c.certificate(EtcdCert::Peer).unwrap().usage.server_auth());
        assert!(c.certificate(EtcdCert::Server).unwrap().usage.server_auth());
        assert!(!c.certificate(EtcdCert::Server).unwrap().usage.client_auth());
        assert!(
            c.certificate(EtcdCert::ApiServerClient)
                .unwrap()
                .usage
                .client_auth()
        );
    }

    #[test]
    fn peer_cert_carries_node_sans_including_loopback() {
        let mut b = bundle();
        let mut c = controller();
        c.reconcile(&mut b, 1000).unwrap();
        let peer = c.certificate(EtcdCert::Peer).unwrap();
        assert!(peer.covers_dns("cp-1"));
        let has_loopback = peer
            .sans
            .iter()
            .any(|s| matches!(s, San::Ip(ip) if ip.to_string() == "127.0.0.1"));
        let has_node_ip = peer
            .sans
            .iter()
            .any(|s| matches!(s, San::Ip(ip) if ip.to_string() == "10.0.0.5"));
        assert!(has_loopback && has_node_ip);
        // The apiserver client cert has no SANs.
        assert!(
            c.certificate(EtcdCert::ApiServerClient)
                .unwrap()
                .sans
                .is_empty()
        );
    }

    #[test]
    fn address_change_makes_san_certs_stale() {
        let mut b = bundle();
        let mut c = controller();
        c.reconcile(&mut b, 1000).unwrap();
        c.set_addresses(&[NodeAddress::parse("10.0.0.9").unwrap()])
            .unwrap();
        assert_eq!(c.state(EtcdCert::Peer, &b, 1000), CertState::Stale);
        assert_eq!(c.state(EtcdCert::Server, &b, 1000), CertState::Stale);
        // Client cert has no SANs, so it stays valid.
        assert_eq!(
            c.state(EtcdCert::ApiServerClient, &b, 1000),
            CertState::Valid
        );
        let issued = c.reconcile(&mut b, 1000).unwrap();
        assert_eq!(issued, vec![EtcdCert::Peer, EtcdCert::Server]);
    }

    #[test]
    fn issued_certs_verify_against_etcd_ca() {
        let mut b = bundle();
        let mut c = controller();
        c.reconcile(&mut b, 1000).unwrap();
        for which in EtcdCert::all() {
            c.verify(which, &b, 2000).unwrap();
        }
        // They do NOT verify against the kubernetes CA.
        let peer = c.certificate(EtcdCert::Peer).unwrap();
        assert!(b.ca(CaKind::Kubernetes).verify(peer, 2000).is_err());
    }
}
