//! The counters report what the process actually did.
//!
//! Asserting that `/metrics` *mentions* a counter proves nothing: the
//! exposition emits a `# HELP` and `# TYPE` line for every declared metric
//! whether or not anything ever increments it. That is how a metric declared
//! and never sampled survived a suite that claimed to check it. These tests
//! drive the real router and assert exact VALUE lines, so a counter that
//! never counts fails here.
//!
//! Operator procedure: these values are process-lifetime and unlabelled by
//! tenant. `/metrics` is unauthenticated by design, so it must not become a
//! tenancy oracle — that is why an objective over them is a scrape-level
//! statement and not a per-tenant one.

#[path = "facade_support/mod.rs"]
mod support;

use axum::http::StatusCode;
use support::{Fixture, Session};

const WRITE: &str = r#"{"object_ref":"ent_alpha","action_type":"aty_record_write","idempotency_key":"idem_1","occurred_at_epoch_seconds":1700000000,"properties":{"name":"Ada"}}"#;

async fn scrape(session: &Session) -> String {
    let (status, body) = session.get(None, "/metrics").await;
    assert_eq!(status, StatusCode::OK);
    body
}

fn value_of(body: &str, metric: &str) -> u64 {
    body.lines()
        .find_map(|line| line.strip_prefix(&format!("{metric} ")))
        .unwrap_or_else(|| panic!("{metric} has no value line in:\n{body}"))
        .trim()
        .parse()
        .expect("a metric value is a number")
}

#[tokio::test]
async fn an_accepted_submission_increments_served_and_not_refused() {
    let fixture = Fixture::new("metrics-submit-served");
    let session = fixture.session();
    let (status, _) = session.post(Some(fixture.operator_token()), WRITE).await;
    assert_eq!(status, StatusCode::OK);
    let body = scrape(&session).await;
    assert_eq!(value_of(&body, "foundry_action_submit_served_total"), 1);
    assert_eq!(value_of(&body, "foundry_action_submit_refused_total"), 0);
}

#[tokio::test]
async fn a_refusal_before_the_writer_still_counts_against_availability() {
    // The denominator must include authorization failures. A submission
    // refused for want of a credential never reaches the writer, and an
    // availability number that omitted it would be flatter than the service.
    let fixture = Fixture::new("metrics-submit-anon");
    let session = fixture.session();
    let (status, _) = session.post(None, WRITE).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let body = scrape(&session).await;
    assert_eq!(value_of(&body, "foundry_action_submit_refused_total"), 1);
    assert_eq!(value_of(&body, "foundry_action_submit_served_total"), 0);
}

#[tokio::test]
async fn a_policy_denial_counts_as_a_refused_submission() {
    let fixture = Fixture::new("metrics-submit-roleless");
    let session = fixture.session();
    let (status, _) = session.post(Some(fixture.roleless_token()), WRITE).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let body = scrape(&session).await;
    assert_eq!(value_of(&body, "foundry_action_submit_refused_total"), 1);
}

#[tokio::test]
async fn an_answered_read_increments_served() {
    let fixture = Fixture::new("metrics-read-served");
    let session = fixture.session();
    let (write, _) = session.post(Some(fixture.operator_token()), WRITE).await;
    assert_eq!(write, StatusCode::OK);
    let (status, _) = session
        .get(
            Some(fixture.operator_token()),
            "/v1/objects/ent_alpha?revision=1",
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let body = scrape(&session).await;
    assert_eq!(value_of(&body, "foundry_read_served_total"), 1);
    assert_eq!(value_of(&body, "foundry_read_refused_total"), 0);
}

#[tokio::test]
async fn every_read_refusal_shape_counts() {
    // One case per refusing site, so deleting any single counting call
    // fails here rather than silently shrinking the denominator.
    let fixture = Fixture::new("metrics-read-refused");
    let session = fixture.session();
    // no credential
    let (anon, _) = session.get(None, "/v1/objects/ent_alpha?revision=1").await;
    assert_eq!(anon, StatusCode::UNAUTHORIZED);
    // unrecognised credential
    let (bad, _) = session
        .get(Some("nope"), "/v1/objects/ent_alpha?revision=1")
        .await;
    assert_eq!(bad, StatusCode::UNAUTHORIZED);
    // recognised, but policy refuses
    let (roleless, _) = session
        .get(
            Some(fixture.roleless_token()),
            "/v1/objects/ent_alpha?revision=1",
        )
        .await;
    assert_eq!(roleless, StatusCode::FORBIDDEN);
    // a credential naming a tenant this process does not serve
    let (foreign, _) = session
        .get(
            Some(fixture.foreign_token()),
            "/v1/objects/ent_alpha?revision=1",
        )
        .await;
    assert_eq!(foreign, StatusCode::FORBIDDEN);
    // an unusable revision pin
    let (unusable, _) = session
        .get(
            Some(fixture.operator_token()),
            "/v1/objects/ent_alpha?revision=abc",
        )
        .await;
    assert_eq!(unusable, StatusCode::BAD_REQUEST);

    let body = scrape(&session).await;
    assert_eq!(
        value_of(&body, "foundry_read_refused_total"),
        5,
        "every refusing site must count exactly once:\n{body}"
    );
    assert_eq!(value_of(&body, "foundry_read_served_total"), 0);
}

#[tokio::test]
async fn the_gauges_report_the_state_they_name() {
    let fixture = Fixture::new("metrics-gauges");
    let session = fixture.session();
    let body = scrape(&session).await;
    assert_eq!(value_of(&body, "foundry_served_tenants"), 1);
    assert_eq!(value_of(&body, "foundry_projection_lag"), 0);
    assert_eq!(value_of(&body, "foundry_poisoned_entries"), 0);
}
