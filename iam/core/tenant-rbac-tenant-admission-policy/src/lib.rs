//! FD-001 tenant admission policy contract for later Oyatie Cloud dogfooding.
//!
//! This review-only crate defines the Kubernetes admission guardrails that the
//! future Oyatie Cloud substrate must enforce before FD-001 Tenant RBAC,
//! HR, Payroll, and Accounting workloads can be promoted as production tenant
//! workloads. It binds the existing FD-001 tenant-workload manifest to
//! ValidatingAdmissionPolicy, Pod Security Admission, image digest, resource,
//! service-account, and default-deny network policy requirements. It does not
//! connect to a Kubernetes cluster, install admission policies, enforce runtime
//! admission, deploy workloads, attach a cloud substrate runtime, or emit
//! runtime audit-chain events.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use iam_tenant_rbac_tenant_workload_manifest::{
    Fd001TenantWorkloadKind, Fd001TenantWorkloadManifestError, fd001_tenant_workload_manifest,
    validate_fd001_tenant_workload_manifest,
};

const SCHEMA_VERSION: u32 = 1;
const MIN_RULE_COUNT: usize = 10;
const CONTRACT_NAME: &str = "fd001-tenant-rbac-tenant-admission-policy-contract";
const PROGRAM_NAME: &str = "fd-001-tenant-rbac-generic";
const SUBSTRATE_NAME: &str = "oyatie-cloud";
const TENANT_NAMESPACE: &str = "oyatie-fd001-tenant-rbac-dev";
const SOURCE_MANIFEST_REF: &str =
    "crates/tenant-rbac-tenant-workload-manifest/src/lib.rs::fd001_tenant_workload_manifest";
const POLICY_API_KIND: &str = "ValidatingAdmissionPolicy";
const BINDING_API_KIND: &str = "ValidatingAdmissionPolicyBinding";
const FAILURE_POLICY: &str = "Fail";
const VALIDATION_ACTION: &str = "Deny";

const VALIDATING_ADMISSION_POLICY_DOC_URL: &str =
    "https://kubernetes.io/docs/reference/access-authn-authz/validating-admission-policy/";
const VALIDATING_ADMISSION_POLICY_API_DOC_URL: &str = "https://kubernetes.io/docs/reference/kubernetes-api/admissionregistration/validating-admission-policy-v1/";
const POD_SECURITY_ADMISSION_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/security/pod-security-admission/";
const POD_SECURITY_STANDARDS_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/security/pod-security-standards/";
const CONTAINER_IMAGES_DOC_URL: &str = "https://kubernetes.io/docs/concepts/containers/images/";
const RESOURCE_MANAGEMENT_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/";
const SERVICE_ACCOUNTS_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/security/service-accounts/";
const NETWORK_POLICIES_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/services-networking/network-policies/";
const RESOURCE_QUOTAS_DOC_URL: &str = "https://kubernetes.io/docs/concepts/policy/resource-quotas/";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Fd001TenantAdmissionRuleKind {
    TenantLabelRequired,
    DigestPinnedImageRequired,
    LatestImageTagForbidden,
    ResourceRequestsLimitsRequired,
    ServiceAccountRequired,
    DefaultServiceAccountForbidden,
    AutomountServiceAccountTokenDisabled,
    PodSecurityRestrictedNamespaceRequired,
    ResourceQuotaRequired,
    NetworkPolicyDefaultDenyRequired,
    AdmissionAuditAnnotationRequired,
}

