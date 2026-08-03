//! Cloud Network DNS API boundary for tenant DNS zone creation.
//!
//! This crate owns tenant/header/path/body normalization, idempotency, and
//! authenticated API projection before handing typed DNS zone creation requests
//! to the Cloud network kernel.

use std::collections::BTreeMap;

use network_domain::{
    CloudNetworkCatalog, CloudNetworkError, DnsZone, DnsZoneCreate, DnsZoneKind, DnsZoneState,
    NetworkRepo,
};
use data_boundary_kernel::{DataClass, parse_data_class_label};

pub mod authz;

pub use authz::{
    AuthzProviderConfigError, CallerCredential, CloudNetworkDnsAuthzProvider,
    ConfiguredBearerPrincipalVerifier, DnsZoneCreateAuthorizationError, DnsZoneCreateAuthorizer,
    DnsZoneCreateResource, PrincipalVerificationError, PrincipalVerifier, VerifiedPrincipal,
    constant_time_eq,
};

pub const CLOUD_NETWORK_DNS_ZONE_CREATE_SURFACE: &str = "cloud.network.dns.zone.create";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudNetworkDnsZoneCreateApiStatus {
    Created,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

impl CloudNetworkDnsZoneCreateApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Created => 201,
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::NotFound => 404,
            Self::Conflict => 409,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudNetworkDnsApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    PathZoneIdEmpty,
    ZoneIdMismatch,
    TenantMismatch,
    CallerUnauthenticated,
    VerifiedPrincipalMismatch,
    VerifiedTenantMismatch,
    AuthorizationDenied,
    IdempotencyKeyReused,
    ZoneKindInvalid,
    DataClassInvalid,
    NetworkInvalidRequest,
    NetworkForbidden,
    NetworkNotFound,
    NetworkConflict,
}

