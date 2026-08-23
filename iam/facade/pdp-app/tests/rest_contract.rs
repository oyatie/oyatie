//! REST contract suite for the iam PDP decision surface.
//!
//! Drives the axum router in-process (`tower::ServiceExt::oneshot`) over the
//! seed bundle. Covers the G004 doctrine matrix on the wire contract:
//!
//! - RBAC / ABAC / PBAC exemplars decide correctly (full-spectrum authz —
//!   the surface is PARC + entity slice, never an RBAC-only shape);
//! - deny-by-default and the structural cross-tenant forbid;
//! - refusal mapping (invalid request 400, unknown action 400, stale zookie
//!   pin 409) — refusals are NEVER 200s;
//! - default-deny on every route (unknown route -> 404, closed body schema
//!   -> 400);
//! - one attributable audit record per decision, none per refusal;
//! - decision-cache replay mints fresh decision ids (audit-chain integrity).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use axum::body::Body;
use axum::http::{Request as HttpRequest, StatusCode, header};
use http_body_util::BodyExt;
use tower::ServiceExt;

use shared_platform_contracts_kernel::pdp::AuthorizationRequest;

use common::{bob_read_link, entity_ref, entity_slice, request, seeded_state};

fn authorize_body(request: &AuthorizationRequest) -> serde_json::Value {
    serde_json::json!({
        "request": request,
        "entities": entity_slice().entities,
    })
}

async fn post_authorize(
    router: axum::Router,
    body: &serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let response = router
        .oneshot(
            HttpRequest::post("/v1/authorize")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value =
        serde_json::from_slice(&bytes).unwrap_or(serde_json::json!({"raw": "unparseable"}));
    (status, json)
}

async fn get_route(router: axum::Router, path: &str) -> (StatusCode, serde_json::Value) {
    let response = router
        .oneshot(HttpRequest::get(path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value =
        serde_json::from_slice(&bytes).unwrap_or(serde_json::json!({"raw": "unparseable"}));
    (status, json)
}

// ---------------------------------------------------------------- RBAC ----

#[tokio::test]
async fn rbac_group_admin_allow_is_attributable() {
    let (state, sink) = seeded_state(vec![]);
    let router = iam_pdp_app::rest::build_router(state);
    let body = authorize_body(&request(
        "req-rbac-allow",
        "acme",
        entity_ref("OyaPlatform::Principal", "alice"),
        "tenant.administer",
        entity_ref("OyaPlatform::Tenant", "acme"),
    ));
    let (status, json) = post_authorize(router, &body).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["decision"], "allow");
    assert_eq!(json["request_id"], "req-rbac-allow");
    assert_eq!(json["policy_version"], "psv-000001");
    assert!(
        json["determining_policy_ids"]
            .as_array()
            .unwrap()
            .iter()
            .any(|id| id == "rbac-tenant-admin-group"),
        "allow must name its permit policy: {json}"
    );
    assert!(!json["decision_id"].as_str().unwrap().is_empty());
    let records = sink.records();
    assert_eq!(records.len(), 1, "exactly one audit record per decision");
    assert!(!records[0].cache_hit);
}

#[tokio::test]
async fn deny_by_default_is_a_decision_not_an_error() {
    let (state, sink) = seeded_state(vec![]);
    let router = iam_pdp_app::rest::build_router(state);
    // bob has no group, no step-up, no PBAC link: nothing permits. acme-doc-2
    // is non-restricted, so this is a CLEAN deny-by-omission (no permit AND no
    // forbid fires), keeping determining_policy_ids empty. (On the restricted
    // acme-doc-1 the step-up forbid would fire and name itself — covered by the
    // ABAC test below.)
    let body = authorize_body(&request(
        "req-default-deny",
        "acme",
        entity_ref("OyaPlatform::Principal", "bob"),
        "resource.read",
        entity_ref("OyaPlatform::TenantResource", "acme-doc-2"),
    ));
    let (status, json) = post_authorize(router, &body).await;
    assert_eq!(status, StatusCode::OK, "a deny is a decision, not an error");
    assert_eq!(json["decision"], "deny");
    assert_eq!(json["determining_policy_ids"], serde_json::json!([]));
    assert_eq!(sink.records().len(), 1, "denies are audited too");
}

// ---------------------------------------------------------------- ABAC ----

#[tokio::test]
async fn abac_step_up_attribute_gates_restricted_read() {
    let (state, _sink) = seeded_state(vec![]);
    let router = iam_pdp_app::rest::build_router(state);
    // alice carries step_up_class "a": restricted read allowed via ABAC.
    let body = authorize_body(&request(
        "req-abac-allow",
        "acme",
        entity_ref("OyaPlatform::Principal", "alice"),
        "resource.read",
        entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
    ));
    let (status, json) = post_authorize(router.clone(), &body).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["decision"], "allow");
    assert!(
        json["determining_policy_ids"]
            .as_array()
            .unwrap()
            .iter()
            .any(|id| id == "abac-step-up-restricted-read"),
        "{json}"
    );
    // bob has NO step_up_class attribute: the `has` guard drops him through
    // to deny-by-default.
    let body = authorize_body(&request(
        "req-abac-deny",
        "acme",
        entity_ref("OyaPlatform::Principal", "bob"),
        "resource.read",
        entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
    ));
    let (status, json) = post_authorize(router, &body).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["decision"], "deny");
}

