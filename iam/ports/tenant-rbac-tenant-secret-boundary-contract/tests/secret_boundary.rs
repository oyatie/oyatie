use iam_tenant_rbac_tenant_secret_boundary_contract::{
    Fd001TenantSecretBoundaryControlKind, Fd001TenantSecretBoundaryError,
    fd001_tenant_secret_boundary_contract, fd001_tenant_secret_boundary_doc_urls,
    validate_fd001_tenant_secret_boundary_contract,
};
use iam_tenant_rbac_tenant_workload_manifest::Fd001TenantWorkloadKind;

#[test]
fn tenant_secret_boundary_contract_validates_controls_and_nonclaims() {
    let contract = fd001_tenant_secret_boundary_contract().expect("contract builds");
    validate_fd001_tenant_secret_boundary_contract(&contract).expect("contract validates");

    assert_eq!(
        contract.contract_name,
        "fd001-tenant-rbac-tenant-secret-boundary-contract"
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
    assert!(contract.inline_secret_material_forbidden);
    assert!(contract.kubernetes_secret_reference_required);
    assert!(contract.secret_at_rest_encryption_required);
    assert!(contract.rbac_least_privilege_required);
    assert!(contract.namespace_secret_isolation_required);
    assert!(contract.workload_scoped_service_account_required);
    assert!(contract.automount_service_account_token_forbidden);
    assert!(contract.short_lived_projected_token_boundary_required);
    assert!(contract.external_secret_store_boundary_required);
    assert!(contract.secret_rotation_evidence_required);
    assert!(contract.secret_access_audit_evidence_required);
    assert!(contract.review_only_contract);
    assert!(!contract.kubernetes_secret_created);
    assert!(!contract.secret_data_materialized);
    assert!(!contract.encryption_provider_runtime_attached);
    assert!(!contract.external_secret_store_runtime_attached);
    assert!(!contract.rbac_runtime_applied);
    assert!(!contract.projected_token_runtime_attached);
    assert!(!contract.secret_rotation_runtime_attached);
    assert!(!contract.secret_access_runtime_audited);
    assert!(!contract.admission_controller_runtime_attached);
    assert!(!contract.workload_runtime_deployed);
    assert!(!contract.cloud_substrate_runtime_attached);
    assert!(!contract.runtime_audit_chain_emission_attached);
}

#[test]
fn tenant_secret_boundary_contract_covers_workloads_controls_and_docs() {
    let contract = fd001_tenant_secret_boundary_contract().expect("contract builds");
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
    ] {
        assert!(controls.contains(&control), "missing {control:?}");
    }

    let docs = fd001_tenant_secret_boundary_doc_urls(&contract);
    assert!(docs.contains(&"https://kubernetes.io/docs/concepts/configuration/secret/"));
    assert!(docs.contains(&"https://kubernetes.io/docs/concepts/security/secrets-good-practices/"));
    assert!(docs.contains(&"https://kubernetes.io/docs/tasks/administer-cluster/encrypt-data/"));
    assert!(docs.contains(&"https://kubernetes.io/docs/concepts/security/rbac-good-practices/"));
    assert!(docs.contains(&"https://kubernetes.io/docs/reference/access-authn-authz/rbac/"));
    assert!(docs.contains(&"https://kubernetes.io/docs/concepts/security/service-accounts/"));
    assert!(docs.contains(&"https://kubernetes.io/docs/concepts/storage/projected-volumes/"));
}

#[test]
fn tenant_secret_boundary_contract_preserves_refs_and_all_workload_scope() {
    let contract = fd001_tenant_secret_boundary_contract().expect("contract builds");

    assert!(contract.requirements.iter().all(|requirement| {
        requirement
            .boundary_ref
            .starts_with("secret-boundary/fd001-tenant-rbac/")
            && requirement
                .policy_ref
                .starts_with("policy/secret-boundary/fd001/")
            && requirement
                .expected_evidence_ref
                .starts_with("evidence/secret-boundary/fd001-tenant-rbac/")
            && requirement
                .source_manifest_ref
                .starts_with("crates/tenant-rbac-tenant-workload-manifest/")
            && requirement
                .source_admission_policy_ref
                .starts_with("crates/tenant-rbac-tenant-admission-policy/")
            && requirement.applies_to_all_manifest_workloads
            && !requirement.runtime_secret_material_attached
    }));
}

#[test]
fn tenant_secret_boundary_contract_rejects_missing_duplicate_doc_and_ref_drift() {
    let mut contract = fd001_tenant_secret_boundary_contract().expect("contract builds");
    contract.requirements.truncate(3);
    assert_eq!(
        validate_fd001_tenant_secret_boundary_contract(&contract),
        Err(Fd001TenantSecretBoundaryError::MissingRequirements)
    );

    let mut contract = fd001_tenant_secret_boundary_contract().expect("contract builds");
    contract.requirements[1] = contract.requirements[0].clone();
    assert_eq!(
        validate_fd001_tenant_secret_boundary_contract(&contract),
        Err(Fd001TenantSecretBoundaryError::DuplicateRequirement(
            "no-inline-secret-material-all-workloads".to_owned()
        ))
    );

    let mut contract = fd001_tenant_secret_boundary_contract().expect("contract builds");
    contract.requirements[0].official_doc_url = "https://example.com/secrets";
    assert_eq!(
        validate_fd001_tenant_secret_boundary_contract(&contract),
        Err(Fd001TenantSecretBoundaryError::InvalidOfficialDocUrl)
    );

    let mut contract = fd001_tenant_secret_boundary_contract().expect("contract builds");
    contract.requirements[0].policy_ref = "deploy/secret-boundary/fd001/inline";
    assert_eq!(
        validate_fd001_tenant_secret_boundary_contract(&contract),
        Err(Fd001TenantSecretBoundaryError::InvalidPolicyRef)
    );
}

#[test]
fn tenant_secret_boundary_contract_rejects_missing_controls_and_overclaims() {
    let mut contract = fd001_tenant_secret_boundary_contract().expect("contract builds");
    contract.inline_secret_material_forbidden = false;
    assert_eq!(
        validate_fd001_tenant_secret_boundary_contract(&contract),
        Err(Fd001TenantSecretBoundaryError::MissingRequiredControl(
            "inline_secret_material_forbidden"
        ))
    );

    let mut contract = fd001_tenant_secret_boundary_contract().expect("contract builds");
    contract.all_manifest_workloads_in_scope = false;
    assert_eq!(
        validate_fd001_tenant_secret_boundary_contract(&contract),
        Err(Fd001TenantSecretBoundaryError::MissingRequiredControl(
            "all_manifest_workloads_in_scope"
        ))
    );

    let mut contract = fd001_tenant_secret_boundary_contract().expect("contract builds");
    contract.requirements[0].applies_to_all_manifest_workloads = false;
    assert_eq!(
        validate_fd001_tenant_secret_boundary_contract(&contract),
        Err(Fd001TenantSecretBoundaryError::MissingRequiredControl(
            "requirement_applies_to_all_manifest_workloads"
        ))
    );

    let mut contract = fd001_tenant_secret_boundary_contract().expect("contract builds");
    contract.secret_data_materialized = true;
    assert_eq!(
        validate_fd001_tenant_secret_boundary_contract(&contract),
        Err(Fd001TenantSecretBoundaryError::RuntimeAttachmentOverclaim)
    );
}
