//! FD-001 tenant autoscaling contract for later Oyatie Cloud dogfooding.
//!
//! This review-only crate defines workload elasticity guardrails that must exist
//! before FD-001 Tenant RBAC, HR, Payroll, and Accounting workloads can be
//! promoted as production tenant workloads on the future Oyatie Cloud substrate.
//! It binds the FD-001 tenant-workload manifest and tenant admission policy to
//! Kubernetes HorizontalPodAutoscaler, autoscaling/v2 API requirements,
//! min/max replica bounds, CPU and memory resource metrics, metrics pipeline
//! evidence, scale-up and scale-down behavior policies, stabilization windows,
//! tenant-label selectors, scaling audit evidence, and admission-policy evidence.
//! It does not attach a Kubernetes cluster, attach Metrics Server runtime,
//! attach a supplemental metrics API, apply HorizontalPodAutoscaler objects,
//! observe autoscaling controller runtime, observe scale events, deploy
//! workloads, attach a cloud substrate runtime, or emit runtime audit-chain events.
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
const CONTRACT_NAME: &str = "fd001-tenant-rbac-tenant-autoscaling-contract";
const PROGRAM_NAME: &str = "fd-001-tenant-rbac-generic";
const SUBSTRATE_NAME: &str = "oyatie-cloud";
const TENANT_NAMESPACE: &str = "oyatie-fd001-tenant-rbac-dev";
const SOURCE_MANIFEST_REF: &str =
    "crates/tenant-rbac-tenant-workload-manifest/src/lib.rs::fd001_tenant_workload_manifest";
const SOURCE_ADMISSION_POLICY_REF: &str = "crates/tenant-rbac-tenant-admission-policy/src/lib.rs::fd001_tenant_admission_policy_contract";
const POLICY_REF_PREFIX: &str = "policy/autoscaling/fd001/";
const EXPECTED_EVIDENCE_REF: &str =
    "evidence/autoscaling/fd001-tenant-rbac/autoscaling-contract-review.jsonl";

const HPA_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/workloads/autoscaling/horizontal-pod-autoscale/";
const HPA_WALKTHROUGH_DOC_URL: &str =
    "https://kubernetes.io/docs/tasks/run-application/horizontal-pod-autoscale-walkthrough/";
const RESOURCE_METRICS_PIPELINE_DOC_URL: &str =
    "https://kubernetes.io/docs/tasks/debug/debug-cluster/resource-metrics-pipeline/";
const WORKLOAD_AUTOSCALING_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/workloads/autoscaling/";
const HPA_API_DOC_URL: &str =
    "https://kubernetes.io/docs/reference/kubernetes-api/autoscaling/horizontal-pod-autoscaler-v2/";
const RESOURCE_MANAGEMENT_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Fd001TenantAutoscalingRequirementKind {
    HorizontalPodAutoscalerRequired,
    AutoscalingV2ApiRequired,
    MinReplicaFloorRequired,
    MaxReplicaCeilingRequired,
    CpuResourceMetricRequired,
    MemoryResourceMetricRequired,
    MetricsPipelineEvidenceRequired,
    ScaleUpBehaviorPolicyRequired,
    ScaleDownBehaviorPolicyRequired,
    StabilizationWindowRequired,
    TenantLabelSelectorRequired,
    ScalingAuditEvidenceRequired,
    AdmissionPolicyEvidenceRequired,
}

impl Fd001TenantAutoscalingRequirementKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HorizontalPodAutoscalerRequired => "horizontal_pod_autoscaler_required",
            Self::AutoscalingV2ApiRequired => "autoscaling_v2_api_required",
            Self::MinReplicaFloorRequired => "min_replica_floor_required",
            Self::MaxReplicaCeilingRequired => "max_replica_ceiling_required",
            Self::CpuResourceMetricRequired => "cpu_resource_metric_required",
            Self::MemoryResourceMetricRequired => "memory_resource_metric_required",
            Self::MetricsPipelineEvidenceRequired => "metrics_pipeline_evidence_required",
            Self::ScaleUpBehaviorPolicyRequired => "scale_up_behavior_policy_required",
            Self::ScaleDownBehaviorPolicyRequired => "scale_down_behavior_policy_required",
            Self::StabilizationWindowRequired => "stabilization_window_required",
            Self::TenantLabelSelectorRequired => "tenant_label_selector_required",
            Self::ScalingAuditEvidenceRequired => "scaling_audit_evidence_required",
            Self::AdmissionPolicyEvidenceRequired => "admission_policy_evidence_required",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fd001TenantAutoscalingRequirement {
    pub requirement_id: &'static str,           // data_class: PUBLIC
    pub workload_kind: Fd001TenantWorkloadKind, // data_class: PUBLIC
    pub requirement_kind: Fd001TenantAutoscalingRequirementKind, // data_class: PUBLIC
    pub kubernetes_resource_kind: &'static str, // data_class: PUBLIC
    pub policy_ref: &'static str,               // data_class: INTERNAL_ONLY
    pub expected_evidence_ref: &'static str,    // data_class: INTERNAL_ONLY
    pub official_doc_url: &'static str,         // data_class: PUBLIC
    pub source_manifest_ref: &'static str,      // data_class: INTERNAL_ONLY
    pub source_admission_policy_ref: &'static str, // data_class: INTERNAL_ONLY
    pub applies_to_all_manifest_workloads: bool, // data_class: PUBLIC
    pub runtime_observation_attached: bool,     // data_class: INTERNAL_ONLY
    pub schema_version: u32,                    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fd001TenantAutoscalingContract {
    pub contract_name: &'static str,          // data_class: PUBLIC
    pub program_name: &'static str,           // data_class: PUBLIC
    pub substrate_name: &'static str,         // data_class: PUBLIC
    pub tenant_namespace: &'static str,       // data_class: INTERNAL_ONLY
    pub workload_manifest_name: &'static str, // data_class: PUBLIC
    pub workload_manifest_count: usize,       // data_class: PUBLIC
    pub tenant_admission_policy_contract_name: &'static str, // data_class: PUBLIC
    pub requirements: Vec<Fd001TenantAutoscalingRequirement>, // data_class: INTERNAL_ONLY
    pub official_docs_required: bool,         // data_class: PUBLIC
    pub all_manifest_workloads_in_scope: bool, // data_class: PUBLIC
    pub horizontal_pod_autoscaler_required: bool, // data_class: PUBLIC
    pub autoscaling_v2_api_required: bool,    // data_class: PUBLIC
    pub min_replica_floor_required: bool,     // data_class: PUBLIC
    pub max_replica_ceiling_required: bool,   // data_class: PUBLIC
    pub cpu_resource_metric_required: bool,   // data_class: PUBLIC
    pub memory_resource_metric_required: bool, // data_class: PUBLIC
    pub metrics_pipeline_evidence_required: bool, // data_class: PUBLIC
    pub scale_up_behavior_policy_required: bool, // data_class: PUBLIC
    pub scale_down_behavior_policy_required: bool, // data_class: PUBLIC
    pub stabilization_window_required: bool,  // data_class: PUBLIC
    pub tenant_label_selector_required: bool, // data_class: PUBLIC
    pub scaling_audit_evidence_required: bool, // data_class: PUBLIC
    pub admission_policy_evidence_required: bool, // data_class: PUBLIC
    pub review_only_contract: bool,           // data_class: PUBLIC
    pub kubernetes_cluster_attached: bool,    // data_class: INTERNAL_ONLY
    pub metrics_server_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub custom_metrics_api_attached: bool,    // data_class: INTERNAL_ONLY
    pub horizontal_pod_autoscaler_applied: bool, // data_class: INTERNAL_ONLY
    pub autoscaling_controller_runtime_observed: bool, // data_class: INTERNAL_ONLY
    pub scale_event_runtime_observed: bool,   // data_class: INTERNAL_ONLY
    pub workload_runtime_deployed: bool,      // data_class: INTERNAL_ONLY
    pub cloud_substrate_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Fd001TenantAutoscalingError {
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
    MissingRequirementKind(Fd001TenantAutoscalingRequirementKind),
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

pub fn fd001_tenant_autoscaling_contract()
-> Result<Fd001TenantAutoscalingContract, Fd001TenantAutoscalingError> {
    let manifest = fd001_tenant_workload_manifest();
    validate_fd001_tenant_workload_manifest(&manifest)
        .map_err(Fd001TenantAutoscalingError::WorkloadManifest)?;
    let admission_policy = fd001_tenant_admission_policy_contract()
        .map_err(Fd001TenantAutoscalingError::TenantAdmissionPolicy)?;
    validate_fd001_tenant_admission_policy_contract(&admission_policy)
        .map_err(Fd001TenantAutoscalingError::TenantAdmissionPolicy)?;

    Ok(Fd001TenantAutoscalingContract {
        contract_name: CONTRACT_NAME,
        program_name: PROGRAM_NAME,
        substrate_name: SUBSTRATE_NAME,
        tenant_namespace: manifest.tenant_namespace,
        workload_manifest_name: manifest.manifest_name,
        workload_manifest_count: manifest.workloads.len(),
        tenant_admission_policy_contract_name: admission_policy.contract_name,
        requirements: autoscaling_requirements(),
        official_docs_required: true,
        all_manifest_workloads_in_scope: true,
        horizontal_pod_autoscaler_required: true,
        autoscaling_v2_api_required: true,
        min_replica_floor_required: true,
        max_replica_ceiling_required: true,
        cpu_resource_metric_required: true,
        memory_resource_metric_required: true,
        metrics_pipeline_evidence_required: true,
        scale_up_behavior_policy_required: true,
        scale_down_behavior_policy_required: true,
        stabilization_window_required: true,
        tenant_label_selector_required: true,
        scaling_audit_evidence_required: true,
        admission_policy_evidence_required: true,
        review_only_contract: true,
        kubernetes_cluster_attached: false,
        metrics_server_runtime_attached: false,
        custom_metrics_api_attached: false,
        horizontal_pod_autoscaler_applied: false,
        autoscaling_controller_runtime_observed: false,
        scale_event_runtime_observed: false,
        workload_runtime_deployed: false,
        cloud_substrate_runtime_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: SCHEMA_VERSION,
    })
}

pub fn validate_fd001_tenant_autoscaling_contract(
    contract: &Fd001TenantAutoscalingContract,
) -> Result<(), Fd001TenantAutoscalingError> {
    let manifest = fd001_tenant_workload_manifest();
    validate_fd001_tenant_workload_manifest(&manifest)
        .map_err(Fd001TenantAutoscalingError::WorkloadManifest)?;
    let admission_policy = fd001_tenant_admission_policy_contract()
        .map_err(Fd001TenantAutoscalingError::TenantAdmissionPolicy)?;
    validate_fd001_tenant_admission_policy_contract(&admission_policy)
        .map_err(Fd001TenantAutoscalingError::TenantAdmissionPolicy)?;

    validate_slug(
        contract.contract_name,
        Fd001TenantAutoscalingError::InvalidContractName,
    )?;
    if contract.program_name != PROGRAM_NAME {
        return Err(Fd001TenantAutoscalingError::InvalidProgramName);
    }
    if contract.substrate_name != SUBSTRATE_NAME {
        return Err(Fd001TenantAutoscalingError::InvalidSubstrateName);
    }
    validate_tenant_namespace(contract.tenant_namespace)?;
    if contract.workload_manifest_name != manifest.manifest_name {
        return Err(Fd001TenantAutoscalingError::InvalidManifestName);
    }
    if contract.workload_manifest_count != manifest.workloads.len() {
        return Err(Fd001TenantAutoscalingError::InvalidWorkloadCount);
    }
    if contract.tenant_admission_policy_contract_name != admission_policy.contract_name {
        return Err(Fd001TenantAutoscalingError::InvalidAdmissionPolicyContractName);
    }
    if contract.requirements.len() < MIN_REQUIREMENT_COUNT
        || contract.schema_version != SCHEMA_VERSION
    {
        return Err(Fd001TenantAutoscalingError::MissingRequirements);
    }
    validate_required_controls(contract)?;
    validate_nonclaims(contract)?;
    validate_requirements(contract)?;
    Ok(())
}

pub fn fd001_tenant_autoscaling_doc_urls(
    contract: &Fd001TenantAutoscalingContract,
) -> Vec<&'static str> {
    let mut docs = BTreeSet::new();
    for requirement in &contract.requirements {
        docs.insert(requirement.official_doc_url);
    }
    docs.into_iter().collect()
}

fn autoscaling_requirements() -> Vec<Fd001TenantAutoscalingRequirement> {
    vec![
        requirement(
            "horizontal-pod-autoscaler-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantAutoscalingRequirementKind::HorizontalPodAutoscalerRequired,
            "HorizontalPodAutoscaler",
            "horizontal-pod-autoscaler",
            HPA_DOC_URL,
        ),
        requirement(
            "autoscaling-v2-api-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantAutoscalingRequirementKind::AutoscalingV2ApiRequired,
            "AutoscalingV2Api",
            "autoscaling-v2-api",
            HPA_API_DOC_URL,
        ),
        requirement(
            "min-replica-floor-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantAutoscalingRequirementKind::MinReplicaFloorRequired,
            "ReplicaBounds",
            "min-replica-floor",
            HPA_API_DOC_URL,
        ),
        requirement(
            "max-replica-ceiling-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantAutoscalingRequirementKind::MaxReplicaCeilingRequired,
            "ReplicaBounds",
            "max-replica-ceiling",
            HPA_API_DOC_URL,
        ),
        requirement(
            "cpu-resource-metric-all-workloads",
            Fd001TenantWorkloadKind::AccountingJournal,
            Fd001TenantAutoscalingRequirementKind::CpuResourceMetricRequired,
            "ResourceMetric",
            "cpu-resource-metric",
            RESOURCE_MANAGEMENT_DOC_URL,
        ),
        requirement(
            "memory-resource-metric-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantAutoscalingRequirementKind::MemoryResourceMetricRequired,
            "ResourceMetric",
            "memory-resource-metric",
            RESOURCE_MANAGEMENT_DOC_URL,
        ),
        requirement(
            "metrics-pipeline-evidence-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantAutoscalingRequirementKind::MetricsPipelineEvidenceRequired,
            "MetricsPipeline",
            "metrics-pipeline-evidence",
            RESOURCE_METRICS_PIPELINE_DOC_URL,
        ),
        requirement(
            "scale-up-behavior-policy-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantAutoscalingRequirementKind::ScaleUpBehaviorPolicyRequired,
            "HorizontalPodAutoscalerBehavior",
            "scale-up-behavior-policy",
            HPA_DOC_URL,
        ),
        requirement(
            "scale-down-behavior-policy-all-workloads",
            Fd001TenantWorkloadKind::AccountingJournal,
            Fd001TenantAutoscalingRequirementKind::ScaleDownBehaviorPolicyRequired,
            "HorizontalPodAutoscalerBehavior",
            "scale-down-behavior-policy",
            HPA_DOC_URL,
        ),
        requirement(
            "stabilization-window-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantAutoscalingRequirementKind::StabilizationWindowRequired,
            "HorizontalPodAutoscalerBehavior",
            "stabilization-window",
            HPA_DOC_URL,
        ),
        requirement(
            "tenant-label-selector-autoscaling-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantAutoscalingRequirementKind::TenantLabelSelectorRequired,
            "LabelSelector",
            "tenant-label-selector-autoscaling",
            WORKLOAD_AUTOSCALING_DOC_URL,
        ),
        requirement(
            "scaling-audit-evidence-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantAutoscalingRequirementKind::ScalingAuditEvidenceRequired,
            "ScalingAuditEvidence",
            "scaling-audit-evidence",
            HPA_WALKTHROUGH_DOC_URL,
        ),
        requirement(
            "admission-policy-autoscaling-evidence-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantAutoscalingRequirementKind::AdmissionPolicyEvidenceRequired,
            "ValidatingAdmissionPolicy",
            "admission-policy-autoscaling-evidence",
            HPA_DOC_URL,
        ),
    ]
}

fn requirement(
    requirement_id: &'static str,
    workload_kind: Fd001TenantWorkloadKind,
    requirement_kind: Fd001TenantAutoscalingRequirementKind,
    kubernetes_resource_kind: &'static str,
    policy_suffix: &'static str,
    official_doc_url: &'static str,
) -> Fd001TenantAutoscalingRequirement {
    Fd001TenantAutoscalingRequirement {
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
        runtime_observation_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

fn policy_ref(policy_suffix: &'static str) -> &'static str {
    match policy_suffix {
        "horizontal-pod-autoscaler" => "policy/autoscaling/fd001/horizontal-pod-autoscaler",
        "autoscaling-v2-api" => "policy/autoscaling/fd001/autoscaling-v2-api",
        "min-replica-floor" => "policy/autoscaling/fd001/min-replica-floor",
        "max-replica-ceiling" => "policy/autoscaling/fd001/max-replica-ceiling",
        "cpu-resource-metric" => "policy/autoscaling/fd001/cpu-resource-metric",
        "memory-resource-metric" => "policy/autoscaling/fd001/memory-resource-metric",
        "metrics-pipeline-evidence" => "policy/autoscaling/fd001/metrics-pipeline-evidence",
        "scale-up-behavior-policy" => "policy/autoscaling/fd001/scale-up-behavior-policy",
        "scale-down-behavior-policy" => "policy/autoscaling/fd001/scale-down-behavior-policy",
        "stabilization-window" => "policy/autoscaling/fd001/stabilization-window",
        "tenant-label-selector-autoscaling" => {
            "policy/autoscaling/fd001/tenant-label-selector-autoscaling"
        }
        "scaling-audit-evidence" => "policy/autoscaling/fd001/scaling-audit-evidence",
        "admission-policy-autoscaling-evidence" => {
            "policy/autoscaling/fd001/admission-policy-autoscaling-evidence"
        }
        _ => "policy/autoscaling/fd001/invalid",
    }
}

fn validate_required_controls(
    contract: &Fd001TenantAutoscalingContract,
) -> Result<(), Fd001TenantAutoscalingError> {
    for (enabled, name) in [
        (contract.official_docs_required, "official_docs_required"),
        (
            contract.all_manifest_workloads_in_scope,
            "all_manifest_workloads_in_scope",
        ),
        (
            contract.horizontal_pod_autoscaler_required,
            "horizontal_pod_autoscaler_required",
        ),
        (
            contract.autoscaling_v2_api_required,
            "autoscaling_v2_api_required",
        ),
        (
            contract.min_replica_floor_required,
            "min_replica_floor_required",
        ),
        (
            contract.max_replica_ceiling_required,
            "max_replica_ceiling_required",
        ),
        (
            contract.cpu_resource_metric_required,
            "cpu_resource_metric_required",
        ),
        (
            contract.memory_resource_metric_required,
            "memory_resource_metric_required",
        ),
        (
            contract.metrics_pipeline_evidence_required,
            "metrics_pipeline_evidence_required",
        ),
        (
            contract.scale_up_behavior_policy_required,
            "scale_up_behavior_policy_required",
        ),
        (
            contract.scale_down_behavior_policy_required,
            "scale_down_behavior_policy_required",
        ),
        (
            contract.stabilization_window_required,
            "stabilization_window_required",
        ),
        (
            contract.tenant_label_selector_required,
            "tenant_label_selector_required",
        ),
        (
            contract.scaling_audit_evidence_required,
            "scaling_audit_evidence_required",
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
    contract: &Fd001TenantAutoscalingContract,
) -> Result<(), Fd001TenantAutoscalingError> {
    if contract.kubernetes_cluster_attached
        || contract.metrics_server_runtime_attached
        || contract.custom_metrics_api_attached
        || contract.horizontal_pod_autoscaler_applied
        || contract.autoscaling_controller_runtime_observed
        || contract.scale_event_runtime_observed
        || contract.workload_runtime_deployed
        || contract.cloud_substrate_runtime_attached
        || contract.runtime_audit_chain_emission_attached
    {
        return Err(Fd001TenantAutoscalingError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_requirements(
    contract: &Fd001TenantAutoscalingContract,
) -> Result<(), Fd001TenantAutoscalingError> {
    let mut seen_requirements = BTreeSet::new();
    let mut seen_workloads = BTreeSet::new();
    let mut seen_requirement_kinds = BTreeSet::new();
    for requirement in &contract.requirements {
        validate_requirement(requirement)?;
        if !seen_requirements.insert(requirement.requirement_id) {
            return Err(Fd001TenantAutoscalingError::DuplicateRequirement(
                requirement.requirement_id.to_owned(),
            ));
        }
        seen_workloads.insert(requirement.workload_kind);
        seen_requirement_kinds.insert(requirement.requirement_kind);
    }
    for workload_kind in required_workload_kinds() {
        if !seen_workloads.contains(&workload_kind) {
            return Err(Fd001TenantAutoscalingError::MissingWorkloadKind(
                workload_kind,
            ));
        }
    }
    for requirement_kind in required_requirement_kinds() {
        if !seen_requirement_kinds.contains(&requirement_kind) {
            return Err(Fd001TenantAutoscalingError::MissingRequirementKind(
                requirement_kind,
            ));
        }
    }
    Ok(())
}

fn validate_requirement(
    requirement: &Fd001TenantAutoscalingRequirement,
) -> Result<(), Fd001TenantAutoscalingError> {
    validate_slug(
        requirement.requirement_id,
        Fd001TenantAutoscalingError::InvalidRequirementId,
    )?;
    validate_kubernetes_resource_kind(requirement.kubernetes_resource_kind)?;
    validate_prefixed_ref(
        requirement.policy_ref,
        POLICY_REF_PREFIX,
        Fd001TenantAutoscalingError::InvalidPolicyRef,
    )?;
    validate_prefixed_ref(
        requirement.expected_evidence_ref,
        "evidence/autoscaling/fd001-tenant-rbac/",
        Fd001TenantAutoscalingError::InvalidExpectedEvidenceRef,
    )?;
    validate_doc_url(requirement.official_doc_url)?;
    validate_prefixed_ref(
        requirement.source_manifest_ref,
        "crates/tenant-rbac-tenant-workload-manifest/",
        Fd001TenantAutoscalingError::InvalidSourceManifestRef,
    )?;
    validate_prefixed_ref(
        requirement.source_admission_policy_ref,
        "crates/tenant-rbac-tenant-admission-policy/",
        Fd001TenantAutoscalingError::InvalidSourceAdmissionPolicyRef,
    )?;
    if !requirement.applies_to_all_manifest_workloads {
        return Err(Fd001TenantAutoscalingError::MissingRequiredControl(
            "requirement_applies_to_all_manifest_workloads",
        ));
    }
    if requirement.runtime_observation_attached || requirement.schema_version != SCHEMA_VERSION {
        return Err(Fd001TenantAutoscalingError::RuntimeAttachmentOverclaim);
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

fn required_requirement_kinds() -> [Fd001TenantAutoscalingRequirementKind; 13] {
    [
        Fd001TenantAutoscalingRequirementKind::HorizontalPodAutoscalerRequired,
        Fd001TenantAutoscalingRequirementKind::AutoscalingV2ApiRequired,
        Fd001TenantAutoscalingRequirementKind::MinReplicaFloorRequired,
        Fd001TenantAutoscalingRequirementKind::MaxReplicaCeilingRequired,
        Fd001TenantAutoscalingRequirementKind::CpuResourceMetricRequired,
        Fd001TenantAutoscalingRequirementKind::MemoryResourceMetricRequired,
        Fd001TenantAutoscalingRequirementKind::MetricsPipelineEvidenceRequired,
        Fd001TenantAutoscalingRequirementKind::ScaleUpBehaviorPolicyRequired,
        Fd001TenantAutoscalingRequirementKind::ScaleDownBehaviorPolicyRequired,
        Fd001TenantAutoscalingRequirementKind::StabilizationWindowRequired,
        Fd001TenantAutoscalingRequirementKind::TenantLabelSelectorRequired,
        Fd001TenantAutoscalingRequirementKind::ScalingAuditEvidenceRequired,
        Fd001TenantAutoscalingRequirementKind::AdmissionPolicyEvidenceRequired,
    ]
}

fn validate_kubernetes_resource_kind(value: &str) -> Result<(), Fd001TenantAutoscalingError> {
    if ![
        "HorizontalPodAutoscaler",
        "AutoscalingV2Api",
        "ReplicaBounds",
        "ResourceMetric",
        "MetricsPipeline",
        "HorizontalPodAutoscalerBehavior",
        "LabelSelector",
        "ScalingAuditEvidence",
        "ValidatingAdmissionPolicy",
    ]
    .contains(&value)
    {
        return Err(Fd001TenantAutoscalingError::InvalidKubernetesResourceKind);
    }
    Ok(())
}

fn validate_doc_url(url: &str) -> Result<(), Fd001TenantAutoscalingError> {
    if has_unsafe_ref_text(url)
        || ![
            HPA_DOC_URL,
            HPA_WALKTHROUGH_DOC_URL,
            RESOURCE_METRICS_PIPELINE_DOC_URL,
            WORKLOAD_AUTOSCALING_DOC_URL,
            HPA_API_DOC_URL,
            RESOURCE_MANAGEMENT_DOC_URL,
        ]
        .contains(&url)
    {
        return Err(Fd001TenantAutoscalingError::InvalidOfficialDocUrl);
    }
    Ok(())
}

fn validate_slug(
    value: &str,
    error: Fd001TenantAutoscalingError,
) -> Result<(), Fd001TenantAutoscalingError> {
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

fn validate_tenant_namespace(value: &str) -> Result<(), Fd001TenantAutoscalingError> {
    validate_slug(value, Fd001TenantAutoscalingError::InvalidTenantNamespace)?;
    if value != TENANT_NAMESPACE {
        return Err(Fd001TenantAutoscalingError::InvalidTenantNamespace);
    }
    Ok(())
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: Fd001TenantAutoscalingError,
) -> Result<(), Fd001TenantAutoscalingError> {
    if value.len() <= prefix.len() || !value.starts_with(prefix) || has_unsafe_ref_text(value) {
        return Err(error);
    }
    Ok(())
}

fn require_control(value: bool, control: &'static str) -> Result<(), Fd001TenantAutoscalingError> {
    if value {
        Ok(())
    } else {
        Err(Fd001TenantAutoscalingError::MissingRequiredControl(control))
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
