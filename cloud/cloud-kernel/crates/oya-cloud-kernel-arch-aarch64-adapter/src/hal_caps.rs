//! Floor backings for the reshaped P0 HAL capability surface, expressed against
//! the **existing, working** aarch64 K3 hardware path.
//!
//! # What this module is (and is not)
//!
//! This is the *additive* half of the P0 HAL reshape for aarch64: it re-expresses
//! the backend's already-booting hardware code (GICv2, the EL1 generic timer,
//! TTBR0 switching, the EL0/trap frame, the `ID_AA64*` feature registers) as the
//! new capability *shapes* from `hal::{cpu,mm,irq,time}`. Nothing here is on the
//! boot path — the kernel still drives the original [`crate::Aarch64`] /
//! `Arch`/`ConsoleWrite`/`MemoryApi`/`InterruptApi`/`TimerApi` impls unchanged.
//! These items exist *alongside* them, proving the new boundary fits the hardware
//! we already run, with byte-identical boot + golden trace (the boot path is
//! untouched).
//!
//! # The sealing finding (resolved — these are now real trait impls)
//!
//! Most of the new capability traits — [`hal::time::Clock`], [`hal::time::Timer`],
//! [`hal::irq::IrqChip`], [`hal::mm::Pte`], [`hal::mm::PageTableConfig`],
//! [`hal::cpu::TrapFrame`], [`hal::cpu::UserContext`] — are **sealed** via
//! `hal::sealed::Sealed`. A prior increment found that with `sealed` declared
//! `mod sealed;` (private) in `hal`'s crate root, an arch backend **cannot name
//! the supertrait** and therefore **cannot write the `impl`**
//! (`error[E0603]: module 'sealed' is private`).
//!
//! That seal is now opened *to this workspace only*: `hal` declares
//! `#[doc(hidden)] pub mod sealed;`, the standard "sealed-to-downstream,
//! open-to-this-workspace, hidden-from-docs" idiom. The backend can now name
//! `hal::sealed::Sealed`, so the floor-backing *logic* that previously lived as
//! inherent methods is now expressed as the genuine `impl hal::sealed::Sealed`
//! and `impl hal::<trait>` blocks below — same bodies, real traits. The seal
//! still holds against the external ecosystem (the supertrait is hidden from
//! docs and intended only for this workspace's Frame backends).
//!
//! [`hal::mm::PagingConsts`] was already unsealed, so [`Aarch64PagingConsts`]
//! below was always a real trait impl. The [`hal::cpu::CpuCaps`] snapshot is a
//! struct, and `hal` now exposes [`CpuCaps::new`] (the `#[non_exhaustive]`
//! constructor seam), so [`probe_cpu_caps`] feeds a real, populated `CpuCaps`
//! via [`detected_cpu_caps`].
//!
//! Still **deferred / unimplemented**: [`hal::cpu::UserContext`] (its `run()`
//! half needs the EL0/`eret` boot-path rewire = the switchover slice, which this
//! boot-path-preserving increment must not touch — its register accessors are
//! noted below), and the virtio / confidential / DMA seams (P5/P6).

#![allow(dead_code)] // Additive: not yet called by the kernel's boot path.

use aarch64_cpu::registers::{
    CNTFRQ_EL0, CNTPCT_EL0, CNTP_CTL_EL0, CNTP_CVAL_EL0, ID_AA64ISAR0_EL1, ID_AA64MMFR0_EL1,
    ID_AA64MMFR1_EL1, MPIDR_EL1,
};
use tock_registers::interfaces::{Readable, Writeable};

use hal::cpu::CpuCaps;
use hal::irq::{CpuId, IrqChip, IrqVector, MsiMessage};
use hal::mm::{AsidTag, GenericPteFlags, PageTableConfig, PagingConsts, PhysAddr, Pte};
use hal::sealed::Sealed;
use hal::time::{Clock, Timer};
use hal::ArchError;

