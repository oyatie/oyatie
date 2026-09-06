use cell_placement::{
    CellProofConsumptionV1, SignedBindingParticipantManifestCommitmentV1,
    VerifiedCellPlacementDecision,
};

use crate::{
    BindingAuditRecordV1, BindingIdempotencyRecordV1, BindingOperationKey,
    BindingOperationRevision, BindingOperationV1, BindingPersistenceAuthorityV1,
    BindingProofConsumptionV1, BindingReadAuthorityV1, BindingReconciliationLeaseV1,
    BindingReconciliationReadAuthorityV1, BindingReservationAttemptRevision, BindingStoreError,
    BoxTenancyFuture, ParticipantManifestMemberPageRequestV1, ParticipantManifestMemberPageV1,
    ParticipantManifestMemberSetV1, ParticipantReceiptLedgerRevision, ParticipantReceiptLedgerV1,
    ParticipantReceiptPhaseV1, ParticipantReceiptWorkItemV1, ParticipantReceiptWorkPageRequestV1,
    ParticipantReceiptWorkPageV1, ParticipantReceiptWorkPreconditionV1,
    SignedParticipantManifestV1, SignedParticipantPhaseClosureV1, TenantId,
    VerifiedParticipantManifest, VerifiedParticipantPhaseClosure, VerifiedParticipantReceipt,
};

#[derive(Debug, Eq, PartialEq)]
pub struct PutParticipantManifestWriteSetV1 {
    parts: PutParticipantManifestWriteSetPartsV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct PutParticipantManifestWriteSetPartsV1 {
    pub authority: crate::BindingWorkSnapshotMutationAuthorityV1,
    pub expected_attempt_revision: BindingReservationAttemptRevision,
    pub expected_operation_revision: BindingOperationRevision,
    pub operation: BindingOperationV1,
    pub placement_decision: VerifiedCellPlacementDecision,
    pub manifest: VerifiedParticipantManifest,
    pub members: ParticipantManifestMemberSetV1,
    pub published_snapshot: crate::BindingWorkSnapshotProgressV1,
    pub cell_commitment: SignedBindingParticipantManifestCommitmentV1,
    pub idempotency: BindingIdempotencyRecordV1,
    pub proof_consumptions: Vec<BindingProofConsumptionV1>,
    pub cell_proof_consumptions: Vec<CellProofConsumptionV1>,
    pub audit_outbox: BindingAuditRecordV1,
}

impl PutParticipantManifestWriteSetV1 {
    pub fn assemble(
        _parts: PutParticipantManifestWriteSetPartsV1,
    ) -> Result<Self, BindingStoreError> {
        Err(BindingStoreError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &PutParticipantManifestWriteSetPartsV1 {
        &self.parts
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PutParticipantManifestResultV1 {
    pub manifest: SignedParticipantManifestV1,
    pub cell_commitment: SignedBindingParticipantManifestCommitmentV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct AppendParticipantReceiptWriteSetV1 {
    parts: AppendParticipantReceiptWriteSetPartsV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct AppendParticipantReceiptWriteSetPartsV1 {
    pub authority: BindingPersistenceAuthorityV1,
    pub expected_operation_revision: BindingOperationRevision,
    pub operation: BindingOperationV1,
    pub expected_ledger_revision: ParticipantReceiptLedgerRevision,
    pub receipt: VerifiedParticipantReceipt,
    pub work_precondition: ParticipantReceiptWorkPreconditionV1,
    pub next_work_item: ParticipantReceiptWorkItemV1,
    pub next_ledger: ParticipantReceiptLedgerV1,
    pub idempotency: BindingIdempotencyRecordV1,
    pub proof_consumptions: Vec<BindingProofConsumptionV1>,
    pub audit_outbox: BindingAuditRecordV1,
}

impl AppendParticipantReceiptWriteSetV1 {
    pub fn assemble(
        _parts: AppendParticipantReceiptWriteSetPartsV1,
    ) -> Result<Self, BindingStoreError> {
        Err(BindingStoreError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &AppendParticipantReceiptWriteSetPartsV1 {
        &self.parts
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct CloseParticipantPhaseWriteSetV1 {
    parts: CloseParticipantPhaseWriteSetPartsV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CloseParticipantPhaseWriteSetPartsV1 {
    pub authority: BindingPersistenceAuthorityV1,
    pub expected_operation_revision: BindingOperationRevision,
    pub operation: BindingOperationV1,
    pub expected_ledger_revision: ParticipantReceiptLedgerRevision,
    pub closure: VerifiedParticipantPhaseClosure,
    pub idempotency: BindingIdempotencyRecordV1,
    pub proof_consumptions: Vec<BindingProofConsumptionV1>,
    pub audit_outbox: BindingAuditRecordV1,
}

impl CloseParticipantPhaseWriteSetV1 {
    pub fn assemble(
        _parts: CloseParticipantPhaseWriteSetPartsV1,
    ) -> Result<Self, BindingStoreError> {
        Err(BindingStoreError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &CloseParticipantPhaseWriteSetPartsV1 {
        &self.parts
    }
}

pub trait ParticipantManifestStore: Send + Sync {
    fn put_manifest<'a>(
        &'a self,
        write_set: &'a PutParticipantManifestWriteSetV1,
    ) -> BoxTenancyFuture<'a, Result<PutParticipantManifestResultV1, BindingStoreError>>;

    fn get_manifest<'a>(
        &'a self,
        authority: &'a BindingReadAuthorityV1,
        tenant_id: &'a TenantId,
        operation: &'a BindingOperationKey,
    ) -> BoxTenancyFuture<'a, Result<Option<PutParticipantManifestResultV1>, BindingStoreError>>;

    fn append_receipt<'a>(
        &'a self,
        write_set: &'a AppendParticipantReceiptWriteSetV1,
    ) -> BoxTenancyFuture<'a, Result<ParticipantReceiptLedgerV1, BindingStoreError>>;

    fn close_phase<'a>(
        &'a self,
        write_set: &'a CloseParticipantPhaseWriteSetV1,
    ) -> BoxTenancyFuture<'a, Result<SignedParticipantPhaseClosureV1, BindingStoreError>>;

    fn get_ledger<'a>(
        &'a self,
        authority: &'a BindingReadAuthorityV1,
        operation: &'a BindingOperationKey,
        phase: ParticipantReceiptPhaseV1,
    ) -> BoxTenancyFuture<'a, Result<Option<ParticipantReceiptLedgerV1>, BindingStoreError>>;

    fn read_member_page<'a>(
        &'a self,
        authority: &'a BindingReadAuthorityV1,
        request: &'a ParticipantManifestMemberPageRequestV1,
    ) -> BoxTenancyFuture<'a, Result<ParticipantManifestMemberPageV1, BindingStoreError>>;

    fn read_member_page_for_reconciliation<'a>(
        &'a self,
        authority: &'a BindingReconciliationReadAuthorityV1,
        reconciliation_lease: &'a BindingReconciliationLeaseV1,
        request: &'a ParticipantManifestMemberPageRequestV1,
    ) -> BoxTenancyFuture<'a, Result<ParticipantManifestMemberPageV1, BindingStoreError>>;

    fn read_receipt_work_page_for_reconciliation<'a>(
        &'a self,
        authority: &'a BindingReconciliationReadAuthorityV1,
        reconciliation_lease: &'a BindingReconciliationLeaseV1,
        request: &'a ParticipantReceiptWorkPageRequestV1,
    ) -> BoxTenancyFuture<'a, Result<ParticipantReceiptWorkPageV1, BindingStoreError>>;
}
