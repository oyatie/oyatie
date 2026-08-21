//! The screening port and the case-settlement usecase.
//!
//! The port is SYNCHRONOUS and returns owned values. Real watchlist providers
//! are network services and would be async behind an adapter; that adapter is
//! out of scope for this crate (see the Gaps paragraph in `lib.rs`) and the
//! seam is drawn here so it can be added without touching the decision rules.

use core::fmt;

use crate::domain::{
    Assessment, AssessmentReason, TransitionError, VerificationCase, VerificationCaseId,
    apply_verdict, assess_at,
};
use crate::kernel::{
    REDACTED, ScreeningResult, Timestamp, VerificationDecision, VerificationError, VerificationKind,
};

/// What a screening provider is being asked about.
///
/// The request carries a pseudonymous `subject_ref`, never identity attributes:
/// resolving that handle to a person is the adapter's job and its blast radius,
/// not this domain's. `Debug` redacts the handle for the same reason it is
/// redacted on the case.
#[derive(Clone, Eq, PartialEq)]
pub struct ScreeningRequest {
    pub tenant_id: String,           // data_class: INTERNAL_ONLY
    pub case_id: VerificationCaseId, // data_class: INTERNAL_ONLY
    pub kind: VerificationKind,      // data_class: INTERNAL_ONLY
    pub subject_ref: String,         // data_class: SECRET
    pub jurisdiction: String,        // data_class: INTERNAL_ONLY
}

impl fmt::Debug for ScreeningRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScreeningRequest")
            .field("tenant_id", &self.tenant_id)
            .field("case_id", &self.case_id)
            .field("kind", &self.kind)
            .field("subject_ref", &REDACTED)
            .field("jurisdiction", &self.jurisdiction)
            .finish()
    }
}

impl ScreeningRequest {
    /// The screening request implied by a case.
    #[must_use]
    pub fn for_case(case: &VerificationCase) -> Self {
        Self {
            tenant_id: case.tenant_id.clone(),
            case_id: case.id.clone(),
            kind: case.kind,
            subject_ref: case.subject_ref.clone(),
            jurisdiction: case.jurisdiction.clone(),
        }
    }
}

/// Why a screening lookup produced no usable answer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScreeningError {
    ProviderUnavailable {
        provider: String,
    },
    ProviderTimeout {
        provider: String,
    },
    UnsupportedKind {
        provider: String,
        kind: VerificationKind,
    },
    EmptySubjectRef,
    InvalidResult {
        provider: String,
        cause: VerificationError,
    },
}

impl ScreeningError {
    /// The provider this failure is attributed to, when there is one.
    #[must_use]
    pub fn provider(&self) -> Option<&str> {
        match self {
            Self::ProviderUnavailable { provider }
            | Self::ProviderTimeout { provider }
            | Self::UnsupportedKind { provider, .. }
            | Self::InvalidResult { provider, .. } => Some(provider.as_str()),
            Self::EmptySubjectRef => None,
        }
    }

    /// The assessment reason this failure contributes.
    ///
    /// Each variant keeps its own reason. An operator working
    /// `tenancy/runbooks/kyb-kyc-pipeline-stalled.md` has to tell "the vendor
    /// is down" from "the wrong vendor is bound to this kind of case" from "the
    /// vendor answered with something we refused to store" — those have three
    /// different remedies, and collapsing them into one outage reason sends
    /// every one of them to the vendor's support desk.
    #[must_use]
    pub fn assessment_reason(&self) -> AssessmentReason {
        match self {
            Self::ProviderUnavailable { provider } => {
                AssessmentReason::ScreeningProviderUnavailable {
                    provider: provider.clone(),
                }
            }
            Self::ProviderTimeout { provider } => AssessmentReason::ScreeningProviderTimedOut {
                provider: provider.clone(),
            },
            Self::UnsupportedKind { provider, kind } => {
                AssessmentReason::ScreeningProviderCannotScreen {
                    provider: provider.clone(),
                    kind: *kind,
                }
            }
            Self::InvalidResult { provider, .. } => AssessmentReason::ScreeningResultUnusable {
                provider: provider.clone(),
            },
            Self::EmptySubjectRef => AssessmentReason::ScreeningSubjectRefMissing,
        }
    }
}

