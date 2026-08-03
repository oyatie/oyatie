//! Cloud Storage Block API boundary for block volume creation.
//!
//! This crate owns tenant/header/path/body normalization, idempotency, and
//! authenticated API projection before handing typed volume creation requests to
//! the Cloud storage kernel.

use std::collections::BTreeMap;

use storage_domain::{
    BlockVolume, CloudStorageCatalog, CloudStorageError, EncryptionMode, StorageRepo, VolumeCreate,
    VolumePerformance, VolumeState, VolumeTier,
};
use data_boundary_kernel::{DataClass, parse_data_class_label};
use network_residency::{ResidencyClass, parse_residency_class_label};

pub const CLOUD_STORAGE_BLOCK_CREATE_SURFACE: &str = "cloud.storage.block.create";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudStorageBlockCreateApiStatus {
    Created,
    BadRequest,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

impl CloudStorageBlockCreateApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Created => 201,
            Self::BadRequest => 400,
            Self::Forbidden => 403,
            Self::NotFound => 404,
            Self::Conflict => 409,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudStorageBlockApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    PathVolumeIdEmpty,
    VolumeIdMismatch,
    TenantMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    IdempotencyKeyReused,
    ResidencyInvalid,
    VolumeTierInvalid,
    EncryptionInvalid,
    DataClassInvalid,
    StorageInvalidRequest,
    StorageForbidden,
    StorageNotFound,
    StorageConflict,
}

