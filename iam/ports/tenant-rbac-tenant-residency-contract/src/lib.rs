//! FD-001 tenant residency placement contract for later Oyatie Cloud dogfooding.
//!
//! This review-only crate defines data-perimeter placement guardrails that must
//! exist before FD-001 Tenant RBAC, HR, Payroll, and Accounting workloads
//! can be promoted as production tenant workloads on the future Oyatie Cloud
//! substrate. It binds the FD-001 tenant-workload manifest and tenant admission
//! policy to tenant residency labels, namespace labels, node affinity, topology
//! constraints, storage residency policy refs, telemetry residency policy refs,
//! audit residency policy refs, cross-region egress policy refs, tenant-model
//! jurisdiction refs, cell-placement refs, admission-policy evidence,
//! workload-manifest evidence, and residency audit evidence. It does not attach
//! a Kubernetes cluster, create a namespace, apply node affinity, observe the
//! scheduler, attach storage or telemetry residency runtimes, observe
//! cross-region egress, deploy workloads, attach a cloud substrate runtime, or
//! emit runtime audit-chain events.
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
const CONTRACT_NAME: &str = "fd001-tenant-rbac-tenant-residency-contract";
const PROGRAM_NAME: &str = "fd-001-tenant-rbac-generic";
const SUBSTRATE_NAME: &str = "oyatie-cloud";
const TENANT_NAMESPACE: &str = "oyatie-fd001-tenant-rbac-dev";
const TENANT_CELL_ID: &str = "cell-us-east-001";
const RESIDENCY_REGION: &str = "us-east-1";
const SOURCE_MANIFEST_REF: &str =
    "crates/tenant-rbac-tenant-workload-manifest/src/lib.rs::fd001_tenant_workload_manifest";
const SOURCE_ADMISSION_POLICY_REF: &str = "crates/tenant-rbac-tenant-admission-policy/src/lib.rs::fd001_tenant_admission_policy_contract";
const SOURCE_TENANT_MODEL_REF: &str = "specs/tenant-model.json#jurisdiction";
const POLICY_REF_PREFIX: &str = "policy/residency/fd001/";
const EXPECTED_EVIDENCE_REF: &str =
    "evidence/residency/fd001-tenant-rbac/residency-contract-review.jsonl";

const KUBERNETES_LABELS_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/overview/working-with-objects/labels/";
const KUBERNETES_NODE_AFFINITY_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/scheduling-eviction/assign-pod-node/";
const KUBERNETES_TOPOLOGY_SPREAD_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/scheduling-eviction/topology-spread-constraints/";
const KUBERNETES_NETWORK_POLICY_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/services-networking/network-policies/";
const OTEL_RESOURCE_SEMCONV_DOC_URL: &str = "https://opentelemetry.io/docs/specs/semconv/resource/";
const AWS_DATA_PERIMETER_DOC_URL: &str =
    "https://aws.amazon.com/blogs/security/establishing-a-data-perimeter-on-aws/";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Fd001TenantResidencyRequirementKind {
    TenantResidencyRegionLabelRequired,
    NamespaceResidencyLabelRequired,
    WorkloadNodeAffinityRequired,
    TopologyRegionConstraintRequired,
    StorageResidencyPolicyRefRequired,
    TelemetryResidencyPolicyRefRequired,
    AuditResidencyPolicyRefRequired,
    CrossRegionEgressPolicyRefRequired,
    TenantModelJurisdictionRefRequired,
    CellPlacementResidencyRefRequired,
    AdmissionPolicyEvidenceRequired,
    WorkloadManifestEvidenceRequired,
    ResidencyAuditEvidenceRequired,
}

