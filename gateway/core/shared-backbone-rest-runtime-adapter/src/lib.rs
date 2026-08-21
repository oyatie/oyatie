//! Shared REST runtime adapter for the four backbone microservice route catalogs.
//!
//! This crate binds the framework-free REST catalogs for messenger, mail,
//! social, and community into transport-neutral routers that the canonical
//! Hyper runtime adapter can serve without this crate importing Hyper directly.
//! It is intentionally honest about the current seam: probes, contract-only
//! OpenAPI routes, and the stateless typed write routes are
//! runtime-dispatchable. The implemented messenger, mail, social, and community
//! write routes can also be bound to SQL write-plan/outbox command seams with a
//! recording executor for loopback tests. The community vote/moderation paths
//! require an explicitly supplied local read-state object. Those paths still
//! make no live database, broker, cluster, cloud substrate, or production
//! deployment claim.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::{
    collections::BTreeMap,
    fmt::Write as _,
    sync::{Arc, Mutex},
};

use comms_mail_mailbox_api::{DmarcApiAction, MailApiEnvelope};
use comms_mail_mailbox_rest as mail_rest;
use comms_messenger_stream_api::MessengerApiEnvelope;
use comms_messenger_stream_rest as messenger_rest;
use community_post_store_api::{
    CastVoteRequest, CommunityApiMode, CreatePostRequest, ModeratePostRequest, ModerationVerb,
    VoteDirection,
};
use community_post_store_domain::{CommunityPost, VoteLedger};
use community_post_store_rest as community_rest;
use community_social_post_composition_api::SocialApiArtifactKind;
use community_social_post_composition_rest as social_rest;
use http_middleware_kernel::{HttpRequest, HttpResponse, Middleware, MiddlewareChain, Next};
use http_router_kernel::{HttpMethod, Router, RouterError};
use http_runtime_hyper_adapter::{HyperRuntimeError, ServerConfig, serve_listener};
use shared_postgres_command_kernel::{
    RecordingSqlBatchExecutor, SqlBatchExecutor, SqlExecutionPlan, SqlExecutionReport,
    TenantSqlContext,
};
use serde_json::Value;
use tokio::net::TcpListener;

/// Handler shape expected by the canonical Hyper adapter. Kept as a local type
/// alias so this crate does not depend on or import Hyper-family crates.
pub type BackboneRestSyncHandler = Arc<dyn Fn(HttpRequest) -> HttpResponse + Send + Sync>;

/// The four REST-backed microservices in the current backbone slice.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BackboneRestMicroservice {
    Messenger,
    Mail,
    Social,
    Community,
}

impl BackboneRestMicroservice {
    pub fn slug(self) -> &'static str {
        match self {
            BackboneRestMicroservice::Messenger => "messenger",
            BackboneRestMicroservice::Mail => "mail",
            BackboneRestMicroservice::Social => "social",
            BackboneRestMicroservice::Community => "community",
        }
    }
}

pub const ALL_BACKBONE_REST_MICROSERVICES: [BackboneRestMicroservice; 4] = [
    BackboneRestMicroservice::Messenger,
    BackboneRestMicroservice::Mail,
    BackboneRestMicroservice::Social,
    BackboneRestMicroservice::Community,
];

/// Shared readiness-dependency shape accepted by the runtime adapter. Each
/// service REST crate has an equivalent type; this adapter converts at the
/// boundary to keep the public runtime composition API service-neutral.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackboneRestReadinessDependency {
    pub name: &'static str, // data_class: INTERNAL_ONLY
    pub ready: bool,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeRouteHandlerKind {
    Probe,
    ContractOnly,
    JsonWrite,
    SqlWritePlan,
    TypedWritePlan,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackboneRestRuntimeRoute {
    pub microservice: BackboneRestMicroservice,
    pub method: &'static str,
    pub path: &'static str,
    pub handler_kind: RuntimeRouteHandlerKind,
}

#[derive(Clone, Default)]
pub struct BackboneRestRuntimeState {
    stateless_sql: Option<BackboneSqlWriteState>,
    community: Option<BackboneCommunityJsonState>,
    community_sql: Option<BackboneCommunitySqlWriteState>,
}

impl BackboneRestRuntimeState {
    pub fn without_state() -> Self {
        Self::default()
    }

    pub fn with_community_state(community: BackboneCommunityJsonState) -> Self {
        Self {
            stateless_sql: None,
            community: Some(community),
            community_sql: None,
        }
    }

    pub fn with_community_sql_write_state(community_sql: BackboneCommunitySqlWriteState) -> Self {
        Self {
            stateless_sql: None,
            community: None,
            community_sql: Some(community_sql),
        }
    }

    pub fn with_stateless_sql_write_state(stateless_sql: BackboneSqlWriteState) -> Self {
        Self {
            stateless_sql: Some(stateless_sql),
            community: None,
            community_sql: None,
        }
    }

    fn stateless_sql_write_state(&self) -> Option<&BackboneSqlWriteState> {
        self.stateless_sql.as_ref()
    }

    fn community_state(&self) -> Option<&BackboneCommunityJsonState> {
        self.community.as_ref()
    }

    fn community_sql_write_state(&self) -> Option<&BackboneCommunitySqlWriteState> {
        self.community_sql.as_ref()
    }

    fn handler_kind_for(&self, route: BackboneRestRuntimeRoute) -> RuntimeRouteHandlerKind {
        if self.binds_stateless_sql_write_route(route)
            || self.binds_community_sql_write_route(route)
        {
            RuntimeRouteHandlerKind::SqlWritePlan
        } else if self.binds_stateful_community_route(route) {
            RuntimeRouteHandlerKind::JsonWrite
        } else {
            route.handler_kind
        }
    }

    fn binds_stateless_sql_write_route(&self, route: BackboneRestRuntimeRoute) -> bool {
        self.stateless_sql.is_some()
            && matches!(
                (route.microservice, route.method, route.path),
                (
                    BackboneRestMicroservice::Messenger,
                    messenger_rest::POST_MESSAGE_METHOD,
                    messenger_rest::POST_MESSAGE_ROUTE
                ) | (
                    BackboneRestMicroservice::Mail,
                    mail_rest::SUBMIT_MESSAGE_METHOD,
                    mail_rest::SUBMIT_MESSAGE_ROUTE
                ) | (
                    BackboneRestMicroservice::Social,
                    social_rest::PUBLISH_POST_METHOD,
                    social_rest::PUBLISH_POST_ROUTE
                )
            )
    }

    fn binds_community_sql_write_route(&self, route: BackboneRestRuntimeRoute) -> bool {
        self.community_sql.is_some()
            && route.microservice == BackboneRestMicroservice::Community
            && matches!(
                (route.method, route.path),
                (
                    community_rest::CREATE_POST_METHOD,
                    community_rest::CREATE_POST_ROUTE
                ) | (
                    community_rest::CAST_VOTE_METHOD,
                    community_rest::CAST_VOTE_ROUTE
                ) | (
                    community_rest::APPLY_MODERATION_ACTION_METHOD,
                    community_rest::APPLY_MODERATION_ACTION_ROUTE
                )
            )
    }

    fn binds_stateful_community_route(&self, route: BackboneRestRuntimeRoute) -> bool {
        self.community.is_some()
            && route.microservice == BackboneRestMicroservice::Community
            && route.handler_kind == RuntimeRouteHandlerKind::TypedWritePlan
            && matches!(
                (route.method, route.path),
                (
                    community_rest::CAST_VOTE_METHOD,
                    community_rest::CAST_VOTE_ROUTE
                ) | (
                    community_rest::APPLY_MODERATION_ACTION_METHOD,
                    community_rest::APPLY_MODERATION_ACTION_ROUTE
                )
            )
    }
}

/// Recording SQL executor state for stateless backbone write routes. This
/// exercises tenant-scoped SQL/outbox command plans without opening a live
/// Postgres connection, publishing a broker message, or claiming deployment.
#[derive(Clone, Default)]
pub struct BackboneSqlWriteState {
    executor: Arc<Mutex<RecordingSqlBatchExecutor>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BackboneSqlWriteStateError {
    Poisoned,
}

impl std::fmt::Display for BackboneSqlWriteStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackboneSqlWriteStateError::Poisoned => {
                write!(f, "stateless SQL write state lock is poisoned")
            }
        }
    }
}

impl std::error::Error for BackboneSqlWriteStateError {}

impl BackboneSqlWriteState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execution_reports(&self) -> Result<Vec<SqlExecutionReport>, BackboneSqlWriteStateError> {
        self.executor
            .lock()
            .map(|executor| executor.reports.clone())
            .map_err(|_| BackboneSqlWriteStateError::Poisoned)
    }
}

#[derive(Clone, Default)]
pub struct BackboneCommunityJsonState {
    inner: Arc<Mutex<BackboneCommunityJsonStateInner>>,
}

#[derive(Default)]
struct BackboneCommunityJsonStateInner {
    posts: BTreeMap<String, CommunityPost>,
    vote_ledgers: BTreeMap<String, VoteLedger>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BackboneCommunityJsonStateError {
    Poisoned,
}

impl std::fmt::Display for BackboneCommunityJsonStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackboneCommunityJsonStateError::Poisoned => {
                write!(f, "community JSON state lock is poisoned")
            }
        }
    }
}

impl std::error::Error for BackboneCommunityJsonStateError {}

impl BackboneCommunityJsonState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn seed_post(&self, post: CommunityPost) -> Result<(), BackboneCommunityJsonStateError> {
        let post_id = post.post_id.value.clone();
        let ledger = VoteLedger::new(&post);
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| BackboneCommunityJsonStateError::Poisoned)?;
        inner.posts.insert(post_id.clone(), post);
        inner.vote_ledgers.entry(post_id).or_insert(ledger);
        Ok(())
    }

    pub fn post_count(&self) -> Result<usize, BackboneCommunityJsonStateError> {
        self.inner
            .lock()
            .map(|inner| inner.posts.len())
            .map_err(|_| BackboneCommunityJsonStateError::Poisoned)
    }
}

/// Local community state plus a recording SQL executor for loopback write-plan
/// tests. The backing post/ledger store is still in-memory; the durable seam
/// being exercised is the generated tenant-scoped SQL/outbox command plan, not
/// a live Postgres connection.
#[derive(Clone, Default)]
pub struct BackboneCommunitySqlWriteState {
    community: BackboneCommunityJsonState,
    executor: Arc<Mutex<RecordingSqlBatchExecutor>>,
}

impl BackboneCommunitySqlWriteState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn seed_post(&self, post: CommunityPost) -> Result<(), BackboneCommunityJsonStateError> {
        self.community.seed_post(post)
    }

    pub fn post_count(&self) -> Result<usize, BackboneCommunityJsonStateError> {
        self.community.post_count()
    }

    pub fn execution_reports(
        &self,
    ) -> Result<Vec<SqlExecutionReport>, BackboneCommunityJsonStateError> {
        self.executor
            .lock()
            .map(|executor| executor.reports.clone())
            .map_err(|_| BackboneCommunityJsonStateError::Poisoned)
    }
}

/// A router plus the counts auditors need to understand what is really bound.
pub struct BackboneRestRouter {
    pub microservice: BackboneRestMicroservice,
    pub router: Router<BackboneRestSyncHandler>,
    pub route_count: usize,
    pub probe_route_count: usize,
    pub contract_only_route_count: usize,
    pub json_write_route_count: usize,
    pub sql_write_plan_route_count: usize,
    pub typed_write_plan_route_count: usize,
    pub non_claim: &'static str,
}

#[derive(Debug)]
pub enum BackboneRestRuntimeError {
    UnsupportedMethod {
        microservice: BackboneRestMicroservice,
        method: &'static str,
        path: &'static str,
    },
    RouteRegistration {
        microservice: BackboneRestMicroservice,
        method: &'static str,
        path: &'static str,
        reason: String,
    },
    Hyper(HyperRuntimeError),
}

impl std::fmt::Display for BackboneRestRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackboneRestRuntimeError::UnsupportedMethod {
                microservice,
                method,
                path,
            } => write!(
                f,
                "{} REST route {method} {path} uses an unsupported HTTP method",
                microservice.slug()
            ),
            BackboneRestRuntimeError::RouteRegistration {
                microservice,
                method,
                path,
                reason,
            } => write!(
                f,
                "{} REST route {method} {path} failed router registration: {reason}",
                microservice.slug()
            ),
            BackboneRestRuntimeError::Hyper(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for BackboneRestRuntimeError {}

impl From<HyperRuntimeError> for BackboneRestRuntimeError {
    fn from(error: HyperRuntimeError) -> Self {
        Self::Hyper(error)
    }
}

// ---------------------------------------------------------------------------
// Authz port — fail-closed verified-principal layer (AUTH-005)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackboneVerifiedPrincipal {
    pub principal_ref: String,
    pub policy_decision_ref: String,
}

impl BackboneVerifiedPrincipal {
    pub fn new(principal_ref: impl Into<String>, policy_decision_ref: impl Into<String>) -> Self {
        Self {
            principal_ref: principal_ref.into(),
            policy_decision_ref: policy_decision_ref.into(),
        }
    }

