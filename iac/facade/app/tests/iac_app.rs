// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` to assert
// invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::sync::Arc;
use std::thread;

use http_middleware_kernel::HttpRequest;
use http_router_kernel::HttpMethod;
use iac_api::{
    CLOUD_IAC_MODULE_REGISTRY_DISCOVERY_SURFACE, CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE,
    CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE, CloudIacModuleRegistryApiBoundaryContext,
    CloudIacModuleRegistryAuthzProvider, ConfiguredBearerPrincipalVerifier,
    ConfiguredSurfaceAuthorizer, OPENTOFU_SERVICE_DISCOVERY_PATH,
};
use iac_app::{
    CLOUD_IAC_APP_ARCHIVE_DIGEST_NON_CLAIM, CLOUD_IAC_APP_ARTIFACT_DOWNLOAD_NON_CLAIM,
    CLOUD_IAC_APP_ARTIFACT_ROUTE_TEMPLATE, CLOUD_IAC_APP_ARTIFACTS_BASE_PATH,
    CLOUD_IAC_APP_BINARY_NAME, CLOUD_IAC_APP_BIND_ADDR_ENV, CLOUD_IAC_APP_DEFAULT_BIND_ADDR,
    CLOUD_IAC_APP_DEFAULT_RELEASE_INDEX_PATH, CLOUD_IAC_APP_ENTRYPOINT_NON_CLAIM,
    CLOUD_IAC_APP_MODULE_REGISTRY_BEARER_ENV, CLOUD_IAC_APP_MODULE_REGISTRY_PRINCIPAL_ENV,
    CLOUD_IAC_APP_OBJECT_PINNING_NON_CLAIM, CLOUD_IAC_APP_OBJECT_SOURCE_NON_CLAIM,
    CLOUD_IAC_APP_PACKAGE_NAME, CLOUD_IAC_APP_RELEASE_INDEX_NON_CLAIM,
    CLOUD_IAC_APP_RELEASE_INDEX_PATH_ENV, CLOUD_IAC_APP_REQUEST_AUTH_NON_CLAIM,
    CLOUD_IAC_HEALTH_PATH, CLOUD_IAC_LIVENESS_PATH, CloudIacAppConfig, CloudIacAppConfigError,
    build_iac_app_service, build_iac_app_service_from_release_index_str, dispatch_iac_app_request,
    load_release_index_seed_from_str, serve_bounded_iac_app_on_listener,
};
use iac_domain::{ModuleRegistry, OpenTofuModuleRelease};

const TEST_BEARER: &str = "local-registry-bearer-fixture";
const TEST_PRINCIPAL: &str = "sp_iac_app_test_reader";

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

fn http_request_with_auth(method: HttpMethod, path: &str, bearer: &str) -> HttpRequest {
    let mut request = http_request(method, path);
    request
        .headers
        .insert("authorization".to_string(), format!("Bearer {bearer}"));
    request
}

fn body_text(response: &http_middleware_kernel::HttpResponse) -> String {
    String::from_utf8(response.body.clone()).expect("response body is UTF-8")
}

/// A fail-closed provider for tests: constant-time bearer verifier bound to a
/// test principal, permitting all three read surfaces (deny-by-default).
fn test_provider() -> Arc<CloudIacModuleRegistryAuthzProvider> {
    let verifier = Arc::new(
        ConfiguredBearerPrincipalVerifier::new(TEST_BEARER, TEST_PRINCIPAL)
            .expect("valid test verifier config"),
    );
    let authorizer = Arc::new(ConfiguredSurfaceAuthorizer::new(
        [
            CLOUD_IAC_MODULE_REGISTRY_DISCOVERY_SURFACE,
            CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE,
            CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE,
        ]
        .iter()
        .map(|surface| (*surface).to_string()),
    ));
    Arc::new(CloudIacModuleRegistryAuthzProvider::new(
        verifier, authorizer,
    ))
}

const RELEASE_INDEX_JSON: &str = include_str!("release-index.json");
const TEST_ARTIFACT_ARCHIVE: &str = "oyatie-unit-artifact-opentofu-0.1.0.zip";
const TEST_ARTIFACT_PATH: &str =
    "target/iac-app/module-archives/oyatie-unit-artifact-opentofu-0.1.0.zip";
const TEST_ARTIFACT_SHA256: &str =
    "c3c49717514288b70d1efe74929f5531a4b3a7610cb2fdf821c6b62f08683014";
const TEST_MISMATCH_ARTIFACT_PATH: &str =
    "target/iac-app/module-archives/oyatie-digest-mismatch-opentofu-0.1.0.zip";
