//! Reader pinning over a mixed-revision projection: filter-down for pins
//! behind the write, honest absence plus `UpcastPending` for pins ahead of
//! it, typed refusals for unknown objects and unretained pins — and the
//! additive-safety tripwire: head-bound instance conformance is safe iff
//! evolution stays additive, so any future breaking-shape door must ship
//! revision-pinned conformance in the same change.

use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    ActionTypeDefinition, ActionTypeId, AutonomyTier, EntityTypeDefinition, EntityTypeId,
    EntityTypePropertyDefinition, ObjectEntity, ObjectProperty, OntologyEngine, PropertyTier,
    PropertyValue,
};
use foundry_edits::{
    ActionRecord, EditSet, OntologyEdit, WireDataClass, WireProperty, WireTier, WireValue,
    encode_action_record,
};
use foundry_records_draft::{ActionEnvelope, Receipt, SealedEnvelope};
use foundry_spine::{
    ProjectionState, UpcastState, ViewError, fold_from_scratch, object_at_revision,
};

fn internal() -> PrivacyDataClass {
    PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap()
}

fn name_definition() -> EntityTypePropertyDefinition {
    EntityTypePropertyDefinition::new("name", PropertyTier::Scalar, internal(), true).unwrap()
}

fn grade_definition() -> EntityTypePropertyDefinition {
    EntityTypePropertyDefinition::new("grade", PropertyTier::Scalar, internal(), false).unwrap()
}

/// Revision 1 declares required `name`; revision 2 adds optional `grade`
/// (the blessed additive idiom). Both revisions are retained.
fn registry_two_revisions() -> OntologyEngine {
    let mut engine = OntologyEngine::default();
    engine
        .register_entity_type(
            EntityTypeDefinition::new(
                "ten_test",
                EntityTypeId::new("ety_reading").unwrap(),
                "Reading",
                vec![name_definition()],
                1,
            )
            .unwrap(),
        )
        .unwrap();
    engine
        .evolve_entity_type(
            EntityTypeDefinition::new(
                "ten_test",
                EntityTypeId::new("ety_reading").unwrap(),
                "Reading",
                vec![name_definition(), grade_definition()],
                2,
            )
            .unwrap(),
        )
        .unwrap();
    engine
        .register_action_type(
            ActionTypeDefinition::new(
                "ten_test",
                ActionTypeId::new("aty_calibrate").unwrap(),
                EntityTypeId::new("ety_reading").unwrap(),
                "ops-console",
                AutonomyTier::T1Assist,
                "reading.calibrated",
            )
            .unwrap(),
        )
        .unwrap();
    engine
}

fn wire_string(name: &str, value: &str) -> WireProperty {
    WireProperty::new(
        name,
        WireTier::Scalar,
        WireDataClass::InternalOnly,
        WireValue::String(value.into()),
    )
    .unwrap()
}

fn sealed_at(
    object_ref: &str,
    ordinal: u64,
    schema_revision: u32,
    edits: Vec<OntologyEdit>,
) -> SealedEnvelope {
    let key = format!("idem_{ordinal}");
    let record = ActionRecord::new(
        "prn_alice",
        "dec_1",
        "reading.calibrated",
        &key,
        1_700_000_000_000,
        vec![],
        EditSet::new(edits).unwrap(),
    )
    .unwrap();
    SealedEnvelope {
        envelope: ActionEnvelope::new(
            "ten_test",
            object_ref,
            "aty_calibrate",
            &key,
            schema_revision,
            encode_action_record(&record),
            1_700_000_000_000,
        )
        .unwrap(),
        receipt: Receipt {
            ordinal,
            object_sequence: 1,
            deduplicated: false,
        },
    }
}

/// `ent_old` written under revision 1 (name only); `ent_new` written under
/// revision 2 (name + grade). Both entries must APPLY — a poisoned fixture
/// would silently hollow every assertion downstream.
fn mixed_revision_state() -> ProjectionState {
    let registry = registry_two_revisions();
    let old = sealed_at(
        "ent_old",
        1,
        1,
        vec![OntologyEdit::create_object("ety_reading", vec![wire_string("name", "Ada")]).unwrap()],
    );
    let new = sealed_at(
        "ent_new",
        2,
        2,
        vec![
            OntologyEdit::create_object(
                "ety_reading",
                vec![wire_string("name", "Grace"), wire_string("grade", "A")],
            )
            .unwrap(),
        ],
    );
    let state = fold_from_scratch("ten_test", &registry, [&old, &new]);
    assert!(
        state.poison.is_empty(),
        "fixture must fold clean: {:?}",
        state.poison
    );
    assert_eq!(state.applied_ordinal, 2);
    state
}

#[test]
fn pinned_ahead_of_written_shows_honest_absence_and_pending() {
    let state = mixed_revision_state();
    let pinned = object_at_revision(&state, "ent_old", 2).unwrap();
    assert_eq!(pinned.written_revision, 1);
    assert_eq!(pinned.upcast_state, UpcastState::UpcastPending);
    assert_eq!(
        pinned.properties["name"].value.value,
        PropertyValue::String("Ada".into())
    );
    assert!(
        !pinned.properties.contains_key("grade"),
        "a value the log never carried must not be synthesized at read"
    );
}

#[test]
fn pinned_behind_written_filters_down_to_pinned_vocabulary() {
    let state = mixed_revision_state();
    let pinned = object_at_revision(&state, "ent_new", 1).unwrap();
    assert_eq!(pinned.written_revision, 2);
    assert_eq!(pinned.upcast_state, UpcastState::Current);
    assert_eq!(
        pinned.properties["name"].value.value,
        PropertyValue::String("Grace".into())
    );
    assert!(
        !pinned.properties.contains_key("grade"),
        "a revision-1 reader must see only revision-1 vocabulary"
    );
}

#[test]
fn pinned_at_written_revision_is_current_and_complete() {
    let state = mixed_revision_state();
    let pinned = object_at_revision(&state, "ent_new", 2).unwrap();
    assert_eq!(pinned.written_revision, 2);
    assert_eq!(pinned.upcast_state, UpcastState::Current);
    assert_eq!(
        pinned.properties["grade"].value.value,
        PropertyValue::String("A".into())
    );
}

#[test]
fn unretained_pin_is_a_typed_refusal_never_a_poison() {
    let state = mixed_revision_state();
    assert_eq!(
        object_at_revision(&state, "ent_old", 3),
        Err(ViewError::UnretainedRevision)
    );
    assert!(
        state.poison.is_empty(),
        "a read may never write the poison ledger"
    );
}

#[test]
fn unknown_object_is_a_typed_refusal() {
    let state = mixed_revision_state();
    assert_eq!(
        object_at_revision(&state, "ent_ghost", 1),
        Err(ViewError::UnknownObject)
    );
}

#[test]
fn old_shaped_instance_conforms_at_head_because_evolution_is_additive() {
    let engine = registry_two_revisions();
    let old_shaped = ObjectEntity::new(
        "ten_test".into(),
        "ent_shape".into(),
        "ety_reading".into(),
        vec![ObjectProperty::new(
            "name".into(),
            "Ada".into(),
            PropertyTier::Scalar,
            internal(),
        )],
    )
    .unwrap();
    assert_eq!(engine.check_instance_conformance(&old_shaped), Ok(()));
}
