//! IP-007 — the tenant-scoped claim set and its validation rules.
//!
//! What this module does: models the claims a tenant-scoped access token
//! carries, encodes them into a canonical, unambiguous wire form, and decides
//! whether a decoded claim set is acceptable to a relying party at a given
//! instant.
//!
//! What this module does NOT do: verify a signature. See the crate-level Gaps
//! paragraph. [`ClaimsPolicy::validate`] answers "are these claims well-formed,
//! in-window, and addressed to me", never "did the issuer really mint this".
//!
//! Time is a parameter everywhere: `now` is supplied by the caller, so the
//! expiry boundary is testable to the second and two runs of the same input can
//! never disagree.

use std::collections::BTreeMap;

use crate::{IsolationKernelError, JwtIssuer, JwtVerifier};

/// Seconds since the Unix epoch. A plain `i64` so pre-epoch instants and
/// arithmetic below zero stay representable instead of wrapping.
pub type UnixSeconds = i64;

/// Claim key names on the wire. Kept as constants so encoder, decoder and
/// tests cannot drift apart.
pub const CLAIM_ISSUER: &str = "iss";
/// Subject claim key.
pub const CLAIM_SUBJECT: &str = "sub";
/// Audience claim key.
pub const CLAIM_AUDIENCE: &str = "aud";
/// Tenant claim key — the tenant this token is scoped to.
pub const CLAIM_TENANT: &str = "tenant";
/// Scope claim key.
///
/// The VALUE is a length-prefixed list ([`encode_scope_list`]), not the
/// space-delimited OAuth 2.0 string. That is a deliberate departure: a
/// space-delimited value makes the encoder and the decoder disagree about how
/// many scopes there are the moment one scope contains whitespace, and the
/// disagreement runs in the privilege-GRANTING direction. See
/// [`encode_scope_list`].
pub const CLAIM_SCOPE: &str = "scope";
/// Issued-at claim key.
pub const CLAIM_ISSUED_AT: &str = "iat";
/// Not-before claim key.
pub const CLAIM_NOT_BEFORE: &str = "nbf";
/// Expiry claim key.
pub const CLAIM_EXPIRES_AT: &str = "exp";
/// Signing-key fingerprint claim key (the `kid` seam a real verifier uses).
pub const CLAIM_KEY_FINGERPRINT: &str = "kid";

/// Longest tenant identifier this crate will treat as well-formed.
pub const MAX_TENANT_LEN: usize = 64;

/// The tenant-id prefix, matching `tenancy/core/domain`'s `Tenant::new`.
pub const TENANT_ID_PREFIX: &str = "ten_";

/// A tenant-scoped set of access-token claims.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantClaims {
    pub issuer: String,          // data_class: INTERNAL_ONLY
    pub subject: String,         // data_class: TENANT_SCOPED
    pub audience: String,        // data_class: INTERNAL_ONLY
    pub tenant: String,          // data_class: TENANT_SCOPED
    pub scopes: Vec<String>,     // data_class: TENANT_SCOPED
    pub issued_at: UnixSeconds,  // data_class: INTERNAL_ONLY
    pub not_before: UnixSeconds, // data_class: INTERNAL_ONLY
    pub expires_at: UnixSeconds, // data_class: INTERNAL_ONLY
    pub key_fingerprint: String, // data_class: INTERNAL_ONLY
}

/// What a relying party requires of a claim set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimsPolicy {
    pub expected_issuer: String,      // data_class: INTERNAL_ONLY
    pub expected_audience: String,    // data_class: INTERNAL_ONLY
    pub required_scopes: Vec<String>, // data_class: INTERNAL_ONLY
    pub max_lifetime_seconds: i64,    // data_class: INTERNAL_ONLY
    pub leeway_seconds: i64,          // data_class: INTERNAL_ONLY
}

/// A claim set that has passed [`ClaimsPolicy::validate`].
///
/// Parse, don't validate: the inner claims are unreachable except through this
/// wrapper, so a caller cannot hold a `ValidatedClaims` it did not earn.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedClaims {
    claims: TenantClaims,
}

/// Why a claim set was refused.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimsError {
    /// The `iss` claim was empty.
    IssuerMissing,
    /// The `iss` claim did not match the relying party's expectation.
    IssuerMismatch { expected: String, found: String },
    /// The `aud` claim was empty.
    AudienceMissing,
    /// The `aud` claim did not name this relying party.
    AudienceMismatch { expected: String, found: String },
    /// The `sub` claim was empty.
    SubjectMissing,
    /// The `tenant` claim was empty.
    TenantMissing,
    /// The `tenant` claim was present but not a well-formed tenant id.
    TenantMalformed { tenant: String },
    /// A scope string was empty or carried whitespace.
    ScopeMalformed { scope: String },
    /// A scope the relying party requires was absent.
    MissingScope { scope: String },
    /// `exp` was not strictly after `iat`.
    InvalidWindow {
        issued_at: UnixSeconds,
        expires_at: UnixSeconds,
    },
    /// The token's lifetime exceeded the policy ceiling.
    LifetimeTooLong {
        lifetime_seconds: i64,
        max_lifetime_seconds: i64,
    },
    /// `iat` was in the future relative to `now`.
    IssuedInFuture {
        now: UnixSeconds,
        issued_at: UnixSeconds,
    },
    /// `now` was before `nbf`.
    NotYetValid {
        now: UnixSeconds,
        not_before: UnixSeconds,
    },
    /// `now` was at or after `exp`. RFC 7519 says `exp` is exclusive: a token
    /// is expired ON its expiry second, not one second later.
    Expired {
        now: UnixSeconds,
        expires_at: UnixSeconds,
    },
    /// The `kid` claim was empty, so no verifier could ever select a key.
    KeyFingerprintMissing,
    /// The canonical claim encoding could not be parsed.
    MalformedEncoding,
    /// A required claim key was absent from the decoded pairs.
    MissingClaim { name: String },
    /// A numeric claim was not an integer.
    NonNumericClaim { name: String },
    /// A claim key appeared more than once. Refused rather than resolved,
    /// because every resolution rule is a rule about which tenant wins.
    DuplicateClaim { name: String },
}

