use cell_placement::{
    BindingOutcomeQueryRefV1, ReservationRefV1, SignedPlacementDecisionV1,
    SignedReservationArmIntentV1, SignedReservationArmReceiptV1, SignedReservationCommitPermitV1,
    VerifiedReservationArmReceipt, VerifiedReservationCommitPermit,
};

use crate::{BindingDigest32, BindingOperationKey, SignedParticipantManifestV1};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BindingReservationAttemptRevision(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BindingReservationAttemptStateV1 {
    Opened,
    PartiallyArmed,
    FullyArmed,
    CommitPermitted,
    OutcomeCommitted,
    OutcomeAborted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingReservationAttemptV1 {
    operation: BindingOperationKey,
    outcome_query: BindingOutcomeQueryRefV1,
    binding_attempt_digest: BindingDigest32,
    binding_precondition_digest: BindingDigest32,
    placement_decision_digest: BindingDigest32,
    required_reservation_set_digest: BindingDigest32,
    arm_intent_set_digest: BindingDigest32,
    placement_decision: SignedPlacementDecisionV1,
    participant_manifest: SignedParticipantManifestV1,
    home_reservation: ReservationRefV1,
    warm_recovery_reservation: Option<ReservationRefV1>,
    arm_intents: Vec<SignedReservationArmIntentV1>,
    arm_receipts: Vec<SignedReservationArmReceiptV1>,
    commit_permit: Option<SignedReservationCommitPermitV1>,
    state: BindingReservationAttemptStateV1,
    revision: BindingReservationAttemptRevision,
    opened_at_unix_seconds: u64,
    settlement_deadline_unix_seconds: u64,
    record_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingReservationAttemptPartsV1 {
    pub operation: BindingOperationKey,
    pub outcome_query: BindingOutcomeQueryRefV1,
    pub binding_attempt_digest: BindingDigest32,
    pub binding_precondition_digest: BindingDigest32,
    pub placement_decision_digest: BindingDigest32,
    pub required_reservation_set_digest: BindingDigest32,
    pub arm_intent_set_digest: BindingDigest32,
    pub placement_decision: SignedPlacementDecisionV1,
    pub participant_manifest: SignedParticipantManifestV1,
    pub home_reservation: ReservationRefV1,
    pub warm_recovery_reservation: Option<ReservationRefV1>,
    pub arm_intents: Vec<SignedReservationArmIntentV1>,
    pub arm_receipts: Vec<SignedReservationArmReceiptV1>,
    pub commit_permit: Option<SignedReservationCommitPermitV1>,
    pub state: BindingReservationAttemptStateV1,
    pub revision: BindingReservationAttemptRevision,
    pub opened_at_unix_seconds: u64,
    pub settlement_deadline_unix_seconds: u64,
    pub record_digest: BindingDigest32,
}

impl BindingReservationAttemptV1 {
    pub fn rehydrate(
        _parts: BindingReservationAttemptPartsV1,
    ) -> Result<Self, BindingReservationAttemptError> {
        Err(BindingReservationAttemptError::NotImplemented)
    }

    #[must_use]
    pub fn operation(&self) -> &BindingOperationKey {
        &self.operation
    }

    #[must_use]
    pub fn outcome_query(&self) -> &BindingOutcomeQueryRefV1 {
        &self.outcome_query
    }

    #[must_use]
    pub fn binding_attempt_digest(&self) -> BindingDigest32 {
        self.binding_attempt_digest
    }

    #[must_use]
    pub fn binding_precondition_digest(&self) -> BindingDigest32 {
        self.binding_precondition_digest
    }

    #[must_use]
    pub fn placement_decision_digest(&self) -> BindingDigest32 {
        self.placement_decision_digest
    }

    #[must_use]
    pub fn required_reservation_set_digest(&self) -> BindingDigest32 {
        self.required_reservation_set_digest
    }

    #[must_use]
    pub fn arm_intent_set_digest(&self) -> BindingDigest32 {
        self.arm_intent_set_digest
    }

    #[must_use]
    pub fn placement_decision(&self) -> &SignedPlacementDecisionV1 {
        &self.placement_decision
    }

    #[must_use]
    pub fn participant_manifest(&self) -> &SignedParticipantManifestV1 {
        &self.participant_manifest
    }

    #[must_use]
    pub fn reservations(&self) -> (&ReservationRefV1, Option<&ReservationRefV1>) {
        (
            &self.home_reservation,
            self.warm_recovery_reservation.as_ref(),
        )
    }

    #[must_use]
    pub fn arm_receipts(&self) -> &[SignedReservationArmReceiptV1] {
        &self.arm_receipts
    }

    #[must_use]
    pub fn arm_intents(&self) -> &[SignedReservationArmIntentV1] {
        &self.arm_intents
    }

    #[must_use]
    pub fn commit_permit(&self) -> Option<&SignedReservationCommitPermitV1> {
        self.commit_permit.as_ref()
    }

    #[must_use]
    pub fn state(&self) -> BindingReservationAttemptStateV1 {
        self.state
    }

    #[must_use]
    pub fn revision(&self) -> BindingReservationAttemptRevision {
        self.revision
    }

    #[must_use]
    pub fn opened_at_unix_seconds(&self) -> u64 {
        self.opened_at_unix_seconds
    }

    #[must_use]
    pub fn settlement_deadline_unix_seconds(&self) -> u64 {
        self.settlement_deadline_unix_seconds
    }

    #[must_use]
    pub fn record_digest(&self) -> BindingDigest32 {
        self.record_digest
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum BindingAttemptCheckpointEvidenceV1 {
    ArmReceipt(Box<VerifiedReservationArmReceipt>),
    CommitPermit(Box<VerifiedReservationCommitPermit>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BindingReservationAttemptPreconditionV1(pub BindingReservationAttemptRevision);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingReservationAttemptError {
    NotImplemented,
    InvalidState,
    InvalidRevision,
    DuplicateReceipt,
    MissingArmIntent,
    IncompleteReservationSet,
    CommitPermitBeforeFullyArmed,
    ReservationSetMismatch,
    ProofRelationMismatch,
    TerminalMutation,
}

#[derive(Debug, Eq, PartialEq)]
pub struct BindingAttemptMutationResultV1 {
    pub attempt: BindingReservationAttemptV1,
    pub operation: crate::BindingOperationV1,
}
