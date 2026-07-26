//! Upgrade rollback.
//!
//! Talos's `Rollback` API reverts to the previous OS install. Because upgrades
//! use A/B system partitions and write the previous version into the META
//! `Upgrade` key, rollback is: flip the bootloader default back to the other
//! partition (which still holds the prior OS) and reboot. Rollback is only
//! possible while the previous partition still contains a bootable install and
//! a previous-version marker exists; once the inactive partition is overwritten
//! by a *subsequent* upgrade, rollback is no longer available.
//!
//! This module models the rollback decision and execution against the boot
//! manager from [`crate::kexec`] and a small boot-history record.

use crate::kexec::{InMemoryBootManager, KexecError, PartitionLabel};
use alloc::string::{String, ToString};
use core::fmt;

/// Errors raised by rollback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RollbackError {
    /// No previous version marker exists (never upgraded, or marker cleared).
    NoPreviousVersion,
    /// The previous partition no longer holds a bootable OS.
    PreviousPartitionEmpty(PartitionLabel),
    /// The boot manager rejected the partition switch.
    Boot(KexecError),
}

impl fmt::Display for RollbackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RollbackError::NoPreviousVersion => write!(f, "no previous version to roll back to"),
            RollbackError::PreviousPartitionEmpty(p) => {
                write!(f, "previous partition {p} has no OS installed")
            }
            RollbackError::Boot(e) => write!(f, "rollback boot switch failed: {e}"),
        }
    }
}

impl From<KexecError> for RollbackError {
    fn from(e: KexecError) -> Self {
        RollbackError::Boot(e)
    }
}

/// The result of a successful rollback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollbackOutcome {
    /// The partition now active after rollback.
    pub active: PartitionLabel,
    /// The OS version booted by the rollback.
    pub version: String,
}

/// A minimal record of which versions were installed in each partition, used to
/// reason about rollback eligibility independently of the live boot manager.
#[derive(Debug, Default, Clone)]
pub struct InMemoryBootHistory {
    previous_version: Option<String>,
}

impl InMemoryBootHistory {
    /// Empty history (no upgrade performed).
    pub fn new() -> Self {
        InMemoryBootHistory::default()
    }

    /// Record that an upgrade happened away from `previous`.
    pub fn record_upgrade(&mut self, previous: &str) {
        self.previous_version = Some(previous.to_string());
    }

    /// Clear the previous-version marker (e.g. after a successful boot
    /// commits the new version — Talos clears the META `Upgrade` key).
    pub fn commit(&mut self) {
        self.previous_version = None;
    }

    /// The previous version available for rollback, if any.
    pub fn previous_version(&self) -> Option<&str> {
        self.previous_version.as_deref()
    }

    /// Whether a rollback target exists.
    pub fn can_rollback(&self) -> bool {
        self.previous_version.is_some()
    }
}

/// Drives a rollback against a boot manager and boot history.
#[derive(Debug)]
pub struct RollbackController<'a> {
    mgr: &'a mut InMemoryBootManager,
    history: &'a mut InMemoryBootHistory,
}

impl<'a> RollbackController<'a> {
    /// Wrap the boot manager and history.
    pub fn new(mgr: &'a mut InMemoryBootManager, history: &'a mut InMemoryBootHistory) -> Self {
        RollbackController { mgr, history }
    }

    /// Whether rollback is currently possible: a previous-version marker exists
    /// and the inactive (previous) partition still holds a bootable OS.
    pub fn is_available(&self) -> bool {
        self.history.can_rollback() && self.mgr.entry(self.mgr.active().other()).is_installed()
    }

    /// Execute the rollback: flip back to the previous partition and clear the
    /// marker. Does not reboot itself — the caller triggers the reboot/kexec.
    pub fn rollback(&mut self) -> Result<RollbackOutcome, RollbackError> {
        let previous = self
            .history
            .previous_version()
            .ok_or(RollbackError::NoPreviousVersion)?
            .to_string();

        let target = self.mgr.active().other();
        if !self.mgr.entry(target).is_installed() {
            return Err(RollbackError::PreviousPartitionEmpty(target));
        }

        let now_active = self.mgr.switch_back()?;
        let version = self
            .mgr
            .entry(now_active)
            .version
            .clone()
            .unwrap_or(previous);

        // Rollback consumes the previous-version marker.
        self.history.commit();

        Ok(RollbackOutcome {
            active: now_active,
            version,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a manager that has already been upgraded A(v1.7.0) -> B(v1.8.0),
    /// so B is active and A still holds the old OS.
    fn upgraded_manager() -> InMemoryBootManager {
        let mut mgr = InMemoryBootManager::new(PartitionLabel::A, "v1.7.0");
        mgr.install_inactive("v1.8.0", "cmdline");
        mgr.switch_to_inactive().unwrap();
        mgr
    }

    #[test]
    fn boot_history_tracks_previous_version() {
        let mut h = InMemoryBootHistory::new();
        assert!(!h.can_rollback());
        h.record_upgrade("v1.7.0");
        assert!(h.can_rollback());
        assert_eq!(h.previous_version(), Some("v1.7.0"));
        h.commit();
        assert!(!h.can_rollback());
    }

    #[test]
    fn rollback_available_after_upgrade() {
        let mut mgr = upgraded_manager();
        let mut h = InMemoryBootHistory::new();
        h.record_upgrade("v1.7.0");
        let ctrl = RollbackController::new(&mut mgr, &mut h);
        assert!(ctrl.is_available());
    }

    #[test]
    fn rollback_flips_partition_and_reports_version() {
        let mut mgr = upgraded_manager();
        let mut h = InMemoryBootHistory::new();
        h.record_upgrade("v1.7.0");

        let outcome = {
            let mut ctrl = RollbackController::new(&mut mgr, &mut h);
            ctrl.rollback().unwrap()
        };
        assert_eq!(outcome.active, PartitionLabel::A);
        assert_eq!(outcome.version, "v1.7.0");
        assert_eq!(mgr.active(), PartitionLabel::A);
        // Marker consumed.
        assert!(!h.can_rollback());
    }

    #[test]
    fn rollback_fails_without_previous_version() {
        let mut mgr = upgraded_manager();
        let mut h = InMemoryBootHistory::new(); // no marker
        let mut ctrl = RollbackController::new(&mut mgr, &mut h);
        assert!(!ctrl.is_available());
        assert_eq!(ctrl.rollback(), Err(RollbackError::NoPreviousVersion));
    }

    #[test]
    fn rollback_fails_when_previous_partition_overwritten() {
        // Fresh install, B never populated -> nothing to roll back to.
        let mut mgr = InMemoryBootManager::new(PartitionLabel::A, "v1.7.0");
        let mut h = InMemoryBootHistory::new();
        h.record_upgrade("v1.6.0");
        let mut ctrl = RollbackController::new(&mut mgr, &mut h);
        assert!(!ctrl.is_available());
        assert_eq!(
            ctrl.rollback(),
            Err(RollbackError::PreviousPartitionEmpty(PartitionLabel::B))
        );
    }

    #[test]
    fn rollback_not_available_after_commit() {
        let mut mgr = upgraded_manager();
        let mut h = InMemoryBootHistory::new();
        h.record_upgrade("v1.7.0");
        h.commit(); // new version committed; rollback window closed
        let ctrl = RollbackController::new(&mut mgr, &mut h);
        assert!(!ctrl.is_available());
    }
}
