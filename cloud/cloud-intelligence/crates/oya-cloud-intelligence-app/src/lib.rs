//! OAuth subscription-pool app crate (ADR-0384 Path B).
//!
//! This is the composition layer for the cloud-intelligence
//! µservice. It wires:
//!
//! - [`oya_cloud_intelligence_kernel`] — pure-Rust pool + trait seams.
//! - [`oya_cloud_intelligence_authz_cedar_adapter`] — Cedar AuthzGate.
//! - [`oya_cloud_intelligence_rest`] — axum REST adapter + AnthropicAdapter.
//! - [`oya_cloud_intelligence_openbao_adapter::OpenBaoTransitStore`] — real
//!   OpenBao Transit envelope-encrypted secret store (D8).
//! - [`oya_cloud_intelligence_eventsink_clickhouse_adapter::ClickHouseEventSink`] +
//!   [`oya_cloud_intelligence_eventsink_valkey_adapter::ValkeyEventSink`] — real
//!   D6 event sinks fanned out via [`EventSinkFanout`].
//!
//! Entry-point for the binary is `src/main.rs`; `build_app` is the
//! production composition function. `build_app_for_tests` uses in-process
//! mocks and is available unconditionally for unit/integration tests.
//!
//! ADR-0083 Tier-3: no unwrap/expect/panic on the request path. Errors from
//! build_app propagate as `AppBuildError`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use oya_cloud_intelligence_authz_cedar_adapter::CedarAuthzGate;
use oya_cloud_intelligence_eventsink_clickhouse_adapter::ClickHouseEventSink;
use oya_cloud_intelligence_eventsink_valkey_adapter::ValkeyEventSink;
use oya_cloud_intelligence_kernel::{
    EventSink, LlmGatewayEvent, OAuthSubscription, Provider, SelectionStrategy, SubscriptionPool,
    TenantId,
};
use oya_cloud_intelligence_openbao_adapter::OpenBaoTransitStore;
use oya_cloud_intelligence_rest::{
    AppState, EventSinkFanout, OpenBaoSecretStore, RestAdapterError,
};
use oya_shared_olap_clickhouse_adapter::ClickHouseConfig;
use tracing::info;

// ---------------------------------------------------------------------------
// AppConfig — read from environment / caller
// ---------------------------------------------------------------------------

/// Application configuration. Populated from environment variables by `main.rs`;
/// unit tests can construct it directly.
#[derive(Clone, Debug)]
pub struct AppConfig {
    /// TCP address to bind the axum listener (e.g. `0.0.0.0:8080`).
    pub listen_addr: String, // data_class: INTERNAL_ONLY
    /// Tenant ID this gateway instance serves. Must be non-empty.
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    /// Anthropic API base URL (override for testing; default is production URL).
    pub anthropic_base_url: String, // data_class: INTERNAL_ONLY
    /// Optional comma-separated initial seat handles for bootstrapping the pool.
    /// Format: `seat_id:handle,...`
    pub initial_seats: Vec<(String, String)>, // data_class: INTERNAL_ONLY
    /// OpenBao base URL for Transit envelope-encryption (D8).
    /// e.g. `https://openbao.infra.svc:8200`
    pub openbao_url: String, // data_class: INTERNAL_ONLY
    /// OpenBao vault token. Sourced from `OYA_CLOUD_INTEL_OPENBAO_TOKEN`.
    pub openbao_token: String, // data_class: SECRET
    /// Transit key name used for envelope-encryption of refresh tokens.
    pub transit_key_name: String, // data_class: INTERNAL_ONLY
    /// ClickHouse HTTP URL for OLAP event sink (D6).
    /// e.g. `http://clickhouse.analytics.svc:8123`
    pub clickhouse_url: String, // data_class: INTERNAL_ONLY
    /// ClickHouse user.
    pub clickhouse_user: String, // data_class: INTERNAL_ONLY
    /// ClickHouse password. Sourced from `OYA_CLOUD_INTEL_CLICKHOUSE_PASSWORD`.
    pub clickhouse_password: String, // data_class: SECRET
    /// Valkey/Redis URL for stream event sink (D6).
    /// e.g. `redis://valkey.infra.svc:6379` or `rediss://...` for TLS.
    pub valkey_url: String, // data_class: INTERNAL_ONLY
}

