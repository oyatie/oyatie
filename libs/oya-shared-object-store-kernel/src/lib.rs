//! Object-store/CAS kernel — ADR-0520 + ADR-0536 D-11.
//!
//! # Why this crate exists
//!
//! ADR-0520 amends ADR-0196: SeaweedFS, Ceph RGW, and public-cloud object
//! stores are now transitional implementations behind a stable owned
//! `object-store-kernel` interface. ADR-0536 D-11 locks the destination shape:
//! a tenant-scoped content-addressed store using BLAKE3 addresses, no
//! cross-tenant deduplication, strong read-after-write, Object-Lock-style WORM
//! compliance for audit anchors, and a clear adapter boundary for transitional
//! backends.
//!
//! Application code MUST call this owned CAS port, not S3-style bucket/key
//! clients. Adapter crates may translate this port to SeaweedFS/Ceph/AWS/GCS
//! while preserving the tenant-scoped BLAKE3 address and WORM/audit contract.
//!
//! # Naming justification
//!
//! `oya-shared-object-store-kernel` follows BNF v4.1:
//! `oya-<axis:shared>-<topic:object-store>-<layer:kernel>`.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Mutex, MutexGuard};

const BLAKE3_HEX_LEN: usize = 64;
const TENANT_ID_PREFIX: &str = "ten_";
const MAX_TENANT_ID_LEN: usize = 128;
const MAX_REFERENCE_LEN: usize = 512;

// =====================================================================
// Addressing and policy types
// =====================================================================

/// Tenant identifier. Every CAS address is tenant-scoped so identical content
/// in two tenants never implies cross-tenant deduplication or shared KEK scope.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TenantId(String);

impl TenantId {
    /// Parse a canonical tenant id (`ten_...`).
    ///
    /// # Errors
    /// Returns `ObjectStoreError::InvalidTenantId` when the identifier is empty,
    /// malformed, too long, or contains non-canonical characters.
    pub fn parse(value: &str) -> Result<Self, ObjectStoreError> {
        if value.len() <= TENANT_ID_PREFIX.len()
            || value.len() > MAX_TENANT_ID_LEN
            || !value.starts_with(TENANT_ID_PREFIX)
            || !value[TENANT_ID_PREFIX.len()..].bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
            })
        {
            return Err(ObjectStoreError::InvalidTenantId);
        }
        Ok(Self(value.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// BLAKE3 digest encoded as 64 lowercase hex characters.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Blake3Digest(String);

impl Blake3Digest {
    /// Compute the BLAKE3 digest for a payload.
    #[must_use]
    pub fn for_payload(bytes: &[u8]) -> Self {
        Self(blake3::hash(bytes).to_hex().to_string())
    }

    /// Parse a lower-case BLAKE3 hex digest.
    ///
    /// # Errors
    /// Returns `ObjectStoreError::InvalidBlake3Digest` when the value is not
    /// exactly 64 lowercase hex characters.
    pub fn parse(value: &str) -> Result<Self, ObjectStoreError> {
        if is_lower_hex(value, BLAKE3_HEX_LEN) {
            Ok(Self(value.to_string()))
        } else {
            Err(ObjectStoreError::InvalidBlake3Digest)
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Blake3Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The owned content address: `(tenant_id, blake3(payload))`.
///
/// The tenant component is not decoration. It is the anti-dedup boundary that
/// preserves ADR-0536 D-8 crypto-shred and prevents cross-tenant side channels.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TenantScopedBlake3Address {
    pub tenant_id: TenantId,  // data_class: INTERNAL_ONLY
    pub digest: Blake3Digest, // data_class: INTERNAL_ONLY
}

impl TenantScopedBlake3Address {
    #[must_use]
    pub fn for_payload(tenant_id: TenantId, bytes: &[u8]) -> Self {
        Self {
            tenant_id,
            digest: Blake3Digest::for_payload(bytes),
        }
    }

    /// Build an address from already-computed parts.
    ///
    /// # Errors
    /// Returns validation errors from `TenantId` or `Blake3Digest` parsing.
    pub fn parse(tenant_id: &str, digest: &str) -> Result<Self, ObjectStoreError> {
        Ok(Self {
            tenant_id: TenantId::parse(tenant_id)?,
            digest: Blake3Digest::parse(digest)?,
        })
    }

    #[must_use]
    pub fn canonical(&self) -> String {
        format!("cas://{}/blake3/{}", self.tenant_id, self.digest)
    }
}

/// Per-tenant KEK boundary used by CAS writes.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TenantKekBoundary {
    pub tenant_id: TenantId,              // data_class: INTERNAL_ONLY
    pub kms_key_ref: String,              // data_class: INTERNAL_ONLY
    pub kms_key_version: u32,             // data_class: INTERNAL_ONLY
    pub ciphertext_ref: String,           // data_class: INTERNAL_ONLY
    pub crypto_shred_ref: Option<String>, // data_class: INTERNAL_ONLY
}

impl TenantKekBoundary {
    /// Build a KEK boundary.
    ///
    /// # Errors
    /// Returns `ObjectStoreError::InvalidKekBoundary` when references are empty,
    /// contain control characters, or the key version is zero.
    pub fn new(
        tenant_id: TenantId,
        kms_key_ref: impl Into<String>,
        kms_key_version: u32,
        ciphertext_ref: impl Into<String>,
        crypto_shred_ref: Option<String>,
    ) -> Result<Self, ObjectStoreError> {
        let boundary = Self {
            tenant_id,
            kms_key_ref: kms_key_ref.into(),
            kms_key_version,
            ciphertext_ref: ciphertext_ref.into(),
            crypto_shred_ref,
        };
        boundary.validate()?;
        Ok(boundary)
    }

    fn validate(&self) -> Result<(), ObjectStoreError> {
        if self.kms_key_version == 0
            || !is_valid_reference(&self.kms_key_ref)
            || !is_valid_reference(&self.ciphertext_ref)
            || self
                .crypto_shred_ref
                .as_ref()
                .is_some_and(|reference| !is_valid_reference(reference))
        {
            return Err(ObjectStoreError::InvalidKekBoundary);
        }
        Ok(())
    }
}

/// Object-Lock-style WORM mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CasWormMode {
    Governance,
    Compliance,
}

impl CasWormMode {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Governance => "governance",
            Self::Compliance => "compliance",
        }
    }
}

