#![allow(clippy::expect_used, clippy::panic)]

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use intelligence_kernel::{
    AgentId, AuthzDecision, AuthzGate, AuthzRequest, OAuthSubscription, Provider, QuotaWindow,
    QuotaWindowKind, SeatId, SeatOutcome, SelectionStrategy, StickyLeaseSpec, SubscriptionId,
    SubscriptionPool, SubscriptionState, TenantId, privacy_preserving_sticky_key,
};

struct AllowAll;
impl AuthzGate for AllowAll {
    fn decide(&self, _request: &AuthzRequest<'_>) -> AuthzDecision {
        AuthzDecision::Allow
    }
}

fn tenant() -> TenantId {
    TenantId::new("tenant-a").unwrap()
}

fn seat(id: &str) -> SeatId {
    SeatId::new(id).unwrap()
}

fn agent() -> AgentId {
    AgentId::new("agent-a").unwrap()
}

fn subscription(seat_id: &str, used_5h: u64, used_7d: u64, now: Instant) -> OAuthSubscription {
    OAuthSubscription::new(
        tenant(),
        seat(seat_id),
        SubscriptionId::new(format!("sub-{seat_id}")).unwrap(),
        Provider::Anthropic,
        SubscriptionState::Active,
        format!("secret-ref://tenant-a/anthropic/{seat_id}"),
        0,
    )
    .with_quota_windows([
        QuotaWindow::new(
            QuotaWindowKind::FiveHour,
            100,
            used_5h,
            now + Duration::from_secs(5 * 60 * 60),
            Duration::from_secs(5 * 60 * 60),
        ),
        QuotaWindow::new(
            QuotaWindowKind::Weekly,
            100,
            used_7d,
            now + Duration::from_secs(7 * 24 * 60 * 60),
            Duration::from_secs(7 * 24 * 60 * 60),
        ),
    ])
}

#[test]
fn headroom_uses_one_minus_max_five_hour_and_weekly_utilization() {
    let now = Instant::now();
    let mut pool = SubscriptionPool::new(
        tenant(),
        Provider::Anthropic,
        SelectionStrategy::TimeNormalizedQuotaPercent,
    );
    pool.add_seat(subscription("seat-a", 20, 70, now)).unwrap();

    let headroom = pool
        .seat_headroom(&seat("seat-a"), now, 0)
        .expect("headroom should be available");

    assert!(
        (headroom - 0.30).abs() < f64::EPSILON,
        "headroom={headroom}"
    );
}

#[test]
fn sticky_key_does_not_store_raw_prompt_and_rebinds_after_429() {
    let now = Instant::now();
    let pool_ref = Arc::new(Mutex::new(SubscriptionPool::new(
        tenant(),
        Provider::Anthropic,
        SelectionStrategy::RoundRobin,
    )));
    {
        let mut pool = pool_ref.lock().unwrap();
        pool.add_seat(subscription("seat-a", 10, 10, now)).unwrap();
        pool.add_seat(subscription("seat-b", 10, 10, now)).unwrap();
    }

    let key = privacy_preserving_sticky_key("raw user prompt must not be stored");
    assert!(!key.contains("raw user prompt"));

    let gate = AllowAll;
    let first = SubscriptionPool::lease_sticky_with_estimate(
        &pool_ref,
        &tenant(),
        &agent(),
        &gate,
        now,
        StickyLeaseSpec::new(&key, Duration::from_secs(60), 1),
    )
    .expect("first sticky lease");
    let first_seat = first.seat_id().clone();
    first.complete(SeatOutcome::Ok, now).unwrap();

    let second = SubscriptionPool::lease_sticky_with_estimate(
        &pool_ref,
        &tenant(),
        &agent(),
        &gate,
        now + Duration::from_secs(1),
        StickyLeaseSpec::new(&key, Duration::from_secs(60), 1),
    )
    .expect("second sticky lease");
    assert_eq!(second.seat_id(), &first_seat);
    second
        .complete(SeatOutcome::RateLimited429, now + Duration::from_secs(1))
        .unwrap();

    let rebound = SubscriptionPool::lease_sticky_with_estimate(
        &pool_ref,
        &tenant(),
        &agent(),
        &gate,
        now + Duration::from_secs(2),
        StickyLeaseSpec::new(&key, Duration::from_secs(60), 1),
    )
    .expect("sticky key should rebind after 429 cooldown");
    assert_ne!(rebound.seat_id(), &first_seat);
}
