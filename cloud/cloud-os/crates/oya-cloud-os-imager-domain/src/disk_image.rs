//! Raw disk image assembly and partition layout.
//!
//! Mirrors Talos `pkg/imager` disk-image output: a GPT-partitioned raw disk
//! with the Talos partition set (EFI, BIOS, BOOT, META, STATE, EPHEMERAL). This
//! module computes the partition table given a disk size and an optional board
//! overlay (which dictates the first-partition offset), and models conversion
//! to a cloud disk format.

use crate::output::DiskFormat;
use crate::overlay::Overlay;

/// A single GPT partition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Partition {
    /// Partition label (Talos uses `EFI`, `BIOS`, `BOOT`, `META`, `STATE`,
    /// `EPHEMERAL`).
    pub label: String,
    /// Start offset in bytes.
    pub start: u64,
    /// Size in bytes (`EPHEMERAL` consumes the remainder).
    pub size: u64,
}

impl Partition {
    /// Exclusive end offset.
    pub fn end(&self) -> u64 {
        self.start + self.size
    }
}

/// The Talos partition sizes (bytes). These mirror the constants in Talos's
/// partition package closely enough to model layout and overflow.
const EFI_SIZE: u64 = 100 * 1024 * 1024; // 100 MiB
const BIOS_SIZE: u64 = 1024 * 1024; // 1 MiB
const BOOT_SIZE: u64 = 1024 * 1024 * 1024; // 1000 MiB
const META_SIZE: u64 = 1024 * 1024; // 1 MiB
const STATE_SIZE: u64 = 100 * 1024 * 1024; // 100 MiB
/// GPT primary header + partition entries reserve at the head of the disk.
const GPT_RESERVE: u64 = 33 * 512;
/// GPT secondary (backup) header reserve at the tail of the disk.
const GPT_BACKUP: u64 = 33 * 512;

/// An assembled raw disk image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskImage {
    /// Total disk size in bytes.
    pub size: u64,
    /// Partitions in on-disk order.
    pub partitions: Vec<Partition>,
    /// Container format.
    pub format: DiskFormat,
}

impl DiskImage {
    /// Assemble a Talos disk image of `size` bytes, applying `overlay` if
    /// present (its boot-partition offset shifts the first partition to make
    /// room for board firmware).
    ///
    /// Returns an error if the requested size cannot hold the fixed system
    /// partitions plus a non-empty EPHEMERAL partition.
    pub fn assemble(
        size: u64,
        format: DiskFormat,
        overlay: Option<&Overlay>,
    ) -> Result<DiskImage, DiskImageError> {
        let first_offset = overlay
            .map(|o| o.boot_partition_offset.max(GPT_RESERVE))
            .unwrap_or(GPT_RESERVE);

        let mut cursor = first_offset;
        let mut partitions = Vec::new();

        let mut push = |label: &str, sz: u64, cursor: &mut u64| {
            partitions.push(Partition {
                label: label.to_string(),
                start: *cursor,
                size: sz,
            });
            *cursor += sz;
        };

        push("EFI", EFI_SIZE, &mut cursor);
        push("BIOS", BIOS_SIZE, &mut cursor);
        push("BOOT", BOOT_SIZE, &mut cursor);
        push("META", META_SIZE, &mut cursor);
        push("STATE", STATE_SIZE, &mut cursor);

        let fixed_end = cursor + GPT_BACKUP;
        if size <= fixed_end {
            return Err(DiskImageError::TooSmall {
                size,
                minimum: fixed_end + 1,
            });
        }

        // EPHEMERAL consumes the remainder.
        let ephemeral_size = size - cursor - GPT_BACKUP;
        partitions.push(Partition {
            label: "EPHEMERAL".to_string(),
            start: cursor,
            size: ephemeral_size,
        });

        Ok(DiskImage {
            size,
            partitions,
            format,
        })
    }

    /// Look a partition up by label.
    pub fn partition(&self, label: &str) -> Option<&Partition> {
        self.partitions.iter().find(|p| p.label == label)
    }

    /// Verify no two partitions overlap and all fit within the disk.
    pub fn check_layout(&self) -> Result<(), DiskImageError> {
        let mut prev_end = 0u64;
        for p in &self.partitions {
            if p.start < prev_end {
                return Err(DiskImageError::Overlap(p.label.clone()));
            }
            if p.end() > self.size {
                return Err(DiskImageError::OutOfBounds(p.label.clone()));
            }
            prev_end = p.end();
        }
        Ok(())
    }

    /// Convert the image to a different container format. Raw->raw is a no-op;
    /// other conversions are modeled as a format change (the byte layout is an
    /// OS boundary handled by `qemu-img` in the real imager).
    pub fn convert(&mut self, format: DiskFormat) {
        self.format = format;
    }

    /// The on-disk filename for the image given an architecture name.
    pub fn filename(&self, arch: &str) -> String {
        format!("talos-{arch}.{}", self.format.extension())
    }
}

/// A disk-image assembly failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiskImageError {
    /// The disk is too small to hold the fixed partitions plus EPHEMERAL.
    TooSmall {
        /// Requested size.
        size: u64,
        /// Minimum workable size.
        minimum: u64,
    },
    /// Two partitions overlap.
    Overlap(String),
    /// A partition extends beyond the disk.
    OutOfBounds(String),
}

impl std::fmt::Display for DiskImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiskImageError::TooSmall { size, minimum } => {
                write!(f, "disk size {size} too small (need at least {minimum})")
            }
            DiskImageError::Overlap(l) => write!(f, "partition '{l}' overlaps its predecessor"),
            DiskImageError::OutOfBounds(l) => write!(f, "partition '{l}' extends past disk end"),
        }
    }
}

impl std::error::Error for DiskImageError {}
