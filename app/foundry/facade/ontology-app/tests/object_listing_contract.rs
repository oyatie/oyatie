//! `GET /v1/objects` judged as a per-tenant surface: three tenants, the
//! first and the last populated, the middle one read. Both ends hold
//! objects of the same type at references that sort BEFORE every reference
//! the middle uses, so a listing that leaked would surface as a foreign
//! reference at the head of the page, and the empty case is asserted with
//! those neighbours still populated — an empty page must mean "this tenant
//! has none", never "the query found none".

mod facade_support;

use axum::http::StatusCode;
use facade_support::{FIRST_TENANT, Fixture, LAST_TENANT, Session, TENANT};
use serde_json::Value;

const TYPE: &str = "ety_record";

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

async fn list(session: &Session, token: &str, query: &str) -> (StatusCode, Value) {
    let (status, body) = session
        .get(Some(token), &format!("/v1/objects?{query}"))
        .await;
    let parsed = serde_json::from_str(&body)
        .unwrap_or_else(|error| panic!("the body is JSON ({error}): {body}"));
    (status, parsed)
}

fn refs(page: &Value) -> Vec<String> {
    page["objects"]
        .as_array()
        .expect("objects is an array")
        .iter()
        .map(|row| row["object_ref"].as_str().expect("a ref").to_owned())
        .collect()
}

/// The three-tenant fixture, with the ends populated and the middle's own
/// objects written last so nothing depends on write order.
async fn three_tenants(case: &str, middle: &[&str]) -> (Fixture, Session) {
    let fixture = Fixture::new(case);
    let session = fixture.three_tenants_session();
    for object_ref in ["ent_early_a", "ent_early_b"] {
        write(&session, fixture.first_token(), object_ref, FIRST_TENANT).await;
    }
    for object_ref in ["ent_early_a", "ent_early_b", "ent_early_c"] {
        write(&session, fixture.last_token(), object_ref, LAST_TENANT).await;
    }
    for object_ref in middle {
        write(&session, fixture.operator_token(), object_ref, TENANT).await;
    }
    (fixture, session)
}

