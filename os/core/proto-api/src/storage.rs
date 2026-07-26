//! The `StorageService` API surface (block device / disk discovery).
//!
//! Mirrors `pkg/machinery/api/storage/storage.proto`: the `Disks` call that
//! lists block devices with their model/type/size, used by `talosctl disks`
//! and the installer to pick an install target.

use crate::common::{ApiError, Code, RequestContext};
use os_kernel::role::Role;

/// The transport/bus a disk is attached over, mirroring `storage.Disk.Type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskType {
    /// Unknown bus.
    Unknown,
    /// SSD over SATA/SAS.
    Ssd,
    /// Rotational hard disk.
    Hdd,
    /// NVMe device.
    Nvme,
    /// SD card.
    Sd,
}

impl DiskType {
    /// Lowercase string form.
    pub fn as_str(self) -> &'static str {
        match self {
            DiskType::Unknown => "unknown",
            DiskType::Ssd => "ssd",
            DiskType::Hdd => "hdd",
            DiskType::Nvme => "nvme",
            DiskType::Sd => "sd",
        }
    }
}

/// A discovered block device, mirroring `storage.Disk`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Disk {
    /// The device node (`/dev/sda`, `/dev/nvme0n1`).
    pub device_name: String,
    /// The model string.
    pub model: String,
    /// The serial number.
    pub serial: String,
    /// Total size in bytes.
    pub size: u64,
    /// The bus/transport type.
    pub disk_type: DiskType,
    /// Whether the device is read-only.
    pub readonly: bool,
    /// Whether the device is removable.
    pub removable: bool,
    /// The bus path / system path.
    pub bus_path: String,
}

impl Disk {
    /// Human-readable size in GiB (truncated).
    pub fn size_gib(&self) -> u64 {
        self.size / (1024 * 1024 * 1024)
    }

    /// Whether this disk is a viable Talos install target: writable, not
    /// removable, and at least the minimum size. Mirrors the installer's disk
    /// eligibility filter.
    pub fn is_install_candidate(&self, min_bytes: u64) -> bool {
        !self.readonly && !self.removable && self.size >= min_bytes
    }
}

/// The disk-discovery backend, behind a trait.
pub trait StorageBackend {
    /// All discovered block devices.
    fn disks(&self) -> Vec<Disk>;
}

/// The `StorageService`.
pub struct StorageService<B: StorageBackend> {
    backend: B,
}

impl<B: StorageBackend> StorageService<B> {
    /// Wrap a backend.
    pub fn new(backend: B) -> Self {
        StorageService { backend }
    }

    /// `Disks`: list disks sorted by device name.
    pub fn disks(&self, ctx: &RequestContext) -> Result<Vec<Disk>, ApiError> {
        ctx.authorize(Role::Reader)?;
        let mut disks = self.backend.disks();
        disks.sort_by(|a, b| a.device_name.cmp(&b.device_name));
        Ok(disks)
    }

    /// Pick the smallest eligible install target of at least `min_bytes`,
    /// preferring NVMe/SSD over HDD. Mirrors the installer's default selection.
    pub fn select_install_disk(
        &self,
        ctx: &RequestContext,
        min_bytes: u64,
    ) -> Result<Disk, ApiError> {
        let mut candidates: Vec<Disk> = self
            .disks(ctx)?
            .into_iter()
            .filter(|d| d.is_install_candidate(min_bytes))
            .collect();
        if candidates.is_empty() {
            return Err(ApiError::new(
                Code::NotFound,
                format!("no install-eligible disk of at least {min_bytes} bytes"),
            ));
        }
        // Prefer faster media, then smallest size, then device name.
        candidates.sort_by(|a, b| {
            media_rank(a.disk_type)
                .cmp(&media_rank(b.disk_type))
                .then(a.size.cmp(&b.size))
                .then(a.device_name.cmp(&b.device_name))
        });
        Ok(candidates.into_iter().next().unwrap())
    }
}

/// Lower rank = preferred install media.
fn media_rank(t: DiskType) -> u8 {
    match t {
        DiskType::Nvme => 0,
        DiskType::Ssd => 1,
        DiskType::Hdd => 2,
        DiskType::Sd => 3,
        DiskType::Unknown => 4,
    }
}

/// An in-memory storage view for tests.
#[derive(Debug, Clone, Default)]
pub struct InMemoryStorage {
    /// The disks.
    pub disks: Vec<Disk>,
}

impl InMemoryStorage {
    /// A typical node with one NVMe system disk and a removable USB stick.
    pub fn typical() -> Self {
        InMemoryStorage {
            disks: vec![
                Disk {
                    device_name: "/dev/nvme0n1".into(),
                    model: "Samsung SSD 980".into(),
                    serial: "S1".into(),
                    size: 512 * 1024 * 1024 * 1024,
                    disk_type: DiskType::Nvme,
                    readonly: false,
                    removable: false,
                    bus_path: "pci-0000:01:00.0-nvme-1".into(),
                },
                Disk {
                    device_name: "/dev/sda".into(),
                    model: "Generic USB".into(),
                    serial: "U1".into(),
                    size: 16 * 1024 * 1024 * 1024,
                    disk_type: DiskType::Sd,
                    readonly: false,
                    removable: true,
                    bus_path: "usb-1".into(),
                },
                Disk {
                    device_name: "/dev/sdb".into(),
                    model: "Spinning Rust".into(),
                    serial: "H1".into(),
                    size: 2 * 1024 * 1024 * 1024 * 1024,
                    disk_type: DiskType::Hdd,
                    readonly: false,
                    removable: false,
                    bus_path: "ata-2".into(),
                },
            ],
        }
    }
}

impl StorageBackend for InMemoryStorage {
    fn disks(&self) -> Vec<Disk> {
        self.disks.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEN_GIB: u64 = 10 * 1024 * 1024 * 1024;

    #[test]
    fn disks_sorted_and_sized() {
        let svc = StorageService::new(InMemoryStorage::typical());
        let disks = svc.disks(&RequestContext::admin_local()).unwrap();
        assert_eq!(disks[0].device_name, "/dev/nvme0n1");
        assert_eq!(disks[0].size_gib(), 512);
        assert_eq!(disks[0].disk_type.as_str(), "nvme");
    }

    #[test]
    fn install_candidate_excludes_removable() {
        let storage = InMemoryStorage::typical();
        let usb = storage.disks.iter().find(|d| d.removable).unwrap();
        assert!(!usb.is_install_candidate(TEN_GIB));
        let nvme = &storage.disks[0];
        assert!(nvme.is_install_candidate(TEN_GIB));
    }

    #[test]
    fn select_install_disk_prefers_nvme() {
        let svc = StorageService::new(InMemoryStorage::typical());
        let chosen = svc
            .select_install_disk(&RequestContext::admin_local(), TEN_GIB)
            .unwrap();
        // NVMe beats HDD despite being smaller-ranked-first.
        assert_eq!(chosen.device_name, "/dev/nvme0n1");
    }

    #[test]
    fn no_candidate_is_not_found() {
        let svc = StorageService::new(InMemoryStorage::typical());
        let huge = 10 * 1024 * 1024 * 1024 * 1024u64; // 10 TiB
        assert_eq!(
            svc.select_install_disk(&RequestContext::admin_local(), huge)
                .unwrap_err()
                .code,
            Code::NotFound
        );
    }

    #[test]
    fn read_gated() {
        let svc = StorageService::new(InMemoryStorage::typical());
        let nobody = RequestContext::with_roles(os_kernel::role::RoleSet::new());
        assert_eq!(svc.disks(&nobody).unwrap_err().code, Code::PermissionDenied);
    }
}
