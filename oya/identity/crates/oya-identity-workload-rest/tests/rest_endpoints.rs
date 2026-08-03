//! End-to-end tests for the workload-identity REST surface.
//!
//! Each test drives the real axum router via `tower::ServiceExt::oneshot` with
//! a genuinely-signed ES256 workload JWT (minted with `ring`), exercising the
//! full validate -> resolve -> authorize hot path through the REAL adapters and
//! asserting the fail-closed PEP status mapping + one-audit-record-per-call
//! invariant (PRD §1.2/§3.4/§3.5/§5, AC-W-13).
//!
//! Fixtures are shared with `grpc_authorize_deny.rs` via `tests/common/mod.rs`
//! so both surfaces provably exercise one shared setup with identical JWKS,
//! Cedar policies, and provisioned principal.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt as _;
use serde_json::{Value, json};
use tower::ServiceExt as _;

use oya_identity_oidc_issuer_kernel::{
    Algorithm as IssuerAlgorithm, SigningKey, build_jwks as build_issuer_jwks,
};
use oya_identity_workload_app::{
    InMemoryRevocationDenylist, InMemoryWorkloadPrincipalRepository, RepositoryError,
    RevocationDenylist, WorkloadPrincipalRepository, activate, provision,
};
use oya_identity_workload_authz_cedar_adapter::CedarWorkloadAuthorizer;
use oya_identity_workload_domain::{WorkloadId, WorkloadPrincipal};
use oya_identity_workload_oidc_adapter::{Jwk, JwkMaterial, Jwks, ValidationConfig};
use oya_identity_workload_rest::{
    AuditEvent, InMemoryAuditSink, SharedState, WorkloadAuthzState, build_router,
};

use common::{
    AUDIENCE, FailingRepository, ISSUER, NOW, mint_token, permit_authorizer, provisioned_state,
    provisioned_state_with_jwks,
};

