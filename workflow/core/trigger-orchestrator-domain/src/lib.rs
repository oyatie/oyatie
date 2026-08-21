//! Workflow-engine trigger-orchestrator domain foundation.
//!
//! The domain layer binds trigger admission to tenant, principal, policy,
//! source-surface, scheduler/webhook/event evidence, and audit refs before
//! delegating to the pure trigger-orchestrator kernel. It performs no Cedar
//! evaluation, scheduler execution, HMAC verification, HTTP serving, event-bus
//! consumption, run creation, storage, network, filesystem, wall-clock, random,
//! Kubernetes, cloud, or tenant workload scheduling work.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use workflow_trigger_orchestrator_kernel::{
    TRIGGER_ORCHESTRATOR_CLOUDEVENTS_SPECVERSION, TRIGGER_ORCHESTRATOR_KERNEL_CONTRACT_REF,
    TRIGGER_ORCHESTRATOR_KERNEL_SURFACE, TriggerOrchestratorDecision,
    TriggerOrchestratorDecisionStatus, TriggerOrchestratorDenialReason,
    TriggerOrchestratorEventEnvelope, TriggerOrchestratorOverlapPolicy,
    TriggerOrchestratorPolicyContext, TriggerOrchestratorRequest,
    TriggerOrchestratorScheduleMetadata, TriggerOrchestratorTriggerKind,
    TriggerOrchestratorWebhookMetadata, evaluate_trigger,
};

pub const TRIGGER_ORCHESTRATOR_DOMAIN_SURFACE: &str = "workflow-engine.trigger-orchestrator.domain";
pub const TRIGGER_ORCHESTRATOR_DOMAIN_CONTRACT_REF: &str =
    "workflow/workflow-engine/PRD.md#f2-run-trigger-dispatch";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TriggerOrchestratorDomainSource {
    Scheduler,
    StudioWebhook,
    SiblingEventBus,
    ManualUi,
    ApiCommand,
    WorkflowSpawn,
    OntologyProjection,
}

