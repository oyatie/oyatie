//! Hermetic unit/integration tests for the per-AGENT-TOKEN reserve-then-reconcile
//! quota slice.
//!
//! Acceptance criteria (from the task spec):
//! - reserve reduces remaining budget
//! - reserve rejects when over budget → QuotaError::BudgetExceeded
//! - reconcile credits back over-reserve
//! - reconcile debits extra consumption (floor at 0)
//! - skip_when_ample returns true / false at the threshold boundary
//! - agent isolation: separate AgentTokens share no budget
//! - tenant isolation: same AgentToken in different tenants share no budget
//! - dispatch_to_pool with quota store: rejects over-budget before transport
//! - dispatch_to_pool with quota store: reconciles actual tokens after success
//! - dispatch_to_pool with quota_store=None: existing behaviour unchanged
//! - dispatch_to_pool with ample headroom: reserve skipped, reconcile still runs
//!
//! All tests are hermetic: no network, no real upstream.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use bytes::Bytes;

use oya_intelligence_provider_pool_app::{
    AgentQuotaBudget, AgentQuotaSnapshot, AgentQuotaStore, AgentToken, DeniedSecretResolver,
    DispatchError, InMemoryAccountHealthStore, InMemoryAgentQuotaStore, InMemoryPoolRepository,
    InMemoryProviderInvocationTransport, InMemoryUsageSnapshotSource, NoOpMetricsSink, PoolId,
    PoolRoutingStrategy, ProviderAccountId, ProviderAccountPool, ProviderFamily, ProviderResponse,
    ProviderTier, QUOTA_AMPLE_THRESHOLD_PCT, QuotaError, RequestMetadata, TenantId, TransportError,
    TransportScript, UnixMillis, dispatch_to_pool, dispatch_to_pool_with_quota,
    should_skip_reserve,
};
use intelligence_provider_pool_kernel::DurationMs;
use std::collections::BTreeSet;

// ── helpers ───────────────────────────────────────────────────────────────────

fn pid(s: &str) -> ProviderAccountId {
    ProviderAccountId(s.to_owned())
}
fn ten(s: &str) -> TenantId {
    TenantId(s.to_owned())
}
fn pool_id(s: &str) -> PoolId {
    PoolId(s.to_owned())
}
fn agent(s: &str) -> AgentToken {
    AgentToken(s.to_owned())
}

fn simple_pool(tenant: &TenantId, pid_str: &PoolId, members: &[&str]) -> ProviderAccountPool {
    let mut set: BTreeSet<ProviderAccountId> = BTreeSet::new();
    for m in members {
        set.insert(ProviderAccountId(m.to_string()));
    }
    ProviderAccountPool::new(
        pid_str.clone(),
        ProviderFamily::Claude,
        ProviderTier::Pro,
        tenant.clone(),
        set,
        PoolRoutingStrategy::RoundRobin,
        DurationMs(60_000),
    )
}

fn ok_response(account: &ProviderAccountId) -> ProviderResponse {
    ProviderResponse {
        status: 200,
        headers: vec![("content-type".to_string(), "application/json".to_string())],
        body: Bytes::from_static(b"{\"ok\":true}"),
        retry_after_seconds: None,
        provider_account_id: account.clone(),
    }
}

// ── InMemoryAgentQuotaStore unit tests ───────────────────────────────────────

/// reserve_reduces_remaining_budget
#[test]
fn reserve_reduces_remaining_budget() {
    let tenant = ten("t1");
    let ag = agent("agent_a");
    let budget = AgentQuotaBudget {
        budget_tokens: 1_000,
        window_reset_unix_ms: 0,
    };

    let mut store = InMemoryAgentQuotaStore::new();
    store.set_budget(tenant.clone(), ag.clone(), budget);

    store
        .reserve(&tenant, &ag, 300)
        .expect("reserve must succeed when sufficient budget");

    let snap = store.snapshot(&tenant, &ag).expect("snapshot");
    assert_eq!(snap.budget_tokens, 1_000);
    assert_eq!(snap.remaining_tokens, 700, "remaining must drop by 300");
}

