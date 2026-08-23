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
//!    (`RS256`/`RS384`/`RS512`, `ES256`, or `EdDSA` over OKP Ed25519 per
//!    RFC 8037). The unsecured `none` algorithm is rejected unconditionally
//!    and a symmetric `HS*` alg (against this asymmetric-only verifier) is
//!    rejected as an algorithm-mismatch — the RS256→HS256 key-confusion class
//!    (RFC 8725 §2.1/§2.2/§3.1).
//! 3. The JOSE `typ` header is present and in the accepted set (cross-JWT
//!    confusion — RFC 8725 §3.11), and any `jku`/`x5u` key-source URL is
//!    refused unless explicitly allowlisted (SSRF / key-injection —
//!    RFC 8725 §3.8). Keys come from the static JWKS, never a header URL.
//! 4. The token's `alg` is bound to the algorithm the resolved `kid` is for:
//!    the key material pins a family (RSA vs EC) and the JWK may pin an exact
//!    `alg`; a mismatch is refused (RFC 8725 §2.2/§3.1).
//! 5. The signature verifies against the issuer's JWK (selected by `kid`)
//!    using **aws-lc-rs** (AWS-LC-backed; constant-time; the canonical Phase-1
//!    crypto backend per ADR-0506).
//! 6. The `iss` claim equals the configured expected issuer.
//! 7. The `aud` claim contains the configured expected audience.
//! 8. `exp` is in the future and `nbf`/`iat` (if present) are not in the
//!    future, using the caller-supplied `now_epoch_seconds` (no ambient clock).
//!
//! Only after all checks pass are the claims projected into a principal whose
//! SPIFFE `trust_domain` is rooted at (and equal to) the verified tenant.
//!
//! ## Why aws-lc-rs (and not a higher-level JWT crate)
//!
//! `aws-lc-rs` (Apache-2.0 + ISC — OSI-clean) is the canonical Phase-1 crypto
//! backend per ADR-0506. It is backed by AWS-LC (a hardened, FIPS-validatable
//! fork of BoringSSL) and exports ring-compatible module paths. Validating the
//! JWS directly against `aws-lc-rs` keeps the trusted compute base small and
//! avoids pulling an unvetted transitive tree. JWKS RSA keys arrive as `n`/`e`;
//! we encode the PKCS#1 `RSAPublicKey` DER that `aws-lc-rs` expects (see
//! [`rsa_pkcs1_der`]).

// ADR-0083 Tier 3: production code stays panic-free; tests may use unwrap/expect.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

#[cfg(test)]
mod eddsa;

use std::collections::HashSet;

use aws_lc_rs::signature::{self, ED25519};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Deserialize;

use iam_identity_workload_domain::{
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
    /// The JOSE `alg` was the unsecured `none` algorithm, which is rejected
    /// unconditionally (RFC 8725 §2.1/§3.1 — never accept an unsecured JWS).
    AlgNone,
    /// The JOSE `typ` header was absent or not in the accepted set, guarding
    /// against cross-JWT confusion (RFC 8725 §3.11).
    InvalidType,
    /// The JOSE header carried a `jku`/`x5u` key-source URL that was not in the
    /// configured allowlist — refused to prevent SSRF / key-injection
    /// (RFC 8725 §3.8). With no allowlist configured (the default), *any*
    /// `jku`/`x5u` is refused because keys come from the static JWKS.
    UntrustedKeySourceUrl,
    /// The token's `alg` did not match the algorithm the resolved `kid` is
    /// bound to (e.g. an `HS256` token presented against an RSA `kid`), an
    /// algorithm-substitution / key-confusion attack (RFC 8725 §2.2/§3.1).
    AlgorithmMismatch,
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
            Self::AlgNone => f.write_str("unsecured 'none' algorithm is rejected (RFC 8725)"),
            Self::InvalidType => {
                f.write_str("JOSE typ header absent or not in the accepted set (RFC 8725)")
            }
            Self::UntrustedKeySourceUrl => {
                f.write_str("JOSE jku/x5u key-source URL is not in the allowlist (RFC 8725)")
            }
            Self::AlgorithmMismatch => {
                f.write_str("token alg does not match the algorithm bound to the kid (RFC 8725)")
            }
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

/// Errors produced while parsing an issuer-published JWKS document into the
/// static verifier keyset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JwksError {
    /// The document was not valid JWKS JSON.
    DecodeError,
    /// A required JWK member was absent or empty.
    MissingComponent(&'static str),
    /// The JWK `use` member was present but not `sig`.
    UnsupportedKeyUse(String),
    /// The JWK `kty` member was absent or outside the supported asymmetric set.
    UnsupportedKeyType(String),
    /// The JWK `crv` member was absent or unsupported for the key type.
    UnsupportedCurve(String),
    /// The issuer published a JWKS with no verifier keys.
    EmptyKeySet,
    /// Two or more keys shared one `kid`, making deterministic resolution ambiguous.
    DuplicateKid(String),
}

impl std::fmt::Display for JwksError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DecodeError => f.write_str("JWKS JSON decode failed"),
            Self::MissingComponent(name) => write!(f, "JWK missing required member: {name}"),
            Self::UnsupportedKeyUse(key_use) => {
                write!(f, "unsupported JWK use '{key_use}' (expected 'sig')")
            }
            Self::UnsupportedKeyType(kty) => write!(f, "unsupported JWK kty '{kty}'"),
            Self::UnsupportedCurve(crv) => write!(f, "unsupported JWK curve '{crv}'"),
            Self::EmptyKeySet => f.write_str("issuer JWKS contained no keys"),
            Self::DuplicateKid(kid) => write!(f, "issuer JWKS contains duplicate kid '{kid}'"),
        }
    }
}

