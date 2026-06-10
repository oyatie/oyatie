//! Block-device discovery metadata.
//!
//! Mirrors the `block.Disk` COSI resource: the inventory record machined keeps
//! for every raw block device it discovers via udev/sysfs. It carries the
//! immutable hardware facts (size, sector size, bus, transport) plus the device
//! name used to address it.

use crate::{BlockError, DEFAULT_SECTOR_SIZE, Result};

/// The physical/logical transport a disk hangs off of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskBus {
    /// ATA / SATA.
    Ata,
    /// NVM Express.
    Nvme,
    /// SCSI / SAS.
    Scsi,
    /// USB mass storage.
    Usb,
    /// Virtio (paravirtualized) block device.
    Virtio,
    /// Unknown or not reported.
    Unknown,
}

impl DiskBus {
    /// Talos/CEL transport string used by `disk.transport` selectors.
    pub fn as_transport(self) -> &'static str {
        match self {
            DiskBus::Ata => "sata",
            DiskBus::Nvme => "nvme",
            DiskBus::Scsi => "scsi",
            DiskBus::Usb => "usb",
            DiskBus::Virtio => "virtio",
            DiskBus::Unknown => "",
        }
    }

    /// Whether a disk on this bus is generally considered removable.
    pub fn is_removable_by_default(self) -> bool {
        matches!(self, DiskBus::Usb)
    }
}

/// The rotational classification of a disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskType {
    /// Spinning rust (rotational).
    Hdd,
    /// Solid-state, non-rotational.
    Ssd,
    /// `NVMe` solid-state.
    Nvme,
    /// SD / eMMC card.
    Sd,
    /// CD/DVD optical drive.
    Cd,
    /// Unknown.
    Unknown,
}

impl DiskType {
    /// Talos/CEL type string used by `disk.type` selectors.
    pub fn as_talos_type(self) -> &'static str {
        match self {
            DiskType::Hdd => "hdd",
            DiskType::Ssd => "ssd",
            DiskType::Nvme => "nvme",
            DiskType::Sd => "sd",
            DiskType::Cd => "cd",
            DiskType::Unknown => "",
        }
    }

    /// Whether this type represents rotational storage.
    pub fn is_rotational(self) -> bool {
        matches!(self, DiskType::Hdd)
    }

    /// Whether this type represents optical media.
    pub fn is_cdrom(self) -> bool {
        matches!(self, DiskType::Cd)
    }
}

/// A discovered block device.
///
/// Sizes are in bytes. `sector_size` is the logical block size reported by the
/// device (commonly 512 or 4096).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Disk {
    /// Kernel device name, e.g. `sda`, `nvme0n1`, `vda`.
    pub dev_name: String,
    /// Total capacity in bytes.
    pub size: u64,
    /// Logical sector / block size in bytes.
    pub sector_size: u64,
    /// Bus / transport.
    pub bus: DiskBus,
    /// Rotational classification.
    pub disk_type: DiskType,
    /// Whether the device is read-only.
    pub readonly: bool,
    /// Whether the device is removable (USB stick, SD card, ...).
    pub removable: bool,
    /// Optional model string from the device.
    pub model: Option<String>,
    /// Optional hardware serial number.
    pub serial: Option<String>,
    /// Stable `/dev/disk/by-id` style symlinks.
    pub symlinks: Vec<String>,
}

impl Disk {
    /// Construct a disk with sane defaults for everything but the essentials.
    pub fn new(dev_name: impl Into<String>, size: u64, bus: DiskBus) -> Self {
        let disk_type = match bus {
            DiskBus::Nvme => DiskType::Nvme,
            DiskBus::Usb => DiskType::Sd,
            _ => DiskType::Unknown,
        };
        Disk {
            dev_name: dev_name.into(),
            size,
            sector_size: DEFAULT_SECTOR_SIZE,
            bus,
            disk_type,
            readonly: false,
            removable: bus.is_removable_by_default(),
            model: None,
            serial: None,
            symlinks: Vec::new(),
        }
    }

