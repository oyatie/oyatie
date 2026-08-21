//! The verification case aggregate, its decision rules, and its state machine.
//!
//! Three authorities live here and they are deliberately separate:
//!
//! * [`assess_at`] reads the FACTS on a case (requirements, submissions,
//!   screenings, validity window) and says what they imply, as of an explicit
//!   instant. It mutates nothing.
//! * [`VerificationCase::transition_to`] is the authority on which single
//!   stored state changes are legal at all. It knows nothing about evidence.
//! * [`apply_verdict`] is the authority on how a verdict is WRITTEN: which
//!   verdicts route through a mandatory human review, and which are holds
//!   rather than moves.
//!
//! [`advance_at`] is the one place all three meet.

use crate::kernel::{
    DocumentRequirement, DocumentSubmission, REDACTED, RequirementKey, ScreeningCheck,
    ScreeningResult, Timestamp, ValidityWindow, VerificationDecision, VerificationError,
    VerificationKind, jurisdiction_matches,
};

use core::fmt;
use std::collections::BTreeSet;

/// The stable identity of a verification case.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct VerificationCaseId(pub String); // data_class: INTERNAL_ONLY

impl VerificationCaseId {
    /// Build a case id, rejecting an empty one.
    pub fn new(raw: String) -> Result<Self, VerificationError> {
        if raw.trim().is_empty() {
            return Err(VerificationError::EmptyCaseId);
        }
        Ok(Self(raw))
    }

    /// The id as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for VerificationCaseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Why an assessment landed where it did.
///
/// Reasons name obligations, checks and providers — never the content of a
/// document or the narrative of a screening match, both of which stay in SECRET
/// fields and out of anything a reason list is likely to be logged into.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssessmentReason {
    /// The case was already closed before this assessment ran.
    AlreadyTerminal { state: VerificationDecision },
    /// The evidence window has lapsed as of the assessed instant.
    ValidityWindowLapsed { expires_at: Timestamp },
    /// A human reviewer confirmed this provider's hit as a true match.
    HumanConfirmedHit { provider: String },
    /// This provider reported a hit that no human has adjudicated yet.
    UnresolvedScreeningHit { provider: String },
    /// A mandatory document obligation is still open.
    MandatoryDocumentMissing { key: RequirementKey },
    /// A required screening question has no answer on this case at all.
    ScreeningCoverageMissing { check: ScreeningCheck },
    /// A screening provider could not be reached, so the picture is incomplete.
    ScreeningProviderUnavailable { provider: String },
    /// A screening provider was reached but did not answer in time.
    ScreeningProviderTimedOut { provider: String },
    /// A screening provider does not answer this kind of case — a WIRING fault,
    /// not an outage, and the two must never read alike to an operator.
    ScreeningProviderCannotScreen {
        provider: String,
        kind: VerificationKind,
    },
    /// A screening provider answered with something this domain refuses to
    /// store.
    ScreeningResultUnusable { provider: String },
    /// The case carries no subject handle, so no provider can be asked.
    ScreeningSubjectRefMissing,
    /// The facts imply `verdict`, but the case cannot move there and is held at
    /// `state` instead.
    VerdictHeld {
        verdict: VerificationDecision,
        state: VerificationDecision,
    },
}

