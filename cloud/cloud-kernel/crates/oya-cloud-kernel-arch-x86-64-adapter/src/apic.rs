//! x2APIC local-APIC: the modern per-CPU interrupt chip + LVT timer, the P4
//! replacement for the legacy 8259 PIC + 8254 PIT preemption tick.
//!
//! ## The 3-tier timer model (gate x2APIC SEPARATELY from the timer)
//!
//! The dev host is Apple-Silicon QEMU under **TCG**, which emulates `+x2apic`
//! (including the local-APIC LVT timer) but **cannot** emulate the
//! TSC-deadline timer mode (`-cpu …,+tsc-deadline` prints
//! `TCG doesn't support requested feature: …tsc-deadline [bit 24]` and drops
//! it). So we gate x2APIC separately from the timer into three tiers, picked
//! ONCE at boot from the immutable [`crate::hal_caps::DetectedCaps`] into
//! [`active_tier`]; all three drive the SAME `__kuberos_timer_entry -> preempt`
//! hook (`user.rs`), only the ack/re-arm differs:
//!
//! | Tier | CpuCaps gate | Chip + timer | EOI / re-arm |
//! |------|--------------|--------------|--------------|
//! | 1 | `x2apic && tsc_deadline` | x2APIC + LVT **TSC-deadline** mode | x2APIC EOI MSR `0x80B` + re-arm `IA32_TSC_DEADLINE` `0x6E0` from `tsc_hz` |
//! | 2 | `x2apic && !tsc_deadline` | x2APIC + LVT **periodic** mode (PIT-calibrated initial-count, auto-reload) | x2APIC EOI MSR `0x80B` (no per-tick re-arm) |
//! | 3 | `!x2apic` | 8259 PIC + 8254 PIT (unchanged) | `out 0x20, al` |
//!
//! Tier 2 RETIRES the PIT and is FULLY verifiable on the dev host under
//! `-cpu qemu64,+x2apic`. Tier 1 is implemented + reviewed but its empirical
//! verification is **PENDING an x86 KVM CI runner** (TCG cannot exercise it).
//! Tier 3 is byte-identical to the pre-P4 path.
//!
//! ## This is Frame code
//!
//! Every x2APIC register access is a raw MSR read/write (x2APIC mode maps the
//! APIC registers to MSRs `0x800..0x83F`, no MMIO) and every PIT-calibration
//! read is raw port I/O — all `unsafe`, all documented at its site. The safe
//! kernel never references this module; the boot dispatch ([`init_for_caps`])
//! and the per-tick ack ([`ack_timer`]) are the only entry points, both called
//! from the Frame's `crate::user`.

use core::sync::atomic::{AtomicU64, AtomicU8, Ordering};

use x86_64::instructions::port::Port;
use x86_64::registers::model_specific::Msr;

use crate::hal_caps::DetectedCaps;

// ---------------------------------------------------------------------------
// MSR register block (x2APIC mode: APIC registers are MSRs 0x800..0x83F)
// ---------------------------------------------------------------------------

/// `IA32_APIC_BASE` — bit 11 = APIC global enable, bit 10 = x2APIC enable.
const IA32_APIC_BASE: u32 = 0x1B;
/// x2APIC **TPR** (task priority): write 0 to accept all interrupt priorities.
const X2APIC_TPR: u32 = 0x808;
/// x2APIC **EOI**: write 0 to signal end-of-interrupt.
const X2APIC_EOI: u32 = 0x80B;
/// x2APIC **SVR** (spurious-interrupt vector register): bit 8 software-enables
/// the APIC; bits[7:0] are the spurious vector.
const X2APIC_SVR: u32 = 0x80F;
/// x2APIC **LVT timer** entry: mode bits[18:17], mask bit 16, vector bits[7:0].
const X2APIC_LVT_TIMER: u32 = 0x832;
/// x2APIC **divide-configuration** register for the LVT timer count rate.
const X2APIC_DIV_CONF: u32 = 0x83E;
/// x2APIC **initial-count** register (periodic/one-shot reload value).
const X2APIC_INIT_COUNT: u32 = 0x838;
/// x2APIC **current-count** register (counts down from the initial count).
const X2APIC_CUR_COUNT: u32 = 0x839;
/// `IA32_TSC_DEADLINE` — absolute TSC deadline for the one-shot Tier-1 timer.
const IA32_TSC_DEADLINE: u32 = 0x6E0;
/// x2APIC **ICR** (interrupt-command register). In x2APIC mode the 64-bit ICR is
/// a SINGLE MSR (`0x830`): the destination APIC ID is the high 32 bits and the
/// command (delivery mode / level / vector) is the low 32 bits — no ICR-high/
/// ICR-low split as in legacy xAPIC. Used for INIT-SIPI-SIPI in
/// [`send_init_sipi`] (P4·SMP·S3). No ICR constant existed before this slice.
const X2APIC_ICR: u32 = 0x830;

