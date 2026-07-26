//! The runtime event-sink controller.
//!
//! Mirrors the Talos event-sink machinery
//! (`internal/app/machined/pkg/controllers/runtime/event_sink_config.go` plus
//! the `pkg/machinery/api/resource/definitions/runtime` `EventSinkConfig` and
//! the `events.Sink` gRPC publisher). The machined runtime can be configured
//! with a remote `events` collector (`apid` on the control plane). Events
//! published onto the [`EventStream`](crate::event_stream::EventStream) are
//! forwarded to that endpoint; if the endpoint is unreachable the controller
//! retries with backoff and resumes from the last acknowledged id so no event
//! is dropped while the sink is reachable.
//!
//! The network boundary is modeled as the [`EventEndpoint`] trait with an
//! in-memory implementation used by tests; the controller itself is a pure,
//! deterministic state machine driven by a `reconcile`/`drain` step.

use crate::events::Event;
use os_kernel::error::{Error, Result};

/// Static configuration of the event sink.
///
/// Mirrors `runtime.EventSinkConfig`: a single endpoint (`host:port`) the
/// machined forwards runtime events to. An empty endpoint disables forwarding.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EventSinkConfig {
    /// `host:port` of the remote events collector, or empty to disable.
    pub endpoint: String,
}

impl EventSinkConfig {
    /// Disabled config (no endpoint).
    pub fn disabled() -> Self {
        EventSinkConfig::default()
    }

    /// Build and validate a config from an endpoint string.
    pub fn new(endpoint: impl Into<String>) -> Result<Self> {
        let endpoint = endpoint.into();
        let trimmed = endpoint.trim();
        if trimmed.is_empty() {
            return Ok(EventSinkConfig::disabled());
        }
        if !trimmed.contains(':') {
            return Err(Error::invalid(format!(
                "event sink endpoint '{trimmed}' must be host:port"
            )));
        }
        // host and port both non-empty.
        let (host, port) = trimmed.rsplit_once(':').unwrap();
        if host.is_empty() {
            return Err(Error::invalid("event sink endpoint missing host"));
        }
        port.parse::<u16>()
            .map_err(|_| Error::invalid(format!("event sink endpoint port '{port}' invalid")))?;
        Ok(EventSinkConfig {
            endpoint: trimmed.to_string(),
        })
    }

    /// Whether forwarding is enabled.
    pub fn is_enabled(&self) -> bool {
        !self.endpoint.is_empty()
    }
}

/// Result of attempting to deliver a batch of events to the remote endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeliveryOutcome {
    /// All events accepted; the argument is the highest id acknowledged.
    Acked(u64),
    /// The endpoint rejected/failed; nothing was accepted.
    Failed(String),
}

/// Network boundary to the remote event collector.
///
/// In production this is a gRPC `EventSinkService/Publish` client; tests use
/// [`MemoryEndpoint`].
pub trait EventEndpoint {
    /// Attempt to publish a batch of events. On success returns the highest id
    /// that was durably accepted by the remote.
    fn publish(&mut self, events: &[Event]) -> DeliveryOutcome;
}

/// In-memory [`EventEndpoint`] for tests: records delivered events and can be
/// toggled offline to exercise the controller's retry path.
#[derive(Debug, Default, Clone)]
pub struct MemoryEndpoint {
    /// All successfully delivered events, in order.
    pub delivered: Vec<Event>,
    /// When `true`, every `publish` fails.
    pub offline: bool,
    /// Count of failed publish attempts.
    pub failures: usize,
}

impl MemoryEndpoint {
    /// A reachable endpoint.
    pub fn new() -> Self {
        Self::default()
    }

    /// An endpoint that starts offline.
    pub fn offline() -> Self {
        MemoryEndpoint {
            offline: true,
            ..Default::default()
        }
    }

    /// Toggle reachability.
    pub fn set_offline(&mut self, offline: bool) {
        self.offline = offline;
    }
}

impl EventEndpoint for MemoryEndpoint {
    fn publish(&mut self, events: &[Event]) -> DeliveryOutcome {
        if self.offline {
            self.failures += 1;
            return DeliveryOutcome::Failed("endpoint offline".into());
        }
        if events.is_empty() {
            // Nothing to do; report the last delivered id.
            return DeliveryOutcome::Acked(self.delivered.last().map_or(0, |e| e.id));
        }
        self.delivered.extend_from_slice(events);
        DeliveryOutcome::Acked(events.last().unwrap().id)
    }
}

/// Exponential backoff state used between failed delivery attempts.
///
/// Mirrors the retry policy Talos applies around the event sink client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backoff {
    base_ms: u64,
    max_ms: u64,
    /// Number of consecutive failures.
    attempts: u32,
}

impl Backoff {
    /// Create a backoff with the given base and ceiling (milliseconds).
    pub fn new(base_ms: u64, max_ms: u64) -> Self {
        Backoff {
            base_ms: base_ms.max(1),
            max_ms: max_ms.max(base_ms.max(1)),
            attempts: 0,
        }
    }

