//! # talos-hardware
//!
//! Discovers and models node hardware inventory for the operating-system Talos port.
//! Mirrors the Talos subsystems under
//! `internal/app/machined/pkg/controllers/hardware` and the COSI specs under
//! `pkg/machinery/resources/hardware`:
//!
//! * [`smbios`] — SMBIOS / DMI structure-table parsing: header decoding, the
//!   NUL-terminated string area, the canonical UUID byte-swap, and decoders for
//!   the Type 0 (BIOS) and Type 1 (System) structures.
//! * [`processor`] — Type 4 *Processor Information*: socket status, clock
//!   speeds, core/thread counts, and an aggregate [`processor::CpuTopology`].
//! * [`memory_module`] — Type 17 *Memory Device*: size-unit decoding, DIMM
//!   labels, memory type, and an aggregate [`memory_module::MemorySummary`].
//! * [`pci`] — PCI device enumeration: addresses, class codes, and the
//!   [`pci::PciScanner`] boundary.
//! * [`controller`] — the reconcile logic that turns the SMBIOS table and PCI
//!   scan into COSI resources ([`controller::HardwareInventory`]), stamping
//!   each with [`os_kernel`] metadata.
//!
//! OS boundaries (the DMI byte table, the PCI bus) are modeled as the
//! [`smbios::SmbiosSource`] and [`pci::PciScanner`] traits with in-memory
//! implementations used by the controller and unit tests. The crate uses only
//! the standard library plus an internal path dependency on `talos-core`.

pub mod controller;
pub mod memory_module;
pub mod pci;
pub mod processor;
pub mod smbios;

pub use controller::{HardwareController, HardwareInventory, OWNER, Resource, SystemInformation};
pub use memory_module::{MemoryModule, MemorySummary, MemoryType};
pub use pci::{MemoryPciScanner, PciAddress, PciClass, PciDevice, PciError, PciScanner};
pub use processor::{CpuTopology, Processor, ProcessorStatus};
pub use smbios::{
    BiosInfo, MemorySmbios, SmbiosBuilder, SmbiosError, SmbiosSource, SmbiosTable, SmbiosUuid,
    Structure, StructureType, SystemInfo,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// End-to-end smoke test exercising the public surface: build a synthetic
    /// SMBIOS blob and PCI scan, reconcile, and assert the inventory.
    #[test]
    fn end_to_end_inventory() {
        // System info with a known UUID.
        let mut sys = vec![1u8, 2, 3, 4];
        sys.extend_from_slice(&[
            0x33, 0x22, 0x11, 0x00, 0x55, 0x44, 0x77, 0x66, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
            0xee, 0xff,
        ]);
        sys.push(0x06);

        let raw = SmbiosBuilder::new()
            .structure(
                StructureType::System,
                1,
                &sys,
                &["Acme", "MetalBox", "1.0", "SN1"],
            )
            .finish();

        let smbios = MemorySmbios::new(raw);
        let pci = MemoryPciScanner::new().with(PciDevice::new(
            PciAddress::parse("0000:00:1f.6").unwrap(),
            0x8086,
            0x15bb,
            0x02_00_00,
        ));

        let inv = HardwareController::new()
            .reconcile_all(&smbios, &pci)
            .unwrap();

        assert_eq!(inv.system.as_ref().unwrap().spec.serial(), "SN1");
        assert_eq!(inv.pci_devices.len(), 1);
        assert!(inv.pci_devices[0].spec.is_network());
    }

    #[test]
    fn public_reexports_are_usable() {
        let uuid = SmbiosUuid::from_smbios_bytes([0; 16]);
        assert!(uuid.is_unset());
        assert_eq!(MemoryType::Ddr4.label(), "DDR4");
        assert_eq!(PciClass::Network.label(), "Network controller");
    }
}
