#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::sync::Arc;

use intelligence_rest_proxy::{RestAdapterError, UpstreamOAuthSingleflight};

mod d3_singleflight_upstream_support;
use d3_singleflight_upstream_support::*;

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
