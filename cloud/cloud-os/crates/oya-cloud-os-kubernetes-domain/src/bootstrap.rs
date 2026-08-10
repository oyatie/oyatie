//! The ordered collection of bootstrap manifests applied exactly once.
//!
//! Mirrors Talos `k8s.BootstrapManifestController`: when the cluster is first
//! brought up, machined applies a fixed, ordered set of manifests to the
//! apiserver. We model the collection as a deduplicated, priority-ordered set
//! that records whether it has been applied, so bootstrap is idempotent.

use crate::error::{K8sError, Result};
use crate::manifests::Manifest;
use std::collections::BTreeSet;

/// The ordered set of bootstrap manifests.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BootstrapManifests {
    manifests: Vec<Manifest>,
    names: BTreeSet<String>,
    applied: bool,
}

impl BootstrapManifests {
    /// An empty, unapplied bootstrap set.
    pub fn new() -> Self {
        BootstrapManifests {
            manifests: Vec::new(),
            names: BTreeSet::new(),
            applied: false,
        }
    }

    /// Add a manifest. Errors on a duplicate name or if already applied.
    pub fn push(&mut self, manifest: Manifest) -> Result<()> {
        if self.applied {
            return Err(K8sError::Bootstrap(
                "cannot modify bootstrap manifests after apply".to_string(),
            ));
        }
        if !self.names.insert(manifest.name.clone()) {
            return Err(K8sError::Bootstrap(format!(
                "duplicate bootstrap manifest: {}",
                manifest.name
            )));
        }
        self.manifests.push(manifest);
        Ok(())
    }

    /// The manifests in apply order (priority, then name).
    pub fn ordered(&self) -> Vec<&Manifest> {
        let mut refs: Vec<&Manifest> = self.manifests.iter().collect();
        refs.sort_by(|a, b| a.order_key().cmp(&b.order_key()));
        refs
    }

    /// Number of manifests.
    pub fn len(&self) -> usize {
        self.manifests.len()
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.manifests.is_empty()
    }

    /// Whether the set has been applied.
    pub fn is_applied(&self) -> bool {
        self.applied
    }

    /// Apply the bootstrap set exactly once, returning the ordered names that
    /// were applied. A second call errors with [`K8sError::Bootstrap`].
    pub fn apply(&mut self) -> Result<Vec<String>> {
        if self.applied {
            return Err(K8sError::Bootstrap(
                "bootstrap manifests already applied".to_string(),
            ));
        }
        if self.manifests.is_empty() {
            return Err(K8sError::Bootstrap(
                "refusing to apply empty bootstrap set".to_string(),
            ));
        }
        self.applied = true;
        Ok(self.ordered().into_iter().map(|m| m.name.clone()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifests::ManifestKind;

    fn m(name: &str, kind: ManifestKind) -> Manifest {
        Manifest::new(name, kind, format!("# {name}")).unwrap()
    }

    #[test]
    fn push_orders_by_priority() {
        let mut b = BootstrapManifests::new();
        b.push(m("coredns", ManifestKind::CoreDns)).unwrap();
        b.push(m("rbac", ManifestKind::BootstrapRbac)).unwrap();
        b.push(m("cni", ManifestKind::Cni)).unwrap();
        let names: Vec<&str> = b.ordered().iter().map(|x| x.name.as_str()).collect();
        assert_eq!(names, vec!["rbac", "cni", "coredns"]);
    }

    #[test]
    fn duplicate_name_rejected() {
        let mut b = BootstrapManifests::new();
        b.push(m("rbac", ManifestKind::BootstrapRbac)).unwrap();
        let err = b.push(m("rbac", ManifestKind::Policy)).unwrap_err();
        assert_eq!(err.kind(), "bootstrap");
    }

    #[test]
    fn apply_is_idempotent_once() {
        let mut b = BootstrapManifests::new();
        b.push(m("rbac", ManifestKind::BootstrapRbac)).unwrap();
        b.push(m("cni", ManifestKind::Cni)).unwrap();
        let applied = b.apply().unwrap();
        assert_eq!(applied, vec!["rbac".to_string(), "cni".to_string()]);
        assert!(b.is_applied());
        // Second apply errors.
        assert!(b.apply().is_err());
        // Cannot push after apply.
        assert!(b.push(m("x", ManifestKind::Extra)).is_err());
    }

    #[test]
    fn empty_apply_rejected() {
        let mut b = BootstrapManifests::new();
        assert!(b.apply().is_err());
        assert!(b.is_empty());
    }
}
