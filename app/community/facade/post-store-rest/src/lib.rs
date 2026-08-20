//! Framework-free REST boundary for community post-store.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use community_post_store_api::{
    AuthorizedCommunityContext, CastVoteRequest, CommunityApiError, CreatePostRequest,
    ModeratePostRequest, ModerationReceipt, PostReceipt, VoteReceiptEnvelope,
};
use community_post_store_app::{
    CommunityAppError, CommunityModerationPlan, CommunityPostPlan, CommunityVotePlan,
    plan_cast_vote, plan_create_post, plan_moderation_action,
};
use community_post_store_domain::{CommunityPost, VoteLedger};
use community_post_store_usecase::{CommunityUsecaseError, cast_vote, create_post, moderate_post};
use oya_shared_hyperscaler_metrics_kernel::{
    MetricsContext, MetricsError, RequestTelemetryBinding,
};
use oya_shared_postgres_command_kernel::TenantSqlContext;

pub const CREATE_POST_ROUTE: &str = "/spaces/{space_id}/posts";
pub const CREATE_POST_METHOD: &str = "POST";
pub const CAST_VOTE_ROUTE: &str = "/posts/{post_id}/vote";
pub const CAST_VOTE_METHOD: &str = "POST";
pub const APPLY_MODERATION_ACTION_ROUTE: &str = "/moderation/actions";
pub const APPLY_MODERATION_ACTION_METHOD: &str = "POST";

pub const HEALTH_ROUTE: &str = "/health";
pub const READY_ROUTE: &str = "/ready";
pub const PROBE_METHOD: &str = "GET";

pub const COMMUNITY_REST_MICROSERVICE: &str = "community";
pub const CREATE_POST_OPERATION_ID: &str = "community.create_post";
pub const CAST_VOTE_OPERATION_ID: &str = "community.cast_vote";
pub const APPLY_MODERATION_ACTION_OPERATION_ID: &str = "community.apply_moderation_action";

