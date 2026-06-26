//! Hermetic end-to-end tests for the JWT/OIDC [`PrincipalVerifier`].
//!
//! All keys are minted in-process with `aws-lc-rs`; the JWKS is built from the
//! freshly generated public key and the JWT is signed with the matching private
//! key. No network, no clock dependence (time is injected via `verify_at`), no
//! checked-in vectors — the positive path proves the same code that the
//! negative paths attack.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use aws_lc_rs::rand::SystemRandom;
use aws_lc_rs::signature::{
    ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, KeyPair, RSA_PKCS1_SHA256, RsaKeyPair,
};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use intelligence_authn_jwt_adapter::{JwksError, JwtPrincipalVerifier, VerifierConfig};
use intelligence_kernel::{AuthnError, PrincipalVerifier};
use serde_json::{Value, json};

const ISS: &str = "https://iam.cloud.example/realms/oyatie";
const AUD: &str = "cloud-intelligence";
const NOW: u64 = 1_900_000_000;

fn b64u(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

fn mint(header: &Value, claims: &Value, sign: impl Fn(&[u8]) -> Vec<u8>) -> String {
    let h = b64u(&serde_json::to_vec(header).unwrap());
    let p = b64u(&serde_json::to_vec(claims).unwrap());
    let signing_input = format!("{h}.{p}");
    let sig = sign(signing_input.as_bytes());
    format!("{signing_input}.{}", b64u(&sig))
}

fn good_claims() -> Value {
    json!({
        "iss": ISS,
        "aud": AUD,
        "sub": "agent-7",
        "tenant_id": "acme",
        "exp": NOW + 300,
        "nbf": NOW - 10,
        "iat": NOW - 10,
    })
}

fn config() -> VerifierConfig {
    VerifierConfig::new(ISS, AUD)
}

// --- ES256 key material -----------------------------------------------------

struct EcKey {
    kp: EcdsaKeyPair,
    kid: String,
}

impl EcKey {
    fn generate(kid: &str) -> Self {
        let kp = EcdsaKeyPair::generate(&ECDSA_P256_SHA256_FIXED_SIGNING).unwrap();
        Self {
            kp,
            kid: kid.to_string(),
        }
    }

    /// JWK with a pinned `alg: ES256`.
    fn jwk(&self) -> Value {
        let pk = self.kp.public_key().as_ref();
        assert_eq!(pk.len(), 65, "P-256 uncompressed point");
        assert_eq!(pk[0], 0x04);
        json!({
            "kty": "EC",
            "crv": "P-256",
            "kid": self.kid,
            "alg": "ES256",
            "use": "sig",
            "x": b64u(&pk[1..33]),
            "y": b64u(&pk[33..65]),
        })
    }

    fn sign(&self, msg: &[u8]) -> Vec<u8> {
        let rng = SystemRandom::new();
        self.kp.sign(&rng, msg).unwrap().as_ref().to_vec()
    }

    fn header(&self) -> Value {
        json!({ "alg": "ES256", "kid": self.kid, "typ": "JWT" })
    }
}

fn jwks(keys: &[Value]) -> String {
    serde_json::to_string(&json!({ "keys": keys })).unwrap()
}

// --- RSA (RS256) key material ----------------------------------------------

struct RsaKey {
    kp: RsaKeyPair,
    kid: String,
}

impl RsaKey {
    fn generate(kid: &str) -> Self {
        let kp = RsaKeyPair::generate(aws_lc_rs::rsa::KeySize::Rsa2048).unwrap();
        Self {
            kp,
            kid: kid.to_string(),
        }
    }

    fn sign(&self, msg: &[u8]) -> Vec<u8> {
        let rng = SystemRandom::new();
        let mut sig = vec![0u8; self.kp.public_modulus_len()];
        self.kp.sign(&RSA_PKCS1_SHA256, &rng, msg, &mut sig).unwrap();
        sig
    }

    /// JWK from the PKCS#1 DER public key. `with_alg` controls whether the JWK
    /// pins `alg: RS256` (so we can exercise the unpinned algorithm-family path).
    fn jwk(&self, with_alg: bool) -> Value {
        let der = self.kp.public_key().as_ref();
        let (n, e) = parse_pkcs1_rsa_public(der);
        let mut v = json!({
            "kty": "RSA",
            "kid": self.kid,
            "use": "sig",
            "n": b64u(&n),
            "e": b64u(&e),
        });
        if with_alg {
            v["alg"] = json!("RS256");
        }
        v
    }

    fn header(&self) -> Value {
        json!({ "alg": "RS256", "kid": self.kid, "typ": "JWT" })
    }
}

/// Minimal DER walker for `RSAPublicKey ::= SEQUENCE { INTEGER n, INTEGER e }`
/// (RFC 8017 / PKCS#1). Strips the sign-byte leading zero so `n`/`e` are the
/// minimal big-endian octet strings JWKs use.
fn parse_pkcs1_rsa_public(der: &[u8]) -> (Vec<u8>, Vec<u8>) {
    assert_eq!(der[0], 0x30, "SEQUENCE");
    let (_seq_len, mut i) = read_der_len(der, 1);
    // INTEGER n
    assert_eq!(der[i], 0x02, "INTEGER n");
    i += 1;
    let (n_len, ni) = read_der_len(der, i);
    i = ni;
    let mut n = der[i..i + n_len].to_vec();
    i += n_len;
    // INTEGER e
    assert_eq!(der[i], 0x02, "INTEGER e");
    i += 1;
    let (e_len, ei) = read_der_len(der, i);
    i = ei;
    let mut e = der[i..i + e_len].to_vec();
    while n.first() == Some(&0) {
        n.remove(0);
    }
    while e.first() == Some(&0) {
        e.remove(0);
    }
    (n, e)
}

fn read_der_len(b: &[u8], i: usize) -> (usize, usize) {
    let first = b[i];
    if first < 0x80 {
        (first as usize, i + 1)
    } else {
        let nbytes = (first & 0x7f) as usize;
        let mut len = 0usize;
        for k in 0..nbytes {
            len = (len << 8) | b[i + 1 + k] as usize;
        }
        (len, i + 1 + nbytes)
    }
}

// ===========================================================================
// Positive paths
// ===========================================================================

#[test]
fn es256_happy_path_extracts_tenant_and_agent() {
    let key = EcKey::generate("ec-1");
    let verifier = JwtPrincipalVerifier::from_jwks_json(&jwks(&[key.jwk()]), config()).unwrap();
    let token = mint(&key.header(), &good_claims(), |m| key.sign(m));

    let vp = verifier.verify_at(&token, NOW).unwrap();
    assert_eq!(vp.tenant.as_str(), "acme");
    assert_eq!(vp.agent.as_str(), "agent-7");
    assert_eq!(vp.subject, "agent-7");
    assert_eq!(vp.issuer, ISS);
    assert_eq!(vp.expires_at_unix, NOW + 300);
}

#[test]
fn rs256_happy_path() {
    let key = RsaKey::generate("rsa-1");
    let verifier =
        JwtPrincipalVerifier::from_jwks_json(&jwks(&[key.jwk(true)]), config()).unwrap();
    let token = mint(&key.header(), &good_claims(), |m| key.sign(m));

    let vp = verifier.verify_at(&token, NOW).unwrap();
    assert_eq!(vp.tenant.as_str(), "acme");
    assert_eq!(vp.agent.as_str(), "agent-7");
}

#[test]
fn aud_array_with_match_is_accepted() {
    let key = EcKey::generate("ec-1");
    let verifier = JwtPrincipalVerifier::from_jwks_json(&jwks(&[key.jwk()]), config()).unwrap();
    let mut claims = good_claims();
    claims["aud"] = json!(["someone-else", AUD]);
    let token = mint(&key.header(), &claims, |m| key.sign(m));
    assert!(verifier.verify_at(&token, NOW).is_ok());
}

#[test]
fn exp_just_inside_leeway_is_accepted() {
    let key = EcKey::generate("ec-1");
    let verifier =
        JwtPrincipalVerifier::from_jwks_json(&jwks(&[key.jwk()]), config().with_leeway_secs(60))
            .unwrap();
    let mut claims = good_claims();
    claims["exp"] = json!(NOW - 30); // expired 30s ago, within 60s leeway
    let token = mint(&key.header(), &claims, |m| key.sign(m));
    assert!(verifier.verify_at(&token, NOW).is_ok());
}

#[test]
fn token_without_kid_against_single_key_is_accepted() {
    let key = EcKey::generate("ec-1");
    let verifier = JwtPrincipalVerifier::from_jwks_json(&jwks(&[key.jwk()]), config()).unwrap();
    let header = json!({ "alg": "ES256", "typ": "JWT" }); // no kid
    let token = mint(&header, &good_claims(), |m| key.sign(m));
    assert!(verifier.verify_at(&token, NOW).is_ok());
}

#[test]
fn custom_tenant_claim_is_honored() {
    let key = EcKey::generate("ec-1");
    let cfg = config().with_tenant_claim("tid");
    let verifier = JwtPrincipalVerifier::from_jwks_json(&jwks(&[key.jwk()]), cfg).unwrap();
    let mut claims = good_claims();
    claims.as_object_mut().unwrap().remove("tenant_id");
    claims["tid"] = json!("globex");
    let token = mint(&key.header(), &claims, |m| key.sign(m));
    assert_eq!(verifier.verify_at(&token, NOW).unwrap().tenant.as_str(), "globex");
}

// ===========================================================================
// Negative paths — every one is a deny
// ===========================================================================

fn verifier_es256() -> (EcKey, JwtPrincipalVerifier) {
    let key = EcKey::generate("ec-1");
    let v = JwtPrincipalVerifier::from_jwks_json(&jwks(&[key.jwk()]), config()).unwrap();
    (key, v)
}

#[test]
fn empty_token_is_missing() {
    let (_k, v) = verifier_es256();
    assert_eq!(v.verify_at("   ", NOW), Err(AuthnError::MissingToken));
    assert_eq!(v.verify("").err(), Some(AuthnError::MissingToken));
}

#[test]
fn malformed_segments_are_rejected() {
    let (_k, v) = verifier_es256();
    assert_eq!(v.verify_at("only.two", NOW), Err(AuthnError::MalformedToken));
    assert_eq!(v.verify_at("a.b.c.d", NOW), Err(AuthnError::MalformedToken));
    assert_eq!(v.verify_at("a..c", NOW), Err(AuthnError::MalformedToken));
    assert_eq!(
        v.verify_at("not-base64!.x.y", NOW),
        Err(AuthnError::MalformedToken)
    );
}

#[test]
fn alg_none_is_unsupported() {
    let (_k, v) = verifier_es256();
    let header = b64u(br#"{"alg":"none","kid":"ec-1"}"#);
    let payload = b64u(&serde_json::to_vec(&good_claims()).unwrap());
    // Non-empty (but irrelevant) signature segment so it parses structurally.
    let token = format!("{header}.{payload}.AAAA");
    assert_eq!(v.verify_at(&token, NOW), Err(AuthnError::UnsupportedAlgorithm));
}

#[test]
fn hs256_symmetric_alg_is_unsupported() {
    let (_k, v) = verifier_es256();
    let header = b64u(br#"{"alg":"HS256","kid":"ec-1"}"#);
    let payload = b64u(&serde_json::to_vec(&good_claims()).unwrap());
    let token = format!("{header}.{payload}.AAAA");
    assert_eq!(v.verify_at(&token, NOW), Err(AuthnError::UnsupportedAlgorithm));
}

#[test]
fn unrecognized_crit_header_is_rejected() {
    // RFC 7515 §4.1.11: a `crit` header names extensions the recipient MUST
    // understand. This adapter implements none, so ANY crit entry is unrecognized
    // and the token must be rejected — even when the signature is otherwise valid.
    // Signed with the real key so this proves crit is rejected on its own merits,
    // not as a side effect of a signature/claim failure.
    let (key, v) = verifier_es256();
    let header = json!({ "alg": "ES256", "kid": "ec-1", "typ": "JWT", "crit": ["exp"], "exp": true });
    let token = mint(&header, &good_claims(), |m| key.sign(m));
    assert_eq!(v.verify_at(&token, NOW), Err(AuthnError::MalformedToken));
}

#[test]
fn empty_crit_header_is_rejected() {
    // RFC 7515 §4.1.11: `crit` MUST NOT be empty if present. A producer that
    // emits `crit: []` is malformed — fail closed rather than treat it as absent.
    let (key, v) = verifier_es256();
    let header = json!({ "alg": "ES256", "kid": "ec-1", "typ": "JWT", "crit": [] });
    let token = mint(&header, &good_claims(), |m| key.sign(m));
    assert_eq!(v.verify_at(&token, NOW), Err(AuthnError::MalformedToken));
}

#[test]
fn unknown_kid_is_rejected() {
    let (key, v) = verifier_es256();
    let header = json!({ "alg": "ES256", "kid": "other-kid", "typ": "JWT" });
    let token = mint(&header, &good_claims(), |m| key.sign(m));
    assert_eq!(v.verify_at(&token, NOW), Err(AuthnError::UnknownKeyId));
}

#[test]
fn tampered_signature_is_rejected() {
    let (key, v) = verifier_es256();
    let token = mint(&key.header(), &good_claims(), |m| key.sign(m));
    // Flip the FIRST char of the signature segment. The final char of a
    // fixed-length ECDSA signature encodes only 2 significant bits (the low 4 are
    // canonical-zero padding), so flipping it can yield a non-canonical
    // base64url that decodes as MalformedToken rather than SignatureInvalid ~25%
    // of the time. The first char is fully significant: it always decodes, and a
    // one-byte change to `r` always breaks ECDSA verification deterministically.
    let (rest, sig) = token.rsplit_once('.').unwrap();
    let repl = if sig.starts_with('A') { 'B' } else { 'A' };
    let t = format!("{rest}.{repl}{}", &sig[1..]);
    assert_eq!(v.verify_at(&t, NOW), Err(AuthnError::SignatureInvalid));
}

#[test]
fn signature_from_a_different_key_is_rejected() {
    let (_key, v) = verifier_es256();
    let attacker = EcKey::generate("ec-1"); // same kid, different key
    let token = mint(&attacker.header(), &good_claims(), |m| attacker.sign(m));
    assert_eq!(v.verify_at(&token, NOW), Err(AuthnError::SignatureInvalid));
}

#[test]
fn expired_token_is_rejected() {
    let (key, v) = verifier_es256();
    let mut claims = good_claims();
    claims["exp"] = json!(NOW - 1000);
    let token = mint(&key.header(), &claims, |m| key.sign(m));
    assert_eq!(v.verify_at(&token, NOW), Err(AuthnError::Expired));
}

#[test]
fn not_yet_valid_token_is_rejected() {
    let (key, v) = verifier_es256();
    let mut claims = good_claims();
    claims["nbf"] = json!(NOW + 1000);
    let token = mint(&key.header(), &claims, |m| key.sign(m));
    assert_eq!(v.verify_at(&token, NOW), Err(AuthnError::NotYetValid));
}

#[test]
fn wrong_issuer_is_rejected() {
    let (key, v) = verifier_es256();
    let mut claims = good_claims();
    claims["iss"] = json!("https://evil.example/realms/attacker");
    let token = mint(&key.header(), &claims, |m| key.sign(m));
    assert_eq!(v.verify_at(&token, NOW), Err(AuthnError::IssuerMismatch));
}

#[test]
fn wrong_audience_is_rejected() {
    let (key, v) = verifier_es256();
    let mut claims = good_claims();
    claims["aud"] = json!("some-other-service");
    let token = mint(&key.header(), &claims, |m| key.sign(m));
    assert_eq!(v.verify_at(&token, NOW), Err(AuthnError::AudienceMismatch));
}

#[test]
fn missing_exp_is_rejected() {
    let (key, v) = verifier_es256();
    let mut claims = good_claims();
    claims.as_object_mut().unwrap().remove("exp");
    let token = mint(&key.header(), &claims, |m| key.sign(m));
    assert_eq!(v.verify_at(&token, NOW), Err(AuthnError::MissingClaim("exp")));
}

#[test]
fn missing_tenant_is_rejected() {
    let (key, v) = verifier_es256();
    let mut claims = good_claims();
    claims.as_object_mut().unwrap().remove("tenant_id");
    let token = mint(&key.header(), &claims, |m| key.sign(m));
    assert_eq!(v.verify_at(&token, NOW), Err(AuthnError::MissingClaim("tenant")));
}

#[test]
fn missing_sub_is_rejected() {
    let (key, v) = verifier_es256();
    let mut claims = good_claims();
    claims.as_object_mut().unwrap().remove("sub");
    let token = mint(&key.header(), &claims, |m| key.sign(m));
    assert_eq!(v.verify_at(&token, NOW), Err(AuthnError::MissingClaim("sub")));
}

#[test]
fn pinned_alg_mismatch_is_unsupported() {
    // JWK pins ES256; token header claims ES384 (still signed by the ES256 key).
    let (key, v) = verifier_es256();
    let header = json!({ "alg": "ES384", "kid": "ec-1", "typ": "JWT" });
    let token = mint(&header, &good_claims(), |m| key.sign(m));
    assert_eq!(v.verify_at(&token, NOW), Err(AuthnError::UnsupportedAlgorithm));
}

#[test]
fn algorithm_family_confusion_is_rejected() {
    // RSA JWK WITHOUT a pinned alg; token header claims ES256. The key type
    // (RSA) does not match the asserted EC algorithm — must fail closed.
    let key = RsaKey::generate("rsa-1");
    let v = JwtPrincipalVerifier::from_jwks_json(&jwks(&[key.jwk(false)]), config()).unwrap();
    let header = json!({ "alg": "ES256", "kid": "rsa-1", "typ": "JWT" });
    let token = mint(&header, &good_claims(), |m| key.sign(m));
    assert_eq!(v.verify_at(&token, NOW), Err(AuthnError::UnsupportedAlgorithm));
}

// ===========================================================================
// Construction-time (JWKS) errors
// ===========================================================================

#[test]
fn malformed_jwks_json_is_rejected() {
    let err = JwtPrincipalVerifier::from_jwks_json("{ not json", config()).unwrap_err();
    assert!(matches!(err, JwksError::MalformedDocument(_)));
}

#[test]
fn empty_jwks_is_rejected() {
    let err = JwtPrincipalVerifier::from_jwks_json(r#"{"keys":[]}"#, config()).unwrap_err();
    assert_eq!(err, JwksError::Empty);
}

#[test]
fn unusable_jwk_is_rejected() {
    let bad = r#"{"keys":[{"kty":"EC","crv":"P-256","kid":"x","x":"!!!","y":"!!!"}]}"#;
    let err = JwtPrincipalVerifier::from_jwks_json(bad, config()).unwrap_err();
    assert!(matches!(err, JwksError::UnusableKey(_)));
}

#[test]
fn invalid_config_is_rejected() {
    let key = EcKey::generate("ec-1");
    let err =
        JwtPrincipalVerifier::from_jwks_json(&jwks(&[key.jwk()]), VerifierConfig::new("", AUD))
            .unwrap_err();
    assert!(matches!(err, JwksError::InvalidConfig(_)));
}

#[test]
fn enc_use_keys_are_skipped() {
    // A JWKS whose only key is marked use:enc has no usable signing key.
    let key = EcKey::generate("ec-1");
    let mut jwk = key.jwk();
    jwk["use"] = json!("enc");
    let err = JwtPrincipalVerifier::from_jwks_json(&jwks(&[jwk]), config()).unwrap_err();
    assert_eq!(err, JwksError::Empty);
}

#[test]
fn duplicate_kid_in_jwks_fails_closed() {
    // Two DIFFERENT keys sharing one kid. HashMap::insert last-wins would let the
    // second silently shadow the first — an attacker who slips a key with a
    // colliding kid into the JWKS could displace the genuine signer. Reject the
    // whole document instead of guessing which key is authoritative.
    let k1 = EcKey::generate("dup");
    let k2 = EcKey::generate("dup");
    let err = JwtPrincipalVerifier::from_jwks_json(&jwks(&[k1.jwk(), k2.jwk()]), config())
        .unwrap_err();
    assert_eq!(err, JwksError::DuplicateKeyId("dup".to_string()));
}

#[test]
fn ambiguous_keyless_multi_key_without_kid_is_rejected() {
    // Two keyless keys + token without kid → ambiguous → fail closed.
    let k1 = EcKey::generate("a");
    let k2 = EcKey::generate("b");
    let mut j1 = k1.jwk();
    let mut j2 = k2.jwk();
    j1.as_object_mut().unwrap().remove("kid");
    j2.as_object_mut().unwrap().remove("kid");
    let v = JwtPrincipalVerifier::from_jwks_json(&jwks(&[j1, j2]), config()).unwrap();
    let header = json!({ "alg": "ES256", "typ": "JWT" }); // no kid
    let token = mint(&header, &good_claims(), |m| k1.sign(m));
    assert_eq!(v.verify_at(&token, NOW), Err(AuthnError::UnknownKeyId));
}
