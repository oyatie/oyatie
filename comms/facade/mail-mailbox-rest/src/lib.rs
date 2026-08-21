//! Framework-free REST boundary for mail mailbox-store.
//!
//! Owns OpenAPI-aligned route constants and transport-shaped handlers while the
//! runtime crate binds Hyper/h3 and middleware.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use comms_mail_mailbox_api::{
    AuthorizedMailContext, MailApiContext, MailApiEnvelope, MailApiError, SubmissionReceipt,
    SubmitMessageRequest,
};
use comms_mail_mailbox_app::{MailAppError, MailSubmissionPlan, plan_submit_message};
use comms_mail_mailbox_usecase::{MailUsecaseError, submit_message};
use shared_hyperscaler_metrics_kernel::{
    MetricsContext, MetricsError, RequestTelemetryBinding,
};
use shared_postgres_command_kernel::TenantSqlContext;

pub const SUBMIT_MESSAGE_ROUTE: &str = "/messages";
pub const SUBMIT_MESSAGE_METHOD: &str = "POST";

pub const HEALTH_ROUTE: &str = "/health";
pub const READY_ROUTE: &str = "/ready";
pub const PROBE_METHOD: &str = "GET";

pub const MAIL_REST_MICROSERVICE: &str = "mail";
pub const SUBMIT_MESSAGE_OPERATION_ID: &str = "mail.submit_message";

