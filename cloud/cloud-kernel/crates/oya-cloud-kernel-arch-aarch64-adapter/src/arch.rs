//! The `Aarch64` Frame: bring-up sequence, HAL implementation, heap, power-off.
//!
//! This module ties the device drivers together and exposes the safe
//! [`hal::Arch`] surface the kernel programs against. It is the one place that
//! owns the global allocator (a `linked_list_allocator` over a static region),
//! interrupt enable/disable, and the PSCI power-off used to self-terminate the
//! QEMU run.

use core::sync::atomic::Ordering;

use aarch64_cpu::asm;
use aarch64_cpu::registers::DAIF;
use linked_list_allocator::LockedHeap;
use tock_registers::interfaces::{Readable, Writeable};

use hal::{Arch, ArchError, ConsoleWrite, InterruptApi, MemoryApi, MemoryRegion, TimerApi};

/// Size of the static kernel heap region. The default image keeps the original
/// **1 MiB** so the default boot trace (and the aarch64 golden) is byte-identical.
/// Under `--features talos-init` it grows to **16 MiB**: the real `talos-init`
/// musl image is ~1 MB and the ELF loader backs every user page with its own
/// 4 KiB kernel-heap frame (`alloc_frame`), so loading the image plus the 8 MiB
/// window's leaf page tables, the initial stack, COW copies, and process
/// metadata far exceeds 1 MiB. 16 MiB sits comfortably inside QEMU's 256 MiB and
/// the 1 GiB RAM identity map. NON-default; gated so the golden is untouched.
#[cfg(not(feature = "talos-init"))]
pub const HEAP_SIZE: usize = 1024 * 1024;
#[cfg(feature = "talos-init")]
pub const HEAP_SIZE: usize = 16 * 1024 * 1024;

/// The backing storage for the kernel heap. Lives in `.bss` (zeroed on boot).
static mut HEAP_SPACE: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

/// The global allocator. `linked_list_allocator`'s `LockedHeap` is a
/// `#[global_allocator]`-compatible allocator over a single contiguous region.
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Initialize the global heap allocator over [`HEAP_SPACE`].
///
/// # Safety
/// Call exactly once, after the MMU is enabled (so the region is Normal
/// cacheable memory) and before any allocation occurs.
unsafe fn init_heap() -> usize {
    let start = core::ptr::addr_of_mut!(HEAP_SPACE) as *mut u8;
    // SAFETY: `HEAP_SPACE` is a unique static of exactly `HEAP_SIZE` bytes; we
    // hand the allocator that exact range, once.
    unsafe {
        ALLOCATOR.lock().init(start, HEAP_SIZE);
    }
    HEAP_SIZE
}

/// The aarch64 architecture backend handed to the safe kernel.
pub struct Aarch64 {
    console: crate::uart::Pl011,
    heap_bytes: usize,
}

impl Aarch64 {
    /// Borrow the underlying console for direct Frame-side printing.
    fn console_ref(&mut self) -> &mut crate::uart::Pl011 {
        &mut self.console
    }
}

impl MemoryApi for Aarch64 {
    fn init_memory(&mut self) -> Result<MemoryRegion, ArchError> {
        // The heap region was already installed during bring-up; report it.
        let start = core::ptr::addr_of!(HEAP_SPACE) as usize;
        Ok(MemoryRegion::new(start, self.heap_bytes))
    }
}

impl InterruptApi for Aarch64 {
    fn enable_irq(&mut self) {
        // Clear the IRQ mask bit in DAIF.
        DAIF.write(DAIF::I::Unmasked);
    }

    fn disable_irq(&mut self) {
        DAIF.write(DAIF::I::Masked);
    }
}

/// aarch64 [`ksync::spinlock::IrqController`] for `lock_irqsave` (P4·SMP·S2,
/// Part B).
///
/// Masks IRQs on this CPU (`DAIF.I = Masked`) while a `SpinLock` is held and
/// restores the PRIOR `DAIF.I` on drop — re-unmasking only if IRQs were unmasked
/// before, never a blind unmask (which would wrongly enable IRQs inside an
/// already-masked region, e.g. an EL1 trap handler that took the lock). Uses the
/// same `DAIF` primitive as the `InterruptApi` impl above. Lives in the Frame,
/// not `ksync`, so `ksync` stays arch-agnostic + loom-testable.
///
/// Defined in S2 (proven by H2's `lock_irqsave` model under `NoIrq`); wired into
/// the global `ProcTable` lock in S4 — hence `dead_code` until then.
#[allow(dead_code)]
pub struct Aarch64Irq;