/// `IA32_APIC_BASE` bit 11 — APIC global enable.
const APIC_BASE_GLOBAL_ENABLE: u64 = 1 << 11;
/// `IA32_APIC_BASE` bit 10 — x2APIC mode enable (requires bit 11 also set).
const APIC_BASE_X2APIC_ENABLE: u64 = 1 << 10;

/// SVR bit 8 — APIC software-enable.
const SVR_SOFTWARE_ENABLE: u32 = 1 << 8;

/// LVT-timer mode bit 17 — periodic mode (auto-reload from the initial count).
const LVT_TIMER_PERIODIC: u32 = 1 << 17;
/// LVT-timer mode bit 18 — TSC-deadline mode (one-shot off `IA32_TSC_DEADLINE`).
const LVT_TIMER_TSC_DEADLINE: u32 = 1 << 18;

/// Divide config = divide-by-1 (encoding `0b1011`: bits[3,1,0]). The APIC timer
/// then counts at the full bus/crystal rate; we size the initial count to the
/// tick period at boot via calibration ([`calibrate_periodic_count`]).
const DIV_CONF_BY_1: u64 = 0b1011;

/// The CPU vector the LVT timer raises — **reused** from the PIT/IRQ0 vector
/// (`crate::interrupts::PIC_1_OFFSET` = 0x20) so the IDT gate, the gate-patch
/// path, and the `__kuberos_timer_entry` trampoline are unchanged (zero IDT
/// churn). Safe because the 8259 is fully masked once x2APIC engages, so no
/// legacy IRQ0 can also target 0x20.
const TIMER_VECTOR: u32 = crate::interrupts::PIC_1_OFFSET as u32;

/// The spurious-interrupt vector. Needs a no-op IDT gate (installed in
/// `crate::interrupts`); the APIC does not require an EOI for it.
pub const SPURIOUS_VECTOR: u8 = 0xFF;

/// Scheduler tick frequency — must equal the PIT's `TIMER_HZ` (`arch.rs`) so the
/// preemption period (and thus the observable interleaving) is unchanged across
/// all three tiers.
const TIMER_HZ: u64 = 100;

// ---------------------------------------------------------------------------
// Boot-immutable tier selection (no per-tick CPUID)
// ---------------------------------------------------------------------------

/// Which timer tier is live. Chosen ONCE at boot from the immutable CpuCaps and
/// stored here; the per-tick ack reads it `Relaxed` (a single predictable load,
/// no MSR read, no CPUID — see [`ack_timer`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TimerTier {
    /// 8259 PIC + 8254 PIT, EOI via `out 0x20, al`. Byte-identical to pre-P4.
    PicPit = 0,
    /// x2APIC + LVT periodic timer, EOI via x2APIC EOI MSR. PIT retired.
    X2apicPeriodic = 1,
    /// x2APIC + LVT TSC-deadline timer, EOI + re-arm via MSRs. KVM-only.
    X2apicTscDeadline = 2,
}

impl TimerTier {
    /// The non-golden-shape boot announce line for this tier.
    fn announce(self) -> &'static str {
        match self {
            TimerTier::PicPit => "irq: 8259 PIC + PIT (tier3)",
            TimerTier::X2apicPeriodic => "irq: x2APIC + LVT-periodic (tier2)",
            TimerTier::X2apicTscDeadline => "irq: x2APIC + TSC-deadline (tier1)",
        }
    }
}

