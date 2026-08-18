//! Workflow-engine execution-engine API boundary foundation.
//!
//! This crate owns a source-level, framework-free API boundary for execution
//! commands that will later be served by REST/gRPC adapters. It validates
//! request, tenant, principal, authorization, version, path/body, and metadata
//! bindings; maps OpenAPI-facing command bodies into typed execution-engine
//! usecase inputs; delegates only after boundary checks pass; returns stable
//! status/error DTOs; and preserves in-memory idempotent API replay semantics.
//! It performs no HTTP serving, serialization-framework work, concrete storage,
//! network I/O, durable idempotency storage, queue processing, wall-clock reads,
//! signing, or cloud runtime execution.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

pub use workflow_execution_engine_usecase::{
    ExecutionDispatchError, ExecutionDomainCommandKind, ExecutionDomainDenialKind,
    ExecutionDomainOrigin, ExecutionEngineDomainRequest, ExecutionEngineKernelError,
    ExecutionEngineUsecase, ExecutionEngineUsecaseInput, ExecutionStoreError,
    ExecutionUsecaseReceipt, ExecutionUsecaseStatus, RetryAttempt, RetryPolicyEvaluator, SlaTimer,
    SlaTimerStore, StepDispatcher, StepExecution, StepExecutionStatus, WorkflowExecutionStatus,
    WorkflowRun, WorkflowRunStore,
};

pub const EXECUTION_ENGINE_API_SURFACE: &str = "workflow-engine.execution-engine.command";
pub const EXECUTION_ENGINE_API_DECLARED_VERSION: &str = "2026-05-21";
pub const EXECUTION_ENGINE_API_CONTRACT_REF: &str =
    "workflow/workflow-engine/contracts/openapi/workflow-engine.yaml";
pub const EXECUTION_ENGINE_START_RUN_ROUTE: &str = "/runs";
pub const EXECUTION_ENGINE_DISPATCH_STEP_ROUTE: &str = "/runs/{run_id}/steps/{step_index}/dispatch";
pub const EXECUTION_ENGINE_SCHEDULE_RETRY_ROUTE: &str = "/runs/{run_id}/steps/{step_index}/retry";
pub const EXECUTION_ENGINE_ARM_TIMER_ROUTE: &str = "/runs/{run_id}/timers";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExecutionEngineApiStatus {
    Created,
    Accepted,
    BadRequest,
    Forbidden,
    Conflict,
    UnprocessableContent,
    ServiceUnavailable,
}

impl ExecutionEngineApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Created => 201,
            Self::Accepted => 202,
            Self::BadRequest => 400,
            Self::Forbidden => 403,
            Self::Conflict => 409,
            Self::UnprocessableContent => 422,
            Self::ServiceUnavailable => 503,
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::Created => "Created",
            Self::Accepted => "Accepted",
            Self::BadRequest => "Bad Request",
            Self::Forbidden => "Forbidden",
            Self::Conflict => "Conflict",
            Self::UnprocessableContent => "Unprocessable Content",
            Self::ServiceUnavailable => "Service Unavailable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExecutionEngineApiErrorCode {
    AuthorizationDenied,
    AuthorizationEvidenceInvalid,
    AuthorizationPrincipalMismatch,
    AuthorizationTenantMismatch,
    CommandShapeInvalid,
    ContractVersionUnsupported,
    DispatchDenied,
    DispatchUnavailable,
    DomainDenied,
    IdempotencyKeyReused,
    PathRunMismatch,
    PathStepMismatch,
    RequestIdEmpty,
    RetryPolicyRejected,
    RunVersionInvalid,
    StoreConflict,
    StoreUnavailable,
    TenantBindingMismatch,
    TenantHeaderEmpty,
    TimerUnavailable,
    TraceContextInvalid,
    UnsafeMetadata,
    UnknownCommand,
    UnknownRunStatus,
    UnknownStepStatus,
}

