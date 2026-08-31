impl CloudComputeFunctionsApiError {
    pub fn invoke_status(&self) -> CloudComputeFunctionsInvokeApiStatus {
        match self.status_kind() {
            CloudComputeFunctionsApiStatusKind::BadRequest => {
                CloudComputeFunctionsInvokeApiStatus::BadRequest
            }
            CloudComputeFunctionsApiStatusKind::Unauthorized => {
                CloudComputeFunctionsInvokeApiStatus::Unauthorized
            }
            CloudComputeFunctionsApiStatusKind::Forbidden => {
                CloudComputeFunctionsInvokeApiStatus::Forbidden
            }
            CloudComputeFunctionsApiStatusKind::NotFound => {
                CloudComputeFunctionsInvokeApiStatus::NotFound
            }
            CloudComputeFunctionsApiStatusKind::Conflict => {
                CloudComputeFunctionsInvokeApiStatus::Conflict
            }
            CloudComputeFunctionsApiStatusKind::UnprocessableEntity => {
                CloudComputeFunctionsInvokeApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn invoke_status_code(&self) -> u16 {
        self.invoke_status().code()
    }

    pub fn code(&self) -> CloudComputeFunctionsApiErrorCode {
        match self {
            Self::EmptyRequestId => CloudComputeFunctionsApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => CloudComputeFunctionsApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => CloudComputeFunctionsApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => CloudComputeFunctionsApiErrorCode::PrincipalIdEmpty,
            Self::EmptyPathFunctionId => CloudComputeFunctionsApiErrorCode::PathFunctionIdEmpty,
            Self::InvalidFunctionId { .. } => CloudComputeFunctionsApiErrorCode::FunctionIdInvalid,
            Self::FunctionKindMismatch { .. } => {
                CloudComputeFunctionsApiErrorCode::FunctionKindMismatch
            }
            Self::FunctionIdMismatch { .. } => {
                CloudComputeFunctionsApiErrorCode::FunctionIdMismatch
            }
            Self::TenantMismatch { .. } => CloudComputeFunctionsApiErrorCode::TenantMismatch,
            Self::EmptyAuthorizationDecisionId => {
                CloudComputeFunctionsApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationVerifierMissing => {
                CloudComputeFunctionsApiErrorCode::AuthorizationVerifierMissing
            }
            Self::AuthorizationTenantMismatch { .. } => {
                CloudComputeFunctionsApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                CloudComputeFunctionsApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => {
                CloudComputeFunctionsApiErrorCode::AuthorizationDenied
            }
            Self::IdempotencyKeyReused { .. } => {
                CloudComputeFunctionsApiErrorCode::IdempotencyKeyReused
            }
            Self::InvalidPayloadDataClassLabel { .. } => {
                CloudComputeFunctionsApiErrorCode::PayloadDataClassInvalid
            }
            Self::Compute(error) => match cloud_compute_status_kind(error) {
                CloudComputeFunctionsApiStatusKind::BadRequest
                | CloudComputeFunctionsApiStatusKind::Unauthorized
                | CloudComputeFunctionsApiStatusKind::UnprocessableEntity => {
                    CloudComputeFunctionsApiErrorCode::ComputeInvalidRequest
                }
                CloudComputeFunctionsApiStatusKind::Forbidden => {
                    CloudComputeFunctionsApiErrorCode::ComputeForbidden
                }
                CloudComputeFunctionsApiStatusKind::NotFound => {
                    CloudComputeFunctionsApiErrorCode::ComputeNotFound
                }
                CloudComputeFunctionsApiStatusKind::Conflict => {
                    CloudComputeFunctionsApiErrorCode::ComputeConflict
                }
            },
        }
    }

    pub fn error_response(
        &self,
        request_id: impl Into<String>,
    ) -> CloudComputeFunctionsApiErrorResponse {
        CloudComputeFunctionsApiErrorResponse {
            error: CloudComputeFunctionsApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> CloudComputeFunctionsApiStatusKind {
        match self {
            Self::EmptyPrincipalId => CloudComputeFunctionsApiStatusKind::Unauthorized,
            Self::TenantMismatch { .. }
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationVerifierMissing
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. } => CloudComputeFunctionsApiStatusKind::Forbidden,
            Self::IdempotencyKeyReused { .. } => {
                CloudComputeFunctionsApiStatusKind::UnprocessableEntity
            }
            Self::Compute(error) => cloud_compute_status_kind(error),
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::EmptyPathFunctionId
            | Self::InvalidFunctionId { .. }
            | Self::FunctionKindMismatch { .. }
            | Self::FunctionIdMismatch { .. }
            | Self::InvalidPayloadDataClassLabel { .. } => {
                CloudComputeFunctionsApiStatusKind::BadRequest
            }
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::EmptyPathFunctionId => "Path function id is required",
            Self::InvalidFunctionId { .. } => "Function id must be a canonical Cloud resource id",
            Self::FunctionKindMismatch { .. } => "Function id must identify a function resource",
            Self::FunctionIdMismatch { .. } => "Path and body function ids must match",
            Self::TenantMismatch { .. } => {
                "Tenant header must match authenticated principal, function id, and request body"
            }
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationVerifierMissing => "Compute authorization verifier is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the requested Cloud Compute Functions surface"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::InvalidPayloadDataClassLabel { .. } => {
                "Payload data class must be a known data-class label"
            }
            Self::Compute(error) => cloud_compute_message(error),
        }
    }

    fn details(&self) -> Vec<CloudComputeFunctionsApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::EmptyPathFunctionId => vec![detail("path.function_id", "must be non-empty")],
            Self::InvalidFunctionId { .. } => vec![detail(
                "path.function_id",
                "must be a canonical oyatie:cloud function resource id",
            )],
            Self::FunctionKindMismatch { .. } => {
                vec![detail("path.function_id", "resource kind must be function")]
            }
            Self::FunctionIdMismatch { .. } => vec![detail(
                "function_id",
                "path function_id and body function_id must match",
            )],
            Self::TenantMismatch { .. } => vec![detail(
                "tenant_id",
                "header tenant, principal tenant, resource tenant, and body tenant_id must match",
            )],
            Self::EmptyAuthorizationDecisionId => vec![detail(
                "authorization.decision_id",
                "must be non-empty authorization evidence",
            )],
            Self::AuthorizationVerifierMissing => vec![detail(
                "authorization.verifier",
                "compute boundary must verify decision_id against trusted local authorization state",
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
                "authorization",
                "must bind tenant, principal, requested surface, and unexpired decision evidence",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::InvalidPayloadDataClassLabel { .. } => vec![detail(
                "body.payload_data_class",
                "must be a canonical data-class label",
            )],
            Self::Compute(error) => vec![detail("cloud_compute", cloud_compute_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CloudComputeFunctionsApiStatusKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}