    /// Talos-like default: 100ms base, 30s ceiling.
    pub fn default_policy() -> Self {
        Backoff::new(100, 30_000)
    }

    /// Current consecutive-failure count.
    pub fn attempts(&self) -> u32 {
        self.attempts
    }

    /// The delay that should be waited *before* the next attempt, given the
    /// failures observed so far (`base * 2^(attempts-1)` capped at `max`). Zero
    /// while there have been no failures.
    pub fn current_delay_ms(&self) -> u64 {
        if self.attempts == 0 {
            return 0;
        }
        let shift = (self.attempts - 1).min(63);
        let scaled = self.base_ms.saturating_mul(1u64 << shift);
        scaled.min(self.max_ms)
    }

    /// Record a failure, advancing the backoff and returning the new delay.
    pub fn record_failure(&mut self) -> u64 {
        self.attempts = self.attempts.saturating_add(1);
        self.current_delay_ms()
    }

    /// Reset after a success.
    pub fn reset(&mut self) {
        self.attempts = 0;
    }
}

/// The event-sink controller: a deterministic state machine that owns a delivery
/// cursor and a backoff, and drains pending events from an
/// [`EventStream`](crate::event_stream::EventStream)-like source to an
/// [`EventEndpoint`].
///
/// The cursor (`last_acked`) is the highest event id durably accepted by the
/// remote. On a successful `drain` it advances; on failure it stays put and the
/// backoff advances so the same events are retried.
pub struct EventSinkController<E: EventEndpoint> {
    config: EventSinkConfig,
    endpoint: E,
    last_acked: u64,
    backoff: Backoff,
    /// Total events delivered across all drains.
    delivered_total: u64,
    /// Number of drains that resulted in a failure.
    failed_drains: u64,
}

impl<E: EventEndpoint> EventSinkController<E> {
    /// Build a controller from config and an endpoint.
    pub fn new(config: EventSinkConfig, endpoint: E) -> Self {
        EventSinkController {
            config,
            endpoint,
            last_acked: 0,
            backoff: Backoff::default_policy(),
            delivered_total: 0,
            failed_drains: 0,
        }
    }

    /// The configured endpoint string.
    pub fn config(&self) -> &EventSinkConfig {
        &self.config
    }

    /// The highest id acknowledged by the remote so far.
    pub fn last_acked(&self) -> u64 {
        self.last_acked
    }

    /// Current backoff delay (ms) before the next retry; 0 when healthy.
    pub fn backoff_delay_ms(&self) -> u64 {
        self.backoff.current_delay_ms()
    }

    /// Whether the sink is currently in a retry/backoff state.
    pub fn is_backing_off(&self) -> bool {
        self.backoff.attempts() > 0
    }

    /// Total events delivered.
    pub fn delivered_total(&self) -> u64 {
        self.delivered_total
    }

    /// Number of failed drains.
    pub fn failed_drains(&self) -> u64 {
        self.failed_drains
    }

    /// Borrow the endpoint (e.g. to inspect delivered events in tests).
    pub fn endpoint(&self) -> &E {
        &self.endpoint
    }

    /// Mutable access to the endpoint (e.g. to toggle reachability in tests).
    pub fn endpoint_mut(&mut self) -> &mut E {
        &mut self.endpoint
    }

    /// Attempt to deliver all `events` whose id is strictly greater than the
    /// current cursor to the remote endpoint, in order. Events must be sorted by
    /// id ascending (as the stream yields them).
    ///
    /// Returns the [`DeliveryOutcome`]. On `Acked`, the cursor advances and the
    /// backoff resets; on `Failed`, the cursor is unchanged and the backoff
    /// advances. When forwarding is disabled the call is a no-op `Acked(cursor)`.
    pub fn drain(&mut self, events: &[Event]) -> DeliveryOutcome {
        if !self.config.is_enabled() {
            return DeliveryOutcome::Acked(self.last_acked);
        }
        let pending: Vec<Event> = events
            .iter()
            .filter(|e| e.id > self.last_acked)
            .cloned()
            .collect();
        if pending.is_empty() {
            return DeliveryOutcome::Acked(self.last_acked);
        }
        match self.endpoint.publish(&pending) {
            DeliveryOutcome::Acked(id) => {
                let id = id.max(self.last_acked);
                self.delivered_total += pending.iter().filter(|e| e.id <= id).count() as u64;
                self.last_acked = id;
                self.backoff.reset();
                DeliveryOutcome::Acked(id)
            }
            DeliveryOutcome::Failed(msg) => {
                self.failed_drains += 1;
                self.backoff.record_failure();
                DeliveryOutcome::Failed(msg)
            }
        }
    }

