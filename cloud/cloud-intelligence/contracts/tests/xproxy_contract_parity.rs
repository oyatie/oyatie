#![allow(clippy::expect_used, clippy::panic)]

const OPENAPI: &str = include_str!("../cloud-intelligence.openapi.yaml");
const ASYNCAPI: &str = include_str!("../cloud-intelligence.asyncapi.yaml");
const PROTO: &str = include_str!("../cloud-intelligence.proto");

fn assert_contract_has_path(contract: &str, path: &str) {
    assert!(
        contract.contains(&format!("  {path}:")),
        "contract missing path {path}"
    );
}

fn assert_contract_mentions(contract: &str, needle: &str) {
    assert!(contract.contains(needle), "contract missing {needle}");
}

fn assert_contract_omits_direct_transient_engines(contract: &str, label: &str) {
    let forbidden_terms = [
        ["Open", "Bao"].concat(),
        ["open", "bao"].concat(),
        ["Ce", "dar"].concat(),
        ["ce", "dar"].concat(),
        ["va", "ult"].concat(),
    ];
    for forbidden in forbidden_terms {
        assert!(
            !contract.contains(&forbidden),
            "{label} must target owned ports/adapters, not direct transient engine wording: {forbidden}"
        );
    }
}

#[test]
fn xproxy_openapi_declares_reference_protocol_and_admin_surfaces() {
    for path in [
        "/v1/messages",
        "/v1/messages/count_tokens",
        "/v1/complete",
        "/v1/chat/completions",
        "/v1/models",
        "/admin/v1/status",
        "/admin/v1/accounts",
        "/admin/v1/analytics",
        "/admin/v1/analytics/stream",
        "/admin/v1/resume",
    ] {
        assert_contract_has_path(OPENAPI, path);
    }

    for operation_id in [
        "createAnthropicMessage",
        "countAnthropicMessageTokens",
        "createLegacyCompletion",
        "getGatewayStatus",
        "listAccounts",
        "getAnalyticsSummary",
        "streamAnalytics",
        "resumeCircuitBreaker",
    ] {
        assert_contract_mentions(OPENAPI, operation_id);
    }

    assert_contract_mentions(OPENAPI, "owned-secret-provider-port");
    assert_contract_mentions(OPENAPI, "owned-policy-engine-port");
    assert_contract_omits_direct_transient_engines(OPENAPI, "OpenAPI");
}

#[test]
fn xproxy_asyncapi_declares_parity_drift_compatibility_and_circuit_events() {
    for channel in [
        "llm.usage.v1",
        "llm.audit.v1",
        "llm.parity.v1",
        "llm.drift.v1",
        "llm.compatibility.v1",
        "llm.circuit_breaker.v1",
    ] {
        assert_contract_mentions(ASYNCAPI, channel);
    }

    for message in [
        "UsageRecord",
        "AuditRecord",
        "CapabilityParitySnapshot",
        "DriftProbeResult",
        "CompatibilityCanaryResult",
        "CircuitBreakerEvent",
    ] {
        assert_contract_mentions(ASYNCAPI, message);
    }

    assert_contract_omits_direct_transient_engines(ASYNCAPI, "AsyncAPI");
}

#[test]
fn xproxy_proto_declares_admin_control_plane_methods_without_direct_engine_ownership() {
    for rpc in [
        "rpc PoolStatus",
        "rpc KeyRefresh",
        "rpc GatewayStatus",
        "rpc AccountStatus",
        "rpc AnalyticsSummary",
        "rpc ResumeCircuitBreaker",
    ] {
        assert_contract_mentions(PROTO, rpc);
    }

    for message in [
        "GatewayStatusRequest",
        "GatewayStatusResponse",
        "AccountStatusRequest",
        "AccountStatusResponse",
        "AnalyticsSummaryRequest",
        "AnalyticsSummaryResponse",
        "ResumeCircuitBreakerRequest",
        "ResumeCircuitBreakerResponse",
    ] {
        assert_contract_mentions(PROTO, message);
    }

    assert_contract_mentions(PROTO, "owned-secret-provider-port");
    assert_contract_mentions(PROTO, "owned-policy-engine-port");
    assert_contract_omits_direct_transient_engines(PROTO, "proto");
}
