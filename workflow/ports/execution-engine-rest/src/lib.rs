//! Workflow-engine execution-engine REST boundary foundation.
//!
//! This crate defines a framework-free, source-level REST route facade for the
//! execution-engine API boundary. It maps HTTP method/path checks to stable
//! HTTP-shaped responses, extracts route parameters for the currently supported
//! execution commands, delegates valid bodies to `oya-workflow-engine-execution-engine-api`,
//! and returns safe success/problem bodies. It performs no HTTP serving, socket
//! binding, serialization framework work, concrete storage, network I/O,
//! durable idempotency storage, durable audit-chain emission, queue processing,
//! signing, wall-clock reads, or runtime execution.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use workflow_execution_engine_api::*;

pub const EXECUTION_ENGINE_REST_START_RUN_ROUTE: &str = "/runs";
pub const EXECUTION_ENGINE_REST_DISPATCH_STEP_ROUTE: &str =
    "/runs/{run_id}/steps/{step_index}/dispatch";
pub const EXECUTION_ENGINE_REST_SCHEDULE_RETRY_ROUTE: &str =
    "/runs/{run_id}/steps/{step_index}/retry";
pub const EXECUTION_ENGINE_REST_ARM_TIMER_ROUTE: &str = "/runs/{run_id}/timers";
pub const EXECUTION_ENGINE_REST_CONTRACT_REF: &str =
    "workflow/workflow-engine/contracts/openapi/workflow-engine.yaml#/paths";
pub const EXECUTION_ENGINE_REST_START_RUN_METHOD: ExecutionEngineRestMethod =
    ExecutionEngineRestMethod::Post;
pub const EXECUTION_ENGINE_REST_PROBLEM_CONTENT_TYPE: &str = "application/problem+json";
pub const EXECUTION_ENGINE_REST_JSON_CONTENT_TYPE: &str = "application/json";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExecutionEngineRestMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExecutionEngineRestOperation {
    StartRun,
    DispatchStep,
    ScheduleRetry,
    ArmSlaTimer,
}

