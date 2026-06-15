// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::sync::Arc;
use std::thread;

use iac_api::{
    CLOUD_IAC_MODULE_REGISTRY_DISCOVERY_SURFACE, CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE,
    CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE, CloudIacModuleRegistryApiAuthorization,
    CloudIacModuleRegistryApiBoundaryContext, CloudIacModuleRegistryApiError,
    CloudIacModuleRegistryRouteResponse, OPENTOFU_MODULES_V1_BASE_PATH,
    OPENTOFU_SERVICE_DISCOVERY_PATH,
};
use iac_domain::{ModuleRegistry, OpenTofuModuleRelease};
use iac_rest::{
    CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE as REST_DOWNLOAD_SURFACE,
    CloudIacModuleRegistryRestError, CloudIacModuleRegistryRestRoute,
    MODULE_REGISTRY_DISCOVERY_REST_ROUTE, MODULE_REGISTRY_DOWNLOAD_REST_ROUTE,
    MODULE_REGISTRY_VERSIONS_REST_ROUTE,
};
use iac_infrastructure::{
    CLOUD_IAC_MODULE_REGISTRY_HTTP_HANDLER_NON_CLAIM,
    CLOUD_IAC_MODULE_REGISTRY_LOOPBACK_LISTENER_NON_CLAIM,
    CLOUD_IAC_MODULE_REGISTRY_RUNTIME_COMPOSITION_NON_CLAIM,
    CLOUD_IAC_MODULE_REGISTRY_SERVICE_ASSEMBLY_NON_CLAIM, CloudIacModuleRegistryHttpHandler,
    CloudIacModuleRegistryRuntimeError, CloudIacModuleRegistryRuntimeRequest,
    assemble_module_registry_http_service, dispatch_module_registry_http_service_request,
    dispatch_module_registry_runtime_request, handle_module_registry_http_request,
};
use oya_http_middleware_kernel::HttpRequest;
use oya_http_router_kernel::HttpMethod;
use oya_http_runtime_hyper_adapter::serve_one_connection_on_std_listener;

fn release(name: &str, version: &str, digest_hex: char) -> OpenTofuModuleRelease {
    OpenTofuModuleRelease::new(
        "oyatie",
        name,
        "opentofu",
        version,
        format!(
            "git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/cloud-iac/tofu/modules/{name}?ref=v{version}"
        ),
        format!("sha256:{}", digest_hex.to_string().repeat(64)),
        format!("evidence://cloud-iac/modules/{name}/{version}/runtime-composition"),
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
        request_id: "req_cloud_iac_registry_runtime_001".to_string(),
    }
}

fn authorization(surfaces: &[&str]) -> CloudIacModuleRegistryApiAuthorization {
    CloudIacModuleRegistryApiAuthorization {
        principal_id: "sp_cloud_iac_registry_runtime_reader".to_string(),
        decision_id: "authz_cloud_iac_registry_runtime_001".to_string(),
        allowed_surfaces: surfaces
            .iter()
            .map(|surface| (*surface).to_string())
            .collect(),
    }
}

fn all_surfaces() -> CloudIacModuleRegistryApiAuthorization {
    authorization(&[
        CLOUD_IAC_MODULE_REGISTRY_DISCOVERY_SURFACE,
        CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE,
        CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE,
    ])
}

