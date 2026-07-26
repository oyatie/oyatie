//! Human-identity OIDC issuer kernel — PURE (zero external dependencies).
//!
//! This crate is the canonical pure-domain core of the **OIDC issuer** side of
//! the identity microservice (IP-002), as distinct from the relying-party
//! verifier (`oya-shared-oidc-client-kernel`, IP-002 RP half) and the
//! workload-identity surface (`oya-identity-workload-domain`).
//!
//! Scope (per IP-002, surface only what the IP specifies):
//!
//! - the [`IssuerMetadata`] shape returned at `/.well-known/openid-configuration`
//!   per RFC 8414 §3.2,
//! - the [`Jwks`] publication shape and the [`SigningKey`] lifecycle
//!   ([`SigningKeyState::NotYetActive`] → [`SigningKeyState::Active`] →
//!   [`SigningKeyState::RotatedOut`] → [`SigningKeyState::Retired`]),
//! - the [`IdTokenClaims`] and [`AccessTokenClaims`] claim shapes per RFC 9068
//!   + the oyatie superset (tenant_id, acr, purpose, data_class),
//! - the [`JwsSigner`] *port* trait: an abstract `sign(payload, kid)` so this
//!   kernel remains pure-domain and any cryptographic implementation
//!   (ring / aws-lc-rs / HSM) plugs in from an outer adapter,
//! - the [`ClientAssertion`] + [`RefreshRequest`] request-shape validators
//!   (structural validation only — signature checks are adapter-side),
//! - clock-skew tolerance bounded by [`MAX_CLOCK_SKEW_SECONDS`].
//!
//! ## Layering invariant (ADR-0131 / architecture-boundaries gate)
//!
//! This is a `kernel` crate. It has ZERO dependencies (not even workspace
//! crates). The architecture-boundaries gate permits a `kernel` crate to
//! import only kernel/domain peers, and this one imports none so the
//! cryptographic implementation is plug-in via the [`JwsSigner`] port.
//!
//! ## Determinism
//!
//! Every function here is total and deterministic: no clock, no RNG, no I/O.
//! Time is always passed in as `now_epoch_seconds`. Crypto, JWKS fetching, and
//! token signing live in adapter crates, never here.
//!
//! ## ADR-0083 Tier 3 panic-free invariant
//!
//! Production source must not `panic!`, `unwrap`, or `expect`. Every fallible
//! operation returns a [`Result<_, IssuerError>`]. The `cfg(test)` exemption
//! is scoped to the inline `mod tests` block at the bottom of this file (see
//! the workspace lint baseline).

// ADR-0083 Tier 3: production code stays panic-free (deny in release); inline
// `mod tests` may use unwrap/expect/panic under cfg(test) only.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;

/// Schema version for the serialized issuer-metadata payload.
pub const ISSUER_METADATA_SCHEMA_VERSION: u32 = 1;

/// Schema version for the serialized [`SigningKey`] shape.
pub const SIGNING_KEY_SCHEMA_VERSION: u32 = 1;

/// Schema version for the serialized [`IdTokenClaims`] shape.
pub const ID_TOKEN_CLAIMS_SCHEMA_VERSION: u32 = 1;

/// Hard ceiling on per-verifier clock-skew tolerance. RFC 7519 §4.1.4 advises
/// "a small amount of leeway, usually no more than a few minutes"; we cap at
/// 300 seconds (5 minutes) so a misconfigured deployment cannot silently
/// accept long-stale tokens.
pub const MAX_CLOCK_SKEW_SECONDS: i64 = 300;

/// Hard ceiling on the verification grace period for rotated-out keys (24 hours).
/// A relying party must finish consuming tokens signed by a rotated-out key within
/// this window; after that, the key is no longer trusted for verification.
pub const VERIFICATION_GRACE_SECONDS: i64 = 86_400;

/// Recommended ID-token lifetime ceiling (1 hour). Mirrors the
/// `MAX_TOKEN_TTL_SECONDS` ceiling in `oya-identity-domain` for symmetry.
pub const MAX_ID_TOKEN_TTL_SECONDS: i64 = 60 * 60;

/// Recommended access-token lifetime ceiling (1 hour).
pub const MAX_ACCESS_TOKEN_TTL_SECONDS: i64 = 60 * 60;

/// Failure modes produced by this kernel. Exhaustive on purpose so callers
/// match every variant (workspace gate forbids `anyhow` in domain/kernel).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IssuerError {
    /// Issuer URL did not match the canonical `https://identity-<pack>.…`
    /// shape and was rejected at construction.
    InvalidIssuerUrl,
    /// Audience identifier was empty or whitespace-only.
    InvalidAudience,
    /// Subject identifier was empty or whitespace-only.
    InvalidSubject,
    /// `tenant_id` claim was empty (required per ADR-0244).
    MissingTenantId,
    /// `nonce` was required (OIDC §3.1.2.1) but absent or empty.
    MissingNonce,
    /// Key id (`kid`) was empty or whitespace-only.
    InvalidKid,
    /// JWS algorithm string was empty or not in the allowed set.
    UnsupportedAlgorithm {
        /// The algorithm label that was rejected.
        alg: String,
    },
    /// HS-family symmetric algorithm presented; forbidden for RP verification
    /// per RFC 8725 BCP §3.1 and refused at issuance as well so an HS-signed
    /// token never escapes the issuer.
    SymmetricAlgorithmForbidden,
    /// Issuance time was non-positive.
    InvalidIssuedAt,
    /// Expiry was at or before the issuance time.
    InvalidExpiry {
        /// Issuance time presented.
        iat: i64,
        /// Expiry presented.
        exp: i64,
    },
    /// `exp - iat` exceeded the configured ceiling.
    TokenLifetimeTooLong {
        /// `exp - iat` requested.
        requested_seconds: i64,
        /// Ceiling enforced.
        ceiling_seconds: i64,
    },
    /// Clock-skew tolerance exceeded [`MAX_CLOCK_SKEW_SECONDS`].
    ClockSkewTooWide {
        /// Tolerance requested.
        requested_seconds: i64,
        /// Ceiling enforced.
        ceiling_seconds: i64,
    },
    /// Negative clock-skew tolerance presented (nonsensical).
    NegativeClockSkew,
    /// Attempted an illegal lifecycle transition for a signing key.
    IllegalKeyTransition {
        /// State the key was in.
        from: SigningKeyState,
        /// State the caller attempted to move to.
        to: SigningKeyState,
    },
    /// `client_assertion` was empty, malformed (wrong segment count), or its
    /// type was not the canonical JWT bearer.
    MalformedClientAssertion(&'static str),
    /// Refresh request was missing the `refresh_token` or `client_id` field.
    MalformedRefreshRequest(&'static str),
    /// Introspection request was malformed (e.g. empty token).
    MalformedIntrospectionRequest(&'static str),
    /// Token presented was already expired at validation time.
    Expired {
        /// Validation clock.
        now: i64,
        /// Token expiry.
        exp: i64,
    },
    /// Token presented was not yet valid (`nbf` in the future).
    NotYetValid {
        /// Validation clock.
        now: i64,
        /// Token `nbf`.
        nbf: i64,
    },
    /// Negative grace period presented (nonsensical).
    NegativeGracePeriod,
    /// Grace period exceeded [`VERIFICATION_GRACE_SECONDS`].
    GracePeriodTooLong {
        /// Grace period requested.
        requested_seconds: i64,
        /// Ceiling enforced.
        ceiling_seconds: i64,
    },
}

impl fmt::Display for IssuerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIssuerUrl => f.write_str("invalid issuer url (expected https://…)"),
            Self::InvalidAudience => f.write_str("invalid audience"),
            Self::InvalidSubject => f.write_str("invalid subject"),
            Self::MissingTenantId => f.write_str("tenant_id claim is required and was empty"),
            Self::MissingNonce => f.write_str("nonce claim is required and was empty"),
            Self::InvalidKid => f.write_str("invalid signing-key id (kid)"),
            Self::UnsupportedAlgorithm { alg } => {
                write!(f, "unsupported JWS alg '{alg}'")
            }
            Self::SymmetricAlgorithmForbidden => {
                f.write_str("symmetric HS-family algorithms are forbidden (RFC 8725 BCP §3.1)")
            }
            Self::InvalidIssuedAt => f.write_str("iat must be greater than zero"),
            Self::InvalidExpiry { iat, exp } => {
                write!(f, "exp ({exp}) must be strictly after iat ({iat})")
            }
            Self::TokenLifetimeTooLong {
                requested_seconds,
                ceiling_seconds,
            } => write!(
                f,
                "token lifetime {requested_seconds}s exceeds ceiling {ceiling_seconds}s"
            ),
            Self::ClockSkewTooWide {
                requested_seconds,
                ceiling_seconds,
            } => write!(
                f,
                "clock skew tolerance {requested_seconds}s exceeds ceiling {ceiling_seconds}s"
            ),
            Self::NegativeClockSkew => f.write_str("clock skew tolerance must be non-negative"),
            Self::IllegalKeyTransition { from, to } => {
                write!(f, "illegal signing-key transition: {from:?} -> {to:?}")
            }
            Self::MalformedClientAssertion(reason) => {
                write!(f, "malformed client_assertion: {reason}")
            }
            Self::MalformedRefreshRequest(reason) => {
                write!(f, "malformed refresh request: {reason}")
            }
            Self::MalformedIntrospectionRequest(reason) => {
                write!(f, "malformed introspection request: {reason}")
            }
            Self::Expired { now, exp } => write!(f, "token expired (now={now}, exp={exp})"),
            Self::NotYetValid { now, nbf } => {
                write!(f, "token not yet valid (now={now}, nbf={nbf})")
            }
            Self::NegativeGracePeriod => {
                f.write_str("verification grace period must be non-negative")
            }
            Self::GracePeriodTooLong {
                requested_seconds,
                ceiling_seconds,
            } => write!(
                f,
                "verification grace period {requested_seconds}s exceeds ceiling {ceiling_seconds}s"
            ),
        }
    }
}

