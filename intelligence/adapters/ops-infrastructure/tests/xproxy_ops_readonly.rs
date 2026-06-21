#![allow(clippy::expect_used, clippy::panic)]

use intelligence_ops_infrastructure::{
    AdminClientSurface, CircuitBreakerState, DiagnosticSnapshot, ReadOnlyMcpToolset,
    ReadOnlyOpsSurface, UsageEvent,
};

#[test]
fn ops_surface_is_read_only_and_redacts_sensitive_values() {
    let surface = ReadOnlyOpsSurface::default_cloud_dashboard();
    assert!(surface.read_only);
    assert!(surface.mutation_routes.is_empty());
    assert!(surface.views.contains(&"accounts".to_string()));
    assert!(surface.views.contains(&"backends".to_string()));

    let snapshot = DiagnosticSnapshot::new("tenant-a", "secret-ref://tenant-a/openai/seat-a");
    let rendered = snapshot.render_redacted_json().unwrap();
    assert!(rendered.contains("tenant-a"));
    assert!(!rendered.contains("secret-ref://tenant-a/openai/seat-a"));
    assert!(rendered.contains("redacted"));
}

#[test]
fn xproxy_api_006_admin_client_surface_is_generated_contract_shape_not_binary_workflow() {
    let surface = AdminClientSurface::generated_contract_shape();
    assert!(
        surface
            .grpc_methods
            .contains(&"RefreshProviderPool".to_string())
    );
    assert!(
        surface
            .rest_routes
            .contains(&"/admin/v1/status".to_string())
    );
    assert!(!surface.includes_binary_entrypoint);
}

#[test]
fn xproxy_obs_002_usage_events_have_route_account_dimensions_and_redact_payloads() {
    let event = UsageEvent::llm_usage(
        "tenant-a",
        "route-openai",
        "seat-a",
        "gpt-4o",
        "raw prompt must not appear",
    );
    let rendered = serde_json::to_string(&event).unwrap();
    assert!(rendered.contains("llm.usage.v1"));
    assert!(rendered.contains("route-openai"));
    assert!(rendered.contains("seat-a"));
    assert!(!rendered.contains("raw prompt"));
}

#[test]
fn xproxy_obs_003_circuit_breaker_returns_retry_after_and_admin_resume_state() {
    let tripped = CircuitBreakerState::tripped("tenant-a", "budget_exceeded", 120);
    assert!(tripped.is_tripped);
    assert_eq!(tripped.retry_after_seconds, Some(120));

    let resumed = tripped.admin_resume("admin:ops");
    assert!(!resumed.is_tripped);
    assert_eq!(resumed.resumed_by.as_deref(), Some("admin:ops"));
}

#[test]
fn xproxy_obs_005_read_only_mcp_toolset_excludes_mutations() {
    let toolset = ReadOnlyMcpToolset::default_ops_tools();
    assert!(toolset.read_only);
    assert!(toolset.tools.contains(&"status".to_string()));
    assert!(toolset.tools.contains(&"accounts".to_string()));
    assert!(toolset.tools.contains(&"backends".to_string()));
    assert!(!toolset.tools.iter().any(|tool| tool.contains("resume")));
}
