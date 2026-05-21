//! Workspace collab-runtime kernel.
//!
//! Workspace-internal typed kernel records for the shared real-time
//! collaborative runtime named by `docs/products/workspace/PRD.md` and
//! ADR-0029. This crate owns CRDT snapshot references, state-vector metadata,
//! operation-order checks, awareness presence metadata, and the persistence /
//! access-control seams used by Docs, Sheets, Slides, Sites, and Notes without
//! owning WebSocket, Redis, object-storage, or Yrs adapter code.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const COLLAB_RUNTIME_SCHEMA_VERSION: u32 = 1;
const SUPPORTED_CRDT_FORMAT_VERSION: u32 = 1;
const MIN_CRDT_BYTES: u64 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CollabError {
    InvalidDocumentId,
    InvalidTenantId,
    InvalidRegion,
    InvalidCellId,
    InvalidSnapshotId,
    InvalidSnapshotStorageKey,
    InvalidStateHash,
    InvalidStateVectorHash,
    InvalidFormatVersion,
    EmptySnapshot,
    EmptyStateVector,
    InvalidOperationId,
    InvalidActorRef,
    InvalidReplicaId,
    InvalidOperationHash,
    EmptyOperation,
    InvalidOperationSequence,
    StaleStateVector,
    StateVectorMismatch,
    OperationDocumentMismatch,
    OperationTenantMismatch,
    AwarenessDocumentMismatch,
    AwarenessTenantMismatch,
    InvalidSessionId,
    InvalidAwarenessExpiry,
    InvalidTimeOrder,
    InvalidDataClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CollabSurface {
    Docs,
    Sheets,
    Slides,
    Sites,
    Notes,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AwarenessStatus {
    Viewing,
    Editing,
    Idle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollabRuntimeCreate {
    pub document_id: String,                   // data_class: INTERNAL_ONLY
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub region: String,                        // data_class: INTERNAL_ONLY
    pub cell_id: String,                       // data_class: INTERNAL_ONLY
    pub surface: CollabSurface,                // data_class: INTERNAL_ONLY
    pub data_class: Option<PrivacyDataClass>,  // data_class: INTERNAL_ONLY
    pub snapshot: CollabSnapshotRef,           // data_class: PII_IDENTIFYING
    pub state_vector: CollabStateVectorRef,    // data_class: PII_IDENTIFYING
    pub active_awareness: Vec<AwarenessState>, // data_class: PII_IDENTIFYING
    pub created_at_epoch_millis: u64,          // data_class: INTERNAL_ONLY
    pub updated_at_epoch_millis: u64,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollabRuntime {
    pub document_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub region: Classified<String>,      // data_class: INTERNAL_ONLY
    pub cell_id: Classified<String>,     // data_class: INTERNAL_ONLY
    pub surface: Classified<CollabSurface>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub snapshot: Classified<CollabSnapshotRef>, // data_class: PII_IDENTIFYING
    pub state_vector: Classified<CollabStateVectorRef>, // data_class: PII_IDENTIFYING
    pub active_awareness: Classified<Vec<AwarenessState>>, // data_class: PII_IDENTIFYING
    pub created_at_epoch_millis: Classified<u64>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_millis: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollabSnapshotRef {
    pub snapshot_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub storage_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub state_hash: Classified<String>,  // data_class: INTERNAL_ONLY
    pub state_vector_hash: Classified<String>, // data_class: INTERNAL_ONLY
    pub format_version: Classified<u32>, // data_class: INTERNAL_ONLY
    pub last_operation_sequence: Classified<u64>, // data_class: INTERNAL_ONLY
    pub byte_len: Classified<u64>,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollabStateVectorRef {
    pub state_vector_hash: Classified<String>, // data_class: INTERNAL_ONLY
    pub format_version: Classified<u32>,       // data_class: INTERNAL_ONLY
    pub last_operation_sequence: Classified<u64>, // data_class: INTERNAL_ONLY
    pub byte_len: Classified<u64>,             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollabOperationCreate {
    pub operation_id: String,             // data_class: INTERNAL_ONLY
    pub document_id: String,              // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub actor_ref: String,                // data_class: PII_IDENTIFYING
    pub replica_id: String,               // data_class: INTERNAL_ONLY
    pub base_state_vector_hash: String,   // data_class: INTERNAL_ONLY
    pub result_state_vector_hash: String, // data_class: INTERNAL_ONLY
    pub operation_hash: String,           // data_class: INTERNAL_ONLY
    pub format_version: u32,              // data_class: INTERNAL_ONLY
    pub sequence: u64,                    // data_class: INTERNAL_ONLY
    pub byte_len: u64,                    // data_class: INTERNAL_ONLY
    pub observed_at_epoch_millis: u64,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollabOperation {
    pub operation_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub document_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,    // data_class: INTERNAL_ONLY
    pub actor_ref: Classified<String>,    // data_class: PII_IDENTIFYING
    pub replica_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub base_state_vector_hash: Classified<String>, // data_class: INTERNAL_ONLY
    pub result_state_vector_hash: Classified<String>, // data_class: INTERNAL_ONLY
    pub operation_hash: Classified<String>, // data_class: INTERNAL_ONLY
    pub format_version: Classified<u32>,  // data_class: INTERNAL_ONLY
    pub sequence: Classified<u64>,        // data_class: INTERNAL_ONLY
    pub byte_len: Classified<u64>,        // data_class: INTERNAL_ONLY
    pub observed_at_epoch_millis: Classified<u64>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AwarenessStateCreate {
    pub document_id: String,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub actor_ref: String,             // data_class: PII_IDENTIFYING
    pub replica_id: String,            // data_class: INTERNAL_ONLY
    pub session_id: String,            // data_class: PII_IDENTIFYING
    pub status: AwarenessStatus,       // data_class: INTERNAL_ONLY
    pub cursor_anchor: Option<String>, // data_class: PII_IDENTIFYING
    pub observed_at_epoch_millis: u64, // data_class: INTERNAL_ONLY
    pub expires_at_epoch_millis: u64,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AwarenessState {
    pub document_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub actor_ref: Classified<String>,   // data_class: PII_IDENTIFYING
    pub replica_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub session_id: Classified<String>,  // data_class: PII_IDENTIFYING
    pub status: Classified<AwarenessStatus>, // data_class: INTERNAL_ONLY
    pub cursor_anchor: Classified<Option<String>>, // data_class: PII_IDENTIFYING
    pub observed_at_epoch_millis: Classified<u64>, // data_class: INTERNAL_ONLY
    pub expires_at_epoch_millis: Classified<u64>, // data_class: INTERNAL_ONLY
}

pub trait CollabPersistenceAdapter {
    fn load_snapshot(
        &self,
        tenant_id: &str,
        document_id: &str,
    ) -> Result<Option<CollabSnapshotRef>, CollabError>;

    fn append_operation(&self, operation: &CollabOperation) -> Result<(), CollabError>;
}

pub trait CollabAccessControlAdapter {
    fn can_read(
        &self,
        tenant_id: &str,
        document_id: &str,
        actor_ref: &str,
    ) -> Result<bool, CollabError>;

    fn can_write(
        &self,
        tenant_id: &str,
        document_id: &str,
        actor_ref: &str,
    ) -> Result<bool, CollabError>;
}

impl CollabRuntime {
    pub fn new(input: CollabRuntimeCreate) -> Result<Self, CollabError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_collab_data_class());
        validate_non_empty(&input.document_id, CollabError::InvalidDocumentId)?;
        validate_non_empty(&input.tenant_id, CollabError::InvalidTenantId)?;
        validate_non_empty(&input.region, CollabError::InvalidRegion)?;
        validate_non_empty(&input.cell_id, CollabError::InvalidCellId)?;
        validate_time_order(input.created_at_epoch_millis, input.updated_at_epoch_millis)?;
        validate_runtime_head(&input.snapshot, &input.state_vector)?;
        for awareness in &input.active_awareness {
            awareness.validate_for_document(&input.tenant_id, &input.document_id)?;
        }

        Ok(Self {
            document_id: internal(input.document_id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            cell_id: internal(input.cell_id),
            surface: internal(input.surface),
            data_class: internal(data_class),
            snapshot: Classified::new(input.snapshot, collab_state_data_class()),
            state_vector: Classified::new(input.state_vector, collab_state_data_class()),
            active_awareness: Classified::new(input.active_awareness, awareness_data_class()),
            created_at_epoch_millis: internal(input.created_at_epoch_millis),
            updated_at_epoch_millis: internal(input.updated_at_epoch_millis),
            schema_version: internal(COLLAB_RUNTIME_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }

    pub fn validate_next_operation(&self, operation: &CollabOperation) -> Result<(), CollabError> {
        if operation.document_id.value != self.document_id.value {
            return Err(CollabError::OperationDocumentMismatch);
        }
        if operation.tenant_id.value != self.tenant_id.value {
            return Err(CollabError::OperationTenantMismatch);
        }
        operation.validate_next_after_state_vector(&self.state_vector.value)
    }
}

impl CollabSnapshotRef {
    pub fn new(
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
            snapshot_id: internal(snapshot_id),
            storage_key: internal(storage_key),
            state_hash: internal(state_hash),
            state_vector_hash: internal(state_vector_hash),
            format_version: internal(format_version),
            last_operation_sequence: internal(last_operation_sequence),
            byte_len: internal(byte_len),
        })
    }

    fn validate(&self) -> Result<(), CollabError> {
        validate_non_empty(&self.snapshot_id.value, CollabError::InvalidSnapshotId)?;
        validate_non_empty(
            &self.storage_key.value,
            CollabError::InvalidSnapshotStorageKey,
        )?;
        validate_non_empty(&self.state_hash.value, CollabError::InvalidStateHash)?;
        validate_non_empty(
            &self.state_vector_hash.value,
            CollabError::InvalidStateVectorHash,
        )?;
        validate_format_version(self.format_version.value)?;
        if self.byte_len.value < MIN_CRDT_BYTES {
            return Err(CollabError::EmptySnapshot);
        }
        Ok(())
    }
}

impl CollabStateVectorRef {
    pub fn new(
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
            state_vector_hash: internal(state_vector_hash),
            format_version: internal(format_version),
            last_operation_sequence: internal(last_operation_sequence),
            byte_len: internal(byte_len),
        })
    }

    fn validate(&self) -> Result<(), CollabError> {
        validate_non_empty(
            &self.state_vector_hash.value,
            CollabError::InvalidStateVectorHash,
        )?;
        validate_format_version(self.format_version.value)?;
        if self.byte_len.value < MIN_CRDT_BYTES {
            return Err(CollabError::EmptyStateVector);
        }
        Ok(())
    }
}

impl CollabOperation {
    pub fn new(input: CollabOperationCreate) -> Result<Self, CollabError> {
        validate_non_empty(&input.operation_id, CollabError::InvalidOperationId)?;
        validate_non_empty(&input.document_id, CollabError::InvalidDocumentId)?;
        validate_non_empty(&input.tenant_id, CollabError::InvalidTenantId)?;
        validate_non_empty(&input.actor_ref, CollabError::InvalidActorRef)?;
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
            operation_id: internal(input.operation_id),
            document_id: internal(input.document_id),
            tenant_id: internal(input.tenant_id),
            actor_ref: Classified::new(input.actor_ref, actor_data_class()),
            replica_id: internal(input.replica_id),
            base_state_vector_hash: internal(input.base_state_vector_hash),
            result_state_vector_hash: internal(input.result_state_vector_hash),
            operation_hash: internal(input.operation_hash),
            format_version: internal(input.format_version),
            sequence: internal(input.sequence),
            byte_len: internal(input.byte_len),
            observed_at_epoch_millis: internal(input.observed_at_epoch_millis),
        })
    }

    pub fn validate_next_after_snapshot(
        &self,
        snapshot: &CollabSnapshotRef,
    ) -> Result<(), CollabError> {
        snapshot.validate()?;
        validate_next_vector_step(
            self.format_version.value,
            self.sequence.value,
            &self.base_state_vector_hash.value,
            snapshot.format_version.value,
            snapshot.last_operation_sequence.value,
            &snapshot.state_vector_hash.value,
        )
    }

    pub fn validate_next_after_state_vector(
        &self,
        state_vector: &CollabStateVectorRef,
    ) -> Result<(), CollabError> {
        state_vector.validate()?;
        validate_next_vector_step(
            self.format_version.value,
            self.sequence.value,
            &self.base_state_vector_hash.value,
            state_vector.format_version.value,
            state_vector.last_operation_sequence.value,
            &state_vector.state_vector_hash.value,
        )
    }
}

impl AwarenessState {
    pub fn new(input: AwarenessStateCreate) -> Result<Self, CollabError> {
        validate_non_empty(&input.document_id, CollabError::InvalidDocumentId)?;
        validate_non_empty(&input.tenant_id, CollabError::InvalidTenantId)?;
        validate_non_empty(&input.actor_ref, CollabError::InvalidActorRef)?;
        validate_non_empty(&input.replica_id, CollabError::InvalidReplicaId)?;
        validate_non_empty(&input.session_id, CollabError::InvalidSessionId)?;
        if input.expires_at_epoch_millis <= input.observed_at_epoch_millis {
            return Err(CollabError::InvalidAwarenessExpiry);
        }

        Ok(Self {
            document_id: internal(input.document_id),
            tenant_id: internal(input.tenant_id),
            actor_ref: Classified::new(input.actor_ref, actor_data_class()),
            replica_id: internal(input.replica_id),
            session_id: Classified::new(input.session_id, actor_data_class()),
            status: internal(input.status),
            cursor_anchor: Classified::new(input.cursor_anchor, awareness_data_class()),
            observed_at_epoch_millis: internal(input.observed_at_epoch_millis),
            expires_at_epoch_millis: internal(input.expires_at_epoch_millis),
        })
    }

    fn validate_for_document(&self, tenant_id: &str, document_id: &str) -> Result<(), CollabError> {
        if self.document_id.value != document_id {
            return Err(CollabError::AwarenessDocumentMismatch);
        }
        if self.tenant_id.value != tenant_id {
            return Err(CollabError::AwarenessTenantMismatch);
        }
        validate_non_empty(&self.actor_ref.value, CollabError::InvalidActorRef)?;
        validate_non_empty(&self.replica_id.value, CollabError::InvalidReplicaId)?;
        validate_non_empty(&self.session_id.value, CollabError::InvalidSessionId)?;
        if self.expires_at_epoch_millis.value <= self.observed_at_epoch_millis.value {
            return Err(CollabError::InvalidAwarenessExpiry);
        }
        Ok(())
    }
}

pub fn default_workspace_collab_data_class() -> PrivacyDataClass {
    // ADR-0083 Tier 1: use kernel's infallible `pii_identifying()` constructor
    // (sibling of `internal_only`) instead of `PrivacyDataClass::new(...)
    // .expect(...)`. `PII_IDENTIFYING` is statically guaranteed to be a
    // privacy-program member; the kernel constructor encodes that invariant.
    PrivacyDataClass::pii_identifying()
}

pub fn collab_state_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn actor_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn awareness_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn workspace_collab_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, CollabError> {
    PrivacyDataClass::new(data_class).map_err(|_| CollabError::InvalidDataClass)
}

fn validate_runtime_head(
    snapshot: &CollabSnapshotRef,
    state_vector: &CollabStateVectorRef,
) -> Result<(), CollabError> {
    snapshot.validate()?;
    state_vector.validate()?;
    if snapshot.format_version.value != state_vector.format_version.value {
        return Err(CollabError::StateVectorMismatch);
    }
    if state_vector.last_operation_sequence.value < snapshot.last_operation_sequence.value {
        return Err(CollabError::StaleStateVector);
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

fn validate_non_empty(value: &str, error: CollabError) -> Result<(), CollabError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_data_boundary_kernel::{DataClassification, OperationalDataClass};

    fn snapshot() -> CollabSnapshotRef {
        CollabSnapshotRef::new(
            "snap-1".into(),
            "tenant-1/docs/doc-1/snap-1".into(),
            "sha256:snapshot".into(),
            "sv:1".into(),
            1,
            7,
            128,
        )
        .unwrap()
    }

    fn state_vector() -> CollabStateVectorRef {
        CollabStateVectorRef::new("sv:1".into(), 1, 7, 32).unwrap()
    }

    fn awareness() -> AwarenessState {
        AwarenessState::new(AwarenessStateCreate {
            document_id: "doc-1".into(),
            tenant_id: "tenant-1".into(),
            actor_ref: "user:writer@example.com".into(),
            replica_id: "replica-1".into(),
            session_id: "session-1".into(),
            status: AwarenessStatus::Editing,
            cursor_anchor: Some("paragraph:3".into()),
            observed_at_epoch_millis: 1_700_000_000_000,
            expires_at_epoch_millis: 1_700_000_030_000,
        })
        .unwrap()
    }

    fn runtime_input() -> CollabRuntimeCreate {
        CollabRuntimeCreate {
            document_id: "doc-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            surface: CollabSurface::Docs,
            data_class: None,
            snapshot: snapshot(),
            state_vector: state_vector(),
            active_awareness: vec![awareness()],
            created_at_epoch_millis: 1_700_000_000_000,
            updated_at_epoch_millis: 1_700_000_010_000,
        }
    }

    fn operation_input() -> CollabOperationCreate {
        CollabOperationCreate {
            operation_id: "op-8".into(),
            document_id: "doc-1".into(),
            tenant_id: "tenant-1".into(),
            actor_ref: "user:writer@example.com".into(),
            replica_id: "replica-1".into(),
            base_state_vector_hash: "sv:1".into(),
            result_state_vector_hash: "sv:2".into(),
            operation_hash: "sha256:op".into(),
            format_version: 1,
            sequence: 8,
            byte_len: 12,
            observed_at_epoch_millis: 1_700_000_011_000,
        }
    }

    #[test]
    fn runtime_defaults_to_identifying_class_and_classifies_awareness() {
        let runtime = CollabRuntime::new(runtime_input()).unwrap();

        assert_eq!(
            runtime.privacy_data_class().data_class(),
            DataClass::PiiIdentifying
        );
        assert_eq!(
            runtime.active_awareness.data_class,
            DataClassification::Privacy(awareness_data_class())
        );
        assert_eq!(
            runtime.active_awareness.value[0].actor_ref.data_class,
            DataClassification::Privacy(actor_data_class())
        );
        assert_eq!(runtime.schema_version.value, 1);
    }

    #[test]
    fn snapshot_and_operation_reject_empty_crdt_payload_refs() {
        let empty_snapshot = CollabSnapshotRef::new(
            "snap-1".into(),
            "tenant-1/docs/doc-1/snap-1".into(),
            "sha256:snapshot".into(),
            "sv:1".into(),
            1,
            7,
            0,
        );
        assert_eq!(empty_snapshot, Err(CollabError::EmptySnapshot));

        let mut invalid_operation = operation_input();
        invalid_operation.byte_len = 0;
        assert_eq!(
            CollabOperation::new(invalid_operation),
            Err(CollabError::EmptyOperation)
        );
    }

    #[test]
    fn operation_requires_next_version_vector_step() {
        let operation = CollabOperation::new(operation_input()).unwrap();
        let runtime = CollabRuntime::new(runtime_input()).unwrap();

        assert_eq!(runtime.validate_next_operation(&operation), Ok(()));
        assert_eq!(operation.validate_next_after_snapshot(&snapshot()), Ok(()));

        let mut stale = operation_input();
        stale.sequence = 7;
        let stale = CollabOperation::new(stale).unwrap();
        assert_eq!(
            runtime.validate_next_operation(&stale),
            Err(CollabError::InvalidOperationSequence)
        );

        let mut mismatched = operation_input();
        mismatched.base_state_vector_hash = "sv:other".into();
        let mismatched = CollabOperation::new(mismatched).unwrap();
        assert_eq!(
            runtime.validate_next_operation(&mismatched),
            Err(CollabError::StateVectorMismatch)
        );
    }

    #[test]
    fn awareness_expiry_must_follow_observed_time_and_match_runtime_document() {
        let invalid_awareness = AwarenessState::new(AwarenessStateCreate {
            expires_at_epoch_millis: 1_700_000_000_000,
            ..AwarenessStateCreate {
                document_id: "doc-1".into(),
                tenant_id: "tenant-1".into(),
                actor_ref: "user:writer@example.com".into(),
                replica_id: "replica-1".into(),
                session_id: "session-1".into(),
                status: AwarenessStatus::Editing,
                cursor_anchor: None,
                observed_at_epoch_millis: 1_700_000_000_000,
                expires_at_epoch_millis: 1_700_000_030_000,
            }
        });
        assert_eq!(invalid_awareness, Err(CollabError::InvalidAwarenessExpiry));

        let mut runtime = runtime_input();
        runtime.active_awareness = vec![
            AwarenessState::new(AwarenessStateCreate {
                document_id: "other-doc".into(),
                tenant_id: "tenant-1".into(),
                actor_ref: "user:writer@example.com".into(),
                replica_id: "replica-1".into(),
                session_id: "session-1".into(),
                status: AwarenessStatus::Viewing,
                cursor_anchor: None,
                observed_at_epoch_millis: 1_700_000_000_000,
                expires_at_epoch_millis: 1_700_000_030_000,
            })
            .unwrap(),
        ];
        assert_eq!(
            CollabRuntime::new(runtime),
            Err(CollabError::AwarenessDocumentMismatch)
        );
    }

    #[test]
    fn legacy_data_class_conversion_rejects_operational_markers() {
        assert_eq!(
            workspace_collab_data_class_from_legacy(DataClass::Audit),
            Err(CollabError::InvalidDataClass)
        );
        assert_eq!(
            DataClassification::from(OperationalDataClass::Audit).privacy_data_class(),
            None
        );
    }
}
