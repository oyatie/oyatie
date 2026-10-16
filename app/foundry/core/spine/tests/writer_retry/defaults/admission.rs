use super::*;

#[test]
fn defaults_satisfy_required_creation_but_cannot_mask_explicit_invalid_values() {
    let mut engine = registry();
    revise(&mut engine, |definition| {
        definition.properties[0].default_value = Some(PropertyValue::String("preset".into()))
    });
    let mut state = ProjectionState::new("ten_test", &engine);
    let (mut log, mut denials) = (MemoryLog::default(), MemoryLog::default());
    let mut omitted = create_submission("omitted");
    omitted.edits = EditSet::new(vec![
        OntologyEdit::create_object("ety_reading", vec![]).unwrap(),
    ])
    .unwrap();
    send(omitted, &mut log, &mut denials, &mut state);
    let mut invalid = create_submission("invalid");
    invalid.request.entity_id = "ent_invalid".into();
    invalid.edits = EditSet::new(vec![
        OntologyEdit::create_object(
            "ety_reading",
            vec![
                WireProperty::new(
                    "name",
                    WireTier::Scalar,
                    WireDataClass::InternalOnly,
                    WireValue::Integer(0),
                )
                .unwrap(),
            ],
        )
        .unwrap(),
    ])
    .unwrap();
    assert!(matches!(
        submit(invalid, &mut log, &mut denials, &mut state),
        Err(WriteError::Refused(_))
    ));
    assert_eq!(log.entries.len(), 1);
    let mut replacement = create_submission("empty-replacement");
    replacement.edits = EditSet::new(vec![
        OntologyEdit::create_object("ety_reading", vec![]).unwrap(),
    ])
    .unwrap();
    assert!(matches!(
        submit(replacement, &mut log, &mut denials, &mut state),
        Err(WriteError::Refused(_))
    ));
    assert_eq!(log.entries.len(), 1);
}

#[test]
fn supplied_structs_are_validated_whole_and_legacy_tier_and_class_are_preserved() {
    let mut engine = registry();
    let fields = ["a", "b"].map(|name| StructFieldDeclaration {
        name: name.into(),
        required: true,
        value_type: ValueTypeDeclaration::Scalar(ScalarType::String),
    });
    let mut composite = EntityTypePropertyDefinition::typed(
        "config",
        ValueTypeDeclaration::Struct(StructSchema {
            fields: fields.to_vec(),
        }),
        internal(),
        false,
    )
    .unwrap();
    composite.default_value = Some(PropertyValue::Struct(
        [
            ("a".into(), PropertyValue::String("a".into())),
            ("b".into(), PropertyValue::String("b".into())),
        ]
        .into(),
    ));
    let mut legacy = EntityTypePropertyDefinition::new(
        "geo",
        PropertyTier::Geo,
        PrivacyDataClass::try_from(DataClass::PiiIdentifying).unwrap(),
        false,
    )
    .unwrap();
    legacy.default_value = Some(PropertyValue::String("legacy".into()));
    revise(&mut engine, |definition| {
        definition.properties.extend([composite, legacy])
    });
    let mut state = ProjectionState::new("ten_test", &engine);
    let (mut log, mut denials) = (MemoryLog::default(), MemoryLog::default());
    send(
        create_submission("omitted"),
        &mut log,
        &mut denials,
        &mut state,
    );
    let property = &state.objects.get("ten_test", "ent_r1").unwrap().properties["geo"];
    assert_eq!(property.tier, PropertyTier::Geo);
    assert_eq!(
        property.value.data_class,
        data_boundary_kernel::DataClassification::from(DataClass::PiiIdentifying)
    );
    let mut partial = create_submission("partial");
    partial.request.entity_id = "ent_partial".into();
    partial.edits = EditSet::new(vec![
        OntologyEdit::create_object(
            "ety_reading",
            vec![
                name_property("Ada"),
                WireProperty::new(
                    "config",
                    WireTier::Struct,
                    WireDataClass::InternalOnly,
                    WireValue::Struct([("a".into(), WireValue::String("supplied".into()))].into()),
                )
                .unwrap(),
            ],
        )
        .unwrap(),
    ])
    .unwrap();
    assert!(matches!(
        submit(partial, &mut log, &mut denials, &mut state),
        Err(WriteError::Refused(_))
    ));
    assert_eq!(log.entries.len(), 1);
}

#[test]
fn materialized_primary_keys_keep_the_existing_projection_uniqueness_gate() {
    use foundry_projection_draft::{MemoryProjectionStore, ProjectionStore, ProjectionStoreError};
    let mut engine = registry();
    revise(&mut engine, |definition| {
        definition.primary_key_property = Some("name".into());
        definition.properties[0].default_value = Some(PropertyValue::String("same-key".into()));
    });
    let mut state = ProjectionState::new("ten_test", &engine);
    let (mut log, mut denials) = (MemoryLog::default(), MemoryLog::default());
    let mut store = MemoryProjectionStore::default();
    for (id, expected_ok) in [("ent_first", true), ("ent_duplicate", false)] {
        let mut raw = create_submission(id);
        raw.request.entity_id = id.into();
        raw.edits = EditSet::new(vec![
            OntologyEdit::create_object("ety_reading", vec![]).unwrap(),
        ])
        .unwrap();
        let result =
            foundry_spine::submit_through(raw, &mut log, &mut denials, &mut state, &mut store)
                .unwrap();
        if expected_ok {
            assert!(result.mirror.is_ok());
        } else {
            assert!(matches!(
                result.mirror,
                Err(foundry_spine::WriteThroughError::Store {
                    error: ProjectionStoreError::DuplicatePrimaryKey { .. },
                    ..
                })
            ));
        }
    }
    assert_eq!(store.applied_head("ten_test").unwrap(), 1);
    assert!(store.get("ten_test", "ent_duplicate").unwrap().is_none());
}
