use iam_tenant_rbac_tenant_residency_contract::{
    Fd001TenantResidencyError, Fd001TenantResidencyRequirementKind,
    fd001_tenant_residency_contract, fd001_tenant_residency_doc_urls,
    validate_fd001_tenant_residency_contract,
};
use iam_tenant_rbac_tenant_workload_manifest::Fd001TenantWorkloadKind;

#[test]
fn tenant_residency_contract_validates_controls_and_nonclaims() {
    let contract = fd001_tenant_residency_contract().expect("contract builds");
    validate_fd001_tenant_residency_contract(&contract).expect("contract validates");

    assert_eq!(
        contract.contract_name,
        "fd001-tenant-rbac-tenant-residency-contract"
    );
    assert_eq!(contract.program_name, "fd-001-tenant-rbac-generic");
    assert_eq!(contract.substrate_name, "oyatie-cloud");
    assert_eq!(contract.tenant_namespace, "oyatie-fd001-tenant-rbac-dev");
    assert_eq!(contract.tenant_cell_id, "cell-us-east-001");
    assert_eq!(contract.residency_region, "us-east-1");
    assert_eq!(contract.workload_manifest_count, 4);
    assert_eq!(
        contract.tenant_admission_policy_contract_name,
        "fd001-tenant-rbac-tenant-admission-policy-contract"
    );
    assert_eq!(contract.requirements.len(), 13);
    assert!(contract.official_docs_required);
    assert!(contract.all_manifest_workloads_in_scope);
    assert!(contract.tenant_residency_region_label_required);
    assert!(contract.namespace_residency_label_required);
    assert!(contract.workload_node_affinity_required);
    assert!(contract.topology_region_constraint_required);
    assert!(contract.storage_residency_policy_ref_required);
    assert!(contract.telemetry_residency_policy_ref_required);
    assert!(contract.audit_residency_policy_ref_required);
    assert!(contract.cross_region_egress_policy_ref_required);
    assert!(contract.tenant_model_jurisdiction_ref_required);
    assert!(contract.cell_placement_residency_ref_required);
    assert!(contract.admission_policy_evidence_required);
    assert!(contract.workload_manifest_evidence_required);
    assert!(contract.residency_audit_evidence_required);
    assert!(contract.review_only_contract);
    assert!(!contract.kubernetes_cluster_attached);
    assert!(!contract.namespace_created);
    assert!(!contract.node_affinity_applied);
    assert!(!contract.scheduler_runtime_observed);
    assert!(!contract.storage_residency_runtime_attached);
    assert!(!contract.telemetry_residency_runtime_attached);
    assert!(!contract.audit_residency_runtime_attached);
    assert!(!contract.cross_region_egress_runtime_observed);
    assert!(!contract.workload_runtime_deployed);
    assert!(!contract.cloud_substrate_runtime_attached);
    assert!(!contract.runtime_audit_chain_emission_attached);
}

#[test]
fn tenant_residency_contract_covers_workloads_requirements_and_docs() {
    let contract = fd001_tenant_residency_contract().expect("contract builds");
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
        Fd001TenantResidencyRequirementKind::TenantResidencyRegionLabelRequired,
        Fd001TenantResidencyRequirementKind::NamespaceResidencyLabelRequired,
        Fd001TenantResidencyRequirementKind::WorkloadNodeAffinityRequired,
        Fd001TenantResidencyRequirementKind::TopologyRegionConstraintRequired,
        Fd001TenantResidencyRequirementKind::StorageResidencyPolicyRefRequired,
        Fd001TenantResidencyRequirementKind::TelemetryResidencyPolicyRefRequired,
        Fd001TenantResidencyRequirementKind::AuditResidencyPolicyRefRequired,
        Fd001TenantResidencyRequirementKind::CrossRegionEgressPolicyRefRequired,
        Fd001TenantResidencyRequirementKind::TenantModelJurisdictionRefRequired,
        Fd001TenantResidencyRequirementKind::CellPlacementResidencyRefRequired,
        Fd001TenantResidencyRequirementKind::AdmissionPolicyEvidenceRequired,
        Fd001TenantResidencyRequirementKind::WorkloadManifestEvidenceRequired,
        Fd001TenantResidencyRequirementKind::ResidencyAuditEvidenceRequired,
    ] {
        assert!(requirement_kinds.contains(&kind), "missing {kind:?}");
    }

    let docs = fd001_tenant_residency_doc_urls(&contract);
    assert!(
        docs.contains(&"https://kubernetes.io/docs/concepts/overview/working-with-objects/labels/")
    );
    assert!(
        docs.contains(&"https://kubernetes.io/docs/concepts/scheduling-eviction/assign-pod-node/")
    );
    assert!(docs.contains(
        &"https://kubernetes.io/docs/concepts/scheduling-eviction/topology-spread-constraints/"
    ));
    assert!(
        docs.contains(&"https://kubernetes.io/docs/concepts/services-networking/network-policies/")
    );
    assert!(docs.contains(&"https://opentelemetry.io/docs/specs/semconv/resource/"));
    assert!(
        docs.contains(
            &"https://aws.amazon.com/blogs/security/establishing-a-data-perimeter-on-aws/"
        )
    );
}

