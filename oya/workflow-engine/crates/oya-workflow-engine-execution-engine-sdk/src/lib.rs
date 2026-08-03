//! Workflow-engine execution-engine SDK foundation.
//!
//! This crate provides a source-level, language-native SDK facade over the
//! execution-engine REST/API boundary for future generated SDK work. It builds
//! typed command requests for start, dispatch, retry, and timer routes; binds
//! version, tenant, principal, authorization, trace, idempotency, and route
//! metadata; exposes an in-process preview execution seam for local tests; and
//! rejects raw prompt/output/payload/secret-shaped material before delegating to
//! REST/API/usecase ports. It performs no HTTP client work, DNS, sockets,
//! serialization-framework work, credential loading, random/UUID generation,
//! wall-clock reads, automatic retries, durable idempotency, queueing, signing,
//! filesystem access, or cloud-runtime side effects.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use oya_workflow_engine_execution_engine_rest::{
    EXECUTION_ENGINE_API_CONTRACT_REF, EXECUTION_ENGINE_API_DECLARED_VERSION,
    EXECUTION_ENGINE_API_SURFACE, EXECUTION_ENGINE_REST_ARM_TIMER_ROUTE,
    EXECUTION_ENGINE_REST_DISPATCH_STEP_ROUTE, EXECUTION_ENGINE_REST_SCHEDULE_RETRY_ROUTE,
    EXECUTION_ENGINE_REST_START_RUN_METHOD, EXECUTION_ENGINE_REST_START_RUN_ROUTE,
    ExecutionDomainCommandKind, ExecutionEngineApiAuthorization, ExecutionEngineApiBoundaryContext,
    ExecutionEngineApiCommandBody, ExecutionEngineApiPrincipal, ExecutionEngineApiRequest,
    ExecutionEngineApiResponseMetadata, ExecutionEngineApiStatus,
    ExecutionEngineApiSuccessResponse, ExecutionEngineRestBody, ExecutionEngineRestError,
    ExecutionEngineRestMethod, ExecutionEngineRestOperation, ExecutionEngineRestRequest,
    ExecutionEngineRestResponse, ExecutionEngineRestService, RetryPolicyEvaluator, SlaTimerStore,
    StepDispatcher, WorkflowExecutionStatus, WorkflowRunStore,
};

pub const EXECUTION_ENGINE_SDK_SURFACE: &str = "workflow-engine.execution-engine.sdk";
pub const EXECUTION_ENGINE_SDK_CONTRACT_REF: &str = EXECUTION_ENGINE_API_CONTRACT_REF;
pub const EXECUTION_ENGINE_SDK_DECLARED_VERSION: &str = EXECUTION_ENGINE_API_DECLARED_VERSION;
pub const EXECUTION_ENGINE_SDK_AUTOMATIC_RETRIES_ENABLED: bool = false;
pub const EXECUTION_ENGINE_SDK_RETRY_POLICY_REF: &str =
    "workflow-execution-sdk:automatic-retry-disabled";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExecutionEngineSdkOperation {
    StartRun,
    DispatchStep,
    ScheduleRetry,
    ArmSlaTimer,
}

