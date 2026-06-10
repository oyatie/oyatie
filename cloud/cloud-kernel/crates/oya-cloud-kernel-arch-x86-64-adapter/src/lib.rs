//! # `arch-x86_64` — the x86_64 Frame backend
//!
//! Part of the **Frame**: the tiny, audited, `unsafe` core that actually
//! touches hardware. Everything in the safe kernel above is written against the
//! [`hal`] traits; this crate is one of the few places allowed to write
//! `unsafe` (boot assembly, descriptor tables, port I/O).
//!
//! This backend implements a full minimal bring-up under
//! `qemu-system-x86_64 -kernel <elf>` to parity with aarch64:
//!
//!   * [`boot`] — PVH (`XEN_ELFNOTE_PHYS32_ENTRY`) entry + a 32-bit ->
//!     long-mode trampoline (GDT, identity page tables, PAE/LME/PG),
//!   * the 16550 UART console at port 0x3F8 (`console`),
//!   * a GDT+TSS with a double-fault IST stack (`gdt`),
//!   * an IDT with exception handlers and the timer IRQ (`interrupts`),
//!   * a global heap allocator over a static region (in `arch`),
//!   * the 8259 PIC + 8254 PIT delivering periodic IRQs (`interrupts`,
//!     `timer`), and clean `isa-debug-exit` self-termination (`exit`).
//!
//! It exposes the safe [`hal::Arch`] surface (via [`X86_64`]) plus safe
//! [`boot`] / [`start_timer_and_idle`] entry points the
//! `#![forbid(unsafe_code)]` kernel drives.
//!
//! `unsafe` is *allowed* here (this is the Frame), kept minimal and documented
//! at every site.
#![no_std]
// The `x86-interrupt` calling convention (for IDT handlers) is still unstable;
// enable it. Used only inside this Frame crate.
#![cfg_attr(target_arch = "x86_64", feature(abi_x86_interrupt))]
// This crate is the Frame; it is deliberately NOT `#![forbid(unsafe_code)]`.
// We instead require every unsafe block to carry a safety comment, and require
// explicit `unsafe` inside `unsafe fn`.
#![cfg_attr(target_arch = "x86_64", deny(unsafe_op_in_unsafe_fn))]
// The boot trampoline uses lowercase `static mut` page-table names matched by
// the asm (`boot_pml4` etc.); allow the non-uppercase static convention there.
#![cfg_attr(target_arch = "x86_64", allow(non_upper_case_globals))]

// The process model owns its page tables, frames, and process table on the
// kernel heap (the arch backend installs the `#[global_allocator]`), so it needs
// `alloc`. Gated to x86_64 like the rest of the Frame backend.
#[cfg(target_arch = "x86_64")]
extern crate alloc;

#[cfg(target_arch = "x86_64")]
mod boot;
#[cfg(target_arch = "x86_64")]
pub mod console;
#[cfg(target_arch = "x86_64")]
mod cr4;
#[cfg(target_arch = "x86_64")]
mod exit;
#[cfg(target_arch = "x86_64")]
mod gdt;
#[cfg(target_arch = "x86_64")]
mod interrupts;
// The x86_64 process model: per-process address spaces, the scheduler, the
// register context, and context switching. Mirrors `arch-aarch64::process`.
#[cfg(target_arch = "x86_64")]
mod process;
// P4·SMP·S3: secondary-CPU (AP) bring-up via the .code16 SIPI trampoline +
// INIT-SIPI-SIPI through the x2APIC ICR + MADT enumeration. All AP asm/MSR
// unsafe lives here in the Frame; the safe kernel sees only the `hal::smp::Smp`
// seam this implements on `X86_64`.
#[cfg(target_arch = "x86_64")]
mod smp;
// P4·SMP·S4c: cross-CPU TLB shootdown — wires the loom-verified H1 protocol to a
// fixed-vector x2APIC IPI + a per-CPU CR3 reload. All ICR/CR3 unsafe is here.
#[cfg(target_arch = "x86_64")]
mod shootdown;
// P4·SMP·S4b: the reschedule IPI — wakes an idle CPU when work lands on its run
// queue (a fork/wake placement or a steal), via a fixed-vector x2APIC IPI
// (RESCHED_VECTOR = 0xF0). All ICR unsafe stays in `apic::send_fixed_ipi`.
#[cfg(target_arch = "x86_64")]
mod reschedule;
// P4·SMP·S2 (Part A.2): the general per-CPU `UnsafeCell` primitive S4's
// `LocalRunQueue` needs. Defined now (sound shape on record under the S2 gate),
// wired into a real run-queue in S4 — hence `dead_code` until then.
#[allow(dead_code)]
mod percpu_local;
#[cfg(target_arch = "x86_64")]
mod timer;
// P4: x2APIC local-APIC + LVT timer (the modern interrupt chip), gated into the
// 3-tier timer model alongside the legacy 8259 PIC + 8254 PIT. All x2APIC MSR /
// PIT-calibration unsafe lives here, in the Frame.
#[cfg(target_arch = "x86_64")]
mod apic;
// P3 timekeeping: TSC-backed monotonic/realtime clock, PIT-calibrated at boot.
// The only new unsafe is the `rdtsc` read + the PIT calibration port I/O; all
// counter->ns math is the shared zero-unsafe `user_layout::timekeep`.
#[cfg(target_arch = "x86_64")]
mod timekeeping;
#[cfg(target_arch = "x86_64")]
mod user;

#[cfg(target_arch = "x86_64")]
mod arch;
#[cfg(target_arch = "x86_64")]
pub use arch::{boot, run_user, start_timer_and_idle, X86_64};

// Additive P0 HAL-reshape floor backings (NOT on the boot path). Re-expresses
// this backend's existing hardware code (8259 PIC, 8254 PIT, long-mode CR3/PML4
// paging, the IDT interrupt frame, and CPUID feature leaves) as the new
// `hal::{cpu,mm,irq,time}` capability shapes — the real `impl hal::sealed::Sealed`
// + `impl hal::<trait>` blocks, now that `hal` opens its `sealed` module to its
// workspace backends. Mirrors `arch-aarch64::hal_caps`. Not yet called by the
// kernel; the switchover is a later slice. See the module docs for details.
#[cfg(target_arch = "x86_64")]
pub mod hal_caps;
