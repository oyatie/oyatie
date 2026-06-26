//! JWT/OIDC adapter implementing the kernel's [`PrincipalVerifier`] port —
//! fail-closed authentication against a cloud-iam-issued JWKS.
//!
//! This is the I/O side of the authn seam: it owns the verification mechanics
//! (compact-JWS parsing, JWKS key selection, asymmetric signature verification,
//! and time/issuer/audience claim checks) that the pure kernel port deliberately
//! does not. cloud-iam is the issuing IdP; this adapter only *validates* tokens
//! cloud-iam minted — it is not a parallel identity provider.
//!
//! ## Owned-stack crypto
//!
//! Signature verification runs on `aws-lc-rs`, Oyatie's ADR-0506 canonical
//! crypto backend — no `jsonwebtoken`/`jose` dependency is pulled. The JWS/JWKS
//! *protocol* mechanics are reimplemented here behind the kernel port (cutover
//! litmus: the port shape does not change when the backend does), rather than
//! adopting an upstream crate's structure.
//!
//! ## Accepted algorithms (asymmetric only)
//!
//! `RS256`, `RS384`, `RS512` (RSA PKCS#1 v1.5) and `ES256`, `ES384` (ECDSA,
//! fixed-format signatures). Symmetric (`HS*`) and `none` are rejected as a
//! class — an attacker cannot downgrade a JWKS-signed token to a MAC the
//! verifier would check with public key material (algorithm-confusion defense).
//!
//! ## Fail-closed contract
//!
//! Every error path returns an [`AuthnError`] (a deny). The signature is
//! verified *before* any claim is trusted. Verification uses `aws-lc-rs`, whose
//! signature check is constant-time; the issuer/audience string comparisons are
//! over non-secret values, so timing-safe compares are not applicable there.
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use aws_lc_rs::signature::{
    ECDSA_P256_SHA256_FIXED, ECDSA_P384_SHA384_FIXED, RSA_PKCS1_2048_8192_SHA256,
    RSA_PKCS1_2048_8192_SHA384, RSA_PKCS1_2048_8192_SHA512, RsaPublicKeyComponents,
    UnparsedPublicKey,
};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use intelligence_kernel::{AgentId, AuthnError, PrincipalVerifier, TenantId, VerifiedPrincipal};
use serde::Deserialize;
use serde_json::Value;

/// Errors raised while *constructing* a verifier (parsing a JWKS document or
/// config). Distinct from [`AuthnError`]: these surface at wiring time, never on
/// the request path. Once built, every request-path failure is an [`AuthnError`]
/// deny.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JwksError {
    /// The JWKS document was not valid JSON / not a `{ "keys": [...] }` shape.
    MalformedDocument(String),
    /// A JWK was present but its key material was unusable (bad base64, wrong
    /// length for the curve, unsupported `kty`/`crv`).
    UnusableKey(String),
    /// The JWKS contained no usable keys.
    Empty,
    /// Verifier configuration was invalid (e.g. empty issuer/audience).
    InvalidConfig(&'static str),
}

impl std::fmt::Display for JwksError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JwksError::MalformedDocument(r) => write!(f, "malformed JWKS document: {r}"),
            JwksError::UnusableKey(r) => write!(f, "unusable JWK: {r}"),
            JwksError::Empty => write!(f, "JWKS contained no usable keys"),
            JwksError::InvalidConfig(r) => write!(f, "invalid verifier config: {r}"),
        }
    }
}

impl std::error::Error for JwksError {}

/// Validation policy: which issuer/audience to require, clock-skew tolerance,
/// and which claim carries the tenant id.
#[derive(Clone, Debug)]
pub struct VerifierConfig {
    /// Required `iss`. Must equal the cloud-iam realm issuer exactly.
    pub issuer: String,
    /// Accepted audiences. The token's `aud` must contain at least one of these.
    pub audiences: Vec<String>,
    /// Clock-skew tolerance applied to `exp`/`nbf`, in seconds.
    pub leeway_secs: u64,
    /// Claim name carrying the tenant id (default `tenant_id`).
    pub tenant_claim: String,
}

impl VerifierConfig {
    /// Build a config requiring a single audience and the default `tenant_id`
    /// tenant claim, with a 60s clock-skew tolerance.
    pub fn new(issuer: impl Into<String>, audience: impl Into<String>) -> Self {
        Self {
            issuer: issuer.into(),
            audiences: vec![audience.into()],
            leeway_secs: 60,
            tenant_claim: "tenant_id".to_string(),
        }
    }

    /// Override the clock-skew tolerance.
    #[must_use]
    pub fn with_leeway_secs(mut self, leeway_secs: u64) -> Self {
        self.leeway_secs = leeway_secs;
        self
    }

