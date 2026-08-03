//! Intelligence eval domain foundation.
//!
//! The domain layer validates tenant/principal policy authority, eval surface,
//! model and dataset allowlists, case-kind coverage boundaries, and threshold
//! floors before returning deterministic metadata-only eval reports from the
//! kernel. It performs no model calls, grader calls, hosted eval runs, dataset
//! fetches, filesystem, network, durable storage, or audit-chain emission.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use intelligence_eval_kernel::{
    EvalCaseKind, EvalCaseOutcome, EvalCaseResult, EvalFailureKind, EvalKindSummary, EvalSet,
    EvalSetReport, EvalSetStatus, EvalSetThresholds, evaluate_eval_set,
};

const BASIS_POINTS_DENOMINATOR: u32 = 10_000;
const DOMAIN_MAX_EVAL_CASES: usize = 10_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalPolicyDecision {
    pub decision_id: String,                        // data_class: INTERNAL_ONLY
    pub tenant_id: String,                          // data_class: INTERNAL_ONLY
    pub principal_id: String,                       // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>,              // data_class: INTERNAL_ONLY
    pub allowed_model_refs: Vec<String>,            // data_class: INTERNAL_ONLY
    pub allowed_dataset_snapshot_refs: Vec<String>, // data_class: INTERNAL_ONLY
    pub allowed_case_kinds: Vec<EvalCaseKind>,      // data_class: INTERNAL_ONLY
    pub min_case_count: usize,                      // data_class: INTERNAL_ONLY
    pub max_case_count: usize,                      // data_class: INTERNAL_ONLY
    pub min_pass_rate_bps: u32,                     // data_class: INTERNAL_ONLY
    pub max_safety_violation_rate_bps: u32,         // data_class: INTERNAL_ONLY
    pub evidence_ref: String,                       // data_class: INTERNAL_ONLY
    pub eval_registry_snapshot_ref: String,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainEvalSetRequest {
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub principal_id: String,                // data_class: INTERNAL_ONLY
    pub eval_surface: String,                // data_class: INTERNAL_ONLY
    pub request_evidence_ref: String,        // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,           // data_class: INTERNAL_ONLY
    pub policy_decision_ref: String,         // data_class: INTERNAL_ONLY
    pub policy_decision: EvalPolicyDecision, // data_class: INTERNAL_ONLY
    pub eval_set: EvalSet,                   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvalDomainStatus {
    Evaluated,
    Denied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EvalDomainDenialKind {
    CaseCountDenied,
    DatasetDenied,
    EvalKindDenied,
    InvalidInput,
    KernelInvalid,
    ModelDenied,
    PolicyDrift,
    SurfaceDenied,
    ThresholdWeakened,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalDomainReport {
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub principal_id: String,           // data_class: INTERNAL_ONLY
    pub eval_surface: String,           // data_class: INTERNAL_ONLY
    pub status: EvalDomainStatus,       // data_class: PUBLIC
    pub eval_set_report: EvalSetReport, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalDomainDenial {
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub principal_id: String,              // data_class: INTERNAL_ONLY
    pub eval_surface: String,              // data_class: INTERNAL_ONLY
    pub eval_set_id: String,               // data_class: INTERNAL_ONLY
    pub model_ref: String,                 // data_class: INTERNAL_ONLY
    pub status: EvalDomainStatus,          // data_class: PUBLIC
    pub denial_kind: EvalDomainDenialKind, // data_class: INTERNAL_ONLY
    pub reasons: Vec<String>,              // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvalDomainDecision {
    Report(EvalDomainReport),
    Deny(EvalDomainDenial),
}

impl EvalDomainDecision {
    pub fn status(&self) -> EvalDomainStatus {
        match self {
            Self::Report(report) => report.status,
            Self::Deny(denial) => denial.status,
        }
    }

    pub fn evidence_refs(&self) -> &[String] {
        match self {
            Self::Report(report) => &report.evidence_refs,
            Self::Deny(denial) => &denial.evidence_refs,
        }
    }
}

pub fn evaluate_domain_eval_set(input: DomainEvalSetRequest) -> EvalDomainDecision {
    let invalid = invalid_input_reasons(&input);
    if !invalid.is_empty() {
        return EvalDomainDecision::Deny(denial_from_parts(EvalDenialParts {
            tenant_id: safe_tenant(&input.tenant_id),
            principal_id: safe_metadata(&input.principal_id, "redacted-invalid-principal-id"),
            eval_surface: safe_metadata(&input.eval_surface, "redacted-invalid-eval-surface"),
            eval_set_id: safe_metadata(&input.eval_set.eval_set_id, "redacted-invalid-eval_set-id"),
            model_ref: safe_ref(&input.eval_set.model_ref, "redacted-invalid-model-ref"),
            denial_kind: EvalDomainDenialKind::InvalidInput,
            reasons: invalid,
            evidence_refs: vec!["validation:intelligence-eval-domain-input".to_owned()],
        }));
    }

    let eval_set_report = evaluate_eval_set(input.eval_set.clone());
    if eval_set_report.status == EvalSetStatus::Invalid {
        return EvalDomainDecision::Deny(denial_from_parts(EvalDenialParts {
            tenant_id: input.tenant_id.clone(),
            principal_id: input.principal_id.clone(),
            eval_surface: input.eval_surface.clone(),
            eval_set_id: eval_set_report.eval_set_id.clone(),
            model_ref: eval_set_report.model_ref.clone(),
            denial_kind: EvalDomainDenialKind::KernelInvalid,
            reasons: vec!["eval kernel rejected the eval_set metadata".to_owned()],
            evidence_refs: sorted_unique(
                [
                    eval_set_report.evidence_refs.clone(),
                    policy_evidence_refs(&input),
                ]
                .concat(),
            ),
        }));
    }

    if input.policy_decision.tenant_id != input.tenant_id
        || input.policy_decision.principal_id != input.principal_id
        || input.policy_decision_ref != input.policy_decision.evidence_ref
    {
        return domain_denial(
            &input,
            EvalDomainDenialKind::PolicyDrift,
            vec![
                "eval policy decision is not bound to request tenant/principal/evidence".to_owned(),
            ],
            vec![
                input.request_evidence_ref.clone(),
                input.policy_decision_ref.clone(),
                input.policy_decision.evidence_ref.clone(),
                "validation:intelligence-eval-policy-drift".to_owned(),
            ],
        );
    }

    if !input
        .policy_decision
        .allowed_surfaces
        .iter()
        .any(|surface| surface == &input.eval_surface)
    {
        return domain_denial(
            &input,
            EvalDomainDenialKind::SurfaceDenied,
            vec!["eval policy decision does not allow this surface".to_owned()],
            policy_evidence_refs(&input),
        );
    }

    if !input
        .policy_decision
        .allowed_model_refs
        .iter()
        .any(|model_ref| model_ref == &input.eval_set.model_ref)
    {
        return domain_denial(
            &input,
            EvalDomainDenialKind::ModelDenied,
            vec!["eval policy decision does not allow this model ref".to_owned()],
            policy_and_eval_set_evidence_refs(&input, &eval_set_report),
        );
    }

    if !input
        .policy_decision
        .allowed_dataset_snapshot_refs
        .iter()
        .any(|dataset_ref| dataset_ref == &input.eval_set.dataset_snapshot_ref)
    {
        return domain_denial(
            &input,
            EvalDomainDenialKind::DatasetDenied,
            vec!["eval policy decision does not allow this dataset snapshot".to_owned()],
            policy_and_eval_set_evidence_refs(&input, &eval_set_report),
        );
    }

    if !eval_set_case_kinds_are_policy_allowed(&input) {
        return domain_denial(
            &input,
            EvalDomainDenialKind::EvalKindDenied,
            vec!["eval set case kinds exceed policy decision".to_owned()],
            policy_and_eval_set_evidence_refs(&input, &eval_set_report),
        );
    }

    let case_count = input.eval_set.cases.len();
    if case_count < input.policy_decision.min_case_count
        || case_count > input.policy_decision.max_case_count
    {
        return domain_denial(
            &input,
            EvalDomainDenialKind::CaseCountDenied,
            vec!["eval set case count is outside policy bounds".to_owned()],
            policy_and_eval_set_evidence_refs(&input, &eval_set_report),
        );
    }

    if input.eval_set.thresholds.min_pass_rate_bps < input.policy_decision.min_pass_rate_bps
        || input.eval_set.thresholds.max_safety_violation_rate_bps
            > input.policy_decision.max_safety_violation_rate_bps
    {
        return domain_denial(
            &input,
            EvalDomainDenialKind::ThresholdWeakened,
            vec!["eval set thresholds are weaker than policy decision".to_owned()],
            policy_and_eval_set_evidence_refs(&input, &eval_set_report),
        );
    }

    let evidence_refs = policy_and_report_evidence_refs(&input, &eval_set_report);
    EvalDomainDecision::Report(EvalDomainReport {
        tenant_id: input.tenant_id,
        principal_id: input.principal_id,
        eval_surface: input.eval_surface,
        status: EvalDomainStatus::Evaluated,
        eval_set_report,
        evidence_refs,
    })
}

fn invalid_input_reasons(input: &DomainEvalSetRequest) -> Vec<String> {
    let mut reasons = Vec::new();
    require_tenant("tenant id", &input.tenant_id, &mut reasons);
    require_metadata_ref("principal id", &input.principal_id, &mut reasons);
    require_metadata_ref("eval surface", &input.eval_surface, &mut reasons);
    require_evidence_ref(
        "request evidence ref",
        &input.request_evidence_ref,
        &mut reasons,
    );
    require_evidence_ref("trace context ref", &input.trace_context_ref, &mut reasons);
    require_evidence_ref(
        "policy decision ref",
        &input.policy_decision_ref,
        &mut reasons,
    );
    require_metadata_ref(
        "policy decision id",
        &input.policy_decision.decision_id,
        &mut reasons,
    );
    require_tenant(
        "policy tenant id",
        &input.policy_decision.tenant_id,
        &mut reasons,
    );
    require_metadata_ref(
        "policy principal id",
        &input.policy_decision.principal_id,
        &mut reasons,
    );
    validate_metadata_list(
        "policy allowed surface",
        &input.policy_decision.allowed_surfaces,
        &mut reasons,
    );
    validate_ref_list(
        "policy allowed model ref",
        &input.policy_decision.allowed_model_refs,
        &mut reasons,
    );
    validate_ref_list(
        "policy allowed dataset snapshot ref",
        &input.policy_decision.allowed_dataset_snapshot_refs,
        &mut reasons,
    );
    if input.policy_decision.allowed_case_kinds.is_empty() {
        reasons.push("policy allowed case kinds are required".to_owned());
    }
    if input.policy_decision.min_case_count == 0 {
        reasons.push("policy min case count must be greater than zero".to_owned());
    }
    if input.policy_decision.max_case_count < input.policy_decision.min_case_count
        || input.policy_decision.max_case_count > DOMAIN_MAX_EVAL_CASES
    {
        reasons.push(format!(
            "policy max case count must be between min case count and {DOMAIN_MAX_EVAL_CASES}"
        ));
    }
    if input.policy_decision.min_pass_rate_bps > BASIS_POINTS_DENOMINATOR {
        reasons.push("policy minimum pass rate must be 0..=10000 basis points".to_owned());
    }
    if input.policy_decision.max_safety_violation_rate_bps > BASIS_POINTS_DENOMINATOR {
        reasons
            .push("policy maximum safety violation rate must be 0..=10000 basis points".to_owned());
    }
    require_evidence_ref(
        "policy evidence ref",
        &input.policy_decision.evidence_ref,
        &mut reasons,
    );
    require_evidence_ref(
        "eval registry snapshot ref",
        &input.policy_decision.eval_registry_snapshot_ref,
        &mut reasons,
    );
    sorted_unique(reasons)
}

fn validate_metadata_list(label: &str, values: &[String], reasons: &mut Vec<String>) {
    if values.is_empty() {
        reasons.push(format!("{label} list is required"));
    }
    for value in values {
        require_metadata_ref(label, value, reasons);
    }
}

fn validate_ref_list(label: &str, values: &[String], reasons: &mut Vec<String>) {
    if values.is_empty() {
        reasons.push(format!("{label} list is required"));
    }
    for value in values {
        require_resource_ref(label, value, reasons);
    }
}

fn eval_set_case_kinds_are_policy_allowed(input: &DomainEvalSetRequest) -> bool {
    input.eval_set.cases.iter().all(|case| {
        input
            .policy_decision
            .allowed_case_kinds
            .iter()
            .any(|allowed| allowed == &case.kind)
    })
}

fn domain_denial(
    input: &DomainEvalSetRequest,
    denial_kind: EvalDomainDenialKind,
    reasons: Vec<String>,
    evidence_refs: Vec<String>,
) -> EvalDomainDecision {
    EvalDomainDecision::Deny(denial_from_parts(EvalDenialParts {
        tenant_id: input.tenant_id.clone(),
        principal_id: input.principal_id.clone(),
        eval_surface: input.eval_surface.clone(),
        eval_set_id: input.eval_set.eval_set_id.clone(),
        model_ref: input.eval_set.model_ref.clone(),
        denial_kind,
        reasons,
        evidence_refs,
    }))
}

struct EvalDenialParts {
    tenant_id: String,
    principal_id: String,
    eval_surface: String,
    eval_set_id: String,
    model_ref: String,
    denial_kind: EvalDomainDenialKind,
    reasons: Vec<String>,
    evidence_refs: Vec<String>,
}

fn denial_from_parts(parts: EvalDenialParts) -> EvalDomainDenial {
    EvalDomainDenial {
        tenant_id: parts.tenant_id,
        principal_id: parts.principal_id,
        eval_surface: parts.eval_surface,
        eval_set_id: parts.eval_set_id,
        model_ref: parts.model_ref,
        status: EvalDomainStatus::Denied,
        denial_kind: parts.denial_kind,
        reasons: sorted_unique(parts.reasons),
        evidence_refs: sorted_unique(parts.evidence_refs),
    }
}

fn policy_evidence_refs(input: &DomainEvalSetRequest) -> Vec<String> {
    sorted_unique(vec![
        input.request_evidence_ref.clone(),
        input.trace_context_ref.clone(),
        input.policy_decision_ref.clone(),
        input.policy_decision.evidence_ref.clone(),
        input.policy_decision.eval_registry_snapshot_ref.clone(),
    ])
}

fn policy_and_eval_set_evidence_refs(
    input: &DomainEvalSetRequest,
    eval_set_report: &EvalSetReport,
) -> Vec<String> {
    sorted_unique(
        [
            eval_set_report.evidence_refs.clone(),
            policy_evidence_refs(input),
        ]
        .concat(),
    )
}

fn policy_and_report_evidence_refs(
    input: &DomainEvalSetRequest,
    eval_set_report: &EvalSetReport,
) -> Vec<String> {
    policy_and_eval_set_evidence_refs(input, eval_set_report)
}

fn require_tenant(label: &str, value: &str, reasons: &mut Vec<String>) {
    require_opaque_ref(label, value, "tenant ref", reasons);
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

fn safe_tenant(value: &str) -> String {
    safe_ref(value, "redacted-invalid-tenant-id")
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
            route_evidence_ref: "route:evidence:domain:1".to_owned(),
            guardrail_evidence_ref: "guardrail:evidence:domain:1".to_owned(),
            dataset_snapshot_ref: "eval-dataset:snapshot:domain:1".to_owned(),
            thresholds: EvalSetThresholds {
                min_pass_rate_bps: 7_500,
                max_safety_violation_rate_bps: 0,
                require_golden: true,
                require_adversarial: true,
                require_linguistic: true,
            },
            cases: vec![
                case(
                    "case-golden-domain-1",
                    EvalCaseKind::Golden,
                    EvalCaseOutcome::Passed,
                    9_500,
                    "eval:case:domain:golden:1",
                ),
                case(
                    "case-adversarial-domain-1",
                    EvalCaseKind::Adversarial,
                    EvalCaseOutcome::Passed,
                    8_900,
                    "eval:case:domain:adversarial:1",
                ),
                case(
                    "case-linguistic-domain-1",
                    EvalCaseKind::Linguistic,
                    EvalCaseOutcome::Passed,
                    8_400,
                    "eval:case:domain:linguistic:1",
                ),
                case(
                    "case-regression-domain-1",
                    EvalCaseKind::Regression,
                    EvalCaseOutcome::Failed,
                    4_000,
                    "eval:case:domain:regression:1",
                ),
            ],
        }
    }

    fn sample_policy() -> EvalPolicyDecision {
        EvalPolicyDecision {
            decision_id: "eval-policy-decision:domain:1".to_owned(),
            tenant_id: "tenant:alpha".to_owned(),
            principal_id: "principal:eval-owner".to_owned(),
            allowed_surfaces: vec!["surface:release-gate".to_owned()],
            allowed_model_refs: vec!["modelref://openai/gpt-preview".to_owned()],
            allowed_dataset_snapshot_refs: vec!["eval-dataset:snapshot:domain:1".to_owned()],
            allowed_case_kinds: vec![
                EvalCaseKind::Adversarial,
                EvalCaseKind::Golden,
                EvalCaseKind::Linguistic,
                EvalCaseKind::Regression,
                EvalCaseKind::Safety,
            ],
            min_case_count: 3,
            max_case_count: 12,
            min_pass_rate_bps: 7_500,
            max_safety_violation_rate_bps: 0,
            evidence_ref: "policy:evidence:eval-domain:1".to_owned(),
            eval_registry_snapshot_ref: "eval-registry:snapshot:1".to_owned(),
        }
    }

    fn sample_domain_request(eval_set_id: &str) -> DomainEvalSetRequest {
        DomainEvalSetRequest {
            tenant_id: "tenant:alpha".to_owned(),
            principal_id: "principal:eval-owner".to_owned(),
            eval_surface: "surface:release-gate".to_owned(),
            request_evidence_ref: "request:evidence:eval-domain:1".to_owned(),
            trace_context_ref: "trace:eval-domain:1".to_owned(),
            policy_decision_ref: "policy:evidence:eval-domain:1".to_owned(),
            policy_decision: sample_policy(),
            eval_set: sample_eval_set(eval_set_id),
        }
    }

    #[test]
    fn authorized_eval_domain_request_scores_kernel_report_with_policy_evidence() {
        let decision = evaluate_domain_eval_set(sample_domain_request("eval_set:domain-pass"));

        let EvalDomainDecision::Report(report) = decision else {
            panic!("expected report");
        };
        assert_eq!(report.status, EvalDomainStatus::Evaluated);
        assert_eq!(report.eval_set_report.status, EvalSetStatus::Passed);
        assert_eq!(report.eval_set_report.pass_rate_bps, 7_500);
        assert!(
            report
                .evidence_refs
                .contains(&"policy:evidence:eval-domain:1".to_owned())
        );
        assert!(
            report
                .evidence_refs
                .contains(&"eval-registry:snapshot:1".to_owned())
        );
        assert!(
            report
                .evidence_refs
                .contains(&"eval-dataset:snapshot:domain:1".to_owned())
        );
    }

    #[test]
    fn policy_drift_and_surface_denial_block_before_kernel() {
        let mut drift = sample_domain_request("eval_set:policy-drift");
        drift.policy_decision.tenant_id = "tenant:other".to_owned();
        let drift_decision = evaluate_domain_eval_set(drift);
        assert_eq!(drift_decision.status(), EvalDomainStatus::Denied);
        let EvalDomainDecision::Deny(denial) = drift_decision else {
            panic!("expected policy drift denial");
        };
        assert_eq!(denial.denial_kind, EvalDomainDenialKind::PolicyDrift);

        let mut surface = sample_domain_request("eval_set:surface-denied");
        surface.eval_surface = "surface:unapproved".to_owned();
        let surface_decision = evaluate_domain_eval_set(surface);
        let EvalDomainDecision::Deny(denial) = surface_decision else {
            panic!("expected surface denial");
        };
        assert_eq!(denial.denial_kind, EvalDomainDenialKind::SurfaceDenied);
    }

    #[test]
    fn model_dataset_and_case_kind_allowlists_are_enforced() {
        let mut model = sample_domain_request("eval_set:model-denied");
        model.eval_set.model_ref = "modelref://openai/other".to_owned();
        let EvalDomainDecision::Deny(denial) = evaluate_domain_eval_set(model) else {
            panic!("expected model denial");
        };
        assert_eq!(denial.denial_kind, EvalDomainDenialKind::ModelDenied);

        let mut dataset = sample_domain_request("eval_set:dataset-denied");
        dataset.eval_set.dataset_snapshot_ref = "eval-dataset:snapshot:unapproved".to_owned();
        let EvalDomainDecision::Deny(denial) = evaluate_domain_eval_set(dataset) else {
            panic!("expected dataset denial");
        };
        assert_eq!(denial.denial_kind, EvalDomainDenialKind::DatasetDenied);

        let mut kind = sample_domain_request("eval_set:kind-denied");
        kind.policy_decision
            .allowed_case_kinds
            .retain(|allowed| allowed != &EvalCaseKind::Linguistic);
        let EvalDomainDecision::Deny(denial) = evaluate_domain_eval_set(kind) else {
            panic!("expected kind denial");
        };
        assert_eq!(denial.denial_kind, EvalDomainDenialKind::EvalKindDenied);
    }

    #[test]
    fn weak_thresholds_or_case_count_outside_policy_denies() {
        let mut weak = sample_domain_request("eval_set:weak-thresholds");
        weak.eval_set.thresholds.min_pass_rate_bps = 5_000;
        let EvalDomainDecision::Deny(denial) = evaluate_domain_eval_set(weak) else {
            panic!("expected threshold denial");
        };
        assert_eq!(denial.denial_kind, EvalDomainDenialKind::ThresholdWeakened);

        let mut too_few = sample_domain_request("eval_set:too-few");
        too_few.policy_decision.min_case_count = 5;
        let EvalDomainDecision::Deny(denial) = evaluate_domain_eval_set(too_few) else {
            panic!("expected case count denial");
        };
        assert_eq!(denial.denial_kind, EvalDomainDenialKind::CaseCountDenied);
    }

    #[test]
    fn kernel_invalid_eval_set_is_preserved_as_fail_closed_denial() {
        let mut request = sample_domain_request("eval_set:kernel-invalid");
        request.eval_set.cases.clear();

        let decision = evaluate_domain_eval_set(request);

        let EvalDomainDecision::Deny(denial) = decision else {
            panic!("expected kernel invalid denial");
        };
        assert_eq!(denial.denial_kind, EvalDomainDenialKind::KernelInvalid);
        assert!(
            denial
                .evidence_refs
                .contains(&"validation:intelligence-eval-kernel-input".to_owned())
        );
    }

    #[test]
    fn kernel_invalid_raw_eval_set_metadata_is_redacted_by_domain_denial() {
        let mut request = sample_domain_request("raw prompt: write an email");
        request.eval_set.model_ref = "Bearer sk-unsafe model answer".to_owned();

        let decision = evaluate_domain_eval_set(request);
        let debug = format!("{decision:?}");

        let EvalDomainDecision::Deny(denial) = decision else {
            panic!("expected kernel invalid denial");
        };
        assert_eq!(denial.denial_kind, EvalDomainDenialKind::KernelInvalid);
        assert_eq!(denial.eval_set_id, "redacted-invalid-eval_set-id");
        assert_eq!(denial.model_ref, "redacted-invalid-model-ref");
        assert!(!debug.contains("write an email"));
        assert!(!debug.contains("Bearer"));
        assert!(!debug.contains("model answer"));
        assert!(!debug.contains("sk-unsafe"));
    }

    #[test]
    fn failed_kernel_report_remains_evaluated_fail_closed_with_evidence() {
        let mut request = sample_domain_request("eval_set:safety-failure");
        request.eval_set.cases[3] = case(
            "case-safety-domain-1",
            EvalCaseKind::Safety,
            EvalCaseOutcome::SafetyViolation,
            0,
            "eval:case:domain:safety:1",
        );

        let EvalDomainDecision::Report(report) = evaluate_domain_eval_set(request) else {
            panic!("expected failed kernel report, not domain denial");
        };
        assert_eq!(report.eval_set_report.status, EvalSetStatus::Failed);
        assert!(
            report
                .eval_set_report
                .failure_kinds
                .contains(&EvalFailureKind::SafetyViolationRateExceeded)
        );
    }

    #[test]
    fn invalid_raw_identity_and_policy_refs_are_redacted() {
        let mut request = sample_domain_request("sk-domain-eval_set");
        request.principal_id = "raw prompt: write an email".to_owned();
        request.policy_decision.evidence_ref = "Bearer token".to_owned();

        let decision = evaluate_domain_eval_set(request);
        let debug = format!("{decision:?}");

        let EvalDomainDecision::Deny(denial) = decision else {
            panic!("expected invalid input denial");
        };
        assert_eq!(denial.denial_kind, EvalDomainDenialKind::InvalidInput);
        assert_eq!(denial.principal_id, "redacted-invalid-principal-id");
        assert_eq!(denial.eval_set_id, "redacted-invalid-eval_set-id");
        assert!(!debug.contains("write an email"));
        assert!(!debug.contains("Bearer"));
        assert!(!debug.contains("sk-domain"));
    }

    #[test]
    fn reports_are_deterministic_independent_of_case_order() {
        let request = sample_domain_request("eval_set:deterministic-domain");
        let mut reversed = request.clone();
        reversed.eval_set.cases.reverse();

        let report = evaluate_domain_eval_set(request);
        let reversed_report = evaluate_domain_eval_set(reversed);

        assert_eq!(report.status(), reversed_report.status());
        assert_eq!(report.evidence_refs(), reversed_report.evidence_refs());
    }
}
