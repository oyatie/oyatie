//! Hardware resource population controller.
//!
//! Mirrors Talos's `internal/app/machined/pkg/controllers/hardware`:
//!
//! * the **SMBIOS / system-info controller** reads the DMI table and emits the
//!   system `SystemInformation` resource (manufacturer, product, UUID, serial)
//!   plus one `Processor` and one `MemoryModule` resource per populated socket
//!   and DIMM, and
//! * the **PCI controller** enumerates PCI devices into `PCIDevice` resources.
//!
//! All of this is a pure reconcile over the [`SmbiosSource`] / [`PciScanner`]
//! boundaries into a [`HardwareInventory`], which is the in-memory analog of
//! the COSI store the real controller writes to. Each emitted item carries
//! COSI [`Metadata`] so it round-trips through the resource machinery.

use os_kernel::resource::{Metadata, Namespace, ResourceKind};
use os_kernel::{Error, ResourceId, Result};

use crate::memory_module::{MemoryModule, MemorySummary};
use crate::pci::{PciDevice, PciScanner};
use crate::processor::{CpuTopology, Processor};
use crate::smbios::{BiosInfo, SmbiosSource, SmbiosTable, StructureType, SystemInfo};

/// The controller owner string stamped onto every resource it emits, matching
/// the Talos controller name convention.
pub const OWNER: &str = "hardware.SMBIOSController";

/// COSI kind for the system information resource.
pub const KIND_SYSTEM: &str = "SystemInformations.hardware.talos.dev";
/// COSI kind for a processor resource.
pub const KIND_PROCESSOR: &str = "Processors.hardware.talos.dev";
/// COSI kind for a memory-module resource.
pub const KIND_MEMORY: &str = "MemoryModules.hardware.talos.dev";
/// COSI kind for a PCI device resource.
pub const KIND_PCI: &str = "PCIDevices.hardware.talos.dev";

/// One COSI resource: typed metadata paired with its decoded spec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource<T> {
    /// COSI metadata (namespace/kind/id, version, owner, ...).
    pub metadata: Metadata,
    /// The decoded resource spec.
    pub spec: T,
}

impl<T> Resource<T> {
    /// The resource id as a string.
    pub fn id(&self) -> &str {
        self.metadata.pointer().id.as_str()
    }
}

/// The aggregate system info resource Talos exposes as a single `id == system`
/// resource (`talosctl get systeminformation`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SystemInformation {
    /// Decoded Type 1 system info.
    pub system: SystemInfo,
    /// Decoded Type 0 BIOS info.
    pub bios: BiosInfo,
    /// CPU topology summary.
    pub cpu: CpuTopology,
    /// Memory summary.
    pub memory: MemorySummary,
}

impl SystemInformation {
    /// The node UUID (empty when firmware did not report one).
    pub fn uuid(&self) -> &str {
        &self.system.uuid
    }

    /// The node serial number.
    pub fn serial(&self) -> &str {
        &self.system.serial_number
    }
}

/// The in-memory COSI store the controller populates — the analog of the
/// resource state the real Talos controller writes into.
#[derive(Debug, Clone, Default)]
pub struct HardwareInventory {
    /// The single system-information resource (id `system`).
    pub system: Option<Resource<SystemInformation>>,
    /// Per-socket processors, in socket order.
    pub processors: Vec<Resource<Processor>>,
    /// Per-DIMM memory modules, in slot order.
    pub memory_modules: Vec<Resource<MemoryModule>>,
    /// PCI devices.
    pub pci_devices: Vec<Resource<PciDevice>>,
}

impl HardwareInventory {
    /// Look up a processor resource by its id (e.g. `CPU0`).
    pub fn processor(&self, id: &str) -> Option<&Resource<Processor>> {
        self.processors.iter().find(|r| r.id() == id)
    }

    /// Look up a memory module by its id (e.g. `DIMM_A1`).
    pub fn memory_module(&self, id: &str) -> Option<&Resource<MemoryModule>> {
        self.memory_modules.iter().find(|r| r.id() == id)
    }

    /// Total number of resources held (system counts as one when present).
    pub fn resource_count(&self) -> usize {
        self.system.is_some() as usize
            + self.processors.len()
            + self.memory_modules.len()
            + self.pci_devices.len()
    }
}

