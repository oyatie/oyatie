use iam_tenant_rbac_tenant_autoscaling_contract::{
    Fd001TenantAutoscalingError, Fd001TenantAutoscalingRequirementKind,
    fd001_tenant_autoscaling_contract, fd001_tenant_autoscaling_doc_urls,
    validate_fd001_tenant_autoscaling_contract,
};
use iam_tenant_rbac_tenant_workload_manifest::Fd001TenantWorkloadKind;

#[test]
fn tenant_autoscaling_contract_validates_controls_and_nonclaims() {
    let contract = fd001_tenant_autoscaling_contract().expect("contract builds");
    validate_fd001_tenant_autoscaling_contract(&contract).expect("contract validates");

    assert_eq!(
        contract.contract_name,
        "fd001-tenant-rbac-tenant-autoscaling-contract"
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
    assert!(contract.horizontal_pod_autoscaler_required);
    assert!(contract.autoscaling_v2_api_required);
    assert!(contract.min_replica_floor_required);
    assert!(contract.max_replica_ceiling_required);
    assert!(contract.cpu_resource_metric_required);
    assert!(contract.memory_resource_metric_required);
    assert!(contract.metrics_pipeline_evidence_required);
    assert!(contract.scale_up_behavior_policy_required);
    assert!(contract.scale_down_behavior_policy_required);
    assert!(contract.stabilization_window_required);
    assert!(contract.tenant_label_selector_required);
    assert!(contract.scaling_audit_evidence_required);
    assert!(contract.admission_policy_evidence_required);
    assert!(contract.review_only_contract);
    assert!(!contract.kubernetes_cluster_attached);
    assert!(!contract.metrics_server_runtime_attached);
    assert!(!contract.custom_metrics_api_attached);
    assert!(!contract.horizontal_pod_autoscaler_applied);
    assert!(!contract.autoscaling_controller_runtime_observed);
    assert!(!contract.scale_event_runtime_observed);
    assert!(!contract.workload_runtime_deployed);
    assert!(!contract.cloud_substrate_runtime_attached);
    assert!(!contract.runtime_audit_chain_emission_attached);
}

#[test]
fn tenant_autoscaling_contract_covers_workloads_requirements_and_docs() {
    let contract = fd001_tenant_autoscaling_contract().expect("contract builds");
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
        Fd001TenantAutoscalingRequirementKind::HorizontalPodAutoscalerRequired,
        Fd001TenantAutoscalingRequirementKind::AutoscalingV2ApiRequired,
        Fd001TenantAutoscalingRequirementKind::MinReplicaFloorRequired,
        Fd001TenantAutoscalingRequirementKind::MaxReplicaCeilingRequired,
        Fd001TenantAutoscalingRequirementKind::CpuResourceMetricRequired,
        Fd001TenantAutoscalingRequirementKind::MemoryResourceMetricRequired,
        Fd001TenantAutoscalingRequirementKind::MetricsPipelineEvidenceRequired,
        Fd001TenantAutoscalingRequirementKind::ScaleUpBehaviorPolicyRequired,
        Fd001TenantAutoscalingRequirementKind::ScaleDownBehaviorPolicyRequired,
        Fd001TenantAutoscalingRequirementKind::StabilizationWindowRequired,
        Fd001TenantAutoscalingRequirementKind::TenantLabelSelectorRequired,
        Fd001TenantAutoscalingRequirementKind::ScalingAuditEvidenceRequired,
        Fd001TenantAutoscalingRequirementKind::AdmissionPolicyEvidenceRequired,
    ] {
        assert!(requirement_kinds.contains(&kind), "missing {kind:?}");
    }

    let docs = fd001_tenant_autoscaling_doc_urls(&contract);
    assert!(docs.contains(
        &"https://kubernetes.io/docs/concepts/workloads/autoscaling/horizontal-pod-autoscale/"
    ));
    assert!(docs.contains(
        &"https://kubernetes.io/docs/tasks/run-application/horizontal-pod-autoscale-walkthrough/"
    ));
    assert!(docs.contains(
        &"https://kubernetes.io/docs/tasks/debug/debug-cluster/resource-metrics-pipeline/"
    ));
    assert!(docs.contains(&"https://kubernetes.io/docs/concepts/workloads/autoscaling/"));
    assert!(docs.contains(
        &"https://kubernetes.io/docs/reference/kubernetes-api/autoscaling/horizontal-pod-autoscaler-v2/"
    ));
    assert!(docs.contains(
        &"https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/"
    ));
}

