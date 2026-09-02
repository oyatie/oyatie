use cell_placement::{AssuranceAuditPolicyV1, CellId};

use crate::{
    BindingAuthorizationDecisionReceiptV1, BindingDigest32, BindingGeneration,
    BindingIdempotencyKey, BindingOperationKey, BindingOperationRevision,
    BindingPersistenceAuthorityV1, BindingReservationAttemptRevision, BindingRevision,
    BindingStoreError, TenantId, WriteAuthorityEpoch,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BindingAbortContextV1 {
    InitialPlacement,
    MigrationBeforeAuthorityAllocation,
    MigrationWithAuthorityAllocation {
        migration_fence_claim_digest: BindingDigest32,
        authority_high_water_record_digest: BindingDigest32,
        source_write_authority_lease_digest: BindingDigest32,
        frozen_source_write_authority_lease_state_digest: BindingDigest32,
        maximum_source_write_authority_lease_expires_at_unix_seconds: u64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingMovedAuditEffectV1 {
    pub source_cell_id: CellId,
    pub target_cell_id: CellId,
    pub expected_generation: BindingGeneration,
    pub expected_revision: BindingRevision,
    pub committed_generation: BindingGeneration,
    pub committed_revision: BindingRevision,
    pub write_authority_epoch: WriteAuthorityEpoch,
    pub authority_high_water_revision: crate::TenantWriteAuthorityHighWaterRevision,
    pub authority_high_water_record_digest: BindingDigest32,
    pub assurance_requirements_digest: BindingDigest32,
    pub assurance_evidence_digest: BindingDigest32,
    pub recovery_evidence_digest: BindingDigest32,
    pub binding_attempt_digest: BindingDigest32,
    pub required_reservation_set_digest: BindingDigest32,
    pub placement_decision_digest: BindingDigest32,
    pub reservation_commit_permit_digest: BindingDigest32,
    pub transfer_authorization_set_digest: BindingDigest32,
    pub write_fence_digest: BindingDigest32,
    pub source_fencing_completion_digest: BindingDigest32,
    pub source_write_authority_lease_digest: BindingDigest32,
    pub frozen_source_write_authority_lease_state_digest: BindingDigest32,
    pub maximum_source_write_authority_lease_expires_at_unix_seconds: u64,
    pub migration_commit_seal_digest: BindingDigest32,
    pub binding_outcome_digest: BindingDigest32,
    pub projection_digest: BindingDigest32,
    pub target_write_authority_lease_digest: BindingDigest32,
    pub target_write_authority_lease_issuance_digest: BindingDigest32,
    pub maximum_target_write_authority_lease_expires_at_unix_seconds: u64,
    pub target_write_authority_lease_state_digest: BindingDigest32,
    pub primary_location_digest: BindingDigest32,
    pub recovery_location_digest: Option<BindingDigest32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BindingAuditEffectV1 {
    WorkSnapshotPageCommitted {
        key: crate::BindingWorkSnapshotKeyV1,
        start_ordinal: u64,
        next_ordinal: u64,
        progress_record_digest: BindingDigest32,
    },
    WorkSnapshotSealed {
        key: crate::BindingWorkSnapshotKeyV1,
        progress_record_digest: BindingDigest32,
    },
    ReservationAttemptOpened {
        binding_attempt_digest: BindingDigest32,
        binding_precondition_digest: BindingDigest32,
        placement_decision_digest: BindingDigest32,
        required_reservation_set_digest: BindingDigest32,
        arm_intent_set_digest: BindingDigest32,
    },
    ReservationAttemptCheckpointed {
        binding_attempt_digest: BindingDigest32,
        attempt_revision: BindingReservationAttemptRevision,
        evidence_digest: BindingDigest32,
    },
    InitialBindingCommitted {
        target_cell_id: CellId,
        committed_generation: BindingGeneration,
        committed_revision: BindingRevision,
        write_authority_epoch: WriteAuthorityEpoch,
        authority_high_water_revision: crate::TenantWriteAuthorityHighWaterRevision,
        authority_high_water_record_digest: BindingDigest32,
        assurance_requirements_digest: BindingDigest32,
        assurance_evidence_digest: BindingDigest32,
        recovery_evidence_digest: BindingDigest32,
        binding_attempt_digest: BindingDigest32,
        required_reservation_set_digest: BindingDigest32,
        placement_decision_digest: BindingDigest32,
        reservation_commit_permit_digest: BindingDigest32,
        participant_preparation_closure_digest: BindingDigest32,
        projection_audience_policy_digest: BindingDigest32,
        binding_outcome_digest: BindingDigest32,
        projection_digest: BindingDigest32,
        write_authority_lease_digest: BindingDigest32,
        write_authority_lease_issuance_digest: BindingDigest32,
        maximum_write_authority_lease_expires_at_unix_seconds: u64,
        write_authority_lease_state_digest: BindingDigest32,
        primary_location_digest: BindingDigest32,
        recovery_location_digest: Option<BindingDigest32>,
    },
    BindingMoved(Box<BindingMovedAuditEffectV1>),
    BindingOutcomeAborted {
        context: BindingAbortContextV1,
        binding_attempt_digest: BindingDigest32,
        attempt_revision: BindingReservationAttemptRevision,
        required_reservation_set_digest: BindingDigest32,
        armed_reservation_set_digest: BindingDigest32,
        binding_outcome_digest: BindingDigest32,
    },
    MigrationFenceClaimed {
        source_cell_id: CellId,
        target_cell_id: CellId,
        source_generation: BindingGeneration,
        source_revision: BindingRevision,
        successor_generation: BindingGeneration,
        write_authority_epoch: WriteAuthorityEpoch,
        authority_high_water_revision: crate::TenantWriteAuthorityHighWaterRevision,
        authority_high_water_record_digest: BindingDigest32,
        source_write_authority_lease_digest: BindingDigest32,
        frozen_source_write_authority_lease_state_revision: crate::WriteAuthorityLeaseStateRevision,
        frozen_source_write_authority_lease_state_digest: BindingDigest32,
        maximum_source_write_authority_lease_expires_at_unix_seconds: u64,
        binding_attempt_digest: BindingDigest32,
        migration_fence_claim_digest: BindingDigest32,
        movement_permit_digest: BindingDigest32,
        transfer_authorization_set_digest: BindingDigest32,
        transfer_effect_manifest_digest: BindingDigest32,
        participant_manifest_digest: BindingDigest32,
        prepared_participant_closure_digest: BindingDigest32,
    },
    MigrationWriteFenceCommitted {
        source_cell_id: CellId,
        target_cell_id: CellId,
        source_generation: BindingGeneration,
        source_revision: BindingRevision,
        successor_generation: BindingGeneration,
        write_authority_epoch: WriteAuthorityEpoch,
        binding_attempt_digest: BindingDigest32,
        migration_fence_claim_digest: BindingDigest32,
        write_fence_digest: BindingDigest32,
        source_fencing_completion_digest: BindingDigest32,
        final_delta_digest: BindingDigest32,
    },
    ParticipantManifestCommitted {
        participant_manifest_digest: BindingDigest32,
        cell_commitment_digest: BindingDigest32,
    },
    ParticipantReceiptAppended {
        phase: crate::ParticipantReceiptPhaseV1,
        participant_id: crate::CapabilityParticipantId,
        receipt_digest: BindingDigest32,
        ledger_record_digest: BindingDigest32,
    },
    ParticipantPhaseClosed {
        phase: crate::ParticipantReceiptPhaseV1,
        closure_digest: BindingDigest32,
    },
    TransferEffectManifestCommitted {
        manifest_digest: BindingDigest32,
    },
    TransferAuthorizationAppended {
        effect_fingerprint: BindingDigest32,
        authorization_digest: BindingDigest32,
        journal_record_digest: BindingDigest32,
    },
    TransferAuthorizationSetSealed {
        authorization_set_digest: BindingDigest32,
        journal_record_digest: BindingDigest32,
    },
    TransferExecutionPermitIssued {
        effect_fingerprint: BindingDigest32,
        execution_permit_digest: BindingDigest32,
        ledger_record_digest: BindingDigest32,
    },
    TransferExecutionOutcomeRecorded {
        effect_fingerprint: BindingDigest32,
        outcome_digest: BindingDigest32,
        ledger_record_digest: BindingDigest32,
    },
    SourceFenceDirectiveIssued {
        participant_id: crate::CapabilityParticipantId,
        directive_digest: BindingDigest32,
        ledger_record_digest: BindingDigest32,
    },
    WriteAuthorityLeaseRenewalCommitted {
        cell_id: CellId,
        binding_generation: BindingGeneration,
        binding_revision: BindingRevision,
        write_authority_epoch: WriteAuthorityEpoch,
        previous_lease_digest: BindingDigest32,
        renewal_lease_digest: BindingDigest32,
        renewal_lease_issuance_digest: BindingDigest32,
        maximum_write_authority_lease_expires_at_unix_seconds: u64,
        write_authority_lease_state_digest: BindingDigest32,
    },
    WriteAuthorityLeasePublished {
        cell_id: CellId,
        binding_generation: BindingGeneration,
        binding_revision: BindingRevision,
        write_authority_epoch: WriteAuthorityEpoch,
        lease_digest: BindingDigest32,
        issuance_record_digest: BindingDigest32,
    },
    MigrationReleaseFinalized {
        target_activation_closure_digest: BindingDigest32,
        source_release_closure_digest: BindingDigest32,
        projection_convergence_digest: BindingDigest32,
        source_reservation_release_permit_digest: BindingDigest32,
    },
    OperationCancelled {
        target_operation: BindingOperationKey,
        expected_target_revision: BindingOperationRevision,
        result_digest: BindingDigest32,
    },
    OperationCheckpointed {
        checkpoint_digest: BindingDigest32,
    },
    OperationRepaired {
        target_operation: BindingOperationKey,
        expected_target_revision: BindingOperationRevision,
        repair_authority_digest: BindingDigest32,
        applied_checkpoint_digest: BindingDigest32,
    },
    OperationRefused {
        refusal_digest: BindingDigest32,
    },
    OperationFailed {
        failure_digest: BindingDigest32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingAuditRecordV1 {
    parts: BindingAuditRecordPartsV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingAuditRecordPartsV1 {
    pub audit_event_id: String,
    pub tenant_id: TenantId,
    pub operation: BindingOperationKey,
    pub actor_digest: BindingDigest32,
    pub authorization: BindingAuthorizationDecisionReceiptV1,
    pub idempotency_key: BindingIdempotencyKey,
    pub request_digest: BindingDigest32,
    pub effect: BindingAuditEffectV1,
    pub occurred_at_unix_seconds: u64,
    pub assurance_audit_policy: AssuranceAuditPolicyV1,
    pub record_digest: BindingDigest32,
}

impl BindingAuditRecordV1 {
    pub fn assemble(
        _authority: &BindingPersistenceAuthorityV1,
        _parts: BindingAuditRecordPartsV1,
    ) -> Result<Self, BindingStoreError> {
        Err(BindingStoreError::NotImplemented)
    }

    pub fn assemble_for_reconciliation(
        _authority: &crate::BindingReconciliationPersistenceAuthorityV1,
        _expected_lease: &crate::BindingReconciliationLeaseV1,
        _parts: BindingAuditRecordPartsV1,
    ) -> Result<Self, BindingStoreError> {
        Err(BindingStoreError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &BindingAuditRecordPartsV1 {
        &self.parts
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingAuditPageTokenV1(Vec<u8>);

impl BindingAuditPageTokenV1 {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn parse(_value: Vec<u8>) -> Result<Self, crate::BindingContractError> {
        Err(crate::BindingContractError::NotImplemented)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingAuditPageRequestV1 {
    pub tenant_id: TenantId,
    pub page_size: u32,
    pub page_token: Option<BindingAuditPageTokenV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingAuditPageV1 {
    pub records: Vec<BindingAuditRecordV1>,
    pub next_page_token: Option<BindingAuditPageTokenV1>,
}

pub trait BindingAuditReader: Send + Sync {
    fn read_page<'a>(
        &'a self,
        authority: &'a crate::BindingReadAuthorityV1,
        request: &'a BindingAuditPageRequestV1,
    ) -> crate::BoxTenancyFuture<'a, Result<BindingAuditPageV1, BindingStoreError>>;
}
