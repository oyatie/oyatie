// Pure, arch-neutral timekeeping math: the counter->nanoseconds conversion
// (`mult`/`shift` derivation + the read-path multiply) and the `timespec` /
// `timeval` field splits the clock syscalls need.
//
// This is the single source of truth for the parts of P3 timekeeping that are
// pure functions of their inputs and therefore identical on every arch: the
// aarch64 generic counter (`CNTPCT_EL0` at `CNTFRQ_EL0` Hz) and the x86_64 TSC
// (`rdtsc` at the PIT-calibrated frequency) both feed the SAME `mult`/
// `shift`/`cycles_to_ns` arithmetic; only the act of *reading* the raw counter
// (a system-register read / `rdtsc`) is arch `unsafe` and lives in the arch
// backends.
//
// Like `layout.rs`/`signal.rs` this file carries NO inner attributes / inner
// doc comments so the out-of-workspace host harness can `include!` it inside a
// plain `mod` body and unit-test the real production code (no copy/drift). It is
// ZERO `unsafe` — const + bit math only — so adding it keeps the `check-tcb.sh`
// forbid-set (`user_layout`) at zero unsafe tokens.
//
// ## Why `mult`/`shift` instead of a divide on the read path
// Reading the clock must be cheap (it is on the `clock_gettime` hot path and,
// later, the vDSO). Linux precomputes a fixed-point scale `(mult, shift)` once
// at init from the counter frequency so the read path is a single `u128`
// multiply + right shift — no division. We mirror Linux's
// `clocks_calc_mult_shift`: pick the largest `shift <= 32` for which
// `mult = (1e9 << shift) / freq` still fits in `u32`, maximising precision while
// keeping `mult` in 32 bits (so `delta * mult` fits comfortably in the `u128`
// intermediate even for a multi-century 64-bit counter).

/// Nanoseconds per second.
pub const NS_PER_SEC: u64 = 1_000_000_000;
/// Microseconds per second.
pub const US_PER_SEC: u64 = 1_000_000;
/// Nanoseconds per microsecond.
pub const NS_PER_US: u64 = 1_000;

/// Fixed wall-clock epoch offset (CLOCK_REALTIME = CLOCK_MONOTONIC + this), in
/// nanoseconds. v1 has no RTC, so REALTIME is a fixed compile-time epoch:
/// **2024-01-01T00:00:00Z = 1_704_067_200 s** since the Unix epoch. Documented
/// as a constant so a later RTC slice can replace it with a live read without
/// touching the read path. (`1_704_067_200 * 1e9` fits in u64.)
pub const WALLCLOCK_OFFSET_SECS: u64 = 1_704_067_200;
pub const WALLCLOCK_OFFSET_NS: u64 = WALLCLOCK_OFFSET_SECS * NS_PER_SEC;

/// The maximum `shift` `calc_mult_shift` will choose. Linux caps the shift; 32
/// is the natural cap here because it keeps `mult` in `u32` for every frequency
/// at or above 1 Hz while leaving the `u128` `delta*mult` product far from
/// overflow (a 64-bit counter times a 32-bit mult is at most ~96 bits).
pub const MAX_SHIFT: u32 = 32;

/// The published, immutable timekeeper base + scale. Pure POD: the arch backend
/// fills it once at init from the live counter + frequency and parks the
/// tick-varying `(counter_at_base, mono_ns_base)` pair in a `ksync::SeqLock`;
/// `mult`/`shift` and the realtime offset are compile-time/init constants read
/// without the lock (see the spec §1.3 "Option A"). Stored here as a struct so
/// the field layout has one documented home shared by both arches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimekeeperData {
    /// Raw counter value sampled when the base was published.
    pub counter_at_base: u64,
    /// Monotonic nanoseconds at the base (v1: 0 — the counter is free-running
    /// and we anchor mono time to the boot sample).
    pub mono_ns_base: u64,
    /// Realtime nanoseconds at the base (v1: `WALLCLOCK_OFFSET_NS`).
    pub real_ns_base: u64,
    /// Fixed-point multiplier (`cycles -> ns`), fits `u32` by construction.
    pub mult: u32,
    /// Fixed-point right-shift applied after the multiply.
    pub shift: u32,
}

impl TimekeeperData {
    /// Build the base from a boot counter sample + frequency. `mono_ns_base` is
    /// 0 (mono time is anchored at the boot sample); `real_ns_base` is the fixed
    /// wall-clock offset; `(mult, shift)` are derived from `freq_hz`.
    pub const fn from_boot_sample(counter_at_base: u64, freq_hz: u64) -> Self {
        let (mult, shift) = calc_mult_shift(freq_hz);
        Self {
            counter_at_base,
            mono_ns_base: 0,
            real_ns_base: WALLCLOCK_OFFSET_NS,
            mult,
            shift,
        }
    }
}

