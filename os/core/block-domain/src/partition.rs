//! A single partition on a disk.
//!
//! Mirrors the well-known Talos partitions: the EFI system partition, the
//! `BIOS` boot partition, `BOOT`, `META`, `STATE` and the `EPHEMERAL` data
//! partition. Each partition carries its role, byte/sector geometry and an
//! optional filesystem.

use crate::filesystem::FilesystemType;
use crate::{BlockError, DEFAULT_SECTOR_SIZE, Result};

/// The semantic role a partition plays in a Talos install.
///
/// These mirror the partition labels Talos writes during install
/// (`pkg/machinery/constants`): `EFI`, `BIOS`, `BOOT`, `META`, `STATE` and
/// `EPHEMERAL`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionRole {
    /// The EFI system partition (FAT, mounted at `/boot/EFI`).
    Efi,
    /// The legacy BIOS boot partition (no filesystem; holds GRUB stage 1.5).
    Bios,
    /// The `/boot` partition holding kernel + initramfs.
    Boot,
    /// The `META` partition holding small key/value install metadata.
    Meta,
    /// The `STATE` partition holding machine config and secrets.
    State,
    /// The `EPHEMERAL` data partition mounted at `/var`.
    Ephemeral,
    /// A user-defined or unrecognised partition.
    Other,
}

impl PartitionRole {
    /// The canonical uppercase GPT partition label Talos uses for this role.
    pub fn label(self) -> &'static str {
        match self {
            PartitionRole::Efi => "EFI",
            PartitionRole::Bios => "BIOS",
            PartitionRole::Boot => "BOOT",
            PartitionRole::Meta => "META",
            PartitionRole::State => "STATE",
            PartitionRole::Ephemeral => "EPHEMERAL",
            PartitionRole::Other => "OTHER",
        }
    }

    /// Parse a GPT partition label back into a role. Matching is
    /// case-insensitive; unknown labels map to [`PartitionRole::Other`].
    pub fn from_label(label: &str) -> Self {
        match label.to_ascii_uppercase().as_str() {
            "EFI" => PartitionRole::Efi,
            "BIOS" => PartitionRole::Bios,
            "BOOT" => PartitionRole::Boot,
            "META" => PartitionRole::Meta,
            "STATE" => PartitionRole::State,
            "EPHEMERAL" => PartitionRole::Ephemeral,
            _ => PartitionRole::Other,
        }
    }

    /// The filesystem Talos formats this partition with by default, if any.
    /// `BIOS` carries raw bootloader bytes and has no filesystem.
    pub fn default_filesystem(self) -> Option<FilesystemType> {
        match self {
            PartitionRole::Efi => Some(FilesystemType::Vfat),
            PartitionRole::Bios => None,
            PartitionRole::Boot => Some(FilesystemType::Xfs),
            PartitionRole::Meta => None,
            PartitionRole::State => Some(FilesystemType::Xfs),
            PartitionRole::Ephemeral => Some(FilesystemType::Xfs),
            PartitionRole::Other => None,
        }
    }

    /// Whether this partition is system-critical and must never be wiped during
    /// an upgrade that preserves user state.
    pub fn is_system(self) -> bool {
        matches!(
            self,
            PartitionRole::Efi
                | PartitionRole::Bios
                | PartitionRole::Boot
                | PartitionRole::Meta
                | PartitionRole::State
        )
    }
}

/// A partition occupying a contiguous range of sectors on a disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Partition {
    /// Kernel device name, e.g. `sda1`, `nvme0n1p2`.
    pub dev_name: String,
    /// 1-based partition number within the table.
    pub number: u32,
    /// First sector (inclusive).
    pub start_sector: u64,
    /// Last sector (inclusive).
    pub end_sector: u64,
    /// Logical sector size in bytes.
    pub sector_size: u64,
    /// The partition's role.
    pub role: PartitionRole,
    /// The on-disk filesystem, once known.
    pub filesystem: Option<FilesystemType>,
    /// Filesystem label / partition name.
    pub label: Option<String>,
    /// Filesystem UUID, if formatted.
    pub uuid: Option<String>,
}

