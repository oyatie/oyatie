//! SMBIOS / DMI table parsing.
//!
//! Mirrors what Talos does in `internal/app/machined/pkg/controllers/hardware`
//! via the `go-smbios` library: it reads the raw SMBIOS structure table
//! (normally exposed by the firmware through `/sys/firmware/dmi/tables/DMI`)
//! and decodes the handful of structure types Talos cares about:
//!
//! * **Type 0** – BIOS information,
//! * **Type 1** – System information (manufacturer, product, UUID, serial),
//! * **Type 2** – Baseboard information,
//! * **Type 4** – Processor information (one per socket), and
//! * **Type 17** – Memory Device (one per DIMM slot).
//!
//! Each SMBIOS structure has a 4-byte header (`type`, `length`, 2-byte
//! `handle`), a fixed *formatted area* of `length` bytes, then an *unformatted
//! string area*: a sequence of NUL-terminated strings terminated by a double
//! NUL. Fields in the formatted area that hold text are 1-based indices into
//! that string area (index 0 means "not specified").
//!
//! The byte-table boundary is exactly the kind of OS interface Talos hides
//! behind an interface, so here it is modeled as the [`SmbiosSource`] trait
//! with an in-memory [`MemorySmbios`] used by the controller and tests.

use std::collections::BTreeMap;
use std::fmt;

/// SMBIOS structure type numbers (the subset Talos decodes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum StructureType {
    /// Type 0: BIOS Information.
    Bios = 0,
    /// Type 1: System Information.
    System = 1,
    /// Type 2: Baseboard (Module) Information.
    Baseboard = 2,
    /// Type 4: Processor Information.
    Processor = 4,
    /// Type 16: Physical Memory Array.
    MemoryArray = 16,
    /// Type 17: Memory Device.
    MemoryDevice = 17,
    /// Type 127: End-of-table marker.
    EndOfTable = 127,
    /// Any other structure type we don't specifically model.
    Other(u8),
}

impl StructureType {
    /// Decode a raw type byte.
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => StructureType::Bios,
            1 => StructureType::System,
            2 => StructureType::Baseboard,
            4 => StructureType::Processor,
            16 => StructureType::MemoryArray,
            17 => StructureType::MemoryDevice,
            127 => StructureType::EndOfTable,
            other => StructureType::Other(other),
        }
    }

    /// The raw type byte.
    pub fn as_u8(self) -> u8 {
        match self {
            StructureType::Bios => 0,
            StructureType::System => 1,
            StructureType::Baseboard => 2,
            StructureType::Processor => 4,
            StructureType::MemoryArray => 16,
            StructureType::MemoryDevice => 17,
            StructureType::EndOfTable => 127,
            StructureType::Other(v) => v,
        }
    }
}

/// Errors raised while decoding an SMBIOS table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmbiosError {
    /// The table ended in the middle of a structure header or formatted area.
    Truncated,
    /// A structure declared a length smaller than the 4-byte header.
    BadLength {
        /// The offending structure's type byte.
        ty: u8,
        /// The declared length.
        length: u8,
    },
    /// The string area was not terminated by a double NUL.
    UnterminatedStrings,
}

impl fmt::Display for SmbiosError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SmbiosError::Truncated => write!(f, "smbios table truncated"),
            SmbiosError::BadLength { ty, length } => {
                write!(f, "smbios structure type {ty} has bad length {length}")
            }
            SmbiosError::UnterminatedStrings => write!(f, "smbios string area unterminated"),
        }
    }
}

impl std::error::Error for SmbiosError {}

impl From<SmbiosError> for os_kernel::Error {
    fn from(e: SmbiosError) -> Self {
        os_kernel::Error::parse(e.to_string())
    }
}

/// One decoded SMBIOS structure: its header plus the raw formatted area and the
/// resolved string table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Structure {
    /// The structure type.
    pub ty: StructureType,
    /// The firmware-assigned handle (unique within the table).
    pub handle: u16,
    /// The formatted-area bytes (including the 4-byte header).
    pub formatted: Vec<u8>,
    /// The unformatted string area, resolved to a 1-based list.
    pub strings: Vec<String>,
}

