//! Cryptographic material the control plane depends on.
//!
//! Mirrors the Talos `k8s.Secrets`/`secrets.Kubernetes` resource: the bundle of
//! CA certificates, service-account keys, and tokens that the apiserver,
//! controller-manager, and kubelet reference. We model the *presence* and
//! identity of each secret (not real crypto), which is what the controllers
//! need to gate static-pod rendering on.

use crate::error::{K8sError, Result};
use std::collections::BTreeMap;

/// A single named secret (PEM blob, token, or key), stored opaquely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Secret {
    /// The secret's logical name, e.g. `"ca.crt"`.
    pub name: String,
    /// The opaque bytes. We don't interpret them.
    pub data: Vec<u8>,
}

/// The bundle of Kubernetes secrets needed to render the control plane.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct K8sSecrets {
    entries: BTreeMap<String, Vec<u8>>,
}

/// The secrets the control plane cannot start without.
pub const REQUIRED_SECRETS: &[&str] = &[
    "ca.crt",
    "ca.key",
    "sa.key",
    "sa.pub",
    "etcd-ca.crt",
    "front-proxy-ca.crt",
    "apiserver.crt",
    "apiserver.key",
    "apiserver-etcd-client.crt",
    "apiserver-etcd-client.key",
    "front-proxy-client.crt",
    "front-proxy-client.key",
    "apiserver-kubelet-client.crt",
    "apiserver-kubelet-client.key",
    "controller-manager.crt",
    "controller-manager.key",
    "scheduler.crt",
    "scheduler.key",
    "admin.crt",
    "admin.key",
];

impl K8sSecrets {
    /// An empty secret bundle.
    pub fn new() -> Self {
        K8sSecrets {
            entries: BTreeMap::new(),
        }
    }

    /// Insert or replace a secret, returning the bundle for chaining.
    pub fn with(mut self, name: impl Into<String>, data: impl Into<Vec<u8>>) -> Self {
        self.entries.insert(name.into(), data.into());
        self
    }

    /// Insert a secret.
    pub fn insert(&mut self, name: impl Into<String>, data: impl Into<Vec<u8>>) {
        self.entries.insert(name.into(), data.into());
    }

    /// Build a bundle from `(name, data)` entries.
    pub fn from_entries<N, D, I>(entries: I) -> Self
    where
        N: Into<String>,
        D: Into<Vec<u8>>,
        I: IntoIterator<Item = (N, D)>,
    {
        let mut secrets = K8sSecrets::new();
        for (name, data) in entries {
            secrets.insert(name, data);
        }
        secrets
    }

    /// Build a bundle from entries and require the full control-plane secret
    /// closure to be present.
    pub fn from_required_entries<N, D, I>(entries: I) -> Result<Self>
    where
        N: Into<String>,
        D: Into<Vec<u8>>,
        I: IntoIterator<Item = (N, D)>,
    {
        let secrets = Self::from_entries(entries);
        secrets.require_complete()?;
        Ok(secrets)
    }

    /// Fetch a secret by name.
    pub fn get(&self, name: &str) -> Option<&[u8]> {
        self.entries.get(name).map(std::vec::Vec::as_slice)
    }

    /// True if every secret in [`REQUIRED_SECRETS`] is present and non-empty.
    pub fn is_complete(&self) -> bool {
        self.missing().is_empty()
    }

