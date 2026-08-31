pub const CLOUD_COMPUTE_VM_CREATE_SURFACE: &str = "cloud.compute.vm.create";
const DEFAULT_VM_CREATE_IDEMPOTENCY_LEDGER_MAX_ENTRIES: usize = 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudComputeVmCreateApiStatus {
    Created,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

impl CloudComputeVmCreateApiStatus {
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
pub enum CloudComputeVmApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    PathInstanceIdEmpty,
    InstanceIdInvalid,
    InstanceKindMismatch,
    InstanceIdMismatch,
    TenantMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    IdempotencyKeyReused,
    FlavorClassInvalid,
    SecurityGroupBindingInvalid,
    IamRoleBindingInvalid,
    ResidencyInvalid,
    DataClassInvalid,
    ComputeInvalidRequest,
    ComputeForbidden,
    ComputeNotFound,
    ComputeConflict,
}

impl CloudComputeVmApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "CLOUD_COMPUTE_VM_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "CLOUD_COMPUTE_VM_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "CLOUD_COMPUTE_VM_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "CLOUD_COMPUTE_VM_PRINCIPAL_ID_EMPTY",
            Self::PathInstanceIdEmpty => "CLOUD_COMPUTE_VM_PATH_INSTANCE_ID_EMPTY",
            Self::InstanceIdInvalid => "CLOUD_COMPUTE_VM_INSTANCE_ID_INVALID",
            Self::InstanceKindMismatch => "CLOUD_COMPUTE_VM_INSTANCE_KIND_MISMATCH",
            Self::InstanceIdMismatch => "CLOUD_COMPUTE_VM_INSTANCE_ID_MISMATCH",
            Self::TenantMismatch => "CLOUD_COMPUTE_VM_TENANT_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => {
                "CLOUD_COMPUTE_VM_AUTHORIZATION_DECISION_ID_EMPTY"
            }
            Self::AuthorizationTenantMismatch => "CLOUD_COMPUTE_VM_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => {
                "CLOUD_COMPUTE_VM_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationDenied => "CLOUD_COMPUTE_VM_AUTHORIZATION_DENIED",
            Self::IdempotencyKeyReused => "CLOUD_COMPUTE_VM_IDEMPOTENCY_KEY_REUSED",
            Self::FlavorClassInvalid => "CLOUD_COMPUTE_VM_FLAVOR_CLASS_INVALID",
            Self::SecurityGroupBindingInvalid => "CLOUD_COMPUTE_VM_SECURITY_GROUP_BINDING_INVALID",
            Self::IamRoleBindingInvalid => "CLOUD_COMPUTE_VM_IAM_ROLE_BINDING_INVALID",
            Self::ResidencyInvalid => "CLOUD_COMPUTE_VM_RESIDENCY_INVALID",
            Self::DataClassInvalid => "CLOUD_COMPUTE_VM_DATA_CLASS_INVALID",
            Self::ComputeInvalidRequest => "CLOUD_COMPUTE_VM_INVALID_REQUEST",
            Self::ComputeForbidden => "CLOUD_COMPUTE_VM_FORBIDDEN",
            Self::ComputeNotFound => "CLOUD_COMPUTE_VM_NOT_FOUND",
            Self::ComputeConflict => "CLOUD_COMPUTE_VM_CONFLICT",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
    pub proof: Option<CloudComputeVmApiAuthorizationProof>, // data_class: INTERNAL_ONLY
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmApiAuthorizationProof {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub surface: String,               // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub verified: bool,                // data_class: INTERNAL_ONLY
    pub issued_at_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
    pub expires_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

pub trait CloudComputeVmApiAuthorizationVerifier {
    fn proof_for_decision(&self, decision_id: &str)
    -> Option<&CloudComputeVmApiAuthorizationProof>;

    fn evaluation_epoch_seconds(&self) -> u64;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmTrustedAuthorizationVerifier {
    evaluation_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    proofs_by_decision_id: BTreeMap<String, CloudComputeVmApiAuthorizationProof>, // data_class: INTERNAL_ONLY
}

impl CloudComputeVmTrustedAuthorizationVerifier {
    pub fn new(evaluation_epoch_seconds: u64) -> Self {
        Self {
            evaluation_epoch_seconds,
            proofs_by_decision_id: BTreeMap::new(),
        }
    }

    pub fn with_trusted_proof(mut self, proof: CloudComputeVmApiAuthorizationProof) -> Self {
        self.insert_trusted_proof(proof);
        self
    }

    pub fn insert_trusted_proof(&mut self, proof: CloudComputeVmApiAuthorizationProof) {
        self.proofs_by_decision_id
            .insert(proof.decision_id.clone(), proof);
    }
}

impl CloudComputeVmApiAuthorizationVerifier for CloudComputeVmTrustedAuthorizationVerifier {
    fn proof_for_decision(
        &self,
        decision_id: &str,
    ) -> Option<&CloudComputeVmApiAuthorizationProof> {
        self.proofs_by_decision_id.get(decision_id)
    }

    fn evaluation_epoch_seconds(&self) -> u64 {
        self.evaluation_epoch_seconds
    }
}

struct CloudComputeVmMissingAuthorizationVerifier;

impl CloudComputeVmApiAuthorizationVerifier for CloudComputeVmMissingAuthorizationVerifier {
    fn proof_for_decision(
        &self,
        _decision_id: &str,
    ) -> Option<&CloudComputeVmApiAuthorizationProof> {
        None
    }

    fn evaluation_epoch_seconds(&self) -> u64 {
        u64::MAX
    }
}
