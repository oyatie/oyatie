// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::sync::Arc;
use std::thread;

use iac_api::{
    CLOUD_IAC_MODULE_REGISTRY_DISCOVERY_SURFACE, CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE,
    CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE, CallerCredential,
    CloudIacModuleRegistryApiBoundaryContext, CloudIacModuleRegistryApiError,
    CloudIacModuleRegistryAuthzProvider, CloudIacModuleRegistryRouteResponse,
    ConfiguredBearerPrincipalVerifier, ConfiguredSurfaceAuthorizer,
    ModuleRegistryAuthorizationError, ModuleRegistryAuthorizer, OPENTOFU_MODULES_V1_BASE_PATH,
    OPENTOFU_SERVICE_DISCOVERY_PATH, VerifiedPrincipal,
};
use iac_domain::{ModuleRegistry, OpenTofuModuleRelease};
use iac_infrastructure::{
    CLOUD_IAC_MODULE_REGISTRY_HTTP_HANDLER_NON_CLAIM,
    CLOUD_IAC_MODULE_REGISTRY_LOOPBACK_LISTENER_NON_CLAIM,
    CLOUD_IAC_MODULE_REGISTRY_RUNTIME_COMPOSITION_NON_CLAIM,
    CLOUD_IAC_MODULE_REGISTRY_SERVICE_ASSEMBLY_NON_CLAIM, CloudIacModuleRegistryHttpHandler,
    CloudIacModuleRegistryRuntimeError, CloudIacModuleRegistryRuntimeRequest,
    assemble_module_registry_http_service, dispatch_module_registry_http_service_request,
    dispatch_module_registry_runtime_request, handle_module_registry_http_request,
};
use iac_rest::{
    CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE as REST_DOWNLOAD_SURFACE,
    CloudIacModuleRegistryRestError, CloudIacModuleRegistryRestRoute,
    MODULE_REGISTRY_DISCOVERY_REST_ROUTE, MODULE_REGISTRY_DOWNLOAD_REST_ROUTE,
    MODULE_REGISTRY_VERSIONS_REST_ROUTE,
};
use http_middleware_kernel::HttpRequest;
use http_router_kernel::HttpMethod;
use http_runtime_hyper_adapter::serve_one_connection_on_std_listener;

const BEARER_SECRET: &str = "break-glass-iac-registry-secret";
const PRINCIPAL_ID: &str = "sp_iac_app_registry_reader";

struct AllowAllAuthorizer;
impl ModuleRegistryAuthorizer for AllowAllAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _surface: &str,
    ) -> Result<(), ModuleRegistryAuthorizationError> {
        Ok(())
    }
}

struct DenyAllAuthorizer;
impl ModuleRegistryAuthorizer for DenyAllAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _surface: &str,
    ) -> Result<(), ModuleRegistryAuthorizationError> {
        Err(ModuleRegistryAuthorizationError::Denied)
    }
}

struct RefuseAuthorizer;
impl ModuleRegistryAuthorizer for RefuseAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _surface: &str,
    ) -> Result<(), ModuleRegistryAuthorizationError> {
        Err(ModuleRegistryAuthorizationError::Refused)
    }
}

fn provider_with(
    authorizer: Arc<dyn ModuleRegistryAuthorizer>,
) -> CloudIacModuleRegistryAuthzProvider {
    let verifier = Arc::new(
        ConfiguredBearerPrincipalVerifier::new(BEARER_SECRET, PRINCIPAL_ID)
            .expect("valid break-glass verifier config"),
    );
    CloudIacModuleRegistryAuthzProvider::new(verifier, authorizer)
}

fn reader_provider(surfaces: &[&str]) -> CloudIacModuleRegistryAuthzProvider {
    let verifier = Arc::new(
        ConfiguredBearerPrincipalVerifier::new(BEARER_SECRET, PRINCIPAL_ID)
            .expect("valid break-glass verifier config"),
    );
    let authorizer = Arc::new(ConfiguredSurfaceAuthorizer::new(
        surfaces.iter().map(|surface| (*surface).to_string()),
    ));
    CloudIacModuleRegistryAuthzProvider::new(verifier, authorizer)
}