impl ExecutionEngineRestOperation {
    pub const fn route_template(self) -> &'static str {
        match self {
            Self::StartRun => EXECUTION_ENGINE_REST_START_RUN_ROUTE,
            Self::DispatchStep => EXECUTION_ENGINE_REST_DISPATCH_STEP_ROUTE,
            Self::ScheduleRetry => EXECUTION_ENGINE_REST_SCHEDULE_RETRY_ROUTE,
            Self::ArmSlaTimer => EXECUTION_ENGINE_REST_ARM_TIMER_ROUTE,
        }
    }

    pub const fn expected_command(self) -> &'static str {
        match self {
            Self::StartRun => "StartRun",
            Self::DispatchStep => "DispatchStep",
            Self::ScheduleRetry => "ScheduleRetry",
            Self::ArmSlaTimer => "ArmSlaTimer",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineRestRouteMatch {
    pub operation: ExecutionEngineRestOperation, // data_class: PUBLIC
    pub run_id: Option<String>,                  // data_class: INTERNAL_ONLY
    pub step_index: Option<u32>,                 // data_class: INTERNAL_ONLY
    pub route_template: String,                  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineRestRequest {
    pub method: ExecutionEngineRestMethod, // data_class: PUBLIC
    pub path: String,                      // data_class: PUBLIC
    pub request_id: String,                // data_class: INTERNAL_ONLY
    pub body: ExecutionEngineApiRequest,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineRestResponse {
    pub status_code: u16,              // data_class: PUBLIC
    pub content_type: String,          // data_class: PUBLIC
    pub body: ExecutionEngineRestBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionEngineRestBody {
    Success(Box<ExecutionEngineApiSuccessResponse>),
    Problem(Box<ExecutionEngineApiProblem>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineRestError {
    pub reason_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Default)]
pub struct ExecutionEngineRestService {
    api: ExecutionEngineApi,
}

impl ExecutionEngineRestService {
    pub fn new(api: ExecutionEngineApi) -> Self {
        Self { api }
    }

    pub fn handle<S, D, R, T>(
        &mut self,
        store: &mut S,
        dispatcher: &mut D,
        retry_policy: &R,
        timers: &mut T,
        request: ExecutionEngineRestRequest,
    ) -> Result<ExecutionEngineRestResponse, ExecutionEngineRestError>
    where
        S: WorkflowRunStore,
        D: StepDispatcher,
        R: RetryPolicyEvaluator,
        T: SlaTimerStore,
    {
        if request.method != ExecutionEngineRestMethod::Post {
            return Ok(rest_problem_response(
                405,
                "Method Not Allowed",
                "WORKFLOW_EXECUTION_REST_METHOD_NOT_ALLOWED",
                "workflow-execution-rest:method-not-allowed",
                &request.request_id,
            ));
        }

        let route = match match_route(&request.path) {
            Some(route) => route,
            None => {
                return Ok(rest_problem_response(
                    404,
                    "Not Found",
                    "WORKFLOW_EXECUTION_REST_ROUTE_NOT_FOUND",
                    "workflow-execution-rest:route-not-found",
                    &request.request_id,
                ));
            }
        };

        let api_request = bind_route_to_api_request(route, request.body)?;
        match self
            .api
            .apply_command(store, dispatcher, retry_policy, timers, api_request)
        {
            Ok(success) => Ok(ExecutionEngineRestResponse {
                status_code: success.http_status_code(),
                content_type: EXECUTION_ENGINE_REST_JSON_CONTENT_TYPE.to_owned(),
                body: ExecutionEngineRestBody::Success(Box::new(success)),
            }),
            Err(error) => Ok(ExecutionEngineRestResponse {
                status_code: error.status_code(),
                content_type: EXECUTION_ENGINE_REST_PROBLEM_CONTENT_TYPE.to_owned(),
                body: ExecutionEngineRestBody::Problem(Box::new(
                    ExecutionEngineApiProblem::from_error(
                        &error,
                        &safe_problem_instance(&request.request_id),
                    ),
                )),
            }),
        }
    }

    pub fn api_cached_response_count(&self) -> usize {
        self.api.cached_response_count()
    }
}

pub fn match_route(path: &str) -> Option<ExecutionEngineRestRouteMatch> {
    let trimmed = path.trim();
    if trimmed != path || contains_unsafe_debug_material(path) {
        return None;
    }
    if path == EXECUTION_ENGINE_REST_START_RUN_ROUTE {
        return Some(route_match(
            ExecutionEngineRestOperation::StartRun,
            None,
            None,
        ));
    }
    let parts: Vec<&str> = path.trim_matches('/').split('/').collect();
    match parts.as_slice() {
        ["runs", run_id, "steps", step_index, "dispatch"] => Some(route_match(
            ExecutionEngineRestOperation::DispatchStep,
            Some((*run_id).to_owned()),
            step_index.parse::<u32>().ok(),
        )),
        ["runs", run_id, "steps", step_index, "retry"] => Some(route_match(
            ExecutionEngineRestOperation::ScheduleRetry,
            Some((*run_id).to_owned()),
            step_index.parse::<u32>().ok(),
        )),
        ["runs", run_id, "timers"] => Some(route_match(
            ExecutionEngineRestOperation::ArmSlaTimer,
            Some((*run_id).to_owned()),
            None,
        )),
        _ => None,
    }
}

fn route_match(
    operation: ExecutionEngineRestOperation,
    run_id: Option<String>,
    step_index: Option<u32>,
) -> ExecutionEngineRestRouteMatch {
    ExecutionEngineRestRouteMatch {
        operation,
        run_id,
        step_index,
        route_template: operation.route_template().to_owned(),
    }
}

fn bind_route_to_api_request(
    route: ExecutionEngineRestRouteMatch,
    mut request: ExecutionEngineApiRequest,
) -> Result<ExecutionEngineApiRequest, ExecutionEngineRestError> {
    if route.operation.expected_command() != request.body.command {
        return Err(ExecutionEngineRestError {
            reason_ref: "workflow-execution-rest:command-route-mismatch".to_owned(),
        });
    }
    request.route_run_id = route.run_id;
    request.route_step_index = route.step_index;
    Ok(request)
}

fn rest_problem_response(
    status_code: u16,
    title: &str,
    code: &str,
    detail_ref: &str,
    request_id: &str,
) -> ExecutionEngineRestResponse {
    ExecutionEngineRestResponse {
        status_code,
        content_type: EXECUTION_ENGINE_REST_PROBLEM_CONTENT_TYPE.to_owned(),
        body: ExecutionEngineRestBody::Problem(Box::new(ExecutionEngineApiProblem {
            type_uri: format!(
                "https://oyatie.com/problems/workflow-engine/execution-engine/rest/{}",
                code.to_ascii_lowercase().replace('_', "-")
            ),
            title: title.to_owned(),
            status: status_code,
            code: code.to_owned(),
            detail_ref: detail_ref.to_owned(),
            instance: safe_problem_instance(request_id),
        })),
    }
}

fn safe_problem_instance(request_id: &str) -> String {
    if is_safe_rest_ref(request_id) {
        request_id.to_owned()
    } else {
        "problem-instance:workflow-execution-rest:redacted".to_owned()
    }
}

fn is_safe_rest_ref(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && value == trimmed
        && value.contains(':')
        && !value.chars().any(char::is_whitespace)
        && !contains_unsafe_debug_material(value)
}

fn contains_unsafe_debug_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw output")
        || lower.contains("payload")
        || lower.contains("write an email")
        || lower.contains("customer message")
        || lower.contains("model answer")
        || lower.contains("sk-")
        || lower.contains("bearer")
        || lower.contains("authorization:")
        || lower.contains("api_key=")
        || lower.contains("secret=")
        || lower.contains("private key")
        || lower.contains("-----begin")
}

#[cfg(test)]
mod tests {
    use super::*;
    use workflow_execution_engine_adapter::WorkflowExecutionMemoryAdapterBundle;

    fn start_request(seq: u64) -> ExecutionEngineApiRequest {
        ExecutionEngineApiRequest {
            boundary: ExecutionEngineApiBoundaryContext {
                request_id: format!("req:workflow-execution-rest:{seq}"),
                tenant_id: "ten_a".to_owned(),
                idempotency_key: format!("idem:workflow-execution-rest:{seq}"),
                trace_context_ref: format!("trace:workflow-execution-rest:{seq}"),
                oyatie_version: EXECUTION_ENGINE_API_DECLARED_VERSION.to_owned(),
            },
            principal: ExecutionEngineApiPrincipal {
                tenant_id: "ten_a".to_owned(),
                principal_id: "principal:workflow-operator:1".to_owned(),
            },
            authorization: ExecutionEngineApiAuthorization {
                tenant_id: "ten_a".to_owned(),
                principal_id: "principal:workflow-operator:1".to_owned(),
                decision_id: "authz:workflow-execution-rest:allow".to_owned(),
                evidence_ref: "cedar://workflow/execution/rest/allow".to_owned(),
                allowed_surfaces: vec![EXECUTION_ENGINE_API_SURFACE.to_owned()],
            },
            expected_run_version: 1,
            expected_spec_id: "workflow-spec:invoice-approval".to_owned(),
            expected_version_sha: "sha256:spec-v1".to_owned(),
            expected_cell_id: "cell:use1:a".to_owned(),
            spec_integrity_ref: "spec-integrity:workflow:v1".to_owned(),
            replay_epoch_ref: "replay-epoch:rest:1".to_owned(),
            scheduler_epoch_ref: "scheduler-epoch:rest:1".to_owned(),
            route_run_id: None,
            route_step_index: None,
            body: ExecutionEngineApiCommandBody {
                command: "StartRun".to_owned(),
                run_id: "run:workflow:rest:1".to_owned(),
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
                evidence_refs: vec!["workflow-execution:rest-request".to_owned()],
            },
        }
    }

    fn run_current() -> WorkflowRun {
        let mut run = WorkflowRun::new(
            "ten_a",
            "run:workflow:rest:1",
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
        let mut request = start_request(seq);
        request.body.command = "ArmSlaTimer".to_owned();
        request.body.current_run_status = "running".to_owned();
        request.body.step_id = None;
        request.body.step_index = None;
        request.body.step_status = None;
        request.body.timer_id = Some("timer:workflow:rest:1".to_owned());
        request.body.armed_at_epoch_seconds = Some(100);
        request.body.deadline_epoch_seconds = Some(200);
        request
    }

    fn rest_request(path: &str, body: ExecutionEngineApiRequest) -> ExecutionEngineRestRequest {
        ExecutionEngineRestRequest {
            method: ExecutionEngineRestMethod::Post,
            path: path.to_owned(),
            request_id: "problem-instance:workflow-execution-rest:1".to_owned(),
            body,
        }
    }

    fn handle(
        rest: &mut ExecutionEngineRestService,
        bundle: &mut WorkflowExecutionMemoryAdapterBundle,
        request: ExecutionEngineRestRequest,
    ) -> ExecutionEngineRestResponse {
        rest.handle(
            &mut bundle.store,
            &mut bundle.dispatcher,
            &bundle.retry_policy,
            &mut bundle.timers,
            request,
        )
        .unwrap()
    }

    #[test]
    fn route_constants_match_execution_engine_openapi_contract() {
        assert_eq!(EXECUTION_ENGINE_REST_START_RUN_ROUTE, "/runs");
        assert_eq!(
            EXECUTION_ENGINE_REST_DISPATCH_STEP_ROUTE,
            "/runs/{run_id}/steps/{step_index}/dispatch"
        );
        assert_eq!(
            EXECUTION_ENGINE_REST_START_RUN_METHOD,
            ExecutionEngineRestMethod::Post
        );
        assert_eq!(
            match_route("/runs/run:workflow:rest:1/steps/0/dispatch")
                .unwrap()
                .operation,
            ExecutionEngineRestOperation::DispatchStep
        );
    }

    #[test]
    fn post_runs_maps_to_api_created_response_without_http_runtime() {
        let mut rest = ExecutionEngineRestService::default();
        let mut bundle = WorkflowExecutionMemoryAdapterBundle::default();

        let response = handle(
            &mut rest,
            &mut bundle,
            rest_request(EXECUTION_ENGINE_REST_START_RUN_ROUTE, start_request(1)),
        );

        assert_eq!(response.status_code, 201);
        assert_eq!(
            response.content_type,
            EXECUTION_ENGINE_REST_JSON_CONTENT_TYPE
        );
        assert!(matches!(response.body, ExecutionEngineRestBody::Success(_)));
        assert_eq!(bundle.store.run_count(), 1);
        assert_eq!(bundle.dispatcher.recorded_actions().len(), 1);
    }

    #[test]
    fn dynamic_dispatch_retry_and_timer_routes_bind_path_params_before_api() {
        let mut rest = ExecutionEngineRestService::default();
        let mut bundle = WorkflowExecutionMemoryAdapterBundle::default();
        bundle.store.create_run(run_current()).unwrap();

        let dispatch = handle(
            &mut rest,
            &mut bundle,
            rest_request(
                "/runs/run:workflow:rest:1/steps/0/dispatch",
                dispatch_request(2),
            ),
        );
        assert_eq!(dispatch.status_code, 202);
        assert!(matches!(dispatch.body, ExecutionEngineRestBody::Success(_)));

        let mut retry_rest = ExecutionEngineRestService::default();
        let mut retry_bundle = WorkflowExecutionMemoryAdapterBundle::default();
        retry_bundle.store.create_run(run_current()).unwrap();
        let retry = handle(
            &mut retry_rest,
            &mut retry_bundle,
            rest_request("/runs/run:workflow:rest:1/steps/0/retry", retry_request(3)),
        );
        assert_eq!(retry.status_code, 202);

        let mut timer_rest = ExecutionEngineRestService::default();
        let mut timer_bundle = WorkflowExecutionMemoryAdapterBundle::default();
        timer_bundle.store.create_run(run_current()).unwrap();
        let timer = handle(
            &mut timer_rest,
            &mut timer_bundle,
            rest_request("/runs/run:workflow:rest:1/timers", timer_request(4)),
        );
        assert_eq!(timer.status_code, 202);
        assert_eq!(timer_bundle.timers.timer_count(), 1);
    }

    #[test]
    fn method_or_path_mismatch_never_calls_api_or_ports() {
        let mut rest = ExecutionEngineRestService::default();
        let mut bundle = WorkflowExecutionMemoryAdapterBundle::default();
        let mut request = rest_request("/runs/run:workflow:rest:1/unknown", dispatch_request(2));
        request.method = ExecutionEngineRestMethod::Get;

        let response = handle(&mut rest, &mut bundle, request);

        assert_eq!(response.status_code, 405);
        assert_eq!(rest.api_cached_response_count(), 0);
        assert_eq!(bundle.store.run_count(), 0);
        assert_eq!(bundle.dispatcher.recorded_actions().len(), 0);

        let response = handle(
            &mut rest,
            &mut bundle,
            rest_request("/runs/run:workflow:rest:1/unknown", dispatch_request(3)),
        );
        assert_eq!(response.status_code, 404);
        assert_eq!(bundle.store.run_count(), 0);
    }

    #[test]
    fn route_command_mismatch_returns_explicit_rest_error_without_api_side_effects() {
        let mut rest = ExecutionEngineRestService::default();
        let mut bundle = WorkflowExecutionMemoryAdapterBundle::default();

        let error = rest
            .handle(
                &mut bundle.store,
                &mut bundle.dispatcher,
                &bundle.retry_policy,
                &mut bundle.timers,
                rest_request("/runs/run:workflow:rest:1/timers", dispatch_request(2)),
            )
            .unwrap_err();

        assert_eq!(
            error.reason_ref,
            "workflow-execution-rest:command-route-mismatch"
        );
        assert_eq!(rest.api_cached_response_count(), 0);
        assert_eq!(bundle.store.run_count(), 0);
    }

    #[test]
    fn api_errors_are_returned_as_problem_details_without_raw_echo() {
        let mut rest = ExecutionEngineRestService::default();
        let mut bundle = WorkflowExecutionMemoryAdapterBundle::default();
        let mut request = start_request(1);
        request.boundary.trace_context_ref = "Authorization: Bearer sk-test raw prompt".to_owned();

        let response = handle(
            &mut rest,
            &mut bundle,
            rest_request(EXECUTION_ENGINE_REST_START_RUN_ROUTE, request),
        );

        assert_eq!(response.status_code, 400);
        assert_eq!(
            response.content_type,
            EXECUTION_ENGINE_REST_PROBLEM_CONTENT_TYPE
        );
        let rendered = format!("{response:?}").to_ascii_lowercase();
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("raw prompt"));
        assert_eq!(bundle.store.run_count(), 0);
    }

    #[test]
    fn rest_service_type_exists_for_api_composition() {
        let rest = ExecutionEngineRestService::default();
        assert_eq!(rest.api_cached_response_count(), 0);
    }
}