/// WORM policy required for CAS objects that anchor audit material.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CasWormPolicy {
    pub mode: CasWormMode,               // data_class: PUBLIC
    pub retain_until_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub legal_hold: bool,                // data_class: INTERNAL_ONLY
}

impl CasWormPolicy {
    #[must_use]
    pub const fn governance_until(retain_until_epoch_seconds: u64, legal_hold: bool) -> Self {
        Self {
            mode: CasWormMode::Governance,
            retain_until_epoch_seconds,
            legal_hold,
        }
    }

    #[must_use]
    pub const fn compliance_until(retain_until_epoch_seconds: u64, legal_hold: bool) -> Self {
        Self {
            mode: CasWormMode::Compliance,
            retain_until_epoch_seconds,
            legal_hold,
        }
    }

    fn validate(&self) -> Result<(), ObjectStoreError> {
        if self.retain_until_epoch_seconds == 0 && !self.legal_hold {
            return Err(ObjectStoreError::InvalidWormPolicy);
        }
        Ok(())
    }

    #[must_use]
    pub const fn deletion_protected_at(&self, epoch_seconds: u64) -> bool {
        self.legal_hold || epoch_seconds < self.retain_until_epoch_seconds
    }
}

/// Audit digest-chain anchor stored alongside CAS metadata.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CasAuditAnchor {
    pub audit_event_id: String,          // data_class: INTERNAL_ONLY
    pub digest_chain_head: Blake3Digest, // data_class: INTERNAL_ONLY
    pub anchored_at_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
}

impl CasAuditAnchor {
    /// Build an audit anchor.
    ///
    /// # Errors
    /// Returns `ObjectStoreError::InvalidAuditAnchor` when the event id is empty
    /// or the timestamp is zero.
    pub fn new(
        audit_event_id: impl Into<String>,
        digest_chain_head: Blake3Digest,
        anchored_at_epoch_seconds: u64,
    ) -> Result<Self, ObjectStoreError> {
        let anchor = Self {
            audit_event_id: audit_event_id.into(),
            digest_chain_head,
            anchored_at_epoch_seconds,
        };
        anchor.validate()?;
        Ok(anchor)
    }

    fn validate(&self) -> Result<(), ObjectStoreError> {
        if !is_valid_reference(&self.audit_event_id) || self.anchored_at_epoch_seconds == 0 {
            return Err(ObjectStoreError::InvalidAuditAnchor);
        }
        Ok(())
    }
}

/// Write-path durability locked by ADR-0536 D-11.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CasWritePath {
    ChainReplication3x,
}

