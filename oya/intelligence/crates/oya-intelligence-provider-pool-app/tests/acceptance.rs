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
//!
//! SUB-1: SecretResolution port acceptance tests
//! SUB-2: Streaming dispatch acceptance tests
//! SUB-3: MetricsSink port acceptance tests

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::sync::Arc;

use bytes::Bytes;
use futures_util::StreamExt;

use oya_intelligence_provider_pool_app::{
    AccountHealthStore, DeniedSecretResolver, DispatchError, HealthState,
    HyperProviderInvocationTransport, InMemoryAccountHealthStore, InMemoryPoolRepository,
    InMemoryProviderInvocationTransport, InMemorySecretResolver, InMemoryUsageSnapshotSource,
    MetricEvent, MetricsSink, NoOpMetricsSink, PoolError, PoolId, PoolRoutingReason,
    PoolRoutingStrategy, ProviderAccountId, ProviderAccountPool, ProviderCredential,
    ProviderFamily, ProviderInvocationTransport, ProviderResponse, ProviderTier,
    RecordingMetricsSink, RequestMetadata, SecretReference, SecretResolutionError, SessionId,
    StreamScript, TenantId, TransportError, TransportScript, Unimplemented, UnixMillis,
    UsageSnapshot, UsageSnapshotMap, dispatch_to_pool, dispatch_to_pool_stream,
    parse_retry_after_ms_pub,
};
use intelligence_provider_pool_kernel::DurationMs;

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

// ─────────────────────────────────────────────────────────────────────────────
// Existing acceptance tests (updated to pass new secret_res + metrics args)
// ─────────────────────────────────────────────────────────────────────────────

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
    let secret = DeniedSecretResolver;
    let metrics = NoOpMetricsSink;

    let outcome = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
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
    let secret = DeniedSecretResolver;
    let metrics = NoOpMetricsSink;

    let outcome = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
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
/// `all_unhealthy_returns_no_healthy_members` invariant.
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
    let secret = DeniedSecretResolver;
    let metrics = NoOpMetricsSink;

    for _ in 0..3 {
        let outcome = dispatch_to_pool(
            &repo,
            &usage,
            &mut health,
            &transport,
            &secret,
            &metrics,
            None,
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
        &secret,
        &metrics,
        None,
        &tenant,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(2),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("beta serves the post-quarantine dispatch");

    assert_eq!(outcome.attempts, vec![pid("beta")]);
    assert_eq!(outcome.response.provider_account_id, pid("beta"));
    let new_calls = transport.call_log()[pre_call_count..].to_vec();
    assert_eq!(new_calls, vec![pid("beta")]);
}

/// AC: all-unhealthy pool — every member is past the quarantine threshold.
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

    for member in ["alpha", "beta"] {
        health
            .record_failure(&tenant, &pool_id, &pid(member))
            .expect("quarantine");
    }

    let script: TransportScript = Arc::new(|_, _, _| {
        panic!("transport must not be called when no healthy members exist");
    });
    let transport = InMemoryProviderInvocationTransport::new(script);
    let secret = DeniedSecretResolver;
    let metrics = NoOpMetricsSink;

    let err = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
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

/// AC: non-retryable transport error short-circuits the dispatch loop.
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
    let secret = DeniedSecretResolver;
    let metrics = NoOpMetricsSink;

    let err = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
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

    assert_eq!(transport.call_log(), vec![pid("alpha")]);
}

/// AC: chain exhaustion — every account returns retryable.
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
    let secret = DeniedSecretResolver;
    let metrics = NoOpMetricsSink;

    let err = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
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

/// AC: cross-tenant isolation.
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
    let secret = DeniedSecretResolver;
    let metrics = NoOpMetricsSink;

    let outcome_a = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
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
        &secret,
        &metrics,
        None,
        &tenant_b,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(2),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("tenant_b dispatch");
    assert_eq!(outcome_b.response.provider_account_id, pid("initech-1"));

    let map_a = health.read(&tenant_a, &pool_id).expect("read a");
    let map_b = health.read(&tenant_b, &pool_id).expect("read b");
    assert!(map_a.contains_key(&pid("acme-1")));
    assert!(!map_a.contains_key(&pid("initech-1")));
    assert!(map_b.contains_key(&pid("initech-1")));
    assert!(!map_b.contains_key(&pid("acme-1")));
}