impl Structure {
    /// The length byte of the formatted area (== `formatted.len()`).
    pub fn length(&self) -> usize {
        self.formatted.len()
    }

    /// Read a formatted-area byte by absolute offset (0 = type byte). Returns
    /// `None` if the offset is past the formatted area.
    pub fn byte(&self, offset: usize) -> Option<u8> {
        self.formatted.get(offset).copied()
    }

    /// Read a little-endian `u16` at `offset`.
    pub fn word(&self, offset: usize) -> Option<u16> {
        let lo = self.byte(offset)? as u16;
        let hi = self.byte(offset + 1)? as u16;
        Some(lo | (hi << 8))
    }

    /// Read a little-endian `u32` at `offset`.
    pub fn dword(&self, offset: usize) -> Option<u32> {
        let mut v: u32 = 0;
        for i in 0..4 {
            v |= (self.byte(offset + i)? as u32) << (8 * i as u32);
        }
        Some(v)
    }

    /// Resolve a string field. The byte at `offset` is a 1-based index into the
    /// string area; index 0 (or out of range) yields an empty string, matching
    /// the SMBIOS "not specified" convention.
    pub fn string(&self, offset: usize) -> String {
        match self.byte(offset) {
            None | Some(0) => String::new(),
            Some(idx) => self
                .strings
                .get((idx as usize) - 1)
                .cloned()
                .unwrap_or_default(),
        }
    }
}

/// A fully decoded SMBIOS table: an ordered list of structures.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SmbiosTable {
    /// The structures in table order.
    pub structures: Vec<Structure>,
}

impl SmbiosTable {
    /// Decode a raw SMBIOS structure-table blob.
    ///
    /// Parsing stops cleanly at the first `End-of-table` (type 127) structure
    /// or at the end of the buffer, whichever comes first.
    pub fn parse(raw: &[u8]) -> Result<Self, SmbiosError> {
        let mut structures = Vec::new();
        let mut pos = 0usize;

        while pos + 4 <= raw.len() {
            let ty = raw[pos];
            let length = raw[pos + 1];
            if (length as usize) < 4 {
                return Err(SmbiosError::BadLength { ty, length });
            }
            let handle = (raw[pos + 2] as u16) | ((raw[pos + 3] as u16) << 8);

            let formatted_end = pos + length as usize;
            if formatted_end > raw.len() {
                return Err(SmbiosError::Truncated);
            }
            let formatted = raw[pos..formatted_end].to_vec();

            // String area follows the formatted area, terminated by 00 00.
            let (strings, next) = decode_strings(raw, formatted_end)?;

            let parsed_ty = StructureType::from_u8(ty);
            structures.push(Structure {
                ty: parsed_ty,
                handle,
                formatted,
                strings,
            });

            pos = next;
            if parsed_ty == StructureType::EndOfTable {
                break;
            }
        }

        Ok(SmbiosTable { structures })
    }

    /// The first structure of a given type, if present.
    pub fn first(&self, ty: StructureType) -> Option<&Structure> {
        self.structures.iter().find(|s| s.ty == ty)
    }

    /// All structures of a given type, in table order.
    pub fn all(&self, ty: StructureType) -> Vec<&Structure> {
        self.structures.iter().filter(|s| s.ty == ty).collect()
    }

    /// Number of structures decoded.
    pub fn len(&self) -> usize {
        self.structures.len()
    }

    /// Whether the table is empty.
    pub fn is_empty(&self) -> bool {
        self.structures.is_empty()
    }
}

