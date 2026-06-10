//! The standard Talos on-disk partition layout.
//!
//! Mirrors `internal/app/machined/pkg/runtime/v1alpha1/bootloader` and the
//! install partition table Talos lays down: a small EFI system partition, a
//! BIOS-boot partition (when not pure-UEFI), a BOOT partition, the META and
//! STATE partitions, and finally an EPHEMERAL partition consuming the rest of
//! the disk. This module centralises the partition *sizes*, *type GUIDs* and
//! the function that allocates the whole layout into a [`GptTable`].

use crate::gpt::{ATTR_LEGACY_BOOT, ATTR_REQUIRED, GptTable, PartitionEntry};
use crate::partition::PartitionRole;
use crate::volume::{VolumeConfig, VolumeType};
use crate::{BlockError, Result};

/// Well-known GPT partition-type GUIDs Talos stamps into the table.
pub mod type_guid {
    /// EFI System Partition.
    pub const EFI: &str = "c12a7328-f81f-11d2-ba4b-00a0c93ec93b";
    /// BIOS boot partition (GRUB embedding area).
    pub const BIOS_BOOT: &str = "21686148-6449-6e6f-744e-656564454649";
    /// Generic Linux filesystem data (used for BOOT/META/STATE/EPHEMERAL).
    pub const LINUX_FILESYSTEM: &str = "0fc63daf-8483-4772-8e79-3d69d8477de4";
    /// Linux swap partition type (`partition.LinkSwap` upstream).
    pub const LINUX_SWAP: &str = "0657fd6d-a4ab-43c4-84e5-0933c84b4f4f";
}

/// Default sizes (in bytes) for the fixed-size Talos system partitions.
///
/// These mirror the constants in `pkg/machinery/constants`. EPHEMERAL has no
/// fixed size — it grows to fill the remaining space.
pub mod size {
    /// EFI system partition — 100 MiB.
    pub const EFI: u64 = 100 * 1024 * 1024;
    /// BIOS boot partition — 1 MiB (GRUB core image).
    pub const BIOS_BOOT: u64 = 1024 * 1024;
    /// BOOT partition — 1000 MiB (kernel + initramfs across A/B).
    pub const BOOT: u64 = 1000 * 1024 * 1024;
    /// META partition — 1 MiB of key/value install metadata.
    pub const META: u64 = 1024 * 1024;
    /// STATE partition — 100 MiB of machine config + secrets.
    pub const STATE: u64 = 100 * 1024 * 1024;
}

/// Talos does not grow an existing partition when the free contiguous extent is
/// at most this size. The threshold is checked before applying an optional
/// `maxSize` cap, matching upstream `volumes.Grow`.
pub const MIN_PARTITION_GROWTH_BYTES: u64 = 1024 * 1024;

/// A description of one partition to lay down, before LBA allocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartitionSpec {
    /// The semantic role.
    pub role: PartitionRole,
    /// GPT partition-type GUID.
    pub type_guid: String,
    /// Requested size in bytes, or `None` to grow to fill the disk.
    pub size_bytes: Option<u64>,
    /// Optional absolute cap for a grow partition's initial allocation.
    pub max_size_bytes: Option<u64>,
    /// GPT attribute flags to set.
    pub attributes: u64,
}

impl PartitionSpec {
    /// Fixed-size partition for `role` with `size_bytes`.
    pub fn fixed(role: PartitionRole, type_guid: &str, size_bytes: u64, attributes: u64) -> Self {
        PartitionSpec {
            role,
            type_guid: type_guid.to_string(),
            size_bytes: Some(size_bytes),
            max_size_bytes: None,
            attributes,
        }
    }

    /// Grow-to-fill partition for `role`.
    pub fn grow(role: PartitionRole, type_guid: &str) -> Self {
        PartitionSpec {
            role,
            type_guid: type_guid.to_string(),
            size_bytes: None,
            max_size_bytes: None,
            attributes: 0,
        }
    }

    /// Grow-to-fill partition capped by an absolute byte size.
    pub fn grow_bounded(role: PartitionRole, type_guid: &str, max_size_bytes: u64) -> Self {
        PartitionSpec {
            role,
            type_guid: type_guid.to_string(),
            size_bytes: None,
            max_size_bytes: Some(max_size_bytes),
            attributes: 0,
        }
    }
}

