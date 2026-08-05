//! EdDSA/Ed25519 (RFC 8037) signing harness and deterministic tests.
//!
//! This module is **test-only** (`#[cfg(test)]`). It mirrors the ES256 signing
//! harness in `lib.rs` but for OKP/Ed25519 keys: generates a fresh
//! `Ed25519KeyPair` via `aws-lc-rs`, signs a compact JWS, and validates it
//! through the full `validate_workload_token` pipeline.
//!
//! The harness is deterministic in the sense that each test generates its own
//! ephemeral key pair — no shared mutable state, no ambient clock reads.

use aws_lc_rs::signature::{Ed25519KeyPair, KeyPair};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

use crate::{Jwk, Jwks, OidcValidationError, ValidationConfig, validate_workload_token};

fn b64url(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Mint a genuine EdDSA/Ed25519 JWS for `claims_json`.
///
/// Returns the compact token string and the matching [`Jwk`] to publish in
/// the JWKS. The key pair is freshly generated for each call.
pub(crate) fn mint_ed25519_token(claims_json: &str, kid: &str) -> (String, Jwk) {
    let key_pair = Ed25519KeyPair::generate().expect("Ed25519 key generation");
    // The public key is the raw 32-byte Edwards point.
    let pub_bytes = key_pair.public_key().as_ref();
    assert_eq!(pub_bytes.len(), 32, "Ed25519 public key must be 32 bytes");

    let header = format!(r#"{{"alg":"EdDSA","typ":"JWT","kid":"{kid}"}}"#);
    let signing_input = format!(
        "{}.{}",
        b64url(header.as_bytes()),
        b64url(claims_json.as_bytes())
    );
    // Ed25519KeyPair::sign takes no rng — deterministic signing per RFC 8032.
    let sig = key_pair.sign(signing_input.as_bytes());
    let token = format!("{signing_input}.{}", b64url(sig.as_ref()));

    let jwk = Jwk::okp_ed25519(kid, b64url(pub_bytes));
    (token, jwk)
}

fn config() -> ValidationConfig {
    ValidationConfig::new("https://idp.oyatie.com", "oya-cloud-kms")
}

fn valid_claims(now: i64) -> String {
    format!(
        r#"{{"iss":"https://idp.oyatie.com","aud":"oya-cloud-kms","exp":{},"iat":{},"tenant_id":"ten_acme","sub":"wl_secrets_sync","owning_capability":"cap.cloud.kms","scope":"cloud.kms.decrypt cloud.kms.describe","mfa":true}}"#,
        now + 300,
        now
    )
}

#[test]
fn valid_ed25519_token_projects_to_active_principal() {
    use oya_identity_workload_domain::{ClaimValue, WorkloadState};

    let now: i64 = 1_700_000_000;
    let (token, jwk) = mint_ed25519_token(&valid_claims(now), "kid-ed-1");
    let jwks = Jwks::new().add_key(jwk);

    let principal =
        validate_workload_token(&token, &jwks, &config(), now).expect("valid EdDSA token");

    assert_eq!(principal.tenant_id().as_str(), "ten_acme");
    assert_eq!(principal.workload_id().as_str(), "wl_secrets_sync");
    assert_eq!(principal.owning_capability().as_str(), "cap.cloud.kms");
    assert_eq!(principal.trust_domain().as_str(), "spiffe://ten_acme");
    assert_eq!(principal.state(), WorkloadState::Active);
    assert!(principal.has_scope("cloud.kms.decrypt"));
    assert!(principal.has_scope("cloud.kms.describe"));
    assert_eq!(principal.claim("mfa"), Some(&ClaimValue::Bool(true)));
}

#[test]
fn tampered_ed25519_payload_fails_signature() {
    let now: i64 = 1_700_000_000;
    let (token, jwk) = mint_ed25519_token(&valid_claims(now), "kid-ed-1");
    let jwks = Jwks::new().add_key(jwk);

    // Forge the payload segment (escalate scope).
    let mut parts: Vec<&str> = token.split('.').collect();
    let forged_payload = b64url(valid_claims(now).replace("decrypt", "ADMIN").as_bytes());
    parts[1] = &forged_payload;
    let forged = parts.join(".");

    let err =
        validate_workload_token(&forged, &jwks, &config(), now).expect_err("tampered must fail");
    assert!(matches!(err, OidcValidationError::SignatureInvalid));
}

#[test]
fn eddsa_against_rsa_kid_is_algorithm_mismatch() {
    use crate::Jwk;

    let now: i64 = 1_700_000_000;
    let (token, _okp_jwk) = mint_ed25519_token(&valid_claims(now), "kid-ed-1");

    // RSA JWK under the same kid: family mismatch should fire.
    // Use small (clearly synthetic) n/e values; the mismatch is caught before
    // any signature verification.
    let rsa_jwk = Jwk::rsa("kid-ed-1", "AAAA", "AQAB");
    let jwks = Jwks::new().add_key(rsa_jwk);

    let err = validate_workload_token(&token, &jwks, &config(), now)
        .expect_err("EdDSA against RSA key must be mismatch");
    assert!(matches!(err, OidcValidationError::AlgorithmMismatch));
}

#[test]
fn eddsa_against_ec_kid_is_algorithm_mismatch() {
    use crate::Jwk;

    let now: i64 = 1_700_000_000;
    let (token, _okp_jwk) = mint_ed25519_token(&valid_claims(now), "kid-ed-1");

    // EC P-256 JWK under the same kid.
    let ec_jwk = Jwk::ec_p256("kid-ed-1", "AAAA", "BBBB");
    let jwks = Jwks::new().add_key(ec_jwk);

    let err = validate_workload_token(&token, &jwks, &config(), now)
        .expect_err("EdDSA against EC key must be mismatch");
    assert!(matches!(err, OidcValidationError::AlgorithmMismatch));
}

#[test]
fn rsa_token_against_okp_kid_is_algorithm_mismatch() {
    // An RS256 header token presented against an OKP JWK must be refused.
    use aws_lc_rs::rand::SystemRandom;
    use aws_lc_rs::signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, KeyPair as _};

    let now: i64 = 1_700_000_000;

    // Mint a genuine ES256 token (not RS256, but non-OKP family is the point).
    let rng = SystemRandom::new();
    let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng)
        .expect("generate pkcs8");
    let ec_key = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8.as_ref())
        .expect("load ec key");
    let _public = ec_key.public_key().as_ref();
    let header = r#"{"alg":"ES256","typ":"JWT","kid":"kid-ed-1"}"#.to_owned();
    let signing_input = format!(
        "{}.{}",
        b64url(header.as_bytes()),
        b64url(valid_claims(now).as_bytes())
    );
    let sig = ec_key.sign(&rng, signing_input.as_bytes()).expect("sign");
    let token = format!("{signing_input}.{}", b64url(sig.as_ref()));

    // But present an OKP JWK for that kid.
    let key_pair = Ed25519KeyPair::generate().expect("Ed25519 generate");
    let okp_jwk = Jwk::okp_ed25519("kid-ed-1", b64url(key_pair.public_key().as_ref()));
    let jwks = Jwks::new().add_key(okp_jwk);

    let err = validate_workload_token(&token, &jwks, &config(), now)
        .expect_err("EC token against OKP key must be mismatch");
    assert!(matches!(err, OidcValidationError::AlgorithmMismatch));
}