#[test]
fn tenant_autoscaling_contract_preserves_autoscaling_refs_and_scope() {
    let contract = fd001_tenant_autoscaling_contract().expect("contract builds");

    assert!(contract.requirements.iter().all(|requirement| {
        [
            "HorizontalPodAutoscaler",
            "AutoscalingV2Api",
            "ReplicaBounds",
            "ResourceMetric",
            "MetricsPipeline",
            "HorizontalPodAutoscalerBehavior",
            "LabelSelector",
            "ScalingAuditEvidence",
            "ValidatingAdmissionPolicy",
        ]
        .contains(&requirement.kubernetes_resource_kind)
            && requirement
                .policy_ref
                .starts_with("policy/autoscaling/fd001/")
            && requirement
                .expected_evidence_ref
                .starts_with("evidence/autoscaling/fd001-tenant-rbac/")
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
fn tenant_autoscaling_contract_rejects_missing_duplicate_doc_and_ref_drift() {
    let mut contract = fd001_tenant_autoscaling_contract().expect("contract builds");
    contract.requirements.truncate(3);
    assert_eq!(
        validate_fd001_tenant_autoscaling_contract(&contract),
        Err(Fd001TenantAutoscalingError::MissingRequirements)
    );

    let mut contract = fd001_tenant_autoscaling_contract().expect("contract builds");
    contract.requirements[1] = contract.requirements[0].clone();
    assert_eq!(
        validate_fd001_tenant_autoscaling_contract(&contract),
        Err(Fd001TenantAutoscalingError::DuplicateRequirement(
            "horizontal-pod-autoscaler-all-workloads".to_owned()
        ))
    );

    let mut contract = fd001_tenant_autoscaling_contract().expect("contract builds");
    contract.requirements[0].official_doc_url = "https://example.com/autoscaling";
    assert_eq!(
        validate_fd001_tenant_autoscaling_contract(&contract),
        Err(Fd001TenantAutoscalingError::InvalidOfficialDocUrl)
    );

    let mut contract = fd001_tenant_autoscaling_contract().expect("contract builds");
    contract.requirements[0].policy_ref = "deploy/autoscaling/fd001/hpa";
    assert_eq!(
        validate_fd001_tenant_autoscaling_contract(&contract),
        Err(Fd001TenantAutoscalingError::InvalidPolicyRef)
    );
}

#[test]
fn tenant_autoscaling_contract_rejects_missing_controls_and_overclaims() {
    let mut contract = fd001_tenant_autoscaling_contract().expect("contract builds");
    contract.horizontal_pod_autoscaler_required = false;
    assert_eq!(
        validate_fd001_tenant_autoscaling_contract(&contract),
        Err(Fd001TenantAutoscalingError::MissingRequiredControl(
            "horizontal_pod_autoscaler_required"
        ))
    );

    let mut contract = fd001_tenant_autoscaling_contract().expect("contract builds");
    contract.all_manifest_workloads_in_scope = false;
    assert_eq!(
        validate_fd001_tenant_autoscaling_contract(&contract),
        Err(Fd001TenantAutoscalingError::MissingRequiredControl(
            "all_manifest_workloads_in_scope"
        ))
    );

    let mut contract = fd001_tenant_autoscaling_contract().expect("contract builds");
    contract.requirements[0].applies_to_all_manifest_workloads = false;
    assert_eq!(
        validate_fd001_tenant_autoscaling_contract(&contract),
        Err(Fd001TenantAutoscalingError::MissingRequiredControl(
            "requirement_applies_to_all_manifest_workloads"
        ))
    );

    let mut contract = fd001_tenant_autoscaling_contract().expect("contract builds");
    contract.horizontal_pod_autoscaler_applied = true;
    assert_eq!(
        validate_fd001_tenant_autoscaling_contract(&contract),
        Err(Fd001TenantAutoscalingError::RuntimeAttachmentOverclaim)
    );
}
