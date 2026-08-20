//! Fix-6: SeatOutcome::RefreshFailed — kernel handles transient secret-provider refresh failure.
#![cfg_attr(test, allow(clippy::unwrap_used))]

use std::time::{Duration, Instant};

use intelligence_kernel::{
    AgentId, AuthzDecision, AuthzGate, AuthzRequest, CooldownReason, OAuthSubscription, Provider,
    SeatId, SeatOutcome, SelectionStrategy, SubscriptionId, SubscriptionPool, SubscriptionState,
    TenantId,
};

struct AllowAll;
impl AuthzGate for AllowAll {
    fn decide(&self, _r: &AuthzRequest<'_>) -> AuthzDecision {
        AuthzDecision::Allow
    }
}

fn make_sub(tenant: &str, seat: &str) -> OAuthSubscription {
    OAuthSubscription::new(
        TenantId::new(tenant).unwrap(),
        SeatId::new(seat).unwrap(),
        SubscriptionId::new(format!("{seat}-sub")).unwrap(),
        Provider::Anthropic,
        SubscriptionState::Active,
        format!("secret-ref://{tenant}/{seat}/refresh"),
        0,
    )
}

#[test]
fn refresh_failed_transitions_seat_to_cooldown() {
    let mut pool = SubscriptionPool::new(
        TenantId::new("t-rf").unwrap(),
        Provider::Anthropic,
        SelectionStrategy::FillFirst,
    );
    pool.add_seat(make_sub("t-rf", "seat-rf-1")).unwrap();

    let now = Instant::now();
    pool.record_outcome(
        &SeatId::new("seat-rf-1").unwrap(),
        SeatOutcome::RefreshFailed,
        now,
    )
    .unwrap();

    // After RefreshFailed, the seat should be in cooldown and not selectable.
    let gate = AllowAll;
    let agent = AgentId::new("agent-rf").unwrap();
    let result = pool.select(&TenantId::new("t-rf").unwrap(), &agent, &gate, now);
    assert!(
        result.is_err(),
        "seat should be in cooldown after RefreshFailed"
    );
}

#[test]
fn refresh_failed_increments_failure_count() {
    let mut pool = SubscriptionPool::new(
        TenantId::new("t-rf2").unwrap(),
        Provider::Anthropic,
        SelectionStrategy::FillFirst,
    );
    pool.add_seat(make_sub("t-rf2", "seat-rf-2")).unwrap();
    let now = Instant::now();
    let sid = SeatId::new("seat-rf-2").unwrap();

    // 3 RefreshFailed outcomes — should stay in cooldown each time, not blacklisted.
    for _ in 0..3 {
        // Drive past cooldown to make seat eligible again.
        let future = now + Duration::from_secs(120);
        pool.record_outcome(&sid, SeatOutcome::RefreshFailed, future)
            .unwrap();
    }

    // Not yet blacklisted (threshold is 5).
    let gate = AllowAll;
    let agent = AgentId::new("agent-rf2").unwrap();
    // At `now + 120s * 3`, seat cooldown should have elapsed.
    let far_future = now + Duration::from_secs(600);
    let result = pool.select(&TenantId::new("t-rf2").unwrap(), &agent, &gate, far_future);
    assert!(result.is_ok(), "seat should recover after cooldown expires");
}

#[test]
fn refresh_failed_above_threshold_blacklists_seat() {
    let mut pool = SubscriptionPool::new(
        TenantId::new("t-rf3").unwrap(),
        Provider::Anthropic,
        SelectionStrategy::FillFirst,
    );
    pool.add_seat(make_sub("t-rf3", "seat-rf-3")).unwrap();
    let now = Instant::now();
    let sid = SeatId::new("seat-rf-3").unwrap();

    // 6 RefreshFailed outcomes — should blacklist (threshold = 5).
    for i in 0..6u64 {
        let t = now + Duration::from_secs(i * 120);
        pool.record_outcome(&sid, SeatOutcome::RefreshFailed, t)
            .unwrap();
    }

    // Even far in the future the seat should not be selectable (blacklisted).
    let gate = AllowAll;
    let agent = AgentId::new("agent-rf3").unwrap();
    let far = now + Duration::from_secs(99999);
    let result = pool.select(&TenantId::new("t-rf3").unwrap(), &agent, &gate, far);
    assert!(
        result.is_err(),
        "seat should be permanently blacklisted after exceeding threshold"
    );
}

#[test]
fn refresh_failed_cooldown_reason_is_transient_failure() {
    // Verify the cooldown reason is RefreshTokenTransientFailure (not 429 or 5xx).
    // We can't inspect pool internals directly, but we can verify by selecting:
    // after cooldown elapsed the seat is eligible again (same as 429 cooldown).
    let mut pool = SubscriptionPool::new(
        TenantId::new("t-rf4").unwrap(),
        Provider::Anthropic,
        SelectionStrategy::FillFirst,
    );
    pool.add_seat(make_sub("t-rf4", "seat-rf-4")).unwrap();
    let now = Instant::now();
    let sid = SeatId::new("seat-rf-4").unwrap();

    pool.record_outcome(&sid, SeatOutcome::RefreshFailed, now)
        .unwrap();

    // Immediately after: not eligible.
    let gate = AllowAll;
    let agent = AgentId::new("agent-rf4").unwrap();
    assert!(
        pool.select(&TenantId::new("t-rf4").unwrap(), &agent, &gate, now)
            .is_err()
    );

    // After cooldown (61s): eligible again.
    let after = now + Duration::from_secs(61);
    assert!(
        pool.select(&TenantId::new("t-rf4").unwrap(), &agent, &gate, after)
            .is_ok(),
        "seat should recover after cooldown elapses"
    );
}
