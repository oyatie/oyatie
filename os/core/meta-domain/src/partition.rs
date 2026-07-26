//! The on-disk META partition layout.
//!
//! In Talos the META partition is a small, fixed-size partition labelled `META`
//! (constant `constants.MetaPartitionLabel`). It stores **two** ADV documents
//! back-to-back: a primary copy and a backup copy, each occupying half of the
//! partition. Writes update the backup first, then the primary, so a crash mid
//! write can always fall back to a consistent copy.
//!
//! This module models that layout: it validates partition sizing, splits the
//! raw partition bytes into the two ADV slots, and selects the most recent
//! valid copy on load.

use crate::adv::{Adv, AdvFormat};
use os_kernel::{Error, Result};

/// The GPT partition label Talos uses for the META partition.
pub const META_PARTITION_LABEL: &str = "META";

/// The fixed size of the META partition, in bytes (1 MiB in Talos).
pub const META_PARTITION_SIZE: usize = 1024 * 1024;

/// Number of ADV copies stored within the partition (primary + backup).
pub const ADV_COPIES: usize = 2;

/// Which of the two ADV slots a document occupies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    /// The primary copy, in the first half of the partition.
    Primary,
    /// The backup copy, in the second half of the partition.
    Backup,
}

/// Models the META partition: a fixed-size byte region split into two ADV slots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaPartition {
    size: usize,
}

impl Default for MetaPartition {
    fn default() -> Self {
        Self {
            size: META_PARTITION_SIZE,
        }
    }
}

impl MetaPartition {
    /// Constructs a partition descriptor of the given byte size.
    ///
    /// The size must be even (so it splits cleanly into two slots) and large
    /// enough to hold a minimal ADV document in each slot.
    pub fn new(size: usize) -> Result<Self> {
        if !size.is_multiple_of(ADV_COPIES) {
            return Err(Error::invalid("META partition size must be even"));
        }
        if size / ADV_COPIES < crate::adv::ADV_HEADER_LEN + crate::adv::ADV_CRC_LEN {
            return Err(Error::invalid(
                "META partition slot too small for an ADV header",
            ));
        }
        Ok(Self { size })
    }

    /// The total partition size in bytes.
    pub fn size(&self) -> usize {
        self.size
    }

    /// The size of a single ADV slot (half the partition).
    pub fn slot_size(&self) -> usize {
        self.size / ADV_COPIES
    }

    /// The byte offset of the given slot within the partition.
    pub fn slot_offset(&self, slot: Slot) -> usize {
        match slot {
            Slot::Primary => 0,
            Slot::Backup => self.slot_size(),
        }
    }

    /// Serializes `adv` and writes it into both slots of a fresh partition
    /// image, zero-padding each slot to [`Self::slot_size`].
    ///
    /// Returns the full partition image, exactly [`Self::size`] bytes long.
    pub fn write_image(&self, adv: &Adv) -> Result<Vec<u8>> {
        self.write_image_as(adv, AdvFormat::V1)
    }

    /// Like [`Self::write_image`] but encodes the ADV in the given `format`,
    /// allowing a legacy-format META partition to be (re)written faithfully.
    pub fn write_image_as(&self, adv: &Adv, format: AdvFormat) -> Result<Vec<u8>> {
        let encoded = adv.encode_as(format)?;
        if encoded.len() > self.slot_size() {
            return Err(Error::invalid(
                "ADV document does not fit in a partition slot",
            ));
        }
        let mut image = vec![0u8; self.size];
        for slot in [Slot::Primary, Slot::Backup] {
            let off = self.slot_offset(slot);
            image[off..off + encoded.len()].copy_from_slice(&encoded);
        }
        Ok(image)
    }

    /// Reads a partition image, returning the first slot that decodes cleanly.
    ///
    /// Tries the primary slot first, then the backup. Errors only if *neither*
    /// slot contains a valid ADV document.
    pub fn read_image(&self, image: &[u8]) -> Result<(Slot, Adv)> {
        if image.len() != self.size {
            return Err(Error::invalid(
                "partition image size does not match descriptor",
            ));
        }
        let mut last_err = None;
        for slot in [Slot::Primary, Slot::Backup] {
            let off = self.slot_offset(slot);
            let slice = &image[off..off + self.slot_size()];
            match Adv::decode(slice) {
                Ok(adv) => return Ok((slot, adv)),
                Err(e) => last_err = Some(e),
            }
        }
        Err(last_err
            .unwrap_or_else(|| Error::not_found("no valid ADV copy found in META partition")))
    }

