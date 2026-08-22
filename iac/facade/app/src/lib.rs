//! Cloud IaC application entrypoint composition.
//!
//! This crate is the app/composition-root layer for the local Cloud IaC
//! OpenTofu module-registry service. It wires health/liveness probes plus the
//! existing module-registry runtime assembly into the canonical Hyper adapter
//! without importing `tokio`, `hyper`, or provider SDKs directly.
//!
//! Current scope is intentionally narrow: a runnable local app process and a
//! bounded loopback harness. It does not implement production authentication,
//! persistence, production object storage, signed releases, OpenTofu plan/apply,
//! provider calls, cloud provisioning, FD-001 tenant workload hosting, or a
//! deployed Kubernetes/Argo CD rollout.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::net::{SocketAddr, TcpListener as StdTcpListener};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use iac_api::{
    CLOUD_IAC_MODULE_REGISTRY_DISCOVERY_SURFACE, CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE,
    CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE, CallerCredential,
    CloudIacModuleRegistryApiBoundaryContext, CloudIacModuleRegistryAuthzProvider,
    ConfiguredBearerPrincipalVerifier, ConfiguredSurfaceAuthorizer,
};
use iac_domain::{CloudIacError, ModuleRegistry, OpenTofuModuleRelease};
use iac_infrastructure::{
    CloudIacModuleRegistryHttpHandler, CloudIacModuleRegistryServiceAssemblyError,
    assemble_module_registry_http_service,
};
use http_middleware_kernel::{HttpRequest, HttpResponse, MiddlewareChain};
use http_router_kernel::{HttpMethod, Router, RouterError};
use http_runtime_hyper_adapter::{
    HyperRuntimeError, ServerConfig, SyncHandler, dispatch as dispatch_hyper_adapter_request,
    serve_n_connections_on_std_listener, serve_on_std_listener,
};
use sha2::{Digest, Sha256};

pub const CLOUD_IAC_APP_ENTRYPOINT_NON_CLAIM: &str =
    "local-app-entrypoint-health-and-module-registry-no-deploy-no-production-readiness";
pub const CLOUD_IAC_APP_RELEASE_INDEX_NON_CLAIM: &str =
    "local-release-index-loader-no-registry-publish-no-object-store-no-production-readiness";
pub const CLOUD_IAC_APP_ARTIFACT_DOWNLOAD_NON_CLAIM: &str =
    "local-filesystem-artifact-serving-no-object-store-no-production-readiness";
pub const CLOUD_IAC_APP_ARCHIVE_DIGEST_NON_CLAIM: &str =
    "local-request-time-sha256-check-no-signing-no-slsa-no-object-store";
pub const CLOUD_IAC_APP_REQUEST_AUTH_NON_CLAIM: &str =
    "local-request-bearer-gate-no-production-auth-no-cedar";
pub const CLOUD_IAC_APP_OBJECT_SOURCE_NON_CLAIM: &str =
    "opentofu-s3-gcs-source-location-no-live-object-store-no-upload";
pub const CLOUD_IAC_APP_OBJECT_PINNING_NON_CLAIM: &str =
    "object-source-metadata-pin-no-live-object-store-preconditions";
pub const CLOUD_IAC_APP_BINARY_NAME: &str = "cloud-iac";
pub const CLOUD_IAC_APP_PACKAGE_NAME: &str = "cloud-iac-app";
pub const CLOUD_IAC_APP_BIND_ADDR_ENV: &str = "OYATIE_CLOUD_IAC_BIND_ADDR";
pub const CLOUD_IAC_APP_DEFAULT_BIND_ADDR: &str = "0.0.0.0:8080";
pub const CLOUD_IAC_APP_RELEASE_INDEX_PATH_ENV: &str = "OYATIE_CLOUD_IAC_RELEASE_INDEX_PATH";
pub const CLOUD_IAC_APP_MODULE_REGISTRY_BEARER_ENV: &str = "OYATIE_CLOUD_IAC_MODULE_REGISTRY_BEARER";
pub const CLOUD_IAC_APP_MODULE_REGISTRY_PRINCIPAL_ENV: &str =
    "OYATIE_CLOUD_IAC_MODULE_REGISTRY_PRINCIPAL";
pub const CLOUD_IAC_APP_DEFAULT_RELEASE_INDEX_PATH: &str =
    "microservices/cloud-iac/tofu/modules/release-index.json";
pub const CLOUD_IAC_APP_ARTIFACTS_BASE_PATH: &str = "/artifacts/modules/";
pub const CLOUD_IAC_APP_ARTIFACT_ROUTE_TEMPLATE: &str = "/artifacts/modules/{archive_file}";
pub const CLOUD_IAC_HEALTH_PATH: &str = "/healthz";
pub const CLOUD_IAC_LIVENESS_PATH: &str = "/livez";

const CLOUD_IAC_APP_RELEASE_SOURCE_ROOT: &str = "microservices/cloud-iac/tofu/modules/";
const CLOUD_IAC_APP_ARCHIVE_FILE_ROOT: &str = "target/cloud-iac/module-archives/";
const CLOUD_IAC_APP_ARCHIVE_MEDIA_TYPE: &str = "archive/zip";

#[derive(Clone, Eq, PartialEq)]
pub struct CloudIacAppConfig {
    pub bind_addr: SocketAddr,                  // data_class: INTERNAL_ONLY
    pub release_index_path: PathBuf,            // data_class: INTERNAL_ONLY
    pub module_registry_bearer: Option<String>, // data_class: SECRET
    pub module_registry_principal_id: Option<String>, // data_class: INTERNAL_ONLY
}

impl std::fmt::Debug for CloudIacAppConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CloudIacAppConfig")
            .field("bind_addr", &self.bind_addr)
            .field("release_index_path", &self.release_index_path)
            .field(
                "module_registry_bearer",
                &self.module_registry_bearer.as_ref().map(|_| "<redacted>"),
            )
            .field(
                "module_registry_principal_id",
                &self.module_registry_principal_id,
            )
            .finish()
    }
}

