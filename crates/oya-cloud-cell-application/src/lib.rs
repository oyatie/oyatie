//! Cloud Cell API boundary.
//!
//! This crate owns tenant/header/path/body normalization, idempotency, and
//! authenticated API projection before handing typed route-binding requests to
//! the Cloud Region and platform Cell kernels.

use std::collections::BTreeMap;

use oya_cloud_region_kernel::{
    CloudRegionCatalog, CloudRegionError, TenantCellRouteRequest, TenantDensityClass,
};
use oya_platform_cell_kernel::{CellBinding, CellError, CellRouter, CellTier};
use oya_platform_residency_kernel::{parse_residency_class_label, ResidencyClass};

pub const CLOUD_CELL_BIND_SURFACE: &str = "cloud.cell.bind";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudCellBindApiStatus {
    Created,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

impl CloudCellBindApiStatus {
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
pub enum CloudCellApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    PathTenantIdEmpty,
    TenantMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    IdempotencyKeyReused,
    ResidencyClassInvalid,
    TenantDensityInvalid,
    RegionInvalidRequest,
    RegionForbidden,
    RegionNotFound,
    RegionConflict,
    RegionUnprocessable,
}

impl CloudCellApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "CLOUD_CELL_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "CLOUD_CELL_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "CLOUD_CELL_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "CLOUD_CELL_PRINCIPAL_ID_EMPTY",
            Self::PathTenantIdEmpty => "CLOUD_CELL_PATH_TENANT_ID_EMPTY",
            Self::TenantMismatch => "CLOUD_CELL_TENANT_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => "CLOUD_CELL_AUTHORIZATION_DECISION_ID_EMPTY",
            Self::AuthorizationTenantMismatch => "CLOUD_CELL_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => "CLOUD_CELL_AUTHORIZATION_PRINCIPAL_MISMATCH",
            Self::AuthorizationDenied => "CLOUD_CELL_AUTHORIZATION_DENIED",
            Self::IdempotencyKeyReused => "CLOUD_CELL_IDEMPOTENCY_KEY_REUSED",
            Self::ResidencyClassInvalid => "CLOUD_CELL_RESIDENCY_CLASS_INVALID",
            Self::TenantDensityInvalid => "CLOUD_CELL_TENANT_DENSITY_INVALID",
            Self::RegionInvalidRequest => "CLOUD_CELL_REGION_INVALID_REQUEST",
            Self::RegionForbidden => "CLOUD_CELL_REGION_FORBIDDEN",
            Self::RegionNotFound => "CLOUD_CELL_REGION_NOT_FOUND",
            Self::RegionConflict => "CLOUD_CELL_REGION_CONFLICT",
            Self::RegionUnprocessable => "CLOUD_CELL_REGION_UNPROCESSABLE",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudCellApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudCellApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudCellApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudCellBindRequest {
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub home_region_code: String,         // data_class: INTERNAL_ONLY
    pub residency_class: String,          // data_class: INTERNAL_ONLY
    pub required_density: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudCellBindApiRequest {
    pub path_tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub boundary: CloudCellApiBoundaryContext,    // data_class: INTERNAL_ONLY
    pub principal: CloudCellApiPrincipal,         // data_class: INTERNAL_ONLY
    pub authorization: CloudCellApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: CloudCellBindRequest,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudCellBindIdempotencyLedger {
    entries: BTreeMap<CloudCellBindIdempotencyLedgerKey, CloudCellBindIdempotencyLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl CloudCellBindIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CloudCellBindIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudCellBindIdempotencyLedgerEntry {
    fingerprint: CloudCellBindRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CloudCellBindApiResult,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudCellBindRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

type CloudCellBindApiResult = Result<CloudCellBindSuccessResponse, CloudCellApiError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudCellBindSuccessResponse {
    pub data: CloudCellBindingRecord,   // data_class: INTERNAL_ONLY
    pub metadata: CloudCellApiMetadata, // data_class: INTERNAL_ONLY
}

impl CloudCellBindSuccessResponse {
    pub fn created(
        data: CloudCellBindingRecord,
        request_id: impl Into<String>,
        tenant_id: impl Into<String>,
        region: impl Into<String>,
    ) -> Self {
        Self {
            data,
            metadata: CloudCellApiMetadata {
                request_id: request_id.into(),
                tenant_id: tenant_id.into(),
                region: region.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudCellApiMetadata {
    pub request_id: String, // data_class: INTERNAL_ONLY
    pub tenant_id: String,  // data_class: INTERNAL_ONLY
    pub region: String,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudCellBindingRecord {
    pub tenant_id: String,         // data_class: INTERNAL_ONLY
    pub region: String,            // data_class: INTERNAL_ONLY
    pub residency_class: String,   // data_class: INTERNAL_ONLY
    pub az: String,                // data_class: INTERNAL_ONLY
    pub cell_id: String,           // data_class: INTERNAL_ONLY
    pub tier: String,              // data_class: INTERNAL_ONLY
    pub hsm_partition_ref: String, // data_class: INTERNAL_ONLY
    pub schema_version: u32,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudCellApiErrorResponse {
    pub error: CloudCellApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudCellApiErrorBody {
    pub code: String,                          // data_class: INTERNAL_ONLY
    pub message: String,                       // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,     // data_class: INTERNAL_ONLY
    pub request_id: String,                    // data_class: INTERNAL_ONLY
    pub details: Vec<CloudCellApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudCellApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudCellApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    EmptyPathTenantId,
    TenantMismatch {
        header_tenant_id: String,
        path_tenant_id: String,
        principal_tenant_id: String,
        body_tenant_id: String,
    },
    EmptyAuthorizationDecisionId,
    AuthorizationTenantMismatch {
        authorization_tenant_id: String,
        principal_tenant_id: String,
    },
    AuthorizationPrincipalMismatch {
        authorization_principal_id: String,
        principal_id: String,
    },
    AuthorizationDenied {
        surface: String,
    },
    IdempotencyKeyReused {
        idempotency_key: String,
    },
    InvalidResidencyClassLabel {
        residency_class: String,
    },
    InvalidTenantDensityLabel {
        required_density: String,
    },
    Region(CloudRegionError),
}

impl CloudCellApiError {
    pub fn cell_bind_status(&self) -> CloudCellBindApiStatus {
        match self.status_kind() {
            CloudCellApiStatusKind::BadRequest => CloudCellBindApiStatus::BadRequest,
            CloudCellApiStatusKind::Unauthorized => CloudCellBindApiStatus::Unauthorized,
            CloudCellApiStatusKind::Forbidden => CloudCellBindApiStatus::Forbidden,
            CloudCellApiStatusKind::NotFound => CloudCellBindApiStatus::NotFound,
            CloudCellApiStatusKind::Conflict => CloudCellBindApiStatus::Conflict,
            CloudCellApiStatusKind::UnprocessableEntity => {
                CloudCellBindApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn cell_bind_status_code(&self) -> u16 {
        self.cell_bind_status().code()
    }

    pub fn code(&self) -> CloudCellApiErrorCode {
        match self {
            Self::EmptyRequestId => CloudCellApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => CloudCellApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => CloudCellApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => CloudCellApiErrorCode::PrincipalIdEmpty,
            Self::EmptyPathTenantId => CloudCellApiErrorCode::PathTenantIdEmpty,
            Self::TenantMismatch { .. } => CloudCellApiErrorCode::TenantMismatch,
            Self::EmptyAuthorizationDecisionId => {
                CloudCellApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                CloudCellApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                CloudCellApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => CloudCellApiErrorCode::AuthorizationDenied,
            Self::IdempotencyKeyReused { .. } => CloudCellApiErrorCode::IdempotencyKeyReused,
            Self::InvalidResidencyClassLabel { .. } => CloudCellApiErrorCode::ResidencyClassInvalid,
            Self::InvalidTenantDensityLabel { .. } => CloudCellApiErrorCode::TenantDensityInvalid,
            Self::Region(error) => match cloud_region_status_kind(error) {
                CloudCellApiStatusKind::BadRequest => CloudCellApiErrorCode::RegionInvalidRequest,
                CloudCellApiStatusKind::Unauthorized => CloudCellApiErrorCode::RegionInvalidRequest,
                CloudCellApiStatusKind::Forbidden => CloudCellApiErrorCode::RegionForbidden,
                CloudCellApiStatusKind::NotFound => CloudCellApiErrorCode::RegionNotFound,
                CloudCellApiStatusKind::Conflict => CloudCellApiErrorCode::RegionConflict,
                CloudCellApiStatusKind::UnprocessableEntity => {
                    CloudCellApiErrorCode::RegionUnprocessable
                }
            },
        }
    }

    pub fn error_response(&self, request_id: impl Into<String>) -> CloudCellApiErrorResponse {
        CloudCellApiErrorResponse {
            error: CloudCellApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> CloudCellApiStatusKind {
        match self {
            Self::EmptyPrincipalId => CloudCellApiStatusKind::Unauthorized,
            Self::TenantMismatch { .. }
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. } => CloudCellApiStatusKind::Forbidden,
            Self::IdempotencyKeyReused { .. } => CloudCellApiStatusKind::UnprocessableEntity,
            Self::Region(error) => cloud_region_status_kind(error),
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::EmptyPathTenantId
            | Self::InvalidResidencyClassLabel { .. }
            | Self::InvalidTenantDensityLabel { .. } => CloudCellApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::EmptyPathTenantId => "Path tenant id is required",
            Self::TenantMismatch { .. } => {
                "Tenant header must match path tenant, authenticated principal, and request body"
            }
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the requested Cloud Cell binding surface"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::InvalidResidencyClassLabel { .. } => {
                "Request residency_class must be a supported residency class label"
            }
            Self::InvalidTenantDensityLabel { .. } => {
                "Request required_density must be a supported tenant density label"
            }
            Self::Region(error) => cloud_region_message(error),
        }
    }

    fn details(&self) -> Vec<CloudCellApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::EmptyPathTenantId => vec![detail("path.tenant_id", "must be non-empty")],
            Self::TenantMismatch { .. } => vec![detail(
                "tenant_id",
                "header tenant, path tenant, principal tenant, and body tenant_id must match",
            )],
            Self::EmptyAuthorizationDecisionId => vec![detail(
                "authorization.decision_id",
                "must be non-empty authorization evidence",
            )],
            Self::AuthorizationTenantMismatch { .. } => vec![detail(
                "authorization.tenant_id",
                "must match the authenticated principal tenant",
            )],
            Self::AuthorizationPrincipalMismatch { .. } => vec![detail(
                "authorization.principal_id",
                "must match the authenticated principal id",
            )],
            Self::AuthorizationDenied { .. } => vec![detail(
                "authorization.allowed_surfaces",
                "must include the requested Cloud Cell binding surface",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::InvalidResidencyClassLabel { .. } => vec![detail(
                "body.residency_class",
                "must be one of strict_kr, kr_with_us_failover, or global",
            )],
            Self::InvalidTenantDensityLabel { .. } => vec![detail(
                "body.required_density",
                "must be one of shared, dedicated, sovereign, air_gapped, or foundry_runtime",
            )],
            Self::Region(error) => vec![detail("cloud_region", cloud_region_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CloudCellApiStatusKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

pub fn validate_cloud_cell_bind_request(
    request: &CloudCellBindApiRequest,
) -> Result<(), CloudCellApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_tenant_id(&request.path_tenant_id)?;
    validate_tenant_binding(
        &request.boundary,
        &request.path_tenant_id,
        &request.principal,
        &request.body.tenant_id,
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        CLOUD_CELL_BIND_SURFACE,
    )?;
    validate_body_labels(&request.body)
}

pub fn bind_cloud_cell_from_api(
    catalog: &CloudRegionCatalog,
    router: &mut CellRouter,
    idempotency_ledger: &mut CloudCellBindIdempotencyLedger,
    request: CloudCellBindApiRequest,
) -> Result<CloudCellBindSuccessResponse, CloudCellApiError> {
    validate_cloud_cell_bind_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_CELL_BIND_SURFACE,
    );
    let fingerprint = cell_bind_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudCellApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let result = route_request_input(request.body)
        .and_then(|input| {
            catalog
                .bind_route_for_tenant(router, input)
                .map_err(CloudCellApiError::Region)
        })
        .map(|binding| {
            let tenant_id = binding.tenant_id.clone();
            let region = binding.region.clone();
            CloudCellBindSuccessResponse::created(
                cell_binding_record(binding),
                request_id,
                tenant_id,
                region,
            )
        });

    idempotency_ledger.entries.insert(
        key,
        CloudCellBindIdempotencyLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

fn validate_boundary(boundary: &CloudCellApiBoundaryContext) -> Result<(), CloudCellApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(CloudCellApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(CloudCellApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(CloudCellApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_tenant_id(path_tenant_id: &str) -> Result<(), CloudCellApiError> {
    if path_tenant_id.trim().is_empty() {
        Err(CloudCellApiError::EmptyPathTenantId)
    } else {
        Ok(())
    }
}

fn validate_tenant_binding(
    boundary: &CloudCellApiBoundaryContext,
    path_tenant_id: &str,
    principal: &CloudCellApiPrincipal,
    body_tenant_id: &str,
) -> Result<(), CloudCellApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(CloudCellApiError::EmptyPrincipalId);
    }
    if boundary.tenant_id != path_tenant_id
        || boundary.tenant_id != principal.tenant_id
        || boundary.tenant_id != body_tenant_id
    {
        return Err(CloudCellApiError::TenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            path_tenant_id: path_tenant_id.to_string(),
            principal_tenant_id: principal.tenant_id.clone(),
            body_tenant_id: body_tenant_id.to_string(),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &CloudCellApiPrincipal,
    authorization: &CloudCellApiAuthorization,
    surface: &str,
) -> Result<(), CloudCellApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(CloudCellApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(CloudCellApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(CloudCellApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed_surface| allowed_surface == surface)
    {
        return Err(CloudCellApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn validate_body_labels(body: &CloudCellBindRequest) -> Result<(), CloudCellApiError> {
    parse_api_residency_class(body.residency_class.clone())?;
    if let Some(required_density) = &body.required_density {
        parse_tenant_density_label(required_density.clone())?;
    }
    Ok(())
}

fn route_request_input(
    input: CloudCellBindRequest,
) -> Result<TenantCellRouteRequest, CloudCellApiError> {
    Ok(TenantCellRouteRequest {
        tenant_id: input.tenant_id,
        home_region_code: input.home_region_code,
        residency_class: parse_api_residency_class(input.residency_class)?,
        required_density: input
            .required_density
            .map(parse_tenant_density_label)
            .transpose()?,
    })
}

fn parse_api_residency_class(label: String) -> Result<ResidencyClass, CloudCellApiError> {
    parse_residency_class_label(&label).ok_or(CloudCellApiError::InvalidResidencyClassLabel {
        residency_class: label,
    })
}

fn parse_tenant_density_label(label: String) -> Result<TenantDensityClass, CloudCellApiError> {
    match label.as_str() {
        "shared" => Ok(TenantDensityClass::Shared),
        "dedicated" => Ok(TenantDensityClass::Dedicated),
        "sovereign" => Ok(TenantDensityClass::Sovereign),
        "air_gapped" => Ok(TenantDensityClass::AirGapped),
        "foundry_runtime" => Ok(TenantDensityClass::FoundryRuntime),
        _ => Err(CloudCellApiError::InvalidTenantDensityLabel {
            required_density: label,
        }),
    }
}

fn idempotency_key_for(
    boundary: &CloudCellApiBoundaryContext,
    principal: &CloudCellApiPrincipal,
    surface: &str,
) -> CloudCellBindIdempotencyLedgerKey {
    CloudCellBindIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn cell_bind_fingerprint_for(request: &CloudCellBindApiRequest) -> CloudCellBindRequestFingerprint {
    CloudCellBindRequestFingerprint {
        canonical: [
            format!("path.tenant_id={}", request.path_tenant_id),
            format!("header.tenant_id={}", request.boundary.tenant_id),
            format!("principal.tenant_id={}", request.principal.tenant_id),
            format!("principal.principal_id={}", request.principal.principal_id),
            format!(
                "authorization.tenant_id={}",
                request.authorization.tenant_id
            ),
            format!(
                "authorization.principal_id={}",
                request.authorization.principal_id
            ),
            format!(
                "authorization.decision_id={}",
                request.authorization.decision_id
            ),
            format!(
                "authorization.allowed_surfaces={}",
                request.authorization.allowed_surfaces.join(",")
            ),
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.home_region_code={}", request.body.home_region_code),
            format!("body.residency_class={}", request.body.residency_class),
            format!("body.required_density={:?}", request.body.required_density),
        ]
        .join("|"),
    }
}

fn cell_binding_record(binding: CellBinding) -> CloudCellBindingRecord {
    CloudCellBindingRecord {
        tenant_id: binding.tenant_id,
        region: binding.region,
        residency_class: residency_class_label(&binding.residency_class.value).to_string(),
        az: binding.az.value,
        cell_id: binding.cell_id.value,
        tier: cell_tier_label(binding.tier.value).to_string(),
        hsm_partition_ref: binding.hsm_partition_ref.value,
        schema_version: binding.schema_version.value,
    }
}

fn residency_class_label(residency_class: &ResidencyClass) -> &'static str {
    residency_class.label().unwrap_or("per_pack")
}

fn cell_tier_label(tier: CellTier) -> &'static str {
    match tier {
        CellTier::Shared => "shared",
        CellTier::Pooled => "pooled",
        CellTier::Dedicated => "dedicated",
        CellTier::SovereignAirGapped => "sovereign_air_gapped",
        CellTier::FoundryRuntime => "foundry_runtime",
    }
}

fn cloud_region_status_kind(error: &CloudRegionError) -> CloudCellApiStatusKind {
    match error {
        CloudRegionError::DuplicateRegion
        | CloudRegionError::DuplicateAz
        | CloudRegionError::DuplicateCell
        | CloudRegionError::CellBindingRejected(CellError::AlreadyBound) => {
            CloudCellApiStatusKind::Conflict
        }
        CloudRegionError::UnknownRegion
        | CloudRegionError::UnknownAz
        | CloudRegionError::UnknownCell => CloudCellApiStatusKind::NotFound,
        CloudRegionError::RegionResidencyMismatch
        | CloudRegionError::CellResidencyNotAllowedInRegion
        | CloudRegionError::CellResidencyDenied
        | CloudRegionError::CellBindingRejected(CellError::ResidencyRegionMismatch) => {
            CloudCellApiStatusKind::Forbidden
        }
        CloudRegionError::NoCompatibleCell => CloudCellApiStatusKind::UnprocessableEntity,
        CloudRegionError::InvalidRegionCode
        | CloudRegionError::InvalidAzCode
        | CloudRegionError::InvalidCellId
        | CloudRegionError::InvalidDisplayName
        | CloudRegionError::InvalidRegulatoryPack
        | CloudRegionError::EmptyRegulatoryPackSet
        | CloudRegionError::DuplicateRegulatoryPack
        | CloudRegionError::InvalidPhysicalRef
        | CloudRegionError::InvalidPowerZone
        | CloudRegionError::EmptyPowerZoneSet
        | CloudRegionError::DuplicatePowerZone
        | CloudRegionError::InvalidHsmPartitionRef
        | CloudRegionError::InvalidTenantId
        | CloudRegionError::InvalidCapacity
        | CloudRegionError::UtilizationExceedsCapacity
        | CloudRegionError::EmptyAllowedResidencySet
        | CloudRegionError::DuplicateAllowedResidencyClass
        | CloudRegionError::AzRegionMismatch
        | CloudRegionError::CellRegionMismatch
        | CloudRegionError::CellAzMismatch
        | CloudRegionError::ResidencyReferenceRejected(_)
        | CloudRegionError::CellBindingRejected(
            CellError::InvalidTenantId
            | CellError::EmptyAz
            | CellError::EmptyCell
            | CellError::EmptyHsmPartition
            | CellError::AzRegionMismatch,
        ) => CloudCellApiStatusKind::BadRequest,
    }
}

fn cloud_region_message(error: &CloudRegionError) -> &'static str {
    match cloud_region_status_kind(error) {
        CloudCellApiStatusKind::BadRequest => "Cloud Cell rejected the request shape",
        CloudCellApiStatusKind::Unauthorized => "Cloud Cell authentication is required",
        CloudCellApiStatusKind::Forbidden => "Cloud Cell policy denied the requested binding",
        CloudCellApiStatusKind::NotFound => {
            "Cloud Cell region, availability zone, or cell was not found"
        }
        CloudCellApiStatusKind::Conflict => "Cloud Cell binding already exists",
        CloudCellApiStatusKind::UnprocessableEntity => {
            "Cloud Cell could not find an active compatible cell"
        }
    }
}

fn cloud_region_issue(error: &CloudRegionError) -> &'static str {
    match error {
        CloudRegionError::InvalidRegionCode => "region code must be canonical",
        CloudRegionError::InvalidAzCode => "availability-zone code must be canonical",
        CloudRegionError::InvalidCellId => "cell id must be canonical and cell-prefixed",
        CloudRegionError::InvalidDisplayName => "region display name must be non-empty",
        CloudRegionError::InvalidRegulatoryPack => "regulatory pack must be canonical",
        CloudRegionError::EmptyRegulatoryPackSet => "at least one regulatory pack is required",
        CloudRegionError::DuplicateRegulatoryPack => "regulatory packs must be unique",
        CloudRegionError::InvalidPhysicalRef => "physical reference must be non-empty",
        CloudRegionError::InvalidPowerZone => "power zone must be valid",
        CloudRegionError::EmptyPowerZoneSet => "at least one power zone is required",
        CloudRegionError::DuplicatePowerZone => "power zones must be unique",
        CloudRegionError::InvalidHsmPartitionRef => {
            "HSM partition reference must match the selected region and cell"
        }
        CloudRegionError::InvalidTenantId => "tenant id must use the ten_ prefix",
        CloudRegionError::InvalidCapacity => "cell capacity must be positive",
        CloudRegionError::UtilizationExceedsCapacity => "cell utilization cannot exceed capacity",
        CloudRegionError::RegionResidencyMismatch => {
            "requested residency class is not allowed in the home region"
        }
        CloudRegionError::EmptyAllowedResidencySet => {
            "cell must allow at least one residency class"
        }
        CloudRegionError::DuplicateAllowedResidencyClass => {
            "cell allowed residency classes must be unique"
        }
        CloudRegionError::CellResidencyNotAllowedInRegion => {
            "cell residency class must be allowed by its region"
        }
        CloudRegionError::CellResidencyDenied => {
            "requested residency class is not allowed by the selected cell"
        }
        CloudRegionError::DuplicateRegion => "region already exists",
        CloudRegionError::DuplicateAz => "availability zone already exists",
        CloudRegionError::DuplicateCell => "cell already exists",
        CloudRegionError::UnknownRegion => "region was not found",
        CloudRegionError::UnknownAz => "availability zone was not found",
        CloudRegionError::UnknownCell => "cell was not found",
        CloudRegionError::AzRegionMismatch => {
            "availability-zone code must be within its region namespace"
        }
        CloudRegionError::CellRegionMismatch => "cell region must match selected region",
        CloudRegionError::CellAzMismatch => {
            "cell availability zone must match selected availability zone"
        }
        CloudRegionError::NoCompatibleCell => "no active compatible cell has routing capacity",
        CloudRegionError::CellBindingRejected(error) => cell_issue(error),
        CloudRegionError::ResidencyReferenceRejected(_) => {
            "region residency reference was rejected"
        }
    }
}

fn cell_issue(error: &CellError) -> &'static str {
    match error {
        CellError::AlreadyBound => "tenant already has an immutable cell binding",
        CellError::InvalidTenantId => "tenant id must use the ten_ prefix",
        CellError::EmptyAz => "availability zone is required",
        CellError::EmptyCell => "cell id is required",
        CellError::EmptyHsmPartition => "HSM partition reference is required",
        CellError::AzRegionMismatch => "availability zone must belong to the selected region",
        CellError::ResidencyRegionMismatch => {
            "residency class is not allowed in the selected region"
        }
    }
}

fn detail(field: impl Into<String>, issue: impl Into<String>) -> CloudCellApiErrorDetail {
    CloudCellApiErrorDetail {
        field: field.into(),
        issue: issue.into(),
    }
}
