//! Platform regulatory pack binding API boundary.
//!
//! This crate owns authenticated REST-boundary normalization, path/body tenant
//! binding, request fingerprint idempotency, regional-pack shape validation,
//! tenant residency immutability, and stable public error projection for
//! `regulatory-pack.bind` before handing typed pack and residency construction to
//! the platform regional-pack and residency kernels.

use std::collections::BTreeMap;

use cell_regional_pack::{RegionalPack, RegionalPackError};
use network_residency::{
    RegionRef, RegionRefCreate, ResidencyClass, ResidencyError, TenantResidencyBindingCreate,
    TenantResidencyRegistry, infer_region_jurisdiction_label, parse_residency_class_label,
};

pub const REGULATORY_PACK_BIND_SURFACE: &str = "regulatory-pack.bind";
pub const REGULATORY_PACK_BIND_OPENAPI_CONTRACT: &str =
    "contracts/openapi/platform/platform-regulatory-pack-v1.yaml";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegulatoryPackBindApiStatus {
    Created,
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

impl RegulatoryPackBindApiStatus {
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
pub enum RegulatoryPackBindApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    PathTenantIdEmpty,
    TenantPathBodyMismatch,
    PrincipalTenantMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    PackRefsEmpty,
    PrimaryPackMissing,
    PrimaryPackRegionMismatch,
    PrimaryPackResidencyMismatch,
    ResidencyClassInvalid,
    DuplicateBinding,
    IdempotencyKeyReused,
    RegionalPackInvalidPackId,
    RegionalPackEmptyRegion,
    RegionalPackEmptyResidencyClass,
    RegionalPackInvalidResidencyClass,
    RegionalPackMissingControls,
    ResidencyInvalidTenantId,
    ResidencyInvalidRegionId,
    ResidencyInvalidCellGroupRef,
    ResidencyInvalidPackId,
    ResidencyInvalidEvidenceRef,
    ResidencyPrimaryRegionDenied,
    ResidencyAlreadyBound,
}

impl RegulatoryPackBindApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "REGULATORY_PACK_BIND_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "REGULATORY_PACK_BIND_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "REGULATORY_PACK_BIND_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "REGULATORY_PACK_BIND_PRINCIPAL_ID_EMPTY",
            Self::PathTenantIdEmpty => "REGULATORY_PACK_BIND_PATH_TENANT_ID_EMPTY",
            Self::TenantPathBodyMismatch => "REGULATORY_PACK_BIND_TENANT_PATH_BODY_MISMATCH",
            Self::PrincipalTenantMismatch => "REGULATORY_PACK_BIND_PRINCIPAL_TENANT_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => {
                "REGULATORY_PACK_BIND_AUTHORIZATION_DECISION_ID_EMPTY"
            }
            Self::AuthorizationTenantMismatch => {
                "REGULATORY_PACK_BIND_AUTHORIZATION_TENANT_MISMATCH"
            }
            Self::AuthorizationPrincipalMismatch => {
                "REGULATORY_PACK_BIND_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationDenied => "REGULATORY_PACK_BIND_AUTHORIZATION_DENIED",
            Self::PackRefsEmpty => "REGULATORY_PACK_BIND_PACK_REFS_EMPTY",
            Self::PrimaryPackMissing => "REGULATORY_PACK_BIND_PRIMARY_PACK_MISSING",
            Self::PrimaryPackRegionMismatch => "REGULATORY_PACK_BIND_PRIMARY_PACK_REGION_MISMATCH",
            Self::PrimaryPackResidencyMismatch => {
                "REGULATORY_PACK_BIND_PRIMARY_PACK_RESIDENCY_MISMATCH"
            }
            Self::ResidencyClassInvalid => "REGULATORY_PACK_BIND_RESIDENCY_CLASS_INVALID",
            Self::DuplicateBinding => "REGULATORY_PACK_BIND_DUPLICATE_BINDING",
            Self::IdempotencyKeyReused => "REGULATORY_PACK_BIND_IDEMPOTENCY_KEY_REUSED",
            Self::RegionalPackInvalidPackId => "REGULATORY_PACK_BIND_PACK_INVALID_ID",
            Self::RegionalPackEmptyRegion => "REGULATORY_PACK_BIND_PACK_EMPTY_REGION",
            Self::RegionalPackEmptyResidencyClass => {
                "REGULATORY_PACK_BIND_PACK_EMPTY_RESIDENCY_CLASS"
            }
            Self::RegionalPackInvalidResidencyClass => {
                "REGULATORY_PACK_BIND_PACK_INVALID_RESIDENCY_CLASS"
            }
            Self::RegionalPackMissingControls => "REGULATORY_PACK_BIND_PACK_MISSING_CONTROLS",
            Self::ResidencyInvalidTenantId => "REGULATORY_PACK_BIND_RESIDENCY_INVALID_TENANT_ID",
            Self::ResidencyInvalidRegionId => "REGULATORY_PACK_BIND_RESIDENCY_INVALID_REGION_ID",
            Self::ResidencyInvalidCellGroupRef => {
                "REGULATORY_PACK_BIND_RESIDENCY_INVALID_CELL_GROUP_REF"
            }
            Self::ResidencyInvalidPackId => "REGULATORY_PACK_BIND_RESIDENCY_INVALID_PACK_ID",
            Self::ResidencyInvalidEvidenceRef => {
                "REGULATORY_PACK_BIND_RESIDENCY_INVALID_EVIDENCE_REF"
            }
            Self::ResidencyPrimaryRegionDenied => {
                "REGULATORY_PACK_BIND_RESIDENCY_PRIMARY_REGION_DENIED"
            }
            Self::ResidencyAlreadyBound => "REGULATORY_PACK_BIND_RESIDENCY_ALREADY_BOUND",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegulatoryPackApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegulatoryPackApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegulatoryPackApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegulatoryPackControlRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegulatoryPackBindingPackRef {
    pub pack_id: String,                         // data_class: INTERNAL_ONLY
    pub region: String,                          // data_class: INTERNAL_ONLY
    pub residency_class: String,                 // data_class: INTERNAL_ONLY
    pub controls: Vec<RegulatoryPackControlRef>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegulatoryPackBindRequest {
    pub tenant_id: String,                            // data_class: INTERNAL_ONLY
    pub primary_pack_id: String,                      // data_class: INTERNAL_ONLY
    pub home_region: String,                          // data_class: INTERNAL_ONLY
    pub cell_group_ref: String,                       // data_class: INTERNAL_ONLY
    pub residency_class: String,                      // data_class: INTERNAL_ONLY
    pub evidence_ref: String,                         // data_class: INTERNAL_ONLY
    pub bound_at_epoch_seconds: u64,                  // data_class: INTERNAL_ONLY
    pub pack_refs: Vec<RegulatoryPackBindingPackRef>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegulatoryPackBindApiRequest {
    pub path_tenant_id: String, // data_class: INTERNAL_ONLY
    pub boundary: RegulatoryPackApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: RegulatoryPackApiPrincipal, // data_class: INTERNAL_ONLY
    pub authorization: RegulatoryPackApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: RegulatoryPackBindRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RegulatoryPackBindingDirectory {
    residency_registry: TenantResidencyRegistry, // data_class: INTERNAL_ONLY
    bindings: BTreeMap<String, RegulatoryPackBindingRecord>, // data_class: INTERNAL_ONLY
}

impl RegulatoryPackBindingDirectory {
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    pub fn get(&self, tenant_id: &str) -> Option<&RegulatoryPackBindingRecord> {
        self.bindings.get(tenant_id)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RegulatoryPackBindIdempotencyLedger {
    entries:
        BTreeMap<RegulatoryPackBindIdempotencyLedgerKey, RegulatoryPackBindIdempotencyLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl RegulatoryPackBindIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct RegulatoryPackBindIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RegulatoryPackBindIdempotencyLedgerEntry {
    fingerprint: RegulatoryPackBindRequestFingerprint, // data_class: INTERNAL_ONLY
    result: RegulatoryPackBindSuccessResponse,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RegulatoryPackBindRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegulatoryPackBindSuccessResponse {
    pub data: RegulatoryPackBindingRecord, // data_class: INTERNAL_ONLY
    pub metadata: RegulatoryPackBindMetadata, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegulatoryPackBindMetadata {
    pub request_id: String,   // data_class: INTERNAL_ONLY
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegulatoryPackBindingRecord {
    pub tenant_id: String,                            // data_class: INTERNAL_ONLY
    pub primary_pack_id: String,                      // data_class: INTERNAL_ONLY
    pub home_region: String,                          // data_class: INTERNAL_ONLY
    pub cell_group_ref: String,                       // data_class: INTERNAL_ONLY
    pub residency_class: String,                      // data_class: INTERNAL_ONLY
    pub evidence_ref: String,                         // data_class: INTERNAL_ONLY
    pub bound_at_epoch_seconds: u64,                  // data_class: INTERNAL_ONLY
    pub pack_refs: Vec<RegulatoryPackBindingPackRef>, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegulatoryPackBindApiErrorResponse {
    pub error: RegulatoryPackBindApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegulatoryPackBindApiErrorBody {
    pub code: String,                                   // data_class: INTERNAL_ONLY
    pub message: String,                                // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,              // data_class: INTERNAL_ONLY
    pub request_id: String,                             // data_class: INTERNAL_ONLY
    pub details: Vec<RegulatoryPackBindApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegulatoryPackBindApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegulatoryPackBindApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    EmptyPathTenantId,
    TenantPathBodyMismatch {
        path_tenant_id: String,
        body_tenant_id: String,
    },
    PrincipalTenantMismatch {
        header_tenant_id: String,
        principal_tenant_id: String,
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
    EmptyPackRefs,
    PrimaryPackMissing {
        primary_pack_id: String,
    },
    PrimaryPackRegionMismatch {
        primary_pack_id: String,
        home_region: String,
        pack_region: String,
    },
    PrimaryPackResidencyMismatch {
        primary_pack_id: String,
        residency_class: String,
        pack_residency_class: String,
    },
    InvalidResidencyClass {
        residency_class: String,
    },
    DuplicateBinding {
        tenant_id: String,
    },
    IdempotencyKeyReused {
        idempotency_key: String,
    },
    RegionalPack(RegionalPackError),
    Residency(ResidencyError),
}

impl RegulatoryPackBindApiError {
    pub fn regulatory_pack_bind_status(&self) -> RegulatoryPackBindApiStatus {
        match self.status_kind() {
            RegulatoryPackBindApiStatusKind::BadRequest => RegulatoryPackBindApiStatus::BadRequest,
            RegulatoryPackBindApiStatusKind::Unauthorized => {
                RegulatoryPackBindApiStatus::Unauthorized
            }
            RegulatoryPackBindApiStatusKind::Forbidden => RegulatoryPackBindApiStatus::Forbidden,
            RegulatoryPackBindApiStatusKind::Conflict => RegulatoryPackBindApiStatus::Conflict,
            RegulatoryPackBindApiStatusKind::UnprocessableEntity => {
                RegulatoryPackBindApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn regulatory_pack_bind_status_code(&self) -> u16 {
        self.regulatory_pack_bind_status().code()
    }

    pub fn code(&self) -> RegulatoryPackBindApiErrorCode {
        match self {
            Self::EmptyRequestId => RegulatoryPackBindApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => RegulatoryPackBindApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => RegulatoryPackBindApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => RegulatoryPackBindApiErrorCode::PrincipalIdEmpty,
            Self::EmptyPathTenantId => RegulatoryPackBindApiErrorCode::PathTenantIdEmpty,
            Self::TenantPathBodyMismatch { .. } => {
                RegulatoryPackBindApiErrorCode::TenantPathBodyMismatch
            }
            Self::PrincipalTenantMismatch { .. } => {
                RegulatoryPackBindApiErrorCode::PrincipalTenantMismatch
            }
            Self::EmptyAuthorizationDecisionId => {
                RegulatoryPackBindApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                RegulatoryPackBindApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                RegulatoryPackBindApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => RegulatoryPackBindApiErrorCode::AuthorizationDenied,
            Self::EmptyPackRefs => RegulatoryPackBindApiErrorCode::PackRefsEmpty,
            Self::PrimaryPackMissing { .. } => RegulatoryPackBindApiErrorCode::PrimaryPackMissing,
            Self::PrimaryPackRegionMismatch { .. } => {
                RegulatoryPackBindApiErrorCode::PrimaryPackRegionMismatch
            }
            Self::PrimaryPackResidencyMismatch { .. } => {
                RegulatoryPackBindApiErrorCode::PrimaryPackResidencyMismatch
            }
            Self::InvalidResidencyClass { .. } => {
                RegulatoryPackBindApiErrorCode::ResidencyClassInvalid
            }
            Self::DuplicateBinding { .. } => RegulatoryPackBindApiErrorCode::DuplicateBinding,
            Self::IdempotencyKeyReused { .. } => {
                RegulatoryPackBindApiErrorCode::IdempotencyKeyReused
            }
            Self::RegionalPack(error) => regional_pack_error_code(error),
            Self::Residency(error) => residency_error_code(error),
        }
    }

    pub fn error_response(
        &self,
        request_id: impl Into<String>,
    ) -> RegulatoryPackBindApiErrorResponse {
        RegulatoryPackBindApiErrorResponse {
            error: RegulatoryPackBindApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> RegulatoryPackBindApiStatusKind {
        match self {
            Self::EmptyPrincipalId => RegulatoryPackBindApiStatusKind::Unauthorized,
            Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::PrincipalTenantMismatch { .. }
            | Self::AuthorizationDenied { .. } => RegulatoryPackBindApiStatusKind::Forbidden,
            Self::DuplicateBinding { .. }
            | Self::Residency(ResidencyError::ResidencyAlreadyBound) => {
                RegulatoryPackBindApiStatusKind::Conflict
            }
            Self::IdempotencyKeyReused { .. } => {
                RegulatoryPackBindApiStatusKind::UnprocessableEntity
            }
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::EmptyPathTenantId
            | Self::TenantPathBodyMismatch { .. }
            | Self::EmptyPackRefs
            | Self::PrimaryPackMissing { .. }
            | Self::PrimaryPackRegionMismatch { .. }
            | Self::PrimaryPackResidencyMismatch { .. }
            | Self::InvalidResidencyClass { .. }
            | Self::RegionalPack(_)
            | Self::Residency(_) => RegulatoryPackBindApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::EmptyPathTenantId => "Path tenant id is required",
            Self::TenantPathBodyMismatch { .. } => {
                "Path tenant id must match request body tenant_id"
            }
            Self::PrincipalTenantMismatch { .. } => {
                "Authenticated principal tenant must match X-Tenant-Id"
            }
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal id"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the requested regulatory pack binding surface"
            }
            Self::EmptyPackRefs => "At least one regional pack reference is required",
            Self::PrimaryPackMissing { .. } => "Primary pack id must be present in pack_refs",
            Self::PrimaryPackRegionMismatch { .. } => {
                "Primary pack region must match the tenant home_region"
            }
            Self::PrimaryPackResidencyMismatch { .. } => {
                "Primary pack residency class must match the tenant residency_class"
            }
            Self::InvalidResidencyClass { .. } => {
                "Request residency_class must be a supported residency class label"
            }
            Self::DuplicateBinding { .. } => {
                "Tenant already has an immutable regulatory pack residency binding"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::RegionalPack(error) => regional_pack_error_message(error),
            Self::Residency(error) => residency_error_message(error),
        }
    }

    fn details(&self) -> Vec<RegulatoryPackBindApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::EmptyPathTenantId => vec![detail("path.tenant_id", "must be non-empty")],
            Self::TenantPathBodyMismatch { .. } => vec![detail(
                "body.tenant_id",
                "must match the tenant_id path parameter",
            )],
            Self::PrincipalTenantMismatch { .. } => vec![detail(
                "principal.tenant_id",
                "must match X-Tenant-Id for this tenant-scoped bind",
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
                "must include the regulatory-pack.bind surface",
            )],
            Self::EmptyPackRefs => vec![detail("body.pack_refs", "must contain at least one pack")],
            Self::PrimaryPackMissing { .. } => vec![detail(
                "body.primary_pack_id",
                "must match one pack_refs[].pack_id value",
            )],
            Self::PrimaryPackRegionMismatch { .. } => vec![detail(
                "body.pack_refs[].region",
                "primary pack region must match body.home_region",
            )],
            Self::PrimaryPackResidencyMismatch { .. } => vec![detail(
                "body.pack_refs[].residency_class",
                "primary pack residency_class must match body.residency_class",
            )],
            Self::InvalidResidencyClass { .. } => vec![detail(
                "body.residency_class",
                "must be one of strict_home_region, home_with_recovery_failover, or global",
            )],
            Self::DuplicateBinding { .. } => vec![detail(
                "body.tenant_id",
                "tenant residency and regulatory pack binding is immutable post-bind",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::RegionalPack(error) => vec![detail(
                "regional_pack_kernel",
                regional_pack_error_issue(error),
            )],
            Self::Residency(error) => {
                vec![detail("residency_kernel", residency_error_issue(error))]
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RegulatoryPackBindApiStatusKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

pub fn validate_regulatory_pack_bind_request(
    request: &RegulatoryPackBindApiRequest,
) -> Result<(), RegulatoryPackBindApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_body_binding(&request.path_tenant_id, &request.body.tenant_id)?;
    validate_principal_binding(&request.boundary, &request.principal)?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        REGULATORY_PACK_BIND_SURFACE,
    )?;
    parse_api_residency_class(&request.body.residency_class)?;
    validate_primary_pack_binding(&request.body)?;
    Ok(())
}

pub fn bind_regulatory_pack_from_api(
    directory: &mut RegulatoryPackBindingDirectory,
    idempotency_ledger: &mut RegulatoryPackBindIdempotencyLedger,
    request: RegulatoryPackBindApiRequest,
) -> Result<RegulatoryPackBindSuccessResponse, RegulatoryPackBindApiError> {
    validate_regulatory_pack_bind_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        REGULATORY_PACK_BIND_SURFACE,
    );
    let fingerprint = regulatory_pack_bind_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return Ok(entry.result.clone());
        }
        return Err(RegulatoryPackBindApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }
    if directory.bindings.contains_key(&request.body.tenant_id) {
        return Err(RegulatoryPackBindApiError::DuplicateBinding {
            tenant_id: request.body.tenant_id,
        });
    }

    let packs = regional_packs_from_request(&request.body)?;
    let residency_class = parse_api_residency_class(&request.body.residency_class)?;
    let primary_region = region_ref_from_request(&request.body)?;
    let binding = directory
        .residency_registry
        .bind(TenantResidencyBindingCreate {
            tenant_id: request.body.tenant_id.clone(),
            primary_region,
            residency_class,
            regional_pack_id: request.body.primary_pack_id.clone(),
            evidence_ref: request.body.evidence_ref.clone(),
            bound_at_epoch_seconds: request.body.bound_at_epoch_seconds,
        })
        .map_err(RegulatoryPackBindApiError::Residency)?;
    let record = binding_record_from_request(&request.body, &packs, binding.schema_version.value);
    let response = RegulatoryPackBindSuccessResponse {
        data: record.clone(),
        metadata: RegulatoryPackBindMetadata {
            request_id: request.boundary.request_id.clone(),
            tenant_id: request.boundary.tenant_id.clone(),
            principal_id: request.principal.principal_id.clone(),
        },
    };
    directory
        .bindings
        .insert(request.body.tenant_id.clone(), record);
    idempotency_ledger.entries.insert(
        key,
        RegulatoryPackBindIdempotencyLedgerEntry {
            fingerprint,
            result: response.clone(),
        },
    );
    Ok(response)
}

fn validate_boundary(
    boundary: &RegulatoryPackApiBoundaryContext,
) -> Result<(), RegulatoryPackBindApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(RegulatoryPackBindApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(RegulatoryPackBindApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(RegulatoryPackBindApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_body_binding(
    path_tenant_id: &str,
    body_tenant_id: &str,
) -> Result<(), RegulatoryPackBindApiError> {
    if path_tenant_id.trim().is_empty() {
        return Err(RegulatoryPackBindApiError::EmptyPathTenantId);
    }
    if path_tenant_id != body_tenant_id {
        return Err(RegulatoryPackBindApiError::TenantPathBodyMismatch {
            path_tenant_id: path_tenant_id.to_string(),
            body_tenant_id: body_tenant_id.to_string(),
        });
    }
    Ok(())
}

fn validate_principal_binding(
    boundary: &RegulatoryPackApiBoundaryContext,
    principal: &RegulatoryPackApiPrincipal,
) -> Result<(), RegulatoryPackBindApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(RegulatoryPackBindApiError::EmptyPrincipalId);
    }
    if boundary.tenant_id != principal.tenant_id {
        return Err(RegulatoryPackBindApiError::PrincipalTenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &RegulatoryPackApiPrincipal,
    authorization: &RegulatoryPackApiAuthorization,
    surface: &str,
) -> Result<(), RegulatoryPackBindApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(RegulatoryPackBindApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(RegulatoryPackBindApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(RegulatoryPackBindApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed_surface| allowed_surface == surface)
    {
        return Err(RegulatoryPackBindApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn validate_primary_pack_binding(
    body: &RegulatoryPackBindRequest,
) -> Result<(), RegulatoryPackBindApiError> {
    if body.pack_refs.is_empty() {
        return Err(RegulatoryPackBindApiError::EmptyPackRefs);
    }
    let primary = body
        .pack_refs
        .iter()
        .find(|pack| pack.pack_id == body.primary_pack_id)
        .ok_or_else(|| RegulatoryPackBindApiError::PrimaryPackMissing {
            primary_pack_id: body.primary_pack_id.clone(),
        })?;
    if primary.region != body.home_region {
        return Err(RegulatoryPackBindApiError::PrimaryPackRegionMismatch {
            primary_pack_id: body.primary_pack_id.clone(),
            home_region: body.home_region.clone(),
            pack_region: primary.region.clone(),
        });
    }
    if primary.residency_class != body.residency_class {
        return Err(RegulatoryPackBindApiError::PrimaryPackResidencyMismatch {
            primary_pack_id: body.primary_pack_id.clone(),
            residency_class: body.residency_class.clone(),
            pack_residency_class: primary.residency_class.clone(),
        });
    }
    Ok(())
}

fn parse_api_residency_class(label: &str) -> Result<ResidencyClass, RegulatoryPackBindApiError> {
    parse_residency_class_label(label).ok_or(RegulatoryPackBindApiError::InvalidResidencyClass {
        residency_class: label.to_string(),
    })
}

fn regional_packs_from_request(
    body: &RegulatoryPackBindRequest,
) -> Result<Vec<RegionalPack>, RegulatoryPackBindApiError> {
    body.pack_refs
        .iter()
        .map(|pack| {
            RegionalPack::new(
                pack.pack_id.clone(),
                pack.region.clone(),
                pack.residency_class.clone(),
                pack.controls
                    .iter()
                    .map(|control| control.value.clone())
                    .collect(),
            )
            .map_err(RegulatoryPackBindApiError::RegionalPack)
        })
        .collect()
}

fn region_ref_from_request(
    body: &RegulatoryPackBindRequest,
) -> Result<RegionRef, RegulatoryPackBindApiError> {
    RegionRef::new(RegionRefCreate {
        region_id: body.home_region.clone(),
        jurisdiction: infer_region_jurisdiction_label(&body.home_region),
        cell_group_ref: body.cell_group_ref.clone(),
    })
    .map_err(RegulatoryPackBindApiError::Residency)
}

fn binding_record_from_request(
    body: &RegulatoryPackBindRequest,
    packs: &[RegionalPack],
    schema_version: u32,
) -> RegulatoryPackBindingRecord {
    RegulatoryPackBindingRecord {
        tenant_id: body.tenant_id.clone(),
        primary_pack_id: body.primary_pack_id.clone(),
        home_region: body.home_region.clone(),
        cell_group_ref: body.cell_group_ref.clone(),
        residency_class: body.residency_class.clone(),
        evidence_ref: body.evidence_ref.clone(),
        bound_at_epoch_seconds: body.bound_at_epoch_seconds,
        pack_refs: packs.iter().map(pack_ref_from_kernel).collect(),
        schema_version,
    }
}

fn pack_ref_from_kernel(pack: &RegionalPack) -> RegulatoryPackBindingPackRef {
    RegulatoryPackBindingPackRef {
        pack_id: pack.id.clone(),
        region: pack.region.value.clone(),
        residency_class: pack
            .residency_class
            .value
            .label()
            .unwrap_or("per_pack")
            .to_string(),
        controls: pack
            .controls
            .value
            .iter()
            .map(|control| RegulatoryPackControlRef {
                value: control.clone(),
            })
            .collect(),
    }
}

fn idempotency_key_for(
    boundary: &RegulatoryPackApiBoundaryContext,
    principal: &RegulatoryPackApiPrincipal,
    surface: &str,
) -> RegulatoryPackBindIdempotencyLedgerKey {
    RegulatoryPackBindIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn regulatory_pack_bind_fingerprint_for(
    request: &RegulatoryPackBindApiRequest,
) -> RegulatoryPackBindRequestFingerprint {
    let mut parts = vec![
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
        format!("body.tenant_id={}", request.body.tenant_id),
        format!("body.primary_pack_id={}", request.body.primary_pack_id),
        format!("body.home_region={}", request.body.home_region),
        format!("body.cell_group_ref={}", request.body.cell_group_ref),
        format!("body.residency_class={}", request.body.residency_class),
        format!("body.evidence_ref={}", request.body.evidence_ref),
        format!(
            "body.bound_at_epoch_seconds={}",
            request.body.bound_at_epoch_seconds
        ),
    ];
    for pack in &request.body.pack_refs {
        parts.push(format!(
            "body.pack_ref={}:{}:{}:{}",
            pack.pack_id,
            pack.region,
            pack.residency_class,
            pack.controls
                .iter()
                .map(|control| control.value.as_str())
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    RegulatoryPackBindRequestFingerprint {
        canonical: parts.join("|"),
    }
}

fn detail(field: &str, issue: &str) -> RegulatoryPackBindApiErrorDetail {
    RegulatoryPackBindApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}

fn regional_pack_error_code(error: &RegionalPackError) -> RegulatoryPackBindApiErrorCode {
    match error {
        RegionalPackError::InvalidPackId => {
            RegulatoryPackBindApiErrorCode::RegionalPackInvalidPackId
        }
        RegionalPackError::EmptyRegion => RegulatoryPackBindApiErrorCode::RegionalPackEmptyRegion,
        RegionalPackError::EmptyResidencyClass => {
            RegulatoryPackBindApiErrorCode::RegionalPackEmptyResidencyClass
        }
        RegionalPackError::InvalidResidencyClass => {
            RegulatoryPackBindApiErrorCode::RegionalPackInvalidResidencyClass
        }
        RegionalPackError::MissingControls => {
            RegulatoryPackBindApiErrorCode::RegionalPackMissingControls
        }
    }
}

fn regional_pack_error_message(error: &RegionalPackError) -> &'static str {
    match error {
        RegionalPackError::InvalidPackId => "Regional pack id must use the pack- prefix",
        RegionalPackError::EmptyRegion => "Regional pack region is required",
        RegionalPackError::EmptyResidencyClass => "Regional pack residency_class is required",
        RegionalPackError::InvalidResidencyClass => {
            "Regional pack residency_class must use a supported canonical label"
        }
        RegionalPackError::MissingControls => "Regional pack controls must be non-empty",
    }
}

fn regional_pack_error_issue(error: &RegionalPackError) -> &'static str {
    match error {
        RegionalPackError::InvalidPackId => "pack_id must start with pack-",
        RegionalPackError::EmptyRegion => "region must be non-empty",
        RegionalPackError::EmptyResidencyClass => "residency_class must be non-empty",
        RegionalPackError::InvalidResidencyClass => {
            "residency_class must be strict_home_region, home_with_recovery_failover, or global"
        }
        RegionalPackError::MissingControls => "controls must contain at least one control mapping",
    }
}

fn residency_error_code(error: &ResidencyError) -> RegulatoryPackBindApiErrorCode {
    match error {
        ResidencyError::InvalidTenantId => RegulatoryPackBindApiErrorCode::ResidencyInvalidTenantId,
        ResidencyError::InvalidRegionId => RegulatoryPackBindApiErrorCode::ResidencyInvalidRegionId,
        ResidencyError::InvalidCellGroupRef => {
            RegulatoryPackBindApiErrorCode::ResidencyInvalidCellGroupRef
        }
        ResidencyError::InvalidPackId => RegulatoryPackBindApiErrorCode::ResidencyInvalidPackId,
        ResidencyError::InvalidEvidenceRef => {
            RegulatoryPackBindApiErrorCode::ResidencyInvalidEvidenceRef
        }
        ResidencyError::SourceRegionNotAllowed | ResidencyError::DefaultResidencyNotAllowed => {
            RegulatoryPackBindApiErrorCode::ResidencyPrimaryRegionDenied
        }
        ResidencyError::ResidencyAlreadyBound => {
            RegulatoryPackBindApiErrorCode::ResidencyAlreadyBound
        }
        _ => RegulatoryPackBindApiErrorCode::ResidencyPrimaryRegionDenied,
    }
}

fn residency_error_message(error: &ResidencyError) -> &'static str {
    match error {
        ResidencyError::InvalidTenantId => "Tenant id is invalid for residency binding",
        ResidencyError::InvalidRegionId => "Home region is invalid for residency binding",
        ResidencyError::InvalidCellGroupRef => "Cell group reference is required",
        ResidencyError::InvalidPackId => "Primary regional pack id is invalid",
        ResidencyError::InvalidEvidenceRef => "Residency evidence reference is required",
        ResidencyError::ResidencyAlreadyBound => {
            "Tenant residency is already bound and immutable post-bind"
        }
        _ => "Residency binding violates platform residency invariants",
    }
}

fn residency_error_issue(error: &ResidencyError) -> &'static str {
    match error {
        ResidencyError::InvalidTenantId => "tenant_id must be a valid tenant identifier",
        ResidencyError::InvalidRegionId => "home_region must be non-empty",
        ResidencyError::InvalidCellGroupRef => "cell_group_ref must be non-empty",
        ResidencyError::InvalidPackId => "primary_pack_id must be a valid pack id",
        ResidencyError::InvalidEvidenceRef => "evidence_ref must be non-empty",
        ResidencyError::ResidencyAlreadyBound => "tenant binding is immutable post-bind",
        _ => "residency class must allow the requested home region and binding inputs",
    }
}
