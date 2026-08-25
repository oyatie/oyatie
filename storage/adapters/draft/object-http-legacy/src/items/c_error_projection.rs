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
