//! Value types of the KYB/KYC verifier domain: the timeline, the validity
//! window, document requirements and submissions, and screening results.
//!
//! Everything here is a plain owned value with no I/O and no clock read.
//! `Timestamp` exists precisely so that expiry is a function of its inputs.
//!
//! Types holding a SECRET field render a REDACTING `Debug`. A derived `Debug`
//! would put the evidence handle and the provider's match narrative into the
//! first `tracing::error!(case = ?case, ..)` an adapter writes, which is exactly
//! the leak the classification exists to prevent.

use core::fmt;

/// An instant on the verification timeline, in whole seconds since the Unix
/// epoch.
///
/// Time is always a PARAMETER in this crate: no function reads a clock, so
/// every expiry verdict is reproducible from its arguments alone.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Timestamp(pub i64); // data_class: INTERNAL_ONLY

impl Timestamp {
    /// Build a timestamp from whole seconds since the Unix epoch.
    #[must_use]
    pub const fn new(seconds: i64) -> Self {
        Self(seconds)
    }

    /// Whole seconds since the Unix epoch.
    #[must_use]
    pub const fn seconds(self) -> i64 {
        self.0
    }

    /// This instant shifted by `seconds`, saturating instead of wrapping so a
    /// hostile or corrupt input can never fold the timeline around.
    #[must_use]
    pub const fn saturating_shift(self, seconds: i64) -> Self {
        Self(self.0.saturating_add(seconds))
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "epoch+{}s", self.0)
    }
}

/// The window during which a case's evidence is considered fresh.
///
/// The window is HALF-OPEN — `[opened_at, expires_at)`. Evidence is fresh
/// strictly before `expires_at`; at `expires_at` itself it has already lapsed.
/// Choosing the half-open form makes "renewed at exactly the expiry instant"
/// unambiguous: the new window owns that instant, the old one does not.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidityWindow {
    pub opened_at: Timestamp,  // data_class: INTERNAL_ONLY
    pub expires_at: Timestamp, // data_class: INTERNAL_ONLY
}

impl ValidityWindow {
    /// Build a window, rejecting anything that is not strictly positive in
    /// length. A zero-length window would be expired at the instant it opened,
    /// which is a construction bug rather than a legitimate state.
    pub fn new(opened_at: Timestamp, expires_at: Timestamp) -> Result<Self, VerificationError> {
        if expires_at <= opened_at {
            return Err(VerificationError::InvalidValidityWindow {
                opened_at,
                expires_at,
            });
        }
        Ok(Self {
            opened_at,
            expires_at,
        })
    }

    /// Has the evidence window lapsed as of `now`?
    ///
    /// True at `expires_at` and after it; false strictly before it.
    #[must_use]
    pub const fn is_expired_at(&self, now: Timestamp) -> bool {
        now.0 >= self.expires_at.0
    }

    /// Is `now` inside the window at all (never before it opened, never at or
    /// after it lapsed)?
    #[must_use]
    pub const fn contains(&self, now: Timestamp) -> bool {
        now.0 >= self.opened_at.0 && now.0 < self.expires_at.0
    }

    /// The last instant at which this window is still fresh.
    ///
    /// This is a property of the WINDOW, not a substitute for a clock reading.
    /// Nothing in this crate may use it to decide whether evidence is fresh —
    /// by construction it never is lapsed, so such a check always passes.
    #[must_use]
    pub const fn last_fresh_instant(&self) -> Timestamp {
        Timestamp(self.expires_at.0.saturating_sub(1))
    }
}

/// What a verification case is proving.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum VerificationKind {
    Kyb,
    Kyc,
    Ubo,
    Sanctions,
}

impl VerificationKind {
    /// Stable lowercase label for logs and events.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Kyb => "kyb",
            Self::Kyc => "kyc",
            Self::Ubo => "ubo",
            Self::Sanctions => "sanctions",
        }
    }
}

