// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use iac_api::{
    CLOUD_IAC_MODULE_REGISTRY_API_BOUNDARY_NON_CLAIM, CLOUD_IAC_MODULE_REGISTRY_DISCOVERY_SURFACE,
    CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE, CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE,
    CloudIacModuleDownloadApiRequest, CloudIacModuleRegistryApiAuthorization,
    CloudIacModuleRegistryApiBoundaryContext, CloudIacModuleRegistryApiError,
    CloudIacModuleRegistryRouteRequest, CloudIacModuleRegistryRouteResponse,
    CloudIacModuleVersionsApiRequest, OPENTOFU_MODULES_V1_BASE_PATH,
    OPENTOFU_SERVICE_DISCOVERY_PATH, discover_module_registry_from_api,
    get_module_download_from_api, list_module_versions_from_api, route_module_registry_request,
};
use iac_domain::{CloudIacError, ModuleRegistry, OpenTofuModuleRelease};

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
        format!("evidence://cloud-iac/modules/{name}/{version}/local-foundation"),
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
        request_id: "req_cloud_iac_registry_001".to_string(),
    }
}

fn authorization(surfaces: &[&str]) -> CloudIacModuleRegistryApiAuthorization {
    CloudIacModuleRegistryApiAuthorization {
        principal_id: "sp_cloud_iac_registry_reader".to_string(),
        decision_id: "authz_cloud_iac_registry_001".to_string(),
        allowed_surfaces: surfaces
            .iter()
            .map(|surface| (*surface).to_string())
            .collect(),
    }
}

fn route_request(method: &str, path: &str) -> CloudIacModuleRegistryRouteRequest {
    CloudIacModuleRegistryRouteRequest {
        boundary: boundary(),
        authorization: authorization(&[
            CLOUD_IAC_MODULE_REGISTRY_DISCOVERY_SURFACE,
            CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE,
            CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE,
        ]),
        method: method.to_string(),
        path: path.to_string(),
    }
}

#[test]
fn discovery_response_is_opentofu_modules_v1_shape_without_runtime_claim() {
    let response = discover_module_registry_from_api(
        &boundary(),
        &authorization(&[CLOUD_IAC_MODULE_REGISTRY_DISCOVERY_SURFACE]),
    )
    .expect("discovery response is authorized");

    assert_eq!(response.path, OPENTOFU_SERVICE_DISCOVERY_PATH);
    assert_eq!(response.modules_v1, OPENTOFU_MODULES_V1_BASE_PATH);
    assert_eq!(
        CLOUD_IAC_MODULE_REGISTRY_API_BOUNDARY_NON_CLAIM,
        "pure-api-boundary-no-rest-server-no-live-registry-runtime"
    );
}

#[test]
fn versions_response_uses_single_module_array_and_semver_order() {
    let response = list_module_versions_from_api(
        &registry(),
        CloudIacModuleVersionsApiRequest {
            boundary: boundary(),
            authorization: authorization(&[CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE]),
            namespace: "oyatie".to_string(),
            name: "vpc".to_string(),
            system: "opentofu".to_string(),
        },
    )
    .expect("versions response is authorized");

    assert_eq!(response.modules.len(), 1);
    assert_eq!(
        response.modules[0]
            .versions
            .iter()
            .map(|entry| entry.version.as_str())
            .collect::<Vec<_>>(),
        vec!["1.0.0", "1.2.0", "1.10.0"]
    );
}

#[test]
fn download_response_returns_pinned_source_location_for_exact_version() {
    let response = get_module_download_from_api(
        &registry(),
        CloudIacModuleDownloadApiRequest {
            boundary: boundary(),
            authorization: authorization(&[CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE]),
            namespace: "oyatie".to_string(),
            name: "vpc".to_string(),
            system: "opentofu".to_string(),
            version: "1.2.0".to_string(),
        },
    )
    .expect("download response is authorized");

    assert_eq!(
        response.location,
        "git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/cloud-iac/tofu/modules/vpc?ref=v1.2.0"
    );
}

