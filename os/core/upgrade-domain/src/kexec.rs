//! `kexec` and A/B boot-partition switching.
//!
//! Talos installs the OS into one of two redundant system partitions, labeled
//! `A` and `B`. An upgrade writes the new system into the *inactive* partition
//! and flips the bootloader's default to it, leaving the old partition intact
//! so a failed boot can fall back. When `kexec` is enabled, instead of a full
//! firmware reboot Talos loads the new kernel/initramfs directly and jumps into
//! it, cutting reboot time dramatically (see
//! `internal/app/machined/pkg/runtime/v1alpha1/v1alpha1_sequencer_tasks.go`,
//! the `kexecPrepare`/`KexecPrepare` task).
//!
//! This module models the two partition labels, the boot-manager that flips the
//! active partition, and the `kexec` load/exec state machine. The kernel
//! `kexec_file_load` syscall is the OS boundary, expressed by [`KexecLoader`].

use alloc::string::{String, ToString};
use core::fmt;

/// The two redundant system-partition labels Talos uses for A/B upgrades.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionLabel {
    /// System partition A.
    A,
    /// System partition B.
    B,
}

impl PartitionLabel {
    /// The other partition (the inactive one becomes the upgrade target).
    pub fn other(self) -> PartitionLabel {
        match self {
            PartitionLabel::A => PartitionLabel::B,
            PartitionLabel::B => PartitionLabel::A,
        }
    }

    /// The on-disk GPT partition label string Talos uses.
    pub fn as_str(self) -> &'static str {
        match self {
            PartitionLabel::A => "A",
            PartitionLabel::B => "B",
        }
    }
}

impl fmt::Display for PartitionLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A bootloader menu entry describing what is installed in a partition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootEntry {
    /// Which physical partition this entry boots.
    pub label: PartitionLabel,
    /// OS version installed in the partition (e.g. `v1.8.0`), if any.
    pub version: Option<String>,
    /// Kernel command line for the entry.
    pub cmdline: String,
}

impl BootEntry {
    /// An empty entry for a partition with nothing installed.
    pub fn empty(label: PartitionLabel) -> Self {
        BootEntry {
            label,
            version: None,
            cmdline: String::new(),
        }
    }

    /// Whether this entry has a bootable OS installed.
    pub fn is_installed(&self) -> bool {
        self.version.is_some()
    }
}

/// A boot partition: a label plus its current installed contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootPartition {
    /// The partition entry.
    pub entry: BootEntry,
}

/// Errors raised by the boot/kexec flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KexecError {
    /// Tried to make a partition active that has nothing installed.
    NotInstalled(PartitionLabel),
    /// `kexec` was executed without a kernel being loaded first.
    NotLoaded,
    /// A kernel image was empty / failed validation.
    InvalidImage(String),
    /// A second load was attempted while one is already armed.
    AlreadyLoaded,
}

impl fmt::Display for KexecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KexecError::NotInstalled(p) => write!(f, "partition {p} has no OS installed"),
            KexecError::NotLoaded => write!(f, "no kexec image loaded"),
            KexecError::InvalidImage(m) => write!(f, "invalid kexec image: {m}"),
            KexecError::AlreadyLoaded => write!(f, "a kexec image is already loaded"),
        }
    }
}

/// The state of the `kexec` machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KexecState {
    /// No kernel loaded.
    Idle,
    /// `kexec_file_load` has armed a new kernel; awaiting exec.
    Loaded,
    /// `kexec` jumped into the new kernel (terminal).
    Executed,
}

/// The OS boundary for the two `kexec` syscalls (`kexec_file_load` and the
/// `reboot(LINUX_REBOOT_CMD_KEXEC)` jump).
pub trait KexecLoader {
    /// Load (arm) a kernel + initramfs with the given command line.
    fn load(&mut self, kernel: &[u8], initramfs: &[u8], cmdline: &str) -> Result<(), KexecError>;

    /// Whether a kernel is currently armed.
    fn is_loaded(&self) -> bool;

    /// Jump into the armed kernel. Terminal: no code runs afterwards.
    fn exec(&mut self) -> Result<(), KexecError>;
}

/// In-memory A/B boot manager and [`KexecLoader`] used in tests.
///
/// Tracks which partition is active, the two partition contents, and the kexec
/// arming state. It records whether a real reboot would have happened so the
/// upgrade sequence can assert on the boot path taken.
#[derive(Debug, Clone)]
pub struct InMemoryBootManager {
    active: PartitionLabel,
    a: BootEntry,
    b: BootEntry,
    kexec_enabled: bool,
    kexec_state: KexecState,
    loaded_cmdline: Option<String>,
}