impl std::error::Error for JwksError {}

/// Supported JWS signature algorithms. Symmetric (`HS*`) algorithms are
/// intentionally unsupported: workload tokens are issuer-signed with asymmetric
/// keys, so a shared HMAC secret would be a downgrade.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum JwsAlg {
    Rs256,
    Rs384,
    Rs512,
    Es256,
    /// EdDSA over Ed25519 (RFC 8037 / RFC 8032). The `alg` header value is
    /// `"EdDSA"` and the JWK must be `kty=OKP, crv=Ed25519`.
    EdDsa,
}

impl JwsAlg {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "RS256" => Some(Self::Rs256),
            "RS384" => Some(Self::Rs384),
            "RS512" => Some(Self::Rs512),
            "ES256" => Some(Self::Es256),
            "EdDSA" => Some(Self::EdDsa),
            _ => None,
        }
    }

    fn is_rsa(self) -> bool {
        matches!(self, Self::Rs256 | Self::Rs384 | Self::Rs512)
    }

    /// The key family this algorithm requires, used to bind a token's `alg` to
    /// the algorithm the resolved `kid` is declared for (RFC 8725 §2.2/§3.1).
    fn family(self) -> AlgFamily {
        if self.is_rsa() {
            AlgFamily::Rsa
        } else if matches!(self, Self::EdDsa) {
            AlgFamily::OkpEd25519
        } else {
            AlgFamily::EcP256
        }
    }
}

/// The asymmetric key family an [`JwsAlg`] belongs to. A JWK may pin the family
/// (and, optionally, the exact `alg`) it is valid for so a token cannot present
/// one algorithm against a key minted for another.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AlgFamily {
    Rsa,
    EcP256,
    /// OKP `crv=Ed25519` key family (RFC 8037).
    OkpEd25519,
}

/// A single JSON Web Key. Supports RSA (`n`/`e`) and EC P-256 (`x`/`y`) keys.
///
/// The key material implies an algorithm *family* (RSA vs EC P-256) that the
/// token's `alg` is always bound to. A key MAY additionally pin the exact `alg`
/// it is valid for via [`Jwk::with_alg`]; when pinned, a token presenting any
/// other `alg` is refused with [`OidcValidationError::AlgorithmMismatch`]
/// (RFC 8725 §2.2 algorithm-substitution defense).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Jwk {
    /// Key id matched against the token's `kid`.
    pub kid: String,
    /// Key material.
    pub material: JwkMaterial,
    /// Optional exact JWS `alg` this key is pinned to (e.g. `"RS256"`). When
    /// `Some`, the token's `alg` MUST equal it; when `None`, only the family
    /// implied by `material` is enforced.
    pub alg: Option<String>,
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
    /// OKP Ed25519 public key (RFC 8037 §2): the raw 32-byte Edwards point
    /// encoded as base64url in `x`. There is no `y` for OKP keys.
    OkpEd25519 {
        /// base64url-encoded 32-byte raw public key (the Edwards point).
        x: String,
    },
}

