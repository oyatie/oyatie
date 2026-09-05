use crate::{
    BoxCellFuture, CellId, CellProofEnvelopeV1, CellProofVerifier, Digest32,
    DrainContributorKindV1, DrainTermV1, PlacementContractError, ProducerId,
    ProofVerificationError,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DrainContributorStateRevision(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DrainContributorStateDispositionV1 {
    Open,
    Sealed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrainContributorStateV1 {
    pub cell_id: CellId,
    pub contributor_kind: DrainContributorKindV1,
    pub contributor_id: String,
    pub disposition: DrainContributorStateDispositionV1,
    pub drain_term: Option<DrainTermV1>,
    pub state_creation_high_water: u64,
    pub state_root_digest: Digest32,
    pub state_count: u64,
    pub revision: DrainContributorStateRevision,
    pub record_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrainContributorMutationPreconditionV1 {
    pub cell_id: CellId,
    pub contributor_id: String,
    pub expected_disposition: DrainContributorStateDispositionV1,
    pub expected_revision: DrainContributorStateRevision,
    pub expected_record_digest: Digest32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DrainContributorMutationKindV1 {
    Create,
    UpdateWhileOpen,
    Cleanup,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DrainContributorSubjectRelationKindV1 {
    Absent,
    Present,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrainContributorSubjectRelationV1 {
    pub relation_kind: DrainContributorSubjectRelationKindV1,
    pub subject_digest: Digest32,
    pub expected_state_root_digest: Digest32,
    pub relation_proof_path: Vec<Digest32>,
    pub relation_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrainContributorStateMutationV1 {
    pub precondition: DrainContributorMutationPreconditionV1,
    pub next_state: DrainContributorStateV1,
    pub mutation_kind: DrainContributorMutationKindV1,
    pub subject_relation: DrainContributorSubjectRelationV1,
    pub mutation_digest: Digest32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct DrainContributorMutationSetV1 {
    parts: DrainContributorMutationSetPartsV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrainContributorMutationSetPartsV1 {
    pub mutations: Vec<DrainContributorStateMutationV1>,
    pub ordered_mutation_root_digest: Digest32,
    pub mutation_count: u64,
    pub ordered_subject_relation_root_digest: Digest32,
    pub set_digest: Digest32,
}

impl DrainContributorMutationSetV1 {
    pub fn assemble(
        _parts: DrainContributorMutationSetPartsV1,
        _maximum_mutation_count: u32,
        _maximum_relation_proof_depth: u32,
    ) -> Result<Self, PlacementContractError> {
        Err(PlacementContractError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &DrainContributorMutationSetPartsV1 {
        &self.parts
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrainContributorSealIntentV1 {
    pub schema_version: u32,
    pub cell_id: CellId,
    pub drain_term: DrainTermV1,
    pub manifest_digest: Digest32,
    pub contributor_kind: DrainContributorKindV1,
    pub contributor_id: String,
    pub sealed_state_revision: DrainContributorStateRevision,
    pub sealed_state_creation_high_water: u64,
    pub sealed_state_root_digest: Digest32,
    pub sealed_state_count: u64,
    pub sealed_state_record_digest: Digest32,
    pub intent_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrainContributorSealCommitAttestationPayloadV1 {
    pub schema_version: u32,
    pub seal_intent_digest: Digest32,
    pub sealed_state_revision: DrainContributorStateRevision,
    pub sealed_state_record_digest: Digest32,
    pub committed_transaction_digest: Digest32,
    pub committed_at_unix_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedDrainContributorSealCommitAttestationV1 {
    pub payload: DrainContributorSealCommitAttestationPayloadV1,
    pub envelope: CellProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CommittedDrainContributorSealClaimV1 {
    pub intent: DrainContributorSealIntentV1,
    pub sealed_state: DrainContributorStateV1,
    pub attestation: SignedDrainContributorSealCommitAttestationV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrainContributorSealCommitAttestationExpectationV1 {
    pub seal_intent_digest: Digest32,
    pub sealed_state_revision: DrainContributorStateRevision,
    pub sealed_state_record_digest: Digest32,
    pub expected_producer: ProducerId,
    pub expected_audience: ProducerId,
    pub now_unix_seconds: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedCommittedDrainContributorSeal(CommittedDrainContributorSealClaimV1);

impl VerifiedCommittedDrainContributorSeal {
    #[must_use]
    pub fn claim(&self) -> &CommittedDrainContributorSealClaimV1 {
        &self.0
    }
}

pub fn verify_committed_drain_contributor_seal(
    _verifier: &dyn CellProofVerifier,
    _claim: CommittedDrainContributorSealClaimV1,
    _expectation: &DrainContributorSealCommitAttestationExpectationV1,
) -> Result<VerifiedCommittedDrainContributorSeal, ProofVerificationError> {
    Err(ProofVerificationError::NotImplemented)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrainContributorSealPayloadV1 {
    pub schema_version: u32,
    pub intent: DrainContributorSealIntentV1,
    pub commit_attestation: SignedDrainContributorSealCommitAttestationV1,
    pub seal_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedDrainContributorSealV1 {
    pub payload: DrainContributorSealPayloadV1,
    pub envelope: CellProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrainContributorSealExpectationV1 {
    pub cell_id: CellId,
    pub drain_term: DrainTermV1,
    pub manifest_digest: Digest32,
    pub contributor_kind: DrainContributorKindV1,
    pub contributor_id: String,
    pub expected_seal_intent_digest: Digest32,
    pub expected_commit_attestation_digest: Digest32,
    pub expected_producer: ProducerId,
    pub expected_audience: ProducerId,
    pub now_unix_seconds: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedDrainContributorSeal(SignedDrainContributorSealV1);

impl VerifiedDrainContributorSeal {
    #[must_use]
    pub fn signed(&self) -> &SignedDrainContributorSealV1 {
        &self.0
    }
}

pub fn verify_drain_contributor_seal(
    _verifier: &dyn CellProofVerifier,
    _signed: SignedDrainContributorSealV1,
    _expectation: &DrainContributorSealExpectationV1,
) -> Result<VerifiedDrainContributorSeal, ProofVerificationError> {
    Err(ProofVerificationError::NotImplemented)
}

#[derive(Debug, Eq, PartialEq)]
pub struct DrainContributorSealWriteSetV1 {
    parts: DrainContributorSealWriteSetPartsV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct DrainContributorSealWriteSetPartsV1 {
    pub precondition: DrainContributorMutationPreconditionV1,
    pub next_state: DrainContributorStateV1,
    pub seal_intent: DrainContributorSealIntentV1,
    pub local_idempotency_digest: Digest32,
    pub local_audit_record_digest: Digest32,
}

impl DrainContributorSealWriteSetV1 {
    pub fn assemble(
        _parts: DrainContributorSealWriteSetPartsV1,
    ) -> Result<Self, crate::PlacementContractError> {
        Err(crate::PlacementContractError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &DrainContributorSealWriteSetPartsV1 {
        &self.parts
    }
}

pub trait DrainContributorSealStore: Send + Sync {
    fn seal<'a>(
        &'a self,
        write_set: &'a DrainContributorSealWriteSetV1,
    ) -> BoxCellFuture<'a, Result<CommittedDrainContributorSealClaimV1, PlacementContractError>>;

    fn load_committed_seal<'a>(
        &'a self,
        cell_id: &'a CellId,
        contributor_id: &'a str,
        drain_term: DrainTermV1,
    ) -> BoxCellFuture<
        'a,
        Result<Option<CommittedDrainContributorSealClaimV1>, PlacementContractError>,
    >;
}

pub trait DrainContributorSealAuthority: Send + Sync {
    fn sign_committed<'a>(
        &'a self,
        claim: &'a VerifiedCommittedDrainContributorSeal,
    ) -> BoxCellFuture<'a, Result<VerifiedDrainContributorSeal, PlacementContractError>>;
}