    /// Number of events from `events` still awaiting acknowledgement (id beyond
    /// the cursor).
    pub fn pending_count(&self, events: &[Event]) -> usize {
        events.iter().filter(|e| e.id > self.last_acked).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_stream::EventStream;
    use crate::events::ServiceAction;

    fn stream_with(n: u64) -> EventStream {
        let mut s = EventStream::with_capacity(64);
        for _ in 0..n {
            s.publish(Event::service("system", "etcd", ServiceAction::Running));
        }
        s
    }

    #[test]
    fn config_validation() {
        assert!(!EventSinkConfig::disabled().is_enabled());
        assert!(EventSinkConfig::new("").unwrap().endpoint.is_empty());
        let c = EventSinkConfig::new("10.0.0.1:50001").unwrap();
        assert!(c.is_enabled());
        assert_eq!(c.endpoint, "10.0.0.1:50001");
        assert!(EventSinkConfig::new("nohostport").is_err());
        assert!(EventSinkConfig::new(":50001").is_err());
        assert!(EventSinkConfig::new("host:notaport").is_err());
        assert!(EventSinkConfig::new("host:70000").is_err());
    }

    #[test]
    fn disabled_sink_is_noop() {
        let mut ctrl = EventSinkController::new(EventSinkConfig::disabled(), MemoryEndpoint::new());
        let evs = stream_with(3).tail(3);
        assert_eq!(ctrl.drain(&evs), DeliveryOutcome::Acked(0));
        assert!(ctrl.endpoint().delivered.is_empty());
    }

    #[test]
    fn happy_path_advances_cursor() {
        let cfg = EventSinkConfig::new("h:1").unwrap();
        let mut ctrl = EventSinkController::new(cfg, MemoryEndpoint::new());
        let evs = stream_with(3).tail(3);
        let out = ctrl.drain(&evs);
        assert_eq!(out, DeliveryOutcome::Acked(3));
        assert_eq!(ctrl.last_acked(), 3);
        assert_eq!(ctrl.delivered_total(), 3);
        assert_eq!(ctrl.endpoint().delivered.len(), 3);
        // re-draining the same events is a no-op (cursor already past them).
        assert_eq!(ctrl.drain(&evs), DeliveryOutcome::Acked(3));
        assert_eq!(ctrl.endpoint().delivered.len(), 3);
        assert!(!ctrl.is_backing_off());
    }

    #[test]
    fn incremental_drain_only_sends_new_events() {
        let cfg = EventSinkConfig::new("h:1").unwrap();
        let mut ctrl = EventSinkController::new(cfg, MemoryEndpoint::new());
        let mut s = stream_with(2);
        ctrl.drain(&s.tail(2));
        assert_eq!(ctrl.last_acked(), 2);
        // publish two more, drain everything retained: only ids 3,4 sent.
        s.publish(Event::service("system", "kubelet", ServiceAction::Running));
        s.publish(Event::service("system", "kubelet", ServiceAction::Failed));
        let out = ctrl.drain(&s.tail(s.len()));
        assert_eq!(out, DeliveryOutcome::Acked(4));
        assert_eq!(ctrl.endpoint().delivered.len(), 4);
        assert_eq!(ctrl.endpoint().delivered[2].id, 3);
    }

    #[test]
    fn failure_retries_then_recovers() {
        let cfg = EventSinkConfig::new("h:1").unwrap();
        let mut ctrl = EventSinkController::new(cfg, MemoryEndpoint::offline());
        let evs = stream_with(3).tail(3);

        let out = ctrl.drain(&evs);
        assert!(matches!(out, DeliveryOutcome::Failed(_)));
        assert_eq!(ctrl.last_acked(), 0);
        assert!(ctrl.is_backing_off());
        assert_eq!(ctrl.failed_drains(), 1);
        let d1 = ctrl.backoff_delay_ms();
        assert_eq!(d1, 100);

        // second failure grows the backoff.
        ctrl.drain(&evs);
        assert_eq!(ctrl.backoff_delay_ms(), 200);
        assert_eq!(ctrl.pending_count(&evs), 3);

        // endpoint comes back; same events delivered, cursor catches up, backoff reset.
        ctrl.endpoint_mut().set_offline(false);
        let out = ctrl.drain(&evs);
        assert_eq!(out, DeliveryOutcome::Acked(3));
        assert_eq!(ctrl.last_acked(), 3);
        assert!(!ctrl.is_backing_off());
        assert_eq!(ctrl.backoff_delay_ms(), 0);
        assert_eq!(ctrl.pending_count(&evs), 0);
        // exactly the 3 events delivered, not duplicated by the retries.
        assert_eq!(ctrl.endpoint().delivered.len(), 3);
    }

    #[test]
    fn backoff_caps_at_max() {
        let mut b = Backoff::new(100, 500);
        assert_eq!(b.current_delay_ms(), 0);
        assert_eq!(b.record_failure(), 100);
        assert_eq!(b.record_failure(), 200);
        assert_eq!(b.record_failure(), 400);
        assert_eq!(b.record_failure(), 500); // capped
        assert_eq!(b.record_failure(), 500);
        b.reset();
        assert_eq!(b.attempts(), 0);
        assert_eq!(b.current_delay_ms(), 0);
    }

    #[test]
    fn empty_drain_is_acked_at_cursor() {
        let cfg = EventSinkConfig::new("h:1").unwrap();
        let mut ctrl = EventSinkController::new(cfg, MemoryEndpoint::new());
        assert_eq!(ctrl.drain(&[]), DeliveryOutcome::Acked(0));
    }
}
