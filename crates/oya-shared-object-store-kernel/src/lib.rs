//! Object-store kernel — ADR-0196.
//!
//! # Why this crate exists
//!
//! ADR-0196 establishes SeaweedFS as the canonical S3-compatible object
//! store with Ceph RGW as the named scale-up path. Application code MUST
//! NOT call SeaweedFS / Ceph / AWS-S3 directly. This crate exposes the
//! `ObjectStore` trait — the inviolate seam between application code and
//! the backend implementation.
//!
//! The crate is **pure-Rust + dependency-free** at kernel scope (per
//! ADR-0083 kernel-tier invariant); concrete adapters (SeaweedFS, Ceph
//! RGW, AWS S3, GCS, Azure Blob) live in companion adapter crates that
//! import this kernel.
//!
//! # In-house roadmap (per ADR-0196 §In-house roadmap)
//!
//! - Phase 0 (TODAY) — SeaweedFS adapter via this trait.
//! - Phase 1 — Ceph RGW, AWS S3, GCS, Azure Blob adapters via the same trait.
//! - Phase 2 — `oya-object-store-server` in-house build behind the same trait.
//!
//! The trait surface is designed so the migration is a single adapter swap.
//!
//! # Naming justification
//!
//! `oya-shared-object-store-kernel` follows BNF v4.1:
//! `oya-<axis:shared>-<topic:object-store>-<layer:kernel>`.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;
use std::time::Duration;

// =====================================================================
// Types
// =====================================================================

/// Bucket name. Validated to follow ADR-0196 D-3 canonical convention:
/// `oya-<purpose>-<tenant-or-shared>-<env>`.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct BucketName(String);

