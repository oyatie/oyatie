//! Workflow-engine execution-engine generic adapter foundation.
//!
//! This crate provides source-level in-memory adapters for preview integration
//! with the execution-engine usecase. It is intentionally non-durable and
//! performs no database, filesystem, network, queue, signing, wall-clock,
//! Valkey, Postgres, or cloud-runtime work.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

pub use workflow_execution_engine_usecase::{
    ExecutionDispatchError, ExecutionDomainCommandKind, ExecutionDomainOrigin,
    ExecutionEngineDomainRequest, ExecutionEngineUsecase, ExecutionEngineUsecaseInput,
    ExecutionStoreError, ExecutionUsecaseReceipt, ExecutionUsecaseStatus, RetryAttempt,
    RetryPolicyEvaluator, SlaTimer, SlaTimerStore, StepDispatcher, StepExecution,
    StepExecutionStatus, WorkflowExecutionStatus, WorkflowRun, WorkflowRunStore,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExecutionAdapterMode {
    InMemoryPreview,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExecutionAdapterActionKind {
    CancelTimer,
    CreateRun,
    DispatchStep,
    FireExpiredTimers,
    LoadRun,
    SaveStep,
    ScheduleRetryDelay,
    StoreConflict,
    TimerArmed,
    UnsafeMetadataRejected,
    UpdateRunStatus,
}

impl ExecutionAdapterActionKind {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::CancelTimer => "cancel-timer",
            Self::CreateRun => "create-run",
            Self::DispatchStep => "dispatch-step",
            Self::FireExpiredTimers => "fire-expired-timers",
            Self::LoadRun => "load-run",
            Self::SaveStep => "save-step",
            Self::ScheduleRetryDelay => "schedule-retry-delay",
            Self::StoreConflict => "store-conflict",
            Self::TimerArmed => "timer-armed",
            Self::UnsafeMetadataRejected => "unsafe-metadata-rejected",
            Self::UpdateRunStatus => "update-run-status",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionAdapterAction {
    pub kind: ExecutionAdapterActionKind, // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub run_id: String,                   // data_class: INTERNAL_ONLY
    pub step_index: Option<u32>,          // data_class: INTERNAL_ONLY
    pub attempt: Option<u32>,             // data_class: INTERNAL_ONLY
    pub expected_version: Option<u64>,    // data_class: INTERNAL_ONLY
    pub observed_version: Option<u64>,    // data_class: INTERNAL_ONLY
    pub evidence_ref: String,             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct RunKey {
    tenant_id: String,
    run_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct StepKey {
    tenant_id: String,
    run_id: String,
    step_id: String,
    attempt: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct TimerKey {
    tenant_id: String,
    timer_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowExecutionMemoryAdapter {
    runs_by_key: BTreeMap<RunKey, WorkflowRun>,
    steps_by_key: BTreeMap<StepKey, StepExecution>,
    timers_by_key: BTreeMap<TimerKey, SlaTimer>,
    recorded_actions: Vec<ExecutionAdapterAction>,
    retry_base_delay_seconds: u64,
    retry_multiplier: u64,
    retry_max_delay_seconds: u64,
    retry_max_attempts: u32,
}

impl Default for WorkflowExecutionMemoryAdapter {
    fn default() -> Self {
        Self {
            runs_by_key: BTreeMap::new(),
            steps_by_key: BTreeMap::new(),
            timers_by_key: BTreeMap::new(),
            recorded_actions: Vec::new(),
            retry_base_delay_seconds: 5,
            retry_multiplier: 2,
            retry_max_delay_seconds: 300,
            retry_max_attempts: 5,
        }
    }
}

pub type InMemoryExecutionEngineAdapter = WorkflowExecutionMemoryAdapter;

impl WorkflowExecutionMemoryAdapter {
    pub fn adapter_mode(&self) -> ExecutionAdapterMode {
        ExecutionAdapterMode::InMemoryPreview
    }

    pub fn run_count(&self) -> usize {
        self.runs_by_key.len()
    }

    pub fn step_count(&self) -> usize {
        self.steps_by_key.len()
    }

    pub fn timer_count(&self) -> usize {
        self.timers_by_key.len()
    }

    pub fn recorded_actions(&self) -> &[ExecutionAdapterAction] {
        &self.recorded_actions
    }

    pub fn with_retry_policy(
        base_delay_seconds: u64,
        multiplier: u64,
        max_delay_seconds: u64,
        max_attempts: u32,
    ) -> Self {
        Self {
            retry_base_delay_seconds: base_delay_seconds.max(1),
            retry_multiplier: multiplier.max(1),
            retry_max_delay_seconds: max_delay_seconds.max(1),
            retry_max_attempts: max_attempts.max(1),
            ..Self::default()
        }
    }

    fn push_action(&mut self, action: ExecutionAdapterAction) {
        self.recorded_actions.push(action);
    }
}

#[derive(Default, Debug)]
pub struct WorkflowExecutionMemoryAdapterBundle {
    pub usecase: ExecutionEngineUsecase, // data_class: INTERNAL_ONLY
    pub store: WorkflowExecutionMemoryAdapter, // data_class: INTERNAL_ONLY
    pub dispatcher: WorkflowExecutionMemoryAdapter, // data_class: INTERNAL_ONLY
    pub retry_policy: WorkflowExecutionMemoryAdapter, // data_class: INTERNAL_ONLY
    pub timers: WorkflowExecutionMemoryAdapter, // data_class: INTERNAL_ONLY
}

impl WorkflowExecutionMemoryAdapterBundle {
    pub fn apply(&mut self, input: ExecutionEngineUsecaseInput) -> ExecutionUsecaseReceipt {
        self.usecase.apply(
            &mut self.store,
            &mut self.dispatcher,
            &self.retry_policy,
            &mut self.timers,
            input,
        )
    }
}

impl WorkflowRunStore for WorkflowExecutionMemoryAdapter {
    fn create_run(&mut self, run: WorkflowRun) -> Result<(), ExecutionStoreError> {
        if !is_safe_run(&run) {
            self.push_redacted_action(ExecutionAdapterActionKind::UnsafeMetadataRejected);
            return Err(ExecutionStoreError::Unavailable {
                evidence_ref: "workflow-execution-adapter:unsafe-run-metadata".to_owned(),
            });
        }
        let key = run_key(&run.tenant_id, &run.run_id);
        if let Some(current) = self.runs_by_key.get(&key) {
            let observed_version = current.version;
            self.push_action(action(
                ExecutionAdapterActionKind::StoreConflict,
                &run.tenant_id,
                &run.run_id,
                None,
                None,
                Some(run.version),
                Some(observed_version),
                "workflow-execution-adapter:run-already-exists",
            ));
            return Err(ExecutionStoreError::Conflict {
                expected_version: run.version,
                observed_version,
                evidence_ref: "workflow-execution-adapter:run-already-exists".to_owned(),
            });
        }
        self.push_action(action(
            ExecutionAdapterActionKind::CreateRun,
            &run.tenant_id,
            &run.run_id,
            run.current_step_index,
            None,
            Some(run.version),
            None,
            "workflow-execution-adapter:run-created",
        ));
        self.runs_by_key.insert(key, normalize_run(run));
        Ok(())
    }

    fn load_run(
        &self,
        tenant_id: &str,
        run_id: &str,
    ) -> Result<Option<WorkflowRun>, ExecutionStoreError> {
        if !is_safe_tenant(tenant_id) || !is_safe_ref(run_id) {
            return Err(ExecutionStoreError::Unavailable {
                evidence_ref: "workflow-execution-adapter:unsafe-load-metadata".to_owned(),
            });
        }
        Ok(self.runs_by_key.get(&run_key(tenant_id, run_id)).cloned())
    }

    fn update_run_status(
        &mut self,
        tenant_id: &str,
        run_id: &str,
        expected_version: u64,
        status: WorkflowExecutionStatus,
        evidence_ref: &str,
    ) -> Result<(), ExecutionStoreError> {
        if !is_safe_tenant(tenant_id) || !is_safe_ref(run_id) || !is_safe_ref(evidence_ref) {
            self.push_redacted_action(ExecutionAdapterActionKind::UnsafeMetadataRejected);
            return Err(ExecutionStoreError::Unavailable {
                evidence_ref: "workflow-execution-adapter:unsafe-update-metadata".to_owned(),
            });
        }
        let key = run_key(tenant_id, run_id);
        let Some(current_snapshot) = self.runs_by_key.get(&key) else {
            return Err(ExecutionStoreError::Unavailable {
                evidence_ref: "workflow-execution-adapter:run-missing".to_owned(),
            });
        };
        if current_snapshot.version != expected_version {
            let observed_version = current_snapshot.version;
            let current_step_index = current_snapshot.current_step_index;
            self.push_action(action(
                ExecutionAdapterActionKind::StoreConflict,
                tenant_id,
                run_id,
                current_step_index,
                None,
                Some(expected_version),
                Some(observed_version),
                "workflow-execution-adapter:run-version-conflict",
            ));
            return Err(ExecutionStoreError::Conflict {
                expected_version,
                observed_version,
                evidence_ref: "workflow-execution-adapter:run-version-conflict".to_owned(),
            });
        }
        let (current_step_index, observed_version) = {
            let current = self
                .runs_by_key
                .get_mut(&key)
                .expect("run existed for version check");
            current.status = status;
            current.version = current.version.saturating_add(1);
            (current.current_step_index, current.version)
        };
        self.push_action(action(
            ExecutionAdapterActionKind::UpdateRunStatus,
            tenant_id,
            run_id,
            current_step_index,
            None,
            Some(expected_version),
            Some(observed_version),
            evidence_ref,
        ));
        Ok(())
    }

    fn save_step(&mut self, step: StepExecution) -> Result<(), ExecutionStoreError> {
        if !is_safe_step(&step) {
            self.push_redacted_action(ExecutionAdapterActionKind::UnsafeMetadataRejected);
            return Err(ExecutionStoreError::Unavailable {
                evidence_ref: "workflow-execution-adapter:unsafe-step-metadata".to_owned(),
            });
        }
        self.push_action(action(
            ExecutionAdapterActionKind::SaveStep,
            &step.tenant_id,
            &step.run_id,
            Some(step.step_index),
            Some(step.attempt),
            None,
            None,
            "workflow-execution-adapter:step-saved",
        ));
        self.steps_by_key
            .insert(step_key(&step), normalize_step(step));
        Ok(())
    }
}

impl StepDispatcher for WorkflowExecutionMemoryAdapter {
    fn dispatch_step(
        &mut self,
        tenant_id: &str,
        run_id: &str,
        step_index: u32,
        evidence_ref: &str,
    ) -> Result<(), ExecutionDispatchError> {
        if !is_safe_tenant(tenant_id) || !is_safe_ref(run_id) || !is_safe_ref(evidence_ref) {
            self.push_redacted_action(ExecutionAdapterActionKind::UnsafeMetadataRejected);
            return Err(ExecutionDispatchError::Denied {
                evidence_ref: "workflow-execution-adapter:unsafe-dispatch-metadata".to_owned(),
            });
        }
        self.push_action(action(
            ExecutionAdapterActionKind::DispatchStep,
            tenant_id,
            run_id,
            Some(step_index),
            None,
            None,
            None,
            evidence_ref,
        ));
        Ok(())
    }
}

impl RetryPolicyEvaluator for WorkflowExecutionMemoryAdapter {
    fn next_delay_seconds(
        &self,
        attempt: &RetryAttempt,
    ) -> Result<Option<u64>, workflow_execution_engine_usecase::ExecutionEngineKernelError> {
        if !is_safe_retry(attempt) || attempt.attempt == 0 {
            return Err(
                workflow_execution_engine_usecase::ExecutionEngineKernelError::UnsafeMetadata,
            );
        }
        if attempt.attempt > self.retry_max_attempts {
            return Ok(None);
        }
        let exponent = attempt.attempt.saturating_sub(1);
        let multiplier = self.retry_multiplier.saturating_pow(exponent);
        let delay = self
            .retry_base_delay_seconds
            .saturating_mul(multiplier)
            .min(self.retry_max_delay_seconds);
        Ok(Some(delay))
    }
}

impl SlaTimerStore for WorkflowExecutionMemoryAdapter {
    fn arm_timer(&mut self, timer: SlaTimer) -> Result<(), ExecutionStoreError> {
        if !is_safe_timer(&timer) {
            self.push_redacted_action(ExecutionAdapterActionKind::UnsafeMetadataRejected);
            return Err(ExecutionStoreError::Unavailable {
                evidence_ref: "workflow-execution-adapter:unsafe-timer-metadata".to_owned(),
            });
        }
        self.push_action(action(
            ExecutionAdapterActionKind::TimerArmed,
            &timer.tenant_id,
            &timer.run_id,
            timer.step_index,
            None,
            None,
            None,
            "workflow-execution-adapter:timer-armed",
        ));
        self.timers_by_key.insert(timer_key(&timer), timer);
        Ok(())
    }

    fn cancel_timer(&mut self, tenant_id: &str, timer_id: &str) -> Result<(), ExecutionStoreError> {
        if !is_safe_tenant(tenant_id) || !is_safe_ref(timer_id) {
            self.push_redacted_action(ExecutionAdapterActionKind::UnsafeMetadataRejected);
            return Err(ExecutionStoreError::Unavailable {
                evidence_ref: "workflow-execution-adapter:unsafe-cancel-timer-metadata".to_owned(),
            });
        }
        self.timers_by_key.remove(&TimerKey {
            tenant_id: tenant_id.to_owned(),
            timer_id: timer_id.to_owned(),
        });
        self.push_action(action(
            ExecutionAdapterActionKind::CancelTimer,
            tenant_id,
            timer_id,
            None,
            None,
            None,
            None,
            "workflow-execution-adapter:timer-cancelled",
        ));
        Ok(())
    }

    fn fire_expired(
        &mut self,
        tenant_id: &str,
        now_epoch_seconds: u64,
    ) -> Result<Vec<SlaTimer>, ExecutionStoreError> {
        if !is_safe_tenant(tenant_id) {
            self.push_redacted_action(ExecutionAdapterActionKind::UnsafeMetadataRejected);
            return Err(ExecutionStoreError::Unavailable {
                evidence_ref: "workflow-execution-adapter:unsafe-fire-timer-metadata".to_owned(),
            });
        }
        let expired_keys: Vec<TimerKey> = self
            .timers_by_key
            .iter()
            .filter(|(key, timer)| {
                key.tenant_id == tenant_id && timer.deadline_epoch_seconds <= now_epoch_seconds
            })
            .map(|(key, _)| key.clone())
            .collect();
        let mut expired = Vec::new();
        for key in expired_keys {
            if let Some(timer) = self.timers_by_key.remove(&key) {
                expired.push(timer);
            }
        }
        self.push_action(action(
            ExecutionAdapterActionKind::FireExpiredTimers,
            tenant_id,
            "run:timer-scan",
            None,
            None,
            None,
            None,
            "workflow-execution-adapter:timers-fired",
        ));
        Ok(expired)
    }
}

impl WorkflowExecutionMemoryAdapter {
    fn push_redacted_action(&mut self, kind: ExecutionAdapterActionKind) {
        self.push_action(action(
            kind,
            "redacted-invalid-tenant-id",
            "redacted-invalid-run-id",
            None,
            None,
            None,
            None,
            "workflow-execution-adapter:redacted-invalid-metadata",
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn action(
    kind: ExecutionAdapterActionKind,
    tenant_id: &str,
    run_id: &str,
    step_index: Option<u32>,
    attempt: Option<u32>,
    expected_version: Option<u64>,
    observed_version: Option<u64>,
    evidence_ref: &str,
) -> ExecutionAdapterAction {
    ExecutionAdapterAction {
        kind,
        tenant_id: tenant_id.to_owned(),
        run_id: run_id.to_owned(),
        step_index,
        attempt,
        expected_version,
        observed_version,
        evidence_ref: evidence_ref.to_owned(),
    }
}

fn run_key(tenant_id: &str, run_id: &str) -> RunKey {
    RunKey {
        tenant_id: tenant_id.to_owned(),
        run_id: run_id.to_owned(),
    }
}

fn step_key(step: &StepExecution) -> StepKey {
    StepKey {
        tenant_id: step.tenant_id.clone(),
        run_id: step.run_id.clone(),
        step_id: step.step_id.clone(),
        attempt: step.attempt,
    }
}

fn timer_key(timer: &SlaTimer) -> TimerKey {
    TimerKey {
        tenant_id: timer.tenant_id.clone(),
        timer_id: timer.timer_id.clone(),
    }
}

fn normalize_run(mut run: WorkflowRun) -> WorkflowRun {
    run.evidence_refs = sorted_unique(run.evidence_refs);
    run
}

fn normalize_step(mut step: StepExecution) -> StepExecution {
    step.evidence_refs = sorted_unique(step.evidence_refs);
    step
}

fn is_safe_run(run: &WorkflowRun) -> bool {
    is_safe_tenant(&run.tenant_id)
        && is_safe_ref(&run.run_id)
        && is_safe_ref(&run.spec_id)
        && is_safe_ref(&run.version_sha)
        && is_safe_ref(&run.active_cell_id)
        && run.evidence_refs.iter().all(|value| is_safe_ref(value))
}

fn is_safe_step(step: &StepExecution) -> bool {
    is_safe_tenant(&step.tenant_id)
        && is_safe_ref(&step.run_id)
        && is_safe_ref(&step.step_id)
        && is_safe_ref(&step.idempotency_key)
        && step
            .lease_owner_ref
            .as_ref()
            .is_none_or(|value| is_safe_ref(value))
        && step
            .side_effect_ref
            .as_ref()
            .is_none_or(|value| is_safe_ref(value))
        && step
            .last_error_ref
            .as_ref()
            .is_none_or(|value| is_safe_ref(value))
        && step.evidence_refs.iter().all(|value| is_safe_ref(value))
}

fn is_safe_retry(retry: &RetryAttempt) -> bool {
    is_safe_tenant(&retry.tenant_id)
        && is_safe_ref(&retry.run_id)
        && is_safe_ref(&retry.step_id)
        && is_safe_ref(&retry.error_class_ref)
        && is_safe_ref(&retry.retry_policy_ref)
        && retry.evidence_refs.iter().all(|value| is_safe_ref(value))
}

fn is_safe_timer(timer: &SlaTimer) -> bool {
    is_safe_ref(&timer.timer_id)
        && is_safe_tenant(&timer.tenant_id)
        && is_safe_ref(&timer.run_id)
        && timer.evidence_refs.iter().all(|value| is_safe_ref(value))
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

    fn run_with_status(status: WorkflowExecutionStatus, version: u64) -> WorkflowRun {
        let mut run = WorkflowRun::new(
            "ten_a",
            "run:execution-adapter:1",
            "workflow-spec:invoice-approval",
            "sha256:spec-v1",
            "cell:use1:a",
            vec![
                "workflow-execution-adapter:run".to_owned(),
                "workflow-execution-adapter:run".to_owned(),
            ],
        )
        .unwrap();
        run.status = status;
        run.version = version;
        run
    }

    fn step_with_status(status: StepExecutionStatus, attempt: u32) -> StepExecution {
        let mut step = StepExecution::new(
            "ten_a",
            "run:execution-adapter:1",
            "step:approve",
            0,
            attempt,
            "idempotency:step:approve:adapter:1",
            vec!["workflow-execution-adapter:step".to_owned()],
        )
        .unwrap();
        step.status = status;
        step
    }

    fn retry_attempt(attempt: u32) -> RetryAttempt {
        RetryAttempt::new(
            "ten_a",
            "run:execution-adapter:1",
            "step:approve",
            attempt,
            "error-class:retryable-http-503",
            "retry-policy:workflow-standard",
            vec!["workflow-execution-adapter:retry".to_owned()],
        )
        .unwrap()
    }

    fn timer(deadline_epoch_seconds: u64) -> SlaTimer {
        SlaTimer::new(
            "timer:execution-adapter:1",
            "ten_a",
            "run:execution-adapter:1",
            Some(0),
            100,
            deadline_epoch_seconds,
            vec!["workflow-execution-adapter:sla".to_owned()],
        )
        .unwrap()
    }

    fn dispatch_input() -> ExecutionEngineUsecaseInput {
        ExecutionEngineUsecaseInput {
            request_id: "req:execution-adapter:1".to_owned(),
            idempotency_key: "idem:execution-adapter:1".to_owned(),
            trace_ref: "trace:execution-adapter:1".to_owned(),
            expected_run_version: 7,
            domain_request: ExecutionEngineDomainRequest {
                run: run_with_status(WorkflowExecutionStatus::Running, 7),
                step: Some(step_with_status(StepExecutionStatus::Pending, 1)),
                retry_attempt: None,
                sla_timer: None,
                expected_tenant_id: "ten_a".to_owned(),
                expected_spec_id: "workflow-spec:invoice-approval".to_owned(),
                expected_version_sha: "sha256:spec-v1".to_owned(),
                expected_cell_id: "cell:use1:a".to_owned(),
                policy_evidence_ref: "cedar://workflow/execution/dispatch".to_owned(),
                spec_integrity_ref: "spec-integrity:workflow:v1".to_owned(),
                replay_epoch_ref: "replay-epoch:execution-adapter:1".to_owned(),
                scheduler_epoch_ref: "scheduler-epoch:execution-adapter:1".to_owned(),
                sla_reference_epoch_seconds: 0,
                command: ExecutionDomainCommandKind::DispatchStep,
                origin: ExecutionDomainOrigin::WorkerScheduler,
            },
        }
    }

    #[test]
    fn memory_adapter_loads_updates_steps_and_records_metadata_only_actions() {
        let mut adapter = WorkflowExecutionMemoryAdapter::default();
        adapter
            .create_run(run_with_status(WorkflowExecutionStatus::Pending, 1))
            .unwrap();
        adapter
            .update_run_status(
                "ten_a",
                "run:execution-adapter:1",
                1,
                WorkflowExecutionStatus::Running,
                "workflow-execution-adapter:update",
            )
            .unwrap();
        adapter
            .save_step(step_with_status(StepExecutionStatus::Leased, 1))
            .unwrap();

        assert_eq!(
            adapter.adapter_mode(),
            ExecutionAdapterMode::InMemoryPreview
        );
        assert_eq!(adapter.run_count(), 1);
        assert_eq!(adapter.step_count(), 1);
        assert_eq!(
            adapter
                .load_run("ten_a", "run:execution-adapter:1")
                .unwrap()
                .unwrap()
                .version,
            2
        );
        assert!(adapter.recorded_actions().iter().any(|action| {
            action.kind == ExecutionAdapterActionKind::UpdateRunStatus
                && action.evidence_ref == "workflow-execution-adapter:update"
        }));
    }

    #[test]
    fn stale_run_version_maps_to_conflict_without_mutating_current() {
        let mut adapter = WorkflowExecutionMemoryAdapter::default();
        adapter
            .create_run(run_with_status(WorkflowExecutionStatus::Running, 7))
            .unwrap();

        let failure = adapter
            .update_run_status(
                "ten_a",
                "run:execution-adapter:1",
                6,
                WorkflowExecutionStatus::Completed,
                "workflow-execution-adapter:update",
            )
            .unwrap_err();

        assert_eq!(
            failure,
            ExecutionStoreError::Conflict {
                expected_version: 6,
                observed_version: 7,
                evidence_ref: "workflow-execution-adapter:run-version-conflict".to_owned(),
            }
        );
        assert_eq!(
            adapter
                .load_run("ten_a", "run:execution-adapter:1")
                .unwrap()
                .unwrap()
                .status,
            WorkflowExecutionStatus::Running
        );
    }

    #[test]
    fn unsafe_metadata_is_rejected_before_mutation_without_echo() {
        let mut adapter = WorkflowExecutionMemoryAdapter::default();
        let mut unsafe_run = run_with_status(WorkflowExecutionStatus::Pending, 1);
        unsafe_run.run_id = "run raw prompt Authorization: Bearer sk-test".to_owned();

        let failure = adapter.create_run(unsafe_run).unwrap_err();

        assert_eq!(
            failure,
            ExecutionStoreError::Unavailable {
                evidence_ref: "workflow-execution-adapter:unsafe-run-metadata".to_owned(),
            }
        );
        assert_eq!(adapter.run_count(), 0);
        let rendered = format!("{failure:?}").to_ascii_lowercase();
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("raw prompt"));
    }

    #[test]
    fn dispatcher_and_retry_policy_are_source_level_metadata_only_ports() {
        let mut dispatcher = WorkflowExecutionMemoryAdapter::default();
        dispatcher
            .dispatch_step(
                "ten_a",
                "run:execution-adapter:1",
                0,
                "workflow-execution-adapter:dispatch",
            )
            .unwrap();
        assert_eq!(
            dispatcher.recorded_actions()[0].kind,
            ExecutionAdapterActionKind::DispatchStep
        );

        let retry = WorkflowExecutionMemoryAdapter::with_retry_policy(5, 2, 60, 5);
        assert_eq!(
            retry.next_delay_seconds(&retry_attempt(1)).unwrap(),
            Some(5)
        );
        assert_eq!(
            retry.next_delay_seconds(&retry_attempt(3)).unwrap(),
            Some(20)
        );
        assert_eq!(retry.next_delay_seconds(&retry_attempt(6)).unwrap(), None);
    }

    #[test]
    fn sla_timer_store_arms_cancels_and_fires_with_injected_epoch_only() {
        let mut timers = WorkflowExecutionMemoryAdapter::default();
        timers.arm_timer(timer(130)).unwrap();
        assert_eq!(timers.timer_count(), 1);
        assert!(timers.fire_expired("ten_a", 129).unwrap().is_empty());
        let expired = timers.fire_expired("ten_a", 130).unwrap();
        assert_eq!(expired.len(), 1);
        assert_eq!(timers.timer_count(), 0);

        timers.arm_timer(timer(160)).unwrap();
        timers
            .cancel_timer("ten_a", "timer:execution-adapter:1")
            .unwrap();
        assert_eq!(timers.timer_count(), 0);
    }

    #[test]
    fn usecase_dispatches_through_memory_adapter_bundle_without_durable_claims() {
        let mut bundle = WorkflowExecutionMemoryAdapterBundle::default();
        bundle
            .store
            .create_run(run_with_status(WorkflowExecutionStatus::Running, 7))
            .unwrap();

        let receipt = bundle.apply(dispatch_input());

        assert_eq!(receipt.status, ExecutionUsecaseStatus::Applied);
        assert_eq!(receipt.step_status, Some(StepExecutionStatus::Leased));
        assert_eq!(bundle.store.step_count(), 1);
        assert_eq!(
            bundle
                .store
                .load_run("ten_a", "run:execution-adapter:1")
                .unwrap()
                .unwrap()
                .version,
            8
        );
        assert!(bundle.dispatcher.recorded_actions().iter().any(|action| {
            action.kind == ExecutionAdapterActionKind::DispatchStep
                && action.tenant_id == "ten_a"
                && action.run_id == "run:execution-adapter:1"
        }));
    }

    #[test]
    fn usecase_retry_and_timer_commands_flow_through_adapter_bundle() {
        let mut retry_bundle = WorkflowExecutionMemoryAdapterBundle::default();
        retry_bundle
            .store
            .create_run(run_with_status(WorkflowExecutionStatus::Running, 7))
            .unwrap();
        let mut retry_input = dispatch_input();
        retry_input.idempotency_key = "idem:execution-adapter:retry".to_owned();
        retry_input.domain_request.command = ExecutionDomainCommandKind::ScheduleRetry;
        retry_input.domain_request.step = Some(step_with_status(StepExecutionStatus::Failed, 1));
        retry_input.domain_request.retry_attempt = Some(retry_attempt(2));

        let retry_receipt = retry_bundle.apply(retry_input);

        assert_eq!(retry_receipt.status, ExecutionUsecaseStatus::Applied);
        assert_eq!(retry_receipt.retry_delay_seconds, Some(10));
        assert!(retry_bundle.dispatcher.recorded_actions().is_empty());

        let mut timer_bundle = WorkflowExecutionMemoryAdapterBundle::default();
        timer_bundle
            .store
            .create_run(run_with_status(WorkflowExecutionStatus::Running, 7))
            .unwrap();
        let mut timer_input = dispatch_input();
        timer_input.idempotency_key = "idem:execution-adapter:timer".to_owned();
        timer_input.domain_request.command = ExecutionDomainCommandKind::ArmSlaTimer;
        timer_input.domain_request.step = None;
        timer_input.domain_request.sla_timer = Some(timer(150));

        let timer_receipt = timer_bundle.apply(timer_input);

        assert_eq!(timer_receipt.status, ExecutionUsecaseStatus::Applied);
        assert_eq!(timer_bundle.timers.timer_count(), 1);
    }
}
