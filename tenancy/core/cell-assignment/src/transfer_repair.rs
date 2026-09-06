use cell_placement::{
    CellProofConsumptionV1, SignedMovementBudgetSettlementClaimV1, VerifiedCellMovementPermit,
};

use crate::{
    BindingAuditRecordV1, BindingIdempotencyRecordV1, BindingOperationPreconditionV1,
    BindingOperationV1, BindingPersistenceAuthorityV1, BindingProofConsumptionV1,
    BindingReconciliationPersistenceAuthorityV1, BindingStoreError,
    TransferExecutionLedgerRevision, TransferExecutionLedgerV1, VerifiedBindingRepairAuthority,
};

#[derive(Debug, Eq, PartialEq)]
pub enum TransferExecutionRepairAuthorityV1 {
    Operator(BindingPersistenceAuthorityV1),
    Reconciler(BindingReconciliationPersistenceAuthorityV1),
}

#[derive(Debug, Eq, PartialEq)]
pub struct TransferExecutionRepairWriteSetV1 {
    parts: TransferExecutionRepairWriteSetPartsV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TransferExecutionRepairWriteSetPartsV1 {
    pub authority: TransferExecutionRepairAuthorityV1,
    pub repair_authority: VerifiedBindingRepairAuthority,
    pub target_operation_precondition: BindingOperationPreconditionV1,
    pub repair_operation_precondition: BindingOperationPreconditionV1,
    pub target_operation: BindingOperationV1,
    pub repair_operation: BindingOperationV1,
    pub expected_ledger_revision: TransferExecutionLedgerRevision,
    pub expected_ledger_record_digest: crate::BindingDigest32,
    pub movement_permit: VerifiedCellMovementPermit,
    pub next_ledger: TransferExecutionLedgerV1,
    pub settlement_claim_outbox: SignedMovementBudgetSettlementClaimV1,
    pub idempotency: BindingIdempotencyRecordV1,
    pub audit_outbox: BindingAuditRecordV1,
    pub cell_proof_consumptions: Vec<CellProofConsumptionV1>,
    pub binding_proof_consumptions: Vec<BindingProofConsumptionV1>,
}

impl TransferExecutionRepairWriteSetV1 {
    pub fn assemble(
        _parts: TransferExecutionRepairWriteSetPartsV1,
    ) -> Result<Self, BindingStoreError> {
        Err(BindingStoreError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &TransferExecutionRepairWriteSetPartsV1 {
        &self.parts
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct TransferExecutionRepairMutationResultV1 {
    pub ledger: TransferExecutionLedgerV1,
    pub target_operation: BindingOperationV1,
    pub repair_operation: BindingOperationV1,
    pub settlement_claim: SignedMovementBudgetSettlementClaimV1,
}