    /// Like [`Self::read_image`] but also reports which on-disk [`AdvFormat`]
    /// the selected slot used, so callers can re-write in the same format.
    pub fn read_image_detect(&self, image: &[u8]) -> Result<(Slot, Adv, AdvFormat)> {
        if image.len() != self.size {
            return Err(Error::invalid(
                "partition image size does not match descriptor",
            ));
        }
        let mut last_err = None;
        for slot in [Slot::Primary, Slot::Backup] {
            let off = self.slot_offset(slot);
            let slice = &image[off..off + self.slot_size()];
            match Adv::decode_detect(slice) {
                Ok((adv, fmt)) => return Ok((slot, adv, fmt)),
                Err(e) => last_err = Some(e),
            }
        }
        Err(last_err
            .unwrap_or_else(|| Error::not_found("no valid ADV copy found in META partition")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key::MetaKey;
    use crate::value::MetaValue;

    fn sample_adv() -> Adv {
        let mut adv = Adv::new();
        adv.set(
            MetaKey::UniqueMachineToken,
            MetaValue::from_str("tok-123").unwrap(),
        );
        adv
    }

    #[test]
    fn default_partition_is_one_mib_split_in_two() {
        let p = MetaPartition::default();
        assert_eq!(p.size(), META_PARTITION_SIZE);
        assert_eq!(p.slot_size(), META_PARTITION_SIZE / 2);
        assert_eq!(p.slot_offset(Slot::Primary), 0);
        assert_eq!(p.slot_offset(Slot::Backup), META_PARTITION_SIZE / 2);
    }

    #[test]
    fn rejects_odd_and_tiny_sizes() {
        assert!(MetaPartition::new(1025).is_err());
        assert!(MetaPartition::new(2).is_err());
        assert!(MetaPartition::new(4096).is_ok());
    }

    #[test]
    fn write_then_read_round_trips_via_primary() {
        let p = MetaPartition::new(4096).unwrap();
        let adv = sample_adv();
        let image = p.write_image(&adv).unwrap();
        assert_eq!(image.len(), 4096);
        let (slot, decoded) = p.read_image(&image).unwrap();
        assert_eq!(slot, Slot::Primary);
        assert_eq!(decoded, adv);
    }

    #[test]
    fn falls_back_to_backup_when_primary_corrupt() {
        let p = MetaPartition::new(4096).unwrap();
        let mut image = p.write_image(&sample_adv()).unwrap();
        // Corrupt the primary magic so primary decode fails.
        image[0] = b'X';
        let (slot, decoded) = p.read_image(&image).unwrap();
        assert_eq!(slot, Slot::Backup);
        assert_eq!(
            decoded
                .get(MetaKey::UniqueMachineToken)
                .unwrap()
                .as_str()
                .unwrap(),
            "tok-123"
        );
    }

    #[test]
    fn both_copies_corrupt_errors() {
        let p = MetaPartition::new(4096).unwrap();
        let mut image = p.write_image(&sample_adv()).unwrap();
        image[0] = b'X';
        image[p.slot_offset(Slot::Backup)] = b'X';
        assert!(p.read_image(&image).is_err());
    }

    #[test]
    fn wrong_image_size_errors() {
        let p = MetaPartition::new(4096).unwrap();
        assert!(p.read_image(&[0u8; 100]).is_err());
    }

    #[test]
    fn legacy_image_round_trips_and_reports_format() {
        let p = MetaPartition::new(4096).unwrap();
        let adv = sample_adv();
        let image = p.write_image_as(&adv, AdvFormat::Legacy).unwrap();
        let (slot, decoded, fmt) = p.read_image_detect(&image).unwrap();
        assert_eq!(slot, Slot::Primary);
        assert_eq!(fmt, AdvFormat::Legacy);
        assert_eq!(decoded, adv);
    }

    #[test]
    fn detect_reports_v1_for_default_write() {
        let p = MetaPartition::new(4096).unwrap();
        let image = p.write_image(&sample_adv()).unwrap();
        let (_, _, fmt) = p.read_image_detect(&image).unwrap();
        assert_eq!(fmt, AdvFormat::V1);
    }

    #[test]
    fn legacy_fallback_to_backup() {
        let p = MetaPartition::new(4096).unwrap();
        let mut image = p.write_image_as(&sample_adv(), AdvFormat::Legacy).unwrap();
        image[0] = b'X';
        let (slot, _, fmt) = p.read_image_detect(&image).unwrap();
        assert_eq!(slot, Slot::Backup);
        assert_eq!(fmt, AdvFormat::Legacy);
    }

    #[test]
    fn oversized_adv_does_not_fit_slot() {
        let p = MetaPartition::new(64).unwrap();
        let mut adv = Adv::new();
        adv.set(
            MetaKey::StateEncryptionConfig,
            MetaValue::new(vec![0u8; 200]).unwrap(),
        );
        assert!(p.write_image(&adv).is_err());
    }
}