/// Decode the NUL-terminated string area starting at `start`. Returns the list
/// of strings (1-based when indexed by field values) and the offset just past
/// the terminating double NUL.
fn decode_strings(raw: &[u8], start: usize) -> Result<(Vec<String>, usize), SmbiosError> {
    // A structure with no strings is encoded as a single trailing 00 00.
    let mut strings = Vec::new();
    let mut i = start;

    if i >= raw.len() {
        return Err(SmbiosError::UnterminatedStrings);
    }

    // Special case: empty string set, encoded as 0x00 0x00.
    if raw[i] == 0 {
        // Need a second NUL.
        if i + 1 >= raw.len() {
            return Err(SmbiosError::UnterminatedStrings);
        }
        return Ok((strings, i + 2));
    }

    let mut cur = Vec::new();
    while i < raw.len() {
        let b = raw[i];
        i += 1;
        if b == 0 {
            // End of one string.
            strings.push(String::from_utf8_lossy(&cur).into_owned());
            cur.clear();
            // A following NUL terminates the whole area.
            if i < raw.len() && raw[i] == 0 {
                return Ok((strings, i + 1));
            }
        } else {
            cur.push(b);
        }
    }
    Err(SmbiosError::UnterminatedStrings)
}

/// OS boundary: a source of raw SMBIOS table bytes.
///
/// On a real node this reads `/sys/firmware/dmi/tables/DMI`. Tests and the
/// in-memory implementation supply a synthetic blob.
pub trait SmbiosSource {
    /// Return the raw SMBIOS structure-table bytes.
    fn read_table(&self) -> Result<Vec<u8>, SmbiosError>;
}

/// In-memory [`SmbiosSource`] backed by a byte vector.
#[derive(Debug, Clone, Default)]
pub struct MemorySmbios {
    raw: Vec<u8>,
}

impl MemorySmbios {
    /// Construct from raw table bytes.
    pub fn new(raw: Vec<u8>) -> Self {
        MemorySmbios { raw }
    }
}

impl SmbiosSource for MemorySmbios {
    fn read_table(&self) -> Result<Vec<u8>, SmbiosError> {
        Ok(self.raw.clone())
    }
}

/// A small builder that encodes SMBIOS structures into a raw blob.
///
/// Used by tests (and by anyone wanting a synthetic table) to round-trip
/// through [`SmbiosTable::parse`]. It enforces the wire layout: header,
/// formatted area, then the string area terminated by a double NUL.
#[derive(Debug, Clone, Default)]
pub struct SmbiosBuilder {
    out: Vec<u8>,
}

impl SmbiosBuilder {
    /// A fresh, empty builder.
    pub fn new() -> Self {
        SmbiosBuilder { out: Vec::new() }
    }

    /// Append a structure.
    ///
    /// `formatted` is the formatted area *excluding* the 4-byte header (the
    /// header is synthesized from `ty`, the computed length, and `handle`).
    /// `strings` becomes the 1-based string area.
    pub fn structure(
        mut self,
        ty: StructureType,
        handle: u16,
        formatted: &[u8],
        strings: &[&str],
    ) -> Self {
        let length = (formatted.len() + 4) as u8;
        self.out.push(ty.as_u8());
        self.out.push(length);
        self.out.push((handle & 0xff) as u8);
        self.out.push((handle >> 8) as u8);
        self.out.extend_from_slice(formatted);

        if strings.is_empty() {
            // Empty string area: 00 00.
            self.out.push(0);
            self.out.push(0);
        } else {
            for s in strings {
                self.out.extend_from_slice(s.as_bytes());
                self.out.push(0);
            }
            // Terminating NUL for the whole area.
            self.out.push(0);
        }
        self
    }

    /// Append an End-of-table (type 127) marker and finish, returning the raw
    /// blob.
    pub fn finish(self) -> Vec<u8> {
        self.structure(StructureType::EndOfTable, 0xffff, &[], &[])
            .out
    }

    /// Finish without an explicit End-of-table marker.
    pub fn finish_raw(self) -> Vec<u8> {
        self.out
    }
}

/// A normalized SMBIOS UUID (Type 1 offset 0x08, 16 bytes).
///
/// SMBIOS stores the first three fields of the UUID in little-endian byte order
/// (per the DMTF spec and the way `dmidecode`/Talos render it), so decoding has
/// to byte-swap them to produce the canonical text form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SmbiosUuid([u8; 16]);

impl SmbiosUuid {
    /// Construct from the 16 raw bytes as laid out in the SMBIOS structure.
    pub fn from_smbios_bytes(b: [u8; 16]) -> Self {
        SmbiosUuid(b)
    }

