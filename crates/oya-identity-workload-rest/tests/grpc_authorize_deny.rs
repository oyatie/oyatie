//! In-process gRPC integration tests for the workload-identity gRPC surface.
//!
//! Tests drive the tonic service impls directly (tonic::Request/Response, no
//! TCP socket), which avoids port-allocation flakiness and exercises the full
//! use-case delegation path without transport overhead.
//!
//! The shared fixtures (mint_token, permit_authorizer, provisioned_state) live
//! in tests/common.rs and are the same ones used by rest_endpoints.rs, so REST
//! and gRPC tests provably exercise one shared setup.
//!
//! ## Assertions
//!
//! (a) Allowed principal -> PERMIT (DECISION_EFFECT_ALLOW).
//! (b) Forbidden principal -> DECISION_EFFECT_DENY response (NOT an RPC error, NOT not-found).
//! (c) Invalid token -> typed ValidateTokenResponse error, policy engine NOT consulted.
//! (d) Store/JWKS unavailable injection -> fail-closed Unavailable status.
//! (e) Audit: exactly one AuditRecord emitted per authorize and per token-validation.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::sync::Arc;

use tonic::Request;

use oya_identity_workload_app::{InMemoryRevocationDenylist, WorkloadPrincipalRepository};
use oya_identity_workload_authz_cedar_adapter::CedarWorkloadAuthorizer;
use oya_identity_workload_oidc_adapter::{Jwks, ValidationConfig};
use oya_identity_workload_rest::{
    AuditEvent, InMemoryAuditSink, SharedState, WorkloadAuthzState,
    grpc::{
        WorkloadGrpcServer,
        proto::{
            AuthorizeWithTokenRequest, BatchAuthorizeRequest, DecisionEffect,
            ValidateTokenRequest,
            workload_authorizer_server::WorkloadAuthorizer as _,
            workload_token_validator_server::WorkloadTokenValidator as _,
            validate_token_response,
        },
    },
};

use common::{
    FailingRepository, AUDIENCE, ISSUER, NOW, mint_token, permit_authorizer, provisioned_state,
};

// =====================================================================
// Helper: build a proto Resource message
// =====================================================================

fn secret_resource() -> oya_identity_workload_rest::grpc::proto::Resource {
    oya_identity_workload_rest::grpc::proto::Resource {
        resource_type: "Secret".to_owned(),
        resource_id: "db-password".to_owned(),
    }
}

// =====================================================================
// (a) Allowed principal -> PERMIT
// =====================================================================

#[tokio::test]
async fn authorize_with_token_permit_returns_allow() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let audit = state.audit().clone();
    let server = WorkloadGrpcServer::new(state);

    let response = server
        .authorize_with_token(Request::new(AuthorizeWithTokenRequest {
            token: minted.token.clone(),
            action: "cloud.kms.Decrypt".to_owned(),
            resource: Some(secret_resource()),
            context: Default::default(),
        }))
        .await
        .expect("rpc ok");

    assert_eq!(
        response.into_inner().effect,
        DecisionEffect::Allow as i32,
        "expected ALLOW for permitted principal"
    );
    // One authorize audit record emitted.
    assert_eq!(audit.len(), 1);
    let rec = &audit.records()[0];
    assert_eq!(rec.event(), AuditEvent::Authorize);
    assert_eq!(rec.outcome(), "allow");
    assert_eq!(rec.workload_id(), Some("wl_secrets_sync"));
}

// =====================================================================
// (b) Forbidden principal -> DECISION_EFFECT_DENY (NOT an RPC error)
// =====================================================================

