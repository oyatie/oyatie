//! ISO9660 boot image assembly.
//!
//! Mirrors Talos `pkg/imager` ISO output: a hybrid ISO that boots both via
//! legacy BIOS (isolinux/GRUB El Torito) and UEFI (an EFI system partition
//! image embedded as an El Torito boot catalog entry). This module models the
//! ISO directory layout and computes the resulting image size.

use crate::profile::{Arch, SecureBootMode};

/// One file placed into the ISO filesystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IsoEntry {
    /// Path within the ISO (e.g. `/boot/vmlinuz`).
    pub path: String,
    /// File length in bytes.
    pub len: u64,
}

impl IsoEntry {
    /// Construct an entry.
    pub fn new(path: impl Into<String>, len: u64) -> IsoEntry {
        IsoEntry {
            path: path.into(),
            len,
        }
    }
}

/// The assembled ISO layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IsoImage {
    /// Target architecture.
    pub arch: Arch,
    /// Files placed in the ISO.
    pub entries: Vec<IsoEntry>,
    /// Whether a BIOS (El Torito legacy) boot entry exists.
    pub bios_boot: bool,
    /// Whether a UEFI (EFI system partition) boot entry exists.
    pub uefi_boot: bool,
    /// Volume identifier.
    pub volume_id: String,
}

/// ISO9660 logical sector size.
pub const SECTOR_SIZE: u64 = 2048;

impl IsoImage {
    /// Assemble a Talos ISO for the given architecture and SecureBoot mode.
    ///
    /// In SecureBoot mode no BIOS boot path is created (BIOS cannot do
    /// SecureBoot); a signed UKI is expected to be present at the EFI fallback
    /// path. Otherwise both a GRUB BIOS path and a UEFI path are present.
    pub fn assemble(
        arch: Arch,
        secureboot: SecureBootMode,
        kernel_len: u64,
        initramfs_len: u64,
        uki_len: Option<u64>,
    ) -> IsoImage {
        let mut entries = Vec::new();

        let bios_boot = if secureboot.is_enabled() {
            // SecureBoot ISO: only the signed UKI at the removable-media path.
            let uki = uki_len.unwrap_or(0);
            entries.push(IsoEntry::new(format!("/{}", arch.efi_boot_path()), uki));
            false
        } else {
            entries.push(IsoEntry::new("/boot/vmlinuz", kernel_len));
            entries.push(IsoEntry::new("/boot/initramfs.xz", initramfs_len));
            entries.push(IsoEntry::new("/boot/grub/grub.cfg", 512));
            entries.push(IsoEntry::new(
                format!("/{}", arch.efi_boot_path()),
                kernel_len + 4096,
            ));
            true
        };

        IsoImage {
            arch,
            entries,
            bios_boot,
            uefi_boot: true,
            volume_id: "TALOS".to_string(),
        }
    }

    /// Total payload bytes across all entries.
    pub fn payload_bytes(&self) -> u64 {
        self.entries.iter().map(|e| e.len).sum()
    }

    /// The final ISO size: payload rounded up to whole sectors plus a fixed
    /// 32 KiB system area + 16 sectors of volume descriptors and path tables.
    pub fn image_size(&self) -> u64 {
        const SYSTEM_AREA: u64 = 32 * 1024;
        const METADATA_SECTORS: u64 = 16;
        let payload_sectors = self.payload_bytes().div_ceil(SECTOR_SIZE);
        SYSTEM_AREA + (METADATA_SECTORS + payload_sectors) * SECTOR_SIZE
    }

    /// Whether the ISO can boot at all (at least one boot path).
    pub fn is_bootable(&self) -> bool {
        self.bios_boot || self.uefi_boot
    }

    /// Whether a given path exists in the ISO.
    pub fn contains(&self, path: &str) -> bool {
        self.entries.iter().any(|e| e.path == path)
    }
}
