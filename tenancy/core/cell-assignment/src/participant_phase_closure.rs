use crate::{
    BindingDigest32, BindingOperationKey, BindingProducerId, BindingProofEnvelopeV1,
    BindingProofVerificationError, BindingProofVerifier, ParticipantReceiptLedgerRevision,
    ParticipantReceiptPhaseV1, TenantId,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantPhaseClosurePayloadV1 {
    pub schema_version: u32,
    pub operation: BindingOperationKey,
    pub tenant_id: TenantId,
    pub phase: ParticipantReceiptPhaseV1,
    pub manifest_digest: BindingDigest32,
    pub placement_context_digest: BindingDigest32,
    pub participant_root_digest: BindingDigest32,
    pub receipt_root_digest: BindingDigest32,
    pub participant_count: u64,
    pub ledger_revision: ParticipantReceiptLedgerRevision,
    pub closed_at_unix_seconds: u64,
    pub record_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedParticipantPhaseClosureV1 {
    pub payload: ParticipantPhaseClosurePayloadV1,
    pub envelope: BindingProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantPhaseClosureExpectationV1 {
    pub operation: BindingOperationKey,
    pub tenant_id: TenantId,
    pub phase: ParticipantReceiptPhaseV1,
    pub manifest_digest: BindingDigest32,
    pub placement_context_digest: BindingDigest32,
    pub participant_root_digest: BindingDigest32,
    pub receipt_root_digest: BindingDigest32,
    pub participant_count: u64,
    pub ledger_revision: ParticipantReceiptLedgerRevision,
    pub expected_producer: BindingProducerId,
    pub expected_audience: BindingProducerId,
    pub now_unix_seconds: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedParticipantPhaseClosure(SignedParticipantPhaseClosureV1);

impl VerifiedParticipantPhaseClosure {
    #[must_use]
    pub fn signed(&self) -> &SignedParticipantPhaseClosureV1 {
        &self.0
    }
}

pub fn verify_participant_phase_closure(
    _verifier: &dyn BindingProofVerifier,
    _signed: SignedParticipantPhaseClosureV1,
    _expectation: &ParticipantPhaseClosureExpectationV1,
) -> Result<VerifiedParticipantPhaseClosure, BindingProofVerificationError> {
    Err(BindingProofVerificationError::NotImplemented)
}
