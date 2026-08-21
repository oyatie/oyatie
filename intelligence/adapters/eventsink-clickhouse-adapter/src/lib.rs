//! ClickHouse EventSink adapter for the cloud-intelligence OAuth subscription pool
//! (ADR-0384 Path B, Stage-7 D6 production seam).
//!
//! Implements [`EventSink`] from `intelligence-kernel` by
//! INSERTing [`LlmGatewayEvent`] rows into the `cloud_intelligence_receipts`
//! table in the caller's per-tenant ClickHouse database via the shared
//! [`shared_olap_clickhouse_adapter::ClickHouseOlapClient`] (ADR-0193).
//!
//! ## Insert shape
//!
//! Each [`LlmGatewayEvent`] maps to one row:
//!
//! ```text
//! INSERT INTO tenant_{tenant_id}.cloud_intelligence_receipts
//!   (request_id, tenant_id, agent_id, seat_id, provider, model,
//!    prompt_tokens, completion_tokens, ms_latency, status, timestamp_unix_ms)
//! VALUES (...)
//! ```
//!
//! ## non_claims
//!
//! - emit is best-effort: failures are logged via `tracing::warn` but never
//!   propagate to the caller (D6 non-fatal contract).
//! - no batching / coalescing (Stage-8 follow-up).
//! - no DDL bootstrap: the `cloud_intelligence_receipts` table must exist
//!   before the adapter is used (Stage-7 admin runbook item).
//! - no retry: transient ClickHouse errors are logged and dropped.
//!
//! ADR-0083 Tier-3 panic-free: no `unwrap`, `expect`, or `panic!` outside tests.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;
use std::sync::Mutex;

use intelligence_kernel::{EventSink, LlmGatewayEvent};
use shared_olap_clickhouse_adapter::{ClickHouseConfig, ClickHouseOlapClient};
use shared_olap_client_kernel::{
    InsertBatch, OlapClient, QualifiedTable, TableName, TenantId, Value,
};
use tracing::warn;

// ---------------------------------------------------------------------------
// Table constant
// ---------------------------------------------------------------------------

const TABLE: &str = "cloud_intelligence_receipts";

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors raised during event emission. Non-fatal per D6 contract.
#[derive(Debug)]
pub enum ClickHouseSinkError {
    /// Row construction failed (e.g. invalid tenant_id format).
    RowBuild(String),
    /// ClickHouse INSERT returned an error.
    Insert(String),
}

impl fmt::Display for ClickHouseSinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClickHouseSinkError::RowBuild(msg) => write!(f, "row build error: {msg}"),
            ClickHouseSinkError::Insert(msg) => write!(f, "clickhouse insert error: {msg}"),
        }
    }
}

// ---------------------------------------------------------------------------
// ClickHouseEventSink
// ---------------------------------------------------------------------------

/// [`EventSink`] backed by ClickHouse 26.3 LTS.
///
/// Wraps a [`ClickHouseOlapClient`] behind a `Mutex` so the sync trait method
/// can mutably borrow the client. The mutex is uncontended in the common case
/// (one emitter per service instance).
pub struct ClickHouseEventSink {
    client: Mutex<ClickHouseOlapClient>, // data_class: INTERNAL_ONLY
}

impl fmt::Debug for ClickHouseEventSink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClickHouseEventSink")
            .finish_non_exhaustive()
    }
}

impl ClickHouseEventSink {
    /// Construct from a [`ClickHouseConfig`].
    pub fn new(config: ClickHouseConfig) -> Self {
        Self {
            client: Mutex::new(ClickHouseOlapClient::new(config)),
        }
    }

    /// Emit one event, returning a typed error for test assertions.
    fn try_emit(&self, event: &LlmGatewayEvent) -> Result<(), ClickHouseSinkError> {
        let tenant_id = TenantId::try_new(event.tenant_id.as_str())
            .map_err(|e| ClickHouseSinkError::RowBuild(format!("tenant_id invalid: {e:?}")))?;

        let table = TableName::try_new(TABLE)
            .map_err(|e| ClickHouseSinkError::RowBuild(format!("table name invalid: {e:?}")))?;

        let target = QualifiedTable::new(tenant_id, table);

        // Column order must match the row values order below.
        let columns = vec![
            "request_id".to_string(),
            "tenant_id".to_string(),
            "agent_id".to_string(),
            "seat_id".to_string(),
            "provider".to_string(),
            "model".to_string(),
            "prompt_tokens".to_string(),
            "completion_tokens".to_string(),
            "ms_latency".to_string(),
            "status".to_string(),
            "timestamp_unix_ms".to_string(),
        ];

        let row = vec![
            Value::String(event.request_id.clone()),
            Value::String(event.tenant_id.as_str().to_string()),
            Value::String(event.agent_id.as_str().to_string()),
            Value::String(event.seat_id.as_str().to_string()),
            Value::String(event.provider.to_string()),
            Value::String(event.model.clone()),
            Value::UInt(event.prompt_tokens),
            Value::UInt(event.completion_tokens),
            Value::UInt(event.ms_latency),
            Value::String(format!("{:?}", event.status)),
            Value::UInt(event.timestamp_unix_ms),
        ];

        let batch = InsertBatch {
            target,
            columns,
            rows: vec![row],
        };

        let mut client = self
            .client
            .lock()
            .map_err(|_| ClickHouseSinkError::Insert("mutex poisoned".to_string()))?;

        client
            .insert(&batch)
            .map_err(|e| ClickHouseSinkError::Insert(e.to_string()))?;

        Ok(())
    }
}

