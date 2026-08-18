//! Workflow-engine event-bus REST boundary foundation.
//!
//! This crate defines a framework-free, source-level REST route facade for the
//! event-bus API boundary. It validates HTTP-shaped method/path/request identity,
//! rejects REST/body route drift before API delegation, delegates accepted
//! publish and delivery-evaluation requests to
//! `oya-workflow-engine-event-bus-api`, and returns safe success/problem bodies.
//! It performs no HTTP serving, socket binding, serialization-framework work,
//! concrete storage, broker connection, topic creation, network I/O, durable
//! idempotency storage, durable outbox/inbox writes, consumer group coordination,
//! offset commits, signing, Kubernetes calls, cloud deployment, or tenant
//! workload scheduling.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use workflow_event_bus_api::*;

pub const WORKFLOW_EVENT_BUS_REST_PUBLISH_ROUTE: &str = WORKFLOW_EVENT_BUS_API_PUBLISH_ROUTE;
pub const WORKFLOW_EVENT_BUS_REST_DELIVERY_ROUTE: &str = WORKFLOW_EVENT_BUS_API_DELIVERY_ROUTE;
pub const WORKFLOW_EVENT_BUS_REST_METHOD: WorkflowEventBusRestMethod =
    WorkflowEventBusRestMethod::Post;
pub const WORKFLOW_EVENT_BUS_REST_CONTRACT_REF: &str =
    "workflow/workflow-engine/contracts/openapi/workflow-engine.yaml#/paths";
pub const WORKFLOW_EVENT_BUS_REST_SUCCESS_CONTENT_TYPE: &str = "application/json";
pub const WORKFLOW_EVENT_BUS_REST_PROBLEM_CONTENT_TYPE: &str = "application/problem+json";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowEventBusRestMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowEventBusRestOperation {
    Publish,
    EvaluateDelivery,
}

