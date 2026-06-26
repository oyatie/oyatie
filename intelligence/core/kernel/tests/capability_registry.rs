//! Integration coverage for the per-model capability registry.
//!
//! Exercises the public surface from outside the crate and proves the
//! router -> registry seam: a model string is resolved by [`ModelRouter`] to a
//! canonical upstream name + routing capabilities, which then feed the
//! [`CapabilityRegistry`] pre-flight. Both layers must agree on one identifier.
#![allow(clippy::expect_used, clippy::panic)]

use intelligence_kernel::capabilities::{
    CallRequirements, CapabilityRegistry, CapabilityViolation, ModelCapabilities,
};
use intelligence_kernel::model_routing::{
    ModelRouter, ProtocolShape, RoutePolicy, RouteRequest,
};

/// Resolve a user-facing model string the same way the dispatch path does, then
/// pre-flight it against the capability registry.
fn route_then_preflight(
    model: &str,
    req: &CallRequirements,
) -> Result<ModelCapabilities, Vec<CapabilityViolation>> {
    let decision = ModelRouter::default()
        .route(RouteRequest {
            protocol: ProtocolShape::AnthropicMessages,
            model: model.to_string(),
            route_policy: RoutePolicy::default(),
            tenant_default_backend: None,
        })
        .expect("model should route");
    CapabilityRegistry::default()
        .preflight(&decision.upstream_model, &decision.capabilities, req)
        .map_err(|rejection| rejection.violations)
}

#[test]
fn registry_table_matrix_matches_canonical_models() {
    // (user model string, expects vision, expects reasoning) — table-driven.
    let cases = [
        ("opus", true, true),
        ("opus47", true, true),
        ("sonnet", true, true),
        ("haiku", true, false),
        ("gpt-4o", true, false),
        // o3-mini / o1-mini are TEXT-ONLY reasoning models: vision == FALSE
        // (fail-closed; the coarse o3/o1 family rows must not grant vision).
        ("o3-mini", false, true),
        ("o1-mini", false, true),
        // o4-mini is multimodal: vision == true.
        ("o4-mini", true, true),
        ("gemini-2.5-pro", true, true),
        ("claude:opus", true, true),
    ];
    for (model, vision, reasoning) in cases {
        let caps = route_then_preflight(model, &CallRequirements::default())
            .unwrap_or_else(|v| panic!("{model} should preflight clean: {v:?}"));
        assert_eq!(caps.supports_vision, vision, "vision {model}");
        assert_eq!(caps.supports_reasoning, reasoning, "reasoning {model}");
    }
}

#[test]
fn vision_request_to_text_only_model_is_pre_flight_rejected() {
    let req = CallRequirements {
        needs_vision: true,
        ..Default::default()
    };
    let violations = route_then_preflight("text-embedding-3-small", &req)
        .expect_err("vision to an embedding model must reject before dispatch");
    assert!(violations.contains(&CapabilityViolation::VisionUnsupported));
}

#[test]
fn reasoning_request_to_haiku_is_pre_flight_rejected() {
    let req = CallRequirements {
        needs_reasoning: true,
        ..Default::default()
    };
    let violations = route_then_preflight("haiku", &req)
        .expect_err("reasoning to haiku must reject");
    assert_eq!(violations, vec![CapabilityViolation::ReasoningUnsupported]);
}

#[test]
fn one_million_context_tag_widens_input_window_through_routing() {
    let req = CallRequirements {
        estimated_input_tokens: 600_000,
        ..Default::default()
    };
    // Plain opus = 200k window -> reject.
    let violations = route_then_preflight("opus", &req).expect_err("over 200k window");
    assert!(violations
        .iter()
        .any(|v| matches!(v, CapabilityViolation::InputTokensExceeded { .. })));

    // opus[1m] -> router emits OneMillionContext -> registry widens to 1M -> pass.
    route_then_preflight("opus[1m]", &req).expect("1m-tagged opus accepts 600k input");
}

#[test]
fn fully_supported_call_returns_effective_capabilities() {
    let req = CallRequirements {
        needs_function_calling: true,
        needs_vision: true,
        needs_prompt_caching: true,
        needs_reasoning: true,
        needs_streaming: true,
        estimated_input_tokens: 50_000,
        // Opus 4.x caps output at 32k — stay within it.
        requested_max_output_tokens: 30_000,
    };
    let caps = route_then_preflight("opus", &req).expect("supported call passes");
    assert!(caps.supports_function_calling);
    // Opus 4.x effective output ceiling is 32k (NOT Sonnet's 64k).
    assert_eq!(caps.max_output_tokens, 32_000);
}

/// Premortem P6 at the router→registry seam: the hidden `gpt-5.4-mini`
/// auxiliary model (Codex background compaction/titling) must route AND
/// pre-flight clean, never falling through to `UnknownModel`. A vision payload
/// to it must still be rejected (text-only, fail-closed).
#[test]
fn auxiliary_gpt_5_4_mini_routes_and_preflights_text_only() {
    // A background compaction-shaped call passes pre-flight.
    let bg = CallRequirements {
        needs_function_calling: true,
        needs_streaming: true,
        estimated_input_tokens: 100_000,
        requested_max_output_tokens: 1_024,
        ..Default::default()
    };
    let caps = route_then_preflight("gpt-5.4-mini", &bg)
        .unwrap_or_else(|v| panic!("gpt-5.4-mini must preflight clean (P6): {v:?}"));
    assert!(!caps.supports_vision, "gpt-5.4-mini is text-only");

    // A vision payload to the same model is rejected, not waved through.
    let vision = CallRequirements {
        needs_vision: true,
        ..Default::default()
    };
    let violations = route_then_preflight("gpt-5.4-mini", &vision)
        .expect_err("vision to text-only auxiliary model must reject");
    assert!(violations.contains(&CapabilityViolation::VisionUnsupported));
}

#[test]
fn unknown_upstream_model_fails_closed_with_serializable_rejection() {
    // tenant-private models route via tenant default but have no capability row.
    let rejection = CapabilityRegistry::default()
        .preflight("tenant-private-model", &[], &CallRequirements::default())
        .expect_err("unknown model must fail closed");
    assert_eq!(rejection.violations, vec![CapabilityViolation::UnknownModel]);

    // Rejection is serializable for transport back through the REST adapter.
    let json = serde_json::to_string(&rejection).expect("serialize rejection");
    assert!(json.contains("UnknownModel"));
    assert!(json.contains("tenant-private-model"));
}
