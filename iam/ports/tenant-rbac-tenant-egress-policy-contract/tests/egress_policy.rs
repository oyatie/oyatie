use iam_tenant_rbac_tenant_egress_policy_contract::{
    Fd001TenantEgressPolicyError, Fd001TenantEgressPolicyRuleKind,
    fd001_tenant_egress_policy_contract, fd001_tenant_egress_policy_doc_urls,
    validate_fd001_tenant_egress_policy_contract,
};
use iam_tenant_rbac_tenant_workload_manifest::Fd001TenantWorkloadKind;

#[test]
fn tenant_egress_policy_contract_validates_controls_and_nonclaims() {
    let contract = fd001_tenant_egress_policy_contract().expect("contract builds");
    validate_fd001_tenant_egress_policy_contract(&contract).expect("contract validates");

    assert_eq!(
        contract.contract_name,
        "fd001-tenant-rbac-tenant-egress-policy-contract"
    );
    assert_eq!(contract.program_name, "fd-001-tenant-rbac-generic");
    assert_eq!(contract.substrate_name, "oyatie-cloud");
    assert_eq!(contract.tenant_namespace, "oyatie-fd001-tenant-rbac-dev");
    assert_eq!(contract.workload_manifest_count, 4);
    assert_eq!(
        contract.tenant_admission_policy_contract_name,
        "fd001-tenant-rbac-tenant-admission-policy-contract"
    );
    assert_eq!(contract.rules.len(), 11);
    assert!(contract.official_docs_required);
    assert!(contract.all_manifest_workloads_in_scope);
    assert!(contract.default_deny_egress_required);
    assert!(contract.dns_egress_only_required);
    assert!(contract.same_namespace_service_egress_required);
    assert!(contract.cross_namespace_egress_explicit_selector_required);
    assert!(contract.external_cidr_egress_forbidden_by_default);
    assert!(contract.ip_block_exception_evidence_required);
    assert!(contract.protocol_port_pinned_required);
    assert!(contract.tenant_label_selector_required);
    assert!(contract.network_policy_provider_evidence_required);
    assert!(contract.egress_audit_evidence_required);
    assert!(contract.admission_policy_evidence_required);
    assert!(contract.review_only_contract);
    assert!(!contract.kubernetes_cluster_attached);
    assert!(!contract.network_policy_provider_attached);
    assert!(!contract.network_policy_applied);
    assert!(!contract.egress_runtime_enforced);
    assert!(!contract.dns_probe_runtime_attached);
    assert!(!contract.external_egress_runtime_allowed);
    assert!(!contract.workload_runtime_deployed);
    assert!(!contract.cloud_substrate_runtime_attached);
    assert!(!contract.runtime_audit_chain_emission_attached);
}

#[test]
fn tenant_egress_policy_contract_covers_workloads_rules_and_docs() {
    let contract = fd001_tenant_egress_policy_contract().expect("contract builds");
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
    ] {
        assert!(rule_kinds.contains(&kind), "missing {kind:?}");
    }

    let docs = fd001_tenant_egress_policy_doc_urls(&contract);
    assert!(
        docs.contains(&"https://kubernetes.io/docs/concepts/services-networking/network-policies/")
    );
    assert!(
        docs.contains(
            &"https://kubernetes.io/docs/tasks/administer-cluster/declare-network-policy/"
        )
    );
    assert!(
        docs.contains(&"https://kubernetes.io/docs/concepts/services-networking/dns-pod-service/")
    );
    assert!(docs.contains(&"https://kubernetes.io/docs/concepts/services-networking/service/"));
    assert!(docs.contains(
        &"https://kubernetes.io/docs/concepts/overview/working-with-objects/namespaces/"
    ));
    assert!(docs.contains(&"https://kubernetes.io/docs/concepts/security/multi-tenancy/"));
}

#[test]
fn tenant_egress_policy_contract_preserves_network_policy_refs_and_scope() {
    let contract = fd001_tenant_egress_policy_contract().expect("contract builds");

    assert!(contract.rules.iter().all(|rule| {
        rule.network_policy_kind == "NetworkPolicy"
            && rule.policy_type == "Egress"
            && rule.policy_ref.starts_with("policy/network-egress/fd001/")
            && rule
                .expected_evidence_ref
                .starts_with("evidence/network-egress/fd001-tenant-rbac/")
            && rule
                .source_manifest_ref
                .starts_with("crates/tenant-rbac-tenant-workload-manifest/")
            && rule
                .source_admission_policy_ref
                .starts_with("crates/tenant-rbac-tenant-admission-policy/")
            && rule.applies_to_all_manifest_workloads
            && !rule.runtime_enforcement_attached
    }));
}

#[test]
fn tenant_egress_policy_contract_rejects_missing_duplicate_doc_and_ref_drift() {
    let mut contract = fd001_tenant_egress_policy_contract().expect("contract builds");
    contract.rules.truncate(3);
    assert_eq!(
        validate_fd001_tenant_egress_policy_contract(&contract),
        Err(Fd001TenantEgressPolicyError::MissingRules)
    );

    let mut contract = fd001_tenant_egress_policy_contract().expect("contract builds");
    contract.rules[1] = contract.rules[0].clone();
    assert_eq!(
        validate_fd001_tenant_egress_policy_contract(&contract),
        Err(Fd001TenantEgressPolicyError::DuplicateRule(
            "default-deny-egress-all-workloads".to_owned()
        ))
    );

    let mut contract = fd001_tenant_egress_policy_contract().expect("contract builds");
    contract.rules[0].official_doc_url = "https://example.com/network";
    assert_eq!(
        validate_fd001_tenant_egress_policy_contract(&contract),
        Err(Fd001TenantEgressPolicyError::InvalidOfficialDocUrl)
    );

    let mut contract = fd001_tenant_egress_policy_contract().expect("contract builds");
    contract.rules[0].policy_ref = "deploy/network-egress/fd001/default-deny";
    assert_eq!(
        validate_fd001_tenant_egress_policy_contract(&contract),
        Err(Fd001TenantEgressPolicyError::InvalidPolicyRef)
    );
}

#[test]
fn tenant_egress_policy_contract_rejects_missing_controls_and_overclaims() {
    let mut contract = fd001_tenant_egress_policy_contract().expect("contract builds");
    contract.default_deny_egress_required = false;
    assert_eq!(
        validate_fd001_tenant_egress_policy_contract(&contract),
        Err(Fd001TenantEgressPolicyError::MissingRequiredControl(
            "default_deny_egress_required"
        ))
    );

    let mut contract = fd001_tenant_egress_policy_contract().expect("contract builds");
    contract.all_manifest_workloads_in_scope = false;
    assert_eq!(
        validate_fd001_tenant_egress_policy_contract(&contract),
        Err(Fd001TenantEgressPolicyError::MissingRequiredControl(
            "all_manifest_workloads_in_scope"
        ))
    );

    let mut contract = fd001_tenant_egress_policy_contract().expect("contract builds");
    contract.rules[0].applies_to_all_manifest_workloads = false;
    assert_eq!(
        validate_fd001_tenant_egress_policy_contract(&contract),
        Err(Fd001TenantEgressPolicyError::MissingRequiredControl(
            "rule_applies_to_all_manifest_workloads"
        ))
    );

    let mut contract = fd001_tenant_egress_policy_contract().expect("contract builds");
    contract.egress_runtime_enforced = true;
    assert_eq!(
        validate_fd001_tenant_egress_policy_contract(&contract),
        Err(Fd001TenantEgressPolicyError::RuntimeAttachmentOverclaim)
    );
}
