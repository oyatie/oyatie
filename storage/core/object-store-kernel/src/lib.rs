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
//! Application code MUST call this owned CAS port, not vendor bucket/key
//! clients. Adapter crates may translate this port to bridge implementations
//! while preserving the tenant-scoped BLAKE3 address and WORM/audit contract.
//!
//! # Current placement
//!
//! This crate is the capability-first storage home for the stable W1 CAS port;
//! compatibility shims in legacy `shared-*` homes are not destination authority.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Mutex, MutexGuard};

const BLAKE3_HEX_LEN: usize = 64;
const TENANT_ID_PREFIX: &str = "ten_";
const MAX_TENANT_ID_LEN: usize = 128;
const MAX_REFERENCE_LEN: usize = 512;
const MAX_PAYLOAD_CHUNK_BYTES: usize = 16 * 1024 * 1024;

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
        Self::for_digest(tenant_id, Blake3Digest::for_payload(bytes))
    }

    #[must_use]
    pub const fn for_digest(tenant_id: TenantId, digest: Blake3Digest) -> Self {
        Self { tenant_id, digest }
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
    pub const fn write_protected_after(&self, epoch_seconds: u64) -> bool {
        self.legal_hold || self.retain_until_epoch_seconds > epoch_seconds
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

/// Storage backend kind exposed to diagnostics. Transitional bridge details
/// collapse to a destination-neutral adapter class so the stable interface does
/// not freeze today’s bridge implementations into the owned CAS contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ObjectStoreBackendKind {
    InMemoryReference,
    OwnedCas,
    TransitionalAdapter,
}

impl ObjectStoreBackendKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::InMemoryReference => "in_memory_reference",
            Self::OwnedCas => "owned_cas",
            Self::TransitionalAdapter => "transitional_adapter",
        }
    }

    #[must_use]
    pub const fn is_transitional(self) -> bool {
        matches!(self, Self::TransitionalAdapter)
    }
}

/// Destination-neutral class for a transitional object-store adapter. Concrete
/// vendor or bridge names live in adapter-local config and evidence records, not
/// in the stable `ObjectStore` trait shape.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TransitionalAdapterClass {
    ProtocolCompatible,
    ObjectGateway,
    BlobCompatible,
}

impl TransitionalAdapterClass {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ProtocolCompatible => "protocol_compatible",
            Self::ObjectGateway => "object_gateway",
            Self::BlobCompatible => "blob_compatible",
        }
    }
}

/// Receipt boundary for adapters translating the owned CAS port to a
/// transitional object-store backend.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TransitionalAdapterBoundary {
    pub adapter_class: TransitionalAdapterClass, // data_class: PUBLIC
    pub adapter_id: String,                      // data_class: INTERNAL_ONLY
    pub adapter_namespace: String,               // data_class: INTERNAL_ONLY
    pub adapter_object_ref: String,              // data_class: INTERNAL_ONLY
    pub adapter_evidence_ref: String,            // data_class: INTERNAL_ONLY
}

impl TransitionalAdapterBoundary {
    /// Build a transitional adapter boundary.
    ///
    /// # Errors
    /// Returns `ObjectStoreError::InvalidTransitionalBoundary` when adapter
    /// identity or adapter references are malformed.
    pub fn new(
        adapter_class: TransitionalAdapterClass,
        adapter_id: impl Into<String>,
        adapter_namespace: impl Into<String>,
        adapter_object_ref: impl Into<String>,
        adapter_evidence_ref: impl Into<String>,
    ) -> Result<Self, ObjectStoreError> {
        let boundary = Self {
            adapter_class,
            adapter_id: adapter_id.into(),
            adapter_namespace: adapter_namespace.into(),
            adapter_object_ref: adapter_object_ref.into(),
            adapter_evidence_ref: adapter_evidence_ref.into(),
        };
        boundary.validate()?;
        Ok(boundary)
    }

    fn validate(&self) -> Result<(), ObjectStoreError> {
        if !is_valid_reference(&self.adapter_id)
            || !is_valid_reference(&self.adapter_namespace)
            || !is_valid_reference(&self.adapter_object_ref)
            || !is_valid_reference(&self.adapter_evidence_ref)
        {
            return Err(ObjectStoreError::InvalidTransitionalBoundary);
        }
        Ok(())
    }
}

/// One payload chunk in the destination CAS write/read contract. The trait is
/// chunk-aware so real adapters do not have to expose a vendor bucket/key API or
/// pretend infinite-scale objects are whole-buffer values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CasPayloadChunk {
    pub ordinal: u32,         // data_class: INTERNAL_ONLY
    pub size_bytes: u64,      // data_class: INTERNAL_ONLY
    pub digest: Blake3Digest, // data_class: INTERNAL_ONLY
}

