//! Per-vertical regulatory profile: binds a vertical to its KR regulatory pack subset.
//!
//! M06-P03 merge-variant delta-1.  No new crate, no new deps (std-only additions).
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// The 13 ad verticals defined in M06-P02.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AdVertical {
    Automotive,
    ECommerce,
    Finance,
    Gaming,
    Healthcare,
    RealEstate,
    Retail,
    SocialMedia,
    Telecom,
    Travel,
    Education,
    Entertainment,
    Food,
}

impl AdVertical {
    /// Canonical kebab-case label used in file paths and registry keys.
    pub fn label(self) -> &'static str {
        match self {
            Self::Automotive => "automotive",
            Self::ECommerce => "e-commerce",
            Self::Finance => "finance",
            Self::Gaming => "gaming",
            Self::Healthcare => "healthcare",
            Self::RealEstate => "real-estate",
            Self::Retail => "retail",
            Self::SocialMedia => "social-media",
            Self::Telecom => "telecom",
            Self::Travel => "travel",
            Self::Education => "education",
            Self::Entertainment => "entertainment",
            Self::Food => "food",
        }
    }
}

/// Binds one vertical to a KR regulatory pack subset and a set of applicable controls.
///
/// `pack_id` must match a `RegionalPack::id` already provisioned in the pack registry
/// (prefix `pack-`).  `controls` is the non-empty subset of controls from that pack
/// that apply to this vertical.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerticalRegulatoryProfile {
    pub vertical: AdVertical,
    pub pack_id: String,
    pub controls: Vec<String>,
}

/// Errors produced when constructing a [`VerticalRegulatoryProfile`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerticalRegulatoryProfileError {
    /// `pack_id` does not start with `pack-`.
    InvalidPackId,
    /// `controls` slice is empty — every vertical must bind at least one control.
    EmptyControls,
}

impl VerticalRegulatoryProfile {
    /// Construct and validate a binding.
    pub fn new(
        vertical: AdVertical,
        pack_id: String,
        controls: Vec<String>,
    ) -> Result<Self, VerticalRegulatoryProfileError> {
        if !pack_id.starts_with("pack-") {
            return Err(VerticalRegulatoryProfileError::InvalidPackId);
        }
        if controls.is_empty() {
            return Err(VerticalRegulatoryProfileError::EmptyControls);
        }
        Ok(Self {
            vertical,
            pack_id,
            controls,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_vertical_regulatory_profile() {
        let profile = VerticalRegulatoryProfile::new(
            AdVertical::Healthcare,
            "pack-alpha".to_string(),
            vec!["PIPA".to_string(), "HMIS".to_string()],
        )
        .expect("valid profile should be accepted");

        assert_eq!(profile.vertical, AdVertical::Healthcare);
        assert_eq!(profile.vertical.label(), "healthcare");
        assert_eq!(profile.pack_id, "pack-alpha");
        assert_eq!(profile.controls.len(), 2);
    }

    #[test]
    fn rejects_invalid_pack_id_prefix() {
        let err = VerticalRegulatoryProfile::new(
            AdVertical::Finance,
            "pack-kr".to_string(),
            vec!["PIPA".to_string()],
        )
        .expect_err("pack_id without pack- prefix must be rejected");

        assert_eq!(err, VerticalRegulatoryProfileError::InvalidPackId);
    }

    #[test]
    fn rejects_empty_controls() {
        let err = VerticalRegulatoryProfile::new(
            AdVertical::Retail,
            "pack-alpha".to_string(),
            vec![],
        )
        .expect_err("empty controls must be rejected");

        assert_eq!(err, VerticalRegulatoryProfileError::EmptyControls);
    }

    #[test]
    fn all_13_verticals_have_distinct_labels() {
        use std::collections::BTreeSet;
        let verticals = [
            AdVertical::Automotive,
            AdVertical::ECommerce,
            AdVertical::Finance,
            AdVertical::Gaming,
            AdVertical::Healthcare,
            AdVertical::RealEstate,
            AdVertical::Retail,
            AdVertical::SocialMedia,
            AdVertical::Telecom,
            AdVertical::Travel,
            AdVertical::Education,
            AdVertical::Entertainment,
            AdVertical::Food,
        ];
        let labels: BTreeSet<_> = verticals.iter().map(|v| v.label()).collect();
        assert_eq!(
            labels.len(),
            13,
            "all 13 verticals must have distinct labels"
        );
    }
}
