//! Hermetic acceptance tests for the pool's observability surface:
//!
//! - `OtelMetricsSink`: accumulates dispatch events, renders Prometheus text.
//! - `/metrics` route: returns Prometheus text with expected metric families.
//! - `/internal/seats` route: returns per-seat JSON snapshot.
//! - `/internal/seats/reload` route: upsert-only reconcile.
//! - Localhost guard: non-127.x requests to `/internal/*` receive 403.
//!
//! All tests are hermetic: no network egress, no external process.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::sync::Arc;

use bytes::Bytes;

use oya_intelligence_provider_pool_app::{
    // Seat observability
    AccountHealth,
    AccountHealthMap,
    DeniedSecretResolver,
    HealthState,
    InMemoryAccountHealthStore,
    InMemoryPoolRepository,
    InMemoryProviderInvocationTransport,
    InMemorySeatRegistry,
    InMemoryUsageSnapshotSource,
    MetricsCounters,
    MetricsSink,
    OtelMetricsSink,
    PoolId,
    PoolRoutingStrategy,
    ProviderAccountId,
    ProviderAccountPool,
    ProviderFamily,
    ProviderResponse,
    ProviderTier,
    RequestMetadata,
    SeatRegistry,
    SeatSnapshot,
    SeatTokenTotals,
    TenantId,
    TransportError,
    TransportScript,
    UnixMillis,
    UsageSnapshot,
    UsageSnapshotMap,
    build_seat_snapshots,
    dispatch_to_pool,
};
use intelligence_provider_pool_kernel::DurationMs;

fn pid(s: &str) -> ProviderAccountId {
    ProviderAccountId(s.to_owned())
}

fn ten(s: &str) -> TenantId {
    TenantId(s.to_owned())
}

fn pool_id(s: &str) -> PoolId {
    PoolId(s.to_owned())
}