impl fmt::Display for AssessmentReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyTerminal { state } => write!(f, "case already {state}"),
            Self::ValidityWindowLapsed { expires_at } => {
                write!(f, "evidence window lapsed at {expires_at}")
            }
            Self::HumanConfirmedHit { provider } => {
                write!(f, "reviewer confirmed hit from {provider}")
            }
            Self::UnresolvedScreeningHit { provider } => {
                write!(f, "unresolved hit from {provider}")
            }
            Self::MandatoryDocumentMissing { key } => {
                write!(f, "mandatory document missing: {key}")
            }
            Self::ScreeningCoverageMissing { check } => {
                write!(f, "no {check} screening answer on file")
            }
            Self::ScreeningProviderUnavailable { provider } => {
                write!(f, "screening provider unavailable: {provider}")
            }
            Self::ScreeningProviderTimedOut { provider } => {
                write!(f, "screening provider timed out: {provider}")
            }
            Self::ScreeningProviderCannotScreen { provider, kind } => {
                write!(f, "screening provider {provider} does not screen {kind}")
            }
            Self::ScreeningResultUnusable { provider } => {
                write!(
                    f,
                    "screening provider {provider} returned an unusable result"
                )
            }
            Self::ScreeningSubjectRefMissing => {
                f.write_str("case carries no subject reference to screen")
            }
            Self::VerdictHeld { verdict, state } => {
                write!(f, "verdict {verdict} held at {state}")
            }
        }
    }
}

/// The verdict of an assessment plus the evidence that produced it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Assessment {
    pub decision: VerificationDecision, // data_class: TENANT_SCOPED
    pub reasons: Vec<AssessmentReason>, // data_class: TENANT_SCOPED
    pub unmet_mandatory: Vec<RequirementKey>, // data_class: INTERNAL_ONLY
    pub missing_screenings: Vec<ScreeningCheck>, // data_class: INTERNAL_ONLY
}

impl Assessment {
    /// May tenant activation proceed on this assessment?
    #[must_use]
    pub const fn permits_activation(&self) -> bool {
        self.decision.permits_activation()
    }
}

/// A refused state change.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransitionError {
    /// The edge is not in the state machine.
    IllegalTransition {
        from: VerificationDecision,
        to: VerificationDecision,
    },
    /// The case is closed; closed cases absorb.
    TerminalState {
        state: VerificationDecision,
        attempted: VerificationDecision,
    },
}

impl fmt::Display for TransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IllegalTransition { from, to } => {
                write!(f, "illegal verification transition {from} -> {to}")
            }
            Self::TerminalState { state, attempted } => write!(
                f,
                "verification case is terminal in {state}; {attempted} is unreachable without a new case"
            ),
        }
    }
}

impl std::error::Error for TransitionError {}

/// The closed transition table.
///
/// Every state lists itself, so re-asserting the current state is a legal
/// no-op. Beyond that:
///
/// * `Rejected` and `Expired` are TERMINAL — they list nothing else. A tenant
///   who later produces fresh evidence gets a NEW case, so the refusal and the
///   later approval both survive in the record.
/// * `Approved` may NOT drop straight to `Rejected`. Withdrawing an approval
///   goes through `EscalatedToHuman` first, which forces a named human review
///   between "we let you in" and "we are throwing you out". [`apply_verdict`]
///   WALKS that route so the rule is a required detour, not a dead end.
/// * Nothing returns to `Pending`. Pending means "not yet looked at"; once a
///   case has been decided or escalated, that is no longer true. A `Pending`
///   verdict on a case that has moved on is therefore a HOLD, not an edge —
///   see [`apply_verdict`].
#[must_use]
pub const fn legal_transitions(from: VerificationDecision) -> &'static [VerificationDecision] {
    match from {
        VerificationDecision::Pending => &[
            VerificationDecision::Pending,
            VerificationDecision::Approved,
            VerificationDecision::Rejected,
            VerificationDecision::EscalatedToHuman,
            VerificationDecision::Expired,
        ],
        VerificationDecision::EscalatedToHuman => &[
            VerificationDecision::EscalatedToHuman,
            VerificationDecision::Approved,
            VerificationDecision::Rejected,
            VerificationDecision::Expired,
        ],
        VerificationDecision::Approved => &[
            VerificationDecision::Approved,
            VerificationDecision::EscalatedToHuman,
            VerificationDecision::Expired,
        ],
        VerificationDecision::Rejected => &[VerificationDecision::Rejected],
        VerificationDecision::Expired => &[VerificationDecision::Expired],
    }
}

