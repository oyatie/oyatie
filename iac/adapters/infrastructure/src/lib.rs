//! In-process runtime composition for the Cloud IaC OpenTofu module registry.
//!
//! This crate composes the framework-free REST router boundary with the pure
//! API DTO boundary at the clean-architecture `infrastructure` role. It does not bind
//! sockets, start a Hyper server, read request bodies, call an auth service,
//! persist registry state, sign modules, invoke OpenTofu, call providers, or
//! provision cloud resources.

#![forbid(unsafe_code)]

use std::sync::Arc;

use iac_api::{
    CLOUD_IAC_MODULE_REGISTRY_DISCOVERY_SURFACE as API_DISCOVERY_SURFACE,
    CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE as API_DOWNLOAD_SURFACE,
    CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE as API_VERSIONS_SURFACE, CallerCredential,
    CloudIacModuleRegistryApiBoundaryContext, CloudIacModuleRegistryApiError,
    CloudIacModuleRegistryAuthzProvider, CloudIacModuleRegistryRouteRequest,
    CloudIacModuleRegistryRouteResponse, route_module_registry_request,
};
use iac_domain::{CloudIacError, ModuleRegistry};
use iac_rest::{
    CloudIacModuleRegistryRestError, CloudIacModuleRegistryRestMatch,
    CloudIacModuleRegistryRestRoute, MODULE_REGISTRY_DISCOVERY_REST_ROUTE,
    MODULE_REGISTRY_DOWNLOAD_REST_ROUTE, MODULE_REGISTRY_REST_METHOD,
    MODULE_REGISTRY_VERSIONS_REST_ROUTE, match_module_registry_rest_route,
};
use http_middleware_kernel::{Handler, HttpRequest, HttpResponse, MiddlewareChain};
use http_router_kernel::{HttpMethod, Router, RouterError};
use http_runtime_hyper_adapter::{
    ServerConfig, SyncHandler, dispatch as dispatch_hyper_adapter_request, handler_to_sync,
};

pub const CLOUD_IAC_MODULE_REGISTRY_RUNTIME_COMPOSITION_NON_CLAIM: &str =
    "in-process-runtime-composition-no-live-http-listener-no-cloud-provisioning";
pub const CLOUD_IAC_MODULE_REGISTRY_HTTP_HANDLER_NON_CLAIM: &str =
    "transport-neutral-http-handler-no-live-listener-no-deployed-endpoint";
pub const CLOUD_IAC_MODULE_REGISTRY_SERVICE_ASSEMBLY_NON_CLAIM: &str =
    "hyper-adapter-service-assembly-no-bind-no-listen-no-deploy";
pub const CLOUD_IAC_MODULE_REGISTRY_LOOPBACK_LISTENER_NON_CLAIM: &str =
    "local-one-connection-loopback-listener-no-deploy-no-production-endpoint";