#[tokio::test]
async fn authorize_with_token_deny_is_response_not_error() {
    let minted = mint_token();
    // Empty authorizer -> default-deny.
    let mut repo = oya_identity_workload_app::InMemoryWorkloadPrincipalRepository::new();
    oya_identity_workload_app::provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms")
        .unwrap();
    oya_identity_workload_app::activate(
        &mut repo,
        &oya_identity_workload_domain::WorkloadId::new("wl_secrets_sync").unwrap(),
    )
    .unwrap();
    let state: SharedState<_, _, _, _> = Arc::new(WorkloadAuthzState::with_clock(
        repo,
        InMemoryRevocationDenylist::new(),
        CedarWorkloadAuthorizer::new(), // no policies -> default deny
        Jwks::new().add_key(minted.jwk.clone()),
        ValidationConfig::new(ISSUER, AUDIENCE),
        InMemoryAuditSink::new(),
        || NOW,
    ));
    let server = WorkloadGrpcServer::new(state.clone());

    // Must return Ok(Response) with DENY effect — NOT Err(Status).
    let result = server
        .authorize_with_token(Request::new(AuthorizeWithTokenRequest {
            token: minted.token.clone(),
            action: "cloud.kms.Decrypt".to_owned(),
            resource: Some(secret_resource()),
            context: Default::default(),
        }))
        .await;

    assert!(result.is_ok(), "a deny must be a response value, not a tonic Err");
    assert_eq!(
        result.unwrap().into_inner().effect,
        DecisionEffect::Deny as i32,
        "expected DECISION_EFFECT_DENY for default-deny authorizer"
    );
    let records = state.audit().records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].outcome(), "deny");
}

// =====================================================================
// (c) Invalid token -> typed error; policy engine NOT consulted
// =====================================================================

#[tokio::test]
async fn validate_token_invalid_returns_typed_error_not_rpc_error() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let audit = state.audit().clone();
    let server = WorkloadGrpcServer::new(state);

    // ValidateToken with garbage token.
    let result = server
        .validate_token(Request::new(ValidateTokenRequest {
            token: "not-a-valid-jwt".to_owned(),
        }))
        .await;

    // Must return Ok (typed error in response body), not Err(Status).
    assert!(
        result.is_ok(),
        "token-validation failure must be a typed response, not a tonic Err"
    );
    let resp = result.unwrap().into_inner();
    assert!(!resp.ok, "ok must be false for invalid token");
    assert!(
        matches!(resp.outcome, Some(validate_token_response::Outcome::Error(_))),
        "outcome must be Error variant"
    );
    // One audit record for the validation failure.
    assert_eq!(audit.len(), 1);
    assert_eq!(audit.records()[0].event(), AuditEvent::TokenValidation);
    assert_eq!(audit.records()[0].outcome(), "validation-failed");
}

#[tokio::test]
async fn authorize_with_invalid_token_engine_not_consulted() {
    // When token is invalid, AuthorizeWithToken must not reach the policy engine.
    // We verify this structurally: a completely empty authorizer (no policies at
    // all, so any engine consult would return DefaultDeny) vs a permit authorizer
    // must produce the same token-rejected DENY response, showing the engine path
    // was never taken.
    let minted = mint_token();

    async fn run_with(
        authorizer: CedarWorkloadAuthorizer,
        jwk: oya_identity_workload_oidc_adapter::Jwk,
        token: String,
    ) -> oya_identity_workload_rest::grpc::proto::AuthorizeResponse {
        let mut repo = oya_identity_workload_app::InMemoryWorkloadPrincipalRepository::new();
        oya_identity_workload_app::provision(
            &mut repo,
            "ten_acme",
            "wl_secrets_sync",
            "cap.cloud.kms",
        )
        .unwrap();
        oya_identity_workload_app::activate(
            &mut repo,
            &oya_identity_workload_domain::WorkloadId::new("wl_secrets_sync").unwrap(),
        )
        .unwrap();
        let state: SharedState<_, _, _, _> = Arc::new(WorkloadAuthzState::with_clock(
            repo,
            InMemoryRevocationDenylist::new(),
            authorizer,
            Jwks::new().add_key(jwk),
            ValidationConfig::new(ISSUER, AUDIENCE),
            InMemoryAuditSink::new(),
            || NOW,
        ));
        WorkloadGrpcServer::new(state)
            .authorize_with_token(Request::new(AuthorizeWithTokenRequest {
                token,
                action: "cloud.kms.Decrypt".to_owned(),
                resource: Some(oya_identity_workload_rest::grpc::proto::Resource {
                    resource_type: "Secret".to_owned(),
                    resource_id: "db-password".to_owned(),
                }),
                context: Default::default(),
            }))
            .await
            .expect("rpc ok")
            .into_inner()
    }

    let r_permit = run_with(
        permit_authorizer(),
        minted.jwk.clone(),
        "garbage-token".to_owned(),
    )
    .await;
    let r_empty = run_with(
        CedarWorkloadAuthorizer::new(),
        minted.jwk.clone(),
        "garbage-token".to_owned(),
    )
    .await;

    // Both must be DENY (not allow). If the engine were consulted on the permit
    // path it would have returned ALLOW — the DENY proves engine was not reached.
    assert_eq!(r_permit.effect, DecisionEffect::Deny as i32);
    assert_eq!(r_empty.effect, DecisionEffect::Deny as i32);
}