/// Chunked CAS payload manifest with a root BLAKE3 digest over the ordered
/// bytes. It deliberately carries digests and sizes, not object bytes; bytes
/// flow through `CasPayloadReader` / `CasPayloadSink`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CasPayload {
    pub total_size_bytes: u64,        // data_class: INTERNAL_ONLY
    pub root_digest: Blake3Digest,    // data_class: INTERNAL_ONLY
    pub chunks: Vec<CasPayloadChunk>, // data_class: INTERNAL_ONLY
}

impl CasPayload {
    /// Build a payload manifest from a single in-memory buffer. This helper is
    /// for tests and small callers; the stable `ObjectStore` trait remains
    /// reader/sink based.
    ///
    /// # Errors
    /// Returns `ObjectStoreError::InvalidPayload` when payload accounting fails.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ObjectStoreError> {
        Self::from_chunks(&[bytes.to_vec()])
    }

    /// Build a payload manifest from ordered chunks.
    ///
    /// # Errors
    /// Returns `ObjectStoreError::InvalidPayload` when chunks are empty, contain
    /// non-terminal empty chunks, exceed `MAX_PAYLOAD_CHUNK_BYTES`, or overflow
    /// size accounting.
    pub fn from_chunks(chunks: &[Vec<u8>]) -> Result<Self, ObjectStoreError> {
        if chunks.is_empty() {
            return Err(ObjectStoreError::InvalidPayload);
        }
        if chunks.len() > 1 && chunks.iter().any(Vec::is_empty) {
            return Err(ObjectStoreError::InvalidPayload);
        }
        if chunks
            .iter()
            .any(|chunk| chunk.len() > MAX_PAYLOAD_CHUNK_BYTES)
        {
            return Err(ObjectStoreError::InvalidPayload);
        }

        let mut hasher = blake3::Hasher::new();
        let mut total_size_bytes = 0_u64;
        let mut payload_chunks = Vec::with_capacity(chunks.len());
        for (ordinal, bytes) in chunks.iter().enumerate() {
            let ordinal = u32::try_from(ordinal).map_err(|_| ObjectStoreError::InvalidPayload)?;
            total_size_bytes = total_size_bytes
                .checked_add(bytes.len() as u64)
                .ok_or(ObjectStoreError::InvalidPayload)?;
            hasher.update(bytes);
            payload_chunks.push(CasPayloadChunk {
                ordinal,
                size_bytes: bytes.len() as u64,
                digest: Blake3Digest::for_payload(bytes),
            });
        }

        Ok(Self {
            total_size_bytes,
            root_digest: Blake3Digest(hasher.finalize().to_hex().to_string()),
            chunks: payload_chunks,
        })
    }

    fn validate(&self) -> Result<(), ObjectStoreError> {
        if self.chunks.is_empty() {
            return Err(ObjectStoreError::InvalidPayload);
        }
        if self.chunks.len() > 1 && self.chunks.iter().any(|chunk| chunk.size_bytes == 0) {
            return Err(ObjectStoreError::InvalidPayload);
        }
        if self
            .chunks
            .iter()
            .any(|chunk| chunk.size_bytes > MAX_PAYLOAD_CHUNK_BYTES as u64)
        {
            return Err(ObjectStoreError::InvalidPayload);
        }

        let mut total_size_bytes = 0_u64;
        for (expected_ordinal, chunk) in self.chunks.iter().enumerate() {
            let expected_ordinal =
                u32::try_from(expected_ordinal).map_err(|_| ObjectStoreError::InvalidPayload)?;
            if chunk.ordinal != expected_ordinal {
                return Err(ObjectStoreError::InvalidPayload);
            }
            total_size_bytes = total_size_bytes
                .checked_add(chunk.size_bytes)
                .ok_or(ObjectStoreError::InvalidPayload)?;
        }
        if total_size_bytes != self.total_size_bytes {
            return Err(ObjectStoreError::InvalidPayload);
        }
        Ok(())
    }
}

/// Streaming reader for CAS payload bytes. Adapters pull bounded chunks from
/// this port instead of receiving a whole object buffer. Each returned chunk
/// MUST be no larger than `MAX_PAYLOAD_CHUNK_BYTES`; resumable/network backpressure
/// belongs in adapter-local transport, not the stable CAS manifest.
pub trait CasPayloadReader {
    fn read_next_chunk(&mut self) -> Result<Option<Vec<u8>>, ObjectStoreError>;
}

