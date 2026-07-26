//! A minimal LUKS2 header model and key-slot logic.
//!
//! Models the parts of a LUKS2 header machined reasons about: the magic, the
//! UUID, the cipher spec and the array of keyslots, each of which is either
//! empty or filled with key material that a passphrase must unlock. Real LUKS
//! uses Argon2/PBKDF2 KDFs; we model the "does this passphrase open this slot"
//! relation deterministically.

use crate::encryption::{Cipher, EncryptionConfig};
use crate::{BlockError, Result};

/// The LUKS2 primary header magic (`"LUKS\xba\xbe"`).
pub const LUKS_MAGIC: [u8; 6] = [b'L', b'U', b'K', b'S', 0xba, 0xbe];

/// LUKS2 version number stored in the header.
pub const LUKS2_VERSION: u16 = 2;

/// The maximum number of keyslots in a LUKS2 header.
pub const MAX_KEYSLOTS: u8 = 8;

/// The state of a single keyslot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LuksKeySlot {
    /// No key material; the slot is free.
    Empty,
    /// The slot holds a digest of the passphrase that opens it.
    ///
    /// `digest` is a deterministic stand-in for the Argon2-hashed anti-forensic
    /// key material a real header stores.
    Active { digest: u64 },
}

impl LuksKeySlot {
    /// Whether the slot currently holds key material.
    pub fn is_active(&self) -> bool {
        matches!(self, LuksKeySlot::Active { .. })
    }

    /// Whether `passphrase` opens this slot.
    pub fn unlocks(&self, passphrase: &[u8]) -> bool {
        match self {
            LuksKeySlot::Empty => false,
            LuksKeySlot::Active { digest } => *digest == digest_of(passphrase),
        }
    }
}

