//! # `vfs` — arch-neutral, pure in-RAM VFS
//!
//! An index-slab inode/dentry tree (rooted at `/`, pre-populating `/dev/console`
//! + `/dev/null`), a multi-component absolute-path walker (`/`, `.`, `..`,
//! trailing slash, empty/double-slash components), an idempotent `mkdir -p`, and
//! a mount table. The M2 tmpfs/`mount(2)` shim (Slice 1) is built on this.
//!
//! Zero `unsafe` — the arch backends supply only the per-arch `with_vfs` accessor
//! (one `unsafe` block, in the Frame) + the user-string copy. Uses `alloc`
//! (`Vec`/`String`); the kernel registers a global allocator.
//!
//! The math lives in `vfs.rs`, which is `include!`d here. That file carries **no
//! inner attributes** so the host harnesses can `include!` it inside a plain `mod`
//! body (where `#![...]` is illegal); the crate-level `#![no_std]` is supplied
//! below instead. `cfg(not(test))` keeps `std` available when this crate itself is
//! built as a host test target, while staying `no_std` for the bare-metal arch
//! builds that consume it.
#![cfg_attr(not(test), no_std)]

include!("vfs.rs");
