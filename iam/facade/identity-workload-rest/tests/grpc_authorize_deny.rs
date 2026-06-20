//! In-process gRPC integration tests for the workload-identity gRPC surface.
//!
//! Tests drive the tonic service impls directly (tonic::Request/Response, no
//! TCP socket), which avoids port-allocation flakiness and exercises the full
//! use-case delegation path without transport overhead.
//!
//! The shared fixtures (mint_token, permit_authorizer, provisioned_state) live
//! in tests/common/mod.rs and are the same ones used by rest_endpoints.rs, so
//! REST and gRPC tests provably exercise one shared setup.
//!
//! ## Assertions
//!
//! (a) Allowed principal -> PERMIT (DECISION_EFFECT_ALLOW).
//! (b) Forbidden principal -> DECISION_EFFECT_DENY response (NOT an RPC error, NOT not-found).
//! (c) Invalid token -> typed ValidateTokenResponse error, policy engine NOT consulted.
//! (d) Store unavailable injection -> fail-closed Unavailable status.
//!     Note: in this design JWKS is a static in-memory keyset, so an unknown-kid
//!     token maps to a typed TokenRejected DENY (not Unavailable); the store-fault
//!     is the only path that produces tonic Code::Unavailable.
//! (e) Audit: exactly one AuditRecord emitted per authorize and per token-validation.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::sync::Arc;

use tonic::Request;

use iam_identity_workload_app::{InMemoryRevocationDenylist, WorkloadPrincipalRepository};
use iam_identity_workload_authz_cedar::CedarWorkloadAuthorizer;
use iam_identity_workload_oidc::{Jwks, ValidationConfig};
use iam_identity_workload_rest::{
    AuditEvent, InMemoryAuditSink, SharedState, WorkloadAuthzState,
    grpc::{
        WorkloadGrpcServer,
        proto::{
            AuthorizeRequest, AuthorizeWithTokenRequest, BatchAuthorizeRequest, DecisionEffect,
            ValidateTokenRequest,
            workload_authorizer_server::WorkloadAuthorizer as _,
            workload_token_validator_server::WorkloadTokenValidator as _,
            validate_token_response,
        },
    },
};

use iam_identity_workload_rest::grpc::proto::claim_value::Value as ProtoClaimValue;

use common::{
    FailingRepository, AUDIENCE, ISSUER, NOW, mint_token, permit_authorizer, provisioned_state,
};

// =====================================================================
// Helper: build a proto Resource message
// =====================================================================

