//! COSI resource base types: namespaces, kinds, pointers and metadata.
//!
//! Talos models all OS state as COSI resources, each addressed by a
//! `(namespace, type, id)` triple and carrying metadata: a monotonically
//! increasing version, a lifecycle phase, an owner, labels, and a finalizer
//! set. This module provides those base types so every other crate can build
//! typed resources on top of a shared metadata core.

use crate::address::ResourceId;
use crate::error::{Error, Result};
use crate::id::Fingerprint;
use crate::primitives::Labels;
use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

/// A resource namespace (e.g. `default`, `runtime`, `k8s`, `network`,
/// `config`). Validated to a DNS-label-ish character set.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Namespace(String);

impl Namespace {
    /// Validate and construct a namespace. Must be non-empty, lowercase
    /// `[a-z0-9-]`, not starting/ending with `-`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(Error::invalid("namespace is empty"));
        }
        if s.len() > 63 {
            return Err(Error::invalid("namespace exceeds 63 characters"));
        }
        if s.starts_with('-') || s.ends_with('-') {
            return Err(Error::invalid("namespace may not start or end with '-'"));
        }
        for c in s.chars() {
            if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
                return Err(Error::invalid(alloc::format!(
                    "invalid namespace character '{c}'"
                )));
            }
        }
        Ok(Namespace(s))
    }

    /// The string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Well-known Talos namespaces, provided as constructors for convenience.
impl Namespace {
    /// The `default` namespace.
    pub fn default_ns() -> Self {
        Namespace("default".to_string())
    }
    /// The `runtime` namespace (machine runtime state).
    pub fn runtime() -> Self {
        Namespace("runtime".to_string())
    }
    /// The `config` namespace (machine config provider).
    pub fn config() -> Self {
        Namespace("config".to_string())
    }
    /// The `network` namespace.
    pub fn network() -> Self {
        Namespace("network".to_string())
    }
    /// The `k8s` namespace (Kubernetes control-plane state).
    pub fn k8s() -> Self {
        Namespace("k8s".to_string())
    }
}

/// A resource kind/type string (e.g. `MachineConfigs.config.talos.dev`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceKind(String);

impl ResourceKind {
    /// Validate and construct a resource kind. Must be non-empty and contain
    /// only `[A-Za-z0-9._-]`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(Error::invalid("resource kind is empty"));
        }
        for c in s.chars() {
            if !(c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')) {
                return Err(Error::invalid(alloc::format!(
                    "invalid resource kind character '{c}'"
                )));
            }
        }
        Ok(ResourceKind(s))
    }

    /// The string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ResourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A fully-qualified pointer to a single resource: `namespace/kind/id`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourcePointer {
    /// The owning namespace.
    pub namespace: Namespace,
    /// The resource kind/type.
    pub kind: ResourceKind,
    /// The stable id within `(namespace, kind)`.
    pub id: ResourceId,
}

impl ResourcePointer {
    /// Construct a pointer.
    pub fn new(namespace: Namespace, kind: ResourceKind, id: ResourceId) -> Self {
        ResourcePointer {
            namespace,
            kind,
            id,
        }
    }

    /// The canonical `namespace/kind/id` string.
    pub fn to_canonical(&self) -> String {
        alloc::format!("{}/{}/{}", self.namespace, self.kind, self.id)
    }
}

impl fmt::Display for ResourcePointer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_canonical())
    }
}

/// Lifecycle phase of a resource, mirroring COSI `resource.Phase`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Live and usable.
    Running,
    /// Marked for deletion; held until finalizers are cleared (teardown).
    TearingDown,
}

impl Phase {
    /// The lowercase string form.
    pub fn as_str(self) -> &'static str {
        match self {
            Phase::Running => "running",
            Phase::TearingDown => "tearingdown",
        }
    }
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// COSI resource metadata.
///
/// Carries the addressing pointer plus the mutable bookkeeping COSI maintains:
/// a version counter, phase, optional owner (the controller that created it),
/// labels, and an ordered set of finalizers that block teardown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    pointer: ResourcePointer,
    version: u64,
    phase: Phase,
    owner: Option<String>,
    labels: Labels,
    finalizers: BTreeSet<String>,
}

impl Metadata {
    /// New metadata at version 1, `Running` phase, no owner/finalizers.
    pub fn new(namespace: Namespace, kind: ResourceKind, id: ResourceId) -> Self {
        Metadata {
            pointer: ResourcePointer::new(namespace, kind, id),
            version: 1,
            phase: Phase::Running,
            owner: None,
            labels: Labels::new(),
            finalizers: BTreeSet::new(),
        }
    }

    /// The resource pointer.
    pub fn pointer(&self) -> &ResourcePointer {
        &self.pointer
    }

    /// The current version counter.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Bump the version, returning the new value. Controllers call this on
    /// every spec mutation so watchers can detect change.
    pub fn bump_version(&mut self) -> u64 {
        self.version += 1;
        self.version
    }

    /// The current phase.
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// The owner string, if set.
    pub fn owner(&self) -> Option<&str> {
        self.owner.as_deref()
    }

    /// Set the owner. Errors if already owned by a different controller, which
    /// mirrors COSI's single-owner invariant.
    pub fn set_owner(&mut self, owner: impl Into<String>) -> Result<()> {
        let owner = owner.into();
        match &self.owner {
            Some(existing) if existing != &owner => Err(Error::invalid_state(alloc::format!(
                "resource already owned by '{existing}'"
            ))),
            _ => {
                self.owner = Some(owner);
                Ok(())
            }
        }
    }

