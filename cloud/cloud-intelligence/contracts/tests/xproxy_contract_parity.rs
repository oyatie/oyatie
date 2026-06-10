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

    for field in [
        "policy_engine_ready",
        "default_data_plane_ready",
        "secret_provider_ready",
        "registered_pools",
        "boundaries",
        "secret-handles-redacted",
        "gemini",
    ] {
        assert_contract_mentions(OPENAPI, field);
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

    for field in [
        "GatewayBoundaryStatus",
        "default_data_plane_ready",
        "secret_provider_ready",
        "registered_pools",
        "boundaries",
        "redaction",
        "PROVIDER_CHANNEL_GEMINI",
    ] {
        assert_contract_mentions(PROTO, field);
    }

    assert_contract_mentions(PROTO, "owned-secret-provider-port");
    assert_contract_mentions(PROTO, "owned-policy-engine-port");
    assert_contract_omits_direct_transient_engines(PROTO, "proto");
}

mod transient_adapter_boundary;

#[test]
fn cloud_intelligence_safety_contracts_expose_guardrail_evidence_and_review_surfaces() {
    for path in [
        "/admin/v1/guardrails",
        "/admin/v1/guardrails/escalations",
        "/admin/v1/evidence/retention",
        "/admin/v1/redaction/profiles",
    ] {
        assert_contract_has_path(OPENAPI, path);
    }

    for operation_id in [
        "listGuardrailProfiles",
        "listSafetyEscalations",
        "listEvidenceRetentionProfiles",
        "listRedactionProfiles",
    ] {
        assert_contract_mentions(OPENAPI, operation_id);
    }

    for schema in [
        "GuardrailDetectionProfile",
        "ManualReviewEscalation",
        "EvidenceRetentionProfile",
        "InTransitRedactionProfile",
        "SafetySignalPolicy",
    ] {
        assert_contract_mentions(OPENAPI, schema);
    }
}

#[test]
fn cloud_intelligence_safety_asyncapi_declares_guardrail_review_and_evidence_events() {
    for channel in [
        "llm.guardrail.v1",
        "llm.safety_review.v1",
        "llm.evidence.v1",
    ] {
        assert_contract_mentions(ASYNCAPI, channel);
    }

    for message in [
        "GuardrailSignal",
        "SecondaryReviewRequest",
        "EvidenceRetentionEvent",
    ] {
        assert_contract_mentions(ASYNCAPI, message);
    }
}

#[test]
fn cloud_intelligence_safety_proto_declares_policy_evaluation_and_break_glass() {
    for rpc in [
        "rpc EvaluateSafety",
        "rpc SafetyEscalationStatus",
        "rpc RequestBreakGlassEvidenceAccess",
    ] {
        assert_contract_mentions(PROTO, rpc);
    }

    for message in [
        "SafetyEvaluationRequest",
        "SafetyEvaluationResponse",
        "SafetyEscalationStatusRequest",
        "SafetyEscalationStatusResponse",
        "BreakGlassEvidenceAccessRequest",
        "BreakGlassEvidenceAccessResponse",
    ] {
        assert_contract_mentions(PROTO, message);
    }

    assert_contract_omits_direct_transient_engines(PROTO, "proto");
}

#[test]
fn cloud_intelligence_agent_runtime_and_canary_contracts_are_first_class_status_surfaces() {
    for path in [
        "/admin/v1/agent-runtimes",
        "/admin/v1/agent-schedules",
        "/admin/v1/parity/canaries",
    ] {
        assert_contract_has_path(OPENAPI, path);
    }

    for operation_id in [
        "listAgentRuntimeProfiles",
        "listAgentSchedules",
        "listParityCanaries",
    ] {
        assert_contract_mentions(OPENAPI, operation_id);
    }

    for schema in [
        "AgentRuntimeProfile",
        "AgentSchedule",
        "AgentDelegationPolicy",
        "AgentWorkflowStatus",
        "ParityCanaryPlan",
        "ParityCanaryStatus",
    ] {
        assert_contract_mentions(OPENAPI, schema);
    }

    for rpc in [
        "rpc AgentRuntimeStatus",
        "rpc AgentScheduleStatus",
        "rpc ParityCanaryStatus",
    ] {
        assert_contract_mentions(PROTO, rpc);
    }

    for message in [
        "AgentRuntimeStatusRequest",
        "AgentRuntimeStatusResponse",
        "AgentScheduleStatusRequest",
        "AgentScheduleStatusResponse",
        "ParityCanaryStatusRequest",
        "ParityCanaryStatusResponse",
    ] {
        assert_contract_mentions(PROTO, message);
    }
}

#[test]
fn cloud_intelligence_agent_and_canary_asyncapi_events_are_redacted_status_only() {
    for channel in [
        "llm.agent_runtime.v1",
        "llm.agent_schedule.v1",
        "llm.parity_canary.v1",
    ] {
        assert_contract_mentions(ASYNCAPI, channel);
    }

    for message in [
        "AgentWorkflowStatusEvent",
        "AgentScheduleStatusEvent",
        "ParityCanaryStatusEvent",
    ] {
        assert_contract_mentions(ASYNCAPI, message);
    }

    for forbidden_positive_surface in [
        ["local", "panel"].join(" "),
        ["local", "panel"].join("-"),
        ["shell", "out", "to", "cli"].join(" "),
        ["shells", "out", "to", "cli"].join(" "),
        ["local", "tui", "workflow"].join(" "),
    ] {
        assert!(
            !OPENAPI
                .to_ascii_lowercase()
                .contains(forbidden_positive_surface.as_str()),
            "OpenAPI must not expose positive local control-plane surface: {forbidden_positive_surface}"
        );
        assert!(
            !ASYNCAPI
                .to_ascii_lowercase()
                .contains(forbidden_positive_surface.as_str()),
            "AsyncAPI must not expose positive local control-plane surface: {forbidden_positive_surface}"
        );
        assert!(
            !PROTO
                .to_ascii_lowercase()
                .contains(forbidden_positive_surface.as_str()),
            "proto must not expose positive local control-plane surface: {forbidden_positive_surface}"
        );
    }
}
