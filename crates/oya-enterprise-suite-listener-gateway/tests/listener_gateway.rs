use oya_enterprise_suite_listener_gateway::{
    EnterpriseSuiteListenerGatewayError, enterprise_suite_listener_gateway_plan,
    render_enterprise_suite_listener_gateway_review_manifest,
    validate_enterprise_suite_listener_gateway_plan,
};

#[test]
fn listener_gateway_plan_covers_runtime_routes_auth_policy_and_probe_contract() {
    let plan = enterprise_suite_listener_gateway_plan();
    validate_enterprise_suite_listener_gateway_plan(&plan)
        .expect("listener gateway plan validates");

    assert_eq!(plan.plan_name, "enterprise-suite-listener-gateway-plan");
    assert_eq!(plan.service_name, "enterprise-suite");
    assert_eq!(plan.kubernetes_service_type, "ClusterIP");
    assert_eq!(plan.gateway_api_version, "gateway.networking.k8s.io/v1");
    assert_eq!(plan.gateway_route_kind, "HTTPRoute");
    assert_eq!(plan.gateway_listener_port, 443);
    assert_eq!(plan.backend_port, 8080);
    assert_eq!(plan.service_port, 8080);
    assert_eq!(
        plan.application_health_route_path,
        "/enterprise-suite/v1/healthz"
    );
    assert_eq!(plan.platform_readiness_probe_path, "/health");
    assert_eq!(plan.platform_liveness_probe_path, "/health");
    assert_eq!(plan.route_count, 19);
    assert_eq!(plan.auth_policy_route_count, 19);
    assert!(plan.gateway_api_required);
    assert!(plan.ingress_tls_required);
    assert!(plan.network_policy_required);
    assert!(plan.authz_required);
    assert!(plan.deny_by_default_required);
    assert!(plan.rendered_kubernetes_manifest_review_only);
    assert!(!plan.direct_public_node_port_allowed);
    assert!(!plan.direct_public_load_balancer_allowed);
    assert!(!plan.deployed_listener_attached);
    assert!(!plan.gateway_controller_attached);
    assert!(!plan.load_balancer_attached);
    assert!(!plan.tls_certificate_attached);
    assert!(!plan.runtime_auth_middleware_attached);
    assert!(!plan.cloud_deployment_evidence_attached);
    assert!(!plan.production_slo_evidence_attached);
    assert!(!plan.runtime_audit_chain_emission_attached);

    let payroll_trial_close = plan
        .routes
        .iter()
        .find(|route| route.method == "POST" && route.path == "/payroll/v1/trial-closes")
        .expect("payroll trial-close route is present");
    assert_eq!(
        payroll_trial_close.required_scope,
        "payroll:trial-close:write"
    );
    assert!(payroll_trial_close.sensitive_data);
    assert!(payroll_trial_close.mfa_required);
    assert_eq!(payroll_trial_close.backend_service_name, "enterprise-suite");
}

#[test]
fn listener_gateway_plan_renders_review_only_service_and_httproute_manifest() {
    let plan = enterprise_suite_listener_gateway_plan();
    let manifest = render_enterprise_suite_listener_gateway_review_manifest(&plan)
        .expect("review-only manifest renders");

    assert!(manifest.contains("# review_only: true"));
    assert!(manifest.contains("kind: Service"));
    assert!(manifest.contains("type: ClusterIP"));
    assert!(manifest.contains("apiVersion: gateway.networking.k8s.io/v1"));
    assert!(manifest.contains("kind: HTTPRoute"));
    assert!(manifest.contains("sectionName: https"));
    assert!(manifest.contains("name: enterprise-route-00"));
    assert!(manifest.contains("method: POST"));
    assert!(manifest.contains("value: /enterprise-suite/v1/policy-admissions"));
    assert!(manifest.contains("request: 30000ms"));
    assert!(!manifest.contains("type: NodePort"));
    assert!(!manifest.contains("type: LoadBalancer"));
}

#[test]
fn listener_gateway_plan_rejects_route_auth_mismatch_or_duplicate_routes() {
    let mut plan = enterprise_suite_listener_gateway_plan();
    plan.routes[0].required_scope = "enterprise-suite:wrong-scope";
    assert!(matches!(
        validate_enterprise_suite_listener_gateway_plan(&plan),
        Err(EnterpriseSuiteListenerGatewayError::MissingAuthPolicyCoverage(_))
    ));

    let mut plan = enterprise_suite_listener_gateway_plan();
    let duplicate = plan.routes[0].clone();
    plan.routes.push(duplicate);
    plan.route_count = plan.routes.len();
    assert!(matches!(
        validate_enterprise_suite_listener_gateway_plan(&plan),
        Err(EnterpriseSuiteListenerGatewayError::MissingRouteCoverage(_))
            | Err(EnterpriseSuiteListenerGatewayError::DuplicateRoute(_))
    ));
}

#[test]
fn listener_gateway_plan_rejects_public_ingress_or_runtime_overclaims() {
    let mut plan = enterprise_suite_listener_gateway_plan();
    plan.direct_public_node_port_allowed = true;
    assert_eq!(
        validate_enterprise_suite_listener_gateway_plan(&plan),
        Err(EnterpriseSuiteListenerGatewayError::PublicIngressOverclaim)
    );

    let mut plan = enterprise_suite_listener_gateway_plan();
    plan.gateway_controller_attached = true;
    assert_eq!(
        validate_enterprise_suite_listener_gateway_plan(&plan),
        Err(EnterpriseSuiteListenerGatewayError::RuntimeAttachmentOverclaim)
    );

    let mut plan = enterprise_suite_listener_gateway_plan();
    plan.authz_required = false;
    assert_eq!(
        validate_enterprise_suite_listener_gateway_plan(&plan),
        Err(EnterpriseSuiteListenerGatewayError::SecurityControlMissing)
    );
}

#[test]
fn listener_gateway_plan_validates_ports_hosts_timeouts_and_probe_paths() {
    let mut plan = enterprise_suite_listener_gateway_plan();
    plan.gateway_listener_port = 80;
    assert_eq!(
        validate_enterprise_suite_listener_gateway_plan(&plan),
        Err(EnterpriseSuiteListenerGatewayError::InvalidPort)
    );

    let mut plan = enterprise_suite_listener_gateway_plan();
    plan.external_hostname = "192.0.2.10";
    assert_eq!(
        validate_enterprise_suite_listener_gateway_plan(&plan),
        Err(EnterpriseSuiteListenerGatewayError::InvalidHostname)
    );

    let mut plan = enterprise_suite_listener_gateway_plan();
    plan.backend_request_timeout_ms = plan.request_timeout_ms + 1;
    assert_eq!(
        validate_enterprise_suite_listener_gateway_plan(&plan),
        Err(EnterpriseSuiteListenerGatewayError::InvalidTimeout)
    );

    let mut plan = enterprise_suite_listener_gateway_plan();
    plan.platform_readiness_probe_path = "health";
    assert_eq!(
        validate_enterprise_suite_listener_gateway_plan(&plan),
        Err(EnterpriseSuiteListenerGatewayError::InvalidProbePath)
    );
}