/// Convert a counter `delta` (cycles since the base) to nanoseconds using the
/// precomputed fixed-point scale: `ns = (delta * mult) >> shift`.
///
/// The multiply is done in `u128` so `delta * mult` never overflows: `delta` is
/// a 64-bit counter and `mult` fits in `u32`, so the product is at most ~96
/// bits — well within `u128` — even for a counter that has been running for
/// centuries at GHz. The `>> shift` then lands the result back in the `u64` ns
/// range.
pub const fn cycles_to_ns(delta: u64, mult: u32, shift: u32) -> u64 {
    (((delta as u128) * (mult as u128)) >> shift) as u64
}

/// Linux `clocks_calc_mult_shift`: derive `(mult, shift)` so that
/// `cycles_to_ns(delta, mult, shift) ~= delta * 1e9 / freq_hz` with the read
/// path being a single multiply + shift (no divide).
///
/// We pick the **largest** `shift` in `0..=MAX_SHIFT` for which
/// `mult = (1e9 << shift) / freq_hz` still fits in `u32`, which maximises the
/// fractional precision. Computed in `u128` to keep `1e9 << shift` exact before
/// the divide. Degenerate guards: `freq_hz == 0` is treated as 1 Hz (no
/// divide-by-zero), and a `mult` that would round to 0 is floored to 1 so time
/// still advances on absurdly high frequencies.
pub const fn calc_mult_shift(freq_hz: u64) -> (u32, u32) {
    let freq = if freq_hz == 0 { 1 } else { freq_hz } as u128;
    let ns = NS_PER_SEC as u128;

    // Search downward from MAX_SHIFT for the largest shift whose mult fits u32.
    // `const fn` has no `for`, so use a manual loop.
    let mut shift = MAX_SHIFT;
    loop {
        let mult = (ns << shift) / freq;
        if mult <= u32::MAX as u128 {
            // `mult` fits u32 at this shift. Floor to 1 so time advances even if
            // truncation produced 0 (only possible at freq > 1e9*2^shift).
            let m = if mult == 0 { 1 } else { mult as u32 };
            return (m, shift);
        }
        if shift == 0 {
            // Even shift 0 overflows u32 -> the frequency is below 1 Hz, which
            // our counters never are. Saturate mult to u32::MAX at shift 0 as a
            // defensive floor (keeps the function total).
            return (u32::MAX, 0);
        }
        shift -= 1;
    }
}

/// Split a nanosecond count into a `timespec` `(tv_sec, tv_nsec)` with
/// `0 <= tv_nsec < 1e9`. Slow-path (syscall) helper — a divide here is fine.
pub const fn ns_to_timespec(ns: u64) -> (i64, i64) {
    ((ns / NS_PER_SEC) as i64, (ns % NS_PER_SEC) as i64)
}

/// Split a nanosecond count into a `timeval` `(tv_sec, tv_usec)` with
/// `0 <= tv_usec < 1e6` (microsecond resolution; truncates sub-us). Slow-path
/// helper for `gettimeofday`.
pub const fn ns_to_timeval(ns: u64) -> (i64, i64) {
    let secs = (ns / NS_PER_SEC) as i64;
    let usec = ((ns % NS_PER_SEC) / NS_PER_US) as i64;
    (secs, usec)
}

// ===========================================================================
// Host unit tests (std; run via the out-of-workspace tests-host harness).
//   cargo test --manifest-path crates/arch-aarch64/tests-host/Cargo.toml
// They `include!` this exact file, so they exercise the real production math.
// ===========================================================================
#[cfg(test)]
mod timekeep_tests {
    use super::*;

    /// The frequencies the spec calls out: 1 MHz, 24 MHz (aa QEMU virt CNTFRQ),
    /// 62.5 MHz (aa QEMU virt alt), 1 GHz and 2.5 GHz (x86 TSC).
    const FREQS: [u64; 5] = [
        1_000_000,
        24_000_000,
        62_500_000,
        1_000_000_000,
        2_500_000_000,
    ];

    #[test]
    fn mult_fits_u32_and_shift_capped() {
        for &f in &FREQS {
            let (mult, shift) = calc_mult_shift(f);
            assert!(shift <= MAX_SHIFT, "shift {shift} > MAX_SHIFT for freq {f}");
            // `mult` is a u32 by type; assert it is nonzero so time advances.
            assert!(mult > 0, "mult is 0 for freq {f}");
        }
    }

