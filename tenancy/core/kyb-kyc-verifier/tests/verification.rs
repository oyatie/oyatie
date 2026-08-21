//! Decision rules, jurisdiction-scoped requirement satisfaction, screening
//! coverage, the transition table, expiry boundaries, redaction, and
//! provider-failure behavior.
//!
//! Every fixture here is obviously synthetic: subjects are opaque `subject-*`
//! handles, evidence is an `evidence://fixture/...` URI, and screening
//! narratives say only that they are fixtures. Nothing in this file asserts on
//! anything shaped like real personal data.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use tenancy_kyb_kyc_verifier::{
    Assessment, AssessmentReason, DocumentRequirement, DocumentStatus, DocumentSubmission,
    InMemoryScreeningProvider, REDACTED, ScreeningCheck, ScreeningError, ScreeningPort,
    ScreeningRequest, ScreeningResolution, ScreeningResult, Timestamp, TransitionError,
    ValidityWindow, VerificationCase, VerificationCaseId, VerificationDecision, VerificationError,
    VerificationKind, advance_at, apply_verdict, assess_at, assess_with_screening, decide,
    evaluate_at, legal_transitions, refresh_screenings, settle_with_screening,
};

const OPENED_AT: i64 = 1_000;
const EXPIRES_AT: i64 = 2_000;
const NOW: i64 = 1_500;
const LONG_AFTER: i64 = 9_000_000;
const TENANT: &str = "tenant-fixture-a";
const OTHER_TENANT: &str = "tenant-fixture-b";
const SUBJECT: &str = "subject-ref-fixture-0001";
const PROVIDER: &str = "fixture-screening-provider";
const FIXTURE_NARRATIVE: &str = "SYNTHETIC FIXTURE: narrative withheld";
const EVIDENCE_REF: &str = "evidence://fixture/synthetic-0001";

fn window() -> ValidityWindow {
    ValidityWindow::new(Timestamp::new(OPENED_AT), Timestamp::new(EXPIRES_AT))
        .expect("fixture window is strictly positive in length")
}

fn requirement(name: &str, mandatory: bool, jurisdiction: &str) -> DocumentRequirement {
    DocumentRequirement::new(name.to_owned(), mandatory, jurisdiction.to_owned())
        .expect("fixture requirement is valid")
}

fn submission(name: &str, jurisdiction: &str, status: DocumentStatus) -> DocumentSubmission {
    DocumentSubmission::new(
        name.to_owned(),
        jurisdiction.to_owned(),
        status,
        EVIDENCE_REF.to_owned(),
    )
    .expect("fixture submission is valid")
}

fn hit(resolution: ScreeningResolution) -> ScreeningResult {
    ScreeningResult::new(
        PROVIDER.to_owned(),
        true,
        FIXTURE_NARRATIVE.to_owned(),
        resolution,
    )
    .expect("fixture screening result is valid")
}

fn clear() -> ScreeningResult {
    ScreeningResult::clear(PROVIDER.to_owned()).expect("a clear result is valid")
}

fn case_with(requirements: Vec<DocumentRequirement>) -> VerificationCase {
    VerificationCase::new(
        TENANT.to_owned(),
        VerificationCaseId::new("case-fixture-0001".to_owned()).expect("fixture id is non-empty"),
        VerificationKind::Kyb,
        SUBJECT.to_owned(),
        "KR".to_owned(),
        window(),
        requirements,
    )
    .expect("fixture case is valid")
}

/// One mandatory KR obligation, discharged. Never screened.
fn unscreened_case() -> VerificationCase {
    let mut case = case_with(vec![requirement("business-registration", true, "KR")]);
    case.record_submission(submission(
        "business-registration",
        "KR",
        DocumentStatus::Verified,
    ))
    .expect("the submission answers a declared requirement");
    case
}

/// One mandatory KR obligation discharged AND the required sanctions question
/// answered clear: nothing outstanding.
fn clean_case() -> VerificationCase {
    let mut case = unscreened_case();
    case.record_screening(clear())
        .expect("recording a clear result is allowed");
    case
}

fn provider() -> InMemoryScreeningProvider {
    InMemoryScreeningProvider::new(PROVIDER.to_owned()).expect("fixture provider name is non-empty")
}

fn now() -> Timestamp {
    Timestamp::new(NOW)
}

fn has_reason(assessment: &Assessment, wanted: &AssessmentReason) -> bool {
    assessment.reasons.iter().any(|reason| reason == wanted)
}

/// A port that answers with exactly the rows it was handed — including none.
/// `InMemoryScreeningProvider` fails closed on an unknown subject, so a real
/// adapter's "provider has no record for this subject" shape needs its own
/// fixture.
struct ScriptedProvider {
    rows: Vec<ScreeningResult>,
}

impl ScreeningPort for ScriptedProvider {
    fn screen(&self, _request: &ScreeningRequest) -> Result<Vec<ScreeningResult>, ScreeningError> {
        Ok(self.rows.clone())
    }
}

