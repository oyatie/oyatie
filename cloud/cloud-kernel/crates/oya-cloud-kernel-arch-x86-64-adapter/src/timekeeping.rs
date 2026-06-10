//! x86_64 **timekeeping**: the syscall-facing monotonic/realtime clock backed by
//! the TSC (`rdtsc`), calibrated against the 8254 PIT at boot.
//!
//! This is the x86_64 counterpart of aarch64's generic-counter timekeeper
//! (`arch-aarch64/src/timer.rs`). Where aarch64 has a self-describing counter
//! (`CNTFRQ_EL0`), x86_64 under `-cpu qemu64` exposes **no** TSC frequency via
//! CPUID (no leaf 0x15/0x16), so we **calibrate** the TSC against the PIT's
//! known 1.193182 MHz input at boot and derive the same `mult`/`shift`
//! fixed-point scale the pure `user_layout::timekeep` math defines.
//!
//! ## The only `unsafe` here
//! `now_counter()` issues `_rdtsc` (a pure register read), and calibration does
//! raw PIT port I/O. Both are documented at their sites; everything else
//! (`mult`/`shift`, `cycles_to_ns`) is the shared zero-unsafe pure math. The
//! tick-varying base `(counter_at_base, mono_ns_base)` lives in a
//! `ksync::SeqLock` (single boot-core writer, lock-free readers); `mult`/`shift`
//! are immutable `AtomicU32`s read without the lock (spec §1.3 "Option A").
//!
//! ## Calibration method (IRQ-independent)
//! We use **PIT channel 2** (the legacy speaker timer) in one-shot mode, gated by
//! port 0x61 bit 0. Channel 2 is independent of channel 0 (the scheduler tick),
//! so calibration neither needs nor disturbs IRQ0. We load a known count, read
//! `rdtsc`, spin until the channel-2 counter has run for the loaded interval
//! (detected via the 0x61 bit-5 output), read `rdtsc` again, and divide the TSC
//! delta by the elapsed PIT seconds. Runs with IRQs masked (we are called before
//! the scheduler is live). A degenerate/zero calibration falls back to a nonzero
//! floor so time still advances and no divide-by-zero occurs.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use x86_64::instructions::port::Port;

use ksync::seqlock::SeqLock;
use user_layout::timekeep;

/// PIT input clock: 1.193182 MHz (matches [`crate::timer::PIT_FREQUENCY`]).
const PIT_FREQUENCY: u64 = crate::timer::PIT_FREQUENCY as u64;

/// PIT channel-2 data port.
const PIT_CH2_DATA: u16 = 0x42;
/// PIT mode/command port.
const PIT_MODE_COMMAND: u16 = 0x43;
/// NMI status / speaker control port; bit 0 gates channel 2, bit 5 is its output.
const PORT_0x61: u16 = 0x61;

/// Nonzero TSC-frequency floor if calibration yields 0 (degenerate hardware /
/// emulator). 1 GHz is a sane default so time advances and `mult`/`shift` are
/// well-formed; better an approximate clock than a divide-by-zero. Documented.
const TSC_HZ_FLOOR: u64 = 1_000_000_000;

/// The two tick-varying base words `(counter_at_base, mono_ns_base)`, published
/// once at init and read as a consistent snapshot by `clock_gettime`.
static TIMEKEEPER_BASE: SeqLock = SeqLock::new(0, 0);
/// Fixed-point `cycles -> ns` multiplier, written once at init.
static TK_MULT: AtomicU32 = AtomicU32::new(0);
/// Fixed-point `cycles -> ns` right-shift, written once at init.
static TK_SHIFT: AtomicU32 = AtomicU32::new(0);
/// Calibrated TSC frequency in Hz (also published once at init). Read by the
/// nanosleep path to convert a `timespec` into a TSC-cycle deadline.
static TSC_HZ: AtomicU64 = AtomicU64::new(0);

/// Read the raw timestamp counter.
#[inline]
pub fn now_counter() -> u64 {
    // SAFETY: `_rdtsc` reads the 64-bit timestamp counter; a pure read with no
    // memory or device side effects, valid in long mode at any privilege. This
    // is the single counter-read unsafe site for x86_64 timekeeping (mirrors the
    // documented `hal_caps.rs` `_rdtsc` site).
    unsafe { core::arch::x86_64::_rdtsc() }
}

/// The calibrated TSC frequency in Hz (>= 1 after init).
#[inline]
pub fn tsc_hz() -> u64 {
    TSC_HZ.load(Ordering::Relaxed).max(1)
}

