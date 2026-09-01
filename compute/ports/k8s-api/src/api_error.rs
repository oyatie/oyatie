impl CloudComputeK8sApiError {
    pub fn cluster_create_status(&self) -> CloudComputeK8sClusterCreateApiStatus {
        match self.status_kind() {
            CloudComputeK8sApiStatusKind::BadRequest => {
                CloudComputeK8sClusterCreateApiStatus::BadRequest
            }
            CloudComputeK8sApiStatusKind::Unauthorized => {
                CloudComputeK8sClusterCreateApiStatus::Unauthorized
            }
            CloudComputeK8sApiStatusKind::Forbidden => {
                CloudComputeK8sClusterCreateApiStatus::Forbidden
            }
            CloudComputeK8sApiStatusKind::NotFound => {
                CloudComputeK8sClusterCreateApiStatus::NotFound
            }
            CloudComputeK8sApiStatusKind::Conflict => {
                CloudComputeK8sClusterCreateApiStatus::Conflict
            }
            CloudComputeK8sApiStatusKind::UnprocessableEntity => {
                CloudComputeK8sClusterCreateApiStatus::UnprocessableEntity
            }
            CloudComputeK8sApiStatusKind::ServiceUnavailable => {
                CloudComputeK8sClusterCreateApiStatus::ServiceUnavailable
            }
        }
    }

    pub fn cluster_create_status_code(&self) -> u16 {
        self.cluster_create_status().code()
    }

    pub fn code(&self) -> CloudComputeK8sApiErrorCode {
        match self {
            Self::EmptyRequestId => CloudComputeK8sApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => CloudComputeK8sApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => CloudComputeK8sApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => CloudComputeK8sApiErrorCode::PrincipalIdEmpty,
            Self::EmptyPathClusterId => CloudComputeK8sApiErrorCode::PathClusterIdEmpty,
            Self::InvalidClusterId { .. } => CloudComputeK8sApiErrorCode::ClusterIdInvalid,
            Self::ClusterKindMismatch { .. } => CloudComputeK8sApiErrorCode::ClusterKindMismatch,
            Self::ClusterIdMismatch { .. } => CloudComputeK8sApiErrorCode::ClusterIdMismatch,
            Self::TenantMismatch { .. } => CloudComputeK8sApiErrorCode::TenantMismatch,
            Self::EmptyAuthorizationDecisionId => {
                CloudComputeK8sApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                CloudComputeK8sApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                CloudComputeK8sApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => CloudComputeK8sApiErrorCode::AuthorizationDenied,
            Self::IdempotencyKeyReused { .. } => CloudComputeK8sApiErrorCode::IdempotencyKeyReused,
            Self::InvalidClusterFlavorLabel { .. } => {
                CloudComputeK8sApiErrorCode::ClusterFlavorInvalid
            }
            Self::InvalidNodePoolFlavorClassLabel { .. } => {
                CloudComputeK8sApiErrorCode::NodePoolFlavorInvalid
            }
            Self::NodePoolSecurityGroupBindingMismatch { .. } => {
                CloudComputeK8sApiErrorCode::NodePoolSecurityGroupBindingInvalid
            }
            Self::InvalidResidencyLabel { .. } => CloudComputeK8sApiErrorCode::ResidencyInvalid,
            Self::InvalidDataClassLabel { .. } => CloudComputeK8sApiErrorCode::DataClassInvalid,
            Self::Compute(error) => match cloud_compute_status_kind(error) {
                CloudComputeK8sApiStatusKind::BadRequest
                | CloudComputeK8sApiStatusKind::Unauthorized
                | CloudComputeK8sApiStatusKind::UnprocessableEntity => {
                    CloudComputeK8sApiErrorCode::ComputeInvalidRequest
                }
                CloudComputeK8sApiStatusKind::Forbidden => {
                    CloudComputeK8sApiErrorCode::ComputeForbidden
                }
                CloudComputeK8sApiStatusKind::NotFound => {
                    CloudComputeK8sApiErrorCode::ComputeNotFound
                }
                CloudComputeK8sApiStatusKind::Conflict => {
                    CloudComputeK8sApiErrorCode::ComputeConflict
                }
                CloudComputeK8sApiStatusKind::ServiceUnavailable => {
                    CloudComputeK8sApiErrorCode::LifecycleRepositoryUnavailable
                }
            },
            Self::ClusterNotFound { .. } => CloudComputeK8sApiErrorCode::ComputeNotFound,
            Self::LifecycleRepositoryUnavailable => {
                CloudComputeK8sApiErrorCode::LifecycleRepositoryUnavailable
            }
            Self::LifecycleRepositoryInvariantViolation => {
                CloudComputeK8sApiErrorCode::LifecycleRepositoryUnavailable
            }
        }
    }

    pub fn error_response(&self, request_id: impl Into<String>) -> CloudComputeK8sApiErrorResponse {
        CloudComputeK8sApiErrorResponse {
            error: CloudComputeK8sApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: match self {
                    Self::LifecycleRepositoryUnavailable
                    | Self::LifecycleRepositoryInvariantViolation => Some(1),
                    _ => None,
                },
            },
        }
    }

    fn status_kind(&self) -> CloudComputeK8sApiStatusKind {
        match self {
            Self::EmptyPrincipalId => CloudComputeK8sApiStatusKind::Unauthorized,
            Self::TenantMismatch { .. }
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. }
            | Self::NodePoolSecurityGroupBindingMismatch { .. } => {
                CloudComputeK8sApiStatusKind::Forbidden
            }
            Self::IdempotencyKeyReused { .. } => CloudComputeK8sApiStatusKind::UnprocessableEntity,
            Self::ClusterNotFound { .. } => CloudComputeK8sApiStatusKind::NotFound,
            Self::LifecycleRepositoryUnavailable | Self::LifecycleRepositoryInvariantViolation => {
                CloudComputeK8sApiStatusKind::ServiceUnavailable
            }
            Self::Compute(error) => cloud_compute_status_kind(error),
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::EmptyPathClusterId
            | Self::InvalidClusterId { .. }
            | Self::ClusterKindMismatch { .. }
            | Self::ClusterIdMismatch { .. }
            | Self::InvalidClusterFlavorLabel { .. }
            | Self::InvalidNodePoolFlavorClassLabel { .. }
            | Self::InvalidResidencyLabel { .. }
            | Self::InvalidDataClassLabel { .. } => CloudComputeK8sApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::EmptyPathClusterId => "Path cluster id is required",
            Self::InvalidClusterId { .. } => "Cluster id must be a canonical Cloud resource id",
            Self::ClusterKindMismatch { .. } => "Cluster id must identify a k8s resource",
            Self::ClusterIdMismatch { .. } => "Path and body cluster ids must match",
            Self::TenantMismatch { .. } => {
                "Tenant header must match authenticated principal, cluster id, and request body"
            }
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal id"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the requested Cloud Compute Kubernetes surface"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::InvalidClusterFlavorLabel { .. } => "Kubernetes cluster flavor must be known",
            Self::InvalidNodePoolFlavorClassLabel { .. } => "Node-pool flavor class must be known",
            Self::NodePoolSecurityGroupBindingMismatch { .. } => {
                "Node-pool security group proof must match tenant, region, and subnet"
            }
            Self::InvalidResidencyLabel { .. } => "Residency must be a known residency label",
            Self::InvalidDataClassLabel { .. } => {
                "Request data_class must be a known privacy data class"
            }
            Self::Compute(error) => cloud_compute_message(error),
            Self::ClusterNotFound { .. } => "Kubernetes cluster was not found",
            Self::LifecycleRepositoryUnavailable | Self::LifecycleRepositoryInvariantViolation => {
                "Kubernetes lifecycle repository is temporarily unavailable"
            }
        }
    }

    fn details(&self) -> Vec<CloudComputeK8sApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::EmptyPathClusterId => vec![detail("path.cluster_id", "must be non-empty")],
            Self::InvalidClusterId { .. } => vec![detail(
                "path.cluster_id",
                "must be a canonical oyatie:cloud k8s resource id",
            )],
            Self::ClusterKindMismatch { .. } => {
                vec![detail("path.cluster_id", "resource kind must be k8s")]
            }
            Self::ClusterIdMismatch { .. } => vec![detail(
                "resource_id",
                "path cluster_id and body resource_id must match",
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
                "must resolve to a trusted compute-owned authorization verifier decision for the requested surface",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::InvalidClusterFlavorLabel { .. } => vec![detail(
                "body.flavor",
                "must be standard or high_availability",
            )],
            Self::InvalidNodePoolFlavorClassLabel { .. } => vec![detail(
                "body.node_pools[].flavor.class",
                "must be general_purpose, compute_optimized, memory_optimized, or gpu",
            )],
            Self::NodePoolSecurityGroupBindingMismatch { .. } => vec![detail(
                "body.node_pools[].security_groups",
                "each security group proof must match body tenant_id, region, and node-pool subnet_id",
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
            Self::ClusterNotFound { .. } => {
                vec![detail("path.cluster_id", "no cluster found with this id")]
            }
            Self::LifecycleRepositoryUnavailable | Self::LifecycleRepositoryInvariantViolation => {
                vec![detail(
                    "lifecycle_repository",
                    "retry after the repository becomes available",
                )]
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CloudComputeK8sApiStatusKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
    ServiceUnavailable,
}
