// ── Delete surface ────────────────────────────────────────────────────────────

/// Authorization surface constant for cluster teardown requests.
pub const CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE: &str = "cloud.compute.k8s.cluster.delete";

/// HTTP status codes for the cluster DELETE boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudComputeK8sClusterDeleteApiStatus {
    Accepted,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    UnprocessableEntity,
    ServiceUnavailable,
}

impl CloudComputeK8sClusterDeleteApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Accepted => 202,
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::NotFound => 404,
            Self::UnprocessableEntity => 422,
            Self::ServiceUnavailable => 503,
        }
    }
}

/// Inbound delete request boundary envelope.
///
/// There is no mutable body beyond the cluster identity in the path — the
/// caller identifies the cluster via `path_cluster_id` alone.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sClusterDeleteApiRequest {
    pub path_cluster_id: String, // data_class: INTERNAL_ONLY
    pub boundary: CloudComputeK8sApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CloudComputeK8sApiPrincipal, // data_class: INTERNAL_ONLY
    pub authorization: CloudComputeK8sApiAuthorization, // data_class: INTERNAL_ONLY
}

/// Successful delete acceptance response.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sClusterDeleteSuccessResponse {
    pub data: CloudComputeK8sClusterRecord, // data_class: INTERNAL_ONLY
    pub metadata: CloudComputeK8sMetadata,  // data_class: INTERNAL_ONLY
}

impl CloudComputeK8sClusterDeleteSuccessResponse {
    fn accepted(data: CloudComputeK8sClusterRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: CloudComputeK8sMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CloudComputeK8sDeleteOperationKey {
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub principal_id: String,    // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sDeleteCommand {
    pub operation_key: CloudComputeK8sDeleteOperationKey, // data_class: INTERNAL_ONLY
    pub resource_id: ResourceId,                          // data_class: INTERNAL_ONLY
    pub request_id: String,                               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sDeleteReceipt {
    pub cluster: KubernetesCluster, // data_class: INTERNAL_ONLY
    pub request_id: String,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudComputeK8sDeleteRepositoryError {
    ClusterNotFound,
    IdempotencyKeyReused {
        idempotency_key: String, // data_class: INTERNAL_ONLY
    },
    Unavailable,
}

pub trait CloudComputeK8sDeleteRepository {
    fn commit_deletion(
        &mut self,
        command: CloudComputeK8sDeleteCommand,
    ) -> Result<CloudComputeK8sDeleteReceipt, CloudComputeK8sDeleteRepositoryError>;
}

impl CloudComputeK8sApiError {
    /// Maps this error to the HTTP status for the cluster DELETE surface.
    pub fn cluster_delete_status(&self) -> CloudComputeK8sClusterDeleteApiStatus {
        match self.status_kind() {
            CloudComputeK8sApiStatusKind::BadRequest => {
                CloudComputeK8sClusterDeleteApiStatus::BadRequest
            }
            CloudComputeK8sApiStatusKind::Unauthorized => {
                CloudComputeK8sClusterDeleteApiStatus::Unauthorized
            }
            CloudComputeK8sApiStatusKind::Forbidden => {
                CloudComputeK8sClusterDeleteApiStatus::Forbidden
            }
            CloudComputeK8sApiStatusKind::NotFound => {
                CloudComputeK8sClusterDeleteApiStatus::NotFound
            }
            CloudComputeK8sApiStatusKind::Conflict => {
                // Conflict maps to 422 on the delete surface (no 409 variant).
                CloudComputeK8sClusterDeleteApiStatus::UnprocessableEntity
            }
            CloudComputeK8sApiStatusKind::UnprocessableEntity => {
                CloudComputeK8sClusterDeleteApiStatus::UnprocessableEntity
            }
            CloudComputeK8sApiStatusKind::ServiceUnavailable => {
                CloudComputeK8sClusterDeleteApiStatus::ServiceUnavailable
            }
        }
    }

    /// Convenience accessor — returns the numeric HTTP status code for delete.
    pub fn cluster_delete_status_code(&self) -> u16 {
        self.cluster_delete_status().code()
    }
}