impl AppConfig {
    /// Read config from environment variables.
    ///
    /// | Env var                              | Default                          |
    /// |--------------------------------------|----------------------------------|
    /// | `OYA_CLOUD_INTEL_LISTEN_ADDR`        | `0.0.0.0:8080`                   |
    /// | `OYA_CLOUD_INTEL_TENANT_ID`          | *(required)*                     |
    /// | `OYA_CLOUD_INTEL_ANTHROPIC_URL`      | `https://api.anthropic.com`      |
    /// | `OYA_CLOUD_INTEL_INITIAL_SEATS`      | *(empty)*                        |
    /// | `OYA_CLOUD_INTEL_OPENBAO_URL`        | *(required)*                     |
    /// | `OYA_CLOUD_INTEL_OPENBAO_TOKEN`      | *(required)*                     |
    /// | `OYA_CLOUD_INTEL_TRANSIT_KEY_NAME`   | `cloud-intelligence-rt`                 |
    /// | `OYA_CLOUD_INTEL_CLICKHOUSE_URL`     | `http://clickhouse.analytics.svc:8123` |
    /// | `OYA_CLOUD_INTEL_CLICKHOUSE_USER`    | `default`                        |
    /// | `OYA_CLOUD_INTEL_CLICKHOUSE_PASSWORD`| *(required)*                     |
    /// | `OYA_CLOUD_INTEL_VALKEY_URL`         | `redis://valkey.infra.svc:6379`  |
    pub fn from_env() -> Result<Self, AppBuildError> {
        let listen_addr = std::env::var("OYA_CLOUD_INTEL_LISTEN_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        let tenant_id = std::env::var("OYA_CLOUD_INTEL_TENANT_ID").map_err(|_| {
            AppBuildError::Config("OYA_CLOUD_INTEL_TENANT_ID is required".to_string())
        })?;
        let anthropic_base_url = std::env::var("OYA_CLOUD_INTEL_ANTHROPIC_URL")
            .unwrap_or_else(|_| "https://api.anthropic.com".to_string());
        let initial_seats = parse_initial_seats(
            &std::env::var("OYA_CLOUD_INTEL_INITIAL_SEATS").unwrap_or_default(),
        );
        let openbao_url = std::env::var("OYA_CLOUD_INTEL_OPENBAO_URL").map_err(|_| {
            AppBuildError::Config("OYA_CLOUD_INTEL_OPENBAO_URL is required".to_string())
        })?;
        let openbao_token = std::env::var("OYA_CLOUD_INTEL_OPENBAO_TOKEN").map_err(|_| {
            AppBuildError::Config("OYA_CLOUD_INTEL_OPENBAO_TOKEN is required".to_string())
        })?;
        let transit_key_name = std::env::var("OYA_CLOUD_INTEL_TRANSIT_KEY_NAME")
            .unwrap_or_else(|_| "cloud-intelligence-rt".to_string());
        let clickhouse_url = std::env::var("OYA_CLOUD_INTEL_CLICKHOUSE_URL")
            .unwrap_or_else(|_| "http://clickhouse.analytics.svc:8123".to_string());
        let clickhouse_user = std::env::var("OYA_CLOUD_INTEL_CLICKHOUSE_USER")
            .unwrap_or_else(|_| "default".to_string());
        let clickhouse_password =
            std::env::var("OYA_CLOUD_INTEL_CLICKHOUSE_PASSWORD").map_err(|_| {
                AppBuildError::Config("OYA_CLOUD_INTEL_CLICKHOUSE_PASSWORD is required".to_string())
            })?;
        let valkey_url = std::env::var("OYA_CLOUD_INTEL_VALKEY_URL")
            .unwrap_or_else(|_| "redis://valkey.infra.svc:6379".to_string());
        Ok(Self {
            listen_addr,
            tenant_id,
            anthropic_base_url,
            initial_seats,
            openbao_url,
            openbao_token,
            transit_key_name,
            clickhouse_url,
            clickhouse_user,
            clickhouse_password,
            valkey_url,
        })
    }
}

