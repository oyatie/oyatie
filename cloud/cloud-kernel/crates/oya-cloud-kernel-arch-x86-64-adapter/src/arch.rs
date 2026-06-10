//! The `X86_64` Frame: bring-up sequence, HAL implementation, heap, QEMU exit.
//!
//! Ties the x86_64 device modules together and exposes the safe [`hal::Arch`]
//! surface the kernel programs against. Owns the global allocator (a
//! `linked_list_allocator` over a static region), interrupt enable/disable, and
//! the isa-debug-exit self-termination used to end the QEMU run. Mirrors the
//! aarch64 backend so both arches reach the same parity.

use linked_list_allocator::LockedHeap;

use hal::{Arch, ArchError, ConsoleWrite, InterruptApi, MemoryApi, MemoryRegion, TimerApi};

/// Size of the static kernel heap region, matching aarch64. The default image
/// keeps the original **1 MiB** so the default boot trace is unchanged. Under
/// `--features talos-init` it grows to **16 MiB**: the real `talos-init` musl
/// image is ~1.2 MB and the ELF loader backs every user page with its own 4 KiB
/// kernel-heap frame, so loading the image plus the 8 MiB window's leaf page
/// tables, the initial stack, COW copies, and process metadata far exceeds
/// 1 MiB. 16 MiB fits inside QEMU's 256 MiB. NON-default; gated so the default
/// golden is untouched.
#[cfg(not(feature = "talos-init"))]
pub const HEAP_SIZE: usize = 1024 * 1024;
#[cfg(feature = "talos-init")]
pub const HEAP_SIZE: usize = 16 * 1024 * 1024;

/// Backing storage for the kernel heap.
///
/// Placed in the dedicated `.heap` linker section, which the linker script pins
/// at `0xC0_0000` (== `user_layout::USER_TOP`) — i.e. ABOVE the ring-3 user
/// window `[0x40_0000, 0xC0_0000)`. This is load-bearing on x86_64: a process
/// page table refines every PD slot inside the user window to its own leaf page
/// tables, so once a process CR3 is live, any *kernel* virtual address that fell
/// inside the window is shadowed by the process's user mapping. If the heap
/// overlapped the window, a kernel structure the allocator placed there (the
/// scheduler's process table, a page-table `Box`, etc.) would read back the user
/// image — not kernel data — whenever the timer ISR or a syscall ran under that
/// CR3, corrupting the scheduler. Anchoring the heap above the window keeps every
/// heap object identity-mapped under every CR3. (aarch64 needs no such anchor:
/// its heap links at ~1 GiB, far above the window.) `.heap` is NOLOAD and lies
/// outside `[__bss_start, __bss_end)`, so the boot trampoline's `.bss` clear does
/// not touch it; the allocator seeds its own free-list node and every frame /
/// page table is zeroed on allocation, so pre-zeroing is unnecessary.
#[link_section = ".heap"]
static mut HEAP_SPACE: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

/// The virtual address the linker script pins the `.heap` section at
/// (`linker.ld`: `. = 0xC00000; .heap (NOLOAD) : ...`). It MUST equal
/// `user_layout::USER_TOP` so the whole kernel heap sits ABOVE the ring-3 user
/// window `[USER_BASE, USER_TOP)`. If the heap ever fell back inside the window,
/// every heap object would be shadowed under a live process CR3 (the Slice-0b
/// regression). This constant mirrors the literal in `linker.ld`; the
/// compile-time guard below ties it to `USER_TOP`, and the link-time guard in
/// `init_heap` ties the *actual* `.heap` symbol to it so neither can drift
/// silently.
const HEAP_LINK_BASE: usize = 0xC0_0000;

/// COMPILE-TIME GUARD (Slice-0b regression fence): fail the build if the pinned
/// heap base is not at/above `USER_TOP`. This is the cheap, always-on half of
/// the guard — it catches anyone lowering `HEAP_LINK_BASE` (or `USER_TOP`
/// rising past it) without re-checking the overlap. The `init_heap` runtime
/// check covers the complementary case where `linker.ld`'s literal diverges
/// from this const. Kept in the arch crate (TCB), not the safe surface.
const _: () = assert!(
    HEAP_LINK_BASE >= user_layout::USER_TOP,
    "x86_64 kernel heap must start at/above USER_TOP (0xC00000): the heap may \
     not overlap the ring-3 user window [USER_BASE, USER_TOP) or it is shadowed \
     under a process CR3 (Slice-0b overlap regression)"
);
/// And it must equal `USER_TOP` exactly (the heap is anchored *at* the top of
/// the window so no kernel VA is wasted and the whole heap stays in the boot
/// identity map's low 1 GiB).
const _: () = assert!(
    HEAP_LINK_BASE == user_layout::USER_TOP,
    "HEAP_LINK_BASE must equal USER_TOP; update linker.ld and this const together"
);

/// The global allocator: a spinlocked linked-list heap over [`HEAP_SPACE`].
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Default periodic timer frequency for the demo (~100 Hz).
const TIMER_HZ: u32 = 100;

