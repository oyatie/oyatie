//! # `netlink` — arch-neutral, pure `AF_NETLINK`/`NETLINK_ROUTE` wire logic
//!
//! Parse an outbound `RTM_GETLINK` dump request (extracting the
//! `nlmsg_seq`/`nlmsg_pid` to echo) and build the dump RESPONSE (a single
//! well-formed `NLMSG_DONE` = the empty link set the real, unmodified talos-init
//! link-status snapshot converges on).
//!
//! Zero `unsafe` — the arch backends supply only the per-fd response buffer + the
//! SMAP/PAN-bracketed flat byte copies. Uses `alloc` (`Vec<u8>`).
//!
//! The codec lives in `netlink.rs`, which is `include!`d here. That file carries
//! **no inner attributes** so the host harnesses can `include!` it inside a plain
//! `mod` body (where `#![...]` is illegal); the crate-level `#![no_std]` is
//! supplied below instead. `cfg(not(test))` keeps `std` available when this crate
//! itself is built as a host test target, while staying `no_std` for the
//! bare-metal arch builds that consume it.
#![cfg_attr(not(test), no_std)]

include!("netlink.rs");
