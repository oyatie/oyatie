#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use crate::{conformance::*, *};
use data_ontology_kernel::{EntityTypeId, OntologyEngineError, PropertyValue};

fn decode(value: &str) -> ModelValue {
    serde_json::from_str(value).unwrap()
}

fn document(value: ModelValue) -> ModelDocument {
    let mut model = additive_model();
    model.entities[0].properties[1].default_value = Some(value);
    model
}

#[test]
fn defaults_are_optional_without_changing_legacy_property_bytes() {
    let legacy = br#"{"name":"notes","tier":"scalar","data_class":"InternalOnly","required":false,"value_type":{"shape":"scalar","kind":"string"},"display":null}"#;
    let property: ModelProperty = serde_json::from_slice(legacy).unwrap();
    assert_eq!(property.default_value, None);
    assert_eq!(serde_json::to_vec(&property).unwrap(), legacy);
    let mut present = property.clone();
    present.default_value = Some(ModelValue::String {
        value: String::new(),
    });
    let bytes = serde_json::to_vec(&present).unwrap();
    assert_eq!(
        serde_json::from_slice::<ModelProperty>(&bytes).unwrap(),
        present
    );
    let mut fields: serde_json::Value = serde_json::from_slice(legacy).unwrap();
    fields["default_value"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<ModelProperty>(fields).is_err());
    let duplicate = String::from_utf8(bytes).unwrap().replacen(
        "\"default_value\":",
        "\"default_value\":{\"kind\":\"string\",\"value\":\"\"},\"default_value\":",
        1,
    );
    assert!(serde_json::from_str::<ModelProperty>(&duplicate).is_err());
}

