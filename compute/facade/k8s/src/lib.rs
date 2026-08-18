//! Cloud Compute managed Kubernetes API boundary for cluster lifecycle.
//!
//! This crate owns request boundary normalization, authorization proof checks,
//! idempotent create and delete semantics, and tenant-safe Kubernetes cluster
//! metadata projection around the Cloud compute kernel. Cluster reconciliation
//! and provider adapters live behind later adapter crates.
//!
//! # Surfaces
//!
//! - [`CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE`] — `cloud.compute.k8s.cluster.create`
//! - [`CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE`] — `cloud.compute.k8s.cluster.delete`

use std::collections::BTreeMap;

use compute_domain::{
    CloudComputeCatalog, CloudComputeError, ComputeFlavorSpec, ComputeQuotaEnvelope, ComputeRepo,
    KubernetesCluster, KubernetesClusterCreate, KubernetesClusterState, KubernetesNodePoolCreate,
};
use compute_resource::{InstanceFlavor, K8sFlavor, ResourceId};
use network_residency::{ResidencyClass, parse_residency_class_label};
use oya_data_boundary_kernel::{DataClass, parse_data_class_label};

pub const CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE: &str = "cloud.compute.k8s.cluster.create";
const DEFAULT_K8S_CREATE_IDEMPOTENCY_LEDGER_MAX_ENTRIES: usize = 1024;
const DEFAULT_K8S_DELETE_IDEMPOTENCY_LEDGER_MAX_ENTRIES: usize = 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudComputeK8sClusterCreateApiStatus {
    Created,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

impl CloudComputeK8sClusterCreateApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Created => 201,
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::NotFound => 404,
            Self::Conflict => 409,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudComputeK8sApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    PathClusterIdEmpty,
    ClusterIdInvalid,
    ClusterKindMismatch,
    ClusterIdMismatch,
    TenantMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    IdempotencyKeyReused,
    ClusterFlavorInvalid,
    NodePoolFlavorInvalid,
    NodePoolSecurityGroupBindingInvalid,
    ResidencyInvalid,
    DataClassInvalid,
    ComputeInvalidRequest,
    ComputeForbidden,
    ComputeNotFound,
    ComputeConflict,
}

impl CloudComputeK8sApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "CLOUD_COMPUTE_K8S_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "CLOUD_COMPUTE_K8S_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "CLOUD_COMPUTE_K8S_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "CLOUD_COMPUTE_K8S_PRINCIPAL_ID_EMPTY",
            Self::PathClusterIdEmpty => "CLOUD_COMPUTE_K8S_PATH_CLUSTER_ID_EMPTY",
            Self::ClusterIdInvalid => "CLOUD_COMPUTE_K8S_CLUSTER_ID_INVALID",
            Self::ClusterKindMismatch => "CLOUD_COMPUTE_K8S_CLUSTER_KIND_MISMATCH",
            Self::ClusterIdMismatch => "CLOUD_COMPUTE_K8S_CLUSTER_ID_MISMATCH",
            Self::TenantMismatch => "CLOUD_COMPUTE_K8S_TENANT_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => {
                "CLOUD_COMPUTE_K8S_AUTHORIZATION_DECISION_ID_EMPTY"
            }
            Self::AuthorizationTenantMismatch => "CLOUD_COMPUTE_K8S_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => {
                "CLOUD_COMPUTE_K8S_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationDenied => "CLOUD_COMPUTE_K8S_AUTHORIZATION_DENIED",
            Self::IdempotencyKeyReused => "CLOUD_COMPUTE_K8S_IDEMPOTENCY_KEY_REUSED",
            Self::ClusterFlavorInvalid => "CLOUD_COMPUTE_K8S_CLUSTER_FLAVOR_INVALID",
            Self::NodePoolFlavorInvalid => "CLOUD_COMPUTE_K8S_NODE_POOL_FLAVOR_INVALID",
            Self::NodePoolSecurityGroupBindingInvalid => {
                "CLOUD_COMPUTE_K8S_NODE_POOL_SECURITY_GROUP_BINDING_INVALID"
            }
            Self::ResidencyInvalid => "CLOUD_COMPUTE_K8S_RESIDENCY_INVALID",
            Self::DataClassInvalid => "CLOUD_COMPUTE_K8S_DATA_CLASS_INVALID",
            Self::ComputeInvalidRequest => "CLOUD_COMPUTE_K8S_INVALID_REQUEST",
            Self::ComputeForbidden => "CLOUD_COMPUTE_K8S_FORBIDDEN",
            Self::ComputeNotFound => "CLOUD_COMPUTE_K8S_NOT_FOUND",
            Self::ComputeConflict => "CLOUD_COMPUTE_K8S_CONFLICT",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
    pub proof: Option<CloudComputeK8sApiAuthorizationProof>, // data_class: INTERNAL_ONLY
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sApiAuthorizationProof {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub surface: String,               // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub verified: bool,                // data_class: INTERNAL_ONLY
    pub issued_at_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
    pub expires_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}