    /// The names of required secrets that are absent or empty.
    pub fn missing(&self) -> Vec<&'static str> {
        REQUIRED_SECRETS
            .iter()
            .copied()
            .filter(|name| self.entries.get(*name).is_none_or(std::vec::Vec::is_empty))
            .collect()
    }

    /// Validate the bundle is complete, erroring on the first missing secret.
    pub fn require_complete(&self) -> Result<()> {
        match self.missing().first() {
            None => Ok(()),
            Some(name) => Err(K8sError::MissingSecret(name.to_string())),
        }
    }

    /// Number of secrets stored.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the bundle is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use os_kernel::NodeAddress;
    use oya_cloud_os_secrets_domain::certsans::CertSans;
    use oya_cloud_os_secrets_domain::etcd::{EtcdCert, EtcdController};
    use oya_cloud_os_secrets_domain::kubernetes::{K8sCert, KubernetesController};
    use oya_cloud_os_secrets_domain::{
        CaKind, KUBERNETES_SECRET_PROJECTION_NAMES, ModelPemSecretMaterialEncoder, SecretsBundle,
        kubernetes_secret_entries, kubernetes_secret_entries_with_encoder,
    };

    fn complete() -> K8sSecrets {
        let mut s = K8sSecrets::new();
        for name in REQUIRED_SECRETS {
            s.insert(*name, b"pem".to_vec());
        }
        s
    }

    fn generated_projection() -> (SecretsBundle, KubernetesController, EtcdController) {
        let mut bundle = SecretsBundle::generate("generated-k8s-pki", 1000).unwrap();
        let mut sans = CertSans::new();
        sans.append("api.example.com").unwrap();
        let mut k8s = KubernetesController::new(sans, "cluster.local").unwrap();
        let mut etcd =
            EtcdController::new("cp-1", &[NodeAddress::parse("10.0.0.5").unwrap()]).unwrap();
        k8s.reconcile(&mut bundle, 1000).unwrap();
        etcd.reconcile(&mut bundle, 1000).unwrap();
        (bundle, k8s, etcd)
    }

    #[test]
    fn empty_bundle_is_incomplete() {
        let s = K8sSecrets::new();
        assert!(!s.is_complete());
        assert_eq!(s.missing().len(), REQUIRED_SECRETS.len());
        assert!(s.require_complete().is_err());
    }

    #[test]
    fn complete_bundle_passes() {
        let s = complete();
        assert!(s.is_complete());
        assert!(s.require_complete().is_ok());
        assert_eq!(s.get("ca.crt"), Some(&b"pem"[..]));
    }

    #[test]
    fn empty_value_counts_as_missing() {
        let mut s = complete();
        s.insert("sa.key", Vec::new());
        assert!(!s.is_complete());
        assert_eq!(s.require_complete().unwrap_err().kind(), "missing_secret");
        assert!(s.missing().contains(&"sa.key"));
    }

    #[test]
    fn builder_with_chains() {
        let s = K8sSecrets::new().with("ca.crt", b"x".to_vec());
        assert_eq!(s.len(), 1);
        assert!(!s.is_empty());
    }

    #[test]
    fn from_required_entries_rejects_incomplete_projection() {
        let entries = [("ca.crt", b"pem".to_vec())];
        let err = K8sSecrets::from_required_entries(entries).unwrap_err();
        assert_eq!(err.kind(), "missing_secret");
    }

    #[test]
    fn from_required_entries_accepts_complete_projection() {
        let entries = REQUIRED_SECRETS
            .iter()
            .map(|name| ((*name).to_string(), format!("pem-{name}").into_bytes()));
        let secrets = K8sSecrets::from_required_entries(entries).unwrap();
        assert_eq!(secrets.len(), REQUIRED_SECRETS.len());
        assert_eq!(
            secrets.get("apiserver.crt"),
            Some(&b"pem-apiserver.crt"[..])
        );
    }

    #[test]
    fn generated_k8s_pki_requires_reconciled_leaf_certificates() {
        let bundle = SecretsBundle::generate("generated-k8s-pki-missing", 1000).unwrap();
        let mut sans = CertSans::new();
        sans.append("api.example.com").unwrap();
        let k8s = KubernetesController::new(sans, "cluster.local").unwrap();
        let etcd = EtcdController::new("cp-1", &[NodeAddress::parse("10.0.0.5").unwrap()]).unwrap();

        let err = kubernetes_secret_entries(&bundle, &k8s, &etcd).unwrap_err();
        assert_eq!(err.kind(), "not_found");
    }

    #[test]
    fn generated_k8s_pki_secret_bundle_is_complete() {
        let (bundle, k8s, etcd) = generated_projection();
        let entries = kubernetes_secret_entries(&bundle, &k8s, &etcd).unwrap();
        let names: Vec<&str> = entries.iter().map(|entry| entry.name).collect();
        assert_eq!(names, KUBERNETES_SECRET_PROJECTION_NAMES);
        assert_eq!(names, REQUIRED_SECRETS);

        let secrets =
            K8sSecrets::from_required_entries(entries.into_iter().map(|entry| entry.into_pair()))
                .unwrap();

        assert!(secrets.is_complete());
        assert!(secrets.missing().is_empty());
        for name in REQUIRED_SECRETS {
            assert!(
                secrets.get(name).is_some_and(|data| !data.is_empty()),
                "{name}"
            );
        }
    }

    #[test]
    fn generated_k8s_pki_maps_roots_service_account_and_leaves_to_required_names() {
        let (bundle, k8s, etcd) = generated_projection();
        let entries = kubernetes_secret_entries(&bundle, &k8s, &etcd).unwrap();
        let secrets =
            K8sSecrets::from_required_entries(entries.into_iter().map(|entry| entry.into_pair()))
                .unwrap();

        let expected = [
            (
                "ca.crt",
                bundle
                    .ca(CaKind::Kubernetes)
                    .certificate()
                    .model_certificate_bytes(),
            ),
            (
                "ca.key",
                bundle
                    .ca(CaKind::Kubernetes)
                    .keypair()
                    .model_private_key_bytes(),
            ),
            (
                "etcd-ca.crt",
                bundle
                    .ca(CaKind::Etcd)
                    .certificate()
                    .model_certificate_bytes(),
            ),
            (
                "front-proxy-ca.crt",
                bundle
                    .ca(CaKind::Aggregator)
                    .certificate()
                    .model_certificate_bytes(),
            ),
            (
                "sa.key",
                bundle.service_account_key().model_private_key_bytes(),
            ),
            (
                "sa.pub",
                bundle.service_account_key().model_public_key_bytes(),
            ),
            (
                "apiserver.crt",
                k8s.certificate(K8sCert::ApiServer)
                    .unwrap()
                    .model_certificate_bytes(),
            ),
            (
                "apiserver.key",
                K8sCert::ApiServer.keypair().model_private_key_bytes(),
            ),
            (
                "apiserver-etcd-client.crt",
                etcd.certificate(EtcdCert::ApiServerClient)
                    .unwrap()
                    .model_certificate_bytes(),
            ),
            (
                "apiserver-etcd-client.key",
                EtcdCert::ApiServerClient
                    .keypair(etcd.node_name())
                    .model_private_key_bytes(),
            ),
            (
                "front-proxy-client.crt",
                k8s.certificate(K8sCert::FrontProxy)
                    .unwrap()
                    .model_certificate_bytes(),
            ),
            (
                "front-proxy-client.key",
                K8sCert::FrontProxy.keypair().model_private_key_bytes(),
            ),
            (
                "apiserver-kubelet-client.crt",
                k8s.certificate(K8sCert::ApiServerKubeletClient)
                    .unwrap()
                    .model_certificate_bytes(),
            ),
            (
                "apiserver-kubelet-client.key",
                K8sCert::ApiServerKubeletClient
                    .keypair()
                    .model_private_key_bytes(),
            ),
            (
                "controller-manager.crt",
                k8s.certificate(K8sCert::ControllerManager)
                    .unwrap()
                    .model_certificate_bytes(),
            ),
            (
                "controller-manager.key",
                K8sCert::ControllerManager
                    .keypair()
                    .model_private_key_bytes(),
            ),
            (
                "scheduler.crt",
                k8s.certificate(K8sCert::Scheduler)
                    .unwrap()
                    .model_certificate_bytes(),
            ),
            (
                "scheduler.key",
                K8sCert::Scheduler.keypair().model_private_key_bytes(),
            ),
            (
                "admin.crt",
                k8s.certificate(K8sCert::Admin)
                    .unwrap()
                    .model_certificate_bytes(),
            ),
            (
                "admin.key",
                K8sCert::Admin.keypair().model_private_key_bytes(),
            ),
        ];

        for (name, expected_bytes) in expected {
            assert_eq!(
                secrets.get(name),
                Some(expected_bytes.as_slice()),
                "{name} generated source mapping"
            );
        }
    }

    #[test]
    fn model_pem_encoder_output_builds_complete_k8s_secret_bundle() {
        let (bundle, k8s, etcd) = generated_projection();
        let entries = kubernetes_secret_entries_with_encoder(
            &bundle,
            &k8s,
            &etcd,
            &ModelPemSecretMaterialEncoder,
        )
        .unwrap();
        let names: Vec<&str> = entries.iter().map(|entry| entry.name).collect();
        assert_eq!(names, REQUIRED_SECRETS);

        let secrets =
            K8sSecrets::from_required_entries(entries.into_iter().map(|entry| entry.into_pair()))
                .unwrap();

        assert_eq!(secrets.len(), REQUIRED_SECRETS.len());
        assert!(secrets.is_complete());
        for name in REQUIRED_SECRETS {
            let data = secrets
                .get(name)
                .unwrap_or_else(|| panic!("missing PEM secret {name}"));
            let text = String::from_utf8_lossy(data);
            assert!(text.starts_with("-----BEGIN "), "{name}");
            assert!(!text.contains("KUBEROS-MODEL-"), "{name}");
        }
    }
}
