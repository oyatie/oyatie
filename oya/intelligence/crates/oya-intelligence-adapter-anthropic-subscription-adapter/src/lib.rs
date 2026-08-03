//! M02-P01 — Anthropic subscription-auth OAuth runtime adapter.
//!
//! Replaces the previous in-memory mock with a live Anthropic OAuth adapter:
//!
//! - POST oauth/token `grant_type=refresh_token` via hyper/hyper-rustls (ADR-0090/ADR-0506).
//! - Per-seat SINGLEFLIGHT refresh lock (coalesces concurrent refreshes).
//! - Persist new token to `CredentialStorePort` BEFORE mutating in-memory state.
//! - `RefreshPolicy::ExpiresLead` background ticker (BinaryHeap min-heap of next_due).
//! - Terminal-vs-transient error classification; `OperatorAlertPort` on terminal.
//! - `Authorization: Bearer` (not x-api-key) + `anthropic-version` + `anthropic-beta` on outbound.
//! - PKCE enrollment path via `oya-intelligence-oauth-subscription-kernel`.
//!
//! No raw secrets appear in any `Debug` output or tracing span.
//!
//! ADR-0043: tokens are NEVER stored in raw form — only through `CredentialStorePort`.
//! ADR-0083: panic-free in production; tests may use `unwrap`/`expect`/`panic`.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tracing::{debug, error, warn};

// ── Submodules ───────────────────────────────────────────────────────────────

pub mod enrollment;
pub mod inmemory_store;
pub mod oauth_client;
pub mod ports;
pub mod refresh_policy;
pub mod singleflight;
pub mod token_state;

// ── Re-exports ───────────────────────────────────────────────────────────────

pub use enrollment::{EnrollmentError, build_enrollment_flow, complete_enrollment, parse_callback};
pub use inmemory_store::{InMemoryAlertPort, InMemoryCredentialStore};
pub use oauth_client::{
    ANTHROPIC_BETA, ANTHROPIC_VERSION, OAuthClientError, OAuthTokenClient, build_https_client,
    build_loopback_http_or_https_test_client, outbound_auth_headers,
};
pub use ports::{AlertKind, CredentialStorePort, OperatorAlertPort, SeatId, TokenBytes};
pub use refresh_policy::{RefreshEntry, RefreshScheduler};
pub use singleflight::RefreshSingleflight;
pub use token_state::{
    EXPIRES_LEAD_SECS, RefreshFailureKind, SeatTokenState, TERMINAL_BACKOFF_SECS,
    classify_oauth_error,
};

use intelligence_account_kernel::{
    AuthError, AuthMode, AuthToken, ProviderAuthPort, ProviderFamily,
};
use intelligence_account_domain::SecretReference;

// ── AnthropicOAuthAdapter ────────────────────────────────────────────────────

/// Live Anthropic OAuth runtime adapter.
///
/// Holds a per-seat token map (guarded by `Mutex`), a shared `OAuthTokenClient`,
/// a singleflight coalescer, and pluggable credential-store and alert ports.
///
/// The adapter satisfies `ProviderAuthPort` synchronously (the existing port
/// contract is sync). `authenticate()` returns a cached token if still valid,
/// or triggers a synchronous refresh via `tokio::task::block_in_place` if in a
/// tokio context, or returns `AuthError::NetworkUnavailable` if no runtime is
/// available (cold-start path before background ticker has run).
pub struct AnthropicOAuthAdapter {
    /// Per-seat token state, keyed by the SecretReference URI string.
    // data_class: INTERNAL_ONLY
    seats: Arc<Mutex<HashMap<String, SeatTokenState>>>,
    http_client: Arc<oauth_client::HyperHttpsClient>,
    oauth_client: Arc<OAuthTokenClient>,
    singleflight: Arc<RefreshSingleflight>,
    store: Arc<dyn CredentialStorePort>,
    alert: Arc<dyn OperatorAlertPort>,
    /// Injected clock for tests (None = use system time).
    clock_fn: Option<Arc<dyn Fn() -> u64 + Send + Sync>>,
}
struct RefreshExecution {
    seat_id: SeatId,
    current_refresh_token: String,
    now_secs: u64,
    seats: Arc<Mutex<HashMap<String, SeatTokenState>>>,
    oauth_client: Arc<OAuthTokenClient>,
    singleflight: Arc<RefreshSingleflight>,
    store: Arc<dyn CredentialStorePort>,
    alert: Arc<dyn OperatorAlertPort>,
}

