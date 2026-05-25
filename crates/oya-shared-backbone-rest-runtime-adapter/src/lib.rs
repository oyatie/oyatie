//! Shared REST runtime adapter for the four backbone microservice route catalogs.
//!
//! This crate binds the framework-free REST catalogs for messenger, mail,
//! social, and community into transport-neutral routers that the canonical
//! Hyper runtime adapter can serve without this crate importing Hyper directly.
//! It is intentionally honest about the current seam: probes, contract-only
//! OpenAPI routes, and the stateless typed write routes are
//! runtime-dispatchable; stateful write-plan routes remain explicit
//! `501` seams until a backing read/write store is composed. The community
//! vote/moderation routes can be bound to an explicitly supplied local
//! in-memory state object for loopback tests; that path still makes no durable
//! database, broker, cluster, or production deployment claim.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::{
    collections::BTreeMap,
    fmt::Write as _,
    sync::{Arc, Mutex},
};

use oya_community_post_store_api::{
    CastVoteRequest, CommunityApiMode, CreatePostRequest, ModeratePostRequest, ModerationVerb,
    VoteDirection,
};
use oya_community_post_store_domain::{CommunityPost, VoteLedger};
use oya_community_post_store_rest as community_rest;
use oya_http_middleware_kernel::{HttpRequest, HttpResponse, MiddlewareChain};
use oya_http_router_kernel::{HttpMethod, Router, RouterError};
use oya_http_runtime_hyper_adapter::{HyperRuntimeError, ServerConfig, serve_listener};
use oya_mail_mailbox_store_api::{DmarcApiAction, MailApiEnvelope};
use oya_mail_mailbox_store_rest as mail_rest;
use oya_messenger_message_stream_api::MessengerApiEnvelope;
use oya_messenger_message_stream_rest as messenger_rest;
use oya_social_post_composition_api::SocialApiArtifactKind;
use oya_social_post_composition_rest as social_rest;
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
    community: Option<BackboneCommunityJsonState>,
}

impl BackboneRestRuntimeState {
    pub fn without_state() -> Self {
        Self::default()
    }

    pub fn with_community_state(community: BackboneCommunityJsonState) -> Self {
        Self {
            community: Some(community),
        }
    }

    fn community_state(&self) -> Option<&BackboneCommunityJsonState> {
        self.community.as_ref()
    }

