//! Deterministic D3 upstream OAuth keyed-flight lifecycle tests.
//!
//! Ported off `httpmock` onto the first-party `scripted-http-server` (ADR-0709 D-6
//! Rule 2). This file is the one place where the httpmock usage was genuinely
//! MATCHER-shaped rather than positional — every mock selected on
//! `body_contains(r#""refresh_token":"X""#)` — so it ports onto content routing
//! (`ScriptedServer::start_with`) rather than onto a positional script.
//!
//! `OAuthUpstream` below models what those matchers were really describing: an OAuth
//! token endpoint whose refresh tokens are SINGLE USE. A refresh token is exchanged by
//! being in the rotation table; `revoke` takes it out, after which presenting it again
//! is answered `400 single-use refresh token already consumed` — which is precisely what
//! the `rejected_replay` mocks in the original did.
//!
//! The negative assertions get STRICTLY STRONGER in the port. `rejected_replay.assert_hits(0)`
//! only said "httpmock never SELECTED that mock", which is silent if matcher precedence
//! ever sent the request elsewhere. `assert_no_replay_since` reads the recorded bodies and
//! asserts no request carried the retired token at all — the property the test is named
//! for. Likewise `mock.assert_hits(1)` becomes a count of the requests that actually
//! carried that specific refresh token.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::Duration;

use intelligence_rest::{
    AnthropicAdapter, RestAdapterError, SecretProviderFuture, SecretProviderStore,
    UpstreamOAuthSingleflight,
};
use scripted_http_server::{RecordedRequest, ScriptedResponse, ScriptedServer};
use tokio::sync::Semaphore;

type RefreshFuture =
    Pin<Box<dyn Future<Output = Result<String, RestAdapterError>> + Send + 'static>>;

#[derive(Default)]
struct StoreState {
    tokens: HashMap<String, String>,
    fetch_attempts: HashMap<String, usize>,
    store_attempts: HashMap<String, usize>,
    fetch_panics_remaining: usize,
    store_panics_remaining: usize,
    store_hangs_remaining: usize,
    transient_store_failure: Option<RestAdapterError>,
    fetch_error: Option<RestAdapterError>,
    store_error: Option<RestAdapterError>,
}

#[derive(Clone)]
struct RecordingStore {
    state: Arc<Mutex<StoreState>>,
    fetch_started: Option<Arc<Semaphore>>,
    fetch_release: Option<Arc<Semaphore>>,
}

impl RecordingStore {
    fn new(tokens: impl IntoIterator<Item = (&'static str, &'static str)>) -> Self {
        Self {
            state: Arc::new(Mutex::new(StoreState {
                tokens: tokens
                    .into_iter()
                    .map(|(handle, token)| (handle.to_string(), token.to_string()))
                    .collect(),
                ..StoreState::default()
            })),
            fetch_started: None,
            fetch_release: None,
        }
    }

    fn with_fetch_gate(
        tokens: impl IntoIterator<Item = (&'static str, &'static str)>,
    ) -> (Self, Arc<Semaphore>, Arc<Semaphore>) {
        let started = Arc::new(Semaphore::new(0));
        let release = Arc::new(Semaphore::new(0));
        let mut store = Self::new(tokens);
        store.fetch_started = Some(Arc::clone(&started));
        store.fetch_release = Some(Arc::clone(&release));
        (store, started, release)
    }

    fn fetch_attempts(&self, handle: &str) -> usize {
        self.state
            .lock()
            .unwrap()
            .fetch_attempts
            .get(handle)
            .copied()
            .unwrap_or_default()
    }

    fn store_attempts(&self, handle: &str) -> usize {
        self.state
            .lock()
            .unwrap()
            .store_attempts
            .get(handle)
            .copied()
            .unwrap_or_default()
    }

    fn token(&self, handle: &str) -> String {
        self.state.lock().unwrap().tokens[handle].clone()
    }

    fn set_fetch_error(&self, error: Option<RestAdapterError>) {
        self.state.lock().unwrap().fetch_error = error;
    }

    fn panic_next_fetch(&self) {
        self.state.lock().unwrap().fetch_panics_remaining += 1;
    }

    fn fail_next_store(&self, error: RestAdapterError) {
        self.state.lock().unwrap().transient_store_failure = Some(error);
    }

    fn panic_next_store(&self) {
        self.state.lock().unwrap().store_panics_remaining += 1;
    }