/// Streaming sink for CAS payload bytes. Adapters push bounded chunks into this
/// port instead of returning a whole object buffer.
pub trait CasPayloadSink {
    fn write_chunk(&mut self, chunk: &[u8]) -> Result<(), ObjectStoreError>;
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMemoryPayloadReader {
    chunks: Vec<Vec<u8>>,
    next_chunk_index: usize,
}

impl InMemoryPayloadReader {
    #[must_use]
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            chunks: vec![bytes],
            next_chunk_index: 0,
        }
    }

    /// Build a reference reader from chunks.
    ///
    /// # Errors
    /// Returns `ObjectStoreError::InvalidPayload` when the chunks do not form a
    /// valid CAS payload manifest.
    pub fn from_chunks(chunks: Vec<Vec<u8>>) -> Result<Self, ObjectStoreError> {
        CasPayload::from_chunks(&chunks)?;
        Ok(Self {
            chunks,
            next_chunk_index: 0,
        })
    }

    /// Return the manifest represented by this reader.
    ///
    /// # Errors
    /// Returns `ObjectStoreError::InvalidPayload` when the chunks do not form a
    /// valid CAS payload manifest.
    pub fn payload(&self) -> Result<CasPayload, ObjectStoreError> {
        CasPayload::from_chunks(&self.chunks)
    }
}

impl CasPayloadReader for InMemoryPayloadReader {
    fn read_next_chunk(&mut self) -> Result<Option<Vec<u8>>, ObjectStoreError> {
        let Some(chunk) = self.chunks.get(self.next_chunk_index) else {
            return Ok(None);
        };
        self.next_chunk_index += 1;
        Ok(Some(chunk.clone()))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMemoryPayloadSink {
    chunks: Vec<Vec<u8>>,
}

impl InMemoryPayloadSink {
    /// Return the payload manifest written to the sink.
    ///
    /// # Errors
    /// Returns `ObjectStoreError::InvalidPayload` when the chunks do not form a
    /// valid CAS payload manifest.
    pub fn payload(&self) -> Result<CasPayload, ObjectStoreError> {
        CasPayload::from_chunks(&self.chunks)
    }

    #[must_use]
    pub fn to_bytes_for_reference(&self) -> Vec<u8> {
        let total_size = self.chunks.iter().map(Vec::len).sum();
        let mut bytes = Vec::with_capacity(total_size);
        for chunk in &self.chunks {
            bytes.extend_from_slice(chunk);
        }
        bytes
    }
}

impl CasPayloadSink for InMemoryPayloadSink {
    fn write_chunk(&mut self, chunk: &[u8]) -> Result<(), ObjectStoreError> {
        if chunk.len() > MAX_PAYLOAD_CHUNK_BYTES {
            return Err(ObjectStoreError::InvalidPayload);
        }
        self.chunks.push(chunk.to_vec());
        Ok(())
    }
}

// =====================================================================
// Request/response types
// =====================================================================

/// CAS write request. The caller supplies a chunked payload and the kernel
/// verifies that the supplied tenant-scoped address is the BLAKE3 root digest of
/// those bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CasPutRequest {
    pub address: TenantScopedBlake3Address, // data_class: INTERNAL_ONLY
    pub payload: CasPayload,                // data_class: INTERNAL_ONLY
    pub kms_boundary: TenantKekBoundary,    // data_class: INTERNAL_ONLY
    pub worm_policy: CasWormPolicy,         // data_class: INTERNAL_ONLY
    pub audit_anchor: CasAuditAnchor,       // data_class: INTERNAL_ONLY
    pub durability: CasDurabilityPolicy,    // data_class: PUBLIC
    pub user_metadata: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64,    // data_class: INTERNAL_ONLY
}

impl CasPutRequest {
    /// Build a write request from a single in-memory buffer and compute the
    /// tenant-scoped BLAKE3 address.
    ///
    /// # Errors
    /// Returns validation errors from payload, WORM, audit, or KEK policy
    /// objects.
    pub fn new(
        tenant_id: TenantId,
        bytes: Vec<u8>,
        kms_boundary: TenantKekBoundary,
        worm_policy: CasWormPolicy,
        audit_anchor: CasAuditAnchor,
        requested_at_epoch_seconds: u64,
    ) -> Result<Self, ObjectStoreError> {
        let payload = CasPayload::from_bytes(&bytes)?;
        Self::new_with_payload(
            tenant_id,
            payload,
            kms_boundary,
            worm_policy,
            audit_anchor,
            requested_at_epoch_seconds,
        )
    }

