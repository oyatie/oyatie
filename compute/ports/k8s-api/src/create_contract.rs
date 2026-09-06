pub const CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE: &str = "cloud.compute.k8s.cluster.create";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudComputeK8sClusterCreateApiStatus {
    Created,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
    ServiceUnavailable,
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
            Self::ServiceUnavailable => 503,
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
    LifecycleRepositoryUnavailable,
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
            Self::LifecycleRepositoryUnavailable => {
                "CLOUD_COMPUTE_K8S_LIFECYCLE_REPOSITORY_UNAVAILABLE"
            }
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

pub type CloudComputeK8sRepositoryFuture<'a, T> =
    Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct CloudComputeK8sOperationKey {
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub principal_id: String,    // data_class: INTERNAL_ONLY
    pub surface: String,         // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sCreateCommand {
    pub operation_key: CloudComputeK8sOperationKey, // data_class: INTERNAL_ONLY
    pub fingerprint: String,                        // data_class: INTERNAL_ONLY
    pub desired_spec: CloudComputeK8sClusterCreateRequest, // data_class: INTERNAL_ONLY
    pub cluster: CloudComputeK8sClusterRecord,      // data_class: INTERNAL_ONLY
    pub request_id: String,                         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CloudComputeK8sCreateReceipt {
    pub cluster: CloudComputeK8sClusterRecord, // data_class: INTERNAL_ONLY
    pub request_id: String,                   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudComputeK8sLifecycleRepositoryError {
    ClusterAlreadyExists,
    ClusterNotFound,
    IdempotencyKeyReused {
        idempotency_key: String, // data_class: INTERNAL_ONLY
    },
    Unavailable,
    IntegrityViolation,
}

pub trait CloudComputeK8sLifecycleRepository: Send + Sync {
    fn commit_create<'a>(
        &'a self,
        command: CloudComputeK8sCreateCommand,
    ) -> CloudComputeK8sRepositoryFuture<
        'a,
        Result<CloudComputeK8sCreateReceipt, CloudComputeK8sLifecycleRepositoryError>,
    >;

    fn commit_deletion<'a>(
        &'a self,
        command: CloudComputeK8sDeleteCommand,
    ) -> CloudComputeK8sRepositoryFuture<
        'a,
        Result<CloudComputeK8sDeleteReceipt, CloudComputeK8sLifecycleRepositoryError>,
    >;
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