pub trait CloudComputeK8sAuthorizationVerifier {
    fn verified_authorization_proof(
        &self,
        decision_id: &str,
    ) -> Option<&CloudComputeK8sApiAuthorizationProof>;
    fn evaluation_epoch_seconds(&self) -> u64;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sTrustedAuthorizationVerifier {
    evaluation_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    proofs_by_decision_id: BTreeMap<String, CloudComputeK8sApiAuthorizationProof>, // data_class: INTERNAL_ONLY
}

impl Default for CloudComputeK8sTrustedAuthorizationVerifier {
    fn default() -> Self {
        Self {
            evaluation_epoch_seconds: u64::MAX,
            proofs_by_decision_id: BTreeMap::new(),
        }
    }
}

impl CloudComputeK8sTrustedAuthorizationVerifier {
    pub fn new(evaluation_epoch_seconds: u64) -> Self {
        Self {
            evaluation_epoch_seconds,
            proofs_by_decision_id: BTreeMap::new(),
        }
    }

    pub fn trust_authorization_proof(
        &mut self,
        proof: CloudComputeK8sApiAuthorizationProof,
    ) -> Option<CloudComputeK8sApiAuthorizationProof> {
        self.proofs_by_decision_id
            .insert(proof.decision_id.clone(), proof)
    }

    pub fn trust_authorization_proof_for_decision(
        &mut self,
        decision_id: impl Into<String>,
        proof: CloudComputeK8sApiAuthorizationProof,
    ) -> Option<CloudComputeK8sApiAuthorizationProof> {
        self.proofs_by_decision_id.insert(decision_id.into(), proof)
    }

    pub fn with_authorization_proof(mut self, proof: CloudComputeK8sApiAuthorizationProof) -> Self {
        self.trust_authorization_proof(proof);
        self
    }

    pub fn len(&self) -> usize {
        self.proofs_by_decision_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.proofs_by_decision_id.is_empty()
    }
}

impl CloudComputeK8sAuthorizationVerifier for CloudComputeK8sTrustedAuthorizationVerifier {
    fn verified_authorization_proof(
        &self,
        decision_id: &str,
    ) -> Option<&CloudComputeK8sApiAuthorizationProof> {
        self.proofs_by_decision_id.get(decision_id)
    }

    fn evaluation_epoch_seconds(&self) -> u64 {
        self.evaluation_epoch_seconds
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct CloudComputeK8sFailClosedAuthorizationVerifier;

impl CloudComputeK8sAuthorizationVerifier for CloudComputeK8sFailClosedAuthorizationVerifier {
    fn verified_authorization_proof(
        &self,
        _decision_id: &str,
    ) -> Option<&CloudComputeK8sApiAuthorizationProof> {
        None
    }

    fn evaluation_epoch_seconds(&self) -> u64 {
        u64::MAX
    }
}

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
            },
            Self::ClusterNotFound { .. } => CloudComputeK8sApiErrorCode::ComputeNotFound,
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
                retry_after_seconds: None,
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
                "must be a canonical oya:cloud k8s resource id",
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
}

pub fn validate_cloud_compute_k8s_cluster_create_request(
    request: &CloudComputeK8sClusterCreateApiRequest,
) -> Result<ResourceId, CloudComputeK8sApiError> {
    validate_cloud_compute_k8s_cluster_create_request_with_authorization_verifier(
        request,
        &CloudComputeK8sFailClosedAuthorizationVerifier,
    )
}

pub fn validate_cloud_compute_k8s_cluster_create_request_with_authorization_verifier(
    request: &CloudComputeK8sClusterCreateApiRequest,
    authorization_verifier: &impl CloudComputeK8sAuthorizationVerifier,
) -> Result<ResourceId, CloudComputeK8sApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_cluster_id(&request.path_cluster_id, &request.body.resource_id)?;
    let resource_id = validate_cluster_resource_id(&request.path_cluster_id)?;
    validate_tenant_binding(
        &request.boundary,
        &request.principal,
        &resource_id,
        &request.body.tenant_id,
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization.decision_id,
        CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE,
        authorization_verifier,
    )?;
    Ok(resource_id)
}

