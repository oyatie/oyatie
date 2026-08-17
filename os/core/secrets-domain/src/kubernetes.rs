//! The Kubernetes secrets controller.
//!
//! Mirrors `internal/app/machined/pkg/controllers/secrets/kubernetes.go`. From
//! the Kubernetes root CA in the [`SecretsBundle`] it derives the control-plane
//! leaf certificates: the API server serving cert (with the cluster `certSANs`
//! and the well-known service DNS names), the API-server-to-kubelet client
//! cert, the controller-manager and scheduler client certs, the front-proxy
//! (aggregator) client cert, and the admin kubeconfig client cert. It also
//! surfaces the service-account signing key.
//!
//! Each leaf is tracked for rotation: the controller re-issues a cert when it
//! is missing, expired, inside the renewal window, or when its input SANs
//! change.

use crate::bundle::{CaKind, CertUsage, Certificate, KeyPair, SecretsBundle, Subject};
use crate::certsans::{CertSans, San};
use crate::rotation::{CertState, RenewalPolicy, evaluate};
use os_kernel::Role;
use os_kernel::error::{Error, Result};
use std::collections::BTreeMap;

/// Default TTL for Kubernetes control-plane leaf certificates (1 year), the
/// value Talos uses for generated control-plane PKI.
pub const K8S_CERT_TTL_SECS: u64 = 365 * 24 * 60 * 60;

/// The well-known Kubernetes control-plane certificates this controller owns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum K8sCert {
    /// `kube-apiserver` serving certificate.
    ApiServer,
    /// `kube-apiserver` -> kubelet client certificate.
    ApiServerKubeletClient,
    /// `kube-controller-manager` client certificate.
    ControllerManager,
    /// `kube-scheduler` client certificate.
    Scheduler,
    /// front-proxy / aggregator client certificate (signed by aggregator CA).
    FrontProxy,
    /// cluster-admin kubeconfig client certificate.
    Admin,
}

impl K8sCert {
    /// All managed Kubernetes certificates.
    pub fn all() -> [K8sCert; 6] {
        [
            K8sCert::ApiServer,
            K8sCert::ApiServerKubeletClient,
            K8sCert::ControllerManager,
            K8sCert::Scheduler,
            K8sCert::FrontProxy,
            K8sCert::Admin,
        ]
    }

    /// The subject common name.
    pub fn common_name(self) -> &'static str {
        match self {
            K8sCert::ApiServer => "kube-apiserver",
            K8sCert::ApiServerKubeletClient => "apiserver-kubelet-client",
            K8sCert::ControllerManager => "system:kube-controller-manager",
            K8sCert::Scheduler => "system:kube-scheduler",
            K8sCert::FrontProxy => "front-proxy-client",
            K8sCert::Admin => "admin",
        }
    }

    /// The issuing CA. Everything is the Kubernetes CA except the front-proxy
    /// client, which is signed by the dedicated aggregator CA.
    pub fn issuing_ca(self) -> CaKind {
        match self {
            K8sCert::FrontProxy => CaKind::Aggregator,
            _ => CaKind::Kubernetes,
        }
    }

    /// The certificate usage.
    pub fn usage(self) -> CertUsage {
        match self {
            K8sCert::ApiServer => CertUsage::ServerAuth,
            _ => CertUsage::ClientAuth,
        }
    }

    /// Whether this cert carries SANs (only the apiserver serving cert does).
    pub fn uses_sans(self) -> bool {
        matches!(self, K8sCert::ApiServer)
    }

    /// The organizations (groups) embedded in the subject.
    pub fn organizations(self) -> Vec<String> {
        match self {
            // cluster-admin: the kubeconfig admin cert is a member of system:masters.
            K8sCert::Admin => vec!["system:masters".to_string()],
            _ => Vec::new(),
        }
    }

    /// Deterministic model key pair for this certificate.
    ///
    /// The Rust migration keeps key generation deterministic and side-effect
    /// free until a real crypto backend is wired in. The secrets projection and
    /// issuance path both use this helper so rendered keys match issued certs.
    pub fn keypair(self) -> KeyPair {
        KeyPair::from_seed(&format!("k8s-leaf:{}", self.common_name()))
    }
}

/// The Kubernetes secrets controller: holds derived leaf certs and rotates them.
#[derive(Debug, Clone)]
pub struct KubernetesController {
    policy: RenewalPolicy,
    ttl_secs: u64,
    certs: BTreeMap<K8sCert, Certificate>,
    /// fingerprint of the inputs each cert was built from (SAN set / CA serial).
    fingerprints: BTreeMap<K8sCert, u64>,
    sans: CertSans,
}

