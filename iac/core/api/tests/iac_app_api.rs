// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use iac_api::{
    CLOUD_IAC_MODULE_REGISTRY_API_BOUNDARY_NON_CLAIM, CLOUD_IAC_MODULE_REGISTRY_DISCOVERY_SURFACE,
    CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE, CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE,
    CallerCredential, CloudIacModuleDownloadApiRequest, CloudIacModuleRegistryApiBoundaryContext,
    CloudIacModuleRegistryApiError, CloudIacModuleRegistryAuthzProvider,
    CloudIacModuleRegistryRouteRequest, CloudIacModuleRegistryRouteResponse,
    CloudIacModuleVersionsApiRequest, ConfiguredBearerPrincipalVerifier,
    ConfiguredSurfaceAuthorizer, ModuleRegistryAuthorizationError, ModuleRegistryAuthorizer,
    OPENTOFU_MODULES_V1_BASE_PATH, OPENTOFU_SERVICE_DISCOVERY_PATH, VerifiedPrincipal,
    constant_time_eq, discover_module_registry_from_api, get_module_download_from_api,
    list_module_versions_from_api, route_module_registry_request,
};
use iac_domain::{CloudIacError, ModuleRegistry, OpenTofuModuleRelease};

const BEARER_SECRET: &str = "break-glass-iac-registry-secret";
const PRINCIPAL_ID: &str = "sp_iac_app_registry_reader";

/// A test PDP that allows every surface — used to PROVE the credential gate
/// fails closed even when the PDP would otherwise allow.
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

/// A test PDP that denies every surface (proves PDP-deny → Forbidden).
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

/// A test PDP that refuses (fault) — proves a PDP fault is fail-closed to
/// Forbidden, never a 5xx / panic.
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

fn allow_all_provider() -> CloudIacModuleRegistryAuthzProvider {
    provider_with(Arc::new(AllowAllAuthorizer))
}

/// A provider whose real `ConfiguredSurfaceAuthorizer` permits ONLY `surfaces`
/// (deny-by-default for anything else) — proves the break-glass authorizer
/// enforces surface scope.
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
        format!("evidence://iac-app/modules/{name}/{version}/local-foundation"),
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
        request_id: "req_iac_app_registry_001".to_string(),
    }
}

fn route_request(method: &str, path: &str) -> CloudIacModuleRegistryRouteRequest {
    CloudIacModuleRegistryRouteRequest {
        boundary: boundary(),
        credential: valid_credential(),
        method: method.to_string(),
        path: path.to_string(),
    }
}

fn versions_request() -> CloudIacModuleVersionsApiRequest {
    CloudIacModuleVersionsApiRequest {
        boundary: boundary(),
        credential: valid_credential(),
        namespace: "oyatie".to_string(),
        name: "vpc".to_string(),
        system: "opentofu".to_string(),
    }
}

fn download_request(version: &str) -> CloudIacModuleDownloadApiRequest {
    CloudIacModuleDownloadApiRequest {
        boundary: boundary(),
        credential: valid_credential(),
        namespace: "oyatie".to_string(),
        name: "vpc".to_string(),
        system: "opentofu".to_string(),
        version: version.to_string(),
    }
}

#[test]
fn discovery_response_is_opentofu_modules_v1_shape_without_runtime_claim() {
    let provider = all_reader_provider();
    let response = discover_module_registry_from_api(&boundary(), &provider, &valid_credential())
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
    let provider = all_reader_provider();
    let response = list_module_versions_from_api(&registry(), &provider, versions_request())
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
    let provider = all_reader_provider();
    let response = get_module_download_from_api(&registry(), &provider, download_request("1.2.0"))
        .expect("download response is authorized");

    assert_eq!(
        response.location,
        "git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/iac-app/tofu/modules/vpc?ref=v1.2.0"
    );
}

// ===========================================================================
// FAIL-CLOSED AUTHZ SEAM (AUTH-005 / ADR-0587) — RED/GREEN tests that MUST fail
// if the verified-principal + PDP gate is removed.
// ===========================================================================

