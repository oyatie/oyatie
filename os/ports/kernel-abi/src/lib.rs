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
//! ## Scope (deliberately small, and narrower than "the network")
//!
//! [`net`] models the **rtnetlink configuration** operations — link up, address
//! assign, route install, address read-back, operstate read. It does *not* yet
//! model every network operation PID 1 issues, and the seam is therefore
//! **partial**: swapping the kernel substrate today still requires touching PID
//! 1, not only the adapter binding.
//!
//! Specifically, `os-init-app` still reaches Linux directly for:
//!
//! - interface discovery and readiness — `read_dir("/sys/class/net")` and the
//!   bounded `/sys/class/net/<iface>` poll,
//! - the MAC read from `/sys/class/net/<iface>/address`,
//! - link-local IPv6 discovery by parsing `/proc/net/if_inet6`,
//! - `libc::if_nametoindex`, the `libc::setsockopt` calls that bind a socket to
//!   an interface, and the `ioctl(SIOCSIFFLAGS)` link-up fallback,
//! - the `/proc/version` and `/proc/sys/kernel/hostname` boot reads.
//!
//! Those are listed rather than waved at so no consumer mistakes this for a
//! completed swappability boundary. Each arrives in the port when a caller is
//! routed through it — the operations that are here are here because they have
//! callers, and empty traits would claim a seam that has not been drawn.
//!
//! Everything else Talos eventually needs from a kernel — mount/`pivot_root`,
//! cgroups v2, kexec, EFI variables, dm-crypt, block enumeration, process
//! fork/exec/reap, kernel module loading — is **not** modelled here either.
//!
//! Note that `os-kernel`'s [`os`](os_kernel::os) module already ports a
//! different slice of the same boundary (`Clock`, `FileSystem`,
//! `CommandExecutor`, `SyscallProvider` — hostname, mount, reboot) with
//! in-memory fakes. This crate does not duplicate it; it covers the surface
//! that module leaves raw.

extern crate alloc;

pub mod net;

pub use net::{InMemoryKernelNet, KernelNet, RouteOrigin};
