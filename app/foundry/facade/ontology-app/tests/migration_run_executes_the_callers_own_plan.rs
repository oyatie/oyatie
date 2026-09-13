mod facade_support;
mod migration_support;

use axum::http::StatusCode;
use facade_support::{Fixture, Session, scrape, value_of};
use migration_support::{
    action_head, attest, plan_for, run, state_with_two_revisions, upcast_row, write_owing,
    write_settled,
};

#[tokio::test]
async fn an_operator_runs_a_plan_to_its_fixpoint() {
    let fixture = Fixture::new("run-fixpoint");
    let config = fixture.config();
    let session = Session::from_state(state_with_two_revisions(&config));
    write_owing(
        &session,
        Some(fixture.operator_token()),
        "ent_alpha",
        "idem_1",
    )
    .await;
    // A SECOND object that owes nothing. `total` counts the population of the
    // plan's entity type and `upcast` counts the work; with one object the two
    // numbers coincide, so a `total` fixed at the number owed reads correct
    // forever. They must be able to disagree for either to mean anything.
    write_settled(
        &session,
        Some(fixture.operator_token()),
        "ent_beta",
        "idem_2",
    )
    .await;

    let (status, body) = run(
        &session,
        Some(fixture.operator_token()),
        &plan_for("ten_acme"),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body,
        r#"{"total":2,"upcast":1,"pending":0,"refused":0,"conflicted":0,"unavailable":0,"poisoned":0,"fixpoint":true}"#
    );
    // The attestation is the independent witness: the surface's own report
    // could say anything, but a plan at its fixpoint owes nothing.
    let (_, attested) = attest(
        &session,
        Some(fixture.operator_token()),
        &plan_for("ten_acme"),
    )
    .await;
    assert!(
        attested.contains(r#""fixpoint":true"#),
        "after the run the plan owes nothing: {attested}"
    );
}

#[tokio::test]
async fn the_upcast_carries_the_decision_the_pdp_minted_for_this_run() {
    let fixture = Fixture::new("run-attribution");
    let config = fixture.config();
    let session = Session::from_state(state_with_two_revisions(&config));
    let token = Some(fixture.operator_token());

    write_owing(&session, token, "ent_alpha", "idem_1").await;
    let (status, body) = run(&session, token, &plan_for("ten_acme")).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (decision, principal) = upcast_row(&session, token, "ent_alpha").await;
    assert_eq!(
        decision,
        nth_decision(2),
        "the run is the SECOND authorization this process made, and the upcast \
         must carry that decision rather than one it invented"
    );
    assert_eq!(
        principal, "prn_alice",
        "attributed to the principal that asked"
    );
}

const SEEDED_ID_PREFIX: &str = "01hmz";
const SEEDED_ID_COUNTER_WIDTH: usize = 21;

fn nth_decision(n: u64) -> String {
    format!(
        "{SEEDED_ID_PREFIX}{n:0width$}",
        width = SEEDED_ID_COUNTER_WIDTH
    )
}

/// Accepts one token that no roster in this suite carries.
struct OutOfRosterVerifier;

const OUT_OF_ROSTER_TOKEN: &str = "token-no-roster-holds";

impl foundry_ontology_app::CallerVerifier for OutOfRosterVerifier {
    fn verify(&self, presented_bearer: Option<&str>) -> Option<foundry_ontology_app::Caller> {
        (presented_bearer == Some(OUT_OF_ROSTER_TOKEN)).then(|| foundry_ontology_app::Caller {
            tenant_id: "ten_acme".into(),
            principal_id: "prn_alice".into(),
            roles: vec!["foundry-operator".into()],
        })
    }
}

#[tokio::test]
async fn the_run_route_verifies_callers_through_the_port_not_the_roster() {
    let fixture = Fixture::new("run-port-verifier");
    let mut state = state_with_two_revisions(&fixture.config());
    state.verifier = Box::new(OutOfRosterVerifier);
    let session = Session::from_state(state);
    write_owing(&session, Some(OUT_OF_ROSTER_TOKEN), "ent_alpha", "idem_1").await;

    let (status, body) = run(&session, Some(OUT_OF_ROSTER_TOKEN), &plan_for("ten_acme")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body,
        r#"{"total":1,"upcast":1,"pending":0,"refused":0,"conflicted":0,"unavailable":0,"poisoned":0,"fixpoint":true}"#
    );

    let (status, body) = run(
        &session,
        Some(fixture.operator_token()),
        &plan_for("ten_acme"),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "a roster token the verifier does not know must not run a plan: {body}"
    );
}

#[tokio::test]
async fn a_second_run_at_the_fixpoint_writes_nothing() {
    let fixture = Fixture::new("run-idempotent");
    let config = fixture.config();
    let session = Session::from_state(state_with_two_revisions(&config));
    write_owing(
        &session,
        Some(fixture.operator_token()),
        "ent_alpha",
        "idem_1",
    )
    .await;
    let token = Some(fixture.operator_token());
    let (first, body) = run(&session, token, &plan_for("ten_acme")).await;
    assert_eq!(first, StatusCode::OK, "{body}");
    let settled = action_head(&config);

    let (second, again) = run(&session, token, &plan_for("ten_acme")).await;

    assert_eq!(second, StatusCode::OK, "{again}");
    assert!(
        again.contains(r#""upcast":0"#),
        "a settled population owes nothing to upcast: {again}"
    );
    assert_eq!(
        action_head(&config),
        settled,
        "and the log did not grow: {again}"
    );
}
