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
