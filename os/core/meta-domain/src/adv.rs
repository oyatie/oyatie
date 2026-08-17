//! The ADV ("advertised") binary container.
//!
//! ADV is the on-disk serialization of the META key/value store. This module
//! models the **ADV1** layout used by modern Talos.
//!
//! Layout (all integers big-endian):
//!
//! ```text
//! +---------+---------+------------------ ... ------------------+
//! | magic   | count   | tag(1) len(2) value[len]  (repeated)    | crc32(4)
//! | 4 bytes | 2 bytes |                                         |
//! +---------+---------+-----------------------------------------+
//! ```
//!
//! - `magic`  — [`ADV1_MAGIC`], identifies the format/version.
//! - `count`  — number of tag/length/value records that follow.
//! - records  — each is a `u8` tag, a `u16` big-endian length, and `len` bytes.
//! - `crc32`  — IEEE CRC32 over `magic .. end-of-records` (everything before it).
//!
//! Decoding validates the magic, the record count, that lengths stay in
//! bounds, and the trailing checksum.

use crate::key::MetaKey;
use crate::value::MetaValue;
use os_kernel::{Error, Result};
use std::collections::BTreeMap;

/// Magic identifying the ADV1 format: ASCII `"ADV1"`.
pub const ADV1_MAGIC: [u8; 4] = *b"ADV1";

/// Magic identifying the legacy ADV format: ASCII `"ADV0"`.
///
/// The legacy layout predates ADV1 and constrains records far more tightly:
/// tags are a single byte and value lengths are encoded as a single byte too,
/// so no value may exceed 255 bytes. Talos still has to read partitions that
/// were written by old bootloaders in this format, so we model both.
pub const ADV_LEGACY_MAGIC: [u8; 4] = *b"ADV0";

/// Length of the fixed ADV1 header: 4-byte magic + 2-byte record count.
pub const ADV_HEADER_LEN: usize = 6;

/// Length of the trailing CRC32 checksum, in bytes.
pub const ADV_CRC_LEN: usize = 4;

/// The maximum value length representable in the legacy ADV format (`u8` len).
pub const ADV_LEGACY_MAX_VALUE_LEN: usize = u8::MAX as usize;

/// On-disk ADV container format/version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvFormat {
    /// The legacy single-byte-tag, single-byte-length layout (`ADV0`).
    Legacy,
    /// The modern `ADV1` layout: single-byte tag, `u16` length.
    V1,
}

impl AdvFormat {
    /// The 4-byte magic for this format.
    pub const fn magic(self) -> [u8; 4] {
        match self {
            AdvFormat::Legacy => ADV_LEGACY_MAGIC,
            AdvFormat::V1 => ADV1_MAGIC,
        }
    }

    /// Detects the format from a buffer's leading magic, if recognized.
    pub fn detect(buf: &[u8]) -> Option<AdvFormat> {
        if buf.len() < 4 {
            return None;
        }
        if buf[0..4] == ADV1_MAGIC {
            Some(AdvFormat::V1)
        } else if buf[0..4] == ADV_LEGACY_MAGIC {
            Some(AdvFormat::Legacy)
        } else {
            None
        }
    }

    /// The maximum value length a single record may hold in this format.
    pub const fn max_value_len(self) -> usize {
        match self {
            AdvFormat::Legacy => ADV_LEGACY_MAX_VALUE_LEN,
            AdvFormat::V1 => u16::MAX as usize,
        }
    }
}

/// The parsed ADV header (magic + record count).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvHeader {
    /// Format magic. One of [`ADV1_MAGIC`] or [`ADV_LEGACY_MAGIC`].
    pub magic: [u8; 4],
    /// Number of tag/length/value records in the body.
    pub count: u16,
}

impl AdvHeader {
    /// Builds a header for `count` records using the ADV1 magic.
    pub fn new(count: u16) -> Self {
        Self {
            magic: ADV1_MAGIC,
            count,
        }
    }

    /// Builds a header for `count` records in the given format.
    pub fn with_format(format: AdvFormat, count: u16) -> Self {
        Self {
            magic: format.magic(),
            count,
        }
    }