/// Pure growth decision for an existing partition-backed volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartitionGrowthPlan {
    /// Current partition size in bytes.
    pub current_size: u64,
    /// Contiguous bytes available after the partition before `maxSize` is
    /// applied.
    pub available_growth: u64,
    /// Bytes the partition should grow by.
    pub grow_by: u64,
    /// Size after applying the growth decision.
    pub target_size: u64,
}

impl PartitionGrowthPlan {
    /// Whether the plan changes the partition geometry.
    pub fn grows(self) -> bool {
        self.grow_by > 0
    }
}

/// Resolve the initial size for a newly-created partition-backed volume.
///
/// Mirrors Talos `CreatePartition`: start from `minSize`, require the selected
/// disk extent to fit that minimum, then allocate all available space unless
/// `maxSize` resolves to a non-zero cap below the available extent.
pub fn plan_partition_create_size(config: &VolumeConfig, available_bytes: u64) -> Result<u64> {
    if config.volume_type != VolumeType::Partition {
        return Err(BlockError::InvalidDevice(
            "partition create plan requires a partition volume".to_string(),
        ));
    }
    config.validate()?;
    if available_bytes < config.min_size {
        return Err(BlockError::Geometry("not enough space on disk".to_string()));
    }
    match config.resolve_max_size(available_bytes)? {
        0 => Ok(available_bytes),
        max_size if max_size < available_bytes => Ok(max_size),
        _ => Ok(available_bytes),
    }
}

/// Plan growth for an already-located partition-backed volume.
///
/// Mirrors Talos `Grow`: only partition volumes with `grow: true` are eligible;
/// raw available growth at or below 1 MiB is ignored; otherwise the growth
/// amount is bounded by the remaining absolute `maxSize` allowance. Talos
/// v1.13 resolves relative/negative max sizes for create-time allocation, but
/// the grow controller reads only the raw absolute `MaxSize` field.
pub fn plan_partition_growth(
    config: &VolumeConfig,
    current_size: u64,
    available_growth: u64,
) -> Result<PartitionGrowthPlan> {
    if config.volume_type != VolumeType::Partition || config.grow != Some(true) {
        return Ok(PartitionGrowthPlan {
            current_size,
            available_growth,
            grow_by: 0,
            target_size: current_size,
        });
    }
    config.validate()?;
    if available_growth <= MIN_PARTITION_GROWTH_BYTES {
        return Ok(PartitionGrowthPlan {
            current_size,
            available_growth,
            grow_by: 0,
            target_size: current_size,
        });
    }

    let max_size = config.max_size.unwrap_or(0);
    if max_size != 0 && current_size >= max_size {
        return Ok(PartitionGrowthPlan {
            current_size,
            available_growth,
            grow_by: 0,
            target_size: current_size,
        });
    }

    let grow_by = if max_size != 0 {
        available_growth.min(max_size - current_size)
    } else {
        available_growth
    };
    let target_size = current_size
        .checked_add(grow_by)
        .ok_or_else(|| BlockError::Geometry("partition growth overflows size".to_string()))?;
    Ok(PartitionGrowthPlan {
        current_size,
        available_growth,
        grow_by,
        target_size,
    })
}

/// Apply the partition create plan to an in-memory GPT table.
///
/// This is the side-effect boundary for a newly-provisioned partition-backed
/// volume: resolve Talos `minSize`/`maxSize` semantics against the currently
/// allocatable GPT extent, allocate a Linux filesystem partition, and return the
/// GPT entry that would be written. The real runtime can replace this in-memory
/// mutation with an `sgdisk`/kernel implementation without changing the
/// source-guided planning semantics.
pub fn apply_partition_create_plan(
    table: &mut GptTable,
    config: &VolumeConfig,
    sector_size: u64,
    part_guid: impl Into<String>,
) -> Result<PartitionEntry> {
    if sector_size == 0 {
        return Err(BlockError::Geometry("zero sector size".to_string()));
    }
    if config.volume_type != VolumeType::Partition {
        return Err(BlockError::InvalidDevice(
            "partition create plan requires a partition volume".to_string(),
        ));
    }

    let available_bytes = remaining_aligned_sectors(table)
        .checked_mul(sector_size)
        .ok_or_else(|| BlockError::Geometry("available partition extent overflows".to_string()))?;
    let size_bytes = plan_partition_create_size(config, available_bytes)?;
    let sectors = size_bytes.div_ceil(sector_size);
    let label = config.match_label.as_deref().unwrap_or(&config.id);
    table.allocate(
        sectors,
        type_guid::LINUX_FILESYSTEM,
        part_guid.into(),
        label,
    )?;
    table
        .entries
        .last()
        .cloned()
        .ok_or_else(|| BlockError::BadTable("partition allocation produced no entry".to_string()))
}