#[tokio::test]
async fn a_listing_answers_the_reading_tenants_own_objects_and_no_others() {
    let (fixture, session) = three_tenants("listing-own", &["ent_mine_a", "ent_mine_b"]).await;
    let (status, page) = list(
        &session,
        fixture.operator_token(),
        &format!("type={TYPE}&revision=1"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{page}");
    assert_eq!(refs(&page), vec!["ent_mine_a", "ent_mine_b"]);
    assert_eq!(page["next"], Value::Null, "one page holds them all");
    assert_eq!(
        page["objects"][0]["properties"]["name"]["value"], TENANT,
        "the values served are the reading tenant's own"
    );
}

/// The null condition, with both neighbours holding objects of this very
/// type: an empty page here is this tenant's emptiness, not the query's.
#[tokio::test]
async fn a_tenant_holding_none_of_the_type_is_served_an_empty_page() {
    let (fixture, session) = three_tenants("listing-empty", &[]).await;
    let (status, page) = list(
        &session,
        fixture.operator_token(),
        &format!("type={TYPE}&revision=1"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{page}");
    assert_eq!(refs(&page), Vec::<String>::new());
    assert_eq!(page["next"], Value::Null, "nothing remains past nothing");

    let (status, neighbour) = list(
        &session,
        fixture.last_token(),
        &format!("type={TYPE}&revision=1"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{neighbour}");
    assert_eq!(
        refs(&neighbour).len(),
        3,
        "the neighbour's objects were there to be leaked and were not"
    );
}

#[tokio::test]
async fn two_pages_partition_the_objects_with_no_overlap_and_no_gap() {
    let (fixture, session) =
        three_tenants("listing-pages", &["ent_mine_a", "ent_mine_b", "ent_mine_c"]).await;
    let (status, first) = list(
        &session,
        fixture.operator_token(),
        &format!("type={TYPE}&revision=1&limit=2"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{first}");
    assert_eq!(refs(&first), vec!["ent_mine_a", "ent_mine_b"]);
    let cursor = first["next"]
        .as_str()
        .expect("a full page carries a cursor");
    assert_eq!(
        cursor, "ent_mine_b",
        "the cursor is the ref the page ended on"
    );

    let (status, second) = list(
        &session,
        fixture.operator_token(),
        &format!("type={TYPE}&revision=1&limit=2&cursor={cursor}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{second}");
    assert_eq!(refs(&second), vec!["ent_mine_c"]);
    assert_eq!(second["next"], Value::Null, "a short page ends the listing");

    let mut seen = refs(&first);
    seen.extend(refs(&second));
    assert_eq!(seen, vec!["ent_mine_a", "ent_mine_b", "ent_mine_c"]);

    // `next` is present exactly when objects remain: a page whose limit
    // equals the count left carries none, so a caller is never sent back
    // for a page that would be empty.
    let (status, exact) = list(
        &session,
        fixture.operator_token(),
        &format!("type={TYPE}&revision=1&limit=3"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{exact}");
    assert_eq!(refs(&exact), vec!["ent_mine_a", "ent_mine_b", "ent_mine_c"]);
    assert_eq!(
        exact["next"],
        Value::Null,
        "a page holding the last object carries no cursor"
    );
}

/// A cursor is a position, not a capability. The reference used here is a
/// neighbour's and sorts BEFORE every object the reader owns, so resuming
/// at it must yield all of the reader's own and none of the neighbour's —
/// a cursor that sorted after them would pass on an empty page.
#[tokio::test]
async fn a_cursor_naming_another_tenants_object_lists_only_the_readers_own() {
    let (fixture, session) =
        three_tenants("listing-foreign-cursor", &["ent_mine_a", "ent_mine_b"]).await;
    let (status, page) = list(
        &session,
        fixture.operator_token(),
        &format!("type={TYPE}&revision=1&cursor=ent_early_a"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{page}");
    assert_eq!(
        refs(&page),
        vec!["ent_mine_a", "ent_mine_b"],
        "resumed past a neighbour's reference, inside the reader's own objects"
    );
}

/// Both sides of every stated boundary of the query contract. The served
/// rows prove the refusals are the value under test and not the route.
#[tokio::test]
async fn the_query_contract_holds_at_both_edges() {
    let (fixture, session) = three_tenants("listing-query", &["ent_mine_a"]).await;
    let token = fixture.operator_token();

    let refused: &[(&str, &str)] = &[
        ("revision=1", "must name the entity type"),
        (&format!("type={TYPE}"), "must pin the revision"),
        (
            &format!("type={TYPE}&revision=abc"),
            "must pin the revision",
        ),
        (
            &format!("type={TYPE}&revision=1&colour=red"),
            "defines type",
        ),
        (&format!("type={TYPE}&revision=1&limit=2&limit=3"), "twice"),
        (&format!("type={TYPE}&revision=1&limit=0"), "1 to 1000"),
        (&format!("type={TYPE}&revision=1&limit=1001"), "1 to 1000"),
        (&format!("type={TYPE}&revision=1&limit=abc"), "1 to 1000"),
        (
            &format!("type={TYPE}&revision=1&cursor=nope"),
            "object reference",
        ),
        ("type=ety_absent&revision=1", "declares no entity type"),
        ("type=not_prefixed&revision=1", "declares no entity type"),
    ];
    for (query, expected) in refused {
        let (status, body) = list(&session, token, query).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{query}: {body}");
        assert_eq!(body["gate"], "surface", "{query}");
        let cause = body["cause"].as_str().expect("a cause");
        assert!(cause.contains(expected), "{query} answered {cause}");
    }

    let served: &[&str] = &[
        &format!("type={TYPE}&revision=1&limit=1"),
        &format!("type={TYPE}&revision=1&limit=1000"),
        &format!("type={TYPE}&revision=1&cursor=ent_"),
    ];
    for query in served {
        let (status, body) = list(&session, token, query).await;
        assert_eq!(status, StatusCode::OK, "{query}: {body}");
    }
}

/// A pin the tenant's registry never accepted is a conflict, distinct from
/// a type it never declared: one names a vocabulary, the other a version.
#[tokio::test]
async fn a_revision_the_registry_never_accepted_is_refused_as_a_conflict() {
    let (fixture, session) = three_tenants("listing-pin", &["ent_mine_a"]).await;
    let (status, body) = list(
        &session,
        fixture.operator_token(),
        &format!("type={TYPE}&revision=9"),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["gate"], "surface");
}

#[tokio::test]
async fn a_listing_refuses_a_caller_it_cannot_place_in_a_tenant() {
    let (fixture, session) = three_tenants("listing-credential", &["ent_mine_a"]).await;
    for (token, expected) in [
        (None, StatusCode::UNAUTHORIZED),
        (Some("not-a-token"), StatusCode::UNAUTHORIZED),
        (Some(fixture.roleless_token()), StatusCode::FORBIDDEN),
    ] {
        // A query this route refuses: a caller it cannot place in a tenant
        // must be answered on the credential, never on the query string.
        let (status, body) = session.get(token, "/v1/objects?colour=red").await;
        assert_eq!(status, expected, "{body}");
    }
}