// ---------------------------------------------------------------- PBAC ----

#[tokio::test]
async fn pbac_template_link_grants_scoped_read() {
    let (state, _sink) = seeded_state(vec![bob_read_link()]);
    let router = iam_pdp_app::rest::build_router(state);
    let body = authorize_body(&request(
        "req-pbac-allow",
        "acme",
        entity_ref("OyaPlatform::Principal", "bob"),
        "resource.read",
        entity_ref("OyaPlatform::TenantResource", "acme-doc-2"),
    ));
    let (status, json) = post_authorize(router, &body).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["decision"], "allow");
    assert!(
        json["determining_policy_ids"]
            .as_array()
            .unwrap()
            .iter()
            .any(|id| id == "pbac-link-bob-acme-doc-2"),
        "PBAC allow must name the template link: {json}"
    );
}

// ------------------------------------------------- structural forbid ----

#[tokio::test]
async fn structural_forbid_neutralizes_cross_tenant_group_pollution() {
    let (state, _sink) = seeded_state(vec![]);
    let router = iam_pdp_app::rest::build_router(state);
    // mallory (globex) is mis-joined into acme's admin group; the structural
    // tenant-isolation forbid must still deny.
    let body = authorize_body(&request(
        "req-cross-tenant",
        "globex",
        entity_ref("OyaPlatform::Principal", "mallory"),
        "tenant.administer",
        entity_ref("OyaPlatform::Tenant", "acme"),
    ));
    let (status, json) = post_authorize(router, &body).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["decision"], "deny");
    assert!(
        json["determining_policy_ids"]
            .as_array()
            .unwrap()
            .iter()
            .any(|id| id == "structural-tenant-isolation"),
        "the structural forbid must be the determining policy: {json}"
    );
}

// ------------------------------------------------------------- refusals ----

#[tokio::test]
async fn unknown_action_is_refused_not_decided() {
    let (state, sink) = seeded_state(vec![]);
    let router = iam_pdp_app::rest::build_router(state);
    let body = authorize_body(&request(
        "req-unknown-action",
        "acme",
        entity_ref("OyaPlatform::Principal", "alice"),
        "resource.delete",
        entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
    ));
    let (status, json) = post_authorize(router, &body).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json["error_code"], "unknown_action");
    assert!(sink.is_empty(), "refusals never enter the decision audit");
}

