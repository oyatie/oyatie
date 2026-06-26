//! Hermetic integration tests for the redactor guardrail and the
//! non-bypassable EventSink redaction stage. No real network or runtime
//! dependencies — secret-bearing fixtures only.

use std::sync::{Arc, Mutex};

use intelligence_guardrail_adapter::{redact, RedactingEventSink, RedactorGuardrail};
use intelligence_kernel::guardrail::{
    GuardClass, GuardContent, GuardContext, GuardDecision, Guardrail, GuardrailChain,
};
use intelligence_kernel::{
    AgentId, EventSink, EventStatus, LlmGatewayEvent, Provider, SeatId, TenantId,
};

fn ctx() -> GuardContext {
    GuardContext::new(
        TenantId::new("tenant-a").unwrap(),
        AgentId::new("agent-a").unwrap(),
        Provider::Anthropic,
        "req-1",
    )
}

#[tokio::test]
async fn pre_call_redacts_outbound_prompt() {
    let g = RedactorGuardrail::new();
    let content = GuardContent::new("ssh in with Bearer sk-ant-api03-AAAAAAAAAAAAAAAAAAAA please");
    let outcome = g.pre_call(&content, &ctx()).await;
    assert_eq!(outcome.decision(), GuardDecision::Redacted);
    assert!(!outcome.content().as_str().contains("sk-ant-api03"));
    assert!(outcome.findings().iter().any(|f| f.class == GuardClass::Credential));
}

#[tokio::test]
async fn post_call_redacts_inbound_response() {
    let g = RedactorGuardrail::new();
    let content = GuardContent::new("here is your file /Users/victim/.aws/credentials");
    let outcome = g.post_call(&content, &ctx()).await;
    assert_eq!(outcome.decision(), GuardDecision::Redacted);
    assert!(!outcome.content().as_str().contains("/Users/victim"));
}

#[tokio::test]
async fn clean_content_passes_as_allow() {
    let g = RedactorGuardrail::new();
    let content = GuardContent::new("summarize this paragraph about otters");
    let outcome = g.pre_call(&content, &ctx()).await;
    assert!(outcome.is_clean());
}

#[tokio::test]
async fn redactor_composes_in_a_chain() {
    let chain = GuardrailChain::new().with(Arc::new(RedactorGuardrail::new()));
    let outcome = chain
        .pre_call(
            &GuardContent::new("token ghp_AAAAAAAAAAAAAAAAAAAAAAAA done"),
            &ctx(),
        )
        .await;
    assert_eq!(outcome.decision(), GuardDecision::Redacted);
    assert!(!outcome.content().as_str().contains("ghp_AAAA"));
}

// ---------------------------------------------------------------------------
// Non-bypassable EventSink redaction
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct CaptureSink {
    events: Arc<Mutex<Vec<LlmGatewayEvent>>>,
}

impl EventSink for CaptureSink {
    fn emit(&self, event: LlmGatewayEvent) {
        self.events.lock().unwrap().push(event);
    }
}

fn event_with(request_id: &str, model: &str) -> LlmGatewayEvent {
    LlmGatewayEvent {
        request_id: request_id.to_string(),
        tenant_id: TenantId::new("tenant-a").unwrap(),
        agent_id: AgentId::new("agent-a").unwrap(),
        seat_id: SeatId::new("seat-1").unwrap(),
        provider: Provider::Anthropic,
        model: model.to_string(),
        prompt_tokens: 1,
        completion_tokens: 1,
        ms_latency: 1,
        status: EventStatus::Ok,
        timestamp_unix_ms: 1,
    }
}

#[test]
fn redacting_sink_scrubs_secret_before_inner_sink_sees_it() {
    let captured = Arc::new(Mutex::new(Vec::new()));
    let inner = CaptureSink {
        events: captured.clone(),
    };
    let sink = RedactingEventSink::new(inner);

    // A secret smuggled into a free-text event field.
    sink.emit(event_with(
        "req-Bearer eyJaaa.bbb.ccc",
        "model-sk-ant-api03-AAAAAAAAAAAAAAAAAAAA",
    ));

    let events = captured.lock().unwrap();
    assert_eq!(events.len(), 1);
    assert!(
        !events[0].request_id.contains("eyJaaa"),
        "secret reached the inner sink via request_id: {}",
        events[0].request_id
    );
    assert!(
        !events[0].model.contains("sk-ant-api03"),
        "secret reached the inner sink via model: {}",
        events[0].model
    );
}

#[test]
fn redacting_sink_passes_clean_events_through_unchanged() {
    let captured = Arc::new(Mutex::new(Vec::new()));
    let sink = RedactingEventSink::new(CaptureSink {
        events: captured.clone(),
    });
    sink.emit(event_with("req-abc123", "claude-opus-4-5"));
    let events = captured.lock().unwrap();
    assert_eq!(events[0].request_id, "req-abc123");
    assert_eq!(events[0].model, "claude-opus-4-5");
}

#[test]
fn redact_is_idempotent() {
    let once = redact("Bearer eyJa.b.c and /Users/x/y").0;
    let twice = redact(&once).0;
    assert_eq!(once, twice, "redaction should be a fixed point");
}
