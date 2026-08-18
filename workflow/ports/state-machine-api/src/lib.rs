//! Workflow-engine state-machine API boundary foundation.
//!
//! This crate owns a source-level, framework-free API boundary for applying
//! typed workflow state-machine transitions. It validates header/principal/
//! authorization/version binding, maps API event bodies to AsyncAPI lifecycle
//! event variants, delegates to the state-machine usecase, maps checkpoint
//! receipts to OpenAPI/proto status DTOs, and preserves in-memory idempotent API
//! replay semantics. It performs no HTTP serving, serialization-framework work,
//! concrete storage, network I/O, durable idempotency storage, queue processing,
//! or cloud runtime execution.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

pub use workflow_state_machine_domain::TransitionOrigin;
pub use workflow_state_machine_kernel::{
    StateCheckpoint, StepStatus, TransitionEventValidationError, WorkflowEventKind,
    WorkflowRunStatus, WorkflowTransitionEvent,
};
pub use workflow_state_machine_usecase::{
    StateCheckpointAppendFailure, StateCheckpointStoreFailure, StateCheckpointStorePort,
    StateMachineTransitionUsecaseInput, StateMachineUsecaseReceipt, StateMachineUsecaseStatus,
    apply_state_machine_transition,
};

pub const STATE_MACHINE_API_SURFACE: &str = "workflow-engine.state-machine.transition";
pub const STATE_MACHINE_API_DECLARED_VERSION: &str = "2026-05-21";
pub const STATE_MACHINE_API_CONTRACT_REF: &str =
    "workflow/workflow-engine/contracts/openapi/workflow-engine.yaml";
pub const STATE_MACHINE_TRANSITION_ROUTE: &str =
    "/v/2026-05-21/runs/{run_id}/state-machine/transitions";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateMachineApiStatus {
    Accepted,
    BadRequest,
    Forbidden,
    Conflict,
    FailedDependency,
    ServiceUnavailable,
}

impl StateMachineApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Accepted => 202,
            Self::BadRequest => 400,
            Self::Forbidden => 403,
            Self::Conflict => 409,
            Self::FailedDependency => 424,
            Self::ServiceUnavailable => 503,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum StateMachineApiErrorCode {
    AuthorizationDenied,
    AuthorizationEvidenceInvalid,
    AuthorizationPrincipalMismatch,
    AuthorizationTenantMismatch,
    CheckpointAppendConflict,
    ContractVersionUnsupported,
    DomainDenied,
    IdempotencyKeyReused,
    MissingStepIndex,
    RequestIdEmpty,
    StoreUnavailable,
    TenantBindingMismatch,
    TenantHeaderEmpty,
    TraceContextInvalid,
    UnknownTransitionEvent,
    UnsafeMetadata,
}

