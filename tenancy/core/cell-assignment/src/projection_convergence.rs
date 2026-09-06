use cell_placement::CellId;

use crate::{
    BindingDigest32, BindingGeneration, BindingProducerId, BindingProofEnvelopeV1,
    BindingProofVerificationError, BindingProofVerifier, BindingRevision, TenantId,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionAudiencePolicyV1 {
    pub source: cell_placement::ImmutableEvidenceRefV1,
    pub policy_generation: u64,
    pub required_audience_root_digest: BindingDigest32,
    pub required_audience_count: u64,
    pub maximum_allowed_projection_age_seconds: u64,
    pub policy_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConvergencePayloadV1 {
    pub schema_version: u32,
    pub tenant_id: TenantId,
    pub cell_id: CellId,
    pub binding_generation: BindingGeneration,
    pub binding_revision: BindingRevision,
    pub write_authority_epoch: crate::WriteAuthorityEpoch,
    pub binding_record_digest: BindingDigest32,
    pub audience_policy: ProjectionAudiencePolicyV1,
    pub installed_audience_root_digest: BindingDigest32,
    pub installed_audience_count: u64,
    pub oldest_installed_projection_age_seconds: u64,
    pub observed_at_unix_seconds: u64,
    pub record_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedProjectionConvergenceV1 {
    pub payload: ProjectionConvergencePayloadV1,
    pub envelope: BindingProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedProjectionConvergence(SignedProjectionConvergenceV1);

impl VerifiedProjectionConvergence {
    #[must_use]
    pub fn signed(&self) -> &SignedProjectionConvergenceV1 {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConvergenceExpectationV1 {
    pub tenant_id: TenantId,
    pub cell_id: CellId,
    pub binding_generation: BindingGeneration,
    pub binding_revision: BindingRevision,
    pub write_authority_epoch: crate::WriteAuthorityEpoch,
    pub binding_record_digest: BindingDigest32,
    pub audience_policy: ProjectionAudiencePolicyV1,
    pub expected_producer: BindingProducerId,
    pub expected_audience: BindingProducerId,
    pub now_unix_seconds: u64,
}

pub fn verify_projection_convergence(
    _verifier: &dyn BindingProofVerifier,
    _signed: SignedProjectionConvergenceV1,
    _successor_binding: &crate::TenantCellBinding,
    _expectation: &ProjectionConvergenceExpectationV1,
) -> Result<VerifiedProjectionConvergence, BindingProofVerificationError> {
    Err(BindingProofVerificationError::NotImplemented)
}
