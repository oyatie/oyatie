//! Floor backings for the reshaped P0 HAL capability surface, expressed against
//! the **existing, working** x86_64 bring-up path.
//!
//! # What this module is (and is not)
//!
//! This is the *additive* half of the P0 HAL reshape for x86_64: it re-expresses
//! the backend's already-booting hardware code (the 8259 PIC, the 8254 PIT, the
//! long-mode CR3/PML4 paging the boot trampoline set up, the IDT exception/
//! interrupt frame, and the CPUID feature leaves) as the new capability *shapes*
//! from `hal::{cpu,mm,irq,time}`. Nothing here is on the boot path — the kernel
//! still drives the original [`crate::X86_64`] /
//! `Arch`/`ConsoleWrite`/`MemoryApi`/`InterruptApi`/`TimerApi` impls unchanged.
//! These items exist *alongside* them, proving the new boundary fits the
//! hardware we already run, with byte-identical boot (the boot path is
//! untouched).
//!
//! This mirrors the aarch64 floor backings in
//! `crates/arch-aarch64/src/hal_caps.rs` (Clock/Timer/IrqChip/Pte/
//! PageTableConfig/TrapFrame/CpuCaps) so both arches reach the same HAL floor.
//!
//! # The sealing finding (resolved — these are now real trait impls)
//!
//! Most of the new capability traits — [`hal::time::Clock`], [`hal::time::Timer`],
//! [`hal::irq::IrqChip`], [`hal::mm::Pte`], [`hal::mm::PageTableConfig`],
//! [`hal::cpu::TrapFrame`] — are **sealed** via `hal::sealed::Sealed`. `hal` now
//! declares `#[doc(hidden)] pub mod sealed;` (the standard "sealed-to-downstream,
//! open-to-this-workspace, hidden-from-docs" idiom), so this backend can name
//! `hal::sealed::Sealed` and write the genuine `impl hal::sealed::Sealed` +
//! `impl hal::<trait>` blocks below. The seal still holds against the external
//! ecosystem (the supertrait is hidden from docs, intended only for this
//! workspace's Frame backends).
//!
//! [`hal::mm::PagingConsts`] was already unsealed, so [`X86_64PagingConsts`]
//! below is a real trait impl. The [`hal::cpu::CpuCaps`] snapshot is a struct,
//! and `hal` exposes [`hal::cpu::CpuCaps::new`] (the `#[non_exhaustive]`
//! constructor seam), so [`detected_cpu_caps`] feeds a real, populated `CpuCaps`
//! from the probed CPUID bits.
//!
//! Still **deferred / unimplemented**: [`hal::cpu::UserContext`] (x86_64 has no
//! user mode yet — that is P1; its `run()`/`sysret` path is the switchover slice
//! this boot-path-preserving increment must not touch), and the virtio /
//! confidential / DMA seams (P5/P6).

#![allow(dead_code)] // Additive: not yet called by the kernel's boot path.

use core::arch::x86_64::{__cpuid, __cpuid_count, _rdtsc};

use x86_64::instructions::port::Port;
use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::structures::idt::InterruptStackFrameValue;
use x86_64::structures::paging::PhysFrame;
use x86_64::PhysAddr as X86PhysAddr;

use hal::cpu::CpuCaps;
use hal::irq::{CpuId, IrqChip, IrqVector, MsiMessage};
use hal::mm::{AsidTag, GenericPteFlags, PageTableConfig, PagingConsts, PhysAddr, Pte};
use hal::sealed::Sealed;
use hal::time::{Clock, Timer};
use hal::ArchError;

// ===========================================================================
// PagingConsts  —  UNSEALED, so this is a genuine trait impl.
// ===========================================================================

/// x86_64 translation regime: 4-level (PML4), 4 KiB base page, 48-bit VA,
/// sign-extended canonical addresses.
///
/// This is the floor regime — the standard 4-level / 4 KiB / 48-bit
/// configuration the boot trampoline in [`crate::boot`] already establishes
/// (PML4 -> PDPT -> PD identity map of the low 1 GiB). 5-level paging (LA57,
/// 57-bit VA) is a later opt; these consts describe the full 4-level regime the
/// generic page-table engine in `frame` will target.
pub struct X86_64PagingConsts;

impl PagingConsts for X86_64PagingConsts {
    /// PML4 -> PDPT -> PD -> PT (4 levels for a 48-bit VA at the 4 KiB base).
    const NR_LEVELS: u8 = 4;
    /// 4 KiB base page.
    const BASE_PAGE_SIZE: usize = 4096;
    /// 48-bit canonical virtual addresses (4-level paging, no LA57).
    const ADDRESS_WIDTH: u8 = 48;
    /// x86_64 VAs are canonical: bit 47 sign-extends through bits 48..63.
    const VA_SIGN_EXT: bool = true;
    /// Highest level that may hold a leaf (huge) mapping: the PDPT level (a
    /// 1 GiB page, PDPTE.PS=1). The PML4 level cannot hold a leaf on x86_64.
    const HIGHEST_TRANSLATION_LEVEL: u8 = 3;
}