    fn is_complete(&self) -> bool {
        !self.principal_ref.trim().is_empty() && !self.policy_decision_ref.trim().is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackboneAuthzRequest {
    pub bearer: String,
    pub microservice: Option<BackboneRestMicroservice>,
    pub method: String,
    pub path: String,
    pub matched_template: String,
    pub action: String,
    pub resource: String,
    pub tenant_ref: Option<String>,
    pub context_kind: Option<String>,
    pub caller_principal_ref: Option<String>,
    pub caller_policy_decision_ref: Option<String>,
    pub path_captures: BTreeMap<String, String>,
}

/// Authz port that callers MUST supply to build a fail-closed middleware chain.
///
/// Implement this to perform constant-time bearer comparison and PDP resolution.
/// Absence of a provider means no non-probe request can proceed (default-deny).
///
/// Reference doctrine: tenancy/facade/tenant-lifecycle-app/src/lib.rs
/// (authenticate_caller + constant-time bearer + authorize() per route).
pub trait BackboneAuthzProvider: Send + Sync {
    /// Verify the bearer token and authorize the concrete route request.
    ///
    /// The request carries PBAC inputs while preserving RBAC/ABAC facts: route
    /// action/resource, tenant/context attributes, and untrusted caller-supplied
    /// authorization hints. Implementations must authenticate the bearer with a
    /// constant-time comparison and perform a PDP decision over this full request.
    ///
    /// Returns a verified principal + policy decision on success.
    /// Returns `Err(401)` for a missing or invalid credential.
    /// Returns `Err(403)` for an authenticated but unauthorized caller.
    fn verify_and_authorize(
        &self,
        request: &BackboneAuthzRequest,
    ) -> Result<BackboneVerifiedPrincipal, u16>;
}

/// Fail-closed verified-principal middleware. Probe paths (/health, /ready) are exempt.
/// All other paths require a valid bearer; absent bearer → 401; provider denial → 403.
/// authn-before-body-parse: body bytes are never accessed if the bearer check fails.
struct BackboneAuthzMiddleware {
    provider: Arc<dyn BackboneAuthzProvider>,
    microservice: Option<BackboneRestMicroservice>,
}

impl Middleware<HttpRequest, HttpResponse> for BackboneAuthzMiddleware {
    fn handle(
        &self,
        request: HttpRequest,
        next: Next<'_, HttpRequest, HttpResponse>,
    ) -> HttpResponse {
        // Probe paths are load-balancer health checks; exempt from bearer auth.
        if backbone_is_probe_path(&request.path) {
            return next.run(request);
        }
        // authn-before-body-parse: body bytes not accessed before this check.
        let bearer = match backbone_extract_bearer(&request) {
            Some(b) => b.to_string(),
            None => {
                return backbone_authn_response(
                    401,
                    "missing_bearer",
                    "Authorization: Bearer header is required",
                );
            }
        };
        let authz_request = backbone_authz_request(self.microservice, &bearer, &request);
        // Constant-time comparison and PDP decision are the provider's responsibility.
        match self.provider.verify_and_authorize(&authz_request) {
            Ok(principal) if principal.is_complete() => {
                let mut verified_request = request;
                backbone_install_verified_authz_headers(&mut verified_request, &principal);
                next.run(verified_request)
            }
            Ok(_) => backbone_authn_response(
                403,
                "authz_denied",
                "verified principal and policy decision are required",
            ),
            Err(status) => {
                backbone_authn_response(status, "authz_denied", "request is not authorized")
            }
        }
    }
}

/// Build a fail-closed authz middleware chain requiring a caller-supplied verified-principal
/// provider. Probe paths (/health, /ready) pass through without bearer checks. There is no
/// default-allow: a server that omits this chain has no authz on non-probe routes.
pub fn backbone_authz_chain(
    provider: Arc<dyn BackboneAuthzProvider>,
) -> MiddlewareChain<HttpRequest, HttpResponse> {
    MiddlewareChain::new().push(Box::new(BackboneAuthzMiddleware {
        provider,
        microservice: None,
    }))
}

/// Build a fail-closed authz chain with the serving microservice bound into the PBAC request.
pub fn backbone_authz_chain_for_microservice(
    microservice: BackboneRestMicroservice,
    provider: Arc<dyn BackboneAuthzProvider>,
) -> MiddlewareChain<HttpRequest, HttpResponse> {
    MiddlewareChain::new().push(Box::new(BackboneAuthzMiddleware {
        provider,
        microservice: Some(microservice),
    }))
}

/// Return the runtime binding catalog for one backbone REST service.
pub fn route_runtime_catalog(
    microservice: BackboneRestMicroservice,
) -> Vec<BackboneRestRuntimeRoute> {
    match microservice {
        BackboneRestMicroservice::Messenger => messenger_rest::OPENAPI_ROUTES
            .iter()
            .map(|route| BackboneRestRuntimeRoute {
                microservice,
                method: route.method,
                path: route.path,
                handler_kind: classify_messenger_route(route),
            })
            .collect(),
        BackboneRestMicroservice::Mail => mail_rest::OPENAPI_ROUTES
            .iter()
            .map(|route| BackboneRestRuntimeRoute {
                microservice,
                method: route.method,
                path: route.path,
                handler_kind: classify_mail_route(route),
            })
            .collect(),
        BackboneRestMicroservice::Social => social_rest::OPENAPI_ROUTES
            .iter()
            .map(|route| BackboneRestRuntimeRoute {
                microservice,
                method: route.method,
                path: route.path,
                handler_kind: classify_social_route(route),
            })
            .collect(),
        BackboneRestMicroservice::Community => community_rest::OPENAPI_ROUTES
            .iter()
            .map(|route| BackboneRestRuntimeRoute {
                microservice,
                method: route.method,
                path: route.path,
                handler_kind: classify_community_route(route),
            })
            .collect(),
    }
}

/// Build a Hyper-runtime-compatible router for a service's whole OpenAPI route
/// catalog. Every route is registered; response semantics remain honest:
/// probes dispatch to the service probe handler, contract-only routes return
/// 501, stateless write routes parse JSON and call the service-owned typed
/// dispatcher, and stateful write-plan routes return a typed-plan-required 501.
pub fn build_backbone_rest_router(
    microservice: BackboneRestMicroservice,
    dependencies: Vec<BackboneRestReadinessDependency>,
) -> Result<BackboneRestRouter, BackboneRestRuntimeError> {
    build_backbone_rest_router_with_state(
        microservice,
        dependencies,
        BackboneRestRuntimeState::without_state(),
    )
}

pub fn build_backbone_rest_router_with_state(
    microservice: BackboneRestMicroservice,
    dependencies: Vec<BackboneRestReadinessDependency>,
    runtime_state: BackboneRestRuntimeState,
) -> Result<BackboneRestRouter, BackboneRestRuntimeError> {
    let routes = route_runtime_catalog(microservice)
        .into_iter()
        .map(|mut route| {
            route.handler_kind = runtime_state.handler_kind_for(route);
            route
        })
        .collect::<Vec<_>>();
    let mut router: Router<BackboneRestSyncHandler> = Router::new();
    let mut probe_route_count = 0;
    let mut contract_only_route_count = 0;
    let mut json_write_route_count = 0;
    let mut sql_write_plan_route_count = 0;
    let mut typed_write_plan_route_count = 0;

    for route in routes.iter().copied() {
        match route.handler_kind {
            RuntimeRouteHandlerKind::Probe => probe_route_count += 1,
            RuntimeRouteHandlerKind::ContractOnly => contract_only_route_count += 1,
            RuntimeRouteHandlerKind::JsonWrite => json_write_route_count += 1,
            RuntimeRouteHandlerKind::SqlWritePlan => sql_write_plan_route_count += 1,
            RuntimeRouteHandlerKind::TypedWritePlan => typed_write_plan_route_count += 1,
        }
        let method =
            HttpMethod::parse(route.method).ok_or(BackboneRestRuntimeError::UnsupportedMethod {
                microservice,
                method: route.method,
                path: route.path,
            })?;
        let route_dependencies = dependencies.clone();
        let route_state = runtime_state.clone();
        let handler: BackboneRestSyncHandler = Arc::new(move |request: HttpRequest| {
            // verify_principal: bearer-presence guard, authn-before-body-parse.
            // Probe routes (health/ready) are exempt; all write and contract-only routes require
            // a bearer token even when the chain is empty_middleware_chain() (test dispatch).
            // Full PDP decision lives in BackboneAuthzMiddleware for the serve_* path.
            if route.handler_kind != RuntimeRouteHandlerKind::Probe
                && let Err(resp) = verify_principal(&request)
            {
                return resp;
            }
            dispatch_runtime_route(route, &route_dependencies, &route_state, &request)
        });
        router
            .route(method, route.path, handler)
            .map_err(|error| route_registration_error(route, error))?;
    }

    Ok(BackboneRestRouter {
        microservice,
        router,
        route_count: routes.len(),
        probe_route_count,
        contract_only_route_count,
        json_write_route_count,
        sql_write_plan_route_count,
        typed_write_plan_route_count,
        non_claim: if runtime_state.stateless_sql_write_state().is_some()
            && matches!(
                microservice,
                BackboneRestMicroservice::Messenger
                    | BackboneRestMicroservice::Mail
                    | BackboneRestMicroservice::Social
            ) {
            "local Hyper loopback/runtime binding with stateless SQL write-plan recording executor; no live database, broker drain, production gateway, TLS, OpenCost, ArgoCD sync, cloud substrate, or live deployment claim"
        } else if microservice == BackboneRestMicroservice::Community
            && runtime_state.community_sql_write_state().is_some()
        {
            "local Hyper loopback/runtime binding with community SQL write-plan recording executor; no live database, broker drain, production gateway, TLS, OpenCost, ArgoCD sync, cloud substrate, or live deployment claim"
        } else if microservice == BackboneRestMicroservice::Community
            && runtime_state.community_state().is_some()
        {
            "local Hyper loopback/runtime binding with local in-memory community state only; no production gateway, TLS, durable database, broker, OpenCost, ArgoCD sync, or live deployment claim"
        } else {
            "local Hyper loopback/runtime binding only; no production gateway, TLS, database, broker, or stateful write-route claim"
        },
    })
}

/// Empty middleware chain — no-op pass-through.
///
/// ponytail: test-only. Production serves MUST use `backbone_authz_chain(provider)` instead.
/// Using this chain on a live server leaves all backbone non-probe routes unauthenticated.
pub fn empty_middleware_chain() -> MiddlewareChain<HttpRequest, HttpResponse> {
    MiddlewareChain::new()
}

/// Serve one backbone REST microservice using an already-bound listener.
///
/// `authz_provider` is REQUIRED — the serving chain is fail-closed. Probe paths
/// (/health, /ready) are exempt; all other routes require a verified bearer token.
/// Use [`backbone_authz_chain`] + [`dispatch_backbone_rest_request`] for test dispatch.
pub async fn serve_backbone_rest_microservice_listener(
    listener: TcpListener,
    microservice: BackboneRestMicroservice,
    dependencies: Vec<BackboneRestReadinessDependency>,
    authz_provider: Arc<dyn BackboneAuthzProvider>,
    config: ServerConfig,
) -> Result<(), BackboneRestRuntimeError> {
    let built = build_backbone_rest_router(microservice, dependencies)?;
    serve_listener(
        listener,
        Arc::new(built.router),
        Arc::new(backbone_authz_chain_for_microservice(
            microservice,
            authz_provider,
        )),
        config,
    )
    .await
    .map_err(BackboneRestRuntimeError::Hyper)
}

/// Dispatch through a backbone REST router without binding a network socket.
/// The Hyper adapter uses the same router, request, response, and middleware
/// types at its outer boundary.
pub fn dispatch_backbone_rest_request(
    request: HttpRequest,
    router: &Router<BackboneRestSyncHandler>,
    chain: &MiddlewareChain<HttpRequest, HttpResponse>,
) -> HttpResponse {
    let (handler, captures, template) = match router.match_route(request.method, &request.path) {
        Some(triple) => triple,
        None => return HttpResponse::not_found(),
    };
    let mut req_with_captures = request;
    req_with_captures.path_captures = captures;
    req_with_captures.matched_template = Some(template.to_string());
    let handler_arc = handler.clone();
    chain.execute(req_with_captures, move |req| handler_arc(req))
}

fn dispatch_runtime_route(
    route: BackboneRestRuntimeRoute,
    dependencies: &[BackboneRestReadinessDependency],
    runtime_state: &BackboneRestRuntimeState,
    request: &HttpRequest,
) -> HttpResponse {
    match route.handler_kind {
        RuntimeRouteHandlerKind::Probe => dispatch_probe_route(route, dependencies),
        RuntimeRouteHandlerKind::ContractOnly => contract_only_response(route, request),
        RuntimeRouteHandlerKind::JsonWrite => {
            dispatch_json_write_route(route, request, runtime_state)
        }
        RuntimeRouteHandlerKind::SqlWritePlan => {
            dispatch_sql_write_plan_route(route, request, runtime_state)
        }
        RuntimeRouteHandlerKind::TypedWritePlan => typed_write_plan_response(route, request),
    }
}

fn dispatch_probe_route(
    route: BackboneRestRuntimeRoute,
    dependencies: &[BackboneRestReadinessDependency],
) -> HttpResponse {
    match route.microservice {
        BackboneRestMicroservice::Messenger => match messenger_rest::dispatch_probe_route(
            route.method,
            route.path,
            messenger_dependencies(dependencies),
        ) {
            Ok(response) => probe_response(route, response.status_code, dependencies.len()),
            Err(error) => probe_error_response(route, format!("{error:?}")),
        },
        BackboneRestMicroservice::Mail => match mail_rest::dispatch_probe_route(
            route.method,
            route.path,
            mail_dependencies(dependencies),
        ) {
            Ok(response) => probe_response(route, response.status_code, dependencies.len()),
            Err(error) => probe_error_response(route, format!("{error:?}")),
        },
        BackboneRestMicroservice::Social => match social_rest::dispatch_probe_route(
            route.method,
            route.path,
            social_dependencies(dependencies),
        ) {
            Ok(response) => probe_response(route, response.status_code, dependencies.len()),
            Err(error) => probe_error_response(route, format!("{error:?}")),
        },
        BackboneRestMicroservice::Community => match community_rest::dispatch_probe_route(
            route.method,
            route.path,
            community_dependencies(dependencies),
        ) {
            Ok(response) => probe_response(route, response.status_code, dependencies.len()),
            Err(error) => probe_error_response(route, format!("{error:?}")),
        },
    }
}

fn probe_response(
    route: BackboneRestRuntimeRoute,
    status_code: u16,
    dependency_count: usize,
) -> HttpResponse {
    let body = format!(
        "{{\"microservice\":\"{}\",\"method\":\"{}\",\"path\":\"{}\",\"runtime_handler\":\"probe\",\"status_code\":{},\"dependency_count\":{},\"non_claim\":\"process/readiness route only; no live deployment probe claim\"}}",
        route.microservice.slug(),
        route.method,
        route.path,
        status_code,
        dependency_count
    );
    json_response(status_code, body)
}

fn probe_error_response(route: BackboneRestRuntimeRoute, reason: String) -> HttpResponse {
    let body = format!(
        "{{\"microservice\":\"{}\",\"method\":\"{}\",\"path\":\"{}\",\"runtime_handler\":\"probe\",\"status_code\":500,\"reason\":\"{}\"}}",
        route.microservice.slug(),
        route.method,
        route.path,
        json_string_escape(&reason)
    );
    json_response(500, body)
}

fn contract_only_response(route: BackboneRestRuntimeRoute, request: &HttpRequest) -> HttpResponse {
    let body = format!(
        "{{\"microservice\":\"{}\",\"method\":\"{}\",\"path\":\"{}\",\"matched_template\":\"{}\",\"runtime_handler\":\"contract_only\",\"status_code\":501,\"reason\":\"contract-only OpenAPI route; no runtime handler claim\"}}",
        route.microservice.slug(),
        route.method,
        route.path,
        json_string_escape(request.matched_template.as_deref().unwrap_or(route.path))
    );
    json_response(501, body)
}

#[derive(Debug)]
enum JsonWriteBindingError {
    MissingHeader {
        name: &'static str,
    },
    MissingPathCapture {
        name: &'static str,
    },
    BodyNotJson {
        reason: String,
    },
    BodyNotObject,
    MissingField {
        name: &'static str,
    },
    InvalidField {
        name: &'static str,
        expected: &'static str,
    },
    PathBodyDrift {
        field: &'static str,
        path_value: String,
        body_value: String,
    },
    StateUnavailable {
        path: &'static str,
    },
    StatePoisoned,
    NotFound {
        resource: &'static str,
        id: String,
    },
    Handler {
        reason: String,
    },
    SqlPlan {
        reason: String,
    },
    SqlExecution {
        reason: String,
    },
}

impl JsonWriteBindingError {
    fn status_code(&self) -> u16 {
        match self {
            JsonWriteBindingError::Handler { .. } => 422,
            JsonWriteBindingError::SqlPlan { .. } => 422,
            JsonWriteBindingError::SqlExecution { .. } => 500,
            JsonWriteBindingError::StateUnavailable { .. } => 501,
            JsonWriteBindingError::StatePoisoned => 500,
            JsonWriteBindingError::NotFound { .. } => 404,
            _ => 400,
        }
    }

    fn code(&self) -> &'static str {
        match self {
            JsonWriteBindingError::MissingHeader { .. } => "missing_header",
            JsonWriteBindingError::MissingPathCapture { .. } => "missing_path_capture",
            JsonWriteBindingError::BodyNotJson { .. } => "invalid_json",
            JsonWriteBindingError::BodyNotObject => "body_not_object",
            JsonWriteBindingError::MissingField { .. } => "missing_field",
            JsonWriteBindingError::InvalidField { .. } => "invalid_field",
            JsonWriteBindingError::PathBodyDrift { .. } => "path_body_drift",
            JsonWriteBindingError::StateUnavailable { .. } => "state_unavailable",
            JsonWriteBindingError::StatePoisoned => "state_poisoned",
            JsonWriteBindingError::NotFound { .. } => "not_found",
            JsonWriteBindingError::Handler { .. } => "handler_rejected",
            JsonWriteBindingError::SqlPlan { .. } => "sql_plan_rejected",
            JsonWriteBindingError::SqlExecution { .. } => "sql_execution_failed",
        }
    }

    fn detail(&self) -> String {
        match self {
            JsonWriteBindingError::MissingHeader { name } => {
                format!("required header `{name}` is missing or empty")
            }
            JsonWriteBindingError::MissingPathCapture { name } => {
                format!("required path capture `{name}` is missing")
            }
            JsonWriteBindingError::BodyNotJson { reason } => {
                format!("request body is not valid JSON: {reason}")
            }
            JsonWriteBindingError::BodyNotObject => {
                "request body must be a JSON object".to_string()
            }
            JsonWriteBindingError::MissingField { name } => {
                format!("required JSON field `{name}` is missing")
            }
            JsonWriteBindingError::InvalidField { name, expected } => {
                format!("JSON field `{name}` must be {expected}")
            }
            JsonWriteBindingError::PathBodyDrift {
                field,
                path_value,
                body_value,
            } => format!(
                "JSON field `{field}` value `{body_value}` does not match path value `{path_value}`"
            ),
            JsonWriteBindingError::StateUnavailable { path } => {
                format!("route `{path}` requires explicitly supplied local state")
            }
            JsonWriteBindingError::StatePoisoned => {
                "local community JSON state lock is poisoned".to_string()
            }
            JsonWriteBindingError::NotFound { resource, id } => {
                format!("{resource} `{id}` was not found in local community JSON state")
            }
            JsonWriteBindingError::Handler { reason } => {
                format!("typed REST handler rejected the request: {reason}")
            }
            JsonWriteBindingError::SqlPlan { reason } => {
                format!("SQL write-plan construction rejected the request: {reason}")
            }
            JsonWriteBindingError::SqlExecution { reason } => {
                format!("SQL write-plan execution failed: {reason}")
            }
        }
    }
}

fn community_state_lock(
    state: &BackboneCommunityJsonState,
) -> Result<std::sync::MutexGuard<'_, BackboneCommunityJsonStateInner>, JsonWriteBindingError> {
    state
        .inner
        .lock()
        .map_err(|_| JsonWriteBindingError::StatePoisoned)
}