async fn post_json(router: axum::Router, path: &str, body: Value) -> (StatusCode, Value) {
    let request = Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json")
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

fn verifier_jwks_from_issuer_publication(jwk: &Jwk) -> Jwks {
    let JwkMaterial::EcP256 { x, y } = &jwk.material else {
        panic!("fixture mints ES256/P-256 only");
    };
    let mut components = BTreeMap::new();
    components.insert("crv".to_owned(), "P-256".to_owned());
    components.insert("x".to_owned(), x.clone());
    components.insert("y".to_owned(), y.clone());

    let mut signing_key =
        SigningKey::provision(&jwk.kid, IssuerAlgorithm::Es256, components).expect("signing key");
    signing_key.activate(NOW).expect("activate signing key");
    let issuer_jwks = build_issuer_jwks(&[signing_key]);
    let keys: Vec<Value> = issuer_jwks
        .keys()
        .iter()
        .map(|key| {
            let mut value = serde_json::Map::new();
            value.insert("kid".to_owned(), json!(key.kid));
            value.insert("kty".to_owned(), json!(key.kty));
            value.insert("alg".to_owned(), json!(key.alg));
            value.insert("use".to_owned(), json!(key.key_use));
            for (name, component) in &key.public_components {
                value.insert(name.clone(), json!(component));
            }
            Value::Object(value)
        })
        .collect();
    let issuer_jwks_json = json!({ "keys": keys }).to_string();
    Jwks::from_jwks_json(&issuer_jwks_json).expect("issuer JWKS parses for verifier")
}

#[tokio::test]
async fn issuer_published_es256_jwks_validates_offline_and_policy_deny_is_403() {
    let minted = mint_token();
    // The verifier receives only the issuer-published JWKS document. No
    // introspection/token-status callback is configured or needed on the hot path.
    let verifier_jwks = verifier_jwks_from_issuer_publication(&minted.jwk);
    let state = provisioned_state_with_jwks(verifier_jwks);
    let router = build_router(state);

    let (validate_status, validate_body) = post_json(
        router.clone(),
        "/tokens/validate",
        json!({ "token": minted.token }),
    )
    .await;
    assert_eq!(validate_status, StatusCode::OK);
    assert_eq!(validate_body["tenantId"], "ten_acme");
    assert_eq!(validate_body["workloadId"], "wl_secrets_sync");

    let (authorize_status, authorize_body) = post_json(
        router,
        "/authorize-with-token",
        json!({
            "token": minted.token,
            "action": "cloud.kms.Encrypt",
            "resource": { "resourceType": "Secret", "resourceId": "db-password" }
        }),
    )
    .await;
    assert_eq!(authorize_status, StatusCode::FORBIDDEN);
    assert_eq!(authorize_body["effect"], "DENY");
    assert_eq!(authorize_body["reason"]["kind"], "defaultDeny");
}

#[tokio::test]
async fn authorize_with_token_permit_is_200_allow_and_audits() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let audit = state.audit().clone();
    let router = build_router(state);

    let (status, body) = post_json(
        router,
        "/authorize-with-token",
        json!({
            "token": minted.token,
            "action": "cloud.kms.Decrypt",
            "resource": { "resourceType": "Secret", "resourceId": "db-password" }
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["effect"], "ALLOW");
    // Exactly one authorize audit record, marked allow.
    assert_eq!(audit.len(), 1);
    let record = &audit.records()[0];
    assert_eq!(record.event(), AuditEvent::Authorize);
    assert_eq!(record.outcome(), "allow");
    assert_eq!(record.workload_id(), Some("wl_secrets_sync"));
}

#[tokio::test]
async fn authorize_with_token_policy_deny_is_403_forbidden() {
    let minted = mint_token();
    // Empty authorizer -> default-deny.
    let mut repo = InMemoryWorkloadPrincipalRepository::new();
    provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms").expect("provision");
    activate(&mut repo, &WorkloadId::new("wl_secrets_sync").unwrap()).expect("activate");
    let state: SharedState<_, _, _, _> = Arc::new(WorkloadAuthzState::with_clock(
        repo,
        InMemoryRevocationDenylist::new(),
        CedarWorkloadAuthorizer::new(),
        Jwks::new().add_key(minted.jwk.clone()),
        ValidationConfig::new(ISSUER, AUDIENCE),
        InMemoryAuditSink::new(),
        || NOW,
    ));
    let router = build_router(state);

    let (status, body) = post_json(
        router,
        "/authorize-with-token",
        json!({
            "token": minted.token,
            "action": "cloud.kms.Decrypt",
            "resource": { "resourceType": "Secret", "resourceId": "db-password" }
        }),
    )
    .await;

    // A deny is a 403 — NEVER a 404.
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["effect"], "DENY");
    assert_eq!(body["reason"]["kind"], "defaultDeny");
}

#[tokio::test]
async fn invalid_token_on_authorize_is_422() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let audit = state.audit().clone();
    let router = build_router(state);

    let (status, body) = post_json(
        router,
        "/authorize-with-token",
        json!({
            "token": "not-a-valid-jwt",
            "action": "cloud.kms.Decrypt",
            "resource": { "resourceType": "Secret", "resourceId": "db-password" }
        }),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "TOKEN_INVALID");
    // Still emits exactly one audit record (token-rejected).
    assert_eq!(audit.len(), 1);
    assert_eq!(audit.records()[0].outcome(), "token-rejected");
}

#[tokio::test]
async fn unknown_subject_is_403_not_404() {
    let minted = mint_token();
    // Empty repository: the token validates but no persisted principal exists.
    let state: SharedState<_, _, _, _> = Arc::new(WorkloadAuthzState::with_clock(
        InMemoryWorkloadPrincipalRepository::new(),
        InMemoryRevocationDenylist::new(),
        permit_authorizer(),
        Jwks::new().add_key(minted.jwk.clone()),
        ValidationConfig::new(ISSUER, AUDIENCE),
        InMemoryAuditSink::new(),
        || NOW,
    ));
    let router = build_router(state);

    let (status, _body) = post_json(
        router,
        "/authorize-with-token",
        json!({
            "token": minted.token,
            "action": "cloud.kms.Decrypt",
            "resource": { "resourceType": "Secret", "resourceId": "db-password" }
        }),
    )
    .await;

    // An unknown subject must NOT be a 404 — it is a fail-closed 403.
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn store_unavailable_is_503_fail_closed() {
    let minted = mint_token();
    let state: SharedState<FailingRepository, _, _, _> = Arc::new(WorkloadAuthzState::with_clock(
        FailingRepository,
        InMemoryRevocationDenylist::new(),
        permit_authorizer(),
        Jwks::new().add_key(minted.jwk.clone()),
        ValidationConfig::new(ISSUER, AUDIENCE),
        InMemoryAuditSink::new(),
        || NOW,
    ));
    let router = build_router(state);

    let (status, body) = post_json(
        router,
        "/authorize-with-token",
        json!({
            "token": minted.token,
            "action": "cloud.kms.Decrypt",
            "resource": { "resourceType": "Secret", "resourceId": "db-password" }
        }),
    )
    .await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["error"]["code"], "DEPENDENCY_UNAVAILABLE");
}