/// Apply a partition growth decision to an existing in-memory GPT entry.
///
/// The pure growth planner operates in bytes; GPT entries are sector-based, so
/// this side-effect boundary applies only complete sectors and returns the
/// sector-aligned growth that was actually written to the table.
pub fn apply_partition_growth_plan(
    table: &mut GptTable,
    config: &VolumeConfig,
    sector_size: u64,
) -> Result<PartitionGrowthPlan> {
    if sector_size == 0 {
        return Err(BlockError::Geometry("zero sector size".to_string()));
    }
    if config.volume_type != VolumeType::Partition {
        return Err(BlockError::InvalidDevice(
            "partition growth plan requires a partition volume".to_string(),
        ));
    }

    let label = config.match_label.as_deref().unwrap_or(&config.id);
    let index = table
        .entries
        .iter()
        .position(|entry| entry.name.eq_ignore_ascii_case(label))
        .ok_or_else(|| BlockError::NotFound(format!("partition {label} not found")))?;
    let entry = &table.entries[index];
    let current_size = entry
        .sector_count()
        .checked_mul(sector_size)
        .ok_or_else(|| BlockError::Geometry("partition size overflows".to_string()))?;

    let next_start = table
        .entries
        .iter()
        .enumerate()
        .filter_map(|(pos, other)| {
            (pos != index && other.first_lba > entry.last_lba).then_some(other.first_lba)
        })
        .min();
    let growth_limit_lba = next_start
        .and_then(|start| start.checked_sub(1))
        .unwrap_or(table.last_usable_lba);
    let available_growth_sectors = growth_limit_lba.saturating_sub(entry.last_lba);
    let available_growth = available_growth_sectors
        .checked_mul(sector_size)
        .ok_or_else(|| BlockError::Geometry("available partition growth overflows".to_string()))?;
    let planned = plan_partition_growth(config, current_size, available_growth)?;
    let grow_sectors = planned.grow_by / sector_size;
    let applied_grow_by = grow_sectors
        .checked_mul(sector_size)
        .ok_or_else(|| BlockError::Geometry("partition growth overflows size".to_string()))?;
    let applied_target_size = current_size
        .checked_add(applied_grow_by)
        .ok_or_else(|| BlockError::Geometry("partition growth overflows size".to_string()))?;

    if grow_sectors != 0 {
        let new_last_lba = table.entries[index]
            .last_lba
            .checked_add(grow_sectors)
            .ok_or_else(|| BlockError::Geometry("partition growth overflows LBA".to_string()))?;
        if new_last_lba > growth_limit_lba {
            return Err(BlockError::Geometry(
                "partition growth exceeds contiguous free space".to_string(),
            ));
        }
        table.entries[index].last_lba = new_last_lba;
        table.validate()?;
    }

    Ok(PartitionGrowthPlan {
        current_size,
        available_growth,
        grow_by: applied_grow_by,
        target_size: applied_target_size,
    })
}

/// Whether the machine boots via UEFI or legacy BIOS. Determines whether a
/// BIOS-boot partition is added to the layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootMode {
    /// Pure UEFI install (no BIOS-boot partition).
    Uefi,
    /// Legacy BIOS / hybrid (adds a 1 MiB BIOS-boot partition).
    Bios,
}

