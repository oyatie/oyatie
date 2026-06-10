//! LUKS encryption configuration and key providers.
//!
//! Mirrors Talos's volume encryption config (`block.EncryptionSpec`): a cipher,
//! one or more key providers (static secret, node UID, TPM, KMS) and the rules
//! machined uses to derive the passphrase that unlocks a LUKS keyslot.

use crate::{BlockError, Result};

/// The block cipher / mode used for the LUKS volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cipher {
    /// AES in XTS mode with a 512-bit key (the Talos default).
    AesXtsPlain64,
    /// AES in CBC mode (legacy).
    AesCbcEssiv,
}

impl Cipher {
    /// The `cryptsetup`-style cipher spec string.
    pub fn as_str(self) -> &'static str {
        match self {
            Cipher::AesXtsPlain64 => "aes-xts-plain64",
            Cipher::AesCbcEssiv => "aes-cbc-essiv:sha256",
        }
    }

    /// Key size in bits.
    pub fn key_bits(self) -> u32 {
        match self {
            Cipher::AesXtsPlain64 => 512,
            Cipher::AesCbcEssiv => 256,
        }
    }
}

/// Where the passphrase material for a keyslot comes from.
///
/// Mirrors the `EncryptionKey` provider kinds in Talos config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyProvider {
    /// A static passphrase carried in machine config.
    Static { passphrase: String },
    /// Derived from the node's hardware UID (`nodeID`).
    NodeId { node_uid: String },
    /// Sealed against a TPM 2.0 device; here modelled by an opaque blob.
    Tpm { sealed_blob: Vec<u8> },
    /// Fetched from an external KMS endpoint identified by URL.
    Kms { endpoint: String, token: String },
}

impl KeyProvider {
    /// A stable identifier string for the provider kind.
    pub fn kind(&self) -> &'static str {
        match self {
            KeyProvider::Static { .. } => "static",
            KeyProvider::NodeId { .. } => "nodeID",
            KeyProvider::Tpm { .. } => "tpm",
            KeyProvider::Kms { .. } => "kms",
        }
    }

    /// Derive the raw passphrase bytes this provider contributes.
    ///
    /// Real Talos seals/unseals against hardware; the in-memory model derives a
    /// deterministic passphrase so keyslot logic can be exercised in tests.
    pub fn derive(&self) -> Result<Vec<u8>> {
        match self {
            KeyProvider::Static { passphrase } => {
                if passphrase.is_empty() {
                    return Err(BlockError::KeyFailure("empty passphrase".to_string()));
                }
                Ok(passphrase.as_bytes().to_vec())
            }
            KeyProvider::NodeId { node_uid } => {
                if node_uid.is_empty() {
                    return Err(BlockError::KeyFailure("empty node UID".to_string()));
                }
                Ok(format!("nodeid:{node_uid}").into_bytes())
            }
            KeyProvider::Tpm { sealed_blob } => {
                if sealed_blob.is_empty() {
                    return Err(BlockError::KeyFailure("empty TPM blob".to_string()));
                }
                Ok(sealed_blob.clone())
            }
            KeyProvider::Kms { endpoint, token } => {
                if endpoint.is_empty() || token.is_empty() {
                    return Err(BlockError::KeyFailure(
                        "KMS endpoint and token required".to_string(),
                    ));
                }
                Ok(format!("kms:{endpoint}:{token}").into_bytes())
            }
        }
    }
}

/// A key bound to a specific LUKS keyslot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptionKey {
    /// The keyslot index (0..=7 for LUKS2).
    pub slot: u8,
    /// The provider supplying the passphrase.
    pub provider: KeyProvider,
}

impl EncryptionKey {
    /// Construct a key, validating the slot index against the LUKS2 maximum.
    pub fn new(slot: u8, provider: KeyProvider) -> Result<Self> {
        if slot >= crate::luks::MAX_KEYSLOTS {
            return Err(BlockError::KeyFailure(format!(
                "slot {slot} out of range (max {})",
                crate::luks::MAX_KEYSLOTS - 1
            )));
        }
        Ok(EncryptionKey { slot, provider })
    }
}

/// Full encryption configuration for a volume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptionConfig {
    /// The cipher / mode.
    pub cipher: Cipher,
    /// The keys, one per occupied keyslot.
    pub keys: Vec<EncryptionKey>,
}

