//! Fixtures shared by the registry REST suites: a module registry, an
//! authorizing provider, and request builders for both the router and the
//! hyper boundary.

// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub use std::collections::BTreeMap;
pub use std::io::{Read, Write};
pub use std::sync::Arc;
pub use std::thread;

pub use http_middleware_kernel::HttpRequest;
pub use http_router_kernel::HttpMethod;
pub use http_runtime_hyper_adapter::serve_one_connection_on_std_listener;
pub use iac_api::{
    CLOUD_IAC_MODULE_REGISTRY_DISCOVERY_SURFACE, CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE,
    CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE, CallerCredential,
    CloudIacModuleRegistryApiBoundaryContext, CloudIacModuleRegistryApiError,
    CloudIacModuleRegistryAuthzProvider, CloudIacModuleRegistryRouteResponse,
    ConfiguredBearerPrincipalVerifier, ConfiguredSurfaceAuthorizer,
    ModuleRegistryAuthorizationError, ModuleRegistryAuthorizer, OPENTOFU_MODULES_V1_BASE_PATH,
    OPENTOFU_SERVICE_DISCOVERY_PATH, VerifiedPrincipal,
};
pub use iac_domain::{ModuleRegistry, OpenTofuModuleRelease};
pub use iac_registry_rest::{
    CLOUD_IAC_MODULE_REGISTRY_HTTP_HANDLER_NON_CLAIM,
    CLOUD_IAC_MODULE_REGISTRY_LOOPBACK_LISTENER_NON_CLAIM,
    CLOUD_IAC_MODULE_REGISTRY_RUNTIME_COMPOSITION_NON_CLAIM,
    CLOUD_IAC_MODULE_REGISTRY_SERVICE_ASSEMBLY_NON_CLAIM, CloudIacModuleRegistryHttpHandler,
    CloudIacModuleRegistryRuntimeError, CloudIacModuleRegistryRuntimeRequest,
    assemble_module_registry_http_service, dispatch_module_registry_http_service_request,
    dispatch_module_registry_runtime_request, handle_module_registry_http_request,
};
pub use iac_rest::{
    CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE as REST_DOWNLOAD_SURFACE,
    CloudIacModuleRegistryRestError, CloudIacModuleRegistryRestRoute,
    MODULE_REGISTRY_DISCOVERY_REST_ROUTE, MODULE_REGISTRY_DOWNLOAD_REST_ROUTE,
    MODULE_REGISTRY_VERSIONS_REST_ROUTE,
};

pub const BEARER_SECRET: &str = "break-glass-iac-registry-secret";
pub const PRINCIPAL_ID: &str = "sp_iac_app_registry_reader";

pub struct AllowAllAuthorizer;
impl ModuleRegistryAuthorizer for AllowAllAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _surface: &str,
    ) -> Result<(), ModuleRegistryAuthorizationError> {
        Ok(())
    }
}

pub struct DenyAllAuthorizer;
impl ModuleRegistryAuthorizer for DenyAllAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _surface: &str,
    ) -> Result<(), ModuleRegistryAuthorizationError> {
        Err(ModuleRegistryAuthorizationError::Denied)
    }
}

pub struct RefuseAuthorizer;
impl ModuleRegistryAuthorizer for RefuseAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _surface: &str,
    ) -> Result<(), ModuleRegistryAuthorizationError> {
        Err(ModuleRegistryAuthorizationError::Refused)
    }
}

pub fn provider_with(
    authorizer: Arc<dyn ModuleRegistryAuthorizer>,
) -> CloudIacModuleRegistryAuthzProvider {
    let verifier = Arc::new(
        ConfiguredBearerPrincipalVerifier::new(BEARER_SECRET, PRINCIPAL_ID)
            .expect("valid break-glass verifier config"),
    );
    CloudIacModuleRegistryAuthzProvider::new(verifier, authorizer)
}

pub fn reader_provider(surfaces: &[&str]) -> CloudIacModuleRegistryAuthzProvider {
    let verifier = Arc::new(
        ConfiguredBearerPrincipalVerifier::new(BEARER_SECRET, PRINCIPAL_ID)
            .expect("valid break-glass verifier config"),
    );
    let authorizer = Arc::new(ConfiguredSurfaceAuthorizer::new(
        surfaces.iter().map(|surface| (*surface).to_string()),
    ));
    CloudIacModuleRegistryAuthzProvider::new(verifier, authorizer)
}

pub fn all_reader_provider() -> CloudIacModuleRegistryAuthzProvider {
    reader_provider(&[
        CLOUD_IAC_MODULE_REGISTRY_DISCOVERY_SURFACE,
        CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE,
        CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE,
    ])
}

pub fn allow_all_handler() -> CloudIacModuleRegistryHttpHandler {
    CloudIacModuleRegistryHttpHandler::new(
        registry(),
        boundary(),
        Arc::new(provider_with(Arc::new(AllowAllAuthorizer))),
    )
}

pub fn valid_credential() -> CallerCredential {
    CallerCredential {
        authorization: Some(format!("Bearer {BEARER_SECRET}")),
    }
}

pub fn release(name: &str, version: &str, digest_hex: char) -> OpenTofuModuleRelease {
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

pub fn registry() -> ModuleRegistry {
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

pub fn boundary() -> CloudIacModuleRegistryApiBoundaryContext {
    CloudIacModuleRegistryApiBoundaryContext {
        request_id: "req_iac_app_registry_runtime_001".to_string(),
    }
}

pub fn runtime_request(
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

pub fn http_request(method: HttpMethod, path: &str) -> HttpRequest {
    HttpRequest {
        method,
        path: path.to_string(),
        headers: BTreeMap::new(),
        body: Vec::new(),
        path_captures: BTreeMap::new(),
        matched_template: None,
    }
}

pub fn http_request_with_bearer(method: HttpMethod, path: &str, bearer: &str) -> HttpRequest {
    let mut request = http_request(method, path);
    request
        .headers
        .insert("authorization".to_string(), format!("Bearer {bearer}"));
    request
}

pub fn body_text(response: &http_middleware_kernel::HttpResponse) -> String {
    String::from_utf8(response.body.clone()).expect("response body is UTF-8 JSON")
}
