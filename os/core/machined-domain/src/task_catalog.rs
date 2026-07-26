//! The concrete sequencer task catalog and standard sequence builders.
//!
//! Mirrors `siderolabs/talos` `internal/app/machined/pkg/runtime/v1alpha1/v1alpha1_sequencer_tasks.go`
//! and `v1alpha1_sequencer.go`: the named tasks machined runs and the ordered
//! [`Phase`] lists assembled for each [`Sequence`].
//!
//! The existing [`crate::sequencer::Sequencer`] is deliberately decoupled from
//! any concrete catalog; this module supplies the catalog the real Talos
//! sequencer would build, so a caller can run a faithful Boot/Install/Upgrade/
//! Reset/Reboot/Shutdown sequence end-to-end.
//!
//! Each task records the work it would do into a [`SideEffects`] ledger (via
//! the [`TaskContext`]-driven closures) so tests can assert the exact ordered
//! list of operations a sequence performs, the same way Talos integration tests
//! assert on the sequence of tasks. The real syscalls (mount, wipe, install)
//! are not performed; the ledger is the in-memory boundary.

use crate::phase::Phase;
use crate::sequence::Sequence;
use crate::task::{NamedTask, Task, TaskContext, TaskOutcome};
use std::cell::RefCell;
use std::rc::Rc;

/// An append-only ledger of the side effects a sequence's tasks performed.
///
/// Tasks in the catalog push a short opcode string here instead of touching the
/// real OS, so a test can assert the exact ordered task list that ran for a
/// given sequence/role/mode.
#[derive(Debug, Default, Clone)]
pub struct SideEffects {
    inner: Rc<RefCell<Vec<String>>>,
}