fn community_sql_state_lock(
    state: &BackboneCommunitySqlWriteState,
) -> Result<std::sync::MutexGuard<'_, BackboneCommunityJsonStateInner>, JsonWriteBindingError> {
    state
        .community
        .inner
        .lock()
        .map_err(|_| JsonWriteBindingError::StatePoisoned)
}

fn dispatch_json_write_route(
    route: BackboneRestRuntimeRoute,
    request: &HttpRequest,
    runtime_state: &BackboneRestRuntimeState,
) -> HttpResponse {
    match route.microservice {
        BackboneRestMicroservice::Messenger => dispatch_messenger_json_write(route, request),
        BackboneRestMicroservice::Mail => dispatch_mail_json_write(route, request),
        BackboneRestMicroservice::Social => dispatch_social_json_write(route, request),
        BackboneRestMicroservice::Community => {
            dispatch_community_json_write(route, request, runtime_state.community_state())
        }
    }
    .unwrap_or_else(|error| json_write_error_response(route, request, error))
}

fn dispatch_sql_write_plan_route(
    route: BackboneRestRuntimeRoute,
    request: &HttpRequest,
    runtime_state: &BackboneRestRuntimeState,
) -> HttpResponse {
    let result = match route.microservice {
        BackboneRestMicroservice::Messenger => runtime_state
            .stateless_sql_write_state()
            .ok_or(JsonWriteBindingError::StateUnavailable { path: route.path })
            .and_then(|state| dispatch_messenger_sql_write_plan(route, request, state)),
        BackboneRestMicroservice::Mail => runtime_state
            .stateless_sql_write_state()
            .ok_or(JsonWriteBindingError::StateUnavailable { path: route.path })
            .and_then(|state| dispatch_mail_sql_write_plan(route, request, state)),
        BackboneRestMicroservice::Social => runtime_state
            .stateless_sql_write_state()
            .ok_or(JsonWriteBindingError::StateUnavailable { path: route.path })
            .and_then(|state| dispatch_social_sql_write_plan(route, request, state)),
        BackboneRestMicroservice::Community => runtime_state
            .community_sql_write_state()
            .ok_or(JsonWriteBindingError::StateUnavailable { path: route.path })
            .and_then(|state| dispatch_community_sql_write_plan(route, request, state)),
    };
    result.unwrap_or_else(|error| sql_write_plan_error_response(route, request, error))
}

fn dispatch_messenger_json_write(
    route: BackboneRestRuntimeRoute,
    request: &HttpRequest,
) -> Result<HttpResponse, JsonWriteBindingError> {
    let body = json_body(request)?;
    let context = messenger_context(request)?;
    let rest_request = messenger_post_message_rest_request(request, &body)?;
    let response = messenger_rest::dispatch_write_route(
        route.method,
        route.path,
        context,
        messenger_rest::MessengerWriteRouteRequest::PostMessage(rest_request),
    )
    .map_err(|error| JsonWriteBindingError::Handler {
        reason: format!("{error:?}"),
    })?;
    match response.body {
        messenger_rest::MessengerWriteRouteResponse::PostMessage(receipt) => {
            Ok(json_write_success_response(JsonWriteReceipt {
                route,
                status_code: response.status_code,
                resource_field: "message_id",
                resource_id: &receipt.message_id,
                event_type: receipt.event_type,
                audit_correlation_id: &receipt.audit_correlation_id,
                idempotency_key: &receipt.idempotency_key,
                policy_decision_ref: &receipt.policy_decision_ref,
                extra: Some(("channel_id", receipt.channel_id.as_str())),
                non_claim: STATELESS_JSON_WRITE_NON_CLAIM,
            }))
        }
    }
}

fn dispatch_mail_json_write(
    route: BackboneRestRuntimeRoute,
    request: &HttpRequest,
) -> Result<HttpResponse, JsonWriteBindingError> {
    let body = json_body(request)?;
    let context = mail_context(request)?;
    let rest_request = mail_submit_message_rest_request(&body)?;
    let response = mail_rest::dispatch_write_route(
        route.method,
        route.path,
        context,
        mail_rest::MailWriteRouteRequest::SubmitMessage(rest_request),
    )
    .map_err(|error| JsonWriteBindingError::Handler {
        reason: format!("{error:?}"),
    })?;
    match response.body {
        mail_rest::MailWriteRouteResponse::SubmitMessage(receipt) => {
            Ok(json_write_success_response(JsonWriteReceipt {
                route,
                status_code: response.status_code,
                resource_field: "message_id",
                resource_id: &receipt.message_id,
                event_type: receipt.event_type,
                audit_correlation_id: &receipt.audit_correlation_id,
                idempotency_key: &receipt.idempotency_key,
                policy_decision_ref: &receipt.policy_decision_ref,
                extra: Some(("dmarc_action", dmarc_action_name(receipt.dmarc_action))),
                non_claim: STATELESS_JSON_WRITE_NON_CLAIM,
            }))
        }
    }
}

fn dispatch_social_json_write(
    route: BackboneRestRuntimeRoute,
    request: &HttpRequest,
) -> Result<HttpResponse, JsonWriteBindingError> {
    let body = json_body(request)?;
    let context = social_context(request)?;
    let rest_request = social_publish_post_rest_request(&body)?;
    let response = social_rest::dispatch_write_route(
        route.method,
        route.path,
        context,
        social_rest::SocialWriteRouteRequest::PublishPost(rest_request),
    )
    .map_err(|error| JsonWriteBindingError::Handler {
        reason: format!("{error:?}"),
    })?;
    match response.body {
        social_rest::SocialWriteRouteResponse::PublishPost(receipt) => {
            Ok(json_write_success_response(JsonWriteReceipt {
                route,
                status_code: response.status_code,
                resource_field: "post_id",
                resource_id: &receipt.post_id,
                event_type: receipt.event_type,
                audit_correlation_id: &receipt.audit_correlation_id,
                idempotency_key: &receipt.idempotency_key,
                policy_decision_ref: &receipt.policy_decision_ref,
                extra: None,
                non_claim: STATELESS_JSON_WRITE_NON_CLAIM,
            }))
        }
    }
}

fn messenger_post_message_rest_request(
    request: &HttpRequest,
    body: &Value,
) -> Result<messenger_rest::PostMessageRestRequest, JsonWriteBindingError> {
    let channel_id = path_capture(request, "channel_id")?;
    reject_path_body_drift(body, "channel_id", &channel_id)?;
    Ok(messenger_rest::PostMessageRestRequest {
        channel_id,
        message_id: required_string(body, "message_id")?,
        author_ref: required_string(body, "author_ref")?,
        envelope: messenger_envelope(required_value(body, "envelope")?)?,
        retention_policy_id: required_string(body, "retention_policy_id")?,
        legal_hold_ids: string_array_or_empty(body, "legal_hold_ids")?,
    })
}

fn mail_submit_message_rest_request(
    body: &Value,
) -> Result<mail_rest::SubmitMessageRestRequest, JsonWriteBindingError> {
    Ok(mail_rest::SubmitMessageRestRequest {
        message_id: required_string(body, "message_id")?,
        mailbox_id: required_string(body, "mailbox_id")?,
        subject_ref: required_string(body, "subject_ref")?,
        envelope: mail_envelope(required_value(body, "envelope")?)?,
        retention_policy_id: required_string(body, "retention_policy_id")?,
    })
}

fn social_publish_post_rest_request(
    body: &Value,
) -> Result<social_rest::PublishPostRestRequest, JsonWriteBindingError> {
    Ok(social_rest::PublishPostRestRequest {
        post_id: required_string(body, "post_id")?,
        creator_ref: required_string(body, "creator_ref")?,
        media_refs: string_array_or_empty(body, "media_refs")?,
        kind: social_artifact_kind(&required_string(body, "kind")?)?,
        workflow_consent_ref: optional_string(body, "workflow_consent_ref")?,
    })
}

fn dispatch_messenger_sql_write_plan(
    route: BackboneRestRuntimeRoute,
    request: &HttpRequest,
    state: &BackboneSqlWriteState,
) -> Result<HttpResponse, JsonWriteBindingError> {
    let body = json_body(request)?;
    let context = messenger_context(request)?;
    let tenant = tenant_sql_context(request)?;
    let rest_request = messenger_post_message_rest_request(request, &body)?;
    let response = messenger_rest::post_message_write_plan(tenant, context, rest_request).map_err(
        |error| JsonWriteBindingError::SqlPlan {
            reason: format!("{error:?}"),
        },
    )?;
    let report = execute_sql_write_plan(state, &response.body.sql_execution)?;

    Ok(sql_write_plan_success_response(SqlWritePlanReceipt {
        route,
        status_code: response.status_code,
        resource_field: "message_id",
        resource_id: &response.body.receipt.message_id,
        event_type: response.body.receipt.event_type,
        audit_correlation_id: &response.body.receipt.audit_correlation_id,
        idempotency_key: &response.body.receipt.idempotency_key,
        policy_decision_ref: &response.body.receipt.policy_decision_ref,
        extra: Some(("channel_id", response.body.receipt.channel_id.as_str())),
        sql_report: &report,
        non_claim: SQL_WRITE_PLAN_NON_CLAIM,
    }))
}

fn dispatch_mail_sql_write_plan(
    route: BackboneRestRuntimeRoute,
    request: &HttpRequest,
    state: &BackboneSqlWriteState,
) -> Result<HttpResponse, JsonWriteBindingError> {
    let body = json_body(request)?;
    let context = mail_context(request)?;
    let tenant = tenant_sql_context(request)?;
    let rest_request = mail_submit_message_rest_request(&body)?;
    let response =
        mail_rest::send_message_write_plan(tenant, context, rest_request).map_err(|error| {
            JsonWriteBindingError::SqlPlan {
                reason: format!("{error:?}"),
            }
        })?;
    let report = execute_sql_write_plan(state, &response.body.sql_execution)?;

    Ok(sql_write_plan_success_response(SqlWritePlanReceipt {
        route,
        status_code: response.status_code,
        resource_field: "message_id",
        resource_id: &response.body.receipt.message_id,
        event_type: response.body.receipt.event_type,
        audit_correlation_id: &response.body.receipt.audit_correlation_id,
        idempotency_key: &response.body.receipt.idempotency_key,
        policy_decision_ref: &response.body.receipt.policy_decision_ref,
        extra: Some((
            "dmarc_action",
            dmarc_action_name(response.body.receipt.dmarc_action),
        )),
        sql_report: &report,
        non_claim: SQL_WRITE_PLAN_NON_CLAIM,
    }))
}

fn dispatch_social_sql_write_plan(
    route: BackboneRestRuntimeRoute,
    request: &HttpRequest,
    state: &BackboneSqlWriteState,
) -> Result<HttpResponse, JsonWriteBindingError> {
    let body = json_body(request)?;
    let context = social_context(request)?;
    let tenant = tenant_sql_context(request)?;
    let rest_request = social_publish_post_rest_request(&body)?;
    let response = social_rest::publish_post_write_plan(tenant, context, rest_request, None)
        .map_err(|error| JsonWriteBindingError::SqlPlan {
            reason: format!("{error:?}"),
        })?;
    let report = execute_sql_write_plan(state, &response.body.sql_execution)?;

    Ok(sql_write_plan_success_response(SqlWritePlanReceipt {
        route,
        status_code: response.status_code,
        resource_field: "post_id",
        resource_id: &response.body.receipt.post_id,
        event_type: response.body.receipt.event_type,
        audit_correlation_id: &response.body.receipt.audit_correlation_id,
        idempotency_key: &response.body.receipt.idempotency_key,
        policy_decision_ref: &response.body.receipt.policy_decision_ref,
        extra: None,
        sql_report: &report,
        non_claim: SQL_WRITE_PLAN_NON_CLAIM,
    }))
}

fn dispatch_community_json_write(
    route: BackboneRestRuntimeRoute,
    request: &HttpRequest,
    community_state: Option<&BackboneCommunityJsonState>,
) -> Result<HttpResponse, JsonWriteBindingError> {
    match (route.method, route.path) {
        (community_rest::CREATE_POST_METHOD, community_rest::CREATE_POST_ROUTE) => {
            dispatch_community_create_post_json_write(route, request)
        }
        (community_rest::CAST_VOTE_METHOD, community_rest::CAST_VOTE_ROUTE) => {
            let state = community_state
                .ok_or(JsonWriteBindingError::StateUnavailable { path: route.path })?;
            dispatch_community_vote_json_write(route, request, state)
        }
        (
            community_rest::APPLY_MODERATION_ACTION_METHOD,
            community_rest::APPLY_MODERATION_ACTION_ROUTE,
        ) => {
            let state = community_state
                .ok_or(JsonWriteBindingError::StateUnavailable { path: route.path })?;
            dispatch_community_moderation_json_write(route, request, state)
        }
        _ => Err(JsonWriteBindingError::StateUnavailable { path: route.path }),
    }
}

fn dispatch_community_create_post_json_write(
    route: BackboneRestRuntimeRoute,
    request: &HttpRequest,
) -> Result<HttpResponse, JsonWriteBindingError> {
    let body = json_body(request)?;
    let space_id = path_capture(request, "space_id")?;
    reject_path_body_drift(&body, "space_id", &space_id)?;
    let context = community_context(request)?;
    let rest_request = CreatePostRequest {
        post_id: required_string(&body, "post_id")?,
        thread_id: required_string(&body, "thread_id")?,
        mode: community_mode(&required_string(&body, "mode")?)?,
        routine_display_ref: required_string(&body, "routine_display_ref")?,
        audit_author_ref: required_string(&body, "audit_author_ref")?,
        disclosure_policy_ref: optional_string(&body, "disclosure_policy_ref")?,
        body_ref: required_string(&body, "body_ref")?,
        retention_policy_id: required_string(&body, "retention_policy_id")?,
    };
    let response = community_rest::dispatch_write_route(
        route.method,
        route.path,
        context,
        community_rest::CommunityWriteRouteRequest::CreatePost(rest_request),
    )
    .map_err(|error| JsonWriteBindingError::Handler {
        reason: format!("{error:?}"),
    })?;
    match response.body {
        community_rest::CommunityWriteRouteResponse::CreatePost(receipt) => {
            Ok(json_write_success_response(JsonWriteReceipt {
                route,
                status_code: response.status_code,
                resource_field: "post_id",
                resource_id: &receipt.post_id,
                event_type: receipt.event_type,
                audit_correlation_id: &receipt.audit_correlation_id,
                idempotency_key: &receipt.idempotency_key,
                policy_decision_ref: &receipt.policy_decision_ref,
                extra: Some(("space_id", space_id.as_str())),
                non_claim: STATELESS_JSON_WRITE_NON_CLAIM,
            }))
        }
        community_rest::CommunityWriteRouteResponse::CastVote(_)
        | community_rest::CommunityWriteRouteResponse::ApplyModerationAction(_) => {
            unreachable!("stateful community routes are not classified as JsonWrite")
        }
    }
}

