//! CPU-level shapes: trap/user register frames, the immutable boot-populated
//! capability snapshot, and the [`CpuToken`]-gated per-CPU accessor.
//!
//! Everything here is a *shape* — trait signatures and plain data types. The
//! arch backend supplies the concrete register layout (its `repr(C)` struct
//! ordering must match the entry/exit assembly) and the unsafe machinery that
//! actually enters user mode or reads CPUID / `ID_AA64*`. The safe kernel only
//! ever sees these abstract types, so it never branches on a raw feature bit
//! (consensus C8) and never touches a register frame directly.

use core::sync::atomic::{AtomicU32, Ordering};

use crate::sealed::Sealed;

/// The reason an arch backend's user-mode run returned to the kernel.
///
/// Mirrors the OSTD `UserMode::execute -> ReturnReason` shape (lesson A18): the
/// safe kernel drives a `run` loop and matches on why control came back, rather
/// than inspecting privileged flags itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnReason {
    /// The user task issued a system call (`svc`/`syscall`/`ecall`).
    Syscall,
    /// A synchronous CPU exception (page fault, illegal instruction, …).
    Exception,
    /// A hardware interrupt was delivered while the task ran.
    Interrupt,
    /// A kernel-side event (e.g. a pending preemption) requested re-entry.
    KernelEvent,
}

/// Per-arch saved register state for a trap taken **from kernel or user mode**.
///
/// The concrete type is a `repr(C)` struct in the arch backend whose field
/// order matches the trap-entry stub; this trait is the arch-neutral surface
/// the safe kernel reads. Only the audited fields are exposed — privileged
/// flags (`RFLAGS.IF`/`IOPL`, `DAIF`, `sstatus.SIE`) are deliberately hidden so
/// safe code cannot escalate (lesson A18).
pub trait TrapFrame: Sealed {
    /// The faulting/return instruction pointer (PC / RIP / `ELR`).
    fn instruction_pointer(&self) -> usize;

    /// The user/kernel stack pointer captured at trap time.
    fn stack_pointer(&self) -> usize;

    /// The architectural trap/vector number that caused entry.
    fn trap_number(&self) -> usize;

    /// The CPU-supplied error code, or `0` for traps that carry none.
    fn error_code(&self) -> usize;
}

/// Per-arch user-task register context plus the enter-user / return-to-kernel
/// contract shape.
///
/// The arch backend owns the full register file and the unsafe `eret`/`sysret`
/// path; this trait is the safe seam. `run()` is the half of the OSTD
/// `UserMode::execute` loop the kernel can express safely — it transfers to
/// EL0/ring-3 and returns the [`ReturnReason`] that brought control back. The
/// concrete impl is `!Send` (it is tied to the activated address space of the
/// current task); we cannot express that bound on a trait, so the arch type
/// enforces it on its own struct.
pub trait UserContext: Sealed {
    /// Read the user instruction pointer (PC / RIP).
    fn instruction_pointer(&self) -> usize;
    /// Set the user instruction pointer (e.g. the ELF entry point).
    fn set_instruction_pointer(&mut self, ip: usize);

    /// Read the user stack pointer.
    fn stack_pointer(&self) -> usize;
    /// Set the user stack pointer (e.g. the prepared initial SysV stack top).
    fn set_stack_pointer(&mut self, sp: usize);

    /// Read the syscall return-value register (`x0` / `rax` / `a0`).
    fn return_value(&self) -> usize;
    /// Write the syscall return-value register before resuming the task.
    fn set_return_value(&mut self, val: usize);

    /// Enter user mode and run until a trap, syscall, interrupt, or kernel
    /// event returns control. The arch backend forces a sane privileged state
    /// (IF=1 / IOPL=0 / DAIF cleared) on entry regardless of caller state.
    fn run(&mut self) -> ReturnReason;
}

