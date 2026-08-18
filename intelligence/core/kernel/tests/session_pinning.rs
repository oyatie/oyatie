//! Session-pinning seam: dual-format sticky-key extraction, the 6h TTL,
//! rebind-on-429 failover, and the `tenant::provider::session::model` cache key, wired
//! through the real [`SubscriptionPool`] lease path.
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use intelligence_kernel::{
    AgentId, AuthzDecision, AuthzGate, AuthzRequest, OAuthSubscription, Provider, SeatId,
    SeatOutcome, SelectionStrategy, StickyLeaseSpec, SubscriptionId, SubscriptionPool,
    SubscriptionState, TenantId, derive_sticky_key, prompt_cache_key,
};

const SESSION_TTL: Duration = Duration::from_secs(6 * 60 * 60);

struct AllowAll;
impl AuthzGate for AllowAll {
    fn decide(&self, _request: &AuthzRequest<'_>) -> AuthzDecision {
        AuthzDecision::Allow
    }
}

fn tenant() -> TenantId {
    TenantId::new("tenant-a").unwrap()
}
fn agent() -> AgentId {
    AgentId::new("agent-a").unwrap()
}
fn seat(id: &str) -> SeatId {
    SeatId::new(id).unwrap()
}

fn sub(seat_id: &str) -> OAuthSubscription {
    OAuthSubscription::new(
        tenant(),
        seat(seat_id),
        SubscriptionId::new(format!("sub-{seat_id}")).unwrap(),
        Provider::Anthropic,
        SubscriptionState::Active,
        format!("secret-ref://tenant-a/anthropic/{seat_id}"),
        0,
    )
}

fn two_seat_pool(now: Instant) -> Arc<Mutex<SubscriptionPool>> {
    let pool = Arc::new(Mutex::new(SubscriptionPool::new(
        tenant(),
        Provider::Anthropic,
        SelectionStrategy::RoundRobin,
    )));
    {
        let mut p = pool.lock().unwrap();
        p.add_seat(sub("seat-a")).unwrap();
        p.add_seat(sub("seat-b")).unwrap();
    }
    let _ = now;
    pool
}

#[test]
fn wire_session_id_pins_to_one_seat_for_the_default_ttl() {
    let now = Instant::now();
    let pool = two_seat_pool(now);
    let gate = AllowAll;

    let key = derive_sticky_key(Some("wire-conv-7"), None).expect("wire id present");
    assert!(key.starts_with("wsid:"));
    assert!(!key.contains("wire-conv-7"));

    let first = SubscriptionPool::lease_sticky_with_estimate(
        &pool,
        &tenant(),
        &agent(),
        &gate,
        now,
        StickyLeaseSpec::new(&key, SESSION_TTL, 1),
    )
    .expect("first lease");
    let pinned = first.seat_id().clone();
    first.complete(SeatOutcome::Ok, now).unwrap();

    // Just under the 6h TTL: same seat.
    let later = now + SESSION_TTL - Duration::from_secs(1);
    let again = SubscriptionPool::lease_sticky_with_estimate(
        &pool,
        &tenant(),
        &agent(),
        &gate,
        later,
        StickyLeaseSpec::new(&key, SESSION_TTL, 1),
    )
    .expect("lease within ttl");
    assert_eq!(again.seat_id(), &pinned, "pin must hold within the 6h TTL");
    again.complete(SeatOutcome::Ok, later).unwrap();
}

#[test]
fn rebind_on_429_failover_moves_off_the_cooling_seat() {
    let now = Instant::now();
    let pool = two_seat_pool(now);
    let gate = AllowAll;
    let key = derive_sticky_key(Some("conv-rebind"), None).unwrap();

    let first = SubscriptionPool::lease_sticky_with_estimate(
        &pool,
        &tenant(),
        &agent(),
        &gate,
        now,
        StickyLeaseSpec::new(&key, SESSION_TTL, 1),
    )
    .unwrap();
    let pinned = first.seat_id().clone();
    // 429 on the pinned seat drops the binding and cools the seat.
    first.complete(SeatOutcome::RateLimited429, now).unwrap();

    let rebound = SubscriptionPool::lease_sticky_with_estimate(
        &pool,
        &tenant(),
        &agent(),
        &gate,
        now + Duration::from_secs(1),
        StickyLeaseSpec::new(&key, SESSION_TTL, 1),
    )
    .expect("rebind after 429");
    assert_ne!(
        rebound.seat_id(),
        &pinned,
        "429 failover must rebind to a different seat"
    );
}

#[test]
fn message_fingerprint_path_pins_when_no_wire_id() {
    let now = Instant::now();
    let pool = two_seat_pool(now);
    let gate = AllowAll;
    let key = derive_sticky_key(None, Some("first user message body")).unwrap();
    assert!(!key.contains("first user message body"));

    let first = SubscriptionPool::lease_sticky_with_estimate(
        &pool,
        &tenant(),
        &agent(),
        &gate,
        now,
        StickyLeaseSpec::new(&key, SESSION_TTL, 1),
    )
    .unwrap();
    let pinned = first.seat_id().clone();
    first.complete(SeatOutcome::Ok, now).unwrap();

    let again = SubscriptionPool::lease_sticky_with_estimate(
        &pool,
        &tenant(),
        &agent(),
        &gate,
        now + Duration::from_secs(5),
        StickyLeaseSpec::new(&key, SESSION_TTL, 1),
    )
    .unwrap();
    assert_eq!(again.seat_id(), &pinned);
    again.complete(SeatOutcome::Ok, now).unwrap();
}

#[test]
fn cache_key_namespaces_provider_session_and_model() {
    let key = derive_sticky_key(Some("conv-9"), None).unwrap();
    assert_eq!(
        prompt_cache_key(&tenant(), Provider::Anthropic, &key, "claude-sonnet-4"),
        format!(
            "v1:t8:tenant-ap9:anthropics{}:{}m15:claude-sonnet-4",
            key.len(),
            key
        )
    );
    // Distinct tenant => distinct cache slot (no cross-tenant cache reuse).
    assert_ne!(
        prompt_cache_key(
            &TenantId::new("tenant-a").unwrap(),
            Provider::Anthropic,
            &key,
            "m"
        ),
        prompt_cache_key(
            &TenantId::new("tenant-b").unwrap(),
            Provider::Anthropic,
            &key,
            "m"
        )
    );
    // Distinct model => distinct cache slot (no cross-model cache reuse).
    assert_ne!(
        prompt_cache_key(&tenant(), Provider::Anthropic, &key, "claude-opus-4"),
        prompt_cache_key(&tenant(), Provider::Anthropic, &key, "claude-sonnet-4")
    );
    // Distinct provider => distinct cache slot.
    assert_ne!(
        prompt_cache_key(&tenant(), Provider::Anthropic, &key, "m"),
        prompt_cache_key(&tenant(), Provider::Codex, &key, "m")
    );
}
