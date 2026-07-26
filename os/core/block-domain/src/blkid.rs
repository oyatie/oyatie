//! blkid-style detailed device identification.
//!
//! Where [`crate::probe`] answers "is this a GPT/MBR/filesystem", `blkid`
//! extracts the richer metadata Talos surfaces on the `block.DeviceStatus`
//! resource: the filesystem UUID, the volume label, the block/sector size and a
//! `usage` classification (`filesystem`, `crypto`, `partitiontable`). The
//! superblock-field offsets modelled here are the real on-disk offsets for each
//! filesystem family.

use crate::filesystem::FilesystemType;
use crate::luks::LUKS_MAGIC;
use crate::probe::{BlockReader, TableKind};
use crate::{BlockError, Result};

/// How a device or partition is being used, mirroring blkid's `USAGE` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Usage {
    /// Carries a filesystem.
    Filesystem,
    /// Carries a LUKS/crypto signature.
    Crypto,
    /// Carries a partition table.
    PartitionTable,
    /// Nothing recognised.
    Unknown,
}

/// The detailed identification of a single device or partition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlkidInfo {
    /// Detected filesystem, if any.
    pub filesystem: Option<FilesystemType>,
    /// Filesystem UUID, if extractable.
    pub uuid: Option<String>,
    /// Volume label, if extractable.
    pub label: Option<String>,
    /// Usage classification.
    pub usage: Usage,
    /// Filesystem block size in bytes, if known.
    pub block_size: Option<u32>,
}

impl BlkidInfo {
    /// An info record describing a blank device.
    pub fn blank() -> Self {
        BlkidInfo {
            filesystem: None,
            uuid: None,
            label: None,
            usage: Usage::Unknown,
            block_size: None,
        }
    }

    /// Whether anything was recognised at all.
    pub fn is_recognized(&self) -> bool {
        self.usage != Usage::Unknown
    }
}

/// Render the 16-byte big-endian field at `buf[off..off+16]` as a canonical
/// lower-case UUID string (`8-4-4-4-12`). Returns `None` if all-zero.
fn format_uuid(buf: &[u8], off: usize) -> Option<String> {
    if off + 16 > buf.len() {
        return None;
    }
    let raw = &buf[off..off + 16];
    if raw.iter().all(|&b| b == 0) {
        return None;
    }
    let mut s = String::with_capacity(36);
    for (i, &b) in raw.iter().enumerate() {
        if matches!(i, 4 | 6 | 8 | 10) {
            s.push('-');
        }
        s.push(nibble(b >> 4));
        s.push(nibble(b & 0xf));
    }
    Some(s)
}

fn nibble(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        _ => (b'a' + (n - 10)) as char,
    }
}

/// Read a NUL/space-trimmed ASCII label from `buf[off..off+len]`.
fn read_label(buf: &[u8], off: usize, len: usize) -> Option<String> {
    if off + len > buf.len() {
        return None;
    }
    let raw = &buf[off..off + len];
    let trimmed: Vec<u8> = raw.iter().copied().take_while(|&b| b != 0).collect();
    let s: String = trimmed
        .iter()
        .map(|&b| b as char)
        .collect::<String>()
        .trim()
        .to_string();
    if s.is_empty() { None } else { Some(s) }
}