/// Background repair/space efficiency path locked by ADR-0536 D-11.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CasRepairPath {
    LrcErasureCoding,
}

/// Destination durability policy: 3x chain replication on write, then
/// background re-encode to LRC erasure coding.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CasDurabilityPolicy {
    pub write_path: CasWritePath,   // data_class: PUBLIC
    pub repair_path: CasRepairPath, // data_class: PUBLIC
}

impl Default for CasDurabilityPolicy {
    fn default() -> Self {
        Self {
            write_path: CasWritePath::ChainReplication3x,
            repair_path: CasRepairPath::LrcErasureCoding,
        }
    }
}

/// Storage backend kind. Transitional kinds are adapters behind this owned CAS
/// contract; they are not destination API shapes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ObjectStoreBackendKind {
    InMemoryReference,
    OwnedCas,
    SeaweedFsS3,
    CephRgw,
    AwsS3,
    GoogleCloudStorage,
    AzureBlob,
}

impl ObjectStoreBackendKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::InMemoryReference => "in_memory_reference",
            Self::OwnedCas => "owned_cas",
            Self::SeaweedFsS3 => "seaweedfs_s3_transitional",
            Self::CephRgw => "ceph_rgw_transitional",
            Self::AwsS3 => "aws_s3_transitional",
            Self::GoogleCloudStorage => "gcs_transitional",
            Self::AzureBlob => "azure_blob_transitional",
        }
    }

    #[must_use]
    pub const fn is_transitional(self) -> bool {
        matches!(
            self,
            Self::SeaweedFsS3
                | Self::CephRgw
                | Self::AwsS3
                | Self::GoogleCloudStorage
                | Self::AzureBlob
        )
    }
}

/// Receipt boundary for adapters translating the owned CAS port to a
/// transitional object-store backend.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TransitionalAdapterBoundary {
    pub adapter_kind: ObjectStoreBackendKind, // data_class: PUBLIC
    pub provider_namespace: String,           // data_class: INTERNAL_ONLY
    pub provider_object_ref: String,          // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String,        // data_class: INTERNAL_ONLY
}

impl TransitionalAdapterBoundary {
    /// Build a transitional adapter boundary.
    ///
    /// # Errors
    /// Returns `ObjectStoreError::InvalidTransitionalBoundary` when the backend
    /// kind is not transitional or any provider reference is malformed.
    pub fn new(
        adapter_kind: ObjectStoreBackendKind,
        provider_namespace: impl Into<String>,
        provider_object_ref: impl Into<String>,
        provider_evidence_ref: impl Into<String>,
    ) -> Result<Self, ObjectStoreError> {
        let boundary = Self {
            adapter_kind,
            provider_namespace: provider_namespace.into(),
            provider_object_ref: provider_object_ref.into(),
            provider_evidence_ref: provider_evidence_ref.into(),
        };
        boundary.validate()?;
        Ok(boundary)
    }

    fn validate(&self) -> Result<(), ObjectStoreError> {
        if !self.adapter_kind.is_transitional()
            || !is_valid_reference(&self.provider_namespace)
            || !is_valid_reference(&self.provider_object_ref)
            || !is_valid_reference(&self.provider_evidence_ref)
        {
            return Err(ObjectStoreError::InvalidTransitionalBoundary);
        }
        Ok(())
    }
}

// =====================================================================
// Request/response types
// =====================================================================

/// CAS write request. The caller supplies bytes and the kernel verifies that
/// the supplied tenant-scoped address is the BLAKE3 address of those bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CasPutRequest {
    pub address: TenantScopedBlake3Address, // data_class: INTERNAL_ONLY
    pub bytes: Vec<u8>,                     // data_class: INTERNAL_ONLY
    pub kms_boundary: TenantKekBoundary,    // data_class: INTERNAL_ONLY
    pub worm_policy: CasWormPolicy,         // data_class: INTERNAL_ONLY
    pub audit_anchor: CasAuditAnchor,       // data_class: INTERNAL_ONLY
    pub durability: CasDurabilityPolicy,    // data_class: PUBLIC
    pub user_metadata: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64,    // data_class: INTERNAL_ONLY
}