impl ExecutionEngineApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AuthorizationDenied => "WORKFLOW_EXECUTION_AUTHORIZATION_DENIED",
            Self::AuthorizationEvidenceInvalid => {
                "WORKFLOW_EXECUTION_AUTHORIZATION_EVIDENCE_INVALID"
            }
            Self::AuthorizationPrincipalMismatch => {
                "WORKFLOW_EXECUTION_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationTenantMismatch => "WORKFLOW_EXECUTION_AUTHORIZATION_TENANT_MISMATCH",
            Self::CommandShapeInvalid => "WORKFLOW_EXECUTION_COMMAND_SHAPE_INVALID",
            Self::ContractVersionUnsupported => "WORKFLOW_EXECUTION_CONTRACT_VERSION_UNSUPPORTED",
            Self::DispatchDenied => "WORKFLOW_EXECUTION_DISPATCH_DENIED",
            Self::DispatchUnavailable => "WORKFLOW_EXECUTION_DISPATCH_UNAVAILABLE",
            Self::DomainDenied => "WORKFLOW_EXECUTION_DOMAIN_DENIED",
            Self::IdempotencyKeyReused => "WORKFLOW_EXECUTION_IDEMPOTENCY_KEY_REUSED",
            Self::PathRunMismatch => "WORKFLOW_EXECUTION_PATH_RUN_MISMATCH",
            Self::PathStepMismatch => "WORKFLOW_EXECUTION_PATH_STEP_MISMATCH",
            Self::RequestIdEmpty => "WORKFLOW_EXECUTION_REQUEST_ID_EMPTY",
            Self::RetryPolicyRejected => "WORKFLOW_EXECUTION_RETRY_POLICY_REJECTED",
            Self::RunVersionInvalid => "WORKFLOW_EXECUTION_RUN_VERSION_INVALID",
            Self::StoreConflict => "WORKFLOW_EXECUTION_STORE_CONFLICT",
            Self::StoreUnavailable => "WORKFLOW_EXECUTION_STORE_UNAVAILABLE",
            Self::TenantBindingMismatch => "WORKFLOW_EXECUTION_TENANT_BINDING_MISMATCH",
            Self::TenantHeaderEmpty => "WORKFLOW_EXECUTION_TENANT_HEADER_EMPTY",
            Self::TimerUnavailable => "WORKFLOW_EXECUTION_TIMER_UNAVAILABLE",
            Self::TraceContextInvalid => "WORKFLOW_EXECUTION_TRACE_CONTEXT_INVALID",
            Self::UnsafeMetadata => "WORKFLOW_EXECUTION_UNSAFE_METADATA",
            Self::UnknownCommand => "WORKFLOW_EXECUTION_UNKNOWN_COMMAND",
            Self::UnknownRunStatus => "WORKFLOW_EXECUTION_UNKNOWN_RUN_STATUS",
            Self::UnknownStepStatus => "WORKFLOW_EXECUTION_UNKNOWN_STEP_STATUS",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineApiBoundaryContext {
    pub request_id: String,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,         // data_class: INTERNAL_ONLY
    pub idempotency_key: String,   // data_class: INTERNAL_ONLY
    pub trace_context_ref: String, // data_class: INTERNAL_ONLY
    pub oyatie_version: String,    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub evidence_ref: String,          // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineApiRequest {
    pub boundary: ExecutionEngineApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: ExecutionEngineApiPrincipal,      // data_class: INTERNAL_ONLY
    pub authorization: ExecutionEngineApiAuthorization, // data_class: INTERNAL_ONLY
    pub expected_run_version: u64,                   // data_class: INTERNAL_ONLY
    pub expected_spec_id: String,                    // data_class: INTERNAL_ONLY
    pub expected_version_sha: String,                // data_class: INTERNAL_ONLY
    pub expected_cell_id: String,                    // data_class: INTERNAL_ONLY
    pub spec_integrity_ref: String,                  // data_class: INTERNAL_ONLY
    pub replay_epoch_ref: String,                    // data_class: INTERNAL_ONLY
    pub scheduler_epoch_ref: String,                 // data_class: INTERNAL_ONLY
    pub route_run_id: Option<String>,                // data_class: INTERNAL_ONLY
    pub route_step_index: Option<u32>,               // data_class: INTERNAL_ONLY
    pub body: ExecutionEngineApiCommandBody,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineApiCommandBody {
    pub command: String,                     // data_class: PUBLIC
    pub run_id: String,                      // data_class: INTERNAL_ONLY
    pub spec_id: String,                     // data_class: INTERNAL_ONLY
    pub version_sha: String,                 // data_class: INTERNAL_ONLY
    pub active_cell_id: String,              // data_class: INTERNAL_ONLY
    pub current_run_status: String,          // data_class: PUBLIC
    pub current_run_version: u64,            // data_class: INTERNAL_ONLY
    pub current_step_index: Option<u32>,     // data_class: INTERNAL_ONLY
    pub step_id: Option<String>,             // data_class: INTERNAL_ONLY
    pub step_index: Option<u32>,             // data_class: INTERNAL_ONLY
    pub step_attempt: Option<u32>,           // data_class: INTERNAL_ONLY
    pub step_status: Option<String>,         // data_class: PUBLIC
    pub side_effect_ref: Option<String>,     // data_class: INTERNAL_ONLY
    pub last_error_ref: Option<String>,      // data_class: INTERNAL_ONLY
    pub retry_attempt: Option<u32>,          // data_class: INTERNAL_ONLY
    pub error_class_ref: Option<String>,     // data_class: INTERNAL_ONLY
    pub retry_policy_ref: Option<String>,    // data_class: INTERNAL_ONLY
    pub timer_id: Option<String>,            // data_class: INTERNAL_ONLY
    pub armed_at_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub deadline_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub input_ref: Option<String>,           // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineApiSuccessResponse {
    pub status: ExecutionEngineApiStatus,     // data_class: PUBLIC
    pub route: String,                        // data_class: PUBLIC
    pub run: ExecutionEngineRunDto,           // data_class: INTERNAL_ONLY
    pub step: Option<ExecutionEngineStepDto>, // data_class: INTERNAL_ONLY
    pub retry_delay_seconds: Option<u64>,     // data_class: INTERNAL_ONLY
    pub timer_id: Option<String>,             // data_class: INTERNAL_ONLY
    pub metadata: ExecutionEngineApiResponseMetadata, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,           // data_class: INTERNAL_ONLY
}

impl ExecutionEngineApiSuccessResponse {
    pub fn http_status_code(&self) -> u16 {
        self.status.code()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineApiResponseMetadata {
    pub request_id: String,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,         // data_class: INTERNAL_ONLY
    pub idempotency_key: String,   // data_class: INTERNAL_ONLY
    pub trace_context_ref: String, // data_class: INTERNAL_ONLY
    pub surface: String,           // data_class: INTERNAL_ONLY
    pub contract_ref: String,      // data_class: INTERNAL_ONLY
    pub oyatie_version: String,    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineRunDto {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub run_id: String,                  // data_class: INTERNAL_ONLY
    pub spec_id: String,                 // data_class: INTERNAL_ONLY
    pub version_sha: String,             // data_class: INTERNAL_ONLY
    pub active_cell_id: String,          // data_class: INTERNAL_ONLY
    pub execution_status: String,        // data_class: PUBLIC
    pub openapi_status: String,          // data_class: PUBLIC
    pub proto_execution_status: String,  // data_class: PUBLIC
    pub current_step_index: Option<u32>, // data_class: INTERNAL_ONLY
    pub run_version: u64,                // data_class: INTERNAL_ONLY
}

impl ExecutionEngineRunDto {
    pub fn from_api_and_receipt(
        request: &ExecutionEngineApiRequest,
        receipt: &ExecutionUsecaseReceipt,
    ) -> Self {
        let status = receipt.run_status.unwrap_or_else(|| {
            request
                .body
                .run_status()
                .unwrap_or(WorkflowExecutionStatus::Running)
        });
        Self {
            tenant_id: request.boundary.tenant_id.clone(),
            run_id: request.body.run_id.clone(),
            spec_id: request.body.spec_id.clone(),
            version_sha: request.body.version_sha.clone(),
            active_cell_id: request.body.active_cell_id.clone(),
            execution_status: status.as_wire().to_owned(),
            openapi_status: openapi_run_status(status).to_owned(),
            proto_execution_status: proto_run_status(status).to_owned(),
            current_step_index: request.body.current_step_index.or(request.body.step_index),
            run_version: request.body.current_run_version.saturating_add(1),
        }
    }

    pub fn execution_status_from_wire(value: &str) -> Option<WorkflowExecutionStatus> {
        WorkflowExecutionStatus::from_wire(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineStepDto {
    pub run_id: String,                  // data_class: INTERNAL_ONLY
    pub step_id: String,                 // data_class: INTERNAL_ONLY
    pub step_index: u32,                 // data_class: INTERNAL_ONLY
    pub execution_status: String,        // data_class: PUBLIC
    pub openapi_status: String,          // data_class: PUBLIC
    pub proto_step_status: String,       // data_class: PUBLIC
    pub attempt: u32,                    // data_class: INTERNAL_ONLY
    pub side_effect_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub last_error_ref: Option<String>,  // data_class: INTERNAL_ONLY
}

impl ExecutionEngineStepDto {
    pub fn from_api_and_receipt(
        request: &ExecutionEngineApiRequest,
        receipt: &ExecutionUsecaseReceipt,
    ) -> Option<Self> {
        let step_id = request.body.step_id.clone()?;
        let step_index = request.body.step_index?;
        let status = receipt
            .step_status
            .or_else(|| request.body.step_status().ok().flatten())?;
        Some(Self {
            run_id: request.body.run_id.clone(),
            step_id,
            step_index,
            execution_status: status.as_wire().to_owned(),
            openapi_status: openapi_step_status(status).to_owned(),
            proto_step_status: proto_step_status(status).to_owned(),
            attempt: request.body.step_attempt.unwrap_or(1),
            side_effect_ref: request.body.side_effect_ref.clone(),
            last_error_ref: request.body.last_error_ref.clone(),
        })
    }

    pub fn step_status_from_wire(value: &str) -> Option<StepExecutionStatus> {
        StepExecutionStatus::from_wire(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineApiProblem {
    pub type_uri: String,   // data_class: PUBLIC
    pub title: String,      // data_class: PUBLIC
    pub status: u16,        // data_class: PUBLIC
    pub code: String,       // data_class: PUBLIC
    pub detail_ref: String, // data_class: INTERNAL_ONLY
    pub instance: String,   // data_class: INTERNAL_ONLY
}

impl ExecutionEngineApiProblem {
    pub fn from_error(error: &ExecutionEngineApiError, instance: &str) -> Self {
        let status = error.status();
        Self {
            type_uri: format!(
                "https://oyatie.com/problems/workflow-engine/execution-engine/{}",
                error.code().as_str().to_ascii_lowercase().replace('_', "-")
            ),
            title: status.title().to_owned(),
            status: status.code(),
            code: error.code().as_str().to_owned(),
            detail_ref: error.primary_evidence_ref(),
            instance: if is_safe_ref(instance) {
                instance.to_owned()
            } else {
                "problem-instance:redacted".to_owned()
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionEngineApiError {
    Boundary {
        code: ExecutionEngineApiErrorCode,
        status: ExecutionEngineApiStatus,
        evidence_ref: String,
    },
    IdempotencyConflict,
    UsecaseDenied {
        status: ExecutionEngineApiStatus,
        code: ExecutionEngineApiErrorCode,
        evidence_refs: Vec<String>,
    },
}

impl ExecutionEngineApiError {
    pub fn status(&self) -> ExecutionEngineApiStatus {
        match self {
            Self::Boundary { status, .. } | Self::UsecaseDenied { status, .. } => *status,
            Self::IdempotencyConflict => ExecutionEngineApiStatus::Conflict,
        }
    }

    pub fn status_code(&self) -> u16 {
        self.status().code()
    }

    pub fn code(&self) -> ExecutionEngineApiErrorCode {
        match self {
            Self::Boundary { code, .. } | Self::UsecaseDenied { code, .. } => *code,
            Self::IdempotencyConflict => ExecutionEngineApiErrorCode::IdempotencyKeyReused,
        }
    }

    pub fn primary_evidence_ref(&self) -> String {
        match self {
            Self::Boundary { evidence_ref, .. } => evidence_ref.clone(),
            Self::IdempotencyConflict => "workflow-execution-api:idempotency-conflict".to_owned(),
            Self::UsecaseDenied { evidence_refs, .. } => evidence_refs
                .first()
                .cloned()
                .unwrap_or_else(|| "workflow-execution-api:usecase-denied".to_owned()),
        }
    }
}

#[derive(Default)]
pub struct ExecutionEngineApi {
    usecase: ExecutionEngineUsecase,
    responses_by_idempotency: BTreeMap<String, (String, ExecutionEngineApiSuccessResponse)>,
}

impl ExecutionEngineApi {
    pub fn apply_command<S, D, R, T>(
        &mut self,
        store: &mut S,
        dispatcher: &mut D,
        retry_policy: &R,
        timers: &mut T,
        request: ExecutionEngineApiRequest,
    ) -> Result<ExecutionEngineApiSuccessResponse, ExecutionEngineApiError>
    where
        S: WorkflowRunStore,
        D: StepDispatcher,
        R: RetryPolicyEvaluator,
        T: SlaTimerStore,
    {
        validate_boundary(&request)?;
        let domain_request = request.body.to_domain_request(&request)?;
        let fingerprint = request_fingerprint(&request, &domain_request);
        let cache_key = idempotency_cache_key(&request);
        if let Some((cached_fingerprint, response)) = self.responses_by_idempotency.get(&cache_key)
        {
            if cached_fingerprint == &fingerprint {
                return Ok(response.clone());
            }
            return Err(ExecutionEngineApiError::IdempotencyConflict);
        }

        let input = ExecutionEngineUsecaseInput {
            request_id: request.boundary.request_id.clone(),
            idempotency_key: request.boundary.idempotency_key.clone(),
            trace_ref: request.boundary.trace_context_ref.clone(),
            expected_run_version: request.expected_run_version,
            domain_request,
        };

        let receipt = self
            .usecase
            .apply(store, dispatcher, retry_policy, timers, input);
        let response = map_usecase_receipt(&request, receipt)?;
        self.responses_by_idempotency
            .insert(cache_key, (fingerprint, response.clone()));
        Ok(response)
    }

    pub fn cached_response_count(&self) -> usize {
        self.responses_by_idempotency.len()
    }

    pub fn cached_usecase_receipt_count(&self) -> usize {
        self.usecase.cached_receipt_count()
    }
}

impl ExecutionEngineApiCommandBody {
    fn to_domain_request(
        &self,
        request: &ExecutionEngineApiRequest,
    ) -> Result<ExecutionEngineDomainRequest, ExecutionEngineApiError> {
        let command = self.command_kind()?;
        validate_route_binding(request, self)?;
        self.validate_metadata()?;
        self.validate_command_shape(command)?;
        let mut run = WorkflowRun::new(
            &request.boundary.tenant_id,
            &self.run_id,
            &self.spec_id,
            &self.version_sha,
            &self.active_cell_id,
            self.evidence_refs_with_input(),
        )
        .map_err(kernel_error_to_api)?;
        run.status = self.run_status()?;
        run.version = self.current_run_version;
        run.current_step_index = self.current_step_index;

        let step = self.step(request).map_err(kernel_error_to_api)?;
        let retry_attempt = self.retry_attempt(request).map_err(kernel_error_to_api)?;
        let sla_timer = self.sla_timer(request).map_err(kernel_error_to_api)?;

        Ok(ExecutionEngineDomainRequest {
            run,
            step,
            retry_attempt,
            sla_timer,
            expected_tenant_id: request.boundary.tenant_id.clone(),
            expected_spec_id: request.expected_spec_id.clone(),
            expected_version_sha: request.expected_version_sha.clone(),
            expected_cell_id: request.expected_cell_id.clone(),
            policy_evidence_ref: request.authorization.evidence_ref.clone(),
            spec_integrity_ref: request.spec_integrity_ref.clone(),
            replay_epoch_ref: request.replay_epoch_ref.clone(),
            scheduler_epoch_ref: request.scheduler_epoch_ref.clone(),
            sla_reference_epoch_seconds: 0,
            command,
            origin: ExecutionDomainOrigin::ApiCommand,
        })
    }

    fn command_kind(&self) -> Result<ExecutionDomainCommandKind, ExecutionEngineApiError> {
        match self.command.as_str() {
            "StartRun" => Ok(ExecutionDomainCommandKind::StartRun),
            "DispatchStep" => Ok(ExecutionDomainCommandKind::DispatchStep),
            "ScheduleRetry" => Ok(ExecutionDomainCommandKind::ScheduleRetry),
            "ArmSlaTimer" => Ok(ExecutionDomainCommandKind::ArmSlaTimer),
            _ => Err(boundary_error(
                ExecutionEngineApiErrorCode::UnknownCommand,
                ExecutionEngineApiStatus::BadRequest,
                "workflow-execution-api:unknown-command",
            )),
        }
    }

    fn validate_metadata(&self) -> Result<(), ExecutionEngineApiError> {
        let refs = [
            Some(&self.run_id),
            Some(&self.spec_id),
            Some(&self.version_sha),
            Some(&self.active_cell_id),
            self.step_id.as_ref(),
            self.side_effect_ref.as_ref(),
            self.last_error_ref.as_ref(),
            self.error_class_ref.as_ref(),
            self.retry_policy_ref.as_ref(),
            self.timer_id.as_ref(),
            self.input_ref.as_ref(),
        ];
        if refs.into_iter().flatten().any(|value| !is_safe_ref(value))
            || !self.evidence_refs.iter().all(|value| is_safe_ref(value))
            || self.current_run_version == 0
        {
            return Err(boundary_error(
                ExecutionEngineApiErrorCode::UnsafeMetadata,
                ExecutionEngineApiStatus::BadRequest,
                "workflow-execution-api:unsafe-command-metadata",
            ));
        }
        Ok(())
    }

    fn validate_command_shape(
        &self,
        command: ExecutionDomainCommandKind,
    ) -> Result<(), ExecutionEngineApiError> {
        let step_required = matches!(
            command,
            ExecutionDomainCommandKind::StartRun
                | ExecutionDomainCommandKind::DispatchStep
                | ExecutionDomainCommandKind::ScheduleRetry
        );
        if step_required
            && (self.step_id.is_none() || self.step_index.is_none() || self.step_status.is_none())
        {
            return Err(command_shape_error());
        }
        if command == ExecutionDomainCommandKind::ScheduleRetry
            && (self.retry_attempt.is_none()
                || self.error_class_ref.is_none()
                || self.retry_policy_ref.is_none())
        {
            return Err(command_shape_error());
        }
        if command == ExecutionDomainCommandKind::ArmSlaTimer
            && (self.timer_id.is_none()
                || self.armed_at_epoch_seconds.is_none()
                || self.deadline_epoch_seconds.is_none())
        {
            return Err(command_shape_error());
        }
        Ok(())
    }

    fn run_status(&self) -> Result<WorkflowExecutionStatus, ExecutionEngineApiError> {
        WorkflowExecutionStatus::from_wire(&self.current_run_status).ok_or_else(|| {
            boundary_error(
                ExecutionEngineApiErrorCode::UnknownRunStatus,
                ExecutionEngineApiStatus::BadRequest,
                "workflow-execution-api:unknown-run-status",
            )
        })
    }

    fn step_status(&self) -> Result<Option<StepExecutionStatus>, ExecutionEngineApiError> {
        self.step_status
            .as_deref()
            .map(|value| {
                StepExecutionStatus::from_wire(value).ok_or_else(|| {
                    boundary_error(
                        ExecutionEngineApiErrorCode::UnknownStepStatus,
                        ExecutionEngineApiStatus::BadRequest,
                        "workflow-execution-api:unknown-step-status",
                    )
                })
            })
            .transpose()
    }

    fn step(
        &self,
        request: &ExecutionEngineApiRequest,
    ) -> Result<Option<StepExecution>, ExecutionEngineKernelError> {
        let Some(step_id) = self.step_id.as_deref() else {
            return Ok(None);
        };
        let mut step = StepExecution::new(
            &request.boundary.tenant_id,
            &self.run_id,
            step_id,
            self.step_index.unwrap_or(0),
            self.step_attempt.unwrap_or(1),
            &request.boundary.idempotency_key,
            self.evidence_refs.clone(),
        )?;
        if let Some(status) = self
            .step_status()
            .map_err(|_| ExecutionEngineKernelError::UnsafeMetadata)?
        {
            step.status = status;
        }
        step.side_effect_ref = self.side_effect_ref.clone();
        step.last_error_ref = self.last_error_ref.clone();
        Ok(Some(step))
    }

    fn retry_attempt(
        &self,
        request: &ExecutionEngineApiRequest,
    ) -> Result<Option<RetryAttempt>, ExecutionEngineKernelError> {
        let Some(error_class_ref) = self.error_class_ref.as_deref() else {
            return Ok(None);
        };
        let Some(retry_policy_ref) = self.retry_policy_ref.as_deref() else {
            return Ok(None);
        };
        let Some(step_id) = self.step_id.as_deref() else {
            return Ok(None);
        };
        RetryAttempt::new(
            &request.boundary.tenant_id,
            &self.run_id,
            step_id,
            self.retry_attempt
                .unwrap_or_else(|| self.step_attempt.unwrap_or(1) + 1),
            error_class_ref,
            retry_policy_ref,
            self.evidence_refs.clone(),
        )
        .map(Some)
    }

    fn sla_timer(
        &self,
        request: &ExecutionEngineApiRequest,
    ) -> Result<Option<SlaTimer>, ExecutionEngineKernelError> {
        let Some(timer_id) = self.timer_id.as_deref() else {
            return Ok(None);
        };
        SlaTimer::new(
            timer_id,
            &request.boundary.tenant_id,
            &self.run_id,
            self.step_index,
            self.armed_at_epoch_seconds.unwrap_or(0),
            self.deadline_epoch_seconds.unwrap_or(0),
            self.evidence_refs.clone(),
        )
        .map(Some)
    }

    fn evidence_refs_with_input(&self) -> Vec<String> {
        let mut refs = self.evidence_refs.clone();
        if let Some(input_ref) = &self.input_ref {
            refs.push(input_ref.clone());
        }
        sorted_unique(refs)
    }
}

fn map_usecase_receipt(
    request: &ExecutionEngineApiRequest,
    receipt: ExecutionUsecaseReceipt,
) -> Result<ExecutionEngineApiSuccessResponse, ExecutionEngineApiError> {
    match receipt.status {
        ExecutionUsecaseStatus::Applied => Ok(ExecutionEngineApiSuccessResponse {
            status: success_status(receipt.command),
            route: route_for_command(receipt.command).to_owned(),
            run: ExecutionEngineRunDto::from_api_and_receipt(request, &receipt),
            step: ExecutionEngineStepDto::from_api_and_receipt(request, &receipt),
            retry_delay_seconds: receipt.retry_delay_seconds,
            timer_id: request.body.timer_id.clone(),
            metadata: response_metadata(request),
            evidence_refs: sorted_unique(receipt.evidence_refs),
        }),
        ExecutionUsecaseStatus::DomainDenied => Err(usecase_error(
            ExecutionEngineApiStatus::Forbidden,
            ExecutionEngineApiErrorCode::DomainDenied,
            receipt.evidence_refs,
        )),
        ExecutionUsecaseStatus::InvalidInput => Err(usecase_error(
            ExecutionEngineApiStatus::BadRequest,
            ExecutionEngineApiErrorCode::UnsafeMetadata,
            receipt.evidence_refs,
        )),
        ExecutionUsecaseStatus::IdempotencyConflict => {
            Err(ExecutionEngineApiError::IdempotencyConflict)
        }
        ExecutionUsecaseStatus::StoreConflict => Err(usecase_error(
            ExecutionEngineApiStatus::Conflict,
            ExecutionEngineApiErrorCode::StoreConflict,
            receipt.evidence_refs,
        )),
        ExecutionUsecaseStatus::StoreUnavailable => Err(usecase_error(
            ExecutionEngineApiStatus::ServiceUnavailable,
            ExecutionEngineApiErrorCode::StoreUnavailable,
            receipt.evidence_refs,
        )),
        ExecutionUsecaseStatus::DispatchDenied => Err(usecase_error(
            ExecutionEngineApiStatus::Forbidden,
            ExecutionEngineApiErrorCode::DispatchDenied,
            receipt.evidence_refs,
        )),
        ExecutionUsecaseStatus::DispatchUnavailable => Err(usecase_error(
            ExecutionEngineApiStatus::ServiceUnavailable,
            ExecutionEngineApiErrorCode::DispatchUnavailable,
            receipt.evidence_refs,
        )),
        ExecutionUsecaseStatus::RetryPolicyRejected => Err(usecase_error(
            ExecutionEngineApiStatus::UnprocessableContent,
            ExecutionEngineApiErrorCode::RetryPolicyRejected,
            receipt.evidence_refs,
        )),
        ExecutionUsecaseStatus::TimerUnavailable => Err(usecase_error(
            ExecutionEngineApiStatus::ServiceUnavailable,
            ExecutionEngineApiErrorCode::TimerUnavailable,
            receipt.evidence_refs,
        )),
    }
}

fn validate_boundary(request: &ExecutionEngineApiRequest) -> Result<(), ExecutionEngineApiError> {
    if request.boundary.request_id.trim().is_empty() {
        return Err(boundary_error(
            ExecutionEngineApiErrorCode::RequestIdEmpty,
            ExecutionEngineApiStatus::BadRequest,
            "workflow-execution-api:request-id-required",
        ));
    }
    if request.boundary.tenant_id.trim().is_empty() {
        return Err(boundary_error(
            ExecutionEngineApiErrorCode::TenantHeaderEmpty,
            ExecutionEngineApiStatus::BadRequest,
            "workflow-execution-api:tenant-required",
        ));
    }
    if request.boundary.oyatie_version != EXECUTION_ENGINE_API_DECLARED_VERSION {
        return Err(boundary_error(
            ExecutionEngineApiErrorCode::ContractVersionUnsupported,
            ExecutionEngineApiStatus::BadRequest,
            "workflow-execution-api:unsupported-version",
        ));
    }
    if request.expected_run_version == 0 {
        return Err(boundary_error(
            ExecutionEngineApiErrorCode::RunVersionInvalid,
            ExecutionEngineApiStatus::BadRequest,
            "workflow-execution-api:run-version-required",
        ));
    }
    if !is_safe_ref(&request.boundary.request_id) {
        return Err(boundary_error(
            ExecutionEngineApiErrorCode::UnsafeMetadata,
            ExecutionEngineApiStatus::BadRequest,
            "workflow-execution-api:request-id-invalid",
        ));
    }
    if !is_safe_ref(&request.boundary.idempotency_key) {
        return Err(boundary_error(
            ExecutionEngineApiErrorCode::UnsafeMetadata,
            ExecutionEngineApiStatus::BadRequest,
            "workflow-execution-api:idempotency-key-invalid",
        ));
    }
    if !is_safe_ref(&request.boundary.trace_context_ref) {
        return Err(boundary_error(
            ExecutionEngineApiErrorCode::TraceContextInvalid,
            ExecutionEngineApiStatus::BadRequest,
            "workflow-execution-api:trace-context-invalid",
        ));
    }
    if !is_safe_ref(&request.expected_spec_id)
        || !is_safe_ref(&request.expected_version_sha)
        || !is_safe_ref(&request.expected_cell_id)
        || !is_safe_ref(&request.spec_integrity_ref)
        || !is_safe_ref(&request.replay_epoch_ref)
        || !is_safe_ref(&request.scheduler_epoch_ref)
    {
        return Err(boundary_error(
            ExecutionEngineApiErrorCode::UnsafeMetadata,
            ExecutionEngineApiStatus::BadRequest,
            "workflow-execution-api:unsafe-contract-metadata",
        ));
    }
    if request.principal.tenant_id != request.boundary.tenant_id {
        return Err(boundary_error(
            ExecutionEngineApiErrorCode::TenantBindingMismatch,
            ExecutionEngineApiStatus::Forbidden,
            "workflow-execution-api:principal-tenant-mismatch",
        ));
    }
    if request.authorization.tenant_id != request.boundary.tenant_id {
        return Err(boundary_error(
            ExecutionEngineApiErrorCode::AuthorizationTenantMismatch,
            ExecutionEngineApiStatus::Forbidden,
            "workflow-execution-api:auth-tenant-mismatch",
        ));
    }
    if request.authorization.principal_id != request.principal.principal_id {
        return Err(boundary_error(
            ExecutionEngineApiErrorCode::AuthorizationPrincipalMismatch,
            ExecutionEngineApiStatus::Forbidden,
            "workflow-execution-api:auth-principal-mismatch",
        ));
    }
    if !is_safe_ref(&request.authorization.decision_id)
        || !is_safe_ref(&request.authorization.evidence_ref)
    {
        return Err(boundary_error(
            ExecutionEngineApiErrorCode::AuthorizationEvidenceInvalid,
            ExecutionEngineApiStatus::Forbidden,
            "workflow-execution-api:auth-evidence-invalid",
        ));
    }
    if !request
        .authorization
        .allowed_surfaces
        .iter()
        .any(|surface| surface == EXECUTION_ENGINE_API_SURFACE)
    {
        return Err(boundary_error(
            ExecutionEngineApiErrorCode::AuthorizationDenied,
            ExecutionEngineApiStatus::Forbidden,
            "workflow-execution-api:surface-denied",
        ));
    }
    Ok(())
}

fn validate_route_binding(
    request: &ExecutionEngineApiRequest,
    body: &ExecutionEngineApiCommandBody,
) -> Result<(), ExecutionEngineApiError> {
    if let Some(route_run_id) = &request.route_run_id
        && route_run_id != &body.run_id
    {
        return Err(boundary_error(
            ExecutionEngineApiErrorCode::PathRunMismatch,
            ExecutionEngineApiStatus::BadRequest,
            "workflow-execution-api:path-run-mismatch",
        ));
    }
    if let Some(route_step_index) = request.route_step_index
        && Some(route_step_index) != body.step_index
    {
        return Err(boundary_error(
            ExecutionEngineApiErrorCode::PathStepMismatch,
            ExecutionEngineApiStatus::BadRequest,
            "workflow-execution-api:path-step-mismatch",
        ));
    }
    Ok(())
}

fn response_metadata(request: &ExecutionEngineApiRequest) -> ExecutionEngineApiResponseMetadata {
    ExecutionEngineApiResponseMetadata {
        request_id: request.boundary.request_id.clone(),
        tenant_id: request.boundary.tenant_id.clone(),
        idempotency_key: request.boundary.idempotency_key.clone(),
        trace_context_ref: request.boundary.trace_context_ref.clone(),
        surface: EXECUTION_ENGINE_API_SURFACE.to_owned(),
        contract_ref: EXECUTION_ENGINE_API_CONTRACT_REF.to_owned(),
        oyatie_version: EXECUTION_ENGINE_API_DECLARED_VERSION.to_owned(),
    }
}

fn request_fingerprint(
    request: &ExecutionEngineApiRequest,
    domain_request: &ExecutionEngineDomainRequest,
) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{:?}|{:?}|{:?}",
        request.boundary.tenant_id,
        request.principal.principal_id,
        request.authorization.decision_id,
        request.expected_run_version,
        request.expected_spec_id,
        request.expected_version_sha,
        request.expected_cell_id,
        request.spec_integrity_ref,
        request.replay_epoch_ref,
        request.scheduler_epoch_ref,
        domain_request.command.as_wire(),
        domain_request.run,
        domain_request.step,
        domain_request.retry_attempt
    )
}

fn idempotency_cache_key(request: &ExecutionEngineApiRequest) -> String {
    format!(
        "{}|{}",
        request.boundary.tenant_id, request.boundary.idempotency_key
    )
}

fn success_status(command: ExecutionDomainCommandKind) -> ExecutionEngineApiStatus {
    match command {
        ExecutionDomainCommandKind::StartRun => ExecutionEngineApiStatus::Created,
        ExecutionDomainCommandKind::DispatchStep
        | ExecutionDomainCommandKind::ScheduleRetry
        | ExecutionDomainCommandKind::ArmSlaTimer => ExecutionEngineApiStatus::Accepted,
    }
}

fn route_for_command(command: ExecutionDomainCommandKind) -> &'static str {
    match command {
        ExecutionDomainCommandKind::StartRun => EXECUTION_ENGINE_START_RUN_ROUTE,
        ExecutionDomainCommandKind::DispatchStep => EXECUTION_ENGINE_DISPATCH_STEP_ROUTE,
        ExecutionDomainCommandKind::ScheduleRetry => EXECUTION_ENGINE_SCHEDULE_RETRY_ROUTE,
        ExecutionDomainCommandKind::ArmSlaTimer => EXECUTION_ENGINE_ARM_TIMER_ROUTE,
    }
}

fn usecase_error(
    status: ExecutionEngineApiStatus,
    code: ExecutionEngineApiErrorCode,
    evidence_refs: Vec<String>,
) -> ExecutionEngineApiError {
    ExecutionEngineApiError::UsecaseDenied {
        status,
        code,
        evidence_refs: sorted_unique(evidence_refs),
    }
}

fn boundary_error(
    code: ExecutionEngineApiErrorCode,
    status: ExecutionEngineApiStatus,
    evidence_ref: &str,
) -> ExecutionEngineApiError {
    ExecutionEngineApiError::Boundary {
        code,
        status,
        evidence_ref: evidence_ref.to_owned(),
    }
}

fn command_shape_error() -> ExecutionEngineApiError {
    boundary_error(
        ExecutionEngineApiErrorCode::CommandShapeInvalid,
        ExecutionEngineApiStatus::BadRequest,
        "workflow-execution-api:command-shape-invalid",
    )
}

fn kernel_error_to_api(error: ExecutionEngineKernelError) -> ExecutionEngineApiError {
    let evidence_ref = match error {
        ExecutionEngineKernelError::InvalidAttempt => "workflow-execution-api:invalid-attempt",
        ExecutionEngineKernelError::InvalidObservationWindow => {
            "workflow-execution-api:invalid-observation-window"
        }
        ExecutionEngineKernelError::InvalidStepIndex => "workflow-execution-api:invalid-step-index",
        ExecutionEngineKernelError::InvalidTimerDeadline => {
            "workflow-execution-api:invalid-timer-deadline"
        }
        ExecutionEngineKernelError::UnsafeMetadata => "workflow-execution-api:unsafe-metadata",
    };
    boundary_error(
        ExecutionEngineApiErrorCode::CommandShapeInvalid,
        ExecutionEngineApiStatus::BadRequest,
        evidence_ref,
    )
}

fn openapi_run_status(status: WorkflowExecutionStatus) -> &'static str {
    match status {
        WorkflowExecutionStatus::Pending => "waiting",
        WorkflowExecutionStatus::Running => "running",
        WorkflowExecutionStatus::Paused => "paused",
        WorkflowExecutionStatus::Completed => "completed",
        WorkflowExecutionStatus::Failed => "failed",
        WorkflowExecutionStatus::Cancelled => "cancelled",
    }
}

fn proto_run_status(status: WorkflowExecutionStatus) -> &'static str {
    match status {
        WorkflowExecutionStatus::Pending => "EXECUTION_RUN_STATUS_PENDING",
        WorkflowExecutionStatus::Running => "EXECUTION_RUN_STATUS_RUNNING",
        WorkflowExecutionStatus::Paused => "EXECUTION_RUN_STATUS_PAUSED",
        WorkflowExecutionStatus::Completed => "EXECUTION_RUN_STATUS_COMPLETED",
        WorkflowExecutionStatus::Failed => "EXECUTION_RUN_STATUS_FAILED",
        WorkflowExecutionStatus::Cancelled => "EXECUTION_RUN_STATUS_CANCELLED",
    }
}

fn openapi_step_status(status: StepExecutionStatus) -> &'static str {
    match status {
        StepExecutionStatus::Pending => "pending",
        StepExecutionStatus::Leased | StepExecutionStatus::Running => "running",
        StepExecutionStatus::Succeeded => "completed",
        StepExecutionStatus::Failed
        | StepExecutionStatus::TimedOut
        | StepExecutionStatus::Cancelled => "failed",
        StepExecutionStatus::Retrying => "retrying",
    }
}

fn proto_step_status(status: StepExecutionStatus) -> &'static str {
    match status {
        StepExecutionStatus::Pending => "EXECUTION_STEP_STATUS_PENDING",
        StepExecutionStatus::Leased => "EXECUTION_STEP_STATUS_LEASED",
        StepExecutionStatus::Running => "EXECUTION_STEP_STATUS_RUNNING",
        StepExecutionStatus::Succeeded => "EXECUTION_STEP_STATUS_SUCCEEDED",
        StepExecutionStatus::Failed => "EXECUTION_STEP_STATUS_FAILED",
        StepExecutionStatus::Retrying => "EXECUTION_STEP_STATUS_RETRYING",
        StepExecutionStatus::TimedOut => "EXECUTION_STEP_STATUS_TIMED_OUT",
        StepExecutionStatus::Cancelled => "EXECUTION_STEP_STATUS_CANCELLED",
    }
}

fn is_safe_tenant(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.starts_with("ten_") && value == trimmed && is_safe_metadata(value)
}

fn is_safe_ref(value: &str) -> bool {
    is_safe_metadata(value) && value.contains(':')
}

fn is_safe_metadata(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && value == trimmed
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
        || lower.contains("payload")
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
        current: Option<WorkflowRun>,
        created: usize,
        loaded: usize,
        status_updates: usize,
        saved_steps: usize,
        conflict_on_update: bool,
        unavailable_on_load: bool,
    }

    impl WorkflowRunStore for FakeStore {
        fn create_run(&mut self, run: WorkflowRun) -> Result<(), ExecutionStoreError> {
            self.created += 1;
            self.current = Some(run);
            Ok(())
        }

        fn load_run(
            &self,
            _tenant_id: &str,
            _run_id: &str,
        ) -> Result<Option<WorkflowRun>, ExecutionStoreError> {
            if self.unavailable_on_load {
                return Err(ExecutionStoreError::Unavailable {
                    evidence_ref: "store:load-unavailable".to_owned(),
                });
            }
            Ok(self.current.clone())
        }

        fn update_run_status(
            &mut self,
            _tenant_id: &str,
            _run_id: &str,
            expected_version: u64,
            status: WorkflowExecutionStatus,
            _evidence_ref: &str,
        ) -> Result<(), ExecutionStoreError> {
            self.loaded += 1;
            if self.conflict_on_update {
                return Err(ExecutionStoreError::Conflict {
                    expected_version,
                    observed_version: expected_version + 1,
                    evidence_ref: "store:update-conflict".to_owned(),
                });
            }
            self.status_updates += 1;
            if let Some(run) = &mut self.current {
                run.status = status;
                run.version = expected_version + 1;
            }
            Ok(())
        }

        fn save_step(&mut self, _step: StepExecution) -> Result<(), ExecutionStoreError> {
            self.saved_steps += 1;
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeDispatcher {
        dispatched: usize,
        denied: bool,
        unavailable: bool,
    }

    impl StepDispatcher for FakeDispatcher {
        fn dispatch_step(
            &mut self,
            _tenant_id: &str,
            _run_id: &str,
            _step_index: u32,
            _evidence_ref: &str,
        ) -> Result<(), ExecutionDispatchError> {
            if self.denied {
                return Err(ExecutionDispatchError::Denied {
                    evidence_ref: "dispatcher:denied".to_owned(),
                });
            }
            if self.unavailable {
                return Err(ExecutionDispatchError::Unavailable {
                    evidence_ref: "dispatcher:unavailable".to_owned(),
                });
            }
            self.dispatched += 1;
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeRetryPolicy {
        reject: bool,
    }

    impl RetryPolicyEvaluator for FakeRetryPolicy {
        fn next_delay_seconds(
            &self,
            _attempt: &RetryAttempt,
        ) -> Result<Option<u64>, ExecutionEngineKernelError> {
            if self.reject {
                return Err(ExecutionEngineKernelError::UnsafeMetadata);
            }
            Ok(Some(30))
        }
    }

    #[derive(Default)]
    struct FakeTimers {
        armed: usize,
    }

    impl SlaTimerStore for FakeTimers {
        fn arm_timer(&mut self, _timer: SlaTimer) -> Result<(), ExecutionStoreError> {
            self.armed += 1;
            Ok(())
        }

        fn cancel_timer(
            &mut self,
            _tenant_id: &str,
            _timer_id: &str,
        ) -> Result<(), ExecutionStoreError> {
            Ok(())
        }

        fn fire_expired(
            &mut self,
            _tenant_id: &str,
            _now_epoch_seconds: u64,
        ) -> Result<Vec<SlaTimer>, ExecutionStoreError> {
            Ok(Vec::new())
        }
    }

    fn start_request(seq: u64) -> ExecutionEngineApiRequest {
        ExecutionEngineApiRequest {
            boundary: ExecutionEngineApiBoundaryContext {
                request_id: format!("req:workflow-execution:{seq}"),
                tenant_id: "ten_a".to_owned(),
                idempotency_key: format!("idem:workflow-execution:{seq}"),
                trace_context_ref: format!("trace:workflow-execution:{seq}"),
                oyatie_version: EXECUTION_ENGINE_API_DECLARED_VERSION.to_owned(),
            },
            principal: ExecutionEngineApiPrincipal {
                tenant_id: "ten_a".to_owned(),
                principal_id: "principal:workflow-operator:1".to_owned(),
            },
            authorization: ExecutionEngineApiAuthorization {
                tenant_id: "ten_a".to_owned(),
                principal_id: "principal:workflow-operator:1".to_owned(),
                decision_id: "authz:workflow-execution:allow".to_owned(),
                evidence_ref: "cedar://workflow/execution/allow".to_owned(),
                allowed_surfaces: vec![EXECUTION_ENGINE_API_SURFACE.to_owned()],
            },
            expected_run_version: 1,
            expected_spec_id: "workflow-spec:invoice-approval".to_owned(),
            expected_version_sha: "sha256:spec-v1".to_owned(),
            expected_cell_id: "cell:use1:a".to_owned(),
            spec_integrity_ref: "spec-integrity:workflow:v1".to_owned(),
            replay_epoch_ref: "replay-epoch:api:1".to_owned(),
            scheduler_epoch_ref: "scheduler-epoch:api:1".to_owned(),
            route_run_id: None,
            route_step_index: None,
            body: ExecutionEngineApiCommandBody {
                command: "StartRun".to_owned(),
                run_id: "run:workflow:api:1".to_owned(),
                spec_id: "workflow-spec:invoice-approval".to_owned(),
                version_sha: "sha256:spec-v1".to_owned(),
                active_cell_id: "cell:use1:a".to_owned(),
                current_run_status: "pending".to_owned(),
                current_run_version: 1,
                current_step_index: Some(0),
                step_id: Some("step:approve".to_owned()),
                step_index: Some(0),
                step_attempt: Some(1),
                step_status: Some("pending".to_owned()),
                side_effect_ref: None,
                last_error_ref: None,
                retry_attempt: None,
                error_class_ref: None,
                retry_policy_ref: None,
                timer_id: None,
                armed_at_epoch_seconds: None,
                deadline_epoch_seconds: None,
                input_ref: Some("input-ref:initial-form".to_owned()),
                evidence_refs: vec!["workflow-execution:api-request".to_owned()],
            },
        }
    }

    fn run_current() -> WorkflowRun {
        let mut run = WorkflowRun::new(
            "ten_a",
            "run:workflow:api:1",
            "workflow-spec:invoice-approval",
            "sha256:spec-v1",
            "cell:use1:a",
            vec!["workflow-execution:existing-run".to_owned()],
        )
        .unwrap();
        run.status = WorkflowExecutionStatus::Running;
        run.version = 1;
        run.current_step_index = Some(0);
        run
    }

    fn dispatch_request(seq: u64) -> ExecutionEngineApiRequest {
        let mut request = start_request(seq);
        request.route_run_id = Some("run:workflow:api:1".to_owned());
        request.route_step_index = Some(0);
        request.body.command = "DispatchStep".to_owned();
        request.body.current_run_status = "running".to_owned();
        request.body.step_status = Some("pending".to_owned());
        request
    }

    fn retry_request(seq: u64) -> ExecutionEngineApiRequest {
        let mut request = dispatch_request(seq);
        request.body.command = "ScheduleRetry".to_owned();
        request.body.step_status = Some("failed".to_owned());
        request.body.step_attempt = Some(1);
        request.body.retry_attempt = Some(2);
        request.body.error_class_ref = Some("error-class:http-503".to_owned());
        request.body.retry_policy_ref = Some("retry-policy:standard".to_owned());
        request
    }

    fn timer_request(seq: u64) -> ExecutionEngineApiRequest {
        let mut request = dispatch_request(seq);
        request.route_step_index = None;
        request.body.command = "ArmSlaTimer".to_owned();
        request.body.step_id = None;
        request.body.step_index = None;
        request.body.step_status = None;
        request.body.timer_id = Some("timer:workflow:approval:1".to_owned());
        request.body.armed_at_epoch_seconds = Some(100);
        request.body.deadline_epoch_seconds = Some(200);
        request
    }

    fn harness() -> (
        ExecutionEngineApi,
        FakeStore,
        FakeDispatcher,
        FakeRetryPolicy,
        FakeTimers,
    ) {
        (
            ExecutionEngineApi::default(),
            FakeStore::default(),
            FakeDispatcher::default(),
            FakeRetryPolicy::default(),
            FakeTimers::default(),
        )
    }

    #[test]
    fn route_constants_follow_openapi_execution_operations_and_stable_status_codes() {
        assert_eq!(EXECUTION_ENGINE_START_RUN_ROUTE, "/runs");
        assert_eq!(
            EXECUTION_ENGINE_DISPATCH_STEP_ROUTE,
            "/runs/{run_id}/steps/{step_index}/dispatch"
        );
        assert_eq!(ExecutionEngineApiStatus::Created.code(), 201);
        assert_eq!(ExecutionEngineApiStatus::Accepted.code(), 202);
        assert_eq!(ExecutionEngineApiStatus::Conflict.code(), 409);
        assert_eq!(ExecutionEngineApiStatus::UnprocessableContent.code(), 422);
    }

    #[test]
    fn accepts_authorized_start_and_returns_openapi_run_dto() {
        let (mut api, mut store, mut dispatcher, retry_policy, mut timers) = harness();

        let response = api
            .apply_command(
                &mut store,
                &mut dispatcher,
                &retry_policy,
                &mut timers,
                start_request(1),
            )
            .expect("created");

        assert_eq!(response.status, ExecutionEngineApiStatus::Created);
        assert_eq!(response.http_status_code(), 201);
        assert_eq!(response.route, EXECUTION_ENGINE_START_RUN_ROUTE);
        assert_eq!(response.run.execution_status, "running");
        assert_eq!(response.run.openapi_status, "running");
        assert_eq!(
            response.run.proto_execution_status,
            "EXECUTION_RUN_STATUS_RUNNING"
        );
        assert_eq!(response.step.unwrap().openapi_status, "pending");
        assert_eq!(store.created, 1);
        assert_eq!(store.saved_steps, 1);
        assert_eq!(store.status_updates, 1);
        assert_eq!(dispatcher.dispatched, 1);
    }

    #[test]
    fn dispatch_retry_and_timer_commands_delegate_after_boundary_validation_only() {
        let (mut api, mut store, mut dispatcher, retry_policy, mut timers) = harness();
        store.current = Some(run_current());

        let dispatch = api
            .apply_command(
                &mut store,
                &mut dispatcher,
                &retry_policy,
                &mut timers,
                dispatch_request(2),
            )
            .unwrap();
        assert_eq!(dispatch.status, ExecutionEngineApiStatus::Accepted);
        assert_eq!(dispatch.step.unwrap().execution_status, "leased");
        assert_eq!(dispatcher.dispatched, 1);

        store.current = Some(run_current());
        let retry = api
            .apply_command(
                &mut store,
                &mut dispatcher,
                &retry_policy,
                &mut timers,
                retry_request(3),
            )
            .unwrap();
        assert_eq!(retry.retry_delay_seconds, Some(30));
        assert_eq!(retry.step.unwrap().openapi_status, "retrying");

        store.current = Some(run_current());
        let timer = api
            .apply_command(
                &mut store,
                &mut dispatcher,
                &retry_policy,
                &mut timers,
                timer_request(4),
            )
            .unwrap();
        assert_eq!(timer.timer_id.as_deref(), Some("timer:workflow:approval:1"));
        assert_eq!(timers.armed, 1);
    }

    #[test]
    fn tenant_principal_authorization_and_path_drift_deny_before_usecase_side_effects() {
        let (mut api, mut store, mut dispatcher, retry_policy, mut timers) = harness();
        let mut request = dispatch_request(2);
        request.authorization.principal_id = "principal:other".to_owned();

        let error = api
            .apply_command(
                &mut store,
                &mut dispatcher,
                &retry_policy,
                &mut timers,
                request,
            )
            .unwrap_err();

        assert_eq!(error.status_code(), 403);
        assert_eq!(
            error.code(),
            ExecutionEngineApiErrorCode::AuthorizationPrincipalMismatch
        );
        assert_eq!(store.created + store.saved_steps + store.status_updates, 0);
        assert_eq!(dispatcher.dispatched, 0);

        let mut path_drift = dispatch_request(3);
        path_drift.route_run_id = Some("run:other".to_owned());
        let error = api
            .apply_command(
                &mut store,
                &mut dispatcher,
                &retry_policy,
                &mut timers,
                path_drift,
            )
            .unwrap_err();
        assert_eq!(error.code(), ExecutionEngineApiErrorCode::PathRunMismatch);
        assert_eq!(store.created + store.saved_steps + store.status_updates, 0);
    }

    #[test]
    fn raw_secret_prompt_output_or_payload_metadata_is_rejected_without_echo() {
        let (mut api, mut store, mut dispatcher, retry_policy, mut timers) = harness();
        let mut request = start_request(1);
        request.boundary.trace_context_ref = "Authorization: Bearer sk-test raw prompt".to_owned();

        let error = api
            .apply_command(
                &mut store,
                &mut dispatcher,
                &retry_policy,
                &mut timers,
                request,
            )
            .unwrap_err();

        assert_eq!(
            error.code(),
            ExecutionEngineApiErrorCode::TraceContextInvalid
        );
        let rendered = format!("{error:?}").to_ascii_lowercase();
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("raw prompt"));
        assert_eq!(store.created + store.saved_steps + store.status_updates, 0);

        let mut raw_body = dispatch_request(2);
        raw_body.body.side_effect_ref = Some("side-effect:raw-output-payload".to_owned());
        let error = api
            .apply_command(
                &mut store,
                &mut dispatcher,
                &retry_policy,
                &mut timers,
                raw_body,
            )
            .unwrap_err();
        assert_eq!(error.code(), ExecutionEngineApiErrorCode::UnsafeMetadata);
        assert_eq!(store.created + store.saved_steps + store.status_updates, 0);

        let mut raw_input = start_request(3);
        raw_input.body.input_ref = Some("payload:raw-output".to_owned());
        let error = api
            .apply_command(
                &mut store,
                &mut dispatcher,
                &retry_policy,
                &mut timers,
                raw_input,
            )
            .unwrap_err();
        assert_eq!(error.code(), ExecutionEngineApiErrorCode::UnsafeMetadata);
        let problem = ExecutionEngineApiProblem::from_error(&error, "instance:execution-api:3");
        assert_eq!(problem.status, 400);
        assert_eq!(problem.code, "WORKFLOW_EXECUTION_UNSAFE_METADATA");
    }

    #[test]
    fn idempotent_replay_returns_cached_response_and_conflict_skips_ports() {
        let (mut api, mut store, mut dispatcher, retry_policy, mut timers) = harness();
        let request = start_request(1);

        let first = api
            .apply_command(
                &mut store,
                &mut dispatcher,
                &retry_policy,
                &mut timers,
                request.clone(),
            )
            .unwrap();
        let replay = api
            .apply_command(
                &mut store,
                &mut dispatcher,
                &retry_policy,
                &mut timers,
                request,
            )
            .unwrap();
        assert_eq!(first, replay);
        assert_eq!(api.cached_response_count(), 1);
        assert_eq!(api.cached_usecase_receipt_count(), 1);
        assert_eq!(store.created, 1);

        let mut conflict = start_request(1);
        conflict.body.step_id = Some("step:different".to_owned());
        let error = api
            .apply_command(
                &mut store,
                &mut dispatcher,
                &retry_policy,
                &mut timers,
                conflict,
            )
            .unwrap_err();
        assert_eq!(
            error.code(),
            ExecutionEngineApiErrorCode::IdempotencyKeyReused
        );
        assert_eq!(store.created, 1);
    }

    #[test]
    fn usecase_failures_map_to_stable_http_status_and_problem_details() {
        let (mut api, mut store, mut dispatcher, retry_policy, mut timers) = harness();
        store.current = Some(run_current());
        store.conflict_on_update = true;

        let conflict = api
            .apply_command(
                &mut store,
                &mut dispatcher,
                &retry_policy,
                &mut timers,
                dispatch_request(2),
            )
            .unwrap_err();
        assert_eq!(conflict.status_code(), 409);
        assert_eq!(conflict.code(), ExecutionEngineApiErrorCode::StoreConflict);

        let (mut api, mut store, mut dispatcher, retry_policy, mut timers) = harness();
        store.current = Some(run_current());
        dispatcher.denied = true;
        let denied = api
            .apply_command(
                &mut store,
                &mut dispatcher,
                &retry_policy,
                &mut timers,
                dispatch_request(3),
            )
            .unwrap_err();
        assert_eq!(denied.status_code(), 403);
        assert_eq!(denied.code(), ExecutionEngineApiErrorCode::DispatchDenied);

        let problem = ExecutionEngineApiProblem::from_error(&denied, "instance:execution-api:3");
        assert_eq!(problem.status, 403);
        assert_eq!(problem.title, "Forbidden");
        assert!(
            problem
                .type_uri
                .contains("workflow-execution-dispatch-denied")
        );
    }

    #[test]
    fn dto_conversion_refuses_unknown_status_strings() {
        assert_eq!(
            ExecutionEngineRunDto::execution_status_from_wire("running"),
            Some(WorkflowExecutionStatus::Running)
        );
        assert_eq!(
            ExecutionEngineRunDto::execution_status_from_wire("waiting"),
            None
        );
        assert_eq!(
            ExecutionEngineStepDto::step_status_from_wire("retrying"),
            Some(StepExecutionStatus::Retrying)
        );
        assert_eq!(
            ExecutionEngineStepDto::step_status_from_wire("queued"),
            None
        );
    }
}