/// Probe `reader` for blkid-style metadata.
///
/// Recognises (in order): LUKS, GPT/MBR partition tables, then filesystem
/// superblocks (xfs, ext4, vfat). For filesystems it extracts the UUID and
/// label from their canonical superblock offsets:
/// * xfs: magic `XFSB` @0, UUID @0x20, label @0x6c (12 bytes), block size @4 (BE u32).
/// * ext4: magic @0x438, UUID @0x468 (1128), label @0x478 (16 bytes).
/// * vfat: boot sig @510; volume id @0x43 (FAT16) and label @0x47.
pub fn blkid<R: BlockReader>(reader: &R) -> Result<BlkidInfo> {
    let size = reader.size();
    if size < 512 {
        return Err(BlockError::Geometry(
            "device smaller than one sector".to_string(),
        ));
    }
    // Read a generous head covering ext4's superblock (offset 0x400 + 0x400).
    let size_usize = usize::try_from(size).unwrap_or(usize::MAX);
    let head_len = (0x8006usize).min(size_usize).max(0x900);
    let head = reader.read_at_vec(0, head_len)?;

    // LUKS signature at offset 0.
    if head.len() >= LUKS_MAGIC.len() && head[..LUKS_MAGIC.len()] == LUKS_MAGIC {
        let uuid = read_label(&head, 0xa8, 40); // LUKS2 stores an ASCII UUID @0xa8
        return Ok(BlkidInfo {
            filesystem: None,
            uuid,
            label: None,
            usage: Usage::Crypto,
            block_size: None,
        });
    }

    // GPT.
    if head.len() >= 512 + 8 && &head[512..512 + 8] == b"EFI PART" {
        return Ok(BlkidInfo {
            usage: Usage::PartitionTable,
            ..BlkidInfo::blank()
        });
    }

    // XFS.
    if head.len() >= 4 && &head[0..4] == b"XFSB" {
        let uuid = format_uuid(&head, 0x20);
        let label = read_label(&head, 0x6c, 12);
        let block_size = if head.len() >= 8 {
            Some(u32::from_be_bytes([head[4], head[5], head[6], head[7]]))
        } else {
            None
        };
        return Ok(BlkidInfo {
            filesystem: Some(FilesystemType::Xfs),
            uuid,
            label,
            usage: Usage::Filesystem,
            block_size,
        });
    }

    // ext4.
    if head.len() >= 0x43A && head[0x438] == 0x53 && head[0x439] == 0xEF {
        let uuid = format_uuid(&head, 0x468);
        let label = read_label(&head, 0x478, 16);
        // s_log_block_size @0x418 (u32 LE): block size = 1024 << value.
        let block_size = if head.len() >= 0x41C {
            let log = u32::from_le_bytes([head[0x418], head[0x419], head[0x41a], head[0x41b]]);
            Some(1024u32 << log.min(6))
        } else {
            None
        };
        return Ok(BlkidInfo {
            filesystem: Some(FilesystemType::Ext4),
            uuid,
            label,
            usage: Usage::Filesystem,
            block_size,
        });
    }

    // ISO9660.
    if head.len() >= 0x8006 && &head[0x8001..0x8006] == b"CD001" {
        return Ok(BlkidInfo {
            filesystem: Some(FilesystemType::Iso9660),
            usage: Usage::Filesystem,
            ..BlkidInfo::blank()
        });
    }

    // vfat — boot signature at 510, label at 0x47 (FAT16) for 11 bytes.
    if head.len() >= 512 && head[510] == 0x55 && head[511] == 0xAA {
        let label = read_label(&head, 0x47, 11).filter(|l| l != "NO NAME");
        return Ok(BlkidInfo {
            filesystem: Some(FilesystemType::Vfat),
            label,
            usage: Usage::Filesystem,
            ..BlkidInfo::blank()
        });
    }

    Ok(BlkidInfo::blank())
}

/// Map a [`TableKind`] into the blkid [`Usage`] it would imply.
pub fn usage_for_table(table: TableKind) -> Usage {
    match table {
        TableKind::Gpt | TableKind::Mbr => Usage::PartitionTable,
        TableKind::None => Usage::Unknown,
    }
}

/// Extension to read a fresh owned buffer of up to `len` bytes (short reads OK).
trait ReadAtVec {
    fn read_at_vec(&self, offset: u64, len: usize) -> Result<Vec<u8>>;
}

impl<R: BlockReader> ReadAtVec for R {
    fn read_at_vec(&self, offset: u64, len: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; len];
        let n = self.read_at(offset, &mut buf)?;
        buf.truncate(n);
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::MemReader;

    fn dev(size: usize) -> MemReader {
        MemReader::zeroed(size)
    }

