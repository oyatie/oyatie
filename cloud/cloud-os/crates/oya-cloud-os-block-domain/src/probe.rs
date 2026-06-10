//! Turning raw device bytes into a [`ProbeResult`].
//!
//! Talos probes a freshly discovered block device to learn whether it carries a
//! partition table and/or a filesystem signature. The real implementation reads
//! sectors through `pread`/`ioctl`; here the device boundary is the
//! [`BlockReader`] trait, with an in-memory [`MemReader`] for tests.

use crate::filesystem::FilesystemType;
use crate::{BlockError, Result};

/// The "EFI PART" signature lives at LBA 1; the protective MBR's `0x55AA` lives
/// at offset 510 of LBA 0.
const MBR_SIG_OFFSET: usize = 510;
const GPT_HEADER_OFFSET: usize = 512; // start of LBA 1 with 512-byte sectors

/// Abstraction over a readable block device.
///
/// Models the subset of behaviour the probe needs: total size and positional
/// reads. The real machined uses a file descriptor; tests use [`MemReader`].
pub trait BlockReader {
    /// Total addressable size of the device in bytes.
    fn size(&self) -> u64;

    /// Read into `buf` starting at byte `offset`, returning the number of bytes
    /// read (which may be short at the end of the device).
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize>;

    /// Convenience: read exactly `len` bytes at `offset` into a fresh buffer,
    /// erroring if the device is too short.
    fn read_exact_vec(&self, offset: u64, len: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; len];
        let n = self.read_at(offset, &mut buf)?;
        if n < len {
            return Err(BlockError::BadTable("short read".to_string()));
        }
        Ok(buf)
    }
}

/// An in-memory block device backed by a byte buffer.
#[derive(Debug, Clone)]
pub struct MemReader {
    data: Vec<u8>,
}

impl MemReader {
    /// Wrap an owned byte buffer as a device.
    pub fn new(data: Vec<u8>) -> Self {
        MemReader { data }
    }

    /// A zero-filled device of `size` bytes.
    pub fn zeroed(size: usize) -> Self {
        MemReader {
            data: vec![0u8; size],
        }
    }

    /// Mutable access to the backing bytes, for test fixtures.
    pub fn bytes_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }
}

impl BlockReader for MemReader {
    fn size(&self) -> u64 {
        self.data.len() as u64
    }

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        let offset = usize::try_from(offset).unwrap_or(usize::MAX);
        if offset >= self.data.len() {
            return Ok(0);
        }
        let avail = self.data.len() - offset;
        let n = avail.min(buf.len());
        buf[..n].copy_from_slice(&self.data[offset..offset + n]);
        Ok(n)
    }
}

/// What kind of partition table (if any) a device carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableKind {
    /// A GUID partition table.
    Gpt,
    /// A legacy MBR / msdos table (protective MBR with no GPT, or real MBR).
    Mbr,
    /// No recognised partition table.
    None,
}

/// The outcome of probing a device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeResult {
    /// Detected partition-table kind.
    pub table: TableKind,
    /// Detected whole-device filesystem, if the device is not partitioned.
    pub filesystem: Option<FilesystemType>,
    /// Total device size in bytes.
    pub size: u64,
}

impl ProbeResult {
    /// Whether the device carries a usable partition table.
    pub fn is_partitioned(&self) -> bool {
        matches!(self.table, TableKind::Gpt | TableKind::Mbr)
    }

    /// Whether the device looks completely blank (no table, no filesystem).
    pub fn is_blank(&self) -> bool {
        self.table == TableKind::None && self.filesystem.is_none()
    }
}

