//! Reindeer dev-dependency forcing crate (move-22a / ADR-0554 affected-set).
//!
//! See `Cargo.toml` for the full rationale. In short: reindeer does not vendor
//! dev-only dependencies of workspace members, so the test-only crates the
//! intelligence test suite needs (`httpmock`, `proptest`) must be promoted to
//! normal dependencies in one first-party crate to make reindeer emit their
//! `third-party/BUCK` targets. This crate is never linked into a shipped binary;
//! the re-exports below simply ensure the linker/compiler considers the crates
//! reachable from a normal (non-dev) edge.
#![forbid(unsafe_code)]

pub use httpmock;
pub use proptest;
