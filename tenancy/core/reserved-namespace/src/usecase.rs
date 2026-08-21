//! The guard itself: sequences the two ports around the pure decisions in
//! [`crate::domain`] and produces either one [`NamespaceDecision`] or one
//! [`NamespaceUsecaseError`] — never a decision that secretly means
//! "something broke".

use crate::domain::{MalformedReason, ReservedSet, fnv1a_64, skeleton, validate_syntax};
use crate::kernel::{
    NamespaceActionAuthorizer, NamespaceCandidate, NamespaceDecision, NamespaceUsecaseError,
    ReservedNamespaceSource,
};

/// The full record of one evaluation, for audit emission.
///
/// IP-017 §D.5 requires the refusal event to carry the normalized skeleton,
/// the matched reserved class, the actor, the tenant id and a candidate hash
/// — and NOT the raw candidate. This struct is that payload minus the actor,
/// which the caller already holds in [`NamespaceCandidate::principal`], plus
/// §D.4's audit correlation id so two refusals of the same string from
/// different tenants are not byte-identical events.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamespaceEvaluation {
    /// The verdict.
    pub decision: NamespaceDecision, // data_class: INTERNAL_ONLY
    /// The candidate's confusable skeleton. Derived from tenant input, so
    /// it carries tenant payload even though it is not the raw name.
    pub skeleton: String, // data_class: TENANT_PAYLOAD
    /// FNV-1a digest of the raw candidate — the join key an operator uses
    /// to correlate refusals without the log holding the name itself. Not
    /// a cryptographic commitment; see [`fnv1a_64`].
    pub candidate_digest: u64, // data_class: TENANT_PAYLOAD
    /// The reservation entry that was hit, verbatim, when the decision is
    /// [`NamespaceDecision::DenyReserved`] or
    /// [`NamespaceDecision::DenyConfusable`]. Platform-owned, never tenant
    /// data.
    pub matched_reserved: Option<String>, // data_class: INTERNAL_ONLY
    /// Which syntax rule was broken, when the decision is
    /// [`NamespaceDecision::DenyMalformed`].
    pub malformed_reason: Option<MalformedReason>, // data_class: INTERNAL_ONLY
    /// The tenant the claim was scoped to, echoed from
    /// [`NamespaceCandidate::tenant`]. Without it an investigator cannot
    /// attribute a refusal to the tenant that produced it, which is the
    /// whole point of §D.5's tenant-id requirement.
    pub tenant: Option<String>, // data_class: TENANT_SCOPED
    /// §D.4's `AuditCorrelationId`, echoed from
    /// [`NamespaceCandidate::correlation_id`], joining this refusal back to
    /// the originating request.
    pub correlation_id: Option<String>, // data_class: INTERNAL_ONLY
}

impl NamespaceEvaluation {
    /// The audit event name IP-017 §D.5 specifies for a refusal, or `None`
    /// for an allow (nothing is emitted on the happy path).
    #[must_use]
    pub fn refusal_event(&self) -> Option<&'static str> {
        if self.decision.is_allowed() {
            None
        } else {
            Some("oya.tenancy.reserved-namespace-create-refused")
        }
    }
}

/// Evaluate a candidate name and return the decision only.
///
/// This is the scaffold's entry point and its signature is unchanged. It
/// delegates to [`evaluate_detailed`]; use that one where an audit record
/// is being written.
///
/// # Errors
///
/// Propagates every [`NamespaceUsecaseError`] from [`evaluate_detailed`].
pub fn evaluate<S: ReservedNamespaceSource, A: NamespaceActionAuthorizer>(
    source: &S,
    authorizer: &A,
    candidate: &NamespaceCandidate,
) -> Result<NamespaceDecision, NamespaceUsecaseError> {
    evaluate_detailed(source, authorizer, candidate).map(|evaluation| evaluation.decision)
}

