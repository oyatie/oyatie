//! # `kernel` — the safe kernel binary
//!
//! This is the top of the framekernel. It is `#![no_std]`, `#![no_main]`, and
//! **forbids `unsafe`**. All hardware interaction goes through the [`hal`]
//! traits and the safe [`frame`] services; the dangerous code lives only in
//! the per-arch Frame backends (`arch-aarch64` / `arch-x86_64`), each of which
//! exposes a *safe* entry that does the audited bring-up for us.
//!
//! Boot flow on aarch64:
//!   1. The Frame's `_start` assembly sets the stack + zeroes `.bss`, then
//!      calls our [`kmain`].
//!   2. [`kmain`] performs the safe-wrapped bring-up (UART, vectors, MMU, heap,
//!      GIC), prints the banner, proves `alloc` works, then starts the timer.
//!   3. The Frame's timer ISR prints `timer tick <n>`; after a few ticks it
//!      prints `kernel: OK` and powers the machine off via PSCI.
#![no_std]
#![no_main]
// The kernel is unsafe-free *code*: `deny` makes any `unsafe {}` block a hard
// error. We cannot use `forbid` because the single FFI entry symbol below needs
// `#[no_mangle]`, which the `unsafe_code` lint flags as unsafe-adjacent even
// though it contains no `unsafe` block. That one attribute carries a scoped
// `#[allow(unsafe_code)]`; everything else in the crate remains unable to write
// `unsafe`. No actual `unsafe` block exists in this crate.
#![deny(unsafe_code)]

// Both arch backends now register a `#[global_allocator]` (heap over a static
// region), so `alloc` is available on every supported target.
extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::fmt::Write;

use hal::{Arch, FmtConsole, MemoryApi, TimerApi};

// Link the selected arch Frame backend. This pulls in the boot entry symbol
// (`_start`) and the panic handler. Exactly one is linked per target.
#[cfg(target_arch = "aarch64")]
use arch_aarch64 as arch;

#[cfg(target_arch = "x86_64")]
use arch_x86_64 as arch;

#[allow(unused_imports)]
use frame as _frame;

/// Print a line to the architecture console via the safe `FmtConsole` adapter.
macro_rules! kline {
    ($arch:expr, $($t:tt)*) => {{
        let mut c = FmtConsole($arch.console());
        let _ = writeln!(c, $($t)*);
    }};
}

/// Safe kernel entry, called by the selected Frame's boot stub. Never returns.
#[allow(unsafe_code)] // `#[no_mangle]` only; no `unsafe` block — see crate docs.
#[no_mangle]
extern "C" fn kmain() -> ! {
    // Audited bring-up happens behind this safe call (UART, vectors, MMU, heap,
    // GIC are all live once it returns). `mmu: enabled` and `heap: ok` are
    // printed from inside the Frame as those milestones complete.
    let mut arch = arch::boot();

    // ---- P4·SMP·S3: bring secondary CPUs online (idle) immediately after the
    // Frame bring-up, before the user workload. On the default 1-vCPU image
    // `cpu_count()==1`, so this is a silent no-op emitting NOTHING — the
    // golden/talos serial stream is byte-identical. Under `-smp N` each AP comes
    // up (its per-CPU anchor + stack + local IRQ chip), publishes its online
    // bit, and idles; the BSP prints `cpu k online` per AP + the `smp:` summary.
    // Placed here (right after `boot()`, before any heap/banner work) so it is
    // the SAME early point on both arches and never interleaves with the later
    // ring-3 / EL0 user run.
    smp_bring_up(&mut arch);

    // ---- Milestone 1: banner ----
    let arch_name = arch.name();
    kline!(arch, "kernel: hello ({})", arch_name);
    kline!(arch, "build: {} profile, {} target", PROFILE, TARGET);
    kline!(arch, "frame: console + traps + paging + heap + IRQ online");

    // ---- Milestone 4 (proof): exercise the heap allocator ----
    let mut v: Vec<u32> = Vec::new();
    for i in 0..8 {
        v.push(i * i);
    }
    let boxed = Box::new(0xABCD_1234u32);
    kline!(arch, "alloc: vec={:?} box=*{:#x}", v.as_slice(), *boxed);

    // Report the usable RAM region the Frame carved the heap from.
    if let Ok(region) = arch.init_memory() {
        kline!(
            arch,
            "memory: heap region {:#x}..{:#x} ({} KiB)",
            region.start,
            region.end(),
            region.size / 1024
        );
    }

    // ---- Milestone 5: report the timer (IRQs are armed in the Frame) ----
    let hz = arch.timer_frequency();
    kline!(arch, "timer: CNTFRQ={} Hz, generic timer ready", hz);

    // ---- Milestone 6: drop to EL0 and run the embedded user program ----
    //
    // The Frame loads the bundled ELF, builds EL0-accessible page mappings, and
    // `eret`s to user space. The user program prints its message *through* our
    // `write` syscall, then calls `exit(0)`. Our syscall layer records the code,
    // restores the kernel map, prints `user program exited: code=0`, then
    // `kernel: OK`, and powers the machine off — so this never returns.
    #[cfg(target_arch = "aarch64")]
    {
        kline!(arch, "kernel: launching user program (EL0)");
        arch::run_user()
    }

    // On x86_64 we now run the timer-tick heartbeat *then* drop to ring 3 and
    // serve the first Linux-ABI syscalls — the x86_64 equivalent of aarch64's
    // K4. `run_user` arms the PIT (a few `timer tick <n>` lines print first, so
    // the original demo is preserved), then drops to ring 3; the user program
    // prints its message through our `write` syscall and `exit`s; the kernel
    // regains control, prints `user exited: code=0`, then `kernel: OK`,
    // and powers off — so this never returns.
    //
    // To fall back to the timer-only terminal milestone (e.g. if the ring-3
    // round-trip regresses), replace `arch::run_user()` with
    // `arch::start_timer_and_idle(200)`; the ISR then owns the OK marker.
    #[cfg(target_arch = "x86_64")]
    {
        kline!(arch, "kernel: timer heartbeat, then launching user program (ring 3)");
        arch::run_user()
    }
}