// ===========================================================================
// PagingConsts  —  UNSEALED, so this is a genuine trait impl.
// ===========================================================================

/// aarch64 translation regime: VMSAv8-64, 4 KiB granule, 4 levels, 48-bit VA.
///
/// This is the floor regime — the standard 4 KiB/4-level/48-bit configuration
/// QEMU `virt` and every server-class aarch64 part support. (The boot MMU in
/// [`crate::mmu`] uses a smaller 32-bit-VA identity map for the bring-up demo;
/// these consts describe the full regime the generic page-table engine in
/// `frame` will target, not that minimal boot table.)
pub struct Aarch64PagingConsts;

impl PagingConsts for Aarch64PagingConsts {
    /// L0..L3 (4 KiB granule, 48-bit VA needs 4 translation levels).
    const NR_LEVELS: u8 = 4;
    /// 4 KiB base page.
    const BASE_PAGE_SIZE: usize = 4096;
    /// 48-bit virtual addresses (TnSZ = 16).
    const ADDRESS_WIDTH: u8 = 48;
    /// aarch64 VAs sign-extend: TTBR0 region is low, TTBR1 region is high.
    const VA_SIGN_EXT: bool = true;
    /// Highest level that may hold a leaf/block: L1 (1 GiB block descriptor).
    /// L0 blocks are not architecturally permitted at the 4 KiB granule.
    const HIGHEST_TRANSLATION_LEVEL: u8 = 1;
}

// ===========================================================================
// CpuCaps  —  a struct to populate (not a trait), so this is real and callable.
// ===========================================================================

// ---- ID_AA64ISAR0_EL1 ------------------------------------------------------
/// `ID_AA64ISAR0_EL1.Atomic` field: bits [23:20]. `>= 0b0010` ⇒ LSE atomics.
const ISAR0_ATOMIC_SHIFT: u64 = 20;
const ISAR0_ATOMIC_MASK: u64 = 0xF;

// ---- ID_AA64MMFR0_EL1 ------------------------------------------------------
/// `ASIDBits` field: bits [7:4]. `0b0010` ⇒ 16-bit ASIDs (tagged TLB usable).
const MMFR0_ASIDBITS_SHIFT: u64 = 4;
const MMFR0_ASIDBITS_MASK: u64 = 0xF;
const MMFR0_ASIDBITS_16: u64 = 0b0010;
/// `TGran4` field: bits [31:28]. `0b1111` ⇒ 4 KiB granule NOT supported.
const MMFR0_TGRAN4_SHIFT: u64 = 28;
const MMFR0_TGRAN4_MASK: u64 = 0xF;
const MMFR0_TGRAN4_UNSUPPORTED: u64 = 0b1111;

// ---- ID_AA64MMFR1_EL1 ------------------------------------------------------
/// `PAN` field: bits [23:20]. Non-zero ⇒ Privileged-Access-Never present.
const MMFR1_PAN_SHIFT: u64 = 20;
const MMFR1_PAN_MASK: u64 = 0xF;

// ---- MPIDR_EL1 -------------------------------------------------------------
/// `MT` bit [24]: lowest affinity level is multithreaded (Aff0 = thread).
const MPIDR_MT_BIT: u64 = 1 << 24;

/// The capability bits this backend can positively detect today from the
/// aarch64 ID registers. Arch-local mirror of the subset of [`hal::cpu::CpuCaps`]
/// we can populate — see [`probe_cpu_caps`] for why we cannot return a real
/// `CpuCaps` yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DetectedCaps {
    /// `ID_AA64ISAR0_EL1.Atomic >= 0b0010` ⇒ LSE single-instruction atomics.
    pub lse_atomics: bool,
    /// `ID_AA64MMFR0_EL1.ASIDBits == 0b0010` ⇒ 16-bit ASIDs ⇒ taggable TLB.
    pub tagged_tlb: bool,
    /// `ID_AA64MMFR1_EL1.PAN != 0` ⇒ Privileged-Access-Never present.
    pub user_access_protection: bool,
    /// `ID_AA64MMFR0_EL1.TGran4 != 0b1111` ⇒ 4 KiB granule supported.
    pub granule_4k: bool,
}