/// Immutable, boot-populated snapshot of the CPU's capabilities.
///
/// Probed **once** by the arch backend (CPUID `0x4000_0000`/feature leaves on
/// x86, `ID_AA64*`+`MIDR` on aarch64) before any hardware fast path runs, then
/// frozen. The safe kernel reads only these abstract bits and never the raw
/// registers (consensus C8). Every field is a *capability shape*; the arch
/// backend sets them, the kernel gates fast paths on them, and a feature-poor
/// VM simply reports `false`/`1` so the same binary takes the safe fallback
/// (consensus C9). Arch-agnostic by construction — fields name capabilities,
/// not instruction-set encodings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct CpuCaps {
    /// Number of logical CPUs the platform brought online (≥ 1).
    pub cpu_count: u32,
    /// Running as a paravirtualized guest (KVM/Nitro/Hyper-V/GCE) rather than
    /// bare metal — probed from the hypervisor signature.
    pub paravirt_guest: bool,
    /// Tagged-TLB context tags are available (x86 PCID / aarch64 ASID); when
    /// `false` the [`crate::mm::AddressSpace::switch_to`] fallback full-flushes.
    pub tagged_tlb: bool,
    /// A deadline-mode timer exists (x86 TSC-deadline / aarch64 Generic-Timer
    /// ECV); when `false` the [`crate::time::Timer`] emulates one-shots.
    pub deadline_timer: bool,
    /// Hardware user-access protection is present and pinned (x86 SMAP/SMEP/UMIP
    /// / aarch64 PAN/PXN).
    pub user_access_protection: bool,
    /// Single-instruction atomics are available (aarch64 `+lse`); when `false`
    /// the build relies on LL/SC.
    pub lse_atomics: bool,
    /// Hardware memory tagging is present (aarch64 MTE) — gates Frame tagging.
    pub memory_tagging: bool,
    /// Forward/backward-edge CFI hardware is present (x86 CET / aarch64
    /// PAC+BTI).
    pub control_flow_integrity: bool,
    /// A confidential-compute platform is active (SEV-SNP / TDX / CCA); gates
    /// the [`crate::confidential::ConfidentialPlatform`] path.
    pub confidential: bool,
}

impl CpuCaps {
    /// The conservative all-fallback snapshot: a single bare-metal CPU with no
    /// optional hardware. Useful as a `const` default and as the floor the
    /// differential tests assume when QEMU exposes nothing extra.
    pub const FALLBACK: Self = Self {
        cpu_count: 1,
        paravirt_guest: false,
        tagged_tlb: false,
        deadline_timer: false,
        user_access_protection: false,
        lse_atomics: false,
        memory_tagging: false,
        control_flow_integrity: false,
        confidential: false,
    };

    /// Mint a populated snapshot from the documented capability fields.
    ///
    /// `#[non_exhaustive]` forbids an out-of-crate struct expression (even with
    /// functional-update `..base`), so an arch backend physically cannot
    /// construct `CpuCaps` by literal — it must go through this constructor.
    /// That is the deliberate seam: `hal` owns the field set, the backend
    /// supplies the probed values. Adding a future field is non-breaking — it
    /// extends this signature (or gains a `with_*` setter) without forcing every
    /// caller to re-spell a struct literal they could never write anyway.
    ///
    /// All fields are documented on [`CpuCaps`]; pass each backend-detected bit
    /// in declaration order. Use [`CpuCaps::FALLBACK`] for the conservative
    /// all-fallback floor, or this constructor to overlay positively-probed
    /// capabilities on top of it.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        cpu_count: u32,
        paravirt_guest: bool,
        tagged_tlb: bool,
        deadline_timer: bool,
        user_access_protection: bool,
        lse_atomics: bool,
        memory_tagging: bool,
        control_flow_integrity: bool,
        confidential: bool,
    ) -> Self {
        Self {
            cpu_count,
            paravirt_guest,
            tagged_tlb,
            deadline_timer,
            user_access_protection,
            lse_atomics,
            memory_tagging,
            control_flow_integrity,
            confidential,
        }
    }
}

