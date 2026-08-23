//! FD-001 tenant availability, scheduling, and disruption contract for later Oyatie Cloud dogfooding.
//!
//! This review-only crate defines workload resilience guardrails that must exist
//! before FD-001 Tenant RBAC, HR, Payroll, and Accounting workloads can be
//! promoted as production tenant workloads on the future Oyatie Cloud substrate.
//! It binds the FD-001 tenant-workload manifest and tenant admission policy to
//! Kubernetes PodDisruptionBudget, multi-replica availability, topology-spread,
//! pod anti-affinity, node/zone topology-label evidence, rolling-update
//! availability, progress-deadline, readiness-probe, tenant-label, disruption
//! audit, and admission-policy evidence. It does not attach a Kubernetes cluster,
//! apply PDB or scheduling objects, observe scheduler/rollout/probe runtime,
//! deploy workloads, attach a cloud substrate runtime, or emit runtime
//! audit-chain events.
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
const CONTRACT_NAME: &str = "fd001-tenant-rbac-tenant-availability-contract";
const PROGRAM_NAME: &str = "fd-001-tenant-rbac-generic";
const SUBSTRATE_NAME: &str = "oyatie-cloud";
const TENANT_NAMESPACE: &str = "oyatie-fd001-tenant-rbac-dev";
const SOURCE_MANIFEST_REF: &str =
    "crates/tenant-rbac-tenant-workload-manifest/src/lib.rs::fd001_tenant_workload_manifest";
const SOURCE_ADMISSION_POLICY_REF: &str =
    "crates/tenant-rbac-tenant-admission-policy/src/lib.rs::fd001_tenant_admission_policy_contract";
const POLICY_REF_PREFIX: &str = "policy/availability/fd001/";
const EXPECTED_EVIDENCE_REF: &str =
    "evidence/availability/fd001-tenant-rbac/availability-contract-review.jsonl";

const DISRUPTIONS_DOC_URL: &str = "https://kubernetes.io/docs/concepts/workloads/pods/disruptions/";
const CONFIGURE_PDB_DOC_URL: &str =
    "https://kubernetes.io/docs/tasks/run-application/configure-pdb/";
const PDB_API_DOC_URL: &str =
    "https://kubernetes.io/docs/reference/kubernetes-api/policy/pod-disruption-budget-v1/";
const TOPOLOGY_SPREAD_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/scheduling-eviction/topology-spread-constraints/";
const ASSIGN_POD_NODE_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/scheduling-eviction/assign-pod-node/";
const DEPLOYMENT_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/workloads/controllers/deployment/";
const ROLLING_UPDATE_DOC_URL: &str =
    "https://kubernetes.io/docs/tasks/run-application/update-deployment-rolling/";
const PROBES_DOC_URL: &str = "https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Fd001TenantAvailabilityRequirementKind {
    PodDisruptionBudgetRequired,
    MinimumAvailableBudgetRequired,
    MultiReplicaWorkloadRequired,
    ZoneTopologySpreadRequired,
    HostnameTopologySpreadRequired,
    PodAntiAffinityRequired,
    NodeTopologyLabelEvidenceRequired,
    RollingUpdateAvailabilityRequired,
    ProgressDeadlineRequired,
    ReadinessProbeEvidenceRequired,
    TenantLabelSelectorRequired,
    DisruptionAuditEvidenceRequired,
    AdmissionPolicyEvidenceRequired,
}

impl Fd001TenantAvailabilityRequirementKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PodDisruptionBudgetRequired => "pod_disruption_budget_required",
            Self::MinimumAvailableBudgetRequired => "minimum_available_budget_required",
            Self::MultiReplicaWorkloadRequired => "multi_replica_workload_required",
            Self::ZoneTopologySpreadRequired => "zone_topology_spread_required",
            Self::HostnameTopologySpreadRequired => "hostname_topology_spread_required",
            Self::PodAntiAffinityRequired => "pod_anti_affinity_required",
            Self::NodeTopologyLabelEvidenceRequired => "node_topology_label_evidence_required",
            Self::RollingUpdateAvailabilityRequired => "rolling_update_availability_required",
            Self::ProgressDeadlineRequired => "progress_deadline_required",
            Self::ReadinessProbeEvidenceRequired => "readiness_probe_evidence_required",
            Self::TenantLabelSelectorRequired => "tenant_label_selector_required",
            Self::DisruptionAuditEvidenceRequired => "disruption_audit_evidence_required",
            Self::AdmissionPolicyEvidenceRequired => "admission_policy_evidence_required",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fd001TenantAvailabilityRequirement {
    pub requirement_id: &'static str,           // data_class: PUBLIC
    pub workload_kind: Fd001TenantWorkloadKind, // data_class: PUBLIC
    pub requirement_kind: Fd001TenantAvailabilityRequirementKind, // data_class: PUBLIC
    pub kubernetes_resource_kind: &'static str, // data_class: PUBLIC
    pub topology_key: &'static str,             // data_class: PUBLIC
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
pub struct Fd001TenantAvailabilityContract {
    pub contract_name: &'static str,          // data_class: PUBLIC
    pub program_name: &'static str,           // data_class: PUBLIC
    pub substrate_name: &'static str,         // data_class: PUBLIC
    pub tenant_namespace: &'static str,       // data_class: INTERNAL_ONLY
    pub workload_manifest_name: &'static str, // data_class: PUBLIC
    pub workload_manifest_count: usize,       // data_class: PUBLIC
    pub tenant_admission_policy_contract_name: &'static str, // data_class: PUBLIC
    pub requirements: Vec<Fd001TenantAvailabilityRequirement>, // data_class: INTERNAL_ONLY
    pub official_docs_required: bool,         // data_class: PUBLIC
    pub all_manifest_workloads_in_scope: bool, // data_class: PUBLIC
    pub pod_disruption_budget_required: bool, // data_class: PUBLIC
    pub minimum_available_budget_required: bool, // data_class: PUBLIC
    pub multi_replica_workload_required: bool, // data_class: PUBLIC
    pub zone_topology_spread_required: bool,  // data_class: PUBLIC
    pub hostname_topology_spread_required: bool, // data_class: PUBLIC
    pub pod_anti_affinity_required: bool,     // data_class: PUBLIC
    pub node_topology_label_evidence_required: bool, // data_class: PUBLIC
    pub rolling_update_availability_required: bool, // data_class: PUBLIC
    pub progress_deadline_required: bool,     // data_class: PUBLIC
    pub readiness_probe_evidence_required: bool, // data_class: PUBLIC
    pub tenant_label_selector_required: bool, // data_class: PUBLIC
    pub disruption_audit_evidence_required: bool, // data_class: PUBLIC
    pub admission_policy_evidence_required: bool, // data_class: PUBLIC
    pub review_only_contract: bool,           // data_class: PUBLIC
    pub kubernetes_cluster_attached: bool,    // data_class: INTERNAL_ONLY
    pub pod_disruption_budget_applied: bool,  // data_class: INTERNAL_ONLY
    pub topology_spread_applied: bool,        // data_class: INTERNAL_ONLY
    pub pod_anti_affinity_applied: bool,      // data_class: INTERNAL_ONLY
    pub scheduler_runtime_observed: bool,     // data_class: INTERNAL_ONLY
    pub rolling_update_runtime_observed: bool, // data_class: INTERNAL_ONLY
    pub readiness_probe_runtime_observed: bool, // data_class: INTERNAL_ONLY
    pub workload_runtime_deployed: bool,      // data_class: INTERNAL_ONLY
    pub cloud_substrate_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Fd001TenantAvailabilityError {
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
    MissingRequirementKind(Fd001TenantAvailabilityRequirementKind),
    InvalidRequirementId,
    InvalidKubernetesResourceKind,
    InvalidTopologyKey,
    InvalidPolicyRef,
    InvalidExpectedEvidenceRef,
    InvalidOfficialDocUrl,
    InvalidSourceManifestRef,
    InvalidSourceAdmissionPolicyRef,
    MissingRequiredControl(&'static str),
    RuntimeAttachmentOverclaim,
}

pub fn fd001_tenant_availability_contract()
-> Result<Fd001TenantAvailabilityContract, Fd001TenantAvailabilityError> {
    let manifest = fd001_tenant_workload_manifest();
    validate_fd001_tenant_workload_manifest(&manifest)
        .map_err(Fd001TenantAvailabilityError::WorkloadManifest)?;
    let admission_policy = fd001_tenant_admission_policy_contract()
        .map_err(Fd001TenantAvailabilityError::TenantAdmissionPolicy)?;
    validate_fd001_tenant_admission_policy_contract(&admission_policy)
        .map_err(Fd001TenantAvailabilityError::TenantAdmissionPolicy)?;

    Ok(Fd001TenantAvailabilityContract {
        contract_name: CONTRACT_NAME,
        program_name: PROGRAM_NAME,
        substrate_name: SUBSTRATE_NAME,
        tenant_namespace: manifest.tenant_namespace,
        workload_manifest_name: manifest.manifest_name,
        workload_manifest_count: manifest.workloads.len(),
        tenant_admission_policy_contract_name: admission_policy.contract_name,
        requirements: availability_requirements(),
        official_docs_required: true,
        all_manifest_workloads_in_scope: true,
        pod_disruption_budget_required: true,
        minimum_available_budget_required: true,
        multi_replica_workload_required: true,
        zone_topology_spread_required: true,
        hostname_topology_spread_required: true,
        pod_anti_affinity_required: true,
        node_topology_label_evidence_required: true,
        rolling_update_availability_required: true,
        progress_deadline_required: true,
        readiness_probe_evidence_required: true,
        tenant_label_selector_required: true,
        disruption_audit_evidence_required: true,
        admission_policy_evidence_required: true,
        review_only_contract: true,
        kubernetes_cluster_attached: false,
        pod_disruption_budget_applied: false,
        topology_spread_applied: false,
        pod_anti_affinity_applied: false,
        scheduler_runtime_observed: false,
        rolling_update_runtime_observed: false,
        readiness_probe_runtime_observed: false,
        workload_runtime_deployed: false,
        cloud_substrate_runtime_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: SCHEMA_VERSION,
    })
}

pub fn validate_fd001_tenant_availability_contract(
    contract: &Fd001TenantAvailabilityContract,
) -> Result<(), Fd001TenantAvailabilityError> {
    let manifest = fd001_tenant_workload_manifest();
    validate_fd001_tenant_workload_manifest(&manifest)
        .map_err(Fd001TenantAvailabilityError::WorkloadManifest)?;
    let admission_policy = fd001_tenant_admission_policy_contract()
        .map_err(Fd001TenantAvailabilityError::TenantAdmissionPolicy)?;
    validate_fd001_tenant_admission_policy_contract(&admission_policy)
        .map_err(Fd001TenantAvailabilityError::TenantAdmissionPolicy)?;

    validate_slug(
        contract.contract_name,
        Fd001TenantAvailabilityError::InvalidContractName,
    )?;
    if contract.program_name != PROGRAM_NAME {
        return Err(Fd001TenantAvailabilityError::InvalidProgramName);
    }
    if contract.substrate_name != SUBSTRATE_NAME {
        return Err(Fd001TenantAvailabilityError::InvalidSubstrateName);
    }
    validate_tenant_namespace(contract.tenant_namespace)?;
    if contract.workload_manifest_name != manifest.manifest_name {
        return Err(Fd001TenantAvailabilityError::InvalidManifestName);
    }
    if contract.workload_manifest_count != manifest.workloads.len() {
        return Err(Fd001TenantAvailabilityError::InvalidWorkloadCount);
    }
    if contract.tenant_admission_policy_contract_name != admission_policy.contract_name {
        return Err(Fd001TenantAvailabilityError::InvalidAdmissionPolicyContractName);
    }
    if contract.requirements.len() < MIN_REQUIREMENT_COUNT
        || contract.schema_version != SCHEMA_VERSION
    {
        return Err(Fd001TenantAvailabilityError::MissingRequirements);
    }
    validate_required_controls(contract)?;
    validate_nonclaims(contract)?;
    validate_requirements(contract)?;
    Ok(())
}

pub fn fd001_tenant_availability_doc_urls(
    contract: &Fd001TenantAvailabilityContract,
) -> Vec<&'static str> {
    let mut docs = BTreeSet::new();
    for requirement in &contract.requirements {
        docs.insert(requirement.official_doc_url);
    }
    docs.into_iter().collect()
}

fn availability_requirements() -> Vec<Fd001TenantAvailabilityRequirement> {
    vec![
        requirement(
            "pod-disruption-budget-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantAvailabilityRequirementKind::PodDisruptionBudgetRequired,
            "PodDisruptionBudget",
            "none",
            "pod-disruption-budget",
            CONFIGURE_PDB_DOC_URL,
        ),
        requirement(
            "minimum-available-budget-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantAvailabilityRequirementKind::MinimumAvailableBudgetRequired,
            "PodDisruptionBudget",
            "none",
            "minimum-available-budget",
            PDB_API_DOC_URL,
        ),
        requirement(
            "multi-replica-workload-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantAvailabilityRequirementKind::MultiReplicaWorkloadRequired,
            "Deployment",
            "none",
            "multi-replica-workload",
            DEPLOYMENT_DOC_URL,
        ),
        requirement(
            "zone-topology-spread-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantAvailabilityRequirementKind::ZoneTopologySpreadRequired,
            "TopologySpreadConstraint",
            "topology.kubernetes.io/zone",
            "zone-topology-spread",
            TOPOLOGY_SPREAD_DOC_URL,
        ),
        requirement(
            "hostname-topology-spread-all-workloads",
            Fd001TenantWorkloadKind::AccountingJournal,
            Fd001TenantAvailabilityRequirementKind::HostnameTopologySpreadRequired,
            "TopologySpreadConstraint",
            "kubernetes.io/hostname",
            "hostname-topology-spread",
            TOPOLOGY_SPREAD_DOC_URL,
        ),
        requirement(
            "pod-anti-affinity-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantAvailabilityRequirementKind::PodAntiAffinityRequired,
            "PodAntiAffinity",
            "kubernetes.io/hostname",
            "pod-anti-affinity",
            ASSIGN_POD_NODE_DOC_URL,
        ),
        requirement(
            "node-topology-label-evidence-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantAvailabilityRequirementKind::NodeTopologyLabelEvidenceRequired,
            "NodeLabel",
            "topology.kubernetes.io/zone",
            "node-topology-label-evidence",
            ASSIGN_POD_NODE_DOC_URL,
        ),
        requirement(
            "rolling-update-availability-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantAvailabilityRequirementKind::RollingUpdateAvailabilityRequired,
            "DeploymentStrategy",
            "none",
            "rolling-update-availability",
            ROLLING_UPDATE_DOC_URL,
        ),
        requirement(
            "progress-deadline-all-workloads",
            Fd001TenantWorkloadKind::AccountingJournal,
            Fd001TenantAvailabilityRequirementKind::ProgressDeadlineRequired,
            "Deployment",
            "none",
            "progress-deadline",
            DEPLOYMENT_DOC_URL,
        ),
        requirement(
            "readiness-probe-evidence-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantAvailabilityRequirementKind::ReadinessProbeEvidenceRequired,
            "ReadinessProbe",
            "none",
            "readiness-probe-evidence",
            PROBES_DOC_URL,
        ),
        requirement(
            "tenant-label-selector-availability-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantAvailabilityRequirementKind::TenantLabelSelectorRequired,
            "LabelSelector",
            "none",
            "tenant-label-selector-availability",
            DISRUPTIONS_DOC_URL,
        ),
        requirement(
            "disruption-audit-evidence-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantAvailabilityRequirementKind::DisruptionAuditEvidenceRequired,
            "EvictionAuditEvidence",
            "none",
            "disruption-audit-evidence",
            DISRUPTIONS_DOC_URL,
        ),
        requirement(
            "admission-policy-availability-evidence-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantAvailabilityRequirementKind::AdmissionPolicyEvidenceRequired,
            "ValidatingAdmissionPolicy",
            "none",
            "admission-policy-availability-evidence",
            DEPLOYMENT_DOC_URL,
        ),
    ]
}

fn requirement(
    requirement_id: &'static str,
    workload_kind: Fd001TenantWorkloadKind,
    requirement_kind: Fd001TenantAvailabilityRequirementKind,
    kubernetes_resource_kind: &'static str,
    topology_key: &'static str,
    policy_suffix: &'static str,
    official_doc_url: &'static str,
) -> Fd001TenantAvailabilityRequirement {
    Fd001TenantAvailabilityRequirement {
        requirement_id,
        workload_kind,
        requirement_kind,
        kubernetes_resource_kind,
        topology_key,
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
        "pod-disruption-budget" => "policy/availability/fd001/pod-disruption-budget",
        "minimum-available-budget" => "policy/availability/fd001/minimum-available-budget",
        "multi-replica-workload" => "policy/availability/fd001/multi-replica-workload",
        "zone-topology-spread" => "policy/availability/fd001/zone-topology-spread",
        "hostname-topology-spread" => "policy/availability/fd001/hostname-topology-spread",
        "pod-anti-affinity" => "policy/availability/fd001/pod-anti-affinity",
        "node-topology-label-evidence" => "policy/availability/fd001/node-topology-label-evidence",
        "rolling-update-availability" => "policy/availability/fd001/rolling-update-availability",
        "progress-deadline" => "policy/availability/fd001/progress-deadline",
        "readiness-probe-evidence" => "policy/availability/fd001/readiness-probe-evidence",
        "tenant-label-selector-availability" => {
            "policy/availability/fd001/tenant-label-selector-availability"
        }
        "disruption-audit-evidence" => "policy/availability/fd001/disruption-audit-evidence",
        "admission-policy-availability-evidence" => {
            "policy/availability/fd001/admission-policy-availability-evidence"
        }
        _ => "policy/availability/fd001/invalid",
    }
}

fn validate_required_controls(
    contract: &Fd001TenantAvailabilityContract,
) -> Result<(), Fd001TenantAvailabilityError> {
    for (enabled, name) in [
        (contract.official_docs_required, "official_docs_required"),
        (
            contract.all_manifest_workloads_in_scope,
            "all_manifest_workloads_in_scope",
        ),
        (
            contract.pod_disruption_budget_required,
            "pod_disruption_budget_required",
        ),
        (
            contract.minimum_available_budget_required,
            "minimum_available_budget_required",
        ),
        (
            contract.multi_replica_workload_required,
            "multi_replica_workload_required",
        ),
        (
            contract.zone_topology_spread_required,
            "zone_topology_spread_required",
        ),
        (
            contract.hostname_topology_spread_required,
            "hostname_topology_spread_required",
        ),
        (
            contract.pod_anti_affinity_required,
            "pod_anti_affinity_required",
        ),
        (
            contract.node_topology_label_evidence_required,
            "node_topology_label_evidence_required",
        ),
        (
            contract.rolling_update_availability_required,
            "rolling_update_availability_required",
        ),
        (
            contract.progress_deadline_required,
            "progress_deadline_required",
        ),
        (
            contract.readiness_probe_evidence_required,
            "readiness_probe_evidence_required",
        ),
        (
            contract.tenant_label_selector_required,
            "tenant_label_selector_required",
        ),
        (
            contract.disruption_audit_evidence_required,
            "disruption_audit_evidence_required",
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
    contract: &Fd001TenantAvailabilityContract,
) -> Result<(), Fd001TenantAvailabilityError> {
    if contract.kubernetes_cluster_attached
        || contract.pod_disruption_budget_applied
        || contract.topology_spread_applied
        || contract.pod_anti_affinity_applied
        || contract.scheduler_runtime_observed
        || contract.rolling_update_runtime_observed
        || contract.readiness_probe_runtime_observed
        || contract.workload_runtime_deployed
        || contract.cloud_substrate_runtime_attached
        || contract.runtime_audit_chain_emission_attached
    {
        return Err(Fd001TenantAvailabilityError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_requirements(
    contract: &Fd001TenantAvailabilityContract,
) -> Result<(), Fd001TenantAvailabilityError> {
    let mut seen_requirements = BTreeSet::new();
    let mut seen_workloads = BTreeSet::new();
    let mut seen_requirement_kinds = BTreeSet::new();
    for requirement in &contract.requirements {
        validate_requirement(requirement)?;
        if !seen_requirements.insert(requirement.requirement_id) {
            return Err(Fd001TenantAvailabilityError::DuplicateRequirement(
                requirement.requirement_id.to_owned(),
            ));
        }
        seen_workloads.insert(requirement.workload_kind);
        seen_requirement_kinds.insert(requirement.requirement_kind);
    }
    for workload_kind in required_workload_kinds() {
        if !seen_workloads.contains(&workload_kind) {
            return Err(Fd001TenantAvailabilityError::MissingWorkloadKind(
                workload_kind,
            ));
        }
    }
    for requirement_kind in required_requirement_kinds() {
        if !seen_requirement_kinds.contains(&requirement_kind) {
            return Err(Fd001TenantAvailabilityError::MissingRequirementKind(
                requirement_kind,
            ));
        }
    }
    Ok(())
}

fn validate_requirement(
    requirement: &Fd001TenantAvailabilityRequirement,
) -> Result<(), Fd001TenantAvailabilityError> {
    validate_slug(
        requirement.requirement_id,
        Fd001TenantAvailabilityError::InvalidRequirementId,
    )?;
    validate_kubernetes_resource_kind(requirement.kubernetes_resource_kind)?;
    validate_topology_key(requirement.topology_key)?;
    validate_prefixed_ref(
        requirement.policy_ref,
        POLICY_REF_PREFIX,
        Fd001TenantAvailabilityError::InvalidPolicyRef,
    )?;
    validate_prefixed_ref(
        requirement.expected_evidence_ref,
        "evidence/availability/fd001-tenant-rbac/",
        Fd001TenantAvailabilityError::InvalidExpectedEvidenceRef,
    )?;
    validate_doc_url(requirement.official_doc_url)?;
    validate_prefixed_ref(
        requirement.source_manifest_ref,
        "crates/tenant-rbac-tenant-workload-manifest/",
        Fd001TenantAvailabilityError::InvalidSourceManifestRef,
    )?;
    validate_prefixed_ref(
        requirement.source_admission_policy_ref,
        "crates/tenant-rbac-tenant-admission-policy/",
        Fd001TenantAvailabilityError::InvalidSourceAdmissionPolicyRef,
    )?;
    if !requirement.applies_to_all_manifest_workloads {
        return Err(Fd001TenantAvailabilityError::MissingRequiredControl(
            "requirement_applies_to_all_manifest_workloads",
        ));
    }
    if requirement.runtime_observation_attached || requirement.schema_version != SCHEMA_VERSION {
        return Err(Fd001TenantAvailabilityError::RuntimeAttachmentOverclaim);
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

fn required_requirement_kinds() -> [Fd001TenantAvailabilityRequirementKind; 13] {
    [
        Fd001TenantAvailabilityRequirementKind::PodDisruptionBudgetRequired,
        Fd001TenantAvailabilityRequirementKind::MinimumAvailableBudgetRequired,
        Fd001TenantAvailabilityRequirementKind::MultiReplicaWorkloadRequired,
        Fd001TenantAvailabilityRequirementKind::ZoneTopologySpreadRequired,
        Fd001TenantAvailabilityRequirementKind::HostnameTopologySpreadRequired,
        Fd001TenantAvailabilityRequirementKind::PodAntiAffinityRequired,
        Fd001TenantAvailabilityRequirementKind::NodeTopologyLabelEvidenceRequired,
        Fd001TenantAvailabilityRequirementKind::RollingUpdateAvailabilityRequired,
        Fd001TenantAvailabilityRequirementKind::ProgressDeadlineRequired,
        Fd001TenantAvailabilityRequirementKind::ReadinessProbeEvidenceRequired,
        Fd001TenantAvailabilityRequirementKind::TenantLabelSelectorRequired,
        Fd001TenantAvailabilityRequirementKind::DisruptionAuditEvidenceRequired,
        Fd001TenantAvailabilityRequirementKind::AdmissionPolicyEvidenceRequired,
    ]
}

fn validate_kubernetes_resource_kind(value: &str) -> Result<(), Fd001TenantAvailabilityError> {
    if ![
        "PodDisruptionBudget",
        "Deployment",
        "TopologySpreadConstraint",
        "PodAntiAffinity",
        "NodeLabel",
        "DeploymentStrategy",
        "ReadinessProbe",
        "LabelSelector",
        "EvictionAuditEvidence",
        "ValidatingAdmissionPolicy",
    ]
    .contains(&value)
    {
        return Err(Fd001TenantAvailabilityError::InvalidKubernetesResourceKind);
    }
    Ok(())
}

fn validate_topology_key(value: &str) -> Result<(), Fd001TenantAvailabilityError> {
    if ![
        "none",
        "topology.kubernetes.io/zone",
        "kubernetes.io/hostname",
    ]
    .contains(&value)
    {
        return Err(Fd001TenantAvailabilityError::InvalidTopologyKey);
    }
    Ok(())
}

fn validate_doc_url(url: &str) -> Result<(), Fd001TenantAvailabilityError> {
    if has_unsafe_ref_text(url)
        || ![
            DISRUPTIONS_DOC_URL,
            CONFIGURE_PDB_DOC_URL,
            PDB_API_DOC_URL,
            TOPOLOGY_SPREAD_DOC_URL,
            ASSIGN_POD_NODE_DOC_URL,
            DEPLOYMENT_DOC_URL,
            ROLLING_UPDATE_DOC_URL,
            PROBES_DOC_URL,
        ]
        .contains(&url)
    {
        return Err(Fd001TenantAvailabilityError::InvalidOfficialDocUrl);
    }
    Ok(())
}

fn validate_slug(
    value: &str,
    error: Fd001TenantAvailabilityError,
) -> Result<(), Fd001TenantAvailabilityError> {
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

fn validate_tenant_namespace(value: &str) -> Result<(), Fd001TenantAvailabilityError> {
    validate_slug(value, Fd001TenantAvailabilityError::InvalidTenantNamespace)?;
    if value != TENANT_NAMESPACE {
        return Err(Fd001TenantAvailabilityError::InvalidTenantNamespace);
    }
    Ok(())
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: Fd001TenantAvailabilityError,
) -> Result<(), Fd001TenantAvailabilityError> {
    if value.len() <= prefix.len() || !value.starts_with(prefix) || has_unsafe_ref_text(value) {
        return Err(error);
    }
    Ok(())
}

fn require_control(value: bool, control: &'static str) -> Result<(), Fd001TenantAvailabilityError> {
    if value {
        Ok(())
    } else {
        Err(Fd001TenantAvailabilityError::MissingRequiredControl(
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