/// A deterministic FNV-1a digest used in place of a real KDF. Sufficient to
/// model "the right passphrase opens the slot, the wrong one does not".
fn digest_of(passphrase: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for &b in passphrase {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// A minimal LUKS2 header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Luks2Header {
    /// Header magic bytes.
    pub magic: [u8; 6],
    /// LUKS version.
    pub version: u16,
    /// Volume UUID.
    pub uuid: String,
    /// Cipher spec used for the bulk data.
    pub cipher: Cipher,
    /// The keyslot array.
    pub slots: Vec<LuksKeySlot>,
}

impl Luks2Header {
    /// Format a fresh header with all keyslots empty.
    pub fn format(uuid: impl Into<String>, cipher: Cipher) -> Self {
        Luks2Header {
            magic: LUKS_MAGIC,
            version: LUKS2_VERSION,
            uuid: uuid.into(),
            cipher,
            slots: vec![LuksKeySlot::Empty; MAX_KEYSLOTS as usize],
        }
    }

    /// Whether the header magic and version are valid.
    pub fn is_valid(&self) -> bool {
        self.magic == LUKS_MAGIC && self.version == LUKS2_VERSION
    }

    /// Add key material for `passphrase` into `slot`.
    pub fn add_key(&mut self, slot: u8, passphrase: &[u8]) -> Result<()> {
        let idx = Self::slot_index(slot)?;
        if self.slots[idx].is_active() {
            return Err(BlockError::KeyFailure(format!(
                "slot {slot} already active"
            )));
        }
        if passphrase.is_empty() {
            return Err(BlockError::KeyFailure("empty passphrase".to_string()));
        }
        self.slots[idx] = LuksKeySlot::Active {
            digest: digest_of(passphrase),
        };
        Ok(())
    }

    /// Erase the key material in `slot`, refusing to remove the last active key
    /// (which would render the volume permanently unopenable).
    pub fn remove_key(&mut self, slot: u8) -> Result<()> {
        let idx = Self::slot_index(slot)?;
        if !self.slots[idx].is_active() {
            return Err(BlockError::KeyFailure(format!("slot {slot} not active")));
        }
        if self.active_slots() == 1 {
            return Err(BlockError::KeyFailure(
                "refusing to remove last keyslot".to_string(),
            ));
        }
        self.slots[idx] = LuksKeySlot::Empty;
        Ok(())
    }

    /// Try to open the volume with `passphrase`, returning the index of the
    /// first keyslot it unlocks.
    pub fn open(&self, passphrase: &[u8]) -> Result<u8> {
        self.slots
            .iter()
            .position(|slot| slot.unlocks(passphrase))
            .and_then(|i| u8::try_from(i).ok())
            .ok_or_else(|| BlockError::KeyFailure("no keyslot matched".to_string()))
    }

    /// Number of keyslots currently holding key material.
    pub fn active_slots(&self) -> usize {
        self.slots.iter().filter(|s| s.is_active()).count()
    }

    /// Enroll every key in an [`EncryptionConfig`] into the header.
    pub fn enroll(&mut self, cfg: &EncryptionConfig) -> Result<()> {
        cfg.validate()?;
        for key in &cfg.keys {
            let passphrase = key.provider.derive()?;
            let idx = Self::slot_index(key.slot)?;
            if self.slots[idx].unlocks(&passphrase) {
                continue;
            }
            self.add_key(key.slot, &passphrase)?;
        }
        Ok(())
    }

    fn slot_index(slot: u8) -> Result<usize> {
        if slot >= MAX_KEYSLOTS {
            return Err(BlockError::KeyFailure(format!("slot {slot} out of range")));
        }
        Ok(slot as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encryption::{EncryptionKey, KeyProvider};

    #[test]
    fn fresh_header_is_valid_and_empty() {
        let h = Luks2Header::format("uuid-1", Cipher::AesXtsPlain64);
        assert!(h.is_valid());
        assert_eq!(h.active_slots(), 0);
        assert_eq!(h.slots.len(), MAX_KEYSLOTS as usize);
    }

    #[test]
    fn add_open_and_wrong_passphrase() {
        let mut h = Luks2Header::format("u", Cipher::AesXtsPlain64);
        h.add_key(0, b"secret").unwrap();
        assert_eq!(h.open(b"secret").unwrap(), 0);
        assert!(h.open(b"wrong").is_err());
        // Re-adding to an occupied slot fails.
        assert!(h.add_key(0, b"other").is_err());
    }

    #[test]
    fn cannot_remove_last_key() {
        let mut h = Luks2Header::format("u", Cipher::AesXtsPlain64);
        h.add_key(0, b"a").unwrap();
        assert!(h.remove_key(0).is_err());
        h.add_key(1, b"b").unwrap();
        h.remove_key(0).unwrap();
        assert_eq!(h.active_slots(), 1);
        assert!(h.open(b"b").is_ok());
        assert!(h.open(b"a").is_err());
    }

    #[test]
    fn slot_bounds_enforced() {
        let mut h = Luks2Header::format("u", Cipher::AesXtsPlain64);
        assert!(h.add_key(MAX_KEYSLOTS, b"x").is_err());
        assert!(h.add_key(0, b"").is_err());
    }

    #[test]
    fn enroll_from_config() {
        let mut cfg = EncryptionConfig::new();
        cfg.add_key(
            EncryptionKey::new(
                0,
                KeyProvider::Static {
                    passphrase: "p0".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        cfg.add_key(
            EncryptionKey::new(
                2,
                KeyProvider::NodeId {
                    node_uid: "node".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();

        let mut h = Luks2Header::format("u", cfg.cipher);
        h.enroll(&cfg).unwrap();
        assert_eq!(h.active_slots(), 2);
        assert_eq!(h.open(b"p0").unwrap(), 0);
        assert_eq!(h.open(b"nodeid:node").unwrap(), 2);
    }

    #[test]
    fn enroll_is_idempotent_for_existing_matching_slots() {
        let mut cfg = EncryptionConfig::new();
        cfg.add_key(
            EncryptionKey::new(
                3,
                KeyProvider::Static {
                    passphrase: "retryable".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();

        let mut h = Luks2Header::format("u", cfg.cipher);
        h.enroll(&cfg).unwrap();
        let enrolled = h.clone();
        h.enroll(&cfg).unwrap();

        assert_eq!(h, enrolled);
        assert_eq!(h.active_slots(), 1);
        assert_eq!(h.open(b"retryable").unwrap(), 3);
        assert!(h.open(b"wrong").is_err());

        let mut changed = EncryptionConfig::new();
        changed
            .add_key(
                EncryptionKey::new(
                    3,
                    KeyProvider::Static {
                        passphrase: "different".to_string(),
                    },
                )
                .unwrap(),
            )
            .unwrap();
        assert!(h.enroll(&changed).is_err());
        assert_eq!(h, enrolled);
    }
}