    #[test]
    fn detects_xfs_with_uuid_label_and_blocksize() {
        let mut d = dev(1 << 20);
        {
            let b = d.bytes_mut();
            b[..4].copy_from_slice(b"XFSB");
            // block size 4096 big-endian @4.
            b[4..8].copy_from_slice(&4096u32.to_be_bytes());
            // UUID @0x20.
            for i in 0..16 {
                b[0x20 + i] = (i as u8) + 1;
            }
            // label @0x6c.
            b[0x6c..0x6c + 5].copy_from_slice(b"STATE");
        }
        let info = blkid(&d).unwrap();
        assert_eq!(info.filesystem, Some(FilesystemType::Xfs));
        assert_eq!(info.usage, Usage::Filesystem);
        assert_eq!(info.block_size, Some(4096));
        assert_eq!(info.label.as_deref(), Some("STATE"));
        assert_eq!(
            info.uuid.as_deref(),
            Some("01020304-0506-0708-090a-0b0c0d0e0f10")
        );
    }

    #[test]
    fn detects_ext4_uuid_and_blocksize() {
        let mut d = dev(1 << 20);
        {
            let b = d.bytes_mut();
            b[0x438] = 0x53;
            b[0x439] = 0xEF;
            // s_log_block_size = 2 -> 4096.
            b[0x418..0x41c].copy_from_slice(&2u32.to_le_bytes());
            for i in 0..16 {
                b[0x468 + i] = 0xAB;
            }
            b[0x478..0x478 + 4].copy_from_slice(b"root");
        }
        let info = blkid(&d).unwrap();
        assert_eq!(info.filesystem, Some(FilesystemType::Ext4));
        assert_eq!(info.block_size, Some(4096));
        assert_eq!(info.label.as_deref(), Some("root"));
        assert!(info.uuid.is_some());
    }

    #[test]
    fn detects_luks_as_crypto() {
        let mut d = dev(1 << 20);
        d.bytes_mut()[..6].copy_from_slice(&LUKS_MAGIC);
        let info = blkid(&d).unwrap();
        assert_eq!(info.usage, Usage::Crypto);
        assert_eq!(info.filesystem, None);
    }

    #[test]
    fn detects_gpt_as_partition_table() {
        let mut d = dev(1 << 20);
        d.bytes_mut()[512..520].copy_from_slice(b"EFI PART");
        let info = blkid(&d).unwrap();
        assert_eq!(info.usage, Usage::PartitionTable);
        assert!(info.is_recognized());
    }

    #[test]
    fn vfat_label_and_no_name_filtered() {
        let mut d = dev(1 << 20);
        {
            let b = d.bytes_mut();
            b[510] = 0x55;
            b[511] = 0xAA;
            b[0x47..0x47 + 3].copy_from_slice(b"EFI");
        }
        let info = blkid(&d).unwrap();
        assert_eq!(info.filesystem, Some(FilesystemType::Vfat));
        assert_eq!(info.label.as_deref(), Some("EFI"));

        // "NO NAME" is the FAT placeholder and must be filtered out.
        let mut d2 = dev(1 << 20);
        {
            let b = d2.bytes_mut();
            b[510] = 0x55;
            b[511] = 0xAA;
            b[0x47..0x47 + 7].copy_from_slice(b"NO NAME");
        }
        assert_eq!(blkid(&d2).unwrap().label, None);
    }

    #[test]
    fn blank_device_unrecognized() {
        let d = dev(1 << 20);
        let info = blkid(&d).unwrap();
        assert!(!info.is_recognized());
        assert_eq!(info.usage, Usage::Unknown);
    }

    #[test]
    fn tiny_device_errors() {
        let d = dev(16);
        assert!(matches!(blkid(&d), Err(BlockError::Geometry(_))));
    }

    #[test]
    fn all_zero_uuid_is_none() {
        let buf = vec![0u8; 32];
        assert_eq!(format_uuid(&buf, 0), None);
    }

    #[test]
    fn usage_for_table_mapping() {
        assert_eq!(usage_for_table(TableKind::Gpt), Usage::PartitionTable);
        assert_eq!(usage_for_table(TableKind::None), Usage::Unknown);
    }
}