#[test]
fn a_new_case_opens_pending_and_needs_at_least_one_requirement() {
    let case = case_with(vec![requirement("business-registration", true, "KR")]);
    assert_eq!(case.decision, VerificationDecision::Pending);

    let error = VerificationCase::new(
        TENANT.to_owned(),
        VerificationCaseId::new("case-fixture-0002".to_owned()).expect("fixture id is non-empty"),
        VerificationKind::Kyb,
        SUBJECT.to_owned(),
        "KR".to_owned(),
        window(),
        Vec::new(),
    )
    .expect_err("a case with no obligations would approve everything");
    assert_eq!(error, VerificationError::NoRequirements);
}

#[test]
fn a_case_is_keyed_by_its_tenant_and_can_refuse_a_cross_tenant_claim() {
    let case = clean_case();
    assert_eq!(case.tenant_id, TENANT);
    assert!(case.belongs_to(TENANT));
    assert!(
        case.belongs_to("  TENANT-FIXTURE-A "),
        "scope key is folded"
    );
    assert!(
        !case.belongs_to(OTHER_TENANT),
        "a guessable case id is not an authorization fact; the tenant is"
    );

    let error = VerificationCase::new(
        "   ".to_owned(),
        VerificationCaseId::new("case-fixture-0004".to_owned()).expect("fixture id is non-empty"),
        VerificationKind::Kyb,
        SUBJECT.to_owned(),
        "KR".to_owned(),
        window(),
        vec![requirement("business-registration", true, "KR")],
    )
    .expect_err("an unscoped case cannot be tenant-scoped by anything downstream");
    assert_eq!(error, VerificationError::EmptyTenantId);

    let request = ScreeningRequest::for_case(&case);
    assert_eq!(request.tenant_id, TENANT, "the scope travels with the ask");
}

#[test]
fn a_case_rejects_a_duplicate_requirement_key() {
    let error = VerificationCase::new(
        TENANT.to_owned(),
        VerificationCaseId::new("case-fixture-0003".to_owned()).expect("fixture id is non-empty"),
        VerificationKind::Kyb,
        SUBJECT.to_owned(),
        "KR".to_owned(),
        window(),
        vec![
            requirement("business-registration", true, "KR"),
            requirement("business-registration", false, "kr"),
        ],
    )
    .expect_err("the same obligation twice would be ambiguous about mandatoriness");
    assert!(matches!(
        error,
        VerificationError::DuplicateRequirement { .. }
    ));
}

#[test]
fn a_submission_answering_no_requirement_is_refused() {
    let mut case = case_with(vec![requirement("business-registration", true, "KR")]);
    let error = case
        .record_submission(submission(
            "beneficial-ownership-declaration",
            "KR",
            DocumentStatus::Verified,
        ))
        .expect_err("storing an unmatched document would look like progress");
    assert!(matches!(
        error,
        VerificationError::UnknownRequirement { .. }
    ));
    assert!(case.submissions.is_empty());
}

#[test]
fn a_document_verified_in_one_jurisdiction_does_not_discharge_another() {
    let mut case = case_with(vec![
        requirement("business-registration", true, "KR"),
        requirement("business-registration", true, "US"),
    ]);
    case.record_submission(submission(
        "business-registration",
        "kr",
        DocumentStatus::Verified,
    ))
    .expect("lowercase kr is the same jurisdiction as KR");

    let unmet = case.unmet_mandatory_requirements();
    assert_eq!(unmet.len(), 1, "only the US obligation is still open");
    assert_eq!(unmet[0].jurisdiction, "US");
    assert_eq!(evaluate_at(&case, now()), VerificationDecision::Pending);
}

#[test]
fn an_unmet_mandatory_requirement_holds_the_case_pending() {
    let case = case_with(vec![requirement("business-registration", true, "KR")]);
    let assessment = assess_at(&case, now());
    assert_eq!(assessment.decision, VerificationDecision::Pending);
    assert!(!assessment.permits_activation());
    assert_eq!(assessment.unmet_mandatory.len(), 1);
    assert!(
        assessment
            .reasons
            .iter()
            .any(|reason| matches!(reason, AssessmentReason::MandatoryDocumentMissing { .. }))
    );
}

#[test]
fn an_optional_requirement_left_open_still_approves() {
    let mut case = clean_case();
    case.requirements
        .push(requirement("adverse-media-summary", false, "KR"));
    assert_eq!(evaluate_at(&case, now()), VerificationDecision::Approved);
}

#[test]
fn a_submitted_rejected_or_expired_document_does_not_discharge_a_requirement() {
    for status in [
        DocumentStatus::Submitted,
        DocumentStatus::Rejected,
        DocumentStatus::Expired,
    ] {
        let mut case = case_with(vec![requirement("business-registration", true, "KR")]);
        case.record_submission(submission("business-registration", "KR", status))
            .expect("the submission answers a declared requirement");
        assert_eq!(
            evaluate_at(&case, now()),
            VerificationDecision::Pending,
            "status {status} must not satisfy a mandatory obligation"
        );
    }
}

// ---------------------------------------------------------------------------
// Screening coverage: "unasked" is not "clear".
// ---------------------------------------------------------------------------

