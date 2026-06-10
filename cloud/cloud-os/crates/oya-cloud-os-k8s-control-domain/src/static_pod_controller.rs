//! Control-plane static-pod controller.
//!
//! Mirrors Talos `ControlPlaneStaticPodController` under
//! `internal/app/machined/pkg/controllers/k8s`: it consumes per-component
//! control-plane specs (kube-apiserver, kube-controller-manager, kube-scheduler)
//! together with the rendered secrets and admission/audit/encryption config, and
//! reconciles them into static pod manifests written to
//! `/etc/kubernetes/manifests`, which the kubelet then runs.
//!
//! The controller is a pure reconcile function over an in-memory desired/actual
//! model; the filesystem boundary is the [`crate::rendered::FileSink`] trait.

use crate::error::{ControlError, Result};
use crate::rendered::{FileMode, RenderedFile, RenderedOutput};
use std::collections::BTreeMap;

/// Directory the kubelet watches for static pod manifests.
pub const STATIC_POD_PATH: &str = "/etc/kubernetes/manifests";

/// The namespace control-plane static pods run in.
pub const CONTROL_PLANE_NAMESPACE: &str = "kube-system";

/// The three control-plane components Talos manages as static pods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ControlPlaneComponent {
    /// kube-apiserver.
    ApiServer,
    /// kube-controller-manager.
    ControllerManager,
    /// kube-scheduler.
    Scheduler,
}

impl ControlPlaneComponent {
    /// All components in canonical (apply) order.
    pub const ALL: [ControlPlaneComponent; 3] = [
        ControlPlaneComponent::ApiServer,
        ControlPlaneComponent::ControllerManager,
        ControlPlaneComponent::Scheduler,
    ];

    /// The pod / binary name.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            ControlPlaneComponent::ApiServer => "kube-apiserver",
            ControlPlaneComponent::ControllerManager => "kube-controller-manager",
            ControlPlaneComponent::Scheduler => "kube-scheduler",
        }
    }

    /// The static pod manifest filename.
    #[must_use]
    pub fn manifest_filename(self) -> String {
        format!("{}.yaml", self.name())
    }

    /// Absolute path of the manifest under [`STATIC_POD_PATH`].
    #[must_use]
    pub fn manifest_path(self) -> String {
        format!("{STATIC_POD_PATH}/{}", self.manifest_filename())
    }

    /// Whether this component must bind to a host port (apiserver does).
    #[must_use]
    pub fn is_host_network(self) -> bool {
        matches!(self, ControlPlaneComponent::ApiServer)
    }
}

/// The desired spec for one control-plane component, mirroring the
/// `K8sControlPlane<Component>` resources Talos produces from machine config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentSpec {
    /// Which component this is.
    pub component: ControlPlaneComponent,
    /// Container image (e.g. `registry.k8s.io/kube-apiserver:v1.30.0`).
    pub image: String,
    /// Ordered command-line flags, already `--flag=value` formatted.
    pub extra_args: Vec<String>,
    /// Extra host-path volume mounts (host path -> container path).
    pub extra_volumes: BTreeMap<String, String>,
}

impl ComponentSpec {
    /// Build a spec, validating the image is non-empty.
    pub fn new(component: ControlPlaneComponent, image: impl Into<String>) -> Result<Self> {
        let image = image.into();
        if image.trim().is_empty() {
            return Err(ControlError::InvalidConfig(format!(
                "{} image is empty",
                component.name()
            )));
        }
        Ok(ComponentSpec {
            component,
            image,
            extra_args: Vec::new(),
            extra_volumes: BTreeMap::new(),
        })
    }

    /// Add a flag, rejecting one that isn't `--flag` / `--flag=value` shaped.
    pub fn with_arg(mut self, arg: impl Into<String>) -> Result<Self> {
        let arg = arg.into();
        if !arg.starts_with("--") {
            return Err(ControlError::InvalidConfig(format!(
                "flag must start with '--': {arg}"
            )));
        }
        self.extra_args.push(arg);
        Ok(self)
    }

    /// Add a host-path volume mount.
    pub fn with_volume(mut self, host: impl Into<String>, container: impl Into<String>) -> Self {
        self.extra_volumes.insert(host.into(), container.into());
        self
    }