impl KubernetesController {
    /// Construct with the apiserver SAN set (user `certSANs` already folded in)
    /// and the Kubernetes cluster domain for the service DNS names.
    pub fn new(mut sans: CertSans, cluster_domain: &str) -> Result<Self> {
        sans.append_kubernetes_service_sans(cluster_domain)?;
        Ok(KubernetesController {
            policy: RenewalPolicy::default(),
            ttl_secs: K8S_CERT_TTL_SECS,
            certs: BTreeMap::new(),
            fingerprints: BTreeMap::new(),
            sans,
        })
    }

    /// Override the renewal policy.
    pub fn with_policy(mut self, policy: RenewalPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// The apiserver SAN set.
    pub fn sans(&self) -> &CertSans {
        &self.sans
    }

    /// Replace the apiserver SAN set; the apiserver cert will be re-issued on
    /// the next reconcile because its input fingerprint changes.
    pub fn set_sans(&mut self, mut sans: CertSans, cluster_domain: &str) -> Result<()> {
        sans.append_kubernetes_service_sans(cluster_domain)?;
        self.sans = sans;
        Ok(())
    }

    fn desired_fingerprint(&self, which: K8sCert, bundle: &SecretsBundle) -> u64 {
        let ca_id = bundle.ca(which.issuing_ca()).identity_fingerprint();
        let san_fp = if which.uses_sans() {
            self.sans.fingerprint()
        } else {
            0
        };
        // Mix CA identity (so a CA rotation forces leaf re-issue) with SAN fp.
        san_fp ^ ca_id.wrapping_mul(0x9e3779b97f4a7c15)
    }

    /// Evaluate the state of a single managed cert.
    pub fn state(&self, which: K8sCert, bundle: &SecretsBundle, now: u64) -> CertState {
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

    fn sans_for(&self, which: K8sCert) -> Vec<San> {
        if which.uses_sans() {
            self.sans.all()
        } else {
            Vec::new()
        }
    }

    fn subject_for(&self, which: K8sCert) -> Subject {
        let mut subject = Subject::common(which.common_name());
        subject.organizations = which.organizations();
        subject
    }

    /// Issue (or re-issue) a single cert from the bundle, recording its input
    /// fingerprint. The leaf's public key is derived deterministically from the
    /// cert kind so a real keygen could replace it.
    pub fn issue(&mut self, which: K8sCert, bundle: &mut SecretsBundle, now: u64) -> Result<()> {
        let leaf_key = which.keypair();
        let subject = self.subject_for(which);
        let sans = self.sans_for(which);
        let desired = self.desired_fingerprint(which, bundle);
        let ca = bundle.ca_mut(which.issuing_ca());
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

    /// Reconcile every managed cert, issuing any that need it. Returns the set
    /// of certs that were (re)issued, in stable order.
    pub fn reconcile(&mut self, bundle: &mut SecretsBundle, now: u64) -> Result<Vec<K8sCert>> {
        let mut issued = Vec::new();
        for which in K8sCert::all() {
            if self.state(which, bundle, now).should_rotate() {
                self.issue(which, bundle, now)?;
                issued.push(which);
            }
        }
        Ok(issued)
    }

    /// Borrow an issued certificate.
    pub fn certificate(&self, which: K8sCert) -> Option<&Certificate> {
        self.certs.get(&which)
    }

    /// Verify an issued cert chains to the correct CA at `now`.
    pub fn verify(&self, which: K8sCert, bundle: &SecretsBundle, now: u64) -> Result<()> {
        let cert = self
            .certs
            .get(&which)
            .ok_or_else(|| Error::not_found("certificate not issued"))?;
        bundle.ca(which.issuing_ca()).verify(cert, now)
    }
}

/// Build the subject for a kubelet client certificate. Talos issues kubelet
/// certs with CN `system:node:<hostname>` in the `system:nodes` group; this is
/// used when bootstrapping a node's kubelet.
pub fn kubelet_subject(node_name: &str) -> Subject {
    Subject::common(format!("system:node:{node_name}")).with_org("system:nodes")
}

/// Whether a presented client cert subject is the cluster-admin (member of the
/// `system:masters` group). Used to gate privileged kubeconfig operations.
pub fn is_cluster_admin(subject: &Subject) -> bool {
    subject.organizations.iter().any(|o| o == "system:masters")
}

/// The Talos `os:` role a kubeconfig-bearing client is granted. Admin gets
/// [`Role::Admin`]; everyone else is a reader.
pub fn role_for_subject(subject: &Subject) -> Role {
    if is_cluster_admin(subject) {
        Role::Admin
    } else {
        Role::Reader
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundle() -> SecretsBundle {
        SecretsBundle::generate("k8s-cluster", 1000).unwrap()
    }

    fn controller() -> KubernetesController {
        let mut sans = CertSans::new();
        sans.append("10.0.0.1").unwrap();
        sans.append("api.cluster.local").unwrap();
        KubernetesController::new(sans, "cluster.local").unwrap()
    }

    #[test]
    fn reconcile_issues_all_certs_once() {
        let mut b = bundle();
        let mut c = controller();
        let issued = c.reconcile(&mut b, 1000).unwrap();
        assert_eq!(issued.len(), 6);
        // Second reconcile right away issues nothing.
        let again = c.reconcile(&mut b, 1000).unwrap();
        assert!(again.is_empty());
    }

    #[test]
    fn apiserver_cert_carries_service_sans_and_user_sans() {
        let mut b = bundle();
        let mut c = controller();
        c.reconcile(&mut b, 1000).unwrap();
        let cert = c.certificate(K8sCert::ApiServer).unwrap();
        assert!(cert.covers_dns("kubernetes"));
        assert!(cert.covers_dns("kubernetes.default.svc.cluster.local"));
        assert!(cert.covers_dns("api.cluster.local"));
        assert!(cert.usage.server_auth());
    }

    #[test]
    fn front_proxy_signed_by_aggregator_ca() {
        let mut b = bundle();
        let mut c = controller();
        c.reconcile(&mut b, 1000).unwrap();
        // front-proxy verifies against aggregator CA, not kubernetes CA.
        assert!(c.verify(K8sCert::FrontProxy, &b, 2000).is_ok());
        let fp = c.certificate(K8sCert::FrontProxy).unwrap();
        assert!(b.ca(CaKind::Kubernetes).verify(fp, 2000).is_err());
    }

    #[test]
    fn changing_sans_makes_apiserver_stale_only() {
        let mut b = bundle();
        let mut c = controller();
        c.reconcile(&mut b, 1000).unwrap();
        let mut sans = CertSans::new();
        sans.append("new.endpoint").unwrap();
        c.set_sans(sans, "cluster.local").unwrap();
        assert_eq!(c.state(K8sCert::ApiServer, &b, 1000), CertState::Stale);
        // A non-SAN cert remains valid.
        assert_eq!(c.state(K8sCert::Scheduler, &b, 1000), CertState::Valid);
        let issued = c.reconcile(&mut b, 1000).unwrap();
        assert_eq!(issued, vec![K8sCert::ApiServer]);
    }

    #[test]
    fn ca_rotation_forces_all_leaves_reissue() {
        let mut b = bundle();
        let mut c = controller();
        c.reconcile(&mut b, 1000).unwrap();
        b.rotate_ca(CaKind::Kubernetes, KeyPair::from_seed("new-k8s-ca"), 1000)
            .unwrap();
        // Every kubernetes-CA-signed leaf is now stale; front-proxy (aggregator) is not.
        assert_eq!(c.state(K8sCert::Admin, &b, 1000), CertState::Stale);
        assert_eq!(c.state(K8sCert::FrontProxy, &b, 1000), CertState::Valid);
    }

    #[test]
    fn admin_subject_is_cluster_admin() {
        let mut b = bundle();
        let mut c = controller();
        c.reconcile(&mut b, 1000).unwrap();
        let admin = c.certificate(K8sCert::Admin).unwrap();
        assert!(is_cluster_admin(&admin.subject));
        assert_eq!(role_for_subject(&admin.subject), Role::Admin);
        let sched = c.certificate(K8sCert::Scheduler).unwrap();
        assert_eq!(role_for_subject(&sched.subject), Role::Reader);
    }

    #[test]
    fn kubelet_subject_format() {
        let s = kubelet_subject("worker-1");
        assert_eq!(s.common_name, "system:node:worker-1");
        assert!(s.organizations.contains(&"system:nodes".to_string()));
    }

    #[test]
    fn renewal_triggers_rotation() {
        let mut b = bundle();
        let mut c = controller().with_policy(RenewalPolicy::new(1, 2).unwrap());
        c.reconcile(&mut b, 1000).unwrap();
        // Past half of the 1-year TTL -> renewing -> reissue.
        let later = 1000 + K8S_CERT_TTL_SECS / 2 + 1;
        assert_eq!(c.state(K8sCert::Scheduler, &b, later), CertState::Renewing);
        let issued = c.reconcile(&mut b, later).unwrap();
        assert_eq!(issued.len(), 6);
    }
}
