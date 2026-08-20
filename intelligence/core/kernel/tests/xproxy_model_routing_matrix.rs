#![allow(clippy::expect_used, clippy::panic)]

use intelligence_kernel::model_routing::{
    BackendClass, ModelCapability, ModelRouter, ProtocolShape, RoutePolicy, RouteRequest,
    TranslationMode,
};

fn route(
    protocol: ProtocolShape,
    model: &str,
) -> intelligence_kernel::model_routing::RoutingDecision {
    ModelRouter::default()
        .route(RouteRequest {
            protocol,
            model: model.to_string(),
            route_policy: RoutePolicy::default(),
            tenant_default_backend: None,
        })
        .expect("route should resolve")
}

#[test]
fn xproxy_route_001_prefix_overrides_protocol_and_route_policy() {
    let decision = ModelRouter::default()
        .route(RouteRequest {
            protocol: ProtocolShape::AnthropicMessages,
            model: "openai:gpt-4o".to_string(),
            route_policy: RoutePolicy {
                forced_backend: Some(BackendClass::AnthropicSubscription),
            },
            tenant_default_backend: Some(BackendClass::AnthropicSubscription),
        })
        .expect("explicit provider prefix wins");

    assert_eq!(decision.backend, BackendClass::OpenAiCompatible);
    assert_eq!(decision.provider_prefix.as_deref(), Some("openai"));
    assert_eq!(decision.upstream_model, "gpt-4o");
    assert_eq!(
        decision.translation_mode,
        TranslationMode::AnthropicToOpenAi
    );
}

#[test]
fn xproxy_route_001_policy_overrides_classifier_but_not_prefix() {
    let decision = ModelRouter::default()
        .route(RouteRequest {
            protocol: ProtocolShape::OpenAiChatCompletions,
            model: "gpt-4o".to_string(),
            route_policy: RoutePolicy {
                forced_backend: Some(BackendClass::AnthropicSubscription),
            },
            tenant_default_backend: None,
        })
        .expect("route policy should resolve");

    assert_eq!(decision.backend, BackendClass::AnthropicSubscription);
    assert_eq!(
        decision.translation_mode,
        TranslationMode::OpenAiToAnthropic
    );
}

#[test]
fn xproxy_route_001_model_classifier_cross_routes_protocols() {
    let anthropic_to_openai = route(ProtocolShape::AnthropicMessages, "gpt-4o");
    assert_eq!(anthropic_to_openai.backend, BackendClass::OpenAiCompatible);
    assert_eq!(
        anthropic_to_openai.translation_mode,
        TranslationMode::AnthropicToOpenAi
    );

    let openai_to_anthropic = route(ProtocolShape::OpenAiChatCompletions, "claude-opus-4-5");
    assert_eq!(
        openai_to_anthropic.backend,
        BackendClass::AnthropicSubscription
    );
    assert_eq!(
        openai_to_anthropic.translation_mode,
        TranslationMode::OpenAiToAnthropic
    );

    let passthrough = route(ProtocolShape::OpenAiChatCompletions, "o3-mini");
    assert_eq!(passthrough.backend, BackendClass::OpenAiCompatible);
    assert_eq!(passthrough.translation_mode, TranslationMode::PassThrough);

    let openai_to_gemini = route(ProtocolShape::OpenAiChatCompletions, "gemini-2.5-flash");
    assert_eq!(openai_to_gemini.backend, BackendClass::GeminiNative);
    assert_eq!(
        openai_to_gemini.translation_mode,
        TranslationMode::OpenAiToGemini
    );

    let anthropic_to_gemini = route(ProtocolShape::AnthropicMessages, "gemini:gemini-2.5-pro");
    assert_eq!(anthropic_to_gemini.backend, BackendClass::GeminiNative);
    assert_eq!(
        anthropic_to_gemini.translation_mode,
        TranslationMode::AnthropicToGemini
    );
}

#[test]
fn xproxy_route_002_provider_prefix_registry_maps_current_prefixes() {
    for prefix in ["openai", "codex"] {
        let model = format!("{prefix}:gpt-4o");
        let decision = route(ProtocolShape::AnthropicMessages, &model);
        assert_eq!(decision.backend, BackendClass::OpenAiCompatible, "{prefix}");
        assert_eq!(decision.provider_prefix.as_deref(), Some(prefix));
        assert_eq!(decision.upstream_model, "gpt-4o");
    }

    for prefix in ["claude", "anthropic"] {
        let model = format!("{prefix}:sonnet[1m]");
        let decision = route(ProtocolShape::OpenAiChatCompletions, &model);
        assert_eq!(
            decision.backend,
            BackendClass::AnthropicSubscription,
            "{prefix}"
        );
        assert_eq!(decision.provider_prefix.as_deref(), Some(prefix));
        assert_eq!(decision.upstream_model, "claude-sonnet-4-5");
        assert!(
            decision
                .capabilities
                .contains(&ModelCapability::OneMillionContext)
        );
    }

    for prefix in ["gemini", "google"] {
        let model = format!("{prefix}:gemini-2.5-flash");
        let decision = route(ProtocolShape::OpenAiChatCompletions, &model);
        assert_eq!(decision.backend, BackendClass::GeminiNative, "{prefix}");
        assert_eq!(decision.provider_prefix.as_deref(), Some(prefix));
        assert_eq!(decision.upstream_model, "gemini-2.5-flash");
        assert_eq!(decision.translation_mode, TranslationMode::OpenAiToGemini);
    }
}

#[test]
fn xproxy_route_003_claude_aliases_and_context_tags_are_normalized() {
    for (alias, canonical) in [
        ("fable", "claude-sonnet-4-5"),
        ("fable1m", "claude-sonnet-4-5"),
        ("opus", "claude-opus-4-5"),
        ("opus47", "claude-opus-4-7"),
        ("opus46", "claude-opus-4-6"),
        ("sonnet", "claude-sonnet-4-5"),
        ("haiku", "claude-haiku-3-5"),
    ] {
        let decision = route(ProtocolShape::AnthropicMessages, alias);
        assert_eq!(
            decision.backend,
            BackendClass::AnthropicSubscription,
            "{alias}"
        );
        assert_eq!(decision.upstream_model, canonical, "{alias}");
    }

    let tagged = route(ProtocolShape::AnthropicMessages, "opus[1m]");
    assert_eq!(tagged.upstream_model, "claude-opus-4-5");
    assert!(
        tagged
            .capabilities
            .contains(&ModelCapability::OneMillionContext)
    );
}

#[test]
fn xproxy_route_001_tenant_default_is_last_resort_for_unknown_models() {
    let decision = ModelRouter::default()
        .route(RouteRequest {
            protocol: ProtocolShape::AnthropicMessages,
            model: "tenant-private-model".to_string(),
            route_policy: RoutePolicy::default(),
            tenant_default_backend: Some(BackendClass::OpenAiCompatible),
        })
        .expect("tenant default resolves unknown model");

    assert_eq!(decision.backend, BackendClass::OpenAiCompatible);
    assert_eq!(decision.upstream_model, "tenant-private-model");
    assert_eq!(
        decision.translation_mode,
        TranslationMode::AnthropicToOpenAi
    );
}
