use iam_tenant_rbac_tenant_workload_runtime_evidence::{
    TenantRbacTenantWorkloadRuntimeEvidenceError, TenantWorkloadRuntimeEvidenceRequirementKind,
    tenant_rbac_tenant_workload_runtime_evidence_plan, tenant_workload_runtime_evidence_doc_urls,
    validate_tenant_rbac_tenant_workload_runtime_evidence_plan,
};

#[test]
fn tenant_workload_runtime_evidence_plan_validates_controls_and_nonclaims() {
    let plan = tenant_rbac_tenant_workload_runtime_evidence_plan().expect("plan builds");
    validate_tenant_rbac_tenant_workload_runtime_evidence_plan(&plan).expect("plan validates");

    assert_eq!(
        plan.plan_name,
        "fd001-tenant-workload-runtime-evidence-plan"
    );
    assert_eq!(plan.program_name, "fd-001-tenant-rbac-generic");
    assert_eq!(plan.substrate_name, "oyatie-cloud");
    assert_eq!(plan.tenant_namespace, "oyatie-fd001-tenant-rbac-dev");
    assert_eq!(plan.tenant_cell_id, "cell-us-east-001");
    assert_eq!(plan.residency_region, "us-east-1");
    assert_eq!(plan.otel_service_namespace, "fd001-tenant-rbac");
    assert_eq!(plan.tenant_claim, "tenant_id");
    assert_eq!(
        plan.tenant_workload_manifest_name,
        "fd001-tenant-rbac-tenant-workload-manifest"
    );
    assert_eq!(plan.manifest_workload_count, 4);
    assert_eq!(plan.requirements.len(), 14);
    assert!(plan.fd001_product_delivery_master_goal_preserved);
    assert!(plan.oyatie_cloud_substrate_proof_required);
    assert!(plan.official_docs_required);
    assert!(plan.tenant_namespace_runtime_evidence_required);
    assert!(plan.per_workload_runtime_evidence_required);
    assert!(plan.namespace_observation_required);
    assert!(plan.resource_quota_usage_required);
    assert!(plan.network_policy_default_deny_required);
    assert!(plan.service_account_boundary_required);
    assert!(plan.pod_security_context_required);
    assert!(plan.workload_scheduled_required);
    assert!(plan.resource_requests_limits_required);
    assert!(plan.readiness_probe_required);
    assert!(plan.liveness_probe_required);
    assert!(plan.gateway_route_acceptance_required);
    assert!(plan.tenant_claim_propagation_required);
    assert!(plan.otel_resource_identity_required);
    assert!(plan.rollout_recovery_required);
    assert!(plan.workload_audit_event_required);
    assert!(plan.review_only_contract);
    assert!(!plan.production_tenant_attached);
    assert!(!plan.kubernetes_runtime_attached);
    assert!(!plan.workload_runtime_deployed_attached);
    assert!(!plan.gateway_controller_attached);
    assert!(!plan.cloud_substrate_runtime_attached);
    assert!(!plan.runtime_audit_chain_emission_attached);
    assert!(!plan.production_workload_evidence_attached);
}

#[test]
fn tenant_workload_runtime_evidence_plan_covers_required_requirement_kinds_and_docs() {
    let plan = tenant_rbac_tenant_workload_runtime_evidence_plan().expect("plan builds");
    let kinds = plan
        .requirements
        .iter()
        .map(|requirement| requirement.requirement_kind)
        .collect::<std::collections::BTreeSet<_>>();

    for kind in [
        TenantWorkloadRuntimeEvidenceRequirementKind::NamespaceObserved,
        TenantWorkloadRuntimeEvidenceRequirementKind::ResourceQuotaUsageObserved,
        TenantWorkloadRuntimeEvidenceRequirementKind::NetworkPolicyDefaultDenyObserved,
        TenantWorkloadRuntimeEvidenceRequirementKind::ServiceAccountBoundaryObserved,
        TenantWorkloadRuntimeEvidenceRequirementKind::PodSecurityContextObserved,
        TenantWorkloadRuntimeEvidenceRequirementKind::WorkloadScheduled,
        TenantWorkloadRuntimeEvidenceRequirementKind::ResourceRequestsLimitsObserved,
        TenantWorkloadRuntimeEvidenceRequirementKind::ReadinessProbeObserved,
        TenantWorkloadRuntimeEvidenceRequirementKind::LivenessProbeObserved,
        TenantWorkloadRuntimeEvidenceRequirementKind::GatewayRouteAccepted,
        TenantWorkloadRuntimeEvidenceRequirementKind::TenantClaimPropagated,
        TenantWorkloadRuntimeEvidenceRequirementKind::OTelResourceIdentityObserved,
        TenantWorkloadRuntimeEvidenceRequirementKind::RolloutRecoveryObserved,
        TenantWorkloadRuntimeEvidenceRequirementKind::WorkloadAuditEventRecorded,
    ] {
        assert!(kinds.contains(&kind), "missing {kind:?}");
    }

    let docs = tenant_workload_runtime_evidence_doc_urls(&plan);
    assert!(docs.contains(
        &"https://kubernetes.io/docs/concepts/overview/working-with-objects/namespaces/"
    ));
    assert!(docs.contains(&"https://kubernetes.io/docs/concepts/policy/resource-quotas/"));
    assert!(
        docs.contains(&"https://kubernetes.io/docs/concepts/services-networking/network-policies/")
    );
    assert!(docs.contains(&"https://kubernetes.io/docs/concepts/security/service-accounts/"));
    assert!(docs.contains(&"https://kubernetes.io/docs/concepts/security/pod-security-standards/"));
    assert!(
        docs.contains(&"https://kubernetes.io/docs/concepts/workloads/controllers/deployment/")
    );
    assert!(docs.contains(
        &"https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/"
    ));
    assert!(docs.contains(&"https://gateway-api.sigs.k8s.io/docs/introduction/"));
    assert!(docs.contains(&"https://opentelemetry.io/docs/specs/semconv/resource/service/"));
}