#[tokio::test]
async fn tokens_validate_returns_principal_and_audits() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let audit = state.audit().clone();
    let router = build_router(state);

    let (status, body) =
        post_json(router, "/tokens/validate", json!({ "token": minted.token })).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["tenantId"], "ten_acme");
    assert_eq!(body["workloadId"], "wl_secrets_sync");
    assert_eq!(body["trustDomain"], "spiffe://ten_acme");
    assert_eq!(body["state"], "active");
    assert_eq!(audit.len(), 1);
    assert_eq!(audit.records()[0].event(), AuditEvent::TokenValidation);
    assert_eq!(audit.records()[0].outcome(), "validated");
}

#[tokio::test]
async fn tokens_validate_failure_is_422_and_audits() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let audit = state.audit().clone();
    let router = build_router(state);

    let (status, body) = post_json(router, "/tokens/validate", json!({ "token": "garbage" })).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "TOKEN_INVALID");
    assert_eq!(audit.records()[0].outcome(), "validation-failed");
}

#[tokio::test]
async fn authorize_already_verified_principal_permit() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let router = build_router(state);

    let (status, body) = post_json(
        router,
        "/authorize",
        json!({
            "tenantId": "ten_acme",
            "workloadId": "wl_secrets_sync",
            "owningCapability": "cap.cloud.kms",
            "scopes": ["cloud.kms.decrypt"],
            "claims": {},
            "action": "cloud.kms.Decrypt",
            "resource": { "resourceType": "Secret", "resourceId": "db-password" }
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["effect"], "ALLOW");
}

#[tokio::test]
async fn authorize_batch_returns_per_item_decisions() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let audit = state.audit().clone();
    let router = build_router(state);

    let (status, body) = post_json(
        router,
        "/authorize:batch",
        json!({
            "requests": [
                {
                    "token": minted.token,
                    "action": "cloud.kms.Decrypt",
                    "resource": { "resourceType": "Secret", "resourceId": "db-password" }
                },
                {
                    "token": minted.token,
                    "action": "cloud.kms.Encrypt",
                    "resource": { "resourceType": "Secret", "resourceId": "db-password" }
                }
            ]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    // First permits (decrypt), second default-denies (encrypt not permitted).
    assert_eq!(body["decisions"][0]["effect"], "ALLOW");
    assert_eq!(body["decisions"][1]["effect"], "DENY");
    // One audit record per batch item.
    assert_eq!(audit.len(), 2);
}

#[tokio::test]
async fn suspend_then_authorize_is_403() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let router = build_router(state);

    // Suspend wl_secrets_sync.
    let (suspend_status, suspend_body) = post_json(
        router.clone(),
        "/principals/wl_secrets_sync:suspend",
        json!({}),
    )
    .await;
    assert_eq!(suspend_status, StatusCode::OK);
    assert_eq!(suspend_body["state"], "suspended");

    // The now-revoked principal is denied on the hot path (403, not 404).
    let (status, _body) = post_json(
        router,
        "/authorize-with-token",
        json!({
            "token": minted.token,
            "action": "cloud.kms.Decrypt",
            "resource": { "resourceType": "Secret", "resourceId": "db-password" }
        }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn retire_unknown_principal_is_404() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let router = build_router(state);

    let (status, body) = post_json(router, "/principals/wl_ghost:retire", json!({})).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}

#[tokio::test]
async fn suspend_invalid_id_is_400() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let router = build_router(state);

    let (status, _body) = post_json(router, "/principals/not-a-wl-id:suspend", json!({})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