/// Probe the immutable boot-time CPU capabilities from the aarch64 ID registers
/// (`ID_AA64ISAR0/MMFR0/MMFR1`).
///
/// Best-effort and conservative: capabilities we cannot yet positively detect
/// (paravirt signature, ECV deadline timer, MTE, PAC+BTI, confidential platform)
/// are simply absent from [`DetectedCaps`] and stay at their safe-fallback value
/// in the eventual `CpuCaps` (consensus C9). Pure reads of read-only ID registers
/// — no hardware state changes, so this is safe to call (it is not on the
/// unchanged boot path either).
///
/// **Second sealing-style finding (resolved):** the floor wants a populated
/// [`hal::cpu::CpuCaps`], but `CpuCaps` is `#[non_exhaustive]`, so an arch backend
/// physically cannot mint one with a struct literal **even with functional-update**
/// (`..base`). `hal` now exposes [`hal::cpu::CpuCaps::new`] (the `#[non_exhaustive]`
/// constructor seam), so [`detected_cpu_caps`] folds these detected bits onto the
/// conservative [`hal::cpu::CpuCaps::FALLBACK`] floor and returns a real `CpuCaps`.
pub fn probe_cpu_caps() -> DetectedCaps {
    let isar0 = ID_AA64ISAR0_EL1.get();
    let mmfr0 = ID_AA64MMFR0_EL1.get();
    let mmfr1 = ID_AA64MMFR1_EL1.get();

    let lse_atomics = ((isar0 >> ISAR0_ATOMIC_SHIFT) & ISAR0_ATOMIC_MASK) >= 0b0010;

    let asid_bits = (mmfr0 >> MMFR0_ASIDBITS_SHIFT) & MMFR0_ASIDBITS_MASK;
    let tagged_tlb = asid_bits >= MMFR0_ASIDBITS_16;

    let tgran4 = (mmfr0 >> MMFR0_TGRAN4_SHIFT) & MMFR0_TGRAN4_MASK;
    let granule_4k = tgran4 != MMFR0_TGRAN4_UNSUPPORTED;

    let pan = (mmfr1 >> MMFR1_PAN_SHIFT) & MMFR1_PAN_MASK;
    let user_access_protection = pan != 0;

    DetectedCaps {
        lse_atomics,
        tagged_tlb,
        user_access_protection,
        granule_4k,
    }
}

/// The conservative all-fallback [`hal::cpu::CpuCaps`] snapshot.
///
/// The floor any arch backend can always produce; [`detected_cpu_caps`] overlays
/// positively-probed bits on top of it.
pub fn fallback_cpu_caps() -> CpuCaps {
    CpuCaps::FALLBACK
}

/// A real, populated [`hal::cpu::CpuCaps`] for this aarch64 boot core.
///
/// Folds the positively-detected bits from [`probe_cpu_caps`] onto the
/// conservative [`hal::cpu::CpuCaps::FALLBACK`] floor via the new
/// [`hal::cpu::CpuCaps::new`] constructor. Capabilities this backend cannot yet
/// positively detect (`paravirt_guest`, `deadline_timer` / ECV, `memory_tagging`
/// / MTE, `control_flow_integrity` / PAC+BTI, `confidential`) stay at their safe
/// fallback `false` (consensus C9): a feature-poor VM simply takes the safe path.
/// `cpu_count` is the single boot core (1) until SMP bring-up lands.
///
/// Pure reads of read-only ID registers — not on the unchanged boot path.
pub fn detected_cpu_caps() -> CpuCaps {
    let d = probe_cpu_caps();
    CpuCaps::new(
        1,                       // cpu_count: single boot core until SMP (P4).
        false,                   // paravirt_guest: not yet probed.
        d.tagged_tlb,            // ID_AA64MMFR0_EL1.ASIDBits >= 16-bit.
        false,                   // deadline_timer: ECV not yet probed.
        d.user_access_protection, // ID_AA64MMFR1_EL1.PAN != 0.
        d.lse_atomics,           // ID_AA64ISAR0_EL1.Atomic >= 0b0010.
        false,                   // memory_tagging: MTE not yet probed.
        false,                   // control_flow_integrity: PAC+BTI not yet probed.
        false,                   // confidential: CCA not yet probed.
    )
}