pub fn telemetry_bindings() -> Result<[RequestTelemetryBinding; 3], MetricsError> {
    let context = MetricsContext::new(COMMUNITY_REST_MICROSERVICE)?;
    Ok([
        RequestTelemetryBinding::new(&context, CREATE_POST_OPERATION_ID)?,
        RequestTelemetryBinding::new(&context, CAST_VOTE_OPERATION_ID)?,
        RequestTelemetryBinding::new(&context, APPLY_MODERATION_ACTION_OPERATION_ID)?,
    ])
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
        path: "/spaces",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/spaces/{space_id}/posts",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: CREATE_POST_METHOD,
        path: CREATE_POST_ROUTE,
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
        method: "GET",
        path: "/posts/{post_id}/replies",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/posts/{post_id}/replies",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: CAST_VOTE_METHOD,
        path: CAST_VOTE_ROUTE,
        handler_status: RouteHandlerStatus::Implemented,
    },
    OpenApiRoute {
        method: "POST",
        path: "/posts/{post_id}/accept-answer",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/posts/{post_id}/flag",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/moderation/queue",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: APPLY_MODERATION_ACTION_METHOD,
        path: APPLY_MODERATION_ACTION_ROUTE,
        handler_status: RouteHandlerStatus::Implemented,
    },
    OpenApiRoute {
        method: "GET",
        path: "/kb/articles",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/kb/articles",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/kb/articles/{article_id}",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "PATCH",
        path: "/kb/articles/{article_id}",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/kb/articles/{article_id}/publish",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/kb/articles/{article_id}/attachments",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/search",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "GET",
        path: "/subscriptions",
        handler_status: RouteHandlerStatus::ContractOnly,
    },
    OpenApiRoute {
        method: "POST",
        path: "/subscriptions",
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunityRestContext {
    pub tenant_scope_ref: String,
    pub principal_ref: String,
    pub idempotency_key: String,
    pub policy_decision_ref: String,
    pub request_id: String,
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
                microservice: COMMUNITY_REST_MICROSERVICE,
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
                    microservice: COMMUNITY_REST_MICROSERVICE,
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
pub enum CommunityRestError {
    Api(CommunityApiError),
    Usecase(CommunityUsecaseError),
    App(CommunityAppError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunityWriteRouteRequest {
    CreatePost(CreatePostRequest),
    CastVote {
        post: CommunityPost,
        ledger: VoteLedger,
        request: CastVoteRequest,
    },
    ApplyModerationAction {
        post: CommunityPost,
        request: ModeratePostRequest,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunityWriteRouteResponse {
    CreatePost(PostReceipt),
    CastVote(VoteReceiptEnvelope),
    ApplyModerationAction(ModerationReceipt),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunityWriteRouteDispatchError {
    UnknownRoute,
    ContractOnly {
        method: &'static str,
        path: &'static str,
    },
    PayloadMismatch {
        method: &'static str,
        path: &'static str,
    },
    Handler(CommunityRestError),
}

pub fn dispatch_write_route(
    method: &str,
    path: &str,
    context: CommunityRestContext,
    request: CommunityWriteRouteRequest,
) -> Result<RestResponse<CommunityWriteRouteResponse>, CommunityWriteRouteDispatchError> {
    let Some(route) = find_openapi_route(method, path) else {
        return Err(CommunityWriteRouteDispatchError::UnknownRoute);
    };
    if route.handler_status == RouteHandlerStatus::ContractOnly {
        return Err(CommunityWriteRouteDispatchError::ContractOnly {
            method: route.method,
            path: route.path,
        });
    }
    match (route.method, route.path, request) {
        (
            CREATE_POST_METHOD,
            CREATE_POST_ROUTE,
            CommunityWriteRouteRequest::CreatePost(request),
        ) => create_post_from_rest(context, request)
            .map(|response| RestResponse {
                status_code: response.status_code,
                body: CommunityWriteRouteResponse::CreatePost(response.body),
            })
            .map_err(CommunityWriteRouteDispatchError::Handler),
        (
            CAST_VOTE_METHOD,
            CAST_VOTE_ROUTE,
            CommunityWriteRouteRequest::CastVote {
                post,
                mut ledger,
                request,
            },
        ) => cast_vote_from_rest(context, &post, &mut ledger, request)
            .map(|response| RestResponse {
                status_code: response.status_code,
                body: CommunityWriteRouteResponse::CastVote(response.body),
            })
            .map_err(CommunityWriteRouteDispatchError::Handler),
        (
            APPLY_MODERATION_ACTION_METHOD,
            APPLY_MODERATION_ACTION_ROUTE,
            CommunityWriteRouteRequest::ApplyModerationAction { post, request },
        ) => apply_moderation_action(context, &post, request)
            .map(|response| RestResponse {
                status_code: response.status_code,
                body: CommunityWriteRouteResponse::ApplyModerationAction(response.body),
            })
            .map_err(CommunityWriteRouteDispatchError::Handler),
        (method, path, _) => {
            Err(CommunityWriteRouteDispatchError::PayloadMismatch { method, path })
        }
    }
}

pub fn create_post_from_rest(
    context: CommunityRestContext,
    request: CreatePostRequest,
) -> Result<RestResponse<PostReceipt>, CommunityRestError> {
    let api_context = map_context(context);
    api_context.validate().map_err(CommunityRestError::Api)?;
    let (_, receipt) = create_post(&api_context, request).map_err(CommunityRestError::Usecase)?;
    Ok(RestResponse {
        status_code: 201,
        body: receipt,
    })
}

pub fn create_post_write_plan_from_rest(
    tenant: TenantSqlContext,
    context: CommunityRestContext,
    space_id: impl Into<String>,
    request: CreatePostRequest,
) -> Result<RestResponse<CommunityPostPlan>, CommunityRestError> {
    let api_context = map_context(context);
    api_context.validate().map_err(CommunityRestError::Api)?;
    let plan = plan_create_post(tenant, api_context, space_id, request)
        .map_err(CommunityRestError::App)?;
    Ok(RestResponse {
        status_code: 201,
        body: plan,
    })
}

pub fn cast_vote_from_rest(
    context: CommunityRestContext,
    post: &CommunityPost,
    ledger: &mut VoteLedger,
    request: CastVoteRequest,
) -> Result<RestResponse<VoteReceiptEnvelope>, CommunityRestError> {
    let api_context = map_context(context);
    api_context.validate().map_err(CommunityRestError::Api)?;
    let receipt =
        cast_vote(&api_context, post, ledger, request).map_err(CommunityRestError::Usecase)?;
    Ok(RestResponse {
        status_code: 200,
        body: receipt,
    })
}

pub fn cast_vote_write_plan_from_rest(
    tenant: TenantSqlContext,
    context: CommunityRestContext,
    post: &CommunityPost,
    ledger: &mut VoteLedger,
    request: CastVoteRequest,
) -> Result<RestResponse<CommunityVotePlan>, CommunityRestError> {
    let api_context = map_context(context);
    api_context.validate().map_err(CommunityRestError::Api)?;
    let plan = plan_cast_vote(tenant, api_context, post, ledger, request)
        .map_err(CommunityRestError::App)?;
    Ok(RestResponse {
        status_code: 200,
        body: plan,
    })
}

pub fn apply_moderation_action(
    context: CommunityRestContext,
    post: &CommunityPost,
    request: ModeratePostRequest,
) -> Result<RestResponse<ModerationReceipt>, CommunityRestError> {
    let api_context = map_context(context);
    api_context.validate().map_err(CommunityRestError::Api)?;
    let receipt =
        moderate_post(&api_context, post, request).map_err(CommunityRestError::Usecase)?;
    Ok(RestResponse {
        status_code: 201,
        body: receipt,
    })
}

pub fn apply_moderation_action_write_plan(
    tenant: TenantSqlContext,
    context: CommunityRestContext,
    post: &CommunityPost,
    request: ModeratePostRequest,
) -> Result<RestResponse<CommunityModerationPlan>, CommunityRestError> {
    let api_context = map_context(context);
    api_context.validate().map_err(CommunityRestError::Api)?;
    let plan = plan_moderation_action(tenant, api_context, post, request)
        .map_err(CommunityRestError::App)?;
    Ok(RestResponse {
        status_code: 201,
        body: plan,
    })
}

fn map_context(context: CommunityRestContext) -> AuthorizedCommunityContext {
    AuthorizedCommunityContext {
        tenant_scope_ref: context.tenant_scope_ref,
        principal_ref: context.principal_ref,
        idempotency_key: context.idempotency_key,
        policy_decision_ref: context.policy_decision_ref,
        audit_correlation_id: context.request_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use community_post_store_api::{CommunityApiMode, ModerationVerb, VoteDirection};

    fn tenant() -> TenantSqlContext {
        TenantSqlContext::new("tenant:t", "cell-a", "tenant:t#cell-a", "US").unwrap()
    }

    fn context() -> CommunityRestContext {
        CommunityRestContext {
            tenant_scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "policy".into(),
            request_id: "req".into(),
        }
    }

    fn create_req() -> CreatePostRequest {
        CreatePostRequest {
            post_id: "p".into(),
            thread_id: "space:s".into(),
            mode: CommunityApiMode::Teamblind,
            routine_display_ref: "anon".into(),
            audit_author_ref: "user:u".into(),
            disclosure_policy_ref: Some("disclosure".into()),
            body_ref: "body".into(),
            retention_policy_id: "retain".into(),
        }
    }

    #[test]
    fn routes_match_openapi_post_surfaces() {
        assert_eq!(CREATE_POST_METHOD, "POST");
        assert_eq!(CREATE_POST_ROUTE, "/spaces/{space_id}/posts");
        assert_eq!(CAST_VOTE_METHOD, "POST");
        assert_eq!(CAST_VOTE_ROUTE, "/posts/{post_id}/vote");
        assert_eq!(APPLY_MODERATION_ACTION_METHOD, "POST");
        assert_eq!(APPLY_MODERATION_ACTION_ROUTE, "/moderation/actions");
    }

    #[test]
    fn openapi_route_catalog_covers_declared_operations() {
        assert_eq!(OPENAPI_ROUTES.len(), 24);
        assert_eq!(
            find_openapi_route(CREATE_POST_METHOD, CREATE_POST_ROUTE)
                .map(|route| route.handler_status),
            Some(RouteHandlerStatus::Implemented)
        );
        assert_eq!(
            find_openapi_route(CAST_VOTE_METHOD, CAST_VOTE_ROUTE).map(|route| route.handler_status),
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
        assert!(find_openapi_route("POST", "/subscriptions").is_some());
    }

    #[test]
    fn create_post_returns_created_receipt() {
        let response = create_post_from_rest(context(), create_req()).unwrap();
        assert_eq!(response.status_code, 201);
        assert_eq!(response.body.event_type, "community.post.created");
    }

    #[test]
    fn create_post_write_plan_returns_persistence_and_protocol_event() {
        let response =
            create_post_write_plan_from_rest(tenant(), context(), "space:s", create_req()).unwrap();

        assert_eq!(response.status_code, 201);
        assert_eq!(response.body.receipt.event_type, "community.post.created");
        assert_eq!(response.body.persistence.statements.len(), 2);
        assert_eq!(
            response.body.persistence.statements[1].name,
            "insert_transactional_outbox_event"
        );
        assert_eq!(response.body.protocol_event.binding.proto_rpc, "CreatePost");
    }

    #[test]
    fn vote_route_preserves_principal_binding() {
        let api_context = map_context(context());
        let (post, _) = create_post(&api_context, create_req()).unwrap();
        let mut ledger = VoteLedger::new(&post);
        let response = cast_vote_from_rest(
            CommunityRestContext {
                principal_ref: "user:voter".into(),
                idempotency_key: "vote1".into(),
                ..context()
            },
            &post,
            &mut ledger,
            CastVoteRequest {
                post_id: "p".into(),
                voter_ref: "user:voter".into(),
                direction: VoteDirection::Up,
            },
        )
        .unwrap();
        assert_eq!(response.status_code, 200);
        assert_eq!(response.body.event_type, "community.vote.cast");
    }

    #[test]
    fn vote_and_moderation_write_plans_return_persistence_and_protocol_events() {
        let created =
            create_post_write_plan_from_rest(tenant(), context(), "space:s", create_req()).unwrap();
        let mut ledger = VoteLedger::new(&created.body.post);
        let vote = cast_vote_write_plan_from_rest(
            tenant(),
            CommunityRestContext {
                principal_ref: "user:voter".into(),
                idempotency_key: "vote1".into(),
                ..context()
            },
            &created.body.post,
            &mut ledger,
            CastVoteRequest {
                post_id: "p".into(),
                voter_ref: "user:voter".into(),
                direction: VoteDirection::Up,
            },
        )
        .unwrap();
        assert_eq!(vote.status_code, 200);
        assert_eq!(
            vote.body.persistence.statements[0].name,
            "insert_community_vote"
        );
        assert_eq!(
            vote.body.persistence.statements[1].name,
            "insert_transactional_outbox_event"
        );
        assert_eq!(vote.body.protocol_event.binding.proto_rpc, "CastVote");

        let moderation = apply_moderation_action_write_plan(
            tenant(),
            context(),
            &created.body.post,
            ModeratePostRequest {
                policy_ref: "policy".into(),
                evidence_ref: "evidence".into(),
                verb: ModerationVerb::Hide,
            },
        )
        .unwrap();
        assert_eq!(moderation.status_code, 201);
        assert_eq!(
            moderation.body.persistence.statements[0].name,
            "insert_community_moderation_action"
        );
        assert_eq!(
            moderation.body.persistence.statements[1].name,
            "insert_transactional_outbox_event"
        );
        assert_eq!(
            moderation.body.protocol_event.binding.proto_rpc,
            "ApplyAction"
        );
    }

    #[test]
    fn moderation_action_requires_evidence() {
        let api_context = map_context(context());
        let (post, _) = create_post(&api_context, create_req()).unwrap();
        let result = apply_moderation_action(
            context(),
            &post,
            ModeratePostRequest {
                policy_ref: "policy".into(),
                evidence_ref: "".into(),
                verb: ModerationVerb::Hide,
            },
        );
        assert!(matches!(result, Err(CommunityRestError::Usecase(_))));
    }

    #[test]
    fn create_post_rejects_missing_policy_decision() {
        let mut ctx = context();
        ctx.policy_decision_ref.clear();
        let result = create_post_from_rest(ctx, create_req());
        assert_eq!(
            result,
            Err(CommunityRestError::Api(
                CommunityApiError::MissingPolicyDecision
            ))
        );
    }

    #[test]
    fn community_write_routes_have_canonical_telemetry_bindings() {
        let bindings = telemetry_bindings().unwrap();

        assert_eq!(bindings.len(), 3);
        assert_eq!(bindings[0].operation_id, CREATE_POST_OPERATION_ID);
        assert_eq!(bindings[1].operation_id, CAST_VOTE_OPERATION_ID);
        assert_eq!(
            bindings[2].operation_id,
            APPLY_MODERATION_ACTION_OPERATION_ID
        );
        for binding in bindings {
            assert_eq!(binding.microservice, COMMUNITY_REST_MICROSERVICE);
            assert_eq!(binding.request_total_metric, "oya_community_request_total");
            assert_eq!(
                binding.request_success_metric,
                "oya_community_request_success_total"
            );
            assert_eq!(
                binding.responses_total_metric,
                "oya_community_responses_total"
            );
            assert_eq!(
                binding.responses_5xx_metric,
                "oya_community_responses_5xx_total"
            );
            assert_eq!(
                binding.responses_429_metric,
                "oya_community_responses_429_total"
            );
        }
    }

    #[test]
    fn community_probe_dispatch_reports_liveness_and_readiness() {
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
            dispatch_probe_route("GET", "/spaces", Vec::new()),
            Err(ProbeRouteDispatchError::NotProbeRoute {
                method: "GET",
                path: "/spaces",
            })
        );
        assert_eq!(
            dispatch_probe_route(PROBE_METHOD, "/does-not-exist", Vec::new()),
            Err(ProbeRouteDispatchError::UnknownRoute)
        );
    }

    #[test]
    fn community_contract_only_dispatch_returns_501_for_known_unimplemented_route() {
        let response = dispatch_contract_only_route("GET", "/spaces").unwrap();

        assert_eq!(response.status_code, 501);
        assert_eq!(response.body.status_code, 501);
        assert_eq!(response.body.method, "GET");
        assert_eq!(response.body.path, "/spaces");
        assert_eq!(
            response.body.reason,
            "contract-only route; no runtime handler claim"
        );
    }

    #[test]
    fn community_contract_only_dispatch_refuses_typed_handler_routes() {
        assert_eq!(
            dispatch_contract_only_route(CREATE_POST_METHOD, CREATE_POST_ROUTE),
            Err(RouteDispatchError::TypedHandlerRequired {
                method: CREATE_POST_METHOD,
                path: CREATE_POST_ROUTE
            })
        );
    }

    #[test]
    fn community_contract_only_dispatch_rejects_unknown_route() {
        assert_eq!(
            dispatch_contract_only_route("GET", "/does-not-exist"),
            Err(RouteDispatchError::UnknownRoute)
        );
    }

    #[test]
    fn community_write_route_dispatch_calls_business_handlers() {
        let create = dispatch_write_route(
            CREATE_POST_METHOD,
            CREATE_POST_ROUTE,
            context(),
            CommunityWriteRouteRequest::CreatePost(create_req()),
        )
        .unwrap();
        assert_eq!(create.status_code, 201);
        assert!(matches!(
            create.body,
            CommunityWriteRouteResponse::CreatePost(PostReceipt {
                event_type: "community.post.created",
                ..
            })
        ));

        let api_context = map_context(context());
        let (post, _) = create_post(&api_context, create_req()).unwrap();
        let vote = dispatch_write_route(
            CAST_VOTE_METHOD,
            CAST_VOTE_ROUTE,
            CommunityRestContext {
                principal_ref: "user:voter".into(),
                idempotency_key: "vote1".into(),
                ..context()
            },
            CommunityWriteRouteRequest::CastVote {
                post: post.clone(),
                ledger: VoteLedger::new(&post),
                request: CastVoteRequest {
                    post_id: "p".into(),
                    voter_ref: "user:voter".into(),
                    direction: VoteDirection::Up,
                },
            },
        )
        .unwrap();
        assert_eq!(vote.status_code, 200);
        assert!(matches!(
            vote.body,
            CommunityWriteRouteResponse::CastVote(VoteReceiptEnvelope {
                event_type: "community.vote.cast",
                ..
            })
        ));

        let moderation = dispatch_write_route(
            APPLY_MODERATION_ACTION_METHOD,
            APPLY_MODERATION_ACTION_ROUTE,
            context(),
            CommunityWriteRouteRequest::ApplyModerationAction {
                post,
                request: ModeratePostRequest {
                    policy_ref: "policy".into(),
                    evidence_ref: "evidence".into(),
                    verb: ModerationVerb::Hide,
                },
            },
        )
        .unwrap();
        assert_eq!(moderation.status_code, 201);
        assert!(matches!(
            moderation.body,
            CommunityWriteRouteResponse::ApplyModerationAction(ModerationReceipt {
                event_type: "community.moderation.actioned",
                ..
            })
        ));
    }

    #[test]
    fn community_write_route_dispatch_refuses_contract_only_route() {
        let result = dispatch_write_route(
            "GET",
            "/spaces",
            context(),
            CommunityWriteRouteRequest::CreatePost(create_req()),
        );

        assert_eq!(
            result,
            Err(CommunityWriteRouteDispatchError::ContractOnly {
                method: "GET",
                path: "/spaces",
            })
        );
    }
}
