//! The volume manager controller.
//!
//! Mirrors `internal/app/machined/pkg/controllers/block/volume_manager.go`: the
//! controller that holds the set of declared [`VolumeConfig`]s (both the
//! built-in *system* volumes Talos always needs — EFI, META, STATE, EPHEMERAL —
//! plus optional source-managed system volumes such as IMAGECACHE, and *user*
//! volumes from machine config), reconciles each against the
//! discovered devices, and drives their [`VolumeStatus`] state machines to
//! `Ready`. The actual provisioning side-effects (partitioning, mkfs, LUKS
//! open) are represented by the discovery + status layers already in the crate;
//! this module is the orchestration glue and the user/system classification.

use std::collections::{BTreeMap, BTreeSet};

use crate::cryptsetup::{
    CryptsetupBackend, CryptsetupCommand, LuksAddKeyRequest, LuksCloseRequest, LuksFormatRequest,
    LuksOpenRequest, MemCryptsetupBackend, mapper_name, mapper_path,
};
use crate::discovery::{DiscoveredVolume, Discoverer};
use crate::disk::Disk;
use crate::encryption::{Cipher, EncryptionConfig};
use crate::filesystem::FilesystemType;
use crate::gpt::{GptTable, PartitionEntry};
use crate::layout::{apply_partition_create_plan, apply_partition_growth_plan};
use crate::luks::Luks2Header;
use crate::mount::{
    MountPhase, MountStatus, Mounter, VolumeMountStatusResource, reconcile_unmount,
};
use crate::partition::{Partition, PartitionRole};
use crate::volume::{VolumeConfig, VolumePhase, VolumeStatus, VolumeType};
use crate::{BlockError, Result};

const USER_VOLUME_PRIORITY_BASE: u32 = 100;
const IMAGE_CACHE_SYSTEM_PRIORITY: u32 = 4;

/// Upstream Talos image-cache system volume id and partition label.
///
/// Source: Talos v1.13.0 `constants.ImageCachePartitionLabel` and
/// `ImageCacheConfigController.VolumeImageCacheDISK`.
pub const IMAGE_CACHE_VOLUME_ID: &str = "IMAGECACHE";

/// Talos mount controller finalizer name.
///
/// Source: `MountController.Name()` returns `block.MountController`, and that
/// finalizer is attached to `VolumeStatus` while a mount resource exists.
pub const MOUNT_CONTROLLER_FINALIZER: &str = "block.MountController";

/// Whether a volume is a built-in Talos system volume or a user-declared one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeClass {
    /// A Talos-managed system volume (core EFI/META/STATE/EPHEMERAL or
    /// optional controller-provisioned volumes such as IMAGECACHE).
    System,
    /// A user volume declared in machine config.
    User,
}

/// A managed volume: its config, classification, and a provisioning priority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedVolume {
    /// The declarative config.
    pub config: VolumeConfig,
    /// System vs user.
    pub class: VolumeClass,
    /// Lower numbers are provisioned first; system volumes default lower.
    pub priority: u32,
}

/// Injectable provisioning boundary for volume reconciliation.
///
/// The default manager path remains side-effect free. Tests and future runtime
/// adapters can provide a provisioner that creates or grows a missing
/// partition-backed volume, returning the discovered device that reconciliation
/// should then drive through the status machine.
pub trait VolumeProvisioner {
    /// Provision or update `config` based on the current discovery result.
    fn provision(
        &mut self,
        config: &VolumeConfig,
        discovered: Option<&DiscoveredVolume>,
        disc: &Discoverer,
    ) -> Result<Option<DiscoveredVolume>>;

    /// Whether the last provisioning error should be retried.
    fn failure_is_retryable(&self, _error: &BlockError) -> bool {
        false
    }

    /// Host-safe phase where the last retryable provisioning failure occurred.
    fn failure_phase(&self, _error: &BlockError) -> Option<ProvisioningPhase> {
        None
    }
}

/// Side-effect-free provisioner used by [`VolumeManager::reconcile`].
#[derive(Debug, Default)]
pub struct NoopVolumeProvisioner;

impl VolumeProvisioner for NoopVolumeProvisioner {
    fn provision(
        &mut self,
        _config: &VolumeConfig,
        discovered: Option<&DiscoveredVolume>,
        _disc: &Discoverer,
    ) -> Result<Option<DiscoveredVolume>> {
        Ok(discovered.cloned())
    }
}

/// Host-safe provisioning phase boundary reached by a provisioner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProvisioningPhase {
    /// An already-discovered partition satisfied the declaration.
    LocateExisting,
    /// A partition created by an earlier attempt is reused for retry.
    ReuseCreatedPartition,
    /// The selected parent disk is locked before table mutation.
    LockParent,
    /// GPT partition creation was attempted.
    CreatePartition,
    /// GPT partition growth was attempted.
    GrowPartition,
    /// The target partition wipe phase was attempted.
    WipePartition,
    /// LUKS2 formatting/key enrollment was attempted.
    FormatEncryptedVolume,
    /// The encrypted volume open phase was attempted.
    OpenEncryptedVolume,
    /// The encrypted volume unmount phase was attempted before mapper close.
    UnmountEncryptedVolume,
    /// The encrypted volume close phase was attempted.
    CloseEncryptedVolume,
    /// The filesystem format phase was attempted.
    Format,
}

/// Ordered host-safe record of provisioning phase boundaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProvisioningEvent {
    /// An existing discovered partition was selected.
    LocateExisting {
        /// Volume id.
        id: String,
        /// Device path, e.g. `/dev/sda4`.
        dev_path: String,
    },
    /// A previously-created in-memory partition was reused by a retry.
    ReuseCreatedPartition {
        /// Volume id.
        id: String,
        /// Parent disk device name, e.g. `sda`.
        parent: String,
        /// Device path, e.g. `/dev/sda1`.
        dev_path: String,
    },
    /// Parent disk lock phase reached.
    LockParent {
        /// Volume id.
        id: String,
        /// Parent disk device name, e.g. `sda`.
        parent: String,
    },
    /// GPT partition creation phase reached.
    CreatePartition {
        /// Volume id.
        id: String,
        /// Parent disk device name, e.g. `sda`.
        parent: String,
        /// Created partition device path.
        dev_path: String,
        /// Created partition GUID.
        part_guid: String,
        /// Created partition size in bytes.
        size: u64,
    },
    /// GPT partition growth phase reached.
    GrowPartition {
        /// Volume id.
        id: String,
        /// Parent disk device name, e.g. `sda`.
        parent: String,
        /// Grown partition device path.
        dev_path: String,
        /// Size before growth in bytes.
        from_size: u64,
        /// Size after growth in bytes.
        to_size: u64,
    },
    /// Partition wipe phase reached.
    WipePartition {
        /// Volume id.
        id: String,
        /// Device path.
        dev_path: String,
    },
    /// LUKS2 format/key enrollment phase reached.
    FormatEncryptedVolume {
        /// Volume id.
        id: String,
        /// Device path.
        dev_path: String,
        /// First keyslot used to seed the LUKS2 header.
        key_slot: u8,
        /// Cipher configured for the encrypted volume.
        cipher: Cipher,
    },
    /// Encrypted volume open phase reached.
    OpenEncryptedVolume {
        /// Volume id.
        id: String,
        /// Device path.
        dev_path: String,
        /// Canonical device-mapper path used after opening LUKS2.
        mapped_path: String,
        /// LUKS keyslot selected by the configured key providers.
        key_slot: u8,
        /// Cipher configured for the encrypted volume.
        cipher: Cipher,
    },
    /// Encrypted volume unmount phase reached before mapper close.
    UnmountEncryptedVolume {
        /// Volume id.
        id: String,
        /// Canonical mapped path, e.g. `/dev/mapper/luks2-STATE`.
        mapped_path: String,
        /// Filesystem mount target to unmount.
        target: String,
    },
    /// Encrypted volume close phase reached.
    CloseEncryptedVolume {
        /// Volume id.
        id: String,
        /// Device-mapper name, e.g. `luks2-STATE`.
        mapper_name: String,
        /// Canonical mapped path, e.g. `/dev/mapper/luks2-STATE`.
        mapped_path: String,
    },
    /// Filesystem format phase reached.
    Format {
        /// Volume id.
        id: String,
        /// Device path.
        dev_path: String,
        /// Filesystem type to create.
        filesystem: FilesystemType,
    },
}

impl ProvisioningEvent {
    /// The phase represented by this event.
    pub fn phase(&self) -> ProvisioningPhase {
        match self {
            ProvisioningEvent::LocateExisting { .. } => ProvisioningPhase::LocateExisting,
            ProvisioningEvent::ReuseCreatedPartition { .. } => {
                ProvisioningPhase::ReuseCreatedPartition
            }
            ProvisioningEvent::LockParent { .. } => ProvisioningPhase::LockParent,
            ProvisioningEvent::CreatePartition { .. } => ProvisioningPhase::CreatePartition,
            ProvisioningEvent::GrowPartition { .. } => ProvisioningPhase::GrowPartition,
            ProvisioningEvent::WipePartition { .. } => ProvisioningPhase::WipePartition,
            ProvisioningEvent::FormatEncryptedVolume { .. } => {
                ProvisioningPhase::FormatEncryptedVolume
            }
            ProvisioningEvent::OpenEncryptedVolume { .. } => ProvisioningPhase::OpenEncryptedVolume,
            ProvisioningEvent::UnmountEncryptedVolume { .. } => {
                ProvisioningPhase::UnmountEncryptedVolume
            }
            ProvisioningEvent::CloseEncryptedVolume { .. } => {
                ProvisioningPhase::CloseEncryptedVolume
            }
            ProvisioningEvent::Format { .. } => ProvisioningPhase::Format,
        }
    }
}

/// Reason a tearing-down volume is not yet eligible for provider close.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VolumeCloseBlocker {
    /// The volume is not in the resource teardown path yet.
    NotTearingDown,
    /// A controller finalizer still blocks volume close.
    VolumeFinalizer {
        /// Finalizer name.
        name: String,
    },
    /// The mount status resource still exists, so close must wait even if the
    /// underlying mount point has already been unmounted.
    MountStatusPresent {
        /// Mount target carried by the resource.
        target: String,
    },
    /// A child volume-mount-status resource still exists, so close must wait
    /// until the requester releases its finalizer and the parent status drains.
    VolumeMountStatusPresent {
        /// Child status resource id.
        id: String,
        /// Mount target carried by the child status resource.
        target: String,
    },
}

/// Result of a source-guided close-eligibility pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VolumeCloseOutcome {
    /// Close was deferred because a teardown blocker remains.
    Deferred(VolumeCloseBlocker),
    /// The volume is unencrypted and reached `Closed` without cryptsetup.
    NotEncrypted,
    /// The volume was already closed before this pass.
    AlreadyClosed,
    /// The encrypted mapper was closed in this pass.
    Closed,
}

/// Host-safe model of Talos's volume close gate.
///
/// Talos v1.13.0 computes close eligibility as `tearingDown &&
/// VolumeStatus.finalizers.Empty()`: mount-controller finalizers are removed
/// only after unmount succeeds. This guard also keeps an explicit
/// mount-status-resource bit so tests can prove close remains deferred until
/// the status resource itself is gone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeCloseEligibility {
    tearing_down: bool,
    volume_finalizers: BTreeSet<String>,
    mount_status_target: Option<String>,
    volume_mount_status: Option<(String, String)>,
}

impl VolumeCloseEligibility {
    /// Build an eligibility guard for a live resource; close should defer.
    pub fn running() -> Self {
        VolumeCloseEligibility {
            tearing_down: false,
            volume_finalizers: BTreeSet::new(),
            mount_status_target: None,
            volume_mount_status: None,
        }
    }

    /// Build an eligibility guard for a tearing-down volume with no blockers.
    pub fn tearing_down() -> Self {
        VolumeCloseEligibility {
            tearing_down: true,
            volume_finalizers: BTreeSet::new(),
            mount_status_target: None,
            volume_mount_status: None,
        }
    }

    /// Add a volume-status finalizer blocker.
    pub fn with_volume_finalizer(mut self, name: impl Into<String>) -> Self {
        self.volume_finalizers.insert(name.into());
        self
    }

    /// Record that a mount-status resource still exists.
    pub fn with_mount_status(mut self, mount: &MountStatus) -> Self {
        self.mount_status_target = Some(mount.spec.target.clone());
        self
    }

    /// Record that a mount-status resource still exists by target.
    pub fn with_mount_status_target(mut self, target: impl Into<String>) -> Self {
        self.mount_status_target = Some(target.into());
        self
    }

    /// Record that a child volume-mount-status resource still exists.
    pub fn with_volume_mount_status(mut self, status: &VolumeMountStatusResource) -> Self {
        self.volume_mount_status = Some((
            status.metadata().id().as_str().to_string(),
            status.spec.target.clone(),
        ));
        self
    }

    /// Record that a child volume-mount-status resource still exists by id and
    /// target.
    pub fn with_volume_mount_status_id(
        mut self,
        id: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        self.volume_mount_status = Some((id.into(), target.into()));
        self
    }

    /// Whether this guard contains `name` as a volume finalizer.
    pub fn has_volume_finalizer(&self, name: &str) -> bool {
        self.volume_finalizers.contains(name)
    }

    /// First close blocker, if any.
    pub fn blocker(&self) -> Option<VolumeCloseBlocker> {
        if !self.tearing_down {
            return Some(VolumeCloseBlocker::NotTearingDown);
        }
        if let Some(name) = self.volume_finalizers.iter().next() {
            return Some(VolumeCloseBlocker::VolumeFinalizer { name: name.clone() });
        }
        if let Some((id, target)) = &self.volume_mount_status {
            return Some(VolumeCloseBlocker::VolumeMountStatusPresent {
                id: id.clone(),
                target: target.clone(),
            });
        }
        if let Some(target) = &self.mount_status_target {
            return Some(VolumeCloseBlocker::MountStatusPresent {
                target: target.clone(),
            });
        }
        None
    }

