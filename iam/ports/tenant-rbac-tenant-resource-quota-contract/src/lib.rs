//! FD-001 tenant ResourceQuota and LimitRange contract for later Oyatie Cloud dogfooding.
//!
//! This review-only crate defines namespace resource-consumption guardrails that
//! must exist before FD-001 Tenant RBAC, HR, Payroll, and Accounting
//! workloads can be promoted as production tenant workloads on the future Oyatie
//! Cloud substrate. It binds the FD-001 tenant-workload manifest and tenant
//! admission policy to Kubernetes ResourceQuota, LimitRange, container request
//! and limit requirements, object-count quotas, storage quotas, admission plugin
//! evidence, tenant labels, and quota usage audit evidence. It does not attach a
//! Kubernetes cluster, apply ResourceQuota or LimitRange objects, attach quota
//! admission runtime, observe runtime quota usage, deploy workloads, attach a
//! cloud substrate runtime, or emit runtime audit-chain events.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use iam_tenant_rbac_tenant_admission_policy::{
    Fd001TenantAdmissionPolicyError, fd001_tenant_admission_policy_contract,
    validate_fd001_tenant_admission_policy_contract,
};
use iam_tenant_rbac_tenant_workload_manifest::{
    Fd001TenantWorkloadKind, Fd001TenantWorkloadManifestError, fd001_tenant_workload_manifest,
    validate_fd001_tenant_workload_manifest,
};

const SCHEMA_VERSION: u32 = 1;
const MIN_REQUIREMENT_COUNT: usize = 13;
const CONTRACT_NAME: &str = "fd001-tenant-rbac-tenant-resource-quota-contract";
const PROGRAM_NAME: &str = "fd-001-tenant-rbac-generic";
const SUBSTRATE_NAME: &str = "oyatie-cloud";
const TENANT_NAMESPACE: &str = "oyatie-fd001-tenant-rbac-dev";
const SOURCE_MANIFEST_REF: &str =
    "crates/tenant-rbac-tenant-workload-manifest/src/lib.rs::fd001_tenant_workload_manifest";
const SOURCE_ADMISSION_POLICY_REF: &str =
    "crates/tenant-rbac-tenant-admission-policy/src/lib.rs::fd001_tenant_admission_policy_contract";
const POLICY_REF_PREFIX: &str = "policy/resource-quota/fd001/";
const EXPECTED_EVIDENCE_REF: &str =
    "evidence/resource-quota/fd001-tenant-rbac/resource-quota-contract-review.jsonl";

const RESOURCE_QUOTA_DOC_URL: &str = "https://kubernetes.io/docs/concepts/policy/resource-quotas/";
const LIMIT_RANGE_DOC_URL: &str = "https://kubernetes.io/docs/concepts/policy/limit-range/";
const RESOURCE_MANAGEMENT_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/";
const ADMISSION_CONTROLLERS_DOC_URL: &str =
    "https://kubernetes.io/docs/reference/access-authn-authz/admission-controllers/";
const NAMESPACES_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/overview/working-with-objects/namespaces/";
const MULTI_TENANCY_DOC_URL: &str = "https://kubernetes.io/docs/concepts/security/multi-tenancy/";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Fd001TenantResourceQuotaRequirementKind {
    NamespaceResourceQuotaRequired,
    ComputeRequestsQuotaRequired,
    ComputeLimitsQuotaRequired,
    ObjectCountQuotaRequired,
    PersistentStorageQuotaRequired,
    LimitRangeDefaultsRequired,
    LimitRangeMinMaxRequired,
    ContainerRequestsLimitsRequired,
    ResourceQuotaAdmissionEvidenceRequired,
    LimitRangerAdmissionEvidenceRequired,
    TenantLabelSelectorRequired,
    QuotaUsageAuditEvidenceRequired,
    AdmissionPolicyEvidenceRequired,
}

impl Fd001TenantResourceQuotaRequirementKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NamespaceResourceQuotaRequired => "namespace_resource_quota_required",
            Self::ComputeRequestsQuotaRequired => "compute_requests_quota_required",
            Self::ComputeLimitsQuotaRequired => "compute_limits_quota_required",
            Self::ObjectCountQuotaRequired => "object_count_quota_required",
            Self::PersistentStorageQuotaRequired => "persistent_storage_quota_required",
            Self::LimitRangeDefaultsRequired => "limit_range_defaults_required",
            Self::LimitRangeMinMaxRequired => "limit_range_min_max_required",
            Self::ContainerRequestsLimitsRequired => "container_requests_limits_required",
            Self::ResourceQuotaAdmissionEvidenceRequired => {
                "resource_quota_admission_evidence_required"
            }
            Self::LimitRangerAdmissionEvidenceRequired => {
                "limit_ranger_admission_evidence_required"
            }
            Self::TenantLabelSelectorRequired => "tenant_label_selector_required",
            Self::QuotaUsageAuditEvidenceRequired => "quota_usage_audit_evidence_required",
            Self::AdmissionPolicyEvidenceRequired => "admission_policy_evidence_required",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fd001TenantResourceQuotaRequirement {
    pub requirement_id: &'static str,           // data_class: PUBLIC
    pub workload_kind: Fd001TenantWorkloadKind, // data_class: PUBLIC
    pub requirement_kind: Fd001TenantResourceQuotaRequirementKind, // data_class: PUBLIC
    pub kubernetes_resource_kind: &'static str, // data_class: PUBLIC
    pub policy_ref: &'static str,               // data_class: INTERNAL_ONLY
    pub expected_evidence_ref: &'static str,    // data_class: INTERNAL_ONLY
    pub official_doc_url: &'static str,         // data_class: PUBLIC
    pub source_manifest_ref: &'static str,      // data_class: INTERNAL_ONLY
    pub source_admission_policy_ref: &'static str, // data_class: INTERNAL_ONLY
    pub applies_to_all_manifest_workloads: bool, // data_class: PUBLIC
    pub runtime_enforcement_attached: bool,     // data_class: INTERNAL_ONLY
    pub schema_version: u32,                    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fd001TenantResourceQuotaContract {
    pub contract_name: &'static str,          // data_class: PUBLIC
    pub program_name: &'static str,           // data_class: PUBLIC
    pub substrate_name: &'static str,         // data_class: PUBLIC
    pub tenant_namespace: &'static str,       // data_class: INTERNAL_ONLY
    pub workload_manifest_name: &'static str, // data_class: PUBLIC
    pub workload_manifest_count: usize,       // data_class: PUBLIC
    pub tenant_admission_policy_contract_name: &'static str, // data_class: PUBLIC
    pub requirements: Vec<Fd001TenantResourceQuotaRequirement>, // data_class: INTERNAL_ONLY
    pub official_docs_required: bool,         // data_class: PUBLIC
    pub all_manifest_workloads_in_scope: bool, // data_class: PUBLIC
    pub namespace_resource_quota_required: bool, // data_class: PUBLIC
    pub compute_requests_quota_required: bool, // data_class: PUBLIC
    pub compute_limits_quota_required: bool,  // data_class: PUBLIC
    pub object_count_quota_required: bool,    // data_class: PUBLIC
    pub persistent_storage_quota_required: bool, // data_class: PUBLIC
    pub limit_range_defaults_required: bool,  // data_class: PUBLIC
    pub limit_range_min_max_required: bool,   // data_class: PUBLIC
    pub container_requests_limits_required: bool, // data_class: PUBLIC
    pub resource_quota_admission_evidence_required: bool, // data_class: PUBLIC
    pub limit_ranger_admission_evidence_required: bool, // data_class: PUBLIC
    pub tenant_label_selector_required: bool, // data_class: PUBLIC
    pub quota_usage_audit_evidence_required: bool, // data_class: PUBLIC
    pub admission_policy_evidence_required: bool, // data_class: PUBLIC
    pub review_only_contract: bool,           // data_class: PUBLIC
    pub kubernetes_cluster_attached: bool,    // data_class: INTERNAL_ONLY
    pub resource_quota_applied: bool,         // data_class: INTERNAL_ONLY
    pub limit_range_applied: bool,            // data_class: INTERNAL_ONLY
    pub quota_admission_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub limit_ranger_runtime_attached: bool,  // data_class: INTERNAL_ONLY
    pub quota_usage_runtime_observed: bool,   // data_class: INTERNAL_ONLY
    pub workload_runtime_deployed: bool,      // data_class: INTERNAL_ONLY
    pub cloud_substrate_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Fd001TenantResourceQuotaError {
    WorkloadManifest(Fd001TenantWorkloadManifestError),
    TenantAdmissionPolicy(Fd001TenantAdmissionPolicyError),
    InvalidContractName,
    InvalidProgramName,
    InvalidSubstrateName,
    InvalidTenantNamespace,
    InvalidManifestName,
    InvalidWorkloadCount,
    InvalidAdmissionPolicyContractName,
    MissingRequirements,
    DuplicateRequirement(String),
    MissingWorkloadKind(Fd001TenantWorkloadKind),
    MissingRequirementKind(Fd001TenantResourceQuotaRequirementKind),
    InvalidRequirementId,
    InvalidKubernetesResourceKind,
    InvalidPolicyRef,
    InvalidExpectedEvidenceRef,
    InvalidOfficialDocUrl,
    InvalidSourceManifestRef,
    InvalidSourceAdmissionPolicyRef,
    MissingRequiredControl(&'static str),
    RuntimeAttachmentOverclaim,
}

pub fn fd001_tenant_resource_quota_contract()
-> Result<Fd001TenantResourceQuotaContract, Fd001TenantResourceQuotaError> {
    let manifest = fd001_tenant_workload_manifest();
    validate_fd001_tenant_workload_manifest(&manifest)
        .map_err(Fd001TenantResourceQuotaError::WorkloadManifest)?;
    let admission_policy = fd001_tenant_admission_policy_contract()
        .map_err(Fd001TenantResourceQuotaError::TenantAdmissionPolicy)?;
    validate_fd001_tenant_admission_policy_contract(&admission_policy)
        .map_err(Fd001TenantResourceQuotaError::TenantAdmissionPolicy)?;

    Ok(Fd001TenantResourceQuotaContract {
        contract_name: CONTRACT_NAME,
        program_name: PROGRAM_NAME,
        substrate_name: SUBSTRATE_NAME,
        tenant_namespace: manifest.tenant_namespace,
        workload_manifest_name: manifest.manifest_name,
        workload_manifest_count: manifest.workloads.len(),
        tenant_admission_policy_contract_name: admission_policy.contract_name,
        requirements: quota_requirements(),
        official_docs_required: true,
        all_manifest_workloads_in_scope: true,
        namespace_resource_quota_required: true,
        compute_requests_quota_required: true,
        compute_limits_quota_required: true,
        object_count_quota_required: true,
        persistent_storage_quota_required: true,
        limit_range_defaults_required: true,
        limit_range_min_max_required: true,
        container_requests_limits_required: true,
        resource_quota_admission_evidence_required: true,
        limit_ranger_admission_evidence_required: true,
        tenant_label_selector_required: true,
        quota_usage_audit_evidence_required: true,
        admission_policy_evidence_required: true,
        review_only_contract: true,
        kubernetes_cluster_attached: false,
        resource_quota_applied: false,
        limit_range_applied: false,
        quota_admission_runtime_attached: false,
        limit_ranger_runtime_attached: false,
        quota_usage_runtime_observed: false,
        workload_runtime_deployed: false,
        cloud_substrate_runtime_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: SCHEMA_VERSION,
    })
}

pub fn validate_fd001_tenant_resource_quota_contract(
    contract: &Fd001TenantResourceQuotaContract,
) -> Result<(), Fd001TenantResourceQuotaError> {
    let manifest = fd001_tenant_workload_manifest();
    validate_fd001_tenant_workload_manifest(&manifest)
        .map_err(Fd001TenantResourceQuotaError::WorkloadManifest)?;
    let admission_policy = fd001_tenant_admission_policy_contract()
        .map_err(Fd001TenantResourceQuotaError::TenantAdmissionPolicy)?;
    validate_fd001_tenant_admission_policy_contract(&admission_policy)
        .map_err(Fd001TenantResourceQuotaError::TenantAdmissionPolicy)?;

    validate_slug(
        contract.contract_name,
        Fd001TenantResourceQuotaError::InvalidContractName,
    )?;
    if contract.program_name != PROGRAM_NAME {
        return Err(Fd001TenantResourceQuotaError::InvalidProgramName);
    }
    if contract.substrate_name != SUBSTRATE_NAME {
        return Err(Fd001TenantResourceQuotaError::InvalidSubstrateName);
    }
    validate_tenant_namespace(contract.tenant_namespace)?;
    if contract.workload_manifest_name != manifest.manifest_name {
        return Err(Fd001TenantResourceQuotaError::InvalidManifestName);
    }
    if contract.workload_manifest_count != manifest.workloads.len() {
        return Err(Fd001TenantResourceQuotaError::InvalidWorkloadCount);
    }
    if contract.tenant_admission_policy_contract_name != admission_policy.contract_name {
        return Err(Fd001TenantResourceQuotaError::InvalidAdmissionPolicyContractName);
    }
    if contract.requirements.len() < MIN_REQUIREMENT_COUNT
        || contract.schema_version != SCHEMA_VERSION
    {
        return Err(Fd001TenantResourceQuotaError::MissingRequirements);
    }
    validate_required_controls(contract)?;
    validate_nonclaims(contract)?;
    validate_requirements(contract)?;
    Ok(())
}

pub fn fd001_tenant_resource_quota_doc_urls(
    contract: &Fd001TenantResourceQuotaContract,
) -> Vec<&'static str> {
    let mut docs = BTreeSet::new();
    for requirement in &contract.requirements {
        docs.insert(requirement.official_doc_url);
    }
    docs.into_iter().collect()
}

fn quota_requirements() -> Vec<Fd001TenantResourceQuotaRequirement> {
    vec![
        requirement(
            "namespace-resource-quota-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantResourceQuotaRequirementKind::NamespaceResourceQuotaRequired,
            "ResourceQuota",
            "namespace-resource-quota",
            RESOURCE_QUOTA_DOC_URL,
        ),
        requirement(
            "compute-requests-quota-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantResourceQuotaRequirementKind::ComputeRequestsQuotaRequired,
            "ResourceQuota",
            "compute-requests-quota",
            RESOURCE_QUOTA_DOC_URL,
        ),
        requirement(
            "compute-limits-quota-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantResourceQuotaRequirementKind::ComputeLimitsQuotaRequired,
            "ResourceQuota",
            "compute-limits-quota",
            RESOURCE_QUOTA_DOC_URL,
        ),
        requirement(
            "object-count-quota-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantResourceQuotaRequirementKind::ObjectCountQuotaRequired,
            "ResourceQuota",
            "object-count-quota",
            RESOURCE_QUOTA_DOC_URL,
        ),
        requirement(
            "persistent-storage-quota-all-workloads",
            Fd001TenantWorkloadKind::AccountingJournal,
            Fd001TenantResourceQuotaRequirementKind::PersistentStorageQuotaRequired,
            "ResourceQuota",
            "persistent-storage-quota",
            RESOURCE_QUOTA_DOC_URL,
        ),
        requirement(
            "limit-range-defaults-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantResourceQuotaRequirementKind::LimitRangeDefaultsRequired,
            "LimitRange",
            "limit-range-defaults",
            LIMIT_RANGE_DOC_URL,
        ),
        requirement(
            "limit-range-min-max-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantResourceQuotaRequirementKind::LimitRangeMinMaxRequired,
            "LimitRange",
            "limit-range-min-max",
            LIMIT_RANGE_DOC_URL,
        ),
        requirement(
            "container-requests-limits-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantResourceQuotaRequirementKind::ContainerRequestsLimitsRequired,
            "PodSpecResources",
            "container-requests-limits",
            RESOURCE_MANAGEMENT_DOC_URL,
        ),
        requirement(
            "resource-quota-admission-evidence-all-workloads",
            Fd001TenantWorkloadKind::AccountingJournal,
            Fd001TenantResourceQuotaRequirementKind::ResourceQuotaAdmissionEvidenceRequired,
            "AdmissionController",
            "resource-quota-admission-evidence",
            ADMISSION_CONTROLLERS_DOC_URL,
        ),
        requirement(
            "limit-ranger-admission-evidence-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantResourceQuotaRequirementKind::LimitRangerAdmissionEvidenceRequired,
            "AdmissionController",
            "limit-ranger-admission-evidence",
            ADMISSION_CONTROLLERS_DOC_URL,
        ),
        requirement(
            "tenant-label-selector-quota-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantResourceQuotaRequirementKind::TenantLabelSelectorRequired,
            "NamespaceLabel",
            "tenant-label-selector-quota",
            MULTI_TENANCY_DOC_URL,
        ),
        requirement(
            "quota-usage-audit-evidence-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantResourceQuotaRequirementKind::QuotaUsageAuditEvidenceRequired,
            "ResourceQuotaStatus",
            "quota-usage-audit-evidence",
            NAMESPACES_DOC_URL,
        ),
        requirement(
            "admission-policy-quota-evidence-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantResourceQuotaRequirementKind::AdmissionPolicyEvidenceRequired,
            "ValidatingAdmissionPolicy",
            "admission-policy-quota-evidence",
            ADMISSION_CONTROLLERS_DOC_URL,
        ),
    ]
}

fn requirement(
    requirement_id: &'static str,
    workload_kind: Fd001TenantWorkloadKind,
    requirement_kind: Fd001TenantResourceQuotaRequirementKind,
    kubernetes_resource_kind: &'static str,
    policy_suffix: &'static str,
    official_doc_url: &'static str,
) -> Fd001TenantResourceQuotaRequirement {
    Fd001TenantResourceQuotaRequirement {
        requirement_id,
        workload_kind,
        requirement_kind,
        kubernetes_resource_kind,
        policy_ref: policy_ref(policy_suffix),
        expected_evidence_ref: EXPECTED_EVIDENCE_REF,
        official_doc_url,
        source_manifest_ref: SOURCE_MANIFEST_REF,
        source_admission_policy_ref: SOURCE_ADMISSION_POLICY_REF,
        applies_to_all_manifest_workloads: true,
        runtime_enforcement_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

fn policy_ref(policy_suffix: &'static str) -> &'static str {
    match policy_suffix {
        "namespace-resource-quota" => "policy/resource-quota/fd001/namespace-resource-quota",
        "compute-requests-quota" => "policy/resource-quota/fd001/compute-requests-quota",
        "compute-limits-quota" => "policy/resource-quota/fd001/compute-limits-quota",
        "object-count-quota" => "policy/resource-quota/fd001/object-count-quota",
        "persistent-storage-quota" => "policy/resource-quota/fd001/persistent-storage-quota",
        "limit-range-defaults" => "policy/resource-quota/fd001/limit-range-defaults",
        "limit-range-min-max" => "policy/resource-quota/fd001/limit-range-min-max",
        "container-requests-limits" => "policy/resource-quota/fd001/container-requests-limits",
        "resource-quota-admission-evidence" => {
            "policy/resource-quota/fd001/resource-quota-admission-evidence"
        }
        "limit-ranger-admission-evidence" => {
            "policy/resource-quota/fd001/limit-ranger-admission-evidence"
        }
        "tenant-label-selector-quota" => "policy/resource-quota/fd001/tenant-label-selector-quota",
        "quota-usage-audit-evidence" => "policy/resource-quota/fd001/quota-usage-audit-evidence",
        "admission-policy-quota-evidence" => {
            "policy/resource-quota/fd001/admission-policy-quota-evidence"
        }
        _ => "policy/resource-quota/fd001/invalid",
    }
}

fn validate_required_controls(
    contract: &Fd001TenantResourceQuotaContract,
) -> Result<(), Fd001TenantResourceQuotaError> {
    for (enabled, name) in [
        (contract.official_docs_required, "official_docs_required"),
        (
            contract.all_manifest_workloads_in_scope,
            "all_manifest_workloads_in_scope",
        ),
        (
            contract.namespace_resource_quota_required,
            "namespace_resource_quota_required",
        ),
        (
            contract.compute_requests_quota_required,
            "compute_requests_quota_required",
        ),
        (
            contract.compute_limits_quota_required,
            "compute_limits_quota_required",
        ),
        (
            contract.object_count_quota_required,
            "object_count_quota_required",
        ),
        (
            contract.persistent_storage_quota_required,
            "persistent_storage_quota_required",
        ),
        (
            contract.limit_range_defaults_required,
            "limit_range_defaults_required",
        ),
        (
            contract.limit_range_min_max_required,
            "limit_range_min_max_required",
        ),
        (
            contract.container_requests_limits_required,
            "container_requests_limits_required",
        ),
        (
            contract.resource_quota_admission_evidence_required,
            "resource_quota_admission_evidence_required",
        ),
        (
            contract.limit_ranger_admission_evidence_required,
            "limit_ranger_admission_evidence_required",
        ),
        (
            contract.tenant_label_selector_required,
            "tenant_label_selector_required",
        ),
        (
            contract.quota_usage_audit_evidence_required,
            "quota_usage_audit_evidence_required",
        ),
        (
            contract.admission_policy_evidence_required,
            "admission_policy_evidence_required",
        ),
        (contract.review_only_contract, "review_only_contract"),
    ] {
        require_control(enabled, name)?;
    }
    Ok(())
}

fn validate_nonclaims(
    contract: &Fd001TenantResourceQuotaContract,
) -> Result<(), Fd001TenantResourceQuotaError> {
    if contract.kubernetes_cluster_attached
        || contract.resource_quota_applied
        || contract.limit_range_applied
        || contract.quota_admission_runtime_attached
        || contract.limit_ranger_runtime_attached
        || contract.quota_usage_runtime_observed
        || contract.workload_runtime_deployed
        || contract.cloud_substrate_runtime_attached
        || contract.runtime_audit_chain_emission_attached
    {
        return Err(Fd001TenantResourceQuotaError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_requirements(
    contract: &Fd001TenantResourceQuotaContract,
) -> Result<(), Fd001TenantResourceQuotaError> {
    let mut seen_requirements = BTreeSet::new();
    let mut seen_workloads = BTreeSet::new();
    let mut seen_requirement_kinds = BTreeSet::new();
    for requirement in &contract.requirements {
        validate_requirement(requirement)?;
        if !seen_requirements.insert(requirement.requirement_id) {
            return Err(Fd001TenantResourceQuotaError::DuplicateRequirement(
                requirement.requirement_id.to_owned(),
            ));
        }
        seen_workloads.insert(requirement.workload_kind);
        seen_requirement_kinds.insert(requirement.requirement_kind);
    }
    for workload_kind in required_workload_kinds() {
        if !seen_workloads.contains(&workload_kind) {
            return Err(Fd001TenantResourceQuotaError::MissingWorkloadKind(
                workload_kind,
            ));
        }
    }
    for requirement_kind in required_requirement_kinds() {
        if !seen_requirement_kinds.contains(&requirement_kind) {
            return Err(Fd001TenantResourceQuotaError::MissingRequirementKind(
                requirement_kind,
            ));
        }
    }
    Ok(())
}

fn validate_requirement(
    requirement: &Fd001TenantResourceQuotaRequirement,
) -> Result<(), Fd001TenantResourceQuotaError> {
    validate_slug(
        requirement.requirement_id,
        Fd001TenantResourceQuotaError::InvalidRequirementId,
    )?;
    validate_kubernetes_resource_kind(requirement.kubernetes_resource_kind)?;
    validate_prefixed_ref(
        requirement.policy_ref,
        POLICY_REF_PREFIX,
        Fd001TenantResourceQuotaError::InvalidPolicyRef,
    )?;
    validate_prefixed_ref(
        requirement.expected_evidence_ref,
        "evidence/resource-quota/fd001-tenant-rbac/",
        Fd001TenantResourceQuotaError::InvalidExpectedEvidenceRef,
    )?;
    validate_doc_url(requirement.official_doc_url)?;
    validate_prefixed_ref(
        requirement.source_manifest_ref,
        "crates/tenant-rbac-tenant-workload-manifest/",
        Fd001TenantResourceQuotaError::InvalidSourceManifestRef,
    )?;
    validate_prefixed_ref(
        requirement.source_admission_policy_ref,
        "crates/tenant-rbac-tenant-admission-policy/",
        Fd001TenantResourceQuotaError::InvalidSourceAdmissionPolicyRef,
    )?;
    if !requirement.applies_to_all_manifest_workloads {
        return Err(Fd001TenantResourceQuotaError::MissingRequiredControl(
            "requirement_applies_to_all_manifest_workloads",
        ));
    }
    if requirement.runtime_enforcement_attached || requirement.schema_version != SCHEMA_VERSION {
        return Err(Fd001TenantResourceQuotaError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn required_workload_kinds() -> [Fd001TenantWorkloadKind; 4] {
    [
        Fd001TenantWorkloadKind::TenantRbac,
        Fd001TenantWorkloadKind::HrEmployment,
        Fd001TenantWorkloadKind::PayrollRun,
        Fd001TenantWorkloadKind::AccountingJournal,
    ]
}

fn required_requirement_kinds() -> [Fd001TenantResourceQuotaRequirementKind; 13] {
    [
        Fd001TenantResourceQuotaRequirementKind::NamespaceResourceQuotaRequired,
        Fd001TenantResourceQuotaRequirementKind::ComputeRequestsQuotaRequired,
        Fd001TenantResourceQuotaRequirementKind::ComputeLimitsQuotaRequired,
        Fd001TenantResourceQuotaRequirementKind::ObjectCountQuotaRequired,
        Fd001TenantResourceQuotaRequirementKind::PersistentStorageQuotaRequired,
        Fd001TenantResourceQuotaRequirementKind::LimitRangeDefaultsRequired,
        Fd001TenantResourceQuotaRequirementKind::LimitRangeMinMaxRequired,
        Fd001TenantResourceQuotaRequirementKind::ContainerRequestsLimitsRequired,
        Fd001TenantResourceQuotaRequirementKind::ResourceQuotaAdmissionEvidenceRequired,
        Fd001TenantResourceQuotaRequirementKind::LimitRangerAdmissionEvidenceRequired,
        Fd001TenantResourceQuotaRequirementKind::TenantLabelSelectorRequired,
        Fd001TenantResourceQuotaRequirementKind::QuotaUsageAuditEvidenceRequired,
        Fd001TenantResourceQuotaRequirementKind::AdmissionPolicyEvidenceRequired,
    ]
}

fn validate_kubernetes_resource_kind(value: &str) -> Result<(), Fd001TenantResourceQuotaError> {
    if ![
        "ResourceQuota",
        "LimitRange",
        "PodSpecResources",
        "AdmissionController",
        "NamespaceLabel",
        "ResourceQuotaStatus",
        "ValidatingAdmissionPolicy",
    ]
    .contains(&value)
    {
        return Err(Fd001TenantResourceQuotaError::InvalidKubernetesResourceKind);
    }
    Ok(())
}

fn validate_doc_url(url: &str) -> Result<(), Fd001TenantResourceQuotaError> {
    if has_unsafe_ref_text(url)
        || ![
            RESOURCE_QUOTA_DOC_URL,
            LIMIT_RANGE_DOC_URL,
            RESOURCE_MANAGEMENT_DOC_URL,
            ADMISSION_CONTROLLERS_DOC_URL,
            NAMESPACES_DOC_URL,
            MULTI_TENANCY_DOC_URL,
        ]
        .contains(&url)
    {
        return Err(Fd001TenantResourceQuotaError::InvalidOfficialDocUrl);
    }
    Ok(())
}

fn validate_slug(
    value: &str,
    error: Fd001TenantResourceQuotaError,
) -> Result<(), Fd001TenantResourceQuotaError> {
    if value.is_empty()
        || has_unsafe_text(value)
        || value
            .chars()
            .any(|ch| !(ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-'))
    {
        return Err(error);
    }
    Ok(())
}

fn validate_tenant_namespace(value: &str) -> Result<(), Fd001TenantResourceQuotaError> {
    validate_slug(value, Fd001TenantResourceQuotaError::InvalidTenantNamespace)?;
    if value != TENANT_NAMESPACE {
        return Err(Fd001TenantResourceQuotaError::InvalidTenantNamespace);
    }
    Ok(())
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: Fd001TenantResourceQuotaError,
) -> Result<(), Fd001TenantResourceQuotaError> {
    if value.len() <= prefix.len() || !value.starts_with(prefix) || has_unsafe_ref_text(value) {
        return Err(error);
    }
    Ok(())
}

fn require_control(
    value: bool,
    control: &'static str,
) -> Result<(), Fd001TenantResourceQuotaError> {
    if value {
        Ok(())
    } else {
        Err(Fd001TenantResourceQuotaError::MissingRequiredControl(
            control,
        ))
    }
}

fn has_unsafe_text(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.trim() != value
        || value.contains("..")
        || value.contains('\\')
        || value.contains('/')
        || value.chars().any(char::is_control)
        || lower.contains("pending")
        || lower.contains("todo")
        || lower.contains("fixme")
        || lower.contains("placeholder")
        || lower.contains("mock")
        || lower.contains("stub")
        || lower.contains("password")
        || lower.contains("secret")
        || lower.contains("credential")
        || lower.contains("api_key")
        || lower.contains("bearer")
}

fn has_unsafe_ref_text(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.trim() != value
        || value.contains("..")
        || value.contains('\\')
        || value.chars().any(char::is_control)
        || lower.contains("pending")
        || lower.contains("todo")
        || lower.contains("fixme")
        || lower.contains("placeholder")
        || lower.contains("mock")
        || lower.contains("stub")
        || lower.contains("password")
        || lower.contains("secret")
        || lower.contains("credential")
        || lower.contains("api_key")
        || lower.contains("bearer")
}
