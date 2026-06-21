//! D6 — event-emission shape contract.
//!
//! Verifies the LlmGatewayEvent shape stays stable + the EventSink trait is
//! callable. Sink-fanout (ClickHouse + Valkey Stream) lives in adapter crates;
//! the kernel only owns the shape.
//!
//! Stage-4 RED: trivially passes for shape construction (we are not yet
//! invoking the sink from inside the kernel). The fanout integration test
//! lands in the REST-adapter Stage-5 PR. We keep the shape test here so that
//! any future change to the wire-format is caught by a unit test rather than
//! by a deployed consumer breaking.
use std::cell::RefCell;

use intelligence_kernel::{
    AgentId, EventSink, EventStatus, LlmGatewayEvent, Provider, SeatId, TenantId,
};

struct RecordingSink {
    events: RefCell<Vec<LlmGatewayEvent>>,
}

impl RecordingSink {
    fn new() -> Self {
        Self {
            events: RefCell::new(Vec::new()),
        }
    }
}

impl EventSink for RecordingSink {
    fn emit(&self, event: LlmGatewayEvent) {
        self.events.borrow_mut().push(event);
    }
}

#[test]
fn event_shape_includes_all_required_fields() {
    let event = LlmGatewayEvent {
        request_id: "req-abc-123".to_string(),
        tenant_id: TenantId::new("t-acme").unwrap(),
        agent_id: AgentId::new("agent-1").unwrap(),
        seat_id: SeatId::new("seat-a").unwrap(),
        provider: Provider::Anthropic,
        model: "claude-opus-4-7".to_string(),
        prompt_tokens: 1234,
        completion_tokens: 567,
        ms_latency: 891,
        status: EventStatus::Ok,
        timestamp_unix_ms: 1_716_900_000_000,
    };

    assert_eq!(event.request_id, "req-abc-123");
    assert_eq!(event.tenant_id.as_str(), "t-acme");
    assert_eq!(event.agent_id.as_str(), "agent-1");
    assert_eq!(event.seat_id.as_str(), "seat-a");
    assert_eq!(event.provider, Provider::Anthropic);
    assert_eq!(event.model, "claude-opus-4-7");
    assert_eq!(event.prompt_tokens, 1234);
    assert_eq!(event.completion_tokens, 567);
    assert_eq!(event.ms_latency, 891);
    assert_eq!(event.status, EventStatus::Ok);
    assert_eq!(event.timestamp_unix_ms, 1_716_900_000_000);
}

#[test]
fn event_sink_records_emitted_events_in_order() {
    let sink = RecordingSink::new();
    for i in 0..5 {
        sink.emit(LlmGatewayEvent {
            request_id: format!("req-{i}"),
            tenant_id: TenantId::new("t-acme").unwrap(),
            agent_id: AgentId::new("agent-1").unwrap(),
            seat_id: SeatId::new("seat-a").unwrap(),
            provider: Provider::Anthropic,
            model: "claude-opus-4-7".to_string(),
            prompt_tokens: i,
            completion_tokens: i * 2,
            ms_latency: 100 + i,
            status: EventStatus::Ok,
            timestamp_unix_ms: 1_716_900_000_000 + i,
        });
    }
    let recorded = sink.events.borrow();
    assert_eq!(recorded.len(), 5);
    for (i, ev) in recorded.iter().enumerate() {
        assert_eq!(ev.request_id, format!("req-{i}"));
        assert_eq!(ev.prompt_tokens, i as u64);
    }
}

#[test]
fn event_status_distinguishes_pool_exhausted_from_forbidden() {
    // ClickHouse/Valkey analytics queries rely on EventStatus being
    // unambiguous — a forbidden request and an exhausted pool are different
    // operational signals.
    assert_ne!(EventStatus::Forbidden, EventStatus::PoolExhausted);
    assert_ne!(EventStatus::Forbidden, EventStatus::RateLimited);
    assert_ne!(EventStatus::PoolExhausted, EventStatus::UpstreamError);
}

#[test]
#[ignore = "Stage-5 REST adapter — kernel select() must emit a Forbidden event when the gate forbids; lands with the rest crate"]
fn forbidden_request_emits_forbidden_event() {
    // Placeholder marker for the integration assertion that the REST adapter
    // will own. Marked ignored so Stage-4 RED still passes the harness.
}
