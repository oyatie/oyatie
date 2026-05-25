use oya_enterprise_suite_cloud_deployment_evidence::{
    CloudDeploymentEvidenceRequirementKind, EnterpriseCloudDeploymentEvidenceError,
    cloud_deployment_evidence_doc_urls, enterprise_cloud_deployment_evidence_plan,
    validate_enterprise_cloud_deployment_evidence_plan,
};

#[test]
fn cloud_deployment_evidence_plan_validates_controls_and_nonclaims() {
    let plan = enterprise_cloud_deployment_evidence_plan().expect("plan builds");
    validate_enterprise_cloud_deployment_evidence_plan(&plan).expect("plan validates");

    assert_eq!(
        plan.plan_name,
        "enterprise-suite-cloud-deployment-evidence-plan"
    );
    assert_eq!(plan.service_name, "enterprise-suite");
    assert_eq!(plan.substrate_name, "oyatie-cloud");
    assert_eq!(plan.tenant_namespace, "oyatie-fd001-enterprise-dev");
    assert_eq!(plan.requirements.len(), 14);
    assert!(plan.fd001_product_delivery_master_goal_preserved);
    assert!(plan.oyatie_cloud_substrate_proof_required);
    assert!(plan.official_docs_required);
    assert!(plan.argocd_sync_evidence_required);
    assert!(plan.argocd_health_evidence_required);
    assert!(plan.git_revision_pin_required);
    assert!(plan.cosign_verification_required);
    assert!(plan.namespace_observation_required);
    assert!(plan.quota_observation_required);
    assert!(plan.network_policy_observation_required);
    assert!(plan.service_account_observation_required);
    assert!(plan.deployment_available_required);
    assert!(plan.readiness_probe_required);
    assert!(plan.gateway_route_acceptance_required);
    assert!(plan.otel_resource_identity_required);
    assert!(plan.deployment_audit_event_required);
    assert!(plan.rollback_plan_required);
    assert!(plan.review_only_contract);
    assert!(!plan.argocd_controller_attached);
    assert!(!plan.kubernetes_cluster_attached);
    assert!(!plan.namespace_created_attached);
    assert!(!plan.quota_applied_attached);
    assert!(!plan.network_policy_applied_attached);
    assert!(!plan.gateway_route_attached);
    assert!(!plan.workload_runtime_deployed_attached);
    assert!(!plan.runtime_otel_export_attached);
    assert!(!plan.runtime_audit_chain_emission_attached);
    assert!(!plan.production_cloud_deployment_evidence_attached);
}

#[test]
fn cloud_deployment_evidence_plan_covers_required_requirement_kinds_and_docs() {
    let plan = enterprise_cloud_deployment_evidence_plan().expect("plan builds");
    let kinds = plan
        .requirements
        .iter()
        .map(|requirement| requirement.requirement_kind)
        .collect::<std::collections::BTreeSet<_>>();

    for kind in [
        CloudDeploymentEvidenceRequirementKind::ArgoCdApplicationSynced,
        CloudDeploymentEvidenceRequirementKind::ArgoCdApplicationHealthy,
        CloudDeploymentEvidenceRequirementKind::GitRevisionPinned,
        CloudDeploymentEvidenceRequirementKind::CosignImageVerified,
        CloudDeploymentEvidenceRequirementKind::NamespaceObserved,
        CloudDeploymentEvidenceRequirementKind::ResourceQuotaObserved,
        CloudDeploymentEvidenceRequirementKind::NetworkPolicyObserved,
        CloudDeploymentEvidenceRequirementKind::ServiceAccountObserved,
        CloudDeploymentEvidenceRequirementKind::DeploymentAvailable,
        CloudDeploymentEvidenceRequirementKind::ReadinessProbeGreen,
        CloudDeploymentEvidenceRequirementKind::GatewayHttpRouteAccepted,
        CloudDeploymentEvidenceRequirementKind::OtelResourceIdentityObserved,
        CloudDeploymentEvidenceRequirementKind::DeploymentAuditEventRecorded,
        CloudDeploymentEvidenceRequirementKind::RollbackPlanAttached,
    ] {
        assert!(kinds.contains(&kind), "missing {kind:?}");
    }

    let docs = cloud_deployment_evidence_doc_urls(&plan);
    assert!(docs.contains(&"https://argo-cd.readthedocs.io/en/stable/"));
    assert!(docs.contains(&"https://docs.sigstore.dev/cosign/verifying/verify/"));
    assert!(
        docs.contains(&"https://kubernetes.io/docs/reference/kubernetes-api/apps/deployment-v1/")
    );
    assert!(docs.contains(&"https://gateway-api.sigs.k8s.io/docs/introduction/"));
    assert!(docs.contains(&"https://opentelemetry.io/docs/specs/semconv/resource/service/"));
}

