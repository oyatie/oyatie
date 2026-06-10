#![allow(clippy::expect_used, clippy::panic)]

use oya_cloud_intelligence_workers::{
    BackendRegistry, CloudAuthRequirements, ConfigLayer, ConfigSource, CredentialRefreshPlan,
    DriftParityPlan, ModelRouteSpec, OAuthLifecyclePlan, PoolActivation, ProviderBackendSpec,
    ProviderClass, WorkerKind, default_worker_ownership, resolve_config_precedence,
};

#[test]
fn worker_ownership_map_keeps_hot_path_and_control_plane_separate() {
    let map = default_worker_ownership();
    assert!(
        map.iter()
            .any(|worker| worker.name == "cloud-intelligence-gateway"
                && worker.kind == WorkerKind::GatewayDeployment)
    );
    assert!(map.iter().any(|worker| worker.name == "route-controller"));
    assert!(
        map.iter()
            .any(|worker| worker.name == "model-inventory-worker")
    );
    assert!(
        map.iter()
            .any(|worker| worker.name == "drift-parity-worker")
    );
    assert!(map.iter().any(|worker| worker.kind == WorkerKind::CronJob));
    assert!(
        map.iter()
            .all(|worker| !worker.hot_path || worker.name == "cloud-intelligence-gateway")
    );
    assert!(
        map.iter()
            .all(|worker| !worker.writes_raw_prompts_or_secrets)
    );
}

fn manifest() -> String {
    std::fs::read_to_string("cloud/cloud-intelligence/k8s/cloud-intelligence.yaml")
        .expect("read cloud-intelligence manifest")
}

fn assert_manifest_contains(manifest: &str, needle: &str) {
    assert!(manifest.contains(needle), "manifest missing {needle}");
}

#[test]
fn k8s_manifest_declares_crds_workers_canaries_and_hardening() {
    let manifest = manifest();
    for crd in [
        "providerbackends.cloud-intelligence.oya.io",
        "modelroutes.cloud-intelligence.oya.io",
        "modelaliassets.cloud-intelligence.oya.io",
        "promptprofiles.cloud-intelligence.oya.io",
        "thinkingpolicies.cloud-intelligence.oya.io",
        "subscriptionseats.cloud-intelligence.oya.io",
        "wireprofiles.cloud-intelligence.oya.io",
        "toolcompatibilityprofiles.cloud-intelligence.oya.io",
        "gatewaycircuitbreakers.cloud-intelligence.oya.io",
        "capabilityparitybaselines.cloud-intelligence.oya.io",
    ] {
        assert_manifest_contains(&manifest, crd);
    }

    for deployment in [
        "name: cloud-intelligence-gateway",
        "name: route-controller",
        "name: model-inventory-worker",
        "name: credential-refresh-worker",
        "name: analytics-metering-worker",
        "name: circuit-breaker-worker",
        "name: ops-api",
    ] {
        assert_manifest_contains(&manifest, deployment);
    }

    assert_manifest_contains(&manifest, "kind: CronJob");
    assert_manifest_contains(&manifest, "name: drift-parity-worker");
    assert_manifest_contains(&manifest, "kind: Job");
    assert_manifest_contains(&manifest, "name: compatibility-worker");
    assert_manifest_contains(&manifest, "kind: ServiceAccount");
    assert_manifest_contains(&manifest, "kind: Role");
    assert_manifest_contains(&manifest, "kind: RoleBinding");
    assert_manifest_contains(&manifest, "kind: NetworkPolicy");
    assert_manifest_contains(&manifest, "kind: PodDisruptionBudget");
    assert_manifest_contains(&manifest, "readOnlyRootFilesystem: true");
    assert_manifest_contains(&manifest, "allowPrivilegeEscalation: false");
    assert_manifest_contains(&manifest, "runAsNonRoot: true");
    assert_manifest_contains(&manifest, "path: /livez");
    assert_manifest_contains(&manifest, "path: /readyz");
    assert!(!manifest.contains("value: sk-"));
}

