#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod support;

use policy_authoring_app::{PolicyProject, QualificationError};
use shared_platform_contracts_kernel::pdp::Decision;
use support::*;

#[test]
fn real_cedar_cases_qualify_exact_content_and_obligations() {
    let prepared = project().prepare(ids()).unwrap();
    assert!(prepared.bundle().version.as_str().starts_with("sha256:"));
    assert_eq!(prepared.report().passed_cases, 2);
    assert_eq!(prepared.report().policy_version, prepared.bundle().version);
    assert_eq!(
        prepared.bundle().policies_src,
        project().source.policies_src
    );
}

#[test]
fn outcome_and_obligation_mismatches_refuse_qualification() {
    let mut candidate = project();
    candidate.cases[0].expected.decision = Decision::Deny;
    assert!(matches!(
        candidate.prepare(ids()),
        Err(QualificationError::CaseMismatch { .. })
    ));
    let mut candidate = project();
    candidate.cases[0].expected.obligations.clear();
    assert!(matches!(
        candidate.prepare(ids()),
        Err(QualificationError::CaseMismatch { .. })
    ));
}

#[test]
fn invalid_source_and_case_evaluation_are_distinct_refusals() {
    let mut candidate = project();
    candidate.source.policies_src = "not Cedar".into();
    assert!(matches!(
        candidate.prepare(ids()),
        Err(QualificationError::Admission(_))
    ));
    let mut candidate = project();
    candidate.cases[0].request.action = "unknown".into();
    assert!(matches!(
        candidate.prepare(ids()),
        Err(QualificationError::CaseRefused { .. })
    ));
}

#[test]
fn cases_require_nonempty_unique_names_and_at_least_one_case() {
    let mut candidate = project();
    candidate.cases.clear();
    assert!(matches!(
        candidate.prepare(ids()),
        Err(QualificationError::InvalidCases { .. })
    ));
    let mut candidate = project();
    candidate.cases[1].name = candidate.cases[0].name.clone();
    assert!(matches!(
        candidate.prepare(ids()),
        Err(QualificationError::InvalidCases { .. })
    ));
    let mut candidate = project();
    candidate.cases[0].name = " ".into();
    assert!(matches!(
        candidate.prepare(ids()),
        Err(QualificationError::InvalidCases { .. })
    ));
}

#[test]
fn project_wire_contract_rejects_unknown_fields() {
    let mut wire = serde_json::to_value(project()).unwrap();
    wire["skip_tests"] = true.into();
    assert!(serde_json::from_value::<PolicyProject>(wire).is_err());
    let mut wire = serde_json::to_value(project()).unwrap();
    wire["source"]["skip_validation"] = true.into();
    assert!(serde_json::from_value::<PolicyProject>(wire).is_err());
}