#[test]
fn a_case_that_was_never_screened_cannot_be_approved() {
    let case = unscreened_case();
    assert!(case.unmet_mandatory_requirements().is_empty());

    let assessment = assess_at(&case, now());
    assert_eq!(
        assessment.decision,
        VerificationDecision::Pending,
        "IP-018 D4 requires sanctions CLEAR, which is not the same as sanctions unasked"
    );
    assert_eq!(
        assessment.missing_screenings,
        vec![ScreeningCheck::Sanctions]
    );
    assert!(has_reason(
        &assessment,
        &AssessmentReason::ScreeningCoverageMissing {
            check: ScreeningCheck::Sanctions,
        }
    ));
}

#[test]
fn approval_requires_every_mandatory_obligation_met_and_every_question_answered() {
    let mut case = unscreened_case();
    assert_eq!(evaluate_at(&case, now()), VerificationDecision::Pending);

    case.record_screening(clear())
        .expect("recording a clear result is allowed");
    assert_eq!(evaluate_at(&case, now()), VerificationDecision::Approved);
}

#[test]
fn a_provider_response_with_no_rows_leaves_the_coverage_gap_open() {
    let silent = ScriptedProvider { rows: Vec::new() };
    let mut case = unscreened_case();

    let recorded =
        refresh_screenings(&mut case, &silent).expect("an empty response is not an error");
    assert_eq!(recorded, 0);

    let assessment = assess_with_screening(&mut case, &silent, now());
    assert_eq!(
        assessment.decision,
        VerificationDecision::Pending,
        "a provider that answered nothing has not cleared anybody"
    );
    assert!(has_reason(
        &assessment,
        &AssessmentReason::ScreeningCoverageMissing {
            check: ScreeningCheck::Sanctions,
        }
    ));
}

#[test]
fn a_consumer_case_additionally_requires_the_minor_protection_answer() {
    let mut case = VerificationCase::new(
        TENANT.to_owned(),
        VerificationCaseId::new("case-fixture-kyc".to_owned()).expect("fixture id is non-empty"),
        VerificationKind::Kyc,
        SUBJECT.to_owned(),
        "KR".to_owned(),
        window(),
        vec![requirement("identity-document", true, "KR")],
    )
    .expect("fixture case is valid");
    case.record_submission(submission(
        "identity-document",
        "KR",
        DocumentStatus::Verified,
    ))
    .expect("the submission answers a declared requirement");
    case.record_screening(clear())
        .expect("recording a clear sanctions result is allowed");

    assert_eq!(
        evaluate_at(&case, now()),
        VerificationDecision::Pending,
        "a consumer case with no minor-protection answer cannot approve"
    );
    case.record_screening(
        ScreeningResult::clear_for_check(PROVIDER.to_owned(), ScreeningCheck::MinorProtection)
            .expect("a clear result is valid"),
    )
    .expect("recording a clear result is allowed");
    assert_eq!(evaluate_at(&case, now()), VerificationDecision::Approved);
}

#[test]
fn a_case_may_declare_that_it_needs_no_screening_at_all() {
    let case = unscreened_case().requiring_screenings(Vec::new());
    assert_eq!(
        evaluate_at(&case, now()),
        VerificationDecision::Approved,
        "an explicitly empty required set is a deliberate declaration, never a default"
    );
}

// ---------------------------------------------------------------------------
// Screening answers: one provider, several questions.
// ---------------------------------------------------------------------------

#[test]
fn one_provider_answering_several_questions_keeps_every_answer() {
    let multi = provider()
        .with_hit(SUBJECT, ScreeningResolution::Unresolved, FIXTURE_NARRATIVE)
        .expect("fixture hit is valid")
        .with_clear_for_check(SUBJECT, ScreeningCheck::Pep)
        .expect("fixture clear is valid");
    let mut case = unscreened_case();

    let recorded = refresh_screenings(&mut case, &multi).expect("the fixture answers this subject");
    assert_eq!(recorded, 2, "two questions, two stored answers");
    assert_eq!(case.screenings.len(), 2);
    assert_eq!(
        evaluate_at(&case, now()),
        VerificationDecision::EscalatedToHuman,
        "a PEP clearance must not overwrite the same vendor's sanctions hit"
    );
}

#[test]
fn a_multi_answer_response_is_order_independent() {
    let forward = provider()
        .with_hit(SUBJECT, ScreeningResolution::Unresolved, FIXTURE_NARRATIVE)
        .expect("fixture hit is valid")
        .with_clear_for_check(SUBJECT, ScreeningCheck::Pep)
        .expect("fixture clear is valid");
    let reversed = provider()
        .with_clear_for_check(SUBJECT, ScreeningCheck::Pep)
        .expect("fixture clear is valid")
        .with_hit(SUBJECT, ScreeningResolution::Unresolved, FIXTURE_NARRATIVE)
        .expect("fixture hit is valid");

    let mut first = unscreened_case();
    let mut second = unscreened_case();
    refresh_screenings(&mut first, &forward).expect("the fixture answers this subject");
    refresh_screenings(&mut second, &reversed).expect("the fixture answers this subject");

    assert_eq!(
        evaluate_at(&first, now()),
        evaluate_at(&second, now()),
        "a verdict must not depend on the order the port returned its rows in"
    );
    assert_eq!(
        evaluate_at(&first, now()),
        VerificationDecision::EscalatedToHuman
    );
}