#[test]
fn ed25519_alg_pin_mismatch_is_rejected() {
    let now: i64 = 1_700_000_000;
    let (token, jwk) = mint_ed25519_token(&valid_claims(now), "kid-ed-1");

    // Pin the JWK to RS256 — the EdDSA token must be refused.
    let pinned = jwk.with_alg("RS256");
    let jwks = Jwks::new().add_key(pinned);

    let err = validate_workload_token(&token, &jwks, &config(), now)
        .expect_err("alg pin mismatch must be rejected");
    assert!(matches!(err, OidcValidationError::AlgorithmMismatch));
}

#[test]
fn ed25519_alg_pin_match_is_accepted() {
    let now: i64 = 1_700_000_000;
    let (token, jwk) = mint_ed25519_token(&valid_claims(now), "kid-ed-1");

    // Pinning the JWK to the exact alg that the token also uses must succeed.
    let pinned = jwk.with_alg("EdDSA");
    let jwks = Jwks::new().add_key(pinned);

    validate_workload_token(&token, &jwks, &config(), now).expect("EdDSA pin match must succeed");
}

#[test]
fn malformed_okp_x_coord_is_rejected() {
    // A JWK whose x is not 32 bytes must be refused as MalformedKey.
    let now: i64 = 1_700_000_000;
    let (token, _jwk) = mint_ed25519_token(&valid_claims(now), "kid-ed-1");

    // Provide a JWK with only 16 bytes of x (invalid — must be 32).
    let short_x = b64url(&[0u8; 16]);
    let bad_jwk = Jwk::okp_ed25519("kid-ed-1", short_x);
    let jwks = Jwks::new().add_key(bad_jwk);

    let err = validate_workload_token(&token, &jwks, &config(), now)
        .expect_err("malformed OKP key must be rejected");
    assert!(matches!(err, OidcValidationError::MalformedKey));
}
