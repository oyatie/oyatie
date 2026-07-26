//! Snapshot backup/restore orchestration.
//!
//! Talos exposes `talosctl etcd snapshot` (save) and the disaster-recovery
//! restore flow, plus a periodic on-disk backup. This module models the store
//! boundary as a trait ([`SnapshotStore`]) with an in-memory implementation,
//! and provides the higher-level [`take_backup`] / [`restore_from_backup`]
//! routines that drive the [`EtcdClient`] + [`SnapshotStore`].

use std::collections::BTreeMap;
use std::sync::Mutex;

use os_kernel::{Error, Result};

use crate::client::EtcdClient;
use crate::member::MemberId;
use crate::snapshot::Snapshot;

/// Where snapshots are persisted (local disk, object storage, ...).
pub trait SnapshotStore {
    /// Save a snapshot under a name, returning the bytes written.
    fn put(&self, name: &str, snapshot: &Snapshot) -> Result<u64>;

    /// Load a snapshot by name.
    fn get(&self, name: &str) -> Result<Snapshot>;

    /// List stored snapshot names, newest revision first.
    fn list(&self) -> Result<Vec<String>>;

    /// Delete a snapshot by name.
    fn delete(&self, name: &str) -> Result<()>;
}

/// In-memory snapshot store for tests.
#[derive(Debug, Default)]
pub struct InMemorySnapshotStore {
    inner: Mutex<BTreeMap<String, Snapshot>>,
}

impl InMemorySnapshotStore {
    /// New empty store.
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(BTreeMap::new()),
        }
    }

    /// Number of stored snapshots.
    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.lock().unwrap().is_empty()
    }
}

impl SnapshotStore for InMemorySnapshotStore {
    fn put(&self, name: &str, snapshot: &Snapshot) -> Result<u64> {
        snapshot.verify_integrity()?;
        self.inner
            .lock()
            .unwrap()
            .insert(name.to_string(), snapshot.clone());
        Ok(snapshot.metadata.total_size)
    }

    fn get(&self, name: &str) -> Result<Snapshot> {
        self.inner
            .lock()
            .unwrap()
            .get(name)
            .cloned()
            .ok_or_else(|| Error::not_found(format!("snapshot {name:?}")))
    }

    fn list(&self) -> Result<Vec<String>> {
        let map = self.inner.lock().unwrap();
        let mut entries: Vec<(&String, &Snapshot)> = map.iter().collect();
        entries.sort_by_key(|e| core::cmp::Reverse(e.1.metadata.revision));
        Ok(entries.into_iter().map(|(k, _)| k.clone()).collect())
    }

    fn delete(&self, name: &str) -> Result<()> {
        if self.inner.lock().unwrap().remove(name).is_none() {
            return Err(Error::not_found(format!("snapshot {name:?}")));
        }
        Ok(())
    }
}

/// Outcome of a backup operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupOutcome {
    /// Name the snapshot was stored under.
    pub name: String,
    /// Revision captured.
    pub revision: u64,
    /// Bytes written.
    pub bytes: u64,
}

/// Take a snapshot from `client`, persist it to `store` under `name`.
///
/// The snapshot bytes are synthesized from the current revision so the model
/// is self-consistent; a real implementation would stream the bbolt DB.
pub fn take_backup<C: EtcdClient, S: SnapshotStore>(
    client: &C,
    store: &S,
    name: &str,
    member: MemberId,
) -> Result<BackupOutcome> {
    let revision = client.revision()?;
    let status = client.status(member)?;
    if !status.alarms.is_empty() {
        return Err(Error::invalid_state(format!(
            "cannot snapshot member with alarms: {:?}",
            status.alarms
        )));
    }
    // Synthesize deterministic payload bytes from the revision/index.
    let mut data = Vec::new();
    data.extend_from_slice(&revision.to_be_bytes());
    data.extend_from_slice(&status.raft_index.to_be_bytes());
    data.extend_from_slice(name.as_bytes());
    let snapshot = Snapshot::create(
        revision,
        status.raft_term,
        status.raft_index,
        revision, // keys ~ revision for the model
        data,
    );
    let bytes = store.put(name, &snapshot)?;
    Ok(BackupOutcome {
        name: name.to_string(),
        revision,
        bytes,
    })
}