/// Build typed COSI metadata in the `hardware` namespace for a kind/id.
fn meta(kind: &str, id: &str) -> Result<Metadata> {
    let ns = Namespace::runtime();
    let kind = ResourceKind::new(kind)?;
    let id = ResourceId::new(id)?;
    let mut m = Metadata::new(ns, kind, id);
    m.set_owner(OWNER)?;
    Ok(m)
}

/// Sanitize an SMBIOS label into a valid COSI [`ResourceId`].
///
/// SMBIOS socket/DIMM labels can contain spaces and parentheses, which are not
/// allowed in a [`ResourceId`]; Talos slugifies them. We replace any
/// disallowed character with `-` and collapse to a non-empty fallback.
fn slug_id(label: &str, fallback: &str, index: usize) -> String {
    let mut out = String::with_capacity(label.len());
    for c in label.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '/' | ':' | '-') {
            out.push(c);
        } else {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        format!("{fallback}-{index}")
    } else {
        trimmed
    }
}

/// The hardware controller. Stateless apart from the resources it produces.
#[derive(Debug, Default)]
pub struct HardwareController;

impl HardwareController {
    /// A fresh controller.
    pub fn new() -> Self {
        HardwareController
    }

    /// Reconcile the SMBIOS table from `source` into an inventory: system info,
    /// processors, and memory modules.
    pub fn reconcile_smbios(
        &self,
        source: &impl SmbiosSource,
        inventory: &mut HardwareInventory,
    ) -> Result<()> {
        let raw = source.read_table()?;
        let table = SmbiosTable::parse(&raw)?;
        self.populate_from_table(&table, inventory)
    }

    /// Populate an inventory from an already-parsed table. Split out so callers
    /// (and tests) can supply a [`SmbiosTable`] directly.
    pub fn populate_from_table(
        &self,
        table: &SmbiosTable,
        inventory: &mut HardwareInventory,
    ) -> Result<()> {
        // Type 1 + Type 0 -> SystemInformation.
        let system = table
            .first(StructureType::System)
            .map(SystemInfo::decode)
            .unwrap_or_default();
        let bios = table
            .first(StructureType::Bios)
            .map(BiosInfo::decode)
            .unwrap_or_default();

        // Type 4 -> processors (populated sockets only).
        let mut processors = Vec::new();
        for (i, s) in table.all(StructureType::Processor).into_iter().enumerate() {
            if let Some(p) = Processor::decode(s) {
                let id = slug_id(&p.socket, "CPU", i);
                processors.push(Resource {
                    metadata: meta(KIND_PROCESSOR, &id)?,
                    spec: p,
                });
            }
        }

        // Type 17 -> memory modules (populated slots only).
        let mut memory_modules = Vec::new();
        for (i, s) in table
            .all(StructureType::MemoryDevice)
            .into_iter()
            .enumerate()
        {
            if let Some(m) = MemoryModule::decode(s) {
                let id = slug_id(&m.device_locator, "DIMM", i);
                memory_modules.push(Resource {
                    metadata: meta(KIND_MEMORY, &id)?,
                    spec: m,
                });
            }
        }

        let cpu = CpuTopology::from_processors(
            &processors
                .iter()
                .map(|r| r.spec.clone())
                .collect::<Vec<_>>(),
        );
        let memory = MemorySummary::from_modules(
            &memory_modules
                .iter()
                .map(|r| r.spec.clone())
                .collect::<Vec<_>>(),
        );

        let sys_res = Resource {
            metadata: meta(KIND_SYSTEM, "system")?,
            spec: SystemInformation {
                system,
                bios,
                cpu,
                memory,
            },
        };

        inventory.system = Some(sys_res);
        inventory.processors = processors;
        inventory.memory_modules = memory_modules;
        Ok(())
    }

    /// Reconcile PCI devices from `scanner` into the inventory.
    pub fn reconcile_pci(
        &self,
        scanner: &impl PciScanner,
        inventory: &mut HardwareInventory,
    ) -> Result<()> {
        let devices = scanner.scan().map_err(Error::from)?;
        let mut out = Vec::with_capacity(devices.len());
        for dev in devices {
            let id = dev.address.to_string();
            out.push(Resource {
                metadata: meta(KIND_PCI, &id)?,
                spec: dev,
            });
        }
        inventory.pci_devices = out;
        Ok(())
    }

