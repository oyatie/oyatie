//! Render-secrets controller for control-plane static pods.
//!
//! Mirrors Talos `RenderSecretsStaticPodController` under
//! `internal/app/machined/pkg/controllers/k8s`: it takes the in-memory
//! Kubernetes PKI secrets bundle and renders it to files under
//! `/system/secrets/kubernetes/<component>/` that the static pods mount. Secret
//! files are written with mode 0600; CA certificates with 0644.
//!
//! The PKI material itself is opaque bytes here; the crypto generation lives in
//! the secrets crates. This controller is the file-layout + reconcile logic.

use crate::error::{ControlError, Result};
use crate::rendered::{FileMode, RenderedFile, RenderedOutput};
use crate::static_pod_controller::ControlPlaneComponent;

/// Root directory the rendered secrets live under.
pub const SECRETS_ROOT: &str = "/system/secrets/kubernetes";

/// A logical secret item to render: a filename within a component directory and
/// its bytes, plus whether it is private (key) material.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretItem {
    /// Filename, e.g. `apiserver.crt`.
    pub filename: String,
    /// Raw bytes (PEM, key, kubeconfig, ...).
    pub data: Vec<u8>,
    /// Whether this is private key material (rendered 0600).
    pub private: bool,
}

impl SecretItem {
    /// A certificate / public item (mode 0644).
    pub fn cert(filename: impl Into<String>, data: impl Into<Vec<u8>>) -> Self {
        SecretItem {
            filename: filename.into(),
            data: data.into(),
            private: false,
        }
    }

    /// A private key / token item (mode 0600).
    pub fn key(filename: impl Into<String>, data: impl Into<Vec<u8>>) -> Self {
        SecretItem {
            filename: filename.into(),
            data: data.into(),
            private: true,
        }
    }

    /// The file mode this item renders with.
    #[must_use]
    pub fn mode(&self) -> FileMode {
        if self.private {
            FileMode::SECRET
        } else {
            FileMode::CONFIG
        }
    }
}

/// The input PKI bundle. Mirrors the subset of `secrets.Kubernetes` the
/// render-secrets controller consumes: the cluster CA, the API server cert/key,
/// the service-account key, and the etcd client material.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SecretsBundle {
    /// Cluster CA certificate (PEM).
    pub ca_crt: Vec<u8>,
    /// Cluster CA private key (PEM).
    pub ca_key: Vec<u8>,
    /// kube-apiserver serving cert (PEM).
    pub apiserver_crt: Vec<u8>,
    /// kube-apiserver serving key (PEM).
    pub apiserver_key: Vec<u8>,
    /// Service-account signing key (PEM).
    pub service_account_key: Vec<u8>,
    /// etcd client cert used by the apiserver (PEM).
    pub etcd_client_crt: Vec<u8>,
    /// etcd client key used by the apiserver (PEM).
    pub etcd_client_key: Vec<u8>,
}

impl SecretsBundle {
    /// Validate that all required fields are populated. The render-secrets
    /// controller refuses to write a partial bundle.
    pub fn validate(&self) -> Result<()> {
        let checks: [(&str, &[u8]); 7] = [
            ("ca.crt", &self.ca_crt),
            ("ca.key", &self.ca_key),
            ("apiserver.crt", &self.apiserver_crt),
            ("apiserver.key", &self.apiserver_key),
            ("service-account.key", &self.service_account_key),
            ("apiserver-etcd-client.crt", &self.etcd_client_crt),
            ("apiserver-etcd-client.key", &self.etcd_client_key),
        ];
        for (name, bytes) in checks {
            if bytes.is_empty() {
                return Err(ControlError::MissingSecret(name.into()));
            }
        }
        Ok(())
    }

    /// The secret items the kube-apiserver component needs.
    fn apiserver_items(&self) -> Vec<SecretItem> {
        vec![
            SecretItem::cert("ca.crt", self.ca_crt.clone()),
            SecretItem::cert("apiserver.crt", self.apiserver_crt.clone()),
            SecretItem::key("apiserver.key", self.apiserver_key.clone()),
            SecretItem::key("service-account.key", self.service_account_key.clone()),
            SecretItem::cert("apiserver-etcd-client.crt", self.etcd_client_crt.clone()),
            SecretItem::key("apiserver-etcd-client.key", self.etcd_client_key.clone()),
        ]
    }

    /// The secret items the controller-manager needs (CA + key for signing).
    fn controller_manager_items(&self) -> Vec<SecretItem> {
        vec![
            SecretItem::cert("ca.crt", self.ca_crt.clone()),
            SecretItem::key("ca.key", self.ca_key.clone()),
            SecretItem::key("service-account.key", self.service_account_key.clone()),
        ]
    }