/// Is `from -> to` a single edge of the state machine?
#[must_use]
pub fn is_legal_transition(from: VerificationDecision, to: VerificationDecision) -> bool {
    legal_transitions(from).contains(&to)
}

/// The screening questions every case must have an answer to before it can be
/// approved.
///
/// IP-018 §D4 requires "sanctions clear", which is not the same as "sanctions
/// unasked": a case nobody ever screened has no more evidence of cleanliness
/// than one nobody ever documented. Sanctions is the floor for every kind;
/// consumer (KYC) cases additionally require the minor-protection answer that
/// §D4 names alongside it.
#[must_use]
pub fn default_required_screenings(kind: VerificationKind) -> Vec<ScreeningCheck> {
    match kind {
        VerificationKind::Kyc => vec![ScreeningCheck::Sanctions, ScreeningCheck::MinorProtection],
        VerificationKind::Kyb | VerificationKind::Ubo | VerificationKind::Sanctions => {
            vec![ScreeningCheck::Sanctions]
        }
    }
}

/// A KYB/KYC verification case: the obligations placed on one subject, the
/// evidence produced against them, and the decision reached so far.
#[derive(Clone, Eq, PartialEq)]
pub struct VerificationCase {
    pub tenant_id: String,                        // data_class: INTERNAL_ONLY
    pub id: VerificationCaseId,                   // data_class: INTERNAL_ONLY
    pub kind: VerificationKind,                   // data_class: INTERNAL_ONLY
    pub decision: VerificationDecision,           // data_class: TENANT_SCOPED
    pub requirements: Vec<DocumentRequirement>,   // data_class: INTERNAL_ONLY
    pub required_screenings: Vec<ScreeningCheck>, // data_class: INTERNAL_ONLY
    pub screenings: Vec<ScreeningResult>,         // data_class: TENANT_PAYLOAD
    pub subject_ref: String,                      // data_class: SECRET
    pub jurisdiction: String,                     // data_class: INTERNAL_ONLY
    pub window: ValidityWindow,                   // data_class: INTERNAL_ONLY
    pub submissions: Vec<DocumentSubmission>,     // data_class: TENANT_PAYLOAD
}

/// Redacts `subject_ref`. An adapter logging a failed settlement with
/// `tracing::error!(case = ?case, ..)` is the expected handling of a
/// [`TransitionError`], and a derived `Debug` would put the subject handle —
/// and, through the nested results, every provider narrative — into that line.
impl fmt::Debug for VerificationCase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VerificationCase")
            .field("tenant_id", &self.tenant_id)
            .field("id", &self.id)
            .field("kind", &self.kind)
            .field("decision", &self.decision)
            .field("requirements", &self.requirements)
            .field("required_screenings", &self.required_screenings)
            .field("screenings", &self.screenings)
            .field("subject_ref", &REDACTED)
            .field("jurisdiction", &self.jurisdiction)
            .field("window", &self.window)
            .field("submissions", &self.submissions)
            .finish()
    }
}

