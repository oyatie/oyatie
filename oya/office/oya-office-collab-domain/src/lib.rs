#![forbid(unsafe_code)]
//! CRDT/op-log, presence, session, snapshot, replay, and collaboration correctness domain.
//!
//! This crate models the collaboration vertical-slice boundary required by the
//! approved Oya Office technical spec.

use oya_office_kernel::{CellId, ObjectId, PrincipalId, TenantId};

/// Stable crate identifier used by workspace and Buck2 scaffold verification.
pub const CRATE_NAME: &str = "oya-office-collab-domain";

/// Product vertical slice owned by this crate.
pub const VERTICAL_SLICE: &str = "collab";

/// Source-shaped architectural layer represented by this crate.
pub const ARCHITECTURE_LAYER: &str = "domain";

/// Version for the collaboration runtime contract schema.
pub const COLLAB_RUNTIME_SCHEMA_VERSION: u32 = 1;

/// Supported provider-neutral CRDT payload format version.
pub const SUPPORTED_CRDT_FORMAT_VERSION: u32 = 1;

const MIN_CRDT_BYTES: u64 = 1;

/// Collaboration-domain validation error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CollabError {
    /// Snapshot id is empty.
    InvalidSnapshotId,
    /// Snapshot storage key is empty.
    InvalidSnapshotStorageKey,
    /// Snapshot state hash is empty.
    InvalidStateHash,
    /// State-vector hash is empty.
    InvalidStateVectorHash,
    /// CRDT payload format version is unsupported.
    InvalidFormatVersion,
    /// Snapshot payload length is zero.
    EmptySnapshot,
    /// State-vector payload length is zero.
    EmptyStateVector,
    /// Operation id is empty.
    InvalidOperationId,
    /// Replica id is empty.
    InvalidReplicaId,
    /// Operation hash is empty.
    InvalidOperationHash,
    /// Operation payload length is zero.
    EmptyOperation,
    /// Operation sequence is not the next accepted sequence.
    InvalidOperationSequence,
    /// State vector precedes the snapshot it claims to summarize.
    StaleStateVector,
    /// State-vector hash or version does not match the expected replay base.
    StateVectorMismatch,
    /// Operation belongs to a different Drive object.
    OperationObjectMismatch,
    /// Operation belongs to a different tenant.
    OperationTenantMismatch,
    /// Operation belongs to a different editor surface.
    OperationSurfaceMismatch,
    /// Snapshot belongs to a different Drive object.
    SnapshotObjectMismatch,
    /// Snapshot belongs to a different tenant.
    SnapshotTenantMismatch,
    /// Snapshot belongs to a different editor surface.
    SnapshotSurfaceMismatch,
    /// Replay output does not match the declared final state vector.
    ReplayFinalStateMismatch,
    /// Presence state belongs to a different Drive object.
    AwarenessObjectMismatch,
    /// Presence state belongs to a different tenant.
    AwarenessTenantMismatch,
    /// Presence state belongs to a different editor surface.
    AwarenessSurfaceMismatch,
    /// Presence replica id is empty.
    InvalidAwarenessReplicaId,
    /// Presence session id is empty.
    InvalidSessionId,
    /// Presence expiry is not after observation time.
    InvalidAwarenessExpiry,
    /// Updated timestamp precedes created timestamp.
    InvalidTimeOrder,
    /// Load-harness scenario is missing a required bound or signal.
    InvalidLoadHarnessScenario,
    /// Latency samples are empty.
    EmptyLatencySamples,
    /// Percentile budget is not monotonic.
    InvalidPercentileBudget,
}

impl core::fmt::Display for CollabError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for CollabError {}

/// Product surface whose collaboration state is being synchronized.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CollabSurface {
    /// Oya Docs document editing.
    Docs,
    /// Oya Sheets workbook editing.
    Sheets,
    /// Oya Slides presentation editing.
    Slides,
}

/// Ephemeral awareness/presence status.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AwarenessStatus {
    /// Actor can see the object but is not actively editing.
    Viewing,
    /// Actor is actively editing.
    Editing,
    /// Actor session is present but idle.
    Idle,
}

/// Tenant/object-bound reference to a durable CRDT snapshot payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollabSnapshotRef {
    tenant_id: TenantId,
    object_id: ObjectId,
    surface: CollabSurface,
    snapshot_id: String,
    storage_key: String,
    state_hash: String,
    state_vector_hash: String,
    format_version: u32,
    last_operation_sequence: u64,
    byte_len: u64,
}

impl CollabSnapshotRef {
    /// Creates a validated snapshot reference.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: TenantId,
        object_id: ObjectId,
        surface: CollabSurface,
        snapshot_id: String,
        storage_key: String,
        state_hash: String,
        state_vector_hash: String,
        format_version: u32,
        last_operation_sequence: u64,
        byte_len: u64,
    ) -> Result<Self, CollabError> {
        validate_non_empty(&snapshot_id, CollabError::InvalidSnapshotId)?;
        validate_non_empty(&storage_key, CollabError::InvalidSnapshotStorageKey)?;
        validate_non_empty(&state_hash, CollabError::InvalidStateHash)?;
        validate_non_empty(&state_vector_hash, CollabError::InvalidStateVectorHash)?;
        validate_format_version(format_version)?;
        if byte_len < MIN_CRDT_BYTES {
            return Err(CollabError::EmptySnapshot);
        }

        Ok(Self {
            tenant_id,
            object_id,
            surface,
            snapshot_id,
            storage_key,
            state_hash,
            state_vector_hash,
            format_version,
            last_operation_sequence,
            byte_len,
        })
    }

    /// Returns the tenant that owns the snapshot.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns the Drive object represented by this snapshot.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    /// Returns the product surface represented by this snapshot.
    #[must_use]
    pub const fn surface(&self) -> CollabSurface {
        self.surface
    }

    /// Returns the snapshot identifier.
    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        self.snapshot_id.as_str()
    }

    /// Returns the provider-neutral storage key.
    #[must_use]
    pub fn storage_key(&self) -> &str {
        self.storage_key.as_str()
    }

    /// Returns the state hash.
    #[must_use]
    pub fn state_hash(&self) -> &str {
        self.state_hash.as_str()
    }

    /// Returns the state-vector hash at snapshot time.
    #[must_use]
    pub fn state_vector_hash(&self) -> &str {
        self.state_vector_hash.as_str()
    }

    /// Returns the CRDT payload format version.
    #[must_use]
    pub const fn format_version(&self) -> u32 {
        self.format_version
    }

    /// Returns the last operation sequence included in the snapshot.
    #[must_use]
    pub const fn last_operation_sequence(&self) -> u64 {
        self.last_operation_sequence
    }

    /// Returns the snapshot byte length.
    #[must_use]
    pub const fn byte_len(&self) -> u64 {
        self.byte_len
    }
}

/// Tenant/object-bound reference to a CRDT state vector.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollabStateVectorRef {
    tenant_id: TenantId,
    object_id: ObjectId,
    surface: CollabSurface,
    state_vector_hash: String,
    format_version: u32,
    last_operation_sequence: u64,
    byte_len: u64,
}

impl CollabStateVectorRef {
    /// Creates a validated state-vector reference.
    pub fn new(
        tenant_id: TenantId,
        object_id: ObjectId,
        surface: CollabSurface,
        state_vector_hash: String,
        format_version: u32,
        last_operation_sequence: u64,
        byte_len: u64,
    ) -> Result<Self, CollabError> {
        validate_non_empty(&state_vector_hash, CollabError::InvalidStateVectorHash)?;
        validate_format_version(format_version)?;
        if byte_len < MIN_CRDT_BYTES {
            return Err(CollabError::EmptyStateVector);
        }
        Ok(Self {
            tenant_id,
            object_id,
            surface,
            state_vector_hash,
            format_version,
            last_operation_sequence,
            byte_len,
        })
    }

    /// Returns the tenant that owns the state vector.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns the Drive object represented by this state vector.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    /// Returns the product surface represented by this state vector.
    #[must_use]
    pub const fn surface(&self) -> CollabSurface {
        self.surface
    }

    /// Returns the state-vector hash.
    #[must_use]
    pub fn state_vector_hash(&self) -> &str {
        self.state_vector_hash.as_str()
    }