    #[test]
    fn one_second_of_cycles_is_one_billion_ns_within_tolerance() {
        // cycles_to_ns(freq cycles) should be ~1e9 ns (== 1 second) for every
        // frequency, since `freq` cycles == exactly one second. Tolerance is
        // sub-ppm by construction (largest shift keeping mult in u32); we allow
        // a generous absolute slack to stay robust across freqs.
        for &f in &FREQS {
            let (mult, shift) = calc_mult_shift(f);
            let ns = cycles_to_ns(f, mult, shift);
            let target = NS_PER_SEC;
            let diff = ns.abs_diff(target);
            // Allow < 1000 ns error over a full second (< 1 ppm).
            assert!(
                diff < 1_000,
                "freq {f}: 1s of cycles -> {ns} ns (target {target}, diff {diff})"
            );
        }
    }

    #[test]
    fn ten_seconds_scales_linearly_within_tolerance() {
        // A longer interval must stay accurate (the fixed-point error does not
        // accumulate badly): 10 s of cycles -> ~10e9 ns.
        for &f in &FREQS {
            let (mult, shift) = calc_mult_shift(f);
            let cycles = f.checked_mul(10).expect("10s of cycles fits u64");
            let ns = cycles_to_ns(cycles, mult, shift);
            let target = 10 * NS_PER_SEC;
            let diff = ns.abs_diff(target);
            // < 10_000 ns over 10 s (< 1 ppm).
            assert!(
                diff < 10_000,
                "freq {f}: 10s of cycles -> {ns} ns (target {target}, diff {diff})"
            );
        }
    }

    #[test]
    fn cycles_to_ns_zero_is_zero() {
        let (mult, shift) = calc_mult_shift(24_000_000);
        assert_eq!(cycles_to_ns(0, mult, shift), 0);
    }

    #[test]
    fn cycles_to_ns_no_overflow_at_large_delta() {
        // A counter that has run for ~100 years at 2.5 GHz: ~7.9e18 cycles, near
        // u64::MAX. The u128 intermediate must not overflow; the result must be
        // a sane (large but finite) ns value, not a wrap.
        let (mult, shift) = calc_mult_shift(2_500_000_000);
        let delta = 7_800_000_000_000_000_000u64; // < u64::MAX
        let ns = cycles_to_ns(delta, mult, shift);
        // ns ~= delta / 2.5e9 * 1e9 = delta * 0.4 ~= 3.12e18.
        let expected = (delta as u128 * NS_PER_SEC as u128 / 2_500_000_000u128) as u64;
        assert!(ns.abs_diff(expected) < expected / 1_000_000 + 1_000);
    }

    #[test]
    fn calc_mult_shift_handles_zero_freq() {
        // Degenerate 0 Hz is treated as 1 Hz (no divide-by-zero); mult/shift are
        // well-formed (mult fits u32).
        let (mult, shift) = calc_mult_shift(0);
        assert!(shift <= MAX_SHIFT);
        assert!(mult > 0);
    }

    #[test]
    fn ns_to_timespec_splits_seconds_and_nanos() {
        assert_eq!(ns_to_timespec(0), (0, 0));
        assert_eq!(ns_to_timespec(1), (0, 1));
        assert_eq!(ns_to_timespec(NS_PER_SEC), (1, 0));
        assert_eq!(ns_to_timespec(NS_PER_SEC + 500), (1, 500));
        // 2.5 seconds.
        assert_eq!(ns_to_timespec(2_500_000_000), (2, 500_000_000));
        // tv_nsec stays in [0, 1e9).
        let (_s, nsec) = ns_to_timespec(123_456_789_987);
        assert!((0..NS_PER_SEC as i64).contains(&nsec));
    }

    #[test]
    fn ns_to_timeval_splits_seconds_and_micros() {
        assert_eq!(ns_to_timeval(0), (0, 0));
        // 1500 ns -> 0 s, 1 us (truncates the 500 ns).
        assert_eq!(ns_to_timeval(1_500), (0, 1));
        assert_eq!(ns_to_timeval(NS_PER_SEC), (1, 0));
        // 2.5 s -> 2 s, 500000 us.
        assert_eq!(ns_to_timeval(2_500_000_000), (2, 500_000));
        // tv_usec stays in [0, 1e6).
        let (_s, usec) = ns_to_timeval(123_456_789_987);
        assert!((0..US_PER_SEC as i64).contains(&usec));
    }

    #[test]
    fn wallclock_offset_is_2024_epoch() {
        assert_eq!(WALLCLOCK_OFFSET_SECS, 1_704_067_200);
        assert_eq!(WALLCLOCK_OFFSET_NS, 1_704_067_200 * NS_PER_SEC);
        // Realtime base derived from a boot sample carries the offset.
        let tk = TimekeeperData::from_boot_sample(12345, 24_000_000);
        assert_eq!(tk.real_ns_base, WALLCLOCK_OFFSET_NS);
        assert_eq!(tk.mono_ns_base, 0);
        assert_eq!(tk.counter_at_base, 12345);
    }
}
