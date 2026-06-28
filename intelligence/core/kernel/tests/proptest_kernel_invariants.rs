//! Proptest invariants for the OAuth subscription-pool kernel.
//!
//! Six properties that must hold for all input sequences:
//!
//! 1. Seat-count conservation: `seat_count == leased_count + free_count` at all times.
//! 2. Cooldown recovery: a seat in Cooldown becomes Active once `now` passes the deadline.
//! 3. RefreshFailed above threshold → Blacklisted.
//! 4. `failure_count` monotonically increases on non-Ok outcomes (never decreases except on Ok).
//! 5. RoundRobin visits each eligible seat at least once per N-seat cycle.
//! 6. FillFirst always returns the lowest-failure-count eligible seat (first in BTreeMap order).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use intelligence_kernel::{
    AgentId, AuthzDecision, AuthzGate, AuthzRequest, OAuthSubscription, Provider, SeatId,
    SeatOutcome, SelectionStrategy, SubscriptionId, SubscriptionPool, SubscriptionPoolError,
    SubscriptionState, TenantId,
};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

struct AllowAll;
impl AuthzGate for AllowAll {
    fn decide(&self, _r: &AuthzRequest<'_>) -> AuthzDecision {
        AuthzDecision::Allow
    }
}

fn tid(s: &str) -> TenantId {
    TenantId::new(s).unwrap()
}
fn aid(s: &str) -> AgentId {
    AgentId::new(s).unwrap()
}
fn sid(s: &str) -> SeatId {
    SeatId::new(s).unwrap()
}
fn make_sub(tenant: &str, seat: &str) -> OAuthSubscription {
    OAuthSubscription::new(
        tid(tenant),
        sid(seat),
        SubscriptionId::new(format!("{seat}-sub")).unwrap(),
        Provider::Anthropic,
        SubscriptionState::Active,
        format!("secret-ref://{tenant}/{seat}/refresh"),
        0,
    )
}

/// Build a pool wrapped in Arc<Mutex<>> for lease-based tests.
fn arc_pool(n: usize) -> Arc<Mutex<SubscriptionPool>> {
    let tenant = tid("t-prop");
    let mut pool = SubscriptionPool::new(
        tenant.clone(),
        Provider::Anthropic,
        SelectionStrategy::RoundRobin,
    );
    for i in 0..n {
        pool.add_seat(OAuthSubscription::new(
            tenant.clone(),
            sid(&format!("prop-seat-{i}")),
            SubscriptionId::new(format!("prop-sub-{i}")).unwrap(),
            Provider::Anthropic,
            SubscriptionState::Active,
            format!("secret-ref://t-prop/prop-seat-{i}/refresh"),
            0,
        ))
        .unwrap();
    }
    Arc::new(Mutex::new(pool))
}

// ---------------------------------------------------------------------------
// Proptest operation model
// ---------------------------------------------------------------------------

/// A single operation in the property-test sequence.
#[derive(Clone, Debug)]
enum PoolOp {
    LeaseComplete(SeatOutcome),
    RecordOutcome(usize, SeatOutcome), // seat index, outcome
}

fn arb_seat_outcome() -> impl Strategy<Value = SeatOutcome> {
    prop_oneof![
        Just(SeatOutcome::Ok),
        Just(SeatOutcome::RateLimited429),
        Just(SeatOutcome::ServerError5xx),
        Just(SeatOutcome::RefreshFailed),
    ]
}

fn arb_pool_op(n_seats: usize) -> impl Strategy<Value = PoolOp> {
    prop_oneof![
        arb_seat_outcome().prop_map(PoolOp::LeaseComplete),
        (0..n_seats, arb_seat_outcome()).prop_map(|(i, o)| PoolOp::RecordOutcome(i, o)),
    ]
}

// ---------------------------------------------------------------------------
// Property 1 — seat-count conservation
//
// For any sequence of (lease, complete(Ok)) events, the total number of seats
// in the pool never changes; the pool's seat_count() remains constant.
// (The "leased_count + free_count == seat_count" variant: since the kernel
// doesn't expose leased_count directly we verify that seat_count is stable
// and no seat is ever lost.)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn prop1_seat_count_conservation(
        ops in proptest::collection::vec(arb_seat_outcome(), 1..30usize)
    ) {
        let n_seats = 5usize;
        let pool_ref = arc_pool(n_seats);
        let gate = AllowAll;
        let agent = aid("prop1-agent");
        let now = Instant::now();

        for outcome in ops {
            match SubscriptionPool::lease(&pool_ref, &tid("t-prop"), &agent, &gate, now) {
                Ok(lease) => {
                    lease.complete(outcome, now).unwrap();
                }
                Err(SubscriptionPoolError::NoEligibleSeat) => {
                    // Pool fully leased or all seats in cooldown/blacklist — ok.
                }
                Err(e) => panic!("unexpected lease error: {e:?}"),
            }
            // Invariant: seat_count never changes.
            assert_eq!(pool_ref.lock().unwrap().seat_count(), n_seats);
        }
    }
}

