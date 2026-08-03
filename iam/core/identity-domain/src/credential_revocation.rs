// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

//! Credential and token revocation state machine.
//!
//! This mod is the `credential_revocation` flat extension of the
//! `oya-identity-domain` crate (additive, ADR-0509 flat clean-arch). It
//! composes the existing TTL-expiry invariant (`MAX_TOKEN_TTL_SECONDS`,
//! `expires_at_epoch_seconds`) with explicit, reason-tagged revocation into a
//! single `now_epoch_seconds`-driven verdict that obeys **deny-precedence**:
//!
//! ```text
//! is_valid(now) == (NOT revoked) AND (NOT expired_at(now))
//! ```
//!
//! Precedence order (deny-overrides, per RFC 7009 token revocation + Cedar /
//! NIST deny semantics):
//!
//! 1. If the fingerprint is in the tenant's [`RevocationLedger`] →
//!    [`CredentialStatus::Revoked`] (wins even if `now < expires_at`).
//! 2. Else if `now >= expires_at_epoch_seconds` → [`CredentialStatus::Expired`].
//! 3. Else → [`CredentialStatus::Active`].
//!
//! **No new runtime dependency.** All time is caller-supplied
//! `now_epoch_seconds`; there is no `SystemTime::now()` call here.
//!
//! The two existing credential carriers (`Token`, `StsCredential`) are used
//! read-only; no existing `lib.rs` logic is changed beyond the additive
//! re-export block.

use std::collections::BTreeMap;
use std::fmt;

use crate::{StsCredential, Token};

// ────────────────────────────────────────────────────────────────────────────
// RevocationReason
// ────────────────────────────────────────────────────────────────────────────

/// The reason a credential was revoked (closed enum, wire round-trip).
///
/// Wire strings are the contract surface for a future AsyncAPI 3.1.0
/// `credential.revoked` event. Use [`RevocationReason::from_wire`] /
/// [`RevocationReason::as_str`] for serialisation round-trips.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum RevocationReason {
    /// Credential material believed leaked or stolen.
    Compromised,
    /// Replaced by a newer credential (key rotation / re-issue).
    Superseded,
    /// The owning user or service principal has been deprovisioned.
    PrincipalDeprovisioned,
    /// Revoked by a governance or compliance action.
    PolicyViolation,
    /// Operator-initiated revocation that does not fit another category.
    AdministrativeRevoke,
}

/// Error returned when a raw wire string does not match any
/// [`RevocationReason`] variant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnknownRevocationReason(pub String);

impl fmt::Display for UnknownRevocationReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown revocation reason: {:?}; expected one of \
             compromised|superseded|principal_deprovisioned|\
             policy_violation|administrative_revoke",
            self.0
        )
    }
}

impl std::error::Error for UnknownRevocationReason {}

impl RevocationReason {
    /// Returns the canonical wire string for this reason.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Compromised => "compromised",
            Self::Superseded => "superseded",
            Self::PrincipalDeprovisioned => "principal_deprovisioned",
            Self::PolicyViolation => "policy_violation",
            Self::AdministrativeRevoke => "administrative_revoke",
        }
    }

    /// Parse from the wire string.
    pub fn from_wire(s: &str) -> Result<Self, UnknownRevocationReason> {
        match s {
            "compromised" => Ok(Self::Compromised),
            "superseded" => Ok(Self::Superseded),
            "principal_deprovisioned" => Ok(Self::PrincipalDeprovisioned),
            "policy_violation" => Ok(Self::PolicyViolation),
            "administrative_revoke" => Ok(Self::AdministrativeRevoke),
            other => Err(UnknownRevocationReason(other.to_string())),
        }
    }

    /// All 5 variants in declaration order.
    pub const ALL: [Self; 5] = [
        Self::Compromised,
        Self::Superseded,
        Self::PrincipalDeprovisioned,
        Self::PolicyViolation,
        Self::AdministrativeRevoke,
    ];
}

impl fmt::Display for RevocationReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// CredentialStatus
// ────────────────────────────────────────────────────────────────────────────

