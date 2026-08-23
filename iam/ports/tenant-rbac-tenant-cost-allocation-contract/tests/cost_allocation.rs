use iam_tenant_rbac_tenant_cost_allocation_contract::{
    Fd001TenantCostAllocationError, Fd001TenantCostAllocationRequirementKind,
    fd001_tenant_cost_allocation_contract, fd001_tenant_cost_allocation_doc_urls,
    validate_fd001_tenant_cost_allocation_contract,
};
use iam_tenant_rbac_tenant_workload_manifest::Fd001TenantWorkloadKind;

#[test]
fn tenant_cost_allocation_contract_validates_controls_and_nonclaims() {
    let contract = fd001_tenant_cost_allocation_contract().expect("contract builds");
    validate_fd001_tenant_cost_allocation_contract(&contract).expect("contract validates");

    assert_eq!(
        contract.contract_name,
        "fd001-tenant-rbac-tenant-cost-allocation-contract"
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
    assert!(contract.tenant_cost_allocation_labels_required);
    assert!(contract.kubernetes_recommended_labels_required);
    assert!(contract.namespace_cost_boundary_required);
    assert!(contract.workload_resource_requests_required);
    assert!(contract.resource_quota_usage_evidence_required);
    assert!(contract.opentelemetry_service_resource_required);
    assert!(contract.opentelemetry_kubernetes_resource_attributes_required);
    assert!(contract.finops_allocation_strategy_required);
    assert!(contract.shared_cost_policy_required);
    assert!(contract.allocation_coverage_kpi_required);
    assert!(contract.tenant_label_selector_required);
    assert!(contract.cost_allocation_audit_evidence_required);
    assert!(contract.admission_policy_evidence_required);
    assert!(contract.review_only_contract);
    assert!(!contract.kubernetes_cluster_attached);
    assert!(!contract.resource_metrics_runtime_attached);
    assert!(!contract.otel_collector_runtime_attached);
    assert!(!contract.finops_runtime_attached);
    assert!(!contract.cost_report_runtime_generated);
    assert!(!contract.billing_export_runtime_attached);
    assert!(!contract.workload_runtime_deployed);
    assert!(!contract.cloud_substrate_runtime_attached);
    assert!(!contract.runtime_audit_chain_emission_attached);
}

#[test]
fn tenant_cost_allocation_contract_covers_workloads_requirements_and_docs() {
    let contract = fd001_tenant_cost_allocation_contract().expect("contract builds");
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
        Fd001TenantCostAllocationRequirementKind::TenantCostAllocationLabelsRequired,
        Fd001TenantCostAllocationRequirementKind::KubernetesRecommendedLabelsRequired,
        Fd001TenantCostAllocationRequirementKind::NamespaceCostBoundaryRequired,
        Fd001TenantCostAllocationRequirementKind::WorkloadResourceRequestsRequired,
        Fd001TenantCostAllocationRequirementKind::ResourceQuotaUsageEvidenceRequired,
        Fd001TenantCostAllocationRequirementKind::OpenTelemetryServiceResourceRequired,
        Fd001TenantCostAllocationRequirementKind::OpenTelemetryKubernetesResourceAttributesRequired,
        Fd001TenantCostAllocationRequirementKind::FinOpsAllocationStrategyRequired,
        Fd001TenantCostAllocationRequirementKind::SharedCostPolicyRequired,
        Fd001TenantCostAllocationRequirementKind::AllocationCoverageKpiRequired,
        Fd001TenantCostAllocationRequirementKind::TenantLabelSelectorRequired,
        Fd001TenantCostAllocationRequirementKind::CostAllocationAuditEvidenceRequired,
        Fd001TenantCostAllocationRequirementKind::AdmissionPolicyEvidenceRequired,
    ] {
        assert!(requirement_kinds.contains(&kind), "missing {kind:?}");
    }

    let docs = fd001_tenant_cost_allocation_doc_urls(&contract);
    assert!(docs.contains(&"https://www.finops.org/framework/capabilities/allocation/"));
    assert!(docs.contains(
        &"https://kubernetes.io/docs/concepts/overview/working-with-objects/common-labels/"
    ));
    assert!(
        docs.contains(&"https://kubernetes.io/docs/concepts/overview/working-with-objects/labels/")
    );
    assert!(docs.contains(
        &"https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/"
    ));
    assert!(docs.contains(&"https://kubernetes.io/docs/concepts/policy/resource-quotas/"));
    assert!(docs.contains(&"https://opentelemetry.io/docs/concepts/resources/"));
    assert!(docs.contains(&"https://opentelemetry.io/docs/specs/semconv/resource/"));
    assert!(docs.contains(&"https://opentelemetry.io/docs/specs/semconv/resource/k8s/"));
}

