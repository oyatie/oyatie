//! Tests for the `MaxHeadroom` selection strategy and the split auth-vs-rate-limit
//! cooldown ladder.
//!
//! Coverage:
//! - Unified rate-limit header parsing (`anthropic-ratelimit-unified-*-utilization`).
//! - `MaxHeadroom` picks the least-utilized eligible seat; floor 0.02 keeps a
//!   saturated seat orderable.
//! - `ErrorClass` classification (401/403/invalid_grant/refresh_token_reused →
//!   auth; 429 → rate limit).
//! - Auth failures cool longer and blacklist faster than 429s.
//! - An auth-dead seat is never resurrected by stale utilization (headroom
//!   selector respects cooldown/blacklist eligibility).
//! - 429 backoff is exponential and bounded by the policy max.
//! - Proptest invariants: floored headroom, monotone backoff, deterministic jitter.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::time::{Duration, Instant};

use intelligence_kernel::{
    AgentId, AuthzDecision, AuthzGate, AuthzRequest, CooldownPolicy, ErrorClass, OAuthSubscription,
    Provider, QuotaWindow, QuotaWindowKind, SeatId, SeatOutcome, SelectionStrategy, SubscriptionId,
    SubscriptionPool, SubscriptionPoolError, SubscriptionState, TenantId,
    UnifiedRateLimitUtilization, parse_utilization_percent,
};
use proptest::prelude::*;

struct AllowAll;
impl AuthzGate for AllowAll {
    fn decide(&self, _r: &AuthzRequest<'_>) -> AuthzDecision {
        AuthzDecision::Allow
    }
}

fn tid(s: &str) -> TenantId {
    TenantId::new(s).unwrap()
}
fn aid(s: &str) -> AgentId {
    AgentId::new(s).unwrap()
}
fn sid(s: &str) -> SeatId {
    SeatId::new(s).unwrap()
}

fn sub(seat: &str) -> OAuthSubscription {
    OAuthSubscription::new(
        tid("t"),
        sid(seat),
        SubscriptionId::new(format!("{seat}-sub")).unwrap(),
        Provider::Anthropic,
        SubscriptionState::Active,
        format!("secret-ref://t/{seat}/refresh"),
        0,
    )
}

fn pool(strategy: SelectionStrategy) -> SubscriptionPool {
    SubscriptionPool::new(tid("t"), Provider::Anthropic, strategy)
}

// ---------------------------------------------------------------------------
// Header parsing
// ---------------------------------------------------------------------------

#[test]
fn parse_utilization_percent_normalizes_and_rejects() {
    assert_eq!(parse_utilization_percent("0"), Some(0.0));
    assert_eq!(parse_utilization_percent("50"), Some(0.5));
    assert_eq!(parse_utilization_percent(" 100 "), Some(1.0));
    // Out-of-range clamps to fully utilized rather than producing >1.
    assert_eq!(parse_utilization_percent("150"), Some(1.0));
    // Rejected inputs.
    assert_eq!(parse_utilization_percent("-5"), None);
    assert_eq!(parse_utilization_percent("abc"), None);
    assert_eq!(parse_utilization_percent("NaN"), None);
    assert_eq!(parse_utilization_percent(""), None);
}

#[test]
fn from_headers_parses_5h_7d_and_per_model_case_insensitively() {
    let util = UnifiedRateLimitUtilization::from_headers([
        ("Anthropic-RateLimit-Unified-5h-Utilization", "30"),
        ("anthropic-ratelimit-unified-7d-utilization", "60"),
        ("anthropic-ratelimit-unified-7d-opus-utilization", "80"),
        ("anthropic-ratelimit-unified-7d-sonnet-utilization", "10"),
        ("x-unrelated", "999"),
    ])
    .expect("at least one unified header present");

    assert_eq!(util.five_hour, Some(0.30));
    assert_eq!(util.seven_day, Some(0.60));
    // Per-model retains the worst (opus 80%).
    assert_eq!(util.seven_day_per_model, Some(0.80));
    assert_eq!(util.max_utilization(), Some(0.80));
}