    /// Override the tenant claim name.
    #[must_use]
    pub fn with_tenant_claim(mut self, claim: impl Into<String>) -> Self {
        self.tenant_claim = claim.into();
        self
    }

    /// Add an accepted audience.
    #[must_use]
    pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
        self.audiences.push(audience.into());
        self
    }

    fn validate(&self) -> Result<(), JwksError> {
        if self.issuer.trim().is_empty() {
            return Err(JwksError::InvalidConfig("issuer is empty"));
        }
        if self.audiences.iter().all(|a| a.trim().is_empty()) {
            return Err(JwksError::InvalidConfig("no non-empty audience"));
        }
        if self.tenant_claim.trim().is_empty() {
            return Err(JwksError::InvalidConfig("tenant_claim is empty"));
        }
        Ok(())
    }
}

/// Supported asymmetric algorithm families.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Alg {
    Rs256,
    Rs384,
    Rs512,
    Es256,
    Es384,
}

impl Alg {
    fn parse(s: &str) -> Option<Self> {
        match s {
            "RS256" => Some(Alg::Rs256),
            "RS384" => Some(Alg::Rs384),
            "RS512" => Some(Alg::Rs512),
            "ES256" => Some(Alg::Es256),
            "ES384" => Some(Alg::Es384),
            _ => None,
        }
    }
}

/// Decoded public key material for one JWK.
enum KeyMaterial {
    /// RSA modulus + exponent, big-endian, no leading zeros.
    Rsa { n: Vec<u8>, e: Vec<u8> },
    /// ECDSA uncompressed point `0x04 || X || Y`, plus its curve.
    Ec { point: Vec<u8>, es: Alg },
}

struct VerifyKey {
    material: KeyMaterial,
    /// If the JWK pinned an `alg`, the token header `alg` must equal it.
    pinned_alg: Option<Alg>,
}

/// JSON shapes for parsing a JWKS document (RFC 7517 / 7518 subset).
#[derive(Deserialize)]
struct JwksDoc {
    keys: Vec<JwkJson>,
}

#[derive(Deserialize)]
struct JwkJson {
    kty: String,
    #[serde(default)]
    kid: Option<String>,
    #[serde(default)]
    alg: Option<String>,
    #[serde(rename = "use", default)]
    use_: Option<String>,
    // RSA
    #[serde(default)]
    n: Option<String>,
    #[serde(default)]
    e: Option<String>,
    // EC
    #[serde(default)]
    crv: Option<String>,
    #[serde(default)]
    x: Option<String>,
    #[serde(default)]
    y: Option<String>,
}

/// Compact-JWS header.
#[derive(Deserialize)]
struct JwsHeader {
    alg: String,
    #[serde(default)]
    kid: Option<String>,
}

/// JWT/OIDC implementation of [`PrincipalVerifier`].
///
/// Construct from a JWKS document (the cloud-iam IdP's `jwks.json`) plus a
/// [`VerifierConfig`]. Verification is pure CPU after construction; refreshing
/// the JWKS is a separate transport concern (rebuild the verifier on rotation).
pub struct JwtPrincipalVerifier {
    /// kid → key. JWKs without a `kid` are also tracked for the single-key case.
    keys_by_kid: HashMap<String, VerifyKey>,
    /// Keys that carried no `kid` (used only when the token also omits `kid`).
    keyless: Vec<VerifyKey>,
    config: VerifierConfig,
}

impl JwtPrincipalVerifier {
    /// Build from a JWKS JSON document and validation policy.
    ///
    /// # Errors
    /// [`JwksError`] if the config is invalid, the document is malformed, every
    /// key is unusable, or no usable key remains.
    pub fn from_jwks_json(jwks_json: &str, config: VerifierConfig) -> Result<Self, JwksError> {
        config.validate()?;
        let doc: JwksDoc = serde_json::from_str(jwks_json)
            .map_err(|e| JwksError::MalformedDocument(e.to_string()))?;

        let mut keys_by_kid = HashMap::new();
        let mut keyless = Vec::new();

        for jwk in doc.keys {
            // Skip keys explicitly marked for encryption; only signing keys verify.
            if jwk.use_.as_deref() == Some("enc") {
                continue;
            }
            let kid = jwk.kid.clone();
            let key = parse_jwk(jwk)?;
            match kid {
                Some(k) => {
                    keys_by_kid.insert(k, key);
                }
                None => keyless.push(key),
            }
        }

        if keys_by_kid.is_empty() && keyless.is_empty() {
            return Err(JwksError::Empty);
        }

        Ok(Self {
            keys_by_kid,
            keyless,
            config,
        })
    }

