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
