//! Item 2 — upstream OAuth singleflight test.
//!
//! 10 concurrent `refresh_token()` calls on the same handle → exactly 1
//! mock-server hit. Uses `httpmock` as the upstream stand-in.
//!
//! ADR-0384 D3 singleflight contract: concurrent callers on the same handle
//! coalesce into ONE upstream OAuth call; followers wait on the broadcast
//! channel and receive the same result.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::sync::Arc;

use httpmock::prelude::*;
use intelligence_rest::{AnthropicAdapter, RestAdapterError, SecretProviderStore};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

struct StubStore {
    token: String,
}

impl SecretProviderStore for StubStore {
    fn fetch_refresh_token<'a>(
        &'a self,
        _: &'a str,
    ) -> intelligence_rest::SecretProviderFuture<'a, String> {
        Box::pin(async move { Ok(self.token.clone()) })
    }
    fn store_refresh_token<'a>(
        &'a self,
        _: &'a str,
        _: &'a str,
    ) -> intelligence_rest::SecretProviderFuture<'a, ()> {
        Box::pin(async { Ok(()) })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// 10 concurrent `refresh_token()` calls on the same handle must result in
/// exactly 1 hit on the mock OAuth endpoint. All 10 callers receive the same
/// access token.
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn singleflight_10_concurrent_refresh_token_calls_produce_1_upstream_hit() {
    let server = MockServer::start();

    // Mock the token endpoint. httpmock's `assert_hits(1)` will fail the test
    // if more than 1 request reaches this mock.
    let token_mock = server.mock(|when, then| {
        when.method(POST).path("/v1/oauth/token");
        then.status(200)
            .header("content-type", "application/json")
            // Inject a small delay so concurrent tasks pile up behind the leader.
            .delay(std::time::Duration::from_millis(60))
            .body(
                r#"{"access_token":"coalesced-access-tok","refresh_token":"new-rt","expires_in":3600}"#,
            );
    });

    let adapter = Arc::new(AnthropicAdapter::with_base_url(
        StubStore {
            token: "initial-rt".to_string(),
        },
        server.base_url(),
    ));

    let http_client = Arc::new(
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap(),
    );

    let mut handles = Vec::new();
    for _ in 0..10 {
        let adapter = Arc::clone(&adapter);
        let client = Arc::clone(&http_client);
        handles.push(tokio::spawn(async move {
            adapter
                .refresh_token(&client, "tenant-sf/seat-sf-upstream")
                .await
        }));
    }

    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    // All callers receive the coalesced access token.
    for result in &results {
        assert!(result.is_ok(), "all callers should receive Ok: {result:?}");
        assert_eq!(
            result.as_deref().unwrap(),
            "coalesced-access-tok",
            "all callers should receive the same coalesced access token"
        );
    }

    // Exactly 1 upstream call — the singleflight coalesced the rest.
    token_mock.assert_hits(1);
}

/// Singleflight error propagation: if the leader fails, all followers receive
/// the same error.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn singleflight_error_propagates_to_all_followers() {
    let server = MockServer::start();

    // Token endpoint returns an error.
    let _token_mock = server.mock(|when, then| {
        when.method(POST).path("/v1/oauth/token");
        then.status(503)
            .delay(std::time::Duration::from_millis(40))
            .body(r#"{"error":"service_unavailable"}"#);
    });

    let adapter = Arc::new(AnthropicAdapter::with_base_url(
        StubStore {
            token: "rt-err".to_string(),
        },
        server.base_url(),
    ));

    let http_client = Arc::new(reqwest::Client::new());

    let mut handles = Vec::new();
    for _ in 0..5 {
        let adapter = Arc::clone(&adapter);
        let client = Arc::clone(&http_client);
        handles.push(tokio::spawn(async move {
            adapter.refresh_token(&client, "tenant-err/seat-err").await
        }));
    }

    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    for result in results {
        assert!(result.is_err(), "all callers should receive the error");
        assert!(
            matches!(result.unwrap_err(), RestAdapterError::OAuthRefreshFailed(_)),
            "error should be OAuthRefreshFailed"
        );
    }
}

// ---------------------------------------------------------------------------
// Minimal futures re-export (avoids adding a full dep for join_all).
// ---------------------------------------------------------------------------
mod futures {
    pub mod future {
        pub async fn join_all<I>(iter: I) -> Vec<<I::Item as std::future::IntoFuture>::Output>
        where
            I: IntoIterator,
            I::Item: std::future::IntoFuture,
        {
            let mut results = Vec::new();
            for fut in iter {
                results.push(fut.into_future().await);
            }
            results
        }
    }
}
