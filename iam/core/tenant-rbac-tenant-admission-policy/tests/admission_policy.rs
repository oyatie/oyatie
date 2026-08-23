use iam_tenant_rbac_tenant_admission_policy::{
    Fd001TenantAdmissionPolicyError, Fd001TenantAdmissionRuleKind,
    fd001_tenant_admission_policy_contract, fd001_tenant_admission_policy_doc_urls,
    validate_fd001_tenant_admission_policy_contract,
};
use iam_tenant_rbac_tenant_workload_manifest::Fd001TenantWorkloadKind;

#[test]
fn tenant_admission_policy_contract_validates_controls_and_nonclaims() {
    let contract = fd001_tenant_admission_policy_contract().expect("contract builds");
    validate_fd001_tenant_admission_policy_contract(&contract).expect("contract validates");

    assert_eq!(
        contract.contract_name,
        "fd001-tenant-rbac-tenant-admission-policy-contract"
    );
    assert_eq!(contract.program_name, "fd-001-tenant-rbac-generic");
    assert_eq!(contract.substrate_name, "oyatie-cloud");
    assert_eq!(contract.tenant_namespace, "oyatie-fd001-tenant-rbac-dev");
    assert_eq!(contract.workload_manifest_count, 4);
    assert!(contract.all_manifest_workloads_in_scope);
    assert_eq!(contract.rules.len(), 11);
    assert!(contract.official_docs_required);
    assert!(contract.validating_admission_policy_required);
    assert!(contract.admission_binding_required);
    assert!(contract.failure_policy_fail_required);
    assert!(contract.deny_action_required);
    assert!(contract.pod_security_restricted_required);
    assert!(contract.digest_pinned_image_required);
    assert!(contract.latest_image_tag_forbidden);
    assert!(contract.tenant_labels_required);
    assert!(contract.resource_requests_limits_required);
    assert!(contract.service_account_boundary_required);
    assert!(contract.default_service_account_forbidden);
    assert!(contract.automount_service_account_token_forbidden);
    assert!(contract.resource_quota_required);
    assert!(contract.network_policy_default_deny_required);
    assert!(contract.admission_audit_annotation_required);
    assert!(contract.review_only_contract);
    assert!(!contract.kubernetes_cluster_attached);
    assert!(!contract.admission_controller_runtime_attached);
    assert!(!contract.admission_policy_applied);
    assert!(!contract.admission_runtime_enforced);
    assert!(!contract.workload_runtime_deployed);
    assert!(!contract.cloud_substrate_runtime_attached);
    assert!(!contract.runtime_audit_chain_emission_attached);
}

#[test]
fn tenant_admission_policy_contract_covers_workloads_rule_kinds_and_docs() {
    let contract = fd001_tenant_admission_policy_contract().expect("contract builds");
    let workloads = contract
        .rules
        .iter()
        .map(|rule| rule.workload_kind)
        .collect::<std::collections::BTreeSet<_>>();
    for workload in [
        Fd001TenantWorkloadKind::TenantRbac,
        Fd001TenantWorkloadKind::HrEmployment,
        Fd001TenantWorkloadKind::PayrollRun,
        Fd001TenantWorkloadKind::AccountingJournal,
    ] {
        assert!(workloads.contains(&workload), "missing {workload:?}");
    }

    let rule_kinds = contract
        .rules
        .iter()
        .map(|rule| rule.rule_kind)
        .collect::<std::collections::BTreeSet<_>>();
    for kind in [
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
    ] {
        assert!(rule_kinds.contains(&kind), "missing {kind:?}");
    }

    let docs = fd001_tenant_admission_policy_doc_urls(&contract);
    assert!(docs.contains(
        &"https://kubernetes.io/docs/reference/access-authn-authz/validating-admission-policy/"
    ));
    assert!(docs.contains(&"https://kubernetes.io/docs/concepts/security/pod-security-admission/"));
    assert!(docs.contains(&"https://kubernetes.io/docs/concepts/containers/images/"));
    assert!(docs.contains(
        &"https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/"
    ));
    assert!(docs.contains(&"https://kubernetes.io/docs/concepts/security/service-accounts/"));
    assert!(
        docs.contains(&"https://kubernetes.io/docs/concepts/services-networking/network-policies/")
    );
    assert!(docs.contains(&"https://kubernetes.io/docs/concepts/policy/resource-quotas/"));
    assert!(docs.contains(
        &"https://kubernetes.io/docs/reference/kubernetes-api/admissionregistration/validating-admission-policy-v1/"
    ));
}

