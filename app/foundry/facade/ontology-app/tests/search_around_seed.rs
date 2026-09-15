//! What a SEED must hold for a walk to happen, and the order it is judged in:
//! three tenants, the first and the last populated, the middle one reading.
//!
//! This facade's write path creates no LINKS, so every walk here returns its
//! seed's own members at depth zero and no edges. That is a limit of what can
//! be written, not of the route: the seed, the pin, the policy ceiling, the
//! consent posture and the idempotency of an execution are all exercised, and
//! the traversal itself is the query domain's own, tested there. A lane that
//! adds link writes is what makes a multi-hop walk assertable here.

mod facade_support;

use axum::http::StatusCode;
use facade_support::{FIRST_TENANT, Fixture, LAST_TENANT, Session};
use serde_json::Value;

const WALK: &str = "/v1/search-around";

async fn write(session: &Session, token: &str, object_ref: &str, name: &str) {
    let body = format!(
        r#"{{"object_ref":"{object_ref}","action_type":"aty_record_write","idempotency_key":"idem-{object_ref}-{name}","occurred_at_epoch_seconds":1700000000,"properties":{{"name":"{name}"}}}}"#
    );
    let (status, reply) = session.post(Some(token), &body).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "the fixture write must land: {reply}"
    );
}

/// The reader holds `ent_mid_a` and `ent_mid_b`; each neighbour holds two
/// objects under its own refs, so a walk that leaked would name one of them.
async fn three_tenants(case: &str) -> (Fixture, Session) {
    let fixture = Fixture::new(case);
    let session = fixture.three_tenants_session();
    for (token, tenant) in [
        (fixture.first_token(), FIRST_TENANT),
        (fixture.last_token(), LAST_TENANT),
    ] {
        for object_ref in ["ent_early_a", "ent_early_b"] {
            write(&session, token, object_ref, tenant).await;
        }
    }
    write(&session, fixture.operator_token(), "ent_mid_a", "Ada").await;
    write(&session, fixture.operator_token(), "ent_mid_b", "Grace").await;
    (fixture, session)
}

/// One walk, with every field spelled: `parts` overrides the defaults.
fn walk_body(key: &str, set: &str, depth: u32, consent: &str) -> String {
    format!(
        r#"{{"idempotency_key":"{key}","seed":{{"type":"ety_record","revision":1,"set":{set}}},"edge_types":["lty_owns"],"max_depth":{depth},"direction":"outbound","consent":{consent},"freshness_floor_epoch_seconds":0,"observed_at_epoch_seconds":1700000000}}"#
    )
}

async fn walk(session: &Session, token: Option<&str>, body: &str) -> (StatusCode, Value) {
    let (status, reply) = session.post_to(token, WALK, body).await;
    let parsed = serde_json::from_str(&reply)
        .unwrap_or_else(|error| panic!("the body is JSON ({error}): {reply}"));
    (status, parsed)
}

fn nodes(graph: &Value) -> Vec<String> {
    graph["nodes"]
        .as_array()
        .expect("nodes is an array")
        .iter()
        .map(|node| node["entity_id"].as_str().expect("an id").to_owned())
        .collect()
}

/// A seed that holds nothing cannot be walked: the domain refuses a walk from a
/// root it cannot find, so there is no graph to answer. The refusal names the
/// seed, and it consumes no idempotency key — the same key answers a real walk
/// afterwards.
#[tokio::test]
async fn a_seed_that_holds_nothing_is_refused_and_keeps_no_key() {
    let (fixture, session) = three_tenants("walk-empty").await;
    let token = fixture.operator_token();
    let key = "idem-empty";
    let (status, refused) = walk(
        &session,
        Some(token),
        &walk_body(key, r#"{"named":["ent_nowhere"]}"#, 2, r#""unrestricted""#),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{refused}");
    assert_eq!(refused["gate"], "surface");
    assert!(
        refused["cause"]
            .as_str()
            .expect("a cause")
            .contains("holds no object to walk from"),
        "{refused}"
    );

    let (status, graph) = walk(
        &session,
        Some(token),
        &walk_body(key, r#""every""#, 2, r#""unrestricted""#),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{graph}");
    assert_eq!(nodes(&graph), vec!["ent_mid_a", "ent_mid_b"]);
}

/// An empty seed is judged BEFORE the walk's own fields, so a request wrong in
/// both ways is told about its seed — and the walk's fields are refused
/// identically whenever the seed holds something, which is the case the
/// refusals are for.
#[tokio::test]
async fn an_empty_seed_is_judged_before_the_walks_own_fields() {
    let (fixture, session) = three_tenants("walk-empty-strict").await;
    let token = fixture.operator_token();
    let nowhere = r#"{"named":["ent_nowhere"]}"#;
    for (label, depth, consent, expected) in [
        (
            "zero depth",
            0,
            r#""unrestricted""#,
            StatusCode::BAD_REQUEST,
        ),
        (
            "past the policy",
            9,
            r#""unrestricted""#,
            StatusCode::FORBIDDEN,
        ),
        (
            "past the domain",
            17,
            r#""unrestricted""#,
            StatusCode::BAD_REQUEST,
        ),
        (
            "a grant that is not a link type",
            2,
            r#"{"granted":["owns"]}"#,
            StatusCode::BAD_REQUEST,
        ),
    ] {
        let key = format!("idem-strict-{depth}-{}", consent.len());
        let (status, reply) = walk(
            &session,
            Some(token),
            &walk_body(&key, nowhere, depth, consent),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "{label}, empty seed: {reply}"
        );
        assert!(
            reply["cause"]
                .as_str()
                .expect("a cause")
                .contains("holds no object to walk from"),
            "{label}: the seed is judged first, {reply}"
        );
        let (status, reply) = walk(
            &session,
            Some(token),
            &walk_body(&format!("{key}-full"), r#""every""#, depth, consent),
        )
        .await;
        assert_eq!(status, expected, "{label} with a seed that holds: {reply}");
    }
}
