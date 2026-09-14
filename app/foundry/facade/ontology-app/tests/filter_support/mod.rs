//! The three-tenant fixture the filter tests are judged on: the first and
//! the last tenant populated, the middle one read.
//!
//! The seeded type declares `name` and `note`; every write sets `name` and
//! nothing sets `note`, so a filter on `name` selects among objects that differ
//! only in that value, and `note` is a property the pin declares that no object
//! carries. Both ends hold objects whose `name` is their own tenant id, so a
//! filter that leaked would answer with a value no object of the reading tenant
//! carries.

#[path = "../facade_support/mod.rs"]
mod facade_support;

use axum::http::StatusCode;
pub(crate) use facade_support::{FIRST_TENANT, Fixture, LAST_TENANT, Session, TENANT};
use serde_json::Value;

pub(crate) const TYPE: &str = "ety_record";

pub(crate) async fn write(session: &Session, token: &str, object_ref: &str, name: &str) {
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

pub(crate) async fn list(session: &Session, token: &str, query: &str) -> (StatusCode, Value) {
    let (status, body) = session
        .get(Some(token), &format!("/v1/objects?{query}"))
        .await;
    let parsed = serde_json::from_str(&body)
        .unwrap_or_else(|error| panic!("the body is JSON ({error}): {body}"));
    (status, parsed)
}

pub(crate) fn refs(page: &Value) -> Vec<String> {
    page["objects"]
        .as_array()
        .expect("objects is an array")
        .iter()
        .map(|row| row["object_ref"].as_str().expect("a ref").to_owned())
        .collect()
}

/// The middle tenant holds `ent_mine_a` named "Ada" and `ent_mine_b` named
/// "Grace"; both neighbours hold objects named after themselves.
pub(crate) async fn three_tenants(case: &str) -> (Fixture, Session) {
    let fixture = Fixture::new(case);
    let session = fixture.three_tenants_session();
    for (token, name) in [
        (fixture.first_token(), FIRST_TENANT),
        (fixture.last_token(), LAST_TENANT),
    ] {
        for object_ref in ["ent_early_a", "ent_early_b"] {
            write(&session, token, object_ref, name).await;
        }
    }
    write(&session, fixture.operator_token(), "ent_mine_a", "Ada").await;
    write(&session, fixture.operator_token(), "ent_mine_b", "Grace").await;
    (fixture, session)
}
