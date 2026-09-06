use cell_placement::CellProofConsumptionV1;

use crate::{
    BindingAttemptCheckpointEvidenceV1, BindingAuditRecordV1, BindingIdempotencyRecordV1,
    BindingOperationPreconditionV1, BindingOperationV1, BindingPersistenceAuthorityV1,
    BindingProofConsumptionV1, BindingReservationAttemptPreconditionV1,
    BindingReservationAttemptV1, BindingStoreError, BindingWritePrecondition,
};

#[derive(Debug, Eq, PartialEq)]
pub struct BindingReservationAttemptOpenWriteSetV1 {
    authority: BindingPersistenceAuthorityV1,
    binding_precondition: BindingWritePrecondition,
    operation_precondition: BindingOperationPreconditionV1,
    attempt: BindingReservationAttemptV1,
    operation: BindingOperationV1,
    placement_decision: cell_placement::VerifiedCellPlacementDecision,
    idempotency: BindingIdempotencyRecordV1,
    audit_outbox: BindingAuditRecordV1,
    cell_proof_consumptions: Vec<CellProofConsumptionV1>,
    binding_proof_consumptions: Vec<BindingProofConsumptionV1>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct BindingReservationAttemptOpenWriteSetPartsV1 {
    pub authority: BindingPersistenceAuthorityV1,
    pub binding_precondition: BindingWritePrecondition,
    pub operation_precondition: BindingOperationPreconditionV1,
    pub attempt: BindingReservationAttemptV1,
    pub operation: BindingOperationV1,
    pub placement_decision: cell_placement::VerifiedCellPlacementDecision,
    pub idempotency: BindingIdempotencyRecordV1,
    pub audit_outbox: BindingAuditRecordV1,
    pub cell_proof_consumptions: Vec<CellProofConsumptionV1>,
    pub binding_proof_consumptions: Vec<BindingProofConsumptionV1>,
}

impl BindingReservationAttemptOpenWriteSetV1 {
    pub fn assemble(
        _parts: BindingReservationAttemptOpenWriteSetPartsV1,
    ) -> Result<Self, BindingStoreError> {
        Err(BindingStoreError::NotImplemented)
    }

    #[must_use]
    pub fn authority(&self) -> &BindingPersistenceAuthorityV1 {
        &self.authority
    }

    #[must_use]
    pub fn binding_precondition(&self) -> &BindingWritePrecondition {
        &self.binding_precondition
    }

    #[must_use]
    pub fn operation_precondition(&self) -> BindingOperationPreconditionV1 {
        self.operation_precondition
    }

    #[must_use]
    pub fn attempt(&self) -> &BindingReservationAttemptV1 {
        &self.attempt
    }

    #[must_use]
    pub fn operation(&self) -> &BindingOperationV1 {
        &self.operation
    }

    #[must_use]
    pub fn placement_decision(&self) -> &cell_placement::VerifiedCellPlacementDecision {
        &self.placement_decision
    }

    #[must_use]
    pub fn idempotency(&self) -> &BindingIdempotencyRecordV1 {
        &self.idempotency
    }

    #[must_use]
    pub fn audit_outbox(&self) -> &BindingAuditRecordV1 {
        &self.audit_outbox
    }

    #[must_use]
    pub fn proof_consumptions(&self) -> (&[CellProofConsumptionV1], &[BindingProofConsumptionV1]) {
        (
            &self.cell_proof_consumptions,
            &self.binding_proof_consumptions,
        )
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct BindingReservationAttemptWriteSetV1 {
    authority: BindingPersistenceAuthorityV1,
    precondition: BindingReservationAttemptPreconditionV1,
    operation_precondition: BindingOperationPreconditionV1,
    attempt: BindingReservationAttemptV1,
    operation: BindingOperationV1,
    evidence: BindingAttemptCheckpointEvidenceV1,
    idempotency: BindingIdempotencyRecordV1,
    audit_outbox: BindingAuditRecordV1,
    cell_proof_consumptions: Vec<CellProofConsumptionV1>,
    binding_proof_consumptions: Vec<BindingProofConsumptionV1>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct BindingReservationAttemptWriteSetPartsV1 {
    pub authority: BindingPersistenceAuthorityV1,
    pub precondition: BindingReservationAttemptPreconditionV1,
    pub operation_precondition: BindingOperationPreconditionV1,
    pub attempt: BindingReservationAttemptV1,
    pub operation: BindingOperationV1,
    pub evidence: BindingAttemptCheckpointEvidenceV1,
    pub idempotency: BindingIdempotencyRecordV1,
    pub audit_outbox: BindingAuditRecordV1,
    pub cell_proof_consumptions: Vec<CellProofConsumptionV1>,
    pub binding_proof_consumptions: Vec<BindingProofConsumptionV1>,
}

impl BindingReservationAttemptWriteSetV1 {
    pub fn assemble(
        _parts: BindingReservationAttemptWriteSetPartsV1,
    ) -> Result<Self, BindingStoreError> {
        Err(BindingStoreError::NotImplemented)
    }

    #[must_use]
    pub fn authority(&self) -> &BindingPersistenceAuthorityV1 {
        &self.authority
    }

    #[must_use]
    pub fn precondition(&self) -> BindingReservationAttemptPreconditionV1 {
        self.precondition
    }

    #[must_use]
    pub fn operation_precondition(&self) -> BindingOperationPreconditionV1 {
        self.operation_precondition
    }

    #[must_use]
    pub fn attempt(&self) -> &BindingReservationAttemptV1 {
        &self.attempt
    }

    #[must_use]
    pub fn operation(&self) -> &BindingOperationV1 {
        &self.operation
    }

    #[must_use]
    pub fn evidence(&self) -> &BindingAttemptCheckpointEvidenceV1 {
        &self.evidence
    }

    #[must_use]
    pub fn idempotency(&self) -> &BindingIdempotencyRecordV1 {
        &self.idempotency
    }

    #[must_use]
    pub fn audit_outbox(&self) -> &BindingAuditRecordV1 {
        &self.audit_outbox
    }

    #[must_use]
    pub fn proof_consumptions(&self) -> (&[CellProofConsumptionV1], &[BindingProofConsumptionV1]) {
        (
            &self.cell_proof_consumptions,
            &self.binding_proof_consumptions,
        )
    }
}
