#![allow(clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;

use intelligence_wire_kernel::{
    DriftProbePlan, PacingPolicy, PromptCachePolicy, PromptProfile, SessionAffinityPolicy,
    ShimSupersession, StreamLifecyclePolicy, ThinkingPolicy, TransportFingerprintPolicy,
    WireCaptureBaseline, WireProfile, apply_prompt_profile, apply_thinking_policy,
    filter_wire_headers,
};

#[test]
fn wire_profile_strips_provider_control_headers_except_allowlisted_values() {
    let mut headers = BTreeMap::new();
    headers.insert("authorization".to_string(), "Bearer caller".to_string());
    headers.insert("connection".to_string(), "x-drop-me".to_string());
    headers.insert("x-drop-me".to_string(), "remove".to_string());
    headers.insert("openai-organization".to_string(), "org-safe".to_string());
    headers.insert("x-openai-beta".to_string(), "remove".to_string());

    let filtered = filter_wire_headers(&WireProfile::openai_compatible_default(), &headers);

    assert_eq!(
        filtered.get("openai-organization").map(String::as_str),
        Some("org-safe")
    );
    assert!(!filtered.contains_key("authorization"));
    assert!(!filtered.contains_key("x-drop-me"));
    assert!(!filtered.contains_key("x-openai-beta"));
}

#[test]
fn prompt_profile_uses_named_resources_not_cluster_file_paths() {
    let payload = serde_json::json!({"messages":[{"role":"user","content":"hi"}]});
    let profile = PromptProfile::named_resource("concise-default", "Be concise").unwrap();
    let rewritten = apply_prompt_profile(&profile, payload).unwrap();
    assert_eq!(rewritten["system"], "Be concise");
    assert!(PromptProfile::cluster_file_path("/tmp/prompt.txt", "body").is_err());
}

#[test]
fn thinking_policy_defaults_to_provider_compatible_shape_unless_explicitly_passthrough() {
    let payload = serde_json::json!({"thinking":{"type":"client","trace":"private"},"messages":[]});
    let defaulted = apply_thinking_policy(
        &ThinkingPolicy::provider_compatible_default(),
        payload.clone(),
    );
    assert!(defaulted.get("thinking").is_none());

    let passthrough =
        apply_thinking_policy(&ThinkingPolicy::explicit_client_passthrough(), payload);
    assert_eq!(passthrough["thinking"]["type"], "client");
}

#[test]
fn shim_supersession_is_recorded_as_cloud_gateway_not_child_process_patch() {
    let status = ShimSupersession::cloud_gateway_supersedes_local_patch();
    assert_eq!(status.capability_id, "XPROXY-COMPAT-006");
    assert_eq!(status.status, "superseded");
    assert!(
        !status
            .implementation_target
            .to_ascii_lowercase()
            .contains("child-process")
    );
    assert!(
        !status
            .implementation_target
            .to_ascii_lowercase()
            .contains("tui")
    );
}

#[test]
fn xproxy_wire_002_capture_baseline_is_signed_and_prompt_redacted() {
    let capture = WireCaptureBaseline::signed_provider_capture(
        "anthropic-messages",
        "sha256:abcdef",
        "prompt text must not appear",
    )
    .expect("signed capture baseline");
    assert_eq!(capture.profile_kind, "WireProfile");
    assert_eq!(capture.signature_ref, "sha256:abcdef");
    assert!(!capture.redacted_summary.contains("prompt text"));

    let probe = DriftProbePlan::from_capture(&capture);
    assert!(probe.worker_owned);
    assert!(probe.audit_event_required);
}

#[test]
fn xproxy_wire_004_005_transport_and_pacing_are_explicit_policy_decisions() {
    assert!(TransportFingerprintPolicy::default().adapter_isolated);
    assert!(!TransportFingerprintPolicy::default().strict_fingerprint_replay);
    assert!(
        TransportFingerprintPolicy::approved_adapter_isolated("provider-adapter")
            .strict_fingerprint_replay
    );

    let pacing = PacingPolicy::default_off();
    assert!(!pacing.enabled);
    assert!(PacingPolicy::compliance_approved(15, 5).enabled);
}

#[test]
fn xproxy_wire_006_007_stream_and_session_lifecycle_are_route_policies() {
    let no_drain = StreamLifecyclePolicy::default_no_drain();
    assert!(!no_drain.drain_to_eof_on_disconnect);
    assert!(!no_drain.retry_after_first_byte_allowed);

    let session = SessionAffinityPolicy::sticky_with_rotation(300, 3600, 30).unwrap();
    assert_eq!(session.idle_ttl_seconds, 300);
    assert_eq!(session.max_age_seconds, 3600);
    assert_eq!(session.rotation_jitter_seconds, 30);
}

#[test]
fn xproxy_wire_008_prompt_cache_and_beta_headers_are_allowlisted() {
    let policy = PromptCachePolicy::provider_allowlist(["anthropic-beta", "openai-organization"]);
    assert!(policy.allows_header("anthropic-beta"));
    assert!(policy.allows_header("openai-organization"));
    assert!(!policy.allows_header("x-unreviewed-beta"));
    assert!(policy.requires_provider_adapter);
}