fn parse_initial_seats(raw: &str) -> Vec<(String, String)> {
    if raw.trim().is_empty() {
        return Vec::new();
    }
    raw.split(',')
        .filter_map(|entry| {
            let mut parts = entry.splitn(2, ':');
            let seat_id = parts.next()?.trim().to_string();
            let handle = parts.next()?.trim().to_string();
            if seat_id.is_empty() || handle.is_empty() {
                None
            } else {
                Some((seat_id, handle))
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// AppBuildError
// ---------------------------------------------------------------------------

/// Errors raised during `build_app`. All variants are fatal at start-up time
/// and are surfaced as non-zero exit codes in `main.rs`.
#[derive(Debug)]
pub enum AppBuildError {
    /// A required configuration value is missing or invalid.
    Config(String),
    /// Cedar policy failed to parse. This is a compile-time invariant in
    /// production (policy is bundled); reported here for explicit error
    /// surfacing during tests with custom policy text.
    CedarPolicy(String),
    /// `reqwest::Client` construction failed (platform TLS error).
    HttpClient(reqwest::Error),
    /// Kernel rejected a seat configuration at startup.
    PoolSetup(String),
}

impl std::fmt::Display for AppBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppBuildError::Config(msg) => write!(f, "config error: {msg}"),
            AppBuildError::CedarPolicy(msg) => write!(f, "cedar policy error: {msg}"),
            AppBuildError::HttpClient(e) => write!(f, "http client error: {e}"),
            AppBuildError::PoolSetup(msg) => write!(f, "pool setup error: {msg}"),
        }
    }
}

impl std::error::Error for AppBuildError {}

impl From<reqwest::Error> for AppBuildError {
    fn from(e: reqwest::Error) -> Self {
        AppBuildError::HttpClient(e)
    }
}

// ---------------------------------------------------------------------------
// EventSinkFanoutAdapter — thin EventSink wrapper around EventSinkFanout
// ---------------------------------------------------------------------------

/// Wraps [`EventSinkFanout`] so it can be stored as `Arc<dyn EventSink>`.
/// `EventSinkFanout` exposes `broadcast()` rather than `emit()` directly;
/// this adapter bridges the two.
struct EventSinkFanoutAdapter(EventSinkFanout);

impl EventSink for EventSinkFanoutAdapter {
    fn emit(&self, event: LlmGatewayEvent) {
        self.0.broadcast(event);
    }
}

// ---------------------------------------------------------------------------
// In-process stubs (kept for build_app_for_tests)
// ---------------------------------------------------------------------------

/// In-process stub secret store. Holds a plaintext map keyed by handle.
/// Stage-7: replace with `oya-cloud-intelligence-openbao-adapter`.
///
/// NOTE: plaintext tokens in memory are acceptable for the local-foundation
/// phase only. See ADR-0384 D8 and the Stage-7 deferral note.
pub struct InProcessSecretStore {
    map: Mutex<HashMap<String, String>>, // data_class: INTERNAL_ONLY
}

impl InProcessSecretStore {
    pub fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }

    /// Pre-load a handle → plaintext pair (used at startup when
    /// `OYA_CLOUD_INTEL_INITIAL_SEATS` is set).
    pub fn preload(&self, handle: &str, token: &str) {
        if let Ok(mut m) = self.map.lock() {
            m.insert(handle.to_string(), token.to_string());
        }
    }
}

impl Default for InProcessSecretStore {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenBaoSecretStore for InProcessSecretStore {
    fn fetch_refresh_token(&self, handle: &str) -> Result<String, RestAdapterError> {
        self.map
            .lock()
            .map_err(|_| RestAdapterError::SecretStoreUnavailable("lock poisoned".to_string()))?
            .get(handle)
            .cloned()
            .ok_or(RestAdapterError::SecretNotFound)
    }

