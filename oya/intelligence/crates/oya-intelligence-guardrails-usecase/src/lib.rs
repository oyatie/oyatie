//! Intelligence guardrails usecase foundation.
//!
//! This crate is the source-level orchestration seam for the Intelligence
//! guardrail stack. It wraps the existing domain/kernel decision logic with
//! policy-decision validation, idempotent receipts, metadata-only audit events,
//! and fail-closed sanitization. It deliberately has no classifier SDK,
//! network, filesystem, durable store, policy-engine runtime, or provider I/O.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

use intelligence_guardrails_domain::{
    DomainGuardrailRequest, GuardrailDecision, GuardrailDeny, decide_domain_guardrail,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardrailsPolicyDecision {
    pub decision_id: String,             // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub principal_id: String,            // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>,   // data_class: INTERNAL_ONLY
    pub evidence_ref: String,            // data_class: INTERNAL_ONLY
    pub classifier_snapshot_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardrailsUsecaseInput {
    pub idempotency_key: String,                // data_class: INTERNAL_ONLY
    pub principal_id: String,                   // data_class: INTERNAL_ONLY
    pub guardrail_surface: String,              // data_class: INTERNAL_ONLY
    pub request_evidence_ref: String,           // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,              // data_class: INTERNAL_ONLY
    pub domain_request: DomainGuardrailRequest, // data_class: INTERNAL_ONLY
    pub policy_decision: GuardrailsPolicyDecision, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuardrailsUsecaseStatus {
    Allowed,
    Denied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuardrailsUsecaseDenialKind {
    GuardrailDenied,
    IdempotencyConflict,
    InvalidInput,
    PolicyDrift,
    SurfaceDenied,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardrailsUsecaseReceipt {
    pub idempotency_key: String,         // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub principal_id: String,            // data_class: INTERNAL_ONLY
    pub guardrail_surface: String,       // data_class: INTERNAL_ONLY
    pub status: GuardrailsUsecaseStatus, // data_class: PUBLIC
    pub denial_kind: Option<GuardrailsUsecaseDenialKind>, // data_class: INTERNAL_ONLY
    pub refusal_reasons: Vec<String>,    // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuardrailsAuditEventKind {
    GuardrailsRequestReceived,
    GuardrailsAllowed,
    GuardrailsDenied,
    IdempotencyConflict,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardrailsAuditEvent {
    pub kind: GuardrailsAuditEventKind, // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub principal_id: String,           // data_class: INTERNAL_ONLY
    pub idempotency_key: String,        // data_class: INTERNAL_ONLY
    pub guardrail_surface: String,      // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GuardrailsUsecase {
    receipts_by_idempotency_key: BTreeMap<String, GuardrailsUsecaseReceipt>,
    intents_by_idempotency_key: BTreeMap<String, GuardrailsIntent>,
    audit_events: Vec<GuardrailsAuditEvent>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GuardrailsIntent {
    canonical: String,
}

impl GuardrailsUsecase {
    pub fn evaluate(&mut self, input: GuardrailsUsecaseInput) -> GuardrailsUsecaseReceipt {
        let invalid = invalid_input_reasons(&input);
        if !invalid.is_empty() {
            return denied_receipt(
                safe_identity(&input.idempotency_key, "redacted-invalid-idempotency-key"),
                safe_tenant(&input.domain_request.guardrail_request.tenant_id),
                safe_identity(&input.principal_id, "redacted-invalid-principal-id"),
                safe_identity(
                    &input.guardrail_surface,
                    "redacted-invalid-guardrail-surface",
                ),
                GuardrailsUsecaseDenialKind::InvalidInput,
                invalid,
                vec!["validation:intelligence-guardrails-usecase-input".to_owned()],
            );
        }

        let intent = GuardrailsIntent {
            canonical: canonical_intent(&input),
        };
        if let Some(existing) = self.receipts_by_idempotency_key.get(&input.idempotency_key) {
            if self.intents_by_idempotency_key.get(&input.idempotency_key) == Some(&intent) {
                return existing.clone();
            }
            let receipt = denied_receipt_from_validated_input(
                &input,
                GuardrailsUsecaseDenialKind::IdempotencyConflict,
                vec!["idempotency key already used for different guardrail intent".to_owned()],
                vec![
                    input.request_evidence_ref.clone(),
                    "validation:intelligence-guardrails-idempotency-conflict".to_owned(),
                ],
            );
            self.record_event(
                GuardrailsAuditEventKind::IdempotencyConflict,
                &input,
                &receipt,
            );
            return receipt;
        }

        let evidence_refs = canonical_evidence_refs(&input);
        self.record_audit_event(
            GuardrailsAuditEventKind::GuardrailsRequestReceived,
            input.domain_request.guardrail_request.tenant_id.clone(),
            input.principal_id.clone(),
            input.idempotency_key.clone(),
            input.guardrail_surface.clone(),
            evidence_refs.clone(),
        );

        if let Some(receipt) = self.policy_denial(&input, evidence_refs.clone()) {
            self.cache_receipt(&input.idempotency_key, intent, &receipt);
            return receipt;
        }

        let receipt = match decide_domain_guardrail(&input.domain_request) {
            GuardrailDecision::Allow(allow) => GuardrailsUsecaseReceipt {
                idempotency_key: input.idempotency_key.clone(),
                tenant_id: input.domain_request.guardrail_request.tenant_id.clone(),
                principal_id: input.principal_id.clone(),
                guardrail_surface: input.guardrail_surface.clone(),
                status: GuardrailsUsecaseStatus::Allowed,
                denial_kind: None,
                refusal_reasons: Vec::new(),
                evidence_refs: sorted_unique([evidence_refs, allow.evidence_refs].concat()),
            },
            GuardrailDecision::Deny(deny) => {
                self.guardrail_denied_receipt(&input, &deny, evidence_refs)
            }
        };

        let event_kind = match receipt.status {
            GuardrailsUsecaseStatus::Allowed => GuardrailsAuditEventKind::GuardrailsAllowed,
            GuardrailsUsecaseStatus::Denied => GuardrailsAuditEventKind::GuardrailsDenied,
        };
        self.record_event(event_kind, &input, &receipt);
        self.cache_receipt(&input.idempotency_key, intent, &receipt);
        receipt
    }

    pub fn audit_events(&self) -> &[GuardrailsAuditEvent] {
        &self.audit_events
    }

    pub fn receipt_count(&self) -> usize {
        self.receipts_by_idempotency_key.len()
    }

    fn policy_denial(
        &mut self,
        input: &GuardrailsUsecaseInput,
        evidence_refs: Vec<String>,
    ) -> Option<GuardrailsUsecaseReceipt> {
        let denial_kind = if input.policy_decision.tenant_id
            != input.domain_request.guardrail_request.tenant_id
            || input.policy_decision.principal_id != input.principal_id
        {
            GuardrailsUsecaseDenialKind::PolicyDrift
        } else if !input
            .policy_decision
            .allowed_surfaces
            .iter()
            .any(|surface| surface == &input.guardrail_surface)
        {
            GuardrailsUsecaseDenialKind::SurfaceDenied
        } else {
            return None;
        };

        let receipt = denied_receipt_from_validated_input(
            input,
            denial_kind,
            vec!["guardrail policy decision does not authorize this request".to_owned()],
            evidence_refs,
        );
        self.record_event(GuardrailsAuditEventKind::GuardrailsDenied, input, &receipt);
        Some(receipt)
    }

    fn guardrail_denied_receipt(
        &self,
        input: &GuardrailsUsecaseInput,
        deny: &GuardrailDeny,
        evidence_refs: Vec<String>,
    ) -> GuardrailsUsecaseReceipt {
        denied_receipt_from_validated_input(
            input,
            GuardrailsUsecaseDenialKind::GuardrailDenied,
            deny.refusal_reasons
                .iter()
                .map(|reason| safe_refusal_reason(reason))
                .collect(),
            sorted_unique([evidence_refs, deny.evidence_refs.clone()].concat()),
        )
    }

    fn cache_receipt(
        &mut self,
        idempotency_key: &str,
        intent: GuardrailsIntent,
        receipt: &GuardrailsUsecaseReceipt,
    ) {
        self.intents_by_idempotency_key
            .insert(idempotency_key.to_owned(), intent);
        self.receipts_by_idempotency_key
            .insert(idempotency_key.to_owned(), receipt.clone());
    }

    fn record_event(
        &mut self,
        kind: GuardrailsAuditEventKind,
        input: &GuardrailsUsecaseInput,
        receipt: &GuardrailsUsecaseReceipt,
    ) {
        self.record_audit_event(
            kind,
            receipt.tenant_id.clone(),
            input.principal_id.clone(),
            input.idempotency_key.clone(),
            input.guardrail_surface.clone(),
            receipt.evidence_refs.clone(),
        );
    }

    fn record_audit_event(
        &mut self,
        kind: GuardrailsAuditEventKind,
        tenant_id: String,
        principal_id: String,
        idempotency_key: String,
        guardrail_surface: String,
        evidence_refs: Vec<String>,
    ) {
        self.audit_events.push(GuardrailsAuditEvent {
            kind,
            tenant_id,
            principal_id,
            idempotency_key,
            guardrail_surface,
            evidence_refs: sorted_unique(evidence_refs),
        });
    }
}

fn invalid_input_reasons(input: &GuardrailsUsecaseInput) -> Vec<String> {
    let mut reasons = Vec::new();
    require_metadata_ref("idempotency key", &input.idempotency_key, &mut reasons);
    require_metadata_ref("principal id", &input.principal_id, &mut reasons);
    require_metadata_ref("guardrail surface", &input.guardrail_surface, &mut reasons);
    require_evidence_ref(
        "request evidence ref",
        &input.request_evidence_ref,
        &mut reasons,
    );
    require_evidence_ref("trace context ref", &input.trace_context_ref, &mut reasons);
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
    if input.policy_decision.allowed_surfaces.is_empty() {
        reasons.push("policy allowed surfaces are required".to_owned());
    }
    for surface in &input.policy_decision.allowed_surfaces {
        require_metadata_ref("policy allowed surface", surface, &mut reasons);
    }
    require_evidence_ref(
        "policy evidence ref",
        &input.policy_decision.evidence_ref,
        &mut reasons,
    );
    require_evidence_ref(
        "classifier snapshot ref",
        &input.policy_decision.classifier_snapshot_ref,
        &mut reasons,
    );

    let guardrail_request = &input.domain_request.guardrail_request;
    require_tenant(
        "request tenant id",
        &guardrail_request.tenant_id,
        &mut reasons,
    );
    require_resource_ref("content ref", &guardrail_request.content_ref, &mut reasons);
    require_evidence_ref(
        "guardrail request evidence ref",
        &guardrail_request.request_evidence_ref,
        &mut reasons,
    );
    for finding in &guardrail_request.findings {
        require_evidence_ref("finding evidence ref", &finding.evidence_ref, &mut reasons);
        if contains_raw_secret_material(&finding.reason)
            || contains_raw_content_material(&finding.reason)
        {
            reasons.push("finding reason must be audit-safe metadata".to_owned());
        }
    }
    sorted_unique(reasons)
}

fn policy_evidence_refs(input: &GuardrailsUsecaseInput) -> Vec<String> {
    vec![
        input.policy_decision.evidence_ref.clone(),
        input.policy_decision.classifier_snapshot_ref.clone(),
    ]
}

fn canonical_evidence_refs(input: &GuardrailsUsecaseInput) -> Vec<String> {
    let mut refs = vec![
        input.request_evidence_ref.clone(),
        input.trace_context_ref.clone(),
        input
            .domain_request
            .guardrail_request
            .request_evidence_ref
            .clone(),
    ];
    refs.extend(policy_evidence_refs(input));
    refs.extend(
        input
            .domain_request
            .guardrail_request
            .findings
            .iter()
            .map(|finding| finding.evidence_ref.clone()),
    );
    sorted_unique(refs)
}

fn canonical_intent(input: &GuardrailsUsecaseInput) -> String {
    let mut finding_entries: Vec<String> = input
        .domain_request
        .guardrail_request
        .findings
        .iter()
        .map(|finding| {
            [
                canonical_entry("category", &format!("{:?}", finding.category)),
                canonical_entry("risk", &format!("{:?}", finding.risk_level)),
                canonical_entry("reason", &safe_refusal_reason(&finding.reason)),
                canonical_entry("evidence", &finding.evidence_ref),
            ]
            .concat()
        })
        .collect();
    finding_entries.sort();
    [
        canonical_entry("idempotency_key", &input.idempotency_key),
        canonical_entry("principal_id", &input.principal_id),
        canonical_entry("surface", &input.guardrail_surface),
        canonical_entry("request_evidence_ref", &input.request_evidence_ref),
        canonical_entry("trace_context_ref", &input.trace_context_ref),
        canonical_entry(
            "tenant_id",
            &input.domain_request.guardrail_request.tenant_id,
        ),
        canonical_entry(
            "content_ref",
            &input.domain_request.guardrail_request.content_ref,
        ),
        canonical_entry(
            "guardrail_request_evidence_ref",
            &input.domain_request.guardrail_request.request_evidence_ref,
        ),
        canonical_entry(
            "data_class",
            &format!("{:?}", input.domain_request.data_class),
        ),
        canonical_entry("audience", &format!("{:?}", input.domain_request.audience)),
        canonical_entry("policy_decision_id", &input.policy_decision.decision_id),
        canonical_entry("policy_tenant_id", &input.policy_decision.tenant_id),
        canonical_entry("policy_principal_id", &input.policy_decision.principal_id),
        canonical_entry("policy_evidence_ref", &input.policy_decision.evidence_ref),
        canonical_entry(
            "classifier_snapshot_ref",
            &input.policy_decision.classifier_snapshot_ref,
        ),
        canonical_vec_entry("policy_surfaces", &input.policy_decision.allowed_surfaces),
        canonical_vec_entry("findings", &finding_entries),
    ]
    .concat()
}

fn denied_receipt_from_validated_input(
    input: &GuardrailsUsecaseInput,
    denial_kind: GuardrailsUsecaseDenialKind,
    refusal_reasons: Vec<String>,
    evidence_refs: Vec<String>,
) -> GuardrailsUsecaseReceipt {
    denied_receipt(
        input.idempotency_key.clone(),
        input.domain_request.guardrail_request.tenant_id.clone(),
        input.principal_id.clone(),
        input.guardrail_surface.clone(),
        denial_kind,
        refusal_reasons,
        evidence_refs,
    )
}

fn denied_receipt(
    idempotency_key: String,
    tenant_id: String,
    principal_id: String,
    guardrail_surface: String,
    denial_kind: GuardrailsUsecaseDenialKind,
    refusal_reasons: Vec<String>,
    evidence_refs: Vec<String>,
) -> GuardrailsUsecaseReceipt {
    GuardrailsUsecaseReceipt {
        idempotency_key,
        tenant_id,
        principal_id,
        guardrail_surface,
        status: GuardrailsUsecaseStatus::Denied,
        denial_kind: Some(denial_kind),
        refusal_reasons: sorted_unique(refusal_reasons),
        evidence_refs: sorted_unique(evidence_refs),
    }
}

fn require_tenant(label: &str, value: &str, reasons: &mut Vec<String>) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        reasons.push(format!("{label} is required"));
    } else if value != trimmed
        || !trimmed.starts_with("ten_")
        || contains_whitespace(trimmed)
        || trimmed.contains('/')
        || contains_raw_secret_material(trimmed)
        || contains_raw_content_material(trimmed)
    {
        reasons.push(format!("{label} is invalid"));
    }
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
    let trimmed = value.trim();
    if trimmed.is_empty() {
        reasons.push(format!("{label} is required"));
    } else if value != trimmed
        || contains_whitespace(trimmed)
        || contains_raw_secret_material(trimmed)
        || contains_raw_content_material(trimmed)
        || !trimmed.contains(':')
    {
        reasons.push(format!("{label} must be an opaque evidence ref"));
    }
}

fn require_resource_ref(label: &str, value: &str, reasons: &mut Vec<String>) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        reasons.push(format!("{label} is required"));
    } else if value != trimmed
        || contains_whitespace(trimmed)
        || contains_raw_secret_material(trimmed)
        || contains_raw_content_material(trimmed)
        || !trimmed.contains(':')
    {
        reasons.push(format!("{label} must be an opaque resource ref"));
    }
}

fn safe_identity(value: &str, fallback: &str) -> String {
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

fn safe_tenant(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || value != trimmed
        || !trimmed.starts_with("ten_")
        || contains_whitespace(trimmed)
        || trimmed.contains('/')
        || contains_raw_secret_material(trimmed)
        || contains_raw_content_material(trimmed)
    {
        "redacted-invalid-tenant-id".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn safe_refusal_reason(reason: &str) -> String {
    let trimmed = reason.trim();
    if trimmed.is_empty()
        || contains_raw_secret_material(trimmed)
        || contains_raw_content_material(trimmed)
    {
        "guardrail denied closed".to_owned()
    } else {
        trimmed.to_owned()
    }
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
        || lower.contains("raw model")
        || lower.contains("write an email")
        || lower.contains("customer message")
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
    use intelligence_guardrails_domain::{
        GuardrailAudience, GuardrailCategory, GuardrailDataClass, GuardrailFinding,
        GuardrailRequest, RiskLevel,
    };

    fn sample_input(idempotency_key: &str) -> GuardrailsUsecaseInput {
        GuardrailsUsecaseInput {
            idempotency_key: idempotency_key.to_owned(),
            principal_id: "principal-1".to_owned(),
            guardrail_surface: "intelligence.dispatch.pre".to_owned(),
            request_evidence_ref: "req:guardrails".to_owned(),
            trace_context_ref: "trace:guardrails".to_owned(),
            domain_request: DomainGuardrailRequest {
                guardrail_request: GuardrailRequest {
                    tenant_id: "ten_a".to_owned(),
                    content_ref: "contentref://prompt/1".to_owned(),
                    findings: vec![GuardrailFinding {
                        category: GuardrailCategory::Benign,
                        risk_level: RiskLevel::Low,
                        reason: "benign".to_owned(),
                        evidence_ref: "classifier:benign".to_owned(),
                    }],
                    request_evidence_ref: "guardrail:req:1".to_owned(),
                },
                data_class: GuardrailDataClass::InternalOnly,
                audience: GuardrailAudience::TenantOperator,
            },
            policy_decision: GuardrailsPolicyDecision {
                decision_id: "policy-decision-1".to_owned(),
                tenant_id: "ten_a".to_owned(),
                principal_id: "principal-1".to_owned(),
                allowed_surfaces: vec!["intelligence.dispatch.pre".to_owned()],
                evidence_ref: "cedar:guardrails:allow".to_owned(),
                classifier_snapshot_ref: "classifier:snapshot:1".to_owned(),
            },
        }
    }

    #[test]
    fn evaluates_allowed_guardrail_request_with_metadata_audit() {
        let mut usecase = GuardrailsUsecase::default();

        let receipt = usecase.evaluate(sample_input("idem-1"));

        assert_eq!(receipt.status, GuardrailsUsecaseStatus::Allowed);
        assert_eq!(receipt.refusal_reasons, Vec::<String>::new());
        assert!(
            receipt
                .evidence_refs
                .contains(&"classifier:benign".to_owned())
        );
        assert_eq!(usecase.audit_events().len(), 2);
        let debug = format!("{receipt:?}{:?}", usecase.audit_events());
        assert!(!debug.contains("write an email"));
        assert!(!debug.contains("sk-"));
    }

    #[test]
    fn unsafe_finding_reason_denies_before_audit_side_effect() {
        let mut usecase = GuardrailsUsecase::default();
        let mut input = sample_input("idem-deny");
        input.domain_request.guardrail_request.findings = vec![GuardrailFinding {
            category: GuardrailCategory::PromptInjection,
            risk_level: RiskLevel::High,
            reason: "write an email to the customer".to_owned(),
            evidence_ref: "classifier:prompt-injection".to_owned(),
        }];

        let receipt = usecase.evaluate(input);
        let debug = format!("{receipt:?}");

        assert_eq!(receipt.status, GuardrailsUsecaseStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(GuardrailsUsecaseDenialKind::InvalidInput)
        );
        assert!(usecase.audit_events().is_empty());
        assert!(!debug.contains("write an email"));
    }

    #[test]
    fn high_risk_guardrail_denial_sorts_metadata_and_records_denial_audit() {
        let mut usecase = GuardrailsUsecase::default();
        let mut input = sample_input("idem-high-risk");
        input.domain_request.guardrail_request.findings = vec![GuardrailFinding {
            category: GuardrailCategory::PromptInjection,
            risk_level: RiskLevel::High,
            reason: "prompt injection attempt requires refusal".to_owned(),
            evidence_ref: "classifier:prompt-injection".to_owned(),
        }];

        let receipt = usecase.evaluate(input);

        assert_eq!(receipt.status, GuardrailsUsecaseStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(GuardrailsUsecaseDenialKind::GuardrailDenied)
        );
        assert_eq!(
            receipt.refusal_reasons,
            vec!["prompt injection attempt requires refusal".to_owned()]
        );
        assert!(
            receipt
                .evidence_refs
                .contains(&"classifier:prompt-injection".to_owned())
        );
        assert!(
            usecase
                .audit_events()
                .iter()
                .any(|event| event.kind == GuardrailsAuditEventKind::GuardrailsDenied)
        );
    }

    #[test]
    fn idempotent_replay_and_conflict_are_deterministic() {
        let mut usecase = GuardrailsUsecase::default();

        let first = usecase.evaluate(sample_input("idem-1"));
        let replay = usecase.evaluate(sample_input("idem-1"));
        let mut changed = sample_input("idem-1");
        changed.policy_decision.allowed_surfaces = vec!["intelligence.dispatch.post".to_owned()];
        let conflict = usecase.evaluate(changed);

        assert_eq!(first, replay);
        assert_eq!(conflict.status, GuardrailsUsecaseStatus::Denied);
        assert_eq!(
            conflict.denial_kind,
            Some(GuardrailsUsecaseDenialKind::IdempotencyConflict)
        );
        assert_eq!(usecase.receipt_count(), 1);
    }

    #[test]
    fn idempotent_replay_conflicts_on_policy_binding_drift() {
        let mut usecase = GuardrailsUsecase::default();

        let first = usecase.evaluate(sample_input("idem-policy-drift"));
        let mut drift = sample_input("idem-policy-drift");
        drift.policy_decision.tenant_id = "ten_other".to_owned();
        let conflict = usecase.evaluate(drift);

        assert_eq!(first.status, GuardrailsUsecaseStatus::Allowed);
        assert_eq!(conflict.status, GuardrailsUsecaseStatus::Denied);
        assert_eq!(
            conflict.denial_kind,
            Some(GuardrailsUsecaseDenialKind::IdempotencyConflict)
        );
        assert_eq!(usecase.receipt_count(), 1);
    }

    #[test]
    fn idempotent_replay_treats_findings_as_an_order_independent_set() {
        let mut usecase = GuardrailsUsecase::default();
        let first_finding = GuardrailFinding {
            category: GuardrailCategory::Benign,
            risk_level: RiskLevel::Low,
            reason: "benign".to_owned(),
            evidence_ref: "classifier:benign".to_owned(),
        };
        let second_finding = GuardrailFinding {
            category: GuardrailCategory::Violence,
            risk_level: RiskLevel::Medium,
            reason: "medium risk violence classifier metadata".to_owned(),
            evidence_ref: "classifier:violence-medium".to_owned(),
        };
        let mut first = sample_input("idem-order-independent");
        first.domain_request.guardrail_request.findings =
            vec![first_finding.clone(), second_finding.clone()];
        let mut replay = sample_input("idem-order-independent");
        replay.domain_request.guardrail_request.findings = vec![second_finding, first_finding];

        let first_receipt = usecase.evaluate(first);
        let replay_receipt = usecase.evaluate(replay);

        assert_eq!(first_receipt, replay_receipt);
        assert_eq!(usecase.receipt_count(), 1);
    }

    #[test]
    fn policy_drift_and_surface_denial_block_before_domain_decision() {
        let mut usecase = GuardrailsUsecase::default();
        let mut drift = sample_input("idem-drift");
        drift.policy_decision.tenant_id = "ten_other".to_owned();
        let mut surface = sample_input("idem-surface");
        surface.policy_decision.allowed_surfaces = vec!["intelligence.dispatch.post".to_owned()];

        let drift_receipt = usecase.evaluate(drift);
        let surface_receipt = usecase.evaluate(surface);

        assert_eq!(
            drift_receipt.denial_kind,
            Some(GuardrailsUsecaseDenialKind::PolicyDrift)
        );
        assert_eq!(
            surface_receipt.denial_kind,
            Some(GuardrailsUsecaseDenialKind::SurfaceDenied)
        );
    }

    #[test]
    fn invalid_secret_shaped_identity_is_redacted_before_audit_side_effect() {
        let mut usecase = GuardrailsUsecase::default();
        let mut bad = sample_input("sk-test-idem");
        bad.principal_id = "Bearer principal token".to_owned();
        bad.domain_request.guardrail_request.tenant_id = "sk-test-tenant".to_owned();

        let receipt = usecase.evaluate(bad);
        let debug = format!("{receipt:?}{:?}", usecase.audit_events());

        assert_eq!(receipt.status, GuardrailsUsecaseStatus::Denied);
        assert_eq!(receipt.idempotency_key, "redacted-invalid-idempotency-key");
        assert_eq!(receipt.principal_id, "redacted-invalid-principal-id");
        assert_eq!(receipt.tenant_id, "redacted-invalid-tenant-id");
        assert!(usecase.audit_events().is_empty());
        assert!(!debug.contains("sk-test"));
        assert!(!debug.contains("Bearer"));
    }

    #[test]
    fn missing_classifier_output_denies_closed() {
        let mut usecase = GuardrailsUsecase::default();
        let mut input = sample_input("idem-missing-classifier");
        input.domain_request.guardrail_request.findings.clear();

        let receipt = usecase.evaluate(input);

        assert_eq!(receipt.status, GuardrailsUsecaseStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(GuardrailsUsecaseDenialKind::GuardrailDenied)
        );
        assert!(
            receipt
                .refusal_reasons
                .contains(&"guardrail classification missing; request denied closed".to_owned())
        );
    }
}