/// Maximum number of logical CPUs the per-CPU storage is sized for.
///
/// [`PerCpu<T>`] holds a `[T; MAX_CPUS]` array indexed by the running CPU, so
/// this is the static array cap — **not** the production vCPU target (later P4
/// scales to far more). Raising it is a one-line change, not a redesign. It
/// comfortably covers QEMU `-smp N` test sizes for the SMP arc (S3+). On the
/// 1-vCPU image only slot 0 is ever touched, so the extra slots cost only a
/// bounded amount of static `.bss` (`MAX_CPUS × sizeof(T)` per `PerCpu`) and
/// change no behavior.
pub const MAX_CPUS: usize = 8;

/// Zero-sized-except-for-its-index witness that the bearer is pinned to the
/// current CPU with migration disabled (preemption/IRQs off, per the arch
/// backend).
///
/// This is the key to [`PerCpu`]: a `&CpuToken` proves "we will not migrate
/// before this borrow ends", so accessing per-CPU data through it cannot read
/// another CPU's slot after a silent migration — the bug class OSTD's
/// `CpuLocalDerefGuard` closes (lesson A13), made a *compile-time* requirement
/// here. The token is `!Send`/`!Sync` (it must not cross CPUs) and is minted
/// only by the arch backend, which holds the actual preempt/IRQ guard and reads
/// *which* CPU this is from its per-CPU register base (GS on x86, `TPIDR_EL1`
/// on aarch64). The carried `cpu_index` is the array index [`PerCpu`] uses; it
/// is a plain `usize` the arch backend supplies — adding it keeps `hal`
/// `unsafe`-free (the only `unsafe` is the arch register read that produces the
/// index, which lives in the Frame).
pub struct CpuToken {
    /// Which logical CPU minted this token (the [`PerCpu`] array index, in
    /// `0..MAX_CPUS`). Filled in by the arch backend from its per-CPU register
    /// base when the token is minted.
    cpu_index: usize,
    /// Makes the token neither `Send` nor `Sync` and un-constructible outside
    /// this crate (no public field initializer).
    _not_send_sync: core::marker::PhantomData<*const ()>,
}

impl CpuToken {
    /// Mint a token for the CPU whose logical index is `cpu_index`.
    ///
    /// This is the seam the arch backend uses to construct the token: only the
    /// Frame (which holds the actual preempt/IRQ guard that makes the pinning
    /// claim true, and reads `cpu_index` from its per-CPU register base — GS on
    /// x86, `TPIDR_EL1` on aarch64) is the intended caller. It is a plain `fn`
    /// because `hal` is `#![forbid(unsafe_code)]` and so cannot mark it
    /// `unsafe`; the *real* guard is structural — the token is `!Send`/`!Sync`
    /// so it can never cross to another CPU, and the only meaningful index is
    /// the one the arch register read produces. Mirrors [`CpuCaps::new`], the
    /// other "hal owns the type, the backend supplies probed values" seam.
    ///
    /// `cpu_index` must be `< MAX_CPUS`; it is the array slot [`PerCpu::get`]
    /// will index. Out-of-range indices are caught by the bounds check on every
    /// access (a panic, not UB) — `hal` adds no `unsafe`.
    pub const fn new(cpu_index: usize) -> Self {
        Self {
            cpu_index,
            _not_send_sync: core::marker::PhantomData,
        }
    }

    /// The logical index of the CPU this token was minted on (its [`PerCpu`]
    /// array slot, in `0..MAX_CPUS`).
    pub fn cpu_index(&self) -> usize {
        self.cpu_index
    }
}

/// A per-CPU value, accessible only while pinned to the current CPU.
///
/// Storage is a fixed `[T; MAX_CPUS]` array; the *index* (this CPU's id) is
/// supplied by the arch backend through the [`CpuToken`]. `get`/`get_mut` do a
/// **bounds-checked array index** — ordinary safe Rust, so this stays inside
/// the `#![forbid(unsafe_code)]` `hal` crate with **zero `unsafe`**. The real
/// per-CPU register base (GS-relative on x86, `TPIDR_EL1` on aarch64) is only
/// consulted by the arch Frame when it mints the token's `cpu_index`; the
/// stale-after-migration bug class is rejected by the borrow checker because
/// every access requires the `&CpuToken` pinning proof (three-pillars §5:
/// fearless concurrency).
pub struct PerCpu<T> {
    /// One slot per logical CPU. Indexed by [`CpuToken::cpu_index`].
    slots: [T; MAX_CPUS],
}

