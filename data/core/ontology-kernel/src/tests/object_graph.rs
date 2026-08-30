use crate::*;
use data_boundary_kernel::{DataClass, PrivacyDataClass};

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
