use iam_tenant_rbac_listener_runtime_evidence::{
    ListenerRuntimeEvidenceRequirementKind, TenantRbacListenerRuntimeEvidenceError,
    listener_runtime_evidence_doc_urls, tenant_rbac_listener_runtime_evidence_plan,
    validate_tenant_rbac_listener_runtime_evidence_plan,
};

#[test]
fn listener_runtime_evidence_plan_validates_controls_and_nonclaims() {
    let plan = tenant_rbac_listener_runtime_evidence_plan().expect("plan builds");
    validate_tenant_rbac_listener_runtime_evidence_plan(&plan).expect("plan validates");

    assert_eq!(plan.plan_name, "tenant-rbac-listener-runtime-evidence-plan");
    assert_eq!(plan.service_name, "tenant-rbac");
    assert_eq!(plan.substrate_name, "oyatie-cloud");
    assert_eq!(plan.tenant_namespace, "oyatie-fd001-tenant-rbac-dev");
    assert_eq!(
        plan.listener_gateway_plan_name,
        "tenant-rbac-listener-gateway-plan"
    );
    assert_eq!(plan.listener_gateway_route_count, 19);
    assert_eq!(plan.listener_gateway_service_type, "ClusterIP");
    assert_eq!(plan.requirements.len(), 12);
    assert!(plan.fd001_product_delivery_master_goal_preserved);
    assert!(plan.oyatie_cloud_substrate_proof_required);
    assert!(plan.official_docs_required);
    assert!(plan.cluster_ip_service_observation_required);
    assert!(plan.gateway_route_acceptance_required);
    assert!(plan.tls_certificate_binding_required);
    assert!(plan.readiness_probe_success_required);
    assert!(plan.liveness_probe_success_required);
    assert!(plan.synthetic_health_check_required);
    assert!(plan.route_authz_enforcement_required);
    assert!(plan.default_deny_network_policy_required);
    assert!(plan.endpoint_slice_ready_required);
    assert!(plan.graceful_shutdown_drain_required);
    assert!(plan.access_log_trace_correlation_required);
    assert!(plan.listener_deployment_audit_event_required);
    assert!(plan.review_only_contract);
    assert!(!plan.deployed_listener_attached);
    assert!(!plan.bound_socket_attached);
    assert!(!plan.gateway_controller_attached);
    assert!(!plan.load_balancer_attached);
    assert!(!plan.tls_certificate_attached);
    assert!(!plan.runtime_auth_middleware_attached);
    assert!(!plan.network_policy_applied_attached);
    assert!(!plan.readiness_probe_runtime_attached);
    assert!(!plan.liveness_probe_runtime_attached);
    assert!(!plan.production_listener_evidence_attached);
    assert!(!plan.runtime_audit_chain_emission_attached);
}

#[test]
fn listener_runtime_evidence_plan_covers_required_requirement_kinds_and_docs() {
    let plan = tenant_rbac_listener_runtime_evidence_plan().expect("plan builds");
    let kinds = plan
        .requirements
        .iter()
        .map(|requirement| requirement.requirement_kind)
        .collect::<std::collections::BTreeSet<_>>();

    for kind in [
        ListenerRuntimeEvidenceRequirementKind::ClusterIpServiceObserved,
        ListenerRuntimeEvidenceRequirementKind::GatewayHttpRouteAccepted,
        ListenerRuntimeEvidenceRequirementKind::TlsCertificateBound,
        ListenerRuntimeEvidenceRequirementKind::ReadinessProbeSucceeded,
        ListenerRuntimeEvidenceRequirementKind::LivenessProbeSucceeded,
        ListenerRuntimeEvidenceRequirementKind::SyntheticHealthCheckSucceeded,
        ListenerRuntimeEvidenceRequirementKind::RouteAuthzEnforced,
        ListenerRuntimeEvidenceRequirementKind::NetworkPolicyIngressDenied,
        ListenerRuntimeEvidenceRequirementKind::EndpointSliceReadyObserved,
        ListenerRuntimeEvidenceRequirementKind::GracefulShutdownObserved,
        ListenerRuntimeEvidenceRequirementKind::AccessLogTraceCorrelated,
        ListenerRuntimeEvidenceRequirementKind::ListenerDeploymentAuditRecorded,
    ] {
        assert!(kinds.contains(&kind), "missing {kind:?}");
    }

    let docs = listener_runtime_evidence_doc_urls(&plan);
    assert!(docs.contains(&"https://kubernetes.io/docs/concepts/services-networking/service/"));
    assert!(docs.contains(&"https://gateway-api.sigs.k8s.io/concepts/api-overview/"));
    assert!(docs.contains(&"https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/"));
    assert!(
        docs.contains(&"https://kubernetes.io/docs/concepts/services-networking/network-policies/")
    );
    assert!(
        docs.contains(&"https://kubernetes.io/docs/concepts/services-networking/endpoint-slices/")
    );
    assert!(docs.contains(&"https://opentelemetry.io/docs/specs/semconv/http/http-spans/"));
}