impl WorkflowEventBusRestOperation {
    pub const fn route(self) -> &'static str {
        match self {
            Self::Publish => WORKFLOW_EVENT_BUS_REST_PUBLISH_ROUTE,
            Self::EvaluateDelivery => WORKFLOW_EVENT_BUS_REST_DELIVERY_ROUTE,
        }
    }

    pub const fn expected_body_kind(self) -> WorkflowEventBusRestBodyKind {
        match self {
            Self::Publish => WorkflowEventBusRestBodyKind::Publish,
            Self::EvaluateDelivery => WorkflowEventBusRestBodyKind::Delivery,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowEventBusRestBodyKind {
    Publish,
    Delivery,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowEventBusRestRequestBody {
    Publish(Box<WorkflowEventBusApiPublishRequest>),
    Delivery(Box<WorkflowEventBusApiDeliveryRequest>),
}

impl WorkflowEventBusRestRequestBody {
    pub const fn kind(&self) -> WorkflowEventBusRestBodyKind {
        match self {
            Self::Publish(_) => WorkflowEventBusRestBodyKind::Publish,
            Self::Delivery(_) => WorkflowEventBusRestBodyKind::Delivery,
        }
    }

    pub fn api_route(&self) -> &str {
        match self {
            Self::Publish(request) => &request.route,
            Self::Delivery(request) => &request.route,
        }
    }

    pub fn api_method(&self) -> &str {
        match self {
            Self::Publish(request) => &request.method,
            Self::Delivery(request) => &request.method,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusRestRequest {
    pub method: WorkflowEventBusRestMethod, // data_class: PUBLIC
    pub path: String,                       // data_class: PUBLIC
    pub request_id: String,                 // data_class: INTERNAL_ONLY
    pub body: WorkflowEventBusRestRequestBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusRestResponse {
    pub status_code: u16,                       // data_class: PUBLIC
    pub content_type: String,                   // data_class: PUBLIC
    pub body: WorkflowEventBusRestResponseBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowEventBusRestResponseBody {
    Success(Box<WorkflowEventBusApiSuccessResponse>),
    Problem(Box<WorkflowEventBusApiProblemDetails>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusRestError {
    pub reason_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Default)]
pub struct WorkflowEventBusRestService {
    api: WorkflowEventBusApi,
    api_delegation_count: usize,
}

impl WorkflowEventBusRestService {
    pub fn new(api: WorkflowEventBusApi) -> Self {
        Self {
            api,
            api_delegation_count: 0,
        }
    }

    pub fn handle(
        &mut self,
        request: WorkflowEventBusRestRequest,
    ) -> Result<WorkflowEventBusRestResponse, WorkflowEventBusRestError> {
        if !is_safe_rest_ref(&request.request_id) {
            return Ok(rest_problem_response(
                400,
                "Bad Request",
                "WORKFLOW_EVENT_BUS_REST_UNSAFE_REQUEST_ID",
                "workflow-event-bus-rest:unsafe-request-id",
                &request.request_id,
            ));
        }
        if request.method != WORKFLOW_EVENT_BUS_REST_METHOD {
            return Ok(rest_problem_response(
                405,
                "Method Not Allowed",
                "WORKFLOW_EVENT_BUS_REST_METHOD_NOT_ALLOWED",
                "workflow-event-bus-rest:method-not-allowed",
                &request.request_id,
            ));
        }

        let Some(operation) = match_route(&request.path) else {
            return Ok(rest_problem_response(
                404,
                "Not Found",
                "WORKFLOW_EVENT_BUS_REST_ROUTE_NOT_FOUND",
                "workflow-event-bus-rest:route-not-found",
                &request.request_id,
            ));
        };
        if operation.expected_body_kind() != request.body.kind() {
            return Ok(rest_problem_response(
                400,
                "Bad Request",
                "WORKFLOW_EVENT_BUS_REST_BODY_KIND_MISMATCH",
                "workflow-event-bus-rest:body-kind-mismatch",
                &request.request_id,
            ));
        }
        if request.body.api_method() != WORKFLOW_EVENT_BUS_API_METHOD {
            return Ok(rest_problem_response(
                400,
                "Bad Request",
                "WORKFLOW_EVENT_BUS_REST_BODY_METHOD_MISMATCH",
                "workflow-event-bus-rest:body-method-mismatch",
                &request.request_id,
            ));
        }
        if request.body.api_route() != operation.route() {
            return Ok(rest_problem_response(
                400,
                "Bad Request",
                "WORKFLOW_EVENT_BUS_REST_BODY_ROUTE_MISMATCH",
                "workflow-event-bus-rest:body-route-mismatch",
                &request.request_id,
            ));
        }

        self.api_delegation_count += 1;
        match request.body {
            WorkflowEventBusRestRequestBody::Publish(api_request) => {
                Self::map_api_result(self.api.publish_event(*api_request), &request.request_id)
            }
            WorkflowEventBusRestRequestBody::Delivery(api_request) => Self::map_api_result(
                self.api.evaluate_delivery(*api_request),
                &request.request_id,
            ),
        }
    }

    pub const fn api_delegation_count(&self) -> usize {
        self.api_delegation_count
    }

    fn map_api_result(
        result: Result<WorkflowEventBusApiSuccessResponse, WorkflowEventBusApiError>,
        request_id: &str,
    ) -> Result<WorkflowEventBusRestResponse, WorkflowEventBusRestError> {
        match result {
            Ok(success) => Ok(WorkflowEventBusRestResponse {
                status_code: success.http_status_code(),
                content_type: WORKFLOW_EVENT_BUS_REST_SUCCESS_CONTENT_TYPE.to_owned(),
                body: WorkflowEventBusRestResponseBody::Success(Box::new(success)),
            }),
            Err(error) => Ok(WorkflowEventBusRestResponse {
                status_code: error.status_code(),
                content_type: WORKFLOW_EVENT_BUS_REST_PROBLEM_CONTENT_TYPE.to_owned(),
                body: WorkflowEventBusRestResponseBody::Problem(Box::new(api_problem_response(
                    &error, request_id,
                ))),
            }),
        }
    }
}

pub fn match_route(path: &str) -> Option<WorkflowEventBusRestOperation> {
    let trimmed = path.trim();
    if trimmed != path || contains_unsafe_debug_material(path) {
        return None;
    }
    match path {
        WORKFLOW_EVENT_BUS_REST_PUBLISH_ROUTE => Some(WorkflowEventBusRestOperation::Publish),
        WORKFLOW_EVENT_BUS_REST_DELIVERY_ROUTE => {
            Some(WorkflowEventBusRestOperation::EvaluateDelivery)
        }
        _ => None,
    }
}

fn api_problem_response(
    error: &WorkflowEventBusApiError,
    request_id: &str,
) -> WorkflowEventBusApiProblemDetails {
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
) -> WorkflowEventBusRestResponse {
    WorkflowEventBusRestResponse {
        status_code,
        content_type: WORKFLOW_EVENT_BUS_REST_PROBLEM_CONTENT_TYPE.to_owned(),
        body: WorkflowEventBusRestResponseBody::Problem(Box::new(
            WorkflowEventBusApiProblemDetails {
                type_ref: format!(
                    "problem:workflow-event-bus-rest:{}",
                    code.to_ascii_lowercase().replace('_', "-")
                ),
                status: status_code,
                code: code.to_owned(),
                title: title.to_owned(),
                detail_ref: detail_ref.to_owned(),
                evidence_refs: vec![
                    "workflow-event-bus-rest:framework-free-route-facade".to_owned(),
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
        "problem-instance:workflow-event-bus-rest:redacted".to_owned()
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
    fn route_constants_match_event_bus_api_contract() {
        assert_eq!(
            WORKFLOW_EVENT_BUS_REST_PUBLISH_ROUTE,
            WORKFLOW_EVENT_BUS_API_PUBLISH_ROUTE
        );
        assert_eq!(
            WORKFLOW_EVENT_BUS_REST_DELIVERY_ROUTE,
            WORKFLOW_EVENT_BUS_API_DELIVERY_ROUTE
        );
        assert_eq!(
            WORKFLOW_EVENT_BUS_REST_METHOD,
            WorkflowEventBusRestMethod::Post
        );
        assert!(WORKFLOW_EVENT_BUS_REST_CONTRACT_REF.contains("workflow-engine.yaml"));
        assert_eq!(
            match_route(WORKFLOW_EVENT_BUS_REST_PUBLISH_ROUTE),
            Some(WorkflowEventBusRestOperation::Publish)
        );
        assert_eq!(
            match_route(WORKFLOW_EVENT_BUS_REST_DELIVERY_ROUTE),
            Some(WorkflowEventBusRestOperation::EvaluateDelivery)
        );
    }

    #[test]
    fn post_publish_handler_maps_api_accepted_to_202_json_without_http_runtime() {
        let mut rest = WorkflowEventBusRestService::default();
        let response = rest
            .handle(publish_rest_request("idem:event-bus-rest:publish:1"))
            .expect("rest response");

        assert_eq!(response.status_code, 202);
        assert_eq!(
            response.content_type,
            WORKFLOW_EVENT_BUS_REST_SUCCESS_CONTENT_TYPE
        );
        let WorkflowEventBusRestResponseBody::Success(success) = response.body else {
            unreachable!("expected success body");
        };
        assert_eq!(success.route, WORKFLOW_EVENT_BUS_API_PUBLISH_ROUTE);
        assert_eq!(success.event.operation, "publish");
        assert_eq!(success.metadata.surface, WORKFLOW_EVENT_BUS_API_SURFACE);
        assert!(
            success
                .non_claim_refs
                .contains(&"workflow-event-bus-usecase:no-broker-runtime".to_owned())
        );
        assert_eq!(rest.api_delegation_count(), 1);
    }

    #[test]
    fn delivery_handler_preserves_accepted_and_denied_metadata_only_successes() {
        let mut rest = WorkflowEventBusRestService::default();
        let accepted = rest
            .handle(delivery_rest_request("idem:event-bus-rest:delivery:1"))
            .expect("accepted delivery");
        let WorkflowEventBusRestResponseBody::Success(accepted_body) = accepted.body else {
            unreachable!("expected accepted delivery body");
        };
        assert_eq!(accepted.status_code, 202);
        assert_eq!(accepted_body.event.operation, "delivery-evaluate");
        assert_eq!(accepted_body.event.usecase_status, "delivery-accepted");
        assert_eq!(
            accepted_body.event.consumer_ref.as_deref(),
            Some("consumer:workflow-state-machine")
        );

        let mut denied_request = delivery_rest_request("idem:event-bus-rest:delivery-denied");
        let WorkflowEventBusRestRequestBody::Delivery(api_request) = &mut denied_request.body
        else {
            unreachable!("expected delivery request body");
        };
        api_request.body.candidate_channel = "workflow-runs".to_owned();
        api_request.body.candidate_event_type = WorkflowEventBusEventKind::WorkflowRunStarted
            .event_type()
            .to_owned();
        let denied = rest.handle(denied_request).expect("denied delivery");
        let WorkflowEventBusRestResponseBody::Success(denied_body) = denied.body else {
            unreachable!("expected denied delivery success body");
        };
        assert_eq!(denied.status_code, 202);
        assert_eq!(denied_body.event.usecase_status, "delivery-denied");
        assert!(
            denied_body
                .non_claim_refs
                .contains(&"workflow-event-bus-usecase:no-offset-commit-runtime".to_owned())
        );
    }

    #[test]
    fn method_path_and_body_kind_mismatch_never_delegate_to_api() {
        let mut rest = WorkflowEventBusRestService::default();
        let mut method = publish_rest_request("idem:event-bus-rest:method");
        method.method = WorkflowEventBusRestMethod::Get;
        let method_response = rest.handle(method).expect("method problem");
        assert_eq!(method_response.status_code, 405);
        assert_eq!(rest.api_delegation_count(), 0);

        let mut path = publish_rest_request("idem:event-bus-rest:path");
        path.path = "/v/2026-05-25/event-bus/other".to_owned();
        let path_response = rest.handle(path).expect("path problem");
        assert_eq!(path_response.status_code, 404);
        assert_eq!(rest.api_delegation_count(), 0);

        let mut kind = publish_rest_request("idem:event-bus-rest:kind");
        kind.path = WORKFLOW_EVENT_BUS_REST_DELIVERY_ROUTE.to_owned();
        let kind_response = rest.handle(kind).expect("kind problem");
        assert_eq!(kind_response.status_code, 400);
        assert_eq!(rest.api_delegation_count(), 0);
        assert!(
            format!("{kind_response:?}").contains("WORKFLOW_EVENT_BUS_REST_BODY_KIND_MISMATCH")
        );
    }

    #[test]
    fn body_method_or_route_mismatch_returns_problem_without_api_delegation() {
        let mut rest = WorkflowEventBusRestService::default();
        let mut method = publish_rest_request("idem:event-bus-rest:body-method");
        let WorkflowEventBusRestRequestBody::Publish(api_request) = &mut method.body else {
            unreachable!("expected publish body");
        };
        api_request.method = "GET".to_owned();
        let method_response = rest.handle(method).expect("body method problem");
        assert_eq!(method_response.status_code, 400);
        assert_eq!(rest.api_delegation_count(), 0);

        let mut route = publish_rest_request("idem:event-bus-rest:body-route");
        let WorkflowEventBusRestRequestBody::Publish(api_request) = &mut route.body else {
            unreachable!("expected publish body");
        };
        api_request.route = "/v/2026-05-25/event-bus/other".to_owned();
        let route_response = rest.handle(route).expect("body route problem");
        assert_eq!(route_response.status_code, 400);
        assert_eq!(rest.api_delegation_count(), 0);
        assert!(
            format!("{route_response:?}").contains("WORKFLOW_EVENT_BUS_REST_BODY_ROUTE_MISMATCH")
        );
    }

    #[test]
    fn api_errors_are_returned_as_problem_details_without_raw_echo() {
        let mut rest = WorkflowEventBusRestService::default();
        let mut request = publish_rest_request("idem:event-bus-rest:unsafe");
        let WorkflowEventBusRestRequestBody::Publish(api_request) = &mut request.body else {
            unreachable!("expected publish body");
        };
        api_request.body.payload_ref =
            "raw payload Authorization: Bearer sk-test customer message".to_owned();

        let response = rest.handle(request).expect("api problem");

        assert_eq!(response.status_code, 400);
        assert_eq!(
            response.content_type,
            WORKFLOW_EVENT_BUS_REST_PROBLEM_CONTENT_TYPE
        );
        let rendered = format!("{response:?}");
        assert!(rendered.contains(WorkflowEventBusApiErrorCode::UnsafeMetadata.as_str()));
        assert!(!rendered.contains("raw payload"));
        assert!(!rendered.contains("Authorization"));
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("customer message"));
        assert_eq!(rest.api_delegation_count(), 1);
    }

    #[test]
    fn idempotent_replay_and_conflict_flow_through_rest_service_cache() {
        let mut rest = WorkflowEventBusRestService::default();
        let request = publish_rest_request("idem:event-bus-rest:replay");
        let first = rest.handle(request.clone()).expect("first");
        let second = rest.handle(request).expect("second");

        assert_eq!(first, second);
        assert_eq!(rest.api_delegation_count(), 2);

        let mut drifted = publish_rest_request("idem:event-bus-rest:replay");
        let WorkflowEventBusRestRequestBody::Publish(api_request) = &mut drifted.body else {
            unreachable!("expected publish body");
        };
        api_request.body.event_id = "event:workflow-run-started:drift".to_owned();
        let conflict = rest.handle(drifted).expect("conflict problem");
        assert_eq!(conflict.status_code, 409);
        assert!(
            format!("{conflict:?}")
                .contains(WorkflowEventBusApiErrorCode::IdempotencyKeyReused.as_str())
        );
    }

    #[test]
    fn unsafe_rest_request_id_is_redacted_and_not_delegated() {
        let mut rest = WorkflowEventBusRestService::new(WorkflowEventBusApi::default());
        let mut request = publish_rest_request("idem:event-bus-rest:unsafe-request-id");
        request.request_id = "request:raw prompt bearer sk-test payload".to_owned();

        let response = rest.handle(request).expect("unsafe request id problem");

        assert_eq!(response.status_code, 400);
        assert_eq!(rest.api_delegation_count(), 0);
        let rendered = format!("{response:?}");
        assert!(rendered.contains("problem-instance:workflow-event-bus-rest:redacted"));
        assert!(!rendered.contains("raw prompt"));
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("payload"));
    }

    fn publish_rest_request(idempotency_key: &str) -> WorkflowEventBusRestRequest {
        WorkflowEventBusRestRequest {
            method: WorkflowEventBusRestMethod::Post,
            path: WORKFLOW_EVENT_BUS_REST_PUBLISH_ROUTE.to_owned(),
            request_id: format!("request:event-bus-rest:{idempotency_key}"),
            body: WorkflowEventBusRestRequestBody::Publish(Box::new(publish_request(
                idempotency_key,
            ))),
        }
    }

    fn delivery_rest_request(idempotency_key: &str) -> WorkflowEventBusRestRequest {
        WorkflowEventBusRestRequest {
            method: WorkflowEventBusRestMethod::Post,
            path: WORKFLOW_EVENT_BUS_REST_DELIVERY_ROUTE.to_owned(),
            request_id: format!("request:event-bus-rest:{idempotency_key}"),
            body: WorkflowEventBusRestRequestBody::Delivery(Box::new(delivery_request(
                idempotency_key,
            ))),
        }
    }

    fn publish_request(idempotency_key: &str) -> WorkflowEventBusApiPublishRequest {
        WorkflowEventBusApiPublishRequest {
            boundary: boundary(idempotency_key),
            principal: principal(),
            authorization: authorization(),
            method: WORKFLOW_EVENT_BUS_API_METHOD.to_owned(),
            route: WORKFLOW_EVENT_BUS_API_PUBLISH_ROUTE.to_owned(),
            body: WorkflowEventBusApiPublishBody {
                cell_id: "cell:us-east-1a".to_owned(),
                residency_ref: "residency:us:data-plane".to_owned(),
                audit_chain_ref: "audit-chain:event-bus-rest".to_owned(),
                event_kind: "workflow-run-started".to_owned(),
                producer_ref: "producer:workflow-engine:execution".to_owned(),
                event_id: "event:workflow-run-started:001".to_owned(),
                source_ref: "urn:oyatie:workflow-engine:execution".to_owned(),
                subject_ref: Some("subject:workflow-run:001".to_owned()),
                time_ref: Some("time:2026-05-25T00:00:00Z".to_owned()),
                dataschema_ref: Some("schema:workflow-event-run-started".to_owned()),
                partition_key_ref: "partition:tenant-workflow-run".to_owned(),
                publish_idempotency_key: "idem:event-bus-domain:publish:001".to_owned(),
                causation_ref: "cause:execution-engine:start-run".to_owned(),
                correlation_ref: "corr:workflow-run:001".to_owned(),
                payload_ref: "body-ref:workflow-run-started".to_owned(),
                evidence_refs: vec!["evidence:event-bus-rest:publish".to_owned()],
            },
        }
    }

    fn delivery_request(idempotency_key: &str) -> WorkflowEventBusApiDeliveryRequest {
        WorkflowEventBusApiDeliveryRequest {
            boundary: boundary(idempotency_key),
            principal: principal(),
            authorization: authorization(),
            method: WORKFLOW_EVENT_BUS_API_METHOD.to_owned(),
            route: WORKFLOW_EVENT_BUS_API_DELIVERY_ROUTE.to_owned(),
            body: WorkflowEventBusApiDeliveryBody {
                cell_id: "cell:us-east-1a".to_owned(),
                residency_ref: "residency:us:data-plane".to_owned(),
                audit_chain_ref: "audit-chain:event-bus-rest".to_owned(),
                subscription_channel: "workflow-state".to_owned(),
                consumer_ref: "consumer:workflow-state-machine".to_owned(),
                subscription_event_types: vec![
                    WorkflowEventBusEventKind::WorkflowStateTransitioned
                        .event_type()
                        .to_owned(),
                ],
                replay_cursor_ref: Some("cursor:event-bus-rest:state".to_owned()),
                max_batch_size: 100,
                subscription_authorization_evidence_ref: "authz:event-bus-rest:consume".to_owned(),
                candidate_channel: "workflow-state".to_owned(),
                candidate_event_id: "event:workflow-state:001".to_owned(),
                candidate_event_type: WorkflowEventBusEventKind::WorkflowStateTransitioned
                    .event_type()
                    .to_owned(),
                candidate_idempotency_key: "idem:event-bus-domain:delivery:001".to_owned(),
                candidate_payload_ref: "body-ref:workflow-state-transitioned".to_owned(),
                candidate_offset_ref: "offset:partition-0:42".to_owned(),
                candidate_evidence_refs: vec!["evidence:event-bus-rest:delivery".to_owned()],
            },
        }
    }

    fn boundary(idempotency_key: &str) -> WorkflowEventBusApiBoundaryContext {
        WorkflowEventBusApiBoundaryContext {
            request_id: format!("request:event-bus-api:{idempotency_key}"),
            tenant_id: "ten_workflow_event_bus".to_owned(),
            idempotency_key: idempotency_key.to_owned(),
            trace_context_ref: "trace:event-bus-rest".to_owned(),
            oyatie_version: WORKFLOW_EVENT_BUS_API_DECLARED_VERSION.to_owned(),
        }
    }

    fn principal() -> WorkflowEventBusApiPrincipal {
        WorkflowEventBusApiPrincipal {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            principal_id: "principal:workflow-operator".to_owned(),
        }
    }

    fn authorization() -> WorkflowEventBusApiAuthorization {
        WorkflowEventBusApiAuthorization {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            principal_id: "principal:workflow-operator".to_owned(),
            decision_id: "policy-decision:event-bus-allow".to_owned(),
            evidence_ref: "policy-evidence:event-bus-allow".to_owned(),
            policy_bundle_ref: "policy-bundle:event-bus-v1".to_owned(),
            allowed_surfaces: vec![WORKFLOW_EVENT_BUS_API_SURFACE.to_owned()],
            allowed_channels: vec![
                "workflow-runs".to_owned(),
                "workflow-state".to_owned(),
                "trigger-events".to_owned(),
                "intelligence-requests".to_owned(),
                "ontology-projections".to_owned(),
            ],
            allowed_event_types: vec![
                WorkflowEventBusEventKind::WorkflowRunStarted
                    .event_type()
                    .to_owned(),
                WorkflowEventBusEventKind::WorkflowStateTransitioned
                    .event_type()
                    .to_owned(),
                WorkflowEventBusEventKind::TriggerEvaluated
                    .event_type()
                    .to_owned(),
                WorkflowEventBusEventKind::IntelligenceDraftRequested
                    .event_type()
                    .to_owned(),
                WorkflowEventBusEventKind::OntologyProjectionUpdated
                    .event_type()
                    .to_owned(),
            ],
        }
    }
}
