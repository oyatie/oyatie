//! In-memory COSI state store. Mirrors `cosi-project/runtime` `state.State`:
//! CRUD with optimistic concurrency, create/update/teardown/destroy lifecycle
//! rules, label-selector listing and watch fan-out.
//!
//! In real COSI the state is backed by an in-memory or etcd-like store guarded
//! by a mutex and exposing async channels. Here we model the boundary as a
//! plain owned struct that controllers drive synchronously; watch streams are
//! delivered through [`WatchChannel`]s registered per kind.

use crate::metadata::Phase;
use crate::resource::{AnyResource, ResourceKind};
use crate::watch::{Event, WatchChannel};
use core::fmt;
use std::collections::BTreeMap;

/// Errors returned by store operations, mirroring COSI's typed state errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    /// A resource with the same key already exists (on create).
    AlreadyExists(String),
    /// No resource exists for the given key (on update/get/destroy).
    NotFound(String),
    /// The supplied version did not match the stored version (optimistic
    /// concurrency conflict).
    Conflict {
        /// The resource key.
        key: String,
        /// Version the caller believed was current.
        expected: u64,
        /// Version actually stored.
        actual: u64,
    },
    /// A destroy was requested but the resource still has finalizers or is not
    /// tearing down.
    StillReferenced(String),
    /// An owner mismatch: a controller tried to mutate a resource owned by a
    /// different controller.
    OwnerConflict {
        /// The resource key.
        key: String,
        /// The actual owner.
        owner: String,
    },
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::AlreadyExists(k) => write!(f, "resource {k} already exists"),
            StoreError::NotFound(k) => write!(f, "resource {k} not found"),
            StoreError::Conflict {
                key,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "version conflict on {key}: expected {expected}, actual {actual}"
                )
            }
            StoreError::StillReferenced(k) => {
                write!(
                    f,
                    "resource {k} is still referenced (finalizers or not tearing down)"
                )
            }
            StoreError::OwnerConflict { key, owner } => {
                write!(f, "resource {key} is owned by {owner}")
            }
        }
    }
}

impl std::error::Error for StoreError {}

/// Result alias for store operations.
pub type StoreResult<T> = Result<T, StoreError>;

/// An in-memory COSI state store.
#[derive(Default)]
pub struct State {
    resources: BTreeMap<String, AnyResource>,
    watches: BTreeMap<ResourceKind, Vec<WatchChannel>>,
}

impl State {
    /// Create an empty store.
    pub fn new() -> Self {
        State {
            resources: BTreeMap::new(),
            watches: BTreeMap::new(),
        }
    }

    /// Number of stored resources.
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    /// Get a clone of a resource by key (`namespace/kind/id`).
    pub fn get(&self, key: &str) -> Option<AnyResource> {
        self.resources.get(key).map(|r| r.clone_box())
    }

    /// Whether a resource exists for the key.
    pub fn contains(&self, key: &str) -> bool {
        self.resources.contains_key(key)
    }

    /// List clones of all resources of a kind, optionally filtered by a label
    /// selector. Results are sorted by key.
    pub fn list(
        &self,
        kind: &ResourceKind,
        selector: Option<&crate::metadata::Labels>,
    ) -> Vec<AnyResource> {
        let mut out = Vec::new();
        for r in self.resources.values() {
            if r.metadata().namespace() == kind.namespace() && r.metadata().kind() == kind.kind() {
                if let Some(sel) = selector
                    && !r.metadata().labels().matches(sel)
                {
                    continue;
                }
                out.push(r.clone_box());
            }
        }
        out
    }

    /// Register a watch channel for a kind. The channel immediately receives a
    /// snapshot of existing resources (as `Created` events) followed by the
    /// `Bootstrapped` sentinel, then live events.
    pub fn watch_kind(&mut self, kind: ResourceKind, capacity: usize) -> usize {
        let mut ch = WatchChannel::new(capacity);
        let mut snapshot = self.list(&kind, None);
        snapshot.sort_by_key(|r| r.metadata().key());
        for r in snapshot {
            ch.push(Event::created(r));
        }
        ch.push(Event::bootstrapped());
        let entry = self.watches.entry(kind).or_default();
        entry.push(ch);
        entry.len() - 1
    }

