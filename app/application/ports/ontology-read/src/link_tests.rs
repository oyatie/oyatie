use serde_json::json;

use super::*;

fn wire(cardinality: &str) -> serde_json::Value {
    json!({"link_type":"lty_related","from_entity_type":"ety_record",
        "to_entity_type":"ety_record","cardinality":cardinality,
        "allow_cross_tenant":false,"revision":0})
}

#[test]
fn native_cardinalities_self_links_and_zero_revision_round_trip() {
    for cardinality in ["one_to_one", "one_to_many", "many_to_many"] {
        let wire = wire(cardinality);
        let declaration: LinkSchema = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(declaration.validate(), Ok(()));
        assert_eq!(serde_json::to_value(declaration).unwrap(), wire);
    }
}

#[test]
fn raw_catalog_rows_require_objects_and_duplicate_keys_remain_refused() {
    for wire in [
        r#"[["lty_related","ety_record","ety_record","one_to_many",false,0]]"#,
        r#"[{"link_type":"lty_related","link_type":"lty_other","from_entity_type":"ety_record",
            "to_entity_type":"ety_record","cardinality":"one_to_many","allow_cross_tenant":false,"revision":0}]"#,
    ] {
        assert!(serde_json::from_str::<Vec<LinkSchema>>(wire).is_err());
    }
}

#[test]
fn declarations_require_closed_typed_fields_and_canonical_cardinality() {
    for field in [
        "link_type",
        "from_entity_type",
        "to_entity_type",
        "cardinality",
        "allow_cross_tenant",
        "revision",
    ] {
        let mut missing = wire("one_to_many");
        missing.as_object_mut().unwrap().remove(field);
        assert!(
            serde_json::from_value::<LinkSchema>(missing).is_err(),
            "{field}"
        );
    }
    for (field, value) in [
        ("extra", json!(true)),
        ("cardinality", json!("OneToMany")),
        ("cardinality", json!("many_to_one")),
        ("cardinality", json!({"one_to_many":null})),
        ("allow_cross_tenant", json!("false")),
        ("revision", json!(-1)),
        ("revision", json!(4294967296_u64)),
        ("revision", json!(1.5)),
    ] {
        let mut invalid = wire("one_to_many");
        invalid[field] = value;
        assert!(
            serde_json::from_value::<LinkSchema>(invalid).is_err(),
            "{field}"
        );
    }
    for (field, value) in [
        ("link_type", "lty_"),
        ("link_type", "ety_wrong"),
        ("from_entity_type", "ent_wrong"),
        ("to_entity_type", "ety_"),
    ] {
        let mut invalid = wire("one_to_many");
        invalid[field] = json!(value);
        assert_eq!(
            serde_json::from_value::<LinkSchema>(invalid)
                .unwrap()
                .validate(),
            Err(OntologyReadError::SourceUnavailable)
        );
    }
}
