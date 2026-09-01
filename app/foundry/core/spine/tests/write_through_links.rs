//! Links are durable projection state: a store rebuilt from the log
//! must come back with its edges, not just its objects.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    ActionTypeDefinition, ActionTypeId, AutonomyTier, EntityTypeDefinition, EntityTypeId,
    EntityTypePropertyDefinition, LinkCardinality, LinkTypeDefinition, LinkTypeId, OntologyEngine,
    PropertyTier,
};
use foundry_edits::{
    ActionRecord, EditSet, OntologyEdit, WireDataClass, WireProperty, WireTier, WireValue,
    encode_action_record,
};
use foundry_projection_draft::{MemoryProjectionStore, ProjectionStore};
use foundry_records_draft::{ActionEnvelope, Receipt, SealedEnvelope};
use foundry_spine::{ProjectionState, project_through};

fn internal() -> PrivacyDataClass {
    PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap()
}

fn registry() -> OntologyEngine {
    let mut engine = OntologyEngine::default();
    let definition = EntityTypeDefinition::new(
        "ten_test",
        EntityTypeId::new("ety_reading").unwrap(),
        "Reading",
        vec![
            EntityTypePropertyDefinition::new("name", PropertyTier::Scalar, internal(), true)
                .unwrap(),
        ],
        1,
    )
    .unwrap()
    .with_primary_key_property("name");
    engine.register_entity_type(definition).unwrap();
    engine
        .register_link_type(
            LinkTypeDefinition::new(
                "ten_test",
                LinkTypeId::new("lty_measures").unwrap(),
                EntityTypeId::new("ety_reading").unwrap(),
                EntityTypeId::new("ety_reading").unwrap(),
                LinkCardinality::ManyToMany,
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

fn sealed(object_ref: &str, ordinal: u64, name: &str) -> SealedEnvelope {
    let key = format!("idem_{ordinal}");
    let record = ActionRecord::new(
        "prn_alice",
        "dec_1",
        "reading.calibrated",
        &key,
        1_700_000_000_000,
        vec![],
        EditSet::new(vec![
            OntologyEdit::create_object("ety_reading", vec![name_property(name)]).unwrap(),
        ])
        .unwrap(),
    )
    .unwrap();
    SealedEnvelope {
        envelope: ActionEnvelope::new(
            "ten_test",
            object_ref,
            "aty_calibrate",
            &key,
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

/// A link edit reaches the durable store. Without this the projection
/// rebuilds with its objects and NO edges — the traversal surface would
/// see an empty graph and report it as truth.
fn linking(object_ref: &str, ordinal: u64, to: &str) -> SealedEnvelope {
    let key = format!("idem_{ordinal}");
    let record = ActionRecord::new(
        "prn_alice",
        "dec_1",
        "reading.calibrated",
        &key,
        1_700_000_000_000,
        vec![],
        EditSet::new(vec![OntologyEdit::create_link("lty_measures", to).unwrap()]).unwrap(),
    )
    .unwrap();
    SealedEnvelope {
        envelope: ActionEnvelope::new(
            "ten_test",
            object_ref,
            "aty_calibrate",
            &key,
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

#[test]
fn a_registered_link_reaches_the_store_in_both_directions() {
    let registry = registry();
    let entries = vec![
        sealed("ent_r1", 1, "Ada"),
        sealed("ent_r2", 2, "Grace"),
        linking("ent_r1", 3, "ent_r2"),
    ];
    let mut state = ProjectionState::new("ten_test", &registry);
    let mut store = MemoryProjectionStore::default();
    project_through(&mut state, &mut store, &entries).expect("a healthy store");

    let outbound = store.links_from("ten_test", "ent_r1").unwrap();
    assert_eq!(outbound.len(), 1, "the edge is durable: {outbound:?}");
    assert_eq!(outbound[0].link_type, "lty_measures");
    assert_eq!(outbound[0].to_object_ref, "ent_r2");

    let inbound = store.links_to("ten_test", "ent_r2").unwrap();
    assert_eq!(inbound, outbound, "and readable from the target too");
}
