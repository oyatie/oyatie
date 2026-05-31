#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};

use oya_intelligence_provider_pool_kernel::{
    AgentId, FailureKind, LeaseId, OAuthSubscription, PoolId, ProviderAccountId, ProviderFamily,
    SeatBlacklistReason, SeatId, SeatLease, SeatOutcome, SecretReference, SubscriptionId,
    SubscriptionPool, SubscriptionPoolError, SubscriptionPoolStrategy, SubscriptionSeatState,
    TenantId, UnixMillis, UsageSnapshot,
};
use proptest::prelude::*;
use proptest::test_runner::TestCaseResult;

fn aid(s: impl Into<String>) -> AgentId {
    AgentId(s.into())
}

fn pid(s: impl Into<String>) -> ProviderAccountId {
    ProviderAccountId(s.into())
}

fn seat(s: impl Into<String>) -> SeatId {
    SeatId(s.into())
}

fn sub(s: impl Into<String>) -> SubscriptionId {
    SubscriptionId(s.into())
}

fn secret(s: impl Into<String>) -> SecretReference {
    SecretReference::new(format!("sref://provider-subscriptions/{}", s.into()))
        .expect("test secret reference uses sref scheme")
}

fn oauth_subscription(s: &str) -> OAuthSubscription {
    OAuthSubscription::new(
        seat(s),
        pid(s),
        sub(format!("sub-{s}")),
        ProviderFamily::Claude,
        secret(s),
        SubscriptionSeatState::Active,
        UsageSnapshot::zero(),
    )
}

fn pool_with(strategy: SubscriptionPoolStrategy, seat_ids: &[&str]) -> SubscriptionPool {
    let mut pool = SubscriptionPool::new(
        PoolId("subscription-pool-proptest".into()),
        TenantId("tenant-proptest".into()),
        ProviderFamily::Claude,
        strategy,
    );
    for seat_id in seat_ids {
        pool.add_subscription(oauth_subscription(seat_id))
            .expect("unique test seat can be added");
    }
    pool
}

#[derive(Clone, Debug)]
enum OutcomeCase {
    Success { remaining_quota_pct: u8 },
    RateLimited,
    ServerError,
    Timeout,
    RefreshTokenExhausted,
    OperatorBlacklisted,
}

impl OutcomeCase {
    fn to_outcome(&self, now: UnixMillis) -> SeatOutcome {
        match self {
            Self::Success {
                remaining_quota_pct,
            } => SeatOutcome::Succeeded {
                usage: UsageSnapshot {
                    requests_in_window: 1,
                    remaining_quota_pct: *remaining_quota_pct,
                    last_used_unix_ms: now,
                    p99_latency_ms: 100,
                },
            },
            Self::RateLimited => SeatOutcome::RetryableFailure {
                kind: FailureKind::UpstreamRateLimit429,
            },
            Self::ServerError => SeatOutcome::RetryableFailure {
                kind: FailureKind::UpstreamServerError5xx,
            },
            Self::Timeout => SeatOutcome::RetryableFailure {
                kind: FailureKind::ConnectionTimeout,
            },
            Self::RefreshTokenExhausted => SeatOutcome::RefreshTokenExhausted,
            Self::OperatorBlacklisted => SeatOutcome::OperatorBlacklisted,
        }
    }
}

#[derive(Clone, Debug)]
enum PoolOp {
    Lease {
        agent_slot: u8,
        elapsed_ms: u16,
    },
    Complete {
        lease_slot: u8,
        outcome: OutcomeCase,
    },
    Advance {
        elapsed_ms: u32,
    },
}

fn arb_outcome_case() -> impl Strategy<Value = OutcomeCase> {
    prop_oneof![
        (5u8..=100u8).prop_map(|remaining_quota_pct| OutcomeCase::Success {
            remaining_quota_pct
        }),
        Just(OutcomeCase::RateLimited),
        Just(OutcomeCase::ServerError),
        Just(OutcomeCase::Timeout),
        Just(OutcomeCase::RefreshTokenExhausted),
        Just(OutcomeCase::OperatorBlacklisted),
    ]
}

fn arb_pool_op() -> impl Strategy<Value = PoolOp> {
    prop_oneof![
        (0u8..16u8, 0u16..20_000u16).prop_map(|(agent_slot, elapsed_ms)| PoolOp::Lease {
            agent_slot,
            elapsed_ms,
        }),
        (0u8..8u8, arb_outcome_case()).prop_map(|(lease_slot, outcome)| PoolOp::Complete {
            lease_slot,
            outcome,
        }),
        (0u32..1_000_000u32).prop_map(|elapsed_ms| PoolOp::Advance { elapsed_ms }),
    ]
}

