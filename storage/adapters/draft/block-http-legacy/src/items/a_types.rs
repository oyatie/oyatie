pub const STORAGE_BLOCK_CREATE_SURFACE: &str = "storage.block.create";

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
            Self::RequestIdEmpty => "STORAGE_BLOCK_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "STORAGE_BLOCK_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "STORAGE_BLOCK_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "STORAGE_BLOCK_PRINCIPAL_ID_EMPTY",
            Self::PathVolumeIdEmpty => "STORAGE_BLOCK_PATH_VOLUME_ID_EMPTY",
            Self::VolumeIdMismatch => "STORAGE_BLOCK_VOLUME_ID_MISMATCH",
            Self::TenantMismatch => "STORAGE_BLOCK_TENANT_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => "STORAGE_BLOCK_AUTHORIZATION_DECISION_ID_EMPTY",
            Self::AuthorizationTenantMismatch => "STORAGE_BLOCK_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => {
                "STORAGE_BLOCK_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationDenied => "STORAGE_BLOCK_AUTHORIZATION_DENIED",
            Self::IdempotencyKeyReused => "STORAGE_BLOCK_IDEMPOTENCY_KEY_REUSED",
            Self::ResidencyInvalid => "STORAGE_BLOCK_RESIDENCY_INVALID",
            Self::VolumeTierInvalid => "STORAGE_BLOCK_VOLUME_TIER_INVALID",
            Self::EncryptionInvalid => "STORAGE_BLOCK_ENCRYPTION_INVALID",
            Self::DataClassInvalid => "STORAGE_BLOCK_DATA_CLASS_INVALID",
            Self::StorageInvalidRequest => "STORAGE_BLOCK_INVALID_REQUEST",
            Self::StorageForbidden => "STORAGE_BLOCK_FORBIDDEN",
            Self::StorageNotFound => "STORAGE_BLOCK_NOT_FOUND",
            Self::StorageConflict => "STORAGE_BLOCK_CONFLICT",
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
