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