impl core::fmt::Display for ClaimsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::IssuerMissing => f.write_str("iss claim is empty"),
            Self::IssuerMismatch { expected, found } => {
                write!(f, "iss {found:?} does not match expected {expected:?}")
            }
            Self::AudienceMissing => f.write_str("aud claim is empty"),
            Self::AudienceMismatch { expected, found } => {
                write!(f, "aud {found:?} does not match expected {expected:?}")
            }
            Self::SubjectMissing => f.write_str("sub claim is empty"),
            Self::TenantMissing => f.write_str("tenant claim is empty"),
            Self::TenantMalformed { tenant } => write!(
                f,
                "tenant {tenant:?} does not satisfy the isolation-policy tenant-id rule \
                 (\"ten_\" + [a-z0-9_-]+, at most {MAX_TENANT_LEN} bytes); note this rule is \
                 STRICTER than the one tenancy/core/domain applies at tenant creation, so an \
                 id can be valid in the tenant store and still be refused here"
            ),
            Self::ScopeMalformed { scope } => write!(f, "scope {scope:?} is malformed"),
            Self::MissingScope { scope } => write!(f, "required scope {scope:?} is absent"),
            Self::InvalidWindow {
                issued_at,
                expires_at,
            } => write!(f, "exp {expires_at} is not after iat {issued_at}"),
            Self::LifetimeTooLong {
                lifetime_seconds,
                max_lifetime_seconds,
            } => write!(
                f,
                "token lifetime {lifetime_seconds}s exceeds the {max_lifetime_seconds}s ceiling"
            ),
            Self::IssuedInFuture { now, issued_at } => {
                write!(f, "iat {issued_at} is after now {now}")
            }
            Self::NotYetValid { now, not_before } => {
                write!(f, "now {now} is before nbf {not_before}")
            }
            Self::Expired { now, expires_at } => {
                write!(f, "now {now} is at or after exp {expires_at}")
            }
            Self::KeyFingerprintMissing => f.write_str("kid claim is empty"),
            Self::MalformedEncoding => f.write_str("claim encoding is malformed"),
            Self::MissingClaim { name } => write!(f, "claim {name:?} is absent"),
            Self::NonNumericClaim { name } => write!(f, "claim {name:?} is not an integer"),
            Self::DuplicateClaim { name } => {
                write!(f, "claim {name:?} appears more than once")
            }
        }
    }
}

impl std::error::Error for ClaimsError {}

impl From<ClaimsError> for IsolationKernelError {
    fn from(source: ClaimsError) -> Self {
        Self::ClaimsRejected { source }
    }
}

/// A failure anywhere on the token path: at the port, or in the claims.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenError {
    /// The issuer or verifier port failed.
    Port(IsolationKernelError),
    /// The claims decoded but were unacceptable.
    Claims(ClaimsError),
}

impl core::fmt::Display for TokenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Port(error) => write!(f, "token port failure: {error}"),
            Self::Claims(error) => write!(f, "token claims rejected: {error}"),
        }
    }
}

impl std::error::Error for TokenError {}

impl From<IsolationKernelError> for TokenError {
    fn from(error: IsolationKernelError) -> Self {
        Self::Port(error)
    }
}

impl From<ClaimsError> for TokenError {
    fn from(error: ClaimsError) -> Self {
        Self::Claims(error)
    }
}

/// Whether `tenant` is a well-formed tenant id: `ten_` followed by at least one
/// `[a-z0-9_-]` character, at most [`MAX_TENANT_LEN`] bytes overall.
///
/// # This rule is STRICTER than `tenancy/core/domain`
///
/// `tenancy/core/domain`'s `Tenant::new` accepts any id where
/// `id.starts_with("ten_") && id.len() > 4` — no character class and no length
/// ceiling. This crate additionally requires the suffix to be `[a-z0-9_-]` and
/// the whole id to fit [`MAX_TENANT_LEN`]. So `ten_ACME`, `ten_acme.eu`,
/// `ten_a b` and any id over 64 bytes are all ids the domain crate will mint
/// and this crate will refuse.
///
/// The narrowing is deliberate, not accidental: [`ValidatedClaims::tenant`] is
/// the value an adapter interpolates into
/// `SET app.current_tenant_id = '<tenant>'`, so an id carrying a quote, a
/// semicolon or whitespace is an injection surface at the session boundary, and
/// rejection is the boundary while quoting is only a mitigation.
///
/// It is nonetheless a live cross-crate divergence and a real operational risk:
/// a tenant created with an id outside this class authenticates nowhere. The
/// crate-level Gaps paragraph names it, and
/// `tenant_id_rule_is_deliberately_stricter_than_the_domain_crate` pins the
/// exact divergence set rather than asserting a parity that does not exist. The
/// fix belongs in `tenancy/core/domain` (narrow `Tenant::new` to this class),
/// which this crate cannot reach: it holds no dependency on that crate and the
/// lockfile is frozen for this wave.
pub fn tenant_id_is_well_formed(tenant: &str) -> bool {
    let Some(suffix) = tenant.strip_prefix(TENANT_ID_PREFIX) else {
        return false;
    };
    if suffix.is_empty() || tenant.len() > MAX_TENANT_LEN {
        return false;
    }
    suffix
        .chars()
        .all(|ch| ch == '_' || ch == '-' || ch.is_ascii_lowercase() || ch.is_ascii_digit())
}

