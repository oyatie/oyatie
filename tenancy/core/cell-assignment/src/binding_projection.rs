use cell_placement::CellId;
use tenancy_kernel::TenantId;

use crate::{
    BindingDigest32, BindingGeneration, BindingProducerId, BindingProofEnvelopeV1,
    BindingProofVerificationError, BindingProofVerifier, WriteAuthorityEpoch,
};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProjectionAudienceId(pub String);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ProjectionRefreshClassV1 {
    Standard,
    ExtendedControlPlaneOutage,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ProjectionServingPolicyV1 {
    ContinueLastVerifiedAssignmentDuringControlPlaneLoss,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingProjectionV1 {
    pub schema_version: u32,
    pub audience: ProjectionAudienceId,
    pub tenant_id: TenantId,
    pub cell_id: CellId,
    pub generation: BindingGeneration,
    pub write_authority_epoch: WriteAuthorityEpoch,
    pub issued_at_unix_seconds: u64,
    pub refresh_due_at_unix_seconds: u64,
    pub refresh_class: ProjectionRefreshClassV1,
    pub serving_policy: ProjectionServingPolicyV1,
    pub binding_record_digest: BindingDigest32,
    pub projection_partition_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedBindingProjectionV1 {
    pub payload: BindingProjectionV1,
    pub envelope: BindingProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingProjectionExpectationV1 {
    pub audience: ProjectionAudienceId,
    pub tenant_id: TenantId,
    pub cell_id: CellId,
    pub generation: BindingGeneration,
    pub write_authority_epoch: WriteAuthorityEpoch,
    pub binding_record_digest: BindingDigest32,
    pub projection_partition_digest: BindingDigest32,
    pub expected_refresh_class: ProjectionRefreshClassV1,
    pub maximum_refresh_interval_seconds: u64,
    pub expected_serving_policy: ProjectionServingPolicyV1,
    pub expected_producer: BindingProducerId,
    pub expected_audience: BindingProducerId,
    pub now_unix_seconds: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedBindingProjection(SignedBindingProjectionV1);

impl VerifiedBindingProjection {
    #[must_use]
    pub fn signed(&self) -> &SignedBindingProjectionV1 {
        &self.0
    }
}

pub fn verify_binding_projection(
    _verifier: &dyn BindingProofVerifier,
    _signed: SignedBindingProjectionV1,
    _expectation: &BindingProjectionExpectationV1,
) -> Result<VerifiedBindingProjection, BindingProofVerificationError> {
    Err(BindingProofVerificationError::NotImplemented)
}