impl CasPutRequest {
    /// Build a write request and compute the tenant-scoped BLAKE3 address from
    /// the payload.
    ///
    /// # Errors
    /// Returns validation errors from nested WORM/audit/KEK policy objects.
    pub fn new(
        tenant_id: TenantId,
        bytes: Vec<u8>,
        kms_boundary: TenantKekBoundary,
        worm_policy: CasWormPolicy,
        audit_anchor: CasAuditAnchor,
        requested_at_epoch_seconds: u64,
    ) -> Result<Self, ObjectStoreError> {
        let address = TenantScopedBlake3Address::for_payload(tenant_id, &bytes);
        let request = Self {
            address,
            bytes,
            kms_boundary,
            worm_policy,
            audit_anchor,
            durability: CasDurabilityPolicy::default(),
            user_metadata: BTreeMap::new(),
            requested_at_epoch_seconds,
        };
        request.validate()?;
        Ok(request)
    }

    fn validate(&self) -> Result<(), ObjectStoreError> {
        if self.requested_at_epoch_seconds == 0 {
            return Err(ObjectStoreError::InvalidRequestTimestamp);
        }
        self.kms_boundary.validate()?;
        if self.kms_boundary.tenant_id != self.address.tenant_id {
            return Err(ObjectStoreError::CrossTenantAccessDenied);
        }
        self.worm_policy.validate()?;
        self.audit_anchor.validate()?;
        let actual_digest = Blake3Digest::for_payload(&self.bytes);
        if actual_digest != self.address.digest {
            return Err(ObjectStoreError::AddressDigestMismatch {
                expected: self.address.digest.as_str().to_string(),
                actual: actual_digest.as_str().to_string(),
            });
        }
        if self.user_metadata.iter().any(|(key, value)| {
            !is_valid_reference(key) || value.bytes().any(|byte| byte.is_ascii_control())
        }) {
            return Err(ObjectStoreError::InvalidUserMetadata);
        }
        Ok(())
    }
}

/// Tenant-bound read/head request. The address tenant must match the caller
/// tenant, even when the BLAKE3 digest is known.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CasReadRequest {
    pub tenant_id: TenantId,                // data_class: INTERNAL_ONLY
    pub address: TenantScopedBlake3Address, // data_class: INTERNAL_ONLY
}

impl CasReadRequest {
    #[must_use]
    pub fn new(tenant_id: TenantId, address: TenantScopedBlake3Address) -> Self {
        Self { tenant_id, address }
    }

    fn validate(&self) -> Result<(), ObjectStoreError> {
        if self.tenant_id != self.address.tenant_id {
            return Err(ObjectStoreError::CrossTenantAccessDenied);
        }
        Ok(())
    }
}

/// Tenant-bound delete request. WORM policy is enforced before deletion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CasDeleteRequest {
    pub tenant_id: TenantId,                // data_class: INTERNAL_ONLY
    pub address: TenantScopedBlake3Address, // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64,    // data_class: INTERNAL_ONLY
    pub audit_event_id: String,             // data_class: INTERNAL_ONLY
}

impl CasDeleteRequest {
    #[must_use]
    pub fn new(
        tenant_id: TenantId,
        address: TenantScopedBlake3Address,
        requested_at_epoch_seconds: u64,
        audit_event_id: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id,
            address,
            requested_at_epoch_seconds,
            audit_event_id: audit_event_id.into(),
        }
    }

    fn validate(&self) -> Result<(), ObjectStoreError> {
        if self.tenant_id != self.address.tenant_id {
            return Err(ObjectStoreError::CrossTenantAccessDenied);
        }
        if self.requested_at_epoch_seconds == 0 || !is_valid_reference(&self.audit_event_id) {
            return Err(ObjectStoreError::InvalidDeleteRequest);
        }
        Ok(())
    }
}

/// Stored CAS object metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CasObjectRecord {
    pub address: TenantScopedBlake3Address, // data_class: INTERNAL_ONLY
    pub size_bytes: u64,                    // data_class: INTERNAL_ONLY
    pub kms_boundary: TenantKekBoundary,    // data_class: INTERNAL_ONLY
    pub worm_policy: CasWormPolicy,         // data_class: INTERNAL_ONLY
    pub audit_anchor: CasAuditAnchor,       // data_class: INTERNAL_ONLY
    pub durability: CasDurabilityPolicy,    // data_class: PUBLIC
    pub user_metadata: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
    pub stored_at_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
    pub backend_kind: ObjectStoreBackendKind, // data_class: PUBLIC
    pub adapter_boundary: Option<TransitionalAdapterBoundary>, // data_class: INTERNAL_ONLY
}

/// CAS object bytes plus metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CasObjectBytes {
    pub record: CasObjectRecord, // data_class: INTERNAL_ONLY
    pub bytes: Vec<u8>,          // data_class: INTERNAL_ONLY
}

