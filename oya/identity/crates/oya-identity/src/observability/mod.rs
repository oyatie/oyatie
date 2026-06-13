//! Observability subsystem: structured tracing + audit emission.
//!
//! JSON-formatted tracing for K8s log collection (one structured event per
//! request-path decision), env-filtered via `RUST_LOG` (default `info`).
//! [`TracingAuditSink`] bridges the workload-identity [`AuditSink`] port onto
//! the tracing pipeline so every authorize/token-validation decision is on the
//! log stream from first boot; the audit-chain bridge (CloudEvents envelope +
//! signed digest chain, G08 lane) lands behind the SAME port.

use oya_identity_workload_rest::{AuditRecord, AuditSink};

/// Install the global tracing subscriber (idempotent: a second call is a
/// no-op so tests can race it safely).
pub fn init() {
    let _ = tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();
}

/// [`AuditSink`] that emits each sealed record as a structured tracing event.
#[derive(Clone, Copy, Debug, Default)]
pub struct TracingAuditSink;

impl TracingAuditSink {
    /// Build the sink.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl AuditSink for TracingAuditSink {
    fn record(&self, record: AuditRecord) {
        tracing::info!(
            target: "oya_identity::audit",
            event = record.event().label(),
            workload_id = record.workload_id().unwrap_or("-"),
            outcome = record.outcome(),
            detail = record.detail().unwrap_or("-"),
            action = record.action().unwrap_or("-"),
            resource_type = record.resource_type().unwrap_or("-"),
            resource_id = record.resource_id().unwrap_or("-"),
            "audit-record",
        );
    }
}
