//! Shared test fixtures for REST and gRPC integration tests.
//!
//! Both `rest_endpoints.rs` and `grpc_authorize_deny.rs` need the same
//! ES256 JWT mint, JWKS, Cedar authorizer, and provisioned-state builder.
//! Centralised here so REST/gRPC tests exercise one shared setup and any
//! drift between the two surfaces is immediately visible.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]

use std::collections::BTreeMap;
use std::sync::Arc;

use aws_lc_rs::rand::SystemRandom;
use aws_lc_rs::signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, KeyPair};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

use iam_identity_oidc_issuer_kernel::{
    AccessTokenSpec, Algorithm as IssuerAlgorithm, Audience, IssuerError, IssuerUrl, JwsSigner,
    Signature, SigningKey, Subject, build_access_token_claims,
};
use iam_identity_workload_app::{
    InMemoryRevocationDenylist, InMemoryWorkloadPrincipalRepository, RepositoryError,
    RevocationDenylist, WorkloadPrincipalRepository, activate, provision,
};
use iam_identity_workload_authz_cedar::CedarWorkloadAuthorizer;
use iam_identity_workload_domain::{WorkloadId, WorkloadPrincipal};
use iam_identity_workload_oidc::{Jwk, Jwks, ValidationConfig};
use iam_identity_workload_rest::{
    AuthzFault, BearerCallerVerifier, CallerVerifier, DecisionAuthorizer, DecisionAuthzRequest,
    InMemoryAuditSink, LifecycleAuthorizer, LifecycleAuthzRequest, SharedState, WorkloadAuthzState,
};

pub const ISSUER: &str = "https://idp.oyatie.com";
pub const AUDIENCE: &str = "cloud-kms";
pub const KID: &str = "kid-grpc-1";
pub const NOW: i64 = 1_700_000_000;

/// The bearer credential the lifecycle control plane verifies in tests. The
/// verified caller is bound to `ten_acme` so a same-tenant suspend/retire of
/// `wl_secrets_sync` (also `ten_acme`) is permitted by the tenant-scoped
/// authorizer below.
pub const LIFECYCLE_BEARER: &str = "test-lifecycle-bearer";
/// Tenant the verified lifecycle caller acts within (matches the provisioned
/// principal's tenant so the default test caller is same-tenant).
pub const LIFECYCLE_CALLER_TENANT: &str = "ten_acme";
/// Identity label for the verified lifecycle caller.
pub const LIFECYCLE_CALLER_ID: &str = "test-control-plane";

/// Build the reference [`CallerVerifier`] used by the lifecycle tests: a constant
/// bearer bound to `ten_acme`. A request must carry `Authorization: Bearer
/// {LIFECYCLE_BEARER}` to mint a verified caller; anything else is `401`.
pub fn lifecycle_verifier() -> Arc<dyn CallerVerifier> {
    Arc::new(BearerCallerVerifier::new(
        LIFECYCLE_BEARER,
        LIFECYCLE_CALLER_TENANT,
        LIFECYCLE_CALLER_ID,
    ))
}

/// A tenant-scoped [`LifecycleAuthorizer`] reference: PERMIT iff caller_tenant ==
/// target_tenant (mirrors the production adapter's isolation invariant). Used to
/// prove cross-tenant suspend/retire is a 403.
#[derive(Clone, Copy, Debug, Default)]
pub struct SameTenantLifecycleAuthorizer;

impl LifecycleAuthorizer for SameTenantLifecycleAuthorizer {
    fn decide(&self, request: &LifecycleAuthzRequest<'_>) -> Result<bool, AuthzFault> {
        if request.caller_tenant.is_empty() || request.target_tenant.is_empty() {
            return Err(AuthzFault::new("empty tenant"));
        }
        Ok(request.caller_tenant == request.target_tenant)
    }
}

/// An always-permit authorizer used to ISOLATE the cross-tenant binding proof:
/// with this authorizer a cross-tenant request would be permitted UNLESS the
/// handler binds the target's real tenant. Lets a test prove the handler passes
/// the target's tenant (not the caller's) to the PDP.
#[derive(Clone, Copy, Debug, Default)]
pub struct AllowAllLifecycleAuthorizer;

impl LifecycleAuthorizer for AllowAllLifecycleAuthorizer {
    fn decide(&self, _request: &LifecycleAuthzRequest<'_>) -> Result<bool, AuthzFault> {
        Ok(true)
    }
}

/// An authorizer that always FAULTS (simulates a PDP outage/panic surfaced as
/// `Err`). Proves a PDP fault maps to a fail-closed 403, never a 500/allow.
#[derive(Clone, Copy, Debug, Default)]
pub struct FaultingLifecycleAuthorizer;

impl LifecycleAuthorizer for FaultingLifecycleAuthorizer {
    fn decide(&self, _request: &LifecycleAuthzRequest<'_>) -> Result<bool, AuthzFault> {
        Err(AuthzFault::new("induced PDP fault"))
    }
}

