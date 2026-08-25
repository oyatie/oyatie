//! Cloud Region/AZ/Cell taxonomy kernel.
//!
//! This crate owns the catalog and routing behavior around Cell's shared
//! location contract. Region, AZ, and cell identities are defined by the
//! `cell-location` provider port and re-exported for compatibility.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod catalog;
mod entities;
mod model;
mod validation;

pub use cell_location::{AzCode, CellId, CellLocationError, RegionCode};
pub use model::*;

#[cfg(test)]
mod tests;