    /// Full reconcile: SMBIOS then PCI, into a fresh inventory.
    pub fn reconcile_all(
        &self,
        smbios: &impl SmbiosSource,
        pci: &impl PciScanner,
    ) -> Result<HardwareInventory> {
        let mut inv = HardwareInventory::default();
        self.reconcile_smbios(smbios, &mut inv)?;
        self.reconcile_pci(pci, &mut inv)?;
        Ok(inv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pci::{MemoryPciScanner, PciAddress};
    use crate::smbios::{MemorySmbios, SmbiosBuilder};

    /// Build a full synthetic SMBIOS blob: BIOS, System, two CPUs (one
    /// disabled), two DIMMs (one empty).
    fn full_blob() -> Vec<u8> {
        // System info formatted area.
        let mut sys = vec![1u8, 2, 3, 4];
        sys.extend_from_slice(&[
            0x33, 0x22, 0x11, 0x00, 0x55, 0x44, 0x77, 0x66, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
            0xee, 0xff,
        ]);
        sys.push(0x06); // wakeup
        sys.push(5); // sku
        sys.push(6); // family

        // CPU0 enabled.
        let mut cpu0 = vec![0u8; 0x22];
        cpu0[0x00] = 1; // socket idx (relative offset 0x04 in full struct)
        cpu0[0x03] = 2; // manufacturer (offset 0x07)
        cpu0[0x0c] = 3; // version (offset 0x10)
        cpu0[0x10] = 0xa4; // max speed lo (offset 0x14) -> 0x0da4 = 3492
        cpu0[0x11] = 0x0d;
        cpu0[0x12] = 0x60; // boot speed lo (offset 0x16) -> 0x0960 = 2400
        cpu0[0x13] = 0x09;
        cpu0[0x14] = 0x41; // status (offset 0x18) = populated+enabled
        cpu0[0x1f] = 8; // core count (offset 0x23)
        cpu0[0x20] = 8; // core enabled (offset 0x24)
        cpu0[0x21] = 16; // thread count (offset 0x25)

        // CPU1 disabled by user.
        let mut cpu1 = cpu0.clone();
        cpu1[0x14] = 0x42; // disabled by user

        // DIMM0 populated, 16 GB.
        let mut d0 = vec![0u8; 0x17];
        d0[0x08] = 0x00; // size lo (offset 0x0c)
        d0[0x09] = 0x40; // size hi -> 0x4000 = 16384 MB
        d0[0x0c] = 1; // device locator (offset 0x10)
        d0[0x0d] = 2; // bank locator (offset 0x11)
        d0[0x0e] = 0x1a; // memory type (offset 0x12) = DDR4
        d0[0x11] = 0x80; // speed lo (offset 0x15) -> 0x0c80 = 3200
        d0[0x12] = 0x0c;
        d0[0x13] = 3; // manufacturer (offset 0x17)

        // DIMM1 empty.
        let mut d1 = vec![0u8; 0x17];
        d1[0x0c] = 1;
        d1[0x0d] = 2;

        SmbiosBuilder::new()
            .structure(
                StructureType::Bios,
                0x0000,
                &[1u8, 2, 0, 0, 5],
                &["AMI", "F.20"],
            )
            .structure(
                StructureType::System,
                0x0001,
                &sys,
                &["Acme", "MetalBox", "1.0", "SN12345", "SKU-9", "Server"],
            )
            .structure(
                StructureType::Processor,
                0x0400,
                &cpu0,
                &["CPU0", "Intel(R) Corporation", "Xeon Gold"],
            )
            .structure(
                StructureType::Processor,
                0x0401,
                &cpu1,
                &["CPU1", "Intel(R) Corporation", "Xeon Gold"],
            )
            .structure(
                StructureType::MemoryDevice,
                0x1100,
                &d0,
                &["DIMM_A1", "BANK 0", "Samsung"],
            )
            .structure(
                StructureType::MemoryDevice,
                0x1101,
                &d1,
                &["DIMM_B1", "BANK 1"],
            )
            .finish()
    }

    #[test]
    fn reconcile_smbios_populates_system_resource() {
        let src = MemorySmbios::new(full_blob());
        let ctrl = HardwareController::new();
        let mut inv = HardwareInventory::default();
        ctrl.reconcile_smbios(&src, &mut inv).unwrap();

        let sys = inv.system.as_ref().unwrap();
        assert_eq!(sys.id(), "system");
        assert_eq!(sys.metadata.owner(), Some(OWNER));
        assert_eq!(sys.spec.uuid(), "00112233-4455-6677-8899-aabbccddeeff");
        assert_eq!(sys.spec.serial(), "SN12345");
        assert_eq!(sys.spec.system.product_name, "MetalBox");
        assert_eq!(sys.spec.bios.vendor, "AMI");
    }

    #[test]
    fn reconcile_emits_only_enabled_processors_in_topology() {
        let src = MemorySmbios::new(full_blob());
        let ctrl = HardwareController::new();
        let mut inv = HardwareInventory::default();
        ctrl.reconcile_smbios(&src, &mut inv).unwrap();

        // Both populated sockets become resources (even disabled ones).
        assert_eq!(inv.processors.len(), 2);
        let cpu0 = inv.processor("CPU0").unwrap();
        assert_eq!(cpu0.spec.max_speed_mhz, 3492);
        assert_eq!(cpu0.spec.core_count, 8);
        assert_eq!(cpu0.spec.thread_count, 16);

        // Topology only counts the enabled socket.
        let sys = inv.system.as_ref().unwrap();
        assert_eq!(sys.spec.cpu.sockets, 1);
        assert_eq!(sys.spec.cpu.total_cores, 8);
        assert_eq!(sys.spec.cpu.total_threads, 16);
    }

    #[test]
    fn reconcile_skips_empty_dimm_slots() {
        let src = MemorySmbios::new(full_blob());
        let ctrl = HardwareController::new();
        let mut inv = HardwareInventory::default();
        ctrl.reconcile_smbios(&src, &mut inv).unwrap();

        assert_eq!(inv.memory_modules.len(), 1);
        let dimm = inv.memory_module("DIMM_A1").unwrap();
        assert_eq!(dimm.spec.size_mb, 16384);
        assert_eq!(dimm.spec.speed_mts, 3200);

        let sys = inv.system.as_ref().unwrap();
        assert_eq!(sys.spec.memory.populated_slots, 1);
        assert_eq!(sys.spec.memory.total_mb, 16384);
    }

    #[test]
    fn reconcile_pci_emits_device_resources() {
        let scanner = MemoryPciScanner::new()
            .with(PciDevice::new(
                PciAddress::new(0, 0, 0x1f, 6).unwrap(),
                0x8086,
                0x15bb,
                0x02_00_00,
            ))
            .with(PciDevice::new(
                PciAddress::new(0, 0, 0x17, 0).unwrap(),
                0x8086,
                0xa182,
                0x01_06_01,
            ));
        let ctrl = HardwareController::new();
        let mut inv = HardwareInventory::default();
        ctrl.reconcile_pci(&scanner, &mut inv).unwrap();

        assert_eq!(inv.pci_devices.len(), 2);
        let net = inv
            .pci_devices
            .iter()
            .find(|r| r.spec.is_network())
            .unwrap();
        assert_eq!(net.id(), "0000:00:1f.6");
        assert_eq!(net.metadata.owner(), Some(OWNER));
    }

    #[test]
    fn reconcile_all_combines_sources() {
        let src = MemorySmbios::new(full_blob());
        let scanner = MemoryPciScanner::new().with(PciDevice::new(
            PciAddress::new(0, 0, 2, 0).unwrap(),
            0x8086,
            0x5917,
            0x03_00_00,
        ));
        let ctrl = HardwareController::new();
        let inv = ctrl.reconcile_all(&src, &scanner).unwrap();

        // system + 2 cpu + 1 dimm + 1 pci.
        assert_eq!(inv.resource_count(), 5);
    }

    #[test]
    fn slug_id_sanitizes_labels() {
        assert_eq!(slug_id("CPU 0", "CPU", 0), "CPU-0");
        assert_eq!(slug_id("DIMM (A1)", "DIMM", 0), "DIMM--A1");
        assert_eq!(slug_id("", "DIMM", 3), "DIMM-3");
        assert_eq!(slug_id("---", "CPU", 1), "CPU-1");
    }

    #[test]
    fn empty_smbios_table_yields_default_system() {
        let src = MemorySmbios::new(SmbiosBuilder::new().finish());
        let ctrl = HardwareController::new();
        let mut inv = HardwareInventory::default();
        ctrl.reconcile_smbios(&src, &mut inv).unwrap();

        let sys = inv.system.as_ref().unwrap();
        assert_eq!(sys.spec.uuid(), "");
        assert!(inv.processors.is_empty());
        assert!(inv.memory_modules.is_empty());
    }
}
