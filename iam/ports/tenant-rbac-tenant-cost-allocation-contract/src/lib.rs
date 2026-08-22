//! FD-001 tenant cost-allocation contract for later Oyatie Cloud dogfooding.
//!
//! This review-only crate defines FinOps/showback metadata guardrails that must
//! exist before FD-001 Tenant RBAC, HR, Payroll, and Accounting workloads
//! can be promoted as production tenant workloads on the future Oyatie Cloud
//! substrate. It binds the FD-001 tenant-workload manifest and tenant admission
//! policy to Kubernetes cost-allocation labels, recommended application labels,
//! namespace boundaries, resource-request and quota-usage evidence,
//! OpenTelemetry service and Kubernetes resource attributes, FinOps allocation
//! strategy, shared-cost policy, allocation coverage KPIs, tenant label
//! selectors, cost-allocation audit evidence, and admission-policy evidence. It
//! does not attach a Kubernetes cluster, resource metrics runtime, OpenTelemetry
//! collector runtime, FinOps runtime, cost report generation, billing export,
//! workload runtime, cloud substrate runtime, or runtime audit-chain emission.
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
const CONTRACT_NAME: &str = "fd001-tenant-rbac-tenant-cost-allocation-contract";
const PROGRAM_NAME: &str = "fd-001-tenant-rbac-generic";
const SUBSTRATE_NAME: &str = "oyatie-cloud";
const TENANT_NAMESPACE: &str = "oyatie-fd001-tenant-rbac-dev";
const SOURCE_MANIFEST_REF: &str =
    "crates/tenant-rbac-tenant-workload-manifest/src/lib.rs::fd001_tenant_workload_manifest";
const SOURCE_ADMISSION_POLICY_REF: &str = "crates/tenant-rbac-tenant-admission-policy/src/lib.rs::fd001_tenant_admission_policy_contract";
const SOURCE_FINOPS_REF: &str = "crates/cloud-finops-kernel/src/lib.rs::CostReport";
const POLICY_REF_PREFIX: &str = "policy/cost-allocation/fd001/";
const EXPECTED_EVIDENCE_REF: &str =
    "evidence/cost-allocation/fd001-tenant-rbac/cost-allocation-contract-review.jsonl";

const FINOPS_ALLOCATION_DOC_URL: &str = "https://www.finops.org/framework/capabilities/allocation/";
const KUBERNETES_COMMON_LABELS_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/overview/working-with-objects/common-labels/";
const KUBERNETES_LABELS_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/overview/working-with-objects/labels/";
const RESOURCE_MANAGEMENT_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/";
const RESOURCE_QUOTA_DOC_URL: &str = "https://kubernetes.io/docs/concepts/policy/resource-quotas/";
const OTEL_RESOURCES_DOC_URL: &str = "https://opentelemetry.io/docs/concepts/resources/";
const OTEL_RESOURCE_SEMCONV_DOC_URL: &str = "https://opentelemetry.io/docs/specs/semconv/resource/";
const OTEL_K8S_RESOURCE_DOC_URL: &str = "https://opentelemetry.io/docs/specs/semconv/resource/k8s/";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Fd001TenantCostAllocationRequirementKind {
    TenantCostAllocationLabelsRequired,
    KubernetesRecommendedLabelsRequired,
    NamespaceCostBoundaryRequired,
    WorkloadResourceRequestsRequired,
    ResourceQuotaUsageEvidenceRequired,
    OpenTelemetryServiceResourceRequired,
    OpenTelemetryKubernetesResourceAttributesRequired,
    FinOpsAllocationStrategyRequired,
    SharedCostPolicyRequired,
    AllocationCoverageKpiRequired,
    TenantLabelSelectorRequired,
    CostAllocationAuditEvidenceRequired,
    AdmissionPolicyEvidenceRequired,
}

impl Fd001TenantCostAllocationRequirementKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TenantCostAllocationLabelsRequired => "tenant_cost_allocation_labels_required",
            Self::KubernetesRecommendedLabelsRequired => "kubernetes_recommended_labels_required",
            Self::NamespaceCostBoundaryRequired => "namespace_cost_boundary_required",
            Self::WorkloadResourceRequestsRequired => "workload_resource_requests_required",
            Self::ResourceQuotaUsageEvidenceRequired => "resource_quota_usage_evidence_required",
            Self::OpenTelemetryServiceResourceRequired => "opentelemetry_service_resource_required",
            Self::OpenTelemetryKubernetesResourceAttributesRequired => {
                "opentelemetry_kubernetes_resource_attributes_required"
            }
            Self::FinOpsAllocationStrategyRequired => "finops_allocation_strategy_required",
            Self::SharedCostPolicyRequired => "shared_cost_policy_required",
            Self::AllocationCoverageKpiRequired => "allocation_coverage_kpi_required",
            Self::TenantLabelSelectorRequired => "tenant_label_selector_required",
            Self::CostAllocationAuditEvidenceRequired => "cost_allocation_audit_evidence_required",
            Self::AdmissionPolicyEvidenceRequired => "admission_policy_evidence_required",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fd001TenantCostAllocationRequirement {
    pub requirement_id: &'static str,           // data_class: PUBLIC
    pub workload_kind: Fd001TenantWorkloadKind, // data_class: PUBLIC
    pub requirement_kind: Fd001TenantCostAllocationRequirementKind, // data_class: PUBLIC
    pub kubernetes_resource_kind: &'static str, // data_class: PUBLIC
    pub policy_ref: &'static str,               // data_class: INTERNAL_ONLY
    pub expected_evidence_ref: &'static str,    // data_class: INTERNAL_ONLY
    pub official_doc_url: &'static str,         // data_class: PUBLIC
    pub source_manifest_ref: &'static str,      // data_class: INTERNAL_ONLY
    pub source_admission_policy_ref: &'static str, // data_class: INTERNAL_ONLY
    pub source_finops_ref: &'static str,        // data_class: INTERNAL_ONLY
    pub applies_to_all_manifest_workloads: bool, // data_class: PUBLIC
    pub runtime_observation_attached: bool,     // data_class: INTERNAL_ONLY
    pub schema_version: u32,                    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fd001TenantCostAllocationContract {
    pub contract_name: &'static str,          // data_class: PUBLIC
    pub program_name: &'static str,           // data_class: PUBLIC
    pub substrate_name: &'static str,         // data_class: PUBLIC
    pub tenant_namespace: &'static str,       // data_class: INTERNAL_ONLY
    pub workload_manifest_name: &'static str, // data_class: PUBLIC
    pub workload_manifest_count: usize,       // data_class: PUBLIC
    pub tenant_admission_policy_contract_name: &'static str, // data_class: PUBLIC
    pub finops_kernel_ref: &'static str,      // data_class: INTERNAL_ONLY
    pub requirements: Vec<Fd001TenantCostAllocationRequirement>, // data_class: INTERNAL_ONLY
    pub official_docs_required: bool,         // data_class: PUBLIC
    pub all_manifest_workloads_in_scope: bool, // data_class: PUBLIC
    pub tenant_cost_allocation_labels_required: bool, // data_class: PUBLIC
    pub kubernetes_recommended_labels_required: bool, // data_class: PUBLIC
    pub namespace_cost_boundary_required: bool, // data_class: PUBLIC
    pub workload_resource_requests_required: bool, // data_class: PUBLIC
    pub resource_quota_usage_evidence_required: bool, // data_class: PUBLIC
    pub opentelemetry_service_resource_required: bool, // data_class: PUBLIC
    pub opentelemetry_kubernetes_resource_attributes_required: bool, // data_class: PUBLIC
    pub finops_allocation_strategy_required: bool, // data_class: PUBLIC
    pub shared_cost_policy_required: bool,    // data_class: PUBLIC
    pub allocation_coverage_kpi_required: bool, // data_class: PUBLIC
    pub tenant_label_selector_required: bool, // data_class: PUBLIC
    pub cost_allocation_audit_evidence_required: bool, // data_class: PUBLIC
    pub admission_policy_evidence_required: bool, // data_class: PUBLIC
    pub review_only_contract: bool,           // data_class: PUBLIC
    pub kubernetes_cluster_attached: bool,    // data_class: INTERNAL_ONLY
    pub resource_metrics_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub otel_collector_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub finops_runtime_attached: bool,        // data_class: INTERNAL_ONLY
    pub cost_report_runtime_generated: bool,  // data_class: INTERNAL_ONLY
    pub billing_export_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub workload_runtime_deployed: bool,      // data_class: INTERNAL_ONLY
    pub cloud_substrate_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Fd001TenantCostAllocationError {
    WorkloadManifest(Fd001TenantWorkloadManifestError),
    TenantAdmissionPolicy(Fd001TenantAdmissionPolicyError),
    InvalidContractName,
    InvalidProgramName,
    InvalidSubstrateName,
    InvalidTenantNamespace,
    InvalidManifestName,
    InvalidWorkloadCount,
    InvalidAdmissionPolicyContractName,
    InvalidFinopsKernelRef,
    MissingRequirements,
    DuplicateRequirement(String),
    MissingWorkloadKind(Fd001TenantWorkloadKind),
    MissingRequirementKind(Fd001TenantCostAllocationRequirementKind),
    InvalidRequirementId,
    InvalidKubernetesResourceKind,
    InvalidPolicyRef,
    InvalidExpectedEvidenceRef,
    InvalidOfficialDocUrl,
    InvalidSourceManifestRef,
    InvalidSourceAdmissionPolicyRef,
    InvalidSourceFinopsRef,
    MissingRequiredControl(&'static str),
    RuntimeAttachmentOverclaim,
}

pub fn fd001_tenant_cost_allocation_contract()
-> Result<Fd001TenantCostAllocationContract, Fd001TenantCostAllocationError> {
    let manifest = fd001_tenant_workload_manifest();
    validate_fd001_tenant_workload_manifest(&manifest)
        .map_err(Fd001TenantCostAllocationError::WorkloadManifest)?;
    let admission_policy = fd001_tenant_admission_policy_contract()
        .map_err(Fd001TenantCostAllocationError::TenantAdmissionPolicy)?;
    validate_fd001_tenant_admission_policy_contract(&admission_policy)
        .map_err(Fd001TenantCostAllocationError::TenantAdmissionPolicy)?;

    Ok(Fd001TenantCostAllocationContract {
        contract_name: CONTRACT_NAME,
        program_name: PROGRAM_NAME,
        substrate_name: SUBSTRATE_NAME,
        tenant_namespace: manifest.tenant_namespace,
        workload_manifest_name: manifest.manifest_name,
        workload_manifest_count: manifest.workloads.len(),
        tenant_admission_policy_contract_name: admission_policy.contract_name,
        finops_kernel_ref: SOURCE_FINOPS_REF,
        requirements: cost_allocation_requirements(),
        official_docs_required: true,
        all_manifest_workloads_in_scope: true,
        tenant_cost_allocation_labels_required: true,
        kubernetes_recommended_labels_required: true,
        namespace_cost_boundary_required: true,
        workload_resource_requests_required: true,
        resource_quota_usage_evidence_required: true,
        opentelemetry_service_resource_required: true,
        opentelemetry_kubernetes_resource_attributes_required: true,
        finops_allocation_strategy_required: true,
        shared_cost_policy_required: true,
        allocation_coverage_kpi_required: true,
        tenant_label_selector_required: true,
        cost_allocation_audit_evidence_required: true,
        admission_policy_evidence_required: true,
        review_only_contract: true,
        kubernetes_cluster_attached: false,
        resource_metrics_runtime_attached: false,
        otel_collector_runtime_attached: false,
        finops_runtime_attached: false,
        cost_report_runtime_generated: false,
        billing_export_runtime_attached: false,
        workload_runtime_deployed: false,
        cloud_substrate_runtime_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: SCHEMA_VERSION,
    })
}

pub fn validate_fd001_tenant_cost_allocation_contract(
    contract: &Fd001TenantCostAllocationContract,
) -> Result<(), Fd001TenantCostAllocationError> {
    let manifest = fd001_tenant_workload_manifest();
    validate_fd001_tenant_workload_manifest(&manifest)
        .map_err(Fd001TenantCostAllocationError::WorkloadManifest)?;
    let admission_policy = fd001_tenant_admission_policy_contract()
        .map_err(Fd001TenantCostAllocationError::TenantAdmissionPolicy)?;
    validate_fd001_tenant_admission_policy_contract(&admission_policy)
        .map_err(Fd001TenantCostAllocationError::TenantAdmissionPolicy)?;

    validate_slug(
        contract.contract_name,
        Fd001TenantCostAllocationError::InvalidContractName,
    )?;
    if contract.program_name != PROGRAM_NAME {
        return Err(Fd001TenantCostAllocationError::InvalidProgramName);
    }
    if contract.substrate_name != SUBSTRATE_NAME {
        return Err(Fd001TenantCostAllocationError::InvalidSubstrateName);
    }
    validate_tenant_namespace(contract.tenant_namespace)?;
    if contract.workload_manifest_name != manifest.manifest_name {
        return Err(Fd001TenantCostAllocationError::InvalidManifestName);
    }
    if contract.workload_manifest_count != manifest.workloads.len() {
        return Err(Fd001TenantCostAllocationError::InvalidWorkloadCount);
    }
    if contract.tenant_admission_policy_contract_name != admission_policy.contract_name {
        return Err(Fd001TenantCostAllocationError::InvalidAdmissionPolicyContractName);
    }
    validate_prefixed_ref(
        contract.finops_kernel_ref,
        "crates/cloud-finops-kernel/",
        Fd001TenantCostAllocationError::InvalidFinopsKernelRef,
    )?;
    if contract.requirements.len() < MIN_REQUIREMENT_COUNT
        || contract.schema_version != SCHEMA_VERSION
    {
        return Err(Fd001TenantCostAllocationError::MissingRequirements);
    }
    validate_required_controls(contract)?;
    validate_nonclaims(contract)?;
    validate_requirements(contract)?;
    Ok(())
}

pub fn fd001_tenant_cost_allocation_doc_urls(
    contract: &Fd001TenantCostAllocationContract,
) -> Vec<&'static str> {
    let mut docs = BTreeSet::new();
    for requirement in &contract.requirements {
        docs.insert(requirement.official_doc_url);
    }
    docs.into_iter().collect()
}

fn cost_allocation_requirements() -> Vec<Fd001TenantCostAllocationRequirement> {
    vec![
        requirement(
            "tenant-cost-allocation-labels-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantCostAllocationRequirementKind::TenantCostAllocationLabelsRequired,
            "KubernetesLabel",
            "tenant-cost-allocation-labels",
            KUBERNETES_LABELS_DOC_URL,
        ),
        requirement(
            "kubernetes-recommended-labels-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantCostAllocationRequirementKind::KubernetesRecommendedLabelsRequired,
            "KubernetesLabel",
            "kubernetes-recommended-labels",
            KUBERNETES_COMMON_LABELS_DOC_URL,
        ),
        requirement(
            "namespace-cost-boundary-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantCostAllocationRequirementKind::NamespaceCostBoundaryRequired,
            "KubernetesNamespace",
            "namespace-cost-boundary",
            KUBERNETES_LABELS_DOC_URL,
        ),
        requirement(
            "workload-resource-requests-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantCostAllocationRequirementKind::WorkloadResourceRequestsRequired,
            "PodSpecResources",
            "workload-resource-requests",
            RESOURCE_MANAGEMENT_DOC_URL,
        ),
        requirement(
            "resource-quota-usage-evidence-all-workloads",
            Fd001TenantWorkloadKind::AccountingJournal,
            Fd001TenantCostAllocationRequirementKind::ResourceQuotaUsageEvidenceRequired,
            "ResourceQuotaStatus",
            "resource-quota-usage-evidence",
            RESOURCE_QUOTA_DOC_URL,
        ),
        requirement(
            "otel-service-resource-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantCostAllocationRequirementKind::OpenTelemetryServiceResourceRequired,
            "OpenTelemetryResource",
            "otel-service-resource",
            OTEL_RESOURCES_DOC_URL,
        ),
        requirement(
            "otel-kubernetes-resource-attributes-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantCostAllocationRequirementKind::OpenTelemetryKubernetesResourceAttributesRequired,
            "OpenTelemetryKubernetesResource",
            "otel-kubernetes-resource-attributes",
            OTEL_K8S_RESOURCE_DOC_URL,
        ),
        requirement(
            "finops-allocation-strategy-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantCostAllocationRequirementKind::FinOpsAllocationStrategyRequired,
            "FinOpsAllocationStrategy",
            "finops-allocation-strategy",
            FINOPS_ALLOCATION_DOC_URL,
        ),
        requirement(
            "shared-cost-policy-all-workloads",
            Fd001TenantWorkloadKind::AccountingJournal,
            Fd001TenantCostAllocationRequirementKind::SharedCostPolicyRequired,
            "SharedCostPolicy",
            "shared-cost-policy",
            FINOPS_ALLOCATION_DOC_URL,
        ),
        requirement(
            "allocation-coverage-kpi-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantCostAllocationRequirementKind::AllocationCoverageKpiRequired,
            "AllocationKpi",
            "allocation-coverage-kpi",
            FINOPS_ALLOCATION_DOC_URL,
        ),
        requirement(
            "tenant-label-selector-cost-allocation-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantCostAllocationRequirementKind::TenantLabelSelectorRequired,
            "LabelSelector",
            "tenant-label-selector-cost-allocation",
            KUBERNETES_LABELS_DOC_URL,
        ),
        requirement(
            "cost-allocation-audit-evidence-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantCostAllocationRequirementKind::CostAllocationAuditEvidenceRequired,
            "CostAllocationAuditEvidence",
            "cost-allocation-audit-evidence",
            OTEL_RESOURCE_SEMCONV_DOC_URL,
        ),
        requirement(
            "admission-policy-cost-allocation-evidence-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantCostAllocationRequirementKind::AdmissionPolicyEvidenceRequired,
            "ValidatingAdmissionPolicy",
            "admission-policy-cost-allocation-evidence",
            KUBERNETES_COMMON_LABELS_DOC_URL,
        ),
    ]
}

fn requirement(
    requirement_id: &'static str,
    workload_kind: Fd001TenantWorkloadKind,
    requirement_kind: Fd001TenantCostAllocationRequirementKind,
    kubernetes_resource_kind: &'static str,
    policy_suffix: &'static str,
    official_doc_url: &'static str,
) -> Fd001TenantCostAllocationRequirement {
    Fd001TenantCostAllocationRequirement {
        requirement_id,
        workload_kind,
        requirement_kind,
        kubernetes_resource_kind,
        policy_ref: policy_ref(policy_suffix),
        expected_evidence_ref: EXPECTED_EVIDENCE_REF,
        official_doc_url,
        source_manifest_ref: SOURCE_MANIFEST_REF,
        source_admission_policy_ref: SOURCE_ADMISSION_POLICY_REF,
        source_finops_ref: SOURCE_FINOPS_REF,
        applies_to_all_manifest_workloads: true,
        runtime_observation_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

fn policy_ref(policy_suffix: &'static str) -> &'static str {
    match policy_suffix {
        "tenant-cost-allocation-labels" => {
            "policy/cost-allocation/fd001/tenant-cost-allocation-labels"
        }
        "kubernetes-recommended-labels" => {
            "policy/cost-allocation/fd001/kubernetes-recommended-labels"
        }
        "namespace-cost-boundary" => "policy/cost-allocation/fd001/namespace-cost-boundary",
        "workload-resource-requests" => "policy/cost-allocation/fd001/workload-resource-requests",
        "resource-quota-usage-evidence" => {
            "policy/cost-allocation/fd001/resource-quota-usage-evidence"
        }
        "otel-service-resource" => "policy/cost-allocation/fd001/otel-service-resource",
        "otel-kubernetes-resource-attributes" => {
            "policy/cost-allocation/fd001/otel-kubernetes-resource-attributes"
        }
        "finops-allocation-strategy" => "policy/cost-allocation/fd001/finops-allocation-strategy",
        "shared-cost-policy" => "policy/cost-allocation/fd001/shared-cost-policy",
        "allocation-coverage-kpi" => "policy/cost-allocation/fd001/allocation-coverage-kpi",
        "tenant-label-selector-cost-allocation" => {
            "policy/cost-allocation/fd001/tenant-label-selector-cost-allocation"
        }
        "cost-allocation-audit-evidence" => {
            "policy/cost-allocation/fd001/cost-allocation-audit-evidence"
        }
        "admission-policy-cost-allocation-evidence" => {
            "policy/cost-allocation/fd001/admission-policy-cost-allocation-evidence"
        }
        _ => "policy/cost-allocation/fd001/invalid",
    }
}

fn validate_required_controls(
    contract: &Fd001TenantCostAllocationContract,
) -> Result<(), Fd001TenantCostAllocationError> {
    for (enabled, name) in [
        (contract.official_docs_required, "official_docs_required"),
        (
            contract.all_manifest_workloads_in_scope,
            "all_manifest_workloads_in_scope",
        ),
        (
            contract.tenant_cost_allocation_labels_required,
            "tenant_cost_allocation_labels_required",
        ),
        (
            contract.kubernetes_recommended_labels_required,
            "kubernetes_recommended_labels_required",
        ),
        (
            contract.namespace_cost_boundary_required,
            "namespace_cost_boundary_required",
        ),
        (
            contract.workload_resource_requests_required,
            "workload_resource_requests_required",
        ),
        (
            contract.resource_quota_usage_evidence_required,
            "resource_quota_usage_evidence_required",
        ),
        (
            contract.opentelemetry_service_resource_required,
            "opentelemetry_service_resource_required",
        ),
        (
            contract.opentelemetry_kubernetes_resource_attributes_required,
            "opentelemetry_kubernetes_resource_attributes_required",
        ),
        (
            contract.finops_allocation_strategy_required,
            "finops_allocation_strategy_required",
        ),
        (
            contract.shared_cost_policy_required,
            "shared_cost_policy_required",
        ),
        (
            contract.allocation_coverage_kpi_required,
            "allocation_coverage_kpi_required",
        ),
        (
            contract.tenant_label_selector_required,
            "tenant_label_selector_required",
        ),
        (
            contract.cost_allocation_audit_evidence_required,
            "cost_allocation_audit_evidence_required",
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
    contract: &Fd001TenantCostAllocationContract,
) -> Result<(), Fd001TenantCostAllocationError> {
    if contract.kubernetes_cluster_attached
        || contract.resource_metrics_runtime_attached
        || contract.otel_collector_runtime_attached
        || contract.finops_runtime_attached
        || contract.cost_report_runtime_generated
        || contract.billing_export_runtime_attached
        || contract.workload_runtime_deployed
        || contract.cloud_substrate_runtime_attached
        || contract.runtime_audit_chain_emission_attached
    {
        return Err(Fd001TenantCostAllocationError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_requirements(
    contract: &Fd001TenantCostAllocationContract,
) -> Result<(), Fd001TenantCostAllocationError> {
    let mut seen_requirements = BTreeSet::new();
    let mut seen_workloads = BTreeSet::new();
    let mut seen_requirement_kinds = BTreeSet::new();
    for requirement in &contract.requirements {
        validate_requirement(requirement)?;
        if !seen_requirements.insert(requirement.requirement_id) {
            return Err(Fd001TenantCostAllocationError::DuplicateRequirement(
                requirement.requirement_id.to_owned(),
            ));
        }
        seen_workloads.insert(requirement.workload_kind);
        seen_requirement_kinds.insert(requirement.requirement_kind);
    }
    for workload_kind in required_workload_kinds() {
        if !seen_workloads.contains(&workload_kind) {
            return Err(Fd001TenantCostAllocationError::MissingWorkloadKind(
                workload_kind,
            ));
        }
    }
    for requirement_kind in required_requirement_kinds() {
        if !seen_requirement_kinds.contains(&requirement_kind) {
            return Err(Fd001TenantCostAllocationError::MissingRequirementKind(
                requirement_kind,
            ));
        }
    }
    Ok(())
}

fn validate_requirement(
    requirement: &Fd001TenantCostAllocationRequirement,
) -> Result<(), Fd001TenantCostAllocationError> {
    validate_slug(
        requirement.requirement_id,
        Fd001TenantCostAllocationError::InvalidRequirementId,
    )?;
    validate_kubernetes_resource_kind(requirement.kubernetes_resource_kind)?;
    validate_prefixed_ref(
        requirement.policy_ref,
        POLICY_REF_PREFIX,
        Fd001TenantCostAllocationError::InvalidPolicyRef,
    )?;
    validate_prefixed_ref(
        requirement.expected_evidence_ref,
        "evidence/cost-allocation/fd001-tenant-rbac/",
        Fd001TenantCostAllocationError::InvalidExpectedEvidenceRef,
    )?;
    validate_doc_url(requirement.official_doc_url)?;
    validate_prefixed_ref(
        requirement.source_manifest_ref,
        "crates/tenant-rbac-tenant-workload-manifest/",
        Fd001TenantCostAllocationError::InvalidSourceManifestRef,
    )?;
    validate_prefixed_ref(
        requirement.source_admission_policy_ref,
        "crates/tenant-rbac-tenant-admission-policy/",
        Fd001TenantCostAllocationError::InvalidSourceAdmissionPolicyRef,
    )?;
    validate_prefixed_ref(
        requirement.source_finops_ref,
        "crates/cloud-finops-kernel/",
        Fd001TenantCostAllocationError::InvalidSourceFinopsRef,
    )?;
    if !requirement.applies_to_all_manifest_workloads {
        return Err(Fd001TenantCostAllocationError::MissingRequiredControl(
            "requirement_applies_to_all_manifest_workloads",
        ));
    }
    if requirement.runtime_observation_attached || requirement.schema_version != SCHEMA_VERSION {
        return Err(Fd001TenantCostAllocationError::RuntimeAttachmentOverclaim);
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

fn required_requirement_kinds() -> [Fd001TenantCostAllocationRequirementKind; 13] {
    [
        Fd001TenantCostAllocationRequirementKind::TenantCostAllocationLabelsRequired,
        Fd001TenantCostAllocationRequirementKind::KubernetesRecommendedLabelsRequired,
        Fd001TenantCostAllocationRequirementKind::NamespaceCostBoundaryRequired,
        Fd001TenantCostAllocationRequirementKind::WorkloadResourceRequestsRequired,
        Fd001TenantCostAllocationRequirementKind::ResourceQuotaUsageEvidenceRequired,
        Fd001TenantCostAllocationRequirementKind::OpenTelemetryServiceResourceRequired,
        Fd001TenantCostAllocationRequirementKind::OpenTelemetryKubernetesResourceAttributesRequired,
        Fd001TenantCostAllocationRequirementKind::FinOpsAllocationStrategyRequired,
        Fd001TenantCostAllocationRequirementKind::SharedCostPolicyRequired,
        Fd001TenantCostAllocationRequirementKind::AllocationCoverageKpiRequired,
        Fd001TenantCostAllocationRequirementKind::TenantLabelSelectorRequired,
        Fd001TenantCostAllocationRequirementKind::CostAllocationAuditEvidenceRequired,
        Fd001TenantCostAllocationRequirementKind::AdmissionPolicyEvidenceRequired,
    ]
}

fn validate_kubernetes_resource_kind(value: &str) -> Result<(), Fd001TenantCostAllocationError> {
    if ![
        "KubernetesLabel",
        "KubernetesNamespace",
        "PodSpecResources",
        "ResourceQuotaStatus",
        "OpenTelemetryResource",
        "OpenTelemetryKubernetesResource",
        "FinOpsAllocationStrategy",
        "SharedCostPolicy",
        "AllocationKpi",
        "LabelSelector",
        "CostAllocationAuditEvidence",
        "ValidatingAdmissionPolicy",
    ]
    .contains(&value)
    {
        return Err(Fd001TenantCostAllocationError::InvalidKubernetesResourceKind);
    }
    Ok(())
}

fn validate_doc_url(url: &str) -> Result<(), Fd001TenantCostAllocationError> {
    if has_unsafe_ref_text(url)
        || ![
            FINOPS_ALLOCATION_DOC_URL,
            KUBERNETES_COMMON_LABELS_DOC_URL,
            KUBERNETES_LABELS_DOC_URL,
            RESOURCE_MANAGEMENT_DOC_URL,
            RESOURCE_QUOTA_DOC_URL,
            OTEL_RESOURCES_DOC_URL,
            OTEL_RESOURCE_SEMCONV_DOC_URL,
            OTEL_K8S_RESOURCE_DOC_URL,
        ]
        .contains(&url)
    {
        return Err(Fd001TenantCostAllocationError::InvalidOfficialDocUrl);
    }
    Ok(())
}

fn validate_slug(
    value: &str,
    error: Fd001TenantCostAllocationError,
) -> Result<(), Fd001TenantCostAllocationError> {
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

fn validate_tenant_namespace(value: &str) -> Result<(), Fd001TenantCostAllocationError> {
    validate_slug(
        value,
        Fd001TenantCostAllocationError::InvalidTenantNamespace,
    )?;
    if value != TENANT_NAMESPACE {
        return Err(Fd001TenantCostAllocationError::InvalidTenantNamespace);
    }
    Ok(())
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: Fd001TenantCostAllocationError,
) -> Result<(), Fd001TenantCostAllocationError> {
    if value.len() <= prefix.len() || !value.starts_with(prefix) || has_unsafe_ref_text(value) {
        return Err(error);
    }
    Ok(())
}

fn require_control(
    value: bool,
    control: &'static str,
) -> Result<(), Fd001TenantCostAllocationError> {
    if value {
        Ok(())
    } else {
        Err(Fd001TenantCostAllocationError::MissingRequiredControl(
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
