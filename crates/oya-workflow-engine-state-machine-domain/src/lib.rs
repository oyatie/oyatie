//! Workflow-engine state-machine domain foundation.
//!
//! The domain layer binds state-machine transition events to tenant/spec policy
//! evidence before delegating to the pure kernel. It performs no storage,
//! network, wall-clock, random, signing, queue, Postgres, Valkey, or cloud
//! runtime work.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use oya_workflow_engine_state_machine_kernel::{
    StateCheckpoint, StepStatus, TransitionDecision, TransitionDenial, TransitionDenialReason,
    TransitionEventValidationError, WorkflowEventKind, WorkflowRunStatus, WorkflowTransitionEvent,
    evaluate_transition,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TransitionOrigin {
    ApiCommand,
    TriggerOrchestrator,
    WorkerReplay,
}

impl TransitionOrigin {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::ApiCommand => "api-command",
            Self::TriggerOrchestrator => "trigger-orchestrator",
            Self::WorkerReplay => "worker-replay",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowStateMachineDomainRequest {
    pub current_checkpoint: Option<StateCheckpoint>, // data_class: INTERNAL_ONLY
    pub event: WorkflowTransitionEvent,              // data_class: INTERNAL_ONLY
    pub expected_tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub expected_spec_id: String,                    // data_class: INTERNAL_ONLY
    pub expected_version_sha: String,                // data_class: INTERNAL_ONLY
    pub policy_evidence_ref: String,                 // data_class: INTERNAL_ONLY
    pub spec_integrity_ref: String,                  // data_class: INTERNAL_ONLY
    pub replay_epoch_ref: String,                    // data_class: INTERNAL_ONLY
    pub origin: TransitionOrigin,                    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DomainTransitionDenialKind {
    KernelDenied,
    MissingEvidence,
    ScopeMismatch,
    UnsafeMetadata,
}

impl DomainTransitionDenialKind {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::KernelDenied => "kernel-denied",
            Self::MissingEvidence => "missing-evidence",
            Self::ScopeMismatch => "scope-mismatch",
            Self::UnsafeMetadata => "unsafe-metadata",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainTransitionReceipt {
    pub checkpoint: StateCheckpoint, // data_class: INTERNAL_ONLY
    pub origin: TransitionOrigin,    // data_class: INTERNAL_ONLY
    pub audit_refs: Vec<String>,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainTransitionDenial {
    pub kind: DomainTransitionDenialKind, // data_class: INTERNAL_ONLY
    pub kernel_reason: Option<TransitionDenialReason>, // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub run_id: String,                   // data_class: INTERNAL_ONLY
    pub audit_refs: Vec<String>,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainTransitionDecision {
    Applied(DomainTransitionReceipt),
    Denied(DomainTransitionDenial),
}

impl DomainTransitionDecision {
    pub fn expect_applied(self) -> DomainTransitionReceipt {
        match self {
            Self::Applied(receipt) => receipt,
            Self::Denied(denial) => panic!("expected applied domain transition, got {denial:?}"),
        }
    }

    pub fn expect_denied(self) -> DomainTransitionDenial {
        match self {
            Self::Applied(receipt) => {
                panic!("expected denied domain transition, got {receipt:?}")
            }
            Self::Denied(denial) => denial,
        }
    }
}

pub fn evaluate_domain_transition(
    request: WorkflowStateMachineDomainRequest,
) -> DomainTransitionDecision {
    if let Some(denial) = preflight_denial(&request) {
        return DomainTransitionDecision::Denied(denial);
    }

    let current = request.current_checkpoint.as_ref();
    match evaluate_transition(current, request.event.clone()) {
        TransitionDecision::Applied(checkpoint) => {
            let audit_refs = domain_audit_refs(&request, &checkpoint.evidence_refs, "applied");
            DomainTransitionDecision::Applied(DomainTransitionReceipt {
                checkpoint,
                origin: request.origin,
                audit_refs,
            })
        }
        TransitionDecision::Denied(denial) => {
            let mut audit_refs = denial.evidence_refs.clone();
            audit_refs.extend(domain_refs(&request));
            audit_refs.push("workflow-state-machine-domain:kernel-denied".to_owned());
            audit_refs.push(format!(
                "workflow-state-machine-domain:kernel:{}",
                kernel_denial_label(denial.reason)
            ));
            DomainTransitionDecision::Denied(DomainTransitionDenial {
                kind: DomainTransitionDenialKind::KernelDenied,
                kernel_reason: Some(denial.reason),
                tenant_id: denial.tenant_id,
                run_id: denial.run_id,
                audit_refs: sorted_unique(audit_refs),
            })
        }
    }
}

fn preflight_denial(request: &WorkflowStateMachineDomainRequest) -> Option<DomainTransitionDenial> {
    let mut missing = Vec::new();
    if request.policy_evidence_ref.trim().is_empty() {
        missing.push("validation:policy-evidence-required".to_owned());
    }
    if request.spec_integrity_ref.trim().is_empty() {
        missing.push("validation:spec-integrity-required".to_owned());
    }
    if request.replay_epoch_ref.trim().is_empty() {
        missing.push("validation:replay-epoch-required".to_owned());
    }
    if !missing.is_empty() {
        return Some(domain_denial(
            request,
            DomainTransitionDenialKind::MissingEvidence,
            None,
            missing,
        ));
    }

    if has_unsafe_metadata(request) {
        return Some(domain_denial(
            request,
            DomainTransitionDenialKind::UnsafeMetadata,
            None,
            vec!["workflow-state-machine-domain:unsafe-metadata".to_owned()],
        ));
    }

    if request.expected_tenant_id != request.event.tenant_id
        || request.expected_spec_id != request.event.spec_id
        || request.expected_version_sha != request.event.version_sha
        || request
            .current_checkpoint
            .as_ref()
            .is_some_and(|checkpoint| {
                checkpoint.tenant_id != request.expected_tenant_id
                    || checkpoint.spec_id != request.expected_spec_id
                    || checkpoint.version_sha != request.expected_version_sha
            })
    {
        return Some(domain_denial(
            request,
            DomainTransitionDenialKind::ScopeMismatch,
            None,
            vec!["workflow-state-machine-domain:scope-mismatch".to_owned()],
        ));
    }

    None
}

fn domain_denial(
    request: &WorkflowStateMachineDomainRequest,
    kind: DomainTransitionDenialKind,
    kernel_reason: Option<TransitionDenialReason>,
    mut audit_refs: Vec<String>,
) -> DomainTransitionDenial {
    audit_refs.push(format!("workflow-state-machine-domain:{}", kind.as_wire()));
    DomainTransitionDenial {
        kind,
        kernel_reason,
        tenant_id: request.event.tenant_id.clone(),
        run_id: request.event.run_id.clone(),
        audit_refs: sorted_unique(audit_refs),
    }
}

fn domain_audit_refs(
    request: &WorkflowStateMachineDomainRequest,
    kernel_refs: &[String],
    outcome: &str,
) -> Vec<String> {
    let mut refs = kernel_refs.to_vec();
    refs.extend(domain_refs(request));
    refs.push(format!("workflow-state-machine-domain:{outcome}"));
    refs.push(format!(
        "workflow-state-machine-domain:origin:{}",
        request.origin.as_wire()
    ));
    sorted_unique(refs)
}

fn domain_refs(request: &WorkflowStateMachineDomainRequest) -> Vec<String> {
    vec![
        request.policy_evidence_ref.clone(),
        request.spec_integrity_ref.clone(),
        request.replay_epoch_ref.clone(),
    ]
}

fn has_unsafe_metadata(request: &WorkflowStateMachineDomainRequest) -> bool {
    !is_safe_metadata(&request.expected_tenant_id)
        || !is_safe_ref(&request.expected_spec_id)
        || !is_safe_ref(&request.expected_version_sha)
        || !is_safe_ref(&request.policy_evidence_ref)
        || !is_safe_ref(&request.spec_integrity_ref)
        || !is_safe_ref(&request.replay_epoch_ref)
}

fn is_safe_ref(value: &str) -> bool {
    is_safe_metadata(value) && value.contains(':')
}

fn is_safe_metadata(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && value == trimmed
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
        || lower.contains("private key")
        || lower.contains("-----begin")
}

fn contains_raw_content_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw model")
        || lower.contains("write an email")
        || lower.contains("customer message")
        || lower.contains("model answer")
        || lower.contains("raw output")
}

fn kernel_denial_label(reason: TransitionDenialReason) -> &'static str {
    match reason {
        TransitionDenialReason::CheckpointSequenceDrift => "checkpoint-sequence-drift",
        TransitionDenialReason::RunMismatch => "run-mismatch",
        TransitionDenialReason::SpecBindingMismatch => "spec-binding-mismatch",
        TransitionDenialReason::StepCompleteRequiresRunningStep => {
            "step-complete-requires-running-step"
        }
        TransitionDenialReason::StepRetryRequiresFailedOrRetryingStep => {
            "step-retry-requires-failed-or-retrying-step"
        }
        TransitionDenialReason::PauseRequiresRunningOrWaiting => {
            "pause-requires-running-or-waiting"
        }
        TransitionDenialReason::PolicyContextRequired => "policy-context-required",
        TransitionDenialReason::ResumeRequiresPaused => "resume-requires-paused",
        TransitionDenialReason::TenantMismatch => "tenant-mismatch",
        TransitionDenialReason::TerminalStateRefusesEvent => "terminal-state-refuses-event",
        TransitionDenialReason::WorkflowStartRequiresEmptyCheckpoint => {
            "workflow-start-requires-empty-checkpoint"
        }
    }
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

    fn start_event() -> WorkflowTransitionEvent {
        WorkflowTransitionEvent::new(
            "evt:start:domain:1",
            "ten_a",
            "run:workflow:domain:1",
            "workflow-spec:invoice-approval",
            "sha256:spec-v1",
            1,
            WorkflowEventKind::WorkflowStarted,
            "workflow-event:start:domain:1",
        )
        .expect("valid start event")
    }

    fn step_started(seq: u64, step_index: u32) -> WorkflowTransitionEvent {
        WorkflowTransitionEvent::new(
            &format!("evt:step-started:domain:{seq}"),
            "ten_a",
            "run:workflow:domain:1",
            "workflow-spec:invoice-approval",
            "sha256:spec-v1",
            seq,
            WorkflowEventKind::StepStarted { step_index },
            &format!("workflow-event:step-started:domain:{seq}"),
        )
        .expect("valid step event")
    }

    fn request(event: WorkflowTransitionEvent) -> WorkflowStateMachineDomainRequest {
        WorkflowStateMachineDomainRequest {
            current_checkpoint: None,
            event,
            expected_tenant_id: "ten_a".to_owned(),
            expected_spec_id: "workflow-spec:invoice-approval".to_owned(),
            expected_version_sha: "sha256:spec-v1".to_owned(),
            policy_evidence_ref: "cedar://workflow/state-machine/allow".to_owned(),
            spec_integrity_ref: "spec-integrity:workflow:v1".to_owned(),
            replay_epoch_ref: "replay-epoch:domain:1".to_owned(),
            origin: TransitionOrigin::WorkerReplay,
        }
    }

    #[test]
    fn domain_evaluation_delegates_to_kernel_and_attaches_policy_spec_replay_refs() {
        let decision = evaluate_domain_transition(request(start_event())).expect_applied();

        assert_eq!(decision.checkpoint.run_status, WorkflowRunStatus::Running);
        assert!(
            decision
                .audit_refs
                .contains(&"cedar://workflow/state-machine/allow".to_owned())
        );
        assert!(
            decision
                .audit_refs
                .contains(&"spec-integrity:workflow:v1".to_owned())
        );
        assert!(
            decision
                .audit_refs
                .contains(&"replay-epoch:domain:1".to_owned())
        );
        assert_eq!(decision.origin, TransitionOrigin::WorkerReplay);
    }

    #[test]
    fn invalid_tenant_or_spec_scope_denies_before_kernel() {
        let mut invalid = request(start_event());
        invalid.expected_tenant_id = "ten_other".to_owned();
        invalid.expected_version_sha = "sha256:other".to_owned();

        let denial = evaluate_domain_transition(invalid).expect_denied();

        assert_eq!(denial.kind, DomainTransitionDenialKind::ScopeMismatch);
        assert_eq!(denial.kernel_reason, None);
        assert!(
            denial
                .audit_refs
                .contains(&"workflow-state-machine-domain:scope-mismatch".to_owned())
        );
    }

    #[test]
    fn kernel_terminal_denial_is_preserved_with_domain_evidence() {
        let started = evaluate_transition(None, start_event()).expect_applied();
        let terminal = evaluate_transition(
            Some(&started),
            WorkflowTransitionEvent::new(
                "evt:completed:domain:2",
                "ten_a",
                "run:workflow:domain:1",
                "workflow-spec:invoice-approval",
                "sha256:spec-v1",
                2,
                WorkflowEventKind::WorkflowCompleted,
                "workflow-event:completed:domain:2",
            )
            .unwrap(),
        )
        .expect_applied();
        let mut invalid = request(step_started(3, 1));
        invalid.current_checkpoint = Some(terminal);

        let denial = evaluate_domain_transition(invalid).expect_denied();

        assert_eq!(denial.kind, DomainTransitionDenialKind::KernelDenied);
        assert_eq!(
            denial.kernel_reason,
            Some(TransitionDenialReason::TerminalStateRefusesEvent)
        );
        assert!(
            denial
                .audit_refs
                .contains(&"workflow-state-machine-domain:kernel-denied".to_owned())
        );
    }

    #[test]
    fn missing_policy_spec_or_replay_evidence_denies_before_kernel() {
        let mut invalid = request(start_event());
        invalid.policy_evidence_ref.clear();
        invalid.spec_integrity_ref = " ".to_owned();
        invalid.replay_epoch_ref.clear();

        let denial = evaluate_domain_transition(invalid).expect_denied();

        assert_eq!(denial.kind, DomainTransitionDenialKind::MissingEvidence);
        assert_eq!(denial.kernel_reason, None);
        assert!(
            denial
                .audit_refs
                .contains(&"validation:policy-evidence-required".to_owned())
        );
        assert!(
            denial
                .audit_refs
                .contains(&"validation:spec-integrity-required".to_owned())
        );
        assert!(
            denial
                .audit_refs
                .contains(&"validation:replay-epoch-required".to_owned())
        );
    }

    #[test]
    fn raw_prompt_output_or_secret_shaped_domain_metadata_is_rejected_without_echo() {
        let mut invalid = request(start_event());
        invalid.policy_evidence_ref = "Authorization: Bearer sk-test raw prompt".to_owned();

        let denial = evaluate_domain_transition(invalid).expect_denied();

        assert_eq!(denial.kind, DomainTransitionDenialKind::UnsafeMetadata);
        let rendered = format!("{denial:?}").to_ascii_lowercase();
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("raw prompt"));
    }

    #[test]
    fn deterministic_audit_refs_are_sorted_and_deduplicated() {
        let mut domain_request = request(start_event());
        domain_request.spec_integrity_ref = domain_request.policy_evidence_ref.clone();

        let receipt = evaluate_domain_transition(domain_request).expect_applied();
        let mut sorted = receipt.audit_refs.clone();
        sorted.sort();
        sorted.dedup();

        assert_eq!(receipt.audit_refs, sorted);
    }

    #[test]
    fn applied_step_transition_preserves_step_status_from_kernel() {
        let started = evaluate_domain_transition(request(start_event()))
            .expect_applied()
            .checkpoint;
        let mut next = request(step_started(2, 0));
        next.current_checkpoint = Some(started);

        let receipt = evaluate_domain_transition(next).expect_applied();

        assert_eq!(receipt.checkpoint.step_status, Some(StepStatus::Running));
        assert_eq!(receipt.checkpoint.current_step_index, Some(0));
    }
}