#[test]
fn from_headers_returns_none_when_no_unified_header_present() {
    assert_eq!(
        UnifiedRateLimitUtilization::from_headers([("content-type", "application/json")]),
        None
    );
}

// ---------------------------------------------------------------------------
// MaxHeadroom selection
// ---------------------------------------------------------------------------

#[test]
fn max_headroom_picks_least_utilized_seat() {
    let now = Instant::now();
    let mut pool = pool(SelectionStrategy::MaxHeadroom);
    // seat-busy: 90% utilized → headroom 0.10. seat-idle: 20% → headroom 0.80.
    pool.add_seat(sub("seat-busy")).unwrap();
    pool.add_seat(sub("seat-idle")).unwrap();
    pool.record_reported_utilization(
        &sid("seat-busy"),
        UnifiedRateLimitUtilization {
            five_hour: Some(0.90),
            seven_day: Some(0.40),
            seven_day_per_model: None,
        },
    );
    pool.record_reported_utilization(
        &sid("seat-idle"),
        UnifiedRateLimitUtilization {
            five_hour: Some(0.20),
            seven_day: Some(0.10),
            seven_day_per_model: None,
        },
    );

    assert_eq!(
        pool.select(&aid("a"), &AllowAll, now).unwrap(),
        sid("seat-idle")
    );
}

#[test]
fn max_headroom_floor_keeps_saturated_seat_orderable() {
    let now = Instant::now();
    let mut pool = pool(SelectionStrategy::MaxHeadroom);
    pool.add_seat(sub("only")).unwrap();
    pool.record_reported_utilization(
        &sid("only"),
        UnifiedRateLimitUtilization {
            five_hour: Some(1.0),
            seven_day: Some(1.0),
            seven_day_per_model: Some(1.0),
        },
    );
    // Floor 0.02 → still positive, still selectable as last resort.
    let score = pool.seat_max_headroom_score(&sid("only"), now, 1).unwrap();
    assert!((score - 0.02).abs() < 1e-9, "expected floor 0.02, got {score}");
    assert_eq!(pool.select(&aid("a"), &AllowAll, now).unwrap(), sid("only"));
}

#[test]
fn max_headroom_falls_back_to_quota_window_headroom_when_no_headers() {
    let now = Instant::now();
    let five_hours = Duration::from_secs(5 * 60 * 60);
    let mut pool = pool(SelectionStrategy::MaxHeadroom);
    // No reported utilization; rank by quota-window headroom instead.
    let busy = sub("q-busy").with_quota_windows([QuotaWindow::new(
        QuotaWindowKind::FiveHour,
        100,
        80,
        now + five_hours,
        five_hours,
    )]);
    let idle = sub("q-idle").with_quota_windows([QuotaWindow::new(
        QuotaWindowKind::FiveHour,
        100,
        10,
        now + five_hours,
        five_hours,
    )]);
    pool.add_seat(busy).unwrap();
    pool.add_seat(idle).unwrap();

    assert_eq!(
        pool.select(&aid("a"), &AllowAll, now).unwrap(),
        sid("q-idle")
    );
}

// ---------------------------------------------------------------------------
// ErrorClass classification + SeatOutcome::from_upstream
// ---------------------------------------------------------------------------

#[test]
fn error_class_routes_auth_vs_rate_limit() {
    assert_eq!(ErrorClass::classify(200, None), ErrorClass::Success);
    assert_eq!(ErrorClass::classify(429, None), ErrorClass::RateLimit);
    assert_eq!(ErrorClass::classify(401, None), ErrorClass::AuthFailure);
    assert_eq!(ErrorClass::classify(403, None), ErrorClass::AuthFailure);
    assert_eq!(ErrorClass::classify(500, None), ErrorClass::ServerError);
    assert_eq!(ErrorClass::classify(400, None), ErrorClass::OtherClientError);
    // Error code wins even on a non-auth status code.
    assert_eq!(
        ErrorClass::classify(400, Some("invalid_grant")),
        ErrorClass::AuthFailure
    );
    assert_eq!(
        ErrorClass::classify(200, Some("refresh_token_reused")),
        ErrorClass::AuthFailure
    );

    assert_eq!(SeatOutcome::from_upstream(429, None), SeatOutcome::RateLimited429);
    assert_eq!(SeatOutcome::from_upstream(401, None), SeatOutcome::AuthFailure);
    assert_eq!(
        SeatOutcome::from_upstream(400, Some("refresh_token_reused")),
        SeatOutcome::AuthFailure
    );
    assert_eq!(SeatOutcome::from_upstream(200, None), SeatOutcome::Ok);
}