/// Initialize the global heap allocator over [`HEAP_SPACE`].
///
/// # Safety
/// Call exactly once, after paging is up, before any allocation.
unsafe fn init_heap() -> usize {
    let start = core::ptr::addr_of_mut!(HEAP_SPACE) as *mut u8;
    // LINK-TIME GUARD (Slice-0b regression fence, dynamic half): assert the
    // *actual* linked `.heap` base sits at/above USER_TOP. The compile-time
    // `const _` guards above pin `HEAP_LINK_BASE == USER_TOP`; this catches the
    // complementary failure where `linker.ld`'s literal drifts from the const,
    // landing `HEAP_SPACE` back inside the ring-3 window [USER_BASE, USER_TOP)
    // where a live process CR3 would shadow every heap object. Cheap (one
    // compare, once at boot) and panics loudly before any allocation if violated.
    assert!(
        (start as usize) >= user_layout::USER_TOP,
        "x86_64 kernel heap linked below USER_TOP — overlaps the ring-3 user \
         window and would be shadowed under a process CR3 (Slice-0b regression)"
    );
    debug_assert_eq!(
        start as usize,
        HEAP_LINK_BASE,
        "linked .heap base diverged from HEAP_LINK_BASE / linker.ld"
    );
    // SAFETY: `HEAP_SPACE` is a unique static of exactly `HEAP_SIZE` bytes; we
    // hand the allocator that exact range, once.
    unsafe {
        ALLOCATOR.lock().init(start, HEAP_SIZE);
    }
    HEAP_SIZE
}

/// The x86_64 architecture backend handed to the safe kernel.
pub struct X86_64 {
    console: crate::console::Com1,
    heap_bytes: usize,
}

impl MemoryApi for X86_64 {
    fn init_memory(&mut self) -> Result<MemoryRegion, ArchError> {
        let start = core::ptr::addr_of!(HEAP_SPACE) as usize;
        Ok(MemoryRegion::new(start, self.heap_bytes))
    }
}

impl InterruptApi for X86_64 {
    fn enable_irq(&mut self) {
        x86_64::instructions::interrupts::enable();
    }

    fn disable_irq(&mut self) {
        x86_64::instructions::interrupts::disable();
    }
}

/// x86 [`ksync::spinlock::IrqController`] for `lock_irqsave` (P4·SMP·S2, Part B).
///
/// Disables IRQs on this CPU while a `SpinLock` is held and restores the PRIOR
/// `RFLAGS.IF` on drop — re-enabling only if IRQs were enabled before, never a
/// blind `sti` (which would wrongly enable IRQs inside an already-IRQs-off
/// region, e.g. a trap handler that took the lock). Uses the same primitives as
/// the `InterruptApi` impl above (`interrupts::disable/enable`). The Frame, not
/// `ksync`, owns this so `ksync` stays arch-agnostic + loom-testable.
///
/// Defined in S2 (proven by H2's `lock_irqsave` model under `NoIrq`); wired into
/// the global `ProcTable` lock in S4 — hence `dead_code` until then.
#[allow(dead_code)]
pub struct X86Irq;

impl ksync::spinlock::IrqController for X86Irq {
    /// Prior `RFLAGS.IF` (were IRQs enabled before we disabled them?).
    type State = bool;

    fn disable() -> bool {
        let was_enabled = x86_64::instructions::interrupts::are_enabled();
        x86_64::instructions::interrupts::disable();
        was_enabled
    }

    fn restore(was_enabled: bool) {
        if was_enabled {
            x86_64::instructions::interrupts::enable();
        }
    }
}

impl TimerApi for X86_64 {
    fn timer_frequency(&self) -> u64 {
        crate::timer::frequency()
    }

    fn timer_now(&self) -> u64 {
        crate::interrupts::TICKS.load(core::sync::atomic::Ordering::Relaxed) as u64
    }

    fn set_timer(&mut self, _ticks: u64) -> Result<(), ArchError> {
        // The Frame drives the periodic PIT internally; the safe kernel only
        // observes ticks. A bespoke one-shot is not needed for this MVP.
        Err(ArchError::Unsupported)
    }
}

impl Arch for X86_64 {
    type Console = crate::console::Com1;

    fn name(&self) -> &'static str {
        "x86_64"
    }

    fn console(&mut self) -> &mut Self::Console {
        &mut self.console
    }

    fn halt(&self) -> ! {
        loop {
            x86_64::instructions::hlt();
        }
    }
}

/// Safe entry point for the kernel: perform the full, audited bring-up and hand
/// back a ready [`X86_64`].
///
/// All dangerous work is encapsulated in [`bringup`] (which is `unsafe`); this
/// wrapper exposes it as a safe call so the `#![forbid(unsafe_code)]` kernel can
/// drive boot. Call exactly once from the boot path.
pub fn boot() -> X86_64 {
    // SAFETY: invoked exactly once from `rust_start` on the boot core, in the
    // long-mode state the trampoline established (paging on, IRQs masked),
    // which is precisely `bringup`'s contract.
    let (arch, _heap) = unsafe { bringup() };
    arch
}

