//! The `Bootloader` abstraction and crate-local error type.
//!
//! Talos supports multiple bootloaders behind one interface
//! (`internal/pkg/bootloader.Bootloader`): GRUB on BIOS/legacy-UEFI installs
//! and systemd-boot (with UKIs) on SecureBoot installs. The installer probes
//! the disk, picks an implementation, and drives it through the same
//! `Install`/`Revert`/`PreviousLabel` surface.

use crate::boot_entry::{BootEntry, BootSlot};
use std::fmt;

/// Errors raised by bootloader config generation and install/upgrade flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootloaderError {
    /// A required field (kernel path, cmdline, ...) was empty or malformed.
    InvalidConfig(String),
    /// The targeted boot slot is not provisioned / has no kernel.
    SlotNotPopulated(BootEntry),
    /// An attempt to revert when there is no previous slot to fall back to.
    NoPreviousEntry,
    /// The disk layout did not match what this bootloader requires
    /// (missing EFI system partition, wrong label, ...).
    UnsupportedLayout(String),
    /// SecureBoot is required but the image/signature is not trusted.
    SecureBootViolation(String),
    /// Underlying I/O at the syscall boundary failed.
    Io(String),
}

impl BootloaderError {
    /// Stable kind string for matching/logging.
    pub fn kind(&self) -> &'static str {
        match self {
            BootloaderError::InvalidConfig(_) => "invalid_config",
            BootloaderError::SlotNotPopulated(_) => "slot_not_populated",
            BootloaderError::NoPreviousEntry => "no_previous_entry",
            BootloaderError::UnsupportedLayout(_) => "unsupported_layout",
            BootloaderError::SecureBootViolation(_) => "secureboot_violation",
            BootloaderError::Io(_) => "io",
        }
    }
}

impl fmt::Display for BootloaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BootloaderError::InvalidConfig(m) => write!(f, "invalid bootloader config: {m}"),
            BootloaderError::SlotNotPopulated(e) => write!(f, "boot slot {e} not populated"),
            BootloaderError::NoPreviousEntry => write!(f, "no previous boot entry to revert to"),
            BootloaderError::UnsupportedLayout(m) => write!(f, "unsupported disk layout: {m}"),
            BootloaderError::SecureBootViolation(m) => write!(f, "secureboot violation: {m}"),
            BootloaderError::Io(m) => write!(f, "bootloader io error: {m}"),
        }
    }
}

/// Map a bootloader error into the workspace-wide [`os_kernel::Error`].
impl From<BootloaderError> for os_kernel::Error {
    fn from(e: BootloaderError) -> Self {
        match e {
            BootloaderError::InvalidConfig(m) => os_kernel::Error::Invalid(m),
            BootloaderError::SlotNotPopulated(_) => os_kernel::Error::InvalidState(e_display(&e)),
            BootloaderError::NoPreviousEntry => {
                os_kernel::Error::InvalidState("no previous boot entry".to_string())
            }
            BootloaderError::UnsupportedLayout(m) => os_kernel::Error::Unsupported(m),
            BootloaderError::SecureBootViolation(m) => os_kernel::Error::PermissionDenied(m),
            BootloaderError::Io(m) => os_kernel::Error::Other(m),
        }
    }
}

fn e_display(e: &BootloaderError) -> String {
    format!("{e}")
}

/// Crate-local result alias.
pub type BootResult<T> = std::result::Result<T, BootloaderError>;

/// Which family of bootloader an implementation belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootloaderKind {
    /// GRUB (BIOS / legacy UEFI).
    Grub,
    /// systemd-boot (UEFI, supports UKIs and SecureBoot).
    SystemdBoot,
}

/// The common interface every bootloader implements.
///
/// Modeled on `internal/pkg/bootloader.Bootloader`. The installer interacts
/// with the disk only through this trait so GRUB and systemd-boot are
/// interchangeable.
pub trait Bootloader {
    /// Which family this is.
    fn kind(&self) -> BootloaderKind;

    /// Install/update the given slot and make it the default for next boot.
    /// Generates and writes the bootloader configuration.
    fn install(&mut self, slot: &BootSlot) -> BootResult<()>;

    /// The slot that will boot by default.
    fn default_entry(&self) -> Option<BootEntry>;

    /// Revert the default to the previously-active slot (used after a failed
    /// upgrade). Errors if there is no previous slot.
    fn revert(&mut self) -> BootResult<()>;

    /// Render the on-disk configuration as text (GRUB script / loader conf).
    fn render_config(&self) -> BootResult<String>;

    /// Whether this bootloader requires/produces SecureBoot-signed artifacts.
    fn requires_secureboot(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_kinds_and_display() {
        assert_eq!(BootloaderError::NoPreviousEntry.kind(), "no_previous_entry");
        assert_eq!(
            BootloaderError::SlotNotPopulated(BootEntry::A).kind(),
            "slot_not_populated"
        );
        let d = format!("{}", BootloaderError::InvalidConfig("empty cmdline".into()));
        assert_eq!(d, "invalid bootloader config: empty cmdline");
    }

    #[test]
    fn converts_into_core_error() {
        let core: os_kernel::Error = BootloaderError::UnsupportedLayout("no ESP".into()).into();
        assert_eq!(core.kind(), "unsupported");
        let core2: os_kernel::Error =
            BootloaderError::SecureBootViolation("untrusted".into()).into();
        assert_eq!(core2.kind(), "permission_denied");
    }
}