impl BucketName {
    /// Parses a bucket name and validates the canonical shape.
    ///
    /// # Errors
    /// Returns `ObjectStoreError::InvalidBucketName` if the name does not
    /// satisfy the ADR-0196 D-3 shape.
    pub fn parse(name: &str) -> Result<Self, ObjectStoreError> {
        if name.len() < 3 || name.len() > 63 {
            return Err(ObjectStoreError::InvalidBucketName);
        }
        // S3 + SeaweedFS shared shape: lowercase letters, digits, hyphens
        // (no dots — to avoid TLS-SAN ambiguity in path-style endpoints).
        if !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(ObjectStoreError::InvalidBucketName);
        }
        if !name.starts_with("oya-") {
            return Err(ObjectStoreError::InvalidBucketName);
        }
        // canonical purpose-tenant-env shape: minimum 4 hyphen-segments
        // (oya, purpose, tenant-or-shared, env)
        let segments = name.split('-').count();
        if segments < 4 {
            return Err(ObjectStoreError::InvalidBucketName);
        }
        Ok(Self(name.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BucketName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Object key (the path-within-bucket).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ObjectKey(String);

impl ObjectKey {
    /// Parses an object key.
    ///
    /// # Errors
    /// Returns `ObjectStoreError::InvalidObjectKey` if the key is empty,
    /// exceeds 1024 bytes, contains control characters, or begins with
    /// `/`.
    pub fn parse(key: &str) -> Result<Self, ObjectStoreError> {
        if key.is_empty() || key.len() > 1024 {
            return Err(ObjectStoreError::InvalidObjectKey);
        }
        if key.starts_with('/') {
            return Err(ObjectStoreError::InvalidObjectKey);
        }
        if key.chars().any(|c| c.is_control()) {
            return Err(ObjectStoreError::InvalidObjectKey);
        }
        Ok(Self(key.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ObjectKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Direction for a pre-signed URL.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresignDirection {
    Get,
    Put,
}

/// Pre-signed URL request — bounded by ADR-0196 D-5.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresignRequest {
    pub bucket: BucketName,
    pub key: ObjectKey,
    pub direction: PresignDirection,
    pub ttl: Duration,
    pub content_type: Option<String>,
}

impl PresignRequest {
    /// Maximum allowed TTL for GET pre-signed URLs (ADR-0196 D-5).
    pub const MAX_GET_TTL: Duration = Duration::from_secs(15 * 60);
    /// Maximum allowed TTL for PUT pre-signed URLs (ADR-0196 D-5).
    pub const MAX_PUT_TTL: Duration = Duration::from_secs(30 * 60);

    /// Validates the request against ADR-0196 D-5 TTL caps.
    ///
    /// # Errors
    /// Returns `ObjectStoreError::PresignTtlExceeded` if the TTL is above
    /// the cap for the direction.
    pub fn validate(&self) -> Result<(), ObjectStoreError> {
        let cap = match self.direction {
            PresignDirection::Get => Self::MAX_GET_TTL,
            PresignDirection::Put => Self::MAX_PUT_TTL,
        };
        if self.ttl > cap {
            return Err(ObjectStoreError::PresignTtlExceeded {
                requested_seconds: self.ttl.as_secs(),
                cap_seconds: cap.as_secs(),
            });
        }
        if self.ttl.is_zero() {
            return Err(ObjectStoreError::PresignTtlInvalid);
        }
        Ok(())
    }
}

/// A pre-signed URL.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresignedUrl {
    pub url: String,
    pub expires_at_unix_secs: u64,
}

/// Object metadata returned on stat / head.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectMetadata {
    pub key: ObjectKey,
    pub size_bytes: u64,
    pub etag: String,
    pub last_modified_unix_secs: u64,
    /// SHA-256 per ADR-0196 D-6.
    pub sha256_hex: Option<String>,
    pub user_metadata: BTreeMap<String, String>,
}

/// Errors emitted by the kernel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObjectStoreError {
    InvalidBucketName,
    InvalidObjectKey,
    PresignTtlExceeded {
        requested_seconds: u64,
        cap_seconds: u64,
    },
    PresignTtlInvalid,
    NotFound {
        bucket: String,
        key: String,
    },
    IntegrityFailure {
        bucket: String,
        key: String,
    },
    BackendUnavailable {
        detail: String,
    },
}

impl fmt::Display for ObjectStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBucketName => {
                write!(f, "invalid bucket name (ADR-0196 D-3 canonical convention)")
            }
            Self::InvalidObjectKey => write!(f, "invalid object key"),
            Self::PresignTtlExceeded {
                requested_seconds,
                cap_seconds,
            } => write!(
                f,
                "presign TTL {requested_seconds}s exceeds cap {cap_seconds}s (ADR-0196 D-5)"
            ),
            Self::PresignTtlInvalid => write!(f, "presign TTL must be non-zero"),
            Self::NotFound { bucket, key } => {
                write!(f, "object not found: {bucket}/{key}")
            }
            Self::IntegrityFailure { bucket, key } => write!(
                f,
                "object integrity check failed: {bucket}/{key} (ADR-0196 D-6)"
            ),
            Self::BackendUnavailable { detail } => {
                write!(f, "backend unavailable: {detail}")
            }
        }
    }
}

impl std::error::Error for ObjectStoreError {}

// =====================================================================
// Trait
// =====================================================================

/// The canonical object-store seam per ADR-0196.
///
/// Every backend (SeaweedFS, Ceph RGW, AWS S3, GCS, Azure Blob, the
/// future in-house `oya-object-store-server`) implements this trait.
/// Application code calls only this trait, never the backend directly.
pub trait ObjectStore: Send + Sync {
    /// Stat an object (HEAD operation).
    ///
    /// # Errors
    /// Returns `ObjectStoreError::NotFound` if the object does not exist;
    /// `ObjectStoreError::BackendUnavailable` on transient backend
    /// failure.
    fn head(
        &self,
        bucket: &BucketName,
        key: &ObjectKey,
    ) -> Result<ObjectMetadata, ObjectStoreError>;

    /// Get object bytes.
    ///
    /// # Errors
    /// As for `head`; also `IntegrityFailure` if the stored SHA-256 does
    /// not match the read payload per ADR-0196 D-6.
    fn get(&self, bucket: &BucketName, key: &ObjectKey) -> Result<Vec<u8>, ObjectStoreError>;

