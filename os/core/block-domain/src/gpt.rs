//! GUID partition tables.
//!
//! A pared-down model of a GPT: the protective-MBR/header invariants Talos
//! relies on plus a list of [`PartitionEntry`] records. Real GPT parsing reads
//! 512-byte LBAs and CRC-checks the header; here we model the logical structure
//! and the allocation/overlap rules used when Talos lays out a fresh disk.

use crate::partition::{Partition, PartitionRole};
use crate::{BlockError, Result};

/// The "EFI PART" signature at the start of a GPT header.
pub const GPT_SIGNATURE: &[u8; 8] = b"EFI PART";

/// LBA of the primary GPT header (LBA 0 is the protective MBR).
pub const PRIMARY_HEADER_LBA: u64 = 1;

/// Default number of partition entries a GPT reserves (128 * 128 bytes = 32
/// sectors of entry array).
pub const DEFAULT_MAX_ENTRIES: usize = 128;

/// A single entry in the GPT partition array.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartitionEntry {
    /// Partition type GUID (as a formatted string).
    pub type_guid: String,
    /// Unique partition GUID.
    pub part_guid: String,
    /// First LBA (inclusive).
    pub first_lba: u64,
    /// Last LBA (inclusive).
    pub last_lba: u64,
    /// UTF-16-derived partition name / label.
    pub name: String,
    /// GPT attribute bit flags.
    pub attributes: u64,
}

/// Attribute bit: required partition (system; do not delete).
pub const ATTR_REQUIRED: u64 = 1 << 0;
/// Attribute bit: legacy BIOS bootable.
pub const ATTR_LEGACY_BOOT: u64 = 1 << 2;

impl PartitionEntry {
    /// Construct an entry, validating that the LBA range is well-ordered.
    pub fn new(
        type_guid: impl Into<String>,
        part_guid: impl Into<String>,
        first_lba: u64,
        last_lba: u64,
        name: impl Into<String>,
    ) -> Result<Self> {
        if last_lba < first_lba {
            return Err(BlockError::Geometry(
                "last LBA before first LBA".to_string(),
            ));
        }
        Ok(PartitionEntry {
            type_guid: type_guid.into(),
            part_guid: part_guid.into(),
            first_lba,
            last_lba,
            name: name.into(),
            attributes: 0,
        })
    }

    /// Number of sectors covered by this entry.
    pub fn sector_count(&self) -> u64 {
        self.last_lba - self.first_lba + 1
    }

    /// Whether the required (system) attribute is set.
    pub fn is_required(&self) -> bool {
        self.attributes & ATTR_REQUIRED != 0
    }

    /// Whether this entry's LBA range overlaps `other`'s.
    pub fn overlaps(&self, other: &PartitionEntry) -> bool {
        self.first_lba <= other.last_lba && other.first_lba <= self.last_lba
    }
}

/// A logical GUID partition table for a disk of a known sector count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GptTable {
    /// Disk GUID.
    pub disk_guid: String,
    /// Total sectors on the disk (used to derive usable LBA bounds).
    pub total_sectors: u64,
    /// First usable LBA (after the header + entry array).
    pub first_usable_lba: u64,
    /// Last usable LBA (before the backup header + entry array).
    pub last_usable_lba: u64,
    /// Maximum number of entries the array can hold.
    pub max_entries: usize,
    /// The populated partition entries.
    pub entries: Vec<PartitionEntry>,
}

impl GptTable {
    /// Create an empty table for a disk with `total_sectors` sectors.
    ///
    /// Reserves LBA 0 (protective MBR), LBA 1 (primary header), 32 sectors of
    /// entry array, and a mirrored backup region at the end of the disk.
    pub fn new(disk_guid: impl Into<String>, total_sectors: u64) -> Result<Self> {
        // Header (1) + MBR (1) + entry array (32) on each side.
        const RESERVED_EACH_SIDE: u64 = 34;
        if total_sectors < RESERVED_EACH_SIDE * 2 + 1 {
            return Err(BlockError::Geometry("disk too small for a GPT".to_string()));
        }
        Ok(GptTable {
            disk_guid: disk_guid.into(),
            total_sectors,
            first_usable_lba: RESERVED_EACH_SIDE,
            last_usable_lba: total_sectors - RESERVED_EACH_SIDE - 1,
            max_entries: DEFAULT_MAX_ENTRIES,
            entries: Vec::new(),
        })
    }

    /// Add an entry, checking capacity, usable-range bounds and overlaps.
    pub fn add_entry(&mut self, entry: PartitionEntry) -> Result<()> {
        if self.entries.len() >= self.max_entries {
            return Err(BlockError::BadTable("partition array full".to_string()));
        }
        if entry.first_lba < self.first_usable_lba || entry.last_lba > self.last_usable_lba {
            return Err(BlockError::Geometry(
                "entry outside usable LBA range".to_string(),
            ));
        }
        for existing in &self.entries {
            if existing.overlaps(&entry) {
                return Err(BlockError::Geometry(format!(
                    "entry {:?} overlaps {:?}",
                    entry.name, existing.name
                )));
            }
        }
        self.entries.push(entry);
        Ok(())
    }

