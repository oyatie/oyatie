//! Filesystem types and superblock-magic probing.
//!
//! Models the subset of filesystems Talos cares about on a node: `ext4` and
//! `xfs` for the data/state partitions, `vfat` for the EFI system partition,
//! `swap` for swap volumes, `virtiofs` for external shared volumes, and
//! `iso9660` for the read-only image used during install.

use crate::BlockError;

/// A recognised on-disk filesystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesystemType {
    /// The fourth extended filesystem.
    Ext4,
    /// SGI's XFS.
    Xfs,
    /// FAT (used for the EFI system partition).
    Vfat,
    /// Linux swap area.
    Swap,
    /// virtiofs external shared filesystem.
    Virtiofs,
    /// ISO 9660 optical filesystem.
    Iso9660,
}

impl FilesystemType {
    /// The lowercase identifier used by `mount -t` and in Talos config.
    pub fn as_str(self) -> &'static str {
        match self {
            FilesystemType::Ext4 => "ext4",
            FilesystemType::Xfs => "xfs",
            FilesystemType::Vfat => "vfat",
            FilesystemType::Swap => "swap",
            FilesystemType::Virtiofs => "virtiofs",
            FilesystemType::Iso9660 => "iso9660",
        }
    }

    /// Parse a filesystem identifier as it appears in machine config / mount
    /// options. `msdos` and `fat` are accepted as aliases for `vfat`.
    pub fn parse(s: &str) -> Result<Self, BlockError> {
        match s {
            "ext4" => Ok(FilesystemType::Ext4),
            "xfs" => Ok(FilesystemType::Xfs),
            "vfat" | "fat" | "msdos" => Ok(FilesystemType::Vfat),
            "swap" => Ok(FilesystemType::Swap),
            "virtiofs" => Ok(FilesystemType::Virtiofs),
            "iso9660" => Ok(FilesystemType::Iso9660),
            other => Err(BlockError::BadTable(format!(
                "unknown filesystem {other:?}"
            ))),
        }
    }

    /// Whether the filesystem supports growing in place (`resize2fs` / `xfs_growfs`).
    pub fn supports_grow(self) -> bool {
        matches!(self, FilesystemType::Ext4 | FilesystemType::Xfs)
    }

    /// Whether the filesystem is inherently read-only on a node.
    pub fn is_read_only(self) -> bool {
        matches!(self, FilesystemType::Iso9660)
    }

    /// Whether a label of length `len` bytes is valid for this filesystem.
    ///
    /// `vfat` labels are limited to 11 bytes, `xfs` to 12, `ext4`/`swap` to 16.
    pub fn label_fits(self, len: usize) -> bool {
        let max = match self {
            FilesystemType::Vfat => 11,
            FilesystemType::Xfs => 12,
            FilesystemType::Ext4 | FilesystemType::Swap => 16,
            FilesystemType::Iso9660 => 32,
            FilesystemType::Virtiofs => usize::MAX,
        };
        len <= max
    }

    /// Attempt to identify a filesystem by inspecting magic bytes in a buffer
    /// that begins at the start of the partition.
    ///
    /// This recognises the real on-disk signatures at their canonical offsets:
    /// * ext4: magic `0x53 0xEF` at byte offset 0x438 (1080).
    /// * xfs: ASCII `XFSB` at offset 0.
    /// * iso9660: `CD001` at offset 0x8001 (sector 16).
    /// * vfat: the `0x55 0xAA` boot signature at offset 510.
    pub fn detect(buf: &[u8]) -> Option<FilesystemType> {
        if buf.len() >= 4 && &buf[0..4] == b"XFSB" {
            return Some(FilesystemType::Xfs);
        }
        if buf.len() >= 0x8006 && &buf[0x8001..0x8006] == b"CD001" {
            return Some(FilesystemType::Iso9660);
        }
        if buf.len() >= 0x43A && buf[0x438] == 0x53 && buf[0x439] == 0xEF {
            return Some(FilesystemType::Ext4);
        }
        if buf.len() >= 4096
            && (&buf[4086..4096] == b"SWAPSPACE2" || &buf[4086..4096] == b"SWAP-SPACE")
        {
            return Some(FilesystemType::Swap);
        }
        if buf.len() >= 512 && buf[510] == 0x55 && buf[511] == 0xAA {
            return Some(FilesystemType::Vfat);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_aliases() {
        assert_eq!(FilesystemType::parse("swap").unwrap(), FilesystemType::Swap);
        assert_eq!(FilesystemType::Swap.as_str(), "swap");
        assert_eq!(
            FilesystemType::parse("virtiofs").unwrap(),
            FilesystemType::Virtiofs
        );
        assert_eq!(FilesystemType::Virtiofs.as_str(), "virtiofs");
        assert_eq!(FilesystemType::parse("ext4").unwrap(), FilesystemType::Ext4);
        assert_eq!(FilesystemType::parse("fat").unwrap(), FilesystemType::Vfat);
        assert_eq!(
            FilesystemType::parse("msdos").unwrap(),
            FilesystemType::Vfat
        );
        assert!(FilesystemType::parse("zfs").is_err());
        assert_eq!(FilesystemType::Vfat.as_str(), "vfat");
    }

    #[test]
    fn capability_flags() {
        assert!(FilesystemType::Ext4.supports_grow());
        assert!(!FilesystemType::Swap.supports_grow());
        assert!(!FilesystemType::Swap.is_read_only());
        assert!(!FilesystemType::Vfat.supports_grow());
        assert!(FilesystemType::Iso9660.is_read_only());
        assert!(!FilesystemType::Xfs.is_read_only());
    }

    #[test]
    fn label_limits() {
        assert!(FilesystemType::Vfat.label_fits(11));
        assert!(!FilesystemType::Vfat.label_fits(12));
        assert!(FilesystemType::Ext4.label_fits(16));
        assert!(FilesystemType::Swap.label_fits(16));
        assert!(!FilesystemType::Swap.label_fits(17));
    }

    #[test]
    fn detect_by_magic() {
        // xfs at offset 0
        let mut xfs = vec![0u8; 8];
        xfs[..4].copy_from_slice(b"XFSB");
        assert_eq!(FilesystemType::detect(&xfs), Some(FilesystemType::Xfs));

        // ext4 magic at 0x438
        let mut ext = vec![0u8; 0x440];
        ext[0x438] = 0x53;
        ext[0x439] = 0xEF;
        assert_eq!(FilesystemType::detect(&ext), Some(FilesystemType::Ext4));

        // vfat boot signature
        let mut fat = vec![0u8; 512];
        fat[510] = 0x55;
        fat[511] = 0xAA;
        assert_eq!(FilesystemType::detect(&fat), Some(FilesystemType::Vfat));

        // Linux swap signature at the end of the first page.
        let mut swap = vec![0u8; 4096];
        swap[4086..4096].copy_from_slice(b"SWAPSPACE2");
        assert_eq!(FilesystemType::detect(&swap), Some(FilesystemType::Swap));

        // nothing
        let blank = vec![0u8; 4096];
        assert_eq!(FilesystemType::detect(&blank), None);
    }
}
