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

use std::sync::Arc;

use intelligence_rest_proxy::{RestAdapterError, UpstreamOAuthSingleflight};

mod d3_singleflight_upstream_support;
use d3_singleflight_upstream_support::*;

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
