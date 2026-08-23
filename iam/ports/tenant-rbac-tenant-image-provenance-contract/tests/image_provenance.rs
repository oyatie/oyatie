use iam_tenant_rbac_tenant_image_provenance_contract::{
    Fd001TenantImageProvenanceControlKind, Fd001TenantImageProvenanceError,
    fd001_tenant_image_provenance_contract, fd001_tenant_image_provenance_doc_urls,
    validate_fd001_tenant_image_provenance_contract,
};
use iam_tenant_rbac_tenant_workload_manifest::Fd001TenantWorkloadKind;

#[test]
fn tenant_image_provenance_contract_validates_controls_and_nonclaims() {
    let contract = fd001_tenant_image_provenance_contract().expect("contract builds");
    validate_fd001_tenant_image_provenance_contract(&contract).expect("contract validates");

    assert_eq!(
        contract.contract_name,
        "fd001-tenant-rbac-tenant-image-provenance-contract"
    );
    assert_eq!(contract.program_name, "fd-001-tenant-rbac-generic");
    assert_eq!(contract.substrate_name, "oyatie-cloud");
    assert_eq!(contract.tenant_namespace, "oyatie-fd001-tenant-rbac-dev");
    assert_eq!(contract.workload_manifest_count, 4);
    assert_eq!(
        contract.tenant_admission_policy_contract_name,
        "fd001-tenant-rbac-tenant-admission-policy-contract"
    );
    assert_eq!(contract.requirements.len(), 11);
    assert!(contract.official_docs_required);
    assert!(contract.all_manifest_workloads_in_scope);
    assert!(contract.oci_digest_pinned_required);
    assert!(contract.cosign_signature_required);
    assert!(contract.keyless_oidc_identity_required);
    assert!(contract.transparency_log_required);
    assert!(contract.intoto_statement_required);
    assert!(contract.slsa_provenance_required);
    assert!(contract.builder_id_pin_required);
    assert!(contract.source_revision_pin_required);
    assert!(contract.sbom_required);
    assert!(contract.vulnerability_scan_gate_required);
    assert!(contract.admission_policy_evidence_required);
    assert!(contract.review_only_contract);
    assert!(!contract.image_registry_attached);
    assert!(!contract.image_published);
    assert!(!contract.cosign_runtime_verification_attached);
    assert!(!contract.transparency_log_runtime_verified);
    assert!(!contract.slsa_provenance_runtime_verified);
    assert!(!contract.sbom_runtime_published);
    assert!(!contract.vulnerability_scanner_attached);
    assert!(!contract.admission_controller_runtime_attached);
    assert!(!contract.workload_runtime_deployed);
    assert!(!contract.cloud_substrate_runtime_attached);
    assert!(!contract.runtime_audit_chain_emission_attached);
}

#[test]
fn tenant_image_provenance_contract_covers_workloads_controls_and_docs() {
    let contract = fd001_tenant_image_provenance_contract().expect("contract builds");
    let workloads = contract
        .requirements
        .iter()
        .map(|requirement| requirement.workload_kind)
        .collect::<std::collections::BTreeSet<_>>();
    for workload in [
        Fd001TenantWorkloadKind::TenantRbac,
        Fd001TenantWorkloadKind::HrEmployment,
        Fd001TenantWorkloadKind::PayrollRun,
        Fd001TenantWorkloadKind::AccountingJournal,
    ] {
        assert!(workloads.contains(&workload), "missing {workload:?}");
    }

    let controls = contract
        .requirements
        .iter()
        .map(|requirement| requirement.control_kind)
        .collect::<std::collections::BTreeSet<_>>();
    for control in [
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
    ] {
        assert!(controls.contains(&control), "missing {control:?}");
    }

    let docs = fd001_tenant_image_provenance_doc_urls(&contract);
    assert!(docs.contains(&"https://specs.opencontainers.org/image-spec/manifest/?v=v1.1.0"));
    assert!(docs.contains(&"https://docs.sigstore.dev/cosign/verifying/verify/"));
    assert!(docs.contains(&"https://docs.sigstore.dev/cosign/verifying/"));
    assert!(docs.contains(&"https://slsa.dev/spec/v1.2/build-provenance"));
    assert!(docs.contains(&"https://slsa.dev/spec/v1.2/verifying-artifacts"));
    assert!(docs.contains(&"https://slsa.dev/spec/v1.2/attestation-model"));
    assert!(docs.contains(&"https://in-toto.io/docs/specs/"));
    assert!(docs.contains(&"https://spdx.github.io/spdx-spec/v2.3/"));
    assert!(docs.contains(&"https://cyclonedx.org/capabilities/sbom/"));
}