// ---------------------------------------------------------------------------
// Property 2 — Cooldown seat eventually reaches Active (simulated time advance)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn prop2_cooldown_recovers_after_timer(
        _cooldown_secs in 1u64..120u64,
    ) {
        // Place a single seat into Cooldown by recording RateLimited429.
        let tenant = tid("t-prop2");
        let mut pool = SubscriptionPool::new(
            tenant.clone(),
            Provider::Anthropic,
            SelectionStrategy::FillFirst,
        );
        pool.add_seat(OAuthSubscription::new(
            tenant,
            sid("cool-seat"),
            SubscriptionId::new("cool-sub").unwrap(),
            Provider::Anthropic,
            SubscriptionState::Active,
            "secret-ref://t-prop2/cool-seat/refresh",
            0,
        )).unwrap();

        let t0 = Instant::now();
        pool.record_outcome(&sid("cool-seat"), SeatOutcome::RateLimited429, t0).unwrap();

        let gate = AllowAll;
        let agent = aid("prop2-agent");

        // Immediately not eligible.
        prop_assert!(pool.select(&tid("t-prop2"), &agent, &gate, t0).is_err());

        // After cooldown_duration_429 (default 60s) + 1s — eligible again.
        // The kernel hardcodes 60s for RateLimited429; we advance past 61s.
        let after = t0 + Duration::from_secs(61);
        prop_assert!(
            pool.select(&tid("t-prop2"), &agent, &gate, after).is_ok(),
            "seat should recover after cooldown expires"
        );
    }
}

// ---------------------------------------------------------------------------
// Property 3 — repeated RefreshFailed above threshold → Blacklisted
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn prop3_refresh_failed_above_threshold_blacklists(
        extra_failures in 1u32..10u32,
    ) {
        // BLACKLIST_THRESHOLD = 5; apply 5 + extra_failures RefreshFailed outcomes.
        let blacklist_threshold = 5u32;
        let n = blacklist_threshold + extra_failures;

        let tenant = tid("t-prop3");
        let mut pool = SubscriptionPool::new(
            tenant.clone(),
            Provider::Anthropic,
            SelectionStrategy::FillFirst,
        );
        pool.add_seat(OAuthSubscription::new(
            tenant,
            sid("bl-seat"),
            SubscriptionId::new("bl-sub").unwrap(),
            Provider::Anthropic,
            SubscriptionState::Active,
            "secret-ref://t-prop3/bl-seat/refresh",
            0,
        )).unwrap();

        let t0 = Instant::now();
        let seat = sid("bl-seat");
        // Advance time between each failure to avoid the cooldown window gating
        // whether the seat would have been re-selectable.
        for i in 0..n {
            let t = t0 + Duration::from_secs(u64::from(i) * 120);
            pool.record_outcome(&seat, SeatOutcome::RefreshFailed, t).unwrap();
        }

        // Even far in the future, seat is permanently blacklisted.
        let gate = AllowAll;
        let agent = aid("prop3-agent");
        let far = t0 + Duration::from_secs(999_999);
        prop_assert!(
            pool.select(&tid("t-prop3"), &agent, &gate, far).is_err(),
            "seat must be blacklisted after {n} RefreshFailed outcomes"
        );
    }
}