    /// Build a write request from a chunked payload.
    ///
    /// # Errors
    /// Returns validation errors from payload, WORM, audit, or KEK policy
    /// objects.
    pub fn new_with_payload(
        tenant_id: TenantId,
        payload: CasPayload,
        kms_boundary: TenantKekBoundary,
        worm_policy: CasWormPolicy,
        audit_anchor: CasAuditAnchor,
        requested_at_epoch_seconds: u64,
    ) -> Result<Self, ObjectStoreError> {
        let address = TenantScopedBlake3Address::for_digest(tenant_id, payload.root_digest.clone());
        let request = Self {
            address,
            payload,
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
        self.payload.validate()?;
        self.kms_boundary.validate()?;
        if self.kms_boundary.tenant_id != self.address.tenant_id {
            return Err(ObjectStoreError::CrossTenantAccessDenied);
        }
        self.worm_policy.validate()?;
        if !self
            .worm_policy
            .write_protected_after(self.requested_at_epoch_seconds)
        {
            return Err(ObjectStoreError::ExpiredWormPolicy {
                retain_until_epoch_seconds: self.worm_policy.retain_until_epoch_seconds,
                requested_at_epoch_seconds: self.requested_at_epoch_seconds,
            });
        }
        self.audit_anchor.validate()?;
        if self.payload.root_digest != self.address.digest {
            return Err(ObjectStoreError::AddressDigestMismatch {
                expected: self.address.digest.as_str().to_string(),
                actual: self.payload.root_digest.as_str().to_string(),
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
}

/// Errors emitted by the object-store kernel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObjectStoreError {
    InvalidTenantId,
    InvalidBlake3Digest,
    InvalidPayload,
    AddressDigestMismatch {
        expected: String,
        actual: String,
    },
    CrossTenantAccessDenied,
    InvalidKekBoundary,
    InvalidWormPolicy,
    ExpiredWormPolicy {
        retain_until_epoch_seconds: u64,
        requested_at_epoch_seconds: u64,
    },
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
    DuplicateCasWriteConflict {
        tenant_id: String,
        digest: String,
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
            Self::InvalidPayload => write!(f, "invalid CAS payload"),
            Self::AddressDigestMismatch { expected, actual } => write!(
                f,
                "CAS address digest mismatch: expected {expected}, computed {actual}"
            ),
            Self::CrossTenantAccessDenied => write!(f, "cross-tenant CAS access denied"),
            Self::InvalidKekBoundary => write!(f, "invalid tenant KEK boundary"),
            Self::InvalidWormPolicy => write!(f, "invalid WORM policy"),
            Self::ExpiredWormPolicy {
                retain_until_epoch_seconds,
                requested_at_epoch_seconds,
            } => write!(
                f,
                "expired WORM policy: retain_until={retain_until_epoch_seconds} requested_at={requested_at_epoch_seconds}"
            ),
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
            Self::DuplicateCasWriteConflict { tenant_id, digest } => write!(
                f,
                "duplicate CAS write conflicts with existing metadata: tenant={tenant_id} blake3={digest}"
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
/// Every implementation — transitional adapters and the future owned object
/// store — implements this trait. The trait shape is the destination CAS
/// contract, not a vendor bucket/key API mirror. A successful `put_cas` MUST
/// make the address immediately visible to `head_cas` and `get_cas`; adapters
/// prove that invariant through `run_object_store_conformance_suite`.
pub trait ObjectStore: Send + Sync {
    fn put_cas(
        &self,
        request: CasPutRequest,
        payload: &mut dyn CasPayloadReader,
    ) -> Result<CasObjectRecord, ObjectStoreError>;

    fn head_cas(&self, request: CasReadRequest) -> Result<CasObjectRecord, ObjectStoreError>;

    fn get_cas(
        &self,
        request: CasReadRequest,
        sink: &mut dyn CasPayloadSink,
    ) -> Result<CasObjectRecord, ObjectStoreError>;

    fn delete_cas(&self, request: CasDeleteRequest) -> Result<(), ObjectStoreError>;
}

/// Optional diagnostics for adapter evidence. Application code depends on
/// `ObjectStore`; transitional adapter receipts stay on this separate plane.
pub trait ObjectStoreDiagnostics {
    fn backend_kind(&self) -> ObjectStoreBackendKind;

    fn adapter_boundary(
        &self,
        request: CasReadRequest,
    ) -> Result<Option<TransitionalAdapterBoundary>, ObjectStoreError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectStoreConformanceReport {
    pub checks: Vec<&'static str>, // data_class: PUBLIC
}

/// Reusable conformance suite for adapter crates. It intentionally exercises
/// only the stable CAS contract: tenant-scoped addressing, immediate
/// read-after-write visibility, cross-tenant denial, WORM delete refusal, and
/// same-payload cross-tenant isolation.
///
/// # Errors
/// Returns the first failed object-store operation or a `BackendUnavailable`
/// detail when an adapter violates a post-condition.
pub fn run_object_store_conformance_suite(
    store: &dyn ObjectStore,
) -> Result<ObjectStoreConformanceReport, ObjectStoreError> {
    let tenant_id = TenantId::parse("ten_conformance")?;
    let chunks = vec![b"object-store ".to_vec(), b"conformance payload".to_vec()];
    let payload = CasPayload::from_chunks(&chunks)?;
    let request = CasPutRequest::new_with_payload(
        tenant_id.clone(),
        payload.clone(),
        TenantKekBoundary::new(
            tenant_id.clone(),
            "kms/ten_conformance/object-store",
            1,
            "ct/ten_conformance/object-store",
            Some("shred/ten_conformance/object-store".to_string()),
        )?,
        CasWormPolicy::compliance_until(1_800_000_000, false),
        CasAuditAnchor::new(
            "audit_evt_object_store_conformance",
            Blake3Digest::for_payload(b"object-store-conformance-chain"),
            1_700_000_001,
        )?,
        1_700_000_010,
    )?;
    let address = request.address.clone();
    let mut reader = InMemoryPayloadReader::from_chunks(chunks.clone())?;
    let record = store.put_cas(request, &mut reader)?;
    let read_request = CasReadRequest::new(tenant_id.clone(), address.clone());

    let head = store.head_cas(read_request.clone())?;
    if head != record {
        return Err(ObjectStoreError::BackendUnavailable {
            detail: "head_cas did not immediately observe put_cas record".to_string(),
        });
    }

    let mut sink = InMemoryPayloadSink::default();
    let get_record = store.get_cas(read_request.clone(), &mut sink)?;
    if get_record != record || sink.payload()? != payload {
        return Err(ObjectStoreError::BackendUnavailable {
            detail: "get_cas did not immediately observe put_cas payload".to_string(),
        });
    }

    let mut cross_tenant_sink = InMemoryPayloadSink::default();
    match store.get_cas(
        CasReadRequest::new(TenantId::parse("ten_conformance_other")?, address.clone()),
        &mut cross_tenant_sink,
    ) {
        Err(ObjectStoreError::CrossTenantAccessDenied) => {}
        Ok(_) => {
            return Err(ObjectStoreError::BackendUnavailable {
                detail: "cross-tenant read was not denied".to_string(),
            });
        }
        Err(error) => return Err(error),
    }

    match store.delete_cas(CasDeleteRequest::new(
        tenant_id.clone(),
        address.clone(),
        1_799_999_999,
        "audit_evt_object_store_conformance_delete",
    )) {
        Err(ObjectStoreError::WormRetentionActive { .. }) => {}
        Ok(()) => {
            return Err(ObjectStoreError::BackendUnavailable {
                detail: "WORM-protected delete was not refused".to_string(),
            });
        }
        Err(error) => return Err(error),
    }

    let other_tenant = TenantId::parse("ten_conformance_other")?;
    let other_request = CasPutRequest::new_with_payload(
        other_tenant.clone(),
        payload.clone(),
        TenantKekBoundary::new(
            other_tenant.clone(),
            "kms/ten_conformance_other/object-store",
            1,
            "ct/ten_conformance_other/object-store",
            Some("shred/ten_conformance_other/object-store".to_string()),
        )?,
        CasWormPolicy::compliance_until(1_800_000_000, false),
        CasAuditAnchor::new(
            "audit_evt_object_store_conformance_other",
            Blake3Digest::for_payload(b"object-store-conformance-chain-other"),
            1_700_000_002,
        )?,
        1_700_000_011,
    )?;
    let other_address = other_request.address.clone();
    let mut other_reader = InMemoryPayloadReader::from_chunks(chunks)?;
    let other_record = store.put_cas(other_request, &mut other_reader)?;
    if other_record.address == record.address || other_address.tenant_id == address.tenant_id {
        return Err(ObjectStoreError::BackendUnavailable {
            detail: "same payload across tenants was not isolated".to_string(),
        });
    }

    Ok(ObjectStoreConformanceReport {
        checks: vec![
            "put_immediate_head_visibility",
            "put_immediate_get_visibility",
            "tenant_isolation",
            "worm_delete_refusal",
            "same_payload_cross_tenant_isolation",
        ],
    })
}

// =====================================================================
// Reference in-memory adapter
// =====================================================================
//
// tests exercise the owned CAS port without standing up a transitional bridge.
// It also has an explicit transitional-adapter mode so adapter receipts can be
// tested without leaking vendor bucket/key APIs into the trait.

#[derive(Clone, Debug, Eq, PartialEq)]
struct StoredCasObject {
    chunks: Vec<Vec<u8>>,
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
    transitional_adapter: Option<(TransitionalAdapterClass, String)>,
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
            transitional_adapter: None,
        }
    }

    /// Build an in-memory store that emits transitional adapter boundaries for
    /// tests. This does not change the owned CAS trait surface.
    ///
    /// # Errors
    /// Returns `ObjectStoreError::InvalidTransitionalBoundary` when the adapter
    /// identity is malformed.
    pub fn with_transitional_adapter(
        adapter_class: TransitionalAdapterClass,
        adapter_id: impl Into<String>,
    ) -> Result<Self, ObjectStoreError> {
        let adapter_id = adapter_id.into();
        if !is_valid_reference(&adapter_id) {
            return Err(ObjectStoreError::InvalidTransitionalBoundary);
        }
        Ok(Self {
            inner: Mutex::new(InMemoryStorage::default()),
            backend_kind: ObjectStoreBackendKind::TransitionalAdapter,
            transitional_adapter: Some((adapter_class, adapter_id)),
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
        let Some((adapter_class, adapter_id)) = &self.transitional_adapter else {
            return Ok(None);
        };
        TransitionalAdapterBoundary::new(
            *adapter_class,
            adapter_id.clone(),
            format!(
                "adapter/{}/{}/{}",
                adapter_class.label(),
                adapter_id,
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
    fn put_cas(
        &self,
        request: CasPutRequest,
        payload_reader: &mut dyn CasPayloadReader,
    ) -> Result<CasObjectRecord, ObjectStoreError> {
        request.validate()?;
        let chunks = read_payload_chunks(payload_reader)?;
        let observed_payload = CasPayload::from_chunks(&chunks)?;
        if observed_payload != request.payload {
            return Err(ObjectStoreError::AddressDigestMismatch {
                expected: request.payload.root_digest.as_str().to_string(),
                actual: observed_payload.root_digest.as_str().to_string(),
            });
        }

        let mut store = self.lock();
        if let Some(existing) = store.objects.get(&request.address) {
            if CasPayload::from_chunks(&existing.chunks)? == request.payload {
                if stored_record_matches_put(&existing.record, &request) {
                    return Ok(existing.record.clone());
                }
                return Err(ObjectStoreError::DuplicateCasWriteConflict {
                    tenant_id: request.address.tenant_id.as_str().to_string(),
                    digest: request.address.digest.as_str().to_string(),
                });
            }
            return Err(ObjectStoreError::DuplicateCasWriteConflict {
                tenant_id: request.address.tenant_id.as_str().to_string(),
                digest: request.address.digest.as_str().to_string(),
            });
        }

        let record = CasObjectRecord {
            address: request.address.clone(),
            size_bytes: request.payload.total_size_bytes,
            kms_boundary: request.kms_boundary,
            worm_policy: request.worm_policy,
            audit_anchor: request.audit_anchor,
            durability: request.durability,
            user_metadata: request.user_metadata,
            stored_at_epoch_seconds: request.requested_at_epoch_seconds,
        };

        store.objects.insert(
            record.address.clone(),
            StoredCasObject {
                chunks,
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

    fn get_cas(
        &self,
        request: CasReadRequest,
        sink: &mut dyn CasPayloadSink,
    ) -> Result<CasObjectRecord, ObjectStoreError> {
        request.validate()?;
        let store = self.lock();
        let stored = store
            .objects
            .get(&request.address)
            .ok_or_else(|| Self::not_found(&request.address))?
            .clone();
        drop(store);
        for chunk in &stored.chunks {
            sink.write_chunk(chunk)?;
        }
        Ok(stored.record)
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
}

impl ObjectStoreDiagnostics for InMemoryObjectStore {
    fn backend_kind(&self) -> ObjectStoreBackendKind {
        self.backend_kind
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
        self.transitional_boundary_for(&stored.record.address)
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

fn read_payload_chunks(
    payload_reader: &mut dyn CasPayloadReader,
) -> Result<Vec<Vec<u8>>, ObjectStoreError> {
    let mut chunks = Vec::new();
    while let Some(chunk) = payload_reader.read_next_chunk()? {
        if chunk.len() > MAX_PAYLOAD_CHUNK_BYTES {
            return Err(ObjectStoreError::InvalidPayload);
        }
        chunks.push(chunk);
    }
    CasPayload::from_chunks(&chunks)?;
    Ok(chunks)
}

fn stored_record_matches_put(record: &CasObjectRecord, request: &CasPutRequest) -> bool {
    record.address == request.address
        && record.size_bytes == request.payload.total_size_bytes
        && record.kms_boundary == request.kms_boundary
        && record.worm_policy == request.worm_policy
        && record.audit_anchor == request.audit_anchor
        && record.durability == request.durability
        && record.user_metadata == request.user_metadata
}

fn is_valid_reference(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && value == trimmed
        && value.len() <= MAX_REFERENCE_LEN
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

    fn put_request(
        tenant_id: &TenantId,
        bytes: &[u8],
        retain_until: u64,
    ) -> (CasPutRequest, InMemoryPayloadReader) {
        let bytes = bytes.to_vec();
        let request = CasPutRequest::new(
            tenant_id.clone(),
            bytes.clone(),
            kek_boundary(tenant_id),
            CasWormPolicy::compliance_until(retain_until, false),
            audit_anchor(),
            1_700_000_010,
        )
        .unwrap();
        (request, InMemoryPayloadReader::from_bytes(bytes))
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
        let (request, mut reader) = put_request(&tenant_id, bytes, 1_800_000_000);
        let address = request.address.clone();
        let store = InMemoryObjectStore::new();

        let put_record = store.put_cas(request, &mut reader).unwrap();
        assert_eq!(put_record.address, address);
        assert_eq!(put_record.size_bytes, bytes.len() as u64);
        assert_eq!(put_record.worm_policy.mode, CasWormMode::Compliance);
        assert_eq!(
            put_record.audit_anchor.audit_event_id,
            "audit_evt_object_store_001"
        );
        assert_eq!(put_record.durability, CasDurabilityPolicy::default());
        let read = CasReadRequest::new(tenant_id.clone(), address.clone());
        assert_eq!(store.head_cas(read.clone()).unwrap(), put_record);
        let mut sink = InMemoryPayloadSink::default();
        let get_record = store.get_cas(read, &mut sink).unwrap();
        assert_eq!(get_record.address, address);
        assert_eq!(sink.to_bytes_for_reference(), bytes);
    }

    #[test]
    fn duplicate_identical_cas_put_is_idempotent() {
        let tenant_id = tenant("ten_alpha");
        let (request, mut reader) = put_request(&tenant_id, b"idempotent payload", 1_800_000_000);
        let replay = request.clone();
        let mut replay_reader = reader.clone();
        let store = InMemoryObjectStore::new();

        let first = store.put_cas(request, &mut reader).unwrap();
        let second = store.put_cas(replay, &mut replay_reader).unwrap();

        assert_eq!(second, first);
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn duplicate_same_bytes_with_different_worm_policy_is_conflict() {
        let tenant_id = tenant("ten_alpha");
        let bytes = b"same bytes stronger retention";
        let (first, mut first_reader) = put_request(&tenant_id, bytes, 1_800_000_000);
        let address = first.address.clone();
        let (stronger, mut stronger_reader) = put_request(&tenant_id, bytes, 1_900_000_000);
        let store = InMemoryObjectStore::new();

        store.put_cas(first, &mut first_reader).unwrap();
        let error = store.put_cas(stronger, &mut stronger_reader).unwrap_err();

        assert_eq!(
            error,
            ObjectStoreError::DuplicateCasWriteConflict {
                tenant_id: "ten_alpha".to_string(),
                digest: address.digest.as_str().to_string(),
            }
        );
        assert_eq!(
            store
                .head_cas(CasReadRequest::new(tenant_id, address))
                .unwrap()
                .worm_policy
                .retain_until_epoch_seconds,
            1_800_000_000
        );
    }

    #[test]
    fn cross_tenant_reads_are_denied_even_when_digest_is_known() {
        let tenant_alpha = tenant("ten_alpha");
        let tenant_beta = tenant("ten_beta");
        let (request, mut reader) = put_request(&tenant_alpha, b"same bytes", 1_800_000_000);
        let address = request.address.clone();
        let store = InMemoryObjectStore::new();
        store.put_cas(request, &mut reader).unwrap();

        let mut sink = InMemoryPayloadSink::default();
        let error = store
            .get_cas(CasReadRequest::new(tenant_beta, address), &mut sink)
            .unwrap_err();
        assert_eq!(error, ObjectStoreError::CrossTenantAccessDenied);
    }

    #[test]
    fn same_digest_in_different_tenants_is_stored_as_separate_cas_objects() {
        let payload = b"identical object payload";
        let alpha = tenant("ten_alpha");
        let beta = tenant("ten_beta");
        let (alpha_request, mut alpha_reader) = put_request(&alpha, payload, 1_800_000_000);
        let (beta_request, mut beta_reader) = put_request(&beta, payload, 1_800_000_000);
        assert_eq!(alpha_request.address.digest, beta_request.address.digest);
        assert_ne!(alpha_request.address, beta_request.address);

        let store = InMemoryObjectStore::new();
        let alpha_record = store.put_cas(alpha_request, &mut alpha_reader).unwrap();
        let beta_record = store.put_cas(beta_request, &mut beta_reader).unwrap();

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
            payload: CasPayload::from_bytes(b"payload with different digest").unwrap(),
            kms_boundary: kek_boundary(&tenant_id),
            worm_policy: CasWormPolicy::compliance_until(1_800_000_000, false),
            audit_anchor: audit_anchor(),
            durability: CasDurabilityPolicy::default(),
            user_metadata: BTreeMap::new(),
            requested_at_epoch_seconds: 1_700_000_010,
        };
        let mut reader =
            InMemoryPayloadReader::from_bytes(b"payload with different digest".to_vec());
        let error = InMemoryObjectStore::new()
            .put_cas(request, &mut reader)
            .unwrap_err();
        assert!(matches!(
            error,
            ObjectStoreError::AddressDigestMismatch { .. }
        ));
    }

    #[test]
    fn chunked_payload_preserves_root_digest_without_trait_buffer_requirement() {
        let tenant_id = tenant("ten_alpha");
        let chunks = vec![b"chunk-a".to_vec(), b"chunk-b".to_vec()];
        let payload = CasPayload::from_chunks(&chunks).unwrap();
        let request = CasPutRequest::new_with_payload(
            tenant_id.clone(),
            payload.clone(),
            kek_boundary(&tenant_id),
            CasWormPolicy::compliance_until(1_800_000_000, false),
            audit_anchor(),
            1_700_000_010,
        )
        .unwrap();

        assert_eq!(
            request.address.digest,
            Blake3Digest::for_payload(b"chunk-achunk-b")
        );
        assert_eq!(request.payload, payload);
    }

    #[test]
    fn reference_validation_rejects_trim_mismatched_values() {
        let tenant_id = tenant("ten_alpha");
        assert_eq!(
            TenantKekBoundary::new(
                tenant_id.clone(),
                " kms/ten_alpha/object-store",
                1,
                "ct/ten_alpha/object-store",
                None,
            )
            .unwrap_err(),
            ObjectStoreError::InvalidKekBoundary
        );

        let (mut request, _) = put_request(&tenant_id, b"metadata payload", 1_800_000_000);
        request
            .user_metadata
            .insert(" object-store-key".to_string(), "ok".to_string());
        assert_eq!(
            request.validate().unwrap_err(),
            ObjectStoreError::InvalidUserMetadata
        );

        let delete = CasDeleteRequest::new(
            tenant_id.clone(),
            TenantScopedBlake3Address::for_payload(tenant_id, b"metadata payload"),
            1_800_000_001,
            " audit_evt_delete_with_space",
        );
        assert_eq!(
            delete.validate().unwrap_err(),
            ObjectStoreError::InvalidDeleteRequest
        );
    }

    #[test]
    fn put_rejects_worm_policy_already_expired_at_write_time() {
        let tenant_id = tenant("ten_alpha");
        let error = CasPutRequest::new(
            tenant_id.clone(),
            b"expired worm".to_vec(),
            kek_boundary(&tenant_id),
            CasWormPolicy::compliance_until(1_700_000_009, false),
            audit_anchor(),
            1_700_000_010,
        )
        .unwrap_err();

        assert_eq!(
            error,
            ObjectStoreError::ExpiredWormPolicy {
                retain_until_epoch_seconds: 1_700_000_009,
                requested_at_epoch_seconds: 1_700_000_010,
            }
        );
    }

    #[test]
    fn worm_retention_blocks_delete_until_retention_expires() {
        let tenant_id = tenant("ten_alpha");
        let (request, mut reader) = put_request(&tenant_id, b"worm payload", 1_800_000_000);
        let address = request.address.clone();
        let store = InMemoryObjectStore::new();
        store.put_cas(request, &mut reader).unwrap();

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
    fn conformance_suite_proves_reference_adapter_contract() {
        let store = InMemoryObjectStore::new();

        let report = run_object_store_conformance_suite(&store).unwrap();

        assert_eq!(
            report.checks,
            vec![
                "put_immediate_head_visibility",
                "put_immediate_get_visibility",
                "tenant_isolation",
                "worm_delete_refusal",
                "same_payload_cross_tenant_isolation",
            ]
        );
    }

    #[test]
    fn transitional_adapter_boundary_is_diagnostic_not_bridge_shaped() {
        let tenant_id = tenant("ten_alpha");
        let (request, mut reader) = put_request(&tenant_id, b"transitional payload", 1_800_000_000);
        let address = request.address.clone();
        let store = InMemoryObjectStore::with_transitional_adapter(
            TransitionalAdapterClass::ProtocolCompatible,
            "object-protocol-bridge",
        )
        .unwrap();

        let record = store.put_cas(request, &mut reader).unwrap();
        assert_eq!(record.address, address);
        assert_eq!(
            ObjectStoreDiagnostics::backend_kind(&store),
            ObjectStoreBackendKind::TransitionalAdapter
        );
        let boundary = ObjectStoreDiagnostics::adapter_boundary(
            &store,
            CasReadRequest::new(tenant_id, address),
        )
        .unwrap()
        .expect("transitional boundary is present");
        assert_eq!(
            boundary.adapter_class,
            TransitionalAdapterClass::ProtocolCompatible
        );
        assert_eq!(boundary.adapter_id, "object-protocol-bridge");
        assert!(
            boundary
                .adapter_namespace
                .starts_with("adapter/protocol_compatible/object-protocol-bridge/")
        );
        assert!(boundary.adapter_object_ref.starts_with("cas/ten_alpha/"));
        assert!(
            boundary
                .adapter_evidence_ref
                .starts_with("evidence/object-store/ten_alpha/")
        );
    }
}