/// Calibrate the TSC against PIT channel 2 over `~target_ms` milliseconds and
/// return the measured frequency in Hz (floored to a nonzero value).
///
/// # Safety
/// Performs raw PIT (ch2) + port 0x61 I/O. Must run on the boot core with IRQs
/// masked, before the scheduler is live, and must not race another PIT
/// programmer. Channel 2 is independent of channel 0 (the scheduler tick), so it
/// is the safe channel to borrow for calibration.
unsafe fn calibrate_tsc_hz(target_ms: u64) -> u64 {
    // PIT count for the target interval (clamped to the 16-bit reload). At 1.193
    // MHz, ~50 ms == ~59659 counts, which fits 16 bits.
    let mut count = (PIT_FREQUENCY * target_ms / 1000) as u32;
    if count == 0 {
        count = 1;
    }
    if count > 0xFFFF {
        count = 0xFFFF;
    }
    let count = count as u16;

    let mut ch2: Port<u8> = Port::new(PIT_CH2_DATA);
    let mut cmd: Port<u8> = Port::new(PIT_MODE_COMMAND);
    let mut p61: Port<u8> = Port::new(PORT_0x61);

    // SAFETY: standard PIT ch2 calibration sequence (raw port I/O is the device's
    // sole contract; no memory effects):
    //   1. Set port 0x61: clear the speaker output (bit1=0), enable the ch2 gate
    //      (bit0=1) so the counter will run once we load it.
    //   2. Program ch2 (command 0xB0 = channel 2, lobyte/hibyte, mode 0
    //      "interrupt on terminal count", binary), then write the 16-bit count.
    //      Mode 0 drives the OUT line (0x61 bit5) HIGH when the count reaches 0.
    //   3. Sample rdtsc, spin until 0x61 bit5 goes high (count elapsed), sample
    //      rdtsc again.
    let (start, end) = unsafe {
        let p = p61.read();
        // bit1 (speaker data) = 0, bit0 (gate) = 1.
        p61.write((p & !0x02) | 0x01);

        cmd.write(0xB0u8);
        ch2.write((count & 0xFF) as u8);
        ch2.write((count >> 8) as u8);

        // Re-arm the gate with a 0->1 edge so mode-0 counting starts from `count`.
        let p = p61.read() & !0x01;
        p61.write(p);
        p61.write(p | 0x01);

        let start = now_counter();
        // Spin until ch2 OUT (0x61 bit5) is high == terminal count reached.
        let mut guard: u64 = 0;
        while (p61.read() & 0x20) == 0 {
            guard += 1;
            // Defensive bound: if OUT never asserts (broken emulator), give up so
            // we fall through to the floor rather than hang the boot.
            if guard > 100_000_000 {
                break;
            }
            core::hint::spin_loop();
        }
        let end = now_counter();
        (start, end)
    };

    let tsc_delta = end.wrapping_sub(start);
    // tsc_hz = tsc_delta / (count / PIT_FREQUENCY) = tsc_delta * PIT_FREQUENCY / count.
    let hz = (tsc_delta as u128 * PIT_FREQUENCY as u128 / count as u128) as u64;
    if hz == 0 {
        TSC_HZ_FLOOR
    } else {
        hz
    }
}

/// Calibrate the TSC and publish the timekeeper base + scale. Idempotent in
/// effect (single boot-core caller).
///
/// # Safety
/// Call once on the boot core with IRQs masked, before dropping to ring 3 and
/// before the scheduler PIT (channel 0) is the live tick source. Performs raw PIT
/// I/O via [`calibrate_tsc_hz`].
pub unsafe fn init_timekeeper() {
    // SAFETY: boot core, IRQs masked, single PIT programmer (see contract above).
    // `calibrate_tsc_hz` already floors a zero measurement to TSC_HZ_FLOOR; the
    // extra guard keeps `hz >= 1` even against a pathological return.
    let hz = unsafe { calibrate_tsc_hz(50) }.max(1);
    let (mult, shift) = timekeep::calc_mult_shift(hz);
    TSC_HZ.store(hz, Ordering::Relaxed);
    TK_MULT.store(mult, Ordering::Relaxed);
    TK_SHIFT.store(shift, Ordering::Relaxed);
    // Publish the base AFTER mult/shift/hz so a reader that sees a nonzero base
    // also sees a valid scale.
    let base = now_counter();
    TIMEKEEPER_BASE.write(base, 0);
}

/// Current monotonic nanoseconds since the timekeeper base.
pub fn mono_ns() -> u64 {
    let (counter_at_base, mono_ns_base) = TIMEKEEPER_BASE.read();
    let mult = TK_MULT.load(Ordering::Relaxed);
    let shift = TK_SHIFT.load(Ordering::Relaxed);
    let delta = now_counter().wrapping_sub(counter_at_base);
    mono_ns_base.wrapping_add(timekeep::cycles_to_ns(delta, mult, shift))
}

/// Current realtime nanoseconds = monotonic + the fixed wall-clock epoch offset.
pub fn real_ns() -> u64 {
    mono_ns().wrapping_add(timekeep::WALLCLOCK_OFFSET_NS)
}

/// Busy-wait (ring-0 spin) until the TSC reaches `deadline_tsc`. Used by the
/// nanosleep path: a deadline computed from `tsc_hz()` + the host-tested
/// `deadline_after`. A ring-0 busy-wait (vs blocking) avoids re-entering the
/// scheduler from a syscall; the periodic PIT keeps preempting other processes.
pub fn sleep_until(deadline_tsc: u64) {
    while now_counter() < deadline_tsc {
        core::hint::spin_loop();
    }
}