// ===========================================================================
// CpuCaps  —  a struct to populate (not a trait), so this is real and callable.
// ===========================================================================

// ---- CPUID leaf 1 (EDX/ECX feature flags) ---------------------------------
/// `CPUID.1:ECX[21]` — x2APIC present.
const CPUID_1_ECX_X2APIC: u32 = 1 << 21;
/// `CPUID.1:ECX[24]` — local-APIC TSC-deadline timer mode present.
const CPUID_1_ECX_TSC_DEADLINE: u32 = 1 << 24;
/// `CPUID.1:ECX[17]` — PCID (process-context identifiers) supported.
const CPUID_1_ECX_PCID: u32 = 1 << 17;
/// `CPUID.1:ECX[31]` — running under a hypervisor (set by VMMs, clear on bare
/// metal). The "is a guest" hint, refined by the 0x4000_0000 signature leaf.
const CPUID_1_ECX_HYPERVISOR: u32 = 1 << 31;

// ---- CPUID leaf 7, sub-leaf 0 (EBX structured ext. feature flags) ---------
/// `CPUID.7.0:EBX[7]` — SMEP (Supervisor-Mode Execution Prevention).
const CPUID_7_EBX_SMEP: u32 = 1 << 7;
/// `CPUID.7.0:EBX[20]` — SMAP (Supervisor-Mode Access Prevention).
const CPUID_7_EBX_SMAP: u32 = 1 << 20;
/// `CPUID.7.0:ECX[2]` — UMIP (User-Mode Instruction Prevention: traps
/// `sgdt`/`sidt`/`sldt`/`smsw`/`str` issued from ring 3).
const CPUID_7_ECX_UMIP: u32 = 1 << 2;
/// `CPUID.7.0:ECX[7]` — CET shadow stack (forward/backward-edge CFI hardware).
const CPUID_7_ECX_CET_SS: u32 = 1 << 7;

// ---- CPUID leaf 0x8000_0007 (advanced power management) --------------------
/// `CPUID.8000_0007:EDX[8]` — invariant TSC (TSC ticks at a constant rate
/// regardless of P-state / C-state). Required for the [`X86Tsc`] clock floor.
const CPUID_APM_EDX_INVARIANT_TSC: u32 = 1 << 8;

// ---- CPUID hypervisor signature leaf --------------------------------------
/// Base of the hypervisor CPUID leaf range (`0x4000_0000`); its EBX/ECX/EDX
/// carry the VMM vendor signature ("KVMKVMKVM", "TCGTCGTCGTCG", ...). Non-zero
/// here ⇒ a paravirtualized guest.
const CPUID_HYPERVISOR_BASE: u32 = 0x4000_0000;

/// The capability bits this backend positively detects today from CPUID.
///
/// Arch-local mirror of the subset of [`hal::cpu::CpuCaps`] we can populate; see
/// [`detected_cpu_caps`] for how these fold onto the conservative
/// [`hal::cpu::CpuCaps::FALLBACK`] floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DetectedCaps {
    /// `CPUID.1:ECX[21]` — x2APIC present (the modern per-CPU interrupt chip).
    pub x2apic: bool,
    /// `CPUID.1:ECX[24]` — local-APIC TSC-deadline one-shot timer present.
    pub tsc_deadline: bool,
    /// `CPUID.8000_0007:EDX[8]` — invariant TSC (constant-rate timestamp).
    pub invariant_tsc: bool,
    /// `CPUID.1:ECX[17]` — PCID ⇒ a tagged TLB is usable.
    pub pcid: bool,
    /// `CPUID.7.0:EBX[7]` — SMEP (supervisor cannot execute user pages).
    pub smep: bool,
    /// `CPUID.7.0:EBX[20]` — SMAP (supervisor cannot access user pages).
    pub smap: bool,
    /// `CPUID.7.0:ECX[2]` — UMIP (ring 3 cannot run `sgdt`/`sidt`/`sldt`/
    /// `smsw`/`str`, hiding descriptor-table layout from user mode).
    pub umip: bool,
    /// `CPUID.7.0:ECX[7]` — CET shadow stack (control-flow integrity hardware).
    pub cet_shadow_stack: bool,
    /// A hypervisor signature is present at `CPUID.4000_0000` ⇒ paravirt guest.
    pub paravirt_guest: bool,
}

