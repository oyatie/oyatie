//! `mirror_folded` writes whatever the fold decided, so a poisoned entry
//! reaches the store as a poison row and binds no object.
//!
//! Through `submit_through`, `mirror_folded`'s poisoned arm is reached when
//! the log's next ordinal is not the fold's: the dry run probes at the
//! fold's next ordinal and passes, the log appends at its own, and the real
//! fold refuses that as non-dense. Whether the store can then take that
//! poison depends on whether the refused ordinal is the store's next one —
//! its head one below it — which is what the two `submit_through` tests
//! separate.

#[path = "migration_support/mod.rs"]
mod support;

use data_ontology_kernel::{
    ActionInvocationRequest, ActionPolicyDecision, ActionTypeId, AutonomyTier,
};
use foundry_edits::{EditSet, OntologyEdit};
use foundry_projection_draft::{MemoryProjectionStore, ProjectionStore};
use foundry_records_draft::{ActionEnvelope, Receipt, RecordsLog, SealedEnvelope};
use foundry_spine::{
    ActionSubmission, ApplyOutcome, FoldOutcome, ProjectionState, WriteThroughError, apply_sealed,
    mirror_folded, submit_through,
};
use support::{MemoryLog, registry, sealed_create, wire_string};

fn submission(object_ref: &str, key: &str) -> ActionSubmission {
    ActionSubmission {
        request: ActionInvocationRequest {
            tenant_id: "ten_test".into(),
            principal_id: "prn_alice".into(),
            action_id: ActionTypeId::new("aty_calibrate").unwrap(),
            entity_id: object_ref.into(),
            idempotency_key: key.into(),
            requested_at_epoch_seconds: 1_700_000_000,
        },
        decision: ActionPolicyDecision {
            decision_id: "dec_1".into(),
            tenant_id: "ten_test".into(),
            principal_id: "prn_alice".into(),
            allowed_surfaces: vec!["ops-console".into()],
            autonomy_tier: AutonomyTier::T1Assist,
        },
        parameters: vec![],
        edits: EditSet::new(vec![
            OntologyEdit::create_object("ety_reading", vec![wire_string("name", "Ada")]).unwrap(),
        ])
        .unwrap(),
    }
}

/// A log holding one entry the projection never folded, so the next append
/// lands at an ordinal the fold cannot accept.
fn a_projection_behind_its_log() -> (MemoryLog, ProjectionState) {
    let engine = registry();
    let mut log = MemoryLog::default();
    log.seed(sealed_create(
        "ent_unfolded",
        1,
        1,
        vec![wire_string("name", "Grace")],
    ));
    (log, ProjectionState::new("ten_test", &engine))
}

/// The arm itself: a fold that poisoned is mirrored as a poison row.
#[test]
fn mirror_folded_writes_a_poison_row_and_binds_no_object() {
    let engine = registry();
    let mut projection = ProjectionState::new("ten_test", &engine);
    let mut store = MemoryProjectionStore::default();
    // Dense at ordinal 1, so the store can take it, and undecodable, so the
    // fold refuses it: a poison the store is able to record.
    let sealed = SealedEnvelope {
        envelope: ActionEnvelope::new(
            "ten_test",
            "ent_alpha",
            "aty_calibrate",
            "idem_bad",
            1,
            b"not a record".to_vec(),
            1,
        )
        .unwrap(),
        receipt: Receipt {
            ordinal: 1,
            object_sequence: 1,
            deduplicated: false,
        },
    };
    let fold = apply_sealed(&mut projection, &sealed);
    assert!(matches!(fold, FoldOutcome::Poisoned(_)));

    mirror_folded(&projection, &mut store, &sealed, &fold).expect("the store takes the poison");

    assert_eq!(
        store.poisoned("ten_test").unwrap(),
        vec![(1, "payload_decode".to_owned())],
        "the ledger names the refused ordinal and its reason"
    );
    assert_eq!(store.get("ten_test", "ent_alpha").unwrap(), None);
    assert_eq!(store.applied_head("ten_test").unwrap(), 1);
}

