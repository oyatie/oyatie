//! `oya-meter-kernel` — pure Rust usage metering state machine (ADR-0479).
//!
//! This layer owns the usage event ingestion port and aggregation primitives.
//! No I/O, no async, no external crates. All transitions are pure functions of
//! their inputs so the machine is fully deterministic and unit-testable.
//!
//! # What this layer owns
//! - [`UsageEvent`] — a single metered usage record (tenant, resource, quantity).
//! - [`UsageAggregator`] — accumulates events and produces period summaries.
//!
//! # What this layer must NEVER do
//! No async, no I/O, no network, no clock reads (inject timestamps as values),
//! no external crate. Mirrors the discipline of `oya-llm-gateway-kernel`.

#![forbid(unsafe_code)]

// TODO: implement per ADR-0479 D1-D5

/// A single metered usage event (tenant + resource + quantity).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsageEvent {
    /// Opaque tenant identifier.
    pub tenant_id: String,
    /// Resource being metered (e.g. "llm_tokens", "api_calls").
    pub resource: String,
    /// Quantity consumed in this event.
    pub quantity: u64,
    /// Wall-clock timestamp supplied by the caller (unix millis).
    pub timestamp_millis: u64,
}

/// Aggregates [`UsageEvent`]s and surfaces per-tenant totals.
///
/// All state is in-memory. Persistence adapters live in higher layers.
#[derive(Debug, Default)]
pub struct UsageAggregator {
    // TODO: implement per ADR-0479 D1-D5
    events: Vec<UsageEvent>,
}

impl UsageAggregator {
    /// Construct an empty aggregator.
    #[must_use]
    pub fn new() -> Self {
        UsageAggregator::default()
    }

    /// Ingest a single usage event.
    pub fn ingest(&mut self, event: UsageEvent) {
        // TODO: implement per ADR-0479 D1-D5
        self.events.push(event);
    }

    /// Return the total quantity for `tenant_id` and `resource` across all
    /// ingested events. Returns `0` if no matching events exist.
    #[must_use]
    pub fn total(&self, tenant_id: &str, resource: &str) -> u64 {
        // TODO: implement per ADR-0479 D1-D5
        self.events
            .iter()
            .filter(|e| e.tenant_id == tenant_id && e.resource == resource)
            .map(|e| e.quantity)
            .fold(0u64, u64::saturating_add)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ingest_and_total_single_tenant() {
        let mut agg = UsageAggregator::new();
        agg.ingest(UsageEvent {
            tenant_id: "t1".into(),
            resource: "api_calls".into(),
            quantity: 5,
            timestamp_millis: 0,
        });
        agg.ingest(UsageEvent {
            tenant_id: "t1".into(),
            resource: "api_calls".into(),
            quantity: 3,
            timestamp_millis: 1,
        });
        assert_eq!(agg.total("t1", "api_calls"), 8);
        assert_eq!(agg.total("t1", "llm_tokens"), 0);
        assert_eq!(agg.total("t2", "api_calls"), 0);
    }
}