    /// Validate the disk record. Sizes and sector size must be sane and the
    /// device name non-empty.
    pub fn validate(&self) -> Result<()> {
        if self.dev_name.is_empty() {
            return Err(BlockError::InvalidDevice(String::from("empty device name")));
        }
        if self.sector_size == 0 || !self.sector_size.is_power_of_two() {
            return Err(BlockError::Geometry(String::from(
                "sector size must be a power of two",
            )));
        }
        if !self.size.is_multiple_of(self.sector_size) {
            return Err(BlockError::Geometry(String::from(
                "size not a multiple of sector size",
            )));
        }
        Ok(())
    }

    /// Number of addressable sectors on the disk.
    pub fn sector_count(&self) -> u64 {
        if self.sector_size == 0 {
            return 0;
        }
        self.size / self.sector_size
    }

    /// Absolute `/dev` path of the device, e.g. `/dev/sda`.
    pub fn dev_path(&self) -> String {
        let mut p = String::from("/dev/");
        p.push_str(&self.dev_name);
        p
    }

    /// Build the kernel name of the `n`-th partition (1-based) following the
    /// Linux convention: a `p` separator is inserted when the disk name ends in
    /// a digit (`nvme0n1` -> `nvme0n1p1`) but not otherwise (`sda` -> `sda1`).
    pub fn partition_name(&self, index: u32) -> String {
        let mut name = self.dev_name.clone();
        if self
            .dev_name
            .chars()
            .last()
            .is_some_and(|c| c.is_ascii_digit())
        {
            name.push('p');
        }
        name.push_str(&itoa(index));
        name
    }

    /// Whether this disk is a sensible install target: writable, not removable,
    /// not optical, and large enough to hold a typical install (>= 2 GiB here).
    pub fn is_install_candidate(&self) -> bool {
        const MIN_INSTALL: u64 = 2 * 1024 * 1024 * 1024;
        !self.readonly
            && !self.removable
            && self.disk_type != DiskType::Cd
            && self.size >= MIN_INSTALL
    }
}

/// Minimal allocation-friendly integer-to-string used to avoid pulling in
/// formatting machinery in hot paths; `alloc::format!` would also work.
fn itoa(mut n: u32) -> String {
    if n == 0 {
        return String::from("0");
    }
    let mut buf = [0u8; 10];
    let mut i = buf.len();
    while n > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    String::from_utf8_lossy(&buf[i..]).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn big(bus: DiskBus) -> Disk {
        Disk::new("sda", 64 * 1024 * 1024 * 1024, bus)
    }

    #[test]
    fn validate_rejects_bad_geometry() {
        let mut d = big(DiskBus::Scsi);
        d.sector_size = 0;
        assert!(matches!(d.validate(), Err(BlockError::Geometry(_))));
        d.sector_size = 500; // not a power of two
        assert!(d.validate().is_err());
        d.sector_size = 512;
        d.size = 513; // not a multiple
        assert!(d.validate().is_err());
        d.size = 1024;
        assert!(d.validate().is_ok());
    }

    #[test]
    fn partition_naming_follows_linux_rules() {
        let sda = Disk::new("sda", 1024, DiskBus::Ata);
        assert_eq!(sda.partition_name(1), "sda1");
        let nvme = Disk::new("nvme0n1", 1024, DiskBus::Nvme);
        assert_eq!(nvme.partition_name(2), "nvme0n1p2");
        assert_eq!(nvme.disk_type, DiskType::Nvme);
    }

    #[test]
    fn install_candidate_rules() {
        let mut d = big(DiskBus::Scsi);
        assert!(d.is_install_candidate());
        d.removable = true;
        assert!(!d.is_install_candidate());
        d.removable = false;
        d.readonly = true;
        assert!(!d.is_install_candidate());

        let usb = Disk::new("sdb", 64 * 1024 * 1024 * 1024, DiskBus::Usb);
        assert!(usb.removable);
        assert!(!usb.is_install_candidate());

        let tiny = Disk::new("sdc", 1024, DiskBus::Scsi);
        assert!(!tiny.is_install_candidate());
    }

    #[test]
    fn paths_and_sectors() {
        let mut d = big(DiskBus::Scsi);
        d.size = 4096;
        d.sector_size = 512;
        assert_eq!(d.sector_count(), 8);
        assert_eq!(d.dev_path(), "/dev/sda");
        d.serial = Some("S123".to_string());
        assert_eq!(d.serial.as_deref(), Some("S123"));
    }
}
