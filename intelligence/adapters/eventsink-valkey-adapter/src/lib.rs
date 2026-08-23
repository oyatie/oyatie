//! Valkey Stream EventSink adapter for the intelligence-app OAuth subscription pool
//! (ADR-0384 Path B, Stage-7 D6 production seam).
//!
//! Implements [`EventSink`] from `intelligence-kernel` by
//! emitting [`LlmGatewayEvent`] to the Valkey Stream key
//! `intelligence-app-receipts:<tenant_id>` via `XADD`.
//!
//! Valkey is a Redis-protocol-compatible fork; the `redis` crate connects to
//! it without modification.
//!
//! ## Stream key shape
//!
//! `intelligence-app-receipts:<tenant_id>`  (one stream per tenant)
//!
//! ## XADD field mapping
//!
//! Each [`LlmGatewayEvent`] maps to a flat XADD field list:
//! `request_id`, `tenant_id`, `agent_id`, `seat_id`, `provider`, `model`,
//! `prompt_tokens`, `completion_tokens`, `ms_latency`, `status`,
//! `timestamp_unix_ms`.
//!
//! ## non_claims
//!
//! - emit is best-effort: transport failures are logged via `tracing::warn`
//!   and never propagate to the caller (D6 non-fatal contract).
//! - no consumer-group management or MAXLEN trimming (Stage-8 follow-up).
//! - no automatic reconnect beyond what the `redis` crate provides.
//! - no TLS client-certificate authentication (operator supplies `rediss://` URL).
//!
//! ADR-0083 Tier-3 panic-free: no `unwrap`, `expect`, or `panic!` outside tests.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;
use std::sync::Mutex;

use intelligence_kernel::{EventSink, LlmGatewayEvent};
use redis::Commands;
use tracing::warn;

// ---------------------------------------------------------------------------
// Stream key helper
// ---------------------------------------------------------------------------

fn stream_key(tenant_id: &str) -> String {
    format!("intelligence-app-receipts:{tenant_id}")
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors raised during event emission. Non-fatal per D6 contract.
#[derive(Debug)]
pub enum ValkeySinkError {
    /// Redis/Valkey connection error.
    Connection(String),
    /// XADD command failed.
    Xadd(String),
}

impl fmt::Display for ValkeySinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValkeySinkError::Connection(msg) => write!(f, "valkey connection error: {msg}"),
            ValkeySinkError::Xadd(msg) => write!(f, "valkey XADD error: {msg}"),
        }
    }
}

// ---------------------------------------------------------------------------
// ValkeyEventSink
// ---------------------------------------------------------------------------

/// [`EventSink`] backed by a Valkey Stream via XADD.
///
/// Wraps a `redis::Connection` behind a `Mutex` so the sync trait method can
/// issue commands without an async runtime. The mutex is uncontended in the
/// common case (one emitter per service instance).
pub struct ValkeyEventSink {
    conn: Mutex<redis::Connection>, // data_class: INTERNAL_ONLY
}

impl fmt::Debug for ValkeyEventSink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValkeyEventSink").finish_non_exhaustive()
    }
}

impl ValkeyEventSink {
    /// Connect to Valkey at `url` (e.g. `redis://valkey.svc:6379` or
    /// `rediss://valkey.svc:6380` for TLS).
    ///
    /// Returns an error if the initial connection fails so the composition
    /// root can surface it at start-up.
    pub fn connect(url: &str) -> Result<Self, ValkeySinkError> {
        let client =
            redis::Client::open(url).map_err(|e| ValkeySinkError::Connection(e.to_string()))?;
        let conn = client
            .get_connection()
            .map_err(|e| ValkeySinkError::Connection(e.to_string()))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Emit one event, returning a typed error for test assertions.
    fn try_emit(&self, event: &LlmGatewayEvent) -> Result<(), ValkeySinkError> {
        let key = stream_key(event.tenant_id.as_str());

        let fields: Vec<(&str, String)> = vec![
            ("request_id", event.request_id.clone()),
            ("tenant_id", event.tenant_id.as_str().to_string()),
            ("agent_id", event.agent_id.as_str().to_string()),
            ("seat_id", event.seat_id.as_str().to_string()),
            ("provider", event.provider.to_string()),
            ("model", event.model.clone()),
            ("prompt_tokens", event.prompt_tokens.to_string()),
            ("completion_tokens", event.completion_tokens.to_string()),
            ("ms_latency", event.ms_latency.to_string()),
            ("status", format!("{:?}", event.status)),
            ("timestamp_unix_ms", event.timestamp_unix_ms.to_string()),
        ];

        let mut conn = self
            .conn
            .lock()
            .map_err(|_| ValkeySinkError::Connection("mutex poisoned".to_string()))?;

        // XADD <key> * <field> <value> ...
        // The auto-generated stream ID "*" is what we want for append-only receipts.
        let _: redis::Value = conn
            .xadd(&key, "*", &fields)
            .map_err(|e| ValkeySinkError::Xadd(e.to_string()))?;

        Ok(())
    }
}

impl EventSink for ValkeyEventSink {
    /// Emit one event to Valkey Stream. Errors are logged via `tracing::warn`
    /// and swallowed per the D6 non-fatal contract.
    fn emit(&self, event: LlmGatewayEvent) {
        if let Err(e) = self.try_emit(&event) {
            warn!(
                request_id = %event.request_id,
                error = %e,
                "ValkeyEventSink: emit failed (non-fatal)"
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
    fn stream_key_format() {
        assert_eq!(stream_key("tenant-a"), "intelligence-app-receipts:tenant-a");
    }

    #[test]
    fn connect_fails_on_bad_url() {
        let result = ValkeyEventSink::connect("not-a-url");
        assert!(
            matches!(result, Err(ValkeySinkError::Connection(_))),
            "expected Connection error for bad URL, got: {result:?}"
        );
    }

    #[test]
    fn connect_fails_on_unreachable_host() {
        // Port 1 is reserved and never has a listener; this exercises the
        // connection-refused path without requiring a live Valkey instance.
        let result = ValkeyEventSink::connect("redis://127.0.0.1:1");
        assert!(
            matches!(result, Err(ValkeySinkError::Connection(_))),
            "expected Connection error for unreachable host, got: {result:?}"
        );
    }

    #[test]
    fn error_display_connection() {
        let e = ValkeySinkError::Connection("timeout".to_string());
        assert!(e.to_string().contains("valkey connection error"));
    }

    #[test]
    fn error_display_xadd() {
        let e = ValkeySinkError::Xadd("WRONGTYPE".to_string());
        assert!(e.to_string().contains("valkey XADD error"));
    }

    #[test]
    fn event_status_variants_all_have_debug_repr() {
        // Ensures the format!("{:?}", status) call in try_emit works for all variants.
        for status in [
            EventStatus::Ok,
            EventStatus::UpstreamError,
            EventStatus::RateLimited,
            EventStatus::Forbidden,
            EventStatus::PoolExhausted,
        ] {
            let mut ev = test_event();
            ev.status = status;
            // We can't call try_emit without a live Valkey, but we can verify
            // the fields build without panic by exercising stream_key + field list.
            let key = stream_key(ev.tenant_id.as_str());
            assert!(!key.is_empty());
            let status_str = format!("{:?}", ev.status);
            assert!(!status_str.is_empty());
        }
    }
}