    /// The format implied by this header's magic, if recognized.
    pub fn format(&self) -> Result<AdvFormat> {
        AdvFormat::detect(&self.magic).ok_or_else(|| Error::parse("ADV header has invalid magic"))
    }

    /// Validates the magic, returning an error if it is neither ADV1 nor legacy.
    pub fn validate(&self) -> Result<()> {
        self.format().map(|_| ())
    }

    /// Parses the 6-byte header off the front of `buf`.
    pub fn parse(buf: &[u8]) -> Result<Self> {
        if buf.len() < ADV_HEADER_LEN {
            return Err(Error::parse("ADV buffer too short for header"));
        }
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&buf[0..4]);
        let count = u16::from_be_bytes([buf[4], buf[5]]);
        let header = Self { magic, count };
        header.validate()?;
        Ok(header)
    }

    /// Serializes the header into 6 bytes.
    pub fn encode(&self) -> [u8; ADV_HEADER_LEN] {
        let mut out = [0u8; ADV_HEADER_LEN];
        out[0..4].copy_from_slice(&self.magic);
        out[4..6].copy_from_slice(&self.count.to_be_bytes());
        out
    }
}

/// An in-memory ADV document: an ordered key/value store keyed by [`MetaKey`].
///
/// Records are stored in a [`BTreeMap`] so encoding is deterministic (ordered
/// by tag), which keeps the resulting blob — and its checksum — stable.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Adv {
    records: BTreeMap<MetaKey, MetaValue>,
}

impl Adv {
    /// Creates an empty ADV document.
    pub fn new() -> Self {
        Self {
            records: BTreeMap::new(),
        }
    }

    /// Reads the value stored under `key`, if any.
    pub fn get(&self, key: MetaKey) -> Option<&MetaValue> {
        self.records.get(&key)
    }

    /// Inserts or replaces the value for `key`, returning the previous value.
    pub fn set(&mut self, key: MetaKey, value: MetaValue) -> Option<MetaValue> {
        self.records.insert(key, value)
    }

    /// Removes a key, returning the value that was stored.
    pub fn delete(&mut self, key: MetaKey) -> Option<MetaValue> {
        self.records.remove(&key)
    }

    /// Number of records held.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Whether the document holds no records.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Iterates over `(key, value)` pairs in tag order.
    pub fn iter(&self) -> impl Iterator<Item = (&MetaKey, &MetaValue)> {
        self.records.iter()
    }