    fn store_refresh_token(&self, handle: &str, plaintext: &str) -> Result<(), RestAdapterError> {
        if plaintext.is_empty() {
            return Err(RestAdapterError::InvalidSecret);
        }
        self.map
            .lock()
            .map_err(|_| RestAdapterError::SecretStoreUnavailable("lock poisoned".to_string()))?
            .insert(handle.to_string(), plaintext.to_string());
        Ok(())
    }
}

/// In-process event sink — logs events via `tracing`. Stage-7: replace with
/// ClickHouse OLAP adapter + Valkey Stream adapter.
pub struct InProcessEventSink;

impl EventSink for InProcessEventSink {
    fn emit(&self, event: LlmGatewayEvent) {
        info!(
            request_id = %event.request_id,
            tenant_id  = %event.tenant_id.as_str(),
            seat_id    = %event.seat_id.as_str(),
            provider   = %event.provider,
            status     = ?event.status,
            ms_latency = event.ms_latency,
            "cloud-intelligence event"
        );
    }
}

// ---------------------------------------------------------------------------
// build_app — testable composition root
// ---------------------------------------------------------------------------

/// Wire up all components with **real production adapters** and return the
/// shared [`AppState`].
///
/// Uses:
/// - [`OpenBaoTransitStore`] for D8 envelope-encrypted secret storage.
/// - [`ClickHouseEventSink`] + [`ValkeyEventSink`] fanned out via
///   [`EventSinkFanout`] for D6 event emission.
///
/// Reads all adapter config from `AppConfig` (populated from env vars).
/// Returns [`AppBuildError`] on any fatal configuration or connection failure.
pub fn build_app(config: AppConfig) -> Result<Arc<AppState>, AppBuildError> {
    // Cedar gate (loaded from bundled policy; fail-closed on parse error).
    let gate = CedarAuthzGate::with_default_policy()
        .map_err(|e| AppBuildError::CedarPolicy(e.to_string()))?;

    // Tenant ID validation.
    let tenant_id = TenantId::new(&config.tenant_id)
        .map_err(|_| AppBuildError::Config(format!("invalid tenant_id: {:?}", config.tenant_id)))?;

    // Build shared reqwest::Client used by both AppState and OpenBaoTransitStore.
    let http_client = Arc::new(
        reqwest::Client::builder()
            .build()
            .map_err(AppBuildError::HttpClient)?,
    );

    // Real OpenBao Transit secret store (D8).
    let secret_store: Arc<dyn OpenBaoSecretStore> = Arc::new(OpenBaoTransitStore::new(
        Arc::clone(&http_client),
        &config.openbao_url,
        &config.transit_key_name,
        &config.openbao_token,
    ));

    // Real D6 event sinks — ClickHouse + Valkey fanned out.
    let ch_sink = ClickHouseEventSink::new(ClickHouseConfig {
        url: config.clickhouse_url.clone(),
        user: config.clickhouse_user.clone(),
        password: config.clickhouse_password.clone(),
    });

    let valkey_sink = ValkeyEventSink::connect(&config.valkey_url)
        .map_err(|e| AppBuildError::Config(format!("valkey connect failed: {e}")))?;

    let mut fanout = EventSinkFanout::new();
    fanout.add_sink(Box::new(ch_sink));
    fanout.add_sink(Box::new(valkey_sink));
    let sink: Arc<dyn EventSink + Send + Sync> = Arc::new(EventSinkFanoutAdapter(fanout));

    // Subscription pool (one pool per tenant-provider pair; v1 = Anthropic only).
    let pool_arc = build_pool(tenant_id.clone(), &config.initial_seats)?;

    // Build AppState (uses the shared reqwest::Client for upstream proxy calls).
    let state = AppState::new(
        pool_arc,
        Arc::new(gate),
        sink,
        secret_store,
        config.anthropic_base_url,
        tenant_id,
    )
    .map_err(AppBuildError::HttpClient)?;

    Ok(Arc::new(state))
}

/// Wire up all components with **in-process mocks** for unit and integration
/// tests. This constructor is always available (not gated behind a feature flag)
/// so the test suite never depends on live OpenBao / ClickHouse / Valkey.
pub fn build_app_for_tests(config: AppConfig) -> Result<Arc<AppState>, AppBuildError> {
    // Cedar gate.
    let gate = CedarAuthzGate::with_default_policy()
        .map_err(|e| AppBuildError::CedarPolicy(e.to_string()))?;

    // Tenant ID validation.
    let tenant_id = TenantId::new(&config.tenant_id)
        .map_err(|_| AppBuildError::Config(format!("invalid tenant_id: {:?}", config.tenant_id)))?;

    // In-process secret store.
    let secret_store = Arc::new(InProcessSecretStore::new());
    for (_, handle) in &config.initial_seats {
        secret_store.preload(handle, "test-placeholder-token");
    }

    // In-process event sink.
    let sink: Arc<dyn EventSink + Send + Sync> = Arc::new(InProcessEventSink);

    // Subscription pool.
    let pool_arc = build_pool(tenant_id.clone(), &config.initial_seats)?;

    let state = AppState::new(
        pool_arc,
        Arc::new(gate),
        sink,
        secret_store,
        config.anthropic_base_url,
        tenant_id,
    )
    .map_err(AppBuildError::HttpClient)?;

    Ok(Arc::new(state))
}

/// Shared pool construction helper used by both `build_app` and
/// `build_app_for_tests`.
fn build_pool(
    tenant_id: TenantId,
    initial_seats: &[(String, String)],
) -> Result<Arc<Mutex<SubscriptionPool>>, AppBuildError> {
    let mut pool = SubscriptionPool::new(
        tenant_id.clone(),
        Provider::Anthropic,
        SelectionStrategy::RoundRobin,
    );
    for (seat_str, handle) in initial_seats {
        use oya_cloud_intelligence_kernel::{SeatId, SubscriptionId, SubscriptionState};
        let seat_id = SeatId::new(seat_str.as_str()).map_err(|_| {
            AppBuildError::PoolSetup(format!("invalid seat_id in initial_seats: {seat_str}"))
        })?;
        let sub_id = SubscriptionId::new(format!("{seat_str}-sub"))
            .map_err(|_| AppBuildError::PoolSetup(format!("invalid sub_id for: {seat_str}")))?;
        let sub = OAuthSubscription::new(
            tenant_id.clone(),
            seat_id,
            sub_id,
            Provider::Anthropic,
            SubscriptionState::Active,
            handle.clone(),
            0,
        );
        pool.add_seat(sub)
            .map_err(|e| AppBuildError::PoolSetup(format!("add_seat failed: {e:?}")))?;
    }
    Ok(Arc::new(Mutex::new(pool)))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> AppConfig {
        AppConfig {
            listen_addr: "127.0.0.1:0".to_string(),
            tenant_id: "test-tenant".to_string(),
            anthropic_base_url: "http://127.0.0.1:1".to_string(),
            initial_seats: vec![],
            // These fields are not used by build_app_for_tests (in-process mocks).
            openbao_url: "http://127.0.0.1:1".to_string(),
            openbao_token: "test-token".to_string(),
            transit_key_name: "cloud-intelligence-rt".to_string(),
            clickhouse_url: "http://127.0.0.1:1".to_string(),
            clickhouse_user: "default".to_string(),
            clickhouse_password: "test".to_string(),
            valkey_url: "redis://127.0.0.1:1".to_string(),
        }
    }

    #[test]
    fn build_app_for_tests_returns_state_for_valid_config() {
        let config = test_config();
        let state = build_app_for_tests(config).unwrap();
        // Pool exists and has 0 seats (no initial_seats).
        let pool = state.pool.lock().unwrap();
        assert_eq!(pool.seat_count(), 0);
    }

    #[test]
    fn build_app_for_tests_registers_initial_seats() {
        let mut config = test_config();
        config.initial_seats = vec![
            ("seat-a".to_string(), "handle-a".to_string()),
            ("seat-b".to_string(), "handle-b".to_string()),
        ];
        let state = build_app_for_tests(config).unwrap();
        let pool = state.pool.lock().unwrap();
        assert_eq!(pool.seat_count(), 2);
    }

    #[test]
    fn build_app_for_tests_fails_on_empty_tenant_id() {
        let config = AppConfig {
            listen_addr: "127.0.0.1:0".to_string(),
            tenant_id: "".to_string(),
            anthropic_base_url: "http://127.0.0.1:1".to_string(),
            initial_seats: vec![],
            openbao_url: "http://127.0.0.1:1".to_string(),
            openbao_token: "test-token".to_string(),
            transit_key_name: "cloud-intelligence-rt".to_string(),
            clickhouse_url: "http://127.0.0.1:1".to_string(),
            clickhouse_user: "default".to_string(),
            clickhouse_password: "test".to_string(),
            valkey_url: "redis://127.0.0.1:1".to_string(),
        };
        match build_app_for_tests(config) {
            Err(err) => assert!(
                matches!(err, AppBuildError::Config(_)),
                "expected Config error, got: {err}"
            ),
            Ok(_) => panic!("expected error for empty tenant_id but got Ok"),
        }
    }

    #[test]
    fn parse_initial_seats_parses_correctly() {
        let seats = parse_initial_seats("seat-a:handle-a,seat-b:handle-b");
        assert_eq!(seats.len(), 2);
        assert_eq!(seats[0], ("seat-a".to_string(), "handle-a".to_string()));
        assert_eq!(seats[1], ("seat-b".to_string(), "handle-b".to_string()));
    }

    #[test]
    fn parse_initial_seats_empty_string_returns_empty() {
        assert!(parse_initial_seats("").is_empty());
        assert!(parse_initial_seats("   ").is_empty());
    }

    #[test]
    fn in_process_secret_store_roundtrips() {
        let store = InProcessSecretStore::new();
        store.preload("h1", "rt-1");
        let fetched = store.fetch_refresh_token("h1").unwrap();
        assert_eq!(fetched, "rt-1");
    }

    #[test]
    fn in_process_secret_store_not_found() {
        let store = InProcessSecretStore::new();
        let err = store.fetch_refresh_token("missing").unwrap_err();
        assert_eq!(err, RestAdapterError::SecretNotFound);
    }

    #[test]
    fn in_process_secret_store_rejects_empty_plaintext() {
        let store = InProcessSecretStore::new();
        let err = store.store_refresh_token("h1", "").unwrap_err();
        assert_eq!(err, RestAdapterError::InvalidSecret);
    }
}
