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
mod out_of_band;

use axum::http::StatusCode;
use facade_support::{Fixture, Session, scrape, value_of};
use failing_log::AlwaysFailingLog;
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
    assert!(
        body.contains(r#""upcast":1"#) && body.contains(r#""pending":0"#),
        "one object was owed and one was upcast: {body}"
    );
    assert!(
        body.contains(r#""fixpoint":true"#) && body.contains(r#""total":2"#),
        "and the fixpoint is over the whole population, not just the owed: {body}"
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

/// The upcast carries the decision the PDP MINTED FOR THIS RUN.
///
/// Two earlier shapes of this test both failed, and the way they failed is
/// the point. "Differs from the write's decision" is satisfied by a hardcoded
/// constant. "Two runs carry two decisions" is satisfied by any locally
/// minted counter — a `format!("dcn_forged_{n}")` in the handler survives it
/// while writing a fabricated authorization into every durable upcast. Both
/// assert VARIABILITY; neither asserts PROVENANCE, and only provenance is the
/// claim.
///
/// The process authorizes through a `SeededIdGenerator`, whose whole purpose
/// is determinism: decision N is `01hmz` followed by N. So the id is
/// predictable BY VALUE, and the claim becomes exact — the write is the
/// first decision this process made, the run is the second, and the upcast
/// must carry the second. A constant fails it, a private counter fails it,
/// and reusing the write's decision fails it by carrying the first.
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

/// The decision id this process's PDP mints on its `n`th authorization.
///
/// `SeededIdGenerator` renders `01HMZ` followed by the zero-padded counter;
/// the surface lowercases it. Binding to the generator rather than to a
/// literal keeps the assertion about provenance rather than about a string.
fn nth_decision(n: u64) -> String {
    format!("01hmz{n:021}")
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
        body.contains(r#""pending":1"#) && body.contains(r#""unavailable":1"#),
        "the object is still owed, and the STORE FAULT is named as the reason: {body}"
    );
    assert!(
        body.contains(r#""conflicted":0"#),
        "a store that could not accept the append is not a caller who reused \
         an idempotency key — reporting it as one is blame in the wrong place \
         and advice against the retry that would work: {body}"
    );
    assert!(
        body.contains(r#""upcast":0"#),
        "and nothing was upcast: {body}"
    );
}

/// THE RUN TERMINATES. Not "converges quickly" — terminates at all.
///
/// A poison stands in the log and advances the fold, but it never binds the
/// object, so the plan still owes it, the same drift-sensitive key is
/// re-derived, and the byte-identical append deduplicates onto the same
/// poisoned ordinal. Counting that as progress made the loop a fixed point of
/// its own body: every pass identical, no append, nothing converging. The
/// handler never returned, and it holds the tenant's lock for the whole run —
/// so one migration against a projection one entry behind its log wedged
/// every request for that tenant until the process was restarted.
///
/// The state is not exotic. `out_of_band` exists because this repository's own
/// writer commits before it folds, and says a panic between the two "leaves
/// this process permanently one behind for its lifetime".
///
/// Driven on its own thread with a deadline, because the failure this pins is
/// non-termination: an `#[tokio::test]` would hang the runtime rather than
/// fail, and a hang in CI reads as an infrastructure problem rather than as
/// this defect.
#[test]
fn a_run_against_a_lagging_projection_terminates_rather_than_spinning() {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("a runtime");
        runtime.block_on(async {
            let fixture = Fixture::new("run-lagging-projection");
            let config = fixture.config();
            let session = Session::from_state(state_with_two_revisions(&config));
            let token = Some(fixture.operator_token());
            write_owing(&session, token, "ent_alpha", "idem_1").await;
            // One entry the durable log holds and this projection never
            // applied, exactly as a crash between append and fold leaves it.
            out_of_band::append_for(&config.action_log, "ten_acme", "idem_out_of_band");
            let _ = tx.send(run(&session, token, &plan_for("ten_acme")).await);
        });
    });

    let (status, body) = rx
        .recv_timeout(std::time::Duration::from_secs(30))
        .expect("the run must return; a migration that never ends holds the tenant lock forever");

    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        body.contains(r#""fixpoint":false"#) && body.contains(r#""poisoned":1"#),
        "it owes what it could not apply, and says the entry was poisoned: {body}"
    );
}