    fn hang_next_store(&self) {
        self.state.lock().unwrap().store_hangs_remaining += 1;
    }

    fn set_store_error(&self, error: Option<RestAdapterError>) {
        self.state.lock().unwrap().store_error = error;
    }
}

impl SecretProviderStore for RecordingStore {
    fn fetch_refresh_token<'a>(&'a self, handle: &'a str) -> SecretProviderFuture<'a, String> {
        Box::pin(async move {
            if let Some(started) = &self.fetch_started {
                started.add_permits(1);
            }
            if let Some(release) = &self.fetch_release {
                release.acquire().await.unwrap().forget();
            }

            let mut state = self.state.lock().unwrap();
            *state.fetch_attempts.entry(handle.to_string()).or_default() += 1;
            if state.fetch_panics_remaining > 0 {
                state.fetch_panics_remaining -= 1;
                drop(state);
                panic!("injected refresh-token fetch panic");
            }
            if let Some(error) = &state.fetch_error {
                return Err(error.clone());
            }
            state
                .tokens
                .get(handle)
                .cloned()
                .ok_or(RestAdapterError::SecretNotFound)
        })
    }

    fn store_refresh_token<'a>(
        &'a self,
        handle: &'a str,
        plaintext: &'a str,
    ) -> SecretProviderFuture<'a, ()> {
        Box::pin(async move {
            let should_hang = {
                let mut state = self.state.lock().unwrap();
                *state.store_attempts.entry(handle.to_string()).or_default() += 1;
                if state.store_panics_remaining > 0 {
                    state.store_panics_remaining -= 1;
                    drop(state);
                    panic!("injected refresh-token store panic");
                }
                if state.store_hangs_remaining > 0 {
                    state.store_hangs_remaining -= 1;
                    true
                } else {
                    if let Some(error) = state.transient_store_failure.take() {
                        return Err(error);
                    }
                    if let Some(error) = &state.store_error {
                        return Err(error.clone());
                    }
                    state
                        .tokens
                        .insert(handle.to_string(), plaintext.to_string());
                    false
                }
            };
            if should_hang {
                std::future::pending::<()>().await;
            }
            Ok(())
        })
    }
}

/// An OAuth token endpoint with SINGLE-USE refresh tokens.
///
/// `rotate(from, access, to)` is the port of a `success_mock`: presenting `from`
/// exchanges it for `access` and rotates it to `to`. `revoke(from)` is the port of
/// deleting that mock and installing a `rejected_replay` in its place: `from` is no
/// longer exchangeable, so presenting it is answered 400 exactly as a real provider
/// answers a replayed single-use token.
#[derive(Clone, Default)]
struct OAuthUpstream {
    rotations: Arc<Mutex<HashMap<String, (String, String)>>>,
    forced_failure: Arc<Mutex<Option<(u16, String)>>>,
}

impl OAuthUpstream {
    fn new() -> Self {
        Self::default()
    }

    fn rotate(&self, current_refresh: &str, access_token: &str, rotated_refresh: &str) -> &Self {
        self.rotations.lock().unwrap().insert(
            current_refresh.to_string(),
            (access_token.to_string(), rotated_refresh.to_string()),
        );
        self
    }

    /// Retire a refresh token: any later presentation of it is a replay and is refused.
    fn revoke(&self, current_refresh: &str) -> &Self {
        self.rotations.lock().unwrap().remove(current_refresh);
        self
    }

    /// Force every exchange to fail, whatever token is presented.
    fn fail_with(&self, status: u16, body: &str) -> &Self {
        *self.forced_failure.lock().unwrap() = Some((status, body.to_string()));
        self
    }

    fn clear_failure(&self) -> &Self {
        *self.forced_failure.lock().unwrap() = None;
        self
    }

    fn serve(&self) -> ScriptedServer {
        let upstream = self.clone();
        ScriptedServer::start_with(move |request| {
            if request.path() != "/v1/oauth/token" || request.method != "POST" {
                return ScriptedResponse::status(404).text("not the token endpoint");
            }
            if let Some((status, body)) = upstream.forced_failure.lock().unwrap().clone() {
                return ScriptedResponse::status(status).text(body);
            }
            let Some(presented) = presented_refresh_token(request) else {
                return ScriptedResponse::status(400).text("no refresh_token in request body");
            };
            match upstream.rotations.lock().unwrap().get(&presented) {
                Some((access_token, rotated_refresh)) => ScriptedResponse::ok()
                    .header("content-type", "application/json")
                    .body(format!(
                        r#"{{"access_token":"{access_token}","refresh_token":"{rotated_refresh}","expires_in":3600}}"#
                    )),
                // The port of every `rejected_replay` mock in the original.
                None => ScriptedResponse::status(400)
                    .text("single-use refresh token already consumed"),
            }
        })
    }
}

