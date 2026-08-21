//! Platform DSR cascade kernel.
//!
//! Cross-axis DSR request, dispatch, acknowledgement, proof, and completion
//! records for the `DSR_CONSENT_WITHDRAWAL` contract named by `docs/DESIGN.md`,
//! `docs/SPEC.md`, and `docs/machine-readable/contracts.json`. This kernel owns
//! typed invariants only; platform apps own orchestration, audit-chain append,
//! queueing, and trust portal publication.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const DSR_REQUEST_SCHEMA_VERSION: u32 = 1;
const DSR_STORE_REF_SCHEMA_VERSION: u32 = 1;
const DSR_DISPATCH_SCHEMA_VERSION: u32 = 1;
const DSR_PROOF_SCHEMA_VERSION: u32 = 1;
const DSR_ACK_SCHEMA_VERSION: u32 = 1;
const DSR_COMPLETION_SCHEMA_VERSION: u32 = 1;
const DAY_SECONDS: u64 = 86_400;
const SHA256_PREFIX: &str = "sha256:";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlatformDsrError {
    InvalidDsrId,
    InvalidTenantId,
    InvalidRegion,
    InvalidSubjectRef,
    InvalidStoreId,
    InvalidCellId,
    InvalidRecordRef,
    InvalidDispatchId,
    InvalidIdempotencyKey,
    InvalidAckId,
    InvalidProofId,
    InvalidCompletionId,
    InvalidWitnessRef,
    InvalidSignerRef,
    InvalidSignatureRef,
    InvalidEvidenceHash,
    InvalidAggregateProofHash,
    EmptyDataClassSet,
    DuplicateDataClass,
    DeadlineExceedsSla,
    InvalidTimeOrder,
    StoreOutOfScope,
    DataClassOutOfScope,
    ProofDispatchMismatch,
    ProofMethodMismatch,
    AckDispatchMismatch,
    AckStatusInvalid,
    AckProofMismatch,
    EmptyDispatchSet,
    DuplicateDispatchId,
    DuplicateStoreRef,
    EmptyAckSet,
    MissingDispatchAck,
    DuplicateAckDispatchId,
    NonTerminalAck,
    MissingCompletedProof,
    DuplicateProofId,
    ProofCoverageMismatch,
    InvalidDataClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DsrAction {
    Erase,
    Correct,
    Export,
    Restrict,
    ObjectToProcessing,
    AutomatedDecisionOptOut,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DsrSlaTier {
    Preview,
    Stable,
    Ga,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DsrSlaStatus {
    WithinSla,
    Breached,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DsrAxis {
    Saas,
    Workspace,
    Vertical,
    Foundry,
    Cloud,
    Search,
    Ads,
    Analytics,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DsrStoreKind {
    TenantTable,
    WorkspaceObject,
    VerticalRecord,
    FoundryMemory,
    CloudResource,
    SearchIndex,
    AdsAttribution,
    AnalyticsWarehouse,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DsrProofMethod {
    KmsShred,
    RecordDelete,
    IndexRebuild,
    ColdStoragePurge,
    CorrectionApplied,
    ExportProduced,
    RestrictApplied,
    ObjectionApplied,
    AutomatedDecisionOptOutApplied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DsrAckStatus {
    Accepted,
    Completed,
    RetryableFailure,
    PermanentBlock,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DsrAckReason {
    LawfulRetention,
    SubjectIdentityUnverified,
    StoreUnavailable,
    UnsupportedAction,
    ResidencyConflict,
    IntegrityCheckFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DsrCompletionStatus {
    Completed,
    CompletedWithBlocks,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DsrRequestCreate {
    pub dsr_id: String,                      // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub region: String,                      // data_class: INTERNAL_ONLY
    pub subject_ref: String,                 // data_class: PII_IDENTIFYING
    pub action: DsrAction,                   // data_class: INTERNAL_ONLY
    pub sla_tier: DsrSlaTier,                // data_class: INTERNAL_ONLY
    pub data_classes: Vec<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub received_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
    pub deadline_epoch_seconds: u64,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DsrRequest {
    pub dsr_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,    // data_class: INTERNAL_ONLY
    pub region: Classified<String>,       // data_class: INTERNAL_ONLY
    pub subject_ref: Classified<String>,  // data_class: PII_IDENTIFYING
    pub action: Classified<DsrAction>,    // data_class: INTERNAL_ONLY
    pub sla_tier: Classified<DsrSlaTier>, // data_class: INTERNAL_ONLY
    pub data_classes: Classified<Vec<PrivacyDataClass>>, // data_class: INTERNAL_ONLY
    pub received_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub deadline_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DsrStoreRefCreate {
    pub axis: DsrAxis,                // data_class: INTERNAL_ONLY
    pub kind: DsrStoreKind,           // data_class: INTERNAL_ONLY
    pub store_id: String,             // data_class: INTERNAL_ONLY
    pub tenant_id: String,            // data_class: INTERNAL_ONLY
    pub region: String,               // data_class: INTERNAL_ONLY
    pub cell_id: String,              // data_class: INTERNAL_ONLY
    pub record_ref: String,           // data_class: INTERNAL_ONLY
    pub data_class: PrivacyDataClass, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DsrStoreRef {
    pub axis: Classified<DsrAxis>,      // data_class: INTERNAL_ONLY
    pub kind: Classified<DsrStoreKind>, // data_class: INTERNAL_ONLY
    pub store_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub region: Classified<String>,     // data_class: INTERNAL_ONLY
    pub cell_id: Classified<String>,    // data_class: INTERNAL_ONLY
    pub record_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DsrDispatchCreate {
    pub dispatch_id: String,              // data_class: INTERNAL_ONLY
    pub idempotency_key: String,          // data_class: INTERNAL_ONLY
    pub store: DsrStoreRef,               // data_class: INTERNAL_ONLY
    pub dispatched_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DsrDispatch {
    pub dispatch_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub dsr_id: Classified<String>,      // data_class: INTERNAL_ONLY
    pub action: Classified<DsrAction>,   // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub store: Classified<DsrStoreRef>,  // data_class: INTERNAL_ONLY
    pub dispatched_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErasureProofCreate {
    pub proof_id: String,             // data_class: INTERNAL_ONLY
    pub dispatch_id: String,          // data_class: INTERNAL_ONLY
    pub dsr_id: String,               // data_class: INTERNAL_ONLY
    pub action: DsrAction,            // data_class: INTERNAL_ONLY
    pub store: DsrStoreRef,           // data_class: INTERNAL_ONLY
    pub method: DsrProofMethod,       // data_class: INTERNAL_ONLY
    pub evidence_hash: String,        // data_class: INTERNAL_ONLY
    pub witness_ref: String,          // data_class: INTERNAL_ONLY
    pub signer_ref: String,           // data_class: INTERNAL_ONLY
    pub signature_ref: String,        // data_class: INTERNAL_ONLY
    pub rekor_log_index: u64,         // data_class: INTERNAL_ONLY
    pub proved_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErasureProof {
    pub proof_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub dispatch_id: Classified<String>,    // data_class: INTERNAL_ONLY
    pub dsr_id: Classified<String>,         // data_class: INTERNAL_ONLY
    pub action: Classified<DsrAction>,      // data_class: INTERNAL_ONLY
    pub store: Classified<DsrStoreRef>,     // data_class: INTERNAL_ONLY
    pub method: Classified<DsrProofMethod>, // data_class: INTERNAL_ONLY
    pub evidence_hash: Classified<String>,  // data_class: INTERNAL_ONLY
    pub witness_ref: Classified<String>,    // data_class: INTERNAL_ONLY
    pub signer_ref: Classified<String>,     // data_class: INTERNAL_ONLY
    pub signature_ref: Classified<String>,  // data_class: INTERNAL_ONLY
    pub rekor_log_index: Classified<u64>,   // data_class: INTERNAL_ONLY
    pub proved_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DsrCascadeAckCreate {
    pub ack_id: String,                     // data_class: INTERNAL_ONLY
    pub dispatch_id: String,                // data_class: INTERNAL_ONLY
    pub dsr_id: String,                     // data_class: INTERNAL_ONLY
    pub status: DsrAckStatus,               // data_class: INTERNAL_ONLY
    pub reason: Option<DsrAckReason>,       // data_class: INTERNAL_ONLY
    pub proof_id: Option<String>,           // data_class: INTERNAL_ONLY
    pub evidence_hash: Option<String>,      // data_class: INTERNAL_ONLY
    pub acknowledged_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DsrCascadeAck {
    pub ack_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub dispatch_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub dsr_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub status: Classified<DsrAckStatus>, // data_class: INTERNAL_ONLY
    pub reason: Classified<Option<DsrAckReason>>, // data_class: INTERNAL_ONLY
    pub proof_id: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub evidence_hash: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub acknowledged_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DsrCompletionRecordCreate {
    pub completion_id: String,           // data_class: INTERNAL_ONLY
    pub dsr_id: String,                  // data_class: INTERNAL_ONLY
    pub dispatches: Vec<DsrDispatch>,    // data_class: INTERNAL_ONLY
    pub acks: Vec<DsrCascadeAck>,        // data_class: INTERNAL_ONLY
    pub proofs: Vec<ErasureProof>,       // data_class: INTERNAL_ONLY
    pub aggregate_proof_hash: String,    // data_class: INTERNAL_ONLY
    pub signer_ref: String,              // data_class: INTERNAL_ONLY
    pub signature_ref: String,           // data_class: INTERNAL_ONLY
    pub rekor_log_index: u64,            // data_class: INTERNAL_ONLY
    pub completed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DsrCompletionRecord {
    pub completion_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub dsr_id: Classified<String>,        // data_class: INTERNAL_ONLY
    pub completion_status: Classified<DsrCompletionStatus>, // data_class: INTERNAL_ONLY
    pub sla_status: Classified<DsrSlaStatus>, // data_class: INTERNAL_ONLY
    pub dispatch_ids: Classified<Vec<String>>, // data_class: INTERNAL_ONLY
    pub ack_ids: Classified<Vec<String>>,  // data_class: INTERNAL_ONLY
    pub proof_ids: Classified<Vec<String>>, // data_class: INTERNAL_ONLY
    pub aggregate_proof_hash: Classified<String>, // data_class: INTERNAL_ONLY
    pub signer_ref: Classified<String>,    // data_class: INTERNAL_ONLY
    pub signature_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub rekor_log_index: Classified<u64>,  // data_class: INTERNAL_ONLY
    pub completed_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,   // data_class: INTERNAL_ONLY
}

impl DsrSlaTier {
    pub const fn max_seconds(self) -> u64 {
        match self {
            Self::Preview => 30 * DAY_SECONDS,
            Self::Stable => 14 * DAY_SECONDS,
            Self::Ga => 7 * DAY_SECONDS,
        }
    }
}

impl DsrRequest {
    pub fn new(input: DsrRequestCreate) -> Result<Self, PlatformDsrError> {
        validate_non_empty(&input.dsr_id, PlatformDsrError::InvalidDsrId)?;
        validate_non_empty(&input.tenant_id, PlatformDsrError::InvalidTenantId)?;
        validate_non_empty(&input.region, PlatformDsrError::InvalidRegion)?;
        validate_non_empty(&input.subject_ref, PlatformDsrError::InvalidSubjectRef)?;
        validate_data_classes(&input.data_classes)?;
        validate_time_order(
            input.received_at_epoch_seconds,
            input.deadline_epoch_seconds,
        )?;
        let max_deadline = input
            .received_at_epoch_seconds
            .checked_add(input.sla_tier.max_seconds())
            .ok_or(PlatformDsrError::DeadlineExceedsSla)?;
        if input.deadline_epoch_seconds > max_deadline {
            return Err(PlatformDsrError::DeadlineExceedsSla);
        }
        Ok(Self {
            dsr_id: internal(input.dsr_id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            subject_ref: Classified::new(input.subject_ref, subject_data_class()),
            action: internal(input.action),
            sla_tier: internal(input.sla_tier),
            data_classes: internal(input.data_classes),
            received_at_epoch_seconds: internal(input.received_at_epoch_seconds),
            deadline_epoch_seconds: internal(input.deadline_epoch_seconds),
            schema_version: internal(DSR_REQUEST_SCHEMA_VERSION),
        })
    }
}

impl DsrStoreRef {
    pub fn new(input: DsrStoreRefCreate) -> Result<Self, PlatformDsrError> {
        validate_non_empty(&input.store_id, PlatformDsrError::InvalidStoreId)?;
        validate_non_empty(&input.tenant_id, PlatformDsrError::InvalidTenantId)?;
        validate_non_empty(&input.region, PlatformDsrError::InvalidRegion)?;
        validate_non_empty(&input.cell_id, PlatformDsrError::InvalidCellId)?;
        validate_non_empty(&input.record_ref, PlatformDsrError::InvalidRecordRef)?;
        Ok(Self {
            axis: internal(input.axis),
            kind: internal(input.kind),
            store_id: internal(input.store_id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            cell_id: internal(input.cell_id),
            record_ref: internal(input.record_ref),
            data_class: internal(input.data_class),
            schema_version: internal(DSR_STORE_REF_SCHEMA_VERSION),
        })
    }

    pub fn key(&self) -> (DsrAxis, DsrStoreKind, String, String) {
        (
            self.axis.value,
            self.kind.value,
            self.store_id.value.clone(),
            self.record_ref.value.clone(),
        )
    }
}

impl DsrDispatch {
    pub fn new(input: DsrDispatchCreate, request: &DsrRequest) -> Result<Self, PlatformDsrError> {
        validate_non_empty(&input.dispatch_id, PlatformDsrError::InvalidDispatchId)?;
        validate_non_empty(
            &input.idempotency_key,
            PlatformDsrError::InvalidIdempotencyKey,
        )?;
        validate_time_order(
            request.received_at_epoch_seconds.value,
            input.dispatched_at_epoch_seconds,
        )?;
        validate_store_scope(&input.store, request)?;
        Ok(Self {
            dispatch_id: internal(input.dispatch_id),
            dsr_id: internal(request.dsr_id.value.clone()),
            action: internal(request.action.value),
            idempotency_key: internal(input.idempotency_key),
            store: internal(input.store),
            dispatched_at_epoch_seconds: internal(input.dispatched_at_epoch_seconds),
            schema_version: internal(DSR_DISPATCH_SCHEMA_VERSION),
        })
    }
}

impl ErasureProof {
    pub fn new(
        input: ErasureProofCreate,
        dispatch: &DsrDispatch,
    ) -> Result<Self, PlatformDsrError> {
        validate_non_empty(&input.proof_id, PlatformDsrError::InvalidProofId)?;
        validate_non_empty(&input.dispatch_id, PlatformDsrError::InvalidDispatchId)?;
        validate_non_empty(&input.dsr_id, PlatformDsrError::InvalidDsrId)?;
        validate_hash(&input.evidence_hash, PlatformDsrError::InvalidEvidenceHash)?;
        validate_non_empty(&input.witness_ref, PlatformDsrError::InvalidWitnessRef)?;
        validate_non_empty(&input.signer_ref, PlatformDsrError::InvalidSignerRef)?;
        validate_non_empty(&input.signature_ref, PlatformDsrError::InvalidSignatureRef)?;
        validate_proof_matches_dispatch(&input, dispatch)?;
        validate_proof_method(input.action, input.method)?;
        validate_time_order(
            dispatch.dispatched_at_epoch_seconds.value,
            input.proved_at_epoch_seconds,
        )?;
        Ok(Self {
            proof_id: internal(input.proof_id),
            dispatch_id: internal(input.dispatch_id),
            dsr_id: internal(input.dsr_id),
            action: internal(input.action),
            store: internal(input.store),
            method: internal(input.method),
            evidence_hash: internal(input.evidence_hash),
            witness_ref: internal(input.witness_ref),
            signer_ref: internal(input.signer_ref),
            signature_ref: internal(input.signature_ref),
            rekor_log_index: internal(input.rekor_log_index),
            proved_at_epoch_seconds: internal(input.proved_at_epoch_seconds),
            schema_version: internal(DSR_PROOF_SCHEMA_VERSION),
        })
    }
}

impl DsrCascadeAck {
    pub fn new(
        input: DsrCascadeAckCreate,
        dispatch: &DsrDispatch,
        proof: Option<&ErasureProof>,
    ) -> Result<Self, PlatformDsrError> {
        validate_non_empty(&input.ack_id, PlatformDsrError::InvalidAckId)?;
        validate_non_empty(&input.dispatch_id, PlatformDsrError::InvalidDispatchId)?;
        validate_non_empty(&input.dsr_id, PlatformDsrError::InvalidDsrId)?;
        validate_time_order(
            dispatch.dispatched_at_epoch_seconds.value,
            input.acknowledged_at_epoch_seconds,
        )?;
        validate_ack_matches_dispatch(&input, dispatch)?;
        validate_ack_status(&input, proof)?;
        Ok(Self {
            ack_id: internal(input.ack_id),
            dispatch_id: internal(input.dispatch_id),
            dsr_id: internal(input.dsr_id),
            status: internal(input.status),
            reason: internal(input.reason),
            proof_id: internal(input.proof_id),
            evidence_hash: internal(input.evidence_hash),
            acknowledged_at_epoch_seconds: internal(input.acknowledged_at_epoch_seconds),
            schema_version: internal(DSR_ACK_SCHEMA_VERSION),
        })
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status.value,
            DsrAckStatus::Completed | DsrAckStatus::PermanentBlock
        )
    }
}

impl DsrCompletionRecord {
    pub fn new(
        input: DsrCompletionRecordCreate,
        request: &DsrRequest,
    ) -> Result<Self, PlatformDsrError> {
        validate_non_empty(&input.completion_id, PlatformDsrError::InvalidCompletionId)?;
        validate_non_empty(&input.dsr_id, PlatformDsrError::InvalidDsrId)?;
        validate_hash(
            &input.aggregate_proof_hash,
            PlatformDsrError::InvalidAggregateProofHash,
        )?;
        validate_non_empty(&input.signer_ref, PlatformDsrError::InvalidSignerRef)?;
        validate_non_empty(&input.signature_ref, PlatformDsrError::InvalidSignatureRef)?;
        validate_time_order(
            request.received_at_epoch_seconds.value,
            input.completed_at_epoch_seconds,
        )?;
        validate_completion_scope(&input, request)?;
        let dispatch_ids = dispatch_ids(&input.dispatches)?;
        let ack_ids = ack_ids(&input.acks)?;
        let proof_ids = proof_ids(&input.proofs)?;
        let completion_status = if input
            .acks
            .iter()
            .any(|ack| ack.status.value == DsrAckStatus::PermanentBlock)
        {
            DsrCompletionStatus::CompletedWithBlocks
        } else {
            DsrCompletionStatus::Completed
        };
        let sla_status = if input.completed_at_epoch_seconds <= request.deadline_epoch_seconds.value
        {
            DsrSlaStatus::WithinSla
        } else {
            DsrSlaStatus::Breached
        };
        Ok(Self {
            completion_id: internal(input.completion_id),
            dsr_id: internal(input.dsr_id),
            completion_status: internal(completion_status),
            sla_status: internal(sla_status),
            dispatch_ids: internal(dispatch_ids),
            ack_ids: internal(ack_ids),
            proof_ids: internal(proof_ids),
            aggregate_proof_hash: internal(input.aggregate_proof_hash),
            signer_ref: internal(input.signer_ref),
            signature_ref: internal(input.signature_ref),
            rekor_log_index: internal(input.rekor_log_index),
            completed_at_epoch_seconds: internal(input.completed_at_epoch_seconds),
            schema_version: internal(DSR_COMPLETION_SCHEMA_VERSION),
        })
    }
}

pub fn subject_data_class() -> PrivacyDataClass {
    // ADR-0083 Tier 1: use kernel's infallible `pii_identifying()` constructor.
    PrivacyDataClass::pii_identifying()
}

pub fn default_platform_dsr_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn platform_dsr_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, PlatformDsrError> {
    PrivacyDataClass::new(data_class).map_err(|_| PlatformDsrError::InvalidDataClass)
}

fn validate_completion_scope(
    input: &DsrCompletionRecordCreate,
    request: &DsrRequest,
) -> Result<(), PlatformDsrError> {
    if input.dsr_id != request.dsr_id.value {
        return Err(PlatformDsrError::ProofCoverageMismatch);
    }
    validate_dispatches(&input.dispatches, request)?;
    let dispatch_by_id = input
        .dispatches
        .iter()
        .map(|dispatch| (dispatch.dispatch_id.value.as_str(), dispatch))
        .collect::<BTreeMap<_, _>>();
    let proof_by_id = validate_proofs(&input.proofs, request, &dispatch_by_id)?;
    validate_acks(&input.acks, request, &dispatch_by_id, &proof_by_id)
}

fn validate_dispatches(
    dispatches: &[DsrDispatch],
    request: &DsrRequest,
) -> Result<(), PlatformDsrError> {
    if dispatches.is_empty() {
        return Err(PlatformDsrError::EmptyDispatchSet);
    }
    let mut ids = BTreeSet::new();
    let mut stores = BTreeSet::new();
    for dispatch in dispatches {
        if dispatch.dsr_id.value != request.dsr_id.value
            || dispatch.action.value != request.action.value
        {
            return Err(PlatformDsrError::AckDispatchMismatch);
        }
        if !ids.insert(dispatch.dispatch_id.value.clone()) {
            return Err(PlatformDsrError::DuplicateDispatchId);
        }
        if !stores.insert(dispatch.store.value.key()) {
            return Err(PlatformDsrError::DuplicateStoreRef);
        }
        validate_store_scope(&dispatch.store.value, request)?;
    }
    Ok(())
}

fn validate_proofs<'a>(
    proofs: &'a [ErasureProof],
    request: &DsrRequest,
    dispatch_by_id: &BTreeMap<&str, &DsrDispatch>,
) -> Result<BTreeMap<&'a str, &'a ErasureProof>, PlatformDsrError> {
    let mut by_id = BTreeMap::new();
    for proof in proofs {
        if proof.dsr_id.value != request.dsr_id.value || proof.action.value != request.action.value
        {
            return Err(PlatformDsrError::ProofCoverageMismatch);
        }
        if by_id.insert(proof.proof_id.value.as_str(), proof).is_some() {
            return Err(PlatformDsrError::DuplicateProofId);
        }
        let Some(dispatch) = dispatch_by_id.get(proof.dispatch_id.value.as_str()) else {
            return Err(PlatformDsrError::ProofDispatchMismatch);
        };
        if proof.store.value != dispatch.store.value {
            return Err(PlatformDsrError::ProofDispatchMismatch);
        }
    }
    Ok(by_id)
}

fn validate_acks(
    acks: &[DsrCascadeAck],
    request: &DsrRequest,
    dispatch_by_id: &BTreeMap<&str, &DsrDispatch>,
    proof_by_id: &BTreeMap<&str, &ErasureProof>,
) -> Result<(), PlatformDsrError> {
    if acks.is_empty() {
        return Err(PlatformDsrError::EmptyAckSet);
    }
    let mut acked_dispatches = BTreeSet::new();
    let mut completed_proof_ids = BTreeSet::new();
    for ack in acks {
        if ack.dsr_id.value != request.dsr_id.value {
            return Err(PlatformDsrError::AckDispatchMismatch);
        }
        if !ack.is_terminal() {
            return Err(PlatformDsrError::NonTerminalAck);
        }
        if !acked_dispatches.insert(ack.dispatch_id.value.clone()) {
            return Err(PlatformDsrError::DuplicateAckDispatchId);
        }
        if !dispatch_by_id.contains_key(ack.dispatch_id.value.as_str()) {
            return Err(PlatformDsrError::AckDispatchMismatch);
        }
        if ack.status.value == DsrAckStatus::Completed {
            let Some(proof_id) = ack.proof_id.value.as_deref() else {
                return Err(PlatformDsrError::MissingCompletedProof);
            };
            let Some(proof) = proof_by_id.get(proof_id) else {
                return Err(PlatformDsrError::MissingCompletedProof);
            };
            if proof.dispatch_id.value != ack.dispatch_id.value {
                return Err(PlatformDsrError::AckProofMismatch);
            }
            completed_proof_ids.insert(proof_id.to_string());
        }
    }
    let expected_dispatches = dispatch_by_id
        .keys()
        .map(|dispatch_id| (*dispatch_id).to_string())
        .collect::<BTreeSet<_>>();
    if acked_dispatches != expected_dispatches {
        return Err(PlatformDsrError::MissingDispatchAck);
    }
    let proof_ids = proof_by_id
        .keys()
        .map(|proof_id| (*proof_id).to_string())
        .collect::<BTreeSet<_>>();
    if completed_proof_ids != proof_ids {
        return Err(PlatformDsrError::ProofCoverageMismatch);
    }
    Ok(())
}

fn validate_store_scope(store: &DsrStoreRef, request: &DsrRequest) -> Result<(), PlatformDsrError> {
    if store.tenant_id.value != request.tenant_id.value
        || store.region.value != request.region.value
    {
        return Err(PlatformDsrError::StoreOutOfScope);
    }
    if !request.data_classes.value.contains(&store.data_class.value) {
        return Err(PlatformDsrError::DataClassOutOfScope);
    }
    Ok(())
}

fn validate_proof_matches_dispatch(
    input: &ErasureProofCreate,
    dispatch: &DsrDispatch,
) -> Result<(), PlatformDsrError> {
    if input.dispatch_id != dispatch.dispatch_id.value
        || input.dsr_id != dispatch.dsr_id.value
        || input.action != dispatch.action.value
        || input.store != dispatch.store.value
    {
        return Err(PlatformDsrError::ProofDispatchMismatch);
    }
    Ok(())
}

fn validate_proof_method(
    action: DsrAction,
    method: DsrProofMethod,
) -> Result<(), PlatformDsrError> {
    let valid = match action {
        DsrAction::Erase => matches!(
            method,
            DsrProofMethod::KmsShred
                | DsrProofMethod::RecordDelete
                | DsrProofMethod::IndexRebuild
                | DsrProofMethod::ColdStoragePurge
        ),
        DsrAction::Correct => method == DsrProofMethod::CorrectionApplied,
        DsrAction::Export => method == DsrProofMethod::ExportProduced,
        DsrAction::Restrict => method == DsrProofMethod::RestrictApplied,
        DsrAction::ObjectToProcessing => method == DsrProofMethod::ObjectionApplied,
        DsrAction::AutomatedDecisionOptOut => {
            method == DsrProofMethod::AutomatedDecisionOptOutApplied
        }
    };
    if valid {
        Ok(())
    } else {
        Err(PlatformDsrError::ProofMethodMismatch)
    }
}

fn validate_ack_matches_dispatch(
    input: &DsrCascadeAckCreate,
    dispatch: &DsrDispatch,
) -> Result<(), PlatformDsrError> {
    if input.dispatch_id != dispatch.dispatch_id.value || input.dsr_id != dispatch.dsr_id.value {
        return Err(PlatformDsrError::AckDispatchMismatch);
    }
    Ok(())
}

fn validate_ack_status(
    input: &DsrCascadeAckCreate,
    proof: Option<&ErasureProof>,
) -> Result<(), PlatformDsrError> {
    match input.status {
        DsrAckStatus::Accepted => {
            if input.reason.is_some() || input.proof_id.is_some() || input.evidence_hash.is_some() {
                return Err(PlatformDsrError::AckStatusInvalid);
            }
        }
        DsrAckStatus::Completed => {
            if input.reason.is_some() {
                return Err(PlatformDsrError::AckStatusInvalid);
            }
            let Some(proof) = proof else {
                return Err(PlatformDsrError::MissingCompletedProof);
            };
            if input.proof_id.as_deref() != Some(proof.proof_id.value.as_str())
                || input.evidence_hash.as_deref() != Some(proof.evidence_hash.value.as_str())
            {
                return Err(PlatformDsrError::AckProofMismatch);
            }
        }
        DsrAckStatus::RetryableFailure | DsrAckStatus::PermanentBlock => {
            if input.reason.is_none() || input.proof_id.is_some() || input.evidence_hash.is_some() {
                return Err(PlatformDsrError::AckStatusInvalid);
            }
        }
    }
    if let Some(hash) = input.evidence_hash.as_deref() {
        validate_hash(hash, PlatformDsrError::InvalidEvidenceHash)?;
    }
    Ok(())
}

fn dispatch_ids(dispatches: &[DsrDispatch]) -> Result<Vec<String>, PlatformDsrError> {
    let mut ids = BTreeSet::new();
    for dispatch in dispatches {
        if !ids.insert(dispatch.dispatch_id.value.clone()) {
            return Err(PlatformDsrError::DuplicateDispatchId);
        }
    }
    Ok(ids.into_iter().collect())
}

fn ack_ids(acks: &[DsrCascadeAck]) -> Result<Vec<String>, PlatformDsrError> {
    let mut ids = BTreeSet::new();
    for ack in acks {
        if !ids.insert(ack.ack_id.value.clone()) {
            return Err(PlatformDsrError::DuplicateAckDispatchId);
        }
    }
    Ok(ids.into_iter().collect())
}

fn proof_ids(proofs: &[ErasureProof]) -> Result<Vec<String>, PlatformDsrError> {
    let mut ids = BTreeSet::new();
    for proof in proofs {
        if !ids.insert(proof.proof_id.value.clone()) {
            return Err(PlatformDsrError::DuplicateProofId);
        }
    }
    Ok(ids.into_iter().collect())
}

fn validate_data_classes(data_classes: &[PrivacyDataClass]) -> Result<(), PlatformDsrError> {
    if data_classes.is_empty() {
        return Err(PlatformDsrError::EmptyDataClassSet);
    }
    let mut seen = BTreeSet::new();
    for data_class in data_classes {
        if !seen.insert(*data_class) {
            return Err(PlatformDsrError::DuplicateDataClass);
        }
    }
    Ok(())
}

fn validate_hash(hash: &str, error: PlatformDsrError) -> Result<(), PlatformDsrError> {
    if hash.trim() != hash
        || !hash.starts_with(SHA256_PREFIX)
        || hash.len() == SHA256_PREFIX.len()
        || hash.chars().any(char::is_control)
    {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_time_order(start: u64, end: u64) -> Result<(), PlatformDsrError> {
    if start > end {
        Err(PlatformDsrError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

fn validate_non_empty(value: &str, error: PlatformDsrError) -> Result<(), PlatformDsrError> {
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
    use data_boundary_kernel::{DataClassification, OperationalDataClass};

    fn privacy(data_class: DataClass) -> PrivacyDataClass {
        PrivacyDataClass::new(data_class).unwrap()
    }

    fn request(action: DsrAction) -> DsrRequest {
        DsrRequest::new(DsrRequestCreate {
            dsr_id: "dsr-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            subject_ref: "subject-1".into(),
            action,
            sla_tier: DsrSlaTier::Ga,
            data_classes: vec![privacy(DataClass::PiiIdentifying)],
            received_at_epoch_seconds: 1_700_000_000,
            deadline_epoch_seconds: 1_700_000_000 + DsrSlaTier::Ga.max_seconds(),
        })
        .unwrap()
    }

    fn store(axis: DsrAxis, kind: DsrStoreKind, record_ref: &str) -> DsrStoreRef {
        DsrStoreRef::new(DsrStoreRefCreate {
            axis,
            kind,
            store_id: "workspace-mail".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            record_ref: record_ref.into(),
            data_class: privacy(DataClass::PiiIdentifying),
        })
        .unwrap()
    }

    fn dispatch(action: DsrAction, dispatch_id: &str, record_ref: &str) -> DsrDispatch {
        DsrDispatch::new(
            DsrDispatchCreate {
                dispatch_id: dispatch_id.into(),
                idempotency_key: format!("dsr-1:{dispatch_id}"),
                store: store(
                    DsrAxis::Workspace,
                    DsrStoreKind::WorkspaceObject,
                    record_ref,
                ),
                dispatched_at_epoch_seconds: 1_700_000_100,
            },
            &request(action),
        )
        .unwrap()
    }

    fn proof(action: DsrAction, dispatch: &DsrDispatch) -> ErasureProof {
        ErasureProof::new(
            ErasureProofCreate {
                proof_id: format!("proof-{}", dispatch.dispatch_id.value),
                dispatch_id: dispatch.dispatch_id.value.clone(),
                dsr_id: "dsr-1".into(),
                action,
                store: dispatch.store.value.clone(),
                method: method_for(action),
                evidence_hash: format!("sha256:evidence-{}", dispatch.dispatch_id.value),
                witness_ref: "axis-workspace".into(),
                signer_ref: "cosign://tenant-1/keyless".into(),
                signature_ref: format!("rekor://{}", dispatch.dispatch_id.value),
                rekor_log_index: 42,
                proved_at_epoch_seconds: 1_700_000_200,
            },
            dispatch,
        )
        .unwrap()
    }

    fn ack_completed(dispatch: &DsrDispatch, proof: &ErasureProof) -> DsrCascadeAck {
        DsrCascadeAck::new(
            DsrCascadeAckCreate {
                ack_id: format!("ack-{}", dispatch.dispatch_id.value),
                dispatch_id: dispatch.dispatch_id.value.clone(),
                dsr_id: "dsr-1".into(),
                status: DsrAckStatus::Completed,
                reason: None,
                proof_id: Some(proof.proof_id.value.clone()),
                evidence_hash: Some(proof.evidence_hash.value.clone()),
                acknowledged_at_epoch_seconds: 1_700_000_300,
            },
            dispatch,
            Some(proof),
        )
        .unwrap()
    }

    fn method_for(action: DsrAction) -> DsrProofMethod {
        match action {
            DsrAction::Erase => DsrProofMethod::KmsShred,
            DsrAction::Correct => DsrProofMethod::CorrectionApplied,
            DsrAction::Export => DsrProofMethod::ExportProduced,
            DsrAction::Restrict => DsrProofMethod::RestrictApplied,
            DsrAction::ObjectToProcessing => DsrProofMethod::ObjectionApplied,
            DsrAction::AutomatedDecisionOptOut => DsrProofMethod::AutomatedDecisionOptOutApplied,
        }
    }

    #[test]
    fn request_enforces_sla_and_data_class_set() {
        let request = request(DsrAction::Erase);
        assert_eq!(request.schema_version.value, 1);
        assert_eq!(
            request.subject_ref.data_class,
            DataClassification::Privacy(subject_data_class())
        );
        assert_eq!(DsrSlaTier::Preview.max_seconds(), 30 * DAY_SECONDS);
        assert_eq!(
            DsrRequest::new(DsrRequestCreate {
                dsr_id: "dsr-preview-max".into(),
                tenant_id: "tenant-1".into(),
                region: "region-alpha1".into(),
                subject_ref: "subject-1".into(),
                action: DsrAction::Erase,
                sla_tier: DsrSlaTier::Preview,
                data_classes: vec![privacy(DataClass::PiiIdentifying)],
                received_at_epoch_seconds: 1_700_000_000,
                deadline_epoch_seconds: 1_700_000_000 + DsrSlaTier::Preview.max_seconds(),
            })
            .unwrap()
            .deadline_epoch_seconds
            .value,
            1_700_000_000 + (30 * DAY_SECONDS)
        );
        assert_eq!(
            DsrRequest::new(DsrRequestCreate {
                dsr_id: "dsr-preview-late".into(),
                tenant_id: "tenant-1".into(),
                region: "region-alpha1".into(),
                subject_ref: "subject-1".into(),
                action: DsrAction::Erase,
                sla_tier: DsrSlaTier::Preview,
                data_classes: vec![privacy(DataClass::PiiIdentifying)],
                received_at_epoch_seconds: 1_700_000_000,
                deadline_epoch_seconds: 1_700_000_000 + DsrSlaTier::Preview.max_seconds() + 1,
            }),
            Err(PlatformDsrError::DeadlineExceedsSla)
        );

        assert_eq!(
            DsrRequest::new(DsrRequestCreate {
                dsr_id: "dsr-late".into(),
                tenant_id: "tenant-1".into(),
                region: "region-alpha1".into(),
                subject_ref: "subject-1".into(),
                action: DsrAction::Erase,
                sla_tier: DsrSlaTier::Ga,
                data_classes: vec![privacy(DataClass::PiiIdentifying)],
                received_at_epoch_seconds: 1_700_000_000,
                deadline_epoch_seconds: 1_700_000_000 + (8 * DAY_SECONDS),
            }),
            Err(PlatformDsrError::DeadlineExceedsSla)
        );

        assert_eq!(
            DsrRequest::new(DsrRequestCreate {
                dsr_id: "dsr-dup".into(),
                tenant_id: "tenant-1".into(),
                region: "region-alpha1".into(),
                subject_ref: "subject-1".into(),
                action: DsrAction::Export,
                sla_tier: DsrSlaTier::Preview,
                data_classes: vec![
                    privacy(DataClass::PiiIdentifying),
                    privacy(DataClass::PiiIdentifying)
                ],
                received_at_epoch_seconds: 1_700_000_000,
                deadline_epoch_seconds: 1_700_000_100,
            }),
            Err(PlatformDsrError::DuplicateDataClass)
        );
    }

    #[test]
    fn dispatch_requires_store_scope_and_idempotency() {
        let valid = dispatch(DsrAction::Erase, "dispatch-1", "record-1");
        assert_eq!(valid.action.value, DsrAction::Erase);

        assert_eq!(
            DsrDispatch::new(
                DsrDispatchCreate {
                    dispatch_id: "dispatch-2".into(),
                    idempotency_key: "".into(),
                    store: store(
                        DsrAxis::Workspace,
                        DsrStoreKind::WorkspaceObject,
                        "record-1"
                    ),
                    dispatched_at_epoch_seconds: 1_700_000_100,
                },
                &request(DsrAction::Erase),
            ),
            Err(PlatformDsrError::InvalidIdempotencyKey)
        );

        let wrong_region = DsrStoreRef::new(DsrStoreRefCreate {
            axis: DsrAxis::Workspace,
            kind: DsrStoreKind::WorkspaceObject,
            store_id: "workspace-mail".into(),
            tenant_id: "tenant-1".into(),
            region: "region-beta1".into(),
            cell_id: "cell-a".into(),
            record_ref: "record-1".into(),
            data_class: privacy(DataClass::PiiIdentifying),
        })
        .unwrap();
        assert_eq!(
            DsrDispatch::new(
                DsrDispatchCreate {
                    dispatch_id: "dispatch-3".into(),
                    idempotency_key: "dsr-1:dispatch-3".into(),
                    store: wrong_region,
                    dispatched_at_epoch_seconds: 1_700_000_100,
                },
                &request(DsrAction::Erase),
            ),
            Err(PlatformDsrError::StoreOutOfScope)
        );
    }

    #[test]
    fn proof_method_must_match_action_and_dispatch() {
        let dispatch = dispatch(DsrAction::Erase, "dispatch-1", "record-1");
        let proof = proof(DsrAction::Erase, &dispatch);
        assert_eq!(proof.method.value, DsrProofMethod::KmsShred);

        assert_eq!(
            ErasureProof::new(
                ErasureProofCreate {
                    proof_id: "proof-bad".into(),
                    dispatch_id: "dispatch-1".into(),
                    dsr_id: "dsr-1".into(),
                    action: DsrAction::Erase,
                    store: dispatch.store.value.clone(),
                    method: DsrProofMethod::ExportProduced,
                    evidence_hash: "sha256:evidence".into(),
                    witness_ref: "axis-workspace".into(),
                    signer_ref: "cosign://tenant-1/keyless".into(),
                    signature_ref: "rekor://bad".into(),
                    rekor_log_index: 43,
                    proved_at_epoch_seconds: 1_700_000_200,
                },
                &dispatch,
            ),
            Err(PlatformDsrError::ProofMethodMismatch)
        );
    }

    #[test]
    fn ack_status_controls_reason_and_proof_requirements() {
        let dispatch = dispatch(DsrAction::Export, "dispatch-1", "record-1");
        let proof = proof(DsrAction::Export, &dispatch);
        let ack = ack_completed(&dispatch, &proof);
        assert!(ack.is_terminal());

        assert_eq!(
            DsrCascadeAck::new(
                DsrCascadeAckCreate {
                    ack_id: "ack-bad".into(),
                    dispatch_id: "dispatch-1".into(),
                    dsr_id: "dsr-1".into(),
                    status: DsrAckStatus::Completed,
                    reason: None,
                    proof_id: None,
                    evidence_hash: None,
                    acknowledged_at_epoch_seconds: 1_700_000_300,
                },
                &dispatch,
                None,
            ),
            Err(PlatformDsrError::MissingCompletedProof)
        );

        assert_eq!(
            DsrCascadeAck::new(
                DsrCascadeAckCreate {
                    ack_id: "ack-fail".into(),
                    dispatch_id: "dispatch-1".into(),
                    dsr_id: "dsr-1".into(),
                    status: DsrAckStatus::RetryableFailure,
                    reason: None,
                    proof_id: None,
                    evidence_hash: None,
                    acknowledged_at_epoch_seconds: 1_700_000_300,
                },
                &dispatch,
                None,
            ),
            Err(PlatformDsrError::AckStatusInvalid)
        );
    }

    #[test]
    fn completion_requires_terminal_ack_per_dispatch_and_exact_proofs() {
        let request = request(DsrAction::Erase);
        let dispatch = dispatch(DsrAction::Erase, "dispatch-1", "record-1");
        let proof = proof(DsrAction::Erase, &dispatch);
        let ack = ack_completed(&dispatch, &proof);
        let completion = DsrCompletionRecord::new(
            DsrCompletionRecordCreate {
                completion_id: "completion-1".into(),
                dsr_id: "dsr-1".into(),
                dispatches: vec![dispatch.clone()],
                acks: vec![ack],
                proofs: vec![proof],
                aggregate_proof_hash: "sha256:aggregate".into(),
                signer_ref: "cosign://tenant-1/keyless".into(),
                signature_ref: "rekor://completion-1".into(),
                rekor_log_index: 100,
                completed_at_epoch_seconds: 1_700_000_400,
            },
            &request,
        )
        .unwrap();
        assert_eq!(
            completion.completion_status.value,
            DsrCompletionStatus::Completed
        );
        assert_eq!(completion.sla_status.value, DsrSlaStatus::WithinSla);

        let accepted = DsrCascadeAck::new(
            DsrCascadeAckCreate {
                ack_id: "ack-accepted".into(),
                dispatch_id: dispatch.dispatch_id.value.clone(),
                dsr_id: "dsr-1".into(),
                status: DsrAckStatus::Accepted,
                reason: None,
                proof_id: None,
                evidence_hash: None,
                acknowledged_at_epoch_seconds: 1_700_000_300,
            },
            &dispatch,
            None,
        )
        .unwrap();
        assert_eq!(
            DsrCompletionRecord::new(
                DsrCompletionRecordCreate {
                    completion_id: "completion-2".into(),
                    dsr_id: "dsr-1".into(),
                    dispatches: vec![dispatch],
                    acks: vec![accepted],
                    proofs: vec![],
                    aggregate_proof_hash: "sha256:aggregate".into(),
                    signer_ref: "cosign://tenant-1/keyless".into(),
                    signature_ref: "rekor://completion-2".into(),
                    rekor_log_index: 101,
                    completed_at_epoch_seconds: 1_700_000_400,
                },
                &request,
            ),
            Err(PlatformDsrError::NonTerminalAck)
        );
    }

    #[test]
    fn completion_tracks_permanent_blocks_and_late_sla() {
        let request = request(DsrAction::Erase);
        let dispatch = dispatch(DsrAction::Erase, "dispatch-1", "record-1");
        let blocked = DsrCascadeAck::new(
            DsrCascadeAckCreate {
                ack_id: "ack-blocked".into(),
                dispatch_id: "dispatch-1".into(),
                dsr_id: "dsr-1".into(),
                status: DsrAckStatus::PermanentBlock,
                reason: Some(DsrAckReason::LawfulRetention),
                proof_id: None,
                evidence_hash: None,
                acknowledged_at_epoch_seconds: 1_700_000_300,
            },
            &dispatch,
            None,
        )
        .unwrap();
        let completion = DsrCompletionRecord::new(
            DsrCompletionRecordCreate {
                completion_id: "completion-blocked".into(),
                dsr_id: "dsr-1".into(),
                dispatches: vec![dispatch],
                acks: vec![blocked],
                proofs: vec![],
                aggregate_proof_hash: "sha256:aggregate".into(),
                signer_ref: "cosign://tenant-1/keyless".into(),
                signature_ref: "rekor://completion-blocked".into(),
                rekor_log_index: 200,
                completed_at_epoch_seconds: request.deadline_epoch_seconds.value + 1,
            },
            &request,
        )
        .unwrap();
        assert_eq!(
            completion.completion_status.value,
            DsrCompletionStatus::CompletedWithBlocks
        );
        assert_eq!(completion.sla_status.value, DsrSlaStatus::Breached);
    }

    #[test]
    fn legacy_data_class_conversion_rejects_operational_markers() {
        assert_eq!(
            platform_dsr_data_class_from_legacy(DataClass::Audit),
            Err(PlatformDsrError::InvalidDataClass)
        );
        assert_eq!(
            DataClassification::from(OperationalDataClass::Audit).privacy_data_class(),
            None
        );
        assert_eq!(
            default_platform_dsr_data_class().data_class(),
            DataClass::PiiIdentifying
        );
    }
}