impl fmt::Display for VerificationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Which watchlist question a screening result answers.
///
/// IP-018 §D4 makes approval depend on sanctions AND PEP AND adverse-media
/// (and, for consumer cases, minor protection). Those are independent
/// questions, so they are independent ANSWERS: a result is identified by
/// (provider, check), never by provider alone. Collapsing them would let one
/// vendor's PEP clearance overwrite the same vendor's sanctions hit.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ScreeningCheck {
    Sanctions,
    Pep,
    AdverseMedia,
    MinorProtection,
}

impl ScreeningCheck {
    /// Stable kebab-case label for logs and events.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Sanctions => "sanctions",
            Self::Pep => "pep",
            Self::AdverseMedia => "adverse-media",
            Self::MinorProtection => "minor-protection",
        }
    }
}

impl fmt::Display for ScreeningCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// The decision state of a verification case.
///
/// `EscalatedToHuman` is not a courtesy state: it is the ONLY route from an
/// adverse screening signal to a refusal, so that no tenant is ever refused by
/// a machine acting alone on a name match.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum VerificationDecision {
    Pending,
    Approved,
    Rejected,
    EscalatedToHuman,
    Expired,
}

impl VerificationDecision {
    /// Stable kebab-case label for logs and events.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::EscalatedToHuman => "escalated-to-human",
            Self::Expired => "expired",
        }
    }

    /// Terminal states absorb: the only legal transition out of one is back to
    /// itself. Fresh evidence opens a NEW case rather than resurrecting a
    /// closed one, so the adjudication trail stays append-only.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Rejected | Self::Expired)
    }

    /// May tenant lifecycle activation proceed on this decision?
    #[must_use]
    pub const fn permits_activation(self) -> bool {
        matches!(self, Self::Approved)
    }
}

impl fmt::Display for VerificationDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Normalize a jurisdiction label for comparison: trimmed and ASCII-uppercased,
/// so `"kr"`, `" KR "` and `"KR"` are one jurisdiction while `"KR"` and `"US"`
/// stay two.
#[must_use]
pub fn normalized_jurisdiction(raw: &str) -> String {
    raw.trim().to_ascii_uppercase()
}

/// Do two jurisdiction labels denote the same jurisdiction?
#[must_use]
pub fn jurisdiction_matches(left: &str, right: &str) -> bool {
    normalized_jurisdiction(left) == normalized_jurisdiction(right)
}

/// Normalize a provider name for comparison: trimmed and ASCII-lowercased.
///
/// Provider names arrive from adapters and reviewer tooling that disagree about
/// capitalization. Comparing them raw would file `"Acuris"` and `"acuris"` as
/// two providers, and a reviewer's clearance recorded under the other spelling
/// would never supersede the hit it was meant to resolve.
#[must_use]
pub fn normalized_provider(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}

/// A document a case must produce, scoped to the jurisdiction that demands it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentRequirement {
    pub name: String,         // data_class: INTERNAL_ONLY
    pub mandatory: bool,      // data_class: INTERNAL_ONLY
    pub jurisdiction: String, // data_class: INTERNAL_ONLY
}

impl DocumentRequirement {
    /// Build a requirement, rejecting an empty document name or jurisdiction —
    /// an unnamed requirement can never be satisfied and would silently pin a
    /// case at Pending forever.
    pub fn new(
        name: String,
        mandatory: bool,
        jurisdiction: String,
    ) -> Result<Self, VerificationError> {
        if name.trim().is_empty() {
            return Err(VerificationError::EmptyRequirementName);
        }
        if jurisdiction.trim().is_empty() {
            return Err(VerificationError::EmptyJurisdiction);
        }
        Ok(Self {
            name,
            mandatory,
            jurisdiction,
        })
    }

    /// The normalized identity of this requirement.
    #[must_use]
    pub fn key(&self) -> RequirementKey {
        RequirementKey {
            name: self.name.trim().to_owned(),
            jurisdiction: normalized_jurisdiction(&self.jurisdiction),
        }
    }
}

/// The normalized (document, jurisdiction) identity of a requirement.
///
/// A requirement is identified by BOTH halves: the same document name under a
/// different jurisdiction is a different obligation and is not satisfied by the
/// other jurisdiction's paperwork.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RequirementKey {
    pub name: String,         // data_class: INTERNAL_ONLY
    pub jurisdiction: String, // data_class: INTERNAL_ONLY
}