    /// Returns the CRDT payload format version.
    #[must_use]
    pub const fn format_version(&self) -> u32 {
        self.format_version
    }

    /// Returns the last operation sequence summarized by this state vector.
    #[must_use]
    pub const fn last_operation_sequence(&self) -> u64 {
        self.last_operation_sequence
    }

    /// Returns the state-vector byte length.
    #[must_use]
    pub const fn byte_len(&self) -> u64 {
        self.byte_len
    }
}

/// Input for creating a validated collaboration operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollabOperationCreate {
    /// Stable idempotency identifier for the operation.
    pub operation_id: String,
    /// Tenant that owns the operation.
    pub tenant_id: TenantId,
    /// Drive object being edited.
    pub object_id: ObjectId,
    /// Product surface being edited.
    pub surface: CollabSurface,
    /// Actor that authored the operation.
    pub actor_id: PrincipalId,
    /// Replica/session-local actor instance.
    pub replica_id: String,
    /// State-vector hash the operation is based on.
    pub base_state_vector_hash: String,
    /// State-vector hash produced by the operation.
    pub result_state_vector_hash: String,
    /// Stable hash of the operation payload.
    pub operation_hash: String,
    /// CRDT payload format version.
    pub format_version: u32,
    /// Monotonic per-object operation sequence.
    pub sequence: u64,
    /// Operation payload byte length.
    pub byte_len: u64,
    /// Observation timestamp in epoch milliseconds.
    pub observed_at_epoch_millis: u64,
}

/// Validated tenant/object-bound collaboration operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollabOperation {
    operation_id: String,
    tenant_id: TenantId,
    object_id: ObjectId,
    surface: CollabSurface,
    actor_id: PrincipalId,
    replica_id: String,
    base_state_vector_hash: String,
    result_state_vector_hash: String,
    operation_hash: String,
    format_version: u32,
    sequence: u64,
    byte_len: u64,
    observed_at_epoch_millis: u64,
}

impl CollabOperation {
    /// Creates a validated collaboration operation.
    pub fn new(input: CollabOperationCreate) -> Result<Self, CollabError> {
        validate_non_empty(&input.operation_id, CollabError::InvalidOperationId)?;
        validate_non_empty(&input.replica_id, CollabError::InvalidReplicaId)?;
        validate_non_empty(
            &input.base_state_vector_hash,
            CollabError::InvalidStateVectorHash,
        )?;
        validate_non_empty(
            &input.result_state_vector_hash,
            CollabError::InvalidStateVectorHash,
        )?;
        validate_non_empty(&input.operation_hash, CollabError::InvalidOperationHash)?;
        validate_format_version(input.format_version)?;
        if input.sequence == 0 {
            return Err(CollabError::InvalidOperationSequence);
        }
        if input.byte_len < MIN_CRDT_BYTES {
            return Err(CollabError::EmptyOperation);
        }

        Ok(Self {
            operation_id: input.operation_id,
            tenant_id: input.tenant_id,
            object_id: input.object_id,
            surface: input.surface,
            actor_id: input.actor_id,
            replica_id: input.replica_id,
            base_state_vector_hash: input.base_state_vector_hash,
            result_state_vector_hash: input.result_state_vector_hash,
            operation_hash: input.operation_hash,
            format_version: input.format_version,
            sequence: input.sequence,
            byte_len: input.byte_len,
            observed_at_epoch_millis: input.observed_at_epoch_millis,
        })
    }

    /// Returns the operation identifier.
    #[must_use]
    pub fn operation_id(&self) -> &str {
        self.operation_id.as_str()
    }

    /// Returns the tenant that owns the operation.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns the Drive object being edited.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    /// Returns the product surface being edited.
    #[must_use]
    pub const fn surface(&self) -> CollabSurface {
        self.surface
    }

    /// Returns the actor that authored the operation.
    #[must_use]
    pub const fn actor_id(&self) -> &PrincipalId {
        &self.actor_id
    }

    /// Returns the replica identifier.
    #[must_use]
    pub fn replica_id(&self) -> &str {
        self.replica_id.as_str()
    }

    /// Returns the base state-vector hash.
    #[must_use]
    pub fn base_state_vector_hash(&self) -> &str {
        self.base_state_vector_hash.as_str()
    }

    /// Returns the result state-vector hash.
    #[must_use]
    pub fn result_state_vector_hash(&self) -> &str {
        self.result_state_vector_hash.as_str()
    }

    /// Returns the operation payload hash.
    #[must_use]
    pub fn operation_hash(&self) -> &str {
        self.operation_hash.as_str()
    }

    /// Returns the CRDT payload format version.
    #[must_use]
    pub const fn format_version(&self) -> u32 {
        self.format_version
    }

    /// Returns the operation sequence.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Returns the operation payload byte length.
    #[must_use]
    pub const fn byte_len(&self) -> u64 {
        self.byte_len
    }

    /// Returns the observation timestamp in epoch milliseconds.
    #[must_use]
    pub const fn observed_at_epoch_millis(&self) -> u64 {
        self.observed_at_epoch_millis
    }

    /// Validates that this operation is the next operation after a snapshot.
    pub fn validate_next_after_snapshot(
        &self,
        snapshot: &CollabSnapshotRef,
    ) -> Result<(), CollabError> {
        validate_operation_binding(
            &self.tenant_id,
            &self.object_id,
            self.surface,
            snapshot.tenant_id(),
            snapshot.object_id(),
            snapshot.surface(),
        )?;
        validate_next_vector_step(
            self.format_version,
            self.sequence,
            self.base_state_vector_hash(),
            snapshot.format_version(),
            snapshot.last_operation_sequence(),
            snapshot.state_vector_hash(),
        )
    }

    /// Validates that this operation is the next operation after a state vector.
    pub fn validate_next_after_state_vector(
        &self,
        state_vector: &CollabStateVectorRef,
    ) -> Result<(), CollabError> {
        validate_operation_binding(
            &self.tenant_id,
            &self.object_id,
            self.surface,
            state_vector.tenant_id(),
            state_vector.object_id(),
            state_vector.surface(),
        )?;
        validate_next_vector_step(
            self.format_version,
            self.sequence,
            self.base_state_vector_hash(),
            state_vector.format_version(),
            state_vector.last_operation_sequence(),
            state_vector.state_vector_hash(),
        )
    }
}

/// Idempotency outcome when a delivered operation is compared with a receipt.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CollabIdempotencyDecision {
    /// Operation has not been accepted by this receipt and should be evaluated.
    Apply,
    /// Operation was already accepted and should not be applied again.
    DuplicateNoop,
    /// Operation reuses an id with conflicting content.
    RejectConflict,
}

/// Receipt for an accepted operation used to classify duplicate delivery.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollabOperationReceipt {
    tenant_id: TenantId,
    object_id: ObjectId,
    surface: CollabSurface,
    operation_id: String,
    operation_hash: String,
    result_state_vector_hash: String,
    sequence: u64,
}

impl CollabOperationReceipt {
    /// Creates an idempotency receipt from an accepted operation.
    #[must_use]
    pub fn from_operation(operation: &CollabOperation) -> Self {
        Self {
            tenant_id: operation.tenant_id.clone(),
            object_id: operation.object_id.clone(),
            surface: operation.surface,
            operation_id: operation.operation_id.clone(),
            operation_hash: operation.operation_hash.clone(),
            result_state_vector_hash: operation.result_state_vector_hash.clone(),
            sequence: operation.sequence,
        }
    }

    /// Classifies a delivered operation against this accepted-operation receipt.
    #[must_use]
    pub fn classify_delivery(&self, operation: &CollabOperation) -> CollabIdempotencyDecision {
        if self.tenant_id != operation.tenant_id
            || self.object_id != operation.object_id
            || self.surface != operation.surface
            || self.operation_id != operation.operation_id
        {
            return CollabIdempotencyDecision::Apply;
        }

        if self.operation_hash == operation.operation_hash
            && self.result_state_vector_hash == operation.result_state_vector_hash
            && self.sequence == operation.sequence
        {
            CollabIdempotencyDecision::DuplicateNoop
        } else {
            CollabIdempotencyDecision::RejectConflict
        }
    }
}