impl Partition {
    /// Build a partition from a sector range, deriving the default filesystem
    /// and label from the role.
    pub fn new(
        dev_name: impl Into<String>,
        number: u32,
        start_sector: u64,
        end_sector: u64,
        role: PartitionRole,
    ) -> Self {
        Partition {
            dev_name: dev_name.into(),
            number,
            start_sector,
            end_sector,
            sector_size: DEFAULT_SECTOR_SIZE,
            role,
            filesystem: role.default_filesystem(),
            label: Some(role.label().to_string()),
            uuid: None,
        }
    }

    /// Number of sectors this partition spans (inclusive of both endpoints).
    pub fn sector_count(&self) -> u64 {
        if self.end_sector < self.start_sector {
            return 0;
        }
        self.end_sector - self.start_sector + 1
    }

    /// Size of the partition in bytes.
    pub fn size(&self) -> u64 {
        self.sector_count() * self.sector_size
    }

    /// Whether this partition's sector range overlaps `other`'s.
    pub fn overlaps(&self, other: &Partition) -> bool {
        self.start_sector <= other.end_sector && other.start_sector <= self.end_sector
    }

    /// Validate the partition geometry and labelling.
    pub fn validate(&self) -> Result<()> {
        if self.dev_name.is_empty() {
            return Err(BlockError::InvalidDevice(
                "empty partition name".to_string(),
            ));
        }
        if self.number == 0 {
            return Err(BlockError::Geometry(
                "partition number must be 1-based".to_string(),
            ));
        }
        if self.sector_size == 0 || !self.sector_size.is_power_of_two() {
            return Err(BlockError::Geometry(
                "sector size must be a power of two".to_string(),
            ));
        }
        if self.end_sector < self.start_sector {
            return Err(BlockError::Geometry(
                "end sector precedes start sector".to_string(),
            ));
        }
        if let (Some(fs), Some(label)) = (self.filesystem, self.label.as_ref())
            && !fs.label_fits(label.len())
        {
            return Err(BlockError::BadTable(format!(
                "label {label:?} too long for {}",
                fs.as_str()
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_label_round_trip() {
        for role in [
            PartitionRole::Efi,
            PartitionRole::Bios,
            PartitionRole::Boot,
            PartitionRole::Meta,
            PartitionRole::State,
            PartitionRole::Ephemeral,
        ] {
            assert_eq!(PartitionRole::from_label(role.label()), role);
        }
        assert_eq!(PartitionRole::from_label("efi"), PartitionRole::Efi);
        assert_eq!(PartitionRole::from_label("nope"), PartitionRole::Other);
    }

    #[test]
    fn geometry_and_size() {
        let p = Partition::new("sda1", 1, 2048, 4095, PartitionRole::Efi);
        assert_eq!(p.sector_count(), 2048);
        assert_eq!(p.size(), 2048 * 512);
        assert_eq!(p.filesystem, Some(FilesystemType::Vfat));
        assert!(p.validate().is_ok());
    }

    #[test]
    fn overlap_detection() {
        let a = Partition::new("sda1", 1, 0, 99, PartitionRole::Bios);
        let b = Partition::new("sda2", 2, 100, 199, PartitionRole::Efi);
        let c = Partition::new("sda3", 3, 50, 150, PartitionRole::Boot);
        assert!(!a.overlaps(&b));
        assert!(a.overlaps(&c));
        assert!(c.overlaps(&b));
    }

    #[test]
    fn validation_rejects_bad_ranges_and_labels() {
        let mut p = Partition::new("sda1", 1, 100, 99, PartitionRole::State);
        assert!(matches!(p.validate(), Err(BlockError::Geometry(_))));
        p.end_sector = 200;
        p.number = 0;
        assert!(p.validate().is_err());
        p.number = 1;
        p.filesystem = Some(FilesystemType::Vfat);
        p.label = Some("THIS_LABEL_IS_TOO_LONG".to_string());
        assert!(matches!(p.validate(), Err(BlockError::BadTable(_))));
    }

    #[test]
    fn system_partitions_classified() {
        assert!(PartitionRole::State.is_system());
        assert!(PartitionRole::Efi.is_system());
        assert!(!PartitionRole::Ephemeral.is_system());
        assert_eq!(PartitionRole::Bios.default_filesystem(), None);
    }
}
