//! The controller that assembles disks into discovered volumes.
//!
//! Mirrors Talos's discovery controller: it takes the inventory of [`Disk`]s and
//! their partitions, probes them, and produces [`DiscoveredVolume`] records that
//! [`VolumeConfig`]s can be matched against. This is the glue between raw
//! hardware facts and the declarative volume layer.

use crate::disk::Disk;
use crate::filesystem::FilesystemType;
use crate::partition::{Partition, PartitionRole};
use crate::volume::{PartitionMatchPolicy, VolumeConfig, VolumeType};
use crate::{BlockError, Result};
use os_kernel::{DiskLocator, evaluate_disk_locator_bool_expression};

/// A volume the discovery controller has observed on a real device.
///
/// Unlike a [`VolumeConfig`] (which is declarative/desired), this records what
/// actually exists: a device path, its size, label and detected filesystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredVolume {
    /// Device path, e.g. `/dev/sda2`.
    pub dev_path: String,
    /// Parent disk device name, e.g. `sda`.
    pub parent: String,
    /// Size in bytes.
    pub size: u64,
    /// Partition label, if any.
    pub label: Option<String>,
    /// Detected filesystem, if any.
    pub filesystem: Option<FilesystemType>,
    /// Partition role derived from the label.
    pub role: PartitionRole,
}

impl DiscoveredVolume {
    /// Build a discovered volume from a whole disk.
    pub fn from_disk(disk: &Disk) -> Self {
        DiscoveredVolume {
            dev_path: disk.dev_path(),
            parent: disk.dev_name.clone(),
            size: disk.size,
            label: None,
            filesystem: None,
            role: PartitionRole::Other,
        }
    }

    /// Build a discovered volume from a parent disk and one of its partitions.
    pub fn from_partition(parent: &Disk, part: &Partition) -> Self {
        let role = part
            .label
            .as_deref()
            .map_or(part.role, PartitionRole::from_label);
        DiscoveredVolume {
            dev_path: {
                let mut p = String::from("/dev/");
                p.push_str(&part.dev_name);
                p
            },
            parent: parent.dev_name.clone(),
            size: part.size(),
            label: part.label.clone(),
            filesystem: part.filesystem,
            role,
        }
    }

    /// Whether this discovered volume satisfies `config`'s matching rules.
    pub fn satisfies(&self, config: &VolumeConfig) -> bool {
        config.matches(self.label.as_deref(), self.size)
    }
}

/// The discovery controller, holding an inventory of disks and their
/// partitions.
#[derive(Debug, Default)]
pub struct Discoverer {
    disks: Vec<Disk>,
    partitions: Vec<Partition>,
}

impl Discoverer {
    /// A fresh, empty discoverer.
    pub fn new() -> Self {
        Discoverer::default()
    }

    /// Register a disk in the inventory, validating it first.
    pub fn add_disk(&mut self, disk: Disk) -> Result<()> {
        disk.validate()?;
        if self.disks.iter().any(|d| d.dev_name == disk.dev_name) {
            return Err(BlockError::InvalidDevice(format!(
                "duplicate disk {}",
                disk.dev_name
            )));
        }
        self.disks.push(disk);
        Ok(())
    }

    /// Register a partition, validating it and ensuring its parent disk exists.
    pub fn add_partition(&mut self, parent_dev: &str, part: Partition) -> Result<()> {
        part.validate()?;
        if !self.disks.iter().any(|d| d.dev_name == parent_dev) {
            return Err(BlockError::NotFound(format!(
                "parent disk {parent_dev} not registered"
            )));
        }
        self.partitions.push(part);
        Ok(())
    }

    /// Number of registered disks.
    pub fn disk_count(&self) -> usize {
        self.disks.len()
    }

    /// Produce the discovered volumes from all registered partitions.
    pub fn discover(&self) -> Vec<DiscoveredVolume> {
        let mut out = Vec::new();
        for part in &self.partitions {
            // Find the parent disk for this partition by name prefix.
            if let Some(disk) = self
                .disks
                .iter()
                .find(|d| part.dev_name.starts_with(&d.dev_name))
            {
                out.push(DiscoveredVolume::from_partition(disk, part));
            }
        }
        out
    }

