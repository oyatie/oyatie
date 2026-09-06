use cell_placement::{CellProofConsumptionV1, VerifiedCellMovementPermit};

use crate::{
    BindingAuditRecordV1, BindingDigest32, BindingIdempotencyRecordV1, BindingOperationKey,
    BindingOperationRevision, BindingOperationV1, BindingPersistenceAuthorityV1,
    BindingProofConsumptionV1, BindingProofEnvelopeV1, BindingProofVerificationError,
    BindingProofVerifier, BindingReadAuthorityV1, BindingStoreError, BoxTenancyFuture,
    SignedResidencyTransferAuthorizationSetV1, TenantId, TransferAuthorizationJournalRevision,
    VerifiedResidencyTransferAuthorization, VerifiedResidencyTransferAuthorizationSet,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferEffectManifestPayloadV1 {
    pub schema_version: u32,
    pub operation: BindingOperationKey,
    pub tenant_id: TenantId,
    pub participant_manifest_digest: BindingDigest32,
    pub effect_snapshot: crate::TransferEffectSnapshotV1,
    pub ordered_effect_root_digest: BindingDigest32,
    pub effect_count: u64,
    pub maximum_total_bytes: u64,
    pub maximum_total_cost_microunits: u64,
    pub currency: String,
    pub record_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedTransferEffectManifestV1 {
    pub payload: TransferEffectManifestPayloadV1,
    pub envelope: BindingProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedTransferEffectManifest(SignedTransferEffectManifestV1);

impl VerifiedTransferEffectManifest {
    #[must_use]
    pub fn signed(&self) -> &SignedTransferEffectManifestV1 {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferEffectManifestExpectationV1 {
    pub operation: BindingOperationKey,
    pub tenant_id: TenantId,
    pub participant_manifest_digest: BindingDigest32,
    pub effect_snapshot: crate::TransferEffectSnapshotV1,
    pub expected_producer: crate::BindingProducerId,
    pub expected_audience: crate::BindingProducerId,
    pub maximum_path_depth: u32,
    pub now_unix_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferAuthorizationJournalV1 {
    pub operation: BindingOperationKey,
    pub tenant_id: TenantId,
    pub effect_manifest_digest: BindingDigest32,
    pub expected_effect_root_digest: BindingDigest32,
    pub expected_effect_count: u64,
    pub next_effect_ordinal: u64,
    pub applied_effect_root_digest: BindingDigest32,
    pub applied_authorization_root_digest: BindingDigest32,
    pub authorized_maximum_bytes: u64,
    pub authorized_maximum_cost_microunits: u64,
    pub revision: TransferAuthorizationJournalRevision,
    pub record_digest: BindingDigest32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct PutTransferEffectManifestWriteSetV1 {
    parts: PutTransferEffectManifestWriteSetPartsV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct PutTransferEffectManifestWriteSetPartsV1 {
    pub authority: crate::BindingWorkSnapshotMutationAuthorityV1,
    pub expected_operation_revision: BindingOperationRevision,
    pub operation: BindingOperationV1,
    pub manifest: VerifiedTransferEffectManifest,
    pub effects: crate::TransferEffectSetV1,
    pub published_snapshot: crate::BindingWorkSnapshotProgressV1,
    pub idempotency: BindingIdempotencyRecordV1,
    pub audit_outbox: BindingAuditRecordV1,
    pub proof_consumptions: Vec<BindingProofConsumptionV1>,
}

impl PutTransferEffectManifestWriteSetV1 {
    pub fn assemble(
        _parts: PutTransferEffectManifestWriteSetPartsV1,
    ) -> Result<Self, BindingStoreError> {
        Err(BindingStoreError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &PutTransferEffectManifestWriteSetPartsV1 {
        &self.parts
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct AppendTransferAuthorizationWriteSetV1 {
    parts: AppendTransferAuthorizationWriteSetPartsV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct AppendTransferAuthorizationWriteSetPartsV1 {
    pub authority: BindingPersistenceAuthorityV1,
    pub expected_operation_revision: BindingOperationRevision,
    pub operation: BindingOperationV1,
    pub expected_revision: TransferAuthorizationJournalRevision,
    pub manifest: VerifiedTransferEffectManifest,
    pub movement_permit: VerifiedCellMovementPermit,
    pub authorization: VerifiedResidencyTransferAuthorization,
    pub next_journal: TransferAuthorizationJournalV1,
    pub idempotency: BindingIdempotencyRecordV1,
    pub audit_outbox: BindingAuditRecordV1,
    pub cell_proof_consumptions: Vec<CellProofConsumptionV1>,
    pub binding_proof_consumptions: Vec<BindingProofConsumptionV1>,
}

impl AppendTransferAuthorizationWriteSetV1 {
    pub fn assemble(
        _parts: AppendTransferAuthorizationWriteSetPartsV1,
    ) -> Result<Self, BindingStoreError> {
        Err(BindingStoreError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &AppendTransferAuthorizationWriteSetPartsV1 {
        &self.parts
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct SealTransferAuthorizationSetWriteSetV1 {
    parts: SealTransferAuthorizationSetWriteSetPartsV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SealTransferAuthorizationSetWriteSetPartsV1 {
    pub authority: BindingPersistenceAuthorityV1,
    pub expected_operation_revision: BindingOperationRevision,
    pub operation: BindingOperationV1,
    pub expected_revision: TransferAuthorizationJournalRevision,
    pub manifest: VerifiedTransferEffectManifest,
    pub set: VerifiedResidencyTransferAuthorizationSet,
    pub idempotency: BindingIdempotencyRecordV1,
    pub audit_outbox: BindingAuditRecordV1,
    pub binding_proof_consumptions: Vec<BindingProofConsumptionV1>,
}

impl SealTransferAuthorizationSetWriteSetV1 {
    pub fn assemble(
        _parts: SealTransferAuthorizationSetWriteSetPartsV1,
    ) -> Result<Self, BindingStoreError> {
        Err(BindingStoreError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &SealTransferAuthorizationSetWriteSetPartsV1 {
        &self.parts
    }
}

pub fn verify_transfer_effect_manifest(
    _verifier: &dyn BindingProofVerifier,
    _signed: SignedTransferEffectManifestV1,
    _expectation: &TransferEffectManifestExpectationV1,
) -> Result<VerifiedTransferEffectManifest, BindingProofVerificationError> {
    Err(BindingProofVerificationError::NotImplemented)
}

pub trait TransferAuthorizationStore: Send + Sync {
    fn put_manifest<'a>(
        &'a self,
        write_set: &'a PutTransferEffectManifestWriteSetV1,
    ) -> BoxTenancyFuture<'a, Result<TransferAuthorizationJournalV1, BindingStoreError>>;

    fn append_authorization<'a>(
        &'a self,
        write_set: &'a AppendTransferAuthorizationWriteSetV1,
    ) -> BoxTenancyFuture<'a, Result<TransferAuthorizationJournalV1, BindingStoreError>>;

    fn seal_set<'a>(
        &'a self,
        write_set: &'a SealTransferAuthorizationSetWriteSetV1,
    ) -> BoxTenancyFuture<'a, Result<SignedResidencyTransferAuthorizationSetV1, BindingStoreError>>;

    fn get_journal<'a>(
        &'a self,
        authority: &'a BindingReadAuthorityV1,
        operation: &'a BindingOperationKey,
    ) -> BoxTenancyFuture<'a, Result<Option<TransferAuthorizationJournalV1>, BindingStoreError>>;

    fn read_effect_page<'a>(
        &'a self,
        authority: &'a BindingReadAuthorityV1,
        request: &'a crate::TransferEffectPageRequestV1,
    ) -> BoxTenancyFuture<'a, Result<crate::TransferEffectPageV1, BindingStoreError>>;

    fn read_effect_page_for_reconciliation<'a>(
        &'a self,
        authority: &'a crate::BindingReconciliationReadAuthorityV1,
        reconciliation_lease: &'a crate::BindingReconciliationLeaseV1,
        request: &'a crate::TransferEffectPageRequestV1,
    ) -> BoxTenancyFuture<'a, Result<crate::TransferEffectPageV1, BindingStoreError>>;
}
