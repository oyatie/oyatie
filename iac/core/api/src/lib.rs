//! Cloud IaC API boundary for the OpenTofu module registry protocol.
//!
//! This crate owns request/path authorization and response DTO construction for
//! the module-registry surface before any future REST server, router, database,
//! object-store, signer, OpenTofu CLI runner, or provider runtime exists.
//! It intentionally performs no network, filesystem, signing, SLSA, provider,
//! state-backend, plan, apply, or cloud I/O.

#![forbid(unsafe_code)]

use iac_domain::{CloudIacError, ModuleRegistry};

pub const CLOUD_IAC_MODULE_REGISTRY_DISCOVERY_SURFACE: &str = "cloud.iac.module_registry.discovery";
pub const CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE: &str = "cloud.iac.module_registry.versions";
pub const CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE: &str = "cloud.iac.module_registry.download";
pub const OPENTOFU_MODULE_REGISTRY_HTTP_GET: &str = "GET";
pub const OPENTOFU_SERVICE_DISCOVERY_PATH: &str = "/.well-known/terraform.json";
pub const OPENTOFU_MODULES_V1_BASE_PATH: &str = "/v1/modules/";
pub const CLOUD_IAC_MODULE_REGISTRY_API_BOUNDARY_NON_CLAIM: &str =
    "pure-api-boundary-no-rest-server-no-live-registry-runtime";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudIacModuleRegistryApiError {
    EmptyRequestId,
    EmptyPrincipalId,
    EmptyAuthorizationDecisionId,
    MethodNotAllowed { method: String },
    RouteNotFound { path: String },
    ForbiddenSurface { surface: String },
    Domain(CloudIacError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIacModuleRegistryApiBoundaryContext {
    pub request_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIacModuleRegistryApiAuthorization {
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY, AUDIT
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIacModuleVersionsApiRequest {
    pub boundary: CloudIacModuleRegistryApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub authorization: CloudIacModuleRegistryApiAuthorization, // data_class: INTERNAL_ONLY
    pub namespace: String,                                  // data_class: PUBLIC
    pub name: String,                                       // data_class: PUBLIC
    pub system: String,                                     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIacModuleDownloadApiRequest {
    pub boundary: CloudIacModuleRegistryApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub authorization: CloudIacModuleRegistryApiAuthorization, // data_class: INTERNAL_ONLY
    pub namespace: String,                                  // data_class: PUBLIC
    pub name: String,                                       // data_class: PUBLIC
    pub system: String,                                     // data_class: PUBLIC
    pub version: String,                                    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIacModuleRegistryRouteRequest {
    pub boundary: CloudIacModuleRegistryApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub authorization: CloudIacModuleRegistryApiAuthorization, // data_class: INTERNAL_ONLY
    pub method: String,                                     // data_class: PUBLIC
    pub path: String,                                       // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleRegistryDiscoveryResponse {
    pub path: String,       // data_class: PUBLIC
    pub modules_v1: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleVersionEntry {
    pub version: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleVersionsResponseModule {
    pub versions: Vec<ModuleVersionEntry>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleVersionsResponse {
    pub modules: Vec<ModuleVersionsResponseModule>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleDownloadResponse {
    pub location: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudIacModuleRegistryRouteResponse {
    Discovery(ModuleRegistryDiscoveryResponse),
    Versions(ModuleVersionsResponse),
    Download(ModuleDownloadResponse),
}

pub fn discover_module_registry_from_api(
    boundary: &CloudIacModuleRegistryApiBoundaryContext,
    authorization: &CloudIacModuleRegistryApiAuthorization,
) -> Result<ModuleRegistryDiscoveryResponse, CloudIacModuleRegistryApiError> {
    validate_boundary(boundary)?;
    require_surface(authorization, CLOUD_IAC_MODULE_REGISTRY_DISCOVERY_SURFACE)?;
    Ok(ModuleRegistryDiscoveryResponse {
        path: OPENTOFU_SERVICE_DISCOVERY_PATH.to_string(),
        modules_v1: OPENTOFU_MODULES_V1_BASE_PATH.to_string(),
    })
}

pub fn route_module_registry_request(
    registry: &ModuleRegistry,
    request: CloudIacModuleRegistryRouteRequest,
) -> Result<CloudIacModuleRegistryRouteResponse, CloudIacModuleRegistryApiError> {
    if request.method != OPENTOFU_MODULE_REGISTRY_HTTP_GET {
        return Err(CloudIacModuleRegistryApiError::MethodNotAllowed {
            method: request.method,
        });
    }

    if request.path == OPENTOFU_SERVICE_DISCOVERY_PATH {
        return discover_module_registry_from_api(&request.boundary, &request.authorization)
            .map(CloudIacModuleRegistryRouteResponse::Discovery);
    }

    let Some(module_path) = request.path.strip_prefix(OPENTOFU_MODULES_V1_BASE_PATH) else {
        return Err(CloudIacModuleRegistryApiError::RouteNotFound { path: request.path });
    };
    let segments = module_path.split('/').collect::<Vec<_>>();
    if segments.iter().any(|segment| segment.is_empty()) {
        return Err(CloudIacModuleRegistryApiError::RouteNotFound { path: request.path });
    }

    match segments.as_slice() {
        [namespace, name, system, "versions"] => list_module_versions_from_api(
            registry,
            CloudIacModuleVersionsApiRequest {
                boundary: request.boundary,
                authorization: request.authorization,
                namespace: (*namespace).to_string(),
                name: (*name).to_string(),
                system: (*system).to_string(),
            },
        )
        .map(CloudIacModuleRegistryRouteResponse::Versions),
        [namespace, name, system, version, "download"] => get_module_download_from_api(
            registry,
            CloudIacModuleDownloadApiRequest {
                boundary: request.boundary,
                authorization: request.authorization,
                namespace: (*namespace).to_string(),
                name: (*name).to_string(),
                system: (*system).to_string(),
                version: (*version).to_string(),
            },
        )
        .map(CloudIacModuleRegistryRouteResponse::Download),
        _ => Err(CloudIacModuleRegistryApiError::RouteNotFound { path: request.path }),
    }
}

pub fn list_module_versions_from_api(
    registry: &ModuleRegistry,
    request: CloudIacModuleVersionsApiRequest,
) -> Result<ModuleVersionsResponse, CloudIacModuleRegistryApiError> {
    validate_boundary(&request.boundary)?;
    require_surface(
        &request.authorization,
        CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE,
    )?;
    let versions = registry
        .versions(&request.namespace, &request.name, &request.system)?
        .into_iter()
        .map(|release| ModuleVersionEntry {
            version: release.version().to_string(),
        })
        .collect();

    Ok(ModuleVersionsResponse {
        modules: vec![ModuleVersionsResponseModule { versions }],
    })
}

pub fn get_module_download_from_api(
    registry: &ModuleRegistry,
    request: CloudIacModuleDownloadApiRequest,
) -> Result<ModuleDownloadResponse, CloudIacModuleRegistryApiError> {
    validate_boundary(&request.boundary)?;
    require_surface(
        &request.authorization,
        CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE,
    )?;
    let release = registry.resolve(
        &request.namespace,
        &request.name,
        &request.system,
        &request.version,
    )?;
    Ok(ModuleDownloadResponse {
        location: release.source().to_string(),
    })
}

fn validate_boundary(
    boundary: &CloudIacModuleRegistryApiBoundaryContext,
) -> Result<(), CloudIacModuleRegistryApiError> {
    if boundary.request_id.trim().is_empty() {
        Err(CloudIacModuleRegistryApiError::EmptyRequestId)
    } else {
        Ok(())
    }
}

fn require_surface(
    authorization: &CloudIacModuleRegistryApiAuthorization,
    surface: &str,
) -> Result<(), CloudIacModuleRegistryApiError> {
    if authorization.principal_id.trim().is_empty() {
        return Err(CloudIacModuleRegistryApiError::EmptyPrincipalId);
    }
    if authorization.decision_id.trim().is_empty() {
        return Err(CloudIacModuleRegistryApiError::EmptyAuthorizationDecisionId);
    }
    if authorization
        .allowed_surfaces
        .iter()
        .any(|allowed| allowed == surface)
    {
        Ok(())
    } else {
        Err(CloudIacModuleRegistryApiError::ForbiddenSurface {
            surface: surface.to_string(),
        })
    }
}

impl From<CloudIacError> for CloudIacModuleRegistryApiError {
    fn from(value: CloudIacError) -> Self {
        Self::Domain(value)
    }
}
