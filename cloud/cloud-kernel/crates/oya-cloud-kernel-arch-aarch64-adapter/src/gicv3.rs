//! GICv3 interrupt-controller driver for the QEMU `virt` machine.
//!
//! Frame code (TCB). This is the aarch64 mirror of the x86 x2APIC slice
//! (`crates/arch-x86_64/src/apic.rs`): a runtime-probed modern interrupt chip
//! that retires the legacy controller (here GICv2, [`crate::gic`]) for
//! SPI/PPI/SGI delivery while keeping it as a **byte-identical fallback**
//! selected once at boot.
//!
//! The active version is probed ONCE from `GICD_PIDR2[7:4]` and stored in an
//! [`AtomicU8`] ([`ACTIVE_GIC`]) — the exact shape of `apic::ACTIVE_TIER`. The
//! per-IRQ ack/EOI then reads it `Relaxed` (a single predictable load) to pick
//! the register source: `ICC_IAR1_EL1`/`ICC_EOIR1_EL1` (v3) vs `GICC_IAR`/
//! `GICC_EOIR` (v2, in [`crate::gic`]).
//!
//! GICv3 differs from GICv2 in three structural ways, each handled below:
//!   * **GICD** (distributor, base `0x0800_0000`, same as v2 but a 64 KiB
//!     frame): Affinity Routing (`ARE_NS`) must be enabled BEFORE Group1, and
//!     the legacy byte-target model is gone.
//!   * **GICR** (per-CPU redistributor, base `0x080A_0000`, no v2 analog): the
//!     boot CPU's redistributor must be woken (`GICR_WAKER`), and PPIs/SGIs are
//!     configured in its SGI_base frame (`+0x10000`) rather than the
//!     distributor.
//!   * **ICC** (CPU interface): now a set of `S3_0_C12_*` system registers, not
//!     the `GICC_*` MMIO block. `aarch64-cpu` 9.4.0 has no `ICC_*` accessors, so
//!     these use raw `msr`/`mrs` with the literal encodings (the same idiom as
//!     `process.rs` `mrs {}, sp_el0` and `user.rs` `msr sp_el0/elr_el1/...`).
//!
//! All MMIO + system-register `unsafe` lives in this file; the crate denies
//! `unsafe_op_in_unsafe_fn`, so every block carries a `// SAFETY:` note.

use core::sync::atomic::{AtomicU8, Ordering};

use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_structs;
use tock_registers::registers::ReadWrite;

/// Distributor MMIO base on QEMU `virt` — IDENTICAL to the GICv2 base
/// ([`crate::gic`] `GICD_BASE`), so the `PIDR2` probe reads at the same address
/// regardless of the active GIC version (DTB-confirmed: GICD `0x8000000` for
/// both `gic-version=2` and `gic-version=3`).
const GICD_BASE: usize = 0x0800_0000;

/// Redistributor RD_base on QEMU `virt` `gic-version=3` (DTB-confirmed:
/// `reg = <... 0x80a0000 0x00 0xf60000>`, `#redistributor-regions = <1>`). The
/// `GICR_WAKER` register lives here.
const GICR_RD_BASE: usize = 0x080A_0000;

/// Redistributor SGI_base = RD_base + 64 KiB. The per-PPI/SGI config registers
/// (`IGROUPR0`, `IGRPMODR0`, `IPRIORITYR`, `ISENABLER0`) live in this frame, NOT
/// in RD_base.
const GICR_SGI_BASE: usize = GICR_RD_BASE + 0x1_0000;

/// Per-CPU redistributor stride on QEMU `virt` `gic-version=3`. CONFIRMED via
/// dumpdtb (`-M virt,gic-version=3 -smp 4`): the GICR region is `0x80a0000` size
/// `0xf60000` = 123 × `0x20000`, `#redistributor-regions = <1>`. So CPU `k`'s
/// redistributor RD_base = `GICR_RD_BASE + k * GICR_STRIDE` (RD frame 64 KiB +
/// SGI frame 64 KiB = `0x20000`). P4·SMP·S3.
const GICR_STRIDE: usize = 0x2_0000;

/// `GICD_CTLR.ARE_NS` (Affinity Routing Enable, non-secure) — bit 4. Must be set
/// before enabling Group1 so the hardware uses the affinity routing model.
const GICD_CTLR_ARE_NS: u32 = 1 << 4;
/// `GICD_CTLR.EnableGrp1NS` — bit 1. Forwards Group1 non-secure interrupts.
const GICD_CTLR_ENABLE_GRP1NS: u32 = 1 << 1;

