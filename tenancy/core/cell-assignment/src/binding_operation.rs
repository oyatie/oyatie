use cell_placement::{SignedBindingOutcomeV1, SignedSourceReservationReleasePermitV1};

use crate::{BindingDigest32, BindingProofConstructionError, BindingRepairAppliedV1, TenantId};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BindingOperationStateV1 {
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
pub struct BindingOperationId(String);

impl BindingOperationId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parse(_value: impl Into<String>) -> Result<Self, BindingProofConstructionError> {
        Err(BindingProofConstructionError::NotImplemented)
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BindingOperationKey {
    pub tenant_id: TenantId,
    pub operation_id: BindingOperationId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BindingOperationRevision(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingOperationPreconditionV1 {
    Absent,
    Matches(BindingOperationRevision),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BindingCancellationDispositionV1 {
    NotRequested,
    StoppedBeforeEffect,
    Compensating,
    RefusedForwardRecoveryRequired,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BindingRetryClassificationV1 {
    Transient,
    Backpressure,
    Dependency,
    OperatorRequired,
    ForwardRecovery,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingRetryStatusV1 {
    pub attempt: u32,
    pub maximum_attempts: u32,
    pub classification: BindingRetryClassificationV1,
    pub next_attempt_not_before_unix_seconds: u64,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BindingEffectBoundaryV1 {
    NoEffect,
    ReservationArmed,
    ReversiblePreparation,
    SourceAuthorityAllocated,
    WriteFenceCommittedForwardOnly,
    BindingCommitted,
    Converging,
    Released,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BindingCheckpointKindV1 {
    AssuranceCompiled,
    ReservationArmed,
    ParticipantManifestCommitted,
    TransferAuthorizationSealed,
    InitialCopyCompleted,
    ParticipantsPrepared,
    MigrationFenceClaimed,
    SourceFenceDirectivesIssued,
    SourceFenced,
    FinalDeltaApplied,
    BindingCommitted,
    WriteAuthorityLeasePublished,
    TargetActivated,
    RoutingConverged,
    SourceReleased,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingDurableCheckpointV1 {
    pub kind: BindingCheckpointKindV1,
    pub revision: u64,
    pub checkpoint_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingOperationProgressV1 {
    pub prepared_participants: u64,
    pub required_participants: u64,
    pub copied_bytes: u64,
    pub total_bytes: u64,
    pub checkpoint: Option<BindingDurableCheckpointV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BindingOperationSuccessV1 {
    BindingCommitted {
        binding_digest: BindingDigest32,
        outcome: Box<SignedBindingOutcomeV1>,
    },
    BindingOutcomeAborted(Box<SignedBindingOutcomeV1>),
    MigrationReleased(Box<SignedSourceReservationReleasePermitV1>),
    CancelledBeforeEffect,
    RepairApplied(Box<BindingRepairAppliedV1>),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BindingRefusalCodeV1 {
    AlreadyBound,
    NotFoundOrNotAuthorized,
    StaleGeneration,
    StaleRevision,
    SourceCellMismatch,
    PlacementProofRejected,
    ReservationPermitRejected,
    ReservationAttemptRejected,
    TransferAuthorizationRejected,
    WriteFenceRejected,
    WriteAuthorityLeaseRejected,
    WriteAuthorityLeaseFrozen,
    MigrationSealRejected,
    IdempotencyKeyReuse,
    CancellationUnsafe,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BindingFailureCodeV1 {
    DependencyUnavailable,
    IntegrityFailure,
    AuditPersistenceFailed,
    ProjectionPersistenceFailed,
    DeadlineExceeded,
    ForwardRecoveryRequired,
    OperatorInterventionRequired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BindingTerminalOutcomeV1 {
    Succeeded(Box<BindingOperationSuccessV1>),
    Refused(BindingRefusalCodeV1),
    Failed(BindingFailureCodeV1),
}

#[derive(Debug, Eq, PartialEq)]
pub struct BindingOperationV1 {
    key: BindingOperationKey,
    revision: BindingOperationRevision,
    state: BindingOperationStateV1,
    effect_boundary: BindingEffectBoundaryV1,
    cancellation: BindingCancellationDispositionV1,
    retry: Option<BindingRetryStatusV1>,
    progress: BindingOperationProgressV1,
    terminal: Option<BindingTerminalOutcomeV1>,
    record_digest: BindingDigest32,
}

impl BindingOperationV1 {
    pub fn rehydrate(
        _parts: BindingOperationPartsV1,
    ) -> Result<Self, BindingOperationConstructionError> {
        Err(BindingOperationConstructionError::NotImplemented)
    }

    #[must_use]
    pub fn key(&self) -> &BindingOperationKey {
        &self.key
    }

    #[must_use]
    pub fn revision(&self) -> BindingOperationRevision {
        self.revision
    }

    #[must_use]
    pub fn state(&self) -> BindingOperationStateV1 {
        self.state
    }

    #[must_use]
    pub fn effect_boundary(&self) -> BindingEffectBoundaryV1 {
        self.effect_boundary
    }

    #[must_use]
    pub fn cancellation(&self) -> BindingCancellationDispositionV1 {
        self.cancellation
    }

    #[must_use]
    pub fn retry(&self) -> Option<&BindingRetryStatusV1> {
        self.retry.as_ref()
    }

    #[must_use]
    pub fn progress(&self) -> &BindingOperationProgressV1 {
        &self.progress
    }

    #[must_use]
    pub fn terminal(&self) -> Option<&BindingTerminalOutcomeV1> {
        self.terminal.as_ref()
    }

    #[must_use]
    pub fn record_digest(&self) -> BindingDigest32 {
        self.record_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingOperationPartsV1 {
    pub key: BindingOperationKey,
    pub revision: BindingOperationRevision,
    pub state: BindingOperationStateV1,
    pub effect_boundary: BindingEffectBoundaryV1,
    pub cancellation: BindingCancellationDispositionV1,
    pub retry: Option<BindingRetryStatusV1>,
    pub progress: BindingOperationProgressV1,
    pub terminal: Option<BindingTerminalOutcomeV1>,
    pub record_digest: BindingDigest32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingOperationConstructionError {
    NotImplemented,
    InvalidState,
    InvalidRevision,
    InvalidProgress,
    TerminalMutation,
}