#[test]
fn cloud_deployment_evidence_plan_preserves_evidence_ref_boundaries() {
    let plan = enterprise_cloud_deployment_evidence_plan().expect("plan builds");

    assert!(plan.requirements.iter().all(|requirement| {
        requirement
            .expected_evidence_ref
            .starts_with("evidence/cloud-deployment/enterprise-suite/")
            && requirement
                .source_manifest_ref
                .starts_with("crates/oya-enterprise-suite-cloud-deployment-manifest/")
            && requirement.tenant_namespace == "oyatie-fd001-enterprise-dev"
            && requirement.requires_tenant_namespace
            && !requirement.runtime_evidence_attached
    }));
    assert!(plan.requirements.iter().any(|requirement| {
        requirement.requirement_kind == CloudDeploymentEvidenceRequirementKind::GitRevisionPinned
            && requirement.requires_digest_pinned_revision
    }));
    assert!(plan.requirements.iter().any(|requirement| {
        requirement.requirement_kind == CloudDeploymentEvidenceRequirementKind::CosignImageVerified
            && requirement.requires_security_verification
    }));
}

#[test]
fn cloud_deployment_evidence_plan_rejects_missing_duplicate_and_doc_drift() {
    let mut plan = enterprise_cloud_deployment_evidence_plan().expect("plan builds");
    plan.requirements.truncate(2);
    assert_eq!(
        validate_enterprise_cloud_deployment_evidence_plan(&plan),
        Err(EnterpriseCloudDeploymentEvidenceError::MissingRequirements)
    );

    let mut plan = enterprise_cloud_deployment_evidence_plan().expect("plan builds");
    plan.requirements[1].requirement_id = plan.requirements[0].requirement_id;
    assert_eq!(
        validate_enterprise_cloud_deployment_evidence_plan(&plan),
        Err(
            EnterpriseCloudDeploymentEvidenceError::DuplicateRequirement(
                "argocd-application-synced".to_owned()
            )
        )
    );

    let mut plan = enterprise_cloud_deployment_evidence_plan().expect("plan builds");
    plan.requirements[0].official_doc_url = "https://example.com/argocd";
    assert_eq!(
        validate_enterprise_cloud_deployment_evidence_plan(&plan),
        Err(EnterpriseCloudDeploymentEvidenceError::InvalidOfficialDocUrl)
    );
}

#[test]
fn cloud_deployment_evidence_plan_rejects_unsafe_refs_missing_controls_and_overclaims() {
    let mut plan = enterprise_cloud_deployment_evidence_plan().expect("plan builds");
    plan.requirements[0].expected_evidence_ref =
        "evidence/cloud-deployment/enterprise-suite/secret-token";
    assert_eq!(
        validate_enterprise_cloud_deployment_evidence_plan(&plan),
        Err(EnterpriseCloudDeploymentEvidenceError::InvalidExpectedEvidenceRef)
    );

    let mut plan = enterprise_cloud_deployment_evidence_plan().expect("plan builds");
    plan.gateway_route_acceptance_required = false;
    assert_eq!(
        validate_enterprise_cloud_deployment_evidence_plan(&plan),
        Err(
            EnterpriseCloudDeploymentEvidenceError::MissingRequiredControl(
                "gateway_route_acceptance_required"
            )
        )
    );

    let mut plan = enterprise_cloud_deployment_evidence_plan().expect("plan builds");
    plan.fd001_product_delivery_master_goal_preserved = false;
    assert_eq!(
        validate_enterprise_cloud_deployment_evidence_plan(&plan),
        Err(
            EnterpriseCloudDeploymentEvidenceError::MissingRequiredControl(
                "fd001_product_delivery_master_goal_preserved"
            )
        )
    );

    let mut plan = enterprise_cloud_deployment_evidence_plan().expect("plan builds");
    plan.production_cloud_deployment_evidence_attached = true;
    assert_eq!(
        validate_enterprise_cloud_deployment_evidence_plan(&plan),
        Err(EnterpriseCloudDeploymentEvidenceError::RuntimeAttachmentOverclaim)
    );
}