/// Evaluate a candidate name and return the full audit record.
///
/// The stages run in a fixed order, and the order is load-bearing:
///
/// 1. **Request well-formed.** No principal means no authorization question
///    can be asked. No tenant context, for an action that names an existing
///    tenant, means the question that COULD be asked is the wrong one — an
///    unscoped answer would let a grant held in one tenant mint a name in
///    every other, so the guard refuses to answer instead. Both are caller
///    bugs and neither is a tenant-visible policy outcome.
/// 2. **Syntax** ([`validate_syntax`]) — before the reservation list is
///    even read, so a malformed name costs no port call and its refusal
///    does not depend on policy state.
/// 3. **Reservation list read** through
///    [`ReservedNamespaceSource::try_reserved`]. A port failure surfaces as
///    an error, never as a denial: an operator must be able to tell an
///    outage from a policy hit.
/// 4. **Reserved** — any boundary prefix of the candidate equals a reserved
///    token.
/// 5. **Confusable** — the same comparison on ASCII skeletons. Runs after
///    reservation so that a true collision is never mislabeled as a
///    look-alike.
/// 6. **Authorization** through [`NamespaceActionAuthorizer`], which
///    receives the whole candidate including its tenant context. Runs LAST
///    so that a name every principal would be refused is reported as
///    refused for the name, and so an unprivileged caller cannot use this
///    guard to probe the reservation list any further than a privileged one
///    can.
///
/// # Errors
///
/// - [`NamespaceUsecaseError::PrincipalMissing`] — the candidate carried no
///   principal.
/// - [`NamespaceUsecaseError::TenantContextMissing`] — the candidate carried
///   no tenant context for an action that requires one.
/// - [`NamespaceUsecaseError::SourceUnavailable`] — the reservation source
///   could not be read.
/// - [`NamespaceUsecaseError::EmptyReservationList`] /
///   [`NamespaceUsecaseError::MalformedReservationEntry`] — the source
///   answered, but with a list the guard refuses to trust.
/// - [`NamespaceUsecaseError::CedarEvaluationFailed`] — the authorizer
///   reached no verdict.
pub fn evaluate_detailed<S: ReservedNamespaceSource, A: NamespaceActionAuthorizer>(
    source: &S,
    authorizer: &A,
    candidate: &NamespaceCandidate,
) -> Result<NamespaceEvaluation, NamespaceUsecaseError> {
    let record = |decision: NamespaceDecision,
                  matched_reserved: Option<String>,
                  malformed_reason: Option<MalformedReason>| {
        NamespaceEvaluation {
            decision,
            skeleton: skeleton(&candidate.candidate),
            candidate_digest: fnv1a_64(&candidate.candidate),
            matched_reserved,
            malformed_reason,
            tenant: candidate.tenant_context().map(str::to_owned),
            correlation_id: candidate.correlation_id.clone(),
        }
    };

    if candidate.principal.trim().is_empty() {
        return Err(NamespaceUsecaseError::PrincipalMissing);
    }

    if candidate.action.requires_tenant_context() && candidate.tenant_context().is_none() {
        return Err(NamespaceUsecaseError::TenantContextMissing {
            action: candidate.action,
        });
    }

    if let Err(reason) = validate_syntax(&candidate.candidate, candidate.action) {
        return Ok(record(NamespaceDecision::DenyMalformed, None, Some(reason)));
    }

    let reserved = ReservedSet::build(&source.try_reserved()?)?;

    if let Some(entry) = reserved.reserved_hit(&candidate.candidate) {
        return Ok(record(
            NamespaceDecision::DenyReserved,
            Some(entry.to_owned()),
            None,
        ));
    }

    if let Some(entry) = reserved.confusable_hit(&candidate.candidate) {
        return Ok(record(
            NamespaceDecision::DenyConfusable,
            Some(entry.to_owned()),
            None,
        ));
    }

    if authorizer.authorize(candidate)? {
        Ok(record(NamespaceDecision::Allow, None, None))
    } else {
        Ok(record(NamespaceDecision::DenyUnauthorized, None, None))
    }
}
