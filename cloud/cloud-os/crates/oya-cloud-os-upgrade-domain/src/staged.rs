//! Staged upgrades via the META partition.
//!
//! Talos supports two install modes when applying an OS upgrade:
//!
//! * **Immediate** — the installer runs in the current boot, writes the new
//!   system partition, and the node reboots into it.
//! * **Staged** — the upgrade *image reference* and *install options* are
//!   written to the META partition under the `StagedUpgradeImageRef` /
//!   `StagedUpgradeInstallOptions` keys. The node then reboots into the
//!   maintenance/install flow, which reads META, performs the install, clears
//!   the staged keys, and reboots again into the freshly installed system.
//!
//! This module models the META keys the staging flow uses and the small state
//! machine that writes, consumes, and clears them. The persistence boundary is
//! the [`MetaStore`] trait; [`InMemoryMetaStore`] is the deterministic test
//! implementation.

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use core::fmt;

/// The META key tags Talos reserves for the staged-upgrade flow.
///
/// These mirror the `pkg/machinery/meta` constants used by
/// `internal/pkg/meta`. Only the keys the staging flow touches are modeled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum StagedMetaKey {
    /// `StagedUpgradeImageRef` (tag 0x08): the installer image to run.
    UpgradeImageRef = 0x08,
    /// `StagedUpgradeInstallOptions` (tag 0x09): serialized install options.
    UpgradeInstallOptions = 0x09,
    /// `Upgrade` (tag 0x06): the previous OS version, written so a failed boot
    /// can roll back.
    PreviousVersion = 0x06,
}

impl StagedMetaKey {
    /// The numeric tag used on the META partition wire format.
    pub const fn tag(self) -> u8 {
        self as u8
    }

    /// Stable human-readable name.
    pub const fn name(self) -> &'static str {
        match self {
            StagedMetaKey::UpgradeImageRef => "StagedUpgradeImageRef",
            StagedMetaKey::UpgradeInstallOptions => "StagedUpgradeInstallOptions",
            StagedMetaKey::PreviousVersion => "Upgrade",
        }
    }
}

/// Errors raised by the staged-upgrade flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StagedUpgradeError {
    /// The image reference was empty or malformed.
    InvalidImageRef(String),
    /// A staged upgrade was requested while one is already pending.
    AlreadyStaged,
    /// The flow tried to consume staged state but none was present.
    NotStaged,
    /// The underlying META store rejected an operation.
    Store(String),
}

impl fmt::Display for StagedUpgradeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StagedUpgradeError::InvalidImageRef(m) => write!(f, "invalid image ref: {m}"),
            StagedUpgradeError::AlreadyStaged => write!(f, "an upgrade is already staged"),
            StagedUpgradeError::NotStaged => write!(f, "no upgrade is staged"),
            StagedUpgradeError::Store(m) => write!(f, "meta store error: {m}"),
        }
    }
}

/// Persistence boundary for the META partition's staged-upgrade keys.
pub trait MetaStore {
    /// Read the raw string value for a key, if present.
    fn get(&self, key: StagedMetaKey) -> Option<String>;

    /// Write (or replace) the value for a key.
    fn set(&mut self, key: StagedMetaKey, value: &str);

    /// Delete a key, returning whether it was present.
    fn delete(&mut self, key: StagedMetaKey) -> bool;
}

/// An in-memory [`MetaStore`] used by tests, backed by a sorted map keyed by
/// the META tag. A `flushes` counter tracks how many mutating syncs happened so
/// tests can assert the META partition is only rewritten when needed.
#[derive(Debug, Default, Clone)]
pub struct InMemoryMetaStore {
    values: BTreeMap<u8, String>,
    flushes: u32,
}

impl InMemoryMetaStore {
    /// An empty store.
    pub fn new() -> Self {
        InMemoryMetaStore {
            values: BTreeMap::new(),
            flushes: 0,
        }
    }

    /// Number of times a mutating operation flushed to the partition.
    pub fn flushes(&self) -> u32 {
        self.flushes
    }

    /// Number of keys currently set.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether the store holds no keys.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl MetaStore for InMemoryMetaStore {
    fn get(&self, key: StagedMetaKey) -> Option<String> {
        self.values.get(&key.tag()).cloned()
    }

    fn set(&mut self, key: StagedMetaKey, value: &str) {
        self.values.insert(key.tag(), value.to_string());
        self.flushes += 1;
    }

    fn delete(&mut self, key: StagedMetaKey) -> bool {
        let removed = self.values.remove(&key.tag()).is_some();
        if removed {
            self.flushes += 1;
        }
        removed
    }
}