/// Whether `scope` can be carried as a single scope: non-empty and free of
/// whitespace.
///
/// [`ClaimsPolicy::validate`] enforces this, and [`issue_tenant_token`] enforces
/// it too so that a claim set the relying party would reject can never be
/// turned into a token in the first place.
pub fn scope_is_well_formed(scope: &str) -> bool {
    !scope.is_empty() && !scope.chars().any(char::is_whitespace)
}

impl ClaimsPolicy {
    /// A strict policy: exact issuer and audience match, zero clock leeway, and
    /// a one-hour lifetime ceiling.
    pub fn strict(
        expected_issuer: impl Into<String>,
        expected_audience: impl Into<String>,
        required_scopes: Vec<String>,
    ) -> Self {
        Self {
            expected_issuer: expected_issuer.into(),
            expected_audience: expected_audience.into(),
            required_scopes,
            max_lifetime_seconds: 3_600,
            leeway_seconds: 0,
        }
    }

    /// Allow `seconds` of clock skew on the `nbf` and `exp` boundaries.
    pub fn with_leeway_seconds(mut self, seconds: i64) -> Self {
        self.leeway_seconds = seconds.max(0);
        self
    }

    /// Set the maximum acceptable `exp - iat`.
    pub fn with_max_lifetime_seconds(mut self, seconds: i64) -> Self {
        self.max_lifetime_seconds = seconds;
        self
    }

    /// Decide whether `claims` are acceptable at `now`.
    ///
    /// Checks run shape-first, then addressing, then the time window, then
    /// scopes, so the returned error names the most fundamental problem rather
    /// than whichever check happened to be cheapest.
    ///
    /// This validates claim SHAPE only. It does not and cannot establish that
    /// the claims were minted by `expected_issuer`; see the crate Gaps section.
    pub fn validate(
        &self,
        claims: &TenantClaims,
        now: UnixSeconds,
    ) -> Result<ValidatedClaims, ClaimsError> {
        if claims.issuer.is_empty() {
            return Err(ClaimsError::IssuerMissing);
        }
        if claims.audience.is_empty() {
            return Err(ClaimsError::AudienceMissing);
        }
        if claims.subject.is_empty() {
            return Err(ClaimsError::SubjectMissing);
        }
        if claims.tenant.is_empty() {
            return Err(ClaimsError::TenantMissing);
        }
        if !tenant_id_is_well_formed(&claims.tenant) {
            return Err(ClaimsError::TenantMalformed {
                tenant: claims.tenant.clone(),
            });
        }
        if claims.key_fingerprint.is_empty() {
            return Err(ClaimsError::KeyFingerprintMissing);
        }
        for scope in &claims.scopes {
            if !scope_is_well_formed(scope) {
                return Err(ClaimsError::ScopeMalformed {
                    scope: scope.clone(),
                });
            }
        }

        if claims.issuer != self.expected_issuer {
            return Err(ClaimsError::IssuerMismatch {
                expected: self.expected_issuer.clone(),
                found: claims.issuer.clone(),
            });
        }
        if claims.audience != self.expected_audience {
            return Err(ClaimsError::AudienceMismatch {
                expected: self.expected_audience.clone(),
                found: claims.audience.clone(),
            });
        }

        if claims.expires_at <= claims.issued_at {
            return Err(ClaimsError::InvalidWindow {
                issued_at: claims.issued_at,
                expires_at: claims.expires_at,
            });
        }
        let lifetime = claims.expires_at.saturating_sub(claims.issued_at);
        if lifetime > self.max_lifetime_seconds {
            return Err(ClaimsError::LifetimeTooLong {
                lifetime_seconds: lifetime,
                max_lifetime_seconds: self.max_lifetime_seconds,
            });
        }
        if claims.issued_at > now.saturating_add(self.leeway_seconds) {
            return Err(ClaimsError::IssuedInFuture {
                now,
                issued_at: claims.issued_at,
            });
        }
        if now.saturating_add(self.leeway_seconds) < claims.not_before {
            return Err(ClaimsError::NotYetValid {
                now,
                not_before: claims.not_before,
            });
        }
        if now.saturating_sub(self.leeway_seconds) >= claims.expires_at {
            return Err(ClaimsError::Expired {
                now,
                expires_at: claims.expires_at,
            });
        }

        for scope in &self.required_scopes {
            if !claims.scopes.iter().any(|held| held == scope) {
                return Err(ClaimsError::MissingScope {
                    scope: scope.clone(),
                });
            }
        }

        Ok(ValidatedClaims {
            claims: claims.clone(),
        })
    }
}

impl ValidatedClaims {
    /// The tenant this token is scoped to — the value an RLS session should set
    /// into [`crate::rls::CANONICAL_TENANT_SETTING`].
    pub fn tenant(&self) -> &str {
        &self.claims.tenant
    }