    /// Render the static pod manifest YAML for this component.
    ///
    /// Deterministic: flags are emitted in insertion order, volumes sorted by
    /// host path (`BTreeMap` iteration order).
    #[must_use]
    pub fn render_manifest(&self) -> String {
        let mut out = String::new();
        out.push_str("apiVersion: v1\nkind: Pod\nmetadata:\n");
        out.push_str(&format!("  name: {}\n", self.component.name()));
        out.push_str(&format!("  namespace: {CONTROL_PLANE_NAMESPACE}\n"));
        out.push_str("  labels:\n");
        out.push_str(&format!("    k8s-app: {}\n", self.component.name()));
        out.push_str("    tier: control-plane\n");
        out.push_str("spec:\n");
        if self.component.is_host_network() {
            out.push_str("  hostNetwork: true\n");
        }
        out.push_str("  priorityClassName: system-node-critical\n");
        out.push_str("  containers:\n");
        out.push_str(&format!("    - name: {}\n", self.component.name()));
        out.push_str(&format!("      image: {}\n", self.image));
        out.push_str("      command:\n");
        out.push_str(&format!("        - {}\n", self.component.name()));
        for arg in &self.extra_args {
            out.push_str(&format!("        - {arg}\n"));
        }
        if !self.extra_volumes.is_empty() {
            out.push_str("      volumeMounts:\n");
            for (i, (host, container)) in self.extra_volumes.iter().enumerate() {
                out.push_str(&format!("        - name: vol-{i}\n"));
                out.push_str(&format!("          mountPath: {container}\n"));
                let _ = host;
            }
            out.push_str("  volumes:\n");
            for (i, (host, _container)) in self.extra_volumes.iter().enumerate() {
                out.push_str(&format!("    - name: vol-{i}\n"));
                out.push_str("      hostPath:\n");
                out.push_str(&format!("        path: {host}\n"));
            }
        }
        out
    }
}

/// Phase of a single static pod as tracked by the controller's reconcile loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticPodPhase {
    /// Spec known, manifest not yet written.
    Pending,
    /// Manifest written to the static-pod path.
    Applied,
    /// Spec was removed; manifest must be deleted.
    Teardown,
}

/// In-memory view of what the controller has already applied. The real
/// controller compares against the COSI resource state; here we track applied
/// manifests by component so reconcile is idempotent and computes a diff.
#[derive(Debug, Default)]
pub struct StaticPodController {
    applied: BTreeMap<ControlPlaneComponent, String>,
}

/// The outcome of a single reconcile: the files to (re)write and the paths to
/// delete for components no longer desired.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReconcileResult {
    /// Files to write (changed or new manifests).
    pub output: RenderedOutput,
    /// Manifest paths to remove (teardown).
    pub deletions: Vec<String>,
}

impl StaticPodController {
    /// A fresh controller with nothing applied yet.
    #[must_use]
    pub fn new() -> Self {
        StaticPodController {
            applied: BTreeMap::new(),
        }
    }

    /// Components currently applied.
    #[must_use]
    pub fn applied_components(&self) -> Vec<ControlPlaneComponent> {
        self.applied.keys().copied().collect()
    }