    /// Put object bytes; returns the stored metadata.
    ///
    /// # Errors
    /// Returns `ObjectStoreError::BackendUnavailable` on transient backend
    /// failure.
    fn put(
        &self,
        bucket: &BucketName,
        key: &ObjectKey,
        bytes: &[u8],
        user_metadata: &BTreeMap<String, String>,
    ) -> Result<ObjectMetadata, ObjectStoreError>;

    /// Delete object.
    ///
    /// # Errors
    /// Returns `ObjectStoreError::NotFound` if the object does not exist
    /// at the time of delete.
    fn delete(&self, bucket: &BucketName, key: &ObjectKey) -> Result<(), ObjectStoreError>;

    /// Pre-signed URL (per ADR-0196 D-5).
    ///
    /// # Errors
    /// As for `PresignRequest::validate`; backend errors via
    /// `BackendUnavailable`.
    fn presign(&self, request: &PresignRequest) -> Result<PresignedUrl, ObjectStoreError>;

    /// Backend identification (for ops dashboards + cost attribution).
    fn backend_kind(&self) -> &'static str;
}

// =====================================================================
// Reference in-memory adapter
// =====================================================================
//
// The in-memory adapter is the kernel-shipped reference implementation.
// It lets test code in downstream crates exercise the `ObjectStore`
// trait without standing up SeaweedFS. Production backends live in
// companion adapter crates (`oya-shared-object-store-seaweedfs-adapter`,
// `oya-shared-object-store-ceph-rgw-adapter`, etc.).

use std::sync::{Mutex, MutexGuard};

#[derive(Debug, Default)]
struct InMemoryStorage {
    /// (bucket, key) -> (bytes, metadata)
    objects: BTreeMap<(String, String), (Vec<u8>, ObjectMetadata)>,
}

/// Reference in-memory `ObjectStore`. Use in tests.
#[derive(Debug, Default)]
pub struct InMemoryObjectStore {
    inner: Mutex<InMemoryStorage>,
    /// Deterministic clock advance (for tests).
    clock_offset_seconds: u64,
    /// Backend-identification string surfaced via `backend_kind`.
    backend_id: &'static str,
}

impl InMemoryObjectStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(InMemoryStorage::default()),
            clock_offset_seconds: 1_700_000_000, // deterministic baseline
            backend_id: "in-memory-reference",
        }
    }

    fn lock(&self) -> MutexGuard<'_, InMemoryStorage> {
        // Tests never poison; production never calls in-memory adapter
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn compute_sha256_hex(bytes: &[u8]) -> String {
        // Deterministic non-crypto digest sufficient for kernel-scope
        // reference: we are not shipping crypto in the kernel; real
        // adapters use the SHA-256 of the backend client.
        // We implement a tiny FNV-1a 64-bit hash for stability across
        // platforms; this is documented as kernel-reference behavior.
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for &b in bytes {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01B3);
        }
        format!("{h:016x}")
    }
}

impl ObjectStore for InMemoryObjectStore {
    fn head(
        &self,
        bucket: &BucketName,
        key: &ObjectKey,
    ) -> Result<ObjectMetadata, ObjectStoreError> {
        let store = self.lock();
        store
            .objects
            .get(&(bucket.as_str().to_string(), key.as_str().to_string()))
            .map(|(_bytes, meta)| meta.clone())
            .ok_or_else(|| ObjectStoreError::NotFound {
                bucket: bucket.as_str().to_string(),
                key: key.as_str().to_string(),
            })
    }

    fn get(&self, bucket: &BucketName, key: &ObjectKey) -> Result<Vec<u8>, ObjectStoreError> {
        let store = self.lock();
        let (bytes, meta) = store
            .objects
            .get(&(bucket.as_str().to_string(), key.as_str().to_string()))
            .ok_or_else(|| ObjectStoreError::NotFound {
                bucket: bucket.as_str().to_string(),
                key: key.as_str().to_string(),
            })?;
        // Integrity check (ADR-0196 D-6): re-compute and compare.
        let actual = Self::compute_sha256_hex(bytes);
        if let Some(expected) = &meta.sha256_hex
            && &actual != expected
        {
            return Err(ObjectStoreError::IntegrityFailure {
                bucket: bucket.as_str().to_string(),
                key: key.as_str().to_string(),
            });
        }
        Ok(bytes.clone())
    }

