//! Ontology kernel: data-classed entities, property-tier semantics, and pillar isolation.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod pillar;
pub use pillar::{OntologyPillar, UnknownPillarLabel};

use std::collections::BTreeMap;

use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectEntity {
    pub tenant_id: String,                            // data_class: INTERNAL_ONLY
    pub id: String,                                   // data_class: INTERNAL_ONLY
    pub entity_type: Classified<String>,              // data_class: INTERNAL_ONLY
    pub properties: BTreeMap<String, ObjectProperty>, // data_class: PROPERTY_VALUE_PRIVACY_CLASS
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectEntityUpsertOutcome {
    Created,
    Updated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectPropertyUpsertOutcome {
    Inserted,
    Updated,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ObjectGraph {
    entities: BTreeMap<ObjectEntityKey, ObjectEntity>, // data_class: PROPERTY_VALUE_PRIVACY_CLASS
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct ObjectEntityKey {
    tenant_id: String, // data_class: INTERNAL_ONLY
    id: String,        // data_class: INTERNAL_ONLY
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
            validate_property(&property)?;
            by_name.insert(property.name.clone(), property);
        }
        Ok(Self {
            tenant_id,
            id,
            entity_type: Classified::new(entity_type, DataClass::InternalOnly),
            properties: by_name,
        })
    }

    pub fn upsert_property(
        &mut self,
        property: ObjectProperty,
    ) -> Result<ObjectPropertyUpsertOutcome, ObjectGraphError> {
        validate_property(&property)?;
        let outcome = if self
            .properties
            .insert(property.name.clone(), property)
            .is_some()
        {
            ObjectPropertyUpsertOutcome::Updated
        } else {
            ObjectPropertyUpsertOutcome::Inserted
        };
        Ok(outcome)
    }
}

impl ObjectGraph {
    pub fn upsert_entity(
        &mut self,
        entity: ObjectEntity,
    ) -> Result<ObjectEntityUpsertOutcome, ObjectGraphError> {
        validate_entity_key(&entity.tenant_id, &entity.id)?;
        if entity.properties.is_empty() {
            return Err(ObjectGraphError::MissingProperties);
        }
        if entity.entity_type.value.trim().is_empty() {
            return Err(ObjectGraphError::EmptyEntityType);
        }
        for property in entity.properties.values() {
            validate_property(property)?;
        }

        let key = ObjectEntityKey {
            tenant_id: entity.tenant_id.clone(),
            id: entity.id.clone(),
        };
        let outcome = if self.entities.insert(key, entity).is_some() {
            ObjectEntityUpsertOutcome::Updated
        } else {
            ObjectEntityUpsertOutcome::Created
        };
        Ok(outcome)
    }

    pub fn get(&self, tenant_id: &str, entity_id: &str) -> Option<&ObjectEntity> {
        self.entities.get(&ObjectEntityKey {
            tenant_id: tenant_id.to_string(),
            id: entity_id.to_string(),
        })
    }

    pub fn entities_for_tenant(&self, tenant_id: &str) -> impl Iterator<Item = &ObjectEntity> {
        self.entities
            .range(
                ObjectEntityKey {
                    tenant_id: tenant_id.to_string(),
                    id: String::new(),
                }..,
            )
            .map_while(move |(key, entity)| (key.tenant_id == tenant_id).then_some(entity))
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
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

fn validate_property(property: &ObjectProperty) -> Result<(), ObjectGraphError> {
    if property.name.trim().is_empty() {
        return Err(ObjectGraphError::EmptyPropertyName);
    }
    Ok(())
}

fn validate_entity_key(tenant_id: &str, entity_id: &str) -> Result<(), ObjectGraphError> {
    if tenant_id.trim().is_empty() || !entity_id.starts_with("ent_") {
        return Err(ObjectGraphError::InvalidEntityId);
    }
    Ok(())
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

    #[test]
    fn property_tier_contract_exposes_five_object_graph_tiers() {
        let tiers = PropertyTier::object_graph_property_tiers();

        assert_eq!(tiers.len(), 5);
        assert_eq!(
            tiers.map(PropertyTier::wire_label),
            ["vector", "timeseries", "geo", "ciphertext", "struct"]
        );
        assert_eq!(
            PropertyTier::all_tiers().map(PropertyTier::wire_label),
            [
                "scalar",
                "vector",
                "timeseries",
                "geo",
                "ciphertext",
                "struct"
            ]
        );
    }

    #[test]
    fn object_entity_upsert_inserts_and_updates_property_by_name() {
        let mut entity = ObjectEntity::new(
            "tenant_a".into(),
            "ent_profile".into(),
            "profile".into(),
            vec![ObjectProperty::new(
                "embedding".into(),
                "[0.1,0.2]".into(),
                PropertyTier::Vector,
                PrivacyDataClass::try_from(DataClass::PiiQuasiIdentifier).unwrap(),
            )],
        )
        .unwrap();

        assert_eq!(
            entity.upsert_property(ObjectProperty::new(
                "last_seen".into(),
                "2026-05-14T00:00:00Z".into(),
                PropertyTier::Timeseries,
                PrivacyDataClass::try_from(DataClass::BehavioralTenantProduct).unwrap(),
            )),
            Ok(ObjectPropertyUpsertOutcome::Inserted)
        );
        assert_eq!(
            entity.upsert_property(ObjectProperty::new(
                "embedding".into(),
                "[0.3,0.4]".into(),
                PropertyTier::Vector,
                PrivacyDataClass::try_from(DataClass::PiiQuasiIdentifier).unwrap(),
            )),
            Ok(ObjectPropertyUpsertOutcome::Updated)
        );

        assert_eq!(entity.properties.len(), 2);
        assert_eq!(
            entity.properties["embedding"].value.value,
            "[0.3,0.4]".to_string()
        );
        assert_eq!(
            entity.properties["last_seen"].tier,
            PropertyTier::Timeseries
        );
    }

    #[test]
    fn object_entity_upsert_rejects_empty_property_name_without_mutation() {
        let mut entity = ObjectEntity::new(
            "tenant_a".into(),
            "ent_profile".into(),
            "profile".into(),
            vec![ObjectProperty::new(
                "location".into(),
                "{\"lat\":37.0,\"lng\":127.0}".into(),
                PropertyTier::Geo,
                PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap(),
            )],
        )
        .unwrap();

        assert_eq!(
            entity.upsert_property(ObjectProperty::new(
                " ".into(),
                "invalid".into(),
                PropertyTier::Struct,
                PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap(),
            )),
            Err(ObjectGraphError::EmptyPropertyName)
        );
        assert_eq!(entity.properties.len(), 1);
        assert!(entity.properties.contains_key("location"));
    }

    #[test]
    fn object_graph_upsert_creates_and_updates_entity_by_tenant_and_id() {
        let mut graph = ObjectGraph::default();
        let created_entity = ObjectEntity::new(
            "tenant_a".into(),
            "ent_profile".into(),
            "profile".into(),
            vec![ObjectProperty::new(
                "embedding".into(),
                "[0.1,0.2]".into(),
                PropertyTier::Vector,
                PrivacyDataClass::try_from(DataClass::PiiQuasiIdentifier).unwrap(),
            )],
        )
        .unwrap();
        let updated_entity = ObjectEntity::new(
            "tenant_a".into(),
            "ent_profile".into(),
            "profile".into(),
            vec![ObjectProperty::new(
                "location".into(),
                "{\"lat\":37.0,\"lng\":127.0}".into(),
                PropertyTier::Geo,
                PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap(),
            )],
        )
        .unwrap();

        assert_eq!(
            graph.upsert_entity(created_entity),
            Ok(ObjectEntityUpsertOutcome::Created)
        );
        assert_eq!(
            graph.upsert_entity(updated_entity),
            Ok(ObjectEntityUpsertOutcome::Updated)
        );

        assert_eq!(graph.len(), 1);
        let stored = graph
            .get("tenant_a", "ent_profile")
            .expect("entity exists after upsert");
        assert!(stored.properties.contains_key("location"));
        assert!(!stored.properties.contains_key("embedding"));
    }

    #[test]
    fn object_graph_upsert_keeps_tenants_row_isolated() {
        let mut graph = ObjectGraph::default();
        for tenant_id in ["tenant_a", "tenant_b"] {
            let entity = ObjectEntity::new(
                tenant_id.into(),
                "ent_profile".into(),
                "profile".into(),
                vec![ObjectProperty::new(
                    "config".into(),
                    tenant_id.into(),
                    PropertyTier::Struct,
                    PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap(),
                )],
            )
            .unwrap();
            assert_eq!(
                graph.upsert_entity(entity),
                Ok(ObjectEntityUpsertOutcome::Created)
            );
        }

        assert_eq!(graph.len(), 2);
        assert_eq!(
            graph.get("tenant_a", "ent_profile").unwrap().properties["config"]
                .value
                .value,
            "tenant_a"
        );
        assert_eq!(
            graph.get("tenant_b", "ent_profile").unwrap().properties["config"]
                .value
                .value,
            "tenant_b"
        );
        assert_eq!(graph.entities_for_tenant("tenant_a").count(), 1);
        assert_eq!(graph.entities_for_tenant("tenant_b").count(), 1);
    }
}