    /// Whether an encrypted provider close may run.
    pub fn can_close(&self) -> bool {
        self.blocker().is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MemGptDisk {
    table: GptTable,
    sector_size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CreatedPartitionState {
    parent: String,
    label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProvisioningFailure {
    phase: ProvisioningPhase,
    reason: String,
}

/// In-memory GPT provisioner for host-safe controller/runtime tests.
///
/// This is intentionally not a production disk writer: it mutates only
/// registered [`GptTable`] values and fails closed when the selected disk has no
/// table. Real disk writers can implement [`VolumeProvisioner`] without
/// changing manager reconciliation semantics. The event log exposes the same
/// coarse lock/create/wipe/grow/format phase boundaries that the Talos
/// controller reaches, while all effects stay inside registered memory tables.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MemGptProvisioner {
    tables: BTreeMap<String, MemGptDisk>,
    events: Vec<ProvisioningEvent>,
    created: BTreeMap<String, CreatedPartitionState>,
    wiped: BTreeMap<String, bool>,
    luks_headers: BTreeMap<String, Luks2Header>,
    cryptsetup: MemCryptsetupBackend,
    opened: BTreeMap<String, bool>,
    opened_slots: BTreeMap<String, u8>,
    opened_paths: BTreeMap<String, String>,
    formatted: BTreeMap<String, bool>,
    failure: Option<ProvisioningFailure>,
    last_failure_retryable: bool,
    last_failure_phase: Option<ProvisioningPhase>,
}

impl MemGptProvisioner {
    /// Register an in-memory GPT table for a disk device name such as `sda`.
    pub fn register_table(
        &mut self,
        dev_name: impl Into<String>,
        table: GptTable,
        sector_size: u64,
    ) -> Result<()> {
        let dev_name = dev_name.into();
        if dev_name.is_empty() {
            return Err(BlockError::InvalidDevice(
                "empty GPT table disk name".to_string(),
            ));
        }
        if sector_size == 0 {
            return Err(BlockError::Geometry("zero sector size".to_string()));
        }
        table.validate()?;
        self.tables
            .insert(dev_name, MemGptDisk { table, sector_size });
        Ok(())
    }

    /// Inspect a registered in-memory table after reconciliation.
    pub fn table(&self, dev_name: &str) -> Option<&GptTable> {
        self.tables.get(dev_name).map(|disk| &disk.table)
    }

    /// Ordered phase events emitted by this in-memory provisioner.
    pub fn events(&self) -> &[ProvisioningEvent] {
        &self.events
    }

    /// Inspect the host-safe LUKS2 header model for an encrypted volume.
    pub fn luks_header(&self, id: &str) -> Option<&Luks2Header> {
        self.luks_headers.get(id)
    }

    /// The keyslot that last opened an encrypted volume in memory.
    pub fn opened_key_slot(&self, id: &str) -> Option<u8> {
        self.opened_slots.get(id).copied()
    }

    /// The canonical device-mapper path last opened for an encrypted volume.
    pub fn opened_mapped_path(&self, id: &str) -> Option<&str> {
        self.opened_paths.get(id).map(String::as_str)
    }

    /// Recorded host-safe cryptsetup command intents.
    pub fn cryptsetup_commands(&self) -> &[CryptsetupCommand] {
        self.cryptsetup.commands()
    }

    /// Whether the dm-crypt mapper for `id` is currently open in the host-safe
    /// backend.
    pub fn is_mapping_open(&self, id: &str) -> bool {
        mapper_name(id)
            .ok()
            .and_then(|name| self.cryptsetup.opened(&name).map(|_| ()))
            .is_some()
    }

    /// Close the dm-crypt mapper for an encrypted volume.
    ///
    /// This is a teardown boundary for tests and future runtime adapters. It
    /// records the same close intent a Linux backend would send to cryptsetup
    /// while keeping the host state untouched.
    pub fn close_encrypted_volume(&mut self, id: &str) -> Result<()> {
        let mapper_name = mapper_name(id)?;
        let mapped_path =
            self.opened_paths.get(id).cloned().ok_or_else(|| {
                BlockError::NotFound(format!("encrypted volume {id} is not open"))
            })?;
        self.events.push(ProvisioningEvent::CloseEncryptedVolume {
            id: id.to_string(),
            mapper_name: mapper_name.clone(),
            mapped_path,
        });
        self.maybe_fail(ProvisioningPhase::CloseEncryptedVolume)?;
        self.cryptsetup.close(LuksCloseRequest { mapper_name })?;
        self.opened.remove(id);
        self.opened_slots.remove(id);
        self.opened_paths.remove(id);
        Ok(())
    }

    /// Unmount a mounted encrypted volume and then close its dm-crypt mapper.
    ///
    /// Talos tears down the filesystem mount in the mount controller before the
    /// volume-manager close path is allowed to call the LUKS provider. This
    /// helper models that cross-controller choreography without touching host
    /// mounts or device-mapper state.
    pub fn unmount_and_close_encrypted_volume<M: Mounter>(
        &mut self,
        id: &str,
        mount: &mut MountStatus,
        mounter: &mut M,
    ) -> Result<()> {
        let mapper_name = mapper_name(id)?;
        let expected_mapped_path = match self.opened_paths.get(id) {
            Some(path) => path.clone(),
            None => mapper_path(&mapper_name)?,
        };
        if mount.spec.source != expected_mapped_path {
            return Err(BlockError::InvalidDevice(format!(
                "mount source {} does not match encrypted mapping {} for {id}",
                mount.spec.source, expected_mapped_path
            )));
        }
        if mount.phase != MountPhase::Unmounted || mounter.is_mounted(&mount.spec.target) {
            self.events.push(ProvisioningEvent::UnmountEncryptedVolume {
                id: id.to_string(),
                mapped_path: expected_mapped_path,
                target: mount.spec.target.clone(),
            });
            self.maybe_fail(ProvisioningPhase::UnmountEncryptedVolume)?;
            reconcile_unmount(mount, mounter)?;
        }
        if !self.opened_paths.contains_key(id) {
            return Ok(());
        }
        self.close_encrypted_volume(id)
    }

    /// Close a volume only after Talos-style teardown eligibility is satisfied.
    ///
    /// This models the volume-manager gate that waits for teardown and an empty
    /// finalizer set before invoking provider close. A remaining mount finalizer
    /// or mount-status resource defers close without touching the mapper. Close
    /// failures are retryable from the pre-close phase, mirroring Talos's
    /// `PreFailPhase` restore path.
    pub fn close_volume_when_eligible(
        &mut self,
        status: &mut VolumeStatus,
        eligibility: &VolumeCloseEligibility,
    ) -> Result<VolumeCloseOutcome> {
        if let Some(blocker) = eligibility.blocker() {
            return Ok(VolumeCloseOutcome::Deferred(blocker));
        }
        if status.phase == VolumePhase::Failed && status.retryable {
            let _ = status.restore_retry();
        }
        if status.phase == VolumePhase::Closed {
            return Ok(VolumeCloseOutcome::AlreadyClosed);
        }
        if !status.config.is_encrypted() {
            status.make_closed()?;
            return Ok(VolumeCloseOutcome::NotEncrypted);
        }

        let id = status.config.id.clone();
        if !self.opened_paths.contains_key(&id) {
            status.make_closed()?;
            return Ok(VolumeCloseOutcome::AlreadyClosed);
        }

        let pre_close_phase = status.phase;
        match self.close_encrypted_volume(&id) {
            Ok(()) => {
                status.make_closed()?;
                Ok(VolumeCloseOutcome::Closed)
            }
            Err(error) => {
                let message = error.to_string();
                let _ = status.fail_retryable_from(pre_close_phase, message);
                Err(error)
            }
        }
    }

    /// Clear the phase event log without changing in-memory disk state.
    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    /// Inject a one-shot retryable failure at a host-safe phase boundary.
    pub fn fail_next_phase(&mut self, phase: ProvisioningPhase, reason: impl Into<String>) {
        self.failure = Some(ProvisioningFailure {
            phase,
            reason: reason.into(),
        });
    }

    /// Remove any pending injected failure.
    pub fn clear_failure(&mut self) {
        self.failure = None;
        self.last_failure_retryable = false;
        self.last_failure_phase = None;
    }

    fn maybe_fail(&mut self, phase: ProvisioningPhase) -> Result<()> {
        let Some(failure) = self.failure.clone() else {
            return Ok(());
        };
        if failure.phase != phase {
            return Ok(());
        }
        self.failure = None;
        self.last_failure_retryable = true;
        self.last_failure_phase = Some(phase);
        Err(BlockError::InvalidDevice(failure.reason))
    }

    fn ensure_wiped(&mut self, config: &VolumeConfig, found: &DiscoveredVolume) -> Result<()> {
        if self.wiped.get(&config.id).copied().unwrap_or(false) {
            return Ok(());
        }
        self.events.push(ProvisioningEvent::WipePartition {
            id: config.id.clone(),
            dev_path: found.dev_path.clone(),
        });
        self.maybe_fail(ProvisioningPhase::WipePartition)?;
        self.wiped.insert(config.id.clone(), true);
        Ok(())
    }

    fn ensure_opened(&mut self, config: &VolumeConfig, found: &DiscoveredVolume) -> Result<()> {
        let Some(encryption) = config.encryption.as_ref() else {
            return Ok(());
        };
        if self.opened.get(&config.id).copied().unwrap_or(false) {
            return Ok(());
        }
        let derived_keys = Self::derived_encryption_keys(encryption)?;
        let (configured_slot, passphrase) = derived_keys
            .first()
            .cloned()
            .ok_or_else(|| BlockError::KeyFailure("no key provider resolved".to_string()))?;
        if !self.luks_headers.contains_key(&config.id) {
            self.events.push(ProvisioningEvent::FormatEncryptedVolume {
                id: config.id.clone(),
                dev_path: found.dev_path.clone(),
                key_slot: configured_slot,
                cipher: encryption.cipher,
            });
            self.maybe_fail(ProvisioningPhase::FormatEncryptedVolume)?;
            self.cryptsetup.format(LuksFormatRequest {
                dev_path: found.dev_path.clone(),
                uuid: format!("{}:{}", config.id, found.dev_path),
                cipher: encryption.cipher,
                key_slot: configured_slot,
                passphrase: passphrase.clone(),
            })?;
            for (slot, new_passphrase) in derived_keys.iter().skip(1) {
                self.cryptsetup.add_key(LuksAddKeyRequest {
                    dev_path: found.dev_path.clone(),
                    existing_passphrase: passphrase.clone(),
                    new_key_slot: *slot,
                    new_passphrase: new_passphrase.clone(),
                })?;
            }
            let header = self
                .cryptsetup
                .header(&found.dev_path)
                .cloned()
                .ok_or_else(|| {
                    BlockError::KeyFailure(format!(
                        "formatted LUKS2 header for {} was not recorded",
                        found.dev_path
                    ))
                })?;
            self.luks_headers.insert(config.id.clone(), header);
        }
        let key_slot = {
            let header = self.luks_headers.get(&config.id).ok_or_else(|| {
                BlockError::KeyFailure(format!("LUKS header for {} not found", config.id))
            })?;
            if !header.is_valid() {
                return Err(BlockError::KeyFailure(format!(
                    "invalid LUKS2 header for {}",
                    config.id
                )));
            }
            if header.cipher != encryption.cipher {
                return Err(BlockError::KeyFailure(format!(
                    "LUKS cipher mismatch for {}: header={} config={}",
                    config.id,
                    header.cipher.as_str(),
                    encryption.cipher.as_str()
                )));
            }
            if self.cryptsetup.header(&found.dev_path).is_none() {
                self.cryptsetup
                    .insert_header(found.dev_path.clone(), header.clone())?;
            }
            header.open(&passphrase)?
        };
        let mapper_name = mapper_name(&config.id)?;
        let mapped_path = crate::cryptsetup::mapper_path(&mapper_name)?;
        self.events.push(ProvisioningEvent::OpenEncryptedVolume {
            id: config.id.clone(),
            dev_path: found.dev_path.clone(),
            mapped_path: mapped_path.clone(),
            key_slot,
            cipher: encryption.cipher,
        });
        self.maybe_fail(ProvisioningPhase::OpenEncryptedVolume)?;
        let opened = self.cryptsetup.open(LuksOpenRequest {
            dev_path: found.dev_path.clone(),
            mapper_name,
            key_slot,
            passphrase,
        })?;
        self.opened_slots.insert(config.id.clone(), key_slot);
        self.opened_paths
            .insert(config.id.clone(), opened.mapped_path);
        self.opened.insert(config.id.clone(), true);
        Ok(())
    }

    fn ensure_formatted(&mut self, config: &VolumeConfig, found: &DiscoveredVolume) -> Result<()> {
        let Some(filesystem) = config.filesystem else {
            return Ok(());
        };
        if self.formatted.get(&config.id).copied().unwrap_or(false) {
            return Ok(());
        }
        let dev_path = self
            .opened_paths
            .get(&config.id)
            .cloned()
            .unwrap_or_else(|| found.dev_path.clone());
        self.events.push(ProvisioningEvent::Format {
            id: config.id.clone(),
            dev_path,
            filesystem,
        });
        self.maybe_fail(ProvisioningPhase::Format)?;
        self.formatted.insert(config.id.clone(), true);
        Ok(())
    }

    fn ensure_create_post_steps(
        &mut self,
        config: &VolumeConfig,
        found: &DiscoveredVolume,
    ) -> Result<()> {
        self.ensure_wiped(config, found)?;
        self.ensure_opened(config, found)?;
        self.ensure_formatted(config, found)
    }

    fn discovered_from_entry(
        disk: &Disk,
        table: &GptTable,
        entry: &PartitionEntry,
        sector_size: u64,
        filesystem: Option<FilesystemType>,
    ) -> Result<DiscoveredVolume> {
        let index = table
            .entries
            .iter()
            .position(|candidate| {
                candidate.part_guid == entry.part_guid
                    && candidate.name.eq_ignore_ascii_case(&entry.name)
                    && candidate.first_lba == entry.first_lba
                    && candidate.last_lba == entry.last_lba
            })
            .ok_or_else(|| {
                BlockError::BadTable(format!(
                    "partition entry {} not present in registered GPT table",
                    entry.name
                ))
            })?;
        let number = u32::try_from(index + 1)
            .map_err(|_| BlockError::BadTable("partition index does not fit in u32".to_string()))?;
        let role = PartitionRole::from_label(&entry.name);
        let mut partition = Partition::new(
            disk.partition_name(number),
            number,
            entry.first_lba,
            entry.last_lba,
            role,
        );
        partition.sector_size = sector_size;
        partition.label = Some(entry.name.clone());
        partition.uuid = Some(entry.part_guid.clone());
        partition.filesystem = filesystem;
        partition.validate()?;
        Ok(DiscoveredVolume::from_partition(disk, &partition))
    }

    fn derived_encryption_keys(encryption: &EncryptionConfig) -> Result<Vec<(u8, Vec<u8>)>> {
        encryption.validate()?;
        let mut keys = Vec::with_capacity(encryption.keys.len());
        for key in &encryption.keys {
            keys.push((key.slot, key.provider.derive()?));
        }
        keys.sort_by_key(|(slot, _)| *slot);
        Ok(keys)
    }
}

impl VolumeProvisioner for MemGptProvisioner {
    fn provision(
        &mut self,
        config: &VolumeConfig,
        discovered: Option<&DiscoveredVolume>,
        disc: &Discoverer,
    ) -> Result<Option<DiscoveredVolume>> {
        self.last_failure_retryable = false;
        self.last_failure_phase = None;
        if config.volume_type != VolumeType::Partition {
            return Ok(discovered.cloned());
        }

        if let Some(found) = discovered {
            self.events.push(ProvisioningEvent::LocateExisting {
                id: config.id.clone(),
                dev_path: found.dev_path.clone(),
            });
            if config.grow != Some(true) {
                self.ensure_opened(config, found)?;
                self.ensure_formatted(config, found)?;
                return Ok(Some(found.clone()));
            }
            if !self.tables.contains_key(&found.parent) {
                return Err(BlockError::NotFound(format!(
                    "registered GPT table for {} not found",
                    found.parent
                )));
            }
            self.events.push(ProvisioningEvent::LockParent {
                id: config.id.clone(),
                parent: found.parent.clone(),
            });
            self.maybe_fail(ProvisioningPhase::LockParent)?;
            let (entry, grow_event) = {
                let state = self.tables.get_mut(&found.parent).ok_or_else(|| {
                    BlockError::NotFound(format!(
                        "registered GPT table for {} not found",
                        found.parent
                    ))
                })?;
                let before = state
                    .table
                    .find(config.match_label.as_deref().unwrap_or(&config.id))
                    .cloned()
                    .ok_or_else(|| {
                        BlockError::NotFound(format!(
                            "partition {} not found",
                            config.match_label.as_deref().unwrap_or(&config.id)
                        ))
                    })?;
                let from_size = before
                    .sector_count()
                    .checked_mul(state.sector_size)
                    .ok_or_else(|| BlockError::Geometry("partition size overflows".to_string()))?;
                let plan =
                    apply_partition_growth_plan(&mut state.table, config, state.sector_size)?;
                let label = config.match_label.as_deref().unwrap_or(&config.id);
                let entry =
                    state.table.find(label).cloned().ok_or_else(|| {
                        BlockError::NotFound(format!("partition {label} not found"))
                    })?;
                let to_size = entry
                    .sector_count()
                    .checked_mul(state.sector_size)
                    .ok_or_else(|| BlockError::Geometry("partition size overflows".to_string()))?;
                let grow_event = (plan.grow_by != 0).then(|| ProvisioningEvent::GrowPartition {
                    id: config.id.clone(),
                    parent: found.parent.clone(),
                    dev_path: found.dev_path.clone(),
                    from_size,
                    to_size,
                });
                (entry, grow_event)
            };
            if let Some(event) = grow_event {
                self.events.push(event);
                self.maybe_fail(ProvisioningPhase::GrowPartition)?;
            }
            let mut updated = found.clone();
            updated.size = entry
                .sector_count()
                .checked_mul(
                    self.tables
                        .get(&found.parent)
                        .ok_or_else(|| {
                            BlockError::NotFound(format!(
                                "registered GPT table for {} not found",
                                found.parent
                            ))
                        })?
                        .sector_size,
                )
                .ok_or_else(|| BlockError::Geometry("partition size overflows".to_string()))?;
            updated.label = Some(entry.name.clone());
            updated.filesystem = config.filesystem;
            updated.role = PartitionRole::from_label(&entry.name);
            self.ensure_opened(config, &updated)?;
            self.ensure_formatted(config, &updated)?;
            return Ok(Some(updated));
        }

        let Some(disk) = disc.resolve_partition_provisioning_disk(config)? else {
            return Ok(None);
        };
        let label = config
            .match_label
            .as_deref()
            .unwrap_or(&config.id)
            .to_string();
        let created_state = self.created.get(&config.id).cloned();
        if let Some(created) = created_state
            && created.parent == disk.dev_name
            && created.label.eq_ignore_ascii_case(&label)
        {
            let found = {
                let state = self.tables.get(&disk.dev_name).ok_or_else(|| {
                    BlockError::NotFound(format!(
                        "registered GPT table for {} not found",
                        disk.dev_name
                    ))
                })?;
                let entry =
                    state.table.find(&label).cloned().ok_or_else(|| {
                        BlockError::NotFound(format!("partition {label} not found"))
                    })?;
                Self::discovered_from_entry(
                    &disk,
                    &state.table,
                    &entry,
                    state.sector_size,
                    config.filesystem,
                )?
            };
            self.events.push(ProvisioningEvent::ReuseCreatedPartition {
                id: config.id.clone(),
                parent: disk.dev_name.clone(),
                dev_path: found.dev_path.clone(),
            });
            self.ensure_create_post_steps(config, &found)?;
            return Ok(Some(found));
        }

        if !self.tables.contains_key(&disk.dev_name) {
            return Err(BlockError::NotFound(format!(
                "registered GPT table for {} not found",
                disk.dev_name
            )));
        }
        self.events.push(ProvisioningEvent::LockParent {
            id: config.id.clone(),
            parent: disk.dev_name.clone(),
        });
        self.maybe_fail(ProvisioningPhase::LockParent)?;
        let (found, create_event) = {
            let state = self.tables.get_mut(&disk.dev_name).ok_or_else(|| {
                BlockError::NotFound(format!(
                    "registered GPT table for {} not found",
                    disk.dev_name
                ))
            })?;
            let part_guid = format!("{}-part-{}", config.id, state.table.entries.len() + 1);
            let entry = apply_partition_create_plan(
                &mut state.table,
                config,
                state.sector_size,
                part_guid.clone(),
            )?;
            state.table.validate()?;
            let found = Self::discovered_from_entry(
                &disk,
                &state.table,
                &entry,
                state.sector_size,
                config.filesystem,
            )?;
            let size = entry
                .sector_count()
                .checked_mul(state.sector_size)
                .ok_or_else(|| BlockError::Geometry("partition size overflows".to_string()))?;
            let create_event = ProvisioningEvent::CreatePartition {
                id: config.id.clone(),
                parent: disk.dev_name.clone(),
                dev_path: found.dev_path.clone(),
                part_guid,
                size,
            };
            (found, create_event)
        };
        self.created.insert(
            config.id.clone(),
            CreatedPartitionState {
                parent: disk.dev_name.clone(),
                label,
            },
        );
        self.events.push(create_event);
        self.maybe_fail(ProvisioningPhase::CreatePartition)?;
        self.ensure_create_post_steps(config, &found)?;
        Ok(Some(found))
    }

    fn failure_is_retryable(&self, _error: &BlockError) -> bool {
        self.last_failure_retryable
    }

    fn failure_phase(&self, _error: &BlockError) -> Option<ProvisioningPhase> {
        self.last_failure_phase
    }
}

fn failed_volume_phase_for_provisioning_phase(phase: ProvisioningPhase) -> VolumePhase {
    match phase {
        ProvisioningPhase::OpenEncryptedVolume => VolumePhase::Opening,
        ProvisioningPhase::CloseEncryptedVolume
        | ProvisioningPhase::UnmountEncryptedVolume
        | ProvisioningPhase::FormatEncryptedVolume => VolumePhase::Provisioning,
        ProvisioningPhase::LocateExisting
        | ProvisioningPhase::ReuseCreatedPartition
        | ProvisioningPhase::LockParent
        | ProvisioningPhase::CreatePartition
        | ProvisioningPhase::GrowPartition
        | ProvisioningPhase::WipePartition
        | ProvisioningPhase::Format => VolumePhase::Provisioning,
    }
}

/// The default system volumes every Talos node requires.
pub fn system_volumes() -> Vec<ManagedVolume> {
    vec![
        ManagedVolume {
            config: {
                let mut c = VolumeConfig::partition("EFI", "EFI", 0);
                c.filesystem = Some(FilesystemType::Vfat);
                c
            },
            class: VolumeClass::System,
            priority: 0,
        },
        ManagedVolume {
            config: {
                let mut c = VolumeConfig::partition("META", "META", 0);
                c.filesystem = None;
                c
            },
            class: VolumeClass::System,
            priority: 1,
        },
        ManagedVolume {
            config: VolumeConfig::partition("STATE", "STATE", 0),
            class: VolumeClass::System,
            priority: 2,
        },
        ManagedVolume {
            config: VolumeConfig::partition("EPHEMERAL", "EPHEMERAL", 0),
            class: VolumeClass::System,
            priority: 3,
        },
    ]
}

fn optional_system_volume_priority(id: &str) -> Option<u32> {
    match id {
        IMAGE_CACHE_VOLUME_ID => Some(IMAGE_CACHE_SYSTEM_PRIORITY),
        _ => None,
    }
}

/// The volume manager: a registry of managed volumes plus their live statuses.
#[derive(Debug, Default)]
pub struct VolumeManager {
    volumes: Vec<ManagedVolume>,
    statuses: BTreeMap<String, VolumeStatus>,
}

impl VolumeManager {
    /// A fresh manager with no volumes.
    pub fn new() -> Self {
        VolumeManager::default()
    }

    /// A manager pre-seeded with the default system volumes.
    pub fn with_system_volumes() -> Self {
        let mut m = VolumeManager::new();
        for v in system_volumes() {
            m.volumes.push(v);
        }
        m
    }

    /// Build a manager from machine-config declarations: system-volume
    /// overrides replace built-in Talos entries or add known optional system
    /// volumes, and user volumes are appended after system volumes in
    /// declaration order.
    pub fn from_declarations(
        system_overrides: impl IntoIterator<Item = VolumeConfig>,
        user_volumes: impl IntoIterator<Item = VolumeConfig>,
    ) -> Result<Self> {
        let mut manager = VolumeManager::with_system_volumes();
        for config in system_overrides {
            manager.apply_system_override(config)?;
        }
        for (index, config) in user_volumes.into_iter().enumerate() {
            let offset = u32::try_from(index).map_err(|_| {
                BlockError::InvalidDevice("too many user volume declarations".to_string())
            })?;
            manager.register(ManagedVolume {
                config,
                class: VolumeClass::User,
                priority: USER_VOLUME_PRIORITY_BASE
                    .checked_add(offset)
                    .ok_or_else(|| {
                        BlockError::InvalidDevice("too many user volume declarations".to_string())
                    })?,
            })?;
        }
        Ok(manager)
    }

    /// Replace a default system volume config while preserving class/priority,
    /// or add a known optional source-managed system volume.
    pub fn apply_system_override(&mut self, config: VolumeConfig) -> Result<()> {
        config.validate()?;
        let id = config.id.clone();
        if let Some(index) = self
            .volumes
            .iter()
            .position(|v| v.class == VolumeClass::System && v.config.id == id)
        {
            let priority = self.volumes[index].priority;
            self.volumes[index] = ManagedVolume {
                config,
                class: VolumeClass::System,
                priority,
            };
            self.statuses.remove(&id);
            return Ok(());
        }

        let Some(priority) = optional_system_volume_priority(&id) else {
            return Err(BlockError::NotFound(format!(
                "system volume {id} is not managed"
            )));
        };
        self.register(ManagedVolume {
            config,
            class: VolumeClass::System,
            priority,
        })
    }

    /// Register a volume, validating it and rejecting duplicate ids.
    pub fn register(&mut self, volume: ManagedVolume) -> Result<()> {
        volume.config.validate()?;
        if self.volumes.iter().any(|v| v.config.id == volume.config.id) {
            return Err(BlockError::InvalidDevice(format!(
                "duplicate volume id {}",
                volume.config.id
            )));
        }
        self.volumes.push(volume);
        Ok(())
    }

    /// Register a user volume with a default user priority.
    pub fn register_user(&mut self, config: VolumeConfig) -> Result<()> {
        self.register(ManagedVolume {
            config,
            class: VolumeClass::User,
            priority: 100,
        })
    }

    /// Number of registered volumes.
    pub fn len(&self) -> usize {
        self.volumes.len()
    }

    /// Whether there are no registered volumes.
    pub fn is_empty(&self) -> bool {
        self.volumes.is_empty()
    }

    /// The registered volumes, ordered by provisioning priority then id.
    pub fn ordered(&self) -> Vec<&ManagedVolume> {
        let mut v: Vec<&ManagedVolume> = self.volumes.iter().collect();
        v.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| a.config.id.cmp(&b.config.id))
        });
        v
    }

