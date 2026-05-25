//! Intelligence assist-draft kernel foundation.
//!
//! This crate plans advisory, metadata-only AI draft requests for no-code
//! builders. It enforces ADR-0219's no-code-first rule: AI output is always a
//! reviewable draft imported into deterministic builders, never an auto-applied
//! tenant mutation. The kernel performs no model calls, prompt inspection,
//! retrieval, citation rendering, filesystem/network I/O, durable storage, cost
//! charging, or audit-chain emission.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AssistDraftBuilderSurface {
    AuditSearch,
    ReportBuilder,
    TenantAdminConsole,
    WorkflowStudio,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AssistDraftKind {
    PolicyExplanation,
    ReportNarrative,
    ReviewAgenda,
    SearchQuery,
    WorkflowDraft,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AssistDraftAudience {
    ExternalEndUser,
    InternalAutomation,
    TenantAdmin,
    TenantBuilder,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AssistDraftInvocationMode {
    ShadowLoop,
    TenantConfiguredAutomation,
    UserInvoked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AssistDraftReviewGate {
    AutoApplyRequested,
    HumanReviewRequired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AssistDraftDataClass {
    Audit,
    Financial,
    Health,
    Internal,
    PiiIdentifying,
    Phi,
    PiiSensitive,
    Public,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AssistDraftAction {
    CreateDraft,
    ExplainDraft,
    ProposePatch,
    ActivateDraft,
    ApplyDraft,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssistDraftStatus {
    Denied,
    Planned,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AssistDraftDenialReason {
    InvalidInput,
    MissingBudget,
    MissingConsent,
    MissingGuardrail,
    MissingPolicyDecision,
    NoRequestedActions,
    UnsupportedDraftKindForSurface,
    AutoApplyDenied,
    HiddenInvocationDenied,
    SensitiveExternalDraftDenied,
    UnsafeActionDenied,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftRequest {
    pub tenant_id: String,                          // data_class: INTERNAL_ONLY
    pub principal_id: String,                       // data_class: INTERNAL_ONLY
    pub context_id: String,                         // data_class: INTERNAL_ONLY
    pub builder_surface: AssistDraftBuilderSurface, // data_class: PUBLIC
    pub draft_kind: AssistDraftKind,                // data_class: PUBLIC
    pub audience: AssistDraftAudience,              // data_class: INTERNAL_ONLY
    pub invocation_mode: AssistDraftInvocationMode, // data_class: INTERNAL_ONLY
    pub review_gate: AssistDraftReviewGate,         // data_class: INTERNAL_ONLY
    pub prompt_ref: String,                         // data_class: INTERNAL_ONLY
    pub target_builder_ref: String,                 // data_class: INTERNAL_ONLY
    pub output_contract_ref: String,                // data_class: INTERNAL_ONLY
    pub consent_grant_ref: String,                  // data_class: INTERNAL_ONLY
    pub budget_evidence_ref: String,                // data_class: INTERNAL_ONLY
    pub policy_decision_ref: String,                // data_class: INTERNAL_ONLY
    pub model_route_ref: String,                    // data_class: INTERNAL_ONLY
    pub guardrail_evidence_ref: String,             // data_class: INTERNAL_ONLY
    pub request_evidence_ref: String,               // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,                  // data_class: INTERNAL_ONLY
    pub data_classes: Vec<AssistDraftDataClass>,    // data_class: INTERNAL_ONLY
    pub requested_actions: Vec<AssistDraftAction>,  // data_class: INTERNAL_ONLY
    pub additional_evidence_refs: Vec<String>,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftPlan {
    pub tenant_id: String,                          // data_class: INTERNAL_ONLY
    pub context_id: String,                         // data_class: INTERNAL_ONLY
    pub builder_surface: AssistDraftBuilderSurface, // data_class: PUBLIC
    pub draft_kind: AssistDraftKind,                // data_class: PUBLIC
    pub status: AssistDraftStatus,                  // data_class: PUBLIC
    pub review_gate: AssistDraftReviewGate,         // data_class: INTERNAL_ONLY
    pub output_contract_ref: String,                // data_class: INTERNAL_ONLY
    pub target_builder_ref: String,                 // data_class: INTERNAL_ONLY
    pub allowed_actions: Vec<AssistDraftAction>,    // data_class: INTERNAL_ONLY
    pub provenance_refs: Vec<String>,               // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                 // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftDenial {
    pub tenant_id: String,                          // data_class: INTERNAL_ONLY
    pub context_id: String,                         // data_class: INTERNAL_ONLY
    pub builder_surface: AssistDraftBuilderSurface, // data_class: PUBLIC
    pub draft_kind: AssistDraftKind,                // data_class: PUBLIC
    pub status: AssistDraftStatus,                  // data_class: PUBLIC
    pub reasons: Vec<AssistDraftDenialReason>,      // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                 // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssistDraftDecision {
    Plan(AssistDraftPlan),
    Deny(AssistDraftDenial),
}

impl AssistDraftDecision {
    pub fn status(&self) -> AssistDraftStatus {
        match self {
            Self::Plan(plan) => plan.status,
            Self::Deny(denial) => denial.status,
        }
    }

    pub fn plan(&self) -> Option<&AssistDraftPlan> {
        match self {
            Self::Plan(plan) => Some(plan),
            Self::Deny(_) => None,
        }
    }

    pub fn denial(&self) -> Option<&AssistDraftDenial> {
        match self {
            Self::Plan(_) => None,
            Self::Deny(denial) => Some(denial),
        }
    }
}

pub fn decide_assist_draft(request: AssistDraftRequest) -> AssistDraftDecision {
    let reasons = denial_reasons(&request);
    if !reasons.is_empty() {
        return AssistDraftDecision::Deny(AssistDraftDenial {
            tenant_id: safe_ref(&request.tenant_id, "redacted-invalid-tenant-id"),
            context_id: safe_ref(&request.context_id, "redacted-invalid-context-id"),
            builder_surface: request.builder_surface,
            draft_kind: request.draft_kind,
            status: AssistDraftStatus::Denied,
            reasons,
            evidence_refs: denial_evidence_refs(&request),
        });
    }

    let evidence_refs = plan_evidence_refs(&request);

    AssistDraftDecision::Plan(AssistDraftPlan {
        tenant_id: request.tenant_id.clone(),
        context_id: request.context_id.clone(),
        builder_surface: request.builder_surface,
        draft_kind: request.draft_kind,
        status: AssistDraftStatus::Planned,
        review_gate: AssistDraftReviewGate::HumanReviewRequired,
        output_contract_ref: request.output_contract_ref.clone(),
        target_builder_ref: request.target_builder_ref.clone(),
        allowed_actions: sorted_unique_actions(request.requested_actions),
        provenance_refs: sorted_unique(vec![
            request.budget_evidence_ref.clone(),
            request.consent_grant_ref.clone(),
            request.guardrail_evidence_ref.clone(),
            request.model_route_ref.clone(),
            request.policy_decision_ref.clone(),
            request.prompt_ref.clone(),
        ]),
        evidence_refs,
    })
}

fn denial_reasons(request: &AssistDraftRequest) -> Vec<AssistDraftDenialReason> {
    let mut reasons = BTreeSet::new();
    let mut invalid_input = false;

    for value in [
        &request.tenant_id,
        &request.principal_id,
        &request.context_id,
        &request.prompt_ref,
        &request.target_builder_ref,
        &request.output_contract_ref,
        &request.request_evidence_ref,
        &request.trace_context_ref,
        &request.model_route_ref,
    ] {
        if !is_safe_opaque_ref(value) {
            invalid_input = true;
        }
    }

    if !is_safe_opaque_ref(&request.consent_grant_ref) {
        reasons.insert(AssistDraftDenialReason::MissingConsent);
        if contains_raw_secret_material(&request.consent_grant_ref)
            || contains_raw_content_material(&request.consent_grant_ref)
            || !request.consent_grant_ref.trim().is_empty()
        {
            invalid_input = true;
        }
    }
    if !is_safe_opaque_ref(&request.budget_evidence_ref) {
        reasons.insert(AssistDraftDenialReason::MissingBudget);
        if contains_raw_secret_material(&request.budget_evidence_ref)
            || contains_raw_content_material(&request.budget_evidence_ref)
            || !request.budget_evidence_ref.trim().is_empty()
        {
            invalid_input = true;
        }
    }
    if !is_safe_opaque_ref(&request.guardrail_evidence_ref) {
        reasons.insert(AssistDraftDenialReason::MissingGuardrail);
        if contains_raw_secret_material(&request.guardrail_evidence_ref)
            || contains_raw_content_material(&request.guardrail_evidence_ref)
            || !request.guardrail_evidence_ref.trim().is_empty()
        {
            invalid_input = true;
        }
    }
    if !is_safe_opaque_ref(&request.policy_decision_ref) {
        reasons.insert(AssistDraftDenialReason::MissingPolicyDecision);
        if contains_raw_secret_material(&request.policy_decision_ref)
            || contains_raw_content_material(&request.policy_decision_ref)
            || !request.policy_decision_ref.trim().is_empty()
        {
            invalid_input = true;
        }
    }

    for evidence_ref in &request.additional_evidence_refs {
        if !is_safe_opaque_ref(evidence_ref) {
            invalid_input = true;
        }
    }
    if request.data_classes.is_empty() {
        invalid_input = true;
    }
    if request.requested_actions.is_empty() {
        reasons.insert(AssistDraftDenialReason::NoRequestedActions);
    }
    if request.review_gate == AssistDraftReviewGate::AutoApplyRequested {
        reasons.insert(AssistDraftDenialReason::AutoApplyDenied);
    }
    if request.invocation_mode == AssistDraftInvocationMode::ShadowLoop {
        reasons.insert(AssistDraftDenialReason::HiddenInvocationDenied);
    }
    if request.requested_actions.iter().any(|action| {
        matches!(
            action,
            AssistDraftAction::ActivateDraft | AssistDraftAction::ApplyDraft
        )
    }) {
        reasons.insert(AssistDraftDenialReason::UnsafeActionDenied);
    }
    if !draft_kind_supported_by_surface(request.builder_surface, request.draft_kind) {
        reasons.insert(AssistDraftDenialReason::UnsupportedDraftKindForSurface);
    }
    if request.audience == AssistDraftAudience::ExternalEndUser
        && request
            .data_classes
            .iter()
            .any(|data_class| *data_class != AssistDraftDataClass::Public)
    {
        reasons.insert(AssistDraftDenialReason::SensitiveExternalDraftDenied);
    }
    if invalid_input {
        reasons.insert(AssistDraftDenialReason::InvalidInput);
    }

    reasons.into_iter().collect()
}

fn draft_kind_supported_by_surface(
    surface: AssistDraftBuilderSurface,
    draft_kind: AssistDraftKind,
) -> bool {
    matches!(
        (surface, draft_kind),
        (
            AssistDraftBuilderSurface::WorkflowStudio,
            AssistDraftKind::WorkflowDraft
        ) | (
            AssistDraftBuilderSurface::WorkflowStudio,
            AssistDraftKind::PolicyExplanation
        ) | (
            AssistDraftBuilderSurface::TenantAdminConsole,
            AssistDraftKind::PolicyExplanation
        ) | (
            AssistDraftBuilderSurface::TenantAdminConsole,
            AssistDraftKind::ReviewAgenda
        ) | (
            AssistDraftBuilderSurface::ReportBuilder,
            AssistDraftKind::ReportNarrative
        ) | (
            AssistDraftBuilderSurface::AuditSearch,
            AssistDraftKind::SearchQuery
        ) | (
            AssistDraftBuilderSurface::AuditSearch,
            AssistDraftKind::PolicyExplanation
        )
    )
}

fn plan_evidence_refs(request: &AssistDraftRequest) -> Vec<String> {
    let mut refs = vec![
        request.budget_evidence_ref.clone(),
        request.consent_grant_ref.clone(),
        request.guardrail_evidence_ref.clone(),
        request.model_route_ref.clone(),
        request.policy_decision_ref.clone(),
        request.request_evidence_ref.clone(),
        request.trace_context_ref.clone(),
    ];
    refs.extend(request.additional_evidence_refs.clone());
    sorted_unique(refs)
}

fn denial_evidence_refs(request: &AssistDraftRequest) -> Vec<String> {
    let mut refs = Vec::new();
    for value in [
        &request.budget_evidence_ref,
        &request.consent_grant_ref,
        &request.guardrail_evidence_ref,
        &request.model_route_ref,
        &request.policy_decision_ref,
        &request.request_evidence_ref,
        &request.trace_context_ref,
    ] {
        if is_safe_opaque_ref(value) {
            refs.push(value.clone());
        }
    }
    refs.extend(
        request
            .additional_evidence_refs
            .iter()
            .filter(|value| is_safe_opaque_ref(value))
            .cloned(),
    );
    if refs.is_empty() {
        refs.push("validation:intelligence-assist-draft-kernel-input".to_owned());
    }
    sorted_unique(refs)
}

fn sorted_unique_actions(values: Vec<AssistDraftAction>) -> Vec<AssistDraftAction> {
    let mut values = values;
    values.sort();
    values.dedup();
    values
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn safe_ref(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if is_safe_opaque_ref(trimmed) && trimmed == value {
        trimmed.to_owned()
    } else {
        fallback.to_owned()
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
        || lower.contains("full answer text")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_user_invoked_workflow_draft_with_human_review_and_provenance_refs() {
        let decision = decide_assist_draft(valid_request());

        assert_eq!(
            decision,
            AssistDraftDecision::Plan(AssistDraftPlan {
                tenant_id: "tenant:alpha".to_owned(),
                context_id: "context://workflow-studio/canvas-1".to_owned(),
                builder_surface: AssistDraftBuilderSurface::WorkflowStudio,
                draft_kind: AssistDraftKind::WorkflowDraft,
                status: AssistDraftStatus::Planned,
                review_gate: AssistDraftReviewGate::HumanReviewRequired,
                output_contract_ref: "workflow-spec://contracts/v1".to_owned(),
                target_builder_ref: "builder://workflow-studio/canvas-1".to_owned(),
                allowed_actions: vec![
                    AssistDraftAction::CreateDraft,
                    AssistDraftAction::ExplainDraft
                ],
                provenance_refs: vec![
                    "budget:assist-draft:1".to_owned(),
                    "consent:assist-draft:1".to_owned(),
                    "guardrail:assist-draft:allow".to_owned(),
                    "model-route:assist-draft:1".to_owned(),
                    "policy:assist-draft:allow".to_owned(),
                    "prompt://assist-draft/req-1".to_owned(),
                ],
                evidence_refs: vec![
                    "budget:assist-draft:1".to_owned(),
                    "consent:assist-draft:1".to_owned(),
                    "guardrail:assist-draft:allow".to_owned(),
                    "model-route:assist-draft:1".to_owned(),
                    "policy:assist-draft:allow".to_owned(),
                    "request:assist-draft:1".to_owned(),
                    "trace:assist-draft:1".to_owned(),
                ],
            })
        );
    }

    #[test]
    fn denies_auto_apply_activation_and_shadow_loop_invocations() {
        let mut request = valid_request();
        request.invocation_mode = AssistDraftInvocationMode::ShadowLoop;
        request.review_gate = AssistDraftReviewGate::AutoApplyRequested;
        request
            .requested_actions
            .push(AssistDraftAction::ActivateDraft);

        let decision = decide_assist_draft(request);

        assert_eq!(decision.status(), AssistDraftStatus::Denied);
        let denial = decision.denial().expect("denied");
        assert_eq!(
            denial.reasons,
            vec![
                AssistDraftDenialReason::AutoApplyDenied,
                AssistDraftDenialReason::HiddenInvocationDenied,
                AssistDraftDenialReason::UnsafeActionDenied,
            ]
        );
        assert!(
            denial
                .evidence_refs
                .contains(&"request:assist-draft:1".to_owned())
        );
    }

    #[test]
    fn denies_sensitive_external_audience_before_draft_planning() {
        let mut request = valid_request();
        request.audience = AssistDraftAudience::ExternalEndUser;
        request.data_classes = vec![
            AssistDraftDataClass::Public,
            AssistDraftDataClass::PiiSensitive,
        ];

        let decision = decide_assist_draft(request);

        assert_eq!(decision.status(), AssistDraftStatus::Denied);
        assert_eq!(
            decision.denial().expect("denied").reasons,
            vec![AssistDraftDenialReason::SensitiveExternalDraftDenied]
        );
    }

    #[test]
    fn denies_missing_consent_budget_guardrail_and_policy_refs() {
        let mut request = valid_request();
        request.consent_grant_ref.clear();
        request.budget_evidence_ref = " ".to_owned();
        request.guardrail_evidence_ref = "raw prompt: unsafe".to_owned();
        request.policy_decision_ref = "sk-secret".to_owned();

        let decision = decide_assist_draft(request);

        let denial = decision.denial().expect("denied");
        assert_eq!(
            denial.reasons,
            vec![
                AssistDraftDenialReason::InvalidInput,
                AssistDraftDenialReason::MissingBudget,
                AssistDraftDenialReason::MissingConsent,
                AssistDraftDenialReason::MissingGuardrail,
                AssistDraftDenialReason::MissingPolicyDecision,
            ]
        );
        let debug = format!("{denial:?}");
        assert!(!debug.contains("raw prompt"));
        assert!(!debug.contains("sk-secret"));
    }

    #[test]
    fn denies_surface_kind_mismatch_and_empty_actions() {
        let mut request = valid_request();
        request.builder_surface = AssistDraftBuilderSurface::AuditSearch;
        request.draft_kind = AssistDraftKind::WorkflowDraft;
        request.requested_actions.clear();

        let decision = decide_assist_draft(request);

        assert_eq!(
            decision.denial().expect("denied").reasons,
            vec![
                AssistDraftDenialReason::NoRequestedActions,
                AssistDraftDenialReason::UnsupportedDraftKindForSurface,
            ]
        );
    }

    #[test]
    fn raw_prompt_output_document_or_secret_material_is_rejected_without_echo() {
        let mut request = valid_request();
        request.prompt_ref = "raw prompt: draft a secret email".to_owned();
        request.target_builder_ref = "document raw output sk-test".to_owned();
        request.principal_id = "customer message".to_owned();

        let decision = decide_assist_draft(request);

        assert_eq!(decision.status(), AssistDraftStatus::Denied);
        let debug = format!("{decision:?}");
        assert!(!debug.contains("raw prompt"));
        assert!(!debug.contains("raw output"));
        assert!(!debug.contains("sk-test"));
        assert!(!debug.contains("customer message"));
    }

    #[test]
    fn deterministic_ordering_deduplicates_actions_data_classes_and_evidence() {
        let mut request = valid_request();
        request.requested_actions = vec![
            AssistDraftAction::ExplainDraft,
            AssistDraftAction::CreateDraft,
            AssistDraftAction::ExplainDraft,
        ];
        request.data_classes = vec![
            AssistDraftDataClass::Internal,
            AssistDraftDataClass::Public,
            AssistDraftDataClass::Internal,
        ];
        request.additional_evidence_refs = vec![
            "evidence:z".to_owned(),
            "evidence:a".to_owned(),
            "evidence:a".to_owned(),
        ];

        let decision = decide_assist_draft(request);
        let plan = decision.plan().expect("planned");

        assert_eq!(
            plan.allowed_actions,
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
                "evidence:a".to_owned(),
                "evidence:z".to_owned(),
                "guardrail:assist-draft:allow".to_owned(),
                "model-route:assist-draft:1".to_owned(),
                "policy:assist-draft:allow".to_owned(),
                "request:assist-draft:1".to_owned(),
                "trace:assist-draft:1".to_owned(),
            ]
        );
    }

    fn valid_request() -> AssistDraftRequest {
        AssistDraftRequest {
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
        }
    }
}