#[test]
fn two_answers_to_one_question_in_one_response_reduce_to_the_more_adverse() {
    for rows in [
        vec![hit(ScreeningResolution::Unresolved), clear()],
        vec![clear(), hit(ScreeningResolution::Unresolved)],
    ] {
        let scripted = ScriptedProvider { rows };
        let mut case = unscreened_case();

        let recorded =
            refresh_screenings(&mut case, &scripted).expect("the fixture answers this subject");
        assert_eq!(
            recorded, 1,
            "the count reports rows STORED, not rows the port returned"
        );
        assert_eq!(case.screenings.len(), 1);
        assert!(
            case.screenings[0].hit,
            "a vendor response must never cancel out its own hit"
        );
        assert_eq!(
            evaluate_at(&case, now()),
            VerificationDecision::EscalatedToHuman
        );
    }
}

#[test]
fn recording_a_provider_answer_replaces_that_providers_previous_answer() {
    let mut case = clean_case();
    case.record_screening(hit(ScreeningResolution::Unresolved))
        .expect("recording a hit is allowed");
    assert_eq!(case.screenings.len(), 1, "one question, one answer on file");
    assert_eq!(
        evaluate_at(&case, now()),
        VerificationDecision::EscalatedToHuman
    );

    case.record_screening(hit(ScreeningResolution::ClearedByReviewer))
        .expect("a reviewer resolution supersedes the raw hit");
    assert_eq!(case.screenings.len(), 1, "a stale hit must not outlive it");
    assert_eq!(evaluate_at(&case, now()), VerificationDecision::Approved);
}

#[test]
fn a_reviewers_clearance_supersedes_a_hit_spelled_with_different_capitalization() {
    let mut case = unscreened_case();
    case.record_screening(
        ScreeningResult::new(
            "Acuris".to_owned(),
            true,
            FIXTURE_NARRATIVE.to_owned(),
            ScreeningResolution::Unresolved,
        )
        .expect("fixture result is valid"),
    )
    .expect("recording a hit is allowed");
    assert_eq!(
        evaluate_at(&case, now()),
        VerificationDecision::EscalatedToHuman
    );

    case.record_screening(
        ScreeningResult::new(
            "acuris".to_owned(),
            true,
            FIXTURE_NARRATIVE.to_owned(),
            ScreeningResolution::ClearedByReviewer,
        )
        .expect("fixture result is valid"),
    )
    .expect("recording a resolution is allowed");

    assert_eq!(
        case.screenings.len(),
        1,
        "one provider spelled two ways is still one provider"
    );
    assert_eq!(
        evaluate_at(&case, now()),
        VerificationDecision::Approved,
        "a clearance filed under another spelling must still take effect"
    );
}

#[test]
fn recording_refuses_a_result_that_the_constructors_would_have_refused() {
    let mut case = unscreened_case();
    let bypassed = ScreeningResult {
        provider: PROVIDER.to_owned(),
        check: ScreeningCheck::Sanctions,
        hit: false,
        details: String::new(),
        resolution: ScreeningResolution::ClearedByReviewer,
    };
    let error = case
        .record_screening(bypassed)
        .expect_err("a 'cleared' flag on a no-hit row is adjudication that never happened");
    assert_eq!(
        error,
        VerificationError::ResolutionWithoutHit {
            resolution: ScreeningResolution::ClearedByReviewer,
        }
    );
    assert!(case.screenings.is_empty());
}

// ---------------------------------------------------------------------------
// Escalation and refusal.
// ---------------------------------------------------------------------------

#[test]
fn an_unadjudicated_hit_escalates_and_never_auto_rejects() {
    let mut case = clean_case();
    case.record_screening(hit(ScreeningResolution::Unresolved))
        .expect("recording a hit is allowed");

    let assessment = assess_at(&case, now());
    assert_eq!(
        assessment.decision,
        VerificationDecision::EscalatedToHuman,
        "a machine must not refuse a tenant on an unadjudicated name match"
    );
    assert!(
        assessment
            .reasons
            .iter()
            .any(|reason| matches!(reason, AssessmentReason::UnresolvedScreeningHit { .. }))
    );
    assert!(
        !assessment
            .reasons
            .iter()
            .any(|reason| reason.to_string().contains(FIXTURE_NARRATIVE)),
        "a decision reason must never carry the provider narrative"
    );
}

#[test]
fn escalation_outranks_a_missing_document_but_reports_both() {
    let mut case = case_with(vec![requirement("business-registration", true, "KR")]);
    case.record_screening(hit(ScreeningResolution::Unresolved))
        .expect("recording a hit is allowed");

    let assessment = assess_at(&case, now());
    assert_eq!(
        assessment.decision,
        VerificationDecision::EscalatedToHuman,
        "a live hit must not be buried under 'still waiting on paperwork'"
    );
    assert!(
        assessment
            .reasons
            .iter()
            .any(|reason| matches!(reason, AssessmentReason::UnresolvedScreeningHit { .. }))
    );
    assert!(
        assessment
            .reasons
            .iter()
            .any(|reason| matches!(reason, AssessmentReason::MandatoryDocumentMissing { .. }))
    );
}

