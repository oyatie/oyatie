//! Workflow-engine execution-engine kernel foundation.
//!
//! This crate owns the source-level execution-engine value kernel: run and step
//! execution entities, retry-attempt and SLA-timer metadata, and protocol-neutral
//! port traits for later durable adapters. It performs no database, filesystem,
//! network, wall-clock, random, queue, signing, Valkey, Postgres, or cloud
//! runtime work.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowExecutionStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl WorkflowExecutionStatus {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_wire(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "running" => Some(Self::Running),
            "paused" => Some(Self::Paused),
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
pub enum StepExecutionStatus {
    Pending,
    Leased,
    Running,
    Succeeded,
    Failed,
    Retrying,
    TimedOut,
    Cancelled,
}

impl StepExecutionStatus {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Leased => "leased",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Retrying => "retrying",
            Self::TimedOut => "timed-out",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_wire(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "leased" => Some(Self::Leased),
            "running" => Some(Self::Running),
            "succeeded" => Some(Self::Succeeded),
            "failed" => Some(Self::Failed),
            "retrying" => Some(Self::Retrying),
            "timed-out" => Some(Self::TimedOut),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::TimedOut | Self::Cancelled
        )
    }

    pub fn is_terminal_failure(self) -> bool {
        matches!(self, Self::Failed | Self::TimedOut | Self::Cancelled)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExecutionEngineKernelError {
    InvalidAttempt,
    InvalidStepIndex,
    InvalidTimerDeadline,
    UnsafeMetadata,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionStoreError {
    Conflict {
        expected_version: u64,
        observed_version: u64,
        evidence_ref: String,
    },
    Unavailable {
        evidence_ref: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionDispatchError {
    Denied { evidence_ref: String },
    Unavailable { evidence_ref: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowRun {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub run_id: String,                  // data_class: INTERNAL_ONLY
    pub spec_id: String,                 // data_class: INTERNAL_ONLY
    pub version_sha: String,             // data_class: INTERNAL_ONLY
    pub active_cell_id: String,          // data_class: INTERNAL_ONLY
    pub status: WorkflowExecutionStatus, // data_class: PUBLIC
    pub version: u64,                    // data_class: INTERNAL_ONLY
    pub current_step_index: Option<u32>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,      // data_class: INTERNAL_ONLY
}

impl WorkflowRun {
    pub fn new(
        tenant_id: &str,
        run_id: &str,
        spec_id: &str,
        version_sha: &str,
        active_cell_id: &str,
        evidence_refs: Vec<String>,
    ) -> Result<Self, ExecutionEngineKernelError> {
        if !is_safe_tenant(tenant_id)
            || !is_safe_ref(run_id)
            || !is_safe_ref(spec_id)
            || !is_safe_ref(version_sha)
            || !is_safe_ref(active_cell_id)
            || !evidence_refs.iter().all(|value| is_safe_ref(value))
        {
            return Err(ExecutionEngineKernelError::UnsafeMetadata);
        }
        Ok(Self {
            tenant_id: tenant_id.to_owned(),
            run_id: run_id.to_owned(),
            spec_id: spec_id.to_owned(),
            version_sha: version_sha.to_owned(),
            active_cell_id: active_cell_id.to_owned(),
            status: WorkflowExecutionStatus::Pending,
            version: 1,
            current_step_index: None,
            evidence_refs: sorted_unique(evidence_refs),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepExecution {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub run_id: String,                  // data_class: INTERNAL_ONLY
    pub step_id: String,                 // data_class: INTERNAL_ONLY
    pub step_index: u32,                 // data_class: INTERNAL_ONLY
    pub attempt: u32,                    // data_class: INTERNAL_ONLY
    pub status: StepExecutionStatus,     // data_class: PUBLIC
    pub idempotency_key: String,         // data_class: INTERNAL_ONLY
    pub lease_owner_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub side_effect_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub last_error_ref: Option<String>,  // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,      // data_class: INTERNAL_ONLY
}

impl StepExecution {
    pub fn new(
        tenant_id: &str,
        run_id: &str,
        step_id: &str,
        step_index: u32,
        attempt: u32,
        idempotency_key: &str,
        evidence_refs: Vec<String>,
    ) -> Result<Self, ExecutionEngineKernelError> {
        if attempt == 0 {
            return Err(ExecutionEngineKernelError::InvalidAttempt);
        }
        if !is_safe_tenant(tenant_id)
            || !is_safe_ref(run_id)
            || !is_safe_ref(step_id)
            || !is_safe_ref(idempotency_key)
            || !evidence_refs.iter().all(|value| is_safe_ref(value))
        {
            return Err(ExecutionEngineKernelError::UnsafeMetadata);
        }
        Ok(Self {
            tenant_id: tenant_id.to_owned(),
            run_id: run_id.to_owned(),
            step_id: step_id.to_owned(),
            step_index,
            attempt,
            status: StepExecutionStatus::Pending,
            idempotency_key: idempotency_key.to_owned(),
            lease_owner_ref: None,
            side_effect_ref: None,
            last_error_ref: None,
            evidence_refs: sorted_unique(evidence_refs),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetryAttempt {
    pub tenant_id: String,          // data_class: INTERNAL_ONLY
    pub run_id: String,             // data_class: INTERNAL_ONLY
    pub step_id: String,            // data_class: INTERNAL_ONLY
    pub attempt: u32,               // data_class: INTERNAL_ONLY
    pub error_class_ref: String,    // data_class: INTERNAL_ONLY
    pub retry_policy_ref: String,   // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>, // data_class: INTERNAL_ONLY
}

impl RetryAttempt {
    pub fn new(
        tenant_id: &str,
        run_id: &str,
        step_id: &str,
        attempt: u32,
        error_class_ref: &str,
        retry_policy_ref: &str,
        evidence_refs: Vec<String>,
    ) -> Result<Self, ExecutionEngineKernelError> {
        if attempt == 0 {
            return Err(ExecutionEngineKernelError::InvalidAttempt);
        }
        if !is_safe_tenant(tenant_id)
            || !is_safe_ref(run_id)
            || !is_safe_ref(step_id)
            || !is_safe_ref(error_class_ref)
            || !is_safe_ref(retry_policy_ref)
            || !evidence_refs.iter().all(|value| is_safe_ref(value))
        {
            return Err(ExecutionEngineKernelError::UnsafeMetadata);
        }
        Ok(Self {
            tenant_id: tenant_id.to_owned(),
            run_id: run_id.to_owned(),
            step_id: step_id.to_owned(),
            attempt,
            error_class_ref: error_class_ref.to_owned(),
            retry_policy_ref: retry_policy_ref.to_owned(),
            evidence_refs: sorted_unique(evidence_refs),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlaTimer {
    pub timer_id: String,            // data_class: INTERNAL_ONLY
    pub tenant_id: String,           // data_class: INTERNAL_ONLY
    pub run_id: String,              // data_class: INTERNAL_ONLY
    pub step_index: Option<u32>,     // data_class: INTERNAL_ONLY
    pub armed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub deadline_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,  // data_class: INTERNAL_ONLY
}

impl SlaTimer {
    pub fn new(
        timer_id: &str,
        tenant_id: &str,
        run_id: &str,
        step_index: Option<u32>,
        armed_at_epoch_seconds: u64,
        deadline_epoch_seconds: u64,
        evidence_refs: Vec<String>,
    ) -> Result<Self, ExecutionEngineKernelError> {
        if deadline_epoch_seconds <= armed_at_epoch_seconds {
            return Err(ExecutionEngineKernelError::InvalidTimerDeadline);
        }
        if !is_safe_ref(timer_id)
            || !is_safe_tenant(tenant_id)
            || !is_safe_ref(run_id)
            || !evidence_refs.iter().all(|value| is_safe_ref(value))
        {
            return Err(ExecutionEngineKernelError::UnsafeMetadata);
        }
        Ok(Self {
            timer_id: timer_id.to_owned(),
            tenant_id: tenant_id.to_owned(),
            run_id: run_id.to_owned(),
            step_index,
            armed_at_epoch_seconds,
            deadline_epoch_seconds,
            evidence_refs: sorted_unique(evidence_refs),
        })
    }
}

pub trait WorkflowRunStore {
    fn create_run(&mut self, run: WorkflowRun) -> Result<(), ExecutionStoreError>;

    fn load_run(
        &self,
        tenant_id: &str,
        run_id: &str,
    ) -> Result<Option<WorkflowRun>, ExecutionStoreError>;

    fn update_run_status(
        &mut self,
        tenant_id: &str,
        run_id: &str,
        expected_version: u64,
        status: WorkflowExecutionStatus,
        evidence_ref: &str,
    ) -> Result<(), ExecutionStoreError>;

    fn save_step(&mut self, step: StepExecution) -> Result<(), ExecutionStoreError>;
}

pub trait StepDispatcher {
    fn dispatch_step(
        &mut self,
        tenant_id: &str,
        run_id: &str,
        step_index: u32,
        evidence_ref: &str,
    ) -> Result<(), ExecutionDispatchError>;
}

pub trait RetryPolicyEvaluator {
    fn next_delay_seconds(
        &self,
        attempt: &RetryAttempt,
    ) -> Result<Option<u64>, ExecutionEngineKernelError>;
}

pub trait SlaTimerStore {
    fn arm_timer(&mut self, timer: SlaTimer) -> Result<(), ExecutionStoreError>;

    fn cancel_timer(&mut self, tenant_id: &str, timer_id: &str) -> Result<(), ExecutionStoreError>;

    fn fire_expired(
        &mut self,
        tenant_id: &str,
        now_epoch_seconds: u64,
    ) -> Result<Vec<SlaTimer>, ExecutionStoreError>;
}

pub trait EphemeralStateStore {
    fn claim_step_lease(
        &mut self,
        tenant_id: &str,
        run_id: &str,
        step_index: u32,
        worker_ref: &str,
        ttl_seconds: u64,
    ) -> Result<bool, ExecutionStoreError>;

    fn release_step_lease(
        &mut self,
        tenant_id: &str,
        run_id: &str,
        step_index: u32,
        worker_ref: &str,
    ) -> Result<(), ExecutionStoreError>;
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

    fn run() -> WorkflowRun {
        WorkflowRun::new(
            "ten_a",
            "run:execution:1",
            "workflow-spec:invoice-approval",
            "sha256:spec-v1",
            "cell:use1:a",
            vec![
                "workflow-execution:requested".to_owned(),
                "workflow-execution:requested".to_owned(),
            ],
        )
        .unwrap()
    }

    #[test]
    fn workflow_run_validates_identity_and_sorts_evidence_refs() {
        let run = run();

        assert_eq!(run.status, WorkflowExecutionStatus::Pending);
        assert_eq!(run.version, 1);
        assert_eq!(
            run.evidence_refs,
            vec!["workflow-execution:requested".to_owned()]
        );
        assert_eq!(run.tenant_id, "ten_a");
    }

    #[test]
    fn step_execution_rejects_raw_prompt_or_secret_refs_without_echo() {
        let err = StepExecution::new(
            "ten_a",
            "run:execution:1",
            "step:approve",
            0,
            1,
            "idem raw prompt Authorization: Bearer sk-test",
            vec!["workflow-execution:step".to_owned()],
        )
        .unwrap_err();

        assert_eq!(err, ExecutionEngineKernelError::UnsafeMetadata);
        let rendered = format!("{err:?}").to_ascii_lowercase();
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("raw prompt"));
    }

    #[test]
    fn wire_statuses_are_closed_and_terminal_predicates_are_stable() {
        assert_eq!(
            WorkflowExecutionStatus::from_wire("running"),
            Some(WorkflowExecutionStatus::Running)
        );
        assert_eq!(WorkflowExecutionStatus::from_wire("unknown"), None);
        assert!(WorkflowExecutionStatus::Completed.is_terminal());
        assert!(!WorkflowExecutionStatus::Running.is_terminal());
        assert_eq!(
            StepExecutionStatus::from_wire("retrying"),
            Some(StepExecutionStatus::Retrying)
        );
        assert!(StepExecutionStatus::TimedOut.is_terminal_failure());
    }

    #[test]
    fn retry_attempt_and_sla_timer_validate_bounds_without_wall_clock() {
        let retry = RetryAttempt::new(
            "ten_a",
            "run:execution:1",
            "step:approve",
            2,
            "error-class:retryable-http-503",
            "retry-policy:standard",
            vec!["workflow-execution:retry".to_owned()],
        )
        .unwrap();
        assert_eq!(retry.attempt, 2);

        assert_eq!(
            SlaTimer::new(
                "timer:approval:1",
                "ten_a",
                "run:execution:1",
                Some(0),
                10,
                9,
                vec!["workflow-execution:sla".to_owned()],
            )
            .unwrap_err(),
            ExecutionEngineKernelError::InvalidTimerDeadline
        );
    }

    #[test]
    fn port_traits_are_source_level_contracts_for_future_adapters() {
        #[derive(Default)]
        struct RecordingStore {
            saved_runs: Vec<WorkflowRun>,
        }

        impl WorkflowRunStore for RecordingStore {
            fn create_run(&mut self, run: WorkflowRun) -> Result<(), ExecutionStoreError> {
                self.saved_runs.push(run);
                Ok(())
            }

            fn load_run(
                &self,
                tenant_id: &str,
                run_id: &str,
            ) -> Result<Option<WorkflowRun>, ExecutionStoreError> {
                Ok(self
                    .saved_runs
                    .iter()
                    .find(|run| run.tenant_id == tenant_id && run.run_id == run_id)
                    .cloned())
            }

            fn update_run_status(
                &mut self,
                tenant_id: &str,
                run_id: &str,
                expected_version: u64,
                status: WorkflowExecutionStatus,
                evidence_ref: &str,
            ) -> Result<(), ExecutionStoreError> {
                let run = self
                    .saved_runs
                    .iter_mut()
                    .find(|run| run.tenant_id == tenant_id && run.run_id == run_id)
                    .ok_or_else(|| ExecutionStoreError::Unavailable {
                        evidence_ref: "store:missing".to_owned(),
                    })?;
                if run.version != expected_version {
                    return Err(ExecutionStoreError::Conflict {
                        expected_version,
                        observed_version: run.version,
                        evidence_ref: evidence_ref.to_owned(),
                    });
                }
                run.status = status;
                run.version += 1;
                Ok(())
            }

            fn save_step(&mut self, _step: StepExecution) -> Result<(), ExecutionStoreError> {
                Ok(())
            }
        }

        let mut store: Box<dyn WorkflowRunStore> = Box::new(RecordingStore::default());
        store.create_run(run()).unwrap();
        assert_eq!(
            store
                .load_run("ten_a", "run:execution:1")
                .unwrap()
                .unwrap()
                .status,
            WorkflowExecutionStatus::Pending
        );
        store
            .update_run_status(
                "ten_a",
                "run:execution:1",
                1,
                WorkflowExecutionStatus::Running,
                "store:update:running",
            )
            .unwrap();
        assert_eq!(
            store
                .load_run("ten_a", "run:execution:1")
                .unwrap()
                .unwrap()
                .version,
            2
        );
    }
}
