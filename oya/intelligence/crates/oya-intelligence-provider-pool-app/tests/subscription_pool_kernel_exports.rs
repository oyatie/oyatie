#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_intelligence_provider_pool_app::{
    AgentId, OAuthSubscription, PoolId, ProviderAccountId, ProviderFamily, SeatId, SeatOutcome,
    SecretReference, SubscriptionId, SubscriptionPool, SubscriptionPoolStrategy,
    SubscriptionSeatState, TenantId, UnixMillis, UsageSnapshot,
};

fn secret(name: &str) -> SecretReference {
    SecretReference::new(format!("sref://provider-subscriptions/{name}")).unwrap()
}

#[test]
fn app_reexports_subscription_pool_kernel_for_composition_lanes() {
    let mut pool = SubscriptionPool::new(
        PoolId("subscription-pool".into()),
        TenantId("tenant-1".into()),
        ProviderFamily::Claude,
        SubscriptionPoolStrategy::RoundRobin,
    );
    pool.add_subscription(OAuthSubscription::new(
        SeatId("seat-a".into()),
        ProviderAccountId("provider-account-a".into()),
        SubscriptionId("subscription-a".into()),
        ProviderFamily::Claude,
        secret("seat-a"),
        SubscriptionSeatState::Active,
        UsageSnapshot::zero(),
    ))
    .unwrap();

    let lease = pool
        .lease(AgentId("agent-1".into()), UnixMillis(1_000))
        .unwrap();
    assert_eq!(lease.seat_id, SeatId("seat-a".into()));
    assert_eq!(pool.reserved_count(), 1);

    pool.complete(
        lease,
        SeatOutcome::Succeeded {
            usage: UsageSnapshot {
                requests_in_window: 1,
                remaining_quota_pct: 97,
                last_used_unix_ms: UnixMillis(1_100),
                p99_latency_ms: 42,
            },
        },
        UnixMillis(1_100),
    )
    .unwrap();

    assert_eq!(pool.reserved_count(), 0);
    assert_eq!(pool.free_count(), 1);
    assert_eq!(
        pool.usage_snapshot(&SeatId("seat-a".into()))
            .unwrap()
            .remaining_quota_pct,
        97
    );
}