impl CloudStorageBlockApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "CLOUD_STORAGE_BLOCK_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "CLOUD_STORAGE_BLOCK_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "CLOUD_STORAGE_BLOCK_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "CLOUD_STORAGE_BLOCK_PRINCIPAL_ID_EMPTY",
            Self::PathVolumeIdEmpty => "CLOUD_STORAGE_BLOCK_PATH_VOLUME_ID_EMPTY",
            Self::VolumeIdMismatch => "CLOUD_STORAGE_BLOCK_VOLUME_ID_MISMATCH",
            Self::TenantMismatch => "CLOUD_STORAGE_BLOCK_TENANT_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => {
                "CLOUD_STORAGE_BLOCK_AUTHORIZATION_DECISION_ID_EMPTY"
            }
            Self::AuthorizationTenantMismatch => {
                "CLOUD_STORAGE_BLOCK_AUTHORIZATION_TENANT_MISMATCH"
            }
            Self::AuthorizationPrincipalMismatch => {
                "CLOUD_STORAGE_BLOCK_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationDenied => "CLOUD_STORAGE_BLOCK_AUTHORIZATION_DENIED",
            Self::IdempotencyKeyReused => "CLOUD_STORAGE_BLOCK_IDEMPOTENCY_KEY_REUSED",
            Self::ResidencyInvalid => "CLOUD_STORAGE_BLOCK_RESIDENCY_INVALID",
            Self::VolumeTierInvalid => "CLOUD_STORAGE_BLOCK_VOLUME_TIER_INVALID",
            Self::EncryptionInvalid => "CLOUD_STORAGE_BLOCK_ENCRYPTION_INVALID",
            Self::DataClassInvalid => "CLOUD_STORAGE_BLOCK_DATA_CLASS_INVALID",
            Self::StorageInvalidRequest => "CLOUD_STORAGE_BLOCK_INVALID_REQUEST",
            Self::StorageForbidden => "CLOUD_STORAGE_BLOCK_FORBIDDEN",
            Self::StorageNotFound => "CLOUD_STORAGE_BLOCK_NOT_FOUND",
            Self::StorageConflict => "CLOUD_STORAGE_BLOCK_CONFLICT",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageBlockApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageBlockApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageBlockApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageBlockVolumeCreateRequest {
    pub resource_id: String, // data_class: INTERNAL_ONLY
    pub tenant_id: String,   // data_class: INTERNAL_ONLY
    pub name: String,        // data_class: INTERNAL_ONLY
    pub region: String,      // data_class: PUBLIC
    pub az: String,          // data_class: PUBLIC
    pub cell_id: String,     // data_class: PUBLIC
    pub residency: String,   // data_class: INTERNAL_ONLY
    pub tier: String,        // data_class: PUBLIC
    pub size_gib: u64,       // data_class: INTERNAL_ONLY
    pub performance: CloudStorageBlockVolumePerformance, // data_class: PUBLIC
    pub encryption: String,  // data_class: PUBLIC
    pub kms_key: Option<String>, // data_class: INTERNAL_ONLY
    pub data_class: String,  // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CloudStorageBlockVolumePerformance {
    pub iops: u64,            // data_class: PUBLIC
    pub throughput_mbps: u64, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageBlockVolumeCreateApiRequest {
    pub path_volume_id: String, // data_class: INTERNAL_ONLY
    pub boundary: CloudStorageBlockApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CloudStorageBlockApiPrincipal, // data_class: INTERNAL_ONLY
    pub authorization: CloudStorageBlockApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: CloudStorageBlockVolumeCreateRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudStorageBlockCreateIdempotencyLedger {
    entries: BTreeMap<
        CloudStorageBlockIdempotencyLedgerKey,
        CloudStorageBlockCreateIdempotencyLedgerEntry,
    >, // data_class: INTERNAL_ONLY
}

impl CloudStorageBlockCreateIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CloudStorageBlockIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudStorageBlockCreateIdempotencyLedgerEntry {
    fingerprint: CloudStorageBlockRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CloudStorageBlockCreateApiResult,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudStorageBlockRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

type CloudStorageBlockCreateApiResult =
    Result<CloudStorageBlockVolumeCreateSuccessResponse, CloudStorageBlockApiError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageBlockVolumeCreateSuccessResponse {
    pub data: CloudStorageBlockVolumeRecord, // data_class: INTERNAL_ONLY
    pub metadata: CloudStorageBlockApiMetadata, // data_class: INTERNAL_ONLY
}

impl CloudStorageBlockVolumeCreateSuccessResponse {
    pub fn created(data: CloudStorageBlockVolumeRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: CloudStorageBlockApiMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageBlockApiMetadata {
    pub request_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageBlockVolumeRecord {
    pub resource_id: String, // data_class: INTERNAL_ONLY
    pub tenant_id: String,   // data_class: INTERNAL_ONLY
    pub name: String,        // data_class: INTERNAL_ONLY
    pub region: String,      // data_class: PUBLIC
    pub az: String,          // data_class: PUBLIC
    pub cell_id: String,     // data_class: PUBLIC
    pub residency: String,   // data_class: INTERNAL_ONLY
    pub tier: String,        // data_class: PUBLIC
    pub size_gib: u64,       // data_class: INTERNAL_ONLY
    pub performance: CloudStorageBlockVolumePerformance, // data_class: PUBLIC
    pub encryption: String,  // data_class: PUBLIC
    pub kms_key: Option<String>, // data_class: INTERNAL_ONLY
    pub data_class: String,  // data_class: INTERNAL_ONLY
    pub state: String,       // data_class: PUBLIC
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub schema_version: u32, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageBlockApiErrorResponse {
    pub error: CloudStorageBlockApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageBlockApiErrorBody {
    pub code: String,                                  // data_class: INTERNAL_ONLY
    pub message: String,                               // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,             // data_class: INTERNAL_ONLY
    pub request_id: String,                            // data_class: INTERNAL_ONLY
    pub details: Vec<CloudStorageBlockApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,              // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageBlockApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudStorageBlockApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    EmptyPathVolumeId,
    VolumeIdMismatch {
        path_volume_id: String,
        body_resource_id: String,
    },
    TenantMismatch {
        header_tenant_id: String,
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
    InvalidResidencyLabel {
        residency: String,
    },
    InvalidVolumeTierLabel {
        tier: String,
    },
    InvalidEncryptionLabel {
        encryption: String,
    },
    InvalidDataClassLabel {
        data_class: String,
    },
    Storage(CloudStorageError),
}

impl CloudStorageBlockApiError {
    pub fn block_create_status(&self) -> CloudStorageBlockCreateApiStatus {
        match self.status_kind() {
            CloudStorageBlockApiStatusKind::BadRequest => {
                CloudStorageBlockCreateApiStatus::BadRequest
            }
            CloudStorageBlockApiStatusKind::Forbidden => {
                CloudStorageBlockCreateApiStatus::Forbidden
            }
            CloudStorageBlockApiStatusKind::NotFound => CloudStorageBlockCreateApiStatus::NotFound,
            CloudStorageBlockApiStatusKind::Conflict => CloudStorageBlockCreateApiStatus::Conflict,
            CloudStorageBlockApiStatusKind::UnprocessableEntity => {
                CloudStorageBlockCreateApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn block_create_status_code(&self) -> u16 {
        self.block_create_status().code()
    }

    pub fn code(&self) -> CloudStorageBlockApiErrorCode {
        match self {
            Self::EmptyRequestId => CloudStorageBlockApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => CloudStorageBlockApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => CloudStorageBlockApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => CloudStorageBlockApiErrorCode::PrincipalIdEmpty,
            Self::EmptyPathVolumeId => CloudStorageBlockApiErrorCode::PathVolumeIdEmpty,
            Self::VolumeIdMismatch { .. } => CloudStorageBlockApiErrorCode::VolumeIdMismatch,
            Self::TenantMismatch { .. } => CloudStorageBlockApiErrorCode::TenantMismatch,
            Self::EmptyAuthorizationDecisionId => {
                CloudStorageBlockApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                CloudStorageBlockApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                CloudStorageBlockApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => CloudStorageBlockApiErrorCode::AuthorizationDenied,
            Self::IdempotencyKeyReused { .. } => {
                CloudStorageBlockApiErrorCode::IdempotencyKeyReused
            }
            Self::InvalidResidencyLabel { .. } => CloudStorageBlockApiErrorCode::ResidencyInvalid,
            Self::InvalidVolumeTierLabel { .. } => CloudStorageBlockApiErrorCode::VolumeTierInvalid,
            Self::InvalidEncryptionLabel { .. } => CloudStorageBlockApiErrorCode::EncryptionInvalid,
            Self::InvalidDataClassLabel { .. } => CloudStorageBlockApiErrorCode::DataClassInvalid,
            Self::Storage(error) => match cloud_storage_status_kind(error) {
                CloudStorageBlockApiStatusKind::BadRequest => {
                    CloudStorageBlockApiErrorCode::StorageInvalidRequest
                }
                CloudStorageBlockApiStatusKind::Forbidden => {
                    CloudStorageBlockApiErrorCode::StorageForbidden
                }
                CloudStorageBlockApiStatusKind::NotFound => {
                    CloudStorageBlockApiErrorCode::StorageNotFound
                }
                CloudStorageBlockApiStatusKind::Conflict => {
                    CloudStorageBlockApiErrorCode::StorageConflict
                }
                CloudStorageBlockApiStatusKind::UnprocessableEntity => {
                    CloudStorageBlockApiErrorCode::StorageInvalidRequest
                }
            },
        }
    }

    pub fn error_response(
        &self,
        request_id: impl Into<String>,
    ) -> CloudStorageBlockApiErrorResponse {
        CloudStorageBlockApiErrorResponse {
            error: CloudStorageBlockApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> CloudStorageBlockApiStatusKind {
        match self {
            Self::TenantMismatch { .. }
            | Self::EmptyPrincipalId
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. } => CloudStorageBlockApiStatusKind::Forbidden,
            Self::IdempotencyKeyReused { .. } => {
                CloudStorageBlockApiStatusKind::UnprocessableEntity
            }
            Self::Storage(error) => cloud_storage_status_kind(error),
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::EmptyPathVolumeId
            | Self::VolumeIdMismatch { .. }
            | Self::InvalidResidencyLabel { .. }
            | Self::InvalidVolumeTierLabel { .. }
            | Self::InvalidEncryptionLabel { .. }
            | Self::InvalidDataClassLabel { .. } => CloudStorageBlockApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::EmptyPathVolumeId => "Path volume id is required",
            Self::VolumeIdMismatch { .. } => "Path and body volume ids must match",
            Self::TenantMismatch { .. } => {
                "Tenant header must match authenticated principal and request body"
            }
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the requested Cloud Storage Block surface"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::InvalidResidencyLabel { .. } => {
                "Request residency must be a known residency label"
            }
            Self::InvalidVolumeTierLabel { .. } => "Request tier must be a known block volume tier",
            Self::InvalidEncryptionLabel { .. } => {
                "Request encryption must be a known storage encryption mode"
            }
            Self::InvalidDataClassLabel { .. } => {
                "Request data_class must be a known privacy data class"
            }
            Self::Storage(error) => cloud_storage_message(error),
        }
    }

    fn details(&self) -> Vec<CloudStorageBlockApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::EmptyPathVolumeId => vec![detail("path.volume_id", "must be non-empty")],
            Self::VolumeIdMismatch { .. } => vec![detail(
                "resource_id",
                "path volume_id and body resource_id must match",
            )],
            Self::TenantMismatch { .. } => vec![detail(
                "tenant_id",
                "header tenant, principal tenant, and body tenant_id must match",
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
                "must include the requested Cloud Storage Block surface",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::InvalidResidencyLabel { .. } => vec![detail(
                "body.residency",
                "must be a canonical residency label",
            )],
            Self::InvalidVolumeTierLabel { .. } => vec![detail(
                "body.tier",
                "must be general_purpose_ssd or provisioned_iops_ssd",
            )],
            Self::InvalidEncryptionLabel { .. } => vec![detail(
                "body.encryption",
                "must be sse, sse_kms, byok, or hyok",
            )],
            Self::InvalidDataClassLabel { .. } => vec![detail(
                "body.data_class",
                "must be a canonical privacy data-class label",
            )],
            Self::Storage(error) => vec![detail("cloud_storage", cloud_storage_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CloudStorageBlockApiStatusKind {
    BadRequest,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

pub fn validate_cloud_storage_block_create_request(
    request: &CloudStorageBlockVolumeCreateApiRequest,
) -> Result<(), CloudStorageBlockApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_volume_id(&request.path_volume_id, &request.body.resource_id)?;
    validate_tenant_binding(
        &request.boundary,
        &request.principal,
        &request.body.tenant_id,
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        CLOUD_STORAGE_BLOCK_CREATE_SURFACE,
    )
}

pub fn create_cloud_storage_block_volume_from_api(
    catalog: &mut CloudStorageCatalog,
    idempotency_ledger: &mut CloudStorageBlockCreateIdempotencyLedger,
    request: CloudStorageBlockVolumeCreateApiRequest,
) -> Result<CloudStorageBlockVolumeCreateSuccessResponse, CloudStorageBlockApiError> {
    validate_cloud_storage_block_create_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_STORAGE_BLOCK_CREATE_SURFACE,
    );
    let fingerprint = block_create_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudStorageBlockApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let result = volume_create_input(request.body)
        .and_then(|input| {
            catalog
                .create_volume(input)
                .map_err(CloudStorageBlockApiError::Storage)
        })
        .map(|volume| {
            CloudStorageBlockVolumeCreateSuccessResponse::created(
                block_volume_record(volume),
                request_id,
            )
        });
    idempotency_ledger.entries.insert(
        key,
        CloudStorageBlockCreateIdempotencyLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

fn validate_boundary(
    boundary: &CloudStorageBlockApiBoundaryContext,
) -> Result<(), CloudStorageBlockApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(CloudStorageBlockApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(CloudStorageBlockApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(CloudStorageBlockApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_volume_id(
    path_volume_id: &str,
    body_resource_id: &str,
) -> Result<(), CloudStorageBlockApiError> {
    if path_volume_id.trim().is_empty() {
        return Err(CloudStorageBlockApiError::EmptyPathVolumeId);
    }
    if path_volume_id != body_resource_id {
        return Err(CloudStorageBlockApiError::VolumeIdMismatch {
            path_volume_id: path_volume_id.to_string(),
            body_resource_id: body_resource_id.to_string(),
        });
    }
    Ok(())
}

fn validate_tenant_binding(
    boundary: &CloudStorageBlockApiBoundaryContext,
    principal: &CloudStorageBlockApiPrincipal,
    body_tenant_id: &str,
) -> Result<(), CloudStorageBlockApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(CloudStorageBlockApiError::EmptyPrincipalId);
    }
    if boundary.tenant_id != principal.tenant_id || boundary.tenant_id != body_tenant_id {
        return Err(CloudStorageBlockApiError::TenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
            body_tenant_id: body_tenant_id.to_string(),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &CloudStorageBlockApiPrincipal,
    authorization: &CloudStorageBlockApiAuthorization,
    surface: &str,
) -> Result<(), CloudStorageBlockApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(CloudStorageBlockApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(CloudStorageBlockApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(CloudStorageBlockApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed_surface| allowed_surface == surface)
    {
        return Err(CloudStorageBlockApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn volume_create_input(
    body: CloudStorageBlockVolumeCreateRequest,
) -> Result<VolumeCreate, CloudStorageBlockApiError> {
    Ok(VolumeCreate {
        resource_id: body.resource_id,
        tenant_id: body.tenant_id,
        name: body.name,
        region: body.region,
        az: body.az,
        cell_id: body.cell_id,
        residency: parse_api_residency(body.residency)?,
        tier: parse_api_volume_tier(body.tier)?,
        size_gib: body.size_gib,
        performance: VolumePerformance {
            iops: body.performance.iops,
            throughput_mbps: body.performance.throughput_mbps,
        },
        encryption: parse_api_encryption(body.encryption)?,
        kms_key: body.kms_key,
        data_class: parse_api_data_class(body.data_class)?,
        state: VolumeState::Creating,
        created_at_epoch_seconds: body.created_at_epoch_seconds,
    })
}

fn parse_api_residency(label: String) -> Result<ResidencyClass, CloudStorageBlockApiError> {
    parse_residency_class_label(&label)
        .ok_or(CloudStorageBlockApiError::InvalidResidencyLabel { residency: label })
}

fn parse_api_volume_tier(label: String) -> Result<VolumeTier, CloudStorageBlockApiError> {
    match label.as_str() {
        "general_purpose_ssd" => Ok(VolumeTier::GeneralPurposeSsd),
        "provisioned_iops_ssd" => Ok(VolumeTier::ProvisionedIopsSsd),
        _ => Err(CloudStorageBlockApiError::InvalidVolumeTierLabel { tier: label }),
    }
}

fn parse_api_encryption(label: String) -> Result<EncryptionMode, CloudStorageBlockApiError> {
    match label.as_str() {
        "sse" => Ok(EncryptionMode::Sse),
        "sse_kms" => Ok(EncryptionMode::SseKms),
        "byok" => Ok(EncryptionMode::Byok),
        "hyok" => Ok(EncryptionMode::Hyok),
        _ => Err(CloudStorageBlockApiError::InvalidEncryptionLabel { encryption: label }),
    }
}

fn parse_api_data_class(label: String) -> Result<DataClass, CloudStorageBlockApiError> {
    parse_data_class_label(&label)
        .ok_or(CloudStorageBlockApiError::InvalidDataClassLabel { data_class: label })
}

fn idempotency_key_for(
    boundary: &CloudStorageBlockApiBoundaryContext,
    principal: &CloudStorageBlockApiPrincipal,
    surface: &str,
) -> CloudStorageBlockIdempotencyLedgerKey {
    CloudStorageBlockIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn block_create_fingerprint_for(
    request: &CloudStorageBlockVolumeCreateApiRequest,
) -> CloudStorageBlockRequestFingerprint {
    CloudStorageBlockRequestFingerprint {
        canonical: [
            format!("path.volume_id={}", request.path_volume_id),
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
            format!("body.resource_id={}", request.body.resource_id),
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.name={}", request.body.name),
            format!("body.region={}", request.body.region),
            format!("body.az={}", request.body.az),
            format!("body.cell_id={}", request.body.cell_id),
            format!("body.residency={}", request.body.residency),
            format!("body.tier={}", request.body.tier),
            format!("body.size_gib={}", request.body.size_gib),
            format!("body.performance.iops={}", request.body.performance.iops),
            format!(
                "body.performance.throughput_mbps={}",
                request.body.performance.throughput_mbps
            ),
            format!("body.encryption={}", request.body.encryption),
            format!("body.kms_key={:?}", request.body.kms_key),
            format!("body.data_class={}", request.body.data_class),
            format!(
                "body.created_at_epoch_seconds={}",
                request.body.created_at_epoch_seconds
            ),
        ]
        .join("|"),
    }
}

fn block_volume_record(volume: BlockVolume) -> CloudStorageBlockVolumeRecord {
    CloudStorageBlockVolumeRecord {
        resource_id: volume.resource_id.value.value,
        tenant_id: volume.tenant_id.value,
        name: volume.name.value.value,
        region: volume.region.value.value,
        az: volume.az.value.value,
        cell_id: volume.cell_id.value.value,
        residency: volume
            .residency
            .value
            .label()
            .unwrap_or("per_pack")
            .to_string(),
        tier: volume_tier_label(volume.tier.value).to_string(),
        size_gib: volume.size_gib.value,
        performance: CloudStorageBlockVolumePerformance {
            iops: volume.performance.value.iops,
            throughput_mbps: volume.performance.value.throughput_mbps,
        },
        encryption: encryption_label(volume.encryption.value).to_string(),
        kms_key: volume.kms_key.value.map(|key| key.value),
        data_class: volume.data_class.value.label().to_string(),
        state: volume_state_label(volume.state.value).to_string(),
        created_at_epoch_seconds: volume.created_at_epoch_seconds.value,
        schema_version: volume.schema_version.value,
    }
}

fn volume_tier_label(tier: VolumeTier) -> &'static str {
    match tier {
        VolumeTier::GeneralPurposeSsd => "general_purpose_ssd",
        VolumeTier::ProvisionedIopsSsd => "provisioned_iops_ssd",
    }
}

fn encryption_label(encryption: EncryptionMode) -> &'static str {
    match encryption {
        EncryptionMode::Sse => "sse",
        EncryptionMode::SseKms => "sse_kms",
        EncryptionMode::Byok => "byok",
        EncryptionMode::Hyok => "hyok",
    }
}

fn volume_state_label(state: VolumeState) -> &'static str {
    match state {
        VolumeState::Creating => "creating",
        VolumeState::Available => "available",
        VolumeState::Attached => "attached",
        VolumeState::Deleting => "deleting",
        VolumeState::Error => "error",
    }
}

fn cloud_storage_status_kind(error: &CloudStorageError) -> CloudStorageBlockApiStatusKind {
    match error {
        CloudStorageError::DuplicateBucket
        | CloudStorageError::DuplicateObject
        | CloudStorageError::DuplicateVolume
        | CloudStorageError::DuplicateFilesystem
        | CloudStorageError::DuplicateArchiveVault
        | CloudStorageError::DuplicateSnapshot => CloudStorageBlockApiStatusKind::Conflict,
        CloudStorageError::UnknownBucket | CloudStorageError::UnknownVolume => {
            CloudStorageBlockApiStatusKind::NotFound
        }
        CloudStorageError::ResourceTenantMismatch
        | CloudStorageError::ResourceRegionMismatch
        | CloudStorageError::KmsKeyModeMismatch
        | CloudStorageError::KmsKeyTenantMismatch
        | CloudStorageError::KmsKeyRegionMismatch
        | CloudStorageError::ReplicationResidencyDenied
        | CloudStorageError::ObjectDataClassDenied
        | CloudStorageError::CellLocationMismatch => CloudStorageBlockApiStatusKind::Forbidden,
        CloudStorageError::InvalidTenantId
        | CloudStorageError::InvalidResourceId
        | CloudStorageError::ResourceKindMismatch
        | CloudStorageError::InvalidBucketName
        | CloudStorageError::InvalidObjectKey
        | CloudStorageError::InvalidEtag
        | CloudStorageError::InvalidKmsKeyId
        | CloudStorageError::InvalidKmsKeyVersion
        | CloudStorageError::InvalidKmsUseEventId
        | CloudStorageError::InvalidMaterialRef
        | CloudStorageError::InvalidCiphertextRef
        | CloudStorageError::InvalidKmsPurpose
        | CloudStorageError::InvalidDestructionProofRef
        | CloudStorageError::MissingKmsKey
        | CloudStorageError::UnexpectedKmsKey
        | CloudStorageError::InvalidReplicationPolicy
        | CloudStorageError::DuplicateReplicationRegion
        | CloudStorageError::EmptyAllowedDataClassSet
        | CloudStorageError::DuplicateDataClass
        | CloudStorageError::InvalidDataClass
        | CloudStorageError::InvalidObjectLockPolicy
        | CloudStorageError::InvalidSize
        | CloudStorageError::InvalidPerformance
        | CloudStorageError::InvalidAzCode
        | CloudStorageError::InvalidCellId
        | CloudStorageError::AzRegionMismatch
        | CloudStorageError::InvalidSnapshotId
        | CloudStorageError::InvalidInitialState
        | CloudStorageError::InvalidTimeOrder
        | CloudStorageError::InvalidStorageNamespacePolicy
        | CloudStorageError::InvalidEvidenceRef => CloudStorageBlockApiStatusKind::BadRequest,
    }
}

fn cloud_storage_message(error: &CloudStorageError) -> &'static str {
    match cloud_storage_status_kind(error) {
        CloudStorageBlockApiStatusKind::BadRequest => "Cloud Storage rejected the request shape",
        CloudStorageBlockApiStatusKind::Forbidden => "Cloud Storage policy denied the request",
        CloudStorageBlockApiStatusKind::NotFound => "Cloud Storage resource was not found",
        CloudStorageBlockApiStatusKind::Conflict => "Cloud Storage resource already exists",
        CloudStorageBlockApiStatusKind::UnprocessableEntity => {
            "Cloud Storage rejected request idempotency"
        }
    }
}

fn cloud_storage_issue(error: &CloudStorageError) -> &'static str {
    match error {
        CloudStorageError::InvalidTenantId => "tenant_id must be a ten_ identifier",
        CloudStorageError::InvalidResourceId => "resource_id must be canonical cloud resource id",
        CloudStorageError::ResourceTenantMismatch => "resource tenant must match request tenant",
        CloudStorageError::ResourceRegionMismatch => "resource region must match request region",
        CloudStorageError::ResourceKindMismatch => "resource kind must match storage type",
        CloudStorageError::InvalidBucketName => "bucket name must be canonical DNS label",
        CloudStorageError::InvalidObjectKey => "object key must be non-empty and bounded",
        CloudStorageError::InvalidEtag => "etag must be a canonical checksum value",
        CloudStorageError::InvalidKmsKeyId => "kms_key must be canonical for the selected mode",
        CloudStorageError::InvalidKmsKeyVersion => "kms key version must be greater than zero",
        CloudStorageError::InvalidKmsUseEventId => "kms use event id must be canonical",
        CloudStorageError::InvalidMaterialRef => "material_ref must be a matref/ reference",
        CloudStorageError::InvalidCiphertextRef => "ciphertext_ref must be a ct/ reference",
        CloudStorageError::InvalidKmsPurpose => "KMS purpose must match the storage surface",
        CloudStorageError::InvalidDestructionProofRef => "destruction proof must be canonical",
        CloudStorageError::MissingKmsKey => "selected encryption mode requires kms_key",
        CloudStorageError::UnexpectedKmsKey => "selected encryption mode does not accept kms_key",
        CloudStorageError::KmsKeyModeMismatch => {
            "kms_key origin must match selected encryption mode"
        }
        CloudStorageError::KmsKeyTenantMismatch => "kms_key tenant must match request tenant",
        CloudStorageError::KmsKeyRegionMismatch => "kms_key region must match request region",
        CloudStorageError::InvalidReplicationPolicy => "replication policy must be canonical",
        CloudStorageError::DuplicateReplicationRegion => "replication destinations must be unique",
        CloudStorageError::ReplicationResidencyDenied => {
            "replication must satisfy residency policy"
        }
        CloudStorageError::EmptyAllowedDataClassSet => "allowed data-class set must not be empty",
        CloudStorageError::DuplicateDataClass => "allowed data classes must be unique",
        CloudStorageError::InvalidDataClass => "data_class must be a privacy-program class",
        CloudStorageError::ObjectDataClassDenied => {
            "object data_class must be admitted by bucket policy"
        }
        CloudStorageError::InvalidObjectLockPolicy => {
            "object lock policy must define retention or hold"
        }
        CloudStorageError::InvalidSize => "size must be greater than zero",
        CloudStorageError::InvalidPerformance => "volume performance must be greater than zero",
        CloudStorageError::InvalidAzCode => "AZ must be canonical lowercase ASCII",
        CloudStorageError::InvalidCellId => "cell_id must be canonical and use the cell- prefix",
        CloudStorageError::AzRegionMismatch => "AZ code must sit under its region code",
        CloudStorageError::CellLocationMismatch => "cell_id must sit under its AZ and region",
        CloudStorageError::InvalidSnapshotId => "snapshot id must use the snap_ prefix",
        CloudStorageError::InvalidInitialState => "create requests must start in Creating state",
        CloudStorageError::InvalidTimeOrder => "request timestamps must be monotonic",
        CloudStorageError::InvalidStorageNamespacePolicy => {
            "tenant/cell storage namespace policy must be canonical"
        }
        CloudStorageError::InvalidEvidenceRef => {
            "evidence refs must be canonical and must not contain credentials"
        }
        CloudStorageError::DuplicateBucket => "bucket resource id is already present",
        CloudStorageError::UnknownBucket => "bucket must exist before object creation",
        CloudStorageError::DuplicateObject => "object key is already present in the bucket",
        CloudStorageError::DuplicateVolume => "volume resource id is already present",
        CloudStorageError::UnknownVolume => "volume must exist before snapshot creation",
        CloudStorageError::DuplicateFilesystem => "filesystem resource id is already present",
        CloudStorageError::DuplicateArchiveVault => "archive vault resource id is already present",
        CloudStorageError::DuplicateSnapshot => "snapshot id is already present",
    }
}

fn detail(field: &str, issue: &str) -> CloudStorageBlockApiErrorDetail {
    CloudStorageBlockApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}