fn secret_resource() -> iam_identity_workload_rest::grpc::proto::Resource {
    iam_identity_workload_rest::grpc::proto::Resource {
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
    let mut repo = iam_identity_workload_app::InMemoryWorkloadPrincipalRepository::new();
    iam_identity_workload_app::provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms")
        .unwrap();
    iam_identity_workload_app::activate(
        &mut repo,
        &iam_identity_workload_domain::WorkloadId::new("wl_secrets_sync").unwrap(),
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
        jwk: iam_identity_workload_oidc::Jwk,
        token: String,
    ) -> (
        iam_identity_workload_rest::grpc::proto::AuthorizeResponse,
        iam_identity_workload_rest::InMemoryAuditSink,
    ) {
        let mut repo = iam_identity_workload_app::InMemoryWorkloadPrincipalRepository::new();
        iam_identity_workload_app::provision(
            &mut repo,
            "ten_acme",
            "wl_secrets_sync",
            "cap.cloud.kms",
        )
        .unwrap();
        iam_identity_workload_app::activate(
            &mut repo,
            &iam_identity_workload_domain::WorkloadId::new("wl_secrets_sync").unwrap(),
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
        let audit = state.audit().clone();
        let resp = WorkloadGrpcServer::new(state)
            .authorize_with_token(Request::new(AuthorizeWithTokenRequest {
                token,
                action: "cloud.kms.Decrypt".to_owned(),
                resource: Some(iam_identity_workload_rest::grpc::proto::Resource {
                    resource_type: "Secret".to_owned(),
                    resource_id: "db-password".to_owned(),
                }),
                context: Default::default(),
            }))
            .await
            .expect("rpc ok")
            .into_inner();
        (resp, audit)
    }

    let (r_permit, audit_permit) = run_with(
        permit_authorizer(),
        minted.jwk.clone(),
        "garbage-token".to_owned(),
    )
    .await;
    let (r_empty, _audit_empty) = run_with(
        CedarWorkloadAuthorizer::new(),
        minted.jwk.clone(),
        "garbage-token".to_owned(),
    )
    .await;

    // Both must be DENY (not allow). If the engine were consulted on the permit
    // path it would have returned ALLOW — the DENY proves engine was not reached.
    assert_eq!(r_permit.effect, DecisionEffect::Deny as i32);
    assert_eq!(r_empty.effect, DecisionEffect::Deny as i32);

    // F3: audit outcome for invalid-token authorize path must be "token-rejected".
    let permit_records = audit_permit.records();
    assert_eq!(permit_records.len(), 1);
    assert_eq!(
        permit_records[0].outcome(),
        "token-rejected",
        "authorize with invalid token must emit 'token-rejected' audit outcome"
    );
}

// =====================================================================
// (d) Store unavailable -> fail-closed Unavailable status
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
// F2: bare Authorize RPC (already-verified principal, no token)
// =====================================================================

#[tokio::test]
async fn authorize_already_verified_principal_permit() {
    // Mirrors REST `authorize_already_verified_principal_permit`.
    // No token involved — the gRPC caller asserts the principal directly.
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let audit = state.audit().clone();
    let server = WorkloadGrpcServer::new(state);

    let result = server
        .authorize(Request::new(AuthorizeRequest {
            tenant_id: "ten_acme".to_owned(),
            workload_id: "wl_secrets_sync".to_owned(),
            owning_capability: "cap.cloud.kms".to_owned(),
            scopes: vec!["cloud.kms.decrypt".to_owned()],
            claims: Default::default(),
            action: "cloud.kms.Decrypt".to_owned(),
            resource: Some(secret_resource()),
            context: Default::default(),
        }))
        .await;

    // (i) permit -> DECISION_EFFECT_ALLOW.
    assert!(result.is_ok(), "authorize must succeed for permitted principal");
    assert_eq!(
        result.unwrap().into_inner().effect,
        DecisionEffect::Allow as i32,
        "expected ALLOW for permitted principal via bare Authorize RPC"
    );
    // One audit record emitted.
    assert_eq!(audit.len(), 1);
    assert_eq!(audit.records()[0].event(), AuditEvent::Authorize);
    assert_eq!(audit.records()[0].outcome(), "allow");
}

#[tokio::test]
async fn authorize_already_verified_principal_default_deny() {
    // (ii) default-deny -> DECISION_EFFECT_DENY response (not Err).
    let minted = mint_token();
    let mut repo = iam_identity_workload_app::InMemoryWorkloadPrincipalRepository::new();
    iam_identity_workload_app::provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms")
        .unwrap();
    iam_identity_workload_app::activate(
        &mut repo,
        &iam_identity_workload_domain::WorkloadId::new("wl_secrets_sync").unwrap(),
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

    let result = server
        .authorize(Request::new(AuthorizeRequest {
            tenant_id: "ten_acme".to_owned(),
            workload_id: "wl_secrets_sync".to_owned(),
            owning_capability: "cap.cloud.kms".to_owned(),
            scopes: vec!["cloud.kms.decrypt".to_owned()],
            claims: Default::default(),
            action: "cloud.kms.Decrypt".to_owned(),
            resource: Some(secret_resource()),
            context: Default::default(),
        }))
        .await;

    // A deny is a response value, NOT a tonic Err (key PEP invariant).
    assert!(result.is_ok(), "a deny must be a response value, not a tonic Err");
    assert_eq!(
        result.unwrap().into_inner().effect,
        DecisionEffect::Deny as i32,
        "expected DECISION_EFFECT_DENY for default-deny authorizer"
    );
    let records = state.audit().records();
    assert_eq!(records.len(), 1, "exactly one audit record per authorize call");
    assert_eq!(records[0].outcome(), "deny");
}

// =====================================================================
// Claims-parity: a claims-gated Cedar policy must yield the same
// ALLOW/DENY decision over gRPC as over REST (fixes blocker: claims
// were silently dropped from the bare Authorize RPC, causing a
// claim-conditioned forbid/permit to diverge between surfaces).
// =====================================================================

#[tokio::test]
async fn authorize_claims_gated_permit_returns_allow_when_claim_present() {
    // Cedar policy: permit only when principal has a claim `env == "prod"`.
    let claims_policy = CedarWorkloadAuthorizer::from_cedar_policies(
        r#"
        @id("permit-acme-kms-decrypt-prod-env")
        permit (
          principal is Workload,
          action == Action::"cloud.kms.Decrypt",
          resource is Secret
        ) when {
          principal.tenant_id == "ten_acme" &&
          principal.scopes.contains("cloud.kms.decrypt") &&
          principal has claim_env &&
          principal.claim_env.contains("prod")
        };
        "#,
    )
    .expect("cedar parses");

    let minted = mint_token();
    let mut repo = iam_identity_workload_app::InMemoryWorkloadPrincipalRepository::new();
    iam_identity_workload_app::provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms")
        .unwrap();
    iam_identity_workload_app::activate(
        &mut repo,
        &iam_identity_workload_domain::WorkloadId::new("wl_secrets_sync").unwrap(),
    )
    .unwrap();
    let state: SharedState<_, _, _, _> = Arc::new(WorkloadAuthzState::with_clock(
        repo,
        InMemoryRevocationDenylist::new(),
        claims_policy,
        Jwks::new().add_key(minted.jwk.clone()),
        ValidationConfig::new(ISSUER, AUDIENCE),
        InMemoryAuditSink::new(),
        || NOW,
    ));
    let server = WorkloadGrpcServer::new(state.clone());

    // Build proto ClaimValue for `env = "prod"`.
    let claim_env_prod = iam_identity_workload_rest::grpc::proto::ClaimValue {
        value: Some(ProtoClaimValue::Text("prod".to_owned())),
    };

    // With the matching claim -> ALLOW.
    let result_with_claim = server
        .authorize(Request::new(AuthorizeRequest {
            tenant_id: "ten_acme".to_owned(),
            workload_id: "wl_secrets_sync".to_owned(),
            owning_capability: "cap.cloud.kms".to_owned(),
            scopes: vec!["cloud.kms.decrypt".to_owned()],
            claims: [("env".to_owned(), claim_env_prod)].into_iter().collect(),
            action: "cloud.kms.Decrypt".to_owned(),
            resource: Some(secret_resource()),
            context: Default::default(),
        }))
        .await
        .expect("rpc ok");

    assert_eq!(
        result_with_claim.into_inner().effect,
        DecisionEffect::Allow as i32,
        "claims-gated policy must ALLOW when the required claim is present over gRPC (principal.claim_env.contains('prod'))"
    );

    // Without the claim -> DENY (claim-gated policy does not fire).
    let result_without_claim = server
        .authorize(Request::new(AuthorizeRequest {
            tenant_id: "ten_acme".to_owned(),
            workload_id: "wl_secrets_sync".to_owned(),
            owning_capability: "cap.cloud.kms".to_owned(),
            scopes: vec!["cloud.kms.decrypt".to_owned()],
            claims: Default::default(),
            action: "cloud.kms.Decrypt".to_owned(),
            resource: Some(secret_resource()),
            context: Default::default(),
        }))
        .await
        .expect("rpc ok");

    assert_eq!(
        result_without_claim.into_inner().effect,
        DecisionEffect::Deny as i32,
        "claims-gated policy must DENY when the required claim is absent over gRPC"
    );

    // Two audit records, one per authorize call.
    assert_eq!(state.audit().len(), 2);
    assert_eq!(state.audit().records()[0].outcome(), "allow");
    assert_eq!(state.audit().records()[1].outcome(), "deny");
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