impl std::error::Error for IssuerError {}

/// An opaque-but-validated JWS signature produced by a [`JwsSigner`]
/// adapter. The signature payload is base64url-encoded; the kernel does not
/// interpret its bytes, only carries the value through to callers.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Signature(String);

impl Signature {
    /// Construct from a base64url-encoded signature string. Empty / whitespace
    /// values are rejected — callers must produce an actual signature.
    ///
    /// # Errors
    /// Returns [`IssuerError::MalformedClientAssertion`] when the value is
    /// empty (the variant is reused as a generic "malformed signature" sentinel
    /// because at the kernel layer all signature-presentation paths arrive via
    /// either a client assertion or token construction).
    pub fn new(value: impl Into<String>) -> Result<Self, IssuerError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(IssuerError::MalformedClientAssertion("empty signature"));
        }
        Ok(Self(value))
    }

    /// Borrow the underlying base64url-encoded string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Pluggable JWS *signing* port. Implementations (ring, aws-lc-rs, HSM-backed,
/// Zitadel-mediated) live OUTSIDE this kernel.
///
/// The signer receives the canonical signing-input bytes (`header_b64 + '.' +
/// payload_b64` per RFC 7515 §5.1) and a `kid` selector and returns the
/// base64url-encoded signature.
///
/// Adapters that hold no key material for `kid` must return
/// [`IssuerError::InvalidKid`].
pub trait JwsSigner: Send + Sync {
    /// Produce a JWS signature for `signing_input` using the private key
    /// identified by `kid`.
    ///
    /// # Errors
    /// Returns [`IssuerError::InvalidKid`] if the adapter does not hold the
    /// requested key; other errors are adapter-defined and surface through
    /// the kernel transparently.
    fn sign(&self, signing_input: &[u8], kid: &str) -> Result<Signature, IssuerError>;
}

/// ACR class enum per ADR-0189. Re-stated here (rather than importing
/// `oya-shared-oidc-client-kernel`) so this kernel keeps its zero-dependency
/// invariant; the two enums are intentionally identical in surface.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AcrLevel {
    /// `routine` — password / OIDC SSO with no additional factor.
    Routine,
    /// `elevated` — WebAuthn or hardware-bound second factor.
    Elevated,
    /// `sensitive` — re-prompted high-assurance factor within session.
    Sensitive,
    /// `critical` — hardware token + 4-eye approval per ADR-0189.
    Critical,
}

impl AcrLevel {
    /// Numeric rank for ordering comparisons; higher = stronger.
    #[must_use]
    pub const fn rank(self) -> u8 {
        match self {
            Self::Routine => 1,
            Self::Elevated => 2,
            Self::Sensitive => 3,
            Self::Critical => 4,
        }
    }

    /// Returns true if `self` is at least as strong as `floor`.
    #[must_use]
    pub fn meets(self, floor: Self) -> bool {
        self.rank() >= floor.rank()
    }

    /// Canonical lowercase serialization used in the `acr` JWT claim and in
    /// the `acr_values_supported` discovery metadata field per RFC 8414 §2.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Routine => "routine",
            Self::Elevated => "elevated",
            Self::Sensitive => "sensitive",
            Self::Critical => "critical",
        }
    }
}

/// Lifecycle state of a signing key in the JWKS rotation pipeline.
///
/// ```text
///   NotYetActive ──activate──▶ Active ──rotate_out──▶ RotatedOut ──retire──▶ Retired (terminal)
/// ```
///
/// Per [`build_jwks`], only [`SigningKeyState::Active`] and
/// [`SigningKeyState::RotatedOut`] keys are published to relying parties:
/// `NotYetActive` keys are still in pre-publication staging and `Retired`
/// keys are permanently withdrawn.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum SigningKeyState {
    /// Generated but not yet announced (operator may still abort).
    NotYetActive,
    /// The current signing key: used to sign new tokens and published in JWKS.
    Active,
    /// No longer signing new tokens but published so relying parties can
    /// finish verifying tokens that were signed before rotation.
    RotatedOut,
    /// Permanently withdrawn; not signing, not published. Terminal.
    Retired,
}

impl SigningKeyState {
    /// Whether tokens MAY be signed with a key in this state.
    #[must_use]
    pub fn is_signing(self) -> bool {
        matches!(self, Self::Active)
    }

    /// Whether a key in this state MUST be published in the JWKS so relying
    /// parties can finish verifying outstanding tokens. Returns true for
    /// `Active` (current signer) and `RotatedOut` (verify-only overlap window).
    #[must_use]
    pub fn is_published(self) -> bool {
        matches!(self, Self::Active | Self::RotatedOut)
    }

    /// Whether moving from `self` to `target` is a legal monotone transition.
    /// The state machine is strictly forward: no `RotatedOut → Active` revival,
    /// no `Retired → *` outbound, and no `NotYetActive → RotatedOut` skip.
    #[must_use]
    pub fn can_transition_to(self, target: Self) -> bool {
        matches!(
            (self, target),
            (Self::NotYetActive, Self::Active)
                | (Self::Active, Self::RotatedOut)
                | (Self::RotatedOut, Self::Retired)
        )
    }
}

/// JWS algorithm label. Only RS256 and ES256 are accepted; HS256/HS384/HS512
/// (symmetric) are forbidden at the issuer per RFC 8725 BCP §3.1.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Algorithm {
    /// RSA + SHA-256 — Zitadel default.
    Rs256,
    /// ECDSA P-256 + SHA-256 — fallback.
    Es256,
}

impl Algorithm {
    /// Parse a label into an [`Algorithm`], rejecting unsupported values.
    ///
    /// # Errors
    /// Returns [`IssuerError::SymmetricAlgorithmForbidden`] for any HS-family
    /// algorithm and [`IssuerError::UnsupportedAlgorithm`] otherwise.
    pub fn parse(label: &str) -> Result<Self, IssuerError> {
        let trimmed = label.trim();
        match trimmed {
            "RS256" => Ok(Self::Rs256),
            "ES256" => Ok(Self::Es256),
            "HS256" | "HS384" | "HS512" => Err(IssuerError::SymmetricAlgorithmForbidden),
            other => Err(IssuerError::UnsupportedAlgorithm {
                alg: other.to_owned(),
            }),
        }
    }

    /// Canonical uppercase JWS label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Rs256 => "RS256",
            Self::Es256 => "ES256",
        }
    }

    /// JWK `kty` matching this algorithm.
    #[must_use]
    pub const fn kty(self) -> &'static str {
        match self {
            Self::Rs256 => "RSA",
            Self::Es256 => "EC",
        }
    }
}

/// A canonical issuer URL (`https://identity-<pack>.<domain>` shape). The
/// kernel only enforces the `https://` scheme prefix; pack/domain validation
/// is an adapter responsibility.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct IssuerUrl(String); // data_class: PUBLIC

impl IssuerUrl {
    /// Construct, validating the `https://…` prefix.
    ///
    /// # Errors
    /// Returns [`IssuerError::InvalidIssuerUrl`] for non-`https` URLs or
    /// values shorter than `https://x`.
    pub fn new(value: impl Into<String>) -> Result<Self, IssuerError> {
        let value = value.into();
        if value.starts_with("https://") && value.len() > "https://".len() {
            Ok(Self(value))
        } else {
            Err(IssuerError::InvalidIssuerUrl)
        }
    }

    /// Borrow the URL.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for IssuerUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A single signing key as held by the issuer. Public-key material is held as
/// adapter-opaque base64url strings: the kernel never interprets the bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SigningKey {
    /// Stable key identifier echoed in JWS headers.
    kid: String, // data_class: PUBLIC
    /// JWS algorithm the key signs with.
    algorithm: Algorithm, // data_class: PUBLIC
    /// Lifecycle state per [`SigningKeyState`].
    state: SigningKeyState, // data_class: INTERNAL_ONLY
    /// Unix-seconds epoch when this key was activated (or `None` if still
    /// [`SigningKeyState::NotYetActive`]).
    activated_at_epoch_seconds: Option<i64>, // data_class: INTERNAL_ONLY
    /// Public-key material as adapter-defined base64url JWK fields. The kernel
    /// stores these opaquely so it does not need a crypto dependency.
    public_components: BTreeMap<String, String>, // data_class: PUBLIC
    /// Serialized-shape schema version.
    schema_version: u32, // data_class: PUBLIC
}

impl SigningKey {
    /// Mint a freshly generated signing key in the
    /// [`SigningKeyState::NotYetActive`] state. Public-key components are the
    /// caller's `(name, base64url-value)` map (e.g. `{"n": "...", "e": "AQAB"}`
    /// for RSA).
    ///
    /// # Errors
    /// Returns [`IssuerError::InvalidKid`] when `kid` is empty or whitespace.
    pub fn provision(
        kid: impl Into<String>,
        algorithm: Algorithm,
        public_components: BTreeMap<String, String>,
    ) -> Result<Self, IssuerError> {
        let kid = kid.into();
        if kid.trim().is_empty() {
            return Err(IssuerError::InvalidKid);
        }
        Ok(Self {
            kid,
            algorithm,
            state: SigningKeyState::NotYetActive,
            activated_at_epoch_seconds: None,
            public_components,
            schema_version: SIGNING_KEY_SCHEMA_VERSION,
        })
    }