/// `GICR_WAKER.ProcessorSleep` — bit 1. Clear to wake the redistributor.
const GICR_WAKER_PROCESSOR_SLEEP: u32 = 1 << 1;
/// `GICR_WAKER.ChildrenAsleep` — bit 2. Polls to 0 once the redistributor is up.
const GICR_WAKER_CHILDREN_ASLEEP: u32 = 1 << 2;

/// Bounded guard on the `ChildrenAsleep` poll so a misprobe cannot hang boot
/// (mirrors the bounded spins in `apic.rs`). Generous for TCG.
const WAKER_POLL_GUARD: u32 = 1_000_000;

/// `ICC_SRE_EL1.SRE` — bit 0. Enables the system-register CPU interface.
const ICC_SRE_EL1_SRE: u64 = 1 << 0;
/// `ICC_IGRPEN1_EL1.Enable` — bit 0. Enables Group1 interrupt forwarding.
const ICC_IGRPEN1_EL1_ENABLE: u64 = 1 << 0;
/// Priority mask = accept all (mirrors `GICC_PMR` `0xFF` in [`crate::gic`]).
const ICC_PMR_ACCEPT_ALL: u64 = 0xFF;

/// Spurious-interrupt INTIDs returned by `ICC_IAR1_EL1` when nothing is pending
/// (same `>= 1020` convention as the GICv2 `GICC_IAR`, [`crate::gic`]).
const SPURIOUS_MIN: u32 = 1020;

register_structs! {
    /// GICv3 distributor registers (subset we use). The GICv3 distributor is a
    /// 64 KiB frame, but the version PROBE reads `PIDR2` at the **`0xFE8`**
    /// offset, which is decoded by BOTH the QEMU GICv2 model (returns ArchRev=2)
    /// AND the GICv3 model (returns 0 there) — see [`probe_version`]. The
    /// GICv3-only `0xFFE8` PIDR2 mirror is deliberately NOT read, because the
    /// QEMU GICv2 distributor only decodes a 4 KiB window and a read at `0xFFE8`
    /// faults (an external abort) on the v2 board. This block does NOT widen the
    /// v2 `GicdRegs` in [`crate::gic`] (kept byte-identical for the v2 leg).
    GicdV3Regs {
        (0x0000 => ctlr: ReadWrite<u32>),
        (0x0004 => _reserved0),
        (0x0100 => isenabler: [ReadWrite<u32>; 32]),
        (0x0180 => _reserved1),
        (0x0400 => ipriorityr: [ReadWrite<u32>; 256]),
        (0x0800 => _reserved2),
        (0x0FE8 => pidr2: ReadWrite<u32>),
        (0x0FEC => _reserved3),
        (0x1000 => @END),
    }
}

register_structs! {
    /// GICv3 redistributor RD_base frame (subset we use): only `GICR_WAKER`.
    GicrRdRegs {
        (0x0000 => _reserved0),
        (0x0014 => waker: ReadWrite<u32>),
        (0x0018 => _reserved1),
        (0x1_0000 => @END),
    }
}

register_structs! {
    /// GICv3 redistributor SGI_base frame (subset we use): the per-PPI/SGI
    /// config registers for INTIDs 0..31.
    GicrSgiRegs {
        (0x0000 => _reserved0),
        (0x0080 => igroupr0: ReadWrite<u32>),
        (0x0084 => _reserved1),
        (0x0100 => isenabler0: ReadWrite<u32>),
        (0x0104 => _reserved2),
        (0x0400 => ipriorityr: [ReadWrite<u32>; 8]),
        (0x0420 => _reserved3),
        (0x0D00 => igrpmodr0: ReadWrite<u32>),
        (0x0D04 => _reserved4),
        (0x1_0000 => @END),
    }
}

fn gicd() -> &'static GicdV3Regs {
    // SAFETY: GICD_BASE is the QEMU `virt` distributor base, identity-mapped as
    // Device memory (1 GiB device block, `mmu.rs`); single-core bring-up means
    // no aliasing writers. The GICv3 GICD is a 64 KiB frame, fully mapped.
    unsafe { &*(GICD_BASE as *const GicdV3Regs) }
}