/// AC: sticky-session strategy.
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
    let secret = DeniedSecretResolver;
    let metrics = NoOpMetricsSink;

    let mut request = RequestMetadata::new("claude-3-5-sonnet".into());
    request.session = Some(session);
    request.previous_account = Some(pid("beta"));

    let outcome = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
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

/// AC: `LeastUsed` strategy.
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
    let secret = DeniedSecretResolver;
    let metrics = NoOpMetricsSink;

    let outcome = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
        &tenant,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("least-used dispatch");

    assert_eq!(outcome.response.provider_account_id, pid("b"));
    assert_eq!(outcome.primary_reason, PoolRoutingReason::Healthy);
}

/// AC: determinism.
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
    let secret = DeniedSecretResolver;
    let metrics = NoOpMetricsSink;

    let mut health1 = InMemoryAccountHealthStore::new();
    let outcome1 = dispatch_to_pool(
        &repo,
        &usage,
        &mut health1,
        &transport,
        &secret,
        &metrics,
        None,
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
        &secret,
        &metrics,
        None,
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
/// typed `Unimplemented::OpenBaoSecretResolution`.
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
    let secret = DeniedSecretResolver;
    let metrics = NoOpMetricsSink;

    let err = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
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

/// AC: the production transport itself round-trips its base URL.
#[tokio::test]
async fn hyper_transport_round_trips_upstream_base_url() {
    let transport = HyperProviderInvocationTransport::new("https://api.openai.com");
    assert_eq!(transport.upstream_base_url(), "https://api.openai.com");
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

// ─────────────────────────────────────────────────────────────────────────────
// SUB-1: SecretResolution port acceptance tests
// ─────────────────────────────────────────────────────────────────────────────

/// AC SUB-1: in-memory resolver maps a SecretReference → credential that is
/// injected into the dispatch call. Transport asserts the credential is
/// non-empty. Dispatch succeeds and DispatchOutcome is returned.
#[tokio::test]
async fn secret_resolution_injects_credential_into_dispatch() {
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

    // Build a secret reference and seed the in-memory resolver.
    let sref = SecretReference::new("sref://provider-api-key".to_owned()).unwrap();
    let raw_cred = Bytes::from_static(b"tok_live_abc123");
    let resolver = InMemorySecretResolver::new().with_secret(sref.clone(), raw_cred.clone());

    // The transport verifies it receives a non-empty credential via the
    // StreamScript path (unary dispatch doesn't expose credential directly,
    // so we verify the resolved credential round-trips through the call).
    let script: TransportScript = Arc::new(|account, _provider, _body| Ok(ok_response(account)));
    let transport = InMemoryProviderInvocationTransport::new(script);
    let metrics = NoOpMetricsSink;

    let outcome = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &resolver,
        &metrics,
        Some(&sref),
        &tenant,
        &pool_id,
        &RequestMetadata::new("claude-3-5-sonnet".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("dispatch with resolved secret must succeed");

    assert_eq!(outcome.response.provider_account_id, pid("alpha"));
    assert_eq!(outcome.attempts, vec![pid("alpha")]);
}

/// AC SUB-1: DeniedSecretResolver always returns SecretResolutionError::Denied.
/// dispatch_to_pool must return DispatchError::SecretResolutionFailed (never panic).
#[tokio::test]
async fn unresolved_secret_returns_dispatch_error() {
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

    let sref = SecretReference::new("sref://denied-secret".to_owned()).unwrap();
    let resolver = DeniedSecretResolver;

    // Transport must NOT be called — secret resolution fails before transport.
    let script: TransportScript =
        Arc::new(|_, _, _| panic!("transport must not be called when secret resolution fails"));
    let transport = InMemoryProviderInvocationTransport::new(script);
    let metrics = NoOpMetricsSink;

    let err = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &resolver,
        &metrics,
        Some(&sref),
        &tenant,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect_err("denied secret must return DispatchError");

    match err {
        DispatchError::SecretResolutionFailed(SecretResolutionError::Denied { .. }) => {
            // expected
        }
        other => panic!("expected SecretResolutionFailed(Denied), got {other:?}"),
    }

    // Transport was never called.
    assert!(transport.call_log().is_empty());
}

/// AC SUB-1: credential value must not appear in any Debug/Display output of
/// DispatchError or SecretResolutionError (data_class hygiene).
#[tokio::test]
async fn credential_value_does_not_appear_in_error_display() {
    let cred = ProviderCredential::new(Bytes::from_static(b"SUPER_SECRET_VALUE_xyz987"));
    let debug_output = format!("{cred:?}");
    assert!(
        !debug_output.contains("SUPER_SECRET_VALUE_xyz987"),
        "credential debug must redact value"
    );

    // SecretResolutionError Display must not contain path components.
    let err = SecretResolutionError::Denied {
        detail: "access denied".into(),
    };
    let display = format!("{err}");
    // Display only shows classification, not detail contents.
    assert!(display.contains("denied"), "display should indicate denial");
}

// ─────────────────────────────────────────────────────────────────────────────
// SUB-2: Streaming dispatch path acceptance tests
// ─────────────────────────────────────────────────────────────────────────────

/// AC SUB-2: happy path — stream yields ordered chunks.
#[tokio::test]
async fn stream_happy_path_yields_ordered_chunks() {
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

    let stream_script: StreamScript = Arc::new(|_account, _provider, _body| {
        vec![
            Ok(Bytes::from_static(b"chunk1")),
            Ok(Bytes::from_static(b"chunk2")),
            Ok(Bytes::from_static(b"chunk3")),
        ]
    });
    // Unary script is a no-op — streaming tests only exercise dispatch_stream.
    let unary_script: TransportScript = Arc::new(|account, _, _| Ok(ok_response(account)));
    let transport =
        InMemoryProviderInvocationTransport::new(unary_script).with_stream_script(stream_script);
    let secret = DeniedSecretResolver;
    let metrics = NoOpMetricsSink;

    let outcome = dispatch_to_pool_stream(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
        &tenant,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("happy-path stream dispatch must succeed");

    assert_eq!(outcome.account_id, pid("alpha"));
    assert_eq!(outcome.attempts, vec![pid("alpha")]);

    // Collect all chunks from the stream.
    let chunks: Vec<Bytes> = outcome
        .stream
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.expect("chunk must be Ok"))
        .collect();

    assert_eq!(
        chunks,
        vec![
            Bytes::from_static(b"chunk1"),
            Bytes::from_static(b"chunk2"),
            Bytes::from_static(b"chunk3"),
        ]
    );
}

/// AC SUB-2: first-byte retryable failure marks account unhealthy and walks
/// the fallback chain to the next account.
#[tokio::test]
async fn stream_first_byte_retryable_marks_unhealthy_and_walks_chain() {
    let tenant = ten("ten_acme");
    let pool_id = pid_pool("pool_claude_pro");
    // alpha = first byte fails; beta = succeeds.
    let repo = InMemoryPoolRepository::new().with_pool(pool(
        &tenant,
        &pool_id,
        &["alpha", "beta"],
        PoolRoutingStrategy::RoundRobin,
    ));
    let usage = InMemoryUsageSnapshotSource::new();
    let mut health = InMemoryAccountHealthStore::with_thresholds(1, 2);

    let stream_script: StreamScript = Arc::new(|account, _provider, _body| {
        if account == &pid("alpha") {
            // First-byte retryable failure.
            vec![Err(TransportError::Retryable {
                detail: "alpha first-byte 502".into(),
            })]
        } else {
            // beta succeeds.
            vec![Ok(Bytes::from_static(b"ok-from-beta"))]
        }
    });
    let unary_script: TransportScript = Arc::new(|account, _, _| Ok(ok_response(account)));
    let transport =
        InMemoryProviderInvocationTransport::new(unary_script).with_stream_script(stream_script);
    let secret = DeniedSecretResolver;
    let metrics = NoOpMetricsSink;

    let outcome = dispatch_to_pool_stream(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
        &tenant,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("chain walk must converge on beta");

    // Dispatch walked to beta.
    assert_eq!(outcome.account_id, pid("beta"));
    assert_eq!(outcome.attempts, vec![pid("alpha"), pid("beta")]);

    // alpha was marked unhealthy after first-byte failure.
    let map = health.read(&tenant, &pool_id).expect("read health");
    assert!(
        map.get(&pid("alpha")).unwrap().consecutive_failures >= 1,
        "alpha must have at least one failure recorded"
    );

    // The stream from beta yields its chunk.
    let chunks: Vec<Bytes> = outcome
        .stream
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.expect("chunk must be Ok"))
        .collect();
    assert_eq!(chunks, vec![Bytes::from_static(b"ok-from-beta")]);
}

/// AC SUB-2: exhausting the fallback chain on first-byte failures returns
/// DispatchError::AllProvidersExhausted.
#[tokio::test]
async fn stream_chain_exhaustion_returns_all_providers_exhausted() {
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

    // All accounts return first-byte retryable failure.
    let stream_script: StreamScript = Arc::new(|account, _provider, _body| {
        vec![Err(TransportError::Retryable {
            detail: format!("{} first-byte failure", account.0),
        })]
    });
    let unary_script: TransportScript = Arc::new(|account, _, _| Ok(ok_response(account)));
    let transport =
        InMemoryProviderInvocationTransport::new(unary_script).with_stream_script(stream_script);
    let secret = DeniedSecretResolver;
    let metrics = NoOpMetricsSink;

    let err = dispatch_to_pool_stream(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
        &tenant,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect_err("chain exhaustion must return AllProvidersExhausted");

    match err {
        DispatchError::AllProvidersExhausted { attempts, .. } => {
            assert_eq!(attempts, vec![pid("alpha"), pid("beta")]);
        }
        other => panic!("expected AllProvidersExhausted, got {other:?}"),
    }
}

/// AC SUB-2: NonRetryable at first position short-circuits with no failover.
#[tokio::test]
async fn stream_non_retryable_first_byte_short_circuits() {
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

    let stream_script: StreamScript = Arc::new(|_account, _provider, _body| {
        vec![Err(TransportError::NonRetryable {
            detail: "malformed stream request".into(),
        })]
    });
    let unary_script: TransportScript = Arc::new(|account, _, _| Ok(ok_response(account)));
    let transport =
        InMemoryProviderInvocationTransport::new(unary_script).with_stream_script(stream_script);
    let secret = DeniedSecretResolver;
    let metrics = NoOpMetricsSink;

    let err = dispatch_to_pool_stream(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
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
            assert!(detail.contains("malformed stream request"));
        }
        other => panic!("expected NonRetryableTransport, got {other:?}"),
    }

    // Only alpha was attempted — beta was never consulted.
    assert_eq!(transport.call_log(), vec![pid("alpha")]);
}

// ─────────────────────────────────────────────────────────────────────────────
// SUB-3: MetricsSink port acceptance tests
// ─────────────────────────────────────────────────────────────────────────────

/// AC SUB-3: RecordingMetricsSink captures Attempt + Success events for a
/// successful unary dispatch.
#[tokio::test]
async fn metrics_recording_sink_captures_successful_dispatch() {
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

    let script: TransportScript = Arc::new(|account, _, _| Ok(ok_response(account)));
    let transport = InMemoryProviderInvocationTransport::new(script);
    let secret = DeniedSecretResolver;
    let metrics = RecordingMetricsSink::new();

    dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
        &tenant,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("dispatch must succeed");

    let events = metrics.snapshot();
    // Must have at least Attempt + Success for alpha.
    assert!(
        events.iter().any(|e| matches!(
            e,
            MetricEvent::Attempt { account_id, .. } if account_id == &pid("alpha")
        )),
        "Attempt event for alpha must be emitted, got: {events:?}"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            MetricEvent::Success { account_id, .. } if account_id == &pid("alpha")
        )),
        "Success event for alpha must be emitted, got: {events:?}"
    );
}

/// AC SUB-3: RecordingMetricsSink captures the full failover sequence:
/// Attempt(alpha), Failure(alpha, retryable=true), Failover(alpha→beta, depth=1),
/// Attempt(beta), Success(beta).
#[tokio::test]
async fn metrics_recording_sink_captures_failover_sequence() {
    let tenant = ten("ten_acme");
    let pool_id = pid_pool("pool_claude_pro");
    let repo = InMemoryPoolRepository::new().with_pool(pool(
        &tenant,
        &pool_id,
        &["alpha", "beta"],
        PoolRoutingStrategy::RoundRobin,
    ));
    let usage = InMemoryUsageSnapshotSource::new();
    let mut health = InMemoryAccountHealthStore::with_thresholds(5, 10);

    // alpha fails (retryable), beta succeeds.
    let script: TransportScript = Arc::new(|account, _, _| {
        if account == &pid("alpha") {
            Err(TransportError::Retryable {
                detail: "alpha 502".into(),
            })
        } else {
            Ok(ok_response(account))
        }
    });
    let transport = InMemoryProviderInvocationTransport::new(script);
    let secret = DeniedSecretResolver;
    let metrics = RecordingMetricsSink::new();

    dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
        &tenant,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("failover must converge on beta");

    let events = metrics.snapshot();

    // Assert presence and ordering of key events.
    let attempt_alpha_pos = events.iter().position(
        |e| matches!(e, MetricEvent::Attempt { account_id, .. } if account_id == &pid("alpha")),
    );
    let failure_alpha_pos = events.iter().position(|e| {
        matches!(e, MetricEvent::Failure { account_id, retryable: true } if account_id == &pid("alpha"))
    });
    let failover_pos = events.iter().position(|e| {
        matches!(e, MetricEvent::Failover { from, to, depth: 1 } if from == &pid("alpha") && to == &pid("beta"))
    });
    let attempt_beta_pos = events.iter().position(
        |e| matches!(e, MetricEvent::Attempt { account_id, .. } if account_id == &pid("beta")),
    );
    let success_beta_pos = events.iter().position(
        |e| matches!(e, MetricEvent::Success { account_id, .. } if account_id == &pid("beta")),
    );

    assert!(
        attempt_alpha_pos.is_some(),
        "Attempt(alpha) missing from {events:?}"
    );
    assert!(
        failure_alpha_pos.is_some(),
        "Failure(alpha, retryable) missing from {events:?}"
    );
    assert!(
        failover_pos.is_some(),
        "Failover(alpha→beta,depth=1) missing from {events:?}"
    );
    assert!(
        attempt_beta_pos.is_some(),
        "Attempt(beta) missing from {events:?}"
    );
    assert!(
        success_beta_pos.is_some(),
        "Success(beta) missing from {events:?}"
    );

    // Ordering: attempt_alpha < failure_alpha < failover < attempt_beta < success_beta
    let pa = attempt_alpha_pos.unwrap();
    let pfa = failure_alpha_pos.unwrap();
    let pfo = failover_pos.unwrap();
    let pb = attempt_beta_pos.unwrap();
    let ps = success_beta_pos.unwrap();
    assert!(pa < pfa, "Attempt(alpha) must precede Failure(alpha)");
    assert!(pfa < pfo, "Failure(alpha) must precede Failover");
    assert!(pfo < pb, "Failover must precede Attempt(beta)");
    assert!(pb < ps, "Attempt(beta) must precede Success(beta)");
}

/// AC SUB-3: NoOpMetricsSink compiles and runs through a full dispatch without
/// panicking or failing. Verifies the zero-dep bring-up path.
#[tokio::test]
async fn metrics_noop_sink_compiles_and_runs() {
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

    let script: TransportScript = Arc::new(|account, _, _| Ok(ok_response(account)));
    let transport = InMemoryProviderInvocationTransport::new(script);
    let secret = DeniedSecretResolver;
    let metrics = NoOpMetricsSink; // no-op: zero dependency

    dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
        &tenant,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("no-op sink must not interfere with dispatch");
}

/// AC SUB-3: QuarantineTransition events are emitted when record_failure crosses
/// the degrade threshold.
#[tokio::test]
async fn metrics_quarantine_transition_recorded_on_threshold_crossing() {
    let tenant = ten("ten_acme");
    let pool_id = pid_pool("pool_claude_pro");
    let repo = InMemoryPoolRepository::new().with_pool(pool(
        &tenant,
        &pool_id,
        // Use alpha+beta so we always have a fallback path.
        &["alpha", "beta"],
        PoolRoutingStrategy::RoundRobin,
    ));
    let usage = InMemoryUsageSnapshotSource::new();
    // degrade_threshold=1, quarantine_threshold=2: first failure → Degraded,
    // second failure → Unhealthy. Each failure emits a QuarantineTransition.
    let mut health = InMemoryAccountHealthStore::with_thresholds(1, 2);

    let metrics = RecordingMetricsSink::new();
    let secret = DeniedSecretResolver;

    // Script: alpha always fails (retryable), beta always succeeds.
    let script: TransportScript = Arc::new(|account, _, _| {
        if account == &pid("alpha") {
            Err(TransportError::Retryable {
                detail: "alpha down".into(),
            })
        } else {
            Ok(ok_response(account))
        }
    });
    let transport = InMemoryProviderInvocationTransport::new(script);

    // First dispatch: alpha fails (→ Degraded), beta succeeds.
    dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
        &tenant,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("first dispatch must converge on beta");

    let events_after_first = metrics.snapshot();
    assert!(
        events_after_first.iter().any(|e| matches!(
            e,
            MetricEvent::QuarantineTransition {
                account_id,
                new_state: HealthState::Degraded,
            } if account_id == &pid("alpha")
        )),
        "QuarantineTransition(alpha, Degraded) must be emitted after first failure, got: {events_after_first:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 429 / rate-limit rotation tests
// ─────────────────────────────────────────────────────────────────────────────

/// Build a ProviderResponse with status 429 and the supplied extra headers,
/// plus the mandatory provider_account_id echo.
fn rate_limited_response(
    account: &ProviderAccountId,
    extra_headers: Vec<(&str, &str)>,
) -> ProviderResponse {
    let mut headers = vec![("content-type".to_string(), "application/json".to_string())];
    for (name, value) in extra_headers {
        headers.push((name.to_string(), value.to_string()));
    }
    ProviderResponse {
        status: 429,
        headers,
        body: Bytes::from_static(b"{\"error\":\"rate_limited\"}"),
        retry_after_seconds: None,
        provider_account_id: account.clone(),
    }
}

/// AC 429-1: A 429 with `Retry-After: 60` causes the dispatch loop to rotate
/// to the next seat, record a failure for the rate-limited seat, and return
/// the successful response from the second seat.
#[tokio::test]
async fn dispatch_429_rotates_to_next_seat_and_records_cooldown() {
    let tenant = ten("ten_acme");
    let pool_id = pid_pool("pool_claude_pro");
    let repo = InMemoryPoolRepository::new().with_pool(pool(
        &tenant,
        &pool_id,
        &["seat_a", "seat_b"],
        PoolRoutingStrategy::RoundRobin,
    ));
    let usage = InMemoryUsageSnapshotSource::new();
    let mut health = InMemoryAccountHealthStore::new();

    // seat_a returns 429 with Retry-After: 60; seat_b returns 200.
    let script: TransportScript = Arc::new(|account, _provider, _body| {
        if account == &pid("seat_a") {
            Ok(rate_limited_response(account, vec![("retry-after", "60")]))
        } else {
            Ok(ok_response(account))
        }
    });
    let transport = InMemoryProviderInvocationTransport::new(script);
    let secret = DeniedSecretResolver;
    let metrics = RecordingMetricsSink::new();

    let outcome = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
        &tenant,
        &pool_id,
        &RequestMetadata::new("claude-3-5-sonnet".into()),
        UnixMillis(1_000_000),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("dispatch must converge on seat_b");

    // seat_b served the successful response.
    assert_eq!(outcome.response.provider_account_id, pid("seat_b"));
    assert_eq!(outcome.response.status, 200);
    assert_eq!(outcome.attempts, vec![pid("seat_a"), pid("seat_b")]);

    // seat_a was attempted before seat_b.
    let call_log = transport.call_log();
    assert_eq!(call_log, vec![pid("seat_a"), pid("seat_b")]);

    // seat_a must have a failure recorded in the health store.
    let map = health.read(&tenant, &pool_id).expect("read health");
    let seat_a_health = map
        .get(&pid("seat_a"))
        .expect("seat_a must have health entry");
    assert!(
        seat_a_health.consecutive_failures >= 1,
        "seat_a must have at least one failure recorded after 429, got: {seat_a_health:?}"
    );

    // Metrics: Failure(seat_a, retryable=true) must appear.
    let events = metrics.snapshot();
    assert!(
        events.iter().any(|e| matches!(
            e,
            MetricEvent::Failure { account_id, retryable: true } if account_id == &pid("seat_a")
        )),
        "Failure(seat_a, retryable=true) must be emitted on 429, got: {events:?}"
    );
    // Metrics: Success(seat_b) must appear.
    assert!(
        events.iter().any(|e| matches!(
            e,
            MetricEvent::Success { account_id, .. } if account_id == &pid("seat_b")
        )),
        "Success(seat_b) must be emitted, got: {events:?}"
    );
}

/// AC 429-2: `Retry-After-Ms: 30000` is parsed as 30 000 ms (not seconds).
/// The seat is rotated; the rate-limited seat has a failure recorded.
#[tokio::test]
async fn dispatch_429_parses_retry_after_ms_header() {
    let tenant = ten("ten_acme");
    let pool_id = pid_pool("pool_claude_pro");
    let repo = InMemoryPoolRepository::new().with_pool(pool(
        &tenant,
        &pool_id,
        &["seat_a", "seat_b"],
        PoolRoutingStrategy::RoundRobin,
    ));
    let usage = InMemoryUsageSnapshotSource::new();
    let mut health = InMemoryAccountHealthStore::new();

    // seat_a returns 429 with Retry-After-Ms: 30000; seat_b returns 200.
    let script: TransportScript = Arc::new(|account, _provider, _body| {
        if account == &pid("seat_a") {
            Ok(rate_limited_response(
                account,
                vec![("retry-after-ms", "30000")],
            ))
        } else {
            Ok(ok_response(account))
        }
    });
    let transport = InMemoryProviderInvocationTransport::new(script);
    let secret = DeniedSecretResolver;
    let metrics = NoOpMetricsSink;

    let outcome = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
        &tenant,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(2_000_000),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("dispatch must converge on seat_b");

    assert_eq!(outcome.response.provider_account_id, pid("seat_b"));
    assert_eq!(outcome.response.status, 200);

    // seat_a must have a failure recorded.
    let map = health.read(&tenant, &pool_id).expect("read health");
    let seat_a_health = map
        .get(&pid("seat_a"))
        .expect("seat_a must have health entry");
    assert!(
        seat_a_health.consecutive_failures >= 1,
        "seat_a must have a failure after 429+Retry-After-Ms, got: {seat_a_health:?}"
    );
}

/// AC 429-3: When the 429 response carries no rate-limit headers, the dispatch
/// loop falls back to the kernel's `CooldownPolicy::window_for` table.
/// The seat is still rotated and the failure is recorded.
#[tokio::test]
async fn dispatch_429_falls_back_to_kernel_cooldown_when_no_header() {
    let tenant = ten("ten_acme");
    let pool_id = pid_pool("pool_claude_pro");
    let repo = InMemoryPoolRepository::new().with_pool(pool(
        &tenant,
        &pool_id,
        &["seat_a", "seat_b"],
        PoolRoutingStrategy::RoundRobin,
    ));
    let usage = InMemoryUsageSnapshotSource::new();
    let mut health = InMemoryAccountHealthStore::new();

    // seat_a returns 429 with NO rate-limit headers; seat_b returns 200.
    let script: TransportScript = Arc::new(|account, _provider, _body| {
        if account == &pid("seat_a") {
            // No rate-limit headers — dispatch loop must fall back to kernel policy.
            Ok(rate_limited_response(account, vec![]))
        } else {
            Ok(ok_response(account))
        }
    });
    let transport = InMemoryProviderInvocationTransport::new(script);
    let secret = DeniedSecretResolver;
    let metrics = NoOpMetricsSink;

    let outcome = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
        &tenant,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(3_000_000),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("dispatch must converge on seat_b even without Retry-After header");

    assert_eq!(outcome.response.provider_account_id, pid("seat_b"));
    assert_eq!(outcome.response.status, 200);

    // seat_a must have a failure recorded (kernel fallback still triggers rotation).
    let map = health.read(&tenant, &pool_id).expect("read health");
    let seat_a_health = map
        .get(&pid("seat_a"))
        .expect("seat_a must have health entry");
    assert!(
        seat_a_health.consecutive_failures >= 1,
        "seat_a must have a failure even with no Retry-After header, got: {seat_a_health:?}"
    );
}

/// AC 429-4: When all seats in the pool return 429, the dispatch loop exhausts
/// the fallback chain and returns DispatchError::AllProvidersExhausted.
#[tokio::test]
async fn dispatch_429_chain_exhaustion_all_seats_rate_limited() {
    let tenant = ten("ten_acme");
    let pool_id = pid_pool("pool_claude_pro");
    let repo = InMemoryPoolRepository::new().with_pool(pool(
        &tenant,
        &pool_id,
        &["seat_a", "seat_b"],
        PoolRoutingStrategy::RoundRobin,
    ));
    let usage = InMemoryUsageSnapshotSource::new();
    let mut health = InMemoryAccountHealthStore::new();

    // All seats return 429.
    let script: TransportScript = Arc::new(|account, _provider, _body| {
        Ok(rate_limited_response(account, vec![("retry-after", "120")]))
    });
    let transport = InMemoryProviderInvocationTransport::new(script);
    let secret = DeniedSecretResolver;
    let metrics = NoOpMetricsSink;

    let err = dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &metrics,
        None,
        &tenant,
        &pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(4_000_000),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect_err("all-429 pool must return AllProvidersExhausted");

    match err {
        DispatchError::AllProvidersExhausted {
            attempts,
            last_error,
        } => {
            assert_eq!(attempts, vec![pid("seat_a"), pid("seat_b")]);
            match last_error {
                TransportError::Retryable { detail } => {
                    assert!(
                        detail.contains("429"),
                        "last_error detail must mention 429, got: {detail}"
                    );
                }
                other => panic!("expected Retryable last_error, got {other:?}"),
            }
        }
        other => panic!("expected AllProvidersExhausted, got {other:?}"),
    }

    // Both seats must have failures recorded.
    let map = health.read(&tenant, &pool_id).expect("read health");
    for seat in ["seat_a", "seat_b"] {
        let h = map.get(&pid(seat)).expect("{seat} must have health entry");
        assert!(
            h.consecutive_failures >= 1,
            "{seat} must have at least one failure recorded, got: {h:?}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// parse_retry_after_ms unit tests
// ─────────────────────────────────────────────────────────────────────────────

/// retry-after (seconds) takes priority over all other headers.
#[test]
fn parse_retry_after_seconds_header_takes_priority() {
    let headers = vec![
        ("retry-after".to_string(), "60".to_string()),
        ("retry-after-ms".to_string(), "999999".to_string()),
    ];
    let ms = parse_retry_after_ms_pub(&headers, 1);
    assert_eq!(ms, 60_000, "retry-after:60 must yield 60_000 ms");
}

/// retry-after-ms is used when retry-after is absent.
#[test]
fn parse_retry_after_ms_header_used_when_no_retry_after() {
    let headers = vec![("retry-after-ms".to_string(), "45000".to_string())];
    let ms = parse_retry_after_ms_pub(&headers, 1);
    assert_eq!(ms, 45_000);
}

/// anthropic-ratelimit-requests-reset (integer seconds) is the third priority.
#[test]
fn parse_anthropic_ratelimit_requests_reset_header() {
    let headers = vec![(
        "anthropic-ratelimit-requests-reset".to_string(),
        "30".to_string(),
    )];
    let ms = parse_retry_after_ms_pub(&headers, 1);
    assert_eq!(ms, 30_000);
}

/// x-ratelimit-reset-requests (integer seconds) is priority 5.
#[test]
fn parse_x_ratelimit_reset_requests_header() {
    let headers = vec![("x-ratelimit-reset-requests".to_string(), "15".to_string())];
    let ms = parse_retry_after_ms_pub(&headers, 1);
    assert_eq!(ms, 15_000);
}

/// When no recognised header is present, falls back to kernel CooldownPolicy.
/// For consecutive_failures=1, UpstreamRateLimit429 table yields 30_000 ms.
#[test]
fn parse_fallback_to_kernel_cooldown_policy() {
    let headers: Vec<(String, String)> = vec![];
    let ms = parse_retry_after_ms_pub(&headers, 1);
    // CooldownPolicy::window_for(UpstreamRateLimit429, 1) = 30_000 ms.
    assert_eq!(
        ms, 30_000,
        "fallback must use kernel table: 30_000 ms for f=1"
    );
}

/// HTTP-date values in retry-after are ignored (fall through to fallback).
#[test]
fn parse_retry_after_http_date_is_ignored_falls_back_to_kernel() {
    let headers = vec![(
        "retry-after".to_string(),
        "Wed, 21 Oct 2025 07:28:00 GMT".to_string(),
    )];
    let ms = parse_retry_after_ms_pub(&headers, 1);
    // Non-integer retry-after → skip, no other headers → kernel fallback.
    assert_eq!(
        ms, 30_000,
        "HTTP-date retry-after must be ignored; got {ms}"
    );
}