/// A plan describing how a node should be brought up from a restored snapshot.
///
/// Talos restore is offline: the snapshot is restored into a fresh data dir and
/// the node bootstraps a brand-new single-member cluster from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestorePlan {
    /// The snapshot to restore.
    pub snapshot_name: String,
    /// Revision of the restored data.
    pub revision: u64,
    /// The new single-member cluster name.
    pub member_name: String,
    /// Whether to skip the snapshot hash check (matches etcd's
    /// `--skip-hash-check`, used for snapshots taken by copying the DB file).
    pub skip_hash_check: bool,
}

/// Validate and produce a [`RestorePlan`] from a stored snapshot.
pub fn restore_from_backup<S: SnapshotStore>(
    store: &S,
    name: &str,
    member_name: &str,
    skip_hash_check: bool,
) -> Result<RestorePlan> {
    let snapshot = store.get(name)?;
    if !skip_hash_check {
        snapshot.verify_integrity()?;
    }
    if member_name.trim().is_empty() {
        return Err(Error::invalid("restore requires a member name"));
    }
    Ok(RestorePlan {
        snapshot_name: name.to_string(),
        revision: snapshot.metadata.revision,
        member_name: member_name.to_string(),
        skip_hash_check,
    })
}

/// Retain only the newest `keep` snapshots, deleting older ones. Returns the
/// names that were pruned.
pub fn prune_old_backups<S: SnapshotStore>(store: &S, keep: usize) -> Result<Vec<String>> {
    let names = store.list()?; // newest revision first
    let mut pruned = Vec::new();
    for name in names.into_iter().skip(keep) {
        store.delete(&name)?;
        pruned.push(name);
    }
    Ok(pruned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::InMemoryEtcd;

    #[test]
    fn backup_then_restore_roundtrip() {
        let (etcd, id) = InMemoryEtcd::with_bootstrap_member("cp1", vec!["https://a:2380".into()]);
        etcd.advance_leader_index(5);
        let store = InMemorySnapshotStore::new();
        let outcome = take_backup(&etcd, &store, "backup-1", id).unwrap();
        assert!(outcome.revision >= 6);
        assert_eq!(store.len(), 1);

        let plan = restore_from_backup(&store, "backup-1", "cp1", false).unwrap();
        assert_eq!(plan.revision, outcome.revision);
        assert_eq!(plan.member_name, "cp1");
    }

    #[test]
    fn restore_missing_snapshot_errors() {
        let store = InMemorySnapshotStore::new();
        assert!(restore_from_backup(&store, "nope", "cp1", false).is_err());
    }

    #[test]
    fn restore_rejects_empty_member_name() {
        let (etcd, id) = InMemoryEtcd::with_bootstrap_member("cp1", vec!["https://a:2380".into()]);
        let store = InMemorySnapshotStore::new();
        take_backup(&etcd, &store, "b", id).unwrap();
        assert!(restore_from_backup(&store, "b", "  ", false).is_err());
    }

    #[test]
    fn prune_keeps_newest() {
        let store = InMemorySnapshotStore::new();
        for rev in [10u64, 20, 30, 40] {
            let snap = Snapshot::create(rev, 1, rev, rev, vec![rev as u8]);
            store.put(&format!("s{rev}"), &snap).unwrap();
        }
        let pruned = prune_old_backups(&store, 2).unwrap();
        // Newest two (rev 40, 30) kept; 20 and 10 pruned.
        assert_eq!(pruned.len(), 2);
        assert!(pruned.contains(&"s20".to_string()));
        assert!(pruned.contains(&"s10".to_string()));
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn list_orders_newest_first() {
        let store = InMemorySnapshotStore::new();
        store
            .put("old", &Snapshot::create(5, 1, 1, 1, vec![1]))
            .unwrap();
        store
            .put("new", &Snapshot::create(50, 1, 1, 1, vec![2]))
            .unwrap();
        assert_eq!(store.list().unwrap(), vec!["new", "old"]);
    }
}