impl EventSink for ClickHouseEventSink {
    /// Emit one event to ClickHouse. Errors are logged via `tracing::warn` and
    /// swallowed per the D6 non-fatal contract.
    fn emit(&self, event: LlmGatewayEvent) {
        if let Err(e) = self.try_emit(&event) {
            warn!(
                request_id = %event.request_id,
                error = %e,
                "ClickHouseEventSink: emit failed (non-fatal)"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use intelligence_kernel::{AgentId, EventStatus, Provider, SeatId, TenantId as KernelTenantId};

    fn test_config() -> ClickHouseConfig {
        ClickHouseConfig {
            url: "http://clickhouse.test:8123".to_string(),
            user: "default".to_string(),
            password: "test".to_string(),
        }
    }

    fn test_event() -> LlmGatewayEvent {
        LlmGatewayEvent {
            request_id: "req-001".to_string(),
            tenant_id: KernelTenantId::new("tenant-a").unwrap(),
            agent_id: AgentId::new("agent-1").unwrap(),
            seat_id: SeatId::new("seat-1").unwrap(),
            provider: Provider::Anthropic,
            model: "claude-3-5-sonnet".to_string(),
            prompt_tokens: 100,
            completion_tokens: 50,
            ms_latency: 320,
            status: EventStatus::Ok,
            timestamp_unix_ms: 1_700_000_000_000,
        }
    }

    #[test]
    fn sink_constructs_without_panic() {
        let _sink = ClickHouseEventSink::new(test_config());
    }

    #[test]
    fn emit_non_fatal_on_clickhouse_error() {
        // ClickHouseOlapClient.insert() returns AdapterError (IP-003 deferred).
        // emit() must not panic; the error is swallowed per D6 contract.
        let sink = ClickHouseEventSink::new(test_config());
        sink.emit(test_event()); // must not panic
    }

    #[test]
    fn try_emit_returns_insert_error_on_deferred_backend() {
        let sink = ClickHouseEventSink::new(test_config());
        let result = sink.try_emit(&test_event());
        // The ClickHouse adapter is plan-only (IP-003); it always returns
        // AdapterError, which we map to ClickHouseSinkError::Insert.
        assert!(
            matches!(result, Err(ClickHouseSinkError::Insert(_))),
            "expected Insert error from deferred backend, got: {result:?}"
        );
    }

    #[test]
    fn try_emit_error_on_invalid_tenant_id() {
        let sink = ClickHouseEventSink::new(test_config());
        let mut event = test_event();
        // Force an invalid tenant_id by constructing a kernel TenantId we
        // can't create with an empty string, so we patch via a wrapper:
        // instead build an event with a valid kernel TenantId but inject a
        // tenant whose oya-shared-olap-client-kernel TenantId::try_new would
        // reject (the olap kernel disallows chars beyond alphanumeric/-/_).
        event.tenant_id = KernelTenantId::new("tenant a").unwrap(); // space is invalid for olap kernel
        let result = sink.try_emit(&event);
        assert!(
            matches!(result, Err(ClickHouseSinkError::RowBuild(_))),
            "expected RowBuild error for invalid tenant_id, got: {result:?}"
        );
    }

    #[test]
    fn emit_does_not_panic_for_any_event_status() {
        let sink = ClickHouseEventSink::new(test_config());
        for status in [
            EventStatus::Ok,
            EventStatus::UpstreamError,
            EventStatus::RateLimited,
            EventStatus::Forbidden,
            EventStatus::PoolExhausted,
        ] {
            let mut ev = test_event();
            ev.status = status;
            sink.emit(ev); // must not panic
        }
    }
}
