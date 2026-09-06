use crate::{
    CellProofConsumptionV1, PlacementAuditRecordV1, PlacementContractError,
    PlacementIdempotencyRecordV1, PlacementOperationPreconditionV1, PlacementOperationRevision,
    PlacementOperationV1, PlacementPersistenceAuthorityV1, VerifiedPlacementRepairAuthority,
};

#[derive(Debug, Eq, PartialEq)]
pub struct PlacementRepairWriteSetV1 {
    authority: PlacementPersistenceAuthorityV1,
    repair_authority: VerifiedPlacementRepairAuthority,
    repair_operation_precondition: PlacementOperationPreconditionV1,
    expected_target_revision: PlacementOperationRevision,
    target_operation: PlacementOperationV1,
    repair_operation: PlacementOperationV1,
    drain_mutations: crate::DrainContributorMutationSetV1,
    idempotency: PlacementIdempotencyRecordV1,
    proof_consumptions: Vec<CellProofConsumptionV1>,
    audit_outbox: PlacementAuditRecordV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct PlacementRepairWriteSetPartsV1 {
    pub authority: PlacementPersistenceAuthorityV1,
    pub repair_authority: VerifiedPlacementRepairAuthority,
    pub repair_operation_precondition: PlacementOperationPreconditionV1,
    pub expected_target_revision: PlacementOperationRevision,
    pub target_operation: PlacementOperationV1,
    pub repair_operation: PlacementOperationV1,
    pub drain_mutations: crate::DrainContributorMutationSetV1,
    pub idempotency: PlacementIdempotencyRecordV1,
    pub proof_consumptions: Vec<CellProofConsumptionV1>,
    pub audit_outbox: PlacementAuditRecordV1,
}

impl PlacementRepairWriteSetV1 {
    pub fn assemble(
        _parts: PlacementRepairWriteSetPartsV1,
    ) -> Result<Self, PlacementContractError> {
        Err(PlacementContractError::NotImplemented)
    }

    #[must_use]
    pub fn authority(&self) -> &PlacementPersistenceAuthorityV1 {
        &self.authority
    }

    #[must_use]
    pub fn repair_authority(&self) -> &VerifiedPlacementRepairAuthority {
        &self.repair_authority
    }

    #[must_use]
    pub fn repair_operation_precondition(&self) -> PlacementOperationPreconditionV1 {
        self.repair_operation_precondition
    }

    #[must_use]
    pub fn expected_target_revision(&self) -> PlacementOperationRevision {
        self.expected_target_revision
    }

    #[must_use]
    pub fn target_operation(&self) -> &PlacementOperationV1 {
        &self.target_operation
    }

    #[must_use]
    pub fn repair_operation(&self) -> &PlacementOperationV1 {
        &self.repair_operation
    }

    #[must_use]
    pub fn drain_mutations(&self) -> &crate::DrainContributorMutationSetV1 {
        &self.drain_mutations
    }

    #[must_use]
    pub fn idempotency(&self) -> &PlacementIdempotencyRecordV1 {
        &self.idempotency
    }

    #[must_use]
    pub fn proof_consumptions(&self) -> &[CellProofConsumptionV1] {
        &self.proof_consumptions
    }

    #[must_use]
    pub fn audit_outbox(&self) -> &PlacementAuditRecordV1 {
        &self.audit_outbox
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct PlacementRepairMutationResultV1 {
    pub target_operation: PlacementOperationV1,
    pub repair_operation: PlacementOperationV1,
}
