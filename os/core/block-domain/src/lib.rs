//! # talos-block
//!
//! A port of the Talos block-device subsystem, mirroring
//! `internal/app/machined/pkg/controllers/block` and
//! `pkg/machinery/resources/block` from `siderolabs/talos`.
//!
//! It models the pieces machined uses to turn raw disks into mounted,
//! optionally encrypted volumes:
//!
//! * [`disk`] — block-device discovery metadata ([`Disk`](disk::Disk)).
//! * [`partition`] — a single partition on a disk ([`Partition`](partition::Partition)).
//! * [`gpt`] — GUID partition tables ([`GptTable`](gpt::GptTable)).
//! * [`filesystem`] — filesystem types and superblock-magic probing.
//! * [`probe`] — turning raw device bytes into a [`ProbeResult`](probe::ProbeResult).
//! * [`volume`] — declarative volume config and observed status with a state
//!   machine ([`VolumePhase`](volume::VolumePhase)).
//! * [`mount`] — mount specs / statuses and their lifecycle.
//! * [`encryption`] — LUKS encryption configuration and key providers.
//! * [`luks`] — a minimal LUKS2 header model and key-slot logic.
//! * [`discovery`] — the controller that assembles disks into discovered volumes.
//! * [`layout`] — the standard Talos install partition layout + type GUIDs.
//! * [`makefs`] — filesystem creation (`mkfs.{xfs,ext4,vfat}`) command surface.
//! * [`blkid`] — blkid-style detailed device identification (UUID/label/usage).
//! * [`symlink`] — `/dev/disk/by-*` stable symlink resolution.
//! * [`cryptsetup`] — host-safe cryptsetup/dm-crypt command intent boundary.
//! * [`manager`] — the volume-manager controller (system/user volumes).
//!
//! Where the real subsystem performs syscalls (`ioctl`, `mount(2)`,
//! `cryptsetup`) the boundary is expressed as a trait with an in-memory test
//! implementation. The crate uses only the standard library plus workspace
//! primitives such as [`os_kernel`] and [`os_cosi_domain`].

// These pedantic lints are documentation/attribute noise that do not affect the
// idiom or behavior of this crate, so we opt out of them crate-wide rather than
// sprinkling hundreds of `#[must_use]` attributes and `# Errors` doc sections.
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::missing_errors_doc)]

pub mod blkid;
pub mod controller;
pub mod cryptsetup;
pub mod discovery;
pub mod disk;
pub mod encryption;
pub mod filesystem;
pub mod gpt;
pub mod layout;
pub mod luks;
pub mod makefs;
pub mod manager;
pub mod mount;
pub mod partition;
pub mod probe;
pub mod symlink;
pub mod volume;

pub use blkid::{BlkidInfo, Usage, blkid};
pub use controller::{
    EPHEMERAL_CONFIG_PRESENT_LOCATOR_MATCH, EPHEMERAL_DEFAULT_MIN_SIZE, EPHEMERAL_MOUNT_FILE_MODE,
    EPHEMERAL_MOUNT_POINT, EPHEMERAL_SELINUX_LABEL, EPHEMERAL_VOLUME_ID, MACHINE_CONFIG_ACTIVE_ID,
    MACHINE_CONFIG_NAMESPACE, MACHINE_CONFIG_TYPE, META_DEFAULT_LOCATOR_MATCH, META_KEY_TYPE,
    META_VOLUME_ID, RUNTIME_NAMESPACE, STATE_CONFIG_PRESENT_LOCATOR_MATCH,
    STATE_DEFAULT_LOCATOR_MATCH, STATE_ENCRYPTION_META_KEY_ID, STATE_MOUNT_FILE_MODE,
    STATE_MOUNT_POINT, STATE_SELINUX_LABEL, STATE_VOLUME_ID, SYSTEM_DISK_MATCH,
    SYSTEM_VOLUME_LABEL, USER_VOLUME_MOUNT_POINT, VOLUME_CONFIG_CONTROLLER_NAME,
    VolumeConfigController, VolumeConfigRuntimeMode, machine_config_kind, meta_key_kind,
};
pub use cryptsetup::{
    CryptOpenResult, CryptsetupBackend, CryptsetupCommand, LuksAddKeyRequest, LuksCloseRequest,
    LuksFormatRequest, LuksOpenRequest, MemCryptsetupBackend, mapper_name, mapper_path,
};
pub use discovery::{DiscoveredVolume, Discoverer};
pub use disk::{Disk, DiskBus, DiskType};
pub use encryption::{EncryptionConfig, EncryptionKey, KeyProvider};
pub use filesystem::FilesystemType;
pub use gpt::{GptTable, PartitionEntry};
pub use layout::{
    BootMode, MIN_PARTITION_GROWTH_BYTES, PartitionGrowthPlan, PartitionSpec,
    apply_partition_create_plan, apply_partition_growth_plan, install_table,
    plan_partition_create_size, plan_partition_growth, standard_layout,
};
pub use luks::{Luks2Header, LuksKeySlot};
pub use makefs::{FsMaker, MakeFsOptions, MemFsMaker, mkfs_argv};
pub use manager::{
    IMAGE_CACHE_VOLUME_ID, ManagedVolume, MemGptProvisioner, NoopVolumeProvisioner,
    ProvisioningEvent, ProvisioningPhase, VolumeClass, VolumeManager, VolumeProvisioner,
};
pub use mount::{
    MountFlags, MountRequestSpec, MountSpec, MountStatus, MountStatusResource,
    VolumeMountRequestResource, VolumeMountRequestSpec, VolumeMountStatusResource,
    VolumeMountStatusSpec,
};
pub use partition::{Partition, PartitionRole};
pub use probe::{BlockReader, ProbeResult};
pub use symlink::{SymlinkKind, SymlinkTable};
pub use volume::{
    PartitionMatchPolicy, VolumeConfig, VolumeConfigEncryptionKey, VolumeConfigEncryptionKeyType,
    VolumeConfigEncryptionSpec, VolumeConfigMountSpec, VolumeConfigProvisioningSpec,
    VolumeConfigResource, VolumeConfigSpec, VolumePhase, VolumeStatus, VolumeType,
};
pub use volume::{
    VOLUME_CONFIG_TYPE, VOLUME_STATUS_TYPE, VolumeStatusResource, WAVE_SYSTEM_DISK,
    volume_config_key, volume_status_key,
};

