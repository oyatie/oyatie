use serde_json::json;

use super::*;

fn schema_wire() -> serde_json::Value {
    json!({"entity_type":"ety_record","revision":1,"properties":[{
        "name":"name","tier":"scalar","data_class":"PII_IDENTIFYING",
        "required":true,"value_type":null
    }],"primary_key_property":null,"title_property":"name"})
}

#[test]
fn explicit_null_metadata_round_trips_and_missing_metadata_refuses() {
    let wire = schema_wire();
    let schema: EntitySchema = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(schema.validate(), Ok(()));
    assert_eq!(serde_json::to_value(schema).unwrap(), wire);
    for field in ["primary_key_property", "title_property"] {
        let mut missing = schema_wire();
        missing.as_object_mut().unwrap().remove(field);
        assert!(
            serde_json::from_value::<EntitySchema>(missing).is_err(),
            "{field}"
        );
    }
    let mut missing = schema_wire();
    missing["properties"][0]
        .as_object_mut()
        .unwrap()
        .remove("value_type");
    assert!(serde_json::from_value::<EntitySchema>(missing).is_err());
}

#[test]
fn malformed_declarations_are_unavailable_without_coercion_or_deduplication() {
    for field in [
        json!({"kind":"future"}),
        json!({"kind":"string","extra":true}),
    ] {
        let mut wire = schema_wire();
        wire["properties"][0]["value_type"] = field;
        assert!(serde_json::from_value::<EntitySchema>(wire).is_err());
    }
    for (path, value) in [
        ("tier", json!("vector")),
        ("data_class", json!("SECRET")),
        ("name", json!(" ")),
        ("data_class", json!("PII")),
    ] {
        let mut wire = schema_wire();
        wire["properties"][0]["value_type"] = json!({"kind":"string"});
        wire["properties"][0][path] = value;
        let schema: EntitySchema = serde_json::from_value(wire).unwrap();
        assert_eq!(schema.validate(), Err(OntologyReadError::SourceUnavailable));
    }
    let mut wire = schema_wire();
    let duplicate = wire["properties"][0].clone();
    wire["properties"].as_array_mut().unwrap().push(duplicate);
    assert_eq!(
        serde_json::from_value::<EntitySchema>(wire)
            .unwrap()
            .validate(),
        Err(OntologyReadError::SourceUnavailable)
    );
    for (field, value) in [
        ("primary_key_property", json!("missing")),
        ("title_property", json!("missing")),
    ] {
        let mut wire = schema_wire();
        wire[field] = value;
        assert_eq!(
            serde_json::from_value::<EntitySchema>(wire)
                .unwrap()
                .validate(),
            Err(OntologyReadError::SourceUnavailable)
        );
    }
    let mut wire = schema_wire();
    wire["primary_key_property"] = json!("name");
    wire["properties"][0]["required"] = json!(false);
    assert_eq!(
        serde_json::from_value::<EntitySchema>(wire)
            .unwrap()
            .validate(),
        Err(OntologyReadError::SourceUnavailable)
    );
}

#[test]
fn nested_types_retain_structure_and_enforce_native_depth_and_field_integrity() {
    let mut wire = schema_wire();
    let declaration = json!({"kind":"struct","fields":[{"name":"tags","required":false,
        "value_type":{"kind":"array","element":{"kind":"date"}}}]});
    wire["properties"][0]["value_type"] = declaration;
    wire["properties"][0]["tier"] = json!("struct");
    let schema: EntitySchema = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(schema.validate(), Ok(()));
    assert_eq!(serde_json::to_value(schema).unwrap(), wire);
    for fields in [
        json!([]),
        json!([
            {"name":"x","required":false,"value_type":{"kind":"string"}},
            {"name":"x","required":true,"value_type":{"kind":"integer"}}
        ]),
    ] {
        wire["properties"][0]["value_type"]["fields"] = fields;
        assert_eq!(
            serde_json::from_value::<EntitySchema>(wire.clone())
                .unwrap()
                .validate(),
            Err(OntologyReadError::SourceUnavailable)
        );
    }
    for arrays in [7, 8] {
        let mut declaration = json!({"kind":"string"});
        for _ in 0..arrays {
            declaration = json!({"kind":"array","element":declaration});
        }
        wire["properties"][0]["value_type"] = declaration;
        wire["properties"][0]["tier"] = json!("vector");
        let expected = if arrays == 7 {
            Ok(())
        } else {
            Err(OntologyReadError::SourceUnavailable)
        };
        assert_eq!(
            serde_json::from_value::<EntitySchema>(wire.clone())
                .unwrap()
                .validate(),
            expected
        );
    }
}

#[test]
fn all_native_privacy_labels_and_legacy_tiers_remain_admitted() {
    for label in [
        "INTERNAL_ONLY",
        "PHI",
        "PII_IDENTIFYING",
        "PII_QUASI_IDENTIFIER",
        "PCI",
        "FINANCIAL",
        "FINANCIAL_REGULATED_CREDIT",
        "BEHAVIORAL_TENANT_PRODUCT",
        "BEHAVIORAL_ADS",
        "DECLARED_PREFERENCE",
        "SEARCH_QUERY",
        "PUBLIC",
        "SENSITIVE_PIPA_ART23",
    ] {
        for tier in [
            "scalar",
            "vector",
            "timeseries",
            "geo",
            "ciphertext",
            "struct",
        ] {
            let mut wire = schema_wire();
            wire["properties"][0]["data_class"] = json!(label);
            wire["properties"][0]["tier"] = json!(tier);
            assert_eq!(
                serde_json::from_value::<EntitySchema>(wire)
                    .unwrap()
                    .validate(),
                Ok(())
            );
        }
    }
}