impl fmt::Display for ScreeningError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProviderUnavailable { provider } => {
                write!(f, "screening provider {provider} is unavailable")
            }
            Self::ProviderTimeout { provider } => {
                write!(f, "screening provider {provider} timed out")
            }
            Self::UnsupportedKind { provider, kind } => {
                write!(f, "screening provider {provider} does not screen {kind}")
            }
            Self::EmptySubjectRef => {
                f.write_str("screening request carries an empty subject reference")
            }
            Self::InvalidResult { provider, cause } => {
                write!(
                    f,
                    "screening provider {provider} returned an unusable result: {cause}"
                )
            }
        }
    }
}

impl std::error::Error for ScreeningError {}

/// The port a watchlist / sanctions / adverse-media provider implements.
pub trait ScreeningPort {
    /// Screen one subject and return every result the provider produced.
    ///
    /// A response may answer several questions at once — a sanctions row, a PEP
    /// row, an adverse-media row — each attributed to its own
    /// [`ScreeningCheck`]. Implementations must be deterministic for a given
    /// request so that a decision can be replayed from stored facts.
    ///
    /// [`ScreeningCheck`]: crate::kernel::ScreeningCheck
    fn screen(&self, request: &ScreeningRequest) -> Result<Vec<ScreeningResult>, ScreeningError>;
}

/// Pull one fresh provider RESPONSE into the case, replacing that provider's
/// previous answer to each question it answers.
///
/// Returns how many rows were actually STORED. Two answers to the same question
/// inside one response reduce fail-closed to the more adverse one (see
/// [`VerificationCase::record_screening_batch`]), so the count is what the case
/// now holds from this response rather than what the port happened to return.
///
/// One call is one provider's response, so a row this domain refuses to store
/// is attributed to the provider the response came from.
pub fn refresh_screenings<P: ScreeningPort + ?Sized>(
    case: &mut VerificationCase,
    port: &P,
) -> Result<usize, ScreeningError> {
    let request = ScreeningRequest::for_case(case);
    if request.subject_ref.trim().is_empty() {
        return Err(ScreeningError::EmptySubjectRef);
    }
    let results = port.screen(&request)?;
    let provider = results
        .first()
        .map_or_else(String::new, |result| result.provider.clone());
    case.record_screening_batch(results)
        .map_err(|cause| ScreeningError::InvalidResult { provider, cause })
}

/// Refresh screenings and assess the case as of `now`, without writing the
/// verdict back.
///
/// A provider failure is NOT an error of this function: the case simply cannot
/// be GRANTED an approval on an incomplete screening picture, so a would-be
/// approval is downgraded and the failure is named with its own reason. That is
/// IP-018's "provider timeout keeps the tenant pending" rule, and it is
/// one-directional — an outage can never manufacture a refusal either, and it
/// never revokes an approval a complete screening picture already earned
/// (`Pending` is a hold, not an edge; see [`apply_verdict`]).
///
/// A case whose evidence window has already lapsed short-circuits before the
/// port is called: there is nothing a fresh screening could rescue.
///
/// [`apply_verdict`]: crate::domain::apply_verdict
pub fn assess_with_screening<P: ScreeningPort + ?Sized>(
    case: &mut VerificationCase,
    port: &P,
    now: Timestamp,
) -> Assessment {
    if case.decision.is_terminal() || case.is_expired_at(now) {
        return assess_at(case, now);
    }

    match refresh_screenings(case, port) {
        Ok(_) => assess_at(case, now),
        Err(error) => {
            let mut assessment = assess_at(case, now);
            if assessment.decision == VerificationDecision::Approved {
                assessment.decision = VerificationDecision::Pending;
            }
            assessment.reasons.push(error.assessment_reason());
            assessment
        }
    }
}

/// Refresh screenings, assess as of `now`, and write the verdict through
/// [`apply_verdict`].
///
/// Returns the assessment that was applied — whose `decision` is the state the
/// case now holds — or the [`TransitionError`] that stopped it, in which case
/// the case is left exactly as it was.
///
/// [`apply_verdict`]: crate::domain::apply_verdict
pub fn settle_with_screening<P: ScreeningPort + ?Sized>(
    case: &mut VerificationCase,
    port: &P,
    now: Timestamp,
) -> Result<Assessment, TransitionError> {
    let mut assessment = assess_with_screening(case, port, now);
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