#[test]
fn tenant_workload_runtime_evidence_plan_preserves_ref_boundaries_and_source_contract() {
    let plan = tenant_rbac_tenant_workload_runtime_evidence_plan().expect("plan builds");

    assert!(plan.requirements.iter().all(|requirement| {
        requirement
            .expected_evidence_ref
            .starts_with("evidence/tenant-workload-runtime/fd001-tenant-rbac/")
            && requirement
                .source_manifest_ref
                .starts_with("crates/tenant-rbac-tenant-workload-manifest/")
            && requirement.workload_scope == "all-fd001-workloads"
            && requirement.tenant_namespace == "oyatie-fd001-tenant-rbac-dev"
            && requirement.tenant_cell_id == "cell-us-east-001"
            && requirement.residency_region == "us-east-1"
            && requirement.tenant_claim == "tenant_id"
            && !requirement.runtime_evidence_attached
    }));
    assert!(plan.requirements.iter().any(|requirement| {
        requirement.requirement_kind
            == TenantWorkloadRuntimeEvidenceRequirementKind::NetworkPolicyDefaultDenyObserved
            && requirement.requires_network_policy_default_deny
            && requirement.requires_namespace_observation
    }));
    assert!(plan.requirements.iter().any(|requirement| {
        requirement.requirement_kind
            == TenantWorkloadRuntimeEvidenceRequirementKind::GatewayRouteAccepted
            && requirement.requires_gateway_route_acceptance
            && requirement.requires_network_policy_default_deny
            && requirement.requires_readiness_probe
    }));
    assert!(plan.requirements.iter().any(|requirement| {
        requirement.requirement_kind
            == TenantWorkloadRuntimeEvidenceRequirementKind::WorkloadAuditEventRecorded
            && requirement.requires_tenant_claim_propagation
            && requirement.requires_otel_resource_identity
            && requirement.requires_workload_audit_event
    }));
}

#[test]
fn tenant_workload_runtime_evidence_plan_rejects_missing_duplicate_and_doc_drift() {
    let mut plan = tenant_rbac_tenant_workload_runtime_evidence_plan().expect("plan builds");
    plan.requirements.truncate(4);
    assert_eq!(
        validate_tenant_rbac_tenant_workload_runtime_evidence_plan(&plan),
        Err(TenantRbacTenantWorkloadRuntimeEvidenceError::MissingRequirements)
    );

    let mut plan = tenant_rbac_tenant_workload_runtime_evidence_plan().expect("plan builds");
    plan.requirements[1].requirement_id = plan.requirements[0].requirement_id;
    assert_eq!(
        validate_tenant_rbac_tenant_workload_runtime_evidence_plan(&plan),
        Err(
            TenantRbacTenantWorkloadRuntimeEvidenceError::DuplicateRequirement(
                "namespace-observed".to_owned()
            )
        )
    );

    let mut plan = tenant_rbac_tenant_workload_runtime_evidence_plan().expect("plan builds");
    plan.requirements[0].official_doc_url = "https://example.com/kubernetes";
    assert_eq!(
        validate_tenant_rbac_tenant_workload_runtime_evidence_plan(&plan),
        Err(TenantRbacTenantWorkloadRuntimeEvidenceError::InvalidOfficialDocUrl)
    );
}

#[test]
fn tenant_workload_runtime_evidence_plan_rejects_unsafe_refs_missing_controls_and_overclaims() {
    let mut plan = tenant_rbac_tenant_workload_runtime_evidence_plan().expect("plan builds");
    plan.requirements[0].expected_evidence_ref =
        "evidence/tenant-workload-runtime/fd001-tenant-rbac/client_secret";
    assert_eq!(
        validate_tenant_rbac_tenant_workload_runtime_evidence_plan(&plan),
        Err(TenantRbacTenantWorkloadRuntimeEvidenceError::InvalidExpectedEvidenceRef)
    );

    let mut plan = tenant_rbac_tenant_workload_runtime_evidence_plan().expect("plan builds");
    plan.resource_quota_usage_required = false;
    assert_eq!(
        validate_tenant_rbac_tenant_workload_runtime_evidence_plan(&plan),
        Err(
            TenantRbacTenantWorkloadRuntimeEvidenceError::MissingRequiredControl(
                "resource_quota_usage_required"
            )
        )
    );

    let mut plan = tenant_rbac_tenant_workload_runtime_evidence_plan().expect("plan builds");
    plan.workload_runtime_deployed_attached = true;
    assert_eq!(
        validate_tenant_rbac_tenant_workload_runtime_evidence_plan(&plan),
        Err(TenantRbacTenantWorkloadRuntimeEvidenceError::RuntimeAttachmentOverclaim)
    );
}
