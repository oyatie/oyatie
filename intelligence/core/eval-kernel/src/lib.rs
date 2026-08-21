//! Intelligence eval kernel foundation.
//!
//! This crate scores metadata-only evaluation sets for later Intelligence
//! cloud integration. It models golden, adversarial, and linguistic coverage,
//! deterministic pass/violation rates, and fail-closed threshold checks without
//! model calls, grader calls, network, filesystem, durable storage, or raw
//! prompt/output text.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

const BASIS_POINTS_DENOMINATOR: u32 = 10_000;
const MAX_EVAL_CASES: usize = 10_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EvalSetStatus {
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
pub struct EvalSetThresholds {
    pub min_pass_rate_bps: u32,             // data_class: INTERNAL_ONLY
    pub max_safety_violation_rate_bps: u32, // data_class: INTERNAL_ONLY
    pub require_golden: bool,               // data_class: INTERNAL_ONLY
    pub require_adversarial: bool,          // data_class: INTERNAL_ONLY
    pub require_linguistic: bool,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalSet {
    pub eval_set_id: String,            // data_class: INTERNAL_ONLY
    pub model_ref: String,              // data_class: INTERNAL_ONLY
    pub route_evidence_ref: String,     // data_class: INTERNAL_ONLY
    pub guardrail_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub dataset_snapshot_ref: String,   // data_class: INTERNAL_ONLY
    pub thresholds: EvalSetThresholds,  // data_class: INTERNAL_ONLY
    pub cases: Vec<EvalCaseResult>,     // data_class: INTERNAL_ONLY
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
pub struct EvalSetReport {
    pub eval_set_id: String,                 // data_class: INTERNAL_ONLY
    pub model_ref: String,                   // data_class: INTERNAL_ONLY
    pub status: EvalSetStatus,               // data_class: PUBLIC
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

pub fn evaluate_eval_set(eval_set: EvalSet) -> EvalSetReport {
    let invalid = invalid_eval_set_reasons(&eval_set);
    if !invalid.is_empty() {
        return invalid_report(&eval_set);
    }

    let evidence_refs = eval_set_evidence_refs(&eval_set);
    let summaries = summarize_by_kind(&eval_set.cases);
    let total_cases = eval_set.cases.len() as u32;
    let passed_cases = eval_set
        .cases
        .iter()
        .filter(|case| case.outcome == EvalCaseOutcome::Passed)
        .count() as u32;
    let safety_violation_cases = eval_set
        .cases
        .iter()
        .filter(|case| case.outcome == EvalCaseOutcome::SafetyViolation)
        .count() as u32;
    let failed_cases = total_cases.saturating_sub(passed_cases + safety_violation_cases);
    let pass_rate_bps = ratio_bps(passed_cases, total_cases);
    let safety_violation_rate_bps = ratio_bps(safety_violation_cases, total_cases);

    let mut failure_kinds = Vec::new();
    if !required_coverage_present(&eval_set.thresholds, &summaries) {
        failure_kinds.push(EvalFailureKind::MissingRequiredCoverage);
    }
    if pass_rate_bps < eval_set.thresholds.min_pass_rate_bps {
        failure_kinds.push(EvalFailureKind::ThresholdFailed);
    }
    if safety_violation_rate_bps > eval_set.thresholds.max_safety_violation_rate_bps {
        failure_kinds.push(EvalFailureKind::SafetyViolationRateExceeded);
    }
    failure_kinds.sort();
    failure_kinds.dedup();

    EvalSetReport {
        eval_set_id: eval_set.eval_set_id,
        model_ref: eval_set.model_ref,
        status: if failure_kinds.is_empty() {
            EvalSetStatus::Passed
        } else {
            EvalSetStatus::Failed
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

fn invalid_eval_set_reasons(eval_set: &EvalSet) -> Vec<String> {
    let mut reasons = Vec::new();
    require_metadata_ref("eval_set id", &eval_set.eval_set_id, &mut reasons);
    require_resource_ref("model ref", &eval_set.model_ref, &mut reasons);
    require_evidence_ref(
        "route evidence ref",
        &eval_set.route_evidence_ref,
        &mut reasons,
    );
    require_evidence_ref(
        "guardrail evidence ref",
        &eval_set.guardrail_evidence_ref,
        &mut reasons,
    );
    require_evidence_ref(
        "dataset snapshot ref",
        &eval_set.dataset_snapshot_ref,
        &mut reasons,
    );
    if eval_set.cases.is_empty() {
        reasons.push("eval cases are required".to_owned());
    } else if eval_set.cases.len() > MAX_EVAL_CASES {
        reasons.push(format!("eval case count must be <= {MAX_EVAL_CASES}"));
    }
    if eval_set.thresholds.min_pass_rate_bps > BASIS_POINTS_DENOMINATOR {
        reasons.push("minimum pass rate must be 0..=10000 basis points".to_owned());
    }
    if eval_set.thresholds.max_safety_violation_rate_bps > BASIS_POINTS_DENOMINATOR {
        reasons.push("maximum safety violation rate must be 0..=10000 basis points".to_owned());
    }
    for case in &eval_set.cases {
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

fn invalid_report(eval_set: &EvalSet) -> EvalSetReport {
    EvalSetReport {
        eval_set_id: safe_metadata(&eval_set.eval_set_id, "redacted-invalid-eval_set-id"),
        model_ref: safe_ref(&eval_set.model_ref, "redacted-invalid-model-ref"),
        status: EvalSetStatus::Invalid,
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
    thresholds: &EvalSetThresholds,
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

fn eval_set_evidence_refs(eval_set: &EvalSet) -> Vec<String> {
    let mut evidence_refs = vec![
        eval_set.route_evidence_ref.clone(),
        eval_set.guardrail_evidence_ref.clone(),
        eval_set.dataset_snapshot_ref.clone(),
    ];
    evidence_refs.extend(
        eval_set
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

    fn sample_eval_set(eval_set_id: &str) -> EvalSet {
        EvalSet {
            eval_set_id: eval_set_id.to_owned(),
            model_ref: "modelref://openai/gpt-preview".to_owned(),
            route_evidence_ref: "route:evidence:1".to_owned(),
            guardrail_evidence_ref: "guardrail:evidence:1".to_owned(),
            dataset_snapshot_ref: "eval-dataset:snapshot:1".to_owned(),
            thresholds: EvalSetThresholds {
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
    fn scores_eval_set_with_quality_and_safety_thresholds() {
        let report = evaluate_eval_set(sample_eval_set("eval_set:dispatch-safety"));

        assert_eq!(report.status, EvalSetStatus::Passed);
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
        let mut eval_set = sample_eval_set("eval_set:safety-violation");
        eval_set.cases[3] = case(
            "case-safety-1",
            EvalCaseKind::Safety,
            EvalCaseOutcome::SafetyViolation,
            0,
            "eval:case:safety:1",
        );

        let report = evaluate_eval_set(eval_set);

        assert_eq!(report.status, EvalSetStatus::Failed);
        assert_eq!(report.safety_violation_cases, 1);
        assert!(
            report
                .failure_kinds
                .contains(&EvalFailureKind::SafetyViolationRateExceeded)
        );
    }

    #[test]
    fn missing_required_eval_coverage_fails() {
        let mut eval_set = sample_eval_set("eval_set:missing-linguistic");
        eval_set
            .cases
            .retain(|case| case.kind != EvalCaseKind::Linguistic);

        let report = evaluate_eval_set(eval_set);

        assert_eq!(report.status, EvalSetStatus::Failed);
        assert!(
            report
                .failure_kinds
                .contains(&EvalFailureKind::MissingRequiredCoverage)
        );
    }

    #[test]
    fn threshold_failure_reports_stable_rates() {
        let mut eval_set = sample_eval_set("eval_set:threshold");
        eval_set.thresholds.min_pass_rate_bps = 9_000;

        let report = evaluate_eval_set(eval_set);

        assert_eq!(report.status, EvalSetStatus::Failed);
        assert_eq!(report.pass_rate_bps, 7_500);
        assert_eq!(report.failure_kinds, vec![EvalFailureKind::ThresholdFailed]);
    }

    #[test]
    fn invalid_raw_refs_are_redacted_and_do_not_echo_content() {
        let mut eval_set = sample_eval_set("sk-test-set");
        eval_set.model_ref = "raw prompt: write an email to customer".to_owned();
        eval_set.cases[0].evaluator_evidence_ref = "Bearer token".to_owned();

        let report = evaluate_eval_set(eval_set);
        let debug = format!("{report:?}");

        assert_eq!(report.status, EvalSetStatus::Invalid);
        assert_eq!(report.failure_kinds, vec![EvalFailureKind::InvalidInput]);
        assert_eq!(report.eval_set_id, "redacted-invalid-eval_set-id");
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
        let eval_set = sample_eval_set("eval_set:deterministic");
        let mut reversed = eval_set.clone();
        reversed.cases.reverse();

        let report = evaluate_eval_set(eval_set);
        let reversed_report = evaluate_eval_set(reversed);

        assert_eq!(report.status, reversed_report.status);
        assert_eq!(report.summaries, reversed_report.summaries);
        assert_eq!(report.evidence_refs, reversed_report.evidence_refs);
    }
}