    /// The authenticated subject.
    pub fn subject(&self) -> &str {
        &self.claims.subject
    }

    /// The scopes the token carries.
    pub fn scopes(&self) -> &[String] {
        &self.claims.scopes
    }

    /// Borrow the whole validated claim set.
    pub fn claims(&self) -> &TenantClaims {
        &self.claims
    }

    /// Consume the wrapper, yielding the validated claims.
    pub fn into_inner(self) -> TenantClaims {
        self.claims
    }
}

impl TenantClaims {
    /// Every scope this claim set carries that could not survive the wire, in
    /// order. Empty when the claim set is safe to encode.
    pub fn malformed_scopes(&self) -> Vec<&str> {
        self.scopes
            .iter()
            .filter(|scope| !scope_is_well_formed(scope))
            .map(String::as_str)
            .collect()
    }

    /// Refuse a claim set a relying party would reject on shape grounds, before
    /// it is ever encoded.
    ///
    /// Only scope shape is checked here: it is the one field whose problems the
    /// wire form could otherwise launder. Freshness, addressing and tenant
    /// well-formedness stay with [`ClaimsPolicy::validate`], because they are
    /// the relying party's questions, not the encoder's.
    pub fn check_encodable(&self) -> Result<(), ClaimsError> {
        match self.malformed_scopes().first() {
            Some(scope) => Err(ClaimsError::ScopeMalformed {
                scope: (*scope).to_owned(),
            }),
            None => Ok(()),
        }
    }

    /// The canonical claim pairs, sorted by key, ready for a [`JwtIssuer`].
    pub fn to_claim_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = vec![
            (CLAIM_ISSUER.to_owned(), self.issuer.clone()),
            (CLAIM_SUBJECT.to_owned(), self.subject.clone()),
            (CLAIM_AUDIENCE.to_owned(), self.audience.clone()),
            (CLAIM_TENANT.to_owned(), self.tenant.clone()),
            (CLAIM_SCOPE.to_owned(), encode_scope_list(&self.scopes)),
            (CLAIM_ISSUED_AT.to_owned(), self.issued_at.to_string()),
            (CLAIM_NOT_BEFORE.to_owned(), self.not_before.to_string()),
            (CLAIM_EXPIRES_AT.to_owned(), self.expires_at.to_string()),
            (
                CLAIM_KEY_FINGERPRINT.to_owned(),
                self.key_fingerprint.clone(),
            ),
        ];
        pairs.sort();
        pairs
    }

    /// Rebuild a claim set from decoded pairs, failing on any absent,
    /// non-numeric or REPEATED claim.
    ///
    /// Duplicates are refused rather than resolved. Collecting the pairs into a
    /// map would silently keep the last value, and the claim most worth
    /// duplicating is `tenant`: a payload carrying two of them would decode to
    /// one tenant here while an audit log, rate limiter or authz pre-check
    /// reading the first occurrence saw another. Every other malformed input in
    /// this module has a named error, and so does this one.
    pub fn from_claim_pairs(pairs: &[(String, String)]) -> Result<Self, ClaimsError> {
        let mut map: BTreeMap<&str, &str> = BTreeMap::new();
        for (key, value) in pairs {
            if map.insert(key.as_str(), value.as_str()).is_some() {
                return Err(ClaimsError::DuplicateClaim { name: key.clone() });
            }
        }

        fn text(map: &BTreeMap<&str, &str>, name: &str) -> Result<String, ClaimsError> {
            map.get(name)
                .map(|value| (*value).to_owned())
                .ok_or_else(|| ClaimsError::MissingClaim {
                    name: name.to_owned(),
                })
        }

        fn number(map: &BTreeMap<&str, &str>, name: &str) -> Result<UnixSeconds, ClaimsError> {
            text(map, name)?
                .parse::<UnixSeconds>()
                .map_err(|_| ClaimsError::NonNumericClaim {
                    name: name.to_owned(),
                })
        }

        Ok(Self {
            issuer: text(&map, CLAIM_ISSUER)?,
            subject: text(&map, CLAIM_SUBJECT)?,
            audience: text(&map, CLAIM_AUDIENCE)?,
            tenant: text(&map, CLAIM_TENANT)?,
            scopes: decode_scope_list(&text(&map, CLAIM_SCOPE)?)?,
            issued_at: number(&map, CLAIM_ISSUED_AT)?,
            not_before: number(&map, CLAIM_NOT_BEFORE)?,
            expires_at: number(&map, CLAIM_EXPIRES_AT)?,
            key_fingerprint: text(&map, CLAIM_KEY_FINGERPRINT)?,
        })
    }
}

/// Encode claim pairs into the canonical, unambiguous wire form.
///
/// Each field is written length-prefixed as `len:key:len:value`, so a value
/// containing `:` or `.` cannot be mistaken for a delimiter. Pairs are sorted
/// by key first, so encoding is deterministic.
pub fn encode_claim_pairs(pairs: &[(String, String)]) -> String {
    let mut ordered: Vec<&(String, String)> = pairs.iter().collect();
    ordered.sort();
    let mut out = String::new();
    for (key, value) in ordered {
        out.push_str(&key.len().to_string());
        out.push(':');
        out.push_str(key);
        out.push(':');
        out.push_str(&value.len().to_string());
        out.push(':');
        out.push_str(value);
    }
    out
}

