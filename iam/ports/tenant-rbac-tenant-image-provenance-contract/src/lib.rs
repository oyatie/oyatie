//! FD-001 tenant image provenance contract for later Oyatie Cloud dogfooding.
//!
//! This review-only crate defines supply-chain evidence that must exist before
//! FD-001 Tenant RBAC, HR, Payroll, and Accounting images can be promoted
//! as production tenant workloads on the future Oyatie Cloud substrate. It binds
//! the FD-001 tenant-workload manifest and tenant admission policy to OCI digest
//! pinning, Cosign verification, transparency-log evidence, in-toto/SLSA
//! provenance, SBOM publication, source revision pinning, builder identity, and
//! vulnerability gate requirements. It does not publish images, attach a
//! registry, run Cosign, verify a transparency log at runtime, publish SBOMs,
//! attach a scanner, deploy workloads, attach a cloud substrate runtime, or emit
//! runtime audit-chain events.
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
const CONTRACT_NAME: &str = "fd001-tenant-rbac-tenant-image-provenance-contract";
const PROGRAM_NAME: &str = "fd-001-tenant-rbac-generic";
const SUBSTRATE_NAME: &str = "oyatie-cloud";
const TENANT_NAMESPACE: &str = "oyatie-fd001-tenant-rbac-dev";
const SOURCE_MANIFEST_REF: &str =
    "crates/tenant-rbac-tenant-workload-manifest/src/lib.rs::fd001_tenant_workload_manifest";
const SOURCE_ADMISSION_POLICY_REF: &str =
    "crates/tenant-rbac-tenant-admission-policy/src/lib.rs::fd001_tenant_admission_policy_contract";
const ARTIFACT_REF: &str = "oci-image/fd001-tenant-rbac/all-manifest-workloads-sha256-digest";
const POLICY_REF_PREFIX: &str = "policy/supply-chain/fd001/";
const EVIDENCE_REF: &str = "evidence/supply-chain/fd001-tenant-rbac/image-provenance.jsonl";

const OCI_IMAGE_MANIFEST_DOC_URL: &str =
    "https://specs.opencontainers.org/image-spec/manifest/?v=v1.1.0";
const SIGSTORE_COSIGN_VERIFY_DOC_URL: &str = "https://docs.sigstore.dev/cosign/verifying/verify/";
const SIGSTORE_COSIGN_VERIFY_OVERVIEW_DOC_URL: &str = "https://docs.sigstore.dev/cosign/verifying/";
const SLSA_SPEC_DOC_URL: &str = "https://slsa.dev/spec/v1.2/";
const SLSA_ATTESTATION_MODEL_DOC_URL: &str = "https://slsa.dev/spec/v1.2/attestation-model";
const SLSA_BUILD_PROVENANCE_DOC_URL: &str = "https://slsa.dev/spec/v1.2/build-provenance";
const SLSA_VERIFYING_ARTIFACTS_DOC_URL: &str = "https://slsa.dev/spec/v1.2/verifying-artifacts";
const IN_TOTO_SPECS_DOC_URL: &str = "https://in-toto.io/docs/specs/";
const SPDX_SPEC_DOC_URL: &str = "https://spdx.github.io/spdx-spec/v2.3/";
const CYCLONEDX_SBOM_DOC_URL: &str = "https://cyclonedx.org/capabilities/sbom/";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Fd001TenantImageProvenanceControlKind {
    OciDigestPinned,
    CosignSignatureRequired,
    KeylessOidcIdentityRequired,
    RekorTransparencyLogRequired,
    IntotoStatementRequired,
    SlsaProvenanceRequired,
    BuilderIdPinned,
    SourceRevisionPinned,
    SbomRequired,
    VulnerabilityScanGateRequired,
    AdmissionPolicyEvidenceRequired,
}

impl Fd001TenantImageProvenanceControlKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OciDigestPinned => "oci_digest_pinned",
            Self::CosignSignatureRequired => "cosign_signature_required",
            Self::KeylessOidcIdentityRequired => "keyless_oidc_identity_required",
            Self::RekorTransparencyLogRequired => "rekor_transparency_log_required",
            Self::IntotoStatementRequired => "intoto_statement_required",
            Self::SlsaProvenanceRequired => "slsa_provenance_required",
            Self::BuilderIdPinned => "builder_id_pinned",
            Self::SourceRevisionPinned => "source_revision_pinned",
            Self::SbomRequired => "sbom_required",
            Self::VulnerabilityScanGateRequired => "vulnerability_scan_gate_required",
            Self::AdmissionPolicyEvidenceRequired => "admission_policy_evidence_required",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fd001TenantImageProvenanceRequirement {
    pub requirement_id: &'static str,           // data_class: PUBLIC
    pub workload_kind: Fd001TenantWorkloadKind, // data_class: PUBLIC
    pub control_kind: Fd001TenantImageProvenanceControlKind, // data_class: PUBLIC
    pub artifact_ref: &'static str,             // data_class: INTERNAL_ONLY
    pub policy_ref: &'static str,               // data_class: INTERNAL_ONLY
    pub expected_evidence_ref: &'static str,    // data_class: INTERNAL_ONLY
    pub official_doc_url: &'static str,         // data_class: PUBLIC
    pub source_manifest_ref: &'static str,      // data_class: INTERNAL_ONLY
    pub source_admission_policy_ref: &'static str, // data_class: INTERNAL_ONLY
    pub applies_to_all_manifest_workloads: bool, // data_class: PUBLIC
    pub runtime_verification_attached: bool,    // data_class: INTERNAL_ONLY
    pub schema_version: u32,                    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fd001TenantImageProvenanceContract {
    pub contract_name: &'static str,          // data_class: PUBLIC
    pub program_name: &'static str,           // data_class: PUBLIC
    pub substrate_name: &'static str,         // data_class: PUBLIC
    pub tenant_namespace: &'static str,       // data_class: INTERNAL_ONLY
    pub workload_manifest_name: &'static str, // data_class: PUBLIC
    pub workload_manifest_count: usize,       // data_class: PUBLIC
    pub tenant_admission_policy_contract_name: &'static str, // data_class: PUBLIC
    pub requirements: Vec<Fd001TenantImageProvenanceRequirement>, // data_class: INTERNAL_ONLY
    pub official_docs_required: bool,         // data_class: PUBLIC
    pub all_manifest_workloads_in_scope: bool, // data_class: PUBLIC
    pub oci_digest_pinned_required: bool,     // data_class: PUBLIC
    pub cosign_signature_required: bool,      // data_class: PUBLIC
    pub keyless_oidc_identity_required: bool, // data_class: PUBLIC
    pub transparency_log_required: bool,      // data_class: PUBLIC
    pub intoto_statement_required: bool,      // data_class: PUBLIC
    pub slsa_provenance_required: bool,       // data_class: PUBLIC
    pub builder_id_pin_required: bool,        // data_class: PUBLIC
    pub source_revision_pin_required: bool,   // data_class: PUBLIC
    pub sbom_required: bool,                  // data_class: PUBLIC
    pub vulnerability_scan_gate_required: bool, // data_class: PUBLIC
    pub admission_policy_evidence_required: bool, // data_class: PUBLIC
    pub review_only_contract: bool,           // data_class: PUBLIC
    pub image_registry_attached: bool,        // data_class: INTERNAL_ONLY
    pub image_published: bool,                // data_class: INTERNAL_ONLY
    pub cosign_runtime_verification_attached: bool, // data_class: INTERNAL_ONLY
    pub transparency_log_runtime_verified: bool, // data_class: INTERNAL_ONLY
    pub slsa_provenance_runtime_verified: bool, // data_class: INTERNAL_ONLY
    pub sbom_runtime_published: bool,         // data_class: INTERNAL_ONLY
    pub vulnerability_scanner_attached: bool, // data_class: INTERNAL_ONLY
    pub admission_controller_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub workload_runtime_deployed: bool,      // data_class: INTERNAL_ONLY
    pub cloud_substrate_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Fd001TenantImageProvenanceError {
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
    MissingControlKind(Fd001TenantImageProvenanceControlKind),
    InvalidRequirementId,
    InvalidArtifactRef,
    InvalidPolicyRef,
    InvalidExpectedEvidenceRef,
    InvalidOfficialDocUrl,
    InvalidSourceManifestRef,
    InvalidSourceAdmissionPolicyRef,
    MissingRequiredControl(&'static str),
    RuntimeAttachmentOverclaim,
}

pub fn fd001_tenant_image_provenance_contract()
-> Result<Fd001TenantImageProvenanceContract, Fd001TenantImageProvenanceError> {
    let manifest = fd001_tenant_workload_manifest();
    validate_fd001_tenant_workload_manifest(&manifest)
        .map_err(Fd001TenantImageProvenanceError::WorkloadManifest)?;
    let admission_policy = fd001_tenant_admission_policy_contract()
        .map_err(Fd001TenantImageProvenanceError::TenantAdmissionPolicy)?;
    validate_fd001_tenant_admission_policy_contract(&admission_policy)
        .map_err(Fd001TenantImageProvenanceError::TenantAdmissionPolicy)?;

    Ok(Fd001TenantImageProvenanceContract {
        contract_name: CONTRACT_NAME,
        program_name: PROGRAM_NAME,
        substrate_name: SUBSTRATE_NAME,
        tenant_namespace: manifest.tenant_namespace,
        workload_manifest_name: manifest.manifest_name,
        workload_manifest_count: manifest.workloads.len(),
        tenant_admission_policy_contract_name: admission_policy.contract_name,
        requirements: provenance_requirements(),
        official_docs_required: true,
        all_manifest_workloads_in_scope: true,
        oci_digest_pinned_required: true,
        cosign_signature_required: true,
        keyless_oidc_identity_required: true,
        transparency_log_required: true,
        intoto_statement_required: true,
        slsa_provenance_required: true,
        builder_id_pin_required: true,
        source_revision_pin_required: true,
        sbom_required: true,
        vulnerability_scan_gate_required: true,
        admission_policy_evidence_required: true,
        review_only_contract: true,
        image_registry_attached: false,
        image_published: false,
        cosign_runtime_verification_attached: false,
        transparency_log_runtime_verified: false,
        slsa_provenance_runtime_verified: false,
        sbom_runtime_published: false,
        vulnerability_scanner_attached: false,
        admission_controller_runtime_attached: false,
        workload_runtime_deployed: false,
        cloud_substrate_runtime_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: SCHEMA_VERSION,
    })
}

pub fn validate_fd001_tenant_image_provenance_contract(
    contract: &Fd001TenantImageProvenanceContract,
) -> Result<(), Fd001TenantImageProvenanceError> {
    let manifest = fd001_tenant_workload_manifest();
    validate_fd001_tenant_workload_manifest(&manifest)
        .map_err(Fd001TenantImageProvenanceError::WorkloadManifest)?;
    let admission_policy = fd001_tenant_admission_policy_contract()
        .map_err(Fd001TenantImageProvenanceError::TenantAdmissionPolicy)?;
    validate_fd001_tenant_admission_policy_contract(&admission_policy)
        .map_err(Fd001TenantImageProvenanceError::TenantAdmissionPolicy)?;

    validate_slug(
        contract.contract_name,
        Fd001TenantImageProvenanceError::InvalidContractName,
    )?;
    if contract.program_name != PROGRAM_NAME {
        return Err(Fd001TenantImageProvenanceError::InvalidProgramName);
    }
    if contract.substrate_name != SUBSTRATE_NAME {
        return Err(Fd001TenantImageProvenanceError::InvalidSubstrateName);
    }
    validate_tenant_namespace(contract.tenant_namespace)?;
    if contract.workload_manifest_name != manifest.manifest_name {
        return Err(Fd001TenantImageProvenanceError::InvalidManifestName);
    }
    if contract.workload_manifest_count != manifest.workloads.len() {
        return Err(Fd001TenantImageProvenanceError::InvalidWorkloadCount);
    }
    if contract.tenant_admission_policy_contract_name != admission_policy.contract_name {
        return Err(Fd001TenantImageProvenanceError::InvalidAdmissionPolicyContractName);
    }
    if contract.requirements.len() < MIN_REQUIREMENT_COUNT
        || contract.schema_version != SCHEMA_VERSION
    {
        return Err(Fd001TenantImageProvenanceError::MissingRequirements);
    }
    validate_required_controls(contract)?;
    validate_nonclaims(contract)?;
    validate_requirements(contract)?;
    Ok(())
}

pub fn fd001_tenant_image_provenance_doc_urls(
    contract: &Fd001TenantImageProvenanceContract,
) -> Vec<&'static str> {
    let mut docs = BTreeSet::new();
    for requirement in &contract.requirements {
        docs.insert(requirement.official_doc_url);
    }
    docs.into_iter().collect()
}

fn provenance_requirements() -> Vec<Fd001TenantImageProvenanceRequirement> {
    vec![
        requirement(
            "oci-digest-pinned-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantImageProvenanceControlKind::OciDigestPinned,
            "oci-digest-pinned",
            OCI_IMAGE_MANIFEST_DOC_URL,
        ),
        requirement(
            "cosign-signature-required-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantImageProvenanceControlKind::CosignSignatureRequired,
            "cosign-signature-required",
            SIGSTORE_COSIGN_VERIFY_DOC_URL,
        ),
        requirement(
            "keyless-oidc-identity-required-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantImageProvenanceControlKind::KeylessOidcIdentityRequired,
            "keyless-oidc-identity-required",
            SIGSTORE_COSIGN_VERIFY_DOC_URL,
        ),
        requirement(
            "rekor-transparency-log-required-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantImageProvenanceControlKind::RekorTransparencyLogRequired,
            "rekor-transparency-log-required",
            SIGSTORE_COSIGN_VERIFY_OVERVIEW_DOC_URL,
        ),
        requirement(
            "intoto-statement-required-all-workloads",
            Fd001TenantWorkloadKind::AccountingJournal,
            Fd001TenantImageProvenanceControlKind::IntotoStatementRequired,
            "intoto-statement-required",
            IN_TOTO_SPECS_DOC_URL,
        ),
        requirement(
            "slsa-provenance-required-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantImageProvenanceControlKind::SlsaProvenanceRequired,
            "slsa-provenance-required",
            SLSA_BUILD_PROVENANCE_DOC_URL,
        ),
        requirement(
            "builder-id-pinned-all-workloads",
            Fd001TenantWorkloadKind::HrEmployment,
            Fd001TenantImageProvenanceControlKind::BuilderIdPinned,
            "builder-id-pinned",
            SLSA_BUILD_PROVENANCE_DOC_URL,
        ),
        requirement(
            "source-revision-pinned-all-workloads",
            Fd001TenantWorkloadKind::PayrollRun,
            Fd001TenantImageProvenanceControlKind::SourceRevisionPinned,
            "source-revision-pinned",
            SLSA_VERIFYING_ARTIFACTS_DOC_URL,
        ),
        requirement(
            "sbom-required-all-workloads",
            Fd001TenantWorkloadKind::AccountingJournal,
            Fd001TenantImageProvenanceControlKind::SbomRequired,
            "sbom-required",
            CYCLONEDX_SBOM_DOC_URL,
        ),
        requirement(
            "vulnerability-scan-gate-required-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantImageProvenanceControlKind::VulnerabilityScanGateRequired,
            "vulnerability-scan-gate-required",
            SPDX_SPEC_DOC_URL,
        ),
        requirement(
            "admission-policy-evidence-required-all-workloads",
            Fd001TenantWorkloadKind::TenantRbac,
            Fd001TenantImageProvenanceControlKind::AdmissionPolicyEvidenceRequired,
            "admission-policy-evidence-required",
            SLSA_ATTESTATION_MODEL_DOC_URL,
        ),
    ]
}

fn requirement(
    requirement_id: &'static str,
    workload_kind: Fd001TenantWorkloadKind,
    control_kind: Fd001TenantImageProvenanceControlKind,
    policy_suffix: &'static str,
    official_doc_url: &'static str,
) -> Fd001TenantImageProvenanceRequirement {
    Fd001TenantImageProvenanceRequirement {
        requirement_id,
        workload_kind,
        control_kind,
        artifact_ref: ARTIFACT_REF,
        policy_ref: policy_ref(policy_suffix),
        expected_evidence_ref: EVIDENCE_REF,
        official_doc_url,
        source_manifest_ref: SOURCE_MANIFEST_REF,
        source_admission_policy_ref: SOURCE_ADMISSION_POLICY_REF,
        applies_to_all_manifest_workloads: true,
        runtime_verification_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

fn policy_ref(policy_suffix: &'static str) -> &'static str {
    match policy_suffix {
        "oci-digest-pinned" => "policy/supply-chain/fd001/oci-digest-pinned",
        "cosign-signature-required" => "policy/supply-chain/fd001/cosign-signature-required",
        "keyless-oidc-identity-required" => {
            "policy/supply-chain/fd001/keyless-oidc-identity-required"
        }
        "rekor-transparency-log-required" => {
            "policy/supply-chain/fd001/rekor-transparency-log-required"
        }
        "intoto-statement-required" => "policy/supply-chain/fd001/intoto-statement-required",
        "slsa-provenance-required" => "policy/supply-chain/fd001/slsa-provenance-required",
        "builder-id-pinned" => "policy/supply-chain/fd001/builder-id-pinned",
        "source-revision-pinned" => "policy/supply-chain/fd001/source-revision-pinned",
        "sbom-required" => "policy/supply-chain/fd001/sbom-required",
        "vulnerability-scan-gate-required" => {
            "policy/supply-chain/fd001/vulnerability-scan-gate-required"
        }
        "admission-policy-evidence-required" => {
            "policy/supply-chain/fd001/admission-policy-evidence-required"
        }
        _ => "policy/supply-chain/fd001/invalid",
    }
}

fn validate_required_controls(
    contract: &Fd001TenantImageProvenanceContract,
) -> Result<(), Fd001TenantImageProvenanceError> {
    for (enabled, name) in [
        (contract.official_docs_required, "official_docs_required"),
        (
            contract.all_manifest_workloads_in_scope,
            "all_manifest_workloads_in_scope",
        ),
        (
            contract.oci_digest_pinned_required,
            "oci_digest_pinned_required",
        ),
        (
            contract.cosign_signature_required,
            "cosign_signature_required",
        ),
        (
            contract.keyless_oidc_identity_required,
            "keyless_oidc_identity_required",
        ),
        (
            contract.transparency_log_required,
            "transparency_log_required",
        ),
        (
            contract.intoto_statement_required,
            "intoto_statement_required",
        ),
        (
            contract.slsa_provenance_required,
            "slsa_provenance_required",
        ),
        (contract.builder_id_pin_required, "builder_id_pin_required"),
        (
            contract.source_revision_pin_required,
            "source_revision_pin_required",
        ),
        (contract.sbom_required, "sbom_required"),
        (
            contract.vulnerability_scan_gate_required,
            "vulnerability_scan_gate_required",
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
    contract: &Fd001TenantImageProvenanceContract,
) -> Result<(), Fd001TenantImageProvenanceError> {
    if contract.image_registry_attached
        || contract.image_published
        || contract.cosign_runtime_verification_attached
        || contract.transparency_log_runtime_verified
        || contract.slsa_provenance_runtime_verified
        || contract.sbom_runtime_published
        || contract.vulnerability_scanner_attached
        || contract.admission_controller_runtime_attached
        || contract.workload_runtime_deployed
        || contract.cloud_substrate_runtime_attached
        || contract.runtime_audit_chain_emission_attached
    {
        return Err(Fd001TenantImageProvenanceError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_requirements(
    contract: &Fd001TenantImageProvenanceContract,
) -> Result<(), Fd001TenantImageProvenanceError> {
    let mut seen_requirements = BTreeSet::new();
    let mut seen_workloads = BTreeSet::new();
    let mut seen_control_kinds = BTreeSet::new();
    for requirement in &contract.requirements {
        validate_requirement(requirement)?;
        if !seen_requirements.insert(requirement.requirement_id) {
            return Err(Fd001TenantImageProvenanceError::DuplicateRequirement(
                requirement.requirement_id.to_owned(),
            ));
        }
        seen_workloads.insert(requirement.workload_kind);
        seen_control_kinds.insert(requirement.control_kind);
    }
    for workload_kind in required_workload_kinds() {
        if !seen_workloads.contains(&workload_kind) {
            return Err(Fd001TenantImageProvenanceError::MissingWorkloadKind(
                workload_kind,
            ));
        }
    }
    for control_kind in required_control_kinds() {
        if !seen_control_kinds.contains(&control_kind) {
            return Err(Fd001TenantImageProvenanceError::MissingControlKind(
                control_kind,
            ));
        }
    }
    Ok(())
}

fn validate_requirement(
    requirement: &Fd001TenantImageProvenanceRequirement,
) -> Result<(), Fd001TenantImageProvenanceError> {
    validate_slug(
        requirement.requirement_id,
        Fd001TenantImageProvenanceError::InvalidRequirementId,
    )?;
    validate_prefixed_ref(
        requirement.artifact_ref,
        "oci-image/fd001-tenant-rbac/",
        Fd001TenantImageProvenanceError::InvalidArtifactRef,
    )?;
    validate_prefixed_ref(
        requirement.policy_ref,
        POLICY_REF_PREFIX,
        Fd001TenantImageProvenanceError::InvalidPolicyRef,
    )?;
    validate_prefixed_ref(
        requirement.expected_evidence_ref,
        "evidence/supply-chain/fd001-tenant-rbac/",
        Fd001TenantImageProvenanceError::InvalidExpectedEvidenceRef,
    )?;
    validate_doc_url(requirement.official_doc_url)?;
    validate_prefixed_ref(
        requirement.source_manifest_ref,
        "crates/tenant-rbac-tenant-workload-manifest/",
        Fd001TenantImageProvenanceError::InvalidSourceManifestRef,
    )?;
    validate_prefixed_ref(
        requirement.source_admission_policy_ref,
        "crates/tenant-rbac-tenant-admission-policy/",
        Fd001TenantImageProvenanceError::InvalidSourceAdmissionPolicyRef,
    )?;
    if !requirement.applies_to_all_manifest_workloads {
        return Err(Fd001TenantImageProvenanceError::MissingRequiredControl(
            "requirement_applies_to_all_manifest_workloads",
        ));
    }
    if requirement.runtime_verification_attached || requirement.schema_version != SCHEMA_VERSION {
        return Err(Fd001TenantImageProvenanceError::RuntimeAttachmentOverclaim);
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

fn required_control_kinds() -> [Fd001TenantImageProvenanceControlKind; 11] {
    [
        Fd001TenantImageProvenanceControlKind::OciDigestPinned,
        Fd001TenantImageProvenanceControlKind::CosignSignatureRequired,
        Fd001TenantImageProvenanceControlKind::KeylessOidcIdentityRequired,
        Fd001TenantImageProvenanceControlKind::RekorTransparencyLogRequired,
        Fd001TenantImageProvenanceControlKind::IntotoStatementRequired,
        Fd001TenantImageProvenanceControlKind::SlsaProvenanceRequired,
        Fd001TenantImageProvenanceControlKind::BuilderIdPinned,
        Fd001TenantImageProvenanceControlKind::SourceRevisionPinned,
        Fd001TenantImageProvenanceControlKind::SbomRequired,
        Fd001TenantImageProvenanceControlKind::VulnerabilityScanGateRequired,
        Fd001TenantImageProvenanceControlKind::AdmissionPolicyEvidenceRequired,
    ]
}

fn validate_doc_url(url: &str) -> Result<(), Fd001TenantImageProvenanceError> {
    if has_unsafe_ref_text(url)
        || ![
            OCI_IMAGE_MANIFEST_DOC_URL,
            SIGSTORE_COSIGN_VERIFY_DOC_URL,
            SIGSTORE_COSIGN_VERIFY_OVERVIEW_DOC_URL,
            SLSA_SPEC_DOC_URL,
            SLSA_ATTESTATION_MODEL_DOC_URL,
            SLSA_BUILD_PROVENANCE_DOC_URL,
            SLSA_VERIFYING_ARTIFACTS_DOC_URL,
            IN_TOTO_SPECS_DOC_URL,
            SPDX_SPEC_DOC_URL,
            CYCLONEDX_SBOM_DOC_URL,
        ]
        .contains(&url)
    {
        return Err(Fd001TenantImageProvenanceError::InvalidOfficialDocUrl);
    }
    Ok(())
}

fn validate_slug(
    value: &str,
    error: Fd001TenantImageProvenanceError,
) -> Result<(), Fd001TenantImageProvenanceError> {
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

fn validate_tenant_namespace(value: &str) -> Result<(), Fd001TenantImageProvenanceError> {
    validate_slug(
        value,
        Fd001TenantImageProvenanceError::InvalidTenantNamespace,
    )?;
    if value != TENANT_NAMESPACE {
        return Err(Fd001TenantImageProvenanceError::InvalidTenantNamespace);
    }
    Ok(())
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: Fd001TenantImageProvenanceError,
) -> Result<(), Fd001TenantImageProvenanceError> {
    if value.len() <= prefix.len() || !value.starts_with(prefix) || has_unsafe_ref_text(value) {
        return Err(error);
    }
    Ok(())
}

fn require_control(
    value: bool,
    control: &'static str,
) -> Result<(), Fd001TenantImageProvenanceError> {
    if value {
        Ok(())
    } else {
        Err(Fd001TenantImageProvenanceError::MissingRequiredControl(
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