    /// Verify a token at an explicit Unix-seconds wall-clock time.
    ///
    /// The trait [`PrincipalVerifier::verify`] delegates here with the system
    /// clock; tests call this directly for deterministic, hermetic time control.
    ///
    /// # Errors
    /// Any [`AuthnError`] (all denials).
    pub fn verify_at(&self, token: &str, now_unix: u64) -> Result<VerifiedPrincipal, AuthnError> {
        if token.trim().is_empty() {
            return Err(AuthnError::MissingToken);
        }

        // Compact JWS: header.payload.signature
        let mut parts = token.split('.');
        let (h_b64, p_b64, s_b64) = match (parts.next(), parts.next(), parts.next(), parts.next()) {
            (Some(h), Some(p), Some(s), None) if !h.is_empty() && !p.is_empty() && !s.is_empty() => {
                (h, p, s)
            }
            _ => return Err(AuthnError::MalformedToken),
        };

        let header: JwsHeader = decode_json(h_b64).ok_or(AuthnError::MalformedToken)?;
        let alg = Alg::parse(&header.alg).ok_or(AuthnError::UnsupportedAlgorithm)?;

        let key = self.select_key(header.kid.as_deref())?;

        // The JWK may pin an algorithm; if so it is authoritative.
        if let Some(pinned) = key.pinned_alg
            && pinned != alg
        {
            return Err(AuthnError::UnsupportedAlgorithm);
        }

        // Verify the signature over the exact encoded `header.payload` bytes
        // BEFORE trusting any claim in the payload.
        let signing_input = {
            let mut s = String::with_capacity(h_b64.len() + 1 + p_b64.len());
            s.push_str(h_b64);
            s.push('.');
            s.push_str(p_b64);
            s
        };
        let signature = URL_SAFE_NO_PAD
            .decode(s_b64)
            .map_err(|_| AuthnError::MalformedToken)?;
        verify_signature(&key.material, alg, signing_input.as_bytes(), &signature)?;

        // Signature is valid — now the payload is trustworthy.
        let claims: Value = decode_json(p_b64).ok_or(AuthnError::MalformedToken)?;

        self.check_claims(&claims, now_unix)
    }

    fn select_key(&self, kid: Option<&str>) -> Result<&VerifyKey, AuthnError> {
        match kid {
            Some(k) => self.keys_by_kid.get(k).ok_or(AuthnError::UnknownKeyId),
            None => {
                // No kid in the token: accept only if exactly one key exists in
                // the whole set (kid-keyed or keyless). Ambiguity is fail-closed.
                if self.keys_by_kid.is_empty() && self.keyless.len() == 1 {
                    Ok(&self.keyless[0])
                } else if self.keyless.is_empty() && self.keys_by_kid.len() == 1 {
                    self.keys_by_kid
                        .values()
                        .next()
                        .ok_or(AuthnError::UnknownKeyId)
                } else {
                    Err(AuthnError::UnknownKeyId)
                }
            }
        }
    }

    fn check_claims(&self, claims: &Value, now_unix: u64) -> Result<VerifiedPrincipal, AuthnError> {
        // Issuer
        let issuer = claims
            .get("iss")
            .and_then(Value::as_str)
            .ok_or(AuthnError::MissingClaim("iss"))?;
        if issuer != self.config.issuer {
            return Err(AuthnError::IssuerMismatch);
        }

        // Audience: `aud` may be a string or an array of strings.
        if !audience_matches(claims.get("aud"), &self.config.audiences) {
            return Err(AuthnError::AudienceMismatch);
        }

        // Expiry (required) with leeway.
        let exp = claim_u64(claims, "exp").ok_or(AuthnError::MissingClaim("exp"))?;
        if now_unix > exp.saturating_add(self.config.leeway_secs) {
            return Err(AuthnError::Expired);
        }

        // Not-before (optional) with leeway.
        if let Some(nbf) = claim_u64(claims, "nbf")
            && nbf > now_unix.saturating_add(self.config.leeway_secs)
        {
            return Err(AuthnError::NotYetValid);
        }

        // Subject → agent.
        let subject = claims
            .get("sub")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or(AuthnError::MissingClaim("sub"))?;
        let agent = AgentId::new(subject).map_err(|_| AuthnError::InvalidClaim("sub"))?;

        // Tenant claim → tenant.
        let tenant_raw = claims
            .get(&self.config.tenant_claim)
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or(AuthnError::MissingClaim("tenant"))?;
        let tenant = TenantId::new(tenant_raw).map_err(|_| AuthnError::InvalidClaim("tenant"))?;

        Ok(VerifiedPrincipal {
            tenant,
            agent,
            subject: subject.to_string(),
            issuer: issuer.to_string(),
            expires_at_unix: exp,
        })
    }
}

