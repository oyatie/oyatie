//! Static pod manifests for control-plane components.
//!
//! Mirrors Talos `k8s.StaticPod` / `StaticPodServerController`: the kubelet
//! watches [`crate::STATIC_POD_PATH`] and runs whatever pod manifests it finds
//! there. We model the manifest as structured data plus a small lifecycle state
//! machine describing the kubelet's reconciliation of the pod.

use crate::error::{K8sError, Result};

/// The lifecycle phase of a static pod as reported back by the kubelet.
///
/// This mirrors the Kubernetes pod phase the mirror-pod reports to the
/// apiserver, narrowed to the transitions the static-pod controller cares
/// about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticPodPhase {
    /// Manifest written; kubelet has not yet acted.
    Pending,
    /// Image pulled and container created.
    Creating,
    /// At least one container is running.
    Running,
    /// All containers terminated successfully.
    Succeeded,
    /// A container terminated with failure.
    Failed,
}

impl StaticPodPhase {
    /// Whether the pod has reached a terminal phase.
    pub fn is_terminal(self) -> bool {
        matches!(self, StaticPodPhase::Succeeded | StaticPodPhase::Failed)
    }

    /// Validate a phase transition, mirroring the kubelet's monotonic lifecycle.
    pub fn can_transition_to(self, next: StaticPodPhase) -> bool {
        use StaticPodPhase::{Creating, Failed, Pending, Running, Succeeded};
        match (self, next) {
            (Pending, Creating) | (Creating, Running | Failed) | (Running, Succeeded | Failed) => {
                true
            }
            // Idempotent self-transitions are allowed.
            (a, b) if a == b => true,
            _ => false,
        }
    }
}

/// A single container in a static pod.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    /// Container name (also the component name for control-plane pods).
    pub name: String,
    /// Fully qualified image reference.
    pub image: String,
    /// The command + args the container runs.
    pub command: Vec<String>,
    /// Whether the container exposes host networking (control plane does).
    pub host_network: bool,
}

/// A static pod manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticPod {
    /// Pod name, e.g. `kube-apiserver`.
    pub name: String,
    /// Namespace, normally [`crate::CONTROL_PLANE_NAMESPACE`].
    pub namespace: String,
    /// The pod's containers (control-plane pods have exactly one).
    pub containers: Vec<Container>,
    /// Current lifecycle phase.
    pub phase: StaticPodPhase,
}

impl StaticPod {
    /// Build a single-container control-plane static pod in the kube-system
    /// namespace, in the [`StaticPodPhase::Pending`] phase.
    pub fn control_plane(
        name: impl Into<String>,
        image: impl Into<String>,
        command: Vec<String>,
    ) -> Result<Self> {
        let name = name.into();
        let image = image.into();
        if name.is_empty() {
            return Err(K8sError::Render("static pod name empty".to_string()));
        }
        if image.is_empty() {
            return Err(K8sError::Render(format!("static pod {name} has no image")));
        }
        if command.is_empty() {
            return Err(K8sError::Render(format!(
                "static pod {name} has no command"
            )));
        }
        Ok(StaticPod {
            name: name.clone(),
            namespace: crate::CONTROL_PLANE_NAMESPACE.to_string(),
            containers: vec![Container {
                name,
                image,
                command,
                host_network: true,
            }],
            phase: StaticPodPhase::Pending,
        })
    }

    /// The filename the kubelet expects this manifest under.
    pub fn manifest_filename(&self) -> String {
        format!("{}.yaml", self.name)
    }

    /// The mirror-pod name the kubelet registers with the apiserver:
    /// `<name>-<node>`.
    pub fn mirror_pod_name(&self, node: &str) -> String {
        format!("{}-{}", self.name, node)
    }

    /// Advance the pod's phase, enforcing the lifecycle state machine.
    pub fn advance_to(&mut self, next: StaticPodPhase) -> Result<()> {
        if !self.phase.can_transition_to(next) {
            return Err(K8sError::EtcdState(format!(
                "illegal static pod transition {:?} -> {:?}",
                self.phase, next
            )));
        }
        self.phase = next;
        Ok(())
    }

    /// Render a minimal pod manifest body (a deterministic pseudo-YAML).
    pub fn render(&self) -> String {
        use std::fmt::Write as _;

        let mut out = String::new();
        out.push_str("apiVersion: v1\nkind: Pod\nmetadata:\n");
        let _ = writeln!(out, "  name: {}", self.name);
        let _ = writeln!(out, "  namespace: {}", self.namespace);
        out.push_str("spec:\n");
        if self.containers.iter().any(|c| c.host_network) {
            out.push_str("  hostNetwork: true\n");
        }
        out.push_str("  containers:\n");
        for c in &self.containers {
            let _ = writeln!(out, "  - name: {}", c.name);
            let _ = writeln!(out, "    image: {}", c.image);
            out.push_str("    command:\n");
            for part in &c.command {
                let _ = writeln!(out, "    - {part}");
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pod() -> StaticPod {
        StaticPod::control_plane(
            "kube-apiserver",
            "registry.k8s.io/kube-apiserver:v1.30.0",
            vec!["kube-apiserver".into(), "--secure-port=6443".into()],
        )
        .unwrap()
    }

    #[test]
    fn builds_in_kube_system_pending() {
        let p = pod();
        assert_eq!(p.namespace, "kube-system");
        assert_eq!(p.phase, StaticPodPhase::Pending);
        assert!(p.containers[0].host_network);
    }

    #[test]
    fn rejects_empty_fields() {
        assert!(StaticPod::control_plane("", "img", vec!["x".into()]).is_err());
        assert!(StaticPod::control_plane("n", "", vec!["x".into()]).is_err());
        assert!(StaticPod::control_plane("n", "img", vec![]).is_err());
    }

    #[test]
    fn filename_and_mirror_name() {
        let p = pod();
        assert_eq!(p.manifest_filename(), "kube-apiserver.yaml");
        assert_eq!(p.mirror_pod_name("cp-1"), "kube-apiserver-cp-1");
    }

    #[test]
    fn lifecycle_transitions_are_enforced() {
        let mut p = pod();
        assert!(p.advance_to(StaticPodPhase::Creating).is_ok());
        assert!(p.advance_to(StaticPodPhase::Running).is_ok());
        // Cannot jump backwards.
        assert!(p.advance_to(StaticPodPhase::Pending).is_err());
        assert!(p.advance_to(StaticPodPhase::Succeeded).is_ok());
        assert!(p.phase.is_terminal());
    }

    #[test]
    fn render_includes_image_and_command() {
        let y = pod().render();
        assert!(y.contains("kind: Pod"));
        assert!(y.contains("image: registry.k8s.io/kube-apiserver:v1.30.0"));
        assert!(y.contains("- --secure-port=6443"));
        assert!(y.contains("hostNetwork: true"));
    }

    #[test]
    fn illegal_transition_is_invalid_state_kind() {
        let mut p = pod();
        let err = p.advance_to(StaticPodPhase::Running).unwrap_err();
        assert_eq!(err.kind(), "etcd_state");
    }
}
