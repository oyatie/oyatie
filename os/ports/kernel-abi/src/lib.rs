#![cfg_attr(not(test), no_std)]
//! # os-kernel-abi
//!
//! The kernel-ABI **port** for `os/`: the operations `os/` crates require from
//! a kernel, expressed as traits.
//!
//! ## Why this crate exists
//!
//! The Talos-in-Rust port targets the Linux ABI. Today the raw-ABI calls
//! (`libc`, netlink sockets, `/sys` reads) live inside domain crates, which
//! hard-wires Linux into the domain layer. If every kernel interaction goes
//! through this port instead, swapping the kernel substrate is an adapter swap
//! behind an unchanged interface.
//!
//! ## The design test
//!
//! *Would this trait change at cutover?* If a method mentions a Linux
//! implementation detail (an `RTPROT_*` number, a `/sys` path, an `ioctl`
//! request code) rather than an operation, it is wrong. Those encodings belong
//! in the adapter.
//!
//! ## Scope (deliberately small)
//!
//! Only the operations the two `os/` crates that hold raw-ABI dependencies
//! actually issue are modelled. See [`net`]. Everything else Talos eventually
//! needs from a kernel — mount/`pivot_root`, cgroups v2, kexec, EFI variables,
//! dm-crypt, block enumeration, process fork/exec/reap, kernel module loading —
//! is **not** modelled here. Empty traits would claim a seam that has not been
//! drawn; the operations arrive when a caller is routed through them.
//!
//! Note that `os-kernel`'s [`os`](os_kernel::os) module already ports a
//! different slice of the same boundary (`Clock`, `FileSystem`,
//! `CommandExecutor`, `SyscallProvider` — hostname, mount, reboot) with
//! in-memory fakes. This crate does not duplicate it; it covers the surface
//! that module leaves raw.

extern crate alloc;

pub mod net;

pub use net::{InMemoryKernelNet, KernelNet, RouteOrigin};
