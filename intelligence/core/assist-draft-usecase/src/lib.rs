//! Intelligence assist-draft usecase foundation.
//!
//! This crate orchestrates metadata-only no-code-builder draft planning for
//! later cloud integration. It preserves in-memory idempotent receipts and
//! metadata-only audit events around the assist-draft domain layer while
//! performing no prompt rendering, model calls, provider dispatch, builder
//! mutation, filesystem/network I/O, durable idempotency storage, durable audit
//! emission, or queue/runtime processing.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

pub use intelligence_assist_draft_domain::{
    AssistDraftAction, AssistDraftAudience, AssistDraftBuilderSurface, AssistDraftDataClass,
    AssistDraftDenialReason, AssistDraftDomainDecision, AssistDraftDomainDenialKind,
    AssistDraftDomainStatus, AssistDraftInvocationMode, AssistDraftKind, AssistDraftPolicyDecision,
    AssistDraftRequest, AssistDraftReviewGate, DomainAssistDraftRequest, plan_domain_assist_draft,
};

const USECASE_MAX_PROMPT_CONTEXT_REFS: usize = 16;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftUsecaseInput {
    pub idempotency_key: String,           // data_class: INTERNAL_ONLY
    pub request: DomainAssistDraftRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AssistDraftUsecaseStatus {
    Denied,
    Planned,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AssistDraftUsecaseDenialKind {
    DomainDenied,
    IdempotencyConflict,
    InvalidInput,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftUsecaseReceipt {
    pub idempotency_key: String,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub principal_id: String,              // data_class: INTERNAL_ONLY
    pub brand_surface_ref: String,         // data_class: INTERNAL_ONLY
    pub locale: String,                    // data_class: INTERNAL_ONLY
    pub target_builder_ref: String,        // data_class: INTERNAL_ONLY
    pub output_contract_ref: String,       // data_class: INTERNAL_ONLY
    pub cost_floor_disclosure_ref: String, // data_class: INTERNAL_ONLY
    pub status: AssistDraftUsecaseStatus,  // data_class: PUBLIC
    pub denial_kind: Option<AssistDraftUsecaseDenialKind>, // data_class: INTERNAL_ONLY
    pub domain_denial_kind: Option<AssistDraftDomainDenialKind>, // data_class: INTERNAL_ONLY
    pub kernel_reasons: Vec<AssistDraftDenialReason>, // data_class: INTERNAL_ONLY
    pub denial_reasons: Vec<String>,       // data_class: INTERNAL_ONLY
    pub planned_actions: Vec<AssistDraftAction>, // data_class: INTERNAL_ONLY
    pub refusal_banner: Option<String>,    // data_class: PUBLIC
    pub evidence_refs: Vec<String>,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AssistDraftAuditEventKind {
    AssistDraftDenied,
    AssistDraftPlanned,
    AssistDraftRequested,
    IdempotencyConflict,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftAuditEvent {
    pub kind: AssistDraftAuditEventKind, // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub principal_id: String,            // data_class: INTERNAL_ONLY
    pub brand_surface_ref: String,       // data_class: INTERNAL_ONLY
    pub locale: String,                  // data_class: INTERNAL_ONLY
    pub target_builder_ref: String,      // data_class: INTERNAL_ONLY
    pub idempotency_key: String,         // data_class: INTERNAL_ONLY
    pub status: Option<AssistDraftUsecaseStatus>, // data_class: PUBLIC
    pub planned_action_count: Option<usize>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,      // data_class: INTERNAL_ONLY
}

#[derive(Default)]
pub struct IntelligenceAssistDraftUsecase {
    receipts_by_idempotency_key: BTreeMap<String, AssistDraftUsecaseReceipt>,
    intents_by_idempotency_key: BTreeMap<String, AssistDraftIntent>,
    audit_events: Vec<AssistDraftAuditEvent>,
}

impl IntelligenceAssistDraftUsecase {
    pub fn plan(&mut self, input: AssistDraftUsecaseInput) -> AssistDraftUsecaseReceipt {
        let invalid = invalid_usecase_input_reasons(&input);
        if !invalid.is_empty() {
            return invalid_receipt_from_input(
                &input,
                AssistDraftUsecaseDenialKind::InvalidInput,
                None,
                Vec::new(),
                invalid,
                vec!["validation:intelligence-assist-draft-usecase-input".to_owned()],
            );
        }

        let intent = AssistDraftIntent::from_input(&input);
        if let Some(existing) = self.receipts_by_idempotency_key.get(&input.idempotency_key) {
            if self.intents_by_idempotency_key.get(&input.idempotency_key) == Some(&intent) {
                return existing.clone();
            }
            let receipt = invalid_receipt_from_input(
                &input,
                AssistDraftUsecaseDenialKind::IdempotencyConflict,
                None,
                Vec::new(),
                vec!["idempotency key already used for different assist-draft intent".to_owned()],
                vec![
                    input.request.request.request_evidence_ref.clone(),
                    "validation:intelligence-assist-draft-idempotency-conflict".to_owned(),
                ],
            );
            self.record_event(AssistDraftAuditEventKind::IdempotencyConflict, &receipt);
            return receipt;
        }

        let domain_decision = plan_domain_assist_draft(input.request.clone());
        if let AssistDraftDomainDecision::Deny(denial) = &domain_decision
            && (denial.denial_kind == AssistDraftDomainDenialKind::InvalidInput
                || denial
                    .kernel_reasons
                    .contains(&AssistDraftDenialReason::InvalidInput))
        {
            return receipt_from_domain_denial(
                &input.idempotency_key,
                denial,
                AssistDraftUsecaseDenialKind::InvalidInput,
            );
        }

        self.record_event(
            AssistDraftAuditEventKind::AssistDraftRequested,
            &requested_receipt_for(&input),
        );

        let receipt = receipt_from_domain_decision(&input, domain_decision);
        match receipt.status {
            AssistDraftUsecaseStatus::Planned => {
                self.record_event(AssistDraftAuditEventKind::AssistDraftPlanned, &receipt)
            }
            AssistDraftUsecaseStatus::Denied => {
                self.record_event(AssistDraftAuditEventKind::AssistDraftDenied, &receipt)
            }
        }
        self.cache_receipt(&input.idempotency_key, intent, &receipt);
        receipt
    }

    pub fn audit_events(&self) -> &[AssistDraftAuditEvent] {
        &self.audit_events
    }

    pub fn cached_receipt_count(&self) -> usize {
        self.receipts_by_idempotency_key.len()
    }

    fn cache_receipt(
        &mut self,
        idempotency_key: &str,
        intent: AssistDraftIntent,
        receipt: &AssistDraftUsecaseReceipt,
    ) {
        self.intents_by_idempotency_key
            .insert(idempotency_key.to_owned(), intent);
        self.receipts_by_idempotency_key
            .insert(idempotency_key.to_owned(), receipt.clone());
    }

    fn record_event(
        &mut self,
        kind: AssistDraftAuditEventKind,
        receipt: &AssistDraftUsecaseReceipt,
    ) {
        self.audit_events.push(AssistDraftAuditEvent {
            kind,
            tenant_id: receipt.tenant_id.clone(),
            principal_id: receipt.principal_id.clone(),
            brand_surface_ref: receipt.brand_surface_ref.clone(),
            locale: receipt.locale.clone(),
            target_builder_ref: receipt.target_builder_ref.clone(),
            idempotency_key: receipt.idempotency_key.clone(),
            status: if kind == AssistDraftAuditEventKind::AssistDraftRequested {
                None
            } else {
                Some(receipt.status)
            },
            planned_action_count: if kind == AssistDraftAuditEventKind::AssistDraftRequested {
                None
            } else {
                Some(receipt.planned_actions.len())
            },
            evidence_refs: sorted_unique(receipt.evidence_refs.clone()),
        });
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AssistDraftIntent {
    entries: Vec<String>,
}

impl AssistDraftIntent {
    fn from_input(input: &AssistDraftUsecaseInput) -> Self {
        let domain = &input.request;
        let policy = &domain.policy_decision;
        let request = &domain.request;
        let mut entries = vec![
            canonical_entry("idempotency_key", &input.idempotency_key),
            canonical_entry("principal_id", &domain.principal_id),
            canonical_entry("brand_surface_ref", &domain.brand_surface_ref),
            canonical_entry("locale", &domain.locale),
            canonical_vec_entry(
                "prompt_context_refs",
                &sorted_unique(domain.prompt_context_refs.clone()),
            ),
            canonical_entry("policy_decision_id", &policy.decision_id),
            canonical_entry("policy_tenant", &policy.tenant_id),
            canonical_entry("policy_principal", &policy.principal_id),
            canonical_entry("policy_ai_enabled", &policy.ai_assist_enabled.to_string()),
            canonical_entry(
                "policy_explicit_automation_allowed",
                &policy.explicit_automation_allowed.to_string(),
            ),
            canonical_vec_entry(
                "policy_surfaces",
                &surface_entries(&policy.allowed_builder_surfaces),
            ),
            canonical_vec_entry(
                "policy_draft_kinds",
                &kind_entries(&policy.allowed_draft_kinds),
            ),
            canonical_vec_entry(
                "policy_audiences",
                &audience_entries(&policy.allowed_audiences),
            ),
            canonical_vec_entry(
                "policy_data_classes",
                &data_class_entries(&policy.allowed_data_classes),
            ),
            canonical_vec_entry("policy_actions", &action_entries(&policy.allowed_actions)),
            canonical_vec_entry(
                "policy_locales",
                &sorted_unique(policy.allowed_locales.clone()),
            ),
            canonical_entry(
                "policy_max_prompt_context_refs",
                &policy.max_prompt_context_refs.to_string(),
            ),
            canonical_entry("policy_evidence_ref", &policy.evidence_ref),
            canonical_entry(
                "prompt_registry_snapshot_ref",
                &policy.prompt_registry_snapshot_ref,
            ),
            canonical_entry(
                "cost_floor_disclosure_ref",
                &policy.cost_floor_disclosure_ref,
            ),
            canonical_entry(
                "builder_capability_scope_ref",
                &policy.builder_capability_scope_ref,
            ),
            canonical_entry("tenant_id", &request.tenant_id),
            canonical_entry("request_principal", &request.principal_id),
            canonical_entry("context_id", &request.context_id),
            canonical_entry("builder_surface", &format!("{:?}", request.builder_surface)),
            canonical_entry("draft_kind", &format!("{:?}", request.draft_kind)),
            canonical_entry("audience", &format!("{:?}", request.audience)),
            canonical_entry("invocation_mode", &format!("{:?}", request.invocation_mode)),
            canonical_entry("review_gate", &format!("{:?}", request.review_gate)),
            canonical_entry("prompt_ref", &request.prompt_ref),
            canonical_entry("target_builder_ref", &request.target_builder_ref),
            canonical_entry("output_contract_ref", &request.output_contract_ref),
            canonical_entry("consent_grant_ref", &request.consent_grant_ref),
            canonical_entry("budget_evidence_ref", &request.budget_evidence_ref),
            canonical_entry("policy_decision_ref", &request.policy_decision_ref),
            canonical_entry("model_route_ref", &request.model_route_ref),
            canonical_entry("guardrail_evidence_ref", &request.guardrail_evidence_ref),
            canonical_entry("request_evidence_ref", &request.request_evidence_ref),
            canonical_entry("trace_context_ref", &request.trace_context_ref),
            canonical_vec_entry(
                "request_data_classes",
                &data_class_entries(&request.data_classes),
            ),
            canonical_vec_entry(
                "request_actions",
                &action_entries(&request.requested_actions),
            ),
            canonical_vec_entry(
                "additional_evidence_refs",
                &sorted_unique(request.additional_evidence_refs.clone()),
            ),
        ];
        entries.sort();
        Self { entries }
    }
}

fn receipt_from_domain_decision(
    input: &AssistDraftUsecaseInput,
    decision: AssistDraftDomainDecision,
) -> AssistDraftUsecaseReceipt {
    match decision {
        AssistDraftDomainDecision::Plan(plan) => AssistDraftUsecaseReceipt {
            idempotency_key: input.idempotency_key.clone(),
            tenant_id: plan.tenant_id,
            principal_id: plan.principal_id,
            brand_surface_ref: plan.brand_surface_ref,
            locale: plan.locale,
            target_builder_ref: plan.kernel_plan.target_builder_ref,
            output_contract_ref: plan.kernel_plan.output_contract_ref,
            cost_floor_disclosure_ref: plan.cost_floor_disclosure_ref,
            status: AssistDraftUsecaseStatus::Planned,
            denial_kind: None,
            domain_denial_kind: None,
            kernel_reasons: Vec::new(),
            denial_reasons: Vec::new(),
            planned_actions: sorted_unique_actions(plan.kernel_plan.allowed_actions),
            refusal_banner: None,
            evidence_refs: sorted_unique(plan.evidence_refs),
        },
        AssistDraftDomainDecision::Deny(denial) => receipt_from_domain_denial(
            &input.idempotency_key,
            &denial,
            AssistDraftUsecaseDenialKind::DomainDenied,
        ),
    }
}

fn requested_receipt_for(input: &AssistDraftUsecaseInput) -> AssistDraftUsecaseReceipt {
    AssistDraftUsecaseReceipt {
        idempotency_key: input.idempotency_key.clone(),
        tenant_id: input.request.request.tenant_id.clone(),
        principal_id: input.request.principal_id.clone(),
        brand_surface_ref: input.request.brand_surface_ref.clone(),
        locale: input.request.locale.clone(),
        target_builder_ref: input.request.request.target_builder_ref.clone(),
        output_contract_ref: input.request.request.output_contract_ref.clone(),
        cost_floor_disclosure_ref: input
            .request
            .policy_decision
            .cost_floor_disclosure_ref
            .clone(),
        status: AssistDraftUsecaseStatus::Planned,
        denial_kind: None,
        domain_denial_kind: None,
        kernel_reasons: Vec::new(),
        denial_reasons: Vec::new(),
        planned_actions: Vec::new(),
        refusal_banner: None,
        evidence_refs: sorted_unique(vec![
            input.request.request.request_evidence_ref.clone(),
            input.request.request.trace_context_ref.clone(),
            input.request.policy_decision.evidence_ref.clone(),
        ]),
    }
}

fn receipt_from_domain_denial(
    idempotency_key: &str,
    denial: &intelligence_assist_draft_domain::AssistDraftDomainDenial,
    usecase_denial_kind: AssistDraftUsecaseDenialKind,
) -> AssistDraftUsecaseReceipt {
    AssistDraftUsecaseReceipt {
        idempotency_key: safe_ref(idempotency_key, "redacted-invalid-idempotency-key"),
        tenant_id: denial.tenant_id.clone(),
        principal_id: denial.principal_id.clone(),
        brand_surface_ref: denial.brand_surface_ref.clone(),
        locale: denial.locale.clone(),
        target_builder_ref: "redacted-unplanned-target-builder-ref".to_owned(),
        output_contract_ref: "redacted-unplanned-output-contract-ref".to_owned(),
        cost_floor_disclosure_ref: "redacted-unplanned-cost-floor-disclosure-ref".to_owned(),
        status: AssistDraftUsecaseStatus::Denied,
        denial_kind: Some(usecase_denial_kind),
        domain_denial_kind: Some(denial.denial_kind),
        kernel_reasons: denial.kernel_reasons.clone(),
        denial_reasons: sorted_unique(denial.reasons.clone()),
        planned_actions: Vec::new(),
        refusal_banner: Some(denial.refusal_banner.clone()),
        evidence_refs: sorted_unique(denial.evidence_refs.clone()),
    }
}

fn invalid_receipt_from_input(
    input: &AssistDraftUsecaseInput,
    denial_kind: AssistDraftUsecaseDenialKind,
    domain_denial_kind: Option<AssistDraftDomainDenialKind>,
    kernel_reasons: Vec<AssistDraftDenialReason>,
    reasons: Vec<String>,
    evidence_refs: Vec<String>,
) -> AssistDraftUsecaseReceipt {
    AssistDraftUsecaseReceipt {
        idempotency_key: safe_ref(&input.idempotency_key, "redacted-invalid-idempotency-key"),
        tenant_id: safe_ref(
            &input.request.request.tenant_id,
            "redacted-invalid-tenant-id",
        ),
        principal_id: safe_ref(&input.request.principal_id, "redacted-invalid-principal-id"),
        brand_surface_ref: safe_ref(
            &input.request.brand_surface_ref,
            "redacted-invalid-brand-surface-ref",
        ),
        locale: safe_locale(&input.request.locale),
        target_builder_ref: safe_ref(
            &input.request.request.target_builder_ref,
            "redacted-invalid-target-builder-ref",
        ),
        output_contract_ref: safe_ref(
            &input.request.request.output_contract_ref,
            "redacted-invalid-output-contract-ref",
        ),
        cost_floor_disclosure_ref: safe_ref(
            &input.request.policy_decision.cost_floor_disclosure_ref,
            "redacted-invalid-cost-floor-disclosure-ref",
        ),
        status: AssistDraftUsecaseStatus::Denied,
        denial_kind: Some(denial_kind),
        domain_denial_kind,
        kernel_reasons,
        denial_reasons: sorted_unique(reasons),
        planned_actions: Vec::new(),
        refusal_banner: Some("AI draft assistance is unavailable for this request.".to_owned()),
        evidence_refs: sorted_unique(evidence_refs),
    }
}

fn invalid_usecase_input_reasons(input: &AssistDraftUsecaseInput) -> Vec<String> {
    let mut reasons = Vec::new();
    require_ref("idempotency key", &input.idempotency_key, &mut reasons);
    require_ref("principal id", &input.request.principal_id, &mut reasons);
    require_ref(
        "brand surface ref",
        &input.request.brand_surface_ref,
        &mut reasons,
    );
    require_locale("locale", &input.request.locale, &mut reasons);
    if input.request.prompt_context_refs.len() > USECASE_MAX_PROMPT_CONTEXT_REFS {
        reasons.push(format!(
            "prompt context refs must be less than or equal to {USECASE_MAX_PROMPT_CONTEXT_REFS}"
        ));
    }
    for prompt_context_ref in &input.request.prompt_context_refs {
        require_ref("prompt context ref", prompt_context_ref, &mut reasons);
    }

    let policy = &input.request.policy_decision;
    require_ref("policy decision id", &policy.decision_id, &mut reasons);
    require_ref("policy tenant id", &policy.tenant_id, &mut reasons);
    require_ref("policy principal id", &policy.principal_id, &mut reasons);
    require_nonempty(
        "policy allowed builder surfaces",
        &policy.allowed_builder_surfaces,
        &mut reasons,
    );
    require_nonempty(
        "policy allowed draft kinds",
        &policy.allowed_draft_kinds,
        &mut reasons,
    );
    require_nonempty(
        "policy allowed audiences",
        &policy.allowed_audiences,
        &mut reasons,
    );
    require_nonempty(
        "policy allowed data classes",
        &policy.allowed_data_classes,
        &mut reasons,
    );
    require_nonempty(
        "policy allowed actions",
        &policy.allowed_actions,
        &mut reasons,
    );
    if policy.allowed_locales.is_empty() {
        reasons.push("policy allowed locales are required".to_owned());
    }
    for locale in &policy.allowed_locales {
        require_locale("policy allowed locale", locale, &mut reasons);
    }
    if policy.max_prompt_context_refs == 0
        || policy.max_prompt_context_refs > USECASE_MAX_PROMPT_CONTEXT_REFS
    {
        reasons.push(format!(
            "policy max prompt context refs must be in 1..={USECASE_MAX_PROMPT_CONTEXT_REFS}"
        ));
    }
    require_ref("policy evidence ref", &policy.evidence_ref, &mut reasons);
    require_ref(
        "prompt registry snapshot ref",
        &policy.prompt_registry_snapshot_ref,
        &mut reasons,
    );
    require_ref(
        "cost floor disclosure ref",
        &policy.cost_floor_disclosure_ref,
        &mut reasons,
    );
    require_ref(
        "builder capability scope ref",
        &policy.builder_capability_scope_ref,
        &mut reasons,
    );

    let request = &input.request.request;
    require_ref("request tenant id", &request.tenant_id, &mut reasons);
    require_ref("request principal id", &request.principal_id, &mut reasons);
    require_ref("context id", &request.context_id, &mut reasons);
    require_ref("prompt ref", &request.prompt_ref, &mut reasons);
    require_ref(
        "target builder ref",
        &request.target_builder_ref,
        &mut reasons,
    );
    require_ref(
        "output contract ref",
        &request.output_contract_ref,
        &mut reasons,
    );
    require_ref(
        "consent grant ref",
        &request.consent_grant_ref,
        &mut reasons,
    );
    require_ref(
        "budget evidence ref",
        &request.budget_evidence_ref,
        &mut reasons,
    );
    require_ref(
        "policy decision ref",
        &request.policy_decision_ref,
        &mut reasons,
    );
    require_ref("model route ref", &request.model_route_ref, &mut reasons);
    require_ref(
        "guardrail evidence ref",
        &request.guardrail_evidence_ref,
        &mut reasons,
    );
    require_ref(
        "request evidence ref",
        &request.request_evidence_ref,
        &mut reasons,
    );
    require_ref(
        "trace context ref",
        &request.trace_context_ref,
        &mut reasons,
    );
    if request.data_classes.is_empty() {
        reasons.push("request data classes are required".to_owned());
    }
    if request.requested_actions.is_empty() {
        reasons.push("request actions are required".to_owned());
    }
    for evidence_ref in &request.additional_evidence_refs {
        require_ref("additional evidence ref", evidence_ref, &mut reasons);
    }
    sorted_unique(reasons)
}

fn require_nonempty<T>(name: &str, values: &[T], reasons: &mut Vec<String>) {
    if values.is_empty() {
        reasons.push(format!("{name} are required"));
    }
}

fn require_ref(name: &str, value: &str, reasons: &mut Vec<String>) {
    if !is_safe_ref(value) {
        reasons.push(format!("{name} must be an opaque metadata ref"));
    }
}

fn require_locale(name: &str, value: &str, reasons: &mut Vec<String>) {
    if !is_safe_locale(value) {
        reasons.push(format!("{name} must be a safe locale tag"));
    }
}

fn safe_ref(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if is_safe_ref(trimmed) && trimmed == value {
        trimmed.to_owned()
    } else {
        fallback.to_owned()
    }
}

fn safe_locale(value: &str) -> String {
    let trimmed = value.trim();
    if is_safe_locale(trimmed) && trimmed == value {
        trimmed.to_owned()
    } else {
        "und".to_owned()
    }
}

fn is_safe_ref(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
        && trimmed.contains(':')
        && !contains_whitespace(trimmed)
        && !contains_raw_secret_material(trimmed)
        && !contains_raw_content_material(trimmed)
}

fn is_safe_locale(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
        && !contains_whitespace(trimmed)
        && !contains_raw_secret_material(trimmed)
        && !contains_raw_content_material(trimmed)
        && trimmed
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
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
        || lower.contains("secret=")
}

fn contains_raw_content_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw output")
        || lower.contains("raw query")
        || lower.contains("customer message")
        || lower.contains("write an email")
        || lower.contains("model answer")
        || lower.contains("prompt=")
        || lower.contains("completion=")
        || lower.contains("document raw")
        || lower.contains("document=")
        || lower.contains("suggested_patch=")
        || lower.contains("full answer text")
}

fn sorted_unique(values: Vec<String>) -> Vec<String> {
    let mut set = BTreeSet::new();
    for value in values {
        if !value.trim().is_empty()
            && !contains_raw_secret_material(&value)
            && !contains_raw_content_material(&value)
        {
            set.insert(value);
        }
    }
    set.into_iter().collect()
}

fn sorted_unique_actions(mut values: Vec<AssistDraftAction>) -> Vec<AssistDraftAction> {
    values.sort();
    values.dedup();
    values
}

fn canonical_entry(key: &str, value: &str) -> String {
    format!("{key}={value}")
}

fn canonical_vec_entry(key: &str, values: &[String]) -> String {
    format!("{key}=[{}]", values.join(","))
}

fn action_entries(values: &[AssistDraftAction]) -> Vec<String> {
    let mut entries = values
        .iter()
        .map(|value| format!("{value:?}"))
        .collect::<Vec<_>>();
    entries.sort();
    entries.dedup();
    entries
}

fn audience_entries(values: &[AssistDraftAudience]) -> Vec<String> {
    let mut entries = values
        .iter()
        .map(|value| format!("{value:?}"))
        .collect::<Vec<_>>();
    entries.sort();
    entries.dedup();
    entries
}

fn data_class_entries(values: &[AssistDraftDataClass]) -> Vec<String> {
    let mut entries = values
        .iter()
        .map(|value| format!("{value:?}"))
        .collect::<Vec<_>>();
    entries.sort();
    entries.dedup();
    entries
}

fn kind_entries(values: &[AssistDraftKind]) -> Vec<String> {
    let mut entries = values
        .iter()
        .map(|value| format!("{value:?}"))
        .collect::<Vec<_>>();
    entries.sort();
    entries.dedup();
    entries
}

fn surface_entries(values: &[AssistDraftBuilderSurface]) -> Vec<String> {
    let mut entries = values
        .iter()
        .map(|value| format!("{value:?}"))
        .collect::<Vec<_>>();
    entries.sort();
    entries.dedup();
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_authorized_draft_with_metadata_audit_and_cost_disclosure() {
        let mut usecase = IntelligenceAssistDraftUsecase::default();

        let receipt = usecase.plan(valid_usecase_input());

        assert_eq!(receipt.status, AssistDraftUsecaseStatus::Planned);
        assert_eq!(receipt.denial_kind, None);
        assert_eq!(receipt.domain_denial_kind, None);
        assert_eq!(receipt.tenant_id, "tenant:alpha");
        assert_eq!(receipt.principal_id, "principal:builder-owner");
        assert_eq!(receipt.locale, "en-US");
        assert_eq!(
            receipt.cost_floor_disclosure_ref,
            "cost-floor:assist-draft:workflow-studio"
        );
        assert_eq!(
            receipt.planned_actions,
            vec![
                AssistDraftAction::CreateDraft,
                AssistDraftAction::ExplainDraft
            ]
        );
        assert_eq!(receipt.refusal_banner, None);
        assert!(
            receipt
                .evidence_refs
                .contains(&"policy:assist-draft:allow".to_owned())
        );
        assert!(
            receipt
                .evidence_refs
                .contains(&"request:assist-draft:1".to_owned())
        );
        assert_eq!(usecase.cached_receipt_count(), 1);
        assert_eq!(usecase.audit_events().len(), 2);
        assert_eq!(
            usecase.audit_events()[0].kind,
            AssistDraftAuditEventKind::AssistDraftRequested
        );
        assert_eq!(usecase.audit_events()[0].status, None);
        assert_eq!(
            usecase.audit_events()[1].kind,
            AssistDraftAuditEventKind::AssistDraftPlanned
        );
        assert_eq!(
            usecase.audit_events()[1].status,
            Some(AssistDraftUsecaseStatus::Planned)
        );
    }

    #[test]
    fn idempotent_replay_returns_cached_receipt_without_second_audit() {
        let mut usecase = IntelligenceAssistDraftUsecase::default();
        let first = usecase.plan(valid_usecase_input());
        let mut replay = valid_usecase_input();
        replay.request.request.requested_actions.reverse();
        replay.request.request.data_classes.reverse();
        replay.request.prompt_context_refs.reverse();

        let second = usecase.plan(replay);

        assert_eq!(second, first);
        assert_eq!(usecase.cached_receipt_count(), 1);
        assert_eq!(usecase.audit_events().len(), 2);
    }

    #[test]
    fn idempotency_conflict_denies_without_replacing_original_receipt() {
        let mut usecase = IntelligenceAssistDraftUsecase::default();
        let first = usecase.plan(valid_usecase_input());
        let mut conflicting = valid_usecase_input();
        conflicting.request.locale = "ko-KR".to_owned();
        conflicting
            .request
            .policy_decision
            .allowed_locales
            .push("ko-KR".to_owned());

        let conflict = usecase.plan(conflicting);

        assert_eq!(conflict.status, AssistDraftUsecaseStatus::Denied);
        assert_eq!(
            conflict.denial_kind,
            Some(AssistDraftUsecaseDenialKind::IdempotencyConflict)
        );
        assert_eq!(usecase.cached_receipt_count(), 1);
        assert_eq!(usecase.plan(valid_usecase_input()), first);
        assert!(
            usecase
                .audit_events()
                .iter()
                .any(|event| event.kind == AssistDraftAuditEventKind::IdempotencyConflict)
        );
    }

    #[test]
    fn domain_policy_denial_records_fail_closed_audit() {
        let mut usecase = IntelligenceAssistDraftUsecase::default();
        let mut input = valid_usecase_input();
        input.request.policy_decision.allowed_actions = vec![AssistDraftAction::ExplainDraft];

        let receipt = usecase.plan(input);

        assert_eq!(receipt.status, AssistDraftUsecaseStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(AssistDraftUsecaseDenialKind::DomainDenied)
        );
        assert_eq!(
            receipt.domain_denial_kind,
            Some(AssistDraftDomainDenialKind::ActionDenied)
        );
        assert_eq!(
            receipt.kernel_reasons,
            Vec::<AssistDraftDenialReason>::new()
        );
        assert!(
            receipt
                .refusal_banner
                .expect("banner")
                .contains("not allowed")
        );
        assert_eq!(
            usecase.audit_events()[0].kind,
            AssistDraftAuditEventKind::AssistDraftRequested
        );
        assert_eq!(
            usecase.audit_events()[1].kind,
            AssistDraftAuditEventKind::AssistDraftDenied
        );
    }

    #[test]
    fn kernel_sensitive_external_denial_records_domain_denied_audit() {
        let mut usecase = IntelligenceAssistDraftUsecase::default();
        let mut input = valid_usecase_input();
        input.request.request.audience = AssistDraftAudience::ExternalEndUser;
        input.request.request.data_classes = vec![
            AssistDraftDataClass::Public,
            AssistDraftDataClass::Financial,
        ];
        input
            .request
            .policy_decision
            .allowed_audiences
            .push(AssistDraftAudience::ExternalEndUser);
        input
            .request
            .policy_decision
            .allowed_data_classes
            .push(AssistDraftDataClass::Financial);

        let receipt = usecase.plan(input);

        assert_eq!(receipt.status, AssistDraftUsecaseStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(AssistDraftUsecaseDenialKind::DomainDenied)
        );
        assert_eq!(
            receipt.domain_denial_kind,
            Some(AssistDraftDomainDenialKind::KernelDenied)
        );
        assert_eq!(
            receipt.kernel_reasons,
            vec![AssistDraftDenialReason::SensitiveExternalDraftDenied]
        );
        assert!(
            receipt
                .evidence_refs
                .contains(&"policy:assist-draft:allow".to_owned())
        );
    }

    #[test]
    fn invalid_raw_metadata_denies_before_audit_or_cache_side_effects() {
        let mut usecase = IntelligenceAssistDraftUsecase::default();
        let mut input = valid_usecase_input();
        input.idempotency_key = "sk-secret".to_owned();
        input.request.request.prompt_ref = "raw prompt: write an email with secret".to_owned();
        input
            .request
            .prompt_context_refs
            .push("document=raw output".to_owned());

        let receipt = usecase.plan(input);

        assert_eq!(receipt.status, AssistDraftUsecaseStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(AssistDraftUsecaseDenialKind::InvalidInput)
        );
        assert_eq!(usecase.cached_receipt_count(), 0);
        assert!(usecase.audit_events().is_empty());
        let debug = format!("{receipt:?}");
        assert!(!debug.contains("sk-secret"));
        assert!(!debug.contains("raw prompt"));
        assert!(!debug.contains("write an email"));
        assert!(!debug.contains("document=raw output"));
    }

    #[test]
    fn invalid_domain_shaped_input_does_not_cache_or_audit() {
        let mut usecase = IntelligenceAssistDraftUsecase::default();
        let mut input = valid_usecase_input();
        input.request.locale = " ".to_owned();

        let receipt = usecase.plan(input);

        assert_eq!(receipt.status, AssistDraftUsecaseStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(AssistDraftUsecaseDenialKind::InvalidInput)
        );
        assert_eq!(receipt.domain_denial_kind, None);
        assert_eq!(usecase.cached_receipt_count(), 0);
        assert!(usecase.audit_events().is_empty());
    }

    fn valid_usecase_input() -> AssistDraftUsecaseInput {
        AssistDraftUsecaseInput {
            idempotency_key: "idempotency:assist-draft:1".to_owned(),
            request: DomainAssistDraftRequest {
                principal_id: "principal:builder-owner".to_owned(),
                brand_surface_ref: "brand-surface:workflow-studio:assist".to_owned(),
                locale: "en-US".to_owned(),
                prompt_context_refs: vec!["context-snippet:workflow-studio:canvas-1".to_owned()],
                policy_decision: AssistDraftPolicyDecision {
                    decision_id: "decision:assist-draft:1".to_owned(),
                    tenant_id: "tenant:alpha".to_owned(),
                    principal_id: "principal:builder-owner".to_owned(),
                    ai_assist_enabled: true,
                    explicit_automation_allowed: false,
                    allowed_builder_surfaces: vec![AssistDraftBuilderSurface::WorkflowStudio],
                    allowed_draft_kinds: vec![AssistDraftKind::WorkflowDraft],
                    allowed_audiences: vec![AssistDraftAudience::TenantBuilder],
                    allowed_data_classes: vec![
                        AssistDraftDataClass::Internal,
                        AssistDraftDataClass::Public,
                    ],
                    allowed_actions: vec![
                        AssistDraftAction::CreateDraft,
                        AssistDraftAction::ExplainDraft,
                    ],
                    allowed_locales: vec!["en-US".to_owned()],
                    max_prompt_context_refs: 4,
                    evidence_ref: "policy:assist-draft:allow".to_owned(),
                    prompt_registry_snapshot_ref: "prompt-registry:assist-draft:v1".to_owned(),
                    cost_floor_disclosure_ref: "cost-floor:assist-draft:workflow-studio".to_owned(),
                    builder_capability_scope_ref: "builder-scope:workflow-studio:draft".to_owned(),
                },
                request: AssistDraftRequest {
                    tenant_id: "tenant:alpha".to_owned(),
                    principal_id: "principal:builder-owner".to_owned(),
                    context_id: "context://workflow-studio/canvas-1".to_owned(),
                    builder_surface: AssistDraftBuilderSurface::WorkflowStudio,
                    draft_kind: AssistDraftKind::WorkflowDraft,
                    audience: AssistDraftAudience::TenantBuilder,
                    invocation_mode: AssistDraftInvocationMode::UserInvoked,
                    review_gate: AssistDraftReviewGate::HumanReviewRequired,
                    prompt_ref: "prompt://assist-draft/req-1".to_owned(),
                    target_builder_ref: "builder://workflow-studio/canvas-1".to_owned(),
                    output_contract_ref: "workflow-spec://contracts/v1".to_owned(),
                    consent_grant_ref: "consent:assist-draft:1".to_owned(),
                    budget_evidence_ref: "budget:assist-draft:1".to_owned(),
                    policy_decision_ref: "policy:assist-draft:allow".to_owned(),
                    model_route_ref: "model-route:assist-draft:1".to_owned(),
                    guardrail_evidence_ref: "guardrail:assist-draft:allow".to_owned(),
                    request_evidence_ref: "request:assist-draft:1".to_owned(),
                    trace_context_ref: "trace:assist-draft:1".to_owned(),
                    data_classes: vec![
                        AssistDraftDataClass::Internal,
                        AssistDraftDataClass::Public,
                    ],
                    requested_actions: vec![
                        AssistDraftAction::CreateDraft,
                        AssistDraftAction::ExplainDraft,
                    ],
                    additional_evidence_refs: Vec::new(),
                },
            },
        }
    }
}
