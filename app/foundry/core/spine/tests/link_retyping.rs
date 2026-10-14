mod catchup_support;
#[path = "migration_support/log.rs"]
mod log;
#[path = "link_submission/support.rs"]
mod support;

use data_ontology_kernel::{EntityTypeId, LinkTypeId};
use foundry_edits::OntologyEdit;
use foundry_projection_draft::ProjectionStore;
use foundry_records_draft::RecordsLog;
use foundry_spine::{ApplyOutcome, Checkpoint, fold_from_scratch};
use support::{Fixture, linking, name_property, retyping, submission};

const ENDPOINT_REFUSAL: &str = "link endpoints do not match the declared relationship";

fn linked() -> Fixture {
    let mut fixture = Fixture::new();
    fixture.seed("ent_source", "ety_reading");
    fixture.seed("ent_target", "ety_reading");
    fixture
        .send(submission(
            "ent_source",
            "linked",
            vec![linking("lty_measures", "ent_target")],
        ))
        .unwrap()
        .mirror
        .unwrap();
    fixture
}

#[test]
fn retyping_a_retained_source_cannot_break_its_declared_endpoint() {
    let mut fixture = linked();
    fixture.refused(
        retyping("ent_source", "wrong_source", "ety_other"),
        ENDPOINT_REFUSAL,
    );
}

#[test]
fn retyping_a_retained_target_cannot_break_its_declared_endpoint() {
    let mut fixture = linked();
    fixture.refused(
        retyping("ent_target", "wrong_target", "ety_other"),
        ENDPOINT_REFUSAL,
    );
}

#[test]
fn retyping_without_incident_links_preserves_unrelated_historical_edges() {
    let mut fixture = linked();
    fixture.historic_link("ent_source", "ent_absent");
    fixture.seed("ent_unrelated", "ety_reading");
    fixture
        .send(retyping("ent_unrelated", "allowed", "ety_other"))
        .unwrap()
        .mirror
        .unwrap();
    assert_eq!(
        fixture
            .state
            .objects
            .get("ten_test", "ent_unrelated")
            .unwrap()
            .entity_type
            .value,
        "ety_other"
    );
    assert_eq!(
        fixture
            .store
            .links_from("ten_test", "ent_source")
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn retyping_can_correct_a_legacy_endpoint_when_the_final_tuple_is_valid() {
    let mut fixture = Fixture::new();
    fixture.seed("ent_source", "ety_other");
    fixture.seed("ent_target", "ety_reading");
    fixture.historic_link("ent_source", "ent_target");
    fixture
        .send(retyping("ent_source", "corrected", "ety_reading"))
        .unwrap()
        .mirror
        .unwrap();
    assert_eq!(
        fixture
            .state
            .objects
            .get("ten_test", "ent_source")
            .unwrap()
            .entity_type
            .value,
        "ety_reading"
    );
    assert_eq!(
        fixture
            .store
            .links_from("ten_test", "ent_source")
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn property_updates_do_not_rejudge_a_historical_invalid_edge() {
    let mut fixture = Fixture::new();
    fixture.seed("ent_source", "ety_reading");
    fixture.historic_link("ent_source", "ent_absent");
    let edit = OntologyEdit::upsert_properties(vec![name_property("Updated")]).unwrap();
    fixture
        .send(submission("ent_source", "properties", vec![edit]))
        .unwrap()
        .mirror
        .unwrap();
    assert_eq!(
        fixture
            .store
            .links_from("ten_test", "ent_source")
            .unwrap()
            .len(),
        1
    );
    assert!(fixture.state.poison.is_empty());
}

#[test]
fn another_tenants_tuple_cannot_refuse_this_tenants_retype() {
    let mut fixture = Fixture::new();
    for engine in [&mut fixture.state.engine, &mut fixture.state.registry_input] {
        let mut entity = engine
            .entity_type("ten_test", &EntityTypeId::new("ety_reading").unwrap())
            .unwrap()
            .clone();
        entity.tenant_id = "ten_other".into();
        engine.register_entity_type(entity).unwrap();
        let mut relation = engine
            .link_type("ten_test", &LinkTypeId::new("lty_measures").unwrap())
            .unwrap()
            .clone();
        relation.tenant_id = "ten_other".into();
        engine.register_link_type(relation).unwrap();
        engine
            .register_link_instance(
                "ten_other",
                &LinkTypeId::new("lty_measures").unwrap(),
                "ent_source",
                "ent_foreign",
            )
            .unwrap();
    }
    fixture.seed("ent_source", "ety_reading");
    fixture
        .send(retyping("ent_source", "own_retype", "ety_other"))
        .unwrap()
        .mirror
        .unwrap();
    assert_eq!(
        fixture
            .state
            .objects
            .get("ten_test", "ent_source")
            .unwrap()
            .entity_type
            .value,
        "ety_other"
    );
    assert_eq!(fixture.state.engine.link_instances("ten_test").count(), 0);
    assert_eq!(fixture.state.engine.link_instances("ten_other").count(), 1);
}

#[test]
fn historic_retyping_still_replays_and_retries_under_its_original_receipt() {
    let mut fixture = linked();
    let checkpoint = Checkpoint::capture(&fixture.state);
    let retry = fixture.historic_retype("ent_source", "ety_other");
    let historic_ordinal = fixture.state.applied_ordinal;
    let entries = fixture.log.replay("ten_test", 1).unwrap();
    assert_eq!(
        fold_from_scratch("ten_test", &fixture.state.registry_input, &entries),
        fixture.state
    );
    assert_eq!(
        checkpoint.resume(&fixture.state.registry_input, &entries),
        fixture.state
    );
    fixture
        .send(retyping("ent_source", "repair", "ety_reading"))
        .unwrap()
        .mirror
        .unwrap();
    let before = fixture.state.clone();
    let replay = fixture.send(retry).unwrap();
    let ApplyOutcome::Applied { receipt } = replay.outcome else {
        panic!("original receipt must survive");
    };
    assert_eq!(receipt.ordinal, historic_ordinal);
    assert!(receipt.deduplicated);
    replay.mirror.unwrap();
    assert_eq!(fixture.state, before);
}