/// The live tier, as a `u8` (the `TimerTier` discriminant). Written once at boot
/// in [`init_for_caps`], read `Relaxed` per tick in [`ack_timer`]. Defaults to
/// `PicPit` (0) so any path that never calls `init_for_caps` keeps the legacy
/// ack — the conservative fallback.
static ACTIVE_TIER: AtomicU8 = AtomicU8::new(TimerTier::PicPit as u8);

/// TSC cycles per scheduler tick (`tsc_hz / TIMER_HZ`), computed once when the
/// Tier-1 deadline path arms. Read per tick in [`ack_timer`] to re-arm the next
/// deadline. Zero until armed (Tier 1 only).
static TSC_PER_TICK: AtomicU64 = AtomicU64::new(0);

/// APIC-timer initial-count per scheduler tick (Tier 2 periodic), calibrated once
/// by the BSP in [`arm_periodic`] and reused by each AP in [`arm_periodic_ap`] so
/// the APs do not re-run the shared-PIT calibration (P4·SMP·S4a). Zero until the
/// BSP arms the periodic timer.
static PERIODIC_COUNT: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

/// Read the live tier.
#[inline]
pub fn active_tier() -> TimerTier {
    match ACTIVE_TIER.load(Ordering::Relaxed) {
        1 => TimerTier::X2apicPeriodic,
        2 => TimerTier::X2apicTscDeadline,
        _ => TimerTier::PicPit,
    }
}

// ---------------------------------------------------------------------------
// Low-level MSR helpers (the only x2APIC register access path)
// ---------------------------------------------------------------------------

/// Write `value` to MSR `reg`.
///
/// # Safety
/// `reg` must be a valid, writable MSR for the current mode and `value` a legal
/// payload. Used only for the x2APIC register block + `IA32_TSC_DEADLINE` here,
/// after x2APIC mode is enabled (mirrors the established `Msr::write` precedent
/// in `user.rs`/`process.rs`).
#[inline]
unsafe fn wrmsr(reg: u32, value: u64) {
    // SAFETY (delegated to caller): a single architected MSR write; no memory
    // effects beyond the named APIC/timekeeping register.
    unsafe { Msr::new(reg).write(value) }
}

/// Read MSR `reg`.
///
/// # Safety
/// `reg` must be a valid, readable MSR for the current mode.
#[inline]
unsafe fn rdmsr(reg: u32) -> u64 {
    // SAFETY (delegated to caller): a pure architected MSR read; no side effects.
    unsafe { Msr::new(reg).read() }
}

// ---------------------------------------------------------------------------
// Boot-time enable + tier selection
// ---------------------------------------------------------------------------

/// Master-PIC mask port (8259 #1 data register).
const PIC1_DATA: u16 = 0x21;
/// Slave-PIC mask port (8259 #2 data register).
const PIC2_DATA: u16 = 0xA1;

/// Select + initialize the timer tier from the immutable boot CpuCaps. Called
/// once on the boot core with IRQs masked, AFTER `init_timekeeper` (so
/// `tsc_hz()` is valid) and BEFORE the preempt handler is installed. Returns the
/// chosen [`TimerTier`] (the caller announces it).
///
/// - Tier 3 (`!x2apic`): does nothing — the legacy PIC/PIT path is left intact
///   and the PIT is armed by the existing `arch::run_user` flow.
/// - Tier 1/2 (`x2apic`): masks the 8259 fully, enables x2APIC + the LVT timer,
///   and arms the timer in the tier-appropriate mode (TSC-deadline one-shot for
///   Tier 1; PIT-calibrated periodic for Tier 2). The PIT is left idle; the 8259
///   is masked so even a stray PIT pulse cannot deliver.
///
/// # Safety
/// Call once on the boot core, IRQs masked, after `init_timekeeper` and before
/// dropping to ring 3. Reconfigures the interrupt chip + timer hardware.
pub unsafe fn init_for_caps(caps: DetectedCaps) -> TimerTier {
    if !caps.x2apic {
        // Tier 3: leave the entire legacy path untouched (byte-identical).
        ACTIVE_TIER.store(TimerTier::PicPit as u8, Ordering::Relaxed);
        return TimerTier::PicPit;
    }

    // Tiers 1 & 2 share the x2APIC enable + LVT setup; only the timer mode +
    // re-arm differ. SAFETY: boot core, IRQs masked, single APIC/PIC programmer.
    unsafe {
        mask_8259_fully();
        enable_x2apic();
    }

    let tier = if caps.tsc_deadline {
        // SAFETY: x2APIC is enabled; arm the TSC-deadline one-shot (Tier 1).
        unsafe { arm_tsc_deadline_first() };
        TimerTier::X2apicTscDeadline
    } else {
        // SAFETY: x2APIC is enabled; calibrate + arm the periodic LVT timer.
        unsafe { arm_periodic() };
        TimerTier::X2apicPeriodic
    };

    ACTIVE_TIER.store(tier as u8, Ordering::Relaxed);
    tier
}