impl Fd001TenantAdmissionRuleKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TenantLabelRequired => "tenant_label_required",
            Self::DigestPinnedImageRequired => "digest_pinned_image_required",
            Self::LatestImageTagForbidden => "latest_image_tag_forbidden",
            Self::ResourceRequestsLimitsRequired => "resource_requests_limits_required",
            Self::ServiceAccountRequired => "service_account_required",
            Self::DefaultServiceAccountForbidden => "default_service_account_forbidden",
            Self::AutomountServiceAccountTokenDisabled => {
                "automount_service_account_token_disabled"
            }
            Self::PodSecurityRestrictedNamespaceRequired => {
                "pod_security_restricted_namespace_required"
            }
            Self::ResourceQuotaRequired => "resource_quota_required",
            Self::NetworkPolicyDefaultDenyRequired => "network_policy_default_deny_required",
            Self::AdmissionAuditAnnotationRequired => "admission_audit_annotation_required",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fd001TenantAdmissionRule {
    pub rule_id: &'static str,                   // data_class: PUBLIC
    pub workload_kind: Fd001TenantWorkloadKind,  // data_class: PUBLIC
    pub rule_kind: Fd001TenantAdmissionRuleKind, // data_class: PUBLIC
    pub policy_api_kind: &'static str,           // data_class: PUBLIC
    pub binding_api_kind: &'static str,          // data_class: PUBLIC
    pub failure_policy: &'static str,            // data_class: PUBLIC
    pub validation_action: &'static str,         // data_class: PUBLIC
    pub cel_expression_ref: &'static str,        // data_class: INTERNAL_ONLY
    pub expected_evidence_ref: &'static str,     // data_class: INTERNAL_ONLY
    pub official_doc_url: &'static str,          // data_class: PUBLIC
    pub source_manifest_ref: &'static str,       // data_class: INTERNAL_ONLY
    pub applies_to_all_manifest_workloads: bool, // data_class: PUBLIC
    pub requires_audit_annotation: bool,         // data_class: PUBLIC
    pub runtime_enforcement_attached: bool,      // data_class: INTERNAL_ONLY
    pub schema_version: u32,                     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fd001TenantAdmissionPolicyContract {
    pub contract_name: &'static str,                // data_class: PUBLIC
    pub program_name: &'static str,                 // data_class: PUBLIC
    pub substrate_name: &'static str,               // data_class: PUBLIC
    pub tenant_namespace: &'static str,             // data_class: INTERNAL_ONLY
    pub workload_manifest_name: &'static str,       // data_class: PUBLIC
    pub workload_manifest_count: usize,             // data_class: PUBLIC
    pub all_manifest_workloads_in_scope: bool,      // data_class: PUBLIC
    pub rules: Vec<Fd001TenantAdmissionRule>,       // data_class: INTERNAL_ONLY
    pub official_docs_required: bool,               // data_class: PUBLIC
    pub validating_admission_policy_required: bool, // data_class: PUBLIC
    pub admission_binding_required: bool,           // data_class: PUBLIC
    pub failure_policy_fail_required: bool,         // data_class: PUBLIC
    pub deny_action_required: bool,                 // data_class: PUBLIC
    pub pod_security_restricted_required: bool,     // data_class: PUBLIC
    pub digest_pinned_image_required: bool,         // data_class: PUBLIC
    pub latest_image_tag_forbidden: bool,           // data_class: PUBLIC
    pub tenant_labels_required: bool,               // data_class: PUBLIC
    pub resource_requests_limits_required: bool,    // data_class: PUBLIC
    pub service_account_boundary_required: bool,    // data_class: PUBLIC
    pub default_service_account_forbidden: bool,    // data_class: PUBLIC
    pub automount_service_account_token_forbidden: bool, // data_class: PUBLIC
    pub resource_quota_required: bool,              // data_class: PUBLIC
    pub network_policy_default_deny_required: bool, // data_class: PUBLIC
    pub admission_audit_annotation_required: bool,  // data_class: PUBLIC
    pub review_only_contract: bool,                 // data_class: PUBLIC
    pub kubernetes_cluster_attached: bool,          // data_class: INTERNAL_ONLY
    pub admission_controller_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub admission_policy_applied: bool,             // data_class: INTERNAL_ONLY
    pub admission_runtime_enforced: bool,           // data_class: INTERNAL_ONLY
    pub workload_runtime_deployed: bool,            // data_class: INTERNAL_ONLY
    pub cloud_substrate_runtime_attached: bool,     // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                        // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Fd001TenantAdmissionPolicyError {
    WorkloadManifest(Fd001TenantWorkloadManifestError),
    InvalidContractName,
    InvalidProgramName,
    InvalidSubstrateName,
    InvalidTenantNamespace,
    InvalidManifestName,
    InvalidWorkloadCount,
    MissingRules,
    DuplicateRule(String),
    MissingWorkloadKind(Fd001TenantWorkloadKind),
    MissingRuleKind(Fd001TenantAdmissionRuleKind),
    InvalidRuleId,
    InvalidPolicyApiKind,
    InvalidBindingApiKind,
    InvalidFailurePolicy,
    InvalidValidationAction,
    InvalidCelExpressionRef,
    InvalidExpectedEvidenceRef,
    InvalidOfficialDocUrl,
    InvalidSourceManifestRef,
    MissingRequiredControl(&'static str),
    RuntimeAttachmentOverclaim,
}

pub fn fd001_tenant_admission_policy_contract()
-> Result<Fd001TenantAdmissionPolicyContract, Fd001TenantAdmissionPolicyError> {
    let manifest = fd001_tenant_workload_manifest();
    validate_fd001_tenant_workload_manifest(&manifest)
        .map_err(Fd001TenantAdmissionPolicyError::WorkloadManifest)?;

    Ok(Fd001TenantAdmissionPolicyContract {
        contract_name: CONTRACT_NAME,
        program_name: PROGRAM_NAME,
        substrate_name: SUBSTRATE_NAME,
        tenant_namespace: manifest.tenant_namespace,
        workload_manifest_name: manifest.manifest_name,
        workload_manifest_count: manifest.workloads.len(),
        all_manifest_workloads_in_scope: true,
        rules: admission_rules(),
        official_docs_required: true,
        validating_admission_policy_required: true,
        admission_binding_required: true,
        failure_policy_fail_required: true,
        deny_action_required: true,
        pod_security_restricted_required: true,
        digest_pinned_image_required: true,
        latest_image_tag_forbidden: true,
        tenant_labels_required: true,
        resource_requests_limits_required: true,
        service_account_boundary_required: true,
        default_service_account_forbidden: true,
        automount_service_account_token_forbidden: true,
        resource_quota_required: true,
        network_policy_default_deny_required: true,
        admission_audit_annotation_required: true,
        review_only_contract: true,
        kubernetes_cluster_attached: false,
        admission_controller_runtime_attached: false,
        admission_policy_applied: false,
        admission_runtime_enforced: false,
        workload_runtime_deployed: false,
        cloud_substrate_runtime_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: SCHEMA_VERSION,
    })
}

pub fn validate_fd001_tenant_admission_policy_contract(
    contract: &Fd001TenantAdmissionPolicyContract,
) -> Result<(), Fd001TenantAdmissionPolicyError> {
    let manifest = fd001_tenant_workload_manifest();
    validate_fd001_tenant_workload_manifest(&manifest)
        .map_err(Fd001TenantAdmissionPolicyError::WorkloadManifest)?;

    validate_slug(
        contract.contract_name,
        Fd001TenantAdmissionPolicyError::InvalidContractName,
    )?;
    if contract.program_name != PROGRAM_NAME {
        return Err(Fd001TenantAdmissionPolicyError::InvalidProgramName);
    }
    if contract.substrate_name != SUBSTRATE_NAME {
        return Err(Fd001TenantAdmissionPolicyError::InvalidSubstrateName);
    }
    validate_tenant_namespace(contract.tenant_namespace)?;
    if contract.workload_manifest_name != manifest.manifest_name {
        return Err(Fd001TenantAdmissionPolicyError::InvalidManifestName);
    }
    if contract.workload_manifest_count != manifest.workloads.len() {
        return Err(Fd001TenantAdmissionPolicyError::InvalidWorkloadCount);
    }
    if contract.rules.len() < MIN_RULE_COUNT || contract.schema_version != SCHEMA_VERSION {
        return Err(Fd001TenantAdmissionPolicyError::MissingRules);
    }
    validate_required_controls(contract)?;
    validate_nonclaims(contract)?;
    validate_rules(contract)?;
    Ok(())
}

pub fn fd001_tenant_admission_policy_doc_urls(
    contract: &Fd001TenantAdmissionPolicyContract,
) -> Vec<&'static str> {
    let mut docs = BTreeSet::new();
    for rule in &contract.rules {
        docs.insert(rule.official_doc_url);
    }
    docs.into_iter().collect()
}

fn admission_rules() -> Vec<Fd001TenantAdmissionRule> {
    vec![
        rule(
            "tenant-labels-required-tenant-rbac",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantAdmissionRuleKind::TenantLabelRequired,
            "cel/fd001/tenant-labels-required",
            VALIDATING_ADMISSION_POLICY_DOC_URL,
            true,
        ),
        rule(
            "digest-pinned-image-tenant-rbac",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantAdmissionRuleKind::DigestPinnedImageRequired,
            "cel/fd001/image-digest-pinned",
            CONTAINER_IMAGES_DOC_URL,
            true,
        ),
        rule(
            "latest-image-tag-forbidden-tenant-rbac",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantAdmissionRuleKind::LatestImageTagForbidden,
            "cel/fd001/latest-image-tag-forbidden",
            CONTAINER_IMAGES_DOC_URL,
            true,
        ),
        rule(
            "resource-requests-limits-required-hr",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantAdmissionRuleKind::ResourceRequestsLimitsRequired,
            "cel/fd001/resource-requests-limits-required",
            RESOURCE_MANAGEMENT_DOC_URL,
            true,
        ),
        rule(
            "service-account-required-payroll",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantAdmissionRuleKind::ServiceAccountRequired,
            "cel/fd001/service-account-required",
            SERVICE_ACCOUNTS_DOC_URL,
            true,
        ),
        rule(
            "default-service-account-forbidden-payroll",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantAdmissionRuleKind::DefaultServiceAccountForbidden,
            "cel/fd001/default-service-account-forbidden",
            SERVICE_ACCOUNTS_DOC_URL,
            true,
        ),
        rule(
            "automount-service-account-token-disabled-accounting",
            Fd001TenantWorkloadKind::AccountingJournal,
            Fd001TenantAdmissionRuleKind::AutomountServiceAccountTokenDisabled,
            "cel/fd001/automount-service-account-token-disabled",
            SERVICE_ACCOUNTS_DOC_URL,
            true,
        ),
        rule(
            "pod-security-restricted-namespace-fd001",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantAdmissionRuleKind::PodSecurityRestrictedNamespaceRequired,
            "namespace-label/fd001/pod-security-restricted",
            POD_SECURITY_ADMISSION_DOC_URL,
            true,
        ),
        rule(
            "resource-quota-required-fd001",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantAdmissionRuleKind::ResourceQuotaRequired,
            "cel/fd001/resource-quota-required",
            RESOURCE_QUOTAS_DOC_URL,
            true,
        ),
        rule(
            "network-policy-default-deny-required-fd001",
            Fd001TenantWorkloadKind::AccountingJournal,
            Fd001TenantAdmissionRuleKind::NetworkPolicyDefaultDenyRequired,
            "cel/fd001/network-policy-default-deny-required",
            NETWORK_POLICIES_DOC_URL,
            true,
        ),
        rule(
            "admission-audit-annotation-required-fd001",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantAdmissionRuleKind::AdmissionAuditAnnotationRequired,
            "cel/fd001/admission-audit-annotation-required",
            VALIDATING_ADMISSION_POLICY_API_DOC_URL,
            true,
        ),
    ]
}

fn rule(
    rule_id: &'static str,
    workload_kind: Fd001TenantWorkloadKind,
    rule_kind: Fd001TenantAdmissionRuleKind,
    cel_expression_ref: &'static str,
    official_doc_url: &'static str,
    requires_audit_annotation: bool,
) -> Fd001TenantAdmissionRule {
    Fd001TenantAdmissionRule {
        rule_id,
        workload_kind,
        rule_kind,
        policy_api_kind: POLICY_API_KIND,
        binding_api_kind: BINDING_API_KIND,
        failure_policy: FAILURE_POLICY,
        validation_action: VALIDATION_ACTION,
        cel_expression_ref,
        expected_evidence_ref: "evidence/tenant-admission/fd001-tenant-rbac/policy-review.jsonl",
        official_doc_url,
        source_manifest_ref: SOURCE_MANIFEST_REF,
        applies_to_all_manifest_workloads: true,
        requires_audit_annotation,
        runtime_enforcement_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

fn validate_required_controls(
    contract: &Fd001TenantAdmissionPolicyContract,
) -> Result<(), Fd001TenantAdmissionPolicyError> {
    for (enabled, name) in [
        (contract.official_docs_required, "official_docs_required"),
        (
            contract.validating_admission_policy_required,
            "validating_admission_policy_required",
        ),
        (
            contract.admission_binding_required,
            "admission_binding_required",
        ),
        (
            contract.failure_policy_fail_required,
            "failure_policy_fail_required",
        ),
        (contract.deny_action_required, "deny_action_required"),
        (
            contract.pod_security_restricted_required,
            "pod_security_restricted_required",
        ),
        (
            contract.digest_pinned_image_required,
            "digest_pinned_image_required",
        ),
        (
            contract.latest_image_tag_forbidden,
            "latest_image_tag_forbidden",
        ),
        (contract.tenant_labels_required, "tenant_labels_required"),
        (
            contract.resource_requests_limits_required,
            "resource_requests_limits_required",
        ),
        (
            contract.service_account_boundary_required,
            "service_account_boundary_required",
        ),
        (
            contract.default_service_account_forbidden,
            "default_service_account_forbidden",
        ),
        (
            contract.automount_service_account_token_forbidden,
            "automount_service_account_token_forbidden",
        ),
        (contract.resource_quota_required, "resource_quota_required"),
        (
            contract.network_policy_default_deny_required,
            "network_policy_default_deny_required",
        ),
        (
            contract.admission_audit_annotation_required,
            "admission_audit_annotation_required",
        ),
        (contract.review_only_contract, "review_only_contract"),
        (
            contract.all_manifest_workloads_in_scope,
            "all_manifest_workloads_in_scope",
        ),
    ] {
        require_control(enabled, name)?;
    }
    Ok(())
}

fn validate_nonclaims(
    contract: &Fd001TenantAdmissionPolicyContract,
) -> Result<(), Fd001TenantAdmissionPolicyError> {
    if contract.kubernetes_cluster_attached
        || contract.admission_controller_runtime_attached
        || contract.admission_policy_applied
        || contract.admission_runtime_enforced
        || contract.workload_runtime_deployed
        || contract.cloud_substrate_runtime_attached
        || contract.runtime_audit_chain_emission_attached
    {
        return Err(Fd001TenantAdmissionPolicyError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_rules(
    contract: &Fd001TenantAdmissionPolicyContract,
) -> Result<(), Fd001TenantAdmissionPolicyError> {
    let mut seen_rules = BTreeSet::new();
    let mut seen_workloads = BTreeSet::new();
    let mut seen_rule_kinds = BTreeSet::new();
    for rule in &contract.rules {
        validate_rule(rule)?;
        if !seen_rules.insert(rule.rule_id) {
            return Err(Fd001TenantAdmissionPolicyError::DuplicateRule(
                rule.rule_id.to_owned(),
            ));
        }
        seen_workloads.insert(rule.workload_kind);
        seen_rule_kinds.insert(rule.rule_kind);
    }
    for workload_kind in required_workload_kinds() {
        if !seen_workloads.contains(&workload_kind) {
            return Err(Fd001TenantAdmissionPolicyError::MissingWorkloadKind(
                workload_kind,
            ));
        }
    }
    for rule_kind in required_rule_kinds() {
        if !seen_rule_kinds.contains(&rule_kind) {
            return Err(Fd001TenantAdmissionPolicyError::MissingRuleKind(rule_kind));
        }
    }
    Ok(())
}

fn validate_rule(rule: &Fd001TenantAdmissionRule) -> Result<(), Fd001TenantAdmissionPolicyError> {
    validate_slug(rule.rule_id, Fd001TenantAdmissionPolicyError::InvalidRuleId)?;
    if rule.policy_api_kind != POLICY_API_KIND {
        return Err(Fd001TenantAdmissionPolicyError::InvalidPolicyApiKind);
    }
    if rule.binding_api_kind != BINDING_API_KIND {
        return Err(Fd001TenantAdmissionPolicyError::InvalidBindingApiKind);
    }
    if rule.failure_policy != FAILURE_POLICY {
        return Err(Fd001TenantAdmissionPolicyError::InvalidFailurePolicy);
    }
    if rule.validation_action != VALIDATION_ACTION {
        return Err(Fd001TenantAdmissionPolicyError::InvalidValidationAction);
    }
    validate_prefixed_ref(
        rule.cel_expression_ref,
        expected_rule_ref_prefix(rule.rule_kind),
        Fd001TenantAdmissionPolicyError::InvalidCelExpressionRef,
    )?;
    validate_prefixed_ref(
        rule.expected_evidence_ref,
        "evidence/tenant-admission/",
        Fd001TenantAdmissionPolicyError::InvalidExpectedEvidenceRef,
    )?;
    validate_doc_url(rule.official_doc_url)?;
    validate_prefixed_ref(
        rule.source_manifest_ref,
        "crates/tenant-rbac-tenant-workload-manifest/",
        Fd001TenantAdmissionPolicyError::InvalidSourceManifestRef,
    )?;
    if !rule.applies_to_all_manifest_workloads {
        return Err(Fd001TenantAdmissionPolicyError::MissingRequiredControl(
            "rule_applies_to_all_manifest_workloads",
        ));
    }
    if !rule.requires_audit_annotation {
        return Err(Fd001TenantAdmissionPolicyError::MissingRequiredControl(
            "rule_requires_audit_annotation",
        ));
    }
    if rule.runtime_enforcement_attached || rule.schema_version != SCHEMA_VERSION {
        return Err(Fd001TenantAdmissionPolicyError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn expected_rule_ref_prefix(rule_kind: Fd001TenantAdmissionRuleKind) -> &'static str {
    match rule_kind {
        Fd001TenantAdmissionRuleKind::PodSecurityRestrictedNamespaceRequired => {
            "namespace-label/fd001/"
        }
        _ => "cel/fd001/",
    }
}

fn required_workload_kinds() -> [Fd001TenantWorkloadKind; 4] {
    [
        Fd001TenantWorkloadKind::TenantRbac,
        Fd001TenantWorkloadKind::HrEmployment,
        Fd001TenantWorkloadKind::PayrollRun,
        Fd001TenantWorkloadKind::AccountingJournal,
    ]
}

fn required_rule_kinds() -> [Fd001TenantAdmissionRuleKind; 11] {
    [
        Fd001TenantAdmissionRuleKind::TenantLabelRequired,
        Fd001TenantAdmissionRuleKind::DigestPinnedImageRequired,
        Fd001TenantAdmissionRuleKind::LatestImageTagForbidden,
        Fd001TenantAdmissionRuleKind::ResourceRequestsLimitsRequired,
        Fd001TenantAdmissionRuleKind::ServiceAccountRequired,
        Fd001TenantAdmissionRuleKind::DefaultServiceAccountForbidden,
        Fd001TenantAdmissionRuleKind::AutomountServiceAccountTokenDisabled,
        Fd001TenantAdmissionRuleKind::PodSecurityRestrictedNamespaceRequired,
        Fd001TenantAdmissionRuleKind::ResourceQuotaRequired,
        Fd001TenantAdmissionRuleKind::NetworkPolicyDefaultDenyRequired,
        Fd001TenantAdmissionRuleKind::AdmissionAuditAnnotationRequired,
    ]
}

fn validate_doc_url(url: &str) -> Result<(), Fd001TenantAdmissionPolicyError> {
    if has_unsafe_ref_text(url)
        || ![
            VALIDATING_ADMISSION_POLICY_DOC_URL,
            VALIDATING_ADMISSION_POLICY_API_DOC_URL,
            POD_SECURITY_ADMISSION_DOC_URL,
            POD_SECURITY_STANDARDS_DOC_URL,
            CONTAINER_IMAGES_DOC_URL,
            RESOURCE_MANAGEMENT_DOC_URL,
            SERVICE_ACCOUNTS_DOC_URL,
            NETWORK_POLICIES_DOC_URL,
            RESOURCE_QUOTAS_DOC_URL,
        ]
        .contains(&url)
    {
        return Err(Fd001TenantAdmissionPolicyError::InvalidOfficialDocUrl);
    }
    Ok(())
}

fn validate_slug(
    value: &str,
    error: Fd001TenantAdmissionPolicyError,
) -> Result<(), Fd001TenantAdmissionPolicyError> {
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

fn validate_tenant_namespace(value: &str) -> Result<(), Fd001TenantAdmissionPolicyError> {
    validate_slug(
        value,
        Fd001TenantAdmissionPolicyError::InvalidTenantNamespace,
    )?;
    if value != TENANT_NAMESPACE {
        return Err(Fd001TenantAdmissionPolicyError::InvalidTenantNamespace);
    }
    Ok(())
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: Fd001TenantAdmissionPolicyError,
) -> Result<(), Fd001TenantAdmissionPolicyError> {
    if value.len() <= prefix.len() || !value.starts_with(prefix) || has_unsafe_ref_text(value) {
        return Err(error);
    }
    Ok(())
}

fn require_control(
    value: bool,
    control: &'static str,
) -> Result<(), Fd001TenantAdmissionPolicyError> {
    if value {
        Ok(())
    } else {
        Err(Fd001TenantAdmissionPolicyError::MissingRequiredControl(
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
