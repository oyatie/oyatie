use crate::{
    Digest32, PlacementRepairAppliedV1, ProofConstructionError, ReservationStatusV1,
    SignedCellMovementPermitV1, SignedPlacementContinuationV1, SignedPlacementDecisionV1,
    SignedReservationArmReceiptV1, SignedReservationCommitPermitV1, TenantId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PlacementOperationStateV1 {
    Accepted,
    Validating,
    Queued,
    Running,
    WaitingForReconciler,
    Succeeded,
    Failed,
    CancelRequested,
    Cancelled,
    Compensating,
    RolledBack,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PlacementOperationId(String);

impl PlacementOperationId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parse(_value: impl Into<String>) -> Result<Self, ProofConstructionError> {
        Err(ProofConstructionError::NotImplemented)
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PlacementOperationKey {
    pub tenant_id: TenantId,
    pub operation_id: PlacementOperationId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PlacementOperationRevision(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlacementOperationPreconditionV1 {
    Absent,
    Matches(PlacementOperationRevision),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OperationEffectBoundaryV1 {
    NoEffect,
    Reversible,
    AwaitingBindingOutcome,
    IrreversibleForwardOnly,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CancellationDispositionV1 {
    NotRequested,
    StoppedBeforeEffect,
    Compensating,
    RefusedForwardRecoveryRequired,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RetryClassificationV1 {
    Transient,
    Backpressure,
    Dependency,
    OperatorRequired,
    ForwardRecovery,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetryStatusV1 {
    pub attempt: u32,
    pub maximum_attempts: u32,
    pub classification: RetryClassificationV1,
    pub next_attempt_not_before_unix_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlacementProgressV1 {
    pub completed_work_units: u64,
    pub total_work_units: u64,
    pub evaluated_partitions: u64,
    pub evaluated_windows: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlacementOperationSuccessV1 {
    PlacementSelected(Box<SignedPlacementDecisionV1>),
    ReservationArmed(Box<SignedReservationArmReceiptV1>),
    ReservationCommitPermitIssued(Box<SignedReservationCommitPermitV1>),
    MovementScheduled(Box<SignedCellMovementPermitV1>),
    BindingOutcomeApplied(Box<ReservationStatusV1>),
    SourceReservationReleased(Box<ReservationStatusV1>),
    CancelledBeforeEffect,
    RepairApplied(Box<PlacementRepairAppliedV1>),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PlacementRefusalCodeV1 {
    NoCapacity,
    AssuranceUnsatisfiable,
    IdempotencyKeyReuse,
    StaleReservationTerm,
    OutcomeRejected,
    CancellationUnsafe,
    NotFoundOrNotAuthorized,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PlacementFailureCodeV1 {
    DependencyUnavailable,
    IntegrityFailure,
    DeadlineExceeded,
    ForwardRecoveryRequired,
    OperatorInterventionRequired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlacementTerminalOutcomeV1 {
    Succeeded(Box<PlacementOperationSuccessV1>),
    Refused(PlacementRefusalCodeV1),
    Failed(PlacementFailureCodeV1),
}

#[derive(Debug, Eq, PartialEq)]
pub struct PlacementOperationV1 {
    key: PlacementOperationKey,
    revision: PlacementOperationRevision,
    state: PlacementOperationStateV1,
    effect_boundary: OperationEffectBoundaryV1,
    cancellation: CancellationDispositionV1,
    retry: Option<RetryStatusV1>,
    progress: PlacementProgressV1,
    continuation: Option<SignedPlacementContinuationV1>,
    terminal: Option<PlacementTerminalOutcomeV1>,
    durable_checkpoint_digest: Digest32,
    updated_at_unix_seconds: u64,
    record_digest: Digest32,
}

impl PlacementOperationV1 {
    pub fn rehydrate(
        _parts: PlacementOperationPartsV1,
    ) -> Result<Self, OperationConstructionError> {
        Err(OperationConstructionError::NotImplemented)
    }

    #[must_use]
    pub fn key(&self) -> &PlacementOperationKey {
        &self.key
    }

    #[must_use]
    pub fn revision(&self) -> PlacementOperationRevision {
        self.revision
    }

    #[must_use]
    pub fn state(&self) -> PlacementOperationStateV1 {
        self.state
    }

    #[must_use]
    pub fn effect_boundary(&self) -> OperationEffectBoundaryV1 {
        self.effect_boundary
    }

    #[must_use]
    pub fn cancellation(&self) -> CancellationDispositionV1 {
        self.cancellation
    }

    #[must_use]
    pub fn retry(&self) -> Option<&RetryStatusV1> {
        self.retry.as_ref()
    }

    #[must_use]
    pub fn progress(&self) -> &PlacementProgressV1 {
        &self.progress
    }

    #[must_use]
    pub fn continuation(&self) -> Option<&SignedPlacementContinuationV1> {
        self.continuation.as_ref()
    }

    #[must_use]
    pub fn terminal(&self) -> Option<&PlacementTerminalOutcomeV1> {
        self.terminal.as_ref()
    }

    #[must_use]
    pub fn durable_checkpoint_digest(&self) -> Digest32 {
        self.durable_checkpoint_digest
    }

    #[must_use]
    pub fn updated_at_unix_seconds(&self) -> u64 {
        self.updated_at_unix_seconds
    }

    #[must_use]
    pub fn record_digest(&self) -> Digest32 {
        self.record_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlacementOperationPartsV1 {
    pub key: PlacementOperationKey,
    pub revision: PlacementOperationRevision,
    pub state: PlacementOperationStateV1,
    pub effect_boundary: OperationEffectBoundaryV1,
    pub cancellation: CancellationDispositionV1,
    pub retry: Option<RetryStatusV1>,
    pub progress: PlacementProgressV1,
    pub continuation: Option<SignedPlacementContinuationV1>,
    pub terminal: Option<PlacementTerminalOutcomeV1>,
    pub durable_checkpoint_digest: Digest32,
    pub updated_at_unix_seconds: u64,
    pub record_digest: Digest32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationConstructionError {
    NotImplemented,
    InvalidState,
    InvalidRevision,
    InvalidProgress,
    TerminalMutation,
}
