//! # talos-k8s-control
//!
//! Models the Talos-managed Kubernetes control plane controllers, mirroring
//! `internal/app/machined/pkg/controllers/k8s` (control-plane domain) and
//! `internal/app/machined/pkg/controllers/kubeaccess` in `siderolabs/talos`.
//!
//! The crate is pure logic; every OS boundary (filesystem, the API server) is a
//! trait with an in-memory implementation used by the tests.
//!
//! ## Modules
//!
//! * [`static_pod_controller`] — the `ControlPlaneStaticPodController`: reconciles
//!   per-component specs (kube-apiserver / kube-controller-manager /
//!   kube-scheduler) into static pod manifests written under
//!   `/etc/kubernetes/manifests`.
//! * [`manifest_controller`] — the bootstrap + extra-manifest controllers:
//!   collects, orders and applies cluster manifests (kube-proxy, `CoreDNS`,
//!   bootstrap RBAC, inline/remote user manifests).
//! * [`render_secrets`] — the `RenderSecretsStaticPodController`: lays out the
//!   control-plane PKI bundle as files the static pods mount.
//! * [`admission`] — admission-control, audit-policy and secrets-encryption
//!   config rendering.
//! * [`kubeaccess`] — the kube-apiserver `KubeAccess` endpoint config and the
//!   in-cluster Talos-API access authorization policy.
//! * [`rendered`] — the rendered-file model and the filesystem `FileSink`
//!   boundary trait.
//! * [`error`] — the crate-local [`ControlError`] enum.

pub mod admission;
pub mod error;
pub mod kubeaccess;
pub mod manifest_controller;
pub mod render_secrets;
pub mod rendered;
pub mod static_pod_controller;

pub use admission::{
    AdmissionControlConfig, AdmissionPluginConfig, AuditLevel, AuditPolicyConfig, EncryptionConfig,
    EncryptionProvider, PodSecurityConfig, PodSecurityLevel,
};
pub use error::{ControlError, Result};
pub use kubeaccess::{AccessDecision, AccessRequest, KubeAccessConfig, authorize};
pub use manifest_controller::{
    ClusterManifest, InMemoryApplier, ManifestApplier, ManifestController, ManifestSource,
};
pub use render_secrets::{RenderSecretsController, SecretItem, SecretsBundle};
pub use rendered::{FileMode, FileSink, InMemoryFileSink, RenderedFile, RenderedOutput};
pub use static_pod_controller::{
    ComponentSpec, ControlPlaneComponent, ReconcileResult, StaticPodController, StaticPodPhase,
};

/// The default Kubernetes version this crate targets when none is configured.
pub const DEFAULT_KUBERNETES_VERSION: &str = "1.30.0";

/// Top-level Kubernetes control-plane configuration, mirroring the
/// `K8sControlPlane` machine-config-derived resource that drives all of the
/// controllers in this crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct K8sControlConfig {
    /// Kubernetes version (drives the container image tags).
    pub kubernetes_version: String,
    /// Container image registry prefix (e.g. `registry.k8s.io`).
    pub registry: String,
    /// Pod-security / admission configuration.
    pub admission: AdmissionControlConfig,
    /// Audit-policy configuration.
    pub audit: AuditPolicyConfig,
    /// Optional secrets-at-rest encryption configuration.
    pub encryption: Option<EncryptionConfig>,
    /// `KubeAccess` endpoint configuration.
    pub kube_access: KubeAccessConfig,
}

impl Default for K8sControlConfig {
    fn default() -> Self {
        K8sControlConfig {
            kubernetes_version: DEFAULT_KUBERNETES_VERSION.to_string(),
            registry: "registry.k8s.io".to_string(),
            admission: AdmissionControlConfig::with_defaults(),
            audit: AuditPolicyConfig::default(),
            encryption: None,
            kube_access: KubeAccessConfig::disabled(),
        }
    }
}

impl K8sControlConfig {
    /// The fully-qualified image reference for a control-plane component.
    #[must_use]
    pub fn image_for(&self, component: ControlPlaneComponent) -> String {
        format!(
            "{}/{}:v{}",
            self.registry,
            component.name(),
            self.kubernetes_version
        )
    }

