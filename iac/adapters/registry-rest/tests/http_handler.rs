//! The HTTP handler renders OpenTofu responses and maps every refusal to a status.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[path = "registry_support/mod.rs"]
mod registry_support;

use registry_support::*;

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