#[test]
fn discovery_rejects_absent_credential_as_unauthenticated() {
    let provider = allow_all_provider(); // PDP would allow — the credential gate must still block.
    let credential = CallerCredential {
        authorization: None,
    };
    let error = discover_module_registry_from_api(&boundary(), &provider, &credential)
        .expect_err("absent credential must be rejected");
    assert_eq!(error, CloudIacModuleRegistryApiError::Unauthenticated);
}

#[test]
fn versions_rejects_forged_bearer_as_unauthenticated() {
    let provider = allow_all_provider();
    let mut request = versions_request();
    request.credential.authorization = Some("Bearer not-the-real-secret".to_string());
    let error = list_module_versions_from_api(&registry(), &provider, request)
        .expect_err("forged bearer must be rejected");
    assert_eq!(error, CloudIacModuleRegistryApiError::Unauthenticated);
}

#[test]
fn download_denies_when_pdp_denies_as_forbidden() {
    let provider = provider_with(Arc::new(DenyAllAuthorizer));
    let error = get_module_download_from_api(&registry(), &provider, download_request("1.2.0"))
        .expect_err("PDP deny must be Forbidden");
    assert_eq!(
        error,
        CloudIacModuleRegistryApiError::Forbidden {
            surface: CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE.to_string(),
        }
    );
}

#[test]
fn download_fails_closed_when_pdp_faults_as_forbidden_not_error() {
    let provider = provider_with(Arc::new(RefuseAuthorizer));
    let error = get_module_download_from_api(&registry(), &provider, download_request("1.2.0"))
        .expect_err("PDP fault must fail closed to Forbidden");
    // Deny and Refuse are reported IDENTICALLY so probing cannot distinguish them.
    assert_eq!(
        error,
        CloudIacModuleRegistryApiError::Forbidden {
            surface: CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE.to_string(),
        }
    );
}

#[test]
fn configured_reader_serves_all_three_surfaces_with_valid_credential() {
    let provider = all_reader_provider();
    discover_module_registry_from_api(&boundary(), &provider, &valid_credential())
        .expect("discovery served");
    list_module_versions_from_api(&registry(), &provider, versions_request())
        .expect("versions served");
    get_module_download_from_api(&registry(), &provider, download_request("1.2.0"))
        .expect("download served");
}

#[test]
fn configured_authorizer_denies_surface_outside_its_permit_set() {
    // Permit ONLY versions; a download request must be Forbidden (deny-by-default).
    let provider = reader_provider(&[CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE]);
    let error = get_module_download_from_api(&registry(), &provider, download_request("1.2.0"))
        .expect_err("download surface is not in the permit set");
    assert_eq!(
        error,
        CloudIacModuleRegistryApiError::Forbidden {
            surface: CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE.to_string(),
        }
    );
}

#[test]
fn constant_time_eq_matches_only_identical_byte_strings() {
    assert!(constant_time_eq(b"break-glass", b"break-glass"));
    // Differing length must NOT match.
    assert!(!constant_time_eq(b"break-glass", b"break-glas"));
    assert!(!constant_time_eq(b"break-glass", b"break-glass-extra"));
    // A single-byte difference at equal length must NOT match.
    assert!(!constant_time_eq(b"break-glass", b"Break-glass"));
    assert!(constant_time_eq(b"", b""));
}

#[test]
fn api_boundary_authorizes_before_runtime_and_rejects_invalid_path_segments() {
    // PDP deny is Forbidden even with a valid credential.
    let denied = list_module_versions_from_api(
        &registry(),
        &provider_with(Arc::new(DenyAllAuthorizer)),
        versions_request(),
    )
    .expect_err("PDP deny is rejected");
    assert_eq!(
        denied,
        CloudIacModuleRegistryApiError::Forbidden {
            surface: CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE.to_string()
        }
    );

    let mut invalid = versions_request();
    invalid.name = "../vpc".to_string();
    let invalid_path = list_module_versions_from_api(&registry(), &all_reader_provider(), invalid)
        .expect_err("invalid path segment is rejected");
    assert_eq!(
        invalid_path,
        CloudIacModuleRegistryApiError::Domain(CloudIacError::InvalidModuleName)
    );
}