/// The state of the staging machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StagingState {
    /// No staged upgrade is pending.
    Idle,
    /// Image ref + options written to META, awaiting the install-on-boot.
    Staged,
    /// The staged install has been consumed; keys cleared.
    Consumed,
}

/// A reified staged-upgrade request, recovered from META on the install boot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedUpgrade {
    /// Installer image reference to run.
    pub image_ref: String,
    /// Serialized install options (e.g. `--preserve --stage`).
    pub install_options: String,
    /// Previous OS version, for rollback bookkeeping.
    pub previous_version: Option<String>,
}

/// Drives staging into / out of a [`MetaStore`].
#[derive(Debug)]
pub struct StagedUpgradeController<'a, S: MetaStore> {
    store: &'a mut S,
    state: StagingState,
}

impl<'a, S: MetaStore> StagedUpgradeController<'a, S> {
    /// Wrap a META store, deriving the initial state from its current contents.
    pub fn new(store: &'a mut S) -> Self {
        let state = if store.get(StagedMetaKey::UpgradeImageRef).is_some() {
            StagingState::Staged
        } else {
            StagingState::Idle
        };
        StagedUpgradeController { store, state }
    }

    /// The current state.
    pub fn state(&self) -> StagingState {
        self.state
    }

    /// Whether an upgrade is currently staged.
    pub fn is_staged(&self) -> bool {
        self.state == StagingState::Staged
    }

    /// Stage an upgrade: validate and write the META keys.
    ///
    /// Refuses to overwrite an already-staged upgrade.
    pub fn stage(
        &mut self,
        image_ref: &str,
        install_options: &str,
        previous_version: Option<&str>,
    ) -> Result<(), StagedUpgradeError> {
        if self.state == StagingState::Staged {
            return Err(StagedUpgradeError::AlreadyStaged);
        }
        validate_image_ref(image_ref)?;

        self.store.set(StagedMetaKey::UpgradeImageRef, image_ref);
        self.store
            .set(StagedMetaKey::UpgradeInstallOptions, install_options);
        if let Some(prev) = previous_version {
            self.store.set(StagedMetaKey::PreviousVersion, prev);
        }
        self.state = StagingState::Staged;
        Ok(())
    }

    /// Recover the staged upgrade from META without consuming it (the
    /// install-on-boot path reads this first).
    pub fn peek(&self) -> Result<StagedUpgrade, StagedUpgradeError> {
        let image_ref = self
            .store
            .get(StagedMetaKey::UpgradeImageRef)
            .ok_or(StagedUpgradeError::NotStaged)?;
        let install_options = self
            .store
            .get(StagedMetaKey::UpgradeInstallOptions)
            .unwrap_or_default();
        let previous_version = self.store.get(StagedMetaKey::PreviousVersion);
        Ok(StagedUpgrade {
            image_ref,
            install_options,
            previous_version,
        })
    }

    /// Consume the staged upgrade: read it and clear the keys so the next boot
    /// does not re-run the install. The `PreviousVersion` key is intentionally
    /// retained for rollback.
    pub fn consume(&mut self) -> Result<StagedUpgrade, StagedUpgradeError> {
        let staged = self.peek()?;
        self.store.delete(StagedMetaKey::UpgradeImageRef);
        self.store.delete(StagedMetaKey::UpgradeInstallOptions);
        self.state = StagingState::Consumed;
        Ok(staged)
    }

    /// Abort a staged upgrade, clearing the install keys without performing it.
    pub fn cancel(&mut self) -> bool {
        let had = self.store.get(StagedMetaKey::UpgradeImageRef).is_some();
        self.store.delete(StagedMetaKey::UpgradeImageRef);
        self.store.delete(StagedMetaKey::UpgradeInstallOptions);
        self.state = StagingState::Idle;
        had
    }
}

