//! Framework-free REST boundary for messenger message-stream.
//!
//! Hyper bindings stay in runtime composition. This crate owns route constants,
//! header-shaped request context, OpenAPI-aligned request shapes, and handler
//! functions that call the protocol-neutral API/usecase layer.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use oya_messenger_app::{MessengerAppError, MessengerWritePlan, plan_send_message};
use oya_messenger_message_stream_api::{
    AuthorizedMessengerContext, MessageReceipt, MessengerApiContext, MessengerApiEnvelope,
    MessengerApiError, SendMessageRequest,
};
use oya_messenger_message_stream_usecase::{MessengerUsecaseError, send_message};
use oya_shared_hyperscaler_metrics_kernel::{
    MetricsContext, MetricsError, RequestTelemetryBinding,
};
use oya_shared_postgres_command_kernel::TenantSqlContext;

pub const POST_MESSAGE_ROUTE: &str = "/channels/{channel_id}/messages";
pub const POST_MESSAGE_METHOD: &str = "POST";

pub const HEALTH_ROUTE: &str = "/health";
pub const READY_ROUTE: &str = "/ready";
pub const PROBE_METHOD: &str = "GET";

pub const MESSENGER_REST_MICROSERVICE: &str = "messenger";
pub const POST_MESSAGE_OPERATION_ID: &str = "messenger.post_message";

