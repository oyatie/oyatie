//! Determinism pins: two independent folds are Eq-identical, incremental
//! application equals fold-from-scratch, the same entry poisons with the
//! same reason on every fold, and typed values cross the boundary into
//! kernel carriers intact.

use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    ActionTypeDefinition, ActionTypeId, AutonomyTier, EntityTypeDefinition, EntityTypeId,
    EntityTypePropertyDefinition, OntologyEngine, PropertyTier, PropertyValue, ScalarType,
    ValueTypeDeclaration,
};
use foundry_edits::{
    ActionRecord, EditSet, OntologyEdit, WireDataClass, WireDouble, WireProperty, WireTier,
    WireValue, encode_action_record,
};
use foundry_records_draft::{ActionEnvelope, Receipt, SealedEnvelope};
use foundry_spine::{ProjectionState, apply_sealed, fold_from_scratch};

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
                    EntityTypePropertyDefinition::typed(
                        "ratio",
                        ValueTypeDeclaration::Scalar(ScalarType::Double),
                        internal(),
                        false,
                    )
                    .unwrap(),
                ],
                1,
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

fn prop(name: &str, value: WireValue) -> WireProperty {
    WireProperty::new(name, WireTier::Scalar, WireDataClass::InternalOnly, value).unwrap()
}

fn sealed(object_ref: &str, ordinal: u64, edits: Vec<OntologyEdit>) -> SealedEnvelope {
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
            key,
            1,
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

/// A run with applied entries, a poisoned entry, and typed values.
fn log_entries() -> Vec<SealedEnvelope> {
    vec![
        sealed(
            "ent_r1",
            1,
            vec![
                OntologyEdit::create_object(
                    "ety_reading",
                    vec![
                        prop("name", WireValue::String("Ada".into())),
                        prop(
                            "ratio",
                            WireValue::Double(WireDouble::new(-1000.25).unwrap()),
                        ),
                    ],
                )
                .unwrap(),
            ],
        ),
        sealed(
            "ent_ghost",
            2,
            vec![
                OntologyEdit::upsert_properties(vec![prop("name", WireValue::String("x".into()))])
                    .unwrap(),
            ],
        ),
        sealed(
            "ent_r1",
            3,
            vec![
                OntologyEdit::upsert_properties(vec![prop(
                    "name",
                    WireValue::String("Named".into()),
                )])
                .unwrap(),
            ],
        ),
    ]
}

#[test]
fn two_independent_folds_are_identical() {
    let registry = registry();
    let entries = log_entries();
    let first = fold_from_scratch("ten_test", &registry, &entries);
    let second = fold_from_scratch("ten_test", &registry, &entries);
    assert_eq!(first, second);
}

#[test]
fn incremental_application_equals_fold_from_scratch() {
    let registry = registry();
    let entries = log_entries();
    let mut live = ProjectionState::new("ten_test", &registry);
    for entry in &entries {
        let _ = apply_sealed(&mut live, entry);
    }
    assert_eq!(live, fold_from_scratch("ten_test", &registry, &entries));
}

#[test]
fn the_same_entry_poisons_identically_on_every_fold() {
    let registry = registry();
    let entries = log_entries();
    let first = fold_from_scratch("ten_test", &registry, &entries);
    let second = fold_from_scratch("ten_test", &registry, &entries);
    assert_eq!(first.poison.len(), 1, "exactly the ghost upsert poisons");
    assert_eq!(first.poison, second.poison);
    assert_eq!(first.applied_ordinal, 3);
}

#[test]
fn typed_values_cross_the_boundary_intact() {
    let registry = registry();
    let state = fold_from_scratch("ten_test", &registry, &log_entries());
    let entity = state.objects.get("ten_test", "ent_r1").expect("projected");
    match &entity.properties["ratio"].value.value {
        PropertyValue::Double(double) => {
            assert_eq!(double.get(), -1000.25);
        }
        other => panic!("expected a typed double, got {other:?}"),
    }
    assert_eq!(
        entity.properties["name"].value.value,
        PropertyValue::String("Named".into())
    );
    assert_eq!(state.history["ent_r1"], vec![1, 3]);
}