    /// Key id.
    #[must_use]
    pub fn kid(&self) -> &str {
        &self.kid
    }

    /// Algorithm.
    #[must_use]
    pub fn algorithm(&self) -> Algorithm {
        self.algorithm
    }

    /// Current state.
    #[must_use]
    pub fn state(&self) -> SigningKeyState {
        self.state
    }

    /// Epoch-seconds at which the key transitioned to `Active`, if it ever
    /// did.
    #[must_use]
    pub fn activated_at_epoch_seconds(&self) -> Option<i64> {
        self.activated_at_epoch_seconds
    }

    /// Borrow the public-key components map (JWK shape).
    #[must_use]
    pub fn public_components(&self) -> &BTreeMap<String, String> {
        &self.public_components
    }

    /// Serialized-shape schema version.
    #[must_use]
    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// Activate the key at `now_epoch_seconds`.
    ///
    /// # Errors
    /// Returns [`IssuerError::IllegalKeyTransition`] when the current state is
    /// not [`SigningKeyState::NotYetActive`].
    pub fn activate(&mut self, now_epoch_seconds: i64) -> Result<(), IssuerError> {
        self.transition_to(SigningKeyState::Active)?;
        self.activated_at_epoch_seconds = Some(now_epoch_seconds);
        Ok(())
    }

    /// Rotate the key out of active signing (it stays published in JWKS so
    /// relying parties can finish verifying outstanding tokens).
    ///
    /// # Errors
    /// Returns [`IssuerError::IllegalKeyTransition`] when the current state is
    /// not [`SigningKeyState::Active`].
    pub fn rotate_out(&mut self) -> Result<(), IssuerError> {
        self.transition_to(SigningKeyState::RotatedOut)
    }

    /// Retire the key (permanent withdrawal). Terminal.
    ///
    /// # Errors
    /// Returns [`IssuerError::IllegalKeyTransition`] when the current state is
    /// not [`SigningKeyState::RotatedOut`].
    pub fn retire(&mut self) -> Result<(), IssuerError> {
        self.transition_to(SigningKeyState::Retired)
    }

    fn transition_to(&mut self, target: SigningKeyState) -> Result<(), IssuerError> {
        if self.state.can_transition_to(target) {
            self.state = target;
            Ok(())
        } else {
            Err(IssuerError::IllegalKeyTransition {
                from: self.state,
                to: target,
            })
        }
    }
}

/// JWKS document as published at the issuer's `jwks_uri` endpoint. The kernel
/// constructs this from a key-bundle via [`build_jwks`] which filters by
/// [`SigningKeyState::is_published`] so that `NotYetActive` keys and `Retired`
/// keys never leak into the JWKS.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Jwks {
    keys: Vec<PublishedJwk>,
}

impl Jwks {
    /// Borrow the published keys.
    #[must_use]
    pub fn keys(&self) -> &[PublishedJwk] {
        &self.keys
    }

    /// Look up a published key by `kid`.
    #[must_use]
    pub fn find(&self, kid: &str) -> Option<&PublishedJwk> {
        self.keys.iter().find(|key| key.kid == kid)
    }
}

/// A single key entry as published in [`Jwks`]. The shape mirrors RFC 7517 §4
/// JWK members; arbitrary public-key components are carried as opaque strings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishedJwk {
    /// Key id; matches the JWS header `kid`.
    pub kid: String, // data_class: PUBLIC
    /// JWK `kty` (`RSA` or `EC`).
    pub kty: String, // data_class: PUBLIC
    /// JWS `alg` the key signs / verifies with.
    pub alg: String, // data_class: PUBLIC
    /// JWK `use` parameter; always `"sig"` for issuer-published keys.
    pub key_use: String, // data_class: PUBLIC
    /// Public-key components (e.g. `n`, `e`, `x`, `y`, `crv`).
    pub public_components: BTreeMap<String, String>, // data_class: PUBLIC
}

/// Build the published JWKS from a collection of signing keys. Only
/// `Active` and `RotatedOut` keys are included; `NotYetActive` (pre-rollout)
/// and `Retired` (post-rollout) keys are filtered.
///
/// The output is **deterministic**: keys are sorted by `kid` so byte-for-byte
/// snapshots survive a key-bundle reorder.
#[must_use]
pub fn build_jwks(keys: &[SigningKey]) -> Jwks {
    let mut published: Vec<PublishedJwk> = keys
        .iter()
        .filter(|key| key.state.is_published())
        .map(|key| PublishedJwk {
            kid: key.kid.clone(),
            kty: key.algorithm.kty().to_owned(),
            alg: key.algorithm.as_str().to_owned(),
            key_use: "sig".to_owned(),
            public_components: key.public_components.clone(),
        })
        .collect();
    published.sort_by(|a, b| a.kid.cmp(&b.kid));
    Jwks { keys: published }
}

/// Locate the currently-active signing key in a bundle. Returns the first
/// key in [`SigningKeyState::Active`] state (the kernel does not enforce
/// uniqueness; callers managing the bundle SHOULD keep at most one active key
/// at a time).
#[must_use]
pub fn current_signing_key(keys: &[SigningKey]) -> Option<&SigningKey> {
    keys.iter().find(|key| key.state.is_signing())
}

/// Bounded grace period for verification of rotated-out signing keys.
///
/// Construction enforces `0 ≤ value ≤ VERIFICATION_GRACE_SECONDS` so a
/// misconfigured deployment cannot silently trust a retired key indefinitely.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct VerificationGrace(i64); // data_class: INTERNAL_ONLY

impl VerificationGrace {
    /// Construct, validating the `0..=VERIFICATION_GRACE_SECONDS` range.
    ///
    /// # Errors
    /// - [`IssuerError::NegativeGracePeriod`] when `seconds < 0`.
    /// - [`IssuerError::GracePeriodTooLong`] when `seconds > VERIFICATION_GRACE_SECONDS`.
    pub fn new(seconds: i64) -> Result<Self, IssuerError> {
        if seconds < 0 {
            return Err(IssuerError::NegativeGracePeriod);
        }
        if seconds > VERIFICATION_GRACE_SECONDS {
            return Err(IssuerError::GracePeriodTooLong {
                requested_seconds: seconds,
                ceiling_seconds: VERIFICATION_GRACE_SECONDS,
            });
        }
        Ok(Self(seconds))
    }

    /// Return the grace period in seconds.
    #[must_use]
    pub fn seconds(self) -> i64 {
        self.0
    }
}

/// Select a signing key for RP-side signature verification, honouring the
/// rotation grace overlap window.
///
/// Complements [`current_signing_key`] (Active-only signer selector) by also
/// accepting [`SigningKeyState::RotatedOut`] keys within the caller-supplied
/// grace window. This is the verify-only path: a relying party that holds a
/// token signed before rotation can still verify it while the old key is
/// within the grace window.
///
/// ## Selection semantics
///
/// 1. Locate the first key in `keys` whose `kid` matches. If none → `None`.
/// 2. [`SigningKeyState::Active`] → always `Some`.
/// 3. [`SigningKeyState::RotatedOut`]:
///    - `activated_at_epoch_seconds` is `None` → `None` (no activation record).
///    - `now_epoch_seconds - activated_at <= grace.seconds()` → `Some`.
///    - Otherwise → `None`.
/// 4. [`SigningKeyState::NotYetActive`] or [`SigningKeyState::Retired`] → `None`.
///
/// ## Clock
///
/// `now_epoch_seconds` is caller-supplied; this function performs no I/O.
#[must_use]
pub fn select_verification_key<'a>(
    keys: &'a [SigningKey],
    kid: &str,
    now_epoch_seconds: i64,
    grace: VerificationGrace,
) -> Option<&'a SigningKey> {
    let key = keys.iter().find(|k| k.kid() == kid)?;
    match key.state() {
        SigningKeyState::Active => Some(key),
        SigningKeyState::RotatedOut => {
            let activated_at = key.activated_at_epoch_seconds()?;
            let age = now_epoch_seconds.saturating_sub(activated_at);
            if age <= grace.seconds() {
                Some(key)
            } else {
                None
            }
        }
        SigningKeyState::NotYetActive | SigningKeyState::Retired => None,
    }
}

/// OIDC-discovery document per RFC 8414 §3.2 + OpenID Discovery 1.0
/// §3. Fields are the issuer-canonical subset oyatie publishes; downstream
/// adapters MAY add extension fields when serializing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssuerMetadata {
    /// `issuer` — the issuer identifier.
    pub issuer: IssuerUrl, // data_class: PUBLIC
    /// `jwks_uri` — JWKS endpoint URL.
    pub jwks_uri: String, // data_class: PUBLIC
    /// `authorization_endpoint`.
    pub authorization_endpoint: String, // data_class: PUBLIC
    /// `token_endpoint`.
    pub token_endpoint: String, // data_class: PUBLIC
    /// `userinfo_endpoint` (OpenID optional).
    pub userinfo_endpoint: Option<String>, // data_class: PUBLIC
    /// `response_types_supported`.
    pub response_types_supported: Vec<String>, // data_class: PUBLIC
    /// `subject_types_supported`.
    pub subject_types_supported: Vec<String>, // data_class: PUBLIC
    /// `id_token_signing_alg_values_supported`.
    pub id_token_signing_alg_values_supported: Vec<String>, // data_class: PUBLIC
    /// `scopes_supported`.
    pub scopes_supported: Vec<String>, // data_class: PUBLIC
    /// `acr_values_supported` per RFC 8414 §2.
    pub acr_values_supported: Vec<String>, // data_class: PUBLIC
    /// `grant_types_supported` per RFC 8414 §2.
    pub grant_types_supported: Vec<String>, // data_class: PUBLIC
    /// Serialized-shape schema version.
    pub schema_version: u32, // data_class: PUBLIC
}