/// Probe the immutable boot-time CPU capabilities from CPUID (leaf 1, leaf 7,
/// `0x8000_0007`, and the `0x4000_0000` hypervisor signature).
///
/// Best-effort and conservative: any capability we cannot positively detect
/// stays at its safe-fallback value in the eventual `CpuCaps` (consensus C9).
/// CPUID is a pure, side-effect-free read of immutable feature state — no
/// hardware is reconfigured, so this is safe to call (and is not on the
/// unchanged boot path).
pub fn probe_cpu_caps() -> DetectedCaps {
    // `__cpuid` / `__cpuid_count` execute the `cpuid` instruction, a pure read of
    // immutable CPU feature state with no memory or device side effects. They are
    // safe on the x86_64 baseline this Frame targets (the feature is unconditional
    // on every 64-bit part). Leaf 1 and 0x8000_0007 exist on every x86_64 part;
    // leaf 7 is present whenever leaf 0's max-leaf >= 7 (true on all CPUs we run).
    let leaf1 = __cpuid(1);
    let leaf7 = __cpuid_count(7, 0);
    let apm = __cpuid(0x8000_0007);
    let hyper_base = __cpuid(CPUID_HYPERVISOR_BASE);

    let x2apic = (leaf1.ecx & CPUID_1_ECX_X2APIC) != 0;
    let tsc_deadline = (leaf1.ecx & CPUID_1_ECX_TSC_DEADLINE) != 0;
    let pcid = (leaf1.ecx & CPUID_1_ECX_PCID) != 0;
    let hypervisor_bit = (leaf1.ecx & CPUID_1_ECX_HYPERVISOR) != 0;

    let smep = (leaf7.ebx & CPUID_7_EBX_SMEP) != 0;
    let smap = (leaf7.ebx & CPUID_7_EBX_SMAP) != 0;
    let umip = (leaf7.ecx & CPUID_7_ECX_UMIP) != 0;
    let cet_shadow_stack = (leaf7.ecx & CPUID_7_ECX_CET_SS) != 0;

    let invariant_tsc = (apm.edx & CPUID_APM_EDX_INVARIANT_TSC) != 0;

    // A real hypervisor signature leaf reports its max leaf (>= the base) in EAX
    // and a non-zero vendor string in EBX/ECX/EDX. Combine with the leaf-1
    // hypervisor bit for a robust "are we a paravirt guest?" answer.
    let paravirt_guest =
        hypervisor_bit || hyper_base.eax >= CPUID_HYPERVISOR_BASE || hyper_base.ebx != 0;

    DetectedCaps {
        x2apic,
        tsc_deadline,
        invariant_tsc,
        pcid,
        smep,
        smap,
        umip,
        cet_shadow_stack,
        paravirt_guest,
    }
}

/// The conservative all-fallback [`hal::cpu::CpuCaps`] snapshot.
///
/// The floor any arch backend can always produce; [`detected_cpu_caps`] overlays
/// positively-probed bits on top of it.
pub fn fallback_cpu_caps() -> CpuCaps {
    CpuCaps::FALLBACK
}

/// A real, populated [`hal::cpu::CpuCaps`] for this x86_64 boot core.
///
/// Folds the positively-detected CPUID bits from [`probe_cpu_caps`] onto the
/// conservative [`hal::cpu::CpuCaps::FALLBACK`] floor via the
/// [`hal::cpu::CpuCaps::new`] constructor. Capabilities this backend cannot yet
/// positively map onto the `hal` field set (`memory_tagging` — x86 has no MTE
/// analog wired; `confidential` — SEV-SNP/TDX not yet probed) stay at their safe
/// fallback `false` (consensus C9). `lse_atomics` is an aarch64 `+lse` concept
/// with no x86 analog, so it stays `false`. `cpu_count` is the single boot core
/// (1) until SMP bring-up lands (P4).
///
/// Pure CPUID reads — not on the unchanged boot path.
pub fn detected_cpu_caps() -> CpuCaps {
    let d = probe_cpu_caps();
    CpuCaps::new(
        1,                          // cpu_count: single boot core until SMP (P4).
        d.paravirt_guest,           // CPUID hypervisor signature / leaf-1 HV bit.
        d.pcid,                     // CPUID.1:ECX.PCID ⇒ tagged TLB usable.
        d.tsc_deadline,             // CPUID.1:ECX.TSC-deadline ⇒ deadline timer.
        d.smep || d.smap,           // SMEP/SMAP ⇒ hardware user-access protection.
        false,                      // lse_atomics: aarch64 +lse concept; n/a on x86.
        false,                      // memory_tagging: no MTE analog wired yet.
        d.cet_shadow_stack,         // CPUID.7.0:ECX.CET-SS ⇒ CFI hardware.
        false,                      // confidential: SEV-SNP/TDX not yet probed.
    )
}

