use iam_tenant_rbac_tenant_resource_quota_contract::{
    Fd001TenantResourceQuotaError, Fd001TenantResourceQuotaRequirementKind,
    fd001_tenant_resource_quota_contract, fd001_tenant_resource_quota_doc_urls,
    validate_fd001_tenant_resource_quota_contract,
};
use iam_tenant_rbac_tenant_workload_manifest::Fd001TenantWorkloadKind;

#[test]
fn tenant_resource_quota_contract_validates_controls_and_nonclaims() {
    let contract = fd001_tenant_resource_quota_contract().expect("contract builds");
    validate_fd001_tenant_resource_quota_contract(&contract).expect("contract validates");

    assert_eq!(
        contract.contract_name,
        "fd001-tenant-rbac-tenant-resource-quota-contract"
    );
    assert_eq!(contract.program_name, "fd-001-tenant-rbac-generic");
    assert_eq!(contract.substrate_name, "oyatie-cloud");
    assert_eq!(contract.tenant_namespace, "oyatie-fd001-tenant-rbac-dev");
    assert_eq!(contract.workload_manifest_count, 4);
    assert_eq!(
        contract.tenant_admission_policy_contract_name,
        "fd001-tenant-rbac-tenant-admission-policy-contract"
    );
    assert_eq!(contract.requirements.len(), 13);
    assert!(contract.official_docs_required);
    assert!(contract.all_manifest_workloads_in_scope);
    assert!(contract.namespace_resource_quota_required);
    assert!(contract.compute_requests_quota_required);
    assert!(contract.compute_limits_quota_required);
    assert!(contract.object_count_quota_required);
    assert!(contract.persistent_storage_quota_required);
    assert!(contract.limit_range_defaults_required);
    assert!(contract.limit_range_min_max_required);
    assert!(contract.container_requests_limits_required);
    assert!(contract.resource_quota_admission_evidence_required);
    assert!(contract.limit_ranger_admission_evidence_required);
    assert!(contract.tenant_label_selector_required);
    assert!(contract.quota_usage_audit_evidence_required);
    assert!(contract.admission_policy_evidence_required);
    assert!(contract.review_only_contract);
    assert!(!contract.kubernetes_cluster_attached);
    assert!(!contract.resource_quota_applied);
    assert!(!contract.limit_range_applied);
    assert!(!contract.quota_admission_runtime_attached);
    assert!(!contract.limit_ranger_runtime_attached);
    assert!(!contract.quota_usage_runtime_observed);
    assert!(!contract.workload_runtime_deployed);
    assert!(!contract.cloud_substrate_runtime_attached);
    assert!(!contract.runtime_audit_chain_emission_attached);
}

#[test]
fn tenant_resource_quota_contract_covers_workloads_requirements_and_docs() {
    let contract = fd001_tenant_resource_quota_contract().expect("contract builds");
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

    let requirement_kinds = contract
        .requirements
        .iter()
        .map(|requirement| requirement.requirement_kind)
        .collect::<std::collections::BTreeSet<_>>();
    for kind in [
        Fd001TenantResourceQuotaRequirementKind::NamespaceResourceQuotaRequired,
        Fd001TenantResourceQuotaRequirementKind::ComputeRequestsQuotaRequired,
        Fd001TenantResourceQuotaRequirementKind::ComputeLimitsQuotaRequired,
        Fd001TenantResourceQuotaRequirementKind::ObjectCountQuotaRequired,
        Fd001TenantResourceQuotaRequirementKind::PersistentStorageQuotaRequired,
        Fd001TenantResourceQuotaRequirementKind::LimitRangeDefaultsRequired,
        Fd001TenantResourceQuotaRequirementKind::LimitRangeMinMaxRequired,
        Fd001TenantResourceQuotaRequirementKind::ContainerRequestsLimitsRequired,
        Fd001TenantResourceQuotaRequirementKind::ResourceQuotaAdmissionEvidenceRequired,
        Fd001TenantResourceQuotaRequirementKind::LimitRangerAdmissionEvidenceRequired,
        Fd001TenantResourceQuotaRequirementKind::TenantLabelSelectorRequired,
        Fd001TenantResourceQuotaRequirementKind::QuotaUsageAuditEvidenceRequired,
        Fd001TenantResourceQuotaRequirementKind::AdmissionPolicyEvidenceRequired,
    ] {
        assert!(requirement_kinds.contains(&kind), "missing {kind:?}");
    }

    let docs = fd001_tenant_resource_quota_doc_urls(&contract);
    assert!(docs.contains(&"https://kubernetes.io/docs/concepts/policy/resource-quotas/"));
    assert!(docs.contains(&"https://kubernetes.io/docs/concepts/policy/limit-range/"));
    assert!(docs.contains(
        &"https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/"
    ));
    assert!(docs.contains(
        &"https://kubernetes.io/docs/reference/access-authn-authz/admission-controllers/"
    ));
    assert!(docs.contains(
        &"https://kubernetes.io/docs/concepts/overview/working-with-objects/namespaces/"
    ));
    assert!(docs.contains(&"https://kubernetes.io/docs/concepts/security/multi-tenancy/"));
}

