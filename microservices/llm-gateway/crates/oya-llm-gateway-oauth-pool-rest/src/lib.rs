//! llm-gateway REST adapter — OAuth subscription pool (ADR-0384 Path B).
//!
//! Stage-4 RED scaffold. Public API surface + trait contracts are defined here;
//! the GREEN implementation lands in a follow-up PR.
//!
//! Crate owns:
//! - [`OpenBaoSecretStore`] trait — D8 envelope-encrypted refresh-token storage
//!   seam. Real OpenBao adapter ships in a separate crate.
//! - [`EventSinkFanout`] — D6 fan-out broadcaster wrapping one or more
//!   [`oya_llm_gateway_oauth_pool_kernel::EventSink`] impls.
//! - [`AnthropicAdapter`] — D3 Anthropic provider adapter + OAuth refresh
//!   state machine. Bodies are `todo!()` until Stage-5 GREEN.
//! - [`ProxyRequest`] / [`ProxyResponse`] — D2 axum reverse-proxy wire types.
//!   The actual axum router is wired in Stage-5 GREEN.
//!
//! TODO(codex-adapter): add `CodexAdapter` mirroring `AnthropicAdapter` once
//! the Codex OAuth refresh flow is documented. Tracked as a separate follow-up
//! PR per ADR-0384 §v1-scope.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

pub use oya_llm_gateway_oauth_pool_kernel::{
    EventSink, LlmGatewayEvent, Provider, SeatId, SubscriptionPool, SubscriptionPoolError, TenantId,
};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Unified error type for the REST adapter layer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RestAdapterError {
    /// Secret handle was not found in the store.
    SecretNotFound,
    /// Secret store returned a transport or vault error.
    SecretStoreUnavailable(String),
    /// Supplied plaintext was empty or otherwise invalid.
    InvalidSecret,
    /// Upstream Anthropic API returned an error status.
    UpstreamError { status: u16, body: String },
    /// OAuth refresh token exchange failed.
    OAuthRefreshFailed(String),
    /// Kernel pool returned an error.
    PoolError(SubscriptionPoolError),
    /// Fan-out sink emit failed (logged; non-fatal in proxy path).
    SinkEmitFailed(String),
}

impl From<SubscriptionPoolError> for RestAdapterError {
    fn from(e: SubscriptionPoolError) -> Self {
        RestAdapterError::PoolError(e)
    }
}

// ---------------------------------------------------------------------------
// D8 — OpenBao envelope-encrypted refresh-token storage seam
// ---------------------------------------------------------------------------

/// D8 secret-store seam. Implementors envelope-encrypt/decrypt via OpenBao
/// transit secrets engine. The kernel never sees plaintext tokens; only opaque
/// handles cross the kernel boundary.
///
/// Real implementation ships in `oya-llm-gateway-openbao-adapter` (separate
/// crate, follow-up PR).
pub trait OpenBaoSecretStore {
    /// Fetch and decrypt the refresh token identified by `handle`.
    fn fetch_refresh_token(&self, handle: &str) -> Result<String, RestAdapterError>;

    /// Envelope-encrypt `plaintext` and store it under `handle`.
    fn store_refresh_token(&self, handle: &str, plaintext: &str) -> Result<(), RestAdapterError>;
}

// ---------------------------------------------------------------------------
// D6 — EventSink fan-out
// ---------------------------------------------------------------------------

/// D6 fan-out broadcaster. Holds a list of [`EventSink`] impls and emits each
/// [`LlmGatewayEvent`] to every registered sink in registration order.
///
/// Sink failures are recorded in `last_errors` (non-fatal): the proxy path
/// MUST NOT fail a request because a sink is unavailable.
pub struct EventSinkFanout {
    sinks: Vec<Box<dyn EventSink + Send + Sync>>,
}

impl EventSinkFanout {
    /// Construct a fanout with no initial sinks.
    pub fn new() -> Self {
        Self { sinks: Vec::new() }
    }

    /// Register an additional sink.
    pub fn add_sink(&mut self, sink: Box<dyn EventSink + Send + Sync>) {
        self.sinks.push(sink);
    }