/// Input for creating a validated runtime head.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollabRuntimeCreate {
    /// Tenant that owns this runtime head.
    pub tenant_id: TenantId,
    /// Drive object being edited.
    pub object_id: ObjectId,
    /// Serving cell for routing and noisy-neighbor isolation.
    pub cell_id: CellId,
    /// Product surface being edited.
    pub surface: CollabSurface,
    /// Latest durable snapshot reference.
    pub snapshot: CollabSnapshotRef,
    /// Latest state-vector reference.
    pub state_vector: CollabStateVectorRef,
    /// Active ephemeral awareness states.
    pub active_awareness: Vec<AwarenessState>,
    /// Creation timestamp in epoch milliseconds.
    pub created_at_epoch_millis: u64,
    /// Last update timestamp in epoch milliseconds.
    pub updated_at_epoch_millis: u64,
}

/// Current validated runtime head for a tenant/object collaboration session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollabRuntimeHead {
    tenant_id: TenantId,
    object_id: ObjectId,
    cell_id: CellId,
    surface: CollabSurface,
    snapshot: CollabSnapshotRef,
    state_vector: CollabStateVectorRef,
    active_awareness: Vec<AwarenessState>,
    created_at_epoch_millis: u64,
    updated_at_epoch_millis: u64,
    schema_version: u32,
}

impl CollabRuntimeHead {
    /// Creates a validated runtime head.
    pub fn new(input: CollabRuntimeCreate) -> Result<Self, CollabError> {
        validate_time_order(input.created_at_epoch_millis, input.updated_at_epoch_millis)?;
        validate_snapshot_state_vector(&input.snapshot, &input.state_vector)?;
        validate_snapshot_binding(
            &input.tenant_id,
            &input.object_id,
            input.surface,
            input.snapshot.tenant_id(),
            input.snapshot.object_id(),
            input.snapshot.surface(),
        )?;
        validate_snapshot_binding(
            &input.tenant_id,
            &input.object_id,
            input.surface,
            input.state_vector.tenant_id(),
            input.state_vector.object_id(),
            input.state_vector.surface(),
        )?;
        for awareness in &input.active_awareness {
            awareness.validate_for_object(&input.tenant_id, &input.object_id, input.surface)?;
        }

        Ok(Self {
            tenant_id: input.tenant_id,
            object_id: input.object_id,
            cell_id: input.cell_id,
            surface: input.surface,
            snapshot: input.snapshot,
            state_vector: input.state_vector,
            active_awareness: input.active_awareness,
            created_at_epoch_millis: input.created_at_epoch_millis,
            updated_at_epoch_millis: input.updated_at_epoch_millis,
            schema_version: COLLAB_RUNTIME_SCHEMA_VERSION,
        })
    }

    /// Returns the runtime schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// Returns the tenant that owns the runtime.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns the Drive object being edited.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    /// Returns the serving cell.
    #[must_use]
    pub const fn cell_id(&self) -> &CellId {
        &self.cell_id
    }

    /// Returns the surface being edited.
    #[must_use]
    pub const fn surface(&self) -> CollabSurface {
        self.surface
    }

    /// Returns the latest durable snapshot reference.
    #[must_use]
    pub const fn snapshot(&self) -> &CollabSnapshotRef {
        &self.snapshot
    }

    /// Returns the latest state-vector reference.
    #[must_use]
    pub const fn state_vector(&self) -> &CollabStateVectorRef {
        &self.state_vector
    }

    /// Returns active ephemeral awareness states.
    #[must_use]
    pub fn active_awareness(&self) -> &[AwarenessState] {
        self.active_awareness.as_slice()
    }

    /// Returns the creation timestamp in epoch milliseconds.
    #[must_use]
    pub const fn created_at_epoch_millis(&self) -> u64 {
        self.created_at_epoch_millis
    }

    /// Returns the updated timestamp in epoch milliseconds.
    #[must_use]
    pub const fn updated_at_epoch_millis(&self) -> u64 {
        self.updated_at_epoch_millis
    }

    /// Validates that an operation is the next accepted operation for this runtime.
    pub fn validate_next_operation(&self, operation: &CollabOperation) -> Result<(), CollabError> {
        operation.validate_next_after_state_vector(&self.state_vector)
    }
}

/// Validated replay plan from a snapshot through operations to a state vector.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollabReplayPlan {
    snapshot: CollabSnapshotRef,
    operations: Vec<CollabOperation>,
    final_state_vector: CollabStateVectorRef,
}

impl CollabReplayPlan {
    /// Creates a replay plan and validates the operation hash/sequence chain.
    pub fn new(
        snapshot: CollabSnapshotRef,
        operations: Vec<CollabOperation>,
        final_state_vector: CollabStateVectorRef,
    ) -> Result<Self, CollabError> {
        validate_snapshot_state_vector(&snapshot, &final_state_vector)?;

        let mut expected_sequence = snapshot
            .last_operation_sequence()
            .checked_add(1)
            .ok_or(CollabError::InvalidOperationSequence)?;
        let mut expected_base_hash = snapshot.state_vector_hash().to_owned();

        if operations.is_empty() {
            if final_state_vector.last_operation_sequence() == snapshot.last_operation_sequence()
                && final_state_vector.state_vector_hash() == snapshot.state_vector_hash()
            {
                return Ok(Self {
                    snapshot,
                    operations,
                    final_state_vector,
                });
            }
            return Err(CollabError::ReplayFinalStateMismatch);
        }

        for operation in &operations {
            validate_operation_binding(
                operation.tenant_id(),
                operation.object_id(),
                operation.surface(),
                snapshot.tenant_id(),
                snapshot.object_id(),
                snapshot.surface(),
            )?;
            if operation.format_version() != snapshot.format_version() {
                return Err(CollabError::StateVectorMismatch);
            }
            if operation.sequence() != expected_sequence {
                return Err(CollabError::InvalidOperationSequence);
            }
            if operation.base_state_vector_hash() != expected_base_hash {
                return Err(CollabError::StateVectorMismatch);
            }
            expected_sequence = expected_sequence
                .checked_add(1)
                .ok_or(CollabError::InvalidOperationSequence)?;
            expected_base_hash = operation.result_state_vector_hash().to_owned();
        }

        let last_sequence = expected_sequence
            .checked_sub(1)
            .ok_or(CollabError::InvalidOperationSequence)?;
        if final_state_vector.last_operation_sequence() != last_sequence
            || final_state_vector.state_vector_hash() != expected_base_hash
        {
            return Err(CollabError::ReplayFinalStateMismatch);
        }

        Ok(Self {
            snapshot,
            operations,
            final_state_vector,
        })
    }

    /// Returns the snapshot at the start of replay.
    #[must_use]
    pub const fn snapshot(&self) -> &CollabSnapshotRef {
        &self.snapshot
    }

    /// Returns the ordered operations.
    #[must_use]
    pub fn operations(&self) -> &[CollabOperation] {
        self.operations.as_slice()
    }

    /// Returns the final state vector.
    #[must_use]
    pub const fn final_state_vector(&self) -> &CollabStateVectorRef {
        &self.final_state_vector
    }

    /// Returns the number of replayed operations.
    #[must_use]
    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }

    /// Returns the last operation sequence represented by the replay plan.
    #[must_use]
    pub const fn last_operation_sequence(&self) -> u64 {
        self.final_state_vector.last_operation_sequence()
    }
}

/// Input for creating an ephemeral awareness/presence state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AwarenessStateCreate {
    /// Tenant that owns the awareness state.
    pub tenant_id: TenantId,
    /// Drive object being viewed or edited.
    pub object_id: ObjectId,
    /// Product surface being viewed or edited.
    pub surface: CollabSurface,
    /// Actor represented by this awareness state.
    pub actor_id: PrincipalId,
    /// Replica/session-local actor instance.
    pub replica_id: String,
    /// Session identifier.
    pub session_id: String,
    /// Presence status.
    pub status: AwarenessStatus,
    /// Optional cursor or selection anchor.
    pub cursor_anchor: Option<String>,
    /// Observation timestamp in epoch milliseconds.
    pub observed_at_epoch_millis: u64,
    /// Expiry timestamp in epoch milliseconds.
    pub expires_at_epoch_millis: u64,
}