    /// Validate the cross-cutting invariants of the config.
    pub fn validate(&self) -> Result<()> {
        if self.kubernetes_version.trim().is_empty() {
            return Err(ControlError::InvalidConfig(
                "kubernetes version is empty".into(),
            ));
        }
        if self.registry.trim().is_empty() {
            return Err(ControlError::InvalidConfig("registry is empty".into()));
        }
        Ok(())
    }

    /// Build the default per-component static-pod specs from this config.
    pub fn component_specs(&self) -> Result<Vec<ComponentSpec>> {
        self.validate()?;
        let mut specs = Vec::new();
        for component in ControlPlaneComponent::ALL {
            specs.push(ComponentSpec::new(component, self.image_for(component))?);
        }
        Ok(specs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let cfg = K8sControlConfig::default();
        assert!(cfg.validate().is_ok());
        assert_eq!(cfg.kubernetes_version, DEFAULT_KUBERNETES_VERSION);
        assert!(!cfg.kube_access.is_enabled());
        assert!(cfg.encryption.is_none());
    }

    #[test]
    fn image_for_renders_reference() {
        let cfg = K8sControlConfig::default();
        assert_eq!(
            cfg.image_for(ControlPlaneComponent::ApiServer),
            "registry.k8s.io/kube-apiserver:v1.30.0"
        );
    }

    #[test]
    fn validate_rejects_empty_fields() {
        let cfg = K8sControlConfig {
            registry: "  ".into(),
            ..K8sControlConfig::default()
        };
        assert_eq!(cfg.validate().unwrap_err().kind(), "invalid_config");
    }

    #[test]
    fn component_specs_cover_all_components() {
        let cfg = K8sControlConfig::default();
        let specs = cfg.component_specs().unwrap();
        assert_eq!(specs.len(), 3);
        assert!(
            specs
                .iter()
                .any(|s| s.component == ControlPlaneComponent::Scheduler)
        );
    }

    #[test]
    fn end_to_end_render_and_reconcile() {
        // Build config, derive specs, reconcile into static pods, render
        // secrets, and flush — exercising the whole crate surface.
        let cfg = K8sControlConfig::default();
        let specs = cfg.component_specs().unwrap();

        let mut ctrl = StaticPodController::new();
        let res = ctrl.reconcile(&specs).unwrap();
        assert_eq!(res.output.len(), 3);

        let mut sink = InMemoryFileSink::new();
        res.output.flush(&mut sink).unwrap();
        assert!(
            sink.get("/etc/kubernetes/manifests/kube-apiserver.yaml")
                .is_some()
        );

        let bundle = SecretsBundle {
            ca_crt: b"c".to_vec(),
            ca_key: b"k".to_vec(),
            apiserver_crt: b"c".to_vec(),
            apiserver_key: b"k".to_vec(),
            service_account_key: b"k".to_vec(),
            etcd_client_crt: b"c".to_vec(),
            etcd_client_key: b"k".to_vec(),
        };
        let secrets_out = RenderSecretsController.render(&bundle).unwrap();
        assert_eq!(secrets_out.len(), 10);

        // And apply the bootstrap + extra manifests.
        let mut mc = ManifestController::new();
        mc.add(
            ClusterManifest::new("kube-proxy", ManifestSource::Bootstrap, "kind: DaemonSet")
                .unwrap(),
        )
        .unwrap();
        mc.add(ClusterManifest::new("user-cm", ManifestSource::Remote, "kind: ConfigMap").unwrap())
            .unwrap();
        let mut applier = InMemoryApplier::new();
        assert_eq!(mc.reconcile(&mut applier).unwrap(), 2);
        assert_eq!(applier.applied()[0], "kube-proxy");
    }

    #[test]
    fn config_with_encryption_and_kubeaccess() {
        let cfg = K8sControlConfig {
            encryption: Some(EncryptionConfig::default_secrets(vec![1u8; 32]).unwrap()),
            kube_access: KubeAccessConfig::enabled(
                vec![os_kernel::Role::Reader],
                vec!["kube-system".into()],
            )
            .unwrap(),
            ..K8sControlConfig::default()
        };
        assert!(cfg.validate().is_ok());
        assert!(cfg.kube_access.is_enabled());
        assert_eq!(
            cfg.encryption.as_ref().unwrap().provider,
            EncryptionProvider::AesGcm
        );
    }
}