#[test]
fn a_reviewer_cleared_hit_stops_blocking_approval() {
    let mut case = clean_case();
    case.record_screening(hit(ScreeningResolution::ClearedByReviewer))
        .expect("recording a cleared hit is allowed");
    assert_eq!(evaluate_at(&case, now()), VerificationDecision::Approved);
}

#[test]
fn a_reviewer_confirmed_hit_is_recorded_as_a_rejection() {
    let mut case = clean_case();
    case.record_screening(hit(ScreeningResolution::ConfirmedByReviewer))
        .expect("recording a confirmed hit is allowed");

    let assessment = assess_at(&case, now());
    assert_eq!(assessment.decision, VerificationDecision::Rejected);
    assert!(matches!(
        assessment.reasons.as_slice(),
        [AssessmentReason::HumanConfirmedHit { .. }]
    ));
}

// ---------------------------------------------------------------------------
// Expiry.
// ---------------------------------------------------------------------------

#[test]
fn a_lapsed_window_expires_the_case_whatever_else_it_holds() {
    let mut escalating = clean_case();
    escalating
        .record_screening(hit(ScreeningResolution::Unresolved))
        .expect("recording a hit is allowed");

    let pending = case_with(vec![requirement("business-registration", true, "KR")]);
    let approved = clean_case();

    for case in [&escalating, &pending, &approved] {
        assert_eq!(
            evaluate_at(case, Timestamp::new(EXPIRES_AT)),
            VerificationDecision::Expired
        );
        assert_eq!(
            evaluate_at(case, Timestamp::new(EXPIRES_AT - 1)),
            evaluate_at(case, now()),
            "one second before expiry nothing has changed yet"
        );
    }
}

#[test]
fn a_confirmed_refusal_outranks_a_lapsed_window() {
    let mut case = clean_case();
    case.record_screening(hit(ScreeningResolution::ConfirmedByReviewer))
        .expect("recording a confirmed hit is allowed");

    let assessment = advance_at(&mut case, Timestamp::new(EXPIRES_AT + 10))
        .expect("pending -> rejected is legal");
    assert_eq!(
        assessment.decision,
        VerificationDecision::Rejected,
        "an adverse determination is a finding, not a paperwork lapse"
    );
    assert_eq!(case.decision, VerificationDecision::Rejected);
}

#[test]
fn decide_never_activates_on_evidence_it_cannot_date() {
    let fresh = clean_case();
    assert_eq!(
        evaluate_at(&fresh, now()),
        VerificationDecision::Approved,
        "the clock-carrying path can approve"
    );
    assert_eq!(
        decide(&fresh),
        VerificationDecision::Pending,
        "a gate with no clock cannot verify freshness, so it must not activate"
    );
    assert!(!decide(&fresh).permits_activation());

    let stale = clean_case();
    assert_eq!(
        evaluate_at(&stale, Timestamp::new(LONG_AFTER)),
        VerificationDecision::Expired
    );
    assert!(
        !decide(&stale).permits_activation(),
        "IP-018 D6: stale KYB evidence must never activate a tenant"
    );
}

#[test]
fn decide_still_answers_every_question_that_does_not_need_a_clock() {
    assert_eq!(
        decide(&case_with(vec![requirement(
            "business-registration",
            true,
            "KR"
        )])),
        VerificationDecision::Pending
    );

    let mut refused = clean_case();
    refused
        .record_screening(hit(ScreeningResolution::ConfirmedByReviewer))
        .expect("recording a confirmed hit is allowed");
    assert_eq!(decide(&refused), VerificationDecision::Rejected);

    let mut escalated = clean_case();
    escalated
        .record_screening(hit(ScreeningResolution::Unresolved))
        .expect("recording a hit is allowed");
    assert_eq!(decide(&escalated), VerificationDecision::EscalatedToHuman);

    let mut stored_expired = clean_case();
    stored_expired
        .transition_to(VerificationDecision::Expired)
        .expect("pending -> expired is legal");
    assert_eq!(
        decide(&stored_expired),
        VerificationDecision::Expired,
        "a stored terminal state is reported faithfully without a clock"
    );
}

// ---------------------------------------------------------------------------
// The state machine and the verdict writer.
// ---------------------------------------------------------------------------

#[test]
fn every_legal_transition_is_accepted() {
    let states = [
        VerificationDecision::Pending,
        VerificationDecision::Approved,
        VerificationDecision::Rejected,
        VerificationDecision::EscalatedToHuman,
        VerificationDecision::Expired,
    ];
    for from in states {
        for to in legal_transitions(from) {
            let mut case = clean_case();
            case.decision = from;
            case.transition_to(*to).unwrap_or_else(|error| {
                panic!("{from} -> {to} is tabled legal but failed: {error}")
            });
            assert_eq!(case.decision, *to);
        }
    }
}