/// Ephemeral awareness/presence state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AwarenessState {
    tenant_id: TenantId,
    object_id: ObjectId,
    surface: CollabSurface,
    actor_id: PrincipalId,
    replica_id: String,
    session_id: String,
    status: AwarenessStatus,
    cursor_anchor: Option<String>,
    observed_at_epoch_millis: u64,
    expires_at_epoch_millis: u64,
}

impl AwarenessState {
    /// Creates a validated awareness state.
    pub fn new(input: AwarenessStateCreate) -> Result<Self, CollabError> {
        validate_non_empty(&input.replica_id, CollabError::InvalidAwarenessReplicaId)?;
        validate_non_empty(&input.session_id, CollabError::InvalidSessionId)?;
        if input.expires_at_epoch_millis <= input.observed_at_epoch_millis {
            return Err(CollabError::InvalidAwarenessExpiry);
        }
        Ok(Self {
            tenant_id: input.tenant_id,
            object_id: input.object_id,
            surface: input.surface,
            actor_id: input.actor_id,
            replica_id: input.replica_id,
            session_id: input.session_id,
            status: input.status,
            cursor_anchor: input.cursor_anchor,
            observed_at_epoch_millis: input.observed_at_epoch_millis,
            expires_at_epoch_millis: input.expires_at_epoch_millis,
        })
    }

    /// Validates that awareness belongs to the provided tenant/object/surface.
    pub fn validate_for_object(
        &self,
        tenant_id: &TenantId,
        object_id: &ObjectId,
        surface: CollabSurface,
    ) -> Result<(), CollabError> {
        if &self.tenant_id != tenant_id {
            return Err(CollabError::AwarenessTenantMismatch);
        }
        if &self.object_id != object_id {
            return Err(CollabError::AwarenessObjectMismatch);
        }
        if self.surface != surface {
            return Err(CollabError::AwarenessSurfaceMismatch);
        }
        validate_non_empty(&self.replica_id, CollabError::InvalidAwarenessReplicaId)?;
        validate_non_empty(&self.session_id, CollabError::InvalidSessionId)?;
        if self.expires_at_epoch_millis <= self.observed_at_epoch_millis {
            return Err(CollabError::InvalidAwarenessExpiry);
        }
        Ok(())
    }

    /// Returns true when this awareness state has expired at `epoch_millis`.
    #[must_use]
    pub const fn is_expired_at(&self, epoch_millis: u64) -> bool {
        epoch_millis >= self.expires_at_epoch_millis
    }

    /// Returns the tenant that owns the awareness state.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns the Drive object represented by awareness.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    /// Returns the product surface represented by awareness.
    #[must_use]
    pub const fn surface(&self) -> CollabSurface {
        self.surface
    }

    /// Returns the actor represented by awareness.
    #[must_use]
    pub const fn actor_id(&self) -> &PrincipalId {
        &self.actor_id
    }

    /// Returns the replica identifier.
    #[must_use]
    pub fn replica_id(&self) -> &str {
        self.replica_id.as_str()
    }

    /// Returns the session identifier.
    #[must_use]
    pub fn session_id(&self) -> &str {
        self.session_id.as_str()
    }

    /// Returns the awareness status.
    #[must_use]
    pub const fn status(&self) -> AwarenessStatus {
        self.status
    }

    /// Returns the optional cursor/selection anchor.
    #[must_use]
    pub fn cursor_anchor(&self) -> Option<&str> {
        self.cursor_anchor.as_deref()
    }

    /// Returns the observation timestamp in epoch milliseconds.
    #[must_use]
    pub const fn observed_at_epoch_millis(&self) -> u64 {
        self.observed_at_epoch_millis
    }

    /// Returns the expiry timestamp in epoch milliseconds.
    #[must_use]
    pub const fn expires_at_epoch_millis(&self) -> u64 {
        self.expires_at_epoch_millis
    }
}

/// Provider-neutral persistence port for snapshots and op-log appends.
pub trait CollabPersistencePort {
    /// Loads the latest snapshot for a tenant/object/surface.
    fn load_snapshot(
        &self,
        tenant_id: &TenantId,
        object_id: &ObjectId,
        surface: CollabSurface,
    ) -> Result<Option<CollabSnapshotRef>, CollabError>;

    /// Appends an operation and returns its idempotency receipt.
    fn append_operation(
        &self,
        operation: &CollabOperation,
    ) -> Result<CollabOperationReceipt, CollabError>;
}

/// Provider-neutral access-control port for collaboration reads and writes.
pub trait CollabAccessControlPort {
    /// Returns whether an actor may read collaboration state.
    fn can_read(
        &self,
        tenant_id: &TenantId,
        object_id: &ObjectId,
        actor_id: &PrincipalId,
    ) -> Result<bool, CollabError>;

    /// Returns whether an actor may append collaboration operations.
    fn can_write(
        &self,
        tenant_id: &TenantId,
        object_id: &ObjectId,
        actor_id: &PrincipalId,
    ) -> Result<bool, CollabError>;
}

/// Collaboration load-harness metric tracked with percentile SLOs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CollabLoadMetric {
    /// End-to-end operation propagation from accepted append to remote visibility.
    Propagation,
    /// Save durability from editor commit to durable snapshot/op-log acknowledgement.
    Save,
    /// Session recovery from disconnect to a validated Drive-bound editor runtime.
    Reconnect,
    /// Opening a Drive object into an editor session with collaboration attached.
    DriveSessionOpen,
}

impl CollabLoadMetric {
    /// Returns the stable metric label used by load-harness reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Propagation => "propagation",
            Self::Save => "save",
            Self::Reconnect => "reconnect",
            Self::DriveSessionOpen => "drive-session-open",
        }
    }
}

/// SLO evaluation result for a percentile budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CollabSloDecision {
    /// All measured percentiles are within the budget.
    Pass,
    /// At least one measured percentile exceeds the budget or targets another metric.
    Fail,
}

/// Nearest-rank latency percentile summary for one load-harness metric.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollabLatencySummary {
    metric: CollabLoadMetric,
    sample_count: usize,
    p50_millis: u64,
    p95_millis: u64,
    p99_millis: u64,
}

impl CollabLatencySummary {
    /// Returns the metric represented by this summary.
    #[must_use]
    pub const fn metric(&self) -> CollabLoadMetric {
        self.metric
    }

    /// Returns the number of samples summarized.
    #[must_use]
    pub const fn sample_count(&self) -> usize {
        self.sample_count
    }

    /// Returns the p50 latency in milliseconds.
    #[must_use]
    pub const fn p50_millis(&self) -> u64 {
        self.p50_millis
    }

    /// Returns the p95 latency in milliseconds.
    #[must_use]
    pub const fn p95_millis(&self) -> u64 {
        self.p95_millis
    }

    /// Returns the p99 latency in milliseconds.
    #[must_use]
    pub const fn p99_millis(&self) -> u64 {
        self.p99_millis
    }
}

/// Latency samples captured by the collaboration load harness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollabLatencySamples {
    metric: CollabLoadMetric,
    samples_millis: Vec<u64>,
}

impl CollabLatencySamples {
    /// Creates a non-empty latency sample set for one metric.
    pub fn new(metric: CollabLoadMetric, samples_millis: Vec<u64>) -> Result<Self, CollabError> {
        if samples_millis.is_empty() {
            return Err(CollabError::EmptyLatencySamples);
        }
        Ok(Self {
            metric,
            samples_millis,
        })
    }

    /// Returns the metric represented by this sample set.
    #[must_use]
    pub const fn metric(&self) -> CollabLoadMetric {
        self.metric
    }

    /// Returns the raw samples in milliseconds.
    #[must_use]
    pub fn samples_millis(&self) -> &[u64] {
        self.samples_millis.as_slice()
    }

