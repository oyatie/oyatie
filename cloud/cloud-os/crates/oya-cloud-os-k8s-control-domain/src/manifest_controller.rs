//! Bootstrap and extra-manifest controllers.
//!
//! Mirrors Talos `BootstrapManifestController` and `ExtraManifestController`
//! under `internal/app/machined/pkg/controllers/k8s`. The bootstrap controller
//! renders the cluster-default manifests (kube-proxy, `CoreDNS`, bootstrap RBAC,
//! pod-security policies) applied exactly once at cluster bootstrap; the extra
//! manifest controller pulls user-supplied manifests (inline or from a URL) and
//! applies them after the bootstrap set.
//!
//! Application to the API server is modeled as the [`ManifestApplier`] trait
//! with an in-memory implementation used by tests.

use crate::error::{ControlError, Result};

/// Origin of a cluster manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestSource {
    /// A built-in bootstrap manifest Talos always renders.
    Bootstrap,
    /// An operator-supplied inline manifest (`cluster.inlineManifests`).
    Inline,
    /// An operator-supplied remote manifest (`cluster.extraManifests`).
    Remote,
}

impl ManifestSource {
    /// Apply priority: bootstrap manifests apply before extra manifests.
    #[must_use]
    pub fn priority(self) -> u8 {
        match self {
            ManifestSource::Bootstrap => 0,
            ManifestSource::Inline => 1,
            ManifestSource::Remote => 2,
        }
    }
}

/// A single cluster manifest: a named YAML document plus its source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterManifest {
    /// Stable, unique name (used for ordering and dedup).
    pub name: String,
    /// Where the manifest came from.
    pub source: ManifestSource,
    /// The manifest YAML body.
    pub contents: String,
}

impl ClusterManifest {
    /// Build a manifest, validating name and contents are non-empty.
    pub fn new(
        name: impl Into<String>,
        source: ManifestSource,
        contents: impl Into<String>,
    ) -> Result<Self> {
        let name = name.into();
        let contents = contents.into();
        if name.trim().is_empty() {
            return Err(ControlError::Manifest("manifest name is empty".into()));
        }
        if contents.trim().is_empty() {
            return Err(ControlError::Manifest(format!(
                "manifest {name} has empty contents"
            )));
        }
        Ok(ClusterManifest {
            name,
            source,
            contents,
        })
    }
}

/// The boundary the controller applies manifests through. The real
/// implementation talks to the API server; tests use [`InMemoryApplier`].
pub trait ManifestApplier {
    /// Apply (server-side) a manifest. Idempotent on name.
    fn apply(&mut self, manifest: &ClusterManifest) -> Result<()>;
}

/// An in-memory [`ManifestApplier`] recording applied manifests in order.
#[derive(Debug, Default)]
pub struct InMemoryApplier {
    applied: Vec<String>,
}

impl InMemoryApplier {
    /// A fresh applier.
    #[must_use]
    pub fn new() -> Self {
        InMemoryApplier {
            applied: Vec::new(),
        }
    }

    /// Names of applied manifests in application order.
    #[must_use]
    pub fn applied(&self) -> &[String] {
        &self.applied
    }

    /// Whether a manifest with `name` was applied.
    #[must_use]
    pub fn has(&self, name: &str) -> bool {
        self.applied.iter().any(|n| n == name)
    }
}

impl ManifestApplier for InMemoryApplier {
    fn apply(&mut self, manifest: &ClusterManifest) -> Result<()> {
        if !self.applied.iter().any(|n| n == &manifest.name) {
            self.applied.push(manifest.name.clone());
        }
        Ok(())
    }
}

/// Default bootstrap manifest names Talos renders, in apply order.
pub const BOOTSTRAP_MANIFEST_NAMES: &[&str] = &[
    "00-kubelet-bootstrapping-token",
    "01-csr-approver-role-binding",
    "01-csr-node-bootstrap",
    "01-csr-renewal-role-binding",
    "11-core-dns",
    "11-core-dns-svc",
    "11-kube-config-in-cluster",
    "kube-proxy",
    "pod-security-policy",
];

/// The manifest controller: collects bootstrap and extra manifests, validates
/// uniqueness, orders them by source priority then declaration order, and
/// applies them through a [`ManifestApplier`].
///
/// Mirrors the combination of `BootstrapManifestController` (which produces the
/// default set) and `ExtraManifestController` (which appends user manifests).
#[derive(Debug, Default)]
pub struct ManifestController {
    manifests: Vec<ClusterManifest>,
}

