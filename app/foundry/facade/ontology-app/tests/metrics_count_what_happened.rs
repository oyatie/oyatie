mod failing_store;
#[path = "facade_support/mod.rs"]
mod support;

use axum::http::StatusCode;
use failing_store::AlwaysFailingStore;
use support::{Fixture, Session, TENANT, WRITE_BODY as WRITE, scrape, value_of};

#[tokio::test]
async fn an_answered_read_increments_served() {
    let fixture = Fixture::new("metrics-read-served");
    let session = fixture.session();
    let (write, _) = session.post(Some(fixture.operator_token()), WRITE).await;
    assert_eq!(write, StatusCode::OK);
    let (status, _) = session
        .get(
            Some(fixture.operator_token()),
            "/v1/objects/ent_alpha?revision=1",
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let body = scrape(&session).await;
    assert_eq!(value_of(&body, "foundry_read_served_total"), 1);
    assert_eq!(value_of(&body, "foundry_read_refused_total"), 0);
}

/// Each refusing site, pinned INDIVIDUALLY by a delta.
///
/// An aggregate total conflates sites: it passes as long as the sum is
/// right, so one site counting twice hides another counting never. A
/// per-case delta localizes, which means deleting any single counting call
/// fails a case that names its site.
///
/// Not every case has a site to itself: the cross-tenant cases and the
/// unserved-tenant fixture below both land on the roster site, because the
/// roster is consulted before the policy decision point and refuses a
/// credential whose tenant it does not hold. Deleting the roster-site call
/// fails both tests, each at its own case.
async fn assert_read_refusal_delta(
    session: &Session,
    label: &str,
    request: impl AsRef<str>,
    token: Option<&str>,
    expect: StatusCode,
) {
    let before = value_of(&scrape(session).await, "foundry_read_refused_total");
    let (status, _) = session.get(token, request.as_ref()).await;
    assert_eq!(status, expect, "{label}");
    let after = value_of(&scrape(session).await, "foundry_read_refused_total");
    assert_eq!(after, before + 1, "{label}: must count exactly one refusal");
}

#[tokio::test]
async fn each_read_refusal_site_counts_exactly_once() {
    let fixture = Fixture::new("metrics-read-refused");
    let session = fixture.session();
    // The unretained-revision refusal needs a BINDING to reach: without one,
    // that request takes the unknown-object site instead and the two cases
    // silently pin the same call.
    let (write, _) = session.post(Some(fixture.operator_token()), WRITE).await;
    assert_eq!(write, StatusCode::OK);
    let path = "/v1/objects/ent_alpha?revision=1";
    assert_read_refusal_delta(
        &session,
        "no credential",
        path,
        None,
        StatusCode::UNAUTHORIZED,
    )
    .await;
    assert_read_refusal_delta(
        &session,
        "unrecognised credential",
        path,
        Some("nope"),
        StatusCode::UNAUTHORIZED,
    )
    .await;
    assert_read_refusal_delta(
        &session,
        "policy denial",
        path,
        Some(fixture.roleless_token()),
        StatusCode::FORBIDDEN,
    )
    .await;
    assert_read_refusal_delta(
        &session,
        "cross-tenant (the roster refuses an unserved tenant)",
        path,
        Some(fixture.foreign_token()),
        StatusCode::FORBIDDEN,
    )
    .await;
    assert_read_refusal_delta(
        &session,
        "unusable revision pin",
        "/v1/objects/ent_alpha?revision=abc",
        Some(fixture.operator_token()),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_read_refusal_delta(
        &session,
        "unknown object",
        "/v1/objects/ent_ghost?revision=1",
        Some(fixture.operator_token()),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_read_refusal_delta(
        &session,
        "unretained revision",
        "/v1/objects/ent_alpha?revision=9",
        Some(fixture.operator_token()),
        StatusCode::CONFLICT,
    )
    .await;
    for (label, path, status) in [
        (
            "listing query refusal",
            "/v1/objects?type=ety_record",
            StatusCode::BAD_REQUEST,
        ),
        (
            "listing undeclared type",
            "/v1/objects?type=ety_absent&revision=1",
            StatusCode::BAD_REQUEST,
        ),
        (
            "listing unretained revision",
            "/v1/objects?type=ety_record&revision=9",
            StatusCode::CONFLICT,
        ),
    ] {
        assert_read_refusal_delta(
            &session,
            label,
            path,
            Some(fixture.operator_token()),
            status,
        )
        .await;
    }
    // The store sites need their own process: the store is unreadable from
    // boot, which the sites above would not survive.
    let mut state = fixture.state();
    state
        .tenants
        .get_mut(TENANT)
        .expect("served")
        .get_mut()
        .projection_store = Box::new(AlwaysFailingStore {
        detail: "the store is gone",
    });
    let wedged = Session::from_state(state);
    for (label, path) in [
        ("unreadable projection store", path),
        (
            "unreadable store under a listing",
            "/v1/objects?type=ety_record&revision=1",
        ),
    ] {
        assert_read_refusal_delta(
            &wedged,
            label,
            path,
            Some(fixture.operator_token()),
            StatusCode::SERVICE_UNAVAILABLE,
        )
        .await;
    }
}

/// The unserved-tenant refusal on BOTH surfaces under a roster that does
/// not hold the operator's own tenant: the same site the cross-tenant cases
/// reach, pinned here from the other direction (a well-formed, policy-clean
/// credential against a roster varied away from it).
#[tokio::test]
async fn the_unserved_tenant_refusal_counts() {
    let fixture = Fixture::new("metrics-unserved-tenant");
    let session = fixture.unserved_session();
    assert_read_refusal_delta(
        &session,
        "operator's tenant is not in the served roster",
        "/v1/objects/ent_alpha?revision=1",
        Some(fixture.operator_token()),
        StatusCode::FORBIDDEN,
    )
    .await;

    let before = value_of(
        &scrape(&session).await,
        "foundry_action_submit_refused_total",
    );
    let (status, _) = session.post(Some(fixture.operator_token()), WRITE).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let after = value_of(
        &scrape(&session).await,
        "foundry_action_submit_refused_total",
    );
    assert_eq!(
        after,
        before + 1,
        "an unserved tenant must count on submit too"
    );
}

#[tokio::test]
async fn each_read_serving_route_counts_exactly_once() {
    let fixture = Fixture::new("metrics-read-routes");
    let session = fixture.session();
    let (write, _) = session.post(Some(fixture.operator_token()), WRITE).await;
    assert_eq!(write, StatusCode::OK);
    for (label, path) in [
        ("object", "/v1/objects/ent_alpha?revision=1"),
        ("history", "/v1/objects/ent_alpha/history"),
        ("audit", "/v1/audit"),
        ("types", "/v1/types"),
        ("object listing", "/v1/objects?type=ety_record&revision=1"),
    ] {
        let before = value_of(&scrape(&session).await, "foundry_read_served_total");
        let (status, _) = session.get(Some(fixture.operator_token()), path).await;
        assert_eq!(status, StatusCode::OK, "{label}");
        let after = value_of(&scrape(&session).await, "foundry_read_served_total");
        assert_eq!(after, before + 1, "{label}: a served read must count once");
    }
}

#[tokio::test]
async fn the_gauges_report_the_state_they_name() {
    let fixture = Fixture::new("metrics-gauges");
    let session = fixture.session();
    let body = scrape(&session).await;
    assert_eq!(value_of(&body, "foundry_served_tenants"), 1);
    assert_eq!(value_of(&body, "foundry_projection_lag"), 0);
    assert_eq!(value_of(&body, "foundry_poisoned_entries"), 0);
}