impl Default for CloudIacAppConfig {
    fn default() -> Self {
        Self {
            bind_addr: CLOUD_IAC_APP_DEFAULT_BIND_ADDR
                .parse()
                .expect("static default bind address parses"),
            release_index_path: PathBuf::from(CLOUD_IAC_APP_DEFAULT_RELEASE_INDEX_PATH),
            module_registry_bearer: None,
            module_registry_principal_id: None,
        }
    }
}

impl CloudIacAppConfig {
    pub fn from_env_pairs<I, K, V>(pairs: I) -> Result<Self, CloudIacAppConfigError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut bind_addr = CLOUD_IAC_APP_DEFAULT_BIND_ADDR.to_string();
        let mut release_index_path = CLOUD_IAC_APP_DEFAULT_RELEASE_INDEX_PATH.to_string();
        let mut module_registry_bearer = None;
        let mut module_registry_principal_id = None;

        for (key, value) in pairs {
            match key.as_ref() {
                CLOUD_IAC_APP_BIND_ADDR_ENV => bind_addr = value.as_ref().to_string(),
                CLOUD_IAC_APP_RELEASE_INDEX_PATH_ENV => {
                    release_index_path = value.as_ref().to_string();
                }
                CLOUD_IAC_APP_MODULE_REGISTRY_BEARER_ENV => {
                    module_registry_bearer = Some(value.as_ref().to_string());
                }
                CLOUD_IAC_APP_MODULE_REGISTRY_PRINCIPAL_ENV => {
                    module_registry_principal_id = Some(value.as_ref().to_string());
                }
                _ => {}
            }
        }

