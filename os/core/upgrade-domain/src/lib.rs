#![cfg_attr(not(test), no_std)]
//! # talos-upgrade
//!
//! Orchestrates Talos OS upgrades: the `Upgrade` API flow, draining/cordoning
//! and etcd leave preflight, staged vs. immediate install, A/B boot-partition
//! switching via `kexec`, the `upgrade-k8s` helper logic, and rollback to the
//! previous version. Mirrors the Talos upgrade sequence in
//! `internal/app/machined/pkg/runtime/v1alpha1/v1alpha1_sequencer.go`, the
//! `Upgrade` gRPC method, and the `pkg/cluster/kubernetes` upgrade-k8s helpers.
//!
//! The whole crate is built around the same OS-boundary trait pattern used by
//! `talos-core`: every interaction with the kernel, the META partition, etcd,
//! and the Kubernetes API server is expressed as a small trait with a
//! deterministic in-memory implementation so the upgrade state machines can be
//! exercised entirely in unit tests.
//!
//! The crate is `no_std` for real builds (using only `alloc`); under `cargo
//! test` it links against `std`.

extern crate alloc;

pub mod drain;
pub mod kexec;
pub mod rollback;
pub mod staged;
pub mod upgrade;
pub mod upgrade_k8s;

pub use drain::{
    DrainController, DrainError, DrainOptions, DrainState, InMemoryNodeApi, NodeApi, NodeStatus,
    PodDisruption,
};
pub use kexec::{
    BootEntry, BootPartition, InMemoryBootManager, KexecError, KexecLoader, KexecState,
    PartitionLabel,
};
pub use rollback::{InMemoryBootHistory, RollbackController, RollbackError, RollbackOutcome};
pub use staged::{
    InMemoryMetaStore, MetaStore, StagedMetaKey, StagedUpgrade, StagedUpgradeController,
    StagedUpgradeError, StagingState,
};
pub use upgrade::{
    EtcdControl, InMemoryEtcd, UpgradeController, UpgradeError, UpgradePhase, UpgradeRequest,
    UpgradeStep,
};
pub use upgrade_k8s::{
    ComponentKind, ComponentVersions, InMemoryK8sApi, K8sApi, K8sUpgradeError, K8sUpgradePlan,
    UpgradeK8sController,
};

#[cfg(test)]
mod tests {
    use super::*;
    use os_kernel::Version;

    #[test]
    fn crate_smoke() {
        // The public surface re-exports the key controllers.
        let v17 = Version::new(1, 7, 0);
        let v18 = Version::new(1, 8, 0);
        assert!(v17.is_upgrade_allowed_to(&v18));
    }

    #[test]
    fn end_to_end_immediate_upgrade() {
        // Drive a complete immediate (non-staged) upgrade through the
        // controller and confirm the sequence reaches Rebooting.
        let req = UpgradeRequest::new(
            Version::new(1, 7, 0),
            "ghcr.io/siderolabs/installer:v1.8.0",
            Version::new(1, 8, 0),
        )
        .unwrap();
        let mut ctrl = UpgradeController::new(req);
        let outcome = ctrl.run_to_completion().unwrap();
        assert_eq!(outcome, UpgradePhase::Rebooting);
        assert!(ctrl.completed_steps().contains(&UpgradeStep::Install));
    }
}
