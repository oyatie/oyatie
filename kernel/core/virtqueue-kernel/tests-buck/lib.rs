//! Buck2 host test harness for the crate's pure split-virtqueue arithmetic.
//!
//! We `include!` the *exact same* `virtqueue.rs` this crate compiles — it carries
//! **no inner attributes** (the crate-level `#![no_std]` lives in `src/lib.rs`) —
//! so its `#[cfg(test)] mod virtqueue_tests` runs on the host with the normal
//! libtest harness (no copy/drift). Co-located in THIS package because buck2
//! source globs cannot cross package boundaries the way the cargo harness's
//! `../../../` include! does.
//!
//! Run: `buck2 test //kernel/core/virtqueue-kernel:host-tests`

#[allow(dead_code)] // the kernel exercises some helpers the tests do not
mod virtqueue {
    include!("../src/virtqueue.rs");
}
