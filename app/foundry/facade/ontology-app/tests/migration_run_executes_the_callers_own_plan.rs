//! `POST /v1/migrations/run` — execute a plan to its fixpoint.
//!
//! The executing half. Unlike attest this WRITES, so it is gated on `Invoke`
//! and every refusal must leave the log exactly as it found it: a migration
//! that half-ran and then refused is worse than one that never started,
//! because the operator's next decision is made against a population no plan
//! describes.
//!
//! THE AUTHORITY IS THE CALLER'S OWN DECISION. `MigrationAuthority` carries a
//! `decision_id` and the surfaces the decision allows, and the runner stamps
//! them onto every upcast it writes. Minting one from anything other than the
//! PDP's answer for THIS caller on THIS surface would put a fabricated
//! authorization into the durable record — an audit trail that says a
//! decision was made when none was.

mod facade_support;
mod failing_log;
mod migration_support;

use axum::http::StatusCode;
use facade_support::{Fixture, Session, scrape, value_of};
use failing_log::AlwaysFailingLog;
use migration_support::{
    action_head, attest, plan_for, run, state_with_two_revisions, upcast_row, write_owing,
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

    let (status, body) = run(
        &session,
        Some(fixture.operator_token()),
        &plan_for("ten_acme"),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        body.contains(r#""upcast":1"#) && body.contains(r#""pending":0"#),
        "one object was owed and one was upcast: {body}"
    );
    assert!(
        body.contains(r#""fixpoint":true"#) && body.contains(r#""total":1"#),
        "and the run reports reaching the fixpoint over the population: {body}"
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

/// The upcast carries the CALLER'S OWN decision and the CALLER'S principal.
///
/// Both halves need a discriminating shape, and the obvious one fails. It is
/// not enough that the upcast's decision differs from the write's: a
/// hardcoded constant differs from it too, and that mutant survived exactly
/// this assertion. The id cannot be predicted either, because the PDP mints
/// it. So the claim is that TWO runs carry TWO decisions — a constant, or a
/// reused one, collapses them into one — while the principal, which IS known,
/// is asserted by value.
#[tokio::test]
async fn each_run_is_attributed_to_its_own_decision_and_the_callers_principal() {
    let fixture = Fixture::new("run-attribution");
    let config = fixture.config();
    let session = Session::from_state(state_with_two_revisions(&config));
    let token = Some(fixture.operator_token());

    write_owing(&session, token, "ent_alpha", "idem_1").await;
    let (first, body) = run(&session, token, &plan_for("ten_acme")).await;
    assert_eq!(first, StatusCode::OK, "{body}");
    write_owing(&session, token, "ent_beta", "idem_2").await;
    let (second, body) = run(&session, token, &plan_for("ten_acme")).await;
    assert_eq!(second, StatusCode::OK, "{body}");

    let alpha = upcast_row(&session, token, "ent_alpha").await;
    let beta = upcast_row(&session, token, "ent_beta").await;
    assert_ne!(
        alpha.0, beta.0,
        "two runs are two authorizations; one id for both is a fabricated or \
         reused decision, not the caller's own"
    );
    assert!(!alpha.0.is_empty() && !beta.0.is_empty(), "and real ones");
    assert_eq!(
        (alpha.1.as_str(), beta.1.as_str()),
        ("prn_alice", "prn_alice"),
        "and both are attributed to the principal that asked"
    );
}

/// Running a plan already at its fixpoint writes nothing.
///
/// The drift-sensitive idempotency key is what makes this true; without it a
/// second run re-submits every object and the log grows without the
/// population changing.
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

/// A run that CANNOT finish says so, in every field that carries the news.
///
/// This is the state the reported fields exist for, and the only one that
/// tells an honest report from a flattering constant: at a clean fixpoint
/// `pending`, `refused`, `conflicted` and `poisoned` are all zero and
/// `fixpoint` is true, so hardcoding any of them survives every other test in
/// this file. The store is broken AFTER the object lands, so an object is
/// genuinely owed and genuinely cannot be written.
///
/// It is a 200, not a refusal: the plan was executable and the run did what it
/// could. The failure is reported in the body, because a migration that
/// stopped short is a fact about the population, not a bad request.
#[tokio::test]
async fn a_run_that_cannot_write_reports_the_work_it_could_not_do() {
    let fixture = Fixture::new("run-store-outage");
    let config = fixture.config();
    let state = std::sync::Arc::new(state_with_two_revisions(&config));
    let session = Session::from_shared(std::sync::Arc::clone(&state));
    let token = Some(fixture.operator_token());
    write_owing(&session, token, "ent_alpha", "idem_1").await;
    for tenant in state.tenants.values() {
        tenant.lock().await.action_log = Box::new(AlwaysFailingLog {
            detail: "the store went away mid-migration",
        });
    }

    let (status, body) = run(&session, token, &plan_for("ten_acme")).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        body.contains(r#""fixpoint":false"#),
        "a run that wrote nothing has not reached a fixpoint: {body}"
    );
    assert!(
        body.contains(r#""pending":1"#) && body.contains(r#""conflicted":1"#),
        "the object is still owed, and the store fault is named as the reason: {body}"
    );
    assert!(
        body.contains(r#""upcast":0"#),
        "and nothing was upcast: {body}"
    );
}