fn all_reader_provider() -> CloudIacModuleRegistryAuthzProvider {
    reader_provider(&[
        CLOUD_IAC_MODULE_REGISTRY_DISCOVERY_SURFACE,
        CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE,
        CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE,
    ])
}

fn allow_all_handler() -> CloudIacModuleRegistryHttpHandler {
    CloudIacModuleRegistryHttpHandler::new(
        registry(),
        boundary(),
        Arc::new(provider_with(Arc::new(AllowAllAuthorizer))),
    )
}

fn valid_credential() -> CallerCredential {
    CallerCredential {
        authorization: Some(format!("Bearer {BEARER_SECRET}")),
    }
}

fn release(name: &str, version: &str, digest_hex: char) -> OpenTofuModuleRelease {
    OpenTofuModuleRelease::new(
        "oyatie",
        name,
        "opentofu",
        version,
        format!(
            "git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/iac-app/tofu/modules/{name}?ref=v{version}"
        ),
        format!("sha256:{}", digest_hex.to_string().repeat(64)),
        format!("evidence://iac-app/modules/{name}/{version}/runtime-composition"),
    )
    .expect("valid module release")
}

fn registry() -> ModuleRegistry {
    let mut registry = ModuleRegistry::default();
    registry
        .publish(release("vpc", "1.10.0", 'c'))
        .expect("vpc 1.10.0 registers");
    registry
        .publish(release("vpc", "1.0.0", 'a'))
        .expect("vpc 1.0.0 registers");
    registry
        .publish(release("vpc", "1.2.0", 'b'))
        .expect("vpc 1.2.0 registers");
    registry
        .publish(release("dns", "1.0.0", 'd'))
        .expect("dns registers");
    registry
}

fn boundary() -> CloudIacModuleRegistryApiBoundaryContext {
    CloudIacModuleRegistryApiBoundaryContext {
        request_id: "req_iac_app_registry_runtime_001".to_string(),
    }
}

fn runtime_request(
    method: HttpMethod,
    path: &str,
    credential: CallerCredential,
) -> CloudIacModuleRegistryRuntimeRequest {
    CloudIacModuleRegistryRuntimeRequest {
        boundary: boundary(),
        credential,
        method,
        path: path.to_string(),
    }
}

fn http_request(method: HttpMethod, path: &str) -> HttpRequest {
    HttpRequest {
        method,
        path: path.to_string(),
        headers: BTreeMap::new(),
        body: Vec::new(),
        path_captures: BTreeMap::new(),
        matched_template: None,
    }
}

fn http_request_with_bearer(method: HttpMethod, path: &str, bearer: &str) -> HttpRequest {
    let mut request = http_request(method, path);
    request
        .headers
        .insert("authorization".to_string(), format!("Bearer {bearer}"));
    request
}

fn body_text(response: &http_middleware_kernel::HttpResponse) -> String {
    String::from_utf8(response.body.clone()).expect("response body is UTF-8 JSON")
}

