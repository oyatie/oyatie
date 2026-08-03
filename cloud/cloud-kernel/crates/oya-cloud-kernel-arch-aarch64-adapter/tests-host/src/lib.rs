//! Host harness that unit-tests the Frame's pure user-layout math.
//!
//! We `include!` the *exact same* `layout.rs` the `user_layout` crate compiles
//! (the single source of truth shared by every arch Frame backend), so the
//! tests verify the real production code (no copy/drift). That file carries
//! **no inner attributes** (its crate-level `#![no_std]` lives in the crate's
//! `lib.rs`), so it is legal to `include!` it inside this plain `mod` body.
//!
//! Run: `cargo test --manifest-path crates/arch-aarch64/tests-host/Cargo.toml`

#[allow(dead_code)] // the kernel exercises some helpers the tests do not
mod user_layout {
    include!("../../../../../../kernel/core/user-layout-kernel/src/layout.rs");
}

/// The pure POSIX-signal math (sigset ops, SigAction/SignalState, default-action
/// classifier, signal-frame offset constants + stack-alignment arithmetic). We
/// `include!` the *exact same* `signal.rs` the `user_layout` crate compiles, so
/// the `#[cfg(test)] mod signal_tests` inside it runs here on the host with the
/// normal libtest harness (no copy/drift).
#[allow(dead_code)]
mod user_layout_signal {
    include!("../../../../../../kernel/core/user-layout-kernel/src/signal.rs");
}

/// The pure timekeeping math (`calc_mult_shift` / `cycles_to_ns` / the
/// `timespec`/`timeval` splits + the wall-clock epoch offset). We `include!` the
/// *exact same* `timekeep.rs` the `user_layout` crate compiles, so its
/// `#[cfg(test)] mod timekeep_tests` runs here on the host (no copy/drift): it
/// checks `cycles_to_ns(freq, mult, shift) ~= 1e9` exhaustively over the spec's
/// frequency set and the u128 no-overflow property.
#[allow(dead_code)]
mod user_layout_timekeep {
    include!("../../../../../../kernel/core/user-layout-kernel/src/timekeep.rs");
}

/// The pure in-RAM VFS (index-slab inode/dentry tree, the multi-component path
/// walker, `mkdir -p`, and the mount table). We `include!` the *exact same*
/// `vfs.rs` the `user_layout` crate compiles, so its `#[cfg(test)] mod
/// vfs_tests` runs here on the host with the normal libtest harness (no
/// copy/drift): it checks the walker edge cases (`/`, `.`, `..`, trailing
/// slash, empty/double-slash, ENOENT/ENOTDIR) and the mount-record/idempotency
/// behaviour the M2 Slice-1 `SYS_MOUNT` shim relies on.
#[allow(dead_code)]
mod user_layout_vfs {
    include!("../../../../../../kernel/core/vfs-kernel/src/vfs.rs");
}

/// The pure `AF_NETLINK`/`RTM_GETLINK` wire logic (request parse + zero-link
/// `NLMSG_DONE` response build). We `include!` the *exact same* `netlink.rs` the
/// `user_layout` crate compiles, so its `#[cfg(test)] mod netlink_tests` runs
/// here on the host with the normal libtest harness (no copy/drift): it checks
/// the request parse, the exact 16-byte `NLMSG_DONE` response bytes, the
/// seq/port echo, truncation handling, and — via faithful copies of talos's
/// `dump_chunk_done_or_error`/`parse_link_dump` — that the real consumer accepts
/// the response as a done-with-zero-links dump (the M2 Network checkpoint).
#[allow(dead_code)]
mod user_layout_netlink {
    include!("../../../../../../kernel/core/netlink-kernel/src/netlink.rs");
}

/// The pure process-info / libc-init layout math (the byte-exact `struct utsname`
/// serializer, the `umask` swap/mask, and the `clock_getres` resolution value).
/// We `include!` the *exact same* `procinfo.rs` the `user_layout` crate
/// compiles, so its `#[cfg(test)] mod tests` runs here on the host with the
/// normal libtest harness (no copy/drift): it checks the six NUL-padded 65-byte
/// `utsname` fields, the per-arch machine string, the umask masking, and the
/// 1-nanosecond `clock_getres` value the WAVE-1 libc-init syscalls return.
#[allow(dead_code)]
mod user_layout_procinfo {
    include!("../../../../../../kernel/core/user-layout-kernel/src/procinfo.rs");
}
