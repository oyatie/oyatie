//! Workload OIDC token validation core.
//!
//! Validates a workload's OIDC ID/access token (a compact JWS) and projects the
//! verified claims into a [`WorkloadPrincipal`] from the pure domain kernel.
//! This is the "authenticate the workload" half of the workload-identity
//! service (the Cedar authz gate is the "authorize" half).
//!
//! ## What is verified (in order)
//!
//! 1. The token is a well-formed three-segment compact JWS.
//! 2. The JOSE header declares a supported signature algorithm
//!    (`RS256`/`RS384`/`RS512` or `ES256`).
//! 3. The signature verifies against the issuer's JWK (selected by `kid`)
//!    using **ring** (BoringSSL-backed; constant-time; the blessed crypto
//!    primitive per the runtime-dependency allowlist).
//! 4. The `iss` claim equals the configured expected issuer.
//! 5. The `aud` claim contains the configured expected audience.
//! 6. `exp` is in the future and `nbf`/`iat` (if present) are not in the
//!    future, using the caller-supplied `now_epoch_seconds` (no ambient clock).
//!
//! Only after all checks pass are the claims projected into a principal.
//!
//! ## Why ring (and not a higher-level JWT crate)
//!
//! `ring` (ISC/MIT/OpenSSL license — OSI-clean) is already in the workspace
//! lockfile and is the hyperscaler-grade, audited crypto primitive named in the
//! runtime-dependency allowlist. Validating the JWS directly against `ring`
//! keeps the trusted compute base small and avoids pulling an unvetted
//! transitive tree. JWKS RSA keys arrive as `n`/`e`; we encode the PKCS#1
//! `RSAPublicKey` DER that `ring` expects (see [`rsa_pkcs1_der`]).

// ADR-0083 Tier 3: production code stays panic-free; tests may use unwrap/expect.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ring::signature;
use serde::Deserialize;

use oya_identity_workload_domain::{
    ClaimValue, WorkloadIdentityError, WorkloadPrincipal, WorkloadState,
};

/// Errors produced while validating a workload OIDC token.
#[derive(Debug)]
pub enum OidcValidationError {
    /// Token was not a three-segment compact JWS.
    MalformedToken,
    /// A segment was not valid base64url, or JSON parse failed.
    DecodeError,
    /// The JOSE `alg` is unsupported or absent.
    UnsupportedAlgorithm,
    /// No JWK matched the token's `kid` (or `kid` was absent).
    UnknownKey,
    /// A JWK was structurally invalid (bad base64url in `n`/`e`/`x`/`y`).
    MalformedKey,
    /// Cryptographic signature verification failed.
    SignatureInvalid,
    /// `iss` did not equal the expected issuer.
    IssuerMismatch,
    /// `aud` did not contain the expected audience.
    AudienceMismatch,
    /// `exp` is in the past (token expired).
    Expired,
    /// `nbf`/`iat` is in the future (token not yet valid).
    NotYetValid,
    /// A required claim for principal construction was missing/invalid.
    MissingClaim(&'static str),
    /// Projecting the claims into a domain principal failed.
    Domain(WorkloadIdentityError),
}

impl std::fmt::Display for OidcValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedToken => f.write_str("token is not a compact three-segment JWS"),
            Self::DecodeError => f.write_str("base64url/JSON decode failed"),
            Self::UnsupportedAlgorithm => f.write_str("unsupported or missing JOSE alg"),
            Self::UnknownKey => f.write_str("no JWK matched the token kid"),
            Self::MalformedKey => f.write_str("JWK is structurally invalid"),
            Self::SignatureInvalid => f.write_str("signature verification failed"),
            Self::IssuerMismatch => f.write_str("iss claim does not match expected issuer"),
            Self::AudienceMismatch => f.write_str("aud claim does not contain expected audience"),
            Self::Expired => f.write_str("token has expired (exp in the past)"),
            Self::NotYetValid => f.write_str("token is not yet valid (nbf/iat in the future)"),
            Self::MissingClaim(name) => write!(f, "required claim missing or invalid: {name}"),
            Self::Domain(error) => write!(f, "domain projection failed: {error}"),
        }
    }
}