    /// Look up a volume's live status, if reconciliation has reached it.
    pub fn status(&self, id: &str) -> Option<&VolumeStatus> {
        self.statuses.get(id)
    }

    /// Look up a registered volume declaration by id.
    pub fn volume(&self, id: &str) -> Option<&ManagedVolume> {
        self.volumes.iter().find(|v| v.config.id == id)
    }

    /// Count of volumes currently in [`VolumePhase::Ready`].
    pub fn ready_count(&self) -> usize {
        self.statuses.values().filter(|s| s.is_ready()).count()
    }

    /// Reconcile every registered volume against `disc`, attempting to drive
    /// each to `Ready`. Volumes whose backing device is not found are left in
    /// [`VolumePhase::Waiting`]; volumes that fail provisioning are recorded as
    /// [`VolumePhase::Failed`]. Returns the number of volumes that reached
    /// `Ready`.
    ///
    /// Reconciliation is idempotent: re-running it picks up volumes that were
    /// previously waiting once their device appears.
    pub fn reconcile(&mut self, disc: &Discoverer) -> Result<usize> {
        let mut provisioner = NoopVolumeProvisioner;
        self.reconcile_with_provisioner(disc, &mut provisioner)
    }

    /// Reconcile using an explicit provisioning boundary.
    ///
    /// This keeps the ordinary controller path host-safe while allowing tests
    /// and runtime adapters to prove that create/grow decisions are wired from
    /// the manager into a side-effect boundary.
    pub fn reconcile_with_provisioner(
        &mut self,
        disc: &Discoverer,
        provisioner: &mut impl VolumeProvisioner,
    ) -> Result<usize> {
        // Snapshot the ordered ids to avoid borrowing self mutably + immutably.
        let order: Vec<(String, VolumeConfig)> = self
            .ordered()
            .into_iter()
            .map(|v| (v.config.id.clone(), v.config.clone()))
            .collect();

        for (id, config) in order {
            // Already ready — skip.
            if self
                .statuses
                .get(&id)
                .map(|s| s.is_ready())
                .unwrap_or(false)
            {
                continue;
            }
            let status = self
                .statuses
                .entry(id.clone())
                .or_insert_with(|| VolumeStatus::new(config.clone()));
            if status.phase == VolumePhase::Failed {
                if status.retryable {
                    let _ = status.restore_retry();
                } else {
                    continue;
                }
            }

            // tmpfs volumes need no backing device.
            if config.volume_type == VolumeType::Tmpfs {
                if status.phase == VolumePhase::Waiting {
                    status.locate("tmpfs")?;
                }
                if status.phase != VolumePhase::Ready {
                    status.make_ready()?;
                }
                continue;
            }

            let discovered = disc.resolve(&config)?;
            match provisioner.provision(&config, discovered.as_ref(), disc) {
                Ok(Some(found)) => {
                    if status.phase == VolumePhase::Waiting {
                        status.locate(found.dev_path.clone())?;
                    } else if status.located_on.is_none() {
                        status.located_on = Some(found.dev_path.clone());
                    }
                    // Drive to ready; record a failure reason if provisioning
                    // raises an error.
                    if let Err(e) = status.make_ready() {
                        // Reset to a fail state with the error message.
                        if status.phase != VolumePhase::Failed {
                            let _ = status.fail(e.to_string());
                        }
                    }
                }
                Ok(None) => {
                    // Device not present yet: stay waiting.
                }
                Err(error) => {
                    if status.phase != VolumePhase::Failed {
                        if provisioner.failure_is_retryable(&error) {
                            let pre_fail_phase = provisioner
                                .failure_phase(&error)
                                .map(failed_volume_phase_for_provisioning_phase)
                                .unwrap_or(VolumePhase::Provisioning);
                            let _ = status.fail_retryable_from(pre_fail_phase, error.to_string());
                        } else {
                            let _ = status.fail(error.to_string());
                        }
                    }
                }
            }
        }
        Ok(self.ready_count())
    }

