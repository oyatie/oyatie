use crate::{
    BindingDigest32, BindingOperationKey, BindingOperationRevision, BindingProducerId,
    BindingProofEnvelopeV1, BindingProofVerificationError, BindingProofVerifier,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BindingRepairScopeV1 {
    ReservationOutcome,
    MigrationFenceClaim,
    ForwardCompletion,
    TransferExecutionLedger,
    ProjectionConvergence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingRepairAuthorityPayloadV1 {
    pub schema_version: u32,
    pub repair_operation: BindingOperationKey,
    pub target_operation: BindingOperationKey,
    pub expected_target_revision: BindingOperationRevision,
    pub scope: BindingRepairScopeV1,
    pub requested_checkpoint_digest: BindingDigest32,
    pub reason_digest: BindingDigest32,
    pub expires_at_unix_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedBindingRepairAuthorityV1 {
    pub payload: BindingRepairAuthorityPayloadV1,
    pub envelope: BindingProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingRepairAuthorityExpectationV1 {
    pub repair_operation: BindingOperationKey,
    pub target_operation: BindingOperationKey,
    pub expected_target_revision: BindingOperationRevision,
    pub scope: BindingRepairScopeV1,
    pub expected_producer: BindingProducerId,
    pub expected_audience: BindingProducerId,
    pub now_unix_seconds: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedBindingRepairAuthority(SignedBindingRepairAuthorityV1);

impl VerifiedBindingRepairAuthority {
    #[must_use]
    pub fn signed(&self) -> &SignedBindingRepairAuthorityV1 {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingRepairAppliedV1 {
    pub target_operation: BindingOperationKey,
    pub repaired_target_revision: BindingOperationRevision,
    pub scope: BindingRepairScopeV1,
    pub applied_checkpoint_digest: BindingDigest32,
}

pub fn verify_binding_repair_authority(
    _verifier: &dyn BindingProofVerifier,
    _signed: SignedBindingRepairAuthorityV1,
    _expectation: &BindingRepairAuthorityExpectationV1,
) -> Result<VerifiedBindingRepairAuthority, BindingProofVerificationError> {
    Err(BindingProofVerificationError::NotImplemented)
}