fn dispatch_community_vote_json_write(
    route: BackboneRestRuntimeRoute,
    request: &HttpRequest,
    state: &BackboneCommunityJsonState,
) -> Result<HttpResponse, JsonWriteBindingError> {
    let body = json_body(request)?;
    let post_id = path_capture(request, "post_id")?;
    reject_path_body_drift(&body, "post_id", &post_id)?;
    let context = community_context(request)?;
    let audit_correlation_id = context.request_id.clone();
    let idempotency_key = context.idempotency_key.clone();
    let rest_request = CastVoteRequest {
        post_id: post_id.clone(),
        voter_ref: required_string(&body, "voter_ref")?,
        direction: vote_direction(&required_string(&body, "direction")?)?,
    };
    let response =
        {
            let mut inner = community_state_lock(state)?;
            let post = inner.posts.get(&post_id).cloned().ok_or_else(|| {
                JsonWriteBindingError::NotFound {
                    resource: "community post",
                    id: post_id.clone(),
                }
            })?;
            let ledger = inner
                .vote_ledgers
                .entry(post_id.clone())
                .or_insert_with(|| VoteLedger::new(&post));
            community_rest::cast_vote_from_rest(context, &post, ledger, rest_request)
        }
        .map_err(|error| JsonWriteBindingError::Handler {
            reason: format!("{error:?}"),
        })?;

    Ok(json_write_success_response(JsonWriteReceipt {
        route,
        status_code: response.status_code,
        resource_field: "post_id",
        resource_id: &response.body.post_id,
        event_type: response.body.event_type,
        audit_correlation_id: &audit_correlation_id,
        idempotency_key: &idempotency_key,
        policy_decision_ref: &response.body.policy_decision_ref,
        extra: Some(("vote_id", response.body.vote_id.as_str())),
        non_claim: STATEFUL_COMMUNITY_JSON_WRITE_NON_CLAIM,
    }))
}

fn dispatch_community_moderation_json_write(
    route: BackboneRestRuntimeRoute,
    request: &HttpRequest,
    state: &BackboneCommunityJsonState,
) -> Result<HttpResponse, JsonWriteBindingError> {
    let body = json_body(request)?;
    let target_type = required_string(&body, "target_type")?;
    if normalized_token(&target_type) != "post" {
        return Err(JsonWriteBindingError::InvalidField {
            name: "target_type",
            expected: "post for the current local community state binding",
        });
    }
    let post_id = match optional_string(&body, "post_id")? {
        Some(post_id) => post_id,
        None => required_string(&body, "target_id")?,
    };
    let context = community_context(request)?;
    let audit_correlation_id = context.request_id.clone();
    let idempotency_key = context.idempotency_key.clone();
    let rest_request = ModeratePostRequest {
        policy_ref: required_string(&body, "policy_ref")?,
        evidence_ref: required_string(&body, "evidence_ref")?,
        verb: moderation_verb(&required_string(&body, "verb")?)?,
    };
    let response =
        {
            let inner = community_state_lock(state)?;
            let post = inner.posts.get(&post_id).cloned().ok_or_else(|| {
                JsonWriteBindingError::NotFound {
                    resource: "community post",
                    id: post_id.clone(),
                }
            })?;
            community_rest::apply_moderation_action(context, &post, rest_request)
        }
        .map_err(|error| JsonWriteBindingError::Handler {
            reason: format!("{error:?}"),
        })?;

    Ok(json_write_success_response(JsonWriteReceipt {
        route,
        status_code: response.status_code,
        resource_field: "post_id",
        resource_id: &response.body.post_id,
        event_type: response.body.event_type,
        audit_correlation_id: &audit_correlation_id,
        idempotency_key: &idempotency_key,
        policy_decision_ref: &response.body.policy_decision_ref,
        extra: Some(("evidence_ref", response.body.evidence_ref.as_str())),
        non_claim: STATEFUL_COMMUNITY_JSON_WRITE_NON_CLAIM,
    }))
}

fn dispatch_community_sql_write_plan(
    route: BackboneRestRuntimeRoute,
    request: &HttpRequest,
    state: &BackboneCommunitySqlWriteState,
) -> Result<HttpResponse, JsonWriteBindingError> {
    match (route.method, route.path) {
        (community_rest::CREATE_POST_METHOD, community_rest::CREATE_POST_ROUTE) => {
            dispatch_community_create_post_sql_write_plan(route, request, state)
        }
        (community_rest::CAST_VOTE_METHOD, community_rest::CAST_VOTE_ROUTE) => {
            dispatch_community_vote_sql_write_plan(route, request, state)
        }
        (
            community_rest::APPLY_MODERATION_ACTION_METHOD,
            community_rest::APPLY_MODERATION_ACTION_ROUTE,
        ) => dispatch_community_moderation_sql_write_plan(route, request, state),
        _ => Err(JsonWriteBindingError::StateUnavailable { path: route.path }),
    }
}

fn dispatch_community_create_post_sql_write_plan(
    route: BackboneRestRuntimeRoute,
    request: &HttpRequest,
    state: &BackboneCommunitySqlWriteState,
) -> Result<HttpResponse, JsonWriteBindingError> {
    let body = json_body(request)?;
    let space_id = path_capture(request, "space_id")?;
    reject_path_body_drift(&body, "space_id", &space_id)?;
    let context = community_context(request)?;
    let tenant = community_tenant_context(request)?;
    let rest_request = CreatePostRequest {
        post_id: required_string(&body, "post_id")?,
        thread_id: required_string(&body, "thread_id")?,
        mode: community_mode(&required_string(&body, "mode")?)?,
        routine_display_ref: required_string(&body, "routine_display_ref")?,
        audit_author_ref: required_string(&body, "audit_author_ref")?,
        disclosure_policy_ref: optional_string(&body, "disclosure_policy_ref")?,
        body_ref: required_string(&body, "body_ref")?,
        retention_policy_id: required_string(&body, "retention_policy_id")?,
    };
    let response = community_rest::create_post_write_plan_from_rest(
        tenant,
        context,
        space_id.clone(),
        rest_request,
    )
    .map_err(|error| JsonWriteBindingError::SqlPlan {
        reason: format!("{error:?}"),
    })?;
    let report = execute_community_sql_write_plan(state, &response.body.sql_execution)?;
    state
        .seed_post(response.body.post.clone())
        .map_err(|_| JsonWriteBindingError::StatePoisoned)?;

    Ok(sql_write_plan_success_response(SqlWritePlanReceipt {
        route,
        status_code: response.status_code,
        resource_field: "post_id",
        resource_id: &response.body.receipt.post_id,
        event_type: response.body.receipt.event_type,
        audit_correlation_id: &response.body.receipt.audit_correlation_id,
        idempotency_key: &response.body.receipt.idempotency_key,
        policy_decision_ref: &response.body.receipt.policy_decision_ref,
        extra: Some(("space_id", space_id.as_str())),
        sql_report: &report,
        non_claim: SQL_WRITE_PLAN_NON_CLAIM,
    }))
}

fn dispatch_community_vote_sql_write_plan(
    route: BackboneRestRuntimeRoute,
    request: &HttpRequest,
    state: &BackboneCommunitySqlWriteState,
) -> Result<HttpResponse, JsonWriteBindingError> {
    let body = json_body(request)?;
    let post_id = path_capture(request, "post_id")?;
    reject_path_body_drift(&body, "post_id", &post_id)?;
    let context = community_context(request)?;
    let tenant = community_tenant_context(request)?;
    let audit_correlation_id = context.request_id.clone();
    let idempotency_key = context.idempotency_key.clone();
    let rest_request = CastVoteRequest {
        post_id: post_id.clone(),
        voter_ref: required_string(&body, "voter_ref")?,
        direction: vote_direction(&required_string(&body, "direction")?)?,
    };
    let response = {
        let mut inner = community_sql_state_lock(state)?;
        let post =
            inner
                .posts
                .get(&post_id)
                .cloned()
                .ok_or_else(|| JsonWriteBindingError::NotFound {
                    resource: "community post",
                    id: post_id.clone(),
                })?;
        let ledger = inner
            .vote_ledgers
            .entry(post_id.clone())
            .or_insert_with(|| VoteLedger::new(&post));
        community_rest::cast_vote_write_plan_from_rest(tenant, context, &post, ledger, rest_request)
    }
    .map_err(|error| JsonWriteBindingError::SqlPlan {
        reason: format!("{error:?}"),
    })?;
    let report = execute_community_sql_write_plan(state, &response.body.sql_execution)?;

    Ok(sql_write_plan_success_response(SqlWritePlanReceipt {
        route,
        status_code: response.status_code,
        resource_field: "post_id",
        resource_id: &response.body.receipt.post_id,
        event_type: response.body.receipt.event_type,
        audit_correlation_id: &audit_correlation_id,
        idempotency_key: &idempotency_key,
        policy_decision_ref: &response.body.receipt.policy_decision_ref,
        extra: Some(("vote_id", response.body.receipt.vote_id.as_str())),
        sql_report: &report,
        non_claim: SQL_WRITE_PLAN_NON_CLAIM,
    }))
}

fn dispatch_community_moderation_sql_write_plan(
    route: BackboneRestRuntimeRoute,
    request: &HttpRequest,
    state: &BackboneCommunitySqlWriteState,
) -> Result<HttpResponse, JsonWriteBindingError> {
    let body = json_body(request)?;
    let target_type = required_string(&body, "target_type")?;
    if normalized_token(&target_type) != "post" {
        return Err(JsonWriteBindingError::InvalidField {
            name: "target_type",
            expected: "post for the current local community read-state binding",
        });
    }
    let post_id = match optional_string(&body, "post_id")? {
        Some(post_id) => post_id,
        None => required_string(&body, "target_id")?,
    };
    let context = community_context(request)?;
    let tenant = community_tenant_context(request)?;
    let audit_correlation_id = context.request_id.clone();
    let idempotency_key = context.idempotency_key.clone();
    let rest_request = ModeratePostRequest {
        policy_ref: required_string(&body, "policy_ref")?,
        evidence_ref: required_string(&body, "evidence_ref")?,
        verb: moderation_verb(&required_string(&body, "verb")?)?,
    };
    let response =
        {
            let inner = community_sql_state_lock(state)?;
            let post = inner.posts.get(&post_id).cloned().ok_or_else(|| {
                JsonWriteBindingError::NotFound {
                    resource: "community post",
                    id: post_id.clone(),
                }
            })?;
            community_rest::apply_moderation_action_write_plan(tenant, context, &post, rest_request)
        }
        .map_err(|error| JsonWriteBindingError::SqlPlan {
            reason: format!("{error:?}"),
        })?;
    let report = execute_community_sql_write_plan(state, &response.body.sql_execution)?;

    Ok(sql_write_plan_success_response(SqlWritePlanReceipt {
        route,
        status_code: response.status_code,
        resource_field: "post_id",
        resource_id: &response.body.receipt.post_id,
        event_type: response.body.receipt.event_type,
        audit_correlation_id: &audit_correlation_id,
        idempotency_key: &idempotency_key,
        policy_decision_ref: &response.body.receipt.policy_decision_ref,
        extra: Some(("evidence_ref", response.body.receipt.evidence_ref.as_str())),
        sql_report: &report,
        non_claim: SQL_WRITE_PLAN_NON_CLAIM,
    }))
}

fn execute_community_sql_write_plan(
    state: &BackboneCommunitySqlWriteState,
    plan: &SqlExecutionPlan,
) -> Result<SqlExecutionReport, JsonWriteBindingError> {
    execute_recorded_sql_write_plan(&state.executor, plan)
}

fn execute_sql_write_plan(
    state: &BackboneSqlWriteState,
    plan: &SqlExecutionPlan,
) -> Result<SqlExecutionReport, JsonWriteBindingError> {
    execute_recorded_sql_write_plan(&state.executor, plan)
}

fn execute_recorded_sql_write_plan(
    executor: &Arc<Mutex<RecordingSqlBatchExecutor>>,
    plan: &SqlExecutionPlan,
) -> Result<SqlExecutionReport, JsonWriteBindingError> {
    executor
        .lock()
        .map_err(|_| JsonWriteBindingError::StatePoisoned)?
        .execute_batch(plan)
        .map_err(|error| JsonWriteBindingError::SqlExecution {
            reason: format!("{error:?}"),
        })
}

struct JsonWriteReceipt<'a> {
    route: BackboneRestRuntimeRoute,
    status_code: u16,
    resource_field: &'a str,
    resource_id: &'a str,
    event_type: &'a str,
    audit_correlation_id: &'a str,
    idempotency_key: &'a str,
    policy_decision_ref: &'a str,
    extra: Option<(&'a str, &'a str)>,
    non_claim: &'a str,
}

struct SqlWritePlanReceipt<'a> {
    route: BackboneRestRuntimeRoute,
    status_code: u16,
    resource_field: &'a str,
    resource_id: &'a str,
    event_type: &'a str,
    audit_correlation_id: &'a str,
    idempotency_key: &'a str,
    policy_decision_ref: &'a str,
    extra: Option<(&'a str, &'a str)>,
    sql_report: &'a SqlExecutionReport,
    non_claim: &'a str,
}

const STATELESS_JSON_WRITE_NON_CLAIM: &str =
    "local stateless write handler only; no database, broker, or live deployment claim";
const STATEFUL_COMMUNITY_JSON_WRITE_NON_CLAIM: &str = "local in-memory state handler only; no durable database, broker, cluster, OpenCost, ArgoCD sync, or live deployment claim";
const SQL_WRITE_PLAN_NON_CLAIM: &str = "local SQL write-plan binding with recording executor only; no live database, broker drain, cluster, OpenCost, ArgoCD sync, cloud substrate, or live deployment claim";

fn json_write_success_response(receipt: JsonWriteReceipt<'_>) -> HttpResponse {
    let mut body = format!(
        "{{\"microservice\":\"{}\",\"method\":\"{}\",\"path\":\"{}\",\"runtime_handler\":\"json_write\",\"status_code\":{},\"{}\":\"{}\",\"event_type\":\"{}\",\"audit_correlation_id\":\"{}\",\"idempotency_key\":\"{}\",\"policy_decision_ref\":\"{}\",\"non_claim\":\"{}\"",
        receipt.route.microservice.slug(),
        receipt.route.method,
        receipt.route.path,
        receipt.status_code,
        json_string_escape(receipt.resource_field),
        json_string_escape(receipt.resource_id),
        json_string_escape(receipt.event_type),
        json_string_escape(receipt.audit_correlation_id),
        json_string_escape(receipt.idempotency_key),
        json_string_escape(receipt.policy_decision_ref),
        json_string_escape(receipt.non_claim)
    );
    if let Some((name, value)) = receipt.extra {
        body.push_str(&format!(
            ",\"{}\":\"{}\"",
            json_string_escape(name),
            json_string_escape(value)
        ));
    }
    body.push('}');
    json_response(receipt.status_code, body)
}

fn sql_write_plan_success_response(receipt: SqlWritePlanReceipt<'_>) -> HttpResponse {
    let mut body = format!(
        "{{\"microservice\":\"{}\",\"method\":\"{}\",\"path\":\"{}\",\"runtime_handler\":\"sql_write_plan\",\"status_code\":{},\"{}\":\"{}\",\"event_type\":\"{}\",\"audit_correlation_id\":\"{}\",\"idempotency_key\":\"{}\",\"policy_decision_ref\":\"{}\",\"application_name\":\"{}\",\"sql_command_count\":{},\"executed_command_names\":{},\"transaction_committed\":{},\"outbox_statement\":\"insert_transactional_outbox_event\",\"non_claim\":\"{}\"",
        receipt.route.microservice.slug(),
        receipt.route.method,
        receipt.route.path,
        receipt.status_code,
        json_string_escape(receipt.resource_field),
        json_string_escape(receipt.resource_id),
        json_string_escape(receipt.event_type),
        json_string_escape(receipt.audit_correlation_id),
        json_string_escape(receipt.idempotency_key),
        json_string_escape(receipt.policy_decision_ref),
        json_string_escape(&receipt.sql_report.application_name),
        receipt.sql_report.executed_command_names.len(),
        json_string_array(&receipt.sql_report.executed_command_names),
        receipt.sql_report.transaction_committed,
        json_string_escape(receipt.non_claim)
    );
    if let Some((name, value)) = receipt.extra {
        body.push_str(&format!(
            ",\"{}\":\"{}\"",
            json_string_escape(name),
            json_string_escape(value)
        ));
    }
    body.push('}');
    json_response(receipt.status_code, body)
}

