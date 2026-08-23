use iam_tenant_rbac_tenant_availability_contract::{
    Fd001TenantAvailabilityError, Fd001TenantAvailabilityRequirementKind,
    fd001_tenant_availability_contract, fd001_tenant_availability_doc_urls,
    validate_fd001_tenant_availability_contract,
};
use iam_tenant_rbac_tenant_workload_manifest::Fd001TenantWorkloadKind;

#[test]
fn tenant_availability_contract_validates_controls_and_nonclaims() {
    let contract = fd001_tenant_availability_contract().expect("contract builds");
    validate_fd001_tenant_availability_contract(&contract).expect("contract validates");

    assert_eq!(
        contract.contract_name,
        "fd001-tenant-rbac-tenant-availability-contract"
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
    assert!(contract.pod_disruption_budget_required);
    assert!(contract.minimum_available_budget_required);
    assert!(contract.multi_replica_workload_required);
    assert!(contract.zone_topology_spread_required);
    assert!(contract.hostname_topology_spread_required);
    assert!(contract.pod_anti_affinity_required);
    assert!(contract.node_topology_label_evidence_required);
    assert!(contract.rolling_update_availability_required);
    assert!(contract.progress_deadline_required);
    assert!(contract.readiness_probe_evidence_required);
    assert!(contract.tenant_label_selector_required);
    assert!(contract.disruption_audit_evidence_required);
    assert!(contract.admission_policy_evidence_required);
    assert!(contract.review_only_contract);
    assert!(!contract.kubernetes_cluster_attached);
    assert!(!contract.pod_disruption_budget_applied);
    assert!(!contract.topology_spread_applied);
    assert!(!contract.pod_anti_affinity_applied);
    assert!(!contract.scheduler_runtime_observed);
    assert!(!contract.rolling_update_runtime_observed);
    assert!(!contract.readiness_probe_runtime_observed);
    assert!(!contract.workload_runtime_deployed);
    assert!(!contract.cloud_substrate_runtime_attached);
    assert!(!contract.runtime_audit_chain_emission_attached);
}

#[test]
fn tenant_availability_contract_covers_workloads_requirements_and_docs() {
    let contract = fd001_tenant_availability_contract().expect("contract builds");
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
        Fd001TenantAvailabilityRequirementKind::PodDisruptionBudgetRequired,
        Fd001TenantAvailabilityRequirementKind::MinimumAvailableBudgetRequired,
        Fd001TenantAvailabilityRequirementKind::MultiReplicaWorkloadRequired,
        Fd001TenantAvailabilityRequirementKind::ZoneTopologySpreadRequired,
        Fd001TenantAvailabilityRequirementKind::HostnameTopologySpreadRequired,
        Fd001TenantAvailabilityRequirementKind::PodAntiAffinityRequired,
        Fd001TenantAvailabilityRequirementKind::NodeTopologyLabelEvidenceRequired,
        Fd001TenantAvailabilityRequirementKind::RollingUpdateAvailabilityRequired,
        Fd001TenantAvailabilityRequirementKind::ProgressDeadlineRequired,
        Fd001TenantAvailabilityRequirementKind::ReadinessProbeEvidenceRequired,
        Fd001TenantAvailabilityRequirementKind::TenantLabelSelectorRequired,
        Fd001TenantAvailabilityRequirementKind::DisruptionAuditEvidenceRequired,
        Fd001TenantAvailabilityRequirementKind::AdmissionPolicyEvidenceRequired,
    ] {
        assert!(requirement_kinds.contains(&kind), "missing {kind:?}");
    }

    let docs = fd001_tenant_availability_doc_urls(&contract);
    assert!(docs.contains(&"https://kubernetes.io/docs/concepts/workloads/pods/disruptions/"));
    assert!(docs.contains(&"https://kubernetes.io/docs/tasks/run-application/configure-pdb/"));
    assert!(docs.contains(
        &"https://kubernetes.io/docs/reference/kubernetes-api/policy/pod-disruption-budget-v1/"
    ));
    assert!(docs.contains(
        &"https://kubernetes.io/docs/concepts/scheduling-eviction/topology-spread-constraints/"
    ));
    assert!(
        docs.contains(&"https://kubernetes.io/docs/concepts/scheduling-eviction/assign-pod-node/")
    );
    assert!(
        docs.contains(&"https://kubernetes.io/docs/concepts/workloads/controllers/deployment/")
    );
    assert!(
        docs.contains(
            &"https://kubernetes.io/docs/tasks/run-application/update-deployment-rolling/"
        )
    );
    assert!(docs.contains(
        &"https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/"
    ));
}