/// The `refresh_token` value an exchange request presented.
fn presented_refresh_token(request: &RecordedRequest) -> Option<String> {
    serde_json::from_slice::<serde_json::Value>(&request.body)
        .ok()?
        .get("refresh_token")?
        .as_str()
        .map(str::to_owned)
}

/// How many exchanges presented `refresh`, counting only requests recorded at or after
/// `since` (the index captured where the original deleted a mock and installed a new one).
fn exchanges_since(server: &ScriptedServer, refresh: &str, since: usize) -> usize {
    server
        .requests()
        .iter()
        .skip(since)
        .filter(|request| presented_refresh_token(request).as_deref() == Some(refresh))
        .count()
}

fn exchanges_for(server: &ScriptedServer, refresh: &str) -> usize {
    exchanges_since(server, refresh, 0)
}

/// The port of `rejected_replay.assert_hits(0)`, and strictly stronger than it: the
/// original only proved httpmock never SELECTED the replay mock, whereas this reads the
/// bodies that actually went on the wire.
fn assert_no_replay_since(server: &ScriptedServer, refresh: &str, since: usize) {
    let replays = exchanges_since(server, refresh, since);
    assert_eq!(
        replays,
        0,
        "the single-use refresh token '{refresh}' was replayed {replays} time(s) after \
         being retired; bodies seen: {:?}",
        server
            .requests()
            .iter()
            .skip(since)
            .map(presented_refresh_token)
            .collect::<Vec<_>>()
    );
}

fn adapter(
    store: RecordingStore,
    server: &ScriptedServer,
    singleflight: Arc<UpstreamOAuthSingleflight>,
) -> Arc<AnthropicAdapter<RecordingStore>> {
    Arc::new(AnthropicAdapter::with_base_url_and_singleflight(
        store,
        server.base_url().to_owned(),
        singleflight,
    ))
}

fn refresh_future(
    adapter: Arc<AnthropicAdapter<RecordingStore>>,
    client: reqwest::Client,
    handle: &'static str,
) -> RefreshFuture {
    Box::pin(async move { adapter.refresh_token(&client, handle).await })
}

fn independent_adapter_calls(
    count: usize,
    store: &RecordingStore,
    server: &ScriptedServer,
    singleflight: &Arc<UpstreamOAuthSingleflight>,
    client: &reqwest::Client,
    handle: &'static str,
) -> Vec<RefreshFuture> {
    (0..count)
        .map(|_| {
            refresh_future(
                adapter(store.clone(), server, Arc::clone(singleflight)),
                client.clone(),
                handle,
            )
        })
        .collect()
}

fn admit_all(calls: &mut [RefreshFuture]) {
    let mut context = Context::from_waker(Waker::noop());
    for call in calls {
        assert!(
            matches!(call.as_mut().poll(&mut context), Poll::Pending),
            "a newly admitted flight must wait for its helper-owned worker"
        );
    }
}

async fn complete_admitted(mut calls: Vec<RefreshFuture>) -> Vec<Result<String, RestAdapterError>> {
    admit_all(&mut calls);
    let tasks: Vec<_> = calls.into_iter().map(tokio::spawn).collect();
    tokio::time::timeout(Duration::from_secs(5), async move {
        let mut results = Vec::with_capacity(tasks.len());
        for task in tasks {
            results.push(task.await.unwrap());
        }
        results
    })
    .await
    .expect("admitted refresh flights must finish within five seconds")
}

