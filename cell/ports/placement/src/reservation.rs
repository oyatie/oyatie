use crate::{
    CellProofEnvelopeV1, CellProofVerifier, Digest32, PlacementOperationKey, ProducerId,
    ProofVerificationError, ReservationRefV1, TenantId,
};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BindingOutcomeQueryRefV1 {
    pub tenant_id: TenantId,
    pub binding_operation_id: String,
    pub tenancy_shard_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReservationArmIntentPayloadV1 {
    pub schema_version: u32,
    pub placement_operation: PlacementOperationKey,
    pub arm_operation: PlacementOperationKey,
    pub binding_outcome_query: BindingOutcomeQueryRefV1,
    pub binding_attempt_digest: Digest32,
    pub placement_decision_digest: Digest32,
    pub reservation_to_arm: ReservationRefV1,
    pub home_reservation: ReservationRefV1,
    pub warm_recovery_reservation: Option<ReservationRefV1>,
    pub required_reservation_set_digest: Digest32,
    pub binding_precondition_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedReservationArmIntentV1 {
    pub payload: ReservationArmIntentPayloadV1,
    pub envelope: CellProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReservationArmReceiptPayloadV1 {
    pub schema_version: u32,
    pub placement_operation: PlacementOperationKey,
    pub arm_operation: PlacementOperationKey,
    pub binding_outcome_query: BindingOutcomeQueryRefV1,
    pub binding_attempt_digest: Digest32,
    pub placement_decision_digest: Digest32,
    pub reservation: ReservationRefV1,
    pub required_reservation_set_digest: Digest32,
    pub armed_revision: u64,
    pub armed_at_unix_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedReservationArmReceiptV1 {
    pub payload: ReservationArmReceiptPayloadV1,
    pub envelope: CellProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReservationArmIntentExpectation {
    pub placement_operation: PlacementOperationKey,
    pub arm_operation: PlacementOperationKey,
    pub binding_outcome_query: BindingOutcomeQueryRefV1,
    pub binding_attempt_digest: Digest32,
    pub placement_decision_digest: Digest32,
    pub reservation_to_arm: ReservationRefV1,
    pub required_reservation_set_digest: Digest32,
    pub expected_producer: ProducerId,
    pub expected_audience: ProducerId,
    pub now_unix_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReservationArmReceiptExpectation {
    pub placement_operation: PlacementOperationKey,
    pub arm_operation: PlacementOperationKey,
    pub binding_outcome_query: BindingOutcomeQueryRefV1,
    pub binding_attempt_digest: Digest32,
    pub placement_decision_digest: Digest32,
    pub reservation: ReservationRefV1,
    pub required_reservation_set_digest: Digest32,
    pub expected_producer: ProducerId,
    pub expected_audience: ProducerId,
    pub now_unix_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReservationCommitPermitPayloadV1 {
    pub schema_version: u32,
    pub placement_operation: PlacementOperationKey,
    pub permit_operation: PlacementOperationKey,
    pub binding_outcome_query: BindingOutcomeQueryRefV1,
    pub binding_attempt_digest: Digest32,
    pub placement_decision_digest: Digest32,
    pub home_reservation: ReservationRefV1,
    pub warm_recovery_reservation: Option<ReservationRefV1>,
    pub required_reservation_set_digest: Digest32,
    pub home_arm_receipt_digest: Digest32,
    pub warm_recovery_arm_receipt_digest: Option<Digest32>,
    pub binding_precondition_digest: Digest32,
    pub fully_armed_at_unix_seconds: u64,
    pub outcome_pending_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedReservationCommitPermitV1 {
    pub payload: ReservationCommitPermitPayloadV1,
    pub envelope: CellProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BindingAbortCodeV1 {
    PlacementAbandoned,
    BindingConflict,
    CapabilityPreparationFailed,
    DeadlineExceededBeforeFence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingCommittedV1 {
    pub binding_generation: u64,
    pub binding_revision: u64,
    pub binding_record_digest: Digest32,
    pub committed_at_unix_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BindingOutcomeV1 {
    Committed(BindingCommittedV1),
    Aborted(BindingAbortCodeV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReservationOutcomeMemberV1 {
    pub reservation: ReservationRefV1,
    pub arm_receipt_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingOutcomePayloadV1 {
    pub schema_version: u32,
    pub tenant_id: TenantId,
    pub binding_operation_id: String,
    pub tenancy_shard_key: String,
    pub binding_attempt_digest: Digest32,
    pub home_reservation: ReservationRefV1,
    pub warm_recovery_reservation: Option<ReservationRefV1>,
    pub required_reservation_set_digest: Digest32,
    pub armed_reservations: Vec<ReservationOutcomeMemberV1>,
    pub reservation_commit_permit_digest: Option<Digest32>,
    pub placement_decision_digest: Digest32,
    pub outcome: BindingOutcomeV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedBindingOutcomeV1 {
    pub payload: BindingOutcomePayloadV1,
    pub envelope: CellProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReservationCommitPermitExpectation {
    pub placement_operation: PlacementOperationKey,
    pub permit_operation: PlacementOperationKey,
    pub binding_outcome_query: BindingOutcomeQueryRefV1,
    pub binding_attempt_digest: Digest32,
    pub placement_decision_digest: Digest32,
    pub home_reservation: ReservationRefV1,
    pub warm_recovery_reservation: Option<ReservationRefV1>,
    pub required_reservation_set_digest: Digest32,
    pub home_arm_receipt_digest: Digest32,
    pub warm_recovery_arm_receipt_digest: Option<Digest32>,
    pub binding_precondition_digest: Digest32,
    pub expected_producer: ProducerId,
    pub expected_audience: ProducerId,
    pub now_unix_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingOutcomeExpectation {
    pub tenant_id: TenantId,
    pub binding_operation_id: String,
    pub tenancy_shard_key: String,
    pub binding_attempt_digest: Digest32,
    pub home_reservation: ReservationRefV1,
    pub warm_recovery_reservation: Option<ReservationRefV1>,
    pub required_reservation_set_digest: Digest32,
    pub reservation_commit_permit_digest: Option<Digest32>,
    pub placement_decision_digest: Digest32,
    pub expected_producer: ProducerId,
    pub expected_audience: ProducerId,
    pub now_unix_seconds: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedReservationCommitPermit(SignedReservationCommitPermitV1);

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedBindingOutcome(SignedBindingOutcomeV1);

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedReservationArmIntent(SignedReservationArmIntentV1);

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedReservationArmReceipt(SignedReservationArmReceiptV1);

impl VerifiedReservationCommitPermit {
    #[must_use]
    pub fn signed(&self) -> &SignedReservationCommitPermitV1 {
        &self.0
    }
}

impl VerifiedBindingOutcome {
    #[must_use]
    pub fn signed(&self) -> &SignedBindingOutcomeV1 {
        &self.0
    }
}

impl VerifiedReservationArmIntent {
    #[must_use]
    pub fn signed(&self) -> &SignedReservationArmIntentV1 {
        &self.0
    }
}

impl VerifiedReservationArmReceipt {
    #[must_use]
    pub fn signed(&self) -> &SignedReservationArmReceiptV1 {
        &self.0
    }
}

pub fn verify_reservation_arm_intent(
    _verifier: &dyn CellProofVerifier,
    _signed: SignedReservationArmIntentV1,
    _expectation: &ReservationArmIntentExpectation,
) -> Result<VerifiedReservationArmIntent, ProofVerificationError> {
    Err(ProofVerificationError::NotImplemented)
}

pub fn verify_reservation_arm_receipt(
    _verifier: &dyn CellProofVerifier,
    _signed: SignedReservationArmReceiptV1,
    _expectation: &ReservationArmReceiptExpectation,
) -> Result<VerifiedReservationArmReceipt, ProofVerificationError> {
    Err(ProofVerificationError::NotImplemented)
}

pub fn verify_reservation_commit_permit(
    _verifier: &dyn CellProofVerifier,
    _signed: SignedReservationCommitPermitV1,
    _expectation: &ReservationCommitPermitExpectation,
) -> Result<VerifiedReservationCommitPermit, ProofVerificationError> {
    Err(ProofVerificationError::NotImplemented)
}

pub fn verify_binding_outcome(
    _verifier: &dyn CellProofVerifier,
    _signed: SignedBindingOutcomeV1,
    _expectation: &BindingOutcomeExpectation,
) -> Result<VerifiedBindingOutcome, ProofVerificationError> {
    Err(ProofVerificationError::NotImplemented)
}