#[test]
fn a_rejected_case_is_terminal() {
    let mut case = clean_case();
    case.decision = VerificationDecision::Rejected;
    let error = case
        .transition_to(VerificationDecision::Approved)
        .expect_err("a refusal is never silently reversed; fresh evidence opens a new case");
    assert_eq!(
        error,
        TransitionError::TerminalState {
            state: VerificationDecision::Rejected,
            attempted: VerificationDecision::Approved,
        }
    );
    assert_eq!(case.decision, VerificationDecision::Rejected);
}

#[test]
fn an_expired_case_is_terminal() {
    let mut case = clean_case();
    case.decision = VerificationDecision::Expired;
    let error = case
        .transition_to(VerificationDecision::Pending)
        .expect_err("stale evidence is not revived by re-opening the same case");
    assert_eq!(
        error,
        TransitionError::TerminalState {
            state: VerificationDecision::Expired,
            attempted: VerificationDecision::Pending,
        }
    );
}

#[test]
fn an_approval_cannot_drop_straight_to_a_rejection() {
    let mut case = clean_case();
    case.decision = VerificationDecision::Approved;
    let error = case
        .transition_to(VerificationDecision::Rejected)
        .expect_err("withdrawing an approval must pass through a human review");
    assert_eq!(
        error,
        TransitionError::IllegalTransition {
            from: VerificationDecision::Approved,
            to: VerificationDecision::Rejected,
        }
    );

    case.transition_to(VerificationDecision::EscalatedToHuman)
        .expect("approved -> escalated is the documented route");
    case.transition_to(VerificationDecision::Rejected)
        .expect("escalated -> rejected is legal once a human is involved");
}

#[test]
fn no_state_ever_returns_to_pending() {
    for from in [
        VerificationDecision::Approved,
        VerificationDecision::EscalatedToHuman,
    ] {
        let mut case = clean_case();
        case.decision = from;
        let error = case
            .transition_to(VerificationDecision::Pending)
            .expect_err("'not yet looked at' is false once a case has been decided");
        assert_eq!(
            error,
            TransitionError::IllegalTransition {
                from,
                to: VerificationDecision::Pending,
            }
        );
    }
}

#[test]
fn an_approved_case_with_a_confirmed_hit_is_revoked_through_the_review_detour() {
    let mut case = clean_case();
    advance_at(&mut case, now()).expect("pending -> approved is legal");
    assert_eq!(case.decision, VerificationDecision::Approved);

    case.record_screening(hit(ScreeningResolution::ConfirmedByReviewer))
        .expect("recording a confirmed hit is allowed");

    let assessment = advance_at(&mut case, now())
        .expect("a confirmed match must be able to revoke an approval, not dead-end");
    assert_eq!(assessment.decision, VerificationDecision::Rejected);
    assert_eq!(case.decision, VerificationDecision::Rejected);
    assert!(
        !case.decision.permits_activation(),
        "a sanctioned tenant must stop being activation-eligible"
    );
}

#[test]
fn revocation_also_works_through_the_settle_path() {
    let screening = provider()
        .with_hit(
            SUBJECT,
            ScreeningResolution::ConfirmedByReviewer,
            FIXTURE_NARRATIVE,
        )
        .expect("fixture hit is valid");
    let mut case = clean_case();
    case.decision = VerificationDecision::Approved;

    let assessment = settle_with_screening(&mut case, &screening, now())
        .expect("the nightly settle must not return the same error forever");
    assert_eq!(assessment.decision, VerificationDecision::Rejected);
    assert_eq!(case.decision, VerificationDecision::Rejected);
}

#[test]
fn a_pending_verdict_holds_a_case_that_has_already_moved_on() {
    let mut case = case_with(vec![
        requirement("business-registration", true, "KR"),
        requirement("beneficial-ownership-declaration", true, "KR"),
    ]);
    case.record_submission(submission(
        "business-registration",
        "KR",
        DocumentStatus::Verified,
    ))
    .expect("the submission answers a declared requirement");
    case.record_screening(hit(ScreeningResolution::Unresolved))
        .expect("recording a hit is allowed");

    advance_at(&mut case, now()).expect("pending -> escalated is legal");
    assert_eq!(case.decision, VerificationDecision::EscalatedToHuman);

    case.record_screening(hit(ScreeningResolution::ClearedByReviewer))
        .expect("a reviewer resolution supersedes the raw hit");

    let assessment = advance_at(&mut case, now())
        .expect("a cleared hit with paperwork still open is routine, not a fault");
    assert_eq!(
        assessment.decision,
        VerificationDecision::EscalatedToHuman,
        "the case is HELD, and the assessment reports where it actually stands"
    );
    assert!(has_reason(
        &assessment,
        &AssessmentReason::VerdictHeld {
            verdict: VerificationDecision::Pending,
            state: VerificationDecision::EscalatedToHuman,
        }
    ));
    assert!(
        assessment
            .reasons
            .iter()
            .any(|reason| matches!(reason, AssessmentReason::MandatoryDocumentMissing { .. }))
    );

    case.record_submission(submission(
        "beneficial-ownership-declaration",
        "KR",
        DocumentStatus::Verified,
    ))
    .expect("the submission answers a declared requirement");
    let assessment = advance_at(&mut case, now()).expect("escalated -> approved is legal");
    assert_eq!(assessment.decision, VerificationDecision::Approved);
}