impl ExecutionEngineSdkOperation {
    pub const fn command(self) -> &'static str {
        match self {
            Self::StartRun => "StartRun",
            Self::DispatchStep => "DispatchStep",
            Self::ScheduleRetry => "ScheduleRetry",
            Self::ArmSlaTimer => "ArmSlaTimer",
        }
    }

    pub const fn rest_operation(self) -> ExecutionEngineRestOperation {
        match self {
            Self::StartRun => ExecutionEngineRestOperation::StartRun,
            Self::DispatchStep => ExecutionEngineRestOperation::DispatchStep,
            Self::ScheduleRetry => ExecutionEngineRestOperation::ScheduleRetry,
            Self::ArmSlaTimer => ExecutionEngineRestOperation::ArmSlaTimer,
        }
    }

    pub const fn operation_id(self) -> &'static str {
        match self {
            Self::StartRun => "startRun",
            Self::DispatchStep => "dispatchStep",
            Self::ScheduleRetry => "scheduleRetry",
            Self::ArmSlaTimer => "armSlaTimer",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineSdkConfig {
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub principal_id: String,               // data_class: INTERNAL_ONLY
    pub authorization_decision_id: String,  // data_class: INTERNAL_ONLY
    pub authorization_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub default_spec_id: String,            // data_class: INTERNAL_ONLY
    pub default_version_sha: String,        // data_class: INTERNAL_ONLY
    pub default_cell_id: String,            // data_class: INTERNAL_ONLY
    pub spec_integrity_ref: String,         // data_class: INTERNAL_ONLY
    pub replay_epoch_ref: String,           // data_class: INTERNAL_ONLY
    pub scheduler_epoch_ref: String,        // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,          // data_class: INTERNAL_ONLY
    pub oyatie_version: String,             // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineSdkRequestContext {
    pub request_id: String,                // data_class: INTERNAL_ONLY
    pub idempotency_key: String,           // data_class: INTERNAL_ONLY
    pub trace_context_ref: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineSdkRunDescriptor {
    pub run_id: String,                  // data_class: INTERNAL_ONLY
    pub spec_id: Option<String>,         // data_class: INTERNAL_ONLY
    pub version_sha: Option<String>,     // data_class: INTERNAL_ONLY
    pub active_cell_id: Option<String>,  // data_class: INTERNAL_ONLY
    pub current_run_status: String,      // data_class: PUBLIC
    pub current_run_version: u64,        // data_class: INTERNAL_ONLY
    pub current_step_index: Option<u32>, // data_class: INTERNAL_ONLY
    pub input_ref: Option<String>,       // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayrollCloseWorkflowEnvelope {
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,            // data_class: INTERNAL_ONLY
    pub payroll_run_id: String,             // data_class: INTERNAL_ONLY
    pub workflow_ref: String,               // data_class: INTERNAL_ONLY
    pub workflow_version_sha: String,       // data_class: INTERNAL_ONLY
    pub required_steps: Vec<String>,        // data_class: INTERNAL_ONLY
    pub input_ref: String,                  // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,         // data_class: INTERNAL_ONLY
    pub idempotency_key: String,            // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,          // data_class: INTERNAL_ONLY
    pub authorization_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub cell_id: String,                    // data_class: INTERNAL_ONLY
    pub replay_epoch_ref: String,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineSdkStepDescriptor {
    pub step_id: String,                 // data_class: INTERNAL_ONLY
    pub step_index: u32,                 // data_class: INTERNAL_ONLY
    pub step_attempt: u32,               // data_class: INTERNAL_ONLY
    pub step_status: String,             // data_class: PUBLIC
    pub side_effect_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub last_error_ref: Option<String>,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineSdkRetryDescriptor {
    pub retry_attempt: u32,       // data_class: INTERNAL_ONLY
    pub error_class_ref: String,  // data_class: INTERNAL_ONLY
    pub retry_policy_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineSdkTimerDescriptor {
    pub timer_id: String,            // data_class: INTERNAL_ONLY
    pub armed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub deadline_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub step_index: Option<u32>,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineSdkRetryPolicy {
    pub automatic_retries_enabled: bool, // data_class: PUBLIC
    pub retry_policy_ref: String,        // data_class: INTERNAL_ONLY
}

impl Default for ExecutionEngineSdkRetryPolicy {
    fn default() -> Self {
        Self {
            automatic_retries_enabled: EXECUTION_ENGINE_SDK_AUTOMATIC_RETRIES_ENABLED,
            retry_policy_ref: EXECUTION_ENGINE_SDK_RETRY_POLICY_REF.to_owned(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineSdkCommandPlan {
    pub operation: ExecutionEngineSdkOperation, // data_class: PUBLIC
    pub operation_id: String,                   // data_class: PUBLIC
    pub method: ExecutionEngineRestMethod,      // data_class: PUBLIC
    pub path: String,                           // data_class: PUBLIC
    pub route_template: String,                 // data_class: PUBLIC
    pub contract_ref: String,                   // data_class: INTERNAL_ONLY
    pub oyatie_version: String,                 // data_class: PUBLIC
    pub retry_policy: ExecutionEngineSdkRetryPolicy, // data_class: INTERNAL_ONLY
    pub rest_request: ExecutionEngineRestRequest, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionEngineSdkError {
    InvalidConfig { evidence_ref: String },
    InvalidRequest { evidence_ref: String },
    UnsafeMetadata { evidence_ref: String },
    RestRejected { evidence_ref: String },
}

impl ExecutionEngineSdkError {
    pub fn primary_evidence_ref(&self) -> &str {
        match self {
            Self::InvalidConfig { evidence_ref }
            | Self::InvalidRequest { evidence_ref }
            | Self::UnsafeMetadata { evidence_ref }
            | Self::RestRejected { evidence_ref } => evidence_ref,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineSdkClient {
    pub config: ExecutionEngineSdkConfig, // data_class: INTERNAL_ONLY
    pub retry_policy: ExecutionEngineSdkRetryPolicy, // data_class: INTERNAL_ONLY
}

impl ExecutionEngineSdkClient {
    pub fn new(config: ExecutionEngineSdkConfig) -> Result<Self, ExecutionEngineSdkError> {
        validate_config(&config)?;
        Ok(Self {
            config,
            retry_policy: ExecutionEngineSdkRetryPolicy::default(),
        })
    }

    pub fn start_run(
        &self,
        context: ExecutionEngineSdkRequestContext,
        run: ExecutionEngineSdkRunDescriptor,
        initial_step: ExecutionEngineSdkStepDescriptor,
    ) -> Result<ExecutionEngineSdkCommandPlan, ExecutionEngineSdkError> {
        self.plan_command(
            ExecutionEngineSdkOperation::StartRun,
            context,
            run,
            Some(initial_step),
            None,
            None,
        )
    }

    pub fn plan_payroll_close_start_run(
        &self,
        envelope: PayrollCloseWorkflowEnvelope,
    ) -> Result<ExecutionEngineSdkCommandPlan, ExecutionEngineSdkError> {
        validate_payroll_close_envelope(&self.config, &envelope)?;

        let scoped_client = Self::new(ExecutionEngineSdkConfig {
            authorization_evidence_ref: envelope.authorization_evidence_ref.clone(),
            default_spec_id: envelope.workflow_ref.clone(),
            default_version_sha: envelope.workflow_version_sha.clone(),
            default_cell_id: envelope.cell_id.clone(),
            replay_epoch_ref: envelope.replay_epoch_ref.clone(),
            trace_context_ref: envelope.trace_context_ref.clone(),
            ..self.config.clone()
        })?;
        scoped_client.start_run(
            ExecutionEngineSdkRequestContext {
                request_id: payroll_close_request_id(&envelope),
                idempotency_key: envelope.idempotency_key.clone(),
                trace_context_ref: Some(envelope.trace_context_ref.clone()),
            },
            ExecutionEngineSdkRunDescriptor {
                run_id: payroll_close_workflow_run_id(&envelope),
                spec_id: Some(envelope.workflow_ref.clone()),
                version_sha: Some(envelope.workflow_version_sha.clone()),
                active_cell_id: Some(envelope.cell_id.clone()),
                current_run_status: "pending".to_owned(),
                current_run_version: 1,
                current_step_index: Some(1),
                input_ref: Some(envelope.input_ref.clone()),
                evidence_refs: payroll_close_evidence_refs(&envelope),
            },
            ExecutionEngineSdkStepDescriptor {
                step_id: envelope
                    .required_steps
                    .first()
                    .cloned()
                    .ok_or_else(|| invalid_request("workflow-payroll-close-intake:missing-step"))?,
                step_index: 1,
                step_attempt: 1,
                step_status: "pending".to_owned(),
                side_effect_ref: None,
                last_error_ref: None,
            },
        )
    }

    pub fn dispatch_step(
        &self,
        context: ExecutionEngineSdkRequestContext,
        run: ExecutionEngineSdkRunDescriptor,
        step: ExecutionEngineSdkStepDescriptor,
    ) -> Result<ExecutionEngineSdkCommandPlan, ExecutionEngineSdkError> {
        self.plan_command(
            ExecutionEngineSdkOperation::DispatchStep,
            context,
            run,
            Some(step),
            None,
            None,
        )
    }

    pub fn schedule_retry(
        &self,
        context: ExecutionEngineSdkRequestContext,
        run: ExecutionEngineSdkRunDescriptor,
        step: ExecutionEngineSdkStepDescriptor,
        retry: ExecutionEngineSdkRetryDescriptor,
    ) -> Result<ExecutionEngineSdkCommandPlan, ExecutionEngineSdkError> {
        self.plan_command(
            ExecutionEngineSdkOperation::ScheduleRetry,
            context,
            run,
            Some(step),
            Some(retry),
            None,
        )
    }

    pub fn arm_sla_timer(
        &self,
        context: ExecutionEngineSdkRequestContext,
        run: ExecutionEngineSdkRunDescriptor,
        timer: ExecutionEngineSdkTimerDescriptor,
    ) -> Result<ExecutionEngineSdkCommandPlan, ExecutionEngineSdkError> {
        self.plan_command(
            ExecutionEngineSdkOperation::ArmSlaTimer,
            context,
            run,
            None,
            None,
            Some(timer),
        )
    }

    pub fn execute_in_process<S, D, R, T>(
        &self,
        rest_service: &mut ExecutionEngineRestService,
        store: &mut S,
        dispatcher: &mut D,
        retry_policy: &R,
        timers: &mut T,
        plan: ExecutionEngineSdkCommandPlan,
    ) -> Result<ExecutionEngineRestResponse, ExecutionEngineSdkError>
    where
        S: WorkflowRunStore,
        D: StepDispatcher,
        R: RetryPolicyEvaluator,
        T: SlaTimerStore,
    {
        validate_plan(&plan)?;
        rest_service
            .handle(store, dispatcher, retry_policy, timers, plan.rest_request)
            .map_err(|error| ExecutionEngineSdkError::RestRejected {
                evidence_ref: error.reason_ref,
            })
    }

    fn plan_command(
        &self,
        operation: ExecutionEngineSdkOperation,
        context: ExecutionEngineSdkRequestContext,
        run: ExecutionEngineSdkRunDescriptor,
        step: Option<ExecutionEngineSdkStepDescriptor>,
        retry: Option<ExecutionEngineSdkRetryDescriptor>,
        timer: Option<ExecutionEngineSdkTimerDescriptor>,
    ) -> Result<ExecutionEngineSdkCommandPlan, ExecutionEngineSdkError> {
        validate_context(&context)?;
        validate_run(&run)?;
        if let Some(step) = &step {
            validate_step(step)?;
        }
        if let Some(retry) = &retry {
            validate_retry(retry)?;
        }
        if let Some(timer) = &timer {
            validate_timer(timer)?;
        }
        validate_shape(operation, step.as_ref(), retry.as_ref(), timer.as_ref())?;

        let body = command_body(
            &self.config,
            operation,
            &run,
            step.as_ref(),
            retry.as_ref(),
            timer.as_ref(),
        );
        let api_request = ExecutionEngineApiRequest {
            boundary: ExecutionEngineApiBoundaryContext {
                request_id: context.request_id.clone(),
                tenant_id: self.config.tenant_id.clone(),
                idempotency_key: context.idempotency_key.clone(),
                trace_context_ref: context
                    .trace_context_ref
                    .clone()
                    .unwrap_or_else(|| self.config.trace_context_ref.clone()),
                oyatie_version: self.config.oyatie_version.clone(),
            },
            principal: ExecutionEngineApiPrincipal {
                tenant_id: self.config.tenant_id.clone(),
                principal_id: self.config.principal_id.clone(),
            },
            authorization: ExecutionEngineApiAuthorization {
                tenant_id: self.config.tenant_id.clone(),
                principal_id: self.config.principal_id.clone(),
                decision_id: self.config.authorization_decision_id.clone(),
                evidence_ref: self.config.authorization_evidence_ref.clone(),
                allowed_surfaces: sorted_unique(vec![
                    EXECUTION_ENGINE_API_SURFACE.to_owned(),
                    EXECUTION_ENGINE_SDK_SURFACE.to_owned(),
                ]),
            },
            expected_run_version: run.current_run_version,
            expected_spec_id: run
                .spec_id
                .clone()
                .unwrap_or_else(|| self.config.default_spec_id.clone()),
            expected_version_sha: run
                .version_sha
                .clone()
                .unwrap_or_else(|| self.config.default_version_sha.clone()),
            expected_cell_id: run
                .active_cell_id
                .clone()
                .unwrap_or_else(|| self.config.default_cell_id.clone()),
            spec_integrity_ref: self.config.spec_integrity_ref.clone(),
            replay_epoch_ref: self.config.replay_epoch_ref.clone(),
            scheduler_epoch_ref: self.config.scheduler_epoch_ref.clone(),
            route_run_id: None,
            route_step_index: None,
            body,
        };
        let (path, route_template) = path_for(operation, &run, step.as_ref())?;
        Ok(ExecutionEngineSdkCommandPlan {
            operation,
            operation_id: operation.operation_id().to_owned(),
            method: EXECUTION_ENGINE_REST_START_RUN_METHOD,
            path: path.clone(),
            route_template,
            contract_ref: EXECUTION_ENGINE_SDK_CONTRACT_REF.to_owned(),
            oyatie_version: EXECUTION_ENGINE_SDK_DECLARED_VERSION.to_owned(),
            retry_policy: self.retry_policy.clone(),
            rest_request: ExecutionEngineRestRequest {
                method: EXECUTION_ENGINE_REST_START_RUN_METHOD,
                path,
                request_id: context.request_id,
                body: api_request,
            },
            evidence_refs: sorted_unique(vec![
                EXECUTION_ENGINE_SDK_RETRY_POLICY_REF.to_owned(),
                format!(
                    "workflow-execution-sdk:operation:{}",
                    operation.operation_id()
                ),
                "workflow-execution-sdk:request-planned".to_owned(),
            ]),
        })
    }
}

fn command_body(
    config: &ExecutionEngineSdkConfig,
    operation: ExecutionEngineSdkOperation,
    run: &ExecutionEngineSdkRunDescriptor,
    step: Option<&ExecutionEngineSdkStepDescriptor>,
    retry: Option<&ExecutionEngineSdkRetryDescriptor>,
    timer: Option<&ExecutionEngineSdkTimerDescriptor>,
) -> ExecutionEngineApiCommandBody {
    ExecutionEngineApiCommandBody {
        command: operation.command().to_owned(),
        run_id: run.run_id.clone(),
        spec_id: run
            .spec_id
            .clone()
            .unwrap_or_else(|| config.default_spec_id.clone()),
        version_sha: run
            .version_sha
            .clone()
            .unwrap_or_else(|| config.default_version_sha.clone()),
        active_cell_id: run
            .active_cell_id
            .clone()
            .unwrap_or_else(|| config.default_cell_id.clone()),
        current_run_status: run.current_run_status.clone(),
        current_run_version: run.current_run_version,
        current_step_index: run.current_step_index,
        step_id: step.map(|value| value.step_id.clone()),
        step_index: step
            .map(|value| value.step_index)
            .or(timer.and_then(|value| value.step_index)),
        step_attempt: step.map(|value| value.step_attempt),
        step_status: step.map(|value| value.step_status.clone()),
        side_effect_ref: step.and_then(|value| value.side_effect_ref.clone()),
        last_error_ref: step.and_then(|value| value.last_error_ref.clone()),
        retry_attempt: retry.map(|value| value.retry_attempt),
        error_class_ref: retry.map(|value| value.error_class_ref.clone()),
        retry_policy_ref: retry.map(|value| value.retry_policy_ref.clone()),
        timer_id: timer.map(|value| value.timer_id.clone()),
        armed_at_epoch_seconds: timer.map(|value| value.armed_at_epoch_seconds),
        deadline_epoch_seconds: timer.map(|value| value.deadline_epoch_seconds),
        input_ref: run.input_ref.clone(),
        evidence_refs: sorted_unique(
            [
                run.evidence_refs.clone(),
                vec![
                    "surface:workflow-engine.execution-engine.sdk".to_owned(),
                    format!(
                        "workflow-execution-sdk:operation:{}",
                        operation.operation_id()
                    ),
                ],
            ]
            .concat(),
        ),
    }
}

fn path_for(
    operation: ExecutionEngineSdkOperation,
    run: &ExecutionEngineSdkRunDescriptor,
    step: Option<&ExecutionEngineSdkStepDescriptor>,
) -> Result<(String, String), ExecutionEngineSdkError> {
    match operation {
        ExecutionEngineSdkOperation::StartRun => Ok((
            EXECUTION_ENGINE_REST_START_RUN_ROUTE.to_owned(),
            EXECUTION_ENGINE_REST_START_RUN_ROUTE.to_owned(),
        )),
        ExecutionEngineSdkOperation::DispatchStep => {
            let step =
                step.ok_or_else(|| invalid_request("workflow-execution-sdk:missing-step"))?;
            Ok((
                format!("/runs/{}/steps/{}/dispatch", run.run_id, step.step_index),
                EXECUTION_ENGINE_REST_DISPATCH_STEP_ROUTE.to_owned(),
            ))
        }
        ExecutionEngineSdkOperation::ScheduleRetry => {
            let step =
                step.ok_or_else(|| invalid_request("workflow-execution-sdk:missing-step"))?;
            Ok((
                format!("/runs/{}/steps/{}/retry", run.run_id, step.step_index),
                EXECUTION_ENGINE_REST_SCHEDULE_RETRY_ROUTE.to_owned(),
            ))
        }
        ExecutionEngineSdkOperation::ArmSlaTimer => Ok((
            format!("/runs/{}/timers", run.run_id),
            EXECUTION_ENGINE_REST_ARM_TIMER_ROUTE.to_owned(),
        )),
    }
}

fn validate_shape(
    operation: ExecutionEngineSdkOperation,
    step: Option<&ExecutionEngineSdkStepDescriptor>,
    retry: Option<&ExecutionEngineSdkRetryDescriptor>,
    timer: Option<&ExecutionEngineSdkTimerDescriptor>,
) -> Result<(), ExecutionEngineSdkError> {
    match operation {
        ExecutionEngineSdkOperation::StartRun
            if step.is_some() && retry.is_none() && timer.is_none() =>
        {
            Ok(())
        }
        ExecutionEngineSdkOperation::DispatchStep
            if step.is_some() && retry.is_none() && timer.is_none() =>
        {
            Ok(())
        }
        ExecutionEngineSdkOperation::ScheduleRetry
            if step.is_some() && retry.is_some() && timer.is_none() =>
        {
            Ok(())
        }
        ExecutionEngineSdkOperation::ArmSlaTimer
            if step.is_none() && retry.is_none() && timer.is_some() =>
        {
            Ok(())
        }
        _ => Err(invalid_request(
            "workflow-execution-sdk:command-shape-invalid",
        )),
    }
}

fn validate_payroll_close_envelope(
    config: &ExecutionEngineSdkConfig,
    envelope: &PayrollCloseWorkflowEnvelope,
) -> Result<(), ExecutionEngineSdkError> {
    if envelope.tenant_id != config.tenant_id {
        return Err(invalid_request(
            "workflow-payroll-close-intake:tenant-mismatch",
        ));
    }
    if envelope.workflow_ref != config.default_spec_id
        || envelope.workflow_version_sha != config.default_version_sha
        || envelope.cell_id != config.default_cell_id
    {
        return Err(invalid_request(
            "workflow-payroll-close-intake:boundary-mismatch",
        ));
    }
    if envelope.required_steps.is_empty() {
        return Err(invalid_request(
            "workflow-payroll-close-intake:missing-step",
        ));
    }
    if !is_safe_tenant(&envelope.tenant_id)
        || !is_safe_token(&envelope.legal_entity_id)
        || !is_safe_token(&envelope.payroll_run_id)
        || !is_safe_ref(&envelope.workflow_ref)
        || !is_safe_ref(&envelope.workflow_version_sha)
        || !envelope
            .required_steps
            .iter()
            .all(|value| is_safe_ref(value))
        || !is_safe_ref(&envelope.input_ref)
        || !envelope
            .evidence_refs
            .iter()
            .all(|value| is_safe_ref(value))
        || !is_safe_ref(&envelope.idempotency_key)
        || !is_safe_ref(&envelope.trace_context_ref)
        || !is_safe_ref(&envelope.authorization_evidence_ref)
        || !is_safe_ref(&envelope.cell_id)
        || !is_safe_ref(&envelope.replay_epoch_ref)
        || !is_safe_ref(&payroll_close_workflow_run_id(envelope))
    {
        return Err(ExecutionEngineSdkError::UnsafeMetadata {
            evidence_ref: "workflow-payroll-close-intake:unsafe-metadata".to_owned(),
        });
    }

    let has_approval_step = envelope
        .required_steps
        .iter()
        .any(|value| contains_ref_fragment(value, "approval"));
    let has_evidence_gate_step = envelope.required_steps.iter().any(|value| {
        contains_ref_fragment(value, "evidence-gate") || contains_ref_fragment(value, "canary")
    });
    if !has_approval_step || !has_evidence_gate_step {
        return Err(invalid_request(
            "workflow-payroll-close-intake:missing-approval-or-evidence-gate",
        ));
    }

    let close_gate_failed = envelope
        .evidence_refs
        .iter()
        .any(|value| contains_ref_fragment(value, "failed"));
    if close_gate_failed {
        let has_repair_step = envelope.required_steps.iter().any(|value| {
            contains_ref_fragment(value, "rollback") || contains_ref_fragment(value, "quarantine")
        });
        let has_repair_evidence = envelope.evidence_refs.iter().any(|value| {
            contains_ref_fragment(value, "rollback") || contains_ref_fragment(value, "quarantine")
        });
        if !has_repair_step || !has_repair_evidence {
            return Err(invalid_request(
                "workflow-payroll-close-intake:missing-rollback-quarantine-evidence",
            ));
        }
    }

    Ok(())
}

fn payroll_close_request_id(envelope: &PayrollCloseWorkflowEnvelope) -> String {
    format!(
        "req:workflow:payroll-close:{}:{}:{}",
        envelope.tenant_id, envelope.legal_entity_id, envelope.payroll_run_id
    )
}

fn payroll_close_workflow_run_id(envelope: &PayrollCloseWorkflowEnvelope) -> String {
    format!(
        "run:workflow:payroll-close:{}:{}:{}",
        envelope.tenant_id, envelope.legal_entity_id, envelope.payroll_run_id
    )
}

fn payroll_close_evidence_refs(envelope: &PayrollCloseWorkflowEnvelope) -> Vec<String> {
    let mut evidence_refs = envelope.evidence_refs.clone();
    evidence_refs.push(envelope.authorization_evidence_ref.clone());
    evidence_refs.push(envelope.replay_epoch_ref.clone());
    sorted_unique(evidence_refs)
}

fn is_safe_token(value: &str) -> bool {
    is_safe_metadata(value) && !value.contains(':')
}

fn contains_ref_fragment(value: &str, fragment: &str) -> bool {
    value.to_ascii_lowercase().contains(fragment)
}

fn validate_config(config: &ExecutionEngineSdkConfig) -> Result<(), ExecutionEngineSdkError> {
    if !is_safe_tenant(&config.tenant_id)
        || !is_safe_ref(&config.principal_id)
        || !is_safe_ref(&config.authorization_decision_id)
        || !is_safe_ref(&config.authorization_evidence_ref)
        || !is_safe_ref(&config.default_spec_id)
        || !is_safe_ref(&config.default_version_sha)
        || !is_safe_ref(&config.default_cell_id)
        || !is_safe_ref(&config.spec_integrity_ref)
        || !is_safe_ref(&config.replay_epoch_ref)
        || !is_safe_ref(&config.scheduler_epoch_ref)
        || !is_safe_ref(&config.trace_context_ref)
    {
        return Err(ExecutionEngineSdkError::InvalidConfig {
            evidence_ref: "workflow-execution-sdk:invalid-config-metadata".to_owned(),
        });
    }
    if config.oyatie_version != EXECUTION_ENGINE_SDK_DECLARED_VERSION {
        return Err(ExecutionEngineSdkError::InvalidConfig {
            evidence_ref: "workflow-execution-sdk:unsupported-version".to_owned(),
        });
    }
    Ok(())
}

fn validate_context(
    context: &ExecutionEngineSdkRequestContext,
) -> Result<(), ExecutionEngineSdkError> {
    if !is_safe_ref(&context.request_id)
        || !is_safe_ref(&context.idempotency_key)
        || !context
            .trace_context_ref
            .as_ref()
            .is_none_or(|value| is_safe_ref(value))
    {
        return Err(ExecutionEngineSdkError::UnsafeMetadata {
            evidence_ref: "workflow-execution-sdk:unsafe-request-context".to_owned(),
        });
    }
    Ok(())
}

fn validate_run(run: &ExecutionEngineSdkRunDescriptor) -> Result<(), ExecutionEngineSdkError> {
    if !is_safe_ref(&run.run_id)
        || !run.spec_id.as_ref().is_none_or(|value| is_safe_ref(value))
        || !run
            .version_sha
            .as_ref()
            .is_none_or(|value| is_safe_ref(value))
        || !run
            .active_cell_id
            .as_ref()
            .is_none_or(|value| is_safe_ref(value))
        || !run
            .input_ref
            .as_ref()
            .is_none_or(|value| is_safe_ref(value))
        || !run.evidence_refs.iter().all(|value| is_safe_ref(value))
    {
        return Err(ExecutionEngineSdkError::UnsafeMetadata {
            evidence_ref: "workflow-execution-sdk:unsafe-run-metadata".to_owned(),
        });
    }
    if WorkflowExecutionStatus::from_wire(&run.current_run_status).is_none() {
        return Err(invalid_request("workflow-execution-sdk:unknown-run-status"));
    }
    if run.current_run_version == 0 {
        return Err(invalid_request(
            "workflow-execution-sdk:run-version-invalid",
        ));
    }
    Ok(())
}

fn validate_step(step: &ExecutionEngineSdkStepDescriptor) -> Result<(), ExecutionEngineSdkError> {
    if !is_safe_ref(&step.step_id)
        || !step
            .side_effect_ref
            .as_ref()
            .is_none_or(|value| is_safe_ref(value))
        || !step
            .last_error_ref
            .as_ref()
            .is_none_or(|value| is_safe_ref(value))
    {
        return Err(ExecutionEngineSdkError::UnsafeMetadata {
            evidence_ref: "workflow-execution-sdk:unsafe-step-metadata".to_owned(),
        });
    }
    if step.step_attempt == 0 {
        return Err(invalid_request(
            "workflow-execution-sdk:step-attempt-invalid",
        ));
    }
    if oya_workflow_engine_execution_engine_rest::StepExecutionStatus::from_wire(&step.step_status)
        .is_none()
    {
        return Err(invalid_request(
            "workflow-execution-sdk:unknown-step-status",
        ));
    }
    Ok(())
}

fn validate_retry(
    retry: &ExecutionEngineSdkRetryDescriptor,
) -> Result<(), ExecutionEngineSdkError> {
    if retry.retry_attempt == 0
        || !is_safe_ref(&retry.error_class_ref)
        || !is_safe_ref(&retry.retry_policy_ref)
    {
        return Err(ExecutionEngineSdkError::UnsafeMetadata {
            evidence_ref: "workflow-execution-sdk:unsafe-retry-metadata".to_owned(),
        });
    }
    Ok(())
}

fn validate_timer(
    timer: &ExecutionEngineSdkTimerDescriptor,
) -> Result<(), ExecutionEngineSdkError> {
    if !is_safe_ref(&timer.timer_id) || timer.deadline_epoch_seconds <= timer.armed_at_epoch_seconds
    {
        return Err(ExecutionEngineSdkError::UnsafeMetadata {
            evidence_ref: "workflow-execution-sdk:unsafe-timer-metadata".to_owned(),
        });
    }
    Ok(())
}

fn validate_plan(plan: &ExecutionEngineSdkCommandPlan) -> Result<(), ExecutionEngineSdkError> {
    if plan.retry_policy.automatic_retries_enabled {
        return Err(invalid_request(
            "workflow-execution-sdk:automatic-retry-forbidden",
        ));
    }
    if plan.oyatie_version != EXECUTION_ENGINE_SDK_DECLARED_VERSION
        || plan.contract_ref != EXECUTION_ENGINE_SDK_CONTRACT_REF
        || plan.method != EXECUTION_ENGINE_REST_START_RUN_METHOD
        || !is_safe_ref(&plan.rest_request.request_id)
        || !plan.evidence_refs.iter().all(|value| is_safe_ref(value))
    {
        return Err(invalid_request("workflow-execution-sdk:invalid-plan"));
    }
    Ok(())
}

fn invalid_request(evidence_ref: &str) -> ExecutionEngineSdkError {
    ExecutionEngineSdkError::InvalidRequest {
        evidence_ref: evidence_ref.to_owned(),
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
        || lower.contains("secret=")
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
    use oya_workflow_engine_execution_engine_adapter::{
        ExecutionAdapterActionKind, WorkflowExecutionMemoryAdapter,
    };

    fn config() -> ExecutionEngineSdkConfig {
        ExecutionEngineSdkConfig {
            tenant_id: "ten_execution_sdk".to_owned(),
            principal_id: "principal:workflow-sdk:tester".to_owned(),
            authorization_decision_id: "authz:workflow-sdk:allow".to_owned(),
            authorization_evidence_ref: "policy:workflow-sdk:allow".to_owned(),
            default_spec_id: "spec:workflow:invoice".to_owned(),
            default_version_sha: "sha:workflow:invoice:v1".to_owned(),
            default_cell_id: "cell:us-east-1a".to_owned(),
            spec_integrity_ref: "integrity:workflow:invoice".to_owned(),
            replay_epoch_ref: "replay:epoch:20260525".to_owned(),
            scheduler_epoch_ref: "scheduler:epoch:20260525".to_owned(),
            trace_context_ref: "trace:workflow-sdk:root".to_owned(),
            oyatie_version: EXECUTION_ENGINE_SDK_DECLARED_VERSION.to_owned(),
        }
    }

    fn context(seq: u32) -> ExecutionEngineSdkRequestContext {
        ExecutionEngineSdkRequestContext {
            request_id: format!("req:workflow-sdk:{seq}"),
            idempotency_key: format!("idem:workflow-sdk:{seq}"),
            trace_context_ref: None,
        }
    }

    fn run(status: &str, version: u64) -> ExecutionEngineSdkRunDescriptor {
        ExecutionEngineSdkRunDescriptor {
            run_id: "run:workflow-sdk:invoice:1".to_owned(),
            spec_id: None,
            version_sha: None,
            active_cell_id: None,
            current_run_status: status.to_owned(),
            current_run_version: version,
            current_step_index: Some(1),
            input_ref: Some("input:workflow-sdk:invoice:1".to_owned()),
            evidence_refs: vec!["evidence:workflow-sdk:invoice:1".to_owned()],
        }
    }

    fn step(status: &str) -> ExecutionEngineSdkStepDescriptor {
        ExecutionEngineSdkStepDescriptor {
            step_id: "step:workflow-sdk:approve".to_owned(),
            step_index: 1,
            step_attempt: 1,
            step_status: status.to_owned(),
            side_effect_ref: Some("effect:workflow-sdk:approval".to_owned()),
            last_error_ref: None,
        }
    }

    fn payroll_config() -> ExecutionEngineSdkConfig {
        ExecutionEngineSdkConfig {
            tenant_id: "ten_acme".to_owned(),
            principal_id: "principal:workflow-sdk:payroll-close".to_owned(),
            authorization_decision_id: "authz:workflow:payroll-close:allow".to_owned(),
            authorization_evidence_ref: "policy:workflow:payroll-close:allow".to_owned(),
            default_spec_id: "spec:workflow:payroll-close".to_owned(),
            default_version_sha: "sha:workflow:payroll-close:v1".to_owned(),
            default_cell_id: "cell:kr:payroll-close".to_owned(),
            spec_integrity_ref: "integrity:workflow:payroll-close:v1".to_owned(),
            replay_epoch_ref: "replay:workflow:payroll-close:epoch:001".to_owned(),
            scheduler_epoch_ref: "scheduler:workflow:payroll-close:epoch:001".to_owned(),
            trace_context_ref: "trace:workflow:payroll-close:root".to_owned(),
            oyatie_version: EXECUTION_ENGINE_SDK_DECLARED_VERSION.to_owned(),
        }
    }

    fn payroll_envelope() -> PayrollCloseWorkflowEnvelope {
        PayrollCloseWorkflowEnvelope {
            tenant_id: "ten_acme".to_owned(),
            legal_entity_id: "le_kr_001".to_owned(),
            payroll_run_id: "prun_kr_2026_01".to_owned(),
            workflow_ref: "spec:workflow:payroll-close".to_owned(),
            workflow_version_sha: "sha:workflow:payroll-close:v1".to_owned(),
            required_steps: vec![
                "step:workflow:payroll-close:approval".to_owned(),
                "step:workflow:payroll-close:evidence-gate".to_owned(),
                "step:workflow:payroll-close:rollback-quarantine".to_owned(),
                "step:workflow:payroll-close:resume-promotion".to_owned(),
            ],
            input_ref: "payroll-close-envelope:ten_acme:le_kr_001:prun_kr_2026_01:v1".to_owned(),
            evidence_refs: vec![
                "evidence:payroll:close:canary:passed".to_owned(),
                "evidence:payroll:close:approval:sha1".to_owned(),
                "evidence:payroll:close:approval:sha1".to_owned(),
                "evidence:payroll:close:rollback:ready".to_owned(),
            ],
            idempotency_key:
                "workflow-payroll-close:ten_acme:le_kr_001:prun_kr_2026_01:sha_payroll_close_v1"
                    .to_owned(),
            trace_context_ref: "trace:workflow:payroll-close:prun_kr_2026_01".to_owned(),
            authorization_evidence_ref: "evidence:workflow:payroll-close:policy:allow".to_owned(),
            cell_id: "cell:kr:payroll-close".to_owned(),
            replay_epoch_ref: "evidence:workflow:payroll-close:replay:epoch:001".to_owned(),
        }
    }

    #[test]
    fn payroll_close_envelope_plans_start_run_with_tenant_idempotency_evidence_and_replay_safety() {
        let client =
            ExecutionEngineSdkClient::new(payroll_config()).expect("valid payroll sdk config");
        let plan = client
            .plan_payroll_close_start_run(payroll_envelope())
            .expect("payroll close plan");

        assert_eq!(plan.operation, ExecutionEngineSdkOperation::StartRun);
        assert_eq!(plan.operation_id, "startRun");
        assert_eq!(plan.path, "/runs");
        assert_eq!(plan.route_template, EXECUTION_ENGINE_REST_START_RUN_ROUTE);
        assert!(!plan.retry_policy.automatic_retries_enabled);
        assert_eq!(
            plan.rest_request.body.body.run_id,
            "run:workflow:payroll-close:ten_acme:le_kr_001:prun_kr_2026_01"
        );
        assert_eq!(
            plan.rest_request.body.body.spec_id,
            "spec:workflow:payroll-close"
        );
        assert_eq!(
            plan.rest_request.body.body.version_sha,
            "sha:workflow:payroll-close:v1"
        );
        assert_eq!(
            plan.rest_request.body.body.active_cell_id,
            "cell:kr:payroll-close"
        );
        assert_eq!(plan.rest_request.body.boundary.tenant_id, "ten_acme");
        assert_eq!(
            plan.rest_request.body.boundary.idempotency_key,
            "workflow-payroll-close:ten_acme:le_kr_001:prun_kr_2026_01:sha_payroll_close_v1"
        );
        assert_eq!(
            plan.rest_request.body.boundary.trace_context_ref,
            "trace:workflow:payroll-close:prun_kr_2026_01"
        );
        assert_eq!(
            plan.rest_request.body.authorization.evidence_ref,
            "evidence:workflow:payroll-close:policy:allow"
        );
        assert_eq!(
            plan.rest_request.body.body.input_ref.as_deref(),
            Some("payroll-close-envelope:ten_acme:le_kr_001:prun_kr_2026_01:v1")
        );
        assert_eq!(
            plan.rest_request.body.body.step_id.as_deref(),
            Some("step:workflow:payroll-close:approval")
        );
        assert_eq!(
            plan.rest_request.body.body.evidence_refs,
            vec![
                "evidence:payroll:close:approval:sha1".to_owned(),
                "evidence:payroll:close:canary:passed".to_owned(),
                "evidence:payroll:close:rollback:ready".to_owned(),
                "evidence:workflow:payroll-close:policy:allow".to_owned(),
                "evidence:workflow:payroll-close:replay:epoch:001".to_owned(),
                "surface:workflow-engine.execution-engine.sdk".to_owned(),
                "workflow-execution-sdk:operation:startRun".to_owned(),
            ]
        );

        let mut rest = ExecutionEngineRestService::default();
        let mut store = WorkflowExecutionMemoryAdapter::default();
        let mut dispatcher = WorkflowExecutionMemoryAdapter::default();
        let retry_policy = WorkflowExecutionMemoryAdapter::default();
        let mut timers = WorkflowExecutionMemoryAdapter::default();
        let first = client
            .execute_in_process(
                &mut rest,
                &mut store,
                &mut dispatcher,
                &retry_policy,
                &mut timers,
                plan.clone(),
            )
            .expect("first payroll close response");
        assert_eq!(first.status_code, 201);
        let action_count_after_first = store.recorded_actions().len();
        let second = client
            .execute_in_process(
                &mut rest,
                &mut store,
                &mut dispatcher,
                &retry_policy,
                &mut timers,
                plan,
            )
            .expect("idempotent payroll close replay");
        assert_eq!(first, second);
        assert_eq!(store.run_count(), 1);
        assert_eq!(store.recorded_actions().len(), action_count_after_first);

        let tenant_error = client
            .plan_payroll_close_start_run(PayrollCloseWorkflowEnvelope {
                tenant_id: "ten_other".to_owned(),
                ..payroll_envelope()
            })
            .expect_err("tenant mismatch denied");
        assert_eq!(
            tenant_error.primary_evidence_ref(),
            "workflow-payroll-close-intake:tenant-mismatch"
        );

        let missing_gate_error = client
            .plan_payroll_close_start_run(PayrollCloseWorkflowEnvelope {
                required_steps: vec!["step:workflow:payroll-close:resume-promotion".to_owned()],
                ..payroll_envelope()
            })
            .expect_err("approval/evidence gates required");
        assert_eq!(
            missing_gate_error.primary_evidence_ref(),
            "workflow-payroll-close-intake:missing-approval-or-evidence-gate"
        );

        let failed_without_repair_error = client
            .plan_payroll_close_start_run(PayrollCloseWorkflowEnvelope {
                required_steps: vec![
                    "step:workflow:payroll-close:approval".to_owned(),
                    "step:workflow:payroll-close:evidence-gate".to_owned(),
                ],
                evidence_refs: vec![
                    "evidence:payroll:close:approval:sha1".to_owned(),
                    "evidence:payroll:close:evidence-gate:failed".to_owned(),
                ],
                ..payroll_envelope()
            })
            .expect_err("failed close gate requires rollback/quarantine evidence");
        assert_eq!(
            failed_without_repair_error.primary_evidence_ref(),
            "workflow-payroll-close-intake:missing-rollback-quarantine-evidence"
        );

        let unsafe_ref_error = client
            .plan_payroll_close_start_run(PayrollCloseWorkflowEnvelope {
                input_ref: "payload:raw-payroll:salary-details".to_owned(),
                ..payroll_envelope()
            })
            .expect_err("raw payload ref denied");
        assert_eq!(
            unsafe_ref_error.primary_evidence_ref(),
            "workflow-payroll-close-intake:unsafe-metadata"
        );

        let conflict = client
            .plan_payroll_close_start_run(PayrollCloseWorkflowEnvelope {
                input_ref: "payroll-close-envelope:ten_acme:le_kr_001:prun_kr_2026_01:v2"
                    .to_owned(),
                ..payroll_envelope()
            })
            .expect("conflicting replay plan");
        let conflict = client
            .execute_in_process(
                &mut rest,
                &mut store,
                &mut dispatcher,
                &retry_policy,
                &mut timers,
                conflict,
            )
            .expect("conflicting replay response");
        assert_eq!(conflict.status_code, 409);
        assert_eq!(store.run_count(), 1);
        assert_eq!(store.recorded_actions().len(), action_count_after_first);
    }

    #[test]
    fn sdk_constants_and_defaults_are_contract_bound_and_retry_off() {
        let client = ExecutionEngineSdkClient::new(config()).expect("valid sdk config");
        assert_eq!(
            EXECUTION_ENGINE_SDK_SURFACE,
            "workflow-engine.execution-engine.sdk"
        );
        assert_eq!(
            EXECUTION_ENGINE_SDK_DECLARED_VERSION,
            EXECUTION_ENGINE_API_DECLARED_VERSION
        );
        assert!(!client.retry_policy.automatic_retries_enabled);
        assert_eq!(
            client.retry_policy.retry_policy_ref,
            EXECUTION_ENGINE_SDK_RETRY_POLICY_REF
        );
    }

    #[test]
    fn start_run_plan_binds_version_authorization_idempotency_and_route() {
        let client = ExecutionEngineSdkClient::new(config()).expect("valid sdk config");
        let plan = client
            .start_run(context(1), run("pending", 1), step("pending"))
            .expect("plan");
        assert_eq!(plan.operation, ExecutionEngineSdkOperation::StartRun);
        assert_eq!(plan.operation_id, "startRun");
        assert_eq!(plan.method, ExecutionEngineRestMethod::Post);
        assert_eq!(plan.path, "/runs");
        assert_eq!(
            plan.rest_request.body.boundary.oyatie_version,
            EXECUTION_ENGINE_API_DECLARED_VERSION
        );
        assert_eq!(
            plan.rest_request.body.boundary.idempotency_key,
            "idem:workflow-sdk:1"
        );
        assert_eq!(
            plan.rest_request.body.authorization.evidence_ref,
            "policy:workflow-sdk:allow"
        );
        assert!(
            plan.rest_request
                .body
                .authorization
                .allowed_surfaces
                .contains(&EXECUTION_ENGINE_API_SURFACE.to_owned())
        );
        assert_eq!(plan.rest_request.body.body.command, "StartRun");
        assert_eq!(plan.rest_request.body.body.spec_id, "spec:workflow:invoice");
        assert!(
            plan.evidence_refs
                .contains(&"workflow-execution-sdk:request-planned".to_owned())
        );
    }

    #[test]
    fn dispatch_retry_and_timer_plans_bind_routes_and_commands() {
        let client = ExecutionEngineSdkClient::new(config()).expect("valid sdk config");
        let dispatch = client
            .dispatch_step(context(2), run("running", 2), step("pending"))
            .expect("dispatch plan");
        assert_eq!(
            dispatch.path,
            "/runs/run:workflow-sdk:invoice:1/steps/1/dispatch"
        );
        assert_eq!(
            dispatch.route_template,
            EXECUTION_ENGINE_REST_DISPATCH_STEP_ROUTE
        );
        assert_eq!(dispatch.rest_request.body.body.command, "DispatchStep");
        assert_eq!(dispatch.rest_request.body.body.step_index, Some(1));

        let retry = client
            .schedule_retry(
                context(3),
                run("running", 2),
                step("failed"),
                ExecutionEngineSdkRetryDescriptor {
                    retry_attempt: 2,
                    error_class_ref: "error:workflow-sdk:timeout".to_owned(),
                    retry_policy_ref: "retry-policy:workflow-sdk:standard".to_owned(),
                },
            )
            .expect("retry plan");
        assert_eq!(retry.path, "/runs/run:workflow-sdk:invoice:1/steps/1/retry");
        assert_eq!(retry.rest_request.body.body.command, "ScheduleRetry");
        assert_eq!(retry.rest_request.body.body.retry_attempt, Some(2));

        let timer = client
            .arm_sla_timer(
                context(4),
                run("running", 2),
                ExecutionEngineSdkTimerDescriptor {
                    timer_id: "timer:workflow-sdk:sla".to_owned(),
                    armed_at_epoch_seconds: 10,
                    deadline_epoch_seconds: 60,
                    step_index: Some(1),
                },
            )
            .expect("timer plan");
        assert_eq!(timer.path, "/runs/run:workflow-sdk:invoice:1/timers");
        assert_eq!(timer.route_template, EXECUTION_ENGINE_REST_ARM_TIMER_ROUTE);
        assert_eq!(timer.rest_request.body.body.command, "ArmSlaTimer");
        assert_eq!(
            timer.rest_request.body.body.timer_id,
            Some("timer:workflow-sdk:sla".to_owned())
        );
    }

    #[test]
    fn in_process_preview_execute_delegates_through_rest_api_and_adapter_without_network() {
        let client = ExecutionEngineSdkClient::new(config()).expect("valid sdk config");
        let mut rest = ExecutionEngineRestService::default();
        let mut store = WorkflowExecutionMemoryAdapter::default();
        let mut dispatcher = WorkflowExecutionMemoryAdapter::default();
        let retry_policy = WorkflowExecutionMemoryAdapter::default();
        let mut timers = WorkflowExecutionMemoryAdapter::default();

        let start = client
            .start_run(context(10), run("pending", 1), step("pending"))
            .expect("start plan");
        let response = client
            .execute_in_process(
                &mut rest,
                &mut store,
                &mut dispatcher,
                &retry_policy,
                &mut timers,
                start,
            )
            .expect("response");
        assert_eq!(response.status_code, 201);
        assert_eq!(store.run_count(), 1);
        assert!(
            store
                .recorded_actions()
                .iter()
                .any(|action| action.kind == ExecutionAdapterActionKind::CreateRun)
        );

        let dispatch = client
            .dispatch_step(context(11), run("running", 2), step("pending"))
            .expect("dispatch plan");
        let response = client
            .execute_in_process(
                &mut rest,
                &mut store,
                &mut dispatcher,
                &retry_policy,
                &mut timers,
                dispatch,
            )
            .expect("dispatch response");
        assert_eq!(response.status_code, 202);
        assert!(
            dispatcher
                .recorded_actions()
                .iter()
                .any(|action| action.kind == ExecutionAdapterActionKind::DispatchStep)
        );
        assert_eq!(rest.api_cached_response_count(), 2);
    }

    #[test]
    fn idempotent_replay_keeps_stable_request_identity_without_second_side_effect() {
        let client = ExecutionEngineSdkClient::new(config()).expect("valid sdk config");
        let mut rest = ExecutionEngineRestService::default();
        let mut store = WorkflowExecutionMemoryAdapter::default();
        let mut dispatcher = WorkflowExecutionMemoryAdapter::default();
        let retry_policy = WorkflowExecutionMemoryAdapter::default();
        let mut timers = WorkflowExecutionMemoryAdapter::default();
        let plan = client
            .start_run(context(20), run("pending", 1), step("pending"))
            .expect("start plan");
        let first = client
            .execute_in_process(
                &mut rest,
                &mut store,
                &mut dispatcher,
                &retry_policy,
                &mut timers,
                plan.clone(),
            )
            .expect("first response");
        let action_count_after_first = store.recorded_actions().len();
        let second = client
            .execute_in_process(
                &mut rest,
                &mut store,
                &mut dispatcher,
                &retry_policy,
                &mut timers,
                plan,
            )
            .expect("second response");
        assert_eq!(first, second);
        assert_eq!(store.run_count(), 1);
        assert_eq!(store.recorded_actions().len(), action_count_after_first);
    }

    #[test]
    fn invalid_raw_metadata_denies_before_rest_or_adapter_side_effects_without_echo() {
        let client = ExecutionEngineSdkClient::new(config()).expect("valid sdk config");
        let mut unsafe_run = run("pending", 1);
        unsafe_run.input_ref = Some("raw prompt: write an email to customer".to_owned());
        let error = client
            .start_run(context(30), unsafe_run, step("pending"))
            .expect_err("raw content denied");
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-execution-sdk:unsafe-run-metadata"
        );
        assert!(!format!("{error:?}").contains("write an email"));

        let bad_config = ExecutionEngineSdkConfig {
            authorization_evidence_ref: "secret=super-secret-token".to_owned(),
            ..config()
        };
        let error = ExecutionEngineSdkClient::new(bad_config).expect_err("secret denied");
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-execution-sdk:invalid-config-metadata"
        );
        assert!(!format!("{error:?}").contains("super-secret-token"));
    }

    #[test]
    fn sdk_never_enables_automatic_retries_for_state_changing_plans() {
        let client = ExecutionEngineSdkClient::new(config()).expect("valid sdk config");
        let mut plan = client
            .start_run(context(40), run("pending", 1), step("pending"))
            .expect("start plan");
        assert!(!plan.retry_policy.automatic_retries_enabled);
        plan.retry_policy.automatic_retries_enabled = true;
        let mut rest = ExecutionEngineRestService::default();
        let mut store = WorkflowExecutionMemoryAdapter::default();
        let mut dispatcher = WorkflowExecutionMemoryAdapter::default();
        let retry_policy = WorkflowExecutionMemoryAdapter::default();
        let mut timers = WorkflowExecutionMemoryAdapter::default();
        let error = client
            .execute_in_process(
                &mut rest,
                &mut store,
                &mut dispatcher,
                &retry_policy,
                &mut timers,
                plan,
            )
            .expect_err("automatic retries forbidden");
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-execution-sdk:automatic-retry-forbidden"
        );
        assert_eq!(store.run_count(), 0);
    }
}
