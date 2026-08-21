//! Resource metadata: namespace/type/id triple, version, phase, owner,
//! finalizers and labels. Mirrors `cosi-project/runtime` `resource.Metadata`.

use core::fmt;
use os_kernel::ResourceId;
use std::collections::BTreeMap;

/// Lifecycle phase of a resource.
///
/// A resource is created in [`Phase::Running`]. When a destroy is requested
/// while finalizers are still attached, it transitions to
/// [`Phase::TearingDown`] until the last finalizer is removed, at which point
/// it may actually be destroyed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Phase {
    /// The resource is live and reconciled normally.
    #[default]
    Running,
    /// A teardown was requested; controllers must clean up and remove their
    /// finalizers before the resource is destroyed.
    TearingDown,
}

impl Phase {
    /// Stable lowercase string for the phase.
    pub fn as_str(&self) -> &'static str {
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

/// Set of finalizer names attached by controllers. While non-empty during
/// [`Phase::TearingDown`], the resource cannot be destroyed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Finalizers(Vec<String>);

impl Finalizers {
    /// An empty finalizer set.
    pub fn new() -> Self {
        Finalizers(Vec::new())
    }

    /// Add a finalizer. Returns `true` if it was newly added (idempotent).
    pub fn add(&mut self, name: impl Into<String>) -> bool {
        let name = name.into();
        if self.0.iter().any(|f| f == &name) {
            return false;
        }
        self.0.push(name);
        true
    }

    /// Remove a finalizer. Returns `true` if it was present.
    pub fn remove(&mut self, name: &str) -> bool {
        if let Some(idx) = self.0.iter().position(|f| f == name) {
            self.0.remove(idx);
            true
        } else {
            false
        }
    }

    /// Whether a given finalizer is attached.
    pub fn contains(&self, name: &str) -> bool {
        self.0.iter().any(|f| f == name)
    }

    /// Whether no finalizers remain.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Number of attached finalizers.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Iterate over finalizer names.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(String::as_str)
    }
}

/// Key/value labels used to select and filter resources.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Labels(BTreeMap<String, String>);

impl Labels {
    /// An empty label set.
    pub fn new() -> Self {
        Labels(BTreeMap::new())
    }

    /// Set a label, returning the previous value if present.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) -> Option<String> {
        self.0.insert(key.into(), value.into())
    }

    /// Get a label value.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }

    /// Whether a label key exists.
    pub fn has(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    /// Remove a label.
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.0.remove(key)
    }

    /// Whether this label set matches an equality selector (all key=value
    /// pairs in `selector` are present and equal).
    pub fn matches(&self, selector: &Labels) -> bool {
        selector.0.iter().all(|(k, v)| self.0.get(k) == Some(v))
    }

    /// Number of labels.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether there are no labels.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// The metadata block carried by every resource.
///
/// `(namespace, kind, id)` uniquely addresses a resource. `version` is a
/// monotonically increasing revision that the store bumps on every update and
/// that controllers use for optimistic concurrency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    namespace: String,
    kind: String,
    id: ResourceId,
    version: u64,
    phase: Phase,
    owner: String,
    finalizers: Finalizers,
    labels: Labels,
}

impl Metadata {
    /// Construct fresh metadata at version 0, phase Running, no owner.
    pub fn new(namespace: impl Into<String>, kind: impl Into<String>, id: ResourceId) -> Self {
        Metadata {
            namespace: namespace.into(),
            kind: kind.into(),
            id,
            version: 0,
            phase: Phase::Running,
            owner: String::new(),
            finalizers: Finalizers::new(),
            labels: Labels::new(),
        }
    }

    /// The resource namespace.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// The resource kind (type name).
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// The resource id.
    pub fn id(&self) -> &ResourceId {
        &self.id
    }

    /// The current revision.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Increment the revision (called by the store on mutation).
    pub fn bump_version(&mut self) {
        self.version += 1;
    }

    /// Set the revision directly. Used by the store to align a written
    /// resource's version with the authoritative store version (callers may
    /// submit a resource constructed independently of the stored revision).
    pub fn set_version(&mut self, version: u64) {
        self.version = version;
    }

    /// The lifecycle phase.
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// Transition into teardown. Returns an error if already tearing down.
    pub fn set_phase(&mut self, phase: Phase) {
        self.phase = phase;
    }

    /// The owning controller name (empty for user-managed resources).
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// Set the owner. In COSI an owner can only be set once (when created by a
    /// controller) — callers should enforce that at a higher level.
    pub fn set_owner(&mut self, owner: impl Into<String>) {
        self.owner = owner.into();
    }

    /// Read-only access to finalizers.
    pub fn finalizers(&self) -> &Finalizers {
        &self.finalizers
    }

    /// Mutable access to finalizers.
    pub fn finalizers_mut(&mut self) -> &mut Finalizers {
        &mut self.finalizers
    }

    /// Read-only access to labels.
    pub fn labels(&self) -> &Labels {
        &self.labels
    }

    /// Mutable access to labels.
    pub fn labels_mut(&mut self) -> &mut Labels {
        &mut self.labels
    }

    /// Whether this resource is allowed to be destroyed: it must be tearing
    /// down with no finalizers left.
    pub fn can_destroy(&self) -> bool {
        self.phase == Phase::TearingDown && self.finalizers.is_empty()
    }

    /// Canonical `namespace/kind/id` string used as a map key.
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

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}@{} ({})",
            self.namespace, self.kind, self.id, self.version, self.phase
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta() -> Metadata {
        Metadata::new("default", "TestResource", ResourceId::new("a").unwrap())
    }

    #[test]
    fn finalizers_are_idempotent() {
        let mut f = Finalizers::new();
        assert!(f.add("ctrl-a"));
        assert!(!f.add("ctrl-a"));
        assert_eq!(f.len(), 1);
        assert!(f.contains("ctrl-a"));
        assert!(f.remove("ctrl-a"));
        assert!(!f.remove("ctrl-a"));
        assert!(f.is_empty());
    }

    #[test]
    fn labels_match_selector() {
        let mut l = Labels::new();
        l.set("app", "etcd");
        l.set("tier", "control-plane");
        let mut sel = Labels::new();
        sel.set("app", "etcd");
        assert!(l.matches(&sel));
        sel.set("tier", "worker");
        assert!(!l.matches(&sel));
    }

    #[test]
    fn destroy_requires_teardown_and_no_finalizers() {
        let mut m = meta();
        assert!(!m.can_destroy());
        m.finalizers_mut().add("ctrl");
        m.set_phase(Phase::TearingDown);
        assert!(!m.can_destroy());
        m.finalizers_mut().remove("ctrl");
        assert!(m.can_destroy());
    }

    #[test]
    fn version_bumps_and_key_is_canonical() {
        let mut m = meta();
        assert_eq!(m.version(), 0);
        m.bump_version();
        assert_eq!(m.version(), 1);
        assert_eq!(m.key(), "default/TestResource/a");
    }
}