/// Errors emitted by the object-store kernel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObjectStoreError {
    InvalidTenantId,
    InvalidBlake3Digest,
    AddressDigestMismatch {
        expected: String,
        actual: String,
    },
    CrossTenantAccessDenied,
    InvalidKekBoundary,
    InvalidWormPolicy,
    InvalidAuditAnchor,
    InvalidTransitionalBoundary,
    InvalidRequestTimestamp,
    InvalidUserMetadata,
    InvalidDeleteRequest,
    NotFound {
        tenant_id: String,
        digest: String,
    },
    WormRetentionActive {
        retain_until_epoch_seconds: u64,
        legal_hold: bool,
    },
    BackendUnavailable {
        detail: String,
    },
}

impl fmt::Display for ObjectStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTenantId => write!(f, "invalid tenant id; expected ten_<id>"),
            Self::InvalidBlake3Digest => {
                write!(f, "invalid BLAKE3 digest; expected 64 lowercase hex chars")
            }
            Self::AddressDigestMismatch { expected, actual } => write!(
                f,
                "CAS address digest mismatch: expected {expected}, computed {actual}"
            ),
            Self::CrossTenantAccessDenied => write!(f, "cross-tenant CAS access denied"),
            Self::InvalidKekBoundary => write!(f, "invalid tenant KEK boundary"),
            Self::InvalidWormPolicy => write!(f, "invalid WORM policy"),
            Self::InvalidAuditAnchor => write!(f, "invalid audit anchor"),
            Self::InvalidTransitionalBoundary => write!(f, "invalid transitional adapter boundary"),
            Self::InvalidRequestTimestamp => write!(f, "request timestamp must be non-zero"),
            Self::InvalidUserMetadata => write!(f, "invalid CAS user metadata"),
            Self::InvalidDeleteRequest => write!(f, "invalid CAS delete request"),
            Self::NotFound { tenant_id, digest } => {
                write!(
                    f,
                    "CAS object not found: tenant={tenant_id} blake3={digest}"
                )
            }
            Self::WormRetentionActive {
                retain_until_epoch_seconds,
                legal_hold,
            } => write!(
                f,
                "CAS object is WORM-protected until {retain_until_epoch_seconds} (legal_hold={legal_hold})"
            ),
            Self::BackendUnavailable { detail } => {
                write!(f, "object-store backend unavailable: {detail}")
            }
        }
    }
}

impl std::error::Error for ObjectStoreError {}

// =====================================================================
// Trait
// =====================================================================

/// Owned object-store/CAS seam per ADR-0520 and ADR-0536 D-11.
///
/// Every implementation — transitional SeaweedFS/Ceph/AWS/GCS/Azure adapters
/// and the future owned object store — implements this trait. The trait shape
/// is the destination CAS contract, not a vendor S3 API mirror.
pub trait ObjectStore: Send + Sync {
    fn put_cas(&self, request: CasPutRequest) -> Result<CasObjectRecord, ObjectStoreError>;

    fn head_cas(&self, request: CasReadRequest) -> Result<CasObjectRecord, ObjectStoreError>;

    fn get_cas(&self, request: CasReadRequest) -> Result<CasObjectBytes, ObjectStoreError>;

    fn delete_cas(&self, request: CasDeleteRequest) -> Result<(), ObjectStoreError>;

    fn adapter_boundary(
        &self,
        request: CasReadRequest,
    ) -> Result<Option<TransitionalAdapterBoundary>, ObjectStoreError>;

    fn backend_kind(&self) -> ObjectStoreBackendKind;
}

// =====================================================================
// Reference in-memory adapter
// =====================================================================
//
// The in-memory adapter is the kernel-shipped reference implementation. It lets
// tests exercise the owned CAS port without standing up SeaweedFS/Ceph. It also
// has an explicit transitional-adapter mode so adapter receipts can be tested
// without leaking S3 bucket/key APIs into the trait.

#[derive(Clone, Debug, Eq, PartialEq)]
struct StoredCasObject {
    bytes: Vec<u8>,
    record: CasObjectRecord,
}

#[derive(Debug, Default)]
struct InMemoryStorage {
    objects: BTreeMap<TenantScopedBlake3Address, StoredCasObject>,
}

/// Reference in-memory `ObjectStore`. Use in tests.
#[derive(Debug)]
pub struct InMemoryObjectStore {
    inner: Mutex<InMemoryStorage>,
    backend_kind: ObjectStoreBackendKind,
}

