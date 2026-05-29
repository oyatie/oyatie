//! End-to-end tests for the workload-identity REST surface.
//!
//! Each test drives the real axum router via `tower::ServiceExt::oneshot` with
//! a genuinely-signed ES256 workload JWT (minted with `ring`), exercising the
//! full validate -> resolve -> authorize hot path through the REAL adapters and
//! asserting the fail-closed PEP status mapping + one-audit-record-per-call
//! invariant (PRD §1.2/§3.4/§3.5/§5, AC-W-13).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use http_body_util::BodyExt as _;
use ring::rand::SystemRandom;
use ring::signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, KeyPair};
use serde_json::{Value, json};
use tower::ServiceExt as _;

use oya_identity_workload_app::{
    InMemoryRevocationDenylist, InMemoryWorkloadPrincipalRepository, RepositoryError,
    RevocationDenylist, WorkloadPrincipalRepository, activate, provision,
};
use oya_identity_workload_authz_cedar_adapter::CedarWorkloadAuthorizer;
use oya_identity_workload_domain::{WorkloadId, WorkloadPrincipal};
use oya_identity_workload_oidc_adapter::{Jwk, Jwks, ValidationConfig};
use oya_identity_workload_rest::{
    AuditEvent, InMemoryAuditSink, SharedState, WorkloadAuthzState, build_router,
};

const ISSUER: &str = "https://idp.oyatie.com";
const AUDIENCE: &str = "oya-cloud-kms";
const KID: &str = "kid-rest-1";
const NOW: i64 = 1_700_000_000;

fn now() -> i64 {
    NOW
}

fn b64url(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

struct MintedToken {
    token: String,
    jwk: Jwk,
}

/// Mint a real ES256 workload JWT for `wl_secrets_sync` (ten_acme).
fn mint_token() -> MintedToken {
    let rng = SystemRandom::new();
    let pkcs8 =
        EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).expect("pkcs8");
    let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8.as_ref(), &rng)
        .expect("key");
    let public = key_pair.public_key().as_ref();
    let x = &public[1..33];
    let y = &public[33..65];
    let claims = format!(
        r#"{{"iss":"{ISSUER}","aud":"{AUDIENCE}","exp":{},"iat":{NOW},"tenant_id":"ten_acme","sub":"wl_secrets_sync","owning_capability":"cap.cloud.kms","scope":"cloud.kms.decrypt"}}"#,
        NOW + 300
    );
    let header = format!(r#"{{"alg":"ES256","typ":"JWT","kid":"{KID}"}}"#);
    let signing_input = format!(
        "{}.{}",
        b64url(header.as_bytes()),
        b64url(claims.as_bytes())
    );
    let sig = key_pair.sign(&rng, signing_input.as_bytes()).expect("sign");
    MintedToken {
        token: format!("{signing_input}.{}", b64url(sig.as_ref())),
        jwk: Jwk::ec_p256(KID, b64url(x), b64url(y)),
    }
}

/// A permit allowing ten_acme + cloud.kms.decrypt to Decrypt a Secret.
fn permit_authorizer() -> CedarWorkloadAuthorizer {
    CedarWorkloadAuthorizer::from_cedar_policies(
        r#"
        @id("permit-acme-kms-decrypt")
        permit (
          principal is Workload,
          action == Action::"cloud.kms.Decrypt",
          resource is Secret
        ) when {
          principal.tenant_id == "ten_acme" &&
          principal.scopes.contains("cloud.kms.decrypt")
        };
        "#,
    )
    .expect("cedar parses")
}

type RestState = SharedState<
    InMemoryWorkloadPrincipalRepository,
    InMemoryRevocationDenylist,
    CedarWorkloadAuthorizer,
    InMemoryAuditSink,
>;

/// Build router state with a provisioned+activated `wl_secrets_sync`.
fn provisioned_state(jwk: Jwk) -> RestState {
    let mut repo = InMemoryWorkloadPrincipalRepository::new();
    provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms").expect("provision");
    activate(&mut repo, &WorkloadId::new("wl_secrets_sync").unwrap()).expect("activate");
    let jwks = Jwks::new().add_key(jwk);
    std::sync::Arc::new(WorkloadAuthzState::with_clock(
        repo,
        InMemoryRevocationDenylist::new(),
        permit_authorizer(),
        jwks,
        ValidationConfig::new(ISSUER, AUDIENCE),
        InMemoryAuditSink::new(),
        now,
    ))
}

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
    let state: RestState = std::sync::Arc::new(WorkloadAuthzState::with_clock(
        repo,
        InMemoryRevocationDenylist::new(),
        CedarWorkloadAuthorizer::new(),
        Jwks::new().add_key(minted.jwk.clone()),
        ValidationConfig::new(ISSUER, AUDIENCE),
        InMemoryAuditSink::new(),
        now,
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
    let state: RestState = std::sync::Arc::new(WorkloadAuthzState::with_clock(
        InMemoryWorkloadPrincipalRepository::new(),
        InMemoryRevocationDenylist::new(),
        permit_authorizer(),
        Jwks::new().add_key(minted.jwk.clone()),
        ValidationConfig::new(ISSUER, AUDIENCE),
        InMemoryAuditSink::new(),
        now,
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

/// A repository whose reads always fail — proves the hot path fail-closes to a
/// 503, never an allow.
struct FailingRepository;
impl WorkloadPrincipalRepository for FailingRepository {
    fn load(
        &self,
        _workload_id: &WorkloadId,
    ) -> Result<Option<WorkloadPrincipal>, RepositoryError> {
        Err(RepositoryError::new("induced load failure"))
    }
    fn save(&mut self, _principal: &WorkloadPrincipal) -> Result<(), RepositoryError> {
        Err(RepositoryError::new("induced save failure"))
    }
}

#[tokio::test]
async fn store_unavailable_is_503_fail_closed() {
    let minted = mint_token();
    let state: SharedState<
        FailingRepository,
        InMemoryRevocationDenylist,
        CedarWorkloadAuthorizer,
        InMemoryAuditSink,
    > = std::sync::Arc::new(WorkloadAuthzState::with_clock(
        FailingRepository,
        InMemoryRevocationDenylist::new(),
        permit_authorizer(),
        Jwks::new().add_key(minted.jwk.clone()),
        ValidationConfig::new(ISSUER, AUDIENCE),
        InMemoryAuditSink::new(),
        now,
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