    /// Borrow a registered watch channel by kind and index, for draining.
    pub fn watch_mut(&mut self, kind: &ResourceKind, index: usize) -> Option<&mut WatchChannel> {
        self.watches.get_mut(kind).and_then(|v| v.get_mut(index))
    }

    fn fanout(&mut self, kind: &ResourceKind, make: impl Fn() -> Event) {
        if let Some(channels) = self.watches.get_mut(kind) {
            for ch in channels.iter_mut() {
                ch.push(make());
            }
        }
    }

    /// Create a new resource. Fails if the key already exists. The stored
    /// version is reset to 0. Emits a `Created` watch event.
    pub fn create(&mut self, resource: AnyResource) -> StoreResult<u64> {
        let key = resource.metadata().key();
        if self.resources.contains_key(&key) {
            return Err(StoreError::AlreadyExists(key));
        }
        let mut resource = resource;
        // Fresh resources start at version 1 in COSI (0 is "unversioned").
        resource.metadata_mut().set_version(1);
        let kind = resource.resource_kind();
        let version = resource.metadata().version();
        let stored = resource.clone_box();
        self.resources.insert(key, resource);
        self.fanout(&kind, || Event::created(stored.clone_box()));
        Ok(version)
    }

    /// Update an existing resource using optimistic concurrency. `expected_version`
    /// must equal the stored version. If the new spec fingerprint and metadata
    /// are unchanged, the update is a no-op and the version is not bumped.
    /// Emits an `Updated` event when state actually changes.
    pub fn update(&mut self, resource: AnyResource, expected_version: u64) -> StoreResult<u64> {
        let key = resource.metadata().key();
        let Some(existing) = self.resources.get(&key) else {
            return Err(StoreError::NotFound(key));
        };
        let actual = existing.metadata().version();
        if actual != expected_version {
            return Err(StoreError::Conflict {
                key,
                expected: expected_version,
                actual,
            });
        }
        // Owner enforcement: a controller-owned resource keeps its owner.
        let prev_owner = existing.metadata().owner().to_string();
        let new_owner = resource.metadata().owner().to_string();
        if !prev_owner.is_empty() && new_owner != prev_owner {
            return Err(StoreError::OwnerConflict {
                key,
                owner: prev_owner,
            });
        }

        // No-op detection: same fingerprint, phase, finalizers and labels.
        let unchanged = existing.spec_fingerprint() == resource.spec_fingerprint()
            && existing.metadata().phase() == resource.metadata().phase()
            && existing.metadata().owner() == resource.metadata().owner()
            && existing.metadata().finalizers() == resource.metadata().finalizers()
            && existing.metadata().labels() == resource.metadata().labels();
        if unchanged {
            return Ok(actual);
        }

        let old = existing.clone_box();
        let mut resource = resource;
        resource.metadata_mut().set_version(actual + 1);
        let kind = resource.resource_kind();
        let version = resource.metadata().version();
        let stored = resource.clone_box();
        self.resources.insert(key, resource);
        self.fanout(&kind, || {
            Event::updated(stored.clone_box(), Some(old.clone_box()))
        });
        Ok(version)
    }

    /// Request teardown: transition the resource to [`Phase::TearingDown`] so
    /// controllers can remove finalizers. Idempotent. Emits an `Updated` event
    /// on the first transition. Returns the resource's version.
    pub fn teardown(&mut self, key: &str, expected_version: u64) -> StoreResult<u64> {
        let Some(existing) = self.resources.get(key) else {
            return Err(StoreError::NotFound(key.to_string()));
        };
        let actual = existing.metadata().version();
        if actual != expected_version {
            return Err(StoreError::Conflict {
                key: key.to_string(),
                expected: expected_version,
                actual,
            });
        }
        if existing.metadata().phase() == Phase::TearingDown {
            return Ok(actual);
        }
        let old = existing.clone_box();
        let mut resource = self.resources.remove(key).unwrap();
        resource.metadata_mut().set_phase(Phase::TearingDown);
        resource.metadata_mut().bump_version();
        let kind = resource.resource_kind();
        let version = resource.metadata().version();
        let stored = resource.clone_box();
        self.resources.insert(key.to_string(), resource);
        self.fanout(&kind, || {
            Event::updated(stored.clone_box(), Some(old.clone_box()))
        });
        Ok(version)
    }