fn gicr_rd() -> &'static GicrRdRegs {
    // SAFETY: GICR_RD_BASE is the DTB-confirmed redistributor RD_base on QEMU
    // `virt` gic-version=3, inside the identity-mapped device block.
    unsafe { &*(GICR_RD_BASE as *const GicrRdRegs) }
}

fn gicr_sgi() -> &'static GicrSgiRegs {
    // SAFETY: GICR_SGI_BASE = RD_base + 64 KiB, the redistributor's SGI_base
    // frame, inside the identity-mapped device block.
    unsafe { &*(GICR_SGI_BASE as *const GicrSgiRegs) }
}

// ---------------------------------------------------------------------------
// Boot-immutable GIC-version selection (mirrors apic::ACTIVE_TIER)
// ---------------------------------------------------------------------------

/// Which GIC architecture version is live. Probed ONCE at boot from
/// `GICD_PIDR2[7:4]` ([`probe_version`]) and stored in [`ACTIVE_GIC`]; the
/// per-IRQ ack reads it `Relaxed` (a single predictable load — no MMIO probe,
/// see [`handle_irq`]). This is the structural analog of `apic::TimerTier`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GicVersion {
    /// GICv2: distributor + `GICC_*` MMIO CPU interface ([`crate::gic`]).
    /// Byte-identical to pre-P4-slice-2.
    V2 = 0,
    /// GICv3: distributor + redistributor + ICC system-register CPU interface.
    V3 = 1,
}

impl GicVersion {
    /// The non-golden-shape boot announce line for this chip (the exact mirror
    /// of `TimerTier::announce`). Emitted once via `kprintln!`; deliberately NOT
    /// in the oracle's `[pid N] syscall` shape, so the golden is untouched.
    pub fn announce(self) -> &'static str {
        match self {
            GicVersion::V2 => "irq: GICv2 (MMIO CPU interface)",
            GicVersion::V3 => "irq: GICv3 (sysreg CPU interface)",
        }
    }
}

/// The live GIC version, as a `u8` (the [`GicVersion`] discriminant). Written
/// once at boot in [`store_version`], read `Relaxed` per IRQ in [`handle_irq`].
/// Defaults to `V2` (0) so any path that never runs the probe keeps today's
/// exact GICv2 behavior — the same conservative-default discipline as
/// `apic::ACTIVE_TIER` defaulting to `PicPit`.
static ACTIVE_GIC: AtomicU8 = AtomicU8::new(GicVersion::V2 as u8);

/// Read the live GIC version (read `Relaxed`; written once at boot).
#[inline]
pub fn active_gic() -> GicVersion {
    match ACTIVE_GIC.load(Ordering::Relaxed) {
        1 => GicVersion::V3,
        _ => GicVersion::V2,
    }
}

/// Store the probed GIC version (called once at boot, before IRQs are unmasked).
#[inline]
pub fn store_version(v: GicVersion) {
    ACTIVE_GIC.store(v as u8, Ordering::Relaxed);
}

/// Probe the GIC architecture version from `GICD_PIDR2` at the **`0xFE8`**
/// offset — the one identification offset decoded by BOTH QEMU GIC models
/// (empirically confirmed on QEMU 11): the **GICv2** distributor returns its
/// CoreLink PIDR2 there with `ArchRev[7:4] == 2` (raw `0x2B`), while the
/// **GICv3** distributor decodes its real PIDR2 only at `0xFFE8` and returns
/// `0` at `0xFE8`. A read at the GICv3-only `0xFFE8` offset **faults** on the
/// v2 board (it decodes only a 4 KiB window), so this probe never touches it.
///
/// Decision: `ArchRev == 2` (the v2 distributor responding) => [`GicVersion::V2`];
/// anything else (the v3 distributor's `0` at `0xFE8`, or any future ArchRev)
/// => [`GicVersion::V3`]. This reads ONE always-decoded distributor word and
/// touches no CPU feature register, no `GICC_*`, and no redistributor frame.
pub fn probe_version() -> GicVersion {
    let arch_rev = (gicd().pidr2.get() >> 4) & 0xF;
    if arch_rev == 2 {
        GicVersion::V2
    } else {
        GicVersion::V3
    }
}

