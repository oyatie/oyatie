use cell_placement::CellProofConsumptionV1;

use crate::{
    BindingAuditRecordV1, BindingIdempotencyRecordV1, BindingOperationPreconditionV1,
    BindingOperationRevision, BindingOperationV1, BindingPersistenceAuthorityV1,
    BindingProofConsumptionV1, BindingStoreError, BindingWritePrecondition, MigrationFenceClaimV1,
    TenantWriteAuthorityAdvanceV1, VerifiedBindingRepairAuthority, WriteAuthorityLeaseFreezeV1,
};

#[derive(Debug, Eq, PartialEq)]
pub struct BindingOperationWriteSetV1 {
    authority: BindingPersistenceAuthorityV1,
    precondition: BindingOperationPreconditionV1,
    operation: BindingOperationV1,
    idempotency: BindingIdempotencyRecordV1,
    audit_outbox: BindingAuditRecordV1,
    binding_proof_consumptions: Vec<BindingProofConsumptionV1>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct BindingOperationWriteSetPartsV1 {
    pub authority: BindingPersistenceAuthorityV1,
    pub precondition: BindingOperationPreconditionV1,
    pub operation: BindingOperationV1,
    pub idempotency: BindingIdempotencyRecordV1,
    pub audit_outbox: BindingAuditRecordV1,
    pub binding_proof_consumptions: Vec<BindingProofConsumptionV1>,
}

impl BindingOperationWriteSetV1 {
    pub fn assemble(_parts: BindingOperationWriteSetPartsV1) -> Result<Self, BindingStoreError> {
        Err(BindingStoreError::NotImplemented)
    }

    #[must_use]
    pub fn authority(&self) -> &BindingPersistenceAuthorityV1 {
        &self.authority
    }

    #[must_use]
    pub fn precondition(&self) -> BindingOperationPreconditionV1 {
        self.precondition
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
    pub fn proof_consumptions(&self) -> &[BindingProofConsumptionV1] {
        &self.binding_proof_consumptions
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct BindingMigrationFenceClaimWriteSetV1 {
    authority: BindingPersistenceAuthorityV1,
    binding_precondition: BindingWritePrecondition,
    expected_operation_revision: BindingOperationRevision,
    claim: MigrationFenceClaimV1,
    operation: BindingOperationV1,
    idempotency: BindingIdempotencyRecordV1,
    audit_outbox: BindingAuditRecordV1,
    cell_proof_consumptions: Vec<CellProofConsumptionV1>,
    binding_proof_consumptions: Vec<BindingProofConsumptionV1>,
    authority_high_water_advance: TenantWriteAuthorityAdvanceV1,
    source_write_authority_lease_freeze: WriteAuthorityLeaseFreezeV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct BindingMigrationFenceClaimWriteSetPartsV1 {
    pub authority: BindingPersistenceAuthorityV1,
    pub binding_precondition: BindingWritePrecondition,
    pub expected_operation_revision: BindingOperationRevision,
    pub claim: MigrationFenceClaimV1,
    pub operation: BindingOperationV1,
    pub idempotency: BindingIdempotencyRecordV1,
    pub audit_outbox: BindingAuditRecordV1,
    pub cell_proof_consumptions: Vec<CellProofConsumptionV1>,
    pub binding_proof_consumptions: Vec<BindingProofConsumptionV1>,
    pub authority_high_water_advance: TenantWriteAuthorityAdvanceV1,
    pub source_write_authority_lease_freeze: WriteAuthorityLeaseFreezeV1,
}

impl BindingMigrationFenceClaimWriteSetV1 {
    pub fn assemble(
        _parts: BindingMigrationFenceClaimWriteSetPartsV1,
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
    pub fn expected_operation_revision(&self) -> BindingOperationRevision {
        self.expected_operation_revision
    }

    #[must_use]
    pub fn claim(&self) -> &MigrationFenceClaimV1 {
        &self.claim
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
    pub fn proof_consumptions(&self) -> (&[CellProofConsumptionV1], &[BindingProofConsumptionV1]) {
        (
            &self.cell_proof_consumptions,
            &self.binding_proof_consumptions,
        )
    }

    #[must_use]
    pub fn authority_high_water_advance(&self) -> &TenantWriteAuthorityAdvanceV1 {
        &self.authority_high_water_advance
    }

    #[must_use]
    pub fn source_write_authority_lease_freeze(&self) -> &WriteAuthorityLeaseFreezeV1 {
        &self.source_write_authority_lease_freeze
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct BindingRepairWriteSetV1 {
    authority: BindingPersistenceAuthorityV1,
    repair_authority: VerifiedBindingRepairAuthority,
    repair_operation_precondition: BindingOperationPreconditionV1,
    expected_target_revision: BindingOperationRevision,
    target_operation: BindingOperationV1,
    repair_operation: BindingOperationV1,
    idempotency: BindingIdempotencyRecordV1,
    audit_outbox: BindingAuditRecordV1,
    binding_proof_consumptions: Vec<BindingProofConsumptionV1>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct BindingRepairWriteSetPartsV1 {
    pub authority: BindingPersistenceAuthorityV1,
    pub repair_authority: VerifiedBindingRepairAuthority,
    pub repair_operation_precondition: BindingOperationPreconditionV1,
    pub expected_target_revision: BindingOperationRevision,
    pub target_operation: BindingOperationV1,
    pub repair_operation: BindingOperationV1,
    pub idempotency: BindingIdempotencyRecordV1,
    pub audit_outbox: BindingAuditRecordV1,
    pub binding_proof_consumptions: Vec<BindingProofConsumptionV1>,
}

impl BindingRepairWriteSetV1 {
    pub fn assemble(_parts: BindingRepairWriteSetPartsV1) -> Result<Self, BindingStoreError> {
        Err(BindingStoreError::NotImplemented)
    }

    #[must_use]
    pub fn authority(&self) -> &BindingPersistenceAuthorityV1 {
        &self.authority
    }

    #[must_use]
    pub fn repair_authority(&self) -> &VerifiedBindingRepairAuthority {
        &self.repair_authority
    }

    #[must_use]
    pub fn repair_operation_precondition(&self) -> BindingOperationPreconditionV1 {
        self.repair_operation_precondition
    }

    #[must_use]
    pub fn expected_target_revision(&self) -> BindingOperationRevision {
        self.expected_target_revision
    }

    #[must_use]
    pub fn target_operation(&self) -> &BindingOperationV1 {
        &self.target_operation
    }

    #[must_use]
    pub fn repair_operation(&self) -> &BindingOperationV1 {
        &self.repair_operation
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
    pub fn proof_consumptions(&self) -> &[BindingProofConsumptionV1] {
        &self.binding_proof_consumptions
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct BindingRepairMutationResultV1 {
    pub target_operation: BindingOperationV1,
    pub repair_operation: BindingOperationV1,
}
