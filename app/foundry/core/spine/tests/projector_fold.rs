//! The fold's behavior matrix: apply, poison-never-wedge, all-or-nothing
//! per entry, kernel law (conformance, cardinality) re-checked
//! authoritatively at fold time.

use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    ActionTypeDefinition, ActionTypeId, AutonomyTier, EntityTypeDefinition, EntityTypeId,
    EntityTypePropertyDefinition, LinkCardinality, LinkTypeDefinition, LinkTypeId, OntologyEngine,
    OntologyEngineError, PropertyTier, PropertyValue,
};
use foundry_edits::{
    ActionRecord, EditSet, OntologyEdit, WireDataClass, WireProperty, WireTier, WireValue,
    encode_action_record,
};
use foundry_records_draft::{ActionEnvelope, Receipt, SealedEnvelope};
use foundry_spine::{FoldOutcome, PoisonReason, ProjectionState, apply_sealed, fold_from_scratch};

fn internal() -> PrivacyDataClass {
    PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap()
}

fn registry() -> OntologyEngine {
    let mut engine = OntologyEngine::default();
    engine
        .register_entity_type(
            EntityTypeDefinition::new(
                "ten_test",
                EntityTypeId::new("ety_reading").unwrap(),
                "Reading",
                vec![
                    EntityTypePropertyDefinition::new(
                        "name",
                        PropertyTier::Scalar,
                        internal(),
                        true,
                    )
                    .unwrap(),
                ],
                1,
            )
            .unwrap(),
        )
        .unwrap();
    engine
        .register_link_type(
            LinkTypeDefinition::new(
                "ten_test",
                LinkTypeId::new("lty_measures").unwrap(),
                EntityTypeId::new("ety_reading").unwrap(),
                EntityTypeId::new("ety_reading").unwrap(),
                LinkCardinality::OneToOne,
                false,
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

fn name_property(value: &str) -> WireProperty {
    WireProperty::new(
        "name",
        WireTier::Scalar,
        WireDataClass::InternalOnly,
        WireValue::String(value.into()),
    )
    .unwrap()
}

fn record(idempotency_key: &str, edits: Vec<OntologyEdit>) -> ActionRecord {
    ActionRecord::new(
        "prn_alice",
        "dec_1",
        "reading.calibrated",
        idempotency_key,
        1_700_000_000_000,
        vec![],
        EditSet::new(edits).unwrap(),
    )
    .unwrap()
}

fn sealed_with(
    object_ref: &str,
    ordinal: u64,
    idempotency_key: &str,
    payload: Vec<u8>,
) -> SealedEnvelope {
    SealedEnvelope {
        envelope: ActionEnvelope::new(
            "ten_test",
            object_ref,
            "aty_calibrate",
            idempotency_key,
            1,
            payload,
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

fn sealed(object_ref: &str, ordinal: u64, edits: Vec<OntologyEdit>) -> SealedEnvelope {
    let key = format!("idem_{ordinal}");
    sealed_with(
        object_ref,
        ordinal,
        &key,
        encode_action_record(&record(&key, edits)),
    )
}

fn create_reading(object_ref: &str, ordinal: u64) -> SealedEnvelope {
    sealed(
        object_ref,
        ordinal,
        vec![OntologyEdit::create_object("ety_reading", vec![name_property("Ada")]).unwrap()],
    )
}

#[test]
fn create_object_applies_and_binds() {
    let registry = registry();
    let mut state = ProjectionState::new("ten_test", &registry);
    let outcome = apply_sealed(&mut state, &create_reading("ent_r1", 1));
    assert_eq!(outcome, FoldOutcome::Applied);
    let entity = state.objects.get("ten_test", "ent_r1").expect("projected");
    assert_eq!(
        entity.properties["name"].value.value,
        PropertyValue::String("Ada".into())
    );
    let binding = &state.bindings["ent_r1"];
    assert_eq!(binding.entity_type, "ety_reading");
    assert_eq!(binding.schema_revision, 1);
    assert_eq!(binding.last_ordinal, 1);
    assert_eq!(binding.last_actor, "prn_alice");
    assert_eq!(state.history["ent_r1"], vec![1]);
    assert_eq!(state.applied_ordinal, 1);
    assert!(state.poison.is_empty());
}

#[test]
fn upsert_on_missing_object_poisons_without_wedging() {
    let registry = registry();
    let mut state = ProjectionState::new("ten_test", &registry);
    let entry = sealed(
        "ent_ghost",
        1,
        vec![OntologyEdit::upsert_properties(vec![name_property("x")]).unwrap()],
    );
    assert_eq!(
        apply_sealed(&mut state, &entry),
        FoldOutcome::Poisoned(PoisonReason::MissingObject)
    );
    assert_eq!(state.applied_ordinal, 1);
    assert_eq!(state.poison[&1], PoisonReason::MissingObject);
    assert!(state.objects.is_empty());
    // The fold never wedges: the next entry still applies.
    assert_eq!(
        apply_sealed(&mut state, &create_reading("ent_r1", 2)),
        FoldOutcome::Applied
    );
}

#[test]
fn an_entry_applies_whole_or_not_at_all() {
    let registry = registry();
    let mut state = ProjectionState::new("ten_test", &registry);
    apply_sealed(&mut state, &create_reading("ent_r1", 1));
    let mixed = sealed(
        "ent_r1",
        2,
        vec![
            OntologyEdit::upsert_properties(vec![name_property("Renamed")]).unwrap(),
            OntologyEdit::create_link("lty_unknown", "ent_r1").unwrap(),
        ],
    );
    assert_eq!(
        apply_sealed(&mut state, &mixed),
        FoldOutcome::Poisoned(PoisonReason::Link(OntologyEngineError::UnknownLinkType))
    );
    // The valid first edit must NOT have landed.
    let entity = state.objects.get("ten_test", "ent_r1").unwrap();
    assert_eq!(
        entity.properties["name"].value.value,
        PropertyValue::String("Ada".into())
    );
    assert_eq!(state.bindings["ent_r1"].last_ordinal, 1);
}

#[test]
fn kernel_cardinality_law_holds_at_fold() {
    let registry = registry();
    let mut state = ProjectionState::new("ten_test", &registry);
    for (object, ordinal) in [("ent_a", 1), ("ent_b", 2), ("ent_c", 3)] {
        assert_eq!(
            apply_sealed(&mut state, &create_reading(object, ordinal)),
            FoldOutcome::Applied
        );
    }
    let first = sealed(
        "ent_a",
        4,
        vec![OntologyEdit::create_link("lty_measures", "ent_b").unwrap()],
    );
    assert_eq!(apply_sealed(&mut state, &first), FoldOutcome::Applied);
    // OneToOne: a second outbound from ent_a violates cardinality.
    let second = sealed(
        "ent_a",
        5,
        vec![OntologyEdit::create_link("lty_measures", "ent_c").unwrap()],
    );
    assert_eq!(
        apply_sealed(&mut state, &second),
        FoldOutcome::Poisoned(PoisonReason::Link(
            OntologyEngineError::CardinalityViolation {
                cardinality: LinkCardinality::OneToOne
            }
        ))
    );
}

#[test]
fn ordinals_must_stay_dense() {
    let registry = registry();
    let mut state = ProjectionState::new("ten_test", &registry);
    assert_eq!(
        apply_sealed(&mut state, &create_reading("ent_r1", 2)),
        FoldOutcome::Poisoned(PoisonReason::NonDenseOrdinal {
            expected: 1,
            found: 2
        })
    );
}

#[test]
fn foreign_tenant_poisons() {
    let registry = registry();
    let mut state = ProjectionState::new("ten_other", &registry);
    assert_eq!(
        apply_sealed(&mut state, &create_reading("ent_r1", 1)),
        FoldOutcome::Poisoned(PoisonReason::TenantMismatch)
    );
}
