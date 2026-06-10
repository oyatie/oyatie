//! Memory module (DIMM) information.
//!
//! Mirrors the Talos `MemoryModule` COSI resource populated from each SMBIOS
//! Type 17 *Memory Device* structure (one per DIMM slot). Talos surfaces, per
//! populated slot: device locator, bank locator, manufacturer, product/part
//! number, serial, size, and speed.
//!
//! Empty slots (size == 0) are skipped, exactly like Talos.

use crate::smbios::{Structure, StructureType};

/// A decoded memory module, mirroring the Talos `MemoryModule` spec.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MemoryModule {
    /// Slot label (e.g. `DIMM_A1`).
    pub device_locator: String,
    /// Bank label (e.g. `BANK 0`).
    pub bank_locator: String,
    /// Manufacturer string.
    pub manufacturer: String,
    /// Part number string.
    pub product_name: String,
    /// Serial number.
    pub serial_number: String,
    /// Asset tag.
    pub asset_tag: String,
    /// Size in megabytes (0 for an empty slot).
    pub size_mb: u32,
    /// Configured/operating speed in MT/s (offset 0x20, SMBIOS 2.3+).
    pub speed_mts: u16,
    /// Type of memory, decoded from the device-type byte.
    pub memory_type: MemoryType,
}

impl MemoryModule {
    /// Decode a Type 17 structure into a [`MemoryModule`].
    ///
    /// Returns `None` for non-Type-17 structures or empty slots (size 0).
    pub fn decode(s: &Structure) -> Option<MemoryModule> {
        if s.ty != StructureType::MemoryDevice {
            return None;
        }
        let size_mb = decode_size(s)?;
        if size_mb == 0 {
            return None;
        }

        let memory_type = MemoryType::from_byte(s.byte(0x12).unwrap_or(0x02));
        let speed = s.word(0x15).unwrap_or(0);

        Some(MemoryModule {
            device_locator: s.string(0x10),
            bank_locator: s.string(0x11),
            manufacturer: s.string(0x17),
            serial_number: s.string(0x18),
            asset_tag: s.string(0x19),
            product_name: s.string(0x1a),
            size_mb,
            speed_mts: speed,
            memory_type,
        })
    }

    /// The size rendered in gigabytes (integer division).
    pub fn size_gb(&self) -> u32 {
        self.size_mb / 1024
    }
}

/// Decode the SMBIOS Type 17 *size* field (offset 0x0c, a 16-bit value).
///
/// Per spec: `0x0000` = empty slot, `0xFFFF` = unknown, `0x7FFF` = "use the
/// 32-bit extended size at offset 0x1c". Otherwise bit 15 selects the unit:
/// clear => megabytes, set => kilobytes.
fn decode_size(s: &Structure) -> Option<u32> {
    let raw = s.word(0x0c)?;
    if raw == 0x0000 {
        return Some(0);
    }
    if raw == 0xffff {
        // Unknown — treat as empty for inventory purposes.
        return Some(0);
    }
    if raw == 0x7fff {
        // Extended size in MB at offset 0x1c (bit 31 reserved).
        let ext = s.dword(0x1c).unwrap_or(0) & 0x7fff_ffff;
        return Some(ext);
    }
    let in_kb = (raw & 0x8000) != 0;
    let magnitude = (raw & 0x7fff) as u32;
    if in_kb {
        // Round kilobytes up to whole megabytes.
        Some(magnitude.div_ceil(1024))
    } else {
        Some(magnitude)
    }
}

/// SMBIOS memory device type (Type 17 offset 0x12), the common subset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MemoryType {
    /// Unknown / not reported.
    #[default]
    Unknown,
    /// DDR2 SDRAM.
    Ddr2,
    /// DDR3 SDRAM.
    Ddr3,
    /// DDR4 SDRAM.
    Ddr4,
    /// DDR5 SDRAM.
    Ddr5,
    /// Low-power DDR4.
    Lpddr4,
    /// Low-power DDR5.
    Lpddr5,
    /// Some other reported type.
    Other,
}

impl MemoryType {
    /// Decode the device-type byte.
    pub fn from_byte(b: u8) -> Self {
        match b {
            0x01 | 0x02 => MemoryType::Other, // Other / Unknown -> Other
            0x13 => MemoryType::Ddr2,
            0x18 => MemoryType::Ddr3,
            0x1a => MemoryType::Ddr4,
            0x22 => MemoryType::Ddr5,
            0x1e => MemoryType::Lpddr4,
            0x23 => MemoryType::Lpddr5,
            _ => MemoryType::Unknown,
        }
    }

