//! Property-plane vocabulary: [`PropertyTier`] and the classified
//! [`ObjectProperty`] value carried by object instances.

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

use crate::object_graph::ObjectGraphError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PropertyTier {
    Scalar,
    Vector,
    Timeseries,
    Geo,
    Ciphertext,
    Struct,
}

impl PropertyTier {
    pub const fn wire_label(self) -> &'static str {
        match self {
            Self::Scalar => "scalar",
            Self::Vector => "vector",
            Self::Timeseries => "timeseries",
            Self::Geo => "geo",
            Self::Ciphertext => "ciphertext",
            Self::Struct => "struct",
        }
    }

    pub const fn all_tiers() -> [Self; 6] {
        [
            Self::Scalar,
            Self::Vector,
            Self::Timeseries,
            Self::Geo,
            Self::Ciphertext,
            Self::Struct,
        ]
    }

    pub const fn object_graph_property_tiers() -> [Self; 5] {
        [
            Self::Vector,
            Self::Timeseries,
            Self::Geo,
            Self::Ciphertext,
            Self::Struct,
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectProperty {
    pub name: String,              // data_class: INTERNAL_ONLY
    pub value: Classified<String>, // data_class: PROPERTY_VALUE_PRIVACY_CLASS
    pub tier: PropertyTier,        // data_class: INTERNAL_ONLY
}

impl ObjectProperty {
    pub fn new(
        name: String,
        value: String,
        tier: PropertyTier,
        data_class: PrivacyDataClass,
    ) -> Self {
        Self::new_with_privacy_data_class(name, value, tier, data_class)
    }

    /// Compatibility constructor for request/import seams that still carry raw
    /// `DataClass` labels. Canonical object properties take
    /// `PrivacyDataClass`, and this path fails closed for operational markers
    /// and subject markers.
    pub fn try_from_legacy_data_class(
        name: String,
        value: String,
        tier: PropertyTier,
        data_class: DataClass,
    ) -> Result<Self, ObjectGraphError> {
        let data_class = PrivacyDataClass::try_from(data_class)
            .map_err(|_| ObjectGraphError::InvalidDataClass)?;
        Ok(Self::new(name, value, tier, data_class))
    }

    pub fn new_with_privacy_data_class(
        name: String,
        value: String,
        tier: PropertyTier,
        data_class: PrivacyDataClass,
    ) -> Self {
        Self {
            name,
            value: Classified::new(value, data_class),
            tier,
        }
    }
}

pub(crate) fn validate_property(property: &ObjectProperty) -> Result<(), ObjectGraphError> {
    if property.name.trim().is_empty() {
        return Err(ObjectGraphError::EmptyPropertyName);
    }
    Ok(())
}