impl CloudNetworkDnsApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "CLOUD_NETWORK_DNS_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "CLOUD_NETWORK_DNS_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "CLOUD_NETWORK_DNS_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "CLOUD_NETWORK_DNS_PRINCIPAL_ID_EMPTY",
            Self::PathZoneIdEmpty => "CLOUD_NETWORK_DNS_PATH_ZONE_ID_EMPTY",
            Self::ZoneIdMismatch => "CLOUD_NETWORK_DNS_ZONE_ID_MISMATCH",
            Self::TenantMismatch => "CLOUD_NETWORK_DNS_TENANT_MISMATCH",
            Self::CallerUnauthenticated => "CLOUD_NETWORK_DNS_CALLER_UNAUTHENTICATED",
            Self::VerifiedPrincipalMismatch => "CLOUD_NETWORK_DNS_VERIFIED_PRINCIPAL_MISMATCH",
            Self::VerifiedTenantMismatch => "CLOUD_NETWORK_DNS_VERIFIED_TENANT_MISMATCH",
            Self::AuthorizationDenied => "CLOUD_NETWORK_DNS_AUTHORIZATION_DENIED",
            Self::IdempotencyKeyReused => "CLOUD_NETWORK_DNS_IDEMPOTENCY_KEY_REUSED",
            Self::ZoneKindInvalid => "CLOUD_NETWORK_DNS_ZONE_KIND_INVALID",
            Self::DataClassInvalid => "CLOUD_NETWORK_DNS_DATA_CLASS_INVALID",
            Self::NetworkInvalidRequest => "CLOUD_NETWORK_DNS_INVALID_REQUEST",
            Self::NetworkForbidden => "CLOUD_NETWORK_DNS_FORBIDDEN",
            Self::NetworkNotFound => "CLOUD_NETWORK_DNS_NOT_FOUND",
            Self::NetworkConflict => "CLOUD_NETWORK_DNS_CONFLICT",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkDnsApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkDnsApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkDnsZoneCreateRequest {
    pub resource_id: String,            // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub region: String,                 // data_class: PUBLIC
    pub name: String,                   // data_class: PUBLIC
    pub kind: String,                   // data_class: PUBLIC
    pub vpc_id: Option<String>,         // data_class: INTERNAL_ONLY
    pub dnssec_key_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub data_class: String,             // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkDnsZoneCreateApiRequest {
    pub path_zone_id: String,                        // data_class: INTERNAL_ONLY
    pub boundary: CloudNetworkDnsApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CloudNetworkDnsApiPrincipal,      // data_class: INTERNAL_ONLY
    /// The caller credential (e.g. bearer token). The boundary verifies this via
    /// the injected [`authz::PrincipalVerifier`]; the request-supplied
    /// `principal` above is only ever a CROSS-CHECK against the verified
    /// identity, never a grant. (C11 fix: a request blob never authorizes.)
    pub credential: authz::CallerCredential, // data_class: SECRET
    pub body: CloudNetworkDnsZoneCreateRequest,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudNetworkDnsZoneCreateIdempotencyLedger {
    entries: BTreeMap<
        CloudNetworkDnsIdempotencyLedgerKey,
        CloudNetworkDnsZoneCreateIdempotencyLedgerEntry,
    >, // data_class: INTERNAL_ONLY
}

impl CloudNetworkDnsZoneCreateIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CloudNetworkDnsIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudNetworkDnsZoneCreateIdempotencyLedgerEntry {
    fingerprint: CloudNetworkDnsRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CloudNetworkDnsZoneCreateApiResult,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudNetworkDnsRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

type CloudNetworkDnsZoneCreateApiResult =
    Result<CloudNetworkDnsZoneCreateSuccessResponse, CloudNetworkDnsApiError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkDnsZoneCreateSuccessResponse {
    pub data: CloudNetworkDnsZoneRecord, // data_class: INTERNAL_ONLY
    pub metadata: CloudNetworkDnsApiMetadata, // data_class: INTERNAL_ONLY
}

impl CloudNetworkDnsZoneCreateSuccessResponse {
    pub fn created(data: CloudNetworkDnsZoneRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: CloudNetworkDnsApiMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkDnsApiMetadata {
    pub request_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkDnsZoneRecord {
    pub resource_id: String,            // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub region: String,                 // data_class: PUBLIC
    pub name: String,                   // data_class: PUBLIC
    pub kind: String,                   // data_class: PUBLIC
    pub vpc_id: Option<String>,         // data_class: INTERNAL_ONLY
    pub dnssec_key_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub data_class: String,             // data_class: PUBLIC
    pub state: String,                  // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
    pub schema_version: u32,            // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkDnsApiErrorResponse {
    pub error: CloudNetworkDnsApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkDnsApiErrorBody {
    pub code: String,                                // data_class: INTERNAL_ONLY
    pub message: String,                             // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,           // data_class: INTERNAL_ONLY
    pub request_id: String,                          // data_class: INTERNAL_ONLY
    pub details: Vec<CloudNetworkDnsApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkDnsApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudNetworkDnsApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    EmptyPathZoneId,
    ZoneIdMismatch {
        path_zone_id: String,
        body_resource_id: String,
    },
    TenantMismatch {
        header_tenant_id: String,
        principal_tenant_id: String,
        body_tenant_id: String,
    },
    /// The caller credential did not verify (missing or invalid). 401.
    CallerUnauthenticated,
    /// The request-supplied principal id did not match the VERIFIED principal.
    /// 403 (an authenticated caller cannot act as a different principal).
    VerifiedPrincipalMismatch {
        verified_principal_id: String,
        request_principal_id: String,
    },
    /// The request/target tenant did not match the VERIFIED principal's tenant.
    /// 403 (an authenticated caller cannot act on another tenant's resource).
    VerifiedTenantMismatch {
        verified_tenant_id: String,
        request_tenant_id: String,
    },
    /// The PDP denied or refused (fail-closed) the create on the target
    /// resource. 403.
    AuthorizationDenied {
        surface: String,
    },
    IdempotencyKeyReused {
        idempotency_key: String,
    },
    InvalidZoneKindLabel {
        kind: String,
    },
    InvalidDataClassLabel {
        data_class: String,
    },
    Network(CloudNetworkError),
}

impl CloudNetworkDnsApiError {
    pub fn dns_zone_create_status(&self) -> CloudNetworkDnsZoneCreateApiStatus {
        match self.status_kind() {
            CloudNetworkDnsApiStatusKind::BadRequest => {
                CloudNetworkDnsZoneCreateApiStatus::BadRequest
            }
            CloudNetworkDnsApiStatusKind::Unauthorized => {
                CloudNetworkDnsZoneCreateApiStatus::Unauthorized
            }
            CloudNetworkDnsApiStatusKind::Forbidden => {
                CloudNetworkDnsZoneCreateApiStatus::Forbidden
            }
            CloudNetworkDnsApiStatusKind::NotFound => CloudNetworkDnsZoneCreateApiStatus::NotFound,
            CloudNetworkDnsApiStatusKind::Conflict => CloudNetworkDnsZoneCreateApiStatus::Conflict,
            CloudNetworkDnsApiStatusKind::UnprocessableEntity => {
                CloudNetworkDnsZoneCreateApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn dns_zone_create_status_code(&self) -> u16 {
        self.dns_zone_create_status().code()
    }

    pub fn code(&self) -> CloudNetworkDnsApiErrorCode {
        match self {
            Self::EmptyRequestId => CloudNetworkDnsApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => CloudNetworkDnsApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => CloudNetworkDnsApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => CloudNetworkDnsApiErrorCode::PrincipalIdEmpty,
            Self::EmptyPathZoneId => CloudNetworkDnsApiErrorCode::PathZoneIdEmpty,
            Self::ZoneIdMismatch { .. } => CloudNetworkDnsApiErrorCode::ZoneIdMismatch,
            Self::TenantMismatch { .. } => CloudNetworkDnsApiErrorCode::TenantMismatch,
            Self::CallerUnauthenticated => CloudNetworkDnsApiErrorCode::CallerUnauthenticated,
            Self::VerifiedPrincipalMismatch { .. } => {
                CloudNetworkDnsApiErrorCode::VerifiedPrincipalMismatch
            }
            Self::VerifiedTenantMismatch { .. } => {
                CloudNetworkDnsApiErrorCode::VerifiedTenantMismatch
            }
            Self::AuthorizationDenied { .. } => CloudNetworkDnsApiErrorCode::AuthorizationDenied,
            Self::IdempotencyKeyReused { .. } => CloudNetworkDnsApiErrorCode::IdempotencyKeyReused,
            Self::InvalidZoneKindLabel { .. } => CloudNetworkDnsApiErrorCode::ZoneKindInvalid,
            Self::InvalidDataClassLabel { .. } => CloudNetworkDnsApiErrorCode::DataClassInvalid,
            Self::Network(error) => match cloud_network_status_kind(error) {
                CloudNetworkDnsApiStatusKind::BadRequest => {
                    CloudNetworkDnsApiErrorCode::NetworkInvalidRequest
                }
                CloudNetworkDnsApiStatusKind::Forbidden => {
                    CloudNetworkDnsApiErrorCode::NetworkForbidden
                }
                CloudNetworkDnsApiStatusKind::NotFound => {
                    CloudNetworkDnsApiErrorCode::NetworkNotFound
                }
                CloudNetworkDnsApiStatusKind::Conflict => {
                    CloudNetworkDnsApiErrorCode::NetworkConflict
                }
                CloudNetworkDnsApiStatusKind::Unauthorized
                | CloudNetworkDnsApiStatusKind::UnprocessableEntity => {
                    CloudNetworkDnsApiErrorCode::NetworkInvalidRequest
                }
            },
        }
    }

    pub fn error_response(&self, request_id: impl Into<String>) -> CloudNetworkDnsApiErrorResponse {
        CloudNetworkDnsApiErrorResponse {
            error: CloudNetworkDnsApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> CloudNetworkDnsApiStatusKind {
        match self {
            Self::EmptyPrincipalId | Self::CallerUnauthenticated => {
                CloudNetworkDnsApiStatusKind::Unauthorized
            }
            Self::TenantMismatch { .. }
            | Self::VerifiedPrincipalMismatch { .. }
            | Self::VerifiedTenantMismatch { .. }
            | Self::AuthorizationDenied { .. } => CloudNetworkDnsApiStatusKind::Forbidden,
            Self::IdempotencyKeyReused { .. } => CloudNetworkDnsApiStatusKind::UnprocessableEntity,
            Self::Network(error) => cloud_network_status_kind(error),
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::EmptyPathZoneId
            | Self::ZoneIdMismatch { .. }
            | Self::InvalidZoneKindLabel { .. }
            | Self::InvalidDataClassLabel { .. } => CloudNetworkDnsApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::EmptyPathZoneId => "Path DNS zone id is required",
            Self::ZoneIdMismatch { .. } => "Path and body DNS zone ids must match",
            Self::TenantMismatch { .. } => {
                "Tenant header must match authenticated principal and request body"
            }
            Self::CallerUnauthenticated => "A verified caller credential is required",
            Self::VerifiedPrincipalMismatch { .. } => {
                "Request principal must match the verified caller identity"
            }
            Self::VerifiedTenantMismatch { .. } => {
                "Request tenant must match the verified caller identity"
            }
            Self::AuthorizationDenied { .. } => {
                "The policy decision point denied the requested Cloud Network DNS surface"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::InvalidZoneKindLabel { .. } => "Request DNS zone kind must be public or private",
            Self::InvalidDataClassLabel { .. } => {
                "Request data_class must be a known privacy data class"
            }
            Self::Network(error) => cloud_network_message(error),
        }
    }

    fn details(&self) -> Vec<CloudNetworkDnsApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::EmptyPathZoneId => vec![detail("path.zone_id", "must be non-empty")],
            Self::ZoneIdMismatch { .. } => vec![detail(
                "resource_id",
                "path zone_id and body resource_id must match",
            )],
            Self::TenantMismatch { .. } => vec![detail(
                "tenant_id",
                "header tenant, principal tenant, and body tenant_id must match",
            )],
            Self::CallerUnauthenticated => vec![detail(
                "header.Authorization",
                "must present a credential the policy decision point can verify",
            )],
            Self::VerifiedPrincipalMismatch { .. } => vec![detail(
                "principal.principal_id",
                "must equal the verified caller principal id",
            )],
            Self::VerifiedTenantMismatch { .. } => vec![detail(
                "tenant_id",
                "must equal the verified caller tenant id",
            )],
            Self::AuthorizationDenied { .. } => vec![detail(
                "authorization",
                "the policy decision point must allow this principal/action/resource",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::InvalidZoneKindLabel { .. } => {
                vec![detail("body.kind", "must be public or private")]
            }
            Self::InvalidDataClassLabel { .. } => vec![detail(
                "body.data_class",
                "must be a canonical privacy data-class label",
            )],
            Self::Network(error) => vec![detail("cloud_network", cloud_network_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CloudNetworkDnsApiStatusKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

/// Validate the request SHAPE (boundary headers, path/body id binding, tenant
/// self-consistency). This does NOT authorize — authorization requires a
/// verified credential and a PDP decision and lives in
/// [`create_cloud_network_dns_zone_from_api`], which holds the injected
/// [`authz::CloudNetworkDnsAuthzProvider`]. Shape validation alone NEVER grants
/// the request.
pub fn validate_cloud_network_dns_zone_create_request(
    request: &CloudNetworkDnsZoneCreateApiRequest,
) -> Result<(), CloudNetworkDnsApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_zone_id(&request.path_zone_id, &request.body.resource_id)?;
    validate_tenant_binding(
        &request.boundary,
        &request.principal,
        &request.body.tenant_id,
    )
}

/// Create a Cloud Network DNS zone from an API request, FAIL-CLOSED.
///
/// The flow is (C11 fix; ADR-0587):
/// 1. validate request shape (headers, id binding, tenant self-consistency);
/// 2. VERIFY the caller credential via the injected provider → `401`
///    ([`CloudNetworkDnsApiError::CallerUnauthenticated`]) on missing/invalid;
/// 3. CROSS-CHECK the request principal/tenant against the verified identity →
///    `403` on mismatch (a verified caller cannot act as another
///    principal/tenant);
/// 4. AUTHORIZE via the PDP against the TARGET `{tenant, dns_zone}` derived from
///    the trusted body → `403` on deny/fault (fail-closed, never allow);
/// 5. only then mutate the catalog.
///
/// The `authz_provider` is REQUIRED (non-optional): there is no code path to the
/// mutation that skips the gate, and there is no default-allow provider.
pub fn create_cloud_network_dns_zone_from_api(
    catalog: &mut CloudNetworkCatalog,
    idempotency_ledger: &mut CloudNetworkDnsZoneCreateIdempotencyLedger,
    authz_provider: &authz::CloudNetworkDnsAuthzProvider,
    request: CloudNetworkDnsZoneCreateApiRequest,
) -> Result<CloudNetworkDnsZoneCreateSuccessResponse, CloudNetworkDnsApiError> {
    validate_cloud_network_dns_zone_create_request(&request)?;
    authorize_request(authz_provider, &request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_NETWORK_DNS_ZONE_CREATE_SURFACE,
    );
    let fingerprint = dns_zone_create_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudNetworkDnsApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let result = dns_zone_create_input(request.body)
        .and_then(|input| {
            catalog
                .create_dns_zone(input)
                .map_err(CloudNetworkDnsApiError::Network)
        })
        .map(|zone| {
            CloudNetworkDnsZoneCreateSuccessResponse::created(dns_zone_record(zone), request_id)
        });
    idempotency_ledger.entries.insert(
        key,
        CloudNetworkDnsZoneCreateIdempotencyLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

fn validate_boundary(
    boundary: &CloudNetworkDnsApiBoundaryContext,
) -> Result<(), CloudNetworkDnsApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(CloudNetworkDnsApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(CloudNetworkDnsApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(CloudNetworkDnsApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_zone_id(
    path_zone_id: &str,
    body_resource_id: &str,
) -> Result<(), CloudNetworkDnsApiError> {
    if path_zone_id.trim().is_empty() {
        return Err(CloudNetworkDnsApiError::EmptyPathZoneId);
    }
    if path_zone_id != body_resource_id {
        return Err(CloudNetworkDnsApiError::ZoneIdMismatch {
            path_zone_id: path_zone_id.to_string(),
            body_resource_id: body_resource_id.to_string(),
        });
    }
    Ok(())
}

fn validate_tenant_binding(
    boundary: &CloudNetworkDnsApiBoundaryContext,
    principal: &CloudNetworkDnsApiPrincipal,
    body_tenant_id: &str,
) -> Result<(), CloudNetworkDnsApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(CloudNetworkDnsApiError::EmptyPrincipalId);
    }
    if boundary.tenant_id != principal.tenant_id || boundary.tenant_id != body_tenant_id {
        return Err(CloudNetworkDnsApiError::TenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
            body_tenant_id: body_tenant_id.to_string(),
        });
    }
    Ok(())
}

/// FAIL-CLOSED authorization: verify the caller credential, cross-check the
/// request principal/tenant against the verified identity, then ask the PDP for
/// a decision against the TARGET resource (the trusted body tenant + path/body
/// DNS zone id). No request-supplied blob authorizes anything.
fn authorize_request(
    authz_provider: &authz::CloudNetworkDnsAuthzProvider,
    request: &CloudNetworkDnsZoneCreateApiRequest,
) -> Result<(), CloudNetworkDnsApiError> {
    // (2) Verify the caller credential. Missing/invalid → 401. The verified
    // principal — never the request blob — is the source of truth.
    let verified = authz_provider
        .verify_principal(&request.credential)
        .map_err(|error| match error {
            authz::PrincipalVerificationError::MissingCredential
            | authz::PrincipalVerificationError::InvalidCredential => {
                CloudNetworkDnsApiError::CallerUnauthenticated
            }
        })?;

    // (3) Cross-check the request-asserted principal/tenant against the verified
    // identity. A verified caller cannot act as another principal or tenant.
    if verified.principal_id() != request.principal.principal_id {
        return Err(CloudNetworkDnsApiError::VerifiedPrincipalMismatch {
            verified_principal_id: verified.principal_id().to_string(),
            request_principal_id: request.principal.principal_id.clone(),
        });
    }
    if verified.tenant_id() != request.body.tenant_id {
        return Err(CloudNetworkDnsApiError::VerifiedTenantMismatch {
            verified_tenant_id: verified.tenant_id().to_string(),
            request_tenant_id: request.body.tenant_id.clone(),
        });
    }

    // (4) Ask the PDP for a decision against the TARGET resource. The tenant is
    // the trusted body tenant (not a flattened caller tenant); a cross-tenant
    // create is deniable here. Deny OR any fault → 403 (fail-closed).
    let resource = authz::DnsZoneCreateResource {
        tenant_id: request.body.tenant_id.clone(),
        dns_zone_id: request.body.resource_id.clone(),
    };
    authz_provider
        .ensure_authorized(&verified, &resource)
        .map_err(|error| match error {
            authz::DnsZoneCreateAuthorizationError::Denied
            | authz::DnsZoneCreateAuthorizationError::Refused => {
                CloudNetworkDnsApiError::AuthorizationDenied {
                    surface: CLOUD_NETWORK_DNS_ZONE_CREATE_SURFACE.to_string(),
                }
            }
        })
}

fn dns_zone_create_input(
    body: CloudNetworkDnsZoneCreateRequest,
) -> Result<DnsZoneCreate, CloudNetworkDnsApiError> {
    Ok(DnsZoneCreate {
        resource_id: body.resource_id,
        tenant_id: body.tenant_id,
        region: body.region,
        name: body.name,
        kind: parse_api_zone_kind(body.kind)?,
        vpc_id: body.vpc_id,
        dnssec_key_ref: body.dnssec_key_ref,
        state: DnsZoneState::Creating,
        data_class: parse_api_data_class(body.data_class)?,
        created_at_epoch_seconds: body.created_at_epoch_seconds,
    })
}

fn parse_api_zone_kind(label: String) -> Result<DnsZoneKind, CloudNetworkDnsApiError> {
    match label.as_str() {
        "public" => Ok(DnsZoneKind::Public),
        "private" => Ok(DnsZoneKind::Private),
        _ => Err(CloudNetworkDnsApiError::InvalidZoneKindLabel { kind: label }),
    }
}

fn parse_api_data_class(label: String) -> Result<DataClass, CloudNetworkDnsApiError> {
    parse_data_class_label(&label)
        .ok_or(CloudNetworkDnsApiError::InvalidDataClassLabel { data_class: label })
}

fn idempotency_key_for(
    boundary: &CloudNetworkDnsApiBoundaryContext,
    principal: &CloudNetworkDnsApiPrincipal,
    surface: &str,
) -> CloudNetworkDnsIdempotencyLedgerKey {
    CloudNetworkDnsIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn dns_zone_create_fingerprint_for(
    request: &CloudNetworkDnsZoneCreateApiRequest,
) -> CloudNetworkDnsRequestFingerprint {
    CloudNetworkDnsRequestFingerprint {
        canonical: [
            format!("path.zone_id={}", request.path_zone_id),
            format!("header.tenant_id={}", request.boundary.tenant_id),
            format!("principal.tenant_id={}", request.principal.tenant_id),
            format!("principal.principal_id={}", request.principal.principal_id),
            format!("body.resource_id={}", request.body.resource_id),
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.region={}", request.body.region),
            format!("body.name={}", request.body.name),
            format!("body.kind={}", request.body.kind),
            format!("body.vpc_id={:?}", request.body.vpc_id),
            format!("body.dnssec_key_ref={:?}", request.body.dnssec_key_ref),
            format!("body.data_class={}", request.body.data_class),
            format!(
                "body.created_at_epoch_seconds={}",
                request.body.created_at_epoch_seconds
            ),
        ]
        .join("|"),
    }
}

fn dns_zone_record(zone: DnsZone) -> CloudNetworkDnsZoneRecord {
    CloudNetworkDnsZoneRecord {
        resource_id: zone.resource_id.value.value,
        tenant_id: zone.tenant_id.value,
        region: zone.region.value.value,
        name: zone.name.value.value,
        kind: dns_zone_kind_label(zone.kind.value).to_string(),
        vpc_id: zone.vpc_id.value.map(|id| id.value),
        dnssec_key_ref: zone.dnssec_key_ref.value.map(|key| key.value),
        data_class: zone.data_class.value.label().to_string(),
        state: dns_zone_state_label(zone.state.value).to_string(),
        created_at_epoch_seconds: zone.created_at_epoch_seconds.value,
        schema_version: zone.schema_version.value,
    }
}

fn dns_zone_kind_label(kind: DnsZoneKind) -> &'static str {
    match kind {
        DnsZoneKind::Public => "public",
        DnsZoneKind::Private => "private",
    }
}

fn dns_zone_state_label(state: DnsZoneState) -> &'static str {
    match state {
        DnsZoneState::Creating => "creating",
        DnsZoneState::Active => "active",
        DnsZoneState::Suspended => "suspended",
        DnsZoneState::Deleting => "deleting",
    }
}

fn cloud_network_status_kind(error: &CloudNetworkError) -> CloudNetworkDnsApiStatusKind {
    match error {
        CloudNetworkError::DuplicateRoute
        | CloudNetworkError::DuplicateSecurityGroup
        | CloudNetworkError::DuplicateSubnet
        | CloudNetworkError::DuplicateTargetGroup
        | CloudNetworkError::DuplicateListenerPort
        | CloudNetworkError::DuplicateCdnOrigin
        | CloudNetworkError::DuplicateCdnHostname
        | CloudNetworkError::DuplicateBgpSession
        | CloudNetworkError::DuplicateInterconnectPartner
        | CloudNetworkError::DuplicateProtectedResource
        | CloudNetworkError::DuplicateServiceMesh
        | CloudNetworkError::DuplicateFlowAnomaly
        | CloudNetworkError::DuplicateVpc
        | CloudNetworkError::DuplicateLoadBalancer
        | CloudNetworkError::DuplicateDnsZone
        | CloudNetworkError::DuplicateCdnDistribution
        | CloudNetworkError::DuplicateDirectInterconnect
        | CloudNetworkError::DuplicateDdosProtection => CloudNetworkDnsApiStatusKind::Conflict,
        CloudNetworkError::UnknownVpc
        | CloudNetworkError::UnknownSubnet
        | CloudNetworkError::UnknownLoadBalancer
        | CloudNetworkError::UnknownDnsZone
        | CloudNetworkError::UnknownInterconnectPartner
        | CloudNetworkError::UnknownProtectedResource => CloudNetworkDnsApiStatusKind::NotFound,
        CloudNetworkError::ResourceTenantMismatch
        | CloudNetworkError::ResourceRegionMismatch
        | CloudNetworkError::AzRegionMismatch
        | CloudNetworkError::SubnetOutsideVpc
        | CloudNetworkError::ListenerTargetGroupMissing
        | CloudNetworkError::PrivateZoneRequiresVpc
        | CloudNetworkError::PublicZoneMustNotBindVpc
        | CloudNetworkError::ScrubbingRegionRequired => CloudNetworkDnsApiStatusKind::Forbidden,
        CloudNetworkError::InvalidTenantId
        | CloudNetworkError::InvalidResourceId
        | CloudNetworkError::ResourceKindMismatch
        | CloudNetworkError::InvalidDataClass
        | CloudNetworkError::InvalidVpcState
        | CloudNetworkError::InvalidSubnetState
        | CloudNetworkError::InvalidLbState
        | CloudNetworkError::InvalidDnsZoneState
        | CloudNetworkError::InvalidIpv4Cidr
        | CloudNetworkError::InvalidIpv6Cidr
        | CloudNetworkError::InvalidCidrPrefix
        | CloudNetworkError::Ipv6Required
        | CloudNetworkError::InvalidRouteTableId
        | CloudNetworkError::InvalidRoute
        | CloudNetworkError::InvalidSecurityGroupId
        | CloudNetworkError::InvalidSecurityRule
        | CloudNetworkError::InvalidAzCode
        | CloudNetworkError::OverlappingSubnet
        | CloudNetworkError::InvalidTargetGroupId
        | CloudNetworkError::InvalidListener
        | CloudNetworkError::L7RequiresTls
        | CloudNetworkError::GrpcRequiresMtls
        | CloudNetworkError::InvalidCertificateRef
        | CloudNetworkError::InvalidWafPolicyId
        | CloudNetworkError::InvalidDnsName
        | CloudNetworkError::InvalidDnssecKeyRef
        | CloudNetworkError::DnssecRequired
        | CloudNetworkError::InvalidCdnState
        | CloudNetworkError::InvalidCdnOrigin
        | CloudNetworkError::CdnWafRequired
        | CloudNetworkError::CdnTlsRequired
        | CloudNetworkError::InvalidInterconnectPartnerId
        | CloudNetworkError::InvalidInterconnectPortId
        | CloudNetworkError::InvalidPeeringLocation
        | CloudNetworkError::InvalidInterconnectState
        | CloudNetworkError::InvalidBandwidth
        | CloudNetworkError::InvalidVlanTag
        | CloudNetworkError::InvalidBgpSessionId
        | CloudNetworkError::InvalidBgpSession
        | CloudNetworkError::InvalidAsn
        | CloudNetworkError::InterconnectRedundancyRequired
        | CloudNetworkError::InterconnectSlaRequired
        | CloudNetworkError::RegionalInterconnectDiversityRequired
        | CloudNetworkError::InvalidDdosState
        | CloudNetworkError::LineRateScrubbingRequired
        | CloudNetworkError::DdosAlwaysOnRequired
        | CloudNetworkError::InvalidRunbookRef
        | CloudNetworkError::InvalidOnCallGroupRef
        | CloudNetworkError::InvalidMeshId
        | CloudNetworkError::InvalidCellId
        | CloudNetworkError::InvalidMeshNamespace
        | CloudNetworkError::InvalidMeshState
        | CloudNetworkError::InvalidMeshMode
        | CloudNetworkError::InvalidMeshGateway
        | CloudNetworkError::MeshMtlsRequired
        | CloudNetworkError::MeshExtAuthzRequired
        | CloudNetworkError::InvalidCedarPolicyRef
        | CloudNetworkError::InvalidAuditStreamRef
        | CloudNetworkError::InvalidHealthAlarmRef
        | CloudNetworkError::MeshControlPlaneReplicasRequired
        | CloudNetworkError::MeshUpgradeDrillRequired
        | CloudNetworkError::DefaultDenyIngressRequired
        | CloudNetworkError::DefaultDenyEgressRequired
        | CloudNetworkError::DnsEgressExceptionRequired
        | CloudNetworkError::CrossCellDefaultTrafficForbidden
        | CloudNetworkError::EnvoyExtAuthzRequired
        | CloudNetworkError::EnvoyFailClosedRequired
        | CloudNetworkError::CoreDnsInsecurePodModeForbidden
        | CloudNetworkError::EvidenceRefMissing
        | CloudNetworkError::EvidenceRefLooksSecretLike
        | CloudNetworkError::InvalidFlowAnomalyId
        | CloudNetworkError::FlowLogsRequired => CloudNetworkDnsApiStatusKind::BadRequest,
    }
}

fn cloud_network_message(error: &CloudNetworkError) -> &'static str {
    match cloud_network_status_kind(error) {
        CloudNetworkDnsApiStatusKind::BadRequest => "Cloud Network rejected the request shape",
        CloudNetworkDnsApiStatusKind::Unauthorized => "Cloud Network authentication is required",
        CloudNetworkDnsApiStatusKind::Forbidden => "Cloud Network policy denied the request",
        CloudNetworkDnsApiStatusKind::NotFound => "Cloud Network resource was not found",
        CloudNetworkDnsApiStatusKind::Conflict => "Cloud Network resource already exists",
        CloudNetworkDnsApiStatusKind::UnprocessableEntity => {
            "Cloud Network rejected request idempotency"
        }
    }
}

fn cloud_network_issue(error: &CloudNetworkError) -> &'static str {
    match error {
        CloudNetworkError::InvalidTenantId => "tenant_id must be a ten_ identifier",
        CloudNetworkError::InvalidResourceId => "resource_id must be canonical cloud resource id",
        CloudNetworkError::ResourceTenantMismatch => "resource tenant must match request tenant",
        CloudNetworkError::ResourceRegionMismatch => {
            "resource region must match request region and residency"
        }
        CloudNetworkError::ResourceKindMismatch => "resource kind must match network type",
        CloudNetworkError::InvalidDataClass => "data_class must be public metadata for VPC create",
        CloudNetworkError::InvalidVpcState => "VPC create requests must start in Creating state",
        CloudNetworkError::InvalidSubnetState => {
            "subnet create requests must start in Creating state"
        }
        CloudNetworkError::InvalidLbState => {
            "load balancer create requests must start in Creating state"
        }
        CloudNetworkError::InvalidDnsZoneState => {
            "DNS zone create requests must start in Creating state"
        }
        CloudNetworkError::InvalidIpv4Cidr => "IPv4 CIDR must be canonical",
        CloudNetworkError::InvalidIpv6Cidr => "IPv6 CIDR must be canonical",
        CloudNetworkError::InvalidCidrPrefix => "CIDR prefix length is out of range",
        CloudNetworkError::Ipv6Required => "IPv6 CIDR is required",
        CloudNetworkError::InvalidRouteTableId => "route table id must use the rtb_ prefix",
        CloudNetworkError::InvalidRoute => "route next hop and target reference must be consistent",
        CloudNetworkError::DuplicateRoute => "route destinations must be unique",
        CloudNetworkError::InvalidSecurityGroupId => "security group id must use the sg_ prefix",
        CloudNetworkError::InvalidSecurityRule => {
            "security rule must use valid ports and description"
        }
        CloudNetworkError::DuplicateSecurityGroup => "security group ids must be unique",
        CloudNetworkError::InvalidAzCode => "AZ must be canonical lowercase ASCII",
        CloudNetworkError::AzRegionMismatch => "AZ code must sit under its region code",
        CloudNetworkError::SubnetOutsideVpc => "subnet CIDR must fit inside the VPC CIDR",
        CloudNetworkError::OverlappingSubnet => "subnet CIDRs must not overlap",
        CloudNetworkError::DuplicateSubnet => "subnet resource id is already present",
        CloudNetworkError::UnknownVpc => "referenced VPC must exist",
        CloudNetworkError::UnknownSubnet => "referenced subnet must exist",
        CloudNetworkError::InvalidTargetGroupId => "target group id must use the tg_ prefix",
        CloudNetworkError::DuplicateTargetGroup => "target group ids must be unique",
        CloudNetworkError::InvalidListener => "listener must bind a valid port and target group",
        CloudNetworkError::DuplicateListenerPort => "listener ports must be unique",
        CloudNetworkError::ListenerTargetGroupMissing => "listener target group must exist",
        CloudNetworkError::L7RequiresTls => "L7 load balancers require TLS",
        CloudNetworkError::GrpcRequiresMtls => "gRPC load balancers require mTLS",
        CloudNetworkError::InvalidCertificateRef => {
            "certificate reference must use the cert/ prefix"
        }
        CloudNetworkError::InvalidWafPolicyId => "WAF policy id must use the waf_ prefix",
        CloudNetworkError::InvalidDnsName => "DNS name must be a canonical DNS label",
        CloudNetworkError::InvalidDnssecKeyRef => {
            "DNSSEC key reference must use the dnssec/ prefix"
        }
        CloudNetworkError::DnssecRequired => "DNSSEC key reference is required",
        CloudNetworkError::PrivateZoneRequiresVpc => "private DNS zones require a VPC binding",
        CloudNetworkError::PublicZoneMustNotBindVpc => "public DNS zones must not bind to a VPC",
        CloudNetworkError::InvalidCdnState => "CDN create requests must start in Creating state",
        CloudNetworkError::InvalidCdnOrigin => "CDN origin must reference a supported origin type",
        CloudNetworkError::DuplicateCdnOrigin => "CDN origins must be unique",
        CloudNetworkError::UnknownLoadBalancer => "referenced load balancer must exist",
        CloudNetworkError::UnknownDnsZone => "referenced DNS zone must exist",
        CloudNetworkError::CdnWafRequired => "CDN distribution requires a WAF policy",
        CloudNetworkError::CdnTlsRequired => "CDN distribution requires a TLS certificate",
        CloudNetworkError::DuplicateCdnHostname => "CDN hostnames must be unique",
        CloudNetworkError::InvalidInterconnectPartnerId => {
            "interconnect partner id must use the ixp_ prefix"
        }
        CloudNetworkError::InvalidInterconnectPortId => {
            "interconnect port id must use the icp_ prefix"
        }
        CloudNetworkError::InvalidPeeringLocation => "peering location must be canonical",
        CloudNetworkError::InvalidInterconnectState => {
            "direct interconnect create requests must start in Creating state"
        }
        CloudNetworkError::InvalidBandwidth => "bandwidth must be greater than zero",
        CloudNetworkError::InvalidVlanTag => "VLAN tag must be in the valid customer range",
        CloudNetworkError::InvalidBgpSessionId => "BGP session id must use the bgp_ prefix",
        CloudNetworkError::InvalidBgpSession => "BGP session addresses must be valid",
        CloudNetworkError::DuplicateBgpSession => "BGP session ids must be unique",
        CloudNetworkError::InvalidAsn => "ASN must be non-zero",
        CloudNetworkError::InterconnectRedundancyRequired => {
            "interconnect redundancy must meet minimum policy"
        }
        CloudNetworkError::InterconnectSlaRequired => "interconnect SLA must meet policy",
        CloudNetworkError::RegionalInterconnectDiversityRequired => {
            "interconnect must include regional diversity"
        }
        CloudNetworkError::UnknownInterconnectPartner => "interconnect partner must exist",
        CloudNetworkError::DuplicateInterconnectPartner => {
            "interconnect partner id is already present"
        }
        CloudNetworkError::DuplicateDirectInterconnect => {
            "direct interconnect resource id is already present"
        }
        CloudNetworkError::InvalidDdosState => "DDoS create requests must start in Creating state",
        CloudNetworkError::UnknownProtectedResource => "protected resource must exist",
        CloudNetworkError::DuplicateProtectedResource => "protected resources must be unique",
        CloudNetworkError::ScrubbingRegionRequired => {
            "scrubbing region must satisfy residency policy"
        }
        CloudNetworkError::LineRateScrubbingRequired => "line-rate scrubbing is required",
        CloudNetworkError::DdosAlwaysOnRequired => "always-on DDoS protection is required",
        CloudNetworkError::InvalidRunbookRef => "runbook reference must use the runbook/ prefix",
        CloudNetworkError::InvalidOnCallGroupRef => {
            "on-call group reference must use the oncall/ prefix"
        }
        CloudNetworkError::DuplicateDdosProtection => {
            "DDoS protection resource id is already present"
        }
        CloudNetworkError::InvalidMeshId => "service mesh id must use the mesh_ prefix",
        CloudNetworkError::InvalidCellId => "cell id must be canonical",
        CloudNetworkError::InvalidMeshNamespace => "mesh namespace must be canonical",
        CloudNetworkError::InvalidMeshState => {
            "service mesh create requests must start in Creating state"
        }
        CloudNetworkError::InvalidMeshMode => "service mesh mode must be supported",
        CloudNetworkError::InvalidMeshGateway => "service mesh gateway must be supported",
        CloudNetworkError::MeshMtlsRequired => "service mesh requires mTLS everywhere",
        CloudNetworkError::MeshExtAuthzRequired => "service mesh requires external authorization",
        CloudNetworkError::InvalidCedarPolicyRef => {
            "Cedar policy reference must use the cedar/ prefix"
        }
        CloudNetworkError::InvalidAuditStreamRef => {
            "audit stream reference must use the audit/ prefix"
        }
        CloudNetworkError::InvalidHealthAlarmRef => {
            "health alarm reference must use the alarm/ prefix"
        }
        CloudNetworkError::MeshControlPlaneReplicasRequired => {
            "service mesh requires control-plane replicas"
        }
        CloudNetworkError::MeshUpgradeDrillRequired => {
            "service mesh requires quarterly upgrade drills"
        }
        CloudNetworkError::DefaultDenyIngressRequired => {
            "cell guardrail requires default-deny ingress"
        }
        CloudNetworkError::DefaultDenyEgressRequired => {
            "cell guardrail requires default-deny egress"
        }
        CloudNetworkError::DnsEgressExceptionRequired => {
            "cell guardrail requires an explicit DNS egress exception"
        }
        CloudNetworkError::CrossCellDefaultTrafficForbidden => {
            "cell guardrail forbids default cross-cell traffic"
        }
        CloudNetworkError::EnvoyExtAuthzRequired => {
            "Envoy guardrail requires external authorization"
        }
        CloudNetworkError::EnvoyFailClosedRequired => {
            "Envoy external authorization must fail closed"
        }
        CloudNetworkError::CoreDnsInsecurePodModeForbidden => {
            "CoreDNS pod mode must not be insecure"
        }
        CloudNetworkError::EvidenceRefMissing => "evidence ref must use evidence://",
        CloudNetworkError::EvidenceRefLooksSecretLike => {
            "evidence ref must not contain secret-like material"
        }
        CloudNetworkError::DuplicateServiceMesh => "service mesh id is already present",
        CloudNetworkError::InvalidFlowAnomalyId => "flow anomaly id must use the flowanom_ prefix",
        CloudNetworkError::FlowLogsRequired => "VPC flow logs must be enabled",
        CloudNetworkError::DuplicateFlowAnomaly => "flow anomaly id is already present",
        CloudNetworkError::DuplicateVpc => "VPC resource id is already present",
        CloudNetworkError::DuplicateLoadBalancer => "load balancer resource id is already present",
        CloudNetworkError::DuplicateDnsZone => "DNS zone resource id is already present",
        CloudNetworkError::DuplicateCdnDistribution => {
            "CDN distribution resource id is already present"
        }
    }
}

fn detail(field: &str, issue: &str) -> CloudNetworkDnsApiErrorDetail {
    CloudNetworkDnsApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}