#[test]
fn api_boundary_rejects_missing_auth_and_invalid_path_segments_before_runtime() {
    let unauthorized = list_module_versions_from_api(
        &registry(),
        CloudIacModuleVersionsApiRequest {
            boundary: boundary(),
            authorization: authorization(&[]),
            namespace: "oyatie".to_string(),
            name: "vpc".to_string(),
            system: "opentofu".to_string(),
        },
    )
    .expect_err("missing surface is rejected");

    assert_eq!(
        unauthorized,
        CloudIacModuleRegistryApiError::ForbiddenSurface {
            surface: CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE.to_string()
        }
    );

    let invalid_path = list_module_versions_from_api(
        &registry(),
        CloudIacModuleVersionsApiRequest {
            boundary: boundary(),
            authorization: authorization(&[CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE]),
            namespace: "oyatie".to_string(),
            name: "../vpc".to_string(),
            system: "opentofu".to_string(),
        },
    )
    .expect_err("invalid path segment is rejected");

    assert_eq!(
        invalid_path,
        CloudIacModuleRegistryApiError::Domain(CloudIacError::InvalidModuleName)
    );
}

#[test]
fn api_boundary_rejects_empty_request_id_and_missing_versions() {
    let empty_request_id = get_module_download_from_api(
        &registry(),
        CloudIacModuleDownloadApiRequest {
            boundary: CloudIacModuleRegistryApiBoundaryContext {
                request_id: " ".to_string(),
            },
            authorization: authorization(&[CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE]),
            namespace: "oyatie".to_string(),
            name: "vpc".to_string(),
            system: "opentofu".to_string(),
            version: "1.2.0".to_string(),
        },
    )
    .expect_err("empty request ID is rejected");

    assert_eq!(
        empty_request_id,
        CloudIacModuleRegistryApiError::EmptyRequestId
    );

    let missing = get_module_download_from_api(
        &registry(),
        CloudIacModuleDownloadApiRequest {
            boundary: boundary(),
            authorization: authorization(&[CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE]),
            namespace: "oyatie".to_string(),
            name: "vpc".to_string(),
            system: "opentofu".to_string(),
            version: "9.9.9".to_string(),
        },
    )
    .expect_err("missing version is rejected");

    assert_eq!(
        missing,
        CloudIacModuleRegistryApiError::Domain(CloudIacError::ModuleVersionNotFound)
    );
}

#[test]
fn api_boundary_rejects_empty_authorization_identifiers() {
    let mut empty_principal = authorization(&[CLOUD_IAC_MODULE_REGISTRY_DISCOVERY_SURFACE]);
    empty_principal.principal_id = " ".to_string();

    let principal_error = discover_module_registry_from_api(&boundary(), &empty_principal)
        .expect_err("empty principal id is rejected");

    assert_eq!(
        principal_error,
        CloudIacModuleRegistryApiError::EmptyPrincipalId
    );

    let mut empty_decision = authorization(&[CLOUD_IAC_MODULE_REGISTRY_DISCOVERY_SURFACE]);
    empty_decision.decision_id = " ".to_string();

    let decision_error = discover_module_registry_from_api(&boundary(), &empty_decision)
        .expect_err("empty authorization decision id is rejected");

    assert_eq!(
        decision_error,
        CloudIacModuleRegistryApiError::EmptyAuthorizationDecisionId
    );
}

#[test]
fn debug_output_does_not_contain_secret_like_material() {
    let response = get_module_download_from_api(
        &registry(),
        CloudIacModuleDownloadApiRequest {
            boundary: boundary(),
            authorization: authorization(&[CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE]),
            namespace: "oyatie".to_string(),
            name: "dns".to_string(),
            system: "opentofu".to_string(),
            version: "1.0.0".to_string(),
        },
    )
    .expect("download response is available");

    let debug = format!("{response:?}").to_ascii_lowercase();
    assert!(!debug.contains("token="));
    assert!(!debug.contains("password="));
    assert!(!debug.contains("-----begin"));
    assert!(!debug.contains("kubeconfig"));
}

