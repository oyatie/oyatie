//! FD-001 tenant workload-identity contract for later Oyatie Cloud dogfooding.
//!
//! This review-only crate defines service-to-service identity guardrails that
//! must exist before FD-001 Tenant RBAC, HR, Payroll, and Accounting
//! workloads can be promoted as production tenant workloads on the future
//! Oyatie Cloud substrate. It binds the FD-001 tenant-workload manifest and
//! tenant admission policy to SPIFFE IDs, pinned trust domains, X.509 SVIDs,
//! JWT-SVID policy, mutual TLS policy, Gateway API BackendTLSPolicy refs,
//! certificate-rotation evidence, trust-bundle evidence, Workload API
//! boundaries, workload attestation selectors, OpenTelemetry service identity,
//! authorization-policy binding, and identity audit evidence. It does not
//! attach a Kubernetes cluster, attach a SPIFFE Workload API, run SPIRE server
//! or agent processes, issue SVIDs, observe mTLS handshakes, rotate
//! certificates at runtime, apply BackendTLSPolicy, attach authorization policy
//! runtime, deploy workloads, attach a cloud substrate runtime, or emit runtime
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
const CONTRACT_NAME: &str = "fd001-tenant-rbac-tenant-workload-identity-contract";
const PROGRAM_NAME: &str = "fd-001-tenant-rbac-generic";
const SUBSTRATE_NAME: &str = "oyatie-cloud";
const TENANT_NAMESPACE: &str = "oyatie-fd001-tenant-rbac-dev";
const TRUST_DOMAIN: &str = "oyatie.dev";
const SPIFFE_ID_PREFIX: &str = "spiffe://oyatie.dev/fd001-tenant-rbac/";
const SOURCE_MANIFEST_REF: &str =
    "crates/tenant-rbac-tenant-workload-manifest/src/lib.rs::fd001_tenant_workload_manifest";
const SOURCE_ADMISSION_POLICY_REF: &str =
    "crates/tenant-rbac-tenant-admission-policy/src/lib.rs::fd001_tenant_admission_policy_contract";
const POLICY_REF_PREFIX: &str = "policy/workload-identity/fd001/";
const EXPECTED_EVIDENCE_REF: &str =
    "evidence/workload-identity/fd001-tenant-rbac/workload-identity-review.jsonl";

const KUBERNETES_SERVICE_ACCOUNTS_DOC_URL: &str =
    "https://kubernetes.io/docs/concepts/security/service-accounts/";
const SPIFFE_OVERVIEW_DOC_URL: &str = "https://spiffe.io/docs/latest/spiffe-about/overview/";
const SPIFFE_CONCEPTS_DOC_URL: &str = "https://spiffe.io/docs/latest/spiffe-about/spiffe-concepts/";
const SPIRE_CONCEPTS_DOC_URL: &str = "https://spiffe.io/docs/latest/spire-about/spire-concepts/";
const GATEWAY_BACKEND_TLS_DOC_URL: &str =
    "https://gateway-api.sigs.k8s.io/api-types/backendtlspolicy/";
const OTEL_SERVICE_SEMCONV_DOC_URL: &str =
    "https://opentelemetry.io/docs/specs/semconv/resource/service/";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Fd001TenantWorkloadIdentityRequirementKind {
    SpiffeIdRequired,
    TrustDomainPinned,
    X509SvidRequired,
    JwtSvidPolicyRequired,
    MutualTlsRequired,
    GatewayBackendTlsPolicyRequired,
    CertificateRotationEvidenceRequired,
    TrustBundleEvidenceRequired,
    WorkloadApiBoundaryRequired,
    WorkloadAttestationSelectorRequired,
    ServiceTelemetryIdentityRequired,
    AuthorizationPolicyBindingRequired,
    IdentityAuditEvidenceRequired,
}