// ---------------------------------------------------------------------------
// Low-level ICC system-register access (raw msr/mrs S3_0_C12_* encodings)
// ---------------------------------------------------------------------------
//
// `aarch64-cpu` 9.4.0 has no `ICC_*` accessors and this toolchain's assembler
// has no `ICC_*` mnemonic without `-march=...+gic`, so we emit the literal
// `S3_<op1>_C<Cn>_C<Cm>_<op2>` operand form (always assembles). Each helper is
// a single architected sysreg access with no memory effects beyond the named
// GIC CPU-interface register.

/// `msr ICC_SRE_EL1, value` — `S3_0_C12_C12_5` (set SRE to enable the sysreg
/// interface). Must be followed by an `isb` before any other ICC write.
///
/// # Safety
/// Call on the boot core during bring-up; enables the GICv3 CPU interface.
#[inline]
unsafe fn write_icc_sre_el1(value: u64) {
    // SAFETY (delegated to caller): a single architected sysreg write enabling
    // the GICv3 system-register CPU interface; no memory effects.
    unsafe {
        core::arch::asm!("msr S3_0_C12_C12_5, {v}", v = in(reg) value, options(nostack, nomem));
    }
}

/// `msr ICC_PMR_EL1, value` — `S3_0_C4_C6_0` (priority mask).
///
/// # Safety
/// Call after SRE is enabled + an `isb`.
#[inline]
unsafe fn write_icc_pmr_el1(value: u64) {
    // SAFETY (delegated to caller): architected priority-mask write.
    unsafe {
        core::arch::asm!("msr S3_0_C4_C6_0, {v}", v = in(reg) value, options(nostack, nomem));
    }
}

/// `msr ICC_BPR1_EL1, value` — `S3_0_C12_C12_3` (binary point, group 1).
///
/// # Safety
/// Call after SRE is enabled + an `isb`.
#[inline]
unsafe fn write_icc_bpr1_el1(value: u64) {
    // SAFETY (delegated to caller): architected binary-point write.
    unsafe {
        core::arch::asm!("msr S3_0_C12_C12_3, {v}", v = in(reg) value, options(nostack, nomem));
    }
}

/// `msr ICC_CTLR_EL1, value` — `S3_0_C12_C12_4` (CPU-interface control;
/// `value = 0` => `EOImode = 0`, combined priority-drop + deactivate on EOIR1).
///
/// # Safety
/// Call after SRE is enabled + an `isb`.
#[inline]
unsafe fn write_icc_ctlr_el1(value: u64) {
    // SAFETY (delegated to caller): architected control-register write;
    // value 0 selects the single-write EOI mode that matches v2.
    unsafe {
        core::arch::asm!("msr S3_0_C12_C12_4, {v}", v = in(reg) value, options(nostack, nomem));
    }
}

/// `msr ICC_IGRPEN1_EL1, value` — `S3_0_C12_C12_7` (enable Group1 forwarding).
///
/// # Safety
/// Call last in the ICC init order, after SRE + an `isb`.
#[inline]
unsafe fn write_icc_igrpen1_el1(value: u64) {
    // SAFETY (delegated to caller): architected group-enable write.
    unsafe {
        core::arch::asm!("msr S3_0_C12_C12_7, {v}", v = in(reg) value, options(nostack, nomem));
    }
}

/// `mrs value, ICC_IAR1_EL1` — `S3_0_C12_C12_0` (acknowledge: read the pending
/// Group1 INTID). Replaces the `GICC_IAR` read on v2.
///
/// # Safety
/// Call from the IRQ path after the GICv3 CPU interface is enabled.
#[inline]
unsafe fn read_icc_iar1_el1() -> u64 {
    let v: u64;
    // SAFETY (delegated to caller): architected acknowledge read; the GIC
    // dequeues the highest-priority pending Group1 interrupt.
    unsafe {
        core::arch::asm!("mrs {v}, S3_0_C12_C12_0", v = out(reg) v, options(nostack, nomem));
    }
    v
}

/// `msr ICC_EOIR1_EL1, value` — `S3_0_C12_C12_1` (end-of-interrupt for the
/// INTID just acked). With `EOImode = 0` this both drops priority AND
/// deactivates the interrupt — the single-write retire that matches v2's single
/// `GICC_EOIR` write. Replaces the `GICC_EOIR` write on v2.
///
/// # Safety
/// `value` must be an INTID previously returned by [`read_icc_iar1_el1`].
#[inline]
unsafe fn write_icc_eoir1_el1(value: u64) {
    // SAFETY (delegated to caller): architected EOI write for a just-acked INTID.
    unsafe {
        core::arch::asm!("msr S3_0_C12_C12_1, {v}", v = in(reg) value, options(nostack, nomem));
    }
}

