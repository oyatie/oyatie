//! Acceptance tests for the tenant registration / lifecycle service.
//!
//! These drive the FULL HTTP surface end-to-end against the in-memory store,
//! exercising the REAL tenancy lifecycle core (the contract FSM inside the
//! usecase) — register lands a tenant in `Provisioning`, `:provision` drives
//! it to `Active` through the operation ledger, and the read surface reflects
//! every transition. No sockets are opened; the `axum::Router` is invoked via
//! `tower::ServiceExt::oneshot`.

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use tenancy_tenant_lifecycle_app::build_inmemory_router;
use tower::ServiceExt;

/// A distinct canonical-UUID idempotency key per call (AIP-155 client token).
fn key(n: u8) -> String {
    format!("00000000-0000-4000-8000-0000000000{n:02x}")
}

async fn body_json(body: Body) -> serde_json::Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null)
}

fn register_body(tenant_id: &str) -> serde_json::Value {
    serde_json::json!({
        "tenant_id": tenant_id,
        "display_name": "Acme Corporation",
        "isolation_posture": "pooled",
        "cell_id": "cell-001",
        "residency_zone": "kr-seoul"
    })
}

async fn register(app: &axum::Router, tenant_id: &str, idem: &str) -> axum::http::Response<Body> {
    app.clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/v1/tenants")
                .header("content-type", "application/json")
                .header("idempotency-key", idem)
                .body(Body::from(serde_json::to_vec(&register_body(tenant_id)).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn lifecycle(
    app: &axum::Router,
    tenant_id: &str,
    verb: &str,
    idem: &str,
) -> axum::http::Response<Body> {
    app.clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/v1/tenants/{tenant_id}/{verb}"))
                .header("idempotency-key", idem)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn get_state(app: &axum::Router, tenant_id: &str) -> (StatusCode, serde_json::Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/v1/tenants/{tenant_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    (status, body_json(resp.into_body()).await)
}

#[tokio::test]
async fn healthz_returns_200() {
    let app = build_inmemory_router();
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

/// The headline E2E flow: register -> read (Provisioning) -> provision ->
/// read (Active). Proves the delivery layer exercises the real FSM and the
/// state transition is observable on the read surface.
#[tokio::test]
async fn register_then_provision_drives_provisioning_to_active() {
    let app = build_inmemory_router();

    // Register: a brand-new tenant is born in Provisioning.
    let resp = register(&app, "acme", &key(1)).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let created = body_json(resp.into_body()).await;
    assert_eq!(created["tenant_id"], "acme");
    assert_eq!(created["state"], "provisioning");
    assert_eq!(created["cell_id"], "cell-001");

    // Read back: still Provisioning until provisioned.
    let (status, view) = get_state(&app, "acme").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(view["state"], "provisioning");

    // Provision: the real Activate transition drives it to Active.
    let resp = lifecycle(&app, "acme", "provision", &key(2)).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let provisioned = body_json(resp.into_body()).await;
    assert_eq!(provisioned["state"], "active");

    // Read confirms the persisted transition.
    let (status, view) = get_state(&app, "acme").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(view["state"], "active");
}

/// The full lifecycle FSM is reachable over HTTP: provision -> suspend ->
/// resume -> retire, each a real contract transition.
#[tokio::test]
async fn full_lifecycle_transitions_over_http() {
    let app = build_inmemory_router();
    assert_eq!(register(&app, "globex", &key(1)).await.status(), StatusCode::CREATED);

    assert_eq!(
        lifecycle(&app, "globex", "provision", &key(2)).await.status(),
        StatusCode::OK
    );
    let resp = lifecycle(&app, "globex", "suspend", &key(3)).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_json(resp.into_body()).await["state"], "suspended");

    let resp = lifecycle(&app, "globex", "resume", &key(4)).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_json(resp.into_body()).await["state"], "active");

    // Retire is terminal; the read surface hides the tombstone.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/v1/tenants/globex")
                .header("idempotency-key", key(5))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    let (status, _) = get_state(&app, "globex").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// Registration without an Idempotency-Key header is rejected.
#[tokio::test]
async fn register_requires_idempotency_key() {
    let app = build_inmemory_router();
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/v1/tenants")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&register_body("acme")).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

/// Re-registering with the SAME key + SAME body replays (idempotent), 200.
#[tokio::test]
async fn register_replay_is_idempotent() {
    let app = build_inmemory_router();
    assert_eq!(register(&app, "acme", &key(1)).await.status(), StatusCode::CREATED);
    let resp = register(&app, "acme", &key(1)).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_json(resp.into_body()).await["state"], "provisioning");
}

/// A NEW key onto an existing tenant id is a conflict (AlreadyExists).
#[tokio::test]
async fn register_existing_tenant_new_key_conflicts() {
    let app = build_inmemory_router();
    assert_eq!(register(&app, "acme", &key(1)).await.status(), StatusCode::CREATED);
    let resp = register(&app, "acme", &key(2)).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

/// Provisioning an unknown tenant is a 404.
#[tokio::test]
async fn provision_unknown_tenant_returns_404() {
    let app = build_inmemory_router();
    let resp = lifecycle(&app, "ghost", "provision", &key(1)).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

/// Suspending a tenant still in Provisioning is a precondition conflict (the
/// FSM forbids Provisioning -> Suspended directly).
#[tokio::test]
async fn suspend_before_provision_is_conflict() {
    let app = build_inmemory_router();
    assert_eq!(register(&app, "acme", &key(1)).await.status(), StatusCode::CREATED);
    let resp = lifecycle(&app, "acme", "suspend", &key(2)).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    // The failed transition left the tenant untouched.
    let (_, view) = get_state(&app, "acme").await;
    assert_eq!(view["state"], "provisioning");
}

/// Reading an unregistered tenant is a 404.
#[tokio::test]
async fn get_unknown_tenant_returns_404() {
    let app = build_inmemory_router();
    let (status, _) = get_state(&app, "nobody").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// The list surface enumerates registered tenants.
#[tokio::test]
async fn list_returns_registered_tenants() {
    let app = build_inmemory_router();
    assert_eq!(register(&app, "alpha", &key(1)).await.status(), StatusCode::CREATED);
    assert_eq!(register(&app, "bravo", &key(2)).await.status(), StatusCode::CREATED);

    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/v1/tenants")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let page = body_json(resp.into_body()).await;
    let ids: Vec<&str> = page["tenants"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["tenant_id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec!["alpha", "bravo"]);
}
