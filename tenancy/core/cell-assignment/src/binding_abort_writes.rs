use cell_placement::{CellProofConsumptionV1, SignedBindingOutcomeV1};

use crate::{
    BindingAuditRecordV1, BindingIdempotencyRecordV1, BindingOperationRevision, BindingOperationV1,
    BindingOutcomePreconditionV1, BindingPersistenceAuthorityV1, BindingProofConsumptionV1,
    BindingReservationAttemptRevision, BindingReservationAttemptV1, BindingStoreError,
};

#[derive(Debug, Eq, PartialEq)]
pub struct BindingAbortWriteSetV1 {
    authority: BindingPersistenceAuthorityV1,
    outcome_precondition: BindingOutcomePreconditionV1,
    no_active_migration_claim: crate::NoActiveMigrationFenceClaimPreconditionV1,
    expected_attempt_revision: BindingReservationAttemptRevision,
    reservation_attempt: BindingReservationAttemptV1,
    expected_operation_revision: BindingOperationRevision,
    operation: BindingOperationV1,
    idempotency: BindingIdempotencyRecordV1,
    audit_outbox: BindingAuditRecordV1,
    reservation_outcome: SignedBindingOutcomeV1,
    cell_proof_consumptions: Vec<CellProofConsumptionV1>,
    binding_proof_consumptions: Vec<BindingProofConsumptionV1>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct BindingAbortWriteSetPartsV1 {
    pub authority: BindingPersistenceAuthorityV1,
    pub outcome_precondition: BindingOutcomePreconditionV1,
    pub no_active_migration_claim: crate::NoActiveMigrationFenceClaimPreconditionV1,
    pub expected_attempt_revision: BindingReservationAttemptRevision,
    pub reservation_attempt: BindingReservationAttemptV1,
    pub expected_operation_revision: BindingOperationRevision,
    pub operation: BindingOperationV1,
    pub idempotency: BindingIdempotencyRecordV1,
    pub audit_outbox: BindingAuditRecordV1,
    pub reservation_outcome: SignedBindingOutcomeV1,
    pub cell_proof_consumptions: Vec<CellProofConsumptionV1>,
    pub binding_proof_consumptions: Vec<BindingProofConsumptionV1>,
}

impl BindingAbortWriteSetV1 {
    pub fn assemble(_parts: BindingAbortWriteSetPartsV1) -> Result<Self, BindingStoreError> {
        Err(BindingStoreError::NotImplemented)
    }

    #[must_use]
    pub fn authority(&self) -> &BindingPersistenceAuthorityV1 {
        &self.authority
    }

    #[must_use]
    pub fn outcome_precondition(&self) -> &BindingOutcomePreconditionV1 {
        &self.outcome_precondition
    }

    #[must_use]
    pub fn no_active_migration_claim(&self) -> &crate::NoActiveMigrationFenceClaimPreconditionV1 {
        &self.no_active_migration_claim
    }

    #[must_use]
    pub fn expected_attempt_revision(&self) -> BindingReservationAttemptRevision {
        self.expected_attempt_revision
    }

    #[must_use]
    pub fn reservation_attempt(&self) -> &BindingReservationAttemptV1 {
        &self.reservation_attempt
    }

    #[must_use]
    pub fn expected_operation_revision(&self) -> BindingOperationRevision {
        self.expected_operation_revision
    }

    #[must_use]
    pub fn operation(&self) -> &BindingOperationV1 {
        &self.operation
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
    pub fn reservation_outcome(&self) -> &SignedBindingOutcomeV1 {
        &self.reservation_outcome
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
pub struct BindingAbortTransactionResultV1 {
    pub outcome: SignedBindingOutcomeV1,
    pub reservation_attempt: BindingReservationAttemptV1,
    pub operation: BindingOperationV1,
}
