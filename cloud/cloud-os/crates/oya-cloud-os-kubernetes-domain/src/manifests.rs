//! Individual cluster manifests (CNI, `CoreDNS`, kube-proxy, bootstrap RBAC, ...).
//!
//! Mirrors Talos `k8s.Manifest` resources produced by the
//! `ControlPlaneManifestController`: each is a named blob of one or more
//! Kubernetes objects, applied to the apiserver after bootstrap. We model the
//! manifest identity, kind, and ordering priority rather than the full YAML.

use crate::error::{K8sError, Result};

/// The category of a cluster manifest, which also drives apply ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ManifestKind {
    /// RBAC objects that must exist before anything else (priority 0).
    BootstrapRbac,
    /// The pod-security / default policies (priority 1).
    Policy,
    /// The CNI plugin (priority 2).
    Cni,
    /// kube-proxy (priority 3).
    KubeProxy,
    /// `CoreDNS` and its service (priority 4).
    CoreDns,
    /// Any additional user-supplied manifest (priority 5).
    Extra,
}

impl ManifestKind {
    /// The apply-order priority; lower applies first.
    pub fn priority(self) -> u8 {
        match self {
            ManifestKind::BootstrapRbac => 0,
            ManifestKind::Policy => 1,
            ManifestKind::Cni => 2,
            ManifestKind::KubeProxy => 3,
            ManifestKind::CoreDns => 4,
            ManifestKind::Extra => 5,
        }
    }
}

/// A single cluster manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// Stable manifest name, e.g. `"11-kube-config-in-cluster"`.
    pub name: String,
    /// The manifest category.
    pub kind: ManifestKind,
    /// The rendered object body (opaque YAML).
    pub body: String,
}

impl Manifest {
    /// Construct a manifest, validating name and body are non-empty.
    pub fn new(
        name: impl Into<String>,
        kind: ManifestKind,
        body: impl Into<String>,
    ) -> Result<Self> {
        let name = name.into();
        let body = body.into();
        if name.trim().is_empty() {
            return Err(K8sError::Render("manifest name empty".to_string()));
        }
        if body.trim().is_empty() {
            return Err(K8sError::Render(format!("manifest {name} has empty body")));
        }
        Ok(Manifest { name, kind, body })
    }

    /// The ordering key used to sort manifests for apply: `(priority, name)`.
    pub fn order_key(&self) -> (u8, &str) {
        (self.kind.priority(), self.name.as_str())
    }

    /// A coarse hash of the body, used to detect drift between runs.
    pub fn content_hash(&self) -> u64 {
        // FNV-1a over the body bytes; deterministic and dependency-free.
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for b in self.body.as_bytes() {
            hash ^= u64::from(*b);
            hash = hash.wrapping_mul(0x100_0000_01b3);
        }
        hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_orders_kinds() {
        assert!(ManifestKind::BootstrapRbac.priority() < ManifestKind::Cni.priority());
        assert!(ManifestKind::CoreDns.priority() < ManifestKind::Extra.priority());
    }

    #[test]
    fn rejects_empty_name_or_body() {
        assert!(Manifest::new("", ManifestKind::Cni, "x").is_err());
        assert!(Manifest::new("n", ManifestKind::Cni, "   ").is_err());
    }

    #[test]
    fn order_key_combines_priority_and_name() {
        let m = Manifest::new("flannel", ManifestKind::Cni, "kind: DaemonSet").unwrap();
        assert_eq!(m.order_key(), (2, "flannel"));
    }

    #[test]
    fn content_hash_is_stable_and_sensitive() {
        let a = Manifest::new("a", ManifestKind::Extra, "body-1").unwrap();
        let b = Manifest::new("a", ManifestKind::Extra, "body-1").unwrap();
        let c = Manifest::new("a", ManifestKind::Extra, "body-2").unwrap();
        assert_eq!(a.content_hash(), b.content_hash());
        assert_ne!(a.content_hash(), c.content_hash());
    }
}