/// Crate-local error type for the block subsystem.
///
/// Distinct from [`os_kernel::Error`] so callers can match on block-specific
/// failure modes; it converts into the workspace error at the crate boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockError {
    /// A device path or identifier was malformed.
    InvalidDevice(String),
    /// A partition table or superblock failed to parse.
    BadTable(String),
    /// A geometry/size constraint was violated (overlap, out of range, ...).
    Geometry(String),
    /// No filesystem / partition table signature was recognised.
    Unrecognized,
    /// A volume/mount state transition was not permitted.
    BadTransition(String),
    /// An encryption key could not be provided or did not unlock a slot.
    KeyFailure(String),
    /// A required resource was not found.
    NotFound(String),
}

impl std::fmt::Display for BlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockError::InvalidDevice(m) => write!(f, "invalid device: {m}"),
            BlockError::BadTable(m) => write!(f, "bad table: {m}"),
            BlockError::Geometry(m) => write!(f, "geometry error: {m}"),
            BlockError::Unrecognized => write!(f, "unrecognized signature"),
            BlockError::BadTransition(m) => write!(f, "bad transition: {m}"),
            BlockError::KeyFailure(m) => write!(f, "key failure: {m}"),
            BlockError::NotFound(m) => write!(f, "not found: {m}"),
        }
    }
}

impl std::error::Error for BlockError {}

impl From<BlockError> for os_kernel::Error {
    fn from(e: BlockError) -> Self {
        match e {
            BlockError::InvalidDevice(m) | BlockError::Geometry(m) => os_kernel::Error::Invalid(m),
            BlockError::BadTable(m) => os_kernel::Error::Parse(m),
            BlockError::Unrecognized => {
                os_kernel::Error::NotFound("no recognized signature".to_string())
            }
            BlockError::BadTransition(m) => os_kernel::Error::InvalidState(m),
            BlockError::KeyFailure(m) => os_kernel::Error::PermissionDenied(m),
            BlockError::NotFound(m) => os_kernel::Error::NotFound(m),
        }
    }
}

/// Crate-local result alias.
pub type Result<T> = core::result::Result<T, BlockError>;

/// Size, in bytes, of a "standard" logical block / sector used throughout the
/// crate when a device does not advertise its own sector size.
pub const DEFAULT_SECTOR_SIZE: u64 = 512;

/// Round `bytes` up to a whole number of sectors of `sector_size`.
pub fn sectors_for(bytes: u64, sector_size: u64) -> u64 {
    if sector_size == 0 {
        return 0;
    }
    bytes.div_ceil(sector_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_converts_to_core_error() {
        let e: os_kernel::Error = BlockError::InvalidDevice("/dev/null".to_string()).into();
        assert_eq!(e.kind(), "invalid");
        let e: os_kernel::Error = BlockError::BadTransition("x".to_string()).into();
        assert_eq!(e.kind(), "invalid_state");
        let e: os_kernel::Error = BlockError::Unrecognized.into();
        assert_eq!(e.kind(), "not_found");
    }

    #[test]
    fn sectors_round_up() {
        assert_eq!(sectors_for(0, 512), 0);
        assert_eq!(sectors_for(1, 512), 1);
        assert_eq!(sectors_for(512, 512), 1);
        assert_eq!(sectors_for(513, 512), 2);
        assert_eq!(sectors_for(100, 0), 0);
    }

    #[test]
    fn block_error_displays() {
        assert_eq!(
            BlockError::Geometry("overlap".to_string()).to_string(),
            "geometry error: overlap"
        );
        assert_eq!(
            BlockError::Unrecognized.to_string(),
            "unrecognized signature"
        );
    }
}