/// Whether the boot core reports a multithreaded lowest affinity level.
///
/// Small helper exercising `MPIDR_EL1`; folded into a real per-CPU topology
/// probe when SMP bring-up lands.
pub fn boot_core_is_multithreaded() -> bool {
    (MPIDR_EL1.get() & MPIDR_MT_BIT) != 0
}

// ===========================================================================
// Clock + Timer  (hal::time)  —  SEALED. Floor logic as inherent methods.
// ===========================================================================

/// Floor backing for [`hal::time::Clock`] + [`hal::time::Timer`] over the EL1
/// physical generic timer (`CNTPCT_EL0` / `CNTFRQ_EL0` / `CNTP_CVAL_EL0`).
///
/// Re-expresses the existing [`crate::timer`] hardware path as the deadline
/// shape: `now_ns` converts the monotonic counter to nanoseconds via the counter
/// frequency, and `set_deadline_ns` programs the architectural compare register
/// `CNTP_CVAL_EL0` directly (the true one-shot — strictly better than the
/// periodic re-arm the boot demo uses, and what the deadline shape wants). On a
/// part without ECV this is still a correct one-shot; ECV only changes *when* the
/// compare is sampled, not this code.
///
/// Real `impl hal::time::Clock` + `impl hal::time::Timer`, now that the `Sealed`
/// supertrait is reachable from this workspace (see module docs). Each trait-method
/// body is unchanged from the prior floor logic.
pub struct Aarch64Timer;

impl Aarch64Timer {
    /// Counter frequency in Hz (`CNTFRQ_EL0`). Set by firmware; on QEMU `virt`
    /// this is 62.5 MHz.
    fn frequency_hz() -> u64 {
        CNTFRQ_EL0.get()
    }

    /// Convert an absolute nanosecond instant to an absolute counter value.
    fn ns_to_ticks(ns: u64) -> u64 {
        let freq = Self::frequency_hz() as u128;
        ((ns as u128 * freq) / 1_000_000_000u128) as u64
    }
}

impl Sealed for Aarch64Timer {}

impl Clock for Aarch64Timer {
    /// Monotonic nanoseconds.
    ///
    /// `ns = ticks * 1e9 / freq`. Uses `u128` intermediate so the multiply does
    /// not overflow before the divide.
    fn now_ns(&self) -> u64 {
        let ticks = CNTPCT_EL0.get() as u128;
        let freq = Self::frequency_hz() as u128;
        if freq == 0 {
            return 0;
        }
        ((ticks * 1_000_000_000u128) / freq) as u64
    }
}

impl Timer for Aarch64Timer {
    /// Arm a one-shot for the absolute monotonic instant `deadline_ns`.
    ///
    /// Programs `CNTP_CVAL_EL0` (the architectural compare value) and enables the
    /// timer with its interrupt unmasked. A deadline already in the past makes
    /// the comparator fire immediately, satisfying "fires promptly".
    fn set_deadline_ns(&mut self, deadline_ns: u64) -> Result<(), ArchError> {
        if Self::frequency_hz() == 0 {
            return Err(ArchError::Interrupt);
        }
        let cval = Self::ns_to_ticks(deadline_ns);
        CNTP_CVAL_EL0.set(cval);
        CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE::SET + CNTP_CTL_EL0::IMASK::CLEAR);
        Ok(())
    }

    /// Disarm any pending deadline.
    ///
    /// Idempotent — masks + disables the comparator whether or not one was armed.
    fn cancel(&mut self) {
        CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE::CLEAR + CNTP_CTL_EL0::IMASK::SET);
    }
}