impl std::error::Error for OidcValidationError {}

impl From<WorkloadIdentityError> for OidcValidationError {
    fn from(error: WorkloadIdentityError) -> Self {
        Self::Domain(error)
    }
}

/// Supported JWS signature algorithms. Symmetric (`HS*`) algorithms are
/// intentionally unsupported: workload tokens are issuer-signed with asymmetric
/// keys, so a shared HMAC secret would be a downgrade.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum JwsAlg {
    Rs256,
    Rs384,
    Rs512,
    Es256,
}

impl JwsAlg {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "RS256" => Some(Self::Rs256),
            "RS384" => Some(Self::Rs384),
            "RS512" => Some(Self::Rs512),
            "ES256" => Some(Self::Es256),
            _ => None,
        }
    }

    fn is_rsa(self) -> bool {
        matches!(self, Self::Rs256 | Self::Rs384 | Self::Rs512)
    }
}

/// A single JSON Web Key. Supports RSA (`n`/`e`) and EC P-256 (`x`/`y`) keys.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Jwk {
    /// Key id matched against the token's `kid`.
    pub kid: String,
    /// Key material.
    pub material: JwkMaterial,
}

/// The key-type-specific material of a [`Jwk`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JwkMaterial {
    /// RSA public key: base64url modulus `n` and exponent `e`.
    Rsa {
        /// base64url-encoded modulus.
        n: String,
        /// base64url-encoded public exponent.
        e: String,
    },
    /// EC P-256 public key: base64url affine coordinates `x` and `y`.
    EcP256 {
        /// base64url-encoded x coordinate.
        x: String,
        /// base64url-encoded y coordinate.
        y: String,
    },
}

impl Jwk {
    /// Construct an RSA JWK.
    #[must_use]
    pub fn rsa(kid: impl Into<String>, n: impl Into<String>, e: impl Into<String>) -> Self {
        Self {
            kid: kid.into(),
            material: JwkMaterial::Rsa {
                n: n.into(),
                e: e.into(),
            },
        }
    }

    /// Construct an EC P-256 JWK.
    #[must_use]
    pub fn ec_p256(kid: impl Into<String>, x: impl Into<String>, y: impl Into<String>) -> Self {
        Self {
            kid: kid.into(),
            material: JwkMaterial::EcP256 {
                x: x.into(),
                y: y.into(),
            },
        }
    }
}

/// A keyset (the issuer's JWKS) used to resolve a `kid` to a [`Jwk`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Jwks {
    keys: Vec<Jwk>,
}

impl Jwks {
    /// Build an empty keyset.
    #[must_use]
    pub fn new() -> Self {
        Self { keys: Vec::new() }
    }

    /// Build from a list of keys.
    #[must_use]
    pub fn with_keys(keys: Vec<Jwk>) -> Self {
        Self { keys }
    }

    /// Add a key (builder).
    #[must_use]
    pub fn add_key(mut self, key: Jwk) -> Self {
        self.keys.push(key);
        self
    }

    fn find(&self, kid: &str) -> Option<&Jwk> {
        self.keys.iter().find(|key| key.kid == kid)
    }
}

/// Validation policy: the expected issuer + audience and the claim names that
/// carry the workload identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationConfig {
    /// Expected `iss`.
    pub expected_issuer: String,
    /// Expected `aud` (the token's `aud` must contain this).
    pub expected_audience: String,
    /// Claim carrying the tenant id (default `tenant_id`).
    pub tenant_claim: String,
    /// Claim carrying the workload id (default `sub`).
    pub workload_claim: String,
    /// Claim carrying the owning capability id (default `owning_capability`).
    pub capability_claim: String,
    /// Claim carrying the granted scopes as a space-delimited string
    /// (default `scope`, per OAuth 2.0).
    pub scope_claim: String,
}