    /// Find the discovered volume that best satisfies `config`, if any.
    ///
    /// Among matching candidates, prefers an exact label match, then the
    /// smallest device that still meets the size requirement (least waste).
    /// If `config.disk_selector` is set, the selector must also evaluate true
    /// for the candidate's parent disk; selector parse/evaluation errors are
    /// returned so callers fail closed instead of falling back to any label
    /// match.
    pub fn resolve(&self, config: &VolumeConfig) -> Result<Option<DiscoveredVolume>> {
        config.validate()?;
        if config.volume_type == VolumeType::Disk {
            return self.resolve_disk_volume(config);
        }

        let mut candidates: Vec<DiscoveredVolume> = Vec::new();
        for volume in self.discover() {
            if !volume.satisfies(config) {
                continue;
            }
            if let Some(selector) = config.disk_selector.as_deref() {
                let disk = self
                    .disks
                    .iter()
                    .find(|disk| disk.dev_name == volume.parent)
                    .ok_or_else(|| {
                        BlockError::NotFound(format!(
                            "parent disk {} for {} not registered",
                            volume.parent, volume.dev_path
                        ))
                    })?;
                if !self.disk_matches_selector(disk, selector)? {
                    continue;
                }
            }
            candidates.push(volume);
        }
        if config.partition_match_policy == PartitionMatchPolicy::FirstMatch {
            return Ok(candidates.into_iter().next());
        }

        candidates.sort_by_key(|v| v.size);
        Ok(candidates.into_iter().next())
    }

    /// Resolve the disk that a missing partition-backed volume may be
    /// provisioned on.
    ///
    /// Existing partition discovery can legitimately return no matching volume;
    /// Talos then applies the partition creation controller to a selected disk.
    /// This helper keeps that create-time selection fail-closed: an explicit
    /// selector must match exactly one writable disk, while an implicit selector
    /// falls back to the best install target.
    pub fn resolve_partition_provisioning_disk(
        &self,
        config: &VolumeConfig,
    ) -> Result<Option<Disk>> {
        config.validate()?;
        if config.volume_type != VolumeType::Partition {
            return Ok(None);
        }

        let Some(selector) = config.disk_selector.as_deref() else {
            return Ok(self.install_target().cloned());
        };

        let mut candidates = Vec::new();
        for disk in &self.disks {
            if self.disk_matches_selector(disk, selector)? {
                candidates.push(disk.clone());
            }
        }
        match candidates.len() {
            0 => Ok(None),
            1 => Ok(candidates.pop()),
            count => Err(BlockError::InvalidDevice(format!(
                "partition volume {} matched {count} provisioning disks; refine disk selector",
                config.id
            ))),
        }
    }

    fn resolve_disk_volume(&self, config: &VolumeConfig) -> Result<Option<DiscoveredVolume>> {
        let selector = config.disk_selector.as_deref().ok_or_else(|| {
            BlockError::InvalidDevice("disk volume requires a disk selector".to_string())
        })?;
        let mut candidates = Vec::new();
        for disk in &self.disks {
            if self.disk_matches_selector(disk, selector)? {
                candidates.push(DiscoveredVolume::from_disk(disk));
            }
        }
        match candidates.len() {
            0 => Ok(None),
            1 => Ok(candidates.pop()),
            count => Err(BlockError::InvalidDevice(format!(
                "disk volume {} matched {count} disks; refine disk selector",
                config.id
            ))),
        }
    }

    /// Pick the best install-target disk among the inventory, if any.
    pub fn install_target(&self) -> Option<&Disk> {
        self.disks
            .iter()
            .filter(|d| d.is_install_candidate())
            .min_by_key(|d| d.size)
    }