impl Fd001TenantWorkloadIdentityRequirementKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SpiffeIdRequired => "spiffe_id_required",
            Self::TrustDomainPinned => "trust_domain_pinned",
            Self::X509SvidRequired => "x509_svid_required",
            Self::JwtSvidPolicyRequired => "jwt_svid_policy_required",
            Self::MutualTlsRequired => "mutual_tls_required",
            Self::GatewayBackendTlsPolicyRequired => "gateway_backend_tls_policy_required",
            Self::CertificateRotationEvidenceRequired => "certificate_rotation_evidence_required",
            Self::TrustBundleEvidenceRequired => "trust_bundle_evidence_required",
            Self::WorkloadApiBoundaryRequired => "workload_api_boundary_required",
            Self::WorkloadAttestationSelectorRequired => "workload_attestation_selector_required",
            Self::ServiceTelemetryIdentityRequired => "service_telemetry_identity_required",
            Self::AuthorizationPolicyBindingRequired => "authorization_policy_binding_required",
            Self::IdentityAuditEvidenceRequired => "identity_audit_evidence_required",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fd001TenantWorkloadIdentityRequirement {
    pub requirement_id: &'static str,           // data_class: PUBLIC
    pub workload_kind: Fd001TenantWorkloadKind, // data_class: PUBLIC
    pub requirement_kind: Fd001TenantWorkloadIdentityRequirementKind, // data_class: PUBLIC
    pub resource_kind: &'static str,            // data_class: PUBLIC
    pub spiffe_id: &'static str,                // data_class: INTERNAL_ONLY
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
pub struct Fd001TenantWorkloadIdentityContract {
    pub contract_name: &'static str,          // data_class: PUBLIC
    pub program_name: &'static str,           // data_class: PUBLIC
    pub substrate_name: &'static str,         // data_class: PUBLIC
    pub tenant_namespace: &'static str,       // data_class: INTERNAL_ONLY
    pub trust_domain: &'static str,           // data_class: INTERNAL_ONLY
    pub spiffe_id_prefix: &'static str,       // data_class: INTERNAL_ONLY
    pub workload_manifest_name: &'static str, // data_class: PUBLIC
    pub workload_manifest_count: usize,       // data_class: PUBLIC
    pub tenant_admission_policy_contract_name: &'static str, // data_class: PUBLIC
    pub requirements: Vec<Fd001TenantWorkloadIdentityRequirement>, // data_class: INTERNAL_ONLY
    pub official_docs_required: bool,         // data_class: PUBLIC
    pub all_manifest_workloads_in_scope: bool, // data_class: PUBLIC
    pub spiffe_id_required: bool,             // data_class: PUBLIC
    pub trust_domain_pinned: bool,            // data_class: PUBLIC
    pub x509_svid_required: bool,             // data_class: PUBLIC
    pub jwt_svid_policy_required: bool,       // data_class: PUBLIC
    pub mutual_tls_required: bool,            // data_class: PUBLIC
    pub gateway_backend_tls_policy_required: bool, // data_class: PUBLIC
    pub certificate_rotation_evidence_required: bool, // data_class: PUBLIC
    pub trust_bundle_evidence_required: bool, // data_class: PUBLIC
    pub workload_api_boundary_required: bool, // data_class: PUBLIC
    pub workload_attestation_selector_required: bool, // data_class: PUBLIC
    pub service_telemetry_identity_required: bool, // data_class: PUBLIC
    pub authorization_policy_binding_required: bool, // data_class: PUBLIC
    pub identity_audit_evidence_required: bool, // data_class: PUBLIC
    pub review_only_contract: bool,           // data_class: PUBLIC
    pub kubernetes_cluster_attached: bool,    // data_class: INTERNAL_ONLY
    pub spiffe_workload_api_attached: bool,   // data_class: INTERNAL_ONLY
    pub spire_server_runtime_attached: bool,  // data_class: INTERNAL_ONLY
    pub spire_agent_runtime_attached: bool,   // data_class: INTERNAL_ONLY
    pub svid_runtime_issued: bool,            // data_class: INTERNAL_ONLY
    pub mtls_handshake_observed: bool,        // data_class: INTERNAL_ONLY
    pub certificate_rotation_runtime_observed: bool, // data_class: INTERNAL_ONLY
    pub gateway_backend_tls_applied: bool,    // data_class: INTERNAL_ONLY
    pub authorization_policy_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub workload_runtime_deployed: bool,      // data_class: INTERNAL_ONLY
    pub cloud_substrate_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Fd001TenantWorkloadIdentityError {
    WorkloadManifest(Fd001TenantWorkloadManifestError),
    TenantAdmissionPolicy(Fd001TenantAdmissionPolicyError),
    InvalidContractName,
    InvalidProgramName,
    InvalidSubstrateName,
    InvalidTenantNamespace,
    InvalidTrustDomain,
    InvalidSpiffeIdPrefix,
    InvalidManifestName,
    InvalidWorkloadCount,
    InvalidAdmissionPolicyContractName,
    MissingRequirements,
    DuplicateRequirement(String),
    MissingWorkloadKind(Fd001TenantWorkloadKind),
    MissingRequirementKind(Fd001TenantWorkloadIdentityRequirementKind),
    InvalidRequirementId,
    InvalidResourceKind,
    InvalidSpiffeId,
    InvalidPolicyRef,
    InvalidExpectedEvidenceRef,
    InvalidOfficialDocUrl,
    InvalidSourceManifestRef,
    InvalidSourceAdmissionPolicyRef,
    MissingRequiredControl(&'static str),
    RuntimeAttachmentOverclaim,
}

pub fn fd001_tenant_workload_identity_contract()
-> Result<Fd001TenantWorkloadIdentityContract, Fd001TenantWorkloadIdentityError> {
    let manifest = fd001_tenant_workload_manifest();
    validate_fd001_tenant_workload_manifest(&manifest)
        .map_err(Fd001TenantWorkloadIdentityError::WorkloadManifest)?;
    let admission_policy = fd001_tenant_admission_policy_contract()
        .map_err(Fd001TenantWorkloadIdentityError::TenantAdmissionPolicy)?;
    validate_fd001_tenant_admission_policy_contract(&admission_policy)
        .map_err(Fd001TenantWorkloadIdentityError::TenantAdmissionPolicy)?;

    Ok(Fd001TenantWorkloadIdentityContract {
        contract_name: CONTRACT_NAME,
        program_name: PROGRAM_NAME,
        substrate_name: SUBSTRATE_NAME,
        tenant_namespace: manifest.tenant_namespace,
        trust_domain: TRUST_DOMAIN,
        spiffe_id_prefix: SPIFFE_ID_PREFIX,
        workload_manifest_name: manifest.manifest_name,
        workload_manifest_count: manifest.workloads.len(),
        tenant_admission_policy_contract_name: admission_policy.contract_name,
        requirements: workload_identity_requirements(),
        official_docs_required: true,
        all_manifest_workloads_in_scope: true,
        spiffe_id_required: true,
        trust_domain_pinned: true,
        x509_svid_required: true,
        jwt_svid_policy_required: true,
        mutual_tls_required: true,
        gateway_backend_tls_policy_required: true,
        certificate_rotation_evidence_required: true,
        trust_bundle_evidence_required: true,
        workload_api_boundary_required: true,
        workload_attestation_selector_required: true,
        service_telemetry_identity_required: true,
        authorization_policy_binding_required: true,
        identity_audit_evidence_required: true,
        review_only_contract: true,
        kubernetes_cluster_attached: false,
        spiffe_workload_api_attached: false,
        spire_server_runtime_attached: false,
        spire_agent_runtime_attached: false,
        svid_runtime_issued: false,
        mtls_handshake_observed: false,
        certificate_rotation_runtime_observed: false,
        gateway_backend_tls_applied: false,
        authorization_policy_runtime_attached: false,
        workload_runtime_deployed: false,
        cloud_substrate_runtime_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: SCHEMA_VERSION,
    })
}

pub fn validate_fd001_tenant_workload_identity_contract(
    contract: &Fd001TenantWorkloadIdentityContract,
) -> Result<(), Fd001TenantWorkloadIdentityError> {
    let manifest = fd001_tenant_workload_manifest();
    validate_fd001_tenant_workload_manifest(&manifest)
        .map_err(Fd001TenantWorkloadIdentityError::WorkloadManifest)?;
    let admission_policy = fd001_tenant_admission_policy_contract()
        .map_err(Fd001TenantWorkloadIdentityError::TenantAdmissionPolicy)?;
    validate_fd001_tenant_admission_policy_contract(&admission_policy)
        .map_err(Fd001TenantWorkloadIdentityError::TenantAdmissionPolicy)?;

    validate_slug(
        contract.contract_name,
        Fd001TenantWorkloadIdentityError::InvalidContractName,
    )?;
    if contract.program_name != PROGRAM_NAME {
        return Err(Fd001TenantWorkloadIdentityError::InvalidProgramName);
    }
    if contract.substrate_name != SUBSTRATE_NAME {
        return Err(Fd001TenantWorkloadIdentityError::InvalidSubstrateName);
    }
    validate_tenant_namespace(contract.tenant_namespace)?;
    validate_trust_domain(contract.trust_domain)?;
    validate_spiffe_prefix(contract.spiffe_id_prefix)?;
    if contract.workload_manifest_name != manifest.manifest_name {
        return Err(Fd001TenantWorkloadIdentityError::InvalidManifestName);
    }
    if contract.workload_manifest_count != manifest.workloads.len() {
        return Err(Fd001TenantWorkloadIdentityError::InvalidWorkloadCount);
    }
    if contract.tenant_admission_policy_contract_name != admission_policy.contract_name {
        return Err(Fd001TenantWorkloadIdentityError::InvalidAdmissionPolicyContractName);
    }
    if contract.requirements.len() < MIN_REQUIREMENT_COUNT
        || contract.schema_version != SCHEMA_VERSION
    {
        return Err(Fd001TenantWorkloadIdentityError::MissingRequirements);
    }
    validate_required_controls(contract)?;
    validate_nonclaims(contract)?;
    validate_requirements(contract)?;
    Ok(())
}

pub fn fd001_tenant_workload_identity_doc_urls(
    contract: &Fd001TenantWorkloadIdentityContract,
) -> Vec<&'static str> {
    let mut docs = BTreeSet::new();
    for requirement in &contract.requirements {
        docs.insert(requirement.official_doc_url);
    }
    docs.into_iter().collect()
}

fn workload_identity_requirements() -> Vec<Fd001TenantWorkloadIdentityRequirement> {
    vec![
        requirement(
            "spiffe-id-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantWorkloadIdentityRequirementKind::SpiffeIdRequired,
            "SpiffeId",
            "tenant-rbac-api",
            "spiffe-id",
            SPIFFE_CONCEPTS_DOC_URL,
        ),
        requirement(
            "trust-domain-pinned-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantWorkloadIdentityRequirementKind::TrustDomainPinned,
            "SpiffeTrustDomain",
            "hr-employment-api",
            "trust-domain-pinned",
            SPIFFE_CONCEPTS_DOC_URL,
        ),
        requirement(
            "x509-svid-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantWorkloadIdentityRequirementKind::X509SvidRequired,
            "X509Svid",
            "payroll-run-api",
            "x509-svid",
            SPIFFE_OVERVIEW_DOC_URL,
        ),
        requirement(
            "jwt-svid-policy-all-workloads",
            Fd001TenantWorkloadKind::AccountingJournal,
            Fd001TenantWorkloadIdentityRequirementKind::JwtSvidPolicyRequired,
            "JwtSvid",
            "accounting-journal-api",
            "jwt-svid-policy",
            SPIFFE_CONCEPTS_DOC_URL,
        ),
        requirement(
            "mutual-tls-policy-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantWorkloadIdentityRequirementKind::MutualTlsRequired,
            "MutualTlsPolicy",
            "tenant-rbac-runtime",
            "mutual-tls-policy",
            SPIFFE_OVERVIEW_DOC_URL,
        ),
        requirement(
            "gateway-backend-tls-policy-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantWorkloadIdentityRequirementKind::GatewayBackendTlsPolicyRequired,
            "GatewayBackendTlsPolicy",
            "hr-employment-runtime",
            "gateway-backend-tls-policy",
            GATEWAY_BACKEND_TLS_DOC_URL,
        ),
        requirement(
            "certificate-rotation-evidence-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantWorkloadIdentityRequirementKind::CertificateRotationEvidenceRequired,
            "CertificateRotationPolicy",
            "payroll-run-runtime",
            "certificate-rotation-evidence",
            SPIFFE_OVERVIEW_DOC_URL,
        ),
        requirement(
            "trust-bundle-evidence-all-workloads",
            Fd001TenantWorkloadKind::AccountingJournal,
            Fd001TenantWorkloadIdentityRequirementKind::TrustBundleEvidenceRequired,
            "TrustBundle",
            "accounting-journal-runtime",
            "trust-bundle-evidence",
            SPIFFE_CONCEPTS_DOC_URL,
        ),
        requirement(
            "workload-api-boundary-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantWorkloadIdentityRequirementKind::WorkloadApiBoundaryRequired,
            "SpiffeWorkloadApi",
            "tenant-rbac-workload-api",
            "workload-api-boundary",
            SPIRE_CONCEPTS_DOC_URL,
        ),
        requirement(
            "workload-attestation-selector-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantWorkloadIdentityRequirementKind::WorkloadAttestationSelectorRequired,
            "WorkloadAttestationSelector",
            "hr-employment-workload-api",
            "workload-attestation-selector",
            SPIRE_CONCEPTS_DOC_URL,
        ),
        requirement(
            "service-telemetry-identity-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantWorkloadIdentityRequirementKind::ServiceTelemetryIdentityRequired,
            "OpenTelemetryServiceResource",
            "payroll-run-telemetry",
            "service-telemetry-identity",
            OTEL_SERVICE_SEMCONV_DOC_URL,
        ),
        requirement(
            "authorization-policy-binding-all-workloads",
            Fd001TenantWorkloadKind::AccountingJournal,
            Fd001TenantWorkloadIdentityRequirementKind::AuthorizationPolicyBindingRequired,
            "AuthorizationPolicyBinding",
            "accounting-journal-authz",
            "authorization-policy-binding",
            KUBERNETES_SERVICE_ACCOUNTS_DOC_URL,
        ),
        requirement(
            "identity-audit-evidence-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantWorkloadIdentityRequirementKind::IdentityAuditEvidenceRequired,
            "IdentityAuditEvidence",
            "tenant-rbac-audit",
            "identity-audit-evidence",
            OTEL_SERVICE_SEMCONV_DOC_URL,
        ),
    ]
}

fn requirement(
    requirement_id: &'static str,
    workload_kind: Fd001TenantWorkloadKind,
    requirement_kind: Fd001TenantWorkloadIdentityRequirementKind,
    resource_kind: &'static str,
    spiffe_suffix: &'static str,
    policy_suffix: &'static str,
    official_doc_url: &'static str,
) -> Fd001TenantWorkloadIdentityRequirement {
    Fd001TenantWorkloadIdentityRequirement {
        requirement_id,
        workload_kind,
        requirement_kind,
        resource_kind,
        spiffe_id: spiffe_id(spiffe_suffix),
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

fn spiffe_id(spiffe_suffix: &'static str) -> &'static str {
    match spiffe_suffix {
        "tenant-rbac-api" => "spiffe://oyatie.dev/fd001-tenant-rbac/tenant-rbac-api",
        "hr-employment-api" => "spiffe://oyatie.dev/fd001-tenant-rbac/hr-employment-api",
        "payroll-run-api" => "spiffe://oyatie.dev/fd001-tenant-rbac/payroll-run-api",
        "accounting-journal-api" => "spiffe://oyatie.dev/fd001-tenant-rbac/accounting-journal-api",
        "tenant-rbac-runtime" => "spiffe://oyatie.dev/fd001-tenant-rbac/tenant-rbac-runtime",
        "hr-employment-runtime" => "spiffe://oyatie.dev/fd001-tenant-rbac/hr-employment-runtime",
        "payroll-run-runtime" => "spiffe://oyatie.dev/fd001-tenant-rbac/payroll-run-runtime",
        "accounting-journal-runtime" => {
            "spiffe://oyatie.dev/fd001-tenant-rbac/accounting-journal-runtime"
        }
        "tenant-rbac-workload-api" => {
            "spiffe://oyatie.dev/fd001-tenant-rbac/tenant-rbac-workload-api"
        }
        "hr-employment-workload-api" => {
            "spiffe://oyatie.dev/fd001-tenant-rbac/hr-employment-workload-api"
        }
        "payroll-run-telemetry" => "spiffe://oyatie.dev/fd001-tenant-rbac/payroll-run-telemetry",
        "accounting-journal-authz" => {
            "spiffe://oyatie.dev/fd001-tenant-rbac/accounting-journal-authz"
        }
        "tenant-rbac-audit" => "spiffe://oyatie.dev/fd001-tenant-rbac/tenant-rbac-audit",
        _ => "spiffe://oyatie.dev/fd001-tenant-rbac/invalid",
    }
}

fn policy_ref(policy_suffix: &'static str) -> &'static str {
    match policy_suffix {
        "spiffe-id" => "policy/workload-identity/fd001/spiffe-id",
        "trust-domain-pinned" => "policy/workload-identity/fd001/trust-domain-pinned",
        "x509-svid" => "policy/workload-identity/fd001/x509-svid",
        "jwt-svid-policy" => "policy/workload-identity/fd001/jwt-svid-policy",
        "mutual-tls-policy" => "policy/workload-identity/fd001/mutual-tls-policy",
        "gateway-backend-tls-policy" => "policy/workload-identity/fd001/gateway-backend-tls-policy",
        "certificate-rotation-evidence" => {
            "policy/workload-identity/fd001/certificate-rotation-evidence"
        }
        "trust-bundle-evidence" => "policy/workload-identity/fd001/trust-bundle-evidence",
        "workload-api-boundary" => "policy/workload-identity/fd001/workload-api-boundary",
        "workload-attestation-selector" => {
            "policy/workload-identity/fd001/workload-attestation-selector"
        }
        "service-telemetry-identity" => "policy/workload-identity/fd001/service-telemetry-identity",
        "authorization-policy-binding" => {
            "policy/workload-identity/fd001/authorization-policy-binding"
        }
        "identity-audit-evidence" => "policy/workload-identity/fd001/identity-audit-evidence",
        _ => "policy/workload-identity/fd001/invalid",
    }
}

fn validate_required_controls(
    contract: &Fd001TenantWorkloadIdentityContract,
) -> Result<(), Fd001TenantWorkloadIdentityError> {
    for (enabled, name) in [
        (contract.official_docs_required, "official_docs_required"),
        (
            contract.all_manifest_workloads_in_scope,
            "all_manifest_workloads_in_scope",
        ),
        (contract.spiffe_id_required, "spiffe_id_required"),
        (contract.trust_domain_pinned, "trust_domain_pinned"),
        (contract.x509_svid_required, "x509_svid_required"),
        (
            contract.jwt_svid_policy_required,
            "jwt_svid_policy_required",
        ),
        (contract.mutual_tls_required, "mutual_tls_required"),
        (
            contract.gateway_backend_tls_policy_required,
            "gateway_backend_tls_policy_required",
        ),
        (
            contract.certificate_rotation_evidence_required,
            "certificate_rotation_evidence_required",
        ),
        (
            contract.trust_bundle_evidence_required,
            "trust_bundle_evidence_required",
        ),
        (
            contract.workload_api_boundary_required,
            "workload_api_boundary_required",
        ),
        (
            contract.workload_attestation_selector_required,
            "workload_attestation_selector_required",
        ),
        (
            contract.service_telemetry_identity_required,
            "service_telemetry_identity_required",
        ),
        (
            contract.authorization_policy_binding_required,
            "authorization_policy_binding_required",
        ),
        (
            contract.identity_audit_evidence_required,
            "identity_audit_evidence_required",
        ),
        (contract.review_only_contract, "review_only_contract"),
    ] {
        require_control(enabled, name)?;
    }
    Ok(())
}

fn validate_nonclaims(
    contract: &Fd001TenantWorkloadIdentityContract,
) -> Result<(), Fd001TenantWorkloadIdentityError> {
    if contract.kubernetes_cluster_attached
        || contract.spiffe_workload_api_attached
        || contract.spire_server_runtime_attached
        || contract.spire_agent_runtime_attached
        || contract.svid_runtime_issued
        || contract.mtls_handshake_observed
        || contract.certificate_rotation_runtime_observed
        || contract.gateway_backend_tls_applied
        || contract.authorization_policy_runtime_attached
        || contract.workload_runtime_deployed
        || contract.cloud_substrate_runtime_attached
        || contract.runtime_audit_chain_emission_attached
    {
        return Err(Fd001TenantWorkloadIdentityError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_requirements(
    contract: &Fd001TenantWorkloadIdentityContract,
) -> Result<(), Fd001TenantWorkloadIdentityError> {
    let mut seen_requirements = BTreeSet::new();
    let mut seen_workloads = BTreeSet::new();
    let mut seen_requirement_kinds = BTreeSet::new();
    for requirement in &contract.requirements {
        validate_requirement(requirement)?;
        if !seen_requirements.insert(requirement.requirement_id) {
            return Err(Fd001TenantWorkloadIdentityError::DuplicateRequirement(
                requirement.requirement_id.to_owned(),
            ));
        }
        seen_workloads.insert(requirement.workload_kind);
        seen_requirement_kinds.insert(requirement.requirement_kind);
    }
    for workload_kind in required_workload_kinds() {
        if !seen_workloads.contains(&workload_kind) {
            return Err(Fd001TenantWorkloadIdentityError::MissingWorkloadKind(
                workload_kind,
            ));
        }
    }
    for requirement_kind in required_requirement_kinds() {
        if !seen_requirement_kinds.contains(&requirement_kind) {
            return Err(Fd001TenantWorkloadIdentityError::MissingRequirementKind(
                requirement_kind,
            ));
        }
    }
    Ok(())
}

fn validate_requirement(
    requirement: &Fd001TenantWorkloadIdentityRequirement,
) -> Result<(), Fd001TenantWorkloadIdentityError> {
    validate_slug(
        requirement.requirement_id,
        Fd001TenantWorkloadIdentityError::InvalidRequirementId,
    )?;
    validate_resource_kind(requirement.resource_kind)?;
    validate_spiffe_id(requirement.spiffe_id)?;
    validate_prefixed_ref(
        requirement.policy_ref,
        POLICY_REF_PREFIX,
        Fd001TenantWorkloadIdentityError::InvalidPolicyRef,
    )?;
    validate_prefixed_ref(
        requirement.expected_evidence_ref,
        "evidence/workload-identity/fd001-tenant-rbac/",
        Fd001TenantWorkloadIdentityError::InvalidExpectedEvidenceRef,
    )?;
    validate_doc_url(requirement.official_doc_url)?;
    validate_prefixed_ref(
        requirement.source_manifest_ref,
        "crates/tenant-rbac-tenant-workload-manifest/",
        Fd001TenantWorkloadIdentityError::InvalidSourceManifestRef,
    )?;
    validate_prefixed_ref(
        requirement.source_admission_policy_ref,
        "crates/tenant-rbac-tenant-admission-policy/",
        Fd001TenantWorkloadIdentityError::InvalidSourceAdmissionPolicyRef,
    )?;
    if !requirement.applies_to_all_manifest_workloads {
        return Err(Fd001TenantWorkloadIdentityError::MissingRequiredControl(
            "requirement_applies_to_all_manifest_workloads",
        ));
    }
    if requirement.runtime_observation_attached || requirement.schema_version != SCHEMA_VERSION {
        return Err(Fd001TenantWorkloadIdentityError::RuntimeAttachmentOverclaim);
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

fn required_requirement_kinds() -> [Fd001TenantWorkloadIdentityRequirementKind; 13] {
    [
        Fd001TenantWorkloadIdentityRequirementKind::SpiffeIdRequired,
        Fd001TenantWorkloadIdentityRequirementKind::TrustDomainPinned,
        Fd001TenantWorkloadIdentityRequirementKind::X509SvidRequired,
        Fd001TenantWorkloadIdentityRequirementKind::JwtSvidPolicyRequired,
        Fd001TenantWorkloadIdentityRequirementKind::MutualTlsRequired,
        Fd001TenantWorkloadIdentityRequirementKind::GatewayBackendTlsPolicyRequired,
        Fd001TenantWorkloadIdentityRequirementKind::CertificateRotationEvidenceRequired,
        Fd001TenantWorkloadIdentityRequirementKind::TrustBundleEvidenceRequired,
        Fd001TenantWorkloadIdentityRequirementKind::WorkloadApiBoundaryRequired,
        Fd001TenantWorkloadIdentityRequirementKind::WorkloadAttestationSelectorRequired,
        Fd001TenantWorkloadIdentityRequirementKind::ServiceTelemetryIdentityRequired,
        Fd001TenantWorkloadIdentityRequirementKind::AuthorizationPolicyBindingRequired,
        Fd001TenantWorkloadIdentityRequirementKind::IdentityAuditEvidenceRequired,
    ]
}

fn validate_resource_kind(value: &str) -> Result<(), Fd001TenantWorkloadIdentityError> {
    if ![
        "SpiffeId",
        "SpiffeTrustDomain",
        "X509Svid",
        "JwtSvid",
        "MutualTlsPolicy",
        "GatewayBackendTlsPolicy",
        "CertificateRotationPolicy",
        "TrustBundle",
        "SpiffeWorkloadApi",
        "WorkloadAttestationSelector",
        "OpenTelemetryServiceResource",
        "AuthorizationPolicyBinding",
        "IdentityAuditEvidence",
    ]
    .contains(&value)
    {
        return Err(Fd001TenantWorkloadIdentityError::InvalidResourceKind);
    }
    Ok(())
}

fn validate_doc_url(url: &str) -> Result<(), Fd001TenantWorkloadIdentityError> {
    if has_unsafe_ref_text(url)
        || ![
            KUBERNETES_SERVICE_ACCOUNTS_DOC_URL,
            SPIFFE_OVERVIEW_DOC_URL,
            SPIFFE_CONCEPTS_DOC_URL,
            SPIRE_CONCEPTS_DOC_URL,
            GATEWAY_BACKEND_TLS_DOC_URL,
            OTEL_SERVICE_SEMCONV_DOC_URL,
        ]
        .contains(&url)
    {
        return Err(Fd001TenantWorkloadIdentityError::InvalidOfficialDocUrl);
    }
    Ok(())
}

fn validate_slug(
    value: &str,
    error: Fd001TenantWorkloadIdentityError,
) -> Result<(), Fd001TenantWorkloadIdentityError> {
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

fn validate_tenant_namespace(value: &str) -> Result<(), Fd001TenantWorkloadIdentityError> {
    validate_slug(
        value,
        Fd001TenantWorkloadIdentityError::InvalidTenantNamespace,
    )?;
    if value != TENANT_NAMESPACE {
        return Err(Fd001TenantWorkloadIdentityError::InvalidTenantNamespace);
    }
    Ok(())
}

fn validate_trust_domain(value: &str) -> Result<(), Fd001TenantWorkloadIdentityError> {
    if value != TRUST_DOMAIN
        || has_unsafe_ref_text(value)
        || !value.contains('.')
        || value
            .chars()
            .any(|ch| !(ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '.'))
    {
        return Err(Fd001TenantWorkloadIdentityError::InvalidTrustDomain);
    }
    Ok(())
}

fn validate_spiffe_prefix(value: &str) -> Result<(), Fd001TenantWorkloadIdentityError> {
    if value != SPIFFE_ID_PREFIX || has_unsafe_ref_text(value) {
        return Err(Fd001TenantWorkloadIdentityError::InvalidSpiffeIdPrefix);
    }
    Ok(())
}

fn validate_spiffe_id(value: &str) -> Result<(), Fd001TenantWorkloadIdentityError> {
    if value.len() <= SPIFFE_ID_PREFIX.len()
        || !value.starts_with(SPIFFE_ID_PREFIX)
        || has_unsafe_ref_text(value)
        || value[SPIFFE_ID_PREFIX.len()..]
            .chars()
            .any(|ch| !(ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-'))
    {
        return Err(Fd001TenantWorkloadIdentityError::InvalidSpiffeId);
    }
    Ok(())
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: Fd001TenantWorkloadIdentityError,
) -> Result<(), Fd001TenantWorkloadIdentityError> {
    if value.len() <= prefix.len() || !value.starts_with(prefix) || has_unsafe_ref_text(value) {
        return Err(error);
    }
    Ok(())
}

fn require_control(
    value: bool,
    control: &'static str,
) -> Result<(), Fd001TenantWorkloadIdentityError> {
    if value {
        Ok(())
    } else {
        Err(Fd001TenantWorkloadIdentityError::MissingRequiredControl(
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
