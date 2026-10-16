use super::*;

#[test]
fn retries_keep_original_input_across_default_change_removal_and_addition() {
    let mut engine = registry();
    let (mut log, mut denials) = (MemoryLog::default(), MemoryLog::default());
    let mut state = ProjectionState::new("ten_test", &engine);
    let legacy = create_submission("legacy");
    send(legacy.clone(), &mut log, &mut denials, &mut state);
    revise(&mut engine, |definition| {
        definition.properties.push(default_property(
            "note",
            PropertyValue::String("first".into()),
        ))
    });
    state = fold_from_scratch("ten_test", &engine, &log.entries);
    let mut raw = create_submission("defaulted");
    raw.request.entity_id = "ent_new".into();
    send(raw.clone(), &mut log, &mut denials, &mut state);
    let accepted = log.entries.clone();
    for value in [Some("changed"), None, Some("added-again")] {
        revise(&mut engine, |definition| {
            definition
                .properties
                .iter_mut()
                .find(|property| property.name == "note")
                .unwrap()
                .default_value = value.map(|value| PropertyValue::String(value.into()));
        });
        state = fold_from_scratch("ten_test", &engine, &log.entries);
        let before = state.clone();
        for original in [&raw, &legacy] {
            let mut retry = original.clone();
            retry.decision.decision_id = "fresh-decision".into();
            let ApplyOutcome::Applied { receipt } = send(retry, &mut log, &mut denials, &mut state)
            else {
                panic!()
            };
            assert!(receipt.deduplicated);
        }
        assert_eq!(log.entries, accepted);
        assert_eq!(state, before);
    }
    let mut explicit_equal = raw.clone();
    explicit_equal.edits = EditSet::new(vec![
        OntologyEdit::create_object(
            "ety_reading",
            vec![name_property("Ada"), name_property("first")],
        )
        .unwrap(),
    ])
    .unwrap();
    // Change only omitted note to an explicitly supplied value equal to its original default.
    let mut edits = explicit_equal.edits.edits().to_vec();
    let OntologyEdit::CreateObject { properties, .. } = &mut edits[0] else {
        unreachable!()
    };
    properties[1].name = "note".into();
    explicit_equal.edits = EditSet::new(edits).unwrap();
    assert!(matches!(
        submit(explicit_equal, &mut log, &mut denials, &mut state),
        Err(WriteError::Log(RecordsLogError::IdempotencyConflict { .. }))
    ));
    let mut denied = raw;
    denied.decision.allowed_surfaces.clear();
    assert!(
        matches!(submit(denied, &mut log, &mut denials, &mut state), Err(WriteError::Refused(refused)) if refused.gate == RefusalGate::Authorization)
    );
    assert_eq!(log.entries, accepted);
}