impl Default for InMemoryObjectStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryObjectStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(InMemoryStorage::default()),
            backend_kind: ObjectStoreBackendKind::InMemoryReference,
        }
    }

    /// Build an in-memory store that emits transitional adapter boundaries for
    /// tests. This does not change the owned CAS trait surface.
    ///
    /// # Errors
    /// Returns `ObjectStoreError::InvalidTransitionalBoundary` when the backend
    /// kind is not a transitional adapter kind.
    pub fn with_transitional_adapter(
        backend_kind: ObjectStoreBackendKind,
    ) -> Result<Self, ObjectStoreError> {
        if !backend_kind.is_transitional() {
            return Err(ObjectStoreError::InvalidTransitionalBoundary);
        }
        Ok(Self {
            inner: Mutex::new(InMemoryStorage::default()),
            backend_kind,
        })
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.lock().objects.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn lock(&self) -> MutexGuard<'_, InMemoryStorage> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn not_found(address: &TenantScopedBlake3Address) -> ObjectStoreError {
        ObjectStoreError::NotFound {
            tenant_id: address.tenant_id.as_str().to_string(),
            digest: address.digest.as_str().to_string(),
        }
    }

    fn transitional_boundary_for(
        &self,
        address: &TenantScopedBlake3Address,
    ) -> Result<Option<TransitionalAdapterBoundary>, ObjectStoreError> {
        if !self.backend_kind.is_transitional() {
            return Ok(None);
        }
        TransitionalAdapterBoundary::new(
            self.backend_kind,
            format!(
                "adapter/{}/{}",
                self.backend_kind.label(),
                address.tenant_id
            ),
            format!("cas/{}/{}", address.tenant_id, address.digest),
            format!(
                "evidence/object-store/{}/{}",
                address.tenant_id, address.digest
            ),
        )
        .map(Some)
    }
}

impl ObjectStore for InMemoryObjectStore {
    fn put_cas(&self, request: CasPutRequest) -> Result<CasObjectRecord, ObjectStoreError> {
        request.validate()?;
        let mut store = self.lock();
        if let Some(existing) = store.objects.get(&request.address) {
            if existing.bytes == request.bytes {
                return Ok(existing.record.clone());
            }
            let actual = Blake3Digest::for_payload(&request.bytes);
            return Err(ObjectStoreError::AddressDigestMismatch {
                expected: request.address.digest.as_str().to_string(),
                actual: actual.as_str().to_string(),
            });
        }

        let record = CasObjectRecord {
            address: request.address.clone(),
            size_bytes: request.bytes.len() as u64,
            kms_boundary: request.kms_boundary,
            worm_policy: request.worm_policy,
            audit_anchor: request.audit_anchor,
            durability: request.durability,
            user_metadata: request.user_metadata,
            stored_at_epoch_seconds: request.requested_at_epoch_seconds,
            backend_kind: self.backend_kind,
            adapter_boundary: self.transitional_boundary_for(&request.address)?,
        };

        store.objects.insert(
            record.address.clone(),
            StoredCasObject {
                bytes: request.bytes,
                record: record.clone(),
            },
        );
        Ok(record)
    }

    fn head_cas(&self, request: CasReadRequest) -> Result<CasObjectRecord, ObjectStoreError> {
        request.validate()?;
        let store = self.lock();
        store
            .objects
            .get(&request.address)
            .map(|stored| stored.record.clone())
            .ok_or_else(|| Self::not_found(&request.address))
    }

    fn get_cas(&self, request: CasReadRequest) -> Result<CasObjectBytes, ObjectStoreError> {
        request.validate()?;
        let store = self.lock();
        let stored = store
            .objects
            .get(&request.address)
            .ok_or_else(|| Self::not_found(&request.address))?;
        Ok(CasObjectBytes {
            record: stored.record.clone(),
            bytes: stored.bytes.clone(),
        })
    }

    fn delete_cas(&self, request: CasDeleteRequest) -> Result<(), ObjectStoreError> {
        request.validate()?;
        let mut store = self.lock();
        let stored = store
            .objects
            .get(&request.address)
            .ok_or_else(|| Self::not_found(&request.address))?;
        if stored
            .record
            .worm_policy
            .deletion_protected_at(request.requested_at_epoch_seconds)
        {
            return Err(ObjectStoreError::WormRetentionActive {
                retain_until_epoch_seconds: stored.record.worm_policy.retain_until_epoch_seconds,
                legal_hold: stored.record.worm_policy.legal_hold,
            });
        }
        store.objects.remove(&request.address);
        Ok(())
    }

