//! Both sides of every boundary the set body states: the shapes it refuses,
//! each matched to its own cause, and the shapes it serves.
//!
//! The served shapes prove a refusal is the body under test and not the route,
//! and the filter clause is refused by the same causes `GET /v1/objects`
//! gives it, because both routes parse it with one function.

mod facade_support;

use axum::http::StatusCode;
use facade_support::{Fixture, Session};
use serde_json::Value;

const PAGE: &str = "/v1/object-sets/page";

/// One object of the seeded type, so a served shape has a row to answer with.
async fn one_object(case: &str) -> (Fixture, Session) {
    let fixture = Fixture::new(case);
    let session = fixture.session();
    let body = r#"{"object_ref":"ent_mid_a","action_type":"aty_record_write","idempotency_key":"idem-a","occurred_at_epoch_seconds":1700000000,"properties":{"name":"Ada"}}"#;
    let (status, reply) = session.post(Some(fixture.operator_token()), body).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "the fixture write must land: {reply}"
    );
    (fixture, session)
}

async fn post(session: &Session, token: &str, body: &str) -> (StatusCode, Value) {
    let (status, reply) = session.post_to(Some(token), PAGE, body).await;
    let parsed = serde_json::from_str(&reply)
        .unwrap_or_else(|error| panic!("the body is JSON ({error}): {reply}"));
    (status, parsed)
}