/// Build the OIDC-discovery document for an issuer. The endpoints are
/// constructed by appending the canonical OIDC paths to `issuer`; downstream
/// adapters MAY override individual endpoints via builder-style mutation if
/// their deployment uses non-canonical paths.
///
/// `signing_algs` is the set of algorithms the JWKS currently offers and is
/// reflected as `id_token_signing_alg_values_supported`. ACR values are taken
/// directly from [`AcrLevel`] enumeration so they cannot drift.
///
/// # Errors
/// Returns [`IssuerError::InvalidIssuerUrl`] only if `issuer.as_str()` is
/// somehow empty (constructed [`IssuerUrl`] values are already validated so
/// this is a belt-and-braces check for `unsafe` future extensions).
pub fn build_issuer_metadata(
    issuer: IssuerUrl,
    signing_algs: &[Algorithm],
) -> Result<IssuerMetadata, IssuerError> {
    if issuer.as_str().is_empty() {
        return Err(IssuerError::InvalidIssuerUrl);
    }
    let base = issuer.as_str().trim_end_matches('/').to_owned();
    Ok(IssuerMetadata {
        jwks_uri: format!("{base}/oauth/v2/keys"),
        authorization_endpoint: format!("{base}/oauth/authorize"),
        token_endpoint: format!("{base}/oauth/v2/token"),
        userinfo_endpoint: Some(format!("{base}/oauth/v2/userinfo")),
        response_types_supported: vec!["code".to_owned()],
        subject_types_supported: vec!["public".to_owned()],
        id_token_signing_alg_values_supported: signing_algs
            .iter()
            .map(|alg| alg.as_str().to_owned())
            .collect(),
        scopes_supported: vec![
            "openid".to_owned(),
            "profile".to_owned(),
            "email".to_owned(),
        ],
        acr_values_supported: vec![
            AcrLevel::Routine.as_str().to_owned(),
            AcrLevel::Elevated.as_str().to_owned(),
            AcrLevel::Sensitive.as_str().to_owned(),
            AcrLevel::Critical.as_str().to_owned(),
        ],
        grant_types_supported: vec![
            "authorization_code".to_owned(),
            "refresh_token".to_owned(),
            "client_credentials".to_owned(),
            "urn:ietf:params:oauth:grant-type:jwt-bearer".to_owned(),
        ],
        issuer,
        schema_version: ISSUER_METADATA_SCHEMA_VERSION,
    })
}

/// Subject-claim values (`sub`) carry the principal identifier. The kernel
/// validates non-empty + non-whitespace.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Subject(String); // data_class: PII_IDENTIFYING

impl Subject {
    /// Construct, validating non-emptiness.
    ///
    /// # Errors
    /// Returns [`IssuerError::InvalidSubject`] for an empty or whitespace-only
    /// value.
    pub fn new(value: impl Into<String>) -> Result<Self, IssuerError> {
        let value = value.into();
        if value.trim().is_empty() {
            Err(IssuerError::InvalidSubject)
        } else {
            Ok(Self(value))
        }
    }

    /// Borrow the subject string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Subject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Audience-claim value (`aud`). The kernel canonicalises every audience to
/// at least one entry; constructors reject empty inputs.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Audience(Vec<String>); // data_class: PUBLIC

impl Audience {
    /// Build a single-audience claim.
    ///
    /// # Errors
    /// Returns [`IssuerError::InvalidAudience`] for an empty value.
    pub fn single(value: impl Into<String>) -> Result<Self, IssuerError> {
        let value = value.into();
        if value.trim().is_empty() {
            Err(IssuerError::InvalidAudience)
        } else {
            Ok(Self(vec![value]))
        }
    }

    /// Build a multi-audience claim. Empty entries are rejected; the overall
    /// list must be non-empty.
    ///
    /// # Errors
    /// Returns [`IssuerError::InvalidAudience`] for an empty list or any
    /// empty-string entry.
    pub fn many(values: Vec<String>) -> Result<Self, IssuerError> {
        if values.is_empty() || values.iter().any(|value| value.trim().is_empty()) {
            Err(IssuerError::InvalidAudience)
        } else {
            Ok(Self(values))
        }
    }

    /// Borrow the audience list.
    #[must_use]
    pub fn as_slice(&self) -> &[String] {
        &self.0
    }

    /// Whether the audience contains `expected`.
    #[must_use]
    pub fn contains(&self, expected: &str) -> bool {
        self.0.iter().any(|value| value == expected)
    }
}

/// Specification for an ID-token to mint. The kernel validates the shape and
/// returns [`IdTokenClaims`]; signing is delegated to a [`JwsSigner`] adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdTokenSpec {
    /// `iss` — the issuer URL.
    pub issuer: IssuerUrl, // data_class: PUBLIC
    /// `aud` — the relying-party audience.
    pub audience: Audience, // data_class: PUBLIC
    /// `sub` — the principal identifier.
    pub subject: Subject, // data_class: PII_IDENTIFYING
    /// `tenant_id` — the principal's tenant per ADR-0244.
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    /// `iat` — issuance time (epoch seconds, > 0).
    pub issued_at_epoch_seconds: i64, // data_class: INTERNAL_ONLY
    /// `exp` — expiry time (epoch seconds, strictly > `issued_at`).
    pub expires_at_epoch_seconds: i64, // data_class: INTERNAL_ONLY
    /// `nonce` — required for `code` flow per OIDC §3.1.2.1.
    pub nonce: String, // data_class: INTERNAL_ONLY
    /// `acr` — authentication-context class per ADR-0189.
    pub acr: AcrLevel, // data_class: INTERNAL_ONLY
    /// `purpose` — purpose-binding label.
    pub purpose: Option<String>, // data_class: INTERNAL_ONLY
    /// `data_class` — declared data class for callers' validation.
    pub data_class: Option<String>, // data_class: PUBLIC
}

/// Validated ID-token claims ready for serialisation + signing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdTokenClaims {
    /// `iss`.
    pub iss: String, // data_class: PUBLIC
    /// `aud`.
    pub aud: Vec<String>, // data_class: PUBLIC
    /// `sub`.
    pub sub: String, // data_class: PII_IDENTIFYING
    /// `iat`.
    pub iat: i64, // data_class: INTERNAL_ONLY
    /// `exp`.
    pub exp: i64, // data_class: INTERNAL_ONLY
    /// `nbf` — defaulted to `iat`.
    pub nbf: i64, // data_class: INTERNAL_ONLY
    /// `nonce`.
    pub nonce: String, // data_class: INTERNAL_ONLY
    /// `tenant_id` per ADR-0244.
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    /// `acr` per ADR-0189 (lowercase).
    pub acr: String, // data_class: INTERNAL_ONLY
    /// `purpose`.
    pub purpose: Option<String>, // data_class: INTERNAL_ONLY
    /// `data_class`.
    pub data_class: Option<String>, // data_class: PUBLIC
    /// Serialized-shape schema version.
    pub schema_version: u32, // data_class: PUBLIC
}

/// Build the [`IdTokenClaims`] from a spec, applying ADR-0083 panic-free
/// validation (every error path returns a typed [`IssuerError`]).
///
/// # Errors
/// - [`IssuerError::InvalidIssuedAt`] when `iat <= 0`.
/// - [`IssuerError::InvalidExpiry`] when `exp <= iat`.
/// - [`IssuerError::TokenLifetimeTooLong`] when `exp - iat > MAX_ID_TOKEN_TTL_SECONDS`.
/// - [`IssuerError::MissingTenantId`] when `tenant_id` is empty/whitespace.
/// - [`IssuerError::MissingNonce`] when `nonce` is empty/whitespace.
pub fn build_id_token_claims(spec: IdTokenSpec) -> Result<IdTokenClaims, IssuerError> {
    if spec.issued_at_epoch_seconds <= 0 {
        return Err(IssuerError::InvalidIssuedAt);
    }
    if spec.expires_at_epoch_seconds <= spec.issued_at_epoch_seconds {
        return Err(IssuerError::InvalidExpiry {
            iat: spec.issued_at_epoch_seconds,
            exp: spec.expires_at_epoch_seconds,
        });
    }
    let lifetime = spec.expires_at_epoch_seconds - spec.issued_at_epoch_seconds;
    if lifetime > MAX_ID_TOKEN_TTL_SECONDS {
        return Err(IssuerError::TokenLifetimeTooLong {
            requested_seconds: lifetime,
            ceiling_seconds: MAX_ID_TOKEN_TTL_SECONDS,
        });
    }
    if spec.tenant_id.trim().is_empty() {
        return Err(IssuerError::MissingTenantId);
    }
    if spec.nonce.trim().is_empty() {
        return Err(IssuerError::MissingNonce);
    }
    Ok(IdTokenClaims {
        iss: spec.issuer.as_str().to_owned(),
        aud: spec.audience.as_slice().to_vec(),
        sub: spec.subject.as_str().to_owned(),
        iat: spec.issued_at_epoch_seconds,
        exp: spec.expires_at_epoch_seconds,
        nbf: spec.issued_at_epoch_seconds,
        nonce: spec.nonce,
        tenant_id: spec.tenant_id,
        acr: spec.acr.as_str().to_owned(),
        purpose: spec.purpose,
        data_class: spec.data_class,
        schema_version: ID_TOKEN_CLAIMS_SCHEMA_VERSION,
    })
}

