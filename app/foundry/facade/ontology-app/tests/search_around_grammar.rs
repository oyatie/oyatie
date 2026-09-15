//! Both sides of every boundary the search-around body states: the shapes it
//! refuses, each matched to the cause naming the rule it broke, and the shapes
//! it serves.
//!
//! The seed's own grammar is the set route's, parsed by the set route's
//! function, so a seed and a set page refuse the same spelling for the same
//! reason; the rows here are the ones only this body can reach.

mod facade_support;

use axum::http::StatusCode;
use facade_support::{Fixture, Session};
use serde_json::Value;

const WALK: &str = "/v1/search-around";

/// One object of the seeded type, so a served shape has a root to walk from.
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
    let (status, reply) = session.post_to(Some(token), WALK, body).await;
    let parsed = serde_json::from_str(&reply)
        .unwrap_or_else(|error| panic!("the body is JSON ({error}): {reply}"));
    (status, parsed)
}

/// A body with every field spelled, so a row varies exactly one of them.
fn body_with(field: &str, value: &str) -> String {
    let mut fields: Vec<String> = [
        (r#""idempotency_key""#, r#""idem-g""#),
        (
            r#""seed""#,
            r#"{"type":"ety_record","revision":1,"set":"every"}"#,
        ),
        (r#""edge_types""#, r#"["lty_owns"]"#),
        (r#""max_depth""#, "2"),
        (r#""direction""#, r#""outbound""#),
        (r#""consent""#, r#""unrestricted""#),
        (r#""freshness_floor_epoch_seconds""#, "0"),
        (r#""observed_at_epoch_seconds""#, "1700000000"),
    ]
    .into_iter()
    .map(|(key, default)| {
        let spelled = if key == field { value } else { default };
        format!("{key}:{spelled}")
    })
    .collect();
    if field == "drop" {
        fields.retain(|pair| !pair.starts_with(value));
    }
    format!("{{{}}}", fields.join(","))
}

#[tokio::test]
async fn the_body_grammar_holds_at_both_edges() {
    let (fixture, session) = one_object("walk-grammar").await;
    let token = fixture.operator_token();
    let shape = "a search-around is";
    let refused: &[(String, &str)] = &[
        (String::new(), shape),
        ("{}".to_owned(), shape),
        (body_with("drop", r#""seed""#), shape),
        (body_with("drop", r#""consent""#), shape),
        (body_with("drop", r#""max_depth""#), shape),
        // A field this body does not define is refused, not ignored.
        (
            body_with(r#""direction""#, r#""outbound","extra":1"#),
            shape,
        ),
        (
            body_with(r#""idempotency_key""#, r#""   ""#),
            "must name the attempt it repeats",
        ),
        (body_with(r#""max_depth""#, "-1"), "a whole number"),
        (body_with(r#""max_depth""#, "1.5"), "a whole number"),
        (
            body_with(r#""observed_at_epoch_seconds""#, "-1"),
            "a whole number",
        ),
        (
            body_with(r#""direction""#, r#""sideways""#),
            "a direction is",
        ),
        (body_with(r#""consent""#, r#""open""#), "consent is"),
        (
            body_with(r#""consent""#, r#"{"granted":"lty_owns"}"#),
            "consent is",
        ),
        (
            body_with(r#""consent""#, r#"{"granted":[1]}"#),
            "consent is",
        ),
        (
            body_with(r#""consent""#, r#"{"granted":["owns"]}"#),
            "consent grant are both lty_",
        ),
        (
            body_with(r#""edge_types""#, r#"["owns"]"#),
            "consent grant are both lty_",
        ),
        (body_with(r#""max_depth""#, "0"), "at least one step"),
        (body_with(r#""max_depth""#, "17"), "at most sixteen steps"),
        // The seed's own grammar, refused by the set route's own causes.
        (
            body_with(
                r#""seed""#,
                r#"{"type":"ety_record","revision":1,"set":"everything"}"#,
            ),
            "a set page is",
        ),
        (
            body_with(
                r#""seed""#,
                r#"{"type":"ety_absent","revision":1,"set":"every"}"#,
            ),
            "declares no entity type by that id",
        ),
        (
            body_with(
                r#""seed""#,
                r#"{"type":"ety_record","revision":9,"set":"every"}"#,
            ),
            "never accepted",
        ),
        (
            body_with(
                r#""seed""#,
                r#"{"type":"ety_record","revision":1,"set":{"named":["ent_nowhere"]}}"#,
            ),
            "holds no object to walk from",
        ),
    ];
    for (body, expected) in refused {
        let (status, reply) = post(&session, token, body).await;
        assert_ne!(status, StatusCode::OK, "{body}: {reply}");
        let cause = reply["cause"].as_str().expect("a cause");
        assert!(cause.contains(expected), "{body} answered {cause}");
    }

    // Each served shape is judged by the root it walks from, so a status alone
    // could not tell a served walk from one that seeded nothing.
    let served: &[(String, Vec<&str>)] = &[
        (body_with("drop", r#""nothing""#), vec!["ent_mid_a"]),
        (
            body_with(r#""direction""#, r#""inbound""#),
            vec!["ent_mid_a"],
        ),
        (body_with(r#""direction""#, r#""both""#), vec!["ent_mid_a"]),
        (body_with(r#""max_depth""#, "1"), vec!["ent_mid_a"]),
        (body_with(r#""max_depth""#, "8"), vec!["ent_mid_a"]),
        (body_with(r#""edge_types""#, "[]"), vec!["ent_mid_a"]),
        (
            body_with(r#""consent""#, r#"{"granted":[]}"#),
            vec!["ent_mid_a"],
        ),
        (
            body_with(r#""freshness_floor_epoch_seconds""#, "1700000000"),
            vec!["ent_mid_a"],
        ),
    ];
    for (index, (body, expected)) in served.iter().enumerate() {
        // One key per row: a key already answered would replay its receipt.
        let body = body.replace(r#""idem-g""#, &format!(r#""idem-g{index}""#));
        let (status, reply) = post(&session, token, &body).await;
        assert_eq!(status, StatusCode::OK, "{body}: {reply}");
        let roots: Vec<&str> = reply["nodes"]
            .as_array()
            .expect("nodes is an array")
            .iter()
            .map(|node| node["entity_id"].as_str().expect("an id"))
            .collect();
        assert_eq!(&roots, expected, "{body}");
    }
}
