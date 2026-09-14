mod facade_support;
mod failing_log;
use facade_support as support;

use std::sync::Arc;
use std::time::Duration;

use axum::http::StatusCode;
use failing_log::{AlwaysFailingLog, state_with_a_failing_log};
use foundry_ontology_app::metrics::INVOCATION_LATENCY_BUDGET;
use support::{Fixture, Session, TENANT, WRITE_BODY as WRITE, scrape, value_of};

const ANSWERED: &str = "foundry_action_invocation_answered_total";
const WITHIN_BUDGET: &str = "foundry_action_invocation_answered_within_250ms_total";
const REFUSED: &str = "foundry_action_submit_refused_total";
const ISSUED: &str = "foundry_denial_issued_total";
const RECORDED: &str = "foundry_denial_recorded_total";

/// The seeded `ety_record` requires `name`. A write without it is well
/// formed, credentialed, in roster and policy-allowed, so every gate before
/// the writer passes it and the writer's own admission refuses it.
const NAMELESS_WRITE: &str = r#"{"object_ref":"ent_nameless","action_type":"aty_record_write","idempotency_key":"idem_nameless","occurred_at_epoch_seconds":1700000000,"properties":{"note":"no name"}}"#;

#[tokio::test]
async fn an_accepted_invocation_answered_within_budget_counts_on_both_series() {
    let fixture = Fixture::new("latency-fast");
    let session = fixture.session();
    let (status, _) = session.post(Some(fixture.operator_token()), WRITE).await;
    assert_eq!(status, StatusCode::OK);
    let body = scrape(&session).await;
    assert_eq!(value_of(&body, ANSWERED), 1);
    assert_eq!(value_of(&body, WITHIN_BUDGET), 1);
    assert_eq!(value_of(&body, ISSUED), 0);
}

/// The tenant lock is held by the test across the budget, so the invocation
/// is accepted late through the same path a contended tenant would delay it.
#[tokio::test]
async fn an_accepted_invocation_held_past_the_budget_is_answered_but_not_within_it() {
    let fixture = Fixture::new("latency-slow");
    let state = Arc::new(fixture.state());
    let token = fixture.operator_token();
    let held = state.tenants.get(TENANT).expect("served").lock().await;
    let posting = tokio::spawn({
        let state = Arc::clone(&state);
        async move { Session::from_shared(state).post(Some(token), WRITE).await }
    });
    tokio::time::sleep(INVOCATION_LATENCY_BUDGET + Duration::from_millis(100)).await;
    drop(held);
    let (status, _) = posting.await.expect("the post completes");
    assert_eq!(status, StatusCode::OK);
    let body = scrape(&Session::from_shared(state)).await;
    assert_eq!(value_of(&body, ANSWERED), 1);
    assert_eq!(value_of(&body, WITHIN_BUDGET), 0, "held past the budget");
}

#[tokio::test]
async fn the_budget_in_the_series_name_is_the_budget_measured() {
    let fixture = Fixture::new("latency-name");
    let body = scrape(&fixture.session()).await;
    let named = format!("within_{}ms", INVOCATION_LATENCY_BUDGET.as_millis());
    assert!(
        WITHIN_BUDGET.contains(&named),
        "{WITHIN_BUDGET} must carry {named}"
    );
    assert_eq!(value_of(&body, WITHIN_BUDGET), 0, "the series is exported");
}