        Ok(Self {
            bind_addr: parse_bind_addr(&bind_addr)?,
            release_index_path: parse_release_index_path(&release_index_path)?,
            module_registry_bearer,
            module_registry_principal_id,
        })
    }

    /// Build the fail-closed module-registry authz provider from config. BOOT-FATAL
    /// when the bearer SECRET or the bound principal id is unset — a process that
    /// cannot prove a credential root and a bound identity must NEVER serve the
    /// supply-chain surface (AUTH-005 / no default-allow).
    ///
    /// # Errors
    /// [`CloudIacAppConfigError`] when the bearer or principal is unset, or the
    /// bearer is malformed.
    pub fn module_registry_authz_provider(
        &self,
    ) -> Result<Arc<CloudIacModuleRegistryAuthzProvider>, CloudIacAppConfigError> {
        let Some(bearer) = &self.module_registry_bearer else {
            return Err(CloudIacAppConfigError::MissingModuleRegistryBearer);
        };
        let Some(principal_id) = &self.module_registry_principal_id else {
            return Err(CloudIacAppConfigError::MissingModuleRegistryPrincipal);
        };
        build_module_registry_authz_provider(bearer, principal_id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudIacAppConfigError {
    EmptyBindAddr,
    InvalidBindAddr { value: String, reason: String },
    EmptyReleaseIndexPath,
    MissingModuleRegistryBearer,
    MissingModuleRegistryPrincipal,
    InvalidModuleRegistryBearer { reason: String },
}

fn parse_bind_addr(value: &str) -> Result<SocketAddr, CloudIacAppConfigError> {
    if value.trim().is_empty() {
        return Err(CloudIacAppConfigError::EmptyBindAddr);
    }
    value.parse().map_err(|error: std::net::AddrParseError| {
        CloudIacAppConfigError::InvalidBindAddr {
            value: value.to_string(),
            reason: error.to_string(),
        }
    })
}

fn parse_release_index_path(value: &str) -> Result<PathBuf, CloudIacAppConfigError> {
    if value.trim().is_empty() {
        return Err(CloudIacAppConfigError::EmptyReleaseIndexPath);
    }
    Ok(PathBuf::from(value))
}

/// The permitted module-registry surfaces for the break-glass reader principal:
/// the three read surfaces. Deny-by-default — anything not listed is refused by
/// the [`ConfiguredSurfaceAuthorizer`].
const CLOUD_IAC_MODULE_REGISTRY_READER_SURFACES: [&str; 3] = [
    CLOUD_IAC_MODULE_REGISTRY_DISCOVERY_SURFACE,
    CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE,
    CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE,
];

/// Assemble the fail-closed module-registry authz provider from a bearer SECRET
/// and a bound principal id: a constant-time bearer [`ConfiguredBearerPrincipalVerifier`]
/// (AUTHN) plus a deny-by-default [`ConfiguredSurfaceAuthorizer`] (the PDP port).
///
/// The bearer must be free of whitespace/control characters so the `Bearer
/// <token>` header round-trips byte-for-byte; the verifier additionally refuses an
/// empty secret/identity at construction.
fn build_module_registry_authz_provider(
    bearer: &str,
    principal_id: &str,
) -> Result<Arc<CloudIacModuleRegistryAuthzProvider>, CloudIacAppConfigError> {
    if bearer.trim().is_empty()
        || bearer.chars().any(|ch| ch.is_ascii_whitespace())
        || bearer.chars().any(|ch| ch.is_control())
    {
        return Err(CloudIacAppConfigError::InvalidModuleRegistryBearer {
            reason: "bearer must not contain whitespace or control characters".to_string(),
        });
    }
    let verifier =
        ConfiguredBearerPrincipalVerifier::new(bearer, principal_id).map_err(|error| {
            CloudIacAppConfigError::InvalidModuleRegistryBearer {
                reason: error.to_string(),
            }
        })?;
    let authorizer = ConfiguredSurfaceAuthorizer::new(
        CLOUD_IAC_MODULE_REGISTRY_READER_SURFACES
            .iter()
            .map(|surface| (*surface).to_string()),
    );
    Ok(Arc::new(CloudIacModuleRegistryAuthzProvider::new(
        Arc::new(verifier),
        Arc::new(authorizer),
    )))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIacReleaseIndexSeed {
    modules: Vec<CloudIacReleaseIndexModuleSeed>, // data_class: INTERNAL_ONLY
}

impl CloudIacReleaseIndexSeed {
    pub fn modules(&self) -> &[CloudIacReleaseIndexModuleSeed] {
        &self.modules
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIacReleaseIndexModuleSeed {
    namespace: String,                               // data_class: INTERNAL_ONLY
    name: String,                                    // data_class: INTERNAL_ONLY
    system: String,                                  // data_class: INTERNAL_ONLY
    version: String,                                 // data_class: INTERNAL_ONLY
    source_path: String,                             // data_class: INTERNAL_ONLY
    archive_file: String,                            // data_class: INTERNAL_ONLY
    archive_sha256: String,                          // data_class: INTERNAL_ONLY
    archive_media_type: String,                      // data_class: INTERNAL_ONLY
    archive_source_location: Option<String>,         // data_class: PUBLIC
    archive_source_integrity_sha256: Option<String>, // data_class: INTERNAL_ONLY
    archive_source_version_id: Option<String>,       // data_class: INTERNAL_ONLY
    archive_source_generation: Option<String>,       // data_class: INTERNAL_ONLY
    evidence_ref: String,                            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudIacAppArchiveArtifact {
    archive_file: PathBuf,  // data_class: INTERNAL_ONLY
    archive_sha256: String, // data_class: INTERNAL_ONLY
    media_type: String,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudIacReleaseIndexError {
    EmptyDocument,
    MissingField { field: &'static str },
    MalformedJson { reason: String },
    EmptyModules,
    InvalidField { field: &'static str, reason: String },
    Io { path: String, reason: String },
    Domain(CloudIacError),
}

pub fn load_release_index_seed_from_path(
    path: impl AsRef<Path>,
) -> Result<CloudIacReleaseIndexSeed, CloudIacReleaseIndexError> {
    let path_ref = path.as_ref();
    let body = fs::read_to_string(path_ref).map_err(|error| CloudIacReleaseIndexError::Io {
        path: path_ref.display().to_string(),
        reason: error.to_string(),
    })?;
    load_release_index_seed_from_str(&body)
}

pub fn load_release_index_seed_from_str(
    input: &str,
) -> Result<CloudIacReleaseIndexSeed, CloudIacReleaseIndexError> {
    if input.trim().is_empty() {
        return Err(CloudIacReleaseIndexError::EmptyDocument);
    }
    let module_array = array_field_contents(input, "modules")?;
    let objects = top_level_object_slices(module_array)?;
    if objects.is_empty() {
        return Err(CloudIacReleaseIndexError::EmptyModules);
    }

    let mut modules = Vec::with_capacity(objects.len());
    for object in objects {
        let module = CloudIacReleaseIndexModuleSeed {
            namespace: required_string_field(object, "namespace")?,
            name: required_string_field(object, "name")?,
            system: required_string_field(object, "system")?,
            version: required_string_field(object, "version")?,
            source_path: required_string_field(object, "source_path")?,
            archive_file: required_string_field(object, "archive_file")?,
            archive_sha256: required_string_field(object, "archive_sha256")?,
            archive_media_type: required_string_field(object, "archive_media_type")?,
            archive_source_location: optional_string_field(object, "archive_source_location")?,
            archive_source_integrity_sha256: optional_string_field(
                object,
                "archive_source_integrity_sha256",
            )?,
            archive_source_version_id: optional_string_field(object, "archive_source_version_id")?,
            archive_source_generation: optional_string_field(object, "archive_source_generation")?,
            evidence_ref: required_string_field(object, "evidence_ref")?,
        };
        validate_release_index_module_seed(&module)?;
        modules.push(module);
    }

    Ok(CloudIacReleaseIndexSeed { modules })
}

pub fn build_module_registry_from_release_index_seed(
    seed: &CloudIacReleaseIndexSeed,
) -> Result<ModuleRegistry, CloudIacReleaseIndexError> {
    let mut registry = ModuleRegistry::default();
    for module in seed.modules() {
        let source = release_source_for_module(module)?;
        let digest = format!("sha256:{}", module.archive_sha256);
        let release = OpenTofuModuleRelease::new(
            module.namespace.clone(),
            module.name.clone(),
            module.system.clone(),
            module.version.clone(),
            source,
            digest,
            module.evidence_ref.clone(),
        )
        .map_err(CloudIacReleaseIndexError::Domain)?;
        registry
            .publish(release)
            .map_err(CloudIacReleaseIndexError::Domain)?;
    }
    Ok(registry)
}

fn validate_release_index_module_seed(
    module: &CloudIacReleaseIndexModuleSeed,
) -> Result<(), CloudIacReleaseIndexError> {
    validate_non_empty("namespace", &module.namespace)?;
    validate_non_empty("name", &module.name)?;
    validate_non_empty("system", &module.system)?;
    validate_non_empty("version", &module.version)?;
    validate_release_source_path(&module.source_path)?;
    validate_archive_file(&module.archive_file, &module.version)?;
    validate_archive_sha256(&module.archive_sha256)?;
    validate_archive_media_type(&module.archive_media_type)?;
    validate_archive_source_location(module)?;
    validate_archive_source_pin(module)?;
    validate_evidence_ref_seed(&module.evidence_ref)?;
    Ok(())
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), CloudIacReleaseIndexError> {
    if value.trim().is_empty() {
        Err(CloudIacReleaseIndexError::InvalidField {
            field,
            reason: "value must be non-empty".to_string(),
        })
    } else {
        Ok(())
    }
}

fn validate_release_source_path(path: &str) -> Result<(), CloudIacReleaseIndexError> {
    validate_non_empty("source_path", path)?;
    if contains_secret_like_marker(path) {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "source_path",
            reason: "path contains a credential-like marker".to_string(),
        });
    }
    if !path.starts_with(CLOUD_IAC_APP_RELEASE_SOURCE_ROOT) {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "source_path",
            reason: format!("path must start with {CLOUD_IAC_APP_RELEASE_SOURCE_ROOT}"),
        });
    }
    if path.starts_with('/')
        || path.contains('\\')
        || path.contains('?')
        || path.contains('#')
        || path.contains("//")
    {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "source_path",
            reason:
                "path must be repo-relative without query, fragment, backslash, or empty segment"
                    .to_string(),
        });
    }
    for segment in path.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(CloudIacReleaseIndexError::InvalidField {
                field: "source_path",
                reason: "path contains an empty, current-directory, or parent-directory segment"
                    .to_string(),
            });
        }
        if !segment
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
        {
            return Err(CloudIacReleaseIndexError::InvalidField {
                field: "source_path",
                reason: "path segment contains unsupported characters".to_string(),
            });
        }
    }
    Ok(())
}

fn validate_archive_sha256(value: &str) -> Result<(), CloudIacReleaseIndexError> {
    if value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_sha256",
            reason: "archive digest must be exactly 64 hexadecimal characters without prefix"
                .to_string(),
        })
    }
}