#[test]
fn tenant_image_provenance_contract_preserves_refs_and_all_workload_scope() {
    let contract = fd001_tenant_image_provenance_contract().expect("contract builds");

    assert!(contract.requirements.iter().all(|requirement| {
        requirement
            .artifact_ref
            .starts_with("oci-image/fd001-tenant-rbac/")
            && requirement
                .policy_ref
                .starts_with("policy/supply-chain/fd001/")
            && requirement
                .expected_evidence_ref
                .starts_with("evidence/supply-chain/fd001-tenant-rbac/")
            && requirement
                .source_manifest_ref
                .starts_with("crates/tenant-rbac-tenant-workload-manifest/")
            && requirement
                .source_admission_policy_ref
                .starts_with("crates/tenant-rbac-tenant-admission-policy/")
            && requirement.applies_to_all_manifest_workloads
            && !requirement.runtime_verification_attached
    }));
}

#[test]
fn tenant_image_provenance_contract_rejects_missing_duplicate_doc_and_ref_drift() {
    let mut contract = fd001_tenant_image_provenance_contract().expect("contract builds");
    contract.requirements.truncate(3);
    assert_eq!(
        validate_fd001_tenant_image_provenance_contract(&contract),
        Err(Fd001TenantImageProvenanceError::MissingRequirements)
    );

    let mut contract = fd001_tenant_image_provenance_contract().expect("contract builds");
    contract.requirements[1] = contract.requirements[0].clone();
    assert_eq!(
        validate_fd001_tenant_image_provenance_contract(&contract),
        Err(Fd001TenantImageProvenanceError::DuplicateRequirement(
            "oci-digest-pinned-all-workloads".to_owned()
        ))
    );

    let mut contract = fd001_tenant_image_provenance_contract().expect("contract builds");
    contract.requirements[0].official_doc_url = "https://example.com/provenance";
    assert_eq!(
        validate_fd001_tenant_image_provenance_contract(&contract),
        Err(Fd001TenantImageProvenanceError::InvalidOfficialDocUrl)
    );

    let mut contract = fd001_tenant_image_provenance_contract().expect("contract builds");
    contract.requirements[0].policy_ref = "deploy/supply-chain/fd001/oci";
    assert_eq!(
        validate_fd001_tenant_image_provenance_contract(&contract),
        Err(Fd001TenantImageProvenanceError::InvalidPolicyRef)
    );
}

#[test]
fn tenant_image_provenance_contract_rejects_missing_controls_and_overclaims() {
    let mut contract = fd001_tenant_image_provenance_contract().expect("contract builds");
    contract.cosign_signature_required = false;
    assert_eq!(
        validate_fd001_tenant_image_provenance_contract(&contract),
        Err(Fd001TenantImageProvenanceError::MissingRequiredControl(
            "cosign_signature_required"
        ))
    );

    let mut contract = fd001_tenant_image_provenance_contract().expect("contract builds");
    contract.all_manifest_workloads_in_scope = false;
    assert_eq!(
        validate_fd001_tenant_image_provenance_contract(&contract),
        Err(Fd001TenantImageProvenanceError::MissingRequiredControl(
            "all_manifest_workloads_in_scope"
        ))
    );

    let mut contract = fd001_tenant_image_provenance_contract().expect("contract builds");
    contract.requirements[0].applies_to_all_manifest_workloads = false;
    assert_eq!(
        validate_fd001_tenant_image_provenance_contract(&contract),
        Err(Fd001TenantImageProvenanceError::MissingRequiredControl(
            "requirement_applies_to_all_manifest_workloads"
        ))
    );

    let mut contract = fd001_tenant_image_provenance_contract().expect("contract builds");
    contract.cosign_runtime_verification_attached = true;
    assert_eq!(
        validate_fd001_tenant_image_provenance_contract(&contract),
        Err(Fd001TenantImageProvenanceError::RuntimeAttachmentOverclaim)
    );
}
