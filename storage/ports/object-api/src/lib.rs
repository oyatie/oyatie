//! Cloud Storage Object API boundary for object metadata PUT and GET.
//!
//! This crate owns HTTP boundary normalization, authorization proof checks,
//! PUT idempotency, and tenant-safe object metadata projection around the Cloud
//! storage kernel. Object bytes remain behind storage adapters; this boundary
//! records and projects the typed metadata and KMS shred binding only.

use std::collections::BTreeMap;

use compute_resource::ResourceId;
use oya_data_boundary_kernel::{DataClass, parse_data_class_label};
use secrets_kms_domain::KmsPurpose;
use storage_domain::{
    CloudStorageCatalog, CloudStorageError, ObjectCreate, ObjectEncryptionBindingCreate, ObjectKey,
    StorageRepo, StoredObject,
};

pub const CLOUD_STORAGE_OBJECT_PUT_SURFACE: &str = "cloud.storage.object.put";
pub const CLOUD_STORAGE_OBJECT_GET_SURFACE: &str = "cloud.storage.object.get";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudStorageObjectPutApiStatus {
    Created,
    BadRequest,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

impl CloudStorageObjectPutApiStatus {
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
pub enum CloudStorageObjectGetApiStatus {
    Ok,
    BadRequest,
    Forbidden,
    NotFound,
}

impl CloudStorageObjectGetApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Ok => 200,
            Self::BadRequest => 400,
            Self::Forbidden => 403,
            Self::NotFound => 404,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudStorageObjectApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    BucketIdInvalid,
    BucketKindMismatch,
    ObjectKeyInvalid,
    BucketIdMismatch,
    ObjectKeyMismatch,
    TenantMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    IdempotencyKeyReused,
    DataClassInvalid,
    KmsPurposeInvalid,
    ObjectNotFound,
    StorageInvalidRequest,
    StorageForbidden,
    StorageNotFound,
    StorageConflict,
}

impl CloudStorageObjectApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "CLOUD_STORAGE_OBJECT_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "CLOUD_STORAGE_OBJECT_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "CLOUD_STORAGE_OBJECT_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "CLOUD_STORAGE_OBJECT_PRINCIPAL_ID_EMPTY",
            Self::BucketIdInvalid => "CLOUD_STORAGE_OBJECT_BUCKET_ID_INVALID",
            Self::BucketKindMismatch => "CLOUD_STORAGE_OBJECT_BUCKET_KIND_MISMATCH",
            Self::ObjectKeyInvalid => "CLOUD_STORAGE_OBJECT_KEY_INVALID",
            Self::BucketIdMismatch => "CLOUD_STORAGE_OBJECT_BUCKET_ID_MISMATCH",
            Self::ObjectKeyMismatch => "CLOUD_STORAGE_OBJECT_KEY_MISMATCH",
            Self::TenantMismatch => "CLOUD_STORAGE_OBJECT_TENANT_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => {
                "CLOUD_STORAGE_OBJECT_AUTHORIZATION_DECISION_ID_EMPTY"
            }
            Self::AuthorizationTenantMismatch => {
                "CLOUD_STORAGE_OBJECT_AUTHORIZATION_TENANT_MISMATCH"
            }
            Self::AuthorizationPrincipalMismatch => {
                "CLOUD_STORAGE_OBJECT_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationDenied => "CLOUD_STORAGE_OBJECT_AUTHORIZATION_DENIED",
            Self::IdempotencyKeyReused => "CLOUD_STORAGE_OBJECT_IDEMPOTENCY_KEY_REUSED",
            Self::DataClassInvalid => "CLOUD_STORAGE_OBJECT_DATA_CLASS_INVALID",
            Self::KmsPurposeInvalid => "CLOUD_STORAGE_OBJECT_KMS_PURPOSE_INVALID",
            Self::ObjectNotFound => "CLOUD_STORAGE_OBJECT_NOT_FOUND",
            Self::StorageInvalidRequest => "CLOUD_STORAGE_OBJECT_INVALID_REQUEST",
            Self::StorageForbidden => "CLOUD_STORAGE_OBJECT_FORBIDDEN",
            Self::StorageNotFound => "CLOUD_STORAGE_OBJECT_STORAGE_NOT_FOUND",
            Self::StorageConflict => "CLOUD_STORAGE_OBJECT_CONFLICT",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectMutationBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectReadBoundaryContext {
    pub request_id: String, // data_class: INTERNAL_ONLY
    pub tenant_id: String,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectPutRequest {
    pub bucket_id: String,  // data_class: INTERNAL_ONLY
    pub tenant_id: String,  // data_class: INTERNAL_ONLY
    pub key: String,        // data_class: INTERNAL_ONLY
    pub size_bytes: u64,    // data_class: INTERNAL_ONLY
    pub etag: String,       // data_class: INTERNAL_ONLY
    pub data_class: String, // data_class: INTERNAL_ONLY
    pub encryption: CloudStorageObjectEncryptionBindingRequest, // data_class: INTERNAL_ONLY
    pub stored_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub last_accessed_at_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectEncryptionBindingRequest {
    pub kms_key: String,                 // data_class: INTERNAL_ONLY
    pub kms_key_version: u32,            // data_class: INTERNAL_ONLY
    pub material_ref: String,            // data_class: INTERNAL_ONLY
    pub ciphertext_ref: String,          // data_class: INTERNAL_ONLY
    pub kms_encrypt_event_id: String,    // data_class: INTERNAL_ONLY
    pub purpose: String,                 // data_class: INTERNAL_ONLY
    pub shred_proof_ref: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectPutApiRequest {
    pub path_bucket_id: String,  // data_class: INTERNAL_ONLY
    pub path_object_key: String, // data_class: INTERNAL_ONLY
    pub boundary: CloudStorageObjectMutationBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CloudStorageObjectApiPrincipal, // data_class: INTERNAL_ONLY
    pub authorization: CloudStorageObjectApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: CloudStorageObjectPutRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectGetApiRequest {
    pub path_bucket_id: String,  // data_class: INTERNAL_ONLY
    pub path_object_key: String, // data_class: INTERNAL_ONLY
    pub boundary: CloudStorageObjectReadBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CloudStorageObjectApiPrincipal, // data_class: INTERNAL_ONLY
    pub authorization: CloudStorageObjectApiAuthorization, // data_class: INTERNAL_ONLY
}

/// Typed outcome of inspecting a recorded idempotency ledger entry.
///
/// `Replayed` — the recorded entry holds a success response; the same idempotency
/// key with a matching fingerprint can safely replay the stored result.
///
/// `Conflict` — the same idempotency key was recorded but the caller's fingerprint
/// differs; the caller must return `CloudStorageObjectApiError::IdempotencyKeyReused`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudStorageObjectReplayOutcome {
    Replayed {
        response: Box<CloudStorageObjectPutSuccessResponse>,
    },
    Conflict {
        idempotency_key: String,
    },
}

/// Public projection of a recorded ledger entry.
///
/// Does not expose private `CloudStorageObjectPutLedgerEntry` or
/// `CloudStorageObjectRequestFingerprint`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectPutIdempotencyEntry {
    pub idempotency_key: String,                  // data_class: INTERNAL_ONLY
    pub outcome: CloudStorageObjectReplayOutcome, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudStorageObjectPutIdempotencyLedger {
    entries: BTreeMap<CloudStorageObjectIdempotencyLedgerKey, CloudStorageObjectPutLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl CloudStorageObjectPutIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Return a public projection of the recorded entry for the given composite key,
    /// or `None` if no entry has been recorded yet.
    ///
    /// Does not mutate the ledger. Does not drive the catalog.
    /// The `outcome` field reflects the *recorded* result, not a re-evaluation.
    pub fn peek(
        &self,
        tenant_id: &str,
        principal_id: &str,
        surface: &str,
        idempotency_key: &str,
    ) -> Option<CloudStorageObjectPutIdempotencyEntry> {
        let key = CloudStorageObjectIdempotencyLedgerKey {
            tenant_id: tenant_id.to_string(),
            principal_id: principal_id.to_string(),
            surface: surface.to_string(),
            idempotency_key: idempotency_key.to_string(),
        };
        let entry = self.entries.get(&key)?;
        let outcome = match &entry.result {
            Ok(response) => CloudStorageObjectReplayOutcome::Replayed {
                response: Box::new(response.clone()),
            },
            Err(_) => CloudStorageObjectReplayOutcome::Conflict {
                idempotency_key: idempotency_key.to_string(),
            },
        };
        Some(CloudStorageObjectPutIdempotencyEntry {
            idempotency_key: idempotency_key.to_string(),
            outcome,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CloudStorageObjectIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudStorageObjectPutLedgerEntry {
    fingerprint: CloudStorageObjectRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CloudStorageObjectPutApiResult,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudStorageObjectRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

type CloudStorageObjectPutApiResult =
    Result<CloudStorageObjectPutSuccessResponse, CloudStorageObjectApiError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectPutSuccessResponse {
    pub data: CloudStorageObjectRecord, // data_class: INTERNAL_ONLY
    pub metadata: CloudStorageObjectMetadata, // data_class: INTERNAL_ONLY
}

impl CloudStorageObjectPutSuccessResponse {
    pub fn created(data: CloudStorageObjectRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: CloudStorageObjectMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectGetSuccessResponse {
    pub data: CloudStorageObjectRecord, // data_class: INTERNAL_ONLY
    pub metadata: CloudStorageObjectMetadata, // data_class: INTERNAL_ONLY
}

impl CloudStorageObjectGetSuccessResponse {
    pub fn ok(data: CloudStorageObjectRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: CloudStorageObjectMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectMetadata {
    pub request_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectRecord {
    pub bucket_id: String,  // data_class: INTERNAL_ONLY
    pub tenant_id: String,  // data_class: INTERNAL_ONLY
    pub key: String,        // data_class: INTERNAL_ONLY
    pub size_bytes: u64,    // data_class: INTERNAL_ONLY
    pub etag: String,       // data_class: INTERNAL_ONLY
    pub data_class: String, // data_class: INTERNAL_ONLY
    pub encryption: CloudStorageObjectEncryptionBindingRecord, // data_class: INTERNAL_ONLY
    pub stored_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub last_accessed_at_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: u32, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectEncryptionBindingRecord {
    pub kms_key: String,                 // data_class: INTERNAL_ONLY
    pub kms_key_version: u32,            // data_class: INTERNAL_ONLY
    pub material_ref: String,            // data_class: INTERNAL_ONLY
    pub ciphertext_ref: String,          // data_class: INTERNAL_ONLY
    pub kms_encrypt_event_id: String,    // data_class: INTERNAL_ONLY
    pub purpose: String,                 // data_class: INTERNAL_ONLY
    pub shred_proof_ref: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectApiErrorResponse {
    pub error: CloudStorageObjectApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectApiErrorBody {
    pub code: String,                                   // data_class: INTERNAL_ONLY
    pub message: String,                                // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,              // data_class: INTERNAL_ONLY
    pub request_id: String,                             // data_class: INTERNAL_ONLY
    pub details: Vec<CloudStorageObjectApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudStorageObjectApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    InvalidBucketId {
        bucket_id: String,
    },
    BucketKindMismatch {
        bucket_id: String,
        kind_label: String,
    },
    InvalidObjectKey {
        object_key: String,
    },
    BucketIdMismatch {
        path_bucket_id: String,
        body_bucket_id: String,
    },
    ObjectKeyMismatch {
        path_object_key: String,
        body_key: String,
    },
    TenantMismatch {
        header_tenant_id: String,
        principal_tenant_id: String,
        resource_tenant_id: String,
        body_tenant_id: Option<String>,
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
    InvalidDataClassLabel {
        data_class: String,
    },
    InvalidKmsPurposeLabel {
        purpose: String,
    },
    ObjectNotFound {
        bucket_id: String,
        key: String,
    },
    Storage(CloudStorageError),
}

impl CloudStorageObjectApiError {
    pub fn object_status_code(&self) -> u16 {
        match self.status_kind() {
            CloudStorageObjectApiStatusKind::BadRequest => 400,
            CloudStorageObjectApiStatusKind::Forbidden => 403,
            CloudStorageObjectApiStatusKind::NotFound => 404,
            CloudStorageObjectApiStatusKind::Conflict => 409,
            CloudStorageObjectApiStatusKind::UnprocessableEntity => 422,
        }
    }

    pub fn code(&self) -> CloudStorageObjectApiErrorCode {
        match self {
            Self::EmptyRequestId => CloudStorageObjectApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => CloudStorageObjectApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => CloudStorageObjectApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => CloudStorageObjectApiErrorCode::PrincipalIdEmpty,
            Self::InvalidBucketId { .. } => CloudStorageObjectApiErrorCode::BucketIdInvalid,
            Self::BucketKindMismatch { .. } => CloudStorageObjectApiErrorCode::BucketKindMismatch,
            Self::InvalidObjectKey { .. } => CloudStorageObjectApiErrorCode::ObjectKeyInvalid,
            Self::BucketIdMismatch { .. } => CloudStorageObjectApiErrorCode::BucketIdMismatch,
            Self::ObjectKeyMismatch { .. } => CloudStorageObjectApiErrorCode::ObjectKeyMismatch,
            Self::TenantMismatch { .. } => CloudStorageObjectApiErrorCode::TenantMismatch,
            Self::EmptyAuthorizationDecisionId => {
                CloudStorageObjectApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                CloudStorageObjectApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                CloudStorageObjectApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => CloudStorageObjectApiErrorCode::AuthorizationDenied,
            Self::IdempotencyKeyReused { .. } => {
                CloudStorageObjectApiErrorCode::IdempotencyKeyReused
            }
            Self::InvalidDataClassLabel { .. } => CloudStorageObjectApiErrorCode::DataClassInvalid,
            Self::InvalidKmsPurposeLabel { .. } => {
                CloudStorageObjectApiErrorCode::KmsPurposeInvalid
            }
            Self::ObjectNotFound { .. } => CloudStorageObjectApiErrorCode::ObjectNotFound,
            Self::Storage(error) => match cloud_storage_status_kind(error) {
                CloudStorageObjectApiStatusKind::BadRequest => {
                    CloudStorageObjectApiErrorCode::StorageInvalidRequest
                }
                CloudStorageObjectApiStatusKind::Forbidden => {
                    CloudStorageObjectApiErrorCode::StorageForbidden
                }
                CloudStorageObjectApiStatusKind::NotFound => {
                    CloudStorageObjectApiErrorCode::StorageNotFound
                }
                CloudStorageObjectApiStatusKind::Conflict => {
                    CloudStorageObjectApiErrorCode::StorageConflict
                }
                CloudStorageObjectApiStatusKind::UnprocessableEntity => {
                    CloudStorageObjectApiErrorCode::StorageInvalidRequest
                }
            },
        }
    }

    pub fn error_response(
        &self,
        request_id: impl Into<String>,
    ) -> CloudStorageObjectApiErrorResponse {
        CloudStorageObjectApiErrorResponse {
            error: CloudStorageObjectApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> CloudStorageObjectApiStatusKind {
        match self {
            Self::TenantMismatch { .. }
            | Self::EmptyPrincipalId
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. } => CloudStorageObjectApiStatusKind::Forbidden,
            Self::ObjectNotFound { .. } => CloudStorageObjectApiStatusKind::NotFound,
            Self::IdempotencyKeyReused { .. } => {
                CloudStorageObjectApiStatusKind::UnprocessableEntity
            }
            Self::Storage(error) => cloud_storage_status_kind(error),
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::InvalidBucketId { .. }
            | Self::BucketKindMismatch { .. }
            | Self::InvalidObjectKey { .. }
            | Self::BucketIdMismatch { .. }
            | Self::ObjectKeyMismatch { .. }
            | Self::InvalidDataClassLabel { .. }
            | Self::InvalidKmsPurposeLabel { .. } => CloudStorageObjectApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::InvalidBucketId { .. } => {
                "Bucket id must be a canonical Cloud bucket resource id"
            }
            Self::BucketKindMismatch { .. } => "Bucket id must identify a bucket resource",
            Self::InvalidObjectKey { .. } => "Object key must be canonical",
            Self::BucketIdMismatch { .. } => "Path and body bucket ids must match",
            Self::ObjectKeyMismatch { .. } => "Path and body object keys must match",
            Self::TenantMismatch { .. } => {
                "Tenant header must match authenticated principal, bucket id, and request body"
            }
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the requested Cloud Storage Object surface"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::InvalidDataClassLabel { .. } => {
                "Request data_class must be a known privacy data class"
            }
            Self::InvalidKmsPurposeLabel { .. } => {
                "Request KMS purpose must be a known KMS purpose"
            }
            Self::ObjectNotFound { .. } => "Cloud Storage object was not found",
            Self::Storage(error) => cloud_storage_message(error),
        }
    }

    fn details(&self) -> Vec<CloudStorageObjectApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::InvalidBucketId { .. } => vec![detail(
                "path.bucket_id",
                "must be a canonical oya:cloud bucket resource id",
            )],
            Self::BucketKindMismatch { .. } => {
                vec![detail("path.bucket_id", "resource kind must be bucket")]
            }
            Self::InvalidObjectKey { .. } => vec![detail(
                "path.object_key",
                "must be non-empty, bounded, and cannot contain control bytes or parent traversal",
            )],
            Self::BucketIdMismatch { .. } => vec![detail(
                "bucket_id",
                "path bucket_id and body bucket_id must match",
            )],
            Self::ObjectKeyMismatch { .. } => {
                vec![detail("key", "path object_key and body key must match")]
            }
            Self::TenantMismatch { .. } => vec![detail(
                "tenant_id",
                "header tenant, principal tenant, bucket tenant, and body tenant_id must match",
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
                "must include the requested Cloud Storage Object surface",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::InvalidDataClassLabel { .. } => vec![detail(
                "body.data_class",
                "must be a canonical privacy data-class label",
            )],
            Self::InvalidKmsPurposeLabel { .. } => vec![detail(
                "body.encryption.purpose",
                "must be a canonical KMS purpose label",
            )],
            Self::ObjectNotFound { .. } => vec![detail(
                "path.object_key",
                "object metadata was not found in the requested bucket",
            )],
            Self::Storage(error) => vec![detail("cloud_storage", cloud_storage_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CloudStorageObjectApiStatusKind {
    BadRequest,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

pub fn validate_cloud_storage_object_put_request(
    request: &CloudStorageObjectPutApiRequest,
) -> Result<(), CloudStorageObjectApiError> {
    validate_mutation_boundary(&request.boundary)?;
    validate_path_body_binding(
        &request.path_bucket_id,
        &request.path_object_key,
        &request.body.bucket_id,
        &request.body.key,
    )?;
    let bucket_id = validate_bucket_id(&request.path_bucket_id)?;
    validate_object_key(&request.path_object_key)?;
    validate_tenant_binding(
        &request.boundary.tenant_id,
        &request.principal,
        &bucket_id,
        Some(&request.body.tenant_id),
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        CLOUD_STORAGE_OBJECT_PUT_SURFACE,
    )
}

pub fn put_cloud_storage_object_from_api(
    catalog: &mut CloudStorageCatalog,
    idempotency_ledger: &mut CloudStorageObjectPutIdempotencyLedger,
    request: CloudStorageObjectPutApiRequest,
) -> Result<CloudStorageObjectPutSuccessResponse, CloudStorageObjectApiError> {
    validate_cloud_storage_object_put_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_STORAGE_OBJECT_PUT_SURFACE,
    );
    let fingerprint = put_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        return match replay_outcome_for(entry, &fingerprint, &request.boundary.idempotency_key) {
            CloudStorageObjectReplayOutcome::Replayed { response } => Ok(*response),
            CloudStorageObjectReplayOutcome::Conflict { idempotency_key } => {
                Err(CloudStorageObjectApiError::IdempotencyKeyReused { idempotency_key })
            }
        };
    }

    let request_id = request.boundary.request_id.clone();
    let result = object_put_input(request.body)
        .and_then(|input| {
            catalog
                .put_object(input)
                .map_err(CloudStorageObjectApiError::Storage)
        })
        .map(|object| {
            CloudStorageObjectPutSuccessResponse::created(object_record(object), request_id)
        });
    idempotency_ledger.entries.insert(
        key,
        CloudStorageObjectPutLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

pub fn validate_cloud_storage_object_get_request(
    request: &CloudStorageObjectGetApiRequest,
) -> Result<ResourceId, CloudStorageObjectApiError> {
    validate_read_boundary(&request.boundary)?;
    let bucket_id = validate_bucket_id(&request.path_bucket_id)?;
    validate_object_key(&request.path_object_key)?;
    validate_tenant_binding(
        &request.boundary.tenant_id,
        &request.principal,
        &bucket_id,
        None,
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        CLOUD_STORAGE_OBJECT_GET_SURFACE,
    )?;
    Ok(bucket_id)
}

pub fn get_cloud_storage_object_from_api(
    catalog: &CloudStorageCatalog,
    request: CloudStorageObjectGetApiRequest,
) -> Result<CloudStorageObjectGetSuccessResponse, CloudStorageObjectApiError> {
    let bucket_id = validate_cloud_storage_object_get_request(&request)?;
    let request_id = request.boundary.request_id.clone();
    let object = catalog
        .objects()
        .find(|object| {
            object.bucket_id.value == bucket_id && object.key.value.value == request.path_object_key
        })
        .ok_or_else(|| CloudStorageObjectApiError::ObjectNotFound {
            bucket_id: request.path_bucket_id.clone(),
            key: request.path_object_key.clone(),
        })?;
    if object.tenant_id.value != request.boundary.tenant_id {
        return Err(CloudStorageObjectApiError::TenantMismatch {
            header_tenant_id: request.boundary.tenant_id,
            principal_tenant_id: request.principal.tenant_id,
            resource_tenant_id: object.tenant_id.value.clone(),
            body_tenant_id: None,
        });
    }
    Ok(CloudStorageObjectGetSuccessResponse::ok(
        object_record(object.clone()),
        request_id,
    ))
}

fn validate_mutation_boundary(
    boundary: &CloudStorageObjectMutationBoundaryContext,
) -> Result<(), CloudStorageObjectApiError> {
    validate_request_tenant(&boundary.request_id, &boundary.tenant_id)?;
    if boundary.idempotency_key.trim().is_empty() {
        return Err(CloudStorageObjectApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_read_boundary(
    boundary: &CloudStorageObjectReadBoundaryContext,
) -> Result<(), CloudStorageObjectApiError> {
    validate_request_tenant(&boundary.request_id, &boundary.tenant_id)
}

fn validate_request_tenant(
    request_id: &str,
    tenant_id: &str,
) -> Result<(), CloudStorageObjectApiError> {
    if request_id.trim().is_empty() {
        return Err(CloudStorageObjectApiError::EmptyRequestId);
    }
    if tenant_id.trim().is_empty() {
        return Err(CloudStorageObjectApiError::EmptyTenantHeader);
    }
    Ok(())
}

fn validate_path_body_binding(
    path_bucket_id: &str,
    path_object_key: &str,
    body_bucket_id: &str,
    body_key: &str,
) -> Result<(), CloudStorageObjectApiError> {
    if path_bucket_id != body_bucket_id {
        return Err(CloudStorageObjectApiError::BucketIdMismatch {
            path_bucket_id: path_bucket_id.to_string(),
            body_bucket_id: body_bucket_id.to_string(),
        });
    }
    if path_object_key != body_key {
        return Err(CloudStorageObjectApiError::ObjectKeyMismatch {
            path_object_key: path_object_key.to_string(),
            body_key: body_key.to_string(),
        });
    }
    Ok(())
}

fn validate_bucket_id(value: &str) -> Result<ResourceId, CloudStorageObjectApiError> {
    let bucket_id = ResourceId::new(value.to_string()).map_err(|_| {
        CloudStorageObjectApiError::InvalidBucketId {
            bucket_id: value.to_string(),
        }
    })?;
    let kind_label =
        bucket_id
            .kind_label()
            .map_err(|_| CloudStorageObjectApiError::InvalidBucketId {
                bucket_id: value.to_string(),
            })?;
    if kind_label != "bucket" {
        return Err(CloudStorageObjectApiError::BucketKindMismatch {
            bucket_id: value.to_string(),
            kind_label,
        });
    }
    Ok(bucket_id)
}

fn validate_object_key(value: &str) -> Result<ObjectKey, CloudStorageObjectApiError> {
    ObjectKey::new(value.to_string()).map_err(|_| CloudStorageObjectApiError::InvalidObjectKey {
        object_key: value.to_string(),
    })
}

fn validate_tenant_binding(
    header_tenant_id: &str,
    principal: &CloudStorageObjectApiPrincipal,
    bucket_id: &ResourceId,
    body_tenant_id: Option<&str>,
) -> Result<(), CloudStorageObjectApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(CloudStorageObjectApiError::EmptyPrincipalId);
    }
    let resource_tenant_id =
        bucket_id
            .tenant_id()
            .map_err(|_| CloudStorageObjectApiError::InvalidBucketId {
                bucket_id: bucket_id.value.clone(),
            })?;
    if header_tenant_id != principal.tenant_id || header_tenant_id != resource_tenant_id {
        return Err(CloudStorageObjectApiError::TenantMismatch {
            header_tenant_id: header_tenant_id.to_string(),
            principal_tenant_id: principal.tenant_id.clone(),
            resource_tenant_id,
            body_tenant_id: body_tenant_id.map(str::to_string),
        });
    }
    if body_tenant_id.is_some_and(|tenant_id| header_tenant_id != tenant_id) {
        return Err(CloudStorageObjectApiError::TenantMismatch {
            header_tenant_id: header_tenant_id.to_string(),
            principal_tenant_id: principal.tenant_id.clone(),
            resource_tenant_id,
            body_tenant_id: body_tenant_id.map(str::to_string),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &CloudStorageObjectApiPrincipal,
    authorization: &CloudStorageObjectApiAuthorization,
    surface: &str,
) -> Result<(), CloudStorageObjectApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(CloudStorageObjectApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(CloudStorageObjectApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(CloudStorageObjectApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed_surface| allowed_surface == surface)
    {
        return Err(CloudStorageObjectApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn object_put_input(
    body: CloudStorageObjectPutRequest,
) -> Result<ObjectCreate, CloudStorageObjectApiError> {
    Ok(ObjectCreate {
        bucket_id: body.bucket_id,
        tenant_id: body.tenant_id,
        key: body.key,
        size_bytes: body.size_bytes,
        etag: body.etag,
        data_class: parse_api_data_class(body.data_class)?,
        encryption: ObjectEncryptionBindingCreate {
            kms_key: body.encryption.kms_key,
            kms_key_version: body.encryption.kms_key_version,
            material_ref: body.encryption.material_ref,
            ciphertext_ref: body.encryption.ciphertext_ref,
            kms_encrypt_event_id: body.encryption.kms_encrypt_event_id,
            purpose: parse_api_purpose(body.encryption.purpose)?,
            shred_proof_ref: body.encryption.shred_proof_ref,
        },
        stored_at_epoch_seconds: body.stored_at_epoch_seconds,
        last_accessed_at_epoch_seconds: body.last_accessed_at_epoch_seconds,
    })
}

fn parse_api_data_class(label: String) -> Result<DataClass, CloudStorageObjectApiError> {
    parse_data_class_label(&label)
        .ok_or(CloudStorageObjectApiError::InvalidDataClassLabel { data_class: label })
}

fn parse_api_purpose(label: String) -> Result<KmsPurpose, CloudStorageObjectApiError> {
    match label.as_str() {
        "cloud_object_storage" => Ok(KmsPurpose::CloudObjectStorage),
        "cloud_block_storage" => Ok(KmsPurpose::CloudBlockStorage),
        "cloud_file_storage" => Ok(KmsPurpose::CloudFileStorage),
        "cloud_archive_storage" => Ok(KmsPurpose::CloudArchiveStorage),
        "workspace_drive_object" => Ok(KmsPurpose::WorkspaceDriveObject),
        "workspace_recording" => Ok(KmsPurpose::WorkspaceRecording),
        "secret_provider" => Ok(KmsPurpose::SecretProvider),
        "cross_region_replication" => Ok(KmsPurpose::CrossRegionReplication),
        "database_backup" => Ok(KmsPurpose::DatabaseBackup),
        _ => Err(CloudStorageObjectApiError::InvalidKmsPurposeLabel { purpose: label }),
    }
}

fn purpose_label(purpose: KmsPurpose) -> &'static str {
    match purpose {
        KmsPurpose::CloudObjectStorage => "cloud_object_storage",
        KmsPurpose::CloudBlockStorage => "cloud_block_storage",
        KmsPurpose::CloudFileStorage => "cloud_file_storage",
        KmsPurpose::CloudArchiveStorage => "cloud_archive_storage",
        KmsPurpose::WorkspaceDriveObject => "workspace_drive_object",
        KmsPurpose::WorkspaceRecording => "workspace_recording",
        KmsPurpose::SecretProvider => "secret_provider",
        KmsPurpose::CrossRegionReplication => "cross_region_replication",
        KmsPurpose::DatabaseBackup => "database_backup",
    }
}

/// Compute the replay outcome for an existing ledger entry against the presented
/// fingerprint. This is the single source of truth for same-fingerprint vs
/// different-fingerprint decisions; both `put_cloud_storage_object_from_api` and
/// `CloudStorageObjectPutIdempotencyLedger::peek` delegate here.
fn replay_outcome_for(
    entry: &CloudStorageObjectPutLedgerEntry,
    presented_fingerprint: &CloudStorageObjectRequestFingerprint,
    idempotency_key: &str,
) -> CloudStorageObjectReplayOutcome {
    if entry.fingerprint == *presented_fingerprint {
        match &entry.result {
            Ok(response) => CloudStorageObjectReplayOutcome::Replayed {
                response: Box::new(response.clone()),
            },
            Err(_) => CloudStorageObjectReplayOutcome::Conflict {
                idempotency_key: idempotency_key.to_string(),
            },
        }
    } else {
        CloudStorageObjectReplayOutcome::Conflict {
            idempotency_key: idempotency_key.to_string(),
        }
    }
}

fn idempotency_key_for(
    boundary: &CloudStorageObjectMutationBoundaryContext,
    principal: &CloudStorageObjectApiPrincipal,
    surface: &str,
) -> CloudStorageObjectIdempotencyLedgerKey {
    CloudStorageObjectIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn put_fingerprint_for(
    request: &CloudStorageObjectPutApiRequest,
) -> CloudStorageObjectRequestFingerprint {
    CloudStorageObjectRequestFingerprint {
        canonical: [
            format!("path.bucket_id={}", request.path_bucket_id),
            format!("path.object_key={}", request.path_object_key),
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
            format!("body.bucket_id={}", request.body.bucket_id),
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.key={}", request.body.key),
            format!("body.size_bytes={}", request.body.size_bytes),
            format!("body.etag={}", request.body.etag),
            format!("body.data_class={}", request.body.data_class),
            format!(
                "body.encryption.kms_key={}",
                request.body.encryption.kms_key
            ),
            format!(
                "body.encryption.kms_key_version={}",
                request.body.encryption.kms_key_version
            ),
            format!(
                "body.encryption.material_ref={}",
                request.body.encryption.material_ref
            ),
            format!(
                "body.encryption.ciphertext_ref={}",
                request.body.encryption.ciphertext_ref
            ),
            format!(
                "body.encryption.kms_encrypt_event_id={}",
                request.body.encryption.kms_encrypt_event_id
            ),
            format!(
                "body.encryption.purpose={}",
                request.body.encryption.purpose
            ),
            format!(
                "body.encryption.shred_proof_ref={:?}",
                request.body.encryption.shred_proof_ref
            ),
            format!(
                "body.stored_at_epoch_seconds={}",
                request.body.stored_at_epoch_seconds
            ),
            format!(
                "body.last_accessed_at_epoch_seconds={:?}",
                request.body.last_accessed_at_epoch_seconds
            ),
        ]
        .join("|"),
    }
}

fn object_record(object: StoredObject) -> CloudStorageObjectRecord {
    CloudStorageObjectRecord {
        bucket_id: object.bucket_id.value.value,
        tenant_id: object.tenant_id.value,
        key: object.key.value.value,
        size_bytes: object.size_bytes.value,
        etag: object.etag.value.value,
        data_class: object.data_class.value.label().to_string(),
        encryption: CloudStorageObjectEncryptionBindingRecord {
            kms_key: object.encryption.value.kms_key.value,
            kms_key_version: object.encryption.value.kms_key_version,
            material_ref: object.encryption.value.material_ref.value,
            ciphertext_ref: object.encryption.value.ciphertext_ref.value,
            kms_encrypt_event_id: object.encryption.value.kms_encrypt_event_id.value,
            purpose: purpose_label(object.encryption.value.purpose).to_string(),
            shred_proof_ref: object
                .encryption
                .value
                .shred_proof_ref
                .map(|proof| proof.value),
        },
        stored_at_epoch_seconds: object.stored_at_epoch_seconds.value,
        last_accessed_at_epoch_seconds: object.last_accessed_at_epoch_seconds.value,
        schema_version: object.schema_version.value,
    }
}

fn cloud_storage_status_kind(error: &CloudStorageError) -> CloudStorageObjectApiStatusKind {
    match error {
        CloudStorageError::DuplicateBucket
        | CloudStorageError::DuplicateObject
        | CloudStorageError::DuplicateVolume
        | CloudStorageError::DuplicateFilesystem
        | CloudStorageError::DuplicateArchiveVault
        | CloudStorageError::DuplicateSnapshot => CloudStorageObjectApiStatusKind::Conflict,
        CloudStorageError::UnknownBucket | CloudStorageError::UnknownVolume => {
            CloudStorageObjectApiStatusKind::NotFound
        }
        CloudStorageError::ResourceTenantMismatch
        | CloudStorageError::ResourceRegionMismatch
        | CloudStorageError::KmsKeyModeMismatch
        | CloudStorageError::KmsKeyTenantMismatch
        | CloudStorageError::KmsKeyRegionMismatch
        | CloudStorageError::ReplicationResidencyDenied
        | CloudStorageError::ObjectDataClassDenied
        | CloudStorageError::CellLocationMismatch => CloudStorageObjectApiStatusKind::Forbidden,
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
        | CloudStorageError::InvalidEvidenceRef => CloudStorageObjectApiStatusKind::BadRequest,
    }
}

fn cloud_storage_message(error: &CloudStorageError) -> &'static str {
    match cloud_storage_status_kind(error) {
        CloudStorageObjectApiStatusKind::BadRequest => "Cloud Storage rejected the request shape",
        CloudStorageObjectApiStatusKind::Forbidden => "Cloud Storage policy denied the request",
        CloudStorageObjectApiStatusKind::NotFound => "Cloud Storage resource was not found",
        CloudStorageObjectApiStatusKind::Conflict => "Cloud Storage resource already exists",
        CloudStorageObjectApiStatusKind::UnprocessableEntity => {
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
        CloudStorageError::InvalidInitialState => {
            "create requests must start in active storage state"
        }
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

fn detail(field: &str, issue: &str) -> CloudStorageObjectApiErrorDetail {
    CloudStorageObjectApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use oya_data_boundary_kernel::DataClass;
    use storage_domain::{
        BucketCreate, BucketState, BucketTier, CloudStorageCatalog, EncryptionMode, ObjectLockMode,
        ObjectLockPolicy, ReplicationPolicyCreate, StorageRepo,
    };

    use super::{
        CLOUD_STORAGE_OBJECT_PUT_SURFACE, CloudStorageObjectApiAuthorization,
        CloudStorageObjectApiError, CloudStorageObjectApiPrincipal,
        CloudStorageObjectEncryptionBindingRequest, CloudStorageObjectMutationBoundaryContext,
        CloudStorageObjectPutApiRequest, CloudStorageObjectPutIdempotencyLedger,
        CloudStorageObjectPutRequest, CloudStorageObjectReplayOutcome,
        put_cloud_storage_object_from_api,
    };

    // Region must be in the "region-home" family so StrictHomeRegion residency passes.
    const BUCKET_ID: &str = "oya:cloud:region-home:ten_unit:bucket:unit-assets";
    const OBJECT_KEY: &str = "unit/obj.bin";

    fn bucket_create() -> BucketCreate {
        BucketCreate {
            resource_id: BUCKET_ID.to_string(),
            tenant_id: "ten_unit".to_string(),
            name: "unit-assets".to_string(),
            region: "region-home".to_string(),
            residency: network_residency::ResidencyClass::StrictHomeRegion,
            tier: BucketTier::Standard,
            replication: ReplicationPolicyCreate::Regional,
            encryption: EncryptionMode::SseKms,
            kms_key: Some("kms/region-home/ten_unit/unit-key".to_string()),
            object_lock: Some(ObjectLockPolicy {
                mode: ObjectLockMode::Compliance,
                retain_until_epoch_seconds: 1_800_000_000,
                legal_hold: false,
            }),
            allowed_data_classes: vec![DataClass::Public, DataClass::PiiIdentifying],
            state: BucketState::Creating,
            created_at_epoch_seconds: 1_700_000_000,
        }
    }

    fn catalog_with_active_bucket() -> CloudStorageCatalog {
        let mut catalog = CloudStorageCatalog::default();
        catalog
            .create_bucket(bucket_create())
            .expect("unit bucket creates");
        catalog
            .activate_bucket(BUCKET_ID)
            .expect("unit bucket activates");
        catalog
    }

    fn encryption() -> CloudStorageObjectEncryptionBindingRequest {
        CloudStorageObjectEncryptionBindingRequest {
            kms_key: "kms/region-home/ten_unit/unit-key".to_string(),
            kms_key_version: 1,
            material_ref: "matref/ten_unit/unit/obj".to_string(),
            ciphertext_ref: "ct/ten_unit/unit/obj".to_string(),
            kms_encrypt_event_id: "kmsuse_unit_obj_001".to_string(),
            purpose: "cloud_object_storage".to_string(),
            shred_proof_ref: None,
        }
    }

    fn make_request(request_id: &str, idempotency_key: &str) -> CloudStorageObjectPutApiRequest {
        CloudStorageObjectPutApiRequest {
            path_bucket_id: BUCKET_ID.to_string(),
            path_object_key: OBJECT_KEY.to_string(),
            boundary: CloudStorageObjectMutationBoundaryContext {
                request_id: request_id.to_string(),
                tenant_id: "ten_unit".to_string(),
                idempotency_key: idempotency_key.to_string(),
            },
            principal: CloudStorageObjectApiPrincipal {
                tenant_id: "ten_unit".to_string(),
                principal_id: "sp_unit".to_string(),
            },
            authorization: CloudStorageObjectApiAuthorization {
                tenant_id: "ten_unit".to_string(),
                principal_id: "sp_unit".to_string(),
                decision_id: "authz_unit_001".to_string(),
                allowed_surfaces: vec![CLOUD_STORAGE_OBJECT_PUT_SURFACE.to_string()],
            },
            body: CloudStorageObjectPutRequest {
                bucket_id: BUCKET_ID.to_string(),
                tenant_id: "ten_unit".to_string(),
                key: OBJECT_KEY.to_string(),
                size_bytes: 16,
                etag: "aabbccddeeff00112233445566778899".to_string(),
                data_class: "PII_IDENTIFYING".to_string(),
                encryption: encryption(),
                stored_at_epoch_seconds: 1_700_000_100,
                last_accessed_at_epoch_seconds: None,
            },
        }
    }

    #[test]
    fn first_put_records_and_returns_created() {
        let mut catalog = catalog_with_active_bucket();
        let mut ledger = CloudStorageObjectPutIdempotencyLedger::default();

        let response = put_cloud_storage_object_from_api(
            &mut catalog,
            &mut ledger,
            make_request("req-u1", "idem-u1"),
        )
        .expect("first PUT succeeds");

        assert_eq!(ledger.len(), 1);
        assert_eq!(catalog.objects().count(), 1);
        assert_eq!(response.data.key, OBJECT_KEY);
        assert_eq!(response.metadata.request_id, "req-u1");
    }

    #[test]
    fn replay_same_fingerprint_no_catalog_mutation() {
        let mut catalog = catalog_with_active_bucket();
        let mut ledger = CloudStorageObjectPutIdempotencyLedger::default();
        let request = make_request("req-u2", "idem-u2");

        let first = put_cloud_storage_object_from_api(&mut catalog, &mut ledger, request.clone())
            .expect("first PUT succeeds");
        let second = put_cloud_storage_object_from_api(&mut catalog, &mut ledger, request)
            .expect("replay succeeds");

        assert_eq!(first, second);
        assert_eq!(
            catalog.objects().count(),
            1,
            "catalog not mutated on replay"
        );
        assert_eq!(ledger.len(), 1, "ledger has exactly one entry");
    }

    #[test]
    fn conflict_different_fingerprint_yields_idempotency_key_reused() {
        let mut catalog = catalog_with_active_bucket();
        let mut ledger = CloudStorageObjectPutIdempotencyLedger::default();
        let original = make_request("req-u3", "idem-u3");
        put_cloud_storage_object_from_api(&mut catalog, &mut ledger, original.clone())
            .expect("first PUT succeeds");

        let mut drifted = original;
        drifted.body.etag = "00000000000000000000000000000000".to_string();
        // etag also lives in path-body binding check as separate field; update body only
        let err = put_cloud_storage_object_from_api(&mut catalog, &mut ledger, drifted)
            .expect_err("different fingerprint yields conflict");

        assert_eq!(
            err,
            CloudStorageObjectApiError::IdempotencyKeyReused {
                idempotency_key: "idem-u3".to_string(),
            }
        );
        assert_eq!(
            catalog.objects().count(),
            1,
            "catalog not mutated on conflict"
        );
    }

    #[test]
    fn peek_reflects_each_state() {
        let mut catalog = catalog_with_active_bucket();
        let mut ledger = CloudStorageObjectPutIdempotencyLedger::default();

        // Before any PUT, peek returns None.
        assert!(
            ledger
                .peek(
                    "ten_unit",
                    "sp_unit",
                    CLOUD_STORAGE_OBJECT_PUT_SURFACE,
                    "idem-u4"
                )
                .is_none(),
            "peek returns None before recording"
        );

        // After first PUT, peek returns Some(Replayed { .. }).
        put_cloud_storage_object_from_api(
            &mut catalog,
            &mut ledger,
            make_request("req-u4", "idem-u4"),
        )
        .expect("first PUT succeeds");

        let entry = ledger
            .peek(
                "ten_unit",
                "sp_unit",
                CLOUD_STORAGE_OBJECT_PUT_SURFACE,
                "idem-u4",
            )
            .expect("peek returns Some after record");
        assert_eq!(entry.idempotency_key, "idem-u4");
        assert!(
            matches!(
                entry.outcome,
                CloudStorageObjectReplayOutcome::Replayed { .. }
            ),
            "outcome is Replayed after successful PUT"
        );

        // A different (unknown) key still returns None.
        assert!(
            ledger
                .peek(
                    "ten_unit",
                    "sp_unit",
                    CLOUD_STORAGE_OBJECT_PUT_SURFACE,
                    "idem-unknown"
                )
                .is_none(),
            "peek returns None for unknown key"
        );
    }
}