impl InMemoryBootManager {
    /// Create a manager with `active` selected and an OS installed there.
    pub fn new(active: PartitionLabel, active_version: &str) -> Self {
        let installed = BootEntry {
            label: active,
            version: Some(active_version.to_string()),
            cmdline: "talos.platform=metal".to_string(),
        };
        let empty = BootEntry::empty(active.other());
        let (a, b) = match active {
            PartitionLabel::A => (installed, empty),
            PartitionLabel::B => (empty, installed),
        };
        InMemoryBootManager {
            active,
            a,
            b,
            kexec_enabled: true,
            kexec_state: KexecState::Idle,
            loaded_cmdline: None,
        }
    }

    /// Enable or disable kexec; when disabled, upgrades use a full reboot.
    pub fn set_kexec_enabled(&mut self, enabled: bool) {
        self.kexec_enabled = enabled;
    }

    /// Whether kexec is enabled.
    pub fn kexec_enabled(&self) -> bool {
        self.kexec_enabled
    }

    /// The currently active partition label.
    pub fn active(&self) -> PartitionLabel {
        self.active
    }

    /// The inactive partition that an upgrade installs into.
    pub fn inactive(&self) -> PartitionLabel {
        self.active.other()
    }

    /// Borrow the entry for a label.
    pub fn entry(&self, label: PartitionLabel) -> &BootEntry {
        match label {
            PartitionLabel::A => &self.a,
            PartitionLabel::B => &self.b,
        }
    }

    fn entry_mut(&mut self, label: PartitionLabel) -> &mut BootEntry {
        match label {
            PartitionLabel::A => &mut self.a,
            PartitionLabel::B => &mut self.b,
        }
    }

    /// The current kexec state.
    pub fn kexec_state(&self) -> KexecState {
        self.kexec_state
    }

    /// Install an OS version into the inactive partition (an upgrade write).
    pub fn install_inactive(&mut self, version: &str, cmdline: &str) {
        let target = self.inactive();
        let e = self.entry_mut(target);
        e.version = Some(version.to_string());
        e.cmdline = cmdline.to_string();
    }

    /// Flip the bootloader default to the inactive partition, which must have an
    /// OS installed. Returns the new active label.
    pub fn switch_to_inactive(&mut self) -> Result<PartitionLabel, KexecError> {
        let target = self.inactive();
        if !self.entry(target).is_installed() {
            return Err(KexecError::NotInstalled(target));
        }
        self.active = target;
        Ok(self.active)
    }

    /// Roll the active partition back to the other one if it still has an OS.
    pub fn switch_back(&mut self) -> Result<PartitionLabel, KexecError> {
        let target = self.active.other();
        if !self.entry(target).is_installed() {
            return Err(KexecError::NotInstalled(target));
        }
        self.active = target;
        Ok(self.active)
    }

    /// The command line the most recent `load` armed, if any.
    pub fn loaded_cmdline(&self) -> Option<&str> {
        self.loaded_cmdline.as_deref()
    }
}

impl KexecLoader for InMemoryBootManager {
    fn load(&mut self, kernel: &[u8], initramfs: &[u8], cmdline: &str) -> Result<(), KexecError> {
        if self.kexec_state == KexecState::Loaded {
            return Err(KexecError::AlreadyLoaded);
        }
        if kernel.is_empty() {
            return Err(KexecError::InvalidImage("empty kernel".to_string()));
        }
        if initramfs.is_empty() {
            return Err(KexecError::InvalidImage("empty initramfs".to_string()));
        }
        self.kexec_state = KexecState::Loaded;
        self.loaded_cmdline = Some(cmdline.to_string());
        Ok(())
    }

    fn is_loaded(&self) -> bool {
        self.kexec_state == KexecState::Loaded
    }

    fn exec(&mut self) -> Result<(), KexecError> {
        if self.kexec_state != KexecState::Loaded {
            return Err(KexecError::NotLoaded);
        }
        self.kexec_state = KexecState::Executed;
        Ok(())
    }
}

/// Prepare and perform the boot transition for an upgrade.
///
/// Installs the new OS into the inactive partition, flips the bootloader, and
/// then either arms+execs kexec (fast path) or signals a full reboot. Returns
/// the boot method used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootMethod {
    /// Jumped into the new kernel via kexec.
    Kexec,
    /// Requires a full firmware reboot.
    FullReboot,
}

