//! KYB / KYC verifier domain — IP-018 (`tenancy` / `kyb-kyc`, domain layer).
//!
//! `tenancy` can model a tenant's lifecycle, but activation is only safe if
//! something first answers "is this tenant ALLOWED to become active?". This
//! crate is that answer: verification cases, jurisdiction-scoped document
//! obligations, screening results, an explicit decision state machine, and an
//! expiry rule — all pure, all reproducible from their inputs.
//!
//! # Shape
//!
//! IP-018 describes a family of crates (`case`, `document`, `screening`,
//! `rules`, `events`). The `tenancy` capability is capped at twelve crates and
//! the workspace lockfile is frozen, so that family is collapsed into ONE crate
//! as a module tree:
//!
//! * [`kernel`] — timeline, validity window, requirements, submissions,
//!   screening checks and results, and the construction errors over them.
//! * [`domain`] — the [`VerificationCase`] aggregate, the decision rules
//!   ([`assess_at`]), the transition table ([`legal_transitions`]) and the
//!   verdict writer ([`apply_verdict`]).
//! * [`usecase`] — the [`ScreeningPort`] seam and the settlement usecase.
//! * [`inmemory`] — a deterministic fixture implementation of that port.
//!
//! Everything is re-exported at the crate root, so the published names the
//! scaffold declared keep their original paths.
//!
//! # The rules that matter
//!
//! 1. **A machine never refuses a person on a name match.** An unadjudicated
//!    screening hit produces [`VerificationDecision::EscalatedToHuman`], never
//!    `Rejected`. `Rejected` is reachable from screening evidence only after a
//!    reviewer marks the hit [`ScreeningResolution::ConfirmedByReviewer`], at
//!    which point this domain is recording a human's call, not making one.
//! 2. **Nothing here reads a clock.** Expiry takes an explicit `now`
//!    ([`ValidityWindow::is_expired_at`]), so an expiry verdict is a pure
//!    function of its arguments and replays identically forever. A function
//!    that cannot see a clock is not allowed to grant activation — see
//!    [`decide`].
//! 3. **A lost answer is never symmetric.** One provider answers several
//!    questions; each answer is keyed by (provider, check) so a PEP clearance
//!    cannot overwrite the same vendor's sanctions hit, and two answers to one
//!    question inside a single response reduce to the MORE adverse one.
//! 4. **"Unasked" is not "clear".** A case must carry an answer to every
//!    screening question it requires ([`VerificationCase::required_screenings`])
//!    before it can be approved.
//! 5. **A confirmed refusal is always reachable.** [`apply_verdict`] walks the
//!    mandatory `Approved -> EscalatedToHuman -> Rejected` review detour rather
//!    than reporting a dead end, so an approved tenant with a confirmed
//!    sanctions match can actually be revoked.
//!
//! Personal data never enters this crate. A case carries a pseudonymous
//! `subject_ref`; the document CONTENT lives behind an opaque `evidence_ref`;
//! and a provider's match narrative stays in `ScreeningResult::details`. Those
//! three fields are classified SECRET, no decision reason ever quotes them, and
//! every type holding one renders a REDACTING [`core::fmt::Debug`] so an
//! adapter's `tracing::error!(case = ?case, ..)` cannot leak them either.
//!
//! # Gaps
//!
//! Deliberately deferred, and honest about it:
//!
//! * **No provider adapter.** [`ScreeningPort`] is the seam; a real sanctions /
//!   PEP / adverse-media integration is network I/O and belongs in an adapter
//!   crate. Only [`InMemoryScreeningProvider`] ships here.
//! * **The port is synchronous.** Real providers are async. Adding `async` (or
//!   a boxed-future port, as the tenant-lifecycle usecase does) would add a
//!   runtime dependency, and this lane holds no lockfile waiver. The decision
//!   rules are runtime-agnostic, so the seam can be made async without touching
//!   them.
//! * **No persistence and no event bus.** IP-018 §D3 wants domain events
//!   (`oya.tenancy.kyb-kyc-{completed,declined,escalated}`) appended as facts
//!   are recorded. [`Assessment`] carries the same information as a returned
//!   value; publishing it is the adapter's job and is not implemented.
//! * **No jurisdiction rule catalog.** IP-018 §D2 wants
//!   `required_documents(country, tenant_class, capability)` to MATERIALIZE the
//!   KR-PASS / EU-UBO / US-BAA / COPPA obligations. This crate evaluates
//!   whatever requirement set it is handed, per jurisdiction, but does not ship
//!   the per-country catalog that produces one — that catalog is policy data,
//!   and inventing it in code would be a compliance claim this lane cannot back.
//! * **Aggregate invariants are enforced at the constructors, not by the type
//!   system.** The fields of [`VerificationCase`] are `pub` (`decision` was
//!   already `pub` in the published scaffold), so a struct literal or a direct
//!   assignment can still express a state `new()` would refuse. Every path
//!   inside this crate goes through the validated constructors; a caller that
//!   writes `case.decision = ..` directly bypasses the state machine, and
//!   sealing that off means removing published fields, which this lane may not
//!   do. Tenant scoping has the same shape: [`VerificationCase::belongs_to`] is
//!   the guard, but nothing forces a repository to call it.
//! * **[`decide`] cannot see the clock, so it cannot grant activation.** Its
//!   scaffold signature takes no `now`. Freshness is exactly the thing it
//!   cannot verify, so it refuses to return `Approved` at all and reports
//!   `Pending` instead: an activation gate that cannot date its evidence must
//!   fail closed. Callers that can supply a clock reading use [`evaluate_at`],
//!   [`assess_at`] or [`advance_at`], all of which apply the expiry rule in
//!   full. The signature is preserved because it is published contract.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod domain;
pub mod inmemory;
pub mod kernel;
pub mod usecase;