    fn handler_kind_for(&self, route: BackboneRestRuntimeRoute) -> RuntimeRouteHandlerKind {
        if self.binds_stateful_community_route(route) {
            RuntimeRouteHandlerKind::JsonWrite
        } else {
            route.handler_kind
        }
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

/// A router plus the counts auditors need to understand what is really bound.
pub struct BackboneRestRouter {
    pub microservice: BackboneRestMicroservice,
    pub router: Router<BackboneRestSyncHandler>,
    pub route_count: usize,
    pub probe_route_count: usize,
    pub contract_only_route_count: usize,
    pub json_write_route_count: usize,
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
    let mut typed_write_plan_route_count = 0;

    for route in routes.iter().copied() {
        match route.handler_kind {
            RuntimeRouteHandlerKind::Probe => probe_route_count += 1,
            RuntimeRouteHandlerKind::ContractOnly => contract_only_route_count += 1,
            RuntimeRouteHandlerKind::JsonWrite => json_write_route_count += 1,
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
        typed_write_plan_route_count,
        non_claim: if microservice == BackboneRestMicroservice::Community
            && runtime_state.community_state().is_some()
        {
            "local Hyper loopback/runtime binding with local in-memory community state only; no production gateway, TLS, durable database, broker, OpenCost, ArgoCD sync, or live deployment claim"
        } else {
            "local Hyper loopback/runtime binding only; no production gateway, TLS, database, broker, or stateful write-route claim"
        },
    })
}

pub fn empty_middleware_chain() -> MiddlewareChain<HttpRequest, HttpResponse> {
    MiddlewareChain::new()
}

/// Serve one backbone REST microservice using an already-bound listener.
pub async fn serve_backbone_rest_microservice_listener(
    listener: TcpListener,
    microservice: BackboneRestMicroservice,
    dependencies: Vec<BackboneRestReadinessDependency>,
    config: ServerConfig,
) -> Result<(), BackboneRestRuntimeError> {
    let built = build_backbone_rest_router(microservice, dependencies)?;
    serve_listener(
        listener,
        Arc::new(built.router),
        Arc::new(empty_middleware_chain()),
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
}

impl JsonWriteBindingError {
    fn status_code(&self) -> u16 {
        match self {
            JsonWriteBindingError::Handler { .. } => 422,
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

fn dispatch_messenger_json_write(
    route: BackboneRestRuntimeRoute,
    request: &HttpRequest,
) -> Result<HttpResponse, JsonWriteBindingError> {
    let body = json_body(request)?;
    let channel_id = path_capture(request, "channel_id")?;
    reject_path_body_drift(&body, "channel_id", &channel_id)?;
    let context = messenger_context(request)?;
    let rest_request = messenger_rest::PostMessageRestRequest {
        channel_id,
        message_id: required_string(&body, "message_id")?,
        author_ref: required_string(&body, "author_ref")?,
        envelope: messenger_envelope(required_value(&body, "envelope")?)?,
        retention_policy_id: required_string(&body, "retention_policy_id")?,
        legal_hold_ids: string_array_or_empty(&body, "legal_hold_ids")?,
    };
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
    let rest_request = mail_rest::SubmitMessageRestRequest {
        message_id: required_string(&body, "message_id")?,
        mailbox_id: required_string(&body, "mailbox_id")?,
        subject_ref: required_string(&body, "subject_ref")?,
        envelope: mail_envelope(required_value(&body, "envelope")?)?,
        retention_policy_id: required_string(&body, "retention_policy_id")?,
    };
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
    let rest_request = social_rest::PublishPostRestRequest {
        post_id: required_string(&body, "post_id")?,
        creator_ref: required_string(&body, "creator_ref")?,
        media_refs: string_array_or_empty(&body, "media_refs")?,
        kind: social_artifact_kind(&required_string(&body, "kind")?)?,
        workflow_consent_ref: optional_string(&body, "workflow_consent_ref")?,
    };
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

const STATELESS_JSON_WRITE_NON_CLAIM: &str =
    "local stateless write handler only; no database, broker, or live deployment claim";
const STATEFUL_COMMUNITY_JSON_WRITE_NON_CLAIM: &str = "local in-memory state handler only; no durable database, broker, cluster, OpenCost, ArgoCD sync, or live deployment claim";

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
    use oya_community_post_store_domain::{CommunityAuthor, CommunityMode, CommunityPost};
    use std::collections::BTreeMap;
    use std::io::{Read, Write};
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;

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

    #[test]
    fn route_catalog_registers_every_openapi_route_with_honest_counts() {
        let expected = [
            (BackboneRestMicroservice::Messenger, 26, 2, 23, 1, 0),
            (BackboneRestMicroservice::Mail, 17, 2, 14, 1, 0),
            (BackboneRestMicroservice::Social, 27, 2, 24, 1, 0),
            (BackboneRestMicroservice::Community, 24, 2, 19, 1, 2),
        ];

        for (service, total, probes, contract_only, json_write, typed_write_plan) in expected {
            let built = build_backbone_rest_router(service, Vec::new()).unwrap();
            assert_eq!(built.microservice, service);
            assert_eq!(built.router.count(), total);
            assert_eq!(built.route_count, total);
            assert_eq!(built.probe_route_count, probes);
            assert_eq!(built.contract_only_route_count, contract_only);
            assert_eq!(built.json_write_route_count, json_write);
            assert_eq!(built.typed_write_plan_route_count, typed_write_plan);
            assert!(built.non_claim.contains("no production gateway"));
        }
    }

    #[test]
    fn contract_only_route_returns_honest_501() {
        let built =
            build_backbone_rest_router(BackboneRestMicroservice::Messenger, Vec::new()).unwrap();
        let chain = empty_middleware_chain();
        let response = dispatch_backbone_rest_request(
            request(HttpMethod::Get, "/channels"),
            &built.router,
            &chain,
        );
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
                    ServerConfig::default(),
                )
                .await
            });

            let health = raw_http_request(addr, "GET", "/health").await;
            assert!(
                health.starts_with("HTTP/1.1 200"),
                "{} /health response was {health:?}",
                service.slug()
            );
            assert!(health.contains("\"runtime_handler\":\"probe\""));

            let contract = raw_http_request(addr, "GET", contract_path).await;
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

    async fn raw_http_request(addr: SocketAddr, method: &str, path: &str) -> String {
        let method = method.to_string();
        let path = path.to_string();
        tokio::task::spawn_blocking(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let request =
                format!("{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
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
                "{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\nX-Oya-Scope-Ref: tenant:t\r\nX-Oya-Context-Kind: professional\r\nX-Oya-Principal-Ref: user:u\r\nIdempotency-Key: {idempotency_key}\r\nX-Oya-Policy-Decision-Ref: cedar:allow\r\nX-Request-Id: {request_id}\r\n\r\n{body}",
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
}