pub fn telemetry_binding() -> Result<RequestTelemetryBinding, MetricsError> {
    let context = MetricsContext::new(MAIL_REST_MICROSERVICE)?;
    RequestTelemetryBinding::new(&context, SUBMIT_MESSAGE_OPERATION_ID)
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
        path: "/mailboxes",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/mailboxes/{mailboxId}",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/mailboxes/{mailboxId}/threads",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/mailboxes/{mailboxId}/messages/{messageId}",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "DELETE",
        path: "/mailboxes/{mailboxId}/messages/{messageId}",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: SUBMIT_MESSAGE_METHOD,
        path: SUBMIT_MESSAGE_ROUTE,
        handler_status: RouteHandlerStatus::Implemented,
    },
    OpenApiRoute {
        method: "POST",
        path: "/search",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/legal-holds",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "DELETE",
        path: "/legal-holds/{holdId}",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/ediscovery/exports",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/ediscovery/exports/{exportId}",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/dlp/quarantine",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/dlp/quarantine/{quarantineId}/release",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/tenants/{tenantId}/dkim/rotate",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/admin/mailboxes/{mailboxId}/restore",
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
pub enum MailRestContextKind {
    Personal,
    Professional,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailRestContext {
    pub tenant_id: String,
    pub context_kind: MailRestContextKind,
    pub principal_ref: String,
    pub idempotency_key: String,
    pub policy_decision_ref: String,
    pub request_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubmitMessageRestRequest {
    pub message_id: String,
    pub mailbox_id: String,
    pub subject_ref: String,
    pub envelope: MailApiEnvelope,
    pub retention_policy_id: String,
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
                microservice: MAIL_REST_MICROSERVICE,
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
                    microservice: MAIL_REST_MICROSERVICE,
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
pub enum MailRestError {
    Api(MailApiError),
    Usecase(MailUsecaseError),
    App(MailAppError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailWriteRouteRequest {
    SubmitMessage(SubmitMessageRestRequest),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailWriteRouteResponse {
    SubmitMessage(SubmissionReceipt),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailWriteRouteDispatchError {
    UnknownRoute,
    ContractOnly {
        method: &'static str,
        path: &'static str,
    },
    PayloadMismatch {
        method: &'static str,
        path: &'static str,
    },
    Handler(MailRestError),
}

pub fn dispatch_write_route(
    method: &str,
    path: &str,
    context: MailRestContext,
    request: MailWriteRouteRequest,
) -> Result<RestResponse<MailWriteRouteResponse>, MailWriteRouteDispatchError> {
    let Some(route) = find_openapi_route(method, path) else {
        return Err(MailWriteRouteDispatchError::UnknownRoute);
    };
    if route.handler_status == RouteHandlerStatus::ContractOnly {
        return Err(MailWriteRouteDispatchError::ContractOnly {
            method: route.method,
            path: route.path,
        });
    }
    match (route.method, route.path, request) {
        (
            SUBMIT_MESSAGE_METHOD,
            SUBMIT_MESSAGE_ROUTE,
            MailWriteRouteRequest::SubmitMessage(request),
        ) => send_message(context, request)
            .map(|response| RestResponse {
                status_code: response.status_code,
                body: MailWriteRouteResponse::SubmitMessage(response.body),
            })
            .map_err(MailWriteRouteDispatchError::Handler),
        (method, path, _) => Err(MailWriteRouteDispatchError::PayloadMismatch { method, path }),
    }
}

pub fn send_message(
    context: MailRestContext,
    request: SubmitMessageRestRequest,
) -> Result<RestResponse<SubmissionReceipt>, MailRestError> {
    let api_context = AuthorizedMailContext {
        context: map_context(context.context_kind),
        scope_ref: context.tenant_id,
        principal_ref: context.principal_ref,
        idempotency_key: context.idempotency_key,
        policy_decision_ref: context.policy_decision_ref,
        audit_correlation_id: context.request_id,
    };
    api_context.validate().map_err(MailRestError::Api)?;
    let receipt = submit_message(
        &api_context,
        SubmitMessageRequest {
            message_id: request.message_id,
            mailbox_id: request.mailbox_id,
            subject_ref: request.subject_ref,
            envelope: request.envelope,
            retention_policy_id: request.retention_policy_id,
            dmarc_check: None,
        },
    )
    .map_err(MailRestError::Usecase)?;
    Ok(RestResponse {
        status_code: 202,
        body: receipt,
    })
}

pub fn send_message_write_plan(
    tenant: TenantSqlContext,
    context: MailRestContext,
    request: SubmitMessageRestRequest,
) -> Result<RestResponse<MailSubmissionPlan>, MailRestError> {
    let api_context = mail_api_context(context);
    api_context.validate().map_err(MailRestError::Api)?;
    let plan = plan_submit_message(
        tenant,
        api_context,
        SubmitMessageRequest {
            message_id: request.message_id,
            mailbox_id: request.mailbox_id,
            subject_ref: request.subject_ref,
            envelope: request.envelope,
            retention_policy_id: request.retention_policy_id,
            dmarc_check: None,
        },
    )
    .map_err(MailRestError::App)?;
    Ok(RestResponse {
        status_code: 202,
        body: plan,
    })
}

fn mail_api_context(context: MailRestContext) -> AuthorizedMailContext {
    AuthorizedMailContext {
        context: map_context(context.context_kind),
        scope_ref: context.tenant_id,
        principal_ref: context.principal_ref,
        idempotency_key: context.idempotency_key,
        policy_decision_ref: context.policy_decision_ref,
        audit_correlation_id: context.request_id,
    }
}

fn map_context(context: MailRestContextKind) -> MailApiContext {
    match context {
        MailRestContextKind::Personal => MailApiContext::Personal,
        MailRestContextKind::Professional => MailApiContext::Work,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tenant() -> TenantSqlContext {
        TenantSqlContext::new("tenant:t", "cell-a", "tenant:t#cell-a", "US").unwrap()
    }

    fn context() -> MailRestContext {
        MailRestContext {
            tenant_id: "tenant:t".into(),
            context_kind: MailRestContextKind::Professional,
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "cedar:allow:mail-submit".into(),
            request_id: "req".into(),
        }
    }

    #[test]
    fn route_matches_openapi_send_message() {
        assert_eq!(SUBMIT_MESSAGE_METHOD, "POST");
        assert_eq!(SUBMIT_MESSAGE_ROUTE, "/messages");
    }

    #[test]
    fn openapi_route_catalog_covers_declared_operations() {
        assert_eq!(OPENAPI_ROUTES.len(), 17);
        assert_eq!(
            find_openapi_route(SUBMIT_MESSAGE_METHOD, SUBMIT_MESSAGE_ROUTE)
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
        assert!(find_openapi_route("POST", "/ediscovery/exports").is_some());
    }

    #[test]
    fn send_message_returns_accepted_receipt() {
        let response = send_message(
            context(),
            SubmitMessageRestRequest {
                message_id: "m".into(),
                mailbox_id: "mb".into(),
                subject_ref: "user:u".into(),
                envelope: MailApiEnvelope::TenantDek {
                    dek_ref: "dek".into(),
                },
                retention_policy_id: "retain".into(),
            },
        )
        .unwrap();
        assert_eq!(response.status_code, 202);
        assert_eq!(response.body.event_type, "mail.message.submitted");
    }

    #[test]
    fn send_message_write_plan_returns_persistence_and_protocol_event() {
        let response = send_message_write_plan(
            tenant(),
            context(),
            SubmitMessageRestRequest {
                message_id: "m".into(),
                mailbox_id: "mb".into(),
                subject_ref: "user:u".into(),
                envelope: MailApiEnvelope::TenantDek {
                    dek_ref: "dek".into(),
                },
                retention_policy_id: "retain".into(),
            },
        )
        .unwrap();

        assert_eq!(response.status_code, 202);
        assert_eq!(response.body.receipt.event_type, "mail.message.submitted");
        assert_eq!(response.body.persistence.statements.len(), 3);
        assert_eq!(
            response.body.persistence.statements[2].name,
            "insert_transactional_outbox_event"
        );
        assert_eq!(
            response.body.protocol_event.binding.proto_rpc,
            "SendMessage"
        );
    }

    #[test]
    fn send_message_rejects_work_context_without_tenant_scope() {
        let mut ctx = context();
        ctx.tenant_id = "person:u".into();
        let result = send_message(
            ctx,
            SubmitMessageRestRequest {
                message_id: "m".into(),
                mailbox_id: "mb".into(),
                subject_ref: "user:u".into(),
                envelope: MailApiEnvelope::TenantDek {
                    dek_ref: "dek".into(),
                },
                retention_policy_id: "retain".into(),
            },
        );
        assert_eq!(
            result,
            Err(MailRestError::Api(MailApiError::MissingTenantScope))
        );
    }

    #[test]
    fn send_message_rejects_missing_policy_decision() {
        let mut ctx = context();
        ctx.policy_decision_ref.clear();
        let result = send_message(
            ctx,
            SubmitMessageRestRequest {
                message_id: "m".into(),
                mailbox_id: "mb".into(),
                subject_ref: "user:u".into(),
                envelope: MailApiEnvelope::TenantDek {
                    dek_ref: "dek".into(),
                },
                retention_policy_id: "retain".into(),
            },
        );
        assert_eq!(
            result,
            Err(MailRestError::Api(MailApiError::MissingPolicyDecision))
        );
    }

    #[test]
    fn mail_submit_message_telemetry_binding_uses_canonical_metric_names() {
        let binding = telemetry_binding().unwrap();

        assert_eq!(binding.microservice, MAIL_REST_MICROSERVICE);
        assert_eq!(binding.operation_id, SUBMIT_MESSAGE_OPERATION_ID);
        assert_eq!(binding.request_total_metric, "oya_mail_request_total");
        assert_eq!(
            binding.request_success_metric,
            "oya_mail_request_success_total"
        );
        assert_eq!(binding.responses_total_metric, "oya_mail_responses_total");
        assert_eq!(binding.responses_5xx_metric, "oya_mail_responses_5xx_total");
        assert_eq!(binding.responses_429_metric, "oya_mail_responses_429_total");
    }

    #[test]
    fn mail_probe_dispatch_reports_liveness_and_readiness() {
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
            dispatch_probe_route("GET", "/mailboxes", Vec::new()),
            Err(ProbeRouteDispatchError::NotProbeRoute {
                method: "GET",
                path: "/mailboxes",
            })
        );
        assert_eq!(
            dispatch_probe_route(PROBE_METHOD, "/does-not-exist", Vec::new()),
            Err(ProbeRouteDispatchError::UnknownRoute)
        );
    }

    #[test]
    fn mail_contract_only_dispatch_returns_501_for_known_unimplemented_route() {
        let response = dispatch_contract_only_route("GET", "/mailboxes").unwrap();

        assert_eq!(response.status_code, 501);
        assert_eq!(response.body.status_code, 501);
        assert_eq!(response.body.method, "GET");
        assert_eq!(response.body.path, "/mailboxes");
        assert_eq!(
            response.body.reason,
            "contract-only route; no runtime handler claim"
        );
    }

    #[test]
    fn mail_contract_only_dispatch_refuses_typed_handler_routes() {
        assert_eq!(
            dispatch_contract_only_route(SUBMIT_MESSAGE_METHOD, SUBMIT_MESSAGE_ROUTE),
            Err(RouteDispatchError::TypedHandlerRequired {
                method: SUBMIT_MESSAGE_METHOD,
                path: SUBMIT_MESSAGE_ROUTE
            })
        );
    }

    #[test]
    fn mail_contract_only_dispatch_rejects_unknown_route() {
        assert_eq!(
            dispatch_contract_only_route("GET", "/does-not-exist"),
            Err(RouteDispatchError::UnknownRoute)
        );
    }

    #[test]
    fn mail_write_route_dispatch_calls_business_handler() {
        let response = dispatch_write_route(
            SUBMIT_MESSAGE_METHOD,
            SUBMIT_MESSAGE_ROUTE,
            context(),
            MailWriteRouteRequest::SubmitMessage(SubmitMessageRestRequest {
                message_id: "m".into(),
                mailbox_id: "mb".into(),
                subject_ref: "user:u".into(),
                envelope: MailApiEnvelope::TenantDek {
                    dek_ref: "dek".into(),
                },
                retention_policy_id: "retain".into(),
            }),
        )
        .unwrap();

        assert_eq!(response.status_code, 202);
        let MailWriteRouteResponse::SubmitMessage(receipt) = response.body;
        assert_eq!(receipt.event_type, "mail.message.submitted");
    }

    #[test]
    fn mail_write_route_dispatch_refuses_contract_only_route() {
        let result = dispatch_write_route(
            "GET",
            "/mailboxes",
            context(),
            MailWriteRouteRequest::SubmitMessage(SubmitMessageRestRequest {
                message_id: "m".into(),
                mailbox_id: "mb".into(),
                subject_ref: "user:u".into(),
                envelope: MailApiEnvelope::TenantDek {
                    dek_ref: "dek".into(),
                },
                retention_policy_id: "retain".into(),
            }),
        );

        assert_eq!(
            result,
            Err(MailWriteRouteDispatchError::ContractOnly {
                method: "GET",
                path: "/mailboxes",
            })
        );
    }
}