    /// Computes p50/p95/p99 with the nearest-rank method.
    #[must_use]
    pub fn summary(&self) -> CollabLatencySummary {
        let mut sorted = self.samples_millis.clone();
        sorted.sort_unstable();
        let sample_count = sorted.len();
        CollabLatencySummary {
            metric: self.metric,
            sample_count,
            p50_millis: percentile_nearest_rank(&sorted, 50),
            p95_millis: percentile_nearest_rank(&sorted, 95),
            p99_millis: percentile_nearest_rank(&sorted, 99),
        }
    }
}

/// Percentile SLO budget for one collaboration load metric.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CollabPercentileBudget {
    metric: CollabLoadMetric,
    p50_millis: u64,
    p95_millis: u64,
    p99_millis: u64,
}

impl CollabPercentileBudget {
    /// Creates a monotonic p50/p95/p99 budget.
    pub const fn new(
        metric: CollabLoadMetric,
        p50_millis: u64,
        p95_millis: u64,
        p99_millis: u64,
    ) -> Result<Self, CollabError> {
        if p50_millis > p95_millis || p95_millis > p99_millis {
            return Err(CollabError::InvalidPercentileBudget);
        }
        Ok(Self {
            metric,
            p50_millis,
            p95_millis,
            p99_millis,
        })
    }

    /// Returns the metric constrained by this budget.
    #[must_use]
    pub const fn metric(self) -> CollabLoadMetric {
        self.metric
    }

    /// Returns the p50 budget in milliseconds.
    #[must_use]
    pub const fn p50_millis(self) -> u64 {
        self.p50_millis
    }

    /// Returns the p95 budget in milliseconds.
    #[must_use]
    pub const fn p95_millis(self) -> u64 {
        self.p95_millis
    }

    /// Returns the p99 budget in milliseconds.
    #[must_use]
    pub const fn p99_millis(self) -> u64 {
        self.p99_millis
    }

    /// Evaluates a latency summary against this budget.
    #[must_use]
    pub fn evaluate(self, summary: &CollabLatencySummary) -> CollabSloDecision {
        if summary.metric() != self.metric
            || summary.p50_millis() > self.p50_millis
            || summary.p95_millis() > self.p95_millis
            || summary.p99_millis() > self.p99_millis
        {
            CollabSloDecision::Fail
        } else {
            CollabSloDecision::Pass
        }
    }
}

/// Drive-bound editor-session load scenario for Docs, Sheets, and Slides.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollabLoadHarnessScenario {
    name: String,
    tenant_id: TenantId,
    object_id: ObjectId,
    cell_id: CellId,
    surface: CollabSurface,
    target_active_sessions: u32,
    operation_rate_per_second: u32,
    drive_bound_editor_sessions: bool,
}

impl CollabLoadHarnessScenario {
    /// Creates a validated Drive-bound editor-session load scenario.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        tenant_id: TenantId,
        object_id: ObjectId,
        cell_id: CellId,
        surface: CollabSurface,
        target_active_sessions: u32,
        operation_rate_per_second: u32,
        drive_bound_editor_sessions: bool,
    ) -> Result<Self, CollabError> {
        let name = name.trim().to_owned();
        if name.is_empty()
            || target_active_sessions == 0
            || operation_rate_per_second == 0
            || !drive_bound_editor_sessions
        {
            return Err(CollabError::InvalidLoadHarnessScenario);
        }
        Ok(Self {
            name,
            tenant_id,
            object_id,
            cell_id,
            surface,
            target_active_sessions,
            operation_rate_per_second,
            drive_bound_editor_sessions,
        })
    }

    /// Returns the scenario name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the tenant under test.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns the Drive object under test.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    /// Returns the serving cell under test.
    #[must_use]
    pub const fn cell_id(&self) -> &CellId {
        &self.cell_id
    }

    /// Returns the product surface under test.
    #[must_use]
    pub const fn surface(&self) -> CollabSurface {
        self.surface
    }

    /// Returns the target active editor sessions.
    #[must_use]
    pub const fn target_active_sessions(&self) -> u32 {
        self.target_active_sessions
    }

    /// Returns the target operation rate per second.
    #[must_use]
    pub const fn operation_rate_per_second(&self) -> u32 {
        self.operation_rate_per_second
    }

    /// Returns whether the scenario is bound to Drive-backed editor sessions.
    #[must_use]
    pub const fn is_drive_bound(&self) -> bool {
        self.drive_bound_editor_sessions
    }
}

/// Load-harness plan for collaboration SLO verification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollabLoadHarnessPlan {
    scenarios: Vec<CollabLoadHarnessScenario>,
    budgets: Vec<CollabPercentileBudget>,
}

impl CollabLoadHarnessPlan {
    /// Creates a load-harness plan from validated scenarios and budgets.
    pub fn new(
        scenarios: Vec<CollabLoadHarnessScenario>,
        budgets: Vec<CollabPercentileBudget>,
    ) -> Result<Self, CollabError> {
        if scenarios.is_empty() || budgets.is_empty() {
            return Err(CollabError::InvalidLoadHarnessScenario);
        }
        Ok(Self { scenarios, budgets })
    }

    /// Returns the Drive-bound load scenarios.
    #[must_use]
    pub fn scenarios(&self) -> &[CollabLoadHarnessScenario] {
        self.scenarios.as_slice()
    }

    /// Returns the percentile budgets.
    #[must_use]
    pub fn budgets(&self) -> &[CollabPercentileBudget] {
        self.budgets.as_slice()
    }

    /// Returns true when the plan constrains the provided metric.
    #[must_use]
    pub fn includes_metric(&self, metric: CollabLoadMetric) -> bool {
        self.budgets.iter().any(|budget| budget.metric() == metric)
    }
}

/// Returns the default Docs/Sheets/Slides collaboration load-harness plan.
///
/// The plan is intentionally vertical-slice aware: every scenario opens a
/// Drive-bound editor session in one serving cell, while the metrics remain
/// comparable across parallel gateway, worker, and API lanes.
#[must_use]
pub fn collab_load_harness_plan() -> CollabLoadHarnessPlan {
    let scenarios = vec![
        static_load_scenario(
            "docs-drive-bound-editors",
            "tenant-load-docs",
            "object-load-docs",
            "cell-a",
            CollabSurface::Docs,
            1_000,
            3_000,
        ),
        static_load_scenario(
            "sheets-drive-bound-editors",
            "tenant-load-sheets",
            "object-load-sheets",
            "cell-b",
            CollabSurface::Sheets,
            750,
            2_250,
        ),
        static_load_scenario(
            "slides-drive-bound-editors",
            "tenant-load-slides",
            "object-load-slides",
            "cell-c",
            CollabSurface::Slides,
            500,
            1_500,
        ),
    ];
    let budgets = vec![
        static_percentile_budget(CollabLoadMetric::Propagation, 100, 300, 750),
        static_percentile_budget(CollabLoadMetric::Save, 150, 500, 1_200),
        static_percentile_budget(CollabLoadMetric::Reconnect, 500, 1_500, 3_000),
        static_percentile_budget(CollabLoadMetric::DriveSessionOpen, 400, 1_200, 2_500),
    ];
    match CollabLoadHarnessPlan::new(scenarios, budgets) {
        Ok(plan) => plan,
        Err(_) => unreachable!("static load-harness plan is valid"),
    }
}

/// Version for the CRDT/op-log dependency review contract.
pub const COLLAB_DEPENDENCY_REVIEW_SCHEMA_VERSION: u32 = 1;

/// Provider-neutral collaboration architecture surface that may own state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CollabArchitectureSeam {
    /// Domain-level op log that records tenant-scoped operation metadata.
    TenantScopedOpLog,
    /// Vendor-neutral CRDT runtime port, modeled before concrete adapters.
    CrdtRuntimePort,
    /// Snapshot/state-vector contract used by replay and recovery workers.
    SnapshotReplayPort,
    /// Ephemeral awareness/presence contract kept outside durable document state.
    PresenceAwarenessPort,
}

/// Dependency or internal seam reviewed for CRDT/op-log adoption.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CollabDependencyCandidate {
    /// Office-owned provider-neutral port and op-log seam.
    OfficePortSeam,
    /// Loro CRDT candidate adapter.
    Loro,
    /// Yrs/Yjs-compatible Rust candidate adapter.
    Yrs,
    /// Automerge candidate adapter.
    Automerge,
    /// Diamond Types candidate adapter for text/op-log research.
    DiamondTypes,
}

