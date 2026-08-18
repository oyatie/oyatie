//! Workflow-engine trigger-orchestrator usecase foundation.
//!
//! The usecase composes request/idempotency/trace validation, deterministic
//! idempotency replay/conflict handling, and the policy/source-bound trigger
//! domain. It returns metadata-only receipts and audit events; it performs no
//! concrete storage, scheduler execution, Cedar evaluation, webhook serving,
//! HMAC verification, event-bus consumption, run creation, network, filesystem,
//! wall-clock, random, Kubernetes, cloud, or tenant workload scheduling work.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

pub use workflow_trigger_orchestrator_domain::{
    TRIGGER_ORCHESTRATOR_CLOUDEVENTS_SPECVERSION, TRIGGER_ORCHESTRATOR_DOMAIN_CONTRACT_REF,
    TRIGGER_ORCHESTRATOR_DOMAIN_SURFACE, TRIGGER_ORCHESTRATOR_KERNEL_CONTRACT_REF,
    TRIGGER_ORCHESTRATOR_KERNEL_SURFACE, TriggerOrchestratorDecisionStatus,
    TriggerOrchestratorDenialReason, TriggerOrchestratorDomainDecision,
    TriggerOrchestratorDomainDenialKind, TriggerOrchestratorDomainPolicyBinding,
    TriggerOrchestratorDomainRequest, TriggerOrchestratorDomainSource,
    TriggerOrchestratorDomainStatus, TriggerOrchestratorEventEnvelope,
    TriggerOrchestratorOverlapPolicy, TriggerOrchestratorPolicyContext, TriggerOrchestratorRequest,
    TriggerOrchestratorScheduleMetadata, TriggerOrchestratorTriggerKind,
    TriggerOrchestratorWebhookMetadata, evaluate_trigger_domain,
};

pub const TRIGGER_ORCHESTRATOR_USECASE_SURFACE: &str =
    "workflow-engine.trigger-orchestrator.usecase";