fn validate_archive_file(path: &str, version: &str) -> Result<(), CloudIacReleaseIndexError> {
    validate_non_empty("archive_file", path)?;
    if contains_secret_like_marker(path) {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_file",
            reason: "path contains a credential-like marker".to_string(),
        });
    }
    if !path.starts_with(CLOUD_IAC_APP_ARCHIVE_FILE_ROOT) {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_file",
            reason: format!("path must start with {CLOUD_IAC_APP_ARCHIVE_FILE_ROOT}"),
        });
    }
    if path.contains('\\') || path.contains('?') || path.contains('#') || path.contains("//") {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_file",
            reason:
                "path must be repo-relative without query, fragment, backslash, or empty segment"
                    .to_string(),
        });
    }
    let file_name = archive_file_name(path)?;
    let Some(relative_file_name) = path.strip_prefix(CLOUD_IAC_APP_ARCHIVE_FILE_ROOT) else {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_file",
            reason: format!("path must start with {CLOUD_IAC_APP_ARCHIVE_FILE_ROOT}"),
        });
    };
    if relative_file_name != file_name {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_file",
            reason: "archive file must live directly under the local module archive root"
                .to_string(),
        });
    }
    if !is_valid_archive_file_name(&file_name, version) {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_file",
            reason:
                "archive filename must be a safe lowercase .zip name pinned to the module version"
                    .to_string(),
        });
    }
    for segment in path.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(CloudIacReleaseIndexError::InvalidField {
                field: "archive_file",
                reason: "path contains an empty, current-directory, or parent-directory segment"
                    .to_string(),
            });
        }
    }
    Ok(())
}

fn validate_archive_media_type(value: &str) -> Result<(), CloudIacReleaseIndexError> {
    if value == CLOUD_IAC_APP_ARCHIVE_MEDIA_TYPE {
        Ok(())
    } else {
        Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_media_type",
            reason: format!("archive media type must be {CLOUD_IAC_APP_ARCHIVE_MEDIA_TYPE}"),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ArchiveSourceProvider {
    S3,
    Gcs,
}

fn validate_archive_source_location(
    module: &CloudIacReleaseIndexModuleSeed,
) -> Result<(), CloudIacReleaseIndexError> {
    let Some(location) = &module.archive_source_location else {
        return Ok(());
    };
    validate_non_empty("archive_source_location", location)?;
    if contains_secret_like_marker(location) {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_location",
            reason: "object source contains a credential-like marker".to_string(),
        });
    }
    if location
        .chars()
        .any(|ch| ch.is_ascii_whitespace() || ch.is_control())
    {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_location",
            reason: "object source must not contain whitespace or control characters".to_string(),
        });
    }
    if archive_source_provider(location).is_none() {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_location",
            reason: "object source must use s3::https:// or gcs::https://".to_string(),
        });
    }
    if location.contains('@') || location.contains('?') || location.contains('#') {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_location",
            reason: "object source must not embed userinfo, query strings, or fragments"
                .to_string(),
        });
    }
    let archive_name = archive_file_name(&module.archive_file)?;
    if !location.ends_with(&archive_name) {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_location",
            reason: "object source must end with the configured archive filename".to_string(),
        });
    }
    Ok(())
}

fn validate_archive_source_pin(
    module: &CloudIacReleaseIndexModuleSeed,
) -> Result<(), CloudIacReleaseIndexError> {
    let Some(location) = &module.archive_source_location else {
        return validate_no_orphan_archive_source_pin(module);
    };
    let provider = archive_source_provider(location).ok_or_else(|| {
        CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_location",
            reason: "object source must use s3::https:// or gcs::https://".to_string(),
        }
    })?;

    let Some(source_integrity) = module.archive_source_integrity_sha256.as_deref() else {
        return Err(CloudIacReleaseIndexError::MissingField {
            field: "archive_source_integrity_sha256",
        });
    };
    validate_archive_source_integrity_sha256(source_integrity, &module.archive_sha256)?;

    match provider {
        ArchiveSourceProvider::S3 => {
            let Some(version_id) = module.archive_source_version_id.as_deref() else {
                return Err(CloudIacReleaseIndexError::MissingField {
                    field: "archive_source_version_id",
                });
            };
            validate_archive_source_version_id(version_id)?;
            if module.archive_source_generation.is_some() {
                return Err(CloudIacReleaseIndexError::InvalidField {
                    field: "archive_source_generation",
                    reason: "GCS generation metadata must not be set for S3 object sources"
                        .to_string(),
                });
            }
        }
        ArchiveSourceProvider::Gcs => {
            let Some(generation) = module.archive_source_generation.as_deref() else {
                return Err(CloudIacReleaseIndexError::MissingField {
                    field: "archive_source_generation",
                });
            };
            validate_archive_source_generation(generation)?;
            if module.archive_source_version_id.is_some() {
                return Err(CloudIacReleaseIndexError::InvalidField {
                    field: "archive_source_version_id",
                    reason: "S3 version-id metadata must not be set for GCS object sources"
                        .to_string(),
                });
            }
        }
    }
    Ok(())
}