#[tokio::test(flavor = "current_thread")]
async fn shared_flight_spans_fetch_exchange_store_and_uses_rotated_token_next() {
    const HANDLE: &str = "tenant-sf/seat-sf";
    let upstream = OAuthUpstream::new();
    upstream
        .rotate("initial-rt", "access-1", "rotated-rt-1")
        .rotate("rotated-rt-1", "access-2", "rotated-rt-2");
    let server = upstream.serve();
    let store = RecordingStore::new([(HANDLE, "initial-rt")]);
    let singleflight = Arc::new(UpstreamOAuthSingleflight::new());
    let client = reqwest::Client::new();

    let calls = independent_adapter_calls(10, &store, &server, &singleflight, &client, HANDLE);
    let results = complete_admitted(calls).await;

    assert!(
        results
            .iter()
            .all(|result| result.as_deref() == Ok("access-1"))
    );
    assert_eq!(store.fetch_attempts(HANDLE), 1);
    assert_eq!(store.store_attempts(HANDLE), 1);
    assert_eq!(store.token(HANDLE), "rotated-rt-1");
    // Ten concurrent callers must collapse to exactly ONE upstream exchange.
    assert_eq!(exchanges_for(&server, "initial-rt"), 1);
    assert_eq!(
        server.request_count(),
        1,
        "singleflight must produce exactly one upstream call in total, not just one \
         matching one: {:?}",
        server.request_lines()
    );

    let next = adapter(store.clone(), &server, Arc::clone(&singleflight))
        .refresh_token(&client, HANDLE)
        .await
        .unwrap();
    assert_eq!(next, "access-2");
    assert_eq!(store.fetch_attempts(HANDLE), 2);
    assert_eq!(store.store_attempts(HANDLE), 2);
    assert_eq!(store.token(HANDLE), "rotated-rt-2");
    assert_eq!(exchanges_for(&server, "rotated-rt-1"), 1);
    assert_eq!(server.request_count(), 2);
}

#[tokio::test(flavor = "current_thread")]
async fn different_handles_run_independent_flights() {
    const HANDLE_A: &str = "tenant/seat-a";
    const HANDLE_B: &str = "tenant/seat-b";
    let upstream = OAuthUpstream::new();
    upstream
        .rotate("refresh-a", "access-a", "rotated-a")
        .rotate("refresh-b", "access-b", "rotated-b");
    let server = upstream.serve();
    let store = RecordingStore::new([(HANDLE_A, "refresh-a"), (HANDLE_B, "refresh-b")]);
    let singleflight = Arc::new(UpstreamOAuthSingleflight::new());
    let client = reqwest::Client::new();

    let results = complete_admitted(vec![
        refresh_future(
            adapter(store.clone(), &server, Arc::clone(&singleflight)),
            client.clone(),
            HANDLE_A,
        ),
        refresh_future(
            adapter(store.clone(), &server, Arc::clone(&singleflight)),
            client.clone(),
            HANDLE_B,
        ),
    ])
    .await;

    assert_eq!(
        results,
        vec![Ok("access-a".to_string()), Ok("access-b".to_string())]
    );
    assert_eq!(store.fetch_attempts(HANDLE_A), 1);
    assert_eq!(store.fetch_attempts(HANDLE_B), 1);
    assert_eq!(store.store_attempts(HANDLE_A), 1);
    assert_eq!(store.store_attempts(HANDLE_B), 1);
    assert_eq!(exchanges_for(&server, "refresh-a"), 1);
    assert_eq!(exchanges_for(&server, "refresh-b"), 1);
    assert_eq!(server.request_count(), 2);
}

#[tokio::test(flavor = "current_thread")]
async fn exchange_failure_is_shared_without_store_and_a_later_retry_starts() {
    const HANDLE: &str = "tenant/seat-error";
    let upstream = OAuthUpstream::new();
    upstream.fail_with(503, "provider unavailable");
    let server = upstream.serve();
    let store = RecordingStore::new([(HANDLE, "refresh-error")]);
    let singleflight = Arc::new(UpstreamOAuthSingleflight::new());
    let client = reqwest::Client::new();

    let calls = independent_adapter_calls(5, &store, &server, &singleflight, &client, HANDLE);
    let results = complete_admitted(calls).await;
    let expected = RestAdapterError::OAuthRefreshFailed(
        "token refresh failed: HTTP 503: provider unavailable".to_string(),
    );

    assert!(
        results
            .iter()
            .all(|result| result == &Err(expected.clone()))
    );
    assert_eq!(store.fetch_attempts(HANDLE), 1);
    assert_eq!(store.store_attempts(HANDLE), 0);
    // Five callers, one shared failure: exactly one exchange was attempted.
    assert_eq!(exchanges_for(&server, "refresh-error"), 1);
    assert_eq!(server.request_count(), 1);
    let after_failure = server.request_count();

    upstream
        .clear_failure()
        .rotate("refresh-error", "access-retry", "rotated-retry");
    let retried = adapter(store.clone(), &server, Arc::clone(&singleflight))
        .refresh_token(&client, HANDLE)
        .await
        .unwrap();
    assert_eq!(retried, "access-retry");
    assert_eq!(store.fetch_attempts(HANDLE), 2);
    assert_eq!(store.store_attempts(HANDLE), 1);
    assert_eq!(exchanges_since(&server, "refresh-error", after_failure), 1);
}

