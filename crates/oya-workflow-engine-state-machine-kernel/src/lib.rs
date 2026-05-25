//! Workflow-engine state-machine kernel foundation.
//!
//! This crate provides the pure value kernel for workflow-engine transition
//! checkpoints. It models the OpenAPI/proto run and step statuses, the AsyncAPI
//! lifecycle events, deterministic checkpoint sequencing, and replay-safe
//! transition denials. It performs no storage, network, wall-clock, random,
//! signing, queue, Postgres, Valkey, or cloud-runtime work.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowRunStatus {
    Running,
    Paused,
    Waiting,
    Completed,
    Failed,
    Cancelled,
}

impl WorkflowRunStatus {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Waiting => "waiting",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_wire(value: &str) -> Option<Self> {
        match value {
            "running" => Some(Self::Running),
            "paused" => Some(Self::Paused),
            "waiting" => Some(Self::Waiting),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Retrying,
}

impl StepStatus {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Retrying => "retrying",
        }
    }

    pub fn from_wire(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "running" => Some(Self::Running),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "retrying" => Some(Self::Retrying),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowEventKind {
    WorkflowStarted,
    StepStarted { step_index: u32 },
    StepCompleted { step_index: u32 },
    StepFailed { step_index: u32, retry_count: u32 },
    StepRetried { step_index: u32, retry_attempt: u32 },
    WorkflowPaused { policy_context_ref: String },
    WorkflowResumed,
    WorkflowCancelled { policy_context_ref: String },
    WorkflowCompleted,
    WorkflowFailed,
}

impl WorkflowEventKind {
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::WorkflowStarted => "WorkflowStarted",
            Self::StepStarted { .. } => "StepStarted",
            Self::StepCompleted { .. } => "StepCompleted",
            Self::StepFailed { .. } => "StepFailed",
            Self::StepRetried { .. } => "StepRetried",
            Self::WorkflowPaused { .. } => "WorkflowPaused",
            Self::WorkflowResumed => "WorkflowResumed",
            Self::WorkflowCancelled { .. } => "WorkflowCancelled",
            Self::WorkflowCompleted => "WorkflowCompleted",
            Self::WorkflowFailed => "WorkflowFailed",
        }
    }