impl fmt::Display for RequirementKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.jurisdiction)
    }
}

/// Where a submitted document stands in review.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DocumentStatus {
    Submitted,
    Verified,
    Rejected,
    Expired,
}

impl DocumentStatus {
    /// Only a verified document discharges a requirement. Submitted-but-unread,
    /// rejected and lapsed documents all leave the obligation open.
    #[must_use]
    pub const fn satisfies_requirement(self) -> bool {
        matches!(self, Self::Verified)
    }

    /// Stable lowercase label for logs and events.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Submitted => "submitted",
            Self::Verified => "verified",
            Self::Rejected => "rejected",
            Self::Expired => "expired",
        }
    }
}

impl fmt::Display for DocumentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// The placeholder rendered in `Debug` output in place of a SECRET field.
pub const REDACTED: &str = "<redacted:secret>";

/// A document produced against a requirement.
///
/// The document CONTENT never enters this domain: `evidence_ref` is an opaque
/// handle into the evidence store, which is why it is classified SECRET while
/// the requirement name it answers is not.
#[derive(Clone, Eq, PartialEq)]
pub struct DocumentSubmission {
    pub requirement_name: String, // data_class: INTERNAL_ONLY
    pub jurisdiction: String,     // data_class: INTERNAL_ONLY
    pub status: DocumentStatus,   // data_class: TENANT_SCOPED
    pub evidence_ref: String,     // data_class: SECRET
}

/// Redacts `evidence_ref`: a derived `Debug` would carry the evidence handle
/// into any log line that formats a submission or the case holding it.
impl fmt::Debug for DocumentSubmission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DocumentSubmission")
            .field("requirement_name", &self.requirement_name)
            .field("jurisdiction", &self.jurisdiction)
            .field("status", &self.status)
            .field("evidence_ref", &REDACTED)
            .finish()
    }
}

impl DocumentSubmission {
    /// Build a submission, rejecting an empty requirement name, jurisdiction or
    /// evidence handle.
    pub fn new(
        requirement_name: String,
        jurisdiction: String,
        status: DocumentStatus,
        evidence_ref: String,
    ) -> Result<Self, VerificationError> {
        if requirement_name.trim().is_empty() {
            return Err(VerificationError::EmptyRequirementName);
        }
        if jurisdiction.trim().is_empty() {
            return Err(VerificationError::EmptyJurisdiction);
        }
        if evidence_ref.trim().is_empty() {
            return Err(VerificationError::EmptyEvidenceRef);
        }
        Ok(Self {
            requirement_name,
            jurisdiction,
            status,
            evidence_ref,
        })
    }

    /// The normalized requirement this submission answers.
    #[must_use]
    pub fn key(&self) -> RequirementKey {
        RequirementKey {
            name: self.requirement_name.trim().to_owned(),
            jurisdiction: normalized_jurisdiction(&self.jurisdiction),
        }
    }
}

/// What a human reviewer concluded about a screening hit.
///
/// A hit is a NAME MATCH, not a finding. Until a reviewer resolves it the case
/// escalates; the machine never turns `Unresolved` into a refusal by itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ScreeningResolution {
    Unresolved,
    ClearedByReviewer,
    ConfirmedByReviewer,
}

impl ScreeningResolution {
    /// Stable kebab-case label for logs and events.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Unresolved => "unresolved",
            Self::ClearedByReviewer => "cleared-by-reviewer",
            Self::ConfirmedByReviewer => "confirmed-by-reviewer",
        }
    }
}

impl fmt::Display for ScreeningResolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// The normalized (provider, check) identity of a screening answer.
///
/// One provider answering three questions produces three independent rows. Only
/// a LATER answer to the SAME question supersedes an earlier one.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ScreeningKey {
    pub provider: String,      // data_class: INTERNAL_ONLY
    pub check: ScreeningCheck, // data_class: INTERNAL_ONLY
}

impl fmt::Display for ScreeningKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.provider, self.check)
    }
}