const JSON_CONTENT_TYPE: &str = "application/json";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIacModuleRegistryRuntimeRequest {
    pub boundary: CloudIacModuleRegistryApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub credential: CallerCredential,                       // data_class: SECRET
    pub method: HttpMethod,                                 // data_class: PUBLIC
    pub path: String,                                       // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIacModuleRegistryRuntimeResponse {
    pub rest_match: CloudIacModuleRegistryRestMatch, // data_class: INTERNAL_ONLY
    pub api_response: CloudIacModuleRegistryRouteResponse, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudIacModuleRegistryRuntimeError {
    Rest(CloudIacModuleRegistryRestError),
    Api(CloudIacModuleRegistryApiError),
    SurfaceContractMismatch {
        route: CloudIacModuleRegistryRestRoute,
        rest_surface: String,
        api_surface: String,
    },
}

pub fn dispatch_module_registry_runtime_request(
    registry: &ModuleRegistry,
    authz_provider: &CloudIacModuleRegistryAuthzProvider,
    request: CloudIacModuleRegistryRuntimeRequest,
) -> Result<CloudIacModuleRegistryRuntimeResponse, CloudIacModuleRegistryRuntimeError> {
    let rest_match = match_module_registry_rest_route(request.method, &request.path)?;
    assert_rest_surface_matches_api_contract(&rest_match)?;

    let api_response = route_module_registry_request(
        registry,
        authz_provider,
        CloudIacModuleRegistryRouteRequest {
            boundary: request.boundary,
            credential: request.credential,
            method: request.method.name().to_string(),
            path: request.path,
        },
    )?;

    Ok(CloudIacModuleRegistryRuntimeResponse {
        rest_match,
        api_response,
    })
}

fn assert_rest_surface_matches_api_contract(
    rest_match: &CloudIacModuleRegistryRestMatch,
) -> Result<(), CloudIacModuleRegistryRuntimeError> {
    let api_surface = match rest_match.route {
        CloudIacModuleRegistryRestRoute::Discovery => API_DISCOVERY_SURFACE,
        CloudIacModuleRegistryRestRoute::Versions => API_VERSIONS_SURFACE,
        CloudIacModuleRegistryRestRoute::Download => API_DOWNLOAD_SURFACE,
    };

    if rest_match.required_surface == api_surface {
        Ok(())
    } else {
        Err(
            CloudIacModuleRegistryRuntimeError::SurfaceContractMismatch {
                route: rest_match.route,
                rest_surface: rest_match.required_surface.clone(),
                api_surface: api_surface.to_string(),
            },
        )
    }
}

impl From<CloudIacModuleRegistryRestError> for CloudIacModuleRegistryRuntimeError {
    fn from(value: CloudIacModuleRegistryRestError) -> Self {
        Self::Rest(value)
    }
}

impl From<CloudIacModuleRegistryApiError> for CloudIacModuleRegistryRuntimeError {
    fn from(value: CloudIacModuleRegistryApiError) -> Self {
        Self::Api(value)
    }
}

#[derive(Clone)]
pub struct CloudIacModuleRegistryHttpHandler {
    registry: ModuleRegistry, // data_class: INTERNAL_ONLY
    boundary: CloudIacModuleRegistryApiBoundaryContext, // data_class: INTERNAL_ONLY
    authz_provider: Arc<CloudIacModuleRegistryAuthzProvider>, // data_class: INTERNAL_ONLY
}

impl CloudIacModuleRegistryHttpHandler {
    pub fn new(
        registry: ModuleRegistry,
        boundary: CloudIacModuleRegistryApiBoundaryContext,
        authz_provider: Arc<CloudIacModuleRegistryAuthzProvider>,
    ) -> Self {
        Self {
            registry,
            boundary,
            authz_provider,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudIacModuleRegistryHttpError {
    UnsupportedMethod { method: String },
    UnexpectedBody { path: String },
    Runtime(CloudIacModuleRegistryRuntimeError),
}

pub fn handle_module_registry_http_request(
    handler: &CloudIacModuleRegistryHttpHandler,
    request: HttpRequest,
) -> HttpResponse {
    match handler.call(request) {
        Ok(response) => response,
        Err(error) => error.into(),
    }
}

impl Handler for CloudIacModuleRegistryHttpHandler {
    type Error = CloudIacModuleRegistryHttpError;

    fn call(&self, request: HttpRequest) -> Result<HttpResponse, Self::Error> {
        if request.method != HttpMethod::Get {
            return Err(CloudIacModuleRegistryHttpError::UnsupportedMethod {
                method: request.method.name().to_string(),
            });
        }
        if !request.body.is_empty() {
            return Err(CloudIacModuleRegistryHttpError::UnexpectedBody { path: request.path });
        }

        // Build the caller credential from the transport headers. The injected
        // authz provider VERIFIES it and PDP-authorizes the surface; the headers
        // are never trusted as an authorization decision (C-class / AUTH-005).
        let credential = CallerCredential {
            authorization: request.headers.get("authorization").cloned(),
        };
        let response = dispatch_module_registry_runtime_request(
            &self.registry,
            &self.authz_provider,
            CloudIacModuleRegistryRuntimeRequest {
                boundary: self.boundary.clone(),
                credential,
                method: request.method,
                path: request.path,
            },
        )
        .map_err(CloudIacModuleRegistryHttpError::Runtime)?;

        Ok(render_module_registry_http_response(response.api_response))
    }
}

impl CloudIacModuleRegistryHttpError {
    pub fn status_code(&self) -> u16 {
        match self {
            Self::UnsupportedMethod { .. } => 405,
            Self::UnexpectedBody { .. } => 400,
            Self::Runtime(error) => runtime_error_status(error),
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedMethod { .. } => "method_not_allowed",
            Self::UnexpectedBody { .. } => "unexpected_body",
            Self::Runtime(error) => runtime_error_code(error),
        }
    }
}

impl From<CloudIacModuleRegistryHttpError> for HttpResponse {
    fn from(value: CloudIacModuleRegistryHttpError) -> Self {
        let status = value.status_code();
        let response = json_response(status, format!(r#"{{"error":"{}"}}"#, value.code()));
        // A 401 MUST advertise the accepted auth scheme (RFC 7235).
        if status == 401 {
            response.with_header("www-authenticate", "Bearer")
        } else {
            response
        }
    }
}

fn render_module_registry_http_response(
    response: CloudIacModuleRegistryRouteResponse,
) -> HttpResponse {
    match response {
        CloudIacModuleRegistryRouteResponse::Discovery(discovery) => json_response(
            200,
            format!(
                r#"{{"modules.v1":{}}}"#,
                json_string(discovery.modules_v1.as_str())
            ),
        ),
        CloudIacModuleRegistryRouteResponse::Versions(versions) => {
            let modules = versions
                .modules
                .iter()
                .map(|module| {
                    let version_entries = module
                        .versions
                        .iter()
                        .map(|entry| {
                            format!(r#"{{"version":{}}}"#, json_string(entry.version.as_str()))
                        })
                        .collect::<Vec<_>>()
                        .join(",");
                    format!(r#"{{"versions":[{version_entries}]}}"#)
                })
                .collect::<Vec<_>>()
                .join(",");
            json_response(200, format!(r#"{{"modules":[{modules}]}}"#))
        }
        CloudIacModuleRegistryRouteResponse::Download(download) => json_response(
            200,
            format!(
                r#"{{"location":{}}}"#,
                json_string(download.location.as_str())
            ),
        ),
    }
}

fn json_response(status: u16, body: String) -> HttpResponse {
    HttpResponse::new(status)
        .with_header("content-type", JSON_CONTENT_TYPE)
        .with_body(body.into_bytes())
}

fn runtime_error_status(error: &CloudIacModuleRegistryRuntimeError) -> u16 {
    match error {
        CloudIacModuleRegistryRuntimeError::Rest(
            CloudIacModuleRegistryRestError::RouteNotFound { .. },
        ) => 404,
        CloudIacModuleRegistryRuntimeError::Rest(CloudIacModuleRegistryRestError::Router(_)) => 500,
        CloudIacModuleRegistryRuntimeError::Api(api_error) => api_error_status(api_error),
        CloudIacModuleRegistryRuntimeError::SurfaceContractMismatch { .. } => 500,
    }
}

fn runtime_error_code(error: &CloudIacModuleRegistryRuntimeError) -> &'static str {
    match error {
        CloudIacModuleRegistryRuntimeError::Rest(
            CloudIacModuleRegistryRestError::RouteNotFound { .. },
        ) => "not_found",
        CloudIacModuleRegistryRuntimeError::Rest(CloudIacModuleRegistryRestError::Router(_)) => {
            "router_error"
        }
        CloudIacModuleRegistryRuntimeError::Api(api_error) => api_error_code(api_error),
        CloudIacModuleRegistryRuntimeError::SurfaceContractMismatch { .. } => {
            "surface_contract_mismatch"
        }
    }
}

fn api_error_status(error: &CloudIacModuleRegistryApiError) -> u16 {
    match error {
        CloudIacModuleRegistryApiError::EmptyRequestId => 400,
        CloudIacModuleRegistryApiError::Unauthenticated => 401,
        CloudIacModuleRegistryApiError::MethodNotAllowed { .. } => 405,
        CloudIacModuleRegistryApiError::RouteNotFound { .. } => 404,
        CloudIacModuleRegistryApiError::Forbidden { .. } => 403,
        CloudIacModuleRegistryApiError::Domain(domain_error) => domain_error_status(domain_error),
    }
}

fn api_error_code(error: &CloudIacModuleRegistryApiError) -> &'static str {
    match error {
        CloudIacModuleRegistryApiError::EmptyRequestId => "empty_request_id",
        CloudIacModuleRegistryApiError::Unauthenticated => "unauthorized",
        CloudIacModuleRegistryApiError::MethodNotAllowed { .. } => "method_not_allowed",
        CloudIacModuleRegistryApiError::RouteNotFound { .. } => "not_found",
        CloudIacModuleRegistryApiError::Forbidden { .. } => "forbidden",
        CloudIacModuleRegistryApiError::Domain(domain_error) => domain_error_code(domain_error),
    }
}

fn domain_error_status(error: &CloudIacError) -> u16 {
    match error {
        CloudIacError::ModuleVersionNotFound => 404,
        _ => 400,
    }
}

fn domain_error_code(error: &CloudIacError) -> &'static str {
    match error {
        CloudIacError::ModuleVersionNotFound => "not_found",
        _ => "invalid_module_request",
    }
}

fn json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => {
                use std::fmt::Write as _;
                let _ = write!(escaped, "\\u{:04x}", ch as u32);
            }
            ch => escaped.push(ch),
        }
    }
    escaped.push('"');
    escaped
}

pub struct CloudIacModuleRegistryHttpServiceAssembly {
    router: Router<SyncHandler>, // data_class: INTERNAL_ONLY
    middleware: MiddlewareChain<HttpRequest, HttpResponse>, // data_class: INTERNAL_ONLY
    server_config: ServerConfig, // data_class: INTERNAL_ONLY
}

impl CloudIacModuleRegistryHttpServiceAssembly {
    pub fn route_count(&self) -> usize {
        self.router.count()
    }

    pub fn middleware_count(&self) -> usize {
        self.middleware.count()
    }

    pub fn server_config(&self) -> &ServerConfig {
        &self.server_config
    }

    pub fn into_serve_parts(
        self,
    ) -> (
        Router<SyncHandler>,
        MiddlewareChain<HttpRequest, HttpResponse>,
        ServerConfig,
    ) {
        (self.router, self.middleware, self.server_config)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudIacModuleRegistryServiceAssemblyError {
    Router(RouterError),
}

pub fn assemble_module_registry_http_service(
    handler: CloudIacModuleRegistryHttpHandler,
) -> Result<CloudIacModuleRegistryHttpServiceAssembly, CloudIacModuleRegistryServiceAssemblyError> {
    let sync_handler = handler_to_sync(handler);
    let mut router = Router::new();
    for template in [
        MODULE_REGISTRY_DISCOVERY_REST_ROUTE,
        MODULE_REGISTRY_VERSIONS_REST_ROUTE,
        MODULE_REGISTRY_DOWNLOAD_REST_ROUTE,
    ] {
        router
            .route(MODULE_REGISTRY_REST_METHOD, template, sync_handler.clone())
            .map_err(CloudIacModuleRegistryServiceAssemblyError::Router)?;
    }

    Ok(CloudIacModuleRegistryHttpServiceAssembly {
        router,
        middleware: MiddlewareChain::new(),
        server_config: ServerConfig::default().with_max_body_bytes(0),
    })
}

pub fn dispatch_module_registry_http_service_request(
    service: &CloudIacModuleRegistryHttpServiceAssembly,
    request: HttpRequest,
) -> HttpResponse {
    dispatch_hyper_adapter_request(request, &service.router, &service.middleware)
}