pub fn create_cloud_compute_k8s_cluster_from_api(
    catalog: &mut CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeK8sCreateIdempotencyLedger,
    request: CloudComputeK8sClusterCreateApiRequest,
) -> Result<CloudComputeK8sClusterCreateSuccessResponse, CloudComputeK8sApiError> {
    validate_cloud_compute_k8s_cluster_create_request(&request)?;
    let input = cluster_create_input(&request.body)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE,
    );
    let fingerprint = cluster_create_fingerprint_for(&request.path_cluster_id, &input);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudComputeK8sApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let result = catalog
        .create_kubernetes_cluster(input)
        .map_err(CloudComputeK8sApiError::Compute)
        .map(|cluster| {
            CloudComputeK8sClusterCreateSuccessResponse::created(
                cluster_record(cluster),
                request_id,
            )
        });
    idempotency_ledger.remember(
        key,
        CloudComputeK8sCreateLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

pub fn create_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
    catalog: &mut CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeK8sCreateIdempotencyLedger,
    request: CloudComputeK8sClusterCreateApiRequest,
    authorization_verifier: &impl CloudComputeK8sAuthorizationVerifier,
) -> Result<CloudComputeK8sClusterCreateSuccessResponse, CloudComputeK8sApiError> {
    validate_cloud_compute_k8s_cluster_create_request_with_authorization_verifier(
        &request,
        authorization_verifier,
    )?;
    let input = cluster_create_input(&request.body)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE,
    );
    let fingerprint = cluster_create_fingerprint_for(&request.path_cluster_id, &input);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudComputeK8sApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let result = catalog
        .create_kubernetes_cluster(input)
        .map_err(CloudComputeK8sApiError::Compute)
        .map(|cluster| {
            CloudComputeK8sClusterCreateSuccessResponse::created(
                cluster_record(cluster),
                request_id,
            )
        });
    idempotency_ledger.remember(
        key,
        CloudComputeK8sCreateLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

/// Stable planned entrypoint for `cloud.compute.k8s.cluster.create`.
///
/// The implementation delegates to the explicit API-boundary function so the
/// plan symbol remains stable without adding a second validation path.
pub fn create_cluster(
    catalog: &mut CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeK8sCreateIdempotencyLedger,
    request: CloudComputeK8sClusterCreateApiRequest,
) -> Result<CloudComputeK8sClusterCreateSuccessResponse, CloudComputeK8sApiError> {
    create_cloud_compute_k8s_cluster_from_api(catalog, idempotency_ledger, request)
}

pub fn create_cluster_with_authorization_verifier(
    catalog: &mut CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeK8sCreateIdempotencyLedger,
    request: CloudComputeK8sClusterCreateApiRequest,
    authorization_verifier: &impl CloudComputeK8sAuthorizationVerifier,
) -> Result<CloudComputeK8sClusterCreateSuccessResponse, CloudComputeK8sApiError> {
    create_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
        catalog,
        idempotency_ledger,
        request,
        authorization_verifier,
    )
}