    /// Whether every registered volume has reached `Ready`.
    pub fn all_ready(&self) -> bool {
        !self.volumes.is_empty() && self.ready_count() == self.volumes.len()
    }

    /// The ids of volumes still waiting for a device.
    pub fn waiting(&self) -> Vec<String> {
        self.volumes
            .iter()
            .filter(|v| {
                self.statuses
                    .get(&v.config.id)
                    .map(|s| s.phase == VolumePhase::Waiting)
                    .unwrap_or(true)
            })
            .map(|v| v.config.id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::disk::{Disk, DiskBus};
    use crate::encryption::{EncryptionConfig, EncryptionKey, KeyProvider};
    use crate::layout::type_guid;
    use crate::mount::{
        MemMounter, MountSpec, VolumeMountStatusResource, VolumeMountStatusSpec, reconcile_mount,
        reconcile_unmount,
    };
    use crate::partition::{Partition, PartitionRole};

    const SECTOR: u64 = 512;

    fn disc_with(parts: &[(&str, u32, u64, u64, PartitionRole)]) -> Discoverer {
        let mut d = Discoverer::new();
        let mut disk = Disk::new("sda", 64 * 1024 * 1024 * 1024, DiskBus::Scsi);
        disk.sector_size = 512;
        d.add_disk(disk).unwrap();
        for &(name, num, start, end, role) in parts {
            let mut p = Partition::new(name, num, start, end, role);
            p.sector_size = 512;
            d.add_partition("sda", p).unwrap();
        }
        d
    }

    fn full_disc() -> Discoverer {
        disc_with(&[
            ("sda1", 1, 2048, 206847, PartitionRole::Efi),
            ("sda2", 2, 206848, 208895, PartitionRole::Meta),
            ("sda3", 3, 208896, 413695, PartitionRole::State),
            ("sda4", 4, 413696, 50_000_000, PartitionRole::Ephemeral),
        ])
    }

    fn full_disc_with_data() -> Discoverer {
        let mut disc = full_disc();
        let mut data = Partition::new("sda5", 5, 50_000_001, 51_000_000, PartitionRole::Other);
        data.label = Some("DATA".to_string());
        data.filesystem = Some(FilesystemType::Xfs);
        disc.add_partition("sda", data).unwrap();
        disc
    }

    #[test]
    fn system_volumes_default_set() {
        let m = VolumeManager::with_system_volumes();
        assert_eq!(m.len(), 4);
        let ids: Vec<&str> = m.ordered().iter().map(|v| v.config.id.as_str()).collect();
        assert_eq!(ids, vec!["EFI", "META", "STATE", "EPHEMERAL"]);
        assert!(m.ordered().iter().all(|v| v.class == VolumeClass::System));
    }

    #[test]
    fn reconcile_drives_all_system_volumes_ready() {
        let mut m = VolumeManager::with_system_volumes();
        let disc = full_disc();
        let ready = m.reconcile(&disc).unwrap();
        assert_eq!(ready, 4);
        assert!(m.all_ready());
        assert!(m.waiting().is_empty());
        assert!(m.status("STATE").unwrap().is_ready());
        assert_eq!(
            m.status("EFI").unwrap().located_on.as_deref(),
            Some("/dev/sda1")
        );
    }

    #[test]
    fn missing_device_stays_waiting_then_resolves() {
        let mut m = VolumeManager::with_system_volumes();
        // Only EFI present.
        let disc = disc_with(&[("sda1", 1, 2048, 206847, PartitionRole::Efi)]);
        let ready = m.reconcile(&disc).unwrap();
        assert_eq!(ready, 1);
        assert!(!m.all_ready());
        let waiting = m.waiting();
        assert!(waiting.contains(&"STATE".to_string()));

        // Now the rest appear; re-reconcile picks them up.
        let full = full_disc();
        let ready2 = m.reconcile(&full).unwrap();
        assert_eq!(ready2, 4);
        assert!(m.all_ready());
    }

    #[test]
    fn duplicate_volume_id_rejected() {
        let mut m = VolumeManager::with_system_volumes();
        let dup = ManagedVolume {
            config: VolumeConfig::partition("STATE", "STATE", 0),
            class: VolumeClass::User,
            priority: 50,
        };
        assert!(m.register(dup).is_err());
    }

    #[test]
    fn user_volume_registered_with_default_priority() {
        let mut m = VolumeManager::new();
        m.register_user(VolumeConfig::partition("DATA", "DATA", 1024))
            .unwrap();
        let v = m.ordered()[0];
        assert_eq!(v.class, VolumeClass::User);
        assert_eq!(v.priority, 100);
    }

    #[test]
    fn tmpfs_volume_becomes_ready_without_device() {
        let mut m = VolumeManager::new();
        let mut cfg = VolumeConfig::partition("RUN", "RUN", 0);
        cfg.volume_type = VolumeType::Tmpfs;
        cfg.match_label = None;
        cfg.filesystem = None;
        m.register(ManagedVolume {
            config: cfg,
            class: VolumeClass::System,
            priority: 5,
        })
        .unwrap();
        let empty = Discoverer::new();
        let ready = m.reconcile(&empty).unwrap();
        assert_eq!(ready, 1);
        assert!(m.status("RUN").unwrap().is_ready());
    }

    #[test]
    fn ordering_respects_priority_then_id() {
        let mut m = VolumeManager::new();
        m.register(ManagedVolume {
            config: VolumeConfig::partition("Z", "Z", 0),
            class: VolumeClass::User,
            priority: 10,
        })
        .unwrap();
        m.register(ManagedVolume {
            config: VolumeConfig::partition("A", "A", 0),
            class: VolumeClass::User,
            priority: 10,
        })
        .unwrap();
        m.register(ManagedVolume {
            config: VolumeConfig::partition("FIRST", "FIRST", 0),
            class: VolumeClass::System,
            priority: 1,
        })
        .unwrap();
        let ids: Vec<&str> = m.ordered().iter().map(|v| v.config.id.as_str()).collect();
        assert_eq!(ids, vec!["FIRST", "A", "Z"]);
    }

    #[test]
    fn volume_declaration_plan_merges_system_overrides_and_user_volumes() {
        let mut state = VolumeConfig::partition("STATE", "STATE", 32 * 1024 * 1024);
        state.filesystem = Some(FilesystemType::Ext4);
        state.max_size = Some(128 * 1024 * 1024);

        let data = VolumeConfig::partition("DATA", "DATA", 16 * 1024 * 1024);
        let logs = VolumeConfig::partition("LOGS", "LOGS", 8 * 1024 * 1024);
        let manager =
            VolumeManager::from_declarations([state.clone()], [data.clone(), logs.clone()])
                .unwrap();

        let ids: Vec<&str> = manager
            .ordered()
            .iter()
            .map(|v| v.config.id.as_str())
            .collect();
        assert_eq!(
            ids,
            vec!["EFI", "META", "STATE", "EPHEMERAL", "DATA", "LOGS"]
        );

        let state_volume = manager.volume("STATE").unwrap();
        assert_eq!(state_volume.class, VolumeClass::System);
        assert_eq!(state_volume.priority, 2);
        assert_eq!(state_volume.config.filesystem, Some(FilesystemType::Ext4));
        assert_eq!(state_volume.config.max_size, Some(128 * 1024 * 1024));

        let data_volume = manager.volume("DATA").unwrap();
        assert_eq!(data_volume.class, VolumeClass::User);
        assert_eq!(data_volume.priority, 100);
        assert_eq!(data_volume.config, data);
        assert_eq!(manager.volume("LOGS").unwrap().priority, 101);
    }

    #[test]
    fn volume_declaration_plan_adds_optional_imagecache_system_volume() {
        let mut image_cache =
            VolumeConfig::partition("IMAGECACHE", "IMAGECACHE", 500 * 1024 * 1024);
        image_cache.disk_selector = Some("system_disk".to_string());
        image_cache.max_size = Some(10 * 1024 * 1024 * 1024);
        image_cache.grow = Some(false);
        image_cache.filesystem = Some(FilesystemType::Ext4);

        let manager = VolumeManager::from_declarations([image_cache.clone()], []).unwrap();

        let ids: Vec<&str> = manager
            .ordered()
            .iter()
            .map(|v| v.config.id.as_str())
            .collect();
        assert_eq!(ids, vec!["EFI", "META", "STATE", "EPHEMERAL", "IMAGECACHE"]);

        let planned = manager.volume("IMAGECACHE").unwrap();
        assert_eq!(planned.class, VolumeClass::System);
        assert_eq!(planned.priority, 4);
        assert_eq!(planned.config, image_cache);
    }

    #[test]
    fn partition_grow_plan_preserves_declaration_fields_through_manager() {
        let mut ephemeral = VolumeConfig::partition("EPHEMERAL", "EPHEMERAL", 1024);
        ephemeral.grow = Some(true);
        ephemeral.max_size = Some(64 * 1024 * 1024 * 1024);

        let mut data = VolumeConfig::partition("DATA", "DATA", 1024);
        data.grow = Some(false);
        data.max_size = Some(8 * 1024 * 1024 * 1024);

        let manager = VolumeManager::from_declarations([ephemeral.clone()], [data.clone()])
            .expect("valid grow declarations");

        let planned_ephemeral = manager.volume("EPHEMERAL").unwrap();
        assert_eq!(planned_ephemeral.class, VolumeClass::System);
        assert_eq!(planned_ephemeral.priority, 3);
        assert_eq!(planned_ephemeral.config.grow, Some(true));
        assert_eq!(planned_ephemeral.config.max_size, ephemeral.max_size);

        let planned_data = manager.volume("DATA").unwrap();
        assert_eq!(planned_data.class, VolumeClass::User);
        assert_eq!(planned_data.priority, USER_VOLUME_PRIORITY_BASE);
        assert_eq!(planned_data.config.grow, Some(false));
        assert_eq!(planned_data.config.max_size, data.max_size);
    }

    #[test]
    fn raw_disk_volume_reconciles_ready_on_exact_disk_match() {
        let mut disc = Discoverer::new();
        disc.add_disk(Disk::new("nvme0n1", 64 * 1024 * 1024 * 1024, DiskBus::Nvme))
            .unwrap();

        let mut manager = VolumeManager::new();
        manager
            .register_user(VolumeConfig::disk("u-data", r#"disk.transport == "nvme""#))
            .unwrap();

        let ready = manager.reconcile(&disc).unwrap();

        assert_eq!(ready, 1);
        assert!(manager.status("u-data").unwrap().is_ready());
        assert_eq!(
            manager.status("u-data").unwrap().located_on.as_deref(),
            Some("/dev/nvme0n1")
        );
    }

    #[test]
    fn raw_volume_partition_reconciles_ready_without_filesystem_requirement() {
        let mut disc = full_disc();
        let mut raw = Partition::new("sda5", 5, 50_000_001, 51_000_000, PartitionRole::Other);
        raw.label = Some("r-local-data".to_string());
        disc.add_partition("sda", raw).unwrap();

        let mut manager = VolumeManager::new();
        manager
            .register_user(VolumeConfig::raw_partition(
                "r-local-data",
                "r-local-data",
                1024,
            ))
            .unwrap();

        let ready = manager.reconcile(&disc).unwrap();

        assert_eq!(ready, 1);
        assert!(manager.status("r-local-data").unwrap().is_ready());
        assert_eq!(
            manager
                .status("r-local-data")
                .unwrap()
                .located_on
                .as_deref(),
            Some("/dev/sda5")
        );
    }

    #[test]
    fn raw_disk_volume_reconcile_propagates_multiple_disk_match_error() {
        let mut disc = Discoverer::new();
        disc.add_disk(Disk::new("nvme0n1", 64 * 1024 * 1024 * 1024, DiskBus::Nvme))
            .unwrap();
        disc.add_disk(Disk::new(
            "nvme1n1",
            128 * 1024 * 1024 * 1024,
            DiskBus::Nvme,
        ))
        .unwrap();

        let mut manager = VolumeManager::new();
        manager
            .register_user(VolumeConfig::disk("u-data", r#"disk.transport == "nvme""#))
            .unwrap();

        let err = manager.reconcile(&disc);

        assert!(matches!(
            err,
            Err(BlockError::InvalidDevice(message))
                if message.contains("matched 2 disks") && message.contains("refine disk selector")
        ));
    }

    #[test]
    fn volume_declaration_plan_rejects_duplicate_or_unknown_ids() {
        let duplicate_user = VolumeManager::from_declarations(
            [],
            [
                VolumeConfig::partition("DATA", "DATA", 1024),
                VolumeConfig::partition("DATA", "LOGS", 1024),
            ],
        );
        assert!(matches!(
            duplicate_user,
            Err(BlockError::InvalidDevice(message)) if message.contains("duplicate volume id DATA")
        ));

        let user_collides_with_system =
            VolumeManager::from_declarations([], [VolumeConfig::partition("STATE", "DATA", 1024)]);
        assert!(matches!(
            user_collides_with_system,
            Err(BlockError::InvalidDevice(message)) if message.contains("duplicate volume id STATE")
        ));

        let unknown_system =
            VolumeManager::from_declarations([VolumeConfig::partition("DATA", "DATA", 1024)], []);
        assert!(matches!(
            unknown_system,
            Err(BlockError::NotFound(message)) if message.contains("system volume DATA")
        ));
    }

    #[test]
    fn volume_declaration_plan_reconciles_system_and_user_devices() {
        let mut ephemeral = VolumeConfig::partition("EPHEMERAL", "EPHEMERAL", 1024);
        ephemeral.filesystem = Some(FilesystemType::Xfs);
        let data = VolumeConfig::partition("DATA", "DATA", 1024);

        let mut manager = VolumeManager::from_declarations([ephemeral], [data]).unwrap();
        let ready = manager.reconcile(&full_disc_with_data()).unwrap();

        assert_eq!(ready, 5);
        assert!(manager.all_ready());
        assert_eq!(
            manager.status("DATA").unwrap().located_on.as_deref(),
            Some("/dev/sda5")
        );
        assert_eq!(
            manager.status("EPHEMERAL").unwrap().located_on.as_deref(),
            Some("/dev/sda4")
        );
    }

    #[test]
    fn optional_imagecache_system_volume_reconciles_on_source_partition_label() {
        let mut image_cache =
            VolumeConfig::partition("IMAGECACHE", "IMAGECACHE", 500 * 1024 * 1024);
        image_cache.disk_selector = Some("system_disk".to_string());
        image_cache.max_size = Some(1024 * 1024 * 1024);
        image_cache.grow = Some(false);
        image_cache.filesystem = Some(FilesystemType::Ext4);

        let mut disc = full_disc();
        let mut partition = Partition::new("sda5", 5, 50_000_001, 52_000_000, PartitionRole::Other);
        partition.label = Some("IMAGECACHE".to_string());
        partition.filesystem = Some(FilesystemType::Ext4);
        disc.add_partition("sda", partition).unwrap();

        let mut manager = VolumeManager::from_declarations([image_cache], []).unwrap();
        let ready = manager.reconcile(&disc).unwrap();

        assert_eq!(ready, 5);
        assert!(manager.all_ready());
        assert_eq!(
            manager.status("IMAGECACHE").unwrap().located_on.as_deref(),
            Some("/dev/sda5")
        );
    }

    #[test]
    // A false selector leaves the volume waiting; label-only matching is not attempted.
    fn runtime_disk_selector_false_keeps_user_volume_waiting_without_label_defaulting() {
        let mut data = VolumeConfig::partition("DATA", "DATA", 1024);
        data.disk_selector = Some(r#"disk.transport == "nvme""#.to_string());

        let mut manager = VolumeManager::from_declarations([], [data]).unwrap();
        let ready = manager.reconcile(&full_disc_with_data()).unwrap();

        assert_eq!(ready, 4);
        assert!(
            manager
                .status("DATA")
                .is_none_or(|status| !status.is_ready())
        );
        assert!(manager.waiting().contains(&"DATA".to_string()));
    }

    #[test]
    fn runtime_gpt_mutation_create_invokes_partition_create_boundary() {
        let disk_size = 16 * 1024 * 1024 * 1024;
        let total_sectors = disk_size / SECTOR;
        let table = GptTable::new("disk-guid", total_sectors).unwrap();
        let available_bytes = (table.last_usable_lba - 2048 + 1) * SECTOR;

        let mut disc = Discoverer::new();
        let mut disk = Disk::new("sda", disk_size, DiskBus::Scsi);
        disk.sector_size = SECTOR;
        disc.add_disk(disk).unwrap();

        let mut config = VolumeConfig::partition("DATA", "DATA", 1024);
        config.disk_selector = Some(r#"disk.transport == "scsi""#.to_string());
        config.relative_max_size = Some(50);
        let expected_sectors = config.resolve_max_size(available_bytes).unwrap() / SECTOR;

        let mut provisioner = MemGptProvisioner::default();
        provisioner.register_table("sda", table, SECTOR).unwrap();
        let mut manager = VolumeManager::new();
        manager.register_user(config).unwrap();

        let ready = manager
            .reconcile_with_provisioner(&disc, &mut provisioner)
            .unwrap();

        assert_eq!(ready, 1);
        assert!(manager.status("DATA").unwrap().is_ready());
        assert_eq!(
            manager.status("DATA").unwrap().located_on.as_deref(),
            Some("/dev/sda1")
        );
        let entry = provisioner.table("sda").unwrap().find("DATA").unwrap();
        assert_eq!(entry.type_guid, type_guid::LINUX_FILESYSTEM);
        assert_eq!(entry.sector_count(), expected_sectors);
        provisioner.table("sda").unwrap().validate().unwrap();
    }

    #[test]
    fn runtime_gpt_mutation_underflow_failure_does_not_mutate_table() {
        let disk_size = 16 * 1024 * 1024 * 1024;
        let table = GptTable::new("disk-guid", disk_size / SECTOR).unwrap();

        let mut disc = Discoverer::new();
        let mut disk = Disk::new("sda", disk_size, DiskBus::Scsi);
        disk.sector_size = SECTOR;
        disc.add_disk(disk).unwrap();

        let mut config = VolumeConfig::partition("DATA", "DATA", 1024);
        config.disk_selector = Some(r#"disk.transport == "scsi""#.to_string());
        config.max_size = Some(128 * 1024 * 1024 * 1024);
        config.negative_max_size = true;

        let mut provisioner = MemGptProvisioner::default();
        provisioner
            .register_table("sda", table.clone(), SECTOR)
            .unwrap();
        let mut manager = VolumeManager::new();
        manager.register_user(config).unwrap();

        let ready = manager
            .reconcile_with_provisioner(&disc, &mut provisioner)
            .unwrap();

        assert_eq!(ready, 0);
        assert_eq!(provisioner.table("sda").unwrap(), &table);
        let status = manager.status("DATA").unwrap();
        assert_eq!(status.phase, VolumePhase::Failed);
        assert!(
            status
                .reason
                .as_deref()
                .is_some_and(|reason| reason.contains("partition size cannot be negative"))
        );
    }

    #[test]
    fn runtime_gpt_mutation_growth_invokes_partition_growth_boundary() {
        let disk_size = 16 * 1024 * 1024 * 1024;
        let total_sectors = disk_size / SECTOR;
        let mut table = GptTable::new("disk-guid", total_sectors).unwrap();
        table
            .allocate(
                (4 * 1024 * 1024 * 1024) / SECTOR,
                type_guid::LINUX_FILESYSTEM,
                "data-part-1",
                "DATA",
            )
            .unwrap();
        let original_entry = table.find("DATA").unwrap().clone();

        let mut disc = Discoverer::new();
        let mut disk = Disk::new("sda", disk_size, DiskBus::Scsi);
        disk.sector_size = SECTOR;
        disc.add_disk(disk).unwrap();
        let mut data = Partition::new(
            "sda1",
            1,
            original_entry.first_lba,
            original_entry.last_lba,
            PartitionRole::Other,
        );
        data.label = Some("DATA".to_string());
        data.filesystem = Some(FilesystemType::Xfs);
        disc.add_partition("sda", data).unwrap();

        let mut config = VolumeConfig::partition("DATA", "DATA", 1024);
        config.disk_selector = Some(r#"disk.transport == "scsi""#.to_string());
        config.grow = Some(true);
        config.max_size = Some(6 * 1024 * 1024 * 1024);

        let mut provisioner = MemGptProvisioner::default();
        provisioner.register_table("sda", table, SECTOR).unwrap();
        let mut manager = VolumeManager::new();
        manager.register_user(config).unwrap();

        let ready = manager
            .reconcile_with_provisioner(&disc, &mut provisioner)
            .unwrap();

        assert_eq!(ready, 1);
        assert!(manager.status("DATA").unwrap().is_ready());
        let grown = provisioner.table("sda").unwrap().find("DATA").unwrap();
        assert!(grown.last_lba > original_entry.last_lba);
        assert_eq!(grown.sector_count() * SECTOR, 6 * 1024 * 1024 * 1024);
        provisioner.table("sda").unwrap().validate().unwrap();
    }

    #[test]
    fn runtime_gpt_mutation_ambiguous_disk_selector_does_not_mutate_tables() {
        let disk_size = 16 * 1024 * 1024 * 1024;
        let sda_table = GptTable::new("sda-guid", disk_size / SECTOR).unwrap();
        let sdb_table = GptTable::new("sdb-guid", disk_size / SECTOR).unwrap();

        let mut disc = Discoverer::new();
        disc.add_disk(Disk::new("sda", disk_size, DiskBus::Scsi))
            .unwrap();
        disc.add_disk(Disk::new("sdb", disk_size, DiskBus::Scsi))
            .unwrap();

        let mut config = VolumeConfig::partition("DATA", "DATA", 1024);
        config.disk_selector = Some(r#"disk.transport == "scsi""#.to_string());

        let mut provisioner = MemGptProvisioner::default();
        provisioner
            .register_table("sda", sda_table.clone(), SECTOR)
            .unwrap();
        provisioner
            .register_table("sdb", sdb_table.clone(), SECTOR)
            .unwrap();
        let mut manager = VolumeManager::new();
        manager.register_user(config).unwrap();

        let ready = manager
            .reconcile_with_provisioner(&disc, &mut provisioner)
            .unwrap();

        assert_eq!(ready, 0);
        assert_eq!(provisioner.table("sda").unwrap(), &sda_table);
        assert_eq!(provisioner.table("sdb").unwrap(), &sdb_table);
        let status = manager.status("DATA").unwrap();
        assert_eq!(status.phase, VolumePhase::Failed);
        assert!(
            status
                .reason
                .as_deref()
                .is_some_and(|reason| reason.contains("matched 2 provisioning disks"))
        );
    }

    #[test]
    fn runtime_gpt_mutation_noop_reconcile_still_waits_for_missing_partition() {
        let mut disc = Discoverer::new();
        disc.add_disk(Disk::new("sda", 16 * 1024 * 1024 * 1024, DiskBus::Scsi))
            .unwrap();

        let mut config = VolumeConfig::partition("DATA", "DATA", 1024);
        config.disk_selector = Some(r#"disk.transport == "scsi""#.to_string());
        let mut manager = VolumeManager::new();
        manager.register_user(config).unwrap();

        let ready = manager.reconcile(&disc).unwrap();

        assert_eq!(ready, 0);
        assert_eq!(manager.status("DATA").unwrap().phase, VolumePhase::Waiting);
        assert!(manager.waiting().contains(&"DATA".to_string()));
    }

    fn event_phases(provisioner: &MemGptProvisioner) -> Vec<ProvisioningPhase> {
        provisioner
            .events()
            .iter()
            .map(ProvisioningEvent::phase)
            .collect()
    }

    fn missing_scsi_data_inputs(
        config: VolumeConfig,
    ) -> (Discoverer, MemGptProvisioner, VolumeManager) {
        let disk_size = 16 * 1024 * 1024 * 1024;
        let table = GptTable::new("disk-guid", disk_size / SECTOR).unwrap();

        let mut disc = Discoverer::new();
        let mut disk = Disk::new("sda", disk_size, DiskBus::Scsi);
        disk.sector_size = SECTOR;
        disc.add_disk(disk).unwrap();

        let mut provisioner = MemGptProvisioner::default();
        provisioner.register_table("sda", table, SECTOR).unwrap();
        let mut manager = VolumeManager::new();
        manager.register_user(config).unwrap();
        (disc, provisioner, manager)
    }

    fn static_encryption_config() -> EncryptionConfig {
        let mut enc = EncryptionConfig::new();
        enc.add_key(
            EncryptionKey::new(
                0,
                KeyProvider::Static {
                    passphrase: "wave74-host-safe-key".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        enc
    }

    fn encrypted_scsi_partition_config() -> VolumeConfig {
        let mut config =
            VolumeConfig::partition("DATA", "DATA", 1024).encrypted(static_encryption_config());
        config.disk_selector = Some(r#"disk.transport == "scsi""#.to_string());
        config
    }

    fn ready_encrypted_data_provisioner() -> MemGptProvisioner {
        let (disc, mut provisioner, mut manager) =
            missing_scsi_data_inputs(encrypted_scsi_partition_config());
        let ready = manager
            .reconcile_with_provisioner(&disc, &mut provisioner)
            .unwrap();
        assert_eq!(ready, 1);
        assert!(provisioner.is_mapping_open("DATA"));
        provisioner
    }

    fn mounted_data_mapping(target: &str) -> (MountStatus, MemMounter) {
        let mut status = MountStatus::new(MountSpec::new(
            "/dev/mapper/luks2-DATA",
            target,
            FilesystemType::Xfs,
        ));
        let mut mounter = MemMounter::new();
        reconcile_mount(&mut status, &mut mounter).unwrap();
        (status, mounter)
    }

    fn ready_encrypted_data_status() -> VolumeStatus {
        let mut status = VolumeStatus::new(encrypted_scsi_partition_config());
        status.locate("/dev/sda1").unwrap();
        status.make_ready().unwrap();
        status
    }

    fn ready_unencrypted_data_status() -> VolumeStatus {
        let mut status = VolumeStatus::new(VolumeConfig::partition("DATA", "DATA", 1024));
        status.locate("/dev/sda1").unwrap();
        status.make_ready().unwrap();
        status
    }

    #[test]
    fn runtime_encryption_phase_create_opens_before_format() {
        let (disc, mut provisioner, mut manager) =
            missing_scsi_data_inputs(encrypted_scsi_partition_config());

        let ready = manager
            .reconcile_with_provisioner(&disc, &mut provisioner)
            .unwrap();

        assert_eq!(ready, 1);
        let status = manager.status("DATA").unwrap();
        assert!(status.is_ready());
        assert!(status.config.is_encrypted());
        assert_eq!(
            event_phases(&provisioner),
            vec![
                ProvisioningPhase::LockParent,
                ProvisioningPhase::CreatePartition,
                ProvisioningPhase::WipePartition,
                ProvisioningPhase::FormatEncryptedVolume,
                ProvisioningPhase::OpenEncryptedVolume,
                ProvisioningPhase::Format,
            ]
        );
        assert!(provisioner.events().iter().any(|event| matches!(
            event,
            ProvisioningEvent::OpenEncryptedVolume {
                id,
                dev_path,
                mapped_path,
                key_slot: 0,
                cipher: Cipher::AesXtsPlain64,
            } if id == "DATA" && dev_path == "/dev/sda1" && mapped_path == "/dev/mapper/luks2-DATA"
        )));
        let header = provisioner.luks_header("DATA").unwrap();
        assert!(header.is_valid());
        assert_eq!(header.cipher, Cipher::AesXtsPlain64);
        assert_eq!(header.active_slots(), 1);
        assert_eq!(header.open(b"wave74-host-safe-key").unwrap(), 0);
        assert_eq!(provisioner.opened_key_slot("DATA"), Some(0));
        assert_eq!(
            provisioner.opened_mapped_path("DATA"),
            Some("/dev/mapper/luks2-DATA")
        );
        assert_eq!(
            provisioner
                .cryptsetup_commands()
                .iter()
                .map(|cmd| cmd.args[0].as_str())
                .collect::<Vec<_>>(),
            vec!["luksFormat", "open"]
        );
        assert!(provisioner.events().iter().any(|event| matches!(
            event,
            ProvisioningEvent::Format {
                id,
                dev_path,
                filesystem: FilesystemType::Xfs,
            } if id == "DATA" && dev_path == "/dev/mapper/luks2-DATA"
        )));
    }

    #[test]
    fn runtime_cryptsetup_created_partition_records_format_open_and_mapped_format_target() {
        let (disc, mut provisioner, mut manager) =
            missing_scsi_data_inputs(encrypted_scsi_partition_config());

        let ready = manager
            .reconcile_with_provisioner(&disc, &mut provisioner)
            .unwrap();

        assert_eq!(ready, 1);
        assert_eq!(
            provisioner
                .cryptsetup_commands()
                .iter()
                .map(|cmd| cmd.args[0].as_str())
                .collect::<Vec<_>>(),
            vec!["luksFormat", "open"]
        );
        assert!(provisioner.cryptsetup_commands().iter().all(|cmd| {
            let rendered = format!("{cmd:?}");
            !rendered.contains("wave74-host-safe-key")
        }));
        assert!(provisioner.events().iter().any(|event| matches!(
            event,
            ProvisioningEvent::FormatEncryptedVolume {
                id,
                dev_path,
                key_slot: 0,
                cipher: Cipher::AesXtsPlain64,
            } if id == "DATA" && dev_path == "/dev/sda1"
        )));
        assert!(provisioner.events().iter().any(|event| matches!(
            event,
            ProvisioningEvent::Format {
                id,
                dev_path,
                filesystem: FilesystemType::Xfs,
            } if id == "DATA" && dev_path == "/dev/mapper/luks2-DATA"
        )));
        assert!(provisioner.is_mapping_open("DATA"));
    }

    #[test]
    fn runtime_cryptsetup_close_records_mapper_close_without_header_loss() {
        let (disc, mut provisioner, mut manager) =
            missing_scsi_data_inputs(encrypted_scsi_partition_config());
        manager
            .reconcile_with_provisioner(&disc, &mut provisioner)
            .unwrap();
        provisioner.clear_events();

        provisioner.close_encrypted_volume("DATA").unwrap();

        assert!(!provisioner.is_mapping_open("DATA"));
        assert_eq!(provisioner.opened_key_slot("DATA"), None);
        assert_eq!(provisioner.opened_mapped_path("DATA"), None);
        assert!(provisioner.luks_header("DATA").is_some());
        assert!(provisioner.events().iter().any(|event| matches!(
            event,
            ProvisioningEvent::CloseEncryptedVolume {
                id,
                mapper_name,
                mapped_path,
            } if id == "DATA" && mapper_name == "luks2-DATA" && mapped_path == "/dev/mapper/luks2-DATA"
        )));
        assert_eq!(
            provisioner.cryptsetup_commands().last().unwrap(),
            &CryptsetupCommand::close("luks2-DATA")
        );
    }

    #[test]
    fn runtime_cryptsetup_close_failure_keeps_mapping_open() {
        let (disc, mut provisioner, mut manager) =
            missing_scsi_data_inputs(encrypted_scsi_partition_config());
        manager
            .reconcile_with_provisioner(&disc, &mut provisioner)
            .unwrap();
        provisioner.clear_events();
        let before_commands = provisioner.cryptsetup_commands().len();
        provisioner.fail_next_phase(
            ProvisioningPhase::CloseEncryptedVolume,
            "cryptsetup close interrupted",
        );

        let err = provisioner.close_encrypted_volume("DATA").unwrap_err();

        assert!(err.to_string().contains("cryptsetup close interrupted"));
        assert!(provisioner.is_mapping_open("DATA"));
        assert_eq!(
            provisioner.opened_mapped_path("DATA"),
            Some("/dev/mapper/luks2-DATA")
        );
        assert_eq!(provisioner.cryptsetup_commands().len(), before_commands);
        assert_eq!(
            event_phases(&provisioner),
            vec![ProvisioningPhase::CloseEncryptedVolume]
        );
    }

    #[test]
    fn runtime_encrypted_unmount_close_unmounts_before_closing_mapper() {
        let mut provisioner = ready_encrypted_data_provisioner();
        let (mut mount_status, mut mounter) = mounted_data_mapping("/var");
        provisioner.clear_events();

        provisioner
            .unmount_and_close_encrypted_volume("DATA", &mut mount_status, &mut mounter)
            .unwrap();

        assert_eq!(mount_status.phase, MountPhase::Unmounted);
        assert!(!mounter.is_mounted("/var"));
        assert!(!provisioner.is_mapping_open("DATA"));
        assert_eq!(
            event_phases(&provisioner),
            vec![
                ProvisioningPhase::UnmountEncryptedVolume,
                ProvisioningPhase::CloseEncryptedVolume,
            ]
        );
        assert!(provisioner.events().iter().any(|event| matches!(
            event,
            ProvisioningEvent::UnmountEncryptedVolume {
                id,
                mapped_path,
                target,
            } if id == "DATA" && mapped_path == "/dev/mapper/luks2-DATA" && target == "/var"
        )));
        assert_eq!(
            provisioner.cryptsetup_commands().last().unwrap(),
            &CryptsetupCommand::close("luks2-DATA")
        );
    }

    #[test]
    fn runtime_encrypted_unmount_close_unmount_failure_does_not_close_mapper() {
        let mut provisioner = ready_encrypted_data_provisioner();
        let (mut mount_status, mut mounter) = mounted_data_mapping("/var");
        provisioner.clear_events();
        let before_commands = provisioner.cryptsetup_commands().len();
        mounter.fail_next_unmount("/var", "umount interrupted");

        let err = provisioner
            .unmount_and_close_encrypted_volume("DATA", &mut mount_status, &mut mounter)
            .unwrap_err();

        assert!(err.to_string().contains("umount interrupted"));
        assert_eq!(mount_status.phase, MountPhase::Unmounting);
        assert!(mounter.is_mounted("/var"));
        assert!(provisioner.is_mapping_open("DATA"));
        assert_eq!(provisioner.cryptsetup_commands().len(), before_commands);
        assert_eq!(
            event_phases(&provisioner),
            vec![ProvisioningPhase::UnmountEncryptedVolume]
        );
    }

    #[test]
    fn runtime_encrypted_unmount_close_close_failure_leaves_mapper_open_after_unmount() {
        let mut provisioner = ready_encrypted_data_provisioner();
        let (mut mount_status, mut mounter) = mounted_data_mapping("/var");
        provisioner.clear_events();
        let before_commands = provisioner.cryptsetup_commands().len();
        provisioner.fail_next_phase(
            ProvisioningPhase::CloseEncryptedVolume,
            "cryptsetup close interrupted",
        );

        let err = provisioner
            .unmount_and_close_encrypted_volume("DATA", &mut mount_status, &mut mounter)
            .unwrap_err();

        assert!(err.to_string().contains("cryptsetup close interrupted"));
        assert_eq!(mount_status.phase, MountPhase::Unmounted);
        assert!(!mounter.is_mounted("/var"));
        assert!(provisioner.is_mapping_open("DATA"));
        assert_eq!(
            provisioner.opened_mapped_path("DATA"),
            Some("/dev/mapper/luks2-DATA")
        );
        assert_eq!(provisioner.cryptsetup_commands().len(), before_commands);
        assert_eq!(
            event_phases(&provisioner),
            vec![
                ProvisioningPhase::UnmountEncryptedVolume,
                ProvisioningPhase::CloseEncryptedVolume,
            ]
        );
    }

    #[test]
    fn runtime_encrypted_unmount_close_is_idempotent_after_success() {
        let mut provisioner = ready_encrypted_data_provisioner();
        let (mut mount_status, mut mounter) = mounted_data_mapping("/var");

        provisioner
            .unmount_and_close_encrypted_volume("DATA", &mut mount_status, &mut mounter)
            .unwrap();
        let commands_after_first = provisioner.cryptsetup_commands().len();
        let events_after_first = provisioner.events().len();

        provisioner
            .unmount_and_close_encrypted_volume("DATA", &mut mount_status, &mut mounter)
            .unwrap();

        assert_eq!(mount_status.phase, MountPhase::Unmounted);
        assert!(!mounter.is_mounted("/var"));
        assert!(!provisioner.is_mapping_open("DATA"));
        assert_eq!(
            provisioner.cryptsetup_commands().len(),
            commands_after_first
        );
        assert_eq!(provisioner.events().len(), events_after_first);
    }

    #[test]
    fn runtime_encrypted_unmount_close_rejects_mount_source_mismatch() {
        let mut provisioner = ready_encrypted_data_provisioner();
        let mut mount_status = MountStatus::new(MountSpec::new(
            "/dev/mapper/luks2-OTHER",
            "/var",
            FilesystemType::Xfs,
        ));
        let mut mounter = MemMounter::new();
        reconcile_mount(&mut mount_status, &mut mounter).unwrap();
        provisioner.clear_events();
        let before_commands = provisioner.cryptsetup_commands().len();

        let err = provisioner
            .unmount_and_close_encrypted_volume("DATA", &mut mount_status, &mut mounter)
            .unwrap_err();

        assert!(err.to_string().contains("does not match encrypted mapping"));
        assert!(mounter.is_mounted("/var"));
        assert!(provisioner.is_mapping_open("DATA"));
        assert!(provisioner.events().is_empty());
        assert_eq!(provisioner.cryptsetup_commands().len(), before_commands);
    }

    #[test]
    fn runtime_encrypted_unmount_close_does_not_leak_passphrase() {
        let mut provisioner = ready_encrypted_data_provisioner();
        let (mut mount_status, mut mounter) = mounted_data_mapping("/var");

        provisioner
            .unmount_and_close_encrypted_volume("DATA", &mut mount_status, &mut mounter)
            .unwrap();

        for event in provisioner.events() {
            assert!(!format!("{event:?}").contains("wave74-host-safe-key"));
        }
        for command in provisioner.cryptsetup_commands() {
            assert!(!format!("{command:?}").contains("wave74-host-safe-key"));
        }
        assert_eq!(
            provisioner.cryptsetup_commands().last().unwrap(),
            &CryptsetupCommand::close("luks2-DATA")
        );
        assert_eq!(
            provisioner
                .cryptsetup_commands()
                .last()
                .unwrap()
                .stdin_bytes,
            0
        );
    }

    #[test]
    fn runtime_encrypted_unmount_close_preserves_wave76_cryptsetup_contract() {
        let mut provisioner = ready_encrypted_data_provisioner();
        let (mut mount_status, mut mounter) = mounted_data_mapping("/var");

        provisioner
            .unmount_and_close_encrypted_volume("DATA", &mut mount_status, &mut mounter)
            .unwrap();

        assert_eq!(
            provisioner
                .cryptsetup_commands()
                .iter()
                .map(|cmd| cmd.args[0].as_str())
                .collect::<Vec<_>>(),
            vec!["luksFormat", "open", "close"]
        );
        assert!(provisioner.events().iter().any(|event| matches!(
            event,
            ProvisioningEvent::FormatEncryptedVolume { dev_path, .. } if dev_path == "/dev/sda1"
        )));
        assert!(provisioner.events().iter().any(|event| matches!(
            event,
            ProvisioningEvent::Format { dev_path, .. } if dev_path == "/dev/mapper/luks2-DATA"
        )));
        assert_eq!(
            provisioner.cryptsetup_commands().last().unwrap(),
            &CryptsetupCommand::close("luks2-DATA")
        );
        assert!(provisioner.luks_header("DATA").is_some());
    }

    #[test]
    fn runtime_encrypted_mount_close_eligibility_defers_while_mount_finalizer_remains() {
        let mut provisioner = ready_encrypted_data_provisioner();
        let mut status = ready_encrypted_data_status();
        provisioner.clear_events();
        let before_commands = provisioner.cryptsetup_commands().len();
        let eligibility = VolumeCloseEligibility::tearing_down()
            .with_volume_finalizer(MOUNT_CONTROLLER_FINALIZER);

        let outcome = provisioner
            .close_volume_when_eligible(&mut status, &eligibility)
            .unwrap();

        assert_eq!(
            outcome,
            VolumeCloseOutcome::Deferred(VolumeCloseBlocker::VolumeFinalizer {
                name: MOUNT_CONTROLLER_FINALIZER.to_string()
            })
        );
        assert!(eligibility.has_volume_finalizer(MOUNT_CONTROLLER_FINALIZER));
        assert_eq!(status.phase, VolumePhase::Ready);
        assert!(provisioner.is_mapping_open("DATA"));
        assert_eq!(
            provisioner.opened_mapped_path("DATA"),
            Some("/dev/mapper/luks2-DATA")
        );
        assert!(provisioner.events().is_empty());
        assert_eq!(provisioner.cryptsetup_commands().len(), before_commands);
    }

    #[test]
    fn runtime_encrypted_mount_close_eligibility_defers_until_volume_is_tearing_down() {
        let mut provisioner = ready_encrypted_data_provisioner();
        let mut status = ready_encrypted_data_status();
        provisioner.clear_events();
        let before_commands = provisioner.cryptsetup_commands().len();
        let eligibility = VolumeCloseEligibility::running();

        let outcome = provisioner
            .close_volume_when_eligible(&mut status, &eligibility)
            .unwrap();

        assert_eq!(
            outcome,
            VolumeCloseOutcome::Deferred(VolumeCloseBlocker::NotTearingDown)
        );
        assert_eq!(status.phase, VolumePhase::Ready);
        assert!(provisioner.is_mapping_open("DATA"));
        assert!(provisioner.events().is_empty());
        assert_eq!(provisioner.cryptsetup_commands().len(), before_commands);
    }

    #[test]
    fn runtime_encrypted_mount_close_eligibility_defers_while_mount_resource_remains() {
        let mut provisioner = ready_encrypted_data_provisioner();
        let mut status = ready_encrypted_data_status();
        let (mut mount_status, mut mounter) = mounted_data_mapping("/var");
        reconcile_unmount(&mut mount_status, &mut mounter).unwrap();
        provisioner.clear_events();
        let before_commands = provisioner.cryptsetup_commands().len();
        let eligibility = VolumeCloseEligibility::tearing_down().with_mount_status(&mount_status);

        let outcome = provisioner
            .close_volume_when_eligible(&mut status, &eligibility)
            .unwrap();

        assert_eq!(
            outcome,
            VolumeCloseOutcome::Deferred(VolumeCloseBlocker::MountStatusPresent {
                target: "/var".to_string()
            })
        );
        assert_eq!(mount_status.phase, MountPhase::Unmounted);
        assert!(!mounter.is_mounted("/var"));
        assert_eq!(status.phase, VolumePhase::Ready);
        assert!(provisioner.is_mapping_open("DATA"));
        assert!(provisioner.events().is_empty());
        assert_eq!(provisioner.cryptsetup_commands().len(), before_commands);
    }

    #[test]
    fn runtime_encrypted_mount_close_eligibility_defers_while_volume_mount_status_resource_remains()
    {
        let mut provisioner = ready_encrypted_data_provisioner();
        let mut status = ready_encrypted_data_status();
        let volume_mount_status = VolumeMountStatusResource::new(
            "kubelet-DATA",
            VolumeMountStatusSpec::new("DATA", "kubelet", "/var/lib/kubelet")
                .with_read_only(false)
                .with_detached(false)
                .with_disable_access_time(true)
                .with_secure(true),
        )
        .unwrap();
        provisioner.clear_events();
        let before_commands = provisioner.cryptsetup_commands().len();
        let eligibility =
            VolumeCloseEligibility::tearing_down().with_volume_mount_status(&volume_mount_status);

        let outcome = provisioner
            .close_volume_when_eligible(&mut status, &eligibility)
            .unwrap();

        assert_eq!(
            outcome,
            VolumeCloseOutcome::Deferred(VolumeCloseBlocker::VolumeMountStatusPresent {
                id: "kubelet-DATA".to_string(),
                target: "/var/lib/kubelet".to_string()
            })
        );
        assert_eq!(status.phase, VolumePhase::Ready);
        assert!(provisioner.is_mapping_open("DATA"));
        assert!(provisioner.events().is_empty());
        assert_eq!(provisioner.cryptsetup_commands().len(), before_commands);

        let outcome = provisioner
            .close_volume_when_eligible(&mut status, &VolumeCloseEligibility::tearing_down())
            .unwrap();

        assert_eq!(outcome, VolumeCloseOutcome::Closed);
        assert_eq!(status.phase, VolumePhase::Closed);
        assert!(!provisioner.is_mapping_open("DATA"));
    }

    #[test]
    fn runtime_encrypted_mount_close_eligibility_closes_after_unmount_finalizer_removed() {
        let mut provisioner = ready_encrypted_data_provisioner();
        let mut status = ready_encrypted_data_status();
        provisioner.clear_events();
        let eligibility = VolumeCloseEligibility::tearing_down();

        let outcome = provisioner
            .close_volume_when_eligible(&mut status, &eligibility)
            .unwrap();

        assert_eq!(outcome, VolumeCloseOutcome::Closed);
        assert_eq!(status.phase, VolumePhase::Closed);
        assert_eq!(status.pre_fail_phase, None);
        assert!(!status.retryable);
        assert_eq!(status.reason, None);
        assert!(!provisioner.is_mapping_open("DATA"));
        assert_eq!(provisioner.opened_key_slot("DATA"), None);
        assert_eq!(provisioner.opened_mapped_path("DATA"), None);
        assert_eq!(
            event_phases(&provisioner),
            vec![ProvisioningPhase::CloseEncryptedVolume]
        );
        assert_eq!(
            provisioner.cryptsetup_commands().last().unwrap(),
            &CryptsetupCommand::close("luks2-DATA")
        );
    }

    #[test]
    fn runtime_encrypted_mount_close_eligibility_close_failure_is_retryable_and_restores_prior_phase()
     {
        let mut provisioner = ready_encrypted_data_provisioner();
        let mut status = ready_encrypted_data_status();
        provisioner.clear_events();
        let before_commands = provisioner.cryptsetup_commands().len();
        provisioner.fail_next_phase(
            ProvisioningPhase::CloseEncryptedVolume,
            "cryptsetup close interrupted",
        );
        let eligibility = VolumeCloseEligibility::tearing_down();

        let err = provisioner
            .close_volume_when_eligible(&mut status, &eligibility)
            .unwrap_err();

        assert!(err.to_string().contains("cryptsetup close interrupted"));
        assert!(provisioner.is_mapping_open("DATA"));
        assert_eq!(status.phase, VolumePhase::Failed);
        assert!(status.retryable);
        assert_eq!(status.pre_fail_phase, Some(VolumePhase::Ready));
        assert_eq!(
            status.reason.as_deref(),
            Some("invalid device: cryptsetup close interrupted")
        );
        assert_eq!(provisioner.cryptsetup_commands().len(), before_commands);
        assert_eq!(
            event_phases(&provisioner),
            vec![ProvisioningPhase::CloseEncryptedVolume]
        );

        let outcome = provisioner
            .close_volume_when_eligible(&mut status, &eligibility)
            .unwrap();

        assert_eq!(outcome, VolumeCloseOutcome::Closed);
        assert_eq!(status.phase, VolumePhase::Closed);
        assert!(!status.retryable);
        assert_eq!(status.pre_fail_phase, None);
        assert_eq!(status.reason, None);
        assert!(!provisioner.is_mapping_open("DATA"));
        assert_eq!(
            provisioner.cryptsetup_commands().last().unwrap(),
            &CryptsetupCommand::close("luks2-DATA")
        );
    }

    #[test]
    fn runtime_encrypted_mount_close_eligibility_closed_mapping_short_circuits_without_close_command()
     {
        let mut provisioner = ready_encrypted_data_provisioner();
        let mut status = ready_encrypted_data_status();
        let eligibility = VolumeCloseEligibility::tearing_down();

        let first = provisioner
            .close_volume_when_eligible(&mut status, &eligibility)
            .unwrap();
        let commands_after_first = provisioner.cryptsetup_commands().len();
        let events_after_first = provisioner.events().len();

        let second = provisioner
            .close_volume_when_eligible(&mut status, &eligibility)
            .unwrap();

        assert_eq!(first, VolumeCloseOutcome::Closed);
        assert_eq!(second, VolumeCloseOutcome::AlreadyClosed);
        assert_eq!(status.phase, VolumePhase::Closed);
        assert!(!provisioner.is_mapping_open("DATA"));
        assert_eq!(
            provisioner.cryptsetup_commands().len(),
            commands_after_first
        );
        assert_eq!(provisioner.events().len(), events_after_first);
    }

    #[test]
    fn runtime_encrypted_mount_close_eligibility_unencrypted_volume_short_circuits_without_close_command()
     {
        let mut provisioner = MemGptProvisioner::default();
        let mut status = ready_unencrypted_data_status();
        let eligibility = VolumeCloseEligibility::tearing_down();

        let outcome = provisioner
            .close_volume_when_eligible(&mut status, &eligibility)
            .unwrap();

        assert_eq!(outcome, VolumeCloseOutcome::NotEncrypted);
        assert!(!status.config.is_encrypted());
        assert_eq!(status.phase, VolumePhase::Closed);
        assert!(provisioner.events().is_empty());
        assert!(provisioner.cryptsetup_commands().is_empty());
        assert_eq!(provisioner.opened_key_slot("DATA"), None);
        assert_eq!(provisioner.opened_mapped_path("DATA"), None);
    }

    #[test]
    fn runtime_encryption_phase_open_failure_retries_without_rewipe_or_duplicate_create() {
        let (disc, mut provisioner, mut manager) =
            missing_scsi_data_inputs(encrypted_scsi_partition_config());
        provisioner.fail_next_phase(
            ProvisioningPhase::OpenEncryptedVolume,
            "cryptsetup open interrupted",
        );

        let first = manager
            .reconcile_with_provisioner(&disc, &mut provisioner)
            .unwrap();

        assert_eq!(first, 0);
        assert_eq!(provisioner.table("sda").unwrap().entries.len(), 1);
        assert_eq!(
            event_phases(&provisioner),
            vec![
                ProvisioningPhase::LockParent,
                ProvisioningPhase::CreatePartition,
                ProvisioningPhase::WipePartition,
                ProvisioningPhase::FormatEncryptedVolume,
                ProvisioningPhase::OpenEncryptedVolume,
            ]
        );
        let failed = manager.status("DATA").unwrap();
        assert_eq!(failed.phase, VolumePhase::Failed);
        assert!(failed.retryable);
        assert_eq!(failed.pre_fail_phase, Some(VolumePhase::Opening));
        assert!(failed.config.is_encrypted());
        let failed_header = provisioner.luks_header("DATA").unwrap();
        assert!(failed_header.is_valid());
        assert_eq!(failed_header.active_slots(), 1);
        assert_eq!(failed_header.open(b"wave74-host-safe-key").unwrap(), 0);
        assert_eq!(provisioner.opened_key_slot("DATA"), None);
        assert_eq!(provisioner.opened_mapped_path("DATA"), None);

        provisioner.clear_events();
        let second = manager
            .reconcile_with_provisioner(&disc, &mut provisioner)
            .unwrap();

        assert_eq!(second, 1);
        assert_eq!(provisioner.table("sda").unwrap().entries.len(), 1);
        assert!(manager.status("DATA").unwrap().is_ready());
        assert_eq!(manager.status("DATA").unwrap().reason, None);
        let retried_header = provisioner.luks_header("DATA").unwrap();
        assert_eq!(retried_header.active_slots(), 1);
        assert_eq!(retried_header.open(b"wave74-host-safe-key").unwrap(), 0);
        assert_eq!(provisioner.opened_key_slot("DATA"), Some(0));
        assert_eq!(
            provisioner.opened_mapped_path("DATA"),
            Some("/dev/mapper/luks2-DATA")
        );
        assert_eq!(
            event_phases(&provisioner),
            vec![
                ProvisioningPhase::ReuseCreatedPartition,
                ProvisioningPhase::OpenEncryptedVolume,
                ProvisioningPhase::Format,
            ]
        );
    }

    #[test]
    fn runtime_luks2_header_enrollment_opens_selected_keyslot() {
        let mut enc = EncryptionConfig::new();
        enc.cipher = Cipher::AesCbcEssiv;
        enc.add_key(
            EncryptionKey::new(
                6,
                KeyProvider::Static {
                    passphrase: "wave75-secondary".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        enc.add_key(
            EncryptionKey::new(
                3,
                KeyProvider::Static {
                    passphrase: "wave75-primary".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        let mut config = VolumeConfig::partition("DATA", "DATA", 1024).encrypted(enc);
        config.disk_selector = Some(r#"disk.transport == "scsi""#.to_string());
        let (disc, mut provisioner, mut manager) = missing_scsi_data_inputs(config);

        let ready = manager
            .reconcile_with_provisioner(&disc, &mut provisioner)
            .unwrap();

        assert_eq!(ready, 1);
        assert!(manager.status("DATA").unwrap().is_ready());
        let header = provisioner.luks_header("DATA").unwrap();
        assert!(header.is_valid());
        assert_eq!(header.cipher, Cipher::AesCbcEssiv);
        assert_eq!(header.active_slots(), 2);
        assert_eq!(header.open(b"wave75-primary").unwrap(), 3);
        assert_eq!(header.open(b"wave75-secondary").unwrap(), 6);
        assert!(header.open(b"wrong").is_err());
        assert_eq!(provisioner.opened_key_slot("DATA"), Some(3));
        assert_eq!(
            provisioner.opened_mapped_path("DATA"),
            Some("/dev/mapper/luks2-DATA")
        );
        assert!(provisioner.events().iter().any(|event| matches!(
            event,
            ProvisioningEvent::OpenEncryptedVolume {
                id,
                dev_path,
                mapped_path,
                key_slot: 3,
                cipher: Cipher::AesCbcEssiv,
            } if id == "DATA" && dev_path == "/dev/sda1" && mapped_path == "/dev/mapper/luks2-DATA"
        )));
    }

    #[test]
    fn runtime_luks2_open_validation_rejects_cipher_mismatch_before_ready() {
        let (disc, mut provisioner, mut manager) =
            missing_scsi_data_inputs(encrypted_scsi_partition_config());
        provisioner.luks_headers.insert(
            "DATA".to_string(),
            Luks2Header::format("stale-DATA", Cipher::AesCbcEssiv),
        );

        let ready = manager
            .reconcile_with_provisioner(&disc, &mut provisioner)
            .unwrap();

        assert_eq!(ready, 0);
        assert_eq!(provisioner.table("sda").unwrap().entries.len(), 1);
        assert_eq!(
            event_phases(&provisioner),
            vec![
                ProvisioningPhase::LockParent,
                ProvisioningPhase::CreatePartition,
                ProvisioningPhase::WipePartition,
            ]
        );
        let status = manager.status("DATA").unwrap();
        assert_eq!(status.phase, VolumePhase::Failed);
        assert!(!status.retryable);
        assert!(
            status
                .reason
                .as_deref()
                .is_some_and(|reason| reason.contains("LUKS cipher mismatch"))
        );
        assert_eq!(provisioner.opened_key_slot("DATA"), None);
        assert_eq!(provisioner.opened_mapped_path("DATA"), None);
    }

    #[test]
    fn runtime_luks2_unencrypted_partition_never_creates_or_opens_header() {
        let mut config = VolumeConfig::partition("DATA", "DATA", 1024);
        config.disk_selector = Some(r#"disk.transport == "scsi""#.to_string());
        let (disc, mut provisioner, mut manager) = missing_scsi_data_inputs(config);

        let ready = manager
            .reconcile_with_provisioner(&disc, &mut provisioner)
            .unwrap();

        assert_eq!(ready, 1);
        assert!(manager.status("DATA").unwrap().is_ready());
        assert_eq!(
            event_phases(&provisioner),
            vec![
                ProvisioningPhase::LockParent,
                ProvisioningPhase::CreatePartition,
                ProvisioningPhase::WipePartition,
                ProvisioningPhase::Format,
            ]
        );
        assert!(provisioner.luks_header("DATA").is_none());
        assert_eq!(provisioner.opened_key_slot("DATA"), None);
        assert_eq!(provisioner.opened_mapped_path("DATA"), None);
        assert!(
            provisioner
                .events()
                .iter()
                .all(|event| event.phase() != ProvisioningPhase::OpenEncryptedVolume)
        );
    }

    #[test]
    fn runtime_encryption_phase_existing_partition_opens_before_format() {
        let disk_size = 16 * 1024 * 1024 * 1024;
        let mut disc = Discoverer::new();
        let mut disk = Disk::new("sda", disk_size, DiskBus::Scsi);
        disk.sector_size = SECTOR;
        disc.add_disk(disk).unwrap();
        let mut data = Partition::new(
            "sda1",
            1,
            2048,
            (4 * 1024 * 1024 * 1024) / SECTOR,
            PartitionRole::Other,
        );
        data.sector_size = SECTOR;
        data.label = Some("DATA".to_string());
        disc.add_partition("sda", data).unwrap();

        let mut provisioner = MemGptProvisioner::default();
        let mut manager = VolumeManager::new();
        manager
            .register_user(
                VolumeConfig::partition("DATA", "DATA", 1024).encrypted(static_encryption_config()),
            )
            .unwrap();

        let ready = manager
            .reconcile_with_provisioner(&disc, &mut provisioner)
            .unwrap();

        assert_eq!(ready, 1);
        assert!(manager.status("DATA").unwrap().is_ready());
        assert_eq!(
            event_phases(&provisioner),
            vec![
                ProvisioningPhase::LocateExisting,
                ProvisioningPhase::FormatEncryptedVolume,
                ProvisioningPhase::OpenEncryptedVolume,
                ProvisioningPhase::Format,
            ]
        );
        assert!(event_phases(&provisioner).iter().all(|phase| !matches!(
            phase,
            ProvisioningPhase::LockParent
                | ProvisioningPhase::CreatePartition
                | ProvisioningPhase::GrowPartition
                | ProvisioningPhase::WipePartition
        )));
    }

    #[test]
    fn runtime_encryption_phase_raw_partition_skips_format_but_opens() {
        let mut config =
            VolumeConfig::raw_partition("DATA", "DATA", 1024).encrypted(static_encryption_config());
        config.disk_selector = Some(r#"disk.transport == "scsi""#.to_string());
        let (disc, mut provisioner, mut manager) = missing_scsi_data_inputs(config);

        let ready = manager
            .reconcile_with_provisioner(&disc, &mut provisioner)
            .unwrap();

        assert_eq!(ready, 1);
        assert!(manager.status("DATA").unwrap().is_ready());
        assert!(manager.status("DATA").unwrap().config.is_encrypted());
        assert_eq!(
            event_phases(&provisioner),
            vec![
                ProvisioningPhase::LockParent,
                ProvisioningPhase::CreatePartition,
                ProvisioningPhase::WipePartition,
                ProvisioningPhase::FormatEncryptedVolume,
                ProvisioningPhase::OpenEncryptedVolume,
            ]
        );
        assert!(
            provisioner
                .events()
                .iter()
                .all(|event| event.phase() != ProvisioningPhase::Format)
        );
    }

    #[test]
    fn runtime_provisioning_phase_create_wipe_format_order_for_missing_partition() {
        let mut config = VolumeConfig::partition("DATA", "DATA", 1024);
        config.disk_selector = Some(r#"disk.transport == "scsi""#.to_string());
        let (disc, mut provisioner, mut manager) = missing_scsi_data_inputs(config);

        let ready = manager
            .reconcile_with_provisioner(&disc, &mut provisioner)
            .unwrap();

        assert_eq!(ready, 1);
        assert!(manager.status("DATA").unwrap().is_ready());
        assert_eq!(
            event_phases(&provisioner),
            vec![
                ProvisioningPhase::LockParent,
                ProvisioningPhase::CreatePartition,
                ProvisioningPhase::WipePartition,
                ProvisioningPhase::Format,
            ]
        );
        assert!(provisioner.table("sda").unwrap().find("DATA").is_some());
    }

    #[test]
    fn runtime_provisioning_phase_raw_partition_skips_format() {
        let mut config = VolumeConfig::raw_partition("DATA", "DATA", 1024);
        config.disk_selector = Some(r#"disk.transport == "scsi""#.to_string());
        let (disc, mut provisioner, mut manager) = missing_scsi_data_inputs(config);

        let ready = manager
            .reconcile_with_provisioner(&disc, &mut provisioner)
            .unwrap();

        assert_eq!(ready, 1);
        assert_eq!(
            event_phases(&provisioner),
            vec![
                ProvisioningPhase::LockParent,
                ProvisioningPhase::CreatePartition,
                ProvisioningPhase::WipePartition,
            ]
        );
        assert!(
            provisioner
                .events()
                .iter()
                .all(|event| event.phase() != ProvisioningPhase::Format)
        );
    }

    #[test]
    fn runtime_provisioning_phase_wipe_failure_retries_without_duplicate_create() {
        let mut config = VolumeConfig::partition("DATA", "DATA", 1024);
        config.disk_selector = Some(r#"disk.transport == "scsi""#.to_string());
        let (disc, mut provisioner, mut manager) = missing_scsi_data_inputs(config);
        provisioner.fail_next_phase(ProvisioningPhase::WipePartition, "wipe interrupted");

        let first = manager
            .reconcile_with_provisioner(&disc, &mut provisioner)
            .unwrap();

        assert_eq!(first, 0);
        assert_eq!(provisioner.table("sda").unwrap().entries.len(), 1);
        assert_eq!(
            event_phases(&provisioner),
            vec![
                ProvisioningPhase::LockParent,
                ProvisioningPhase::CreatePartition,
                ProvisioningPhase::WipePartition,
            ]
        );
        let failed = manager.status("DATA").unwrap();
        assert_eq!(failed.phase, VolumePhase::Failed);
        assert!(failed.retryable);
        assert_eq!(failed.pre_fail_phase, Some(VolumePhase::Provisioning));

        provisioner.clear_events();
        let second = manager
            .reconcile_with_provisioner(&disc, &mut provisioner)
            .unwrap();

        assert_eq!(second, 1);
        assert_eq!(provisioner.table("sda").unwrap().entries.len(), 1);
        assert!(manager.status("DATA").unwrap().is_ready());
        assert_eq!(manager.status("DATA").unwrap().reason, None);
        assert_eq!(
            event_phases(&provisioner),
            vec![
                ProvisioningPhase::ReuseCreatedPartition,
                ProvisioningPhase::WipePartition,
                ProvisioningPhase::Format,
            ]
        );
    }

    #[test]
    fn runtime_provisioning_phase_format_failure_retries_without_rewipe() {
        let mut config = VolumeConfig::partition("DATA", "DATA", 1024);
        config.disk_selector = Some(r#"disk.transport == "scsi""#.to_string());
        let (disc, mut provisioner, mut manager) = missing_scsi_data_inputs(config);
        provisioner.fail_next_phase(ProvisioningPhase::Format, "format interrupted");

        let first = manager
            .reconcile_with_provisioner(&disc, &mut provisioner)
            .unwrap();

        assert_eq!(first, 0);
        assert_eq!(
            event_phases(&provisioner),
            vec![
                ProvisioningPhase::LockParent,
                ProvisioningPhase::CreatePartition,
                ProvisioningPhase::WipePartition,
                ProvisioningPhase::Format,
            ]
        );
        assert!(manager.status("DATA").unwrap().retryable);

        provisioner.clear_events();
        let second = manager
            .reconcile_with_provisioner(&disc, &mut provisioner)
            .unwrap();

        assert_eq!(second, 1);
        assert!(manager.status("DATA").unwrap().is_ready());
        assert_eq!(
            event_phases(&provisioner),
            vec![
                ProvisioningPhase::ReuseCreatedPartition,
                ProvisioningPhase::Format,
            ]
        );
    }

    #[test]
    fn runtime_provisioning_phase_growth_records_grow_before_format() {
        let disk_size = 16 * 1024 * 1024 * 1024;
        let mut table = GptTable::new("disk-guid", disk_size / SECTOR).unwrap();
        table
            .allocate(
                (4 * 1024 * 1024 * 1024) / SECTOR,
                type_guid::LINUX_FILESYSTEM,
                "data-part-1",
                "DATA",
            )
            .unwrap();
        let original_entry = table.find("DATA").unwrap().clone();

        let mut disc = Discoverer::new();
        let mut disk = Disk::new("sda", disk_size, DiskBus::Scsi);
        disk.sector_size = SECTOR;
        disc.add_disk(disk).unwrap();
        let mut data = Partition::new(
            "sda1",
            1,
            original_entry.first_lba,
            original_entry.last_lba,
            PartitionRole::Other,
        );
        data.label = Some("DATA".to_string());
        data.filesystem = Some(FilesystemType::Xfs);
        disc.add_partition("sda", data).unwrap();

        let mut config = VolumeConfig::partition("DATA", "DATA", 1024);
        config.disk_selector = Some(r#"disk.transport == "scsi""#.to_string());
        config.grow = Some(true);
        config.max_size = Some(6 * 1024 * 1024 * 1024);

        let mut provisioner = MemGptProvisioner::default();
        provisioner.register_table("sda", table, SECTOR).unwrap();
        let mut manager = VolumeManager::new();
        manager.register_user(config).unwrap();

        let ready = manager
            .reconcile_with_provisioner(&disc, &mut provisioner)
            .unwrap();

        assert_eq!(ready, 1);
        assert_eq!(
            event_phases(&provisioner),
            vec![
                ProvisioningPhase::LocateExisting,
                ProvisioningPhase::LockParent,
                ProvisioningPhase::GrowPartition,
                ProvisioningPhase::Format,
            ]
        );
        let grown = provisioner.table("sda").unwrap().find("DATA").unwrap();
        assert!(grown.last_lba > original_entry.last_lba);
    }
}
