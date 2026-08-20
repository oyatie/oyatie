#![allow(clippy::expect_used, clippy::panic)]

use intelligence_tool_compat_kernel::{
    AgentProfilePackage, ClientCompatibilityMatrix, ExternalClientProfile, ToolMode,
    classify_tool_request, default_tool_registry, sanitize_orchestration_tags,
};

#[test]
fn tool_registry_has_schema_verified_compatibility_entries() {
    let registry = default_tool_registry();
    assert!(
        registry.entries.len() >= 66,
        "expected 66-entry compatibility floor"
    );
    assert!(registry.entries.iter().all(|entry| entry.schema_verified));
    assert!(
        registry
            .entries
            .iter()
            .all(|entry| entry.capability_id.starts_with("XPROXY-COMPAT-"))
    );
}

#[test]
fn tool_classifier_detects_text_tool_and_policy_modes() {
    let text_tool = serde_json::json!({"tools":[{"name":"str_replace_editor","input_schema":{"type":"object"}}]});
    let decision = classify_tool_request(&default_tool_registry(), &text_tool, ToolMode::Hybrid)
        .expect("tool request should classify");
    assert!(decision.text_tool_detected);
    assert_eq!(decision.mode, ToolMode::Hybrid);
    assert!(decision.telemetry_safe_summary.contains("tools=1"));
    assert!(
        !decision
            .telemetry_safe_summary
            .contains("str_replace_editor")
    );
}

#[test]
fn xproxy_compat_001_matrix_records_supported_inferred_and_blocked_clients() {
    let matrix = ClientCompatibilityMatrix::default_profiles();
    assert!(matrix.is_supported("codex-compatible-client"));
    assert!(matrix.is_supported("gemini-compatible-client"));
    assert!(matrix.is_inferred("continue-dev"));
    assert!(matrix.is_blocked("unsafe-public-tunnel-default"));
    assert!(
        matrix
            .canary_names()
            .contains(&"openai-chat-pass-through".to_string())
    );
    assert!(
        matrix
            .canary_names()
            .contains(&"gemini-generate-content".to_string())
    );
}

#[test]
fn xproxy_compat_004_sanitizer_strips_orchestration_tags_without_raw_logging() {
    let sanitized = sanitize_orchestration_tags(
        "<orchestration>hidden route note</orchestration>visible answer",
    );
    assert_eq!(sanitized.visible_text, "visible answer");
    assert!(sanitized.telemetry_safe_summary.contains("stripped_tags=1"));
    assert!(
        !sanitized
            .telemetry_safe_summary
            .contains("hidden route note")
    );
}

#[test]
fn xproxy_compat_005_006_client_profiles_are_cloud_safe_and_outside_gateway_hot_path() {
    let profile = ExternalClientProfile::cloud_safe("cursor", Some("anthropic:"));
    assert!(profile.requires_https);
    assert!(!profile.recommends_public_tunnel_default);
    assert_eq!(
        profile.provider_prefix_workaround.as_deref(),
        Some("anthropic:")
    );

    let package = AgentProfilePackage::status_only("claude-code-subagent");
    assert!(!package.gateway_hot_path);
    assert!(!package.installs_local_hook);
    assert_eq!(package.surface, "cloud-dashboard-profile");
}
