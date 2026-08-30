//! Shared builders for the in-crate test files.

use crate::{EntityTypePropertyDefinition, PropertyTier};
use data_boundary_kernel::{DataClass, PrivacyDataClass};

pub(super) fn property(name: &str) -> EntityTypePropertyDefinition {
    EntityTypePropertyDefinition::new(
        name,
        PropertyTier::Scalar,
        PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap(),
        true,
    )
    .unwrap()
}
