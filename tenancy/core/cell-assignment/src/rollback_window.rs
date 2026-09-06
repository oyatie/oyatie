use cell_placement::ImmutableEvidenceRefV1;

use crate::{
    BindingDigest32, BindingGeneration, BindingOperationKey, BindingProducerId,
    BindingProofEnvelopeV1, BindingProofVerificationError, BindingProofVerifier, BindingRevision,
    TenantId,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RollbackWindowPolicyV1 {
    pub policy_generation: u64,
    pub minimum_duration_seconds: u64,
    pub clock_authority_digest: BindingDigest32,
    pub maximum_clock_uncertainty_millis: u64,
    pub policy_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RollbackWindowElapsedPayloadV1 {
    pub schema_version: u32,
    pub operation: BindingOperationKey,
    pub tenant_id: TenantId,
    pub successor_binding_generation: BindingGeneration,
    pub successor_binding_revision: BindingRevision,
    pub successor_binding_record_digest: BindingDigest32,
    pub migration_commit_seal_digest: BindingDigest32,
    pub policy: RollbackWindowPolicyV1,
    pub binding_committed_at_unix_seconds: u64,
    pub release_not_before_unix_seconds: u64,
    pub trusted_time_evidence: ImmutableEvidenceRefV1,
    pub observed_at_unix_seconds: u64,
    pub observed_clock_uncertainty_millis: u64,
    pub record_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedRollbackWindowElapsedV1 {
    pub payload: RollbackWindowElapsedPayloadV1,
    pub envelope: BindingProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RollbackWindowElapsedExpectationV1 {
    pub operation: BindingOperationKey,
    pub tenant_id: TenantId,
    pub successor_binding_generation: BindingGeneration,
    pub successor_binding_revision: BindingRevision,
    pub successor_binding_record_digest: BindingDigest32,
    pub migration_commit_seal_digest: BindingDigest32,
    pub policy: RollbackWindowPolicyV1,
    pub authoritative_binding_committed_at_unix_seconds: u64,
    pub expected_producer: BindingProducerId,
    pub expected_audience: BindingProducerId,
    pub now_unix_seconds: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedRollbackWindowElapsed(SignedRollbackWindowElapsedV1);

impl VerifiedRollbackWindowElapsed {
    #[must_use]
    pub fn signed(&self) -> &SignedRollbackWindowElapsedV1 {
        &self.0
    }
}

pub fn verify_rollback_window_elapsed(
    _verifier: &dyn BindingProofVerifier,
    _signed: SignedRollbackWindowElapsedV1,
    _expectation: &RollbackWindowElapsedExpectationV1,
) -> Result<VerifiedRollbackWindowElapsed, BindingProofVerificationError> {
    Err(BindingProofVerificationError::NotImplemented)
}
