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

type CloudComputeK8sDeleteApiResult =
    Result<CloudComputeK8sClusterDeleteSuccessResponse, CloudComputeK8sApiError>;

/// Idempotency ledger for cluster delete requests.
///
/// Keyed on `(tenant_id, principal_id, "cloud.compute.k8s.cluster.delete",
/// idempotency_key)`. A replayed key with the same `path_cluster_id`
/// fingerprint returns the identical response without a second teardown.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sDeleteIdempotencyLedger {
    entries: BTreeMap<CloudComputeK8sIdempotencyLedgerKey, CloudComputeK8sDeleteLedgerEntry>, // data_class: INTERNAL_ONLY
    max_entries: usize, // data_class: INTERNAL_ONLY
}

impl Default for CloudComputeK8sDeleteIdempotencyLedger {
    fn default() -> Self {
        Self::with_max_entries(DEFAULT_K8S_DELETE_IDEMPOTENCY_LEDGER_MAX_ENTRIES)
    }
}

impl CloudComputeK8sDeleteIdempotencyLedger {
    pub fn with_max_entries(max_entries: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            max_entries: max_entries.max(1),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn remember(
        &mut self,
        key: CloudComputeK8sIdempotencyLedgerKey,
        entry: CloudComputeK8sDeleteLedgerEntry,
    ) {
        if self.entries.len() >= self.max_entries
            && let Some(evicted) = self.entries.keys().next().cloned()
        {
            self.entries.remove(&evicted);
        }
        self.entries.insert(key, entry);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudComputeK8sDeleteLedgerEntry {
    path_cluster_id: String,                // data_class: INTERNAL_ONLY
    result: CloudComputeK8sDeleteApiResult, // data_class: INTERNAL_ONLY
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
        }
    }

    /// Convenience accessor — returns the numeric HTTP status code for delete.
    pub fn cluster_delete_status_code(&self) -> u16 {
        self.cluster_delete_status().code()
    }
}