impl EncryptionConfig {
    /// Create a config with the Talos default cipher and no keys.
    pub fn new() -> Self {
        EncryptionConfig {
            cipher: Cipher::AesXtsPlain64,
            keys: Vec::new(),
        }
    }

    /// Add a key, rejecting duplicate slot assignments.
    pub fn add_key(&mut self, key: EncryptionKey) -> Result<()> {
        if self.keys.iter().any(|k| k.slot == key.slot) {
            return Err(BlockError::KeyFailure(format!(
                "slot {} already in use",
                key.slot
            )));
        }
        self.keys.push(key);
        Ok(())
    }

    /// Validate that the config has at least one key and no slot collisions.
    pub fn validate(&self) -> Result<()> {
        if self.keys.is_empty() {
            return Err(BlockError::KeyFailure(
                "encryption requires at least one key".to_string(),
            ));
        }
        for (i, a) in self.keys.iter().enumerate() {
            for b in &self.keys[i + 1..] {
                if a.slot == b.slot {
                    return Err(BlockError::KeyFailure("duplicate keyslot".to_string()));
                }
            }
        }
        Ok(())
    }

    /// Try to derive a passphrase from any configured key whose provider
    /// resolves, returning the lowest-numbered slot + bytes that succeed.
    /// Models Talos's "try every keyslot in slot order" behaviour.
    pub fn derive_any(&self) -> Result<(u8, Vec<u8>)> {
        let mut keys: Vec<_> = self.keys.iter().collect();
        keys.sort_by_key(|key| key.slot);
        for key in keys {
            if let Ok(bytes) = key.provider.derive() {
                return Ok((key.slot, bytes));
            }
        }
        Err(BlockError::KeyFailure(
            "no key provider resolved".to_string(),
        ))
    }
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_derivation() {
        let p = KeyProvider::Static {
            passphrase: "hunter2".to_string(),
        };
        assert_eq!(p.kind(), "static");
        assert_eq!(p.derive().unwrap(), b"hunter2");

        let empty = KeyProvider::Static {
            passphrase: String::new(),
        };
        assert!(empty.derive().is_err());

        let node = KeyProvider::NodeId {
            node_uid: "abc".to_string(),
        };
        assert_eq!(node.derive().unwrap(), b"nodeid:abc".to_vec());
    }

    #[test]
    fn slot_range_enforced() {
        let prov = KeyProvider::NodeId {
            node_uid: "x".to_string(),
        };
        assert!(EncryptionKey::new(0, prov.clone()).is_ok());
        assert!(EncryptionKey::new(99, prov).is_err());
    }

    #[test]
    fn config_rejects_duplicate_slots_and_empty() {
        let mut cfg = EncryptionConfig::new();
        assert_eq!(cfg.cipher, Cipher::AesXtsPlain64);
        assert!(cfg.validate().is_err());

        let prov = KeyProvider::Static {
            passphrase: "p".to_string(),
        };
        cfg.add_key(EncryptionKey::new(0, prov.clone()).unwrap())
            .unwrap();
        assert!(
            cfg.add_key(EncryptionKey::new(0, prov.clone()).unwrap())
                .is_err()
        );
        cfg.add_key(EncryptionKey::new(1, prov).unwrap()).unwrap();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn derive_any_picks_lowest_resolvable_slot() {
        let mut cfg = EncryptionConfig::new();
        cfg.add_key(
            EncryptionKey::new(
                7,
                KeyProvider::Static {
                    passphrase: "late".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        cfg.add_key(
            EncryptionKey::new(
                0,
                KeyProvider::Tpm {
                    sealed_blob: Vec::new(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        cfg.add_key(
            EncryptionKey::new(
                1,
                KeyProvider::Static {
                    passphrase: "good".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        let (slot, bytes) = cfg.derive_any().unwrap();
        assert_eq!(slot, 1);
        assert_eq!(bytes, b"good");
    }

    #[test]
    fn cipher_metadata() {
        assert_eq!(Cipher::AesXtsPlain64.key_bits(), 512);
        assert_eq!(Cipher::AesCbcEssiv.as_str(), "aes-cbc-essiv:sha256");
    }
}