impl VerificationCase {
    /// Open a case in `Pending` with its obligations fixed up front.
    ///
    /// `tenant_id` is the scope key: every read of this aggregate is a
    /// tenant-scoped read, and IP-018 §D1 keys the case by it. A repository
    /// that hands a case to a caller must check [`belongs_to`] first — the
    /// case id alone is guessable and is not an authorization fact.
    ///
    /// `subject_ref` is a pseudonymous handle to the subject held elsewhere;
    /// this domain never receives a name, a document number or a date of birth,
    /// which is why the handle is the most sensitive field on the aggregate.
    ///
    /// The case starts out requiring [`default_required_screenings`] for its
    /// kind; [`requiring_screenings`] narrows or widens that set explicitly.
    ///
    /// [`belongs_to`]: VerificationCase::belongs_to
    /// [`requiring_screenings`]: VerificationCase::requiring_screenings
    pub fn new(
        tenant_id: String,
        id: VerificationCaseId,
        kind: VerificationKind,
        subject_ref: String,
        jurisdiction: String,
        window: ValidityWindow,
        requirements: Vec<DocumentRequirement>,
    ) -> Result<Self, VerificationError> {
        if tenant_id.trim().is_empty() {
            return Err(VerificationError::EmptyTenantId);
        }
        if subject_ref.trim().is_empty() {
            return Err(VerificationError::EmptySubjectRef);
        }
        if jurisdiction.trim().is_empty() {
            return Err(VerificationError::EmptyJurisdiction);
        }
        if requirements.is_empty() {
            return Err(VerificationError::NoRequirements);
        }

        let mut seen: BTreeSet<RequirementKey> = BTreeSet::new();
        for requirement in &requirements {
            if requirement.name.trim().is_empty() {
                return Err(VerificationError::EmptyRequirementName);
            }
            if requirement.jurisdiction.trim().is_empty() {
                return Err(VerificationError::EmptyJurisdiction);
            }
            let key = requirement.key();
            if !seen.insert(key.clone()) {
                return Err(VerificationError::DuplicateRequirement { key });
            }
        }

        Ok(Self {
            tenant_id,
            id,
            kind,
            decision: VerificationDecision::Pending,
            requirements,
            required_screenings: default_required_screenings(kind),
            screenings: Vec::new(),
            subject_ref,
            jurisdiction,
            window,
            submissions: Vec::new(),
        })
    }

    /// Replace the screening questions this case must have answers to.
    ///
    /// Passing an EMPTY set is a deliberate declaration that this case needs no
    /// screening at all. It is never the default, and it is the only way to
    /// approve a case that was never screened.
    #[must_use]
    pub fn requiring_screenings(mut self, checks: Vec<ScreeningCheck>) -> Self {
        let mut deduped: Vec<ScreeningCheck> = Vec::with_capacity(checks.len());
        for check in checks {
            if !deduped.contains(&check) {
                deduped.push(check);
            }
        }
        self.required_screenings = deduped;
        self
    }

    /// Is this case inside `tenant_id`'s scope?
    ///
    /// The guard a repository or policy layer calls before returning the case
    /// to a caller acting in a `TenantContext`. Comparison is trimmed and
    /// case-folded for the same reason jurisdictions are.
    #[must_use]
    pub fn belongs_to(&self, tenant_id: &str) -> bool {
        self.tenant_id.trim().eq_ignore_ascii_case(tenant_id.trim())
    }

    /// Record a produced document.
    ///
    /// A submission that answers no requirement of this case is REFUSED rather
    /// than stored: silently keeping it would let a jurisdiction mismatch look
    /// like progress while the real obligation stays open.
    pub fn record_submission(
        &mut self,
        submission: DocumentSubmission,
    ) -> Result<(), VerificationError> {
        let key = submission.key();
        if !self.requirements.iter().any(|r| r.key() == key) {
            return Err(VerificationError::UnknownRequirement { key });
        }
        self.submissions
            .retain(|existing| existing.key() != submission.key());
        self.submissions.push(submission);
        Ok(())
    }

    fn validate_screening(result: &ScreeningResult) -> Result<(), VerificationError> {
        if result.provider.trim().is_empty() {
            return Err(VerificationError::EmptyProvider);
        }
        if !result.hit && result.resolution != crate::kernel::ScreeningResolution::Unresolved {
            return Err(VerificationError::ResolutionWithoutHit {
                resolution: result.resolution,
            });
        }
        Ok(())
    }

    /// Record one provider's answer to ONE screening question, replacing that
    /// provider's previous answer to the SAME question so a stale hit cannot
    /// outlive its resolution.
    ///
    /// Identity is (provider, check), normalized — see [`ScreeningResult::key`].
    /// A provider's PEP answer therefore never overwrites its own sanctions
    /// answer, and a reviewer's clearance filed under a differently-capitalized
    /// spelling of the provider name still supersedes the hit it resolves.
    pub fn record_screening(&mut self, result: ScreeningResult) -> Result<(), VerificationError> {
        Self::validate_screening(&result)?;
        let key = result.key();
        self.screenings.retain(|existing| existing.key() != key);
        self.screenings.push(result);
        Ok(())
    }