// ===========================================================================
// Clock + Timer  (hal::time)  —  SEALED. Real trait impls.
// ===========================================================================

/// Floor backing for [`hal::time::Clock`] over RDTSC.
///
/// `now_ns` reads the timestamp counter (`rdtsc`) and converts to nanoseconds
/// using a fixed TSC frequency (`tsc_hz`). On a part with invariant TSC the
/// counter ticks at a constant rate, so a single calibration constant is exact;
/// see [`detected_cpu_caps`] for the invariant-TSC probe. The true LAPIC
/// TSC-deadline fast path (and a runtime calibration against the PIT) is P4 —
/// this floor takes the caller-supplied frequency.
///
/// **Floor frequency:** when the invariant-TSC rate is unknown this backend
/// cannot synthesize a correct ns scale (the PIT only gives a periodic tick, not
/// a TSC ratio, in the current bring-up). The floor therefore stores an explicit
/// `tsc_hz` the caller supplies from calibration; [`X86Tsc::from_pit_calibration`]
/// documents the intended P4 calibration seam (count TSC ticks across a known
/// number of PIT periods). With a `0` frequency `now_ns` returns `0` rather than
/// dividing by zero — the documented degenerate floor.
#[derive(Debug, Clone, Copy)]
pub struct X86Tsc {
    /// Calibrated TSC frequency in Hz. `0` means "uncalibrated" (→ `now_ns` 0).
    tsc_hz: u64,
}

impl X86Tsc {
    /// Construct a clock with an explicit calibrated TSC frequency (Hz).
    pub const fn new(tsc_hz: u64) -> Self {
        Self { tsc_hz }
    }

    /// The intended P4 calibration seam: measure how many TSC ticks elapse over
    /// `pit_periods` PIT periods at [`crate::timer::PIT_FREQUENCY`] and derive
    /// the TSC frequency. Documented here as the floor's calibration contract;
    /// the actual busy-wait calibration loop (which must run with IRQs masked
    /// against the live PIT) lands in P4 alongside the LAPIC bring-up. For now it
    /// derives the frequency arithmetically from a measured tick delta.
    pub const fn from_pit_calibration(tsc_ticks: u64, pit_periods: u64) -> Self {
        if pit_periods == 0 {
            return Self { tsc_hz: 0 };
        }
        // tsc_hz = tsc_ticks / (pit_periods / PIT_FREQUENCY)
        //        = tsc_ticks * PIT_FREQUENCY / pit_periods
        let pit_freq = crate::timer::PIT_FREQUENCY as u64;
        Self {
            tsc_hz: tsc_ticks.saturating_mul(pit_freq) / pit_periods,
        }
    }
}

impl Sealed for X86Tsc {}

impl Clock for X86Tsc {
    /// Monotonic nanoseconds: `ns = tsc * 1e9 / tsc_hz`.
    ///
    /// Uses a `u128` intermediate so the multiply does not overflow before the
    /// divide. Returns `0` when uncalibrated (`tsc_hz == 0`) — the documented
    /// degenerate floor (see the type docs).
    fn now_ns(&self) -> u64 {
        if self.tsc_hz == 0 {
            return 0;
        }
        // SAFETY: `_rdtsc` reads the timestamp counter; a pure read with no
        // memory or device side effects, valid in long mode at any privilege.
        let tsc = unsafe { _rdtsc() } as u128;
        ((tsc * 1_000_000_000u128) / self.tsc_hz as u128) as u64
    }
}

/// Floor backing for [`hal::time::Timer`] over the existing 8254 PIT
/// (`crate::timer`).
///
/// The deadline shape (`set_deadline_ns` / `cancel`) is emulated over the
/// periodic PIT: `set_deadline_ns` programs the PIT for the nearest periodic
/// rate that covers the requested deadline, exactly as the consensus Q1 fallback
/// prescribes ("the fallback body programs the nearest periodic tick that covers
/// the deadline"). The true one-shot is the LAPIC TSC-deadline fast path (P4);
/// at the floor periodic-emulation over the PIT is the honest backing. `cancel`
/// masks IRQ0 at the PIC (the existing `crate::interrupts::PICS`).
///
/// Real `impl hal::time::Timer`, now that `Sealed` is reachable. The bodies
/// reuse the existing [`crate::timer::init`] PIT programming and the
/// [`crate::interrupts::PICS`] masking the bring-up already owns.
pub struct X86PitTimer {
    /// The clock used to translate an absolute deadline into a relative delay.
    clock: X86Tsc,
}

impl X86PitTimer {
    /// Build a PIT-backed deadline timer over the given calibrated [`X86Tsc`].
    pub const fn new(clock: X86Tsc) -> Self {
        Self { clock }
    }