fn assert_pool_invariants(pool: &SubscriptionPool, active: &[SeatLease]) -> TestCaseResult {
    prop_assert_eq!(
        pool.seat_count(),
        pool.reserved_count() + pool.free_count(),
        "seat_count must equal reserved_count plus free_count"
    );

    let mut reserved_by_seat: BTreeMap<SeatId, LeaseId> = BTreeMap::new();
    let mut reserved_lease_ids = BTreeSet::new();
    for (seat_id, subscription) in &pool.seats {
        prop_assert_eq!(
            seat_id,
            &subscription.seat_id,
            "seat map key must match subscription value"
        );
        if let SubscriptionSeatState::Reserved { lease_id, .. } = &subscription.state {
            prop_assert!(
                reserved_by_seat
                    .insert(seat_id.clone(), lease_id.clone())
                    .is_none(),
                "seat can be reserved once"
            );
            prop_assert!(
                reserved_lease_ids.insert(lease_id.clone()),
                "lease id can be active on one seat"
            );
        }
    }

    prop_assert_eq!(pool.reserved_count(), reserved_by_seat.len());
    prop_assert_eq!(reserved_by_seat.len(), active.len());

    let mut active_seats = BTreeSet::new();
    for lease in active {
        prop_assert!(
            active_seats.insert(lease.seat_id.clone()),
            "tracked active leases must hold distinct seats"
        );
        prop_assert_eq!(
            reserved_by_seat.get(&lease.seat_id),
            Some(&lease.lease_id),
            "tracked active lease must match reserved seat state"
        );
    }

    Ok(())
}

fn assert_pool_invariants_result(pool: &SubscriptionPool, active: &[SeatLease]) {
    assert_pool_invariants(pool, active).expect("pool invariants hold");
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 200,
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    #[test]
    fn subscription_pool_state_machine_preserves_lease_invariants(
        ops in proptest::collection::vec(arb_pool_op(), 1..80usize)
    ) {
        let mut pool = pool_with(SubscriptionPoolStrategy::RoundRobin, &["a", "b", "c", "d"]);
        let mut active: Vec<SeatLease> = Vec::new();
        let mut now = UnixMillis(1_000);

        for op in ops {
            match op {
                PoolOp::Lease { agent_slot, elapsed_ms } => {
                    now = UnixMillis(now.0.saturating_add(u64::from(elapsed_ms)));
                    match pool.lease(aid(format!("agent-{agent_slot}")), now) {
                        Ok(lease) => active.push(lease),
                        Err(SubscriptionPoolError::PoolExhausted) => {}
                        Err(err) => panic!("unexpected lease error: {err:?}"),
                    }
                }
                PoolOp::Complete { lease_slot, outcome } => {
                    if !active.is_empty() {
                        let index = usize::from(lease_slot) % active.len();
                        let lease = active.swap_remove(index);
                        pool.complete(lease, outcome.to_outcome(now), now)
                            .expect("tracked active lease completes exactly once");
                    }
                }
                PoolOp::Advance { elapsed_ms } => {
                    now = UnixMillis(now.0.saturating_add(u64::from(elapsed_ms)));
                }
            }

            assert_pool_invariants(&pool, &active)?;
        }
    }

    #[test]
    fn subscription_pool_failure_count_only_resets_on_success(
        outcomes in proptest::collection::vec(arb_outcome_case(), 1..24usize)
    ) {
        let mut pool = pool_with(SubscriptionPoolStrategy::RoundRobin, &["a"]);
        let mut now = UnixMillis(1_000);
        let mut expected_failures = 0u32;
        let active: Vec<SeatLease> = Vec::new();

        for outcome in outcomes {
            let lease = match pool.lease(aid("agent"), now) {
                Ok(lease) => lease,
                Err(SubscriptionPoolError::PoolExhausted) => {
                    assert_pool_invariants(&pool, &active)?;
                    continue;
                }
                Err(err) => panic!("unexpected lease error: {err:?}"),
            };

            let success = matches!(outcome, OutcomeCase::Success { .. });
            pool.complete(lease, outcome.to_outcome(now), now)
                .expect("newly issued lease completes");

            if success {
                expected_failures = 0;
            } else if !matches!(outcome, OutcomeCase::RefreshTokenExhausted | OutcomeCase::OperatorBlacklisted) {
                expected_failures = expected_failures.saturating_add(1);
            }

            let subscription = pool.seats.get(&seat("a")).expect("test seat exists");
            prop_assert_eq!(subscription.failure_count, expected_failures);
            if expected_failures > 5 {
                prop_assert_eq!(
                    pool.seat_state(&seat("a")),
                    Some(&SubscriptionSeatState::Blacklisted {
                        reason: SeatBlacklistReason::RepeatedFailuresExceededThreshold {
                            failure_count: expected_failures,
                        },
                    })
                );
            }
            assert_pool_invariants(&pool, &active)?;
            now = UnixMillis(now.0.saturating_add(1_000_000));
        }
    }
}

#[test]
fn subscription_pool_rejects_forged_lease_for_property_harness_baseline() {
    let mut pool = pool_with(SubscriptionPoolStrategy::RoundRobin, &["a"]);
    let issued = pool.lease(aid("agent-1"), UnixMillis(1)).unwrap();
    let forged = SeatLease {
        lease_id: LeaseId("forged".into()),
        seat_id: issued.seat_id.clone(),
        provider_account_id: issued.provider_account_id.clone(),
        agent_id: issued.agent_id.clone(),
        leased_at_unix_ms: issued.leased_at_unix_ms,
    };

    assert_eq!(
        pool.complete(
            forged,
            SeatOutcome::Succeeded {
                usage: UsageSnapshot::zero(),
            },
            UnixMillis(2),
        )
        .unwrap_err(),
        SubscriptionPoolError::LeaseSeatMismatch {
            seat_id: seat("a"),
            lease_id: LeaseId("forged".into()),
        }
    );
    assert_pool_invariants_result(&pool, &[issued]);
}
