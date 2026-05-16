//! Cloud KMS API boundary for encrypt and decrypt authorization receipts.
//!
//! This crate owns tenant/header/path/body normalization before handing typed
//! requests to the Cloud KMS kernel.

use std::collections::BTreeMap;

use oya_cloud_kms_domain::{
    CloudKmsDirectory, CloudKmsError, KmsDecryptRequest, KmsEncryptRequest, KmsOperation,
    KmsPurpose, KmsRepo, KmsUseReceipt,
};
use oya_data_boundary_kernel::parse_data_class_label;

pub const CLOUD_KMS_ENCRYPT_SURFACE: &str = "cloud.kms.encrypt";
pub const CLOUD_KMS_DECRYPT_SURFACE: &str = "cloud.kms.decrypt";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudKmsCryptoApiStatus {
    Ok,
    BadRequest,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

impl CloudKmsCryptoApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Ok => 200,
            Self::BadRequest => 400,
            Self::Forbidden => 403,
            Self::NotFound => 404,
            Self::Conflict => 409,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudKmsApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PathKeyIdEmpty,
    KeyIdMismatch,
    TenantMismatch,
    PrincipalMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    IdempotencyKeyReused,
    DataClassInvalid,
    PurposeInvalid,
    KmsInvalidRequest,
    KmsForbidden,
    KmsNotFound,
    KmsConflict,
}