fn validate_no_orphan_archive_source_pin(
    module: &CloudIacReleaseIndexModuleSeed,
) -> Result<(), CloudIacReleaseIndexError> {
    if module.archive_source_integrity_sha256.is_some() {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_integrity_sha256",
            reason: "object-source integrity metadata requires archive_source_location".to_string(),
        });
    }
    if module.archive_source_version_id.is_some() {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_version_id",
            reason: "S3 version-id metadata requires archive_source_location".to_string(),
        });
    }
    if module.archive_source_generation.is_some() {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_generation",
            reason: "GCS generation metadata requires archive_source_location".to_string(),
        });
    }
    Ok(())
}

fn validate_archive_source_integrity_sha256(
    source_integrity: &str,
    archive_sha256: &str,
) -> Result<(), CloudIacReleaseIndexError> {
    validate_non_empty("archive_source_integrity_sha256", source_integrity)?;
    if !is_lowercase_sha256(source_integrity) {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_integrity_sha256",
            reason: "object-source integrity must be exactly 64 lowercase hexadecimal characters"
                .to_string(),
        });
    }
    if source_integrity != archive_sha256 {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_integrity_sha256",
            reason: "object-source integrity must match archive_sha256".to_string(),
        });
    }
    Ok(())
}

fn validate_archive_source_version_id(value: &str) -> Result<(), CloudIacReleaseIndexError> {
    validate_pin_token("archive_source_version_id", value, "S3 version-id metadata")
}

fn validate_archive_source_generation(value: &str) -> Result<(), CloudIacReleaseIndexError> {
    validate_non_empty("archive_source_generation", value)?;
    if !value.chars().all(|ch| ch.is_ascii_digit()) || value.chars().all(|ch| ch == '0') {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_generation",
            reason: "GCS generation metadata must be a non-zero ASCII decimal string".to_string(),
        });
    }
    Ok(())
}

fn validate_pin_token(
    field: &'static str,
    value: &str,
    label: &'static str,
) -> Result<(), CloudIacReleaseIndexError> {
    validate_non_empty(field, value)?;
    if contains_secret_like_marker(value) {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field,
            reason: format!("{label} contains a credential-like marker"),
        });
    }
    if value
        .chars()
        .any(|ch| ch.is_ascii_whitespace() || ch.is_control())
    {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field,
            reason: format!("{label} must not contain whitespace or control characters"),
        });
    }
    if value.contains('\\')
        || value.contains('"')
        || value.contains('@')
        || value.contains('?')
        || value.contains('#')
    {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field,
            reason: format!("{label} must not contain URL/userinfo/control delimiters"),
        });
    }
    Ok(())
}

fn archive_source_provider(location: &str) -> Option<ArchiveSourceProvider> {
    if location.starts_with("s3::https://") {
        Some(ArchiveSourceProvider::S3)
    } else if location.starts_with("gcs::https://") {
        Some(ArchiveSourceProvider::Gcs)
    } else {
        None
    }
}

fn is_lowercase_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .chars()
            .all(|ch| ch.is_ascii_digit() || matches!(ch, 'a'..='f'))
}

fn validate_evidence_ref_seed(value: &str) -> Result<(), CloudIacReleaseIndexError> {
    validate_non_empty("evidence_ref", value)?;
    if !value.starts_with("evidence://") {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "evidence_ref",
            reason: "evidence ref must use evidence:// scheme".to_string(),
        });
    }
    if contains_secret_like_marker(value) {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "evidence_ref",
            reason: "evidence ref contains a credential-like marker".to_string(),
        });
    }
    Ok(())
}

fn release_source_for_module(
    module: &CloudIacReleaseIndexModuleSeed,
) -> Result<String, CloudIacReleaseIndexError> {
    if let Some(location) = &module.archive_source_location {
        return Ok(location.clone());
    }
    Ok(format!(
        "{CLOUD_IAC_APP_ARTIFACTS_BASE_PATH}{}",
        archive_file_name(&module.archive_file)?
    ))
}

fn archive_file_name(path: &str) -> Result<String, CloudIacReleaseIndexError> {
    Path::new(path)
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .map(str::to_string)
        .ok_or_else(|| CloudIacReleaseIndexError::InvalidField {
            field: "archive_file",
            reason: "archive path must include a UTF-8 filename".to_string(),
        })
}

fn is_valid_archive_file_name(file_name: &str, version: &str) -> bool {
    if file_name.is_empty()
        || file_name == "."
        || file_name == ".."
        || !file_name.ends_with(".zip")
        || !file_name.ends_with(&format!("-{version}.zip"))
    {
        return false;
    }
    file_name
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '-' | '.'))
}

fn archive_artifacts_from_seed(
    seed: &CloudIacReleaseIndexSeed,
) -> Result<BTreeMap<String, CloudIacAppArchiveArtifact>, CloudIacReleaseIndexError> {
    let mut artifacts = BTreeMap::new();
    for module in seed.modules() {
        if module.archive_source_location.is_some() {
            continue;
        }
        let file_name = archive_file_name(&module.archive_file)?;
        artifacts.insert(
            file_name,
            CloudIacAppArchiveArtifact {
                archive_file: PathBuf::from(&module.archive_file),
                archive_sha256: module.archive_sha256.clone(),
                media_type: module.archive_media_type.clone(),
            },
        );
    }
    Ok(artifacts)
}

fn contains_secret_like_marker(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("token=")
        || lower.contains("password=")
        || lower.contains("secret=")
        || lower.contains("kubeconfig")
        || lower.contains("-----begin")
        || lower.contains("sk-live")
        || lower.contains("sk-")
}