/// Specification for an access-token to mint. RFC 9068 §2.1 access-token
/// claims; the oyatie superset adds `tenant_id` + `purpose` + `data_class`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessTokenSpec {
    /// `iss`.
    pub issuer: IssuerUrl, // data_class: PUBLIC
    /// `aud` — the resource server identifier.
    pub audience: Audience, // data_class: PUBLIC
    /// `sub` — the principal.
    pub subject: Subject, // data_class: PII_IDENTIFYING
    /// `tenant_id`.
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    /// `scope` claim, space-joined per RFC 6749 §3.3.
    pub scopes: Vec<String>, // data_class: INTERNAL_ONLY
    /// `iat`.
    pub issued_at_epoch_seconds: i64, // data_class: INTERNAL_ONLY
    /// `exp`.
    pub expires_at_epoch_seconds: i64, // data_class: INTERNAL_ONLY
    /// `purpose`.
    pub purpose: Option<String>, // data_class: INTERNAL_ONLY
    /// `data_class`.
    pub data_class: Option<String>, // data_class: PUBLIC
}

/// Validated access-token claims ready for serialisation + signing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessTokenClaims {
    /// `iss`.
    pub iss: String, // data_class: PUBLIC
    /// `aud`.
    pub aud: Vec<String>, // data_class: PUBLIC
    /// `sub`.
    pub sub: String, // data_class: PII_IDENTIFYING
    /// `iat`.
    pub iat: i64, // data_class: INTERNAL_ONLY
    /// `exp`.
    pub exp: i64, // data_class: INTERNAL_ONLY
    /// `nbf`.
    pub nbf: i64, // data_class: INTERNAL_ONLY
    /// `scope` (space-joined).
    pub scope: String, // data_class: INTERNAL_ONLY
    /// `tenant_id`.
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    /// `purpose`.
    pub purpose: Option<String>, // data_class: INTERNAL_ONLY
    /// `data_class`.
    pub data_class: Option<String>, // data_class: PUBLIC
    /// `token_type` per RFC 9068; always `"at+jwt"`.
    pub token_type: String, // data_class: PUBLIC
}

/// Build [`AccessTokenClaims`] from a spec.
///
/// # Errors
/// Same set as [`build_id_token_claims`], minus the `nonce` requirement
/// (access tokens have no `nonce`), plus
/// [`IssuerError::TokenLifetimeTooLong`] against [`MAX_ACCESS_TOKEN_TTL_SECONDS`].
pub fn build_access_token_claims(spec: AccessTokenSpec) -> Result<AccessTokenClaims, IssuerError> {
    if spec.issued_at_epoch_seconds <= 0 {
        return Err(IssuerError::InvalidIssuedAt);
    }
    if spec.expires_at_epoch_seconds <= spec.issued_at_epoch_seconds {
        return Err(IssuerError::InvalidExpiry {
            iat: spec.issued_at_epoch_seconds,
            exp: spec.expires_at_epoch_seconds,
        });
    }
    let lifetime = spec.expires_at_epoch_seconds - spec.issued_at_epoch_seconds;
    if lifetime > MAX_ACCESS_TOKEN_TTL_SECONDS {
        return Err(IssuerError::TokenLifetimeTooLong {
            requested_seconds: lifetime,
            ceiling_seconds: MAX_ACCESS_TOKEN_TTL_SECONDS,
        });
    }
    if spec.tenant_id.trim().is_empty() {
        return Err(IssuerError::MissingTenantId);
    }
    Ok(AccessTokenClaims {
        iss: spec.issuer.as_str().to_owned(),
        aud: spec.audience.as_slice().to_vec(),
        sub: spec.subject.as_str().to_owned(),
        iat: spec.issued_at_epoch_seconds,
        exp: spec.expires_at_epoch_seconds,
        nbf: spec.issued_at_epoch_seconds,
        scope: spec.scopes.join(" "),
        tenant_id: spec.tenant_id,
        purpose: spec.purpose,
        data_class: spec.data_class,
        token_type: "at+jwt".to_owned(),
    })
}

/// Bounded clock-skew tolerance for token validation. Construction enforces
/// `0 ≤ value ≤ MAX_CLOCK_SKEW_SECONDS` so a misconfigured deployment cannot
/// silently extend token lifetime by hours.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ClockSkewTolerance(i64); // data_class: INTERNAL_ONLY

impl ClockSkewTolerance {
    /// Construct, validating the `0..=MAX_CLOCK_SKEW_SECONDS` range.
    ///
    /// # Errors
    /// - [`IssuerError::NegativeClockSkew`] when `seconds < 0`.
    /// - [`IssuerError::ClockSkewTooWide`] when `seconds > MAX_CLOCK_SKEW_SECONDS`.
    pub fn new(seconds: i64) -> Result<Self, IssuerError> {
        if seconds < 0 {
            return Err(IssuerError::NegativeClockSkew);
        }
        if seconds > MAX_CLOCK_SKEW_SECONDS {
            return Err(IssuerError::ClockSkewTooWide {
                requested_seconds: seconds,
                ceiling_seconds: MAX_CLOCK_SKEW_SECONDS,
            });
        }
        Ok(Self(seconds))
    }

    /// Borrow the tolerance in seconds.
    #[must_use]
    pub fn seconds(self) -> i64 {
        self.0
    }
}

impl Default for ClockSkewTolerance {
    /// Default tolerance: 60 seconds (matches RFC 7519 §4.1.4 "small amount
    /// of leeway").
    fn default() -> Self {
        Self(60)
    }
}

/// Result of validating an outstanding token against the current clock.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TokenTemporalStatus {
    /// Token is within its validity window (`nbf ≤ now ≤ exp`, modulo skew).
    Valid,
}

/// Check whether `iat`/`nbf`/`exp` are consistent with `now` modulo a bounded
/// clock-skew tolerance. This is the kernel's CLOCK-SKEW validator; signature
/// verification belongs in the adapter.
///
/// # Errors
/// - [`IssuerError::Expired`] when `now > exp + skew`.
/// - [`IssuerError::NotYetValid`] when `now + skew < nbf`.
pub fn check_temporal_window(
    now_epoch_seconds: i64,
    nbf: i64,
    exp: i64,
    skew: ClockSkewTolerance,
) -> Result<TokenTemporalStatus, IssuerError> {
    let skew = skew.seconds();
    if now_epoch_seconds > exp.saturating_add(skew) {
        return Err(IssuerError::Expired {
            now: now_epoch_seconds,
            exp,
        });
    }
    if now_epoch_seconds.saturating_add(skew) < nbf {
        return Err(IssuerError::NotYetValid {
            now: now_epoch_seconds,
            nbf,
        });
    }
    Ok(TokenTemporalStatus::Valid)
}

/// Structural validation of an OAuth 2.0 `client_assertion` (RFC 7521 +
/// RFC 7523): three-segment JWS shape with non-empty header / payload /
/// signature segments. Signature verification is adapter-side.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientAssertion {
    /// Base64url header segment.
    pub header_b64url: String, // data_class: PUBLIC
    /// Base64url payload segment.
    pub payload_b64url: String, // data_class: INTERNAL_ONLY
    /// Base64url signature segment.
    pub signature_b64url: String, // data_class: SECRET
}

impl ClientAssertion {
    /// Parse the compact-JWS shape from `value`. Refuses fewer or more than
    /// three dot-separated segments, and refuses empty segments.
    ///
    /// # Errors
    /// Returns [`IssuerError::MalformedClientAssertion`] for any structural
    /// fault.
    pub fn parse(value: &str) -> Result<Self, IssuerError> {
        if value.trim().is_empty() {
            return Err(IssuerError::MalformedClientAssertion("empty"));
        }
        let segments: Vec<&str> = value.split('.').collect();
        if segments.len() != 3 {
            return Err(IssuerError::MalformedClientAssertion(
                "client_assertion must have exactly three dot-separated segments",
            ));
        }
        let header = segments[0];
        let payload = segments[1];
        let signature = segments[2];
        if header.is_empty() {
            return Err(IssuerError::MalformedClientAssertion(
                "empty header segment",
            ));
        }
        if payload.is_empty() {
            return Err(IssuerError::MalformedClientAssertion(
                "empty payload segment",
            ));
        }
        if signature.is_empty() {
            return Err(IssuerError::MalformedClientAssertion(
                "empty signature segment",
            ));
        }
        Ok(Self {
            header_b64url: header.to_owned(),
            payload_b64url: payload.to_owned(),
            signature_b64url: signature.to_owned(),
        })
    }

    /// Reconstruct the canonical signing input (`header.payload`) for
    /// signature verification by an adapter.
    #[must_use]
    pub fn signing_input(&self) -> String {
        format!("{}.{}", self.header_b64url, self.payload_b64url)
    }
}

/// Refresh-token request shape per RFC 6749 §6 (`grant_type=refresh_token`).
/// Kernel validates structure only — token-identity lookup and rotation
/// bookkeeping are adapter responsibilities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefreshRequest {
    /// `refresh_token` field.
    pub refresh_token: String, // data_class: SECRET
    /// `client_id` field.
    pub client_id: String, // data_class: INTERNAL_ONLY
    /// `scope` field (space-joined), optional.
    pub requested_scope: Option<String>, // data_class: INTERNAL_ONLY
}