/// Mask every line on both 8259 PICs (master + slave IMR = 0xFF) before
/// switching to x2APIC, so a stray ISA IRQ cannot reach a vector. The PIC stays
/// remapped (from `init_pic`) — we only mask — so vectors 0x20..0x2F remain
/// safely owned and out of the CPU-exception range.
///
/// # Safety
/// Raw port I/O to the 8259 data registers; valid post-`init_pic`, boot core.
unsafe fn mask_8259_fully() {
    let mut m1: Port<u8> = Port::new(PIC1_DATA);
    let mut m2: Port<u8> = Port::new(PIC2_DATA);
    // SAFETY: writing 0xFF to both IMRs masks all 16 legacy lines; the documented
    // PIC mask protocol, identical to `Pic8259IrqChip::cancel`. No memory effects.
    unsafe {
        m1.write(0xFF);
        m2.write(0xFF);
    }
}

/// Enable x2APIC mode + software-enable the local APIC. Leaves the LVT timer
/// masked (the arming functions program + unmask it). Other LVT entries
/// (LINT0/1, error, thermal, perf) are left at reset — single-CPU, no I/O-APIC
/// routing in this slice.
///
/// # Safety
/// Call once on the boot core with IRQs masked, only when CPUID reported x2APIC.
unsafe fn enable_x2apic() {
    // SAFETY: the full enable sequence on a part that reports x2APIC. Each MSR is
    // the architected x2APIC register; the ordering (global+x2APIC enable, then
    // SVR software-enable, then TPR) is the documented bring-up.
    unsafe {
        // 1. Global + x2APIC enable in IA32_APIC_BASE (bit 10 requires bit 11).
        let base = rdmsr(IA32_APIC_BASE);
        wrmsr(
            IA32_APIC_BASE,
            base | APIC_BASE_GLOBAL_ENABLE | APIC_BASE_X2APIC_ENABLE,
        );

        // 2. SVR: software-enable the APIC + point the spurious vector at the
        //    no-op IDT gate (0xFF).
        wrmsr(
            X2APIC_SVR,
            (SVR_SOFTWARE_ENABLE | SPURIOUS_VECTOR as u32) as u64,
        );

        // 3. TPR = 0: accept all interrupt priorities.
        wrmsr(X2APIC_TPR, 0);

        // 4. Mask the LVT timer for now; the arm functions program its mode +
        //    vector and unmask it.
        wrmsr(X2APIC_LVT_TIMER, (1 << 16) | TIMER_VECTOR as u64);
    }
}

// ---------------------------------------------------------------------------
// Tier 1: TSC-deadline one-shot (KVM-only; review-only on this TCG host)
// ---------------------------------------------------------------------------

