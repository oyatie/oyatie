//! Reindeer dev-dependency forcing crate (move-22a / ADR-0554 affected-set).
//!
//! See `Cargo.toml` for the full rationale. In short: reindeer does not vendor
//! dev-only dependencies of workspace members, so the test-only crates the
//! intelligence test suite needs (`proptest`) must be promoted to normal
//! dependencies in one first-party crate to make reindeer emit their
//! `third-party/BUCK` targets. This crate is never linked into a shipped binary;
//! the re-export below simply ensures the linker/compiler considers the crate
//! reachable from a normal (non-dev) edge.
//!
//! `httpmock` was forced here until ADR-0709 D-6 Rule 2 retired it; the
//! first-party `scripted-http-server` crate beside this one replaces it and needs
//! no forcing, because reindeer vendors only third-party crates.
#![forbid(unsafe_code)]

pub use proptest;