    /// The secret items the scheduler needs (just the CA to verify the API).
    fn scheduler_items(&self) -> Vec<SecretItem> {
        vec![SecretItem::cert("ca.crt", self.ca_crt.clone())]
    }

    /// Items for a given component.
    #[must_use]
    pub fn items_for(&self, component: ControlPlaneComponent) -> Vec<SecretItem> {
        match component {
            ControlPlaneComponent::ApiServer => self.apiserver_items(),
            ControlPlaneComponent::ControllerManager => self.controller_manager_items(),
            ControlPlaneComponent::Scheduler => self.scheduler_items(),
        }
    }
}

/// Directory a component's secrets render into.
#[must_use]
pub fn component_dir(component: ControlPlaneComponent) -> String {
    format!("{SECRETS_ROOT}/{}", component.name())
}

/// The render-secrets controller. Stateless: given a validated bundle it
/// produces the full set of rendered files for all control-plane components.
#[derive(Debug, Default)]
pub struct RenderSecretsController;

impl RenderSecretsController {
    /// Render the secrets for all control-plane components into a
    /// [`RenderedOutput`]. Fails if the bundle is incomplete.
    pub fn render(&self, bundle: &SecretsBundle) -> Result<RenderedOutput> {
        bundle.validate()?;
        let mut out = RenderedOutput::new();
        for component in ControlPlaneComponent::ALL {
            let dir = component_dir(component);
            for item in bundle.items_for(component) {
                let path = format!("{dir}/{}", item.filename);
                let mode = item.mode();
                out.add(RenderedFile::new(path, item.data, mode)?)?;
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendered::InMemoryFileSink;

    fn full_bundle() -> SecretsBundle {
        SecretsBundle {
            ca_crt: b"CA-CRT".to_vec(),
            ca_key: b"CA-KEY".to_vec(),
            apiserver_crt: b"API-CRT".to_vec(),
            apiserver_key: b"API-KEY".to_vec(),
            service_account_key: b"SA-KEY".to_vec(),
            etcd_client_crt: b"ETCD-CRT".to_vec(),
            etcd_client_key: b"ETCD-KEY".to_vec(),
        }
    }

    #[test]
    fn validate_rejects_incomplete_bundle() {
        let mut b = full_bundle();
        b.apiserver_key.clear();
        let err = b.validate().unwrap_err();
        assert_eq!(err.kind(), "missing_secret");
        assert!(full_bundle().validate().is_ok());
    }

    #[test]
    fn render_fails_on_missing_secret() {
        let ctrl = RenderSecretsController;
        let b = SecretsBundle::default();
        assert_eq!(ctrl.render(&b).unwrap_err().kind(), "missing_secret");
    }

    #[test]
    fn keys_are_private_certs_are_world_readable() {
        assert!(SecretItem::key("x.key", b"k".to_vec()).mode() == FileMode::SECRET);
        assert!(SecretItem::cert("x.crt", b"c".to_vec()).mode() == FileMode::CONFIG);
    }

    #[test]
    fn render_produces_expected_layout() {
        let ctrl = RenderSecretsController;
        let out = ctrl.render(&full_bundle()).unwrap();

        let api_key = out
            .get("/system/secrets/kubernetes/kube-apiserver/apiserver.key")
            .expect("apiserver key rendered");
        assert!(api_key.is_secret());

        let ca = out
            .get("/system/secrets/kubernetes/kube-apiserver/ca.crt")
            .expect("ca cert rendered");
        assert!(!ca.is_secret());

        // Scheduler only gets the CA.
        assert!(
            out.get("/system/secrets/kubernetes/kube-scheduler/ca.crt")
                .is_some()
        );
        assert!(
            out.get("/system/secrets/kubernetes/kube-scheduler/apiserver.key")
                .is_none()
        );
    }

    #[test]
    fn render_flushes_through_sink() {
        let ctrl = RenderSecretsController;
        let out = ctrl.render(&full_bundle()).unwrap();
        let mut sink = InMemoryFileSink::new();
        out.flush(&mut sink).unwrap();
        // apiserver(6) + controller-manager(3) + scheduler(1) = 10 files.
        assert_eq!(sink.count(), 10);
        assert_eq!(out.len(), 10);
    }

    #[test]
    fn controller_manager_gets_ca_key() {
        let b = full_bundle();
        let items = b.items_for(ControlPlaneComponent::ControllerManager);
        assert!(items.iter().any(|i| i.filename == "ca.key" && i.private));
    }
}