/// Arm the first TSC-deadline tick (Tier 1). Puts the LVT timer in TSC-deadline
/// mode, fences so the mode change is globally visible BEFORE the first deadline
/// write (the documented TSC-deadline ordering hazard, Intel SDM Vol.3
/// §10.5.4.1), computes `tsc_per_tick` from the calibrated `tsc_hz()`, and writes
/// the first absolute deadline. Subsequent ticks re-arm in [`ack_timer`].
///
/// # Safety
/// Call once on the boot core after `enable_x2apic`, IRQs masked.
unsafe fn arm_tsc_deadline_first() {
    let tsc_per_tick = (crate::timekeeping::tsc_hz() / TIMER_HZ).max(1);
    TSC_PER_TICK.store(tsc_per_tick, Ordering::Relaxed);

    // SAFETY: x2APIC is enabled; program TSC-deadline mode on the LVT timer at
    // vector 0x20 (unmasked), serialize, then arm the first absolute deadline.
    unsafe {
        wrmsr(
            X2APIC_LVT_TIMER,
            (LVT_TIMER_TSC_DEADLINE | TIMER_VECTOR) as u64,
        );
        // Serializing barrier between the LVT mode write and the deadline write,
        // so the mode change is globally visible before the deadline is armed
        // (else the deadline could be interpreted in the wrong mode).
        core::arch::asm!("mfence; lfence", options(nostack, preserves_flags));
        let deadline = crate::timekeeping::now_counter().wrapping_add(tsc_per_tick);
        wrmsr(IA32_TSC_DEADLINE, deadline);
    }
}

// ---------------------------------------------------------------------------
// Tier 2: LVT periodic timer (PIT-calibrated; fully verifiable under TCG)
// ---------------------------------------------------------------------------

/// Calibrate the APIC-timer count rate against PIT channel 2 and return the
/// initial-count value for one scheduler tick (`apic_count / TIMER_HZ`).
///
/// Classic LAPIC calibration: program the APIC timer to count down from a large
/// initial count at divide-by-1, gate PIT channel 2 for a known interval, read
/// how far the APIC timer counted in that interval, and scale to the APIC count
/// rate (Hz). Reuses the exact PIT-channel-2 gating the TSC calibration uses
/// (channel 2 is independent of channel 0, the scheduler tick).
///
/// # Safety
/// Raw PIT (ch2) + port 0x61 I/O + APIC MSR access. Boot core, IRQs masked,
/// after `enable_x2apic`, single PIT programmer (we own it here).
unsafe fn calibrate_periodic_count() -> u32 {
    // PIT channel-2 ports (mirror `timekeeping::calibrate_tsc_hz`).
    const PIT_CH2_DATA: u16 = 0x42;
    const PIT_MODE_COMMAND: u16 = 0x43;
    const PORT_0X61: u16 = 0x61;
    /// PIT input clock: 1.193182 MHz.
    const PIT_FREQUENCY: u64 = 1_193_182;
    /// Calibration window ~10 ms (fits the 16-bit PIT reload: ~11932 counts).
    const CALIB_MS: u64 = 10;

    let pit_count = ((PIT_FREQUENCY * CALIB_MS / 1000).clamp(1, 0xFFFF)) as u16;

    let mut ch2: Port<u8> = Port::new(PIT_CH2_DATA);
    let mut cmd: Port<u8> = Port::new(PIT_MODE_COMMAND);
    let mut p61: Port<u8> = Port::new(PORT_0X61);

    // SAFETY: standard PIT ch2 gating + APIC initial-count load/read. The APIC
    // timer counts down from 0xFFFF_FFFF at divide-by-1 while the PIT runs the
    // known interval; the consumed APIC count over that interval gives the rate.
    let apic_ticks = unsafe {
        // Divide-by-1 so the APIC count maps directly to the bus/crystal rate.
        wrmsr(X2APIC_DIV_CONF, DIV_CONF_BY_1);

        // Gate ch2: speaker data off (bit1=0), gate on (bit0=1).
        let p = p61.read();
        p61.write((p & !0x02) | 0x01);

        // Program ch2: command 0xB0 = channel 2, lobyte/hibyte, mode 0
        // (interrupt on terminal count), binary; then the 16-bit count.
        cmd.write(0xB0u8);
        ch2.write((pit_count & 0xFF) as u8);
        ch2.write((pit_count >> 8) as u8);

        // Re-arm the gate with a 0->1 edge so mode-0 counting starts from count.
        let p = p61.read() & !0x01;
        p61.write(p);
        p61.write(p | 0x01);

        // Start the APIC timer at max initial count (one-shot down-count) the
        // instant the PIT window opens.
        wrmsr(X2APIC_INIT_COUNT, 0xFFFF_FFFF);

        // Spin until ch2 OUT (0x61 bit5) is high == terminal count reached.
        let mut guard: u64 = 0;
        while (p61.read() & 0x20) == 0 {
            guard += 1;
            if guard > 100_000_000 {
                break;
            }
            core::hint::spin_loop();
        }

        // How far the APIC timer counted down during the PIT interval.
        let remaining = rdmsr(X2APIC_CUR_COUNT) as u32;
        // Stop the APIC timer (initial count 0 halts it).
        wrmsr(X2APIC_INIT_COUNT, 0);
        0xFFFF_FFFFu32.wrapping_sub(remaining)
    };

    // apic_hz = apic_ticks / (pit_count / PIT_FREQUENCY)
    //         = apic_ticks * PIT_FREQUENCY / pit_count
    let apic_hz = (apic_ticks as u128 * PIT_FREQUENCY as u128 / pit_count.max(1) as u128) as u64;
    // initial count for one scheduler tick; floor to 1 so the timer always fires.
    let per_tick = (apic_hz / TIMER_HZ).max(1);
    per_tick.min(0xFFFF_FFFF) as u32
}