pub fn telemetry_binding() -> Result<RequestTelemetryBinding, MetricsError> {
    let context = MetricsContext::new(MESSENGER_REST_MICROSERVICE)?;
    RequestTelemetryBinding::new(&context, POST_MESSAGE_OPERATION_ID)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RouteHandlerStatus {
    Implemented,
    ContractOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpenApiRoute {
    pub method: &'static str,
    pub path: &'static str,
    pub handler_status: RouteHandlerStatus,
}

pub const OPENAPI_ROUTES: &[OpenApiRoute] = &[
    OpenApiRoute {
        method: "GET",
        path: "/channels",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/channels",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/channels/{channel_id}",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "DELETE",
        path: "/channels/{channel_id}",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/channels/{channel_id}/members",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/channels/{channel_id}/members",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "DELETE",
        path: "/channels/{channel_id}/members/{user_ref}",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/channels/{channel_id}/messages",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: POST_MESSAGE_METHOD,
        path: POST_MESSAGE_ROUTE,
        handler_status: RouteHandlerStatus::Implemented,
    },
    OpenApiRoute {
        method: "GET",
        path: "/channels/{channel_id}/messages/{message_id}",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "PATCH",
        path: "/channels/{channel_id}/messages/{message_id}",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "DELETE",
        path: "/channels/{channel_id}/messages/{message_id}",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/channels/{channel_id}/messages/{message_id}/reactions",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "DELETE",
        path: "/channels/{channel_id}/messages/{message_id}/reactions/{emoji}",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/channels/{channel_id}/messages/{message_id}/read",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/threads/{thread_id}/replies",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/threads/{thread_id}/replies",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/attachments",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/attachments/{attachment_id}",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/search",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/presence/{user_ref}",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "PUT",
        path: "/presence/me",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/holds",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/disclosures",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: PROBE_METHOD,
        path: HEALTH_ROUTE,
        handler_status: RouteHandlerStatus::Implemented,
    },
    OpenApiRoute {
        method: PROBE_METHOD,
        path: READY_ROUTE,
        handler_status: RouteHandlerStatus::Implemented,
    },
];

pub fn find_openapi_route(method: &str, path: &str) -> Option<&'static OpenApiRoute> {
    OPENAPI_ROUTES
        .iter()
        .find(|route| route.method == method && route.path == path)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RestContextKind {
    Personal,
    Professional,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessengerRestContext {
    pub scope_org_id: String,
    pub context_kind: RestContextKind,
    pub principal_ref: String,
    pub idempotency_key: String,
    pub policy_decision_ref: String,
    pub request_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostMessageRestRequest {
    pub channel_id: String,
    pub message_id: String,
    pub author_ref: String,
    pub envelope: MessengerApiEnvelope,
    pub retention_policy_id: String,
    pub legal_hold_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RestResponse<T> {
    pub status_code: u16,
    pub body: T,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadinessDependency {
    pub name: &'static str, // data_class: INTERNAL_ONLY
    pub ready: bool,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProbeStatus {
    Healthy,
    Ready,
    NotReady,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProbeRouteResponse {
    pub status_code: u16,                       // data_class: INTERNAL_ONLY
    pub microservice: &'static str,             // data_class: INTERNAL_ONLY
    pub route: &'static str,                    // data_class: INTERNAL_ONLY
    pub status: ProbeStatus,                    // data_class: INTERNAL_ONLY
    pub dependencies: Vec<ReadinessDependency>, // data_class: INTERNAL_ONLY
    pub non_claim: &'static str,                // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProbeRouteDispatchError {
    UnknownRoute,
    NotProbeRoute {
        method: &'static str,
        path: &'static str,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractOnlyRouteResponse {
    pub status_code: u16,
    pub method: &'static str,
    pub path: &'static str,
    pub reason: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RouteDispatchError {
    UnknownRoute,
    TypedHandlerRequired {
        method: &'static str,
        path: &'static str,
    },
}

pub fn dispatch_probe_route(
    method: &str,
    path: &str,
    dependencies: Vec<ReadinessDependency>,
) -> Result<RestResponse<ProbeRouteResponse>, ProbeRouteDispatchError> {
    let Some(route) = find_openapi_route(method, path) else {
        return Err(ProbeRouteDispatchError::UnknownRoute);
    };
    match (route.method, route.path) {
        (PROBE_METHOD, HEALTH_ROUTE) => Ok(RestResponse {
            status_code: 200,
            body: ProbeRouteResponse {
                status_code: 200,
                microservice: MESSENGER_REST_MICROSERVICE,
                route: HEALTH_ROUTE,
                status: ProbeStatus::Healthy,
                dependencies: Vec::new(),
                non_claim: "process-level liveness only; no downstream dependency readiness claim",
            },
        }),
        (PROBE_METHOD, READY_ROUTE) => {
            let ready = dependencies.iter().all(|dependency| dependency.ready);
            let status_code = if ready { 200 } else { 503 };
            Ok(RestResponse {
                status_code,
                body: ProbeRouteResponse {
                    status_code,
                    microservice: MESSENGER_REST_MICROSERVICE,
                    route: READY_ROUTE,
                    status: if ready {
                        ProbeStatus::Ready
                    } else {
                        ProbeStatus::NotReady
                    },
                    dependencies,
                    non_claim: "readiness is caller-supplied framework-free evidence; no live deployment probe has run",
                },
            })
        }
        (method, path) => Err(ProbeRouteDispatchError::NotProbeRoute { method, path }),
    }
}

pub fn dispatch_contract_only_route(
    method: &str,
    path: &str,
) -> Result<RestResponse<ContractOnlyRouteResponse>, RouteDispatchError> {
    let Some(route) = find_openapi_route(method, path) else {
        return Err(RouteDispatchError::UnknownRoute);
    };
    match route.handler_status {
        RouteHandlerStatus::ContractOnly => Ok(RestResponse {
            status_code: 501,
            body: ContractOnlyRouteResponse {
                status_code: 501,
                method: route.method,
                path: route.path,
                reason: "contract-only route; no runtime handler claim",
            },
        }),
        RouteHandlerStatus::Implemented => Err(RouteDispatchError::TypedHandlerRequired {
            method: route.method,
            path: route.path,
        }),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessengerRestError {
    MissingPathChannel,
    Api(MessengerApiError),
    Usecase(MessengerUsecaseError),
    App(MessengerAppError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessengerWriteRouteRequest {
    PostMessage(PostMessageRestRequest),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessengerWriteRouteResponse {
    PostMessage(MessageReceipt),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessengerWriteRouteDispatchError {
    UnknownRoute,
    ContractOnly {
        method: &'static str,
        path: &'static str,
    },
    PayloadMismatch {
        method: &'static str,
        path: &'static str,
    },
    Handler(MessengerRestError),
}

pub fn dispatch_write_route(
    method: &str,
    path: &str,
    context: MessengerRestContext,
    request: MessengerWriteRouteRequest,
) -> Result<RestResponse<MessengerWriteRouteResponse>, MessengerWriteRouteDispatchError> {
    let Some(route) = find_openapi_route(method, path) else {
        return Err(MessengerWriteRouteDispatchError::UnknownRoute);
    };
    if route.handler_status == RouteHandlerStatus::ContractOnly {
        return Err(MessengerWriteRouteDispatchError::ContractOnly {
            method: route.method,
            path: route.path,
        });
    }
    match (route.method, route.path, request) {
        (
            POST_MESSAGE_METHOD,
            POST_MESSAGE_ROUTE,
            MessengerWriteRouteRequest::PostMessage(request),
        ) => post_message(context, request)
            .map(|response| RestResponse {
                status_code: response.status_code,
                body: MessengerWriteRouteResponse::PostMessage(response.body),
            })
            .map_err(MessengerWriteRouteDispatchError::Handler),
        (method, path, _) => {
            Err(MessengerWriteRouteDispatchError::PayloadMismatch { method, path })
        }
    }
}

pub fn post_message(
    context: MessengerRestContext,
    request: PostMessageRestRequest,
) -> Result<RestResponse<MessageReceipt>, MessengerRestError> {
    if request.channel_id.trim().is_empty() {
        return Err(MessengerRestError::MissingPathChannel);
    }
    let api_context = AuthorizedMessengerContext {
        context: map_context(context.context_kind),
        scope_ref: context.scope_org_id,
        principal_ref: context.principal_ref,
        idempotency_key: context.idempotency_key,
        policy_decision_ref: context.policy_decision_ref,
        audit_correlation_id: context.request_id,
    };
    api_context.validate().map_err(MessengerRestError::Api)?;
    let (_, receipt) = send_message(
        &api_context,
        SendMessageRequest {
            message_id: request.message_id,
            channel_id: request.channel_id,
            author_ref: request.author_ref,
            envelope: request.envelope,
            retention_policy_id: request.retention_policy_id,
            legal_hold_ids: request.legal_hold_ids,
        },
    )
    .map_err(MessengerRestError::Usecase)?;
    Ok(RestResponse {
        status_code: 201,
        body: receipt,
    })
}

pub fn post_message_write_plan(
    tenant: TenantSqlContext,
    context: MessengerRestContext,
    request: PostMessageRestRequest,
) -> Result<RestResponse<MessengerWritePlan>, MessengerRestError> {
    if request.channel_id.trim().is_empty() {
        return Err(MessengerRestError::MissingPathChannel);
    }
    let api_context = messenger_api_context(context);
    api_context.validate().map_err(MessengerRestError::Api)?;
    let plan = plan_send_message(
        tenant,
        api_context,
        SendMessageRequest {
            message_id: request.message_id,
            channel_id: request.channel_id,
            author_ref: request.author_ref,
            envelope: request.envelope,
            retention_policy_id: request.retention_policy_id,
            legal_hold_ids: request.legal_hold_ids,
        },
    )
    .map_err(MessengerRestError::App)?;
    Ok(RestResponse {
        status_code: 201,
        body: plan,
    })
}

fn messenger_api_context(context: MessengerRestContext) -> AuthorizedMessengerContext {
    AuthorizedMessengerContext {
        context: map_context(context.context_kind),
        scope_ref: context.scope_org_id,
        principal_ref: context.principal_ref,
        idempotency_key: context.idempotency_key,
        policy_decision_ref: context.policy_decision_ref,
        audit_correlation_id: context.request_id,
    }
}

fn map_context(context: RestContextKind) -> MessengerApiContext {
    match context {
        RestContextKind::Personal => MessengerApiContext::Personal,
        RestContextKind::Professional => MessengerApiContext::Work,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tenant() -> TenantSqlContext {
        TenantSqlContext::new("tenant:t", "cell-a", "tenant:t#cell-a", "US").unwrap()
    }

    fn context() -> MessengerRestContext {
        MessengerRestContext {
            scope_org_id: "tenant:t".into(),
            context_kind: RestContextKind::Professional,
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "cedar:allow:message-send".into(),
            request_id: "req".into(),
        }
    }

    #[test]
    fn route_matches_openapi_post_message() {
        assert_eq!(POST_MESSAGE_METHOD, "POST");
        assert_eq!(POST_MESSAGE_ROUTE, "/channels/{channel_id}/messages");
    }

    #[test]
    fn openapi_route_catalog_covers_declared_operations() {
        assert_eq!(OPENAPI_ROUTES.len(), 26);
        assert_eq!(
            find_openapi_route(POST_MESSAGE_METHOD, POST_MESSAGE_ROUTE)
                .map(|route| route.handler_status),
            Some(RouteHandlerStatus::Implemented)
        );
        assert_eq!(
            find_openapi_route(PROBE_METHOD, HEALTH_ROUTE).map(|route| route.handler_status),
            Some(RouteHandlerStatus::Implemented)
        );
        assert_eq!(
            find_openapi_route(PROBE_METHOD, READY_ROUTE).map(|route| route.handler_status),
            Some(RouteHandlerStatus::Implemented)
        );
        assert!(find_openapi_route("GET", "/ready").is_some());
    }

    #[test]
    fn post_message_returns_created_receipt() {
        let response = post_message(
            context(),
            PostMessageRestRequest {
                channel_id: "c".into(),
                message_id: "m".into(),
                author_ref: "user:u".into(),
                envelope: MessengerApiEnvelope::TenantDek {
                    dek_ref: "dek".into(),
                    four_eyes: true,
                },
                retention_policy_id: "retain".into(),
                legal_hold_ids: vec![],
            },
        )
        .unwrap();
        assert_eq!(response.status_code, 201);
        assert_eq!(response.body.event_type, "messenger.message.sent");
    }

    #[test]
    fn post_message_write_plan_returns_persistence_and_protocol_event() {
        let response = post_message_write_plan(
            tenant(),
            context(),
            PostMessageRestRequest {
                channel_id: "c".into(),
                message_id: "m".into(),
                author_ref: "user:u".into(),
                envelope: MessengerApiEnvelope::TenantDek {
                    dek_ref: "dek".into(),
                    four_eyes: true,
                },
                retention_policy_id: "retain".into(),
                legal_hold_ids: vec![],
            },
        )
        .unwrap();

        assert_eq!(response.status_code, 201);
        assert_eq!(response.body.receipt.event_type, "messenger.message.sent");
        assert_eq!(response.body.persistence.statements.len(), 3);
        assert_eq!(
            response.body.persistence.statements[2].name,
            "insert_transactional_outbox_event"
        );
        assert_eq!(
            response.body.protocol_event.binding.proto_rpc,
            "PostMessage"
        );
    }

    #[test]
    fn post_message_rejects_missing_idempotency_before_usecase() {
        let mut ctx = context();
        ctx.idempotency_key.clear();
        let result = post_message(
            ctx,
            PostMessageRestRequest {
                channel_id: "c".into(),
                message_id: "m".into(),
                author_ref: "user:u".into(),
                envelope: MessengerApiEnvelope::TenantDek {
                    dek_ref: "dek".into(),
                    four_eyes: true,
                },
                retention_policy_id: "retain".into(),
                legal_hold_ids: vec![],
            },
        );
        assert_eq!(
            result,
            Err(MessengerRestError::Api(
                MessengerApiError::MissingIdempotencyKey
            ))
        );
    }

    #[test]
    fn post_message_rejects_missing_policy_decision() {
        let mut ctx = context();
        ctx.policy_decision_ref.clear();
        let result = post_message(
            ctx,
            PostMessageRestRequest {
                channel_id: "c".into(),
                message_id: "m".into(),
                author_ref: "user:u".into(),
                envelope: MessengerApiEnvelope::TenantDek {
                    dek_ref: "dek".into(),
                    four_eyes: true,
                },
                retention_policy_id: "retain".into(),
                legal_hold_ids: vec![],
            },
        );
        assert_eq!(
            result,
            Err(MessengerRestError::Api(
                MessengerApiError::MissingPolicyDecision
            ))
        );
    }

    #[test]
    fn messenger_post_message_telemetry_binding_uses_canonical_metric_names() {
        let binding = telemetry_binding().unwrap();

        assert_eq!(binding.microservice, MESSENGER_REST_MICROSERVICE);
        assert_eq!(binding.operation_id, POST_MESSAGE_OPERATION_ID);
        assert_eq!(binding.request_total_metric, "oya_messenger_request_total");
        assert_eq!(
            binding.request_success_metric,
            "oya_messenger_request_success_total"
        );
        assert_eq!(
            binding.responses_total_metric,
            "oya_messenger_responses_total"
        );
        assert_eq!(
            binding.responses_5xx_metric,
            "oya_messenger_responses_5xx_total"
        );
        assert_eq!(
            binding.responses_429_metric,
            "oya_messenger_responses_429_total"
        );
    }

    #[test]
    fn messenger_probe_dispatch_reports_liveness_and_readiness() {
        let health = dispatch_probe_route(
            PROBE_METHOD,
            HEALTH_ROUTE,
            vec![ReadinessDependency {
                name: "sql",
                ready: false,
            }],
        )
        .unwrap();
        assert_eq!(health.status_code, 200);
        assert_eq!(health.body.status, ProbeStatus::Healthy);
        assert!(health.body.dependencies.is_empty());
        assert!(health.body.non_claim.contains("liveness"));

        let not_ready = dispatch_probe_route(
            PROBE_METHOD,
            READY_ROUTE,
            vec![
                ReadinessDependency {
                    name: "sql",
                    ready: true,
                },
                ReadinessDependency {
                    name: "outbox",
                    ready: false,
                },
            ],
        )
        .unwrap();
        assert_eq!(not_ready.status_code, 503);
        assert_eq!(not_ready.body.status, ProbeStatus::NotReady);
        assert_eq!(not_ready.body.dependencies.len(), 2);
        assert!(
            not_ready
                .body
                .non_claim
                .contains("no live deployment probe")
        );

        let ready = dispatch_probe_route(
            PROBE_METHOD,
            READY_ROUTE,
            vec![ReadinessDependency {
                name: "sql",
                ready: true,
            }],
        )
        .unwrap();
        assert_eq!(ready.status_code, 200);
        assert_eq!(ready.body.status, ProbeStatus::Ready);

        assert_eq!(
            dispatch_probe_route("GET", "/channels", Vec::new()),
            Err(ProbeRouteDispatchError::NotProbeRoute {
                method: "GET",
                path: "/channels",
            })
        );
        assert_eq!(
            dispatch_probe_route(PROBE_METHOD, "/does-not-exist", Vec::new()),
            Err(ProbeRouteDispatchError::UnknownRoute)
        );
    }

    #[test]
    fn messenger_contract_only_dispatch_returns_501_for_known_unimplemented_route() {
        let response = dispatch_contract_only_route("GET", "/channels").unwrap();

        assert_eq!(response.status_code, 501);
        assert_eq!(response.body.status_code, 501);
        assert_eq!(response.body.method, "GET");
        assert_eq!(response.body.path, "/channels");
        assert_eq!(
            response.body.reason,
            "contract-only route; no runtime handler claim"
        );
    }

    #[test]
    fn messenger_contract_only_dispatch_refuses_typed_handler_routes() {
        assert_eq!(
            dispatch_contract_only_route(POST_MESSAGE_METHOD, POST_MESSAGE_ROUTE),
            Err(RouteDispatchError::TypedHandlerRequired {
                method: POST_MESSAGE_METHOD,
                path: POST_MESSAGE_ROUTE
            })
        );
    }

    #[test]
    fn messenger_contract_only_dispatch_rejects_unknown_route() {
        assert_eq!(
            dispatch_contract_only_route("GET", "/does-not-exist"),
            Err(RouteDispatchError::UnknownRoute)
        );
    }

    #[test]
    fn messenger_write_route_dispatch_calls_business_handler() {
        let response = dispatch_write_route(
            POST_MESSAGE_METHOD,
            POST_MESSAGE_ROUTE,
            context(),
            MessengerWriteRouteRequest::PostMessage(PostMessageRestRequest {
                channel_id: "c".into(),
                message_id: "m".into(),
                author_ref: "user:u".into(),
                envelope: MessengerApiEnvelope::TenantDek {
                    dek_ref: "dek".into(),
                    four_eyes: true,
                },
                retention_policy_id: "retain".into(),
                legal_hold_ids: vec![],
            }),
        )
        .unwrap();

        assert_eq!(response.status_code, 201);
        let MessengerWriteRouteResponse::PostMessage(receipt) = response.body;
        assert_eq!(receipt.event_type, "messenger.message.sent");
    }

    #[test]
    fn messenger_write_route_dispatch_refuses_contract_only_route() {
        let result = dispatch_write_route(
            "GET",
            "/channels",
            context(),
            MessengerWriteRouteRequest::PostMessage(PostMessageRestRequest {
                channel_id: "c".into(),
                message_id: "m".into(),
                author_ref: "user:u".into(),
                envelope: MessengerApiEnvelope::TenantDek {
                    dek_ref: "dek".into(),
                    four_eyes: true,
                },
                retention_policy_id: "retain".into(),
                legal_hold_ids: vec![],
            }),
        );

        assert_eq!(
            result,
            Err(MessengerWriteRouteDispatchError::ContractOnly {
                method: "GET",
                path: "/channels",
            })
        );
    }
}
