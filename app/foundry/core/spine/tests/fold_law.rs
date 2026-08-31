//! The fold's law tests: decode corruption, receipt consistency,
//! revision binding against the registry snapshot, and both conformance
//! stations re-checked authoritatively at fold time.

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
fn corrupt_payload_poisons_as_decode() {
    let registry = registry();
    let mut state = ProjectionState::new("ten_test", &registry);
    let entry = sealed_with("ent_r1", 1, "idem_1", vec![0xFF, 0x00, 0x01]);
    assert!(matches!(
        apply_sealed(&mut state, &entry),
        FoldOutcome::Poisoned(PoisonReason::Decode(_))
    ));
}

#[test]
fn embedded_receipt_must_match_the_envelope() {
    let registry = registry();
    let mut state = ProjectionState::new("ten_test", &registry);
    let payload = encode_action_record(&record(
        "idem_other",
        vec![OntologyEdit::create_object("ety_reading", vec![name_property("Ada")]).unwrap()],
    ));
    let entry = sealed_with("ent_r1", 1, "idem_1", payload);
    assert_eq!(
        apply_sealed(&mut state, &entry),
        FoldOutcome::Poisoned(PoisonReason::ReceiptMismatch)
    );
}

#[test]
fn revision_ahead_poisons_then_unpoisons_on_refold_after_evolution() {
    let registry = registry();
    let mut state = ProjectionState::new("ten_test", &registry);
    let mut entry = create_reading("ent_r1", 1);
    entry.envelope.schema_revision = 2;
    assert_eq!(
        apply_sealed(&mut state, &entry),
        FoldOutcome::Poisoned(PoisonReason::UnknownRevision { revision: 2 })
    );

    let mut evolved = registry.clone();
    evolved
        .evolve_entity_type(
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
                    EntityTypePropertyDefinition::new(
                        "unit",
                        PropertyTier::Scalar,
                        internal(),
                        false,
                    )
                    .unwrap(),
                ],
                2,
            )
            .unwrap(),
        )
        .unwrap();
    let refolded = fold_from_scratch("ten_test", &evolved, [&entry]);
    assert!(
        refolded.poison.is_empty(),
        "revision 2 un-poisons after evolution"
    );
    assert!(refolded.objects.get("ten_test", "ent_r1").is_some());
}

#[test]
fn undeclared_property_fails_instance_conformance() {
    let registry = registry();
    let mut state = ProjectionState::new("ten_test", &registry);
    let rogue = WireProperty::new(
        "rogue",
        WireTier::Scalar,
        WireDataClass::InternalOnly,
        WireValue::String("x".into()),
    )
    .unwrap();
    let entry = sealed(
        "ent_r1",
        1,
        vec![
            OntologyEdit::create_object("ety_reading", vec![name_property("Ada"), rogue]).unwrap(),
        ],
    );
    assert_eq!(
        apply_sealed(&mut state, &entry),
        FoldOutcome::Poisoned(PoisonReason::Conformance(
            OntologyEngineError::UndeclaredProperty {
                name: "rogue".into()
            }
        ))
    );
}

#[test]
fn undeclared_parameter_fails_parameter_conformance() {
    let registry = registry();
    let mut state = ProjectionState::new("ten_test", &registry);
    let mut with_param = record(
        "idem_1",
        vec![OntologyEdit::create_object("ety_reading", vec![name_property("Ada")]).unwrap()],
    );
    with_param.parameters = vec![name_property("Ada")];
    let entry = sealed_with("ent_r1", 1, "idem_1", encode_action_record(&with_param));
    assert_eq!(
        apply_sealed(&mut state, &entry),
        FoldOutcome::Poisoned(PoisonReason::Parameters(
            OntologyEngineError::UndeclaredParameter {
                name: "name".into()
            }
        ))
    );
}