fn json_write_error_response(
    route: BackboneRestRuntimeRoute,
    request: &HttpRequest,
    error: JsonWriteBindingError,
) -> HttpResponse {
    let status_code = error.status_code();
    let body = format!(
        "{{\"microservice\":\"{}\",\"method\":\"{}\",\"path\":\"{}\",\"matched_template\":\"{}\",\"runtime_handler\":\"json_write\",\"status_code\":{},\"error_code\":\"{}\",\"reason\":\"{}\"}}",
        route.microservice.slug(),
        route.method,
        route.path,
        json_string_escape(request.matched_template.as_deref().unwrap_or(route.path)),
        status_code,
        error.code(),
        json_string_escape(&error.detail())
    );
    json_response(status_code, body)
}

fn sql_write_plan_error_response(
    route: BackboneRestRuntimeRoute,
    request: &HttpRequest,
    error: JsonWriteBindingError,
) -> HttpResponse {
    let status_code = error.status_code();
    let body = format!(
        "{{\"microservice\":\"{}\",\"method\":\"{}\",\"path\":\"{}\",\"matched_template\":\"{}\",\"runtime_handler\":\"sql_write_plan\",\"status_code\":{},\"error_code\":\"{}\",\"reason\":\"{}\"}}",
        route.microservice.slug(),
        route.method,
        route.path,
        json_string_escape(request.matched_template.as_deref().unwrap_or(route.path)),
        status_code,
        error.code(),
        json_string_escape(&error.detail())
    );
    json_response(status_code, body)
}

fn typed_write_plan_response(
    route: BackboneRestRuntimeRoute,
    request: &HttpRequest,
) -> HttpResponse {
    let body = format!(
        "{{\"microservice\":\"{}\",\"method\":\"{}\",\"path\":\"{}\",\"matched_template\":\"{}\",\"runtime_handler\":\"typed_write_plan_required\",\"status_code\":501,\"reason\":\"stateful typed write route requires backing read/write state; generic JSON binding is intentionally not claimed for this route\"}}",
        route.microservice.slug(),
        route.method,
        route.path,
        json_string_escape(request.matched_template.as_deref().unwrap_or(route.path))
    );
    json_response(501, body)
}

fn json_body(request: &HttpRequest) -> Result<Value, JsonWriteBindingError> {
    let value: Value = serde_json::from_slice(&request.body).map_err(|error| {
        JsonWriteBindingError::BodyNotJson {
            reason: error.to_string(),
        }
    })?;
    if value.is_object() {
        Ok(value)
    } else {
        Err(JsonWriteBindingError::BodyNotObject)
    }
}

fn required_value<'a>(
    body: &'a Value,
    name: &'static str,
) -> Result<&'a Value, JsonWriteBindingError> {
    body.get(name)
        .ok_or(JsonWriteBindingError::MissingField { name })
}

fn required_string(body: &Value, name: &'static str) -> Result<String, JsonWriteBindingError> {
    let value = required_value(body, name)?;
    match value.as_str().map(str::trim) {
        Some(value) if !value.is_empty() => Ok(value.to_string()),
        _ => Err(JsonWriteBindingError::InvalidField {
            name,
            expected: "a non-empty string",
        }),
    }
}

fn optional_string(
    body: &Value,
    name: &'static str,
) -> Result<Option<String>, JsonWriteBindingError> {
    match body.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => match value.as_str().map(str::trim) {
            Some(value) if !value.is_empty() => Ok(Some(value.to_string())),
            _ => Err(JsonWriteBindingError::InvalidField {
                name,
                expected: "null or a non-empty string",
            }),
        },
    }
}

fn bool_or_false(body: &Value, name: &'static str) -> Result<bool, JsonWriteBindingError> {
    match body.get(name) {
        None | Some(Value::Null) => Ok(false),
        Some(value) => value.as_bool().ok_or(JsonWriteBindingError::InvalidField {
            name,
            expected: "a boolean",
        }),
    }
}

fn string_array_or_empty(
    body: &Value,
    name: &'static str,
) -> Result<Vec<String>, JsonWriteBindingError> {
    let Some(value) = body.get(name) else {
        return Ok(Vec::new());
    };
    let Value::Array(items) = value else {
        return Err(JsonWriteBindingError::InvalidField {
            name,
            expected: "an array of non-empty strings",
        });
    };
    items
        .iter()
        .map(|item| match item.as_str().map(str::trim) {
            Some(value) if !value.is_empty() => Ok(value.to_string()),
            _ => Err(JsonWriteBindingError::InvalidField {
                name,
                expected: "an array of non-empty strings",
            }),
        })
        .collect()
}

fn path_capture(
    request: &HttpRequest,
    name: &'static str,
) -> Result<String, JsonWriteBindingError> {
    request
        .path_captures
        .get(name)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .ok_or(JsonWriteBindingError::MissingPathCapture { name })
}

fn reject_path_body_drift(
    body: &Value,
    field: &'static str,
    path_value: &str,
) -> Result<(), JsonWriteBindingError> {
    if let Some(body_value) = optional_string(body, field)?
        && body_value != path_value
    {
        return Err(JsonWriteBindingError::PathBodyDrift {
            field,
            path_value: path_value.to_string(),
            body_value,
        });
    }
    Ok(())
}

fn required_header(
    request: &HttpRequest,
    names: &[&'static str],
) -> Result<String, JsonWriteBindingError> {
    for name in names {
        if let Some(value) = header_value(request, name).map(str::trim)
            && !value.is_empty()
        {
            return Ok(value.to_string());
        }
    }
    Err(JsonWriteBindingError::MissingHeader { name: names[0] })
}

fn header_value<'a>(request: &'a HttpRequest, name: &str) -> Option<&'a str> {
    request.headers.get(name).map(String::as_str).or_else(|| {
        request
            .headers
            .iter()
            .find_map(|(key, value)| key.eq_ignore_ascii_case(name).then_some(value.as_str()))
    })
}

fn scope_header(request: &HttpRequest) -> Result<String, JsonWriteBindingError> {
    required_header(
        request,
        &[
            "x-oya-scope-ref",
            "x-oya-tenant-scope-ref",
            "x-oya-tenant-id",
        ],
    )
}

fn principal_header(request: &HttpRequest) -> Result<String, JsonWriteBindingError> {
    required_header(request, &["x-oya-principal-ref"])
}

fn idempotency_header(request: &HttpRequest) -> Result<String, JsonWriteBindingError> {
    required_header(
        request,
        &[
            "idempotency-key",
            "x-idempotency-key",
            "x-oya-idempotency-key",
        ],
    )
}

fn policy_header(request: &HttpRequest) -> Result<String, JsonWriteBindingError> {
    required_header(request, &["x-oya-policy-decision-ref"])
}

fn request_id_header(request: &HttpRequest) -> Result<String, JsonWriteBindingError> {
    required_header(request, &["x-request-id", "x-oya-request-id"])
}

fn messenger_context(
    request: &HttpRequest,
) -> Result<messenger_rest::MessengerRestContext, JsonWriteBindingError> {
    Ok(messenger_rest::MessengerRestContext {
        scope_org_id: scope_header(request)?,
        context_kind: messenger_context_kind(&required_header(request, &["x-oya-context-kind"])?)?,
        principal_ref: principal_header(request)?,
        idempotency_key: idempotency_header(request)?,
        policy_decision_ref: policy_header(request)?,
        request_id: request_id_header(request)?,
    })
}

fn messenger_context_kind(
    value: &str,
) -> Result<messenger_rest::RestContextKind, JsonWriteBindingError> {
    match normalized_token(value).as_str() {
        "personal" => Ok(messenger_rest::RestContextKind::Personal),
        "professional" | "work" => Ok(messenger_rest::RestContextKind::Professional),
        _ => Err(JsonWriteBindingError::InvalidField {
            name: "x-oya-context-kind",
            expected: "one of personal, professional, work",
        }),
    }
}

fn mail_context(
    request: &HttpRequest,
) -> Result<mail_rest::MailRestContext, JsonWriteBindingError> {
    Ok(mail_rest::MailRestContext {
        tenant_id: scope_header(request)?,
        context_kind: mail_context_kind(&required_header(request, &["x-oya-context-kind"])?)?,
        principal_ref: principal_header(request)?,
        idempotency_key: idempotency_header(request)?,
        policy_decision_ref: policy_header(request)?,
        request_id: request_id_header(request)?,
    })
}

fn mail_context_kind(value: &str) -> Result<mail_rest::MailRestContextKind, JsonWriteBindingError> {
    match normalized_token(value).as_str() {
        "personal" => Ok(mail_rest::MailRestContextKind::Personal),
        "professional" | "work" => Ok(mail_rest::MailRestContextKind::Professional),
        _ => Err(JsonWriteBindingError::InvalidField {
            name: "x-oya-context-kind",
            expected: "one of personal, professional, work",
        }),
    }
}

fn social_context(
    request: &HttpRequest,
) -> Result<social_rest::SocialRestContext, JsonWriteBindingError> {
    Ok(social_rest::SocialRestContext {
        scope_org_id: scope_header(request)?,
        context_kind: social_context_kind(&required_header(request, &["x-oya-context-kind"])?)?,
        principal_ref: principal_header(request)?,
        idempotency_key: idempotency_header(request)?,
        policy_decision_ref: policy_header(request)?,
        request_id: request_id_header(request)?,
    })
}

fn social_context_kind(
    value: &str,
) -> Result<social_rest::SocialRestContextKind, JsonWriteBindingError> {
    match normalized_token(value).as_str() {
        "personal" => Ok(social_rest::SocialRestContextKind::Personal),
        "professional" | "work" => Ok(social_rest::SocialRestContextKind::Professional),
        _ => Err(JsonWriteBindingError::InvalidField {
            name: "x-oya-context-kind",
            expected: "one of personal, professional, work",
        }),
    }
}

fn community_context(
    request: &HttpRequest,
) -> Result<community_rest::CommunityRestContext, JsonWriteBindingError> {
    Ok(community_rest::CommunityRestContext {
        tenant_scope_ref: scope_header(request)?,
        principal_ref: principal_header(request)?,
        idempotency_key: idempotency_header(request)?,
        policy_decision_ref: policy_header(request)?,
        request_id: request_id_header(request)?,
    })
}

fn community_tenant_context(
    request: &HttpRequest,
) -> Result<TenantSqlContext, JsonWriteBindingError> {
    tenant_sql_context(request)
}

fn tenant_sql_context(request: &HttpRequest) -> Result<TenantSqlContext, JsonWriteBindingError> {
    TenantSqlContext::new(
        scope_header(request)?,
        required_header(request, &["x-oya-home-cell"])?,
        required_header(request, &["x-oya-shard-key"])?,
        required_header(request, &["x-oya-jurisdiction-code"])?,
    )
    .map_err(|error| JsonWriteBindingError::SqlPlan {
        reason: format!("{error:?}"),
    })
}

fn messenger_envelope(value: &Value) -> Result<MessengerApiEnvelope, JsonWriteBindingError> {
    let kind = required_string(value, "kind")?;
    match normalized_token(&kind).as_str() {
        "personal_e2e" => Ok(MessengerApiEnvelope::PersonalE2e {
            envelope_ref: required_string(value, "envelope_ref")?,
        }),
        "tenant_dek" => Ok(MessengerApiEnvelope::TenantDek {
            dek_ref: required_string(value, "dek_ref")?,
            four_eyes: bool_or_false(value, "four_eyes")?,
        }),
        "cross_org" => Ok(MessengerApiEnvelope::CrossOrg {
            local_dek_ref: required_string(value, "local_dek_ref")?,
            partner_scope_ref: required_string(value, "partner_scope_ref")?,
            partner_dek_ref: required_string(value, "partner_dek_ref")?,
            partner_ediscovery_allowed: bool_or_false(value, "partner_ediscovery_allowed")?,
        }),
        _ => Err(JsonWriteBindingError::InvalidField {
            name: "envelope.kind",
            expected: "one of personal_e2e, tenant_dek, cross_org",
        }),
    }
}

fn mail_envelope(value: &Value) -> Result<MailApiEnvelope, JsonWriteBindingError> {
    let kind = required_string(value, "kind")?;
    match normalized_token(&kind).as_str() {
        "personal_client_only" => Ok(MailApiEnvelope::PersonalClientOnly {
            envelope_ref: required_string(value, "envelope_ref")?,
        }),
        "tenant_dek" => Ok(MailApiEnvelope::TenantDek {
            dek_ref: required_string(value, "dek_ref")?,
        }),
        "imported" => Ok(MailApiEnvelope::Imported {
            source_hash: required_string(value, "source_hash")?,
            evidence_ref: required_string(value, "evidence_ref")?,
        }),
        _ => Err(JsonWriteBindingError::InvalidField {
            name: "envelope.kind",
            expected: "one of personal_client_only, tenant_dek, imported",
        }),
    }
}

fn social_artifact_kind(value: &str) -> Result<SocialApiArtifactKind, JsonWriteBindingError> {
    match normalized_token(value).as_str() {
        "feed_post" => Ok(SocialApiArtifactKind::FeedPost),
        "story" => Ok(SocialApiArtifactKind::Story),
        "collaborative_post" => Ok(SocialApiArtifactKind::CollaborativePost),
        _ => Err(JsonWriteBindingError::InvalidField {
            name: "kind",
            expected: "one of feed_post, story, collaborative_post",
        }),
    }
}

fn community_mode(value: &str) -> Result<CommunityApiMode, JsonWriteBindingError> {
    match normalized_token(value).as_str() {
        "reddit" => Ok(CommunityApiMode::Reddit),
        "teamblind" => Ok(CommunityApiMode::Teamblind),
        "handshake" => Ok(CommunityApiMode::Handshake),
        "knowledge_base" => Ok(CommunityApiMode::KnowledgeBase),
        _ => Err(JsonWriteBindingError::InvalidField {
            name: "mode",
            expected: "one of reddit, teamblind, handshake, knowledge_base",
        }),
    }
}

fn vote_direction(value: &str) -> Result<VoteDirection, JsonWriteBindingError> {
    match normalized_token(value).as_str() {
        "up" | "upvote" => Ok(VoteDirection::Up),
        "down" | "downvote" => Ok(VoteDirection::Down),
        "clear" | "none" => Ok(VoteDirection::Clear),
        _ => Err(JsonWriteBindingError::InvalidField {
            name: "direction",
            expected: "one of up, down, clear",
        }),
    }
}

fn moderation_verb(value: &str) -> Result<ModerationVerb, JsonWriteBindingError> {
    match normalized_token(value).as_str() {
        "allow" | "resolve_flag" | "unhide" | "unlock" => Ok(ModerationVerb::Allow),
        "hide" | "lock" | "quarantine" => Ok(ModerationVerb::Hide),
        "remove" | "delete" => Ok(ModerationVerb::Remove),
        _ => Err(JsonWriteBindingError::InvalidField {
            name: "verb",
            expected: "one of allow, hide, remove, resolve_flag, unhide, unlock, lock, quarantine, delete",
        }),
    }
}

fn dmarc_action_name(value: DmarcApiAction) -> &'static str {
    match value {
        DmarcApiAction::Accept => "accept",
        DmarcApiAction::Quarantine => "quarantine",
        DmarcApiAction::Reject => "reject",
    }
}

fn normalized_token(value: &str) -> String {
    value.trim().replace('-', "_").to_ascii_lowercase()
}

