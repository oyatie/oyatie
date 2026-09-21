//! Service assembly registers routes and serves them through the canonical hyper adapter.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[path = "registry_support/mod.rs"]
mod registry_support;

use registry_support::*;

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