impl std::fmt::Debug for JwtPrincipalVerifier {
    /// Redaction-safe: prints key *counts* and policy, never key material.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtPrincipalVerifier")
            .field("keyed_jwks", &self.keys_by_kid.len())
            .field("keyless_jwks", &self.keyless.len())
            .field("issuer", &self.config.issuer)
            .field("audiences", &self.config.audiences)
            .field("leeway_secs", &self.config.leeway_secs)
            .finish()
    }
}

impl PrincipalVerifier for JwtPrincipalVerifier {
    fn verify(&self, token: &str) -> Result<VerifiedPrincipal, AuthnError> {
        // Production path uses the system clock; a clock read failure
        // (pre-epoch) is fail-closed.
        let now_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .map_err(|_| AuthnError::Expired)?;
        self.verify_at(token, now_unix)
    }
}

/// Parse one JWK into decoded key material.
fn parse_jwk(jwk: JwkJson) -> Result<VerifyKey, JwksError> {
    let pinned_alg = match jwk.alg.as_deref() {
        Some(a) => Some(Alg::parse(a).ok_or_else(|| {
            JwksError::UnusableKey(format!("unsupported JWK alg: {a}"))
        })?),
        None => None,
    };

    match jwk.kty.as_str() {
        "RSA" => {
            let n = jwk
                .n
                .as_deref()
                .ok_or_else(|| JwksError::UnusableKey("RSA JWK missing n".into()))?;
            let e = jwk
                .e
                .as_deref()
                .ok_or_else(|| JwksError::UnusableKey("RSA JWK missing e".into()))?;
            let n = b64url_field(n, "RSA n")?;
            let e = b64url_field(e, "RSA e")?;
            // aws-lc-rs rejects leading-zero components; RFC 7518 mandates
            // minimal big-endian, so a leading zero is a malformed key.
            if n.first() == Some(&0) || e.first() == Some(&0) || n.is_empty() || e.is_empty() {
                return Err(JwksError::UnusableKey("RSA component has leading zero".into()));
            }
            if let Some(pinned) = pinned_alg
                && !matches!(pinned, Alg::Rs256 | Alg::Rs384 | Alg::Rs512)
            {
                return Err(JwksError::UnusableKey("RSA JWK pinned to non-RSA alg".into()));
            }
            Ok(VerifyKey {
                material: KeyMaterial::Rsa { n, e },
                pinned_alg,
            })
        }
        "EC" => {
            let crv = jwk
                .crv
                .as_deref()
                .ok_or_else(|| JwksError::UnusableKey("EC JWK missing crv".into()))?;
            let (es, coord_len) = match crv {
                "P-256" => (Alg::Es256, 32usize),
                "P-384" => (Alg::Es384, 48usize),
                other => {
                    return Err(JwksError::UnusableKey(format!("unsupported EC crv: {other}")));
                }
            };
            let x = b64url_field(
                jwk.x
                    .as_deref()
                    .ok_or_else(|| JwksError::UnusableKey("EC JWK missing x".into()))?,
                "EC x",
            )?;
            let y = b64url_field(
                jwk.y
                    .as_deref()
                    .ok_or_else(|| JwksError::UnusableKey("EC JWK missing y".into()))?,
                "EC y",
            )?;
            if x.len() != coord_len || y.len() != coord_len {
                return Err(JwksError::UnusableKey(format!(
                    "EC coordinate wrong length for {crv}"
                )));
            }
            if let Some(pinned) = pinned_alg
                && pinned != es
            {
                return Err(JwksError::UnusableKey("EC JWK alg/crv mismatch".into()));
            }
            // Uncompressed SEC1 point: 0x04 || X || Y.
            let mut point = Vec::with_capacity(1 + x.len() + y.len());
            point.push(0x04);
            point.extend_from_slice(&x);
            point.extend_from_slice(&y);
            Ok(VerifyKey {
                material: KeyMaterial::Ec { point, es },
                pinned_alg: Some(es),
            })
        }
        other => Err(JwksError::UnusableKey(format!("unsupported kty: {other}"))),
    }
}

