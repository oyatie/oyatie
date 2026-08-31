//! Cloud datacenter-operations kernel.
//!
//! This crate owns the preview DC-ops control contract named by `cloud.dcops.*`:
//! DCIM hierarchy, BMS points, power and cooling capacity, cable maps, physical
//! security zones, asset lifecycle, work orders, and sustainability evidence.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod accounting;
mod bms;
mod cabling;
mod catalog;
mod classification;
mod equipment;
mod error;
mod facility;
mod identifiers;
mod lifecycle;
mod power;
mod rack;
mod site;
mod sustainability;
mod validation;
mod work_order;

pub use bms::*;
pub use cabling::*;
pub use catalog::CloudDcopsCatalog;
pub use equipment::*;
pub use error::*;
pub use facility::*;
pub use identifiers::*;
pub use power::*;
pub use rack::*;
pub use site::*;
pub use sustainability::*;
pub use work_order::*;

#[cfg(test)]
mod tests;
