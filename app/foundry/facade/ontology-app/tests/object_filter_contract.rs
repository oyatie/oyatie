//! The filter half of `GET /v1/objects`: what a filter selects, and that it
//! selects within the reading tenant only. The grammar's own edges are next
//! door in `object_filter_grammar`.

mod filter_support;

use axum::http::StatusCode;
use filter_support::{LAST_TENANT, TYPE, list, refs, three_tenants, write};
use serde_json::Value;

#[tokio::test]
async fn an_equality_filter_selects_within_the_reading_tenant() {
    let (fixture, session) = three_tenants("filter-equality").await;
    let (status, page) = list(
        &session,
        fixture.operator_token(),
        &format!("type={TYPE}&revision=1&property=name&equals=text:Ada"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{page}");
    assert_eq!(refs(&page), vec!["ent_mine_a"]);
    assert_eq!(page["objects"][0]["properties"]["name"]["value"], "Ada");

    // The value both neighbours carry, asked for by the middle tenant: the
    // objects exist and are named exactly this, in the other two tenants.
    let (status, leaked) = list(
        &session,
        fixture.operator_token(),
        &format!("type={TYPE}&revision=1&property=name&equals=text:{LAST_TENANT}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{leaked}");
    assert_eq!(
        refs(&leaked),
        Vec::<String>::new(),
        "a neighbour's value selects none of the reader's objects"
    );
}

/// A filter that matches nothing is an empty page; the unfiltered page is
/// the control on what was there to match.
#[tokio::test]
async fn a_filter_matching_nothing_is_an_empty_page_not_a_refusal() {
    let (fixture, session) = three_tenants("filter-empty").await;
    let (status, none) = list(
        &session,
        fixture.operator_token(),
        &format!("type={TYPE}&revision=1&property=name&equals=text:Nobody"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{none}");
    assert_eq!(refs(&none), Vec::<String>::new());
    assert_eq!(none["next"], Value::Null);

    let (status, all) = list(
        &session,
        fixture.operator_token(),
        &format!("type={TYPE}&revision=1"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{all}");
    assert_eq!(refs(&all), vec!["ent_mine_a", "ent_mine_b"]);
}

/// A filter and a cursor compose: the filter decides membership, the cursor
/// decides where the page resumes on the same keyset a listing uses, and `next`
/// means more MATCHES remain — which is where a filtered page stops behaving
/// like an unfiltered one.
#[tokio::test]
async fn a_filter_pages_on_the_listing_cursor_and_ends_at_its_last_match() {
    let (fixture, session) = three_tenants("filter-paging").await;
    let token = fixture.operator_token();
    // `ent_early_*` in the middle tenant too, all named "Ada", so the
    // filter has three matches to page through.
    write(&session, token, "ent_early_a", "Ada").await;
    write(&session, token, "ent_early_b", "Ada").await;

    let filter = format!("type={TYPE}&revision=1&property=name&equals=text:Ada");
    let (status, first) = list(&session, token, &format!("{filter}&limit=2")).await;
    assert_eq!(status, StatusCode::OK, "{first}");
    assert_eq!(refs(&first), vec!["ent_early_a", "ent_early_b"]);
    let cursor = first["next"]
        .as_str()
        .expect("a full page carries a cursor");
    assert_eq!(cursor, "ent_early_b");

    let (status, second) = list(
        &session,
        token,
        &format!("{filter}&limit=2&cursor={cursor}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{second}");
    assert_eq!(
        refs(&second),
        vec!["ent_mine_a"],
        "ent_mine_b is named Grace and is filtered out"
    );
    assert_eq!(second["next"], Value::Null);

    // A FULL page whose last row is the last match: the filtered half of the
    // `next` contract, which an unfiltered page cannot witness. The page is
    // exactly its limit and `ent_mine_b` remains past it in the store, so a
    // cursor here would promise a match that does not exist.
    let (status, exact) = list(&session, token, &format!("{filter}&limit=3")).await;
    assert_eq!(status, StatusCode::OK, "{exact}");
    assert_eq!(
        refs(&exact),
        vec!["ent_early_a", "ent_early_b", "ent_mine_a"]
    );
    assert_eq!(exact["next"], Value::Null);
}

/// The filter is parsed AFTER the credential: the query here is one the
/// grammar refuses, so a caller who cannot be placed in a tenant is answered
/// about their credential and never about the filter's shape.
#[tokio::test]
async fn a_filter_refuses_a_caller_it_cannot_place_in_a_tenant() {
    let (fixture, session) = three_tenants("filter-credential").await;
    let query = format!("/v1/objects?type={TYPE}&revision=1&property=name&from=int:1");
    for (token, expected) in [
        (None, StatusCode::UNAUTHORIZED),
        (Some("not-a-token"), StatusCode::UNAUTHORIZED),
        (Some(fixture.roleless_token()), StatusCode::FORBIDDEN),
    ] {
        let (status, body) = session.get(token, &query).await;
        assert_eq!(status, expected, "{body}");
    }
}
