use crate::{
    BindingParticipantManifestCommitmentExpectationV1, CellMovementPermitExpectationV1,
    CellProofVerifier, ProofVerificationError, SignedBindingParticipantManifestCommitmentV1,
    SignedCellMovementPermitV1, VerifiedBindingParticipantManifestCommitment,
};

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedCellMovementPermit(SignedCellMovementPermitV1);

impl VerifiedCellMovementPermit {
    #[must_use]
    pub fn signed(&self) -> &SignedCellMovementPermitV1 {
        &self.0
    }
}

pub fn verify_cell_movement_permit(
    _verifier: &dyn CellProofVerifier,
    _signed: SignedCellMovementPermitV1,
    _expectation: &CellMovementPermitExpectationV1,
) -> Result<VerifiedCellMovementPermit, ProofVerificationError> {
    Err(ProofVerificationError::NotImplemented)
}

pub fn verify_binding_participant_manifest_commitment(
    _verifier: &dyn CellProofVerifier,
    _signed: SignedBindingParticipantManifestCommitmentV1,
    _expectation: &BindingParticipantManifestCommitmentExpectationV1,
) -> Result<VerifiedBindingParticipantManifestCommitment, ProofVerificationError> {
    Err(ProofVerificationError::NotImplemented)
}
