//! Migration audit law: an upcast is fully attributed in per-object
//! history and in derived audit events under the plan's event type; the
//! attestation reports fixpoint by the ONE shared predicate and lists
//! per-object poisons; and the pinned view refines `UpcastPending` by the
//! same predicate the runner scans with.

#[allow(dead_code)]
#[path = "migration_support/mod.rs"]
mod support;

use foundry_edits::EditTag;
use foundry_records_draft::RecordsLog;
use foundry_spine::{
    UpcastState, apply_sealed, derive_action_events, migration_attestation, object_at_revision,
    object_history, run_to_fixpoint,
};
use support::{MemoryLog, authority, fixture, plan, sealed_create, wire_string};

#[test]
fn object_history_shows_the_upcast_attributed() {
    let (_, mut log, mut state) = fixture();
    let mut denials = MemoryLog::default();
    run_to_fixpoint(&plan(), &authority(), &mut log, &mut denials, &mut state).unwrap();
    let entries = log.replay("ten_test", 1).unwrap();
    let history = object_history(&state, &entries, "ent_a");
    let upcast = history.last().unwrap();
    assert_eq!(upcast.audit_event_type, "reading.upcast_to_2");
    assert_eq!(upcast.schema_revision, 2);
    assert_eq!(upcast.principal_id, "prn_migrator");
    assert_eq!(upcast.decision_id, "dec_migration_run");
    assert_eq!(upcast.edits, vec![EditTag::UpsertProperties]);
}

#[test]
fn derived_audit_events_carry_the_plan_event_type_attributed() {
    let (_, mut log, mut state) = fixture();
    let mut denials = MemoryLog::default();
    run_to_fixpoint(&plan(), &authority(), &mut log, &mut denials, &mut state).unwrap();
    let entries = log.replay("ten_test", 1).unwrap();
    let derived = derive_action_events(&state, &entries);
    assert!(derived.underivable.is_empty());
    let event = derived
        .events
        .iter()
        .find(|event| event.audit_event_type == "reading.upcast_to_2")
        .expect("the upcast derives an audit event");
    assert_eq!(event.tenant_id, "ten_test");
    assert_eq!(event.principal_id, "prn_migrator");
    assert_eq!(event.decision_id, "dec_migration_run");
    assert_eq!(event.object_ref, "ent_a");
}

#[test]
fn attestation_reports_fixpoint_by_the_one_shared_predicate() {
    let (_, mut log, mut state) = fixture();
    let before = migration_attestation(&state, &plan());
    assert!(!before.fixpoint);
    assert_eq!(before.pending, vec!["ent_a".to_string()]);
    assert!(before.poisoned.is_empty());
    let mut denials = MemoryLog::default();
    run_to_fixpoint(&plan(), &authority(), &mut log, &mut denials, &mut state).unwrap();
    let after = migration_attestation(&state, &plan());
    assert!(after.fixpoint);
    assert!(after.pending.is_empty());
}

#[test]
fn attestation_lists_poisoned_ordinals() {
    let (_, _, mut state) = fixture();
    // A revision-ahead entry poisons deterministically; the attestation
    // must surface it, never hide it behind the fixpoint claim.
    let ahead = sealed_create("ent_c", 3, 9, vec![wire_string("name", "Cy")]);
    apply_sealed(&mut state, &ahead);
    assert_eq!(state.poison.len(), 1);
    let attested = migration_attestation(&state, &plan());
    assert_eq!(attested.poisoned, vec![3]);
}

#[test]
fn pinned_view_refines_pending_by_the_plan_predicate() {
    let (_, _, state) = fixture();
    // ent_b is written at revision 1 but owes nothing: structurally behind
    // the pin, yet Current under the plan's own predicate.
    let structural = object_at_revision(&state, "ent_b", 2, None).unwrap();
    assert_eq!(structural.upcast_state, UpcastState::UpcastPending);
    let refined = object_at_revision(&state, "ent_b", 2, Some(&plan())).unwrap();
    assert_eq!(refined.upcast_state, UpcastState::Current);
    // ent_a owes values: pending under both views.
    let owed = object_at_revision(&state, "ent_a", 2, Some(&plan())).unwrap();
    assert_eq!(owed.upcast_state, UpcastState::UpcastPending);
}