// ===========================================================================
// IrqChip  (hal::irq)  —  SEALED. Floor logic over the existing GICv2.
// ===========================================================================

/// Floor backing for [`hal::irq::IrqChip`] over the existing GICv2
/// (`enable`/`disable`/`eoi` reuse [`crate::gic`]).
///
/// GICv2 has no architected per-CPU IPI mechanism we use in this bring-up and no
/// MSI/ITS, so `send_ipi`/`map_msi` are the documented `Unsupported` floor — the
/// GICv3 redistributor + ITS path is the P4/P5 fast path. `enable`/`disable`/`eoi`
/// are the real GICv2 bodies, re-expressed onto the [`hal::irq::IrqVector`] shape.
///
/// Real `impl hal::irq::IrqChip`, now that `Sealed` is reachable. `enable`/`eoi`
/// are the real GICv2 bodies; `disable`/`send_ipi`/`map_msi` are the documented
/// `Unsupported` floor (the GICv3 redistributor + ITS path is P4/P5).
pub struct Gicv2IrqChip;

impl Sealed for Gicv2IrqChip {}

impl IrqChip for Gicv2IrqChip {
    /// Enable delivery of `vector`.
    ///
    /// Re-expresses `crate::gic::enable_interrupt`. That routine is `unsafe`
    /// (raw MMIO); we keep the unsafe contained here behind a safe Frame seam.
    fn enable(&mut self, vector: IrqVector) -> Result<(), ArchError> {
        // SAFETY: post-`gic::init` on the boot core; `vector.0` is a board INTID.
        // This mirrors the existing, audited bring-up call exactly.
        unsafe {
            crate::gic::enable_interrupt(vector.0);
        }
        Ok(())
    }

    /// GICv2 disable is a `GICD_ICENABLER` write; the current bring-up never
    /// disables a line (it only powers off), so the masking primitive is not yet
    /// in [`crate::gic`]. Reported as `Unsupported` at the floor rather than
    /// faking it — the honest state until the ICENABLER write lands.
    fn disable(&mut self, _vector: IrqVector) -> Result<(), ArchError> {
        Err(ArchError::Unsupported)
    }

    /// End-of-interrupt.
    ///
    /// The existing IRQ path retires via `GICC_EOIR` inside
    /// [`crate::gic::handle_irq`] (it writes back the IAR value it read). At the
    /// floor we expose the same EOIR write for the in-service vector.
    fn eoi(&mut self, vector: IrqVector) {
        crate::gic::eoi(vector.0);
    }

    /// GICv2 SGIs exist (`GICD_SGIR`) but no IPI is used in single-core bring-up;
    /// SMP IPIs are the P4 SMP slice. Documented `Unsupported` floor.
    fn send_ipi(&mut self, _target: CpuId, _vector: IrqVector) -> Result<(), ArchError> {
        Err(ArchError::Unsupported)
    }

    /// GICv2 has no MSI/ITS; virtio on aarch64 here is mmio+SPI, not PCI+MSI-X.
    /// MSI mapping is the GICv3 ITS fast path (P5). Documented `Unsupported`.
    fn map_msi(&mut self, _target: CpuId, _vector: IrqVector) -> Result<MsiMessage, ArchError> {
        Err(ArchError::Unsupported)
    }
}

// ===========================================================================
// Pte + PageTableConfig  (hal::mm)  —  SEALED. Floor logic over VMSAv8 + TTBR0.
// ===========================================================================