// libtest runs `#[test]`s as parallel threads in one process, and `fs::write` is
// `File::create` (truncate to 0) + `write_all`. Two tests sharing one fixture path
// let one test's request-time sha256 read land inside the other's truncate window,
// digest-mismatching into a 409. Every test that writes archive bytes therefore owns
// its own path. The bytes stay identical, so TEST_ARTIFACT_SHA256 covers all of them.
// The archive must sit directly under the local module-archive root (a subdirectory is
// rejected by `validate_archive_file`), so the path is made unique by file name.
const TEST_BEARER_ARTIFACT_PATH: &str =
    "target/iac-app/module-archives/oyatie-bearer-artifact-opentofu-0.1.0.zip";
const TEST_PDP_DENIED_ARTIFACT_PATH: &str =
    "target/iac-app/module-archives/oyatie-pdp-denied-artifact-opentofu-0.1.0.zip";

fn registry() -> ModuleRegistry {
    let mut registry = ModuleRegistry::default();
    registry
        .publish(
            OpenTofuModuleRelease::new(
                "oyatie",
                "vpc",
                "opentofu",
                "1.0.0",
                "git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/iac-app/tofu/modules/vpc?ref=v1.0.0",
                format!("sha256:{}", "a".repeat(64)),
                "evidence://iac-app/app-entrypoint/vpc/1.0.0",
            )
            .expect("valid module release"),
        )
        .expect("publish vpc release");
    registry
}

fn boundary() -> CloudIacModuleRegistryApiBoundaryContext {
    CloudIacModuleRegistryApiBoundaryContext {
        request_id: "req_iac_app_entrypoint_test_001".to_string(),
    }
}

#[test]
fn app_config_defaults_to_helm_port_and_accepts_env_override() {
    let default_config = CloudIacAppConfig::from_env_pairs(std::iter::empty::<(&str, &str)>())
        .expect("default config parses");
    assert_eq!(
        default_config.bind_addr.to_string(),
        CLOUD_IAC_APP_DEFAULT_BIND_ADDR
    );
    assert_eq!(
        default_config.release_index_path.to_string_lossy(),
        CLOUD_IAC_APP_DEFAULT_RELEASE_INDEX_PATH
    );
    assert_eq!(default_config.module_registry_bearer, None);
    assert_eq!(default_config.module_registry_principal_id, None);

    let override_config = CloudIacAppConfig::from_env_pairs([
        (CLOUD_IAC_APP_BIND_ADDR_ENV, "127.0.0.1:0"),
        (
            CLOUD_IAC_APP_RELEASE_INDEX_PATH_ENV,
            "/tmp/oyatie-iac-app-release-index.json",
        ),
        (CLOUD_IAC_APP_MODULE_REGISTRY_BEARER_ENV, TEST_BEARER),
        (CLOUD_IAC_APP_MODULE_REGISTRY_PRINCIPAL_ENV, TEST_PRINCIPAL),
    ])
    .expect("override config parses");
    assert_eq!(override_config.bind_addr.to_string(), "127.0.0.1:0");
    assert_eq!(
        override_config.release_index_path.to_string_lossy(),
        "/tmp/oyatie-iac-app-release-index.json"
    );
    assert_eq!(
        override_config.module_registry_bearer.as_deref(),
        Some(TEST_BEARER)
    );
    assert_eq!(
        override_config.module_registry_principal_id.as_deref(),
        Some(TEST_PRINCIPAL)
    );
    assert!(override_config.module_registry_authz_provider().is_ok());
}

#[test]
fn serve_refuses_without_bearer_and_principal() {
    // No bearer, no principal → boot-fatal (no default-allow on a supply-chain
    // surface; AUTH-005).
    assert_eq!(
        CloudIacAppConfig::default()
            .module_registry_authz_provider()
            .err(),
        Some(CloudIacAppConfigError::MissingModuleRegistryBearer)
    );

    // Bearer set, principal unset → still boot-fatal.
    let bearer_only = CloudIacAppConfig {
        module_registry_bearer: Some(TEST_BEARER.to_string()),
        ..CloudIacAppConfig::default()
    };
    assert_eq!(
        bearer_only.module_registry_authz_provider().err(),
        Some(CloudIacAppConfigError::MissingModuleRegistryPrincipal)
    );

    // Both set but the bearer carries whitespace/control → rejected.
    let malformed = CloudIacAppConfig {
        module_registry_bearer: Some("line\nbreak".to_string()),
        module_registry_principal_id: Some(TEST_PRINCIPAL.to_string()),
        ..CloudIacAppConfig::default()
    };
    assert!(matches!(
        malformed.module_registry_authz_provider(),
        Err(CloudIacAppConfigError::InvalidModuleRegistryBearer { .. })
    ));

    // Both valid → a provider is assembled.
    let ready = CloudIacAppConfig {
        module_registry_bearer: Some(TEST_BEARER.to_string()),
        module_registry_principal_id: Some(TEST_PRINCIPAL.to_string()),
        ..CloudIacAppConfig::default()
    };
    assert!(ready.module_registry_authz_provider().is_ok());
}