    /// PIT channel-0 data port (mirrors `crate::timer`'s private constant).
    const PIT_CHANNEL0_DATA: u16 = 0x40;
    /// PIT mode/command port.
    const PIT_MODE_COMMAND: u16 = 0x43;
}

impl Sealed for X86PitTimer {}

impl Timer for X86PitTimer {
    /// Arm a one-shot for the absolute monotonic instant `deadline_ns` by
    /// emulating it over the periodic PIT.
    ///
    /// Computes the remaining delay `deadline_ns - now_ns()` and programs PIT
    /// channel 0 (mode 2, rate generator) to fire at the rate whose period is no
    /// longer than that delay (so the periodic tick *covers* the deadline, per
    /// the consensus Q1 fallback). A deadline already in the past clamps to the
    /// fastest representable rate so it fires promptly. Requires a calibrated
    /// clock (`tsc_hz != 0`); without one the delay is unknown and we report
    /// [`ArchError::Interrupt`].
    fn set_deadline_ns(&mut self, deadline_ns: u64) -> Result<(), ArchError> {
        if self.clock.tsc_hz == 0 {
            return Err(ArchError::Interrupt);
        }
        let now = self.clock.now_ns();
        // Delay until the deadline; past deadlines → 1 ns so we pick the fastest
        // rate and fire promptly.
        let delay_ns = deadline_ns.saturating_sub(now).max(1);

        // Desired periodic frequency that covers the deadline: a period <=
        // delay_ns means freq >= 1e9 / delay_ns. Round up to guarantee coverage.
        let pit_freq = crate::timer::PIT_FREQUENCY as u64;
        let desired_hz = 1_000_000_000u64.div_ceil(delay_ns).max(1);
        // PIT divisor = PIT_FREQUENCY / desired_hz, clamped to the 16-bit reload.
        let divisor = (pit_freq / desired_hz).clamp(1, 0xFFFF) as u16;

        let mut command: Port<u8> = Port::new(Self::PIT_MODE_COMMAND);
        let mut data: Port<u8> = Port::new(Self::PIT_CHANNEL0_DATA);
        // SAFETY: identical PIT programming to the audited `crate::timer::init`
        // (command byte 0x36 = channel 0, lobyte/hibyte access, mode 2 rate
        // generator, binary; then the 16-bit reload low byte then high byte).
        // Raw port I/O to the PIT is the device's sole contract; no memory
        // effects.
        unsafe {
            command.write(0x36u8);
            data.write((divisor & 0xFF) as u8);
            data.write((divisor >> 8) as u8);
        }
        Ok(())
    }

    /// Disarm any pending deadline by masking IRQ0 (the PIT line) at the master
    /// 8259 PIC. Idempotent — masking an already-masked line is a no-op.
    fn cancel(&mut self) {
        // SAFETY: re-uses the existing PIC the bring-up configured. Setting the
        // master mask to 0xFF masks every line including IRQ0 (PIT). This is the
        // documented PIC mask protocol; no memory effects. (The slave mask stays
        // fully masked as the bring-up left it.)
        unsafe {
            crate::interrupts::PICS.lock().write_masks(0xFF, 0xFF);
        }
    }
}

// ===========================================================================
// IrqChip  (hal::irq)  —  SEALED. Real trait impl over the existing 8259 PIC.
// ===========================================================================

/// Floor backing for [`hal::irq::IrqChip`] over the existing 8259 PIC
/// (`crate::interrupts::PICS`).
///
/// `enable`/`disable` re-express the PIC mask register (clear/set the line's
/// mask bit); `eoi` re-expresses the existing end-of-interrupt the timer ISR
/// already sends (`notify_end_of_interrupt`). The PIC has no architected IPI and
/// no MSI, so `send_ipi`/`map_msi` are the documented [`ArchError::Unsupported`]
/// floor — x2APIC IPIs and IOAPIC/MSI(-X) are the P4/P5 fast path.
///
/// The [`hal::irq::IrqVector`] carries the **CPU vector** (e.g. `0x20` for the
/// PIT/IRQ0), matching the remapped offsets in [`crate::interrupts`]; we map it
/// back to the 0..15 PIC line by subtracting [`crate::interrupts::PIC_1_OFFSET`].
///
/// Real `impl hal::irq::IrqChip`, now that `Sealed` is reachable.
pub struct Pic8259IrqChip;

impl Pic8259IrqChip {
    /// Master-PIC mask port (8259 #1 data register).
    const PIC1_DATA: u16 = 0x21;
    /// Slave-PIC mask port (8259 #2 data register).
    const PIC2_DATA: u16 = 0xA1;