impl SideEffects {
    /// Create an empty ledger.
    pub fn new() -> Self {
        SideEffects {
            inner: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Record that an operation ran.
    pub fn record(&self, op: impl Into<String>) {
        self.inner.borrow_mut().push(op.into());
    }

    /// A snapshot copy of the recorded operations in order.
    pub fn ops(&self) -> Vec<String> {
        self.inner.borrow().clone()
    }

    /// Number of operations recorded.
    pub fn len(&self) -> usize {
        self.inner.borrow().len()
    }

    /// Whether nothing was recorded.
    pub fn is_empty(&self) -> bool {
        self.inner.borrow().is_empty()
    }

    /// Whether a given op was recorded.
    pub fn contains(&self, op: &str) -> bool {
        self.inner.borrow().iter().any(|o| o == op)
    }

    /// The index of the first occurrence of `op`, if present.
    pub fn position(&self, op: &str) -> Option<usize> {
        self.inner.borrow().iter().position(|o| o == op)
    }
}

/// Build a catalog task that records `op` and reports [`TaskOutcome::Done`].
fn record_task(name: &str, op: &str, fx: &SideEffects) -> Box<dyn Task> {
    let fx = fx.clone();
    let op = op.to_string();
    Box::new(NamedTask::new(name, move |_ctx: &TaskContext| {
        fx.record(op.clone());
        Ok(TaskOutcome::Done)
    }))
}

/// A catalog task that records `op` then requests a reboot.
fn reboot_task(name: &str, op: &str, fx: &SideEffects) -> Box<dyn Task> {
    let fx = fx.clone();
    let op = op.to_string();
    Box::new(NamedTask::new(name, move |_ctx: &TaskContext| {
        fx.record(op.clone());
        Ok(TaskOutcome::RebootRequested)
    }))
}

/// A catalog task gated on the node being a control-plane node.
fn cp_task(name: &str, op: &str, fx: &SideEffects) -> Box<dyn Task> {
    let fx = fx.clone();
    let op = op.to_string();
    let gate: fn(&TaskContext) -> bool = |c| c.machine_type().is_control_plane();
    Box::new(
        NamedTask::new(name, move |_ctx: &TaskContext| {
            fx.record(op.clone());
            Ok(TaskOutcome::Done)
        })
        .with_gate(gate),
    )
}

/// A catalog task gated on the runtime having real block devices.
fn disk_task(name: &str, op: &str, fx: &SideEffects) -> Box<dyn Task> {
    let fx = fx.clone();
    let op = op.to_string();
    let gate: fn(&TaskContext) -> bool = |c| c.mode().has_disks();
    Box::new(
        NamedTask::new(name, move |_ctx: &TaskContext| {
            fx.record(op.clone());
            Ok(TaskOutcome::Done)
        })
        .with_gate(gate),
    )
}

/// The standard Talos sequence/phase/task catalog, parameterized by a
/// [`SideEffects`] ledger that every produced task records into.
///
/// Mirrors the Talos sequencer's per-sequence task builders. Each method
/// returns the ordered [`Phase`] list for one sequence; the
/// [`crate::sequencer::Sequencer`] runs that list.
pub struct TaskCatalog {
    fx: SideEffects,
}

impl TaskCatalog {
    /// Build a catalog writing into the given ledger.
    pub fn new(fx: SideEffects) -> Self {
        TaskCatalog { fx }
    }

    /// The ledger this catalog records into.
    pub fn effects(&self) -> &SideEffects {
        &self.fx
    }

    /// The phases for the `Boot` sequence.
    ///
    /// Mirrors Talos `Boot`: mount system disks, set up the environment, write
    /// the platform metadata, mount state/ephemeral, start the
    /// system/containerd services, then (on control-plane) bootstrap and join
    /// the cluster.
    pub fn boot(&self) -> Vec<Phase> {
        vec![
            Phase::new("systemRequirements")
                .with_task(record_task("enforceKSPP", "enforceKSPP", &self.fx))
                .with_task(record_task(
                    "setupSystemDirectory",
                    "setupSystemDirectory",
                    &self.fx,
                )),
            Phase::new("mountSystem")
                .with_task(disk_task("mountStatePartition", "mount(STATE)", &self.fx))
                .with_task(disk_task("mountEphemeral", "mount(EPHEMERAL)", &self.fx)),
            Phase::new("config")
                .with_task(record_task("loadConfig", "loadConfig", &self.fx))
                .with_task(record_task("validateConfig", "validateConfig", &self.fx)),
            Phase::new("env")
                .with_task(record_task("setUserEnvVars", "setUserEnvVars", &self.fx))
                .with_task(record_task("writeUserFiles", "writeUserFiles", &self.fx)),
            Phase::new("network").with_task(record_task("setupNetwork", "setupNetwork", &self.fx)),
            Phase::new("services")
                .with_task(record_task(
                    "startContainerd",
                    "start(containerd)",
                    &self.fx,
                ))
                .with_task(record_task("startUdevd", "start(udevd)", &self.fx))
                .with_task(record_task("startKubelet", "start(kubelet)", &self.fx)),
            Phase::new("bootstrap")
                .with_task(cp_task("startEtcd", "start(etcd)", &self.fx))
                .with_task(cp_task("bootstrapEtcd", "bootstrap(etcd)", &self.fx))
                .with_task(cp_task("labelNode", "labelNodeAsControlPlane", &self.fx)),
        ]
    }

    /// The phases for the `Install` sequence (metal/cloud only).
    pub fn install(&self) -> Vec<Phase> {
        vec![
            Phase::new("validateConfig").with_task(record_task(
                "validateConfig",
                "validateConfig",
                &self.fx,
            )),
            Phase::new("install")
                .with_task(disk_task("partitionDisk", "partition(disk)", &self.fx))
                .with_task(disk_task(
                    "formatPartitions",
                    "format(partitions)",
                    &self.fx,
                ))
                .with_task(disk_task("installAssets", "install(assets)", &self.fx))
                .with_task(disk_task(
                    "installBootloader",
                    "install(bootloader)",
                    &self.fx,
                )),
            Phase::new("reboot").with_task(reboot_task("rebootAfterInstall", "reboot", &self.fx)),
        ]
    }

    /// The phases for the `Upgrade` sequence.
    pub fn upgrade(&self) -> Vec<Phase> {
        vec![
            Phase::new("preflight")
                .with_task(cp_task("preflightEtcd", "preflight(etcd)", &self.fx))
                .with_task(record_task("cordonNode", "cordon(node)", &self.fx)),
            Phase::new("drain").with_task(record_task("drainNode", "drain(node)", &self.fx)),
            Phase::new("stopServices").with_task(record_task(
                "stopAllServices",
                "stop(all)",
                &self.fx,
            )),
            Phase::new("unmount").with_task(disk_task(
                "unmountEphemeral",
                "unmount(EPHEMERAL)",
                &self.fx,
            )),
            Phase::new("install").with_task(disk_task(
                "installUpgrade",
                "install(upgrade)",
                &self.fx,
            )),
            Phase::new("reboot").with_task(reboot_task("rebootAfterUpgrade", "reboot", &self.fx)),
        ]
    }

    /// The phases for the `Reset` sequence: wipe state, then reboot to
    /// maintenance.
    pub fn reset(&self) -> Vec<Phase> {
        vec![
            Phase::new("leaveCluster")
                .with_task(cp_task("leaveEtcd", "leave(etcd)", &self.fx))
                .with_task(record_task("resetKubelet", "reset(kubelet)", &self.fx)),
            Phase::new("stopServices").with_task(record_task(
                "stopAllServices",
                "stop(all)",
                &self.fx,
            )),
            Phase::new("wipe")
                .with_task(disk_task(
                    "unmountEphemeral",
                    "unmount(EPHEMERAL)",
                    &self.fx,
                ))
                .with_task(disk_task(
                    "wipeSystemDisk",
                    "wipe(STATE+EPHEMERAL)",
                    &self.fx,
                )),
            Phase::new("reboot").with_task(reboot_task("rebootAfterReset", "reboot", &self.fx)),
        ]
    }

    /// The phases for the `Reboot` sequence.
    pub fn reboot(&self) -> Vec<Phase> {
        vec![
            Phase::new("cordonAndDrain").with_task(record_task(
                "cordonNode",
                "cordon(node)",
                &self.fx,
            )),
            Phase::new("stopServices").with_task(record_task(
                "stopAllServices",
                "stop(all)",
                &self.fx,
            )),
            Phase::new("unmount").with_task(disk_task("unmountAll", "unmount(all)", &self.fx)),
            Phase::new("reboot").with_task(reboot_task("reboot", "reboot", &self.fx)),
        ]
    }

    /// The phases for the `Shutdown` sequence.
    pub fn shutdown(&self) -> Vec<Phase> {
        vec![
            Phase::new("stopServices").with_task(record_task(
                "stopAllServices",
                "stop(all)",
                &self.fx,
            )),
            Phase::new("unmount").with_task(disk_task("unmountAll", "unmount(all)", &self.fx)),
            Phase::new("poweroff").with_task(reboot_task("poweroff", "poweroff", &self.fx)),
        ]
    }

    /// The phases for the `StageUpgrade` sequence: write the staged upgrade
    /// marker for the next boot, without rebooting now.
    pub fn stage_upgrade(&self) -> Vec<Phase> {
        vec![Phase::new("stage").with_task(disk_task(
            "writeUpgradeMarker",
            "write(upgradeMarker)",
            &self.fx,
        ))]
    }

    /// The phases for the `MaintenanceUpgrade` sequence: validate then install
    /// directly from maintenance mode (no running cluster to drain).
    pub fn maintenance_upgrade(&self) -> Vec<Phase> {
        vec![
            Phase::new("validateConfig").with_task(record_task(
                "validateConfig",
                "validateConfig",
                &self.fx,
            )),
            Phase::new("install").with_task(disk_task(
                "installUpgrade",
                "install(upgrade)",
                &self.fx,
            )),
            Phase::new("reboot").with_task(reboot_task("rebootAfterUpgrade", "reboot", &self.fx)),
        ]
    }

    /// Dispatch to the phase builder for `seq`. `NoOp` yields no phases.
    pub fn phases_for(&self, seq: Sequence) -> Vec<Phase> {
        match seq {
            Sequence::Boot => self.boot(),
            Sequence::Install => self.install(),
            Sequence::Upgrade => self.upgrade(),
            Sequence::Reset => self.reset(),
            Sequence::Reboot => self.reboot(),
            Sequence::Shutdown => self.shutdown(),
            Sequence::StageUpgrade => self.stage_upgrade(),
            Sequence::MaintenanceUpgrade => self.maintenance_upgrade(),
            Sequence::NoOp => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{InMemoryRuntime, RuntimeMode};
    use crate::sequencer::Sequencer;
    use crate::state_machine::MachineState;
    use os_kernel::MachineType;

    fn run(
        mode: RuntimeMode,
        mt: MachineType,
        seq: Sequence,
        boot_first: bool,
    ) -> (SideEffects, crate::sequencer::SequenceReport) {
        let fx = SideEffects::new();
        let cat = TaskCatalog::new(fx.clone());
        let rt = InMemoryRuntime::new(mode, mt).with_config("node-1");
        let mut sequencer = Sequencer::new(rt);
        if boot_first {
            sequencer.run(Sequence::Boot, cat.boot()).unwrap();
        }
        let report = sequencer.run(seq, cat.phases_for(seq)).unwrap();
        (fx, report)
    }

    #[test]
    fn boot_runs_full_control_plane_catalog() {
        let (fx, report) = run(
            RuntimeMode::Metal,
            MachineType::ControlPlane,
            Sequence::Boot,
            false,
        );
        assert_eq!(report.final_state, MachineState::Running);
        // Control-plane bootstrap tasks ran.
        assert!(fx.contains("bootstrap(etcd)"));
        assert!(fx.contains("labelNodeAsControlPlane"));
        // Mount happened before services, services before bootstrap.
        assert!(fx.position("mount(STATE)").unwrap() < fx.position("start(kubelet)").unwrap());
        assert!(fx.position("start(kubelet)").unwrap() < fx.position("bootstrap(etcd)").unwrap());
    }

    #[test]
    fn worker_boot_skips_control_plane_tasks() {
        let (fx, _) = run(
            RuntimeMode::Metal,
            MachineType::Worker,
            Sequence::Boot,
            false,
        );
        assert!(!fx.contains("bootstrap(etcd)"));
        assert!(!fx.contains("labelNodeAsControlPlane"));
        // But shared tasks still ran.
        assert!(fx.contains("start(kubelet)"));
    }

    #[test]
    fn container_boot_skips_disk_tasks() {
        let fx = SideEffects::new();
        let cat = TaskCatalog::new(fx.clone());
        let rt =
            InMemoryRuntime::new(RuntimeMode::Container, MachineType::Worker).with_config("c-1");
        let mut sequencer = Sequencer::new(rt);
        sequencer.run(Sequence::Boot, cat.boot()).unwrap();
        // Disk mounts are gated out in a container.
        assert!(!fx.contains("mount(STATE)"));
        assert!(!fx.contains("mount(EPHEMERAL)"));
        // Non-disk tasks still ran.
        assert!(fx.contains("setupNetwork"));
    }

    #[test]
    fn install_reboots_at_end() {
        let fx = SideEffects::new();
        let cat = TaskCatalog::new(fx.clone());
        let rt = InMemoryRuntime::new(RuntimeMode::Metal, MachineType::ControlPlane);
        let mut sequencer = Sequencer::new(rt);
        let report = sequencer.run(Sequence::Install, cat.install()).unwrap();
        assert!(report.rebooted);
        assert_eq!(report.final_state, MachineState::Running);
        assert!(fx.contains("install(bootloader)"));
        // The "reboot" op is the last recorded op.
        assert_eq!(fx.ops().last().map(String::as_str), Some("reboot"));
    }

    #[test]
    fn upgrade_drains_before_install() {
        let (fx, report) = run(
            RuntimeMode::Metal,
            MachineType::ControlPlane,
            Sequence::Upgrade,
            true,
        );
        assert!(report.rebooted);
        assert!(fx.position("drain(node)").unwrap() < fx.position("install(upgrade)").unwrap());
        assert!(fx.position("stop(all)").unwrap() < fx.position("install(upgrade)").unwrap());
    }

    #[test]
    fn reset_wipes_then_reboots() {
        let (fx, report) = run(
            RuntimeMode::Metal,
            MachineType::ControlPlane,
            Sequence::Reset,
            true,
        );
        assert!(report.rebooted);
        // Reset completes back to Initializing (returns to maintenance).
        assert_eq!(report.final_state, MachineState::Initializing);
        assert!(fx.contains("leave(etcd)"));
        assert!(fx.position("wipe(STATE+EPHEMERAL)").unwrap() < fx.position("reboot").unwrap());
    }

    #[test]
    fn shutdown_powers_off_last() {
        let (fx, report) = run(
            RuntimeMode::Metal,
            MachineType::Worker,
            Sequence::Shutdown,
            true,
        );
        assert!(report.rebooted);
        assert_eq!(report.final_state, MachineState::ShuttingDown);
        assert_eq!(fx.ops().last().map(String::as_str), Some("poweroff"));
    }

    #[test]
    fn maintenance_upgrade_has_no_drain() {
        let fx = SideEffects::new();
        let cat = TaskCatalog::new(fx.clone());
        let rt =
            InMemoryRuntime::new(RuntimeMode::Metal, MachineType::ControlPlane).with_config("cfg");
        let mut sequencer = Sequencer::new(rt);
        let report = sequencer
            .run(Sequence::MaintenanceUpgrade, cat.maintenance_upgrade())
            .unwrap();
        assert!(report.rebooted);
        assert!(!fx.contains("drain(node)"));
        assert!(fx.contains("install(upgrade)"));
    }

    #[test]
    fn stage_upgrade_does_not_reboot() {
        let fx = SideEffects::new();
        let cat = TaskCatalog::new(fx.clone());
        let rt = InMemoryRuntime::new(RuntimeMode::Metal, MachineType::Worker).with_config("cfg");
        let mut sequencer = Sequencer::new(rt);
        let report = sequencer
            .run(Sequence::StageUpgrade, cat.stage_upgrade())
            .unwrap();
        assert!(!report.rebooted);
        assert!(fx.contains("write(upgradeMarker)"));
    }

    #[test]
    fn noop_has_no_phases() {
        let fx = SideEffects::new();
        let cat = TaskCatalog::new(fx.clone());
        assert!(cat.phases_for(Sequence::NoOp).is_empty());
    }

    #[test]
    fn side_effects_position_and_contains() {
        let fx = SideEffects::new();
        fx.record("a");
        fx.record("b");
        assert_eq!(fx.len(), 2);
        assert_eq!(fx.position("b"), Some(1));
        assert!(fx.contains("a"));
        assert!(!fx.contains("z"));
    }
}