#[tokio::test(flavor = "current_thread")]
async fn fetch_and_permanent_store_failures_are_shared_and_bounded() {
    const HANDLE: &str = "tenant/seat-storage-error";
    let upstream = OAuthUpstream::new();
    let server = upstream.serve();
    let store = RecordingStore::new([(HANDLE, "refresh-storage")]);
    let singleflight = Arc::new(UpstreamOAuthSingleflight::new());
    let client = reqwest::Client::new();
    let fetch_error = RestAdapterError::SecretStoreUnavailable("fetch unavailable".to_string());
    store.set_fetch_error(Some(fetch_error.clone()));

    let fetch_results = complete_admitted(independent_adapter_calls(
        3,
        &store,
        &server,
        &singleflight,
        &client,
        HANDLE,
    ))
    .await;
    assert!(
        fetch_results
            .iter()
            .all(|result| result == &Err(fetch_error.clone()))
    );
    assert_eq!(store.fetch_attempts(HANDLE), 1);
    assert_eq!(store.store_attempts(HANDLE), 0);
    // A fetch that never succeeded must never have reached the token endpoint.
    assert_eq!(
        server.request_count(),
        0,
        "a failed secret fetch must not produce an upstream exchange: {:?}",
        server.request_lines()
    );

    store.set_fetch_error(None);
    upstream.rotate("refresh-storage", "must-not-publish", "rotated-storage");
    let store_error = RestAdapterError::SecretStoreUnavailable("store unavailable".to_string());
    store.set_store_error(Some(store_error.clone()));
    let store_results = complete_admitted(independent_adapter_calls(
        3,
        &store,
        &server,
        &singleflight,
        &client,
        HANDLE,
    ))
    .await;
    assert!(
        store_results
            .iter()
            .all(|result| { result == &Err(RestAdapterError::OAuthRefreshRetryRequired) })
    );
    assert_eq!(store.fetch_attempts(HANDLE), 2);
    assert_eq!(store.store_attempts(HANDLE), 3);
    assert_eq!(store.token(HANDLE), "refresh-storage");
    assert_eq!(exchanges_for(&server, "refresh-storage"), 1);

    store.set_store_error(None);
    // `refresh-storage` has now been consumed upstream: retire it, so any replay is
    // refused exactly as the original's `rejected_replay` mock refused it.
    upstream.revoke("refresh-storage");
    let after_consumption = server.request_count();
    let recovery_error = adapter(store.clone(), &server, Arc::clone(&singleflight))
        .refresh_token(&client, HANDLE)
        .await
        .unwrap_err();
    assert_eq!(recovery_error, RestAdapterError::OAuthRefreshRetryRequired);
    assert_eq!(store.fetch_attempts(HANDLE), 2);
    assert_eq!(store.store_attempts(HANDLE), 4);
    assert_eq!(store.token(HANDLE), "rotated-storage");
    // The recovery path re-stores the CACHED rotated token and must NOT re-exchange, so
    // the stale single-use token is never sent again.
    assert_no_replay_since(&server, "refresh-storage", after_consumption);
    assert_eq!(
        server.request_count(),
        after_consumption,
        "recovery must re-store the cached token without any upstream exchange: {:?}",
        server.request_lines()
    );

    upstream.rotate("rotated-storage", "access-after-recovery", "next-rotation");
    let retried = adapter(store.clone(), &server, Arc::clone(&singleflight))
        .refresh_token(&client, HANDLE)
        .await
        .unwrap();
    assert_eq!(retried, "access-after-recovery");
    assert_eq!(store.fetch_attempts(HANDLE), 3);
    assert_eq!(store.store_attempts(HANDLE), 5);
    assert_eq!(store.token(HANDLE), "next-rotation");
    assert_no_replay_since(&server, "refresh-storage", after_consumption);
    assert_eq!(exchanges_for(&server, "rotated-storage"), 1);
}

