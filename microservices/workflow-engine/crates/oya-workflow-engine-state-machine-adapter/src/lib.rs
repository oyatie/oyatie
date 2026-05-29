//! Workflow-engine state-machine generic adapter foundation.
//!
//! This crate provides a source-level in-memory checkpoint-store adapter for
//! preview integration with the state-machine usecase. It is intentionally
//! non-durable and performs no database, filesystem, network, queue, signing,
//! wall-clock, or cloud-runtime work.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

pub use oya_workflow_engine_state_machine_kernel::{
    StateCheckpoint, StepStatus, WorkflowEventKind, WorkflowRunStatus, WorkflowTransitionEvent,
};
pub use oya_workflow_engine_state_machine_usecase::{
    StateCheckpointAppendFailure, StateCheckpointStoreFailure, StateCheckpointStorePort,
    StateMachineTransitionUsecaseInput, StateMachineUsecaseStatus, TransitionOrigin,
    apply_state_machine_transition,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum StateMachineAdapterMode {
    InMemoryPreview,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum StateMachineAdapterActionKind {
    AppendAccepted,
    AppendConflict,
    LoadCurrent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateMachineAdapterAction {
    pub kind: StateMachineAdapterActionKind, // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub run_id: String,                      // data_class: INTERNAL_ONLY
    pub expected_checkpoint_seq: Option<u64>, // data_class: INTERNAL_ONLY
    pub observed_checkpoint_seq: Option<u64>, // data_class: INTERNAL_ONLY
    pub evidence_ref: String,                // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CheckpointKey {
    tenant_id: String,
    run_id: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkflowStateMachineMemoryAdapter {
    checkpoints_by_key: BTreeMap<CheckpointKey, StateCheckpoint>,
    recorded_actions: Vec<StateMachineAdapterAction>,
}

pub type InMemoryStateCheckpointAdapter = WorkflowStateMachineMemoryAdapter;

impl WorkflowStateMachineMemoryAdapter {
    pub fn adapter_mode(&self) -> StateMachineAdapterMode {
        StateMachineAdapterMode::InMemoryPreview
    }

    pub fn checkpoint_count(&self) -> usize {
        self.checkpoints_by_key.len()
    }

    pub fn recorded_actions(&self) -> &[StateMachineAdapterAction] {
        &self.recorded_actions
    }

    fn push_action(
        &mut self,
        kind: StateMachineAdapterActionKind,
        tenant_id: &str,
        run_id: &str,
        expected_checkpoint_seq: Option<u64>,
        observed_checkpoint_seq: Option<u64>,
        evidence_ref: &str,
    ) {
        self.recorded_actions.push(StateMachineAdapterAction {
            kind,
            tenant_id: tenant_id.to_owned(),
            run_id: run_id.to_owned(),
            expected_checkpoint_seq,
            observed_checkpoint_seq,
            evidence_ref: evidence_ref.to_owned(),
        });
    }
}

impl StateCheckpointStorePort for WorkflowStateMachineMemoryAdapter {
    fn load_current(
        &mut self,
        tenant_id: &str,
        run_id: &str,
    ) -> Result<Option<StateCheckpoint>, StateCheckpointStoreFailure> {
        if !is_safe_tenant(tenant_id) || !is_safe_ref(run_id) {
            return Err(StateCheckpointStoreFailure::Unavailable {
                evidence_ref: "workflow-state-machine-adapter:unsafe-load-metadata".to_owned(),
            });
        }
        let key = CheckpointKey {
            tenant_id: tenant_id.to_owned(),
            run_id: run_id.to_owned(),
        };
        let current = self.checkpoints_by_key.get(&key).cloned();
        self.push_action(
            StateMachineAdapterActionKind::LoadCurrent,
            tenant_id,
            run_id,
            None,
            current.as_ref().map(|checkpoint| checkpoint.checkpoint_seq),
            "workflow-state-machine-adapter:load-current",
        );
        Ok(current)
    }

    fn append_checkpoint(
        &mut self,
        expected_checkpoint_seq: u64,
        checkpoint: StateCheckpoint,
    ) -> Result<(), StateCheckpointAppendFailure> {
        if !is_safe_checkpoint(&checkpoint) {
            return Err(StateCheckpointAppendFailure::Unavailable {
                evidence_ref: "workflow-state-machine-adapter:unsafe-checkpoint-metadata"
                    .to_owned(),
            });
        }

        let key = CheckpointKey {
            tenant_id: checkpoint.tenant_id.clone(),
            run_id: checkpoint.run_id.clone(),
        };
        let observed_checkpoint_seq = self
            .checkpoints_by_key
            .get(&key)
            .map_or(0, |current| current.checkpoint_seq);
        let required_checkpoint_seq = observed_checkpoint_seq.saturating_add(1);
        if expected_checkpoint_seq != required_checkpoint_seq
            || checkpoint.checkpoint_seq != expected_checkpoint_seq
        {
            self.push_action(
                StateMachineAdapterActionKind::AppendConflict,
                &checkpoint.tenant_id,
                &checkpoint.run_id,
                Some(expected_checkpoint_seq),
                Some(observed_checkpoint_seq),
                "workflow-state-machine-adapter:checkpoint-sequence-conflict",
            );
            return Err(StateCheckpointAppendFailure::Conflict {
                expected_checkpoint_seq,
                observed_checkpoint_seq,
                evidence_ref: "workflow-state-machine-adapter:checkpoint-sequence-conflict"
                    .to_owned(),
            });
        }

        self.push_action(
            StateMachineAdapterActionKind::AppendAccepted,
            &checkpoint.tenant_id,
            &checkpoint.run_id,
            Some(expected_checkpoint_seq),
            Some(observed_checkpoint_seq),
            "workflow-state-machine-adapter:checkpoint-appended",
        );
        self.checkpoints_by_key.insert(key, normalize(checkpoint));
        Ok(())
    }
}

fn normalize(mut checkpoint: StateCheckpoint) -> StateCheckpoint {
    checkpoint.evidence_refs = sorted_unique(checkpoint.evidence_refs);
    checkpoint
}

fn is_safe_checkpoint(checkpoint: &StateCheckpoint) -> bool {
    is_safe_tenant(&checkpoint.tenant_id)
        && is_safe_ref(&checkpoint.run_id)
        && is_safe_ref(&checkpoint.spec_id)
        && is_safe_ref(&checkpoint.version_sha)
        && is_safe_ref(&checkpoint.last_event_id)
        && is_safe_metadata(&checkpoint.last_event_type)
        && checkpoint
            .evidence_refs
            .iter()
            .all(|evidence_ref| is_safe_ref(evidence_ref))
}

fn is_safe_tenant(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.starts_with("ten_") && value == trimmed && is_safe_metadata(value)
}

fn is_safe_ref(value: &str) -> bool {
    is_safe_metadata(value) && value.contains(':')
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

    fn checkpoint(seq: u64) -> StateCheckpoint {
        StateCheckpoint {
            tenant_id: "ten_a".to_owned(),
            run_id: "run:workflow:memory:1".to_owned(),
            spec_id: "workflow-spec:invoice-approval".to_owned(),
            version_sha: "sha256:spec-v1".to_owned(),
            checkpoint_seq: seq,
            run_status: WorkflowRunStatus::Running,
            current_step_index: Some(0),
            step_status: Some(StepStatus::Running),
            last_event_id: format!("evt:memory:{seq}"),
            last_event_type: "StepStarted".to_owned(),
            evidence_refs: vec!["workflow-event:memory".to_owned()],
        }
    }

    fn event(sequence_num: u64, kind: WorkflowEventKind) -> WorkflowTransitionEvent {
        WorkflowTransitionEvent::new(
            &format!("evt:memory:{sequence_num}"),
            "ten_a",
            "run:workflow:memory:1",
            "workflow-spec:invoice-approval",
            "sha256:spec-v1",
            sequence_num,
            kind,
            &format!("workflow-event:memory:{sequence_num}"),
        )
        .expect("valid event")
    }

    fn input(sequence_num: u64, kind: WorkflowEventKind) -> StateMachineTransitionUsecaseInput {
        StateMachineTransitionUsecaseInput {
            request_id: format!("req:state-machine-memory:{sequence_num}"),
            idempotency_key: format!("idem:state-machine-memory:{sequence_num}"),
            trace_ref: format!("trace:state-machine-memory:{sequence_num}"),
            event: event(sequence_num, kind),
            expected_tenant_id: "ten_a".to_owned(),
            expected_spec_id: "workflow-spec:invoice-approval".to_owned(),
            expected_version_sha: "sha256:spec-v1".to_owned(),
            policy_evidence_ref: "cedar://workflow/state-machine/allow".to_owned(),
            spec_integrity_ref: "spec-integrity:workflow:v1".to_owned(),
            replay_epoch_ref: "replay-epoch:memory:1".to_owned(),
            origin: TransitionOrigin::TriggerOrchestrator,
        }
    }

    #[test]
    fn append_and_load_round_trip_is_tenant_scoped_and_monotonic() {
        let mut adapter = WorkflowStateMachineMemoryAdapter::default();

        adapter.append_checkpoint(1, checkpoint(1)).unwrap();
        assert_eq!(
            adapter
                .load_current("ten_a", "run:workflow:memory:1")
                .unwrap(),
            Some(checkpoint(1))
        );
        assert_eq!(
            adapter
                .load_current("ten_b", "run:workflow:memory:1")
                .unwrap(),
            None
        );

        let mut second = checkpoint(2);
        second.last_event_id = "evt:memory:2".to_owned();
        adapter.append_checkpoint(2, second.clone()).unwrap();
        assert_eq!(
            adapter
                .load_current("ten_a", "run:workflow:memory:1")
                .unwrap(),
            Some(second)
        );
        assert_eq!(adapter.checkpoint_count(), 1);
    }

    #[test]
    fn stale_expected_sequence_maps_to_conflict_without_mutating_current() {
        let mut adapter = WorkflowStateMachineMemoryAdapter::default();
        adapter.append_checkpoint(1, checkpoint(1)).unwrap();

        let failure = adapter.append_checkpoint(3, checkpoint(3)).unwrap_err();

        assert_eq!(
            failure,
            StateCheckpointAppendFailure::Conflict {
                expected_checkpoint_seq: 3,
                observed_checkpoint_seq: 1,
                evidence_ref: "workflow-state-machine-adapter:checkpoint-sequence-conflict"
                    .to_owned(),
            }
        );
        assert_eq!(
            adapter
                .load_current("ten_a", "run:workflow:memory:1")
                .unwrap(),
            Some(checkpoint(1))
        );
    }

    #[test]
    fn unsafe_metadata_is_rejected_before_store_mutation_without_echo() {
        let mut adapter = WorkflowStateMachineMemoryAdapter::default();
        let mut unsafe_checkpoint = checkpoint(1);
        unsafe_checkpoint.run_id = "run raw prompt Authorization: Bearer sk-test".to_owned();

        let failure = adapter.append_checkpoint(1, unsafe_checkpoint).unwrap_err();

        assert_eq!(
            failure,
            StateCheckpointAppendFailure::Unavailable {
                evidence_ref: "workflow-state-machine-adapter:unsafe-checkpoint-metadata"
                    .to_owned(),
            }
        );
        assert_eq!(adapter.checkpoint_count(), 0);
        let rendered = format!("{failure:?}").to_ascii_lowercase();
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("raw prompt"));
    }

    #[test]
    fn load_rejects_unsafe_tenant_or_run_before_lookup() {
        let mut adapter = WorkflowStateMachineMemoryAdapter::default();
        adapter.append_checkpoint(1, checkpoint(1)).unwrap();

        assert_eq!(
            adapter.load_current("ten_a", "run raw output").unwrap_err(),
            StateCheckpointStoreFailure::Unavailable {
                evidence_ref: "workflow-state-machine-adapter:unsafe-load-metadata".to_owned(),
            }
        );
    }

    #[test]
    fn usecase_applies_transitions_through_memory_adapter_without_durable_claims() {
        let mut adapter = WorkflowStateMachineMemoryAdapter::default();

        let start = apply_state_machine_transition(
            &mut adapter,
            input(1, WorkflowEventKind::WorkflowStarted),
        );
        assert_eq!(start.status, StateMachineUsecaseStatus::Applied);

        let step = apply_state_machine_transition(
            &mut adapter,
            input(2, WorkflowEventKind::StepStarted { step_index: 0 }),
        );
        assert_eq!(step.status, StateMachineUsecaseStatus::Applied);
        assert_eq!(step.checkpoint.unwrap().checkpoint_seq, 2);
        assert_eq!(
            adapter.adapter_mode(),
            StateMachineAdapterMode::InMemoryPreview
        );
        assert_eq!(adapter.recorded_actions().len(), 4);
    }
}