fn runtime_request(
    method: HttpMethod,
    path: &str,
    authorization: CloudIacModuleRegistryApiAuthorization,
) -> CloudIacModuleRegistryRuntimeRequest {
    CloudIacModuleRegistryRuntimeRequest {
        boundary: boundary(),
        authorization,
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

fn body_text(response: &oya_http_middleware_kernel::HttpResponse) -> String {
    String::from_utf8(response.body.clone()).expect("response body is UTF-8 JSON")
}

#[test]
fn runtime_dispatches_discovery_versions_and_download_through_rest_router_and_api_boundary() {
    let registry = registry();

    let discovery = dispatch_module_registry_runtime_request(
        &registry,
        runtime_request(
            HttpMethod::Get,
            OPENTOFU_SERVICE_DISCOVERY_PATH,
            all_surfaces(),
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
        CloudIacModuleRegistryRouteResponse::Discovery(
            iac_api::ModuleRegistryDiscoveryResponse {
                path: OPENTOFU_SERVICE_DISCOVERY_PATH.to_string(),
                modules_v1: OPENTOFU_MODULES_V1_BASE_PATH.to_string(),
            }
        )
    );

    let versions = dispatch_module_registry_runtime_request(
        &registry,
        runtime_request(
            HttpMethod::Get,
            "/v1/modules/oyatie/vpc/opentofu/versions",
            all_surfaces(),
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
        runtime_request(
            HttpMethod::Get,
            "/v1/modules/oyatie/vpc/opentofu/1.2.0/download",
            all_surfaces(),
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
            if response.location.ends_with("/microservices/cloud-iac/tofu/modules/vpc?ref=v1.2.0")
    ));
}

#[test]
fn runtime_rejects_wrong_method_unknown_path_dot_segment_and_whitespace_before_api_dispatch() {
    let registry = registry();

    for (method, path) in [
        (HttpMethod::Post, OPENTOFU_SERVICE_DISCOVERY_PATH),
        (HttpMethod::Get, "/v1/modules/oyatie/vpc"),
        (HttpMethod::Get, "/v1/modules/oyatie/../opentofu/versions"),
        (HttpMethod::Get, " /.well-known/terraform.json "),
    ] {
        let error = dispatch_module_registry_runtime_request(
            &registry,
            runtime_request(method, path, all_surfaces()),
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
fn runtime_preserves_route_specific_surface_authorization_at_api_boundary() {
    let error = dispatch_module_registry_runtime_request(
        &registry(),
        runtime_request(
            HttpMethod::Get,
            "/v1/modules/oyatie/vpc/opentofu/1.2.0/download",
            authorization(&[CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE]),
        ),
    )
    .expect_err("download dispatch requires download surface, not versions surface");

    assert_eq!(
        error,
        CloudIacModuleRegistryRuntimeError::Api(CloudIacModuleRegistryApiError::ForbiddenSurface {
            surface: CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE.to_string(),
        },)
    );
}

#[test]
fn runtime_still_validates_api_boundary_context_and_makes_no_live_runtime_claim() {
    let error = dispatch_module_registry_runtime_request(
        &registry(),
        CloudIacModuleRegistryRuntimeRequest {
            boundary: CloudIacModuleRegistryApiBoundaryContext {
                request_id: " ".to_string(),
            },
            authorization: all_surfaces(),
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
    let handler = CloudIacModuleRegistryHttpHandler::new(registry(), boundary(), all_surfaces());

    let discovery = handle_module_registry_http_request(
        &handler,
        http_request(HttpMethod::Get, OPENTOFU_SERVICE_DISCOVERY_PATH),
    );
    assert_eq!(discovery.status, 200);
    assert_eq!(
        discovery.headers.get("content-type").map(String::as_str),
        Some("application/json")
    );
    assert_eq!(body_text(&discovery), r#"{"modules.v1":"/v1/modules/"}"#);

    let versions = handle_module_registry_http_request(
        &handler,
        http_request(HttpMethod::Get, "/v1/modules/oyatie/vpc/opentofu/versions"),
    );
    assert_eq!(versions.status, 200);
    assert_eq!(
        body_text(&versions),
        r#"{"modules":[{"versions":[{"version":"1.0.0"},{"version":"1.2.0"},{"version":"1.10.0"}]}]}"#
    );

    let download = handle_module_registry_http_request(
        &handler,
        http_request(
            HttpMethod::Get,
            "/v1/modules/oyatie/vpc/opentofu/1.2.0/download",
        ),
    );
    assert_eq!(download.status, 200);
    assert_eq!(
        body_text(&download),
        r#"{"location":"git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/cloud-iac/tofu/modules/vpc?ref=v1.2.0"}"#
    );
    assert_eq!(
        CLOUD_IAC_MODULE_REGISTRY_HTTP_HANDLER_NON_CLAIM,
        "transport-neutral-http-handler-no-live-listener-no-deployed-endpoint"
    );
}

#[test]
fn http_handler_maps_route_auth_domain_and_body_errors_to_http_statuses() {
    let handler = CloudIacModuleRegistryHttpHandler::new(registry(), boundary(), all_surfaces());

    let wrong_method = handle_module_registry_http_request(
        &handler,
        http_request(HttpMethod::Post, OPENTOFU_SERVICE_DISCOVERY_PATH),
    );
    assert_eq!(wrong_method.status, 405);

    let unknown = handle_module_registry_http_request(
        &handler,
        http_request(HttpMethod::Get, "/v1/modules/oyatie/vpc"),
    );
    assert_eq!(unknown.status, 404);

    let missing_version = handle_module_registry_http_request(
        &handler,
        http_request(
            HttpMethod::Get,
            "/v1/modules/oyatie/vpc/opentofu/9.9.9/download",
        ),
    );
    assert_eq!(missing_version.status, 404);

    let forbidden_handler = CloudIacModuleRegistryHttpHandler::new(
        registry(),
        boundary(),
        authorization(&[CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE]),
    );
    let forbidden = handle_module_registry_http_request(
        &forbidden_handler,
        http_request(
            HttpMethod::Get,
            "/v1/modules/oyatie/vpc/opentofu/1.2.0/download",
        ),
    );
    assert_eq!(forbidden.status, 403);

    let mut get_with_body = http_request(HttpMethod::Get, OPENTOFU_SERVICE_DISCOVERY_PATH);
    get_with_body.body = b"unexpected".to_vec();
    let unexpected_body = handle_module_registry_http_request(&handler, get_with_body);
    assert_eq!(unexpected_body.status, 400);
}

#[test]
fn service_assembly_registers_opentofu_routes_with_safe_config_without_listener_claim() {
    let service = assemble_module_registry_http_service(CloudIacModuleRegistryHttpHandler::new(
        registry(),
        boundary(),
        all_surfaces(),
    ))
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
    let service = assemble_module_registry_http_service(CloudIacModuleRegistryHttpHandler::new(
        registry(),
        boundary(),
        all_surfaces(),
    ))
    .expect("service assembly registers routes");

    let versions = dispatch_module_registry_http_service_request(
        &service,
        http_request(HttpMethod::Get, "/v1/modules/oyatie/vpc/opentofu/versions"),
    );
    assert_eq!(versions.status, 200);
    assert_eq!(
        body_text(&versions),
        r#"{"modules":[{"versions":[{"version":"1.0.0"},{"version":"1.2.0"},{"version":"1.10.0"}]}]}"#
    );

    let unknown = dispatch_module_registry_http_service_request(
        &service,
        http_request(HttpMethod::Get, "/v1/modules/oyatie/vpc"),
    );
    assert_eq!(unknown.status, 404);
    assert_eq!(body_text(&unknown), "not found");
}

#[test]
fn service_assembly_preserves_method_not_allowed_through_canonical_adapter_path() {
    let service = assemble_module_registry_http_service(CloudIacModuleRegistryHttpHandler::new(
        registry(),
        boundary(),
        all_surfaces(),
    ))
    .expect("service assembly registers routes");

    let wrong_method = dispatch_module_registry_http_service_request(
        &service,
        http_request(HttpMethod::Post, OPENTOFU_SERVICE_DISCOVERY_PATH),
    );

    assert_eq!(wrong_method.status, 405);
    assert_eq!(body_text(&wrong_method), "method not allowed");
}

#[test]
fn loopback_listener_serves_discovery_through_hyper_boundary_without_deploy_claim() {
    let service = assemble_module_registry_http_service(CloudIacModuleRegistryHttpHandler::new(
        registry(),
        boundary(),
        all_surfaces(),
    ))
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
                b"GET /.well-known/terraform.json HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
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
