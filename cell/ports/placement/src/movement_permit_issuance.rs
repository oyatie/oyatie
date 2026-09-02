use crate::{
    BoxCellFuture, CellMovementPermitIntentV1, CellProofConsumptionV1, CellProofEnvelopeV1,
    CellProofVerifier, Digest32, MovementBudgetAuthorityPartition, MovementBudgetAuthorityRevision,
    MovementBudgetGrantV1, PlacementAuditRecordV1, PlacementContractError,
    PlacementIdempotencyRecordV1, PlacementOperationPreconditionV1, PlacementOperationV1,
    PlacementPersistenceAuthorityV1, PlacementReconciliationPersistenceAuthorityV1, ProducerId,
    ProofVerificationError, SignedCellMovementPermitV1, VerifiedCellMovementPermit,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MovementPermitIssuanceRevision(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MovementPermitIssuanceStatusV1 {
    PendingSignature,
    Published,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MovementPermitCommitContextV1 {
    Request,
    Reconciliation {
        candidate_digest: Digest32,
        lease_epoch: u64,
        lease_digest: Digest32,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum MovementPermitIssuanceClaimContextV1 {
    Request,
    Reconciliation(crate::CellReconciliationLeaseV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MovementPermitIssuancePreconditionV1 {
    pub expected_revision: MovementPermitIssuanceRevision,
    pub expected_status: MovementPermitIssuanceStatusV1,
    pub expected_record_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MovementPermitIssuanceRecordV1 {
    intent: CellMovementPermitIntentV1,
    grant: MovementBudgetGrantV1,
    leaf_state_revision_at_commit: MovementBudgetAuthorityRevision,
    leaf_state_record_digest_at_commit: Digest32,
    status: MovementPermitIssuanceStatusV1,
    signed_permit: Option<SignedCellMovementPermitV1>,
    revision: MovementPermitIssuanceRevision,
    record_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MovementPermitIssuanceRecordPartsV1 {
    pub intent: CellMovementPermitIntentV1,
    pub grant: MovementBudgetGrantV1,
    pub leaf_state_revision_at_commit: MovementBudgetAuthorityRevision,
    pub leaf_state_record_digest_at_commit: Digest32,
    pub status: MovementPermitIssuanceStatusV1,
    pub signed_permit: Option<SignedCellMovementPermitV1>,
    pub revision: MovementPermitIssuanceRevision,
    pub record_digest: Digest32,
}

impl MovementPermitIssuanceRecordV1 {
    pub fn rehydrate(
        _parts: MovementPermitIssuanceRecordPartsV1,
    ) -> Result<Self, PlacementContractError> {
        Err(PlacementContractError::NotImplemented)
    }

    #[must_use]
    pub fn intent(&self) -> &CellMovementPermitIntentV1 {
        &self.intent
    }

    #[must_use]
    pub fn grant(&self) -> &MovementBudgetGrantV1 {
        &self.grant
    }

    #[must_use]
    pub fn leaf_state_revision_at_commit(&self) -> MovementBudgetAuthorityRevision {
        self.leaf_state_revision_at_commit
    }

    #[must_use]
    pub fn leaf_state_record_digest_at_commit(&self) -> Digest32 {
        self.leaf_state_record_digest_at_commit
    }

    #[must_use]
    pub fn status(&self) -> MovementPermitIssuanceStatusV1 {
        self.status
    }

    #[must_use]
    pub fn signed_permit(&self) -> Option<&SignedCellMovementPermitV1> {
        self.signed_permit.as_ref()
    }

    #[must_use]
    pub fn revision(&self) -> MovementPermitIssuanceRevision {
        self.revision
    }

    #[must_use]
    pub fn record_digest(&self) -> Digest32 {
        self.record_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MovementPermitCommitAttestationPayloadV1 {
    pub schema_version: u32,
    pub issuance_revision: MovementPermitIssuanceRevision,
    pub issuance_record_digest: Digest32,
    pub movement_intent_digest: Digest32,
    pub grant_digest: Digest32,
    pub leaf_state_revision: MovementBudgetAuthorityRevision,
    pub leaf_state_record_digest: Digest32,
    pub context: MovementPermitCommitContextV1,
    pub committed_transaction_digest: Digest32,
    pub committed_at_unix_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedMovementPermitCommitAttestationV1 {
    pub payload: MovementPermitCommitAttestationPayloadV1,
    pub envelope: CellProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CommittedMovementPermitIssuanceClaimV1 {
    pub issuance: MovementPermitIssuanceRecordV1,
    pub context: MovementPermitIssuanceClaimContextV1,
    pub attestation: SignedMovementPermitCommitAttestationV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MovementPermitCommitAttestationExpectationV1 {
    pub issuance_revision: MovementPermitIssuanceRevision,
    pub issuance_record_digest: Digest32,
    pub movement_intent_digest: Digest32,
    pub grant_digest: Digest32,
    pub leaf_state_revision: MovementBudgetAuthorityRevision,
    pub leaf_state_record_digest: Digest32,
    pub context: MovementPermitCommitContextV1,
    pub expected_producer: ProducerId,
    pub expected_audience: ProducerId,
    pub now_unix_seconds: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedCommittedMovementPermitIssuance(CommittedMovementPermitIssuanceClaimV1);

impl VerifiedCommittedMovementPermitIssuance {
    #[must_use]
    pub fn claim(&self) -> &CommittedMovementPermitIssuanceClaimV1 {
        &self.0
    }
}

pub fn verify_committed_movement_permit_issuance(
    _verifier: &dyn CellProofVerifier,
    _claim: CommittedMovementPermitIssuanceClaimV1,
    _expectation: &MovementPermitCommitAttestationExpectationV1,
) -> Result<VerifiedCommittedMovementPermitIssuance, ProofVerificationError> {
    Err(ProofVerificationError::NotImplemented)
}

pub trait CellMovementAuthority: Send + Sync {
    fn sign_committed<'a>(
        &'a self,
        claim: &'a VerifiedCommittedMovementPermitIssuance,
    ) -> BoxCellFuture<'a, Result<VerifiedCellMovementPermit, PlacementContractError>>;
}

#[derive(Debug, Eq, PartialEq)]
pub enum MovementPermitPublicationAuthorityV1 {
    Request(PlacementPersistenceAuthorityV1),
    Reconciler(PlacementReconciliationPersistenceAuthorityV1),
}

#[derive(Debug, Eq, PartialEq)]
pub struct MovementPermitPublicationWriteSetV1 {
    parts: MovementPermitPublicationWriteSetPartsV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct MovementPermitPublicationWriteSetPartsV1 {
    pub authority: MovementPermitPublicationAuthorityV1,
    pub issuance_precondition: MovementPermitIssuancePreconditionV1,
    pub committed_issuance: VerifiedCommittedMovementPermitIssuance,
    pub permit: VerifiedCellMovementPermit,
    pub next_issuance: MovementPermitIssuanceRecordV1,
    pub operation_precondition: PlacementOperationPreconditionV1,
    pub operation: PlacementOperationV1,
    pub drain_mutations: crate::DrainContributorMutationSetV1,
    pub idempotency: PlacementIdempotencyRecordV1,
    pub audit_outbox: PlacementAuditRecordV1,
    pub proof_consumptions: Vec<CellProofConsumptionV1>,
}

impl MovementPermitPublicationWriteSetV1 {
    pub fn assemble(
        _parts: MovementPermitPublicationWriteSetPartsV1,
    ) -> Result<Self, PlacementContractError> {
        Err(PlacementContractError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &MovementPermitPublicationWriteSetPartsV1 {
        &self.parts
    }
}

pub trait MovementPermitPublicationStore: Send + Sync {
    fn load_committed_issuance<'a>(
        &'a self,
        authority: &'a PlacementReconciliationPersistenceAuthorityV1,
        authority_partition: &'a MovementBudgetAuthorityPartition,
        issuance_digest: &'a Digest32,
        reconciliation_lease: &'a crate::CellReconciliationLeaseV1,
    ) -> BoxCellFuture<
        'a,
        Result<Option<CommittedMovementPermitIssuanceClaimV1>, PlacementContractError>,
    >;

    fn publish<'a>(
        &'a self,
        write_set: &'a MovementPermitPublicationWriteSetV1,
    ) -> BoxCellFuture<'a, Result<MovementPermitIssuanceRecordV1, PlacementContractError>>;
}
