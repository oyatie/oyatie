// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Router-level tests for the Cedar policy publish REST edge.
//!
//! Each test drives the real axum router via `tower::ServiceExt::oneshot`,
//! asserting HTTP status + JSON body shape.  No business logic lives here;
//! the boundary-fn integration tests in `cedar_policy_publish_api.rs` cover
//! the domain rules end-to-end.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt as _;
use serde_json::{Value, json};
use tower::ServiceExt as _;

use iam_policy_cedar_api::rest::{CedarPolicyRestState, build_router};

// ── Test constants ──────────────────────────────────────────────────────────

const REQUEST_ID: &str = "req_rest_001";
const TENANT_ID: &str = "ten_platform";
const IDEMPOTENCY_KEY: &str = "idem_rest_001";
const PRINCIPAL_TENANT_ID: &str = "ten_platform";
const PRINCIPAL_ID: &str = "usr_platform_admin";
const DECISION_ID: &str = "authz_cedar_publish_001";
const POLICY_ID: &str = "pol_tenant_admin";
const VERSION: &str = "1.0.0";

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Build a fresh router with default (empty) state.
fn fresh_router() -> axum::Router {
    let state = Arc::new(CedarPolicyRestState::default());
    build_router(state)
}

/// POST to the publish route with the canonical test headers and body.
async fn post_publish(
    router: axum::Router,
    policy_id: &str,
    version: &str,
    body: Value,
) -> (StatusCode, Value) {
    post_publish_with_overrides(router, policy_id, version, body, &[]).await
}

/// POST with optional header overrides (name, value) that REPLACE the defaults.
/// A default header is omitted entirely when an override supplies the same name.
async fn post_publish_with_overrides(
    router: axum::Router,
    policy_id: &str,
    version: &str,
    body: Value,
    overrides: &[(&str, &str)],
) -> (StatusCode, Value) {
    // Build a lookup of which default names are being overridden.
    let override_names: std::collections::HashSet<&str> =
        overrides.iter().map(|(name, _)| *name).collect();

    // Default headers — only included when not overridden.
    let defaults: &[(&str, &str)] = &[
        ("x-request-id", REQUEST_ID),
        ("x-tenant-id", TENANT_ID),
        ("idempotency-key", IDEMPOTENCY_KEY),
        ("x-principal-tenant-id", PRINCIPAL_TENANT_ID),
        ("x-principal-id", PRINCIPAL_ID),
        ("x-authorization-decision-id", DECISION_ID),
        ("x-authorization-tenant-id", TENANT_ID),
        ("x-authorization-principal-id", PRINCIPAL_ID),
        ("x-authorization-surfaces", "cedar.policy.publish"),
    ];

    let mut builder = Request::builder()
        .method("POST")
        .uri(format!("/policies/{policy_id}/versions/{version}"))
        .header("content-type", "application/json");

    // Apply defaults not being overridden.
    for (name, value) in defaults {
        if !override_names.contains(*name) {
            builder = builder.header(*name, *value);
        }
    }
    // Apply overrides (only non-empty values are meaningful; empty strings
    // are intentionally omitted so the header is absent entirely).
    for (name, value) in overrides {
        if !value.is_empty() {
            builder = builder.header(*name, *value);
        }
    }

    let request = builder
        .body(Body::from(body.to_string()))
        .expect("request");
    let response = router.oneshot(request).await.expect("response");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let json: Value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, json)
}

/// Default valid request body.
fn valid_body(policy_id: &str, version: &str) -> Value {
    json!({
        "policy_id": policy_id,
        "version": version,
        "scope": { "kind": "tenant", "tenant_id": "ten_alpha" },
        "supersedes": null,
        "rules": [{
            "effect": "allow",
            "principal_role": "tenant-admin",
            "action": "tenant.settings.update",
            "resource_prefix": "tenant:",
            "required_attribute": { "key": "region", "value": "region-home" }
        }]
    })
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn publish_happy_path_returns_201_created() {
    let (status, body) = post_publish(
        fresh_router(),
        POLICY_ID,
        VERSION,
        valid_body(POLICY_ID, VERSION),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["data"]["policy_id"], POLICY_ID);
    assert_eq!(body["data"]["version"], VERSION);
    assert_eq!(body["data"]["scope"]["kind"], "tenant");
    assert_eq!(body["data"]["scope"]["tenant_id"], "ten_alpha");
    assert_eq!(body["data"]["rules"][0]["effect"], "allow");
    assert_eq!(body["data"]["schema_version"], 1);
    assert_eq!(body["metadata"]["request_id"], REQUEST_ID);
    assert_eq!(body["metadata"]["operator_tenant_id"], TENANT_ID);
    assert_eq!(body["metadata"]["principal_id"], PRINCIPAL_ID);
}

#[tokio::test]
async fn publish_idempotent_replay_returns_201_same_body() {
    // Use shared state so the second POST sees the ledger entry from the first.
    let state = Arc::new(CedarPolicyRestState::default());
    let router = build_router(state);

    let body = valid_body(POLICY_ID, VERSION);

    let (status1, body1) = post_publish(router.clone(), POLICY_ID, VERSION, body.clone()).await;
    let (status2, body2) = post_publish(router, POLICY_ID, VERSION, body).await;

    assert_eq!(status1, StatusCode::CREATED);
    assert_eq!(status2, StatusCode::CREATED);
    assert_eq!(body1["data"]["policy_id"], body2["data"]["policy_id"]);
    assert_eq!(body1["data"]["version"], body2["data"]["version"]);
}

#[tokio::test]
async fn publish_path_body_policy_id_mismatch_returns_400() {
    // Body has a different policy_id than path.
    let mut body = valid_body(POLICY_ID, VERSION);
    body["policy_id"] = json!("pol_other");

    let (status, resp_body) =
        post_publish(fresh_router(), POLICY_ID, VERSION, body).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        resp_body["error"]["code"],
        "CEDAR_POLICY_POLICY_PATH_BODY_MISMATCH"
    );
}