#[test]
fn tenant_availability_contract_preserves_availability_refs_and_scope() {
    let contract = fd001_tenant_availability_contract().expect("contract builds");

    assert!(contract.requirements.iter().all(|requirement| {
        [
            "PodDisruptionBudget",
            "Deployment",
            "TopologySpreadConstraint",
            "PodAntiAffinity",
            "NodeLabel",
            "DeploymentStrategy",
            "ReadinessProbe",
            "LabelSelector",
            "EvictionAuditEvidence",
            "ValidatingAdmissionPolicy",
        ]
        .contains(&requirement.kubernetes_resource_kind)
            && [
                "none",
                "topology.kubernetes.io/zone",
                "kubernetes.io/hostname",
            ]
            .contains(&requirement.topology_key)
            && requirement
                .policy_ref
                .starts_with("policy/availability/fd001/")
            && requirement
                .expected_evidence_ref
                .starts_with("evidence/availability/fd001-tenant-rbac/")
            && requirement
                .source_manifest_ref
                .starts_with("crates/tenant-rbac-tenant-workload-manifest/")
            && requirement
                .source_admission_policy_ref
                .starts_with("crates/tenant-rbac-tenant-admission-policy/")
            && requirement.applies_to_all_manifest_workloads
            && !requirement.runtime_observation_attached
    }));
}

#[test]
fn tenant_availability_contract_rejects_missing_duplicate_doc_and_ref_drift() {
    let mut contract = fd001_tenant_availability_contract().expect("contract builds");
    contract.requirements.truncate(3);
    assert_eq!(
        validate_fd001_tenant_availability_contract(&contract),
        Err(Fd001TenantAvailabilityError::MissingRequirements)
    );

    let mut contract = fd001_tenant_availability_contract().expect("contract builds");
    contract.requirements[1] = contract.requirements[0].clone();
    assert_eq!(
        validate_fd001_tenant_availability_contract(&contract),
        Err(Fd001TenantAvailabilityError::DuplicateRequirement(
            "pod-disruption-budget-all-workloads".to_owned()
        ))
    );

    let mut contract = fd001_tenant_availability_contract().expect("contract builds");
    contract.requirements[0].official_doc_url = "https://example.com/availability";
    assert_eq!(
        validate_fd001_tenant_availability_contract(&contract),
        Err(Fd001TenantAvailabilityError::InvalidOfficialDocUrl)
    );

    let mut contract = fd001_tenant_availability_contract().expect("contract builds");
    contract.requirements[0].policy_ref = "deploy/availability/fd001/pdb";
    assert_eq!(
        validate_fd001_tenant_availability_contract(&contract),
        Err(Fd001TenantAvailabilityError::InvalidPolicyRef)
    );
}

#[test]
fn tenant_availability_contract_rejects_missing_controls_and_overclaims() {
    let mut contract = fd001_tenant_availability_contract().expect("contract builds");
    contract.pod_disruption_budget_required = false;
    assert_eq!(
        validate_fd001_tenant_availability_contract(&contract),
        Err(Fd001TenantAvailabilityError::MissingRequiredControl(
            "pod_disruption_budget_required"
        ))
    );

    let mut contract = fd001_tenant_availability_contract().expect("contract builds");
    contract.all_manifest_workloads_in_scope = false;
    assert_eq!(
        validate_fd001_tenant_availability_contract(&contract),
        Err(Fd001TenantAvailabilityError::MissingRequiredControl(
            "all_manifest_workloads_in_scope"
        ))
    );

    let mut contract = fd001_tenant_availability_contract().expect("contract builds");
    contract.requirements[0].applies_to_all_manifest_workloads = false;
    assert_eq!(
        validate_fd001_tenant_availability_contract(&contract),
        Err(Fd001TenantAvailabilityError::MissingRequiredControl(
            "requirement_applies_to_all_manifest_workloads"
        ))
    );

    let mut contract = fd001_tenant_availability_contract().expect("contract builds");
    contract.pod_disruption_budget_applied = true;
    assert_eq!(
        validate_fd001_tenant_availability_contract(&contract),
        Err(Fd001TenantAvailabilityError::RuntimeAttachmentOverclaim)
    );
}
