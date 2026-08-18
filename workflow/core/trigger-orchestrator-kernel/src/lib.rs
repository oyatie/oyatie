//! Workflow-engine trigger-orchestrator kernel foundation.
//!
//! This crate owns source-level trigger admission planning for cron, webhook,
//! event-bus, manual, API, workflow-spawn, and ontology trigger surfaces. It
//! validates policy-bound metadata refs, emits deterministic dispatch decisions,
//! suppresses replay dispatch, and performs no scheduler, HTTP, HMAC, event-bus,
//! run-creation, filesystem, network, wall-clock, random, Kubernetes, cloud, or
//! tenant workload scheduling side effects.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub const TRIGGER_ORCHESTRATOR_KERNEL_SURFACE: &str = "workflow-engine.trigger-orchestrator.kernel";
pub const TRIGGER_ORCHESTRATOR_KERNEL_CONTRACT_REF: &str =
    "workflow/workflow-engine/PRD.md#d5-trigger-surface";
pub const TRIGGER_ORCHESTRATOR_CLOUDEVENTS_SPECVERSION: &str = "1.0";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TriggerOrchestratorTriggerKind {
    Cron,
    Webhook,
    EventBus,
    Manual,
    Api,
    WorkflowSpawn,
    Ontology,
}

impl TriggerOrchestratorTriggerKind {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Cron => "cron",
            Self::Webhook => "webhook",
            Self::EventBus => "event-bus",
            Self::Manual => "manual",
            Self::Api => "api",
            Self::WorkflowSpawn => "workflow-spawn",
            Self::Ontology => "ontology",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TriggerOrchestratorOverlapPolicy {
    Skip,
    BufferOne,
    BufferAll,
    CancelOther,
    TerminateOther,
    AllowAll,
}

impl TriggerOrchestratorOverlapPolicy {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Skip => "skip",
            Self::BufferOne => "buffer-one",
            Self::BufferAll => "buffer-all",
            Self::CancelOther => "cancel-other",
            Self::TerminateOther => "terminate-other",
            Self::AllowAll => "allow-all",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TriggerOrchestratorDecisionStatus {
    Accepted,
    Denied,
    Deferred,
    Suppressed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TriggerOrchestratorDenialReason {
    UnsafeMetadata,
    MissingPolicyEvidence,
    MissingScheduleMetadata,
    ScheduleNotDue,
    SchedulePaused,
    ScheduleOutsideCatchupWindow,
    MissingWebhookAuthentication,
    WebhookExpired,
    MissingEventEnvelope,
    MissingIdempotencyKey,
    ReplayModeSuppressed,
    TriggerKindMetadataMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorPolicyContext {
    pub tenant_id: String,           // data_class: INTERNAL_ONLY
    pub trigger_id: String,          // data_class: INTERNAL_ONLY
    pub workflow_spec_id: String,    // data_class: INTERNAL_ONLY
    pub version_sha: String,         // data_class: INTERNAL_ONLY
    pub active_cell_id: String,      // data_class: INTERNAL_ONLY
    pub principal_id: String,        // data_class: INTERNAL_ONLY
    pub policy_decision_id: String,  // data_class: INTERNAL_ONLY
    pub policy_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub replay_epoch_ref: String,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorScheduleMetadata {
    pub cron_expr_ref: String,       // data_class: INTERNAL_ONLY
    pub timezone_ref: String,        // data_class: INTERNAL_ONLY
    pub due_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
    pub observed_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub catchup_window_seconds: u64, // data_class: INTERNAL_ONLY
    pub overlap_policy: TriggerOrchestratorOverlapPolicy, // data_class: PUBLIC
    pub paused: bool,                // data_class: PUBLIC
    pub pause_reason_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub last_fired_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorWebhookMetadata {
    pub endpoint_ref: String,        // data_class: INTERNAL_ONLY
    pub signature_ref: String,       // data_class: INTERNAL_ONLY
    pub nonce_ref: String,           // data_class: INTERNAL_ONLY
    pub hmac_key_ref: String,        // data_class: INTERNAL_ONLY
    pub received_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub expires_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorEventEnvelope {
    pub event_id: String,               // data_class: INTERNAL_ONLY
    pub source: String,                 // data_class: INTERNAL_ONLY
    pub event_type: String,             // data_class: INTERNAL_ONLY
    pub specversion: String,            // data_class: PUBLIC
    pub subject_ref: Option<String>,    // data_class: INTERNAL_ONLY
    pub event_time_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub correlation_id: String,         // data_class: INTERNAL_ONLY
    pub idempotency_key: String,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorRequest {
    pub kind: TriggerOrchestratorTriggerKind, // data_class: PUBLIC
    pub context: TriggerOrchestratorPolicyContext, // data_class: INTERNAL_ONLY
    pub schedule: Option<TriggerOrchestratorScheduleMetadata>, // data_class: INTERNAL_ONLY
    pub webhook: Option<TriggerOrchestratorWebhookMetadata>, // data_class: INTERNAL_ONLY
    pub event: Option<TriggerOrchestratorEventEnvelope>, // data_class: INTERNAL_ONLY
    pub trigger_lineage_ref: String,          // data_class: INTERNAL_ONLY
    pub idempotency_key: String,              // data_class: INTERNAL_ONLY
    pub replay_mode: bool,                    // data_class: PUBLIC
    pub dry_run: bool,                        // data_class: PUBLIC
    pub evidence_refs: Vec<String>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorDecision {
    pub status: TriggerOrchestratorDecisionStatus, // data_class: PUBLIC
    pub reason: Option<TriggerOrchestratorDenialReason>, // data_class: INTERNAL_ONLY
    pub tenant_id: String,                         // data_class: INTERNAL_ONLY
    pub trigger_id: String,                        // data_class: INTERNAL_ONLY
    pub workflow_spec_id: String,                  // data_class: INTERNAL_ONLY
    pub run_idempotency_key: Option<String>,       // data_class: INTERNAL_ONLY
    pub dispatch_required: bool,                   // data_class: PUBLIC
    pub start_run_command_ref: Option<String>,     // data_class: INTERNAL_ONLY
    pub schedule_next_check_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                // data_class: INTERNAL_ONLY
    pub non_claim_refs: Vec<String>,               // data_class: INTERNAL_ONLY
}

pub fn evaluate_trigger(request: TriggerOrchestratorRequest) -> TriggerOrchestratorDecision {
    if let Err(reason) = validate_common_request(&request) {
        return denied_redacted_or_context(&request, reason);
    }

    if let Some(reason) = metadata_shape_denial(&request) {
        return decision_for(
            &request,
            TriggerOrchestratorDecisionStatus::Denied,
            Some(reason),
            false,
            None,
        );
    }

    if request.replay_mode {
        return decision_for(
            &request,
            TriggerOrchestratorDecisionStatus::Suppressed,
            Some(TriggerOrchestratorDenialReason::ReplayModeSuppressed),
            false,
            None,
        );
    }

    match request.kind {
        TriggerOrchestratorTriggerKind::Cron => evaluate_cron(&request),
        TriggerOrchestratorTriggerKind::Webhook => evaluate_webhook(&request),
        TriggerOrchestratorTriggerKind::EventBus => evaluate_event_bus(&request),
        TriggerOrchestratorTriggerKind::Manual
        | TriggerOrchestratorTriggerKind::Api
        | TriggerOrchestratorTriggerKind::WorkflowSpawn
        | TriggerOrchestratorTriggerKind::Ontology => decision_for(
            &request,
            TriggerOrchestratorDecisionStatus::Accepted,
            None,
            !request.dry_run,
            None,
        ),
    }
}

fn evaluate_cron(request: &TriggerOrchestratorRequest) -> TriggerOrchestratorDecision {
    let Some(schedule) = request.schedule.as_ref() else {
        return decision_for(
            request,
            TriggerOrchestratorDecisionStatus::Denied,
            Some(TriggerOrchestratorDenialReason::MissingScheduleMetadata),
            false,
            None,
        );
    };
    if !is_safe_ref(&schedule.cron_expr_ref)
        || !is_safe_ref(&schedule.timezone_ref)
        || !is_safe_optional_ref(schedule.pause_reason_ref.as_deref())
    {
        return denied_redacted_or_context(
            request,
            TriggerOrchestratorDenialReason::UnsafeMetadata,
        );
    }
    if schedule.paused {
        return decision_for(
            request,
            TriggerOrchestratorDecisionStatus::Deferred,
            Some(TriggerOrchestratorDenialReason::SchedulePaused),
            false,
            None,
        );
    }
    if schedule.observed_epoch_seconds < schedule.due_epoch_seconds {
        return decision_for(
            request,
            TriggerOrchestratorDecisionStatus::Deferred,
            Some(TriggerOrchestratorDenialReason::ScheduleNotDue),
            false,
            Some(schedule.due_epoch_seconds),
        );
    }
    if schedule.observed_epoch_seconds
        > schedule
            .due_epoch_seconds
            .saturating_add(schedule.catchup_window_seconds)
    {
        return decision_for(
            request,
            TriggerOrchestratorDecisionStatus::Denied,
            Some(TriggerOrchestratorDenialReason::ScheduleOutsideCatchupWindow),
            false,
            None,
        );
    }
    decision_for(
        request,
        TriggerOrchestratorDecisionStatus::Accepted,
        None,
        !request.dry_run,
        None,
    )
}

fn evaluate_webhook(request: &TriggerOrchestratorRequest) -> TriggerOrchestratorDecision {
    let Some(webhook) = request.webhook.as_ref() else {
        return decision_for(
            request,
            TriggerOrchestratorDecisionStatus::Denied,
            Some(TriggerOrchestratorDenialReason::MissingWebhookAuthentication),
            false,
            None,
        );
    };
    if has_missing_webhook_auth(webhook) {
        return decision_for(
            request,
            TriggerOrchestratorDecisionStatus::Denied,
            Some(TriggerOrchestratorDenialReason::MissingWebhookAuthentication),
            false,
            None,
        );
    }
    if !is_safe_ref(&webhook.endpoint_ref)
        || !is_safe_ref(&webhook.signature_ref)
        || !is_safe_ref(&webhook.nonce_ref)
        || !is_safe_ref(&webhook.hmac_key_ref)
    {
        return denied_redacted_or_context(
            request,
            TriggerOrchestratorDenialReason::UnsafeMetadata,
        );
    }
    if webhook.received_epoch_seconds > webhook.expires_epoch_seconds {
        return decision_for(
            request,
            TriggerOrchestratorDecisionStatus::Denied,
            Some(TriggerOrchestratorDenialReason::WebhookExpired),
            false,
            None,
        );
    }
    decision_for(
        request,
        TriggerOrchestratorDecisionStatus::Accepted,
        None,
        !request.dry_run,
        None,
    )
}

fn evaluate_event_bus(request: &TriggerOrchestratorRequest) -> TriggerOrchestratorDecision {
    let Some(event) = request.event.as_ref() else {
        return decision_for(
            request,
            TriggerOrchestratorDecisionStatus::Denied,
            Some(TriggerOrchestratorDenialReason::MissingEventEnvelope),
            false,
            None,
        );
    };
    if !is_safe_ref(&event.event_id)
        || !is_safe_ref(&event.source)
        || !is_safe_metadata(&event.event_type)
        || event.specversion != TRIGGER_ORCHESTRATOR_CLOUDEVENTS_SPECVERSION
        || !is_safe_optional_ref(event.subject_ref.as_deref())
        || !is_safe_optional_ref(event.event_time_ref.as_deref())
        || !is_safe_ref(&event.correlation_id)
        || !is_safe_ref(&event.idempotency_key)
    {
        return decision_for(
            request,
            TriggerOrchestratorDecisionStatus::Denied,
            Some(TriggerOrchestratorDenialReason::MissingEventEnvelope),
            false,
            None,
        );
    }
    decision_for(
        request,
        TriggerOrchestratorDecisionStatus::Accepted,
        None,
        !request.dry_run,
        None,
    )
}

fn validate_common_request(
    request: &TriggerOrchestratorRequest,
) -> Result<(), TriggerOrchestratorDenialReason> {
    if request.context.policy_decision_id.trim().is_empty()
        || request.context.policy_evidence_ref.trim().is_empty()
    {
        return Err(TriggerOrchestratorDenialReason::MissingPolicyEvidence);
    }
    if request.idempotency_key.trim().is_empty() {
        return Err(TriggerOrchestratorDenialReason::MissingIdempotencyKey);
    }
    if !is_safe_tenant(&request.context.tenant_id)
        || !is_safe_ref(&request.context.trigger_id)
        || !is_safe_ref(&request.context.workflow_spec_id)
        || !is_safe_ref(&request.context.version_sha)
        || !is_safe_ref(&request.context.active_cell_id)
        || !is_safe_ref(&request.context.principal_id)
        || !is_safe_ref(&request.context.policy_decision_id)
        || !is_safe_ref(&request.context.policy_evidence_ref)
        || !is_safe_ref(&request.context.replay_epoch_ref)
        || !is_safe_ref(&request.trigger_lineage_ref)
        || !is_safe_ref(&request.idempotency_key)
        || !request.evidence_refs.iter().all(|value| is_safe_ref(value))
    {
        return Err(TriggerOrchestratorDenialReason::UnsafeMetadata);
    }
    Ok(())
}

fn metadata_shape_denial(
    request: &TriggerOrchestratorRequest,
) -> Option<TriggerOrchestratorDenialReason> {
    match request.kind {
        TriggerOrchestratorTriggerKind::Cron => {
            if request.webhook.is_some() || request.event.is_some() {
                Some(TriggerOrchestratorDenialReason::TriggerKindMetadataMismatch)
            } else {
                None
            }
        }
        TriggerOrchestratorTriggerKind::Webhook => {
            if request.schedule.is_some() || request.event.is_some() {
                Some(TriggerOrchestratorDenialReason::TriggerKindMetadataMismatch)
            } else {
                None
            }
        }
        TriggerOrchestratorTriggerKind::EventBus => {
            if request.schedule.is_some() || request.webhook.is_some() {
                Some(TriggerOrchestratorDenialReason::TriggerKindMetadataMismatch)
            } else {
                None
            }
        }
        TriggerOrchestratorTriggerKind::Manual
        | TriggerOrchestratorTriggerKind::Api
        | TriggerOrchestratorTriggerKind::WorkflowSpawn
        | TriggerOrchestratorTriggerKind::Ontology => {
            if request.schedule.is_some() || request.webhook.is_some() || request.event.is_some() {
                Some(TriggerOrchestratorDenialReason::TriggerKindMetadataMismatch)
            } else {
                None
            }
        }
    }
}

fn decision_for(
    request: &TriggerOrchestratorRequest,
    status: TriggerOrchestratorDecisionStatus,
    reason: Option<TriggerOrchestratorDenialReason>,
    dispatch_required: bool,
    schedule_next_check_epoch_seconds: Option<u64>,
) -> TriggerOrchestratorDecision {
    let start_run_command_ref = dispatch_required.then(|| {
        format!(
            "start-run:{}:{}:{}",
            request.context.tenant_id, request.context.workflow_spec_id, request.idempotency_key
        )
    });
    TriggerOrchestratorDecision {
        status,
        reason,
        tenant_id: request.context.tenant_id.clone(),
        trigger_id: request.context.trigger_id.clone(),
        workflow_spec_id: request.context.workflow_spec_id.clone(),
        run_idempotency_key: dispatch_required.then(|| request.idempotency_key.clone()),
        dispatch_required,
        start_run_command_ref,
        schedule_next_check_epoch_seconds,
        evidence_refs: decision_evidence_refs(request),
        non_claim_refs: non_claim_refs(),
    }
}

fn denied_redacted_or_context(
    request: &TriggerOrchestratorRequest,
    reason: TriggerOrchestratorDenialReason,
) -> TriggerOrchestratorDecision {
    if reason == TriggerOrchestratorDenialReason::UnsafeMetadata {
        return TriggerOrchestratorDecision {
            status: TriggerOrchestratorDecisionStatus::Denied,
            reason: Some(reason),
            tenant_id: safe_tenant_or_redacted(&request.context.tenant_id),
            trigger_id: safe_ref_or_redacted(&request.context.trigger_id, "redacted:trigger"),
            workflow_spec_id: safe_ref_or_redacted(
                &request.context.workflow_spec_id,
                "redacted:workflow-spec",
            ),
            run_idempotency_key: None,
            dispatch_required: false,
            start_run_command_ref: None,
            schedule_next_check_epoch_seconds: None,
            evidence_refs: vec!["trigger-orchestrator:unsafe-metadata".to_owned()],
            non_claim_refs: non_claim_refs(),
        };
    }
    if is_safe_tenant(&request.context.tenant_id)
        && is_safe_ref(&request.context.trigger_id)
        && is_safe_ref(&request.context.workflow_spec_id)
    {
        decision_for(
            request,
            TriggerOrchestratorDecisionStatus::Denied,
            Some(reason),
            false,
            None,
        )
    } else {
        TriggerOrchestratorDecision {
            status: TriggerOrchestratorDecisionStatus::Denied,
            reason: Some(reason),
            tenant_id: "redacted:tenant".to_owned(),
            trigger_id: "redacted:trigger".to_owned(),
            workflow_spec_id: "redacted:workflow-spec".to_owned(),
            run_idempotency_key: None,
            dispatch_required: false,
            start_run_command_ref: None,
            schedule_next_check_epoch_seconds: None,
            evidence_refs: vec!["trigger-orchestrator:unsafe-metadata".to_owned()],
            non_claim_refs: non_claim_refs(),
        }
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

fn decision_evidence_refs(request: &TriggerOrchestratorRequest) -> Vec<String> {
    let mut refs = vec![
        format!("trigger-kind:{}", request.kind.as_wire()),
        "contract:workflow-prd-d5-trigger-surface".to_owned(),
        request.context.policy_evidence_ref.clone(),
        request.context.replay_epoch_ref.clone(),
        request.trigger_lineage_ref.clone(),
    ];
    refs.extend(request.evidence_refs.clone());
    if let Some(schedule) = request.schedule.as_ref() {
        refs.push(format!(
            "overlap-policy:{}",
            schedule.overlap_policy.as_wire()
        ));
        refs.push(schedule.cron_expr_ref.clone());
        refs.push(schedule.timezone_ref.clone());
        if let Some(pause_reason_ref) = schedule.pause_reason_ref.clone() {
            refs.push(pause_reason_ref);
        }
    }
    if let Some(webhook) = request.webhook.as_ref() {
        refs.push(webhook.endpoint_ref.clone());
        refs.push(webhook.signature_ref.clone());
        refs.push(webhook.nonce_ref.clone());
        refs.push(webhook.hmac_key_ref.clone());
    }
    if let Some(event) = request.event.as_ref() {
        refs.push(event.event_id.clone());
        refs.push(event.source.clone());
        refs.push(format!("cloudevents-specversion:{}", event.specversion));
        refs.push(event.correlation_id.clone());
        refs.push(event.idempotency_key.clone());
        if let Some(subject_ref) = event.subject_ref.clone() {
            refs.push(subject_ref);
        }
        if let Some(event_time_ref) = event.event_time_ref.clone() {
            refs.push(event_time_ref);
        }
    }
    sorted_unique(refs)
}

fn non_claim_refs() -> Vec<String> {
    vec![
        "no-scheduler-runtime".to_owned(),
        "no-webhook-server".to_owned(),
        "no-hmac-verification".to_owned(),
        "no-nonce-persistence".to_owned(),
        "no-event-bus-consumer".to_owned(),
        "no-run-start-side-effect".to_owned(),
        "no-durable-trigger-store".to_owned(),
        "no-cloud-deployment".to_owned(),
        "no-tenant-workload-scheduling".to_owned(),
        "no-hyperscaler-claim".to_owned(),
    ]
}

fn has_missing_webhook_auth(webhook: &TriggerOrchestratorWebhookMetadata) -> bool {
    webhook.endpoint_ref.trim().is_empty()
        || webhook.signature_ref.trim().is_empty()
        || webhook.nonce_ref.trim().is_empty()
        || webhook.hmac_key_ref.trim().is_empty()
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
    fn trigger_kind_labels_cover_prd_surfaces_and_are_unique() {
        let kinds = [
            TriggerOrchestratorTriggerKind::Cron,
            TriggerOrchestratorTriggerKind::Webhook,
            TriggerOrchestratorTriggerKind::EventBus,
            TriggerOrchestratorTriggerKind::Manual,
            TriggerOrchestratorTriggerKind::Api,
            TriggerOrchestratorTriggerKind::WorkflowSpawn,
            TriggerOrchestratorTriggerKind::Ontology,
        ];
        let labels: Vec<&str> = kinds.iter().map(|kind| kind.as_wire()).collect();
        let unique: BTreeSet<&str> = labels.iter().copied().collect();
        assert_eq!(labels.len(), unique.len());
        assert_eq!(
            unique,
            BTreeSet::from([
                "api",
                "cron",
                "event-bus",
                "manual",
                "ontology",
                "webhook",
                "workflow-spawn",
            ])
        );
        assert_eq!(
            TRIGGER_ORCHESTRATOR_KERNEL_SURFACE,
            "workflow-engine.trigger-orchestrator.kernel"
        );
    }

    #[test]
    fn cron_schedule_accepts_due_window_and_preserves_overlap_policy_without_clock_reads() {
        let request =
            request_for(TriggerOrchestratorTriggerKind::Cron).with_schedule(schedule_due());
        let decision = evaluate_trigger(request);

        assert_eq!(decision.status, TriggerOrchestratorDecisionStatus::Accepted);
        assert_eq!(decision.reason, None);
        assert!(decision.dispatch_required);
        assert_eq!(
            decision.run_idempotency_key.as_deref(),
            Some("idem:trigger-001")
        );
        assert!(
            decision
                .start_run_command_ref
                .as_deref()
                .unwrap()
                .contains("start-run:ten_foundry:workflow:invoice-approval:idem:trigger-001")
        );
        assert!(
            decision
                .evidence_refs
                .contains(&"overlap-policy:buffer-one".to_owned())
        );
        assert!(
            decision
                .non_claim_refs
                .contains(&"no-scheduler-runtime".to_owned())
        );
    }

    #[test]
    fn cron_schedule_defers_paused_or_not_due_and_denies_stale_catchup() {
        let mut paused = schedule_due();
        paused.paused = true;
        paused.pause_reason_ref = Some("pause:maintenance".to_owned());
        let paused_decision = evaluate_trigger(
            request_for(TriggerOrchestratorTriggerKind::Cron).with_schedule(paused),
        );
        assert_eq!(
            paused_decision.status,
            TriggerOrchestratorDecisionStatus::Deferred
        );
        assert_eq!(
            paused_decision.reason,
            Some(TriggerOrchestratorDenialReason::SchedulePaused)
        );
        assert!(!paused_decision.dispatch_required);

        let mut not_due = schedule_due();
        not_due.observed_epoch_seconds = not_due.due_epoch_seconds - 1;
        let not_due_decision = evaluate_trigger(
            request_for(TriggerOrchestratorTriggerKind::Cron).with_schedule(not_due),
        );
        assert_eq!(
            not_due_decision.status,
            TriggerOrchestratorDecisionStatus::Deferred
        );
        assert_eq!(
            not_due_decision.reason,
            Some(TriggerOrchestratorDenialReason::ScheduleNotDue)
        );
        assert_eq!(
            not_due_decision.schedule_next_check_epoch_seconds,
            Some(1_750_000_000)
        );

        let mut stale = schedule_due();
        stale.observed_epoch_seconds = stale.due_epoch_seconds + stale.catchup_window_seconds + 1;
        let stale_decision = evaluate_trigger(
            request_for(TriggerOrchestratorTriggerKind::Cron).with_schedule(stale),
        );
        assert_eq!(
            stale_decision.status,
            TriggerOrchestratorDecisionStatus::Denied
        );
        assert_eq!(
            stale_decision.reason,
            Some(TriggerOrchestratorDenialReason::ScheduleOutsideCatchupWindow)
        );
        assert!(!stale_decision.dispatch_required);
    }

    #[test]
    fn webhook_trigger_requires_signature_nonce_hmac_refs_and_expiry() {
        let accepted = evaluate_trigger(
            request_for(TriggerOrchestratorTriggerKind::Webhook).with_webhook(webhook_valid()),
        );
        assert_eq!(accepted.status, TriggerOrchestratorDecisionStatus::Accepted);
        assert!(accepted.dispatch_required);
        assert!(
            accepted
                .evidence_refs
                .contains(&"hmac-key:webhook-signing".to_owned())
        );

        let mut missing = webhook_valid();
        missing.nonce_ref.clear();
        let missing_decision = evaluate_trigger(
            request_for(TriggerOrchestratorTriggerKind::Webhook).with_webhook(missing),
        );
        assert_eq!(
            missing_decision.status,
            TriggerOrchestratorDecisionStatus::Denied
        );
        assert_eq!(
            missing_decision.reason,
            Some(TriggerOrchestratorDenialReason::MissingWebhookAuthentication)
        );

        let mut expired = webhook_valid();
        expired.received_epoch_seconds = expired.expires_epoch_seconds + 1;
        let expired_decision = evaluate_trigger(
            request_for(TriggerOrchestratorTriggerKind::Webhook).with_webhook(expired),
        );
        assert_eq!(
            expired_decision.status,
            TriggerOrchestratorDecisionStatus::Denied
        );
        assert_eq!(
            expired_decision.reason,
            Some(TriggerOrchestratorDenialReason::WebhookExpired)
        );
    }

    #[test]
    fn event_bus_trigger_requires_cloudevents_identity_and_idempotency() {
        let accepted = evaluate_trigger(
            request_for(TriggerOrchestratorTriggerKind::EventBus).with_event(event_valid()),
        );
        assert_eq!(accepted.status, TriggerOrchestratorDecisionStatus::Accepted);
        assert!(accepted.dispatch_required);
        assert!(
            accepted
                .evidence_refs
                .contains(&"cloudevents-specversion:1.0".to_owned())
        );

        let mut missing_idempotency = event_valid();
        missing_idempotency.idempotency_key.clear();
        let missing_decision = evaluate_trigger(
            request_for(TriggerOrchestratorTriggerKind::EventBus).with_event(missing_idempotency),
        );
        assert_eq!(
            missing_decision.status,
            TriggerOrchestratorDecisionStatus::Denied
        );
        assert_eq!(
            missing_decision.reason,
            Some(TriggerOrchestratorDenialReason::MissingEventEnvelope)
        );

        let mut bad_version = event_valid();
        bad_version.specversion = "0.3".to_owned();
        let bad_version_decision = evaluate_trigger(
            request_for(TriggerOrchestratorTriggerKind::EventBus).with_event(bad_version),
        );
        assert_eq!(
            bad_version_decision.status,
            TriggerOrchestratorDecisionStatus::Denied
        );
        assert_eq!(
            bad_version_decision.reason,
            Some(TriggerOrchestratorDenialReason::MissingEventEnvelope)
        );
    }

    #[test]
    fn manual_api_workflow_spawn_and_ontology_require_policy_bound_context() {
        for kind in [
            TriggerOrchestratorTriggerKind::Manual,
            TriggerOrchestratorTriggerKind::Api,
            TriggerOrchestratorTriggerKind::WorkflowSpawn,
            TriggerOrchestratorTriggerKind::Ontology,
        ] {
            let accepted = evaluate_trigger(request_for(kind));
            assert_eq!(accepted.status, TriggerOrchestratorDecisionStatus::Accepted);
            assert!(accepted.dispatch_required);
            assert_eq!(accepted.tenant_id, "ten_foundry");
            assert_eq!(accepted.workflow_spec_id, "workflow:invoice-approval");

            let mut missing_policy = request_for(kind);
            missing_policy.context.policy_evidence_ref.clear();
            let missing_policy_decision = evaluate_trigger(missing_policy);
            assert_eq!(
                missing_policy_decision.status,
                TriggerOrchestratorDecisionStatus::Denied
            );
            assert_eq!(
                missing_policy_decision.reason,
                Some(TriggerOrchestratorDenialReason::MissingPolicyEvidence)
            );
        }

        let mismatched = evaluate_trigger(
            request_for(TriggerOrchestratorTriggerKind::Manual).with_schedule(schedule_due()),
        );
        assert_eq!(mismatched.status, TriggerOrchestratorDecisionStatus::Denied);
        assert_eq!(
            mismatched.reason,
            Some(TriggerOrchestratorDenialReason::TriggerKindMetadataMismatch)
        );
    }

    #[test]
    fn replay_mode_suppresses_dispatch_and_raw_metadata_is_rejected_without_echo() {
        let mut replay =
            request_for(TriggerOrchestratorTriggerKind::Webhook).with_webhook(webhook_valid());
        replay.replay_mode = true;
        let replay_decision = evaluate_trigger(replay);
        assert_eq!(
            replay_decision.status,
            TriggerOrchestratorDecisionStatus::Suppressed
        );
        assert_eq!(
            replay_decision.reason,
            Some(TriggerOrchestratorDenialReason::ReplayModeSuppressed)
        );
        assert!(!replay_decision.dispatch_required);
        assert_eq!(replay_decision.start_run_command_ref, None);

        let mut raw =
            request_for(TriggerOrchestratorTriggerKind::Webhook).with_webhook(webhook_valid());
        raw.webhook.as_mut().unwrap().signature_ref =
            "raw prompt Authorization: Bearer sk-test payload".to_owned();
        let raw_decision = evaluate_trigger(raw);
        assert_eq!(
            raw_decision.status,
            TriggerOrchestratorDecisionStatus::Denied
        );
        assert_eq!(
            raw_decision.reason,
            Some(TriggerOrchestratorDenialReason::UnsafeMetadata)
        );
        let rendered = format!("{raw_decision:?}");
        assert!(!rendered.contains("raw prompt"));
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("payload"));
    }

    fn request_for(kind: TriggerOrchestratorTriggerKind) -> TriggerOrchestratorRequest {
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
            evidence_refs: vec!["evidence:unit-test".to_owned()],
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