    /// The raw bytes.
    pub fn bytes(&self) -> [u8; 16] {
        self.0
    }

    /// Whether the UUID is all-zero or all-0xff, which firmware uses to mean
    /// "not set" and which Talos treats as absent.
    pub fn is_unset(&self) -> bool {
        self.0.iter().all(|&b| b == 0) || self.0.iter().all(|&b| b == 0xff)
    }

    /// Render the canonical `8-4-4-4-12` lowercase string, byte-swapping the
    /// first three little-endian fields exactly as `dmidecode` does.
    pub fn to_canonical(&self) -> String {
        let b = &self.0;
        // time_low (LE), time_mid (LE), time_hi_and_version (LE), then the rest
        // big-endian.
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            b[3],
            b[2],
            b[1],
            b[0],
            b[5],
            b[4],
            b[7],
            b[6],
            b[8],
            b[9],
            b[10],
            b[11],
            b[12],
            b[13],
            b[14],
            b[15],
        )
    }
}

impl fmt::Display for SmbiosUuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_canonical())
    }
}

/// Decoded Type 1 *System Information*.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SystemInfo {
    /// Manufacturer string.
    pub manufacturer: String,
    /// Product name.
    pub product_name: String,
    /// Product version.
    pub version: String,
    /// Serial number.
    pub serial_number: String,
    /// Canonical UUID, or empty if unset.
    pub uuid: String,
    /// SKU number (SMBIOS 2.4+).
    pub sku_number: String,
    /// Family (SMBIOS 2.4+).
    pub family: String,
}

impl SystemInfo {
    /// Decode from a Type 1 structure.
    pub fn decode(s: &Structure) -> Self {
        let mut info = SystemInfo {
            manufacturer: s.string(0x04),
            product_name: s.string(0x05),
            version: s.string(0x06),
            serial_number: s.string(0x07),
            ..Default::default()
        };
        // UUID at 0x08..0x18 (present when length >= 0x19).
        if s.length() >= 0x18 {
            let mut raw = [0u8; 16];
            for (i, slot) in raw.iter_mut().enumerate() {
                *slot = s.byte(0x08 + i).unwrap_or(0);
            }
            let uuid = SmbiosUuid::from_smbios_bytes(raw);
            if !uuid.is_unset() {
                info.uuid = uuid.to_canonical();
            }
        }
        // SKU (0x19) and Family (0x1a) added in SMBIOS 2.4.
        if s.length() > 0x19 {
            info.sku_number = s.string(0x19);
        }
        if s.length() > 0x1a {
            info.family = s.string(0x1a);
        }
        info
    }
}

/// Decoded Type 0 *BIOS Information* (the fields Talos surfaces).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BiosInfo {
    /// BIOS vendor.
    pub vendor: String,
    /// BIOS version string.
    pub version: String,
    /// Release date string.
    pub release_date: String,
}

impl BiosInfo {
    /// Decode from a Type 0 structure.
    pub fn decode(s: &Structure) -> Self {
        BiosInfo {
            vendor: s.string(0x04),
            version: s.string(0x05),
            release_date: s.string(0x08),
        }
    }
}

