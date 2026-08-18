//! Workflow-engine trigger-orchestrator REST boundary foundation.
//!
//! This crate defines a framework-free, source-level REST route facade for the
//! trigger-orchestrator API boundary. It maps HTTP-shaped method/path checks to
//! stable responses, rejects REST/body route drift before API delegation,
//! delegates accepted POST bodies to `oya-workflow-engine-trigger-orchestrator-api`,
//! and returns safe success/problem bodies. It performs no HTTP serving, socket
//! binding, serialization framework work, concrete idempotency store, trigger
//! registry store, policy-store I/O, Cedar evaluation, scheduler execution,
//! webhook serving, HMAC verification, event-bus consumption, run creation,
//! durable trigger storage, network I/O, wall-clock reads, Kubernetes calls,
//! cloud deployment, or tenant workload scheduling.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use workflow_trigger_orchestrator_api::*;

pub const TRIGGER_ORCHESTRATOR_REST_ROUTE: &str = TRIGGER_ORCHESTRATOR_API_ROUTE;
pub const TRIGGER_ORCHESTRATOR_REST_METHOD: TriggerOrchestratorRestMethod =
    TriggerOrchestratorRestMethod::Post;
pub const TRIGGER_ORCHESTRATOR_REST_CONTRACT_REF: &str = "workflow/workflow-engine/contracts/openapi/workflow-engine.yaml#/paths/~1v~12026-05-25~1triggers~1evaluate/post";
pub const TRIGGER_ORCHESTRATOR_REST_SUCCESS_CONTENT_TYPE: &str = "application/json";
pub const TRIGGER_ORCHESTRATOR_REST_PROBLEM_CONTENT_TYPE: &str = "application/problem+json";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TriggerOrchestratorRestMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorRestRequest {
    pub method: TriggerOrchestratorRestMethod, // data_class: PUBLIC
    pub path: String,                          // data_class: PUBLIC
    pub request_id: String,                    // data_class: INTERNAL_ONLY
    pub body: TriggerOrchestratorApiRequest,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorRestResponse {
    pub status_code: u16,                          // data_class: PUBLIC
    pub content_type: String,                      // data_class: PUBLIC
    pub body: TriggerOrchestratorRestResponseBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TriggerOrchestratorRestResponseBody {
    Success(Box<TriggerOrchestratorApiSuccessResponse>),
    Problem(Box<TriggerOrchestratorApiProblemDetails>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorRestError {
    pub reason_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Default)]
pub struct TriggerOrchestratorRestService {
    api: WorkflowTriggerOrchestratorApi,
    api_delegation_count: usize,
}

impl TriggerOrchestratorRestService {
    pub fn new(api: WorkflowTriggerOrchestratorApi) -> Self {
        Self {
            api,
            api_delegation_count: 0,
        }
    }

    pub fn handle(
        &mut self,
        request: TriggerOrchestratorRestRequest,
    ) -> Result<TriggerOrchestratorRestResponse, TriggerOrchestratorRestError> {
        if !is_safe_rest_ref(&request.request_id) {
            return Ok(rest_problem_response(
                400,
                "Bad Request",
                "WORKFLOW_TRIGGER_REST_UNSAFE_REQUEST_ID",
                "workflow-trigger-rest:unsafe-request-id",
                &request.request_id,
            ));
        }
        if request.method != TRIGGER_ORCHESTRATOR_REST_METHOD {
            return Ok(rest_problem_response(
                405,
                "Method Not Allowed",
                "WORKFLOW_TRIGGER_REST_METHOD_NOT_ALLOWED",
                "workflow-trigger-rest:method-not-allowed",
                &request.request_id,
            ));
        }
        if request.path.trim() != request.path
            || request.path != TRIGGER_ORCHESTRATOR_REST_ROUTE
            || contains_unsafe_debug_material(&request.path)
        {
            return Ok(rest_problem_response(
                404,
                "Not Found",
                "WORKFLOW_TRIGGER_REST_ROUTE_NOT_FOUND",
                "workflow-trigger-rest:route-not-found",
                &request.request_id,
            ));
        }
        if request.body.method != TRIGGER_ORCHESTRATOR_API_METHOD {
            return Ok(rest_problem_response(
                400,
                "Bad Request",
                "WORKFLOW_TRIGGER_REST_BODY_METHOD_MISMATCH",
                "workflow-trigger-rest:body-method-mismatch",
                &request.request_id,
            ));
        }
        if request.body.route != TRIGGER_ORCHESTRATOR_REST_ROUTE {
            return Ok(rest_problem_response(
                400,
                "Bad Request",
                "WORKFLOW_TRIGGER_REST_BODY_ROUTE_MISMATCH",
                "workflow-trigger-rest:body-route-mismatch",
                &request.request_id,
            ));
        }

        self.api_delegation_count += 1;
        match self.api.apply_trigger(request.body) {
            Ok(success) => Ok(TriggerOrchestratorRestResponse {
                status_code: success.http_status_code(),
                content_type: TRIGGER_ORCHESTRATOR_REST_SUCCESS_CONTENT_TYPE.to_owned(),
                body: TriggerOrchestratorRestResponseBody::Success(Box::new(success)),
            }),
            Err(error) => Ok(TriggerOrchestratorRestResponse {
                status_code: error.status_code(),
                content_type: TRIGGER_ORCHESTRATOR_REST_PROBLEM_CONTENT_TYPE.to_owned(),
                body: TriggerOrchestratorRestResponseBody::Problem(Box::new(api_problem_response(
                    &error,
                    &request.request_id,
                ))),
            }),
        }
    }

    pub const fn api_delegation_count(&self) -> usize {
        self.api_delegation_count
    }
}

fn api_problem_response(
    error: &TriggerOrchestratorApiError,
    request_id: &str,
) -> TriggerOrchestratorApiProblemDetails {
    let mut problem = error.problem();
    problem
        .evidence_refs
        .push(safe_problem_evidence_ref(request_id));
    problem.evidence_refs.sort();
    problem.evidence_refs.dedup();
    problem
}

fn rest_problem_response(
    status_code: u16,
    title: &str,
    code: &str,
    detail_ref: &str,
    request_id: &str,
) -> TriggerOrchestratorRestResponse {
    TriggerOrchestratorRestResponse {
        status_code,
        content_type: TRIGGER_ORCHESTRATOR_REST_PROBLEM_CONTENT_TYPE.to_owned(),
        body: TriggerOrchestratorRestResponseBody::Problem(Box::new(
            TriggerOrchestratorApiProblemDetails {
                type_ref: format!(
                    "problem:workflow-trigger-rest:{}",
                    code.to_ascii_lowercase().replace('_', "-")
                ),
                status: status_code,
                code: code.to_owned(),
                title: title.to_owned(),
                detail_ref: detail_ref.to_owned(),
                evidence_refs: vec![
                    "workflow-trigger-rest:framework-free-route-facade".to_owned(),
                    safe_problem_evidence_ref(request_id),
                ],
            },
        )),
    }
}

fn safe_problem_evidence_ref(request_id: &str) -> String {
    if is_safe_rest_ref(request_id) {
        format!("problem-instance:{request_id}")
    } else {
        "problem-instance:workflow-trigger-rest:redacted".to_owned()
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
        || lower.contains("raw model")
        || lower.contains("payload")
        || lower.contains("write an email")
        || lower.contains("customer message")
        || lower.contains("model answer")
        || lower.contains("sk-")
        || lower.contains("bearer")
        || lower.contains("authorization:")
        || lower.contains("api_key=")
        || lower.contains("openai_api_key")
        || lower.contains("secret=")
        || lower.contains("private key")
        || lower.contains("-----begin")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_constants_match_trigger_orchestrator_api_contract() {
        assert_eq!(
            TRIGGER_ORCHESTRATOR_REST_ROUTE,
            TRIGGER_ORCHESTRATOR_API_ROUTE
        );
        assert_eq!(
            TRIGGER_ORCHESTRATOR_REST_METHOD,
            TriggerOrchestratorRestMethod::Post
        );
        assert!(TRIGGER_ORCHESTRATOR_REST_CONTRACT_REF.contains("workflow-engine.yaml"));
    }

    #[test]
    fn post_handler_maps_api_accepted_to_202_json_response_without_http_runtime() {
        let mut rest = TriggerOrchestratorRestService::default();
        let response = rest.handle(valid_rest_request("idem:rest:1", "scheduler", "cron"));
        let response = response.expect("rest response");

        assert_eq!(response.status_code, 202);
        assert_eq!(
            response.content_type,
            TRIGGER_ORCHESTRATOR_REST_SUCCESS_CONTENT_TYPE
        );
        let TriggerOrchestratorRestResponseBody::Success(success) = response.body else {
            panic!("expected success body");
        };
        assert_eq!(success.route, TRIGGER_ORCHESTRATOR_API_ROUTE);
        assert_eq!(success.metadata.surface, TRIGGER_ORCHESTRATOR_API_SURFACE);
        assert_eq!(rest.api_delegation_count(), 1);
    }

    #[test]
    fn method_or_path_mismatch_never_delegates_to_api() {
        let mut rest = TriggerOrchestratorRestService::default();
        let mut method = valid_rest_request("idem:rest:method", "scheduler", "cron");
        method.method = TriggerOrchestratorRestMethod::Get;
        let method_response = rest.handle(method).expect("method problem");
        assert_eq!(method_response.status_code, 405);
        assert_eq!(rest.api_delegation_count(), 0);

        let mut path = valid_rest_request("idem:rest:path", "scheduler", "cron");
        path.path = "/v/2026-05-25/triggers/other".to_owned();
        let path_response = rest.handle(path).expect("path problem");
        assert_eq!(path_response.status_code, 404);
        assert_eq!(rest.api_delegation_count(), 0);
    }

    #[test]
    fn rest_body_route_mismatch_returns_problem_without_api_delegation() {
        let mut rest = TriggerOrchestratorRestService::default();
        let mut request = valid_rest_request("idem:rest:body-route", "scheduler", "cron");
        request.body.route = "/v/2026-05-25/triggers/other".to_owned();

        let response = rest.handle(request).expect("body route mismatch");

        assert_eq!(response.status_code, 400);
        assert_eq!(rest.api_delegation_count(), 0);
        let rendered = format!("{response:?}");
        assert!(rendered.contains("WORKFLOW_TRIGGER_REST_BODY_ROUTE_MISMATCH"));
    }

    #[test]
    fn api_errors_are_returned_as_problem_details_without_raw_echo() {
        let mut rest = TriggerOrchestratorRestService::default();
        let mut request = valid_rest_request("idem:rest:unsafe", "scheduler", "cron");
        request.body.body.correlation_ref =
            "corr:raw prompt Authorization: Bearer sk-test payload".to_owned();

        let response = rest.handle(request).expect("api problem");

        assert_eq!(response.status_code, 400);
        assert_eq!(
            response.content_type,
            TRIGGER_ORCHESTRATOR_REST_PROBLEM_CONTENT_TYPE
        );
        let rendered = format!("{response:?}");
        assert!(rendered.contains(TriggerOrchestratorApiErrorCode::UnsafeMetadata.as_str()));
        assert!(!rendered.contains("raw prompt"));
        assert!(!rendered.contains("Authorization"));
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("payload"));
        assert_eq!(rest.api_delegation_count(), 1);
    }

    #[test]
    fn idempotent_replay_flows_through_rest_service_cache() {
        let mut rest = TriggerOrchestratorRestService::default();
        let request = valid_rest_request("idem:rest:replay", "scheduler", "cron");
        let first = rest.handle(request.clone()).expect("first");
        let second = rest.handle(request).expect("second");

        assert_eq!(first, second);
        assert_eq!(rest.api_delegation_count(), 2);

        let mut drifted = valid_rest_request("idem:rest:replay", "scheduler", "cron");
        drifted.body.body.workflow_spec_id = "workflow:other".to_owned();
        let conflict = rest.handle(drifted).expect("conflict problem");
        assert_eq!(conflict.status_code, 409);
        assert!(
            format!("{conflict:?}")
                .contains(TriggerOrchestratorApiErrorCode::IdempotencyKeyReused.as_str())
        );
    }

    #[test]
    fn webhook_event_and_deferred_responses_remain_metadata_only() {
        let mut rest = TriggerOrchestratorRestService::default();

        let mut paused = valid_rest_request("idem:rest:paused", "scheduler", "cron");
        paused.body.body.schedule.as_mut().unwrap().paused = true;
        paused.body.body.schedule.as_mut().unwrap().pause_reason_ref =
            Some("pause:maintenance".to_owned());
        let deferred = rest.handle(paused).expect("deferred");
        let TriggerOrchestratorRestResponseBody::Success(deferred_body) = deferred.body else {
            panic!("expected deferred success");
        };
        assert_eq!(deferred_body.trigger.usecase_status, "deferred");
        assert!(!deferred_body.trigger.dispatch_required);

        let mut event = valid_rest_request("idem:rest:event", "sibling-event-bus", "event-bus");
        event.body.body.schedule = None;
        event.body.body.event = Some(event_valid());
        event.body.body.scheduler_evidence_ref = None;
        event.body.body.event_contract_ref = Some("event-contract:cloudevents-v1".to_owned());
        event.body.body.replay_mode = true;
        let suppressed = rest.handle(event).expect("event suppressed");
        let TriggerOrchestratorRestResponseBody::Success(suppressed_body) = suppressed.body else {
            panic!("expected suppressed success");
        };
        assert_eq!(suppressed_body.trigger.usecase_status, "suppressed");
        assert!(!format!("{suppressed_body:?}").contains("payload"));

        let mut webhook = valid_rest_request("idem:rest:webhook", "studio-webhook", "webhook");
        webhook.body.body.schedule = None;
        webhook.body.body.webhook = Some(webhook_valid());
        webhook.body.body.scheduler_evidence_ref = None;
        webhook.body.body.webhook_auth_evidence_ref =
            Some("webhook-auth:hmac-nonce-bound".to_owned());
        let accepted = rest.handle(webhook).expect("webhook accepted");
        let TriggerOrchestratorRestResponseBody::Success(accepted_body) = accepted.body else {
            panic!("expected webhook success");
        };
        assert!(
            accepted_body
                .non_claim_refs
                .contains(&"no-webhook-server".to_owned())
        );
    }

    #[test]
    fn unsafe_rest_request_id_is_redacted_and_not_delegated() {
        let mut rest =
            TriggerOrchestratorRestService::new(WorkflowTriggerOrchestratorApi::default());
        let mut request = valid_rest_request("idem:rest:unsafe-request-id", "scheduler", "cron");
        request.request_id = "request:raw prompt bearer sk-test payload".to_owned();

        let response = rest.handle(request).expect("unsafe request id problem");

        assert_eq!(response.status_code, 400);
        assert_eq!(rest.api_delegation_count(), 0);
        let rendered = format!("{response:?}");
        assert!(rendered.contains("problem-instance:workflow-trigger-rest:redacted"));
        assert!(!rendered.contains("raw prompt"));
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("payload"));
    }

    fn valid_rest_request(
        idempotency_key: &str,
        source: &str,
        kind: &str,
    ) -> TriggerOrchestratorRestRequest {
        TriggerOrchestratorRestRequest {
            method: TriggerOrchestratorRestMethod::Post,
            path: TRIGGER_ORCHESTRATOR_REST_ROUTE.to_owned(),
            request_id: format!("request:trigger-rest:{idempotency_key}"),
            body: authorized_request(idempotency_key, source, kind),
        }
    }

    fn authorized_request(
        idempotency_key: &str,
        source: &str,
        kind: &str,
    ) -> TriggerOrchestratorApiRequest {
        TriggerOrchestratorApiRequest {
            boundary: TriggerOrchestratorApiBoundaryContext {
                request_id: format!("request:trigger-api:{idempotency_key}"),
                tenant_id: "ten_foundry".to_owned(),
                idempotency_key: idempotency_key.to_owned(),
                trace_context_ref: "trace:trigger-api".to_owned(),
                oyatie_version: TRIGGER_ORCHESTRATOR_API_DECLARED_VERSION.to_owned(),
            },
            principal: TriggerOrchestratorApiPrincipal {
                tenant_id: "ten_foundry".to_owned(),
                principal_id: "principal:workflow-operator".to_owned(),
            },
            authorization: TriggerOrchestratorApiAuthorization {
                tenant_id: "ten_foundry".to_owned(),
                principal_id: "principal:workflow-operator".to_owned(),
                decision_id: "policy-decision:allow-trigger".to_owned(),
                evidence_ref: "policy-evidence:cedar-allow".to_owned(),
                policy_bundle_ref: "policy-bundle:trigger-v1".to_owned(),
                allowed_surfaces: vec![TRIGGER_ORCHESTRATOR_API_SURFACE.to_owned()],
            },
            method: TRIGGER_ORCHESTRATOR_API_METHOD.to_owned(),
            route: TRIGGER_ORCHESTRATOR_API_ROUTE.to_owned(),
            body: TriggerOrchestratorApiTriggerBody {
                source: source.to_owned(),
                trigger_kind: kind.to_owned(),
                trigger_id: "trigger:daily-invoice".to_owned(),
                workflow_spec_id: "workflow:invoice-approval".to_owned(),
                version_sha: "sha:abc123".to_owned(),
                active_cell_id: "cell:use1-a".to_owned(),
                trigger_lineage_ref: "lineage:trigger-parent".to_owned(),
                run_idempotency_key: "idem:trigger-run".to_owned(),
                authorization_surface_ref: "authz-surface:trigger-admission".to_owned(),
                source_evidence_ref: "source-evidence:trigger-admission".to_owned(),
                scheduler_evidence_ref: Some("scheduler:durable-clock-window".to_owned()),
                webhook_auth_evidence_ref: None,
                event_contract_ref: None,
                replay_epoch_ref: "replay-epoch:2026-05-25T000000Z".to_owned(),
                audit_chain_ref: "audit-chain:trigger-api".to_owned(),
                correlation_ref: "corr:trigger-api".to_owned(),
                idempotency_scope_ref: "idem-scope:tenant-trigger".to_owned(),
                dry_run_reason_ref: None,
                replay_mode: false,
                dry_run: false,
                schedule: Some(schedule_due()),
                webhook: None,
                event: None,
                evidence_refs: vec!["evidence:rest-unit-test".to_owned()],
            },
        }
    }

    fn schedule_due() -> TriggerOrchestratorApiScheduleDto {
        TriggerOrchestratorApiScheduleDto {
            cron_expr_ref: "cron:every-hour".to_owned(),
            timezone_ref: "tz:America-New_York".to_owned(),
            due_epoch_seconds: 1_750_000_000,
            observed_epoch_seconds: 1_750_000_008,
            catchup_window_seconds: 10,
            overlap_policy: "buffer-one".to_owned(),
            paused: false,
            pause_reason_ref: None,
            last_fired_epoch_seconds: Some(1_749_996_400),
        }
    }

    fn webhook_valid() -> TriggerOrchestratorApiWebhookDto {
        TriggerOrchestratorApiWebhookDto {
            endpoint_ref: "endpoint:webhook-invoice".to_owned(),
            signature_ref: "signature:webhook-headers".to_owned(),
            nonce_ref: "nonce:webhook-001".to_owned(),
            hmac_key_ref: "hmac-key:webhook-signing".to_owned(),
            received_epoch_seconds: 1_750_000_001,
            expires_epoch_seconds: 1_750_000_061,
        }
    }

    fn event_valid() -> TriggerOrchestratorApiEventDto {
        TriggerOrchestratorApiEventDto {
            event_id: "event:invoice-approved-001".to_owned(),
            source: "https://events.oyatie.example/workflow".to_owned(),
            event_type: "com.oyatie.workflow.invoice_approved".to_owned(),
            specversion: TRIGGER_ORCHESTRATOR_CLOUDEVENTS_SPECVERSION.to_owned(),
            subject_ref: Some("subject:invoice-123".to_owned()),
            event_time_ref: Some("time:2026-05-25T00:00:00Z".to_owned()),
            correlation_id: "corr:invoice-123".to_owned(),
            idempotency_key: "idem:event-001".to_owned(),
        }
    }
}