impl ManifestController {
    /// A fresh, empty controller.
    #[must_use]
    pub fn new() -> Self {
        ManifestController {
            manifests: Vec::new(),
        }
    }

    /// Add a manifest, rejecting a duplicate name.
    pub fn add(&mut self, manifest: ClusterManifest) -> Result<()> {
        if self.manifests.iter().any(|m| m.name == manifest.name) {
            return Err(ControlError::Manifest(format!(
                "duplicate manifest name: {}",
                manifest.name
            )));
        }
        self.manifests.push(manifest);
        Ok(())
    }

    /// Number of registered manifests.
    #[must_use]
    pub fn len(&self) -> usize {
        self.manifests.len()
    }

    /// Whether no manifests are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.manifests.is_empty()
    }

    /// The manifests in apply order (stable sort by source priority, preserving
    /// declaration order within a source).
    #[must_use]
    pub fn ordered(&self) -> Vec<&ClusterManifest> {
        let mut v: Vec<&ClusterManifest> = self.manifests.iter().collect();
        v.sort_by_key(|m| m.source.priority());
        v
    }

    /// Reconcile: apply every manifest in order through the applier. Returns the
    /// number of manifests applied.
    pub fn reconcile(&self, applier: &mut dyn ManifestApplier) -> Result<usize> {
        let ordered = self.ordered();
        for m in &ordered {
            applier.apply(m)?;
        }
        Ok(ordered.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(name: &str, source: ManifestSource) -> ClusterManifest {
        ClusterManifest::new(name, source, "apiVersion: v1\nkind: ConfigMap\n").unwrap()
    }

    #[test]
    fn manifest_validates_inputs() {
        assert!(ClusterManifest::new("", ManifestSource::Inline, "x").is_err());
        assert!(ClusterManifest::new("n", ManifestSource::Inline, "  ").is_err());
        assert!(ClusterManifest::new("n", ManifestSource::Inline, "x").is_ok());
    }

    #[test]
    fn source_priority_orders_bootstrap_first() {
        assert!(ManifestSource::Bootstrap.priority() < ManifestSource::Inline.priority());
        assert!(ManifestSource::Inline.priority() < ManifestSource::Remote.priority());
    }

    #[test]
    fn controller_rejects_duplicate_names() {
        let mut c = ManifestController::new();
        c.add(m("kube-proxy", ManifestSource::Bootstrap)).unwrap();
        let dup = c.add(m("kube-proxy", ManifestSource::Bootstrap));
        assert_eq!(dup.unwrap_err().kind(), "manifest");
    }

    #[test]
    fn ordered_puts_bootstrap_before_extra() {
        let mut c = ManifestController::new();
        c.add(m("user-app", ManifestSource::Remote)).unwrap();
        c.add(m("kube-proxy", ManifestSource::Bootstrap)).unwrap();
        c.add(m("inline-cfg", ManifestSource::Inline)).unwrap();
        let names: Vec<&str> = c.ordered().iter().map(|m| m.name.as_str()).collect();
        assert_eq!(names, vec!["kube-proxy", "inline-cfg", "user-app"]);
    }

    #[test]
    fn reconcile_applies_in_order_and_is_idempotent() {
        let mut c = ManifestController::new();
        c.add(m("user-app", ManifestSource::Remote)).unwrap();
        c.add(m("kube-proxy", ManifestSource::Bootstrap)).unwrap();
        let mut applier = InMemoryApplier::new();
        let n = c.reconcile(&mut applier).unwrap();
        assert_eq!(n, 2);
        assert_eq!(
            applier.applied(),
            &["kube-proxy".to_string(), "user-app".to_string()]
        );
        // Re-applying does not duplicate.
        c.reconcile(&mut applier).unwrap();
        assert_eq!(applier.applied().len(), 2);
        assert!(applier.has("kube-proxy"));
    }

    #[test]
    fn bootstrap_names_present() {
        assert!(BOOTSTRAP_MANIFEST_NAMES.contains(&"kube-proxy"));
        assert!(BOOTSTRAP_MANIFEST_NAMES.contains(&"11-core-dns"));
        assert!(BOOTSTRAP_MANIFEST_NAMES.len() >= 8);
    }
}