#[test]
fn runtime_dispatches_discovery_versions_and_download_through_rest_router_and_api_boundary() {
    let registry = registry();
    let provider = all_reader_provider();

    let discovery = dispatch_module_registry_runtime_request(
        &registry,
        &provider,
        runtime_request(
            HttpMethod::Get,
            OPENTOFU_SERVICE_DISCOVERY_PATH,
            valid_credential(),
        ),
    )
    .expect("discovery dispatches");
    assert_eq!(
        discovery.rest_match.route,
        CloudIacModuleRegistryRestRoute::Discovery
    );
    assert_eq!(
        discovery.rest_match.matched_template,
        MODULE_REGISTRY_DISCOVERY_REST_ROUTE
    );
    assert_eq!(
        discovery.api_response,
        CloudIacModuleRegistryRouteResponse::Discovery(iac_api::ModuleRegistryDiscoveryResponse {
            path: OPENTOFU_SERVICE_DISCOVERY_PATH.to_string(),
            modules_v1: OPENTOFU_MODULES_V1_BASE_PATH.to_string(),
        })
    );

    let versions = dispatch_module_registry_runtime_request(
        &registry,
        &provider,
        runtime_request(
            HttpMethod::Get,
            "/v1/modules/oyatie/vpc/opentofu/versions",
            valid_credential(),
        ),
    )
    .expect("versions dispatches");
    assert_eq!(
        versions.rest_match.route,
        CloudIacModuleRegistryRestRoute::Versions
    );
    assert_eq!(
        versions.rest_match.matched_template,
        MODULE_REGISTRY_VERSIONS_REST_ROUTE
    );
    assert_eq!(
        versions.rest_match.captures.get("namespace").unwrap(),
        "oyatie"
    );
    assert!(!versions.rest_match.matched_template.contains("oyatie"));
    assert!(matches!(
        versions.api_response,
        CloudIacModuleRegistryRouteResponse::Versions(response)
            if response.modules[0]
                .versions
                .iter()
                .map(|entry| entry.version.as_str())
                .collect::<Vec<_>>() == vec!["1.0.0", "1.2.0", "1.10.0"]
    ));

    let download = dispatch_module_registry_runtime_request(
        &registry,
        &provider,
        runtime_request(
            HttpMethod::Get,
            "/v1/modules/oyatie/vpc/opentofu/1.2.0/download",
            valid_credential(),
        ),
    )
    .expect("download dispatches");
    assert_eq!(
        download.rest_match.route,
        CloudIacModuleRegistryRestRoute::Download
    );
    assert_eq!(
        download.rest_match.matched_template,
        MODULE_REGISTRY_DOWNLOAD_REST_ROUTE
    );
    assert_eq!(download.rest_match.required_surface, REST_DOWNLOAD_SURFACE);
    assert!(matches!(
        download.api_response,
        CloudIacModuleRegistryRouteResponse::Download(response)
            if response.location.ends_with("/microservices/iac-app/tofu/modules/vpc?ref=v1.2.0")
    ));
}

#[test]
fn runtime_rejects_wrong_method_unknown_path_dot_segment_and_whitespace_before_api_dispatch() {
    let registry = registry();
    let provider = all_reader_provider();

    for (method, path) in [
        (HttpMethod::Post, OPENTOFU_SERVICE_DISCOVERY_PATH),
        (HttpMethod::Get, "/v1/modules/oyatie/vpc"),
        (HttpMethod::Get, "/v1/modules/oyatie/../opentofu/versions"),
        (HttpMethod::Get, " /.well-known/terraform.json "),
    ] {
        let error = dispatch_module_registry_runtime_request(
            &registry,
            &provider,
            runtime_request(method, path, valid_credential()),
        )
        .expect_err("invalid route is rejected by REST router composition before API dispatch");
        assert!(matches!(
            error,
            CloudIacModuleRegistryRuntimeError::Rest(
                CloudIacModuleRegistryRestError::RouteNotFound { .. }
            )
        ));
    }
}