#[test]
fn literals_roundtrip_all_carriers_without_losing_integer_or_timestamp_precision() {
    for text in [
        r#"{"kind":"string","value":""}"#,
        r#"{"kind":"integer","value":"0"}"#,
        r#"{"kind":"integer","value":"9223372036854775807"}"#,
        r#"{"kind":"integer","value":"-9223372036854775808"}"#,
        r#"{"kind":"double","value":1.25}"#,
        r#"{"kind":"boolean","value":false}"#,
        r#"{"kind":"date","year":-2147483648,"month":1,"day":1}"#,
        r#"{"kind":"date","year":2000,"month":2,"day":29}"#,
        r#"{"kind":"timestamp","epoch_millis":"9223372036854775807"}"#,
        r#"{"kind":"timestamp","epoch_millis":"-9223372036854775808"}"#,
        r#"{"kind":"array","items":[{"kind":"integer","value":"0"}]}"#,
        r#"{"kind":"struct","fields":[{"name":"x","value":{"kind":"string","value":""}}]}"#,
    ] {
        let value = decode(text);
        assert_eq!(serde_json::to_string(&value).unwrap(), text);
        let native = value.native().unwrap();
        assert_eq!(ModelValue::from(&native), value);
        let model = document(value);
        assert_eq!(
            ModelDocument::decode(&model.encode().unwrap()).unwrap(),
            model
        );
    }
    assert_eq!(
        decode(r#"{"kind":"double","value":-0.0}"#),
        decode(r#"{"kind":"double","value":0.0}"#)
    );
}

#[test]
fn struct_values_canonicalize_field_order_recursively_without_reordering_arrays() {
    let reversed = decode(
        r#"{"kind":"struct","fields":[{"name":"z","value":{"kind":"array","items":[{"kind":"integer","value":"2"},{"kind":"integer","value":"1"}]}},{"name":"a","value":{"kind":"struct","fields":[{"name":"z","value":{"kind":"boolean","value":false}},{"name":"a","value":{"kind":"string","value":""}}]}}]}"#,
    );
    let canonical = serde_json::to_string(&reversed).unwrap();
    let value: serde_json::Value = serde_json::from_str(&canonical).unwrap();
    assert_eq!(value["fields"][0]["name"], "a");
    assert_eq!(value["fields"][0]["value"]["fields"][0]["name"], "a");
    assert_eq!(value["fields"][1]["value"]["items"][0]["value"], "2");
    assert_eq!(ModelValue::from(&reversed.native().unwrap()), reversed);
    assert_eq!(decode(&canonical), reversed);
    let unicode = decode(
        r#"{"kind":"struct","fields":[{"name":"𐀀","value":{"kind":"boolean","value":false}},{"name":"","value":{"kind":"boolean","value":true}}]}"#,
    );
    let value = serde_json::to_value(&unicode).unwrap();
    assert_eq!(value["fields"][0]["name"], "\u{e000}");
    assert_eq!(ModelValue::from(&unicode.native().unwrap()), unicode);
}

#[test]
fn literal_wire_refuses_null_unknown_duplicate_positional_and_noncanonical_values() {
    for text in [
        "null",
        "false",
        "0",
        "[]",
        r#"["string","x"]"#,
        r#"{"kind":"unknown","value":"x"}"#,
        r#"{"kind":"string"}"#,
        r#"{"kind":"string","value":null}"#,
        r#"{"kind":"string","value":"x","unknown":true}"#,
        r#"{"kind":"string","kind":"string","value":"x"}"#,
        r#"{"kind":"string","value":"x","value":"x"}"#,
        r#"{"kind":"integer","value":9007199254740993}"#,
        r#"{"kind":"integer","value":"9223372036854775808"}"#,
        r#"{"kind":"integer","value":"-9223372036854775809"}"#,
        r#"{"kind":"integer","value":"+1"}"#,
        r#"{"kind":"integer","value":"01"}"#,
        r#"{"kind":"integer","value":"-0"}"#,
        r#"{"kind":"integer","value":"1e3"}"#,
        r#"{"kind":"timestamp","epoch_millis":1}"#,
        r#"{"kind":"timestamp","epoch_millis":" 1"}"#,
        r#"{"kind":"double","value":"1.25"}"#,
        r#"{"kind":"double","value":1e999}"#,
        r#"{"kind":"double","value":null}"#,
        r#"{"kind":"boolean","value":0}"#,
        r#"{"kind":"date","year":1900,"month":2,"day":29}"#,
        r#"{"kind":"date","year":2026,"month":13,"day":1}"#,
        r#"{"kind":"date","year":2026,"month":1,"day":0}"#,
        r#"{"kind":"date","year":2147483648,"month":1,"day":1}"#,
        r#"{"kind":"array","items":null}"#,
        r#"{"kind":"struct","fields":[["x",{"kind":"string","value":""}]]}"#,
        r#"{"kind":"struct","fields":[{"name":"x","name":"x","value":{"kind":"string","value":""}}]}"#,
        r#"{"kind":"struct","fields":[{"name":"x","value":{"kind":"string","value":""},"unknown":true}]}"#,
        r#"{"kind":"struct","fields":[{"name":"x","value":{"kind":"string","value":""}},{"name":"x","value":{"kind":"string","value":""}}]}"#,
    ] {
        assert!(serde_json::from_str::<ModelValue>(text).is_err(), "{text}");
    }
}

#[test]
fn native_publication_captures_defaults_and_keeps_schemas_unchanged() {
    let mut model = named_string_model();
    model.entities[0].properties[2].default_value = Some(ModelValue::String {
        value: "TestActive".into(),
    });
    let mut store = ReferenceModelStore::initialize(binding(), baseline()).unwrap();
    let without_defaults = {
        let mut other = model.clone();
        other.entities[0].properties[2].default_value = None;
        let saved = store.create_draft(&caller(), 1, other).unwrap();
        let prepared = store.prepare(&caller(), saved.id, 1, 1).unwrap();
        schema_bytes(prepared.model_registry(), "ten_model").unwrap()
    };
    let saved = store.create_draft(&caller(), 1, model.clone()).unwrap();
    let prepared = store.prepare(&caller(), saved.id, 1, 1).unwrap();
    assert_eq!(
        schema_bytes(prepared.model_registry(), "ten_model").unwrap(),
        without_defaults
    );
    assert_eq!(
        ModelDocument::capture(prepared.model_registry(), "ten_model").unwrap(),
        model
    );
    store.publish(&caller(), &prepared).unwrap();
    let registry = store.history(&caller()).unwrap().registry().unwrap();
    assert_eq!(
        ModelBaseline::capture(&registry, "ten_model")
            .unwrap()
            .registry("ten_model")
            .unwrap(),
        registry
    );
    let id = EntityTypeId::new("ety_record").unwrap();
    assert_eq!(
        registry.entity_type("ten_model", &id).unwrap().properties[2].default_value,
        Some(PropertyValue::String("TestActive".into()))
    );
    assert!(
        registry
            .entity_type_at_revision("ten_model", &id, 1)
            .unwrap()
            .properties
            .iter()
            .all(|p| p.default_value.is_none())
    );
    let mut invalid = model;
    invalid.entities[0].revision = 3;
    invalid.entities[0].properties[2].default_value = Some(ModelValue::String {
        value: "testactive".into(),
    });
    let saved = store.create_draft(&caller(), 2, invalid).unwrap();
    assert!(matches!(
        store.prepare(&caller(), saved.id, 1, 2),
        Err(ModelError::Native {
            cause: OntologyEngineError::InvalidPropertyDefault { .. },
            ..
        })
    ));
}

mod limits;