#[test]
fn api_boundary_rejects_empty_request_id_and_missing_versions() {
    let provider = all_reader_provider();
    let mut empty_request_id = download_request("1.2.0");
    empty_request_id.boundary = CloudIacModuleRegistryApiBoundaryContext {
        request_id: " ".to_string(),
    };
    let empty_request_id_error =
        get_module_download_from_api(&registry(), &provider, empty_request_id)
            .expect_err("empty request ID is rejected");
    assert_eq!(
        empty_request_id_error,
        CloudIacModuleRegistryApiError::EmptyRequestId
    );

    let missing = get_module_download_from_api(&registry(), &provider, download_request("9.9.9"))
        .expect_err("missing version is rejected");
    assert_eq!(
        missing,
        CloudIacModuleRegistryApiError::Domain(CloudIacError::ModuleVersionNotFound)
    );
}

#[test]
fn debug_output_does_not_contain_secret_like_material() {
    let provider = all_reader_provider();
    let mut request = download_request("1.0.0");
    request.name = "dns".to_string();
    let response = get_module_download_from_api(&registry(), &provider, request)
        .expect("download response is available");

    let debug = format!("{response:?}").to_ascii_lowercase();
    assert!(!debug.contains("token="));
    assert!(!debug.contains("password="));
    assert!(!debug.contains("-----begin"));
    assert!(!debug.contains("kubeconfig"));
    // The served response must not leak the bearer credential.
    assert!(!debug.contains(BEARER_SECRET));
}

#[test]
fn route_boundary_dispatches_official_get_paths_into_dtos() {
    let registry = registry();
    let provider = all_reader_provider();

    let discovery = route_module_registry_request(
        &registry,
        &provider,
        route_request("GET", OPENTOFU_SERVICE_DISCOVERY_PATH),
    )
    .expect("discovery route dispatches");
    assert_eq!(
        discovery,
        CloudIacModuleRegistryRouteResponse::Discovery(
            discover_module_registry_from_api(&boundary(), &provider, &valid_credential())
                .expect("discovery response")
        )
    );

    let versions = route_module_registry_request(
        &registry,
        &provider,
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
        &provider,
        route_request("GET", "/v1/modules/oyatie/vpc/opentofu/1.2.0/download"),
    )
    .expect("download route dispatches");
    assert!(matches!(
        download,
        CloudIacModuleRegistryRouteResponse::Download(response)
            if response.location.ends_with("/microservices/iac-app/tofu/modules/vpc?ref=v1.2.0")
    ));
}

#[test]
fn route_boundary_rejects_wrong_methods_unknown_paths_and_denied_surfaces() {
    let registry = registry();
    let provider = all_reader_provider();

    let wrong_method = route_module_registry_request(
        &registry,
        &provider,
        route_request("POST", OPENTOFU_SERVICE_DISCOVERY_PATH),
    )
    .expect_err("non-GET requests are rejected before runtime");
    assert_eq!(
        wrong_method,
        CloudIacModuleRegistryApiError::MethodNotAllowed {
            method: "POST".to_string()
        }
    );

    let unknown_path = route_module_registry_request(
        &registry,
        &provider,
        route_request("GET", "/v1/modules/oyatie/vpc"),
    )
    .expect_err("incomplete module route is rejected");
    assert_eq!(
        unknown_path,
        CloudIacModuleRegistryApiError::RouteNotFound {
            path: "/v1/modules/oyatie/vpc".to_string()
        }
    );

    // A provider that permits only versions must Forbid a download route.
    let versions_only = reader_provider(&[CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE]);
    let denied_download = route_module_registry_request(
        &registry,
        &versions_only,
        route_request("GET", "/v1/modules/oyatie/vpc/opentofu/1.2.0/download"),
    )
    .expect_err("download surface is required");
    assert_eq!(
        denied_download,
        CloudIacModuleRegistryApiError::Forbidden {
            surface: CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE.to_string()
        }
    );
}

#[test]
fn route_boundary_rejects_whitespace_mutated_method_and_path() {
    let registry = registry();
    let provider = all_reader_provider();

    let padded_method = route_module_registry_request(
        &registry,
        &provider,
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
        &provider,
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
