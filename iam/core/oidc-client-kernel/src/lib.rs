//! OIDC relying-party client kernel.
//!
//! Authority: ADR-0145 (inter-microservice OIDC bearer), ADR-0476 (bespoke
//! identity issuer, founder-locked 2026-05-28; supersedes the ADR-0187
//! Zitadel decision), ADR-0189 (ACR step-up classes).
//!
//! This kernel defines the [`OidcClient`] trait and a reference verifier
//! (`ReferenceOidcVerifier`) that:
//!
//! 1. Parses a compact JWS (header.payload.signature).
//! 2. Looks up the signing key by `kid` in the cached JWKS.
//! 3. Verifies the signature against the public key.
//! 4. Validates `iss`, `aud`, `exp`, `nbf`, `iat`.
//! 5. Extracts oyatie custom claims: `tenant_id`, `acr`, `purpose`,
//!    `data_class`, `acr_event_at`.
//!
//! Cryptographic primitives are abstracted behind the [`JwsVerifier`] trait
//! so that downstream crates may plug in `ring`, `rustls`, `aws-lc-rs`, or
//! HSM-backed signing without forcing a cryptographic dependency into the
//! kernel. The reference test verifier in this crate validates structure
//! and claim parsing using a deterministic stub verifier (no live network).
//!
//! ### Non-goals
//!
//! - No HTTP client. Callers supply JWKS bytes via [`Jwks::from_json`] or
//!   plug a [`JwksFetcher`] adapter.
//! - No clock skew tolerance beyond the configurable [`ClockTolerance`].
//! - No token revocation cache (caller's responsibility).
//! - No refresh-token semantics (the ADR-0476 identity issuer handles
//!   them server-side).

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// ACR class enum per ADR-0189.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AcrLevel {
    Routine,
    Elevated,
    Sensitive,
    Critical,
}

impl AcrLevel {
    /// Numeric rank for ordering comparisons; higher = stronger.
    pub const fn rank(self) -> u8 {
        match self {
            Self::Routine => 1,
            Self::Elevated => 2,
            Self::Sensitive => 3,
            Self::Critical => 4,
        }
    }

    /// Returns true if `self` is at least as strong as `floor`.
    pub fn meets(self, floor: Self) -> bool {
        self.rank() >= floor.rank()
    }
}

/// Oyatie-canonical OIDC claims; superset of RFC 9068 + RFC 7519.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OidcClaims {
    pub iss: String,
    pub aud: Audience,
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
    #[serde(default)]
    pub nbf: Option<i64>,
    #[serde(default)]
    pub jti: Option<String>,
    pub tenant_id: String,
    pub acr: AcrLevel,
    #[serde(default)]
    pub acr_event_at: Option<i64>,
    #[serde(default)]
    pub purpose: Option<String>,
    #[serde(default)]
    pub data_class: Option<String>,
    #[serde(default, flatten)]
    pub additional: BTreeMap<String, serde_json::Value>,
}

/// `aud` may be a single string or an array per RFC 7519 §4.1.3.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Audience {
    Single(String),
    Many(Vec<String>),
}

impl Audience {
    pub fn contains(&self, expected: &str) -> bool {
        match self {
            Self::Single(s) => s == expected,
            Self::Many(v) => v.iter().any(|s| s == expected),
        }
    }
}

/// Header of a compact JWS (subset).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct JwtHeader {
    pub alg: String,
    pub kid: String,
    #[serde(default)]
    pub typ: Option<String>,
}

/// Single signing key as published in JWKS.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Jwk {
    pub kid: String,
    pub kty: String,
    pub alg: String,
    #[serde(default)]
    pub r#use: Option<String>, // data_class: INTERNAL_ONLY
    /// Base64url-encoded modulus (RSA) or x-coordinate (EC) etc. We keep this
    /// opaque at the kernel; the `JwsVerifier` adapter knows how to decode.
    #[serde(default)]
    pub n: Option<String>,
    #[serde(default)]
    pub e: Option<String>,
    #[serde(default)]
    pub x: Option<String>,
    #[serde(default)]
    pub y: Option<String>,
    #[serde(default)]
    pub crv: Option<String>,
}

/// JWKS document.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

impl Jwks {
    pub fn from_json(bytes: &[u8]) -> Result<Self, OidcError> {
        serde_json::from_slice(bytes).map_err(|e| OidcError::Malformed(format!("jwks: {e}")))
    }

    pub fn find(&self, kid: &str) -> Option<&Jwk> {
        self.keys.iter().find(|k| k.kid == kid)
    }
}

