//! Framework-free REST boundary for social post-composition.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use community_social_app::{SocialAppError, SocialPublishPlan, plan_publish_post};
use community_social_post_composition_api::{
    AuthorizedSocialContext, ComposePostRequest, SocialApiArtifactKind, SocialApiContext,
    SocialApiError, SocialPostReceipt,
};
use community_social_post_composition_usecase::{SocialUsecaseError, compose_post};
use oya_shared_hyperscaler_metrics_kernel::{
    MetricsContext, MetricsError, RequestTelemetryBinding,
};
use oya_shared_postgres_command_kernel::TenantSqlContext;

pub const PUBLISH_POST_ROUTE: &str = "/posts";
pub const PUBLISH_POST_METHOD: &str = "POST";

pub const HEALTH_ROUTE: &str = "/health";
pub const READY_ROUTE: &str = "/ready";
pub const PROBE_METHOD: &str = "GET";

pub const SOCIAL_REST_MICROSERVICE: &str = "social";
pub const PUBLISH_POST_OPERATION_ID: &str = "social.publish_post";

pub fn telemetry_binding() -> Result<RequestTelemetryBinding, MetricsError> {
    let context = MetricsContext::new(SOCIAL_REST_MICROSERVICE)?;
    RequestTelemetryBinding::new(&context, PUBLISH_POST_OPERATION_ID)
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
        method: "POST",
        path: "/profiles",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/profiles/me",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "PATCH",
        path: "/profiles/me",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/profiles/{handle}",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/profiles/{handle}/follow",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/profiles/{handle}/unfollow",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/profiles/{handle}/block",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/profiles/{handle}/mute",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: PUBLISH_POST_METHOD,
        path: PUBLISH_POST_ROUTE,
        handler_status: RouteHandlerStatus::Implemented,
    },
    OpenApiRoute {
        method: "GET",
        path: "/posts/{post_id}",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "PATCH",
        path: "/posts/{post_id}",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "DELETE",
        path: "/posts/{post_id}",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/posts/{post_id}/reactions",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "DELETE",
        path: "/posts/{post_id}/reactions/{emoji}",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/posts/{post_id}/bookmark",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/feed",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/search/people",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/search/content",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/trending",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/media",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/notifications",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/reports",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/appeals",
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
pub enum SocialRestContextKind {
    Personal,
    Professional,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SocialRestContext {
    pub scope_org_id: String,
    pub context_kind: SocialRestContextKind,
    pub principal_ref: String,
    pub idempotency_key: String,
    pub policy_decision_ref: String,
    pub request_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishPostRestRequest {
    pub post_id: String,
    pub creator_ref: String,
    pub media_refs: Vec<String>,
    pub kind: SocialApiArtifactKind,
    pub workflow_consent_ref: Option<String>,
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
                microservice: SOCIAL_REST_MICROSERVICE,
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
                    microservice: SOCIAL_REST_MICROSERVICE,
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
pub enum SocialRestError {
    Api(SocialApiError),
    Usecase(SocialUsecaseError),
    App(SocialAppError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SocialWriteRouteRequest {
    PublishPost(PublishPostRestRequest),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SocialWriteRouteResponse {
    PublishPost(SocialPostReceipt),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SocialWriteRouteDispatchError {
    UnknownRoute,
    ContractOnly {
        method: &'static str,
        path: &'static str,
    },
    PayloadMismatch {
        method: &'static str,
        path: &'static str,
    },
    Handler(SocialRestError),
}

pub fn dispatch_write_route(
    method: &str,
    path: &str,
    context: SocialRestContext,
    request: SocialWriteRouteRequest,
) -> Result<RestResponse<SocialWriteRouteResponse>, SocialWriteRouteDispatchError> {
    let Some(route) = find_openapi_route(method, path) else {
        return Err(SocialWriteRouteDispatchError::UnknownRoute);
    };
    if route.handler_status == RouteHandlerStatus::ContractOnly {
        return Err(SocialWriteRouteDispatchError::ContractOnly {
            method: route.method,
            path: route.path,
        });
    }
    match (route.method, route.path, request) {
        (
            PUBLISH_POST_METHOD,
            PUBLISH_POST_ROUTE,
            SocialWriteRouteRequest::PublishPost(request),
        ) => publish_post(context, request)
            .map(|response| RestResponse {
                status_code: response.status_code,
                body: SocialWriteRouteResponse::PublishPost(response.body),
            })
            .map_err(SocialWriteRouteDispatchError::Handler),
        (method, path, _) => Err(SocialWriteRouteDispatchError::PayloadMismatch { method, path }),
    }
}

pub fn publish_post(
    context: SocialRestContext,
    request: PublishPostRestRequest,
) -> Result<RestResponse<SocialPostReceipt>, SocialRestError> {
    let api_context = AuthorizedSocialContext {
        context: map_context(context.context_kind),
        scope_ref: context.scope_org_id,
        principal_ref: context.principal_ref,
        idempotency_key: context.idempotency_key,
        policy_decision_ref: context.policy_decision_ref,
        audit_correlation_id: context.request_id,
    };
    api_context.validate().map_err(SocialRestError::Api)?;
    let (_, receipt) = compose_post(
        &api_context,
        ComposePostRequest {
            post_id: request.post_id,
            creator_ref: request.creator_ref,
            kind: request.kind,
            media_refs: request.media_refs,
            story_expires_at: None,
            collab_owner_refs: vec![],
            collab_consent_refs: vec![],
            workflow_consent_ref: request.workflow_consent_ref,
            ar_biometric_persisted: false,
        },
    )
    .map_err(SocialRestError::Usecase)?;
    Ok(RestResponse {
        status_code: 201,
        body: receipt,
    })
}

pub fn publish_post_write_plan(
    tenant: TenantSqlContext,
    context: SocialRestContext,
    request: PublishPostRestRequest,
    story_purge_now: Option<u64>,
) -> Result<RestResponse<SocialPublishPlan>, SocialRestError> {
    let api_context = social_api_context(context);
    api_context.validate().map_err(SocialRestError::Api)?;
    let plan = plan_publish_post(
        tenant,
        api_context,
        ComposePostRequest {
            post_id: request.post_id,
            creator_ref: request.creator_ref,
            kind: request.kind,
            media_refs: request.media_refs,
            story_expires_at: None,
            collab_owner_refs: vec![],
            collab_consent_refs: vec![],
            workflow_consent_ref: request.workflow_consent_ref,
            ar_biometric_persisted: false,
        },
        story_purge_now,
    )
    .map_err(SocialRestError::App)?;
    Ok(RestResponse {
        status_code: 201,
        body: plan,
    })
}

fn social_api_context(context: SocialRestContext) -> AuthorizedSocialContext {
    AuthorizedSocialContext {
        context: map_context(context.context_kind),
        scope_ref: context.scope_org_id,
        principal_ref: context.principal_ref,
        idempotency_key: context.idempotency_key,
        policy_decision_ref: context.policy_decision_ref,
        audit_correlation_id: context.request_id,
    }
}

fn map_context(context: SocialRestContextKind) -> SocialApiContext {
    match context {
        SocialRestContextKind::Personal => SocialApiContext::Personal,
        SocialRestContextKind::Professional => SocialApiContext::Work,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tenant() -> TenantSqlContext {
        TenantSqlContext::new("person:u", "cell-a", "person:u#cell-a", "US").unwrap()
    }

    fn context() -> SocialRestContext {
        SocialRestContext {
            scope_org_id: "person:u".into(),
            context_kind: SocialRestContextKind::Personal,
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "cedar:allow:post-compose".into(),
            request_id: "req".into(),
        }
    }

    #[test]
    fn route_matches_openapi_publish_post() {
        assert_eq!(PUBLISH_POST_METHOD, "POST");
        assert_eq!(PUBLISH_POST_ROUTE, "/posts");
    }

    #[test]
    fn openapi_route_catalog_covers_declared_operations() {
        assert_eq!(OPENAPI_ROUTES.len(), 27);
        assert_eq!(
            find_openapi_route(PUBLISH_POST_METHOD, PUBLISH_POST_ROUTE)
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
        assert!(find_openapi_route("POST", "/reports").is_some());
    }

    #[test]
    fn publish_post_returns_created_receipt() {
        let response = publish_post(
            context(),
            PublishPostRestRequest {
                post_id: "p".into(),
                creator_ref: "user:u".into(),
                media_refs: vec!["m".into()],
                kind: SocialApiArtifactKind::FeedPost,
                workflow_consent_ref: None,
            },
        )
        .unwrap();
        assert_eq!(response.status_code, 201);
        assert_eq!(response.body.event_type, "social.post.created");
    }

    #[test]
    fn publish_post_write_plan_returns_persistence_and_protocol_event() {
        let response = publish_post_write_plan(
            tenant(),
            context(),
            PublishPostRestRequest {
                post_id: "p".into(),
                creator_ref: "user:u".into(),
                media_refs: vec!["m".into()],
                kind: SocialApiArtifactKind::FeedPost,
                workflow_consent_ref: None,
            },
            None,
        )
        .unwrap();

        assert_eq!(response.status_code, 201);
        assert_eq!(response.body.receipt.event_type, "social.post.created");
        assert_eq!(response.body.persistence.statements.len(), 2);
        assert_eq!(
            response.body.persistence.statements[1].name,
            "insert_transactional_outbox_event"
        );
        assert_eq!(
            response.body.protocol_event.binding.proto_rpc,
            "PublishPost"
        );
    }

    #[test]
    fn professional_publish_requires_workflow_consent() {
        let mut ctx = context();
        ctx.scope_org_id = "tenant:t".into();
        ctx.context_kind = SocialRestContextKind::Professional;
        let result = publish_post(
            ctx,
            PublishPostRestRequest {
                post_id: "p".into(),
                creator_ref: "user:u".into(),
                media_refs: vec!["m".into()],
                kind: SocialApiArtifactKind::FeedPost,
                workflow_consent_ref: None,
            },
        );
        assert!(matches!(result, Err(SocialRestError::Usecase(_))));
    }

    #[test]
    fn publish_post_rejects_missing_policy_decision() {
        let mut ctx = context();
        ctx.policy_decision_ref.clear();
        let result = publish_post(
            ctx,
            PublishPostRestRequest {
                post_id: "p".into(),
                creator_ref: "user:u".into(),
                media_refs: vec!["m".into()],
                kind: SocialApiArtifactKind::FeedPost,
                workflow_consent_ref: None,
            },
        );
        assert_eq!(
            result,
            Err(SocialRestError::Api(SocialApiError::MissingPolicyDecision))
        );
    }

    #[test]
    fn social_publish_post_telemetry_binding_uses_canonical_metric_names() {
        let binding = telemetry_binding().unwrap();

        assert_eq!(binding.microservice, SOCIAL_REST_MICROSERVICE);
        assert_eq!(binding.operation_id, PUBLISH_POST_OPERATION_ID);
        assert_eq!(binding.request_total_metric, "oya_social_request_total");
        assert_eq!(
            binding.request_success_metric,
            "oya_social_request_success_total"
        );
        assert_eq!(binding.responses_total_metric, "oya_social_responses_total");
        assert_eq!(
            binding.responses_5xx_metric,
            "oya_social_responses_5xx_total"
        );
        assert_eq!(
            binding.responses_429_metric,
            "oya_social_responses_429_total"
        );
    }

    #[test]
    fn social_probe_dispatch_reports_liveness_and_readiness() {
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
            dispatch_probe_route("GET", "/profiles/me", Vec::new()),
            Err(ProbeRouteDispatchError::NotProbeRoute {
                method: "GET",
                path: "/profiles/me",
            })
        );
        assert_eq!(
            dispatch_probe_route(PROBE_METHOD, "/does-not-exist", Vec::new()),
            Err(ProbeRouteDispatchError::UnknownRoute)
        );
    }

    #[test]
    fn social_contract_only_dispatch_returns_501_for_known_unimplemented_route() {
        let response = dispatch_contract_only_route("GET", "/profiles/me").unwrap();

        assert_eq!(response.status_code, 501);
        assert_eq!(response.body.status_code, 501);
        assert_eq!(response.body.method, "GET");
        assert_eq!(response.body.path, "/profiles/me");
        assert_eq!(
            response.body.reason,
            "contract-only route; no runtime handler claim"
        );
    }

    #[test]
    fn social_contract_only_dispatch_refuses_typed_handler_routes() {
        assert_eq!(
            dispatch_contract_only_route(PUBLISH_POST_METHOD, PUBLISH_POST_ROUTE),
            Err(RouteDispatchError::TypedHandlerRequired {
                method: PUBLISH_POST_METHOD,
                path: PUBLISH_POST_ROUTE
            })
        );
    }

    #[test]
    fn social_contract_only_dispatch_rejects_unknown_route() {
        assert_eq!(
            dispatch_contract_only_route("GET", "/does-not-exist"),
            Err(RouteDispatchError::UnknownRoute)
        );
    }

    #[test]
    fn social_write_route_dispatch_calls_business_handler() {
        let response = dispatch_write_route(
            PUBLISH_POST_METHOD,
            PUBLISH_POST_ROUTE,
            context(),
            SocialWriteRouteRequest::PublishPost(PublishPostRestRequest {
                post_id: "p".into(),
                creator_ref: "user:u".into(),
                media_refs: vec!["m".into()],
                kind: SocialApiArtifactKind::FeedPost,
                workflow_consent_ref: None,
            }),
        )
        .unwrap();

        assert_eq!(response.status_code, 201);
        let SocialWriteRouteResponse::PublishPost(receipt) = response.body;
        assert_eq!(receipt.event_type, "social.post.created");
    }

    #[test]
    fn social_write_route_dispatch_refuses_contract_only_route() {
        let result = dispatch_write_route(
            "GET",
            "/profiles/me",
            context(),
            SocialWriteRouteRequest::PublishPost(PublishPostRestRequest {
                post_id: "p".into(),
                creator_ref: "user:u".into(),
                media_refs: vec!["m".into()],
                kind: SocialApiArtifactKind::FeedPost,
                workflow_consent_ref: None,
            }),
        );

        assert_eq!(
            result,
            Err(SocialWriteRouteDispatchError::ContractOnly {
                method: "GET",
                path: "/profiles/me",
            })
        );
    }
}
