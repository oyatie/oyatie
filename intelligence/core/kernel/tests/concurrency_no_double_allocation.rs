//! Fix-1: SeatLease — same-seat concurrency guard.
//! N tokio tasks all call SubscriptionPool::lease concurrently; asserts that
//! no seat_id is returned to two tasks simultaneously.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use intelligence_kernel::{
    AgentId, AuthzDecision, AuthzGate, AuthzRequest, OAuthSubscription, Provider, SeatId,
    SeatOutcome, SelectionStrategy, SubscriptionId, SubscriptionPool, SubscriptionPoolError,
    SubscriptionState, TenantId,
};

struct AllowAll;
impl AuthzGate for AllowAll {
    fn decide(&self, _r: &AuthzRequest<'_>) -> AuthzDecision {
        AuthzDecision::Allow
    }
}

fn make_pool(n_seats: usize) -> Arc<Mutex<SubscriptionPool>> {
    let tenant = TenantId::new("t-concurrent").unwrap();
    let mut pool = SubscriptionPool::new(
        tenant.clone(),
        Provider::Anthropic,
        SelectionStrategy::RoundRobin,
    );
    for i in 0..n_seats {
        pool.add_seat(OAuthSubscription::new(
            tenant.clone(),
            SeatId::new(format!("seat-{i}")).unwrap(),
            SubscriptionId::new(format!("sub-{i}")).unwrap(),
            Provider::Anthropic,
            SubscriptionState::Active,
            format!("secret-ref://t-concurrent/seat-{i}/refresh"),
            0,
        ))
        .unwrap();
    }
    Arc::new(Mutex::new(pool))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn concurrent_leases_never_duplicate_seat() {
    // 3 seats, 12 concurrent tasks — each task holds the lease 1ms then completes.
    let pool_ref = make_pool(3);
    let gate = Arc::new(AllowAll);
    let agent = AgentId::new("agent-concurrent").unwrap();

    // Track concurrent holders per seat.
    let in_flight: Arc<Mutex<HashMap<String, u32>>> = Arc::new(Mutex::new(HashMap::new()));
    let violation = Arc::new(Mutex::new(false));

    let mut handles = Vec::new();
    for _ in 0..12 {
        let pool_ref = Arc::clone(&pool_ref);
        let gate = Arc::clone(&gate);
        let agent = agent.clone();
        let in_flight = Arc::clone(&in_flight);
        let violation = Arc::clone(&violation);

        handles.push(tokio::spawn(async move {
            let now = Instant::now();
            let lease = SubscriptionPool::lease(
                &pool_ref,
                &TenantId::new("t-concurrent").unwrap(),
                &agent,
                gate.as_ref(),
                now,
            );
            match lease {
                Ok(lease) => {
                    let seat_key = lease.seat_id().as_str().to_string();
                    // Mark seat as in-flight; check for double allocation.
                    {
                        let mut map = in_flight.lock().unwrap();
                        let entry = map.entry(seat_key.clone()).or_insert(0);
                        *entry += 1;
                        if *entry > 1 {
                            *violation.lock().unwrap() = true;
                        }
                    }
                    // Simulate short work.
                    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                    // Release.
                    {
                        let mut map = in_flight.lock().unwrap();
                        let entry = map.entry(seat_key).or_insert(1);
                        *entry = entry.saturating_sub(1);
                    }
                    lease.complete(SeatOutcome::Ok, Instant::now()).unwrap();
                }
                Err(SubscriptionPoolError::NoEligibleSeat) => {
                    // All seats leased — acceptable under heavy concurrency.
                }
                Err(e) => panic!("unexpected error: {e:?}"),
            }
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    assert!(
        !*violation.lock().unwrap(),
        "same seat was leased to two tasks concurrently"
    );
}

#[tokio::test]
async fn lease_drop_without_complete_releases_seat() {
    // One seat. Acquire lease, drop it without completing, then lease again —
    // should succeed (seat is released by Drop).
    let pool_ref = make_pool(1);
    let gate = AllowAll;
    let agent = AgentId::new("agent-drop").unwrap();
    let now = Instant::now();

    let lease = SubscriptionPool::lease(
        &pool_ref,
        &TenantId::new("t-concurrent").unwrap(),
        &agent,
        &gate,
        now,
    )
    .expect("first lease");
    let seat = lease.seat_id().clone();
    drop(lease); // Drop without complete — should release.

    // Short sleep to let Drop's record_outcome flush.
    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;

    // Drop records Released (no-op outcome) — seat has NO penalty, stays Active.
    let pool = pool_ref.lock().unwrap();
    // seat_count() is still 1 (not removed from seats map).
    assert_eq!(pool.seat_count(), 1);
    drop(pool);

    // Verify seat was returned without penalty: leasing again at `now` must succeed
    // because Released applies no cooldown/blacklist to the seat.
    let result = SubscriptionPool::lease(
        &pool_ref,
        &TenantId::new("t-concurrent").unwrap(),
        &agent,
        &gate,
        now,
    );
    assert!(
        result.is_ok(),
        "seat must be leasable again after drop-without-complete (Released = no penalty)"
    );
    drop(seat);
}

#[test]
fn lease_complete_records_outcome() {
    use std::sync::Mutex;
    let pool_ref = make_pool(2);
    let gate = AllowAll;
    let agent = AgentId::new("agent-outcome").unwrap();
    let now = Instant::now();

    let lease = SubscriptionPool::lease(
        &pool_ref,
        &TenantId::new("t-concurrent").unwrap(),
        &agent,
        &gate,
        now,
    )
    .unwrap();
    let sid = lease.seat_id().clone();
    lease.complete(SeatOutcome::RateLimited429, now).unwrap();

    // Seat should now be in Cooldown, not Active.
    let pool = pool_ref.lock().unwrap();
    // Verify via select: if seat is in cooldown and other seat is available,
    // select should return the other seat.
    drop(pool);

    // Try leasing again; if cooldown, it picks the other seat.
    let gate2 = AllowAll;
    let lease2 = SubscriptionPool::lease(
        &pool_ref,
        &TenantId::new("t-concurrent").unwrap(),
        &agent,
        &gate2,
        now,
    )
    .unwrap();
    assert_ne!(
        lease2.seat_id(),
        &sid,
        "rate-limited seat should not be leased again immediately"
    );
    lease2.complete(SeatOutcome::Ok, now).unwrap();
}