impl Jwk {
    /// Construct an RSA JWK (no exact-`alg` pin; family is enforced).
    #[must_use]
    pub fn rsa(kid: impl Into<String>, n: impl Into<String>, e: impl Into<String>) -> Self {
        Self {
            kid: kid.into(),
            material: JwkMaterial::Rsa {
                n: n.into(),
                e: e.into(),
            },
            alg: None,
        }
    }

    /// Construct an EC P-256 JWK (no exact-`alg` pin; family is enforced).
    #[must_use]
    pub fn ec_p256(kid: impl Into<String>, x: impl Into<String>, y: impl Into<String>) -> Self {
        Self {
            kid: kid.into(),
            material: JwkMaterial::EcP256 {
                x: x.into(),
                y: y.into(),
            },
            alg: None,
        }
    }

    /// Construct an OKP Ed25519 JWK (RFC 8037) from the raw 32-byte base64url
    /// public key `x` (the Edwards point). No exact-`alg` pin; family is enforced.
    #[must_use]
    pub fn okp_ed25519(kid: impl Into<String>, x: impl Into<String>) -> Self {
        Self {
            kid: kid.into(),
            material: JwkMaterial::OkpEd25519 { x: x.into() },
            alg: None,
        }
    }

    /// Pin the exact JWS `alg` this key may verify (builder). A token whose
    /// `alg` differs is refused as an [`OidcValidationError::AlgorithmMismatch`].
    #[must_use]
    pub fn with_alg(mut self, alg: impl Into<String>) -> Self {
        self.alg = Some(alg.into());
        self
    }

