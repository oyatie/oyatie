//! # `arch-aarch64` — the aarch64 Frame backend
//!
//! This crate is part of the **Frame**: the tiny, audited, `unsafe` core that
//! actually touches hardware. Everything in the safe kernel above is written
//! against the [`hal`] traits; this crate is one of the few places in the
//! whole tree allowed to write `unsafe` (boot assembly, register access).
//!
//! This backend implements a full minimal bring-up on the QEMU `virt` machine:
//!
//!   * [`boot`] (assembly `_start` -> `rust_start` -> `kmain`),
//!   * the PL011 UART console (`uart`, `console`),
//!   * exception vectors with a fault-decoding panic path (`exceptions`),
//!   * an identity-mapped MMU with caches enabled (`mmu`),
//!   * a global heap allocator over a static region (in `arch`),
//!   * a GICv2 + EL1 physical generic timer that delivers periodic IRQs
//!     (`gic`, `timer`), and clean PSCI `SYSTEM_OFF` self-termination.
//!
//! It exposes the safe [`hal::Arch`] surface (via [`Aarch64`]) plus safe
//! [`boot`] / [`start_timer_and_idle`] entry points the `#![forbid(unsafe_code)]`
//! kernel drives.
//!
//! `unsafe` is *allowed* here (this is the Frame), but kept to a minimum and
//! documented at every site.
#![no_std]
// NOTE: this crate is deliberately NOT `#![forbid(unsafe_code)]`. It is the
// Frame. We instead require every unsafe block to carry a safety comment.
#![cfg_attr(target_arch = "aarch64", deny(unsafe_op_in_unsafe_fn))]

// The process model owns its page tables, frames, and process table on the
// kernel heap (the arch backend installs the `#[global_allocator]`), so it needs
// `alloc`. Gated to aarch64 because the process model is an aarch64 deliverable.
#[cfg(target_arch = "aarch64")]
extern crate alloc;

#[cfg(target_arch = "aarch64")]
mod boot;
#[cfg(target_arch = "aarch64")]
pub mod console;
#[cfg(target_arch = "aarch64")]
mod exceptions;
#[cfg(target_arch = "aarch64")]
mod gic;
#[cfg(target_arch = "aarch64")]
mod gicv3;
#[cfg(target_arch = "aarch64")]
mod mmu;
#[cfg(target_arch = "aarch64")]
mod percpu;
// P4·SMP·S2 (Part A.2): the general per-CPU `UnsafeCell` primitive S4's
// `LocalRunQueue` needs. Defined now (sound shape on record under the S2 gate),
// wired into a real run-queue in S4 — hence `dead_code` until then.
#[allow(dead_code)]
mod percpu_local;
#[cfg(target_arch = "aarch64")]
mod process;
// P4·SMP·S4c: cross-CPU TLB shootdown — wires the loom-verified H1 protocol to
// a GIC SGI + per-CPU `tlbi`. All SGI/sysreg unsafe lives here in the Frame.
#[cfg(target_arch = "aarch64")]
mod shootdown;
// P4·SMP·S4b: the reschedule IPI — wakes an idle CPU when work lands on its run
// queue (a fork/wake placement or a steal), via a GIC SGI (RESCHED_SGI = INTID
// 1). All SGI/sysreg unsafe stays in `gicv3::send_sgi`.
#[cfg(target_arch = "aarch64")]
mod reschedule;
// P4·SMP·S3: secondary-CPU (AP) bring-up via PSCI CPU_ON + the DTB /cpus walk.
// All AP asm/hvc/sysreg unsafe lives here in the Frame; the safe kernel sees
// only the `hal::smp::Smp` seam this implements on `Aarch64`.
#[cfg(target_arch = "aarch64")]
mod smp;
#[cfg(target_arch = "aarch64")]
mod timer;
#[cfg(target_arch = "aarch64")]
mod uart;
#[cfg(target_arch = "aarch64")]
mod user;
// Pure, hardware-independent layout/allocator/stack-builder math used by
// `user`/`process`. This used to be an inline `mod user_layout` in this crate;
// it now lives in the standalone, dependency-free, `no_std` `user_layout` crate
// so x86_64/riscv64 can reuse the **single source of truth**. We re-alias it as
// `crate::user_layout` so the existing `use crate::user_layout::{..}` call sites
// in `user.rs`/`process.rs` are unchanged. The out-of-workspace host harness in
// `tests-host/` `include!`s that crate's source directly to unit-test it.
#[cfg(target_arch = "aarch64")]
pub use user_layout;
// The in-RAM VFS and the netlink wire codec are independent kernel subsystems,
// each its own crate. Re-aliased the same way so `user.rs`/`process.rs` reach
// them at `crate::vfs` / `crate::netlink`.
#[cfg(target_arch = "aarch64")]
pub use netlink;
#[cfg(target_arch = "aarch64")]
pub use vfs;

#[cfg(target_arch = "aarch64")]
mod arch;
#[cfg(target_arch = "aarch64")]
pub use arch::{boot, run_user, start_timer_and_idle, Aarch64};

// Additive P0 HAL-reshape floor backings (NOT on the boot path). Re-expresses
// this backend's existing hardware code as the new `hal::{cpu,mm,irq,time}`
// capability shapes. See the module docs for the sealed-trait finding: most of
// these are blocked from being literal `impl Trait` lines until `hal` opens its
// private `sealed` module to its arch backends; the floor logic is implemented
// and compiled regardless. `PagingConsts` (unsealed) IS a real trait impl.
#[cfg(target_arch = "aarch64")]
pub mod hal_caps;
