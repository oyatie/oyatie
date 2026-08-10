//! Acceptance tests for the workload-identity service core, grounded in
//! `iam/identity/workload-identity/PRD.md` §6.
//!
//! These drive the FULL flow through the REAL adapters — a genuine ES256 JWT is
//! minted with `ring`, validated by the real OIDC adapter, and authorized by the
//! real `cedar-policy` engine behind [`CedarWorkloadAuthorizer`]. There are no
//! token or policy stubs.
//!
//! Mapped PRD acceptance criteria:
//! - provision -> activate -> authorize-allow            (lifecycle happy path; AC-W-06 permit)
//! - suspend -> authorize-denied-via-denylist            (PRD §3.5 fast revocation; AC-W-07)
//! - retire terminal (no re-activate)                    (PRD §3.5; AC-W-14 tombstone)
//! - illegal transition rejected                         (domain state machine)
//! - default-deny on invalid token                       (PRD §3.4; never reaches the engine)
//! - forbid-overrides-permit via real Cedar              (PRD §1.2 / §3.4; AC-W-06/AC-W-08 forbid wins)

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;

use aws_lc_rs::rand::SystemRandom;
use aws_lc_rs::signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, KeyPair};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

use iam_identity_workload_app::{
    AuthorizeOutcome, InMemoryRevocationDenylist, InMemoryWorkloadPrincipalRepository,
    LifecycleError, activate, authorize_with_token, provision, record_revocation_event, retire,
    suspend,
};
use iam_identity_workload_authz_cedar::CedarWorkloadAuthorizer;
use iam_identity_workload_domain::{Action, Effect, Resource, WorkloadId, WorkloadState};
use iam_identity_workload_oidc::{Jwk, Jwks, ValidationConfig};

const ISSUER: &str = "https://idp.oyatie.com";
const AUDIENCE: &str = "oya-cloud-kms";
const KID: &str = "kid-acceptance-1";
const NOW: i64 = 1_700_000_000;

