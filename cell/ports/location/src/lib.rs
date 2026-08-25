//! Agreed cross-owner contract for region, availability-zone, and cell identity.
//!
//! Consumers bind this port rather than the Cell region engine. During the
//! compatibility window the port re-exports the existing validated identities,
//! preserving their defining type, constructors, and error behavior without
//! copying Cell domain models.

#![forbid(unsafe_code)]

pub use cell_region::{AzCode, CellId, RegionCode};