/// Terminal verdict for a credential evaluation.
///
/// Deny-precedence order: `Revoked` > `Expired` > `Active`. Once a credential
/// is not `Active`, it can never become `Active` again as the clock advances
/// (`is_valid` is monotonically non-increasing in time).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialStatus {
    /// Credential is within its TTL and has not been explicitly revoked.
    Active,
    /// Credential has passed its `expires_at_epoch_seconds` boundary and is
    /// not in the revocation ledger (clock-only expiry).
    Expired,
    /// Credential was explicitly revoked (wins over `Expired` per
    /// deny-precedence).
    Revoked(RevocationReason),
}

impl CredentialStatus {
    /// Returns `true` iff this status is [`CredentialStatus::Active`].
    ///
    /// This is the canonical `is_valid` gate. It is `false` for both
    /// `Expired` and `Revoked(_)`.
    pub const fn is_valid(self) -> bool {
        matches!(self, Self::Active)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// RevocationError
// ────────────────────────────────────────────────────────────────────────────

/// Errors produced by [`RevocationLedger`] operations and evaluation calls.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RevocationError {
    /// `RevocationLedger::new` was given a tenant id that does not start with
    /// the `ten_` prefix (matches `lib.rs` validation convention).
    InvalidTenantId,
    /// `revoke` was called with an empty or whitespace-only fingerprint.
    EmptyFingerprint,
    /// `revoke` was called for an already-revoked fingerprint with a
    /// **different** reason (first-writer-wins; original reason is preserved).
    ConflictingRevocation,
    /// The credential's tenant id does not match the ledger's tenant id.
    /// Fail-closed: never silently treat a cross-tenant credential as `Active`.
    TenantMismatch,
}

impl fmt::Display for RevocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTenantId => {
                f.write_str("invalid tenant id: must start with 'ten_' and have a non-empty suffix")
            }
            Self::EmptyFingerprint => {
                f.write_str("empty fingerprint: fingerprint must be non-empty and non-whitespace")
            }
            Self::ConflictingRevocation => f.write_str(
                "conflicting revocation: fingerprint already revoked with a different reason \
                 (first-writer-wins)",
            ),
            Self::TenantMismatch => f.write_str(
                "tenant mismatch: credential tenant does not match ledger tenant (fail-closed)",
            ),
        }
    }
}

impl std::error::Error for RevocationError {}

// ────────────────────────────────────────────────────────────────────────────
// RevocationLedger
// ────────────────────────────────────────────────────────────────────────────

/// Tenant-scoped, append-only revocation ledger.
///
/// Stores a mapping from credential fingerprint strings to the reason they were
/// revoked. Append-only with **first-writer-wins** semantics: re-revoking an
/// already-revoked fingerprint with the same reason is a no-op (`Ok`); with a
/// different reason it returns [`RevocationError::ConflictingRevocation`] and
/// the original reason is preserved.
///
/// The ledger is a pure in-memory value type — no I/O, no persistence. Use
/// [`RevocationLedger::evaluate_token`] / [`RevocationLedger::evaluate_sts`]
/// to obtain a [`CredentialStatus`] that composes TTL-expiry with revocation
/// under deny-precedence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevocationLedger {
    tenant_id: String,
    /// fingerprint -> reason; append-only, first-writer-wins.
    revoked: BTreeMap<String, RevocationReason>,
}

impl RevocationLedger {
    /// Create a new empty ledger for the given tenant.
    ///
    /// `tenant_id` must start with `ten_` and have a non-empty suffix (mirrors
    /// `lib.rs`'s `validate_tenant_id` convention; the guard is local to keep
    /// the mod self-contained).
    pub fn new(tenant_id: impl Into<String>) -> Result<Self, RevocationError> {
        let tenant_id = tenant_id.into();
        if !tenant_id.starts_with("ten_") || tenant_id.len() <= 4 {
            return Err(RevocationError::InvalidTenantId);
        }
        Ok(Self {
            tenant_id,
            revoked: BTreeMap::new(),
        })
    }