// ---------------------------------------------------------------------------
// Split cooldown ladder
// ---------------------------------------------------------------------------

#[test]
fn auth_failure_cools_longer_than_rate_limit() {
    let policy = CooldownPolicy::default();
    // First failure, no jitter, to compare nominal bases.
    let no_jitter = CooldownPolicy {
        jitter_fraction: 0.0,
        ..policy
    };
    let rl = no_jitter.cooldown_duration(ErrorClass::RateLimit, 1, 0);
    let auth = no_jitter.cooldown_duration(ErrorClass::AuthFailure, 1, 0);
    assert_eq!(rl, Duration::from_secs(60));
    assert_eq!(auth, Duration::from_secs(30 * 60));
    assert!(auth > rl, "auth cooldown must dwarf the rate-limit cooldown");
}

#[test]
fn auth_failure_blacklists_faster_than_rate_limit() {
    let now = Instant::now();
    let mut pool = pool(SelectionStrategy::FillFirst);
    pool.add_seat(sub("auth-seat")).unwrap();
    let seat = sid("auth-seat");

    // Default auth_blacklist_threshold = 2 → blacklisted on the 3rd auth failure.
    pool.record_outcome(&seat, SeatOutcome::AuthFailure, now).unwrap();
    pool.record_outcome(&seat, SeatOutcome::AuthFailure, now).unwrap();
    // Still only cooling after 2 (advance well past any cooldown to prove it is
    // a cooldown, not a blacklist).
    let far = now + Duration::from_secs(48 * 60 * 60);
    assert!(pool.has_eligible_seat(far), "2 auth failures must still be a cooldown");

    pool.record_outcome(&seat, SeatOutcome::AuthFailure, now).unwrap();
    // 3rd auth failure crosses the threshold → permanent blacklist.
    assert!(
        !pool.has_eligible_seat(now + Duration::from_secs(999_999)),
        "3rd auth failure must blacklist permanently"
    );
    assert_eq!(
        pool.select(&aid("a"), &AllowAll, now + Duration::from_secs(999_999)),
        Err(SubscriptionPoolError::NoEligibleSeat)
    );
}

#[test]
fn auth_dead_seat_is_not_resurrected_by_stale_healthy_utilization() {
    // A seat that looks idle (0% utilization) but has hit the auth blacklist
    // must never be selected by MaxHeadroom — eligibility gates the score.
    let now = Instant::now();
    let mut pool = pool(SelectionStrategy::MaxHeadroom);
    pool.add_seat(sub("dead")).unwrap();
    pool.add_seat(sub("live")).unwrap();
    pool.record_reported_utilization(
        &sid("dead"),
        UnifiedRateLimitUtilization {
            five_hour: Some(0.0),
            seven_day: Some(0.0),
            seven_day_per_model: Some(0.0),
        },
    );
    pool.record_reported_utilization(
        &sid("live"),
        UnifiedRateLimitUtilization {
            five_hour: Some(0.95),
            seven_day: Some(0.95),
            seven_day_per_model: Some(0.95),
        },
    );

    // Kill "dead" via repeated auth failures (threshold 2 → 3 failures).
    for _ in 0..3 {
        pool.record_outcome(&sid("dead"), SeatOutcome::AuthFailure, now).unwrap();
    }

    // Despite "dead" reporting 0% utilization (max headroom), the busy-but-live
    // seat is chosen — the dead seat is ineligible.
    assert_eq!(
        pool.select(&aid("a"), &AllowAll, now + Duration::from_secs(999_999))
            .unwrap(),
        sid("live")
    );
}

