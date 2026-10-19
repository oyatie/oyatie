use super::*;

fn string(bytes: usize) -> ModelValue {
    ModelValue::String {
        value: "x".repeat(bytes),
    }
}

#[test]
fn literal_bounds_cover_text_depth_items_and_total_model_bytes() {
    assert!(document(string(MAX_MODEL_TEXT_BYTES)).encode().is_ok());
    assert!(matches!(
        document(string(MAX_MODEL_TEXT_BYTES + 1)).encode(),
        Err(ModelError::Limit { .. })
    ));
    let mut nested = string(0);
    for _ in 1..data_ontology_kernel::MAX_VALUE_TYPE_DEPTH {
        nested = ModelValue::Array {
            items: vec![nested],
        };
    }
    assert!(document(nested.clone()).encode().is_ok());
    assert!(matches!(
        document(ModelValue::Array {
            items: vec![nested]
        })
        .encode(),
        Err(ModelError::Limit { .. })
    ));
    assert!(
        document(ModelValue::Array {
            items: vec![string(0); MAX_MODEL_ITEMS]
        })
        .encode()
        .is_ok()
    );
    assert!(matches!(
        document(ModelValue::Array {
            items: vec![string(0); MAX_MODEL_ITEMS + 1]
        })
        .encode(),
        Err(ModelError::Limit { .. })
    ));
    let fields = (0..MAX_MODEL_ITEMS)
        .map(|index| (index.to_string(), string(0)))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert!(
        document(ModelValue::Struct {
            fields: fields.clone()
        })
        .encode()
        .is_ok()
    );
    let mut too_many = fields.clone();
    too_many.insert("extra".into(), string(0));
    assert!(matches!(
        document(ModelValue::Struct { fields: too_many }).encode(),
        Err(ModelError::Limit { .. })
    ));
    let mut long_name = fields;
    let (_, value) = long_name.pop_first().unwrap();
    long_name.insert("x".repeat(MAX_MODEL_TEXT_BYTES + 1), value);
    assert!(matches!(
        document(ModelValue::Struct { fields: long_name }).encode(),
        Err(ModelError::Limit { .. })
    ));
    assert!(
        matches!(document(ModelValue::Array { items: vec![string(MAX_MODEL_TEXT_BYTES); MAX_MODEL_ITEMS] }).encode(), Err(ModelError::Limit { path }) if path == "document")
    );
    assert!(
        matches!(ModelDocument::decode(&vec![b' '; MAX_MODEL_BYTES + 1]), Err(ModelError::Limit { path }) if path == "document")
    );
}

#[test]
fn literal_nodes_share_the_document_budget_with_declarations_and_other_defaults() {
    let value = ModelValue::Array {
        items: vec![
            ModelValue::Array {
                items: vec![string(0); MAX_MODEL_ITEMS]
            };
            8
        ],
    };
    let mut model = document(value.clone());
    assert!(model.encode().is_ok());
    model.entities[0].properties[0].default_value = Some(value);
    assert!(matches!(model.encode(), Err(ModelError::Limit { .. })));
    let bytes = serde_json::to_vec(&model).unwrap();
    assert!(matches!(
        ModelDocument::decode(&bytes),
        Err(ModelError::Limit { .. })
    ));
}

#[test]
fn programmatic_literals_refuse_invalid_dates() {
    assert_eq!(
        document(ModelValue::Date {
            year: 1900,
            month: 2,
            day: 29
        })
        .encode(),
        Err(ModelError::MalformedDocument)
    );
}