fn is_loopback_http_endpoint(endpoint: &str) -> bool {
    endpoint.starts_with("http://127.0.0.1:")
        || endpoint.starts_with("http://localhost:")
        || endpoint.starts_with("http://[::1]:")
}

impl AnthropicOAuthAdapter {
    /// Construct with production HTTPS client.
    pub fn new(store: Arc<dyn CredentialStorePort>, alert: Arc<dyn OperatorAlertPort>) -> Self {
        let http_client = Arc::new(build_https_client());
        let oauth_client = Arc::new(OAuthTokenClient::new(Arc::clone(&http_client)));
        Self {
            seats: Arc::new(Mutex::new(HashMap::new())),
            http_client,
            oauth_client,
            singleflight: Arc::new(RefreshSingleflight::new()),
            store,
            alert,
            clock_fn: None,
        }
    }

    /// Construct with custom OAuth client endpoint (used in tests against local mock server).
    pub fn with_token_endpoint(
        store: Arc<dyn CredentialStorePort>,
        alert: Arc<dyn OperatorAlertPort>,
        token_endpoint: impl Into<String>,
    ) -> Self {
        let token_endpoint = token_endpoint.into();
        let http_client = Arc::new(if is_loopback_http_endpoint(&token_endpoint) {
            build_loopback_http_or_https_test_client()
        } else {
            build_https_client()
        });
        let oauth_client = Arc::new(
            OAuthTokenClient::new(Arc::clone(&http_client)).with_token_endpoint(token_endpoint),
        );
        Self {
            seats: Arc::new(Mutex::new(HashMap::new())),
            http_client,
            oauth_client,
            singleflight: Arc::new(RefreshSingleflight::new()),
            store,
            alert,
            clock_fn: None,
        }
    }

    /// Inject a clock override (tests only).
    pub fn with_clock(mut self, clock: impl Fn() -> u64 + Send + Sync + 'static) -> Self {
        self.clock_fn = Some(Arc::new(clock));
        self
    }

    /// Seed an already-enrolled seat directly (tests / boot-from-store path).
    pub fn seed_seat(&self, sref_uri: &str, state: SeatTokenState) {
        self.seats
            .lock()
            .unwrap()
            .insert(sref_uri.to_owned(), state);
    }

    fn now(&self) -> u64 {
        if let Some(clock) = &self.clock_fn {
            clock()
        } else {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        }
    }

