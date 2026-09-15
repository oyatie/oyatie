//! Both sides of every LIMIT the set route states: the leaves a definition may
//! read, the objects a step may take, and the depth the body itself accepts.
//!
//! Each is asserted at the limit and past it, so no refusal here can be the
//! route being closed.

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

/// Both sides of the LEAF ceiling: a definition at it is served, one past it is
/// refused, and the refusal names what was counted. The count is leaves, not
/// operand positions — a union of sixteen leaves has thirty of those.
#[tokio::test]
async fn a_definition_past_the_leaf_ceiling_is_refused() {
    let (fixture, session) = one_object("set-operands").await;
    let token = fixture.operator_token();
    // `"every"` leaves rather than empty named ones. Both are leaves and both
    // are counted, so either witnesses the ceiling; these read the store, which
    // is the work the ceiling exists to bound.
    let chain = |operands: usize| {
        (1..operands).fold(r#""every""#.to_owned(), |left, _| {
            format!(r#"{{"union":[{left},"every"]}}"#)
        })
    };
    let at = format!(
        r#"{{"type":"ety_record","revision":1,"set":{}}}"#,
        chain(16)
    );
    let (status, reply) = post(&session, token, &at).await;
    assert_eq!(status, StatusCode::OK, "{reply}");
    assert_eq!(
        reply["objects"][0]["object_ref"], "ent_mid_a",
        "sixteen reading leaves are served, and a union of a type with itself is that type: {reply}"
    );

    let past = format!(
        r#"{{"type":"ety_record","revision":1,"set":{}}}"#,
        chain(17)
    );
    let (status, reply) = post(&session, token, &past).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{reply}");
    assert!(
        reply["cause"]
            .as_str()
            .expect("a cause")
            .contains("at most 16 leaves"),
        "{reply}"
    );
}

/// Both sides of the member ceiling. The refs are named rather than stored,
/// because the ceiling is judged on what the definition can hold before the
/// store is read: a thousand names of absent objects is a served, empty page,
/// and one more name is refused.
#[tokio::test]
async fn a_named_set_past_the_member_ceiling_is_refused() {
    let (fixture, session) = one_object("set-members").await;
    let token = fixture.operator_token();
    let refs = |count: usize| {
        let named = (0..count)
            .map(|index| format!(r#""ent_{index:05}""#))
            .collect::<Vec<String>>()
            .join(",");
        format!(r#"{{"type":"ety_record","revision":1,"set":{{"named":[{named}]}}}}"#)
    };
    let (status, reply) = post(&session, token, &refs(1000)).await;
    assert_eq!(status, StatusCode::OK, "{reply}");
    assert_eq!(reply["objects"].as_array().expect("an array").len(), 0);

    let (status, reply) = post(&session, token, &refs(1001)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{reply}");
    assert!(
        reply["cause"]
            .as_str()
            .expect("a cause")
            .contains("more than 1000 objects"),
        "{reply}"
    );
}

/// Both sides of the depth the body accepts. The leaf ceiling bounds how many
/// leaves a definition reads and bounds depth only as a consequence; what stands
/// between a deeply nested body and a blown stack is the deserializer, which
/// refuses BEFORE evaluation recurses over the tree. It counts two levels per
/// union node (the object and its array) against its own 128-level limit, which
/// puts the boundary at 63 union nodes deep.
///
/// At 63 the body parses and the refusal comes from the domain's leaf ceiling;
/// at 64 the body is refused before anything recurses over it.
#[tokio::test]
async fn a_body_deeper_than_the_deserializer_accepts_is_refused_before_evaluation() {
    let (fixture, session) = one_object("set-depth").await;
    let token = fixture.operator_token();
    let nest = |depth: usize| {
        let set = (0..depth).fold(r#""every""#.to_owned(), |inner, _| {
            format!(r#"{{"union":[{inner},"every"]}}"#)
        });
        format!(r#"{{"type":"ety_record","revision":1,"set":{set}}}"#)
    };
    let (status, reply) = post(&session, token, &nest(63)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{reply}");
    assert!(
        reply["cause"]
            .as_str()
            .expect("a cause")
            .contains("at most 16 leaves"),
        "{reply}"
    );
    let (status, reply) = post(&session, token, &nest(64)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{reply}");
    assert!(
        reply["cause"]
            .as_str()
            .expect("a cause")
            .contains("a set page is"),
        "{reply}"
    );
}