impl RefreshRequest {
    /// Validate the shape of a refresh request.
    ///
    /// # Errors
    /// Returns [`IssuerError::MalformedRefreshRequest`] with a static reason
    /// pointer when any required field is empty.
    pub fn validate(
        refresh_token: impl Into<String>,
        client_id: impl Into<String>,
        requested_scope: Option<String>,
    ) -> Result<Self, IssuerError> {
        let refresh_token = refresh_token.into();
        let client_id = client_id.into();
        if refresh_token.trim().is_empty() {
            return Err(IssuerError::MalformedRefreshRequest(
                "refresh_token must not be empty",
            ));
        }
        if client_id.trim().is_empty() {
            return Err(IssuerError::MalformedRefreshRequest(
                "client_id must not be empty",
            ));
        }
        if let Some(ref scope) = requested_scope
            && scope.trim().is_empty()
        {
            return Err(IssuerError::MalformedRefreshRequest(
                "scope, when present, must not be empty",
            ));
        }
        Ok(Self {
            refresh_token,
            client_id,
            requested_scope,
        })
    }
}

/// Hint about the token type presented for introspection (RFC 7662 §2.1).
/// The kernel accepts only the two standardised values; free-text hint
/// parsing is an adapter responsibility.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenTypeHint {
    /// `access_token` hint.
    AccessToken,
    /// `refresh_token` hint.
    RefreshToken,
}

/// Structural representation of an RFC 7662 §2.1 introspection request.
/// Validates that the token is non-empty; `token_type_hint` is typed so the
/// allowed-set constraint is enforced by the type system.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntrospectionRequest {
    /// The opaque token string to introspect.
    pub token: String, // data_class: SECRET
    /// Optional hint about the token type.
    pub token_type_hint: Option<TokenTypeHint>, // data_class: INTERNAL_ONLY
}

impl IntrospectionRequest {
    /// Validate the shape of an introspection request.
    ///
    /// # Errors
    /// Returns [`IssuerError::MalformedIntrospectionRequest`] when `token` is
    /// empty or whitespace-only.
    pub fn validate(
        token: impl Into<String>,
        token_type_hint: Option<TokenTypeHint>,
    ) -> Result<Self, IssuerError> {
        let token = token.into();
        if token.trim().is_empty() {
            return Err(IssuerError::MalformedIntrospectionRequest(
                "token must not be empty",
            ));
        }
        Ok(Self {
            token,
            token_type_hint,
        })
    }
}

/// Disclosed claim set for an active RFC 7662 introspection response.
/// Passed to [`IntrospectionResponse::active`] to avoid the 8-argument
/// constructor that trips `clippy::too_many_arguments`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveIntrospectionClaims {
    /// `sub` — subject identifier.
    pub sub: String, // data_class: PII_IDENTIFYING
    /// `aud` — intended audience list.
    pub aud: Vec<String>, // data_class: PUBLIC
    /// `exp` — expiry (epoch seconds).
    pub exp: i64, // data_class: INTERNAL_ONLY
    /// `iat` — issuance time (epoch seconds).
    pub iat: i64, // data_class: INTERNAL_ONLY
    /// `scope` — space-separated scope string (None when empty).
    pub scope: Option<String>, // data_class: INTERNAL_ONLY
    /// `client_id` — OAuth 2.0 client identifier.
    pub client_id: Option<String>, // data_class: INTERNAL_ONLY
    /// `tenant_id` — oyatie tenant identifier (ADR-0244 superset).
    pub tenant_id: Option<String>, // data_class: INTERNAL_ONLY
    /// `token_type` — e.g. `"at+jwt"` for RFC 9068 access tokens.
    pub token_type: Option<String>, // data_class: PUBLIC
}

/// RFC 7662 §2.2 introspection response.
///
/// Privacy rule: when `active` is `false`, all other fields MUST be absent.
/// The [`IntrospectionResponse::inactive`] constructor enforces this invariant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntrospectionResponse {
    /// Whether the token is currently active (RFC 7662 §2.2).
    pub active: bool, // data_class: PUBLIC
    /// `sub` — subject identifier.
    pub sub: Option<String>, // data_class: PII_IDENTIFYING
    /// `aud` — intended audience list.
    pub aud: Option<Vec<String>>, // data_class: PUBLIC
    /// `exp` — expiry (epoch seconds).
    pub exp: Option<i64>, // data_class: INTERNAL_ONLY
    /// `iat` — issuance time (epoch seconds).
    pub iat: Option<i64>, // data_class: INTERNAL_ONLY
    /// `scope` — space-separated scope string.
    pub scope: Option<String>, // data_class: INTERNAL_ONLY
    /// `client_id` — OAuth 2.0 client identifier.
    pub client_id: Option<String>, // data_class: INTERNAL_ONLY
    /// `tenant_id` — oyatie tenant identifier (ADR-0244 superset).
    pub tenant_id: Option<String>, // data_class: INTERNAL_ONLY
    /// `token_type` — e.g. `"at+jwt"` for RFC 9068 access tokens.
    pub token_type: Option<String>, // data_class: PUBLIC
}

impl IntrospectionResponse {
    /// Construct an inactive response.
    ///
    /// RFC 7662 §2.2 privacy rule: only `{"active": false}` is disclosed for
    /// unknown or invalid tokens. All optional fields are explicitly `None`.
    #[must_use]
    pub fn inactive() -> Self {
        Self {
            active: false,
            sub: None,
            aud: None,
            exp: None,
            iat: None,
            scope: None,
            client_id: None,
            tenant_id: None,
            token_type: None,
        }
    }

    /// Construct an active response from the disclosed claim set.
    #[must_use]
    pub fn active(claims: ActiveIntrospectionClaims) -> Self {
        Self {
            active: true,
            sub: Some(claims.sub),
            aud: Some(claims.aud),
            exp: Some(claims.exp),
            iat: Some(claims.iat),
            scope: claims.scope,
            client_id: claims.client_id,
            tenant_id: claims.tenant_id,
            token_type: claims.token_type,
        }
    }
}

