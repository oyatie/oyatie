//! Idempotency-Key kernel — per-µservice trait surface for ADR-0149.
//!
//! # ADR-0149 (Tier-A hyperscaler pattern)
//!
//! Every state-changing REST/gRPC operation across the 33 µservices
//! honors the canonical `Idempotency-Key` header (Stripe + AWS
//! ClientToken pattern). The store records `(tenant_id,
//! capability_id, idempotency_key) -> IdempotentResponse` and replays
//! the recorded response on duplicate retry.
//!
//! # Naming justification
//!
//! `shared-idempotency-key-kernel` follows BNF v4.1:
//! `oya-<axis:shared>-<topic:idempotency-key>-<layer:kernel>`.
//!
//! # References
//!
//! - docs/standards/idempotency-keys-canonical.md
//! - ADR-0149-idempotency-keys-canonical.md
//! - ADR-0145-inter-microservice-communication-reform.md

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;

/// Opaque 16-256-byte client-supplied idempotency key.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    /// Construct a key after validating canonical-length bounds (16..=256).
    ///
    /// # Errors
    /// - `IdempotencyStoreError::KeyTooShort` when len < 16.
    /// - `IdempotencyStoreError::KeyTooLong` when len > 256.
    pub fn try_new(raw: impl Into<String>) -> Result<Self, IdempotencyStoreError> {
        let raw = raw.into();
        if raw.len() < 16 {
            return Err(IdempotencyStoreError::KeyTooShort);
        }
        if raw.len() > 256 {
            return Err(IdempotencyStoreError::KeyTooLong);
        }
        Ok(IdempotencyKey(raw))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// SHA-256 fingerprint of the canonical-encoded request (method, path,
/// sorted-headers, body). Two requests with the same fingerprint are
/// semantically equivalent.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct RequestFingerprint(pub String);

/// Cached response returned on duplicate-key replay.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdempotentResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub recorded_fingerprint: RequestFingerprint,
}

/// Failure surface for the kernel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdempotencyStoreError {
    KeyTooShort,
    KeyTooLong,
    FingerprintMismatch {
        recorded: RequestFingerprint,
        attempted: RequestFingerprint,
    },
    SkeletonNotYetImplemented(&'static str),
}

impl fmt::Display for IdempotencyStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdempotencyStoreError::KeyTooShort => write!(
                f,
                "shared-idempotency-key-kernel: idempotency key shorter than 16 bytes"
            ),
            IdempotencyStoreError::KeyTooLong => write!(
                f,
                "shared-idempotency-key-kernel: idempotency key longer than 256 bytes"
            ),
            IdempotencyStoreError::FingerprintMismatch {
                recorded,
                attempted,
            } => write!(
                f,
                "shared-idempotency-key-kernel: fingerprint mismatch (recorded={recorded:?}, attempted={attempted:?})"
            ),
            IdempotencyStoreError::SkeletonNotYetImplemented(method) => write!(
                f,
                "shared-idempotency-key-kernel: {method} is skeleton-only \
                 (tracked under registry/placeholder-debt/adr-follow-ups.yaml#adr-0149-idempotency-impl)"
            ),
        }
    }
}

impl std::error::Error for IdempotencyStoreError {}

/// The trait every µservice integrates to record/replay idempotent
/// state-changing requests.
pub trait IdempotencyKeyStore: Send + Sync {
    /// Look up the cached response or compute it. The store MUST
    /// hold-or-write atomically (DB row insert with `ON CONFLICT
    /// DO NOTHING`).
    ///
    /// # Errors
    /// - `FingerprintMismatch` when the key was used with a different request.
    /// - `SkeletonNotYetImplemented` for the skeleton impl.
    fn get_or_compute(
        &self,
        key: &IdempotencyKey,
        fingerprint: &RequestFingerprint,
        compute: &mut dyn FnMut() -> IdempotentResponse,
    ) -> Result<IdempotentResponse, IdempotencyStoreError>;

    /// Consume (delete) the recorded response. Used by maintenance
    /// jobs to evict TTL-aged keys.
    ///
    /// # Errors
    /// - `SkeletonNotYetImplemented` for the skeleton impl.
    fn consume(&self, key: &IdempotencyKey) -> Result<(), IdempotencyStoreError>;

    /// Inspect without computing or consuming.
    ///
    /// # Errors
    /// - `SkeletonNotYetImplemented` for the skeleton impl.
    fn peek(
        &self,
        key: &IdempotencyKey,
    ) -> Result<Option<IdempotentResponse>, IdempotencyStoreError>;
}

