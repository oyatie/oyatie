//! Per-seat singleflight refresh coalescer.
//!
//! When multiple async callers race to refresh the same seat's token, only one
//! actually performs the HTTP call; the rest wait and receive the same result.
//! This prevents thundering-herd 429s when many pool seats are due for refresh
//! simultaneously.
//!
//! Implementation: a `Mutex<HashMap<SeatId, Shared<BoxFuture>>>` keyed by seat.
//! The first caller inserts and drives the future; latecomers clone the `Shared`
//! handle and await it without issuing a second HTTP request.
// data_class: INTERNAL_ONLY throughout this module.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use futures_util::FutureExt;
use futures_util::future::{BoxFuture, Shared};

use crate::oauth_client::OAuthClientError;
use crate::ports::SeatId;
use crate::token_state::SeatTokenState;

/// Result type shared across concurrent waiters.
pub type RefreshResult = Result<SeatTokenState, OAuthClientError>;

/// The coalescing key: a seat identifier.
type FlightMap = Mutex<HashMap<String, Shared<BoxFuture<'static, RefreshResult>>>>;

/// Per-process singleflight registry for token refreshes.
pub struct RefreshSingleflight {
    in_flight: Arc<FlightMap>,
}

impl Default for RefreshSingleflight {
    fn default() -> Self {
        Self {
            in_flight: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl RefreshSingleflight {
    pub fn new() -> Self {
        Self::default()
    }

    /// Run `make_refresh_fut` for `seat_id` — or wait for an already-in-flight
    /// refresh for the same seat. Either way, returns the result of the single
    /// underlying HTTP call. After the future resolves, the seat is removed from
    /// the in-flight map so the next call triggers a fresh HTTP round-trip.
    pub async fn run<F, Fut>(&self, seat_id: &SeatId, make_refresh_fut: F) -> RefreshResult
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = RefreshResult> + Send + 'static,
    {
        let key = seat_id.0.clone();
        let shared = {
            let mut map = self.in_flight.lock().expect("singleflight lock poisoned");
            if let Some(existing) = map.get(&key) {
                existing.clone()
            } else {
                let fut: BoxFuture<'static, RefreshResult> = Box::pin(make_refresh_fut());
                let shared = fut.shared();
                map.insert(key.clone(), shared.clone());
                shared
            }
        };

        let result = shared.await;

        // Remove from map after resolution so the next call re-triggers.
        {
            let mut map = self.in_flight.lock().expect("singleflight lock poisoned");
            map.remove(&key);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn single_http_call_for_concurrent_refreshes() {
        let sf = Arc::new(RefreshSingleflight::new());
        let call_count = Arc::new(AtomicU32::new(0));

        let seat = SeatId("seat-1".into());

        let mut handles = vec![];
        for _ in 0..10 {
            let sf2 = Arc::clone(&sf);
            let counter = Arc::clone(&call_count);
            let seat2 = seat.clone();
            handles.push(tokio::spawn(async move {
                sf2.run(&seat2, || {
                    let c = Arc::clone(&counter);
                    async move {
                        c.fetch_add(1, Ordering::SeqCst);
                        // Simulate brief HTTP delay.
                        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                        Ok(SeatTokenState::new(
                            "access".into(),
                            "refresh".into(),
                            9999,
                            1000,
                        ))
                    }
                })
                .await
            }));
        }

        // Give all tasks a chance to register before the first completes.
        tokio::task::yield_now().await;

        let results: Vec<_> = futures_util::future::join_all(handles).await;
        assert_eq!(
            call_count.load(Ordering::SeqCst),
            1,
            "only one HTTP call expected"
        );
        for r in results {
            let state = r.unwrap().unwrap();
            assert_eq!(state.access_token, "access");
        }
    }

    #[tokio::test]
    async fn sequential_calls_each_trigger_refresh() {
        let sf = RefreshSingleflight::new();
        let call_count = Arc::new(AtomicU32::new(0));
        let seat = SeatId("seat-seq".into());

        for _ in 0..3 {
            let counter = Arc::clone(&call_count);
            sf.run(&seat, || {
                let c = counter;
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(SeatTokenState::new("a".into(), "r".into(), 9999, 0))
                }
            })
            .await
            .unwrap();
        }

        assert_eq!(
            call_count.load(Ordering::SeqCst),
            3,
            "each sequential call triggers refresh"
        );
    }
}
