// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

//! Crypto-enclave kernel for kms (story G002, ADR-0536 D-8).
//!
//! This crate is the in-process core of the kms crypto enclave: the
//! type-system one-way door for key material, per the AWS KMS domain model
//! (keys live inside the crypto boundary and key material never leaves it).
//! The separate enclave *process* boundary ships as the `[[bin]]` wrapper in a
//! later G002 sub-slice; every key-material invariant it must uphold is
//! enforced here, in types, so the binary cannot weaken them.
//!
//! Invariants carried by construction:
//!
//! - **One-way door** — [`KekMaterial`], [`DekMaterial`] and [`EnclaveRoot`]
//!   hold their 256-bit keys in [`mlocked::MlockedKey`] buffers (page-locked
//!   via `mlock`, zeroized then `munlock`ed on drop). None of these types
//!   implement `Clone`, `Copy`, serde traits, or expose key bytes; the only
//!   public egress is *wrapped* (AEAD-sealed) form. Ingress doors exist
//!   (generation from the CSPRNG, [`EnclaveRoot::from_key_bytes`] for the
//!   unseal ceremony) — egress doors do not.
//! - **KEKs persist only as wrapped tokens** — [`WrappedKekToken`] is the only
//!   serializable representation of a KEK, sealed under a per-cell
//!   [`EnclaveRoot`] with the token header bound as AEAD associated data.
//! - **Decrypt-only rotation** — [`KekVersionChain`] models AWS KMS version
//!   rotation: a new key version encrypts forward, retired versions are
//!   demoted to [`DecryptOnlyKek`] (no wrap API exists on the type), and
//!   existing ciphertext is never re-encrypted.
//! - **Static stability** — [`BoundedTtlDekCache`] keeps the data plane
//!   serving decrypts for a bounded TTL while the KMS control plane is down,
//!   and fails closed (never serves an expired DEK) once the bound is reached.
//!
//! Precedent (ADR-0536 D-8): AWS KMS domains/version-rotation/envelope
//! encryption; GCP Cloud KMS/Keystore; Azure Managed HSM. Rejected
//! anti-patterns: re-encrypt-on-rotate, per-request KMS calls on the data
//! path, shared cross-tenant keys.

pub mod chain;
pub mod dek_cache;
pub mod material;
pub mod mlocked;
pub mod provenance;
pub mod shred;
pub mod token;

pub use chain::{DecryptOnlyKek, KekVersionChain};
pub use dek_cache::{
    BoundedTtlDekCache, ClockSource, ControlPlaneUnavailable, DekCacheError, DekCacheKey,
    FetchSource, SystemClockSource,
};
pub use material::{DekMaterial, EnclaveRoot, KekMaterial, KekVersion, SealingRootId};
pub use provenance::RootProvenance;
pub use shred::{
    CancelEvidence, MIN_WAITING_WINDOW_SECONDS, PendingDeletionChain, QuorumPolicy,
    ScheduledKeyDeletion, ShredAction, ShredAuthorizationPort, ShredAuthorizationRequest,
    ShredDecision, ShredDecisionEvidence, ShredError, ShredProof,
};
pub use token::{TokenError, WrappedDek, WrappedKekToken};

// Re-exported so enclave callers name envelope identifiers without a direct
// dependency on the domain crate.
pub use secrets_kms_domain::envelope_keys::{DekId, EnvelopeKeyError, KekId};

use std::fmt;

/// Errors surfaced by the enclave kernel.
///
/// AEAD failures are deliberately collapsed into [`EnclaveError::CryptoRejected`]
/// without distinguishing tamper, wrong-key, or AAD mismatch — distinguishing
/// them would hand an oracle to a caller probing the crypto boundary.
#[derive(Debug)]
pub enum EnclaveError {
    /// `mlock(2)` refused to pin the key page; key material must never touch
    /// swap, so construction fails closed instead of degrading silently.
    MemoryLockFailed {
        /// `errno` reported by the platform.
        errno: i32,
    },
    /// The CSPRNG failed to produce key material.
    RandomSourceFailed,
    /// AEAD seal/open was rejected (tamper, wrong key, or AAD mismatch).
    CryptoRejected,
    /// A wrapped token failed strict decoding.
    TokenMalformed(TokenError),
    /// The token names a different KEK or sealing root than the unwrapping key.
    KeyBindingMismatch {
        /// Identifier the unwrapping side holds.
        expected: String,
        /// Identifier the token carries.
        found: String,
    },
    /// The wrapped DEK names a KEK version this chain does not hold.
    UnknownKekVersion {
        /// Version carried by the wrapped DEK.
        version: u32,
    },
    /// KEK version arithmetic would overflow `u32`.
    VersionOverflow,
    /// A KEK version of zero is invalid (versions are 1-based).
    ZeroVersion,
    /// An envelope identifier failed domain validation.
    InvalidIdentifier(EnvelopeKeyError),
}

impl fmt::Display for EnclaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MemoryLockFailed { errno } => {
                write!(
                    f,
                    "enclave: mlock failed (errno {errno}); refusing unpinned key material"
                )
            }
            Self::RandomSourceFailed => f.write_str("enclave: CSPRNG failure"),
            Self::CryptoRejected => f.write_str("enclave: AEAD rejected the operation"),
            Self::TokenMalformed(err) => write!(f, "enclave: malformed wrapped token: {err}"),
            Self::KeyBindingMismatch { expected, found } => {
                write!(
                    f,
                    "enclave: token bound to '{found}', unwrapping key is '{expected}'"
                )
            }
            Self::UnknownKekVersion { version } => {
                write!(f, "enclave: no KEK material held for version {version}")
            }
            Self::VersionOverflow => f.write_str("enclave: KEK version overflow"),
            Self::ZeroVersion => f.write_str("enclave: KEK versions are 1-based; zero is invalid"),
            Self::InvalidIdentifier(err) => {
                write!(f, "enclave: invalid envelope identifier: {err}")
            }
        }
    }
}

impl std::error::Error for EnclaveError {}

impl From<TokenError> for EnclaveError {
    fn from(err: TokenError) -> Self {
        Self::TokenMalformed(err)
    }
}

impl From<EnvelopeKeyError> for EnclaveError {
    fn from(err: EnvelopeKeyError) -> Self {
        Self::InvalidIdentifier(err)
    }
}