#[test]
fn runtime_verifies_credential_and_pdp_authorizes_surface_at_api_boundary() {
    let registry = registry();

    // Missing credential → Unauthenticated even though the PDP would allow.
    let missing = dispatch_module_registry_runtime_request(
        &registry,
        &provider_with(Arc::new(AllowAllAuthorizer)),
        runtime_request(
            HttpMethod::Get,
            "/v1/modules/oyatie/vpc/opentofu/1.2.0/download",
            CallerCredential {
                authorization: None,
            },
        ),
    )
    .expect_err("absent credential is rejected at the API boundary");
    assert_eq!(
        missing,
        CloudIacModuleRegistryRuntimeError::Api(CloudIacModuleRegistryApiError::Unauthenticated)
    );

    // Forged bearer → Unauthenticated.
    let forged = dispatch_module_registry_runtime_request(
        &registry,
        &provider_with(Arc::new(AllowAllAuthorizer)),
        runtime_request(
            HttpMethod::Get,
            "/v1/modules/oyatie/vpc/opentofu/1.2.0/download",
            CallerCredential {
                authorization: Some("Bearer not-the-real-secret".to_string()),
            },
        ),
    )
    .expect_err("forged bearer is rejected at the API boundary");
    assert_eq!(
        forged,
        CloudIacModuleRegistryRuntimeError::Api(CloudIacModuleRegistryApiError::Unauthenticated)
    );

    // A provider permitting only versions must Forbid a download (deny-by-default).
    let denied = dispatch_module_registry_runtime_request(
        &registry,
        &reader_provider(&[CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE]),
        runtime_request(
            HttpMethod::Get,
            "/v1/modules/oyatie/vpc/opentofu/1.2.0/download",
            valid_credential(),
        ),
    )
    .expect_err("download dispatch requires download surface, not versions surface");
    assert_eq!(
        denied,
        CloudIacModuleRegistryRuntimeError::Api(CloudIacModuleRegistryApiError::Forbidden {
            surface: CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE.to_string(),
        })
    );

    // A PDP fault must fail closed to Forbidden, never a runtime/5xx surprise.
    let faulted = dispatch_module_registry_runtime_request(
        &registry,
        &provider_with(Arc::new(RefuseAuthorizer)),
        runtime_request(
            HttpMethod::Get,
            "/v1/modules/oyatie/vpc/opentofu/1.2.0/download",
            valid_credential(),
        ),
    )
    .expect_err("PDP fault fails closed");
    assert_eq!(
        faulted,
        CloudIacModuleRegistryRuntimeError::Api(CloudIacModuleRegistryApiError::Forbidden {
            surface: CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE.to_string(),
        })
    );
}

#[test]
fn runtime_still_validates_api_boundary_context_and_makes_no_live_runtime_claim() {
    let error = dispatch_module_registry_runtime_request(
        &registry(),
        &all_reader_provider(),
        CloudIacModuleRegistryRuntimeRequest {
            boundary: CloudIacModuleRegistryApiBoundaryContext {
                request_id: " ".to_string(),
            },
            credential: valid_credential(),
            method: HttpMethod::Get,
            path: OPENTOFU_SERVICE_DISCOVERY_PATH.to_string(),
        },
    )
    .expect_err("empty API boundary request id is still rejected");

    assert_eq!(
        error,
        CloudIacModuleRegistryRuntimeError::Api(CloudIacModuleRegistryApiError::EmptyRequestId)
    );
    assert_eq!(
        CLOUD_IAC_MODULE_REGISTRY_RUNTIME_COMPOSITION_NON_CLAIM,
        "in-process-runtime-composition-no-live-http-listener-no-cloud-provisioning"
    );
}

