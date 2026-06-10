//! # `ksync` — the kernel's concurrency substrate
//!
//! `ksync` vendors the 14 Phase-2 concurrency / memory primitives that were
//! developed and verified as standalone crates (under `port/crates_phase2/`)
//! into a single `no_std` crate, so the kernel has one dependency for its
//! locking, lock-free, and bump/slab allocation building blocks.
//!
//! ## Verification provenance
//!
//! Unlike the Phase-0/1 leaf primitives, a synchronization primitive cannot be
//! verified by differential testing against a C2Rust transpile — you cannot
//! transpile away a race. Each primitive below was instead verified with:
//!
//! - **loom** — exhaustive concurrency model-checking of every legal
//!   interleaving of its atomics (`RUSTFLAGS="--cfg loom" cargo test -p ksync`);
//! - **Miri** — undefined-behaviour / data-race detection on the real atomics
//!   (`cargo +nightly miri test -p ksync`);
//! - **invariant / behavioural tests** under the normal host build
//!   (`cargo test -p ksync`).
//!
//! The module sources are copied **verbatim** from the verified crates: the
//! `#[cfg(loom)]` / `#[cfg(not(loom))]` gating, the `SAFETY:` arguments, and the
//! loom/Miri/behavioural tests are preserved unaltered. Only the per-crate
//! crate-level attributes (`#![cfg_attr(not(any(test, loom)), no_std)]` and the
//! `extern crate alloc;` declaration) were lifted up to this crate root, since
//! those must live on the crate and not inside a module.
//!
//! ## `no_std`
//!
//! This crate is `#![no_std]` for the production / Miri build. The `test` and
//! `loom` builds run on the host and therefore link `std` (loom and libtest
//! both require it); the `cfg_attr` below encodes exactly that.
#![cfg_attr(not(any(test, loom)), no_std)]

// Several primitives (the lock-free stacks/queues and the RCU cell) allocate via
// `alloc::boxed::Box` / `alloc::sync::Arc` on the production / Miri path. Declare
// `alloc` once here at the crate root so those `use alloc::...` imports resolve.
// Under `--cfg loom` the build is a `std` host build that uses `loom::sync::*` /
// `std::*` instead, so `alloc` is not needed (and `extern crate alloc;` while
// `std` is linked would be redundant) — hence the `cfg(not(loom))` gate, matching
// the original crates.
#[cfg(not(loom))]
extern crate alloc;

// --- Locks ---------------------------------------------------------------
pub mod spinlock;
pub mod rwlock;
pub mod rwsem;
pub mod seqlock;

// --- Synchronization / signalling ---------------------------------------
pub mod completion;
pub mod rcu_cell;
// P4·SMP·S2 (H1): the cross-CPU TLB-shootdown 3-step protocol model. As of
// S4c it is INSTANTIATED by the per-arch Frame (`process.rs` wires a real
// `Shootdown` to the shootdown IPI + per-CPU `invlpg`/`tlbi` via
// `poll_and_invalidate`); the loom models (below) still exhaustively check the
// sender/receiver ordering on the host.
pub mod shootdown;

// --- Ring buffers / queues ----------------------------------------------
pub mod spsc_ring;
pub mod mpsc_ring;
pub mod mpmc_queue;

// P4·SMP·S4b (H4): the bounded Chase-Lev work-stealing deque — the per-CPU
// run-queue's lock-free stealable fast path (owner push/pop the bottom, other
// CPUs steal the top). NET-NEW concurrency primitive; loom-gated by the H4
// `loom_cl_deque_*` models (run by scripts/run-loom.sh) before it is allowed to
// race a second CPU, the same C9 rule that gated the S2 primitives.
pub mod cl_deque;

// --- Lock-free stacks (with reclamation schemes) ------------------------
pub mod treiber_stack;
pub mod hazard_stack;
pub mod ebr_stack;

// --- Allocators ----------------------------------------------------------
pub mod bump_alloc;
pub mod slab_alloc;
