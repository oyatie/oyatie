//! What `POST /v1/object-sets/page` serves, judged as a per-tenant surface:
//! three tenants, the first and the last populated, the middle one reading.
//!
//! Both neighbours hold objects under refs the reader has none of and named
//! after themselves, so a set that leaked would either name one of those refs
//! or match one of those names.

mod facade_support;

use axum::http::StatusCode;
use facade_support::{FIRST_TENANT, Fixture, LAST_TENANT, Session};
use serde_json::Value;

const PAGE: &str = "/v1/object-sets/page";

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

/// The reader holds `ent_mid_a` named Ada and `ent_mid_b` named Grace; each
/// neighbour holds two objects under its own refs, named after itself.
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

async fn set_page(session: &Session, token: &str, set: &str) -> (StatusCode, Value) {
    let body = format!(r#"{{"type":"ety_record","revision":1,{set}}}"#);
    let (status, reply) = session.post_to(Some(token), PAGE, &body).await;
    let parsed = serde_json::from_str(&reply)
        .unwrap_or_else(|error| panic!("the body is JSON ({error}): {reply}"));
    (status, parsed)
}

fn refs_of(page: &Value) -> Vec<String> {
    page["objects"]
        .as_array()
        .expect("objects is an array")
        .iter()
        .map(|row| row["object_ref"].as_str().expect("a ref").to_owned())
        .collect()
}

/// A named set holds the refs the reader owns; a neighbour's ref and a ref no
/// tenant holds are each absent, and neither is a refusal. `every` is the
/// control on what the reader had to name.
#[tokio::test]
async fn a_named_set_serves_only_the_refs_the_reader_owns() {
    let (fixture, session) = three_tenants("set-named").await;
    let token = fixture.operator_token();
    let (status, page) = set_page(
        &session,
        token,
        r#""set":{"named":["ent_mid_a","ent_early_a","ent_early_b","ent_nowhere"]}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{page}");
    assert_eq!(refs_of(&page), vec!["ent_mid_a"]);
    assert_eq!(page["objects"][0]["properties"]["name"]["value"], "Ada");

    let (status, every) = set_page(&session, token, r#""set":"every""#).await;
    assert_eq!(status, StatusCode::OK, "{every}");
    assert_eq!(refs_of(&every), vec!["ent_mid_a", "ent_mid_b"]);
}

/// Each operation over the route, and a filter leaf selecting by a value both
/// neighbours carry: their objects exist and are named exactly that.
#[tokio::test]
async fn the_operations_compose_over_the_route() {
    let (fixture, session) = three_tenants("set-operations").await;
    let token = fixture.operator_token();
    let ada = r#"{"matching":{"property":"name","equals":"text:Ada"}}"#;
    let named = r#"{"named":["ent_mid_a","ent_mid_b"]}"#;
    for (set, expected) in [
        (
            format!(r#""set":{{"union":[{named},{ada}]}}"#),
            vec!["ent_mid_a", "ent_mid_b"],
        ),
        (
            format!(r#""set":{{"intersect":[{named},{ada}]}}"#),
            vec!["ent_mid_a"],
        ),
        (
            format!(r#""set":{{"subtract":[{named},{ada}]}}"#),
            vec!["ent_mid_b"],
        ),
        (format!(r#""set":{{"subtract":[{ada},{named}]}}"#), vec![]),
    ] {
        let (status, page) = set_page(&session, token, &set).await;
        assert_eq!(status, StatusCode::OK, "{set}: {page}");
        assert_eq!(refs_of(&page), expected, "{set}");
    }

    let leaked =
        format!(r#""set":{{"matching":{{"property":"name","equals":"text:{LAST_TENANT}"}}}}"#);
    let (status, page) = set_page(&session, token, &leaked).await;
    assert_eq!(status, StatusCode::OK, "{page}");
    assert_eq!(
        refs_of(&page),
        Vec::<String>::new(),
        "a neighbour's own name selects none of the reader's objects"
    );
}

/// A set of every object of a type is the listing of that type: the same rows,
/// the same cursor, page for page, so a caller cannot learn from a set what the
/// listing would not tell them. Below the member ceiling: past it the set is
/// refused where the listing still pages, which is the one place the two
/// surfaces differ.
#[tokio::test]
async fn an_every_set_pages_exactly_as_the_listing_does() {
    let (fixture, session) = three_tenants("set-paging").await;
    let token = fixture.operator_token();
    let body = r#"{"type":"ety_record","revision":1,"limit":1,"set":"every"}"#;
    let (status, first) = session.post_to(Some(token), PAGE, body).await;
    assert_eq!(status, StatusCode::OK, "{first}");
    let (status, listed) = session
        .get(
            Some(token),
            "/v1/objects?type=ety_record&revision=1&limit=1",
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{listed}");
    assert_eq!(first, listed);

    let resumed =
        r#"{"type":"ety_record","revision":1,"limit":1,"cursor":"ent_mid_a","set":"every"}"#;
    let (status, second) = session.post_to(Some(token), PAGE, resumed).await;
    assert_eq!(status, StatusCode::OK, "{second}");
    let (status, listed) = session
        .get(
            Some(token),
            "/v1/objects?type=ety_record&revision=1&limit=1&cursor=ent_mid_a",
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{listed}");
    assert_eq!(second, listed);
}

#[tokio::test]
async fn a_set_page_refuses_a_caller_it_cannot_place_in_a_tenant() {
    let (fixture, session) = three_tenants("set-credential").await;
    let body = r#"{"type":"ety_record","revision":1,"set":"every"}"#;
    for (token, expected) in [
        (None, StatusCode::UNAUTHORIZED),
        (Some("not-a-token"), StatusCode::UNAUTHORIZED),
        (Some(fixture.roleless_token()), StatusCode::FORBIDDEN),
    ] {
        let (status, reply) = session.post_to(token, PAGE, body).await;
        assert_eq!(status, expected, "{reply}");
    }
}

/// A stranger is asked for a credential before the body is judged: the body
/// here is unusable at every level, and the answer names none of it.
#[tokio::test]
async fn an_unusable_body_from_a_stranger_is_still_a_credential_refusal() {
    let (_fixture, session) = three_tenants("set-body-after-credential").await;
    let (status, reply) = session.post_to(None, PAGE, "{\"set\":\"nonsense\"").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "{reply}");
    assert_eq!(
        serde_json::from_str::<Value>(&reply).expect("JSON")["gate"],
        "credential"
    );
}

/// The null condition: the reader holds nothing while both neighbours hold
/// objects under the refs it would have used. Every variant of a definition
/// serves an empty page, and none of them is a refusal — a set cannot borrow a
/// neighbour's rows by asking for its own.
#[tokio::test]
async fn a_reader_holding_nothing_is_served_nothing_of_its_neighbours() {
    let fixture = Fixture::new("set-null");
    let session = fixture.three_tenants_session();
    for (token, tenant) in [
        (fixture.first_token(), FIRST_TENANT),
        (fixture.last_token(), LAST_TENANT),
    ] {
        for object_ref in ["ent_mid_a", "ent_mid_b"] {
            write(&session, token, object_ref, tenant).await;
        }
    }
    let ada = r#"{"matching":{"property":"name","equals":"text:Ada"}}"#;
    let refs = r#"{"named":["ent_mid_a","ent_mid_b"]}"#;
    for set in [
        r#""set":"every""#.to_owned(),
        format!(r#""set":{refs}"#),
        format!(r#""set":{ada}"#),
        format!(r#""set":{{"union":[{refs},{ada}]}}"#),
        format!(r#""set":{{"intersect":["every",{ada}]}}"#),
        format!(r#""set":{{"subtract":["every",{refs}]}}"#),
    ] {
        let (status, page) = set_page(&session, fixture.operator_token(), &set).await;
        assert_eq!(status, StatusCode::OK, "{set}: {page}");
        assert_eq!(refs_of(&page), Vec::<String>::new(), "{set}");
    }
}
