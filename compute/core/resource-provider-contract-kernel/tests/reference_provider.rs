//! The in-crate REFERENCE resource provider: a deterministic in-memory
//! implementation of [`ResourceProvider`] that exists to prove the harness
//! itself (it is the harness fixture — test infrastructure, not a product
//! artifact). Three deliberately nonconformant wrappers prove the harness
//! actually catches violations (masterplan no-false-green rule).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[path = "reference_provider/mod.rs"]
mod reference_provider;