    fn disk_matches_selector(&self, disk: &Disk, selector: &str) -> Result<bool> {
        if disk.readonly {
            return Ok(false);
        }
        let symlinks: Vec<&str> = disk.symlinks.iter().map(String::as_str).collect();
        let dev_path = disk.dev_path();
        let model = disk.model.as_deref().unwrap_or("");
        let serial = disk.serial.as_deref().unwrap_or("");
        let system_disk = self
            .install_target()
            .is_some_and(|candidate| candidate.dev_name == disk.dev_name);
        let locator = DiskLocator {
            size: disk.size,
            io_size: disk.sector_size,
            sector_size: disk.sector_size,
            readonly: disk.readonly,
            cdrom: disk.disk_type.is_cdrom(),
            rotational: disk.disk_type.is_rotational(),
            dev_path: &dev_path,
            pretty_size: "",
            model,
            serial,
            wwid: "",
            bus_path: "",
            sub_system: "block",
            transport: disk.bus.as_transport(),
            name: &disk.dev_name,
            disk_type: disk.disk_type.as_talos_type(),
            uuid: "",
            modalias: "",
            symlinks: &symlinks,
            system_disk,
        };
        evaluate_disk_locator_bool_expression(selector, &locator).map_err(|error| {
            BlockError::InvalidDevice(format!("disk selector evaluation failed: {error}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::disk::DiskBus;

    fn disk(name: &str, size: u64, bus: DiskBus) -> Disk {
        let mut d = Disk::new(name, size, bus);
        d.sector_size = 512;
        d
    }

    fn discoverer() -> Discoverer {
        let mut d = Discoverer::new();
        d.add_disk(disk("sda", 64 * 1024 * 1024 * 1024, DiskBus::Scsi))
            .unwrap();
        // EFI partition (vfat) and STATE partition (xfs).
        let mut efi = Partition::new("sda1", 1, 2048, 4095, PartitionRole::Efi);
        efi.sector_size = 512;
        let mut state = Partition::new("sda2", 2, 4096, 2_000_000, PartitionRole::State);
        state.sector_size = 512;
        d.add_partition("sda", efi).unwrap();
        d.add_partition("sda", state).unwrap();
        d
    }

    #[test]
    fn rejects_orphan_partition_and_dupes() {
        let mut d = Discoverer::new();
        let p = Partition::new("sdz1", 1, 0, 99, PartitionRole::Boot);
        assert!(matches!(
            d.add_partition("sdz", p),
            Err(BlockError::NotFound(_))
        ));
        d.add_disk(disk("sda", 1 << 30, DiskBus::Ata)).unwrap();
        assert!(d.add_disk(disk("sda", 1 << 30, DiskBus::Ata)).is_err());
    }

    #[test]
    fn discovers_partitions_with_roles() {
        let d = discoverer();
        let vols = d.discover();
        assert_eq!(vols.len(), 2);
        let efi = vols.iter().find(|v| v.role == PartitionRole::Efi).unwrap();
        assert_eq!(efi.dev_path, "/dev/sda1");
        assert_eq!(efi.parent, "sda");
        assert_eq!(efi.filesystem, Some(FilesystemType::Vfat));
    }

    #[test]
    fn resolve_matches_config_by_label() {
        let d = discoverer();
        let cfg = VolumeConfig::partition("STATE", "STATE", 1024);
        let resolved = d.resolve(&cfg).unwrap().unwrap();
        assert_eq!(resolved.role, PartitionRole::State);
        assert_eq!(resolved.dev_path, "/dev/sda2");

        let missing = VolumeConfig::partition("DATA", "DATA", 1024);
        assert!(d.resolve(&missing).unwrap().is_none());
    }

    #[test]
    fn resolve_respects_min_size() {
        let d = discoverer();
        // STATE partition spans 4096..=2_000_000 sectors -> ~1GiB. Demand 8 EiB.
        let cfg = VolumeConfig::partition("STATE", "STATE", u64::MAX);
        assert!(d.resolve(&cfg).unwrap().is_none());
    }

    #[test]
    fn runtime_disk_selector_filters_existing_volume_candidates_before_size_order() {
        let mut d = Discoverer::new();
        d.add_disk(disk("sda", 64 * 1024 * 1024 * 1024, DiskBus::Scsi))
            .unwrap();
        d.add_disk(disk("nvme0n1", 128 * 1024 * 1024 * 1024, DiskBus::Nvme))
            .unwrap();

        let mut scsi = Partition::new("sda1", 1, 2048, 2_099_199, PartitionRole::Other);
        scsi.label = Some("u-data".to_string());
        scsi.filesystem = Some(FilesystemType::Xfs);
        d.add_partition("sda", scsi).unwrap();

        let mut nvme = Partition::new("nvme0n1p1", 1, 2048, 4_196_351, PartitionRole::Other);
        nvme.label = Some("u-data".to_string());
        nvme.filesystem = Some(FilesystemType::Xfs);
        d.add_partition("nvme0n1", nvme).unwrap();

        let mut cfg = VolumeConfig::partition("u-data", "u-data", 512 * 1024 * 1024);
        cfg.disk_selector = Some(r#"disk.transport == "nvme""#.to_string());

        let resolved = d.resolve(&cfg).unwrap().unwrap();
        assert_eq!(resolved.dev_path, "/dev/nvme0n1p1");
        assert_eq!(resolved.parent, "nvme0n1");
    }

    #[test]
    // "fallback" names the prohibited behavior: a false selector fails closed instead of using label-only matching.
    fn runtime_disk_selector_false_keeps_volume_unresolved_without_label_fallback() {
        let mut d = Discoverer::new();
        d.add_disk(disk("sda", 64 * 1024 * 1024 * 1024, DiskBus::Scsi))
            .unwrap();
        let mut scsi = Partition::new("sda1", 1, 2048, 2_099_199, PartitionRole::Other);
        scsi.label = Some("u-data".to_string());
        scsi.filesystem = Some(FilesystemType::Xfs);
        d.add_partition("sda", scsi).unwrap();

        let mut cfg = VolumeConfig::partition("u-data", "u-data", 512 * 1024 * 1024);
        cfg.disk_selector = Some(r#"disk.transport == "nvme""#.to_string());

        assert!(d.resolve(&cfg).unwrap().is_none());
    }

    #[test]
    fn runtime_disk_selector_ignores_readonly_disks() {
        let mut d = Discoverer::new();
        let mut readonly_nvme = disk("nvme0n1", 128 * 1024 * 1024 * 1024, DiskBus::Nvme);
        readonly_nvme.readonly = true;
        d.add_disk(readonly_nvme).unwrap();
        let mut part = Partition::new("nvme0n1p1", 1, 2048, 4_196_351, PartitionRole::Other);
        part.label = Some("u-data".to_string());
        part.filesystem = Some(FilesystemType::Xfs);
        d.add_partition("nvme0n1", part).unwrap();

        let mut cfg = VolumeConfig::partition("u-data", "u-data", 512 * 1024 * 1024);
        cfg.disk_selector = Some(r#"disk.transport == "nvme""#.to_string());

        assert!(d.resolve(&cfg).unwrap().is_none());
    }

    #[test]
    fn raw_disk_exact_one_selector_resolves_whole_disk() {
        let mut d = Discoverer::new();
        d.add_disk(disk("sda", 64 * 1024 * 1024 * 1024, DiskBus::Scsi))
            .unwrap();
        d.add_disk(disk("nvme0n1", 128 * 1024 * 1024 * 1024, DiskBus::Nvme))
            .unwrap();
        let cfg = VolumeConfig::disk("u-data", r#"disk.transport == "nvme""#);

        let resolved = d.resolve(&cfg).unwrap().unwrap();

        assert_eq!(resolved.dev_path, "/dev/nvme0n1");
        assert_eq!(resolved.parent, "nvme0n1");
        assert_eq!(resolved.label, None);
        assert_eq!(resolved.role, PartitionRole::Other);
    }

    #[test]
    fn raw_disk_exact_one_selector_false_leaves_volume_unresolved() {
        let mut d = Discoverer::new();
        d.add_disk(disk("sda", 64 * 1024 * 1024 * 1024, DiskBus::Scsi))
            .unwrap();
        let cfg = VolumeConfig::disk("u-data", r#"disk.transport == "nvme""#);

        assert!(d.resolve(&cfg).unwrap().is_none());
    }

    #[test]
    fn raw_disk_exact_one_selector_rejects_multiple_matching_disks() {
        let mut d = Discoverer::new();
        d.add_disk(disk("nvme0n1", 64 * 1024 * 1024 * 1024, DiskBus::Nvme))
            .unwrap();
        d.add_disk(disk("nvme1n1", 128 * 1024 * 1024 * 1024, DiskBus::Nvme))
            .unwrap();
        let cfg = VolumeConfig::disk("u-data", r#"disk.transport == "nvme""#);

        let err = d.resolve(&cfg);

        assert!(matches!(
            err,
            Err(BlockError::InvalidDevice(message))
                if message.contains("matched 2 disks") && message.contains("refine disk selector")
        ));
    }

    #[test]
    fn raw_volume_partition_resolves_first_matching_partition_without_exact_one_error() {
        let mut d = Discoverer::new();
        d.add_disk(disk("sda", 64 * 1024 * 1024 * 1024, DiskBus::Scsi))
            .unwrap();

        let mut first = Partition::new("sda1", 1, 2048, 8_390_655, PartitionRole::Other);
        first.label = Some("r-local-data".to_string());
        d.add_partition("sda", first).unwrap();

        let mut second = Partition::new("sda2", 2, 8_390_656, 10_487_807, PartitionRole::Other);
        second.label = Some("r-local-data".to_string());
        d.add_partition("sda", second).unwrap();

        let cfg = VolumeConfig::raw_partition("r-local-data", "r-local-data", 1024);

        let resolved = d.resolve(&cfg).unwrap().unwrap();

        assert_eq!(resolved.dev_path, "/dev/sda1");
    }

    #[test]
    fn raw_volume_partition_first_match_preserves_discovery_order_over_smallest_size() {
        let mut d = Discoverer::new();
        d.add_disk(disk("sda", 64 * 1024 * 1024 * 1024, DiskBus::Scsi))
            .unwrap();

        let mut larger_first = Partition::new("sda1", 1, 2048, 8_390_655, PartitionRole::Other);
        larger_first.label = Some("r-local-data".to_string());
        d.add_partition("sda", larger_first).unwrap();

        let mut smaller_second =
            Partition::new("sda2", 2, 8_390_656, 9_439_231, PartitionRole::Other);
        smaller_second.label = Some("r-local-data".to_string());
        d.add_partition("sda", smaller_second).unwrap();

        let raw = VolumeConfig::raw_partition("r-local-data", "r-local-data", 1024);
        let partition = VolumeConfig::partition("r-local-data", "r-local-data", 1024);

        assert_eq!(d.resolve(&raw).unwrap().unwrap().dev_path, "/dev/sda1");
        assert_eq!(
            d.resolve(&partition).unwrap().unwrap().dev_path,
            "/dev/sda2"
        );
    }

    #[test]
    fn raw_volume_partition_filters_disk_selector_before_first_match() {
        let mut d = Discoverer::new();
        d.add_disk(disk("sda", 64 * 1024 * 1024 * 1024, DiskBus::Scsi))
            .unwrap();
        d.add_disk(disk("nvme0n1", 128 * 1024 * 1024 * 1024, DiskBus::Nvme))
            .unwrap();

        let mut scsi = Partition::new("sda1", 1, 2048, 8_390_655, PartitionRole::Other);
        scsi.label = Some("r-local-data".to_string());
        d.add_partition("sda", scsi).unwrap();

        let mut nvme = Partition::new("nvme0n1p1", 1, 2048, 4_196_351, PartitionRole::Other);
        nvme.label = Some("r-local-data".to_string());
        d.add_partition("nvme0n1", nvme).unwrap();

        let mut cfg = VolumeConfig::raw_partition("r-local-data", "r-local-data", 1024);
        cfg.disk_selector = Some(r#"disk.transport == "nvme""#.to_string());

        let resolved = d.resolve(&cfg).unwrap().unwrap();

        assert_eq!(resolved.dev_path, "/dev/nvme0n1p1");
    }

    #[test]
    fn install_target_prefers_smallest_eligible() {
        let mut d = Discoverer::new();
        d.add_disk(disk("sda", 64 * 1024 * 1024 * 1024, DiskBus::Scsi))
            .unwrap();
        d.add_disk(disk("sdb", 8 * 1024 * 1024 * 1024, DiskBus::Scsi))
            .unwrap();
        d.add_disk(disk("sdc", 64 * 1024 * 1024 * 1024, DiskBus::Usb))
            .unwrap(); // removable, ineligible
        let target = d.install_target().unwrap();
        assert_eq!(target.dev_name, "sdb");
        assert_eq!(d.disk_count(), 3);
    }
}
