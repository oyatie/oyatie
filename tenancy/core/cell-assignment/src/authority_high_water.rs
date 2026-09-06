use crate::{
    BindingDigest32, BindingGeneration, BindingOperationKey, BindingProofConstructionError,
    TenantId, WriteAuthorityEpoch,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TenantWriteAuthorityHighWaterRevision(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantWriteAuthorityHighWaterV1 {
    tenant_id: TenantId,
    revision: TenantWriteAuthorityHighWaterRevision,
    highest_allocated_generation: BindingGeneration,
    highest_allocated_epoch: WriteAuthorityEpoch,
    last_allocation_operation: BindingOperationKey,
    record_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantWriteAuthorityHighWaterPartsV1 {
    pub tenant_id: TenantId,
    pub revision: TenantWriteAuthorityHighWaterRevision,
    pub highest_allocated_generation: BindingGeneration,
    pub highest_allocated_epoch: WriteAuthorityEpoch,
    pub last_allocation_operation: BindingOperationKey,
    pub record_digest: BindingDigest32,
}

impl TenantWriteAuthorityHighWaterV1 {
    pub fn rehydrate(
        _parts: TenantWriteAuthorityHighWaterPartsV1,
    ) -> Result<Self, BindingProofConstructionError> {
        Err(BindingProofConstructionError::NotImplemented)
    }

    #[must_use]
    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    #[must_use]
    pub fn revision(&self) -> TenantWriteAuthorityHighWaterRevision {
        self.revision
    }

    #[must_use]
    pub fn highest_allocated_generation(&self) -> BindingGeneration {
        self.highest_allocated_generation
    }

    #[must_use]
    pub fn highest_allocated_epoch(&self) -> WriteAuthorityEpoch {
        self.highest_allocated_epoch
    }

    #[must_use]
    pub fn last_allocation_operation(&self) -> &BindingOperationKey {
        &self.last_allocation_operation
    }

    #[must_use]
    pub fn record_digest(&self) -> BindingDigest32 {
        self.record_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantWriteAuthorityHighWaterPreconditionV1 {
    pub revision: TenantWriteAuthorityHighWaterRevision,
    pub highest_allocated_generation: BindingGeneration,
    pub highest_allocated_epoch: WriteAuthorityEpoch,
    pub record_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantWriteAuthorityBindingMutationV1 {
    Initialize(TenantWriteAuthorityHighWaterV1),
    Assert(TenantWriteAuthorityHighWaterPreconditionV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantWriteAuthorityAdvanceV1 {
    pub precondition: TenantWriteAuthorityHighWaterPreconditionV1,
    pub next: TenantWriteAuthorityHighWaterV1,
}
