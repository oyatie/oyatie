//! Fix-2: Token-refresh singleflight — at most one token-exchange call per
//! handle at any time under concurrent load.
//!
//! 10 tokio tasks target the same mock OAuth endpoint simultaneously; the test
//! asserts exactly 1 call was made to the token endpoint.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use oya_cloud_intelligence_rest::{CachedToken, TokenRefreshSingleflight};

/// Shared counter that tracks how many times `do_refresh` was actually called.
#[derive(Clone, Default)]
struct CallCounter {
    count: Arc<Mutex<u32>>,
}

impl CallCounter {
    fn increment(&self) {
        *self.count.lock().unwrap() += 1;
    }
    fn get(&self) -> u32 {
        *self.count.lock().unwrap()
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn singleflight_coalesces_10_concurrent_refreshes_to_1_call() {
    let sf = Arc::new(TokenRefreshSingleflight::new());
    let counter = CallCounter::default();

    let mut handles = Vec::new();
    for _ in 0..10 {
        let sf = Arc::clone(&sf);
        let counter = counter.clone();

        handles.push(tokio::spawn(async move {
            // Use spawn_blocking to mirror the production path where refresh
            // is called on a blocking thread.
            let sf_inner = Arc::clone(&sf);
            let counter_inner = counter.clone();
            tokio::task::spawn_blocking(move || {
                sf_inner.refresh_or_wait("tenant/seat-sf-1", || {
                    // Simulate a slow token exchange.
                    std::thread::sleep(Duration::from_millis(30));
                    counter_inner.increment();
                    Ok(CachedToken {
                        access_token: "access-tok-coalesced".to_string(),
                        expires_at: Instant::now() + Duration::from_secs(3600),
                    })
                })
            })
            .await
            .unwrap()
        }));
    }

    let results: Vec<_> = futures_util::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    // All callers should receive the same access token.
    for result in &results {
        assert_eq!(
            result.as_ref().unwrap().access_token,
            "access-tok-coalesced"
        );
    }

    // Exactly 1 call to the mock OAuth endpoint (the rest were coalesced).
    assert_eq!(
        counter.get(),
        1,
        "expected exactly 1 token-exchange call under singleflight; got {}",
        counter.get()
    );
}

#[tokio::test]
async fn singleflight_different_handles_are_independent() {
    let sf = Arc::new(TokenRefreshSingleflight::new());
    let counter = CallCounter::default();

    let mut handles = Vec::new();
    for i in 0..5u32 {
        let sf = Arc::clone(&sf);
        let counter = counter.clone();
        handles.push(tokio::spawn(async move {
            let sf_inner = Arc::clone(&sf);
            let counter_inner = counter.clone();
            let handle = format!("tenant/seat-{i}");
            tokio::task::spawn_blocking(move || {
                sf_inner.refresh_or_wait(&handle, || {
                    counter_inner.increment();
                    Ok(CachedToken {
                        access_token: format!("tok-{i}"),
                        expires_at: Instant::now() + Duration::from_secs(3600),
                    })
                })
            })
            .await
            .unwrap()
        }));
    }

    let results: Vec<_> = futures_util::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    for result in &results {
        assert!(result.is_ok());
    }
    // 5 different handles = 5 calls.
    assert_eq!(counter.get(), 5);
}

#[tokio::test]
async fn singleflight_propagates_error_to_all_waiters() {
    let sf = Arc::new(TokenRefreshSingleflight::new());

    let mut handles = Vec::new();
    for _ in 0..5 {
        let sf = Arc::clone(&sf);
        handles.push(tokio::spawn(async move {
            let sf_inner = Arc::clone(&sf);
            tokio::task::spawn_blocking(move || {
                sf_inner.refresh_or_wait("tenant/seat-err", || {
                    std::thread::sleep(Duration::from_millis(10));
                    Err("vault unavailable".to_string())
                })
            })
            .await
            .unwrap()
        }));
    }

    let results: Vec<_> = futures_util::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    for result in results {
        assert!(result.is_err(), "all waiters should receive the error");
        assert!(result.unwrap_err().contains("vault unavailable"));
    }
}

// Minimal futures_util re-export so we don't need to add a dev-dep.
mod futures_util {
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
