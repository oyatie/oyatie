//! EL1 physical generic timer (`CNTP_*_EL0`) for the aarch64 Frame.
//!
//! Frame code. We use the EL1 physical timer counting against the system
//! counter at `CNTFRQ_EL0` Hz. Each tick reprograms a one-shot `TVAL` for the
//! next interval and prints `timer tick <n>`. After [`MAX_TICKS`] ticks the
//! kernel has demonstrated working interrupts, so we cleanly power the machine
//! off via PSCI `SYSTEM_OFF` and the run self-terminates.

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

use aarch64_cpu::registers::{CNTFRQ_EL0, CNTPCT_EL0, CNTP_CTL_EL0, CNTP_TVAL_EL0};
use tock_registers::interfaces::{Readable, Writeable};

use crate::kprintln;

/// Number of timer interrupts to demonstrate before powering off (boot demo
/// only; the user-process scheduler never powers off from a tick).
const MAX_TICKS: u32 = 3;

/// Interval between ticks, in counter cycles (set from `CNTFRQ_EL0` in [`init`]).
static INTERVAL: AtomicU64 = AtomicU64::new(0);
/// Ticks observed so far.
static COUNT: AtomicU32 = AtomicU32::new(0);
/// When set, the timer is a **preemption** source for the user scheduler: each
/// tick just re-arms (the IRQ epilogue does the context switch) and never powers
/// the machine off. Cleared during the standalone boot timer demo.
static PREEMPT_MODE: AtomicBool = AtomicBool::new(false);

/// Read the counter frequency (Hz).
pub fn frequency() -> u64 {
    CNTFRQ_EL0.get()
}

/// Read the current monotonic counter value.
pub fn now() -> u64 {
    CNTPCT_EL0.get()
}

// ===========================================================================
// Timekeeper: the syscall-facing monotonic/realtime clock (P3 slice 3).
//
// The aarch64 generic counter (`CNTPCT_EL0`) is a free-running 64-bit monotonic
// counter at `CNTFRQ_EL0` Hz. At init we sample it ONCE as the base and publish
// `(counter_at_base, mono_ns_base=0)` into a `ksync::SeqLock`; `(mult, shift)`
// derived from the live frequency are stored in `AtomicU32`s (immutable after
// init — read WITHOUT the lock, per spec §1.3 "Option A"). The read path is then
// `mono_ns = mono_ns_base + cycles_to_ns(now()-counter_at_base, MULT, SHIFT)` —
// a single u128 multiply + shift, no divide. REALTIME adds the fixed wall-clock
// epoch offset (`WALLCLOCK_OFFSET_NS`).
//
// Single writer (this `init`, on the boot core), so the SeqLock's single-writer
// contract is trivially satisfied; syscalls only ever READ it.
// ===========================================================================

use ksync::seqlock::SeqLock;
use user_layout::timekeep;

/// The two tick-varying base words `(counter_at_base, mono_ns_base)`, published
/// once at init and read as a consistent snapshot by `clock_gettime`.
static TIMEKEEPER_BASE: SeqLock = SeqLock::new(0, 0);
/// Fixed-point `cycles -> ns` multiplier, written once at init.
static TK_MULT: AtomicU32 = AtomicU32::new(0);
/// Fixed-point `cycles -> ns` right-shift, written once at init.
static TK_SHIFT: AtomicU32 = AtomicU32::new(0);

/// Publish the timekeeper base + scale from the live generic counter. Idempotent
/// in effect (single boot-core caller); call once before dropping to EL0.
pub fn init_timekeeper() {
    let freq = frequency();
    let (mult, shift) = timekeep::calc_mult_shift(freq);
    TK_MULT.store(mult, Ordering::Relaxed);
    TK_SHIFT.store(shift, Ordering::Relaxed);
    // Sample the counter as the base and publish (counter_at_base, mono_ns_base=0).
    // Store the base AFTER mult/shift so any (impossible pre-EL0) reader that sees
    // a nonzero base also sees a valid scale.
    let base = now();
    TIMEKEEPER_BASE.write(base, 0);
}

/// Current monotonic nanoseconds since the timekeeper base.
pub fn mono_ns() -> u64 {
    let (counter_at_base, mono_ns_base) = TIMEKEEPER_BASE.read();
    let mult = TK_MULT.load(Ordering::Relaxed);
    let shift = TK_SHIFT.load(Ordering::Relaxed);
    let delta = now().wrapping_sub(counter_at_base);
    mono_ns_base.wrapping_add(timekeep::cycles_to_ns(delta, mult, shift))
}