/// Full x86_64 bring-up. Runs in `unsafe` Frame context; returns a ready
/// [`X86_64`] and the heap size in bytes.
///
/// # Safety
/// Call once, on the boot core, in long mode with interrupts masked (the state
/// the boot trampoline hands us at `rust_start`).
pub unsafe fn bringup() -> (X86_64, usize) {
    // 1. Console first, so everything after is observable.
    crate::console::init();
    crate::kprintln!("kernel: hello (x86_64)");
    crate::kprintln!("paging: long mode");

    // 2. GDT + TSS (IST stack for double faults + RSP0 kernel stack), then the
    //    IDT.
    // SAFETY: first and only install on the boot core, IRQs masked.
    unsafe {
        crate::gdt::init();
        crate::interrupts::init_idt();
    }

    // 2b. Enable the CpuCaps-gated supervisor-mode protections (SMEP/SMAP/UMIP)
    //     now that the IDT is up (so a spurious #GP would reach a handler). Safe
    //     in this slice: there is no user mode/mapping yet for them to break.
    // SAFETY: first and only CR4 protection-bit enable, on the boot core after
    // the GDT/IDT are installed; only the CPUID-gated bits are added.
    let prot = unsafe { crate::cr4::enable_supervisor_protections() };
    crate::kprintln!(
        "cr4: smep={} smap={} umip={}",
        prot.smep,
        prot.smap,
        prot.umip
    );

    // 3. Heap over the static region (paging is already on).
    // SAFETY: first and only heap init.
    let heap_bytes = unsafe { init_heap() };
    crate::kprintln!("heap: ok ({} bytes)", heap_bytes);

    // 4. Remap + mask the 8259 PIC (only IRQ0 enabled).
    // SAFETY: first PIC init on the boot core.
    unsafe {
        crate::interrupts::init_pic();
    }

    (
        X86_64 {
            console: crate::console::Com1,
            heap_bytes,
        },
        heap_bytes,
    )
}

/// Start the periodic timer, unmask IRQs, then idle. The timer ISR exits QEMU
/// after the demo ticks, so in practice this never returns.
///
/// A **safe** wrapper the (unsafe-forbidding) kernel calls: all dangerous work
/// was vetted in [`bringup`]; here we only arm the PIT and run `sti`.
///
/// This is the **preserved fallback** terminal milestone: if the ring-3 step is
/// not wired into the boot path, the kernel still reaches `kernel: OK`
/// here (the ISR owns the exit, [`crate::interrupts::TIMER_OWNS_EXIT`] = true).
pub fn start_timer_and_idle(_period_ms: u64) -> ! {
    // SAFETY: PIC was configured in `bringup`; this is the single place that
    // programs the PIT and unmasks IRQ delivery.
    unsafe {
        crate::timer::init(TIMER_HZ);
    }
    x86_64::instructions::interrupts::enable();
    loop {
        x86_64::instructions::hlt();
    }
}

/// Run the timer-tick heartbeat, then drop to **ring 3** and serve the first
/// Linux-ABI syscalls (the x86_64 equivalent of aarch64's K4). Terminal
/// milestone: never returns to its caller.
///
/// Ordering (so the existing path is preserved and observable *before* the new
/// step): arm the PIT as a heartbeat with the ISR's auto-exit suppressed
/// ([`crate::interrupts::TIMER_OWNS_EXIT`] = false), let a few `timer tick <n>`
/// lines print, then mask IRQs and hand off to [`crate::user::run_user`], which
/// drops to ring 3, prints the user-mode message via our `write` syscall, prints
/// `user exited: code=0`, then `kernel: OK`, and powers the machine off.
///
/// A **safe** wrapper: all dangerous work (SYSCALL MSRs, page-table refinement,
/// the `iretq` to ring 3, the validated user read) is vetted inside
/// `crate::user`; here we only arm the PIT, observe ticks, and call into it.
pub fn run_user() -> ! {
    use core::sync::atomic::Ordering;

    // The timer ticks are now a heartbeat that runs *before* the process model,
    // not the terminal milestone, so the ISR must not print OK / exit QEMU.
    crate::interrupts::TIMER_OWNS_EXIT.store(false, Ordering::Relaxed);

    // SAFETY: PIC was configured in `bringup`; arm the PIT (single programmer)
    // and unmask IRQ delivery for the heartbeat.
    unsafe {
        crate::timer::init(TIMER_HZ);
    }
    x86_64::instructions::interrupts::enable();

    // Let the timer demo tick a few times (printed by the ISR) before the
    // process model, preserving the visible boot heartbeat.
    while crate::interrupts::TICKS.load(Ordering::Relaxed) < 3 {
        x86_64::instructions::hlt();
    }

    // Hand off to the process model. IRQs stay ENABLED: `user::run_user` swaps
    // the IRQ0 handler to the register-saving preemption stub and drops to ring
    // 3, where the periodic PIT time-slices between processes. The workload
    // finishes by powering off from inside `user`. Never returns.
    crate::user::run_user()
}

// Keep `ConsoleWrite` in scope so trait methods on the console resolve.
const _: fn(&mut crate::console::Com1, u8) = <crate::console::Com1 as ConsoleWrite>::write_byte;