    /// Perform a refresh for `seat_id`, coalesced through singleflight.
    /// Persists to credential store before mutating in-memory state.
    /// Emits operator alert on terminal errors.
    pub async fn refresh_seat_async(
        &self,
        seat_id: SeatId,
        current_refresh_token: String,
        now_secs: u64,
    ) -> Result<SeatTokenState, AuthError> {
        let client = Arc::clone(&self.oauth_client);
        let store = Arc::clone(&self.store);
        let alert = Arc::clone(&self.alert);
        let seats = Arc::clone(&self.seats);
        let seat_id_clone = seat_id.clone();

        self.singleflight
            .run(&seat_id, move || {
                let client = Arc::clone(&client);
                let store = Arc::clone(&store);
                let alert = Arc::clone(&alert);
                let seats = Arc::clone(&seats);
                let seat_id = seat_id_clone.clone();
                let current_refresh_token = current_refresh_token.clone();
                async move {
                    let current_state = SeatTokenState::new(
                        String::new(), // access_token not needed for refresh
                        current_refresh_token,
                        0,
                        0,
                    );
                    let result = client.refresh(&current_state, now_secs).await;
                    match &result {
                        Ok(new_state) => {
                            // PERSIST BEFORE MUTATE (persist-before-mutate invariant).
                            match new_state.to_storage_bytes() {
                                Ok(bytes) => {
                                    if let Err(e) = store.store(&seat_id, TokenBytes(bytes)) {
                                        warn!(seat = %seat_id, error = %e, "credential store failed; in-memory not updated");
                                        return Err(OAuthClientError::Transport(e));
                                    }
                                }
                                Err(e) => {
                                    return Err(OAuthClientError::Transport(e));
                                }
                            }
                            // Now safe to update in-memory.
                            seats
                                .lock()
                                .unwrap()
                                .insert(seat_id.0.clone(), new_state.clone());
                            debug!(seat = %seat_id, "token refreshed successfully");
                        }
                        Err(e) => {
                            if e.is_terminal() {
                                if let OAuthClientError::OAuthError { kind: RefreshFailureKind::Terminal(alert_kind), .. } = e {
                                    error!(seat = %seat_id, error = ?e, "terminal OAuth error; emitting operator alert");
                                    alert.alert(&seat_id, alert_kind.clone());
                                }
                                // Mark seat terminal in-memory.
                                let mut map = seats.lock().unwrap();
                                if let Some(state) = map.get_mut(&seat_id.0) {
                                    state.mark_terminal(now_secs);
                                }
                            } else {
                                warn!(seat = %seat_id, error = ?e, "transient OAuth error during refresh");
                            }
                        }
                    }
                    result
                }
            })
            .await
            .map_err(|e| match &e {
                OAuthClientError::OAuthError { kind: RefreshFailureKind::Terminal(_), .. } => {
                    AuthError::ProviderRejected(format!("{e:?}"))
                }
                _ => AuthError::NetworkUnavailable,
            })
    }

