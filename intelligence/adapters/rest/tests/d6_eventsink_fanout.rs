//! D6 EventSink fan-out contract tests (Stage-4 RED).
//!
//! Tests define the broadcast semantics that Stage-5 GREEN must satisfy.
//! All tests compile and most pass (fan-out logic is non-todo!()). Tests
//! asserting downstream sink behaviour use mock impls.

use intelligence_kernel::{
    AgentId, EventSink, EventStatus, LlmGatewayEvent, Provider, SeatId, TenantId,
};
use intelligence_rest::EventSinkFanout;
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Capturing sink that records every event it receives.
struct CaptureSink {
    events: Arc<Mutex<Vec<LlmGatewayEvent>>>,
}

impl CaptureSink {
    fn new() -> (Self, Arc<Mutex<Vec<LlmGatewayEvent>>>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                events: events.clone(),
            },
            events,
        )
    }
}

impl EventSink for CaptureSink {
    fn emit(&self, event: LlmGatewayEvent) {
        self.events.lock().unwrap().push(event);
    }
}

/// Sink that always panics — used to confirm fanout does NOT propagate panics
/// in Stage-5 GREEN (catch_unwind boundary). In Stage-4 this just confirms
/// compilation; the test is guarded with `#[should_panic]`.
struct PanickingSink;

impl EventSink for PanickingSink {
    fn emit(&self, _event: LlmGatewayEvent) {
        panic!("PanickingSink always panics");
    }
}

fn sample_event(tenant_id: TenantId, seat_id: SeatId) -> LlmGatewayEvent {
    LlmGatewayEvent {
        request_id: "req-abc123".to_string(),
        tenant_id,
        agent_id: AgentId::new("agent-bot").unwrap(),
        seat_id,
        provider: Provider::Anthropic,
        model: "claude-opus-4-5".to_string(),
        prompt_tokens: 512,
        completion_tokens: 256,
        ms_latency: 1200,
        status: EventStatus::Ok,
        timestamp_unix_ms: 1_748_390_400_000,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// D6-1: Empty fanout broadcasts zero events.
#[test]
fn d6_empty_fanout_delivers_nothing() {
    let fanout = EventSinkFanout::new();
    assert_eq!(fanout.sink_count(), 0);
    let tenant = TenantId::new("tenant-acme").unwrap();
    let seat = SeatId::new("seat-001").unwrap();
    let delivered = fanout.broadcast(sample_event(tenant, seat));
    assert_eq!(delivered, 0);
}

/// D6-2: Single sink receives the event.
#[test]
fn d6_single_sink_receives_event() {
    let mut fanout = EventSinkFanout::new();
    let (sink, capture) = CaptureSink::new();
    fanout.add_sink(Box::new(sink));
    let tenant = TenantId::new("tenant-acme").unwrap();
    let seat = SeatId::new("seat-001").unwrap();
    let event = sample_event(tenant, seat);
    fanout.broadcast(event.clone());
    let received = capture.lock().unwrap();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].request_id, "req-abc123");
}

/// D6-3: Two sinks both receive the same event.
#[test]
fn d6_two_sinks_both_receive_event() {
    let mut fanout = EventSinkFanout::new();
    let (sink_a, cap_a) = CaptureSink::new();
    let (sink_b, cap_b) = CaptureSink::new();
    fanout.add_sink(Box::new(sink_a));
    fanout.add_sink(Box::new(sink_b));
    let tenant = TenantId::new("tenant-x").unwrap();
    let seat = SeatId::new("seat-x1").unwrap();
    let delivered = fanout.broadcast(sample_event(tenant, seat));
    assert_eq!(delivered, 2);
    assert_eq!(cap_a.lock().unwrap().len(), 1);
    assert_eq!(cap_b.lock().unwrap().len(), 1);
}

/// D6-4: Broadcasting multiple events accumulates in each sink.
#[test]
fn d6_multiple_events_accumulate() {
    let mut fanout = EventSinkFanout::new();
    let (sink, capture) = CaptureSink::new();
    fanout.add_sink(Box::new(sink));
    let tenant = TenantId::new("tenant-acme").unwrap();
    let seat = SeatId::new("seat-001").unwrap();
    for _ in 0..5 {
        fanout.broadcast(sample_event(tenant.clone(), seat.clone()));
    }
    assert_eq!(capture.lock().unwrap().len(), 5);
}

/// D6-5: sink_count reflects registered sinks.
#[test]
fn d6_sink_count_matches_registrations() {
    let mut fanout = EventSinkFanout::new();
    assert_eq!(fanout.sink_count(), 0);
    let (s1, _) = CaptureSink::new();
    fanout.add_sink(Box::new(s1));
    assert_eq!(fanout.sink_count(), 1);
    let (s2, _) = CaptureSink::new();
    fanout.add_sink(Box::new(s2));
    assert_eq!(fanout.sink_count(), 2);
}

/// D6-6: EventStatus variants are distinguishable in captured events.
#[test]
fn d6_event_status_variants_captured() {
    use intelligence_kernel::EventStatus;
    let mut fanout = EventSinkFanout::new();
    let (sink, capture) = CaptureSink::new();
    fanout.add_sink(Box::new(sink));
    let tenant = TenantId::new("tenant-acme").unwrap();
    let seat = SeatId::new("seat-001").unwrap();
    let mut ev = sample_event(tenant, seat);
    ev.status = EventStatus::RateLimited;
    fanout.broadcast(ev);
    let received = capture.lock().unwrap();
    assert_eq!(received[0].status, EventStatus::RateLimited);
}

/// D6-7: Default::default() produces an empty fanout.
#[test]
fn d6_default_produces_empty_fanout() {
    let fanout = EventSinkFanout::default();
    assert_eq!(fanout.sink_count(), 0);
}

/// D6-8: PanickingSink panic is caught by the fanout catch_unwind boundary
/// (Stage-5 GREEN: broadcast wraps each sink call so proxy-path failures are
/// prevented). The broadcast returns 0 delivered (the panicking sink did not
/// successfully deliver), and the caller does NOT see a panic.
#[test]
fn d6_panicking_sink_panic_is_caught_by_fanout() {
    let mut fanout = EventSinkFanout::new();
    fanout.add_sink(Box::new(PanickingSink));
    let tenant = TenantId::new("tenant-acme").unwrap();
    let seat = SeatId::new("seat-001").unwrap();
    // Must NOT panic — catch_unwind swallows the PanickingSink panic.
    let delivered = fanout.broadcast(sample_event(tenant, seat));
    assert_eq!(delivered, 0, "panicking sink should not count as delivered");
}