/// Parse a length prefix, accepting exactly ONE spelling of each number.
///
/// `str::parse::<usize>` also accepts `+3` and `03`, which would make `3:iss`,
/// `03:iss` and `+3:iss` three byte-different encodings of one claim set. The
/// wire form is documented as canonical, and a form is not canonical if it has
/// several spellings: any dedup, replay or revocation list keyed on the token
/// string would be bypassable by re-padding a length.
fn parse_canonical_length(text: &str) -> Result<usize, ClaimsError> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ClaimsError::MalformedEncoding);
    }
    if text.len() > 1 && text.starts_with('0') {
        return Err(ClaimsError::MalformedEncoding);
    }
    text.parse::<usize>()
        .map_err(|_| ClaimsError::MalformedEncoding)
}

fn take_length_prefixed(rest: &str) -> Result<(String, &str), ClaimsError> {
    let (len_text, after) = rest.split_once(':').ok_or(ClaimsError::MalformedEncoding)?;
    let len = parse_canonical_length(len_text)?;
    let field = after.get(..len).ok_or(ClaimsError::MalformedEncoding)?;
    let tail = after.get(len..).ok_or(ClaimsError::MalformedEncoding)?;
    Ok((field.to_owned(), tail))
}

/// Encode a scope list as `len:scope` repeated, so it survives the round trip
/// byte for byte whatever the scopes contain.
///
/// The obvious encoding — `scopes.join(" ")` decoded with `split_whitespace` —
/// is not a round trip, and it fails in the privilege-GRANTING direction: the
/// single scope `"tenancy.read admin"` encodes to `"tenancy.read admin"` and
/// decodes to the TWO scopes `["tenancy.read", "admin"]`, so passing a claim
/// set through this crate's own issuer would mint a token authorizing strictly
/// more than the claim set it was built from. Length-prefixing makes the
/// encoder and the decoder agree about how many scopes there are, so a scope
/// carrying whitespace stays one malformed scope and
/// [`ClaimsPolicy::validate`] rejects it exactly as it would have on the direct
/// path.
pub fn encode_scope_list(scopes: &[String]) -> String {
    let mut out = String::new();
    for scope in scopes {
        out.push_str(&scope.len().to_string());
        out.push(':');
        out.push_str(scope);
    }
    out
}

/// Decode the list [`encode_scope_list`] produces. The empty string is the
/// empty list.
pub fn decode_scope_list(encoded: &str) -> Result<Vec<String>, ClaimsError> {
    let mut rest = encoded;
    let mut scopes = Vec::new();
    while !rest.is_empty() {
        let (scope, tail) = take_length_prefixed(rest)?;
        scopes.push(scope);
        rest = tail;
    }
    Ok(scopes)
}

/// Decode the canonical wire form produced by [`encode_claim_pairs`].
pub fn decode_claim_pairs(encoded: &str) -> Result<Vec<(String, String)>, ClaimsError> {
    let mut rest = encoded;
    let mut pairs: Vec<(String, String)> = Vec::new();
    while !rest.is_empty() {
        let (key, tail) = take_length_prefixed(rest)?;
        let tail = tail
            .strip_prefix(':')
            .ok_or(ClaimsError::MalformedEncoding)?;
        let (value, tail) = take_length_prefixed(tail)?;
        pairs.push((key, value));
        rest = tail;
    }
    Ok(pairs)
}

/// Issue a token for `claims` through an issuer port.
///
/// The claim set is checked with [`TenantClaims::check_encodable`] BEFORE it is
/// encoded, so a scope a relying party would reject as
/// [`ClaimsError::ScopeMalformed`] cannot be minted into a token and discovered
/// later. The claim set is then encoded canonically; the port decides what, if
/// anything, it does about authenticity.
pub fn issue_tenant_token(
    issuer: &dyn JwtIssuer,
    claims: &TenantClaims,
) -> Result<String, TokenError> {
    claims.check_encodable()?;
    Ok(issuer.issue(&claims.to_claim_pairs())?)
}