pub use domain::{
    Assessment, AssessmentReason, TransitionError, VerificationCase, VerificationCaseId,
    advance_at, apply_verdict, assess_at, default_required_screenings, evaluate_at,
    is_legal_transition, legal_transitions,
};
pub use inmemory::InMemoryScreeningProvider;
pub use kernel::{
    DocumentRequirement, DocumentStatus, DocumentSubmission, REDACTED, RequirementKey,
    ScreeningCheck, ScreeningKey, ScreeningResolution, ScreeningResult, Timestamp, ValidityWindow,
    VerificationDecision, VerificationError, VerificationKind, jurisdiction_matches,
    normalized_jurisdiction, normalized_provider,
};
pub use usecase::{
    ScreeningError, ScreeningPort, ScreeningRequest, assess_with_screening, refresh_screenings,
    settle_with_screening,
};

/// Decide whether activation may proceed on a case, WITHOUT a clock.
///
/// Preserved IP-018 scaffold signature. Every rule this crate has except one is
/// a pure function of the stored facts; the exception is expiry, which is a
/// function of the facts AND the current instant. With no `now` to read, this
/// function cannot tell fresh evidence from evidence that lapsed years ago —
/// so it never grants activation. It applies the full rule set, and downgrades
/// a would-be `Approved` to `Pending`.
///
/// That makes it useful for the answers that do not depend on the clock — a
/// stored terminal state, a confirmed adverse hit, an open obligation, a live
/// screening hit — and safe for the one that does. IP-018 §D6 requires that
/// stale KYB evidence cannot activate a tenant; a gate that assumed freshness
/// in order to answer would be the exact failure mode that clause names.
///
/// Use [`evaluate_at`] (or [`assess_at`] / [`advance_at`]) wherever the caller
/// can supply a clock reading — those apply expiry and can approve.
#[must_use]
pub fn decide(case: &VerificationCase) -> VerificationDecision {
    let clock_free = evaluate_at(case, case.window.opened_at);
    if clock_free.permits_activation() {
        VerificationDecision::Pending
    } else {
        clock_free
    }
}