/// One screening provider's answer to ONE watchlist question about a subject.
///
/// `details` is the provider's free-text match narrative. It routinely carries
/// third-party identity data about the matched watchlist entry, which is why it
/// is SECRET, why nothing in this crate renders it into a decision reason, and
/// why `Debug` redacts it.
#[derive(Clone, Eq, PartialEq)]
pub struct ScreeningResult {
    pub provider: String,                // data_class: INTERNAL_ONLY
    pub check: ScreeningCheck,           // data_class: INTERNAL_ONLY
    pub hit: bool,                       // data_class: TENANT_SCOPED
    pub details: String,                 // data_class: SECRET
    pub resolution: ScreeningResolution, // data_class: TENANT_SCOPED
}

/// Redacts `details`: the match narrative is third-party identity data and a
/// derived `Debug` would print it verbatim into any log line.
impl fmt::Debug for ScreeningResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScreeningResult")
            .field("provider", &self.provider)
            .field("check", &self.check)
            .field("hit", &self.hit)
            .field("details", &REDACTED)
            .field("resolution", &self.resolution)
            .finish()
    }
}

impl ScreeningResult {
    /// Build a screening result attributed to an explicit check.
    ///
    /// A clear result carrying a reviewer resolution is rejected: there is
    /// nothing to resolve, and accepting it would let a "cleared" flag on a
    /// no-hit record read as adjudication that never happened.
    pub fn for_check(
        provider: String,
        check: ScreeningCheck,
        hit: bool,
        details: String,
        resolution: ScreeningResolution,
    ) -> Result<Self, VerificationError> {
        if provider.trim().is_empty() {
            return Err(VerificationError::EmptyProvider);
        }
        if !hit && resolution != ScreeningResolution::Unresolved {
            return Err(VerificationError::ResolutionWithoutHit { resolution });
        }
        Ok(Self {
            provider,
            check,
            hit,
            details,
            resolution,
        })
    }

    /// Build a SANCTIONS screening result.
    ///
    /// Preserved scaffold signature. It predates per-check attribution, so it
    /// files the answer under [`ScreeningCheck::Sanctions`] — the one check
    /// every case requires. An adapter reporting a PEP or adverse-media answer
    /// must use [`ScreeningResult::for_check`], otherwise it files that answer
    /// against the sanctions question.
    pub fn new(
        provider: String,
        hit: bool,
        details: String,
        resolution: ScreeningResolution,
    ) -> Result<Self, VerificationError> {
        Self::for_check(
            provider,
            ScreeningCheck::Sanctions,
            hit,
            details,
            resolution,
        )
    }

    /// A clear sanctions result from `provider`.
    pub fn clear(provider: String) -> Result<Self, VerificationError> {
        Self::new(
            provider,
            false,
            String::new(),
            ScreeningResolution::Unresolved,
        )
    }

    /// A clear result from `provider` for an explicit check.
    pub fn clear_for_check(
        provider: String,
        check: ScreeningCheck,
    ) -> Result<Self, VerificationError> {
        Self::for_check(
            provider,
            check,
            false,
            String::new(),
            ScreeningResolution::Unresolved,
        )
    }

    /// The normalized (provider, check) identity of this answer.
    #[must_use]
    pub fn key(&self) -> ScreeningKey {
        ScreeningKey {
            provider: normalized_provider(&self.provider),
            check: self.check,
        }
    }

    /// Does this result block a decision until a human looks at it?
    #[must_use]
    pub const fn is_blocking(&self) -> bool {
        self.hit && matches!(self.resolution, ScreeningResolution::Unresolved)
    }

    /// Has a human confirmed this hit as a true match? Only then may the domain
    /// record a refusal, and even then it is recording the human's call.
    #[must_use]
    pub const fn is_confirmed_adverse(&self) -> bool {
        self.hit && matches!(self.resolution, ScreeningResolution::ConfirmedByReviewer)
    }