impl ValidationConfig {
    /// Construct a config with OAuth/OIDC-conventional claim names.
    #[must_use]
    pub fn new(expected_issuer: impl Into<String>, expected_audience: impl Into<String>) -> Self {
        Self {
            expected_issuer: expected_issuer.into(),
            expected_audience: expected_audience.into(),
            tenant_claim: "tenant_id".into(),
            workload_claim: "sub".into(),
            capability_claim: "owning_capability".into(),
            scope_claim: "scope".into(),
        }
    }
}

// --- JWT payload shape (deserialized from the verified second segment) ---

#[derive(Debug, Deserialize)]
struct JoseHeader {
    alg: String,
    #[serde(default)]
    kid: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RegisteredClaims {
    #[serde(default)]
    iss: Option<String>,
    #[serde(default)]
    aud: Option<Audience>,
    #[serde(default)]
    exp: Option<i64>,
    #[serde(default)]
    nbf: Option<i64>,
    #[serde(default)]
    iat: Option<i64>,
    #[serde(flatten)]
    rest: serde_json::Map<String, serde_json::Value>,
}

/// `aud` may be a single string or an array of strings (RFC 7519 §4.1.3).
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Audience {
    One(String),
    Many(Vec<String>),
}

impl Audience {
    fn contains(&self, expected: &str) -> bool {
        match self {
            Self::One(value) => value == expected,
            Self::Many(values) => values.iter().any(|value| value == expected),
        }
    }
}

/// Validate a compact JWS workload token and project it into a verified
/// [`WorkloadPrincipal`].
///
/// `now_epoch_seconds` is supplied by the caller — this function reads no
/// ambient clock, keeping it deterministic and testable.
///
/// # Errors
/// Returns the specific [`OidcValidationError`] for the first failing check
/// (structure, algorithm, key resolution, signature, issuer, audience,
/// expiry/nbf, required claims, or domain projection).
pub fn validate_workload_token(
    token: &str,
    jwks: &Jwks,
    config: &ValidationConfig,
    now_epoch_seconds: i64,
) -> Result<WorkloadPrincipal, OidcValidationError> {
    // 1. Split the compact JWS.
    let mut parts = token.split('.');
    let (header_b64, payload_b64, signature_b64) =
        match (parts.next(), parts.next(), parts.next(), parts.next()) {
            (Some(h), Some(p), Some(s), None)
                if !h.is_empty() && !p.is_empty() && !s.is_empty() =>
            {
                (h, p, s)
            }
            _ => return Err(OidcValidationError::MalformedToken),
        };

    // 2. Parse the JOSE header and resolve the algorithm.
    let header_bytes = b64url_decode(header_b64)?;
    let header: JoseHeader =
        serde_json::from_slice(&header_bytes).map_err(|_| OidcValidationError::DecodeError)?;
    let alg = JwsAlg::parse(&header.alg).ok_or(OidcValidationError::UnsupportedAlgorithm)?;

    // 3. Resolve the verification key by kid and verify the signature over the
    //    `header.payload` signing input.
    let kid = header
        .kid
        .as_deref()
        .ok_or(OidcValidationError::UnknownKey)?;
    let jwk = jwks.find(kid).ok_or(OidcValidationError::UnknownKey)?;
    let signature_bytes = b64url_decode(signature_b64)?;
    let signing_input = format!("{header_b64}.{payload_b64}");
    verify_signature(alg, jwk, signing_input.as_bytes(), &signature_bytes)?;

    // 4. Only now decode + validate the claims (signature is trusted).
    let payload_bytes = b64url_decode(payload_b64)?;
    let claims: RegisteredClaims =
        serde_json::from_slice(&payload_bytes).map_err(|_| OidcValidationError::DecodeError)?;

    // 5. Issuer.
    match claims.iss.as_deref() {
        Some(iss) if iss == config.expected_issuer => {}
        _ => return Err(OidcValidationError::IssuerMismatch),
    }

    // 6. Audience.
    match &claims.aud {
        Some(aud) if aud.contains(&config.expected_audience) => {}
        _ => return Err(OidcValidationError::AudienceMismatch),
    }

    // 7. Temporal validity.
    match claims.exp {
        Some(exp) if exp > now_epoch_seconds => {}
        _ => return Err(OidcValidationError::Expired),
    }
    if let Some(nbf) = claims.nbf
        && nbf > now_epoch_seconds
    {
        return Err(OidcValidationError::NotYetValid);
    }
    if let Some(iat) = claims.iat
        && iat > now_epoch_seconds
    {
        return Err(OidcValidationError::NotYetValid);
    }

    // 8. Project verified claims into a domain principal.
    project_principal(&claims, config)
}

fn project_principal(
    claims: &RegisteredClaims,
    config: &ValidationConfig,
) -> Result<WorkloadPrincipal, OidcValidationError> {
    let tenant = string_claim(&claims.rest, &config.tenant_claim)
        .ok_or(OidcValidationError::MissingClaim("tenant"))?;
    // `sub` is a top-level registered claim but, when flattened, lands in
    // `rest`; look there for the configured workload claim.
    let workload = string_claim(&claims.rest, &config.workload_claim)
        .ok_or(OidcValidationError::MissingClaim("workload"))?;
    let capability = string_claim(&claims.rest, &config.capability_claim)
        .ok_or(OidcValidationError::MissingClaim("owning_capability"))?;

    let mut principal = WorkloadPrincipal::provision(tenant, workload, capability)?;
    // A successfully-authenticated workload is activated; lifecycle revocation
    // (suspend/retire) is a separate control-plane concern.
    principal.transition_to(WorkloadState::Active)?;

    // Scopes: OAuth 2.0 space-delimited `scope` string.
    if let Some(scope_str) = string_claim(&claims.rest, &config.scope_claim) {
        for scope in scope_str.split_whitespace() {
            principal.grant_scope(scope)?;
        }
    }

    // Carry the registered + custom claims through as typed domain claims so
    // the authz layer can reason over them without re-parsing.
    if let Some(iss) = &claims.iss {
        principal.set_claim("iss", ClaimValue::Text(iss.clone()))?;
    }
    for (name, value) in &claims.rest {
        if let Some(claim) = json_to_claim(value) {
            // Skip the identity-defining claims already consumed above to avoid
            // redundant duplication; keep everything else.
            if name != &config.tenant_claim
                && name != &config.workload_claim
                && name != &config.capability_claim
            {
                principal.set_claim(name.clone(), claim)?;
            }
        }
    }

    Ok(principal)
}

fn verify_signature(
    alg: JwsAlg,
    jwk: &Jwk,
    message: &[u8],
    sig: &[u8],
) -> Result<(), OidcValidationError> {
    match (&jwk.material, alg.is_rsa()) {
        (JwkMaterial::Rsa { n, e }, true) => {
            let der = rsa_pkcs1_der(n, e)?;
            let verifier = match alg {
                JwsAlg::Rs256 => &signature::RSA_PKCS1_2048_8192_SHA256,
                JwsAlg::Rs384 => &signature::RSA_PKCS1_2048_8192_SHA384,
                JwsAlg::Rs512 => &signature::RSA_PKCS1_2048_8192_SHA512,
                JwsAlg::Es256 => unreachable!("is_rsa() guarded against EC here"),
            };
            signature::UnparsedPublicKey::new(verifier, der)
                .verify(message, sig)
                .map_err(|_| OidcValidationError::SignatureInvalid)
        }
        (JwkMaterial::EcP256 { x, y }, false) => {
            // ring expects the uncompressed SEC1 point: 0x04 || X || Y.
            let x_bytes = b64url_decode(x).map_err(|_| OidcValidationError::MalformedKey)?;
            let y_bytes = b64url_decode(y).map_err(|_| OidcValidationError::MalformedKey)?;
            if x_bytes.len() != 32 || y_bytes.len() != 32 {
                return Err(OidcValidationError::MalformedKey);
            }
            let mut point = Vec::with_capacity(65);
            point.push(0x04);
            point.extend_from_slice(&x_bytes);
            point.extend_from_slice(&y_bytes);
            // JWS ES256 uses the IEEE-P1363 fixed-width (r||s) signature form.
            signature::UnparsedPublicKey::new(&signature::ECDSA_P256_SHA256_FIXED, point)
                .verify(message, sig)
                .map_err(|_| OidcValidationError::SignatureInvalid)
        }
        // Algorithm/key-type mismatch (e.g. RS256 header with an EC key).
        _ => Err(OidcValidationError::UnsupportedAlgorithm),
    }
}

/// Encode an RSA public key as the PKCS#1 `RSAPublicKey` DER that
/// `ring`'s `RSA_PKCS1_*` verifiers accept:
///
/// ```text
/// RSAPublicKey ::= SEQUENCE { modulus INTEGER, publicExponent INTEGER }
/// ```
fn rsa_pkcs1_der(n_b64: &str, e_b64: &str) -> Result<Vec<u8>, OidcValidationError> {
    let n = b64url_decode(n_b64).map_err(|_| OidcValidationError::MalformedKey)?;
    let e = b64url_decode(e_b64).map_err(|_| OidcValidationError::MalformedKey)?;
    if n.is_empty() || e.is_empty() {
        return Err(OidcValidationError::MalformedKey);
    }
    let modulus = der_unsigned_integer(&n);
    let exponent = der_unsigned_integer(&e);
    let mut body = modulus;
    body.extend_from_slice(&exponent);
    Ok(der_sequence(&body))
}

/// DER-encode an unsigned big-endian integer (tag 0x02). A leading 0x00 is
/// prepended when the high bit is set so the value is not read as negative.
fn der_unsigned_integer(bytes: &[u8]) -> Vec<u8> {
    // Strip leading zero bytes (canonical minimal encoding) but keep at least one.
    let trimmed = {
        let first_nonzero = bytes
            .iter()
            .position(|b| *b != 0)
            .unwrap_or(bytes.len() - 1);
        &bytes[first_nonzero..]
    };
    let mut content = Vec::with_capacity(trimmed.len() + 1);
    if trimmed.first().is_some_and(|b| b & 0x80 != 0) {
        content.push(0x00);
    }
    content.extend_from_slice(trimmed);
    let mut out = vec![0x02];
    out.extend_from_slice(&der_length(content.len()));
    out.extend_from_slice(&content);
    out
}

/// DER-encode a SEQUENCE (tag 0x30) wrapping the given body.
fn der_sequence(body: &[u8]) -> Vec<u8> {
    let mut out = vec![0x30];
    out.extend_from_slice(&der_length(body.len()));
    out.extend_from_slice(body);
    out
}

/// DER definite-length encoding (short form < 128, else long form).
fn der_length(len: usize) -> Vec<u8> {
    if len < 0x80 {
        vec![len as u8]
    } else {
        let mut be = Vec::new();
        let mut value = len;
        while value > 0 {
            be.push((value & 0xff) as u8);
            value >>= 8;
        }
        be.reverse();
        let mut out = Vec::with_capacity(be.len() + 1);
        out.push(0x80 | be.len() as u8);
        out.extend_from_slice(&be);
        out
    }
}

fn b64url_decode(value: &str) -> Result<Vec<u8>, OidcValidationError> {
    URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| OidcValidationError::DecodeError)
}

