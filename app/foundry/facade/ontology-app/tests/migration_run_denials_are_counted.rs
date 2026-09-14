//! A migration run's writer refusals are denials on the two denial series,
//! recorded when the trail took them and issued either way.
mod facade_support;
mod failing_log;
mod migration_support;
mod out_of_band;
use facade_support as support;

use axum::http::StatusCode;
use data_ontology_kernel::{ActionTypeDefinition, ActionTypeId, AutonomyTier, EntityTypeId};
use failing_log::AlwaysFailingLog;
use foundry_ontology_app::{AppState, OperatorCredential};
use migration_support::{plan_for, run, state_with_two_revisions, write_owing, write_settled};
use support::{Fixture, Session, TENANT, scrape, value_of};

const ISSUED: &str = "foundry_denial_issued_total";
const RECORDED: &str = "foundry_denial_recorded_total";

/// An action the registry knows under a surface no facade decision carries,
/// so every upcast a run submits under it is refused by the writer's
/// authorization gate.
fn with_a_foreign_surface_action(mut state: AppState) -> AppState {
    let projection = &mut state
        .tenants
        .get_mut(TENANT)
        .expect("served")
        .get_mut()
        .projection;
    for engine in [&mut projection.registry_input, &mut projection.engine] {
        engine
            .register_action_type(
                ActionTypeDefinition::new(
                    TENANT,
                    ActionTypeId::new("aty_record_relabel").expect("an action id"),
                    EntityTypeId::new("ety_record").expect("a type id"),
                    "wrong-console",
                    AutonomyTier::T1Assist,
                    "record.relabelled",
                )
                .expect("a definition"),
            )
            .expect("registers");
    }
    state
}

fn plan_under_the_foreign_action() -> String {
    migration_support::plan_for(TENANT).replace("aty_record_write", "aty_record_relabel")
}

#[tokio::test]
async fn a_runs_refusals_are_denials_the_trail_holds() {
    let fixture = Fixture::new("run-denials-recorded");
    let state = with_a_foreign_surface_action(state_with_two_revisions(&fixture.config()));
    let session = Session::from_state(state);
    write_owing(
        &session,
        Some(fixture.operator_token()),
        "ent_alpha",
        "idem_1",
    )
    .await;
    write_owing(
        &session,
        Some(fixture.operator_token()),
        "ent_beta",
        "idem_2",
    )
    .await;
    // A third object that owes nothing: in the population, never submitted.
    write_settled(
        &session,
        Some(fixture.operator_token()),
        "ent_gamma",
        "idem_3",
    )
    .await;
    let (status, body) = run(
        &session,
        Some(fixture.operator_token()),
        &plan_under_the_foreign_action(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        body.contains(r#""total":3,"upcast":0,"pending":2,"refused":2"#),
        "{body}"
    );
    let scraped = scrape(&session).await;
    assert_eq!(value_of(&scraped, ISSUED), 2);
    assert_eq!(value_of(&scraped, RECORDED), 2);
    assert_eq!(fixture.denial_head(), 2, "both denials are on the trail");
}

#[tokio::test]
async fn a_runs_refusals_the_trail_could_not_take_are_issued_and_not_recorded() {
    let fixture = Fixture::new("run-denials-lost");
    // One process: the trail is unwritable from boot, which an accepted write
    // never touches, so the owed object lands and only the run's refusals
    // find the trail closed.
    let mut state = with_a_foreign_surface_action(state_with_two_revisions(&fixture.config()));
    state
        .tenants
        .get_mut(TENANT)
        .expect("served")
        .get_mut()
        .denial_log = Box::new(AlwaysFailingLog {
        detail: "the trail is unwritable",
    });
    let session = Session::from_state(state);
    write_owing(
        &session,
        Some(fixture.operator_token()),
        "ent_alpha",
        "idem_1",
    )
    .await;
    let (status, body) = run(
        &session,
        Some(fixture.operator_token()),
        &plan_under_the_foreign_action(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.contains(r#""refused":1"#), "{body}");
    let scraped = scrape(&session).await;
    assert_eq!(value_of(&scraped, ISSUED), 1);
    assert_eq!(value_of(&scraped, RECORDED), 0, "the trail did not take it");
    assert_eq!(fixture.denial_head(), 0);
}

/// The other side of the boundary: a run whose upcasts APPLY issues no
/// denial, and a plan the runner refuses before submitting anything (its
/// entity type is not in the registry) issues none either.
#[tokio::test]
async fn a_run_that_applies_or_is_refused_before_the_writer_issues_no_denial() {
    let fixture = Fixture::new("run-denials-none");
    let session = Session::from_state(state_with_two_revisions(&fixture.config()));
    write_owing(
        &session,
        Some(fixture.operator_token()),
        "ent_alpha",
        "idem_1",
    )
    .await;
    let (status, body) = run(&session, Some(fixture.operator_token()), &plan_for(TENANT)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        body.contains(r#""upcast":1,"pending":0,"refused":0"#),
        "{body}"
    );
    let absent = plan_for(TENANT).replace("ety_record", "ety_absent");
    let (status, _) = run(&session, Some(fixture.operator_token()), &absent).await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "refused before any submission"
    );
    let scraped = scrape(&session).await;
    assert_eq!(value_of(&scraped, ISSUED), 0);
    assert_eq!(value_of(&scraped, RECORDED), 0);
    assert_eq!(fixture.denial_head(), 0);
}

/// A conflict is not a denial either, and the run CAN produce one: the
/// upcast key carries the plan, the object and the scanned ordinal but not
/// the principal, while the writer's retry identity compares it. A poison
/// spends the key without binding the object, so a second operator of the
/// same tenant re-derives that key with a divergent payload and the log
/// refuses the append.
#[tokio::test]
async fn a_conflict_a_second_operator_provokes_is_not_a_denial() {
    let fixture = Fixture::new("run-denials-conflict");
    let mut config = fixture.config();
    config.operators.push(OperatorCredential {
        token: "second-operator-token".into(),
        tenant_id: TENANT.into(),
        principal_id: "prn_bob".into(),
        roles: vec!["foundry-operator".into()],
    });
    let session = Session::from_state(state_with_two_revisions(&config));
    write_owing(
        &session,
        Some(fixture.operator_token()),
        "ent_alpha",
        "idem_1",
    )
    .await;
    // The log grows behind the fold, so the run's first upcast poisons and
    // spends the key without binding the object.
    out_of_band::append_for(&config.action_log, TENANT, "idem_out_of_band");
    let (status, body) = run(&session, Some(fixture.operator_token()), &plan_for(TENANT)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.contains(r#""poisoned":1"#), "{body}");

    let (status, body) = run(&session, Some("second-operator-token"), &plan_for(TENANT)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        body.contains(r#""conflicted":1"#),
        "a second principal conflicts: {body}"
    );
    let scraped = scrape(&session).await;
    assert_eq!(
        value_of(&scraped, ISSUED),
        0,
        "a conflict is no writer gate's refusal"
    );
    assert_eq!(value_of(&scraped, RECORDED), 0);
}