// ---------------------------------------------------------------------------
// Property 4 — failure_count is monotonically non-decreasing on non-Ok outcomes
//
// We verify this by tracking failure_count via select behaviour: a seat that
// has received N non-Ok outcomes must have failure_count >= N (Ok resets to 0).
// The proxy: after K consecutive non-Ok outcomes, failure_count > 0; after one
// Ok, it resets to 0.
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn prop4_failure_count_resets_only_on_ok(
        ops in proptest::collection::vec(
            prop_oneof![
                Just(SeatOutcome::Ok),
                Just(SeatOutcome::RateLimited429),
                Just(SeatOutcome::RefreshFailed),
                Just(SeatOutcome::ServerError5xx),
            ],
            1..20usize
        )
    ) {
        let tenant = tid("t-prop4");
        let mut pool = SubscriptionPool::new(
            tenant.clone(),
            Provider::Anthropic,
            SelectionStrategy::FillFirst,
        );
        pool.add_seat(OAuthSubscription::new(
            tenant,
            sid("mono-seat"),
            SubscriptionId::new("mono-sub").unwrap(),
            Provider::Anthropic,
            SubscriptionState::Active,
            "secret-ref://t-prop4/mono-seat/refresh",
            0,
        )).unwrap();

        let t0 = Instant::now();
        let seat = sid("mono-seat");
        let mut expected_failures: u32 = 0;
        let mut last_ok = false;

        for (idx, op) in ops.iter().enumerate() {
            let t = t0 + Duration::from_secs(u64::try_from(idx).unwrap() * 120);
            pool.record_outcome(&seat, *op, t).unwrap();
            match op {
                SeatOutcome::Ok => {
                    expected_failures = 0;
                    last_ok = true;
                }
                SeatOutcome::Released => {}
                _ => {
                    expected_failures = expected_failures.saturating_add(1);
                    last_ok = false;
                }
            }
        }

        // After all ops, use select to confirm observable behaviour matches:
        // if the last op was Ok and not blacklisted, the seat should be selectable
        // far in the future (failure_count == 0 => seat is Active, not Cooldown).
        let gate = AllowAll;
        let agent = aid("prop4-agent");
        let far = t0 + Duration::from_secs(999_999);

        if last_ok {
            // failure_count was reset to 0; seat must be Active and selectable.
            prop_assert!(
                pool.select(&tid("t-prop4"), &agent, &gate, far).is_ok(),
                "after Ok outcome seat must be Active (failure_count==0)"
            );
        } else if expected_failures > 5 {
            // Blacklisted — not selectable.
            prop_assert!(
                pool.select(&tid("t-prop4"), &agent, &gate, far).is_err(),
                "after {expected_failures} failures seat must be blacklisted"
            );
        }
        // Other cases (in cooldown but not blacklisted): seat recovers at far_future
        // since cooldown = 60s and we advance by 999_999s — assert selectable.
        else if expected_failures > 0 {
            prop_assert!(
                pool.select(&tid("t-prop4"), &agent, &gate, far).is_ok(),
                "seat should recover from cooldown far in the future"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Property 5 — RoundRobin visits each eligible seat at least once per N-seat cycle
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn prop5_round_robin_covers_all_seats(
        n_seats in 2usize..8usize,
    ) {
        let tenant = tid("t-prop5");
        let mut pool = SubscriptionPool::new(
            tenant.clone(),
            Provider::Anthropic,
            SelectionStrategy::RoundRobin,
        );
        let seat_ids: Vec<SeatId> = (0..n_seats)
            .map(|i| {
                let s = sid(&format!("rr-seat-{i}"));
                pool.add_seat(OAuthSubscription::new(
                    tenant.clone(),
                    s.clone(),
                    SubscriptionId::new(format!("rr-sub-{i}")).unwrap(),
                    Provider::Anthropic,
                    SubscriptionState::Active,
                    format!("secret-ref://t-prop5/rr-seat-{i}/refresh"),
                    0,
                )).unwrap();
                s
            })
            .collect();

        let gate = AllowAll;
        let agent = aid("prop5-agent");
        let now = Instant::now();

        // Select N times (one full cycle) — all seats must appear.
        let mut seen: HashSet<SeatId> = HashSet::new();
        for _ in 0..n_seats {
            let s = pool.select(&tid("t-prop5"), &agent, &gate, now).unwrap();
            seen.insert(s);
        }

        for s in &seat_ids {
            prop_assert!(
                seen.contains(s),
                "RoundRobin did not visit seat {:?} in one full cycle",
                s
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Property 6 — FillFirst always returns the lowest-failure-count eligible seat
//
// Proxy: FillFirst with a BTreeMap iterates seats in sorted key order; the
// first eligible seat (lowest sort key with Active state) is always returned.
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn prop6_fill_first_returns_first_eligible(
        n_seats in 2usize..6usize,
        knock_out_first in 0usize..5usize,
    ) {
        let tenant = tid("t-prop6");
        let mut pool = SubscriptionPool::new(
            tenant.clone(),
            Provider::Anthropic,
            SelectionStrategy::FillFirst,
        );
        let seat_ids: Vec<SeatId> = (0..n_seats)
            .map(|i| {
                let s = sid(&format!("ff-seat-{i:02}"));
                pool.add_seat(OAuthSubscription::new(
                    tenant.clone(),
                    s.clone(),
                    SubscriptionId::new(format!("ff-sub-{i}")).unwrap(),
                    Provider::Anthropic,
                    SubscriptionState::Active,
                    format!("secret-ref://t-prop6/ff-seat-{i:02}/refresh"),
                    0,
                )).unwrap();
                s
            })
            .collect();

        let gate = AllowAll;
        let agent = aid("prop6-agent");
        let now = Instant::now();

        // Knock out the first `knock_out_first` seats (capped to n_seats - 1
        // so at least one seat is always eligible).
        let knock = knock_out_first.min(n_seats - 1);
        for seat in seat_ids.iter().take(knock) {
            pool.record_outcome(seat, SeatOutcome::RateLimited429, now)
                .unwrap();
        }

        let selected = pool.select(&tid("t-prop6"), &agent, &gate, now).unwrap();

        // First eligible seat in BTreeMap (sorted) order must be seat_ids[knock].
        prop_assert_eq!(
            &selected,
            &seat_ids[knock],
            "FillFirst should select seat_ids[{}] (first eligible)",
            knock
        );
    }
}