    /// Reconcile the desired set of specs against the applied state.
    ///
    /// Returns the files that changed (new content or new component) and the
    /// manifest paths for components that are no longer desired. The internal
    /// applied-state is updated to reflect the new desired set, so a second
    /// reconcile with the same input yields no writes (idempotent).
    pub fn reconcile(&mut self, specs: &[ComponentSpec]) -> Result<ReconcileResult> {
        // Reject duplicate components in the desired set.
        let mut seen = std::collections::BTreeSet::new();
        for s in specs {
            if !seen.insert(s.component) {
                return Err(ControlError::Reconcile(format!(
                    "duplicate component in desired set: {}",
                    s.component.name()
                )));
            }
        }

        let mut result = ReconcileResult::default();
        let desired: BTreeMap<ControlPlaneComponent, String> = specs
            .iter()
            .map(|s| (s.component, s.render_manifest()))
            .collect();

        // Writes: new or changed manifests.
        for (component, manifest) in &desired {
            let changed = self.applied.get(component) != Some(manifest);
            if changed {
                let file = RenderedFile::new(
                    component.manifest_path(),
                    manifest.clone().into_bytes(),
                    FileMode::CONFIG,
                )?;
                result.output.add(file)?;
            }
        }

        // Deletions: applied components no longer desired.
        for component in self.applied.keys().copied().collect::<Vec<_>>() {
            if !desired.contains_key(&component) {
                result.deletions.push(component.manifest_path());
            }
        }

        // Commit the new applied state.
        self.applied = desired;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apiserver() -> ComponentSpec {
        ComponentSpec::new(
            ControlPlaneComponent::ApiServer,
            "registry.k8s.io/kube-apiserver:v1.30.0",
        )
        .unwrap()
        .with_arg("--secure-port=6443")
        .unwrap()
        .with_arg("--etcd-servers=https://127.0.0.1:2379")
        .unwrap()
    }

    #[test]
    fn component_paths() {
        assert_eq!(
            ControlPlaneComponent::ApiServer.manifest_path(),
            "/etc/kubernetes/manifests/kube-apiserver.yaml"
        );
        assert!(ControlPlaneComponent::ApiServer.is_host_network());
        assert!(!ControlPlaneComponent::Scheduler.is_host_network());
        assert_eq!(ControlPlaneComponent::ALL.len(), 3);
    }

    #[test]
    fn spec_validates_image_and_flags() {
        assert!(ComponentSpec::new(ControlPlaneComponent::Scheduler, "  ").is_err());
        let bad = ComponentSpec::new(ControlPlaneComponent::Scheduler, "img")
            .unwrap()
            .with_arg("no-dashes");
        assert_eq!(bad.unwrap_err().kind(), "invalid_config");
    }

    #[test]
    fn manifest_contains_expected_fields() {
        let m = apiserver().render_manifest();
        assert!(m.contains("name: kube-apiserver"));
        assert!(m.contains("namespace: kube-system"));
        assert!(m.contains("hostNetwork: true"));
        assert!(m.contains("--secure-port=6443"));
        assert!(m.contains("system-node-critical"));
    }

    #[test]
    fn scheduler_manifest_is_not_host_network() {
        let spec = ComponentSpec::new(ControlPlaneComponent::Scheduler, "img").unwrap();
        let m = spec.render_manifest();
        assert!(!m.contains("hostNetwork"));
    }

    #[test]
    fn manifest_renders_volumes() {
        let spec = ComponentSpec::new(ControlPlaneComponent::ApiServer, "img")
            .unwrap()
            .with_volume("/etc/kubernetes/pki", "/etc/kubernetes/pki");
        let m = spec.render_manifest();
        assert!(m.contains("volumeMounts"));
        assert!(m.contains("hostPath"));
        assert!(m.contains("path: /etc/kubernetes/pki"));
    }

    #[test]
    fn reconcile_writes_then_is_idempotent() {
        let mut ctrl = StaticPodController::new();
        let specs = vec![
            apiserver(),
            ComponentSpec::new(ControlPlaneComponent::Scheduler, "img").unwrap(),
        ];
        let first = ctrl.reconcile(&specs).unwrap();
        assert_eq!(first.output.len(), 2);
        assert!(first.deletions.is_empty());

        let second = ctrl.reconcile(&specs).unwrap();
        assert_eq!(second.output.len(), 0);
        assert!(second.deletions.is_empty());
    }

    #[test]
    fn reconcile_detects_change() {
        let mut ctrl = StaticPodController::new();
        let s1 = vec![ComponentSpec::new(ControlPlaneComponent::Scheduler, "img:v1").unwrap()];
        ctrl.reconcile(&s1).unwrap();
        let s2 = vec![ComponentSpec::new(ControlPlaneComponent::Scheduler, "img:v2").unwrap()];
        let r = ctrl.reconcile(&s2).unwrap();
        assert_eq!(r.output.len(), 1);
        assert!(
            r.output
                .get(&ControlPlaneComponent::Scheduler.manifest_path())
                .is_some()
        );
    }

    #[test]
    fn reconcile_tears_down_removed_component() {
        let mut ctrl = StaticPodController::new();
        let both = vec![
            apiserver(),
            ComponentSpec::new(ControlPlaneComponent::Scheduler, "img").unwrap(),
        ];
        ctrl.reconcile(&both).unwrap();
        let only_apiserver = vec![apiserver()];
        let r = ctrl.reconcile(&only_apiserver).unwrap();
        assert_eq!(r.output.len(), 0);
        assert_eq!(
            r.deletions,
            vec![ControlPlaneComponent::Scheduler.manifest_path()]
        );
        assert_eq!(
            ctrl.applied_components(),
            vec![ControlPlaneComponent::ApiServer]
        );
    }

    #[test]
    fn reconcile_rejects_duplicate_components() {
        let mut ctrl = StaticPodController::new();
        let dup = vec![apiserver(), apiserver()];
        assert_eq!(ctrl.reconcile(&dup).unwrap_err().kind(), "reconcile");
    }
}
