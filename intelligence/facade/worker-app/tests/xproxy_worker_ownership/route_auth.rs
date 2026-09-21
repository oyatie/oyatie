use intelligence_worker_app::{
    BackendRegistry, CloudAuthRequirements, ConfigLayer, ConfigSource, CredentialRefreshPlan,
    ModelRouteSpec, OAuthLifecyclePlan, PoolActivation, ProviderBackendSpec, ProviderClass,
    RoutingAdvisorPurpose, default_routing_advisor_profiles, resolve_config_precedence,
};

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
fn xproxy_route_005_cheaper_model_advisors_are_routing_only_adapter_backed_and_redacted() {
    let advisors = default_routing_advisor_profiles();
    assert!(
        advisors
            .iter()
            .any(|advisor| advisor.model_hint == "chatgpt-spark")
    );
    assert!(
        advisors
            .iter()
            .any(|advisor| advisor.model_hint == "gemini-3.1-flash-lite")
    );
    assert!(
        advisors
            .iter()
            .any(|advisor| advisor.model_hint == "nemotron-3-ultra-550b-a55b")
    );

    for advisor in advisors {
        assert_eq!(advisor.purpose, RoutingAdvisorPurpose::RoutingDecisionOnly);
        assert!(advisor.adapter_backed);
        assert!(!advisor.may_execute_generation);
        assert!(!advisor.receives_raw_prompts_or_secrets);
        assert!(
            advisor.receives_redacted_route_metadata,
            "routing advisors should receive only redacted route metadata"
        );
    }
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