    /// A human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            MemoryType::Unknown => "Unknown",
            MemoryType::Ddr2 => "DDR2",
            MemoryType::Ddr3 => "DDR3",
            MemoryType::Ddr4 => "DDR4",
            MemoryType::Ddr5 => "DDR5",
            MemoryType::Lpddr4 => "LPDDR4",
            MemoryType::Lpddr5 => "LPDDR5",
            MemoryType::Other => "Other",
        }
    }
}

/// Aggregate memory summary across all populated modules.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MemorySummary {
    /// Number of populated DIMM slots.
    pub populated_slots: usize,
    /// Total installed memory in megabytes.
    pub total_mb: u64,
}

impl MemorySummary {
    /// Summarize a list of decoded modules.
    pub fn from_modules(modules: &[MemoryModule]) -> Self {
        let mut s = MemorySummary::default();
        for m in modules {
            s.populated_slots += 1;
            s.total_mb += m.size_mb as u64;
        }
        s
    }

    /// Total memory in gigabytes (integer division).
    pub fn total_gb(&self) -> u64 {
        self.total_mb / 1024
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smbios::{SmbiosBuilder, SmbiosTable, StructureType};

    /// Build a Type 17 memory device. `size_word` is the raw size field;
    /// `mem_type` is the device-type byte; `speed` the 0x15 word.
    fn dimm(size_word: u16, mem_type: u8, speed: u16) -> Structure {
        let mut f = [0u8; 0x1b]; // reach offset 0x1a
        // size field at 0x0c
        f[0x0c] = (size_word & 0xff) as u8;
        f[0x0d] = (size_word >> 8) as u8;
        f[0x10] = 1; // device locator idx
        f[0x11] = 2; // bank locator idx
        f[0x12] = mem_type; // memory type
        f[0x15] = (speed & 0xff) as u8;
        f[0x16] = (speed >> 8) as u8;
        f[0x17] = 3; // manufacturer idx
        f[0x18] = 4; // serial idx
        f[0x19] = 5; // asset tag idx
        f[0x1a] = 6; // part number idx

        let raw = SmbiosBuilder::new()
            .structure(
                StructureType::MemoryDevice,
                0x1100,
                &f[4..],
                &[
                    "DIMM_A1", "BANK 0", "Samsung", "SER123", "ASSET1", "M393A2K",
                ],
            )
            .finish();
        let table = SmbiosTable::parse(&raw).unwrap();
        table.first(StructureType::MemoryDevice).unwrap().clone()
    }

    #[test]
    fn decodes_populated_dimm_in_megabytes() {
        // 16384 MB, bit 15 clear => MB. DDR4 (0x1a), 3200 MT/s.
        let s = dimm(16384, 0x1a, 3200);
        let m = MemoryModule::decode(&s).unwrap();
        assert_eq!(m.device_locator, "DIMM_A1");
        assert_eq!(m.bank_locator, "BANK 0");
        assert_eq!(m.manufacturer, "Samsung");
        assert_eq!(m.serial_number, "SER123");
        assert_eq!(m.product_name, "M393A2K");
        assert_eq!(m.size_mb, 16384);
        assert_eq!(m.size_gb(), 16);
        assert_eq!(m.speed_mts, 3200);
        assert_eq!(m.memory_type, MemoryType::Ddr4);
    }

    #[test]
    fn empty_slot_skipped() {
        let s = dimm(0, 0x02, 0);
        assert!(MemoryModule::decode(&s).is_none());
    }

    #[test]
    fn unknown_size_treated_as_empty() {
        let s = dimm(0xffff, 0x1a, 0);
        assert!(MemoryModule::decode(&s).is_none());
    }

    #[test]
    fn kilobyte_unit_size_rounds_to_mb() {
        // bit 15 set => kilobytes. 0x8400 => magnitude 0x0400 = 1024 KB = 1 MB.
        let s = dimm(0x8400, 0x1a, 2666);
        let m = MemoryModule::decode(&s).unwrap();
        assert_eq!(m.size_mb, 1);
    }

    #[test]
    fn memory_type_decoding() {
        assert_eq!(MemoryType::from_byte(0x18), MemoryType::Ddr3);
        assert_eq!(MemoryType::from_byte(0x1a), MemoryType::Ddr4);
        assert_eq!(MemoryType::from_byte(0x22), MemoryType::Ddr5);
        assert_eq!(MemoryType::from_byte(0x99), MemoryType::Unknown);
        assert_eq!(MemoryType::Ddr5.label(), "DDR5");
    }

    #[test]
    fn summary_totals() {
        let modules = vec![
            MemoryModule {
                size_mb: 16384,
                ..Default::default()
            },
            MemoryModule {
                size_mb: 16384,
                ..Default::default()
            },
        ];
        let summary = MemorySummary::from_modules(&modules);
        assert_eq!(summary.populated_slots, 2);
        assert_eq!(summary.total_mb, 32768);
        assert_eq!(summary.total_gb(), 32);
    }
}
