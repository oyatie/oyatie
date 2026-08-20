//! Deterministic D3 upstream OAuth keyed-flight lifecycle tests.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::Duration;

use httpmock::Mock;
use httpmock::prelude::*;
use intelligence_rest::{
    AnthropicAdapter, RestAdapterError, SecretProviderFuture, SecretProviderStore,
    UpstreamOAuthSingleflight,
};
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

fn success_mock<'a>(
    server: &'a MockServer,
    current_refresh: &str,
    access_token: &str,
    rotated_refresh: &str,
) -> Mock<'a> {
    server.mock(|when, then| {
        when.method(POST)
            .path("/v1/oauth/token")
            .body_contains(format!(r#""refresh_token":"{current_refresh}""#));
        then.status(200)
            .header("content-type", "application/json")
            .body(format!(
                r#"{{"access_token":"{access_token}","refresh_token":"{rotated_refresh}","expires_in":3600}}"#
            ));
    })
}

fn adapter(
    store: RecordingStore,
    server: &MockServer,
    singleflight: Arc<UpstreamOAuthSingleflight>,
) -> Arc<AnthropicAdapter<RecordingStore>> {
    Arc::new(AnthropicAdapter::with_base_url_and_singleflight(
        store,
        server.base_url(),
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
    server: &MockServer,
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
    let server = MockServer::start();
    let first = success_mock(&server, "initial-rt", "access-1", "rotated-rt-1");
    let second = success_mock(&server, "rotated-rt-1", "access-2", "rotated-rt-2");
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
    first.assert_hits(1);

    let next = adapter(store.clone(), &server, Arc::clone(&singleflight))
        .refresh_token(&client, HANDLE)
        .await
        .unwrap();
    assert_eq!(next, "access-2");
    assert_eq!(store.fetch_attempts(HANDLE), 2);
    assert_eq!(store.store_attempts(HANDLE), 2);
    assert_eq!(store.token(HANDLE), "rotated-rt-2");
    second.assert_hits(1);
}

#[tokio::test(flavor = "current_thread")]
async fn different_handles_run_independent_flights() {
    const HANDLE_A: &str = "tenant/seat-a";
    const HANDLE_B: &str = "tenant/seat-b";
    let server = MockServer::start();
    let mock_a = success_mock(&server, "refresh-a", "access-a", "rotated-a");
    let mock_b = success_mock(&server, "refresh-b", "access-b", "rotated-b");
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
    mock_a.assert_hits(1);
    mock_b.assert_hits(1);
}

#[tokio::test(flavor = "current_thread")]
async fn exchange_failure_is_shared_without_store_and_a_later_retry_starts() {
    const HANDLE: &str = "tenant/seat-error";
    let server = MockServer::start();
    let mut failure = server.mock(|when, then| {
        when.method(POST).path("/v1/oauth/token");
        then.status(503).body("provider unavailable");
    });
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
    failure.assert_hits(1);
    failure.delete();

    let success = success_mock(&server, "refresh-error", "access-retry", "rotated-retry");
    let retried = adapter(store.clone(), &server, Arc::clone(&singleflight))
        .refresh_token(&client, HANDLE)
        .await
        .unwrap();
    assert_eq!(retried, "access-retry");
    assert_eq!(store.fetch_attempts(HANDLE), 2);
    assert_eq!(store.store_attempts(HANDLE), 1);
    success.assert_hits(1);
}

#[tokio::test(flavor = "current_thread")]
async fn fetch_and_permanent_store_failures_are_shared_and_bounded() {
    const HANDLE: &str = "tenant/seat-storage-error";
    let server = MockServer::start();
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

    store.set_fetch_error(None);
    let mut success = success_mock(
        &server,
        "refresh-storage",
        "must-not-publish",
        "rotated-storage",
    );
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
    success.assert_hits(1);

    store.set_store_error(None);
    success.delete();
    let rejected_replay = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/oauth/token")
            .body_contains(r#""refresh_token":"refresh-storage""#);
        then.status(400)
            .body("single-use refresh token already consumed");
    });
    let recovery_error = adapter(store.clone(), &server, Arc::clone(&singleflight))
        .refresh_token(&client, HANDLE)
        .await
        .unwrap_err();
    assert_eq!(recovery_error, RestAdapterError::OAuthRefreshRetryRequired);
    assert_eq!(store.fetch_attempts(HANDLE), 2);
    assert_eq!(store.store_attempts(HANDLE), 4);
    assert_eq!(store.token(HANDLE), "rotated-storage");
    rejected_replay.assert_hits(0);

    let rotated = success_mock(
        &server,
        "rotated-storage",
        "access-after-recovery",
        "next-rotation",
    );
    let retried = adapter(store.clone(), &server, Arc::clone(&singleflight))
        .refresh_token(&client, HANDLE)
        .await
        .unwrap();
    assert_eq!(retried, "access-after-recovery");
    assert_eq!(store.fetch_attempts(HANDLE), 3);
    assert_eq!(store.store_attempts(HANDLE), 5);
    assert_eq!(store.token(HANDLE), "next-rotation");
    rejected_replay.assert_hits(0);
    rotated.assert_hits(1);
}

#[tokio::test(flavor = "current_thread")]
async fn initiating_request_abort_does_not_cancel_worker_or_strand_flight() {
    const HANDLE: &str = "tenant/seat-abort";
    let server = MockServer::start();
    let first = success_mock(&server, "refresh-abort", "access-survives", "rotated-abort");
    let second = success_mock(&server, "rotated-abort", "access-next", "rotated-next");
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
    first.assert_hits(1);

    fetch_release.add_permits(1);
    let next = adapter(store.clone(), &server, Arc::clone(&singleflight))
        .refresh_token(&client, HANDLE)
        .await
        .unwrap();
    assert_eq!(next, "access-next");
    assert_eq!(store.fetch_attempts(HANDLE), 2);
    assert_eq!(store.store_attempts(HANDLE), 2);
    second.assert_hits(1);
}

#[tokio::test(flavor = "current_thread")]
async fn panicking_provider_future_publishes_failure_and_allows_retry() {
    const HANDLE: &str = "tenant/seat-panic";
    let server = MockServer::start();
    let success = success_mock(&server, "refresh-panic", "access-retry", "rotated-retry");
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

    let retried = adapter(store.clone(), &server, Arc::clone(&singleflight))
        .refresh_token(&client, HANDLE)
        .await
        .unwrap();
    assert_eq!(retried, "access-retry");
    assert_eq!(store.fetch_attempts(HANDLE), 2);
    assert_eq!(store.store_attempts(HANDLE), 1);
    success.assert_hits(1);
}

#[tokio::test(flavor = "current_thread")]
async fn transient_store_failure_and_panic_retry_without_replaying_single_use_token() {
    const HANDLE: &str = "tenant/seat-single-use";
    let server = MockServer::start();
    let mut first = success_mock(&server, "single-use-old", "access-1", "single-use-rotated");
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
    first.assert_hits(1);
    first.delete();

    let rejected_replay = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/oauth/token")
            .body_contains(r#""refresh_token":"single-use-old""#);
        then.status(400)
            .body("single-use refresh token already consumed");
    });
    store.panic_next_store();
    let mut second = success_mock(&server, "single-use-rotated", "access-2", "single-use-next");
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
    rejected_replay.assert_hits(0);
    second.assert_hits(1);

    let recovery_error = adapter(store.clone(), &server, Arc::clone(&singleflight))
        .refresh_token(&client, HANDLE)
        .await
        .unwrap_err();
    assert_eq!(recovery_error, RestAdapterError::OAuthRefreshRetryRequired);
    assert_eq!(store.token(HANDLE), "single-use-next");
    assert_eq!(store.store_attempts(HANDLE), 4);
    rejected_replay.assert_hits(0);
    second.assert_hits(1);

    second.delete();
    let rejected_second_replay = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/oauth/token")
            .body_contains(r#""refresh_token":"single-use-rotated""#);
        then.status(400)
            .body("single-use refresh token already consumed");
    });
    let third = success_mock(&server, "single-use-next", "access-3", "single-use-final");
    let retried = adapter(store.clone(), &server, Arc::clone(&singleflight))
        .refresh_token(&client, HANDLE)
        .await
        .unwrap();
    assert_eq!(retried, "access-3");
    assert_eq!(store.fetch_attempts(HANDLE), 3);
    assert_eq!(store.store_attempts(HANDLE), 5);
    assert_eq!(store.token(HANDLE), "single-use-final");
    rejected_replay.assert_hits(0);
    rejected_second_replay.assert_hits(0);
    third.assert_hits(1);
}

#[tokio::test(flavor = "current_thread")]
async fn hung_store_attempt_is_aborted_and_retried() {
    const HANDLE: &str = "tenant/seat-store-timeout";
    let server = MockServer::start();
    let success = success_mock(
        &server,
        "refresh-timeout",
        "access-timeout",
        "rotated-timeout",
    );
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
    success.assert_hits(1);
}