    /// Record one provider RESPONSE — every answer it produced, at once.
    ///
    /// Two answers to the same question inside a single response are reduced
    /// FAIL-CLOSED: the more adverse one survives (see
    /// [`ScreeningResult::adversity_rank`]). A vendor's own response must never
    /// cancel itself out, because the direction of that loss is not symmetric —
    /// a discarded hit approves a sanctioned tenant, while a discarded clear
    /// only sends a clean one to a reviewer.
    ///
    /// Returns the number of rows actually STORED, which is the number of
    /// distinct questions the response answered, not the number of rows it
    /// contained.
    pub fn record_screening_batch(
        &mut self,
        results: Vec<ScreeningResult>,
    ) -> Result<usize, VerificationError> {
        let mut reduced: Vec<ScreeningResult> = Vec::with_capacity(results.len());
        for result in results {
            Self::validate_screening(&result)?;
            match reduced.iter_mut().find(|kept| kept.key() == result.key()) {
                Some(kept) => {
                    if result.adversity_rank() > kept.adversity_rank() {
                        *kept = result;
                    }
                }
                None => reduced.push(result),
            }
        }
        let recorded = reduced.len();
        for result in reduced {
            self.record_screening(result)?;
        }
        Ok(recorded)
    }

    /// Is a requirement discharged?
    ///
    /// Satisfaction is per (document, jurisdiction): a verified document filed
    /// under `US` does not discharge the same document under `KR`.
    #[must_use]
    pub fn is_requirement_met(&self, requirement: &DocumentRequirement) -> bool {
        self.submissions.iter().any(|submission| {
            submission.requirement_name.trim() == requirement.name.trim()
                && jurisdiction_matches(&submission.jurisdiction, &requirement.jurisdiction)
                && submission.status.satisfies_requirement()
        })
    }

    /// Every mandatory obligation still open, in declaration order.
    #[must_use]
    pub fn unmet_mandatory_requirements(&self) -> Vec<RequirementKey> {
        self.requirements
            .iter()
            .filter(|requirement| requirement.mandatory && !self.is_requirement_met(requirement))
            .map(DocumentRequirement::key)
            .collect()
    }

    /// Every required screening question with no answer on file, in declaration
    /// order.
    #[must_use]
    pub fn missing_screening_checks(&self) -> Vec<ScreeningCheck> {
        self.required_screenings
            .iter()
            .copied()
            .filter(|check| !self.screenings.iter().any(|result| result.check == *check))
            .collect()
    }

    /// Screening hits that no human has adjudicated yet, in record order.
    #[must_use]
    pub fn unresolved_hits(&self) -> Vec<&ScreeningResult> {
        self.screenings
            .iter()
            .filter(|result| result.is_blocking())
            .collect()
    }

    /// Screening hits a human has confirmed as true matches.
    #[must_use]
    pub fn confirmed_adverse_hits(&self) -> Vec<&ScreeningResult> {
        self.screenings
            .iter()
            .filter(|result| result.is_confirmed_adverse())
            .collect()
    }

    /// Has this case's evidence window lapsed as of `now`?
    #[must_use]
    pub const fn is_expired_at(&self, now: Timestamp) -> bool {
        self.window.is_expired_at(now)
    }

    /// Move the stored decision by ONE edge, refusing anything outside the
    /// state machine.
    ///
    /// This is the strict primitive. Callers writing an assessed verdict want
    /// [`apply_verdict`], which knows about the mandatory review detour and
    /// about holds.
    pub fn transition_to(&mut self, next: VerificationDecision) -> Result<(), TransitionError> {
        if !is_legal_transition(self.decision, next) {
            return Err(if self.decision.is_terminal() {
                TransitionError::TerminalState {
                    state: self.decision,
                    attempted: next,
                }
            } else {
                TransitionError::IllegalTransition {
                    from: self.decision,
                    to: next,
                }
            });
        }
        self.decision = next;
        Ok(())
    }
}