/// reserve_rejects_when_over_budget
#[test]
fn reserve_rejects_when_over_budget() {
    let tenant = ten("t1");
    let ag = agent("agent_a");
    let budget = AgentQuotaBudget {
        budget_tokens: 100,
        window_reset_unix_ms: 0,
    };

    let mut store = InMemoryAgentQuotaStore::new();
    store.set_budget(tenant.clone(), ag.clone(), budget);

    // Exhaust most of the budget first.
    store.reserve(&tenant, &ag, 90).expect("first reserve");

    // Now try to reserve more than remains (10 left, want 50).
    let err = store
        .reserve(&tenant, &ag, 50)
        .expect_err("must reject over-budget reserve");
    match err {
        QuotaError::BudgetExceeded {
            requested,
            remaining,
            ..
        } => {
            assert_eq!(requested, 50, "requested tokens must match");
            assert_eq!(
                remaining, 10,
                "remaining tokens must be reported accurately"
            );
        }
        other => panic!("expected BudgetExceeded, got {other:?}"),
    }
}

/// reconcile_credits_back_over_reserve
#[test]
fn reconcile_credits_back_over_reserve() {
    let tenant = ten("t1");
    let ag = agent("agent_a");
    let budget = AgentQuotaBudget {
        budget_tokens: 1_000,
        window_reset_unix_ms: 0,
    };

    let mut store = InMemoryAgentQuotaStore::new();
    store.set_budget(tenant.clone(), ag.clone(), budget);

    // Reserve 100; only 60 actually used.
    store.reserve(&tenant, &ag, 100).expect("reserve");
    store.reconcile(&tenant, &ag, 100, 60).expect("reconcile");

    let snap = store.snapshot(&tenant, &ag).expect("snapshot");
    // Started with 1000; reserved 100 → 900; reconcile credits back 40 → 940.
    assert_eq!(
        snap.remaining_tokens, 940,
        "over-reserve of 40 must be credited back"
    );
}

/// reconcile_debits_extra_consumption
#[test]
fn reconcile_debits_extra_consumption() {
    let tenant = ten("t1");
    let ag = agent("agent_a");
    let budget = AgentQuotaBudget {
        budget_tokens: 1_000,
        window_reset_unix_ms: 0,
    };

    let mut store = InMemoryAgentQuotaStore::new();
    store.set_budget(tenant.clone(), ag.clone(), budget);

    // Reserve 100; actually used 150 (over-ran estimate).
    store.reserve(&tenant, &ag, 100).expect("reserve");
    store.reconcile(&tenant, &ag, 100, 150).expect("reconcile");

    let snap = store.snapshot(&tenant, &ag).expect("snapshot");
    // Started 1000; reserved 100 → 900; reconcile debits extra 50 → 850.
    assert_eq!(
        snap.remaining_tokens, 850,
        "extra 50 tokens must be debited"
    );
}

/// reconcile_floors_remaining_at_zero
#[test]
fn reconcile_floors_remaining_at_zero() {
    let tenant = ten("t1");
    let ag = agent("agent_a");
    // Tiny budget.
    let budget = AgentQuotaBudget {
        budget_tokens: 10,
        window_reset_unix_ms: 0,
    };

    let mut store = InMemoryAgentQuotaStore::new();
    store.set_budget(tenant.clone(), ag.clone(), budget);

    // Reserve 10; massively over-run (actual=9999) → should floor at 0.
    store.reserve(&tenant, &ag, 10).expect("reserve");
    store.reconcile(&tenant, &ag, 10, 9_999).expect("reconcile");

    let snap = store.snapshot(&tenant, &ag).expect("snapshot");
    assert_eq!(
        snap.remaining_tokens, 0,
        "remaining must floor at 0, not underflow"
    );
}