impl Fd001TenantResidencyRequirementKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TenantResidencyRegionLabelRequired => "tenant_residency_region_label_required",
            Self::NamespaceResidencyLabelRequired => "namespace_residency_label_required",
            Self::WorkloadNodeAffinityRequired => "workload_node_affinity_required",
            Self::TopologyRegionConstraintRequired => "topology_region_constraint_required",
            Self::StorageResidencyPolicyRefRequired => "storage_residency_policy_ref_required",
            Self::TelemetryResidencyPolicyRefRequired => "telemetry_residency_policy_ref_required",
            Self::AuditResidencyPolicyRefRequired => "audit_residency_policy_ref_required",
            Self::CrossRegionEgressPolicyRefRequired => "cross_region_egress_policy_ref_required",
            Self::TenantModelJurisdictionRefRequired => "tenant_model_jurisdiction_ref_required",
            Self::CellPlacementResidencyRefRequired => "cell_placement_residency_ref_required",
            Self::AdmissionPolicyEvidenceRequired => "admission_policy_evidence_required",
            Self::WorkloadManifestEvidenceRequired => "workload_manifest_evidence_required",
            Self::ResidencyAuditEvidenceRequired => "residency_audit_evidence_required",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fd001TenantResidencyRequirement {
    pub requirement_id: &'static str,           // data_class: PUBLIC
    pub workload_kind: Fd001TenantWorkloadKind, // data_class: PUBLIC
    pub requirement_kind: Fd001TenantResidencyRequirementKind, // data_class: PUBLIC
    pub kubernetes_resource_kind: &'static str, // data_class: PUBLIC
    pub policy_ref: &'static str,               // data_class: INTERNAL_ONLY
    pub expected_evidence_ref: &'static str,    // data_class: INTERNAL_ONLY
    pub official_doc_url: &'static str,         // data_class: PUBLIC
    pub source_manifest_ref: &'static str,      // data_class: INTERNAL_ONLY
    pub source_admission_policy_ref: &'static str, // data_class: INTERNAL_ONLY
    pub source_tenant_model_ref: &'static str,  // data_class: INTERNAL_ONLY
    pub applies_to_all_manifest_workloads: bool, // data_class: PUBLIC
    pub runtime_observation_attached: bool,     // data_class: INTERNAL_ONLY
    pub schema_version: u32,                    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fd001TenantResidencyContract {
    pub contract_name: &'static str,          // data_class: PUBLIC
    pub program_name: &'static str,           // data_class: PUBLIC
    pub substrate_name: &'static str,         // data_class: PUBLIC
    pub tenant_namespace: &'static str,       // data_class: INTERNAL_ONLY
    pub tenant_cell_id: &'static str,         // data_class: INTERNAL_ONLY
    pub residency_region: &'static str,       // data_class: INTERNAL_ONLY
    pub workload_manifest_name: &'static str, // data_class: PUBLIC
    pub workload_manifest_count: usize,       // data_class: PUBLIC
    pub tenant_admission_policy_contract_name: &'static str, // data_class: PUBLIC
    pub tenant_model_ref: &'static str,       // data_class: INTERNAL_ONLY
    pub requirements: Vec<Fd001TenantResidencyRequirement>, // data_class: INTERNAL_ONLY
    pub official_docs_required: bool,         // data_class: PUBLIC
    pub all_manifest_workloads_in_scope: bool, // data_class: PUBLIC
    pub tenant_residency_region_label_required: bool, // data_class: PUBLIC
    pub namespace_residency_label_required: bool, // data_class: PUBLIC
    pub workload_node_affinity_required: bool, // data_class: PUBLIC
    pub topology_region_constraint_required: bool, // data_class: PUBLIC
    pub storage_residency_policy_ref_required: bool, // data_class: PUBLIC
    pub telemetry_residency_policy_ref_required: bool, // data_class: PUBLIC
    pub audit_residency_policy_ref_required: bool, // data_class: PUBLIC
    pub cross_region_egress_policy_ref_required: bool, // data_class: PUBLIC
    pub tenant_model_jurisdiction_ref_required: bool, // data_class: PUBLIC
    pub cell_placement_residency_ref_required: bool, // data_class: PUBLIC
    pub admission_policy_evidence_required: bool, // data_class: PUBLIC
    pub workload_manifest_evidence_required: bool, // data_class: PUBLIC
    pub residency_audit_evidence_required: bool, // data_class: PUBLIC
    pub review_only_contract: bool,           // data_class: PUBLIC
    pub kubernetes_cluster_attached: bool,    // data_class: INTERNAL_ONLY
    pub namespace_created: bool,              // data_class: INTERNAL_ONLY
    pub node_affinity_applied: bool,          // data_class: INTERNAL_ONLY
    pub scheduler_runtime_observed: bool,     // data_class: INTERNAL_ONLY
    pub storage_residency_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub telemetry_residency_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub audit_residency_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub cross_region_egress_runtime_observed: bool, // data_class: INTERNAL_ONLY
    pub workload_runtime_deployed: bool,      // data_class: INTERNAL_ONLY
    pub cloud_substrate_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Fd001TenantResidencyError {
    WorkloadManifest(Fd001TenantWorkloadManifestError),
    TenantAdmissionPolicy(Fd001TenantAdmissionPolicyError),
    InvalidContractName,
    InvalidProgramName,
    InvalidSubstrateName,
    InvalidTenantNamespace,
    InvalidTenantCellId,
    InvalidResidencyRegion,
    InvalidManifestName,
    InvalidWorkloadCount,
    InvalidAdmissionPolicyContractName,
    InvalidTenantModelRef,
    MissingRequirements,
    DuplicateRequirement(String),
    MissingWorkloadKind(Fd001TenantWorkloadKind),
    MissingRequirementKind(Fd001TenantResidencyRequirementKind),
    InvalidRequirementId,
    InvalidKubernetesResourceKind,
    InvalidPolicyRef,
    InvalidExpectedEvidenceRef,
    InvalidOfficialDocUrl,
    InvalidSourceManifestRef,
    InvalidSourceAdmissionPolicyRef,
    InvalidSourceTenantModelRef,
    MissingRequiredControl(&'static str),
    RuntimeAttachmentOverclaim,
}

pub fn fd001_tenant_residency_contract()
-> Result<Fd001TenantResidencyContract, Fd001TenantResidencyError> {
    let manifest = fd001_tenant_workload_manifest();
    validate_fd001_tenant_workload_manifest(&manifest)
        .map_err(Fd001TenantResidencyError::WorkloadManifest)?;
    let admission_policy = fd001_tenant_admission_policy_contract()
        .map_err(Fd001TenantResidencyError::TenantAdmissionPolicy)?;
    validate_fd001_tenant_admission_policy_contract(&admission_policy)
        .map_err(Fd001TenantResidencyError::TenantAdmissionPolicy)?;

    Ok(Fd001TenantResidencyContract {
        contract_name: CONTRACT_NAME,
        program_name: PROGRAM_NAME,
        substrate_name: SUBSTRATE_NAME,
        tenant_namespace: manifest.tenant_namespace,
        tenant_cell_id: manifest.tenant_cell_id,
        residency_region: manifest.residency_region,
        workload_manifest_name: manifest.manifest_name,
        workload_manifest_count: manifest.workloads.len(),
        tenant_admission_policy_contract_name: admission_policy.contract_name,
        tenant_model_ref: SOURCE_TENANT_MODEL_REF,
        requirements: residency_requirements(),
        official_docs_required: true,
        all_manifest_workloads_in_scope: true,
        tenant_residency_region_label_required: true,
        namespace_residency_label_required: true,
        workload_node_affinity_required: true,
        topology_region_constraint_required: true,
        storage_residency_policy_ref_required: true,
        telemetry_residency_policy_ref_required: true,
        audit_residency_policy_ref_required: true,
        cross_region_egress_policy_ref_required: true,
        tenant_model_jurisdiction_ref_required: true,
        cell_placement_residency_ref_required: true,
        admission_policy_evidence_required: true,
        workload_manifest_evidence_required: true,
        residency_audit_evidence_required: true,
        review_only_contract: true,
        kubernetes_cluster_attached: false,
        namespace_created: false,
        node_affinity_applied: false,
        scheduler_runtime_observed: false,
        storage_residency_runtime_attached: false,
        telemetry_residency_runtime_attached: false,
        audit_residency_runtime_attached: false,
        cross_region_egress_runtime_observed: false,
        workload_runtime_deployed: false,
        cloud_substrate_runtime_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: SCHEMA_VERSION,
    })
}

pub fn validate_fd001_tenant_residency_contract(
    contract: &Fd001TenantResidencyContract,
) -> Result<(), Fd001TenantResidencyError> {
    let manifest = fd001_tenant_workload_manifest();
    validate_fd001_tenant_workload_manifest(&manifest)
        .map_err(Fd001TenantResidencyError::WorkloadManifest)?;
    let admission_policy = fd001_tenant_admission_policy_contract()
        .map_err(Fd001TenantResidencyError::TenantAdmissionPolicy)?;
    validate_fd001_tenant_admission_policy_contract(&admission_policy)
        .map_err(Fd001TenantResidencyError::TenantAdmissionPolicy)?;

    validate_slug(
        contract.contract_name,
        Fd001TenantResidencyError::InvalidContractName,
    )?;
    if contract.program_name != PROGRAM_NAME {
        return Err(Fd001TenantResidencyError::InvalidProgramName);
    }
    if contract.substrate_name != SUBSTRATE_NAME {
        return Err(Fd001TenantResidencyError::InvalidSubstrateName);
    }
    validate_tenant_namespace(contract.tenant_namespace)?;
    validate_cell_id(contract.tenant_cell_id)?;
    validate_region(contract.residency_region)?;
    if contract.workload_manifest_name != manifest.manifest_name {
        return Err(Fd001TenantResidencyError::InvalidManifestName);
    }
    if contract.workload_manifest_count != manifest.workloads.len() {
        return Err(Fd001TenantResidencyError::InvalidWorkloadCount);
    }
    if contract.tenant_admission_policy_contract_name != admission_policy.contract_name {
        return Err(Fd001TenantResidencyError::InvalidAdmissionPolicyContractName);
    }
    validate_prefixed_ref(
        contract.tenant_model_ref,
        "specs/tenant-model.json",
        Fd001TenantResidencyError::InvalidTenantModelRef,
    )?;
    if contract.requirements.len() < MIN_REQUIREMENT_COUNT
        || contract.schema_version != SCHEMA_VERSION
    {
        return Err(Fd001TenantResidencyError::MissingRequirements);
    }
    validate_required_controls(contract)?;
    validate_nonclaims(contract)?;
    validate_requirements(contract)?;
    Ok(())
}

pub fn fd001_tenant_residency_doc_urls(
    contract: &Fd001TenantResidencyContract,
) -> Vec<&'static str> {
    let mut docs = BTreeSet::new();
    for requirement in &contract.requirements {
        docs.insert(requirement.official_doc_url);
    }
    docs.into_iter().collect()
}

fn residency_requirements() -> Vec<Fd001TenantResidencyRequirement> {
    vec![
        requirement(
            "tenant-residency-region-labels-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantResidencyRequirementKind::TenantResidencyRegionLabelRequired,
            "KubernetesLabel",
            "tenant-residency-region-labels",
            KUBERNETES_LABELS_DOC_URL,
        ),
        requirement(
            "namespace-residency-labels-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantResidencyRequirementKind::NamespaceResidencyLabelRequired,
            "KubernetesNamespace",
            "namespace-residency-labels",
            KUBERNETES_LABELS_DOC_URL,
        ),
        requirement(
            "workload-node-affinity-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantResidencyRequirementKind::WorkloadNodeAffinityRequired,
            "NodeAffinity",
            "workload-node-affinity",
            KUBERNETES_NODE_AFFINITY_DOC_URL,
        ),
        requirement(
            "topology-region-constraint-all-workloads",
            Fd001TenantWorkloadKind::AccountingJournal,
            Fd001TenantResidencyRequirementKind::TopologyRegionConstraintRequired,
            "TopologySpreadConstraint",
            "topology-region-constraint",
            KUBERNETES_TOPOLOGY_SPREAD_DOC_URL,
        ),
        requirement(
            "storage-residency-policy-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantResidencyRequirementKind::StorageResidencyPolicyRefRequired,
            "StorageResidencyPolicy",
            "storage-residency-policy",
            AWS_DATA_PERIMETER_DOC_URL,
        ),
        requirement(
            "telemetry-residency-policy-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantResidencyRequirementKind::TelemetryResidencyPolicyRefRequired,
            "OpenTelemetryResource",
            "telemetry-residency-policy",
            OTEL_RESOURCE_SEMCONV_DOC_URL,
        ),
        requirement(
            "audit-residency-policy-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantResidencyRequirementKind::AuditResidencyPolicyRefRequired,
            "AuditResidencyPolicy",
            "audit-residency-policy",
            AWS_DATA_PERIMETER_DOC_URL,
        ),
        requirement(
            "cross-region-egress-policy-all-workloads",
            Fd001TenantWorkloadKind::AccountingJournal,
            Fd001TenantResidencyRequirementKind::CrossRegionEgressPolicyRefRequired,
            "NetworkPolicy",
            "cross-region-egress-policy",
            KUBERNETES_NETWORK_POLICY_DOC_URL,
        ),
        requirement(
            "tenant-model-jurisdiction-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantResidencyRequirementKind::TenantModelJurisdictionRefRequired,
            "TenantModelJurisdiction",
            "tenant-model-jurisdiction",
            AWS_DATA_PERIMETER_DOC_URL,
        ),
        requirement(
            "cell-placement-residency-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantResidencyRequirementKind::CellPlacementResidencyRefRequired,
            "CellPlacement",
            "cell-placement-residency",
            KUBERNETES_NODE_AFFINITY_DOC_URL,
        ),
        requirement(
            "admission-policy-residency-evidence-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantResidencyRequirementKind::AdmissionPolicyEvidenceRequired,
            "ValidatingAdmissionPolicy",
            "admission-policy-residency-evidence",
            KUBERNETES_LABELS_DOC_URL,
        ),
        requirement(
            "workload-manifest-residency-evidence-all-workloads",
            Fd001TenantWorkloadKind::AccountingJournal,
            Fd001TenantResidencyRequirementKind::WorkloadManifestEvidenceRequired,
            "WorkloadManifest",
            "workload-manifest-residency-evidence",
            KUBERNETES_LABELS_DOC_URL,
        ),
        requirement(
            "residency-audit-evidence-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantResidencyRequirementKind::ResidencyAuditEvidenceRequired,
            "ResidencyAuditEvidence",
            "residency-audit-evidence",
            OTEL_RESOURCE_SEMCONV_DOC_URL,
        ),
    ]
}

fn requirement(
    requirement_id: &'static str,
    workload_kind: Fd001TenantWorkloadKind,
    requirement_kind: Fd001TenantResidencyRequirementKind,
    kubernetes_resource_kind: &'static str,
    policy_suffix: &'static str,
    official_doc_url: &'static str,
) -> Fd001TenantResidencyRequirement {
    Fd001TenantResidencyRequirement {
        requirement_id,
        workload_kind,
        requirement_kind,
        kubernetes_resource_kind,
        policy_ref: policy_ref(policy_suffix),
        expected_evidence_ref: EXPECTED_EVIDENCE_REF,
        official_doc_url,
        source_manifest_ref: SOURCE_MANIFEST_REF,
        source_admission_policy_ref: SOURCE_ADMISSION_POLICY_REF,
        source_tenant_model_ref: SOURCE_TENANT_MODEL_REF,
        applies_to_all_manifest_workloads: true,
        runtime_observation_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

fn policy_ref(policy_suffix: &'static str) -> &'static str {
    match policy_suffix {
        "tenant-residency-region-labels" => "policy/residency/fd001/tenant-residency-region-labels",
        "namespace-residency-labels" => "policy/residency/fd001/namespace-residency-labels",
        "workload-node-affinity" => "policy/residency/fd001/workload-node-affinity",
        "topology-region-constraint" => "policy/residency/fd001/topology-region-constraint",
        "storage-residency-policy" => "policy/residency/fd001/storage-residency-policy",
        "telemetry-residency-policy" => "policy/residency/fd001/telemetry-residency-policy",
        "audit-residency-policy" => "policy/residency/fd001/audit-residency-policy",
        "cross-region-egress-policy" => "policy/residency/fd001/cross-region-egress-policy",
        "tenant-model-jurisdiction" => "policy/residency/fd001/tenant-model-jurisdiction",
        "cell-placement-residency" => "policy/residency/fd001/cell-placement-residency",
        "admission-policy-residency-evidence" => {
            "policy/residency/fd001/admission-policy-residency-evidence"
        }
        "workload-manifest-residency-evidence" => {
            "policy/residency/fd001/workload-manifest-residency-evidence"
        }
        "residency-audit-evidence" => "policy/residency/fd001/residency-audit-evidence",
        _ => "policy/residency/fd001/invalid",
    }
}

fn validate_required_controls(
    contract: &Fd001TenantResidencyContract,
) -> Result<(), Fd001TenantResidencyError> {
    for (enabled, name) in [
        (contract.official_docs_required, "official_docs_required"),
        (
            contract.all_manifest_workloads_in_scope,
            "all_manifest_workloads_in_scope",
        ),
        (
            contract.tenant_residency_region_label_required,
            "tenant_residency_region_label_required",
        ),
        (
            contract.namespace_residency_label_required,
            "namespace_residency_label_required",
        ),
        (
            contract.workload_node_affinity_required,
            "workload_node_affinity_required",
        ),
        (
            contract.topology_region_constraint_required,
            "topology_region_constraint_required",
        ),
        (
            contract.storage_residency_policy_ref_required,
            "storage_residency_policy_ref_required",
        ),
        (
            contract.telemetry_residency_policy_ref_required,
            "telemetry_residency_policy_ref_required",
        ),
        (
            contract.audit_residency_policy_ref_required,
            "audit_residency_policy_ref_required",
        ),
        (
            contract.cross_region_egress_policy_ref_required,
            "cross_region_egress_policy_ref_required",
        ),
        (
            contract.tenant_model_jurisdiction_ref_required,
            "tenant_model_jurisdiction_ref_required",
        ),
        (
            contract.cell_placement_residency_ref_required,
            "cell_placement_residency_ref_required",
        ),
        (
            contract.admission_policy_evidence_required,
            "admission_policy_evidence_required",
        ),
        (
            contract.workload_manifest_evidence_required,
            "workload_manifest_evidence_required",
        ),
        (
            contract.residency_audit_evidence_required,
            "residency_audit_evidence_required",
        ),
        (contract.review_only_contract, "review_only_contract"),
    ] {
        require_control(enabled, name)?;
    }
    Ok(())
}

fn validate_nonclaims(
    contract: &Fd001TenantResidencyContract,
) -> Result<(), Fd001TenantResidencyError> {
    if contract.kubernetes_cluster_attached
        || contract.namespace_created
        || contract.node_affinity_applied
        || contract.scheduler_runtime_observed
        || contract.storage_residency_runtime_attached
        || contract.telemetry_residency_runtime_attached
        || contract.audit_residency_runtime_attached
        || contract.cross_region_egress_runtime_observed
        || contract.workload_runtime_deployed
        || contract.cloud_substrate_runtime_attached
        || contract.runtime_audit_chain_emission_attached
    {
        return Err(Fd001TenantResidencyError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_requirements(
    contract: &Fd001TenantResidencyContract,
) -> Result<(), Fd001TenantResidencyError> {
    let mut seen_requirements = BTreeSet::new();
    let mut seen_workloads = BTreeSet::new();
    let mut seen_requirement_kinds = BTreeSet::new();
    for requirement in &contract.requirements {
        validate_requirement(requirement)?;
        if !seen_requirements.insert(requirement.requirement_id) {
            return Err(Fd001TenantResidencyError::DuplicateRequirement(
                requirement.requirement_id.to_owned(),
            ));
        }
        seen_workloads.insert(requirement.workload_kind);
        seen_requirement_kinds.insert(requirement.requirement_kind);
    }
    for workload_kind in required_workload_kinds() {
        if !seen_workloads.contains(&workload_kind) {
            return Err(Fd001TenantResidencyError::MissingWorkloadKind(
                workload_kind,
            ));
        }
    }
    for requirement_kind in required_requirement_kinds() {
        if !seen_requirement_kinds.contains(&requirement_kind) {
            return Err(Fd001TenantResidencyError::MissingRequirementKind(
                requirement_kind,
            ));
        }
    }
    Ok(())
}

fn validate_requirement(
    requirement: &Fd001TenantResidencyRequirement,
) -> Result<(), Fd001TenantResidencyError> {
    validate_slug(
        requirement.requirement_id,
        Fd001TenantResidencyError::InvalidRequirementId,
    )?;
    validate_kubernetes_resource_kind(requirement.kubernetes_resource_kind)?;
    validate_prefixed_ref(
        requirement.policy_ref,
        POLICY_REF_PREFIX,
        Fd001TenantResidencyError::InvalidPolicyRef,
    )?;
    validate_prefixed_ref(
        requirement.expected_evidence_ref,
        "evidence/residency/fd001-tenant-rbac/",
        Fd001TenantResidencyError::InvalidExpectedEvidenceRef,
    )?;
    validate_doc_url(requirement.official_doc_url)?;
    validate_prefixed_ref(
        requirement.source_manifest_ref,
        "crates/tenant-rbac-tenant-workload-manifest/",
        Fd001TenantResidencyError::InvalidSourceManifestRef,
    )?;
    validate_prefixed_ref(
        requirement.source_admission_policy_ref,
        "crates/tenant-rbac-tenant-admission-policy/",
        Fd001TenantResidencyError::InvalidSourceAdmissionPolicyRef,
    )?;
    validate_prefixed_ref(
        requirement.source_tenant_model_ref,
        "specs/tenant-model.json",
        Fd001TenantResidencyError::InvalidSourceTenantModelRef,
    )?;
    if !requirement.applies_to_all_manifest_workloads {
        return Err(Fd001TenantResidencyError::MissingRequiredControl(
            "requirement_applies_to_all_manifest_workloads",
        ));
    }
    if requirement.runtime_observation_attached || requirement.schema_version != SCHEMA_VERSION {
        return Err(Fd001TenantResidencyError::RuntimeAttachmentOverclaim);
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

fn required_requirement_kinds() -> [Fd001TenantResidencyRequirementKind; 13] {
    [
        Fd001TenantResidencyRequirementKind::TenantResidencyRegionLabelRequired,
        Fd001TenantResidencyRequirementKind::NamespaceResidencyLabelRequired,
        Fd001TenantResidencyRequirementKind::WorkloadNodeAffinityRequired,
        Fd001TenantResidencyRequirementKind::TopologyRegionConstraintRequired,
        Fd001TenantResidencyRequirementKind::StorageResidencyPolicyRefRequired,
        Fd001TenantResidencyRequirementKind::TelemetryResidencyPolicyRefRequired,
        Fd001TenantResidencyRequirementKind::AuditResidencyPolicyRefRequired,
        Fd001TenantResidencyRequirementKind::CrossRegionEgressPolicyRefRequired,
        Fd001TenantResidencyRequirementKind::TenantModelJurisdictionRefRequired,
        Fd001TenantResidencyRequirementKind::CellPlacementResidencyRefRequired,
        Fd001TenantResidencyRequirementKind::AdmissionPolicyEvidenceRequired,
        Fd001TenantResidencyRequirementKind::WorkloadManifestEvidenceRequired,
        Fd001TenantResidencyRequirementKind::ResidencyAuditEvidenceRequired,
    ]
}

fn validate_kubernetes_resource_kind(value: &str) -> Result<(), Fd001TenantResidencyError> {
    if ![
        "KubernetesLabel",
        "KubernetesNamespace",
        "NodeAffinity",
        "TopologySpreadConstraint",
        "StorageResidencyPolicy",
        "OpenTelemetryResource",
        "AuditResidencyPolicy",
        "NetworkPolicy",
        "TenantModelJurisdiction",
        "CellPlacement",
        "ValidatingAdmissionPolicy",
        "WorkloadManifest",
        "ResidencyAuditEvidence",
    ]
    .contains(&value)
    {
        return Err(Fd001TenantResidencyError::InvalidKubernetesResourceKind);
    }
    Ok(())
}

fn validate_doc_url(url: &str) -> Result<(), Fd001TenantResidencyError> {
    if has_unsafe_ref_text(url)
        || ![
            KUBERNETES_LABELS_DOC_URL,
            KUBERNETES_NODE_AFFINITY_DOC_URL,
            KUBERNETES_TOPOLOGY_SPREAD_DOC_URL,
            KUBERNETES_NETWORK_POLICY_DOC_URL,
            OTEL_RESOURCE_SEMCONV_DOC_URL,
            AWS_DATA_PERIMETER_DOC_URL,
        ]
        .contains(&url)
    {
        return Err(Fd001TenantResidencyError::InvalidOfficialDocUrl);
    }
    Ok(())
}

fn validate_slug(
    value: &str,
    error: Fd001TenantResidencyError,
) -> Result<(), Fd001TenantResidencyError> {
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

fn validate_tenant_namespace(value: &str) -> Result<(), Fd001TenantResidencyError> {
    validate_slug(value, Fd001TenantResidencyError::InvalidTenantNamespace)?;
    if value != TENANT_NAMESPACE {
        return Err(Fd001TenantResidencyError::InvalidTenantNamespace);
    }
    Ok(())
}

fn validate_cell_id(value: &str) -> Result<(), Fd001TenantResidencyError> {
    validate_slug(value, Fd001TenantResidencyError::InvalidTenantCellId)?;
    if value != TENANT_CELL_ID {
        return Err(Fd001TenantResidencyError::InvalidTenantCellId);
    }
    Ok(())
}

fn validate_region(value: &str) -> Result<(), Fd001TenantResidencyError> {
    validate_slug(value, Fd001TenantResidencyError::InvalidResidencyRegion)?;
    if value != RESIDENCY_REGION {
        return Err(Fd001TenantResidencyError::InvalidResidencyRegion);
    }
    Ok(())
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: Fd001TenantResidencyError,
) -> Result<(), Fd001TenantResidencyError> {
    if value.len() <= prefix.len() || !value.starts_with(prefix) || has_unsafe_ref_text(value) {
        return Err(error);
    }
    Ok(())
}

fn require_control(value: bool, control: &'static str) -> Result<(), Fd001TenantResidencyError> {
    if value {
        Ok(())
    } else {
        Err(Fd001TenantResidencyError::MissingRequiredControl(control))
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
