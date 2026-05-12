//! Object Graph kernel: data-classed entities and property-tier semantics.

use std::collections::BTreeMap;

use oya_platform_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PropertyTier {
    Scalar,
    Vector,
    Timeseries,
    Geo,
    Ciphertext,
    Struct,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectProperty {
    pub name: String,
    pub value: Classified<String>,
    pub tier: PropertyTier,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectEntity {
    pub tenant_id: String,
    pub id: String,
    pub entity_type: Classified<String>,
    pub properties: BTreeMap<String, ObjectProperty>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObjectGraphError {
    InvalidEntityId,
    EmptyEntityType,
    MissingProperties,
    EmptyPropertyName,
    InvalidDataClass,
}

impl ObjectEntity {
    pub fn new(
        tenant_id: String,
        id: String,
        entity_type: String,
        properties: Vec<ObjectProperty>,
    ) -> Result<Self, ObjectGraphError> {
        if !id.starts_with("ent_") {
            return Err(ObjectGraphError::InvalidEntityId);
        }
        if entity_type.trim().is_empty() {
            return Err(ObjectGraphError::EmptyEntityType);
        }
        if properties.is_empty() {
            return Err(ObjectGraphError::MissingProperties);
        }
        let mut by_name = BTreeMap::new();
        for property in properties {
            if property.name.trim().is_empty() {
                return Err(ObjectGraphError::EmptyPropertyName);
            }
            by_name.insert(property.name.clone(), property);
        }
        Ok(Self {
            tenant_id,
            id,
            entity_type: Classified::new(entity_type, DataClass::InternalOnly),
            properties: by_name,
        })
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_property_accepts_privacy_data_classes() {
        let property = ObjectProperty::new(
            "email".into(),
            "worker@example.com".into(),
            PropertyTier::Scalar,
            PrivacyDataClass::try_from(DataClass::PiiIdentifying).unwrap(),
        );

        assert_eq!(property.name, "email");
        assert_eq!(
            property.value.data_class.compatibility_data_class(),
            DataClass::PiiIdentifying
        );
    }

    #[test]
    fn object_property_rejects_operational_and_subject_markers() {
        for data_class in [DataClass::Audit, DataClass::Secret, DataClass::Children] {
            assert_eq!(
                ObjectProperty::try_from_legacy_data_class(
                    "marker".into(),
                    "not a privacy class".into(),
                    PropertyTier::Scalar,
                    data_class,
                ),
                Err(ObjectGraphError::InvalidDataClass)
            );
        }
    }
}
