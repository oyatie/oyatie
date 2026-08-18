//! Key-material types behind the one-way door.
//!
//! Three holders, one rule: key bytes live only in [`MlockedKey`] buffers and
//! leave this module only AEAD-sealed.
//!
//! - [`EnclaveRoot`] — the per-cell sealing root. Wraps/unwraps KEKs.
//! - [`KekMaterial`] — a tenant KEK at a specific version. Wraps/unwraps DEKs.
//! - [`DekMaterial`] — a data key. Seals/opens payload bytes for the data
//!   plane; the key itself is never readable.
//!
//! AEAD is AES-256-GCM via aws-lc-rs `RandomizedNonceKey` (ADR-0506 canonical
//! backend): the nonce is drawn inside AWS-LC per seal, making nonce reuse
//! impossible by construction. Token headers are bound as associated data, so
//! identifier or version tamper fails authentication.

use std::fmt;

use aws_lc_rs::aead::{AES_256_GCM, Aad, Nonce, RandomizedNonceKey};
use secrets_kms_domain::envelope_keys::{DekId, KekId};
use zeroize::{Zeroize, Zeroizing};

use crate::EnclaveError;
use crate::mlocked::{KEY_LEN, MlockedKey};
use crate::token::{NONCE_LEN, WrappedDek, WrappedKekToken, dek_header, kek_header};

/// 1-based KEK version. Version 1 is the initial key; rotation increments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KekVersion(u32);

impl KekVersion {
    /// The initial version every KEK starts at.
    pub const INITIAL: KekVersion = KekVersion(1);

    /// Construct from a raw value; zero is rejected.
    pub fn new(value: u32) -> Result<Self, EnclaveError> {
        if value == 0 {
            return Err(EnclaveError::ZeroVersion);
        }
        Ok(Self(value))
    }

    /// The next version, guarding against overflow.
    pub fn next(self) -> Result<Self, EnclaveError> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or(EnclaveError::VersionOverflow)
    }

    /// Raw value.
    pub fn value(self) -> u32 {
        self.0
    }
}

impl fmt::Display for KekVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}", self.0)
    }
}

/// Validated identifier of a per-cell sealing root.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SealingRootId(String);

impl SealingRootId {
    /// Construct; the id must be non-empty ASCII without control characters.
    pub fn new(value: impl Into<String>) -> Result<Self, EnclaveError> {
        let value = value.into();
        let valid =
            !value.is_empty() && value.len() <= 128 && value.bytes().all(|b| b.is_ascii_graphic());
        if !valid {
            return Err(EnclaveError::InvalidIdentifier(
                secrets_kms_domain::envelope_keys::EnvelopeKeyError::EmptySlug,
            ));
        }
        Ok(Self(value))
    }

    /// Raw value.
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SealingRootId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Per-cell sealing root: the top of the wrapping hierarchy inside one cell
/// (AWS KMS domain-key precedent). KEKs exist outside locked memory only
/// sealed under a root.
pub struct EnclaveRoot {
    root_id: SealingRootId,
    key: MlockedKey,
}

impl EnclaveRoot {
    /// Generate a fresh sealing root from the CSPRNG.
    pub fn generate(root_id: SealingRootId) -> Result<Self, EnclaveError> {
        Ok(Self {
            root_id,
            key: MlockedKey::generate()?,
        })
    }

    /// Ingress door for the unseal ceremony (OpenBao/PKCS#11 custody per
    /// ADR-0510): move externally reconstructed root bytes into locked
    /// memory. The source array is zeroized. There is no inverse.
    pub fn from_key_bytes(
        root_id: SealingRootId,
        bytes: [u8; KEY_LEN],
    ) -> Result<Self, EnclaveError> {
        Ok(Self {
            root_id,
            key: MlockedKey::from_bytes(bytes)?,
        })
    }

    /// Identifier of this root.
    pub fn root_id(&self) -> &SealingRootId {
        &self.root_id
    }