#[test]
fn tenant_admission_policy_contract_preserves_fail_closed_deny_controls() {
    let contract = fd001_tenant_admission_policy_contract().expect("contract builds");

    assert!(contract.rules.iter().all(|rule| {
        rule.policy_api_kind == "ValidatingAdmissionPolicy"
            && rule.binding_api_kind == "ValidatingAdmissionPolicyBinding"
            && rule.failure_policy == "Fail"
            && rule.validation_action == "Deny"
            && rule.requires_audit_annotation
            && (rule.cel_expression_ref.starts_with("cel/fd001/")
                || rule
                    .cel_expression_ref
                    .starts_with("namespace-label/fd001/"))
            && rule
                .expected_evidence_ref
                .starts_with("evidence/tenant-admission/fd001-tenant-rbac/")
            && rule
                .source_manifest_ref
                .starts_with("crates/tenant-rbac-tenant-workload-manifest/")
            && rule.applies_to_all_manifest_workloads
            && !rule.runtime_enforcement_attached
    }));
}

#[test]
fn tenant_admission_policy_contract_rejects_missing_duplicate_doc_and_ref_drift() {
    let mut contract = fd001_tenant_admission_policy_contract().expect("contract builds");
    contract.rules.truncate(3);
    assert_eq!(
        validate_fd001_tenant_admission_policy_contract(&contract),
        Err(Fd001TenantAdmissionPolicyError::MissingRules)
    );

    let mut contract = fd001_tenant_admission_policy_contract().expect("contract builds");
    contract.rules[1] = contract.rules[0].clone();
    assert_eq!(
        validate_fd001_tenant_admission_policy_contract(&contract),
        Err(Fd001TenantAdmissionPolicyError::DuplicateRule(
            "tenant-labels-required-tenant-rbac".to_owned()
        ))
    );

    let mut contract = fd001_tenant_admission_policy_contract().expect("contract builds");
    contract.rules[0].official_doc_url = "https://example.com/kubernetes";
    assert_eq!(
        validate_fd001_tenant_admission_policy_contract(&contract),
        Err(Fd001TenantAdmissionPolicyError::InvalidOfficialDocUrl)
    );

    let mut contract = fd001_tenant_admission_policy_contract().expect("contract builds");
    contract.rules[0].cel_expression_ref = "deploy/fd001/tenant-labels";
    assert_eq!(
        validate_fd001_tenant_admission_policy_contract(&contract),
        Err(Fd001TenantAdmissionPolicyError::InvalidCelExpressionRef)
    );
}

#[test]
fn tenant_admission_policy_contract_rejects_missing_controls_and_overclaims() {
    let mut contract = fd001_tenant_admission_policy_contract().expect("contract builds");
    contract.failure_policy_fail_required = false;
    assert_eq!(
        validate_fd001_tenant_admission_policy_contract(&contract),
        Err(Fd001TenantAdmissionPolicyError::MissingRequiredControl(
            "failure_policy_fail_required"
        ))
    );

    let mut contract = fd001_tenant_admission_policy_contract().expect("contract builds");
    contract.all_manifest_workloads_in_scope = false;
    assert_eq!(
        validate_fd001_tenant_admission_policy_contract(&contract),
        Err(Fd001TenantAdmissionPolicyError::MissingRequiredControl(
            "all_manifest_workloads_in_scope"
        ))
    );

    let mut contract = fd001_tenant_admission_policy_contract().expect("contract builds");
    contract.rules[0].applies_to_all_manifest_workloads = false;
    assert_eq!(
        validate_fd001_tenant_admission_policy_contract(&contract),
        Err(Fd001TenantAdmissionPolicyError::MissingRequiredControl(
            "rule_applies_to_all_manifest_workloads"
        ))
    );

    let mut contract = fd001_tenant_admission_policy_contract().expect("contract builds");
    contract.admission_runtime_enforced = true;
    assert_eq!(
        validate_fd001_tenant_admission_policy_contract(&contract),
        Err(Fd001TenantAdmissionPolicyError::RuntimeAttachmentOverclaim)
    );

    let mut contract = fd001_tenant_admission_policy_contract().expect("contract builds");
    contract.rules[0].validation_action = "Warn";
    assert_eq!(
        validate_fd001_tenant_admission_policy_contract(&contract),
        Err(Fd001TenantAdmissionPolicyError::InvalidValidationAction)
    );
}