fn string_claim(map: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<String> {
    map.get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

fn json_to_claim(value: &serde_json::Value) -> Option<ClaimValue> {
    match value {
        serde_json::Value::String(s) => Some(ClaimValue::Text(s.clone())),
        serde_json::Value::Bool(b) => Some(ClaimValue::Bool(*b)),
        serde_json::Value::Number(n) => n.as_i64().map(ClaimValue::Int),
        serde_json::Value::Array(items) => {
            let texts: Vec<String> = items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect();
            if texts.len() == items.len() && !items.is_empty() {
                Some(ClaimValue::TextList(texts))
            } else {
                None
            }
        }
        serde_json::Value::Null | serde_json::Value::Object(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A deterministic, self-contained signing harness: we generate an ES256
    // key with `ring` at test time, sign a token, and validate it. This proves
    // the full verify path against real crypto without any network/JWKS fetch.
    use ring::rand::SystemRandom;
    use ring::signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, KeyPair};

    fn b64url(bytes: &[u8]) -> String {
        URL_SAFE_NO_PAD.encode(bytes)
    }

    struct SignedToken {
        token: String,
        jwk: Jwk,
    }

    fn mint_es256_token(claims_json: &str, kid: &str) -> SignedToken {
        let rng = SystemRandom::new();
        let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng)
            .expect("generate pkcs8");
        let key_pair =
            EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8.as_ref(), &rng)
                .expect("load key");

        // Public point is 0x04 || X || Y (65 bytes); split into JWK coords.
        let public = key_pair.public_key().as_ref();
        assert_eq!(public.len(), 65, "uncompressed SEC1 point");
        let x = &public[1..33];
        let y = &public[33..65];

        let header = format!(r#"{{"alg":"ES256","typ":"JWT","kid":"{kid}"}}"#);
        let signing_input = format!(
            "{}.{}",
            b64url(header.as_bytes()),
            b64url(claims_json.as_bytes())
        );
        let sig = key_pair.sign(&rng, signing_input.as_bytes()).expect("sign");
        let token = format!("{signing_input}.{}", b64url(sig.as_ref()));

        SignedToken {
            token,
            jwk: Jwk::ec_p256(kid, b64url(x), b64url(y)),
        }
    }

    fn config() -> ValidationConfig {
        ValidationConfig::new("https://idp.oyatie.dev", "oya-cloud-kms")
    }

    fn valid_claims(now: i64) -> String {
        format!(
            r#"{{"iss":"https://idp.oyatie.dev","aud":"oya-cloud-kms","exp":{},"iat":{},"tenant_id":"ten_acme","sub":"wl_secrets_sync","owning_capability":"cap.cloud.kms","scope":"cloud.kms.decrypt cloud.kms.describe","mfa":true}}"#,
            now + 300,
            now
        )
    }

    #[test]
    fn valid_token_projects_to_active_principal() {
        let now = 1_700_000_000;
        let signed = mint_es256_token(&valid_claims(now), "kid-1");
        let jwks = Jwks::new().add_key(signed.jwk);

        let principal =
            validate_workload_token(&signed.token, &jwks, &config(), now).expect("valid token");

        assert_eq!(principal.tenant_id().as_str(), "ten_acme");
        assert_eq!(principal.workload_id().as_str(), "wl_secrets_sync");
        assert_eq!(principal.owning_capability().as_str(), "cap.cloud.kms");
        assert_eq!(principal.state(), WorkloadState::Active);
        assert!(principal.has_scope("cloud.kms.decrypt"));
        assert!(principal.has_scope("cloud.kms.describe"));
        assert_eq!(
            principal.claim("mfa"),
            Some(&ClaimValue::Bool(true)),
            "boolean claims are carried through as Bool, not coerced to text"
        );
        assert_eq!(
            principal.claim("iss").and_then(ClaimValue::as_text),
            Some("https://idp.oyatie.dev")
        );
    }

    #[test]
    fn tampered_payload_fails_signature() {
        let now = 1_700_000_000;
        let signed = mint_es256_token(&valid_claims(now), "kid-1");
        let jwks = Jwks::new().add_key(signed.jwk);

        // Flip a character in the payload segment to break the signature.
        let mut parts: Vec<&str> = signed.token.split('.').collect();
        let forged_payload = {
            // Re-encode an escalated-scope claim set under the same header.
            b64url(valid_claims(now).replace("decrypt", "ADMIN").as_bytes())
        };
        parts[1] = &forged_payload;
        let forged = parts.join(".");

        let err = validate_workload_token(&forged, &jwks, &config(), now).expect_err("must reject");
        assert!(matches!(err, OidcValidationError::SignatureInvalid));
    }

    #[test]
    fn expired_token_is_rejected() {
        let now = 1_700_000_000;
        let expired = format!(
            r#"{{"iss":"https://idp.oyatie.dev","aud":"oya-cloud-kms","exp":{},"tenant_id":"ten_acme","sub":"wl_a","owning_capability":"cap.x.y"}}"#,
            now - 10
        );
        let signed = mint_es256_token(&expired, "kid-1");
        let jwks = Jwks::new().add_key(signed.jwk);
        let err = validate_workload_token(&signed.token, &jwks, &config(), now)
            .expect_err("must reject expired");
        assert!(matches!(err, OidcValidationError::Expired));
    }

    #[test]
    fn wrong_audience_is_rejected() {
        let now = 1_700_000_000;
        let claims = format!(
            r#"{{"iss":"https://idp.oyatie.dev","aud":"some-other-service","exp":{},"tenant_id":"ten_acme","sub":"wl_a","owning_capability":"cap.x.y"}}"#,
            now + 300
        );
        let signed = mint_es256_token(&claims, "kid-1");
        let jwks = Jwks::new().add_key(signed.jwk);
        let err = validate_workload_token(&signed.token, &jwks, &config(), now)
            .expect_err("must reject audience");
        assert!(matches!(err, OidcValidationError::AudienceMismatch));
    }

    #[test]
    fn wrong_issuer_is_rejected() {
        let now = 1_700_000_000;
        let claims = format!(
            r#"{{"iss":"https://evil.example","aud":"oya-cloud-kms","exp":{},"tenant_id":"ten_acme","sub":"wl_a","owning_capability":"cap.x.y"}}"#,
            now + 300
        );
        let signed = mint_es256_token(&claims, "kid-1");
        let jwks = Jwks::new().add_key(signed.jwk);
        let err = validate_workload_token(&signed.token, &jwks, &config(), now)
            .expect_err("must reject issuer");
        assert!(matches!(err, OidcValidationError::IssuerMismatch));
    }

    #[test]
    fn unknown_kid_is_rejected() {
        let now = 1_700_000_000;
        let signed = mint_es256_token(&valid_claims(now), "kid-1");
        // JWKS only has a DIFFERENT kid.
        let other = mint_es256_token(&valid_claims(now), "kid-2");
        let jwks = Jwks::new().add_key(other.jwk);
        let err = validate_workload_token(&signed.token, &jwks, &config(), now)
            .expect_err("must reject unknown kid");
        assert!(matches!(err, OidcValidationError::UnknownKey));
    }

    #[test]
    fn malformed_token_is_rejected() {
        let jwks = Jwks::new();
        assert!(matches!(
            validate_workload_token("not-a-jwt", &jwks, &config(), 0),
            Err(OidcValidationError::MalformedToken)
        ));
        assert!(matches!(
            validate_workload_token("only.two", &jwks, &config(), 0),
            Err(OidcValidationError::MalformedToken)
        ));
    }

    #[test]
    fn der_integer_prepends_zero_for_high_bit() {
        // 0x80 has the high bit set → must be prefixed with 0x00.
        let encoded = der_unsigned_integer(&[0x80]);
        assert_eq!(encoded, vec![0x02, 0x02, 0x00, 0x80]);
        // 0x7f does not → no prefix.
        let encoded = der_unsigned_integer(&[0x7f]);
        assert_eq!(encoded, vec![0x02, 0x01, 0x7f]);
    }

    #[test]
    fn der_length_long_form() {
        assert_eq!(der_length(0x7f), vec![0x7f]);
        assert_eq!(der_length(0x80), vec![0x81, 0x80]);
        assert_eq!(der_length(0x0102), vec![0x82, 0x01, 0x02]);
    }
}