pub const TRIGGER_ORCHESTRATOR_USECASE_CONTRACT_REF: &str =
    "workflow/workflow-engine/PRD.md#f2-run-trigger-dispatch";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorUsecaseInput {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
    pub trace_ref: String,       // data_class: INTERNAL_ONLY
    pub domain_request: TriggerOrchestratorDomainRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TriggerOrchestratorUsecaseStatus {
    Accepted,
    Deferred,
    DomainDenied,
    IdempotencyConflict,
    InvalidInput,
    Suppressed,
}

impl TriggerOrchestratorUsecaseStatus {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Deferred => "deferred",
            Self::DomainDenied => "domain-denied",
            Self::IdempotencyConflict => "idempotency-conflict",
            Self::InvalidInput => "invalid-input",
            Self::Suppressed => "suppressed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TriggerOrchestratorUsecaseAuditEventKind {
    DomainDenied,
    IdempotencyConflict,
    TriggerAccepted,
    TriggerDeferred,
    TriggerInvalid,
    TriggerRequested,
    TriggerSuppressed,
}

impl TriggerOrchestratorUsecaseAuditEventKind {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::DomainDenied => "domain-denied",
            Self::IdempotencyConflict => "idempotency-conflict",
            Self::TriggerAccepted => "trigger-accepted",
            Self::TriggerDeferred => "trigger-deferred",
            Self::TriggerInvalid => "trigger-invalid",
            Self::TriggerRequested => "trigger-requested",
            Self::TriggerSuppressed => "trigger-suppressed",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorUsecaseAuditEvent {
    pub kind: TriggerOrchestratorUsecaseAuditEventKind, // data_class: INTERNAL_ONLY
    pub tenant_id: String,                              // data_class: INTERNAL_ONLY
    pub trigger_id: String,                             // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorUsecaseReceipt {
    pub status: TriggerOrchestratorUsecaseStatus, // data_class: PUBLIC
    pub domain_status: Option<TriggerOrchestratorDomainStatus>, // data_class: PUBLIC
    pub domain_denial_kind: Option<TriggerOrchestratorDomainDenialKind>, // data_class: INTERNAL_ONLY
    pub kernel_status: Option<TriggerOrchestratorDecisionStatus>,        // data_class: PUBLIC
    pub kernel_reason: Option<TriggerOrchestratorDenialReason>, // data_class: INTERNAL_ONLY
    pub tenant_id: String,                                      // data_class: INTERNAL_ONLY
    pub trigger_id: String,                                     // data_class: INTERNAL_ONLY
    pub workflow_spec_id: String,                               // data_class: INTERNAL_ONLY
    pub dispatch_required: bool,                                // data_class: PUBLIC
    pub run_idempotency_key: Option<String>,                    // data_class: INTERNAL_ONLY
    pub start_run_command_ref: Option<String>,                  // data_class: INTERNAL_ONLY
    pub schedule_next_check_epoch_seconds: Option<u64>,         // data_class: INTERNAL_ONLY
    pub audit_events: Vec<TriggerOrchestratorUsecaseAuditEvent>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                             // data_class: INTERNAL_ONLY
    pub non_claim_refs: Vec<String>,                            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TriggerOrchestratorUsecaseIntent {
    fingerprint: String,
}

#[derive(Default, Debug)]
pub struct TriggerOrchestratorUsecase {
    receipts_by_idempotency_key: BTreeMap<
        String,
        (
            TriggerOrchestratorUsecaseIntent,
            TriggerOrchestratorUsecaseReceipt,
        ),
    >,
}

impl TriggerOrchestratorUsecase {
    pub fn apply(
        &mut self,
        input: TriggerOrchestratorUsecaseInput,
    ) -> TriggerOrchestratorUsecaseReceipt {
        if let Some(receipt) = invalid_input_receipt(&input) {
            return receipt;
        }

        let intent = TriggerOrchestratorUsecaseIntent {
            fingerprint: canonical_fingerprint(&input),
        };
        if let Some((existing_intent, existing_receipt)) =
            self.receipts_by_idempotency_key.get(&input.idempotency_key)
        {
            if existing_intent == &intent {
                return existing_receipt.clone();
            }
            return idempotency_conflict_receipt(&input);
        }

        let requested = requested_event(&input);
        let domain_decision = evaluate_trigger_domain(input.domain_request.clone());
        let receipt = receipt_from_domain_decision(&input, requested, domain_decision);
        self.receipts_by_idempotency_key
            .insert(input.idempotency_key, (intent, receipt.clone()));
        receipt
    }
}

fn invalid_input_receipt(
    input: &TriggerOrchestratorUsecaseInput,
) -> Option<TriggerOrchestratorUsecaseReceipt> {
    let mut refs = collect_invalid_metadata_refs(input);
    if refs.is_empty() {
        return None;
    }
    refs.push("trigger-orchestrator-usecase:invalid-input".to_owned());
    let refs = sorted_unique(refs);
    Some(TriggerOrchestratorUsecaseReceipt {
        status: TriggerOrchestratorUsecaseStatus::InvalidInput,
        domain_status: None,
        domain_denial_kind: None,
        kernel_status: None,
        kernel_reason: None,
        tenant_id: safe_tenant_or_redacted(&input.domain_request.trigger_request.context.tenant_id),
        trigger_id: safe_ref_or_redacted(
            &input.domain_request.trigger_request.context.trigger_id,
            "redacted:trigger",
        ),
        workflow_spec_id: safe_ref_or_redacted(
            &input
                .domain_request
                .trigger_request
                .context
                .workflow_spec_id,
            "redacted:workflow-spec",
        ),
        dispatch_required: false,
        run_idempotency_key: None,
        start_run_command_ref: None,
        schedule_next_check_epoch_seconds: None,
        audit_events: vec![audit_event(
            TriggerOrchestratorUsecaseAuditEventKind::TriggerInvalid,
            &safe_tenant_or_redacted(&input.domain_request.trigger_request.context.tenant_id),
            &safe_ref_or_redacted(
                &input.domain_request.trigger_request.context.trigger_id,
                "redacted:trigger",
            ),
            refs.clone(),
        )],
        evidence_refs: refs,
        non_claim_refs: usecase_non_claim_refs(),
    })
}

fn collect_invalid_metadata_refs(input: &TriggerOrchestratorUsecaseInput) -> Vec<String> {
    let mut refs = Vec::new();
    push_invalid_ref(
        &mut refs,
        "validation:request-id-invalid",
        &input.request_id,
    );
    push_invalid_ref(
        &mut refs,
        "validation:idempotency-key-invalid",
        &input.idempotency_key,
    );
    push_invalid_ref(&mut refs, "validation:trace-ref-invalid", &input.trace_ref);

    let domain = &input.domain_request;
    let binding = &domain.binding;
    push_invalid_tenant(
        &mut refs,
        "validation:domain-binding-tenant-invalid",
        &binding.tenant_id,
    );
    push_invalid_ref(
        &mut refs,
        "validation:domain-binding-trigger-ref-invalid",
        &binding.trigger_id,
    );
    push_invalid_ref(
        &mut refs,
        "validation:domain-binding-workflow-spec-ref-invalid",
        &binding.workflow_spec_id,
    );
    push_invalid_ref(
        &mut refs,
        "validation:domain-binding-version-ref-invalid",
        &binding.version_sha,
    );
    push_invalid_ref(
        &mut refs,
        "validation:domain-binding-cell-ref-invalid",
        &binding.active_cell_id,
    );
    push_invalid_ref(
        &mut refs,
        "validation:domain-binding-principal-ref-invalid",
        &binding.principal_id,
    );
    push_invalid_ref(
        &mut refs,
        "validation:domain-binding-policy-decision-ref-invalid",
        &binding.policy_decision_id,
    );
    push_invalid_ref(
        &mut refs,
        "validation:domain-binding-policy-evidence-ref-invalid",
        &binding.policy_evidence_ref,
    );
    push_invalid_ref(
        &mut refs,
        "validation:domain-binding-policy-bundle-ref-invalid",
        &binding.policy_bundle_ref,
    );
    push_invalid_ref(
        &mut refs,
        "validation:domain-binding-authorization-surface-ref-invalid",
        &binding.authorization_surface_ref,
    );
    push_invalid_ref(
        &mut refs,
        "validation:domain-binding-source-evidence-ref-invalid",
        &binding.source_evidence_ref,
    );
    push_invalid_optional_ref(
        &mut refs,
        "validation:domain-binding-scheduler-evidence-ref-invalid",
        binding.scheduler_evidence_ref.as_deref(),
    );
    push_invalid_optional_ref(
        &mut refs,
        "validation:domain-binding-webhook-auth-evidence-ref-invalid",
        binding.webhook_auth_evidence_ref.as_deref(),
    );
    push_invalid_optional_ref(
        &mut refs,
        "validation:domain-binding-event-contract-ref-invalid",
        binding.event_contract_ref.as_deref(),
    );
    push_invalid_ref(
        &mut refs,
        "validation:domain-binding-replay-epoch-ref-invalid",
        &binding.replay_epoch_ref,
    );
    push_invalid_ref(
        &mut refs,
        "validation:domain-binding-audit-chain-ref-invalid",
        &binding.audit_chain_ref,
    );

    push_invalid_ref(
        &mut refs,
        "validation:domain-trace-ref-invalid",
        &domain.trace_ref,
    );
    push_invalid_ref(
        &mut refs,
        "validation:domain-correlation-ref-invalid",
        &domain.correlation_ref,
    );
    push_invalid_ref(
        &mut refs,
        "validation:domain-idempotency-scope-ref-invalid",
        &domain.idempotency_scope_ref,
    );
    push_invalid_optional_ref(
        &mut refs,
        "validation:domain-dry-run-reason-ref-invalid",
        domain.dry_run_reason_ref.as_deref(),
    );
    push_invalid_ref_vec(
        &mut refs,
        "validation:domain-evidence-ref-invalid",
        &domain.evidence_refs,
    );

    let request = &domain.trigger_request;
    let context = &request.context;
    push_invalid_tenant(
        &mut refs,
        "validation:trigger-context-tenant-invalid",
        &context.tenant_id,
    );
    push_invalid_ref(
        &mut refs,
        "validation:trigger-context-trigger-ref-invalid",
        &context.trigger_id,
    );
    push_invalid_ref(
        &mut refs,
        "validation:trigger-context-workflow-spec-ref-invalid",
        &context.workflow_spec_id,
    );
    push_invalid_ref(
        &mut refs,
        "validation:trigger-context-version-ref-invalid",
        &context.version_sha,
    );
    push_invalid_ref(
        &mut refs,
        "validation:trigger-context-cell-ref-invalid",
        &context.active_cell_id,
    );
    push_invalid_ref(
        &mut refs,
        "validation:trigger-context-principal-ref-invalid",
        &context.principal_id,
    );
    push_invalid_ref(
        &mut refs,
        "validation:trigger-context-policy-decision-ref-invalid",
        &context.policy_decision_id,
    );
    push_invalid_ref(
        &mut refs,
        "validation:trigger-context-policy-evidence-ref-invalid",
        &context.policy_evidence_ref,
    );
    push_invalid_ref(
        &mut refs,
        "validation:trigger-context-replay-epoch-ref-invalid",
        &context.replay_epoch_ref,
    );
    push_invalid_ref(
        &mut refs,
        "validation:trigger-lineage-ref-invalid",
        &request.trigger_lineage_ref,
    );
    push_invalid_ref(
        &mut refs,
        "validation:trigger-idempotency-key-invalid",
        &request.idempotency_key,
    );
    push_invalid_ref_vec(
        &mut refs,
        "validation:trigger-evidence-ref-invalid",
        &request.evidence_refs,
    );

    if let Some(schedule) = request.schedule.as_ref() {
        push_invalid_ref(
            &mut refs,
            "validation:schedule-cron-ref-invalid",
            &schedule.cron_expr_ref,
        );
        push_invalid_ref(
            &mut refs,
            "validation:schedule-timezone-ref-invalid",
            &schedule.timezone_ref,
        );
        push_invalid_optional_ref(
            &mut refs,
            "validation:schedule-pause-reason-ref-invalid",
            schedule.pause_reason_ref.as_deref(),
        );
    }
    if let Some(webhook) = request.webhook.as_ref() {
        push_invalid_ref(
            &mut refs,
            "validation:webhook-endpoint-ref-invalid",
            &webhook.endpoint_ref,
        );
        push_invalid_ref(
            &mut refs,
            "validation:webhook-signature-ref-invalid",
            &webhook.signature_ref,
        );
        push_invalid_ref(
            &mut refs,
            "validation:webhook-nonce-ref-invalid",
            &webhook.nonce_ref,
        );
        push_invalid_ref(
            &mut refs,
            "validation:webhook-hmac-key-ref-invalid",
            &webhook.hmac_key_ref,
        );
    }
    if let Some(event) = request.event.as_ref() {
        push_invalid_ref(&mut refs, "validation:event-id-invalid", &event.event_id);
        push_invalid_ref(&mut refs, "validation:event-source-invalid", &event.source);
        push_invalid_metadata(
            &mut refs,
            "validation:event-type-invalid",
            &event.event_type,
        );
        push_invalid_metadata(
            &mut refs,
            "validation:event-specversion-invalid",
            &event.specversion,
        );
        push_invalid_optional_ref(
            &mut refs,
            "validation:event-subject-ref-invalid",
            event.subject_ref.as_deref(),
        );
        push_invalid_optional_ref(
            &mut refs,
            "validation:event-time-ref-invalid",
            event.event_time_ref.as_deref(),
        );
        push_invalid_ref(
            &mut refs,
            "validation:event-correlation-id-invalid",
            &event.correlation_id,
        );
        push_invalid_ref(
            &mut refs,
            "validation:event-idempotency-key-invalid",
            &event.idempotency_key,
        );
    }
    refs
}

fn idempotency_conflict_receipt(
    input: &TriggerOrchestratorUsecaseInput,
) -> TriggerOrchestratorUsecaseReceipt {
    let refs = sorted_unique(vec![
        input.request_id.clone(),
        input.idempotency_key.clone(),
        input.trace_ref.clone(),
        "trigger-orchestrator-usecase:idempotency-conflict".to_owned(),
    ]);
    let context = &input.domain_request.trigger_request.context;
    TriggerOrchestratorUsecaseReceipt {
        status: TriggerOrchestratorUsecaseStatus::IdempotencyConflict,
        domain_status: None,
        domain_denial_kind: None,
        kernel_status: None,
        kernel_reason: None,
        tenant_id: context.tenant_id.clone(),
        trigger_id: context.trigger_id.clone(),
        workflow_spec_id: context.workflow_spec_id.clone(),
        dispatch_required: false,
        run_idempotency_key: None,
        start_run_command_ref: None,
        schedule_next_check_epoch_seconds: None,
        audit_events: vec![audit_event(
            TriggerOrchestratorUsecaseAuditEventKind::IdempotencyConflict,
            &context.tenant_id,
            &context.trigger_id,
            refs.clone(),
        )],
        evidence_refs: refs,
        non_claim_refs: usecase_non_claim_refs(),
    }
}

fn receipt_from_domain_decision(
    input: &TriggerOrchestratorUsecaseInput,
    requested: TriggerOrchestratorUsecaseAuditEvent,
    domain: TriggerOrchestratorDomainDecision,
) -> TriggerOrchestratorUsecaseReceipt {
    let (status, event_kind) = match domain.status {
        TriggerOrchestratorDomainStatus::Accepted => (
            TriggerOrchestratorUsecaseStatus::Accepted,
            TriggerOrchestratorUsecaseAuditEventKind::TriggerAccepted,
        ),
        TriggerOrchestratorDomainStatus::Deferred => (
            TriggerOrchestratorUsecaseStatus::Deferred,
            TriggerOrchestratorUsecaseAuditEventKind::TriggerDeferred,
        ),
        TriggerOrchestratorDomainStatus::Denied => (
            TriggerOrchestratorUsecaseStatus::DomainDenied,
            TriggerOrchestratorUsecaseAuditEventKind::DomainDenied,
        ),
        TriggerOrchestratorDomainStatus::Suppressed => (
            TriggerOrchestratorUsecaseStatus::Suppressed,
            TriggerOrchestratorUsecaseAuditEventKind::TriggerSuppressed,
        ),
    };
    let mut refs = domain.audit_refs.clone();
    refs.extend([
        input.request_id.clone(),
        input.idempotency_key.clone(),
        input.trace_ref.clone(),
        TRIGGER_ORCHESTRATOR_USECASE_SURFACE.to_owned(),
        format!("trigger-orchestrator-usecase:{}", status.as_wire()),
    ]);
    let refs = sorted_unique(refs);
    let outcome = audit_event(
        event_kind,
        &domain.tenant_id,
        &domain.trigger_id,
        refs.clone(),
    );
    TriggerOrchestratorUsecaseReceipt {
        status,
        domain_status: Some(domain.status),
        domain_denial_kind: domain.denial_kind,
        kernel_status: domain.kernel_status,
        kernel_reason: domain.kernel_reason,
        tenant_id: domain.tenant_id,
        trigger_id: domain.trigger_id,
        workflow_spec_id: domain.workflow_spec_id,
        dispatch_required: domain.dispatch_required,
        run_idempotency_key: domain.run_idempotency_key,
        start_run_command_ref: domain.start_run_command_ref,
        schedule_next_check_epoch_seconds: domain.schedule_next_check_epoch_seconds,
        audit_events: vec![requested, outcome],
        evidence_refs: refs,
        non_claim_refs: sorted_unique([domain.non_claim_refs, usecase_non_claim_refs()].concat()),
    }
}

fn requested_event(
    input: &TriggerOrchestratorUsecaseInput,
) -> TriggerOrchestratorUsecaseAuditEvent {
    let context = &input.domain_request.trigger_request.context;
    audit_event(
        TriggerOrchestratorUsecaseAuditEventKind::TriggerRequested,
        &context.tenant_id,
        &context.trigger_id,
        sorted_unique(vec![
            input.request_id.clone(),
            input.idempotency_key.clone(),
            input.trace_ref.clone(),
            input.domain_request.correlation_ref.clone(),
            input.domain_request.idempotency_scope_ref.clone(),
        ]),
    )
}

fn audit_event(
    kind: TriggerOrchestratorUsecaseAuditEventKind,
    tenant_id: &str,
    trigger_id: &str,
    evidence_refs: Vec<String>,
) -> TriggerOrchestratorUsecaseAuditEvent {
    TriggerOrchestratorUsecaseAuditEvent {
        kind,
        tenant_id: tenant_id.to_owned(),
        trigger_id: trigger_id.to_owned(),
        evidence_refs: sorted_unique(evidence_refs),
    }
}

fn canonical_fingerprint(input: &TriggerOrchestratorUsecaseInput) -> String {
    let domain = &input.domain_request;
    let binding = &domain.binding;
    let request = &domain.trigger_request;
    let context = &request.context;
    let mut parts = Vec::new();

    push_fingerprint_value(&mut parts, "usecase.request_id", &input.request_id);
    push_fingerprint_value(
        &mut parts,
        "usecase.idempotency_key",
        &input.idempotency_key,
    );
    push_fingerprint_value(&mut parts, "usecase.trace_ref", &input.trace_ref);

    push_fingerprint_value(&mut parts, "domain.source", domain.source.as_wire());
    push_fingerprint_value(&mut parts, "domain.trace_ref", &domain.trace_ref);
    push_fingerprint_value(
        &mut parts,
        "domain.correlation_ref",
        &domain.correlation_ref,
    );
    push_fingerprint_value(
        &mut parts,
        "domain.idempotency_scope_ref",
        &domain.idempotency_scope_ref,
    );
    push_fingerprint_optional_ref(
        &mut parts,
        "domain.dry_run_reason_ref",
        domain.dry_run_reason_ref.as_deref(),
    );
    push_fingerprint_sorted_refs(&mut parts, "domain.evidence_refs", &domain.evidence_refs);

    push_fingerprint_value(&mut parts, "binding.tenant_id", &binding.tenant_id);
    push_fingerprint_value(&mut parts, "binding.trigger_id", &binding.trigger_id);
    push_fingerprint_value(
        &mut parts,
        "binding.workflow_spec_id",
        &binding.workflow_spec_id,
    );
    push_fingerprint_value(&mut parts, "binding.version_sha", &binding.version_sha);
    push_fingerprint_value(
        &mut parts,
        "binding.active_cell_id",
        &binding.active_cell_id,
    );
    push_fingerprint_value(&mut parts, "binding.principal_id", &binding.principal_id);
    push_fingerprint_value(
        &mut parts,
        "binding.allowed_kind",
        binding.allowed_kind.as_wire(),
    );
    push_fingerprint_value(
        &mut parts,
        "binding.policy_decision_id",
        &binding.policy_decision_id,
    );
    push_fingerprint_value(
        &mut parts,
        "binding.policy_evidence_ref",
        &binding.policy_evidence_ref,
    );
    push_fingerprint_value(
        &mut parts,
        "binding.policy_bundle_ref",
        &binding.policy_bundle_ref,
    );
    push_fingerprint_value(
        &mut parts,
        "binding.authorization_surface_ref",
        &binding.authorization_surface_ref,
    );
    push_fingerprint_value(
        &mut parts,
        "binding.source_evidence_ref",
        &binding.source_evidence_ref,
    );
    push_fingerprint_optional_ref(
        &mut parts,
        "binding.scheduler_evidence_ref",
        binding.scheduler_evidence_ref.as_deref(),
    );
    push_fingerprint_optional_ref(
        &mut parts,
        "binding.webhook_auth_evidence_ref",
        binding.webhook_auth_evidence_ref.as_deref(),
    );
    push_fingerprint_optional_ref(
        &mut parts,
        "binding.event_contract_ref",
        binding.event_contract_ref.as_deref(),
    );
    push_fingerprint_value(
        &mut parts,
        "binding.replay_epoch_ref",
        &binding.replay_epoch_ref,
    );
    push_fingerprint_value(
        &mut parts,
        "binding.audit_chain_ref",
        &binding.audit_chain_ref,
    );

    push_fingerprint_value(&mut parts, "request.kind", request.kind.as_wire());
    push_fingerprint_value(&mut parts, "context.tenant_id", &context.tenant_id);
    push_fingerprint_value(&mut parts, "context.trigger_id", &context.trigger_id);
    push_fingerprint_value(
        &mut parts,
        "context.workflow_spec_id",
        &context.workflow_spec_id,
    );
    push_fingerprint_value(&mut parts, "context.version_sha", &context.version_sha);
    push_fingerprint_value(
        &mut parts,
        "context.active_cell_id",
        &context.active_cell_id,
    );
    push_fingerprint_value(&mut parts, "context.principal_id", &context.principal_id);
    push_fingerprint_value(
        &mut parts,
        "context.policy_decision_id",
        &context.policy_decision_id,
    );
    push_fingerprint_value(
        &mut parts,
        "context.policy_evidence_ref",
        &context.policy_evidence_ref,
    );
    push_fingerprint_value(
        &mut parts,
        "context.replay_epoch_ref",
        &context.replay_epoch_ref,
    );
    push_fingerprint_value(
        &mut parts,
        "request.trigger_lineage_ref",
        &request.trigger_lineage_ref,
    );
    push_fingerprint_value(
        &mut parts,
        "request.idempotency_key",
        &request.idempotency_key,
    );
    push_fingerprint_bool(&mut parts, "request.replay_mode", request.replay_mode);
    push_fingerprint_bool(&mut parts, "request.dry_run", request.dry_run);
    push_fingerprint_sorted_refs(&mut parts, "request.evidence_refs", &request.evidence_refs);

    if let Some(schedule) = request.schedule.as_ref() {
        push_fingerprint_bool(&mut parts, "request.schedule.present", true);
        push_fingerprint_value(
            &mut parts,
            "schedule.cron_expr_ref",
            &schedule.cron_expr_ref,
        );
        push_fingerprint_value(&mut parts, "schedule.timezone_ref", &schedule.timezone_ref);
        push_fingerprint_u64(
            &mut parts,
            "schedule.due_epoch_seconds",
            schedule.due_epoch_seconds,
        );
        push_fingerprint_u64(
            &mut parts,
            "schedule.observed_epoch_seconds",
            schedule.observed_epoch_seconds,
        );
        push_fingerprint_u64(
            &mut parts,
            "schedule.catchup_window_seconds",
            schedule.catchup_window_seconds,
        );
        push_fingerprint_value(
            &mut parts,
            "schedule.overlap_policy",
            schedule.overlap_policy.as_wire(),
        );
        push_fingerprint_bool(&mut parts, "schedule.paused", schedule.paused);
        push_fingerprint_optional_ref(
            &mut parts,
            "schedule.pause_reason_ref",
            schedule.pause_reason_ref.as_deref(),
        );
        push_fingerprint_optional_u64(
            &mut parts,
            "schedule.last_fired_epoch_seconds",
            schedule.last_fired_epoch_seconds,
        );
    } else {
        push_fingerprint_bool(&mut parts, "request.schedule.present", false);
    }

    if let Some(webhook) = request.webhook.as_ref() {
        push_fingerprint_bool(&mut parts, "request.webhook.present", true);
        push_fingerprint_value(&mut parts, "webhook.endpoint_ref", &webhook.endpoint_ref);
        push_fingerprint_value(&mut parts, "webhook.signature_ref", &webhook.signature_ref);
        push_fingerprint_value(&mut parts, "webhook.nonce_ref", &webhook.nonce_ref);
        push_fingerprint_value(&mut parts, "webhook.hmac_key_ref", &webhook.hmac_key_ref);
        push_fingerprint_u64(
            &mut parts,
            "webhook.received_epoch_seconds",
            webhook.received_epoch_seconds,
        );
        push_fingerprint_u64(
            &mut parts,
            "webhook.expires_epoch_seconds",
            webhook.expires_epoch_seconds,
        );
    } else {
        push_fingerprint_bool(&mut parts, "request.webhook.present", false);
    }

    if let Some(event) = request.event.as_ref() {
        push_fingerprint_bool(&mut parts, "request.event.present", true);
        push_fingerprint_value(&mut parts, "event.event_id", &event.event_id);
        push_fingerprint_value(&mut parts, "event.source", &event.source);
        push_fingerprint_value(&mut parts, "event.event_type", &event.event_type);
        push_fingerprint_value(&mut parts, "event.specversion", &event.specversion);
        push_fingerprint_optional_ref(
            &mut parts,
            "event.subject_ref",
            event.subject_ref.as_deref(),
        );
        push_fingerprint_optional_ref(
            &mut parts,
            "event.event_time_ref",
            event.event_time_ref.as_deref(),
        );
        push_fingerprint_value(&mut parts, "event.correlation_id", &event.correlation_id);
        push_fingerprint_value(&mut parts, "event.idempotency_key", &event.idempotency_key);
    } else {
        push_fingerprint_bool(&mut parts, "request.event.present", false);
    }
    parts.join("\n")
}

fn usecase_non_claim_refs() -> Vec<String> {
    vec![
        "no-concrete-idempotency-store".to_owned(),
        "no-trigger-registry-store".to_owned(),
        "no-policy-engine".to_owned(),
        "no-cedar-evaluation".to_owned(),
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

fn push_invalid_tenant(refs: &mut Vec<String>, label: &str, value: &str) {
    if !is_safe_tenant(value) {
        refs.push(label.to_owned());
    }
}

fn push_invalid_ref(refs: &mut Vec<String>, label: &str, value: &str) {
    if !is_safe_ref(value) {
        refs.push(label.to_owned());
    }
}

fn push_invalid_optional_ref(refs: &mut Vec<String>, label: &str, value: Option<&str>) {
    if !is_safe_optional_ref(value) {
        refs.push(label.to_owned());
    }
}

fn push_invalid_metadata(refs: &mut Vec<String>, label: &str, value: &str) {
    if !is_safe_metadata(value) {
        refs.push(label.to_owned());
    }
}

fn push_invalid_ref_vec(refs: &mut Vec<String>, label: &str, values: &[String]) {
    if !values.iter().all(|value| is_safe_ref(value)) {
        refs.push(label.to_owned());
    }
}

fn push_fingerprint_value(parts: &mut Vec<String>, field: &str, value: &str) {
    parts.push(format!("{field}={}:{}", value.len(), value));
}

fn push_fingerprint_bool(parts: &mut Vec<String>, field: &str, value: bool) {
    push_fingerprint_value(parts, field, if value { "true" } else { "false" });
}

fn push_fingerprint_u64(parts: &mut Vec<String>, field: &str, value: u64) {
    push_fingerprint_value(parts, field, &value.to_string());
}

fn push_fingerprint_optional_ref(parts: &mut Vec<String>, field: &str, value: Option<&str>) {
    match value {
        Some(value) => {
            push_fingerprint_bool(parts, &format!("{field}.present"), true);
            push_fingerprint_value(parts, field, value);
        }
        None => push_fingerprint_bool(parts, &format!("{field}.present"), false),
    }
}

fn push_fingerprint_optional_u64(parts: &mut Vec<String>, field: &str, value: Option<u64>) {
    match value {
        Some(value) => {
            push_fingerprint_bool(parts, &format!("{field}.present"), true);
            push_fingerprint_u64(parts, field, value);
        }
        None => push_fingerprint_bool(parts, &format!("{field}.present"), false),
    }
}

fn push_fingerprint_sorted_refs(parts: &mut Vec<String>, field: &str, values: &[String]) {
    let mut sorted = values.to_vec();
    sorted.sort();
    push_fingerprint_u64(parts, &format!("{field}.count"), sorted.len() as u64);
    for (index, value) in sorted.iter().enumerate() {
        push_fingerprint_value(parts, &format!("{field}[{index}]"), value);
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
    fn usecase_status_and_audit_labels_are_unique() {
        let statuses = [
            TriggerOrchestratorUsecaseStatus::Accepted,
            TriggerOrchestratorUsecaseStatus::Deferred,
            TriggerOrchestratorUsecaseStatus::DomainDenied,
            TriggerOrchestratorUsecaseStatus::IdempotencyConflict,
            TriggerOrchestratorUsecaseStatus::InvalidInput,
            TriggerOrchestratorUsecaseStatus::Suppressed,
        ];
        let labels: Vec<&str> = statuses.iter().map(|status| status.as_wire()).collect();
        let unique: BTreeSet<&str> = labels.iter().copied().collect();
        assert_eq!(labels.len(), unique.len());

        let audit_kinds = [
            TriggerOrchestratorUsecaseAuditEventKind::DomainDenied,
            TriggerOrchestratorUsecaseAuditEventKind::IdempotencyConflict,
            TriggerOrchestratorUsecaseAuditEventKind::TriggerAccepted,
            TriggerOrchestratorUsecaseAuditEventKind::TriggerDeferred,
            TriggerOrchestratorUsecaseAuditEventKind::TriggerInvalid,
            TriggerOrchestratorUsecaseAuditEventKind::TriggerRequested,
            TriggerOrchestratorUsecaseAuditEventKind::TriggerSuppressed,
        ];
        let audit_labels: Vec<&str> = audit_kinds.iter().map(|kind| kind.as_wire()).collect();
        let audit_unique: BTreeSet<&str> = audit_labels.iter().copied().collect();
        assert_eq!(audit_labels.len(), audit_unique.len());
    }

    #[test]
    fn scheduler_due_trigger_returns_metadata_only_accepted_receipt_and_idempotent_replay() {
        let mut usecase = TriggerOrchestratorUsecase::default();
        let input = usecase_input(
            TriggerOrchestratorDomainSource::Scheduler,
            trigger_request(TriggerOrchestratorTriggerKind::Cron).with_schedule(schedule_due()),
        );

        let first = usecase.apply(input.clone());
        let second = usecase.apply(input);

        assert_eq!(first, second);
        assert_eq!(first.status, TriggerOrchestratorUsecaseStatus::Accepted);
        assert_eq!(
            first.domain_status,
            Some(TriggerOrchestratorDomainStatus::Accepted)
        );
        assert_eq!(
            first.kernel_status,
            Some(TriggerOrchestratorDecisionStatus::Accepted)
        );
        assert!(first.dispatch_required);
        assert_eq!(
            first.run_idempotency_key.as_deref(),
            Some("idem:trigger-001")
        );
        assert_eq!(first.audit_events.len(), 2);
        assert!(
            first
                .evidence_refs
                .contains(&TRIGGER_ORCHESTRATOR_USECASE_SURFACE.to_owned())
        );
        assert!(
            first
                .non_claim_refs
                .contains(&"no-concrete-idempotency-store".to_owned())
        );
        assert!(first.non_claim_refs.contains(&"no-run-creation".to_owned()));
    }

    #[test]
    fn idempotency_conflict_prevents_second_domain_evaluation_or_dispatch_plan() {
        let mut usecase = TriggerOrchestratorUsecase::default();
        let accepted = usecase.apply(usecase_input(
            TriggerOrchestratorDomainSource::Scheduler,
            trigger_request(TriggerOrchestratorTriggerKind::Cron).with_schedule(schedule_due()),
        ));
        assert_eq!(accepted.status, TriggerOrchestratorUsecaseStatus::Accepted);

        let mut changed = usecase_input(
            TriggerOrchestratorDomainSource::Scheduler,
            trigger_request(TriggerOrchestratorTriggerKind::Cron).with_schedule(schedule_due()),
        );
        changed.domain_request.trigger_request.idempotency_key = "idem:different-kernel".to_owned();
        let conflict = usecase.apply(changed);

        assert_eq!(
            conflict.status,
            TriggerOrchestratorUsecaseStatus::IdempotencyConflict
        );
        assert_eq!(conflict.domain_status, None);
        assert!(!conflict.dispatch_required);
        assert_eq!(conflict.start_run_command_ref, None);
        assert!(
            conflict
                .evidence_refs
                .contains(&"trigger-orchestrator-usecase:idempotency-conflict".to_owned())
        );
    }

    #[test]
    fn idempotency_conflict_detects_domain_binding_drift_before_replay() {
        let mut usecase = TriggerOrchestratorUsecase::default();
        let accepted = usecase.apply(usecase_input(
            TriggerOrchestratorDomainSource::Scheduler,
            trigger_request(TriggerOrchestratorTriggerKind::Cron).with_schedule(schedule_due()),
        ));
        assert_eq!(accepted.status, TriggerOrchestratorUsecaseStatus::Accepted);

        let mut drifted = usecase_input(
            TriggerOrchestratorDomainSource::Scheduler,
            trigger_request(TriggerOrchestratorTriggerKind::Cron).with_schedule(schedule_due()),
        );
        drifted.domain_request.binding.tenant_id = "ten_other".to_owned();
        let conflict = usecase.apply(drifted);

        assert_eq!(
            conflict.status,
            TriggerOrchestratorUsecaseStatus::IdempotencyConflict
        );
        assert_eq!(conflict.domain_status, None);
        assert!(!conflict.dispatch_required);
        assert_eq!(conflict.start_run_command_ref, None);
    }

    #[test]
    fn invalid_request_metadata_is_rejected_without_echo() {
        let mut usecase = TriggerOrchestratorUsecase::default();
        let mut input = usecase_input(
            TriggerOrchestratorDomainSource::StudioWebhook,
            trigger_request(TriggerOrchestratorTriggerKind::Webhook).with_webhook(webhook_valid()),
        );
        input.trace_ref = "raw prompt Authorization: Bearer sk-test payload".to_owned();

        let receipt = usecase.apply(input);

        assert_eq!(
            receipt.status,
            TriggerOrchestratorUsecaseStatus::InvalidInput
        );
        assert_eq!(receipt.domain_status, None);
        assert!(!receipt.dispatch_required);
        let rendered = format!("{receipt:?}");
        assert!(!rendered.contains("raw prompt"));
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("payload"));
    }

    #[test]
    fn unsafe_nested_domain_metadata_is_rejected_before_requested_audit_or_echo() {
        let mut usecase = TriggerOrchestratorUsecase::default();
        let mut input = usecase_input(
            TriggerOrchestratorDomainSource::Scheduler,
            trigger_request(TriggerOrchestratorTriggerKind::Cron).with_schedule(schedule_due()),
        );
        input.domain_request.correlation_ref =
            "corr:raw prompt Authorization: Bearer sk-test payload".to_owned();
        input.domain_request.binding.policy_bundle_ref = "policy-bundle:raw payload".to_owned();

        let receipt = usecase.apply(input);

        assert_eq!(
            receipt.status,
            TriggerOrchestratorUsecaseStatus::InvalidInput
        );
        assert_eq!(receipt.audit_events.len(), 1);
        assert_eq!(
            receipt.audit_events[0].kind,
            TriggerOrchestratorUsecaseAuditEventKind::TriggerInvalid
        );
        assert!(
            receipt
                .evidence_refs
                .contains(&"validation:domain-correlation-ref-invalid".to_owned())
        );
        assert!(
            receipt
                .evidence_refs
                .contains(&"validation:domain-binding-policy-bundle-ref-invalid".to_owned())
        );
        let rendered = format!("{receipt:?}");
        assert!(!rendered.contains("raw prompt"));
        assert!(!rendered.contains("Authorization"));
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("payload"));
    }

    #[test]
    fn domain_denial_is_mapped_to_fail_closed_receipt() {
        let mut usecase = TriggerOrchestratorUsecase::default();
        let mut input = usecase_input(
            TriggerOrchestratorDomainSource::Scheduler,
            trigger_request(TriggerOrchestratorTriggerKind::Cron).with_schedule(schedule_due()),
        );
        input.domain_request.binding.tenant_id = "ten_other".to_owned();

        let receipt = usecase.apply(input);

        assert_eq!(
            receipt.status,
            TriggerOrchestratorUsecaseStatus::DomainDenied
        );
        assert_eq!(
            receipt.domain_status,
            Some(TriggerOrchestratorDomainStatus::Denied)
        );
        assert_eq!(
            receipt.domain_denial_kind,
            Some(TriggerOrchestratorDomainDenialKind::ScopeMismatch)
        );
        assert_eq!(receipt.kernel_status, None);
        assert!(!receipt.dispatch_required);
        assert!(
            receipt
                .evidence_refs
                .contains(&"trigger-orchestrator-domain:scope-mismatch".to_owned())
        );
    }

    #[test]
    fn deferred_denied_and_replay_suppressed_domain_outcomes_are_preserved() {
        let mut usecase = TriggerOrchestratorUsecase::default();
        let mut paused = schedule_due();
        paused.paused = true;
        paused.pause_reason_ref = Some("pause:maintenance".to_owned());
        let deferred = usecase.apply(usecase_input(
            TriggerOrchestratorDomainSource::Scheduler,
            trigger_request(TriggerOrchestratorTriggerKind::Cron).with_schedule(paused),
        ));
        assert_eq!(deferred.status, TriggerOrchestratorUsecaseStatus::Deferred);
        assert_eq!(
            deferred.kernel_reason,
            Some(TriggerOrchestratorDenialReason::SchedulePaused)
        );
        assert!(!deferred.dispatch_required);

        let mut expired = webhook_valid();
        expired.received_epoch_seconds = expired.expires_epoch_seconds + 1;
        let denied = usecase.apply(usecase_input_with_key(
            "idem:usecase-denied",
            TriggerOrchestratorDomainSource::StudioWebhook,
            trigger_request(TriggerOrchestratorTriggerKind::Webhook).with_webhook(expired),
        ));
        assert_eq!(
            denied.status,
            TriggerOrchestratorUsecaseStatus::DomainDenied
        );
        assert_eq!(
            denied.kernel_reason,
            Some(TriggerOrchestratorDenialReason::WebhookExpired)
        );

        let mut replay =
            trigger_request(TriggerOrchestratorTriggerKind::EventBus).with_event(event_valid());
        replay.replay_mode = true;
        let suppressed = usecase.apply(usecase_input_with_key(
            "idem:usecase-replay",
            TriggerOrchestratorDomainSource::SiblingEventBus,
            replay,
        ));
        assert_eq!(
            suppressed.status,
            TriggerOrchestratorUsecaseStatus::Suppressed
        );
        assert_eq!(
            suppressed.domain_denial_kind,
            Some(TriggerOrchestratorDomainDenialKind::ReplaySuppressed)
        );
        assert!(!suppressed.dispatch_required);
    }

    #[test]
    fn manual_api_workflow_spawn_and_ontology_sources_are_metadata_only_accepted() {
        let mut usecase = TriggerOrchestratorUsecase::default();
        for (index, (source, kind)) in [
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
        ]
        .into_iter()
        .enumerate()
        {
            let receipt = usecase.apply(usecase_input_with_key(
                &format!("idem:manual-family-{index}"),
                source,
                trigger_request(kind),
            ));
            assert_eq!(receipt.status, TriggerOrchestratorUsecaseStatus::Accepted);
            assert!(receipt.dispatch_required);
            assert!(receipt.evidence_refs.contains(&format!(
                "trigger-orchestrator-domain:source:{}",
                source.as_wire()
            )));
        }
    }

    fn usecase_input(
        source: TriggerOrchestratorDomainSource,
        trigger_request: TriggerOrchestratorRequest,
    ) -> TriggerOrchestratorUsecaseInput {
        usecase_input_with_key("idem:usecase-001", source, trigger_request)
    }

    fn usecase_input_with_key(
        idempotency_key: &str,
        source: TriggerOrchestratorDomainSource,
        trigger_request: TriggerOrchestratorRequest,
    ) -> TriggerOrchestratorUsecaseInput {
        TriggerOrchestratorUsecaseInput {
            request_id: "request:trigger-usecase".to_owned(),
            idempotency_key: idempotency_key.to_owned(),
            trace_ref: "trace:trigger-usecase".to_owned(),
            domain_request: TriggerOrchestratorDomainRequest {
                binding: binding_for(trigger_request.kind),
                trigger_request,
                source,
                trace_ref: "trace:trigger-domain".to_owned(),
                correlation_ref: "corr:trigger-domain".to_owned(),
                idempotency_scope_ref: "idem-scope:tenant-trigger".to_owned(),
                dry_run_reason_ref: None,
                evidence_refs: vec!["evidence:usecase-unit-test".to_owned()],
            },
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