    /// The key family implied by the material.
    fn family(&self) -> AlgFamily {
        match self.material {
            JwkMaterial::Rsa { .. } => AlgFamily::Rsa,
            JwkMaterial::EcP256 { .. } => AlgFamily::EcP256,
            JwkMaterial::OkpEd25519 { .. } => AlgFamily::OkpEd25519,
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

    /// Parse an issuer-published RFC 7517 JWKS JSON document into the static
    /// verifier keyset used by [`validate_workload_token`]. This is deliberately
    /// a pure parse/normalization step: callers fetch/cache the JWKS outside
    /// the hot path, then pass this immutable keyset to the verifier. No
    /// synchronous introspection or header-directed key fetch is performed here.
    ///
    /// # Errors
    /// Returns [`JwksError`] when the document is malformed or includes a key
    /// shape this verifier does not support.
    pub fn from_jwks_json(document: &str) -> Result<Self, JwksError> {
        let parsed: JwksDocument =
            serde_json::from_str(document).map_err(|_| JwksError::DecodeError)?;
        if parsed.keys.is_empty() {
            return Err(JwksError::EmptyKeySet);
        }
        let mut keys = Vec::with_capacity(parsed.keys.len());
        let mut seen_kids = HashSet::with_capacity(parsed.keys.len());
        for key in parsed.keys {
            let jwk = key.into_jwk()?;
            if !seen_kids.insert(jwk.kid.clone()) {
                return Err(JwksError::DuplicateKid(jwk.kid));
            }
            keys.push(jwk);
        }
        Ok(Self { keys })
    }

    /// Borrow the normalized keys in this static verifier keyset.
    #[must_use]
    pub fn keys(&self) -> &[Jwk] {
        &self.keys
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

#[derive(Debug, Deserialize)]
struct JwksDocument {
    keys: Vec<JwkDocumentKey>,
}

#[derive(Debug, Deserialize)]
struct JwkDocumentKey {
    kid: Option<String>,
    kty: Option<String>,
    #[serde(default)]
    alg: Option<String>,
    #[serde(default, rename = "use")]
    key_use: Option<String>,
    #[serde(default)]
    n: Option<String>,
    #[serde(default)]
    e: Option<String>,
    #[serde(default)]
    x: Option<String>,
    #[serde(default)]
    y: Option<String>,
    #[serde(default)]
    crv: Option<String>,
}

impl JwkDocumentKey {
    fn into_jwk(self) -> Result<Jwk, JwksError> {
        if let Some(key_use) = self.key_use.as_deref()
            && key_use != "sig"
        {
            return Err(JwksError::UnsupportedKeyUse(key_use.to_owned()));
        }

        let kid = required_jwk_member(self.kid, "kid")?;
        let kty = required_jwk_member(self.kty, "kty")?;
        let mut jwk = match kty.as_str() {
            "RSA" => Jwk::rsa(
                kid,
                required_jwk_member(self.n, "n")?,
                required_jwk_member(self.e, "e")?,
            ),
            "EC" => {
                let crv = required_jwk_member(self.crv, "crv")?;
                if crv != "P-256" {
                    return Err(JwksError::UnsupportedCurve(crv));
                }
                Jwk::ec_p256(
                    kid,
                    required_jwk_member(self.x, "x")?,
                    required_jwk_member(self.y, "y")?,
                )
            }
            "OKP" => {
                let crv = required_jwk_member(self.crv, "crv")?;
                if crv != "Ed25519" {
                    return Err(JwksError::UnsupportedCurve(crv));
                }
                Jwk::okp_ed25519(kid, required_jwk_member(self.x, "x")?)
            }
            other => return Err(JwksError::UnsupportedKeyType(other.to_owned())),
        };
        if let Some(alg) = self.alg
            && !alg.trim().is_empty()
        {
            jwk = jwk.with_alg(alg);
        }
        Ok(jwk)
    }
}

fn required_jwk_member(value: Option<String>, name: &'static str) -> Result<String, JwksError> {
    match value {
        Some(value) if !value.trim().is_empty() => Ok(value),
        _ => Err(JwksError::MissingComponent(name)),
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
    /// Accepted JOSE `typ` header values (case-insensitive), guarding against
    /// cross-JWT confusion (RFC 8725 §3.11). Default: `["JWT", "at+jwt"]`. A
    /// token whose `typ` is absent or outside this set is refused. An empty set
    /// disables the check (NOT recommended).
    pub accepted_token_types: Vec<String>,
    /// Allowlist of `jku`/`x5u` key-source URLs the JOSE header may carry
    /// (RFC 8725 §3.8 SSRF defense). Default: empty — because keys are resolved
    /// from the static [`Jwks`], *any* `jku`/`x5u` in the header is refused.
    pub trusted_key_source_urls: Vec<String>,
}

impl ValidationConfig {
    /// Construct a config with OAuth/OIDC-conventional claim names and the
    /// hardened RFC 8725 defaults (accept `JWT`/`at+jwt` typ; refuse any
    /// `jku`/`x5u` since keys come from the static JWKS).
    #[must_use]
    pub fn new(expected_issuer: impl Into<String>, expected_audience: impl Into<String>) -> Self {
        Self {
            expected_issuer: expected_issuer.into(),
            expected_audience: expected_audience.into(),
            tenant_claim: "tenant_id".into(),
            workload_claim: "sub".into(),
            capability_claim: "owning_capability".into(),
            scope_claim: "scope".into(),
            accepted_token_types: vec!["JWT".into(), "at+jwt".into()],
            trusted_key_source_urls: Vec::new(),
        }
    }

    /// Replace the accepted `typ` set (builder).
    #[must_use]
    pub fn with_accepted_token_types(
        mut self,
        types: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.accepted_token_types = types.into_iter().map(Into::into).collect();
        self
    }

    /// Allowlist a `jku`/`x5u` key-source URL (builder). Without this, any
    /// `jku`/`x5u` header is refused.
    #[must_use]
    pub fn allow_key_source_url(mut self, url: impl Into<String>) -> Self {
        self.trusted_key_source_urls.push(url.into());
        self
    }

    /// Whether `typ` is accepted (case-insensitive). An empty accepted set
    /// disables the check (returns `true`).
    fn accepts_type(&self, typ: &str) -> bool {
        if self.accepted_token_types.is_empty() {
            return true;
        }
        self.accepted_token_types
            .iter()
            .any(|accepted| accepted.eq_ignore_ascii_case(typ))
    }

    /// Whether a `jku`/`x5u` URL is on the allowlist (exact match).
    fn trusts_key_source_url(&self, url: &str) -> bool {
        self.trusted_key_source_urls
            .iter()
            .any(|trusted| trusted == url)
    }
}

// --- JWT payload shape (deserialized from the verified second segment) ---

#[derive(Debug, Deserialize)]
struct JoseHeader {
    alg: String,
    #[serde(default)]
    kid: Option<String>,
    #[serde(default)]
    typ: Option<String>,
    /// JWK Set URL (RFC 7515 §4.1.2). Refused unless allowlisted (RFC 8725 §3.8).
    #[serde(default)]
    jku: Option<String>,
    /// X.509 URL (RFC 7515 §4.1.5). Refused unless allowlisted (RFC 8725 §3.8).
    #[serde(default)]
    x5u: Option<String>,
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

    // 2. Parse the JOSE header.
    let header_bytes = b64url_decode(header_b64)?;
    let header: JoseHeader =
        serde_json::from_slice(&header_bytes).map_err(|_| OidcValidationError::DecodeError)?;

    // 2a. Reject the unsecured `none` algorithm unconditionally, BEFORE any
    //     other algorithm handling (RFC 8725 §2.1/§3.1). Case-insensitive: the
    //     attack uses `none`/`None`/`NONE` interchangeably.
    if header.alg.eq_ignore_ascii_case("none") {
        return Err(OidcValidationError::AlgNone);
    }
    // A symmetric `HS*` alg against this asymmetric-only verifier is the classic
    // RS256->HS256 key-confusion attack (the RSA public key used as the HMAC
    // secret). Surface it as a distinct algorithm-mismatch rather than a generic
    // unsupported-alg so the audit chain records the substitution attempt
    // (RFC 8725 §2.2/§3.1).
    if header.alg.len() >= 2 && header.alg[..2].eq_ignore_ascii_case("hs") {
        return Err(OidcValidationError::AlgorithmMismatch);
    }
    let alg = JwsAlg::parse(&header.alg).ok_or(OidcValidationError::UnsupportedAlgorithm)?;

    // 2b. Explicit `typ` check (cross-JWT confusion — RFC 8725 §3.11). The
    //     header must declare a `typ` in the configured accepted set so a token
    //     minted for another purpose cannot be replayed here.
    match header.typ.as_deref() {
        Some(typ) if config.accepts_type(typ) => {}
        _ => return Err(OidcValidationError::InvalidType),
    }

    // 2c. `jku`/`x5u` SSRF defense (RFC 8725 §3.8): keys are resolved ONLY from
    //     the static JWKS, so a header-supplied key-source URL is refused unless
    //     it is explicitly allowlisted. Default config has an empty allowlist,
    //     so any `jku`/`x5u` is rejected.
    for key_source in [header.jku.as_deref(), header.x5u.as_deref()]
        .into_iter()
        .flatten()
    {
        if !config.trusts_key_source_url(key_source) {
            return Err(OidcValidationError::UntrustedKeySourceUrl);
        }
    }

    // 3. Resolve the verification key by kid and verify the signature over the
    //    `header.payload` signing input.
    let kid = header
        .kid
        .as_deref()
        .ok_or(OidcValidationError::UnknownKey)?;
    let jwk = jwks.find(kid).ok_or(OidcValidationError::UnknownKey)?;

    // 3a. Bind the token's `alg` to the algorithm the resolved `kid` is for
    //     (RFC 8725 §2.2/§3.1 algorithm-substitution defense). The key material
    //     pins a family (RSA vs EC), and the JWK MAY pin an exact `alg`. A
    //     mismatch (e.g. `HS256` against an RSA `kid`, or `ES256` against an
    //     RSA key) is refused as an explicit algorithm mismatch — distinct from
    //     a generic unsupported-alg so the audit chain sees the attack class.
    if alg.family() != jwk.family() {
        return Err(OidcValidationError::AlgorithmMismatch);
    }
    if let Some(pinned) = jwk.alg.as_deref()
        && !pinned.eq_ignore_ascii_case(&header.alg)
    {
        return Err(OidcValidationError::AlgorithmMismatch);
    }

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
    // the authz layer can reason over them without re-parsing. The temporal
    // registered claims stay validated above; projecting them preserves the
    // verified `iat` needed by downstream revocation-cutoff gates.
    if let Some(iss) = &claims.iss {
        principal.set_claim("iss", ClaimValue::Text(iss.clone()))?;
    }
    if let Some(exp) = claims.exp {
        principal.set_claim("exp", ClaimValue::Int(exp))?;
    }
    if let Some(nbf) = claims.nbf {
        principal.set_claim("nbf", ClaimValue::Int(nbf))?;
    }
    if let Some(iat) = claims.iat {
        principal.set_claim("iat", ClaimValue::Int(iat))?;
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
    match (&jwk.material, alg) {
        (JwkMaterial::Rsa { n, e }, alg) if alg.is_rsa() => {
            let der = rsa_pkcs1_der(n, e)?;
            let verifier = match alg {
                JwsAlg::Rs256 => &signature::RSA_PKCS1_2048_8192_SHA256,
                JwsAlg::Rs384 => &signature::RSA_PKCS1_2048_8192_SHA384,
                JwsAlg::Rs512 => &signature::RSA_PKCS1_2048_8192_SHA512,
                _ => unreachable!("is_rsa() guarded against non-RSA here"),
            };
            signature::UnparsedPublicKey::new(verifier, der)
                .verify(message, sig)
                .map_err(|_| OidcValidationError::SignatureInvalid)
        }
        (JwkMaterial::EcP256 { x, y }, JwsAlg::Es256) => {
            // ring/aws-lc-rs expects the uncompressed SEC1 point: 0x04 || X || Y.
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
        (JwkMaterial::OkpEd25519 { x }, JwsAlg::EdDsa) => {
            // RFC 8037 §2: the JWK `x` is the raw 32-byte Edwards public key
            // (no SEC1 prefix, no DER wrapping). The signature is 64 bytes.
            let x_bytes = b64url_decode(x).map_err(|_| OidcValidationError::MalformedKey)?;
            if x_bytes.len() != 32 {
                return Err(OidcValidationError::MalformedKey);
            }
            signature::UnparsedPublicKey::new(&ED25519, x_bytes)
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
    // key with `aws-lc-rs` at test time, sign a token, and validate it. This proves
    // the full verify path against real crypto without any network/JWKS fetch.
    use aws_lc_rs::rand::SystemRandom;
    use aws_lc_rs::signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, KeyPair};

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
        let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8.as_ref())
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

    /// Mint an ES256 token with a caller-supplied RAW JOSE header JSON. The
    /// signature is genuine (so the token only fails on the header-policy
    /// checks under test, not on signature). Returns the token + verifying JWK.
    fn mint_with_header(header_json: &str, claims_json: &str, kid: &str) -> SignedToken {
        let rng = SystemRandom::new();
        let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng)
            .expect("generate pkcs8");
        let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8.as_ref())
            .expect("load key");
        let public = key_pair.public_key().as_ref();
        let x = &public[1..33];
        let y = &public[33..65];
        let signing_input = format!(
            "{}.{}",
            b64url(header_json.as_bytes()),
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
        ValidationConfig::new("https://idp.oyatie.com", "cloud-kms")
    }

    fn valid_claims(now: i64) -> String {
        format!(
            r#"{{"iss":"https://idp.oyatie.com","aud":"cloud-kms","exp":{},"iat":{},"tenant_id":"ten_acme","sub":"wl_secrets_sync","owning_capability":"cap.cloud.kms","scope":"cloud.kms.decrypt cloud.kms.describe","mfa":true}}"#,
            now + 300,
            now
        )
    }

    #[test]
    fn jwks_from_json_normalizes_supported_public_keys() {
        let document = serde_json::json!({
            "keys": [
                {"kid": "rsa-1", "kty": "RSA", "alg": "RS256", "use": "sig", "n": "rsa-mod", "e": "AQAB"},
                {"kid": "ec-1", "kty": "EC", "alg": "ES256", "use": "sig", "crv": "P-256", "x": "ec-x", "y": "ec-y"},
                {"kid": "okp-1", "kty": "OKP", "alg": "EdDSA", "use": "sig", "crv": "Ed25519", "x": "okp-x"}
            ]
        })
        .to_string();

        let jwks = Jwks::from_jwks_json(&document).expect("supported issuer JWKS parses");

        assert_eq!(
            jwks.keys(),
            [
                Jwk::rsa("rsa-1", "rsa-mod", "AQAB").with_alg("RS256"),
                Jwk::ec_p256("ec-1", "ec-x", "ec-y").with_alg("ES256"),
                Jwk::okp_ed25519("okp-1", "okp-x").with_alg("EdDSA"),
            ]
        );
    }

    #[test]
    fn jwks_from_json_rejects_malformed_unsupported_empty_and_ambiguous_keysets() {
        let cases = [
            ("not-json", JwksError::DecodeError),
            (
                r#"{"keys":[{"kid":"missing-kty","n":"rsa-mod","e":"AQAB"}]}"#,
                JwksError::MissingComponent("kty"),
            ),
            (
                r#"{"keys":[{"kid":"enc-key","kty":"RSA","use":"enc","n":"rsa-mod","e":"AQAB"}]}"#,
                JwksError::UnsupportedKeyUse("enc".to_owned()),
            ),
            (
                r#"{"keys":[{"kid":"oct-key","kty":"oct","k":"secret"}]}"#,
                JwksError::UnsupportedKeyType("oct".to_owned()),
            ),
            (
                r#"{"keys":[{"kid":"ec-wrong","kty":"EC","crv":"P-384","x":"x","y":"y"}]}"#,
                JwksError::UnsupportedCurve("P-384".to_owned()),
            ),
        ];

        for (document, expected) in cases {
            let err = Jwks::from_jwks_json(document).expect_err("issuer JWKS must be rejected");
            assert_eq!(err, expected);
        }

        let err = Jwks::from_jwks_json(r#"{"keys":[]}"#)
            .expect_err("issuer-published JWKS must not silently normalize to empty verifier set");
        assert_eq!(err, JwksError::EmptyKeySet);

        let err = Jwks::from_jwks_json(
                r#"{"keys":[{"kid":"dup","kty":"RSA","n":"rsa-a","e":"AQAB"},{"kid":"dup","kty":"RSA","n":"rsa-b","e":"AQAB"}]}"#
            )
            .expect_err("issuer-published JWKS must reject ambiguous duplicate kid values");
        assert_eq!(err, JwksError::DuplicateKid("dup".to_owned()));
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
        // The verified principal's SPIFFE trust domain is rooted at the tenant.
        assert_eq!(principal.trust_domain().as_str(), "spiffe://ten_acme");
        assert!(
            principal
                .trust_domain()
                .matches_tenant(principal.tenant_id())
        );
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
            Some("https://idp.oyatie.com")
        );
        assert_eq!(principal.claim("iat"), Some(&ClaimValue::Int(now)));
        assert_eq!(principal.claim("exp"), Some(&ClaimValue::Int(now + 300)));
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
            r#"{{"iss":"https://idp.oyatie.com","aud":"cloud-kms","exp":{},"tenant_id":"ten_acme","sub":"wl_a","owning_capability":"cap.x.y"}}"#,
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
            r#"{{"iss":"https://idp.oyatie.com","aud":"some-other-service","exp":{},"tenant_id":"ten_acme","sub":"wl_a","owning_capability":"cap.x.y"}}"#,
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
            r#"{{"iss":"https://evil.example","aud":"cloud-kms","exp":{},"tenant_id":"ten_acme","sub":"wl_a","owning_capability":"cap.x.y"}}"#,
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

    // ---- RFC 8725 hardening (steps 2a–4) -----------------------------------

    #[test]
    fn alg_none_is_rejected_unconditionally() {
        let now = 1_700_000_000;
        // An `alg:none` token with an empty signature segment would parse as a
        // JWS structurally; the alg check must reject it before anything else.
        // Build it by hand so the (empty) signature is not the reason.
        let header = b64url(br#"{"alg":"none","typ":"JWT","kid":"kid-1"}"#);
        let payload = b64url(valid_claims(now).as_bytes());
        // Non-empty signature segment so the structural 3-part check passes.
        let token = format!("{header}.{payload}.AAAA");
        let jwks = Jwks::new().add_key(Jwk::ec_p256("kid-1", "AA", "AA"));
        assert!(matches!(
            validate_workload_token(&token, &jwks, &config(), now),
            Err(OidcValidationError::AlgNone)
        ));
        // Case variations of the none attack are caught too.
        let header_upper = b64url(br#"{"alg":"NONE","typ":"JWT","kid":"kid-1"}"#);
        let token_upper = format!("{header_upper}.{payload}.AAAA");
        assert!(matches!(
            validate_workload_token(&token_upper, &jwks, &config(), now),
            Err(OidcValidationError::AlgNone)
        ));
    }

    #[test]
    fn hs256_against_asymmetric_kid_is_algorithm_mismatch() {
        // RS256->HS256 key-confusion: present HS256 against an asymmetric key.
        let now = 1_700_000_000;
        let header = b64url(br#"{"alg":"HS256","typ":"JWT","kid":"kid-1"}"#);
        let payload = b64url(valid_claims(now).as_bytes());
        let token = format!("{header}.{payload}.AAAA");
        let jwks = Jwks::new().add_key(Jwk::ec_p256("kid-1", "AA", "AA"));
        assert!(matches!(
            validate_workload_token(&token, &jwks, &config(), now),
            Err(OidcValidationError::AlgorithmMismatch)
        ));
    }

    #[test]
    fn missing_or_unaccepted_typ_is_rejected() {
        let now = 1_700_000_000;
        // No `typ` header at all -> cross-JWT-confusion guard fires.
        let signed = mint_with_header(
            r#"{"alg":"ES256","kid":"kid-1"}"#,
            &valid_claims(now),
            "kid-1",
        );
        let jwks = Jwks::new().add_key(signed.jwk.clone());
        assert!(matches!(
            validate_workload_token(&signed.token, &jwks, &config(), now),
            Err(OidcValidationError::InvalidType)
        ));

        // A `typ` outside the accepted set is rejected.
        let signed_wrong = mint_with_header(
            r#"{"alg":"ES256","typ":"secevent+jwt","kid":"kid-1"}"#,
            &valid_claims(now),
            "kid-1",
        );
        let jwks_wrong = Jwks::new().add_key(signed_wrong.jwk);
        assert!(matches!(
            validate_workload_token(&signed_wrong.token, &jwks_wrong, &config(), now),
            Err(OidcValidationError::InvalidType)
        ));
    }

    #[test]
    fn accepts_configured_alternate_typ() {
        // `at+jwt` is in the default accepted set (RFC 9068 access tokens).
        let now = 1_700_000_000;
        let signed = mint_with_header(
            r#"{"alg":"ES256","typ":"at+jwt","kid":"kid-1"}"#,
            &valid_claims(now),
            "kid-1",
        );
        let jwks = Jwks::new().add_key(signed.jwk);
        assert!(validate_workload_token(&signed.token, &jwks, &config(), now).is_ok());
    }

    #[test]
    fn jku_and_x5u_are_refused_unless_allowlisted() {
        let now = 1_700_000_000;
        // A `jku` header pointing anywhere is refused with the default (empty)
        // allowlist — keys come from the static JWKS, not a header URL.
        let signed = mint_with_header(
            r#"{"alg":"ES256","typ":"JWT","kid":"kid-1","jku":"https://evil.example/jwks.json"}"#,
            &valid_claims(now),
            "kid-1",
        );
        let jwks = Jwks::new().add_key(signed.jwk.clone());
        assert!(matches!(
            validate_workload_token(&signed.token, &jwks, &config(), now),
            Err(OidcValidationError::UntrustedKeySourceUrl)
        ));

        // `x5u` is treated the same way.
        let signed_x5u = mint_with_header(
            r#"{"alg":"ES256","typ":"JWT","kid":"kid-1","x5u":"https://evil.example/cert.pem"}"#,
            &valid_claims(now),
            "kid-1",
        );
        let jwks_x5u = Jwks::new().add_key(signed_x5u.jwk);
        assert!(matches!(
            validate_workload_token(&signed_x5u.token, &jwks_x5u, &config(), now),
            Err(OidcValidationError::UntrustedKeySourceUrl)
        ));

        // When the exact URL is allowlisted, the header no longer blocks.
        let signed_ok = mint_with_header(
            r#"{"alg":"ES256","typ":"JWT","kid":"kid-1","jku":"https://idp.oyatie.com/jwks.json"}"#,
            &valid_claims(now),
            "kid-1",
        );
        let jwks_ok = Jwks::new().add_key(signed_ok.jwk);
        let cfg = config().allow_key_source_url("https://idp.oyatie.com/jwks.json");
        assert!(validate_workload_token(&signed_ok.token, &jwks_ok, &cfg, now).is_ok());
    }

    #[test]
    fn kid_alg_pin_mismatch_is_rejected() {
        // The resolved JWK pins RS256, but the token presents ES256 -> mismatch
        // even though the family check would also apply for a true RSA key.
        let now = 1_700_000_000;
        let signed = mint_es256_token(&valid_claims(now), "kid-1");
        // Re-key the JWK with an exact RS256 pin (still EC material). The exact
        // alg pin must trip before signature verification.
        let pinned_jwk = signed.jwk.clone().with_alg("RS256");
        let jwks = Jwks::new().add_key(pinned_jwk);
        assert!(matches!(
            validate_workload_token(&signed.token, &jwks, &config(), now),
            Err(OidcValidationError::AlgorithmMismatch)
        ));

        // An ES256 pin on the same key validates normally.
        let signed_ok = mint_es256_token(&valid_claims(now), "kid-2");
        let jwks_ok = Jwks::new().add_key(signed_ok.jwk.clone().with_alg("ES256"));
        assert!(validate_workload_token(&signed_ok.token, &jwks_ok, &config(), now).is_ok());
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