    /// Returns the tenant id this ledger is scoped to.
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    /// Record that `fingerprint` has been revoked for `reason`.
    ///
    /// - If `fingerprint` is empty or whitespace-only: `Err(EmptyFingerprint)`.
    /// - If `fingerprint` is already revoked with the **same** reason: `Ok(())`
    ///   (idempotent; `len()` is unchanged).
    /// - If `fingerprint` is already revoked with a **different** reason:
    ///   `Err(ConflictingRevocation)` (first-writer-wins; original reason
    ///   preserved).
    /// - Otherwise: inserts the entry and returns `Ok(())`.
    pub fn revoke(
        &mut self,
        fingerprint: impl Into<String>,
        reason: RevocationReason,
    ) -> Result<(), RevocationError> {
        let fingerprint = fingerprint.into();
        if fingerprint.trim().is_empty() {
            return Err(RevocationError::EmptyFingerprint);
        }
        if let Some(&existing) = self.revoked.get(&fingerprint) {
            if existing == reason {
                return Ok(());
            }
            return Err(RevocationError::ConflictingRevocation);
        }
        self.revoked.insert(fingerprint, reason);
        Ok(())
    }

    /// Returns the revocation reason for `fingerprint`, if present.
    pub fn reason_for(&self, fingerprint: &str) -> Option<RevocationReason> {
        self.revoked.get(fingerprint).copied()
    }

    /// Returns `true` iff `fingerprint` is in the revocation ledger.
    pub fn is_revoked(&self, fingerprint: &str) -> bool {
        self.revoked.contains_key(fingerprint)
    }

    /// Returns the number of revoked fingerprints in the ledger.
    pub fn len(&self) -> usize {
        self.revoked.len()
    }

    /// Returns `true` iff no credentials have been revoked yet.
    pub fn is_empty(&self) -> bool {
        self.revoked.is_empty()
    }

    /// Evaluate a plain [`Token`] against this ledger at `now_epoch_seconds`.
    ///
    /// Tenant-scope guard: if `token.tenant_id != self.tenant_id`, returns
    /// `Err(TenantMismatch)` (fail-closed — never silently returns `Active`
    /// for a cross-tenant credential).
    ///
    /// Verdict (deny-precedence):
    /// 1. Fingerprint in ledger → `Revoked(reason)`.
    /// 2. `now_epoch_seconds >= token.expires_at_epoch_seconds` → `Expired`.
    /// 3. Otherwise → `Active`.
    pub fn evaluate_token(
        &self,
        token: &Token,
        now_epoch_seconds: u64,
    ) -> Result<CredentialStatus, RevocationError> {
        if token.tenant_id != self.tenant_id {
            return Err(RevocationError::TenantMismatch);
        }
        Ok(self.verdict(&token_fingerprint(token), token.expires_at_epoch_seconds, now_epoch_seconds))
    }

    /// Evaluate an [`StsCredential`] against this ledger at `now_epoch_seconds`.
    ///
    /// Uses the credential's existing `token_fingerprint.value` as the lookup
    /// key (the `sts1:` fingerprint already computed at issue time).
    ///
    /// Tenant-scope guard: extracts the tenant id from the credential's
    /// `principal` and returns `Err(TenantMismatch)` if it does not match
    /// `self.tenant_id`.
    ///
    /// Verdict (deny-precedence):
    /// 1. Fingerprint in ledger → `Revoked(reason)`.
    /// 2. `now_epoch_seconds >= credential.expires_at_epoch_seconds.value` → `Expired`.
    /// 3. Otherwise → `Active`.
    pub fn evaluate_sts(
        &self,
        credential: &StsCredential,
        now_epoch_seconds: u64,
    ) -> Result<CredentialStatus, RevocationError> {
        if sts_tenant_id(credential) != self.tenant_id {
            return Err(RevocationError::TenantMismatch);
        }
        Ok(self.verdict(
            &credential.token_fingerprint.value,
            credential.expires_at_epoch_seconds.value,
            now_epoch_seconds,
        ))
    }

