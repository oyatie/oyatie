//! The runtime dispatches, and refuses, through the REST router and the API boundary.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[path = "registry_support/mod.rs"]
mod registry_support;

use registry_support::*;

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
