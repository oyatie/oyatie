//! Buck2 host test harness for the crate's pure user-layout math.
//!
//! The buck2 analogue of the cargo `tests-host` harness (kept under the
//! arch-aarch64 adapter): we `include!` the *exact same* sources this crate
//! compiles — files that carry **no inner attributes** (the crate-level
//! `#![no_std]` lives in `src/lib.rs`) — so their `#[cfg(test)]` modules run
//! on the host with the normal libtest harness (no copy/drift). Co-located in
//! THIS package because buck2 source globs cannot cross package boundaries the
//! way the cargo harness's `../../../` include! does.
//!
//! Run: `buck2 test //cloud/cloud-kernel/crates/oya-cloud-kernel-user-layout-kernel:host-tests`

#[allow(dead_code)] // the kernel exercises some helpers the tests do not
mod user_layout {
    include!("../src/layout.rs");
}

#[allow(dead_code)]
mod user_layout_signal {
    include!("../src/signal.rs");
}

#[allow(dead_code)]
mod user_layout_timekeep {
    include!("../src/timekeep.rs");
}

#[allow(dead_code)]
mod user_layout_vfs {
    include!("../src/vfs.rs");
}

#[allow(dead_code)]
mod user_layout_netlink {
    include!("../src/netlink.rs");
}

#[allow(dead_code)]
mod user_layout_procinfo {
    include!("../src/procinfo.rs");
}