/// Assess a case's facts as of `now`, without mutating it.
///
/// Precedence, highest first — the order is the whole compliance argument:
///
/// 1. **Already closed.** A terminal stored decision is reported back as-is.
/// 2. **Human-confirmed hit → `Rejected`.** The refusal is a HUMAN's; this
///    function only records the call a reviewer already made. It outranks
///    expiry because an adverse determination is a FINDING, and a finding does
///    not become a paperwork lapse just because the evidence window ran out
///    while the reviewer was making it.
/// 3. **Lapsed window → `Expired`.** Stale evidence cannot carry a live
///    approval, and nothing short of a confirmed finding survives it.
/// 4. **Unresolved hit → `EscalatedToHuman`.** A name match is not a finding.
///    No automated refusal is ever produced from an unadjudicated hit.
/// 5. **Open mandatory obligation, or a required screening question with no
///    answer at all → `Pending`.**
/// 6. **Otherwise `Approved`** — every mandatory obligation met, every required
///    screening question answered, and no hit left standing.
///
/// Step 4 outranks step 5 on purpose: a case carrying both an open document and
/// a live hit needs a reviewer, and reporting only "still waiting on paperwork"
/// would bury the signal that actually matters.
#[must_use]
pub fn assess_at(case: &VerificationCase, now: Timestamp) -> Assessment {
    let unmet_mandatory = case.unmet_mandatory_requirements();
    let missing_screenings = case.missing_screening_checks();

    if case.decision.is_terminal() {
        return Assessment {
            decision: case.decision,
            reasons: vec![AssessmentReason::AlreadyTerminal {
                state: case.decision,
            }],
            unmet_mandatory,
            missing_screenings,
        };
    }

    let confirmed = case.confirmed_adverse_hits();
    if !confirmed.is_empty() {
        return Assessment {
            decision: VerificationDecision::Rejected,
            reasons: confirmed
                .into_iter()
                .map(|result| AssessmentReason::HumanConfirmedHit {
                    provider: result.provider.clone(),
                })
                .collect(),
            unmet_mandatory,
            missing_screenings,
        };
    }

    if case.is_expired_at(now) {
        return Assessment {
            decision: VerificationDecision::Expired,
            reasons: vec![AssessmentReason::ValidityWindowLapsed {
                expires_at: case.window.expires_at,
            }],
            unmet_mandatory,
            missing_screenings,
        };
    }

    let unresolved = case.unresolved_hits();
    if !unresolved.is_empty() {
        let mut reasons: Vec<AssessmentReason> = unresolved
            .into_iter()
            .map(|result| AssessmentReason::UnresolvedScreeningHit {
                provider: result.provider.clone(),
            })
            .collect();
        reasons.extend(open_obligation_reasons(
            &unmet_mandatory,
            &missing_screenings,
        ));
        return Assessment {
            decision: VerificationDecision::EscalatedToHuman,
            reasons,
            unmet_mandatory,
            missing_screenings,
        };
    }

    if !unmet_mandatory.is_empty() || !missing_screenings.is_empty() {
        let reasons = open_obligation_reasons(&unmet_mandatory, &missing_screenings);
        return Assessment {
            decision: VerificationDecision::Pending,
            reasons,
            unmet_mandatory,
            missing_screenings,
        };
    }

    Assessment {
        decision: VerificationDecision::Approved,
        reasons: Vec::new(),
        unmet_mandatory,
        missing_screenings,
    }
}