/// Probe a device for partition-table and filesystem signatures.
///
/// Order of detection mirrors `blkid`/Talos: a GPT (signature at LBA 1) wins;
/// a protective/real MBR boot signature implies MBR; otherwise we fall back to
/// whole-device filesystem magic.
pub fn probe<R: BlockReader>(reader: &R) -> Result<ProbeResult> {
    let size = reader.size();
    if size < 512 {
        return Err(BlockError::Geometry(
            "device smaller than one sector".to_string(),
        ));
    }
    let size_usize = usize::try_from(size).unwrap_or(usize::MAX);

    // Read the first chunk covering MBR + primary GPT header.
    let head_len = (GPT_HEADER_OFFSET + 8).min(size_usize);
    let head = reader.read_exact_vec(0, head_len)?;

    let mut table = TableKind::None;
    if head.len() >= GPT_HEADER_OFFSET + 8
        && &head[GPT_HEADER_OFFSET..GPT_HEADER_OFFSET + 8] == b"EFI PART"
    {
        table = TableKind::Gpt;
    } else if head.len() > MBR_SIG_OFFSET + 1
        && head[MBR_SIG_OFFSET] == 0x55
        && head[MBR_SIG_OFFSET + 1] == 0xAA
    {
        table = TableKind::Mbr;
    }

    // Only look for a whole-device filesystem when there is no partition table.
    let filesystem = if table == TableKind::None {
        // Read enough to cover the iso9660 magic at 0x8001 if the device is big
        // enough; otherwise reuse the head bytes we already have.
        let probe_len = 0x8006usize.min(size_usize);
        if probe_len > head.len() {
            let buf = reader.read_exact_vec(0, probe_len)?;
            FilesystemType::detect(&buf)
        } else {
            FilesystemType::detect(&head)
        }
    } else {
        None
    };

    Ok(ProbeResult {
        table,
        filesystem,
        size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dev_with_gpt() -> MemReader {
        let mut d = MemReader::zeroed(1 << 20);
        d.bytes_mut()[GPT_HEADER_OFFSET..GPT_HEADER_OFFSET + 8].copy_from_slice(b"EFI PART");
        // Also set protective MBR sig.
        d.bytes_mut()[MBR_SIG_OFFSET] = 0x55;
        d.bytes_mut()[MBR_SIG_OFFSET + 1] = 0xAA;
        d
    }

    #[test]
    fn detects_gpt_over_mbr() {
        let d = dev_with_gpt();
        let r = probe(&d).unwrap();
        assert_eq!(r.table, TableKind::Gpt);
        assert!(r.is_partitioned());
        assert!(!r.is_blank());
        assert_eq!(r.filesystem, None);
    }

    #[test]
    fn detects_plain_mbr() {
        let mut d = MemReader::zeroed(1 << 20);
        d.bytes_mut()[MBR_SIG_OFFSET] = 0x55;
        d.bytes_mut()[MBR_SIG_OFFSET + 1] = 0xAA;
        let r = probe(&d).unwrap();
        assert_eq!(r.table, TableKind::Mbr);
    }

    #[test]
    fn detects_whole_device_filesystem() {
        let mut d = MemReader::zeroed(1 << 20);
        d.bytes_mut()[..4].copy_from_slice(b"XFSB");
        let r = probe(&d).unwrap();
        assert_eq!(r.table, TableKind::None);
        assert_eq!(r.filesystem, Some(FilesystemType::Xfs));
    }

    #[test]
    fn blank_device_is_blank() {
        let d = MemReader::zeroed(1 << 20);
        let r = probe(&d).unwrap();
        assert!(r.is_blank());
    }

    #[test]
    fn tiny_device_errors() {
        let d = MemReader::zeroed(16);
        assert!(matches!(probe(&d), Err(BlockError::Geometry(_))));
    }

    #[test]
    fn reader_reads_short_at_end() {
        let d = MemReader::new(vec![1, 2, 3, 4]);
        let mut buf = [0u8; 8];
        let n = d.read_at(2, &mut buf).unwrap();
        assert_eq!(n, 2);
        assert_eq!(&buf[..2], &[3, 4]);
        assert_eq!(d.read_at(99, &mut buf).unwrap(), 0);
    }
}