/// The diverging Rust entry the AP idle path is documented against in the
/// [`hal::smp::Smp`] seam. The arch Frame supplies its own asm prologue + idle
/// loop for S3 (the AP never actually executes this body), but passing a real
/// `extern "C" fn() -> !` keeps the seam fully typed from the safe kernel with
/// no `unsafe`. It simply spins (the Frame's `hlt`/`wfe` idle is what runs).
extern "C" fn ap_idle() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

/// Bring secondary CPUs online via the safe [`hal::smp::Smp`] seam, then report.
///
/// Gated entirely on `cpu_count()`: on the default 1-vCPU image this returns
/// `1` and prints **nothing**, so the golden/talos serial stream is unchanged.
/// Only when more than one CPU is enumerated does it emit the non-golden
/// `smp: N CPU(s) online` summary (the per-AP `cpu k online` lines are printed
/// by the Frame as the BSP observes each online bit — a single console writer).
///
/// No `unsafe`: all AP machinery is behind the arch Frame's `Smp` impl.
///
/// The actual bring-up (start + report) is in a separate `#[inline(never)]`
/// callee [`smp_start_and_report`] reached ONLY when `cpu_count() > 1`. Keeping
/// the heavy `start_secondaries`/`kline!` path out of this gate function makes
/// the 1-vCPU path a pure `cpu_count()`-check-and-return — provably the same
/// minimal codegen on the default (single-CPU) image, so it never perturbs the
/// byte-identical golden/talos boot.
#[inline(never)]
fn smp_bring_up<A: hal::Arch + hal::smp::Smp>(arch: &mut A) {
    if arch.cpu_count() <= 1 {
        // 1-vCPU: no AP code runs and the `smp:` line is suppressed so the
        // default serial stream stays byte-identical.
        return;
    }
    smp_start_and_report(arch);
}

/// The >1-CPU bring-up tail: start the APs and print the summary. Split out (and
/// `#[inline(never)]`) so it is reached only on the SMP path and never inlined
/// into the 1-vCPU gate above.
#[inline(never)]
fn smp_start_and_report<A: hal::Arch + hal::smp::Smp>(arch: &mut A) {
    match arch.start_secondaries(ap_idle) {
        Ok(online) => kline!(arch, "smp: {} CPU(s) online", online),
        Err(_) => kline!(arch, "smp: AP bring-up error"),
    }
}

/// Build profile string, surfaced in the banner.
#[cfg(debug_assertions)]
const PROFILE: &str = "debug";
#[cfg(not(debug_assertions))]
const PROFILE: &str = "release";

/// Target triple family, surfaced in the banner.
#[cfg(target_arch = "aarch64")]
const TARGET: &str = "aarch64-unknown-none";
#[cfg(target_arch = "x86_64")]
const TARGET: &str = "x86_64-unknown-none";