// ---------------------------------------------------------------------------
// GICv3 bring-up (the v3 analog of gic::init)
// ---------------------------------------------------------------------------

/// Initialize the GICv3 distributor, the boot CPU's redistributor, and the ICC
/// system-register CPU interface. The v3 analog of `gic::init` + the CPU-side of
/// `gic::enable_interrupt`. Distributor SPI routing is not needed: our only IRQ
/// is the timer PPI, delivered through the redistributor.
///
/// # Safety
/// Call once on the boot core during bring-up (after the MMU is up so the GICD/
/// GICR device frames are mapped), with IRQs masked.
pub unsafe fn init() {
    let d = gicd();

    // --- GICD: Affinity Routing BEFORE Group1 enable (ordering hazard). ---
    // Two separate writes: setting EnableGrp1 in the same write as ARE_NS can
    // latch the legacy (non-affinity) routing model mid-transition.
    d.ctlr.set(GICD_CTLR_ARE_NS);
    d.ctlr.set(GICD_CTLR_ARE_NS | GICD_CTLR_ENABLE_GRP1NS);

    // --- GICR: wake the boot CPU's redistributor. ---
    let rd = gicr_rd();
    // Clear ProcessorSleep, preserving the other bits.
    let waker = rd.waker.get() & !GICR_WAKER_PROCESSOR_SLEEP;
    rd.waker.set(waker);
    // Poll ChildrenAsleep -> 0 (bounded so a misprobe cannot hang boot).
    let mut guard = WAKER_POLL_GUARD;
    while (rd.waker.get() & GICR_WAKER_CHILDREN_ASLEEP) != 0 && guard > 0 {
        guard -= 1;
        core::hint::spin_loop();
    }

    // --- ICC: enable the system-register CPU interface (init order matters). ---
    // SRE first, then an `isb` so the sysreg interface is visible before any
    // other ICC write (else those writes fault: interface still disabled).
    // SAFETY: boot core, IRQs masked; enabling the GICv3 CPU interface.
    unsafe {
        write_icc_sre_el1(ICC_SRE_EL1_SRE);
        core::arch::asm!("isb", options(nostack, nomem));
        write_icc_pmr_el1(ICC_PMR_ACCEPT_ALL);
        write_icc_bpr1_el1(0);
        // EOImode = 0: a single EOIR1 does priority-drop + deactivate (matches
        // the v2 single GICC_EOIR write). EOImode = 1 would stop the timer.
        write_icc_ctlr_el1(0);
        write_icc_igrpen1_el1(ICC_IGRPEN1_EL1_ENABLE);
    }
}

/// Enable a Private Peripheral Interrupt (PPI, INTID 0..31 — e.g. the timer PPI
/// 30) on the boot CPU's redistributor and give it the same usable priority the
/// GICv2 path uses (`0xA0`). The v3 analog of `gic::enable_interrupt` for PPIs;
/// PPI/SGI config lives in the redistributor SGI_base frame, not the
/// distributor.
///
/// # Safety
/// Call after [`init`]. `intid` must be a valid PPI/SGI (`< 32`) for this board.
pub unsafe fn enable_ppi(intid: u32) {
    let sgi = gicr_sgi();
    let bit = intid % 32;

    // Group1 non-secure: set the bit in IGROUPR0, clear it in IGRPMODR0.
    sgi.igroupr0.set(sgi.igroupr0.get() | (1 << bit));
    sgi.igrpmodr0.set(sgi.igrpmodr0.get() & !(1 << bit));

    // Priority: byte-per-INTID, same 0xA0 mid-range value as the GICv2 path.
    let pri_reg = (intid / 4) as usize;
    let pri_shift = (intid % 4) * 8;
    let old = sgi.ipriorityr[pri_reg].get();
    let cleared = old & !(0xFFu32 << pri_shift);
    sgi.ipriorityr[pri_reg].set(cleared | (0xA0u32 << pri_shift));

    // Enable delivery.
    sgi.isenabler0.set(1 << bit);
}