/// Verify `signature` over `message` using `material` under `alg`. The key's
/// type must be consistent with `alg` (RSA↔RS*, EC↔ES* and matching curve), else
/// the algorithm was confused — fail closed.
fn verify_signature(
    material: &KeyMaterial,
    alg: Alg,
    message: &[u8],
    signature: &[u8],
) -> Result<(), AuthnError> {
    match (material, alg) {
        (KeyMaterial::Rsa { n, e }, Alg::Rs256 | Alg::Rs384 | Alg::Rs512) => {
            let params = match alg {
                Alg::Rs256 => &RSA_PKCS1_2048_8192_SHA256,
                Alg::Rs384 => &RSA_PKCS1_2048_8192_SHA384,
                Alg::Rs512 => &RSA_PKCS1_2048_8192_SHA512,
                _ => unreachable!("guarded by outer match"),
            };
            RsaPublicKeyComponents { n, e }
                .verify(params, message, signature)
                .map_err(|_| AuthnError::SignatureInvalid)
        }
        (KeyMaterial::Ec { point, es }, Alg::Es256 | Alg::Es384) => {
            // Curve/alg must agree: ES256→P-256, ES384→P-384.
            if *es != alg {
                return Err(AuthnError::UnsupportedAlgorithm);
            }
            let verifier = match alg {
                Alg::Es256 => UnparsedPublicKey::new(&ECDSA_P256_SHA256_FIXED, point),
                Alg::Es384 => UnparsedPublicKey::new(&ECDSA_P384_SHA384_FIXED, point),
                _ => unreachable!("guarded by outer match"),
            };
            verifier
                .verify(message, signature)
                .map_err(|_| AuthnError::SignatureInvalid)
        }
        // Key type does not match the header algorithm family.
        _ => Err(AuthnError::UnsupportedAlgorithm),
    }
}

/// Decode a base64url-no-pad segment as UTF-8 JSON into `T`.
fn decode_json<T: for<'de> Deserialize<'de>>(b64: &str) -> Option<T> {
    let bytes = URL_SAFE_NO_PAD.decode(b64).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Decode a base64url-no-pad JWK field, erroring as an unusable key.
fn b64url_field(b64: &str, label: &str) -> Result<Vec<u8>, JwksError> {
    URL_SAFE_NO_PAD
        .decode(b64)
        .map_err(|e| JwksError::UnusableKey(format!("{label} base64url: {e}")))
}

/// `exp`/`nbf` may arrive as a JSON integer or (per some IdPs) a float; accept
/// both, truncating fractional seconds toward zero.
fn claim_u64(claims: &Value, name: &str) -> Option<u64> {
    let v = claims.get(name)?;
    if let Some(u) = v.as_u64() {
        return Some(u);
    }
    let f = v.as_f64()?;
    if f.is_finite() && f >= 0.0 {
        Some(f as u64)
    } else {
        None
    }
}

/// `aud` matches when it is a string equal to an expected audience, or an array
/// containing at least one expected audience.
fn audience_matches(aud: Option<&Value>, expected: &[String]) -> bool {
    match aud {
        Some(Value::String(s)) => expected.iter().any(|e| e == s),
        Some(Value::Array(items)) => items.iter().any(|item| {
            item.as_str()
                .is_some_and(|s| expected.iter().any(|e| e == s))
        }),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audience_matches_string_and_array() {
        let want = vec!["cloud-intelligence".to_string()];
        assert!(audience_matches(
            Some(&Value::String("cloud-intelligence".into())),
            &want
        ));
        assert!(audience_matches(
            Some(&serde_json::json!(["other", "cloud-intelligence"])),
            &want
        ));
        assert!(!audience_matches(Some(&Value::String("other".into())), &want));
        assert!(!audience_matches(None, &want));
    }

    #[test]
    fn claim_u64_accepts_int_and_float() {
        let c = serde_json::json!({ "a": 1700000000, "b": 1700000000.9, "c": -5 });
        assert_eq!(claim_u64(&c, "a"), Some(1_700_000_000));
        assert_eq!(claim_u64(&c, "b"), Some(1_700_000_000));
        assert_eq!(claim_u64(&c, "c"), None);
        assert_eq!(claim_u64(&c, "missing"), None);
    }

    #[test]
    fn config_validation_rejects_empty() {
        assert!(VerifierConfig::new("", "aud").validate().is_err());
        assert!(VerifierConfig::new("iss", "").validate().is_err());
        assert!(
            VerifierConfig::new("iss", "aud")
                .with_tenant_claim("")
                .validate()
                .is_err()
        );
        assert!(VerifierConfig::new("iss", "aud").validate().is_ok());
    }

    #[test]
    fn rejects_none_and_symmetric_algorithms() {
        assert_eq!(Alg::parse("none"), None);
        assert_eq!(Alg::parse("HS256"), None);
        assert!(Alg::parse("RS256").is_some());
        assert!(Alg::parse("ES256").is_some());
    }
}
