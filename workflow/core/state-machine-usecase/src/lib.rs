//! Workflow-engine state-machine usecase foundation.
//!
//! The usecase composes metadata validation, checkpoint-store load/append ports,
//! and the policy-bound state-machine domain. It is source-level only: no
//! concrete storage, network, wall-clock, random, signing, queue, Postgres,
//! Valkey, or cloud-runtime work is performed here.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use workflow_state_machine_domain::{
    DomainTransitionDecision, DomainTransitionDenial, DomainTransitionDenialKind,
    DomainTransitionReceipt, TransitionOrigin, WorkflowStateMachineDomainRequest,
    evaluate_domain_transition,
};
pub use workflow_state_machine_kernel::{
    StateCheckpoint, StepStatus, TransitionDenialReason, WorkflowEventKind, WorkflowRunStatus,
    WorkflowTransitionEvent,
};

pub trait StateCheckpointStorePort {
    fn load_current(
        &mut self,
        tenant_id: &str,
        run_id: &str,
    ) -> Result<Option<StateCheckpoint>, StateCheckpointStoreFailure>;

    fn append_checkpoint(
        &mut self,
        expected_checkpoint_seq: u64,
        checkpoint: StateCheckpoint,
    ) -> Result<(), StateCheckpointAppendFailure>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StateCheckpointStoreFailure {
    Unavailable { evidence_ref: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StateCheckpointAppendFailure {
    Conflict {
        expected_checkpoint_seq: u64,
        observed_checkpoint_seq: u64,
        evidence_ref: String,
    },
    Unavailable {
        evidence_ref: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateMachineTransitionUsecaseInput {
    pub request_id: String,             // data_class: INTERNAL_ONLY
    pub idempotency_key: String,        // data_class: INTERNAL_ONLY
    pub trace_ref: String,              // data_class: INTERNAL_ONLY
    pub event: WorkflowTransitionEvent, // data_class: INTERNAL_ONLY
    pub expected_tenant_id: String,     // data_class: INTERNAL_ONLY
    pub expected_spec_id: String,       // data_class: INTERNAL_ONLY
    pub expected_version_sha: String,   // data_class: INTERNAL_ONLY
    pub policy_evidence_ref: String,    // data_class: INTERNAL_ONLY
    pub spec_integrity_ref: String,     // data_class: INTERNAL_ONLY
    pub replay_epoch_ref: String,       // data_class: INTERNAL_ONLY
    pub origin: TransitionOrigin,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum StateMachineUsecaseStatus {
    Applied,
    DomainDenied,
    InvalidInput,
    StoreConflict,
    StoreUnavailable,
}

impl StateMachineUsecaseStatus {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::DomainDenied => "domain-denied",
            Self::InvalidInput => "invalid-input",
            Self::StoreConflict => "store-conflict",
            Self::StoreUnavailable => "store-unavailable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum StateMachineAuditEventKind {
    TransitionRequested,
    TransitionInvalid,
    TransitionDenied,
    CheckpointAppended,
    StoreAppendConflict,
    StoreUnavailable,
}

impl StateMachineAuditEventKind {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::TransitionRequested => "transition-requested",
            Self::TransitionInvalid => "transition-invalid",
            Self::TransitionDenied => "transition-denied",
            Self::CheckpointAppended => "checkpoint-appended",
            Self::StoreAppendConflict => "store-append-conflict",
            Self::StoreUnavailable => "store-unavailable",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateMachineAuditEvent {
    pub kind: StateMachineAuditEventKind, // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub run_id: String,                   // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateMachineUsecaseReceipt {
    pub status: StateMachineUsecaseStatus, // data_class: INTERNAL_ONLY
    pub checkpoint: Option<StateCheckpoint>, // data_class: INTERNAL_ONLY
    pub expected_checkpoint_seq: Option<u64>, // data_class: INTERNAL_ONLY
    pub observed_checkpoint_seq: Option<u64>, // data_class: INTERNAL_ONLY
    pub domain_denial_kind: Option<DomainTransitionDenialKind>, // data_class: INTERNAL_ONLY
    pub kernel_denial_reason: Option<TransitionDenialReason>, // data_class: INTERNAL_ONLY
    pub audit_events: Vec<StateMachineAuditEvent>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,        // data_class: INTERNAL_ONLY
}

pub fn apply_state_machine_transition<S: StateCheckpointStorePort>(
    store: &mut S,
    input: StateMachineTransitionUsecaseInput,
) -> StateMachineUsecaseReceipt {
    if let Some(receipt) = invalid_input_receipt(&input) {
        return receipt;
    }

    let requested = audit_event(
        StateMachineAuditEventKind::TransitionRequested,
        &input.event.tenant_id,
        &input.event.run_id,
        sorted_unique(vec![
            input.request_id.clone(),
            input.idempotency_key.clone(),
            input.trace_ref.clone(),
            input.event.event_evidence_ref.clone(),
        ]),
    );

    let current = match store.load_current(&input.event.tenant_id, &input.event.run_id) {
        Ok(current) => current,
        Err(failure) => {
            let refs = store_failure_refs(failure);
            return receipt(
                StateMachineUsecaseStatus::StoreUnavailable,
                None,
                None,
                None,
                None,
                None,
                vec![
                    requested,
                    audit_event(
                        StateMachineAuditEventKind::StoreUnavailable,
                        &input.event.tenant_id,
                        &input.event.run_id,
                        refs.clone(),
                    ),
                ],
                refs,
            );
        }
    };

    let expected_checkpoint_seq = current
        .as_ref()
        .map_or(1, |checkpoint| checkpoint.checkpoint_seq.saturating_add(1));
    let domain_request = WorkflowStateMachineDomainRequest {
        current_checkpoint: current,
        event: input.event.clone(),
        expected_tenant_id: input.expected_tenant_id,
        expected_spec_id: input.expected_spec_id,
        expected_version_sha: input.expected_version_sha,
        policy_evidence_ref: input.policy_evidence_ref,
        spec_integrity_ref: input.spec_integrity_ref,
        replay_epoch_ref: input.replay_epoch_ref,
        origin: input.origin,
    };

    match evaluate_domain_transition(domain_request) {
        DomainTransitionDecision::Denied(denial) => domain_denied_receipt(requested, denial),
        DomainTransitionDecision::Applied(receipt_value) => {
            append_checkpoint_receipt(store, requested, expected_checkpoint_seq, receipt_value)
        }
    }
}

fn invalid_input_receipt(
    input: &StateMachineTransitionUsecaseInput,
) -> Option<StateMachineUsecaseReceipt> {
    let mut refs = Vec::new();
    if !is_safe_ref(&input.request_id) {
        refs.push("validation:request-id-invalid".to_owned());
    }
    if !is_safe_ref(&input.idempotency_key) {
        refs.push("validation:idempotency-key-invalid".to_owned());
    }
    if !is_safe_ref(&input.trace_ref) {
        refs.push("validation:trace-ref-invalid".to_owned());
    }
    if refs.is_empty() {
        return None;
    }
    refs.push("workflow-state-machine-usecase:invalid-input".to_owned());
    let refs = sorted_unique(refs);
    Some(receipt(
        StateMachineUsecaseStatus::InvalidInput,
        None,
        None,
        None,
        None,
        None,
        vec![audit_event(
            StateMachineAuditEventKind::TransitionInvalid,
            &input.event.tenant_id,
            &input.event.run_id,
            refs.clone(),
        )],
        refs,
    ))
}

fn domain_denied_receipt(
    requested: StateMachineAuditEvent,
    denial: DomainTransitionDenial,
) -> StateMachineUsecaseReceipt {
    let mut refs = denial.audit_refs.clone();
    refs.push("workflow-state-machine-usecase:domain-denied".to_owned());
    let refs = sorted_unique(refs);
    receipt(
        StateMachineUsecaseStatus::DomainDenied,
        None,
        None,
        None,
        Some(denial.kind),
        denial.kernel_reason,
        vec![
            requested,
            audit_event(
                StateMachineAuditEventKind::TransitionDenied,
                &denial.tenant_id,
                &denial.run_id,
                refs.clone(),
            ),
        ],
        refs,
    )
}

fn append_checkpoint_receipt<S: StateCheckpointStorePort>(
    store: &mut S,
    requested: StateMachineAuditEvent,
    expected_checkpoint_seq: u64,
    receipt_value: DomainTransitionReceipt,
) -> StateMachineUsecaseReceipt {
    let checkpoint = receipt_value.checkpoint;
    match store.append_checkpoint(expected_checkpoint_seq, checkpoint.clone()) {
        Ok(()) => {
            let mut refs = receipt_value.audit_refs;
            refs.push("workflow-state-machine-usecase:checkpoint-appended".to_owned());
            let refs = sorted_unique(refs);
            receipt(
                StateMachineUsecaseStatus::Applied,
                Some(checkpoint.clone()),
                Some(expected_checkpoint_seq),
                None,
                None,
                None,
                vec![
                    requested,
                    audit_event(
                        StateMachineAuditEventKind::CheckpointAppended,
                        &checkpoint.tenant_id,
                        &checkpoint.run_id,
                        refs.clone(),
                    ),
                ],
                refs,
            )
        }
        Err(StateCheckpointAppendFailure::Conflict {
            expected_checkpoint_seq,
            observed_checkpoint_seq,
            evidence_ref,
        }) => {
            let refs = sorted_unique(vec![
                evidence_ref,
                "workflow-state-machine-usecase:store-conflict".to_owned(),
            ]);
            receipt(
                StateMachineUsecaseStatus::StoreConflict,
                None,
                Some(expected_checkpoint_seq),
                Some(observed_checkpoint_seq),
                None,
                None,
                vec![
                    requested,
                    audit_event(
                        StateMachineAuditEventKind::StoreAppendConflict,
                        &checkpoint.tenant_id,
                        &checkpoint.run_id,
                        refs.clone(),
                    ),
                ],
                refs,
            )
        }
        Err(StateCheckpointAppendFailure::Unavailable { evidence_ref }) => {
            let refs = sorted_unique(vec![
                evidence_ref,
                "workflow-state-machine-usecase:store-unavailable".to_owned(),
            ]);
            receipt(
                StateMachineUsecaseStatus::StoreUnavailable,
                None,
                Some(expected_checkpoint_seq),
                None,
                None,
                None,
                vec![
                    requested,
                    audit_event(
                        StateMachineAuditEventKind::StoreUnavailable,
                        &checkpoint.tenant_id,
                        &checkpoint.run_id,
                        refs.clone(),
                    ),
                ],
                refs,
            )
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn receipt(
    status: StateMachineUsecaseStatus,
    checkpoint: Option<StateCheckpoint>,
    expected_checkpoint_seq: Option<u64>,
    observed_checkpoint_seq: Option<u64>,
    domain_denial_kind: Option<DomainTransitionDenialKind>,
    kernel_denial_reason: Option<TransitionDenialReason>,
    audit_events: Vec<StateMachineAuditEvent>,
    evidence_refs: Vec<String>,
) -> StateMachineUsecaseReceipt {
    StateMachineUsecaseReceipt {
        status,
        checkpoint,
        expected_checkpoint_seq,
        observed_checkpoint_seq,
        domain_denial_kind,
        kernel_denial_reason,
        audit_events,
        evidence_refs: sorted_unique(evidence_refs),
    }
}

fn store_failure_refs(failure: StateCheckpointStoreFailure) -> Vec<String> {
    match failure {
        StateCheckpointStoreFailure::Unavailable { evidence_ref } => sorted_unique(vec![
            evidence_ref,
            "workflow-state-machine-usecase:store-unavailable".to_owned(),
        ]),
    }
}

fn audit_event(
    kind: StateMachineAuditEventKind,
    tenant_id: &str,
    run_id: &str,
    evidence_refs: Vec<String>,
) -> StateMachineAuditEvent {
    StateMachineAuditEvent {
        kind,
        tenant_id: tenant_id.to_owned(),
        run_id: run_id.to_owned(),
        evidence_refs: sorted_unique(evidence_refs),
    }
}

fn is_safe_ref(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && value == trimmed
        && value.contains(':')
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

    #[derive(Default)]
    struct FakeStore {
        current: Option<StateCheckpoint>,
        appended: Vec<(u64, StateCheckpoint)>,
        load_calls: usize,
        append_conflict: bool,
        load_failure: bool,
    }

    impl StateCheckpointStorePort for FakeStore {
        fn load_current(
            &mut self,
            tenant_id: &str,
            run_id: &str,
        ) -> Result<Option<StateCheckpoint>, StateCheckpointStoreFailure> {
            self.load_calls += 1;
            if self.load_failure {
                return Err(StateCheckpointStoreFailure::Unavailable {
                    evidence_ref: "store:error:redacted".to_owned(),
                });
            }
            assert_eq!(tenant_id, "ten_a");
            assert!(run_id.starts_with("run:workflow:usecase"));
            Ok(self.current.clone())
        }

        fn append_checkpoint(
            &mut self,
            expected_checkpoint_seq: u64,
            checkpoint: StateCheckpoint,
        ) -> Result<(), StateCheckpointAppendFailure> {
            if self.append_conflict {
                return Err(StateCheckpointAppendFailure::Conflict {
                    expected_checkpoint_seq,
                    observed_checkpoint_seq: checkpoint.checkpoint_seq.saturating_sub(1),
                    evidence_ref: "store:append-conflict".to_owned(),
                });
            }
            self.appended.push((expected_checkpoint_seq, checkpoint));
            Ok(())
        }
    }

    fn event(sequence_num: u64, kind: WorkflowEventKind) -> WorkflowTransitionEvent {
        WorkflowTransitionEvent::new(
            &format!("evt:usecase:{sequence_num}"),
            "ten_a",
            "run:workflow:usecase:1",
            "workflow-spec:invoice-approval",
            "sha256:spec-v1",
            sequence_num,
            kind,
            &format!("workflow-event:usecase:{sequence_num}"),
        )
        .expect("valid event")
    }

    fn input(sequence_num: u64, kind: WorkflowEventKind) -> StateMachineTransitionUsecaseInput {
        StateMachineTransitionUsecaseInput {
            request_id: format!("req:state-machine:{sequence_num}"),
            idempotency_key: format!("idem:state-machine:{sequence_num}"),
            trace_ref: format!("trace:state-machine:{sequence_num}"),
            event: event(sequence_num, kind),
            expected_tenant_id: "ten_a".to_owned(),
            expected_spec_id: "workflow-spec:invoice-approval".to_owned(),
            expected_version_sha: "sha256:spec-v1".to_owned(),
            policy_evidence_ref: "cedar://workflow/state-machine/allow".to_owned(),
            spec_integrity_ref: "spec-integrity:workflow:v1".to_owned(),
            replay_epoch_ref: "replay-epoch:usecase:1".to_owned(),
            origin: TransitionOrigin::TriggerOrchestrator,
        }
    }

    fn started_checkpoint() -> StateCheckpoint {
        let mut store = FakeStore::default();
        let receipt = apply_state_machine_transition(
            &mut store,
            input(1, WorkflowEventKind::WorkflowStarted),
        );
        receipt.checkpoint.expect("checkpoint")
    }

    #[test]
    fn initial_start_appends_expected_sequence_one() {
        let mut store = FakeStore::default();

        let receipt = apply_state_machine_transition(
            &mut store,
            input(1, WorkflowEventKind::WorkflowStarted),
        );

        assert_eq!(receipt.status, StateMachineUsecaseStatus::Applied);
        assert_eq!(store.appended.len(), 1);
        assert_eq!(store.appended[0].0, 1);
        assert_eq!(
            receipt.checkpoint.unwrap().run_status,
            WorkflowRunStatus::Running
        );
    }

    #[test]
    fn valid_transition_loads_current_delegates_domain_and_appends_expected_sequence() {
        let mut store = FakeStore {
            current: Some(started_checkpoint()),
            ..FakeStore::default()
        };

        let receipt = apply_state_machine_transition(
            &mut store,
            input(2, WorkflowEventKind::StepStarted { step_index: 0 }),
        );

        assert_eq!(receipt.status, StateMachineUsecaseStatus::Applied);
        assert_eq!(store.load_calls, 1);
        assert_eq!(store.appended[0].0, 2);
        assert_eq!(store.appended[0].1.step_status, Some(StepStatus::Running));
        assert!(
            receipt
                .audit_events
                .iter()
                .any(|event| event.kind == StateMachineAuditEventKind::CheckpointAppended)
        );
    }

    #[test]
    fn invalid_metadata_denies_before_store_side_effects() {
        let mut store = FakeStore::default();
        let mut invalid = input(1, WorkflowEventKind::WorkflowStarted);
        invalid.idempotency_key.clear();
        invalid.trace_ref = "trace raw output".to_owned();

        let receipt = apply_state_machine_transition(&mut store, invalid);

        assert_eq!(receipt.status, StateMachineUsecaseStatus::InvalidInput);
        assert_eq!(store.load_calls, 0);
        assert!(store.appended.is_empty());
    }

    #[test]
    fn domain_denial_does_not_append_checkpoint() {
        let mut store = FakeStore {
            current: Some(started_checkpoint()),
            ..FakeStore::default()
        };
        let mut invalid = input(2, WorkflowEventKind::WorkflowResumed);
        invalid.expected_version_sha = "sha256:other".to_owned();

        let receipt = apply_state_machine_transition(&mut store, invalid);

        assert_eq!(receipt.status, StateMachineUsecaseStatus::DomainDenied);
        assert!(receipt.checkpoint.is_none());
        assert!(store.appended.is_empty());
    }

    #[test]
    fn append_conflict_detected_and_sanitized() {
        let mut store = FakeStore {
            current: Some(started_checkpoint()),
            append_conflict: true,
            ..FakeStore::default()
        };

        let receipt = apply_state_machine_transition(
            &mut store,
            input(2, WorkflowEventKind::StepStarted { step_index: 0 }),
        );

        assert_eq!(receipt.status, StateMachineUsecaseStatus::StoreConflict);
        assert_eq!(receipt.expected_checkpoint_seq, Some(2));
        assert!(format!("{receipt:?}").contains("store:append-conflict"));
    }

    #[test]
    fn store_load_failure_is_sanitized_without_append() {
        let mut store = FakeStore {
            load_failure: true,
            ..FakeStore::default()
        };

        let receipt = apply_state_machine_transition(
            &mut store,
            input(1, WorkflowEventKind::WorkflowStarted),
        );

        assert_eq!(receipt.status, StateMachineUsecaseStatus::StoreUnavailable);
        assert!(store.appended.is_empty());
        assert!(
            !format!("{receipt:?}")
                .to_ascii_lowercase()
                .contains("database password")
        );
    }

    #[test]
    fn raw_secret_trace_metadata_denies_without_echo() {
        let mut store = FakeStore::default();
        let mut invalid = input(1, WorkflowEventKind::WorkflowStarted);
        invalid.trace_ref = "Authorization: Bearer sk-test".to_owned();

        let receipt = apply_state_machine_transition(&mut store, invalid);

        assert_eq!(receipt.status, StateMachineUsecaseStatus::InvalidInput);
        let rendered = format!("{receipt:?}").to_ascii_lowercase();
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("authorization"));
    }
}