/// Arm the LVT timer in periodic mode (Tier 2). Calibrates the per-tick
/// initial count against the PIT, sets divide-by-1, programs the LVT timer
/// periodic at vector 0x20, and loads the initial count (which auto-reloads — no
/// per-tick re-arm). EOI is still required each tick ([`ack_timer`]).
///
/// # Safety
/// Call once on the boot core after `enable_x2apic`, IRQs masked.
unsafe fn arm_periodic() {
    // SAFETY: calibrate (raw PIT/APIC I/O) then program the periodic LVT timer.
    let init_count = unsafe { calibrate_periodic_count() };
    // P4·SMP·S4a: cache the calibrated per-tick count so APs can arm their OWN
    // periodic timer (`arm_periodic_ap`) WITHOUT re-running the PIT-channel-2
    // calibration — the PIT is a single shared device and concurrent AP
    // calibrations would corrupt each other. The APIC bus rate is the same on
    // every core, so the BSP's count is correct for the APs.
    PERIODIC_COUNT.store(init_count, Ordering::Relaxed);
    unsafe {
        wrmsr(X2APIC_DIV_CONF, DIV_CONF_BY_1);
        // Periodic mode (bit 17), unmasked, vector 0x20.
        wrmsr(
            X2APIC_LVT_TIMER,
            (LVT_TIMER_PERIODIC | TIMER_VECTOR) as u64,
        );
        // Loading a nonzero initial count starts the periodic down-count; it
        // auto-reloads on reaching 0, raising vector 0x20 each period.
        wrmsr(X2APIC_INIT_COUNT, init_count as u64);
    }
}

/// Arm THIS AP's local periodic LVT timer (P4·SMP·S4a) using the per-tick count
/// the BSP already calibrated in [`arm_periodic`]. Each AP must arm its OWN APIC
/// timer to receive preemption ticks (the LVT timer is per-CPU). Reuses the
/// cached count instead of re-calibrating against the shared PIT. No-op unless
/// the live tier is the periodic LVT (Tier 2) — under TSC-deadline (Tier 1) the
/// AP would arm a deadline instead, but the SMP test legs run Tier 2 (no
/// `+tsc-deadline`), and a missing AP timer is handled by the AP idle loop's
/// re-check anyway.
///
/// # Safety
/// Call once per AP, after `enable_x2apic_ap`, IRQs masked, with the BSP having
/// already cached the periodic count. Touches only this CPU's APIC MSRs.
pub unsafe fn arm_periodic_ap() {
    if active_tier() != TimerTier::X2apicPeriodic {
        return;
    }
    let init_count = PERIODIC_COUNT.load(Ordering::Relaxed).max(1);
    // SAFETY: x2APIC enabled on this AP; program its periodic LVT timer at the
    // shared scheduler vector + cached per-tick count.
    unsafe {
        wrmsr(X2APIC_DIV_CONF, DIV_CONF_BY_1);
        wrmsr(
            X2APIC_LVT_TIMER,
            (LVT_TIMER_PERIODIC | TIMER_VECTOR) as u64,
        );
        wrmsr(X2APIC_INIT_COUNT, init_count as u64);
    }
}