/// Enable a PPI/SGI (`intid < 32`) on CPU `cpu_index`'s OWN redistributor
/// SGI_base frame (the per-AP analog of [`enable_ppi`], which targets the boot
/// CPU's redistributor). Used by the AP bring-up to enable the cross-CPU
/// TLB-shootdown SGI (P4·SMP·S4c).
///
/// # Safety
/// Call from CPU `cpu_index`'s own bring-up path after its redistributor is woken
/// ([`init_ap`]), with IRQs masked. `intid < 32`.
pub unsafe fn enable_ppi_ap(cpu_index: usize, intid: u32) {
    let sgi_base = GICR_RD_BASE + cpu_index * GICR_STRIDE + 0x1_0000;
    // SAFETY: `sgi_base` is this CPU's redistributor SGI frame inside the
    // identity-mapped device block; the AP is the sole accessor of its own RD.
    let sgi = unsafe { &*(sgi_base as *const GicrSgiRegs) };
    let bit = intid % 32;
    sgi.igroupr0.set(sgi.igroupr0.get() | (1 << bit));
    sgi.igrpmodr0.set(sgi.igrpmodr0.get() & !(1 << bit));
    let pri_reg = (intid / 4) as usize;
    let pri_shift = (intid % 4) * 8;
    let old = sgi.ipriorityr[pri_reg].get();
    let cleared = old & !(0xFFu32 << pri_shift);
    sgi.ipriorityr[pri_reg].set(cleared | (0xA0u32 << pri_shift));
    sgi.isenabler0.set(1 << bit);
}

// ---------------------------------------------------------------------------
// P4·SMP·S3 — per-AP redistributor + ICC bring-up (the per-CPU analog of init)
// ---------------------------------------------------------------------------

/// Wake CPU `cpu_index`'s OWN redistributor, enable a PPI on it, and enable the
/// ICC system-register CPU interface on THIS AP.
///
/// The distributor (GICD) is shared and already brought up by [`init`] on the
/// BSP, so an AP does NOT touch GICD — it only brings up its **per-CPU** state:
///   * its redistributor at `GICR_RD_BASE + cpu_index * GICR_STRIDE` (RD + SGI
///     frames), woken via `GICR_WAKER` and configured for the timer PPI in its
///     own SGI_base frame, and
///   * the ICC CPU interface, which is a set of **per-CPU banked** system
///     registers (`ICC_SRE/PMR/BPR1/CTLR/IGRPEN1_EL1`) each AP must program
///     itself (the BSP cannot do it on the AP's behalf).
///
/// For S3 the timer PPI is enabled (so the AP *could* receive it) but the AP's
/// generic timer is left masked — it does no scheduling and takes no IRQ.
///
/// # Safety
/// Call once, from the AP's own `_ap_start` path, after its MMU is enabled (so
/// the GICR device frame is mapped) and with IRQs masked. `cpu_index` must be
/// this AP's logical id; `intid` a valid PPI/SGI (`< 32`).
pub unsafe fn init_ap(cpu_index: usize, intid: u32) {
    let rd_base = GICR_RD_BASE + cpu_index * GICR_STRIDE;
    let sgi_base = rd_base + 0x1_0000;

    // --- Wake this AP's redistributor (clear GICR_WAKER.ProcessorSleep). ---
    // SAFETY: `rd_base` is this CPU's RD frame inside the identity-mapped device
    // block; the AP is the sole accessor of its own redistributor.
    let rd = unsafe { &*(rd_base as *const GicrRdRegs) };
    let waker = rd.waker.get() & !GICR_WAKER_PROCESSOR_SLEEP;
    rd.waker.set(waker);
    let mut guard = WAKER_POLL_GUARD;
    while (rd.waker.get() & GICR_WAKER_CHILDREN_ASLEEP) != 0 && guard > 0 {
        guard -= 1;
        core::hint::spin_loop();
    }

    // --- Enable the timer PPI in this AP's SGI_base frame (same as enable_ppi,
    //     but on the per-CPU redistributor rather than the fixed boot one). ---
    // SAFETY: `sgi_base` = this CPU's RD + 64 KiB SGI frame.
    let sgi = unsafe { &*(sgi_base as *const GicrSgiRegs) };
    let bit = intid % 32;
    sgi.igroupr0.set(sgi.igroupr0.get() | (1 << bit));
    sgi.igrpmodr0.set(sgi.igrpmodr0.get() & !(1 << bit));
    let pri_reg = (intid / 4) as usize;
    let pri_shift = (intid % 4) * 8;
    let old = sgi.ipriorityr[pri_reg].get();
    let cleared = old & !(0xFFu32 << pri_shift);
    sgi.ipriorityr[pri_reg].set(cleared | (0xA0u32 << pri_shift));
    sgi.isenabler0.set(1 << bit);

    // --- ICC CPU interface (per-CPU banked sysregs; SRE first, then isb). ---
    // SAFETY: this AP, IRQs masked; enabling its own GICv3 CPU interface in the
    // exact order init() uses on the BSP.
    unsafe {
        write_icc_sre_el1(ICC_SRE_EL1_SRE);
        core::arch::asm!("isb", options(nostack, nomem));
        write_icc_pmr_el1(ICC_PMR_ACCEPT_ALL);
        write_icc_bpr1_el1(0);
        write_icc_ctlr_el1(0);
        write_icc_igrpen1_el1(ICC_IGRPEN1_EL1_ENABLE);
    }
}