    /// Seal a KEK under this root. The resulting token is the only
    /// persistable representation of the KEK.
    pub fn wrap_kek(&self, kek: &KekMaterial) -> Result<WrappedKekToken, EnclaveError> {
        let header = kek_header(self.root_id.value(), &kek.kek_id, kek.version.value());
        let (nonce, ciphertext) = aead_seal(&self.key, &header, kek.key.expose())?;
        Ok(WrappedKekToken {
            root_id: self.root_id.value().to_owned(),
            kek_id: kek.kek_id.clone(),
            kek_version: kek.version.value(),
            nonce,
            ciphertext,
        })
    }

    /// Unseal a KEK token wrapped by this root.
    pub fn unwrap_kek(&self, token: &WrappedKekToken) -> Result<KekMaterial, EnclaveError> {
        if token.root_id != self.root_id.value() {
            return Err(EnclaveError::KeyBindingMismatch {
                expected: self.root_id.value().to_owned(),
                found: token.root_id.clone(),
            });
        }
        let plaintext = aead_open(&self.key, &token.aad(), &token.nonce, &token.ciphertext)?;
        let key = key_from_plaintext(plaintext)?;
        Ok(KekMaterial {
            kek_id: token.kek_id.clone(),
            version: KekVersion::new(token.kek_version)?,
            key,
        })
    }
}

impl fmt::Debug for EnclaveRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EnclaveRoot {{ root_id: {}, key: [REDACTED] }}",
            self.root_id
        )
    }
}

/// A tenant KEK at a specific version. Wraps and unwraps DEKs; persists only
/// as a [`WrappedKekToken`] sealed by an [`EnclaveRoot`].
pub struct KekMaterial {
    kek_id: KekId,
    version: KekVersion,
    key: MlockedKey,
}

impl KekMaterial {
    /// Generate fresh KEK material from the CSPRNG.
    pub fn generate(kek_id: KekId, version: KekVersion) -> Result<Self, EnclaveError> {
        Ok(Self {
            kek_id,
            version,
            key: MlockedKey::generate()?,
        })
    }

    /// Identifier of this KEK.
    pub fn kek_id(&self) -> &KekId {
        &self.kek_id
    }

    /// Version of this KEK.
    pub fn version(&self) -> KekVersion {
        self.version
    }

    /// Generate a fresh DEK and return both the usable material and its
    /// wrapped form (AWS KMS GenerateDataKey shape): the caller keeps the
    /// material for data-plane crypto and persists only the wrapped form.
    pub fn generate_dek(&self, dek_id: DekId) -> Result<(DekMaterial, WrappedDek), EnclaveError> {
        let dek_key = MlockedKey::generate()?;
        let header = dek_header(&self.kek_id, self.version.value(), &dek_id);
        let (nonce, ciphertext) = aead_seal(&self.key, &header, dek_key.expose())?;
        let wrapped = WrappedDek {
            kek_id: self.kek_id.clone(),
            kek_version: self.version.value(),
            dek_id: dek_id.clone(),
            nonce,
            ciphertext,
        };
        Ok((
            DekMaterial {
                dek_id,
                key: dek_key,
            },
            wrapped,
        ))
    }

    /// Unwrap a DEK previously wrapped by this exact KEK id + version.
    pub fn unwrap_dek(&self, wrapped: &WrappedDek) -> Result<DekMaterial, EnclaveError> {
        if wrapped.kek_id != self.kek_id {
            return Err(EnclaveError::KeyBindingMismatch {
                expected: self.kek_id.value().to_owned(),
                found: wrapped.kek_id.value().to_owned(),
            });
        }
        if wrapped.kek_version != self.version.value() {
            return Err(EnclaveError::UnknownKekVersion {
                version: wrapped.kek_version,
            });
        }
        let plaintext = aead_open(
            &self.key,
            &wrapped.aad(),
            &wrapped.nonce,
            &wrapped.ciphertext,
        )?;
        let key = key_from_plaintext(plaintext)?;
        Ok(DekMaterial {
            dek_id: wrapped.dek_id.clone(),
            key,
        })
    }
}

