//! PCI device discovery.
//!
//! Talos enumerates PCI devices from sysfs (`/sys/bus/pci/devices/*`) and
//! resolves vendor/device/class names against the PCI ID database. This module
//! models a PCI address (`domain:bus:device.function`), the config-space
//! identifiers (vendor, device, class), and an [`PciScanner`] trait whose
//! in-memory implementation feeds the controller and tests.
//!
//! Class codes follow the PCI spec base-class table; this models the subset
//! relevant to a node (network, storage, display, bridges, ...).

use std::collections::BTreeMap;
use std::fmt;

/// A PCI bus/device/function address (`DDDD:BB:DD.F`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PciAddress {
    /// PCI domain (segment), usually 0.
    pub domain: u16,
    /// Bus number (0..=255).
    pub bus: u8,
    /// Device/slot number (0..=31).
    pub device: u8,
    /// Function number (0..=7).
    pub function: u8,
}

impl PciAddress {
    /// Construct, validating the device (<32) and function (<8) ranges.
    pub fn new(domain: u16, bus: u8, device: u8, function: u8) -> Result<Self, PciError> {
        if device > 31 {
            return Err(PciError::InvalidAddress(format!("device {device} > 31")));
        }
        if function > 7 {
            return Err(PciError::InvalidAddress(format!("function {function} > 7")));
        }
        Ok(PciAddress {
            domain,
            bus,
            device,
            function,
        })
    }

    /// Parse the canonical sysfs form `0000:00:1f.2`.
    pub fn parse(s: &str) -> Result<Self, PciError> {
        // domain:bus:device.function
        let (head, func) = s
            .rsplit_once('.')
            .ok_or_else(|| PciError::InvalidAddress(s.to_string()))?;
        let parts: Vec<&str> = head.split(':').collect();
        if parts.len() != 3 {
            return Err(PciError::InvalidAddress(s.to_string()));
        }
        let domain = u16::from_str_radix(parts[0], 16)
            .map_err(|_| PciError::InvalidAddress(s.to_string()))?;
        let bus = u8::from_str_radix(parts[1], 16)
            .map_err(|_| PciError::InvalidAddress(s.to_string()))?;
        let device = u8::from_str_radix(parts[2], 16)
            .map_err(|_| PciError::InvalidAddress(s.to_string()))?;
        let function =
            u8::from_str_radix(func, 16).map_err(|_| PciError::InvalidAddress(s.to_string()))?;
        PciAddress::new(domain, bus, device, function)
    }
}

impl fmt::Display for PciAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04x}:{:02x}:{:02x}.{:x}",
            self.domain, self.bus, self.device, self.function
        )
    }
}

/// PCI base class codes (high byte of the 24-bit class triple).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PciClass {
    /// 0x01 Mass storage controller.
    MassStorage,
    /// 0x02 Network controller.
    Network,
    /// 0x03 Display controller.
    Display,
    /// 0x04 Multimedia controller.
    Multimedia,
    /// 0x06 Bridge device.
    Bridge,
    /// 0x07 Communication controller.
    Communication,
    /// 0x0c Serial bus controller (USB, SMBus, ...).
    SerialBus,
    /// Any other base class, raw value preserved.
    Other(u8),
}

impl PciClass {
    /// Decode the base-class byte.
    pub fn from_base(b: u8) -> Self {
        match b {
            0x01 => PciClass::MassStorage,
            0x02 => PciClass::Network,
            0x03 => PciClass::Display,
            0x04 => PciClass::Multimedia,
            0x06 => PciClass::Bridge,
            0x07 => PciClass::Communication,
            0x0c => PciClass::SerialBus,
            other => PciClass::Other(other),
        }
    }

    /// A human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            PciClass::MassStorage => "Mass storage controller",
            PciClass::Network => "Network controller",
            PciClass::Display => "Display controller",
            PciClass::Multimedia => "Multimedia controller",
            PciClass::Bridge => "Bridge",
            PciClass::Communication => "Communication controller",
            PciClass::SerialBus => "Serial bus controller",
            PciClass::Other(_) => "Unknown",
        }
    }
}

/// A discovered PCI device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PciDevice {
    /// The device's PCI address.
    pub address: PciAddress,
    /// 16-bit vendor ID (config space 0x00).
    pub vendor_id: u16,
    /// 16-bit device ID (config space 0x02).
    pub device_id: u16,
    /// 24-bit class triple `(base, sub, prog-if)`.
    pub class_code: u32,
    /// Optional resolved vendor name.
    pub vendor_name: Option<String>,
    /// Optional resolved product name.
    pub product_name: Option<String>,
    /// Optional bound kernel driver (e.g. `nvme`, `i40e`).
    pub driver: Option<String>,
}