/// In-memory reference implementation used by per-µservice integration
/// tests. Not for production (no persistence, no TTL eviction).
#[derive(Default)]
pub struct InMemoryIdempotencyKeyStore {
    inner: std::sync::Mutex<std::collections::HashMap<String, IdempotentResponse>>,
}

// Mutex lock panics on thread poisoning — same severity as a panic.
// ADR-0083 §Tier-3 permits this in reference implementations.
#[allow(clippy::expect_used)]
impl IdempotencyKeyStore for InMemoryIdempotencyKeyStore {
    fn get_or_compute(
        &self,
        key: &IdempotencyKey,
        fingerprint: &RequestFingerprint,
        compute: &mut dyn FnMut() -> IdempotentResponse,
    ) -> Result<IdempotentResponse, IdempotencyStoreError> {
        let mut inner = self.inner.lock().expect("mutex poisoned");
        if let Some(existing) = inner.get(key.as_str()) {
            if &existing.recorded_fingerprint != fingerprint {
                return Err(IdempotencyStoreError::FingerprintMismatch {
                    recorded: existing.recorded_fingerprint.clone(),
                    attempted: fingerprint.clone(),
                });
            }
            return Ok(existing.clone());
        }
        let response = compute();
        inner.insert(key.as_str().to_string(), response.clone());
        Ok(response)
    }

    fn consume(&self, key: &IdempotencyKey) -> Result<(), IdempotencyStoreError> {
        let mut inner = self.inner.lock().expect("mutex poisoned");
        inner.remove(key.as_str());
        Ok(())
    }

    fn peek(
        &self,
        key: &IdempotencyKey,
    ) -> Result<Option<IdempotentResponse>, IdempotencyStoreError> {
        let inner = self.inner.lock().expect("mutex poisoned");
        Ok(inner.get(key.as_str()).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good_response() -> IdempotentResponse {
        IdempotentResponse {
            status: 201,
            headers: vec![("Content-Type".into(), "application/json".into())],
            body: br#"{"ok":true}"#.to_vec(),
            recorded_fingerprint: RequestFingerprint("sha256:abc".into()),
        }
    }

    #[test]
    fn key_rejects_too_short() {
        assert_eq!(
            IdempotencyKey::try_new("short"),
            Err(IdempotencyStoreError::KeyTooShort)
        );
    }

    #[test]
    fn key_rejects_too_long() {
        let long = "x".repeat(257);
        assert_eq!(
            IdempotencyKey::try_new(long),
            Err(IdempotencyStoreError::KeyTooLong)
        );
    }

    #[test]
    fn in_memory_store_records_first_call_and_replays_second() {
        let store = InMemoryIdempotencyKeyStore::default();
        let key = IdempotencyKey::try_new("01HMZ1234567890ABCDEF0123").expect("key");
        let fp = RequestFingerprint("sha256:abc".into());
        let mut compute_count = 0;
        let mut compute = || {
            compute_count += 1;
            good_response()
        };
        let first = store
            .get_or_compute(&key, &fp, &mut compute)
            .expect("first call");
        let second = store
            .get_or_compute(&key, &fp, &mut compute)
            .expect("replay");
        assert_eq!(first, second);
        assert_eq!(compute_count, 1, "compute must run once across two calls");
    }

    #[test]
    fn in_memory_store_rejects_fingerprint_mismatch() {
        let store = InMemoryIdempotencyKeyStore::default();
        let key = IdempotencyKey::try_new("01HMZ1234567890ABCDEF0123").expect("key");
        let fp_a = RequestFingerprint("sha256:abc".into());
        let fp_b = RequestFingerprint("sha256:xyz".into());
        let mut compute_a = || good_response();
        store
            .get_or_compute(&key, &fp_a, &mut compute_a)
            .expect("first call");
        let mut compute_b = || good_response();
        let err = store
            .get_or_compute(&key, &fp_b, &mut compute_b)
            .expect_err("fingerprint mismatch");
        assert!(matches!(
            err,
            IdempotencyStoreError::FingerprintMismatch { .. }
        ));
    }

    #[test]
    fn error_display_carries_follow_up_pointer() {
        let err = IdempotencyStoreError::SkeletonNotYetImplemented("get_or_compute");
        let msg = format!("{err}");
        assert!(msg.contains("adr-0149-idempotency-impl"));
    }
}
