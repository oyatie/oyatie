//! D1 — SubscriptionPool selection state machine (RoundRobin + FillFirst).
//!
//! Stage-4 RED: every test in this file MUST FAIL against the current kernel,
//! because `SubscriptionPool::select` returns `NotYetImplemented`.
//! Stage-5 GREEN replaces the placeholder with the real selection logic
//! described in ADR-0384 D1.
use std::time::Instant;

use intelligence_kernel::{
    AgentId, AuthzDecision, AuthzGate, AuthzRequest, OAuthSubscription, Provider, SeatId,
    SeatOutcome, SelectionStrategy, SubscriptionId, SubscriptionPool, SubscriptionPoolError,
    SubscriptionState, TenantId,
};

struct AllowAll;
impl AuthzGate for AllowAll {
    fn decide(&self, _request: &AuthzRequest<'_>) -> AuthzDecision {
        AuthzDecision::Allow
    }
}

fn tenant(s: &str) -> TenantId {
    TenantId::new(s).unwrap()
}
fn agent(s: &str) -> AgentId {
    AgentId::new(s).unwrap()
}
fn seat(s: &str) -> SeatId {
    SeatId::new(s).unwrap()
}
fn sub(s: &str) -> SubscriptionId {
    SubscriptionId::new(s).unwrap()
}

fn make_sub(tenant_str: &str, seat_str: &str) -> OAuthSubscription {
    OAuthSubscription::new(
        tenant(tenant_str),
        seat(seat_str),
        sub(&format!("{seat_str}-sub-1")),
        Provider::Anthropic,
        SubscriptionState::Active,
        format!("secret-ref://{tenant_str}/{seat_str}/refresh"),
        0,
    )
}

#[test]
fn round_robin_cycles_through_three_active_seats() {
    let mut pool = SubscriptionPool::new(
        tenant("t-1"),
        Provider::Anthropic,
        SelectionStrategy::RoundRobin,
    );
    pool.add_seat(make_sub("t-1", "seat-a")).unwrap();
    pool.add_seat(make_sub("t-1", "seat-b")).unwrap();
    pool.add_seat(make_sub("t-1", "seat-c")).unwrap();

    let gate = AllowAll;
    let now = Instant::now();
    let agent_a = agent("agent-1");

    let picks: Vec<SeatId> = (0..6)
        .map(|_| pool.select(&tenant("t-1"), &agent_a, &gate, now).unwrap())
        .collect();

    assert_eq!(picks[0], seat("seat-a"));
    assert_eq!(picks[1], seat("seat-b"));
    assert_eq!(picks[2], seat("seat-c"));
    assert_eq!(picks[3], seat("seat-a"));
    assert_eq!(picks[4], seat("seat-b"));
    assert_eq!(picks[5], seat("seat-c"));
}

#[test]
fn round_robin_skips_seat_in_cooldown() {
    let mut pool = SubscriptionPool::new(
        tenant("t-1"),
        Provider::Anthropic,
        SelectionStrategy::RoundRobin,
    );
    pool.add_seat(make_sub("t-1", "seat-a")).unwrap();
    pool.add_seat(make_sub("t-1", "seat-b")).unwrap();
    pool.add_seat(make_sub("t-1", "seat-c")).unwrap();

    let gate = AllowAll;
    let now = Instant::now();
    let agent_a = agent("agent-1");

    // Knock seat-b into cooldown
    pool.record_outcome(&seat("seat-b"), SeatOutcome::RateLimited429, now)
        .unwrap();

    let picks: Vec<SeatId> = (0..4)
        .map(|_| pool.select(&tenant("t-1"), &agent_a, &gate, now).unwrap())
        .collect();

    // Expect: a, c, a, c — seat-b skipped because cooldown.
    assert_eq!(picks[0], seat("seat-a"));
    assert_eq!(picks[1], seat("seat-c"));
    assert_eq!(picks[2], seat("seat-a"));
    assert_eq!(picks[3], seat("seat-c"));
}

#[test]
fn fill_first_sticks_to_seat_a_until_unavailable() {
    let mut pool = SubscriptionPool::new(
        tenant("t-1"),
        Provider::Anthropic,
        SelectionStrategy::FillFirst,
    );
    pool.add_seat(make_sub("t-1", "seat-a")).unwrap();
    pool.add_seat(make_sub("t-1", "seat-b")).unwrap();
    pool.add_seat(make_sub("t-1", "seat-c")).unwrap();

    let gate = AllowAll;
    let now = Instant::now();
    let agent_a = agent("agent-1");

    // Five picks with no failures — all should be seat-a (FillFirst).
    for _ in 0..5 {
        assert_eq!(
            pool.select(&tenant("t-1"), &agent_a, &gate, now).unwrap(),
            seat("seat-a")
        );
    }

    // Knock seat-a into cooldown. FillFirst should now pick seat-b until it
    // too becomes unavailable.
    pool.record_outcome(&seat("seat-a"), SeatOutcome::RateLimited429, now)
        .unwrap();

    for _ in 0..3 {
        assert_eq!(
            pool.select(&tenant("t-1"), &agent_a, &gate, now).unwrap(),
            seat("seat-b")
        );
    }
}

#[test]
fn pool_exhausted_returns_no_eligible_seat() {
    let mut pool = SubscriptionPool::new(
        tenant("t-1"),
        Provider::Anthropic,
        SelectionStrategy::RoundRobin,
    );
    pool.add_seat(make_sub("t-1", "seat-a")).unwrap();
    pool.add_seat(make_sub("t-1", "seat-b")).unwrap();

    let gate = AllowAll;
    let now = Instant::now();
    let agent_a = agent("agent-1");

    // Blacklist both seats by repeatedly failing them above threshold.
    for _ in 0..10 {
        let _ = pool.record_outcome(&seat("seat-a"), SeatOutcome::RefreshTokenRevoked, now);
        let _ = pool.record_outcome(&seat("seat-b"), SeatOutcome::RefreshTokenRevoked, now);
    }

    assert_eq!(
        pool.select(&tenant("t-1"), &agent_a, &gate, now),
        Err(SubscriptionPoolError::NoEligibleSeat)
    );
}

#[test]
fn empty_pool_returns_no_eligible_seat() {
    let mut pool = SubscriptionPool::new(
        tenant("t-1"),
        Provider::Anthropic,
        SelectionStrategy::RoundRobin,
    );
    let gate = AllowAll;
    let now = Instant::now();
    let agent_a = agent("agent-1");

    assert_eq!(
        pool.select(&tenant("t-1"), &agent_a, &gate, now),
        Err(SubscriptionPoolError::NoEligibleSeat)
    );
}