fn array_field_contents<'a>(
    input: &'a str,
    field: &'static str,
) -> Result<&'a str, CloudIacReleaseIndexError> {
    let field_position = find_field_position(input, field)
        .ok_or(CloudIacReleaseIndexError::MissingField { field })?;
    let token = quoted_field_token(field);
    let after_field = field_position + token.len();
    let colon_position = find_next_non_string_char(input, after_field, ':').ok_or_else(|| {
        CloudIacReleaseIndexError::MalformedJson {
            reason: format!("field {field} is missing ':' separator"),
        }
    })?;
    let array_start = first_non_whitespace_byte(input, colon_position + 1).ok_or_else(|| {
        CloudIacReleaseIndexError::MalformedJson {
            reason: format!("field {field} is missing an array value"),
        }
    })?;
    if input.as_bytes().get(array_start) != Some(&b'[') {
        return Err(CloudIacReleaseIndexError::MalformedJson {
            reason: format!("field {field} must be an array"),
        });
    }
    let array_end = matching_delimiter(input, array_start, '[', ']').ok_or_else(|| {
        CloudIacReleaseIndexError::MalformedJson {
            reason: format!("field {field} array is not closed"),
        }
    })?;
    Ok(&input[array_start + 1..array_end])
}

fn top_level_object_slices(input: &str) -> Result<Vec<&str>, CloudIacReleaseIndexError> {
    let mut objects = Vec::new();
    let mut depth = 0_usize;
    let mut object_start = None;
    let mut in_string = false;
    let mut escaped = false;

    for (index, ch) in input.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => {
                if depth == 0 {
                    object_start = Some(index);
                }
                depth = depth.saturating_add(1);
            }
            '}' => {
                if depth == 0 {
                    return Err(CloudIacReleaseIndexError::MalformedJson {
                        reason: "module array contains an unmatched object close".to_string(),
                    });
                }
                depth -= 1;
                if depth == 0 {
                    let start =
                        object_start.ok_or_else(|| CloudIacReleaseIndexError::MalformedJson {
                            reason: "module object close has no start".to_string(),
                        })?;
                    objects.push(&input[start..index + ch.len_utf8()]);
                    object_start = None;
                }
            }
            ',' | ' ' | '\n' | '\r' | '\t' if depth == 0 => {}
            _ if depth == 0 => {
                return Err(CloudIacReleaseIndexError::MalformedJson {
                    reason: "modules array must contain JSON objects only".to_string(),
                });
            }
            _ => {}
        }
    }

    if in_string || depth != 0 {
        return Err(CloudIacReleaseIndexError::MalformedJson {
            reason: "module array has an unterminated string or object".to_string(),
        });
    }
    Ok(objects)
}

fn required_string_field(
    object: &str,
    field: &'static str,
) -> Result<String, CloudIacReleaseIndexError> {
    let field_position = find_field_position(object, field)
        .ok_or(CloudIacReleaseIndexError::MissingField { field })?;
    let token = quoted_field_token(field);
    let after_field = field_position + token.len();
    let colon_position = find_next_non_string_char(object, after_field, ':').ok_or_else(|| {
        CloudIacReleaseIndexError::MalformedJson {
            reason: format!("field {field} is missing ':' separator"),
        }
    })?;
    let value_start = first_non_whitespace_byte(object, colon_position + 1).ok_or_else(|| {
        CloudIacReleaseIndexError::MalformedJson {
            reason: format!("field {field} is missing a value"),
        }
    })?;
    if object.as_bytes().get(value_start) != Some(&b'"') {
        return Err(CloudIacReleaseIndexError::MalformedJson {
            reason: format!("field {field} must be a JSON string"),
        });
    }
    parse_json_string(object, value_start)
}

fn optional_string_field(
    object: &str,
    field: &'static str,
) -> Result<Option<String>, CloudIacReleaseIndexError> {
    if find_field_position(object, field).is_none() {
        return Ok(None);
    }
    required_string_field(object, field).map(Some)
}

fn quoted_field_token(field: &str) -> String {
    format!("\"{field}\"")
}

fn find_field_position(input: &str, field: &str) -> Option<usize> {
    input.find(&quoted_field_token(field))
}

fn first_non_whitespace_byte(input: &str, start: usize) -> Option<usize> {
    input[start..]
        .char_indices()
        .find_map(|(offset, ch)| (!ch.is_whitespace()).then_some(start + offset))
}

fn find_next_non_string_char(input: &str, start: usize, target: char) -> Option<usize> {
    let mut in_string = false;
    let mut escaped = false;
    for (offset, ch) in input[start..].char_indices() {
        let index = start + offset;
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            ch if ch == target => return Some(index),
            _ => {}
        }
    }
    None
}

