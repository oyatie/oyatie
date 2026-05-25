//! Shared REST runtime adapter for the four backbone microservice route catalogs.
//!
//! This crate binds the framework-free REST catalogs for messenger, mail,
//! social, and community into transport-neutral routers that the canonical
//! Hyper runtime adapter can serve without this crate importing Hyper directly.
//! It is intentionally honest about the current seam: probes and contract-only
//! OpenAPI routes are runtime-dispatchable; typed write-plan routes remain
//! implemented in their protocol-neutral REST crates but are not yet
//! JSON-body-bound in this generic runtime adapter.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::sync::Arc;

use oya_community_post_store_rest as community_rest;
use oya_http_middleware_kernel::{HttpRequest, HttpResponse, MiddlewareChain};
use oya_http_router_kernel::{HttpMethod, Router, RouterError};
use oya_http_runtime_hyper_adapter::{HyperRuntimeError, ServerConfig, serve_listener};
use oya_mail_mailbox_store_rest as mail_rest;
use oya_messenger_message_stream_rest as messenger_rest;
use oya_social_post_composition_rest as social_rest;
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
    TypedWritePlan,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackboneRestRuntimeRoute {
    pub microservice: BackboneRestMicroservice,
    pub method: &'static str,
    pub path: &'static str,
    pub handler_kind: RuntimeRouteHandlerKind,
}

/// A router plus the counts auditors need to understand what is really bound.
pub struct BackboneRestRouter {
    pub microservice: BackboneRestMicroservice,
    pub router: Router<BackboneRestSyncHandler>,
    pub route_count: usize,
    pub probe_route_count: usize,
    pub contract_only_route_count: usize,
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
/// 501, and typed write-plan routes return a typed-binding-required 501.
pub fn build_backbone_rest_router(
    microservice: BackboneRestMicroservice,
    dependencies: Vec<BackboneRestReadinessDependency>,
) -> Result<BackboneRestRouter, BackboneRestRuntimeError> {
    let routes = route_runtime_catalog(microservice);
    let mut router: Router<BackboneRestSyncHandler> = Router::new();
    let mut probe_route_count = 0;
    let mut contract_only_route_count = 0;
    let mut typed_write_plan_route_count = 0;

    for route in routes.iter().copied() {
        match route.handler_kind {
            RuntimeRouteHandlerKind::Probe => probe_route_count += 1,
            RuntimeRouteHandlerKind::ContractOnly => contract_only_route_count += 1,
            RuntimeRouteHandlerKind::TypedWritePlan => typed_write_plan_route_count += 1,
        }
        let method =
            HttpMethod::parse(route.method).ok_or(BackboneRestRuntimeError::UnsupportedMethod {
                microservice,
                method: route.method,
                path: route.path,
            })?;
        let route_dependencies = dependencies.clone();
        let handler: BackboneRestSyncHandler = Arc::new(move |request: HttpRequest| {
            dispatch_runtime_route(route, &route_dependencies, &request)
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
        typed_write_plan_route_count,
        non_claim: "local Hyper loopback/runtime binding only; no production gateway, TLS, or JSON write-body binding claim",
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
    request: &HttpRequest,
) -> HttpResponse {
    match route.handler_kind {
        RuntimeRouteHandlerKind::Probe => dispatch_probe_route(route, dependencies),
        RuntimeRouteHandlerKind::ContractOnly => contract_only_response(route, request),
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

fn typed_write_plan_response(
    route: BackboneRestRuntimeRoute,
    request: &HttpRequest,
) -> HttpResponse {
    let body = format!(
        "{{\"microservice\":\"{}\",\"method\":\"{}\",\"path\":\"{}\",\"matched_template\":\"{}\",\"runtime_handler\":\"typed_write_plan_required\",\"status_code\":501,\"reason\":\"typed protocol-neutral write dispatcher exists; generic Hyper JSON-body binding is not claimed by this adapter\"}}",
        route.microservice.slug(),
        route.method,
        route.path,
        json_string_escape(request.matched_template.as_deref().unwrap_or(route.path))
    );
    json_response(501, body)
}

fn json_response(status_code: u16, body: String) -> HttpResponse {
    HttpResponse::new(status_code)
        .with_header("content-type", "application/json; charset=utf-8")
        .with_body(body.into_bytes())
}

fn json_string_escape(input: &str) -> String {
    input
        .chars()
        .flat_map(|ch| match ch {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
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
        RuntimeRouteHandlerKind::TypedWritePlan
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
        RuntimeRouteHandlerKind::TypedWritePlan
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
        RuntimeRouteHandlerKind::TypedWritePlan
    }
}

fn classify_community_route(route: &community_rest::OpenApiRoute) -> RuntimeRouteHandlerKind {
    if route.method == community_rest::PROBE_METHOD
        && (route.path == community_rest::HEALTH_ROUTE || route.path == community_rest::READY_ROUTE)
    {
        RuntimeRouteHandlerKind::Probe
    } else if route.handler_status == community_rest::RouteHandlerStatus::ContractOnly {
        RuntimeRouteHandlerKind::ContractOnly
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

    #[test]
    fn route_catalog_registers_every_openapi_route_with_honest_counts() {
        let expected = [
            (BackboneRestMicroservice::Messenger, 26, 2, 23, 1),
            (BackboneRestMicroservice::Mail, 17, 2, 14, 1),
            (BackboneRestMicroservice::Social, 27, 2, 24, 1),
            (BackboneRestMicroservice::Community, 24, 2, 19, 3),
        ];

        for (service, total, probes, contract_only, typed_write_plan) in expected {
            let built = build_backbone_rest_router(service, Vec::new()).unwrap();
            assert_eq!(built.microservice, service);
            assert_eq!(built.router.count(), total);
            assert_eq!(built.route_count, total);
            assert_eq!(built.probe_route_count, probes);
            assert_eq!(built.contract_only_route_count, contract_only);
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
    fn typed_write_route_does_not_fake_generic_json_binding() {
        let built =
            build_backbone_rest_router(BackboneRestMicroservice::Social, Vec::new()).unwrap();
        let chain = empty_middleware_chain();
        let response = dispatch_backbone_rest_request(
            request(HttpMethod::Post, "/posts"),
            &built.router,
            &chain,
        );
        let body = String::from_utf8(response.body).unwrap();

        assert_eq!(response.status, 501);
        assert!(body.contains("typed_write_plan_required"));
        assert!(body.contains("generic Hyper JSON-body binding is not claimed"));
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
}
