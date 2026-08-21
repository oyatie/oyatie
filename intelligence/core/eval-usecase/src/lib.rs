//! Intelligence eval usecase foundation.
//!
//! This usecase adds idempotent, metadata-only orchestration around the eval
//! domain layer for later cloud integration. It records in-memory audit event
//! metadata for request/evaluated/denied/conflict paths, but performs no model
//! calls, hosted eval runs, dataset fetches, filesystem/network I/O, durable
//! idempotency storage, or durable audit-chain emission.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

pub use intelligence_eval_domain::{
    DomainEvalSetRequest, EvalCaseKind, EvalCaseOutcome, EvalCaseResult, EvalDomainDecision,
    EvalDomainDenialKind, EvalDomainStatus, EvalFailureKind, EvalPolicyDecision, EvalSet,
    EvalSetStatus, EvalSetThresholds, evaluate_domain_eval_set,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalUsecaseInput {
    pub idempotency_key: String,       // data_class: INTERNAL_ONLY
    pub request: DomainEvalSetRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvalUsecaseStatus {
    Evaluated,
    Denied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EvalUsecaseDenialKind {
    DomainDenied,
    IdempotencyConflict,
    InvalidInput,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalUsecaseReceipt {
    pub idempotency_key: String,   // data_class: INTERNAL_ONLY
    pub tenant_id: String,         // data_class: INTERNAL_ONLY
    pub principal_id: String,      // data_class: INTERNAL_ONLY
    pub eval_surface: String,      // data_class: INTERNAL_ONLY
    pub eval_set_id: String,       // data_class: INTERNAL_ONLY
    pub model_ref: String,         // data_class: INTERNAL_ONLY
    pub status: EvalUsecaseStatus, // data_class: PUBLIC
    pub denial_kind: Option<EvalUsecaseDenialKind>, // data_class: INTERNAL_ONLY
    pub domain_denial_kind: Option<EvalDomainDenialKind>, // data_class: INTERNAL_ONLY
    pub eval_set_status: Option<EvalSetStatus>, // data_class: INTERNAL_ONLY
    pub failure_kinds: Vec<EvalFailureKind>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EvalAuditEventKind {
    EvalDenied,
    EvalEvaluated,
    EvalRequested,
    IdempotencyConflict,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalAuditEvent {
    pub kind: EvalAuditEventKind,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,                      // data_class: INTERNAL_ONLY
    pub principal_id: String,                   // data_class: INTERNAL_ONLY
    pub eval_surface: String,                   // data_class: INTERNAL_ONLY
    pub eval_set_id: String,                    // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                // data_class: INTERNAL_ONLY
    pub status: Option<EvalUsecaseStatus>,      // data_class: PUBLIC
    pub eval_set_status: Option<EvalSetStatus>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,             // data_class: INTERNAL_ONLY
}

#[derive(Default)]
pub struct IntelligenceEvalUsecase {
    receipts_by_idempotency_key: BTreeMap<String, EvalUsecaseReceipt>,
    intents_by_idempotency_key: BTreeMap<String, EvalUsecaseIntent>,
    audit_events: Vec<EvalAuditEvent>,
}

impl IntelligenceEvalUsecase {
    pub fn evaluate(&mut self, input: EvalUsecaseInput) -> EvalUsecaseReceipt {
        let invalid = invalid_usecase_input_reasons(&input);
        if !invalid.is_empty() {
            return invalid_receipt_from_input(
                &input,
                EvalUsecaseDenialKind::InvalidInput,
                invalid,
                vec!["validation:intelligence-eval-usecase-input".to_owned()],
            );
        }

        let domain_decision = evaluate_domain_eval_set(input.request.clone());
        if let EvalDomainDecision::Deny(denial) = &domain_decision
            && matches!(
                denial.denial_kind,
                EvalDomainDenialKind::InvalidInput | EvalDomainDenialKind::KernelInvalid
            )
        {
            return receipt_from_domain_denial(
                &input.idempotency_key,
                denial,
                EvalUsecaseDenialKind::InvalidInput,
            );
        }

        let intent = EvalUsecaseIntent::from_input(&input);
        if let Some(existing) = self.receipts_by_idempotency_key.get(&input.idempotency_key) {
            if self.intents_by_idempotency_key.get(&input.idempotency_key) == Some(&intent) {
                return existing.clone();
            }
            let receipt = invalid_receipt_from_input(
                &input,
                EvalUsecaseDenialKind::IdempotencyConflict,
                vec!["idempotency key already used for different eval intent".to_owned()],
                vec!["validation:intelligence-eval-idempotency-conflict".to_owned()],
            );
            self.record_event(EvalAuditEventKind::IdempotencyConflict, &receipt);
            return receipt;
        }

        self.record_event(
            EvalAuditEventKind::EvalRequested,
            &requested_receipt_for(&input),
        );

        let receipt = receipt_from_domain_decision(&input.idempotency_key, domain_decision);
        match receipt.status {
            EvalUsecaseStatus::Evaluated => {
                self.record_event(EvalAuditEventKind::EvalEvaluated, &receipt)
            }
            EvalUsecaseStatus::Denied => {
                self.record_event(EvalAuditEventKind::EvalDenied, &receipt)
            }
        }
        self.cache_receipt(&input.idempotency_key, intent, &receipt);
        receipt
    }

    pub fn audit_events(&self) -> &[EvalAuditEvent] {
        &self.audit_events
    }

    pub fn cached_receipt_count(&self) -> usize {
        self.receipts_by_idempotency_key.len()
    }

    fn cache_receipt(
        &mut self,
        idempotency_key: &str,
        intent: EvalUsecaseIntent,
        receipt: &EvalUsecaseReceipt,
    ) {
        self.intents_by_idempotency_key
            .insert(idempotency_key.to_owned(), intent);
        self.receipts_by_idempotency_key
            .insert(idempotency_key.to_owned(), receipt.clone());
    }

    fn record_event(&mut self, kind: EvalAuditEventKind, receipt: &EvalUsecaseReceipt) {
        self.audit_events.push(EvalAuditEvent {
            kind,
            tenant_id: receipt.tenant_id.clone(),
            principal_id: receipt.principal_id.clone(),
            eval_surface: receipt.eval_surface.clone(),
            eval_set_id: receipt.eval_set_id.clone(),
            idempotency_key: receipt.idempotency_key.clone(),
            status: if kind == EvalAuditEventKind::EvalRequested {
                None
            } else {
                Some(receipt.status)
            },
            eval_set_status: receipt.eval_set_status,
            evidence_refs: sorted_unique(receipt.evidence_refs.clone()),
        });
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EvalUsecaseIntent {
    entries: Vec<String>,
}

impl EvalUsecaseIntent {
    fn from_input(input: &EvalUsecaseInput) -> Self {
        let mut entries = vec![
            canonical_entry("idempotency_key", &input.idempotency_key),
            canonical_entry("tenant_id", &input.request.tenant_id),
            canonical_entry("principal_id", &input.request.principal_id),
            canonical_entry("eval_surface", &input.request.eval_surface),
            canonical_entry("request_evidence", &input.request.request_evidence_ref),
            canonical_entry("trace_context", &input.request.trace_context_ref),
            canonical_entry("policy_decision_ref", &input.request.policy_decision_ref),
            canonical_entry("policy_id", &input.request.policy_decision.decision_id),
            canonical_entry("policy_tenant", &input.request.policy_decision.tenant_id),
            canonical_entry(
                "policy_principal",
                &input.request.policy_decision.principal_id,
            ),
            canonical_vec_entry(
                "policy_surfaces",
                &sorted_unique(input.request.policy_decision.allowed_surfaces.clone()),
            ),
            canonical_vec_entry(
                "policy_models",
                &sorted_unique(input.request.policy_decision.allowed_model_refs.clone()),
            ),
            canonical_vec_entry(
                "policy_datasets",
                &sorted_unique(
                    input
                        .request
                        .policy_decision
                        .allowed_dataset_snapshot_refs
                        .clone(),
                ),
            ),
            canonical_vec_entry(
                "policy_case_kinds",
                &case_kind_entries(&input.request.policy_decision.allowed_case_kinds),
            ),
            canonical_entry(
                "policy_min_case_count",
                &input.request.policy_decision.min_case_count.to_string(),
            ),
            canonical_entry(
                "policy_max_case_count",
                &input.request.policy_decision.max_case_count.to_string(),
            ),
            canonical_entry(
                "policy_min_pass_rate",
                &input.request.policy_decision.min_pass_rate_bps.to_string(),
            ),
            canonical_entry(
                "policy_max_safety_rate",
                &input
                    .request
                    .policy_decision
                    .max_safety_violation_rate_bps
                    .to_string(),
            ),
            canonical_entry(
                "policy_evidence",
                &input.request.policy_decision.evidence_ref,
            ),
            canonical_entry(
                "eval_registry_snapshot",
                &input.request.policy_decision.eval_registry_snapshot_ref,
            ),
            canonical_entry("eval_set_id", &input.request.eval_set.eval_set_id),
            canonical_entry("eval_set_model", &input.request.eval_set.model_ref),
            canonical_entry("eval_set_route", &input.request.eval_set.route_evidence_ref),
            canonical_entry(
                "eval_set_guardrail",
                &input.request.eval_set.guardrail_evidence_ref,
            ),
            canonical_entry(
                "eval_set_dataset",
                &input.request.eval_set.dataset_snapshot_ref,
            ),
            canonical_entry(
                "eval_set_min_pass_rate",
                &input
                    .request
                    .eval_set
                    .thresholds
                    .min_pass_rate_bps
                    .to_string(),
            ),
            canonical_entry(
                "eval_set_max_safety_rate",
                &input
                    .request
                    .eval_set
                    .thresholds
                    .max_safety_violation_rate_bps
                    .to_string(),
            ),
            canonical_entry(
                "eval_set_require_golden",
                &input.request.eval_set.thresholds.require_golden.to_string(),
            ),
            canonical_entry(
                "eval_set_require_adversarial",
                &input
                    .request
                    .eval_set
                    .thresholds
                    .require_adversarial
                    .to_string(),
            ),
            canonical_entry(
                "eval_set_require_linguistic",
                &input
                    .request
                    .eval_set
                    .thresholds
                    .require_linguistic
                    .to_string(),
            ),
            canonical_vec_entry(
                "eval_set_cases",
                &case_entries(&input.request.eval_set.cases),
            ),
        ];
        entries.sort();
        Self { entries }
    }
}

fn invalid_usecase_input_reasons(input: &EvalUsecaseInput) -> Vec<String> {
    let mut reasons = Vec::new();
    require_metadata_ref("idempotency key", &input.idempotency_key, &mut reasons);
    sorted_unique(reasons)
}

fn requested_receipt_for(input: &EvalUsecaseInput) -> EvalUsecaseReceipt {
    EvalUsecaseReceipt {
        idempotency_key: input.idempotency_key.clone(),
        tenant_id: input.request.tenant_id.clone(),
        principal_id: input.request.principal_id.clone(),
        eval_surface: input.request.eval_surface.clone(),
        eval_set_id: input.request.eval_set.eval_set_id.clone(),
        model_ref: input.request.eval_set.model_ref.clone(),
        status: EvalUsecaseStatus::Evaluated,
        denial_kind: None,
        domain_denial_kind: None,
        eval_set_status: None,
        failure_kinds: Vec::new(),
        evidence_refs: sorted_unique(vec![
            input.request.request_evidence_ref.clone(),
            input.request.trace_context_ref.clone(),
            input.request.policy_decision_ref.clone(),
        ]),
    }
}

fn receipt_from_domain_decision(
    idempotency_key: &str,
    decision: EvalDomainDecision,
) -> EvalUsecaseReceipt {
    match decision {
        EvalDomainDecision::Report(report) => EvalUsecaseReceipt {
            idempotency_key: idempotency_key.to_owned(),
            tenant_id: report.tenant_id,
            principal_id: report.principal_id,
            eval_surface: report.eval_surface,
            eval_set_id: report.eval_set_report.eval_set_id,
            model_ref: report.eval_set_report.model_ref,
            status: EvalUsecaseStatus::Evaluated,
            denial_kind: None,
            domain_denial_kind: None,
            eval_set_status: Some(report.eval_set_report.status),
            failure_kinds: report.eval_set_report.failure_kinds,
            evidence_refs: sorted_unique(report.evidence_refs),
        },
        EvalDomainDecision::Deny(denial) => receipt_from_domain_denial(
            idempotency_key,
            &denial,
            EvalUsecaseDenialKind::DomainDenied,
        ),
    }
}

fn receipt_from_domain_denial(
    idempotency_key: &str,
    denial: &intelligence_eval_domain::EvalDomainDenial,
    denial_kind: EvalUsecaseDenialKind,
) -> EvalUsecaseReceipt {
    EvalUsecaseReceipt {
        idempotency_key: idempotency_key.to_owned(),
        tenant_id: denial.tenant_id.clone(),
        principal_id: denial.principal_id.clone(),
        eval_surface: denial.eval_surface.clone(),
        eval_set_id: denial.eval_set_id.clone(),
        model_ref: denial.model_ref.clone(),
        status: EvalUsecaseStatus::Denied,
        denial_kind: Some(denial_kind),
        domain_denial_kind: Some(denial.denial_kind),
        eval_set_status: None,
        failure_kinds: Vec::new(),
        evidence_refs: sorted_unique(denial.evidence_refs.clone()),
    }
}

fn invalid_receipt_from_input(
    input: &EvalUsecaseInput,
    denial_kind: EvalUsecaseDenialKind,
    _reasons: Vec<String>,
    evidence_refs: Vec<String>,
) -> EvalUsecaseReceipt {
    EvalUsecaseReceipt {
        idempotency_key: safe_metadata(&input.idempotency_key, "redacted-invalid-idempotency-key"),
        tenant_id: safe_ref(&input.request.tenant_id, "redacted-invalid-tenant-id"),
        principal_id: safe_metadata(&input.request.principal_id, "redacted-invalid-principal-id"),
        eval_surface: safe_metadata(&input.request.eval_surface, "redacted-invalid-eval-surface"),
        eval_set_id: safe_metadata(
            &input.request.eval_set.eval_set_id,
            "redacted-invalid-eval_set-id",
        ),
        model_ref: safe_ref(
            &input.request.eval_set.model_ref,
            "redacted-invalid-model-ref",
        ),
        status: EvalUsecaseStatus::Denied,
        denial_kind: Some(denial_kind),
        domain_denial_kind: None,
        eval_set_status: None,
        failure_kinds: Vec::new(),
        evidence_refs: sorted_unique(evidence_refs),
    }
}

fn case_kind_entries(kinds: &[EvalCaseKind]) -> Vec<String> {
    let mut entries = kinds
        .iter()
        .map(|kind| format!("{kind:?}"))
        .collect::<Vec<_>>();
    entries.sort();
    entries.dedup();
    entries
}

fn case_entries(cases: &[EvalCaseResult]) -> Vec<String> {
    let mut entries = cases
        .iter()
        .map(|case| {
            [
                canonical_entry("case_id", &case.case_id),
                canonical_entry("kind", &format!("{:?}", case.kind)),
                canonical_entry("outcome", &format!("{:?}", case.outcome)),
                canonical_entry("score_bps", &case.score_bps.to_string()),
                canonical_entry("evaluator_evidence", &case.evaluator_evidence_ref),
            ]
            .concat()
        })
        .collect::<Vec<_>>();
    entries.sort();
    entries
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

fn canonical_entry(label: &str, value: &str) -> String {
    format!("{}:{}{}:{}", label.len(), label, value.len(), value)
}

fn canonical_vec_entry(label: &str, values: &[String]) -> String {
    let mut encoded = canonical_entry(label, &values.len().to_string());
    for value in values {
        encoded.push_str(&canonical_entry("item", value));
    }
    encoded
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
            route_evidence_ref: "route:evidence:eval-usecase:1".to_owned(),
            guardrail_evidence_ref: "guardrail:evidence:eval-usecase:1".to_owned(),
            dataset_snapshot_ref: "eval-dataset:snapshot:eval-usecase:1".to_owned(),
            thresholds: EvalSetThresholds {
                min_pass_rate_bps: 7_500,
                max_safety_violation_rate_bps: 0,
                require_golden: true,
                require_adversarial: true,
                require_linguistic: true,
            },
            cases: vec![
                case(
                    "case-golden-usecase-1",
                    EvalCaseKind::Golden,
                    EvalCaseOutcome::Passed,
                    9_500,
                    "eval:case:usecase:golden:1",
                ),
                case(
                    "case-adversarial-usecase-1",
                    EvalCaseKind::Adversarial,
                    EvalCaseOutcome::Passed,
                    8_900,
                    "eval:case:usecase:adversarial:1",
                ),
                case(
                    "case-linguistic-usecase-1",
                    EvalCaseKind::Linguistic,
                    EvalCaseOutcome::Passed,
                    8_400,
                    "eval:case:usecase:linguistic:1",
                ),
                case(
                    "case-regression-usecase-1",
                    EvalCaseKind::Regression,
                    EvalCaseOutcome::Failed,
                    4_000,
                    "eval:case:usecase:regression:1",
                ),
            ],
        }
    }

    fn sample_policy() -> EvalPolicyDecision {
        EvalPolicyDecision {
            decision_id: "eval-policy-decision:usecase:1".to_owned(),
            tenant_id: "tenant:alpha".to_owned(),
            principal_id: "principal:eval-owner".to_owned(),
            allowed_surfaces: vec!["surface:release-gate".to_owned()],
            allowed_model_refs: vec!["modelref://openai/gpt-preview".to_owned()],
            allowed_dataset_snapshot_refs: vec!["eval-dataset:snapshot:eval-usecase:1".to_owned()],
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
            evidence_ref: "policy:evidence:eval-usecase:1".to_owned(),
            eval_registry_snapshot_ref: "eval-registry:snapshot:usecase:1".to_owned(),
        }
    }

    fn sample_domain_request(eval_set_id: &str) -> DomainEvalSetRequest {
        DomainEvalSetRequest {
            tenant_id: "tenant:alpha".to_owned(),
            principal_id: "principal:eval-owner".to_owned(),
            eval_surface: "surface:release-gate".to_owned(),
            request_evidence_ref: "request:evidence:eval-usecase:1".to_owned(),
            trace_context_ref: "trace:eval-usecase:1".to_owned(),
            policy_decision_ref: "policy:evidence:eval-usecase:1".to_owned(),
            policy_decision: sample_policy(),
            eval_set: sample_eval_set(eval_set_id),
        }
    }

    fn sample_input(idempotency_key: &str) -> EvalUsecaseInput {
        EvalUsecaseInput {
            idempotency_key: idempotency_key.to_owned(),
            request: sample_domain_request("eval_set:usecase-pass"),
        }
    }

    #[test]
    fn evaluates_authorized_eval_with_metadata_audit() {
        let mut usecase = IntelligenceEvalUsecase::default();
        let receipt = usecase.evaluate(sample_input("idem:eval:1"));

        assert_eq!(receipt.status, EvalUsecaseStatus::Evaluated);
        assert_eq!(receipt.eval_set_status, Some(EvalSetStatus::Passed));
        assert_eq!(receipt.failure_kinds, Vec::<EvalFailureKind>::new());
        assert_eq!(usecase.cached_receipt_count(), 1);
        assert_eq!(usecase.audit_events().len(), 2);
        assert_eq!(
            usecase.audit_events()[0].kind,
            EvalAuditEventKind::EvalRequested
        );
        assert_eq!(
            usecase.audit_events()[1].kind,
            EvalAuditEventKind::EvalEvaluated
        );
        assert!(
            receipt
                .evidence_refs
                .contains(&"policy:evidence:eval-usecase:1".to_owned())
        );
        assert!(
            receipt
                .evidence_refs
                .contains(&"eval-dataset:snapshot:eval-usecase:1".to_owned())
        );
        let debug = format!("{receipt:?}{:?}", usecase.audit_events());
        assert!(!debug.contains("raw prompt"));
        assert!(!debug.contains("model answer"));
        assert!(!debug.contains("sk-"));
    }

    #[test]
    fn invalid_raw_eval_request_denies_before_audit_side_effect() {
        let mut usecase = IntelligenceEvalUsecase::default();
        let mut input = sample_input("raw prompt: write an email");
        input.request.eval_set.eval_set_id = "sk-usecase-eval_set".to_owned();
        input.request.eval_set.model_ref = "Bearer model answer".to_owned();

        let receipt = usecase.evaluate(input);
        let debug = format!("{receipt:?}");

        assert_eq!(receipt.status, EvalUsecaseStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(EvalUsecaseDenialKind::InvalidInput)
        );
        assert_eq!(receipt.idempotency_key, "redacted-invalid-idempotency-key");
        assert_eq!(receipt.eval_set_id, "redacted-invalid-eval_set-id");
        assert_eq!(receipt.model_ref, "redacted-invalid-model-ref");
        assert!(usecase.audit_events().is_empty());
        assert_eq!(usecase.cached_receipt_count(), 0);
        assert!(!debug.contains("write an email"));
        assert!(!debug.contains("sk-usecase"));
        assert!(!debug.contains("Bearer"));
        assert!(!debug.contains("model answer"));
    }

    #[test]
    fn idempotent_replay_and_conflict_are_deterministic() {
        let mut usecase = IntelligenceEvalUsecase::default();
        let first = usecase.evaluate(sample_input("idem:eval:replay"));
        let replay = usecase.evaluate(sample_input("idem:eval:replay"));

        assert_eq!(first, replay);
        assert_eq!(usecase.audit_events().len(), 2);

        let mut changed = sample_input("idem:eval:replay");
        changed.request.eval_set.thresholds.min_pass_rate_bps = 8_000;
        let conflict = usecase.evaluate(changed);

        assert_eq!(conflict.status, EvalUsecaseStatus::Denied);
        assert_eq!(
            conflict.denial_kind,
            Some(EvalUsecaseDenialKind::IdempotencyConflict)
        );
        assert!(
            usecase
                .audit_events()
                .iter()
                .any(|event| event.kind == EvalAuditEventKind::IdempotencyConflict)
        );
        assert_eq!(usecase.cached_receipt_count(), 1);
    }

    #[test]
    fn idempotent_replay_treats_case_order_as_order_independent_set() {
        let mut usecase = IntelligenceEvalUsecase::default();
        let first = usecase.evaluate(sample_input("idem:eval:case-order"));
        let mut reordered = sample_input("idem:eval:case-order");
        reordered.request.eval_set.cases.reverse();

        let replay = usecase.evaluate(reordered);

        assert_eq!(first, replay);
        assert_eq!(usecase.audit_events().len(), 2);
    }

    #[test]
    fn domain_policy_denial_records_fail_closed_audit() {
        let mut usecase = IntelligenceEvalUsecase::default();
        let mut input = sample_input("idem:eval:domain-denied");
        input.request.eval_set.model_ref = "modelref://openai/unapproved".to_owned();

        let receipt = usecase.evaluate(input);

        assert_eq!(receipt.status, EvalUsecaseStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(EvalUsecaseDenialKind::DomainDenied)
        );
        assert_eq!(
            receipt.domain_denial_kind,
            Some(EvalDomainDenialKind::ModelDenied)
        );
        assert_eq!(
            usecase.audit_events()[0].kind,
            EvalAuditEventKind::EvalRequested
        );
        assert_eq!(
            usecase.audit_events()[1].kind,
            EvalAuditEventKind::EvalDenied
        );
        assert_eq!(usecase.cached_receipt_count(), 1);
    }

    #[test]
    fn safety_failed_eval_report_records_evaluated_audit() {
        let mut usecase = IntelligenceEvalUsecase::default();
        let mut input = sample_input("idem:eval:safety-failed");
        input.request.eval_set.cases[3] = case(
            "case-safety-usecase-1",
            EvalCaseKind::Safety,
            EvalCaseOutcome::SafetyViolation,
            0,
            "eval:case:usecase:safety:1",
        );

        let receipt = usecase.evaluate(input);

        assert_eq!(receipt.status, EvalUsecaseStatus::Evaluated);
        assert_eq!(receipt.eval_set_status, Some(EvalSetStatus::Failed));
        assert!(
            receipt
                .failure_kinds
                .contains(&EvalFailureKind::SafetyViolationRateExceeded)
        );
        assert_eq!(
            usecase.audit_events()[1].kind,
            EvalAuditEventKind::EvalEvaluated
        );
    }

    #[test]
    fn domain_invalid_eval_set_metadata_denies_before_audit_side_effect() {
        let mut usecase = IntelligenceEvalUsecase::default();
        let mut input = sample_input("idem:eval:domain-invalid");
        input.request.eval_set.cases.clear();

        let receipt = usecase.evaluate(input);

        assert_eq!(receipt.status, EvalUsecaseStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(EvalUsecaseDenialKind::InvalidInput)
        );
        assert_eq!(
            receipt.domain_denial_kind,
            Some(EvalDomainDenialKind::KernelInvalid)
        );
        assert!(
            receipt
                .evidence_refs
                .contains(&"validation:intelligence-eval-kernel-input".to_owned())
        );
        assert!(usecase.audit_events().is_empty());
        assert_eq!(usecase.cached_receipt_count(), 0);
    }
}