impl CollabDependencyCandidate {
    /// Returns the stable short name used in ADRs and scorecards.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OfficePortSeam => "oya-office-port-seam",
            Self::Loro => "loro",
            Self::Yrs => "yrs",
            Self::Automerge => "automerge",
            Self::DiamondTypes => "diamond-types",
        }
    }
}

/// Current adoption posture for a reviewed dependency candidate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CollabDependencyDisposition {
    /// Adopt the option in this story without adding a third-party runtime dependency.
    AdoptNow,
    /// Keep the option under evaluation behind a provider-neutral port.
    EvaluateBehindPort,
    /// Reject the option for the current platform wave.
    RejectForNow,
}

impl CollabDependencyDisposition {
    /// Returns the stable machine-readable disposition label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AdoptNow => "adopt-now",
            Self::EvaluateBehindPort => "evaluate-behind-port",
            Self::RejectForNow => "reject-for-now",
        }
    }
}

/// Parallel implementation lane affected by a dependency decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CollabImplementationLane {
    /// Domain contracts, invariants, and deterministic tests.
    DomainKernel,
    /// Adapter evaluation isolated from the domain crate.
    AdapterEvaluation,
    /// Realtime gateway protocol and fanout.
    RealtimeGateway,
    /// Snapshot/replay worker, retention, and recovery drills.
    SnapshotReplayWorker,
}

/// Ordered gate that must pass before the next collaboration lane advances.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CollabSequenceGate {
    /// Define the port before selecting any concrete CRDT adapter.
    PortBeforeAdapter,
    /// Accept tenant-scoped op-log validation before realtime fanout.
    OpLogBeforeRealtimeFanout,
    /// Prove snapshot/replay before load benchmarks or fleet claims.
    SnapshotReplayBeforeLoadBenchmark,
    /// Keep presence ephemeral and outside durable CRDT snapshots.
    PresenceOutsideDurableState,
}

/// One dependency-review row for the collaboration CRDT/op-log ADR.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CollabDependencyReviewEntry {
    candidate: CollabDependencyCandidate,
    disposition: CollabDependencyDisposition,
    primary_seam: CollabArchitectureSeam,
    parallel_lane: CollabImplementationLane,
    sequence_gate: CollabSequenceGate,
    upstream_reference: &'static str,
    decision_note: &'static str,
    requires_tenant_scope: bool,
    requires_buck2_supply_chain_gate: bool,
    requires_deterministic_replay_gate: bool,
    requires_memory_bound_gate: bool,
}

impl CollabDependencyReviewEntry {
    /// Creates a dependency-review entry.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        candidate: CollabDependencyCandidate,
        disposition: CollabDependencyDisposition,
        primary_seam: CollabArchitectureSeam,
        parallel_lane: CollabImplementationLane,
        sequence_gate: CollabSequenceGate,
        upstream_reference: &'static str,
        decision_note: &'static str,
    ) -> Self {
        Self {
            candidate,
            disposition,
            primary_seam,
            parallel_lane,
            sequence_gate,
            upstream_reference,
            decision_note,
            requires_tenant_scope: true,
            requires_buck2_supply_chain_gate: true,
            requires_deterministic_replay_gate: true,
            requires_memory_bound_gate: true,
        }
    }

    /// Returns the reviewed candidate.
    #[must_use]
    pub const fn candidate(self) -> CollabDependencyCandidate {
        self.candidate
    }

    /// Returns the current adoption posture.
    #[must_use]
    pub const fn disposition(self) -> CollabDependencyDisposition {
        self.disposition
    }

    /// Returns the provider-neutral seam this candidate must obey.
    #[must_use]
    pub const fn primary_seam(self) -> CollabArchitectureSeam {
        self.primary_seam
    }

    /// Returns the parallel implementation lane affected by this row.
    #[must_use]
    pub const fn parallel_lane(self) -> CollabImplementationLane {
        self.parallel_lane
    }

    /// Returns the ordered gate that protects downstream work.
    #[must_use]
    pub const fn sequence_gate(self) -> CollabSequenceGate {
        self.sequence_gate
    }

    /// Returns the official or upstream reference URL.
    #[must_use]
    pub const fn upstream_reference(self) -> &'static str {
        self.upstream_reference
    }

    /// Returns the short decision note used by docs and ADRs.
    #[must_use]
    pub const fn decision_note(self) -> &'static str {
        self.decision_note
    }

    /// Returns true when tenant scope is mandatory before adoption.
    #[must_use]
    pub const fn requires_tenant_scope(self) -> bool {
        self.requires_tenant_scope
    }

    /// Returns true when Buck2/supply-chain gates are mandatory before adoption.
    #[must_use]
    pub const fn requires_buck2_supply_chain_gate(self) -> bool {
        self.requires_buck2_supply_chain_gate
    }

    /// Returns true when deterministic replay evidence is mandatory before adoption.
    #[must_use]
    pub const fn requires_deterministic_replay_gate(self) -> bool {
        self.requires_deterministic_replay_gate
    }

    /// Returns true when memory/CPU bound evidence is mandatory before adoption.
    #[must_use]
    pub const fn requires_memory_bound_gate(self) -> bool {
        self.requires_memory_bound_gate
    }
}

/// Returns the accepted CRDT/op-log dependency review for this platform wave.
#[must_use]
pub const fn collab_dependency_review() -> &'static [CollabDependencyReviewEntry] {
    &COLLAB_DEPENDENCY_REVIEW
}

const COLLAB_DEPENDENCY_REVIEW: [CollabDependencyReviewEntry; 5] = [
    CollabDependencyReviewEntry::new(
        CollabDependencyCandidate::OfficePortSeam,
        CollabDependencyDisposition::AdoptNow,
        CollabArchitectureSeam::CrdtRuntimePort,
        CollabImplementationLane::DomainKernel,
        CollabSequenceGate::PortBeforeAdapter,
        "read-only /source oya-collab-crdt-portability-kernel",
        "Adopt the in-house provider-neutral port/op-log seam now; no third-party CRDT runtime is added in the domain crate.",
    ),
    CollabDependencyReviewEntry::new(
        CollabDependencyCandidate::Loro,
        CollabDependencyDisposition::EvaluateBehindPort,
        CollabArchitectureSeam::CrdtRuntimePort,
        CollabImplementationLane::AdapterEvaluation,
        CollabSequenceGate::PortBeforeAdapter,
        "https://github.com/loro-dev/loro",
        "Evaluate as a future adapter because source names Loro as a current candidate, but keep it outside the domain crate until Buck2, license, replay, and resource-bound gates pass.",
    ),
    CollabDependencyReviewEntry::new(
        CollabDependencyCandidate::Yrs,
        CollabDependencyDisposition::EvaluateBehindPort,
        CollabArchitectureSeam::CrdtRuntimePort,
        CollabImplementationLane::AdapterEvaluation,
        CollabSequenceGate::PortBeforeAdapter,
        "https://docs.rs/yrs/latest/yrs/",
        "Evaluate Yjs-compatible Rust interop behind the port; never let wire protocol or WebSocket concerns own domain state.",
    ),
    CollabDependencyReviewEntry::new(
        CollabDependencyCandidate::Automerge,
        CollabDependencyDisposition::EvaluateBehindPort,
        CollabArchitectureSeam::SnapshotReplayPort,
        CollabImplementationLane::AdapterEvaluation,
        CollabSequenceGate::SnapshotReplayBeforeLoadBenchmark,
        "https://automerge.org/",
        "Evaluate for local-first sync and compressed storage properties, gated by deterministic replay and tenant-scoped op-log compatibility.",
    ),
    CollabDependencyReviewEntry::new(
        CollabDependencyCandidate::DiamondTypes,
        CollabDependencyDisposition::EvaluateBehindPort,
        CollabArchitectureSeam::TenantScopedOpLog,
        CollabImplementationLane::AdapterEvaluation,
        CollabSequenceGate::OpLogBeforeRealtimeFanout,
        "https://docs.rs/diamond-types/latest/diamond_types/",
        "Evaluate for text/op-log research; not adopted as suite-wide CRDT because current docs scope it to plain text.",
    ),
];

