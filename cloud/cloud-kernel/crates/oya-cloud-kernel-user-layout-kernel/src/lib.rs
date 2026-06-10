//! # `user_layout` — arch-neutral, pure EL0 user-layout math
//!
//! This crate is the **single source of truth** for the parts of EL0 user
//! bring-up that are *pure functions* of their inputs and therefore identical
//! across architectures: the initial process-stack (argc/argv/envp/auxv)
//! builder, the VMSAv8-64 descriptor bit layout + builders, user-pointer range
//! validation, the on-demand frame-pool index bookkeeping, the TLS variant-I
//! placement, and the timer-sleep (`timespec` -> counter-cycles) arithmetic.
//!
//! It is `no_std` and depends on **nothing** outside `core`, on purpose: each
//! arch Frame backend (`arch-aarch64` today, `arch-x86_64` / riscv64 later)
//! depends on this crate so the math has exactly one home, and a tiny
//! out-of-workspace host harness `include!`s the same source to unit-test it
//! with the normal libtest harness (see `crates/arch-aarch64/tests-host/`).
//!
//! The actual math lives in `layout.rs`, which is `include!`d here. That file
//! carries **no inner attributes** so the host harness can `include!` it inside
//! a plain `mod` body (where `#![...]` is illegal); the crate-level `#![no_std]`
//! is supplied below instead. `cfg(not(test))` keeps `std` available when this
//! crate itself is built as a host test target, while staying `no_std` for the
//! bare-metal arch builds that consume it.
#![cfg_attr(not(test), no_std)]

include!("layout.rs");

/// Pure POSIX-signal math: signal numbers, sigset bit ops, the
/// `SigAction`/`SignalState` PODs, the default-action classifier, and the
/// shared signal-frame offset constants + stack-alignment arithmetic the two
/// arch backends use for delivery and `rt_sigreturn`. Like `layout.rs` this is
/// `include!`d (no inner attributes) so the same source unit-tests on the host.
/// Zero `unsafe` — keeps the safe-kernel TCB ratchet (`check-tcb.sh`) green.
pub mod signal {
    include!("signal.rs");
}

/// Pure timekeeping math: the counter->nanoseconds fixed-point scale
/// (`calc_mult_shift` / `cycles_to_ns`), the `timespec`/`timeval` field splits,
/// the fixed wall-clock epoch offset, and the `TimekeeperData` POD. Like
/// `layout.rs`/`signal.rs` this is `include!`d (no inner attributes) so the same
/// source unit-tests on the host. Zero `unsafe` — the arch backends supply only
/// the raw counter read (`CNTPCT_EL0` / `rdtsc`). Keeps `check-tcb.sh` green.
pub mod timekeep {
    include!("timekeep.rs");
}

/// Pure in-RAM VFS: an index-slab inode/dentry tree (rooted at `/`,
/// pre-populating `/dev/console` + `/dev/null`), a multi-component absolute-path
/// walker (`/`, `.`, `..`, trailing slash, empty/double-slash components), an
/// idempotent `mkdir -p`, and a mount table. Like `layout.rs`/`signal.rs`/
/// `timekeep.rs` this is `include!`d (no inner attributes) so the same source
/// unit-tests on the host. Zero `unsafe` — the arch backends supply only the
/// per-arch `with_vfs` accessor (one `unsafe` block, in the Frame) + the
/// user-string copy. The M2 tmpfs/`mount(2)` shim (Slice 1) is built on this.
/// Uses `alloc` (`Vec`/`String`); the kernel registers a global allocator.
pub mod vfs {
    include!("vfs.rs");
}

/// Pure process-info / libc-init layout math: the byte-exact `struct utsname`
/// serializer (`uname`), the `umask` swap/mask, and the `clock_getres`
/// resolution value. Every real glibc/musl binary queries these during init;
/// their on-wire answer is a pure function of the inputs and therefore identical
/// across arches. Like `layout.rs`/`signal.rs`/`timekeep.rs`/`vfs.rs` this is
/// `include!`d (no inner attributes) so the same source unit-tests on the host.
/// Zero `unsafe` — the arch backends supply only the machine string + the
/// PAN/SMAP-bracketed user copy. Keeps `check-tcb.sh` green.
pub mod procinfo {
    include!("procinfo.rs");
}

/// Pure, arch-neutral `AF_NETLINK`/`NETLINK_ROUTE` wire logic for the M2 network
/// slice: parse an outbound `RTM_GETLINK` dump request (extracting the
/// `nlmsg_seq`/`nlmsg_pid` to echo) and build the dump RESPONSE (a single
/// well-formed `NLMSG_DONE` = the empty link set the real, unmodified talos-init
/// link-status snapshot converges on). Like `layout.rs`/`signal.rs`/`timekeep.rs`/
/// `vfs.rs` this is `include!`d (no inner attributes) so the same source
/// unit-tests on the host. Zero `unsafe` — the arch backends supply only the
/// per-fd response buffer + the SMAP/PAN-bracketed flat byte copies. Uses
/// `alloc` (`Vec<u8>`). Keeps `check-tcb.sh` green.
pub mod netlink {
    include!("netlink.rs");
}