    /// Allocate the next `sectors`-long aligned partition at the lowest free
    /// LBA, append it as an entry, and return its (first, last) LBA.
    ///
    /// Alignment defaults to 2048 sectors (1 MiB at 512-byte sectors), matching
    /// what Talos/`sgdisk` use.
    pub fn allocate(
        &mut self,
        sectors: u64,
        type_guid: impl Into<String>,
        part_guid: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<(u64, u64)> {
        const ALIGN: u64 = 2048;
        if sectors == 0 {
            return Err(BlockError::Geometry("zero-length partition".to_string()));
        }
        // Find the highest end among existing entries, then align past it.
        let mut cursor = self.first_usable_lba;
        for e in &self.entries {
            if e.last_lba + 1 > cursor {
                cursor = e.last_lba + 1;
            }
        }
        let start = cursor.div_ceil(ALIGN) * ALIGN;
        let start = start.max(self.first_usable_lba);
        let end = start + sectors - 1;
        if end > self.last_usable_lba {
            return Err(BlockError::Geometry("not enough free space".to_string()));
        }
        let entry = PartitionEntry::new(type_guid, part_guid, start, end, name)?;
        self.add_entry(entry)?;
        Ok((start, end))
    }

    /// Total free sectors remaining inside the usable range (not counting
    /// fragmentation; this is `usable - sum(allocated)`).
    pub fn free_sectors(&self) -> u64 {
        let usable = self.last_usable_lba - self.first_usable_lba + 1;
        let used: u64 = self.entries.iter().map(PartitionEntry::sector_count).sum();
        usable.saturating_sub(used)
    }

    /// Find an entry by its (case-insensitive) name.
    pub fn find(&self, name: &str) -> Option<&PartitionEntry> {
        self.entries
            .iter()
            .find(|e| e.name.eq_ignore_ascii_case(name))
    }

    /// Project the table into [`Partition`] records, assigning 1-based numbers
    /// in entry order and deriving roles from the entry names.
    pub fn to_partitions(&self, disk_dev: &str, sector_size: u64) -> Vec<Partition> {
        self.entries
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let number = u32::try_from(i + 1).unwrap_or(u32::MAX);
                let role = PartitionRole::from_label(&e.name);
                let mut p = Partition::new(
                    format!("{disk_dev}{number}"),
                    number,
                    e.first_lba,
                    e.last_lba,
                    role,
                );
                p.sector_size = sector_size;
                p.label = Some(e.name.clone());
                p.uuid = Some(e.part_guid.clone());
                p
            })
            .collect()
    }

    /// Validate the whole table: ordered usable bounds and no overlaps.
    pub fn validate(&self) -> Result<()> {
        if self.first_usable_lba > self.last_usable_lba {
            return Err(BlockError::BadTable("inverted usable range".to_string()));
        }
        for (i, a) in self.entries.iter().enumerate() {
            for b in &self.entries[i + 1..] {
                if a.overlaps(b) {
                    return Err(BlockError::Geometry("overlapping entries".to_string()));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_tiny_disk() {
        assert!(GptTable::new("guid", 10).is_err());
        assert!(GptTable::new("guid", 100_000).is_ok());
    }

    #[test]
    fn allocate_packs_aligned_and_non_overlapping() {
        let mut t = GptTable::new("disk-guid", 1_000_000).unwrap();
        let (s1, e1) = t.allocate(2048, "EF00", "g1", "EFI").unwrap();
        let (s2, _e2) = t.allocate(4096, "8300", "g2", "STATE").unwrap();
        assert_eq!(s1 % 2048, 0);
        assert_eq!(s2 % 2048, 0);
        assert!(s2 > e1);
        t.validate().unwrap();
        assert_eq!(t.entries.len(), 2);
    }

    #[test]
    fn overlap_and_bounds_rejected() {
        let mut t = GptTable::new("g", 1_000_000).unwrap();
        let e = PartitionEntry::new("t", "p", t.first_usable_lba, t.first_usable_lba + 99, "A")
            .unwrap();
        t.add_entry(e).unwrap();
        let dup = PartitionEntry::new(
            "t",
            "p2",
            t.first_usable_lba + 50,
            t.first_usable_lba + 150,
            "B",
        )
        .unwrap();
        assert!(matches!(t.add_entry(dup), Err(BlockError::Geometry(_))));

        let oob = PartitionEntry::new("t", "p3", 0, 5, "C").unwrap();
        assert!(t.add_entry(oob).is_err());
    }

    #[test]
    fn free_space_runs_out() {
        let mut t = GptTable::new("g", 100_000).unwrap();
        let usable = t.last_usable_lba - t.first_usable_lba + 1;
        assert!(t.allocate(usable + 1, "t", "p", "X").is_err());
        let before = t.free_sectors();
        t.allocate(2048, "t", "p", "Y").unwrap();
        assert_eq!(t.free_sectors(), before - 2048);
    }

    #[test]
    fn projects_to_partitions_with_roles() {
        let mut t = GptTable::new("g", 1_000_000).unwrap();
        t.allocate(2048, "EF00", "g1", "EFI").unwrap();
        t.allocate(2048, "8300", "g2", "EPHEMERAL").unwrap();
        let parts = t.to_partitions("sda", 512);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].dev_name, "sda1");
        assert_eq!(parts[0].role, PartitionRole::Efi);
        assert_eq!(parts[1].role, PartitionRole::Ephemeral);
        assert!(t.find("ephemeral").is_some());
    }
}