fn percentile_nearest_rank(sorted_samples_millis: &[u64], percentile: usize) -> u64 {
    debug_assert!(!sorted_samples_millis.is_empty());
    debug_assert!((1..=100).contains(&percentile));
    let sample_count = sorted_samples_millis.len();
    let index = ((sample_count * percentile).div_ceil(100)).saturating_sub(1);
    sorted_samples_millis[index.min(sample_count - 1)]
}

#[allow(clippy::too_many_arguments)]
fn static_load_scenario(
    name: &str,
    tenant_id: &str,
    object_id: &str,
    cell_id: &str,
    surface: CollabSurface,
    target_active_sessions: u32,
    operation_rate_per_second: u32,
) -> CollabLoadHarnessScenario {
    match CollabLoadHarnessScenario::new(
        name.to_owned(),
        static_tenant_id(tenant_id),
        static_object_id(object_id),
        static_cell_id(cell_id),
        surface,
        target_active_sessions,
        operation_rate_per_second,
        true,
    ) {
        Ok(scenario) => scenario,
        Err(_) => unreachable!("static load scenario is valid"),
    }
}

fn static_percentile_budget(
    metric: CollabLoadMetric,
    p50_millis: u64,
    p95_millis: u64,
    p99_millis: u64,
) -> CollabPercentileBudget {
    match CollabPercentileBudget::new(metric, p50_millis, p95_millis, p99_millis) {
        Ok(budget) => budget,
        Err(_) => unreachable!("static percentile budget is valid"),
    }
}

fn static_tenant_id(value: &str) -> TenantId {
    match TenantId::new(value) {
        Ok(identifier) => identifier,
        Err(_) => unreachable!("static tenant id is valid"),
    }
}

fn static_object_id(value: &str) -> ObjectId {
    match ObjectId::new(value) {
        Ok(identifier) => identifier,
        Err(_) => unreachable!("static object id is valid"),
    }
}

fn static_cell_id(value: &str) -> CellId {
    match CellId::new(value) {
        Ok(identifier) => identifier,
        Err(_) => unreachable!("static cell id is valid"),
    }
}

fn validate_non_empty(value: &str, error: CollabError) -> Result<(), CollabError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_format_version(format_version: u32) -> Result<(), CollabError> {
    if format_version == SUPPORTED_CRDT_FORMAT_VERSION {
        Ok(())
    } else {
        Err(CollabError::InvalidFormatVersion)
    }
}

