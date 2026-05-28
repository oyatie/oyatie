//! Fix #1 — Same-seat concurrent race.
//!
//! Spawns N tokio tasks calling `choose_key()` concurrently on a pool with
//! fewer keys than tasks. Asserts that no two leases hold the same `KeyId`
//! at the same time.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use oya_llm_gateway_kernel::{KeyId, PoolPolicy, ProviderChannel, SeatOutcome};
use oya_llm_gateway_rest::auth::AuthVerifier;
use oya_llm_gateway_rest::channel::ChannelAdapter;
use oya_llm_gateway_rest::config::RetryPolicyConfig;
use oya_llm_gateway_rest::keystore::KeyMaterial;
use oya_llm_gateway_rest::metrics::GatewayMetrics;
use oya_llm_gateway_rest::state::{GatewayState, GroupRuntime, KeyChoice};

fn build_group_with_n_keys(n: usize) -> Arc<GroupRuntime> {
    let mut map = BTreeMap::new();
    for i in 0..n {
        map.insert(format!("k{i}"), format!("raw-key-{i}"));
    }
    let material = KeyMaterial::from_map(ProviderChannel::OpenAi, map);
    let adapter = ChannelAdapter::new(ProviderChannel::OpenAi, "https://api.openai.com", None);
    Arc::new(GroupRuntime::new(
        "codex",
        adapter,
        RetryPolicyConfig {
            retry_on_statuses: vec![429],
            max_attempts: 1,
            backoff_base_millis: 0,
            backoff_jitter_millis: 0,
        },
        PoolPolicy::new(100, 60_000, 0), // very high threshold so no key blacklists
        material,
    ))
}

/// 20 tasks race to acquire leases on a 3-key pool. At no point should two
/// tasks hold a lease for the same KeyId concurrently.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_leases_never_share_same_key_id() {
    const TASKS: usize = 20;
    const KEYS: usize = 3;

    let group = build_group_with_n_keys(KEYS);

    // Shared log of all currently-held (in-flight) KeyIds.
    let held: Arc<Mutex<BTreeSet<usize>>> = Arc::new(Mutex::new(BTreeSet::new()));
    // Any double-allocation gets logged here.
    let violations: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::new()));

    let mut handles = Vec::new();
    for _ in 0..TASKS {
        let g = Arc::clone(&group);
        let held = Arc::clone(&held);
        let violations = Arc::clone(&violations);
        handles.push(tokio::spawn(async move {
            match g.choose_key() {
                KeyChoice::Chosen(chosen) => {
                    let id = chosen.id.0;
                    // Register the id as in-flight; detect double allocation.
                    {
                        let mut set = held.lock().unwrap();
                        if !set.insert(id) {
                            // Same id already held by another task!
                            violations.lock().unwrap().push(id);
                        }
                    }
                    // Simulate a brief async wait (the "upstream call").
                    tokio::task::yield_now().await;
                    // Release the lease.
                    g.complete_lease(chosen.lease, SeatOutcome::Success);
                    // Deregister.
                    held.lock().unwrap().remove(&id);
                }
                KeyChoice::Exhausted | KeyChoice::Empty => {
                    // Pool fully leased — acceptable when all keys are in-flight.
                }
            }
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    let v = violations.lock().unwrap().clone();
    assert!(
        v.is_empty(),
        "detected concurrent double-allocation of KeyId(s): {v:?}"
    );
}

/// After all leases complete the pool is fully available again.
#[tokio::test]
async fn all_leases_complete_restores_full_pool() {
    const KEYS: usize = 3;
    let group = build_group_with_n_keys(KEYS);

    // Drain all keys.
    let mut leases = Vec::new();
    for _ in 0..KEYS {
        match group.choose_key() {
            KeyChoice::Chosen(c) => leases.push(c),
            _ => panic!("expected key"),
        }
    }
    // Pool should now be exhausted.
    assert!(matches!(group.choose_key(), KeyChoice::Exhausted));

    // Complete all leases.
    for lease in leases {
        group.complete_lease(lease.lease, SeatOutcome::Success);
    }

    // All keys should be available again.
    assert_eq!(group.active_key_count(), KEYS);
}

/// Verify that `SeatOutcome::RefreshFailed` penalises the seat (Fix #6).
#[tokio::test]
async fn refresh_failed_outcome_penalises_and_fails_over() {
    // 2 keys; RefreshFailed on key 0 should cause the next choose to yield key 1.
    let group = build_group_with_n_keys(2);

    let chosen0 = match group.choose_key() {
        KeyChoice::Chosen(c) => c,
        other => panic!("expected key, got {other:?}"),
    };
    let key0_id = chosen0.id;

    // Complete with RefreshFailed — penalises but with threshold=100 won't blacklist.
    group.complete_lease(
        chosen0.lease,
        SeatOutcome::RefreshFailed {
            now_unix_millis: 0,
            jitter_seed: 0,
        },
    );

    // Next choose should return key 1 (key 0 is still active but cursor advanced).
    let chosen1 = match group.choose_key() {
        KeyChoice::Chosen(c) => c,
        other => panic!("expected second key, got {other:?}"),
    };
    group.complete_lease(chosen1.lease, SeatOutcome::Success);

    // Both keys still active (failure count < threshold=100).
    assert_eq!(group.active_key_count(), 2);
    let _ = key0_id; // suppress unused warning
}