    fn step_index(&self) -> Option<u32> {
        match self {
            Self::StepStarted { step_index }
            | Self::StepCompleted { step_index }
            | Self::StepFailed { step_index, .. }
            | Self::StepRetried { step_index, .. } => Some(*step_index),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowTransitionEvent {
    pub event_id: String,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,          // data_class: INTERNAL_ONLY
    pub run_id: String,             // data_class: INTERNAL_ONLY
    pub spec_id: String,            // data_class: INTERNAL_ONLY
    pub version_sha: String,        // data_class: INTERNAL_ONLY
    pub sequence_num: u64,          // data_class: INTERNAL_ONLY
    pub kind: WorkflowEventKind,    // data_class: INTERNAL_ONLY
    pub event_evidence_ref: String, // data_class: INTERNAL_ONLY
}

impl WorkflowTransitionEvent {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        event_id: &str,
        tenant_id: &str,
        run_id: &str,
        spec_id: &str,
        version_sha: &str,
        sequence_num: u64,
        kind: WorkflowEventKind,
        event_evidence_ref: &str,
    ) -> Result<Self, TransitionEventValidationError> {
        require_metadata(event_id).map_err(|_| TransitionEventValidationError::InvalidEventId)?;
        require_tenant(tenant_id).map_err(|_| TransitionEventValidationError::InvalidTenantId)?;
        require_ref(run_id).map_err(|_| TransitionEventValidationError::InvalidRunId)?;
        require_ref(spec_id).map_err(|_| TransitionEventValidationError::InvalidSpecId)?;
        require_ref(version_sha).map_err(|_| TransitionEventValidationError::InvalidVersionSha)?;
        if sequence_num == 0 {
            return Err(TransitionEventValidationError::InvalidSequenceNum);
        }
        require_ref(event_evidence_ref)
            .map_err(|_| TransitionEventValidationError::InvalidEvidenceRef)?;
        Ok(Self {
            event_id: event_id.to_owned(),
            tenant_id: tenant_id.to_owned(),
            run_id: run_id.to_owned(),
            spec_id: spec_id.to_owned(),
            version_sha: version_sha.to_owned(),
            sequence_num,
            kind,
            event_evidence_ref: event_evidence_ref.to_owned(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateCheckpoint {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub run_id: String,                  // data_class: INTERNAL_ONLY
    pub spec_id: String,                 // data_class: INTERNAL_ONLY
    pub version_sha: String,             // data_class: INTERNAL_ONLY
    pub checkpoint_seq: u64,             // data_class: INTERNAL_ONLY
    pub run_status: WorkflowRunStatus,   // data_class: PUBLIC
    pub current_step_index: Option<u32>, // data_class: INTERNAL_ONLY
    pub step_status: Option<StepStatus>, // data_class: PUBLIC
    pub last_event_id: String,           // data_class: INTERNAL_ONLY
    pub last_event_type: String,         // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransitionDenialReason {
    CheckpointSequenceDrift,
    RunMismatch,
    SpecBindingMismatch,
    StepCompleteRequiresRunningStep,
    StepRetryRequiresFailedOrRetryingStep,
    PauseRequiresRunningOrWaiting,
    PolicyContextRequired,
    ResumeRequiresPaused,
    TenantMismatch,
    TerminalStateRefusesEvent,
    WorkflowStartRequiresEmptyCheckpoint,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransitionDenial {
    pub reason: TransitionDenialReason, // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub run_id: String,                 // data_class: INTERNAL_ONLY
    pub current_state: Option<WorkflowRunStatus>, // data_class: INTERNAL_ONLY
    pub expected_checkpoint_seq: Option<u64>, // data_class: INTERNAL_ONLY
    pub observed_sequence_num: u64,     // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransitionDecision {
    Applied(StateCheckpoint),
    Denied(TransitionDenial),
}

impl TransitionDecision {
    pub fn expect_applied(self) -> StateCheckpoint {
        match self {
            Self::Applied(checkpoint) => checkpoint,
            Self::Denied(denial) => panic!("expected applied transition, got {denial:?}"),
        }
    }

    pub fn expect_denied(self) -> TransitionDenial {
        match self {
            Self::Applied(checkpoint) => panic!("expected denied transition, got {checkpoint:?}"),
            Self::Denied(denial) => denial,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransitionEventValidationError {
    InvalidEventId,
    InvalidTenantId,
    InvalidRunId,
    InvalidSpecId,
    InvalidVersionSha,
    InvalidSequenceNum,
    InvalidEvidenceRef,
}

pub fn evaluate_transition(
    current: Option<&StateCheckpoint>,
    event: WorkflowTransitionEvent,
) -> TransitionDecision {
    if let Some(current) = current {
        if current.tenant_id != event.tenant_id {
            return denied(
                &event,
                current,
                TransitionDenialReason::TenantMismatch,
                None,
            );
        }
        if current.run_id != event.run_id {
            return denied(&event, current, TransitionDenialReason::RunMismatch, None);
        }
        if current.spec_id != event.spec_id || current.version_sha != event.version_sha {
            return denied(
                &event,
                current,
                TransitionDenialReason::SpecBindingMismatch,
                None,
            );
        }
        let expected_seq = current.checkpoint_seq.saturating_add(1);
        if event.sequence_num != expected_seq {
            return denied(
                &event,
                current,
                TransitionDenialReason::CheckpointSequenceDrift,
                Some(expected_seq),
            );
        }
        if current.run_status.is_terminal() {
            return denied(
                &event,
                current,
                TransitionDenialReason::TerminalStateRefusesEvent,
                None,
            );
        }
    } else if event.sequence_num != 1 {
        return TransitionDecision::Denied(TransitionDenial {
            reason: TransitionDenialReason::CheckpointSequenceDrift,
            tenant_id: event.tenant_id,
            run_id: event.run_id,
            current_state: None,
            expected_checkpoint_seq: Some(1),
            observed_sequence_num: event.sequence_num,
            evidence_refs: vec!["workflow-state-machine:checkpoint-sequence-drift".to_owned()],
        });
    } else if !matches!(event.kind, WorkflowEventKind::WorkflowStarted) {
        return TransitionDecision::Denied(TransitionDenial {
            reason: TransitionDenialReason::WorkflowStartRequiresEmptyCheckpoint,
            tenant_id: event.tenant_id,
            run_id: event.run_id,
            current_state: None,
            expected_checkpoint_seq: Some(1),
            observed_sequence_num: event.sequence_num,
            evidence_refs: vec!["workflow-state-machine:start-required".to_owned()],
        });
    }

    match next_checkpoint(current, &event) {
        Ok(checkpoint) => TransitionDecision::Applied(checkpoint),
        Err(reason) => match current {
            Some(current) => denied(&event, current, reason, None),
            None => TransitionDecision::Denied(TransitionDenial {
                reason,
                tenant_id: event.tenant_id,
                run_id: event.run_id,
                current_state: None,
                expected_checkpoint_seq: Some(1),
                observed_sequence_num: event.sequence_num,
                evidence_refs: vec![format!(
                    "workflow-state-machine:{}",
                    denial_wire_label(reason)
                )],
            }),
        },
    }
}

fn next_checkpoint(
    current: Option<&StateCheckpoint>,
    event: &WorkflowTransitionEvent,
) -> Result<StateCheckpoint, TransitionDenialReason> {
    let (run_status, step_status, current_step_index) = match &event.kind {
        WorkflowEventKind::WorkflowStarted => {
            if current.is_some() {
                return Err(TransitionDenialReason::WorkflowStartRequiresEmptyCheckpoint);
            }
            (WorkflowRunStatus::Running, None, None)
        }
        WorkflowEventKind::StepStarted { step_index } => (
            WorkflowRunStatus::Running,
            Some(StepStatus::Running),
            Some(*step_index),
        ),
        WorkflowEventKind::StepCompleted { step_index } => {
            let Some(current) = current else {
                return Err(TransitionDenialReason::WorkflowStartRequiresEmptyCheckpoint);
            };
            if current.step_status != Some(StepStatus::Running)
                || current.current_step_index != Some(*step_index)
            {
                return Err(TransitionDenialReason::StepCompleteRequiresRunningStep);
            }
            (
                WorkflowRunStatus::Running,
                Some(StepStatus::Completed),
                Some(*step_index),
            )
        }
        WorkflowEventKind::StepFailed { step_index, .. } => (
            WorkflowRunStatus::Waiting,
            Some(StepStatus::Failed),
            Some(*step_index),
        ),
        WorkflowEventKind::StepRetried { step_index, .. } => {
            let Some(current) = current else {
                return Err(TransitionDenialReason::WorkflowStartRequiresEmptyCheckpoint);
            };
            if current.current_step_index != Some(*step_index)
                || !matches!(
                    current.step_status,
                    Some(StepStatus::Failed | StepStatus::Retrying)
                )
            {
                return Err(TransitionDenialReason::StepRetryRequiresFailedOrRetryingStep);
            }
            (
                WorkflowRunStatus::Running,
                Some(StepStatus::Retrying),
                Some(*step_index),
            )
        }
        WorkflowEventKind::WorkflowPaused { policy_context_ref } => {
            if !is_safe_ref(policy_context_ref) {
                return Err(TransitionDenialReason::PolicyContextRequired);
            }
            let Some(current) = current else {
                return Err(TransitionDenialReason::WorkflowStartRequiresEmptyCheckpoint);
            };
            if !matches!(
                current.run_status,
                WorkflowRunStatus::Running | WorkflowRunStatus::Waiting
            ) {
                return Err(TransitionDenialReason::PauseRequiresRunningOrWaiting);
            }
            (
                WorkflowRunStatus::Paused,
                current.step_status,
                current.current_step_index,
            )
        }
        WorkflowEventKind::WorkflowResumed => {
            let Some(current) = current else {
                return Err(TransitionDenialReason::WorkflowStartRequiresEmptyCheckpoint);
            };
            if current.run_status != WorkflowRunStatus::Paused {
                return Err(TransitionDenialReason::ResumeRequiresPaused);
            }
            (
                WorkflowRunStatus::Running,
                current.step_status,
                current.current_step_index,
            )
        }
        WorkflowEventKind::WorkflowCancelled { policy_context_ref } => {
            if !is_safe_ref(policy_context_ref) {
                return Err(TransitionDenialReason::PolicyContextRequired);
            }
            let current_step = current.and_then(|checkpoint| checkpoint.current_step_index);
            let current_step_status = current.and_then(|checkpoint| checkpoint.step_status);
            (
                WorkflowRunStatus::Cancelled,
                current_step_status,
                current_step,
            )
        }
        WorkflowEventKind::WorkflowCompleted => {
            let current_step = current.and_then(|checkpoint| checkpoint.current_step_index);
            (
                WorkflowRunStatus::Completed,
                current.and_then(|checkpoint| checkpoint.step_status),
                current_step,
            )
        }
        WorkflowEventKind::WorkflowFailed => {
            let current_step = current.and_then(|checkpoint| checkpoint.current_step_index);
            (
                WorkflowRunStatus::Failed,
                current.and_then(|checkpoint| checkpoint.step_status),
                current_step,
            )
        }
    };

    Ok(StateCheckpoint {
        tenant_id: event.tenant_id.clone(),
        run_id: event.run_id.clone(),
        spec_id: event.spec_id.clone(),
        version_sha: event.version_sha.clone(),
        checkpoint_seq: event.sequence_num,
        run_status,
        current_step_index: event.kind.step_index().or(current_step_index),
        step_status,
        last_event_id: event.event_id.clone(),
        last_event_type: event.kind.event_type().to_owned(),
        evidence_refs: sorted_unique(vec![
            event.event_evidence_ref.clone(),
            format!("workflow-state-machine:{}", run_status.as_wire()),
        ]),
    })
}

fn denied(
    event: &WorkflowTransitionEvent,
    current: &StateCheckpoint,
    reason: TransitionDenialReason,
    expected_checkpoint_seq: Option<u64>,
) -> TransitionDecision {
    TransitionDecision::Denied(TransitionDenial {
        reason,
        tenant_id: event.tenant_id.clone(),
        run_id: event.run_id.clone(),
        current_state: Some(current.run_status),
        expected_checkpoint_seq,
        observed_sequence_num: event.sequence_num,
        evidence_refs: sorted_unique(vec![
            current
                .evidence_refs
                .last()
                .cloned()
                .unwrap_or_else(|| "workflow-state-machine:current-checkpoint".to_owned()),
            event.event_evidence_ref.clone(),
            format!("workflow-state-machine:{}", denial_wire_label(reason)),
        ]),
    })
}

fn denial_wire_label(reason: TransitionDenialReason) -> &'static str {
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

fn require_metadata(value: &str) -> Result<(), ()> {
    if is_safe_metadata(value) {
        Ok(())
    } else {
        Err(())
    }
}

fn require_tenant(value: &str) -> Result<(), ()> {
    let trimmed = value.trim();
    if trimmed.starts_with("ten_")
        && value == trimmed
        && !trimmed.contains('/')
        && !contains_whitespace(trimmed)
        && !contains_raw_secret_material(trimmed)
        && !contains_raw_content_material(trimmed)
    {
        Ok(())
    } else {
        Err(())
    }
}

fn require_ref(value: &str) -> Result<(), ()> {
    if is_safe_ref(value) { Ok(()) } else { Err(()) }
}

fn is_safe_metadata(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && value == trimmed
        && !contains_whitespace(trimmed)
        && !contains_raw_secret_material(trimmed)
        && !contains_raw_content_material(trimmed)
}

fn is_safe_ref(value: &str) -> bool {
    is_safe_metadata(value) && value.contains(':')
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
            "evt:start:1",
            "ten_a",
            "run:workflow:1",
            "workflow-spec:invoice-approval",
            "sha256:spec-v1",
            1,
            WorkflowEventKind::WorkflowStarted,
            "workflow-event:start:1",
        )
        .expect("valid start event")
    }

    fn step_started(seq: u64, step_index: u32) -> WorkflowTransitionEvent {
        WorkflowTransitionEvent::new(
            &format!("evt:step-started:{seq}"),
            "ten_a",
            "run:workflow:1",
            "workflow-spec:invoice-approval",
            "sha256:spec-v1",
            seq,
            WorkflowEventKind::StepStarted { step_index },
            &format!("workflow-event:step-started:{seq}"),
        )
        .expect("valid step-started event")
    }

    fn step_completed(seq: u64, step_index: u32) -> WorkflowTransitionEvent {
        WorkflowTransitionEvent::new(
            &format!("evt:step-completed:{seq}"),
            "ten_a",
            "run:workflow:1",
            "workflow-spec:invoice-approval",
            "sha256:spec-v1",
            seq,
            WorkflowEventKind::StepCompleted { step_index },
            &format!("workflow-event:step-completed:{seq}"),
        )
        .expect("valid step-completed event")
    }

    fn pause_event(seq: u64) -> WorkflowTransitionEvent {
        WorkflowTransitionEvent::new(
            &format!("evt:pause:{seq}"),
            "ten_a",
            "run:workflow:1",
            "workflow-spec:invoice-approval",
            "sha256:spec-v1",
            seq,
            WorkflowEventKind::WorkflowPaused {
                policy_context_ref: "cedar://workflow/pause/allow".to_owned(),
            },
            &format!("workflow-event:pause:{seq}"),
        )
        .expect("valid pause event")
    }

    fn resume_event(seq: u64) -> WorkflowTransitionEvent {
        WorkflowTransitionEvent::new(
            &format!("evt:resume:{seq}"),
            "ten_a",
            "run:workflow:1",
            "workflow-spec:invoice-approval",
            "sha256:spec-v1",
            seq,
            WorkflowEventKind::WorkflowResumed,
            &format!("workflow-event:resume:{seq}"),
        )
        .expect("valid resume event")
    }

    #[test]
    fn transition_eval_replays_identically() {
        let start = evaluate_transition(None, start_event()).expect_applied();
        let event = step_started(2, 0);

        let first = evaluate_transition(Some(&start), event.clone());
        let replay = evaluate_transition(Some(&start), event);

        assert_eq!(first, replay);
        let checkpoint = first.expect_applied();
        assert_eq!(checkpoint.run_status, WorkflowRunStatus::Running);
        assert_eq!(checkpoint.step_status, Some(StepStatus::Running));
        assert_eq!(checkpoint.current_step_index, Some(0));

        let completed =
            evaluate_transition(Some(&checkpoint), step_completed(3, 0)).expect_applied();
        assert_eq!(completed.step_status, Some(StepStatus::Completed));
    }

    #[test]
    fn terminal_state_refuses_late_event() {
        let started = evaluate_transition(None, start_event()).expect_applied();
        let terminal = evaluate_transition(
            Some(&started),
            WorkflowTransitionEvent::new(
                "evt:completed:2",
                "ten_a",
                "run:workflow:1",
                "workflow-spec:invoice-approval",
                "sha256:spec-v1",
                2,
                WorkflowEventKind::WorkflowCompleted,
                "workflow-event:completed:2",
            )
            .unwrap(),
        )
        .expect_applied();

        let denied = evaluate_transition(Some(&terminal), step_started(3, 1)).expect_denied();

        assert_eq!(
            denied.reason,
            TransitionDenialReason::TerminalStateRefusesEvent
        );
        assert_eq!(denied.current_state, Some(WorkflowRunStatus::Completed));
    }

    #[test]
    fn tenant_mismatch_refused_before_state_change() {
        let started = evaluate_transition(None, start_event()).expect_applied();
        let mut wrong_tenant = step_started(2, 0);
        wrong_tenant.tenant_id = "ten_other".to_owned();

        let denied = evaluate_transition(Some(&started), wrong_tenant).expect_denied();

        assert_eq!(denied.reason, TransitionDenialReason::TenantMismatch);
    }

    #[test]
    fn checkpoint_append_conflict_detected() {
        let started = evaluate_transition(None, start_event()).expect_applied();

        let denied = evaluate_transition(Some(&started), step_started(3, 0)).expect_denied();

        assert_eq!(
            denied.reason,
            TransitionDenialReason::CheckpointSequenceDrift
        );
        assert_eq!(denied.expected_checkpoint_seq, Some(2));
        assert_eq!(denied.observed_sequence_num, 3);
    }

    #[test]
    fn pause_resume_signal_sequence_preserved() {
        let started = evaluate_transition(None, start_event()).expect_applied();
        let paused = evaluate_transition(Some(&started), pause_event(2)).expect_applied();
        let resumed = evaluate_transition(Some(&paused), resume_event(3)).expect_applied();

        assert_eq!(paused.run_status, WorkflowRunStatus::Paused);
        assert_eq!(resumed.run_status, WorkflowRunStatus::Running);
        assert_eq!(resumed.checkpoint_seq, 3);
    }

    #[test]
    fn cancel_requires_policy_context_ref() {
        let started = evaluate_transition(None, start_event()).expect_applied();
        let cancel = WorkflowTransitionEvent::new(
            "evt:cancel:2",
            "ten_a",
            "run:workflow:1",
            "workflow-spec:invoice-approval",
            "sha256:spec-v1",
            2,
            WorkflowEventKind::WorkflowCancelled {
                policy_context_ref: "".to_owned(),
            },
            "workflow-event:cancel:2",
        )
        .expect("event may be constructed then denied by transition invariant");

        let denied = evaluate_transition(Some(&started), cancel).expect_denied();

        assert_eq!(denied.reason, TransitionDenialReason::PolicyContextRequired);
    }

    #[test]
    fn contract_wire_statuses_are_canonical_and_unknowns_denied() {
        assert_eq!(
            WorkflowRunStatus::from_wire("running"),
            Some(WorkflowRunStatus::Running)
        );
        assert_eq!(WorkflowRunStatus::Completed.as_wire(), "completed");
        assert_eq!(
            StepStatus::from_wire("retrying"),
            Some(StepStatus::Retrying)
        );
        assert_eq!(StepStatus::Failed.as_wire(), "failed");
        assert_eq!(WorkflowRunStatus::from_wire("archived"), None);
        assert_eq!(StepStatus::from_wire("blocked"), None);
    }

    #[test]
    fn raw_prompt_output_or_secret_shaped_metadata_is_rejected() {
        let event = WorkflowTransitionEvent::new(
            "evt:raw:1",
            "ten_a",
            "run:workflow:1",
            "workflow-spec:write an email to a customer",
            "sha256:spec-v1",
            1,
            WorkflowEventKind::WorkflowStarted,
            "Authorization: Bearer sk-test",
        );

        assert_eq!(
            event.unwrap_err(),
            TransitionEventValidationError::InvalidSpecId
        );
    }
}
