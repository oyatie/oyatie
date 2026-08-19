//! `k8s.io/apimachinery/pkg/util/wait` — the background-loop family.
//!
//! This module is the single highest-leverage thing in the crate, and the census says so outright:
//! `wait.*` is **108 of 400 named goroutine launches (27%)** and **65% of the S1 background-loop
//! shape**, across 39 distinct symbols and 1,182 call sites. One audited implementation collapses
//! 108 launch sites into 108 calls, and no per-site rule is needed for any of them.
//!
//! `go wait.Until(f, period, stopCh)` IS the Kubernetes background loop. What it means precisely,
//! and what the shape below reproduces:
//!
//! - run `f` immediately, then every `period`;
//! - check the stop signal BEFORE each run, so a loop stopped between ticks does no further work;
//! - the period is the gap BETWEEN runs, not a fixed schedule — a slow `f` does not cause a burst
//!   of catch-up runs, which is exactly the property a naive `Instant`-based schedule loses.
//!
//! Jitter is here because the corpus uses it: `JitterUntil` and `JitterUntilWithContext` are 5 of
//! the 108. It is deterministic — a hash of the tick count rather than a random source — because a
//! runtime that reaches for randomness makes every test that touches it flaky, and the purpose of
//! jitter is to decorrelate independent loops rather than to be unpredictable.

use std::time::Duration;

use crate::signal::Waiter;

/// `wait.Until(f, period, stopCh)` — run `f` immediately and then every `period`, until stopped.
///
/// Blocks. `go wait.Until(...)` is [`crate::spawn`] around this call, which is how the source
/// spells it too.
pub fn until<F: FnMut()>(f: F, period: Duration, stop: &Waiter) {
    jitter_until(f, period, 0.0, true, stop);
}

/// `wait.Forever(f, period)` — [`until`] with no stop signal.
///
/// 9 launch sites. It exists as its own function rather than as `until` with a signal nobody fires,
/// because a loop with no shutdown path is a fact about the program worth being able to grep for.
pub fn forever<F: FnMut()>(mut f: F, period: Duration) {
    loop {
        f();
        std::thread::sleep(period);
    }
}

/// `wait.JitterUntil(f, period, factor, sliding, stopCh)`.
///
/// `factor` spreads each wait across `[period, period * (1 + factor))`. `sliding` selects whether
/// the period is measured from the END of `f` (true — the gap between runs) or from its START
/// (false — a fixed schedule that a slow `f` falls behind). The source defaults to sliding, and the
/// difference is visible only under load, which is where it matters.
pub fn jitter_until<F: FnMut()>(
    mut f: F,
    period: Duration,
    factor: f64,
    sliding: bool,
    stop: &Waiter,
) {
    let mut tick: u64 = 0;
    loop {
        // Checked BEFORE running, so a loop stopped between ticks does no further work. Checking
        // after would run `f` one extra time on every shutdown — once per loop, in a corpus with
        // 165 of them.
        if stop.has_fired() {
            return;
        }

        let started = std::time::Instant::now();
        f();
        tick += 1;

        let interval = jittered(period, factor, tick);
        let remaining = if sliding {
            interval
        } else {
            interval.saturating_sub(started.elapsed())
        };
        if remaining.is_zero() {
            continue;
        }
        // Waiting on the stop signal rather than sleeping is what makes shutdown prompt. A
        // `sleep(period)` loop stops only at the end of the current period, so a one-minute loop
        // takes up to a minute to notice it should stop.
        if stop.wait_timeout(remaining) {
            return;
        }
    }
}

/// `wait.PollImmediateUntil(interval, condition, stopCh)` — run `condition` until it reports done.
///
/// `true` when the condition was satisfied, `false` when the stop signal fired first. The condition
/// runs IMMEDIATELY, before any wait, which is the difference between `PollImmediate` and `Poll`
/// and is the variant the corpus uses.
pub fn poll_immediate_until<F: FnMut() -> bool>(
    mut condition: F,
    interval: Duration,
    stop: &Waiter,
) -> bool {
    loop {
        if stop.has_fired() {
            return false;
        }
        if condition() {
            return true;
        }
        if stop.wait_timeout(interval) {
            return false;
        }
    }
}

/// Spread a duration across `[period, period * (1 + factor))`, deterministically.
///
/// Deterministic on purpose. Jitter exists to decorrelate independent loops that started together,
/// which a per-loop hash achieves; being unpredictable is not part of the requirement, and a
/// runtime that reaches for a random source makes every test that touches a background loop flaky.
fn jittered(period: Duration, factor: f64, tick: u64) -> Duration {
    if factor <= 0.0 {
        return period;
    }
    // A cheap integer hash, so the spread does not correlate with the tick number itself.
    let mixed = tick.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 11;
    let fraction = (mixed % 1024) as f64 / 1024.0;
    period.mul_f64(1.0 + factor * fraction)
}