    fn put(
        &self,
        bucket: &BucketName,
        key: &ObjectKey,
        bytes: &[u8],
        user_metadata: &BTreeMap<String, String>,
    ) -> Result<ObjectMetadata, ObjectStoreError> {
        let mut store = self.lock();
        let sha = Self::compute_sha256_hex(bytes);
        let meta = ObjectMetadata {
            key: key.clone(),
            size_bytes: bytes.len() as u64,
            etag: format!("\"{sha}\""),
            last_modified_unix_secs: self.clock_offset_seconds,
            sha256_hex: Some(sha),
            user_metadata: user_metadata.clone(),
        };
        store.objects.insert(
            (bucket.as_str().to_string(), key.as_str().to_string()),
            (bytes.to_vec(), meta.clone()),
        );
        Ok(meta)
    }

    fn delete(&self, bucket: &BucketName, key: &ObjectKey) -> Result<(), ObjectStoreError> {
        let mut store = self.lock();
        let key_pair = (bucket.as_str().to_string(), key.as_str().to_string());
        store
            .objects
            .remove(&key_pair)
            .map(|_| ())
            .ok_or(ObjectStoreError::NotFound {
                bucket: bucket.as_str().to_string(),
                key: key.as_str().to_string(),
            })
    }

    fn presign(&self, request: &PresignRequest) -> Result<PresignedUrl, ObjectStoreError> {
        request.validate()?;
        let verb = match request.direction {
            PresignDirection::Get => "GET",
            PresignDirection::Put => "PUT",
        };
        let expires = self.clock_offset_seconds + request.ttl.as_secs();
        Ok(PresignedUrl {
            url: format!(
                "in-memory://{}/{}?verb={verb}&expires={expires}",
                request.bucket, request.key
            ),
            expires_at_unix_secs: expires,
        })
    }

    fn backend_kind(&self) -> &'static str {
        self.backend_id
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn bucket(name: &str) -> BucketName {
        BucketName::parse(name).unwrap_or_else(|_| panic!("bucket parse: {name}"))
    }
    fn key(k: &str) -> ObjectKey {
        ObjectKey::parse(k).unwrap_or_else(|_| panic!("key parse: {k}"))
    }

    // ---- Bucket / key validation ----

    #[test]
    fn bucket_canonical_shape_accepted() {
        let b = BucketName::parse("oya-evidence-shared-prod").unwrap();
        assert_eq!(b.as_str(), "oya-evidence-shared-prod");
    }

    #[test]
    fn bucket_without_oya_prefix_rejected() {
        let err = BucketName::parse("not-oya-evidence-prod").unwrap_err();
        assert_eq!(err, ObjectStoreError::InvalidBucketName);
    }

    #[test]
    fn bucket_uppercase_rejected() {
        let err = BucketName::parse("oya-Evidence-shared-prod").unwrap_err();
        assert_eq!(err, ObjectStoreError::InvalidBucketName);
    }

    #[test]
    fn bucket_too_short_rejected() {
        let err = BucketName::parse("oya").unwrap_err();
        assert_eq!(err, ObjectStoreError::InvalidBucketName);
    }

    #[test]
    fn bucket_missing_segments_rejected() {
        let err = BucketName::parse("oya-shared").unwrap_err();
        assert_eq!(err, ObjectStoreError::InvalidBucketName);
    }

    #[test]
    fn object_key_empty_rejected() {
        assert_eq!(
            ObjectKey::parse("").unwrap_err(),
            ObjectStoreError::InvalidObjectKey
        );
    }

    #[test]
    fn object_key_leading_slash_rejected() {
        assert_eq!(
            ObjectKey::parse("/leading").unwrap_err(),
            ObjectStoreError::InvalidObjectKey
        );
    }

    #[test]
    fn object_key_control_char_rejected() {
        assert_eq!(
            ObjectKey::parse("with\nnewline").unwrap_err(),
            ObjectStoreError::InvalidObjectKey
        );
    }

    // ---- Presign TTL cap enforcement (ADR-0196 D-5) ----

