//! Acceptance tests for the provider-pool composition root, grounded in the
//! kernel's actual behaviors (`oya-intelligence-provider-pool-kernel`) +
//! `microservices/intelligence/PRD.md` (M02-P02 ProviderAccountPool).
//!
//! These drive the FULL flow through the REAL kernel — `pick_account` is
//! the canonical routing function and is exercised verbatim by the
//! composition root. There are NO kernel stubs. The transport is the
//! in-memory scripted adapter so the failover loop is deterministic
//! (acceptance tests must not require network egress).
//!
//! Mapped behaviors (kernel test contract):
//! - happy-path provider selection                    (kernel single_member_no_special_case)
//! - failover walks fallback_chain on retryable error (kernel unhealthy_member_is_skipped_in_fallback)
//! - non-retryable transport short-circuits           (composition-root invariant)
//! - default-deny on missing pool                     (composition-root + kernel EmptyMembers)
//! - blacklist progression honors AccountHealthMap    (kernel all_unhealthy_returns_no_healthy_members)
//! - cross-tenant isolation by (TenantId, PoolId)     (composition-root invariant)
//! - concurrency / determinism                        (kernel deterministic_given_identical_inputs)
//! - sticky session keeps previous_account            (kernel sticky_keeps_previous_account_if_healthy)
//! - HyperProviderInvocationTransport honest boundary (Unimplemented::OpenBaoSecretResolution)

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::sync::Arc;

use bytes::Bytes;

use oya_intelligence_provider_pool_app::{
    AccountHealthStore, DispatchError, HyperProviderInvocationTransport,
    InMemoryAccountHealthStore, InMemoryPoolRepository, InMemoryProviderInvocationTransport,
    InMemoryUsageSnapshotSource, PoolError, PoolId, PoolRoutingReason, PoolRoutingStrategy,
    ProviderAccountId, ProviderAccountPool, ProviderFamily, ProviderInvocationTransport,
    ProviderResponse, ProviderTier, RequestMetadata, SessionId, TenantId, TransportError,
    TransportScript, Unimplemented, UnixMillis, UsageSnapshot, UsageSnapshotMap, dispatch_to_pool,
};
use oya_intelligence_provider_pool_kernel::DurationMs;

fn pid(s: &str) -> ProviderAccountId {
    ProviderAccountId(s.to_owned())
}

fn ten(s: &str) -> TenantId {
    TenantId(s.to_owned())
}

fn pid_pool(s: &str) -> PoolId {
    PoolId(s.to_owned())
}