fn b64url(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

fn config() -> ValidationConfig {
    ValidationConfig::new(ISSUER, AUDIENCE)
}

/// A minted token plus the JWK that verifies it.
struct MintedToken {
    token: String,
    jwk: Jwk,
}

/// Mint a REAL ES256 workload JWT for `wl_secrets_sync` (ten_acme) using `ring`,
/// returning the token and its verifying JWK. The claims project to an Active
/// principal carrying `cloud.kms.decrypt` scope and `env=prod` per the OIDC
/// adapter's projection rules.
fn mint_workload_token() -> MintedToken {
    mint_workload_token_issued_at(NOW)
}

/// Mint the same REAL ES256 workload JWT with a caller-selected issued-at time.
fn mint_workload_token_issued_at(issued_at_epoch_seconds: i64) -> MintedToken {
    mint_workload_token_with_iat(Some(issued_at_epoch_seconds))
}

/// Mint the same REAL ES256 workload JWT without an `iat` claim.
fn mint_workload_token_without_iat() -> MintedToken {
    mint_workload_token_with_iat(None)
}

fn mint_workload_token_with_iat(issued_at_epoch_seconds: Option<i64>) -> MintedToken {
    let rng = SystemRandom::new();
    let pkcs8 =
        EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).expect("pkcs8");
    let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8.as_ref())
        .expect("key pair");

    let public = key_pair.public_key().as_ref();
    assert_eq!(public.len(), 65, "uncompressed SEC1 point");
    let x = &public[1..33];
    let y = &public[33..65];

    let expires_at_epoch_seconds = issued_at_epoch_seconds.unwrap_or(NOW) + 300;
    let iat_claim = issued_at_epoch_seconds
        .map(|iat| format!(r#", "iat":{iat}"#))
        .unwrap_or_default();
    let claims = format!(
        r#"{{"iss":"{ISSUER}","aud":"{AUDIENCE}","exp":{expires_at_epoch_seconds}{iat_claim},"tenant_id":"ten_acme","sub":"wl_secrets_sync","owning_capability":"cap.cloud.kms","scope":"cloud.kms.decrypt cloud.kms.describe","env":"prod","mfa":true}}"#
    );
    let header = format!(r#"{{"alg":"ES256","typ":"JWT","kid":"{KID}"}}"#);
    let signing_input = format!(
        "{}.{}",
        b64url(header.as_bytes()),
        b64url(claims.as_bytes())
    );
    let sig = key_pair.sign(&rng, signing_input.as_bytes()).expect("sign");
    let token = format!("{signing_input}.{}", b64url(sig.as_ref()));

    MintedToken {
        token,
        jwk: Jwk::ec_p256(KID, b64url(x), b64url(y)),
    }
}

/// Permit policy that allows `ten_acme` workloads holding `cloud.kms.decrypt`
/// to `cloud.kms.Decrypt` a `Secret`. Built as raw Cedar text so the real
/// engine parses + evaluates it (PRD §1.2 "exact /authorize contract shape").
fn permit_only_authorizer() -> CedarWorkloadAuthorizer {
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
    .expect("cedar permit policy parses")
}

/// The above permit PLUS a break-glass forbid on the specific secret. Cedar's
/// forbid-overrides-permit must deny even though the permit matches.
fn permit_and_forbid_authorizer() -> CedarWorkloadAuthorizer {
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

        @id("forbid-frozen-secret")
        forbid (
          principal is Workload,
          action,
          resource is Secret
        ) when {
          resource == Secret::"db-password"
        };
        "#,
    )
    .expect("cedar permit+forbid policy parses")
}

fn decrypt_secret() -> (Action, Resource) {
    (
        Action::new("cloud.kms.Decrypt"),
        Resource::new("Secret", "db-password"),
    )
}

/// AC: provision -> activate -> authorize ALLOW, end to end through the real
/// OIDC validation + real Cedar engine (matching permit).
#[test]
fn provision_activate_then_authorize_allows() {
    let minted = mint_workload_token();
    let jwks = Jwks::new().add_key(minted.jwk.clone());
    let mut repo = InMemoryWorkloadPrincipalRepository::new();
    let denylist = InMemoryRevocationDenylist::new();
    let authorizer = permit_only_authorizer();

    provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms").expect("provision");
    let active =
        activate(&mut repo, &WorkloadId::new("wl_secrets_sync").unwrap()).expect("activate");
    assert_eq!(active.state(), WorkloadState::Active);

    let (action, resource) = decrypt_secret();
    let outcome = authorize_with_token(
        &repo,
        &denylist,
        &authorizer,
        &jwks,
        &config(),
        NOW,
        &minted.token,
        action,
        resource,
        BTreeMap::new(),
    );

    assert!(
        outcome.is_allow(),
        "matching permit must allow the active principal, got {outcome:?}"
    );
    assert_eq!(outcome.decision().effect(), Effect::Allow);
}

/// AC: suspend -> authorize DENIED via the denylist (PRD §3.5). The token is
/// still cryptographically valid and unexpired, but the denylist gate fires.
#[test]
fn suspend_denies_via_denylist_even_with_valid_token() {
    let minted = mint_workload_token();
    let jwks = Jwks::new().add_key(minted.jwk.clone());
    let mut repo = InMemoryWorkloadPrincipalRepository::new();
    let mut denylist = InMemoryRevocationDenylist::new();
    let authorizer = permit_only_authorizer();
    let wl = WorkloadId::new("wl_secrets_sync").unwrap();

    provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms").expect("provision");
    activate(&mut repo, &wl).expect("activate");

    // Sanity: it would be allowed while active.
    let (action, resource) = decrypt_secret();
    assert!(
        authorize_with_token(
            &repo,
            &denylist,
            &authorizer,
            &jwks,
            &config(),
            NOW,
            &minted.token,
            action,
            resource,
            BTreeMap::new(),
        )
        .is_allow()
    );

    // Suspend: writes the denylist.
    let suspended = suspend(&mut repo, &mut denylist, &wl).expect("suspend");
    assert_eq!(suspended.state(), WorkloadState::Suspended);

    let (action, resource) = decrypt_secret();
    let outcome = authorize_with_token(
        &repo,
        &denylist,
        &authorizer,
        &jwks,
        &config(),
        NOW,
        &minted.token,
        action,
        resource,
        BTreeMap::new(),
    );
    assert_eq!(
        outcome,
        AuthorizeOutcome::Revoked,
        "a suspended (denylisted) principal must be denied on the hot path"
    );
    assert!(!outcome.is_allow());
    assert_eq!(outcome.decision().effect(), Effect::Deny);
}

/// CAEP-style shared-signal event: a revocation event updates an issue-time
/// cutoff, and a still-unexpired credential issued before that cutoff is denied
/// within the sub-60s propagation window (without relying on token expiry).
#[test]
fn revocation_event_cutoff_denies_stale_credential_within_sixty_seconds() {
    let minted = mint_workload_token();
    let jwks = Jwks::new().add_key(minted.jwk.clone());
    let mut repo = InMemoryWorkloadPrincipalRepository::new();
    let mut denylist = InMemoryRevocationDenylist::new();
    let authorizer = permit_only_authorizer();
    let wl = WorkloadId::new("wl_secrets_sync").unwrap();

    provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms").expect("provision");
    activate(&mut repo, &wl).expect("activate");

    let (action, resource) = decrypt_secret();
    assert!(
        authorize_with_token(
            &repo,
            &denylist,
            &authorizer,
            &jwks,
            &config(),
            NOW + 58,
            &minted.token,
            action,
            resource,
            BTreeMap::new(),
        )
        .is_allow(),
        "the credential is still cryptographically valid before the event"
    );

    record_revocation_event(&mut denylist, &wl, NOW + 59).expect("record revocation event");

    let (action, resource) = decrypt_secret();
    let outcome = authorize_with_token(
        &repo,
        &denylist,
        &authorizer,
        &jwks,
        &config(),
        NOW + 59,
        &minted.token,
        action,
        resource,
        BTreeMap::new(),
    );

    assert_eq!(
        outcome,
        AuthorizeOutcome::Revoked,
        "a CAEP-style revocation event must deny credentials issued at/before the cutoff"
    );
    assert!(!outcome.is_allow());
}

/// A revocation cutoff is an issue-time boundary, not a permanent principal
/// revoke: stale credentials are denied, but a newer re-attested credential
/// issued after the cutoff can still reach policy evaluation.
#[test]
fn revocation_event_cutoff_allows_newer_credential_after_cutoff() {
    let stale = mint_workload_token();
    let fresh = mint_workload_token_issued_at(NOW + 60);
    let mut repo = InMemoryWorkloadPrincipalRepository::new();
    let mut denylist = InMemoryRevocationDenylist::new();
    let authorizer = permit_only_authorizer();
    let wl = WorkloadId::new("wl_secrets_sync").unwrap();

    provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms").expect("provision");
    activate(&mut repo, &wl).expect("activate");
    record_revocation_event(&mut denylist, &wl, NOW + 59).expect("record revocation event");

    let (action, resource) = decrypt_secret();
    let stale_outcome = authorize_with_token(
        &repo,
        &denylist,
        &authorizer,
        &Jwks::new().add_key(stale.jwk),
        &config(),
        NOW + 59,
        &stale.token,
        action,
        resource,
        BTreeMap::new(),
    );
    assert_eq!(stale_outcome, AuthorizeOutcome::Revoked);

    let (action, resource) = decrypt_secret();
    let fresh_outcome = authorize_with_token(
        &repo,
        &denylist,
        &authorizer,
        &Jwks::new().add_key(fresh.jwk),
        &config(),
        NOW + 60,
        &fresh.token,
        action,
        resource,
        BTreeMap::new(),
    );
    assert!(
        fresh_outcome.is_allow(),
        "a post-cutoff re-attested credential should pass the cutoff gate, got {fresh_outcome:?}"
    );
}

/// A credential missing `iat` is still a valid OIDC token, but once a cutoff
/// exists it cannot prove it was minted after the event and must fail closed.
#[test]
fn revocation_event_cutoff_denies_credential_missing_iat() {
    let minted = mint_workload_token_without_iat();
    let jwks = Jwks::new().add_key(minted.jwk.clone());
    let mut repo = InMemoryWorkloadPrincipalRepository::new();
    let mut denylist = InMemoryRevocationDenylist::new();
    let authorizer = permit_only_authorizer();
    let wl = WorkloadId::new("wl_secrets_sync").unwrap();

    provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms").expect("provision");
    activate(&mut repo, &wl).expect("activate");

    let (action, resource) = decrypt_secret();
    assert!(
        authorize_with_token(
            &repo,
            &denylist,
            &authorizer,
            &jwks,
            &config(),
            NOW,
            &minted.token,
            action,
            resource,
            BTreeMap::new(),
        )
        .is_allow(),
        "without a cutoff, an otherwise-valid token with no iat is still policy-eligible"
    );

    record_revocation_event(&mut denylist, &wl, NOW + 59).expect("record revocation event");

    let (action, resource) = decrypt_secret();
    let outcome = authorize_with_token(
        &repo,
        &denylist,
        &authorizer,
        &jwks,
        &config(),
        NOW + 59,
        &minted.token,
        action,
        resource,
        BTreeMap::new(),
    );
    assert_eq!(outcome, AuthorizeOutcome::Revoked);
    assert!(!outcome.is_allow());
}

/// AC: retire is TERMINAL — the id is tombstoned, re-activation is rejected,
/// the denylist still denies the hot path, and provision cannot reuse the id
/// (AC-W-14).
#[test]
fn retire_is_terminal_and_denies() {
    let minted = mint_workload_token();
    let jwks = Jwks::new().add_key(minted.jwk.clone());
    let mut repo = InMemoryWorkloadPrincipalRepository::new();
    let mut denylist = InMemoryRevocationDenylist::new();
    let authorizer = permit_only_authorizer();
    let wl = WorkloadId::new("wl_secrets_sync").unwrap();

    provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms").expect("provision");
    activate(&mut repo, &wl).expect("activate");
    retire(&mut repo, &mut denylist, &wl).expect("retire");

    // Re-activation of a retired principal is an illegal transition.
    assert!(matches!(
        activate(&mut repo, &wl),
        Err(LifecycleError::Domain(
            iam_identity_workload_domain::WorkloadIdentityError::IllegalStateTransition {
                from: WorkloadState::Retired,
                to: WorkloadState::Active,
            }
        ))
    ));

    // The tombstone forbids re-provisioning the same id.
    assert!(matches!(
        provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms"),
        Err(LifecycleError::PrincipalAlreadyExists { .. })
    ));

    // Hot path denies (denylist + non-operational persisted state).
    let (action, resource) = decrypt_secret();
    let outcome = authorize_with_token(
        &repo,
        &denylist,
        &authorizer,
        &jwks,
        &config(),
        NOW,
        &minted.token,
        action,
        resource,
        BTreeMap::new(),
    );
    assert_eq!(outcome, AuthorizeOutcome::Revoked);
    assert!(!outcome.is_allow());
}

/// AC: an illegal lifecycle transition is rejected with a typed domain error
/// (Provisioned -> Suspended is not an edge).
#[test]
fn illegal_transition_is_rejected() {
    let mut repo = InMemoryWorkloadPrincipalRepository::new();
    let mut denylist = InMemoryRevocationDenylist::new();
    let wl = WorkloadId::new("wl_secrets_sync").unwrap();

    provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms").expect("provision");
    // Suspend straight from Provisioned is illegal.
    assert!(matches!(
        suspend(&mut repo, &mut denylist, &wl),
        Err(LifecycleError::Domain(
            iam_identity_workload_domain::WorkloadIdentityError::IllegalStateTransition {
                from: WorkloadState::Provisioned,
                to: WorkloadState::Suspended,
            }
        ))
    ));
    // And the denylist was never touched (no revoke on a rejected transition).
    assert!(denylist.is_empty());
}

/// AC: default-deny on an invalid token (PRD §3.4) — the policy engine is never
/// consulted even though a permit would otherwise match.
#[test]
fn invalid_token_is_default_deny() {
    let minted = mint_workload_token();
    let jwks = Jwks::new().add_key(minted.jwk.clone());
    let mut repo = InMemoryWorkloadPrincipalRepository::new();
    let denylist = InMemoryRevocationDenylist::new();
    let authorizer = permit_only_authorizer();
    let wl = WorkloadId::new("wl_secrets_sync").unwrap();

    provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms").expect("provision");
    activate(&mut repo, &wl).expect("activate");

    let (action, resource) = decrypt_secret();
    // Garbage token: never a valid JWS.
    let outcome = authorize_with_token(
        &repo,
        &denylist,
        &authorizer,
        &jwks,
        &config(),
        NOW,
        "not-a-valid-jwt",
        action,
        resource,
        BTreeMap::new(),
    );
    assert_eq!(outcome, AuthorizeOutcome::TokenRejected);
    assert!(!outcome.is_allow());
    assert_eq!(outcome.decision().effect(), Effect::Deny);

    // An expired-but-otherwise-valid token is also default-deny (now far past exp).
    let outcome_expired = authorize_with_token(
        &repo,
        &denylist,
        &authorizer,
        &jwks,
        &config(),
        NOW + 10_000,
        &minted.token,
        Action::new("cloud.kms.Decrypt"),
        Resource::new("Secret", "db-password"),
        BTreeMap::new(),
    );
    assert_eq!(outcome_expired, AuthorizeOutcome::TokenRejected);
}

/// AC: forbid-overrides-permit via the REAL Cedar engine (PRD §1.2 / §3.4). The
/// active principal matches the permit, but a break-glass forbid on the secret
/// wins — the decision is an explicit forbid, not a default-deny.
#[test]
fn forbid_overrides_permit_via_real_cedar() {
    let minted = mint_workload_token();
    let jwks = Jwks::new().add_key(minted.jwk.clone());
    let mut repo = InMemoryWorkloadPrincipalRepository::new();
    let denylist = InMemoryRevocationDenylist::new();
    let authorizer = permit_and_forbid_authorizer();
    let wl = WorkloadId::new("wl_secrets_sync").unwrap();

    provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms").expect("provision");
    activate(&mut repo, &wl).expect("activate");

    let (action, resource) = decrypt_secret();
    let outcome = authorize_with_token(
        &repo,
        &denylist,
        &authorizer,
        &jwks,
        &config(),
        NOW,
        &minted.token,
        action,
        resource,
        BTreeMap::new(),
    );

    let decision = match &outcome {
        AuthorizeOutcome::Decided(decision) => decision.clone(),
        other => panic!("expected a decided outcome from the engine, got {other:?}"),
    };
    assert_eq!(decision.effect(), Effect::Deny);
    assert!(matches!(
        decision.reason(),
        iam_identity_workload_domain::DecisionReason::ExplicitForbid { policy_id }
            if policy_id == "forbid-frozen-secret"
    ));
    assert!(!outcome.is_allow());
}

/// AC: an unknown subject (valid token, but no persisted principal) is a
/// default-deny — the engine is not consulted (PRD §3.3 PDP boundary).
#[test]
fn unknown_principal_is_default_deny() {
    let minted = mint_workload_token();
    let jwks = Jwks::new().add_key(minted.jwk.clone());
    let repo = InMemoryWorkloadPrincipalRepository::new(); // empty
    let denylist = InMemoryRevocationDenylist::new();
    let authorizer = permit_only_authorizer();

    let (action, resource) = decrypt_secret();
    let outcome = authorize_with_token(
        &repo,
        &denylist,
        &authorizer,
        &jwks,
        &config(),
        NOW,
        &minted.token,
        action,
        resource,
        BTreeMap::new(),
    );
    assert_eq!(outcome, AuthorizeOutcome::PrincipalUnknown);
    assert!(!outcome.is_allow());
}