impl ksync::spinlock::IrqController for Aarch64Irq {
    /// Prior IRQ-unmasked state (were IRQs unmasked, i.e. `DAIF.I == 0`, before?).
    type State = bool;

    fn disable() -> bool {
        // DAIF.I == 1 means IRQs MASKED; so "was enabled" = I bit clear.
        let was_unmasked = !DAIF.is_set(DAIF::I);
        DAIF.write(DAIF::I::Masked);
        was_unmasked
    }

    fn restore(was_unmasked: bool) {
        if was_unmasked {
            DAIF.write(DAIF::I::Unmasked);
        }
    }
}

impl TimerApi for Aarch64 {
    fn timer_frequency(&self) -> u64 {
        crate::timer::frequency()
    }

    fn timer_now(&self) -> u64 {
        crate::timer::now()
    }

    fn set_timer(&mut self, _ticks: u64) -> Result<(), ArchError> {
        // The Frame drives the periodic timer internally; the safe kernel only
        // observes ticks. A bespoke one-shot is not needed for this MVP.
        Err(ArchError::Unsupported)
    }
}

impl Arch for Aarch64 {
    type Console = crate::uart::Pl011;

    fn name(&self) -> &'static str {
        "aarch64"
    }

    fn console(&mut self) -> &mut Self::Console {
        self.console_ref()
    }

    fn halt(&self) -> ! {
        loop {
            asm::wfe();
        }
    }
}

/// PSCI `SYSTEM_OFF` function ID (SMC32/HVC calling convention).
const PSCI_SYSTEM_OFF: u64 = 0x8400_0008;

/// Power the machine off via PSCI `SYSTEM_OFF` over `HVC`. Never returns.
///
/// On QEMU `virt` the PSCI conduit is `HVC`; this cleanly terminates the
/// emulator so the run self-completes.
pub fn power_off() -> ! {
    // SAFETY: `hvc #0` with x0 = PSCI SYSTEM_OFF is the standard, side-effect-
    // free request to the firmware/hypervisor to power down. It does not return.
    unsafe {
        core::arch::asm!(
            "mov x0, {fid}",
            "hvc #0",
            fid = in(reg) PSCI_SYSTEM_OFF,
            options(nomem, nostack, noreturn),
        );
    }
}

/// The number of timer ticks observed so far. Part of the Frame's public
/// surface for inspection/tests; not on the critical boot path.
#[allow(dead_code)]
pub fn ticks() -> u32 {
    crate::gic::TICKS.load(Ordering::Relaxed)
}

/// Safe entry point for the kernel: perform the full, audited bring-up and
/// hand back a ready [`Aarch64`].
///
/// All the dangerous work is encapsulated in [`bringup`] (which is `unsafe`);
/// this wrapper exposes it as a safe call so the `#![forbid(unsafe_code)]`
/// kernel can drive boot. It must be called exactly once from the boot path.
pub fn boot() -> Aarch64 {
    // SAFETY: this is invoked exactly once from `rust_start` on the boot core,
    // in the post-reset state QEMU provides (MMU off, IRQs masked), which is
    // precisely `bringup`'s contract.
    let (arch, _heap) = unsafe { bringup() };
    arch
}