fn matching_delimiter(input: &str, open_at: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0_usize;
    let mut in_string = false;
    let mut escaped = false;

    for (offset, ch) in input[open_at..].char_indices() {
        let index = open_at + offset;
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
        } else if ch == open {
            depth = depth.saturating_add(1);
        } else if ch == close {
            if depth == 0 {
                return None;
            }
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

fn parse_json_string(input: &str, quote_at: usize) -> Result<String, CloudIacReleaseIndexError> {
    let mut output = String::new();
    let mut escaped = false;
    for (offset, ch) in input[quote_at + 1..].char_indices() {
        let index = quote_at + 1 + offset;
        if escaped {
            match ch {
                '"' | '\\' | '/' => output.push(ch),
                'b' => output.push('\u{0008}'),
                'f' => output.push('\u{000c}'),
                'n' => output.push('\n'),
                'r' => output.push('\r'),
                't' => output.push('\t'),
                'u' => {
                    return Err(CloudIacReleaseIndexError::MalformedJson {
                        reason:
                            "unicode escapes are outside the local release-index loader contract"
                                .to_string(),
                    });
                }
                _ => {
                    return Err(CloudIacReleaseIndexError::MalformedJson {
                        reason: "invalid JSON string escape".to_string(),
                    });
                }
            }
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '"' => return Ok(output),
            ch if ch.is_control() => {
                return Err(CloudIacReleaseIndexError::MalformedJson {
                    reason: "JSON string contains an unescaped control character".to_string(),
                });
            }
            _ => output.push(ch),
        }

        if index >= input.len() {
            break;
        }
    }
    Err(CloudIacReleaseIndexError::MalformedJson {
        reason: "JSON string is not closed".to_string(),
    })
}

pub struct CloudIacAppService {
    router: Router<SyncHandler>, // data_class: INTERNAL_ONLY
    middleware: MiddlewareChain<HttpRequest, HttpResponse>, // data_class: INTERNAL_ONLY
    server_config: ServerConfig, // data_class: INTERNAL_ONLY
}

impl CloudIacAppService {
    pub fn route_count(&self) -> usize {
        self.router.count()
    }

    pub fn middleware_count(&self) -> usize {
        self.middleware.count()
    }

    pub fn server_config(&self) -> &ServerConfig {
        &self.server_config
    }

    pub fn into_serve_parts(
        self,
    ) -> (
        Router<SyncHandler>,
        MiddlewareChain<HttpRequest, HttpResponse>,
        ServerConfig,
    ) {
        (self.router, self.middleware, self.server_config)
    }
}

#[derive(Debug)]
pub enum CloudIacAppError {
    Config(CloudIacAppConfigError),
    Bind(String),
    ReleaseIndex(CloudIacReleaseIndexError),
    RegistryService(CloudIacModuleRegistryServiceAssemblyError),
    Router(RouterError),
    Hyper(HyperRuntimeError),
}

impl std::fmt::Display for CloudIacAppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(error) => write!(f, "cloud-iac app config failed: {error:?}"),
            Self::Bind(reason) => write!(f, "cloud-iac app bind failed: {reason}"),
            Self::ReleaseIndex(error) => {
                write!(f, "cloud-iac release-index load failed: {error:?}")
            }
            Self::RegistryService(error) => {
                write!(f, "cloud-iac module registry assembly failed: {error:?}")
            }
            Self::Router(error) => write!(f, "cloud-iac app route registration failed: {error:?}"),
            Self::Hyper(error) => write!(f, "cloud-iac hyper runtime failed: {error}"),
        }
    }
}

impl std::error::Error for CloudIacAppError {}

impl From<CloudIacAppConfigError> for CloudIacAppError {
    fn from(value: CloudIacAppConfigError) -> Self {
        Self::Config(value)
    }
}

impl From<CloudIacReleaseIndexError> for CloudIacAppError {
    fn from(value: CloudIacReleaseIndexError) -> Self {
        Self::ReleaseIndex(value)
    }
}

impl From<CloudIacModuleRegistryServiceAssemblyError> for CloudIacAppError {
    fn from(value: CloudIacModuleRegistryServiceAssemblyError) -> Self {
        Self::RegistryService(value)
    }
}

impl From<RouterError> for CloudIacAppError {
    fn from(value: RouterError) -> Self {
        Self::Router(value)
    }
}

impl From<HyperRuntimeError> for CloudIacAppError {
    fn from(value: HyperRuntimeError) -> Self {
        Self::Hyper(value)
    }
}

pub fn build_cloud_iac_app_service_from_release_index_path(
    path: impl AsRef<Path>,
    authz_provider: Arc<CloudIacModuleRegistryAuthzProvider>,
) -> Result<CloudIacAppService, CloudIacAppError> {
    let seed = load_release_index_seed_from_path(path)?;
    build_cloud_iac_app_service_from_release_index_seed(&seed, authz_provider)
}

pub fn build_cloud_iac_app_service_from_release_index_str(
    input: &str,
    authz_provider: Arc<CloudIacModuleRegistryAuthzProvider>,
) -> Result<CloudIacAppService, CloudIacAppError> {
    let seed = load_release_index_seed_from_str(input)?;
    build_cloud_iac_app_service_from_release_index_seed(&seed, authz_provider)
}

pub fn build_cloud_iac_app_service_from_release_index_seed(
    seed: &CloudIacReleaseIndexSeed,
    authz_provider: Arc<CloudIacModuleRegistryAuthzProvider>,
) -> Result<CloudIacAppService, CloudIacAppError> {
    let registry = build_module_registry_from_release_index_seed(seed)?;
    let artifacts = archive_artifacts_from_seed(seed)?;
    build_cloud_iac_app_service_with_artifacts(
        registry,
        cloud_iac_app_bootstrap_boundary(),
        authz_provider,
        artifacts,
    )
}

fn cloud_iac_app_bootstrap_boundary() -> CloudIacModuleRegistryApiBoundaryContext {
    CloudIacModuleRegistryApiBoundaryContext {
        request_id: "req_cloud_iac_app_bootstrap_local_001".to_string(),
    }
}

pub fn build_cloud_iac_app_service(
    registry: ModuleRegistry,
    boundary: CloudIacModuleRegistryApiBoundaryContext,
    authz_provider: Arc<CloudIacModuleRegistryAuthzProvider>,
) -> Result<CloudIacAppService, CloudIacAppError> {
    build_cloud_iac_app_service_with_artifacts(registry, boundary, authz_provider, BTreeMap::new())
}

fn build_cloud_iac_app_service_with_artifacts(
    registry: ModuleRegistry,
    boundary: CloudIacModuleRegistryApiBoundaryContext,
    authz_provider: Arc<CloudIacModuleRegistryAuthzProvider>,
    artifacts: BTreeMap<String, CloudIacAppArchiveArtifact>,
) -> Result<CloudIacAppService, CloudIacAppError> {
    // The infra handler holds the provider and does the module-registry PEP
    // (discovery/versions/download). The artifact route is the SECOND PEP and
    // shares the SAME provider — both gate on a verified principal + PDP decision.
    let registry_service = assemble_module_registry_http_service(
        CloudIacModuleRegistryHttpHandler::new(registry, boundary, authz_provider.clone()),
    )?;
    let (mut router, middleware, server_config) = registry_service.into_serve_parts();
    register_artifact_routes(&mut router, artifacts, authz_provider)?;
    register_health_routes(&mut router)?;
    Ok(CloudIacAppService {
        router,
        middleware,
        server_config,
    })
}