#[tokio::test(flavor = "current_thread")]
async fn initiating_request_abort_does_not_cancel_worker_or_strand_flight() {
    const HANDLE: &str = "tenant/seat-abort";
    let upstream = OAuthUpstream::new();
    upstream
        .rotate("refresh-abort", "access-survives", "rotated-abort")
        .rotate("rotated-abort", "access-next", "rotated-next");
    let server = upstream.serve();
    let (store, fetch_started, fetch_release) =
        RecordingStore::with_fetch_gate([(HANDLE, "refresh-abort")]);
    let singleflight = Arc::new(UpstreamOAuthSingleflight::new());
    let client = reqwest::Client::new();
    let mut calls = vec![
        refresh_future(
            adapter(store.clone(), &server, Arc::clone(&singleflight)),
            client.clone(),
            HANDLE,
        ),
        refresh_future(
            adapter(store.clone(), &server, Arc::clone(&singleflight)),
            client.clone(),
            HANDLE,
        ),
    ];
    admit_all(&mut calls);
    let follower_call = calls.pop().unwrap();
    let initiator_call = calls.pop().unwrap();
    let initiator = tokio::spawn(initiator_call);
    let follower = tokio::spawn(follower_call);

    fetch_started.acquire().await.unwrap().forget();
    initiator.abort();
    assert!(initiator.await.unwrap_err().is_cancelled());
    fetch_release.add_permits(1);

    assert_eq!(follower.await.unwrap().unwrap(), "access-survives");
    assert_eq!(store.fetch_attempts(HANDLE), 1);
    assert_eq!(store.store_attempts(HANDLE), 1);
    assert_eq!(exchanges_for(&server, "refresh-abort"), 1);

    fetch_release.add_permits(1);
    let next = adapter(store.clone(), &server, Arc::clone(&singleflight))
        .refresh_token(&client, HANDLE)
        .await
        .unwrap();
    assert_eq!(next, "access-next");
    assert_eq!(store.fetch_attempts(HANDLE), 2);
    assert_eq!(store.store_attempts(HANDLE), 2);
    assert_eq!(exchanges_for(&server, "rotated-abort"), 1);
}

#[tokio::test(flavor = "current_thread")]
async fn panicking_provider_future_publishes_failure_and_allows_retry() {
    const HANDLE: &str = "tenant/seat-panic";
    let upstream = OAuthUpstream::new();
    upstream.rotate("refresh-panic", "access-retry", "rotated-retry");
    let server = upstream.serve();
    let store = RecordingStore::new([(HANDLE, "refresh-panic")]);
    store.panic_next_fetch();
    let singleflight = Arc::new(UpstreamOAuthSingleflight::new());
    let client = reqwest::Client::new();

    let results = complete_admitted(independent_adapter_calls(
        3,
        &store,
        &server,
        &singleflight,
        &client,
        HANDLE,
    ))
    .await;
    let expected = RestAdapterError::OAuthRefreshFailed("singleflight worker panicked".to_string());
    assert!(
        results
            .iter()
            .all(|result| result == &Err(expected.clone()))
    );
    assert_eq!(store.fetch_attempts(HANDLE), 1);
    assert_eq!(store.store_attempts(HANDLE), 0);
    assert_eq!(
        server.request_count(),
        0,
        "a panicking fetch must not produce an upstream exchange: {:?}",
        server.request_lines()
    );

    let retried = adapter(store.clone(), &server, Arc::clone(&singleflight))
        .refresh_token(&client, HANDLE)
        .await
        .unwrap();
    assert_eq!(retried, "access-retry");
    assert_eq!(store.fetch_attempts(HANDLE), 2);
    assert_eq!(store.store_attempts(HANDLE), 1);
    assert_eq!(exchanges_for(&server, "refresh-panic"), 1);
}

