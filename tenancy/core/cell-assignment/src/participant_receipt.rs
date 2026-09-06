use cell_placement::CellId;

use crate::{
    BindingDigest32, BindingGeneration, BindingOperationKey, BindingProducerId,
    BindingProofEnvelopeV1, BindingProofVerificationError, BindingProofVerifier, BindingRevision,
    CapabilityParticipantId, ParticipantManifestMemberV1, TenantId, VerifiedParticipantManifest,
    WriteAuthorityEpoch,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ParticipantReceiptPhaseV1 {
    Prepared,
    SourceFenced,
    TargetActivated,
    SourceReleased,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParticipantReceiptEvidenceV1 {
    Prepared {
        preparation_digest: BindingDigest32,
    },
    SourceFenced {
        source_fence_directive_digest: BindingDigest32,
        fenced_authority_state_digest: BindingDigest32,
        write_authority_epoch: WriteAuthorityEpoch,
    },
    TargetActivated {
        binding_record_digest: BindingDigest32,
        generation: BindingGeneration,
        revision: BindingRevision,
        write_authority_epoch: WriteAuthorityEpoch,
        write_token_digest: BindingDigest32,
        write_authority_lease_digest: BindingDigest32,
        activated_authority_state_digest: BindingDigest32,
    },
    SourceReleased {
        successor_binding_record_digest: BindingDigest32,
        release_digest: BindingDigest32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParticipantPlacementContextV1 {
    InitialBinding {
        target_cell_id: CellId,
        binding_attempt_digest: BindingDigest32,
        placement_decision_digest: BindingDigest32,
        reservation_commit_permit_digest: BindingDigest32,
    },
    MigrationPreparation {
        source_cell_id: CellId,
        target_cell_id: CellId,
        source_generation: BindingGeneration,
        source_revision: BindingRevision,
        source_binding_record_digest: BindingDigest32,
        binding_attempt_digest: BindingDigest32,
        placement_decision_digest: BindingDigest32,
        reservation_commit_permit_digest: BindingDigest32,
    },
    Migration {
        source_cell_id: CellId,
        target_cell_id: CellId,
        source_generation: BindingGeneration,
        source_revision: BindingRevision,
        source_binding_record_digest: BindingDigest32,
        migration_fence_claim_digest: BindingDigest32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantReceiptPayloadV1 {
    pub schema_version: u32,
    pub operation: BindingOperationKey,
    pub tenant_id: TenantId,
    pub placement_context: ParticipantPlacementContextV1,
    pub participant_manifest_digest: BindingDigest32,
    pub member: ParticipantManifestMemberV1,
    pub phase: ParticipantReceiptPhaseV1,
    pub evidence: ParticipantReceiptEvidenceV1,
    pub occurred_at_unix_seconds: u64,
    pub record_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedParticipantReceiptV1 {
    pub payload: ParticipantReceiptPayloadV1,
    pub envelope: BindingProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedParticipantReceipt(SignedParticipantReceiptV1);

impl VerifiedParticipantReceipt {
    #[must_use]
    pub fn signed(&self) -> &SignedParticipantReceiptV1 {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ParticipantReceiptLedgerRevision(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantReceiptLedgerV1 {
    pub operation: BindingOperationKey,
    pub phase: ParticipantReceiptPhaseV1,
    pub manifest_digest: BindingDigest32,
    pub placement_context_digest: BindingDigest32,
    pub expected_participant_root_digest: BindingDigest32,
    pub expected_participant_count: u64,
    pub next_participant_ordinal: u64,
    pub applied_participant_root_digest: BindingDigest32,
    pub applied_receipt_root_digest: BindingDigest32,
    pub revision: ParticipantReceiptLedgerRevision,
    pub record_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantReceiptExpectationV1 {
    pub operation: BindingOperationKey,
    pub tenant_id: TenantId,
    pub placement_context: ParticipantPlacementContextV1,
    pub manifest_digest: BindingDigest32,
    pub participant_id: CapabilityParticipantId,
    pub member_digest: BindingDigest32,
    pub phase: ParticipantReceiptPhaseV1,
    pub expected_producer: BindingProducerId,
    pub expected_audience: BindingProducerId,
    pub now_unix_seconds: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ParticipantReceiptPhaseAuthorityV1<'a> {
    Prepared {
        attempt: &'a crate::BindingReservationAttemptV1,
        placement_decision: &'a cell_placement::VerifiedCellPlacementDecision,
        reservation_commit_permit: &'a cell_placement::VerifiedReservationCommitPermit,
    },
    SourceFenced {
        directive: &'a crate::VerifiedSourceFenceDirective,
    },
    TargetActivated {
        lease: &'a crate::VerifiedWriteAuthorityLease,
        token: &'a crate::VerifiedWriteAuthorityToken,
    },
    SourceReleased {
        source_fence_directive: &'a crate::VerifiedSourceFenceDirective,
        successor_lease: &'a crate::VerifiedWriteAuthorityLease,
        target_activation: &'a crate::VerifiedParticipantPhaseClosure,
        projection_convergence: &'a crate::VerifiedProjectionConvergence,
        rollback_window: &'a crate::VerifiedRollbackWindowElapsed,
    },
}

pub fn verify_participant_receipt(
    _verifier: &dyn BindingProofVerifier,
    _signed: SignedParticipantReceiptV1,
    _manifest: &VerifiedParticipantManifest,
    _phase_authority: &ParticipantReceiptPhaseAuthorityV1<'_>,
    _expectation: &ParticipantReceiptExpectationV1,
) -> Result<VerifiedParticipantReceipt, BindingProofVerificationError> {
    Err(BindingProofVerificationError::NotImplemented)
}