/// Clock tolerance used by the verifier (default ±60s per RFC 7519 §4.1.4
/// guidance; sub-minute clock skew tolerated, sub-second exact).
#[derive(Clone, Copy, Debug)]
pub struct ClockTolerance(pub i64);

impl Default for ClockTolerance {
    fn default() -> Self {
        Self(60)
    }
}

/// Verifier configuration.
#[derive(Clone, Debug)]
pub struct VerifyConfig {
    pub expected_issuer: String,
    pub expected_audience: String,
    pub clock_tolerance: ClockTolerance,
    pub now_unix_seconds: i64,
}

/// Pluggable JWS signature verifier. Implementations (ring, rustls,
/// aws-lc-rs, HSM-backed) live outside this kernel.
pub trait JwsVerifier: Send + Sync {
    /// Verify `signing_input` against `signature_b64url` using `jwk`.
    /// Returns Ok(()) on valid signature.
    fn verify(
        &self,
        jwk: &Jwk,
        alg: &str,
        signing_input: &[u8],
        signature_b64url: &str,
    ) -> Result<(), OidcError>;
}

/// Pluggable JWKS fetcher (for caching adapters).
pub trait JwksFetcher: Send + Sync {
    fn fetch(&self, issuer: &str) -> Result<Jwks, OidcError>;
}

/// The canonical OIDC client trait every consumer crate uses.
pub trait OidcClient: Send + Sync {
    /// Verify a compact JWT bearer and extract claims.
    fn verify(&self, bearer: &str, cfg: &VerifyConfig) -> Result<OidcClaims, OidcError>;

    /// Check whether the principal's current ACR meets a required floor.
    fn meets_acr(&self, claims: &OidcClaims, floor: AcrLevel) -> bool {
        claims.acr.meets(floor)
    }
}

/// Failure mode enum. Distinct variants per failure class so caller policy
/// can branch (e.g., issue 401 for expiry vs 403 for audience mismatch).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OidcError {
    Malformed(String),
    UnknownKid(String),
    AlgMismatch { expected: String, actual: String },
    SignatureInvalid,
    IssuerMismatch { expected: String, actual: String },
    AudienceMismatch { expected: String, actual: String },
    Expired { now: i64, exp: i64 },
    NotYetValid { now: i64, nbf: i64 },
    MissingClaim(&'static str),
    InvalidClaim { claim: &'static str, reason: String },
}

impl std::fmt::Display for OidcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed(s) => write!(f, "oidc: malformed token: {s}"),
            Self::UnknownKid(k) => write!(f, "oidc: unknown kid '{k}'"),
            Self::AlgMismatch { expected, actual } => {
                write!(f, "oidc: alg mismatch: expected {expected}, got {actual}")
            }
            Self::SignatureInvalid => write!(f, "oidc: signature invalid"),
            Self::IssuerMismatch { expected, actual } => {
                write!(f, "oidc: iss mismatch: expected {expected}, got {actual}")
            }
            Self::AudienceMismatch { expected, actual } => {
                write!(f, "oidc: aud mismatch: expected {expected}, got {actual}")
            }
            Self::Expired { now, exp } => {
                write!(f, "oidc: expired (now={now}, exp={exp})")
            }
            Self::NotYetValid { now, nbf } => {
                write!(f, "oidc: not yet valid (now={now}, nbf={nbf})")
            }
            Self::MissingClaim(c) => write!(f, "oidc: missing required claim '{c}'"),
            Self::InvalidClaim { claim, reason } => {
                write!(f, "oidc: invalid claim '{claim}': {reason}")
            }
        }
    }
}

impl std::error::Error for OidcError {}

/// The reference verifier wires a [`JwsVerifier`] adapter + a [`Jwks`]
/// snapshot. Production deployments wrap this with a JWKS-cache adapter.
pub struct ReferenceOidcVerifier<V: JwsVerifier> {
    pub jwks: Jwks,
    pub jws_verifier: V,
    pub allowed_algs: Vec<String>,
}

impl<V: JwsVerifier> ReferenceOidcVerifier<V> {
    pub fn new(jwks: Jwks, jws_verifier: V) -> Self {
        Self {
            jwks,
            jws_verifier,
            // Per the ADR-0476 identity issuer default + RFC 8725 BCP §3.1;
            // RS256 + ES256 only.
            // HS* (symmetric) is forbidden for relying-party verification of
            // third-party-issued tokens.
            allowed_algs: vec!["RS256".to_owned(), "ES256".to_owned()],
        }
    }
}