    #[test]
    fn presign_get_ttl_at_cap_accepted() {
        let req = PresignRequest {
            bucket: bucket("oya-evidence-shared-prod"),
            key: key("audit/2026/01/seal.bin"),
            direction: PresignDirection::Get,
            ttl: PresignRequest::MAX_GET_TTL,
            content_type: None,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn presign_get_ttl_over_cap_rejected() {
        let req = PresignRequest {
            bucket: bucket("oya-evidence-shared-prod"),
            key: key("audit/2026/01/seal.bin"),
            direction: PresignDirection::Get,
            ttl: PresignRequest::MAX_GET_TTL + Duration::from_secs(1),
            content_type: None,
        };
        let err = req.validate().unwrap_err();
        assert!(matches!(err, ObjectStoreError::PresignTtlExceeded { .. }));
    }

    #[test]
    fn presign_put_ttl_over_cap_rejected() {
        let req = PresignRequest {
            bucket: bucket("oya-evidence-shared-prod"),
            key: key("upload/blob.bin"),
            direction: PresignDirection::Put,
            ttl: PresignRequest::MAX_PUT_TTL + Duration::from_secs(60),
            content_type: Some("application/octet-stream".into()),
        };
        let err = req.validate().unwrap_err();
        assert!(matches!(err, ObjectStoreError::PresignTtlExceeded { .. }));
    }

    #[test]
    fn presign_zero_ttl_rejected() {
        let req = PresignRequest {
            bucket: bucket("oya-evidence-shared-prod"),
            key: key("k"),
            direction: PresignDirection::Get,
            ttl: Duration::ZERO,
            content_type: None,
        };
        assert_eq!(
            req.validate().unwrap_err(),
            ObjectStoreError::PresignTtlInvalid
        );
    }

    // ---- In-memory adapter round-trip ----

    #[test]
    fn in_memory_put_get_round_trip() {
        let store = InMemoryObjectStore::new();
        let b = bucket("oya-evidence-shared-prod");
        let k = key("audit/2026/01/seal.bin");
        let payload = b"oyatie-test-payload".to_vec();
        let meta = store.put(&b, &k, &payload, &BTreeMap::new()).unwrap();
        assert_eq!(meta.size_bytes, payload.len() as u64);

        let got = store.get(&b, &k).unwrap();
        assert_eq!(got, payload);

        let head = store.head(&b, &k).unwrap();
        assert_eq!(head.size_bytes, payload.len() as u64);
        assert!(head.sha256_hex.is_some());
    }

    #[test]
    fn in_memory_get_not_found() {
        let store = InMemoryObjectStore::new();
        let b = bucket("oya-evidence-shared-prod");
        let k = key("does-not-exist");
        let err = store.get(&b, &k).unwrap_err();
        assert!(matches!(err, ObjectStoreError::NotFound { .. }));
    }

    #[test]
    fn in_memory_delete_then_get_not_found() {
        let store = InMemoryObjectStore::new();
        let b = bucket("oya-evidence-shared-prod");
        let k = key("to-delete");
        store.put(&b, &k, b"x", &BTreeMap::new()).unwrap();
        store.delete(&b, &k).unwrap();
        assert!(matches!(
            store.get(&b, &k).unwrap_err(),
            ObjectStoreError::NotFound { .. }
        ));
    }

    #[test]
    fn in_memory_presign_round_trip() {
        let store = InMemoryObjectStore::new();
        let req = PresignRequest {
            bucket: bucket("oya-evidence-shared-prod"),
            key: key("artifact.bin"),
            direction: PresignDirection::Put,
            ttl: Duration::from_secs(600),
            content_type: Some("application/octet-stream".into()),
        };
        let url = store.presign(&req).unwrap();
        assert!(url.url.contains("oya-evidence-shared-prod"));
        assert!(url.url.contains("verb=PUT"));
        assert!(url.expires_at_unix_secs > 1_700_000_000);
    }

    #[test]
    fn in_memory_backend_kind_stable() {
        let store = InMemoryObjectStore::new();
        assert_eq!(store.backend_kind(), "in-memory-reference");
    }
}