#[test]
fn module_registry_and_artifact_paths_require_verified_bearer_while_health_is_public() {
    fs::create_dir_all("target/iac-app/module-archives")
        .expect("create local artifact fixture directory");
    fs::write(
        TEST_BEARER_ARTIFACT_PATH,
        b"deterministic-local-archive-fixture",
    )
    .expect("write local artifact fixture bytes");

    let release_index = format!(
        r#"{{
          "modules": [
            {{
              "namespace": "oyatie",
              "name": "bearer-artifact",
              "system": "opentofu",
              "version": "0.1.0",
              "source_path": "microservices/iac-app/tofu/modules/vpc",
              "archive_file": "{TEST_BEARER_ARTIFACT_PATH}",
              "archive_sha256": "{TEST_ARTIFACT_SHA256}",
              "archive_media_type": "archive/zip",
              "evidence_ref": "evidence://iac-app/modules/bearer-artifact/0.1.0/local-request-auth"
            }}
          ]
        }}"#
    );
    let service = build_iac_app_service_from_release_index_str(&release_index, test_provider())
        .expect("app service assembles");
    assert_eq!(service.route_count(), 6);
    // The auth is per-handler (PEP), not a middleware layer.
    assert_eq!(service.middleware_count(), 0);

    let health = dispatch_iac_app_request(
        &service,
        http_request(HttpMethod::Get, CLOUD_IAC_HEALTH_PATH),
    );
    assert_eq!(health.status, 200);

    for path in [
        OPENTOFU_SERVICE_DISCOVERY_PATH,
        "/v1/modules/oyatie/bearer-artifact/opentofu/versions",
        "/v1/modules/oyatie/bearer-artifact/opentofu/0.1.0/download",
        "/artifacts/modules/oyatie-bearer-artifact-opentofu-0.1.0.zip",
    ] {
        let missing = dispatch_iac_app_request(&service, http_request(HttpMethod::Get, path));
        assert_eq!(
            missing.status, 401,
            "{path} should require a verified bearer"
        );
        assert_eq!(body_text(&missing), r#"{"error":"unauthorized"}"#);
        assert_eq!(
            missing.headers.get("www-authenticate").map(String::as_str),
            Some("Bearer")
        );

        let wrong = dispatch_iac_app_request(
            &service,
            http_request_with_auth(HttpMethod::Get, path, "wrong-local-bearer"),
        );
        assert_eq!(wrong.status, 401, "{path} should reject a forged bearer");

        let authorized = dispatch_iac_app_request(
            &service,
            http_request_with_auth(HttpMethod::Get, path, TEST_BEARER),
        );
        assert_eq!(
            authorized.status, 200,
            "{path} should accept a verified bearer"
        );
    }

    let artifact = dispatch_iac_app_request(
        &service,
        http_request_with_auth(
            HttpMethod::Get,
            "/artifacts/modules/oyatie-bearer-artifact-opentofu-0.1.0.zip",
            TEST_BEARER,
        ),
    );
    assert_eq!(artifact.body, b"deterministic-local-archive-fixture");
    assert_eq!(
        CLOUD_IAC_APP_REQUEST_AUTH_NON_CLAIM,
        "local-request-bearer-gate-no-production-auth-no-cedar"
    );
}