#[test]
fn advance_never_dead_ends_on_a_reachable_state() {
    for from in [
        VerificationDecision::Pending,
        VerificationDecision::Approved,
        VerificationDecision::EscalatedToHuman,
        VerificationDecision::Rejected,
        VerificationDecision::Expired,
    ] {
        for resolution in [
            ScreeningResolution::Unresolved,
            ScreeningResolution::ClearedByReviewer,
            ScreeningResolution::ConfirmedByReviewer,
        ] {
            let mut case = clean_case();
            case.decision = from;
            case.record_screening(hit(resolution))
                .expect("recording a hit is allowed");
            let assessment = advance_at(&mut case, now()).unwrap_or_else(|error| {
                panic!("{from} with a {resolution} hit dead-ended: {error}")
            });
            assert_eq!(
                assessment.decision, case.decision,
                "the reported verdict must be the state the case now holds"
            );
        }
    }
}

#[test]
fn apply_verdict_still_refuses_to_resurrect_a_closed_case() {
    let mut case = clean_case();
    case.decision = VerificationDecision::Rejected;
    let error = apply_verdict(&mut case, VerificationDecision::Approved)
        .expect_err("a closed case absorbs; fresh evidence opens a new one");
    assert_eq!(
        error,
        TransitionError::TerminalState {
            state: VerificationDecision::Rejected,
            attempted: VerificationDecision::Approved,
        }
    );
    assert_eq!(case.decision, VerificationDecision::Rejected);
}

#[test]
fn advance_writes_the_verdict_and_is_a_no_op_on_a_closed_case() {
    let mut case = clean_case();
    let assessment = advance_at(&mut case, now()).expect("pending -> approved is legal");
    assert_eq!(assessment.decision, VerificationDecision::Approved);
    assert_eq!(case.decision, VerificationDecision::Approved);

    let mut closed = clean_case();
    closed.decision = VerificationDecision::Rejected;
    let assessment = advance_at(&mut closed, now()).expect("a closed case absorbs re-assessment");
    assert_eq!(assessment.decision, VerificationDecision::Rejected);
    assert!(matches!(
        assessment.reasons.as_slice(),
        [AssessmentReason::AlreadyTerminal { .. }]
    ));
}

// ---------------------------------------------------------------------------
// Provider failures.
// ---------------------------------------------------------------------------

#[test]
fn a_provider_outage_holds_an_otherwise_clean_case_pending() {
    let screening = provider().with_outage(SUBJECT);
    let mut case = clean_case();

    let assessment = assess_with_screening(&mut case, &screening, now());
    assert_eq!(
        assessment.decision,
        VerificationDecision::Pending,
        "an incomplete screening picture can never GRANT an approval"
    );
    assert!(has_reason(
        &assessment,
        &AssessmentReason::ScreeningProviderUnavailable {
            provider: PROVIDER.to_owned(),
        }
    ));
}

#[test]
fn each_screening_failure_keeps_its_own_reason() {
    let outage = provider().with_outage(SUBJECT);
    let timeout = provider().with_timeout(SUBJECT);
    let wrong_kind = provider()
        .supporting(vec![VerificationKind::Kyc])
        .with_clear(SUBJECT)
        .expect("fixture clear is valid");
    let unusable = ScriptedProvider {
        rows: vec![ScreeningResult {
            provider: PROVIDER.to_owned(),
            check: ScreeningCheck::Sanctions,
            hit: false,
            details: String::new(),
            resolution: ScreeningResolution::ConfirmedByReviewer,
        }],
    };

    let mut case = clean_case();
    assert!(has_reason(
        &assess_with_screening(&mut case.clone(), &outage, now()),
        &AssessmentReason::ScreeningProviderUnavailable {
            provider: PROVIDER.to_owned(),
        }
    ));
    assert!(has_reason(
        &assess_with_screening(&mut case.clone(), &timeout, now()),
        &AssessmentReason::ScreeningProviderTimedOut {
            provider: PROVIDER.to_owned(),
        }
    ));
    assert!(
        has_reason(
            &assess_with_screening(&mut case.clone(), &wrong_kind, now()),
            &AssessmentReason::ScreeningProviderCannotScreen {
                provider: PROVIDER.to_owned(),
                kind: VerificationKind::Kyb,
            }
        ),
        "the wrong vendor bound to this kind is a wiring fault, not an outage"
    );
    assert!(has_reason(
        &assess_with_screening(&mut case, &unusable, now()),
        &AssessmentReason::ScreeningResultUnusable {
            provider: PROVIDER.to_owned(),
        }
    ));
}