fn register_artifact_routes(
    router: &mut Router<SyncHandler>,
    artifacts: BTreeMap<String, CloudIacAppArchiveArtifact>,
    authz_provider: Arc<CloudIacModuleRegistryAuthzProvider>,
) -> Result<(), RouterError> {
    if artifacts.is_empty() {
        return Ok(());
    }
    router.route(
        HttpMethod::Get,
        CLOUD_IAC_APP_ARTIFACT_ROUTE_TEMPLATE,
        archive_artifact_handler(Arc::new(artifacts), authz_provider),
    )?;
    Ok(())
}

fn archive_artifact_handler(
    artifacts: Arc<BTreeMap<String, CloudIacAppArchiveArtifact>>,
    authz_provider: Arc<CloudIacModuleRegistryAuthzProvider>,
) -> SyncHandler {
    Arc::new(move |request: HttpRequest| {
        // SECOND PEP (supply-chain): the artifact route serves module ZIP bytes,
        // so VERIFY the caller credential and PDP-authorize the DOWNLOAD surface
        // BEFORE reading any bytes. Fail-closed: missing/invalid → 401, deny/fault
        // → 403. The transport headers are never trusted as an authz decision.
        let credential = CallerCredential {
            authorization: request.headers.get("authorization").cloned(),
        };
        let verified = match authz_provider.verify_principal(&credential) {
            Ok(verified) => verified,
            Err(_) => return unauthorized_artifact_response(),
        };
        if authz_provider
            .ensure_authorized(&verified, CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE)
            .is_err()
        {
            return forbidden_artifact_response();
        }
        let Some(file_name) = request.path_captures.get("archive_file") else {
            return fixed_error_response(404, "artifact_not_found");
        };
        if !is_safe_artifact_request_name(file_name) {
            return fixed_error_response(400, "invalid_artifact_name");
        }
        let Some(artifact) = artifacts.get(file_name) else {
            return fixed_error_response(404, "artifact_not_found");
        };
        match fs::read(&artifact.archive_file) {
            Ok(bytes) => {
                if sha256_hex(&bytes) != artifact.archive_sha256 {
                    return fixed_error_response(409, "artifact_digest_mismatch");
                }
                HttpResponse::new(200)
                    .with_header("content-type", artifact.media_type.clone())
                    .with_body(bytes)
            }
            Err(_) => fixed_error_response(404, "artifact_not_found"),
        }
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex_lower(&Sha256::digest(bytes))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn is_safe_artifact_request_name(file_name: &str) -> bool {
    !file_name.is_empty()
        && file_name != "."
        && file_name != ".."
        && !file_name.contains('/')
        && !file_name.contains('\\')
        && !file_name.contains('?')
        && !file_name.contains('#')
        && file_name.ends_with(".zip")
        && file_name
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '-' | '.'))
}

fn register_health_routes(router: &mut Router<SyncHandler>) -> Result<(), RouterError> {
    router.route(
        HttpMethod::Get,
        CLOUD_IAC_HEALTH_PATH,
        fixed_json_handler(r#"{"status":"ok","service":"cloud-iac","check":"healthz"}"#),
    )?;
    router.route(
        HttpMethod::Get,
        CLOUD_IAC_LIVENESS_PATH,
        fixed_json_handler(r#"{"status":"ok","service":"cloud-iac","check":"livez"}"#),
    )?;
    Ok(())
}

fn fixed_error_response(status: u16, code: &'static str) -> HttpResponse {
    HttpResponse::new(status)
        .with_header("content-type", "application/json")
        .with_body(format!(r#"{{"error":"{code}"}}"#).into_bytes())
}

fn unauthorized_artifact_response() -> HttpResponse {
    HttpResponse::new(401)
        .with_header("content-type", "application/json")
        .with_header("www-authenticate", "Bearer")
        .with_body(br#"{"error":"unauthorized"}"#.to_vec())
}

fn forbidden_artifact_response() -> HttpResponse {
    fixed_error_response(403, "forbidden")
}

fn fixed_json_handler(body: &'static str) -> SyncHandler {
    Arc::new(move |_request: HttpRequest| {
        HttpResponse::new(200)
            .with_header("content-type", "application/json")
            .with_body(body.as_bytes().to_vec())
    })
}

pub fn dispatch_cloud_iac_app_request(
    service: &CloudIacAppService,
    request: HttpRequest,
) -> HttpResponse {
    dispatch_hyper_adapter_request(request, &service.router, &service.middleware)
}

pub fn serve_cloud_iac_app_on_listener(
    listener: StdTcpListener,
    service: CloudIacAppService,
) -> Result<(), CloudIacAppError> {
    let (router, middleware, server_config) = service.into_serve_parts();
    serve_on_std_listener(
        listener,
        Arc::new(router),
        Arc::new(middleware),
        server_config,
    )?;
    Ok(())
}

pub fn serve_bounded_cloud_iac_app_on_listener(
    listener: StdTcpListener,
    service: CloudIacAppService,
    max_connections: usize,
) -> Result<(), CloudIacAppError> {
    let (router, middleware, server_config) = service.into_serve_parts();
    serve_n_connections_on_std_listener(
        listener,
        Arc::new(router),
        Arc::new(middleware),
        server_config,
        max_connections,
    )?;
    Ok(())
}

pub fn run_cloud_iac_app(config: CloudIacAppConfig) -> Result<(), CloudIacAppError> {
    // BOOT-FATAL: refuse to serve the supply-chain surface without a verifiable
    // bearer SECRET and a bound principal id (no default-allow; AUTH-005).
    let authz_provider = config.module_registry_authz_provider()?;
    let service = build_cloud_iac_app_service_from_release_index_path(
        &config.release_index_path,
        authz_provider,
    )?;
    let listener = StdTcpListener::bind(config.bind_addr)
        .map_err(|error| CloudIacAppError::Bind(error.to_string()))?;
    serve_cloud_iac_app_on_listener(listener, service)
}

pub fn run_cloud_iac_app_from_env() -> Result<(), CloudIacAppError> {
    let config = CloudIacAppConfig::from_env_pairs(std::env::vars())?;
    run_cloud_iac_app(config)
}
