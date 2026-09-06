use crate::{
    CellProofEnvelopeV1, CellProofVerifier, Digest32, PlacementOperationKey,
    PlacementOperationRevision, ProducerId, ProofVerificationError,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PlacementRepairScopeV1 {
    SearchContinuation,
    ReservationState,
    BindingOutcomeConvergence,
    AuditOutbox,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlacementRepairAuthorityPayloadV1 {
    pub schema_version: u32,
    pub repair_operation: PlacementOperationKey,
    pub target_operation: PlacementOperationKey,
    pub expected_target_revision: PlacementOperationRevision,
    pub scope: PlacementRepairScopeV1,
    pub requested_checkpoint_digest: Digest32,
    pub reason_digest: Digest32,
    pub expires_at_unix_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedPlacementRepairAuthorityV1 {
    pub payload: PlacementRepairAuthorityPayloadV1,
    pub envelope: CellProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlacementRepairAuthorityExpectationV1 {
    pub repair_operation: PlacementOperationKey,
    pub target_operation: PlacementOperationKey,
    pub expected_target_revision: PlacementOperationRevision,
    pub scope: PlacementRepairScopeV1,
    pub expected_producer: ProducerId,
    pub expected_audience: ProducerId,
    pub now_unix_seconds: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedPlacementRepairAuthority(SignedPlacementRepairAuthorityV1);

impl VerifiedPlacementRepairAuthority {
    #[must_use]
    pub fn signed(&self) -> &SignedPlacementRepairAuthorityV1 {
        &self.0
    }
}

pub fn verify_placement_repair_authority(
    _verifier: &dyn CellProofVerifier,
    _signed: SignedPlacementRepairAuthorityV1,
    _expectation: &PlacementRepairAuthorityExpectationV1,
) -> Result<VerifiedPlacementRepairAuthority, ProofVerificationError> {
    Err(ProofVerificationError::NotImplemented)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlacementRepairAppliedV1 {
    pub target_operation: PlacementOperationKey,
    pub repaired_target_revision: PlacementOperationRevision,
    pub scope: PlacementRepairScopeV1,
    pub applied_checkpoint_digest: Digest32,
}