    /// Synchronously authenticate for the given `SecretReference`.
    /// If the seat has a valid cached token, return it immediately.
    /// If the seat needs refresh and we are in a Tokio context, block on the async refresh.
    /// Otherwise return `NetworkUnavailable`.
    fn authenticate_sync(&self, sref: &SecretReference) -> Result<AuthToken, AuthError> {
        let now = self.now();
        let key = format!("{sref:?}"); // uses Debug which is already sref:// form

        // Fast path: valid cached token.
        {
            let map = self.seats.lock().unwrap();
            if let Some(state) = map.get(&key)
                && state.is_valid_at(now)
                && !state.needs_refresh_at(now)
            {
                return make_auth_token(state, now);
            }
        }

        // Slow path: need refresh. Try to drive async refresh from sync context.
        let seat_id = SeatId(key.clone());
        let current_refresh_token = {
            let map = self.seats.lock().unwrap();
            map.get(&key)
                .map(|s| s.refresh_token.clone())
                .unwrap_or_default()
        };

        if current_refresh_token.is_empty() {
            return Err(AuthError::InvalidSecretReference);
        }

        // Drive async refresh. Use block_in_place if we are inside a multi-thread tokio runtime.
        let adapter_seats = Arc::clone(&self.seats);
        let adapter_oauth = Arc::clone(&self.oauth_client);
        let adapter_sf = Arc::clone(&self.singleflight);
        let adapter_store = Arc::clone(&self.store);
        let adapter_alert = Arc::clone(&self.alert);
        let key2 = key.clone();

        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                let result = tokio::task::block_in_place(|| {
                    handle.block_on(Self::do_refresh(RefreshExecution {
                        seat_id,
                        current_refresh_token,
                        now_secs: now,
                        seats: adapter_seats,
                        oauth_client: adapter_oauth,
                        singleflight: adapter_sf,
                        store: adapter_store,
                        alert: adapter_alert,
                    }))
                });
                match result {
                    Ok(_) => {
                        let map = self.seats.lock().unwrap();
                        if let Some(state) = map.get(&key2) {
                            return make_auth_token(state, now);
                        }
                        Err(AuthError::NetworkUnavailable)
                    }
                    Err(e) => Err(e),
                }
            }
            Err(_) => Err(AuthError::NetworkUnavailable),
        }
    }

    async fn do_refresh(execution: RefreshExecution) -> Result<SeatTokenState, AuthError> {
        let RefreshExecution {
            seat_id,
            current_refresh_token,
            now_secs,
            seats,
            oauth_client,
            singleflight,
            store,
            alert,
        } = execution;
        let client = Arc::clone(&oauth_client);
        let store2 = Arc::clone(&store);
        let alert2 = Arc::clone(&alert);
        let seats2 = Arc::clone(&seats);
        let seat_id2 = seat_id.clone();

        singleflight
            .run(&seat_id, move || {
                let client = Arc::clone(&client);
                let store = Arc::clone(&store2);
                let alert = Arc::clone(&alert2);
                let seats = Arc::clone(&seats2);
                let seat_id = seat_id2.clone();
                async move {
                    let current_state = SeatTokenState::new(
                        String::new(),
                        current_refresh_token,
                        0,
                        0,
                    );
                    let result = client.refresh(&current_state, now_secs).await;
                    match &result {
                        Ok(new_state) => {
                            match new_state.to_storage_bytes() {
                                Ok(bytes) => {
                                    if let Err(e) = store.store(&seat_id, TokenBytes(bytes)) {
                                        warn!(seat = %seat_id, error = %e, "credential store failed");
                                        return Err(OAuthClientError::Transport(e));
                                    }
                                }
                                Err(e) => return Err(OAuthClientError::Transport(e)),
                            }
                            seats.lock().unwrap().insert(seat_id.0.clone(), new_state.clone());
                        }
                        Err(e) => {
                            if e.is_terminal() {
                                if let OAuthClientError::OAuthError { kind: RefreshFailureKind::Terminal(ak), .. } = e {
                                    alert.alert(&seat_id, ak.clone());
                                }
                                let mut map = seats.lock().unwrap();
                                if let Some(state) = map.get_mut(&seat_id.0) {
                                    state.mark_terminal(now_secs);
                                }
                            }
                        }
                    }
                    result
                }
            })
            .await
            .map_err(|e| match &e {
                OAuthClientError::OAuthError { kind: RefreshFailureKind::Terminal(_), .. } => {
                    AuthError::ProviderRejected(format!("{e:?}"))
                }
                _ => AuthError::NetworkUnavailable,
            })
    }
}

fn make_auth_token(state: &SeatTokenState, _now: u64) -> Result<AuthToken, AuthError> {
    AuthToken::new(
        state.issued_at,
        state.expires_at,
        ProviderFamily::Claude,
        // token_id_redacted: use a stable hash of access_token length (never expose raw value).
        format!("anthropic-subscription-{}", state.access_token.len()),
    )
}

impl ProviderAuthPort for AnthropicOAuthAdapter {
    const PROVIDER_FAMILY: ProviderFamily = ProviderFamily::Claude;
    const AUTH_MODE: AuthMode = AuthMode::Subscription;

    fn authenticate(&self, sref: &SecretReference) -> Result<AuthToken, AuthError> {
        self.authenticate_sync(sref)
    }

    fn revoke(&self, token: &AuthToken) -> Result<(), AuthError> {
        // Revocation: mark the seat whose token_id matches as terminal.
        let tid = token.token_id_redacted();
        let mut map = self.seats.lock().unwrap();
        for state in map.values_mut() {
            let expected = format!("anthropic-subscription-{}", state.access_token.len());
            if expected == tid {
                state.mark_terminal(0);
                return Ok(());
            }
        }
        // Token not found — nothing to revoke.
        Ok(())
    }
}

// ─── Legacy mock adapter (kept for backward-compat; tests may still use it) ──