/// A convenience index over a parsed table keyed by handle.
pub fn index_by_handle(table: &SmbiosTable) -> BTreeMap<u16, &Structure> {
    table.structures.iter().map(|s| (s.handle, s)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal Type 1 system structure.
    fn system_blob() -> Vec<u8> {
        // Formatted area after the 4-byte header (offsets 0x04..):
        //   0x04 manufacturer idx = 1
        //   0x05 product idx       = 2
        //   0x06 version idx       = 3
        //   0x07 serial idx        = 4
        //   0x08.. UUID (16 bytes)
        //   0x18 wake-up type
        //   0x19 sku idx           = 5
        //   0x1a family idx        = 6
        let mut formatted = vec![1u8, 2, 3, 4];
        // SMBIOS-encoded UUID for canonical 00112233-4455-6677-8899-aabbccddeeff
        formatted.extend_from_slice(&[
            0x33, 0x22, 0x11, 0x00, // time_low LE
            0x55, 0x44, // time_mid LE
            0x77, 0x66, // time_hi LE
            0x88, 0x99, // clock seq (BE)
            0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // node (BE)
        ]);
        formatted.push(0x06); // wake-up type
        formatted.push(5); // sku idx
        formatted.push(6); // family idx

        SmbiosBuilder::new()
            .structure(
                StructureType::System,
                0x0001,
                &formatted,
                &["Acme", "MetalBox", "1.0", "SN12345", "SKU-9", "Server"],
            )
            .finish()
    }

    #[test]
    fn parse_roundtrips_system_structure() {
        let raw = system_blob();
        let table = SmbiosTable::parse(&raw).unwrap();
        // System + EndOfTable.
        assert_eq!(table.len(), 2);
        let sys = table.first(StructureType::System).unwrap();
        assert_eq!(sys.handle, 1);
        assert_eq!(sys.string(0x04), "Acme");
        assert_eq!(sys.string(0x05), "MetalBox");
    }

    #[test]
    fn system_info_decode_extracts_uuid_and_serial() {
        let raw = system_blob();
        let table = SmbiosTable::parse(&raw).unwrap();
        let info = SystemInfo::decode(table.first(StructureType::System).unwrap());
        assert_eq!(info.manufacturer, "Acme");
        assert_eq!(info.product_name, "MetalBox");
        assert_eq!(info.serial_number, "SN12345");
        assert_eq!(info.uuid, "00112233-4455-6677-8899-aabbccddeeff");
        assert_eq!(info.sku_number, "SKU-9");
        assert_eq!(info.family, "Server");
    }

    #[test]
    fn smbios_uuid_byte_swaps_first_three_fields() {
        let uuid = SmbiosUuid::from_smbios_bytes([
            0x33, 0x22, 0x11, 0x00, 0x55, 0x44, 0x77, 0x66, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
            0xee, 0xff,
        ]);
        assert_eq!(uuid.to_canonical(), "00112233-4455-6677-8899-aabbccddeeff");
        assert!(!uuid.is_unset());
    }

    #[test]
    fn unset_uuid_detected() {
        assert!(SmbiosUuid::from_smbios_bytes([0; 16]).is_unset());
        assert!(SmbiosUuid::from_smbios_bytes([0xff; 16]).is_unset());
        assert!(!SmbiosUuid::from_smbios_bytes([1; 16]).is_unset());
    }

    #[test]
    fn missing_string_index_zero_yields_empty() {
        let raw = SmbiosBuilder::new()
            .structure(StructureType::System, 1, &[0u8, 0, 0, 0], &[])
            .finish();
        let table = SmbiosTable::parse(&raw).unwrap();
        let sys = table.first(StructureType::System).unwrap();
        assert_eq!(sys.string(0x04), "");
    }

    #[test]
    fn bad_length_is_rejected() {
        // type=1, length=2 (< 4 header bytes).
        let raw = vec![1u8, 2, 0, 0, 0, 0];
        assert_eq!(
            SmbiosTable::parse(&raw),
            Err(SmbiosError::BadLength { ty: 1, length: 2 })
        );
    }

    #[test]
    fn truncated_formatted_area_is_rejected() {
        // declares length 8 but only 5 bytes present.
        let raw = vec![1u8, 8, 0, 0, 0];
        assert_eq!(SmbiosTable::parse(&raw), Err(SmbiosError::Truncated));
    }

    #[test]
    fn word_and_dword_little_endian() {
        let s = Structure {
            ty: StructureType::Processor,
            handle: 0,
            formatted: vec![4, 8, 0, 0, 0x34, 0x12, 0x78, 0x56, 0x34, 0x12],
            strings: vec![],
        };
        assert_eq!(s.word(0x04), Some(0x1234));
        assert_eq!(s.dword(0x06), Some(0x1234_5678));
    }

    #[test]
    fn memory_source_round_trips_blob() {
        let raw = system_blob();
        let src = MemorySmbios::new(raw.clone());
        assert_eq!(src.read_table().unwrap(), raw);
    }
}