#[test]
fn listener_runtime_evidence_plan_preserves_ref_and_route_boundaries() {
    let plan = tenant_rbac_listener_runtime_evidence_plan().expect("plan builds");

    assert!(plan.requirements.iter().all(|requirement| {
        requirement
            .expected_evidence_ref
            .starts_with("evidence/listener-runtime/tenant-rbac/")
            && requirement
                .source_plan_ref
                .starts_with("crates/tenant-rbac-listener-gateway/")
            && requirement.tenant_namespace == "oyatie-fd001-tenant-rbac-dev"
            && requirement.required_route_count == 19
            && !requirement.runtime_evidence_attached
    }));
    assert!(plan.requirements.iter().any(|requirement| {
        requirement.requirement_kind
            == ListenerRuntimeEvidenceRequirementKind::GatewayHttpRouteAccepted
            && requirement.requires_gateway_observation
    }));
    assert!(plan.requirements.iter().any(|requirement| {
        requirement.requirement_kind == ListenerRuntimeEvidenceRequirementKind::RouteAuthzEnforced
            && requirement.requires_security_enforcement
    }));
    assert!(plan.requirements.iter().any(|requirement| {
        requirement.requirement_kind
            == ListenerRuntimeEvidenceRequirementKind::ReadinessProbeSucceeded
            && requirement.requires_runtime_probe
    }));
}

#[test]
fn listener_runtime_evidence_plan_rejects_missing_duplicate_and_doc_drift() {
    let mut plan = tenant_rbac_listener_runtime_evidence_plan().expect("plan builds");
    plan.requirements.truncate(2);
    assert_eq!(
        validate_tenant_rbac_listener_runtime_evidence_plan(&plan),
        Err(TenantRbacListenerRuntimeEvidenceError::MissingRequirements)
    );

    let mut plan = tenant_rbac_listener_runtime_evidence_plan().expect("plan builds");
    plan.requirements[1].requirement_id = plan.requirements[0].requirement_id;
    assert_eq!(
        validate_tenant_rbac_listener_runtime_evidence_plan(&plan),
        Err(
            TenantRbacListenerRuntimeEvidenceError::DuplicateRequirement(
                "clusterip-service-observed".to_owned()
            )
        )
    );

    let mut plan = tenant_rbac_listener_runtime_evidence_plan().expect("plan builds");
    plan.requirements[0].official_doc_url = "https://example.com/service";
    assert_eq!(
        validate_tenant_rbac_listener_runtime_evidence_plan(&plan),
        Err(TenantRbacListenerRuntimeEvidenceError::InvalidOfficialDocUrl)
    );
}

#[test]
fn listener_runtime_evidence_plan_rejects_unsafe_refs_missing_controls_and_overclaims() {
    let mut plan = tenant_rbac_listener_runtime_evidence_plan().expect("plan builds");
    plan.requirements[0].expected_evidence_ref =
        "evidence/listener-runtime/tenant-rbac/secret-token";
    assert_eq!(
        validate_tenant_rbac_listener_runtime_evidence_plan(&plan),
        Err(TenantRbacListenerRuntimeEvidenceError::InvalidExpectedEvidenceRef)
    );

    let mut plan = tenant_rbac_listener_runtime_evidence_plan().expect("plan builds");
    plan.default_deny_network_policy_required = false;
    assert_eq!(
        validate_tenant_rbac_listener_runtime_evidence_plan(&plan),
        Err(
            TenantRbacListenerRuntimeEvidenceError::MissingRequiredControl(
                "default_deny_network_policy_required"
            )
        )
    );

    let mut plan = tenant_rbac_listener_runtime_evidence_plan().expect("plan builds");
    plan.production_listener_evidence_attached = true;
    assert_eq!(
        validate_tenant_rbac_listener_runtime_evidence_plan(&plan),
        Err(TenantRbacListenerRuntimeEvidenceError::RuntimeAttachmentOverclaim)
    );
}