#[test]
fn xproxy_route_004_005_006_model_and_backend_management_are_declarative_resources() {
    let route = ModelRouteSpec::policy_authorized_override(
        "tenant-a",
        "chat",
        "compat-gateway",
        "gpt-4o-mini",
        "medium",
        8192,
    )
    .expect("policy-authorized route override");
    assert_eq!(route.normalized_effort.as_deref(), Some("medium"));
    assert_eq!(route.max_output_tokens, Some(8192));
    assert!(route.policy_authorized);

    let rejected =
        ModelRouteSpec::unauthorized_override("tenant-a", "chat", "compat-gateway", "gpt-4o-mini");
    assert!(
        rejected.is_err(),
        "route overrides require policy authorization"
    );

    let backends = BackendRegistry::from_specs(vec![
        ProviderBackendSpec::new_openai_compatible(
            "compat-primary",
            "https://provider-a.example/v1",
            "secret-ref://tenant-a/provider/primary",
            90,
        )
        .unwrap(),
        ProviderBackendSpec::new_openai_compatible(
            "compat-secondary",
            "https://provider-b.example/v1",
            "secret-ref://tenant-a/provider/secondary",
            10,
        )
        .unwrap(),
    ])
    .expect("multiple named provider backends");

    assert_eq!(backends.len(), 2);
    assert_eq!(backends.weighted_fallback_order()[0], "compat-primary");
    assert!(
        ProviderBackendSpec::new_openai_compatible(
            "../unsafe",
            "https://provider.example/v1",
            "secret-ref://tenant-a/provider/unsafe",
            1,
        )
        .is_err()
    );
}

#[test]
fn xproxy_auth_001_002_006_007_008_lifecycle_auth_and_config_are_cloud_native() {
    let lifecycle = OAuthLifecyclePlan::manual_headless_enrollment(
        "tenant-a",
        ProviderClass::AnthropicSubscription,
        "secret-ref://tenant-a/oauth/seat-a",
    )
    .expect("worker-safe lifecycle plan");
    assert!(lifecycle.worker_safe);
    assert!(!lifecycle.uses_browser_automation);
    assert!(lifecycle.refresh_token_handle.starts_with("secret-ref://"));

    assert_eq!(
        PoolActivation::from_seat_count(1),
        PoolActivation::SingleSeat
    );
    assert_eq!(
        PoolActivation::from_seat_count(2),
        PoolActivation::MultiSeatActive
    );

    let refresh = CredentialRefreshPlan::singleflight(
        "tenant-a",
        ProviderClass::AnthropicSubscription,
        "secret-ref://tenant-a/oauth/seat-a",
    )
    .unwrap();
    assert_eq!(
        refresh.singleflight_group_key,
        "tenant-a:anthropic-subscription"
    );
    assert!(!refresh.stores_plaintext_secret);

    let auth = CloudAuthRequirements::non_loopback_default();
    assert!(auth.requires_tenant_authn);
    assert!(auth.requires_policy_engine_decision);
    assert!(auth.requires_mtls_or_api_key_at_edge);
    assert!(auth.cors_requires_policy_review);

    let resolved = resolve_config_precedence([
        ConfigLayer::new(ConfigSource::ServiceDefault, "model", "sonnet"),
        ConfigLayer::new(ConfigSource::TenantDefault, "model", "opus"),
        ConfigLayer::new(ConfigSource::ModelRoute, "model", "openai:gpt-4o"),
    ])
    .expect("deterministic cloud config precedence");
    assert_eq!(
        resolved.get("model").map(String::as_str),
        Some("openai:gpt-4o")
    );
    assert!(
        ConfigLayer::new(ConfigSource::ModelRoute, "provider_key", "sk-raw-value")
            .validate_no_raw_secret()
            .is_err()
    );
}

#[test]
fn xproxy_drift_001_002_workers_emit_pinned_parity_and_canary_plans() {
    let plan = DriftParityPlan::for_pinned_baseline(
        "external-proxy-reference",
        "30fed94b362f5106cc7a4feaf37019fc0ccc007f",
    );
    assert_eq!(plan.kind, "CapabilityParityBaseline");
    assert!(plan.audit_event_required);
    assert!(plan.opens_pr_or_task_on_delta);
    assert!(plan.probes.contains(&"wire-profile-drift".to_string()));

    let canaries = plan.compatibility_canaries();
    assert!(canaries.contains(&"route-matrix".to_string()));
    assert!(canaries.contains(&"streaming-fixtures".to_string()));
    assert!(canaries.contains(&"pool-failover".to_string()));
    assert!(canaries.contains(&"security-redaction".to_string()));
}