// ---------------------------------------------------------------------------
// Per-tick ack (the ONE caps-gated branch; replaces the inline `out 0x20, al`)
// ---------------------------------------------------------------------------

/// End-of-interrupt + (Tier-1) re-arm for the just-handled timer tick. Called at
/// the END of the `preempt` hook, replacing the inline `out 0x20, al` the timer
/// trampoline used to emit. A single boot-immutable branch on [`active_tier`]:
///
/// - **Tier 1** (TSC-deadline): re-arm `IA32_TSC_DEADLINE` to `rdtsc + per_tick`
///   then x2APIC EOI. (Re-arm before EOI so no tick is lost.)
/// - **Tier 2** (periodic): x2APIC EOI only — the periodic timer auto-reloads.
/// - **Tier 3** (PIT): `out 0x20, al` — the 8259 master EOI, byte-identical to
///   the inline asm it replaces.
///
/// # Safety
/// Call exactly once per timer tick from the IRQ context (`preempt` tail), in
/// ring 0. Issues the architected EOI for the active chip + (Tier 1) a TSC
/// deadline re-arm.
pub unsafe fn ack_timer() {
    match active_tier() {
        TimerTier::X2apicTscDeadline => {
            // SAFETY: x2APIC enabled; re-arm the next absolute deadline from the
            // calibrated per-tick cycles, then signal EOI for the timer vector.
            unsafe {
                let per_tick = TSC_PER_TICK.load(Ordering::Relaxed).max(1);
                let deadline = crate::timekeeping::now_counter().wrapping_add(per_tick);
                wrmsr(IA32_TSC_DEADLINE, deadline);
                wrmsr(X2APIC_EOI, 0);
            }
        }
        TimerTier::X2apicPeriodic => {
            // SAFETY: x2APIC enabled; the periodic timer auto-reloads, so only an
            // EOI is needed to allow the next tick.
            unsafe { wrmsr(X2APIC_EOI, 0) };
        }
        TimerTier::PicPit => {
            // SAFETY: the 8259 master EOI for the in-service IRQ0 (vector 0x20),
            // the exact `out 0x20, al` the trampoline used to inline — relocated
            // here, same single port write.
            let mut pic1_cmd: Port<u8> = Port::new(0x20);
            unsafe { pic1_cmd.write(0x20u8) };
        }
    }
}

/// The boot announce line for the active tier (non-golden-shape).
pub fn tier_announce() -> &'static str {
    active_tier().announce()
}

// ---------------------------------------------------------------------------
// P4·SMP·S3 — AP local-APIC enable + INIT-SIPI-SIPI via the x2APIC ICR
// ---------------------------------------------------------------------------

/// Software-enable THIS AP's local APIC (x2APIC mode + SVR + TPR), the per-CPU
/// half of [`enable_x2apic`]. Each AP must software-enable its own local APIC so
/// it can later (S4) receive IPIs; for pure-idle S3 it leaves the LVT timer
/// masked (no scheduling). Reuses the exact `enable_x2apic` body, which only
/// touches per-CPU MSRs (already per-CPU-correct).
///
/// # Safety
/// Call once on an AP after it has loaded the shared CR3/GDT/IDT and set its
/// GS-base, with IRQs masked. The platform must report x2APIC (the BSP already
/// gated on this before starting any AP).
pub unsafe fn enable_x2apic_ap() {
    // SAFETY (delegated): the same architected x2APIC enable sequence the BSP
    // runs, executed on this AP's own (per-CPU banked) APIC MSRs.
    unsafe { enable_x2apic() }
}

