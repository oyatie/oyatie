//! Shared test fixtures for REST and gRPC integration tests.
//!
//! Both `rest_endpoints.rs` and `grpc_authorize_deny.rs` need the same
//! ES256 JWT mint, JWKS, Cedar authorizer, and provisioned-state builder.
//! Centralised here so REST/gRPC tests exercise one shared setup and any
//! drift between the two surfaces is immediately visible.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]

use std::sync::Arc;

use aws_lc_rs::rand::SystemRandom;
use aws_lc_rs::signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, KeyPair};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

use oya_identity_workload_app::{
    InMemoryRevocationDenylist, InMemoryWorkloadPrincipalRepository, RepositoryError,
    RevocationDenylist, WorkloadPrincipalRepository, activate, provision,
};
use oya_identity_workload_authz_cedar_adapter::CedarWorkloadAuthorizer;
use oya_identity_workload_domain::{WorkloadId, WorkloadPrincipal};
use oya_identity_workload_oidc_adapter::{Jwk, Jwks, ValidationConfig};
use oya_identity_workload_rest::{InMemoryAuditSink, SharedState, WorkloadAuthzState};

pub const ISSUER: &str = "https://idp.oyatie.com";
pub const AUDIENCE: &str = "oya-cloud-kms";
pub const KID: &str = "kid-grpc-1";
pub const NOW: i64 = 1_700_000_000;

pub fn now() -> i64 {
    NOW
}

pub fn b64url(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

pub struct MintedToken {
    pub token: String,
    pub jwk: Jwk,
}

/// Mint a real ES256 workload JWT for `wl_secrets_sync` (ten_acme).
pub fn mint_token() -> MintedToken {
    let rng = SystemRandom::new();
    let pkcs8 =
        EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).expect("pkcs8");
    let key_pair =
        EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8.as_ref()).expect("key");
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
pub fn permit_authorizer() -> CedarWorkloadAuthorizer {
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

pub type TestState = SharedState<
    InMemoryWorkloadPrincipalRepository,
    InMemoryRevocationDenylist,
    CedarWorkloadAuthorizer,
    InMemoryAuditSink,
>;

/// Build state with a provisioned+activated `wl_secrets_sync`.
pub fn provisioned_state(jwk: Jwk) -> TestState {
    provisioned_state_with_jwks(Jwks::new().add_key(jwk))
}

/// Build state with a caller-supplied static issuer JWKS and a
/// provisioned+activated `wl_secrets_sync`.
pub fn provisioned_state_with_jwks(jwks: Jwks) -> TestState {
    let mut repo = InMemoryWorkloadPrincipalRepository::new();
    provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms").expect("provision");
    activate(&mut repo, &WorkloadId::new("wl_secrets_sync").unwrap()).expect("activate");
    Arc::new(WorkloadAuthzState::with_clock(
        repo,
        InMemoryRevocationDenylist::new(),
        permit_authorizer(),
        jwks,
        ValidationConfig::new(ISSUER, AUDIENCE),
        InMemoryAuditSink::new(),
        now,
    ))
}

/// A repository whose reads always fail — injects store-unavailable.
pub struct FailingRepository;
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
