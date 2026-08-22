//! FD-001 tenant egress policy contract for later Oyatie Cloud dogfooding.
//!
//! This review-only crate defines network egress guardrails that must exist
//! before FD-001 Tenant RBAC, HR, Payroll, and Accounting workloads can be
//! promoted as production tenant workloads on the future Oyatie Cloud substrate.
//! It binds the FD-001 tenant-workload manifest and tenant admission policy to
//! Kubernetes NetworkPolicy egress isolation, default-deny posture, DNS-only
//! egress exceptions, same-namespace Service allowlists, explicit cross-namespace
//! selectors, external CIDR deny-by-default posture, ipBlock exception evidence,
//! pinned ports/protocols, tenant label selectors, network-policy provider
//! evidence, and egress audit evidence. It does not attach a Kubernetes cluster,
//! install a network-policy provider, apply NetworkPolicy objects, enforce
//! runtime egress, run DNS probes, deploy workloads, attach a cloud substrate
//! runtime, or emit runtime audit-chain events.
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
const MIN_RULE_COUNT: usize = 11;
const CONTRACT_NAME: &str = "fd001-tenant-rbac-tenant-egress-policy-contract";
const PROGRAM_NAME: &str = "fd-001-tenant-rbac-generic";
const SUBSTRATE_NAME: &str = "oyatie-cloud";
const TENANT_NAMESPACE: &str = "oyatie-fd001-tenant-rbac-dev";
const SOURCE_MANIFEST_REF: &str =
    "crates/tenant-rbac-tenant-workload-manifest/src/lib.rs::fd001_tenant_workload_manifest";
const SOURCE_ADMISSION_POLICY_REF: &str = "crates/tenant-rbac-tenant-admission-policy/src/lib.rs::fd001_tenant_admission_policy_contract";
const POLICY_REF_PREFIX: &str = "policy/network-egress/fd001/";
const EXPECTED_EVIDENCE_REF: &str =
    "evidence/network-egress/fd001-tenant-rbac/egress-policy-review.jsonl";
const NETWORK_POLICY_KIND: &str = "NetworkPolicy";
const POLICY_TYPE: &str = "Egress";

const NETWORK_POLICIES_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/services-networking/network-policies/";
const DECLARE_NETWORK_POLICY_DOC_URL: &str =
    "https://kubernetes.io/docs/tasks/administer-cluster/declare-network-policy/";
const DNS_FOR_SERVICES_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/services-networking/dns-pod-service/";
const SERVICE_DOC_URL: &str = "https://kubernetes.io/docs/concepts/services-networking/service/";
const NAMESPACES_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/overview/working-with-objects/namespaces/";
const MULTI_TENANCY_DOC_URL: &str = "https://kubernetes.io/docs/concepts/security/multi-tenancy/";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Fd001TenantEgressPolicyRuleKind {
    DefaultDenyEgressRequired,
    DnsEgressOnlyRequired,
    SameNamespaceServiceEgressRequired,
    CrossNamespaceEgressExplicitSelectorRequired,
    ExternalCidrEgressForbiddenByDefault,
    IpBlockExceptionEvidenceRequired,
    ProtocolPortPinnedRequired,
    TenantLabelSelectorRequired,
    NetworkPolicyProviderEvidenceRequired,
    EgressAuditEvidenceRequired,
    AdmissionPolicyEvidenceRequired,
}

impl Fd001TenantEgressPolicyRuleKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DefaultDenyEgressRequired => "default_deny_egress_required",
            Self::DnsEgressOnlyRequired => "dns_egress_only_required",
            Self::SameNamespaceServiceEgressRequired => "same_namespace_service_egress_required",
            Self::CrossNamespaceEgressExplicitSelectorRequired => {
                "cross_namespace_egress_explicit_selector_required"
            }
            Self::ExternalCidrEgressForbiddenByDefault => {
                "external_cidr_egress_forbidden_by_default"
            }
            Self::IpBlockExceptionEvidenceRequired => "ip_block_exception_evidence_required",
            Self::ProtocolPortPinnedRequired => "protocol_port_pinned_required",
            Self::TenantLabelSelectorRequired => "tenant_label_selector_required",
            Self::NetworkPolicyProviderEvidenceRequired => {
                "network_policy_provider_evidence_required"
            }
            Self::EgressAuditEvidenceRequired => "egress_audit_evidence_required",
            Self::AdmissionPolicyEvidenceRequired => "admission_policy_evidence_required",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fd001TenantEgressPolicyRule {
    pub rule_id: &'static str,                      // data_class: PUBLIC
    pub workload_kind: Fd001TenantWorkloadKind,     // data_class: PUBLIC
    pub rule_kind: Fd001TenantEgressPolicyRuleKind, // data_class: PUBLIC
    pub network_policy_kind: &'static str,          // data_class: PUBLIC
    pub policy_type: &'static str,                  // data_class: PUBLIC
    pub policy_ref: &'static str,                   // data_class: INTERNAL_ONLY
    pub expected_evidence_ref: &'static str,        // data_class: INTERNAL_ONLY
    pub official_doc_url: &'static str,             // data_class: PUBLIC
    pub source_manifest_ref: &'static str,          // data_class: INTERNAL_ONLY
    pub source_admission_policy_ref: &'static str,  // data_class: INTERNAL_ONLY
    pub applies_to_all_manifest_workloads: bool,    // data_class: PUBLIC
    pub runtime_enforcement_attached: bool,         // data_class: INTERNAL_ONLY
    pub schema_version: u32,                        // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fd001TenantEgressPolicyContract {
    pub contract_name: &'static str,          // data_class: PUBLIC
    pub program_name: &'static str,           // data_class: PUBLIC
    pub substrate_name: &'static str,         // data_class: PUBLIC
    pub tenant_namespace: &'static str,       // data_class: INTERNAL_ONLY
    pub workload_manifest_name: &'static str, // data_class: PUBLIC
    pub workload_manifest_count: usize,       // data_class: PUBLIC
    pub tenant_admission_policy_contract_name: &'static str, // data_class: PUBLIC
    pub rules: Vec<Fd001TenantEgressPolicyRule>, // data_class: INTERNAL_ONLY
    pub official_docs_required: bool,         // data_class: PUBLIC
    pub all_manifest_workloads_in_scope: bool, // data_class: PUBLIC
    pub default_deny_egress_required: bool,   // data_class: PUBLIC
    pub dns_egress_only_required: bool,       // data_class: PUBLIC
    pub same_namespace_service_egress_required: bool, // data_class: PUBLIC
    pub cross_namespace_egress_explicit_selector_required: bool, // data_class: PUBLIC
    pub external_cidr_egress_forbidden_by_default: bool, // data_class: PUBLIC
    pub ip_block_exception_evidence_required: bool, // data_class: PUBLIC
    pub protocol_port_pinned_required: bool,  // data_class: PUBLIC
    pub tenant_label_selector_required: bool, // data_class: PUBLIC
    pub network_policy_provider_evidence_required: bool, // data_class: PUBLIC
    pub egress_audit_evidence_required: bool, // data_class: PUBLIC
    pub admission_policy_evidence_required: bool, // data_class: PUBLIC
    pub review_only_contract: bool,           // data_class: PUBLIC
    pub kubernetes_cluster_attached: bool,    // data_class: INTERNAL_ONLY
    pub network_policy_provider_attached: bool, // data_class: INTERNAL_ONLY
    pub network_policy_applied: bool,         // data_class: INTERNAL_ONLY
    pub egress_runtime_enforced: bool,        // data_class: INTERNAL_ONLY
    pub dns_probe_runtime_attached: bool,     // data_class: INTERNAL_ONLY
    pub external_egress_runtime_allowed: bool, // data_class: INTERNAL_ONLY
    pub workload_runtime_deployed: bool,      // data_class: INTERNAL_ONLY
    pub cloud_substrate_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Fd001TenantEgressPolicyError {
    WorkloadManifest(Fd001TenantWorkloadManifestError),
    TenantAdmissionPolicy(Fd001TenantAdmissionPolicyError),
    InvalidContractName,
    InvalidProgramName,
    InvalidSubstrateName,
    InvalidTenantNamespace,
    InvalidManifestName,
    InvalidWorkloadCount,
    InvalidAdmissionPolicyContractName,
    MissingRules,
    DuplicateRule(String),
    MissingWorkloadKind(Fd001TenantWorkloadKind),
    MissingRuleKind(Fd001TenantEgressPolicyRuleKind),
    InvalidRuleId,
    InvalidNetworkPolicyKind,
    InvalidPolicyType,
    InvalidPolicyRef,
    InvalidExpectedEvidenceRef,
    InvalidOfficialDocUrl,
    InvalidSourceManifestRef,
    InvalidSourceAdmissionPolicyRef,
    MissingRequiredControl(&'static str),
    RuntimeAttachmentOverclaim,
}

pub fn fd001_tenant_egress_policy_contract()
-> Result<Fd001TenantEgressPolicyContract, Fd001TenantEgressPolicyError> {
    let manifest = fd001_tenant_workload_manifest();
    validate_fd001_tenant_workload_manifest(&manifest)
        .map_err(Fd001TenantEgressPolicyError::WorkloadManifest)?;
    let admission_policy = fd001_tenant_admission_policy_contract()
        .map_err(Fd001TenantEgressPolicyError::TenantAdmissionPolicy)?;
    validate_fd001_tenant_admission_policy_contract(&admission_policy)
        .map_err(Fd001TenantEgressPolicyError::TenantAdmissionPolicy)?;

    Ok(Fd001TenantEgressPolicyContract {
        contract_name: CONTRACT_NAME,
        program_name: PROGRAM_NAME,
        substrate_name: SUBSTRATE_NAME,
        tenant_namespace: manifest.tenant_namespace,
        workload_manifest_name: manifest.manifest_name,
        workload_manifest_count: manifest.workloads.len(),
        tenant_admission_policy_contract_name: admission_policy.contract_name,
        rules: egress_rules(),
        official_docs_required: true,
        all_manifest_workloads_in_scope: true,
        default_deny_egress_required: true,
        dns_egress_only_required: true,
        same_namespace_service_egress_required: true,
        cross_namespace_egress_explicit_selector_required: true,
        external_cidr_egress_forbidden_by_default: true,
        ip_block_exception_evidence_required: true,
        protocol_port_pinned_required: true,
        tenant_label_selector_required: true,
        network_policy_provider_evidence_required: true,
        egress_audit_evidence_required: true,
        admission_policy_evidence_required: true,
        review_only_contract: true,
        kubernetes_cluster_attached: false,
        network_policy_provider_attached: false,
        network_policy_applied: false,
        egress_runtime_enforced: false,
        dns_probe_runtime_attached: false,
        external_egress_runtime_allowed: false,
        workload_runtime_deployed: false,
        cloud_substrate_runtime_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: SCHEMA_VERSION,
    })
}

pub fn validate_fd001_tenant_egress_policy_contract(
    contract: &Fd001TenantEgressPolicyContract,
) -> Result<(), Fd001TenantEgressPolicyError> {
    let manifest = fd001_tenant_workload_manifest();
    validate_fd001_tenant_workload_manifest(&manifest)
        .map_err(Fd001TenantEgressPolicyError::WorkloadManifest)?;
    let admission_policy = fd001_tenant_admission_policy_contract()
        .map_err(Fd001TenantEgressPolicyError::TenantAdmissionPolicy)?;
    validate_fd001_tenant_admission_policy_contract(&admission_policy)
        .map_err(Fd001TenantEgressPolicyError::TenantAdmissionPolicy)?;

    validate_slug(
        contract.contract_name,
        Fd001TenantEgressPolicyError::InvalidContractName,
    )?;
    if contract.program_name != PROGRAM_NAME {
        return Err(Fd001TenantEgressPolicyError::InvalidProgramName);
    }
    if contract.substrate_name != SUBSTRATE_NAME {
        return Err(Fd001TenantEgressPolicyError::InvalidSubstrateName);
    }
    validate_tenant_namespace(contract.tenant_namespace)?;
    if contract.workload_manifest_name != manifest.manifest_name {
        return Err(Fd001TenantEgressPolicyError::InvalidManifestName);
    }
    if contract.workload_manifest_count != manifest.workloads.len() {
        return Err(Fd001TenantEgressPolicyError::InvalidWorkloadCount);
    }
    if contract.tenant_admission_policy_contract_name != admission_policy.contract_name {
        return Err(Fd001TenantEgressPolicyError::InvalidAdmissionPolicyContractName);
    }
    if contract.rules.len() < MIN_RULE_COUNT || contract.schema_version != SCHEMA_VERSION {
        return Err(Fd001TenantEgressPolicyError::MissingRules);
    }
    validate_required_controls(contract)?;
    validate_nonclaims(contract)?;
    validate_rules(contract)?;
    Ok(())
}

pub fn fd001_tenant_egress_policy_doc_urls(
    contract: &Fd001TenantEgressPolicyContract,
) -> Vec<&'static str> {
    let mut docs = BTreeSet::new();
    for rule in &contract.rules {
        docs.insert(rule.official_doc_url);
    }
    docs.into_iter().collect()
}

fn egress_rules() -> Vec<Fd001TenantEgressPolicyRule> {
    vec![
        rule(
            "default-deny-egress-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantEgressPolicyRuleKind::DefaultDenyEgressRequired,
            "default-deny-egress",
            NETWORK_POLICIES_DOC_URL,
        ),
        rule(
            "dns-egress-only-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantEgressPolicyRuleKind::DnsEgressOnlyRequired,
            "dns-egress-only",
            DNS_FOR_SERVICES_DOC_URL,
        ),
        rule(
            "same-namespace-service-egress-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantEgressPolicyRuleKind::SameNamespaceServiceEgressRequired,
            "same-namespace-service-egress",
            SERVICE_DOC_URL,
        ),
        rule(
            "cross-namespace-egress-selector-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantEgressPolicyRuleKind::CrossNamespaceEgressExplicitSelectorRequired,
            "cross-namespace-egress-selector",
            NAMESPACES_DOC_URL,
        ),
        rule(
            "external-cidr-egress-forbidden-all-workloads",
            Fd001TenantWorkloadKind::AccountingJournal,
            Fd001TenantEgressPolicyRuleKind::ExternalCidrEgressForbiddenByDefault,
            "external-cidr-egress-forbidden",
            NETWORK_POLICIES_DOC_URL,
        ),
        rule(
            "ip-block-exception-evidence-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantEgressPolicyRuleKind::IpBlockExceptionEvidenceRequired,
            "ip-block-exception-evidence",
            NETWORK_POLICIES_DOC_URL,
        ),
        rule(
            "protocol-port-pinned-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantEgressPolicyRuleKind::ProtocolPortPinnedRequired,
            "protocol-port-pinned",
            DECLARE_NETWORK_POLICY_DOC_URL,
        ),
        rule(
            "tenant-label-selector-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantEgressPolicyRuleKind::TenantLabelSelectorRequired,
            "tenant-label-selector",
            MULTI_TENANCY_DOC_URL,
        ),
        rule(
            "network-policy-provider-evidence-all-workloads",
            Fd001TenantWorkloadKind::AccountingJournal,
            Fd001TenantEgressPolicyRuleKind::NetworkPolicyProviderEvidenceRequired,
            "network-policy-provider-evidence",
            DECLARE_NETWORK_POLICY_DOC_URL,
        ),
        rule(
            "egress-audit-evidence-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantEgressPolicyRuleKind::EgressAuditEvidenceRequired,
            "egress-audit-evidence",
            MULTI_TENANCY_DOC_URL,
        ),
        rule(
            "admission-policy-egress-evidence-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantEgressPolicyRuleKind::AdmissionPolicyEvidenceRequired,
            "admission-policy-egress-evidence",
            NETWORK_POLICIES_DOC_URL,
        ),
    ]
}

fn rule(
    rule_id: &'static str,
    workload_kind: Fd001TenantWorkloadKind,
    rule_kind: Fd001TenantEgressPolicyRuleKind,
    policy_suffix: &'static str,
    official_doc_url: &'static str,
) -> Fd001TenantEgressPolicyRule {
    Fd001TenantEgressPolicyRule {
        rule_id,
        workload_kind,
        rule_kind,
        network_policy_kind: NETWORK_POLICY_KIND,
        policy_type: POLICY_TYPE,
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
        "default-deny-egress" => "policy/network-egress/fd001/default-deny-egress",
        "dns-egress-only" => "policy/network-egress/fd001/dns-egress-only",
        "same-namespace-service-egress" => {
            "policy/network-egress/fd001/same-namespace-service-egress"
        }
        "cross-namespace-egress-selector" => {
            "policy/network-egress/fd001/cross-namespace-egress-selector"
        }
        "external-cidr-egress-forbidden" => {
            "policy/network-egress/fd001/external-cidr-egress-forbidden"
        }
        "ip-block-exception-evidence" => "policy/network-egress/fd001/ip-block-exception-evidence",
        "protocol-port-pinned" => "policy/network-egress/fd001/protocol-port-pinned",
        "tenant-label-selector" => "policy/network-egress/fd001/tenant-label-selector",
        "network-policy-provider-evidence" => {
            "policy/network-egress/fd001/network-policy-provider-evidence"
        }
        "egress-audit-evidence" => "policy/network-egress/fd001/egress-audit-evidence",
        "admission-policy-egress-evidence" => {
            "policy/network-egress/fd001/admission-policy-egress-evidence"
        }
        _ => "policy/network-egress/fd001/invalid",
    }
}

fn validate_required_controls(
    contract: &Fd001TenantEgressPolicyContract,
) -> Result<(), Fd001TenantEgressPolicyError> {
    for (enabled, name) in [
        (contract.official_docs_required, "official_docs_required"),
        (
            contract.all_manifest_workloads_in_scope,
            "all_manifest_workloads_in_scope",
        ),
        (
            contract.default_deny_egress_required,
            "default_deny_egress_required",
        ),
        (
            contract.dns_egress_only_required,
            "dns_egress_only_required",
        ),
        (
            contract.same_namespace_service_egress_required,
            "same_namespace_service_egress_required",
        ),
        (
            contract.cross_namespace_egress_explicit_selector_required,
            "cross_namespace_egress_explicit_selector_required",
        ),
        (
            contract.external_cidr_egress_forbidden_by_default,
            "external_cidr_egress_forbidden_by_default",
        ),
        (
            contract.ip_block_exception_evidence_required,
            "ip_block_exception_evidence_required",
        ),
        (
            contract.protocol_port_pinned_required,
            "protocol_port_pinned_required",
        ),
        (
            contract.tenant_label_selector_required,
            "tenant_label_selector_required",
        ),
        (
            contract.network_policy_provider_evidence_required,
            "network_policy_provider_evidence_required",
        ),
        (
            contract.egress_audit_evidence_required,
            "egress_audit_evidence_required",
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
    contract: &Fd001TenantEgressPolicyContract,
) -> Result<(), Fd001TenantEgressPolicyError> {
    if contract.kubernetes_cluster_attached
        || contract.network_policy_provider_attached
        || contract.network_policy_applied
        || contract.egress_runtime_enforced
        || contract.dns_probe_runtime_attached
        || contract.external_egress_runtime_allowed
        || contract.workload_runtime_deployed
        || contract.cloud_substrate_runtime_attached
        || contract.runtime_audit_chain_emission_attached
    {
        return Err(Fd001TenantEgressPolicyError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_rules(
    contract: &Fd001TenantEgressPolicyContract,
) -> Result<(), Fd001TenantEgressPolicyError> {
    let mut seen_rules = BTreeSet::new();
    let mut seen_workloads = BTreeSet::new();
    let mut seen_rule_kinds = BTreeSet::new();
    for rule in &contract.rules {
        validate_rule(rule)?;
        if !seen_rules.insert(rule.rule_id) {
            return Err(Fd001TenantEgressPolicyError::DuplicateRule(
                rule.rule_id.to_owned(),
            ));
        }
        seen_workloads.insert(rule.workload_kind);
        seen_rule_kinds.insert(rule.rule_kind);
    }
    for workload_kind in required_workload_kinds() {
        if !seen_workloads.contains(&workload_kind) {
            return Err(Fd001TenantEgressPolicyError::MissingWorkloadKind(
                workload_kind,
            ));
        }
    }
    for rule_kind in required_rule_kinds() {
        if !seen_rule_kinds.contains(&rule_kind) {
            return Err(Fd001TenantEgressPolicyError::MissingRuleKind(rule_kind));
        }
    }
    Ok(())
}

fn validate_rule(rule: &Fd001TenantEgressPolicyRule) -> Result<(), Fd001TenantEgressPolicyError> {
    validate_slug(rule.rule_id, Fd001TenantEgressPolicyError::InvalidRuleId)?;
    if rule.network_policy_kind != NETWORK_POLICY_KIND {
        return Err(Fd001TenantEgressPolicyError::InvalidNetworkPolicyKind);
    }
    if rule.policy_type != POLICY_TYPE {
        return Err(Fd001TenantEgressPolicyError::InvalidPolicyType);
    }
    validate_prefixed_ref(
        rule.policy_ref,
        POLICY_REF_PREFIX,
        Fd001TenantEgressPolicyError::InvalidPolicyRef,
    )?;
    validate_prefixed_ref(
        rule.expected_evidence_ref,
        "evidence/network-egress/fd001-tenant-rbac/",
        Fd001TenantEgressPolicyError::InvalidExpectedEvidenceRef,
    )?;
    validate_doc_url(rule.official_doc_url)?;
    validate_prefixed_ref(
        rule.source_manifest_ref,
        "crates/tenant-rbac-tenant-workload-manifest/",
        Fd001TenantEgressPolicyError::InvalidSourceManifestRef,
    )?;
    validate_prefixed_ref(
        rule.source_admission_policy_ref,
        "crates/tenant-rbac-tenant-admission-policy/",
        Fd001TenantEgressPolicyError::InvalidSourceAdmissionPolicyRef,
    )?;
    if !rule.applies_to_all_manifest_workloads {
        return Err(Fd001TenantEgressPolicyError::MissingRequiredControl(
            "rule_applies_to_all_manifest_workloads",
        ));
    }
    if rule.runtime_enforcement_attached || rule.schema_version != SCHEMA_VERSION {
        return Err(Fd001TenantEgressPolicyError::RuntimeAttachmentOverclaim);
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

fn required_rule_kinds() -> [Fd001TenantEgressPolicyRuleKind; 11] {
    [
        Fd001TenantEgressPolicyRuleKind::DefaultDenyEgressRequired,
        Fd001TenantEgressPolicyRuleKind::DnsEgressOnlyRequired,
        Fd001TenantEgressPolicyRuleKind::SameNamespaceServiceEgressRequired,
        Fd001TenantEgressPolicyRuleKind::CrossNamespaceEgressExplicitSelectorRequired,
        Fd001TenantEgressPolicyRuleKind::ExternalCidrEgressForbiddenByDefault,
        Fd001TenantEgressPolicyRuleKind::IpBlockExceptionEvidenceRequired,
        Fd001TenantEgressPolicyRuleKind::ProtocolPortPinnedRequired,
        Fd001TenantEgressPolicyRuleKind::TenantLabelSelectorRequired,
        Fd001TenantEgressPolicyRuleKind::NetworkPolicyProviderEvidenceRequired,
        Fd001TenantEgressPolicyRuleKind::EgressAuditEvidenceRequired,
        Fd001TenantEgressPolicyRuleKind::AdmissionPolicyEvidenceRequired,
    ]
}

fn validate_doc_url(url: &str) -> Result<(), Fd001TenantEgressPolicyError> {
    if has_unsafe_ref_text(url)
        || ![
            NETWORK_POLICIES_DOC_URL,
            DECLARE_NETWORK_POLICY_DOC_URL,
            DNS_FOR_SERVICES_DOC_URL,
            SERVICE_DOC_URL,
            NAMESPACES_DOC_URL,
            MULTI_TENANCY_DOC_URL,
        ]
        .contains(&url)
    {
        return Err(Fd001TenantEgressPolicyError::InvalidOfficialDocUrl);
    }
    Ok(())
}

fn validate_slug(
    value: &str,
    error: Fd001TenantEgressPolicyError,
) -> Result<(), Fd001TenantEgressPolicyError> {
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

fn validate_tenant_namespace(value: &str) -> Result<(), Fd001TenantEgressPolicyError> {
    validate_slug(value, Fd001TenantEgressPolicyError::InvalidTenantNamespace)?;
    if value != TENANT_NAMESPACE {
        return Err(Fd001TenantEgressPolicyError::InvalidTenantNamespace);
    }
    Ok(())
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: Fd001TenantEgressPolicyError,
) -> Result<(), Fd001TenantEgressPolicyError> {
    if value.len() <= prefix.len() || !value.starts_with(prefix) || has_unsafe_ref_text(value) {
        return Err(error);
    }
    Ok(())
}

fn require_control(value: bool, control: &'static str) -> Result<(), Fd001TenantEgressPolicyError> {
    if value {
        Ok(())
    } else {
        Err(Fd001TenantEgressPolicyError::MissingRequiredControl(
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