fn validate_time_order(created_at: u64, updated_at: u64) -> Result<(), CollabError> {
    if updated_at < created_at {
        Err(CollabError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

fn validate_snapshot_state_vector(
    snapshot: &CollabSnapshotRef,
    state_vector: &CollabStateVectorRef,
) -> Result<(), CollabError> {
    validate_snapshot_binding(
        snapshot.tenant_id(),
        snapshot.object_id(),
        snapshot.surface(),
        state_vector.tenant_id(),
        state_vector.object_id(),
        state_vector.surface(),
    )?;
    if snapshot.format_version() != state_vector.format_version() {
        return Err(CollabError::StateVectorMismatch);
    }
    if state_vector.last_operation_sequence() < snapshot.last_operation_sequence() {
        return Err(CollabError::StaleStateVector);
    }
    Ok(())
}

fn validate_snapshot_binding(
    expected_tenant_id: &TenantId,
    expected_object_id: &ObjectId,
    expected_surface: CollabSurface,
    actual_tenant_id: &TenantId,
    actual_object_id: &ObjectId,
    actual_surface: CollabSurface,
) -> Result<(), CollabError> {
    if expected_tenant_id != actual_tenant_id {
        return Err(CollabError::SnapshotTenantMismatch);
    }
    if expected_object_id != actual_object_id {
        return Err(CollabError::SnapshotObjectMismatch);
    }
    if expected_surface != actual_surface {
        return Err(CollabError::SnapshotSurfaceMismatch);
    }
    Ok(())
}

fn validate_operation_binding(
    expected_tenant_id: &TenantId,
    expected_object_id: &ObjectId,
    expected_surface: CollabSurface,
    actual_tenant_id: &TenantId,
    actual_object_id: &ObjectId,
    actual_surface: CollabSurface,
) -> Result<(), CollabError> {
    if expected_tenant_id != actual_tenant_id {
        return Err(CollabError::OperationTenantMismatch);
    }
    if expected_object_id != actual_object_id {
        return Err(CollabError::OperationObjectMismatch);
    }
    if expected_surface != actual_surface {
        return Err(CollabError::OperationSurfaceMismatch);
    }
    Ok(())
}

fn validate_next_vector_step(
    operation_format_version: u32,
    operation_sequence: u64,
    operation_base_hash: &str,
    base_format_version: u32,
    base_sequence: u64,
    base_state_vector_hash: &str,
) -> Result<(), CollabError> {
    if operation_format_version != base_format_version {
        return Err(CollabError::StateVectorMismatch);
    }
    let expected_sequence = base_sequence
        .checked_add(1)
        .ok_or(CollabError::InvalidOperationSequence)?;
    if operation_sequence != expected_sequence {
        return Err(CollabError::InvalidOperationSequence);
    }
    if operation_base_hash != base_state_vector_hash {
        return Err(CollabError::StateVectorMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        ARCHITECTURE_LAYER, AwarenessState, AwarenessStateCreate, AwarenessStatus, CRATE_NAME,
        CollabDependencyCandidate, CollabDependencyDisposition, CollabError,
        CollabIdempotencyDecision, CollabLatencySamples, CollabLoadHarnessScenario,
        CollabLoadMetric, CollabOperation, CollabOperationCreate, CollabOperationReceipt,
        CollabPercentileBudget, CollabReplayPlan, CollabRuntimeCreate, CollabRuntimeHead,
        CollabSloDecision, CollabSnapshotRef, CollabStateVectorRef, CollabSurface, VERTICAL_SLICE,
        collab_dependency_review, collab_load_harness_plan,
    };
    use oya_office_kernel::{CellId, ObjectId, PrincipalId, TenantId};

    #[test]
    fn scaffold_identity_is_declared() {
        assert!(!ARCHITECTURE_LAYER.is_empty());
        assert!(!CRATE_NAME.is_empty());
        assert!(!VERTICAL_SLICE.is_empty());
    }

    #[test]
    fn dependency_review_adopts_only_port_seam_now() {
        let review = collab_dependency_review();

        assert_eq!(
            review
                .iter()
                .filter(|entry| entry.disposition() == CollabDependencyDisposition::AdoptNow)
                .count(),
            1
        );
        assert_eq!(
            review
                .iter()
                .find(|entry| entry.disposition() == CollabDependencyDisposition::AdoptNow)
                .map(|entry| entry.candidate()),
            Some(CollabDependencyCandidate::OfficePortSeam)
        );
        assert!(
            review
                .iter()
                .filter(|entry| entry.candidate() != CollabDependencyCandidate::OfficePortSeam)
                .all(|entry| entry.disposition()
                    == CollabDependencyDisposition::EvaluateBehindPort)
        );
    }

    #[test]
    fn dependency_review_requires_hyperscaler_safety_gates() {
        for entry in collab_dependency_review() {
            assert!(entry.requires_tenant_scope());
            assert!(entry.requires_buck2_supply_chain_gate());
            assert!(entry.requires_deterministic_replay_gate());
            assert!(entry.requires_memory_bound_gate());
        }
    }

    fn tenant() -> TenantId {
        TenantId::new("tenant-1").expect("valid tenant")
    }

    fn object() -> ObjectId {
        ObjectId::new("object-1").expect("valid object")
    }

    fn actor() -> PrincipalId {
        PrincipalId::new("principal-1").expect("valid principal")
    }

    fn cell() -> CellId {
        CellId::new("cell-a").expect("valid cell")
    }

    fn snapshot() -> CollabSnapshotRef {
        CollabSnapshotRef::new(
            tenant(),
            object(),
            CollabSurface::Docs,
            "snapshot-1".to_owned(),
            "tenant-1/object-1/snapshot-1".to_owned(),
            "state-snapshot".to_owned(),
            "sv-7".to_owned(),
            1,
            7,
            128,
        )
        .expect("valid snapshot")
    }

    fn state_vector(sequence: u64, hash: &str) -> CollabStateVectorRef {
        CollabStateVectorRef::new(
            tenant(),
            object(),
            CollabSurface::Docs,
            hash.to_owned(),
            1,
            sequence,
            32,
        )
        .expect("valid state vector")
    }

    fn operation(
        sequence: u64,
        operation_id: &str,
        base_hash: &str,
        result_hash: &str,
    ) -> CollabOperation {
        CollabOperation::new(CollabOperationCreate {
            operation_id: operation_id.to_owned(),
            tenant_id: tenant(),
            object_id: object(),
            surface: CollabSurface::Docs,
            actor_id: actor(),
            replica_id: "replica-1".to_owned(),
            base_state_vector_hash: base_hash.to_owned(),
            result_state_vector_hash: result_hash.to_owned(),
            operation_hash: format!("operation-hash-{operation_id}-{sequence}"),
            format_version: 1,
            sequence,
            byte_len: 64,
            observed_at_epoch_millis: 1_700_000_000_000 + sequence,
        })
        .expect("valid operation")
    }

    fn awareness() -> AwarenessState {
        AwarenessState::new(AwarenessStateCreate {
            tenant_id: tenant(),
            object_id: object(),
            surface: CollabSurface::Docs,
            actor_id: actor(),
            replica_id: "replica-1".to_owned(),
            session_id: "session-1".to_owned(),
            status: AwarenessStatus::Editing,
            cursor_anchor: Some("paragraph-3".to_owned()),
            observed_at_epoch_millis: 1_700_000_000_000,
            expires_at_epoch_millis: 1_700_000_030_000,
        })
        .expect("valid awareness")
    }

    #[test]
    fn runtime_head_validates_next_operation_sequence_and_binding() {
        let runtime = CollabRuntimeHead::new(CollabRuntimeCreate {
            tenant_id: tenant(),
            object_id: object(),
            cell_id: cell(),
            surface: CollabSurface::Docs,
            snapshot: snapshot(),
            state_vector: state_vector(7, "sv-7"),
            active_awareness: vec![awareness()],
            created_at_epoch_millis: 1_700_000_000_000,
            updated_at_epoch_millis: 1_700_000_010_000,
        })
        .expect("valid runtime");

        assert_eq!(runtime.schema_version(), 1);
        assert!(
            runtime
                .validate_next_operation(&operation(8, "op-8", "sv-7", "sv-8"))
                .is_ok()
        );

        let wrong_sequence = operation(9, "op-9", "sv-7", "sv-9");
        assert_eq!(
            runtime.validate_next_operation(&wrong_sequence),
            Err(CollabError::InvalidOperationSequence)
        );
    }

    #[test]
    fn operation_receipt_classifies_duplicate_and_conflicting_delivery() {
        let accepted = operation(8, "op-8", "sv-7", "sv-8");
        let receipt = CollabOperationReceipt::from_operation(&accepted);

        assert_eq!(
            receipt.classify_delivery(&accepted),
            CollabIdempotencyDecision::DuplicateNoop
        );

        let conflicting = CollabOperation::new(CollabOperationCreate {
            operation_id: "op-8".to_owned(),
            tenant_id: tenant(),
            object_id: object(),
            surface: CollabSurface::Docs,
            actor_id: actor(),
            replica_id: "replica-1".to_owned(),
            base_state_vector_hash: "sv-7".to_owned(),
            result_state_vector_hash: "sv-conflict".to_owned(),
            operation_hash: "operation-hash-conflict".to_owned(),
            format_version: 1,
            sequence: 8,
            byte_len: 64,
            observed_at_epoch_millis: 1_700_000_000_008,
        })
        .expect("valid conflicting operation");

        assert_eq!(
            receipt.classify_delivery(&conflicting),
            CollabIdempotencyDecision::RejectConflict
        );
    }

    #[test]
    fn replay_plan_requires_ordered_hash_chain_to_final_state_vector() {
        let op8 = operation(8, "op-8", "sv-7", "sv-8");
        let op9 = operation(9, "op-9", "sv-8", "sv-9");

        let plan = CollabReplayPlan::new(snapshot(), vec![op8, op9], state_vector(9, "sv-9"))
            .expect("valid replay");

        assert_eq!(plan.operation_count(), 2);
        assert_eq!(plan.last_operation_sequence(), 9);

        let invalid = CollabReplayPlan::new(
            snapshot(),
            vec![
                operation(8, "op-8", "sv-7", "sv-8"),
                operation(9, "op-9", "unexpected-base", "sv-9"),
            ],
            state_vector(9, "sv-9"),
        );
        assert_eq!(invalid, Err(CollabError::StateVectorMismatch));
    }

    #[test]
    fn awareness_presence_is_tenant_bound_and_expires() {
        let state = awareness();

        assert!(
            state
                .validate_for_object(&tenant(), &object(), CollabSurface::Docs)
                .is_ok()
        );
        assert!(!state.is_expired_at(1_700_000_010_000));
        assert!(state.is_expired_at(1_700_000_030_000));

        let wrong_tenant = TenantId::new("tenant-2").expect("valid tenant");
        assert_eq!(
            state.validate_for_object(&wrong_tenant, &object(), CollabSurface::Docs),
            Err(CollabError::AwarenessTenantMismatch)
        );

        let expired = AwarenessState::new(AwarenessStateCreate {
            tenant_id: tenant(),
            object_id: object(),
            surface: CollabSurface::Docs,
            actor_id: actor(),
            replica_id: "replica-1".to_owned(),
            session_id: "session-1".to_owned(),
            status: AwarenessStatus::Viewing,
            cursor_anchor: None,
            observed_at_epoch_millis: 10,
            expires_at_epoch_millis: 10,
        });
        assert_eq!(expired, Err(CollabError::InvalidAwarenessExpiry));
    }

    #[test]
    fn load_harness_plan_is_drive_bound_and_tracks_required_percentiles() {
        let plan = collab_load_harness_plan();

        assert!(plan.scenarios().len() >= 3);
        assert!(
            plan.scenarios()
                .iter()
                .all(CollabLoadHarnessScenario::is_drive_bound)
        );
        assert!(plan.includes_metric(CollabLoadMetric::Propagation));
        assert!(plan.includes_metric(CollabLoadMetric::Save));
        assert!(plan.includes_metric(CollabLoadMetric::Reconnect));

        for budget in plan.budgets() {
            assert!(budget.p50_millis() <= budget.p95_millis());
            assert!(budget.p95_millis() <= budget.p99_millis());
        }
    }

    #[test]
    fn latency_samples_compute_nearest_rank_percentiles() {
        let summary = CollabLatencySamples::new(
            CollabLoadMetric::Propagation,
            vec![100, 10, 20, 30, 40, 50, 60, 70, 80, 90],
        )
        .expect("valid samples")
        .summary();

        assert_eq!(summary.sample_count(), 10);
        assert_eq!(summary.p50_millis(), 50);
        assert_eq!(summary.p95_millis(), 100);
        assert_eq!(summary.p99_millis(), 100);
    }

    #[test]
    fn slo_budget_marks_p99_regression_as_failure() {
        let budget = CollabPercentileBudget::new(CollabLoadMetric::Save, 100, 250, 500)
            .expect("valid budget");
        let passing = CollabLatencySamples::new(CollabLoadMetric::Save, vec![40, 80, 120, 250])
            .expect("valid samples")
            .summary();
        let failing = CollabLatencySamples::new(CollabLoadMetric::Save, vec![40, 80, 120, 800])
            .expect("valid samples")
            .summary();

        assert_eq!(budget.evaluate(&passing), CollabSloDecision::Pass);
        assert_eq!(budget.evaluate(&failing), CollabSloDecision::Fail);
    }

    #[test]
    fn load_scenario_rejects_zero_active_sessions() {
        let scenario = CollabLoadHarnessScenario::new(
            "zero-session".to_owned(),
            tenant(),
            object(),
            cell(),
            CollabSurface::Docs,
            0,
            10,
            true,
        );

        assert_eq!(scenario, Err(CollabError::InvalidLoadHarnessScenario));
    }
}