/// Drive an upgrade boot transition on a [`InMemoryBootManager`].
pub fn perform_upgrade_boot(
    mgr: &mut InMemoryBootManager,
    new_version: &str,
    kernel: &[u8],
    initramfs: &[u8],
    cmdline: &str,
) -> Result<BootMethod, KexecError> {
    mgr.install_inactive(new_version, cmdline);
    mgr.switch_to_inactive()?;

    if mgr.kexec_enabled() {
        mgr.load(kernel, initramfs, cmdline)?;
        mgr.exec()?;
        Ok(BootMethod::Kexec)
    } else {
        Ok(BootMethod::FullReboot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partition_other_and_str() {
        assert_eq!(PartitionLabel::A.other(), PartitionLabel::B);
        assert_eq!(PartitionLabel::B.other(), PartitionLabel::A);
        assert_eq!(PartitionLabel::A.as_str(), "A");
        assert_eq!(PartitionLabel::B.to_string(), "B");
    }

    #[test]
    fn new_manager_has_active_installed_inactive_empty() {
        let mgr = InMemoryBootManager::new(PartitionLabel::A, "v1.7.0");
        assert_eq!(mgr.active(), PartitionLabel::A);
        assert_eq!(mgr.inactive(), PartitionLabel::B);
        assert!(mgr.entry(PartitionLabel::A).is_installed());
        assert!(!mgr.entry(PartitionLabel::B).is_installed());
    }

    #[test]
    fn cannot_switch_to_empty_partition() {
        let mut mgr = InMemoryBootManager::new(PartitionLabel::A, "v1.7.0");
        assert_eq!(
            mgr.switch_to_inactive(),
            Err(KexecError::NotInstalled(PartitionLabel::B))
        );
    }

    #[test]
    fn install_then_switch_flips_active() {
        let mut mgr = InMemoryBootManager::new(PartitionLabel::A, "v1.7.0");
        mgr.install_inactive("v1.8.0", "talos.platform=metal");
        assert!(mgr.entry(PartitionLabel::B).is_installed());
        let now = mgr.switch_to_inactive().unwrap();
        assert_eq!(now, PartitionLabel::B);
        assert_eq!(mgr.active(), PartitionLabel::B);
        assert_eq!(
            mgr.entry(PartitionLabel::B).version.as_deref(),
            Some("v1.8.0")
        );
    }

    #[test]
    fn kexec_load_exec_state_machine() {
        let mut mgr = InMemoryBootManager::new(PartitionLabel::A, "v1.7.0");
        assert_eq!(mgr.kexec_state(), KexecState::Idle);
        assert!(mgr.exec().is_err()); // nothing loaded

        mgr.load(b"vmlinuz", b"initramfs.xz", "console=ttyS0")
            .unwrap();
        assert!(mgr.is_loaded());
        assert_eq!(mgr.loaded_cmdline(), Some("console=ttyS0"));
        assert_eq!(mgr.load(b"x", b"y", ""), Err(KexecError::AlreadyLoaded));

        mgr.exec().unwrap();
        assert_eq!(mgr.kexec_state(), KexecState::Executed);
    }

    #[test]
    fn kexec_rejects_empty_images() {
        let mut mgr = InMemoryBootManager::new(PartitionLabel::A, "v1.7.0");
        assert!(matches!(
            mgr.load(b"", b"x", ""),
            Err(KexecError::InvalidImage(_))
        ));
        assert!(matches!(
            mgr.load(b"x", b"", ""),
            Err(KexecError::InvalidImage(_))
        ));
    }

    #[test]
    fn upgrade_boot_kexec_fast_path() {
        let mut mgr = InMemoryBootManager::new(PartitionLabel::A, "v1.7.0");
        let method =
            perform_upgrade_boot(&mut mgr, "v1.8.0", b"vmlinuz", b"initramfs", "cmdline").unwrap();
        assert_eq!(method, BootMethod::Kexec);
        assert_eq!(mgr.active(), PartitionLabel::B);
        assert_eq!(mgr.kexec_state(), KexecState::Executed);
    }

    #[test]
    fn upgrade_boot_full_reboot_when_kexec_disabled() {
        let mut mgr = InMemoryBootManager::new(PartitionLabel::A, "v1.7.0");
        mgr.set_kexec_enabled(false);
        let method =
            perform_upgrade_boot(&mut mgr, "v1.8.0", b"vmlinuz", b"initramfs", "cmdline").unwrap();
        assert_eq!(method, BootMethod::FullReboot);
        assert_eq!(mgr.active(), PartitionLabel::B);
        assert_eq!(mgr.kexec_state(), KexecState::Idle);
    }

    #[test]
    fn switch_back_rolls_to_previous_partition() {
        let mut mgr = InMemoryBootManager::new(PartitionLabel::A, "v1.7.0");
        mgr.install_inactive("v1.8.0", "c");
        mgr.switch_to_inactive().unwrap();
        assert_eq!(mgr.active(), PartitionLabel::B);
        // Old A still installed -> can roll back.
        let back = mgr.switch_back().unwrap();
        assert_eq!(back, PartitionLabel::A);
    }
}