impl<V: JwsVerifier> OidcClient for ReferenceOidcVerifier<V> {
    fn verify(&self, bearer: &str, cfg: &VerifyConfig) -> Result<OidcClaims, OidcError> {
        let (header_b64, payload_b64, signature_b64) = split_jwt(bearer)?;
        let signing_input = {
            let mut v = Vec::with_capacity(header_b64.len() + 1 + payload_b64.len());
            v.extend_from_slice(header_b64.as_bytes());
            v.push(b'.');
            v.extend_from_slice(payload_b64.as_bytes());
            v
        };

        let header_bytes = b64url_decode(header_b64)?;
        let header: JwtHeader = serde_json::from_slice(&header_bytes)
            .map_err(|e| OidcError::Malformed(format!("header: {e}")))?;

        if !self.allowed_algs.iter().any(|a| a == &header.alg) {
            return Err(OidcError::AlgMismatch {
                expected: self.allowed_algs.join("|"),
                actual: header.alg,
            });
        }

        let jwk = self
            .jwks
            .find(&header.kid)
            .ok_or_else(|| OidcError::UnknownKid(header.kid.clone()))?;

        self.jws_verifier
            .verify(jwk, &header.alg, &signing_input, signature_b64)?;

        let payload_bytes = b64url_decode(payload_b64)?;
        let claims: OidcClaims = serde_json::from_slice(&payload_bytes)
            .map_err(|e| OidcError::Malformed(format!("payload: {e}")))?;

        if claims.iss != cfg.expected_issuer {
            return Err(OidcError::IssuerMismatch {
                expected: cfg.expected_issuer.clone(),
                actual: claims.iss,
            });
        }
        if !claims.aud.contains(&cfg.expected_audience) {
            return Err(OidcError::AudienceMismatch {
                expected: cfg.expected_audience.clone(),
                actual: match &claims.aud {
                    Audience::Single(s) => s.clone(),
                    Audience::Many(v) => v.join(","),
                },
            });
        }

        let skew = cfg.clock_tolerance.0;
        if cfg.now_unix_seconds > claims.exp + skew {
            return Err(OidcError::Expired {
                now: cfg.now_unix_seconds,
                exp: claims.exp,
            });
        }
        if let Some(nbf) = claims.nbf
            && cfg.now_unix_seconds + skew < nbf
        {
            return Err(OidcError::NotYetValid {
                now: cfg.now_unix_seconds,
                nbf,
            });
        }

        if claims.tenant_id.is_empty() {
            return Err(OidcError::MissingClaim("tenant_id"));
        }

        Ok(claims)
    }
}

fn split_jwt(bearer: &str) -> Result<(&str, &str, &str), OidcError> {
    let mut it = bearer.split('.');
    let h = it
        .next()
        .ok_or_else(|| OidcError::Malformed("empty token".to_owned()))?;
    let p = it
        .next()
        .ok_or_else(|| OidcError::Malformed("missing payload".to_owned()))?;
    let s = it
        .next()
        .ok_or_else(|| OidcError::Malformed("missing signature".to_owned()))?;
    if it.next().is_some() {
        return Err(OidcError::Malformed("excess segments".to_owned()));
    }
    if h.is_empty() || p.is_empty() || s.is_empty() {
        return Err(OidcError::Malformed("empty segment".to_owned()));
    }
    Ok((h, p, s))
}

/// Minimal base64url decoder per RFC 4648 §5 (no padding, URL-safe alphabet).
/// Kept in-crate to avoid a base64 dependency in the kernel.
pub fn b64url_decode(input: &str) -> Result<Vec<u8>, OidcError> {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut lut = [0xFFu8; 256];
    for (i, c) in ALPHABET.iter().enumerate() {
        lut[*c as usize] = i as u8;
    }
    lut[b'=' as usize] = 0xFE; // padding marker

    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() * 3 / 4 + 2);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;
    for &b in bytes {
        let v = lut[b as usize];
        if v == 0xFE {
            break; // padding ends
        }
        if v == 0xFF {
            return Err(OidcError::Malformed(format!(
                "invalid b64url byte 0x{b:02x}"
            )));
        }
        buf = (buf << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xFF) as u8);
        }
    }
    Ok(out)
}

/// Inverse of `b64url_decode`; used by tests + tooling.
pub fn b64url_encode(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(input.len() * 4 / 3 + 4);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;
    for &b in input {
        buf = (buf << 8) | u32::from(b);
        bits += 8;
        while bits >= 6 {
            bits -= 6;
            out.push(ALPHABET[((buf >> bits) & 0x3F) as usize] as char);
        }
    }
    if bits > 0 {
        out.push(ALPHABET[((buf << (6 - bits)) & 0x3F) as usize] as char);
    }
    out
}