#[tokio::test]
async fn publish_missing_principal_id_returns_401() {
    let (status, resp_body) = post_publish_with_overrides(
        fresh_router(),
        POLICY_ID,
        VERSION,
        valid_body(POLICY_ID, VERSION),
        &[("x-principal-id", "")],
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        resp_body["error"]["code"],
        "CEDAR_POLICY_PRINCIPAL_ID_EMPTY"
    );
}

#[tokio::test]
async fn publish_authorization_denied_returns_403() {
    // Surface not in the allowed list.
    let (status, resp_body) = post_publish_with_overrides(
        fresh_router(),
        POLICY_ID,
        VERSION,
        valid_body(POLICY_ID, VERSION),
        &[("x-authorization-surfaces", "identity.user.upsert")],
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(
        resp_body["error"]["code"],
        "CEDAR_POLICY_AUTHORIZATION_DENIED"
    );
}

#[tokio::test]
async fn publish_duplicate_version_returns_409_conflict() {
    let state = Arc::new(CedarPolicyRestState::default());
    let router = build_router(state);
    let body = valid_body(POLICY_ID, VERSION);

    // First publish succeeds.
    let (status1, _) = post_publish(router.clone(), POLICY_ID, VERSION, body.clone()).await;
    assert_eq!(status1, StatusCode::CREATED);

    // Second publish of same version with a different idempotency key → conflict.
    let (status2, resp_body) = post_publish_with_overrides(
        router,
        POLICY_ID,
        VERSION,
        body,
        &[("idempotency-key", "idem_rest_002")],
    )
    .await;

    assert_eq!(status2, StatusCode::CONFLICT);
    assert_eq!(
        resp_body["error"]["code"],
        "CEDAR_POLICY_KERNEL_VERSION_ALREADY_EXISTS"
    );
}

#[tokio::test]
async fn publish_reused_idempotency_key_with_changed_body_returns_422() {
    let state = Arc::new(CedarPolicyRestState::default());
    let router = build_router(state);

    // First publish succeeds.
    let body = valid_body(POLICY_ID, VERSION);
    let (status1, _) = post_publish(router.clone(), POLICY_ID, VERSION, body).await;
    assert_eq!(status1, StatusCode::CREATED);

    // Same idempotency key but different body (changed action).
    let mut changed_body = valid_body(POLICY_ID, VERSION);
    changed_body["rules"][0]["action"] = json!("tenant.settings.read");

    let (status2, resp_body) =
        post_publish(router, POLICY_ID, VERSION, changed_body).await;

    assert_eq!(status2, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        resp_body["error"]["code"],
        "CEDAR_POLICY_IDEMPOTENCY_KEY_REUSED"
    );
}

#[tokio::test]
async fn publish_invalid_scope_kind_returns_400() {
    let mut body = valid_body(POLICY_ID, VERSION);
    body["scope"]["kind"] = json!("workspace");

    let (status, resp_body) =
        post_publish(fresh_router(), POLICY_ID, VERSION, body).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        resp_body["error"]["code"],
        "CEDAR_POLICY_SCOPE_KIND_INVALID"
    );
}

#[tokio::test]
async fn publish_global_scope_without_tenant_returns_201() {
    let mut body = valid_body("pol_global_reader", VERSION);
    body["policy_id"] = json!("pol_global_reader");
    body["scope"] = json!({ "kind": "global" });

    let (status, resp_body) =
        post_publish(fresh_router(), "pol_global_reader", VERSION, body).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp_body["data"]["scope"]["kind"], "global");
    assert!(resp_body["data"]["scope"]["tenant_id"].is_null());
}

#[tokio::test]
async fn publish_missing_request_id_header_returns_400() {
    let (status, resp_body) = post_publish_with_overrides(
        fresh_router(),
        POLICY_ID,
        VERSION,
        valid_body(POLICY_ID, VERSION),
        &[("x-request-id", "")],
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        resp_body["error"]["code"],
        "CEDAR_POLICY_REQUEST_ID_EMPTY"
    );
}