/// Build an RFC 7662 introspection response from an [`AccessTokenClaims`]
/// value and the caller's validation clock.
///
/// Reuses [`check_temporal_window`] + [`TokenTemporalStatus`] for the
/// active/inactive verdict. Expired and not-yet-valid tokens collapse to
/// `{"active": false}` per RFC 7662 §2.2 without leaking error details.
///
/// `scope` promotion: an empty `scope` field in the claims is promoted to
/// `None` (a token with no scope discloses nothing rather than an empty
/// string).
///
/// # Errors
/// Currently infallible (always returns `Ok`); typed `Result<_, IssuerError>`
/// for forward compatibility.
pub fn build_introspection_response(
    claims: &AccessTokenClaims,
    now_epoch_seconds: i64,
    skew: ClockSkewTolerance,
) -> Result<IntrospectionResponse, IssuerError> {
    match check_temporal_window(now_epoch_seconds, claims.nbf, claims.exp, skew) {
        Err(_) => Ok(IntrospectionResponse::inactive()),
        Ok(TokenTemporalStatus::Valid) => {
            let scope = if claims.scope.is_empty() {
                None
            } else {
                Some(claims.scope.clone())
            };
            Ok(IntrospectionResponse::active(ActiveIntrospectionClaims {
                sub: claims.sub.clone(),
                aud: claims.aud.clone(),
                exp: claims.exp,
                iat: claims.iat,
                scope,
                client_id: None,
                tenant_id: Some(claims.tenant_id.clone()),
                token_type: Some(claims.token_type.clone()),
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rsa_components() -> BTreeMap<String, String> {
        let mut components = BTreeMap::new();
        components.insert("n".to_owned(), "dummy-modulus".to_owned());
        components.insert("e".to_owned(), "AQAB".to_owned());
        components
    }

    #[test]
    fn issuer_url_rejects_non_https() {
        assert_eq!(
            IssuerUrl::new("http://identity-kr.oyatie.dev"),
            Err(IssuerError::InvalidIssuerUrl)
        );
        assert_eq!(IssuerUrl::new(""), Err(IssuerError::InvalidIssuerUrl));
        assert!(IssuerUrl::new("https://identity-kr.oyatie.com").is_ok());
    }

    #[test]
    fn acr_levels_have_total_order() {
        assert!(AcrLevel::Critical.meets(AcrLevel::Routine));
        assert!(!AcrLevel::Routine.meets(AcrLevel::Critical));
        assert!(AcrLevel::Sensitive.meets(AcrLevel::Elevated));
    }

    #[test]
    fn algorithm_rejects_symmetric() {
        assert_eq!(
            Algorithm::parse("HS256"),
            Err(IssuerError::SymmetricAlgorithmForbidden)
        );
        assert_eq!(
            Algorithm::parse("HS512"),
            Err(IssuerError::SymmetricAlgorithmForbidden)
        );
        match Algorithm::parse("PS256") {
            Err(IssuerError::UnsupportedAlgorithm { alg }) => assert_eq!(alg, "PS256"),
            other => panic!("unexpected {other:?}"),
        }
        assert_eq!(Algorithm::parse("RS256").expect("ok"), Algorithm::Rs256);
    }

    #[test]
    fn signing_key_lifecycle_is_monotone() {
        let mut key =
            SigningKey::provision("k1", Algorithm::Rs256, rsa_components()).expect("provision ok");
        assert_eq!(key.state(), SigningKeyState::NotYetActive);
        key.activate(1_700_000_000).expect("activate ok");
        assert_eq!(key.state(), SigningKeyState::Active);
        assert_eq!(key.activated_at_epoch_seconds(), Some(1_700_000_000));
        key.rotate_out().expect("rotate ok");
        assert_eq!(key.state(), SigningKeyState::RotatedOut);
        key.retire().expect("retire ok");
        assert_eq!(key.state(), SigningKeyState::Retired);
        // Retired is terminal; further transitions refused.
        assert_eq!(
            key.rotate_out(),
            Err(IssuerError::IllegalKeyTransition {
                from: SigningKeyState::Retired,
                to: SigningKeyState::RotatedOut,
            })
        );
    }

    #[test]
    fn signing_key_rejects_state_skips() {
        let mut key =
            SigningKey::provision("k1", Algorithm::Rs256, rsa_components()).expect("provision ok");
        // NotYetActive -> RotatedOut is not a legal edge.
        assert!(key.rotate_out().is_err());
        // NotYetActive -> Retired is not a legal edge.
        assert!(key.retire().is_err());
    }

    #[test]
    fn signing_key_rejects_empty_kid() {
        assert_eq!(
            SigningKey::provision("   ", Algorithm::Rs256, rsa_components()),
            Err(IssuerError::InvalidKid)
        );
    }

    #[test]
    fn id_token_claims_round_trip() {
        let issuer = IssuerUrl::new("https://identity-kr.oyatie.com").expect("ok");
        let audience = Audience::single("oya-application").expect("ok");
        let subject = Subject::new("usr_abc").expect("ok");
        let claims = build_id_token_claims(IdTokenSpec {
            issuer,
            audience,
            subject,
            tenant_id: "ten_acme".to_owned(),
            issued_at_epoch_seconds: 1_700_000_000,
            expires_at_epoch_seconds: 1_700_003_600,
            nonce: "n-abc".to_owned(),
            acr: AcrLevel::Elevated,
            purpose: Some("login".to_owned()),
            data_class: Some("INTERNAL_ONLY".to_owned()),
        })
        .expect("build ok");
        assert_eq!(claims.iss, "https://identity-kr.oyatie.com");
        assert_eq!(claims.aud, vec!["oya-application".to_owned()]);
        assert_eq!(claims.sub, "usr_abc");
        assert_eq!(claims.tenant_id, "ten_acme");
        assert_eq!(claims.acr, "elevated");
        assert_eq!(claims.nonce, "n-abc");
        assert_eq!(claims.nbf, 1_700_000_000);
        assert_eq!(claims.schema_version, ID_TOKEN_CLAIMS_SCHEMA_VERSION);
    }

    #[test]
    fn id_token_rejects_missing_nonce_and_tenant() {
        let issuer = IssuerUrl::new("https://identity-kr.oyatie.com").expect("ok");
        let audience = Audience::single("oya-application").expect("ok");
        let subject = Subject::new("usr_abc").expect("ok");
        let spec = IdTokenSpec {
            issuer: issuer.clone(),
            audience: audience.clone(),
            subject: subject.clone(),
            tenant_id: "".to_owned(),
            issued_at_epoch_seconds: 1,
            expires_at_epoch_seconds: 2,
            nonce: "n".to_owned(),
            acr: AcrLevel::Routine,
            purpose: None,
            data_class: None,
        };
        assert_eq!(
            build_id_token_claims(spec),
            Err(IssuerError::MissingTenantId)
        );

        let spec = IdTokenSpec {
            issuer,
            audience,
            subject,
            tenant_id: "ten_x".to_owned(),
            issued_at_epoch_seconds: 1,
            expires_at_epoch_seconds: 2,
            nonce: "   ".to_owned(),
            acr: AcrLevel::Routine,
            purpose: None,
            data_class: None,
        };
        assert_eq!(build_id_token_claims(spec), Err(IssuerError::MissingNonce));
    }

    #[test]
    fn build_issuer_metadata_is_deterministic() {
        let issuer = IssuerUrl::new("https://identity-kr.oyatie.com").expect("ok");
        let meta = build_issuer_metadata(issuer.clone(), &[Algorithm::Rs256, Algorithm::Es256])
            .expect("build ok");
        assert_eq!(meta.issuer.as_str(), issuer.as_str());
        assert_eq!(
            meta.jwks_uri,
            "https://identity-kr.oyatie.com/oauth/v2/keys".to_owned()
        );
        assert_eq!(
            meta.id_token_signing_alg_values_supported,
            vec!["RS256", "ES256"]
        );
        assert!(meta.acr_values_supported.contains(&"critical".to_owned()));
        assert!(
            meta.grant_types_supported
                .contains(&"refresh_token".to_owned())
        );
    }

    #[test]
    fn build_jwks_filters_state() {
        let mut active =
            SigningKey::provision("k1", Algorithm::Rs256, rsa_components()).expect("ok");
        active.activate(1).expect("ok");

        let pre = SigningKey::provision("k2", Algorithm::Es256, rsa_components()).expect("ok");

        let mut rotated =
            SigningKey::provision("k3", Algorithm::Rs256, rsa_components()).expect("ok");
        rotated.activate(2).expect("ok");
        rotated.rotate_out().expect("ok");

        let mut retired =
            SigningKey::provision("k4", Algorithm::Rs256, rsa_components()).expect("ok");
        retired.activate(3).expect("ok");
        retired.rotate_out().expect("ok");
        retired.retire().expect("ok");

        let jwks = build_jwks(&[active, pre, rotated, retired]);
        let kids: Vec<&str> = jwks.keys().iter().map(|k| k.kid.as_str()).collect();
        // Only Active (k1) + RotatedOut (k3); sorted by kid.
        assert_eq!(kids, vec!["k1", "k3"]);
        let k1 = jwks.find("k1").expect("present");
        assert_eq!(k1.alg, "RS256");
        assert_eq!(k1.kty, "RSA");
        assert_eq!(k1.key_use, "sig");
    }

    #[test]
    fn current_signing_key_picks_first_active() {
        let pre = SigningKey::provision("k0", Algorithm::Rs256, rsa_components()).expect("ok");
        let mut active =
            SigningKey::provision("k1", Algorithm::Rs256, rsa_components()).expect("ok");
        active.activate(1).expect("ok");
        let keys = vec![pre, active];
        let current = current_signing_key(&keys).expect("present");
        assert_eq!(current.kid(), "k1");
    }

    #[test]
    fn check_temporal_window_bounds() {
        let skew = ClockSkewTolerance::new(60).expect("ok");
        // now within window
        assert!(check_temporal_window(100, 50, 200, skew).is_ok());
        // now > exp + skew → Expired
        match check_temporal_window(300, 50, 200, skew) {
            Err(IssuerError::Expired { now, exp }) => {
                assert_eq!(now, 300);
                assert_eq!(exp, 200);
            }
            other => panic!("unexpected {other:?}"),
        }
        // now + skew < nbf → NotYetValid
        match check_temporal_window(10, 200, 300, skew) {
            Err(IssuerError::NotYetValid { now, nbf }) => {
                assert_eq!(now, 10);
                assert_eq!(nbf, 200);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn clock_skew_is_bounded() {
        assert_eq!(
            ClockSkewTolerance::new(-1),
            Err(IssuerError::NegativeClockSkew)
        );
        match ClockSkewTolerance::new(MAX_CLOCK_SKEW_SECONDS + 1) {
            Err(IssuerError::ClockSkewTooWide {
                requested_seconds,
                ceiling_seconds,
            }) => {
                assert_eq!(requested_seconds, MAX_CLOCK_SKEW_SECONDS + 1);
                assert_eq!(ceiling_seconds, MAX_CLOCK_SKEW_SECONDS);
            }
            other => panic!("unexpected {other:?}"),
        }
        assert_eq!(ClockSkewTolerance::default().seconds(), 60);
    }

    #[test]
    fn client_assertion_parses_three_segments() {
        let ok = ClientAssertion::parse("h.p.s").expect("ok");
        assert_eq!(ok.header_b64url, "h");
        assert_eq!(ok.payload_b64url, "p");
        assert_eq!(ok.signature_b64url, "s");
        assert_eq!(ok.signing_input(), "h.p");

        assert!(matches!(
            ClientAssertion::parse(""),
            Err(IssuerError::MalformedClientAssertion(_))
        ));
        assert!(matches!(
            ClientAssertion::parse("only.two"),
            Err(IssuerError::MalformedClientAssertion(_))
        ));
        assert!(matches!(
            ClientAssertion::parse("h..s"),
            Err(IssuerError::MalformedClientAssertion(_))
        ));
        assert!(matches!(
            ClientAssertion::parse("a.b.c.d"),
            Err(IssuerError::MalformedClientAssertion(_))
        ));
    }

    #[test]
    fn refresh_request_validates_shape() {
        let ok =
            RefreshRequest::validate("rt", "client", Some("openid email".to_owned())).expect("ok");
        assert_eq!(ok.refresh_token, "rt");
        assert_eq!(ok.client_id, "client");

        assert!(matches!(
            RefreshRequest::validate("", "client", None),
            Err(IssuerError::MalformedRefreshRequest(_))
        ));
        assert!(matches!(
            RefreshRequest::validate("rt", "   ", None),
            Err(IssuerError::MalformedRefreshRequest(_))
        ));
        assert!(matches!(
            RefreshRequest::validate("rt", "client", Some("   ".to_owned())),
            Err(IssuerError::MalformedRefreshRequest(_))
        ));
    }

    #[test]
    fn signature_rejects_empty() {
        assert!(matches!(
            Signature::new("   "),
            Err(IssuerError::MalformedClientAssertion(_))
        ));
        assert_eq!(Signature::new("sig").expect("ok").as_str(), "sig");
    }

    // --- oidc-introspect-1: IntrospectionRequest ---

    #[test]
    fn introspection_request_rejects_empty_token() {
        assert!(matches!(
            IntrospectionRequest::validate("", None),
            Err(IssuerError::MalformedIntrospectionRequest(_))
        ));
    }

    #[test]
    fn introspection_request_rejects_whitespace_token() {
        assert!(matches!(
            IntrospectionRequest::validate("   ", None),
            Err(IssuerError::MalformedIntrospectionRequest(_))
        ));
    }

    #[test]
    fn introspection_request_accepts_token_without_hint() {
        let req = IntrospectionRequest::validate("opaque-token-123", None).expect("ok");
        assert_eq!(req.token, "opaque-token-123");
        assert_eq!(req.token_type_hint, None);
    }

    #[test]
    fn introspection_request_accepts_token_with_hint() {
        let req =
            IntrospectionRequest::validate("tok", Some(TokenTypeHint::AccessToken)).expect("ok");
        assert_eq!(req.token, "tok");
        assert_eq!(req.token_type_hint, Some(TokenTypeHint::AccessToken));

        let req2 =
            IntrospectionRequest::validate("tok2", Some(TokenTypeHint::RefreshToken)).expect("ok");
        assert_eq!(req2.token_type_hint, Some(TokenTypeHint::RefreshToken));
    }

    // --- oidc-introspect-2: IntrospectionResponse ---

    #[test]
    fn introspection_response_inactive_has_no_disclosed_fields() {
        let resp = IntrospectionResponse::inactive();
        assert!(!resp.active);
        assert!(resp.sub.is_none());
        assert!(resp.aud.is_none());
        assert!(resp.exp.is_none());
        assert!(resp.iat.is_none());
        assert!(resp.scope.is_none());
        assert!(resp.client_id.is_none());
        assert!(resp.tenant_id.is_none());
        assert!(resp.token_type.is_none());
    }

    #[test]
    fn introspection_response_active_carries_disclosed_claims() {
        let resp = IntrospectionResponse::active(ActiveIntrospectionClaims {
            sub: "usr_abc".to_owned(),
            aud: vec!["oya-api".to_owned()],
            exp: 1_700_003_600,
            iat: 1_700_000_000,
            scope: Some("openid email".to_owned()),
            client_id: Some("client-1".to_owned()),
            tenant_id: Some("ten_acme".to_owned()),
            token_type: Some("at+jwt".to_owned()),
        });
        assert!(resp.active);
        assert_eq!(resp.sub.as_deref(), Some("usr_abc"));
        assert_eq!(resp.aud.as_deref(), Some(["oya-api".to_owned()].as_slice()));
        assert_eq!(resp.exp, Some(1_700_003_600));
        assert_eq!(resp.iat, Some(1_700_000_000));
        assert_eq!(resp.scope.as_deref(), Some("openid email"));
        assert_eq!(resp.client_id.as_deref(), Some("client-1"));
        assert_eq!(resp.tenant_id.as_deref(), Some("ten_acme"));
        assert_eq!(resp.token_type.as_deref(), Some("at+jwt"));
    }

    // --- oidc-introspect-3: build_introspection_response ---

    fn sample_access_token_claims(iat: i64, nbf: i64, exp: i64) -> AccessTokenClaims {
        AccessTokenClaims {
            iss: "https://identity-kr.oyatie.com".to_owned(),
            aud: vec!["oya-api".to_owned()],
            sub: "usr_abc".to_owned(),
            iat,
            exp,
            nbf,
            scope: "openid email".to_owned(),
            tenant_id: "ten_acme".to_owned(),
            purpose: None,
            data_class: None,
            token_type: "at+jwt".to_owned(),
        }
    }

    #[test]
    fn build_introspection_response_active_token() {
        let now = 1_700_001_000_i64;
        let claims = sample_access_token_claims(1_700_000_000, 1_700_000_000, 1_700_003_600);
        let skew = ClockSkewTolerance::new(60).expect("ok");
        let resp = build_introspection_response(&claims, now, skew).expect("ok");
        assert!(resp.active);
        assert_eq!(resp.sub.as_deref(), Some("usr_abc"));
        assert_eq!(resp.aud.as_deref(), Some(["oya-api".to_owned()].as_slice()));
        assert_eq!(resp.exp, Some(1_700_003_600));
        assert_eq!(resp.iat, Some(1_700_000_000));
        assert_eq!(resp.scope.as_deref(), Some("openid email"));
        assert_eq!(resp.client_id, None);
        assert_eq!(resp.tenant_id.as_deref(), Some("ten_acme"));
        assert_eq!(resp.token_type.as_deref(), Some("at+jwt"));
    }

    #[test]
    fn build_introspection_response_expired_token() {
        // now > exp + skew → inactive
        let now = 1_700_003_661_i64; // exp=1_700_003_600, skew=60 → boundary is 1_700_003_660
        let claims = sample_access_token_claims(1_700_000_000, 1_700_000_000, 1_700_003_600);
        let skew = ClockSkewTolerance::new(60).expect("ok");
        let resp = build_introspection_response(&claims, now, skew).expect("ok");
        assert!(!resp.active);
        assert!(resp.sub.is_none());
        assert!(resp.exp.is_none());
    }

    #[test]
    fn build_introspection_response_not_yet_valid_token() {
        // now + skew < nbf → inactive
        let now = 39_i64; // now + skew(60) = 99 < nbf(100)
        let claims = sample_access_token_claims(100, 100, 300);
        let skew = ClockSkewTolerance::new(60).expect("ok");
        let resp = build_introspection_response(&claims, now, skew).expect("ok");
        assert!(!resp.active);
        assert!(resp.sub.is_none());
        assert!(resp.exp.is_none());
    }

    #[test]
    fn build_introspection_response_empty_scope_becomes_none() {
        let now = 150_i64;
        let mut claims = sample_access_token_claims(100, 100, 300);
        claims.scope = String::new();
        let skew = ClockSkewTolerance::new(0).expect("ok");
        let resp = build_introspection_response(&claims, now, skew).expect("ok");
        assert!(resp.active);
        assert_eq!(resp.scope, None);
    }

    // ── select_verification_key tests ─────────────────────────────────────────

    fn active_key(kid: &str, activated_at: i64) -> SigningKey {
        let mut k = SigningKey::provision(kid, Algorithm::Rs256, rsa_components()).expect("ok");
        k.activate(activated_at).expect("ok");
        k
    }

    fn rotated_key(kid: &str, activated_at: i64) -> SigningKey {
        let mut k = active_key(kid, activated_at);
        k.rotate_out().expect("ok");
        k
    }

    fn retired_key(kid: &str, activated_at: i64) -> SigningKey {
        let mut k = rotated_key(kid, activated_at);
        k.retire().expect("ok");
        k
    }

    #[test]
    fn verification_key_active_accept() {
        let keys = vec![active_key("k1", 1_000)];
        let grace = VerificationGrace::new(3_600).expect("ok");
        let found = select_verification_key(&keys, "k1", 999_999, grace);
        assert_eq!(found.map(|k| k.kid()), Some("k1"));
    }

    #[test]
    fn verification_key_rotated_within_grace() {
        // activated_at=1000, now=1000+3600=4600, grace=3600 → age==grace → accept
        let keys = vec![rotated_key("k1", 1_000)];
        let grace = VerificationGrace::new(3_600).expect("ok");
        let found = select_verification_key(&keys, "k1", 4_600, grace);
        assert_eq!(found.map(|k| k.kid()), Some("k1"));
    }

    #[test]
    fn verification_key_rotated_past_grace() {
        // activated_at=1000, now=4601, grace=3600 → age=3601 > grace → reject
        let keys = vec![rotated_key("k1", 1_000)];
        let grace = VerificationGrace::new(3_600).expect("ok");
        let found = select_verification_key(&keys, "k1", 4_601, grace);
        assert!(found.is_none());
    }

    #[test]
    fn verification_key_retired_reject() {
        let keys = vec![retired_key("k1", 1_000)];
        let grace = VerificationGrace::new(86_400).expect("ok");
        assert!(select_verification_key(&keys, "k1", 2_000, grace).is_none());
    }

    #[test]
    fn verification_key_not_yet_active_reject() {
        let keys =
            vec![SigningKey::provision("k1", Algorithm::Rs256, rsa_components()).expect("ok")];
        let grace = VerificationGrace::new(86_400).expect("ok");
        assert!(select_verification_key(&keys, "k1", 2_000, grace).is_none());
    }

    #[test]
    fn verification_key_unknown_kid() {
        let keys = vec![active_key("k1", 1_000)];
        let grace = VerificationGrace::new(3_600).expect("ok");
        assert!(select_verification_key(&keys, "k-unknown", 2_000, grace).is_none());
    }

    #[test]
    fn verification_grace_ceiling_bound() {
        match VerificationGrace::new(VERIFICATION_GRACE_SECONDS + 1) {
            Err(IssuerError::GracePeriodTooLong {
                requested_seconds,
                ceiling_seconds,
            }) => {
                assert_eq!(requested_seconds, VERIFICATION_GRACE_SECONDS + 1);
                assert_eq!(ceiling_seconds, VERIFICATION_GRACE_SECONDS);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn verification_grace_at_ceiling_is_ok() {
        assert!(VerificationGrace::new(VERIFICATION_GRACE_SECONDS).is_ok());
    }

    #[test]
    fn verification_grace_negative() {
        assert_eq!(
            VerificationGrace::new(-1),
            Err(IssuerError::NegativeGracePeriod)
        );
    }

    #[test]
    fn verification_grace_zero_exact_boundary() {
        // grace=0: only now == activated_at is accepted
        let keys = vec![rotated_key("k1", 1_000)];
        let grace = VerificationGrace::new(0).expect("ok");
        // now == activated_at → age=0 ≤ grace(0) → accept
        assert!(select_verification_key(&keys, "k1", 1_000, grace).is_some());
        // now > activated_at → age=1 > grace(0) → reject
        assert!(select_verification_key(&keys, "k1", 1_001, grace).is_none());
    }
}