    /// Map a remapped CPU vector back to its 0..15 legacy PIC IRQ line, if it
    /// falls in the PIC range the bring-up remapped (`0x20..0x30`).
    fn vector_to_irq_line(vector: IrqVector) -> Option<u8> {
        let v = vector.0;
        let base = crate::interrupts::PIC_1_OFFSET as u32;
        if v >= base && v < base + 16 {
            Some((v - base) as u8)
        } else {
            None
        }
    }

    /// Read both current PIC mask bytes (master, slave) via the data ports.
    ///
    /// # Safety
    /// Raw port I/O to the 8259 data registers; valid post-`init_pic`.
    unsafe fn read_masks() -> (u8, u8) {
        let mut m1: Port<u8> = Port::new(Self::PIC1_DATA);
        let mut m2: Port<u8> = Port::new(Self::PIC2_DATA);
        // SAFETY: reading the PIC data registers returns the current IMR; pure
        // device reads with no side effects.
        unsafe { (m1.read(), m2.read()) }
    }

    /// Clear (enable) or set (disable) the mask bit for `line` (0..15) across the
    /// master/slave pair, preserving every other line's current mask.
    ///
    /// # Safety
    /// Raw port I/O to the 8259 data registers; valid post-`init_pic`.
    unsafe fn set_line_masked(line: u8, masked: bool) {
        // SAFETY (delegated): read-modify-write of the PIC mask via the data
        // ports under the bring-up PIC; no memory effects.
        let (mut m1, mut m2) = unsafe { Self::read_masks() };
        if line < 8 {
            let bit = 1u8 << line;
            if masked {
                m1 |= bit;
            } else {
                m1 &= !bit;
            }
        } else {
            let bit = 1u8 << (line - 8);
            if masked {
                m2 |= bit;
            } else {
                m2 &= !bit;
            }
        }
        // SAFETY: write the recomputed IMR bytes back via the existing PIC.
        unsafe {
            crate::interrupts::PICS.lock().write_masks(m1, m2);
        }
    }
}

impl Sealed for Pic8259IrqChip {}

impl IrqChip for Pic8259IrqChip {
    /// Enable delivery of `vector` by clearing its mask bit in the 8259 IMR.
    ///
    /// Re-expresses the unmask the bring-up does for IRQ0; only vectors in the
    /// remapped PIC range (`0x20..0x30`) map to a legacy line. A vector outside
    /// that range has no PIC line (it would be an APIC/IOAPIC vector) and is the
    /// documented `Unsupported` floor until the IOAPIC lands (P4).
    fn enable(&mut self, vector: IrqVector) -> Result<(), ArchError> {
        let line = Self::vector_to_irq_line(vector).ok_or(ArchError::Unsupported)?;
        // SAFETY: read-modify-write of the PIC mask via the existing bring-up
        // PIC; clearing the line's bit unmasks it. Mirrors the audited
        // `init_pic` masking exactly.
        unsafe {
            Self::set_line_masked(line, false);
        }
        Ok(())
    }

    /// Disable delivery of `vector` by setting its mask bit in the 8259 IMR.
    fn disable(&mut self, vector: IrqVector) -> Result<(), ArchError> {
        let line = Self::vector_to_irq_line(vector).ok_or(ArchError::Unsupported)?;
        // SAFETY: read-modify-write of the PIC mask; setting the line's bit masks
        // it. Same audited PIC path.
        unsafe {
            Self::set_line_masked(line, true);
        }
        Ok(())
    }

    /// End-of-interrupt for the in-service `vector`.
    ///
    /// Re-expresses the `notify_end_of_interrupt` the timer ISR already calls
    /// (the chained-PIC driver issues the OCW2 EOI, and a second EOI to the
    /// master for a slave-line IRQ). For a vector outside the PIC range this is a
    /// no-op (APIC EOI is the P4 path).
    fn eoi(&mut self, vector: IrqVector) {
        // SAFETY: notifying EOI for the exact in-service vector is the required
        // 8259 protocol; identical to the timer ISR's existing call.
        unsafe {
            crate::interrupts::PICS
                .lock()
                .notify_end_of_interrupt(vector.0 as u8);
        }
    }

    /// The 8259 PIC has no architected inter-processor interrupt; SMP IPIs are
    /// the x2APIC fast path (P4 SMP slice). Documented `Unsupported` floor.
    fn send_ipi(&mut self, _target: CpuId, _vector: IrqVector) -> Result<(), ArchError> {
        Err(ArchError::Unsupported)
    }

    /// The 8259 PIC has no MSI/IOAPIC; MSI(-X) mapping for virtio multi-queue is
    /// the IOAPIC/x2APIC fast path (P5). Documented `Unsupported` floor.
    fn map_msi(&mut self, _target: CpuId, _vector: IrqVector) -> Result<MsiMessage, ArchError> {
        Err(ArchError::Unsupported)
    }
}