#[test]
fn http_handler_renders_opentofu_discovery_versions_and_download_responses() {
    let handler = allow_all_handler();

    let discovery = handle_module_registry_http_request(
        &handler,
        http_request_with_bearer(
            HttpMethod::Get,
            OPENTOFU_SERVICE_DISCOVERY_PATH,
            BEARER_SECRET,
        ),
    );
    assert_eq!(discovery.status, 200);
    assert_eq!(
        discovery.headers.get("content-type").map(String::as_str),
        Some("application/json")
    );
    assert_eq!(body_text(&discovery), r#"{"modules.v1":"/v1/modules/"}"#);

    let versions = handle_module_registry_http_request(
        &handler,
        http_request_with_bearer(
            HttpMethod::Get,
            "/v1/modules/oyatie/vpc/opentofu/versions",
            BEARER_SECRET,
        ),
    );
    assert_eq!(versions.status, 200);
    assert_eq!(
        body_text(&versions),
        r#"{"modules":[{"versions":[{"version":"1.0.0"},{"version":"1.2.0"},{"version":"1.10.0"}]}]}"#
    );

    let download = handle_module_registry_http_request(
        &handler,
        http_request_with_bearer(
            HttpMethod::Get,
            "/v1/modules/oyatie/vpc/opentofu/1.2.0/download",
            BEARER_SECRET,
        ),
    );
    assert_eq!(download.status, 200);
    assert_eq!(
        body_text(&download),
        r#"{"location":"git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/iac-app/tofu/modules/vpc?ref=v1.2.0"}"#
    );
    assert_eq!(
        CLOUD_IAC_MODULE_REGISTRY_HTTP_HANDLER_NON_CLAIM,
        "transport-neutral-http-handler-no-live-listener-no-deployed-endpoint"
    );
}

#[test]
fn http_handler_rejects_absent_and_forged_credentials_with_401() {
    let handler = allow_all_handler();

    let missing = handle_module_registry_http_request(
        &handler,
        http_request(HttpMethod::Get, "/v1/modules/oyatie/vpc/opentofu/versions"),
    );
    assert_eq!(missing.status, 401);
    assert_eq!(body_text(&missing), r#"{"error":"unauthorized"}"#);
    assert_eq!(
        missing.headers.get("www-authenticate").map(String::as_str),
        Some("Bearer")
    );

    let forged = handle_module_registry_http_request(
        &handler,
        http_request_with_bearer(
            HttpMethod::Get,
            "/v1/modules/oyatie/vpc/opentofu/versions",
            "not-the-real-secret",
        ),
    );
    assert_eq!(forged.status, 401);
    assert_eq!(
        forged.headers.get("www-authenticate").map(String::as_str),
        Some("Bearer")
    );
}

#[test]
fn http_handler_maps_route_auth_domain_and_body_errors_to_http_statuses() {
    let handler = allow_all_handler();

    let wrong_method = handle_module_registry_http_request(
        &handler,
        http_request_with_bearer(
            HttpMethod::Post,
            OPENTOFU_SERVICE_DISCOVERY_PATH,
            BEARER_SECRET,
        ),
    );
    assert_eq!(wrong_method.status, 405);

    let unknown = handle_module_registry_http_request(
        &handler,
        http_request_with_bearer(HttpMethod::Get, "/v1/modules/oyatie/vpc", BEARER_SECRET),
    );
    assert_eq!(unknown.status, 404);

    let missing_version = handle_module_registry_http_request(
        &handler,
        http_request_with_bearer(
            HttpMethod::Get,
            "/v1/modules/oyatie/vpc/opentofu/9.9.9/download",
            BEARER_SECRET,
        ),
    );
    assert_eq!(missing_version.status, 404);

    // A provider permitting only versions must answer a download with 403.
    let forbidden_handler = CloudIacModuleRegistryHttpHandler::new(
        registry(),
        boundary(),
        Arc::new(reader_provider(&[
            CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE,
        ])),
    );
    let forbidden = handle_module_registry_http_request(
        &forbidden_handler,
        http_request_with_bearer(
            HttpMethod::Get,
            "/v1/modules/oyatie/vpc/opentofu/1.2.0/download",
            BEARER_SECRET,
        ),
    );
    assert_eq!(forbidden.status, 403);

    let mut get_with_body = http_request_with_bearer(
        HttpMethod::Get,
        OPENTOFU_SERVICE_DISCOVERY_PATH,
        BEARER_SECRET,
    );
    get_with_body.body = b"unexpected".to_vec();
    let unexpected_body = handle_module_registry_http_request(&handler, get_with_body);
    assert_eq!(unexpected_body.status, 400);
}

#[test]
fn service_assembly_registers_opentofu_routes_with_safe_config_without_listener_claim() {
    let service = assemble_module_registry_http_service(allow_all_handler())
        .expect("service assembly registers routes");

    assert_eq!(service.route_count(), 3);
    assert_eq!(service.middleware_count(), 0);
    assert_eq!(service.server_config().max_body_bytes, 0);
    assert_eq!(
        CLOUD_IAC_MODULE_REGISTRY_SERVICE_ASSEMBLY_NON_CLAIM,
        "hyper-adapter-service-assembly-no-bind-no-listen-no-deploy"
    );
}

#[test]
fn service_assembly_dispatches_through_canonical_hyper_adapter_path() {
    let service = assemble_module_registry_http_service(allow_all_handler())
        .expect("service assembly registers routes");

    let versions = dispatch_module_registry_http_service_request(
        &service,
        http_request_with_bearer(
            HttpMethod::Get,
            "/v1/modules/oyatie/vpc/opentofu/versions",
            BEARER_SECRET,
        ),
    );
    assert_eq!(versions.status, 200);
    assert_eq!(
        body_text(&versions),
        r#"{"modules":[{"versions":[{"version":"1.0.0"},{"version":"1.2.0"},{"version":"1.10.0"}]}]}"#
    );

    let unauthorized = dispatch_module_registry_http_service_request(
        &service,
        http_request(HttpMethod::Get, "/v1/modules/oyatie/vpc/opentofu/versions"),
    );
    assert_eq!(unauthorized.status, 401);

    let unknown = dispatch_module_registry_http_service_request(
        &service,
        http_request_with_bearer(HttpMethod::Get, "/v1/modules/oyatie/vpc", BEARER_SECRET),
    );
    assert_eq!(unknown.status, 404);
    assert_eq!(body_text(&unknown), "not found");
}

#[test]
fn service_assembly_preserves_method_not_allowed_through_canonical_adapter_path() {
    let service = assemble_module_registry_http_service(allow_all_handler())
        .expect("service assembly registers routes");

    let wrong_method = dispatch_module_registry_http_service_request(
        &service,
        http_request_with_bearer(
            HttpMethod::Post,
            OPENTOFU_SERVICE_DISCOVERY_PATH,
            BEARER_SECRET,
        ),
    );

    assert_eq!(wrong_method.status, 405);
    assert_eq!(body_text(&wrong_method), "method not allowed");
}

#[test]
fn loopback_listener_serves_discovery_through_hyper_boundary_without_deploy_claim() {
    let service = assemble_module_registry_http_service(allow_all_handler())
        .expect("service assembly registers routes");
    let (router, middleware, server_config) = service.into_serve_parts();
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .expect("bind local loopback listener for deterministic test harness");
    let addr = listener
        .local_addr()
        .expect("loopback listener exposes a local addr");

    let server = thread::spawn(move || {
        serve_one_connection_on_std_listener(
            listener,
            Arc::new(router),
            Arc::new(middleware),
            server_config,
        )
    });

    let response = {
        let mut stream =
            std::net::TcpStream::connect(addr).expect("connect to local loopback harness");
        stream
            .write_all(
                format!(
                    "GET /.well-known/terraform.json HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {BEARER_SECRET}\r\nConnection: close\r\n\r\n"
                )
                .as_bytes(),
            )
            .expect("write request bytes");
        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .expect("read response bytes");
        response
    };

    server
        .join()
        .expect("loopback server thread joins")
        .expect("one loopback connection served");
    assert!(response.starts_with("HTTP/1.1 200 OK"));
    assert!(response.contains("content-type: application/json"));
    assert!(response.contains(r#"{"modules.v1":"/v1/modules/"}"#));
    assert_eq!(
        CLOUD_IAC_MODULE_REGISTRY_LOOPBACK_LISTENER_NON_CLAIM,
        "local-one-connection-loopback-listener-no-deploy-no-production-endpoint"
    );
}