#[test]
fn a_vendor_outage_on_an_approved_case_holds_it_instead_of_erroring() {
    let screening = provider().with_outage(SUBJECT);
    let mut case = clean_case();
    case.decision = VerificationDecision::Approved;

    let assessment = settle_with_screening(&mut case, &screening, now())
        .expect("a transient outage is a routine condition, not a programming fault");
    assert_eq!(case.decision, VerificationDecision::Approved);
    assert_eq!(assessment.decision, VerificationDecision::Approved);
    assert!(has_reason(
        &assessment,
        &AssessmentReason::ScreeningProviderUnavailable {
            provider: PROVIDER.to_owned(),
        }
    ));
    assert!(has_reason(
        &assessment,
        &AssessmentReason::VerdictHeld {
            verdict: VerificationDecision::Pending,
            state: VerificationDecision::Approved,
        }
    ));
}

#[test]
fn a_provider_timeout_never_manufactures_a_refusal() {
    let screening = provider().with_timeout(SUBJECT);
    let mut case = clean_case();

    let assessment = settle_with_screening(&mut case, &screening, now())
        .expect("pending -> pending is a legal no-op");
    assert_eq!(assessment.decision, VerificationDecision::Pending);
    assert_eq!(case.decision, VerificationDecision::Pending);
    assert!(has_reason(
        &assessment,
        &AssessmentReason::ScreeningProviderTimedOut {
            provider: PROVIDER.to_owned(),
        }
    ));
}

#[test]
fn an_unregistered_subject_fails_closed_rather_than_reading_as_clear() {
    let screening = provider()
        .with_clear("subject-ref-fixture-9999")
        .expect("fixture clear is valid");
    let mut case = clean_case();

    let error = refresh_screenings(&mut case, &screening)
        .expect_err("an unanswered question must not read as 'clear'");
    assert_eq!(
        error,
        ScreeningError::ProviderUnavailable {
            provider: PROVIDER.to_owned(),
        }
    );
}

#[test]
fn a_provider_that_does_not_screen_this_kind_reports_it() {
    let screening = provider()
        .supporting(vec![VerificationKind::Kyc])
        .with_clear(SUBJECT)
        .expect("fixture clear is valid");
    let mut case = clean_case();

    let error = refresh_screenings(&mut case, &screening)
        .expect_err("a KYC-only provider cannot answer a KYB case");
    assert_eq!(
        error,
        ScreeningError::UnsupportedKind {
            provider: PROVIDER.to_owned(),
            kind: VerificationKind::Kyb,
        }
    );
}

#[test]
fn settling_a_clean_case_through_a_clear_provider_approves_it() {
    let screening = provider()
        .with_clear(SUBJECT)
        .expect("fixture clear is valid");
    let mut case = unscreened_case();

    let assessment =
        settle_with_screening(&mut case, &screening, now()).expect("pending -> approved is legal");
    assert_eq!(assessment.decision, VerificationDecision::Approved);
    assert_eq!(case.decision, VerificationDecision::Approved);
    assert_eq!(case.screenings.len(), 1);
}

#[test]
fn settling_a_case_the_provider_hits_escalates_it() {
    let screening = provider()
        .with_hit(SUBJECT, ScreeningResolution::Unresolved, FIXTURE_NARRATIVE)
        .expect("fixture hit is valid");
    let mut case = unscreened_case();

    let assessment =
        settle_with_screening(&mut case, &screening, now()).expect("pending -> escalated is legal");
    assert_eq!(assessment.decision, VerificationDecision::EscalatedToHuman);
    assert_eq!(case.decision, VerificationDecision::EscalatedToHuman);
}

#[test]
fn an_expired_case_never_calls_the_provider() {
    let screening = provider()
        .with_hit(SUBJECT, ScreeningResolution::Unresolved, FIXTURE_NARRATIVE)
        .expect("fixture hit is valid");
    let mut case = unscreened_case();

    let assessment = assess_with_screening(&mut case, &screening, Timestamp::new(EXPIRES_AT));
    assert_eq!(assessment.decision, VerificationDecision::Expired);
    assert!(
        case.screenings.is_empty(),
        "nothing a fresh screening could say would rescue lapsed evidence"
    );
}

// ---------------------------------------------------------------------------
// Redaction.
// ---------------------------------------------------------------------------

#[test]
fn debug_of_a_case_leaks_neither_the_subject_handle_nor_a_narrative() {
    let mut case = clean_case();
    case.record_screening(hit(ScreeningResolution::Unresolved))
        .expect("recording a hit is allowed");

    let rendered = format!("{case:?}");
    assert!(
        !rendered.contains(SUBJECT),
        "the subject handle is SECRET and must not reach a log line: {rendered}"
    );
    assert!(
        !rendered.contains(FIXTURE_NARRATIVE),
        "the provider narrative is SECRET and must not reach a log line: {rendered}"
    );
    assert!(
        !rendered.contains(EVIDENCE_REF),
        "the evidence handle is SECRET and must not reach a log line: {rendered}"
    );
    assert!(rendered.contains(REDACTED));
    assert!(
        rendered.contains(TENANT) && rendered.contains("case-fixture-0001"),
        "scope and identity stay visible so a log line is still useful"
    );
}

#[test]
fn debug_of_a_screening_request_redacts_the_subject_handle() {
    let request = ScreeningRequest::for_case(&clean_case());
    let rendered = format!("{request:?}");
    assert!(!rendered.contains(SUBJECT), "{rendered}");
    assert!(rendered.contains(REDACTED));
}
