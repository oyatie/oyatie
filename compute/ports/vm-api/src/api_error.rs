impl CloudComputeVmApiError {
    pub fn vm_create_status(&self) -> CloudComputeVmCreateApiStatus {
        match self.status_kind() {
            CloudComputeVmApiStatusKind::BadRequest => CloudComputeVmCreateApiStatus::BadRequest,
            CloudComputeVmApiStatusKind::Unauthorized => {
                CloudComputeVmCreateApiStatus::Unauthorized
            }
            CloudComputeVmApiStatusKind::Forbidden => CloudComputeVmCreateApiStatus::Forbidden,
            CloudComputeVmApiStatusKind::NotFound => CloudComputeVmCreateApiStatus::NotFound,
            CloudComputeVmApiStatusKind::Conflict => CloudComputeVmCreateApiStatus::Conflict,
            CloudComputeVmApiStatusKind::UnprocessableEntity => {
                CloudComputeVmCreateApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn vm_create_status_code(&self) -> u16 {
        self.vm_create_status().code()
    }

    pub fn code(&self) -> CloudComputeVmApiErrorCode {
        match self {
            Self::EmptyRequestId => CloudComputeVmApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => CloudComputeVmApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => CloudComputeVmApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => CloudComputeVmApiErrorCode::PrincipalIdEmpty,
            Self::EmptyPathInstanceId => CloudComputeVmApiErrorCode::PathInstanceIdEmpty,
            Self::InvalidInstanceId { .. } => CloudComputeVmApiErrorCode::InstanceIdInvalid,
            Self::InstanceKindMismatch { .. } => CloudComputeVmApiErrorCode::InstanceKindMismatch,
            Self::InstanceIdMismatch { .. } => CloudComputeVmApiErrorCode::InstanceIdMismatch,
            Self::TenantMismatch { .. } => CloudComputeVmApiErrorCode::TenantMismatch,
            Self::EmptyAuthorizationDecisionId => {
                CloudComputeVmApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                CloudComputeVmApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                CloudComputeVmApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => CloudComputeVmApiErrorCode::AuthorizationDenied,
            Self::IdempotencyKeyReused { .. } => CloudComputeVmApiErrorCode::IdempotencyKeyReused,
            Self::InvalidFlavorClassLabel { .. } => CloudComputeVmApiErrorCode::FlavorClassInvalid,
            Self::SecurityGroupBindingMismatch { .. } => {
                CloudComputeVmApiErrorCode::SecurityGroupBindingInvalid
            }
            Self::IamRoleBindingMismatch { .. } => {
                CloudComputeVmApiErrorCode::IamRoleBindingInvalid
            }
            Self::InvalidResidencyLabel { .. } => CloudComputeVmApiErrorCode::ResidencyInvalid,
            Self::InvalidDataClassLabel { .. } => CloudComputeVmApiErrorCode::DataClassInvalid,
            Self::Compute(error) => match cloud_compute_status_kind(error) {
                CloudComputeVmApiStatusKind::BadRequest => {
                    CloudComputeVmApiErrorCode::ComputeInvalidRequest
                }
                CloudComputeVmApiStatusKind::Unauthorized => {
                    CloudComputeVmApiErrorCode::ComputeInvalidRequest
                }
                CloudComputeVmApiStatusKind::Forbidden => {
                    CloudComputeVmApiErrorCode::ComputeForbidden
                }
                CloudComputeVmApiStatusKind::NotFound => {
                    CloudComputeVmApiErrorCode::ComputeNotFound
                }
                CloudComputeVmApiStatusKind::Conflict => {
                    CloudComputeVmApiErrorCode::ComputeConflict
                }
                CloudComputeVmApiStatusKind::UnprocessableEntity => {
                    CloudComputeVmApiErrorCode::ComputeInvalidRequest
                }
            },
        }
    }

    pub fn error_response(&self, request_id: impl Into<String>) -> CloudComputeVmApiErrorResponse {
        CloudComputeVmApiErrorResponse {
            error: CloudComputeVmApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> CloudComputeVmApiStatusKind {
        match self {
            Self::EmptyPrincipalId => CloudComputeVmApiStatusKind::Unauthorized,
            Self::TenantMismatch { .. }
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. } => CloudComputeVmApiStatusKind::Forbidden,
            Self::IdempotencyKeyReused { .. } => CloudComputeVmApiStatusKind::UnprocessableEntity,
            Self::Compute(error) => cloud_compute_status_kind(error),
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::EmptyPathInstanceId
            | Self::InvalidInstanceId { .. }
            | Self::InstanceKindMismatch { .. }
            | Self::InstanceIdMismatch { .. }
            | Self::InvalidFlavorClassLabel { .. }
            | Self::InvalidResidencyLabel { .. }
            | Self::InvalidDataClassLabel { .. } => CloudComputeVmApiStatusKind::BadRequest,
            Self::SecurityGroupBindingMismatch { .. } | Self::IamRoleBindingMismatch { .. } => {
                CloudComputeVmApiStatusKind::Forbidden
            }
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::EmptyPathInstanceId => "Path instance id is required",
            Self::InvalidInstanceId { .. } => "Instance id must be a canonical Cloud resource id",
            Self::InstanceKindMismatch { .. } => "Instance id must identify an instance resource",
            Self::InstanceIdMismatch { .. } => "Path and body instance ids must match",
            Self::TenantMismatch { .. } => {
                "Tenant header must match authenticated principal, instance id, and request body"
            }
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal"
            }
            Self::AuthorizationDenied { .. } => {
                "Trusted authorization verifier does not allow the requested Cloud Compute VM surface"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::InvalidFlavorClassLabel { .. } => "Flavor class must be known",
            Self::SecurityGroupBindingMismatch { .. } => {
                "Security group proof must match the VM tenant, region, and VPC"
            }
            Self::IamRoleBindingMismatch { .. } => {
                "IAM role proof must match the VM tenant, region, and VPC"
            }
            Self::InvalidResidencyLabel { .. } => "Residency must be a known residency label",
            Self::InvalidDataClassLabel { .. } => {
                "Request data_class must be a known privacy data class"
            }
            Self::Compute(error) => cloud_compute_message(error),
        }
    }

    fn details(&self) -> Vec<CloudComputeVmApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::EmptyPathInstanceId => vec![detail("path.instance_id", "must be non-empty")],
            Self::InvalidInstanceId { .. } => vec![detail(
                "path.instance_id",
                "must be a canonical oyatie:cloud instance resource id",
            )],
            Self::InstanceKindMismatch { .. } => {
                vec![detail("path.instance_id", "resource kind must be instance")]
            }
            Self::InstanceIdMismatch { .. } => vec![detail(
                "resource_id",
                "path instance_id and body resource_id must match",
            )],
            Self::TenantMismatch { .. } => vec![detail(
                "tenant_id",
                "header tenant, principal tenant, resource tenant, and body tenant_id must match",
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
                "authorization.decision_id",
                "must resolve to a non-expired compute-owned verifier proof for the requested Cloud Compute VM surface",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::InvalidFlavorClassLabel { .. } => vec![detail(
                "body.flavor.class",
                "must be general_purpose, compute_optimized, memory_optimized, or gpu",
            )],
            Self::SecurityGroupBindingMismatch { .. } => vec![detail(
                "body.security_groups",
                "each security group proof must match body tenant_id, region, and vpc_id",
            )],
            Self::IamRoleBindingMismatch { .. } => vec![detail(
                "body.iam_role",
                "IAM role proof must match body tenant_id, region, and vpc_id",
            )],
            Self::InvalidResidencyLabel { .. } => vec![detail(
                "body.residency",
                "must be a canonical residency label",
            )],
            Self::InvalidDataClassLabel { .. } => vec![detail(
                "body.data_class",
                "must be a canonical privacy data-class label",
            )],
            Self::Compute(error) => vec![detail("cloud_compute", cloud_compute_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CloudComputeVmApiStatusKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}