    /// Add a finalizer to a resource (controller registering interest). Emits an
    /// `Updated` event when newly added.
    pub fn add_finalizer(&mut self, key: &str, name: &str) -> StoreResult<()> {
        let existing = self
            .resources
            .get(key)
            .ok_or_else(|| StoreError::NotFound(key.to_string()))?;
        if existing.metadata().finalizers().contains(name) {
            return Ok(());
        }
        let old = existing.clone_box();
        let mut resource = self.resources.remove(key).unwrap();
        resource.metadata_mut().finalizers_mut().add(name);
        resource.metadata_mut().bump_version();
        let kind = resource.resource_kind();
        let stored = resource.clone_box();
        self.resources.insert(key.to_string(), resource);
        self.fanout(&kind, || {
            Event::updated(stored.clone_box(), Some(old.clone_box()))
        });
        Ok(())
    }

    /// Remove a finalizer (controller done cleaning up). Emits an `Updated`
    /// event when removed.
    pub fn remove_finalizer(&mut self, key: &str, name: &str) -> StoreResult<()> {
        let existing = self
            .resources
            .get(key)
            .ok_or_else(|| StoreError::NotFound(key.to_string()))?;
        if !existing.metadata().finalizers().contains(name) {
            return Ok(());
        }
        let old = existing.clone_box();
        let mut resource = self.resources.remove(key).unwrap();
        resource.metadata_mut().finalizers_mut().remove(name);
        resource.metadata_mut().bump_version();
        let kind = resource.resource_kind();
        let stored = resource.clone_box();
        self.resources.insert(key.to_string(), resource);
        self.fanout(&kind, || {
            Event::updated(stored.clone_box(), Some(old.clone_box()))
        });
        Ok(())
    }