/// Through the write path: the caller is told the entry poisoned, and the
/// store refuses its mirror because it is behind the log by the same entry
/// the fold never consumed. The write is in the log; the store stays behind
/// until a catch-up.
#[test]
fn a_write_that_poisons_is_reported_poisoned_and_its_mirror_is_refused() {
    let (mut log, mut projection) = a_projection_behind_its_log();
    let mut denials = MemoryLog::default();
    let mut store = MemoryProjectionStore::default();

    let mirrored = submit_through(
        submission("ent_alpha", "idem_fresh"),
        &mut log,
        &mut denials,
        &mut projection,
        &mut store,
    )
    .expect("the writer accepts it; the fold refuses it");

    let ApplyOutcome::Poisoned { receipt, .. } = mirrored.outcome else {
        panic!("the fold must refuse an entry at a non-dense ordinal");
    };
    assert_eq!(receipt.ordinal, 2, "appended above the unfolded entry");
    assert!(
        matches!(
            mirrored.mirror,
            Err(WriteThroughError::Store { ordinal: 2, .. })
        ),
        "the store is behind by the same entry: {:?}",
        mirrored.mirror
    );
    assert_eq!(
        store.applied_head("ten_test").unwrap(),
        0,
        "nothing mirrored"
    );
    assert_eq!(
        log.head("ten_test").unwrap(),
        2,
        "the log holds it either way"
    );
}

/// The applied arm, for contrast: the same submission against a projection
/// that is level with its log mirrors an object and no poison.
#[test]
fn an_applied_write_is_mirrored_as_an_object_and_no_poison() {
    let engine = registry();
    let mut log = MemoryLog::default();
    let mut denials = MemoryLog::default();
    let mut projection = ProjectionState::new("ten_test", &engine);
    let mut store = MemoryProjectionStore::default();

    let mirrored = submit_through(
        submission("ent_alpha", "idem_1"),
        &mut log,
        &mut denials,
        &mut projection,
        &mut store,
    )
    .expect("the writer accepts it");

    assert!(matches!(mirrored.outcome, ApplyOutcome::Applied { .. }));
    assert!(mirrored.mirror.is_ok());
    assert_eq!(store.poisoned("ten_test").unwrap(), vec![]);
    assert!(store.get("ten_test", "ent_alpha").unwrap().is_some());
}

/// The poison `submit_through` CAN mirror: the store is level with the log
/// while the projection is behind, so the refused entry is dense for the
/// store and it records a poison row rather than binding an object. The
/// test above has the store behind too, so it can only watch the mirror be
/// refused; this one watches what the mirror writes.
#[test]
fn a_poison_the_store_can_take_is_recorded_as_a_poison_and_binds_no_object() {
    let engine = registry();
    let seeded = sealed_create("ent_unfolded", 1, 1, vec![wire_string("name", "Grace")]);
    let mut log = MemoryLog::default();
    log.seed(seeded.clone());
    let mut level = ProjectionState::new("ten_test", &engine);
    let applied = apply_sealed(&mut level, &seeded);
    let mut store = MemoryProjectionStore::default();
    mirror_folded(&level, &mut store, &seeded, &applied).expect("the store takes the first entry");

    let mut projection = ProjectionState::new("ten_test", &engine);
    let mut denials = MemoryLog::default();
    let mirrored = submit_through(
        submission("ent_alpha", "idem_takeable"),
        &mut log,
        &mut denials,
        &mut projection,
        &mut store,
    )
    .expect("the writer accepts it; the fold refuses it");

    assert!(
        matches!(mirrored.outcome, ApplyOutcome::Poisoned { .. }),
        "the fold must refuse an entry at a non-dense ordinal",
    );
    assert!(
        mirrored.mirror.is_ok(),
        "the store is level with the log, so this poison is dense for it: {:?}",
        mirrored.mirror,
    );
    assert_eq!(
        store.poisoned("ten_test").unwrap(),
        vec![(2, "non_dense_ordinal".to_owned())],
        "the refused entry is a poison row",
    );
    assert_eq!(
        store.get("ten_test", "ent_alpha").unwrap(),
        None,
        "an entry the fold refused binds no object",
    );
}
