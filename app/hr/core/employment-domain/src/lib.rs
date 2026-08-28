//! HR employment domain foundation.
//!
//! This crate owns pure HR invariants for employee and employment records:
//! legal-entity-scoped employees, audit-backed lifecycle events, and
//! Korea-first labor-compliance threshold obligations. Tenant RBAC view and
//! Tenant RBAC view remain product-surface metadata; this crate is not an enterprise
//! platform boundary. It does not perform storage, workflow dispatch, payroll
//! derivation, or regulator filing I/O.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// panic assertions to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

include!(concat!(env!("OUT_DIR"), "/lib.generated.rs"));

#[cfg(test)]
include!(concat!(env!("OUT_DIR"), "/tests.generated.rs"));