#[test]
fn tenant_residency_contract_preserves_manifest_policy_refs_and_scope() {
    let contract = fd001_tenant_residency_contract().expect("contract builds");

    assert!(contract.requirements.iter().all(|requirement| {
        [
            "KubernetesLabel",
            "KubernetesNamespace",
            "NodeAffinity",
            "TopologySpreadConstraint",
            "StorageResidencyPolicy",
            "OpenTelemetryResource",
            "AuditResidencyPolicy",
            "NetworkPolicy",
            "TenantModelJurisdiction",
            "CellPlacement",
            "ValidatingAdmissionPolicy",
            "WorkloadManifest",
            "ResidencyAuditEvidence",
        ]
        .contains(&requirement.kubernetes_resource_kind)
            && requirement
                .policy_ref
                .starts_with("policy/residency/fd001/")
            && requirement
                .expected_evidence_ref
                .starts_with("evidence/residency/fd001-tenant-rbac/")
            && requirement
                .source_manifest_ref
                .starts_with("crates/tenant-rbac-tenant-workload-manifest/")
            && requirement
                .source_admission_policy_ref
                .starts_with("crates/tenant-rbac-tenant-admission-policy/")
            && requirement
                .source_tenant_model_ref
                .starts_with("specs/tenant-model.json")
            && requirement.applies_to_all_manifest_workloads
            && !requirement.runtime_observation_attached
    }));
}

#[test]
fn tenant_residency_contract_rejects_missing_duplicate_doc_and_ref_drift() {
    let mut contract = fd001_tenant_residency_contract().expect("contract builds");
    contract.requirements.truncate(3);
    assert_eq!(
        validate_fd001_tenant_residency_contract(&contract),
        Err(Fd001TenantResidencyError::MissingRequirements)
    );

    let mut contract = fd001_tenant_residency_contract().expect("contract builds");
    contract.requirements[1] = contract.requirements[0].clone();
    assert_eq!(
        validate_fd001_tenant_residency_contract(&contract),
        Err(Fd001TenantResidencyError::DuplicateRequirement(
            "tenant-residency-region-labels-all-workloads".to_owned()
        ))
    );

    let mut contract = fd001_tenant_residency_contract().expect("contract builds");
    contract.requirements[0].official_doc_url = "https://example.com/residency";
    assert_eq!(
        validate_fd001_tenant_residency_contract(&contract),
        Err(Fd001TenantResidencyError::InvalidOfficialDocUrl)
    );

    let mut contract = fd001_tenant_residency_contract().expect("contract builds");
    contract.requirements[0].policy_ref = "deploy/residency/fd001/labels";
    assert_eq!(
        validate_fd001_tenant_residency_contract(&contract),
        Err(Fd001TenantResidencyError::InvalidPolicyRef)
    );
}

#[test]
fn tenant_residency_contract_rejects_missing_controls_and_overclaims() {
    let mut contract = fd001_tenant_residency_contract().expect("contract builds");
    contract.tenant_residency_region_label_required = false;
    assert_eq!(
        validate_fd001_tenant_residency_contract(&contract),
        Err(Fd001TenantResidencyError::MissingRequiredControl(
            "tenant_residency_region_label_required"
        ))
    );

    let mut contract = fd001_tenant_residency_contract().expect("contract builds");
    contract.all_manifest_workloads_in_scope = false;
    assert_eq!(
        validate_fd001_tenant_residency_contract(&contract),
        Err(Fd001TenantResidencyError::MissingRequiredControl(
            "all_manifest_workloads_in_scope"
        ))
    );

    let mut contract = fd001_tenant_residency_contract().expect("contract builds");
    contract.requirements[0].applies_to_all_manifest_workloads = false;
    assert_eq!(
        validate_fd001_tenant_residency_contract(&contract),
        Err(Fd001TenantResidencyError::MissingRequiredControl(
            "requirement_applies_to_all_manifest_workloads"
        ))
    );

    let mut contract = fd001_tenant_residency_contract().expect("contract builds");
    contract.scheduler_runtime_observed = true;
    assert_eq!(
        validate_fd001_tenant_residency_contract(&contract),
        Err(Fd001TenantResidencyError::RuntimeAttachmentOverclaim)
    );
}
