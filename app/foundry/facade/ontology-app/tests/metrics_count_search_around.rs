//! Each refusing site of `POST /v1/search-around`, pinned INDIVIDUALLY by a
//! delta, as the other read routes are.
//!
//! The doctrine is the one the sibling census files state: an aggregate total
//! passes while one site counts twice and another counts never, so each case
//! names its own site and deleting any single counting call fails the case that
//! names it.

#[path = "facade_support/mod.rs"]
mod support;

use axum::http::StatusCode;
use support::{Fixture, Session, WRITE_BODY as WRITE, scrape, value_of};

const WALK: &str = "/v1/search-around";

fn body(key: &str, seed: &str, depth: u32) -> String {
    format!(
        r#"{{"idempotency_key":"{key}","seed":{seed},"edge_types":["lty_owns"],"max_depth":{depth},"direction":"outbound","consent":"unrestricted","freshness_floor_epoch_seconds":0,"observed_at_epoch_seconds":1700000000}}"#
    )
}

fn seed(entity_type: &str, revision: u32) -> String {
    format!(r#"{{"type":"{entity_type}","revision":{revision},"set":"every"}}"#)
}

async fn assert_refusal_delta(
    session: &Session,
    label: &str,
    body: &str,
    expect: StatusCode,
    token: Option<&str>,
) {
    let before = value_of(&scrape(session).await, "foundry_read_refused_total");
    let (status, reply) = session.post_to(token, WALK, body).await;
    assert_eq!(status, expect, "{label}: {reply}");
    let after = value_of(&scrape(session).await, "foundry_read_refused_total");
    assert_eq!(after, before + 1, "{label}: must count exactly one refusal");
}

#[tokio::test]
async fn each_search_around_refusal_site_counts_exactly_once() {
    let fixture = Fixture::new("metrics-walk-refused");
    let session = fixture.session();
    let (write, _) = session.post(Some(fixture.operator_token()), WRITE).await;
    assert_eq!(write, StatusCode::OK);
    let token = Some(fixture.operator_token());
    let served = body("idem-served", &seed("ety_record", 1), 2);

    assert_refusal_delta(
        &session,
        "no credential",
        &served,
        StatusCode::UNAUTHORIZED,
        None,
    )
    .await;
    assert_refusal_delta(
        &session,
        "policy denial",
        &served,
        StatusCode::FORBIDDEN,
        Some(fixture.roleless_token()),
    )
    .await;
    for (label, request, status) in [
        (
            "unusable body",
            r#"{"idempotency_key":"idem-x"}"#.to_owned(),
            StatusCode::BAD_REQUEST,
        ),
        (
            "undeclared seed type",
            body("idem-type", &seed("ety_absent", 1), 2),
            StatusCode::BAD_REQUEST,
        ),
        (
            "seed pin the tenant never accepted",
            body("idem-pin", &seed("ety_record", 9), 2),
            StatusCode::CONFLICT,
        ),
        (
            "past the domain's own maximum depth",
            body("idem-deep", &seed("ety_record", 1), 17),
            StatusCode::BAD_REQUEST,
        ),
        (
            "past the policy's depth ceiling",
            body("idem-policy", &seed("ety_record", 1), 9),
            StatusCode::FORBIDDEN,
        ),
        (
            "a seed that holds no object",
            body(
                "idem-seedless",
                r#"{"type":"ety_record","revision":1,"set":{"named":["ent_nowhere"]}}"#,
                2,
            ),
            StatusCode::BAD_REQUEST,
        ),
    ] {
        assert_refusal_delta(&session, label, &request, status, token).await;
    }

    // The conflict site needs the key to mean something else first, so the
    // first post is a SERVED read and only the second is the refusal.
    let (status, _) = session.post_to(token, WALK, &served).await;
    assert_eq!(status, StatusCode::OK);
    assert_refusal_delta(
        &session,
        "one key, another intent",
        &body("idem-served", &seed("ety_record", 1), 3),
        StatusCode::CONFLICT,
        token,
    )
    .await;
}

/// A served walk counts once, and a REPLAY of it counts again: a replay is a
/// read this surface answered, not a read it declined to repeat.
#[tokio::test]
async fn a_served_walk_and_its_replay_each_count_once() {
    let fixture = Fixture::new("metrics-walk-served");
    let session = fixture.session();
    let (write, _) = session.post(Some(fixture.operator_token()), WRITE).await;
    assert_eq!(write, StatusCode::OK);
    let token = Some(fixture.operator_token());
    let request = body("idem-metrics", &seed("ety_record", 1), 2);
    for attempt in ["first", "replay"] {
        let before = value_of(&scrape(&session).await, "foundry_read_served_total");
        let (status, reply) = session.post_to(token, WALK, &request).await;
        assert_eq!(status, StatusCode::OK, "{attempt}: {reply}");
        let after = value_of(&scrape(&session).await, "foundry_read_served_total");
        assert_eq!(after, before + 1, "{attempt}: a served read counts once");
    }
}