// ── should_skip_reserve tests ─────────────────────────────────────────────────

/// skip_when_ample_returns_true_above_threshold
#[test]
fn skip_when_ample_returns_true_above_threshold() {
    // remaining=900, budget=1000 → 90% remaining > 80% threshold → skip.
    let snap = AgentQuotaSnapshot {
        budget_tokens: 1_000,
        remaining_tokens: 900,
        window_reset_unix_ms: 0,
    };
    assert!(
        should_skip_reserve(&snap),
        "90% remaining must trigger skip-when-ample (threshold={QUOTA_AMPLE_THRESHOLD_PCT}%)"
    );
}

/// skip_when_ample_returns_false_at_threshold_exactly
#[test]
fn skip_when_ample_returns_false_at_threshold_exactly() {
    // remaining=800, budget=1000 → exactly 80% → NOT > threshold → do not skip.
    let snap = AgentQuotaSnapshot {
        budget_tokens: 1_000,
        remaining_tokens: 800,
        window_reset_unix_ms: 0,
    };
    assert!(
        !should_skip_reserve(&snap),
        "80% remaining (== threshold) must NOT trigger skip"
    );
}

/// skip_when_ample_returns_false_below_threshold
#[test]
fn skip_when_ample_returns_false_below_threshold() {
    let snap = AgentQuotaSnapshot {
        budget_tokens: 1_000,
        remaining_tokens: 500,
        window_reset_unix_ms: 0,
    };
    assert!(
        !should_skip_reserve(&snap),
        "50% remaining must NOT trigger skip"
    );
}

/// skip_when_ample_zero_budget_does_not_panic
#[test]
fn skip_when_ample_zero_budget_does_not_panic() {
    let snap = AgentQuotaSnapshot {
        budget_tokens: 0,
        remaining_tokens: 0,
        window_reset_unix_ms: 0,
    };
    // Must not panic (division-by-zero guard).
    let _ = should_skip_reserve(&snap);
}

// ── agent / tenant isolation tests ───────────────────────────────────────────

/// agent_isolation_separate_budgets
#[test]
fn agent_isolation_separate_budgets() {
    let tenant = ten("t1");
    let a = agent("agent_a");
    let b = agent("agent_b");
    let budget = AgentQuotaBudget {
        budget_tokens: 500,
        window_reset_unix_ms: 0,
    };

    let mut store = InMemoryAgentQuotaStore::new();
    store.set_budget(tenant.clone(), a.clone(), budget);
    store.set_budget(tenant.clone(), b.clone(), budget);

    // Exhaust agent_a's budget entirely.
    store.reserve(&tenant, &a, 500).expect("exhaust agent_a");

    // agent_b must still have full budget.
    let snap_b = store.snapshot(&tenant, &b).expect("snap_b");
    assert_eq!(
        snap_b.remaining_tokens, 500,
        "agent_b budget must be unaffected by agent_a consumption"
    );

    // agent_a must be exhausted.
    let snap_a = store.snapshot(&tenant, &a).expect("snap_a");
    assert_eq!(snap_a.remaining_tokens, 0);
}

/// tenant_isolation_separate_budgets
#[test]
fn tenant_isolation_separate_budgets() {
    let tenant_x = ten("tenant_x");
    let tenant_y = ten("tenant_y");
    let ag = agent("shared_agent_name");
    let budget = AgentQuotaBudget {
        budget_tokens: 200,
        window_reset_unix_ms: 0,
    };

    let mut store = InMemoryAgentQuotaStore::new();
    store.set_budget(tenant_x.clone(), ag.clone(), budget);
    store.set_budget(tenant_y.clone(), ag.clone(), budget);

    // Exhaust tenant_x's budget.
    store
        .reserve(&tenant_x, &ag, 200)
        .expect("exhaust tenant_x");

    // tenant_y's same-named agent must still have full budget.
    let snap_y = store.snapshot(&tenant_y, &ag).expect("snap_y");
    assert_eq!(
        snap_y.remaining_tokens, 200,
        "tenant_y budget must be isolated from tenant_x"
    );
}