#[test]
fn artifact_route_denies_when_pdp_denies_download_surface() {
    fs::create_dir_all("target/iac-app/module-archives")
        .expect("create local artifact fixture directory");
    fs::write(
        TEST_PDP_DENIED_ARTIFACT_PATH,
        b"deterministic-local-archive-fixture",
    )
    .expect("write local artifact fixture bytes");

    let release_index = format!(
        r#"{{
          "modules": [
            {{
              "namespace": "oyatie",
              "name": "pdp-denied-artifact",
              "system": "opentofu",
              "version": "0.1.0",
              "source_path": "microservices/iac-app/tofu/modules/vpc",
              "archive_file": "{TEST_PDP_DENIED_ARTIFACT_PATH}",
              "archive_sha256": "{TEST_ARTIFACT_SHA256}",
              "archive_media_type": "archive/zip",
              "evidence_ref": "evidence://iac-app/modules/pdp-denied-artifact/0.1.0/local-deny"
            }}
          ]
        }}"#
    );
    // A provider that permits only discovery+versions (NOT download): a verified
    // caller is still Forbidden from the artifact bytes (deny-by-default).
    let verifier = Arc::new(
        ConfiguredBearerPrincipalVerifier::new(TEST_BEARER, TEST_PRINCIPAL)
            .expect("valid test verifier config"),
    );
    let authorizer = Arc::new(ConfiguredSurfaceAuthorizer::new(
        [
            CLOUD_IAC_MODULE_REGISTRY_DISCOVERY_SURFACE,
            CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE,
        ]
        .iter()
        .map(|surface| (*surface).to_string()),
    ));
    let download_denied_provider = Arc::new(CloudIacModuleRegistryAuthzProvider::new(
        verifier, authorizer,
    ));

    let service =
        build_iac_app_service_from_release_index_str(&release_index, download_denied_provider)
            .expect("app service assembles");

    let denied = dispatch_iac_app_request(
        &service,
        http_request_with_auth(
            HttpMethod::Get,
            "/artifacts/modules/oyatie-pdp-denied-artifact-opentofu-0.1.0.zip",
            TEST_BEARER,
        ),
    );
    assert_eq!(denied.status, 403);
    assert_eq!(body_text(&denied), r#"{"error":"forbidden"}"#);
}