#[test]
fn rate_limit_backoff_is_exponential_and_capped() {
    let policy = CooldownPolicy {
        jitter_fraction: 0.0,
        ..CooldownPolicy::default()
    };
    // 60 * 2^(n-1), capped at rate_limit_max (1h).
    assert_eq!(
        policy.cooldown_duration(ErrorClass::RateLimit, 1, 0),
        Duration::from_secs(60)
    );
    assert_eq!(
        policy.cooldown_duration(ErrorClass::RateLimit, 2, 0),
        Duration::from_secs(120)
    );
    assert_eq!(
        policy.cooldown_duration(ErrorClass::RateLimit, 3, 0),
        Duration::from_secs(240)
    );
    // Deep failure: saturates into the 1h cap, never overflows.
    assert_eq!(
        policy.cooldown_duration(ErrorClass::RateLimit, 30, 0),
        Duration::from_secs(60 * 60)
    );
}

#[test]
fn jitter_only_subtracts_keeping_backoff_as_upper_bound() {
    let policy = CooldownPolicy::default(); // jitter 0.2
    let nominal = Duration::from_secs(60);
    for seed in [0u64, 1, 42, u64::MAX, u64::MAX / 2] {
        let jittered = policy.cooldown_duration(ErrorClass::RateLimit, 1, seed);
        assert!(jittered <= nominal, "jitter must not exceed the nominal backoff");
        // Within the 20% jitter band.
        assert!(jittered >= nominal.mul_f64(0.8) - Duration::from_millis(1));
    }
}

// ---------------------------------------------------------------------------
// Proptest invariants
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    // Headroom score is always within [floor, 1.0] for any reported utilization.
    #[test]
    fn prop_headroom_score_is_floored_and_bounded(
        u5 in 0.0f64..=1.0,
        u7 in 0.0f64..=1.0,
        upm in 0.0f64..=1.0,
    ) {
        let now = Instant::now();
        let mut pool = pool(SelectionStrategy::MaxHeadroom);
        pool.add_seat(sub("s")).unwrap();
        pool.record_reported_utilization(
            &sid("s"),
            UnifiedRateLimitUtilization {
                five_hour: Some(u5),
                seven_day: Some(u7),
                seven_day_per_model: Some(upm),
            },
        );
        let score = pool.seat_max_headroom_score(&sid("s"), now, 1).unwrap();
        prop_assert!(score >= 0.02 - 1e-12, "score {score} below floor");
        prop_assert!(score <= 1.0 + 1e-12, "score {score} above 1.0");
        let expected = (1.0 - u5.max(u7).max(upm)).clamp(0.02, 1.0);
        prop_assert!((score - expected).abs() < 1e-9);
    }

    // Rate-limit backoff is monotonically non-decreasing in failure count and
    // never exceeds the configured max, for any jitter seed.
    #[test]
    fn prop_backoff_monotone_and_bounded(
        n in 1u32..40u32,
        seed in any::<u64>(),
    ) {
        let policy = CooldownPolicy { jitter_fraction: 0.0, ..CooldownPolicy::default() };
        let cur = policy.cooldown_duration(ErrorClass::RateLimit, n, seed);
        let max = Duration::from_secs(60 * 60);
        prop_assert!(cur <= max, "backoff {cur:?} exceeds max");
        if n > 1 {
            let prev = policy.cooldown_duration(ErrorClass::RateLimit, n - 1, seed);
            prop_assert!(cur >= prev, "backoff not monotone: {prev:?} -> {cur:?}");
        }
    }

    // Jitter is deterministic: identical inputs yield identical durations, and
    // a jittered duration never exceeds the un-jittered nominal.
    #[test]
    fn prop_jitter_deterministic_and_bounded(
        n in 1u32..10u32,
        seed in any::<u64>(),
    ) {
        let policy = CooldownPolicy::default();
        let a = policy.cooldown_duration(ErrorClass::RateLimit, n, seed);
        let b = policy.cooldown_duration(ErrorClass::RateLimit, n, seed);
        prop_assert_eq!(a, b, "jitter must be deterministic");

        let no_jitter = CooldownPolicy { jitter_fraction: 0.0, ..CooldownPolicy::default() };
        let nominal = no_jitter.cooldown_duration(ErrorClass::RateLimit, n, seed);
        prop_assert!(a <= nominal, "jittered {a:?} exceeds nominal {nominal:?}");
    }
}