fn json_response(status_code: u16, body: String) -> HttpResponse {
    HttpResponse::new(status_code)
        .with_header("content-type", "application/json; charset=utf-8")
        .with_body(body.into_bytes())
}

fn json_string_escape(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0c}' => escaped.push_str("\\f"),
            '\u{00}'..='\u{1f}' => {
                write!(&mut escaped, "\\u{:04x}", ch as u32)
                    .expect("writing to an in-memory string cannot fail");
            }
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn json_string_array(values: &[String]) -> String {
    let rendered = values
        .iter()
        .map(|value| format!("\"{}\"", json_string_escape(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{rendered}]")
}

fn backbone_is_probe_path(path: &str) -> bool {
    path == "/health" || path == "/ready"
}

fn backbone_extract_bearer(request: &HttpRequest) -> Option<&str> {
    header_value(request, "authorization")
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
}

fn backbone_authz_request(
    microservice: Option<BackboneRestMicroservice>,
    bearer: &str,
    request: &HttpRequest,
) -> BackboneAuthzRequest {
    let matched_template = request
        .matched_template
        .clone()
        .unwrap_or_else(|| request.path.clone());
    let method = request.method.name().to_string();
    let service_slug = microservice
        .map(BackboneRestMicroservice::slug)
        .unwrap_or("backbone");
    let action = format!("backbone.{service_slug}.{}", method.to_ascii_lowercase());

    BackboneAuthzRequest {
        bearer: bearer.to_string(),
        microservice,
        method: method.clone(),
        path: request.path.clone(),
        matched_template: matched_template.clone(),
        action,
        resource: format!("{method} {matched_template}"),
        tenant_ref: header_value(request, "x-oya-scope-ref").map(str::to_string),
        context_kind: header_value(request, "x-oya-context-kind").map(str::to_string),
        caller_principal_ref: header_value(request, "x-oya-principal-ref").map(str::to_string),
        caller_policy_decision_ref: header_value(request, "x-oya-policy-decision-ref")
            .map(str::to_string),
        path_captures: request.path_captures.clone(),
    }
}

fn backbone_install_verified_authz_headers(
    request: &mut HttpRequest,
    principal: &BackboneVerifiedPrincipal,
) {
    remove_header_case_insensitive(&mut request.headers, "x-oya-principal-ref");
    remove_header_case_insensitive(&mut request.headers, "x-oya-policy-decision-ref");
    request.headers.insert(
        "x-oya-principal-ref".to_string(),
        principal.principal_ref.clone(),
    );
    request.headers.insert(
        "x-oya-policy-decision-ref".to_string(),
        principal.policy_decision_ref.clone(),
    );
}

fn remove_header_case_insensitive(headers: &mut BTreeMap<String, String>, name: &str) {
    let keys = headers
        .keys()
        .filter(|key| key.eq_ignore_ascii_case(name))
        .cloned()
        .collect::<Vec<_>>();
    for key in keys {
        headers.remove(&key);
    }
}

fn backbone_authn_response(status: u16, code: &str, reason: &str) -> HttpResponse {
    let body = format!(
        "{{\"status\":{},\"error_code\":\"{}\",\"reason\":\"{}\"}}",
        status,
        json_string_escape(code),
        json_string_escape(reason),
    );
    json_response(status, body)
}

/// Bearer-presence guard for backbone write-route handlers.
///
/// Named `verify_principal` so the authz-coverage engine recognises it as a guard
/// (verify_principal in authz_guard_idents in authz-coverage-policy.json).
///
/// The `BackboneAuthzMiddleware` performs the full PDP decision; this guard defends the
/// handler even when the chain is `empty_middleware_chain()` (test dispatch / direct callers).
/// authn-before-body-parse: invoked before `dispatch_runtime_route` reads the request body.
fn verify_principal(request: &HttpRequest) -> Result<(), HttpResponse> {
    if backbone_extract_bearer(request).is_none() {
        return Err(backbone_authn_response(
            401,
            "missing_bearer",
            "Authorization: Bearer header is required",
        ));
    }
    Ok(())
}

fn route_registration_error(
    route: BackboneRestRuntimeRoute,
    error: RouterError,
) -> BackboneRestRuntimeError {
    BackboneRestRuntimeError::RouteRegistration {
        microservice: route.microservice,
        method: route.method,
        path: route.path,
        reason: format!("{error:?}"),
    }
}

fn classify_messenger_route(route: &messenger_rest::OpenApiRoute) -> RuntimeRouteHandlerKind {
    if route.method == messenger_rest::PROBE_METHOD
        && (route.path == messenger_rest::HEALTH_ROUTE || route.path == messenger_rest::READY_ROUTE)
    {
        RuntimeRouteHandlerKind::Probe
    } else if route.handler_status == messenger_rest::RouteHandlerStatus::ContractOnly {
        RuntimeRouteHandlerKind::ContractOnly
    } else {
        RuntimeRouteHandlerKind::JsonWrite
    }
}

fn classify_mail_route(route: &mail_rest::OpenApiRoute) -> RuntimeRouteHandlerKind {
    if route.method == mail_rest::PROBE_METHOD
        && (route.path == mail_rest::HEALTH_ROUTE || route.path == mail_rest::READY_ROUTE)
    {
        RuntimeRouteHandlerKind::Probe
    } else if route.handler_status == mail_rest::RouteHandlerStatus::ContractOnly {
        RuntimeRouteHandlerKind::ContractOnly
    } else {
        RuntimeRouteHandlerKind::JsonWrite
    }
}

fn classify_social_route(route: &social_rest::OpenApiRoute) -> RuntimeRouteHandlerKind {
    if route.method == social_rest::PROBE_METHOD
        && (route.path == social_rest::HEALTH_ROUTE || route.path == social_rest::READY_ROUTE)
    {
        RuntimeRouteHandlerKind::Probe
    } else if route.handler_status == social_rest::RouteHandlerStatus::ContractOnly {
        RuntimeRouteHandlerKind::ContractOnly
    } else {
        RuntimeRouteHandlerKind::JsonWrite
    }
}

fn classify_community_route(route: &community_rest::OpenApiRoute) -> RuntimeRouteHandlerKind {
    if route.method == community_rest::PROBE_METHOD
        && (route.path == community_rest::HEALTH_ROUTE || route.path == community_rest::READY_ROUTE)
    {
        RuntimeRouteHandlerKind::Probe
    } else if route.handler_status == community_rest::RouteHandlerStatus::ContractOnly {
        RuntimeRouteHandlerKind::ContractOnly
    } else if route.method == community_rest::CREATE_POST_METHOD
        && route.path == community_rest::CREATE_POST_ROUTE
    {
        RuntimeRouteHandlerKind::JsonWrite
    } else {
        RuntimeRouteHandlerKind::TypedWritePlan
    }
}

fn messenger_dependencies(
    dependencies: &[BackboneRestReadinessDependency],
) -> Vec<messenger_rest::ReadinessDependency> {
    dependencies
        .iter()
        .map(|dependency| messenger_rest::ReadinessDependency {
            name: dependency.name,
            ready: dependency.ready,
        })
        .collect()
}

fn mail_dependencies(
    dependencies: &[BackboneRestReadinessDependency],
) -> Vec<mail_rest::ReadinessDependency> {
    dependencies
        .iter()
        .map(|dependency| mail_rest::ReadinessDependency {
            name: dependency.name,
            ready: dependency.ready,
        })
        .collect()
}

fn social_dependencies(
    dependencies: &[BackboneRestReadinessDependency],
) -> Vec<social_rest::ReadinessDependency> {
    dependencies
        .iter()
        .map(|dependency| social_rest::ReadinessDependency {
            name: dependency.name,
            ready: dependency.ready,
        })
        .collect()
}

fn community_dependencies(
    dependencies: &[BackboneRestReadinessDependency],
) -> Vec<community_rest::ReadinessDependency> {
    dependencies
        .iter()
        .map(|dependency| community_rest::ReadinessDependency {
            name: dependency.name,
            ready: dependency.ready,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use community_post_store_domain::{CommunityAuthor, CommunityMode, CommunityPost};
    use std::collections::BTreeMap;
    use std::io::{Read, Write};
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;

    // ---------------------------------------------------------------------------
    // Test authz providers
    // ---------------------------------------------------------------------------

    /// Permits any request with any bearer token. Use for dispatch tests that are
    /// not testing authz behaviour — they just need the bearer-presence check to pass.
    struct AlwaysPermitAuthzProvider;
    impl BackboneAuthzProvider for AlwaysPermitAuthzProvider {
        fn verify_and_authorize(
            &self,
            _request: &BackboneAuthzRequest,
        ) -> Result<BackboneVerifiedPrincipal, u16> {
            Ok(BackboneVerifiedPrincipal::new("user:u", "cedar:test-allow"))
        }
    }

    /// Denies every request at the PDP level (403). Use to test authz denial paths.
    struct DenyAllAuthzProvider;
    impl BackboneAuthzProvider for DenyAllAuthzProvider {
        fn verify_and_authorize(
            &self,
            _request: &BackboneAuthzRequest,
        ) -> Result<BackboneVerifiedPrincipal, u16> {
            Err(403)
        }
    }

    /// Accepts exactly one bearer token (exact-match; not constant-time — test only).
    struct FixedTokenAuthzProvider {
        token: &'static str,
    }
    impl BackboneAuthzProvider for FixedTokenAuthzProvider {
        fn verify_and_authorize(
            &self,
            request: &BackboneAuthzRequest,
        ) -> Result<BackboneVerifiedPrincipal, u16> {
            if request.bearer == self.token {
                Ok(BackboneVerifiedPrincipal::new("user:u", "cedar:test-allow"))
            } else {
                Err(401)
            }
        }
    }

    struct RecordingAuthzProvider {
        seen: Mutex<Option<BackboneAuthzRequest>>,
        principal: BackboneVerifiedPrincipal,
    }

    impl RecordingAuthzProvider {
        fn new(principal: BackboneVerifiedPrincipal) -> Self {
            Self {
                seen: Mutex::new(None),
                principal,
            }
        }

        fn seen(&self) -> BackboneAuthzRequest {
            self.seen
                .lock()
                .unwrap()
                .clone()
                .expect("authz provider should receive one request")
        }
    }

    impl BackboneAuthzProvider for RecordingAuthzProvider {
        fn verify_and_authorize(
            &self,
            request: &BackboneAuthzRequest,
        ) -> Result<BackboneVerifiedPrincipal, u16> {
            *self.seen.lock().unwrap() = Some(request.clone());
            Ok(self.principal.clone())
        }
    }

    fn permit_provider() -> Arc<dyn BackboneAuthzProvider> {
        Arc::new(AlwaysPermitAuthzProvider)
    }

    // ---------------------------------------------------------------------------
    // Request helpers
    // ---------------------------------------------------------------------------

    fn request(method: HttpMethod, path: &str) -> HttpRequest {
        HttpRequest {
            method,
            path: path.to_string(),
            headers: BTreeMap::new(),
            body: Vec::new(),
            path_captures: BTreeMap::new(),
            matched_template: None,
        }
    }

    fn json_request(method: HttpMethod, path: &str, body: &str) -> HttpRequest {
        let mut request = request(method, path);
        request.headers = common_headers("idem-json", "req-json");
        request.body = body.as_bytes().to_vec();
        request
    }

    fn common_headers(idempotency_key: &str, request_id: &str) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("x-oya-scope-ref".to_string(), "tenant:t".to_string()),
            ("x-oya-context-kind".to_string(), "professional".to_string()),
            ("x-oya-principal-ref".to_string(), "user:u".to_string()),
            ("idempotency-key".to_string(), idempotency_key.to_string()),
            (
                "x-oya-policy-decision-ref".to_string(),
                "cedar:allow".to_string(),
            ),
            ("x-request-id".to_string(), request_id.to_string()),
            // bearer required by verify_principal in all non-probe handlers
            (
                "authorization".to_string(),
                "Bearer test-bearer".to_string(),
            ),
        ])
    }

    fn seeded_community_post(post_id: &str) -> CommunityPost {
        CommunityPost::new(
            post_id.to_string(),
            "thread:seed".to_string(),
            "tenant:t".to_string(),
            CommunityMode::Reddit,
            CommunityAuthor::new(
                "display:author".to_string(),
                "user:author".to_string(),
                None,
            )
            .unwrap(),
            "body:seed".to_string(),
            "retention:community".to_string(),
        )
        .unwrap()
    }

    fn community_router_with_seeded_post() -> (BackboneRestRouter, BackboneCommunityJsonState) {
        let state = BackboneCommunityJsonState::new();
        state.seed_post(seeded_community_post("post-1")).unwrap();
        let built = build_backbone_rest_router_with_state(
            BackboneRestMicroservice::Community,
            Vec::new(),
            BackboneRestRuntimeState::with_community_state(state.clone()),
        )
        .unwrap();
        (built, state)
    }

    fn community_sql_router() -> (BackboneRestRouter, BackboneCommunitySqlWriteState) {
        let state = BackboneCommunitySqlWriteState::new();
        let built = build_backbone_rest_router_with_state(
            BackboneRestMicroservice::Community,
            Vec::new(),
            BackboneRestRuntimeState::with_community_sql_write_state(state.clone()),
        )
        .unwrap();
        (built, state)
    }

    fn stateless_sql_router(
        microservice: BackboneRestMicroservice,
    ) -> (BackboneRestRouter, BackboneSqlWriteState) {
        let state = BackboneSqlWriteState::new();
        let built = build_backbone_rest_router_with_state(
            microservice,
            Vec::new(),
            BackboneRestRuntimeState::with_stateless_sql_write_state(state.clone()),
        )
        .unwrap();
        (built, state)
    }

    fn community_sql_json_request(method: HttpMethod, path: &str, body: &str) -> HttpRequest {
        let mut request = json_request(method, path, body);
        request.headers.extend([
            ("x-oya-home-cell".to_string(), "cell-a".to_string()),
            ("x-oya-shard-key".to_string(), "tenant:t#cell-a".to_string()),
            ("x-oya-jurisdiction-code".to_string(), "US".to_string()),
        ]);
        request
    }

    fn stateless_sql_json_request(method: HttpMethod, path: &str, body: &str) -> HttpRequest {
        community_sql_json_request(method, path, body)
    }

    #[test]
    fn route_catalog_registers_every_openapi_route_with_honest_counts() {
        let expected = [
            // LIST_MESSAGES_ROUTE is Implemented → json_write=2, contract_only=22 for Messenger.
            (BackboneRestMicroservice::Messenger, 26, 2, 22, 2, 0, 0),
            (BackboneRestMicroservice::Mail, 17, 2, 14, 1, 0, 0),
            (BackboneRestMicroservice::Social, 27, 2, 24, 1, 0, 0),
            (BackboneRestMicroservice::Community, 24, 2, 19, 1, 0, 2),
        ];

        for (service, total, probes, contract_only, json_write, sql_write_plan, typed_write_plan) in
            expected
        {
            let built = build_backbone_rest_router(service, Vec::new()).unwrap();
            assert_eq!(built.microservice, service);
            assert_eq!(built.router.count(), total);
            assert_eq!(built.route_count, total);
            assert_eq!(built.probe_route_count, probes);
            assert_eq!(built.contract_only_route_count, contract_only);
            assert_eq!(built.json_write_route_count, json_write);
            assert_eq!(built.sql_write_plan_route_count, sql_write_plan);
            assert_eq!(built.typed_write_plan_route_count, typed_write_plan);
            assert!(built.non_claim.contains("no production gateway"));
        }
    }

    #[test]
    fn contract_only_route_returns_honest_501() {
        let built =
            build_backbone_rest_router(BackboneRestMicroservice::Messenger, Vec::new()).unwrap();
        let chain = empty_middleware_chain();
        // GET /channels is ContractOnly (not Probe), so verify_principal requires a bearer.
        let mut req = request(HttpMethod::Get, "/channels");
        req.headers.insert(
            "authorization".to_string(),
            "Bearer test-bearer".to_string(),
        );
        let response = dispatch_backbone_rest_request(req, &built.router, &chain);
        let body = String::from_utf8(response.body).unwrap();

        assert_eq!(response.status, 501);
        assert!(body.contains("\"runtime_handler\":\"contract_only\""));
        assert!(body.contains("no runtime handler claim"));
    }

    #[test]
    fn readiness_failure_maps_to_503_through_service_probe_handler() {
        let built = build_backbone_rest_router(
            BackboneRestMicroservice::Mail,
            vec![BackboneRestReadinessDependency {
                name: "postgres",
                ready: false,
            }],
        )
        .unwrap();
        let chain = empty_middleware_chain();
        let response = dispatch_backbone_rest_request(
            request(HttpMethod::Get, "/ready"),
            &built.router,
            &chain,
        );
        let body = String::from_utf8(response.body).unwrap();

        assert_eq!(response.status, 503);
        assert!(body.contains("\"runtime_handler\":\"probe\""));
        assert!(body.contains("\"dependency_count\":1"));
    }

    #[test]
    fn json_write_route_calls_service_owned_typed_handler() {
        let built =
            build_backbone_rest_router(BackboneRestMicroservice::Messenger, Vec::new()).unwrap();
        let chain = empty_middleware_chain();
        let response = dispatch_backbone_rest_request(
            json_request(
                HttpMethod::Post,
                "/channels/channel-a/messages",
                r#"{"message_id":"msg-1","author_ref":"user:u","envelope":{"kind":"tenant_dek","dek_ref":"dek:1","four_eyes":true},"retention_policy_id":"retention:default","legal_hold_ids":["hold:1"]}"#,
            ),
            &built.router,
            &chain,
        );
        let body = String::from_utf8(response.body).unwrap();

        assert_eq!(response.status, 201);
        assert!(body.contains("\"runtime_handler\":\"json_write\""));
        assert!(body.contains("\"event_type\":\"messenger.message.sent\""));
        assert!(body.contains("\"channel_id\":\"channel-a\""));
        assert!(body.contains("no database, broker, or live deployment claim"));
    }

    #[test]
    fn json_write_rejects_path_body_identifier_drift() {
        let built =
            build_backbone_rest_router(BackboneRestMicroservice::Messenger, Vec::new()).unwrap();
        let chain = empty_middleware_chain();
        let response = dispatch_backbone_rest_request(
            json_request(
                HttpMethod::Post,
                "/channels/channel-a/messages",
                r#"{"channel_id":"channel-b","message_id":"msg-1","author_ref":"user:u","envelope":{"kind":"tenant_dek","dek_ref":"dek:1","four_eyes":true},"retention_policy_id":"retention:default"}"#,
            ),
            &built.router,
            &chain,
        );
        let body = String::from_utf8(response.body).unwrap();

        assert_eq!(response.status, 400);
        assert!(body.contains("\"error_code\":\"path_body_drift\""));
    }

    #[test]
    fn json_write_accepts_case_insensitive_http_headers() {
        let built =
            build_backbone_rest_router(BackboneRestMicroservice::Messenger, Vec::new()).unwrap();
        let chain = empty_middleware_chain();
        let mut request = json_request(
            HttpMethod::Post,
            "/channels/channel-a/messages",
            r#"{"message_id":"msg-1","author_ref":"user:u","envelope":{"kind":"tenant_dek","dek_ref":"dek:1","four_eyes":true},"retention_policy_id":"retention:default"}"#,
        );
        request.headers = BTreeMap::from([
            ("X-Oya-Scope-Ref".to_string(), "tenant:t".to_string()),
            ("X-Oya-Context-Kind".to_string(), "professional".to_string()),
            ("X-Oya-Principal-Ref".to_string(), "user:u".to_string()),
            ("Idempotency-Key".to_string(), "idem-case".to_string()),
            (
                "X-Oya-Policy-Decision-Ref".to_string(),
                "cedar:allow".to_string(),
            ),
            ("X-Request-Id".to_string(), "req-case".to_string()),
            // verify_principal requires a bearer even when headers are mixed-case
            (
                "Authorization".to_string(),
                "Bearer test-bearer".to_string(),
            ),
        ]);
        let response = dispatch_backbone_rest_request(request, &built.router, &chain);
        let body = String::from_utf8(response.body).unwrap();

        assert_eq!(response.status, 201);
        assert!(body.contains("\"idempotency_key\":\"idem-case\""));
        assert!(body.contains("\"audit_correlation_id\":\"req-case\""));
    }

    #[test]
    fn json_write_error_response_escapes_control_characters() {
        let built =
            build_backbone_rest_router(BackboneRestMicroservice::Messenger, Vec::new()).unwrap();
        let chain = empty_middleware_chain();
        let response = dispatch_backbone_rest_request(
            json_request(
                HttpMethod::Post,
                "/channels/channel-a/messages",
                r#"{"channel_id":"channel-\u0001","message_id":"msg-1","author_ref":"user:u","envelope":{"kind":"tenant_dek","dek_ref":"dek:1","four_eyes":true},"retention_policy_id":"retention:default"}"#,
            ),
            &built.router,
            &chain,
        );
        let body = String::from_utf8(response.body).unwrap();
        let parsed: Value = serde_json::from_str(&body).unwrap();

        assert_eq!(response.status, 400);
        assert_eq!(parsed["error_code"], "path_body_drift");
        assert!(body.contains("\\u0001"));
    }

    #[test]
    fn stateful_write_plan_route_remains_honest_501() {
        let built =
            build_backbone_rest_router(BackboneRestMicroservice::Community, Vec::new()).unwrap();
        let chain = empty_middleware_chain();
        let response = dispatch_backbone_rest_request(
            json_request(
                HttpMethod::Post,
                "/posts/post-1/vote",
                r#"{"post_id":"post-1","voter_ref":"user:u","direction":"up"}"#,
            ),
            &built.router,
            &chain,
        );
        let body = String::from_utf8(response.body).unwrap();

        assert_eq!(response.status, 501);
        assert!(body.contains("typed_write_plan_required"));
        assert!(body.contains("backing read/write state"));
    }

    #[test]
    fn community_stateful_json_write_routes_bind_when_in_memory_state_is_composed() {
        let (built, _) = community_router_with_seeded_post();

        assert_eq!(built.route_count, 24);
        assert_eq!(built.probe_route_count, 2);
        assert_eq!(built.contract_only_route_count, 19);
        assert_eq!(built.json_write_route_count, 3);
        assert_eq!(built.sql_write_plan_route_count, 0);
        assert_eq!(built.typed_write_plan_route_count, 0);
        assert!(built.non_claim.contains("local in-memory community state"));
    }

    #[test]
    fn community_vote_json_write_uses_seeded_state_and_persists_vote_ledger() {
        let (built, _) = community_router_with_seeded_post();
        let chain = empty_middleware_chain();
        let body = r#"{"post_id":"post-1","voter_ref":"user:u","direction":"up"}"#;

        let response = dispatch_backbone_rest_request(
            json_request(HttpMethod::Post, "/posts/post-1/vote", body),
            &built.router,
            &chain,
        );
        let parsed: Value = serde_json::from_slice(&response.body).unwrap();

        assert_eq!(response.status, 200);
        assert_eq!(parsed["runtime_handler"], "json_write");
        assert_eq!(parsed["event_type"], "community.vote.cast");
        assert_eq!(parsed["post_id"], "post-1");
        assert_eq!(parsed["vote_id"], "idem-json");
        assert!(
            parsed["non_claim"]
                .as_str()
                .unwrap()
                .contains("local in-memory state")
        );

        let duplicate = dispatch_backbone_rest_request(
            json_request(HttpMethod::Post, "/posts/post-1/vote", body),
            &built.router,
            &chain,
        );
        let duplicate_body = String::from_utf8(duplicate.body).unwrap();

        assert_eq!(duplicate.status, 422);
        assert!(duplicate_body.contains("handler_rejected"));
        assert!(duplicate_body.contains("DuplicateVote"));
    }

    #[test]
    fn community_moderation_json_write_uses_seeded_state() {
        let (built, _) = community_router_with_seeded_post();
        let chain = empty_middleware_chain();
        let response = dispatch_backbone_rest_request(
            json_request(
                HttpMethod::Post,
                "/moderation/actions",
                r#"{"target_id":"post-1","target_type":"post","verb":"hide","policy_ref":"policy:moderation","evidence_ref":"flag:1"}"#,
            ),
            &built.router,
            &chain,
        );
        let parsed: Value = serde_json::from_slice(&response.body).unwrap();

        assert_eq!(response.status, 201);
        assert_eq!(parsed["runtime_handler"], "json_write");
        assert_eq!(parsed["event_type"], "community.moderation.actioned");
        assert_eq!(parsed["post_id"], "post-1");
        assert_eq!(parsed["evidence_ref"], "flag:1");
        assert!(
            parsed["non_claim"]
                .as_str()
                .unwrap()
                .contains("local in-memory state")
        );
    }

    #[test]
    fn community_sql_write_plan_json_routes_execute_recorded_sql_batches() {
        let (built, state) = community_sql_router();
        let chain = empty_middleware_chain();

        assert_eq!(built.route_count, 24);
        assert_eq!(built.json_write_route_count, 0);
        assert_eq!(built.sql_write_plan_route_count, 3);
        assert_eq!(built.typed_write_plan_route_count, 0);
        assert!(
            built
                .non_claim
                .contains("SQL write-plan recording executor")
        );

        let create = dispatch_backbone_rest_request(
            community_sql_json_request(
                HttpMethod::Post,
                "/spaces/space-1/posts",
                r#"{"post_id":"post-sql-1","thread_id":"thread:sql","mode":"teamblind","routine_display_ref":"anon:sql","audit_author_ref":"user:u","disclosure_policy_ref":"policy:disclosure","body_ref":"body:sql","retention_policy_id":"retention:community"}"#,
            ),
            &built.router,
            &chain,
        );
        let create_body: Value = serde_json::from_slice(&create.body).unwrap();

        assert_eq!(
            create.status,
            201,
            "{}",
            String::from_utf8_lossy(&create.body)
        );
        assert_eq!(create_body["runtime_handler"], "sql_write_plan");
        assert_eq!(create_body["event_type"], "community.post.created");
        assert_eq!(create_body["post_id"], "post-sql-1");
        assert_eq!(create_body["sql_command_count"], 3);
        assert_eq!(
            create_body["executed_command_names"][1],
            "insert_community_post"
        );
        assert_eq!(
            create_body["outbox_statement"],
            "insert_transactional_outbox_event"
        );

        let mut vote_request = community_sql_json_request(
            HttpMethod::Post,
            "/posts/post-sql-1/vote",
            r#"{"post_id":"post-sql-1","voter_ref":"user:voter","direction":"up"}"#,
        );
        vote_request
            .headers
            .insert("x-oya-principal-ref".to_string(), "user:voter".to_string());
        let vote = dispatch_backbone_rest_request(vote_request, &built.router, &chain);
        let vote_body: Value = serde_json::from_slice(&vote.body).unwrap();

        assert_eq!(vote.status, 200, "{}", String::from_utf8_lossy(&vote.body));
        assert_eq!(vote_body["runtime_handler"], "sql_write_plan");
        assert_eq!(vote_body["event_type"], "community.vote.cast");
        assert_eq!(
            vote_body["executed_command_names"][1],
            "insert_community_vote"
        );

        let moderation = dispatch_backbone_rest_request(
            community_sql_json_request(
                HttpMethod::Post,
                "/moderation/actions",
                r#"{"target_id":"post-sql-1","target_type":"post","verb":"hide","policy_ref":"policy:moderation","evidence_ref":"flag:sql"}"#,
            ),
            &built.router,
            &chain,
        );
        let moderation_body: Value = serde_json::from_slice(&moderation.body).unwrap();

        assert_eq!(
            moderation.status,
            201,
            "{}",
            String::from_utf8_lossy(&moderation.body)
        );
        assert_eq!(moderation_body["runtime_handler"], "sql_write_plan");
        assert_eq!(
            moderation_body["event_type"],
            "community.moderation.actioned"
        );
        assert_eq!(
            moderation_body["executed_command_names"][1],
            "insert_community_moderation_action"
        );
        assert!(
            moderation_body["non_claim"]
                .as_str()
                .unwrap()
                .contains("no live database")
        );

        let reports = state.execution_reports().unwrap();
        assert_eq!(reports.len(), 3);
        assert!(
            reports.iter().all(|report| report.transaction_committed
                && report.application_name == "oyatie-community")
        );
        assert_eq!(state.post_count().unwrap(), 1);
    }

    #[test]
    fn stateless_sql_write_plan_json_routes_execute_recorded_sql_batches() {
        let chain = empty_middleware_chain();
        let cases = [
            (
                BackboneRestMicroservice::Messenger,
                HttpMethod::Post,
                "/channels/channel-sql/messages",
                r#"{"message_id":"msg-sql-1","author_ref":"user:u","envelope":{"kind":"tenant_dek","dek_ref":"dek:sql","four_eyes":true},"retention_policy_id":"retention:default","legal_hold_ids":["hold:sql"]}"#,
                "messenger.message.sent",
                "message_id",
                "msg-sql-1",
                "insert_messenger_message",
                "oyatie-messenger",
                201,
                // LIST_MESSAGES_ROUTE is Implemented but not a stateless-sql-plan route → json_write=1.
                1_usize,
            ),
            (
                BackboneRestMicroservice::Mail,
                HttpMethod::Post,
                "/messages",
                r#"{"message_id":"mail-sql-1","mailbox_id":"mailbox:a","subject_ref":"user:u","envelope":{"kind":"tenant_dek","dek_ref":"dek:mail"},"retention_policy_id":"retention:mail"}"#,
                "mail.message.submitted",
                "message_id",
                "mail-sql-1",
                "insert_mail_message",
                "oyatie-mail",
                202,
                0_usize,
            ),
            (
                BackboneRestMicroservice::Social,
                HttpMethod::Post,
                "/posts",
                r#"{"post_id":"social-sql-1","creator_ref":"user:u","kind":"feed_post","media_refs":["media:1"],"workflow_consent_ref":"consent:workflow"}"#,
                "social.post.created",
                "post_id",
                "social-sql-1",
                "insert_social_post",
                "oyatie-social",
                201,
                0_usize,
            ),
        ];

        for (
            service,
            method,
            path,
            body,
            event_type,
            resource_field,
            resource_id,
            business_command,
            application_name,
            expected_status,
            expected_json_write_count,
        ) in cases
        {
            let (built, state) = stateless_sql_router(service);

            assert_eq!(built.json_write_route_count, expected_json_write_count);
            assert_eq!(built.sql_write_plan_route_count, 1);
            assert_eq!(built.typed_write_plan_route_count, 0);
            assert!(
                built
                    .non_claim
                    .contains("stateless SQL write-plan recording executor")
            );

            let response = dispatch_backbone_rest_request(
                stateless_sql_json_request(method, path, body),
                &built.router,
                &chain,
            );
            let parsed: Value = serde_json::from_slice(&response.body).unwrap();

            assert_eq!(
                response.status,
                expected_status,
                "{}",
                String::from_utf8_lossy(&response.body)
            );
            assert_eq!(parsed["runtime_handler"], "sql_write_plan");
            assert_eq!(parsed["event_type"], event_type);
            assert_eq!(parsed[resource_field], resource_id);
            assert_eq!(parsed["application_name"], application_name);
            assert_eq!(parsed["executed_command_names"][1], business_command);
            assert_eq!(
                parsed["outbox_statement"],
                "insert_transactional_outbox_event"
            );
            assert!(
                parsed["non_claim"]
                    .as_str()
                    .unwrap()
                    .contains("no live database")
            );

            let reports = state.execution_reports().unwrap();
            assert_eq!(reports.len(), 1);
            assert!(reports[0].transaction_committed);
            assert_eq!(reports[0].application_name, application_name);
        }
    }

    #[test]
    fn stateless_sql_write_plan_requires_tenant_sql_headers() {
        let (built, state) = stateless_sql_router(BackboneRestMicroservice::Messenger);
        let chain = empty_middleware_chain();
        let response = dispatch_backbone_rest_request(
            json_request(
                HttpMethod::Post,
                "/channels/channel-sql/messages",
                r#"{"message_id":"msg-sql-1","author_ref":"user:u","envelope":{"kind":"tenant_dek","dek_ref":"dek:sql","four_eyes":true},"retention_policy_id":"retention:default"}"#,
            ),
            &built.router,
            &chain,
        );
        let parsed: Value = serde_json::from_slice(&response.body).unwrap();

        assert_eq!(response.status, 400);
        assert_eq!(parsed["runtime_handler"], "sql_write_plan");
        assert_eq!(parsed["error_code"], "missing_header");
        assert!(
            parsed["reason"]
                .as_str()
                .unwrap()
                .contains("x-oya-home-cell")
        );
        assert!(state.execution_reports().unwrap().is_empty());
    }

    #[tokio::test]
    async fn loopback_clients_reach_all_four_rest_catalogs_over_tcp() {
        let cases = [
            (BackboneRestMicroservice::Messenger, "/channels"),
            (BackboneRestMicroservice::Mail, "/mailboxes"),
            (BackboneRestMicroservice::Social, "/profiles/me"),
            (BackboneRestMicroservice::Community, "/spaces"),
        ];

        for (service, contract_path) in cases {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            let server = tokio::spawn(async move {
                serve_backbone_rest_microservice_listener(
                    listener,
                    service,
                    vec![BackboneRestReadinessDependency {
                        name: "postgres",
                        ready: true,
                    }],
                    Arc::new(AlwaysPermitAuthzProvider),
                    ServerConfig::default(),
                )
                .await
            });

            // Probe path: exempt from bearer check, no Authorization header needed.
            let health = raw_http_request(addr, "GET", "/health").await;
            assert!(
                health.starts_with("HTTP/1.1 200"),
                "{} /health response was {health:?}",
                service.slug()
            );
            assert!(health.contains("\"runtime_handler\":\"probe\""));

            // Contract-only path: requires bearer (verify_principal in handler); AlwaysPermit
            // accepts it via the middleware chain, so we reach the 501 contract-only response.
            let contract = raw_http_authed_request(addr, "GET", contract_path, "test-bearer").await;
            assert!(
                contract.starts_with("HTTP/1.1 501"),
                "{} {contract_path} response was {contract:?}",
                service.slug()
            );
            assert!(contract.contains("\"runtime_handler\":\"contract_only\""));

            server.abort();
        }
    }

    #[tokio::test]
    async fn loopback_clients_execute_stateless_json_write_routes_over_tcp() {
        let cases = [
            (
                BackboneRestMicroservice::Messenger,
                "POST",
                "/channels/channel-a/messages",
                r#"{"message_id":"msg-1","author_ref":"user:u","envelope":{"kind":"tenant_dek","dek_ref":"dek:1","four_eyes":true},"retention_policy_id":"retention:default"}"#,
                "messenger.message.sent",
            ),
            (
                BackboneRestMicroservice::Mail,
                "POST",
                "/messages",
                r#"{"message_id":"mail-1","mailbox_id":"mailbox:inbox","subject_ref":"user:u","envelope":{"kind":"tenant_dek","dek_ref":"dek:mail"},"retention_policy_id":"retention:mail"}"#,
                "mail.message.submitted",
            ),
            (
                BackboneRestMicroservice::Social,
                "POST",
                "/posts",
                r#"{"post_id":"post-1","creator_ref":"user:u","kind":"feed_post","media_refs":["media:1"],"workflow_consent_ref":"workflow:consent"}"#,
                "social.post.created",
            ),
            (
                BackboneRestMicroservice::Community,
                "POST",
                "/spaces/space-a/posts",
                r#"{"space_id":"space-a","post_id":"post-1","thread_id":"thread:1","mode":"reddit","routine_display_ref":"display:user","audit_author_ref":"user:u","body_ref":"body:1","retention_policy_id":"retention:community"}"#,
                "community.post.created",
            ),
        ];

        for (service, method, path, body, event_type) in cases {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            let server = tokio::spawn(async move {
                serve_backbone_rest_microservice_listener(
                    listener,
                    service,
                    vec![BackboneRestReadinessDependency {
                        name: "postgres",
                        ready: true,
                    }],
                    Arc::new(AlwaysPermitAuthzProvider),
                    ServerConfig::default(),
                )
                .await
            });

            let response =
                raw_http_json_request(addr, method, path, body, "idem-loop", "req-loop").await;
            assert!(
                response.starts_with("HTTP/1.1 20"),
                "{} {path} response was {response:?}",
                service.slug()
            );
            assert!(response.contains("\"runtime_handler\":\"json_write\""));
            assert!(response.contains(event_type));

            server.abort();
        }
    }

    /// Raw HTTP request with no body; includes `Authorization: Bearer test-bearer` so
    /// non-probe routes pass `verify_principal`. Probe paths ignore the bearer.
    async fn raw_http_request(addr: SocketAddr, method: &str, path: &str) -> String {
        raw_http_authed_request(addr, method, path, "test-bearer").await
    }

    async fn raw_http_authed_request(
        addr: SocketAddr,
        method: &str,
        path: &str,
        bearer: &str,
    ) -> String {
        let method = method.to_string();
        let path = path.to_string();
        let bearer = bearer.to_string();
        tokio::task::spawn_blocking(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let request = format!(
                "{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nAuthorization: Bearer {bearer}\r\n\r\n"
            );
            stream.write_all(request.as_bytes()).unwrap();
            let mut response = String::new();
            stream.read_to_string(&mut response).unwrap();
            response
        })
        .await
        .unwrap()
    }

    async fn raw_http_json_request(
        addr: SocketAddr,
        method: &str,
        path: &str,
        body: &str,
        idempotency_key: &str,
        request_id: &str,
    ) -> String {
        let method = method.to_string();
        let path = path.to_string();
        let body = body.to_string();
        let idempotency_key = idempotency_key.to_string();
        let request_id = request_id.to_string();
        tokio::task::spawn_blocking(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let request = format!(
                "{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAuthorization: Bearer test-bearer\r\nX-Oya-Scope-Ref: tenant:t\r\nX-Oya-Context-Kind: professional\r\nX-Oya-Principal-Ref: user:u\r\nIdempotency-Key: {idempotency_key}\r\nX-Oya-Policy-Decision-Ref: cedar:allow\r\nX-Request-Id: {request_id}\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(request.as_bytes()).unwrap();
            let mut response = String::new();
            stream.read_to_string(&mut response).unwrap();
            response
        })
        .await
        .unwrap()
    }

    // ---------------------------------------------------------------------------
    // TDD: authz middleware + fail-closed behaviour
    // ---------------------------------------------------------------------------

    #[test]
    fn empty_middleware_chain_has_zero_middlewares_test_only() {
        // Confirms empty_middleware_chain is a no-op pass-through with no authz layer.
        // ponytail: this test documents the intentional gap so reviewers don't mistake
        // "empty chain" for production-safe; backbone_authz_chain is the correct path.
        assert_eq!(empty_middleware_chain().count(), 0);
    }

    #[test]
    fn backbone_authz_chain_has_exactly_one_middleware() {
        let chain = backbone_authz_chain(permit_provider());
        assert_eq!(chain.count(), 1);
    }

    #[test]
    fn authz_middleware_exempts_probe_paths_no_bearer_required() {
        // /health and /ready must reach the probe handler without a bearer token.
        let built = build_backbone_rest_router(BackboneRestMicroservice::Mail, Vec::new()).unwrap();
        let chain = backbone_authz_chain(Arc::new(DenyAllAuthzProvider));

        for path in ["/health", "/ready"] {
            let response = dispatch_backbone_rest_request(
                request(HttpMethod::Get, path),
                &built.router,
                &chain,
            );
            assert_eq!(
                response.status, 200,
                "{path} probe should be 200 with DenyAllAuthzProvider, got {}",
                response.status
            );
            let body = String::from_utf8(response.body).unwrap();
            assert!(body.contains("\"runtime_handler\":\"probe\""));
        }
    }

    #[test]
    fn authz_middleware_returns_401_without_bearer_header() {
        // authn-before-body-parse: body is never read when the bearer is missing.
        let built =
            build_backbone_rest_router(BackboneRestMicroservice::Messenger, Vec::new()).unwrap();
        let chain = backbone_authz_chain(permit_provider());

        // Non-probe route with no Authorization header.
        let response = dispatch_backbone_rest_request(
            request(HttpMethod::Get, "/channels"),
            &built.router,
            &chain,
        );
        assert_eq!(response.status, 401);
        let body = String::from_utf8(response.body).unwrap();
        assert!(body.contains("\"error_code\":\"missing_bearer\""));
    }

    #[test]
    fn authz_middleware_returns_401_for_forged_bearer() {
        let built =
            build_backbone_rest_router(BackboneRestMicroservice::Messenger, Vec::new()).unwrap();
        let chain = backbone_authz_chain(Arc::new(FixedTokenAuthzProvider { token: "correct" }));

        let mut req = request(HttpMethod::Get, "/channels");
        req.headers.insert(
            "authorization".to_string(),
            "Bearer forged-token".to_string(),
        );
        let response = dispatch_backbone_rest_request(req, &built.router, &chain);

        assert_eq!(response.status, 401);
        let body = String::from_utf8(response.body).unwrap();
        assert!(body.contains("\"error_code\":\"authz_denied\""));
    }

    #[test]
    fn authz_middleware_returns_403_when_provider_denies() {
        let built =
            build_backbone_rest_router(BackboneRestMicroservice::Social, Vec::new()).unwrap();
        let chain = backbone_authz_chain(Arc::new(DenyAllAuthzProvider));

        let mut req = request(HttpMethod::Get, "/profiles/me");
        req.headers
            .insert("authorization".to_string(), "Bearer any-bearer".to_string());
        let response = dispatch_backbone_rest_request(req, &built.router, &chain);

        assert_eq!(response.status, 403);
        let body = String::from_utf8(response.body).unwrap();
        assert!(body.contains("\"error_code\":\"authz_denied\""));
    }

    #[test]
    fn authz_middleware_permits_valid_bearer_happy_path() {
        let built =
            build_backbone_rest_router(BackboneRestMicroservice::Messenger, Vec::new()).unwrap();
        let chain = backbone_authz_chain(Arc::new(FixedTokenAuthzProvider { token: "secret" }));

        let mut req = json_request(
            HttpMethod::Post,
            "/channels/channel-a/messages",
            r#"{"message_id":"msg-1","author_ref":"user:u","envelope":{"kind":"tenant_dek","dek_ref":"dek:1","four_eyes":true},"retention_policy_id":"retention:default"}"#,
        );
        // Override the authorization header with the exact token the provider accepts.
        req.headers
            .insert("authorization".to_string(), "Bearer secret".to_string());
        let response = dispatch_backbone_rest_request(req, &built.router, &chain);

        assert_eq!(response.status, 201);
        let body = String::from_utf8(response.body).unwrap();
        assert!(body.contains("\"runtime_handler\":\"json_write\""));
        assert!(body.contains("\"event_type\":\"messenger.message.sent\""));
    }

    #[test]
    fn authz_middleware_passes_pbac_context_and_replaces_caller_authz_headers() {
        let built =
            build_backbone_rest_router(BackboneRestMicroservice::Messenger, Vec::new()).unwrap();
        let provider = Arc::new(RecordingAuthzProvider::new(BackboneVerifiedPrincipal::new(
            "user:u",
            "cedar:decision:verified",
        )));
        let chain = backbone_authz_chain_for_microservice(
            BackboneRestMicroservice::Messenger,
            provider.clone(),
        );

        let mut req = json_request(
            HttpMethod::Post,
            "/channels/channel-a/messages",
            r#"{"message_id":"msg-1","author_ref":"user:u","envelope":{"kind":"tenant_dek","dek_ref":"dek:1","four_eyes":true},"retention_policy_id":"retention:default"}"#,
        );
        req.headers
            .insert("authorization".to_string(), "Bearer secret".to_string());
        req.headers
            .insert("x-oya-principal-ref".to_string(), "forged:user".to_string());
        req.headers.insert(
            "x-oya-policy-decision-ref".to_string(),
            "cedar:forged".to_string(),
        );

        let response = dispatch_backbone_rest_request(req, &built.router, &chain);

        assert_eq!(response.status, 201);
        let seen = provider.seen();
        assert_eq!(seen.bearer, "secret");
        assert_eq!(seen.microservice, Some(BackboneRestMicroservice::Messenger));
        assert_eq!(seen.method, "POST");
        assert_eq!(seen.path, "/channels/channel-a/messages");
        assert_eq!(seen.matched_template, messenger_rest::POST_MESSAGE_ROUTE);
        assert_eq!(seen.action, "backbone.messenger.post");
        assert_eq!(
            seen.resource,
            format!("POST {}", messenger_rest::POST_MESSAGE_ROUTE)
        );
        assert_eq!(seen.tenant_ref.as_deref(), Some("tenant:t"));
        assert_eq!(seen.context_kind.as_deref(), Some("professional"));
        assert_eq!(seen.caller_principal_ref.as_deref(), Some("forged:user"));
        assert_eq!(
            seen.caller_policy_decision_ref.as_deref(),
            Some("cedar:forged")
        );
        assert_eq!(
            seen.path_captures.get("channel_id").map(String::as_str),
            Some("channel-a")
        );

        let body = String::from_utf8(response.body).unwrap();
        assert!(body.contains("\"policy_decision_ref\":\"cedar:decision:verified\""));
        assert!(!body.contains("cedar:forged"));
    }

    #[test]
    fn verify_principal_rejects_request_without_bearer_even_with_empty_chain() {
        // Defence-in-depth: the handler-level guard fires even when the middleware chain is
        // empty_middleware_chain(), so bypassing the serve_* chain still cannot reach write routes.
        let built =
            build_backbone_rest_router(BackboneRestMicroservice::Messenger, Vec::new()).unwrap();
        let chain = empty_middleware_chain(); // no middleware at all

        // No Authorization header → verify_principal returns 401 before body parse.
        let response = dispatch_backbone_rest_request(
            request(HttpMethod::Post, "/channels/channel-a/messages"),
            &built.router,
            &chain,
        );
        assert_eq!(response.status, 401);
        let body = String::from_utf8(response.body).unwrap();
        assert!(body.contains("\"error_code\":\"missing_bearer\""));
    }

    #[test]
    fn literal_write_route_const_strings_appear_in_service_catalogs() {
        // Verifies that the const route strings in the REST crates match the route catalog.
        // This is the "literal-route resolution" test: the consts are the engine-resolvable
        // identifiers that would let the authz-coverage engine classify write routes.
        let write_consts: &[(&str, &str, BackboneRestMicroservice)] = &[
            (
                messenger_rest::POST_MESSAGE_METHOD,
                messenger_rest::POST_MESSAGE_ROUTE,
                BackboneRestMicroservice::Messenger,
            ),
            (
                mail_rest::SUBMIT_MESSAGE_METHOD,
                mail_rest::SUBMIT_MESSAGE_ROUTE,
                BackboneRestMicroservice::Mail,
            ),
            (
                social_rest::PUBLISH_POST_METHOD,
                social_rest::PUBLISH_POST_ROUTE,
                BackboneRestMicroservice::Social,
            ),
            (
                community_rest::CREATE_POST_METHOD,
                community_rest::CREATE_POST_ROUTE,
                BackboneRestMicroservice::Community,
            ),
            (
                community_rest::CAST_VOTE_METHOD,
                community_rest::CAST_VOTE_ROUTE,
                BackboneRestMicroservice::Community,
            ),
            (
                community_rest::APPLY_MODERATION_ACTION_METHOD,
                community_rest::APPLY_MODERATION_ACTION_ROUTE,
                BackboneRestMicroservice::Community,
            ),
        ];

        for (method, path, service) in write_consts {
            assert!(
                !path.is_empty(),
                "{service:?} write route path must not be empty"
            );
            assert!(
                !method.is_empty(),
                "{service:?} write route method must not be empty"
            );
            let catalog = route_runtime_catalog(*service);
            assert!(
                catalog
                    .iter()
                    .any(|r| r.path == *path && r.method == *method),
                "write const ({method} {path}) not found in {service:?} route catalog"
            );
        }
    }
}