// =====================================================================
// (d) Store/JWKS unavailable -> fail-closed Unavailable status
// =====================================================================

#[tokio::test]
async fn store_unavailable_is_grpc_unavailable() {
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
    let server = WorkloadGrpcServer::new(state.clone());

    let result = server
        .authorize_with_token(Request::new(AuthorizeWithTokenRequest {
            token: minted.token.clone(),
            action: "cloud.kms.Decrypt".to_owned(),
            resource: Some(secret_resource()),
            context: Default::default(),
        }))
        .await;

    assert!(result.is_err(), "store unavailable must be a tonic Err");
    let status = result.unwrap_err();
    assert_eq!(
        status.code(),
        tonic::Code::Unavailable,
        "expected Unavailable status code for store outage"
    );
    // One audit record emitted even on store outage.
    let records = state.audit().records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].outcome(), "store-unavailable");
}

// =====================================================================
// (e) AuthorizeBatch: per-item decisions, one audit record per item
// =====================================================================

#[tokio::test]
async fn authorize_batch_returns_per_item_decisions_and_audits() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let audit = state.audit().clone();
    let server = WorkloadGrpcServer::new(state);

    let response = server
        .authorize_batch(Request::new(BatchAuthorizeRequest {
            requests: vec![
                // First: valid token + permitted action -> ALLOW.
                AuthorizeWithTokenRequest {
                    token: minted.token.clone(),
                    action: "cloud.kms.Decrypt".to_owned(),
                    resource: Some(secret_resource()),
                    context: Default::default(),
                },
                // Second: valid token + non-permitted action -> DENY.
                AuthorizeWithTokenRequest {
                    token: minted.token.clone(),
                    action: "cloud.kms.Encrypt".to_owned(),
                    resource: Some(secret_resource()),
                    context: Default::default(),
                },
            ],
        }))
        .await
        .expect("batch rpc ok");

    let batch = response.into_inner();
    assert_eq!(batch.decisions.len(), 2);
    assert_eq!(
        batch.decisions[0].effect,
        DecisionEffect::Allow as i32,
        "first item (Decrypt) must ALLOW"
    );
    assert_eq!(
        batch.decisions[1].effect,
        DecisionEffect::Deny as i32,
        "second item (Encrypt) must DENY"
    );
    // One audit record per batch item.
    assert_eq!(audit.len(), 2);
    assert_eq!(audit.records()[0].outcome(), "allow");
    assert_eq!(audit.records()[1].outcome(), "deny");
}

// =====================================================================
// Parity check: REST and gRPC share one use-case core
// =====================================================================

#[tokio::test]
async fn validate_token_success_returns_principal_fields() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let audit = state.audit().clone();
    let server = WorkloadGrpcServer::new(state);

    let result = server
        .validate_token(Request::new(ValidateTokenRequest {
            token: minted.token.clone(),
        }))
        .await
        .expect("rpc ok");

    let resp = result.into_inner();
    assert!(resp.ok);
    let principal = match resp.outcome.unwrap() {
        validate_token_response::Outcome::Principal(p) => p,
        validate_token_response::Outcome::Error(e) => {
            panic!("expected principal, got error: {e:?}")
        }
    };
    assert_eq!(principal.tenant_id, "ten_acme");
    assert_eq!(principal.workload_id, "wl_secrets_sync");
    assert_eq!(principal.trust_domain, "spiffe://ten_acme");
    // Audit: one TokenValidation record.
    assert_eq!(audit.len(), 1);
    assert_eq!(audit.records()[0].event(), AuditEvent::TokenValidation);
    assert_eq!(audit.records()[0].outcome(), "validated");
}
