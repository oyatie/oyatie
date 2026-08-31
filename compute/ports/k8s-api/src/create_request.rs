#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sClusterCreateRequest {
    pub resource_id: String,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub region: String,                // data_class: PUBLIC
    pub flavor: String,                // data_class: PUBLIC
    pub control_plane_version: String, // data_class: PUBLIC
    pub control_plane_private: bool,   // data_class: PUBLIC
    pub node_pools: Vec<CloudComputeK8sNodePoolCreateRequest>, // data_class: INTERNAL_ONLY
    pub quota: CloudComputeK8sQuotaEnvelope, // data_class: INTERNAL_ONLY
    pub residency: String,             // data_class: INTERNAL_ONLY
    pub data_class: String,            // data_class: PUBLIC
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sNodePoolCreateRequest {
    pub id: String,                                            // data_class: INTERNAL_ONLY
    pub az: String,                                            // data_class: PUBLIC
    pub cell_id: String,                                       // data_class: PUBLIC
    pub subnet_id: String,                                     // data_class: INTERNAL_ONLY
    pub security_groups: Vec<CloudComputeK8sSecurityGroupRef>, // data_class: INTERNAL_ONLY
    pub flavor: CloudComputeK8sNodePoolFlavorSpec,             // data_class: PUBLIC
    pub min_nodes: u32,                                        // data_class: PUBLIC
    pub max_nodes: u32,                                        // data_class: PUBLIC
    pub autoscaling_enabled: bool,                             // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sSecurityGroupRef {
    pub value: String,     // data_class: INTERNAL_ONLY
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    pub region: String,    // data_class: PUBLIC
    pub subnet_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sNodePoolFlavorSpec {
    pub class: String,     // data_class: PUBLIC
    pub vcpu: u32,         // data_class: PUBLIC
    pub memory_gb: u32,    // data_class: PUBLIC
    pub gpu_count: u32,    // data_class: PUBLIC
    pub local_ssd_gb: u32, // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sQuotaEnvelope {
    pub vcpu_limit: u32,           // data_class: INTERNAL_ONLY
    pub memory_gb_limit: u32,      // data_class: INTERNAL_ONLY
    pub gpu_limit: u32,            // data_class: INTERNAL_ONLY
    pub local_ssd_gb_limit: u32,   // data_class: INTERNAL_ONLY
    pub current_vcpu: u32,         // data_class: INTERNAL_ONLY
    pub current_memory_gb: u32,    // data_class: INTERNAL_ONLY
    pub current_gpu: u32,          // data_class: INTERNAL_ONLY
    pub current_local_ssd_gb: u32, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sClusterCreateApiRequest {
    pub path_cluster_id: String, // data_class: INTERNAL_ONLY
    pub boundary: CloudComputeK8sApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CloudComputeK8sApiPrincipal, // data_class: INTERNAL_ONLY
    pub authorization: CloudComputeK8sApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: CloudComputeK8sClusterCreateRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sCreateIdempotencyLedger {
    entries: BTreeMap<CloudComputeK8sIdempotencyLedgerKey, CloudComputeK8sCreateLedgerEntry>, // data_class: INTERNAL_ONLY
    max_entries: usize, // data_class: INTERNAL_ONLY
}

impl Default for CloudComputeK8sCreateIdempotencyLedger {
    fn default() -> Self {
        Self::with_max_entries(DEFAULT_K8S_CREATE_IDEMPOTENCY_LEDGER_MAX_ENTRIES)
    }
}

impl CloudComputeK8sCreateIdempotencyLedger {
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
        entry: CloudComputeK8sCreateLedgerEntry,
    ) {
        if self.entries.len() >= self.max_entries
            && let Some(evicted) = self.entries.keys().next().cloned()
        {
            self.entries.remove(&evicted);
        }
        self.entries.insert(key, entry);
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CloudComputeK8sIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudComputeK8sCreateLedgerEntry {
    fingerprint: CloudComputeK8sRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CloudComputeK8sCreateApiResult,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudComputeK8sRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

type CloudComputeK8sCreateApiResult =
    Result<CloudComputeK8sClusterCreateSuccessResponse, CloudComputeK8sApiError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sClusterCreateSuccessResponse {
    pub data: CloudComputeK8sClusterRecord, // data_class: INTERNAL_ONLY
    pub metadata: CloudComputeK8sMetadata,  // data_class: INTERNAL_ONLY
}

impl CloudComputeK8sClusterCreateSuccessResponse {
    pub fn created(data: CloudComputeK8sClusterRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: CloudComputeK8sMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sMetadata {
    pub request_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sClusterRecord {
    pub resource_id: String,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub region: String,                // data_class: PUBLIC
    pub flavor: String,                // data_class: PUBLIC
    pub control_plane_version: String, // data_class: PUBLIC
    pub control_plane_private: bool,   // data_class: PUBLIC
    pub node_pool_count: u32,          // data_class: PUBLIC
    pub residency: String,             // data_class: INTERNAL_ONLY
    pub state: String,                 // data_class: PUBLIC
    pub data_class: String,            // data_class: PUBLIC
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub schema_version: u32,           // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sApiErrorResponse {
    pub error: CloudComputeK8sApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sApiErrorBody {
    pub code: String,                                // data_class: INTERNAL_ONLY
    pub message: String,                             // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,           // data_class: INTERNAL_ONLY
    pub request_id: String,                          // data_class: INTERNAL_ONLY
    pub details: Vec<CloudComputeK8sApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudComputeK8sApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    EmptyPathClusterId,
    InvalidClusterId {
        cluster_id: String, // data_class: INTERNAL_ONLY
    },
    ClusterKindMismatch {
        cluster_id: String, // data_class: INTERNAL_ONLY
        kind_label: String, // data_class: PUBLIC
    },
    ClusterIdMismatch {
        path_cluster_id: String,  // data_class: INTERNAL_ONLY
        body_resource_id: String, // data_class: INTERNAL_ONLY
    },
    TenantMismatch {
        header_tenant_id: String,    // data_class: INTERNAL_ONLY
        principal_tenant_id: String, // data_class: INTERNAL_ONLY
        resource_tenant_id: String,  // data_class: INTERNAL_ONLY
        body_tenant_id: String,      // data_class: INTERNAL_ONLY
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
    InvalidClusterFlavorLabel {
        flavor: String, // data_class: PUBLIC
    },
    InvalidNodePoolFlavorClassLabel {
        class: String, // data_class: PUBLIC
    },
    NodePoolSecurityGroupBindingMismatch {
        node_pool_id: String,   // data_class: INTERNAL_ONLY
        security_group: String, // data_class: INTERNAL_ONLY
        tenant_id: String,      // data_class: INTERNAL_ONLY
        region: String,         // data_class: PUBLIC
        subnet_id: String,      // data_class: INTERNAL_ONLY
    },
    InvalidResidencyLabel {
        residency: String, // data_class: INTERNAL_ONLY
    },
    InvalidDataClassLabel {
        data_class: String, // data_class: PUBLIC
    },
    Compute(CloudComputeError), // data_class: INTERNAL_ONLY
    /// The cluster identified by `path_cluster_id` does not exist in the
    /// catalog. Used exclusively by the delete surface.
    ClusterNotFound {
        cluster_id: String, // data_class: INTERNAL_ONLY
    },
}
