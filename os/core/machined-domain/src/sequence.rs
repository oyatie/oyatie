//! The set of machine sequences the sequencer can run.
//!
//! Mirrors `siderolabs/talos` `runtime.Sequence`. A sequence is a named,
//! ordered list of phases (defined in [`crate::sequencer`]); this enum is the
//! identity of each one plus the rules about which sequences may interrupt or
//! follow which others.

use crate::runtime::RuntimeMode;
use os_kernel::MachineType;

/// A machine lifecycle sequence.
///
/// Ordering of the variants follows the Talos protobuf enum so `as_i32`
/// round-trips against the API surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sequence {
    /// No sequence is running (idle).
    NoOp,
    /// Bring the machine up: mount, configure, start services, join cluster.
    Boot,
    /// Install Talos to disk (metal/cloud only).
    Install,
    /// Upgrade the installed Talos to a new version.
    Upgrade,
    /// Wipe machine state and (optionally) reboot back to maintenance.
    Reset,
    /// Stop services and power the machine off.
    Shutdown,
    /// Stop services and reboot the machine.
    Reboot,
    /// Stage an upgrade to run on next boot.
    StageUpgrade,
    /// Re-apply machine configuration without a full boot.
    MaintenanceUpgrade,
}

impl Sequence {
    /// Stable lowercase name used in logs and the API.
    pub fn as_str(self) -> &'static str {
        match self {
            Sequence::NoOp => "noop",
            Sequence::Boot => "boot",
            Sequence::Install => "install",
            Sequence::Upgrade => "upgrade",
            Sequence::Reset => "reset",
            Sequence::Shutdown => "shutdown",
            Sequence::Reboot => "reboot",
            Sequence::StageUpgrade => "stageUpgrade",
            Sequence::MaintenanceUpgrade => "maintenanceUpgrade",
        }
    }

    /// Numeric wire value matching the Talos protobuf enum ordering.
    pub fn as_i32(self) -> i32 {
        match self {
            Sequence::NoOp => 0,
            Sequence::Boot => 1,
            Sequence::Install => 2,
            Sequence::Upgrade => 3,
            Sequence::Reset => 4,
            Sequence::Shutdown => 5,
            Sequence::Reboot => 6,
            Sequence::StageUpgrade => 7,
            Sequence::MaintenanceUpgrade => 8,
        }
    }

    /// Whether this sequence ends with the host going down (reboot/poweroff).
    /// The sequencer uses this to decide whether to expect the process to exit.
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Sequence::Shutdown | Sequence::Reboot | Sequence::Upgrade | Sequence::Reset
        )
    }

    /// Whether the sequence touches block devices and therefore requires a
    /// runtime mode that [`RuntimeMode::has_disks`].
    pub fn requires_disks(self) -> bool {
        matches!(
            self,
            Sequence::Install | Sequence::Upgrade | Sequence::Reset | Sequence::StageUpgrade
        )
    }

    /// Whether this sequence is legal in the given runtime mode. Disk-bound
    /// sequences are rejected in modes without disks (e.g. `Container`).
    pub fn allowed_in(self, mode: RuntimeMode) -> bool {
        if self.requires_disks() && !mode.has_disks() {
            return false;
        }
        if matches!(self, Sequence::Reboot | Sequence::Shutdown) && !mode.can_reboot() {
            // Containers cannot reboot a kernel; only shutdown maps to exit.
            return matches!(self, Sequence::Shutdown);
        }
        true
    }

    /// Whether a higher-priority sequence (`incoming`) may interrupt this one.
    ///
    /// Talos lets `Reboot`/`Shutdown`/`Reset` pre-empt a running `Boot`, but a
    /// `Boot` may not interrupt an in-flight `Upgrade`. `NoOp` never interrupts.
    pub fn can_be_interrupted_by(self, incoming: Sequence) -> bool {
        if incoming == Sequence::NoOp {
            return false;
        }
        match self {
            // Boot can be pre-empted by any terminal/maintenance action.
            Sequence::Boot => matches!(
                incoming,
                Sequence::Reboot | Sequence::Shutdown | Sequence::Reset | Sequence::Upgrade
            ),
            // Terminal sequences run to completion; nothing interrupts them.
            Sequence::Shutdown
            | Sequence::Reboot
            | Sequence::Upgrade
            | Sequence::Reset
            | Sequence::StageUpgrade
            | Sequence::MaintenanceUpgrade => false,
            // Idle: any real sequence may start.
            Sequence::NoOp | Sequence::Install => incoming != Sequence::NoOp,
        }
    }

    /// Whether this sequence only makes sense on a configured machine.
    pub fn requires_config(self) -> bool {
        matches!(
            self,
            Sequence::Boot | Sequence::Upgrade | Sequence::MaintenanceUpgrade
        )
    }

    /// Whether running this sequence is meaningful for the given machine role.
    /// (All current sequences apply to every role; control-plane-only phase
    /// gating is handled at the task level.)
    pub fn applies_to(self, _machine_type: MachineType) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_wire_values_roundtrip_shape() {
        assert_eq!(Sequence::Boot.as_str(), "boot");
        assert_eq!(Sequence::Install.as_i32(), 2);
        assert_eq!(Sequence::MaintenanceUpgrade.as_i32(), 8);
    }

    #[test]
    fn disk_sequences_blocked_in_container() {
        assert!(!Sequence::Install.allowed_in(RuntimeMode::Container));
        assert!(!Sequence::Reset.allowed_in(RuntimeMode::Container));
        assert!(Sequence::Install.allowed_in(RuntimeMode::Metal));
        // Shutdown maps to container exit and is allowed; reboot is not.
        assert!(Sequence::Shutdown.allowed_in(RuntimeMode::Container));
        assert!(!Sequence::Reboot.allowed_in(RuntimeMode::Container));
        assert!(Sequence::Reboot.allowed_in(RuntimeMode::Metal));
    }

    #[test]
    fn boot_is_preemptible_by_reboot_but_not_noop() {
        assert!(Sequence::Boot.can_be_interrupted_by(Sequence::Reboot));
        assert!(Sequence::Boot.can_be_interrupted_by(Sequence::Upgrade));
        assert!(!Sequence::Boot.can_be_interrupted_by(Sequence::NoOp));
        assert!(!Sequence::Boot.can_be_interrupted_by(Sequence::Install));
    }

    #[test]
    fn upgrade_runs_to_completion() {
        assert!(!Sequence::Upgrade.can_be_interrupted_by(Sequence::Reboot));
        assert!(!Sequence::Reboot.can_be_interrupted_by(Sequence::Shutdown));
    }

    #[test]
    fn terminal_and_config_flags() {
        assert!(Sequence::Reboot.is_terminal());
        assert!(!Sequence::Boot.is_terminal());
        assert!(Sequence::Boot.requires_config());
        assert!(!Sequence::Install.requires_config());
    }
}