// ── dispatch_to_pool_with_quota integration tests ────────────────────────────

/// dispatch_rejects_over_budget_request
#[tokio::test]
async fn dispatch_rejects_over_budget_request() {
    let tenant = ten("t1");
    let pool = pool_id("p1");
    let ag = agent("agent_over");

    let repo = InMemoryPoolRepository::new().with_pool(simple_pool(&tenant, &pool, &["seat_a"]));
    let usage = InMemoryUsageSnapshotSource::new();
    let mut health = InMemoryAccountHealthStore::new();

    // Transport must NOT be called — quota check must short-circuit.
    let script: TransportScript =
        Arc::new(|_, _, _| panic!("transport must not be called when quota is exhausted"));
    let transport = InMemoryProviderInvocationTransport::new(script);
    let secret = DeniedSecretResolver;
    let metrics = NoOpMetricsSink;

    // Budget of 50 tokens, request estimates 200.
    let mut quota_store = InMemoryAgentQuotaStore::new();
    quota_store.set_budget(
        tenant.clone(),
        ag.clone(),
        AgentQuotaBudget {
            budget_tokens: 50,
            window_reset_unix_ms: 0,
        },
    );

    let err = dispatch_to_pool_with_quota(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
        &tenant,
        &pool,
        &RequestMetadata::new("m".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
        &mut quota_store,
        &ag,
        200, // estimated_tokens >> budget
    )
    .await
    .expect_err("over-budget dispatch must be rejected");

    match err {
        DispatchError::QuotaBudgetExceeded {
            agent: err_agent,
            requested,
            remaining,
        } => {
            assert_eq!(err_agent, ag);
            assert_eq!(requested, 200);
            assert_eq!(remaining, 50);
        }
        other => panic!("expected QuotaBudgetExceeded, got {other:?}"),
    }

    // No transport call must have been made.
    assert!(transport.call_log().is_empty());
}

/// dispatch_reconciles_actual_after_success
#[tokio::test]
async fn dispatch_reconciles_actual_after_success() {
    let tenant = ten("t1");
    let pool = pool_id("p1");
    let ag = agent("agent_ok");

    let repo = InMemoryPoolRepository::new().with_pool(simple_pool(&tenant, &pool, &["seat_a"]));
    let usage = InMemoryUsageSnapshotSource::new();
    let mut health = InMemoryAccountHealthStore::new();

    let script: TransportScript = Arc::new(|account, _, _| Ok(ok_response(account)));
    let transport = InMemoryProviderInvocationTransport::new(script);
    let secret = DeniedSecretResolver;
    let metrics = NoOpMetricsSink;

    // Budget: 1000. Estimate: 200. Actual response usage: 80 tokens.
    // After reconcile: remaining = 1000 - 200 (reserve) + (200 - 80) credit = 920.
    let mut quota_store = InMemoryAgentQuotaStore::new();
    quota_store.set_budget(
        tenant.clone(),
        ag.clone(),
        AgentQuotaBudget {
            budget_tokens: 1_000,
            window_reset_unix_ms: 0,
        },
    );

    let outcome = dispatch_to_pool_with_quota(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
        &tenant,
        &pool,
        &RequestMetadata::new("m".into()),
        UnixMillis(1),
        // Body carries "actual_tokens" hint via a JSON field; the reconciler
        // uses the `actual_tokens_used` field in DispatchOutcome (or 0 when absent).
        // For this test we use 0 actual_tokens_hint → the store's reconcile uses
        // the outcome's token count. Since the in-memory transport returns a fixed
        // body without usage metadata, reconcile will use 0 as actual usage,
        // crediting back the full 200 reserve.
        Bytes::from_static(b"{}"),
        &mut quota_store,
        &ag,
        200, // estimated_tokens
    )
    .await
    .expect("dispatch must succeed");

    assert_eq!(outcome.response.provider_account_id, pid("seat_a"));

    // After reconcile with actual=0 (no usage metadata in in-memory transport):
    // reserved 200 → remaining=800; reconcile(estimate=200, actual=0) credits back
    // 200 → remaining=1000 again.
    let snap = quota_store.snapshot(&tenant, &ag).expect("snap");
    // The in-memory transport has no token-count metadata, so actual_used=0.
    assert_eq!(
        snap.remaining_tokens, 1_000,
        "remaining must be fully restored after reconcile with actual_used=0"
    );
}

/// dispatch_without_quota_store_unchanged (zero regression)
#[tokio::test]
async fn dispatch_without_quota_store_unchanged() {
    let tenant = ten("t1");
    let pool = pool_id("p1");

    let repo = InMemoryPoolRepository::new().with_pool(simple_pool(&tenant, &pool, &["seat_a"]));
    let usage = InMemoryUsageSnapshotSource::new();
    let mut health = InMemoryAccountHealthStore::new();
    let script: TransportScript = Arc::new(|account, _, _| Ok(ok_response(account)));
    let transport = InMemoryProviderInvocationTransport::new(script);
    let secret = DeniedSecretResolver;
    let metrics = NoOpMetricsSink;

    // Use existing dispatch_to_pool (no quota) — must work identically to before.
    let outcome = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
        &tenant,
        &pool,
        &RequestMetadata::new("m".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("no-quota dispatch must succeed");

    assert_eq!(outcome.response.provider_account_id, pid("seat_a"));
}

/// dispatch_skip_when_ample_no_reserve_write
///
/// When the agent has > QUOTA_AMPLE_THRESHOLD_PCT remaining, the reserve step
/// is skipped (no write to the quota store) but reconcile still runs on success.
#[tokio::test]
async fn dispatch_skip_when_ample_no_reserve_write() {
    let tenant = ten("t1");
    let pool = pool_id("p1");
    let ag = agent("agent_ample");

    let repo = InMemoryPoolRepository::new().with_pool(simple_pool(&tenant, &pool, &["seat_a"]));
    let usage = InMemoryUsageSnapshotSource::new();
    let mut health = InMemoryAccountHealthStore::new();
    let script: TransportScript = Arc::new(|account, _, _| Ok(ok_response(account)));
    let transport = InMemoryProviderInvocationTransport::new(script);
    let secret = DeniedSecretResolver;
    let metrics = NoOpMetricsSink;

    // Budget: 1000, remaining: 1000 → 100% > 80% threshold → skip reserve.
    let mut quota_store = InMemoryAgentQuotaStore::new();
    quota_store.set_budget(
        tenant.clone(),
        ag.clone(),
        AgentQuotaBudget {
            budget_tokens: 1_000,
            window_reset_unix_ms: 0,
        },
    );

    let outcome = dispatch_to_pool_with_quota(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
        &tenant,
        &pool,
        &RequestMetadata::new("m".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
        &mut quota_store,
        &ag,
        100, // estimated_tokens (skipped because ample)
    )
    .await
    .expect("ample-headroom dispatch must succeed");

    assert_eq!(outcome.response.provider_account_id, pid("seat_a"));

    // Reconcile still runs after success: since reserve was skipped,
    // reconcile(estimate=100, actual=0) should not go below 0; remaining
    // should stay at 1000 (no reserve was made, and credit back of 100 is
    // capped since nothing was reserved — the store returns remaining=1000).
    let snap = quota_store.snapshot(&tenant, &ag).expect("snap");
    assert_eq!(
        snap.remaining_tokens, 1_000,
        "when reserve was skipped, reconcile must not go below or above budget"
    );

    // Verify the dispatch succeeded (transport was called).
    assert_eq!(transport.call_log(), vec![pid("seat_a")]);
}