fn validate_boundary(
    boundary: &CloudComputeK8sApiBoundaryContext,
) -> Result<(), CloudComputeK8sApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(CloudComputeK8sApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(CloudComputeK8sApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(CloudComputeK8sApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_cluster_id(
    path_cluster_id: &str,
    body_resource_id: &str,
) -> Result<(), CloudComputeK8sApiError> {
    if path_cluster_id.trim().is_empty() {
        return Err(CloudComputeK8sApiError::EmptyPathClusterId);
    }
    if path_cluster_id != body_resource_id {
        return Err(CloudComputeK8sApiError::ClusterIdMismatch {
            path_cluster_id: path_cluster_id.to_string(),
            body_resource_id: body_resource_id.to_string(),
        });
    }
    Ok(())
}

fn validate_cluster_resource_id(value: &str) -> Result<ResourceId, CloudComputeK8sApiError> {
    let id = ResourceId::new(value.to_string()).map_err(|_| {
        CloudComputeK8sApiError::InvalidClusterId {
            cluster_id: value.to_string(),
        }
    })?;
    let kind_label = id
        .kind_label()
        .map_err(|_| CloudComputeK8sApiError::InvalidClusterId {
            cluster_id: value.to_string(),
        })?;
    if kind_label != "k8s" {
        return Err(CloudComputeK8sApiError::ClusterKindMismatch {
            cluster_id: value.to_string(),
            kind_label,
        });
    }
    Ok(id)
}

fn validate_tenant_binding(
    boundary: &CloudComputeK8sApiBoundaryContext,
    principal: &CloudComputeK8sApiPrincipal,
    resource_id: &ResourceId,
    body_tenant_id: &str,
) -> Result<(), CloudComputeK8sApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(CloudComputeK8sApiError::EmptyPrincipalId);
    }
    let resource_tenant_id =
        resource_id
            .tenant_id()
            .map_err(|_| CloudComputeK8sApiError::InvalidClusterId {
                cluster_id: resource_id.value.clone(),
            })?;
    if boundary.tenant_id != principal.tenant_id
        || boundary.tenant_id != resource_tenant_id
        || boundary.tenant_id != body_tenant_id
    {
        return Err(CloudComputeK8sApiError::TenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
            resource_tenant_id,
            body_tenant_id: body_tenant_id.to_string(),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &CloudComputeK8sApiPrincipal,
    decision_id: &str,
    surface: &str,
    authorization_verifier: &impl CloudComputeK8sAuthorizationVerifier,
) -> Result<(), CloudComputeK8sApiError> {
    if decision_id.trim().is_empty() {
        return Err(CloudComputeK8sApiError::EmptyAuthorizationDecisionId);
    }
    validate_authorization_proof(principal, decision_id, surface, authorization_verifier)
}

fn validate_authorization_proof(
    principal: &CloudComputeK8sApiPrincipal,
    decision_id: &str,
    surface: &str,
    authorization_verifier: &impl CloudComputeK8sAuthorizationVerifier,
) -> Result<(), CloudComputeK8sApiError> {
    let Some(proof) = authorization_verifier.verified_authorization_proof(decision_id) else {
        return Err(CloudComputeK8sApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    };
    let evaluation_epoch_seconds = authorization_verifier.evaluation_epoch_seconds();
    if !proof.verified
        || proof.tenant_id != principal.tenant_id
        || proof.principal_id != principal.principal_id
        || proof.surface != surface
        || proof.decision_id != decision_id
        || proof.issued_at_epoch_seconds >= proof.expires_at_epoch_seconds
        || evaluation_epoch_seconds < proof.issued_at_epoch_seconds
        || evaluation_epoch_seconds >= proof.expires_at_epoch_seconds
    {
        return Err(CloudComputeK8sApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn cluster_create_input(
    body: &CloudComputeK8sClusterCreateRequest,
) -> Result<KubernetesClusterCreate, CloudComputeK8sApiError> {
    Ok(KubernetesClusterCreate {
        resource_id: body.resource_id.clone(),
        tenant_id: body.tenant_id.clone(),
        region: body.region.clone(),
        flavor: parse_k8s_flavor(body.flavor.clone())?,
        control_plane_version: body.control_plane_version.clone(),
        control_plane_private: body.control_plane_private,
        node_pools: node_pool_create_inputs(body)?,
        quota: quota_envelope(body.quota),
        residency: parse_api_residency(body.residency.clone())?,
        state: KubernetesClusterState::Creating,
        data_class: parse_api_data_class(body.data_class.clone())?,
        created_at_epoch_seconds: body.created_at_epoch_seconds,
    })
}

fn node_pool_create_inputs(
    body: &CloudComputeK8sClusterCreateRequest,
) -> Result<Vec<KubernetesNodePoolCreate>, CloudComputeK8sApiError> {
    body.node_pools
        .iter()
        .map(|pool| {
            Ok(KubernetesNodePoolCreate {
                id: pool.id.clone(),
                az: pool.az.clone(),
                cell_id: pool.cell_id.clone(),
                subnet_id: pool.subnet_id.clone(),
                security_groups: security_group_values(body, pool)?,
                flavor: flavor_spec(pool.flavor.clone())?,
                min_nodes: pool.min_nodes,
                max_nodes: pool.max_nodes,
                autoscaling_enabled: pool.autoscaling_enabled,
            })
        })
        .collect()
}

fn security_group_values(
    body: &CloudComputeK8sClusterCreateRequest,
    pool: &CloudComputeK8sNodePoolCreateRequest,
) -> Result<Vec<String>, CloudComputeK8sApiError> {
    let mut values = Vec::with_capacity(pool.security_groups.len());
    for group in &pool.security_groups {
        if group.tenant_id != body.tenant_id
            || group.region != body.region
            || group.subnet_id != pool.subnet_id
        {
            return Err(
                CloudComputeK8sApiError::NodePoolSecurityGroupBindingMismatch {
                    node_pool_id: pool.id.clone(),
                    security_group: group.value.clone(),
                    tenant_id: group.tenant_id.clone(),
                    region: group.region.clone(),
                    subnet_id: group.subnet_id.clone(),
                },
            );
        }
        values.push(group.value.clone());
    }
    Ok(values)
}

fn flavor_spec(
    input: CloudComputeK8sNodePoolFlavorSpec,
) -> Result<ComputeFlavorSpec, CloudComputeK8sApiError> {
    Ok(ComputeFlavorSpec {
        class: parse_node_pool_flavor_class(input.class)?,
        vcpu: input.vcpu,
        memory_gb: input.memory_gb,
        gpu_count: input.gpu_count,
        local_ssd_gb: input.local_ssd_gb,
    })
}

fn quota_envelope(input: CloudComputeK8sQuotaEnvelope) -> ComputeQuotaEnvelope {
    ComputeQuotaEnvelope {
        vcpu_limit: input.vcpu_limit,
        memory_gb_limit: input.memory_gb_limit,
        gpu_limit: input.gpu_limit,
        local_ssd_gb_limit: input.local_ssd_gb_limit,
        current_vcpu: input.current_vcpu,
        current_memory_gb: input.current_memory_gb,
        current_gpu: input.current_gpu,
        current_local_ssd_gb: input.current_local_ssd_gb,
    }
}

fn parse_k8s_flavor(label: String) -> Result<K8sFlavor, CloudComputeK8sApiError> {
    match label.as_str() {
        "standard" => Ok(K8sFlavor::Standard),
        "high_availability" => Ok(K8sFlavor::HighAvailability),
        _ => Err(CloudComputeK8sApiError::InvalidClusterFlavorLabel { flavor: label }),
    }
}

fn k8s_flavor_label(flavor: K8sFlavor) -> &'static str {
    match flavor {
        K8sFlavor::Standard => "standard",
        K8sFlavor::HighAvailability => "high_availability",
    }
}

fn parse_node_pool_flavor_class(label: String) -> Result<InstanceFlavor, CloudComputeK8sApiError> {
    match label.as_str() {
        "general_purpose" => Ok(InstanceFlavor::GeneralPurpose),
        "compute_optimized" => Ok(InstanceFlavor::ComputeOptimized),
        "memory_optimized" => Ok(InstanceFlavor::MemoryOptimized),
        "gpu" => Ok(InstanceFlavor::Gpu),
        _ => Err(CloudComputeK8sApiError::InvalidNodePoolFlavorClassLabel { class: label }),
    }
}

fn node_pool_flavor_class_label(class: InstanceFlavor) -> &'static str {
    match class {
        InstanceFlavor::GeneralPurpose => "general_purpose",
        InstanceFlavor::ComputeOptimized => "compute_optimized",
        InstanceFlavor::MemoryOptimized => "memory_optimized",
        InstanceFlavor::Gpu => "gpu",
    }
}

fn parse_api_residency(label: String) -> Result<ResidencyClass, CloudComputeK8sApiError> {
    parse_residency_class_label(&label)
        .ok_or(CloudComputeK8sApiError::InvalidResidencyLabel { residency: label })
}

fn parse_api_data_class(label: String) -> Result<DataClass, CloudComputeK8sApiError> {
    parse_data_class_label(&label)
        .ok_or(CloudComputeK8sApiError::InvalidDataClassLabel { data_class: label })
}

fn cluster_state_label(state: KubernetesClusterState) -> &'static str {
    match state {
        KubernetesClusterState::Creating => "creating",
        KubernetesClusterState::Ready => "ready",
        KubernetesClusterState::Reconciling => "reconciling",
        KubernetesClusterState::Draining => "draining",
        KubernetesClusterState::Deleted => "deleted",
    }
}

fn idempotency_key_for(
    boundary: &CloudComputeK8sApiBoundaryContext,
    principal: &CloudComputeK8sApiPrincipal,
    surface: &str,
) -> CloudComputeK8sIdempotencyLedgerKey {
    CloudComputeK8sIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn cluster_create_fingerprint_for(
    path_cluster_id: &str,
    input: &KubernetesClusterCreate,
) -> CloudComputeK8sRequestFingerprint {
    let mut pools = input.node_pools.clone();
    pools.sort_by(|left, right| left.id.cmp(&right.id));
    CloudComputeK8sRequestFingerprint {
        canonical: canonical_fields(&[
            ("path.cluster_id", path_cluster_id.to_string()),
            ("body.resource_id", input.resource_id.clone()),
            ("body.tenant_id", input.tenant_id.clone()),
            ("body.region", input.region.clone()),
            ("body.flavor", k8s_flavor_label(input.flavor).to_string()),
            (
                "body.control_plane_version",
                input.control_plane_version.clone(),
            ),
            (
                "body.control_plane_private",
                input.control_plane_private.to_string(),
            ),
            ("body.node_pools", canonical_node_pools(&pools)),
            ("body.quota.vcpu_limit", input.quota.vcpu_limit.to_string()),
            (
                "body.quota.memory_gb_limit",
                input.quota.memory_gb_limit.to_string(),
            ),
            ("body.quota.gpu_limit", input.quota.gpu_limit.to_string()),
            (
                "body.quota.local_ssd_gb_limit",
                input.quota.local_ssd_gb_limit.to_string(),
            ),
            (
                "body.quota.current_vcpu",
                input.quota.current_vcpu.to_string(),
            ),
            (
                "body.quota.current_memory_gb",
                input.quota.current_memory_gb.to_string(),
            ),
            (
                "body.quota.current_gpu",
                input.quota.current_gpu.to_string(),
            ),
            (
                "body.quota.current_local_ssd_gb",
                input.quota.current_local_ssd_gb.to_string(),
            ),
            (
                "body.residency",
                input.residency.label().unwrap_or("per_pack").to_string(),
            ),
            ("body.data_class", input.data_class.label().to_string()),
            (
                "body.created_at_epoch_seconds",
                input.created_at_epoch_seconds.to_string(),
            ),
        ]),
    }
}

fn canonical_node_pools(pools: &[KubernetesNodePoolCreate]) -> String {
    pools
        .iter()
        .map(|pool| {
            let mut security_groups = pool.security_groups.clone();
            security_groups.sort();
            canonical_fields(&[
                ("id", pool.id.clone()),
                ("az", pool.az.clone()),
                ("cell_id", pool.cell_id.clone()),
                ("subnet_id", pool.subnet_id.clone()),
                ("security_groups", canonical_sequence(&security_groups)),
                (
                    "flavor.class",
                    node_pool_flavor_class_label(pool.flavor.class).to_string(),
                ),
                ("flavor.vcpu", pool.flavor.vcpu.to_string()),
                ("flavor.memory_gb", pool.flavor.memory_gb.to_string()),
                ("flavor.gpu_count", pool.flavor.gpu_count.to_string()),
                ("flavor.local_ssd_gb", pool.flavor.local_ssd_gb.to_string()),
                ("min_nodes", pool.min_nodes.to_string()),
                ("max_nodes", pool.max_nodes.to_string()),
                ("autoscaling_enabled", pool.autoscaling_enabled.to_string()),
            ])
        })
        .collect::<Vec<_>>()
        .join("")
}

fn canonical_fields(fields: &[(&str, String)]) -> String {
    fields
        .iter()
        .map(|(name, value)| format!("{}:{}={}:{}", name.len(), name, value.len(), value))
        .collect::<Vec<_>>()
        .join("")
}

fn canonical_sequence(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("{}:{}", value.len(), value))
        .collect::<Vec<_>>()
        .join("")
}

fn cluster_record(cluster: KubernetesCluster) -> CloudComputeK8sClusterRecord {
    CloudComputeK8sClusterRecord {
        resource_id: cluster.resource_id.value.value,
        tenant_id: cluster.tenant_id.value,
        region: cluster.region.value.value,
        flavor: k8s_flavor_label(cluster.flavor.value).to_string(),
        control_plane_version: cluster.control_plane_version.value.value,
        control_plane_private: cluster.control_plane_private.value,
        node_pool_count: cluster.node_pools.value.len() as u32,
        residency: cluster
            .residency
            .value
            .label()
            .unwrap_or("per_pack")
            .to_string(),
        state: cluster_state_label(cluster.state.value).to_string(),
        data_class: cluster.data_class.value.label().to_string(),
        created_at_epoch_seconds: cluster.created_at_epoch_seconds.value,
        schema_version: cluster.schema_version.value,
    }
}

fn cloud_compute_status_kind(error: &CloudComputeError) -> CloudComputeK8sApiStatusKind {
    match error {
        CloudComputeError::DuplicateInstance
        | CloudComputeError::DuplicateKubernetesCluster
        | CloudComputeError::DuplicateFunction
        | CloudComputeError::DuplicateInvocation => CloudComputeK8sApiStatusKind::Conflict,
        CloudComputeError::UnknownFunction => CloudComputeK8sApiStatusKind::NotFound,
        CloudComputeError::ResourceTenantMismatch
        | CloudComputeError::ResourceRegionMismatch
        | CloudComputeError::ResidencyRegionMismatch
        | CloudComputeError::QuotaExceeded
        | CloudComputeError::PayloadDataClassNotAllowed => CloudComputeK8sApiStatusKind::Forbidden,
        CloudComputeError::InvalidTenantId
        | CloudComputeError::InvalidResourceId
        | CloudComputeError::ResourceKindMismatch
        | CloudComputeError::InvalidAzCode
        | CloudComputeError::AzRegionMismatch
        | CloudComputeError::InvalidCellId
        | CloudComputeError::CellAzMismatch
        | CloudComputeError::InvalidDataClass
        | CloudComputeError::InvalidImageRef
        | CloudComputeError::InvalidKeyPairId
        | CloudComputeError::InvalidUserDataUri
        | CloudComputeError::InvalidWorkloadIdentityPolicy
        | CloudComputeError::InvalidRuntimeIsolationPolicy
        | CloudComputeError::InvalidSchedulingPolicy
        | CloudComputeError::InvalidAuditEvidenceRef
        | CloudComputeError::InvalidFlavor
        | CloudComputeError::InvalidQuota
        | CloudComputeError::InvalidInstanceState
        | CloudComputeError::InvalidKubernetesState
        | CloudComputeError::InvalidFunctionState
        | CloudComputeError::InvalidNodePoolId
        | CloudComputeError::DuplicateNodePool
        | CloudComputeError::InvalidNodePoolShape
        | CloudComputeError::KubernetesHaRequiresThreeAzs
        | CloudComputeError::InvalidControlPlaneVersion
        | CloudComputeError::InvalidFunctionName
        | CloudComputeError::InvalidFunctionBudget
        | CloudComputeError::InvalidInvocationId
        | CloudComputeError::InvalidIdempotencyKey
        | CloudComputeError::FunctionNotActive => CloudComputeK8sApiStatusKind::BadRequest,
    }
}

fn cloud_compute_message(error: &CloudComputeError) -> &'static str {
    match cloud_compute_status_kind(error) {
        CloudComputeK8sApiStatusKind::BadRequest => "Cloud Compute rejected the request shape",
        CloudComputeK8sApiStatusKind::Unauthorized => {
            "Cloud Compute authentication evidence is missing"
        }
        CloudComputeK8sApiStatusKind::Forbidden => "Cloud Compute policy denied the request",
        CloudComputeK8sApiStatusKind::NotFound => "Cloud Compute resource was not found",
        CloudComputeK8sApiStatusKind::Conflict => "Cloud Compute resource already exists",
        CloudComputeK8sApiStatusKind::UnprocessableEntity => {
            "Cloud Compute rejected request idempotency"
        }
    }
}

fn cloud_compute_issue(error: &CloudComputeError) -> &'static str {
    match error {
        CloudComputeError::InvalidTenantId => "tenant_id must be a ten_ identifier",
        CloudComputeError::InvalidResourceId => "resource_id must be canonical cloud resource id",
        CloudComputeError::ResourceTenantMismatch => "resource tenant must match request tenant",
        CloudComputeError::ResourceRegionMismatch => "resource region must match request region",
        CloudComputeError::ResourceKindMismatch => {
            "resource kind must match requested compute type"
        }
        CloudComputeError::InvalidAzCode => "AZ must be canonical lowercase ASCII",
        CloudComputeError::AzRegionMismatch => "AZ code must sit under its region code",
        CloudComputeError::InvalidCellId => "cell_id must be canonical and use the cell- prefix",
        CloudComputeError::CellAzMismatch => "cell_id must sit under its AZ namespace",
        CloudComputeError::ResidencyRegionMismatch => "region must satisfy residency policy",
        CloudComputeError::InvalidDataClass => "data_class must be public metadata class",
        CloudComputeError::InvalidImageRef => "image must be a supported digest-pinned image ref",
        CloudComputeError::InvalidKeyPairId => "key_pair must use the key_ prefix",
        CloudComputeError::InvalidUserDataUri => "user_data_uri must use the userdata/ prefix",
        CloudComputeError::InvalidWorkloadIdentityPolicy => {
            "workload identity refs must be tenant/cell scoped and non-secret"
        }
        CloudComputeError::InvalidRuntimeIsolationPolicy => {
            "compute workloads require private and sandboxed runtime isolation"
        }
        CloudComputeError::InvalidSchedulingPolicy => {
            "compute scheduling evidence must require topology spread"
        }
        CloudComputeError::InvalidAuditEvidenceRef => {
            "compute audit evidence ref must be a non-secret evidence path"
        }
        CloudComputeError::InvalidFlavor => {
            "flavor resources must be positive and class-consistent"
        }
        CloudComputeError::InvalidQuota => "quota envelope must not start beyond its limits",
        CloudComputeError::QuotaExceeded => "requested cluster exceeds tenant quota envelope",
        CloudComputeError::InvalidInstanceState => "VM create requests must start in Pending state",
        CloudComputeError::InvalidKubernetesState => {
            "Kubernetes create requests must start in Creating state"
        }
        CloudComputeError::InvalidFunctionState => {
            "function create requests must start in Deploying state"
        }
        CloudComputeError::InvalidNodePoolId => "node pool id must use the np_ prefix",
        CloudComputeError::DuplicateNodePool => "node pool ids must be unique",
        CloudComputeError::InvalidNodePoolShape => "node pool shape must be canonical",
        CloudComputeError::KubernetesHaRequiresThreeAzs => {
            "HA Kubernetes requires at least three AZs"
        }
        CloudComputeError::InvalidControlPlaneVersion => "control plane version must be canonical",
        CloudComputeError::InvalidFunctionName => "function name must be canonical",
        CloudComputeError::InvalidFunctionBudget => {
            "function budget must be within platform bounds"
        }
        CloudComputeError::InvalidInvocationId => "invocation id must use the fninv_ prefix",
        CloudComputeError::InvalidIdempotencyKey => "function idempotency key must be bounded",
        CloudComputeError::FunctionNotActive => "function must be active before invocation",
        CloudComputeError::PayloadDataClassNotAllowed => {
            "payload data_class must be admitted by deployment policy"
        }
        CloudComputeError::DuplicateInstance => "instance resource id is already present",
        CloudComputeError::DuplicateKubernetesCluster => {
            "Kubernetes cluster resource id is already present"
        }
        CloudComputeError::DuplicateFunction => "function resource id is already present",
        CloudComputeError::DuplicateInvocation => "function invocation id is already present",
        CloudComputeError::UnknownFunction => "function resource must exist before invocation",
    }
}

fn detail(field: &str, issue: &str) -> CloudComputeK8sApiErrorDetail {
    CloudComputeK8sApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}

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

/// Validates all boundary conditions for a delete request without touching the
/// catalog.
///
/// Returns the parsed [`ResourceId`] on success so the caller can use it for
/// catalog lookup.
pub fn validate_cloud_compute_k8s_cluster_delete_request(
    request: &CloudComputeK8sClusterDeleteApiRequest,
) -> Result<ResourceId, CloudComputeK8sApiError> {
    validate_cloud_compute_k8s_cluster_delete_request_with_authorization_verifier(
        request,
        &CloudComputeK8sFailClosedAuthorizationVerifier,
    )
}

pub fn validate_cloud_compute_k8s_cluster_delete_request_with_authorization_verifier(
    request: &CloudComputeK8sClusterDeleteApiRequest,
    authorization_verifier: &impl CloudComputeK8sAuthorizationVerifier,
) -> Result<ResourceId, CloudComputeK8sApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_cluster_id_only(&request.path_cluster_id)?;
    let resource_id = validate_cluster_resource_id(&request.path_cluster_id)?;
    validate_delete_tenant_binding(&request.boundary, &request.principal, &resource_id)?;
    validate_authorization(
        &request.principal,
        &request.authorization.decision_id,
        CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE,
        authorization_verifier,
    )?;
    Ok(resource_id)
}

/// Full delete execution: validates, checks idempotency, looks up the cluster,
/// projects its state to `Deleting`, records the ledger entry, and returns the
/// typed success response.
///
/// The catalog is accessed read-only — actual teardown is the reconciler's
/// concern. Only the boundary-owned idempotency ledger is mutated.
pub fn delete_cloud_compute_k8s_cluster_from_api(
    catalog: &CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeK8sDeleteIdempotencyLedger,
    request: CloudComputeK8sClusterDeleteApiRequest,
) -> Result<CloudComputeK8sClusterDeleteSuccessResponse, CloudComputeK8sApiError> {
    let resource_id = validate_cloud_compute_k8s_cluster_delete_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE,
    );
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.path_cluster_id == request.path_cluster_id {
            return entry.result.clone();
        }
        return Err(CloudComputeK8sApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let cluster = catalog
        .kubernetes_clusters()
        .find(|c| c.resource_id.value == resource_id)
        .ok_or_else(|| CloudComputeK8sApiError::ClusterNotFound {
            cluster_id: request.path_cluster_id.clone(),
        })?;

    let mut record = cluster_record(cluster.clone());
    record.state = cluster_state_label(KubernetesClusterState::Draining).to_string();

    let request_id = request.boundary.request_id.clone();
    let result: CloudComputeK8sDeleteApiResult = Ok(
        CloudComputeK8sClusterDeleteSuccessResponse::accepted(record, request_id),
    );

    idempotency_ledger.remember(
        key,
        CloudComputeK8sDeleteLedgerEntry {
            path_cluster_id: request.path_cluster_id,
            result: result.clone(),
        },
    );
    result
}

pub fn delete_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
    catalog: &CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeK8sDeleteIdempotencyLedger,
    request: CloudComputeK8sClusterDeleteApiRequest,
    authorization_verifier: &impl CloudComputeK8sAuthorizationVerifier,
) -> Result<CloudComputeK8sClusterDeleteSuccessResponse, CloudComputeK8sApiError> {
    let resource_id =
        validate_cloud_compute_k8s_cluster_delete_request_with_authorization_verifier(
            &request,
            authorization_verifier,
        )?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE,
    );
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.path_cluster_id == request.path_cluster_id {
            return entry.result.clone();
        }
        return Err(CloudComputeK8sApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let cluster = catalog
        .kubernetes_clusters()
        .find(|c| c.resource_id.value == resource_id)
        .ok_or_else(|| CloudComputeK8sApiError::ClusterNotFound {
            cluster_id: request.path_cluster_id.clone(),
        })?;

    let mut record = cluster_record(cluster.clone());
    record.state = cluster_state_label(KubernetesClusterState::Draining).to_string();

    let request_id = request.boundary.request_id.clone();
    let result: CloudComputeK8sDeleteApiResult = Ok(
        CloudComputeK8sClusterDeleteSuccessResponse::accepted(record, request_id),
    );

    idempotency_ledger.remember(
        key,
        CloudComputeK8sDeleteLedgerEntry {
            path_cluster_id: request.path_cluster_id,
            result: result.clone(),
        },
    );
    result
}