/// Legacy in-memory mock adapter. Retained to avoid breaking existing test call
/// sites. New code should use `AnthropicOAuthAdapter`.
#[derive(Default)]
pub struct AnthropicSubscriptionAdapter {
    revoked_token_ids: std::cell::RefCell<std::collections::HashSet<String>>,
    clock_epoch_secs: u64,
    token_lifetime_secs: u64,
}

impl AnthropicSubscriptionAdapter {
    pub fn new() -> Self {
        Self {
            revoked_token_ids: std::cell::RefCell::new(std::collections::HashSet::new()),
            clock_epoch_secs: 1_000_000,
            token_lifetime_secs: 3600,
        }
    }

    pub fn with_clock(mut self, now_epoch_secs: u64, lifetime_secs: u64) -> Self {
        self.clock_epoch_secs = now_epoch_secs;
        self.token_lifetime_secs = lifetime_secs;
        self
    }

    fn synthesize_token_id(&self, sref: &SecretReference) -> String {
        let dbg = format!("{sref:?}");
        format!("mock-anthropic-subscription-{}", dbg.len())
    }
}

impl ProviderAuthPort for AnthropicSubscriptionAdapter {
    const PROVIDER_FAMILY: ProviderFamily = ProviderFamily::Claude;
    const AUTH_MODE: AuthMode = AuthMode::Subscription;

    fn authenticate(&self, sref: &SecretReference) -> Result<AuthToken, AuthError> {
        let token_id = self.synthesize_token_id(sref);
        if self.revoked_token_ids.borrow().contains(&token_id) {
            return Err(AuthError::ProviderRejected(
                "token previously revoked".to_owned(),
            ));
        }
        AuthToken::new(
            self.clock_epoch_secs,
            self.clock_epoch_secs + self.token_lifetime_secs,
            ProviderFamily::Claude,
            token_id,
        )
    }