/// A tenant-scoped [`DecisionAuthorizer`] reference for the READ decision surfaces:
/// PERMIT iff caller_tenant == subject_tenant (mirrors the production
/// `TenantScopedDecisionAuthorizer`). Used to prove a verified caller cannot obtain
/// a cross-tenant decision (forged body / cross-tenant token -> 403).
#[derive(Clone, Copy, Debug, Default)]
pub struct SameTenantDecisionAuthorizer;

impl DecisionAuthorizer for SameTenantDecisionAuthorizer {
    fn decide(&self, request: &DecisionAuthzRequest<'_>) -> Result<bool, AuthzFault> {
        if request.caller_tenant.is_empty() || request.subject_tenant.is_empty() {
            return Err(AuthzFault::new("empty tenant"));
        }
        Ok(request.caller_tenant == request.subject_tenant)
    }
}

/// A [`DecisionAuthorizer`] that always FAULTS (PDP outage surfaced as `Err`).
/// Proves a decision-PDP fault maps to a fail-closed 403, never a 500/allow.
#[derive(Clone, Copy, Debug, Default)]
pub struct FaultingDecisionAuthorizer;

impl DecisionAuthorizer for FaultingDecisionAuthorizer {
    fn decide(&self, _request: &DecisionAuthzRequest<'_>) -> Result<bool, AuthzFault> {
        Err(AuthzFault::new("induced decision PDP fault"))
    }
}

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

struct TestIssuerSigner {
    kid: String,
    key_pair: EcdsaKeyPair,
    rng: SystemRandom,
}

impl JwsSigner for TestIssuerSigner {
    fn sign(&self, signing_input: &[u8], kid: &str) -> Result<Signature, IssuerError> {
        if kid != self.kid {
            return Err(IssuerError::InvalidKid);
        }
        let signature = self
            .key_pair
            .sign(&self.rng, signing_input)
            .map_err(|_| IssuerError::MalformedClientAssertion("test issuer signing failed"))?;
        Signature::new(b64url(signature.as_ref()))
    }
}

fn issuer_kernel_signing_material() -> (TestIssuerSigner, SigningKey, Jwk) {
    let rng = SystemRandom::new();
    let pkcs8 =
        EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).expect("pkcs8");
    let key_pair =
        EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8.as_ref()).expect("key");
    let public = key_pair.public_key().as_ref();
    let x = b64url(&public[1..33]);
    let y = b64url(&public[33..65]);
    let mut components = BTreeMap::new();
    components.insert("crv".to_owned(), "P-256".to_owned());
    components.insert("x".to_owned(), x.clone());
    components.insert("y".to_owned(), y.clone());
    let mut signing_key =
        SigningKey::provision(KID, IssuerAlgorithm::Es256, components).expect("signing key");
    signing_key.activate(NOW).expect("activate signing key");
    (
        TestIssuerSigner {
            kid: KID.to_owned(),
            key_pair,
            rng: SystemRandom::new(),
        },
        signing_key,
        Jwk::ec_p256(KID, x, y),
    )
}

/// Mint a real ES256 workload JWT through the issuer-kernel claim validation
/// path, then add the workload-specific `owning_capability` extension that the
/// workload verifier projects into the principal.
pub fn mint_issuer_kernel_access_token() -> MintedToken {
    let (signer, signing_key, jwk) = issuer_kernel_signing_material();
    let claims = build_access_token_claims(AccessTokenSpec {
        issuer: IssuerUrl::new(ISSUER).expect("issuer"),
        audience: Audience::single(AUDIENCE).expect("audience"),
        subject: Subject::new("wl_secrets_sync").expect("subject"),
        tenant_id: "ten_acme".to_owned(),
        scopes: vec!["cloud.kms.decrypt".to_owned()],
        issued_at_epoch_seconds: NOW,
        expires_at_epoch_seconds: NOW + 300,
        purpose: None,
        data_class: None,
    })
    .expect("issuer-kernel access-token claims");
    let header = serde_json::json!({
        "alg": signing_key.algorithm().as_str(),
        "typ": "at+jwt",
        "kid": signing_key.kid(),
    });
    let mut payload = serde_json::json!({
        "iss": claims.iss,
        "aud": claims.aud,
        "sub": claims.sub,
        "iat": claims.iat,
        "exp": claims.exp,
        "nbf": claims.nbf,
        "scope": claims.scope,
        "tenant_id": claims.tenant_id,
        "token_type": claims.token_type,
    });
    if let serde_json::Value::Object(map) = &mut payload {
        map.insert(
            "owning_capability".to_owned(),
            serde_json::json!("cap.cloud.kms"),
        );
    }
    let signing_input = format!(
        "{}.{}",
        b64url(header.to_string().as_bytes()),
        b64url(payload.to_string().as_bytes())
    );
    let signature = signer
        .sign(signing_input.as_bytes(), signing_key.kid())
        .expect("issuer signer");
    MintedToken {
        token: format!("{signing_input}.{}", signature.as_str()),
        jwk,
    }
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
        lifecycle_verifier(),
        Arc::new(SameTenantLifecycleAuthorizer),
        Arc::new(SameTenantDecisionAuthorizer),
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
