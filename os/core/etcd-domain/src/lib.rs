//! talos-etcd
//!
//! Manages the etcd cluster that backs the Kubernetes control plane on Talos:
//! bootstrap-vs-join decisions, member add/remove and learner promotion, the
//! etcd configuration/spec, PKI wiring, snapshots/backups, defrag, and
//! leave/forfeit-leadership during upgrades and resets. Mirrors Talos's `etcd`
//! service, `pkg/etcd` client, and the
//! `internal/app/machined/pkg/controllers/etcd` controllers.
//!
//! ## Module map
//!
//! * [`member`] — member identity ([`MemberId`]), Raft role/phase state
//!   machine, and the [`MemberSet`] quorum math.
//! * [`config`] — [`EtcdConfig`]/[`EtcdPki`], bootstrap-vs-join, and rendering
//!   the etcd process arguments.
//! * [`client`] — the [`EtcdClient`] trait (membership, status, defrag,
//!   snapshot, leadership) plus an in-memory simulator.
//! * [`snapshot`] — snapshot metadata, integrity hashing, validation.
//! * [`backup`] — the [`SnapshotStore`] boundary and backup/restore/prune
//!   orchestration.
//! * [`spec_controller`] — the spec + lifecycle reconcilers.
//!
//! All OS boundaries (the etcd gRPC API, snapshot storage) are modeled as
//! traits with in-memory implementations, so the crate builds and tests fully
//! offline with no external dependencies beyond `talos-core`.

pub mod backup;
pub mod client;
pub mod config;
pub mod member;
pub mod snapshot;
pub mod spec_controller;

pub use backup::{
    BackupOutcome, InMemorySnapshotStore, RestorePlan, SnapshotStore, prune_old_backups,
    restore_from_backup, take_backup,
};
pub use client::{EtcdClient, InMemoryEtcd, MemberStatus, member_role};
pub use config::{BootstrapMode, DEFAULT_CLIENT_PORT, DEFAULT_PEER_PORT, EtcdConfig, EtcdPki};
pub use member::{Member, MemberId, MemberPhase, MemberRole, MemberSet};
pub use snapshot::{SNAPSHOT_MAGIC, Snapshot, SnapshotMetadata, crc32};
pub use spec_controller::{
    EtcdLifecycleController, EtcdSpecController, LifecycleAction, SpecInput,
};

use os_kernel::Result;

/// The fragmentation ratio above which a member should be defragmented. Talos
/// runs a periodic defrag controller that triggers when the DB is sufficiently
/// fragmented.
pub const DEFAULT_DEFRAG_THRESHOLD: f64 = 2.0;

/// A maintenance recommendation for one member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaintenanceAction {
    /// No maintenance needed.
    None,
    /// The member should be defragmented (fragmentation over threshold).
    Defragment(MemberId),
    /// The member has raised alarms that require operator attention.
    Alarm { id: MemberId, alarms: Vec<String> },
}

/// Decide whether a member needs maintenance, given a defrag threshold.
pub fn evaluate_maintenance<C: EtcdClient>(
    client: &C,
    id: MemberId,
    defrag_threshold: f64,
) -> Result<MaintenanceAction> {
    let status = client.status(id)?;
    if !status.alarms.is_empty() {
        return Ok(MaintenanceAction::Alarm {
            id,
            alarms: status.alarms,
        });
    }
    if status.fragmentation_ratio() >= defrag_threshold {
        return Ok(MaintenanceAction::Defragment(id));
    }
    Ok(MaintenanceAction::None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::InMemoryEtcd;

    #[test]
    fn maintenance_recommends_defrag_when_fragmented() {
        let (etcd, id) =
            InMemoryEtcd::with_bootstrap_member("cp1", vec!["https://10.0.0.1:2380".into()]);
        etcd.inflate_db(id, 8192, 1024);
        let action = evaluate_maintenance(&etcd, id, DEFAULT_DEFRAG_THRESHOLD).unwrap();
        assert_eq!(action, MaintenanceAction::Defragment(id));
    }

    #[test]
    fn maintenance_none_when_compact() {
        let (etcd, id) =
            InMemoryEtcd::with_bootstrap_member("cp1", vec!["https://10.0.0.1:2380".into()]);
        let action = evaluate_maintenance(&etcd, id, DEFAULT_DEFRAG_THRESHOLD).unwrap();
        assert_eq!(action, MaintenanceAction::None);
    }

    #[test]
    fn end_to_end_three_node_cluster() {
        // Bootstrap cp1, join cp2 and cp3 as learners, promote both.
        let (etcd, _) =
            InMemoryEtcd::with_bootstrap_member("cp1", vec!["https://10.0.0.1:2380".into()]);
        for (name, ip) in [("cp2", "10.0.0.2"), ("cp3", "10.0.0.3")] {
            let id = etcd
                .member_add_as_learner(name, &[format!("https://{ip}:2380")])
                .unwrap();
            etcd.sync_member(id);
            etcd.member_promote(id).unwrap();
        }
        let members = etcd.member_list().unwrap();
        let set = MemberSet::from_members(members);
        assert_eq!(set.voter_count(), 3);
        assert_eq!(set.quorum(), 2);
    }
}
