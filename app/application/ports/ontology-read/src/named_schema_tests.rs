use foundry_model_api::{ModelStore, ReferenceModelStore, conformance::*, schema_bytes};
use serde_json::{Value, json};

use super::*;

fn native_wire() -> Value {
    let mut store = ReferenceModelStore::initialize(binding(), baseline()).unwrap();
    let draft = store
        .create_draft(&caller(), 1, named_string_model())
        .unwrap();
    let ready = store
        .prepare(&caller(), draft.id, draft.version, draft.base_generation)
        .unwrap();
    store.publish(&caller(), &ready).unwrap();
    let registry = store.history(&caller()).unwrap().registry().unwrap();
    serde_json::from_slice(&schema_bytes(&registry, "ten_model").unwrap()).unwrap()
}

#[test]
fn native_published_named_strings_round_trip_as_scalar_keys_without_expansion_or_coercion() {
    let mut wire = native_wire();
    let schemas: Vec<EntitySchema> = serde_json::from_value(wire.clone()).unwrap();
    assert!(schemas.iter().all(|schema| schema.validate().is_ok()));
    assert_eq!(serde_json::to_value(schemas).unwrap(), wire);
    let named = wire[0]["properties"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["value_type"]["kind"] == "named_string")
        .unwrap();
    named["required"] = json!(true);
    named["value_type"]["members"] = json!(["\u{feff}", " A\nB\t\u{0000}", "é"]);
    let key = named["name"].clone();
    wire[0]["primary_key_property"] = key;
    let schemas: Vec<EntitySchema> = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(schemas[0].validate(), Ok(()));
    assert_eq!(serde_json::to_value(schemas).unwrap(), wire);
}

#[test]
fn materialized_named_strings_refuse_wrong_shapes_members_and_nested_positions() {
    let original = native_wire();
    let index = original[0]["properties"]
        .as_array()
        .unwrap()
        .iter()
        .position(|row| row["value_type"]["kind"] == "named_string")
        .unwrap();
    for declaration in [
        json!({"kind":"named_string","type_id":"vty_a","revision":1,"display_name":"A","members":["a"],"extra":true}),
        json!({"kind":"named_string","type_id":"vty_a","revision":4294967296_u64,"display_name":"A","members":["a"]}),
        json!({"kind":"named_string","type_id":"vty_a","revision":1,"display_name":"A","members":[1]}),
        json!({"kind":"named_string","type_id":"vty_a","revision":1,"members":["a"]}),
    ] {
        let mut wire = original.clone();
        wire[0]["properties"][index]["value_type"] = declaration;
        assert!(serde_json::from_value::<Vec<EntitySchema>>(wire).is_err());
    }
    for (field, invalid) in [
        ("type_id", json!("vty_")),
        ("type_id", json!("ety_a")),
        ("type_id", json!(format!("vty_{}", "x".repeat(4093)))),
        ("revision", json!(0)),
        ("display_name", json!("\u{85}")),
        ("display_name", json!("x".repeat(4097))),
        ("members", json!([])),
        ("members", json!(["a", "a"])),
        ("members", json!(["\u{85}"])),
        ("members", json!(["x".repeat(4097)])),
        (
            "members",
            json!((0..257).map(|i| i.to_string()).collect::<Vec<_>>()),
        ),
    ] {
        let mut wire = original.clone();
        wire[0]["properties"][index]["value_type"][field] = invalid;
        let rows: Vec<EntitySchema> = serde_json::from_value(wire).unwrap();
        assert_eq!(
            rows[0].validate(),
            Err(OntologyReadError::SourceUnavailable),
            "{field}"
        );
    }
    let named = original[0]["properties"][index]["value_type"].clone();
    for (tier, declaration) in [
        ("vector", json!({"kind":"array","element":named})),
        (
            "struct",
            json!({"kind":"struct","fields":[{"name":"a","required":false,"value_type":named}]}),
        ),
    ] {
        let mut wire = original.clone();
        wire[0]["properties"][index]["tier"] = json!(tier);
        wire[0]["properties"][index]["value_type"] = declaration;
        let rows: Vec<EntitySchema> = serde_json::from_value(wire).unwrap();
        assert_eq!(
            rows[0].validate(),
            Err(OntologyReadError::SourceUnavailable)
        );
    }
}