/// Validate an installer image reference. Talos requires a non-empty,
/// `registry/repo[:tag]` style reference with no spaces.
pub fn validate_image_ref(image_ref: &str) -> Result<(), StagedUpgradeError> {
    let trimmed = image_ref.trim();
    if trimmed.is_empty() {
        return Err(StagedUpgradeError::InvalidImageRef("empty".to_string()));
    }
    if trimmed.contains(char::is_whitespace) {
        return Err(StagedUpgradeError::InvalidImageRef(
            "contains whitespace".to_string(),
        ));
    }
    if !trimmed.contains('/') {
        return Err(StagedUpgradeError::InvalidImageRef(
            "missing registry/repo separator".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const IMAGE: &str = "ghcr.io/siderolabs/installer:v1.8.0";

    #[test]
    fn image_ref_validation() {
        assert!(validate_image_ref(IMAGE).is_ok());
        assert!(validate_image_ref("").is_err());
        assert!(validate_image_ref("   ").is_err());
        assert!(validate_image_ref("installer v1.8.0").is_err());
        assert!(validate_image_ref("installer").is_err());
    }

    #[test]
    fn meta_keys_have_stable_tags() {
        assert_eq!(StagedMetaKey::UpgradeImageRef.tag(), 0x08);
        assert_eq!(StagedMetaKey::UpgradeInstallOptions.tag(), 0x09);
        assert_eq!(StagedMetaKey::PreviousVersion.tag(), 0x06);
        assert_eq!(StagedMetaKey::PreviousVersion.name(), "Upgrade");
    }

    #[test]
    fn stage_writes_meta_and_transitions() {
        let mut store = InMemoryMetaStore::new();
        let mut ctrl = StagedUpgradeController::new(&mut store);
        assert_eq!(ctrl.state(), StagingState::Idle);

        ctrl.stage(IMAGE, "--preserve", Some("v1.7.0")).unwrap();
        assert!(ctrl.is_staged());
        assert_eq!(ctrl.state(), StagingState::Staged);

        let staged = ctrl.peek().unwrap();
        assert_eq!(staged.image_ref, IMAGE);
        assert_eq!(staged.install_options, "--preserve");
        assert_eq!(staged.previous_version.as_deref(), Some("v1.7.0"));
    }

    #[test]
    fn cannot_stage_twice() {
        let mut store = InMemoryMetaStore::new();
        let mut ctrl = StagedUpgradeController::new(&mut store);
        ctrl.stage(IMAGE, "", None).unwrap();
        assert_eq!(
            ctrl.stage(IMAGE, "", None),
            Err(StagedUpgradeError::AlreadyStaged)
        );
    }

    #[test]
    fn invalid_image_rejected_and_not_persisted() {
        let mut store = InMemoryMetaStore::new();
        let mut ctrl = StagedUpgradeController::new(&mut store);
        assert!(ctrl.stage("bad ref", "", None).is_err());
        assert_eq!(ctrl.state(), StagingState::Idle);
        assert!(store.is_empty());
    }

    #[test]
    fn consume_clears_install_keys_but_keeps_previous_version() {
        let mut store = InMemoryMetaStore::new();
        {
            let mut ctrl = StagedUpgradeController::new(&mut store);
            ctrl.stage(IMAGE, "--stage", Some("v1.7.3")).unwrap();
        }
        // Simulate a reboot: a fresh controller derives state from META.
        let mut ctrl = StagedUpgradeController::new(&mut store);
        assert!(ctrl.is_staged());

        let staged = ctrl.consume().unwrap();
        assert_eq!(staged.image_ref, IMAGE);
        assert_eq!(ctrl.state(), StagingState::Consumed);

        assert!(store.get(StagedMetaKey::UpgradeImageRef).is_none());
        assert!(store.get(StagedMetaKey::UpgradeInstallOptions).is_none());
        // Rollback marker survives.
        assert_eq!(
            store.get(StagedMetaKey::PreviousVersion).as_deref(),
            Some("v1.7.3")
        );
    }

    #[test]
    fn peek_and_consume_error_when_not_staged() {
        let mut store = InMemoryMetaStore::new();
        let mut ctrl = StagedUpgradeController::new(&mut store);
        assert_eq!(ctrl.peek(), Err(StagedUpgradeError::NotStaged));
        assert_eq!(ctrl.consume(), Err(StagedUpgradeError::NotStaged));
    }

    #[test]
    fn cancel_clears_staging() {
        let mut store = InMemoryMetaStore::new();
        let mut ctrl = StagedUpgradeController::new(&mut store);
        ctrl.stage(IMAGE, "", None).unwrap();
        assert!(ctrl.cancel());
        assert_eq!(ctrl.state(), StagingState::Idle);
        assert!(!ctrl.cancel()); // nothing left to cancel
    }

    #[test]
    fn flush_counter_tracks_partition_writes() {
        let mut store = InMemoryMetaStore::new();
        {
            let mut ctrl = StagedUpgradeController::new(&mut store);
            ctrl.stage(IMAGE, "--preserve", Some("v1.7.0")).unwrap();
        }
        // 3 sets: image ref, options, previous version.
        assert_eq!(store.flushes(), 3);
    }
}
