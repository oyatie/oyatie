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
pub enum StepObservationKind {
    LeaseGranted,
    Started,
    Heartbeat,
    AwaitingExternalSignal,
    LongRunning,
    Completed,
    Failed,
    TimedOut,
    Cancelled,
}

impl StepObservationKind {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::LeaseGranted => "lease-granted",
            Self::Started => "started",
            Self::Heartbeat => "heartbeat",
            Self::AwaitingExternalSignal => "awaiting-external-signal",
            Self::LongRunning => "long-running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::TimedOut => "timed-out",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_wire(value: &str) -> Option<Self> {
        match value {
            "lease-granted" => Some(Self::LeaseGranted),
            "started" => Some(Self::Started),
            "heartbeat" => Some(Self::Heartbeat),
            "awaiting-external-signal" => Some(Self::AwaitingExternalSignal),
            "long-running" => Some(Self::LongRunning),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "timed-out" => Some(Self::TimedOut),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum StepObservationSeverity {
    Info,
    Warning,
    Degraded,
    Critical,
}

impl StepObservationSeverity {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Degraded => "degraded",
            Self::Critical => "critical",
        }
    }

    pub fn from_wire(value: &str) -> Option<Self> {
        match value {
            "info" => Some(Self::Info),
            "warning" => Some(Self::Warning),
            "degraded" => Some(Self::Degraded),
            "critical" => Some(Self::Critical),
            _ => None,
        }
    }

