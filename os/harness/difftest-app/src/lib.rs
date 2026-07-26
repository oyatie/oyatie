//! Differential test harness crate.
//!
//! This crate carries no production code of its own. Its purpose is the
//! integration tests under `tests/`, which read the Go oracle TSV vectors in
//! `vectors/` and assert that the operating-system Rust port (`talos-core`,
//! `talos-network`) reproduces upstream Talos behavior byte-for-byte.