/// No credential, a malformed body, an action type that is not an action id,
/// an unserved tenant, a policy denial, and a body with no representable edit
/// (refused after the policy allowed it and after the tenant lock, still
/// before the writer): each is refused and none is a denial, because none
/// reached the writer.
#[tokio::test]
async fn a_refusal_before_the_writer_is_not_a_denial() {
    let fixture = Fixture::new("denial-pre-writer");
    let session = fixture.session();
    let unrepresentable = WRITE.replace(r#""name":"Ada""#, r#"" name":"Ada""#);
    let not_an_action_id = WRITE.replace("aty_record_write", "nope");
    for (label, token, body, expect) in [
        ("no credential", None, WRITE, StatusCode::UNAUTHORIZED),
        (
            "malformed body",
            Some(fixture.operator_token()),
            "{not json",
            StatusCode::BAD_REQUEST,
        ),
        (
            "action type is not an action id",
            Some(fixture.operator_token()),
            not_an_action_id.as_str(),
            StatusCode::BAD_REQUEST,
        ),
        (
            "unserved tenant",
            Some(fixture.foreign_token()),
            WRITE,
            StatusCode::FORBIDDEN,
        ),
        (
            "policy denial",
            Some(fixture.roleless_token()),
            WRITE,
            StatusCode::FORBIDDEN,
        ),
        (
            "no representable edit",
            Some(fixture.operator_token()),
            unrepresentable.as_str(),
            StatusCode::BAD_REQUEST,
        ),
    ] {
        let (status, _) = session.post(token, body).await;
        assert_eq!(status, expect, "{label}");
    }
    let body = scrape(&session).await;
    assert_eq!(value_of(&body, REFUSED), 6);
    assert_eq!(value_of(&body, ISSUED), 0);
    assert_eq!(value_of(&body, RECORDED), 0);
    assert_eq!(value_of(&body, ANSWERED), 0, "a refusal is not timed");
    assert_eq!(fixture.denial_head(), 0);
}

/// A log that cannot be written is neither a denial nor an answer: the
/// writer passed every gate and the append faulted, so no gate refused and
/// nothing was accepted.
#[tokio::test]
async fn a_storage_fault_is_a_refusal_that_is_neither_a_denial_nor_an_answer() {
    let fixture = Fixture::new("denial-storage-fault");
    let session = Session::from_state(state_with_a_failing_log(
        &fixture.config(),
        "the action log is unwritable",
    ));
    let (status, _) = session.post(Some(fixture.operator_token()), WRITE).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    let body = scrape(&session).await;
    assert_eq!(value_of(&body, REFUSED), 1);
    assert_eq!(value_of(&body, ISSUED), 0);
    assert_eq!(value_of(&body, ANSWERED), 0);
}

/// For this body the admission gate is the one that refuses: the seeded
/// action declares no parameters and the kernel's authorization checks pass.
#[tokio::test]
async fn a_writer_refusal_is_a_denial_the_trail_holds() {
    let fixture = Fixture::new("denial-recorded");
    let session = fixture.session();
    let (status, body) = session
        .post(Some(fixture.operator_token()), NAMELESS_WRITE)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert!(body.contains(r#""gate":"admission""#), "{body}");
    let scraped = scrape(&session).await;
    assert_eq!(value_of(&scraped, REFUSED), 1);
    assert_eq!(value_of(&scraped, ISSUED), 1);
    assert_eq!(value_of(&scraped, RECORDED), 1);
    assert_eq!(value_of(&scraped, ANSWERED), 0, "a refusal is not timed");
    assert_eq!(fixture.denial_head(), 1, "the trail holds the denial");
    assert_eq!(fixture.log_head(), 0, "no object ordinal spent");
}

/// An action type the registry does not know passes the enforcement point,
/// which never sees it, and is refused by the writer's authorization gate.
#[tokio::test]
async fn an_unknown_action_type_is_a_denial_on_the_authorization_gate() {
    let fixture = Fixture::new("denial-authorization-gate");
    let session = fixture.session();
    let unknown = WRITE.replace("aty_record_write", "aty_nonesuch");
    let (status, body) = session.post(Some(fixture.operator_token()), &unknown).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert!(body.contains(r#""gate":"authorization""#), "{body}");
    let scraped = scrape(&session).await;
    assert_eq!(value_of(&scraped, ISSUED), 1);
    assert_eq!(value_of(&scraped, RECORDED), 1);
    assert_eq!(fixture.denial_head(), 1);
}

/// A divergent reuse of a spent key is the log's refusal after the writer's
/// gates, so it is neither a denial nor an answer.
#[tokio::test]
async fn an_idempotency_conflict_is_neither_a_denial_nor_an_answer() {
    let fixture = Fixture::new("denial-key-conflict");
    let session = fixture.session();
    let (first, _) = session.post(Some(fixture.operator_token()), WRITE).await;
    assert_eq!(first, StatusCode::OK);
    let divergent = WRITE.replace(r#""name":"Ada""#, r#""name":"Bob""#);
    let (second, body) = session
        .post(Some(fixture.operator_token()), &divergent)
        .await;
    assert_eq!(second, StatusCode::CONFLICT, "{body}");
    let scraped = scrape(&session).await;
    assert_eq!(value_of(&scraped, REFUSED), 1);
    assert_eq!(value_of(&scraped, ISSUED), 0);
    assert_eq!(value_of(&scraped, ANSWERED), 1);
    assert_eq!(fixture.log_head(), 1);
}

/// PRESENT BEHAVIOUR, pinned for the lane that changes it: within one
/// process a byte-identical retry is a conflict, not a deduplication, because
/// the decision id is minted per request and encoded into the record.
#[tokio::test]
async fn a_byte_identical_retry_within_one_process_conflicts_today() {
    let fixture = Fixture::new("denial-retry-today");
    let session = fixture.session();
    let (first, _) = session.post(Some(fixture.operator_token()), WRITE).await;
    assert_eq!(first, StatusCode::OK);
    let (second, body) = session.post(Some(fixture.operator_token()), WRITE).await;
    assert_eq!(second, StatusCode::CONFLICT, "{body}");
    assert_eq!(fixture.log_head(), 1);
}

/// The denial's key is its own content, and that content carries the decision
/// id the policy decision point mints per request. A retried refusal is a new
/// decision, so it is a new denial rather than a deduplicated one.
#[tokio::test]
async fn a_retried_refusal_is_a_new_denial_because_each_decision_is_its_own() {
    let fixture = Fixture::new("denial-retry");
    let session = fixture.session();
    for _ in 0..2 {
        let (status, _) = session
            .post(Some(fixture.operator_token()), NAMELESS_WRITE)
            .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }
    let scraped = scrape(&session).await;
    assert_eq!(value_of(&scraped, ISSUED), 2);
    assert_eq!(value_of(&scraped, RECORDED), 2);
    assert_eq!(fixture.denial_head(), 2, "two decisions, two denials");
}

/// The refusal survives a trail that cannot take its record, and the gap
/// between issued and recorded is the objective's whole signal.
#[tokio::test]
async fn a_denial_the_trail_could_not_take_is_issued_and_not_recorded() {
    let fixture = Fixture::new("denial-lost");
    let mut state = fixture.state();
    state
        .tenants
        .get_mut(TENANT)
        .expect("served")
        .get_mut()
        .denial_log = Box::new(AlwaysFailingLog {
        detail: "the trail is unwritable",
    });
    let session = Session::from_state(state);
    let (status, body) = session
        .post(Some(fixture.operator_token()), NAMELESS_WRITE)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert!(body.contains(r#""gate":"admission""#), "{body}");
    let scraped = scrape(&session).await;
    assert_eq!(value_of(&scraped, REFUSED), 1);
    assert_eq!(value_of(&scraped, ISSUED), 1);
    assert_eq!(value_of(&scraped, RECORDED), 0, "the trail did not take it");
    assert_eq!(fixture.denial_head(), 0);
}
