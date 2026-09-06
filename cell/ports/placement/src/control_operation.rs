use crate::{
    BoxCellFuture, CellControlAuditRecordV1, CellControlOperationId,
    CellControlPersistenceAuthorityV1, CellControlReadAuthorityV1, CellControlSubjectV1,
    CellViewV1, Digest32, DrainProofLedgerV1, PlacementContractError, PlacementIdempotencyKey,
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CellControlOperationKeyV1 {
    pub subject: CellControlSubjectV1,
    pub operation_id: CellControlOperationId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CellControlOperationRevision(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CellControlOperationKindV1 {
    CreateCell,
    UpdateCell,
    MutateReadiness,
    DrainCell,
    DecommissionCell,
    Rebalance,
    Repair,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CellControlOperationStateV1 {
    Pending,
    Running,
    CancelRequested,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CellControlOperationFailureCodeV1 {
    DependencyUnavailable,
    IntegrityFailure,
    AuditPersistenceFailed,
    EvidenceUnavailable,
    CapacityExhausted,
    DeadlineExceeded,
    ForwardRecoveryRequired,
    OperatorInterventionRequired,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CellControlOperationStepV1 {
    Accepted,
    ReconcilingResource,
    ValidatingPromotion,
    RejectingNewAdmissions,
    RelocatingBindings,
    ProvingZeroResidue,
    Decommissioning,
    SelectingMoves,
    ExecutingMoves,
    Terminal,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CellControlCancellationDispositionV1 {
    Cancellable,
    StopNewWorkAndConverge,
    ForwardRecoveryRequired,
    Terminal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellControlOperationFailureV1 {
    pub code: CellControlOperationFailureCodeV1,
    pub retryable: bool,
    pub details_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CellControlOperationOutcomeV1 {
    Cell(Box<CellViewV1>),
    DrainLedger(Box<DrainProofLedgerV1>),
    RebalanceJob(Box<crate::RebalanceJobV1>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellControlOperationV1 {
    pub key: CellControlOperationKeyV1,
    pub kind: CellControlOperationKindV1,
    pub state: CellControlOperationStateV1,
    pub step: CellControlOperationStepV1,
    pub cancellation: CellControlCancellationDispositionV1,
    pub revision: CellControlOperationRevision,
    pub durable_checkpoint_digest: Digest32,
    pub outcome: Option<CellControlOperationOutcomeV1>,
    pub failure: Option<CellControlOperationFailureV1>,
    pub created_at_unix_seconds: u64,
    pub updated_at_unix_seconds: u64,
    pub record_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CellControlOperationPreconditionV1 {
    Absent,
    Matches {
        revision: CellControlOperationRevision,
        record_digest: Digest32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellControlIdempotencyRecordV1 {
    pub subject: CellControlSubjectV1,
    pub idempotency_key: PlacementIdempotencyKey,
    pub request_digest: Digest32,
    pub operation: CellControlOperationKeyV1,
    pub immutable_result_digest: Option<Digest32>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CellControlOperationWriteSetV1 {
    parts: CellControlOperationWriteSetPartsV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CellControlOperationWriteSetPartsV1 {
    pub authority: CellControlPersistenceAuthorityV1,
    pub precondition: CellControlOperationPreconditionV1,
    pub operation: CellControlOperationV1,
    pub drain_mutations: crate::DrainContributorMutationSetV1,
    pub idempotency: CellControlIdempotencyRecordV1,
    pub proof_consumptions: Vec<crate::CellProofConsumptionV1>,
    pub audit_outbox: CellControlAuditRecordV1,
}

impl CellControlOperationWriteSetV1 {
    pub fn assemble(
        _parts: CellControlOperationWriteSetPartsV1,
    ) -> Result<Self, PlacementContractError> {
        Err(PlacementContractError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &CellControlOperationWriteSetPartsV1 {
        &self.parts
    }
}

pub trait CellControlOperationStore: Send + Sync {
    fn apply<'a>(
        &'a self,
        write_set: &'a CellControlOperationWriteSetV1,
    ) -> BoxCellFuture<'a, Result<CellControlOperationV1, PlacementContractError>>;

    fn get<'a>(
        &'a self,
        authority: &'a CellControlReadAuthorityV1,
        key: &'a CellControlOperationKeyV1,
    ) -> BoxCellFuture<'a, Result<Option<CellControlOperationV1>, PlacementContractError>>;

    fn get_idempotent<'a>(
        &'a self,
        authority: &'a CellControlReadAuthorityV1,
        subject: &'a CellControlSubjectV1,
        key: &'a PlacementIdempotencyKey,
    ) -> BoxCellFuture<'a, Result<Option<CellControlIdempotencyRecordV1>, PlacementContractError>>;
}
