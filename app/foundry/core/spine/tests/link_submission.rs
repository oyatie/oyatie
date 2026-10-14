mod catchup_support;
#[path = "migration_support/log.rs"]
mod log;
#[path = "link_submission/support.rs"]
mod support;

use catchup_support::sealed_link;
use foundry_projection_draft::ProjectionStore;
use foundry_records_draft::RecordsLog;
use foundry_spine::{ApplyOutcome, Checkpoint, fold_from_scratch, project_through};
use support::{Fixture, create, linking, submission};

const ENDPOINT_REFUSAL: &str = "link endpoints do not match the declared relationship";

#[test]
fn new_links_require_both_existing_typed_endpoints() {
    for (from, to) in [
        (None, Some("ety_reading")),
        (Some("ety_reading"), None),
        (Some("ety_other"), Some("ety_reading")),
        (Some("ety_reading"), Some("ety_other")),
    ] {
        let mut fixture = Fixture::new();
        if let Some(kind) = from {
            fixture.seed("ent_source", kind);
        }
        if let Some(kind) = to {
            fixture.seed("ent_target", kind);
        }
        fixture.refused(
            submission(
                "ent_source",
                "invalid",
                vec![linking("lty_measures", "ent_target")],
            ),
            ENDPOINT_REFUSAL,
        );
    }
}

#[test]
fn another_tenants_object_does_not_supply_the_target() {
    let mut fixture = Fixture::new();
    fixture.seed("ent_source", "ety_reading");
    let mut foreign = fixture
        .state
        .objects
        .get("ten_test", "ent_source")
        .unwrap()
        .clone();
    foreign.tenant_id = "ten_other".into();
    foreign.id = "ent_foreign".into();
    fixture.state.objects.upsert_entity(foreign).unwrap();
    fixture.refused(
        submission(
            "ent_source",
            "foreign",
            vec![linking("lty_measures", "ent_foreign")],
        ),
        ENDPOINT_REFUSAL,
    );
}

#[test]
fn valid_direction_self_links_and_native_tuple_idempotency_survive() {
    let mut fixture = Fixture::new();
    fixture.seed("ent_source", "ety_reading");
    fixture.seed("ent_target", "ety_other");
    for (key, kind, target) in [
        ("directed", "lty_directed", "ent_target"),
        ("duplicate", "lty_directed", "ent_target"),
        ("self", "lty_measures", "ent_source"),
    ] {
        let result = fixture
            .send(submission("ent_source", key, vec![linking(kind, target)]))
            .unwrap();
        assert!(matches!(result.outcome, ApplyOutcome::Applied { .. }));
        result.mirror.unwrap();
    }
    assert_eq!(
        fixture
            .store
            .links_from("ten_test", "ent_source")
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        fixture
            .store
            .links_to("ten_test", "ent_target")
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn endpoint_judgment_uses_final_staged_state_in_either_edit_order() {
    for edits in [
        vec![
            create("ety_reading", "staged"),
            linking("lty_measures", "ent_source"),
        ],
        vec![
            linking("lty_measures", "ent_source"),
            create("ety_reading", "staged"),
        ],
    ] {
        let mut fixture = Fixture::new();
        fixture
            .send(submission("ent_source", "staged", edits))
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
    }
    let mut fixture = Fixture::new();
    fixture.seed("ent_source", "ety_other");
    fixture.seed("ent_target", "ety_reading");
    fixture.refused(
        submission(
            "ent_source",
            "final_mismatch",
            vec![
                linking("lty_from_other", "ent_target"),
                create("ety_reading", "changed"),
            ],
        ),
        ENDPOINT_REFUSAL,
    );
}

#[test]
fn unknown_types_and_cardinality_keep_the_existing_scratch_gate() {
    let mut fixture = Fixture::new();
    fixture.seed("ent_source", "ety_reading");
    fixture.seed("ent_target", "ety_reading");
    fixture.refused(
        submission(
            "ent_source",
            "unknown",
            vec![linking("lty_unknown", "ent_missing")],
        ),
        "edits fail the fold's own admission",
    );
    fixture
        .send(submission(
            "ent_source",
            "first",
            vec![linking("lty_single", "ent_target")],
        ))
        .unwrap()
        .mirror
        .unwrap();
    fixture.refused(
        submission(
            "ent_source",
            "second",
            vec![linking("lty_single", "ent_missing")],
        ),
        "edits fail the fold's own admission",
    );
}

#[test]
fn historic_malformed_links_replay_and_retry_without_new_admission() {
    let mut fixture = Fixture::new();
    fixture.seed("ent_source", "ety_reading");
    let checkpoint = Checkpoint::capture(&fixture.state);
    let historic = sealed_link(2, "ent_source", "ent_missing");
    fixture.log.seed(historic.clone());
    project_through(&mut fixture.state, &mut fixture.store, &[historic]).unwrap();
    let entries = fixture.log.replay("ten_test", 1).unwrap();
    let expected = fixture.state.clone();
    assert_eq!(
        fold_from_scratch("ten_test", &expected.registry_input, &entries),
        expected
    );
    assert_eq!(
        checkpoint.resume(&expected.registry_input, &entries),
        expected
    );
    assert!(expected.poison.is_empty());
    let retried = fixture
        .send(submission(
            "ent_source",
            "idem_2",
            vec![linking("lty_measures", "ent_missing")],
        ))
        .unwrap();
    let ApplyOutcome::Applied { receipt } = retried.outcome else {
        panic!("original outcome must survive");
    };
    assert_eq!(receipt.ordinal, 2);
    assert!(receipt.deduplicated);
    retried.mirror.unwrap();
    assert_eq!(fixture.state, expected);
    assert_eq!(fixture.log.head("ten_test").unwrap(), 2);
    fixture.refused(
        submission(
            "ent_source",
            "fresh",
            vec![linking("lty_measures", "ent_missing")],
        ),
        ENDPOINT_REFUSAL,
    );
}