    fn revoke(&self, token: &AuthToken) -> Result<(), AuthError> {
        self.revoked_token_ids
            .borrow_mut()
            .insert(token.token_id_redacted().to_owned());
        Ok(())
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use intelligence_account_domain::{ProviderFamily, SecretReference};

    fn sref(s: &str) -> SecretReference {
        SecretReference::new(s.to_owned()).unwrap()
    }

    // ── Legacy mock tests (backward compat) ──────────────────────────────────

    #[test]
    fn adapter_authenticates_and_returns_token() {
        let a = AnthropicSubscriptionAdapter::new();
        let t = a.authenticate(&sref("sref://test-key")).unwrap();
        assert_eq!(t.provider_family(), ProviderFamily::Claude);
        assert!(t.expires_at_epoch_secs() > t.issued_at_epoch_secs());
    }

    #[test]
    fn adapter_reports_correct_mode_and_family() {
        let a = AnthropicSubscriptionAdapter::new();
        assert_eq!(a.auth_mode(), AuthMode::Subscription);
        assert_eq!(a.provider_family(), ProviderFamily::Claude);
        assert_eq!(
            <AnthropicSubscriptionAdapter as ProviderAuthPort>::AUTH_MODE,
            AuthMode::Subscription
        );
    }

    #[test]
    fn adapter_token_debug_redacted() {
        let a = AnthropicSubscriptionAdapter::new();
        let t = a.authenticate(&sref("sref://super-secret")).unwrap();
        let dbg = format!("{t:?}");
        assert!(dbg.contains("[REDACTED]"));
        assert!(!dbg.contains("super-secret"));
    }

    #[test]
    fn adapter_revoke_blocks_reuse() {
        let a = AnthropicSubscriptionAdapter::new();
        let t = a.authenticate(&sref("sref://test-key")).unwrap();
        a.revoke(&t).unwrap();
        assert!(matches!(
            a.authenticate(&sref("sref://test-key")),
            Err(AuthError::ProviderRejected(_))
        ));
    }

    #[test]
    fn adapter_clock_override_respected() {
        let a = AnthropicSubscriptionAdapter::new().with_clock(5_000, 60);
        let t = a.authenticate(&sref("sref://k")).unwrap();
        assert_eq!(t.issued_at_epoch_secs(), 5_000);
        assert_eq!(t.expires_at_epoch_secs(), 5_060);
    }

    #[test]
    fn adapter_does_not_leak_secret_in_token_id() {
        let a = AnthropicSubscriptionAdapter::new();
        let t = a
            .authenticate(&sref("sref://very-private-key-material-XYZ"))
            .unwrap();
        assert!(!t.token_id_redacted().contains("very-private"));
        assert!(!t.token_id_redacted().contains("XYZ"));
    }

    // ── Live adapter: seeded-seat path (no HTTP, no tokio runtime needed) ────

    #[test]
    fn oauth_adapter_returns_cached_token_without_network() {
        let store = Arc::new(InMemoryCredentialStore::new());
        let alert = Arc::new(InMemoryAlertPort::new());
        let now_secs = 1_000_000u64;

        let adapter = AnthropicOAuthAdapter::new(store, alert).with_clock(move || now_secs);

        // Seed a valid token (expires far in future).
        let state = SeatTokenState::new(
            "access-tok-123".into(),
            "refresh-tok-abc".into(),
            now_secs + 7200,
            now_secs,
        );
        // The seat key is derived from SecretReference Debug output.
        let sref_val = sref("sref://seat-1");
        let key = format!("{sref_val:?}");
        adapter.seed_seat(&key, state);

        let token = adapter.authenticate(&sref_val).unwrap();
        assert_eq!(token.provider_family(), ProviderFamily::Claude);
        assert_eq!(token.issued_at_epoch_secs(), now_secs);
        assert_eq!(token.expires_at_epoch_secs(), now_secs + 7200);

        let dbg = format!("{token:?}");
        assert!(dbg.contains("[REDACTED]"));
        assert!(!dbg.contains("access-tok-123"));
    }

    #[test]
    fn oauth_adapter_returns_error_for_missing_seat() {
        let store = Arc::new(InMemoryCredentialStore::new());
        let alert = Arc::new(InMemoryAlertPort::new());
        let adapter = AnthropicOAuthAdapter::new(store, alert);
        let result = adapter.authenticate(&sref("sref://unknown-seat"));
        assert!(matches!(result, Err(AuthError::InvalidSecretReference)));
    }

    #[test]
    fn oauth_adapter_revoke_marks_terminal() {
        let store = Arc::new(InMemoryCredentialStore::new());
        let alert = Arc::new(InMemoryAlertPort::new());
        let now_secs = 1_000_000u64;
        let adapter = AnthropicOAuthAdapter::new(store, alert).with_clock(move || now_secs);

        let state = SeatTokenState::new("acc".into(), "ref".into(), now_secs + 3600, now_secs);
        let sref_val = sref("sref://revoke-test");
        let key = format!("{sref_val:?}");
        adapter.seed_seat(&key, state);

        let token = adapter.authenticate(&sref_val).unwrap();
        adapter.revoke(&token).unwrap();

        // After revoke, the seat should be terminal; authenticate should fail.
        // (is_valid_at returns false when terminal_until is set)
        let map = adapter.seats.lock().unwrap();
        let state = map.get(&key).unwrap();
        assert!(state.terminal_until.is_some());
    }

    // ── Outbound header injection test ───────────────────────────────────────

    #[test]
    fn outbound_headers_bearer_not_x_api_key() {
        let hdrs = outbound_auth_headers("my-bearer-token");
        let map: std::collections::BTreeMap<_, _> = hdrs.into_iter().collect();
        assert!(map.contains_key("authorization"));
        assert!(map["authorization"].starts_with("Bearer "));
        assert!(!map.contains_key("x-api-key"));
        assert!(map.contains_key("anthropic-version"));
        assert!(map.contains_key("anthropic-beta"));
    }
}
