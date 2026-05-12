//! Platform Tenant API boundary.
//!
//! This crate owns authenticated REST-boundary normalization, path/body tenant
//! binding, request fingerprint idempotency, and global tenant-id uniqueness for
//! `tenant.create` before handing typed construction to the platform tenant
//! kernel.

use std::collections::BTreeMap;

use oya_platform_residency_kernel::parse_residency_class_label;
use oya_platform_tenant_kernel::{Tenant, TenantError};

pub const TENANT_CREATE_SURFACE: &str = "tenant.create";
pub const TENANT_CREATE_OPENAPI_CONTRACT: &str =
    "contracts/openapi/platform/platform-tenant-v1.yaml";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TenantCreateApiStatus {
    Created,
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

impl TenantCreateApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Created => 201,
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::Conflict => 409,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TenantCreateApiErrorCode {
    RequestIdEmpty,
    OperatorTenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    PathTenantIdEmpty,
    TenantPathBodyMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    ResidencyClassInvalid,
    DuplicateTenant,
    IdempotencyKeyReused,
    TenantInvalidTenantId,
    TenantLegalNameEmpty,
    TenantHomeRegionEmpty,
    TenantHomeRegionDenied,
    TenantRegionalPackMissing,
}

impl TenantCreateApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "TENANT_CREATE_REQUEST_ID_EMPTY",
            Self::OperatorTenantHeaderEmpty => "TENANT_CREATE_OPERATOR_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "TENANT_CREATE_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "TENANT_CREATE_PRINCIPAL_ID_EMPTY",
            Self::PathTenantIdEmpty => "TENANT_CREATE_PATH_TENANT_ID_EMPTY",
            Self::TenantPathBodyMismatch => "TENANT_CREATE_PATH_BODY_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => "TENANT_CREATE_AUTHORIZATION_DECISION_ID_EMPTY",
            Self::AuthorizationTenantMismatch => "TENANT_CREATE_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => {
                "TENANT_CREATE_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationDenied => "TENANT_CREATE_AUTHORIZATION_DENIED",
            Self::ResidencyClassInvalid => "TENANT_CREATE_RESIDENCY_CLASS_INVALID",
            Self::DuplicateTenant => "TENANT_CREATE_DUPLICATE_TENANT",
            Self::IdempotencyKeyReused => "TENANT_CREATE_IDEMPOTENCY_KEY_REUSED",
            Self::TenantInvalidTenantId => "TENANT_CREATE_TENANT_INVALID_ID",
            Self::TenantLegalNameEmpty => "TENANT_CREATE_LEGAL_NAME_EMPTY",
            Self::TenantHomeRegionEmpty => "TENANT_CREATE_HOME_REGION_EMPTY",
            Self::TenantHomeRegionDenied => "TENANT_CREATE_HOME_REGION_DENIED",
            Self::TenantRegionalPackMissing => "TENANT_CREATE_REGIONAL_PACK_MISSING",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRegulatoryPackRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCreateRequest {
    pub tenant_id: String,                              // data_class: INTERNAL_ONLY
    pub legal_name: String,                             // data_class: INTERNAL_ONLY
    pub home_region: String,                            // data_class: INTERNAL_ONLY
    pub residency_class: String,                        // data_class: INTERNAL_ONLY
    pub regulatory_packs: Vec<TenantRegulatoryPackRef>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCreateApiRequest {
    pub path_tenant_id: String,                // data_class: INTERNAL_ONLY
    pub boundary: TenantApiBoundaryContext,    // data_class: INTERNAL_ONLY
    pub principal: TenantApiPrincipal,         // data_class: INTERNAL_ONLY
    pub authorization: TenantApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: TenantCreateRequest,             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TenantDirectory {
    tenants: BTreeMap<String, Tenant>, // data_class: INTERNAL_ONLY
}

impl TenantDirectory {
    pub fn len(&self) -> usize {
        self.tenants.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tenants.is_empty()
    }

    pub fn get(&self, tenant_id: &str) -> Option<&Tenant> {
        self.tenants.get(tenant_id)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TenantCreateIdempotencyLedger {
    entries: BTreeMap<TenantCreateIdempotencyLedgerKey, TenantCreateIdempotencyLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl TenantCreateIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct TenantCreateIdempotencyLedgerKey {
    operator_tenant_id: String, // data_class: INTERNAL_ONLY
    principal_id: String,       // data_class: INTERNAL_ONLY
    surface: String,            // data_class: INTERNAL_ONLY
    idempotency_key: String,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TenantCreateIdempotencyLedgerEntry {
    fingerprint: TenantCreateRequestFingerprint, // data_class: INTERNAL_ONLY
    result: TenantCreateSuccessResponse,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TenantCreateRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCreateSuccessResponse {
    pub data: TenantRecord,             // data_class: INTERNAL_ONLY
    pub metadata: TenantCreateMetadata, // data_class: INTERNAL_ONLY
}

impl TenantCreateSuccessResponse {
    pub fn created(data: TenantRecord, request: &TenantCreateApiRequest) -> Self {
        Self {
            data,
            metadata: TenantCreateMetadata {
                request_id: request.boundary.request_id.clone(),
                operator_tenant_id: request.boundary.tenant_id.clone(),
                principal_id: request.principal.principal_id.clone(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCreateMetadata {
    pub request_id: String,         // data_class: INTERNAL_ONLY
    pub operator_tenant_id: String, // data_class: INTERNAL_ONLY
    pub principal_id: String,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRecord {
    pub tenant_id: String,                              // data_class: INTERNAL_ONLY
    pub legal_name: String,                             // data_class: INTERNAL_ONLY
    pub home_region: String,                            // data_class: INTERNAL_ONLY
    pub residency_class: String,                        // data_class: INTERNAL_ONLY
    pub regulatory_packs: Vec<TenantRegulatoryPackRef>, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                            // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCreateApiErrorResponse {
    pub error: TenantCreateApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCreateApiErrorBody {
    pub code: String,                             // data_class: INTERNAL_ONLY
    pub message: String,                          // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,        // data_class: INTERNAL_ONLY
    pub request_id: String,                       // data_class: INTERNAL_ONLY
    pub details: Vec<TenantCreateApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCreateApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantCreateApiError {
    EmptyRequestId,
    EmptyOperatorTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    EmptyPathTenantId,
    TenantPathBodyMismatch {
        path_tenant_id: String,
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
    InvalidResidencyClass {
        residency_class: String,
    },
    DuplicateTenant {
        tenant_id: String,
    },
    IdempotencyKeyReused {
        idempotency_key: String,
    },
    Tenant(TenantError),
}

impl TenantCreateApiError {
    pub fn tenant_create_status(&self) -> TenantCreateApiStatus {
        match self.status_kind() {
            TenantCreateApiStatusKind::BadRequest => TenantCreateApiStatus::BadRequest,
            TenantCreateApiStatusKind::Unauthorized => TenantCreateApiStatus::Unauthorized,
            TenantCreateApiStatusKind::Forbidden => TenantCreateApiStatus::Forbidden,
            TenantCreateApiStatusKind::Conflict => TenantCreateApiStatus::Conflict,
            TenantCreateApiStatusKind::UnprocessableEntity => {
                TenantCreateApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn tenant_create_status_code(&self) -> u16 {
        self.tenant_create_status().code()
    }

    pub fn code(&self) -> TenantCreateApiErrorCode {
        match self {
            Self::EmptyRequestId => TenantCreateApiErrorCode::RequestIdEmpty,
            Self::EmptyOperatorTenantHeader => TenantCreateApiErrorCode::OperatorTenantHeaderEmpty,
            Self::EmptyIdempotencyKey => TenantCreateApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => TenantCreateApiErrorCode::PrincipalIdEmpty,
            Self::EmptyPathTenantId => TenantCreateApiErrorCode::PathTenantIdEmpty,
            Self::TenantPathBodyMismatch { .. } => TenantCreateApiErrorCode::TenantPathBodyMismatch,
            Self::EmptyAuthorizationDecisionId => {
                TenantCreateApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                TenantCreateApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                TenantCreateApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => TenantCreateApiErrorCode::AuthorizationDenied,
            Self::InvalidResidencyClass { .. } => TenantCreateApiErrorCode::ResidencyClassInvalid,
            Self::DuplicateTenant { .. } => TenantCreateApiErrorCode::DuplicateTenant,
            Self::IdempotencyKeyReused { .. } => TenantCreateApiErrorCode::IdempotencyKeyReused,
            Self::Tenant(error) => tenant_error_code(error),
        }
    }

    pub fn error_response(&self, request_id: impl Into<String>) -> TenantCreateApiErrorResponse {
        TenantCreateApiErrorResponse {
            error: TenantCreateApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> TenantCreateApiStatusKind {
        match self {
            Self::EmptyPrincipalId => TenantCreateApiStatusKind::Unauthorized,
            Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. } => TenantCreateApiStatusKind::Forbidden,
            Self::DuplicateTenant { .. } => TenantCreateApiStatusKind::Conflict,
            Self::IdempotencyKeyReused { .. } => TenantCreateApiStatusKind::UnprocessableEntity,
            Self::EmptyRequestId
            | Self::EmptyOperatorTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::EmptyPathTenantId
            | Self::TenantPathBodyMismatch { .. }
            | Self::InvalidResidencyClass { .. }
            | Self::Tenant(_) => TenantCreateApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyOperatorTenantHeader => "X-Tenant-Id operator header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::EmptyPathTenantId => "Path tenant id is required",
            Self::TenantPathBodyMismatch { .. } => {
                "Path tenant id must match request body tenant_id"
            }
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal id"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the requested tenant creation surface"
            }
            Self::InvalidResidencyClass { .. } => {
                "Request residency_class must be a supported residency class label"
            }
            Self::DuplicateTenant { .. } => "Tenant id already exists",
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::Tenant(error) => tenant_error_message(error),
        }
    }

    fn details(&self) -> Vec<TenantCreateApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyOperatorTenantHeader => {
                vec![detail("header.X-Tenant-Id", "must be non-empty")]
            }
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::EmptyPathTenantId => vec![detail("path.tenant_id", "must be non-empty")],
            Self::TenantPathBodyMismatch { .. } => vec![detail(
                "body.tenant_id",
                "must match the tenant_id path parameter",
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
                "must include the requested tenant.create surface",
            )],
            Self::InvalidResidencyClass { .. } => vec![detail(
                "body.residency_class",
                "must be one of strict_kr, kr_with_us_failover, or global",
            )],
            Self::DuplicateTenant { .. } => {
                vec![detail("body.tenant_id", "must be globally unique")]
            }
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::Tenant(error) => vec![detail("tenant_kernel", tenant_error_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TenantCreateApiStatusKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

pub fn validate_tenant_create_request(
    request: &TenantCreateApiRequest,
) -> Result<(), TenantCreateApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_body_binding(&request.path_tenant_id, &request.body.tenant_id)?;
    validate_operator_binding(&request.boundary, &request.principal)?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        TENANT_CREATE_SURFACE,
    )?;
    parse_api_residency_class(&request.body.residency_class)?;
    Ok(())
}

pub fn create_tenant_from_api(
    directory: &mut TenantDirectory,
    idempotency_ledger: &mut TenantCreateIdempotencyLedger,
    request: TenantCreateApiRequest,
) -> Result<TenantCreateSuccessResponse, TenantCreateApiError> {
    validate_tenant_create_request(&request)?;
    let key = idempotency_key_for(&request.boundary, &request.principal, TENANT_CREATE_SURFACE);
    let fingerprint = tenant_create_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return Ok(entry.result.clone());
        }
        return Err(TenantCreateApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }
    if directory.tenants.contains_key(&request.body.tenant_id) {
        return Err(TenantCreateApiError::DuplicateTenant {
            tenant_id: request.body.tenant_id,
        });
    }

    let tenant = tenant_from_request(&request.body)?;
    let response = TenantCreateSuccessResponse::created(tenant_record(&tenant), &request);
    directory.tenants.insert(tenant.id.clone(), tenant);
    idempotency_ledger.entries.insert(
        key,
        TenantCreateIdempotencyLedgerEntry {
            fingerprint,
            result: response.clone(),
        },
    );
    Ok(response)
}

fn validate_boundary(boundary: &TenantApiBoundaryContext) -> Result<(), TenantCreateApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(TenantCreateApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(TenantCreateApiError::EmptyOperatorTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(TenantCreateApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_body_binding(
    path_tenant_id: &str,
    body_tenant_id: &str,
) -> Result<(), TenantCreateApiError> {
    if path_tenant_id.trim().is_empty() {
        return Err(TenantCreateApiError::EmptyPathTenantId);
    }
    if path_tenant_id != body_tenant_id {
        return Err(TenantCreateApiError::TenantPathBodyMismatch {
            path_tenant_id: path_tenant_id.to_string(),
            body_tenant_id: body_tenant_id.to_string(),
        });
    }
    Ok(())
}

fn validate_operator_binding(
    boundary: &TenantApiBoundaryContext,
    principal: &TenantApiPrincipal,
) -> Result<(), TenantCreateApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(TenantCreateApiError::EmptyPrincipalId);
    }
    if boundary.tenant_id != principal.tenant_id {
        return Err(TenantCreateApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &TenantApiPrincipal,
    authorization: &TenantApiAuthorization,
    surface: &str,
) -> Result<(), TenantCreateApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(TenantCreateApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(TenantCreateApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(TenantCreateApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed_surface| allowed_surface == surface)
    {
        return Err(TenantCreateApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn tenant_from_request(body: &TenantCreateRequest) -> Result<Tenant, TenantCreateApiError> {
    Tenant::new(
        body.tenant_id.clone(),
        body.legal_name.clone(),
        body.home_region.clone(),
        parse_api_residency_class(&body.residency_class)?,
        body.regulatory_packs
            .iter()
            .map(|pack| pack.value.clone())
            .collect(),
    )
    .map_err(TenantCreateApiError::Tenant)
}

fn parse_api_residency_class(
    label: &str,
) -> Result<oya_platform_residency_kernel::ResidencyClass, TenantCreateApiError> {
    parse_residency_class_label(label).ok_or(TenantCreateApiError::InvalidResidencyClass {
        residency_class: label.to_string(),
    })
}

fn idempotency_key_for(
    boundary: &TenantApiBoundaryContext,
    principal: &TenantApiPrincipal,
    surface: &str,
) -> TenantCreateIdempotencyLedgerKey {
    TenantCreateIdempotencyLedgerKey {
        operator_tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn tenant_create_fingerprint_for(
    request: &TenantCreateApiRequest,
) -> TenantCreateRequestFingerprint {
    TenantCreateRequestFingerprint {
        canonical: [
            format!("path.tenant_id={}", request.path_tenant_id),
            format!("header.operator_tenant_id={}", request.boundary.tenant_id),
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
            format!("body.legal_name={}", request.body.legal_name),
            format!("body.home_region={}", request.body.home_region),
            format!("body.residency_class={}", request.body.residency_class),
            format!(
                "body.regulatory_packs={}",
                request
                    .body
                    .regulatory_packs
                    .iter()
                    .map(|pack| pack.value.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        ]
        .join("|"),
    }
}

fn tenant_record(tenant: &Tenant) -> TenantRecord {
    TenantRecord {
        tenant_id: tenant.id.clone(),
        legal_name: tenant.legal_name.value.clone(),
        home_region: tenant.home_region.value.clone(),
        residency_class: tenant
            .residency_class
            .value
            .label()
            .unwrap_or("per_pack")
            .to_string(),
        regulatory_packs: tenant
            .regulatory_packs
            .value
            .iter()
            .cloned()
            .map(|value| TenantRegulatoryPackRef { value })
            .collect(),
        schema_version: 1,
    }
}

fn tenant_error_code(error: &TenantError) -> TenantCreateApiErrorCode {
    match error {
        TenantError::InvalidTenantId => TenantCreateApiErrorCode::TenantInvalidTenantId,
        TenantError::EmptyLegalName => TenantCreateApiErrorCode::TenantLegalNameEmpty,
        TenantError::EmptyHomeRegion => TenantCreateApiErrorCode::TenantHomeRegionEmpty,
        TenantError::HomeRegionNotAllowedForResidency => {
            TenantCreateApiErrorCode::TenantHomeRegionDenied
        }
        TenantError::MissingRegionalPack => TenantCreateApiErrorCode::TenantRegionalPackMissing,
    }
}

fn tenant_error_message(error: &TenantError) -> &'static str {
    match error {
        TenantError::InvalidTenantId => "Tenant id must use the ten_ prefix",
        TenantError::EmptyLegalName => "Tenant legal name is required",
        TenantError::EmptyHomeRegion => "Tenant home region is required",
        TenantError::HomeRegionNotAllowedForResidency => {
            "Tenant home region is not allowed for the requested residency class"
        }
        TenantError::MissingRegionalPack => "At least one regulatory pack is required",
    }
}

fn tenant_error_issue(error: &TenantError) -> &'static str {
    match error {
        TenantError::InvalidTenantId => "tenant id must be globally canonical and ten_-prefixed",
        TenantError::EmptyLegalName => "legal_name must be non-empty",
        TenantError::EmptyHomeRegion => "home_region must be non-empty",
        TenantError::HomeRegionNotAllowedForResidency => {
            "strict KR residency classes require a kr-* home region"
        }
        TenantError::MissingRegionalPack => "regulatory_packs must contain at least one pack",
    }
}

fn detail(field: impl Into<String>, issue: impl Into<String>) -> TenantCreateApiErrorDetail {
    TenantCreateApiErrorDetail {
        field: field.into(),
        issue: issue.into(),
    }
}