/// Full aarch64 bring-up. Runs in `unsafe` Frame context; returns a ready
/// [`Aarch64`] for the safe kernel and the heap size in bytes.
///
/// # Safety
/// Call once, on the boot core, with the MMU off and interrupts masked (the
/// state QEMU hands us at `_start`).
pub unsafe fn bringup() -> (Aarch64, usize) {
    // 1. Console first, so everything after is observable.
    crate::console::init();

    // 2. Exception vectors, so any fault from here on is debuggable.
    // SAFETY: first and only install on the boot core.
    unsafe {
        crate::exceptions::init();
    }

    // 3. MMU: identity map device + RAM, enable translation + caches.
    // SAFETY: MMU is currently off; called once on the boot core.
    unsafe {
        crate::mmu::enable();
    }
    crate::kprintln!("mmu: enabled");

    // 3b. Per-CPU anchor: write this (boot) CPU's logical index into TPIDR_EL1
    //     so `percpu::this_cpu_token()` mints a CpuToken indexing slot 0. On the
    //     1-vCPU image this is always 0, so the per-CPU scheduler `current`
    //     reads/writes slot 0 exactly as the old single global did — golden
    //     byte-identical. APs (S3) will anchor their own index.
    // SAFETY: boot core, first per-CPU anchor install before any per-CPU access.
    unsafe {
        crate::percpu::init_bsp();
    }

    // 4. Heap over the static region (now Normal cacheable under the MMU).
    // SAFETY: MMU is up; first and only heap init.
    let heap_bytes = unsafe { init_heap() };
    crate::kprintln!("heap: ok ({} bytes)", heap_bytes);

    // 5. Interrupt controller + timer PPI. Runtime-probe the GIC architecture
    //    version (GICD_PIDR2[7:4]) and bring up the matching chip, storing the
    //    choice once for the per-IRQ ack/EOI dispatch (mirrors the x86 x2APIC
    //    slice's boot-immutable tier selection). GICv2 is the byte-identical
    //    fallback; the v3 leg never touches the GICC MMIO (absent on v3 boards).
    // SAFETY: first GIC/timer init on the boot core.
    let gic_version = crate::gicv3::probe_version();
    crate::gicv3::store_version(gic_version);
    unsafe {
        match gic_version {
            crate::gicv3::GicVersion::V3 => {
                crate::gicv3::init();
                crate::gicv3::enable_ppi(crate::gic::TIMER_INTID);
                // P4·SMP·S4c: enable the cross-CPU TLB-shootdown SGI on the BSP's
                // redistributor (an INTID < 32, so `enable_ppi` configures it in
                // the SGI_base frame). Harmless on 1-vCPU (no one ever sends it),
                // invisible to the syscall-trace golden.
                crate::gicv3::enable_ppi(crate::shootdown::SHOOTDOWN_SGI);
            }
            crate::gicv3::GicVersion::V2 => {
                crate::gic::init();
                crate::gic::enable_interrupt(crate::gic::TIMER_INTID);
                // P4·SMP·S4c: enable the shootdown SGI in the (per-CPU-banked)
                // distributor SGI registers.
                crate::gic::enable_interrupt(crate::shootdown::SHOOTDOWN_SGI);
            }
        }
    }
    // Non-golden-shape announce (NOT the oracle's `[pid N] syscall` shape).
    crate::kprintln!("{}", gic_version.announce());

    // Build the console handle the safe kernel will use for the banner.
    // SAFETY: same PL011 the global console drives; on a single core the safe
    // kernel and the IRQ path do not run concurrently while it holds this.
    let console = unsafe { crate::uart::Pl011::new(crate::uart::PL011_BASE) };

    (
        Aarch64 {
            console,
            heap_bytes,
        },
        heap_bytes,
    )
}

/// Safe entry the kernel calls to load and run the embedded EL0 user program.
///
/// Loads the embedded ELF, installs EL0-accessible mappings, drops to EL0, and
/// (after the user `exit`s through our syscall layer) returns into
/// [`user_finished`]. This never returns to the caller; the exit path diverts
/// to `user_finished` which powers the machine off.
pub fn run_user() -> ! {
    crate::user::run_user()
}

/// Kernel continuation after the user program has exited. The Frame's syscall
/// layer restored the kernel-only identity map and printed the exit line; here
/// we emit the success banner and power the machine off, completing the run.
pub fn user_finished() -> ! {
    crate::kprintln!("kernel: OK");
    power_off()
}

/// Start the periodic timer and unmask IRQs, then idle forever waiting for
/// interrupts. The timer ISR powers the machine off after the demo ticks, so
/// in practice this never returns to the caller.
///
/// This is a **safe** wrapper the (unsafe-forbidding) kernel can call: all the
/// dangerous work was vetted during [`bringup`]; here we only arm a configured
/// timer and unmask IRQs.
pub fn start_timer_and_idle(period_ms: u64) -> ! {
    // SAFETY: GIC + timer PPI were configured in `bringup`; this is the single
    // place that arms the periodic timer and unmasks IRQ delivery.
    unsafe {
        crate::timer::init(period_ms);
        DAIF.write(DAIF::I::Unmasked);
    }
    loop {
        asm::wfi();
    }
}

// Keep `ConsoleWrite` in scope so trait methods on the console resolve.
const _: fn(&mut crate::uart::Pl011, u8) = <crate::uart::Pl011 as ConsoleWrite>::write_byte;