    /// How adverse this answer is, for fail-closed reduction of a batch.
    ///
    /// When one provider response contains two answers to the SAME question,
    /// the more adverse one survives. A vendor's own response must never
    /// cancel itself out, and the direction of that reduction is the whole
    /// safety property: a lost hit approves a sanctioned tenant, a lost clear
    /// only sends a clean one to a reviewer.
    #[must_use]
    pub const fn adversity_rank(&self) -> u8 {
        if self.is_confirmed_adverse() {
            3
        } else if self.is_blocking() {
            2
        } else if self.hit {
            1
        } else {
            0
        }
    }
}

/// Construction and fact-recording failures in the verifier domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerificationError {
    EmptyTenantId,
    EmptyCaseId,
    EmptySubjectRef,
    EmptyJurisdiction,
    EmptyRequirementName,
    EmptyEvidenceRef,
    EmptyProvider,
    NoRequirements,
    DuplicateRequirement {
        key: RequirementKey,
    },
    UnknownRequirement {
        key: RequirementKey,
    },
    InvalidValidityWindow {
        opened_at: Timestamp,
        expires_at: Timestamp,
    },
    ResolutionWithoutHit {
        resolution: ScreeningResolution,
    },
}

impl fmt::Display for VerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTenantId => f.write_str("verification case tenant id must not be empty"),
            Self::EmptyCaseId => f.write_str("verification case id must not be empty"),
            Self::EmptySubjectRef => f.write_str("case subject reference must not be empty"),
            Self::EmptyJurisdiction => f.write_str("jurisdiction must not be empty"),
            Self::EmptyRequirementName => {
                f.write_str("document requirement name must not be empty")
            }
            Self::EmptyEvidenceRef => f.write_str("document evidence reference must not be empty"),
            Self::EmptyProvider => f.write_str("screening provider must not be empty"),
            Self::NoRequirements => {
                f.write_str("a verification case must carry at least one document requirement")
            }
            Self::DuplicateRequirement { key } => {
                write!(f, "duplicate document requirement {key}")
            }
            Self::UnknownRequirement { key } => {
                write!(f, "no document requirement matches {key}")
            }
            Self::InvalidValidityWindow {
                opened_at,
                expires_at,
            } => write!(
                f,
                "validity window must be strictly positive in length, got [{opened_at}, {expires_at})"
            ),
            Self::ResolutionWithoutHit { resolution } => write!(
                f,
                "a screening result with no hit cannot carry resolution {resolution}"
            ),
        }
    }
}