#[test]
fn app_service_registers_health_liveness_and_module_registry_routes() {
    let service = build_iac_app_service(registry(), boundary(), test_provider())
        .expect("app service assembles");

    assert_eq!(service.route_count(), 5);
    assert_eq!(service.server_config().max_body_bytes, 0);

    let health = dispatch_iac_app_request(
        &service,
        http_request(HttpMethod::Get, CLOUD_IAC_HEALTH_PATH),
    );
    assert_eq!(health.status, 200);
    assert_eq!(
        body_text(&health),
        r#"{"status":"ok","service":"iac-app","check":"healthz"}"#
    );

    let live = dispatch_iac_app_request(
        &service,
        http_request(HttpMethod::Get, CLOUD_IAC_LIVENESS_PATH),
    );
    assert_eq!(live.status, 200);
    assert_eq!(
        body_text(&live),
        r#"{"status":"ok","service":"iac-app","check":"livez"}"#
    );

    // Discovery now requires a verified bearer.
    let unauthenticated = dispatch_iac_app_request(
        &service,
        http_request(HttpMethod::Get, OPENTOFU_SERVICE_DISCOVERY_PATH),
    );
    assert_eq!(unauthenticated.status, 401);

    let discovery = dispatch_iac_app_request(
        &service,
        http_request_with_auth(
            HttpMethod::Get,
            OPENTOFU_SERVICE_DISCOVERY_PATH,
            TEST_BEARER,
        ),
    );
    assert_eq!(discovery.status, 200);
    assert_eq!(body_text(&discovery), r#"{"modules.v1":"/v1/modules/"}"#);
}

#[test]
fn bounded_loopback_entrypoint_serves_health_and_discovery_without_deploy_claim() {
    let service = build_iac_app_service(registry(), boundary(), test_provider())
        .expect("app service assembles");
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .expect("bind local loopback listener for deterministic app harness");
    let addr = listener
        .local_addr()
        .expect("loopback listener exposes a local addr");

    let server = thread::spawn(move || serve_bounded_iac_app_on_listener(listener, service, 2));

    // Health is public; discovery requires a verified bearer.
    let requests = [
        (CLOUD_IAC_HEALTH_PATH, None),
        (OPENTOFU_SERVICE_DISCOVERY_PATH, Some(TEST_BEARER)),
    ];
    let responses = requests.map(|(path, bearer)| {
        let mut stream = std::net::TcpStream::connect(addr).expect("connect local app harness");
        let auth_header = match bearer {
            Some(token) => format!("Authorization: Bearer {token}\r\n"),
            None => String::new(),
        };
        stream
            .write_all(
                format!(
                    "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\n{auth_header}Connection: close\r\n\r\n"
                )
                .as_bytes(),
            )
            .expect("write request bytes");
        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .expect("read response bytes");
        response
    });

    server
        .join()
        .expect("bounded app server thread joins")
        .expect("bounded app server serves two connections");
    assert!(responses[0].starts_with("HTTP/1.1 200 OK"));
    assert!(responses[0].contains(r#"{"status":"ok","service":"iac-app","check":"healthz"}"#));
    assert!(responses[1].starts_with("HTTP/1.1 200 OK"));
    assert!(responses[1].contains(r#"{"modules.v1":"/v1/modules/"}"#));
    assert_eq!(CLOUD_IAC_APP_BINARY_NAME, "iac-app");
    assert_eq!(CLOUD_IAC_APP_PACKAGE_NAME, "iac-app");
    assert_eq!(
        CLOUD_IAC_APP_ENTRYPOINT_NON_CLAIM,
        "local-app-entrypoint-health-and-module-registry-no-deploy-no-production-readiness"
    );
}

#[test]
fn release_index_loader_builds_registry_for_gate_validated_local_modules() {
    let seed = load_release_index_seed_from_str(RELEASE_INDEX_JSON)
        .expect("repo-local gate-validated release index parses");
    assert_eq!(seed.modules().len(), 6);

    let service = build_iac_app_service_from_release_index_str(RELEASE_INDEX_JSON, test_provider())
        .expect("release-index-backed app service assembles");
    assert_eq!(service.route_count(), 6);

    let versions = dispatch_iac_app_request(
        &service,
        http_request_with_auth(
            HttpMethod::Get,
            "/v1/modules/oyatie/vpc/opentofu/versions",
            TEST_BEARER,
        ),
    );
    assert_eq!(versions.status, 200);
    assert_eq!(
        body_text(&versions),
        r#"{"modules":[{"versions":[{"version":"0.1.0"}]}]}"#
    );

    let download = dispatch_iac_app_request(
        &service,
        http_request_with_auth(
            HttpMethod::Get,
            "/v1/modules/oyatie/vpc/opentofu/0.1.0/download",
            TEST_BEARER,
        ),
    );
    assert_eq!(download.status, 200);
    let download_body = body_text(&download);
    assert!(download_body.contains("/artifacts/modules/oyatie-vpc-opentofu-0.1.0.zip"));
    assert_eq!(
        CLOUD_IAC_APP_RELEASE_INDEX_NON_CLAIM,
        "local-release-index-loader-no-registry-publish-no-object-store-no-production-readiness"
    );
}

#[test]
fn release_index_backed_app_serves_local_archive_artifact_without_object_store_claim() {
    fs::create_dir_all("target/iac-app/module-archives")
        .expect("create local artifact fixture directory");
    fs::write(TEST_ARTIFACT_PATH, b"deterministic-local-archive-fixture")
        .expect("write local artifact fixture bytes");

    let release_index = format!(
        r#"{{
          "modules": [
            {{
              "namespace": "oyatie",
              "name": "unit-artifact",
              "system": "opentofu",
              "version": "0.1.0",
              "source_path": "microservices/iac-app/tofu/modules/vpc",
              "archive_file": "{TEST_ARTIFACT_PATH}",
              "archive_sha256": "{TEST_ARTIFACT_SHA256}",
              "archive_media_type": "archive/zip",
              "evidence_ref": "evidence://iac-app/modules/unit-artifact/0.1.0/local-foundation"
            }}
          ]
        }}"#
    );

    let service = build_iac_app_service_from_release_index_str(&release_index, test_provider())
        .expect("release-index-backed app with artifact route assembles");
    assert_eq!(service.route_count(), 6);

    let download = dispatch_iac_app_request(
        &service,
        http_request_with_auth(
            HttpMethod::Get,
            "/v1/modules/oyatie/unit-artifact/opentofu/0.1.0/download",
            TEST_BEARER,
        ),
    );
    assert_eq!(download.status, 200);
    assert_eq!(
        body_text(&download),
        r#"{"location":"/artifacts/modules/oyatie-unit-artifact-opentofu-0.1.0.zip"}"#
    );

    let artifact = dispatch_iac_app_request(
        &service,
        http_request_with_auth(
            HttpMethod::Get,
            "/artifacts/modules/oyatie-unit-artifact-opentofu-0.1.0.zip",
            TEST_BEARER,
        ),
    );
    assert_eq!(artifact.status, 200);
    assert_eq!(
        artifact.headers.get("content-type").map(String::as_str),
        Some("archive/zip")
    );
    assert_eq!(artifact.body, b"deterministic-local-archive-fixture");
    assert_eq!(CLOUD_IAC_APP_ARTIFACTS_BASE_PATH, "/artifacts/modules/");
    assert_eq!(
        CLOUD_IAC_APP_ARTIFACT_ROUTE_TEMPLATE,
        "/artifacts/modules/{archive_file}"
    );
    assert_eq!(
        CLOUD_IAC_APP_ARTIFACT_DOWNLOAD_NON_CLAIM,
        "local-filesystem-artifact-serving-no-object-store-no-production-readiness"
    );
}

#[test]
fn release_index_backed_app_returns_s3_or_gcs_object_source_locations_without_live_object_store_claim()
 {
    let s3_source = format!(
        "s3::https://s3.amazonaws.com/oyatie-iac-app-modules/oyatie/unit-artifact/0.1.0/{TEST_ARTIFACT_ARCHIVE}"
    );
    let release_index = format!(
        r#"{{
          "modules": [
            {{
              "namespace": "oyatie",
              "name": "unit-artifact",
              "system": "opentofu",
              "version": "0.1.0",
              "source_path": "microservices/iac-app/tofu/modules/vpc",
              "archive_file": "{TEST_ARTIFACT_PATH}",
              "archive_sha256": "{TEST_ARTIFACT_SHA256}",
              "archive_media_type": "archive/zip",
              "archive_source_location": "{s3_source}",
              "archive_source_integrity_sha256": "{TEST_ARTIFACT_SHA256}",
              "archive_source_version_id": "s3v-local-foundation-0001",
              "evidence_ref": "evidence://iac-app/modules/unit-artifact/0.1.0/object-source"
            }}
          ]
        }}"#
    );

    let service = build_iac_app_service_from_release_index_str(&release_index, test_provider())
        .expect("object-source-backed app service assembles without local object-store runtime");
    assert_eq!(service.route_count(), 5);

    let download = dispatch_iac_app_request(
        &service,
        http_request_with_auth(
            HttpMethod::Get,
            "/v1/modules/oyatie/unit-artifact/opentofu/0.1.0/download",
            TEST_BEARER,
        ),
    );
    assert_eq!(download.status, 200);
    assert_eq!(
        body_text(&download),
        format!(r#"{{"location":"{s3_source}"}}"#)
    );

    let local_artifact_route = dispatch_iac_app_request(
        &service,
        http_request_with_auth(
            HttpMethod::Get,
            "/artifacts/modules/oyatie-unit-artifact-opentofu-0.1.0.zip",
            TEST_BEARER,
        ),
    );
    assert_eq!(local_artifact_route.status, 404);

    let gcs_source = format!(
        "gcs::https://www.googleapis.com/storage/v1/oyatie-iac-app-modules/oyatie/unit-artifact/0.1.0/{TEST_ARTIFACT_ARCHIVE}"
    );
    let gcs_release_index = release_index.replace(&s3_source, &gcs_source).replace(
        r#""archive_source_version_id": "s3v-local-foundation-0001","#,
        r#""archive_source_generation": "1700000000000001","#,
    );
    let gcs_seed = load_release_index_seed_from_str(&gcs_release_index)
        .expect("GCS object source fixture parses");
    assert_eq!(gcs_seed.modules().len(), 1);

    let bad_http_source = release_index.replace(&s3_source, "https://example.com/plain.zip");
    assert!(load_release_index_seed_from_str(&bad_http_source).is_err());

    let bad_secret_source = release_index.replace(
        &s3_source,
        "s3::https://s3.amazonaws.com/oyatie-iac-app-modules/oyatie/unit-artifact/0.1.0/oyatie-unit-artifact-opentofu-0.1.0.zip?token=abc",
    );
    assert!(load_release_index_seed_from_str(&bad_secret_source).is_err());

    let wrong_archive_source = release_index.replace(
        &s3_source,
        "s3::https://s3.amazonaws.com/oyatie-iac-app-modules/oyatie/unit-artifact/0.1.0/oyatie-other-opentofu-0.1.0.zip",
    );
    assert!(load_release_index_seed_from_str(&wrong_archive_source).is_err());

    assert_eq!(
        CLOUD_IAC_APP_OBJECT_SOURCE_NON_CLAIM,
        "opentofu-s3-gcs-source-location-no-live-object-store-no-upload"
    );
}

#[test]
fn object_source_entries_require_provider_specific_pin_metadata() {
    let s3_source = format!(
        "s3::https://s3.amazonaws.com/oyatie-iac-app-modules/oyatie/unit-artifact/0.1.0/{TEST_ARTIFACT_ARCHIVE}"
    );
    let s3_pinned_release_index = format!(
        r#"{{
          "modules": [
            {{
              "namespace": "oyatie",
              "name": "unit-artifact",
              "system": "opentofu",
              "version": "0.1.0",
              "source_path": "microservices/iac-app/tofu/modules/vpc",
              "archive_file": "{TEST_ARTIFACT_PATH}",
              "archive_sha256": "{TEST_ARTIFACT_SHA256}",
              "archive_media_type": "archive/zip",
              "archive_source_location": "{s3_source}",
              "archive_source_integrity_sha256": "{TEST_ARTIFACT_SHA256}",
              "archive_source_version_id": "s3v-local-foundation-0001",
              "evidence_ref": "evidence://iac-app/modules/unit-artifact/0.1.0/object-source"
            }}
          ]
        }}"#
    );
    assert!(load_release_index_seed_from_str(&s3_pinned_release_index).is_ok());

    let s3_release_index_without_pin = s3_pinned_release_index.replace(
        r#"
              "archive_source_version_id": "s3v-local-foundation-0001","#,
        "",
    );
    assert!(load_release_index_seed_from_str(&s3_release_index_without_pin).is_err());

    let s3_release_index_without_integrity = s3_pinned_release_index.replace(
        &format!(
            r#"
              "archive_source_integrity_sha256": "{TEST_ARTIFACT_SHA256}","#
        ),
        "",
    );
    assert!(load_release_index_seed_from_str(&s3_release_index_without_integrity).is_err());

    let s3_release_index_with_mismatched_integrity = s3_pinned_release_index.replace(
        &format!(r#""archive_source_integrity_sha256": "{TEST_ARTIFACT_SHA256}","#),
        r#""archive_source_integrity_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","#,
    );
    assert!(load_release_index_seed_from_str(&s3_release_index_with_mismatched_integrity).is_err());

    let s3_release_index_with_secret_version =
        s3_pinned_release_index.replace("s3v-local-foundation-0001", "token=abc");
    assert!(load_release_index_seed_from_str(&s3_release_index_with_secret_version).is_err());

    let s3_release_index_with_gcs_generation = s3_pinned_release_index.replace(
        r#""archive_source_version_id": "s3v-local-foundation-0001","#,
        r#""archive_source_version_id": "s3v-local-foundation-0001",
              "archive_source_generation": "1700000000000001","#,
    );
    assert!(load_release_index_seed_from_str(&s3_release_index_with_gcs_generation).is_err());

    let gcs_source = format!(
        "gcs::https://www.googleapis.com/storage/v1/oyatie-iac-app-modules/oyatie/unit-artifact/0.1.0/{TEST_ARTIFACT_ARCHIVE}"
    );
    let gcs_pinned_release_index = s3_pinned_release_index
        .replace(&s3_source, &gcs_source)
        .replace(
            r#""archive_source_version_id": "s3v-local-foundation-0001","#,
            r#""archive_source_generation": "1700000000000001","#,
        );
    assert!(load_release_index_seed_from_str(&gcs_pinned_release_index).is_ok());

    let gcs_release_index_without_generation = gcs_pinned_release_index.replace(
        r#"
              "archive_source_generation": "1700000000000001","#,
        "",
    );
    assert!(load_release_index_seed_from_str(&gcs_release_index_without_generation).is_err());

    let gcs_release_index_with_s3_version = gcs_pinned_release_index.replace(
        r#""archive_source_generation": "1700000000000001","#,
        r#""archive_source_generation": "1700000000000001",
              "archive_source_version_id": "s3v-local-foundation-0001","#,
    );
    assert!(load_release_index_seed_from_str(&gcs_release_index_with_s3_version).is_err());

    let gcs_release_index_with_bad_generation = gcs_pinned_release_index.replace(
        r#""archive_source_generation": "1700000000000001","#,
        r#""archive_source_generation": "generation-1700000000000001","#,
    );
    assert!(load_release_index_seed_from_str(&gcs_release_index_with_bad_generation).is_err());

    let local_release_index_with_orphan_pin = s3_pinned_release_index
        .replace(&format!(r#""archive_source_location": "{s3_source}","#), "")
        .replace(
            r#"
              "archive_source_version_id": "s3v-local-foundation-0001","#,
            "",
        );
    assert!(load_release_index_seed_from_str(&local_release_index_with_orphan_pin).is_err());

    assert_eq!(
        CLOUD_IAC_APP_OBJECT_PINNING_NON_CLAIM,
        "object-source-metadata-pin-no-live-object-store-preconditions"
    );
}

#[test]
fn artifact_route_rejects_local_archive_digest_drift_before_serving_bytes() {
    fs::create_dir_all("target/iac-app/module-archives")
        .expect("create local artifact fixture directory");
    fs::write(
        TEST_MISMATCH_ARTIFACT_PATH,
        b"tampered-local-archive-fixture",
    )
    .expect("write mismatched local artifact fixture bytes");

    let release_index = format!(
        r#"{{
          "modules": [
            {{
              "namespace": "oyatie",
              "name": "digest-mismatch",
              "system": "opentofu",
              "version": "0.1.0",
              "source_path": "microservices/iac-app/tofu/modules/vpc",
              "archive_file": "{TEST_MISMATCH_ARTIFACT_PATH}",
              "archive_sha256": "{TEST_ARTIFACT_SHA256}",
              "archive_media_type": "archive/zip",
              "evidence_ref": "evidence://iac-app/modules/digest-mismatch/0.1.0/local-foundation"
            }}
          ]
        }}"#
    );
    let service = build_iac_app_service_from_release_index_str(&release_index, test_provider())
        .expect("release-index-backed app with digest-check route assembles");

    let artifact = dispatch_iac_app_request(
        &service,
        http_request_with_auth(
            HttpMethod::Get,
            "/artifacts/modules/oyatie-digest-mismatch-opentofu-0.1.0.zip",
            TEST_BEARER,
        ),
    );
    assert_eq!(artifact.status, 409);
    assert_eq!(
        body_text(&artifact),
        r#"{"error":"artifact_digest_mismatch"}"#
    );
    assert_eq!(
        CLOUD_IAC_APP_ARCHIVE_DIGEST_NON_CLAIM,
        "local-request-time-sha256-check-no-signing-no-slsa-no-object-store"
    );
}

#[test]
fn artifact_route_rejects_invalid_unknown_and_missing_local_archive_requests() {
    let missing_artifact_path =
        "target/iac-app/module-archives/oyatie-missing-artifact-opentofu-0.1.0.zip";
    let _ = fs::remove_file(missing_artifact_path);
    let release_index = format!(
        r#"{{
          "modules": [
            {{
              "namespace": "oyatie",
              "name": "missing-artifact",
              "system": "opentofu",
              "version": "0.1.0",
              "source_path": "microservices/iac-app/tofu/modules/vpc",
              "archive_file": "{missing_artifact_path}",
              "archive_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
              "archive_media_type": "archive/zip",
              "evidence_ref": "evidence://iac-app/modules/missing-artifact/0.1.0/local-foundation"
            }}
          ]
        }}"#
    );
    let service = build_iac_app_service_from_release_index_str(&release_index, test_provider())
        .expect("missing local archive is a request-time 404, not startup failure");

    let invalid = dispatch_iac_app_request(
        &service,
        http_request_with_auth(HttpMethod::Get, "/artifacts/modules/UPPER.zip", TEST_BEARER),
    );
    assert_eq!(invalid.status, 400);
    assert_eq!(body_text(&invalid), r#"{"error":"invalid_artifact_name"}"#);

    let unknown = dispatch_iac_app_request(
        &service,
        http_request_with_auth(
            HttpMethod::Get,
            "/artifacts/modules/oyatie-other-opentofu-0.1.0.zip",
            TEST_BEARER,
        ),
    );
    assert_eq!(unknown.status, 404);
    assert_eq!(body_text(&unknown), r#"{"error":"artifact_not_found"}"#);

    let missing = dispatch_iac_app_request(
        &service,
        http_request_with_auth(
            HttpMethod::Get,
            "/artifacts/modules/oyatie-missing-artifact-opentofu-0.1.0.zip",
            TEST_BEARER,
        ),
    );
    assert_eq!(missing.status, 404);
    assert_eq!(body_text(&missing), r#"{"error":"artifact_not_found"}"#);
}

#[test]
fn release_index_loader_rejects_empty_modules_bad_archives_and_secret_like_evidence() {
    let empty_modules = r#"{"modules":[]}"#;
    assert!(load_release_index_seed_from_str(empty_modules).is_err());

    let bad_archive = r#"{
      "modules": [
        {
          "namespace": "oyatie",
          "name": "vpc",
          "system": "opentofu",
          "version": "0.1.0",
          "source_path": "microservices/iac-app/tofu/modules/vpc",
          "archive_file": "target/iac-app/module-archives/../secret-0.1.0.zip",
          "archive_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
          "archive_media_type": "archive/zip",
          "evidence_ref": "evidence://iac-app/modules/vpc/0.1.0/local-foundation"
        }
      ]
    }"#;
    assert!(load_release_index_seed_from_str(bad_archive).is_err());

    let secret_like_evidence = r#"{
      "modules": [
        {
          "namespace": "oyatie",
          "name": "vpc",
          "system": "opentofu",
          "version": "0.1.0",
          "source_path": "microservices/iac-app/tofu/modules/vpc",
          "archive_file": "target/iac-app/module-archives/oyatie-vpc-opentofu-0.1.0.zip",
          "archive_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
          "archive_media_type": "archive/zip",
          "evidence_ref": "evidence://iac-app/modules/vpc/token=abc/0.1.0"
        }
      ]
    }"#;
    assert!(load_release_index_seed_from_str(secret_like_evidence).is_err());
}
