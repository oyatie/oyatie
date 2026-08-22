//! FD-001 tenant secret-boundary contract for later Oyatie Cloud dogfooding.
//!
//! This review-only crate defines the sensitive-material guardrails that must
//! exist before FD-001 Tenant RBAC, HR, Payroll, and Accounting workloads
//! can be promoted as production tenant workloads on the future Oyatie Cloud
//! substrate. It binds the FD-001 tenant-workload manifest and tenant admission
//! policy to Kubernetes Secret references, no-inline-secret requirements,
//! at-rest encryption evidence, RBAC least privilege, namespace isolation,
//! workload-scoped ServiceAccounts, short-lived projected-token boundaries,
//! external store handoff refs, rotation evidence, and access-audit evidence. It
//! does not create Kubernetes Secrets, materialize secret values, attach an
//! encryption provider, attach an external secret store runtime, apply RBAC,
//! mount projected tokens, run rotation, audit secret access at runtime, deploy
//! workloads, attach a cloud substrate runtime, or emit runtime audit-chain
//! events.
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
const MIN_REQUIREMENT_COUNT: usize = 11;
const CONTRACT_NAME: &str = "fd001-tenant-rbac-tenant-secret-boundary-contract";
const PROGRAM_NAME: &str = "fd-001-tenant-rbac-generic";
const SUBSTRATE_NAME: &str = "oyatie-cloud";
const TENANT_NAMESPACE: &str = "oyatie-fd001-tenant-rbac-dev";
const SOURCE_MANIFEST_REF: &str =
    "crates/tenant-rbac-tenant-workload-manifest/src/lib.rs::fd001_tenant_workload_manifest";
const SOURCE_ADMISSION_POLICY_REF: &str = "crates/tenant-rbac-tenant-admission-policy/src/lib.rs::fd001_tenant_admission_policy_contract";
const BOUNDARY_REF: &str = "secret-boundary/fd001-tenant-rbac/all-manifest-workloads";
const POLICY_REF_PREFIX: &str = "policy/secret-boundary/fd001/";
const EVIDENCE_REF: &str =
    "evidence/secret-boundary/fd001-tenant-rbac/secret-boundary-review.jsonl";

const KUBERNETES_SECRETS_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/configuration/secret/";
const KUBERNETES_SECRETS_GOOD_PRACTICES_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/security/secrets-good-practices/";
const KUBERNETES_ENCRYPT_DATA_DOC_URL: &str =
    "https://kubernetes.io/docs/tasks/administer-cluster/encrypt-data/";
const KUBERNETES_RBAC_GOOD_PRACTICES_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/security/rbac-good-practices/";
const KUBERNETES_RBAC_REFERENCE_DOC_URL: &str =
    "https://kubernetes.io/docs/reference/access-authn-authz/rbac/";
const KUBERNETES_SERVICE_ACCOUNTS_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/security/service-accounts/";
const KUBERNETES_PROJECTED_VOLUMES_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/storage/projected-volumes/";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Fd001TenantSecretBoundaryControlKind {
    NoInlineSecretMaterial,
    KubernetesSecretReferenceRequired,
    SecretAtRestEncryptionRequired,
    RbacLeastPrivilegeRequired,
    NamespaceSecretIsolationRequired,
    WorkloadScopedServiceAccountRequired,
    AutomountServiceAccountTokenDisabled,
    ShortLivedProjectedTokenBoundaryRequired,
    ExternalSecretStoreBoundaryRequired,
    SecretRotationEvidenceRequired,
    SecretAccessAuditEvidenceRequired,
}

impl Fd001TenantSecretBoundaryControlKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoInlineSecretMaterial => "no_inline_secret_material",
            Self::KubernetesSecretReferenceRequired => "kubernetes_secret_reference_required",
            Self::SecretAtRestEncryptionRequired => "secret_at_rest_encryption_required",
            Self::RbacLeastPrivilegeRequired => "rbac_least_privilege_required",
            Self::NamespaceSecretIsolationRequired => "namespace_secret_isolation_required",
            Self::WorkloadScopedServiceAccountRequired => {
                "workload_scoped_service_account_required"
            }
            Self::AutomountServiceAccountTokenDisabled => {
                "automount_service_account_token_disabled"
            }
            Self::ShortLivedProjectedTokenBoundaryRequired => {
                "short_lived_projected_token_boundary_required"
            }
            Self::ExternalSecretStoreBoundaryRequired => "external_secret_store_boundary_required",
            Self::SecretRotationEvidenceRequired => "secret_rotation_evidence_required",
            Self::SecretAccessAuditEvidenceRequired => "secret_access_audit_evidence_required",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fd001TenantSecretBoundaryRequirement {
    pub requirement_id: &'static str,           // data_class: PUBLIC
    pub workload_kind: Fd001TenantWorkloadKind, // data_class: PUBLIC
    pub control_kind: Fd001TenantSecretBoundaryControlKind, // data_class: PUBLIC
    pub boundary_ref: &'static str,             // data_class: INTERNAL_ONLY
    pub policy_ref: &'static str,               // data_class: INTERNAL_ONLY
    pub expected_evidence_ref: &'static str,    // data_class: INTERNAL_ONLY
    pub official_doc_url: &'static str,         // data_class: PUBLIC
    pub source_manifest_ref: &'static str,      // data_class: INTERNAL_ONLY
    pub source_admission_policy_ref: &'static str, // data_class: INTERNAL_ONLY
    pub applies_to_all_manifest_workloads: bool, // data_class: PUBLIC
    pub runtime_secret_material_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fd001TenantSecretBoundaryContract {
    pub contract_name: &'static str,          // data_class: PUBLIC
    pub program_name: &'static str,           // data_class: PUBLIC
    pub substrate_name: &'static str,         // data_class: PUBLIC
    pub tenant_namespace: &'static str,       // data_class: INTERNAL_ONLY
    pub workload_manifest_name: &'static str, // data_class: PUBLIC
    pub workload_manifest_count: usize,       // data_class: PUBLIC
    pub tenant_admission_policy_contract_name: &'static str, // data_class: PUBLIC
    pub requirements: Vec<Fd001TenantSecretBoundaryRequirement>, // data_class: INTERNAL_ONLY
    pub official_docs_required: bool,         // data_class: PUBLIC
    pub all_manifest_workloads_in_scope: bool, // data_class: PUBLIC
    pub inline_secret_material_forbidden: bool, // data_class: PUBLIC
    pub kubernetes_secret_reference_required: bool, // data_class: PUBLIC
    pub secret_at_rest_encryption_required: bool, // data_class: PUBLIC
    pub rbac_least_privilege_required: bool,  // data_class: PUBLIC
    pub namespace_secret_isolation_required: bool, // data_class: PUBLIC
    pub workload_scoped_service_account_required: bool, // data_class: PUBLIC
    pub automount_service_account_token_forbidden: bool, // data_class: PUBLIC
    pub short_lived_projected_token_boundary_required: bool, // data_class: PUBLIC
    pub external_secret_store_boundary_required: bool, // data_class: PUBLIC
    pub secret_rotation_evidence_required: bool, // data_class: PUBLIC
    pub secret_access_audit_evidence_required: bool, // data_class: PUBLIC
    pub review_only_contract: bool,           // data_class: PUBLIC
    pub kubernetes_secret_created: bool,      // data_class: INTERNAL_ONLY
    pub secret_data_materialized: bool,       // data_class: INTERNAL_ONLY
    pub encryption_provider_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub external_secret_store_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub rbac_runtime_applied: bool,           // data_class: INTERNAL_ONLY
    pub projected_token_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub secret_rotation_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub secret_access_runtime_audited: bool,  // data_class: INTERNAL_ONLY
    pub admission_controller_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub workload_runtime_deployed: bool,      // data_class: INTERNAL_ONLY
    pub cloud_substrate_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Fd001TenantSecretBoundaryError {
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
    MissingControlKind(Fd001TenantSecretBoundaryControlKind),
    InvalidRequirementId,
    InvalidBoundaryRef,
    InvalidPolicyRef,
    InvalidExpectedEvidenceRef,
    InvalidOfficialDocUrl,
    InvalidSourceManifestRef,
    InvalidSourceAdmissionPolicyRef,
    MissingRequiredControl(&'static str),
    RuntimeAttachmentOverclaim,
}

pub fn fd001_tenant_secret_boundary_contract()
-> Result<Fd001TenantSecretBoundaryContract, Fd001TenantSecretBoundaryError> {
    let manifest = fd001_tenant_workload_manifest();
    validate_fd001_tenant_workload_manifest(&manifest)
        .map_err(Fd001TenantSecretBoundaryError::WorkloadManifest)?;
    let admission_policy = fd001_tenant_admission_policy_contract()
        .map_err(Fd001TenantSecretBoundaryError::TenantAdmissionPolicy)?;
    validate_fd001_tenant_admission_policy_contract(&admission_policy)
        .map_err(Fd001TenantSecretBoundaryError::TenantAdmissionPolicy)?;

    Ok(Fd001TenantSecretBoundaryContract {
        contract_name: CONTRACT_NAME,
        program_name: PROGRAM_NAME,
        substrate_name: SUBSTRATE_NAME,
        tenant_namespace: manifest.tenant_namespace,
        workload_manifest_name: manifest.manifest_name,
        workload_manifest_count: manifest.workloads.len(),
        tenant_admission_policy_contract_name: admission_policy.contract_name,
        requirements: secret_boundary_requirements(),
        official_docs_required: true,
        all_manifest_workloads_in_scope: true,
        inline_secret_material_forbidden: true,
        kubernetes_secret_reference_required: true,
        secret_at_rest_encryption_required: true,
        rbac_least_privilege_required: true,
        namespace_secret_isolation_required: true,
        workload_scoped_service_account_required: true,
        automount_service_account_token_forbidden: true,
        short_lived_projected_token_boundary_required: true,
        external_secret_store_boundary_required: true,
        secret_rotation_evidence_required: true,
        secret_access_audit_evidence_required: true,
        review_only_contract: true,
        kubernetes_secret_created: false,
        secret_data_materialized: false,
        encryption_provider_runtime_attached: false,
        external_secret_store_runtime_attached: false,
        rbac_runtime_applied: false,
        projected_token_runtime_attached: false,
        secret_rotation_runtime_attached: false,
        secret_access_runtime_audited: false,
        admission_controller_runtime_attached: false,
        workload_runtime_deployed: false,
        cloud_substrate_runtime_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: SCHEMA_VERSION,
    })
}

pub fn validate_fd001_tenant_secret_boundary_contract(
    contract: &Fd001TenantSecretBoundaryContract,
) -> Result<(), Fd001TenantSecretBoundaryError> {
    let manifest = fd001_tenant_workload_manifest();
    validate_fd001_tenant_workload_manifest(&manifest)
        .map_err(Fd001TenantSecretBoundaryError::WorkloadManifest)?;
    let admission_policy = fd001_tenant_admission_policy_contract()
        .map_err(Fd001TenantSecretBoundaryError::TenantAdmissionPolicy)?;
    validate_fd001_tenant_admission_policy_contract(&admission_policy)
        .map_err(Fd001TenantSecretBoundaryError::TenantAdmissionPolicy)?;

    validate_slug(
        contract.contract_name,
        Fd001TenantSecretBoundaryError::InvalidContractName,
    )?;
    if contract.program_name != PROGRAM_NAME {
        return Err(Fd001TenantSecretBoundaryError::InvalidProgramName);
    }
    if contract.substrate_name != SUBSTRATE_NAME {
        return Err(Fd001TenantSecretBoundaryError::InvalidSubstrateName);
    }
    validate_tenant_namespace(contract.tenant_namespace)?;
    if contract.workload_manifest_name != manifest.manifest_name {
        return Err(Fd001TenantSecretBoundaryError::InvalidManifestName);
    }
    if contract.workload_manifest_count != manifest.workloads.len() {
        return Err(Fd001TenantSecretBoundaryError::InvalidWorkloadCount);
    }
    if contract.tenant_admission_policy_contract_name != admission_policy.contract_name {
        return Err(Fd001TenantSecretBoundaryError::InvalidAdmissionPolicyContractName);
    }
    if contract.requirements.len() < MIN_REQUIREMENT_COUNT
        || contract.schema_version != SCHEMA_VERSION
    {
        return Err(Fd001TenantSecretBoundaryError::MissingRequirements);
    }
    validate_required_controls(contract)?;
    validate_nonclaims(contract)?;
    validate_requirements(contract)?;
    Ok(())
}

pub fn fd001_tenant_secret_boundary_doc_urls(
    contract: &Fd001TenantSecretBoundaryContract,
) -> Vec<&'static str> {
    let mut docs = BTreeSet::new();
    for requirement in &contract.requirements {
        docs.insert(requirement.official_doc_url);
    }
    docs.into_iter().collect()
}

fn secret_boundary_requirements() -> Vec<Fd001TenantSecretBoundaryRequirement> {
    vec![
        requirement(
            "no-inline-secret-material-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantSecretBoundaryControlKind::NoInlineSecretMaterial,
            "no-inline-secret-material",
            KUBERNETES_SECRETS_GOOD_PRACTICES_DOC_URL,
        ),
        requirement(
            "kubernetes-secret-reference-required-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantSecretBoundaryControlKind::KubernetesSecretReferenceRequired,
            "kubernetes-secret-reference-required",
            KUBERNETES_SECRETS_DOC_URL,
        ),
        requirement(
            "secret-at-rest-encryption-required-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantSecretBoundaryControlKind::SecretAtRestEncryptionRequired,
            "secret-at-rest-encryption-required",
            KUBERNETES_ENCRYPT_DATA_DOC_URL,
        ),
        requirement(
            "rbac-least-privilege-required-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantSecretBoundaryControlKind::RbacLeastPrivilegeRequired,
            "rbac-least-privilege-required",
            KUBERNETES_RBAC_GOOD_PRACTICES_DOC_URL,
        ),
        requirement(
            "namespace-secret-isolation-required-all-workloads",
            Fd001TenantWorkloadKind::AccountingJournal,
            Fd001TenantSecretBoundaryControlKind::NamespaceSecretIsolationRequired,
            "namespace-secret-isolation-required",
            KUBERNETES_RBAC_REFERENCE_DOC_URL,
        ),
        requirement(
            "workload-scoped-service-account-required-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantSecretBoundaryControlKind::WorkloadScopedServiceAccountRequired,
            "workload-scoped-service-account-required",
            KUBERNETES_SERVICE_ACCOUNTS_DOC_URL,
        ),
        requirement(
            "automount-service-account-token-disabled-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantSecretBoundaryControlKind::AutomountServiceAccountTokenDisabled,
            "automount-service-account-token-disabled",
            KUBERNETES_SERVICE_ACCOUNTS_DOC_URL,
        ),
        requirement(
            "short-lived-projected-token-boundary-required-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantSecretBoundaryControlKind::ShortLivedProjectedTokenBoundaryRequired,
            "short-lived-projected-token-boundary-required",
            KUBERNETES_PROJECTED_VOLUMES_DOC_URL,
        ),
        requirement(
            "external-secret-store-boundary-required-all-workloads",
            Fd001TenantWorkloadKind::AccountingJournal,
            Fd001TenantSecretBoundaryControlKind::ExternalSecretStoreBoundaryRequired,
            "external-secret-store-boundary-required",
            KUBERNETES_SECRETS_DOC_URL,
        ),
        requirement(
            "secret-rotation-evidence-required-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantSecretBoundaryControlKind::SecretRotationEvidenceRequired,
            "secret-rotation-evidence-required",
            KUBERNETES_ENCRYPT_DATA_DOC_URL,
        ),
        requirement(
            "secret-access-audit-evidence-required-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantSecretBoundaryControlKind::SecretAccessAuditEvidenceRequired,
            "secret-access-audit-evidence-required",
            KUBERNETES_RBAC_GOOD_PRACTICES_DOC_URL,
        ),
    ]
}

fn requirement(
    requirement_id: &'static str,
    workload_kind: Fd001TenantWorkloadKind,
    control_kind: Fd001TenantSecretBoundaryControlKind,
    policy_suffix: &'static str,
    official_doc_url: &'static str,
) -> Fd001TenantSecretBoundaryRequirement {
    Fd001TenantSecretBoundaryRequirement {
        requirement_id,
        workload_kind,
        control_kind,
        boundary_ref: BOUNDARY_REF,
        policy_ref: policy_ref(policy_suffix),
        expected_evidence_ref: EVIDENCE_REF,
        official_doc_url,
        source_manifest_ref: SOURCE_MANIFEST_REF,
        source_admission_policy_ref: SOURCE_ADMISSION_POLICY_REF,
        applies_to_all_manifest_workloads: true,
        runtime_secret_material_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

fn policy_ref(policy_suffix: &'static str) -> &'static str {
    match policy_suffix {
        "no-inline-secret-material" => "policy/secret-boundary/fd001/no-inline-secret-material",
        "kubernetes-secret-reference-required" => {
            "policy/secret-boundary/fd001/kubernetes-secret-reference-required"
        }
        "secret-at-rest-encryption-required" => {
            "policy/secret-boundary/fd001/secret-at-rest-encryption-required"
        }
        "rbac-least-privilege-required" => {
            "policy/secret-boundary/fd001/rbac-least-privilege-required"
        }
        "namespace-secret-isolation-required" => {
            "policy/secret-boundary/fd001/namespace-secret-isolation-required"
        }
        "workload-scoped-service-account-required" => {
            "policy/secret-boundary/fd001/workload-scoped-service-account-required"
        }
        "automount-service-account-token-disabled" => {
            "policy/secret-boundary/fd001/automount-service-account-token-disabled"
        }
        "short-lived-projected-token-boundary-required" => {
            "policy/secret-boundary/fd001/short-lived-projected-token-boundary-required"
        }
        "external-secret-store-boundary-required" => {
            "policy/secret-boundary/fd001/external-secret-store-boundary-required"
        }
        "secret-rotation-evidence-required" => {
            "policy/secret-boundary/fd001/secret-rotation-evidence-required"
        }
        "secret-access-audit-evidence-required" => {
            "policy/secret-boundary/fd001/secret-access-audit-evidence-required"
        }
        _ => "policy/secret-boundary/fd001/invalid",
    }
}

fn validate_required_controls(
    contract: &Fd001TenantSecretBoundaryContract,
) -> Result<(), Fd001TenantSecretBoundaryError> {
    for (enabled, name) in [
        (contract.official_docs_required, "official_docs_required"),
        (
            contract.all_manifest_workloads_in_scope,
            "all_manifest_workloads_in_scope",
        ),
        (
            contract.inline_secret_material_forbidden,
            "inline_secret_material_forbidden",
        ),
        (
            contract.kubernetes_secret_reference_required,
            "kubernetes_secret_reference_required",
        ),
        (
            contract.secret_at_rest_encryption_required,
            "secret_at_rest_encryption_required",
        ),
        (
            contract.rbac_least_privilege_required,
            "rbac_least_privilege_required",
        ),
        (
            contract.namespace_secret_isolation_required,
            "namespace_secret_isolation_required",
        ),
        (
            contract.workload_scoped_service_account_required,
            "workload_scoped_service_account_required",
        ),
        (
            contract.automount_service_account_token_forbidden,
            "automount_service_account_token_forbidden",
        ),
        (
            contract.short_lived_projected_token_boundary_required,
            "short_lived_projected_token_boundary_required",
        ),
        (
            contract.external_secret_store_boundary_required,
            "external_secret_store_boundary_required",
        ),
        (
            contract.secret_rotation_evidence_required,
            "secret_rotation_evidence_required",
        ),
        (
            contract.secret_access_audit_evidence_required,
            "secret_access_audit_evidence_required",
        ),
        (contract.review_only_contract, "review_only_contract"),
    ] {
        require_control(enabled, name)?;
    }
    Ok(())
}

fn validate_nonclaims(
    contract: &Fd001TenantSecretBoundaryContract,
) -> Result<(), Fd001TenantSecretBoundaryError> {
    if contract.kubernetes_secret_created
        || contract.secret_data_materialized
        || contract.encryption_provider_runtime_attached
        || contract.external_secret_store_runtime_attached
        || contract.rbac_runtime_applied
        || contract.projected_token_runtime_attached
        || contract.secret_rotation_runtime_attached
        || contract.secret_access_runtime_audited
        || contract.admission_controller_runtime_attached
        || contract.workload_runtime_deployed
        || contract.cloud_substrate_runtime_attached
        || contract.runtime_audit_chain_emission_attached
    {
        return Err(Fd001TenantSecretBoundaryError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_requirements(
    contract: &Fd001TenantSecretBoundaryContract,
) -> Result<(), Fd001TenantSecretBoundaryError> {
    let mut seen_requirements = BTreeSet::new();
    let mut seen_workloads = BTreeSet::new();
    let mut seen_control_kinds = BTreeSet::new();
    for requirement in &contract.requirements {
        validate_requirement(requirement)?;
        if !seen_requirements.insert(requirement.requirement_id) {
            return Err(Fd001TenantSecretBoundaryError::DuplicateRequirement(
                requirement.requirement_id.to_owned(),
            ));
        }
        seen_workloads.insert(requirement.workload_kind);
        seen_control_kinds.insert(requirement.control_kind);
    }
    for workload_kind in required_workload_kinds() {
        if !seen_workloads.contains(&workload_kind) {
            return Err(Fd001TenantSecretBoundaryError::MissingWorkloadKind(
                workload_kind,
            ));
        }
    }
    for control_kind in required_control_kinds() {
        if !seen_control_kinds.contains(&control_kind) {
            return Err(Fd001TenantSecretBoundaryError::MissingControlKind(
                control_kind,
            ));
        }
    }
    Ok(())
}

fn validate_requirement(
    requirement: &Fd001TenantSecretBoundaryRequirement,
) -> Result<(), Fd001TenantSecretBoundaryError> {
    validate_slug(
        requirement.requirement_id,
        Fd001TenantSecretBoundaryError::InvalidRequirementId,
    )?;
    validate_prefixed_ref(
        requirement.boundary_ref,
        "secret-boundary/fd001-tenant-rbac/",
        Fd001TenantSecretBoundaryError::InvalidBoundaryRef,
    )?;
    validate_prefixed_ref(
        requirement.policy_ref,
        POLICY_REF_PREFIX,
        Fd001TenantSecretBoundaryError::InvalidPolicyRef,
    )?;
    validate_prefixed_ref(
        requirement.expected_evidence_ref,
        "evidence/secret-boundary/fd001-tenant-rbac/",
        Fd001TenantSecretBoundaryError::InvalidExpectedEvidenceRef,
    )?;
    validate_doc_url(requirement.official_doc_url)?;
    validate_prefixed_ref(
        requirement.source_manifest_ref,
        "crates/tenant-rbac-tenant-workload-manifest/",
        Fd001TenantSecretBoundaryError::InvalidSourceManifestRef,
    )?;
    validate_prefixed_ref(
        requirement.source_admission_policy_ref,
        "crates/tenant-rbac-tenant-admission-policy/",
        Fd001TenantSecretBoundaryError::InvalidSourceAdmissionPolicyRef,
    )?;
    if !requirement.applies_to_all_manifest_workloads {
        return Err(Fd001TenantSecretBoundaryError::MissingRequiredControl(
            "requirement_applies_to_all_manifest_workloads",
        ));
    }
    if requirement.runtime_secret_material_attached || requirement.schema_version != SCHEMA_VERSION
    {
        return Err(Fd001TenantSecretBoundaryError::RuntimeAttachmentOverclaim);
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

fn required_control_kinds() -> [Fd001TenantSecretBoundaryControlKind; 11] {
    [
        Fd001TenantSecretBoundaryControlKind::NoInlineSecretMaterial,
        Fd001TenantSecretBoundaryControlKind::KubernetesSecretReferenceRequired,
        Fd001TenantSecretBoundaryControlKind::SecretAtRestEncryptionRequired,
        Fd001TenantSecretBoundaryControlKind::RbacLeastPrivilegeRequired,
        Fd001TenantSecretBoundaryControlKind::NamespaceSecretIsolationRequired,
        Fd001TenantSecretBoundaryControlKind::WorkloadScopedServiceAccountRequired,
        Fd001TenantSecretBoundaryControlKind::AutomountServiceAccountTokenDisabled,
        Fd001TenantSecretBoundaryControlKind::ShortLivedProjectedTokenBoundaryRequired,
        Fd001TenantSecretBoundaryControlKind::ExternalSecretStoreBoundaryRequired,
        Fd001TenantSecretBoundaryControlKind::SecretRotationEvidenceRequired,
        Fd001TenantSecretBoundaryControlKind::SecretAccessAuditEvidenceRequired,
    ]
}

fn validate_doc_url(url: &str) -> Result<(), Fd001TenantSecretBoundaryError> {
    if has_unsafe_ref_text(url)
        || ![
            KUBERNETES_SECRETS_DOC_URL,
            KUBERNETES_SECRETS_GOOD_PRACTICES_DOC_URL,
            KUBERNETES_ENCRYPT_DATA_DOC_URL,
            KUBERNETES_RBAC_GOOD_PRACTICES_DOC_URL,
            KUBERNETES_RBAC_REFERENCE_DOC_URL,
            KUBERNETES_SERVICE_ACCOUNTS_DOC_URL,
            KUBERNETES_PROJECTED_VOLUMES_DOC_URL,
        ]
        .contains(&url)
    {
        return Err(Fd001TenantSecretBoundaryError::InvalidOfficialDocUrl);
    }
    Ok(())
}

fn validate_slug(
    value: &str,
    error: Fd001TenantSecretBoundaryError,
) -> Result<(), Fd001TenantSecretBoundaryError> {
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

fn validate_tenant_namespace(value: &str) -> Result<(), Fd001TenantSecretBoundaryError> {
    validate_slug(
        value,
        Fd001TenantSecretBoundaryError::InvalidTenantNamespace,
    )?;
    if value != TENANT_NAMESPACE {
        return Err(Fd001TenantSecretBoundaryError::InvalidTenantNamespace);
    }
    Ok(())
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: Fd001TenantSecretBoundaryError,
) -> Result<(), Fd001TenantSecretBoundaryError> {
    if value.len() <= prefix.len() || !value.starts_with(prefix) || has_unsafe_ref_text(value) {
        return Err(error);
    }
    Ok(())
}

fn require_control(
    value: bool,
    control: &'static str,
) -> Result<(), Fd001TenantSecretBoundaryError> {
    if value {
        Ok(())
    } else {
        Err(Fd001TenantSecretBoundaryError::MissingRequiredControl(
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
        || lower.contains("credential")
        || lower.contains("api_key")
        || lower.contains("bearer")
}