#[test]
fn route_boundary_dispatches_official_get_paths_into_dtos() {
    let registry = registry();

    let discovery = route_module_registry_request(
        &registry,
        route_request("GET", OPENTOFU_SERVICE_DISCOVERY_PATH),
    )
    .expect("discovery route dispatches");
    assert_eq!(
        discovery,
        CloudIacModuleRegistryRouteResponse::Discovery(
            discover_module_registry_from_api(
                &boundary(),
                &authorization(&[CLOUD_IAC_MODULE_REGISTRY_DISCOVERY_SURFACE])
            )
            .expect("discovery response")
        )
    );

    let versions = route_module_registry_request(
        &registry,
        route_request("GET", "/v1/modules/oyatie/vpc/opentofu/versions"),
    )
    .expect("versions route dispatches");
    assert!(matches!(
        versions,
        CloudIacModuleRegistryRouteResponse::Versions(response)
            if response.modules[0].versions.iter().map(|entry| entry.version.as_str()).collect::<Vec<_>>()
                == vec!["1.0.0", "1.2.0", "1.10.0"]
    ));

    let download = route_module_registry_request(
        &registry,
        route_request("GET", "/v1/modules/oyatie/vpc/opentofu/1.2.0/download"),
    )
    .expect("download route dispatches");
    assert!(matches!(
        download,
        CloudIacModuleRegistryRouteResponse::Download(response)
            if response.location.ends_with("/microservices/cloud-iac/tofu/modules/vpc?ref=v1.2.0")
    ));
}

#[test]
fn route_boundary_rejects_wrong_methods_unknown_paths_and_missing_surfaces() {
    let registry = registry();

    let wrong_method = route_module_registry_request(
        &registry,
        route_request("POST", OPENTOFU_SERVICE_DISCOVERY_PATH),
    )
    .expect_err("non-GET requests are rejected before runtime");
    assert_eq!(
        wrong_method,
        CloudIacModuleRegistryApiError::MethodNotAllowed {
            method: "POST".to_string()
        }
    );

    let unknown_path =
        route_module_registry_request(&registry, route_request("GET", "/v1/modules/oyatie/vpc"))
            .expect_err("incomplete module route is rejected");
    assert_eq!(
        unknown_path,
        CloudIacModuleRegistryApiError::RouteNotFound {
            path: "/v1/modules/oyatie/vpc".to_string()
        }
    );

    let missing_download_surface = route_module_registry_request(
        &registry,
        CloudIacModuleRegistryRouteRequest {
            boundary: boundary(),
            authorization: authorization(&[CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE]),
            method: "GET".to_string(),
            path: "/v1/modules/oyatie/vpc/opentofu/1.2.0/download".to_string(),
        },
    )
    .expect_err("download surface is required");
    assert_eq!(
        missing_download_surface,
        CloudIacModuleRegistryApiError::ForbiddenSurface {
            surface: CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE.to_string()
        }
    );
}

#[test]
fn route_boundary_rejects_whitespace_mutated_method_and_path() {
    let registry = registry();

    let padded_method = route_module_registry_request(
        &registry,
        route_request(" GET ", OPENTOFU_SERVICE_DISCOVERY_PATH),
    )
    .expect_err("method matching is exact");
    assert_eq!(
        padded_method,
        CloudIacModuleRegistryApiError::MethodNotAllowed {
            method: " GET ".to_string()
        }
    );

    let padded_path = route_module_registry_request(
        &registry,
        route_request("GET", " /.well-known/terraform.json "),
    )
    .expect_err("path matching is exact");
    assert_eq!(
        padded_path,
        CloudIacModuleRegistryApiError::RouteNotFound {
            path: " /.well-known/terraform.json ".to_string()
        }
    );
}
