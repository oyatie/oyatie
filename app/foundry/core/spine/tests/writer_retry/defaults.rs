use super::*;
use data_ontology_kernel::{
    ScalarType, StructFieldDeclaration, StructSchema, ValueTypeDeclaration,
};
use foundry_edits::decode_action_record;
use foundry_spine::fold_from_scratch;

fn default_property(name: &str, value: PropertyValue) -> EntityTypePropertyDefinition {
    let kind = match &value {
        PropertyValue::Boolean(_) => ScalarType::Boolean,
        PropertyValue::Integer(_) => ScalarType::Integer,
        _ => ScalarType::String,
    };
    let mut property = EntityTypePropertyDefinition::typed(
        name,
        ValueTypeDeclaration::Scalar(kind),
        internal(),
        false,
    )
    .unwrap();
    property.default_value = Some(value);
    property
}

fn revise(engine: &mut OntologyEngine, edit: impl FnOnce(&mut EntityTypeDefinition)) {
    let id = EntityTypeId::new("ety_reading").unwrap();
    let mut definition = engine.entity_type("ten_test", &id).unwrap().clone();
    definition.revision += 1;
    edit(&mut definition);
    engine.evolve_entity_type(definition).unwrap();
}

fn send(
    submission: ActionSubmission,
    log: &mut MemoryLog,
    denials: &mut MemoryLog,
    state: &mut ProjectionState,
) -> ApplyOutcome {
    submit(submission, log, denials, state).unwrap()
}

#[test]
fn omitted_defaults_are_explicit_v2_but_provided_false_zero_and_empty_win() {
    let mut engine = registry();
    revise(&mut engine, |definition| {
        definition.properties.extend([
            default_property("flag", PropertyValue::Boolean(true)),
            default_property("count", PropertyValue::Integer(8)),
            default_property("text", PropertyValue::String("preset".into())),
        ])
    });
    let mut state = ProjectionState::new("ten_test", &engine);
    let (mut log, mut denials) = (MemoryLog::default(), MemoryLog::default());
    let raw = create_submission("defaulted");
    send(raw.clone(), &mut log, &mut denials, &mut state);
    let record = decode_action_record(&log.entries[0].envelope.payload).unwrap();
    assert_eq!(record.wire_format_version, 2);
    assert_eq!(record.submitted_edits, Some(raw.edits));
    let object = state.objects.get("ten_test", "ent_r1").unwrap();
    assert_eq!(
        object.properties["flag"].value.value,
        PropertyValue::Boolean(true)
    );
    assert_eq!(
        object.properties["count"].value.value,
        PropertyValue::Integer(8)
    );
    assert_eq!(fold_from_scratch("ten_test", &engine, &log.entries), state);
    let mut explicit = create_submission("explicit");
    explicit.request.entity_id = "ent_explicit".into();
    explicit.edits = EditSet::new(vec![
        OntologyEdit::create_object(
            "ety_reading",
            vec![
                name_property("Ada"),
                WireProperty::new(
                    "flag",
                    WireTier::Scalar,
                    WireDataClass::InternalOnly,
                    WireValue::Boolean(false),
                )
                .unwrap(),
                WireProperty::new(
                    "count",
                    WireTier::Scalar,
                    WireDataClass::InternalOnly,
                    WireValue::Integer(0),
                )
                .unwrap(),
                WireProperty::new(
                    "text",
                    WireTier::Scalar,
                    WireDataClass::InternalOnly,
                    WireValue::String(String::new()),
                )
                .unwrap(),
            ],
        )
        .unwrap(),
    ])
    .unwrap();
    send(explicit, &mut log, &mut denials, &mut state);
    let object = state.objects.get("ten_test", "ent_explicit").unwrap();
    assert_eq!(
        object.properties["flag"].value.value,
        PropertyValue::Boolean(false)
    );
    assert_eq!(
        object.properties["count"].value.value,
        PropertyValue::Integer(0)
    );
    assert_eq!(
        object.properties["text"].value.value,
        PropertyValue::String(String::new())
    );
    assert_eq!(
        decode_action_record(&log.entries[1].envelope.payload)
            .unwrap()
            .wire_format_version,
        1
    );
}

#[test]
fn replacements_updates_and_later_create_edits_never_get_defaults() {
    let mut engine = registry();
    revise(&mut engine, |definition| {
        definition.properties.push(default_property(
            "note",
            PropertyValue::String("preset".into()),
        ))
    });
    let mut state = ProjectionState::new("ten_test", &engine);
    let (mut log, mut denials) = (MemoryLog::default(), MemoryLog::default());
    send(
        create_submission("first"),
        &mut log,
        &mut denials,
        &mut state,
    );
    send(
        create_submission("replace"),
        &mut log,
        &mut denials,
        &mut state,
    );
    assert!(
        !state
            .objects
            .get("ten_test", "ent_r1")
            .unwrap()
            .properties
            .contains_key("note")
    );
    let after_replacement = state.clone();
    let ApplyOutcome::Applied { receipt } = send(
        create_submission("first"),
        &mut log,
        &mut denials,
        &mut state,
    ) else {
        panic!()
    };
    assert!(receipt.deduplicated);
    assert_eq!(state, after_replacement);
    let mut update = create_submission("update");
    update.edits = EditSet::new(vec![
        OntologyEdit::upsert_properties(vec![name_property("Grace")]).unwrap(),
    ])
    .unwrap();
    send(update, &mut log, &mut denials, &mut state);
    assert!(
        !state
            .objects
            .get("ten_test", "ent_r1")
            .unwrap()
            .properties
            .contains_key("note")
    );
    let mut batch = create_submission("batch");
    batch.request.entity_id = "ent_batch".into();
    batch.edits = EditSet::new(vec![
        OntologyEdit::create_object("ety_reading", vec![name_property("Ada")]).unwrap(),
        OntologyEdit::create_object("ety_reading", vec![name_property("Grace")]).unwrap(),
    ])
    .unwrap();
    send(batch, &mut log, &mut denials, &mut state);
    let record = decode_action_record(&log.entries[3].envelope.payload).unwrap();
    let sizes: Vec<_> = record
        .edits
        .edits()
        .iter()
        .map(|edit| {
            let OntologyEdit::CreateObject { properties, .. } = edit else {
                unreachable!()
            };
            properties.len()
        })
        .collect();
    assert_eq!(sizes, [2, 1]);
    assert!(
        !state
            .objects
            .get("ten_test", "ent_batch")
            .unwrap()
            .properties
            .contains_key("note")
    );
}

#[path = "defaults/admission.rs"]
mod admission;
#[path = "defaults/evolution.rs"]
mod evolution;