/// A VMSAv8-64 page-table entry, presented as the [`hal::mm::Pte`] shape.
///
/// Wraps the raw 64-bit descriptor the existing tables in [`crate::mmu`] /
/// `crate::process` use, decoding it through the *same* `user_layout` descriptor
/// bit constants that build it (single source of truth — no second bit layout).
///
/// Real `impl hal::mm::Pte`, now that `Sealed` is reachable. `Pte: Sealed + Copy`
/// — this type already derives `Copy`. The method bodies are unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Aarch64Pte(pub u64);

impl Sealed for Aarch64Pte {}

impl Pte for Aarch64Pte {
    /// Decode typed flags from the VMSAv8 descriptor using `user_layout`'s shared
    /// bit constants.
    fn flags(&self) -> GenericPteFlags {
        use crate::user_layout::{
            DESC_AP_EL0_RO, DESC_AP_EL0_RW, DESC_UXN, DESC_VALID,
        };
        let d = self.0;
        let present = (d & DESC_VALID) != 0;
        let ap = d & (0b11 << 6);
        // EL0-accessible if AP encodes an EL0 permission (RW or RO).
        let user = ap == DESC_AP_EL0_RW || ap == DESC_AP_EL0_RO;
        let writable = ap == DESC_AP_EL0_RW;
        // Executable at the relevant EL when the matching XN bit is clear. For a
        // user page that is UXN; we report executability from the user-visible
        // (UXN) bit, matching `PagePerm::from_desc`'s convention.
        let executable = (d & DESC_UXN) == 0;
        // nG bit [11] set => non-global (per-ASID). Global when clear.
        let global = (d & (1 << 11)) == 0;
        GenericPteFlags {
            present,
            writable,
            executable,
            user,
            global,
        }
    }

    /// The output physical address.
    fn address(&self) -> PhysAddr {
        use crate::user_layout::DESC_ADDR_MASK;
        PhysAddr((self.0 & DESC_ADDR_MASK) as usize)
    }

    /// Does this entry terminate the walk? At L3 bit[1]=1 marks a page (leaf); a
    /// block descriptor (bit[1]=0 at L1/L2) is also a leaf. Only an L0..L2 *table*
    /// descriptor (bit[1]=1 above L3) is non-leaf — the level is not encoded in
    /// the word, so the engine supplies it. At the floor we treat a valid
    /// non-table-shaped entry as leaf; the level-aware refinement lands with the
    /// generic cursor in `frame`.
    fn is_leaf(&self) -> bool {
        use crate::user_layout::DESC_VALID;
        // A block descriptor (bit[1]==0) is always a leaf. Page-vs-table for
        // bit[1]==1 is level-dependent; the `frame` cursor disambiguates.
        (self.0 & DESC_VALID) != 0 && (self.0 & (1 << 1)) == 0
    }
}

/// Floor backing for [`hal::mm::PageTableConfig`] over `TTBR0_EL1`.
///
/// `switch_to` re-expresses the existing TTBR0 switch used throughout
/// `crate::user`/`crate::process` (set `TTBR0_EL1`, then a *full* TLB flush
/// `tlbi vmalle1`). Per the floor spec the [`hal::mm::AsidTag`] is ignored and we
/// always full-flush — exactly today's behavior; ASID-tagged switching (suppress
/// the flush, program the tag) is the P4 fast path and a signature-compatible
/// swap-in (the opaque tag is already in the signature).
///
/// Real `impl hal::mm::PageTableConfig`, now that `Sealed` is reachable. Its
/// associated types are [`Aarch64Pte`] / [`Aarch64PagingConsts`]; `switch_to`
/// carries the existing TTBR0-switch body.
pub struct Aarch64PageTableConfig;

impl Sealed for Aarch64PageTableConfig {}

impl PageTableConfig for Aarch64PageTableConfig {
    type Pte = Aarch64Pte;
    type Consts = Aarch64PagingConsts;

