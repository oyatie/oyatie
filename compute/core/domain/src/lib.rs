//! Cloud compute aggregate kernel.
//!
//! This crate owns the stable `cloud.compute.*` metadata contracts for VM,
//! managed Kubernetes, and function invocation surfaces. Hypervisors,
//! schedulers, registries, and function runtimes consume these typed contracts
//! through adapters; this kernel stays adapter-free and keeps placement,
//! quota, identity, image, and data-class invariants explicit.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use cell_region::{AzCode, CellId, RegionCode};
use compute_resource::{
    CloudResourceError, FunctionRuntime, InstanceFlavor, K8sFlavor, ResourceId, ResourceKind,
};
use iam_cloud_domain::IamRoleId;
use network_domain::SecurityGroupId;
use network_residency::{ResidencyClass, residency_class_allows_home_region_label};
use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const COMPUTE_SCHEMA_VERSION: u32 = 1;
pub const MAX_FUNCTION_COLD_START_BUDGET_MS: u32 = 1_000;
const DEFAULT_FUNCTION_INVOCATION_RETENTION_LIMIT: usize = 1024;
const TENANT_ID_PREFIX: &str = "ten_";
const KEY_PAIR_ID_PREFIX: &str = "key_";
const NODE_POOL_ID_PREFIX: &str = "np_";
const FUNCTION_INVOCATION_ID_PREFIX: &str = "fninv_";
const USER_DATA_URI_PREFIX: &str = "userdata/";
const OCI_IMAGE_PREFIX: &str = "oci://";
const QCOW2_IMAGE_PREFIX: &str = "qcow2://";
const FUNCTION_BUNDLE_PREFIX: &str = "function://";
pub const COMPUTE_WORKLOAD_IDENTITY_EVIDENCE_PREFIX: &str = "evidence/compute/identity/";
pub const COMPUTE_SCHEDULING_EVIDENCE_PREFIX: &str = "evidence/compute/scheduling/";
pub const COMPUTE_AUDIT_EVIDENCE_PREFIX: &str = "evidence/compute/audit/";
const KUBERNETES_SERVICE_ACCOUNT_KIND: &str = "ksa";
const FUNCTION_SERVICE_ACCOUNT_KIND: &str = "function-sa";

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ImageRef {
    pub value: String,      // data_class: INTERNAL_ONLY
    pub kind: ImageRefKind, // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ImageRefKind {
    Oci,
    Qcow2,
    FunctionBundle,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct KeyPairId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct UserDataUri {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct NodePoolId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct FunctionName {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct InvocationId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct IdempotencyKey {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ControlPlaneVersion {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ComputeFlavorSpec {
    pub class: InstanceFlavor, // data_class: PUBLIC
    pub vcpu: u32,             // data_class: PUBLIC
    pub memory_gb: u32,        // data_class: PUBLIC
    pub gpu_count: u32,        // data_class: PUBLIC
    pub local_ssd_gb: u32,     // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ComputeQuotaEnvelope {
    pub vcpu_limit: u32,           // data_class: INTERNAL_ONLY
    pub memory_gb_limit: u32,      // data_class: INTERNAL_ONLY
    pub gpu_limit: u32,            // data_class: INTERNAL_ONLY
    pub local_ssd_gb_limit: u32,   // data_class: INTERNAL_ONLY
    pub current_vcpu: u32,         // data_class: INTERNAL_ONLY
    pub current_memory_gb: u32,    // data_class: INTERNAL_ONLY
    pub current_gpu: u32,          // data_class: INTERNAL_ONLY
    pub current_local_ssd_gb: u32, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
struct ComputeUnits {
    vcpu: u32,         // data_class: INTERNAL_ONLY
    memory_gb: u32,    // data_class: INTERNAL_ONLY
    gpu_count: u32,    // data_class: INTERNAL_ONLY
    local_ssd_gb: u32, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum InstanceState {
    Pending,
    Running,
    Stopping,
    Stopped,
    Terminated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum KubernetesClusterState {
    Creating,
    Ready,
    Reconciling,
    Draining,
    Deleted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FunctionDeploymentState {
    Deploying,
    Active,
    Disabled,
    Deleting,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstanceCreate {
    pub resource_id: String,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub region: String,                // data_class: PUBLIC
    pub az: String,                    // data_class: PUBLIC
    pub cell_id: String,               // data_class: PUBLIC
    pub flavor: ComputeFlavorSpec,     // data_class: PUBLIC
    pub image: String,                 // data_class: INTERNAL_ONLY
    pub key_pair: Option<String>,      // data_class: INTERNAL_ONLY
    pub vpc_id: String,                // data_class: INTERNAL_ONLY
    pub subnet_id: String,             // data_class: INTERNAL_ONLY
    pub security_groups: Vec<String>,  // data_class: INTERNAL_ONLY
    pub iam_role: Option<String>,      // data_class: INTERNAL_ONLY
    pub user_data_uri: Option<String>, // data_class: INTERNAL_ONLY
    pub quota: ComputeQuotaEnvelope,   // data_class: INTERNAL_ONLY
    pub residency: ResidencyClass,     // data_class: INTERNAL_ONLY
    pub state: InstanceState,          // data_class: PUBLIC
    pub data_class: DataClass,         // data_class: PUBLIC
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Instance {
    pub resource_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,      // data_class: PUBLIC
    pub az: Classified<AzCode>,              // data_class: PUBLIC
    pub cell_id: Classified<CellId>,         // data_class: PUBLIC
    pub flavor: Classified<ComputeFlavorSpec>, // data_class: PUBLIC
    pub image: Classified<ImageRef>,         // data_class: INTERNAL_ONLY
    pub key_pair: Classified<Option<KeyPairId>>, // data_class: INTERNAL_ONLY
    pub vpc_id: Classified<ResourceId>,      // data_class: INTERNAL_ONLY
    pub subnet_id: Classified<ResourceId>,   // data_class: INTERNAL_ONLY
    pub security_groups: Classified<Vec<SecurityGroupId>>, // data_class: INTERNAL_ONLY
    pub iam_role: Classified<Option<IamRoleId>>, // data_class: INTERNAL_ONLY
    pub user_data_uri: Classified<Option<UserDataUri>>, // data_class: INTERNAL_ONLY
    pub residency: Classified<ResidencyClass>, // data_class: INTERNAL_ONLY
    pub state: Classified<InstanceState>,    // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KubernetesNodePoolCreate {
    pub id: String,                   // data_class: INTERNAL_ONLY
    pub az: String,                   // data_class: PUBLIC
    pub cell_id: String,              // data_class: PUBLIC
    pub subnet_id: String,            // data_class: INTERNAL_ONLY
    pub security_groups: Vec<String>, // data_class: INTERNAL_ONLY
    pub flavor: ComputeFlavorSpec,    // data_class: PUBLIC
    pub min_nodes: u32,               // data_class: PUBLIC
    pub max_nodes: u32,               // data_class: PUBLIC
    pub autoscaling_enabled: bool,    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KubernetesNodePool {
    pub id: NodePoolId,                        // data_class: INTERNAL_ONLY
    pub az: AzCode,                            // data_class: PUBLIC
    pub cell_id: CellId,                       // data_class: PUBLIC
    pub subnet_id: ResourceId,                 // data_class: INTERNAL_ONLY
    pub security_groups: Vec<SecurityGroupId>, // data_class: INTERNAL_ONLY
    pub flavor: ComputeFlavorSpec,             // data_class: PUBLIC
    pub min_nodes: u32,                        // data_class: PUBLIC
    pub max_nodes: u32,                        // data_class: PUBLIC
    pub autoscaling_enabled: bool,             // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KubernetesClusterCreate {
    pub resource_id: String,                       // data_class: INTERNAL_ONLY
    pub tenant_id: String,                         // data_class: INTERNAL_ONLY
    pub region: String,                            // data_class: PUBLIC
    pub flavor: K8sFlavor,                         // data_class: PUBLIC
    pub control_plane_version: String,             // data_class: PUBLIC
    pub control_plane_private: bool,               // data_class: PUBLIC
    pub node_pools: Vec<KubernetesNodePoolCreate>, // data_class: INTERNAL_ONLY
    pub quota: ComputeQuotaEnvelope,               // data_class: INTERNAL_ONLY
    pub residency: ResidencyClass,                 // data_class: INTERNAL_ONLY
    pub state: KubernetesClusterState,             // data_class: PUBLIC
    pub data_class: DataClass,                     // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KubernetesCluster {
    pub resource_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,      // data_class: PUBLIC
    pub flavor: Classified<K8sFlavor>,       // data_class: PUBLIC
    pub control_plane_version: Classified<ControlPlaneVersion>, // data_class: PUBLIC
    pub control_plane_private: Classified<bool>, // data_class: PUBLIC
    pub node_pools: Classified<Vec<KubernetesNodePool>>, // data_class: INTERNAL_ONLY
    pub residency: Classified<ResidencyClass>, // data_class: INTERNAL_ONLY
    pub state: Classified<KubernetesClusterState>, // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionDeploymentCreate {
    pub resource_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub region: String,                       // data_class: PUBLIC
    pub az: String,                           // data_class: PUBLIC
    pub cell_id: String,                      // data_class: PUBLIC
    pub runtime: FunctionRuntime,             // data_class: PUBLIC
    pub name: String,                         // data_class: PUBLIC
    pub bundle: String,                       // data_class: INTERNAL_ONLY
    pub cold_start_budget_ms: u32,            // data_class: PUBLIC
    pub timeout_ms: u32,                      // data_class: PUBLIC
    pub memory_mb: u32,                       // data_class: PUBLIC
    pub max_concurrency: u32,                 // data_class: PUBLIC
    pub allowed_data_classes: Vec<DataClass>, // data_class: INTERNAL_ONLY
    pub residency: ResidencyClass,            // data_class: INTERNAL_ONLY
    pub state: FunctionDeploymentState,       // data_class: PUBLIC
    pub data_class: DataClass,                // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionDeployment {
    pub resource_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,      // data_class: PUBLIC
    pub az: Classified<AzCode>,              // data_class: PUBLIC
    pub cell_id: Classified<CellId>,         // data_class: PUBLIC
    pub runtime: Classified<FunctionRuntime>, // data_class: PUBLIC
    pub name: Classified<FunctionName>,      // data_class: PUBLIC
    pub bundle: Classified<ImageRef>,        // data_class: INTERNAL_ONLY
    pub cold_start_budget_ms: Classified<u32>, // data_class: PUBLIC
    pub timeout_ms: Classified<u32>,         // data_class: PUBLIC
    pub memory_mb: Classified<u32>,          // data_class: PUBLIC
    pub max_concurrency: Classified<u32>,    // data_class: PUBLIC
    pub allowed_data_classes: Classified<Vec<PrivacyDataClass>>, // data_class: INTERNAL_ONLY
    pub residency: Classified<ResidencyClass>, // data_class: INTERNAL_ONLY
    pub state: Classified<FunctionDeploymentState>, // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionInvocationRequest {
    pub invocation_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub function_id: String,                 // data_class: INTERNAL_ONLY
    pub region: String,                      // data_class: PUBLIC
    pub payload_data_class: DataClass,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String,             // data_class: INTERNAL_ONLY
    pub current_concurrent_invocations: u32, // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionInvocationReceipt {
    pub invocation_id: Classified<InvocationId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,           // data_class: INTERNAL_ONLY
    pub function_id: Classified<ResourceId>,     // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,          // data_class: PUBLIC
    pub payload_data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<IdempotencyKey>, // data_class: INTERNAL_ONLY
    pub cold_start_budget_ms: Classified<u32>,   // data_class: PUBLIC
    pub accepted_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,         // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ComputeWorkloadIsolation {
    SharedHost,
    HardenedNode,
    HardwareVirtualizedVm,
    KataMicroVm,
    FirecrackerMicroVm,
    GvisorSandbox,
    WasmSandbox,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComputeTenantCellGuardrailCreate {
    pub tenant_id: String,                              // data_class: INTERNAL_ONLY
    pub region: String,                                 // data_class: PUBLIC
    pub primary_cell_id: String,                        // data_class: PUBLIC
    pub vm_instance_id: String,                         // data_class: INTERNAL_ONLY
    pub vm_iam_role: String,                            // data_class: INTERNAL_ONLY
    pub vm_runtime_isolation: ComputeWorkloadIsolation, // data_class: PUBLIC
    pub k8s_cluster_id: String,                         // data_class: INTERNAL_ONLY
    pub k8s_service_account_ref: String,                // data_class: INTERNAL_ONLY
    pub k8s_private_control_plane: bool,                // data_class: PUBLIC
    pub k8s_pod_security_restricted: bool,              // data_class: PUBLIC
    pub k8s_topology_spread_required: bool,             // data_class: PUBLIC
    pub k8s_runtime_isolation: ComputeWorkloadIsolation, // data_class: PUBLIC
    pub function_id: String,                            // data_class: INTERNAL_ONLY
    pub function_service_account_ref: String,           // data_class: INTERNAL_ONLY
    pub function_runtime_isolation: ComputeWorkloadIsolation, // data_class: PUBLIC
    pub audit_evidence_ref: String,                     // data_class: INTERNAL_ONLY
    pub scheduling_evidence_ref: String,                // data_class: INTERNAL_ONLY
    pub identity_evidence_refs: Vec<String>,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComputeTenantCellGuardrail {
    pub tenant_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>, // data_class: PUBLIC
    pub primary_cell_id: Classified<CellId>, // data_class: PUBLIC
    pub vm_instance_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub vm_iam_role: Classified<IamRoleId>, // data_class: INTERNAL_ONLY
    pub vm_runtime_isolation: Classified<ComputeWorkloadIsolation>, // data_class: PUBLIC
    pub k8s_cluster_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub k8s_service_account_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub k8s_private_control_plane: Classified<bool>, // data_class: PUBLIC
    pub k8s_pod_security_restricted: Classified<bool>, // data_class: PUBLIC
    pub k8s_topology_spread_required: Classified<bool>, // data_class: PUBLIC
    pub k8s_runtime_isolation: Classified<ComputeWorkloadIsolation>, // data_class: PUBLIC
    pub function_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub function_service_account_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub function_runtime_isolation: Classified<ComputeWorkloadIsolation>, // data_class: PUBLIC
    pub audit_evidence_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub scheduling_evidence_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub identity_evidence_refs: Classified<Vec<String>>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudComputeError {
    InvalidTenantId,
    InvalidResourceId,
    ResourceTenantMismatch,
    ResourceRegionMismatch,
    ResourceKindMismatch,
    InvalidAzCode,
    AzRegionMismatch,
    InvalidCellId,
    CellAzMismatch,
    ResidencyRegionMismatch,
    InvalidDataClass,
    InvalidImageRef,
    InvalidKeyPairId,
    InvalidUserDataUri,
    InvalidWorkloadIdentityPolicy,
    InvalidRuntimeIsolationPolicy,
    InvalidSchedulingPolicy,
    InvalidAuditEvidenceRef,
    InvalidFlavor,
    InvalidQuota,
    QuotaExceeded,
    InvalidInstanceState,
    InvalidKubernetesState,
    InvalidFunctionState,
    InvalidNodePoolId,
    DuplicateNodePool,
    InvalidNodePoolShape,
    KubernetesHaRequiresThreeAzs,
    InvalidControlPlaneVersion,
    InvalidFunctionName,
    InvalidFunctionBudget,
    InvalidInvocationId,
    InvalidIdempotencyKey,
    FunctionNotActive,
    PayloadDataClassNotAllowed,
    DuplicateInstance,
    DuplicateKubernetesCluster,
    DuplicateFunction,
    DuplicateInvocation,
    UnknownFunction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeCatalog {
    instances: BTreeMap<ResourceId, Instance>,
    kubernetes_clusters: BTreeMap<ResourceId, KubernetesCluster>,
    functions: BTreeMap<ResourceId, FunctionDeployment>,
    invocations: BTreeMap<InvocationId, FunctionInvocationReceipt>,
    invocation_retention_limit: usize,
}

impl Default for CloudComputeCatalog {
    fn default() -> Self {
        Self::with_invocation_retention_limit(DEFAULT_FUNCTION_INVOCATION_RETENTION_LIMIT)
    }
}

pub trait ComputeRepo {
    fn create_instance(&mut self, input: InstanceCreate) -> Result<Instance, CloudComputeError>;
    fn create_kubernetes_cluster(
        &mut self,
        input: KubernetesClusterCreate,
    ) -> Result<KubernetesCluster, CloudComputeError>;
    fn register_function(
        &mut self,
        input: FunctionDeploymentCreate,
    ) -> Result<FunctionDeployment, CloudComputeError>;
    fn activate_function(
        &mut self,
        id: &ResourceId,
    ) -> Result<FunctionDeployment, CloudComputeError>;
    fn invoke_function(
        &mut self,
        input: FunctionInvocationRequest,
    ) -> Result<FunctionInvocationReceipt, CloudComputeError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ComputeProviderKind {
    AwsEc2,
    OciCompute,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComputeProviderVmCreateRequest {
    pub request_id: String,              // data_class: INTERNAL_ONLY
    pub provider_instance_ref: String,   // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub actor: String,                   // data_class: INTERNAL_ONLY
    pub idempotency_key: String,         // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub instance: Instance,              // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComputeProviderVmReceipt {
    pub provider_kind: ComputeProviderKind, // data_class: PUBLIC
    pub provider_request_id: String,        // data_class: INTERNAL_ONLY
    pub provider_instance_ref: String,      // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String,      // data_class: INTERNAL_ONLY
    pub instance_resource_id: String,       // data_class: INTERNAL_ONLY
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub region: String,                     // data_class: PUBLIC
    pub az: String,                         // data_class: PUBLIC
    pub cell_id: String,                    // data_class: PUBLIC
    pub schema_version: u32,                // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComputeProviderVmError {
    InvalidRequest,
    ProviderRejected {
        provider: ComputeProviderKind, // data_class: PUBLIC
        reason: String,                // data_class: INTERNAL_ONLY
    },
}

pub trait ComputeProviderVmPort {
    fn provider_kind(&self) -> ComputeProviderKind;
    fn create_vm(
        &self,
        input: ComputeProviderVmCreateRequest,
    ) -> Result<ComputeProviderVmReceipt, ComputeProviderVmError>;
}

impl ImageRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudComputeError> {
        let value = value.into();
        let kind = if value.starts_with(OCI_IMAGE_PREFIX) {
            ImageRefKind::Oci
        } else if value.starts_with(QCOW2_IMAGE_PREFIX) {
            ImageRefKind::Qcow2
        } else if value.starts_with(FUNCTION_BUNDLE_PREFIX) {
            ImageRefKind::FunctionBundle
        } else {
            return Err(CloudComputeError::InvalidImageRef);
        };
        let Some((uri, digest)) = value.rsplit_once("@sha256:") else {
            return Err(CloudComputeError::InvalidImageRef);
        };
        if uri.len() <= kind.prefix().len()
            || digest.len() != 64
            || !digest.bytes().all(|byte| byte.is_ascii_hexdigit())
            || value
                .bytes()
                .any(|byte| byte.is_ascii_control() || byte == b' ')
        {
            return Err(CloudComputeError::InvalidImageRef);
        }
        Ok(Self { value, kind })
    }

    pub const fn is_function_bundle(&self) -> bool {
        matches!(self.kind, ImageRefKind::FunctionBundle)
    }
}

impl ImageRefKind {
    const fn prefix(self) -> &'static str {
        match self {
            Self::Oci => OCI_IMAGE_PREFIX,
            Self::Qcow2 => QCOW2_IMAGE_PREFIX,
            Self::FunctionBundle => FUNCTION_BUNDLE_PREFIX,
        }
    }
}

impl KeyPairId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudComputeError> {
        prefixed_token(
            value.into(),
            KEY_PAIR_ID_PREFIX,
            CloudComputeError::InvalidKeyPairId,
        )
        .map(|value| Self { value })
    }
}

impl UserDataUri {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudComputeError> {
        prefixed_token(
            value.into(),
            USER_DATA_URI_PREFIX,
            CloudComputeError::InvalidUserDataUri,
        )
        .map(|value| Self { value })
    }
}

impl NodePoolId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudComputeError> {
        prefixed_token(
            value.into(),
            NODE_POOL_ID_PREFIX,
            CloudComputeError::InvalidNodePoolId,
        )
        .map(|value| Self { value })
    }
}

impl FunctionName {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudComputeError> {
        let value = value.into();
        if (3..=64).contains(&value.len())
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
            && !value.starts_with('-')
            && !value.ends_with('-')
        {
            Ok(Self { value })
        } else {
            Err(CloudComputeError::InvalidFunctionName)
        }
    }
}

impl InvocationId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudComputeError> {
        prefixed_token(
            value.into(),
            FUNCTION_INVOCATION_ID_PREFIX,
            CloudComputeError::InvalidInvocationId,
        )
        .map(|value| Self { value })
    }
}

impl IdempotencyKey {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudComputeError> {
        let value = value.into();
        if (16..=128).contains(&value.len())
            && !value
                .bytes()
                .any(|byte| byte.is_ascii_control() || byte == b' ')
        {
            Ok(Self { value })
        } else {
            Err(CloudComputeError::InvalidIdempotencyKey)
        }
    }
}

impl ComputeProviderKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::AwsEc2 => "aws_ec2",
            Self::OciCompute => "oci_compute",
        }
    }
}

impl ComputeProviderVmCreateRequest {
    pub fn validate(&self) -> Result<(), ComputeProviderVmError> {
        if self.request_id.trim().is_empty()
            || self.provider_instance_ref.trim().is_empty()
            || self.tenant_id.trim().is_empty()
            || self.actor.trim().is_empty()
            || IdempotencyKey::new(self.idempotency_key.clone()).is_err()
            || self.tenant_id != self.instance.tenant_id.value
        {
            return Err(ComputeProviderVmError::InvalidRequest);
        }
        Ok(())
    }
}

impl ComputeProviderVmReceipt {
    pub fn from_request(
        provider_kind: ComputeProviderKind,
        request: ComputeProviderVmCreateRequest,
        provider_request_id: impl Into<String>,
        provider_evidence_ref: impl Into<String>,
    ) -> Result<Self, ComputeProviderVmError> {
        request.validate()?;
        let provider_request_id = provider_request_id.into();
        let provider_evidence_ref = provider_evidence_ref.into();
        if provider_request_id.trim().is_empty() || provider_evidence_ref.trim().is_empty() {
            return Err(ComputeProviderVmError::InvalidRequest);
        }
        Ok(Self {
            provider_kind,
            provider_request_id,
            provider_instance_ref: request.provider_instance_ref,
            provider_evidence_ref,
            instance_resource_id: request.instance.resource_id.value.value,
            tenant_id: request.tenant_id,
            region: request.instance.region.value.value,
            az: request.instance.az.value.value,
            cell_id: request.instance.cell_id.value.value,
            schema_version: COMPUTE_SCHEMA_VERSION,
        })
    }
}

impl ComputeWorkloadIsolation {
    pub const fn label(self) -> &'static str {
        match self {
            Self::SharedHost => "shared_host",
            Self::HardenedNode => "hardened_node",
            Self::HardwareVirtualizedVm => "hardware_virtualized_vm",
            Self::KataMicroVm => "kata_microvm",
            Self::FirecrackerMicroVm => "firecracker_microvm",
            Self::GvisorSandbox => "gvisor_sandbox",
            Self::WasmSandbox => "wasm_sandbox",
        }
    }

    const fn satisfies_tenant_isolation(self) -> bool {
        matches!(
            self,
            Self::HardwareVirtualizedVm
                | Self::KataMicroVm
                | Self::FirecrackerMicroVm
                | Self::GvisorSandbox
                | Self::WasmSandbox
        )
    }
}

impl ComputeTenantCellGuardrail {
    pub fn new(input: ComputeTenantCellGuardrailCreate) -> Result<Self, CloudComputeError> {
        validate_tenant_id(&input.tenant_id)?;
        let region =
            RegionCode::new(input.region).map_err(|_| CloudComputeError::InvalidResourceId)?;
        let primary_cell_id =
            CellId::new(input.primary_cell_id).map_err(|_| CloudComputeError::InvalidCellId)?;
        validate_cell_region(&primary_cell_id, &region)?;

        let vm_instance_id = resource_id_for_kind_label(
            &input.vm_instance_id,
            &input.tenant_id,
            &region,
            "instance",
        )?;
        let k8s_cluster_id =
            resource_id_for_kind_label(&input.k8s_cluster_id, &input.tenant_id, &region, "k8s")?;
        let function_id =
            resource_id_for_kind_label(&input.function_id, &input.tenant_id, &region, "function")?;

        let vm_iam_role = IamRoleId::new(input.vm_iam_role)
            .map_err(|_| CloudComputeError::InvalidWorkloadIdentityPolicy)?;
        let k8s_service_account_ref = validate_workload_identity_ref(
            input.k8s_service_account_ref,
            KUBERNETES_SERVICE_ACCOUNT_KIND,
            &input.tenant_id,
            &primary_cell_id,
        )?;
        let function_service_account_ref = validate_workload_identity_ref(
            input.function_service_account_ref,
            FUNCTION_SERVICE_ACCOUNT_KIND,
            &input.tenant_id,
            &primary_cell_id,
        )?;

        validate_runtime_isolation(input.vm_runtime_isolation)?;
        validate_runtime_isolation(input.k8s_runtime_isolation)?;
        validate_runtime_isolation(input.function_runtime_isolation)?;

        if !input.k8s_private_control_plane || !input.k8s_pod_security_restricted {
            return Err(CloudComputeError::InvalidRuntimeIsolationPolicy);
        }
        if !input.k8s_topology_spread_required {
            return Err(CloudComputeError::InvalidSchedulingPolicy);
        }

        let audit_evidence_ref = validate_metadata_ref(
            input.audit_evidence_ref,
            COMPUTE_AUDIT_EVIDENCE_PREFIX,
            CloudComputeError::InvalidAuditEvidenceRef,
        )?;
        let scheduling_evidence_ref = validate_metadata_ref(
            input.scheduling_evidence_ref,
            COMPUTE_SCHEDULING_EVIDENCE_PREFIX,
            CloudComputeError::InvalidSchedulingPolicy,
        )?;
        let identity_evidence_refs = validate_metadata_refs(
            input.identity_evidence_refs,
            COMPUTE_WORKLOAD_IDENTITY_EVIDENCE_PREFIX,
            CloudComputeError::InvalidWorkloadIdentityPolicy,
        )?;

        Ok(Self {
            tenant_id: internal(input.tenant_id),
            region: public(region),
            primary_cell_id: public(primary_cell_id),
            vm_instance_id: internal(vm_instance_id),
            vm_iam_role: internal(vm_iam_role),
            vm_runtime_isolation: public(input.vm_runtime_isolation),
            k8s_cluster_id: internal(k8s_cluster_id),
            k8s_service_account_ref: internal(k8s_service_account_ref),
            k8s_private_control_plane: public(input.k8s_private_control_plane),
            k8s_pod_security_restricted: public(input.k8s_pod_security_restricted),
            k8s_topology_spread_required: public(input.k8s_topology_spread_required),
            k8s_runtime_isolation: public(input.k8s_runtime_isolation),
            function_id: internal(function_id),
            function_service_account_ref: internal(function_service_account_ref),
            function_runtime_isolation: public(input.function_runtime_isolation),
            audit_evidence_ref: internal(audit_evidence_ref),
            scheduling_evidence_ref: internal(scheduling_evidence_ref),
            identity_evidence_refs: internal(identity_evidence_refs),
            schema_version: public(COMPUTE_SCHEMA_VERSION),
        })
    }
}

impl ControlPlaneVersion {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudComputeError> {
        let value = value.into();
        let Some(rest) = value.strip_prefix('v') else {
            return Err(CloudComputeError::InvalidControlPlaneVersion);
        };
        if rest.split('.').count() >= 3
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
        {
            Ok(Self { value })
        } else {
            Err(CloudComputeError::InvalidControlPlaneVersion)
        }
    }
}

impl ComputeFlavorSpec {
    pub fn validate(self) -> Result<Self, CloudComputeError> {
        if self.vcpu == 0
            || self.vcpu > 512
            || self.memory_gb == 0
            || self.memory_gb > 8192
            || self.memory_gb < self.vcpu
            || self.local_ssd_gb > 262_144
        {
            return Err(CloudComputeError::InvalidFlavor);
        }
        if matches!(self.class, InstanceFlavor::Gpu) != (self.gpu_count > 0) {
            return Err(CloudComputeError::InvalidFlavor);
        }
        Ok(self)
    }

    fn units(self) -> ComputeUnits {
        ComputeUnits {
            vcpu: self.vcpu,
            memory_gb: self.memory_gb,
            gpu_count: self.gpu_count,
            local_ssd_gb: self.local_ssd_gb,
        }
    }
}

impl ComputeQuotaEnvelope {
    fn admit(self, requested: ComputeUnits) -> Result<(), CloudComputeError> {
        if self.current_vcpu > self.vcpu_limit
            || self.current_memory_gb > self.memory_gb_limit
            || self.current_gpu > self.gpu_limit
            || self.current_local_ssd_gb > self.local_ssd_gb_limit
        {
            return Err(CloudComputeError::InvalidQuota);
        }
        let next_vcpu = self
            .current_vcpu
            .checked_add(requested.vcpu)
            .ok_or(CloudComputeError::QuotaExceeded)?;
        let next_memory = self
            .current_memory_gb
            .checked_add(requested.memory_gb)
            .ok_or(CloudComputeError::QuotaExceeded)?;
        let next_gpu = self
            .current_gpu
            .checked_add(requested.gpu_count)
            .ok_or(CloudComputeError::QuotaExceeded)?;
        let next_ssd = self
            .current_local_ssd_gb
            .checked_add(requested.local_ssd_gb)
            .ok_or(CloudComputeError::QuotaExceeded)?;
        if next_vcpu > self.vcpu_limit
            || next_memory > self.memory_gb_limit
            || next_gpu > self.gpu_limit
            || next_ssd > self.local_ssd_gb_limit
        {
            return Err(CloudComputeError::QuotaExceeded);
        }
        Ok(())
    }
}

impl ComputeUnits {
    fn checked_add(self, other: Self) -> Result<Self, CloudComputeError> {
        Ok(Self {
            vcpu: self
                .vcpu
                .checked_add(other.vcpu)
                .ok_or(CloudComputeError::QuotaExceeded)?,
            memory_gb: self
                .memory_gb
                .checked_add(other.memory_gb)
                .ok_or(CloudComputeError::QuotaExceeded)?,
            gpu_count: self
                .gpu_count
                .checked_add(other.gpu_count)
                .ok_or(CloudComputeError::QuotaExceeded)?,
            local_ssd_gb: self
                .local_ssd_gb
                .checked_add(other.local_ssd_gb)
                .ok_or(CloudComputeError::QuotaExceeded)?,
        })
    }

    fn checked_mul(self, count: u32) -> Result<Self, CloudComputeError> {
        Ok(Self {
            vcpu: self
                .vcpu
                .checked_mul(count)
                .ok_or(CloudComputeError::QuotaExceeded)?,
            memory_gb: self
                .memory_gb
                .checked_mul(count)
                .ok_or(CloudComputeError::QuotaExceeded)?,
            gpu_count: self
                .gpu_count
                .checked_mul(count)
                .ok_or(CloudComputeError::QuotaExceeded)?,
            local_ssd_gb: self
                .local_ssd_gb
                .checked_mul(count)
                .ok_or(CloudComputeError::QuotaExceeded)?,
        })
    }
}

impl Instance {
    pub fn new(input: InstanceCreate) -> Result<Self, CloudComputeError> {
        validate_tenant_id(&input.tenant_id)?;
        if input.state != InstanceState::Pending {
            return Err(CloudComputeError::InvalidInstanceState);
        }
        let region = region_for(&input.region, &input.residency)?;
        let az = AzCode::new(input.az).map_err(|_| CloudComputeError::InvalidAzCode)?;
        validate_az_region(&az, &region)?;
        let cell_id = CellId::new(input.cell_id).map_err(|_| CloudComputeError::InvalidCellId)?;
        validate_cell_az(&cell_id, &az)?;
        let flavor = input.flavor.validate()?;
        input.quota.admit(flavor.units())?;
        let resource_id = resource_id_for(
            &input.resource_id,
            &input.tenant_id,
            &region,
            ResourceKind::ComputeInstance(flavor.class),
        )?;
        let image = ImageRef::new(input.image)?;
        if image.is_function_bundle() {
            return Err(CloudComputeError::InvalidImageRef);
        }
        let vpc_id = resource_ref_for(&input.vpc_id, &input.tenant_id, &region, "vpc")?;
        let subnet_id = resource_ref_for(&input.subnet_id, &input.tenant_id, &region, "subnet")?;
        let security_groups = security_groups(input.security_groups)?;
        let key_pair = input.key_pair.map(KeyPairId::new).transpose()?;
        let iam_role = input
            .iam_role
            .map(IamRoleId::new)
            .transpose()
            .map_err(|_| CloudComputeError::ResourceKindMismatch)?;
        let user_data_uri = input.user_data_uri.map(UserDataUri::new).transpose()?;
        Ok(Self {
            resource_id: internal(resource_id),
            tenant_id: internal(input.tenant_id),
            region: public(region),
            az: public(az),
            cell_id: public(cell_id),
            flavor: public(flavor),
            image: internal(image),
            key_pair: internal(key_pair),
            vpc_id: internal(vpc_id),
            subnet_id: internal(subnet_id),
            security_groups: internal(security_groups),
            iam_role: internal(iam_role),
            user_data_uri: internal(user_data_uri),
            residency: internal(input.residency),
            state: public(input.state),
            data_class: public(public_metadata_class(input.data_class)?),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(COMPUTE_SCHEMA_VERSION),
        })
    }
}

impl KubernetesCluster {
    pub fn new(input: KubernetesClusterCreate) -> Result<Self, CloudComputeError> {
        validate_tenant_id(&input.tenant_id)?;
        if input.state != KubernetesClusterState::Creating {
            return Err(CloudComputeError::InvalidKubernetesState);
        }
        let region = region_for(&input.region, &input.residency)?;
        let resource_id = resource_id_for(
            &input.resource_id,
            &input.tenant_id,
            &region,
            ResourceKind::KubernetesCluster(input.flavor),
        )?;
        let control_plane_version = ControlPlaneVersion::new(input.control_plane_version)?;
        let node_pools = node_pools(&input.tenant_id, &region, input.node_pools)?;
        validate_kubernetes_shape(input.flavor, &node_pools)?;
        let requested = node_pools
            .iter()
            .try_fold(ComputeUnits::default(), |sum, pool| {
                let admitted_nodes = if pool.autoscaling_enabled {
                    pool.max_nodes
                } else {
                    pool.min_nodes
                };
                sum.checked_add(pool.flavor.units().checked_mul(admitted_nodes)?)
            })?;
        input.quota.admit(requested)?;
        Ok(Self {
            resource_id: internal(resource_id),
            tenant_id: internal(input.tenant_id),
            region: public(region),
            flavor: public(input.flavor),
            control_plane_version: public(control_plane_version),
            control_plane_private: public(input.control_plane_private),
            node_pools: internal(node_pools),
            residency: internal(input.residency),
            state: public(input.state),
            data_class: public(public_metadata_class(input.data_class)?),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(COMPUTE_SCHEMA_VERSION),
        })
    }
}

impl FunctionDeployment {
    pub fn new(input: FunctionDeploymentCreate) -> Result<Self, CloudComputeError> {
        validate_tenant_id(&input.tenant_id)?;
        if input.state != FunctionDeploymentState::Deploying {
            return Err(CloudComputeError::InvalidFunctionState);
        }
        let region = region_for(&input.region, &input.residency)?;
        let az = AzCode::new(input.az).map_err(|_| CloudComputeError::InvalidAzCode)?;
        validate_az_region(&az, &region)?;
        let cell_id = CellId::new(input.cell_id).map_err(|_| CloudComputeError::InvalidCellId)?;
        validate_cell_az(&cell_id, &az)?;
        let resource_id = resource_id_for(
            &input.resource_id,
            &input.tenant_id,
            &region,
            ResourceKind::Function(input.runtime),
        )?;
        let bundle = ImageRef::new(input.bundle)?;
        if !bundle.is_function_bundle() {
            return Err(CloudComputeError::InvalidImageRef);
        }
        if input.cold_start_budget_ms == 0
            || input.cold_start_budget_ms > MAX_FUNCTION_COLD_START_BUDGET_MS
            || !(100..=900_000).contains(&input.timeout_ms)
            || !(128..=10_240).contains(&input.memory_mb)
            || input.max_concurrency == 0
            || input.max_concurrency > 10_000
        {
            return Err(CloudComputeError::InvalidFunctionBudget);
        }
        let allowed_data_classes = privacy_classes(input.allowed_data_classes)?;
        if allowed_data_classes.is_empty() {
            return Err(CloudComputeError::PayloadDataClassNotAllowed);
        }
        Ok(Self {
            resource_id: internal(resource_id),
            tenant_id: internal(input.tenant_id),
            region: public(region),
            az: public(az),
            cell_id: public(cell_id),
            runtime: public(input.runtime),
            name: public(FunctionName::new(input.name)?),
            bundle: internal(bundle),
            cold_start_budget_ms: public(input.cold_start_budget_ms),
            timeout_ms: public(input.timeout_ms),
            memory_mb: public(input.memory_mb),
            max_concurrency: public(input.max_concurrency),
            allowed_data_classes: internal(allowed_data_classes),
            residency: internal(input.residency),
            state: public(input.state),
            data_class: public(public_metadata_class(input.data_class)?),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(COMPUTE_SCHEMA_VERSION),
        })
    }

    pub fn activate(mut self) -> Result<Self, CloudComputeError> {
        if self.state.value != FunctionDeploymentState::Deploying {
            return Err(CloudComputeError::InvalidFunctionState);
        }
        self.state = public(FunctionDeploymentState::Active);
        Ok(self)
    }

    pub fn invoke(
        &self,
        input: FunctionInvocationRequest,
    ) -> Result<FunctionInvocationReceipt, CloudComputeError> {
        if self.state.value != FunctionDeploymentState::Active {
            return Err(CloudComputeError::FunctionNotActive);
        }
        validate_tenant_id(&input.tenant_id)?;
        if input.tenant_id != self.tenant_id.value {
            return Err(CloudComputeError::ResourceTenantMismatch);
        }
        let region =
            RegionCode::new(input.region).map_err(|_| CloudComputeError::InvalidResourceId)?;
        if region != self.region.value {
            return Err(CloudComputeError::ResourceRegionMismatch);
        }
        let function_id = ResourceId::new(input.function_id).map_err(map_resource_error)?;
        if function_id != self.resource_id.value {
            return Err(CloudComputeError::UnknownFunction);
        }
        let payload_data_class = PrivacyDataClass::new(input.payload_data_class)
            .map_err(|_| CloudComputeError::InvalidDataClass)?;
        if !self
            .allowed_data_classes
            .value
            .contains(&payload_data_class)
        {
            return Err(CloudComputeError::PayloadDataClassNotAllowed);
        }
        if input.current_concurrent_invocations >= self.max_concurrency.value {
            return Err(CloudComputeError::QuotaExceeded);
        }
        Ok(FunctionInvocationReceipt {
            invocation_id: internal(InvocationId::new(input.invocation_id)?),
            tenant_id: internal(input.tenant_id),
            function_id: internal(function_id),
            region: public(region),
            payload_data_class: internal(payload_data_class),
            idempotency_key: internal(IdempotencyKey::new(input.idempotency_key)?),
            cold_start_budget_ms: public(self.cold_start_budget_ms.value),
            accepted_at_epoch_seconds: internal(input.requested_at_epoch_seconds),
            schema_version: public(COMPUTE_SCHEMA_VERSION),
        })
    }
}

impl ComputeRepo for CloudComputeCatalog {
    fn create_instance(&mut self, input: InstanceCreate) -> Result<Instance, CloudComputeError> {
        let instance = Instance::new(input)?;
        if self.instances.contains_key(&instance.resource_id.value) {
            return Err(CloudComputeError::DuplicateInstance);
        }
        self.instances
            .insert(instance.resource_id.value.clone(), instance.clone());
        Ok(instance)
    }

    fn create_kubernetes_cluster(
        &mut self,
        input: KubernetesClusterCreate,
    ) -> Result<KubernetesCluster, CloudComputeError> {
        let cluster = KubernetesCluster::new(input)?;
        if self
            .kubernetes_clusters
            .contains_key(&cluster.resource_id.value)
        {
            return Err(CloudComputeError::DuplicateKubernetesCluster);
        }
        self.kubernetes_clusters
            .insert(cluster.resource_id.value.clone(), cluster.clone());
        Ok(cluster)
    }

    fn register_function(
        &mut self,
        input: FunctionDeploymentCreate,
    ) -> Result<FunctionDeployment, CloudComputeError> {
        let function = FunctionDeployment::new(input)?;
        if self.functions.contains_key(&function.resource_id.value) {
            return Err(CloudComputeError::DuplicateFunction);
        }
        self.functions
            .insert(function.resource_id.value.clone(), function.clone());
        Ok(function)
    }

    fn activate_function(
        &mut self,
        id: &ResourceId,
    ) -> Result<FunctionDeployment, CloudComputeError> {
        let function = self
            .functions
            .get_mut(id)
            .ok_or(CloudComputeError::UnknownFunction)?;
        if function.state.value != FunctionDeploymentState::Deploying {
            return Err(CloudComputeError::InvalidFunctionState);
        }
        function.state = public(FunctionDeploymentState::Active);
        Ok(function.clone())
    }

    fn invoke_function(
        &mut self,
        input: FunctionInvocationRequest,
    ) -> Result<FunctionInvocationReceipt, CloudComputeError> {
        let invocation_id = InvocationId::new(input.invocation_id.clone())?;
        if self.invocations.contains_key(&invocation_id) {
            return Err(CloudComputeError::DuplicateInvocation);
        }
        let function_id = ResourceId::new(input.function_id.clone()).map_err(map_resource_error)?;
        let function = self
            .functions
            .get(&function_id)
            .ok_or(CloudComputeError::UnknownFunction)?;
        let receipt = function.invoke(input)?;
        self.remember_invocation(invocation_id, receipt.clone());
        Ok(receipt)
    }
}

impl CloudComputeCatalog {
    pub fn with_invocation_retention_limit(invocation_retention_limit: usize) -> Self {
        Self {
            instances: BTreeMap::new(),
            kubernetes_clusters: BTreeMap::new(),
            functions: BTreeMap::new(),
            invocations: BTreeMap::new(),
            invocation_retention_limit: invocation_retention_limit.max(1),
        }
    }

    fn remember_invocation(
        &mut self,
        invocation_id: InvocationId,
        receipt: FunctionInvocationReceipt,
    ) {
        if self.invocations.len() >= self.invocation_retention_limit
            && let Some(evicted) = self.invocations.keys().next().cloned()
        {
            self.invocations.remove(&evicted);
        }
        self.invocations.insert(invocation_id, receipt);
    }

    pub fn instances(&self) -> impl Iterator<Item = &Instance> {
        self.instances.values()
    }

    pub fn kubernetes_clusters(&self) -> impl Iterator<Item = &KubernetesCluster> {
        self.kubernetes_clusters.values()
    }

    pub fn functions(&self) -> impl Iterator<Item = &FunctionDeployment> {
        self.functions.values()
    }

    pub fn invocations(&self) -> impl Iterator<Item = &FunctionInvocationReceipt> {
        self.invocations.values()
    }
}

pub const fn instance_flavor_label(flavor: InstanceFlavor) -> &'static str {
    match flavor {
        InstanceFlavor::GeneralPurpose => "general_purpose",
        InstanceFlavor::ComputeOptimized => "compute_optimized",
        InstanceFlavor::MemoryOptimized => "memory_optimized",
        InstanceFlavor::Gpu => "gpu",
    }
}

pub const fn image_ref_kind_label(kind: ImageRefKind) -> &'static str {
    match kind {
        ImageRefKind::Oci => "oci",
        ImageRefKind::Qcow2 => "qcow2",
        ImageRefKind::FunctionBundle => "function_bundle",
    }
}

pub const fn compute_workload_isolation_label(isolation: ComputeWorkloadIsolation) -> &'static str {
    isolation.label()
}

pub const fn instance_state_label(state: InstanceState) -> &'static str {
    match state {
        InstanceState::Pending => "pending",
        InstanceState::Running => "running",
        InstanceState::Stopping => "stopping",
        InstanceState::Stopped => "stopped",
        InstanceState::Terminated => "terminated",
    }
}

fn node_pools(
    tenant_id: &str,
    region: &RegionCode,
    input: Vec<KubernetesNodePoolCreate>,
) -> Result<Vec<KubernetesNodePool>, CloudComputeError> {
    if input.is_empty() {
        return Err(CloudComputeError::InvalidNodePoolShape);
    }
    let mut seen = BTreeSet::new();
    let mut pools = Vec::with_capacity(input.len());
    for pool in input {
        let id = NodePoolId::new(pool.id)?;
        if !seen.insert(id.clone()) {
            return Err(CloudComputeError::DuplicateNodePool);
        }
        let az = AzCode::new(pool.az).map_err(|_| CloudComputeError::InvalidAzCode)?;
        validate_az_region(&az, region)?;
        let cell_id = CellId::new(pool.cell_id).map_err(|_| CloudComputeError::InvalidCellId)?;
        validate_cell_az(&cell_id, &az)?;
        let subnet_id = resource_ref_for(&pool.subnet_id, tenant_id, region, "subnet")?;
        let security_groups = security_groups(pool.security_groups)?;
        let flavor = pool.flavor.validate()?;
        if pool.min_nodes == 0 || pool.max_nodes < pool.min_nodes || pool.max_nodes > 1_000 {
            return Err(CloudComputeError::InvalidNodePoolShape);
        }
        pools.push(KubernetesNodePool {
            id,
            az,
            cell_id,
            subnet_id,
            security_groups,
            flavor,
            min_nodes: pool.min_nodes,
            max_nodes: pool.max_nodes,
            autoscaling_enabled: pool.autoscaling_enabled,
        });
    }
    Ok(pools)
}

fn validate_kubernetes_shape(
    flavor: K8sFlavor,
    node_pools: &[KubernetesNodePool],
) -> Result<(), CloudComputeError> {
    if matches!(flavor, K8sFlavor::HighAvailability) {
        let az_count = node_pools
            .iter()
            .map(|pool| pool.az.clone())
            .collect::<BTreeSet<_>>()
            .len();
        let min_nodes: u32 = node_pools.iter().map(|pool| pool.min_nodes).sum();
        if az_count < 3 || min_nodes < 3 {
            return Err(CloudComputeError::KubernetesHaRequiresThreeAzs);
        }
    }
    Ok(())
}

fn security_groups(input: Vec<String>) -> Result<Vec<SecurityGroupId>, CloudComputeError> {
    if input.is_empty() {
        return Err(CloudComputeError::ResourceKindMismatch);
    }
    let mut seen = BTreeSet::new();
    let mut groups = Vec::with_capacity(input.len());
    for id in input {
        let id = SecurityGroupId::new(id).map_err(|_| CloudComputeError::ResourceKindMismatch)?;
        if !seen.insert(id.clone()) {
            return Err(CloudComputeError::ResourceKindMismatch);
        }
        groups.push(id);
    }
    Ok(groups)
}

fn privacy_classes(input: Vec<DataClass>) -> Result<Vec<PrivacyDataClass>, CloudComputeError> {
    let mut seen = BTreeSet::new();
    let mut classes = Vec::with_capacity(input.len());
    for data_class in input {
        let data_class =
            PrivacyDataClass::new(data_class).map_err(|_| CloudComputeError::InvalidDataClass)?;
        if seen.insert(data_class) {
            classes.push(data_class);
        }
    }
    Ok(classes)
}

fn resource_id_for(
    value: &str,
    tenant_id: &str,
    region: &RegionCode,
    kind: ResourceKind,
) -> Result<ResourceId, CloudComputeError> {
    let id = ResourceId::new(value.to_string()).map_err(map_resource_error)?;
    if id.tenant_id().map_err(map_resource_error)? != tenant_id {
        return Err(CloudComputeError::ResourceTenantMismatch);
    }
    if id.region().map_err(map_resource_error)? != *region {
        return Err(CloudComputeError::ResourceRegionMismatch);
    }
    if id.kind_label().map_err(map_resource_error)? != kind.type_label() {
        return Err(CloudComputeError::ResourceKindMismatch);
    }
    Ok(id)
}

fn resource_ref_for(
    value: &str,
    tenant_id: &str,
    region: &RegionCode,
    kind_label: &str,
) -> Result<ResourceId, CloudComputeError> {
    let id = ResourceId::new(value.to_string()).map_err(map_resource_error)?;
    if id.tenant_id().map_err(map_resource_error)? != tenant_id {
        return Err(CloudComputeError::ResourceTenantMismatch);
    }
    if id.region().map_err(map_resource_error)? != *region {
        return Err(CloudComputeError::ResourceRegionMismatch);
    }
    if id.kind_label().map_err(map_resource_error)? != kind_label {
        return Err(CloudComputeError::ResourceKindMismatch);
    }
    Ok(id)
}

fn resource_id_for_kind_label(
    value: &str,
    tenant_id: &str,
    region: &RegionCode,
    kind_label: &str,
) -> Result<ResourceId, CloudComputeError> {
    let id = ResourceId::new(value.to_string()).map_err(map_resource_error)?;
    if id.tenant_id().map_err(map_resource_error)? != tenant_id {
        return Err(CloudComputeError::ResourceTenantMismatch);
    }
    if id.region().map_err(map_resource_error)? != *region {
        return Err(CloudComputeError::ResourceRegionMismatch);
    }
    if id.kind_label().map_err(map_resource_error)? != kind_label {
        return Err(CloudComputeError::ResourceKindMismatch);
    }
    Ok(id)
}

fn region_for(value: &str, residency: &ResidencyClass) -> Result<RegionCode, CloudComputeError> {
    let region =
        RegionCode::new(value.to_string()).map_err(|_| CloudComputeError::InvalidResourceId)?;
    if !residency_class_allows_home_region_label(residency, &region.value) {
        return Err(CloudComputeError::ResidencyRegionMismatch);
    }
    Ok(region)
}

fn validate_tenant_id(value: &str) -> Result<(), CloudComputeError> {
    let Some(suffix) = value.strip_prefix(TENANT_ID_PREFIX) else {
        return Err(CloudComputeError::InvalidTenantId);
    };
    if suffix.is_empty()
        || suffix.starts_with('-')
        || suffix.ends_with('-')
        || suffix.contains("--")
        || !suffix.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
    {
        return Err(CloudComputeError::InvalidTenantId);
    }
    Ok(())
}

fn validate_az_region(az: &AzCode, region: &RegionCode) -> Result<(), CloudComputeError> {
    if az.value == region.value
        || az
            .value
            .strip_prefix(&region.value)
            .is_some_and(|suffix| suffix.starts_with('-') && suffix.len() > 1)
    {
        Ok(())
    } else {
        Err(CloudComputeError::AzRegionMismatch)
    }
}

fn validate_cell_az(cell_id: &CellId, az: &AzCode) -> Result<(), CloudComputeError> {
    let expected_prefix = format!("cell-{}-", az.value);
    if cell_id.value.starts_with(&expected_prefix) {
        Ok(())
    } else {
        Err(CloudComputeError::CellAzMismatch)
    }
}

fn validate_cell_region(cell_id: &CellId, region: &RegionCode) -> Result<(), CloudComputeError> {
    let expected_prefix = format!("cell-{}-", region.value);
    if cell_id.value.starts_with(&expected_prefix) {
        Ok(())
    } else {
        Err(CloudComputeError::CellAzMismatch)
    }
}

fn validate_runtime_isolation(
    isolation: ComputeWorkloadIsolation,
) -> Result<(), CloudComputeError> {
    if isolation.satisfies_tenant_isolation() {
        Ok(())
    } else {
        Err(CloudComputeError::InvalidRuntimeIsolationPolicy)
    }
}

fn validate_workload_identity_ref(
    value: String,
    expected_kind: &str,
    tenant_id: &str,
    cell_id: &CellId,
) -> Result<String, CloudComputeError> {
    if value.trim() != value
        || value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
        || looks_secret_like(&value)
    {
        return Err(CloudComputeError::InvalidWorkloadIdentityPolicy);
    }

    let mut segments = value.split('/');
    let kind = segments.next();
    let tenant = segments.next();
    let cell = segments.next();
    let name = segments.next();
    if kind != Some(expected_kind)
        || tenant != Some(tenant_id)
        || cell != Some(cell_id.value.as_str())
        || !name.is_some_and(safe_ref_token)
        || segments.next().is_some()
    {
        return Err(CloudComputeError::InvalidWorkloadIdentityPolicy);
    }
    Ok(value)
}

fn validate_metadata_refs(
    input: Vec<String>,
    prefix: &str,
    error: CloudComputeError,
) -> Result<Vec<String>, CloudComputeError> {
    if input.is_empty() {
        return Err(error);
    }
    let mut seen = BTreeSet::new();
    let mut refs = Vec::with_capacity(input.len());
    for value in input {
        let value = validate_metadata_ref(value, prefix, error.clone())?;
        if !seen.insert(value.clone()) {
            return Err(error);
        }
        refs.push(value);
    }
    Ok(refs)
}

fn validate_metadata_ref(
    value: String,
    prefix: &str,
    error: CloudComputeError,
) -> Result<String, CloudComputeError> {
    if value.trim() != value
        || !value.starts_with(prefix)
        || value.len() == prefix.len()
        || value
            .bytes()
            .any(|byte| byte.is_ascii_control() || matches!(byte, b' ' | b'\\' | b'?' | b'#'))
        || value
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
        || looks_secret_like(&value)
    {
        return Err(error);
    }
    Ok(value)
}

fn safe_ref_token(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        && !value.starts_with('-')
        && !value.ends_with('-')
        && !value.contains("..")
}

fn looks_secret_like(value: &str) -> bool {
    const SECRET_MARKERS: &[&str] = &[
        "password",
        "passwd",
        "secret",
        "credential",
        "private_key",
        "private-key",
        "api_key",
        "api-key",
        "access_key",
        "access-key",
        "secret_key",
        "secret-key",
        "session_token",
        "access_token",
        "refresh_token",
        "token=",
        "token:",
        "bearer ",
        "bearer_",
        "-----begin",
        "kubeconfig",
    ];
    let lower = value.to_ascii_lowercase();
    SECRET_MARKERS.iter().any(|marker| lower.contains(marker))
}

fn public_metadata_class(data_class: DataClass) -> Result<PrivacyDataClass, CloudComputeError> {
    if data_class != DataClass::Public {
        return Err(CloudComputeError::InvalidDataClass);
    }
    PrivacyDataClass::new(data_class).map_err(|_| CloudComputeError::InvalidDataClass)
}

fn prefixed_token(
    value: String,
    prefix: &str,
    error: CloudComputeError,
) -> Result<String, CloudComputeError> {
    if value.starts_with(prefix)
        && value.len() > prefix.len()
        && !value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        Ok(value)
    } else {
        Err(error)
    }
}

fn map_resource_error(error: CloudResourceError) -> CloudComputeError {
    match error {
        CloudResourceError::InvalidResourceId => CloudComputeError::InvalidResourceId,
        CloudResourceError::ResourceIdTenantMismatch => CloudComputeError::ResourceTenantMismatch,
        CloudResourceError::ResourceIdRegionMismatch => CloudComputeError::ResourceRegionMismatch,
        CloudResourceError::ResourceIdKindMismatch => CloudComputeError::ResourceKindMismatch,
        CloudResourceError::InvalidTenantId => CloudComputeError::InvalidTenantId,
        _ => CloudComputeError::InvalidResourceId,
    }
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

#[cfg(test)]
mod tests {
    use super::*;
    use network_residency::{
        PerPackResidency, PerPackResidencyCreate, RegulatorOverlay, RegulatorOverlayCreate,
    };

    const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    fn residency_class() -> ResidencyClass {
        ResidencyClass::PerPack(Box::new(
            PerPackResidency::new(PerPackResidencyCreate {
                allowed_primary_regions: vec!["region-alpha".to_string()],
                allowed_replica_regions: vec!["region-beta".to_string()],
                forbidden_regions: vec!["region-gamma".to_string()],
                regulator_overlay: RegulatorOverlay::new(RegulatorOverlayCreate {
                    regulator_refs: vec!["regulator/global-cloud".to_string()],
                    evidence_ref: "evidence/residency/global-cloud".to_string(),
                })
                .expect("regulator overlay fixture is valid"),
            })
            .expect("per-pack residency fixture is valid"),
        ))
    }

    fn quota() -> ComputeQuotaEnvelope {
        ComputeQuotaEnvelope {
            vcpu_limit: 128,
            memory_gb_limit: 512,
            gpu_limit: 8,
            local_ssd_gb_limit: 4_096,
            current_vcpu: 4,
            current_memory_gb: 16,
            current_gpu: 0,
            current_local_ssd_gb: 100,
        }
    }

    fn flavor() -> ComputeFlavorSpec {
        ComputeFlavorSpec {
            class: InstanceFlavor::GeneralPurpose,
            vcpu: 4,
            memory_gb: 16,
            gpu_count: 0,
            local_ssd_gb: 100,
        }
    }

    fn image() -> String {
        format!("oci://harbor.region-alpha.oya/ten_alpha/app@sha256:{DIGEST}")
    }

    fn function_bundle() -> String {
        format!("function://harbor.region-alpha.oya/ten_alpha/image-resize@sha256:{DIGEST}")
    }

    fn instance_create() -> InstanceCreate {
        InstanceCreate {
            resource_id: "oya:cloud:region-alpha:ten_alpha:instance:app-1".to_string(),
            tenant_id: "ten_alpha".to_string(),
            region: "region-alpha".to_string(),
            az: "region-alpha-a".to_string(),
            cell_id: "cell-region-alpha-a-001".to_string(),
            flavor: flavor(),
            image: image(),
            key_pair: Some("key_prod".to_string()),
            vpc_id: "oya:cloud:region-alpha:ten_alpha:vpc:prod".to_string(),
            subnet_id: "oya:cloud:region-alpha:ten_alpha:subnet:prod-a".to_string(),
            security_groups: vec!["sg_web".to_string()],
            iam_role: Some("role_app".to_string()),
            user_data_uri: Some("userdata/ten_alpha/app-1/cloud-init.yaml".to_string()),
            quota: quota(),
            residency: residency_class(),
            state: InstanceState::Pending,
            data_class: DataClass::Public,
            created_at_epoch_seconds: 1_700_100_000,
        }
    }

    fn node_pool(id: &str, az: &str, subnet: &str) -> KubernetesNodePoolCreate {
        KubernetesNodePoolCreate {
            id: id.to_string(),
            az: az.to_string(),
            cell_id: format!("cell-{az}-001"),
            subnet_id: subnet.to_string(),
            security_groups: vec!["sg_web".to_string()],
            flavor: flavor(),
            min_nodes: 1,
            max_nodes: 5,
            autoscaling_enabled: true,
        }
    }

    fn k8s_create() -> KubernetesClusterCreate {
        KubernetesClusterCreate {
            resource_id: "oya:cloud:region-alpha:ten_alpha:k8s:prod".to_string(),
            tenant_id: "ten_alpha".to_string(),
            region: "region-alpha".to_string(),
            flavor: K8sFlavor::HighAvailability,
            control_plane_version: "v1.30.2-oya.1".to_string(),
            control_plane_private: true,
            node_pools: vec![
                node_pool(
                    "np_a",
                    "region-alpha-a",
                    "oya:cloud:region-alpha:ten_alpha:subnet:prod-a",
                ),
                node_pool(
                    "np_b",
                    "region-alpha-b",
                    "oya:cloud:region-alpha:ten_alpha:subnet:prod-b",
                ),
                node_pool(
                    "np_c",
                    "region-alpha-c",
                    "oya:cloud:region-alpha:ten_alpha:subnet:prod-c",
                ),
            ],
            quota: quota(),
            residency: residency_class(),
            state: KubernetesClusterState::Creating,
            data_class: DataClass::Public,
            created_at_epoch_seconds: 1_700_100_010,
        }
    }

    fn function_create() -> FunctionDeploymentCreate {
        FunctionDeploymentCreate {
            resource_id: "oya:cloud:region-alpha:ten_alpha:function:image-resize".to_string(),
            tenant_id: "ten_alpha".to_string(),
            region: "region-alpha".to_string(),
            az: "region-alpha-a".to_string(),
            cell_id: "cell-region-alpha-a-001".to_string(),
            runtime: FunctionRuntime::Wasm,
            name: "image-resize".to_string(),
            bundle: function_bundle(),
            cold_start_budget_ms: 750,
            timeout_ms: 30_000,
            memory_mb: 512,
            max_concurrency: 250,
            allowed_data_classes: vec![DataClass::Public, DataClass::PiiIdentifying],
            residency: residency_class(),
            state: FunctionDeploymentState::Deploying,
            data_class: DataClass::Public,
            created_at_epoch_seconds: 1_700_100_020,
        }
    }

    fn invocation(id: &str, data_class: DataClass) -> FunctionInvocationRequest {
        FunctionInvocationRequest {
            invocation_id: id.to_string(),
            tenant_id: "ten_alpha".to_string(),
            function_id: "oya:cloud:region-alpha:ten_alpha:function:image-resize".to_string(),
            region: "region-alpha".to_string(),
            payload_data_class: data_class,
            idempotency_key: format!("idem-{id}-0123456789"),
            current_concurrent_invocations: 0,
            requested_at_epoch_seconds: 1_700_100_030,
        }
    }

    fn provider_vm_request() -> ComputeProviderVmCreateRequest {
        let instance = Instance::new(instance_create()).expect("instance contract is valid");
        ComputeProviderVmCreateRequest {
            request_id: "compute-vm-provider-001".to_string(),
            provider_instance_ref: "provider://cell-region-alpha-a-001/app-1".to_string(),
            tenant_id: instance.tenant_id.value.clone(),
            actor: "sp_cloud_provisioner".to_string(),
            idempotency_key: "idem-compute-vm-provider-001".to_string(),
            requested_at_epoch_seconds: 1_700_100_050,
            instance,
        }
    }

    #[test]
    fn creates_vm_instance_with_cell_network_iam_quota_and_digest_image() {
        let instance = Instance::new(instance_create()).expect("instance contract is valid");

        assert_eq!(instance.resource_id.value.kind_label().unwrap(), "instance");
        assert_eq!(instance.az.value.value, "region-alpha-a");
        assert_eq!(instance.cell_id.value.value, "cell-region-alpha-a-001");
        assert_eq!(instance.flavor.value.vcpu, 4);
        assert_eq!(instance.image.value.kind, ImageRefKind::Oci);
        assert_eq!(instance.security_groups.value.len(), 1);
        assert!(instance.iam_role.value.is_some());
        assert_eq!(instance.schema_version.value, COMPUTE_SCHEMA_VERSION);
    }

    #[test]
    fn provider_vm_receipt_requires_non_empty_provider_evidence() {
        let request = provider_vm_request();

        let receipt = ComputeProviderVmReceipt::from_request(
            ComputeProviderKind::AwsEc2,
            request.clone(),
            "aws-req-001",
            "aws-ec2://evidence/req-001",
        )
        .expect("provider receipt keeps neutral VM identity");
        assert_eq!(receipt.provider_kind, ComputeProviderKind::AwsEc2);
        assert_eq!(receipt.tenant_id, "ten_alpha");
        assert_eq!(receipt.region, "region-alpha");
        assert_eq!(receipt.az, "region-alpha-a");

        let mut invalid_idempotency_request = request.clone();
        invalid_idempotency_request.idempotency_key = "short".to_string();
        let invalid_idempotency = ComputeProviderVmReceipt::from_request(
            ComputeProviderKind::AwsEc2,
            invalid_idempotency_request,
            "aws-req-001",
            "aws-ec2://evidence/req-001",
        )
        .expect_err("provider VM idempotency key is bounded before adapter projection");
        assert_eq!(invalid_idempotency, ComputeProviderVmError::InvalidRequest);
        let missing_request_id = ComputeProviderVmReceipt::from_request(
            ComputeProviderKind::AwsEc2,
            request.clone(),
            " ",
            "aws-ec2://evidence/req-001",
        )
        .expect_err("provider request id is required");
        assert_eq!(missing_request_id, ComputeProviderVmError::InvalidRequest);

        let missing_evidence_ref = ComputeProviderVmReceipt::from_request(
            ComputeProviderKind::AwsEc2,
            request,
            "aws-req-001",
            "",
        )
        .expect_err("provider evidence ref is required");
        assert_eq!(missing_evidence_ref, ComputeProviderVmError::InvalidRequest);
    }

    #[test]
    fn rejects_vm_identity_location_quota_image_and_forged_state() {
        let state_error = Instance::new(InstanceCreate {
            state: InstanceState::Running,
            ..instance_create()
        })
        .expect_err("create callers cannot forge runtime state");
        assert_eq!(state_error, CloudComputeError::InvalidInstanceState);

        let quota_error = Instance::new(InstanceCreate {
            quota: ComputeQuotaEnvelope {
                vcpu_limit: 6,
                ..quota()
            },
            ..instance_create()
        })
        .expect_err("cell quota is checked before scheduling");
        assert_eq!(quota_error, CloudComputeError::QuotaExceeded);

        let image_error = Instance::new(InstanceCreate {
            image: "oci://harbor.region-alpha.oya/ten_alpha/app:latest".to_string(),
            ..instance_create()
        })
        .expect_err("image refs must be digest pinned");
        assert_eq!(image_error, CloudComputeError::InvalidImageRef);

        let cell_error = Instance::new(InstanceCreate {
            cell_id: "cell-region-alpha-b-001".to_string(),
            ..instance_create()
        })
        .expect_err("cell id must stay inside selected AZ");
        assert_eq!(cell_error, CloudComputeError::CellAzMismatch);
    }

    #[test]
    fn creates_ha_kubernetes_cluster_across_three_azs_with_quota() {
        let cluster = KubernetesCluster::new(k8s_create()).expect("cluster contract is valid");

        assert_eq!(cluster.resource_id.value.kind_label().unwrap(), "k8s");
        assert_eq!(cluster.node_pools.value.len(), 3);
        assert_eq!(cluster.control_plane_version.value.value, "v1.30.2-oya.1");
        assert!(cluster.control_plane_private.value);
        assert_eq!(cluster.schema_version.value, COMPUTE_SCHEMA_VERSION);
    }

    #[test]
    fn rejects_kubernetes_without_ha_spread_or_autoscale_max_node_quota() {
        let ha_error = KubernetesCluster::new(KubernetesClusterCreate {
            node_pools: vec![node_pool(
                "np_a",
                "region-alpha-a",
                "oya:cloud:region-alpha:ten_alpha:subnet:prod-a",
            )],
            ..k8s_create()
        })
        .expect_err("HA managed control plane needs three AZs");
        assert_eq!(ha_error, CloudComputeError::KubernetesHaRequiresThreeAzs);

        let quota_error = KubernetesCluster::new(KubernetesClusterCreate {
            quota: ComputeQuotaEnvelope {
                vcpu_limit: 8,
                ..quota()
            },
            ..k8s_create()
        })
        .expect_err("node pool minimum capacity must fit quota");
        assert_eq!(quota_error, CloudComputeError::QuotaExceeded);
        let max_quota_error = KubernetesCluster::new(KubernetesClusterCreate {
            quota: ComputeQuotaEnvelope {
                vcpu_limit: 32,
                ..quota()
            },
            ..k8s_create()
        })
        .expect_err("autoscale maximum capacity must fit quota even when min nodes fit");
        assert_eq!(max_quota_error, CloudComputeError::QuotaExceeded);
    }

    #[test]
    fn registers_function_then_invokes_active_function_with_data_class_allowlist() {
        let mut catalog = CloudComputeCatalog::default();
        let function = catalog
            .register_function(function_create())
            .expect("function registers");
        let active = catalog
            .activate_function(&function.resource_id.value)
            .expect("function activates");
        assert_eq!(active.state.value, FunctionDeploymentState::Active);

        let receipt = catalog
            .invoke_function(invocation("fninv_001", DataClass::PiiIdentifying))
            .expect("allowed function invocation is recorded");
        assert_eq!(
            receipt.payload_data_class.value.data_class(),
            DataClass::PiiIdentifying
        );
        assert_eq!(receipt.cold_start_budget_ms.value, 750);
        assert_eq!(catalog.invocations().count(), 1);
    }

    #[test]
    fn function_activation_failure_preserves_registered_function() {
        let mut catalog = CloudComputeCatalog::default();
        let function = catalog
            .register_function(function_create())
            .expect("function registers");
        catalog
            .activate_function(&function.resource_id.value)
            .expect("first activation succeeds");

        let second_activation = catalog
            .activate_function(&function.resource_id.value)
            .expect_err("active functions cannot be activated twice");

        assert_eq!(second_activation, CloudComputeError::InvalidFunctionState);
        assert_eq!(catalog.functions().count(), 1);
        assert_eq!(
            catalog
                .functions()
                .next()
                .expect("function remains")
                .state
                .value,
            FunctionDeploymentState::Active
        );
    }

    #[test]
    fn rejects_function_budget_payload_class_max_concurrency_duplicate_invocation_and_inactive() {
        let budget_error = FunctionDeployment::new(FunctionDeploymentCreate {
            cold_start_budget_ms: MAX_FUNCTION_COLD_START_BUDGET_MS + 1,
            ..function_create()
        })
        .expect_err("function cold-start budgets are capped");
        assert_eq!(budget_error, CloudComputeError::InvalidFunctionBudget);

        let inactive = FunctionDeployment::new(function_create()).expect("deploying function");
        let inactive_error = inactive
            .invoke(invocation("fninv_inactive", DataClass::Public))
            .expect_err("deploying functions cannot be invoked");
        assert_eq!(inactive_error, CloudComputeError::FunctionNotActive);

        let mut catalog = CloudComputeCatalog::default();
        let function = catalog
            .register_function(function_create())
            .expect("function registers");
        catalog
            .activate_function(&function.resource_id.value)
            .expect("function activates");
        let class_error = catalog
            .invoke_function(invocation("fninv_002", DataClass::Phi))
            .expect_err("payload data class must be allowlisted");
        assert_eq!(class_error, CloudComputeError::PayloadDataClassNotAllowed);

        let concurrency_error = catalog
            .invoke_function(FunctionInvocationRequest {
                current_concurrent_invocations: 250,
                ..invocation("fninv_concurrency", DataClass::Public)
            })
            .expect_err("function invocation cannot exceed declared max concurrency");
        assert_eq!(concurrency_error, CloudComputeError::QuotaExceeded);

        catalog
            .invoke_function(invocation("fninv_003", DataClass::Public))
            .expect("first invocation records");
        let duplicate = catalog
            .invoke_function(invocation("fninv_003", DataClass::Public))
            .expect_err("invocation ids are immutable evidence keys");
        assert_eq!(duplicate, CloudComputeError::DuplicateInvocation);
    }

    #[test]
    fn function_invocation_store_enforces_bounded_retention() {
        let mut catalog = CloudComputeCatalog::with_invocation_retention_limit(1);
        let function = catalog
            .register_function(function_create())
            .expect("function registers");
        catalog
            .activate_function(&function.resource_id.value)
            .expect("function activates");

        catalog
            .invoke_function(invocation("fninv_bound_1", DataClass::Public))
            .expect("first invocation records");
        catalog
            .invoke_function(invocation("fninv_bound_2", DataClass::Public))
            .expect("second invocation records");

        assert_eq!(catalog.invocations().count(), 1);
    }

    #[test]
    fn rejects_operational_labels_on_compute_metadata_and_function_payloads() {
        let instance_class_error = Instance::new(InstanceCreate {
            data_class: DataClass::Audit,
            ..instance_create()
        })
        .expect_err("compute metadata is public privacy metadata");
        assert_eq!(instance_class_error, CloudComputeError::InvalidDataClass);

        let function_class_error = FunctionDeployment::new(FunctionDeploymentCreate {
            allowed_data_classes: vec![DataClass::Audit],
            ..function_create()
        })
        .expect_err("function payload allowlists use privacy classes only");
        assert_eq!(function_class_error, CloudComputeError::InvalidDataClass);
    }

    #[test]
    fn catalog_rejects_duplicate_compute_resources() {
        let mut catalog = CloudComputeCatalog::default();
        catalog
            .create_instance(instance_create())
            .expect("first instance");
        let duplicate_instance = catalog
            .create_instance(instance_create())
            .expect_err("duplicate instance id rejected");
        assert_eq!(duplicate_instance, CloudComputeError::DuplicateInstance);

        catalog
            .create_kubernetes_cluster(k8s_create())
            .expect("first cluster");
        let duplicate_cluster = catalog
            .create_kubernetes_cluster(k8s_create())
            .expect_err("duplicate cluster id rejected");
        assert_eq!(
            duplicate_cluster,
            CloudComputeError::DuplicateKubernetesCluster
        );

        catalog
            .register_function(function_create())
            .expect("first function");
        let duplicate_function = catalog
            .register_function(function_create())
            .expect_err("duplicate function id rejected");
        assert_eq!(duplicate_function, CloudComputeError::DuplicateFunction);
    }
}