/// Every shape the body refuses, each with the cause that names what was
/// measured, and the shapes it serves.
#[tokio::test]
async fn the_body_grammar_holds_at_both_edges() {
    let (fixture, session) = one_object("set-grammar").await;
    let token = fixture.operator_token();
    let refused: &[(&str, &str)] = &[
        ("", "a set page is"),
        ("{}", "a set page is"),
        (r#"{"type":"ety_record","set":"every"}"#, "a set page is"),
        (r#"{"revision":1,"set":"every"}"#, "a set page is"),
        (r#"{"type":"ety_record","revision":1}"#, "a set page is"),
        (
            r#"{"type":"ety_record","revision":1,"set":"everything"}"#,
            "a set page is",
        ),
        (
            r#"{"type":"ety_record","revision":1,"set":"every","extra":1}"#,
            "a set page is",
        ),
        (
            r#"{"type":"ety_record","revision":1,"set":{"union":["every"]}}"#,
            "a set page is",
        ),
        (
            r#"{"type":"ety_record","revision":1,"set":{"union":["every","every","every"]}}"#,
            "a set page is",
        ),
        (
            r#"{"type":"ety_record","revision":1,"set":{"matching":{"property":"name"}}}"#,
            "either an equality or both ends",
        ),
        (
            r#"{"type":"ety_record","revision":1,"set":{"matching":{"property":"name","equals":"Ada"}}}"#,
            "text:, int: or bool:",
        ),
        (
            r#"{"type":"ety_record","revision":1,"set":{"matching":{"property":"","equals":"text:Ada"}}}"#,
            "must name the property it constrains",
        ),
        (
            r#"{"type":"ety_record","revision":1,"set":{"matching":{"property":" name","equals":"text:Ada"}}}"#,
            "must name the property it constrains",
        ),
        (
            r#"{"type":"ety_record","revision":1,"set":{"matching":{"property":"name","from":"int:1"}}}"#,
            "either an equality or both ends",
        ),
        (
            r#"{"type":"ety_record","revision":1,"set":{"matching":{"property":"name","from":"int:5","to":"int:1"}}}"#,
            "may not sort after",
        ),
        (
            r#"{"type":"ety_record","revision":1,"limit":0,"set":"every"}"#,
            "from 1 to 1000",
        ),
        (
            r#"{"type":"ety_record","revision":1,"limit":1001,"set":"every"}"#,
            "from 1 to 1000",
        ),
        (
            r#"{"type":"ety_record","revision":1,"limit":-1,"set":"every"}"#,
            "from 1 to 1000",
        ),
        (
            r#"{"type":"ety_record","revision":1,"limit":1.5,"set":"every"}"#,
            "from 1 to 1000",
        ),
        (
            r#"{"type":"ety_record","revision":1,"limit":"100","set":"every"}"#,
            "from 1 to 1000",
        ),
        (
            r#"{"type":"ety_record","revision":-1,"set":"every"}"#,
            "must pin the revision it understands, as a whole number",
        ),
        (
            r#"{"type":"ety_record","revision":"1","set":"every"}"#,
            "must pin the revision it understands, as a whole number",
        ),
        (
            r#"{"type":"ety_record","revision":null,"set":"every"}"#,
            "must pin the revision it understands, as a whole number",
        ),
        (
            r#"{"type":"ety_record","revision":1,"cursor":5,"set":"every"}"#,
            "the object reference a page ended on",
        ),
        (
            r#"{"type":"ety_record","revision":1,"set":{"matching":{"property":"name","equals":"text:Ada","from":"text:A","to":"text:z"}}}"#,
            "not both",
        ),
        (
            r#"{"type":"ety_record","revision":1,"cursor":"not-a-ref","set":"every"}"#,
            "the object reference a page ended on",
        ),
        (
            r#"{"type":"ety_absent","revision":1,"set":"every"}"#,
            "declares no entity type by that id",
        ),
        (
            r#"{"type":"ety_record","revision":1,"set":{"matching":{"property":"absent","equals":"text:Ada"}}}"#,
            "declares no property by that name",
        ),
    ];
    for (body, expected) in refused {
        let (status, reply) = post(&session, token, body).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}: {reply}");
        assert_eq!(reply["gate"], "surface", "{body}");
        let cause = reply["cause"].as_str().expect("a cause");
        assert!(cause.contains(expected), "{body} answered {cause}");
    }

    // Each served shape is judged by the rows it selects: a status alone would
    // pass while the set was ignored and every row came back.
    let served: &[(&str, Vec<&str>)] = &[
        (
            r#"{"type":"ety_record","revision":1,"set":"every"}"#,
            vec!["ent_mid_a"],
        ),
        (
            r#"{"type":"ety_record","revision":1,"limit":1,"set":"every"}"#,
            vec!["ent_mid_a"],
        ),
        (
            r#"{"type":"ety_record","revision":1,"limit":1000,"set":"every"}"#,
            vec!["ent_mid_a"],
        ),
        (
            r#"{"type":"ety_record","revision":1,"cursor":"ent_mid_a","set":"every"}"#,
            vec![],
        ),
        (
            r#"{"type":"ety_record","revision":1,"set":{"named":[]}}"#,
            vec![],
        ),
        // An explicit null is absence for EVERY optional field here, as serde
        // reads it: the page bounds fall back to their defaults, and a filter
        // clause spelled `null` is a clause that was not given. A query string
        // cannot spell absence at all — `?limit=` is an unusable value there,
        // not a default. Nor can it spell a type: `?limit=100` is the string
        // "100" and is served, while `"limit":"100"` is refused above, because
        // a JSON body states the type and a query string does not.
        (
            r#"{"type":"ety_record","revision":1,"limit":null,"cursor":null,"set":"every"}"#,
            vec!["ent_mid_a"],
        ),
        (
            r#"{"type":"ety_record","revision":1,"set":{"matching":{"property":"name","equals":"text:Ada","from":null,"to":null}}}"#,
            vec!["ent_mid_a"],
        ),
        (
            r#"{"type":"ety_record","revision":1,"set":{"matching":{"property":"name","equals":"text:Ada"}}}"#,
            vec!["ent_mid_a"],
        ),
        (
            r#"{"type":"ety_record","revision":1,"set":{"matching":{"property":"name","from":"text:A","to":"text:z"}}}"#,
            vec!["ent_mid_a"],
        ),
        (
            r#"{"type":"ety_record","revision":1,"set":{"subtract":["every",{"named":["ent_mid_a"]}]}}"#,
            vec![],
        ),
    ];
    for (body, expected) in served {
        let (status, reply) = post(&session, token, body).await;
        assert_eq!(status, StatusCode::OK, "{body}: {reply}");
        let refs: Vec<&str> = reply["objects"]
            .as_array()
            .expect("objects is an array")
            .iter()
            .map(|row| row["object_ref"].as_str().expect("a ref"))
            .collect();
        assert_eq!(&refs, expected, "{body}");
    }
}

/// A revision the tenant never accepted is a conflict, not a bad request: the
/// body is well formed and names a type the tenant declares.
#[tokio::test]
async fn a_pin_the_tenant_never_accepted_is_a_conflict() {
    let (fixture, session) = one_object("set-pin").await;
    let (status, reply) = post(
        &session,
        fixture.operator_token(),
        r#"{"type":"ety_record","revision":9,"set":"every"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{reply}");
    assert!(
        reply["cause"]
            .as_str()
            .expect("a cause")
            .contains("never accepted"),
        "{reply}"
    );
}

/// A range whose bounds are of one kind over stored values of another is a
/// conflict, as it is on the listing: the caller's bounds are well formed and
/// the store is not unreadable.
#[tokio::test]
async fn a_range_over_values_of_another_kind_is_a_conflict() {
    let (fixture, session) = one_object("set-kind").await;
    let (status, reply) = post(
        &session,
        fixture.operator_token(),
        r#"{"type":"ety_record","revision":1,"set":{"matching":{"property":"name","from":"int:1","to":"int:9"}}}"#,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{reply}");
    assert!(
        reply["cause"]
            .as_str()
            .expect("a cause")
            .contains("another kind"),
        "{reply}"
    );
}