#[test]
fn tenant_cost_allocation_contract_preserves_cost_refs_and_scope() {
    let contract = fd001_tenant_cost_allocation_contract().expect("contract builds");

    assert!(contract.requirements.iter().all(|requirement| {
        [
            "KubernetesLabel",
            "KubernetesNamespace",
            "PodSpecResources",
            "ResourceQuotaStatus",
            "OpenTelemetryResource",
            "OpenTelemetryKubernetesResource",
            "FinOpsAllocationStrategy",
            "SharedCostPolicy",
            "AllocationKpi",
            "LabelSelector",
            "CostAllocationAuditEvidence",
            "ValidatingAdmissionPolicy",
        ]
        .contains(&requirement.kubernetes_resource_kind)
            && requirement
                .policy_ref
                .starts_with("policy/cost-allocation/fd001/")
            && requirement
                .expected_evidence_ref
                .starts_with("evidence/cost-allocation/fd001-tenant-rbac/")
            && requirement
                .source_manifest_ref
                .starts_with("crates/tenant-rbac-tenant-workload-manifest/")
            && requirement
                .source_admission_policy_ref
                .starts_with("crates/tenant-rbac-tenant-admission-policy/")
            && requirement
                .source_finops_ref
                .starts_with("crates/cloud-finops-kernel/")
            && requirement.applies_to_all_manifest_workloads
            && !requirement.runtime_observation_attached
    }));
}

#[test]
fn tenant_cost_allocation_contract_rejects_missing_duplicate_doc_and_ref_drift() {
    let mut contract = fd001_tenant_cost_allocation_contract().expect("contract builds");
    contract.requirements.truncate(3);
    assert_eq!(
        validate_fd001_tenant_cost_allocation_contract(&contract),
        Err(Fd001TenantCostAllocationError::MissingRequirements)
    );

    let mut contract = fd001_tenant_cost_allocation_contract().expect("contract builds");
    contract.requirements[1] = contract.requirements[0].clone();
    assert_eq!(
        validate_fd001_tenant_cost_allocation_contract(&contract),
        Err(Fd001TenantCostAllocationError::DuplicateRequirement(
            "tenant-cost-allocation-labels-all-workloads".to_owned()
        ))
    );

    let mut contract = fd001_tenant_cost_allocation_contract().expect("contract builds");
    contract.requirements[0].official_doc_url = "https://example.com/cost";
    assert_eq!(
        validate_fd001_tenant_cost_allocation_contract(&contract),
        Err(Fd001TenantCostAllocationError::InvalidOfficialDocUrl)
    );

    let mut contract = fd001_tenant_cost_allocation_contract().expect("contract builds");
    contract.requirements[0].policy_ref = "deploy/cost-allocation/fd001/labels";
    assert_eq!(
        validate_fd001_tenant_cost_allocation_contract(&contract),
        Err(Fd001TenantCostAllocationError::InvalidPolicyRef)
    );
}

#[test]
fn tenant_cost_allocation_contract_rejects_missing_controls_and_overclaims() {
    let mut contract = fd001_tenant_cost_allocation_contract().expect("contract builds");
    contract.tenant_cost_allocation_labels_required = false;
    assert_eq!(
        validate_fd001_tenant_cost_allocation_contract(&contract),
        Err(Fd001TenantCostAllocationError::MissingRequiredControl(
            "tenant_cost_allocation_labels_required"
        ))
    );

    let mut contract = fd001_tenant_cost_allocation_contract().expect("contract builds");
    contract.all_manifest_workloads_in_scope = false;
    assert_eq!(
        validate_fd001_tenant_cost_allocation_contract(&contract),
        Err(Fd001TenantCostAllocationError::MissingRequiredControl(
            "all_manifest_workloads_in_scope"
        ))
    );

    let mut contract = fd001_tenant_cost_allocation_contract().expect("contract builds");
    contract.requirements[0].applies_to_all_manifest_workloads = false;
    assert_eq!(
        validate_fd001_tenant_cost_allocation_contract(&contract),
        Err(Fd001TenantCostAllocationError::MissingRequiredControl(
            "requirement_applies_to_all_manifest_workloads"
        ))
    );

    let mut contract = fd001_tenant_cost_allocation_contract().expect("contract builds");
    contract.cost_report_runtime_generated = true;
    assert_eq!(
        validate_fd001_tenant_cost_allocation_contract(&contract),
        Err(Fd001TenantCostAllocationError::RuntimeAttachmentOverclaim)
    );
}