impl PciDevice {
    /// Construct a minimal device with IDs only.
    pub fn new(address: PciAddress, vendor_id: u16, device_id: u16, class_code: u32) -> Self {
        PciDevice {
            address,
            vendor_id,
            device_id,
            class_code,
            vendor_name: None,
            product_name: None,
            driver: None,
        }
    }

    /// The decoded base class.
    pub fn class(&self) -> PciClass {
        PciClass::from_base((self.class_code >> 16) as u8)
    }

    /// The `vendor:device` identifier in lowercase hex (e.g. `8086:10fb`).
    pub fn ids(&self) -> String {
        format!("{:04x}:{:04x}", self.vendor_id, self.device_id)
    }

    /// Whether this device is a network controller.
    pub fn is_network(&self) -> bool {
        self.class() == PciClass::Network
    }
}

/// Errors raised by PCI parsing/scanning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PciError {
    /// A PCI address string or component was malformed.
    InvalidAddress(String),
    /// The scanner failed to read device data.
    ScanFailed(String),
}

impl fmt::Display for PciError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PciError::InvalidAddress(s) => write!(f, "invalid pci address: {s}"),
            PciError::ScanFailed(s) => write!(f, "pci scan failed: {s}"),
        }
    }
}

impl std::error::Error for PciError {}

impl From<PciError> for os_kernel::Error {
    fn from(e: PciError) -> Self {
        match e {
            PciError::InvalidAddress(s) => os_kernel::Error::parse(s),
            PciError::ScanFailed(s) => os_kernel::Error::Other(s),
        }
    }
}

/// OS boundary: enumerates PCI devices (real impl reads sysfs).
pub trait PciScanner {
    /// Return all discovered PCI devices.
    fn scan(&self) -> Result<Vec<PciDevice>, PciError>;
}

/// In-memory [`PciScanner`] holding a fixed device list, keyed by address.
#[derive(Debug, Clone, Default)]
pub struct MemoryPciScanner {
    devices: BTreeMap<PciAddress, PciDevice>,
}

impl MemoryPciScanner {
    /// An empty scanner.
    pub fn new() -> Self {
        MemoryPciScanner::default()
    }

    /// Add (or replace) a device, returning `self` for chaining.
    pub fn with(mut self, device: PciDevice) -> Self {
        self.devices.insert(device.address, device);
        self
    }

    /// Number of devices held.
    pub fn len(&self) -> usize {
        self.devices.len()
    }

    /// Whether the scanner is empty.
    pub fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }
}

impl PciScanner for MemoryPciScanner {
    fn scan(&self) -> Result<Vec<PciDevice>, PciError> {
        Ok(self.devices.values().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_parse_and_display_roundtrip() {
        let a = PciAddress::parse("0000:00:1f.2").unwrap();
        assert_eq!(a.domain, 0);
        assert_eq!(a.bus, 0);
        assert_eq!(a.device, 0x1f);
        assert_eq!(a.function, 2);
        assert_eq!(a.to_string(), "0000:00:1f.2");
    }

    #[test]
    fn address_validation_rejects_out_of_range() {
        assert!(PciAddress::new(0, 0, 32, 0).is_err());
        assert!(PciAddress::new(0, 0, 0, 8).is_err());
        assert!(PciAddress::parse("nonsense").is_err());
        assert!(PciAddress::parse("0000:00:1f").is_err());
    }

    #[test]
    fn class_decoding_from_class_code() {
        let dev = PciDevice::new(
            PciAddress::new(0, 0, 0, 0).unwrap(),
            0x8086,
            0x10fb,
            0x02_00_00,
        );
        assert_eq!(dev.class(), PciClass::Network);
        assert!(dev.is_network());
        assert_eq!(dev.ids(), "8086:10fb");
        assert_eq!(dev.class().label(), "Network controller");
    }

    #[test]
    fn class_decoding_storage_and_bridge() {
        assert_eq!(PciClass::from_base(0x01), PciClass::MassStorage);
        assert_eq!(PciClass::from_base(0x06), PciClass::Bridge);
        assert_eq!(PciClass::from_base(0x0c), PciClass::SerialBus);
        assert!(matches!(PciClass::from_base(0xff), PciClass::Other(0xff)));
    }

    #[test]
    fn memory_scanner_returns_sorted_devices() {
        let scanner = MemoryPciScanner::new()
            .with(PciDevice::new(
                PciAddress::new(0, 0, 0x1f, 2).unwrap(),
                0x8086,
                0xa102,
                0x01_06_01,
            ))
            .with(PciDevice::new(
                PciAddress::new(0, 0, 2, 0).unwrap(),
                0x8086,
                0x5917,
                0x03_00_00,
            ));
        let devs = scanner.scan().unwrap();
        assert_eq!(devs.len(), 2);
        // BTreeMap keeps address order: 00:02.0 before 00:1f.2
        assert_eq!(devs[0].address.device, 2);
        assert_eq!(devs[1].address.device, 0x1f);
    }
}
