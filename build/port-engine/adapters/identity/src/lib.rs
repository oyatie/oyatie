//! # port-engine-identity — receipt `engine_digest` binder (W0-B Slice 9).
//!
//! ADR-0637 D2: `engine_digest` is one of the six receipt axes. The kernel COMPARES digests and
//! never computes them. This adapter owns a hermetic engine-identity manifest (crate set + wave /
//! slice label) and hashes its bytes via `port-engine-hash`. Binary hash of the driver is deferred
//! (unstable across rebuilds); identity-as-data is the W0-B binding.
#![forbid(unsafe_code)]

use port_engine_api::Digest;
use port_engine_hash::digest_bytes;

/// Embedded engine identity manifest (hermetic; no lock absorb).
const ENGINE_IDENTITY_JSON: &str = include_str!("engine-identity-v0.json");

/// Fail-closed readiness gate. `true` once Slice 9 engine identity is present.
pub const fn w0_ready() -> bool {
    true
}

/// Content digest of the embedded engine identity manifest (`sha256:<hex>`).
#[must_use]
pub fn engine_digest() -> Digest {
    digest_bytes(ENGINE_IDENTITY_JSON.as_bytes())
}

/// Borrow the embedded identity JSON (diagnostics / golden tests).
#[must_use]
pub fn identity_json() -> &'static str {
    ENGINE_IDENTITY_JSON
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice9_claims_identity_readiness() {
        assert!(w0_ready());
    }

    #[test]
    fn engine_digest_is_stable_sha256() {
        let d = engine_digest();
        assert!(d.0.starts_with("sha256:"));
        assert_eq!(d.0.len(), "sha256:".len() + 64);
        assert_eq!(d, engine_digest());
        assert!(identity_json().contains("port-engine-identity"));
        assert!(identity_json().contains("\"slice\": 9"));
    }
}