#[tokio::test(flavor = "current_thread")]
async fn transient_store_failure_and_panic_retry_without_replaying_single_use_token() {
    const HANDLE: &str = "tenant/seat-single-use";
    let upstream = OAuthUpstream::new();
    upstream.rotate("single-use-old", "access-1", "single-use-rotated");
    let server = upstream.serve();
    let store = RecordingStore::new([(HANDLE, "single-use-old")]);
    store.fail_next_store(RestAdapterError::SecretStoreUnavailable(
        "transient store failure".to_string(),
    ));
    let singleflight = Arc::new(UpstreamOAuthSingleflight::new());
    let client = reqwest::Client::new();

    let results = complete_admitted(independent_adapter_calls(
        4,
        &store,
        &server,
        &singleflight,
        &client,
        HANDLE,
    ))
    .await;
    assert!(
        results
            .iter()
            .all(|result| result.as_deref() == Ok("access-1"))
    );
    assert_eq!(store.fetch_attempts(HANDLE), 1);
    assert_eq!(store.store_attempts(HANDLE), 2);
    assert_eq!(store.token(HANDLE), "single-use-rotated");
    assert_eq!(exchanges_for(&server, "single-use-old"), 1);

    // `single-use-old` is spent: retire it so any replay is refused upstream.
    upstream.revoke("single-use-old");
    let after_old_consumed = server.request_count();
    store.panic_next_store();
    upstream.rotate("single-use-rotated", "access-2", "single-use-next");
    let panic_results = complete_admitted(independent_adapter_calls(
        3,
        &store,
        &server,
        &singleflight,
        &client,
        HANDLE,
    ))
    .await;
    let panic_error = RestAdapterError::OAuthRefreshRetryRequired;
    assert!(
        panic_results
            .iter()
            .all(|result| result == &Err(panic_error.clone()))
    );
    assert_eq!(store.fetch_attempts(HANDLE), 2);
    assert_eq!(store.store_attempts(HANDLE), 3);
    assert_eq!(store.token(HANDLE), "single-use-rotated");
    assert_no_replay_since(&server, "single-use-old", after_old_consumed);
    assert_eq!(exchanges_for(&server, "single-use-rotated"), 1);

    let recovery_error = adapter(store.clone(), &server, Arc::clone(&singleflight))
        .refresh_token(&client, HANDLE)
        .await
        .unwrap_err();
    assert_eq!(recovery_error, RestAdapterError::OAuthRefreshRetryRequired);
    assert_eq!(store.token(HANDLE), "single-use-next");
    assert_eq!(store.store_attempts(HANDLE), 4);
    assert_no_replay_since(&server, "single-use-old", after_old_consumed);
    assert_eq!(exchanges_for(&server, "single-use-rotated"), 1);

    upstream.revoke("single-use-rotated");
    let after_rotated_consumed = server.request_count();
    upstream.rotate("single-use-next", "access-3", "single-use-final");
    let retried = adapter(store.clone(), &server, Arc::clone(&singleflight))
        .refresh_token(&client, HANDLE)
        .await
        .unwrap();
    assert_eq!(retried, "access-3");
    assert_eq!(store.fetch_attempts(HANDLE), 3);
    assert_eq!(store.store_attempts(HANDLE), 5);
    assert_eq!(store.token(HANDLE), "single-use-final");
    assert_no_replay_since(&server, "single-use-old", after_old_consumed);
    assert_no_replay_since(&server, "single-use-rotated", after_rotated_consumed);
    assert_eq!(exchanges_for(&server, "single-use-next"), 1);
}

#[tokio::test(flavor = "current_thread")]
async fn hung_store_attempt_is_aborted_and_retried() {
    const HANDLE: &str = "tenant/seat-store-timeout";
    let upstream = OAuthUpstream::new();
    upstream.rotate("refresh-timeout", "access-timeout", "rotated-timeout");
    let server = upstream.serve();
    let store = RecordingStore::new([(HANDLE, "refresh-timeout")]);
    store.hang_next_store();
    let singleflight = Arc::new(UpstreamOAuthSingleflight::new());
    let client = reqwest::Client::new();

    let results = complete_admitted(independent_adapter_calls(
        3,
        &store,
        &server,
        &singleflight,
        &client,
        HANDLE,
    ))
    .await;
    assert!(
        results
            .iter()
            .all(|result| result.as_deref() == Ok("access-timeout"))
    );
    assert_eq!(store.fetch_attempts(HANDLE), 1);
    assert_eq!(store.store_attempts(HANDLE), 2);
    assert_eq!(store.token(HANDLE), "rotated-timeout");
    // The store was retried, but the single-use token must NOT have been re-exchanged.
    assert_eq!(exchanges_for(&server, "refresh-timeout"), 1);
    assert_eq!(server.request_count(), 1);
}