/// Build the ordered list of [`PartitionSpec`]s for a standard Talos install.
///
/// Order matters: EFI first, then (BIOS), BOOT, META, STATE, and EPHEMERAL
/// last so it can consume the remaining space.
pub fn standard_layout(boot: BootMode) -> Vec<PartitionSpec> {
    let mut specs = Vec::with_capacity(6);
    specs.push(PartitionSpec::fixed(
        PartitionRole::Efi,
        type_guid::EFI,
        size::EFI,
        ATTR_REQUIRED,
    ));
    if boot == BootMode::Bios {
        specs.push(PartitionSpec::fixed(
            PartitionRole::Bios,
            type_guid::BIOS_BOOT,
            size::BIOS_BOOT,
            ATTR_REQUIRED | ATTR_LEGACY_BOOT,
        ));
    }
    specs.push(PartitionSpec::fixed(
        PartitionRole::Boot,
        type_guid::LINUX_FILESYSTEM,
        size::BOOT,
        ATTR_REQUIRED,
    ));
    specs.push(PartitionSpec::fixed(
        PartitionRole::Meta,
        type_guid::LINUX_FILESYSTEM,
        size::META,
        ATTR_REQUIRED,
    ));
    specs.push(PartitionSpec::fixed(
        PartitionRole::State,
        type_guid::LINUX_FILESYSTEM,
        size::STATE,
        ATTR_REQUIRED,
    ));
    specs.push(PartitionSpec::grow(
        PartitionRole::Ephemeral,
        type_guid::LINUX_FILESYSTEM,
    ));
    specs
}

/// Minimum disk size, in bytes, needed to hold the fixed-size partitions of a
/// layout plus a token EPHEMERAL partition.
pub fn minimum_disk_size(specs: &[PartitionSpec]) -> u64 {
    specs.iter().filter_map(|s| s.size_bytes).sum::<u64>()
        // Allow at least one alignment unit (1 MiB) for EPHEMERAL.
        + 1024 * 1024
}

/// Allocate `specs` into a fresh [`GptTable`] for a disk of `total_sectors`
/// sectors of `sector_size` bytes each.
///
/// Fixed-size specs are rounded up to whole sectors and allocated in order; a
/// single grow spec (which must be last) consumes all remaining usable space.
/// Returns the populated table.
pub fn apply_layout(
    disk_guid: &str,
    total_sectors: u64,
    sector_size: u64,
    specs: &[PartitionSpec],
) -> Result<GptTable> {
    if sector_size == 0 {
        return Err(BlockError::Geometry("zero sector size".to_string()));
    }
    let mut table = GptTable::new(disk_guid, total_sectors)?;

    // Only one grow partition is allowed, and it must be final.
    let grow_positions = specs
        .iter()
        .enumerate()
        .filter_map(|(pos, spec)| spec.size_bytes.is_none().then_some(pos))
        .collect::<Vec<_>>();
    if let Some(&pos) = grow_positions.first()
        && (grow_positions.len() != 1 || pos != specs.len() - 1)
    {
        return Err(BlockError::Geometry(
            "grow partition must be last".to_string(),
        ));
    }

    for (i, spec) in specs.iter().enumerate() {
        let part_guid = format!("part-{:08x}", i + 1);
        let label = spec.role.label();
        if let Some(bytes) = spec.size_bytes {
            let sectors = bytes.div_ceil(sector_size);
            let (_s, _e) = table.allocate(sectors, spec.type_guid.clone(), part_guid, label)?;
            // Apply attributes onto the just-added entry.
            if let Some(entry) = table.entries.last_mut() {
                entry.attributes = spec.attributes;
            }
        } else {
            // Grow: take whatever usable space is left, aligned.
            let mut sectors = remaining_aligned_sectors(&table);
            if let Some(max_size_bytes) = spec.max_size_bytes {
                let max_sectors = max_size_bytes.div_ceil(sector_size);
                sectors = sectors.min(max_sectors);
            }
            if sectors == 0 {
                return Err(BlockError::Geometry(
                    "no space left for grow partition".to_string(),
                ));
            }
            let (_s, _e) = table.allocate(sectors, spec.type_guid.clone(), part_guid, label)?;
        }
    }
    table.validate()?;
    Ok(table)
}

/// Free sectors remaining after the current highest entry, accounting for the
/// 2048-sector alignment `GptTable::allocate` uses.
fn remaining_aligned_sectors(table: &GptTable) -> u64 {
    const ALIGN: u64 = 2048;
    let mut cursor = table.first_usable_lba;
    for e in &table.entries {
        if e.last_lba + 1 > cursor {
            cursor = e.last_lba + 1;
        }
    }
    let start = cursor.div_ceil(ALIGN) * ALIGN;
    let start = start.max(table.first_usable_lba);
    if start > table.last_usable_lba {
        0
    } else {
        table.last_usable_lba - start + 1
    }
}