impl fmt::Debug for KekMaterial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "KekMaterial {{ kek_id: {}, version: {}, key: [REDACTED] }}",
            self.kek_id, self.version
        )
    }
}

/// A data-encryption key. Seals and opens payload bytes for the data plane;
/// the key bytes themselves are unreachable.
pub struct DekMaterial {
    dek_id: DekId,
    key: MlockedKey,
}

impl DekMaterial {
    /// Identifier of this DEK.
    pub fn dek_id(&self) -> &DekId {
        &self.dek_id
    }

    /// Encrypt a payload. Output layout: `nonce(12) || ciphertext+tag`.
    /// `aad` binds caller context (e.g. object id) into authentication.
    pub fn seal(&self, aad: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, EnclaveError> {
        let (nonce, ciphertext) = aead_seal(&self.key, aad, plaintext)?;
        let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    /// Decrypt a payload produced by [`DekMaterial::seal`]. The plaintext is
    /// returned in a zeroize-on-drop buffer.
    pub fn open(&self, aad: &[u8], blob: &[u8]) -> Result<Zeroizing<Vec<u8>>, EnclaveError> {
        if blob.len() < NONCE_LEN {
            return Err(EnclaveError::CryptoRejected);
        }
        let mut nonce = [0u8; NONCE_LEN];
        nonce.copy_from_slice(&blob[..NONCE_LEN]);
        aead_open(&self.key, aad, &nonce, &blob[NONCE_LEN..])
    }
}

impl fmt::Debug for DekMaterial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DekMaterial {{ dek_id: {}, key: [REDACTED] }}",
            self.dek_id
        )
    }
}

/// Seal `plaintext` under `key` with a backend-drawn random nonce.
fn aead_seal(
    key: &MlockedKey,
    aad: &[u8],
    plaintext: &[u8],
) -> Result<([u8; NONCE_LEN], Vec<u8>), EnclaveError> {
    let sealing = RandomizedNonceKey::new(&AES_256_GCM, key.expose())
        .map_err(|_| EnclaveError::CryptoRejected)?;
    let mut in_out = plaintext.to_vec();
    let nonce = sealing
        .seal_in_place_append_tag(Aad::from(aad), &mut in_out)
        .map_err(|_| EnclaveError::CryptoRejected)?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    nonce_bytes.copy_from_slice(nonce.as_ref());
    Ok((nonce_bytes, in_out))
}

/// Open `ciphertext` under `key`; plaintext lands in a zeroizing buffer and
/// the working copy is scrubbed.
fn aead_open(
    key: &MlockedKey,
    aad: &[u8],
    nonce: &[u8; NONCE_LEN],
    ciphertext: &[u8],
) -> Result<Zeroizing<Vec<u8>>, EnclaveError> {
    let opening = RandomizedNonceKey::new(&AES_256_GCM, key.expose())
        .map_err(|_| EnclaveError::CryptoRejected)?;
    let nonce =
        Nonce::try_assume_unique_for_key(nonce).map_err(|_| EnclaveError::CryptoRejected)?;
    let mut in_out = ciphertext.to_vec();
    let plaintext_len = match opening.open_in_place(nonce, Aad::from(aad), &mut in_out) {
        Ok(plaintext) => plaintext.len(),
        Err(_) => {
            in_out.zeroize();
            return Err(EnclaveError::CryptoRejected);
        }
    };
    let plaintext = Zeroizing::new(in_out[..plaintext_len].to_vec());
    in_out.zeroize();
    Ok(plaintext)
}

/// Move a 32-byte AEAD-opened key into locked memory, scrubbing the
/// intermediate buffer regardless of outcome.
fn key_from_plaintext(plaintext: Zeroizing<Vec<u8>>) -> Result<MlockedKey, EnclaveError> {
    if plaintext.len() != KEY_LEN {
        return Err(EnclaveError::CryptoRejected);
    }
    let mut bytes = [0u8; KEY_LEN];
    bytes.copy_from_slice(&plaintext);
    // `plaintext` zeroizes on drop; `from_bytes` zeroizes `bytes`.
    MlockedKey::from_bytes(bytes)
}
