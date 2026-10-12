use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    EntityTypeDefinition, EntityTypeId, EntityTypePropertyDefinition, PropertyTier, ScalarType,
    StructFieldDeclaration, StructSchema,
};
use serde_json::json;

use super::*;

#[test]
fn current_schemas_preserve_native_types_designations_order_and_tenant_scope() {
    let internal = PrivacyDataClass::internal_only();
    let mut properties = vec![
        EntityTypePropertyDefinition::new(
            "name",
            PropertyTier::Scalar,
            PrivacyDataClass::pii_identifying(),
            true,
        )
        .unwrap(),
    ];
    for scalar in [
        ScalarType::String,
        ScalarType::Integer,
        ScalarType::Double,
        ScalarType::Boolean,
        ScalarType::Date,
        ScalarType::Timestamp,
    ] {
        let declaration = ValueTypeDeclaration::Scalar(scalar);
        properties.push(
            EntityTypePropertyDefinition::typed(
                declaration.type_label(),
                declaration,
                internal,
                false,
            )
            .unwrap(),
        );
    }
    let nested = ValueTypeDeclaration::Struct(StructSchema {
        fields: vec![StructFieldDeclaration {
            name: "tags".to_owned(),
            required: false,
            value_type: ValueTypeDeclaration::Array {
                element: Box::new(ValueTypeDeclaration::Scalar(ScalarType::Date)),
            },
        }],
    });
    properties
        .push(EntityTypePropertyDefinition::typed("nested", nested, internal, false).unwrap());
    let definition = EntityTypeDefinition::new(
        "ten_a",
        EntityTypeId::new("ety_custom").unwrap(),
        "Private display prose",
        properties,
        0,
    )
    .unwrap()
    .with_primary_key_property("name")
    .with_title_property("string");
    let mut engine = OntologyEngine::default();
    engine.register_entity_type(definition.clone()).unwrap();
    let mut foreign = definition.clone();
    foreign.tenant_id = "ten_b".to_owned();
    foreign.id = EntityTypeId::new("ety_foreign").unwrap();
    engine.register_entity_type(foreign).unwrap();
    let wire = serde_json::to_value(rows(&engine, "ten_a")).unwrap();
    assert_eq!(wire.as_array().unwrap().len(), 1);
    assert_eq!(wire[0]["entity_type"], "ety_custom");
    assert_eq!(wire[0]["revision"], 0);
    assert_eq!(wire[0]["primary_key_property"], "name");
    assert_eq!(wire[0]["title_property"], "string");
    assert_eq!(
        wire[0]["properties"][0],
        json!({"name":"name","tier":"scalar",
        "data_class":"PII_IDENTIFYING","required":true,"value_type":null})
    );
    for (index, label) in [
        "string",
        "integer",
        "double",
        "boolean",
        "date",
        "timestamp",
    ]
    .iter()
    .enumerate()
    {
        assert_eq!(
            wire[0]["properties"][index + 1]["value_type"],
            json!({"kind":label})
        );
    }
    assert_eq!(
        wire[0]["properties"][7]["value_type"],
        json!({"kind":"struct","fields":[{
            "name":"tags","required":false,"value_type":{"kind":"array","element":{"kind":"date"}}
        }]})
    );
    assert!(!wire.to_string().contains("Private display prose"));
    let mut evolved = definition;
    evolved.revision = 2;
    evolved.properties.push(
        EntityTypePropertyDefinition::new("new_optional", PropertyTier::Scalar, internal, false)
            .unwrap(),
    );
    engine.evolve_entity_type(evolved).unwrap();
    assert_eq!(rows(&engine, "ten_a")[0].revision, 2);
    assert!(rows(&engine, "ten_absent").is_empty());
}

#[test]
fn privacy_metadata_is_canonical_and_legacy_tiers_never_gain_typed_declarations() {
    let mut properties = Vec::new();
    for (index, tier) in PropertyTier::all_tiers().into_iter().enumerate() {
        properties.push(
            EntityTypePropertyDefinition::new(
                format!("legacy_{index}"),
                tier,
                PrivacyDataClass::internal_only(),
                false,
            )
            .unwrap(),
        );
    }
    for (index, class) in [
        DataClass::InternalOnly,
        DataClass::Phi,
        DataClass::PiiIdentifying,
        DataClass::PiiQuasiIdentifier,
        DataClass::Pci,
        DataClass::Financial,
        DataClass::FinancialRegulatedCredit,
        DataClass::BehavioralTenantProduct,
        DataClass::BehavioralAds,
        DataClass::DeclaredPreference,
        DataClass::SearchQuery,
        DataClass::Public,
        DataClass::SensitivePipaArticle23,
    ]
    .into_iter()
    .enumerate()
    {
        properties.push(
            EntityTypePropertyDefinition::new(
                format!("class_{index}"),
                PropertyTier::Scalar,
                PrivacyDataClass::try_from(class).unwrap(),
                false,
            )
            .unwrap(),
        );
    }
    let mut engine = OntologyEngine::default();
    engine
        .register_entity_type(
            EntityTypeDefinition::new(
                "ten_a",
                EntityTypeId::new("ety_labels").unwrap(),
                "Labels",
                properties,
                1,
            )
            .unwrap(),
        )
        .unwrap();
    let wire = serde_json::to_value(rows(&engine, "ten_a")).unwrap();
    for (index, tier) in PropertyTier::all_tiers().into_iter().enumerate() {
        assert_eq!(wire[0]["properties"][index]["tier"], tier.wire_label());
        assert_eq!(
            wire[0]["properties"][index]["value_type"],
            serde_json::Value::Null
        );
    }
    let labels = wire[0]["properties"].as_array().unwrap()[6..]
        .iter()
        .map(|p| p["data_class"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        labels,
        data_boundary_kernel::PRIVACY_PROGRAM_DATA_CLASS_LABELS
    );
    assert_eq!(wire[0]["primary_key_property"], serde_json::Value::Null);
    assert_eq!(wire[0]["title_property"], serde_json::Value::Null);
}
