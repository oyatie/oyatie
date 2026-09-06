use cell_placement::{
    AssuranceAuditPolicyV1, CellProofConsumptionV1, CurrencyCode, VerifiedCellMovementPermit,
};

use crate::{
    BindingAuditRecordV1, BindingDigest32, BindingIdempotencyRecordV1, BindingOperationKey,
    BindingOperationRevision, BindingOperationV1, BindingPersistenceAuthorityV1,
    BindingProofConsumptionV1, BindingProofEnvelopeV1, BindingProofVerificationError,
    BindingProofVerifier, BindingStoreError, BoxTenancyFuture, CapabilityParticipantId,
    ParticipantReceiptPhaseV1, ResidencyTransferEffectV1, TenantId,
    VerifiedParticipantManifestMember, VerifiedResidencyTransferAuthorization,
    VerifiedResidencyTransferAuthorizationSet,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MovementBudgetPoolV1 {
    Ordinary,
    ForwardCompletionReserve,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferBudgetDebitV1 {
    pub pool: MovementBudgetPoolV1,
    pub maximum_bytes: u64,
    pub effects: u64,
    pub maximum_cost_microunits: u64,
    pub currency: CurrencyCode,
    pub budget_relation_digest: BindingDigest32,
    pub debit_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferExecutionPermitPayloadV1 {
    pub schema_version: u32,
    pub operation: BindingOperationKey,
    pub tenant_id: TenantId,
    pub participant_id: CapabilityParticipantId,
    pub participant_membership_digest: BindingDigest32,
    pub participant_manifest_digest: BindingDigest32,
    pub binding_participant_commitment_digest: BindingDigest32,
    pub effect: ResidencyTransferEffectV1,
    pub effect_fingerprint: BindingDigest32,
    pub transfer_authorization_digest: BindingDigest32,
    pub transfer_authorization_set_digest: BindingDigest32,
    pub movement_permit_digest: BindingDigest32,
    pub scheduling_permit_id: String,
    pub parent_deadline_unix_seconds: u64,
    pub worker_id: String,
    pub worker_lease_epoch: u64,
    pub worker_lease_expires_at_unix_seconds: u64,
    pub phase: ParticipantReceiptPhaseV1,
    pub budget_debit: TransferBudgetDebitV1,
    pub assurance_audit_policy: AssuranceAuditPolicyV1,
    pub not_before_unix_seconds: u64,
    pub expires_at_unix_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedTransferExecutionPermitV1 {
    pub payload: TransferExecutionPermitPayloadV1,
    pub envelope: BindingProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedTransferExecutionPermit(SignedTransferExecutionPermitV1);

impl VerifiedTransferExecutionPermit {
    #[must_use]
    pub fn signed(&self) -> &SignedTransferExecutionPermitV1 {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferExecutionPermitExpectationV1 {
    pub operation: BindingOperationKey,
    pub tenant_id: TenantId,
    pub participant_id: CapabilityParticipantId,
    pub participant_membership_digest: BindingDigest32,
    pub participant_manifest_digest: BindingDigest32,
    pub binding_participant_commitment_digest: BindingDigest32,
    pub effect_fingerprint: BindingDigest32,
    pub transfer_authorization_digest: BindingDigest32,
    pub transfer_authorization_set_digest: BindingDigest32,
    pub movement_permit_digest: BindingDigest32,
    pub scheduling_permit_id: String,
    pub worker_id: String,
    pub worker_lease_epoch: u64,
    pub phase: ParticipantReceiptPhaseV1,
    pub assurance_audit_policy: AssuranceAuditPolicyV1,
    pub now_unix_seconds: u64,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TransferExecutionOutcomeStatusV1 {
    Applied,
    FailedBeforeEffect,
    OutcomeUnknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferExecutionOutcomePayloadV1 {
    pub schema_version: u32,
    pub operation: BindingOperationKey,
    pub tenant_id: TenantId,
    pub participant_id: CapabilityParticipantId,
    pub effect_fingerprint: BindingDigest32,
    pub execution_permit_digest: BindingDigest32,
    pub status: TransferExecutionOutcomeStatusV1,
    pub accounted_bytes: u64,
    pub accounted_cost_microunits: u64,
    pub result_digest: BindingDigest32,
    pub occurred_at_unix_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedTransferExecutionOutcomeV1 {
    pub payload: TransferExecutionOutcomePayloadV1,
    pub envelope: BindingProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedTransferExecutionOutcome(SignedTransferExecutionOutcomeV1);

impl VerifiedTransferExecutionOutcome {
    #[must_use]
    pub fn signed(&self) -> &SignedTransferExecutionOutcomeV1 {
        &self.0
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct TransferExecutionOutcomeExpectationV1 {
    pub permit: VerifiedTransferExecutionPermit,
    pub operation: BindingOperationKey,
    pub tenant_id: TenantId,
    pub participant_id: CapabilityParticipantId,
    pub effect_fingerprint: BindingDigest32,
    pub expected_producer: crate::BindingProducerId,
    pub expected_audience: crate::BindingProducerId,
    pub now_unix_seconds: u64,
    pub maximum_clock_uncertainty_millis: u64,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TransferExecutionLedgerRevision(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferExecutionLedgerV1 {
    pub operation: BindingOperationKey,
    pub ordinary_debited_bytes: u64,
    pub ordinary_debited_effects: u64,
    pub ordinary_debited_cost_microunits: u64,
    pub forward_debited_bytes: u64,
    pub forward_debited_effects: u64,
    pub forward_debited_cost_microunits: u64,
    pub permit_root_digest: BindingDigest32,
    pub permit_count: u64,
    pub outcome_root_digest: BindingDigest32,
    pub outcome_count: u64,
    pub revision: TransferExecutionLedgerRevision,
    pub record_digest: BindingDigest32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct IssueTransferExecutionPermitWriteSetV1 {
    parts: IssueTransferExecutionPermitWriteSetPartsV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct IssueTransferExecutionPermitWriteSetPartsV1 {
    pub authority: BindingPersistenceAuthorityV1,
    pub expected_operation_revision: BindingOperationRevision,
    pub operation: BindingOperationV1,
    pub expected_ledger_revision: TransferExecutionLedgerRevision,
    pub item_precondition: crate::TransferExecutionItemPreconditionV1,
    pub movement_permit: VerifiedCellMovementPermit,
    pub authorization: VerifiedResidencyTransferAuthorization,
    pub authorization_set: VerifiedResidencyTransferAuthorizationSet,
    pub participant: VerifiedParticipantManifestMember,
    pub permit: VerifiedTransferExecutionPermit,
    pub next_ledger: TransferExecutionLedgerV1,
    pub next_item: crate::TransferExecutionItemV1,
    pub idempotency: BindingIdempotencyRecordV1,
    pub audit_outbox: BindingAuditRecordV1,
    pub cell_proof_consumptions: Vec<CellProofConsumptionV1>,
    pub binding_proof_consumptions: Vec<BindingProofConsumptionV1>,
}

impl IssueTransferExecutionPermitWriteSetV1 {
    pub fn assemble(
        _parts: IssueTransferExecutionPermitWriteSetPartsV1,
    ) -> Result<Self, BindingStoreError> {
        Err(BindingStoreError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &IssueTransferExecutionPermitWriteSetPartsV1 {
        &self.parts
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct RecordTransferExecutionOutcomeWriteSetV1 {
    parts: RecordTransferExecutionOutcomeWriteSetPartsV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct RecordTransferExecutionOutcomeWriteSetPartsV1 {
    pub authority: BindingPersistenceAuthorityV1,
    pub expected_operation_revision: BindingOperationRevision,
    pub operation: BindingOperationV1,
    pub expected_ledger_revision: TransferExecutionLedgerRevision,
    pub item_precondition: crate::TransferExecutionItemPreconditionV1,
    pub outcome: VerifiedTransferExecutionOutcome,
    pub next_ledger: TransferExecutionLedgerV1,
    pub next_item: crate::TransferExecutionItemV1,
    pub idempotency: BindingIdempotencyRecordV1,
    pub audit_outbox: BindingAuditRecordV1,
    pub proof_consumptions: Vec<BindingProofConsumptionV1>,
}

impl RecordTransferExecutionOutcomeWriteSetV1 {
    pub fn assemble(
        _parts: RecordTransferExecutionOutcomeWriteSetPartsV1,
    ) -> Result<Self, BindingStoreError> {
        Err(BindingStoreError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &RecordTransferExecutionOutcomeWriteSetPartsV1 {
        &self.parts
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct CapabilityTransferEffectWriteSetV1 {
    parts: CapabilityTransferEffectWriteSetPartsV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CapabilityTransferEffectWriteSetPartsV1 {
    pub permit: VerifiedTransferExecutionPermit,
    pub local_effect_fingerprint: BindingDigest32,
    pub drain_mutation: cell_placement::DrainContributorStateMutationV1,
    pub local_idempotency_digest: BindingDigest32,
    pub local_audit_record_digest: BindingDigest32,
}

impl CapabilityTransferEffectWriteSetV1 {
    pub fn assemble(
        _parts: CapabilityTransferEffectWriteSetPartsV1,
    ) -> Result<Self, BindingStoreError> {
        Err(BindingStoreError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &CapabilityTransferEffectWriteSetPartsV1 {
        &self.parts
    }
}

pub fn verify_transfer_execution_permit(
    _verifier: &dyn BindingProofVerifier,
    _signed: SignedTransferExecutionPermitV1,
    _expectation: &TransferExecutionPermitExpectationV1,
) -> Result<VerifiedTransferExecutionPermit, BindingProofVerificationError> {
    Err(BindingProofVerificationError::NotImplemented)
}

pub fn verify_transfer_execution_outcome(
    _verifier: &dyn BindingProofVerifier,
    _signed: SignedTransferExecutionOutcomeV1,
    _expectation: &TransferExecutionOutcomeExpectationV1,
) -> Result<VerifiedTransferExecutionOutcome, BindingProofVerificationError> {
    Err(BindingProofVerificationError::NotImplemented)
}