impl TriggerOrchestratorDomainSource {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Scheduler => "scheduler",
            Self::StudioWebhook => "studio-webhook",
            Self::SiblingEventBus => "sibling-event-bus",
            Self::ManualUi => "manual-ui",
            Self::ApiCommand => "api-command",
            Self::WorkflowSpawn => "workflow-spawn",
            Self::OntologyProjection => "ontology-projection",
        }
    }

    pub fn expected_kind(self) -> TriggerOrchestratorTriggerKind {
        match self {
            Self::Scheduler => TriggerOrchestratorTriggerKind::Cron,
            Self::StudioWebhook => TriggerOrchestratorTriggerKind::Webhook,
            Self::SiblingEventBus => TriggerOrchestratorTriggerKind::EventBus,
            Self::ManualUi => TriggerOrchestratorTriggerKind::Manual,
            Self::ApiCommand => TriggerOrchestratorTriggerKind::Api,
            Self::WorkflowSpawn => TriggerOrchestratorTriggerKind::WorkflowSpawn,
            Self::OntologyProjection => TriggerOrchestratorTriggerKind::Ontology,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TriggerOrchestratorDomainStatus {
    Accepted,
    Denied,
    Deferred,
    Suppressed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TriggerOrchestratorDomainDenialKind {
    UnsafeMetadata,
    MissingPolicyBinding,
    ScopeMismatch,
    SourceSurfaceMismatch,
    MissingSourceEvidence,
    KernelDenied,
    KernelDeferred,
    ReplaySuppressed,
}

impl TriggerOrchestratorDomainDenialKind {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::UnsafeMetadata => "unsafe-metadata",
            Self::MissingPolicyBinding => "missing-policy-binding",
            Self::ScopeMismatch => "scope-mismatch",
            Self::SourceSurfaceMismatch => "source-surface-mismatch",
            Self::MissingSourceEvidence => "missing-source-evidence",
            Self::KernelDenied => "kernel-denied",
            Self::KernelDeferred => "kernel-deferred",
            Self::ReplaySuppressed => "replay-suppressed",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorDomainPolicyBinding {
    pub tenant_id: String,                            // data_class: INTERNAL_ONLY
    pub trigger_id: String,                           // data_class: INTERNAL_ONLY
    pub workflow_spec_id: String,                     // data_class: INTERNAL_ONLY
    pub version_sha: String,                          // data_class: INTERNAL_ONLY
    pub active_cell_id: String,                       // data_class: INTERNAL_ONLY
    pub principal_id: String,                         // data_class: INTERNAL_ONLY
    pub allowed_kind: TriggerOrchestratorTriggerKind, // data_class: PUBLIC
    pub policy_decision_id: String,                   // data_class: INTERNAL_ONLY
    pub policy_evidence_ref: String,                  // data_class: INTERNAL_ONLY
    pub policy_bundle_ref: String,                    // data_class: INTERNAL_ONLY
    pub authorization_surface_ref: String,            // data_class: INTERNAL_ONLY
    pub source_evidence_ref: String,                  // data_class: INTERNAL_ONLY
    pub scheduler_evidence_ref: Option<String>,       // data_class: INTERNAL_ONLY
    pub webhook_auth_evidence_ref: Option<String>,    // data_class: INTERNAL_ONLY
    pub event_contract_ref: Option<String>,           // data_class: INTERNAL_ONLY
    pub replay_epoch_ref: String,                     // data_class: INTERNAL_ONLY
    pub audit_chain_ref: String,                      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorDomainRequest {
    pub binding: TriggerOrchestratorDomainPolicyBinding, // data_class: INTERNAL_ONLY
    pub trigger_request: TriggerOrchestratorRequest,     // data_class: INTERNAL_ONLY
    pub source: TriggerOrchestratorDomainSource,         // data_class: INTERNAL_ONLY
    pub trace_ref: String,                               // data_class: INTERNAL_ONLY
    pub correlation_ref: String,                         // data_class: INTERNAL_ONLY
    pub idempotency_scope_ref: String,                   // data_class: INTERNAL_ONLY
    pub dry_run_reason_ref: Option<String>,              // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorDomainDecision {
    pub status: TriggerOrchestratorDomainStatus, // data_class: PUBLIC
    pub denial_kind: Option<TriggerOrchestratorDomainDenialKind>, // data_class: INTERNAL_ONLY
    pub kernel_status: Option<TriggerOrchestratorDecisionStatus>, // data_class: PUBLIC
    pub kernel_reason: Option<TriggerOrchestratorDenialReason>, // data_class: INTERNAL_ONLY
    pub tenant_id: String,                       // data_class: INTERNAL_ONLY
    pub trigger_id: String,                      // data_class: INTERNAL_ONLY
    pub workflow_spec_id: String,                // data_class: INTERNAL_ONLY
    pub dispatch_required: bool,                 // data_class: PUBLIC
    pub run_idempotency_key: Option<String>,     // data_class: INTERNAL_ONLY
    pub start_run_command_ref: Option<String>,   // data_class: INTERNAL_ONLY
    pub schedule_next_check_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub audit_refs: Vec<String>,                 // data_class: INTERNAL_ONLY
    pub non_claim_refs: Vec<String>,             // data_class: INTERNAL_ONLY
}

pub fn evaluate_trigger_domain(
    request: TriggerOrchestratorDomainRequest,
) -> TriggerOrchestratorDomainDecision {
    if let Some(decision) = preflight_denial(&request) {
        return decision;
    }

    let kernel_decision = evaluate_trigger(request.trigger_request.clone());
    decision_from_kernel(&request, kernel_decision)
}

fn preflight_denial(
    request: &TriggerOrchestratorDomainRequest,
) -> Option<TriggerOrchestratorDomainDecision> {
    let mut missing = Vec::new();
    if request.binding.policy_decision_id.trim().is_empty() {
        missing.push("validation:policy-decision-required".to_owned());
    }
    if request.binding.policy_evidence_ref.trim().is_empty() {
        missing.push("validation:policy-evidence-required".to_owned());
    }
    if request.binding.policy_bundle_ref.trim().is_empty() {
        missing.push("validation:policy-bundle-required".to_owned());
    }
    if request.binding.authorization_surface_ref.trim().is_empty() {
        missing.push("validation:authorization-surface-required".to_owned());
    }
    if request.binding.source_evidence_ref.trim().is_empty() {
        missing.push("validation:source-evidence-required".to_owned());
    }
    if request.binding.replay_epoch_ref.trim().is_empty() {
        missing.push("validation:replay-epoch-required".to_owned());
    }
    if request.binding.audit_chain_ref.trim().is_empty() {
        missing.push("validation:audit-chain-required".to_owned());
    }
    if !missing.is_empty() {
        return Some(domain_denial(
            request,
            TriggerOrchestratorDomainDenialKind::MissingPolicyBinding,
            None,
            None,
            missing,
        ));
    }

    if has_unsafe_metadata(request) {
        return Some(domain_denial(
            request,
            TriggerOrchestratorDomainDenialKind::UnsafeMetadata,
            None,
            None,
            vec!["trigger-orchestrator-domain:unsafe-metadata".to_owned()],
        ));
    }

    if request.source.expected_kind() != request.trigger_request.kind
        || request.binding.allowed_kind != request.trigger_request.kind
    {
        return Some(domain_denial(
            request,
            TriggerOrchestratorDomainDenialKind::SourceSurfaceMismatch,
            None,
            None,
            vec!["trigger-orchestrator-domain:source-surface-mismatch".to_owned()],
        ));
    }

    if binding_scope_mismatch(request) {
        return Some(domain_denial(
            request,
            TriggerOrchestratorDomainDenialKind::ScopeMismatch,
            None,
            None,
            vec!["trigger-orchestrator-domain:scope-mismatch".to_owned()],
        ));
    }

    if let Some(reason) = missing_source_specific_evidence(request) {
        return Some(domain_denial(
            request,
            TriggerOrchestratorDomainDenialKind::MissingSourceEvidence,
            None,
            None,
            vec![reason],
        ));
    }

    None
}

fn decision_from_kernel(
    request: &TriggerOrchestratorDomainRequest,
    kernel: TriggerOrchestratorDecision,
) -> TriggerOrchestratorDomainDecision {
    let (status, denial_kind) = match kernel.status {
        TriggerOrchestratorDecisionStatus::Accepted => {
            (TriggerOrchestratorDomainStatus::Accepted, None)
        }
        TriggerOrchestratorDecisionStatus::Denied => (
            TriggerOrchestratorDomainStatus::Denied,
            Some(TriggerOrchestratorDomainDenialKind::KernelDenied),
        ),
        TriggerOrchestratorDecisionStatus::Deferred => (
            TriggerOrchestratorDomainStatus::Deferred,
            Some(TriggerOrchestratorDomainDenialKind::KernelDeferred),
        ),
        TriggerOrchestratorDecisionStatus::Suppressed => (
            TriggerOrchestratorDomainStatus::Suppressed,
            Some(TriggerOrchestratorDomainDenialKind::ReplaySuppressed),
        ),
    };
    let mut audit_refs = kernel.evidence_refs.clone();
    audit_refs.extend(domain_refs(request));
    audit_refs.push(format!(
        "trigger-orchestrator-domain:{}",
        status_label(status)
    ));
    if let Some(kind) = denial_kind {
        audit_refs.push(format!("trigger-orchestrator-domain:{}", kind.as_wire()));
    }
    TriggerOrchestratorDomainDecision {
        status,
        denial_kind,
        kernel_status: Some(kernel.status),
        kernel_reason: kernel.reason,
        tenant_id: kernel.tenant_id,
        trigger_id: kernel.trigger_id,
        workflow_spec_id: kernel.workflow_spec_id,
        dispatch_required: kernel.dispatch_required,
        run_idempotency_key: kernel.run_idempotency_key,
        start_run_command_ref: kernel.start_run_command_ref,
        schedule_next_check_epoch_seconds: kernel.schedule_next_check_epoch_seconds,
        audit_refs: sorted_unique(audit_refs),
        non_claim_refs: sorted_unique([kernel.non_claim_refs, domain_non_claim_refs()].concat()),
    }
}

fn domain_denial(
    request: &TriggerOrchestratorDomainRequest,
    denial_kind: TriggerOrchestratorDomainDenialKind,
    kernel_status: Option<TriggerOrchestratorDecisionStatus>,
    kernel_reason: Option<TriggerOrchestratorDenialReason>,
    mut audit_refs: Vec<String>,
) -> TriggerOrchestratorDomainDecision {
    audit_refs.push(format!(
        "trigger-orchestrator-domain:{}",
        denial_kind.as_wire()
    ));
    let unsafe_metadata = denial_kind == TriggerOrchestratorDomainDenialKind::UnsafeMetadata;
    TriggerOrchestratorDomainDecision {
        status: TriggerOrchestratorDomainStatus::Denied,
        denial_kind: Some(denial_kind),
        kernel_status,
        kernel_reason,
        tenant_id: if unsafe_metadata {
            safe_tenant_or_redacted(&request.trigger_request.context.tenant_id)
        } else {
            request.trigger_request.context.tenant_id.clone()
        },
        trigger_id: if unsafe_metadata {
            safe_ref_or_redacted(
                &request.trigger_request.context.trigger_id,
                "redacted:trigger",
            )
        } else {
            request.trigger_request.context.trigger_id.clone()
        },
        workflow_spec_id: if unsafe_metadata {
            safe_ref_or_redacted(
                &request.trigger_request.context.workflow_spec_id,
                "redacted:workflow-spec",
            )
        } else {
            request.trigger_request.context.workflow_spec_id.clone()
        },
        dispatch_required: false,
        run_idempotency_key: None,
        start_run_command_ref: None,
        schedule_next_check_epoch_seconds: None,
        audit_refs: if unsafe_metadata {
            vec!["trigger-orchestrator-domain:unsafe-metadata".to_owned()]
        } else {
            sorted_unique(audit_refs)
        },
        non_claim_refs: domain_non_claim_refs(),
    }
}

fn binding_scope_mismatch(request: &TriggerOrchestratorDomainRequest) -> bool {
    let context = &request.trigger_request.context;
    request.binding.tenant_id != context.tenant_id
        || request.binding.trigger_id != context.trigger_id
        || request.binding.workflow_spec_id != context.workflow_spec_id
        || request.binding.version_sha != context.version_sha
        || request.binding.active_cell_id != context.active_cell_id
        || request.binding.principal_id != context.principal_id
        || request.binding.policy_decision_id != context.policy_decision_id
        || request.binding.policy_evidence_ref != context.policy_evidence_ref
        || request.binding.replay_epoch_ref != context.replay_epoch_ref
}

fn missing_source_specific_evidence(request: &TriggerOrchestratorDomainRequest) -> Option<String> {
    match request.trigger_request.kind {
        TriggerOrchestratorTriggerKind::Cron => request
            .binding
            .scheduler_evidence_ref
            .is_none()
            .then(|| "validation:scheduler-evidence-required".to_owned()),
        TriggerOrchestratorTriggerKind::Webhook => request
            .binding
            .webhook_auth_evidence_ref
            .is_none()
            .then(|| "validation:webhook-auth-evidence-required".to_owned()),
        TriggerOrchestratorTriggerKind::EventBus => request
            .binding
            .event_contract_ref
            .is_none()
            .then(|| "validation:event-contract-required".to_owned()),
        TriggerOrchestratorTriggerKind::Manual
        | TriggerOrchestratorTriggerKind::Api
        | TriggerOrchestratorTriggerKind::WorkflowSpawn
        | TriggerOrchestratorTriggerKind::Ontology => None,
    }
}

fn domain_refs(request: &TriggerOrchestratorDomainRequest) -> Vec<String> {
    let mut refs = vec![
        TRIGGER_ORCHESTRATOR_DOMAIN_SURFACE.to_owned(),
        "contract:workflow-prd-f2-run-trigger-dispatch".to_owned(),
        format!(
            "trigger-orchestrator-domain:source:{}",
            request.source.as_wire()
        ),
        request.binding.policy_decision_id.clone(),
        request.binding.policy_evidence_ref.clone(),
        request.binding.policy_bundle_ref.clone(),
        request.binding.authorization_surface_ref.clone(),
        request.binding.source_evidence_ref.clone(),
        request.binding.replay_epoch_ref.clone(),
        request.binding.audit_chain_ref.clone(),
        request.trace_ref.clone(),
        request.correlation_ref.clone(),
        request.idempotency_scope_ref.clone(),
    ];
    refs.extend(request.evidence_refs.clone());
    if let Some(value) = request.binding.scheduler_evidence_ref.clone() {
        refs.push(value);
    }
    if let Some(value) = request.binding.webhook_auth_evidence_ref.clone() {
        refs.push(value);
    }
    if let Some(value) = request.binding.event_contract_ref.clone() {
        refs.push(value);
    }
    if let Some(value) = request.dry_run_reason_ref.clone() {
        refs.push(value);
    }
    sorted_unique(refs)
}

fn has_unsafe_metadata(request: &TriggerOrchestratorDomainRequest) -> bool {
    !is_safe_tenant(&request.binding.tenant_id)
        || !is_safe_ref(&request.binding.trigger_id)
        || !is_safe_ref(&request.binding.workflow_spec_id)
        || !is_safe_ref(&request.binding.version_sha)
        || !is_safe_ref(&request.binding.active_cell_id)
        || !is_safe_ref(&request.binding.principal_id)
        || !is_safe_ref(&request.binding.policy_decision_id)
        || !is_safe_ref(&request.binding.policy_evidence_ref)
        || !is_safe_ref(&request.binding.policy_bundle_ref)
        || !is_safe_ref(&request.binding.authorization_surface_ref)
        || !is_safe_ref(&request.binding.source_evidence_ref)
        || !is_safe_optional_ref(request.binding.scheduler_evidence_ref.as_deref())
        || !is_safe_optional_ref(request.binding.webhook_auth_evidence_ref.as_deref())
        || !is_safe_optional_ref(request.binding.event_contract_ref.as_deref())
        || !is_safe_ref(&request.binding.replay_epoch_ref)
        || !is_safe_ref(&request.binding.audit_chain_ref)
        || !is_safe_ref(&request.trace_ref)
        || !is_safe_ref(&request.correlation_ref)
        || !is_safe_ref(&request.idempotency_scope_ref)
        || !is_safe_optional_ref(request.dry_run_reason_ref.as_deref())
        || !request.evidence_refs.iter().all(|value| is_safe_ref(value))
}

fn domain_non_claim_refs() -> Vec<String> {
    vec![
        "no-policy-engine".to_owned(),
        "no-cedar-evaluation".to_owned(),
        "no-secret-materialization".to_owned(),
        "no-scheduler-runtime".to_owned(),
        "no-webhook-server".to_owned(),
        "no-hmac-verification".to_owned(),
        "no-event-bus-consumer".to_owned(),
        "no-run-creation".to_owned(),
        "no-durable-trigger-store".to_owned(),
        "no-cloud-deployment".to_owned(),
        "no-tenant-workload-scheduling".to_owned(),
        "no-hyperscaler-claim".to_owned(),
    ]
}

fn status_label(status: TriggerOrchestratorDomainStatus) -> &'static str {
    match status {
        TriggerOrchestratorDomainStatus::Accepted => "accepted",
        TriggerOrchestratorDomainStatus::Denied => "denied",
        TriggerOrchestratorDomainStatus::Deferred => "deferred",
        TriggerOrchestratorDomainStatus::Suppressed => "suppressed",
    }
}

fn safe_tenant_or_redacted(value: &str) -> String {
    if is_safe_tenant(value) {
        value.to_owned()
    } else {
        "redacted:tenant".to_owned()
    }
}

fn safe_ref_or_redacted(value: &str, redacted: &str) -> String {
    if is_safe_ref(value) {
        value.to_owned()
    } else {
        redacted.to_owned()
    }
}

fn is_safe_tenant(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.starts_with("ten_") && value == trimmed && is_safe_metadata(value)
}

fn is_safe_ref(value: &str) -> bool {
    is_safe_metadata(value) && value.contains(':')
}

fn is_safe_optional_ref(value: Option<&str>) -> bool {
    value.is_none_or(is_safe_ref)
}

fn is_safe_metadata(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && value == trimmed
        && !value.chars().any(char::is_whitespace)
        && !contains_raw_secret_material(value)
        && !contains_raw_content_material(value)
}

fn contains_raw_secret_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("sk-")
        || lower.contains("sk-")
        || lower.contains("bearer")
        || lower.contains("authorization:")
        || lower.contains("api_key=")
        || lower.contains("openai_api_key")
        || lower.contains("private key")
        || lower.contains("-----begin")
        || lower.contains("secret=")
}

fn contains_raw_content_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw model")
        || lower.contains("write an email")
        || lower.contains("customer message")
        || lower.contains("model answer")
        || lower.contains("raw output")
        || lower.contains("payload")
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
    use std::collections::BTreeSet;

    #[test]
    fn domain_source_labels_map_to_expected_trigger_kinds() {
        let sources = [
            TriggerOrchestratorDomainSource::Scheduler,
            TriggerOrchestratorDomainSource::StudioWebhook,
            TriggerOrchestratorDomainSource::SiblingEventBus,
            TriggerOrchestratorDomainSource::ManualUi,
            TriggerOrchestratorDomainSource::ApiCommand,
            TriggerOrchestratorDomainSource::WorkflowSpawn,
            TriggerOrchestratorDomainSource::OntologyProjection,
        ];
        let labels: Vec<&str> = sources.iter().map(|source| source.as_wire()).collect();
        let unique: BTreeSet<&str> = labels.iter().copied().collect();
        assert_eq!(labels.len(), unique.len());
        assert_eq!(
            unique,
            BTreeSet::from([
                "api-command",
                "manual-ui",
                "ontology-projection",
                "scheduler",
                "sibling-event-bus",
                "studio-webhook",
                "workflow-spawn",
            ])
        );
        assert_eq!(
            TriggerOrchestratorDomainSource::Scheduler.expected_kind(),
            TriggerOrchestratorTriggerKind::Cron
        );
        assert_eq!(
            TriggerOrchestratorDomainSource::SiblingEventBus.expected_kind(),
            TriggerOrchestratorTriggerKind::EventBus
        );
    }

    #[test]
    fn cron_domain_accepts_policy_bound_due_schedule_with_scheduler_evidence() {
        let decision = evaluate_trigger_domain(domain_request(
            TriggerOrchestratorDomainSource::Scheduler,
            trigger_request(TriggerOrchestratorTriggerKind::Cron).with_schedule(schedule_due()),
        ));

        assert_eq!(decision.status, TriggerOrchestratorDomainStatus::Accepted);
        assert_eq!(
            decision.kernel_status,
            Some(TriggerOrchestratorDecisionStatus::Accepted)
        );
        assert!(decision.dispatch_required);
        assert_eq!(
            decision.run_idempotency_key.as_deref(),
            Some("idem:trigger-001")
        );
        assert!(
            decision
                .audit_refs
                .contains(&"scheduler:durable-clock-window".to_owned())
        );
        assert!(
            decision
                .audit_refs
                .contains(&"policy-bundle:trigger-v1".to_owned())
        );
        assert!(
            decision
                .non_claim_refs
                .contains(&"no-cedar-evaluation".to_owned())
        );
        assert!(
            decision
                .non_claim_refs
                .contains(&"no-run-creation".to_owned())
        );
    }

    #[test]
    fn domain_denies_scope_drift_before_kernel_dispatch_plan() {
        let mut request = domain_request(
            TriggerOrchestratorDomainSource::Scheduler,
            trigger_request(TriggerOrchestratorTriggerKind::Cron).with_schedule(schedule_due()),
        );
        request.binding.tenant_id = "ten_other".to_owned();

        let decision = evaluate_trigger_domain(request);

        assert_eq!(decision.status, TriggerOrchestratorDomainStatus::Denied);
        assert_eq!(
            decision.denial_kind,
            Some(TriggerOrchestratorDomainDenialKind::ScopeMismatch)
        );
        assert_eq!(decision.kernel_status, None);
        assert!(!decision.dispatch_required);
        assert_eq!(decision.start_run_command_ref, None);
    }

    #[test]
    fn domain_requires_source_specific_evidence_for_scheduler_webhook_and_events() {
        let mut cron = domain_request(
            TriggerOrchestratorDomainSource::Scheduler,
            trigger_request(TriggerOrchestratorTriggerKind::Cron).with_schedule(schedule_due()),
        );
        cron.binding.scheduler_evidence_ref = None;
        let cron_decision = evaluate_trigger_domain(cron);
        assert_eq!(
            cron_decision.denial_kind,
            Some(TriggerOrchestratorDomainDenialKind::MissingSourceEvidence)
        );
        assert!(
            cron_decision
                .audit_refs
                .contains(&"validation:scheduler-evidence-required".to_owned())
        );

        let mut webhook = domain_request(
            TriggerOrchestratorDomainSource::StudioWebhook,
            trigger_request(TriggerOrchestratorTriggerKind::Webhook).with_webhook(webhook_valid()),
        );
        webhook.binding.webhook_auth_evidence_ref = None;
        let webhook_decision = evaluate_trigger_domain(webhook);
        assert_eq!(
            webhook_decision.denial_kind,
            Some(TriggerOrchestratorDomainDenialKind::MissingSourceEvidence)
        );
        assert!(
            webhook_decision
                .audit_refs
                .contains(&"validation:webhook-auth-evidence-required".to_owned())
        );

        let mut event = domain_request(
            TriggerOrchestratorDomainSource::SiblingEventBus,
            trigger_request(TriggerOrchestratorTriggerKind::EventBus).with_event(event_valid()),
        );
        event.binding.event_contract_ref = None;
        let event_decision = evaluate_trigger_domain(event);
        assert_eq!(
            event_decision.denial_kind,
            Some(TriggerOrchestratorDomainDenialKind::MissingSourceEvidence)
        );
        assert!(
            event_decision
                .audit_refs
                .contains(&"validation:event-contract-required".to_owned())
        );
    }

    #[test]
    fn domain_maps_kernel_deferred_denied_and_replay_suppressed_statuses() {
        let mut paused = schedule_due();
        paused.paused = true;
        paused.pause_reason_ref = Some("pause:maintenance".to_owned());
        let deferred = evaluate_trigger_domain(domain_request(
            TriggerOrchestratorDomainSource::Scheduler,
            trigger_request(TriggerOrchestratorTriggerKind::Cron).with_schedule(paused),
        ));
        assert_eq!(deferred.status, TriggerOrchestratorDomainStatus::Deferred);
        assert_eq!(
            deferred.denial_kind,
            Some(TriggerOrchestratorDomainDenialKind::KernelDeferred)
        );
        assert_eq!(
            deferred.kernel_reason,
            Some(TriggerOrchestratorDenialReason::SchedulePaused)
        );

        let mut expired = webhook_valid();
        expired.received_epoch_seconds = expired.expires_epoch_seconds + 1;
        let denied = evaluate_trigger_domain(domain_request(
            TriggerOrchestratorDomainSource::StudioWebhook,
            trigger_request(TriggerOrchestratorTriggerKind::Webhook).with_webhook(expired),
        ));
        assert_eq!(denied.status, TriggerOrchestratorDomainStatus::Denied);
        assert_eq!(
            denied.denial_kind,
            Some(TriggerOrchestratorDomainDenialKind::KernelDenied)
        );
        assert_eq!(
            denied.kernel_reason,
            Some(TriggerOrchestratorDenialReason::WebhookExpired)
        );

        let mut replay =
            trigger_request(TriggerOrchestratorTriggerKind::EventBus).with_event(event_valid());
        replay.replay_mode = true;
        let suppressed = evaluate_trigger_domain(domain_request(
            TriggerOrchestratorDomainSource::SiblingEventBus,
            replay,
        ));
        assert_eq!(
            suppressed.status,
            TriggerOrchestratorDomainStatus::Suppressed
        );
        assert_eq!(
            suppressed.denial_kind,
            Some(TriggerOrchestratorDomainDenialKind::ReplaySuppressed)
        );
        assert!(!suppressed.dispatch_required);
    }

    #[test]
    fn manual_api_workflow_spawn_and_ontology_sources_delegate_after_policy_binding() {
        for (source, kind) in [
            (
                TriggerOrchestratorDomainSource::ManualUi,
                TriggerOrchestratorTriggerKind::Manual,
            ),
            (
                TriggerOrchestratorDomainSource::ApiCommand,
                TriggerOrchestratorTriggerKind::Api,
            ),
            (
                TriggerOrchestratorDomainSource::WorkflowSpawn,
                TriggerOrchestratorTriggerKind::WorkflowSpawn,
            ),
            (
                TriggerOrchestratorDomainSource::OntologyProjection,
                TriggerOrchestratorTriggerKind::Ontology,
            ),
        ] {
            let decision = evaluate_trigger_domain(domain_request(source, trigger_request(kind)));
            assert_eq!(decision.status, TriggerOrchestratorDomainStatus::Accepted);
            assert!(decision.dispatch_required);
            assert!(decision.audit_refs.contains(&format!(
                "trigger-orchestrator-domain:source:{}",
                source.as_wire()
            )));
        }

        let mismatch = evaluate_trigger_domain(domain_request(
            TriggerOrchestratorDomainSource::ApiCommand,
            trigger_request(TriggerOrchestratorTriggerKind::Manual),
        ));
        assert_eq!(mismatch.status, TriggerOrchestratorDomainStatus::Denied);
        assert_eq!(
            mismatch.denial_kind,
            Some(TriggerOrchestratorDomainDenialKind::SourceSurfaceMismatch)
        );
    }

    #[test]
    fn raw_domain_metadata_is_rejected_without_echoing_prompt_payload_or_secret() {
        let mut request = domain_request(
            TriggerOrchestratorDomainSource::StudioWebhook,
            trigger_request(TriggerOrchestratorTriggerKind::Webhook).with_webhook(webhook_valid()),
        );
        request.binding.source_evidence_ref =
            "raw prompt Authorization: Bearer sk-test payload".to_owned();

        let decision = evaluate_trigger_domain(request);

        assert_eq!(decision.status, TriggerOrchestratorDomainStatus::Denied);
        assert_eq!(
            decision.denial_kind,
            Some(TriggerOrchestratorDomainDenialKind::UnsafeMetadata)
        );
        let rendered = format!("{decision:?}");
        assert!(!rendered.contains("raw prompt"));
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("payload"));
    }

    fn domain_request(
        source: TriggerOrchestratorDomainSource,
        trigger_request: TriggerOrchestratorRequest,
    ) -> TriggerOrchestratorDomainRequest {
        TriggerOrchestratorDomainRequest {
            binding: binding_for(trigger_request.kind),
            trigger_request,
            source,
            trace_ref: "trace:trigger-domain".to_owned(),
            correlation_ref: "corr:trigger-domain".to_owned(),
            idempotency_scope_ref: "idem-scope:tenant-trigger".to_owned(),
            dry_run_reason_ref: None,
            evidence_refs: vec!["evidence:domain-unit-test".to_owned()],
        }
    }

    fn binding_for(kind: TriggerOrchestratorTriggerKind) -> TriggerOrchestratorDomainPolicyBinding {
        TriggerOrchestratorDomainPolicyBinding {
            tenant_id: "ten_foundry".to_owned(),
            trigger_id: "trigger:daily-invoice".to_owned(),
            workflow_spec_id: "workflow:invoice-approval".to_owned(),
            version_sha: "sha:abc123".to_owned(),
            active_cell_id: "cell:use1-a".to_owned(),
            principal_id: "principal:system".to_owned(),
            allowed_kind: kind,
            policy_decision_id: "policy-decision:allow-trigger".to_owned(),
            policy_evidence_ref: "policy-evidence:cedar-allow".to_owned(),
            policy_bundle_ref: "policy-bundle:trigger-v1".to_owned(),
            authorization_surface_ref: "authz-surface:trigger-admission".to_owned(),
            source_evidence_ref: "source-evidence:trigger-admission".to_owned(),
            scheduler_evidence_ref: Some("scheduler:durable-clock-window".to_owned()),
            webhook_auth_evidence_ref: Some("webhook-auth:hmac-nonce-bound".to_owned()),
            event_contract_ref: Some("event-contract:cloudevents-v1".to_owned()),
            replay_epoch_ref: "replay-epoch:2026-05-25T000000Z".to_owned(),
            audit_chain_ref: "audit-chain:trigger-domain".to_owned(),
        }
    }

    fn trigger_request(kind: TriggerOrchestratorTriggerKind) -> TriggerOrchestratorRequest {
        TriggerOrchestratorRequest {
            kind,
            context: TriggerOrchestratorPolicyContext {
                tenant_id: "ten_foundry".to_owned(),
                trigger_id: "trigger:daily-invoice".to_owned(),
                workflow_spec_id: "workflow:invoice-approval".to_owned(),
                version_sha: "sha:abc123".to_owned(),
                active_cell_id: "cell:use1-a".to_owned(),
                principal_id: "principal:system".to_owned(),
                policy_decision_id: "policy-decision:allow-trigger".to_owned(),
                policy_evidence_ref: "policy-evidence:cedar-allow".to_owned(),
                replay_epoch_ref: "replay-epoch:2026-05-25T000000Z".to_owned(),
            },
            schedule: None,
            webhook: None,
            event: None,
            trigger_lineage_ref: "lineage:trigger-parent".to_owned(),
            idempotency_key: "idem:trigger-001".to_owned(),
            replay_mode: false,
            dry_run: false,
            evidence_refs: vec!["evidence:kernel-request".to_owned()],
        }
    }

    trait RequestBuilder {
        fn with_schedule(self, schedule: TriggerOrchestratorScheduleMetadata) -> Self;
        fn with_webhook(self, webhook: TriggerOrchestratorWebhookMetadata) -> Self;
        fn with_event(self, event: TriggerOrchestratorEventEnvelope) -> Self;
    }

    impl RequestBuilder for TriggerOrchestratorRequest {
        fn with_schedule(mut self, schedule: TriggerOrchestratorScheduleMetadata) -> Self {
            self.schedule = Some(schedule);
            self
        }

        fn with_webhook(mut self, webhook: TriggerOrchestratorWebhookMetadata) -> Self {
            self.webhook = Some(webhook);
            self
        }

        fn with_event(mut self, event: TriggerOrchestratorEventEnvelope) -> Self {
            self.event = Some(event);
            self
        }
    }

    fn schedule_due() -> TriggerOrchestratorScheduleMetadata {
        TriggerOrchestratorScheduleMetadata {
            cron_expr_ref: "cron:every-hour".to_owned(),
            timezone_ref: "tz:America-New_York".to_owned(),
            due_epoch_seconds: 1_750_000_000,
            observed_epoch_seconds: 1_750_000_008,
            catchup_window_seconds: 10,
            overlap_policy: TriggerOrchestratorOverlapPolicy::BufferOne,
            paused: false,
            pause_reason_ref: None,
            last_fired_epoch_seconds: Some(1_749_996_400),
        }
    }

    fn webhook_valid() -> TriggerOrchestratorWebhookMetadata {
        TriggerOrchestratorWebhookMetadata {
            endpoint_ref: "endpoint:webhook-invoice".to_owned(),
            signature_ref: "signature:webhook-headers".to_owned(),
            nonce_ref: "nonce:webhook-001".to_owned(),
            hmac_key_ref: "hmac-key:webhook-signing".to_owned(),
            received_epoch_seconds: 1_750_000_001,
            expires_epoch_seconds: 1_750_000_061,
        }
    }

    fn event_valid() -> TriggerOrchestratorEventEnvelope {
        TriggerOrchestratorEventEnvelope {
            event_id: "event:invoice-approved-001".to_owned(),
            source: "https://events.oyatie.example/workflow".to_owned(),
            event_type: "com.oyatie.workflow.invoice_approved".to_owned(),
            specversion: TRIGGER_ORCHESTRATOR_CLOUDEVENTS_SPECVERSION.to_owned(),
            subject_ref: Some("subject:invoice-123".to_owned()),
            event_time_ref: Some("time:2026-05-25T00:00:00Z".to_owned()),
            correlation_id: "corr:invoice-123".to_owned(),
            idempotency_key: "idem:event-001".to_owned(),
        }
    }
}
