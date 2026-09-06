use crate::{
    AssuranceAuditPolicyV1, AuthorizationDecisionReceiptV1, BoxCellFuture, CellId, Digest32,
    PlacementContractError, PlacementIdempotencyKey, PlacementOperationKey,
    PlacementOperationRevision, PlacementPersistenceAuthorityV1, PlacementReadAuthorityV1,
    TenantId,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlacementAuditSelectionOutcomeV1 {
    Selected {
        primary_location_digest: Digest32,
        recovery_location_digest: Option<Digest32>,
        placement_decision_digest: Digest32,
    },
    Deferred {
        continuation_digest: Digest32,
    },
    Exhausted {
        exhaustion_digest: Digest32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlacementAuditEffectV1 {
    SelectionEvaluated {
        assurance_requirements_digest: Digest32,
        assurance_evidence_digest: Digest32,
        recovery_evidence_digest: Digest32,
        search_plan_digest: Digest32,
        outcome: PlacementAuditSelectionOutcomeV1,
    },
    ReservationArmed {
        cell_id: CellId,
        reservation_digest: Digest32,
        binding_attempt_digest: Digest32,
        placement_decision_digest: Digest32,
        arm_receipt_digest: Digest32,
    },
    ReservationCommitPermitIssued {
        binding_attempt_digest: Digest32,
        placement_decision_digest: Digest32,
        home_reservation_digest: Digest32,
        warm_recovery_reservation_digest: Option<Digest32>,
        reservation_commit_permit_digest: Digest32,
    },
    MovementScheduled {
        source_cell_id: CellId,
        target_cell_id: CellId,
        assurance_requirements_digest: Digest32,
        assurance_evidence_digest: Digest32,
        recovery_evidence_digest: Digest32,
        placement_decision_digest: Digest32,
        reservation_commit_permit_digest: Digest32,
        binding_participant_commitment_digest: Digest32,
        participant_manifest_record_digest: Digest32,
        participant_count: u64,
        budget_request_digest: Digest32,
        budget_authority_previous_record_digest: Digest32,
        budget_authority_next_record_digest: Digest32,
        budget_authority_next_revision: crate::MovementBudgetAuthorityRevision,
        movement_permit_digest: Digest32,
    },
    BindingOutcomeApplied {
        cell_id: CellId,
        reservation_digest: Digest32,
        binding_attempt_digest: Digest32,
        placement_decision_digest: Digest32,
        reservation_commit_permit_digest: Option<Digest32>,
        binding_outcome_digest: Digest32,
    },
    SourceReservationReleased {
        cell_id: CellId,
        reservation_digest: Digest32,
        source_binding_record_digest: Digest32,
        successor_binding_record_digest: Digest32,
        release_permit_digest: Digest32,
    },
    OperationCancelled {
        target_operation: PlacementOperationKey,
        expected_target_revision: PlacementOperationRevision,
    },
    OperationCheckpointed {
        checkpoint_digest: Digest32,
    },
    OperationRepaired {
        target_operation: PlacementOperationKey,
        expected_target_revision: PlacementOperationRevision,
        repair_authority_digest: Digest32,
        applied_checkpoint_digest: Digest32,
    },
    OperationRefused {
        refusal_digest: Digest32,
    },
    OperationFailed {
        failure_digest: Digest32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlacementAuditRecordPartsV1 {
    pub audit_event_id: String,
    pub tenant_id: TenantId,
    pub operation: PlacementOperationKey,
    pub actor_digest: Digest32,
    pub authorization: AuthorizationDecisionReceiptV1,
    pub idempotency_key: PlacementIdempotencyKey,
    pub request_digest: Digest32,
    pub effect: PlacementAuditEffectV1,
    pub result_digest: Digest32,
    pub occurred_at_unix_seconds: u64,
    pub assurance_audit_policy: AssuranceAuditPolicyV1,
    pub record_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlacementAuditRecordV1 {
    parts: PlacementAuditRecordPartsV1,
}

impl PlacementAuditRecordV1 {
    pub fn assemble(
        _authority: &PlacementPersistenceAuthorityV1,
        _parts: PlacementAuditRecordPartsV1,
    ) -> Result<Self, PlacementContractError> {
        Err(PlacementContractError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &PlacementAuditRecordPartsV1 {
        &self.parts
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlacementAuditPageTokenV1(Vec<u8>);

impl PlacementAuditPageTokenV1 {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn parse(_value: Vec<u8>) -> Result<Self, crate::PlacementContractError> {
        Err(crate::PlacementContractError::NotImplemented)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlacementAuditPageRequestV1 {
    pub tenant_id: TenantId,
    pub page_size: u32,
    pub page_token: Option<PlacementAuditPageTokenV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlacementAuditPageV1 {
    pub records: Vec<PlacementAuditRecordV1>,
    pub next_page_token: Option<PlacementAuditPageTokenV1>,
}

pub trait PlacementAuditReader: Send + Sync {
    fn read_page<'a>(
        &'a self,
        authority: &'a PlacementReadAuthorityV1,
        request: &'a PlacementAuditPageRequestV1,
    ) -> BoxCellFuture<'a, Result<PlacementAuditPageV1, PlacementContractError>>;
}
