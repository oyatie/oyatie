//! Reduced (lightweight) resource representation used in watch events and the
//! dependency runtime. Mirrors COSI's `resource.Reduced` / metadata-only view.
//!
//! A reduced resource carries only the identity, version and phase — enough for
//! controllers to decide whether to re-read the full object from the store.

use crate::metadata::{Metadata, Phase};
use core::fmt;
use os_kernel::ResourceId;

/// A metadata-only snapshot of a resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReducedResource {
    namespace: String,
    kind: String,
    id: ResourceId,
    version: u64,
    phase: Phase,
}

impl ReducedResource {
    /// Build a reduced view from full metadata.
    pub fn from_metadata(md: &Metadata) -> Self {
        ReducedResource {
            namespace: md.namespace().to_string(),
            kind: md.kind().to_string(),
            id: md.id().clone(),
            version: md.version(),
            phase: md.phase(),
        }
    }

    /// The namespace.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// The kind.
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// The id.
    pub fn id(&self) -> &ResourceId {
        &self.id
    }

    /// The revision.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// The phase.
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// Whether this reduced resource is newer than another snapshot of the
    /// same identity.
    pub fn is_newer_than(&self, other: &ReducedResource) -> bool {
        self.same_identity(other) && self.version > other.version
    }

    /// Whether two reduced resources address the same object.
    pub fn same_identity(&self, other: &ReducedResource) -> bool {
        self.namespace == other.namespace && self.kind == other.kind && self.id == other.id
    }

    /// Canonical key.
    pub fn key(&self) -> String {
        let mut s = String::new();
        s.push_str(&self.namespace);
        s.push('/');
        s.push_str(&self.kind);
        s.push('/');
        s.push_str(self.id.as_str());
        s
    }
}

impl fmt::Display for ReducedResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{} ({})", self.key(), self.version, self.phase)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn md(ver_bumps: u64) -> Metadata {
        let mut m = Metadata::new("default", "K", ResourceId::new("x").unwrap());
        for _ in 0..ver_bumps {
            m.bump_version();
        }
        m
    }

    #[test]
    fn reduced_tracks_version_and_phase() {
        let mut m = md(0);
        let r0 = ReducedResource::from_metadata(&m);
        m.bump_version();
        m.set_phase(Phase::TearingDown);
        let r1 = ReducedResource::from_metadata(&m);
        assert!(r1.is_newer_than(&r0));
        assert_eq!(r1.phase(), Phase::TearingDown);
    }

    #[test]
    fn identity_compares_namespace_kind_id() {
        let a = ReducedResource::from_metadata(&md(0));
        let b = ReducedResource::from_metadata(&md(2));
        assert!(a.same_identity(&b));
        assert!(!a.is_newer_than(&b));
        assert!(b.is_newer_than(&a));
    }

    #[test]
    fn key_is_canonical() {
        let r = ReducedResource::from_metadata(&md(0));
        assert_eq!(r.key(), "default/K/x");
    }
}