// ---------------------------------------------------------------------------
// Per-IRQ ack/dispatch/EOI (the v3 analog of gic::handle_irq)
// ---------------------------------------------------------------------------

/// Handle one IRQ exception on the GICv3 leg: ack via `ICC_IAR1_EL1`, dispatch
/// (timer -> [`crate::timer`]), then EOI via `ICC_EOIR1_EL1`. Mirrors
/// `gic::handle_irq` exactly except for the ack/EOI register source — the same
/// INTID, the same `>= 1020` spurious check, the same `on_tick` + `TICKS`
/// dispatch (the shared `gic::TICKS` counter, so the HAL view is unchanged).
/// This leg NEVER touches the `GICC_*` MMIO (which does not exist on a v3 board).
pub fn handle_irq() -> bool {
    // SAFETY: the GICv3 CPU interface was enabled in `init`; reading IAR1
    // dequeues the pending Group1 INTID.
    let iar = unsafe { read_icc_iar1_el1() };
    let intid = (iar & 0xFFFFFF) as u32;

    if intid >= SPURIOUS_MIN {
        return false; // Spurious; nothing to retire, not a preemption tick.
    }

    let mut was_tick = false;
    if intid == crate::gic::TIMER_INTID {
        crate::timer::on_tick();
        crate::gic::TICKS.fetch_add(1, Ordering::Relaxed);
        was_tick = true;
    } else if intid == crate::shootdown::SHOOTDOWN_SGI {
        // P4·SMP·S4c: a cross-CPU TLB shootdown request. Invalidate this CPU's
        // stale translations + ack the sender (the H1 receiver step). This is
        // NOT a preemption tick — the receiver must resume EXACTLY the EL0
        // context it interrupted (returning `false` so the caller does not
        // reschedule), else two CPUs would reschedule the same process around a
        // shootdown and cross their saved contexts.
        crate::shootdown::service_on_sgi();
    } else if intid == crate::reschedule::RESCHED_SGI {
        // P4·SMP·S4b: a reschedule wake. Nothing to do in the handler — taking
        // the SGI already pulled an idle AP out of `wfe`, so it falls back into
        // `ap_run_scheduler` and re-pops/steals. NOT a preemption tick: a busy
        // CPU that took the SGI must resume its current EL0 context unchanged
        // (returning `false`); the idle CPU re-runs schedule in its own loop.
    }

    // End-of-interrupt: EOImode=0 => this single write drops priority AND
    // deactivates, matching the v2 single GICC_EOIR write.
    // SAFETY: `iar` is the INTID just returned by IAR1.
    unsafe { write_icc_eoir1_el1(iar) };
    return was_tick;
}

/// Send a Software Generated Interrupt via `ICC_SGI1R_EL1` (`S3_0_C12_C11_5`).
///
/// The cross-CPU IPI primitive on the GICv3 leg. As of P4·SMP·S4c it carries the
/// TLB-shootdown SGI (INTID 0) to the target CPUs (`crate::shootdown`).
///
/// # Safety
/// `value` must be a well-formed `ICC_SGI1R_EL1` payload (affinity + target
/// list + INTID).
pub unsafe fn send_sgi(value: u64) {
    // SAFETY (delegated to caller): architected SGI-generate write.
    unsafe {
        core::arch::asm!("msr S3_0_C12_C11_5, {v}", v = in(reg) value, options(nostack, nomem));
    }
}
