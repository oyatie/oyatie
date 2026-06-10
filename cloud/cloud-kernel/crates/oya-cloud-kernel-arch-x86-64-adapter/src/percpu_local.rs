//! General per-CPU storage for **non-atomic** `T` (P4·SMP·S2, Part A.2).
//!
//! [`hal::cpu::PerCpuU32`] covers `Copy`/atomic per-CPU scalars (`CURRENT`) with
//! zero `unsafe`, in `hal`. But S4's per-CPU `LocalRunQueue` (a `VecDeque`/deque
//! of runnable pids) is **not** atomic and needs `&mut T` per slot. The sound
//! way to hand out a per-slot `&mut` **without** a whole-array `&mut` (the UB
//! that retired `hal::cpu::PerCpu::get_mut`) is `[UnsafeCell<T>; N]` plus a
//! per-slot raw-pointer accessor. `UnsafeCell` + `unsafe impl Sync` are
//! forbidden in `hal`, so this type lives **here in the Frame** (TCB), where
//! `unsafe` is measured.
//!
//! ## Why the raw-pointer borrow is sound where `get_mut` was not
//! `PerCpu::get_mut(&mut self, …)` asserted exclusivity over the **whole
//! array**; `UnsafeCell::get` (`*mut T`) asserts nothing — it takes only a
//! shared `&self`. The exclusivity claim is discharged *per slot* by the
//! `CpuToken` invariant (one token per CPU, disjoint indices, no migration),
//! exactly the "disjoint mutable borrows via raw pointers are fine" rule. The
//! original UB was the whole-array `&mut`; this never forms one. (Same
//! `UnsafeCell::with_mut(|p| &mut *p)` shape `ksync::spinlock` uses soundly.)
//!
//! ## Scope discipline
//! Defined in **S2**, wired into a real `LocalRunQueue` in **S4** — landing the
//! sound shape now keeps S4's diff additive and lets it be reviewed under the
//! calm S2 gate. The Frame is never built under `--cfg loom`, so this uses the
//! production `core::cell::UnsafeCell` directly; if S4 wants a loom model of
//! `PerCpuLocal` it adds the `loom::cell` shim then (the accessor shape is
//! already `with_mut`-compatible).

use core::cell::UnsafeCell;

use hal::cpu::{CpuToken, MAX_CPUS};

/// General per-CPU storage for non-atomic `T`. Each CPU's slot is an independent
/// [`UnsafeCell<T>`]; [`with_local`](Self::with_local) derives `&mut T` from
/// **that slot's** raw pointer — never from `&mut self` of the array — so two
/// CPUs in `with_local` on **disjoint** slots never alias.
pub struct PerCpuLocal<T> {
    /// One independently borrowable cell per logical CPU.
    slots: [UnsafeCell<T>; MAX_CPUS],
}

// SAFETY: the ONLY way to reach a slot is `with_local(&self, token, f)`, which
// indexes `slots[token.cpu_index()]`. `CpuToken` is `!Send`/`!Sync` (a
// `PhantomData<*const ()>`) and is minted only while the bearer is pinned to its
// own CPU with migration disabled (the trap path's IRQs-masked invariant), so a
// given `cpu_index` is used by AT MOST ONE thread at a time, and that thread is
// the unique owner of that slot. Distinct CPUs hold distinct tokens ⇒ index
// DISJOINT slots ⇒ the `&mut T` we hand each are to different `UnsafeCell`s ⇒ no
// aliasing, no data race. We never form a borrow of the `slots` array as a
// whole (the bug that retired `PerCpu::get_mut`). `T: Send` because a slot's
// value is logically owned by whichever CPU currently runs there (it does not
// migrate while a token lives). Mirrors `ksync::spinlock`'s `unsafe impl<T:
// Send> Sync` reasoning.
unsafe impl<T: Send> Sync for PerCpuLocal<T> {}

impl<T> PerCpuLocal<T> {
    /// Wrap one already-built `UnsafeCell` per CPU. The caller (Frame) seeds the
    /// per-CPU initial values; e.g.
    /// `core::array::from_fn(|_| UnsafeCell::new(T::default()))`.
    pub const fn new(slots: [UnsafeCell<T>; MAX_CPUS]) -> Self {
        Self { slots }
    }

    /// Run `f` with `&mut T` for **this** CPU's slot.
    ///
    /// Sound because the `&mut` is derived from the slot's own raw pointer
    /// ([`UnsafeCell::get`] on a *shared* `&self`), not from `&mut self` of the
    /// array; the per-slot exclusivity claim is discharged by the `CpuToken`
    /// invariant (one token per CPU, disjoint indices, no migration). The index
    /// is bounds-checked.
    pub fn with_local<R>(&self, token: &CpuToken, f: impl FnOnce(&mut T) -> R) -> R {
        let cell: &UnsafeCell<T> = &self.slots[token.cpu_index()];
        // SAFETY: see the `unsafe impl Sync` argument — this CPU is the unique,
        // migration-pinned accessor of `cell`; the `&mut T` cannot alias another
        // CPU's slot (disjoint cell) nor a second `&mut` to THIS slot (single
        // CPU, non-reentrant within the no-migration window).
        let r: &mut T = unsafe { &mut *cell.get() };
        f(r)
    }
}

// ===========================================================================
// Host model: single-thread soundness check on disjoint slots. Not a loom
// harness (S2's gate models the LOCK, not PerCpuLocal); this just keeps the
// S2-defined primitive exercised so it is not dead code before S4. Under Miri it
// additionally checks the per-slot raw-pointer borrow is data-race free.
// ===========================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disjoint_slots_are_independent() {
        // Two distinct CPUs (tokens 0 and 1) each mutate their own slot; the
        // values stay independent — proving the accessor is per-slot, not a
        // whole-array borrow.
        let pc: PerCpuLocal<u32> =
            PerCpuLocal::new(core::array::from_fn(|_| UnsafeCell::new(0u32)));
        let t0 = CpuToken::new(0);
        let t1 = CpuToken::new(1);
        pc.with_local(&t0, |v| *v += 10);
        pc.with_local(&t1, |v| *v += 20);
        assert_eq!(pc.with_local(&t0, |v| *v), 10);
        assert_eq!(pc.with_local(&t1, |v| *v), 20);
    }
}