/// Decode `token` through a verifier port and validate the resulting claims
/// against `policy` at `now`.
///
/// A success here means the claims are well-formed and in-window. Whether it
/// also means they are authentic depends entirely on `verifier`; the in-tree
/// [`crate::inmemory::UnsignedTokenIssuer`] provides no such guarantee.
pub fn verify_tenant_token(
    verifier: &dyn JwtVerifier,
    token: &str,
    policy: &ClaimsPolicy,
    now: UnixSeconds,
) -> Result<ValidatedClaims, TokenError> {
    let pairs = verifier.verify(token)?;
    let claims = TenantClaims::from_claim_pairs(&pairs)?;
    Ok(policy.validate(&claims, now)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ISSUED_AT: UnixSeconds = 1_000;
    const EXPIRES_AT: UnixSeconds = 1_600;

    fn claims() -> TenantClaims {
        TenantClaims {
            issuer: "oya-tenancy-eu-prod".to_owned(),
            subject: "svc/billing-worker".to_owned(),
            audience: "oyatie-internal".to_owned(),
            tenant: "ten_alpha".to_owned(),
            scopes: vec!["tenancy.read".to_owned(), "tenancy.write".to_owned()],
            issued_at: ISSUED_AT,
            not_before: ISSUED_AT,
            expires_at: EXPIRES_AT,
            key_fingerprint: "fnv1a64:0123456789abcdef".to_owned(),
        }
    }

    fn policy() -> ClaimsPolicy {
        ClaimsPolicy::strict(
            "oya-tenancy-eu-prod",
            "oyatie-internal",
            vec!["tenancy.read".to_owned()],
        )
    }

    #[test]
    fn well_formed_claims_are_accepted_mid_window() {
        let validated = policy()
            .validate(&claims(), 1_200)
            .expect("mid-window claims are acceptable");
        assert_eq!(validated.tenant(), "ten_alpha");
        assert_eq!(validated.subject(), "svc/billing-worker");
        assert_eq!(validated.scopes().len(), 2);
    }

    #[test]
    fn one_second_before_expiry_is_still_valid() {
        assert!(policy().validate(&claims(), EXPIRES_AT - 1).is_ok());
    }

    #[test]
    fn exactly_at_expiry_is_expired() {
        assert_eq!(
            policy().validate(&claims(), EXPIRES_AT),
            Err(ClaimsError::Expired {
                now: EXPIRES_AT,
                expires_at: EXPIRES_AT,
            })
        );
    }

    #[test]
    fn one_second_after_expiry_is_expired() {
        assert_eq!(
            policy().validate(&claims(), EXPIRES_AT + 1),
            Err(ClaimsError::Expired {
                now: EXPIRES_AT + 1,
                expires_at: EXPIRES_AT,
            })
        );
    }

    #[test]
    fn leeway_moves_the_expiry_boundary_by_exactly_that_many_seconds() {
        let lenient = policy().with_leeway_seconds(5);
        assert!(lenient.validate(&claims(), EXPIRES_AT + 4).is_ok());
        assert!(lenient.validate(&claims(), EXPIRES_AT + 5).is_err());
    }

    #[test]
    fn not_before_boundary_is_exact() {
        let mut claims = claims();
        claims.issued_at = 900;
        claims.not_before = 1_100;
        assert_eq!(
            policy().validate(&claims, 1_099),
            Err(ClaimsError::NotYetValid {
                now: 1_099,
                not_before: 1_100,
            })
        );
        assert!(policy().validate(&claims, 1_100).is_ok());
    }

    #[test]
    fn issuer_mismatch_is_rejected() {
        let mut claims = claims();
        claims.issuer = "oya-tenancy-us-prod".to_owned();
        assert_eq!(
            policy().validate(&claims, 1_200),
            Err(ClaimsError::IssuerMismatch {
                expected: "oya-tenancy-eu-prod".to_owned(),
                found: "oya-tenancy-us-prod".to_owned(),
            })
        );
    }

    #[test]
    fn audience_mismatch_is_rejected() {
        let mut claims = claims();
        claims.audience = "someone-elses-api".to_owned();
        assert_eq!(
            policy().validate(&claims, 1_200),
            Err(ClaimsError::AudienceMismatch {
                expected: "oyatie-internal".to_owned(),
                found: "someone-elses-api".to_owned(),
            })
        );
    }

    #[test]
    fn missing_required_scope_is_rejected() {
        let mut claims = claims();
        claims.scopes = vec!["tenancy.write".to_owned()];
        assert_eq!(
            policy().validate(&claims, 1_200),
            Err(ClaimsError::MissingScope {
                scope: "tenancy.read".to_owned(),
            })
        );
    }

    #[test]
    fn absent_tenant_and_malformed_tenant_are_distinguished() {
        let mut absent = claims();
        absent.tenant = String::new();
        assert_eq!(
            policy().validate(&absent, 1_200),
            Err(ClaimsError::TenantMissing)
        );

        let mut malformed = claims();
        malformed.tenant = "alpha".to_owned();
        assert_eq!(
            policy().validate(&malformed, 1_200),
            Err(ClaimsError::TenantMalformed {
                tenant: "alpha".to_owned(),
            })
        );
    }

    /// A literal transcription of the ENTIRE tenant-id rule in
    /// `tenancy/core/domain/src/lib.rs` (`Tenant::new`, the `InvalidTenantId`
    /// guard) as of this commit.
    ///
    /// It is a copy because it has to be: this crate holds no dependency on the
    /// domain crate and cannot gain one (the lockfile is frozen for this wave,
    /// and a path dependency rewrites it too). A copy at least lets the test
    /// assert something about BOTH rules instead of only this module's own — a
    /// test that calls one predicate and is named for two is not a cross-check,
    /// it is a tautology that pins drift rather than catching it.
    fn domain_crate_tenant_id_rule(id: &str) -> bool {
        id.starts_with("ten_") && id.len() > 4
    }

    #[test]
    fn tenant_id_rule_is_deliberately_stricter_than_the_domain_crate() {
        // Ids both rules accept.
        for id in ["ten_alpha", "ten_a-1_b", "ten_a"] {
            assert!(tenant_id_is_well_formed(id), "{id} must be accepted here");
            assert!(domain_crate_tenant_id_rule(id), "{id} is legal in domain");
        }

        // Ids both rules reject.
        for id in ["ten_", "tenant_alpha", "ten"] {
            assert!(!tenant_id_is_well_formed(id), "{id} must be refused here");
            assert!(
                !domain_crate_tenant_id_rule(id),
                "{id} is illegal in domain"
            );
        }

        // THE DIVERGENCE, enumerated rather than glossed over: every id below is
        // one `tenancy/core/domain::Tenant::new` will mint and this crate will
        // refuse, which is a total authorization outage for that tenant. Named
        // in the crate Gaps paragraph; the fix belongs in the domain crate.
        let over_long = format!("ten_{}", "a".repeat(MAX_TENANT_LEN - 3));
        let divergent = [
            "ten_ACME",
            "ten_acme.eu",
            "ten_a b",
            "ten_alpha; drop",
            over_long.as_str(),
        ];
        for id in divergent {
            assert!(
                domain_crate_tenant_id_rule(id),
                "{id} must be legal in the domain crate for this to be a divergence"
            );
            assert!(
                !tenant_id_is_well_formed(id),
                "{id} must be refused here — that is the documented narrowing"
            );
        }
        assert!(over_long.len() > MAX_TENANT_LEN, "the long fixture is long");
    }

    #[test]
    fn the_tenant_rejection_message_names_the_rule_not_just_the_token() {
        let rendered = ClaimsError::TenantMalformed {
            tenant: "ten_ACME".to_owned(),
        }
        .to_string();
        assert!(rendered.contains("ten_ACME"), "{rendered}");
        assert!(
            rendered.contains("STRICTER"),
            "an operator must be told the id policy is narrower than the tenant \
             store's, not merely that the token is bad: {rendered}"
        );
    }

    #[test]
    fn a_whitespace_scope_cannot_be_laundered_into_two_scopes() {
        // The privilege-GRANTING round trip: `["tenancy.read admin"]` must never
        // come back as `["tenancy.read", "admin"]`, or passing a claim set
        // through this crate's own encoder mints authority it never held.
        let mut original = claims();
        original.scopes = vec!["tenancy.read admin".to_owned()];

        let pairs = original.to_claim_pairs();
        let encoded = encode_claim_pairs(&pairs);
        let decoded = decode_claim_pairs(&encoded).expect("decodes");
        let rebuilt = TenantClaims::from_claim_pairs(&decoded).expect("rebuilds");
        assert_eq!(
            rebuilt.scopes,
            vec!["tenancy.read admin".to_owned()],
            "the wire form must preserve the scope count exactly"
        );
        assert_eq!(rebuilt, original);

        // And the rebuilt claims are refused for the same reason the originals
        // were, rather than quietly satisfying a requirement for `admin`.
        let demanding = ClaimsPolicy::strict(
            "oya-tenancy-eu-prod",
            "oyatie-internal",
            vec!["admin".to_owned()],
        );
        assert_eq!(
            demanding.validate(&rebuilt, 1_200),
            Err(ClaimsError::ScopeMalformed {
                scope: "tenancy.read admin".to_owned(),
            })
        );
        assert_eq!(
            demanding.validate(&original, 1_200),
            demanding.validate(&rebuilt, 1_200),
            "the direct path and the wire path must agree"
        );
    }

    #[test]
    fn a_malformed_scope_is_refused_at_issuance_before_it_becomes_a_token() {
        let mut original = claims();
        original.scopes = vec!["tenancy.read admin".to_owned()];
        assert_eq!(
            original.check_encodable(),
            Err(ClaimsError::ScopeMalformed {
                scope: "tenancy.read admin".to_owned(),
            })
        );
        assert_eq!(original.malformed_scopes(), vec!["tenancy.read admin"]);

        let mut empty_scope = claims();
        empty_scope.scopes = vec![String::new()];
        assert_eq!(
            empty_scope.check_encodable(),
            Err(ClaimsError::ScopeMalformed {
                scope: String::new(),
            })
        );

        assert_eq!(claims().check_encodable(), Ok(()));
        assert!(claims().malformed_scopes().is_empty());
    }

    #[test]
    fn scope_lists_round_trip_including_the_empty_and_duplicated_cases() {
        for scopes in [
            vec![],
            vec!["a".to_owned()],
            vec!["a".to_owned(), "a".to_owned()],
            vec!["a:b:c".to_owned(), "3:x".to_owned(), "".to_owned()],
            vec!["with space".to_owned(), "with\ttab".to_owned()],
        ] {
            let encoded = encode_scope_list(&scopes);
            assert_eq!(
                decode_scope_list(&encoded).expect("scope list decodes"),
                scopes,
                "scope list {scopes:?} must survive its own encoding"
            );
        }
    }

    #[test]
    fn a_truncated_scope_list_is_refused_rather_than_silently_shortened() {
        let encoded = encode_scope_list(&["tenancy.read".to_owned()]);
        assert_eq!(
            decode_scope_list(&encoded[..encoded.len() - 2]),
            Err(ClaimsError::MalformedEncoding)
        );
    }

    #[test]
    fn a_duplicate_claim_key_is_refused_rather_than_resolved_last_wins() {
        let mut pairs = claims().to_claim_pairs();
        pairs.push((CLAIM_TENANT.to_owned(), "ten_victim".to_owned()));

        // The codec is faithful: it carries both, which is exactly why the
        // claims layer has to be the one that refuses.
        let encoded = encode_claim_pairs(&pairs);
        let decoded = decode_claim_pairs(&encoded).expect("both pairs decode");
        assert_eq!(decoded.len(), pairs.len());
        assert_eq!(
            decoded
                .iter()
                .filter(|(key, _)| key == CLAIM_TENANT)
                .count(),
            2
        );

        assert_eq!(
            TenantClaims::from_claim_pairs(&decoded),
            Err(ClaimsError::DuplicateClaim {
                name: CLAIM_TENANT.to_owned(),
            }),
            "last-occurrence-wins would hand back a DIFFERENT tenant than a \
             reader of the first occurrence saw"
        );
    }

    #[test]
    fn a_duplicated_non_identity_claim_is_refused_too() {
        let mut pairs = claims().to_claim_pairs();
        pairs.push((CLAIM_EXPIRES_AT.to_owned(), "99999".to_owned()));
        assert_eq!(
            TenantClaims::from_claim_pairs(&pairs),
            Err(ClaimsError::DuplicateClaim {
                name: CLAIM_EXPIRES_AT.to_owned(),
            })
        );
    }

    #[test]
    fn only_one_spelling_of_each_length_prefix_is_accepted() {
        let canonical = encode_claim_pairs(&claims().to_claim_pairs());
        assert!(decode_claim_pairs(&canonical).is_ok());

        // Zero-padded and sign-prefixed lengths parse as the same number under
        // `str::parse`, which would give one claim set several byte-different
        // canonical encodings.
        for non_canonical in [format!("0{canonical}"), format!("+{canonical}")] {
            assert_eq!(
                decode_claim_pairs(&non_canonical),
                Err(ClaimsError::MalformedEncoding),
                "{non_canonical:?} must not be a second spelling of a canonical token"
            );
        }
        assert_eq!(
            decode_claim_pairs("3a:iss:1:a"),
            Err(ClaimsError::MalformedEncoding)
        );
        assert_eq!(
            decode_claim_pairs("-1:iss"),
            Err(ClaimsError::MalformedEncoding)
        );
        // A genuinely zero-length value is still spelled "0".
        assert_eq!(
            decode_claim_pairs("3:iss:0:"),
            Ok(vec![("iss".to_owned(), String::new())])
        );
    }

    #[test]
    fn inverted_window_is_rejected_before_the_clock_is_consulted() {
        let mut claims = claims();
        claims.expires_at = claims.issued_at;
        assert_eq!(
            policy().validate(&claims, 1_200),
            Err(ClaimsError::InvalidWindow {
                issued_at: ISSUED_AT,
                expires_at: ISSUED_AT,
            })
        );
    }

    #[test]
    fn over_long_lifetime_is_rejected() {
        let mut claims = claims();
        claims.expires_at = claims.issued_at + 7_200;
        assert_eq!(
            policy().validate(&claims, 1_200),
            Err(ClaimsError::LifetimeTooLong {
                lifetime_seconds: 7_200,
                max_lifetime_seconds: 3_600,
            })
        );
    }

    #[test]
    fn token_issued_in_the_future_is_rejected() {
        assert_eq!(
            policy().validate(&claims(), ISSUED_AT - 1),
            Err(ClaimsError::IssuedInFuture {
                now: ISSUED_AT - 1,
                issued_at: ISSUED_AT,
            })
        );
    }

    #[test]
    fn malformed_scope_is_rejected() {
        let mut claims = claims();
        claims.scopes = vec!["tenancy read".to_owned()];
        assert_eq!(
            policy().validate(&claims, 1_200),
            Err(ClaimsError::ScopeMalformed {
                scope: "tenancy read".to_owned(),
            })
        );
    }

    #[test]
    fn empty_key_fingerprint_is_rejected() {
        let mut claims = claims();
        claims.key_fingerprint = String::new();
        assert_eq!(
            policy().validate(&claims, 1_200),
            Err(ClaimsError::KeyFingerprintMissing)
        );
    }

    #[test]
    fn claim_pairs_round_trip_through_the_canonical_encoding() {
        let original = claims();
        let encoded = encode_claim_pairs(&original.to_claim_pairs());
        let decoded = decode_claim_pairs(&encoded).expect("canonical encoding decodes");
        assert_eq!(
            TenantClaims::from_claim_pairs(&decoded).expect("pairs rebuild claims"),
            original
        );
    }

    #[test]
    fn encoding_is_deterministic_and_order_independent() {
        let mut pairs = claims().to_claim_pairs();
        let forward = encode_claim_pairs(&pairs);
        pairs.reverse();
        assert_eq!(encode_claim_pairs(&pairs), forward);
    }

    #[test]
    fn encoding_survives_delimiter_characters_in_values() {
        let mut original = claims();
        original.subject = "svc:billing.worker:1:2:3".to_owned();
        let encoded = encode_claim_pairs(&original.to_claim_pairs());
        let decoded =
            decode_claim_pairs(&encoded).expect("delimiters are length-prefixed, not escaped");
        assert_eq!(
            TenantClaims::from_claim_pairs(&decoded).expect("pairs rebuild claims"),
            original
        );
    }

    #[test]
    fn truncated_encoding_is_refused_rather_than_panicking() {
        let encoded = encode_claim_pairs(&claims().to_claim_pairs());
        let truncated = &encoded[..encoded.len() - 3];
        assert_eq!(
            decode_claim_pairs(truncated),
            Err(ClaimsError::MalformedEncoding)
        );
    }

    #[test]
    fn absent_and_non_numeric_claims_are_named_in_the_error() {
        let mut pairs = claims().to_claim_pairs();
        pairs.retain(|(key, _)| key != CLAIM_EXPIRES_AT);
        assert_eq!(
            TenantClaims::from_claim_pairs(&pairs),
            Err(ClaimsError::MissingClaim {
                name: CLAIM_EXPIRES_AT.to_owned(),
            })
        );

        let mut pairs = claims().to_claim_pairs();
        for pair in &mut pairs {
            if pair.0 == CLAIM_ISSUED_AT {
                pair.1 = "soon".to_owned();
            }
        }
        assert_eq!(
            TenantClaims::from_claim_pairs(&pairs),
            Err(ClaimsError::NonNumericClaim {
                name: CLAIM_ISSUED_AT.to_owned(),
            })
        );
    }
}