    /// Activate `root` with a full TLB flush, ignoring `asid_tag` (full-flush
    /// floor — exactly today's behavior; ASID-tagged switching is the P4 fast
    /// path and a signature-compatible swap-in, the opaque tag already being in
    /// the signature).
    ///
    /// The `hal` trait declares this method **safe**; the hardware writes it
    /// performs (set `TTBR0_EL1`, full `tlbi vmalle1`) carry the same invariant
    /// the existing `crate::user`/`crate::process` switch upholds: `root` must be
    /// the physical base of a valid L0/L1 translation table whose upper entries
    /// keep the kernel identity map live (every `crate::process::AddressSpace`
    /// guarantees this). The unsafe register/TLB ops are contained here behind
    /// the safe Frame seam, mirroring the audited switch in `user.rs`.
    fn switch_to(&self, root: PhysAddr, _asid_tag: AsidTag) {
        use aarch64_cpu::asm::barrier;
        use aarch64_cpu::registers::TTBR0_EL1;
        // SAFETY (contract above): identical to the audited switch in `user.rs`.
        TTBR0_EL1.set(root.as_usize() as u64);
        barrier::dsb(barrier::SY);
        // SAFETY: full inner-shareable TLB invalidate for this VMID, exactly the
        // existing process-switch flush. `tlbi vmalle1` + dsb + isb.
        unsafe {
            core::arch::asm!("tlbi vmalle1", "dsb sy", "isb", options(nostack));
        }
    }
}

// ===========================================================================
// TrapFrame  (hal::cpu)  —  SEALED. Floor accessors over the existing frame.
// ===========================================================================

/// Floor backing for [`hal::cpu::TrapFrame`] over the **existing**
/// [`crate::exceptions::TrapFrame`] (the `repr(C)` frame the vector stubs push).
///
/// These are read-only accessors over the already-saved frame — implementing
/// them requires **no** change to the entry/exit assembly or the boot path, so
/// they are safe to add now. The privileged `spsr` (DAIF/EL bits) is
/// deliberately *not* exposed, matching the trait's "hide privileged flags"
/// contract (lesson A18).
///
/// Real `impl hal::cpu::TrapFrame`, now that `Sealed` is reachable. Read-only
/// accessors over the already-saved frame — no change to the entry/exit assembly
/// or boot path. The privileged `spsr` (DAIF/EL bits) is deliberately *not*
/// exposed, matching the trait's "hide privileged flags" contract (lesson A18).
///
/// [`hal::cpu::UserContext`] remains **deferred**: its `run()` half (enter EL0 /
/// `eret` / return-reason loop) would require rewiring the existing `enter_el0` /
/// `handle_svc` / `on_timer_preempt` boot path in `user.rs`, which this additive,
/// boot-path-preserving increment must not touch. `UserContext` lands in the
/// switchover slice; its register accessors would be the same `elr`/`sp`/`regs[0]`
/// pattern shown below.
pub struct TrapFrameView<'a>(pub &'a crate::exceptions::TrapFrame);

impl Sealed for TrapFrameView<'_> {}

impl hal::cpu::TrapFrame for TrapFrameView<'_> {
    /// `ELR_EL1`.
    fn instruction_pointer(&self) -> usize {
        self.0.elr as usize
    }

    /// The saved SP slot.
    fn stack_pointer(&self) -> usize {
        self.0.sp as usize
    }

    /// The existing frame does not store the vector `kind` (it is passed to
    /// `rust_trap` as a separate argument, not saved in the frame), so at the
    /// floor we derive the architectural trap class from `ESR_EL1`'s EC field —
    /// the closest frame-free "what caused entry" the existing path exposes.
    fn trap_number(&self) -> usize {
        use aarch64_cpu::registers::ESR_EL1;
        ((ESR_EL1.get() >> 26) & 0x3f) as usize
    }

    /// aarch64 carries fault detail in `ESR_EL1.ISS` rather than a pushed error
    /// code; expose the ISS field as the arch-neutral "error code".
    fn error_code(&self) -> usize {
        use aarch64_cpu::registers::ESR_EL1;
        (ESR_EL1.get() & 0x01ff_ffff) as usize
    }
}