#[test]
fn tenant_resource_quota_contract_preserves_quota_refs_and_scope() {
    let contract = fd001_tenant_resource_quota_contract().expect("contract builds");

    assert!(contract.requirements.iter().all(|requirement| {
        [
            "ResourceQuota",
            "LimitRange",
            "PodSpecResources",
            "AdmissionController",
            "NamespaceLabel",
            "ResourceQuotaStatus",
            "ValidatingAdmissionPolicy",
        ]
        .contains(&requirement.kubernetes_resource_kind)
            && requirement
                .policy_ref
                .starts_with("policy/resource-quota/fd001/")
            && requirement
                .expected_evidence_ref
                .starts_with("evidence/resource-quota/fd001-tenant-rbac/")
            && requirement
                .source_manifest_ref
                .starts_with("crates/tenant-rbac-tenant-workload-manifest/")
            && requirement
                .source_admission_policy_ref
                .starts_with("crates/tenant-rbac-tenant-admission-policy/")
            && requirement.applies_to_all_manifest_workloads
            && !requirement.runtime_enforcement_attached
    }));
}

#[test]
fn tenant_resource_quota_contract_rejects_missing_duplicate_doc_and_ref_drift() {
    let mut contract = fd001_tenant_resource_quota_contract().expect("contract builds");
    contract.requirements.truncate(3);
    assert_eq!(
        validate_fd001_tenant_resource_quota_contract(&contract),
        Err(Fd001TenantResourceQuotaError::MissingRequirements)
    );

    let mut contract = fd001_tenant_resource_quota_contract().expect("contract builds");
    contract.requirements[1] = contract.requirements[0].clone();
    assert_eq!(
        validate_fd001_tenant_resource_quota_contract(&contract),
        Err(Fd001TenantResourceQuotaError::DuplicateRequirement(
            "namespace-resource-quota-all-workloads".to_owned()
        ))
    );

    let mut contract = fd001_tenant_resource_quota_contract().expect("contract builds");
    contract.requirements[0].official_doc_url = "https://example.com/quota";
    assert_eq!(
        validate_fd001_tenant_resource_quota_contract(&contract),
        Err(Fd001TenantResourceQuotaError::InvalidOfficialDocUrl)
    );

    let mut contract = fd001_tenant_resource_quota_contract().expect("contract builds");
    contract.requirements[0].policy_ref = "deploy/resource-quota/fd001/namespace";
    assert_eq!(
        validate_fd001_tenant_resource_quota_contract(&contract),
        Err(Fd001TenantResourceQuotaError::InvalidPolicyRef)
    );
}

#[test]
fn tenant_resource_quota_contract_rejects_missing_controls_and_overclaims() {
    let mut contract = fd001_tenant_resource_quota_contract().expect("contract builds");
    contract.namespace_resource_quota_required = false;
    assert_eq!(
        validate_fd001_tenant_resource_quota_contract(&contract),
        Err(Fd001TenantResourceQuotaError::MissingRequiredControl(
            "namespace_resource_quota_required"
        ))
    );

    let mut contract = fd001_tenant_resource_quota_contract().expect("contract builds");
    contract.all_manifest_workloads_in_scope = false;
    assert_eq!(
        validate_fd001_tenant_resource_quota_contract(&contract),
        Err(Fd001TenantResourceQuotaError::MissingRequiredControl(
            "all_manifest_workloads_in_scope"
        ))
    );

    let mut contract = fd001_tenant_resource_quota_contract().expect("contract builds");
    contract.requirements[0].applies_to_all_manifest_workloads = false;
    assert_eq!(
        validate_fd001_tenant_resource_quota_contract(&contract),
        Err(Fd001TenantResourceQuotaError::MissingRequiredControl(
            "requirement_applies_to_all_manifest_workloads"
        ))
    );

    let mut contract = fd001_tenant_resource_quota_contract().expect("contract builds");
    contract.resource_quota_applied = true;
    assert_eq!(
        validate_fd001_tenant_resource_quota_contract(&contract),
        Err(Fd001TenantResourceQuotaError::RuntimeAttachmentOverclaim)
    );
}
