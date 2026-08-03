//! Cloud Compute VM API boundary for instance creation.
//!
//! This crate owns request boundary normalization, compute-owned authorization
//! verifier checks, idempotent create semantics, and tenant-safe VM metadata
//! the Cloud compute kernel. Hypervisor scheduling and boot orchestration live
//! behind later adapters.

use std::collections::BTreeMap;

use compute_domain::{
    CloudComputeCatalog, CloudComputeError, ComputeFlavorSpec, ComputeQuotaEnvelope, ComputeRepo,
    ImageRefKind, Instance, InstanceCreate, InstanceState,
};
use compute_resource::{InstanceFlavor, ResourceId};
use network_residency::{ResidencyClass, parse_residency_class_label};
use data_boundary_kernel::{DataClass, parse_data_class_label};

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmCreateRequest {
    pub resource_id: String,              // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub region: String,                   // data_class: PUBLIC
    pub az: String,                       // data_class: PUBLIC
    pub cell_id: String,                  // data_class: PUBLIC
    pub flavor: CloudComputeVmFlavorSpec, // data_class: PUBLIC
    pub image: String,                    // data_class: INTERNAL_ONLY
    pub key_pair: Option<String>,         // data_class: INTERNAL_ONLY
    pub vpc_id: String,                   // data_class: INTERNAL_ONLY
    pub subnet_id: String,                // data_class: INTERNAL_ONLY
    pub security_groups: Vec<CloudComputeVmSecurityGroupRef>, // data_class: INTERNAL_ONLY
    pub iam_role: Option<CloudComputeVmIamRoleRef>, // data_class: INTERNAL_ONLY
    pub user_data_uri: Option<String>,    // data_class: INTERNAL_ONLY
    pub quota: CloudComputeVmQuotaEnvelope, // data_class: INTERNAL_ONLY
    pub residency: String,                // data_class: INTERNAL_ONLY
    pub data_class: String,               // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmFlavorSpec {
    pub class: String,     // data_class: PUBLIC
    pub vcpu: u32,         // data_class: PUBLIC
    pub memory_gb: u32,    // data_class: PUBLIC
    pub gpu_count: u32,    // data_class: PUBLIC
    pub local_ssd_gb: u32, // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CloudComputeVmQuotaEnvelope {
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
pub struct CloudComputeVmSecurityGroupRef {
    pub value: String,     // data_class: INTERNAL_ONLY
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    pub region: String,    // data_class: PUBLIC
    pub vpc_id: String,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmIamRoleRef {
    pub value: String,     // data_class: INTERNAL_ONLY
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    pub region: String,    // data_class: PUBLIC
    pub vpc_id: String,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmCreateApiRequest {
    pub path_instance_id: String, // data_class: INTERNAL_ONLY
    pub boundary: CloudComputeVmApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CloudComputeVmApiPrincipal, // data_class: INTERNAL_ONLY
    pub authorization: CloudComputeVmApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: CloudComputeVmCreateRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmCreateIdempotencyLedger {
    entries: BTreeMap<CloudComputeVmIdempotencyLedgerKey, CloudComputeVmCreateLedgerEntry>, // data_class: INTERNAL_ONLY
    max_entries: usize, // data_class: INTERNAL_ONLY
}

impl Default for CloudComputeVmCreateIdempotencyLedger {
    fn default() -> Self {
        Self::with_max_entries(DEFAULT_VM_CREATE_IDEMPOTENCY_LEDGER_MAX_ENTRIES)
    }
}

impl CloudComputeVmCreateIdempotencyLedger {
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
        key: CloudComputeVmIdempotencyLedgerKey,
        entry: CloudComputeVmCreateLedgerEntry,
    ) {
        if self.entries.len() >= self.max_entries {
            if let Some(evicted) = self.entries.keys().next().cloned() {
                self.entries.remove(&evicted);
            }
        }
        self.entries.insert(key, entry);
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CloudComputeVmIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudComputeVmCreateLedgerEntry {
    fingerprint: CloudComputeVmRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CloudComputeVmCreateApiResult,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudComputeVmRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

type CloudComputeVmCreateApiResult =
    Result<CloudComputeVmCreateSuccessResponse, CloudComputeVmApiError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmCreateSuccessResponse {
    pub data: CloudComputeVmRecord,       // data_class: INTERNAL_ONLY
    pub metadata: CloudComputeVmMetadata, // data_class: INTERNAL_ONLY
}

impl CloudComputeVmCreateSuccessResponse {
    pub fn created(data: CloudComputeVmRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: CloudComputeVmMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmMetadata {
    pub request_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmRecord {
    pub resource_id: String,              // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub region: String,                   // data_class: PUBLIC
    pub az: String,                       // data_class: PUBLIC
    pub cell_id: String,                  // data_class: PUBLIC
    pub flavor: CloudComputeVmFlavorSpec, // data_class: PUBLIC
    pub image_kind: String,               // data_class: PUBLIC
    pub residency: String,                // data_class: INTERNAL_ONLY
    pub state: String,                    // data_class: PUBLIC
    pub data_class: String,               // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,    // data_class: INTERNAL_ONLY
    pub schema_version: u32,              // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmApiErrorResponse {
    pub error: CloudComputeVmApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmApiErrorBody {
    pub code: String,                               // data_class: INTERNAL_ONLY
    pub message: String,                            // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,          // data_class: INTERNAL_ONLY
    pub request_id: String,                         // data_class: INTERNAL_ONLY
    pub details: Vec<CloudComputeVmApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudComputeVmApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    EmptyPathInstanceId,
    InvalidInstanceId {
        instance_id: String,
    },
    InstanceKindMismatch {
        instance_id: String,
        kind_label: String,
    },
    InstanceIdMismatch {
        path_instance_id: String,
        body_resource_id: String,
    },
    TenantMismatch {
        header_tenant_id: String,
        principal_tenant_id: String,
        resource_tenant_id: String,
        body_tenant_id: String,
    },
    EmptyAuthorizationDecisionId,
    AuthorizationTenantMismatch {
        authorization_tenant_id: String,
        principal_tenant_id: String,
    },
    AuthorizationPrincipalMismatch {
        authorization_principal_id: String,
        principal_id: String,
    },
    AuthorizationDenied {
        surface: String,
    },
    IdempotencyKeyReused {
        idempotency_key: String,
    },
    InvalidFlavorClassLabel {
        class: String,
    },
    InvalidResidencyLabel {
        residency: String,
    },
    InvalidDataClassLabel {
        data_class: String,
    },
    SecurityGroupBindingMismatch {
        security_group: String,
        tenant_id: String,
        region: String,
        vpc_id: String,
    },
    IamRoleBindingMismatch {
        role_id: String,
        tenant_id: String,
        region: String,
        vpc_id: String,
    },
    Compute(CloudComputeError),
}

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
                "must be a canonical oya:cloud instance resource id",
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

pub fn validate_cloud_compute_vm_create_request(
    request: &CloudComputeVmCreateApiRequest,
) -> Result<ResourceId, CloudComputeVmApiError> {
    validate_cloud_compute_vm_create_request_with_verifier(
        request,
        &CloudComputeVmMissingAuthorizationVerifier,
    )
}

pub fn validate_cloud_compute_vm_create_request_with_verifier(
    request: &CloudComputeVmCreateApiRequest,
    authorization_verifier: &impl CloudComputeVmApiAuthorizationVerifier,
) -> Result<ResourceId, CloudComputeVmApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_instance_id(&request.path_instance_id, &request.body.resource_id)?;
    let resource_id = validate_instance_resource_id(&request.path_instance_id)?;
    validate_tenant_binding(
        &request.boundary,
        &request.principal,
        &resource_id,
        &request.body.tenant_id,
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization.decision_id,
        CLOUD_COMPUTE_VM_CREATE_SURFACE,
        authorization_verifier,
    )?;
    Ok(resource_id)
}

pub fn create_cloud_compute_vm_from_api(
    catalog: &mut CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeVmCreateIdempotencyLedger,
    request: CloudComputeVmCreateApiRequest,
) -> Result<CloudComputeVmCreateSuccessResponse, CloudComputeVmApiError> {
    create_cloud_compute_vm_from_api_with_verifier(
        catalog,
        idempotency_ledger,
        request,
        &CloudComputeVmMissingAuthorizationVerifier,
    )
}

pub fn create_cloud_compute_vm_from_api_with_verifier(
    catalog: &mut CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeVmCreateIdempotencyLedger,
    request: CloudComputeVmCreateApiRequest,
    authorization_verifier: &impl CloudComputeVmApiAuthorizationVerifier,
) -> Result<CloudComputeVmCreateSuccessResponse, CloudComputeVmApiError> {
    validate_cloud_compute_vm_create_request_with_verifier(&request, authorization_verifier)?;
    let input = instance_create_input(&request.body)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_COMPUTE_VM_CREATE_SURFACE,
    );
    let fingerprint = vm_create_fingerprint_for(&request.path_instance_id, &input);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudComputeVmApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let result = catalog
        .create_instance(input)
        .map_err(CloudComputeVmApiError::Compute)
        .map(|instance| {
            CloudComputeVmCreateSuccessResponse::created(vm_record(instance), request_id)
        });
    idempotency_ledger.remember(
        key,
        CloudComputeVmCreateLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

fn validate_boundary(
    boundary: &CloudComputeVmApiBoundaryContext,
) -> Result<(), CloudComputeVmApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(CloudComputeVmApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(CloudComputeVmApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(CloudComputeVmApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_instance_id(
    path_instance_id: &str,
    body_resource_id: &str,
) -> Result<(), CloudComputeVmApiError> {
    if path_instance_id.trim().is_empty() {
        return Err(CloudComputeVmApiError::EmptyPathInstanceId);
    }
    if path_instance_id != body_resource_id {
        return Err(CloudComputeVmApiError::InstanceIdMismatch {
            path_instance_id: path_instance_id.to_string(),
            body_resource_id: body_resource_id.to_string(),
        });
    }
    Ok(())
}

fn validate_instance_resource_id(value: &str) -> Result<ResourceId, CloudComputeVmApiError> {
    let id = ResourceId::new(value.to_string()).map_err(|_| {
        CloudComputeVmApiError::InvalidInstanceId {
            instance_id: value.to_string(),
        }
    })?;
    let kind_label = id
        .kind_label()
        .map_err(|_| CloudComputeVmApiError::InvalidInstanceId {
            instance_id: value.to_string(),
        })?;
    if kind_label != "instance" {
        return Err(CloudComputeVmApiError::InstanceKindMismatch {
            instance_id: value.to_string(),
            kind_label,
        });
    }
    Ok(id)
}

fn validate_tenant_binding(
    boundary: &CloudComputeVmApiBoundaryContext,
    principal: &CloudComputeVmApiPrincipal,
    resource_id: &ResourceId,
    body_tenant_id: &str,
) -> Result<(), CloudComputeVmApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(CloudComputeVmApiError::EmptyPrincipalId);
    }
    let resource_tenant_id =
        resource_id
            .tenant_id()
            .map_err(|_| CloudComputeVmApiError::InvalidInstanceId {
                instance_id: resource_id.value.clone(),
            })?;
    if boundary.tenant_id != principal.tenant_id
        || boundary.tenant_id != resource_tenant_id
        || boundary.tenant_id != body_tenant_id
    {
        return Err(CloudComputeVmApiError::TenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
            resource_tenant_id,
            body_tenant_id: body_tenant_id.to_string(),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &CloudComputeVmApiPrincipal,
    decision_id: &str,
    surface: &str,
    authorization_verifier: &impl CloudComputeVmApiAuthorizationVerifier,
) -> Result<(), CloudComputeVmApiError> {
    if decision_id.trim().is_empty() {
        return Err(CloudComputeVmApiError::EmptyAuthorizationDecisionId);
    }
    validate_authorization_proof(principal, decision_id, surface, authorization_verifier)
}

fn validate_authorization_proof(
    principal: &CloudComputeVmApiPrincipal,
    decision_id: &str,
    surface: &str,
    authorization_verifier: &impl CloudComputeVmApiAuthorizationVerifier,
) -> Result<(), CloudComputeVmApiError> {
    let Some(proof) = authorization_verifier.proof_for_decision(decision_id) else {
        return Err(CloudComputeVmApiError::AuthorizationDenied {
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
        return Err(CloudComputeVmApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn instance_create_input(
    body: &CloudComputeVmCreateRequest,
) -> Result<InstanceCreate, CloudComputeVmApiError> {
    Ok(InstanceCreate {
        resource_id: body.resource_id.clone(),
        tenant_id: body.tenant_id.clone(),
        region: body.region.clone(),
        az: body.az.clone(),
        cell_id: body.cell_id.clone(),
        flavor: flavor_spec(body.flavor.clone())?,
        image: body.image.clone(),
        key_pair: body.key_pair.clone(),
        vpc_id: body.vpc_id.clone(),
        subnet_id: body.subnet_id.clone(),
        security_groups: security_group_values(body)?,
        iam_role: iam_role_value(body)?,
        user_data_uri: body.user_data_uri.clone(),
        quota: quota_envelope(body.quota),
        residency: parse_api_residency(body.residency.clone())?,
        state: InstanceState::Pending,
        data_class: parse_api_data_class(body.data_class.clone())?,
        created_at_epoch_seconds: body.created_at_epoch_seconds,
    })
}

fn security_group_values(
    body: &CloudComputeVmCreateRequest,
) -> Result<Vec<String>, CloudComputeVmApiError> {
    let mut values = Vec::with_capacity(body.security_groups.len());
    for group in &body.security_groups {
        if group.tenant_id != body.tenant_id
            || group.region != body.region
            || group.vpc_id != body.vpc_id
        {
            return Err(CloudComputeVmApiError::SecurityGroupBindingMismatch {
                security_group: group.value.clone(),
                tenant_id: group.tenant_id.clone(),
                region: group.region.clone(),
                vpc_id: group.vpc_id.clone(),
            });
        }
        values.push(group.value.clone());
    }
    Ok(values)
}

fn iam_role_value(
    body: &CloudComputeVmCreateRequest,
) -> Result<Option<String>, CloudComputeVmApiError> {
    let Some(role) = &body.iam_role else {
        return Ok(None);
    };
    if role.tenant_id != body.tenant_id || role.region != body.region || role.vpc_id != body.vpc_id
    {
        return Err(CloudComputeVmApiError::IamRoleBindingMismatch {
            role_id: role.value.clone(),
            tenant_id: role.tenant_id.clone(),
            region: role.region.clone(),
            vpc_id: role.vpc_id.clone(),
        });
    }
    Ok(Some(role.value.clone()))
}

fn flavor_spec(
    input: CloudComputeVmFlavorSpec,
) -> Result<ComputeFlavorSpec, CloudComputeVmApiError> {
    Ok(ComputeFlavorSpec {
        class: parse_flavor_class(input.class)?,
        vcpu: input.vcpu,
        memory_gb: input.memory_gb,
        gpu_count: input.gpu_count,
        local_ssd_gb: input.local_ssd_gb,
    })
}

fn quota_envelope(input: CloudComputeVmQuotaEnvelope) -> ComputeQuotaEnvelope {
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

fn parse_flavor_class(label: String) -> Result<InstanceFlavor, CloudComputeVmApiError> {
    match label.as_str() {
        "general_purpose" => Ok(InstanceFlavor::GeneralPurpose),
        "compute_optimized" => Ok(InstanceFlavor::ComputeOptimized),
        "memory_optimized" => Ok(InstanceFlavor::MemoryOptimized),
        "gpu" => Ok(InstanceFlavor::Gpu),
        _ => Err(CloudComputeVmApiError::InvalidFlavorClassLabel { class: label }),
    }
}

fn flavor_class_label(class: InstanceFlavor) -> &'static str {
    match class {
        InstanceFlavor::GeneralPurpose => "general_purpose",
        InstanceFlavor::ComputeOptimized => "compute_optimized",
        InstanceFlavor::MemoryOptimized => "memory_optimized",
        InstanceFlavor::Gpu => "gpu",
    }
}

fn parse_api_residency(label: String) -> Result<ResidencyClass, CloudComputeVmApiError> {
    parse_residency_class_label(&label)
        .ok_or(CloudComputeVmApiError::InvalidResidencyLabel { residency: label })
}

fn parse_api_data_class(label: String) -> Result<DataClass, CloudComputeVmApiError> {
    parse_data_class_label(&label)
        .ok_or(CloudComputeVmApiError::InvalidDataClassLabel { data_class: label })
}

fn image_kind_label(kind: ImageRefKind) -> &'static str {
    match kind {
        ImageRefKind::Oci => "oci",
        ImageRefKind::Qcow2 => "qcow2",
        ImageRefKind::FunctionBundle => "function_bundle",
    }
}

fn instance_state_label(state: InstanceState) -> &'static str {
    match state {
        InstanceState::Pending => "pending",
        InstanceState::Running => "running",
        InstanceState::Stopping => "stopping",
        InstanceState::Stopped => "stopped",
        InstanceState::Terminated => "terminated",
    }
}

fn idempotency_key_for(
    boundary: &CloudComputeVmApiBoundaryContext,
    principal: &CloudComputeVmApiPrincipal,
    surface: &str,
) -> CloudComputeVmIdempotencyLedgerKey {
    CloudComputeVmIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn vm_create_fingerprint_for(
    path_instance_id: &str,
    input: &InstanceCreate,
) -> CloudComputeVmRequestFingerprint {
    let mut security_groups = input.security_groups.clone();
    security_groups.sort();
    CloudComputeVmRequestFingerprint {
        canonical: canonical_fields(&[
            ("path.instance_id", path_instance_id.to_string()),
            ("body.resource_id", input.resource_id.clone()),
            ("body.tenant_id", input.tenant_id.clone()),
            ("body.region", input.region.clone()),
            ("body.az", input.az.clone()),
            ("body.cell_id", input.cell_id.clone()),
            (
                "body.flavor.class",
                flavor_class_label(input.flavor.class).to_string(),
            ),
            ("body.flavor.vcpu", input.flavor.vcpu.to_string()),
            ("body.flavor.memory_gb", input.flavor.memory_gb.to_string()),
            ("body.flavor.gpu_count", input.flavor.gpu_count.to_string()),
            (
                "body.flavor.local_ssd_gb",
                input.flavor.local_ssd_gb.to_string(),
            ),
            ("body.image", input.image.clone()),
            ("body.key_pair", input.key_pair.clone().unwrap_or_default()),
            ("body.vpc_id", input.vpc_id.clone()),
            ("body.subnet_id", input.subnet_id.clone()),
            ("body.security_groups", canonical_sequence(&security_groups)),
            ("body.iam_role", input.iam_role.clone().unwrap_or_default()),
            (
                "body.user_data_uri",
                input.user_data_uri.clone().unwrap_or_default(),
            ),
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

fn vm_record(instance: Instance) -> CloudComputeVmRecord {
    let flavor = instance.flavor.value;
    let image = instance.image.value;
    CloudComputeVmRecord {
        resource_id: instance.resource_id.value.value,
        tenant_id: instance.tenant_id.value,
        region: instance.region.value.value,
        az: instance.az.value.value,
        cell_id: instance.cell_id.value.value,
        flavor: CloudComputeVmFlavorSpec {
            class: flavor_class_label(flavor.class).to_string(),
            vcpu: flavor.vcpu,
            memory_gb: flavor.memory_gb,
            gpu_count: flavor.gpu_count,
            local_ssd_gb: flavor.local_ssd_gb,
        },
        image_kind: image_kind_label(image.kind).to_string(),
        residency: instance
            .residency
            .value
            .label()
            .unwrap_or("per_pack")
            .to_string(),
        state: instance_state_label(instance.state.value).to_string(),
        data_class: instance.data_class.value.label().to_string(),
        created_at_epoch_seconds: instance.created_at_epoch_seconds.value,
        schema_version: instance.schema_version.value,
    }
}

fn cloud_compute_status_kind(error: &CloudComputeError) -> CloudComputeVmApiStatusKind {
    match error {
        CloudComputeError::DuplicateInstance
        | CloudComputeError::DuplicateKubernetesCluster
        | CloudComputeError::DuplicateFunction
        | CloudComputeError::DuplicateInvocation => CloudComputeVmApiStatusKind::Conflict,
        CloudComputeError::UnknownFunction => CloudComputeVmApiStatusKind::NotFound,
        CloudComputeError::ResourceTenantMismatch
        | CloudComputeError::ResourceRegionMismatch
        | CloudComputeError::ResidencyRegionMismatch
        | CloudComputeError::QuotaExceeded
        | CloudComputeError::PayloadDataClassNotAllowed => CloudComputeVmApiStatusKind::Forbidden,
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
        | CloudComputeError::FunctionNotActive => CloudComputeVmApiStatusKind::BadRequest,
    }
}

fn cloud_compute_message(error: &CloudComputeError) -> &'static str {
    match cloud_compute_status_kind(error) {
        CloudComputeVmApiStatusKind::BadRequest => "Cloud Compute rejected the request shape",
        CloudComputeVmApiStatusKind::Unauthorized => {
            "Cloud Compute authentication evidence is missing"
        }
        CloudComputeVmApiStatusKind::Forbidden => "Cloud Compute policy denied the request",
        CloudComputeVmApiStatusKind::NotFound => "Cloud Compute resource was not found",
        CloudComputeVmApiStatusKind::Conflict => "Cloud Compute resource already exists",
        CloudComputeVmApiStatusKind::UnprocessableEntity => {
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
        CloudComputeError::QuotaExceeded => "requested VM exceeds tenant quota envelope",
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

fn detail(field: &str, issue: &str) -> CloudComputeVmApiErrorDetail {
    CloudComputeVmApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}
