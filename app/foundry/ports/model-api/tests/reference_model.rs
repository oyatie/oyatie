use foundry_model_api::{conformance::*, *};

#[test]
fn reference_runs_owner_only_cursor_discovery_including_closed_drafts() {
    let mut store = ReferenceModelStore::initialize(binding(), baseline()).unwrap();
    check_owned_draft_discovery(&mut store).unwrap();
    let opened = store.open_existing(&binding(), &baseline()).unwrap();
    let actor = ModelCaller {
        owner: "usr_author\0one".into(),
        ..caller()
    };
    let request = DraftPageRequest {
        limit: 1,
        after: Some(2),
    };
    assert_eq!(
        store.owned_drafts(&actor, &request),
        opened.owned_drafts(&actor, &request)
    );
}

#[test]
fn reference_runs_shared_owned_lifecycle() {
    let mut store = ReferenceModelStore::initialize(binding(), baseline()).unwrap();
    check_owned_versioned_lifecycle(&mut store).unwrap();
    let opened = store.open_existing(&binding(), &baseline()).unwrap();
    assert_eq!(
        opened.history(&caller()).unwrap(),
        store.history(&caller()).unwrap()
    );
    let mut mismatch = binding();
    mismatch.baseline_id = "other".into();
    assert!(matches!(
        store.open_existing(&mismatch, &baseline()),
        Err(ModelError::BindingMismatch)
    ));
}

#[test]
fn reference_runs_shared_two_handle_conflicts_and_original_retry_after_later_publish() {
    let mut left = ReferenceModelStore::initialize(binding(), baseline()).unwrap();
    let mut right = left.clone();
    check_two_handle_conflicts(&mut left, &mut right).unwrap();
}

#[test]
fn incomplete_shape_is_saved_but_bounded_closed_wire_is_required() {
    let mut document = baseline().document;
    document.entities[0].id.clear();
    document.entities[0].properties.clear();
    document.entities[0].display_name.clear();
    assert_eq!(
        ModelDocument::decode(&document.encode().unwrap()).unwrap(),
        document
    );
    let mut store = ReferenceModelStore::initialize(binding(), baseline()).unwrap();
    let draft = store.create_draft(&caller(), 1, document).unwrap();
    assert!(
        store
            .prepare(&caller(), draft.id, draft.version, draft.base_generation)
            .is_err()
    );
    for wire in [
        r#"[[],[]]"#,
        r#"{"entities":[],"links":[],"extra":1}"#,
        r#"{"entities":[],"entities":[],"links":[]}"#,
        r#"{"entities":[]}"#,
    ] {
        assert!(ModelDocument::decode(wire.as_bytes()).is_err(), "{wire}");
    }
    let mut value = serde_json::to_value(baseline().document).unwrap();
    value["entities"][0]["properties"][0]["data_class"] = "SECRET".into();
    assert!(ModelDocument::decode(&serde_json::to_vec(&value).unwrap()).is_err());
    value = serde_json::to_value(baseline().document).unwrap();
    value["entities"][0] = serde_json::json!([]);
    assert!(ModelDocument::decode(&serde_json::to_vec(&value).unwrap()).is_err());
}

#[test]
fn bounds_refuse_oversized_text_document_and_value_depth_before_native_conversion() {
    let mut model = baseline().document;
    model.entities[0].display_name = "x".repeat(MAX_MODEL_TEXT_BYTES + 1);
    assert!(matches!(
        model.check_bounds(),
        Err(ModelError::Limit { .. })
    ));
    assert!(matches!(
        ModelDocument::decode(&vec![b' '; MAX_MODEL_BYTES + 1]),
        Err(ModelError::Limit { .. })
    ));
    let mut kind = ModelValueType::Scalar {
        kind: ModelScalar::String,
    };
    for _ in 0..8 {
        kind = ModelValueType::Array {
            element: Box::new(kind),
        };
    }
    model = baseline().document;
    model.entities[0].properties[0].value_type = Some(kind);
    assert!(matches!(
        model.check_bounds(),
        Err(ModelError::Limit { .. })
    ));
}
