//! Regional pack kernel: canonical regulatory and residency pack metadata.
//!
//! Also hosts the M04-P01 vertical capability pack types
//! (`CapabilityPack`, `PackVersion`, `CapabilityPackError`) merged per
//! execution-variant decision 2026-05-17 (option 2 — merge-into-existing-crates).
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod vertical_regulatory_profile;
pub use vertical_regulatory_profile::{
    AdVertical, VerticalRegulatoryProfile, VerticalRegulatoryProfileError,
};
pub mod capability_pack;
pub use capability_pack::{CapabilityPack, CapabilityPackError, PackVersion};

use oya_data_boundary_kernel::{Classified, DataClass};
use oya_residency_domain::{ResidencyClass, parse_residency_class_label};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegionalPack {
    pub id: String, // data_class: INTERNAL_ONLY
    pub region: Classified<String>,
    pub residency_class: Classified<ResidencyClass>,
    pub controls: Classified<Vec<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegionalPackError {
    InvalidPackId,
    EmptyRegion,
    EmptyResidencyClass,
    InvalidResidencyClass,
    MissingControls,
}

impl RegionalPack {
    pub fn new(
        id: String,
        region: String,
        residency_class: String,
        controls: Vec<String>,
    ) -> Result<Self, RegionalPackError> {
        if !id.starts_with("oya-pack-") {
            return Err(RegionalPackError::InvalidPackId);
        }
        if region.trim().is_empty() {
            return Err(RegionalPackError::EmptyRegion);
        }
        if residency_class.trim().is_empty() {
            return Err(RegionalPackError::EmptyResidencyClass);
        }
        let residency_class = parse_residency_class_label(&residency_class)
            .ok_or(RegionalPackError::InvalidResidencyClass)?;
        if controls.is_empty() {
            return Err(RegionalPackError::MissingControls);
        }
        Ok(Self {
            id,
            region: Classified::new(region, DataClass::InternalOnly),
            residency_class: Classified::new(residency_class, DataClass::InternalOnly),
            controls: Classified::new(controls, DataClass::InternalOnly),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_canonical_residency_labels() {
        let pack = RegionalPack::new(
            "oya-pack-kr".to_string(),
            "kr-seoul".to_string(),
            "strict_kr".to_string(),
            vec!["PIPA".to_string()],
        )
        .expect("canonical residency label should be accepted");

        assert_eq!(pack.residency_class.value.label(), Some("strict_kr"));
    }

    #[test]
    fn rejects_non_canonical_residency_labels() {
        let error = RegionalPack::new(
            "oya-pack-kr".to_string(),
            "kr-seoul".to_string(),
            "KR_RESIDENT".to_string(),
            vec!["PIPA".to_string()],
        )
        .expect_err("regional packs use ADR-0049 residency labels");

        assert_eq!(error, RegionalPackError::InvalidResidencyClass);
    }
}
