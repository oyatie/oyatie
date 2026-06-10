//! Per-CPU anchor for the aarch64 Frame, built on `TPIDR_EL1`.
//!
//! The HAL's [`hal::cpu::PerCpu<T>`] holds a `[T; MAX_CPUS]` array indexed by
//! the running CPU's logical id; this module is the arch backend that supplies
//! that index by minting a [`hal::cpu::CpuToken`]. The id lives in `TPIDR_EL1`
//! — a thread-pointer register the OS owns at EL1, free for us because user TLS
//! uses `TPIDR_EL0` (`process.rs`), and which survives EL0↔EL1 transitions
//! because EL0 never touches it. So it is the correct per-CPU anchor.
//!
//! ## The only new aarch64 `unsafe` for S1
//!
//! Two register touches: writing the index at boot ([`init_bsp`]) and reading
//! it to mint a token ([`this_cpu_token`]). Both are single `msr`/`mrs`
//! instructions, the sole new `unsafe` this slice adds on aarch64, and they
//! live here in the Frame (TCB) — never the `#![forbid(unsafe_code)]` safe
//! kernel surface. The HAL `PerCpu`/`CpuToken` the safe code sees stay
//! `unsafe`-free (bounds-checked array index).

use hal::cpu::{CpuToken, MAX_CPUS};

/// Write this CPU's logical index into `TPIDR_EL1`, establishing the per-CPU
/// anchor. On the 1-vCPU image the boot core is index `0`; APs (S3) will call
/// this with their own index from their bring-up stub.
///
/// # Safety
/// `cpu_index` must be `< MAX_CPUS` (it is the [`hal::cpu::PerCpu`] array slot)
/// and must be unique per physical CPU. Call once per CPU, early in its
/// bring-up, before any per-CPU access. On 1-vCPU the boot core passes `0`.
pub unsafe fn init_cpu(cpu_index: usize) {
    debug_assert!(cpu_index < MAX_CPUS);
    // SAFETY: `msr tpidr_el1, x` writes an EL1-owned thread-pointer register the
    // kernel reserves for the per-CPU index. It is not used for anything else on
    // aarch64 (user TLS is `TPIDR_EL0`), so this clobbers no live state.
    unsafe {
        core::arch::asm!(
            "msr tpidr_el1, {idx}",
            idx = in(reg) cpu_index as u64,
            options(nomem, nostack, preserves_flags),
        );
    }
}

/// Establish the boot CPU's per-CPU anchor: `TPIDR_EL1 = 0`.
///
/// # Safety
/// Call once, on the boot core, during bring-up before any per-CPU access.
pub unsafe fn init_bsp() {
    // SAFETY: boot core, first and only anchor install; index 0 is the BSP slot.
    unsafe { init_cpu(0) }
}

/// Mint a [`CpuToken`] for the CPU we are running on, reading the logical index
/// from `TPIDR_EL1` with one `mrs`.
///
/// # Safety
/// The caller must hold the no-migration invariant the token encodes — IRQs /
/// preemption masked so we cannot migrate to another CPU before the token is
/// dropped. On the single-core trap path that is the existing "IRQs masked at
/// EL1 during trap handling" invariant `with_sched` already relies on, so the
/// minted index (always 0 on 1-vCPU) is stable for the borrow's lifetime.
pub unsafe fn this_cpu_token() -> CpuToken {
    let idx: u64;
    // SAFETY: `mrs x, tpidr_el1` reads the per-CPU index this Frame wrote in
    // `init_cpu`. A plain register read with no side effects.
    unsafe {
        core::arch::asm!(
            "mrs {idx}, tpidr_el1",
            idx = out(reg) idx,
            options(nomem, nostack, preserves_flags),
        );
    }
    CpuToken::new(idx as usize)
}