fn pool(
    tenant: &TenantId,
    pool_id: &PoolId,
    members: &[&str],
    strategy: PoolRoutingStrategy,
) -> ProviderAccountPool {
    let mut set: BTreeSet<ProviderAccountId> = BTreeSet::new();
    for m in members {
        set.insert(pid(m));
    }
    ProviderAccountPool::new(
        pool_id.clone(),
        ProviderFamily::Claude,
        ProviderTier::Pro,
        tenant.clone(),
        set,
        strategy,
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

/// AC: happy-path — single healthy account is dispatched to with no failover.
#[tokio::test]
async fn happy_path_dispatches_to_chosen_account() {
    let tenant = ten("ten_acme");
    let pool_id = pid_pool("pool_claude_pro");
    let repo = InMemoryPoolRepository::new().with_pool(pool(
        &tenant,
        &pool_id,
        &["alpha", "beta", "gamma"],
        PoolRoutingStrategy::RoundRobin,
    ));
    let usage = InMemoryUsageSnapshotSource::new();
    let mut health = InMemoryAccountHealthStore::new();

    let script: TransportScript = Arc::new(|account, _provider, _body| Ok(ok_response(account)));
    let transport = InMemoryProviderInvocationTransport::new(script);

    let outcome = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &tenant,
        &pool_id,
        &RequestMetadata::new("claude-3-5-sonnet".into()),
        UnixMillis(1_700_000_000_000),
        Bytes::from_static(b"{\"prompt\":\"x\"}"),
    )
    .await
    .expect("happy-path dispatch must succeed");

    // BTreeSet orders "alpha" before "beta"/"gamma" — kernel picks first.
    assert_eq!(outcome.response.provider_account_id, pid("alpha"));
    assert_eq!(outcome.attempts, vec![pid("alpha")]);
    assert_eq!(outcome.primary_reason, PoolRoutingReason::Healthy);
    // The transport was called exactly once.
    assert_eq!(transport.call_log(), vec![pid("alpha")]);
    // Success was recorded.
    let map = health.read(&tenant, &pool_id).expect("read");
    assert!(map.contains_key(&pid("alpha")));
}

/// AC: failover — a retryable transport error walks the kernel's
/// `fallback_chain`. The health-store progression matches the kernel's
/// `unhealthy_member_is_skipped_in_fallback` invariant on the NEXT dispatch.
#[tokio::test]
async fn retryable_failure_walks_fallback_chain_then_succeeds() {
    let tenant = ten("ten_acme");
    let pool_id = pid_pool("pool_claude_pro");
    let repo = InMemoryPoolRepository::new().with_pool(pool(
        &tenant,
        &pool_id,
        &["alpha", "beta", "gamma"],
        PoolRoutingStrategy::RoundRobin,
    ));
    let usage = InMemoryUsageSnapshotSource::new();
    let mut health = InMemoryAccountHealthStore::with_thresholds(2, 5);

    // Script: alpha + beta return retryable; gamma succeeds.
    let script: TransportScript = Arc::new(|account, _provider, _body| {
        if account == &pid("gamma") {
            Ok(ok_response(account))
        } else {
            Err(TransportError::Retryable {
                detail: format!("{account:?} simulated 502"),
            })
        }
    });
    let transport = InMemoryProviderInvocationTransport::new(script);

    let outcome = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &tenant,
        &pool_id,
        &RequestMetadata::new("claude-3-5-sonnet".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("failover must converge on gamma");

    assert_eq!(outcome.response.provider_account_id, pid("gamma"));
    // The dispatch order is alpha (primary), then beta + gamma from
    // BTreeSet-deterministic fallback_chain.
    assert_eq!(
        outcome.attempts,
        vec![pid("alpha"), pid("beta"), pid("gamma")]
    );
    assert_eq!(
        transport.call_log(),
        vec![pid("alpha"), pid("beta"), pid("gamma")]
    );

    // Health: alpha + beta have one failure each, gamma is healthy.
    let map = health.read(&tenant, &pool_id).expect("read");
    assert_eq!(map.get(&pid("alpha")).unwrap().consecutive_failures, 1);
    assert_eq!(map.get(&pid("beta")).unwrap().consecutive_failures, 1);
    assert_eq!(map.get(&pid("gamma")).unwrap().consecutive_failures, 0);
}

/// AC: failover progression respects the kernel's
/// `all_unhealthy_returns_no_healthy_members` invariant. Once an account has
/// crossed the quarantine threshold the kernel filters it from the healthy
/// set, so a subsequent dispatch routes around it.
#[tokio::test]
async fn blacklist_progression_quarantines_after_threshold_then_kernel_skips() {
    let tenant = ten("ten_acme");
    let pool_id = pid_pool("pool_claude_pro");
    let repo = InMemoryPoolRepository::new().with_pool(pool(
        &tenant,
        &pool_id,
        &["alpha", "beta"],
        PoolRoutingStrategy::RoundRobin,
    ));
    let usage = InMemoryUsageSnapshotSource::new();
    let mut health = InMemoryAccountHealthStore::with_thresholds(2, 3);

    // First three dispatches: alpha always fails (retryable). Beta succeeds.
    let script: TransportScript = Arc::new(|account, _provider, _body| {
        if account == &pid("beta") {
            Ok(ok_response(account))
        } else {
            Err(TransportError::Retryable {
                detail: "alpha down".into(),
            })
        }
    });
    let transport = InMemoryProviderInvocationTransport::new(script);

    // Loop: 3 dispatches drive alpha across the quarantine threshold.
    for _ in 0..3 {
        let outcome = dispatch_to_pool(
            &repo,
            &usage,
            &mut health,
            &transport,
            &tenant,
            &pool_id,
            &RequestMetadata::new("m".into()),
            UnixMillis(1),
            Bytes::from_static(b"{}"),
        )
        .await
        .expect("beta absorbs failover");
        assert_eq!(outcome.response.provider_account_id, pid("beta"));
    }

    // alpha is now Unhealthy. The kernel's filter must skip it on the next
    // dispatch — the composition root should never call the transport for
    // alpha at all.
    let map = health.read(&tenant, &pool_id).expect("read");
    assert_eq!(
        map.get(&pid("alpha")).unwrap().state,
        oya_intelligence_provider_pool_app::HealthState::Unhealthy
    );

    let pre_call_count = transport.call_log().len();
    let outcome = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &tenant,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(2),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("beta serves the post-quarantine dispatch");

    // Only beta was called — alpha was filtered by the kernel.
    assert_eq!(outcome.attempts, vec![pid("beta")]);
    assert_eq!(outcome.response.provider_account_id, pid("beta"));
    let new_calls = transport.call_log()[pre_call_count..].to_vec();
    assert_eq!(new_calls, vec![pid("beta")]);
}

/// AC: all-unhealthy pool — every member is past the quarantine threshold,
/// so the kernel returns `PoolError::NoHealthyMembers` and the dispatch
/// loop default-denies (the transport must NEVER be called).
#[tokio::test]
async fn all_unhealthy_members_default_deny_via_kernel() {
    let tenant = ten("ten_acme");
    let pool_id = pid_pool("pool_claude_pro");
    let repo = InMemoryPoolRepository::new().with_pool(pool(
        &tenant,
        &pool_id,
        &["alpha", "beta"],
        PoolRoutingStrategy::RoundRobin,
    ));
    let usage = InMemoryUsageSnapshotSource::new();
    let mut health = InMemoryAccountHealthStore::with_thresholds(1, 1);

    // Pre-quarantine both accounts.
    for member in ["alpha", "beta"] {
        health
            .record_failure(&tenant, &pool_id, &pid(member))
            .expect("quarantine");
    }

    let script: TransportScript = Arc::new(|_, _, _| {
        panic!("transport must not be called when no healthy members exist");
    });
    let transport = InMemoryProviderInvocationTransport::new(script);

    let err = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &tenant,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect_err("all-unhealthy must default-deny");

    assert_eq!(err, DispatchError::Routing(PoolError::NoHealthyMembers));
    assert!(transport.call_log().is_empty());
}

/// AC: non-retryable transport error short-circuits the dispatch loop — the
/// fallback chain is NOT walked, because retrying against another account
/// cannot resolve a non-retryable failure (e.g. malformed body).
#[tokio::test]
async fn non_retryable_transport_short_circuits_failover() {
    let tenant = ten("ten_acme");
    let pool_id = pid_pool("pool_claude_pro");
    let repo = InMemoryPoolRepository::new().with_pool(pool(
        &tenant,
        &pool_id,
        &["alpha", "beta"],
        PoolRoutingStrategy::RoundRobin,
    ));
    let usage = InMemoryUsageSnapshotSource::new();
    let mut health = InMemoryAccountHealthStore::new();

    let script: TransportScript = Arc::new(|_, _, _| {
        Err(TransportError::NonRetryable {
            detail: "malformed request".into(),
        })
    });
    let transport = InMemoryProviderInvocationTransport::new(script);

    let err = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &tenant,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect_err("non-retryable must short-circuit");

    match err {
        DispatchError::NonRetryableTransport(TransportError::NonRetryable { detail }) => {
            assert!(detail.contains("malformed request"));
        }
        other => panic!("expected NonRetryableTransport, got {other:?}"),
    }

    // Only alpha was attempted — beta was NOT consulted.
    assert_eq!(transport.call_log(), vec![pid("alpha")]);
}

/// AC: chain exhaustion — every account in the kernel's fallback_chain
/// returns retryable; the loop exhausts and surfaces
/// `DispatchError::AllProvidersExhausted` carrying the full attempt log.
#[tokio::test]
async fn chain_exhaustion_surfaces_all_providers_exhausted() {
    let tenant = ten("ten_acme");
    let pool_id = pid_pool("pool_claude_pro");
    let repo = InMemoryPoolRepository::new().with_pool(pool(
        &tenant,
        &pool_id,
        &["alpha", "beta"],
        PoolRoutingStrategy::RoundRobin,
    ));
    let usage = InMemoryUsageSnapshotSource::new();
    let mut health = InMemoryAccountHealthStore::new();

    let script: TransportScript = Arc::new(|account, _, _| {
        Err(TransportError::Retryable {
            detail: format!("{} 502", account.0),
        })
    });
    let transport = InMemoryProviderInvocationTransport::new(script);

    let err = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &tenant,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect_err("chain exhaustion must default-deny");

    match err {
        DispatchError::AllProvidersExhausted {
            attempts,
            last_error,
        } => {
            assert_eq!(attempts, vec![pid("alpha"), pid("beta")]);
            match last_error {
                TransportError::Retryable { detail } => assert!(detail.contains("beta 502")),
                other => panic!("expected Retryable last_error, got {other:?}"),
            }
        }
        other => panic!("expected AllProvidersExhausted, got {other:?}"),
    }
}

/// AC: cross-tenant isolation — the same `PoolId` namespace under two
/// different `TenantId`s resolves to two independent pools. A dispatch on
/// tenant A never sees tenant B's pool, and the health-store progression is
/// keyed by `(TenantId, PoolId)` so quarantines do not bleed across tenants.
#[tokio::test]
async fn cross_tenant_pools_are_isolated() {
    let tenant_a = ten("ten_acme");
    let tenant_b = ten("ten_initech");
    let pool_id = pid_pool("pool_claude_pro");

    let repo = InMemoryPoolRepository::new()
        .with_pool(pool(
            &tenant_a,
            &pool_id,
            &["acme-1"],
            PoolRoutingStrategy::RoundRobin,
        ))
        .with_pool(pool(
            &tenant_b,
            &pool_id,
            &["initech-1"],
            PoolRoutingStrategy::RoundRobin,
        ));

    let usage = InMemoryUsageSnapshotSource::new();
    let mut health = InMemoryAccountHealthStore::new();
    let script: TransportScript = Arc::new(|account, _, _| Ok(ok_response(account)));
    let transport = InMemoryProviderInvocationTransport::new(script);

    let outcome_a = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &tenant_a,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("tenant_a dispatch");
    assert_eq!(outcome_a.response.provider_account_id, pid("acme-1"));

    let outcome_b = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &tenant_b,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(2),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("tenant_b dispatch");
    assert_eq!(outcome_b.response.provider_account_id, pid("initech-1"));

    // Health is keyed by (TenantId, PoolId): tenant A's health map does not
    // mention tenant B's account, and vice versa.
    let map_a = health.read(&tenant_a, &pool_id).expect("read a");
    let map_b = health.read(&tenant_b, &pool_id).expect("read b");
    assert!(map_a.contains_key(&pid("acme-1")));
    assert!(!map_a.contains_key(&pid("initech-1")));
    assert!(map_b.contains_key(&pid("initech-1")));
    assert!(!map_b.contains_key(&pid("acme-1")));
}

/// AC: sticky-session strategy — if the request carries a `previous_account`
/// that is still healthy, the kernel returns it as the chosen account
/// (kernel `sticky_keeps_previous_account_if_healthy` invariant). The
/// composition root must dispatch against that account.
#[tokio::test]
async fn sticky_session_keeps_previous_account_if_healthy() {
    let tenant = ten("ten_acme");
    let pool_id = pid_pool("pool_claude_pro");
    let session = SessionId("s1".into());
    let repo = InMemoryPoolRepository::new().with_pool(pool(
        &tenant,
        &pool_id,
        &["alpha", "beta", "gamma"],
        PoolRoutingStrategy::Sticky(session.clone()),
    ));
    let usage = InMemoryUsageSnapshotSource::new();
    let mut health = InMemoryAccountHealthStore::new();

    let script: TransportScript = Arc::new(|account, _, _| Ok(ok_response(account)));
    let transport = InMemoryProviderInvocationTransport::new(script);

    let mut request = RequestMetadata::new("claude-3-5-sonnet".into());
    request.session = Some(session);
    request.previous_account = Some(pid("beta"));

    let outcome = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &tenant,
        &pool_id,
        &request,
        UnixMillis(1),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("sticky dispatch");

    assert_eq!(outcome.response.provider_account_id, pid("beta"));
    assert_eq!(outcome.primary_reason, PoolRoutingReason::Sticky);
    assert_eq!(transport.call_log(), vec![pid("beta")]);
}

/// AC: `LeastUsed` strategy — kernel reads the usage snapshot and picks the
/// member with the lowest `requests_in_window`. The composition root passes
/// the snapshot through verbatim (kernel `least_used_picks_strictly_lowest`
/// invariant).
#[tokio::test]
async fn least_used_uses_kernel_usage_snapshot() {
    let tenant = ten("ten_acme");
    let pool_id = pid_pool("pool_claude_pro");
    let repo = InMemoryPoolRepository::new().with_pool(pool(
        &tenant,
        &pool_id,
        &["a", "b", "c"],
        PoolRoutingStrategy::LeastUsed,
    ));

    let mut usage_map: UsageSnapshotMap = std::collections::BTreeMap::new();
    usage_map.insert(
        pid("a"),
        UsageSnapshot {
            requests_in_window: 99,
            ..UsageSnapshot::zero()
        },
    );
    usage_map.insert(
        pid("b"),
        UsageSnapshot {
            requests_in_window: 1,
            ..UsageSnapshot::zero()
        },
    );
    usage_map.insert(
        pid("c"),
        UsageSnapshot {
            requests_in_window: 50,
            ..UsageSnapshot::zero()
        },
    );

    let usage = InMemoryUsageSnapshotSource::new().with_snapshot(
        tenant.clone(),
        pool_id.clone(),
        usage_map,
    );
    let mut health = InMemoryAccountHealthStore::new();

    let script: TransportScript = Arc::new(|account, _, _| Ok(ok_response(account)));
    let transport = InMemoryProviderInvocationTransport::new(script);

    let outcome = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &tenant,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("least-used dispatch");

    // Kernel picks "b" — lowest requests_in_window.
    assert_eq!(outcome.response.provider_account_id, pid("b"));
    assert_eq!(outcome.primary_reason, PoolRoutingReason::Healthy);
}

/// AC: determinism — two identical dispatches with no intervening state
/// change yield identical outcomes. Mirrors the kernel's
/// `deterministic_given_identical_inputs` invariant; the composition root
/// must not introduce non-determinism (no hidden RNG, no clock-skew, no
/// global mutable state).
#[tokio::test]
async fn dispatch_is_deterministic_given_identical_inputs() {
    let tenant = ten("ten_acme");
    let pool_id = pid_pool("pool_claude_pro");
    let repo = InMemoryPoolRepository::new().with_pool(pool(
        &tenant,
        &pool_id,
        &["alpha", "beta", "gamma"],
        PoolRoutingStrategy::RoundRobin,
    ));
    let usage = InMemoryUsageSnapshotSource::new();
    let script: TransportScript = Arc::new(|account, _, _| Ok(ok_response(account)));
    let transport = InMemoryProviderInvocationTransport::new(script);

    let mut health1 = InMemoryAccountHealthStore::new();
    let outcome1 = dispatch_to_pool(
        &repo,
        &usage,
        &mut health1,
        &transport,
        &tenant,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("dispatch 1");

    let mut health2 = InMemoryAccountHealthStore::new();
    let outcome2 = dispatch_to_pool(
        &repo,
        &usage,
        &mut health2,
        &transport,
        &tenant,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("dispatch 2");

    assert_eq!(
        outcome1.response.provider_account_id,
        outcome2.response.provider_account_id
    );
    assert_eq!(outcome1.attempts, outcome2.attempts);
    assert_eq!(outcome1.primary_reason, outcome2.primary_reason);
}

/// AC: honest-claims boundary — the production hyper transport surfaces a
/// typed `Unimplemented::OpenBaoSecretResolution` because credential
/// resolution is not yet wired. The dispatch loop maps this to a
/// `DispatchError::NonRetryableTransport` (no silent fake success).
#[tokio::test]
async fn hyper_transport_surfaces_unimplemented_via_dispatch_error() {
    let tenant = ten("ten_acme");
    let pool_id = pid_pool("pool_claude_pro");
    let repo = InMemoryPoolRepository::new().with_pool(pool(
        &tenant,
        &pool_id,
        &["alpha"],
        PoolRoutingStrategy::RoundRobin,
    ));
    let usage = InMemoryUsageSnapshotSource::new();
    let mut health = InMemoryAccountHealthStore::new();
    let transport = HyperProviderInvocationTransport::new("https://api.anthropic.com");

    let err = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &tenant,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect_err("hyper transport must surface the unimplemented boundary");

    match err {
        DispatchError::NonRetryableTransport(TransportError::NonRetryable { detail }) => {
            assert!(
                detail.contains(Unimplemented::OpenBaoSecretResolution.as_str()),
                "detail must cite the typed Unimplemented variant, got {detail}"
            );
            assert!(
                detail.contains(Unimplemented::OpenBaoSecretResolution.placeholder_debt_id()),
                "detail must cite the placeholder-debt id, got {detail}"
            );
        }
        other => panic!("expected NonRetryableTransport, got {other:?}"),
    }
}

/// AC: the production transport itself round-trips its base URL (the
/// composition root holds the upstream identity for the hyper client) — this
/// is the seam that the OpenBao + audit follow-ups will close.
#[tokio::test]
async fn hyper_transport_round_trips_upstream_base_url() {
    let transport = HyperProviderInvocationTransport::new("https://api.openai.com");
    assert_eq!(transport.upstream_base_url(), "https://api.openai.com");
    // Even a direct dispatch surfaces the honest boundary.
    let err = transport
        .dispatch(
            pid("a"),
            ProviderFamily::OpenAiOrCodex,
            Bytes::from_static(b"{}"),
        )
        .await
        .expect_err("honest-claims boundary");
    match err {
        TransportError::NonRetryable { detail } => {
            assert!(detail.contains("Unimplemented::OpenBaoSecretResolution"));
        }
        other => panic!("expected NonRetryable, got {other:?}"),
    }
}