impl CloudKmsApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "CLOUD_KMS_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "CLOUD_KMS_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "CLOUD_KMS_IDEMPOTENCY_KEY_EMPTY",
            Self::PathKeyIdEmpty => "CLOUD_KMS_PATH_KEY_ID_EMPTY",
            Self::KeyIdMismatch => "CLOUD_KMS_KEY_ID_MISMATCH",
            Self::TenantMismatch => "CLOUD_KMS_TENANT_MISMATCH",
            Self::PrincipalMismatch => "CLOUD_KMS_PRINCIPAL_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => "CLOUD_KMS_AUTHORIZATION_DECISION_ID_EMPTY",
            Self::AuthorizationTenantMismatch => "CLOUD_KMS_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => "CLOUD_KMS_AUTHORIZATION_PRINCIPAL_MISMATCH",
            Self::AuthorizationDenied => "CLOUD_KMS_AUTHORIZATION_DENIED",
            Self::IdempotencyKeyReused => "CLOUD_KMS_IDEMPOTENCY_KEY_REUSED",
            Self::DataClassInvalid => "CLOUD_KMS_DATA_CLASS_INVALID",
            Self::PurposeInvalid => "CLOUD_KMS_PURPOSE_INVALID",
            Self::KmsInvalidRequest => "CLOUD_KMS_INVALID_REQUEST",
            Self::KmsForbidden => "CLOUD_KMS_FORBIDDEN",
            Self::KmsNotFound => "CLOUD_KMS_NOT_FOUND",
            Self::KmsConflict => "CLOUD_KMS_CONFLICT",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsEncryptRequest {
    pub event_id: String,                // data_class: INTERNAL_ONLY
    pub key_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub plaintext_ref: String,           // data_class: INTERNAL_ONLY
    pub ciphertext_ref: String,          // data_class: INTERNAL_ONLY
    pub data_class: String,              // data_class: INTERNAL_ONLY
    pub purpose: String,                 // data_class: INTERNAL_ONLY
    pub actor: String,                   // data_class: INTERNAL_ONLY
    pub aad_fingerprint: String,         // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsDecryptRequest {
    pub event_id: String,                // data_class: INTERNAL_ONLY
    pub key_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub ciphertext_ref: String,          // data_class: INTERNAL_ONLY
    pub data_class: String,              // data_class: INTERNAL_ONLY
    pub purpose: String,                 // data_class: INTERNAL_ONLY
    pub actor: String,                   // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsEncryptApiRequest {
    pub path_key_id: String,                     // data_class: INTERNAL_ONLY
    pub boundary: CloudKmsApiBoundaryContext,    // data_class: INTERNAL_ONLY
    pub principal: CloudKmsApiPrincipal,         // data_class: INTERNAL_ONLY
    pub authorization: CloudKmsApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: CloudKmsEncryptRequest,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsDecryptApiRequest {
    pub path_key_id: String,                     // data_class: INTERNAL_ONLY
    pub boundary: CloudKmsApiBoundaryContext,    // data_class: INTERNAL_ONLY
    pub principal: CloudKmsApiPrincipal,         // data_class: INTERNAL_ONLY
    pub authorization: CloudKmsApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: CloudKmsDecryptRequest,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudKmsCryptoIdempotencyLedger {
    entries: BTreeMap<CloudKmsIdempotencyLedgerKey, CloudKmsCryptoIdempotencyLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl CloudKmsCryptoIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CloudKmsIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudKmsCryptoIdempotencyLedgerEntry {
    fingerprint: CloudKmsRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CloudKmsCryptoApiResult,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudKmsRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

type CloudKmsCryptoApiResult = Result<CloudKmsCryptoSuccessResponse, CloudKmsApiError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsCryptoSuccessResponse {
    pub data: CloudKmsUseReceiptRecord, // data_class: INTERNAL_ONLY
    pub metadata: CloudKmsApiResponseMetadata, // data_class: INTERNAL_ONLY
}

impl CloudKmsCryptoSuccessResponse {
    pub fn ok(data: CloudKmsUseReceiptRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: CloudKmsApiResponseMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsApiResponseMetadata {
    pub request_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsUseReceiptRecord {
    pub event_id: String,               // data_class: INTERNAL_ONLY
    pub key_id: String,                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub operation: String,              // data_class: PUBLIC
    pub material_ref: Option<String>,   // data_class: INTERNAL_ONLY
    pub ciphertext_ref: String,         // data_class: INTERNAL_ONLY
    pub data_class: String,             // data_class: INTERNAL_ONLY
    pub purpose: String,                // data_class: INTERNAL_ONLY
    pub actor: String,                  // data_class: INTERNAL_ONLY
    pub key_version: u32,               // data_class: INTERNAL_ONLY
    pub hsm_partition_ref: String,      // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub schema_version: u32,            // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsApiErrorResponse {
    pub error: CloudKmsApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsApiErrorBody {
    pub code: String,                         // data_class: INTERNAL_ONLY
    pub message: String,                      // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,    // data_class: INTERNAL_ONLY
    pub request_id: String,                   // data_class: INTERNAL_ONLY
    pub details: Vec<CloudKmsApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudKmsApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPathKeyId,
    KeyIdMismatch {
        path_key_id: String, // data_class: INTERNAL_ONLY
        body_key_id: String, // data_class: INTERNAL_ONLY
    },
    TenantMismatch {
        header_tenant_id: String,    // data_class: INTERNAL_ONLY
        principal_tenant_id: String, // data_class: INTERNAL_ONLY
        body_tenant_id: String,      // data_class: INTERNAL_ONLY
    },
    PrincipalMismatch {
        principal_tenant_id: String, // data_class: INTERNAL_ONLY
        principal_id: String,        // data_class: INTERNAL_ONLY
        body_tenant_id: String,      // data_class: INTERNAL_ONLY
        actor: String,               // data_class: INTERNAL_ONLY
    },
    EmptyAuthorizationDecisionId,
    AuthorizationTenantMismatch {
        authorization_tenant_id: String, // data_class: INTERNAL_ONLY
        principal_tenant_id: String,     // data_class: INTERNAL_ONLY
    },
    AuthorizationPrincipalMismatch {
        authorization_principal_id: String, // data_class: INTERNAL_ONLY
        principal_id: String,               // data_class: INTERNAL_ONLY
    },
    AuthorizationDenied {
        surface: String, // data_class: INTERNAL_ONLY
    },
    IdempotencyKeyReused {
        idempotency_key: String, // data_class: INTERNAL_ONLY
    },
    InvalidDataClassLabel {
        data_class: String, // data_class: INTERNAL_ONLY
    },
    InvalidPurposeLabel {
        purpose: String, // data_class: INTERNAL_ONLY
    },
    Kms(CloudKmsError),
}

impl CloudKmsApiError {
    pub fn crypto_status(&self) -> CloudKmsCryptoApiStatus {
        match self.status_kind() {
            CloudKmsApiStatusKind::BadRequest => CloudKmsCryptoApiStatus::BadRequest,
            CloudKmsApiStatusKind::Forbidden => CloudKmsCryptoApiStatus::Forbidden,
            CloudKmsApiStatusKind::NotFound => CloudKmsCryptoApiStatus::NotFound,
            CloudKmsApiStatusKind::Conflict => CloudKmsCryptoApiStatus::Conflict,
            CloudKmsApiStatusKind::UnprocessableEntity => {
                CloudKmsCryptoApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn crypto_status_code(&self) -> u16 {
        self.crypto_status().code()
    }

    pub fn code(&self) -> CloudKmsApiErrorCode {
        match self {
            Self::EmptyRequestId => CloudKmsApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => CloudKmsApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => CloudKmsApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPathKeyId => CloudKmsApiErrorCode::PathKeyIdEmpty,
            Self::KeyIdMismatch { .. } => CloudKmsApiErrorCode::KeyIdMismatch,
            Self::TenantMismatch { .. } => CloudKmsApiErrorCode::TenantMismatch,
            Self::PrincipalMismatch { .. } => CloudKmsApiErrorCode::PrincipalMismatch,
            Self::EmptyAuthorizationDecisionId => {
                CloudKmsApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                CloudKmsApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                CloudKmsApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => CloudKmsApiErrorCode::AuthorizationDenied,
            Self::IdempotencyKeyReused { .. } => CloudKmsApiErrorCode::IdempotencyKeyReused,
            Self::InvalidDataClassLabel { .. } => CloudKmsApiErrorCode::DataClassInvalid,
            Self::InvalidPurposeLabel { .. } => CloudKmsApiErrorCode::PurposeInvalid,
            Self::Kms(error) => match cloud_kms_status_kind(error) {
                CloudKmsApiStatusKind::BadRequest => CloudKmsApiErrorCode::KmsInvalidRequest,
                CloudKmsApiStatusKind::Forbidden => CloudKmsApiErrorCode::KmsForbidden,
                CloudKmsApiStatusKind::NotFound => CloudKmsApiErrorCode::KmsNotFound,
                CloudKmsApiStatusKind::Conflict => CloudKmsApiErrorCode::KmsConflict,
                CloudKmsApiStatusKind::UnprocessableEntity => {
                    CloudKmsApiErrorCode::KmsInvalidRequest
                }
            },
        }
    }

    pub fn error_response(&self, request_id: impl Into<String>) -> CloudKmsApiErrorResponse {
        CloudKmsApiErrorResponse {
            error: CloudKmsApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> CloudKmsApiStatusKind {
        match self {
            Self::TenantMismatch { .. }
            | Self::PrincipalMismatch { .. }
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. } => CloudKmsApiStatusKind::Forbidden,
            Self::IdempotencyKeyReused { .. } => CloudKmsApiStatusKind::UnprocessableEntity,
            Self::Kms(error) => cloud_kms_status_kind(error),
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::EmptyPathKeyId
            | Self::KeyIdMismatch { .. }
            | Self::InvalidDataClassLabel { .. }
            | Self::InvalidPurposeLabel { .. } => CloudKmsApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPathKeyId => "Path key id is required",
            Self::KeyIdMismatch { .. } => "Path and body key ids must match",
            Self::TenantMismatch { .. } => {
                "Tenant header must match authenticated principal and request body"
            }
            Self::PrincipalMismatch { .. } => {
                "Authenticated principal must match the Cloud KMS actor"
            }
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the requested Cloud KMS surface"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::InvalidDataClassLabel { .. } => {
                "Request data_class must be a known privacy data class"
            }
            Self::InvalidPurposeLabel { .. } => "Request purpose must be a known KMS purpose",
            Self::Kms(error) => cloud_kms_message(error),
        }
    }

    fn details(&self) -> Vec<CloudKmsApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPathKeyId => vec![detail("path.key_id", "must be non-empty")],
            Self::KeyIdMismatch { .. } => {
                vec![detail("key_id", "path key_id and body key_id must match")]
            }
            Self::TenantMismatch { .. } => vec![detail(
                "tenant_id",
                "header tenant, principal tenant, and body tenant_id must match",
            )],
            Self::PrincipalMismatch { .. } => vec![detail(
                "actor",
                "authenticated subject must match body actor and tenant_id",
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
                "must include the requested Cloud KMS surface",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::InvalidDataClassLabel { .. } => vec![detail(
                "body.data_class",
                "must be a canonical privacy data-class label",
            )],
            Self::InvalidPurposeLabel { .. } => vec![detail(
                "body.purpose",
                "must be a canonical Cloud KMS purpose label",
            )],
            Self::Kms(error) => vec![detail("cloud_kms", cloud_kms_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CloudKmsApiStatusKind {
    BadRequest,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

pub fn validate_cloud_kms_encrypt_request(
    request: &CloudKmsEncryptApiRequest,
) -> Result<(), CloudKmsApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_key_id(&request.path_key_id, &request.body.key_id)?;
    validate_tenant_binding(
        &request.boundary,
        &request.principal,
        &request.body.tenant_id,
    )?;
    validate_principal_actor(
        &request.principal,
        &request.body.tenant_id,
        &request.body.actor,
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        CLOUD_KMS_ENCRYPT_SURFACE,
    )
}

pub fn validate_cloud_kms_decrypt_request(
    request: &CloudKmsDecryptApiRequest,
) -> Result<(), CloudKmsApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_key_id(&request.path_key_id, &request.body.key_id)?;
    validate_tenant_binding(
        &request.boundary,
        &request.principal,
        &request.body.tenant_id,
    )?;
    validate_principal_actor(
        &request.principal,
        &request.body.tenant_id,
        &request.body.actor,
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        CLOUD_KMS_DECRYPT_SURFACE,
    )
}

pub fn authorize_cloud_kms_encrypt_from_api(
    directory: &mut CloudKmsDirectory,
    idempotency_ledger: &mut CloudKmsCryptoIdempotencyLedger,
    request: CloudKmsEncryptApiRequest,
) -> Result<CloudKmsCryptoSuccessResponse, CloudKmsApiError> {
    validate_cloud_kms_encrypt_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_KMS_ENCRYPT_SURFACE,
    );
    let fingerprint = encrypt_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudKmsApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let result = encrypt_input(request.body)
        .and_then(|input| {
            directory
                .authorize_encrypt(input)
                .map_err(CloudKmsApiError::Kms)
        })
        .map(|receipt| CloudKmsCryptoSuccessResponse::ok(receipt_record(receipt), request_id));
    idempotency_ledger.entries.insert(
        key,
        CloudKmsCryptoIdempotencyLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

pub fn authorize_cloud_kms_decrypt_from_api(
    directory: &mut CloudKmsDirectory,
    idempotency_ledger: &mut CloudKmsCryptoIdempotencyLedger,
    request: CloudKmsDecryptApiRequest,
) -> Result<CloudKmsCryptoSuccessResponse, CloudKmsApiError> {
    validate_cloud_kms_decrypt_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_KMS_DECRYPT_SURFACE,
    );
    let fingerprint = decrypt_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudKmsApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let result = decrypt_input(request.body)
        .and_then(|input| {
            directory
                .authorize_decrypt(input)
                .map_err(CloudKmsApiError::Kms)
        })
        .map(|receipt| CloudKmsCryptoSuccessResponse::ok(receipt_record(receipt), request_id));
    idempotency_ledger.entries.insert(
        key,
        CloudKmsCryptoIdempotencyLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

fn validate_boundary(boundary: &CloudKmsApiBoundaryContext) -> Result<(), CloudKmsApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(CloudKmsApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(CloudKmsApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(CloudKmsApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_key_id(path_key_id: &str, body_key_id: &str) -> Result<(), CloudKmsApiError> {
    if path_key_id.trim().is_empty() {
        return Err(CloudKmsApiError::EmptyPathKeyId);
    }
    if path_key_id != body_key_id {
        return Err(CloudKmsApiError::KeyIdMismatch {
            path_key_id: path_key_id.to_string(),
            body_key_id: body_key_id.to_string(),
        });
    }
    Ok(())
}

fn validate_tenant_binding(
    boundary: &CloudKmsApiBoundaryContext,
    principal: &CloudKmsApiPrincipal,
    body_tenant_id: &str,
) -> Result<(), CloudKmsApiError> {
    if boundary.tenant_id != principal.tenant_id || boundary.tenant_id != body_tenant_id {
        return Err(CloudKmsApiError::TenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
            body_tenant_id: body_tenant_id.to_string(),
        });
    }
    Ok(())
}

fn validate_principal_actor(
    principal: &CloudKmsApiPrincipal,
    body_tenant_id: &str,
    actor: &str,
) -> Result<(), CloudKmsApiError> {
    if principal.principal_id != actor {
        return Err(CloudKmsApiError::PrincipalMismatch {
            principal_tenant_id: principal.tenant_id.clone(),
            principal_id: principal.principal_id.clone(),
            body_tenant_id: body_tenant_id.to_string(),
            actor: actor.to_string(),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &CloudKmsApiPrincipal,
    authorization: &CloudKmsApiAuthorization,
    surface: &str,
) -> Result<(), CloudKmsApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(CloudKmsApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(CloudKmsApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(CloudKmsApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed_surface| allowed_surface == surface)
    {
        return Err(CloudKmsApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn encrypt_input(body: CloudKmsEncryptRequest) -> Result<KmsEncryptRequest, CloudKmsApiError> {
    Ok(KmsEncryptRequest {
        event_id: body.event_id,
        key_id: body.key_id,
        tenant_id: body.tenant_id,
        plaintext_ref: body.plaintext_ref,
        ciphertext_ref: body.ciphertext_ref,
        data_class: parse_api_data_class(body.data_class)?,
        purpose: parse_api_purpose(body.purpose)?,
        actor: body.actor,
        aad_fingerprint: body.aad_fingerprint,
        requested_at_epoch_seconds: body.requested_at_epoch_seconds,
    })
}

fn decrypt_input(body: CloudKmsDecryptRequest) -> Result<KmsDecryptRequest, CloudKmsApiError> {
    Ok(KmsDecryptRequest {
        event_id: body.event_id,
        key_id: body.key_id,
        tenant_id: body.tenant_id,
        ciphertext_ref: body.ciphertext_ref,
        data_class: parse_api_data_class(body.data_class)?,
        purpose: parse_api_purpose(body.purpose)?,
        actor: body.actor,
        requested_at_epoch_seconds: body.requested_at_epoch_seconds,
    })
}

fn parse_api_data_class(
    label: String,
) -> Result<oya_data_boundary_kernel::DataClass, CloudKmsApiError> {
    parse_data_class_label(&label)
        .ok_or(CloudKmsApiError::InvalidDataClassLabel { data_class: label })
}

fn parse_api_purpose(label: String) -> Result<KmsPurpose, CloudKmsApiError> {
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
        _ => Err(CloudKmsApiError::InvalidPurposeLabel { purpose: label }),
    }
}

fn idempotency_key_for(
    boundary: &CloudKmsApiBoundaryContext,
    principal: &CloudKmsApiPrincipal,
    surface: &str,
) -> CloudKmsIdempotencyLedgerKey {
    CloudKmsIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn encrypt_fingerprint_for(request: &CloudKmsEncryptApiRequest) -> CloudKmsRequestFingerprint {
    CloudKmsRequestFingerprint {
        canonical: [
            format!("path.key_id={}", request.path_key_id),
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
            format!("body.event_id={}", request.body.event_id),
            format!("body.key_id={}", request.body.key_id),
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.plaintext_ref={}", request.body.plaintext_ref),
            format!("body.ciphertext_ref={}", request.body.ciphertext_ref),
            format!("body.data_class={}", request.body.data_class),
            format!("body.purpose={}", request.body.purpose),
            format!("body.actor={}", request.body.actor),
            format!("body.aad_fingerprint={}", request.body.aad_fingerprint),
            format!(
                "body.requested_at_epoch_seconds={}",
                request.body.requested_at_epoch_seconds
            ),
        ]
        .join("|"),
    }
}

fn decrypt_fingerprint_for(request: &CloudKmsDecryptApiRequest) -> CloudKmsRequestFingerprint {
    CloudKmsRequestFingerprint {
        canonical: [
            format!("path.key_id={}", request.path_key_id),
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
            format!("body.event_id={}", request.body.event_id),
            format!("body.key_id={}", request.body.key_id),
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.ciphertext_ref={}", request.body.ciphertext_ref),
            format!("body.data_class={}", request.body.data_class),
            format!("body.purpose={}", request.body.purpose),
            format!("body.actor={}", request.body.actor),
            format!(
                "body.requested_at_epoch_seconds={}",
                request.body.requested_at_epoch_seconds
            ),
        ]
        .join("|"),
    }
}

fn receipt_record(receipt: KmsUseReceipt) -> CloudKmsUseReceiptRecord {
    CloudKmsUseReceiptRecord {
        event_id: receipt.event_id.value.value,
        key_id: receipt.key_id.value.value,
        tenant_id: receipt.tenant_id.value,
        operation: kms_operation_label(receipt.operation.value).to_string(),
        material_ref: receipt.material_ref.value.map(|value| value.value),
        ciphertext_ref: receipt.ciphertext_ref.value.value,
        data_class: receipt.data_class.value.label().to_string(),
        purpose: kms_purpose_label(receipt.purpose.value).to_string(),
        actor: receipt.actor.value.value,
        key_version: receipt.key_version.value,
        hsm_partition_ref: receipt.hsm_partition_ref.value.value,
        occurred_at_epoch_seconds: receipt.occurred_at_epoch_seconds.value,
        schema_version: receipt.schema_version.value,
    }
}

fn kms_operation_label(operation: KmsOperation) -> &'static str {
    match operation {
        KmsOperation::Encrypt => "encrypt",
        KmsOperation::Decrypt => "decrypt",
    }
}

fn kms_purpose_label(purpose: KmsPurpose) -> &'static str {
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

fn cloud_kms_status_kind(error: &CloudKmsError) -> CloudKmsApiStatusKind {
    match error {
        CloudKmsError::DuplicateKey | CloudKmsError::DuplicateUseEvent => {
            CloudKmsApiStatusKind::Conflict
        }
        CloudKmsError::UnknownKey => CloudKmsApiStatusKind::NotFound,
        CloudKmsError::ResourceTenantMismatch
        | CloudKmsError::ResourceRegionMismatch
        | CloudKmsError::InvalidDataClass
        | CloudKmsError::InvalidKeyState
        | CloudKmsError::InvalidKeyUsage => CloudKmsApiStatusKind::Forbidden,
        CloudKmsError::InvalidTenantId
        | CloudKmsError::InvalidResourceId
        | CloudKmsError::ResourceKindMismatch
        | CloudKmsError::InvalidKeyId
        | CloudKmsError::KeyIdOriginMismatch
        | CloudKmsError::KeyIdTenantMismatch
        | CloudKmsError::KeyIdRegionMismatch
        | CloudKmsError::InvalidCellId
        | CloudKmsError::CellRegionMismatch
        | CloudKmsError::InvalidHsmPartitionRef
        | CloudKmsError::HsmPartitionMismatch
        | CloudKmsError::HsmValidationDenied
        | CloudKmsError::ResidencyRegionMismatch
        | CloudKmsError::InvalidRotationPeriod
        | CloudKmsError::InvalidEventId
        | CloudKmsError::InvalidMaterialRef
        | CloudKmsError::InvalidCiphertextRef
        | CloudKmsError::InvalidActorRef
        | CloudKmsError::InvalidAadFingerprint
        | CloudKmsError::InvalidTimeOrder
        | CloudKmsError::DestructionSlaExceeded
        | CloudKmsError::InvalidDestructionProofRef => CloudKmsApiStatusKind::BadRequest,
    }
}

fn cloud_kms_message(error: &CloudKmsError) -> &'static str {
    match cloud_kms_status_kind(error) {
        CloudKmsApiStatusKind::Conflict => "Cloud KMS resource already exists",
        CloudKmsApiStatusKind::Forbidden => "Cloud KMS policy denied the request",
        CloudKmsApiStatusKind::NotFound => "Cloud KMS key was not found",
        CloudKmsApiStatusKind::BadRequest => "Cloud KMS rejected the request shape",
        CloudKmsApiStatusKind::UnprocessableEntity => "Cloud KMS rejected request idempotency",
    }
}

fn cloud_kms_issue(error: &CloudKmsError) -> &'static str {
    match error {
        CloudKmsError::InvalidTenantId => "tenant_id must be a ten_ identifier",
        CloudKmsError::InvalidResourceId => "resource_id must be a canonical cloud resource id",
        CloudKmsError::ResourceTenantMismatch => "resource tenant must match request tenant",
        CloudKmsError::ResourceRegionMismatch => "resource region must match key region",
        CloudKmsError::ResourceKindMismatch => "resource kind must be kms-key",
        CloudKmsError::InvalidKeyId => "key_id must be origin/region/tenant/name",
        CloudKmsError::KeyIdOriginMismatch => "key_id origin must match key origin",
        CloudKmsError::KeyIdTenantMismatch => "key_id tenant must match request tenant",
        CloudKmsError::KeyIdRegionMismatch => "key_id region must match request region",
        CloudKmsError::InvalidCellId => "cell_id must be a canonical cell identifier",
        CloudKmsError::CellRegionMismatch => "cell_id must belong to key region",
        CloudKmsError::InvalidHsmPartitionRef => "hsm_partition_ref must be canonical",
        CloudKmsError::HsmPartitionMismatch => "hsm_partition_ref must bind region and cell",
        CloudKmsError::HsmValidationDenied => {
            "KR keys require KCMVP and global keys require FIPS 140-3"
        }
        CloudKmsError::ResidencyRegionMismatch => "residency class must allow key home region",
        CloudKmsError::InvalidDataClass => "request data_class must match key policy class",
        CloudKmsError::InvalidKeyState => "key state must allow cryptographic use",
        CloudKmsError::InvalidKeyUsage => "key usage must be encrypt/decrypt",
        CloudKmsError::InvalidRotationPeriod => "rotation period must be between 30 and 730 days",
        CloudKmsError::InvalidEventId => "event_id must be a kmsuse_ identifier",
        CloudKmsError::InvalidMaterialRef => "plaintext_ref must be a matref/ reference",
        CloudKmsError::InvalidCiphertextRef => "ciphertext_ref must be a ct/ reference",
        CloudKmsError::InvalidActorRef => "actor must be a usr_ or sp_ principal",
        CloudKmsError::InvalidAadFingerprint => {
            "aad_fingerprint must be a 64-character hexadecimal digest"
        }
        CloudKmsError::InvalidTimeOrder => "request timestamps must be monotonic",
        CloudKmsError::DestructionSlaExceeded => {
            "key destruction proof must complete within 24 hours"
        }
        CloudKmsError::InvalidDestructionProofRef => {
            "destruction proof must be a kproof_ reference"
        }
        CloudKmsError::DuplicateKey => "key identifier is already present",
        CloudKmsError::UnknownKey => "key must exist before cryptographic use",
        CloudKmsError::DuplicateUseEvent => "KMS use event identifier is already present",
    }
}

fn detail(field: &str, issue: &str) -> CloudKmsApiErrorDetail {
    CloudKmsApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}
