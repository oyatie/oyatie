//! Intelligence eval kernel foundation.
//!
//! This crate scores metadata-only evaluation suites for later Intelligence
//! cloud integration. It models golden, adversarial, and linguistic coverage,
//! deterministic pass/violation rates, and fail-closed threshold checks without
//! model calls, grader calls, network, filesystem, durable storage, or raw
//! prompt/output text.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

const BASIS_POINTS_DENOMINATOR: u32 = 10_000;
const MAX_EVAL_CASES: usize = 10_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EvalSuiteStatus {
    Passed,
    Failed,
    Invalid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EvalCaseKind {
    Adversarial,
    Golden,
    Linguistic,
    Regression,
    Safety,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EvalCaseOutcome {
    Failed,
    Passed,
    SafetyViolation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EvalFailureKind {
    InvalidInput,
    MissingRequiredCoverage,
    SafetyViolationRateExceeded,
    ThresholdFailed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalCaseResult {
    pub case_id: String,                // data_class: INTERNAL_ONLY
    pub kind: EvalCaseKind,             // data_class: INTERNAL_ONLY
    pub outcome: EvalCaseOutcome,       // data_class: INTERNAL_ONLY
    pub score_bps: u16,                 // data_class: INTERNAL_ONLY
    pub evaluator_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalSuiteThresholds {
    pub min_pass_rate_bps: u32,             // data_class: INTERNAL_ONLY
    pub max_safety_violation_rate_bps: u32, // data_class: INTERNAL_ONLY
    pub require_golden: bool,               // data_class: INTERNAL_ONLY
    pub require_adversarial: bool,          // data_class: INTERNAL_ONLY
    pub require_linguistic: bool,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalSuite {
    pub suite_id: String,                // data_class: INTERNAL_ONLY
    pub model_ref: String,               // data_class: INTERNAL_ONLY
    pub route_evidence_ref: String,      // data_class: INTERNAL_ONLY
    pub guardrail_evidence_ref: String,  // data_class: INTERNAL_ONLY
    pub dataset_snapshot_ref: String,    // data_class: INTERNAL_ONLY
    pub thresholds: EvalSuiteThresholds, // data_class: INTERNAL_ONLY
    pub cases: Vec<EvalCaseResult>,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalKindSummary {
    pub kind: EvalCaseKind,     // data_class: INTERNAL_ONLY
    pub total: u32,             // data_class: INTERNAL_ONLY
    pub passed: u32,            // data_class: INTERNAL_ONLY
    pub failed: u32,            // data_class: INTERNAL_ONLY
    pub safety_violations: u32, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalSuiteReport {
    pub suite_id: String,                    // data_class: INTERNAL_ONLY
    pub model_ref: String,                   // data_class: INTERNAL_ONLY
    pub status: EvalSuiteStatus,             // data_class: PUBLIC
    pub failure_kinds: Vec<EvalFailureKind>, // data_class: INTERNAL_ONLY
    pub total_cases: u32,                    // data_class: INTERNAL_ONLY
    pub passed_cases: u32,                   // data_class: INTERNAL_ONLY
    pub failed_cases: u32,                   // data_class: INTERNAL_ONLY
    pub safety_violation_cases: u32,         // data_class: INTERNAL_ONLY
    pub pass_rate_bps: u32,                  // data_class: INTERNAL_ONLY
    pub safety_violation_rate_bps: u32,      // data_class: INTERNAL_ONLY
    pub summaries: Vec<EvalKindSummary>,     // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,          // data_class: INTERNAL_ONLY
}

pub fn evaluate_suite(suite: EvalSuite) -> EvalSuiteReport {
    let invalid = invalid_suite_reasons(&suite);
    if !invalid.is_empty() {
        return invalid_report(&suite);
    }

    let evidence_refs = suite_evidence_refs(&suite);
    let summaries = summarize_by_kind(&suite.cases);
    let total_cases = suite.cases.len() as u32;
    let passed_cases = suite
        .cases
        .iter()
        .filter(|case| case.outcome == EvalCaseOutcome::Passed)
        .count() as u32;
    let safety_violation_cases = suite
        .cases
        .iter()
        .filter(|case| case.outcome == EvalCaseOutcome::SafetyViolation)
        .count() as u32;
    let failed_cases = total_cases.saturating_sub(passed_cases + safety_violation_cases);
    let pass_rate_bps = ratio_bps(passed_cases, total_cases);
    let safety_violation_rate_bps = ratio_bps(safety_violation_cases, total_cases);

    let mut failure_kinds = Vec::new();
    if !required_coverage_present(&suite.thresholds, &summaries) {
        failure_kinds.push(EvalFailureKind::MissingRequiredCoverage);
    }
    if pass_rate_bps < suite.thresholds.min_pass_rate_bps {
        failure_kinds.push(EvalFailureKind::ThresholdFailed);
    }
    if safety_violation_rate_bps > suite.thresholds.max_safety_violation_rate_bps {
        failure_kinds.push(EvalFailureKind::SafetyViolationRateExceeded);
    }
    failure_kinds.sort();
    failure_kinds.dedup();

    EvalSuiteReport {
        suite_id: suite.suite_id,
        model_ref: suite.model_ref,
        status: if failure_kinds.is_empty() {
            EvalSuiteStatus::Passed
        } else {
            EvalSuiteStatus::Failed
        },
        failure_kinds,
        total_cases,
        passed_cases,
        failed_cases,
        safety_violation_cases,
        pass_rate_bps,
        safety_violation_rate_bps,
        summaries,
        evidence_refs,
    }
}

fn invalid_suite_reasons(suite: &EvalSuite) -> Vec<String> {
    let mut reasons = Vec::new();
    require_metadata_ref("suite id", &suite.suite_id, &mut reasons);
    require_resource_ref("model ref", &suite.model_ref, &mut reasons);
    require_evidence_ref(
        "route evidence ref",
        &suite.route_evidence_ref,
        &mut reasons,
    );
    require_evidence_ref(
        "guardrail evidence ref",
        &suite.guardrail_evidence_ref,
        &mut reasons,
    );
    require_evidence_ref(
        "dataset snapshot ref",
        &suite.dataset_snapshot_ref,
        &mut reasons,
    );
    if suite.cases.is_empty() {
        reasons.push("eval cases are required".to_owned());
    } else if suite.cases.len() > MAX_EVAL_CASES {
        reasons.push(format!("eval case count must be <= {MAX_EVAL_CASES}"));
    }
    if suite.thresholds.min_pass_rate_bps > BASIS_POINTS_DENOMINATOR {
        reasons.push("minimum pass rate must be 0..=10000 basis points".to_owned());
    }
    if suite.thresholds.max_safety_violation_rate_bps > BASIS_POINTS_DENOMINATOR {
        reasons.push("maximum safety violation rate must be 0..=10000 basis points".to_owned());
    }
    for case in &suite.cases {
        require_metadata_ref("case id", &case.case_id, &mut reasons);
        require_evidence_ref(
            "case evaluator evidence ref",
            &case.evaluator_evidence_ref,
            &mut reasons,
        );
        if u32::from(case.score_bps) > BASIS_POINTS_DENOMINATOR {
            reasons.push("case score must be 0..=10000 basis points".to_owned());
        }
    }
    sorted_unique(reasons)
}

fn invalid_report(suite: &EvalSuite) -> EvalSuiteReport {
    EvalSuiteReport {
        suite_id: safe_metadata(&suite.suite_id, "redacted-invalid-suite-id"),
        model_ref: safe_ref(&suite.model_ref, "redacted-invalid-model-ref"),
        status: EvalSuiteStatus::Invalid,
        failure_kinds: vec![EvalFailureKind::InvalidInput],
        total_cases: 0,
        passed_cases: 0,
        failed_cases: 0,
        safety_violation_cases: 0,
        pass_rate_bps: 0,
        safety_violation_rate_bps: 0,
        summaries: Vec::new(),
        evidence_refs: vec!["validation:intelligence-eval-kernel-input".to_owned()],
    }
}

fn summarize_by_kind(cases: &[EvalCaseResult]) -> Vec<EvalKindSummary> {
    let mut summaries = [
        EvalKindSummary::empty(EvalCaseKind::Adversarial),
        EvalKindSummary::empty(EvalCaseKind::Golden),
        EvalKindSummary::empty(EvalCaseKind::Linguistic),
        EvalKindSummary::empty(EvalCaseKind::Regression),
        EvalKindSummary::empty(EvalCaseKind::Safety),
    ];
    for case in cases {
        if let Some(summary) = summaries
            .iter_mut()
            .find(|summary| summary.kind == case.kind)
        {
            summary.total += 1;
            match case.outcome {
                EvalCaseOutcome::Passed => summary.passed += 1,
                EvalCaseOutcome::Failed => summary.failed += 1,
                EvalCaseOutcome::SafetyViolation => summary.safety_violations += 1,
            }
        }
    }
    summaries
        .into_iter()
        .filter(|summary| summary.total > 0)
        .collect()
}

impl EvalKindSummary {
    fn empty(kind: EvalCaseKind) -> Self {
        Self {
            kind,
            total: 0,
            passed: 0,
            failed: 0,
            safety_violations: 0,
        }
    }
}

fn required_coverage_present(
    thresholds: &EvalSuiteThresholds,
    summaries: &[EvalKindSummary],
) -> bool {
    (!thresholds.require_golden || has_kind(summaries, EvalCaseKind::Golden))
        && (!thresholds.require_adversarial || has_kind(summaries, EvalCaseKind::Adversarial))
        && (!thresholds.require_linguistic || has_kind(summaries, EvalCaseKind::Linguistic))
}

fn has_kind(summaries: &[EvalKindSummary], kind: EvalCaseKind) -> bool {
    summaries
        .iter()
        .any(|summary| summary.kind == kind && summary.total > 0)
}

fn suite_evidence_refs(suite: &EvalSuite) -> Vec<String> {
    let mut evidence_refs = vec![
        suite.route_evidence_ref.clone(),
        suite.guardrail_evidence_ref.clone(),
        suite.dataset_snapshot_ref.clone(),
    ];
    evidence_refs.extend(
        suite
            .cases
            .iter()
            .map(|case| case.evaluator_evidence_ref.clone()),
    );
    sorted_unique(evidence_refs)
}

fn ratio_bps(numerator: u32, denominator: u32) -> u32 {
    if denominator == 0 {
        return 0;
    }
    (numerator * BASIS_POINTS_DENOMINATOR + denominator / 2) / denominator
}

fn require_metadata_ref(label: &str, value: &str, reasons: &mut Vec<String>) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        reasons.push(format!("{label} is required"));
    } else if value != trimmed
        || contains_whitespace(trimmed)
        || contains_raw_secret_material(trimmed)
        || contains_raw_content_material(trimmed)
    {
        reasons.push(format!("{label} must be audit-safe metadata"));
    }
}

fn require_evidence_ref(label: &str, value: &str, reasons: &mut Vec<String>) {
    require_opaque_ref(label, value, "opaque evidence ref", reasons);
}

fn require_resource_ref(label: &str, value: &str, reasons: &mut Vec<String>) {
    require_opaque_ref(label, value, "opaque resource ref", reasons);
}

fn require_opaque_ref(label: &str, value: &str, kind: &str, reasons: &mut Vec<String>) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        reasons.push(format!("{label} is required"));
    } else if value != trimmed
        || contains_whitespace(trimmed)
        || contains_raw_secret_material(trimmed)
        || contains_raw_content_material(trimmed)
        || !trimmed.contains(':')
    {
        reasons.push(format!("{label} must be an {kind}"));
    }
}

fn safe_metadata(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || value != trimmed
        || contains_whitespace(trimmed)
        || contains_raw_secret_material(trimmed)
        || contains_raw_content_material(trimmed)
    {
        fallback.to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn safe_ref(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || value != trimmed
        || contains_whitespace(trimmed)
        || contains_raw_secret_material(trimmed)
        || contains_raw_content_material(trimmed)
        || !trimmed.contains(':')
    {
        fallback.to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn contains_whitespace(value: &str) -> bool {
    value.chars().any(char::is_whitespace)
}

fn contains_raw_secret_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("sk-")
        || lower.contains("sk-")
        || lower.contains("bearer")
        || lower.contains("authorization:")
        || lower.contains("api_key=")
        || lower.contains("openai_api_key")
}

fn contains_raw_content_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw output")
        || lower.contains("customer message")
        || lower.contains("write an email")
        || lower.contains("model answer")
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    fn case(
        case_id: &str,
        kind: EvalCaseKind,
        outcome: EvalCaseOutcome,
        score_bps: u16,
        evidence_ref: &str,
    ) -> EvalCaseResult {
        EvalCaseResult {
            case_id: case_id.to_owned(),
            kind,
            outcome,
            score_bps,
            evaluator_evidence_ref: evidence_ref.to_owned(),
        }
    }

    fn sample_suite(suite_id: &str) -> EvalSuite {
        EvalSuite {
            suite_id: suite_id.to_owned(),
            model_ref: "modelref://openai/gpt-preview".to_owned(),
            route_evidence_ref: "route:evidence:1".to_owned(),
            guardrail_evidence_ref: "guardrail:evidence:1".to_owned(),
            dataset_snapshot_ref: "eval-dataset:snapshot:1".to_owned(),
            thresholds: EvalSuiteThresholds {
                min_pass_rate_bps: 7_500,
                max_safety_violation_rate_bps: 0,
                require_golden: true,
                require_adversarial: true,
                require_linguistic: true,
            },
            cases: vec![
                case(
                    "case-golden-1",
                    EvalCaseKind::Golden,
                    EvalCaseOutcome::Passed,
                    9_500,
                    "eval:case:golden:1",
                ),
                case(
                    "case-adversarial-1",
                    EvalCaseKind::Adversarial,
                    EvalCaseOutcome::Passed,
                    8_900,
                    "eval:case:adversarial:1",
                ),
                case(
                    "case-linguistic-1",
                    EvalCaseKind::Linguistic,
                    EvalCaseOutcome::Passed,
                    8_400,
                    "eval:case:linguistic:1",
                ),
                case(
                    "case-regression-1",
                    EvalCaseKind::Regression,
                    EvalCaseOutcome::Failed,
                    4_000,
                    "eval:case:regression:1",
                ),
            ],
        }
    }

    #[test]
    fn scores_eval_suite_with_quality_and_safety_thresholds() {
        let report = evaluate_suite(sample_suite("eval-suite:dispatch-safety"));

        assert_eq!(report.status, EvalSuiteStatus::Passed);
        assert_eq!(report.total_cases, 4);
        assert_eq!(report.passed_cases, 3);
        assert_eq!(report.failed_cases, 1);
        assert_eq!(report.safety_violation_cases, 0);
        assert_eq!(report.pass_rate_bps, 7_500);
        assert_eq!(report.safety_violation_rate_bps, 0);
        assert_eq!(report.failure_kinds, Vec::<EvalFailureKind>::new());
        assert_eq!(
            report
                .summaries
                .iter()
                .map(|summary| summary.kind)
                .collect::<Vec<_>>(),
            vec![
                EvalCaseKind::Adversarial,
                EvalCaseKind::Golden,
                EvalCaseKind::Linguistic,
                EvalCaseKind::Regression,
            ]
        );
        assert!(
            report
                .evidence_refs
                .contains(&"eval-dataset:snapshot:1".to_owned())
        );
        let debug = format!("{report:?}");
        assert!(!debug.contains("raw prompt"));
        assert!(!debug.contains("model answer"));
        assert!(!debug.contains("sk-"));
    }

    #[test]
    fn safety_violation_fails_closed_even_when_pass_rate_is_high() {
        let mut suite = sample_suite("eval-suite:safety-violation");
        suite.cases[3] = case(
            "case-safety-1",
            EvalCaseKind::Safety,
            EvalCaseOutcome::SafetyViolation,
            0,
            "eval:case:safety:1",
        );

        let report = evaluate_suite(suite);

        assert_eq!(report.status, EvalSuiteStatus::Failed);
        assert_eq!(report.safety_violation_cases, 1);
        assert!(
            report
                .failure_kinds
                .contains(&EvalFailureKind::SafetyViolationRateExceeded)
        );
    }

    #[test]
    fn missing_required_eval_coverage_fails() {
        let mut suite = sample_suite("eval-suite:missing-linguistic");
        suite
            .cases
            .retain(|case| case.kind != EvalCaseKind::Linguistic);

        let report = evaluate_suite(suite);

        assert_eq!(report.status, EvalSuiteStatus::Failed);
        assert!(
            report
                .failure_kinds
                .contains(&EvalFailureKind::MissingRequiredCoverage)
        );
    }

    #[test]
    fn threshold_failure_reports_stable_rates() {
        let mut suite = sample_suite("eval-suite:threshold");
        suite.thresholds.min_pass_rate_bps = 9_000;

        let report = evaluate_suite(suite);

        assert_eq!(report.status, EvalSuiteStatus::Failed);
        assert_eq!(report.pass_rate_bps, 7_500);
        assert_eq!(report.failure_kinds, vec![EvalFailureKind::ThresholdFailed]);
    }

    #[test]
    fn invalid_raw_refs_are_redacted_and_do_not_echo_content() {
        let mut suite = sample_suite("sk-test-suite");
        suite.model_ref = "raw prompt: write an email to customer".to_owned();
        suite.cases[0].evaluator_evidence_ref = "Bearer token".to_owned();

        let report = evaluate_suite(suite);
        let debug = format!("{report:?}");

        assert_eq!(report.status, EvalSuiteStatus::Invalid);
        assert_eq!(report.failure_kinds, vec![EvalFailureKind::InvalidInput]);
        assert_eq!(report.suite_id, "redacted-invalid-suite-id");
        assert_eq!(report.model_ref, "redacted-invalid-model-ref");
        assert_eq!(
            report.evidence_refs,
            vec!["validation:intelligence-eval-kernel-input".to_owned()]
        );
        assert!(!debug.contains("write an email"));
        assert!(!debug.contains("sk-test"));
        assert!(!debug.contains("Bearer"));
    }

    #[test]
    fn report_is_deterministic_independent_of_case_order() {
        let suite = sample_suite("eval-suite:deterministic");
        let mut reversed = suite.clone();
        reversed.cases.reverse();

        let report = evaluate_suite(suite);
        let reversed_report = evaluate_suite(reversed);

        assert_eq!(report.status, reversed_report.status);
        assert_eq!(report.summaries, reversed_report.summaries);
        assert_eq!(report.evidence_refs, reversed_report.evidence_refs);
    }
}