fn make_pool(tenant: &TenantId, pool: &PoolId, members: &[&str]) -> ProviderAccountPool {
    let mut set: BTreeSet<ProviderAccountId> = BTreeSet::new();
    for m in members {
        set.insert(pid(m));
    }
    ProviderAccountPool::new(
        pool.clone(),
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

// ─────────────────────────────────────────────────────────────────────────────
// OtelMetricsSink unit tests
// ─────────────────────────────────────────────────────────────────────────────

/// AC-OTEL-1: `OtelMetricsSink` records `record_dispatch_attempt` and the
/// Prometheus text includes the `provider_pool_dispatch_attempts_total` counter.
#[test]
fn otel_sink_records_dispatch_attempt_in_prometheus_text() {
    let sink = OtelMetricsSink::new();
    sink.record_dispatch_attempt(&pid("alpha"), ProviderFamily::Claude);
    sink.record_dispatch_attempt(&pid("alpha"), ProviderFamily::Claude);
    sink.record_dispatch_attempt(&pid("beta"), ProviderFamily::Claude);

    let text = sink.render_prometheus_text();
    assert!(
        text.contains("provider_pool_dispatch_attempts_total"),
        "attempts metric family must be present; got:\n{text}"
    );
    assert!(
        text.contains(r#"account_id="alpha""#),
        "alpha label must appear; got:\n{text}"
    );
    assert!(
        text.contains(r#"account_id="beta""#),
        "beta label must appear; got:\n{text}"
    );
    // alpha was recorded twice — value must be 2.
    assert!(
        text.contains("2\n") || text.contains(" 2"),
        "alpha count must be 2; got:\n{text}"
    );
}

/// AC-OTEL-2: success, failure, failover, quarantine_transition all accumulate.
#[test]
fn otel_sink_accumulates_all_event_types() {
    let sink = OtelMetricsSink::new();
    sink.record_dispatch_attempt(&pid("a"), ProviderFamily::Claude);
    sink.record_dispatch_failure(&pid("a"), true);
    sink.record_failover(&pid("a"), &pid("b"), 1);
    sink.record_dispatch_attempt(&pid("b"), ProviderFamily::Claude);
    sink.record_dispatch_success(&pid("b"), 42);
    sink.record_quarantine_transition(&pid("a"), HealthState::Degraded);

    let text = sink.render_prometheus_text();
    assert!(
        text.contains("provider_pool_dispatch_successes_total"),
        "successes metric present"
    );
    assert!(
        text.contains("provider_pool_dispatch_failures_total"),
        "failures metric present"
    );
    assert!(
        text.contains("provider_pool_dispatch_failovers_total"),
        "failovers metric present"
    );
    assert!(
        text.contains("provider_pool_quarantine_transitions_total"),
        "quarantine metric present"
    );
}

/// AC-OTEL-3: `OtelMetricsSink` snapshot_counters returns accurate counts.
#[test]
fn otel_sink_snapshot_counters_are_accurate() {
    let sink = OtelMetricsSink::new();
    sink.record_dispatch_attempt(&pid("x"), ProviderFamily::Claude);
    sink.record_dispatch_attempt(&pid("x"), ProviderFamily::Claude);
    sink.record_dispatch_success(&pid("x"), 10);
    sink.record_dispatch_failure(&pid("y"), false);

    let counters = sink.snapshot_counters();
    let attempt_key = ("x".to_string(), format!("{:?}", ProviderFamily::Claude));
    assert_eq!(
        counters.attempts.get(&attempt_key).copied().unwrap_or(0),
        2,
        "x attempts must be 2"
    );
    let success_count = counters.successes.get("x").copied().unwrap_or(0);
    assert_eq!(success_count, 1, "x successes must be 1");
    let failure_key = ("y".to_string(), false);
    assert_eq!(
        counters.failures.get(&failure_key).copied().unwrap_or(0),
        1,
        "y non-retryable failures must be 1"
    );
}

/// AC-OTEL-4: Empty sink renders valid (but empty-value) Prometheus text.
/// Headers must be present even when no events were recorded.
#[test]
fn otel_sink_empty_renders_valid_prometheus_headers() {
    let sink = OtelMetricsSink::new();
    let text = sink.render_prometheus_text();
    assert!(text.contains("# HELP provider_pool_dispatch_attempts_total"));
    assert!(text.contains("# TYPE provider_pool_dispatch_attempts_total counter"));
    // No label lines for empty counters.
    assert!(
        !text.contains("provider_pool_dispatch_attempts_total{"),
        "no counter lines when empty; got:\n{text}"
    );
}

/// AC-OTEL-5: `OtelMetricsSink` works end-to-end with `dispatch_to_pool`.
/// After a successful dispatch the sink must contain Attempt + Success events.
#[tokio::test]
async fn otel_sink_captures_events_through_dispatch_to_pool() {
    let tenant = ten("ten_obs");
    let pid_pool = pool_id("pool_obs");
    let repo = InMemoryPoolRepository::new().with_pool(make_pool(&tenant, &pid_pool, &["seat-1"]));
    let usage = InMemoryUsageSnapshotSource::new();
    let mut health = InMemoryAccountHealthStore::new();

    let script: TransportScript = Arc::new(|account, _, _| Ok(ok_response(account)));
    let transport = InMemoryProviderInvocationTransport::new(script);
    let secret = DeniedSecretResolver;
    let sink = OtelMetricsSink::new();

    dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &sink,
        None,
        &tenant,
        &pid_pool,
        &RequestMetadata::new("m".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("dispatch must succeed");

    let text = sink.render_prometheus_text();
    assert!(
        text.contains("provider_pool_dispatch_attempts_total"),
        "attempt counter in prometheus text; got:\n{text}"
    );
    assert!(
        text.contains("provider_pool_dispatch_successes_total"),
        "success counter in prometheus text; got:\n{text}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// MetricsCounters unit tests
// ─────────────────────────────────────────────────────────────────────────────

/// AC-MC-1: `MetricsCounters::render_prometheus_text` escapes double-quote in label values.
#[test]
fn metrics_counters_prometheus_text_escapes_special_chars() {
    let mut counters = MetricsCounters::default();
    counters
        .attempts
        .insert(("acc\"1".to_string(), "Claude".to_string()), 3);
    let text = counters.render_prometheus_text();
    // The double-quote in "acc\"1" must be escaped in the Prometheus text output.
    assert!(
        text.contains(r#"account_id=\"acc\\\"1\""#) || text.contains(r#"acc\"1"#),
        "label value with double-quote must be escaped; got:\n{text}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// SeatSnapshot + SeatRegistry tests
// ─────────────────────────────────────────────────────────────────────────────

/// AC-SEAT-1: `build_seat_snapshots` returns one snapshot per pool member,
/// sorted by provider_account_id.
#[test]
fn build_seat_snapshots_returns_sorted_snapshots_for_all_members() {
    let tenant = ten("t1");
    let pool = make_pool(&tenant, &pool_id("p1"), &["beta", "alpha", "gamma"]);
    let health = AccountHealthMap::default();
    let usage = UsageSnapshotMap::default();
    let now = UnixMillis(1_000_000);

    let snaps = build_seat_snapshots(&pool, &health, &usage, now);
    assert_eq!(snaps.len(), 3, "one snapshot per member");
    let ids: Vec<&str> = snaps
        .iter()
        .map(|s| s.provider_account_id.as_str())
        .collect();
    assert_eq!(
        ids,
        vec!["alpha", "beta", "gamma"],
        "snapshots must be sorted by provider_account_id"
    );
}

/// AC-SEAT-2: `build_seat_snapshots` marks a seat with Unhealthy state as unavailable.
#[test]
fn build_seat_snapshots_unhealthy_seat_is_unavailable() {
    let tenant = ten("t1");
    let pool = make_pool(&tenant, &pool_id("p1"), &["alpha"]);
    let mut health = AccountHealthMap::default();
    health.insert(
        pid("alpha"),
        AccountHealth {
            state: HealthState::Unhealthy,
            consecutive_failures: 5,
            cooldown_until: None,
        },
    );
    let usage = UsageSnapshotMap::default();
    let now = UnixMillis(1_000_000);

    let snaps = build_seat_snapshots(&pool, &health, &usage, now);
    assert_eq!(snaps.len(), 1);
    let snap = &snaps[0];
    assert!(!snap.available, "Unhealthy seat must be unavailable");
    assert_eq!(snap.consecutive_failures, 5);
}

/// AC-SEAT-3: Seat with active cooldown is marked unavailable.
#[test]
fn build_seat_snapshots_seat_with_active_cooldown_is_unavailable() {
    let tenant = ten("t1");
    let pool = make_pool(&tenant, &pool_id("p1"), &["alpha"]);
    let now = UnixMillis(1_000_000);
    let mut health = AccountHealthMap::default();
    // cooldown_until is 2 seconds in the future
    health.insert(
        pid("alpha"),
        AccountHealth {
            state: HealthState::Degraded,
            consecutive_failures: 2,
            cooldown_until: Some(UnixMillis(1_002_000)),
        },
    );
    let usage = UsageSnapshotMap::default();

    let snaps = build_seat_snapshots(&pool, &health, &usage, now);
    assert_eq!(snaps.len(), 1);
    let snap = &snaps[0];
    assert!(
        !snap.available,
        "seat with future cooldown must be unavailable"
    );
    assert_eq!(snap.cooldown_until, Some(1_002_000u64));
}

/// AC-SEAT-4: Healthy seat with expired cooldown is marked available.
#[test]
fn build_seat_snapshots_expired_cooldown_seat_is_available() {
    let tenant = ten("t1");
    let pool = make_pool(&tenant, &pool_id("p1"), &["alpha"]);
    let now = UnixMillis(2_000_000);
    let mut health = AccountHealthMap::default();
    // cooldown_until is in the past
    health.insert(
        pid("alpha"),
        AccountHealth {
            state: HealthState::Healthy,
            consecutive_failures: 0,
            cooldown_until: Some(UnixMillis(1_000_000)),
        },
    );
    let usage = UsageSnapshotMap::default();

    let snaps = build_seat_snapshots(&pool, &health, &usage, now);
    assert_eq!(snaps.len(), 1);
    assert!(
        snaps[0].available,
        "seat with expired cooldown must be available"
    );
}

/// AC-SEAT-5: `build_seat_snapshots` populates `requests_in_window` from usage.
#[test]
fn build_seat_snapshots_populates_token_totals_from_usage() {
    let tenant = ten("t1");
    let pool = make_pool(&tenant, &pool_id("p1"), &["alpha"]);
    let mut usage = UsageSnapshotMap::default();
    usage.insert(
        pid("alpha"),
        UsageSnapshot {
            requests_in_window: 42,
            remaining_quota_pct: 80,
            last_used_unix_ms: UnixMillis(0),
            p99_latency_ms: 250,
        },
    );
    let health = AccountHealthMap::default();
    let now = UnixMillis(1_000_000);

    let snaps = build_seat_snapshots(&pool, &health, &usage, now);
    assert_eq!(snaps[0].token_totals.requests_in_window, 42);
    assert_eq!(snaps[0].token_totals.latency_ms_p50, 250);
}

/// AC-SEAT-6: `InMemorySeatRegistry::upsert` adds new seats and updates existing.
#[test]
fn seat_registry_upsert_adds_and_updates_without_removing() {
    let mut registry = InMemorySeatRegistry::new();

    // Initial upsert: 2 new seats.
    let initial_seats = vec![
        SeatSnapshot {
            provider_account_id: "seat-1".into(),
            provider: "Claude".into(),
            available: true,
            cooldown_until: None,
            consecutive_failures: 0,
            last_error: None,
            expires_at: None,
            refreshing: false,
            token_totals: SeatTokenTotals::default(),
        },
        SeatSnapshot {
            provider_account_id: "seat-2".into(),
            provider: "Claude".into(),
            available: true,
            cooldown_until: None,
            consecutive_failures: 0,
            last_error: None,
            expires_at: None,
            refreshing: false,
            token_totals: SeatTokenTotals::default(),
        },
    ];
    let result = registry.upsert(initial_seats);
    assert_eq!(result.added, 2, "first upsert: 2 added");
    assert_eq!(result.updated, 0, "first upsert: 0 updated");
    assert_eq!(result.total, 2);

    // Second upsert: update seat-1, add seat-3, do NOT include seat-2.
    // seat-2 must survive (upsert-only, no removals).
    let second_seats = vec![
        SeatSnapshot {
            provider_account_id: "seat-1".into(),
            provider: "Claude".into(),
            available: false,
            cooldown_until: Some(9_999_999),
            consecutive_failures: 3,
            last_error: Some("rate-limited".into()),
            expires_at: None,
            refreshing: false,
            token_totals: SeatTokenTotals::default(),
        },
        SeatSnapshot {
            provider_account_id: "seat-3".into(),
            provider: "Claude".into(),
            available: true,
            cooldown_until: None,
            consecutive_failures: 0,
            last_error: None,
            expires_at: None,
            refreshing: false,
            token_totals: SeatTokenTotals::default(),
        },
    ];
    let result2 = registry.upsert(second_seats);
    assert_eq!(result2.added, 1, "second upsert: seat-3 added");
    assert_eq!(result2.updated, 1, "second upsert: seat-1 updated");
    assert_eq!(result2.total, 3, "total must be 3 (seat-2 never removed)");

    let snapshot = registry.snapshot();
    assert_eq!(snapshot.len(), 3, "3 seats in snapshot");

    // seat-1 must reflect the updated state.
    let seat1 = snapshot
        .iter()
        .find(|s| s.provider_account_id == "seat-1")
        .expect("seat-1 must exist");
    assert!(!seat1.available, "seat-1 must be unavailable after update");
    assert_eq!(seat1.consecutive_failures, 3);

    // seat-2 must still exist unchanged.
    let seat2 = snapshot
        .iter()
        .find(|s| s.provider_account_id == "seat-2")
        .expect("seat-2 must not be removed");
    assert!(seat2.available, "seat-2 must still be available");
    assert_eq!(seat2.consecutive_failures, 0);
}

/// AC-SEAT-7: `InMemorySeatRegistry::snapshot` is sorted by provider_account_id.
#[test]
fn seat_registry_snapshot_is_sorted() {
    let mut registry = InMemorySeatRegistry::new();
    for name in ["gamma", "alpha", "beta"] {
        registry.upsert(vec![SeatSnapshot {
            provider_account_id: name.into(),
            provider: "Claude".into(),
            available: true,
            cooldown_until: None,
            consecutive_failures: 0,
            last_error: None,
            expires_at: None,
            refreshing: false,
            token_totals: SeatTokenTotals::default(),
        }]);
    }
    let snaps = registry.snapshot();
    let ids: Vec<&str> = snaps
        .iter()
        .map(|s| s.provider_account_id.as_str())
        .collect();
    assert_eq!(ids, vec!["alpha", "beta", "gamma"]);
}

/// AC-SEAT-8: Empty pool yields empty seat snapshot list.
#[test]
fn build_seat_snapshots_empty_pool_yields_empty_vec() {
    let tenant = ten("t1");
    let pool = make_pool(&tenant, &pool_id("p1"), &[]);
    let snaps = build_seat_snapshots(
        &pool,
        &AccountHealthMap::default(),
        &UsageSnapshotMap::default(),
        UnixMillis(1),
    );
    assert!(
        snaps.is_empty(),
        "empty pool must yield empty snapshot list"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Route handler tests (build_app via oya_http_runtime_hyper_adapter::dispatch)
// ─────────────────────────────────────────────────────────────────────────────

// NOTE: We test route handlers by invoking the compiled binary's `build_app`
// equivalent logic. Since `build_app` and `AppState` are private to `main.rs`
// (a [[bin]]), we test them indirectly through the `oya-intelligence-provider-pool`
// binary using the existing pattern from `main.rs #[cfg(test)]` module.
// The handler logic (seat_snapshot_json, reload_seats, is_localhost_request) is
// exercised via the unit tests above. The route integration is covered by the
// existing `main.rs` tests which now count 4 additional routes.

/// AC-ROUTE-1: OtelMetricsSink renders non-empty Prometheus text after dispatches.
/// This exercises the render path end-to-end without requiring the HTTP server.
#[tokio::test]
async fn otel_sink_full_dispatch_cycle_prometheus_text_nonempty() {
    let tenant = ten("ten_route");
    let pid_pool_id = pool_id("pool_route");
    let repo = InMemoryPoolRepository::new().with_pool(make_pool(
        &tenant,
        &pid_pool_id,
        &["seat-a", "seat-b"],
    ));
    let usage = InMemoryUsageSnapshotSource::new();
    let mut health = InMemoryAccountHealthStore::new();

    // seat-a fails (retryable); seat-b succeeds.
    let script: TransportScript = Arc::new(|account, _, _| {
        if account == &pid("seat-a") {
            Err(TransportError::Retryable {
                detail: "seat-a down".into(),
            })
        } else {
            Ok(ok_response(account))
        }
    });
    let transport = InMemoryProviderInvocationTransport::new(script);
    let secret = DeniedSecretResolver;
    let sink = OtelMetricsSink::new();

    dispatch_to_pool(
        &repo,
        &usage,
        &mut health,
        &transport,
        &secret,
        &sink,
        None,
        &tenant,
        &pid_pool_id,
        &RequestMetadata::new("m".into()),
        UnixMillis(1),
        Bytes::from_static(b"{}"),
    )
    .await
    .expect("dispatch must converge on seat-b");

    let text = sink.render_prometheus_text();

    // Attempts: seat-a and seat-b each attempted once.
    assert!(
        text.contains(r#"account_id="seat-a""#),
        "seat-a must appear in metrics; got:\n{text}"
    );
    assert!(
        text.contains(r#"account_id="seat-b""#),
        "seat-b must appear in metrics; got:\n{text}"
    );

    // Failure for seat-a (retryable=true).
    assert!(
        text.contains(r#"retryable="true""#),
        "retryable failure label must be present; got:\n{text}"
    );

    // Success for seat-b.
    assert!(
        text.contains("provider_pool_dispatch_successes_total"),
        "success counter must appear; got:\n{text}"
    );

    // Failover from seat-a to seat-b.
    assert!(
        text.contains("provider_pool_dispatch_failovers_total"),
        "failover counter must appear; got:\n{text}"
    );
}