    pub fn blocks_promotion(self) -> bool {
        matches!(self, Self::Degraded | Self::Critical)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExecutionEngineKernelError {
    InvalidAttempt,
    InvalidObservationWindow,
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepObservationEnvelope {
    pub resource_id: String,                // data_class: INTERNAL_ONLY
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub cell_id: String,                    // data_class: INTERNAL_ONLY
    pub region: String,                     // data_class: INTERNAL_ONLY
    pub data_class: String,                 // data_class: INTERNAL_ONLY
    pub policy_domain: String,              // data_class: INTERNAL_ONLY
    pub owner_ref: String,                  // data_class: INTERNAL_ONLY
    pub generation: u64,                    // data_class: INTERNAL_ONLY
    pub observed_generation: u64,           // data_class: INTERNAL_ONLY
    pub status_condition_refs: Vec<String>, // data_class: INTERNAL_ONLY
    pub audit_chain_ref: String,            // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepObservationEnvelopeInput<'a> {
    pub resource_id: &'a str,                // data_class: INTERNAL_ONLY
    pub tenant_id: &'a str,                  // data_class: INTERNAL_ONLY
    pub cell_id: &'a str,                    // data_class: INTERNAL_ONLY
    pub region: &'a str,                     // data_class: INTERNAL_ONLY
    pub data_class: &'a str,                 // data_class: INTERNAL_ONLY
    pub policy_domain: &'a str,              // data_class: INTERNAL_ONLY
    pub owner_ref: &'a str,                  // data_class: INTERNAL_ONLY
    pub generation: u64,                     // data_class: INTERNAL_ONLY
    pub observed_generation: u64,            // data_class: INTERNAL_ONLY
    pub status_condition_refs: Vec<String>,  // data_class: INTERNAL_ONLY
    pub audit_chain_ref: &'a str,             // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
}

impl StepObservationEnvelope {
    pub fn new(input: StepObservationEnvelopeInput<'_>) -> Result<Self, ExecutionEngineKernelError> {
        let StepObservationEnvelopeInput {
            resource_id,
            tenant_id,
            cell_id,
            region,
            data_class,
            policy_domain,
            owner_ref,
            generation,
            observed_generation,
            status_condition_refs,
            audit_chain_ref,
            created_at_epoch_seconds,
            updated_at_epoch_seconds,
        } = input;
        if generation == 0
            || observed_generation > generation
            || updated_at_epoch_seconds < created_at_epoch_seconds
        {
            return Err(ExecutionEngineKernelError::InvalidObservationWindow);
        }
        if !is_safe_ref(resource_id)
            || !is_safe_tenant(tenant_id)
            || !is_safe_ref(cell_id)
            || !is_safe_metadata(region)
            || !is_safe_metadata(data_class)
            || !is_safe_metadata(policy_domain)
            || !is_safe_ref(owner_ref)
            || !is_safe_ref(audit_chain_ref)
            || status_condition_refs.is_empty()
            || !status_condition_refs.iter().all(|value| is_safe_ref(value))
        {
            return Err(ExecutionEngineKernelError::UnsafeMetadata);
        }
        Ok(Self {
            resource_id: resource_id.to_owned(),
            tenant_id: tenant_id.to_owned(),
            cell_id: cell_id.to_owned(),
            region: region.to_owned(),
            data_class: data_class.to_owned(),
            policy_domain: policy_domain.to_owned(),
            owner_ref: owner_ref.to_owned(),
            generation,
            observed_generation,
            status_condition_refs: sorted_unique(status_condition_refs),
            audit_chain_ref: audit_chain_ref.to_owned(),
            created_at_epoch_seconds,
            updated_at_epoch_seconds,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepObservation {
    pub envelope: StepObservationEnvelope,
    pub run_id: String,                     // data_class: INTERNAL_ONLY
    pub step_id: String,                    // data_class: INTERNAL_ONLY
    pub step_index: u32,                    // data_class: INTERNAL_ONLY
    pub attempt: u32,                       // data_class: INTERNAL_ONLY
    pub observed_at_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
    pub stale_after_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
    pub status: StepExecutionStatus,        // data_class: PUBLIC
    pub kind: StepObservationKind,          // data_class: PUBLIC
    pub severity: StepObservationSeverity,  // data_class: PUBLIC
    pub condition_ref: String,              // data_class: INTERNAL_ONLY
    pub adapter_status_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepObservationInput<'a> {
    pub envelope: StepObservationEnvelope,       // data_class: INTERNAL_ONLY
    pub run_id: &'a str,                         // data_class: INTERNAL_ONLY
    pub step_id: &'a str,                        // data_class: INTERNAL_ONLY
    pub step_index: u32,                         // data_class: INTERNAL_ONLY
    pub attempt: u32,                            // data_class: INTERNAL_ONLY
    pub observed_at_epoch_seconds: u64,          // data_class: INTERNAL_ONLY
    pub stale_after_epoch_seconds: u64,           // data_class: INTERNAL_ONLY
    pub status: StepExecutionStatus,             // data_class: PUBLIC
    pub kind: StepObservationKind,               // data_class: PUBLIC
    pub severity: StepObservationSeverity,       // data_class: PUBLIC
    pub condition_ref: &'a str,                  // data_class: INTERNAL_ONLY
    pub adapter_status_ref: Option<&'a str>,      // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,              // data_class: INTERNAL_ONLY
}

impl StepObservation {
    pub fn new(input: StepObservationInput<'_>) -> Result<Self, ExecutionEngineKernelError> {
        let StepObservationInput {
            envelope,
            run_id,
            step_id,
            step_index,
            attempt,
            observed_at_epoch_seconds,
            stale_after_epoch_seconds,
            status,
            kind,
            severity,
            condition_ref,
            adapter_status_ref,
            evidence_refs,
        } = input;
        if attempt == 0 {
            return Err(ExecutionEngineKernelError::InvalidAttempt);
        }
        if stale_after_epoch_seconds <= observed_at_epoch_seconds {
            return Err(ExecutionEngineKernelError::InvalidObservationWindow);
        }
        if !is_safe_ref(run_id)
            || !is_safe_ref(step_id)
            || !is_safe_ref(condition_ref)
            || adapter_status_ref.is_some_and(|value| !is_safe_ref(value))
            || !evidence_refs.iter().all(|value| is_safe_ref(value))
        {
            return Err(ExecutionEngineKernelError::UnsafeMetadata);
        }
        Ok(Self {
            envelope,
            run_id: run_id.to_owned(),
            step_id: step_id.to_owned(),
            step_index,
            attempt,
            observed_at_epoch_seconds,
            stale_after_epoch_seconds,
            status,
            kind,
            severity,
            condition_ref: condition_ref.to_owned(),
            adapter_status_ref: adapter_status_ref.map(str::to_owned),
            evidence_refs: sorted_unique(evidence_refs),
        })
    }

    pub fn is_stale_at(&self, now_epoch_seconds: u64) -> bool {
        now_epoch_seconds >= self.stale_after_epoch_seconds && !self.status.is_terminal()
    }

    pub fn blocks_promotion_at(&self, now_epoch_seconds: u64) -> bool {
        self.severity.blocks_promotion() || self.is_stale_at(now_epoch_seconds)
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
pub trait StepObservationStore {
    fn record_observation(
        &mut self,
        observation: StepObservation,
    ) -> Result<(), ExecutionStoreError>;

    fn observations_for_step(
        &self,
        tenant_id: &str,
        run_id: &str,
        step_index: u32,
    ) -> Result<Vec<StepObservation>, ExecutionStoreError>;
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
    fn observation_envelope() -> StepObservationEnvelope {
        StepObservationEnvelope::new(StepObservationEnvelopeInput {
            resource_id: "observation:deploy-ring:1",
            tenant_id: "ten_a",
            cell_id: "cell:use1:a",
            region: "use1",
            data_class: "INTERNAL_ONLY",
            policy_domain: "delivery-fabric",
            owner_ref: "owner:platform-delivery-fabric",
            generation: 2,
            observed_generation: 2,
            status_condition_refs: vec![
                "condition:accepted".to_owned(),
                "condition:waiting-for-slo-window".to_owned(),
            ],
            audit_chain_ref: "audit-chain:entry-1",
            created_at_epoch_seconds: 100,
            updated_at_epoch_seconds: 101,
        })
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
    fn metadata_ref_validation_allows_letter_s_and_rejects_whitespace() {
        assert!(is_safe_ref("condition:status-ok"));
        assert!(is_safe_ref("evidence:slo-window-open"));
        assert!(!is_safe_ref("condition:has space"));
        assert!(!is_safe_ref("condition"));
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
    fn step_observation_models_long_steps_without_adapter_authority() {
        let observation = StepObservation::new(StepObservationInput {
            envelope: observation_envelope(),
            run_id: "run:execution:1",
            step_id: "step:deploy-ring",
            step_index: 3,
            attempt: 1,
            observed_at_epoch_seconds: 100,
            stale_after_epoch_seconds: 160,
            status: StepExecutionStatus::Running,
            kind: StepObservationKind::LongRunning,
            severity: StepObservationSeverity::Warning,
            condition_ref: "condition:waiting-for-slo-window",
            adapter_status_ref: Some("adapter-status:github-check-123"),
            evidence_refs: vec![
                "evidence:step-heartbeat".to_owned(),
                "evidence:step-heartbeat".to_owned(),
                "evidence:slo-window-open".to_owned(),
            ],
        })
        .unwrap();

        assert_eq!(observation.kind.as_wire(), "long-running");
        assert_eq!(
            StepObservationKind::from_wire("awaiting-external-signal"),
            Some(StepObservationKind::AwaitingExternalSignal)
        );
        assert_eq!(observation.severity.as_wire(), "warning");
        assert!(!observation.blocks_promotion_at(120));
        assert!(observation.blocks_promotion_at(160));
        assert_eq!(
            observation.evidence_refs,
            vec![
                "evidence:slo-window-open".to_owned(),
                "evidence:step-heartbeat".to_owned()
            ]
        );
    }

    #[test]
    fn step_observation_rejects_stale_windows_and_raw_adapter_payloads() {
        assert_eq!(
            StepObservation::new(StepObservationInput {
                envelope: observation_envelope(),
                run_id: "run:execution:1",
                step_id: "step:deploy-ring",
                step_index: 3,
                attempt: 1,
                observed_at_epoch_seconds: 100,
                stale_after_epoch_seconds: 100,
                status: StepExecutionStatus::Running,
                kind: StepObservationKind::Heartbeat,
                severity: StepObservationSeverity::Info,
                condition_ref: "condition:heartbeat",
                adapter_status_ref: None,
                evidence_refs: vec!["evidence:heartbeat".to_owned()],
            })
            .unwrap_err(),
            ExecutionEngineKernelError::InvalidObservationWindow
        );

        let err = StepObservation::new(StepObservationInput {
            envelope: observation_envelope(),
            run_id: "run:execution:1",
            step_id: "step:deploy-ring",
            step_index: 3,
            attempt: 1,
            observed_at_epoch_seconds: 100,
            stale_after_epoch_seconds: 160,
            status: StepExecutionStatus::Running,
            kind: StepObservationKind::Heartbeat,
            severity: StepObservationSeverity::Info,
            condition_ref: "condition:raw output Authorization: Bearer sk-test",
            adapter_status_ref: Some("adapter-status:github"),
            evidence_refs: vec!["evidence:heartbeat".to_owned()],
        })
        .unwrap_err();

        assert_eq!(err, ExecutionEngineKernelError::UnsafeMetadata);
        let rendered = format!("{err:?}").to_ascii_lowercase();
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("raw output"));
    }

    #[test]
    fn observation_store_trait_is_adapter_neutral_control_plane_contract() {
        #[derive(Default)]
        struct RecordingObservationStore {
            observations: Vec<StepObservation>,
        }

        impl StepObservationStore for RecordingObservationStore {
            fn record_observation(
                &mut self,
                observation: StepObservation,
            ) -> Result<(), ExecutionStoreError> {
                self.observations.push(observation);
                Ok(())
            }

            fn observations_for_step(
                &self,
                tenant_id: &str,
                run_id: &str,
                step_index: u32,
            ) -> Result<Vec<StepObservation>, ExecutionStoreError> {
                Ok(self
                    .observations
                    .iter()
                    .filter(|observation| {
                        observation.envelope.tenant_id == tenant_id
                            && observation.run_id == run_id
                            && observation.step_index == step_index
                    })
                    .cloned()
                    .collect())
            }
        }

        let mut store = RecordingObservationStore::default();
        store
            .record_observation(
                StepObservation::new(StepObservationInput {
                    envelope: observation_envelope(),
                    run_id: "run:execution:1",
                    step_id: "step:deploy-ring",
                    step_index: 3,
                    attempt: 1,
                    observed_at_epoch_seconds: 100,
                    stale_after_epoch_seconds: 160,
                    status: StepExecutionStatus::Running,
                    kind: StepObservationKind::LongRunning,
                    severity: StepObservationSeverity::Degraded,
                    condition_ref: "condition:slo-window-missed",
                    adapter_status_ref: Some("adapter-status:github-check-123"),
                    evidence_refs: vec!["evidence:slo-window-missed".to_owned()],
                })
                .unwrap(),
            )
            .unwrap();

        let observations = store
            .observations_for_step("ten_a", "run:execution:1", 3)
            .unwrap();
        assert_eq!(observations.len(), 1);
        assert!(observations[0].blocks_promotion_at(120));
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
