//! The [`Resource`] trait and the [`ResourceKind`] descriptor.
//!
//! A COSI resource is a typed object carrying [`Metadata`] plus an opaque spec.
//! Concrete resources implement [`Resource`]; the runtime stores them as
//! [`AnyResource`] trait objects so heterogeneous kinds share one store.

use crate::metadata::Metadata;
use core::fmt;

/// Identifies a resource type by `namespace` + `kind`.
///
/// Controllers declare the kinds they manage and depend on using this
/// descriptor; the runtime uses it to route events.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceKind {
    namespace: String,
    kind: String,
}

impl ResourceKind {
    /// Construct a kind descriptor.
    pub fn new(namespace: impl Into<String>, kind: impl Into<String>) -> Self {
        ResourceKind {
            namespace: namespace.into(),
            kind: kind.into(),
        }
    }

    /// The namespace.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// The kind/type name.
    pub fn kind(&self) -> &str {
        &self.kind
    }
}

impl fmt::Display for ResourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.namespace, self.kind)
    }
}

/// A typed COSI resource.
///
/// Implementors own their [`Metadata`] and a strongly-typed spec. The
/// [`spec_fingerprint`] is used by the store to detect no-op updates and skip
/// version bumps (mirroring COSI's value equality semantics).
pub trait Resource: fmt::Debug {
    /// Borrow the metadata.
    fn metadata(&self) -> &Metadata;

    /// Mutably borrow the metadata.
    fn metadata_mut(&mut self) -> &mut Metadata;

    /// The static resource kind this instance belongs to.
    fn resource_kind(&self) -> ResourceKind {
        ResourceKind::new(
            self.metadata().namespace().to_string(),
            self.metadata().kind().to_string(),
        )
    }

    /// A stable string fingerprint of the spec used for value-equality checks.
    /// Two resources with equal metadata identity and equal fingerprints are
    /// considered unchanged.
    fn spec_fingerprint(&self) -> String;

    /// Deep-clone into a boxed trait object so the store can keep snapshots.
    fn clone_box(&self) -> Box<dyn Resource>;
}

impl Clone for Box<dyn Resource> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// A boxed, type-erased resource as held by the store.
pub type AnyResource = Box<dyn Resource>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::Metadata;
    use os_kernel::ResourceId;

    #[derive(Debug, Clone)]
    pub struct DummyConfig {
        meta: Metadata,
        pub replicas: u32,
    }

    impl DummyConfig {
        pub fn new(id: &str, replicas: u32) -> Self {
            DummyConfig {
                meta: Metadata::new("default", "DummyConfig", ResourceId::new(id).unwrap()),
                replicas,
            }
        }
    }

    impl Resource for DummyConfig {
        fn metadata(&self) -> &Metadata {
            &self.meta
        }
        fn metadata_mut(&mut self) -> &mut Metadata {
            &mut self.meta
        }
        fn spec_fingerprint(&self) -> String {
            format!("replicas={}", self.replicas)
        }
        fn clone_box(&self) -> Box<dyn Resource> {
            Box::new(self.clone())
        }
    }

    #[test]
    fn resource_kind_from_metadata() {
        let r = DummyConfig::new("a", 3);
        assert_eq!(
            r.resource_kind(),
            ResourceKind::new("default", "DummyConfig")
        );
        assert_eq!(r.resource_kind().to_string(), "default/DummyConfig");
    }

    #[test]
    fn fingerprint_tracks_spec() {
        let a = DummyConfig::new("a", 3);
        let b = DummyConfig::new("a", 3);
        let c = DummyConfig::new("a", 5);
        assert_eq!(a.spec_fingerprint(), b.spec_fingerprint());
        assert_ne!(a.spec_fingerprint(), c.spec_fingerprint());
    }

    #[test]
    fn boxed_resource_clones() {
        let r = DummyConfig::new("a", 3);
        let boxed: AnyResource = Box::new(r);
        let cloned = boxed.clone();
        assert_eq!(boxed.metadata().key(), cloned.metadata().key());
    }
}