/// Stable planned entrypoint for `cloud.compute.k8s.cluster.delete`.
///
/// Delegates to [`delete_cloud_compute_k8s_cluster_from_api`] so the plan
/// symbol remains stable without adding a second validation path.
pub fn delete_cluster(
    catalog: &CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeK8sDeleteIdempotencyLedger,
    request: CloudComputeK8sClusterDeleteApiRequest,
) -> Result<CloudComputeK8sClusterDeleteSuccessResponse, CloudComputeK8sApiError> {
    delete_cloud_compute_k8s_cluster_from_api(catalog, idempotency_ledger, request)
}

pub fn delete_cluster_with_authorization_verifier(
    catalog: &CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeK8sDeleteIdempotencyLedger,
    request: CloudComputeK8sClusterDeleteApiRequest,
    authorization_verifier: &impl CloudComputeK8sAuthorizationVerifier,
) -> Result<CloudComputeK8sClusterDeleteSuccessResponse, CloudComputeK8sApiError> {
    delete_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
        catalog,
        idempotency_ledger,
        request,
        authorization_verifier,
    )
}

/// Validates that `path_cluster_id` is non-empty (delete has no body to match
/// against).
fn validate_path_cluster_id_only(path_cluster_id: &str) -> Result<(), CloudComputeK8sApiError> {
    if path_cluster_id.trim().is_empty() {
        return Err(CloudComputeK8sApiError::EmptyPathClusterId);
    }
    Ok(())
}

/// Validates that the tenant encoded in the cluster resource-id matches both
/// the boundary tenant header and the authenticated principal tenant.
///
/// Returns `EmptyPrincipalId` (401) if the principal id is absent, and
/// `TenantMismatch` (403) if any tenant comparison fails.
fn validate_delete_tenant_binding(
    boundary: &CloudComputeK8sApiBoundaryContext,
    principal: &CloudComputeK8sApiPrincipal,
    resource_id: &ResourceId,
) -> Result<(), CloudComputeK8sApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(CloudComputeK8sApiError::EmptyPrincipalId);
    }
    let resource_tenant_id =
        resource_id
            .tenant_id()
            .map_err(|_| CloudComputeK8sApiError::InvalidClusterId {
                cluster_id: resource_id.value.clone(),
            })?;
    if boundary.tenant_id != principal.tenant_id || boundary.tenant_id != resource_tenant_id {
        return Err(CloudComputeK8sApiError::TenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
            resource_tenant_id,
            body_tenant_id: String::new(),
        });
    }
    Ok(())
}
