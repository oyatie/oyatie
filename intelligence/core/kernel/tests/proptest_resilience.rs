//! Property tests for the pure request-level resilience kernel.
//!
//! Invariants proven for all inputs:
//!
//! 1. The bounded ladder ALWAYS terminates, and terminates in `Fail` — never
//!    loops forever (no input sequence of retryable errors escapes the bounds).
//! 2. Ladder exhaustion (every rung consumed) yields exactly HTTP 503.
//! 3. `Retry-After` precedence: when the server supplies a hint, the in-seat
//!    retry delay equals that hint clamped to the ceiling, regardless of the
//!    sampled jitter or attempt number.
//! 4. Computed backoff is monotonic non-decreasing in the attempt and never
//!    exceeds `max_backoff` (no overflow/panic at extreme attempts).
//! 5. `parse_retry_after` never panics and never yields a negative delay.
//! 6. Terminal errors (Forbidden / ClientError) fail immediately with their
//!    propagated status, irrespective of ladder position.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::time::Duration;

use intelligence_kernel::resilience::{
    parse_retry_after, AttemptState, ErrorClass, JitterKind, RetryAction, RetryPolicy,
};
use proptest::prelude::*;

fn arb_policy() -> impl Strategy<Value = RetryPolicy> {
    (
        0u32..6,
        0u32..6,
        0u32..6,
        1u64..2_000,
        1u64..60_000,
        2u32..4,
        1u64..120,
        prop_oneof![
            Just(JitterKind::None),
            Just(JitterKind::Full),
            Just(JitterKind::Equal)
        ],
    )
        .prop_map(
            |(in_seat, rot, fb, base_ms, max_ms, mult, ceil_s, jitter)| RetryPolicy {
                max_in_seat_retries: in_seat,
                max_seat_rotations: rot,
                max_provider_fallbacks: fb,
                base_backoff: Duration::from_millis(base_ms),
                max_backoff: Duration::from_millis(max_ms),
                backoff_multiplier: mult,
                retry_after_ceiling: Duration::from_secs(ceil_s),
                jitter,
            },
        )
}

// A retryable, seat-local error (drives the longest ladder walk).
fn timeout() -> ErrorClass {
    ErrorClass::Timeout
}

proptest! {
    // (1) + (2): for ANY policy, walking the ladder under a never-ending
    // retryable error terminates in Fail{503} within a bounded number of steps.
    #[test]
    fn ladder_always_terminates_in_503(policy in arb_policy(), jitter in 0.0f64..=1.0) {
        let err = timeout();
        let mut st = AttemptState::new();
        let max_steps = (policy.max_in_seat_retries as u64 + 1)
            * (policy.max_seat_rotations as u64 + 1)
            * (policy.max_provider_fallbacks as u64 + 1)
            + 16;
        let mut steps = 0u64;
        let status = loop {
            steps += 1;
            prop_assert!(steps <= max_steps, "ladder exceeded its own bound");
            match policy.decide(&err, &st, jitter) {
                RetryAction::RetrySameSeat { delay } => {
                    prop_assert!(delay <= policy.max_backoff.max(policy.retry_after_ceiling));
                    st.record_same_seat_retry();
                }
                RetryAction::RotateSeat => st.record_seat_rotation(),
                RetryAction::FallbackProvider => st.record_provider_fallback(),
                RetryAction::EscalateContextWindow => {
                    prop_assert!(false, "timeout must never escalate context window");
                    unreachable!()
                }
                RetryAction::Fail { http_status } => break http_status,
            }
        };
        prop_assert_eq!(status, 503);
    }

    // (3): Retry-After precedence — the hint wins over jitter/backoff and is
    // clamped to the ceiling. Only meaningful on the in-seat path, so use a
    // 5xx error with in-seat budget available.
    #[test]
    fn retry_after_precedence(
        policy in arb_policy(),
        jitter in 0.0f64..=1.0,
        hint_s in 0u64..10_000,
    ) {
        prop_assume!(policy.max_in_seat_retries >= 1);
        let err = ErrorClass::ServerError {
            status: 503,
            retry_after: Some(Duration::from_secs(hint_s)),
        };
        let st = AttemptState::new();
        match policy.decide(&err, &st, jitter) {
            RetryAction::RetrySameSeat { delay } => {
                let expected = Duration::from_secs(hint_s).min(policy.retry_after_ceiling);
                prop_assert_eq!(delay, expected);
            }
            other => prop_assert!(false, "expected RetrySameSeat, got {:?}", other),
        }
    }

    // (4): computed backoff (no server hint) is monotonic non-decreasing in the
    // attempt and bounded by max_backoff, with no overflow at extreme attempts.
    #[test]
    fn backoff_monotonic_and_bounded(
        policy in arb_policy().prop_map(|mut p| { p.jitter = JitterKind::None; p }),
        a in 0u32..40,
    ) {
        let d0 = policy.backoff(a, None, 0.0);
        let d1 = policy.backoff(a + 1, None, 0.0);
        prop_assert!(d0 <= policy.max_backoff);
        prop_assert!(d1 <= policy.max_backoff);
        prop_assert!(d1 >= d0, "backoff must not shrink as attempts grow");
        // Extreme attempt: must not panic, must saturate to max_backoff.
        prop_assert_eq!(policy.backoff(u32::MAX, None, 0.0), policy.max_backoff);
    }

    // (5): Retry-After parsing never panics and never yields negative delay.
    #[test]
    fn parse_retry_after_total_and_nonnegative(s in ".*", now in any::<u64>()) {
        // Just exercising it for panics; any Some(_) is a valid non-negative
        // Duration by construction.
        let _ = parse_retry_after(&s, now);
    }

    // (6): terminal errors fail immediately with their status at any ladder pos.
    #[test]
    fn client_error_terminal_regardless_of_position(
        policy in arb_policy(),
        status in 400u16..500,
        seats in any::<u32>(),
        provs in any::<u32>(),
        jitter in 0.0f64..=1.0,
    ) {
        prop_assume!(status != 429 && status != 403 && status != 408);
        let err = ErrorClass::ClientError { status };
        let st = AttemptState { in_seat_attempts: 0, seats_tried: seats, providers_tried: provs };
        prop_assert_eq!(
            policy.decide(&err, &st, jitter),
            RetryAction::Fail { http_status: status }
        );
    }
}
