//! Agreed cross-owner surface for residency-policy values and predicates.
//!
//! Consumers receive Network's established exact residency identities, typed
//! validation errors, label parsing, and home-region predicate without
//! importing its internal aggregate. The port is signature-closed through the
//! exact Data classification wrapper and Network region-reference values. It
//! intentionally excludes residency registries, tenant bindings, change plans,
//! transfer permits, jurisdiction inference, and data-class conversion. The
//! legacy residency core remains the defining crate until a dedicated Network
//! structural lane decomposes it.

#![forbid(unsafe_code)]

pub use data_classification::Classified;
pub use network_residency::{
    PerPackResidency, PerPackResidencyCreate, RegionJurisdiction, RegionRef, RegionRefCreate,
    RegulatorOverlay, RegulatorOverlayCreate, ResidencyClass, ResidencyError,
    parse_residency_class_label, residency_class_allows_home_region_label,
};