impl std::error::Error for VerificationError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn window() -> ValidityWindow {
        ValidityWindow::new(Timestamp::new(1_000), Timestamp::new(2_000))
            .expect("fixture window is strictly positive in length")
    }

    #[test]
    fn validity_window_rejects_non_positive_length() {
        let zero_length = ValidityWindow::new(Timestamp::new(1_000), Timestamp::new(1_000))
            .expect_err("a zero-length window is expired at the instant it opens");
        assert_eq!(
            zero_length,
            VerificationError::InvalidValidityWindow {
                opened_at: Timestamp::new(1_000),
                expires_at: Timestamp::new(1_000),
            }
        );

        let inverted = ValidityWindow::new(Timestamp::new(1_000), Timestamp::new(999))
            .expect_err("a window cannot expire before it opens");
        assert!(matches!(
            inverted,
            VerificationError::InvalidValidityWindow { .. }
        ));
    }

    #[test]
    fn window_is_fresh_one_second_before_expiry() {
        assert!(!window().is_expired_at(Timestamp::new(1_999)));
    }

    #[test]
    fn window_is_expired_exactly_at_expiry() {
        // Half-open [opened_at, expires_at): the expiry instant itself is OUT.
        assert!(window().is_expired_at(Timestamp::new(2_000)));
    }

    #[test]
    fn window_is_expired_one_second_after_expiry() {
        assert!(window().is_expired_at(Timestamp::new(2_001)));
    }

    #[test]
    fn window_does_not_contain_instants_before_it_opened() {
        assert!(!window().contains(Timestamp::new(999)));
        assert!(window().contains(Timestamp::new(1_000)));
        assert!(!window().contains(Timestamp::new(2_000)));
    }

    #[test]
    fn last_fresh_instant_is_one_second_before_expiry() {
        assert_eq!(window().last_fresh_instant(), Timestamp::new(1_999));
        assert!(!window().is_expired_at(window().last_fresh_instant()));
    }

    #[test]
    fn timestamp_shift_saturates_instead_of_wrapping() {
        assert_eq!(
            Timestamp::new(i64::MAX).saturating_shift(10),
            Timestamp::new(i64::MAX)
        );
        assert_eq!(Timestamp::new(10).saturating_shift(-4), Timestamp::new(6));
    }

    #[test]
    fn jurisdiction_matching_ignores_case_and_padding_but_not_identity() {
        assert!(jurisdiction_matches("kr", " KR "));
        assert!(jurisdiction_matches("Kr", "kR"));
        assert!(!jurisdiction_matches("KR", "US"));
        assert_eq!(normalized_jurisdiction("  kr "), "KR");
    }

    #[test]
    fn provider_names_normalize_case_and_padding() {
        assert_eq!(normalized_provider("  Acuris "), "acuris");
        assert_eq!(normalized_provider("ACURIS"), normalized_provider("acuris"));
        assert_ne!(normalized_provider("acuris"), normalized_provider("ofac"));
    }

    #[test]
    fn screening_result_rejects_a_resolution_with_no_hit() {
        let error = ScreeningResult::new(
            "fixture-screening-provider".to_owned(),
            false,
            String::new(),
            ScreeningResolution::ClearedByReviewer,
        )
        .expect_err("a clear result has nothing for a reviewer to clear");
        assert_eq!(
            error,
            VerificationError::ResolutionWithoutHit {
                resolution: ScreeningResolution::ClearedByReviewer,
            }
        );
    }

    #[test]
    fn screening_result_rejects_an_empty_provider() {
        let error = ScreeningResult::new(
            "   ".to_owned(),
            true,
            "SYNTHETIC FIXTURE narrative".to_owned(),
            ScreeningResolution::Unresolved,
        )
        .expect_err("an unattributed screening result cannot be adjudicated");
        assert_eq!(error, VerificationError::EmptyProvider);
    }

    #[test]
    fn a_screening_key_separates_checks_and_folds_provider_case() {
        let sanctions = ScreeningResult::for_check(
            "Acuris".to_owned(),
            ScreeningCheck::Sanctions,
            false,
            String::new(),
            ScreeningResolution::Unresolved,
        )
        .expect("fixture result is valid");
        let pep = ScreeningResult::for_check(
            "acuris".to_owned(),
            ScreeningCheck::Pep,
            false,
            String::new(),
            ScreeningResolution::Unresolved,
        )
        .expect("fixture result is valid");
        let same_question = ScreeningResult::for_check(
            "  ACURIS ".to_owned(),
            ScreeningCheck::Sanctions,
            false,
            String::new(),
            ScreeningResolution::Unresolved,
        )
        .expect("fixture result is valid");

        assert_ne!(sanctions.key(), pep.key(), "two questions, two answers");
        assert_eq!(
            sanctions.key(),
            same_question.key(),
            "one question spelled two ways is still one question"
        );
    }

    #[test]
    fn the_scaffold_constructor_files_answers_under_the_sanctions_check() {
        let result = ScreeningResult::clear("ofac".to_owned()).expect("a clear result is valid");
        assert_eq!(result.check, ScreeningCheck::Sanctions);
    }

    #[test]
    fn adversity_rank_orders_a_confirmed_hit_above_everything_else() {
        let provider = "ofac".to_owned();
        let narrative = "SYNTHETIC FIXTURE narrative".to_owned();
        let confirmed = ScreeningResult::new(
            provider.clone(),
            true,
            narrative.clone(),
            ScreeningResolution::ConfirmedByReviewer,
        )
        .expect("fixture result is valid");
        let unresolved = ScreeningResult::new(
            provider.clone(),
            true,
            narrative.clone(),
            ScreeningResolution::Unresolved,
        )
        .expect("fixture result is valid");
        let cleared = ScreeningResult::new(
            provider.clone(),
            true,
            narrative,
            ScreeningResolution::ClearedByReviewer,
        )
        .expect("fixture result is valid");
        let clear = ScreeningResult::clear(provider).expect("fixture result is valid");

        assert!(confirmed.adversity_rank() > unresolved.adversity_rank());
        assert!(unresolved.adversity_rank() > cleared.adversity_rank());
        assert!(cleared.adversity_rank() > clear.adversity_rank());
    }

    #[test]
    fn debug_redacts_the_provider_match_narrative() {
        let narrative = "SYNTHETIC FIXTURE: matched entry dossier";
        let result = ScreeningResult::new(
            "ofac".to_owned(),
            true,
            narrative.to_owned(),
            ScreeningResolution::Unresolved,
        )
        .expect("fixture result is valid");

        let rendered = format!("{result:?}");
        assert!(
            !rendered.contains(narrative),
            "a SECRET narrative must not reach a log line through Debug: {rendered}"
        );
        assert!(rendered.contains(REDACTED));
        assert!(rendered.contains("ofac"), "attribution is not secret");
    }

    #[test]
    fn debug_redacts_the_document_evidence_handle() {
        let evidence = "evidence://fixture/synthetic-0001";
        let submission = DocumentSubmission::new(
            "business-registration".to_owned(),
            "KR".to_owned(),
            DocumentStatus::Verified,
            evidence.to_owned(),
        )
        .expect("fixture submission is valid");

        let rendered = format!("{submission:?}");
        assert!(
            !rendered.contains(evidence),
            "a SECRET evidence handle must not reach a log line through Debug: {rendered}"
        );
        assert!(rendered.contains(REDACTED));
    }

    #[test]
    fn only_a_verified_document_satisfies_a_requirement() {
        assert!(DocumentStatus::Verified.satisfies_requirement());
        assert!(!DocumentStatus::Submitted.satisfies_requirement());
        assert!(!DocumentStatus::Rejected.satisfies_requirement());
        assert!(!DocumentStatus::Expired.satisfies_requirement());
    }

    #[test]
    fn requirement_construction_rejects_empty_name_and_jurisdiction() {
        assert_eq!(
            DocumentRequirement::new(" ".to_owned(), true, "KR".to_owned())
                .expect_err("an unnamed requirement can never be discharged"),
            VerificationError::EmptyRequirementName
        );
        assert_eq!(
            DocumentRequirement::new("business-registration".to_owned(), true, String::new())
                .expect_err("a requirement must name the jurisdiction demanding it"),
            VerificationError::EmptyJurisdiction
        );
    }

    #[test]
    fn submission_construction_requires_an_evidence_handle() {
        assert_eq!(
            DocumentSubmission::new(
                "business-registration".to_owned(),
                "KR".to_owned(),
                DocumentStatus::Verified,
                "  ".to_owned(),
            )
            .expect_err("a verified document must point at stored evidence"),
            VerificationError::EmptyEvidenceRef
        );
    }

    #[test]
    fn blocking_and_confirmed_hits_are_distinct_predicates() {
        let unresolved = ScreeningResult::new(
            "fixture-screening-provider".to_owned(),
            true,
            "SYNTHETIC FIXTURE narrative".to_owned(),
            ScreeningResolution::Unresolved,
        )
        .expect("fixture result is valid");
        let cleared = ScreeningResult::new(
            "fixture-screening-provider".to_owned(),
            true,
            "SYNTHETIC FIXTURE narrative".to_owned(),
            ScreeningResolution::ClearedByReviewer,
        )
        .expect("fixture result is valid");
        let confirmed = ScreeningResult::new(
            "fixture-screening-provider".to_owned(),
            true,
            "SYNTHETIC FIXTURE narrative".to_owned(),
            ScreeningResolution::ConfirmedByReviewer,
        )
        .expect("fixture result is valid");

        assert!(unresolved.is_blocking() && !unresolved.is_confirmed_adverse());
        assert!(!cleared.is_blocking() && !cleared.is_confirmed_adverse());
        assert!(!confirmed.is_blocking() && confirmed.is_confirmed_adverse());
    }
}