// ===========================================================================
// Pte + PageTableConfig  (hal::mm)  —  SEALED. Real trait impls over x86 PML4.
// ===========================================================================

// x86_64 page-table entry flag bits (Intel SDM Vol. 3A §4.5, IA-32e paging).
/// Bit 0 — Present (P): the entry maps a valid translation.
const PTE_PRESENT: u64 = 1 << 0;
/// Bit 1 — Read/Write (RW): writable when set, read-only when clear.
const PTE_WRITABLE: u64 = 1 << 1;
/// Bit 2 — User/Supervisor (US): user-accessible (ring 3) when set.
const PTE_USER: u64 = 1 << 2;
/// Bit 7 — Page Size (PS): at PDPTE/PDE level a set PS marks a leaf (huge) page.
const PTE_PAGE_SIZE: u64 = 1 << 7;
/// Bit 8 — Global (G): the translation is not flushed on a CR3 (non-PCID) write.
const PTE_GLOBAL: u64 = 1 << 8;
/// Bit 63 — No-Execute (NX): instruction fetch is forbidden when set (requires
/// EFER.NXE). Executable therefore means this bit is *clear*.
const PTE_NO_EXECUTE: u64 = 1 << 63;
/// Output-address mask: physical frame bits [51:12] of a 4 KiB PTE.
const PTE_ADDR_MASK: u64 = 0x000f_ffff_ffff_f000;

/// An x86_64 page-table entry (any level), presented as the [`hal::mm::Pte`]
/// shape.
///
/// Wraps the raw 64-bit entry the long-mode tables use (the boot trampoline's
/// PML4/PDPT/PD in [`crate::boot`], and the generic tables `frame` will build),
/// decoding the standard IA-32e paging bits (P/RW/US/PS/G/NX).
///
/// Real `impl hal::mm::Pte`, now that `Sealed` is reachable. `Pte: Sealed + Copy`
/// — this type derives `Copy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct X86Pte(pub u64);

impl Sealed for X86Pte {}

impl Pte for X86Pte {
    /// Decode typed flags from the IA-32e paging entry.
    fn flags(&self) -> GenericPteFlags {
        let e = self.0;
        GenericPteFlags {
            present: (e & PTE_PRESENT) != 0,
            writable: (e & PTE_WRITABLE) != 0,
            // Executable on x86_64 is the *absence* of the NX bit (with NXE set);
            // a present entry with NX clear permits instruction fetch.
            executable: (e & PTE_NO_EXECUTE) == 0,
            user: (e & PTE_USER) != 0,
            global: (e & PTE_GLOBAL) != 0,
        }
    }

    /// The output physical address (next-level table base, or the mapped frame
    /// for a leaf). Bits [51:12] of the entry.
    fn address(&self) -> PhysAddr {
        PhysAddr((self.0 & PTE_ADDR_MASK) as usize)
    }

    /// Does this entry terminate the walk?
    ///
    /// A present PTE at the PT level (4 KiB leaf) and a present entry with the
    /// PS bit set at the PDPTE/PDE level (1 GiB / 2 MiB huge page) are leaves;
    /// a present entry with PS clear above the PT level is a table pointer. The
    /// level is not encoded in the word, so at the floor we treat a present entry
    /// whose PS bit is set as a leaf (the unambiguous huge-page case); the
    /// level-aware PT-leaf disambiguation lands with the generic cursor in
    /// `frame`, which supplies the level.
    fn is_leaf(&self) -> bool {
        (self.0 & PTE_PRESENT) != 0 && (self.0 & PTE_PAGE_SIZE) != 0
    }
}

/// Floor backing for [`hal::mm::PageTableConfig`] over CR3 (the PML4 root).
///
/// `switch_to` re-expresses the long-mode address-space switch: write the new
/// PML4 physical base into CR3, which on x86_64 (without PCID) **flushes the
/// entire non-global TLB** as a side effect — exactly the full-flush floor the
/// spec asks for. The [`hal::mm::AsidTag`] is ignored (PCID/INVPCID-tagged
/// switching that suppresses the flush is the P4 fast path, and a
/// signature-compatible swap-in since the opaque tag is already in the
/// signature).
///
/// Real `impl hal::mm::PageTableConfig`, now that `Sealed` is reachable. Its
/// associated types are [`X86Pte`] / [`X86_64PagingConsts`].
pub struct X86PageTableConfig;

impl Sealed for X86PageTableConfig {}

impl PageTableConfig for X86PageTableConfig {
    type Pte = X86Pte;
    type Consts = X86_64PagingConsts;