    /// Compose explicit revocation with TTL-expiry under deny-precedence for a
    /// fingerprint whose tenant scope has already been validated by the caller.
    ///
    /// Order (deny-overrides): revoked wins over expired, expired wins over
    /// active. This is the single source of the deny-precedence ordering shared
    /// by [`Self::evaluate_token`] and [`Self::evaluate_sts`].
    fn verdict(&self, fingerprint: &str, expires_at: u64, now_epoch_seconds: u64) -> CredentialStatus {
        if let Some(reason) = self.reason_for(fingerprint) {
            return CredentialStatus::Revoked(reason);
        }
        if now_epoch_seconds >= expires_at {
            return CredentialStatus::Expired;
        }
        CredentialStatus::Active
    }
}

// ────────────────────────────────────────────────────────────────────────────
// token_fingerprint — public, deterministic, tok1: prefix
// ────────────────────────────────────────────────────────────────────────────

/// Compute a deterministic fingerprint for a plain [`Token`].
///
/// Uses the same FNV-1a construction as the private `credential_fingerprint`
/// in `lib.rs`, but with a distinct `tok1:` prefix (vs `sts1:` for
/// [`StsCredential`]). The output is stable for the same `Token` value and
/// suitable as a [`RevocationLedger`] key.
pub fn token_fingerprint(token: &Token) -> String {
    let mut state: u64 = 0xcbf29ce484222325;
    fn feed(state: &mut u64, bytes: &[u8]) {
        for byte in bytes {
            *state ^= u64::from(*byte);
            *state = state.wrapping_mul(0x100000001b3);
        }
    }
    feed(&mut state, token.tenant_id.as_bytes());
    feed(&mut state, token.user_id.as_bytes());
    feed(&mut state, token.purpose.pascal_label().as_bytes());
    feed(&mut state, token.issued_at_epoch_seconds.to_string().as_bytes());
    feed(&mut state, token.expires_at_epoch_seconds.to_string().as_bytes());
    format!("tok1:{state:016x}")
}

// ────────────────────────────────────────────────────────────────────────────
// Private helpers
// ────────────────────────────────────────────────────────────────────────────

