use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::http::StatusCode;
use foundry_ontology_app::router_with_user_sessions;

use crate::facade_support::{Fixture, Session, scrape, value_of, write_a_record};
use crate::human_support::{FakeSessions, body, cookie, instrument, request, view};
use crate::point_support::{ChangingStore, declare, point, send};

#[tokio::test]
async fn malformed_point_queries_refuse_after_auth_and_before_the_store() {
    let fixture = Fixture::new("human-point-grammar");
    let mut state = fixture.state();
    let reads = ChangingStore::install(&mut state, None, std::time::Duration::ZERO);
    let sessions = FakeSessions::new(Ok(view()));
    let (router, _, pdp) = instrument(state, sessions.clone());
    let mut queries = vec![
        "",
        "revision=1",
        "type=ety_record",
        "type=ety_record&revision=",
        "type=ety_record&revision=1&revision=1",
        "type=ety_record&type=ety_record&revision=1",
        "type=ety_record&revision=1&unknown=x",
        "t%79pe=ety_record&revision=1",
        "type=ety_record&%72evision=1",
        "type=ety_record&revision=%31",
        "type=ety_record&revision=+1",
        "type=ety_record&revision=%2B1",
        "type=ety_record&revision=-1",
        "type=ety_record&revision=4294967296",
        "type=ety_record&revision=1.0",
        "type=ety_record&&revision=1",
        "type=ety_record&revision=1&",
        "type=ety_%&revision=1",
        "type=ety_%ZZ&revision=1",
        "type=ety_%FF&revision=1",
        "type=ety_%C0%AF&revision=1",
        "type=ety_%c3%a9&revision=1",
        "type=ety_%20inside&revision=1",
        "type=+ety_record&revision=1",
        "type=ety_record+&revision=1",
        "type=ety_%00&revision=1",
        "type=ety_%0A&revision=1",
        "type=ety_&revision=1",
        "type=ent_record&revision=1",
        "type=ety_record;revision=1",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    queries.push(format!("type=ety_{}&revision=1", "a".repeat(253)));
    queries.push(format!("type=ety_record&revision={}", "1".repeat(4096)));
    for query in &queries {
        let path = format!("/v1/tenants/ten_acme/objects/ent_alpha?{query}");
        body(request(&router, &path, &[]).await, StatusCode::UNAUTHORIZED).await;
        let refused = body(
            request(&router, &path, &[&cookie()]).await,
            StatusCode::BAD_REQUEST,
        )
        .await;
        assert_eq!(refused["gate"], "surface", "{query}");
    }
    for (object, expected) in [
        ("ent_".to_owned(), StatusCode::BAD_REQUEST),
        ("ety_record".to_owned(), StatusCode::BAD_REQUEST),
        ("ent_padded%20".to_owned(), StatusCode::FORBIDDEN),
        ("ent_control%00".to_owned(), StatusCode::BAD_REQUEST),
        (format!("ent_{}", "a".repeat(253)), StatusCode::BAD_REQUEST),
    ] {
        let response = request(&router, &point("ety_record", &object, 1), &[&cookie()]).await;
        assert_eq!(response.status(), expected, "{object}");
        body(response, expected).await;
    }
    let mut revoked = view();
    revoked.roles.clear();
    *sessions.answer.lock().unwrap() = Ok(revoked);
    body(
        request(
            &router,
            "/v1/tenants/ten_acme/objects/ent_alpha?revision=bad",
            &[&cookie()],
        )
        .await,
        StatusCode::FORBIDDEN,
    )
    .await;
    assert_eq!(reads.load(Ordering::SeqCst), 0);
    assert_eq!(sessions.calls.load(Ordering::SeqCst), queries.len() + 6);
    assert_eq!(pdp.calls.load(Ordering::SeqCst), queries.len() + 6);
}

#[tokio::test]
async fn canonical_form_identifiers_keep_their_exact_type_identity() {
    for (index, kind) in [
        "ety_plus+literal".to_owned(),
        "ety_percent%literal".to_owned(),
        "ety_internal space".to_owned(),
        "ety_unicode_é한".to_owned(),
        "ety_bom\u{feff}".to_owned(),
        format!("ety_{}", "é".repeat(126)),
    ]
    .into_iter()
    .enumerate()
    {
        let fixture = Fixture::new(&format!("human-point-form-{index}"));
        let mut state = fixture.state();
        declare(&mut state, &kind, "aty_other", "other_name");
        let state = Arc::new(state);
        send(&state, &kind, "aty_other", "other_name", "Exact", "first").await;
        let router = router_with_user_sessions(state, FakeSessions::new(Ok(view())));
        let expected = body(
            request(&router, &point(&kind, "ent_alpha", 1), &[&cookie()]).await,
            StatusCode::OK,
        )
        .await;
        assert_eq!(expected["properties"]["other_name"]["value"], "Exact");
        let normal = point(&kind, "ent_alpha", 1);
        let (_, query) = normal.split_once('?').unwrap();
        let (first, second) = query.split_once('&').unwrap();
        let reordered = format!("/v1/tenants/ten_acme/objects/ent_alpha?{second}&{first}");
        assert_eq!(
            body(
                request(&router, &reordered, &[&cookie()]).await,
                StatusCode::OK
            )
            .await,
            expected
        );
    }
}

#[tokio::test]
async fn point_refusals_have_one_counter_and_preserve_the_operator_query_parser() {
    let fixture = Fixture::new("human-point-counters");
    write_a_record(&fixture, "ent_alpha", "idem_counters").await;
    let state = Arc::new(fixture.state());
    let operator = Session::from_shared(state.clone());
    let router = router_with_user_sessions(state, FakeSessions::new(Ok(view())));
    for (path, status) in [
        (
            "/v1/tenants/ten_acme/objects/ent_alpha?revision=1",
            StatusCode::BAD_REQUEST,
        ),
        (&point("ety_other", "ent_alpha", 1), StatusCode::NOT_FOUND),
        (&point("ety_record", "ent_alpha", 9), StatusCode::CONFLICT),
    ] {
        let before = scrape(&operator).await;
        body(request(&router, path, &[&cookie()]).await, status).await;
        let after = scrape(&operator).await;
        assert_eq!(
            value_of(&after, "foundry_read_refused_total"),
            value_of(&before, "foundry_read_refused_total") + 1
        );
        assert_eq!(
            value_of(&after, "foundry_read_served_total"),
            value_of(&before, "foundry_read_served_total")
        );
    }
    for query in [
        "revision=1",
        "revision=1&ignored=x",
        "type=ety_other&revision=1",
    ] {
        assert_eq!(
            operator
                .get(
                    Some(fixture.operator_token()),
                    &format!("/v1/objects/ent_alpha?{query}")
                )
                .await
                .0,
            StatusCode::OK
        );
    }
}