    /// Activate `root` (a PML4 physical base) with a full TLB flush, ignoring
    /// `asid_tag` (full-flush floor — writing CR3 without PCID flushes the entire
    /// non-global TLB; PCID-tagged switching is the P4 fast path).
    ///
    /// The `hal` trait declares this method **safe**; the CR3 write it performs
    /// carries the same invariant the boot trampoline's CR3 load upholds: `root`
    /// must be the physical base of a valid 4-level PML4 whose entries keep the
    /// kernel identity map live (the boot trampoline guarantees this for the demo
    /// table; every future `frame` address space must as well). The unsafe CR3
    /// op is contained here behind the safe Frame seam.
    fn switch_to(&self, root: PhysAddr, _asid_tag: AsidTag) {
        // Build a `PhysFrame` for the 4 KiB-aligned PML4 base. `new_truncate`
        // drops bits [63:52] (the address is already a 52-bit physical base);
        // `containing_address` floors to the enclosing 4 KiB frame.
        let phys = X86PhysAddr::new_truncate(root.as_usize() as u64);
        let frame = PhysFrame::containing_address(phys);
        // SAFETY (contract above): writing CR3 activates `root` as the active
        // PML4 and full-flushes the non-global TLB — the same operation the boot
        // trampoline performs, behind this safe Frame seam. `Cr3Flags::empty()`
        // selects a plain (non-PCID) CR3 write, i.e. the full-flush behavior.
        unsafe {
            Cr3::write(frame, Cr3Flags::empty());
        }
    }
}

// ===========================================================================
// TrapFrame  (hal::cpu)  —  SEALED. Real accessors over the existing IDT frame.
// ===========================================================================

/// Floor backing for [`hal::cpu::TrapFrame`] over the **existing** x86_64 IDT
/// interrupt frame ([`x86_64::structures::idt::InterruptStackFrameValue`]) the
/// handlers in [`crate::interrupts`] already receive.
///
/// The CPU pushes `rip`/`cs`/`rflags`/`rsp`/`ss` on a trap; the `x86_64` crate
/// surfaces that as `InterruptStackFrameValue`. x86 carries the **vector** and
/// the **error code** *outside* that pushed frame (the vector is which IDT entry
/// fired; the error code is a separate handler argument the CPU pushes only for
/// the faults that have one), so this view bundles them explicitly — mirroring
/// how the aarch64 view carries the trap `kind` separately from its register
/// frame. These are read-only accessors over an already-captured frame: adding
/// them requires **no** change to the IDT entry stubs or the boot path.
///
/// The privileged `rflags` (IF/IOPL) is deliberately *not* exposed, matching the
/// trait's "hide privileged flags" contract (lesson A18).
///
/// [`hal::cpu::UserContext`] remains **deferred**: x86_64 has no user mode yet
/// (P1), and its `run()` half (enter ring 3 / `sysret` / return-reason loop)
/// would rewire a boot path that does not exist, which this additive increment
/// must not introduce. `UserContext` lands in the P1 user-mode slice; its
/// register accessors would read/write the same `rip`/`rsp`/`rax` slots of a
/// full saved-register frame.
pub struct TrapFrameView<'a> {
    /// The CPU-pushed interrupt frame (rip/cs/rflags/rsp/ss).
    frame: &'a InterruptStackFrameValue,
    /// The IDT vector that fired (x86 carries this outside the pushed frame).
    vector: u64,
    /// The CPU error code for faults that push one (0 for those that do not).
    error_code: u64,
}

impl<'a> TrapFrameView<'a> {
    /// Wrap an already-captured IDT frame with its vector and error code.
    ///
    /// The IDT handler knows which vector it is (it is the specific
    /// `extern "x86-interrupt"` fn) and receives the error code as its second
    /// argument for the faults that carry one; pass `0` for vectors without an
    /// error code. No hardware is touched — this is a pure view.
    pub fn new(frame: &'a InterruptStackFrameValue, vector: u64, error_code: u64) -> Self {
        Self {
            frame,
            vector,
            error_code,
        }
    }
}

impl Sealed for TrapFrameView<'_> {}

impl hal::cpu::TrapFrame for TrapFrameView<'_> {
    /// The saved `RIP`.
    fn instruction_pointer(&self) -> usize {
        self.frame.instruction_pointer.as_u64() as usize
    }

    /// The saved `RSP`.
    fn stack_pointer(&self) -> usize {
        self.frame.stack_pointer.as_u64() as usize
    }

    /// The architectural trap/IDT vector that caused entry.
    fn trap_number(&self) -> usize {
        self.vector as usize
    }

    /// The CPU-pushed error code, or `0` for traps that carry none.
    fn error_code(&self) -> usize {
        self.error_code as usize
    }
}
