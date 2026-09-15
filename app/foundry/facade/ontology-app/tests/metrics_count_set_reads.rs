//! Each refusing site of `POST /v1/object-sets/page`, pinned INDIVIDUALLY by
//! a delta, as the query-string read routes are next door.
//!
//! These sites need a request body to reach, which the delta helper next door
//! cannot send. The doctrine is the same one: an aggregate total passes while
//! one site counts twice and another counts never, so each case names its own
//! site and deleting any single counting call fails the case that names it.

#[path = "facade_support/mod.rs"]
mod support;

use axum::http::StatusCode;
use support::{Fixture, Session, WRITE_BODY as WRITE, scrape, value_of};

const PAGE: &str = "/v1/object-sets/page";

async fn assert_refusal_delta(
    session: &Session,
    label: &str,
    body: &str,
    expect: StatusCode,
    token: Option<&str>,
) {
    let before = value_of(&scrape(session).await, "foundry_read_refused_total");
    let (status, reply) = session.post_to(token, PAGE, body).await;
    assert_eq!(status, expect, "{label}: {reply}");
    let after = value_of(&scrape(session).await, "foundry_read_refused_total");
    assert_eq!(after, before + 1, "{label}: must count exactly one refusal");
}

/// One ref past the member ceiling, named rather than stored: the ceiling is
/// judged before the store is read, so no fixture of that size is needed.
fn many_refs() -> String {
    (0..=1000)
        .map(|index| format!(r#""ent_{index:05}""#))
        .collect::<Vec<String>>()
        .join(",")
}

/// A union of `leaves` empty named sets, as a body.
fn leaf_chain(leaves: usize) -> String {
    let set = (1..leaves).fold(r#"{"named":[]}"#.to_owned(), |left, _| {
        format!(r#"{{"union":[{left},{{"named":[]}}]}}"#)
    });
    format!(r#"{{"type":"ety_record","revision":1,"set":{set}}}"#)
}

#[tokio::test]
async fn each_set_page_refusal_site_counts_exactly_once() {
    let fixture = Fixture::new("metrics-set-refused");
    let session = fixture.session();
    let (write, _) = session.post(Some(fixture.operator_token()), WRITE).await;
    assert_eq!(write, StatusCode::OK);
    let served = r#"{"type":"ety_record","revision":1,"set":"every"}"#;
    assert_refusal_delta(
        &session,
        "no credential",
        served,
        StatusCode::UNAUTHORIZED,
        None,
    )
    .await;
    assert_refusal_delta(
        &session,
        "policy denial",
        served,
        StatusCode::FORBIDDEN,
        Some(fixture.roleless_token()),
    )
    .await;
    for (label, body, status) in [
        (
            "set body refusal",
            r#"{"type":"ety_record","revision":1}"#.to_owned(),
            StatusCode::BAD_REQUEST,
        ),
        (
            "set undeclared type",
            r#"{"type":"ety_absent","revision":1,"set":"every"}"#.to_owned(),
            StatusCode::BAD_REQUEST,
        ),
        (
            "set unretained revision",
            r#"{"type":"ety_record","revision":9,"set":"every"}"#.to_owned(),
            StatusCode::CONFLICT,
        ),
        (
            "set member ceiling",
            format!(
                r#"{{"type":"ety_record","revision":1,"set":{{"named":[{}]}}}}"#,
                many_refs()
            ),
            StatusCode::BAD_REQUEST,
        ),
        ("set leaf ceiling", leaf_chain(17), StatusCode::BAD_REQUEST),
    ] {
        assert_refusal_delta(
            &session,
            label,
            &body,
            status,
            Some(fixture.operator_token()),
        )
        .await;
    }
}
