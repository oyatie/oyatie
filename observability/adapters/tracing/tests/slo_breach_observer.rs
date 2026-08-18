// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Integration tests for `SloBreachTraceObserver`.
//!
//! All tests use a scoped subscriber (`tracing::subscriber::with_default`) —
//! no global install, so tests are fully independent and composable.
//!
//! Span-field capture uses `tracing_subscriber`'s `fmt::TestWriter` routed
//! through a string buffer so assertions can inspect emitted field text without
//! any external dependency beyond what the workspace already provides.

use std::sync::{Arc, Mutex};

use observability_tracing_adapter::{
    AlertBurnRate, AlertDecision, NoopSloBreachTraceObserver, SLO_BREACH_SPAN_NAME,
    SloBreachTraceContext, SloBreachTraceObserver, SloObjective, TracingSloBreachTraceObserver,
    slo_fields,
};
use tracing_subscriber::fmt::MakeWriter;

// ---------------------------------------------------------------------------
// Minimal in-memory writer so we can capture formatted span output
// ---------------------------------------------------------------------------

#[derive(Clone, Default)]
struct BufWriter(Arc<Mutex<Vec<u8>>>);

impl BufWriter {
    fn contents(&self) -> String {
        let bytes = self.0.lock().unwrap();
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

impl<'a> MakeWriter<'a> for BufWriter {
    type Writer = BufWriterGuard;

    fn make_writer(&'a self) -> Self::Writer {
        BufWriterGuard(Arc::clone(&self.0))
    }
}

struct BufWriterGuard(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for BufWriterGuard {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helper: build a scoped JSON subscriber that writes to a BufWriter
// ---------------------------------------------------------------------------

fn make_scoped_subscriber(writer: BufWriter) -> impl tracing::Subscriber + Send + Sync + 'static {
    use tracing_subscriber::{EnvFilter, fmt as tracing_fmt};
    tracing_fmt()
        .json()
        .with_env_filter(EnvFilter::new("info"))
        .with_current_span(true)
        .with_span_list(true)
        .with_writer(writer)
        .finish()
}

// ---------------------------------------------------------------------------
// Helper: build a canonical SloBreachTraceContext
// ---------------------------------------------------------------------------

fn page_context() -> SloBreachTraceContext {
    SloBreachTraceContext {
        objective: SloObjective {
            name: "api.latency.p99".to_string(),
            target: 0.999,
        },
        burn_rate: AlertBurnRate {
            short_window: 14.5,
            long_window: 6.2,
            error_budget_consumed: 0.42,
        },
        decision: AlertDecision::Page,
    }
}

fn ticket_context() -> SloBreachTraceContext {
    SloBreachTraceContext {
        objective: SloObjective {
            name: "api.availability".to_string(),
            target: 0.995,
        },
        burn_rate: AlertBurnRate {
            short_window: 3.1,
            long_window: 2.0,
            error_budget_consumed: 0.18,
        },
        decision: AlertDecision::Ticket,
    }
}

fn none_context() -> SloBreachTraceContext {
    SloBreachTraceContext {
        objective: SloObjective {
            name: "api.error_rate".to_string(),
            target: 0.99,
        },
        burn_rate: AlertBurnRate {
            short_window: 0.9,
            long_window: 0.7,
            error_budget_consumed: 0.05,
        },
        decision: AlertDecision::None,
    }
}

// ---------------------------------------------------------------------------
// Test: span name constant is stable
// ---------------------------------------------------------------------------

#[test]
fn slo_breach_span_name_is_stable() {
    assert_eq!(
        SLO_BREACH_SPAN_NAME, "slo.breach.evaluate",
        "span name must be stable; downstream OTLP consumers depend on it"
    );
}

// ---------------------------------------------------------------------------
// Test: Page decision — all 6 fields present, decision = "page"
// ---------------------------------------------------------------------------

#[test]
fn slo_breach_page_decision_records_all_fields() {
    let buf = BufWriter::default();
    let subscriber = make_scoped_subscriber(buf.clone());

    let ctx = page_context();
    let observer = TracingSloBreachTraceObserver;

    tracing::subscriber::with_default(subscriber, || {
        observer.observe(&ctx);
    });

    let output = buf.contents();

    // Span name
    assert!(
        output.contains(SLO_BREACH_SPAN_NAME),
        "output must contain span name '{SLO_BREACH_SPAN_NAME}'; got: {output}"
    );
    // SLO name field
    assert!(
        output.contains(slo_fields::SLO_NAME) && output.contains("api.latency.p99"),
        "output must contain slo.name='api.latency.p99'; got: {output}"
    );
    // SLO objective field
    assert!(
        output.contains(slo_fields::SLO_OBJECTIVE),
        "output must contain field key '{}'; got: {output}",
        slo_fields::SLO_OBJECTIVE
    );
    // Error budget consumed
    assert!(
        output.contains(slo_fields::SLO_ERROR_BUDGET_CONSUMED),
        "output must contain field key '{}'; got: {output}",
        slo_fields::SLO_ERROR_BUDGET_CONSUMED
    );
    // Burn rate fields
    assert!(
        output.contains(slo_fields::SLO_BURN_RATE_SHORT),
        "output must contain field key '{}'; got: {output}",
        slo_fields::SLO_BURN_RATE_SHORT
    );
    assert!(
        output.contains(slo_fields::SLO_BURN_RATE_LONG),
        "output must contain field key '{}'; got: {output}",
        slo_fields::SLO_BURN_RATE_LONG
    );
    // Alert decision = page
    assert!(
        output.contains(slo_fields::SLO_ALERT_DECISION) && output.contains("page"),
        "output must contain slo.alert.decision='page'; got: {output}"
    );
}

// ---------------------------------------------------------------------------
// Test: Ticket decision — decision field = "ticket"
// ---------------------------------------------------------------------------

#[test]
fn slo_breach_ticket_decision_records_all_fields() {
    let buf = BufWriter::default();
    let subscriber = make_scoped_subscriber(buf.clone());

    let ctx = ticket_context();
    let observer = TracingSloBreachTraceObserver;

    tracing::subscriber::with_default(subscriber, || {
        observer.observe(&ctx);
    });

    let output = buf.contents();

    assert!(
        output.contains(SLO_BREACH_SPAN_NAME),
        "output must contain span name; got: {output}"
    );
    assert!(
        output.contains(slo_fields::SLO_ALERT_DECISION) && output.contains("ticket"),
        "output must contain slo.alert.decision='ticket'; got: {output}"
    );
    assert!(
        output.contains("api.availability"),
        "output must contain slo name 'api.availability'; got: {output}"
    );
}

// ---------------------------------------------------------------------------
// Test: None decision — decision field = "none"
// ---------------------------------------------------------------------------

#[test]
fn slo_breach_none_decision_records_all_fields() {
    let buf = BufWriter::default();
    let subscriber = make_scoped_subscriber(buf.clone());

    let ctx = none_context();
    let observer = TracingSloBreachTraceObserver;

    tracing::subscriber::with_default(subscriber, || {
        observer.observe(&ctx);
    });

    let output = buf.contents();

    assert!(
        output.contains(SLO_BREACH_SPAN_NAME),
        "output must contain span name; got: {output}"
    );
    assert!(
        output.contains(slo_fields::SLO_ALERT_DECISION) && output.contains("none"),
        "output must contain slo.alert.decision='none'; got: {output}"
    );
    assert!(
        output.contains("api.error_rate"),
        "output must contain slo name 'api.error_rate'; got: {output}"
    );
}

// ---------------------------------------------------------------------------
// Test: Noop observer emits nothing — no span, no panic
// ---------------------------------------------------------------------------

#[test]
fn noop_slo_breach_observer_emits_nothing() {
    // Use a buf subscriber so we can assert nothing was written, not just
    // that there was no panic.
    let buf = BufWriter::default();
    let subscriber = make_scoped_subscriber(buf.clone());

    let ctx = page_context();
    let observer = NoopSloBreachTraceObserver;

    tracing::subscriber::with_default(subscriber, || {
        observer.observe(&ctx);
    });

    let output = buf.contents();
    assert!(
        output.is_empty(),
        "noop observer must produce no span output; got: {output}"
    );
}
