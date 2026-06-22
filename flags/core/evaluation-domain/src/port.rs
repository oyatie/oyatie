//! Ports for the flag-evaluation domain (clean-arch ports-in-core, ADR-0570).
//!
//! These traits are the CUTOVER-STABLE seam between the pure evaluation domain and the outside
//! world. They are DEFINED here in `core/` and IMPLEMENTED later by DEFERRED adapters:
//!
//! - a persistence adapter (`flags/adapters/storage/*`) backed by the owned data substrate,
//! - a cloud/control-plane adapter that streams flag definitions to edge evaluators,
//! - an identity-aware adapter that enriches the [`crate::model::EvaluationContext`].
//!
//! Nothing here knows about Postgres, gRPC, OFREP, tokio, or any wire format. The port shapes are
//! designed for the OWNED stack so they do not need to change at cutover: they speak only in domain
//! types and a domain error.

use crate::model::{Flag, FlagKey};

/// A read port that yields a [`Flag`] definition by key.
///
/// Synchronous and infallible-of-runtime by design: the domain composes over plain values, and an
/// async/streaming adapter wraps this with its own runtime concerns. Returning `Ok(None)` means
/// "no such flag" (the caller decides the no-flag policy); `Err` means the source itself failed.
pub trait FlagSource {
    /// Fetch a flag by key. `Ok(None)` = absent; `Err` = source failure (fail-closed at caller).
    fn get_flag(&self, key: &FlagKey) -> Result<Option<Flag>, FlagSourceError>;
}

/// Errors a [`FlagSource`] may surface. Backend-neutral: adapters map their concrete failures into
/// these variants so the domain never depends on a specific store's error type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlagSourceError {
    /// The underlying source was unavailable (network, store down, lease lost, ...).
    Unavailable(String),
    /// The stored flag definition was corrupt or failed schema validation.
    Corrupt(String),
}

impl core::fmt::Display for FlagSourceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FlagSourceError::Unavailable(m) => write!(f, "flag source unavailable: {m}"),
            FlagSourceError::Corrupt(m) => write!(f, "flag definition corrupt: {m}"),
        }
    }
}

impl std::error::Error for FlagSourceError {}