impl StateMachineApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AuthorizationDenied => "WORKFLOW_STATE_MACHINE_AUTHORIZATION_DENIED",
            Self::AuthorizationEvidenceInvalid => {
                "WORKFLOW_STATE_MACHINE_AUTHORIZATION_EVIDENCE_INVALID"
            }
            Self::AuthorizationPrincipalMismatch => {
                "WORKFLOW_STATE_MACHINE_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationTenantMismatch => {
                "WORKFLOW_STATE_MACHINE_AUTHORIZATION_TENANT_MISMATCH"
            }
            Self::CheckpointAppendConflict => "WORKFLOW_STATE_MACHINE_CHECKPOINT_APPEND_CONFLICT",
            Self::ContractVersionUnsupported => {
                "WORKFLOW_STATE_MACHINE_CONTRACT_VERSION_UNSUPPORTED"
            }
            Self::DomainDenied => "WORKFLOW_STATE_MACHINE_DOMAIN_DENIED",
            Self::IdempotencyKeyReused => "WORKFLOW_STATE_MACHINE_IDEMPOTENCY_KEY_REUSED",
            Self::MissingStepIndex => "WORKFLOW_STATE_MACHINE_STEP_INDEX_REQUIRED",
            Self::RequestIdEmpty => "WORKFLOW_STATE_MACHINE_REQUEST_ID_EMPTY",
            Self::StoreUnavailable => "WORKFLOW_STATE_MACHINE_STORE_UNAVAILABLE",
            Self::TenantBindingMismatch => "WORKFLOW_STATE_MACHINE_TENANT_BINDING_MISMATCH",
            Self::TenantHeaderEmpty => "WORKFLOW_STATE_MACHINE_TENANT_HEADER_EMPTY",
            Self::TraceContextInvalid => "WORKFLOW_STATE_MACHINE_TRACE_CONTEXT_INVALID",
            Self::UnknownTransitionEvent => "WORKFLOW_STATE_MACHINE_UNKNOWN_TRANSITION_EVENT",
            Self::UnsafeMetadata => "WORKFLOW_STATE_MACHINE_UNSAFE_METADATA",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateMachineApiBoundaryContext {
    pub request_id: String,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,         // data_class: INTERNAL_ONLY
    pub idempotency_key: String,   // data_class: INTERNAL_ONLY
    pub trace_context_ref: String, // data_class: INTERNAL_ONLY
    pub oyatie_version: String,    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateMachineApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateMachineApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub evidence_ref: String,          // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateMachineApiRequest {
    pub boundary: StateMachineApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: StateMachineApiPrincipal,      // data_class: INTERNAL_ONLY
    pub authorization: StateMachineApiAuthorization, // data_class: INTERNAL_ONLY
    pub expected_spec_id: String,                 // data_class: INTERNAL_ONLY
    pub expected_version_sha: String,             // data_class: INTERNAL_ONLY
    pub spec_integrity_ref: String,               // data_class: INTERNAL_ONLY
    pub replay_epoch_ref: String,                 // data_class: INTERNAL_ONLY
    pub body: StateMachineApiTransitionEventBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateMachineApiTransitionEventBody {
    pub event_id: String,                   // data_class: INTERNAL_ONLY
    pub run_id: String,                     // data_class: INTERNAL_ONLY
    pub spec_id: String,                    // data_class: INTERNAL_ONLY
    pub version_sha: String,                // data_class: INTERNAL_ONLY
    pub sequence_num: u64,                  // data_class: INTERNAL_ONLY
    pub event_type: String,                 // data_class: INTERNAL_ONLY
    pub step_index: Option<u32>,            // data_class: INTERNAL_ONLY
    pub retry_count: Option<u32>,           // data_class: INTERNAL_ONLY
    pub retry_attempt: Option<u32>,         // data_class: INTERNAL_ONLY
    pub policy_context_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub event_evidence_ref: String,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateMachineApiSuccessResponse {
    pub status: StateMachineApiStatus,             // data_class: PUBLIC
    pub route: String,                             // data_class: PUBLIC
    pub run: StateMachineRunDto,                   // data_class: INTERNAL_ONLY
    pub step: Option<StateMachineStepDto>,         // data_class: INTERNAL_ONLY
    pub metadata: StateMachineApiResponseMetadata, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                // data_class: INTERNAL_ONLY
}

impl StateMachineApiSuccessResponse {
    pub fn http_status_code(&self) -> u16 {
        self.status.code()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateMachineApiResponseMetadata {
    pub request_id: String,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,         // data_class: INTERNAL_ONLY
    pub idempotency_key: String,   // data_class: INTERNAL_ONLY
    pub trace_context_ref: String, // data_class: INTERNAL_ONLY
    pub surface: String,           // data_class: INTERNAL_ONLY
    pub contract_ref: String,      // data_class: INTERNAL_ONLY
    pub oyatie_version: String,    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateMachineRunDto {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub run_id: String,                  // data_class: INTERNAL_ONLY
    pub spec_id: String,                 // data_class: INTERNAL_ONLY
    pub version_sha: String,             // data_class: INTERNAL_ONLY
    pub run_status: String,              // data_class: PUBLIC
    pub proto_run_status: String,        // data_class: PUBLIC
    pub current_step_index: Option<u32>, // data_class: INTERNAL_ONLY
    pub current_state: String,           // data_class: PUBLIC
    pub checkpoint_seq: u64,             // data_class: INTERNAL_ONLY
    pub last_event_type: String,         // data_class: INTERNAL_ONLY
}

impl StateMachineRunDto {
    pub fn from_checkpoint(checkpoint: &StateCheckpoint) -> Self {
        Self {
            tenant_id: checkpoint.tenant_id.clone(),
            run_id: checkpoint.run_id.clone(),
            spec_id: checkpoint.spec_id.clone(),
            version_sha: checkpoint.version_sha.clone(),
            run_status: checkpoint.run_status.as_wire().to_owned(),
            proto_run_status: proto_run_status(checkpoint.run_status).to_owned(),
            current_step_index: checkpoint.current_step_index,
            current_state: checkpoint.run_status.as_wire().to_owned(),
            checkpoint_seq: checkpoint.checkpoint_seq,
            last_event_type: checkpoint.last_event_type.clone(),
        }
    }

    pub fn run_status_from_wire(value: &str) -> Option<WorkflowRunStatus> {
        WorkflowRunStatus::from_wire(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateMachineStepDto {
    pub run_id: String,            // data_class: INTERNAL_ONLY
    pub step_index: u32,           // data_class: INTERNAL_ONLY
    pub step_status: String,       // data_class: PUBLIC
    pub proto_step_status: String, // data_class: PUBLIC
    pub checkpoint_seq: u64,       // data_class: INTERNAL_ONLY
}

impl StateMachineStepDto {
    pub fn from_checkpoint(checkpoint: &StateCheckpoint) -> Option<Self> {
        Some(Self {
            run_id: checkpoint.run_id.clone(),
            step_index: checkpoint.current_step_index?,
            step_status: checkpoint.step_status?.as_wire().to_owned(),
            proto_step_status: proto_step_status(checkpoint.step_status?).to_owned(),
            checkpoint_seq: checkpoint.checkpoint_seq,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StateMachineApiError {
    Boundary {
        code: StateMachineApiErrorCode,
        status: StateMachineApiStatus,
        evidence_ref: String,
    },
    IdempotencyConflict,
    UsecaseDenied {
        status: StateMachineApiStatus,
        code: StateMachineApiErrorCode,
        evidence_refs: Vec<String>,
    },
}

impl StateMachineApiError {
    pub fn status(&self) -> StateMachineApiStatus {
        match self {
            Self::Boundary { status, .. } | Self::UsecaseDenied { status, .. } => *status,
            Self::IdempotencyConflict => StateMachineApiStatus::Conflict,
        }
    }

    pub fn status_code(&self) -> u16 {
        self.status().code()
    }

    pub fn code(&self) -> StateMachineApiErrorCode {
        match self {
            Self::Boundary { code, .. } | Self::UsecaseDenied { code, .. } => *code,
            Self::IdempotencyConflict => StateMachineApiErrorCode::IdempotencyKeyReused,
        }
    }
}

#[derive(Default)]
pub struct WorkflowStateMachineApi {
    responses_by_idempotency: BTreeMap<String, (String, StateMachineApiSuccessResponse)>,
}

impl WorkflowStateMachineApi {
    pub fn apply_transition<S: StateCheckpointStorePort>(
        &mut self,
        store: &mut S,
        request: StateMachineApiRequest,
    ) -> Result<StateMachineApiSuccessResponse, StateMachineApiError> {
        validate_boundary(&request)?;
        let fingerprint = request_fingerprint(&request);
        let cache_key = idempotency_cache_key(&request);
        if let Some((cached_fingerprint, response)) = self.responses_by_idempotency.get(&cache_key)
        {
            if cached_fingerprint == &fingerprint {
                return Ok(response.clone());
            }
            return Err(StateMachineApiError::IdempotencyConflict);
        }

        let event = request.body.to_kernel_event(&request.boundary.tenant_id)?;
        let input = StateMachineTransitionUsecaseInput {
            request_id: request.boundary.request_id.clone(),
            idempotency_key: request.boundary.idempotency_key.clone(),
            trace_ref: request.boundary.trace_context_ref.clone(),
            event,
            expected_tenant_id: request.boundary.tenant_id.clone(),
            expected_spec_id: request.expected_spec_id.clone(),
            expected_version_sha: request.expected_version_sha.clone(),
            policy_evidence_ref: request.authorization.evidence_ref.clone(),
            spec_integrity_ref: request.spec_integrity_ref.clone(),
            replay_epoch_ref: request.replay_epoch_ref.clone(),
            origin: TransitionOrigin::ApiCommand,
        };

        let receipt = apply_state_machine_transition(store, input);
        let response = map_usecase_receipt(&request, receipt)?;
        self.responses_by_idempotency
            .insert(cache_key, (fingerprint, response.clone()));
        Ok(response)
    }
}

impl StateMachineApiTransitionEventBody {
    fn to_kernel_event(
        &self,
        tenant_id: &str,
    ) -> Result<WorkflowTransitionEvent, StateMachineApiError> {
        let kind = self.to_event_kind()?;
        WorkflowTransitionEvent::new(
            &self.event_id,
            tenant_id,
            &self.run_id,
            &self.spec_id,
            &self.version_sha,
            self.sequence_num,
            kind,
            &self.event_evidence_ref,
        )
        .map_err(validation_error_to_api)
    }

    fn to_event_kind(&self) -> Result<WorkflowEventKind, StateMachineApiError> {
        match self.event_type.as_str() {
            "WorkflowStarted" => Ok(WorkflowEventKind::WorkflowStarted),
            "StepStarted" => Ok(WorkflowEventKind::StepStarted {
                step_index: self.step_index.ok_or_else(missing_step_index_error)?,
            }),
            "StepCompleted" => Ok(WorkflowEventKind::StepCompleted {
                step_index: self.step_index.ok_or_else(missing_step_index_error)?,
            }),
            "StepFailed" => Ok(WorkflowEventKind::StepFailed {
                step_index: self.step_index.ok_or_else(missing_step_index_error)?,
                retry_count: self.retry_count.unwrap_or(0),
            }),
            "StepRetried" => Ok(WorkflowEventKind::StepRetried {
                step_index: self.step_index.ok_or_else(missing_step_index_error)?,
                retry_attempt: self.retry_attempt.unwrap_or(1),
            }),
            "WorkflowPaused" => Ok(WorkflowEventKind::WorkflowPaused {
                policy_context_ref: self.policy_context_ref.clone().unwrap_or_default(),
            }),
            "WorkflowResumed" => Ok(WorkflowEventKind::WorkflowResumed),
            "WorkflowCancelled" => Ok(WorkflowEventKind::WorkflowCancelled {
                policy_context_ref: self.policy_context_ref.clone().unwrap_or_default(),
            }),
            "WorkflowCompleted" => Ok(WorkflowEventKind::WorkflowCompleted),
            "WorkflowFailed" => Ok(WorkflowEventKind::WorkflowFailed),
            _ => Err(boundary_error(
                StateMachineApiErrorCode::UnknownTransitionEvent,
                StateMachineApiStatus::BadRequest,
                "workflow-state-machine-api:unknown-event-type",
            )),
        }
    }
}

fn map_usecase_receipt(
    request: &StateMachineApiRequest,
    receipt: StateMachineUsecaseReceipt,
) -> Result<StateMachineApiSuccessResponse, StateMachineApiError> {
    match receipt.status {
        StateMachineUsecaseStatus::Applied => {
            let checkpoint = receipt.checkpoint.ok_or_else(|| {
                boundary_error(
                    StateMachineApiErrorCode::StoreUnavailable,
                    StateMachineApiStatus::ServiceUnavailable,
                    "workflow-state-machine-api:missing-checkpoint",
                )
            })?;
            Ok(StateMachineApiSuccessResponse {
                status: StateMachineApiStatus::Accepted,
                route: STATE_MACHINE_TRANSITION_ROUTE.to_owned(),
                run: StateMachineRunDto::from_checkpoint(&checkpoint),
                step: StateMachineStepDto::from_checkpoint(&checkpoint),
                metadata: response_metadata(request),
                evidence_refs: sorted_unique(receipt.evidence_refs),
            })
        }
        StateMachineUsecaseStatus::DomainDenied => Err(StateMachineApiError::UsecaseDenied {
            status: StateMachineApiStatus::Forbidden,
            code: StateMachineApiErrorCode::DomainDenied,
            evidence_refs: sorted_unique(receipt.evidence_refs),
        }),
        StateMachineUsecaseStatus::InvalidInput => Err(StateMachineApiError::UsecaseDenied {
            status: StateMachineApiStatus::BadRequest,
            code: StateMachineApiErrorCode::UnsafeMetadata,
            evidence_refs: sorted_unique(receipt.evidence_refs),
        }),
        StateMachineUsecaseStatus::StoreConflict => Err(StateMachineApiError::UsecaseDenied {
            status: StateMachineApiStatus::Conflict,
            code: StateMachineApiErrorCode::CheckpointAppendConflict,
            evidence_refs: sorted_unique(receipt.evidence_refs),
        }),
        StateMachineUsecaseStatus::StoreUnavailable => Err(StateMachineApiError::UsecaseDenied {
            status: StateMachineApiStatus::ServiceUnavailable,
            code: StateMachineApiErrorCode::StoreUnavailable,
            evidence_refs: sorted_unique(receipt.evidence_refs),
        }),
    }
}

fn validate_boundary(request: &StateMachineApiRequest) -> Result<(), StateMachineApiError> {
    if request.boundary.request_id.trim().is_empty() {
        return Err(boundary_error(
            StateMachineApiErrorCode::RequestIdEmpty,
            StateMachineApiStatus::BadRequest,
            "workflow-state-machine-api:request-id-required",
        ));
    }
    if request.boundary.tenant_id.trim().is_empty() {
        return Err(boundary_error(
            StateMachineApiErrorCode::TenantHeaderEmpty,
            StateMachineApiStatus::BadRequest,
            "workflow-state-machine-api:tenant-required",
        ));
    }
    if request.boundary.oyatie_version != STATE_MACHINE_API_DECLARED_VERSION {
        return Err(boundary_error(
            StateMachineApiErrorCode::ContractVersionUnsupported,
            StateMachineApiStatus::BadRequest,
            "workflow-state-machine-api:unsupported-version",
        ));
    }
    if !is_safe_ref(&request.boundary.request_id)
        || !is_safe_ref(&request.boundary.idempotency_key)
        || !is_safe_ref(&request.boundary.trace_context_ref)
        || !is_safe_ref(&request.expected_spec_id)
        || !is_safe_ref(&request.expected_version_sha)
        || !is_safe_ref(&request.spec_integrity_ref)
        || !is_safe_ref(&request.replay_epoch_ref)
    {
        return Err(boundary_error(
            StateMachineApiErrorCode::UnsafeMetadata,
            StateMachineApiStatus::BadRequest,
            "workflow-state-machine-api:unsafe-metadata",
        ));
    }
    if request.principal.tenant_id != request.boundary.tenant_id {
        return Err(boundary_error(
            StateMachineApiErrorCode::TenantBindingMismatch,
            StateMachineApiStatus::Forbidden,
            "workflow-state-machine-api:principal-tenant-mismatch",
        ));
    }
    if request.authorization.tenant_id != request.boundary.tenant_id {
        return Err(boundary_error(
            StateMachineApiErrorCode::AuthorizationTenantMismatch,
            StateMachineApiStatus::Forbidden,
            "workflow-state-machine-api:auth-tenant-mismatch",
        ));
    }
    if request.authorization.principal_id != request.principal.principal_id {
        return Err(boundary_error(
            StateMachineApiErrorCode::AuthorizationPrincipalMismatch,
            StateMachineApiStatus::Forbidden,
            "workflow-state-machine-api:auth-principal-mismatch",
        ));
    }
    if !is_safe_ref(&request.authorization.decision_id)
        || !is_safe_ref(&request.authorization.evidence_ref)
    {
        return Err(boundary_error(
            StateMachineApiErrorCode::AuthorizationEvidenceInvalid,
            StateMachineApiStatus::Forbidden,
            "workflow-state-machine-api:auth-evidence-invalid",
        ));
    }
    if !request
        .authorization
        .allowed_surfaces
        .iter()
        .any(|surface| surface == STATE_MACHINE_API_SURFACE)
    {
        return Err(boundary_error(
            StateMachineApiErrorCode::AuthorizationDenied,
            StateMachineApiStatus::Forbidden,
            "workflow-state-machine-api:surface-denied",
        ));
    }
    Ok(())
}

fn response_metadata(request: &StateMachineApiRequest) -> StateMachineApiResponseMetadata {
    StateMachineApiResponseMetadata {
        request_id: request.boundary.request_id.clone(),
        tenant_id: request.boundary.tenant_id.clone(),
        idempotency_key: request.boundary.idempotency_key.clone(),
        trace_context_ref: request.boundary.trace_context_ref.clone(),
        surface: STATE_MACHINE_API_SURFACE.to_owned(),
        contract_ref: STATE_MACHINE_API_CONTRACT_REF.to_owned(),
        oyatie_version: STATE_MACHINE_API_DECLARED_VERSION.to_owned(),
    }
}

fn request_fingerprint(request: &StateMachineApiRequest) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{:?}",
        request.boundary.tenant_id,
        request.principal.principal_id,
        request.authorization.decision_id,
        request.expected_spec_id,
        request.expected_version_sha,
        request.spec_integrity_ref,
        request.replay_epoch_ref,
        request.body.event_id,
        request.body.event_type,
        request.body.sequence_num,
        request.body
    )
}

fn idempotency_cache_key(request: &StateMachineApiRequest) -> String {
    format!(
        "{}|{}",
        request.boundary.tenant_id, request.boundary.idempotency_key
    )
}

fn missing_step_index_error() -> StateMachineApiError {
    boundary_error(
        StateMachineApiErrorCode::MissingStepIndex,
        StateMachineApiStatus::BadRequest,
        "workflow-state-machine-api:step-index-required",
    )
}

fn validation_error_to_api(error: TransitionEventValidationError) -> StateMachineApiError {
    let evidence_ref = match error {
        TransitionEventValidationError::InvalidEventId => {
            "workflow-state-machine-api:event-id-invalid"
        }
        TransitionEventValidationError::InvalidTenantId => {
            "workflow-state-machine-api:tenant-invalid"
        }
        TransitionEventValidationError::InvalidRunId => "workflow-state-machine-api:run-id-invalid",
        TransitionEventValidationError::InvalidSpecId => {
            "workflow-state-machine-api:spec-id-invalid"
        }
        TransitionEventValidationError::InvalidVersionSha => {
            "workflow-state-machine-api:version-sha-invalid"
        }
        TransitionEventValidationError::InvalidSequenceNum => {
            "workflow-state-machine-api:sequence-invalid"
        }
        TransitionEventValidationError::InvalidEvidenceRef => {
            "workflow-state-machine-api:evidence-ref-invalid"
        }
    };
    boundary_error(
        StateMachineApiErrorCode::UnsafeMetadata,
        StateMachineApiStatus::BadRequest,
        evidence_ref,
    )
}

fn boundary_error(
    code: StateMachineApiErrorCode,
    status: StateMachineApiStatus,
    evidence_ref: &str,
) -> StateMachineApiError {
    StateMachineApiError::Boundary {
        code,
        status,
        evidence_ref: evidence_ref.to_owned(),
    }
}

fn proto_run_status(status: WorkflowRunStatus) -> &'static str {
    match status {
        WorkflowRunStatus::Running => "RUN_STATUS_RUNNING",
        WorkflowRunStatus::Paused => "RUN_STATUS_PAUSED",
        WorkflowRunStatus::Waiting => "RUN_STATUS_WAITING",
        WorkflowRunStatus::Completed => "RUN_STATUS_COMPLETED",
        WorkflowRunStatus::Failed => "RUN_STATUS_FAILED",
        WorkflowRunStatus::Cancelled => "RUN_STATUS_CANCELLED",
    }
}

fn proto_step_status(status: StepStatus) -> &'static str {
    match status {
        StepStatus::Pending => "STEP_STATUS_PENDING",
        StepStatus::Running => "STEP_STATUS_RUNNING",
        StepStatus::Completed => "STEP_STATUS_COMPLETED",
        StepStatus::Failed => "STEP_STATUS_FAILED",
        StepStatus::Retrying => "STEP_STATUS_RETRYING",
    }
}

fn is_safe_ref(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && value == trimmed
        && value.contains(':')
        && !value.chars().any(char::is_whitespace)
        && !contains_raw_secret_material(value)
        && !contains_raw_content_material(value)
}

fn contains_raw_secret_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("sk-")
        || lower.contains("sk-")
        || lower.contains("bearer")
        || lower.contains("authorization:")
        || lower.contains("api_key=")
        || lower.contains("openai_api_key")
        || lower.contains("private key")
        || lower.contains("-----begin")
}

fn contains_raw_content_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw model")
        || lower.contains("write an email")
        || lower.contains("customer message")
        || lower.contains("model answer")
        || lower.contains("raw output")
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct FakeStore {
        current: Option<StateCheckpoint>,
        appended: usize,
        load_calls: usize,
        conflict: bool,
        unavailable: bool,
    }

    impl StateCheckpointStorePort for FakeStore {
        fn load_current(
            &mut self,
            _tenant_id: &str,
            _run_id: &str,
        ) -> Result<Option<StateCheckpoint>, StateCheckpointStoreFailure> {
            self.load_calls += 1;
            if self.unavailable {
                return Err(StateCheckpointStoreFailure::Unavailable {
                    evidence_ref: "store:unavailable".to_owned(),
                });
            }
            Ok(self.current.clone())
        }

        fn append_checkpoint(
            &mut self,
            expected_checkpoint_seq: u64,
            checkpoint: StateCheckpoint,
        ) -> Result<(), StateCheckpointAppendFailure> {
            if self.conflict {
                return Err(StateCheckpointAppendFailure::Conflict {
                    expected_checkpoint_seq,
                    observed_checkpoint_seq: checkpoint.checkpoint_seq.saturating_sub(1),
                    evidence_ref: "store:conflict".to_owned(),
                });
            }
            self.appended += 1;
            self.current = Some(checkpoint);
            Ok(())
        }
    }

    fn authorized_request(sequence_num: u64, event_type: &str) -> StateMachineApiRequest {
        StateMachineApiRequest {
            boundary: StateMachineApiBoundaryContext {
                request_id: format!("req:workflow-sm:{sequence_num}"),
                tenant_id: "ten_a".to_owned(),
                idempotency_key: format!("idem:workflow-sm:{sequence_num}"),
                trace_context_ref: format!("trace:workflow-sm:{sequence_num}"),
                oyatie_version: STATE_MACHINE_API_DECLARED_VERSION.to_owned(),
            },
            principal: StateMachineApiPrincipal {
                tenant_id: "ten_a".to_owned(),
                principal_id: "principal:workflow-operator:1".to_owned(),
            },
            authorization: StateMachineApiAuthorization {
                tenant_id: "ten_a".to_owned(),
                principal_id: "principal:workflow-operator:1".to_owned(),
                decision_id: "authz:workflow-sm:allow".to_owned(),
                evidence_ref: "cedar://workflow/state-machine/allow".to_owned(),
                allowed_surfaces: vec![STATE_MACHINE_API_SURFACE.to_owned()],
            },
            expected_spec_id: "workflow-spec:invoice-approval".to_owned(),
            expected_version_sha: "sha256:spec-v1".to_owned(),
            spec_integrity_ref: "spec-integrity:workflow:v1".to_owned(),
            replay_epoch_ref: "replay-epoch:api:1".to_owned(),
            body: StateMachineApiTransitionEventBody {
                event_id: format!("evt:workflow-sm:{sequence_num}"),
                run_id: "run:workflow:api:1".to_owned(),
                spec_id: "workflow-spec:invoice-approval".to_owned(),
                version_sha: "sha256:spec-v1".to_owned(),
                sequence_num,
                event_type: event_type.to_owned(),
                step_index: None,
                retry_count: None,
                retry_attempt: None,
                policy_context_ref: None,
                event_evidence_ref: format!("workflow-event:api:{sequence_num}"),
            },
        }
    }

    #[test]
    fn accepts_authorized_transition_and_returns_openapi_proto_statuses() {
        let mut api = WorkflowStateMachineApi::default();
        let mut store = FakeStore::default();

        let response = api
            .apply_transition(&mut store, authorized_request(1, "WorkflowStarted"))
            .expect("accepted");

        assert_eq!(response.status, StateMachineApiStatus::Accepted);
        assert_eq!(response.http_status_code(), 202);
        assert_eq!(response.run.run_status, "running");
        assert_eq!(response.run.proto_run_status, "RUN_STATUS_RUNNING");
        assert_eq!(response.route, STATE_MACHINE_TRANSITION_ROUTE);
        assert_eq!(store.appended, 1);
    }

    #[test]
    fn step_execution_dto_maps_openapi_and_proto_step_statuses_without_stringly_unknowns() {
        let mut api = WorkflowStateMachineApi::default();
        let mut store = FakeStore::default();
        let start = api
            .apply_transition(&mut store, authorized_request(1, "WorkflowStarted"))
            .unwrap();
        assert_eq!(start.run.current_step_index, None);

        let mut step = authorized_request(2, "StepStarted");
        step.body.step_index = Some(0);
        let response = api.apply_transition(&mut store, step).unwrap();

        assert_eq!(response.step.unwrap().step_status, "running");
        assert_eq!(response.run.proto_run_status, "RUN_STATUS_RUNNING");
        assert_eq!(response.run.current_step_index, Some(0));
    }

    #[test]
    fn unknown_event_type_denies_before_store_side_effects() {
        let mut api = WorkflowStateMachineApi::default();
        let mut store = FakeStore::default();
        let request = authorized_request(1, "WorkflowArchived");

        let error = api.apply_transition(&mut store, request).unwrap_err();

        assert_eq!(
            error.code(),
            StateMachineApiErrorCode::UnknownTransitionEvent
        );
        assert_eq!(error.status_code(), 400);
        assert_eq!(store.load_calls, 0);
        assert_eq!(store.appended, 0);
    }

    #[test]
    fn tenant_principal_authorization_drift_denies_before_store() {
        let mut api = WorkflowStateMachineApi::default();
        let mut store = FakeStore::default();
        let mut request = authorized_request(1, "WorkflowStarted");
        request.authorization.principal_id = "principal:other".to_owned();

        let error = api.apply_transition(&mut store, request).unwrap_err();

        assert_eq!(error.status_code(), 403);
        assert_eq!(
            error.code(),
            StateMachineApiErrorCode::AuthorizationPrincipalMismatch
        );
        assert_eq!(store.load_calls, 0);
    }

    #[test]
    fn idempotent_replay_returns_cached_response_and_conflict_does_not_append() {
        let mut api = WorkflowStateMachineApi::default();
        let mut store = FakeStore::default();
        let request = authorized_request(1, "WorkflowStarted");

        let first = api.apply_transition(&mut store, request.clone()).unwrap();
        let replay = api.apply_transition(&mut store, request).unwrap();
        assert_eq!(first, replay);
        assert_eq!(store.appended, 1);

        let mut conflict = authorized_request(1, "WorkflowCompleted");
        conflict.boundary.idempotency_key = "idem:workflow-sm:1".to_owned();
        let error = api.apply_transition(&mut store, conflict).unwrap_err();
        assert_eq!(error.code(), StateMachineApiErrorCode::IdempotencyKeyReused);
        assert_eq!(store.appended, 1);
    }

    #[test]
    fn store_conflict_maps_to_http_conflict() {
        let mut api = WorkflowStateMachineApi::default();
        let mut store = FakeStore {
            conflict: true,
            ..FakeStore::default()
        };

        let error = api
            .apply_transition(&mut store, authorized_request(1, "WorkflowStarted"))
            .unwrap_err();

        assert_eq!(error.status_code(), 409);
        assert_eq!(
            error.code(),
            StateMachineApiErrorCode::CheckpointAppendConflict
        );
    }

    #[test]
    fn raw_secret_or_prompt_metadata_is_rejected_without_echo() {
        let mut api = WorkflowStateMachineApi::default();
        let mut store = FakeStore::default();
        let mut request = authorized_request(1, "WorkflowStarted");
        request.boundary.trace_context_ref = "Authorization: Bearer sk-test raw prompt".to_owned();

        let error = api.apply_transition(&mut store, request).unwrap_err();

        assert_eq!(error.code(), StateMachineApiErrorCode::UnsafeMetadata);
        let rendered = format!("{error:?}").to_ascii_lowercase();
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("raw prompt"));
        assert_eq!(store.load_calls, 0);
    }

    #[test]
    fn dto_conversion_refuses_unknown_status_strings() {
        assert_eq!(
            StateMachineRunDto::run_status_from_wire("running"),
            Some(WorkflowRunStatus::Running)
        );
        assert_eq!(StateMachineRunDto::run_status_from_wire("archived"), None);
    }
}