    /// Destroy a resource. The resource must be tearing down with no remaining
    /// finalizers ([`Metadata::can_destroy`]). Emits a `Destroyed` event.
    pub fn destroy(&mut self, key: &str, expected_version: u64) -> StoreResult<()> {
        let Some(existing) = self.resources.get(key) else {
            return Err(StoreError::NotFound(key.to_string()));
        };
        let actual = existing.metadata().version();
        if actual != expected_version {
            return Err(StoreError::Conflict {
                key: key.to_string(),
                expected: expected_version,
                actual,
            });
        }
        if !existing.metadata().can_destroy() {
            return Err(StoreError::StillReferenced(key.to_string()));
        }
        let removed = self.resources.remove(key).unwrap();
        let kind = removed.resource_kind();
        self.fanout(&kind, || Event::destroyed(removed.clone_box()));
        Ok(())
    }
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("State")
            .field("resources", &self.resources.len())
            .field("watched_kinds", &self.watches.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::{Labels, Metadata};
    use crate::resource::Resource;
    use os_kernel::ResourceId;

    #[derive(Debug, Clone)]
    struct Cfg {
        meta: Metadata,
        replicas: u32,
    }

    impl Cfg {
        fn new(id: &str, replicas: u32) -> Self {
            Cfg {
                meta: Metadata::new("default", "Cfg", ResourceId::new(id).unwrap()),
                replicas,
            }
        }
        fn boxed(self) -> AnyResource {
            Box::new(self)
        }
    }

    impl Resource for Cfg {
        fn metadata(&self) -> &Metadata {
            &self.meta
        }
        fn metadata_mut(&mut self) -> &mut Metadata {
            &mut self.meta
        }
        fn spec_fingerprint(&self) -> String {
            format!("replicas={}", self.replicas)
        }
        fn clone_box(&self) -> AnyResource {
            Box::new(self.clone())
        }
    }

    fn kind() -> ResourceKind {
        ResourceKind::new("default", "Cfg")
    }

    #[test]
    fn create_then_get_and_reject_duplicate() {
        let mut s = State::new();
        let v = s.create(Cfg::new("a", 1).boxed()).unwrap();
        assert_eq!(v, 1);
        assert!(s.contains("default/Cfg/a"));
        let got = s.get("default/Cfg/a").unwrap();
        assert_eq!(got.metadata().version(), 1);
        let err = s.create(Cfg::new("a", 9).boxed()).unwrap_err();
        assert!(matches!(err, StoreError::AlreadyExists(_)));
    }

    #[test]
    fn update_optimistic_concurrency_and_noop() {
        let mut s = State::new();
        s.create(Cfg::new("a", 1).boxed()).unwrap();
        // wrong version -> conflict
        let err = s.update(Cfg::new("a", 2).boxed(), 0).unwrap_err();
        assert!(matches!(
            err,
            StoreError::Conflict {
                expected: 0,
                actual: 1,
                ..
            }
        ));
        // correct version, changed spec -> bump
        let v = s.update(Cfg::new("a", 2).boxed(), 1).unwrap();
        assert_eq!(v, 2);
        // no-op update at correct version -> version unchanged
        let v2 = s.update(Cfg::new("a", 2).boxed(), 2).unwrap();
        assert_eq!(v2, 2);
    }

    #[test]
    fn teardown_finalizer_destroy_lifecycle() {
        let mut s = State::new();
        s.create(Cfg::new("a", 1).boxed()).unwrap();
        let key = "default/Cfg/a";
        s.add_finalizer(key, "ctrl").unwrap();
        let v = s.get(key).unwrap().metadata().version();
        // cannot destroy a running resource
        assert!(matches!(
            s.destroy(key, v).unwrap_err(),
            StoreError::StillReferenced(_)
        ));
        let v = s.teardown(key, v).unwrap();
        // still has finalizer -> cannot destroy
        assert!(matches!(
            s.destroy(key, v).unwrap_err(),
            StoreError::StillReferenced(_)
        ));
        s.remove_finalizer(key, "ctrl").unwrap();
        let v = s.get(key).unwrap().metadata().version();
        s.destroy(key, v).unwrap();
        assert!(!s.contains(key));
    }

    #[test]
    fn list_filters_by_label_selector() {
        let mut s = State::new();
        let mut a = Cfg::new("a", 1);
        a.meta.labels_mut().set("tier", "control-plane");
        let mut b = Cfg::new("b", 1);
        b.meta.labels_mut().set("tier", "worker");
        s.create(a.boxed()).unwrap();
        s.create(b.boxed()).unwrap();
        assert_eq!(s.list(&kind(), None).len(), 2);
        let mut sel = Labels::new();
        sel.set("tier", "worker");
        let filtered = s.list(&kind(), Some(&sel));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].metadata().id().as_str(), "b");
    }

    #[test]
    fn watch_bootstrap_snapshot_then_live_events() {
        let mut s = State::new();
        s.create(Cfg::new("a", 1).boxed()).unwrap();
        let idx = s.watch_kind(kind(), 16);
        // snapshot: Created(a) then Bootstrapped
        {
            let ch = s.watch_mut(&kind(), idx).unwrap();
            let events = ch.drain();
            assert_eq!(events.len(), 2);
            assert_eq!(events[0].kind(), crate::watch::EventKind::Created);
            assert_eq!(events[1].kind(), crate::watch::EventKind::Bootstrapped);
            assert!(ch.is_bootstrapped());
        }
        // live: create b -> Created event delivered to the watch
        s.create(Cfg::new("b", 1).boxed()).unwrap();
        let ch = s.watch_mut(&kind(), idx).unwrap();
        let live = ch.drain();
        assert_eq!(live.len(), 1);
        assert_eq!(live[0].reduced().unwrap().id().as_str(), "b");
    }

    #[test]
    fn owner_conflict_on_update() {
        let mut s = State::new();
        let mut owned = Cfg::new("a", 1);
        owned.meta.set_owner("controller-x");
        s.create(owned.boxed()).unwrap();
        // a different owner trying to update -> conflict
        let mut intruder = Cfg::new("a", 2);
        intruder.meta.set_owner("controller-y");
        let err = s.update(intruder.boxed(), 1).unwrap_err();
        assert!(matches!(err, StoreError::OwnerConflict { .. }));
    }
}