    /// Serializes the document into the ADV1 wire format with a trailing CRC32.
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.encode_as(AdvFormat::V1)
    }

    /// Serializes the document into the given ADV `format`, with a trailing
    /// CRC32 over everything before the checksum.
    ///
    /// The legacy format constrains both the value length (single byte) — a
    /// value longer than 255 bytes cannot be represented and is rejected.
    pub fn encode_as(&self, format: AdvFormat) -> Result<Vec<u8>> {
        if self.records.len() > u16::MAX as usize {
            return Err(Error::invalid("too many ADV records to encode"));
        }
        let header = AdvHeader::with_format(format, self.records.len() as u16);
        let mut out = Vec::new();
        out.extend_from_slice(&header.encode());
        for (key, value) in &self.records {
            if value.len() > format.max_value_len() {
                return Err(Error::invalid(format!(
                    "value for tag {:#04x} ({} bytes) exceeds the {:?} format limit of {} bytes",
                    key.tag(),
                    value.len(),
                    format,
                    format.max_value_len()
                )));
            }
            out.push(key.tag());
            match format {
                AdvFormat::Legacy => out.push(value.len() as u8),
                AdvFormat::V1 => out.extend_from_slice(&(value.len() as u16).to_be_bytes()),
            }
            out.extend_from_slice(value.as_bytes());
        }
        let crc = crc32(&out);
        out.extend_from_slice(&crc.to_be_bytes());
        Ok(out)
    }

    /// Parses an ADV blob, auto-detecting the format from its magic, validating
    /// header, bounds, and the trailing CRC32.
    ///
    /// The blob is *self-describing*: its length is implied by the record count
    /// and per-record lengths in the header, not by `buf.len()`. This lets an
    /// ADV document be decoded directly out of a larger, zero-padded region
    /// (such as a fixed-size META partition slot) — any bytes after the CRC are
    /// treated as padding and ignored.
    pub fn decode(buf: &[u8]) -> Result<Self> {
        let (adv, _format) = Self::decode_detect(buf)?;
        Ok(adv)
    }

    /// Like [`Adv::decode`] but also reports which on-disk [`AdvFormat`] the
    /// blob used. Useful when round-tripping: a partition read in legacy form
    /// can be re-encoded in legacy form rather than silently upgraded.
    pub fn decode_detect(buf: &[u8]) -> Result<(Self, AdvFormat)> {
        let header = AdvHeader::parse(buf)?;
        let format = header.format()?;

        // Walk the records first to discover where the body ends.
        let mut records = BTreeMap::new();
        let mut pos = ADV_HEADER_LEN;
        for _ in 0..header.count {
            // length field width depends on format: legacy=1, v1=2.
            let len_width = match format {
                AdvFormat::Legacy => 1,
                AdvFormat::V1 => 2,
            };
            if pos + 1 + len_width > buf.len() {
                return Err(Error::parse("ADV record header runs past end of buffer"));
            }
            let tag = buf[pos];
            let len = match format {
                AdvFormat::Legacy => buf[pos + 1] as usize,
                AdvFormat::V1 => u16::from_be_bytes([buf[pos + 1], buf[pos + 2]]) as usize,
            };
            pos += 1 + len_width;
            if pos + len > buf.len() {
                return Err(Error::parse("ADV record value runs past end of buffer"));
            }
            let key = MetaKey::from_tag(tag)?;
            let value = MetaValue::new(buf[pos..pos + len].to_vec())?;
            if records.insert(key, value).is_some() {
                return Err(Error::parse("ADV contains duplicate tag"));
            }
            pos += len;
        }

        // The CRC32 immediately follows the final record.
        let body_end = pos;
        if body_end + ADV_CRC_LEN > buf.len() {
            return Err(Error::parse("ADV buffer too short for checksum"));
        }
        let expected = u32::from_be_bytes([
            buf[body_end],
            buf[body_end + 1],
            buf[body_end + 2],
            buf[body_end + 3],
        ]);
        let actual = crc32(&buf[..body_end]);
        if expected != actual {
            return Err(Error::parse(format!(
                "ADV checksum mismatch: expected {expected:#010x}, computed {actual:#010x}"
            )));
        }

        Ok((Self { records }, format))
    }
}