    /// Mutable access to labels.
    pub fn labels_mut(&mut self) -> &mut Labels {
        &mut self.labels
    }

    /// Read access to labels.
    pub fn labels(&self) -> &Labels {
        &self.labels
    }

    /// The finalizers blocking teardown, in sorted order.
    pub fn finalizers(&self) -> Vec<&str> {
        self.finalizers.iter().map(String::as_str).collect()
    }

    /// Add a finalizer; returns `true` if it was newly added.
    pub fn add_finalizer(&mut self, name: impl Into<String>) -> bool {
        self.finalizers.insert(name.into())
    }

    /// Remove a finalizer; returns `true` if it was present.
    pub fn remove_finalizer(&mut self, name: &str) -> bool {
        self.finalizers.remove(name)
    }

    /// Whether any finalizer is still registered.
    pub fn has_finalizers(&self) -> bool {
        !self.finalizers.is_empty()
    }

    /// Transition into the `TearingDown` phase. Idempotent.
    pub fn mark_tearing_down(&mut self) {
        self.phase = Phase::TearingDown;
    }

    /// Whether the resource is ready to be destroyed: it is tearing down and no
    /// finalizers remain. Mirrors COSI's destroy precondition.
    pub fn ready_to_destroy(&self) -> bool {
        self.phase == Phase::TearingDown && self.finalizers.is_empty()
    }

    /// A content fingerprint over the addressing + version, useful as an
    /// optimistic-concurrency token.
    pub fn fingerprint(&self) -> Fingerprint {
        let s = alloc::format!("{}@{}", self.pointer.to_canonical(), self.version);
        Fingerprint::of_str(&s)
    }
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{} ({})", self.pointer, self.version, self.phase)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta() -> Metadata {
        Metadata::new(
            Namespace::config(),
            ResourceKind::new("MachineConfigs.config.talos.dev").unwrap(),
            ResourceId::new("v1alpha1").unwrap(),
        )
    }

    #[test]
    fn namespace_validation() {
        assert_eq!(Namespace::new("runtime").unwrap().as_str(), "runtime");
        assert!(Namespace::new("").is_err());
        assert!(Namespace::new("-bad").is_err());
        assert!(Namespace::new("Bad").is_err());
        assert!(Namespace::new("under_score").is_err());
        assert_eq!(Namespace::k8s().as_str(), "k8s");
        assert_eq!(Namespace::default_ns().as_str(), "default");
    }

    #[test]
    fn resource_kind_validation() {
        assert!(ResourceKind::new("MachineConfigs.config.talos.dev").is_ok());
        assert!(ResourceKind::new("").is_err());
        assert!(ResourceKind::new("bad kind").is_err());
    }

    #[test]
    fn pointer_canonical_form() {
        let p = ResourcePointer::new(
            Namespace::runtime(),
            ResourceKind::new("Hostnames").unwrap(),
            ResourceId::new("hostname").unwrap(),
        );
        assert_eq!(p.to_canonical(), "runtime/Hostnames/hostname");
        assert_eq!(p.to_string(), "runtime/Hostnames/hostname");
    }

    #[test]
    fn metadata_version_bump() {
        let mut m = meta();
        assert_eq!(m.version(), 1);
        assert_eq!(m.bump_version(), 2);
        assert_eq!(m.version(), 2);
    }

    #[test]
    fn metadata_owner_single_owner_invariant() {
        let mut m = meta();
        assert!(m.owner().is_none());
        m.set_owner("ConfigController").unwrap();
        assert_eq!(m.owner(), Some("ConfigController"));
        // Re-setting same owner is fine.
        m.set_owner("ConfigController").unwrap();
        // Different owner is rejected.
        assert!(m.set_owner("Other").is_err());
    }

    #[test]
    fn metadata_finalizers_and_teardown() {
        let mut m = meta();
        assert!(!m.has_finalizers());
        assert!(m.add_finalizer("k8s-controller"));
        assert!(!m.add_finalizer("k8s-controller")); // already present
        assert!(m.add_finalizer("network-controller"));
        assert_eq!(
            m.finalizers(),
            alloc::vec!["k8s-controller", "network-controller"]
        );

        // Not destroyable while finalizers remain even after teardown.
        m.mark_tearing_down();
        assert_eq!(m.phase(), Phase::TearingDown);
        assert!(!m.ready_to_destroy());

        assert!(m.remove_finalizer("k8s-controller"));
        assert!(!m.ready_to_destroy());
        assert!(m.remove_finalizer("network-controller"));
        assert!(m.ready_to_destroy());
        assert!(!m.remove_finalizer("absent"));
    }

    #[test]
    fn metadata_labels_and_fingerprint() {
        let mut m = meta();
        m.labels_mut().insert("cluster", "prod").unwrap();
        assert_eq!(m.labels().get("cluster"), Some("prod"));

        let fp1 = m.fingerprint();
        m.bump_version();
        let fp2 = m.fingerprint();
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn metadata_display() {
        let m = meta();
        assert_eq!(
            m.to_string(),
            "config/MachineConfigs.config.talos.dev/v1alpha1@1 (running)"
        );
    }
}
