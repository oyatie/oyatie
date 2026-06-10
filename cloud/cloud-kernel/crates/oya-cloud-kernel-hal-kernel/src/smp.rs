//! SMP / AP bring-up: the 0-unsafe seam (P4·SMP·S3).
//!
//! This is the narrow, *safe* contract the kernel uses to bring secondary CPUs
//! (APs) online and observe their progress. Like every `hal` module it is
//! `#![forbid(unsafe_code)]` (inherited from the crate): only trait signatures
//! and plain types live here. **All** of the dangerous AP bring-up — the x86
//! `.code16` SIPI trampoline + INIT-SIPI-SIPI via the x2APIC ICR MSR, and the
//! aarch64 PSCI `CPU_ON` + `_ap_start` MMU/GICR/ICC programming — lives in the
//! arch Frame behind this seam, so `check-tcb.sh`'s forbid-set
//! (`kernel hal frame user_layout`) stays unsafe-free.
//!
//! The discipline this seam encodes (the publish/observe pairing the S2 Loom
//! gate proved as H1): each AP, after its **full** per-CPU init, `Release`-sets
//! its bit in a shared online mask; the BSP `Acquire`-reads that mask, so when
//! it observes bit `k` it also observes CPU `k`'s completed init. [`online_mask`]
//! exposes that observation without any `unsafe`.
//!
//! Scope fence (S3): APs come online, set their per-CPU anchor + stack + local
//! IRQ chip, publish their online bit, then **idle** (`hlt`/`wfe`). They do NOT
//! run the scheduler and never touch the (still single-owner) `ProcTable` — that
//! is S4.

use crate::sealed::Sealed;
use crate::ArchError;

/// The Rust entry an AP tail-calls once its per-CPU anchor + stack are live.
///
/// Diverging (`-> !`): an S3 AP never returns — it publishes its online bit and
/// idles. The trait carries this to document the seam even though, in practice,
/// each arch Frame supplies its **own** asm prologue (the `.code16` trampoline /
/// `_ap_start`) and only tail-calls a Frame-internal idle; the safe kernel passes
/// a tiny diverging `extern "C"` fn (e.g. `ap_idle`) so the seam is fully typed.
pub type ApEntry = extern "C" fn() -> !;

/// Bring secondary CPUs online and observe their progress.
///
/// Implemented by the per-arch backend structs (`X86_64` / `Aarch64`), both of
/// which are already [`Sealed`] (they implement [`crate::Arch`]). Sealing keeps
/// the trait implementable only by this workspace's Frame backends.
pub trait Smp: Sealed {
    /// How many logical CPUs the platform enumerated (≥ 1).
    ///
    /// Mirror of [`crate::cpu::CpuCaps::cpu_count`]; the single gate that decides
    /// whether **any** AP code runs. On the default 1-vCPU image this is `1`, so
    /// [`start_secondaries`](Smp::start_secondaries) is a silent no-op and the
    /// boot serial stream is byte-identical.
    fn cpu_count(&self) -> u32;

    /// Bring every secondary CPU online into `entry`, returning the number of
    /// CPUs now online **including the BSP** (so `== cpu_count()` on full
    /// success). Returns `1` immediately, emitting nothing, when
    /// `cpu_count() == 1` (the 1-vCPU image is untouched).
    fn start_secondaries(&mut self, entry: ApEntry) -> Result<u32, ArchError>;

    /// Bitmask of CPUs that have published their online bit (BSP = bit 0).
    ///
    /// An `Acquire` read of the shared online mask the APs `Release`-set, so a
    /// set bit `k` carries CPU `k`'s completed per-CPU init.
    fn online_mask(&self) -> u64;
}