    /// Broadcast `event` to every registered sink. Returns the count of sinks
    /// that received the event.
    pub fn broadcast(&self, event: LlmGatewayEvent) -> usize {
        let mut delivered = 0usize;
        for sink in &self.sinks {
            sink.emit(event.clone());
            delivered += 1;
        }
        delivered
    }

    /// Number of registered sinks.
    pub fn sink_count(&self) -> usize {
        self.sinks.len()
    }
}

impl Default for EventSinkFanout {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// D2 — proxy wire types
// ---------------------------------------------------------------------------

/// Inbound HTTP request forwarded through the axum reverse proxy (D2).
///
/// Bodies are opaque byte vectors; header maps use `BTreeMap` for
/// deterministic ordering in tests. The full axum extractor wiring is
/// implemented in Stage-5 GREEN.
#[derive(Clone, Debug)]
pub struct ProxyRequest {
    pub method: String,                    // data_class: INTERNAL_ONLY
    pub path: String,                      // data_class: INTERNAL_ONLY
    pub headers: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
    pub body: Vec<u8>,                     // data_class: INTERNAL_ONLY
    pub tenant_id: TenantId,               // data_class: INTERNAL_ONLY
}

/// Outbound HTTP response returned by the axum reverse proxy (D2).
#[derive(Clone, Debug)]
pub struct ProxyResponse {
    pub status: u16,                       // data_class: INTERNAL_ONLY
    pub headers: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
    pub body: Vec<u8>,                     // data_class: INTERNAL_ONLY
}

// ---------------------------------------------------------------------------
// D3 — Anthropic provider adapter
// ---------------------------------------------------------------------------

/// D3 Anthropic provider adapter. Responsible for:
/// 1. Performing the initial OAuth authorization code exchange.
/// 2. Refreshing tokens before expiry (or on 401) via the Anthropic OAuth
///    endpoint.
/// 3. Routing proxied requests to `https://api.anthropic.com` with the
///    bearer token fetched from [`OpenBaoSecretStore`].
///
/// Method bodies are `todo!()` in Stage-4 RED. Stage-5 GREEN wires reqwest.
///
/// TODO(codex-adapter): `CodexAdapter` will mirror this struct for the OpenAI
/// Codex OAuth flow. Deferred to a follow-up PR per ADR-0384 §v1-scope.
pub struct AnthropicAdapter<S: OpenBaoSecretStore> {
    secret_store: S,
}

impl<S: OpenBaoSecretStore> AnthropicAdapter<S> {
    /// Construct with a concrete [`OpenBaoSecretStore`] implementation.
    pub fn new(secret_store: S) -> Self {
        Self { secret_store }
    }

    /// Exchange an OAuth authorization code for access + refresh tokens, then
    /// store the refresh token via the secret store and return the opaque
    /// handle.
    pub fn exchange_authorization_code(
        &self,
        _tenant_id: &TenantId,
        _seat_id: &SeatId,
        _authorization_code: &str,
    ) -> Result<String, RestAdapterError> {
        todo!("Stage-5 GREEN: exchange authorization code with Anthropic OAuth endpoint")
    }

    /// Refresh the token identified by `refresh_token_handle`. Fetches the
    /// current refresh token from the secret store, exchanges it with
    /// Anthropic, stores the new refresh token, and returns the bearer access
    /// token to use for this request.
    pub fn refresh_token(&self, _refresh_token_handle: &str) -> Result<String, RestAdapterError> {
        todo!("Stage-5 GREEN: POST to Anthropic token endpoint, rotate refresh token in OpenBao")
    }

    /// Forward `request` to the Anthropic API using the bearer token obtained
    /// (or refreshed) for the selected seat. Returns the upstream response.
    pub fn proxy(
        &self,
        _request: &ProxyRequest,
        _refresh_token_handle: &str,
    ) -> Result<ProxyResponse, RestAdapterError> {
        todo!("Stage-5 GREEN: reqwest proxy to api.anthropic.com with OAuth bearer token")
    }
}
