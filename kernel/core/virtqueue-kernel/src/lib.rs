//! # `virtqueue` — arch-neutral, pure split-virtqueue ring arithmetic
//!
//! VIRTIO 1.x "split virtqueue" (§2.7): the three rings' byte layout (descriptor
//! table / available ring / used ring), the per-element byte offsets, the
//! available/used index publish + consume, and the intrusive free-descriptor
//! chain.
//!
//! Zero `unsafe` — the arch Frame does the volatile MMIO reads/writes, the
//! DMA-coherent physaddr translation and the memory barriers; this crate answers
//! only "what byte offset?" / "which slot?". `core`-only, no `alloc`.
//!
//! The arithmetic lives in `virtqueue.rs`, which is `include!`d here. That file
//! carries **no inner attributes** so the host harness can `include!` it inside a
//! plain `mod` body (where `#![...]` is illegal); the crate-level `#![no_std]` is
//! supplied below instead. `cfg(not(test))` keeps `std` available when this crate
//! itself is built as a host test target, while staying `no_std` for the
//! bare-metal arch builds that consume it.
#![cfg_attr(not(test), no_std)]

include!("virtqueue.rs");