/// Extract the tenant id string from an `StsCredential`'s principal.
fn sts_tenant_id(credential: &StsCredential) -> &str {
    use crate::Principal;
    match &credential.principal {
        Principal::Human { tenant_id, .. } | Principal::ServicePrincipal { tenant_id, .. } => {
            &tenant_id.value
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Inline tests (Tier-3 exemption — ADR-0083)
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CredentialRequest, CredentialRequestKind, Principal, issue_credential, issue_token,
    };
    use data_boundary_kernel::Purpose;

    // ── RevocationReason ────────────────────────────────────────────────────

    #[test]
    fn revocation_reason_all_has_five_distinct_wire_strings() {
        assert_eq!(RevocationReason::ALL.len(), 5);
        let mut seen = std::collections::HashSet::new();
        for variant in RevocationReason::ALL {
            assert!(
                seen.insert(variant.as_str()),
                "duplicate wire string: {}",
                variant.as_str()
            );
        }
    }

    #[test]
    fn revocation_reason_round_trips_every_variant() {
        for variant in RevocationReason::ALL {
            let wire = variant.as_str();
            let parsed =
                RevocationReason::from_wire(wire).expect("all wire strings must round-trip");
            assert_eq!(parsed, variant, "round-trip failed for {wire}");
        }
    }

    #[test]
    fn revocation_reason_from_wire_rejects_unknown_value() {
        let err =
            RevocationReason::from_wire("nope").expect_err("unknown string must be rejected");
        assert_eq!(err.0, "nope");
        let msg = err.to_string();
        assert!(msg.contains("unknown revocation reason"));
        assert!(msg.contains("nope"));
        assert!(msg.contains("compromised"));
    }

    #[test]
    fn revocation_reason_from_wire_rejects_empty_string() {
        let err = RevocationReason::from_wire("").expect_err("empty string must be rejected");
        assert_eq!(err.0, "");
        let msg = err.to_string();
        assert!(msg.contains("unknown revocation reason"));
        assert!(msg.contains("compromised"));
    }

    #[test]
    fn revocation_reason_display_renders_wire_string() {
        assert_eq!(RevocationReason::Compromised.to_string(), "compromised");
        assert_eq!(
            RevocationReason::PrincipalDeprovisioned.to_string(),
            "principal_deprovisioned"
        );
        assert_eq!(
            RevocationReason::AdministrativeRevoke.to_string(),
            "administrative_revoke"
        );
    }

    #[test]
    fn unknown_revocation_reason_display_covers_all_variants_and_bad_value() {
        let msg = UnknownRevocationReason("xyz".to_string()).to_string();
        assert!(msg.contains("xyz"));
        assert!(msg.contains("compromised"));
        assert!(msg.contains("superseded"));
        assert!(msg.contains("principal_deprovisioned"));
        assert!(msg.contains("policy_violation"));
        assert!(msg.contains("administrative_revoke"));
    }

    // ── CredentialStatus (edge case 11) ─────────────────────────────────────

    #[test]
    fn credential_status_is_valid_true_only_for_active() {
        assert!(CredentialStatus::Active.is_valid());
        assert!(!CredentialStatus::Expired.is_valid());
        assert!(!CredentialStatus::Revoked(RevocationReason::Compromised).is_valid());
        assert!(!CredentialStatus::Revoked(RevocationReason::Superseded).is_valid());
        assert!(!CredentialStatus::Revoked(RevocationReason::PrincipalDeprovisioned).is_valid());
        assert!(!CredentialStatus::Revoked(RevocationReason::PolicyViolation).is_valid());
        assert!(!CredentialStatus::Revoked(RevocationReason::AdministrativeRevoke).is_valid());
    }

    // ── RevocationError Display ─────────────────────────────────────────────

    #[test]
    fn revocation_error_display_is_non_empty_and_distinct() {
        let msgs = [
            RevocationError::InvalidTenantId.to_string(),
            RevocationError::EmptyFingerprint.to_string(),
            RevocationError::ConflictingRevocation.to_string(),
            RevocationError::TenantMismatch.to_string(),
        ];
        for msg in &msgs {
            assert!(!msg.is_empty(), "Display must not be empty");
        }
        // All distinct
        let unique: std::collections::HashSet<_> = msgs.iter().collect();
        assert_eq!(unique.len(), 4, "all Display messages must be distinct");
    }

    // ── RevocationLedger construction ───────────────────────────────────────

    #[test]
    fn revocation_ledger_new_accepts_valid_tenant_id() {
        let ledger = RevocationLedger::new("ten_alpha").expect("valid tenant id");
        assert_eq!(ledger.tenant_id(), "ten_alpha");
        assert!(ledger.is_empty());
        assert_eq!(ledger.len(), 0);
    }

    #[test]
    fn revocation_ledger_new_rejects_invalid_tenant_id() {
        assert_eq!(
            RevocationLedger::new("nope"),
            Err(RevocationError::InvalidTenantId)
        );
        assert_eq!(
            RevocationLedger::new("ten_"),
            Err(RevocationError::InvalidTenantId)
        );
        assert_eq!(
            RevocationLedger::new(""),
            Err(RevocationError::InvalidTenantId)
        );
    }

    // ── RevocationLedger revoke + query (edge cases 6, 7, 8) ───────────────

    #[test]
    fn revoke_then_is_revoked_and_reason_for_reflect_it() {
        let mut ledger = RevocationLedger::new("ten_alpha").unwrap();
        ledger
            .revoke("fp_abc", RevocationReason::Compromised)
            .expect("first revoke must succeed");
        assert!(ledger.is_revoked("fp_abc"));
        assert_eq!(
            ledger.reason_for("fp_abc"),
            Some(RevocationReason::Compromised)
        );
        assert_eq!(ledger.len(), 1);
        assert!(!ledger.is_empty());
    }

    #[test]
    fn revoke_same_reason_is_idempotent_and_len_unchanged() {
        // edge case 6
        let mut ledger = RevocationLedger::new("ten_alpha").unwrap();
        ledger
            .revoke("fp_abc", RevocationReason::Superseded)
            .expect("first revoke");
        ledger
            .revoke("fp_abc", RevocationReason::Superseded)
            .expect("same-reason re-revoke must be idempotent");
        assert_eq!(ledger.len(), 1, "len must not change on idempotent revoke");
    }

    #[test]
    fn revoke_conflicting_reason_returns_error_and_preserves_original() {
        // edge case 7
        let mut ledger = RevocationLedger::new("ten_alpha").unwrap();
        ledger
            .revoke("fp_abc", RevocationReason::Compromised)
            .expect("first revoke");
        let err = ledger
            .revoke("fp_abc", RevocationReason::PolicyViolation)
            .expect_err("conflicting reason must be rejected");
        assert_eq!(err, RevocationError::ConflictingRevocation);
        // original reason must be preserved
        assert_eq!(
            ledger.reason_for("fp_abc"),
            Some(RevocationReason::Compromised)
        );
    }

    #[test]
    fn revoke_empty_fingerprint_returns_empty_fingerprint_error() {
        // edge case 8
        let mut ledger = RevocationLedger::new("ten_alpha").unwrap();
        assert_eq!(
            ledger.revoke("", RevocationReason::Compromised),
            Err(RevocationError::EmptyFingerprint)
        );
        assert_eq!(
            ledger.revoke("   ", RevocationReason::Compromised),
            Err(RevocationError::EmptyFingerprint)
        );
        assert!(ledger.is_empty(), "ledger must remain empty after failed revokes");
    }

    // ── token_fingerprint (edge case 10) ───────────────────────────────────

    #[test]
    fn token_fingerprint_is_deterministic_and_has_tok1_prefix() {
        // edge case 10
        let token = issue_token(
            "ten_alpha".into(),
            "usr_admin".into(),
            Purpose::CapabilityInvocation,
            900,
            1_000,
        )
        .expect("valid token");
        let fp1 = token_fingerprint(&token);
        let fp2 = token_fingerprint(&token);
        assert_eq!(fp1, fp2, "fingerprint must be deterministic");
        assert!(fp1.starts_with("tok1:"), "must have tok1: prefix, got {fp1}");
    }

    #[test]
    fn token_fingerprint_differs_from_sts1_prefix() {
        // edge case 10 — distinct prefix
        let token = issue_token(
            "ten_alpha".into(),
            "usr_admin".into(),
            Purpose::CapabilityInvocation,
            900,
            1_000,
        )
        .expect("valid token");
        let fp = token_fingerprint(&token);
        assert!(!fp.starts_with("sts1:"), "tok1: must be distinct from sts1:");
    }

    // ── evaluate_token — full deny-precedence (edge cases 1–5) ─────────────

    fn make_token(tenant: &str, issued: u64, ttl: u64) -> Token {
        issue_token(
            tenant.into(),
            "usr_admin".into(),
            Purpose::CapabilityInvocation,
            ttl,
            issued,
        )
        .expect("valid token")
    }

    #[test]
    fn evaluate_token_now_at_boundary_is_expired() {
        // edge case 1: now == expires_at -> Expired
        let token = make_token("ten_alpha", 1_000, 900);
        let expires_at = token.expires_at_epoch_seconds;
        let ledger = RevocationLedger::new("ten_alpha").unwrap();
        assert_eq!(
            ledger.evaluate_token(&token, expires_at).unwrap(),
            CredentialStatus::Expired
        );
    }

    #[test]
    fn evaluate_token_one_second_before_expiry_is_active() {
        // edge case 2: now == expires_at - 1 (un-revoked) -> Active
        let token = make_token("ten_alpha", 1_000, 900);
        let ledger = RevocationLedger::new("ten_alpha").unwrap();
        assert_eq!(
            ledger
                .evaluate_token(&token, token.expires_at_epoch_seconds - 1)
                .unwrap(),
            CredentialStatus::Active
        );
    }

    #[test]
    fn evaluate_token_revoked_before_expiry_is_revoked() {
        // edge case 3: revoked + now < expires_at -> Revoked (deny-precedence)
        let token = make_token("ten_alpha", 1_000, 900);
        let fp = token_fingerprint(&token);
        let mut ledger = RevocationLedger::new("ten_alpha").unwrap();
        ledger.revoke(fp, RevocationReason::Compromised).unwrap();
        let status = ledger
            .evaluate_token(&token, token.expires_at_epoch_seconds - 1)
            .unwrap();
        assert_eq!(status, CredentialStatus::Revoked(RevocationReason::Compromised));
        assert!(!status.is_valid());
    }

    #[test]
    fn evaluate_token_revoked_and_expired_is_revoked_not_expired() {
        // edge case 4: revoked + now >= expires_at -> Revoked (revocation outranks expiry)
        let token = make_token("ten_alpha", 1_000, 900);
        let fp = token_fingerprint(&token);
        let mut ledger = RevocationLedger::new("ten_alpha").unwrap();
        ledger.revoke(fp, RevocationReason::PolicyViolation).unwrap();
        let status = ledger
            .evaluate_token(&token, token.expires_at_epoch_seconds + 100)
            .unwrap();
        assert_eq!(
            status,
            CredentialStatus::Revoked(RevocationReason::PolicyViolation)
        );
    }

    #[test]
    fn evaluate_token_cross_tenant_is_tenant_mismatch() {
        // edge case 5: cross-tenant -> TenantMismatch (fail-closed)
        let token = make_token("ten_alpha", 1_000, 900);
        let ledger = RevocationLedger::new("ten_beta").unwrap();
        assert_eq!(
            ledger.evaluate_token(&token, 1_500),
            Err(RevocationError::TenantMismatch)
        );
    }

    // ── evaluate_sts ────────────────────────────────────────────────────────

    fn make_sts(tenant: &str, issued: u64, ttl: u64) -> StsCredential {
        let principal = Principal::human(tenant.into(), "usr_admin".into()).unwrap();
        issue_credential(CredentialRequest {
            principal,
            kind: CredentialRequestKind::Sts,
            purpose: Purpose::CapabilityInvocation,
            scopes: vec!["cloud.iam.read".into()],
            ttl_seconds: ttl,
            issued_at_epoch_seconds: issued,
        })
        .expect("valid credential")
    }

    #[test]
    fn evaluate_sts_active_when_not_revoked_and_within_ttl() {
        let cred = make_sts("ten_alpha", 1_000, 900);
        let ledger = RevocationLedger::new("ten_alpha").unwrap();
        assert_eq!(
            ledger
                .evaluate_sts(&cred, cred.expires_at_epoch_seconds.value - 1)
                .unwrap(),
            CredentialStatus::Active
        );
    }

    #[test]
    fn evaluate_sts_expired_at_boundary() {
        let cred = make_sts("ten_alpha", 1_000, 900);
        let ledger = RevocationLedger::new("ten_alpha").unwrap();
        assert_eq!(
            ledger
                .evaluate_sts(&cred, cred.expires_at_epoch_seconds.value)
                .unwrap(),
            CredentialStatus::Expired
        );
    }

    #[test]
    fn evaluate_sts_revoked_wins_over_active() {
        let cred = make_sts("ten_alpha", 1_000, 900);
        let fp = cred.token_fingerprint.value.clone();
        let mut ledger = RevocationLedger::new("ten_alpha").unwrap();
        ledger.revoke(fp, RevocationReason::Superseded).unwrap();
        let status = ledger
            .evaluate_sts(&cred, cred.expires_at_epoch_seconds.value - 1)
            .unwrap();
        assert_eq!(status, CredentialStatus::Revoked(RevocationReason::Superseded));
        assert!(!status.is_valid());
    }

    #[test]
    fn evaluate_sts_cross_tenant_is_tenant_mismatch() {
        let cred = make_sts("ten_alpha", 1_000, 900);
        let ledger = RevocationLedger::new("ten_beta").unwrap();
        assert_eq!(
            ledger.evaluate_sts(&cred, 1_500),
            Err(RevocationError::TenantMismatch)
        );
    }
}