    fn adapter_boundary(
        &self,
        request: CasReadRequest,
    ) -> Result<Option<TransitionalAdapterBoundary>, ObjectStoreError> {
        request.validate()?;
        let store = self.lock();
        let stored = store
            .objects
            .get(&request.address)
            .ok_or_else(|| Self::not_found(&request.address))?;
        Ok(stored.record.adapter_boundary.clone())
    }

    fn backend_kind(&self) -> ObjectStoreBackendKind {
        self.backend_kind
    }
}

// =====================================================================
// Helpers
// =====================================================================

fn is_lower_hex(value: &str, expected_len: usize) -> bool {
    value.len() == expected_len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn is_valid_reference(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed.len() <= MAX_REFERENCE_LEN
        && !trimmed.bytes().any(|byte| byte.is_ascii_control())
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn tenant(value: &str) -> TenantId {
        TenantId::parse(value).unwrap_or_else(|_| panic!("tenant parse: {value}"))
    }

    fn digest_for(value: &[u8]) -> Blake3Digest {
        Blake3Digest::for_payload(value)
    }

    fn kek_boundary(tenant_id: &TenantId) -> TenantKekBoundary {
        TenantKekBoundary::new(
            tenant_id.clone(),
            format!("kms/{tenant_id}/object-store"),
            1,
            format!("ct/{tenant_id}/object-store"),
            Some(format!("shred/{tenant_id}/object-store")),
        )
        .unwrap()
    }

    fn audit_anchor() -> CasAuditAnchor {
        CasAuditAnchor::new(
            "audit_evt_object_store_001",
            digest_for(b"audit-chain-head"),
            1_700_000_001,
        )
        .unwrap()
    }

    fn put_request(tenant_id: &TenantId, bytes: &[u8], retain_until: u64) -> CasPutRequest {
        CasPutRequest::new(
            tenant_id.clone(),
            bytes.to_vec(),
            kek_boundary(tenant_id),
            CasWormPolicy::compliance_until(retain_until, false),
            audit_anchor(),
            1_700_000_010,
        )
        .unwrap()
    }

    #[test]
    fn tenant_scoped_address_uses_blake3_hex() {
        let bytes = b"oyatie object-store payload";
        let address = TenantScopedBlake3Address::for_payload(tenant("ten_alpha"), bytes);
        assert_eq!(address.tenant_id.as_str(), "ten_alpha");
        assert_eq!(
            address.digest.as_str(),
            blake3::hash(bytes).to_hex().as_str()
        );
        assert_eq!(
            address.canonical(),
            format!("cas://ten_alpha/blake3/{}", blake3::hash(bytes).to_hex())
        );
    }

    #[test]
    fn digest_validation_rejects_uppercase_or_wrong_length() {
        assert_eq!(
            Blake3Digest::parse("ABCDEF").unwrap_err(),
            ObjectStoreError::InvalidBlake3Digest
        );
        assert_eq!(
            Blake3Digest::parse("abc").unwrap_err(),
            ObjectStoreError::InvalidBlake3Digest
        );
    }

    #[test]
    fn put_head_get_records_cas_worm_audit_contract() {
        let tenant_id = tenant("ten_alpha");
        let bytes = b"audit payload";
        let request = put_request(&tenant_id, bytes, 1_800_000_000);
        let address = request.address.clone();
        let store = InMemoryObjectStore::new();

        let put_record = store.put_cas(request).unwrap();
        assert_eq!(put_record.address, address);
        assert_eq!(put_record.size_bytes, bytes.len() as u64);
        assert_eq!(put_record.worm_policy.mode, CasWormMode::Compliance);
        assert_eq!(
            put_record.audit_anchor.audit_event_id,
            "audit_evt_object_store_001"
        );
        assert_eq!(put_record.durability, CasDurabilityPolicy::default());
        assert_eq!(
            put_record.backend_kind,
            ObjectStoreBackendKind::InMemoryReference
        );
        assert!(put_record.adapter_boundary.is_none());

        let read = CasReadRequest::new(tenant_id.clone(), address.clone());
        assert_eq!(store.head_cas(read.clone()).unwrap(), put_record);
        let object = store.get_cas(read).unwrap();
        assert_eq!(object.bytes, bytes);
        assert_eq!(object.record.address, address);
    }

    #[test]
    fn cross_tenant_reads_are_denied_even_when_digest_is_known() {
        let tenant_alpha = tenant("ten_alpha");
        let tenant_beta = tenant("ten_beta");
        let request = put_request(&tenant_alpha, b"same bytes", 1_800_000_000);
        let address = request.address.clone();
        let store = InMemoryObjectStore::new();
        store.put_cas(request).unwrap();

        let error = store
            .get_cas(CasReadRequest::new(tenant_beta, address))
            .unwrap_err();
        assert_eq!(error, ObjectStoreError::CrossTenantAccessDenied);
    }

    #[test]
    fn same_digest_in_different_tenants_is_stored_as_separate_cas_objects() {
        let payload = b"identical object payload";
        let alpha = tenant("ten_alpha");
        let beta = tenant("ten_beta");
        let alpha_request = put_request(&alpha, payload, 1_800_000_000);
        let beta_request = put_request(&beta, payload, 1_800_000_000);
        assert_eq!(alpha_request.address.digest, beta_request.address.digest);
        assert_ne!(alpha_request.address, beta_request.address);

        let store = InMemoryObjectStore::new();
        let alpha_record = store.put_cas(alpha_request).unwrap();
        let beta_record = store.put_cas(beta_request).unwrap();

        assert_eq!(store.len(), 2);
        assert_eq!(alpha_record.address.tenant_id.as_str(), "ten_alpha");
        assert_eq!(beta_record.address.tenant_id.as_str(), "ten_beta");
    }

    #[test]
    fn put_rejects_digest_that_does_not_match_payload() {
        let tenant_id = tenant("ten_alpha");
        let wrong_digest =
            Blake3Digest::parse("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap();
        let request = CasPutRequest {
            address: TenantScopedBlake3Address {
                tenant_id: tenant_id.clone(),
                digest: wrong_digest,
            },
            bytes: b"payload with different digest".to_vec(),
            kms_boundary: kek_boundary(&tenant_id),
            worm_policy: CasWormPolicy::compliance_until(1_800_000_000, false),
            audit_anchor: audit_anchor(),
            durability: CasDurabilityPolicy::default(),
            user_metadata: BTreeMap::new(),
            requested_at_epoch_seconds: 1_700_000_010,
        };
        let error = InMemoryObjectStore::new().put_cas(request).unwrap_err();
        assert!(matches!(
            error,
            ObjectStoreError::AddressDigestMismatch { .. }
        ));
    }

    #[test]
    fn worm_retention_blocks_delete_until_retention_expires() {
        let tenant_id = tenant("ten_alpha");
        let request = put_request(&tenant_id, b"worm payload", 1_800_000_000);
        let address = request.address.clone();
        let store = InMemoryObjectStore::new();
        store.put_cas(request).unwrap();

        let protected = store
            .delete_cas(CasDeleteRequest::new(
                tenant_id.clone(),
                address.clone(),
                1_799_999_999,
                "audit_evt_delete_attempt",
            ))
            .unwrap_err();
        assert_eq!(
            protected,
            ObjectStoreError::WormRetentionActive {
                retain_until_epoch_seconds: 1_800_000_000,
                legal_hold: false,
            }
        );

        store
            .delete_cas(CasDeleteRequest::new(
                tenant_id.clone(),
                address.clone(),
                1_800_000_001,
                "audit_evt_delete_after_retention",
            ))
            .unwrap();
        assert!(matches!(
            store
                .head_cas(CasReadRequest::new(tenant_id, address))
                .unwrap_err(),
            ObjectStoreError::NotFound { .. }
        ));
    }

    #[test]
    fn transitional_adapter_boundary_is_explicit_and_not_s3_shaped() {
        let tenant_id = tenant("ten_alpha");
        let request = put_request(&tenant_id, b"transitional payload", 1_800_000_000);
        let address = request.address.clone();
        let store =
            InMemoryObjectStore::with_transitional_adapter(ObjectStoreBackendKind::SeaweedFsS3)
                .unwrap();

        let record = store.put_cas(request).unwrap();
        assert_eq!(record.backend_kind, ObjectStoreBackendKind::SeaweedFsS3);
        let boundary = store
            .adapter_boundary(CasReadRequest::new(tenant_id, address))
            .unwrap()
            .expect("transitional boundary is present");
        assert_eq!(boundary.adapter_kind, ObjectStoreBackendKind::SeaweedFsS3);
        assert!(
            boundary
                .provider_namespace
                .starts_with("adapter/seaweedfs_s3_transitional/")
        );
        assert!(boundary.provider_object_ref.starts_with("cas/ten_alpha/"));
        assert!(
            boundary
                .provider_evidence_ref
                .starts_with("evidence/object-store/ten_alpha/")
        );
    }
}
