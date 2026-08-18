//! Workspace DSR cascade integration kernel.
//!
//! Typed workspace-side records for the DSR cascade contract defined by
//! ADR-0038, `docs/SPEC.md`, and `docs/products/workspace/PRD.md`. This crate
//! owns Workspace impact planning, per-store proof validation, SLA evaluation,
//! and exact proof coverage. Platform DSR orchestration, audit-chain append, and
//! trust portal rendering remain outside this crate.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use compliance_retention::{
    EraseMethod, RetentionDecision, RetentionDecisionOutcome, RetentionDisposition,
    RetentionHorizon, RetentionLawfulBasis, RetentionPolicy, RetentionPolicyCreate,
    RetentionRequestKind, WorkspaceRetentionSurface,
};
use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const DSR_REQUEST_SCHEMA_VERSION: u32 = 1;
const DSR_STORE_REF_SCHEMA_VERSION: u32 = 1;
const DSR_CAPABILITY_SCHEMA_VERSION: u32 = 1;
const DSR_PLAN_SCHEMA_VERSION: u32 = 1;
const DSR_PROOF_SCHEMA_VERSION: u32 = 1;
const DSR_COMPLETION_SCHEMA_VERSION: u32 = 1;
const SHA256_PREFIX: &str = "sha256:";
const DAY_SECONDS: u64 = 86_400;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DsrError {
    InvalidDsrId,
    InvalidTenantId,
    InvalidRegion,
    InvalidSubjectRef,
    InvalidCellId,
    InvalidStoreId,
    InvalidRecordId,
    InvalidCapabilityId,
    InvalidPlanId,
    InvalidImpactId,
    InvalidProofId,
    InvalidCompletionId,
    InvalidRetentionDecisionId,
    InvalidWitnessRef,
    InvalidSignerRef,
    InvalidSignatureRef,
    InvalidEvidenceHash,
    InvalidAggregateProofHash,
    EmptyDataClassSet,
    DuplicateDataClass,
    EmptyActionSet,
    DuplicateAction,
    EmptyCapabilitySet,
    DuplicateCapabilitySurface,
    EmptyImpactSet,
    DuplicateImpactId,
    DuplicateStoreRef,
    SurfaceCapabilityMissing,
    SurfaceActionUnsupported,
    StoreOutOfScope,
    DataClassOutOfScope,
    DeadlineExceedsSla,
    InvalidTimeOrder,
    RetentionDecisionMismatch,
    RetentionDecisionNotTerminalForAction,
    EraseMethodMismatch,
    ProofCoverageMismatch,
    DuplicateProofId,
    DuplicateProofImpactId,
    InvalidDataClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DsrAction {
    Erase,
    Restrict,
    Export,
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
pub struct WorkspaceStoreRefCreate {
    pub store_id: String,                   // data_class: INTERNAL_ONLY
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub region: String,                     // data_class: INTERNAL_ONLY
    pub cell_id: String,                    // data_class: INTERNAL_ONLY
    pub surface: WorkspaceRetentionSurface, // data_class: INTERNAL_ONLY
    pub record_id: String,                  // data_class: INTERNAL_ONLY
    pub data_class: PrivacyDataClass,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorkspaceStoreRef {
    pub store_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub region: Classified<String>,    // data_class: INTERNAL_ONLY
    pub cell_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub surface: Classified<WorkspaceRetentionSurface>, // data_class: INTERNAL_ONLY
    pub record_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DsrSurfaceCapabilityCreate {
    pub capability_id: String,              // data_class: INTERNAL_ONLY
    pub surface: WorkspaceRetentionSurface, // data_class: INTERNAL_ONLY
    pub supported_actions: Vec<DsrAction>,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DsrSurfaceCapability {
    pub capability_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub surface: Classified<WorkspaceRetentionSurface>, // data_class: INTERNAL_ONLY
    pub supported_actions: Classified<Vec<DsrAction>>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DsrImpactItemCreate {
    pub impact_id: String,        // data_class: INTERNAL_ONLY
    pub store: WorkspaceStoreRef, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DsrImpactItem {
    pub impact_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub store: Classified<WorkspaceStoreRef>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DsrCascadePlanCreate {
    pub plan_id: String,                         // data_class: INTERNAL_ONLY
    pub capabilities: Vec<DsrSurfaceCapability>, // data_class: INTERNAL_ONLY
    pub items: Vec<DsrImpactItem>,               // data_class: INTERNAL_ONLY
    pub planned_at_epoch_seconds: u64,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DsrCascadePlan {
    pub plan_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub dsr_id: Classified<String>,    // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub region: Classified<String>,    // data_class: INTERNAL_ONLY
    pub action: Classified<DsrAction>, // data_class: INTERNAL_ONLY
    pub capabilities: Classified<Vec<DsrSurfaceCapability>>, // data_class: INTERNAL_ONLY
    pub items: Classified<Vec<DsrImpactItem>>, // data_class: INTERNAL_ONLY
    pub planned_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DsrStoreProofCreate {
    pub proof_id: String,                  // data_class: INTERNAL_ONLY
    pub dsr_id: String,                    // data_class: INTERNAL_ONLY
    pub impact_id: String,                 // data_class: INTERNAL_ONLY
    pub store: WorkspaceStoreRef,          // data_class: INTERNAL_ONLY
    pub action: DsrAction,                 // data_class: INTERNAL_ONLY
    pub retention_decision_id: String,     // data_class: INTERNAL_ONLY
    pub erase_method: Option<EraseMethod>, // data_class: INTERNAL_ONLY
    pub evidence_hash: String,             // data_class: INTERNAL_ONLY
    pub witness_ref: String,               // data_class: INTERNAL_ONLY
    pub signer_ref: String,                // data_class: INTERNAL_ONLY
    pub signature_ref: String,             // data_class: INTERNAL_ONLY
    pub rekor_log_index: u64,              // data_class: INTERNAL_ONLY
    pub proved_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DsrStoreProof {
    pub proof_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub dsr_id: Classified<String>,    // data_class: INTERNAL_ONLY
    pub impact_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub store: Classified<WorkspaceStoreRef>, // data_class: INTERNAL_ONLY
    pub action: Classified<DsrAction>, // data_class: INTERNAL_ONLY
    pub retention_decision_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub erase_method: Classified<Option<EraseMethod>>, // data_class: INTERNAL_ONLY
    pub evidence_hash: Classified<String>, // data_class: INTERNAL_ONLY
    pub witness_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub signer_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub signature_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub rekor_log_index: Classified<u64>, // data_class: INTERNAL_ONLY
    pub proved_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DsrCompletionRecordCreate {
    pub completion_id: String,           // data_class: INTERNAL_ONLY
    pub dsr_id: String,                  // data_class: INTERNAL_ONLY
    pub plan_id: String,                 // data_class: INTERNAL_ONLY
    pub proofs: Vec<DsrStoreProof>,      // data_class: INTERNAL_ONLY
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
    pub plan_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub proof_ids: Classified<Vec<String>>, // data_class: INTERNAL_ONLY
    pub aggregate_proof_hash: Classified<String>, // data_class: INTERNAL_ONLY
    pub signer_ref: Classified<String>,    // data_class: INTERNAL_ONLY
    pub signature_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub rekor_log_index: Classified<u64>,  // data_class: INTERNAL_ONLY
    pub sla_status: Classified<DsrSlaStatus>, // data_class: INTERNAL_ONLY
    pub completed_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,   // data_class: INTERNAL_ONLY
}

pub trait WorkspaceDsrCascadeExecutor {
    fn execute_plan(&self, plan: &DsrCascadePlan) -> Result<Vec<DsrStoreProof>, DsrError>;
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
    pub fn new(input: DsrRequestCreate) -> Result<Self, DsrError> {
        validate_non_empty(&input.dsr_id, DsrError::InvalidDsrId)?;
        validate_non_empty(&input.tenant_id, DsrError::InvalidTenantId)?;
        validate_non_empty(&input.region, DsrError::InvalidRegion)?;
        validate_non_empty(&input.subject_ref, DsrError::InvalidSubjectRef)?;
        validate_data_classes(&input.data_classes)?;
        validate_time_order(
            input.received_at_epoch_seconds,
            input.deadline_epoch_seconds,
        )?;
        let max_deadline = input
            .received_at_epoch_seconds
            .checked_add(input.sla_tier.max_seconds())
            .ok_or(DsrError::DeadlineExceedsSla)?;
        if input.deadline_epoch_seconds > max_deadline {
            return Err(DsrError::DeadlineExceedsSla);
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

impl WorkspaceStoreRef {
    pub fn new(input: WorkspaceStoreRefCreate) -> Result<Self, DsrError> {
        validate_non_empty(&input.store_id, DsrError::InvalidStoreId)?;
        validate_non_empty(&input.tenant_id, DsrError::InvalidTenantId)?;
        validate_non_empty(&input.region, DsrError::InvalidRegion)?;
        validate_non_empty(&input.cell_id, DsrError::InvalidCellId)?;
        validate_non_empty(&input.record_id, DsrError::InvalidRecordId)?;
        Ok(Self {
            store_id: internal(input.store_id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            cell_id: internal(input.cell_id),
            surface: internal(input.surface),
            record_id: internal(input.record_id),
            data_class: internal(input.data_class),
            schema_version: internal(DSR_STORE_REF_SCHEMA_VERSION),
        })
    }

    pub fn key(&self) -> (WorkspaceRetentionSurface, String, String) {
        (
            self.surface.value,
            self.store_id.value.clone(),
            self.record_id.value.clone(),
        )
    }
}

impl DsrSurfaceCapability {
    pub fn new(input: DsrSurfaceCapabilityCreate) -> Result<Self, DsrError> {
        validate_non_empty(&input.capability_id, DsrError::InvalidCapabilityId)?;
        validate_actions(&input.supported_actions)?;
        Ok(Self {
            capability_id: internal(input.capability_id),
            surface: internal(input.surface),
            supported_actions: internal(input.supported_actions),
            schema_version: internal(DSR_CAPABILITY_SCHEMA_VERSION),
        })
    }

    pub fn supports(&self, action: DsrAction) -> bool {
        self.supported_actions.value.contains(&action)
    }
}

impl DsrImpactItem {
    pub fn new(input: DsrImpactItemCreate) -> Result<Self, DsrError> {
        validate_non_empty(&input.impact_id, DsrError::InvalidImpactId)?;
        Ok(Self {
            impact_id: internal(input.impact_id),
            store: internal(input.store),
        })
    }
}

impl DsrCascadePlan {
    pub fn new(input: DsrCascadePlanCreate, request: &DsrRequest) -> Result<Self, DsrError> {
        validate_non_empty(&input.plan_id, DsrError::InvalidPlanId)?;
        validate_time_order(
            request.received_at_epoch_seconds.value,
            input.planned_at_epoch_seconds,
        )?;
        validate_capabilities(&input.capabilities)?;
        validate_items(&input.items, &input.capabilities, request)?;
        Ok(Self {
            plan_id: internal(input.plan_id),
            dsr_id: internal(request.dsr_id.value.clone()),
            tenant_id: internal(request.tenant_id.value.clone()),
            region: internal(request.region.value.clone()),
            action: internal(request.action.value),
            capabilities: internal(input.capabilities),
            items: internal(input.items),
            planned_at_epoch_seconds: internal(input.planned_at_epoch_seconds),
            schema_version: internal(DSR_PLAN_SCHEMA_VERSION),
        })
    }
}

impl DsrStoreProof {
    pub fn new(
        input: DsrStoreProofCreate,
        request: &DsrRequest,
        impact: &DsrImpactItem,
        decision: &RetentionDecision,
    ) -> Result<Self, DsrError> {
        validate_non_empty(&input.proof_id, DsrError::InvalidProofId)?;
        validate_non_empty(&input.dsr_id, DsrError::InvalidDsrId)?;
        validate_non_empty(&input.impact_id, DsrError::InvalidImpactId)?;
        validate_non_empty(
            &input.retention_decision_id,
            DsrError::InvalidRetentionDecisionId,
        )?;
        validate_hash(&input.evidence_hash, DsrError::InvalidEvidenceHash)?;
        validate_non_empty(&input.witness_ref, DsrError::InvalidWitnessRef)?;
        validate_non_empty(&input.signer_ref, DsrError::InvalidSignerRef)?;
        validate_non_empty(&input.signature_ref, DsrError::InvalidSignatureRef)?;
        validate_proof_scope(&input, request, impact)?;
        validate_retention_decision(&input, request, impact, decision)?;
        validate_time_order(
            decision.decided_at_epoch_seconds.value,
            input.proved_at_epoch_seconds,
        )?;

        Ok(Self {
            proof_id: internal(input.proof_id),
            dsr_id: internal(input.dsr_id),
            impact_id: internal(input.impact_id),
            store: internal(input.store),
            action: internal(input.action),
            retention_decision_id: internal(input.retention_decision_id),
            erase_method: internal(input.erase_method),
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

impl DsrCompletionRecord {
    pub fn new(
        input: DsrCompletionRecordCreate,
        request: &DsrRequest,
        plan: &DsrCascadePlan,
    ) -> Result<Self, DsrError> {
        validate_non_empty(&input.completion_id, DsrError::InvalidCompletionId)?;
        validate_non_empty(&input.dsr_id, DsrError::InvalidDsrId)?;
        validate_non_empty(&input.plan_id, DsrError::InvalidPlanId)?;
        validate_hash(
            &input.aggregate_proof_hash,
            DsrError::InvalidAggregateProofHash,
        )?;
        validate_non_empty(&input.signer_ref, DsrError::InvalidSignerRef)?;
        validate_non_empty(&input.signature_ref, DsrError::InvalidSignatureRef)?;
        validate_time_order(
            request.received_at_epoch_seconds.value,
            input.completed_at_epoch_seconds,
        )?;
        validate_completion_scope(&input, request, plan)?;
        let proof_ids = proof_ids(&input.proofs)?;
        let sla_status = if input.completed_at_epoch_seconds <= request.deadline_epoch_seconds.value
        {
            DsrSlaStatus::WithinSla
        } else {
            DsrSlaStatus::Breached
        };

        Ok(Self {
            completion_id: internal(input.completion_id),
            dsr_id: internal(input.dsr_id),
            plan_id: internal(input.plan_id),
            proof_ids: internal(proof_ids),
            aggregate_proof_hash: internal(input.aggregate_proof_hash),
            signer_ref: internal(input.signer_ref),
            signature_ref: internal(input.signature_ref),
            rekor_log_index: internal(input.rekor_log_index),
            sla_status: internal(sla_status),
            completed_at_epoch_seconds: internal(input.completed_at_epoch_seconds),
            schema_version: internal(DSR_COMPLETION_SCHEMA_VERSION),
        })
    }
}

pub fn default_retention_dsr_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn subject_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn retention_dsr_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, DsrError> {
    PrivacyDataClass::new(data_class).map_err(|_| DsrError::InvalidDataClass)
}

pub fn retention_policy_for_dsr(
    policy_id: String,
    tenant_id: String,
    region: String,
    surface: WorkspaceRetentionSurface,
    effective_at_epoch_seconds: u64,
) -> Result<RetentionPolicy, DsrError> {
    RetentionPolicy::new(RetentionPolicyCreate {
        policy_id,
        tenant_id,
        region,
        surface,
        horizon: RetentionHorizon::Indefinite,
        lawful_basis: RetentionLawfulBasis::Contract,
        disposition: RetentionDisposition::KmsShred,
        effective_at_epoch_seconds,
        created_at_epoch_seconds: effective_at_epoch_seconds,
        updated_at_epoch_seconds: effective_at_epoch_seconds,
    })
    .map_err(|_| DsrError::RetentionDecisionMismatch)
}

fn validate_items(
    items: &[DsrImpactItem],
    capabilities: &[DsrSurfaceCapability],
    request: &DsrRequest,
) -> Result<(), DsrError> {
    if items.is_empty() {
        return Err(DsrError::EmptyImpactSet);
    }
    let capability_by_surface = capabilities
        .iter()
        .map(|capability| (capability.surface.value, capability))
        .collect::<BTreeMap<_, _>>();
    let mut impact_ids = BTreeSet::new();
    let mut store_keys = BTreeSet::new();
    for item in items {
        if !impact_ids.insert(item.impact_id.value.clone()) {
            return Err(DsrError::DuplicateImpactId);
        }
        if !store_keys.insert(item.store.value.key()) {
            return Err(DsrError::DuplicateStoreRef);
        }
        validate_store_in_scope(&item.store.value, request)?;
        let Some(capability) = capability_by_surface.get(&item.store.value.surface.value) else {
            return Err(DsrError::SurfaceCapabilityMissing);
        };
        if !capability.supports(request.action.value) {
            return Err(DsrError::SurfaceActionUnsupported);
        }
    }
    Ok(())
}

fn validate_store_in_scope(
    store: &WorkspaceStoreRef,
    request: &DsrRequest,
) -> Result<(), DsrError> {
    if store.tenant_id.value != request.tenant_id.value
        || store.region.value != request.region.value
    {
        return Err(DsrError::StoreOutOfScope);
    }
    if !request.data_classes.value.contains(&store.data_class.value) {
        return Err(DsrError::DataClassOutOfScope);
    }
    Ok(())
}

fn validate_capabilities(capabilities: &[DsrSurfaceCapability]) -> Result<(), DsrError> {
    if capabilities.is_empty() {
        return Err(DsrError::EmptyCapabilitySet);
    }
    let mut surfaces = BTreeSet::new();
    for capability in capabilities {
        if !surfaces.insert(capability.surface.value) {
            return Err(DsrError::DuplicateCapabilitySurface);
        }
    }
    Ok(())
}

fn validate_proof_scope(
    input: &DsrStoreProofCreate,
    request: &DsrRequest,
    impact: &DsrImpactItem,
) -> Result<(), DsrError> {
    if input.dsr_id != request.dsr_id.value
        || input.impact_id != impact.impact_id.value
        || input.action != request.action.value
        || input.store != impact.store.value
    {
        return Err(DsrError::ProofCoverageMismatch);
    }
    validate_store_in_scope(&input.store, request)
}

fn validate_retention_decision(
    input: &DsrStoreProofCreate,
    request: &DsrRequest,
    impact: &DsrImpactItem,
    decision: &RetentionDecision,
) -> Result<(), DsrError> {
    if input.retention_decision_id != decision.decision_id.value
        || decision.request_id.value != request.dsr_id.value
        || decision.tenant_id.value != request.tenant_id.value
        || decision.region.value != request.region.value
        || decision.surface.value != impact.store.value.surface.value
        || decision.record_id.value != impact.store.value.record_id.value
        || decision.request_kind.value != retention_request_kind_for(request.action.value)
    {
        return Err(DsrError::RetentionDecisionMismatch);
    }
    match request.action.value {
        DsrAction::Erase => validate_erase_decision(input.erase_method, decision),
        DsrAction::Restrict => validate_no_erase_decision(
            input.erase_method,
            decision,
            RetentionDecisionOutcome::RestrictProcessing,
        ),
        DsrAction::Export => validate_no_erase_decision(
            input.erase_method,
            decision,
            RetentionDecisionOutcome::ExportOnly,
        ),
    }
}

fn validate_erase_decision(
    proof_erase_method: Option<EraseMethod>,
    decision: &RetentionDecision,
) -> Result<(), DsrError> {
    let Some(decision_erase_method) = decision.erase_method.value else {
        return Err(DsrError::RetentionDecisionNotTerminalForAction);
    };
    if proof_erase_method != Some(decision_erase_method) {
        return Err(DsrError::EraseMethodMismatch);
    }
    if !matches!(
        decision.outcome.value,
        RetentionDecisionOutcome::PermitKmsShred
            | RetentionDecisionOutcome::PermitRecordDelete
            | RetentionDecisionOutcome::PermitColdStoragePurge
    ) {
        return Err(DsrError::RetentionDecisionNotTerminalForAction);
    }
    Ok(())
}

fn validate_no_erase_decision(
    proof_erase_method: Option<EraseMethod>,
    decision: &RetentionDecision,
    expected_outcome: RetentionDecisionOutcome,
) -> Result<(), DsrError> {
    if proof_erase_method.is_some() || decision.erase_method.value.is_some() {
        return Err(DsrError::EraseMethodMismatch);
    }
    if decision.outcome.value != expected_outcome {
        return Err(DsrError::RetentionDecisionNotTerminalForAction);
    }
    Ok(())
}

fn validate_completion_scope(
    input: &DsrCompletionRecordCreate,
    request: &DsrRequest,
    plan: &DsrCascadePlan,
) -> Result<(), DsrError> {
    if input.dsr_id != request.dsr_id.value || input.plan_id != plan.plan_id.value {
        return Err(DsrError::ProofCoverageMismatch);
    }
    let expected = plan
        .items
        .value
        .iter()
        .map(|item| item.impact_id.value.clone())
        .collect::<BTreeSet<_>>();
    let mut actual = BTreeSet::new();
    let mut proof_ids = BTreeSet::new();
    for proof in &input.proofs {
        if !proof_ids.insert(proof.proof_id.value.clone()) {
            return Err(DsrError::DuplicateProofId);
        }
        if proof.dsr_id.value != request.dsr_id.value || proof.action.value != plan.action.value {
            return Err(DsrError::ProofCoverageMismatch);
        }
        if !actual.insert(proof.impact_id.value.clone()) {
            return Err(DsrError::DuplicateProofImpactId);
        }
    }
    if actual != expected {
        return Err(DsrError::ProofCoverageMismatch);
    }
    Ok(())
}

fn proof_ids(proofs: &[DsrStoreProof]) -> Result<Vec<String>, DsrError> {
    let mut proof_ids = BTreeSet::new();
    for proof in proofs {
        if !proof_ids.insert(proof.proof_id.value.clone()) {
            return Err(DsrError::DuplicateProofId);
        }
    }
    Ok(proof_ids.into_iter().collect())
}

fn retention_request_kind_for(action: DsrAction) -> RetentionRequestKind {
    match action {
        DsrAction::Erase => RetentionRequestKind::DsrErase,
        DsrAction::Restrict => RetentionRequestKind::DsrRestrict,
        DsrAction::Export => RetentionRequestKind::DsrExport,
    }
}

fn validate_actions(actions: &[DsrAction]) -> Result<(), DsrError> {
    if actions.is_empty() {
        return Err(DsrError::EmptyActionSet);
    }
    let mut seen = BTreeSet::new();
    for action in actions {
        if !seen.insert(*action) {
            return Err(DsrError::DuplicateAction);
        }
    }
    Ok(())
}

fn validate_data_classes(data_classes: &[PrivacyDataClass]) -> Result<(), DsrError> {
    if data_classes.is_empty() {
        return Err(DsrError::EmptyDataClassSet);
    }
    let mut seen = BTreeSet::new();
    for data_class in data_classes {
        if !seen.insert(*data_class) {
            return Err(DsrError::DuplicateDataClass);
        }
    }
    Ok(())
}

fn validate_hash(hash: &str, error: DsrError) -> Result<(), DsrError> {
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

fn validate_time_order(start: u64, end: u64) -> Result<(), DsrError> {
    if start > end {
        Err(DsrError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

fn validate_non_empty(value: &str, error: DsrError) -> Result<(), DsrError> {
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
    use compliance_retention::{
        RetentionDecisionCreate, RetentionRecordRef, RetentionRecordRefCreate,
    };
    use oya_data_boundary_kernel::{DataClassification, OperationalDataClass};

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

    fn store(surface: WorkspaceRetentionSurface, record_id: &str) -> WorkspaceStoreRef {
        WorkspaceStoreRef::new(WorkspaceStoreRefCreate {
            store_id: "mailbox-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            surface,
            record_id: record_id.into(),
            data_class: privacy(DataClass::PiiIdentifying),
        })
        .unwrap()
    }

    fn capability(surface: WorkspaceRetentionSurface) -> DsrSurfaceCapability {
        DsrSurfaceCapability::new(DsrSurfaceCapabilityCreate {
            capability_id: "workspace-mail-dsr".into(),
            surface,
            supported_actions: vec![DsrAction::Erase, DsrAction::Restrict, DsrAction::Export],
        })
        .unwrap()
    }

    fn impact(record_id: &str) -> DsrImpactItem {
        DsrImpactItem::new(DsrImpactItemCreate {
            impact_id: format!("impact-{record_id}"),
            store: store(WorkspaceRetentionSurface::Mail, record_id),
        })
        .unwrap()
    }

    fn plan(action: DsrAction) -> DsrCascadePlan {
        DsrCascadePlan::new(
            DsrCascadePlanCreate {
                plan_id: "plan-1".into(),
                capabilities: vec![capability(WorkspaceRetentionSurface::Mail)],
                items: vec![impact("message-1")],
                planned_at_epoch_seconds: 1_700_000_010,
            },
            &request(action),
        )
        .unwrap()
    }

    fn record(action: DsrAction, record_id: &str) -> RetentionRecordRef {
        RetentionRecordRef::new(RetentionRecordRefCreate {
            record_id: record_id.into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            surface: WorkspaceRetentionSurface::Mail,
            subject_ref: Some("subject-1".into()),
            data_class: privacy(DataClass::PiiIdentifying),
            kms_shred_key_id: if action == DsrAction::Erase {
                Some("kms-message-1".into())
            } else {
                None
            },
            created_at_epoch_seconds: 1_699_999_000,
        })
        .unwrap()
    }

    fn retention_decision(action: DsrAction, record_id: &str) -> RetentionDecision {
        let policy = retention_policy_for_dsr(
            "retention-dsr".into(),
            "tenant-1".into(),
            "region-alpha1".into(),
            WorkspaceRetentionSurface::Mail,
            1_699_000_000,
        )
        .unwrap();
        RetentionDecision::evaluate(
            RetentionDecisionCreate {
                decision_id: format!("decision-{record_id}"),
                request_id: "dsr-1".into(),
                request_kind: retention_request_kind_for(action),
                requested_by_actor_ref: "dpo-1".into(),
                decided_at_epoch_seconds: 1_700_000_100,
            },
            &policy,
            &record(action, record_id),
            &[],
        )
        .unwrap()
    }

    fn proof(action: DsrAction) -> DsrStoreProof {
        DsrStoreProof::new(
            DsrStoreProofCreate {
                proof_id: "proof-1".into(),
                dsr_id: "dsr-1".into(),
                impact_id: "impact-message-1".into(),
                store: store(WorkspaceRetentionSurface::Mail, "message-1"),
                action,
                retention_decision_id: "decision-message-1".into(),
                erase_method: if action == DsrAction::Erase {
                    Some(EraseMethod::KmsShred)
                } else {
                    None
                },
                evidence_hash: "sha256:evidence".into(),
                witness_ref: "workspace-mail-worker".into(),
                signer_ref: "cosign://tenant-1/keyless".into(),
                signature_ref: "rekor://entry-1".into(),
                rekor_log_index: 100,
                proved_at_epoch_seconds: 1_700_000_200,
            },
            &request(action),
            &impact("message-1"),
            &retention_decision(action, "message-1"),
        )
        .unwrap()
    }

    #[test]
    fn request_enforces_sla_and_data_class_scope() {
        let request = request(DsrAction::Erase);
        assert_eq!(request.schema_version.value, 1);
        assert_eq!(
            request.subject_ref.data_class,
            DataClassification::Privacy(subject_data_class())
        );

        assert_eq!(
            DsrRequest::new(DsrRequestCreate {
                dsr_id: "dsr-2".into(),
                tenant_id: "tenant-1".into(),
                region: "region-alpha1".into(),
                subject_ref: "subject-1".into(),
                action: DsrAction::Erase,
                sla_tier: DsrSlaTier::Ga,
                data_classes: vec![privacy(DataClass::PiiIdentifying)],
                received_at_epoch_seconds: 1_700_000_000,
                deadline_epoch_seconds: 1_700_000_000 + (8 * DAY_SECONDS),
            }),
            Err(DsrError::DeadlineExceedsSla)
        );

        assert_eq!(
            DsrRequest::new(DsrRequestCreate {
                dsr_id: "dsr-3".into(),
                tenant_id: "tenant-1".into(),
                region: "region-alpha1".into(),
                subject_ref: "subject-1".into(),
                action: DsrAction::Erase,
                sla_tier: DsrSlaTier::Preview,
                data_classes: vec![
                    privacy(DataClass::PiiIdentifying),
                    privacy(DataClass::PiiIdentifying)
                ],
                received_at_epoch_seconds: 1_700_000_000,
                deadline_epoch_seconds: 1_700_000_100,
            }),
            Err(DsrError::DuplicateDataClass)
        );
    }

    #[test]
    fn plan_requires_surface_capability_and_exact_scope() {
        let good = plan(DsrAction::Erase);
        assert_eq!(good.items.value.len(), 1);

        assert_eq!(
            DsrCascadePlan::new(
                DsrCascadePlanCreate {
                    plan_id: "plan-2".into(),
                    capabilities: vec![],
                    items: vec![impact("message-1")],
                    planned_at_epoch_seconds: 1_700_000_010,
                },
                &request(DsrAction::Erase),
            ),
            Err(DsrError::EmptyCapabilitySet)
        );

        assert_eq!(
            DsrCascadePlan::new(
                DsrCascadePlanCreate {
                    plan_id: "plan-3".into(),
                    capabilities: vec![
                        DsrSurfaceCapability::new(DsrSurfaceCapabilityCreate {
                            capability_id: "workspace-mail-dsr".into(),
                            surface: WorkspaceRetentionSurface::Mail,
                            supported_actions: vec![DsrAction::Export],
                        })
                        .unwrap()
                    ],
                    items: vec![impact("message-1")],
                    planned_at_epoch_seconds: 1_700_000_010,
                },
                &request(DsrAction::Erase),
            ),
            Err(DsrError::SurfaceActionUnsupported)
        );
    }

    #[test]
    fn store_proof_requires_matching_terminal_retention_decision() {
        let erase_proof = proof(DsrAction::Erase);
        assert_eq!(erase_proof.erase_method.value, Some(EraseMethod::KmsShred));
        assert_eq!(erase_proof.schema_version.value, 1);

        let wrong_method = DsrStoreProof::new(
            DsrStoreProofCreate {
                proof_id: "proof-2".into(),
                dsr_id: "dsr-1".into(),
                impact_id: "impact-message-1".into(),
                store: store(WorkspaceRetentionSurface::Mail, "message-1"),
                action: DsrAction::Erase,
                retention_decision_id: "decision-message-1".into(),
                erase_method: Some(EraseMethod::RecordDelete),
                evidence_hash: "sha256:evidence".into(),
                witness_ref: "workspace-mail-worker".into(),
                signer_ref: "cosign://tenant-1/keyless".into(),
                signature_ref: "rekor://entry-2".into(),
                rekor_log_index: 101,
                proved_at_epoch_seconds: 1_700_000_200,
            },
            &request(DsrAction::Erase),
            &impact("message-1"),
            &retention_decision(DsrAction::Erase, "message-1"),
        );
        assert_eq!(wrong_method, Err(DsrError::EraseMethodMismatch));

        let export_proof = proof(DsrAction::Export);
        assert_eq!(export_proof.erase_method.value, None);
    }

    #[test]
    fn proof_rejects_non_terminal_retention_decision() {
        let mut blocked_decision = retention_decision(DsrAction::Restrict, "message-1");
        blocked_decision.outcome.value = RetentionDecisionOutcome::RetainUnderLawfulBasis;
        assert_eq!(
            DsrStoreProof::new(
                DsrStoreProofCreate {
                    proof_id: "proof-blocked".into(),
                    dsr_id: "dsr-1".into(),
                    impact_id: "impact-message-1".into(),
                    store: store(WorkspaceRetentionSurface::Mail, "message-1"),
                    action: DsrAction::Restrict,
                    retention_decision_id: "decision-message-1".into(),
                    erase_method: None,
                    evidence_hash: "sha256:evidence".into(),
                    witness_ref: "workspace-mail-worker".into(),
                    signer_ref: "cosign://tenant-1/keyless".into(),
                    signature_ref: "rekor://entry-blocked".into(),
                    rekor_log_index: 102,
                    proved_at_epoch_seconds: 1_700_000_200,
                },
                &request(DsrAction::Restrict),
                &impact("message-1"),
                &blocked_decision,
            ),
            Err(DsrError::RetentionDecisionNotTerminalForAction)
        );
    }

    #[test]
    fn completion_requires_exact_proof_coverage_and_tracks_sla() {
        let request = request(DsrAction::Erase);
        let plan = plan(DsrAction::Erase);
        let completion = DsrCompletionRecord::new(
            DsrCompletionRecordCreate {
                completion_id: "completion-1".into(),
                dsr_id: "dsr-1".into(),
                plan_id: "plan-1".into(),
                proofs: vec![proof(DsrAction::Erase)],
                aggregate_proof_hash: "sha256:aggregate".into(),
                signer_ref: "cosign://tenant-1/keyless".into(),
                signature_ref: "rekor://completion-1".into(),
                rekor_log_index: 200,
                completed_at_epoch_seconds: 1_700_000_300,
            },
            &request,
            &plan,
        )
        .unwrap();
        assert_eq!(completion.sla_status.value, DsrSlaStatus::WithinSla);
        assert_eq!(completion.proof_ids.value, vec!["proof-1"]);

        assert_eq!(
            DsrCompletionRecord::new(
                DsrCompletionRecordCreate {
                    completion_id: "completion-2".into(),
                    dsr_id: "dsr-1".into(),
                    plan_id: "plan-1".into(),
                    proofs: vec![],
                    aggregate_proof_hash: "sha256:aggregate".into(),
                    signer_ref: "cosign://tenant-1/keyless".into(),
                    signature_ref: "rekor://completion-2".into(),
                    rekor_log_index: 201,
                    completed_at_epoch_seconds: 1_700_000_300,
                },
                &request,
                &plan,
            ),
            Err(DsrError::ProofCoverageMismatch)
        );

        let late = DsrCompletionRecord::new(
            DsrCompletionRecordCreate {
                completion_id: "completion-3".into(),
                dsr_id: "dsr-1".into(),
                plan_id: "plan-1".into(),
                proofs: vec![proof(DsrAction::Erase)],
                aggregate_proof_hash: "sha256:aggregate".into(),
                signer_ref: "cosign://tenant-1/keyless".into(),
                signature_ref: "rekor://completion-3".into(),
                rekor_log_index: 202,
                completed_at_epoch_seconds: request.deadline_epoch_seconds.value + 1,
            },
            &request,
            &plan,
        )
        .unwrap();
        assert_eq!(late.sla_status.value, DsrSlaStatus::Breached);
    }

    #[test]
    fn legacy_data_class_conversion_rejects_operational_markers() {
        assert_eq!(
            retention_dsr_data_class_from_legacy(DataClass::Audit),
            Err(DsrError::InvalidDataClass)
        );
        assert_eq!(
            DataClassification::from(OperationalDataClass::Audit).privacy_data_class(),
            None
        );
        assert_eq!(
            default_retention_dsr_data_class().data_class(),
            DataClass::PiiIdentifying
        );
    }
}