fn open_obligation_reasons(
    unmet_mandatory: &[RequirementKey],
    missing_screenings: &[ScreeningCheck],
) -> Vec<AssessmentReason> {
    let mut reasons: Vec<AssessmentReason> = unmet_mandatory
        .iter()
        .map(|key| AssessmentReason::MandatoryDocumentMissing { key: key.clone() })
        .collect();
    reasons.extend(
        missing_screenings
            .iter()
            .map(|check| AssessmentReason::ScreeningCoverageMissing { check: *check }),
    );
    reasons
}

/// The verdict of [`assess_at`] without its reasons.
#[must_use]
pub fn evaluate_at(case: &VerificationCase, now: Timestamp) -> VerificationDecision {
    assess_at(case, now).decision
}

/// Write an assessed verdict onto a case, and report the state it now holds.
///
/// The state machine and the decision rules answer different questions, and
/// three of the answers do not line up as a single edge. This function is where
/// that is reconciled, so callers get an operational outcome instead of a hard
/// error on a routine condition:
///
/// * **A verdict equal to the stored state is a no-op.** This is what makes a
///   terminal case absorbing rather than an error.
/// * **`Rejected` on an `Approved` case WALKS the documented route.** The table
///   forbids `Approved -> Rejected` so that a named human review always sits
///   between "we let you in" and "we are throwing you out". A `Rejected`
///   verdict only ever comes from a hit a reviewer already confirmed, so the
///   review has happened: the case is stepped `Approved -> EscalatedToHuman ->
///   Rejected` and both edges are recorded. Without this, an approved tenant
///   with a confirmed sanctions match could never be revoked through this
///   crate's own API — the settle path would return the same error forever
///   while `permits_activation()` stayed true.
/// * **A `Pending` verdict never overwrites a case that has moved on.**
///   `Pending` means "not yet looked at", which cannot become true again. A
///   case whose reviewer cleared its hit while a document is still open, or one
///   whose re-screen hit a vendor outage, assesses `Pending` and is HELD where
///   it is. That is a routine operational state, not a programming fault, and
///   [`advance_at`] reports it as [`AssessmentReason::VerdictHeld`].
///
/// Anything else the machine refuses comes back as a [`TransitionError`] with
/// the case left exactly as it was.
pub fn apply_verdict(
    case: &mut VerificationCase,
    verdict: VerificationDecision,
) -> Result<VerificationDecision, TransitionError> {
    if verdict == case.decision {
        return Ok(case.decision);
    }
    if is_legal_transition(case.decision, verdict) {
        case.transition_to(verdict)?;
        return Ok(case.decision);
    }
    if verdict == VerificationDecision::Pending && !case.decision.is_terminal() {
        return Ok(case.decision);
    }
    if !case.decision.is_terminal()
        && is_legal_transition(case.decision, VerificationDecision::EscalatedToHuman)
        && is_legal_transition(VerificationDecision::EscalatedToHuman, verdict)
    {
        case.transition_to(VerificationDecision::EscalatedToHuman)?;
        case.transition_to(verdict)?;
        return Ok(case.decision);
    }
    Err(if case.decision.is_terminal() {
        TransitionError::TerminalState {
            state: case.decision,
            attempted: verdict,
        }
    } else {
        TransitionError::IllegalTransition {
            from: case.decision,
            to: verdict,
        }
    })
}

/// Assess, then write the verdict through [`apply_verdict`].
///
/// The returned assessment's `decision` is the state the case NOW holds, so a
/// caller never has to reconcile the returned verdict against the stored one.
/// When the two differ — a `Pending` verdict held at an escalated case, say —
/// the raw verdict is preserved as an [`AssessmentReason::VerdictHeld`] and
/// every other reason is left untouched.
pub fn advance_at(
    case: &mut VerificationCase,
    now: Timestamp,
) -> Result<Assessment, TransitionError> {
    let mut assessment = assess_at(case, now);
    let settled = apply_verdict(case, assessment.decision)?;
    if settled != assessment.decision {
        assessment.reasons.push(AssessmentReason::VerdictHeld {
            verdict: assessment.decision,
            state: settled,
        });
        assessment.decision = settled;
    }
    Ok(assessment)
}