/// INIT delivery mode (101) + level assert + edge: the INIT IPI command bits.
const ICR_INIT: u64 = 0x0000_4500;
/// STARTUP delivery mode (110) + level assert: the SIPI command bits (OR the
/// trampoline page number into bits[7:0] for the start vector).
const ICR_STARTUP: u64 = 0x0000_4600;

/// Issue INIT-SIPI-SIPI to the AP with x2APIC id `apic_id`, starting it at the
/// trampoline page `vector_page` (the real-mode start vector = trampoline PA >>
/// 12, e.g. `0x8000 >> 12 = 0x08`). x2APIC ICR is a single MSR write with the
/// destination in the high 32 bits.
///
/// The 10 ms / 200 µs delays are kept for real-HW correctness (Intel-recommended
/// INIT→delay→SIPI→delay→SIPI). Under TCG the AP starts on the first valid SIPI
/// regardless; the delays are bounded so a misconfigured AP cannot wedge boot.
///
/// # Safety
/// Call on the BSP with x2APIC enabled and IRQs masked, after the trampoline has
/// been copied to `vector_page << 12` and the per-AP launch block filled.
pub unsafe fn send_init_sipi(apic_id: u32, vector_page: u8) {
    let dest = (apic_id as u64) << 32;
    // SAFETY: single architected ICR MSR writes; each issues one IPI to `apic_id`.
    unsafe {
        // INIT assert.
        wrmsr(X2APIC_ICR, dest | ICR_INIT);
        delay_tsc_us(10_000); // ~10 ms
        // SIPI #1.
        wrmsr(X2APIC_ICR, dest | ICR_STARTUP | vector_page as u64);
        delay_tsc_us(200);
        // SIPI #2 (ignored by an already-started AP).
        wrmsr(X2APIC_ICR, dest | ICR_STARTUP | vector_page as u64);
    }
}

/// ICR fixed-delivery + physical-dest + edge + assert command bits (delivery
/// mode 000 = Fixed). OR the 8-bit IDT vector into bits[7:0]. Used to send the
/// P4·SMP·S4c TLB-shootdown IPI to a single target CPU (vs INIT/STARTUP).
const ICR_FIXED: u64 = 0x0000_4000;

/// Send a fixed-vector IPI (`vector`) to the CPU with x2APIC id `apic_id` via the
/// x2APIC ICR MSR (the same `0x830` `send_init_sipi` uses, but Fixed delivery).
/// Used for the cross-CPU TLB-shootdown IPI (P4·SMP·S4c).
///
/// # Safety
/// Call with x2APIC enabled on this CPU (true once any timer tier 1/2 or AP
/// bring-up ran). `vector` must be a registered IDT gate that EOIs the APIC.
pub unsafe fn send_fixed_ipi(apic_id: u32, vector: u8) {
    let dest = (apic_id as u64) << 32;
    // SAFETY: a single architected ICR MSR write issuing one Fixed IPI.
    unsafe {
        wrmsr(X2APIC_ICR, dest | ICR_FIXED | vector as u64);
    }
}

/// Signal end-of-interrupt to the local x2APIC (write the EOI MSR). For the
/// shootdown-IPI ISR (a Fixed-delivery vector sets the in-service bit, so it
/// MUST be EOI'd, unlike the spurious vector).
///
/// # Safety
/// Call in an ISR for a real (non-spurious) x2APIC-delivered vector, x2APIC on.
pub unsafe fn eoi() {
    // SAFETY: architected EOI write for the in-service vector.
    unsafe { wrmsr(X2APIC_EOI, 0) };
}

/// Busy-spin roughly `us` microseconds off the calibrated TSC. Bounded by a
/// generous guard so a zero/garbage `tsc_hz` cannot hang. Best-effort under TCG.
fn delay_tsc_us(us: u64) {
    let hz = crate::timekeeping::tsc_hz().max(1);
    let cycles = hz.saturating_mul(us) / 1_000_000;
    let start = crate::timekeeping::now_counter();
    let mut guard: u64 = 5_000_000_000;
    while crate::timekeeping::now_counter().wrapping_sub(start) < cycles && guard > 0 {
        guard -= 1;
        core::hint::spin_loop();
    }
}