/// IEEE 802.3 CRC32 (polynomial `0xEDB88320`), computed bit-by-bit so we need
/// no lookup-table allocation and stay fully `no_std`.
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Adv {
        let mut adv = Adv::new();
        adv.set(
            MetaKey::StagedUpgradeImageRef,
            MetaValue::from_str("ghcr.io/siderolabs/installer:v1.7.0").unwrap(),
        );
        adv.set(
            MetaKey::UniqueMachineToken,
            MetaValue::from_str("tok-abc-123").unwrap(),
        );
        adv
    }

    #[test]
    fn encode_decode_round_trip() {
        let adv = sample();
        let blob = adv.encode().unwrap();
        assert_eq!(&blob[0..4], &ADV1_MAGIC);
        let decoded = Adv::decode(&blob).unwrap();
        assert_eq!(decoded, adv);
        assert_eq!(
            decoded
                .get(MetaKey::UniqueMachineToken)
                .unwrap()
                .as_str()
                .unwrap(),
            "tok-abc-123"
        );
    }

    #[test]
    fn corrupted_checksum_is_rejected() {
        let mut blob = sample().encode().unwrap();
        let last = blob.len() - 1;
        blob[last] ^= 0xFF;
        let err = Adv::decode(&blob).unwrap_err();
        assert_eq!(err.kind(), "parse");
    }

    #[test]
    fn bad_magic_is_rejected() {
        let mut blob = sample().encode().unwrap();
        blob[0] = b'X';
        assert!(Adv::decode(&blob).is_err());
    }

    #[test]
    fn truncated_record_is_rejected() {
        let mut adv = Adv::new();
        adv.set(MetaKey::Upgrade, MetaValue::from_str("v1.6.0").unwrap());
        let mut blob = adv.encode().unwrap();
        // Claim there are 2 records though only 1 is present.
        blob[5] = 2;
        // Recompute checksum so we reach the record-walking logic, not CRC check.
        let body_end = blob.len() - ADV_CRC_LEN;
        let crc = crc32(&blob[..body_end]).to_be_bytes();
        blob[body_end..].copy_from_slice(&crc);
        assert!(Adv::decode(&blob).is_err());
    }

    #[test]
    fn crc32_known_vector() {
        // CRC32 of the ASCII string "123456789" is 0xCBF43926.
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn header_encodes_six_bytes() {
        let h = AdvHeader::new(3);
        let bytes = h.encode();
        assert_eq!(bytes.len(), ADV_HEADER_LEN);
        assert_eq!(AdvHeader::parse(&bytes).unwrap(), h);
    }

    #[test]
    fn format_detect_recognizes_both_magics() {
        let mut b = sample().encode().unwrap();
        assert_eq!(AdvFormat::detect(&b), Some(AdvFormat::V1));
        b[0..4].copy_from_slice(&ADV_LEGACY_MAGIC);
        assert_eq!(AdvFormat::detect(&b), Some(AdvFormat::Legacy));
        b[0] = b'Z';
        assert_eq!(AdvFormat::detect(&b), None);
        assert_eq!(AdvFormat::detect(b"AD"), None);
    }

    #[test]
    fn legacy_round_trip_and_format_reported() {
        let mut adv = Adv::new();
        adv.set(MetaKey::Upgrade, MetaValue::from_str("v1.6.0").unwrap());
        adv.set(
            MetaKey::UniqueMachineToken,
            MetaValue::from_str("tok-legacy").unwrap(),
        );
        let blob = adv.encode_as(AdvFormat::Legacy).unwrap();
        assert_eq!(&blob[0..4], &ADV_LEGACY_MAGIC);
        let (decoded, fmt) = Adv::decode_detect(&blob).unwrap();
        assert_eq!(fmt, AdvFormat::Legacy);
        assert_eq!(decoded, adv);
    }

    #[test]
    fn legacy_rejects_oversized_value() {
        let mut adv = Adv::new();
        adv.set(
            MetaKey::StateEncryptionConfig,
            MetaValue::new(vec![0u8; 256]).unwrap(),
        );
        // 256 bytes does not fit the legacy u8 length.
        assert!(adv.encode_as(AdvFormat::Legacy).is_err());
        // ...but is fine in ADV1.
        assert!(adv.encode_as(AdvFormat::V1).is_ok());
        // exactly 255 is the legacy limit.
        let mut adv2 = Adv::new();
        adv2.set(
            MetaKey::StateEncryptionConfig,
            MetaValue::new(vec![0u8; 255]).unwrap(),
        );
        assert!(adv2.encode_as(AdvFormat::Legacy).is_ok());
    }

    #[test]
    fn legacy_and_v1_produce_different_bytes_same_records() {
        let adv = sample();
        let v1 = adv.encode_as(AdvFormat::V1).unwrap();
        let legacy = adv.encode_as(AdvFormat::Legacy).unwrap();
        assert_ne!(v1, legacy);
        // legacy is more compact: 1-byte length vs 2-byte length per record.
        assert!(legacy.len() < v1.len());
        // both decode back to the same logical document.
        assert_eq!(Adv::decode(&v1).unwrap(), Adv::decode(&legacy).unwrap());
    }

    #[test]
    fn decode_default_reports_v1_via_detect() {
        let blob = sample().encode().unwrap();
        let (_, fmt) = Adv::decode_detect(&blob).unwrap();
        assert_eq!(fmt, AdvFormat::V1);
    }

    #[test]
    fn legacy_corrupted_checksum_rejected() {
        let mut blob = sample().encode_as(AdvFormat::Legacy).unwrap();
        let last = blob.len() - 1;
        blob[last] ^= 0xFF;
        assert!(Adv::decode(&blob).is_err());
    }
}