#[tokio::test]
async fn invalid_request_is_refused_with_machine_code() {
    let (state, _sink) = seeded_state(vec![]);
    let router = iam_pdp_app::rest::build_router(state);
    let mut bad = request(
        "req-invalid",
        "acme",
        entity_ref("OyaPlatform::Principal", "alice"),
        "resource.read",
        entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
    );
    bad.tenant_id = String::new();
    let body = authorize_body(&bad);
    let (status, json) = post_authorize(router, &body).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json["error_code"], "invalid_request");
}

#[tokio::test]
async fn stale_zookie_pin_is_conflict_never_a_stale_answer() {
    let (state, sink) = seeded_state(vec![]);
    let router = iam_pdp_app::rest::build_router(state);
    let mut pinned = request(
        "req-stale-pin",
        "acme",
        entity_ref("OyaPlatform::Principal", "alice"),
        "resource.read",
        entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
    );
    pinned.min_policy_version =
        Some(shared_platform_contracts_kernel::pdp::PolicyVersion::new("psv-000099").unwrap());
    let body = authorize_body(&pinned);
    let (status, json) = post_authorize(router, &body).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(json["error_code"], "stale_policy_version");
    assert!(sink.is_empty());
}

#[tokio::test]
async fn closed_body_schema_rejects_unknown_fields() {
    let (state, _sink) = seeded_state(vec![]);
    let router = iam_pdp_app::rest::build_router(state);
    let mut body = authorize_body(&request(
        "req-smuggle",
        "acme",
        entity_ref("OyaPlatform::Principal", "alice"),
        "resource.read",
        entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
    ));
    body["smuggled"] = serde_json::json!(true);
    let (status, _json) = post_authorize(router, &body).await;
    // axum maps serde data errors (deny_unknown_fields) to 422; the contract
    // that matters is fail-closed: any non-200 is a refusal, never an allow.
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn malformed_body_is_refused() {
    let (state, _sink) = seeded_state(vec![]);
    let router = iam_pdp_app::rest::build_router(state);
    let response = router
        .oneshot(
            HttpRequest::post("/v1/authorize")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("{ not json"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------- route surface ----

#[tokio::test]
async fn unknown_routes_are_default_denied() {
    let (state, _sink) = seeded_state(vec![]);
    let router = iam_pdp_app::rest::build_router(state);
    let (status, json) = get_route(router.clone(), "/v1/policies").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(json["error_code"], "unknown_route");
    // The decision route only accepts POST.
    let (status, _json) = get_route(router, "/v1/authorize").await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn health_and_readiness_report_the_loaded_bundle() {
    let (state, _sink) = seeded_state(vec![]);
    let router = iam_pdp_app::rest::build_router(state);
    let (status, json) = get_route(router.clone(), "/healthz").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "ok");
    let (status, json) = get_route(router, "/readyz").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "ready");
    assert_eq!(json["policy_version"], "psv-000001");
}

// ------------------------------------------------------------ cache/audit ----

#[tokio::test]
async fn cache_replay_mints_fresh_decision_ids_and_audits_both() {
    let (state, sink) = seeded_state(vec![]);
    let router = iam_pdp_app::rest::build_router(state);
    let body = authorize_body(&request(
        "req-cache-1",
        "acme",
        entity_ref("OyaPlatform::Principal", "alice"),
        "resource.read",
        entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
    ));
    let (status_a, json_a) = post_authorize(router.clone(), &body).await;
    let (status_b, json_b) = post_authorize(router, &body).await;
    assert_eq!(status_a, StatusCode::OK);
    assert_eq!(status_b, StatusCode::OK);
    assert_eq!(json_a["decision"], json_b["decision"]);
    assert_ne!(
        json_a["decision_id"], json_b["decision_id"],
        "every decision gets a fresh id, cached replays included"
    );
    let records = sink.records();
    assert_eq!(records.len(), 2, "one audit record per decision");
    assert!(!records[0].cache_hit);
    assert!(records[1].cache_hit, "second decision is a cache replay");
}
