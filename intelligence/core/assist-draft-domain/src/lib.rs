//! Intelligence assist-draft domain foundation.
//!
//! The domain layer binds metadata-only no-code-builder draft planning to
//! tenant/principal policy authority, locale, brand refusal banner, cost-floor
//! disclosure, and builder capability scope. It performs no prompt rendering,
//! model calls, provider dispatch, network I/O, filesystem access, durable
//! storage, builder mutation, or policy-engine runtime execution.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

pub use intelligence_assist_draft_kernel::{
    AssistDraftAction, AssistDraftAudience, AssistDraftBuilderSurface, AssistDraftDataClass,
    AssistDraftDecision, AssistDraftDenialReason, AssistDraftInvocationMode, AssistDraftKind,
    AssistDraftPlan, AssistDraftRequest, AssistDraftReviewGate, AssistDraftStatus,
    decide_assist_draft,
};

const DOMAIN_MAX_PROMPT_CONTEXT_REFS: usize = 16;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftPolicyDecision {
    pub decision_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub principal_id: String,              // data_class: INTERNAL_ONLY
    pub ai_assist_enabled: bool,           // data_class: INTERNAL_ONLY
    pub explicit_automation_allowed: bool, // data_class: INTERNAL_ONLY
    pub allowed_builder_surfaces: Vec<AssistDraftBuilderSurface>, // data_class: INTERNAL_ONLY
    pub allowed_draft_kinds: Vec<AssistDraftKind>, // data_class: INTERNAL_ONLY
    pub allowed_audiences: Vec<AssistDraftAudience>, // data_class: INTERNAL_ONLY
    pub allowed_data_classes: Vec<AssistDraftDataClass>, // data_class: INTERNAL_ONLY
    pub allowed_actions: Vec<AssistDraftAction>, // data_class: INTERNAL_ONLY
    pub allowed_locales: Vec<String>,      // data_class: INTERNAL_ONLY
    pub max_prompt_context_refs: usize,    // data_class: INTERNAL_ONLY
    pub evidence_ref: String,              // data_class: INTERNAL_ONLY
    pub prompt_registry_snapshot_ref: String, // data_class: INTERNAL_ONLY
    pub cost_floor_disclosure_ref: String, // data_class: INTERNAL_ONLY
    pub builder_capability_scope_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainAssistDraftRequest {
    pub principal_id: String,                       // data_class: INTERNAL_ONLY
    pub brand_surface_ref: String,                  // data_class: INTERNAL_ONLY
    pub locale: String,                             // data_class: INTERNAL_ONLY
    pub prompt_context_refs: Vec<String>,           // data_class: INTERNAL_ONLY
    pub policy_decision: AssistDraftPolicyDecision, // data_class: INTERNAL_ONLY
    pub request: AssistDraftRequest,                // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssistDraftDomainStatus {
    Denied,
    Planned,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AssistDraftDomainDenialKind {
    ActionDenied,
    AiAssistDisabled,
    AudienceDenied,
    AutoApplyDenied,
    BuilderSurfaceDenied,
    DataClassDenied,
    DraftKindDenied,
    InvalidInput,
    KernelDenied,
    LocaleDenied,
    PolicyDrift,
    PromptContextLimitExceeded,
    AutomationDenied,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftDomainPlan {
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub principal_id: String,              // data_class: INTERNAL_ONLY
    pub brand_surface_ref: String,         // data_class: INTERNAL_ONLY
    pub locale: String,                    // data_class: INTERNAL_ONLY
    pub cost_floor_disclosure_ref: String, // data_class: INTERNAL_ONLY
    pub status: AssistDraftDomainStatus,   // data_class: PUBLIC
    pub kernel_plan: AssistDraftPlan,      // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftDomainDenial {
    pub tenant_id: String,                            // data_class: INTERNAL_ONLY
    pub principal_id: String,                         // data_class: INTERNAL_ONLY
    pub brand_surface_ref: String,                    // data_class: INTERNAL_ONLY
    pub locale: String,                               // data_class: INTERNAL_ONLY
    pub status: AssistDraftDomainStatus,              // data_class: PUBLIC
    pub denial_kind: AssistDraftDomainDenialKind,     // data_class: INTERNAL_ONLY
    pub kernel_reasons: Vec<AssistDraftDenialReason>, // data_class: INTERNAL_ONLY
    pub refusal_banner: String,                       // data_class: PUBLIC
    pub reasons: Vec<String>,                         // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssistDraftDomainDecision {
    Plan(AssistDraftDomainPlan),
    Deny(AssistDraftDomainDenial),
}

impl AssistDraftDomainDecision {
    pub fn status(&self) -> AssistDraftDomainStatus {
        match self {
            Self::Plan(plan) => plan.status,
            Self::Deny(denial) => denial.status,
        }
    }

    pub fn plan(&self) -> Option<&AssistDraftDomainPlan> {
        match self {
            Self::Plan(plan) => Some(plan),
            Self::Deny(_) => None,
        }
    }

    pub fn denial(&self) -> Option<&AssistDraftDomainDenial> {
        match self {
            Self::Plan(_) => None,
            Self::Deny(denial) => Some(denial),
        }
    }
}

pub fn plan_domain_assist_draft(input: DomainAssistDraftRequest) -> AssistDraftDomainDecision {
    let invalid = invalid_input_reasons(&input);
    if !invalid.is_empty() {
        return domain_denial_from_parts(DomainDenialParts {
            tenant_id: safe_ref(&input.request.tenant_id, "redacted-invalid-tenant-id"),
            principal_id: safe_ref(&input.principal_id, "redacted-invalid-principal-id"),
            brand_surface_ref: safe_ref(
                &input.brand_surface_ref,
                "redacted-invalid-brand-surface-ref",
            ),
            locale: safe_locale(&input.locale),
            denial_kind: AssistDraftDomainDenialKind::InvalidInput,
            kernel_reasons: Vec::new(),
            reasons: invalid,
            evidence_refs: vec!["validation:intelligence-assist-draft-domain-input".to_owned()],
        });
    }

    if policy_binding_drifted(&input) {
        return domain_denial(
            &input,
            AssistDraftDomainDenialKind::PolicyDrift,
            Vec::new(),
            vec![
                "assist-draft policy decision is not bound to request tenant/principal/evidence"
                    .to_owned(),
            ],
            vec![
                input.request.request_evidence_ref.clone(),
                input.request.policy_decision_ref.clone(),
                input.policy_decision.evidence_ref.clone(),
                "validation:intelligence-assist-draft-policy-drift".to_owned(),
            ],
        );
    }

    if !input.policy_decision.ai_assist_enabled {
        return domain_denial(
            &input,
            AssistDraftDomainDenialKind::AiAssistDisabled,
            Vec::new(),
            vec!["AI draft assistance is disabled by tenant policy".to_owned()],
            policy_evidence_refs(&input),
        );
    }

    if auto_apply_requested(&input.request) {
        return domain_denial(
            &input,
            AssistDraftDomainDenialKind::AutoApplyDenied,
            Vec::new(),
            vec!["AI draft output must remain human-review-only".to_owned()],
            policy_evidence_refs(&input),
        );
    }

    if input.request.invocation_mode == AssistDraftInvocationMode::ShadowLoop
        || (input.request.invocation_mode == AssistDraftInvocationMode::TenantConfiguredAutomation
            && !input.policy_decision.explicit_automation_allowed)
    {
        return domain_denial(
            &input,
            AssistDraftDomainDenialKind::AutomationDenied,
            Vec::new(),
            vec!["AI draft invocation is not explicitly allowed by tenant policy".to_owned()],
            policy_evidence_refs(&input),
        );
    }

    if !input
        .policy_decision
        .allowed_builder_surfaces
        .contains(&input.request.builder_surface)
    {
        return domain_denial(
            &input,
            AssistDraftDomainDenialKind::BuilderSurfaceDenied,
            Vec::new(),
            vec!["assist-draft policy decision does not allow this builder surface".to_owned()],
            policy_evidence_refs(&input),
        );
    }

    if !input
        .policy_decision
        .allowed_draft_kinds
        .contains(&input.request.draft_kind)
    {
        return domain_denial(
            &input,
            AssistDraftDomainDenialKind::DraftKindDenied,
            Vec::new(),
            vec!["assist-draft policy decision does not allow this draft kind".to_owned()],
            policy_evidence_refs(&input),
        );
    }

    if !input
        .policy_decision
        .allowed_audiences
        .contains(&input.request.audience)
    {
        return domain_denial(
            &input,
            AssistDraftDomainDenialKind::AudienceDenied,
            Vec::new(),
            vec!["assist-draft policy decision does not allow this audience".to_owned()],
            policy_evidence_refs(&input),
        );
    }

    if !input.request.data_classes.iter().all(|data_class| {
        input
            .policy_decision
            .allowed_data_classes
            .contains(data_class)
    }) {
        return domain_denial(
            &input,
            AssistDraftDomainDenialKind::DataClassDenied,
            Vec::new(),
            vec!["assist-draft request data classes exceed policy decision".to_owned()],
            policy_evidence_refs(&input),
        );
    }

    if !input
        .request
        .requested_actions
        .iter()
        .all(|action| input.policy_decision.allowed_actions.contains(action))
    {
        return domain_denial(
            &input,
            AssistDraftDomainDenialKind::ActionDenied,
            Vec::new(),
            vec![
                "assist-draft requested actions are not allowed for this no-code builder"
                    .to_owned(),
            ],
            policy_evidence_refs(&input),
        );
    }

    if !input
        .policy_decision
        .allowed_locales
        .iter()
        .any(|allowed| allowed == &input.locale)
    {
        return domain_denial(
            &input,
            AssistDraftDomainDenialKind::LocaleDenied,
            Vec::new(),
            vec!["assist-draft locale is not available under this policy".to_owned()],
            policy_evidence_refs(&input),
        );
    }

    if input.prompt_context_refs.len() > input.policy_decision.max_prompt_context_refs {
        return domain_denial(
            &input,
            AssistDraftDomainDenialKind::PromptContextLimitExceeded,
            Vec::new(),
            vec!["assist-draft prompt context refs exceed policy cap".to_owned()],
            policy_evidence_refs(&input),
        );
    }

    match decide_assist_draft(input.request.clone()) {
        AssistDraftDecision::Plan(kernel_plan) => {
            let evidence_refs = sorted_unique(
                [
                    kernel_plan.evidence_refs.clone(),
                    policy_evidence_refs(&input),
                ]
                .concat(),
            );
            AssistDraftDomainDecision::Plan(AssistDraftDomainPlan {
                tenant_id: input.request.tenant_id,
                principal_id: input.principal_id,
                brand_surface_ref: input.brand_surface_ref,
                locale: input.locale,
                cost_floor_disclosure_ref: input.policy_decision.cost_floor_disclosure_ref.clone(),
                status: AssistDraftDomainStatus::Planned,
                evidence_refs,
                kernel_plan,
            })
        }
        AssistDraftDecision::Deny(denial) => domain_denial(
            &input,
            AssistDraftDomainDenialKind::KernelDenied,
            denial.reasons,
            vec!["assist-draft kernel denied advisory draft planning".to_owned()],
            sorted_unique([denial.evidence_refs, policy_evidence_refs(&input)].concat()),
        ),
    }
}

fn invalid_input_reasons(input: &DomainAssistDraftRequest) -> Vec<String> {
    let mut reasons = Vec::new();

    require_opaque("principal id", &input.principal_id, &mut reasons);
    require_opaque("brand surface ref", &input.brand_surface_ref, &mut reasons);
    require_locale("locale", &input.locale, &mut reasons);
    if input.prompt_context_refs.len() > DOMAIN_MAX_PROMPT_CONTEXT_REFS {
        reasons.push(format!(
            "prompt context refs must be less than or equal to {DOMAIN_MAX_PROMPT_CONTEXT_REFS}"
        ));
    }
    for prompt_context_ref in &input.prompt_context_refs {
        require_opaque("prompt context ref", prompt_context_ref, &mut reasons);
    }

    require_opaque(
        "policy decision id",
        &input.policy_decision.decision_id,
        &mut reasons,
    );
    require_opaque(
        "policy tenant id",
        &input.policy_decision.tenant_id,
        &mut reasons,
    );
    require_opaque(
        "policy principal id",
        &input.policy_decision.principal_id,
        &mut reasons,
    );
    require_nonempty(
        "policy allowed builder surfaces",
        &input.policy_decision.allowed_builder_surfaces,
        &mut reasons,
    );
    require_nonempty(
        "policy allowed draft kinds",
        &input.policy_decision.allowed_draft_kinds,
        &mut reasons,
    );
    require_nonempty(
        "policy allowed audiences",
        &input.policy_decision.allowed_audiences,
        &mut reasons,
    );
    require_nonempty(
        "policy allowed data classes",
        &input.policy_decision.allowed_data_classes,
        &mut reasons,
    );
    require_nonempty(
        "policy allowed actions",
        &input.policy_decision.allowed_actions,
        &mut reasons,
    );
    if input.policy_decision.allowed_locales.is_empty() {
        reasons.push("policy allowed locales are required".to_owned());
    }
    for locale in &input.policy_decision.allowed_locales {
        require_locale("policy allowed locale", locale, &mut reasons);
    }
    if input.policy_decision.max_prompt_context_refs == 0 {
        reasons.push("policy max prompt context refs must be greater than zero".to_owned());
    } else if input.policy_decision.max_prompt_context_refs > DOMAIN_MAX_PROMPT_CONTEXT_REFS {
        reasons.push(format!(
            "policy max prompt context refs must be less than or equal to {DOMAIN_MAX_PROMPT_CONTEXT_REFS}"
        ));
    }
    require_opaque(
        "policy evidence ref",
        &input.policy_decision.evidence_ref,
        &mut reasons,
    );
    require_opaque(
        "prompt registry snapshot ref",
        &input.policy_decision.prompt_registry_snapshot_ref,
        &mut reasons,
    );
    require_opaque(
        "cost floor disclosure ref",
        &input.policy_decision.cost_floor_disclosure_ref,
        &mut reasons,
    );
    require_opaque(
        "builder capability scope ref",
        &input.policy_decision.builder_capability_scope_ref,
        &mut reasons,
    );

    require_opaque("request tenant id", &input.request.tenant_id, &mut reasons);
    require_opaque(
        "request principal id",
        &input.request.principal_id,
        &mut reasons,
    );
    require_opaque("context id", &input.request.context_id, &mut reasons);
    require_opaque("prompt ref", &input.request.prompt_ref, &mut reasons);
    require_opaque(
        "target builder ref",
        &input.request.target_builder_ref,
        &mut reasons,
    );
    require_opaque(
        "output contract ref",
        &input.request.output_contract_ref,
        &mut reasons,
    );
    require_opaque(
        "consent grant ref",
        &input.request.consent_grant_ref,
        &mut reasons,
    );
    require_opaque(
        "budget evidence ref",
        &input.request.budget_evidence_ref,
        &mut reasons,
    );
    require_opaque(
        "request policy decision ref",
        &input.request.policy_decision_ref,
        &mut reasons,
    );
    require_opaque(
        "model route ref",
        &input.request.model_route_ref,
        &mut reasons,
    );
    require_opaque(
        "guardrail evidence ref",
        &input.request.guardrail_evidence_ref,
        &mut reasons,
    );
    require_opaque(
        "request evidence ref",
        &input.request.request_evidence_ref,
        &mut reasons,
    );
    require_opaque(
        "trace context ref",
        &input.request.trace_context_ref,
        &mut reasons,
    );
    if input.request.data_classes.is_empty() {
        reasons.push("request data classes are required".to_owned());
    }
    if input.request.requested_actions.is_empty() {
        reasons.push("request actions are required".to_owned());
    }
    for evidence_ref in &input.request.additional_evidence_refs {
        require_opaque("additional evidence ref", evidence_ref, &mut reasons);
    }

    sorted_unique(reasons)
}

fn policy_binding_drifted(input: &DomainAssistDraftRequest) -> bool {
    input.policy_decision.tenant_id != input.request.tenant_id
        || input.policy_decision.principal_id != input.principal_id
        || input.request.principal_id != input.principal_id
        || input.request.policy_decision_ref != input.policy_decision.evidence_ref
}

fn auto_apply_requested(request: &AssistDraftRequest) -> bool {
    request.review_gate == AssistDraftReviewGate::AutoApplyRequested
        || request.requested_actions.iter().any(|action| {
            matches!(
                action,
                AssistDraftAction::ApplyDraft | AssistDraftAction::ActivateDraft
            )
        })
}

fn policy_evidence_refs(input: &DomainAssistDraftRequest) -> Vec<String> {
    sorted_unique(vec![
        input.policy_decision.evidence_ref.clone(),
        input.policy_decision.prompt_registry_snapshot_ref.clone(),
        input.policy_decision.cost_floor_disclosure_ref.clone(),
    ])
}

struct DomainDenialParts {
    tenant_id: String,
    principal_id: String,
    brand_surface_ref: String,
    locale: String,
    denial_kind: AssistDraftDomainDenialKind,
    kernel_reasons: Vec<AssistDraftDenialReason>,
    reasons: Vec<String>,
    evidence_refs: Vec<String>,
}

fn domain_denial(
    input: &DomainAssistDraftRequest,
    denial_kind: AssistDraftDomainDenialKind,
    kernel_reasons: Vec<AssistDraftDenialReason>,
    reasons: Vec<String>,
    evidence_refs: Vec<String>,
) -> AssistDraftDomainDecision {
    domain_denial_from_parts(DomainDenialParts {
        tenant_id: safe_ref(&input.request.tenant_id, "redacted-invalid-tenant-id"),
        principal_id: safe_ref(&input.principal_id, "redacted-invalid-principal-id"),
        brand_surface_ref: safe_ref(
            &input.brand_surface_ref,
            "redacted-invalid-brand-surface-ref",
        ),
        locale: safe_locale(&input.locale),
        denial_kind,
        kernel_reasons,
        reasons,
        evidence_refs,
    })
}

fn domain_denial_from_parts(parts: DomainDenialParts) -> AssistDraftDomainDecision {
    AssistDraftDomainDecision::Deny(AssistDraftDomainDenial {
        tenant_id: parts.tenant_id,
        principal_id: parts.principal_id,
        brand_surface_ref: parts.brand_surface_ref,
        locale: parts.locale,
        status: AssistDraftDomainStatus::Denied,
        refusal_banner: refusal_banner(parts.denial_kind),
        denial_kind: parts.denial_kind,
        kernel_reasons: parts.kernel_reasons,
        reasons: sorted_unique(parts.reasons),
        evidence_refs: sorted_unique(parts.evidence_refs),
    })
}

fn refusal_banner(denial_kind: AssistDraftDomainDenialKind) -> String {
    match denial_kind {
        AssistDraftDomainDenialKind::ActionDenied => {
            "AI draft action is not allowed for this no-code builder.".to_owned()
        }
        AssistDraftDomainDenialKind::AutoApplyDenied => {
            "AI draft assistance requires human review before activation.".to_owned()
        }
        AssistDraftDomainDenialKind::LocaleDenied => {
            "AI draft assistance is not available for this locale.".to_owned()
        }
        AssistDraftDomainDenialKind::AiAssistDisabled => {
            "AI draft assistance is disabled by tenant policy.".to_owned()
        }
        _ => "AI draft assistance is unavailable for this request.".to_owned(),
    }
}

fn require_nonempty<T>(name: &str, values: &[T], reasons: &mut Vec<String>) {
    if values.is_empty() {
        reasons.push(format!("{name} are required"));
    }
}

fn require_opaque(name: &str, value: &str, reasons: &mut Vec<String>) {
    if !is_safe_opaque_ref(value) {
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
    if is_safe_opaque_ref(trimmed) && trimmed == value {
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

fn is_safe_opaque_ref(value: &str) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_policy_bound_workflow_draft_with_locale_cost_disclosure_and_kernel_plan() {
        let decision = plan_domain_assist_draft(valid_domain_request());

        assert_eq!(decision.status(), AssistDraftDomainStatus::Planned);
        let plan = decision.plan().expect("planned");
        assert_eq!(plan.tenant_id, "tenant:alpha");
        assert_eq!(plan.principal_id, "principal:builder-owner");
        assert_eq!(plan.locale, "en-US");
        assert_eq!(
            plan.brand_surface_ref,
            "brand-surface:workflow-studio:assist"
        );
        assert_eq!(
            plan.cost_floor_disclosure_ref,
            "cost-floor:assist-draft:workflow-studio"
        );
        assert_eq!(plan.kernel_plan.status, AssistDraftStatus::Planned);
        assert_eq!(
            plan.kernel_plan.review_gate,
            AssistDraftReviewGate::HumanReviewRequired
        );
        assert_eq!(
            plan.kernel_plan.allowed_actions,
            vec![
                AssistDraftAction::CreateDraft,
                AssistDraftAction::ExplainDraft
            ]
        );
        assert_eq!(
            plan.evidence_refs,
            vec![
                "budget:assist-draft:1".to_owned(),
                "consent:assist-draft:1".to_owned(),
                "cost-floor:assist-draft:workflow-studio".to_owned(),
                "guardrail:assist-draft:allow".to_owned(),
                "model-route:assist-draft:1".to_owned(),
                "policy:assist-draft:allow".to_owned(),
                "prompt-registry:assist-draft:v1".to_owned(),
                "request:assist-draft:1".to_owned(),
                "trace:assist-draft:1".to_owned(),
            ]
        );
    }

    #[test]
    fn rejects_builder_prompt_without_tenant_context_before_kernel() {
        let mut input = valid_domain_request();
        input.request.context_id.clear();
        input.principal_id.clear();
        input.locale = " ".to_owned();

        let decision = plan_domain_assist_draft(input);

        assert_eq!(decision.status(), AssistDraftDomainStatus::Denied);
        let denial = decision.denial().expect("denied");
        assert_eq!(
            denial.denial_kind,
            AssistDraftDomainDenialKind::InvalidInput
        );
        assert_eq!(denial.locale, "und");
        assert_eq!(
            denial.refusal_banner,
            "AI draft assistance is unavailable for this request."
        );
        assert!(
            denial
                .reasons
                .iter()
                .any(|reason| reason.contains("context id"))
        );
        assert!(
            denial
                .reasons
                .iter()
                .any(|reason| reason.contains("principal id"))
        );
    }

    #[test]
    fn policy_drift_blocks_unbound_tenant_principal_or_policy_evidence() {
        let mut input = valid_domain_request();
        input.policy_decision.tenant_id = "tenant:other".to_owned();
        input.request.policy_decision_ref = "policy:assist-draft:stale".to_owned();

        let decision = plan_domain_assist_draft(input);

        let denial = decision.denial().expect("denied");
        assert_eq!(denial.denial_kind, AssistDraftDomainDenialKind::PolicyDrift);
        assert_eq!(denial.kernel_reasons, Vec::<AssistDraftDenialReason>::new());
        assert!(
            denial
                .evidence_refs
                .contains(&"validation:intelligence-assist-draft-policy-drift".to_owned())
        );
    }

    #[test]
    fn applies_no_code_builder_capability_scope_before_kernel() {
        let mut input = valid_domain_request();
        input.policy_decision.allowed_actions = vec![AssistDraftAction::ExplainDraft];

        let decision = plan_domain_assist_draft(input);

        let denial = decision.denial().expect("denied");
        assert_eq!(
            denial.denial_kind,
            AssistDraftDomainDenialKind::ActionDenied
        );
        assert!(denial.refusal_banner.contains("not allowed"));
        assert!(
            denial
                .evidence_refs
                .contains(&"policy:assist-draft:allow".to_owned())
        );
    }

    #[test]
    fn preserves_user_locale_and_denies_locale_outside_policy() {
        let mut allowed = valid_domain_request();
        allowed.locale = "ko-KR".to_owned();
        allowed
            .policy_decision
            .allowed_locales
            .push("ko-KR".to_owned());

        let planned = plan_domain_assist_draft(allowed)
            .plan()
            .expect("planned")
            .clone();
        assert_eq!(planned.locale, "ko-KR");

        let mut denied = valid_domain_request();
        denied.locale = "fr-FR".to_owned();

        let decision = plan_domain_assist_draft(denied);
        let denial = decision.denial().expect("denied");
        assert_eq!(
            denial.denial_kind,
            AssistDraftDomainDenialKind::LocaleDenied
        );
        assert_eq!(denial.locale, "fr-FR");
        assert!(denial.refusal_banner.contains("not available"));
    }

    #[test]
    fn never_auto_publishes_generated_change_or_shadow_automation() {
        let mut input = valid_domain_request();
        input.request.review_gate = AssistDraftReviewGate::AutoApplyRequested;
        input
            .request
            .requested_actions
            .push(AssistDraftAction::ActivateDraft);
        input.request.invocation_mode = AssistDraftInvocationMode::ShadowLoop;

        let decision = plan_domain_assist_draft(input);

        let denial = decision.denial().expect("denied");
        assert_eq!(
            denial.denial_kind,
            AssistDraftDomainDenialKind::AutoApplyDenied
        );
        assert_eq!(denial.kernel_reasons, Vec::<AssistDraftDenialReason>::new());
        assert!(denial.refusal_banner.contains("review"));
    }

    #[test]
    fn kernel_external_sensitive_denial_is_preserved_with_policy_evidence() {
        let mut input = valid_domain_request();
        input.request.audience = AssistDraftAudience::ExternalEndUser;
        input.request.data_classes = vec![
            AssistDraftDataClass::Public,
            AssistDraftDataClass::Financial,
        ];
        input
            .policy_decision
            .allowed_audiences
            .push(AssistDraftAudience::ExternalEndUser);
        input
            .policy_decision
            .allowed_data_classes
            .push(AssistDraftDataClass::Financial);

        let decision = plan_domain_assist_draft(input);

        let denial = decision.denial().expect("denied");
        assert_eq!(
            denial.denial_kind,
            AssistDraftDomainDenialKind::KernelDenied
        );
        assert_eq!(
            denial.kernel_reasons,
            vec![AssistDraftDenialReason::SensitiveExternalDraftDenied]
        );
        assert!(
            denial
                .evidence_refs
                .contains(&"policy:assist-draft:allow".to_owned())
        );
    }

    #[test]
    fn rejects_prompt_or_secret_shaped_policy_metadata_without_echo() {
        let mut input = valid_domain_request();
        input.brand_surface_ref = "raw prompt: leak customer message".to_owned();
        input.policy_decision.decision_id = "sk-secret".to_owned();
        input
            .prompt_context_refs
            .push("document=raw output".to_owned());

        let decision = plan_domain_assist_draft(input);

        assert_eq!(decision.status(), AssistDraftDomainStatus::Denied);
        let debug = format!("{decision:?}");
        assert!(!debug.contains("raw prompt"));
        assert!(!debug.contains("customer message"));
        assert!(!debug.contains("sk-secret"));
        assert!(!debug.contains("document=raw output"));
    }

    fn valid_domain_request() -> DomainAssistDraftRequest {
        DomainAssistDraftRequest {
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
                data_classes: vec![AssistDraftDataClass::Internal, AssistDraftDataClass::Public],
                requested_actions: vec![
                    AssistDraftAction::CreateDraft,
                    AssistDraftAction::ExplainDraft,
                ],
                additional_evidence_refs: Vec::new(),
            },
        }
    }
}