/// Current realtime nanoseconds = monotonic + the fixed wall-clock epoch offset.
pub fn real_ns() -> u64 {
    mono_ns().wrapping_add(timekeep::WALLCLOCK_OFFSET_NS)
}

/// Busy-wait until the monotonic counter reaches (at least) `deadline` cycles,
/// i.e. sleep for `deadline - now()` cycles. Used by the `nanosleep` /
/// `clock_nanosleep` / `ppoll`-timeout syscalls to implement a bounded sleep on
/// the generic timer.
///
/// The wait is a `wfe`/poll loop that leaves interrupts unmasked, so the
/// periodic timer PPI keeps firing (and ultimately powers the machine off after
/// `MAX_TICKS`). `wfe` lets the core idle between wakeups instead of spinning
/// hot. The deadline is an absolute counter value the caller computes with the
/// host-tested `deadline_after`, so this routine itself stays trivial.
pub fn sleep_until(deadline: u64) {
    while CNTPCT_EL0.get() < deadline {
        aarch64_cpu::asm::wfe();
    }
}

/// Program the timer for a tick `interval_cycles` from now and enable it.
fn arm(interval_cycles: u64) {
    CNTP_TVAL_EL0.set(interval_cycles);
    CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE::SET + CNTP_CTL_EL0::IMASK::CLEAR);
}

/// Arm THIS AP's CNTP timer as a preemption source (P4·SMP·S4a), reusing the
/// interval the BSP already computed in [`init_preempt`]. The generic timer is
/// per-CPU, so each AP must arm its own to receive preemption ticks + `wfe`
/// wakeups. `PREEMPT_MODE`/`INTERVAL` are global (set once by the BSP); the AP
/// only programs its local `CNTP_TVAL/CTL`.
///
/// # Safety
/// Call once per AP, after the BSP armed preemption (`INTERVAL` published) and
/// this AP's GIC + timer PPI are up. Touches only this CPU's timer registers.
pub unsafe fn arm_preempt_ap() {
    let interval = INTERVAL.load(Ordering::Relaxed);
    arm(interval);
}

/// Initialize the physical timer to fire every `period_ms` milliseconds.
///
/// # Safety
/// Call once on the boot core after the GIC is up and the timer PPI is enabled.
pub unsafe fn init(period_ms: u64) {
    let freq = frequency();
    let interval = freq.saturating_mul(period_ms) / 1000;
    INTERVAL.store(interval, Ordering::Relaxed);
    arm(interval);
}

/// Start the timer as a **scheduler preemption** source: fire every `period_ms`
/// milliseconds, re-arm on every tick, and never power off (the IRQ epilogue
/// handles the actual context switch). Used by the user-process workload.
///
/// # Safety
/// Call once, on the boot core, after the GIC + timer PPI are up, before
/// dropping to EL0.
pub unsafe fn init_preempt(period_ms: u64) {
    let freq = frequency();
    let interval = freq.saturating_mul(period_ms) / 1000;
    INTERVAL.store(interval, Ordering::Relaxed);
    // Release: publishes INTERVAL (above) to APs that Acquire-read PREEMPT_MODE
    // via `preempt_active()` before arming their own timer (P4·SMP·S4a).
    PREEMPT_MODE.store(true, Ordering::Release);
    arm(interval);
}

/// True once the BSP has switched the timer into scheduler-preemption mode
/// (`init_preempt`), which also publishes `INTERVAL`. APs spin on this before
/// arming their own timer + entering the scheduler (P4·SMP·S4a). `Acquire` pairs
/// with the `Release`-ordered process-model handoff.
pub fn preempt_active() -> bool {
    PREEMPT_MODE.load(Ordering::Acquire)
}

/// Called from the IRQ dispatcher when the timer PPI fires.
pub fn on_tick() {
    // Preemption mode (user scheduler): just re-arm. The IRQ epilogue in
    // `exceptions.rs` performs the context switch; we neither print nor power
    // off, so the workload runs to completion across many ticks.
    if PREEMPT_MODE.load(Ordering::Relaxed) {
        let interval = INTERVAL.load(Ordering::Relaxed);
        arm(interval);
        return;
    }

    let n = COUNT.fetch_add(1, Ordering::Relaxed) + 1;
    kprintln!("timer tick {}", n);

    if n >= MAX_TICKS {
        // Disable further timer interrupts and shut the machine down.
        CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE::CLEAR + CNTP_CTL_EL0::IMASK::SET);
        kprintln!("kernel: OK");
        crate::arch::power_off();
    }

    // Re-arm for the next interval.
    let interval = INTERVAL.load(Ordering::Relaxed);
    arm(interval);
}