impl<T: Copy> PerCpu<T> {
    /// Define a per-CPU slot, replicating `value` into every CPU's slot as the
    /// initial value. `T: Copy` so the array can be built in a `const` context
    /// (the per-CPU statics are `const`-initialized).
    pub const fn new(value: T) -> Self {
        Self {
            slots: [value; MAX_CPUS],
        }
    }

    /// Borrow this CPU's slot. The `&CpuToken` proves we are pinned, so the
    /// borrow cannot outlive the no-migration window; the index it carries
    /// selects this CPU's slot via a bounds-checked array index.
    pub fn get<'a>(&'a self, token: &'a CpuToken) -> &'a T {
        &self.slots[token.cpu_index()]
    }

    // NOTE: `get_mut(&mut self, …) -> &mut T` was **removed** in P4·SMP·S2.
    //
    // It took `&mut self` of the *whole* `[T; MAX_CPUS]` array. Under real SMP
    // (S3+) two CPUs each running a per-CPU writeback would both materialise
    // `&mut *static` to the same array object — instant aliasing UB (LLVM
    // `noalias` / Stacked-/Tree-Borrows), even though each then indexes a
    // *disjoint* slot: the borrow is of the array, not the slot, so disjoint
    // indices do not save it. The single former caller (`CURRENT` writeback in
    // each Frame's `with_sched`) migrated to [`PerCpuU32`] below, whose `store`
    // takes `&self` and never forms a whole-array `&mut`. Non-`Copy`/non-atomic
    // per-CPU state takes the Frame's `PerCpuLocal<T>` (`UnsafeCell` per slot).
    // Deleting `get_mut` makes the hazardous shape unrepresentable in `hal`.
}

/// Per-CPU `u32` cell array: one [`AtomicU32`] per logical CPU, indexed by the
/// running CPU's [`CpuToken`].
///
/// Unlike [`PerCpu<T>`] this needs **no** `&mut self` to write a slot —
/// [`store`](Self::store) / [`load`](Self::load) take `&self` — so two CPUs
/// writing **disjoint** slots never form overlapping mutable borrows of the
/// array (the aliasing UB that retired `PerCpu::get_mut`). `AtomicU32: Sync`
/// gives `Sync` with **zero** `unsafe`, so this stays inside `hal`'s
/// `#![forbid(unsafe_code)]` forbid-set.
///
/// Ordering is `Relaxed`: a slot is read/written only by the CPU that owns it
/// (the `CpuToken` proves you are that CPU), so nothing is *published* through
/// it — cross-CPU happens-before for shared tables is the `SpinLock`'s job.
/// This matches the bump allocator's "partitioning, not publishing" `Relaxed`
/// argument.
pub struct PerCpuU32 {
    /// One independently load/store-able cell per logical CPU.
    slots: [AtomicU32; MAX_CPUS],
}

impl PerCpuU32 {
    /// A per-CPU `u32` array with every slot seeded to `0`.
    ///
    /// `AtomicU32::store` is not `const fn`, so a non-zero seed cannot be built
    /// in a `const fn`; every current caller (`CURRENT`) seeds `0`, so this
    /// const constructor builds the all-zero array via the inline-const array
    /// initializer. (If a non-zero seed is ever needed, add a non-`const`
    /// `new_seeded` — not required today.)
    #[allow(clippy::declare_interior_mutable_const)]
    pub const fn new() -> Self {
        Self {
            slots: [const { AtomicU32::new(0) }; MAX_CPUS],
        }
    }

    /// This CPU's slot value. `&self`, no exclusive borrow of the array.
    pub fn load(&self, token: &CpuToken) -> u32 {
        self.slots[token.cpu_index()].load(Ordering::Relaxed)
    }

    /// Write this CPU's slot. `&self` — disjoint slots never alias.
    pub fn store(&self, token: &CpuToken, value: u32) {
        self.slots[token.cpu_index()].store(value, Ordering::Relaxed);
    }
}

impl Default for PerCpuU32 {
    fn default() -> Self {
        Self::new()
    }
}
