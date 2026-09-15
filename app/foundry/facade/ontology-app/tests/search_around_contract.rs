//! What `POST /v1/search-around` serves, judged as a per-tenant surface: three
//! tenants, the first and the last populated, the middle one reading.
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

/// A walk starts from every member of its seed, and from nothing else: the seed
/// is materialized through the same pinned view a listing reads, so a neighbour
/// s object is not a root even when the seed names its ref.
#[tokio::test]
async fn a_walk_starts_from_the_seed_the_reader_owns() {
    let (fixture, session) = three_tenants("walk-seed").await;
    let token = fixture.operator_token();
    let (status, graph) = walk(
        &session,
        Some(token),
        &walk_body("idem-every", r#""every""#, 2, r#""unrestricted""#),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{graph}");
    assert_eq!(nodes(&graph), vec!["ent_mid_a", "ent_mid_b"]);
    assert_eq!(graph["edges"].as_array().expect("edges").len(), 0);
    assert_eq!(graph["truncated"], false);

    let named = r#"{"named":["ent_mid_a","ent_early_a","ent_early_b"]}"#;
    let (status, graph) = walk(
        &session,
        Some(token),
        &walk_body("idem-named", named, 2, r#""unrestricted""#),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{graph}");
    assert_eq!(
        nodes(&graph),
        vec!["ent_mid_a"],
        "both neighbours hold the other two refs, and neither is a root here"
    );
}

/// Both depth bounds, and the difference between them. This process permits a
/// shallower walk than the domain does, so a depth between the two is a WELL
/// FORMED request the policy refuses, while one past the domain's own maximum
/// is a request that cannot be built at all.
#[tokio::test]
async fn the_policy_ceiling_binds_before_the_domains_maximum() {
    let (fixture, session) = three_tenants("walk-depth").await;
    let token = fixture.operator_token();
    let at = walk_body("idem-d8", r#""every""#, 8, r#""unrestricted""#);
    let (status, graph) = walk(&session, Some(token), &at).await;
    assert_eq!(status, StatusCode::OK, "eight steps is permitted: {graph}");

    let past_policy = walk_body("idem-d9", r#""every""#, 9, r#""unrestricted""#);
    let (status, refused) = walk(&session, Some(token), &past_policy).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{refused}");
    assert_eq!(refused["gate"], "authorization");
    assert!(
        refused["cause"]
            .as_str()
            .expect("a cause")
            .contains("shallower walk"),
        "{refused}"
    );

    let past_domain = walk_body("idem-d17", r#""every""#, 17, r#""unrestricted""#);
    let (status, refused) = walk(&session, Some(token), &past_domain).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{refused}");
    assert_eq!(refused["gate"], "surface", "{refused}");
}

/// One key answers one intent. A replay of the same walk is served again, and
/// the same key over a different walk is a conflict rather than a second
/// answer under a key that already means something.
#[tokio::test]
async fn one_idempotency_key_answers_one_walk() {
    let (fixture, session) = three_tenants("walk-idempotent").await;
    let token = fixture.operator_token();
    let body = walk_body("idem-same", r#""every""#, 2, r#""unrestricted""#);
    let (first, graph) = walk(&session, Some(token), &body).await;
    assert_eq!(first, StatusCode::OK, "{graph}");
    let (again, replayed) = walk(&session, Some(token), &body).await;
    assert_eq!(again, StatusCode::OK, "{replayed}");
    assert_eq!(graph, replayed, "a replay is the receipt already given");

    // Every field that defines the walk, one at a time: a key stands for one
    // intent, and a cached graph answered under a changed field would be the
    // old walk reported as the new one.
    let changed = [
        walk_body("idem-same", r#""every""#, 3, r#""unrestricted""#),
        walk_body(
            "idem-same",
            r#"{"named":["ent_mid_a"]}"#,
            2,
            r#""unrestricted""#,
        ),
        walk_body("idem-same", r#""every""#, 2, r#"{"granted":[]}"#),
        body.replace(r#""direction":"outbound""#, r#""direction":"inbound""#),
        body.replace("1700000000", "1700000001"),
    ];
    for other in &changed {
        let (status, conflict) = walk(&session, Some(token), other).await;
        assert_eq!(status, StatusCode::CONFLICT, "{other}: {conflict}");
    }
    // A grant list is a SET: reordering or repeating it is the same posture and
    // replays, where a different posture conflicts above.
    let granted = walk_body(
        "idem-grants",
        r#""every""#,
        2,
        r#"{"granted":["lty_owns","lty_partner"]}"#,
    );
    let (status, first) = walk(&session, Some(token), &granted).await;
    assert_eq!(status, StatusCode::OK, "{first}");
    let reordered = granted.replace(
        r#"["lty_owns","lty_partner"]"#,
        r#"["lty_partner","lty_owns","lty_owns"]"#,
    );
    let (status, replay) = walk(&session, Some(token), &reordered).await;
    assert_eq!(status, StatusCode::OK, "{replay}");
    assert_eq!(first, replay, "one posture, one intent");

    let other = &changed[0];
    let (status, conflict) = walk(&session, Some(token), other).await;
    assert_eq!(status, StatusCode::CONFLICT, "{conflict}");
    assert!(
        conflict["cause"]
            .as_str()
            .expect("a cause")
            .contains("already answering another query"),
        "{conflict}"
    );
}

/// Consent has no silent default. Both postures are served here — this
/// facade writes no links, so neither traverses an edge — and the wire refuses
/// a grant that is not a link type id.
#[tokio::test]
async fn consent_is_stated_and_its_grants_are_link_types() {
    let (fixture, session) = three_tenants("walk-consent").await;
    let token = fixture.operator_token();
    for (key, consent) in [
        ("idem-c1", r#""unrestricted""#),
        ("idem-c2", r#"{"granted":[]}"#),
        ("idem-c3", r#"{"granted":["lty_owns"]}"#),
    ] {
        let (status, graph) = walk(
            &session,
            Some(token),
            &walk_body(key, r#""every""#, 2, consent),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{consent}: {graph}");
        assert_eq!(nodes(&graph), vec!["ent_mid_a", "ent_mid_b"], "{consent}");
    }
    let (status, refused) = walk(
        &session,
        Some(token),
        &walk_body("idem-c4", r#""every""#, 2, r#"{"granted":["owns"]}"#),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{refused}");
    assert_eq!(refused["gate"], "surface");
}

#[tokio::test]
async fn a_walk_refuses_a_caller_it_cannot_place_in_a_tenant() {
    let (fixture, session) = three_tenants("walk-credential").await;
    let body = walk_body("idem-cred", r#""every""#, 2, r#""unrestricted""#);
    for (token, expected) in [
        (None, StatusCode::UNAUTHORIZED),
        (Some("not-a-token"), StatusCode::UNAUTHORIZED),
        (Some(fixture.roleless_token()), StatusCode::FORBIDDEN),
    ] {
        let (status, reply) = walk(&session, token, &body).await;
        assert_eq!(status, expected, "{reply}");
    }
}