/// Convenience: build a [`GptTable`] for the standard Talos layout on a disk.
pub fn install_table(
    disk_guid: &str,
    total_sectors: u64,
    sector_size: u64,
    boot: BootMode,
) -> Result<GptTable> {
    let specs = standard_layout(boot);
    apply_layout(disk_guid, total_sectors, sector_size, &specs)
}

/// Look up the canonical type GUID for a [`PartitionEntry`]'s role label.
pub fn type_guid_for(role: PartitionRole) -> &'static str {
    match role {
        PartitionRole::Efi => type_guid::EFI,
        PartitionRole::Bios => type_guid::BIOS_BOOT,
        _ => type_guid::LINUX_FILESYSTEM,
    }
}

/// Whether an entry's type GUID marks it as the EFI system partition.
pub fn is_efi_entry(entry: &PartitionEntry) -> bool {
    entry.type_guid.eq_ignore_ascii_case(type_guid::EFI)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECTOR: u64 = 512;

    fn big_disk_sectors() -> u64 {
        // 16 GiB disk.
        (16u64 * 1024 * 1024 * 1024) / SECTOR
    }

    #[test]
    fn uefi_layout_has_no_bios_partition() {
        let specs = standard_layout(BootMode::Uefi);
        assert!(!specs.iter().any(|s| s.role == PartitionRole::Bios));
        // EFI, BOOT, META, STATE, EPHEMERAL.
        assert_eq!(specs.len(), 5);
        assert_eq!(specs.first().unwrap().role, PartitionRole::Efi);
        assert_eq!(specs.last().unwrap().role, PartitionRole::Ephemeral);
        assert!(specs.last().unwrap().size_bytes.is_none());
    }

    #[test]
    fn bios_layout_adds_bios_boot() {
        let specs = standard_layout(BootMode::Bios);
        let bios = specs
            .iter()
            .find(|s| s.role == PartitionRole::Bios)
            .unwrap();
        assert_eq!(bios.type_guid, type_guid::BIOS_BOOT);
        assert!(bios.attributes & ATTR_LEGACY_BOOT != 0);
        assert_eq!(specs.len(), 6);
    }

    #[test]
    fn install_table_allocates_all_partitions() {
        let table = install_table("disk-guid", big_disk_sectors(), SECTOR, BootMode::Bios).unwrap();
        assert_eq!(table.entries.len(), 6);
        table.validate().unwrap();
        // EFI partition must carry the EFI type GUID and be required.
        let efi = &table.entries[0];
        assert!(is_efi_entry(efi));
        assert!(efi.is_required());
        // EPHEMERAL (last) should consume the bulk of the disk.
        let ephemeral = table.entries.last().unwrap();
        let efi_sectors = size::EFI / SECTOR;
        assert!(ephemeral.sector_count() > efi_sectors * 100);
    }

    #[test]
    fn partitions_are_ordered_and_nonoverlapping() {
        let table = install_table("g", big_disk_sectors(), SECTOR, BootMode::Uefi).unwrap();
        let mut prev_end = 0;
        for e in &table.entries {
            assert!(
                e.first_lba > prev_end,
                "entries must be strictly increasing"
            );
            prev_end = e.last_lba;
        }
    }

    #[test]
    fn grow_must_be_last() {
        let mut specs = standard_layout(BootMode::Uefi);
        // Move EPHEMERAL (grow) to the front.
        let grow = specs.pop().unwrap();
        specs.insert(0, grow);
        let err = apply_layout("g", big_disk_sectors(), SECTOR, &specs);
        assert!(matches!(
            err,
            Err(BlockError::Geometry(message)) if message.contains("grow partition must be last")
        ));
    }

    #[test]
    fn partition_grow_plan_rejects_multiple_grow_partitions() {
        let mut specs = standard_layout(BootMode::Uefi);
        specs.push(PartitionSpec::grow(
            PartitionRole::Other,
            type_guid::LINUX_FILESYSTEM,
        ));

        let err = apply_layout("g", big_disk_sectors(), SECTOR, &specs);
        assert!(matches!(
            err,
            Err(BlockError::Geometry(message)) if message.contains("grow partition must be last")
        ));
    }

    #[test]
    fn partition_grow_plan_bounded_grow_spec_caps_initial_allocation() {
        let max_size = 512 * 1024 * 1024;
        let specs = vec![
            PartitionSpec::fixed(
                PartitionRole::State,
                type_guid::LINUX_FILESYSTEM,
                size::STATE,
                ATTR_REQUIRED,
            ),
            PartitionSpec::grow_bounded(
                PartitionRole::Ephemeral,
                type_guid::LINUX_FILESYSTEM,
                max_size,
            ),
        ];

        let table = apply_layout("g", big_disk_sectors(), SECTOR, &specs).unwrap();
        let ephemeral = table.find("EPHEMERAL").unwrap();

        assert_eq!(ephemeral.sector_count(), max_size / SECTOR);
        assert!(ephemeral.last_lba < table.last_usable_lba);
        table.validate().unwrap();
    }

    #[test]
    fn partition_grow_plan_create_size_uses_available_extent_unless_absolute_max_caps() {
        let mut cfg = VolumeConfig::partition("DATA", "DATA", 100 * 1024 * 1024);

        assert_eq!(
            plan_partition_create_size(&cfg, 10 * 1024 * 1024 * 1024).unwrap(),
            10 * 1024 * 1024 * 1024
        );

        cfg.max_size = Some(2 * 1024 * 1024 * 1024);
        assert_eq!(
            plan_partition_create_size(&cfg, 10 * 1024 * 1024 * 1024).unwrap(),
            2 * 1024 * 1024 * 1024
        );
    }

    #[test]
    fn partition_grow_plan_create_size_resolves_relative_and_negative_max_size() {
        let gib = 1024 * 1024 * 1024;
        let mut relative = VolumeConfig::partition("DATA", "DATA", 100 * 1024 * 1024);
        relative.relative_max_size = Some(80);
        assert_eq!(
            plan_partition_create_size(&relative, 10 * gib).unwrap(),
            8 * gib
        );

        let mut negative = VolumeConfig::partition("DATA", "DATA", 100 * 1024 * 1024);
        negative.max_size = Some(gib);
        negative.negative_max_size = true;
        assert_eq!(
            plan_partition_create_size(&negative, 10 * gib).unwrap(),
            9 * gib
        );

        let mut too_negative = VolumeConfig::partition("DATA", "DATA", 100 * 1024 * 1024);
        too_negative.max_size = Some(11 * gib);
        too_negative.negative_max_size = true;
        assert!(matches!(
            plan_partition_create_size(&too_negative, 10 * gib),
            Err(BlockError::Geometry(message)) if message.contains("cannot be negative")
        ));
    }

    #[test]
    fn partition_grow_plan_create_size_requires_minimum_available_extent() {
        let cfg = VolumeConfig::partition("DATA", "DATA", 2 * 1024 * 1024 * 1024);

        let err = plan_partition_create_size(&cfg, 1024 * 1024 * 1024);

        assert!(matches!(
            err,
            Err(BlockError::Geometry(message)) if message.contains("not enough space")
        ));
    }

    #[test]
    fn partition_grow_plan_existing_volume_requires_explicit_true_grow() {
        let mut cfg = VolumeConfig::partition("DATA", "DATA", 100 * 1024 * 1024);
        cfg.grow = Some(false);

        let plan = plan_partition_growth(&cfg, 1024 * 1024 * 1024, 4 * 1024 * 1024 * 1024).unwrap();

        assert!(!plan.grows());
        assert_eq!(plan.target_size, 1024 * 1024 * 1024);
    }

    #[test]
    fn partition_grow_plan_existing_volume_uses_one_mib_threshold_before_cap() {
        let mut cfg = VolumeConfig::partition("DATA", "DATA", 100 * 1024 * 1024);
        cfg.grow = Some(true);
        cfg.max_size = Some(1024 * 1024 * 1024 + 512 * 1024);

        let plan =
            plan_partition_growth(&cfg, 1024 * 1024 * 1024, MIN_PARTITION_GROWTH_BYTES).unwrap();

        assert!(!plan.grows());
        assert_eq!(plan.target_size, 1024 * 1024 * 1024);
    }

    #[test]
    fn partition_grow_plan_existing_volume_caps_growth_to_max_size() {
        let mut cfg = VolumeConfig::partition("DATA", "DATA", 100 * 1024 * 1024);
        cfg.grow = Some(true);
        cfg.max_size = Some(3 * 1024 * 1024 * 1024);

        let plan = plan_partition_growth(&cfg, 1024 * 1024 * 1024, 8 * 1024 * 1024 * 1024).unwrap();

        assert!(plan.grows());
        assert_eq!(plan.grow_by, 2 * 1024 * 1024 * 1024);
        assert_eq!(plan.target_size, 3 * 1024 * 1024 * 1024);
    }

    #[test]
    fn partition_grow_plan_existing_volume_uses_absolute_max_size_like_talos() {
        let gib = 1024 * 1024 * 1024;
        let mut relative = VolumeConfig::partition("DATA", "DATA", 100 * 1024 * 1024);
        relative.grow = Some(true);
        relative.relative_max_size = Some(80);

        let plan = plan_partition_growth(&relative, 4 * gib, 6 * gib).unwrap();

        assert!(plan.grows());
        assert_eq!(plan.grow_by, 6 * gib);
        assert_eq!(plan.target_size, 10 * gib);

        let mut negative = VolumeConfig::partition("DATA", "DATA", 100 * 1024 * 1024);
        negative.grow = Some(true);
        negative.max_size = Some(gib);
        negative.negative_max_size = true;

        let plan = plan_partition_growth(&negative, 4 * gib, 6 * gib).unwrap();

        assert!(!plan.grows());
        assert_eq!(plan.grow_by, 0);
        assert_eq!(plan.target_size, 4 * gib);
    }

    #[test]
    fn partition_mutation_plan_create_resolves_relative_and_negative_max_size() {
        let gib = 1024 * 1024 * 1024;

        let mut relative_table =
            GptTable::new("relative-disk", (12 * gib) / SECTOR).expect("valid GPT");
        let relative_available = remaining_aligned_sectors(&relative_table) * SECTOR;
        let mut relative = VolumeConfig::partition("DATA", "DATA", 100 * 1024 * 1024);
        relative.relative_max_size = Some(80);

        let entry =
            apply_partition_create_plan(&mut relative_table, &relative, SECTOR, "relative-part")
                .expect("relative maxSize should allocate a GPT entry");

        let expected_relative_size = relative.resolve_max_size(relative_available).unwrap();
        assert_eq!(entry.name, "DATA");
        assert_eq!(
            entry.sector_count(),
            expected_relative_size.div_ceil(SECTOR)
        );
        assert_eq!(relative_table.find("DATA"), Some(&entry));
        relative_table.validate().unwrap();

        let mut negative_table =
            GptTable::new("negative-disk", (12 * gib) / SECTOR).expect("valid GPT");
        let negative_available = remaining_aligned_sectors(&negative_table) * SECTOR;
        let mut negative = VolumeConfig::partition("CACHE", "CACHE", 100 * 1024 * 1024);
        negative.max_size = Some(gib);
        negative.negative_max_size = true;

        let entry =
            apply_partition_create_plan(&mut negative_table, &negative, SECTOR, "negative-part")
                .expect("negative maxSize should allocate a GPT entry");

        let expected_negative_size = negative.resolve_max_size(negative_available).unwrap();
        assert_eq!(entry.name, "CACHE");
        assert_eq!(
            entry.sector_count(),
            expected_negative_size.div_ceil(SECTOR)
        );
        assert_eq!(negative_table.find("CACHE"), Some(&entry));
        negative_table.validate().unwrap();
    }

    #[test]
    fn partition_mutation_plan_rejects_negative_underflow_without_mutating_gpt() {
        let gib = 1024 * 1024 * 1024;
        let mut table = GptTable::new("g", (4 * gib) / SECTOR).expect("valid GPT");
        let available = remaining_aligned_sectors(&table) * SECTOR;
        let mut too_negative = VolumeConfig::partition("DATA", "DATA", 100 * 1024 * 1024);
        too_negative.max_size = Some(available + SECTOR);
        too_negative.negative_max_size = true;

        let err = apply_partition_create_plan(&mut table, &too_negative, SECTOR, "data");

        assert!(matches!(
            err,
            Err(BlockError::Geometry(message)) if message.contains("cannot be negative")
        ));
        assert!(table.entries.is_empty());
    }

    #[test]
    fn partition_mutation_plan_growth_uses_absolute_max_size_like_talos() {
        let gib = 1024 * 1024 * 1024;

        let mut relative_table =
            GptTable::new("relative-disk", (12 * gib) / SECTOR).expect("valid GPT");
        relative_table
            .allocate(
                (4 * gib) / SECTOR,
                type_guid::LINUX_FILESYSTEM,
                "relative-part",
                "DATA",
            )
            .unwrap();
        let before = relative_table.find("DATA").unwrap().clone();
        let mut relative = VolumeConfig::partition("DATA", "DATA", 100 * 1024 * 1024);
        relative.grow = Some(true);
        relative.relative_max_size = Some(80);

        let plan =
            apply_partition_growth_plan(&mut relative_table, &relative, SECTOR).expect("grow DATA");

        let expected_target =
            (before.sector_count() + (relative_table.last_usable_lba - before.last_lba)) * SECTOR;
        assert!(plan.grows());
        assert_eq!(plan.target_size, expected_target);
        assert_eq!(
            relative_table.find("DATA").unwrap().sector_count() * SECTOR,
            expected_target
        );
        relative_table.validate().unwrap();

        let mut negative_table =
            GptTable::new("negative-disk", (12 * gib) / SECTOR).expect("valid GPT");
        negative_table
            .allocate(
                (4 * gib) / SECTOR,
                type_guid::LINUX_FILESYSTEM,
                "negative-part",
                "CACHE",
            )
            .unwrap();
        let before = negative_table.find("CACHE").unwrap().clone();
        let mut negative = VolumeConfig::partition("CACHE", "CACHE", 100 * 1024 * 1024);
        negative.grow = Some(true);
        negative.max_size = Some(gib);
        negative.negative_max_size = true;

        let plan = apply_partition_growth_plan(&mut negative_table, &negative, SECTOR)
            .expect("grow CACHE");

        assert!(!plan.grows());
        assert_eq!(plan.grow_by, 0);
        assert_eq!(plan.target_size, before.sector_count() * SECTOR);
        assert_eq!(
            negative_table.find("CACHE").unwrap().sector_count() * SECTOR,
            before.sector_count() * SECTOR
        );
        negative_table.validate().unwrap();
    }

    #[test]
    fn partition_grow_plan_existing_volume_at_max_size_is_already_provisioned() {
        let mut cfg = VolumeConfig::partition("DATA", "DATA", 100 * 1024 * 1024);
        cfg.grow = Some(true);
        cfg.max_size = Some(1024 * 1024 * 1024);

        let plan = plan_partition_growth(&cfg, 1024 * 1024 * 1024, 8 * 1024 * 1024).unwrap();

        assert!(!plan.grows());
        assert_eq!(plan.grow_by, 0);
        assert_eq!(plan.target_size, 1024 * 1024 * 1024);
    }

    #[test]
    fn tiny_disk_cannot_hold_layout() {
        // 50 MiB disk: too small for EFI(100MiB)+BOOT(1000MiB)+...
        let small = (50u64 * 1024 * 1024) / SECTOR;
        let err = install_table("g", small, SECTOR, BootMode::Uefi);
        assert!(err.is_err());
    }

    #[test]
    fn minimum_disk_size_sums_fixed() {
        let specs = standard_layout(BootMode::Bios);
        let min = minimum_disk_size(&specs);
        let expected =
            size::EFI + size::BIOS_BOOT + size::BOOT + size::META + size::STATE + 1024 * 1024;
        assert_eq!(min, expected);
    }

    #[test]
    fn type_guid_for_roles() {
        assert_eq!(type_guid_for(PartitionRole::Efi), type_guid::EFI);
        assert_eq!(type_guid_for(PartitionRole::Bios), type_guid::BIOS_BOOT);
        assert_eq!(
            type_guid_for(PartitionRole::Ephemeral),
            type_guid::LINUX_FILESYSTEM
        );
    }

    #[test]
    fn projecting_to_partitions_keeps_roles() {
        let table = install_table("g", big_disk_sectors(), SECTOR, BootMode::Bios).unwrap();
        let parts = table.to_partitions("sda", SECTOR);
        assert_eq!(parts.len(), 6);
        assert_eq!(parts[0].role, PartitionRole::Efi);
        assert_eq!(parts.last().unwrap().role, PartitionRole::Ephemeral);
        assert_eq!(parts[1].dev_name, "sda2");
    }
}
