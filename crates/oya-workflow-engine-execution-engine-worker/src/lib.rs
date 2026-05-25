//! Workflow-engine execution-engine worker foundation.
//!
//! This crate provides a deterministic source-level worker seam for future
//! execution-engine job processing. It validates leased job metadata, honors
//! not-before scheduling and lease expiry before usecase or port side effects,
//! invokes the execution-engine usecase through abstract store/dispatcher/retry
//! and timer ports, classifies retryable failures with capped backoff, and
//! provides a cold-start resume throttle planner. It performs no durable queue
//! polling, Valkey lease I/O, Postgres I/O, activity execution, network I/O,
//! filesystem access, wall-clock reads, durable idempotency storage, event-bus
//! publishing, signing, or cloud-runtime scheduling.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use oya_workflow_engine_execution_engine_usecase::{
    ExecutionDispatchError, ExecutionDomainCommandKind, ExecutionDomainOrigin,
    ExecutionEngineDomainRequest, ExecutionEngineKernelError, ExecutionEngineUsecase,
    ExecutionEngineUsecaseInput, ExecutionStoreError, ExecutionUsecaseReceipt,
    ExecutionUsecaseStatus, RetryAttempt, RetryPolicyEvaluator, SlaTimer, SlaTimerStore,
    StepDispatcher, StepExecution, StepExecutionStatus, WorkflowExecutionStatus, WorkflowRun,
    WorkflowRunStore,
};

pub const EXECUTION_ENGINE_WORKER_SURFACE: &str = "workflow-engine.execution-engine.worker";
pub const EXECUTION_ENGINE_WORKER_MAX_ATTEMPTS: u32 = 10;
pub const EXECUTION_ENGINE_WORKER_BASE_BACKOFF_SECONDS: u64 = 30;
pub const EXECUTION_ENGINE_WORKER_MAX_BACKOFF_SECONDS: u64 = 900;
pub const EXECUTION_ENGINE_WORKER_DEFAULT_RESUME_LIMIT: usize = 16;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionWorkerJob {
    pub job_id: String,                     // data_class: INTERNAL_ONLY
    pub lease_id: String,                   // data_class: INTERNAL_ONLY
    pub worker_ref: String,                 // data_class: INTERNAL_ONLY
    pub attempt_id: String,                 // data_class: INTERNAL_ONLY
    pub attempt_number: u32,                // data_class: INTERNAL_ONLY
    pub max_attempts: u32,                  // data_class: INTERNAL_ONLY
    pub now_epoch_seconds: u64,             // data_class: INTERNAL_ONLY
    pub not_before_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
    pub lease_expires_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
    pub input: ExecutionEngineUsecaseInput, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExecutionWorkerStatus {
    Applied,
    Deferred,
    Denied,
    DispatchRecorded,
    Exhausted,
    RetryPlanned,
    RetryScheduled,
    TimerArmed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExecutionWorkerDenialKind {
    InvalidJob,
    LeaseExpired,
    RetryExhausted,
    UsecaseDenied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExecutionWorkerEventKind {
    JobAccepted,
    JobDeferred,
    JobDenied,
    LeaseExpired,
    RetryExhausted,
    RetryScheduled,
    UsecaseApplied,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionWorkerReceipt {
    pub job_id: String,                                 // data_class: INTERNAL_ONLY
    pub lease_id: String,                               // data_class: INTERNAL_ONLY
    pub worker_ref: String,                             // data_class: INTERNAL_ONLY
    pub attempt_id: String,                             // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                        // data_class: INTERNAL_ONLY
    pub tenant_id: String,                              // data_class: INTERNAL_ONLY
    pub run_id: String,                                 // data_class: INTERNAL_ONLY
    pub command: ExecutionDomainCommandKind,            // data_class: INTERNAL_ONLY
    pub status: ExecutionWorkerStatus,                  // data_class: PUBLIC
    pub denial_kind: Option<ExecutionWorkerDenialKind>, // data_class: INTERNAL_ONLY
    pub usecase_status: Option<ExecutionUsecaseStatus>, // data_class: INTERNAL_ONLY
    pub run_status: Option<WorkflowExecutionStatus>,    // data_class: PUBLIC
    pub step_status: Option<StepExecutionStatus>,       // data_class: PUBLIC
    pub retry_delay_seconds: Option<u64>,               // data_class: INTERNAL_ONLY
    pub next_attempt_epoch_seconds: Option<u64>,        // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExecutionWorkerReceiptParts {
    status: ExecutionWorkerStatus,
    denial_kind: Option<ExecutionWorkerDenialKind>,
    usecase_status: Option<ExecutionUsecaseStatus>,
    run_status: Option<WorkflowExecutionStatus>,
    step_status: Option<StepExecutionStatus>,
    retry_delay_seconds: Option<u64>,
    next_attempt_epoch_seconds: Option<u64>,
    evidence_refs: Vec<String>,
}

impl ExecutionWorkerReceiptParts {
    fn new(status: ExecutionWorkerStatus, evidence_refs: Vec<String>) -> Self {
        Self {
            status,
            denial_kind: None,
            usecase_status: None,
            run_status: None,
            step_status: None,
            retry_delay_seconds: None,
            next_attempt_epoch_seconds: None,
            evidence_refs,
        }
    }

    fn with_denial_kind(mut self, denial_kind: ExecutionWorkerDenialKind) -> Self {
        self.denial_kind = Some(denial_kind);
        self
    }

    fn with_usecase_status(mut self, usecase_status: ExecutionUsecaseStatus) -> Self {
        self.usecase_status = Some(usecase_status);
        self
    }

    fn with_run_status(mut self, run_status: Option<WorkflowExecutionStatus>) -> Self {
        self.run_status = run_status;
        self
    }

    fn with_step_status(mut self, step_status: Option<StepExecutionStatus>) -> Self {
        self.step_status = step_status;
        self
    }

    fn with_retry_delay_seconds(mut self, retry_delay_seconds: Option<u64>) -> Self {
        self.retry_delay_seconds = retry_delay_seconds;
        self
    }

    fn with_next_attempt_epoch_seconds(mut self, next_attempt_epoch_seconds: u64) -> Self {
        self.next_attempt_epoch_seconds = Some(next_attempt_epoch_seconds);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionWorkerEvent {
    pub kind: ExecutionWorkerEventKind, // data_class: INTERNAL_ONLY
    pub job_id: String,                 // data_class: INTERNAL_ONLY
    pub lease_id: String,               // data_class: INTERNAL_ONLY
    pub worker_ref: String,             // data_class: INTERNAL_ONLY
    pub attempt_id: String,             // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub run_id: String,                 // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionWorkerResumeCandidate {
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub run_id: String,                      // data_class: INTERNAL_ONLY
    pub run_status: WorkflowExecutionStatus, // data_class: PUBLIC
    pub observed_run_version: u64,           // data_class: INTERNAL_ONLY
    pub resume_priority: u32,                // data_class: INTERNAL_ONLY
    pub resume_evidence_ref: String,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionWorkerResumePlan {
    pub accepted: Vec<ExecutionWorkerResumeCandidate>, // data_class: INTERNAL_ONLY
    pub deferred: Vec<ExecutionWorkerResumeCandidate>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutionWorkerResumeThrottle {
    pub max_resumes_per_tick: usize, // data_class: INTERNAL_ONLY
}

impl Default for ExecutionWorkerResumeThrottle {
    fn default() -> Self {
        Self {
            max_resumes_per_tick: EXECUTION_ENGINE_WORKER_DEFAULT_RESUME_LIMIT,
        }
    }
}

#[derive(Default)]
pub struct ExecutionEngineWorker {
    usecase: ExecutionEngineUsecase,
    events: Vec<ExecutionWorkerEvent>,
}

impl ExecutionEngineWorker {
    pub fn run_once<S, D, R, T>(
        &mut self,
        store: &mut S,
        dispatcher: &mut D,
        retry_policy: &R,
        timers: &mut T,
        job: ExecutionWorkerJob,
    ) -> ExecutionWorkerReceipt
    where
        S: WorkflowRunStore,
        D: StepDispatcher,
        R: RetryPolicyEvaluator,
        T: SlaTimerStore,
    {
        if let Err(evidence_ref) = validate_job(&job) {
            let receipt = receipt_from_job(
                &job,
                ExecutionWorkerReceiptParts::new(ExecutionWorkerStatus::Denied, vec![evidence_ref])
                    .with_denial_kind(ExecutionWorkerDenialKind::InvalidJob),
            );
            self.record_event(
                ExecutionWorkerEventKind::JobDenied,
                &job,
                receipt.evidence_refs.clone(),
            );
            return receipt;
        }

        if job.now_epoch_seconds < job.not_before_epoch_seconds {
            let receipt = receipt_from_job(
                &job,
                ExecutionWorkerReceiptParts::new(
                    ExecutionWorkerStatus::Deferred,
                    vec!["workflow-execution-worker:deferred:not-before".to_owned()],
                )
                .with_next_attempt_epoch_seconds(job.not_before_epoch_seconds),
            );
            self.record_event(
                ExecutionWorkerEventKind::JobDeferred,
                &job,
                receipt.evidence_refs.clone(),
            );
            return receipt;
        }

        if job.lease_expires_epoch_seconds <= job.now_epoch_seconds {
            let receipt = receipt_from_job(
                &job,
                ExecutionWorkerReceiptParts::new(
                    ExecutionWorkerStatus::Denied,
                    vec!["workflow-execution-worker:lease-expired".to_owned()],
                )
                .with_denial_kind(ExecutionWorkerDenialKind::LeaseExpired),
            );
            self.record_event(
                ExecutionWorkerEventKind::LeaseExpired,
                &job,
                receipt.evidence_refs.clone(),
            );
            return receipt;
        }

        self.record_event(
            ExecutionWorkerEventKind::JobAccepted,
            &job,
            sorted_unique(vec![
                job.job_id.clone(),
                job.lease_id.clone(),
                job.attempt_id.clone(),
                "workflow-execution-worker:job-accepted".to_owned(),
            ]),
        );

        let receipt_value =
            self.usecase
                .apply(store, dispatcher, retry_policy, timers, job.input.clone());
        let worker_receipt = self.map_usecase_receipt(&job, receipt_value);
        self.record_terminal_event(&job, &worker_receipt);
        worker_receipt
    }

    pub fn events(&self) -> &[ExecutionWorkerEvent] {
        &self.events
    }

    pub fn recorded_event_count(&self) -> usize {
        self.events.len()
    }

    pub fn cached_usecase_receipt_count(&self) -> usize {
        self.usecase.cached_receipt_count()
    }

    pub fn plan_cold_start_resume(
        &self,
        throttle: ExecutionWorkerResumeThrottle,
        candidates: Vec<ExecutionWorkerResumeCandidate>,
    ) -> Result<ExecutionWorkerResumePlan, String> {
        plan_cold_start_resume(throttle, candidates)
    }

    fn map_usecase_receipt(
        &self,
        job: &ExecutionWorkerJob,
        receipt: ExecutionUsecaseReceipt,
    ) -> ExecutionWorkerReceipt {
        match receipt.status {
            ExecutionUsecaseStatus::Applied => receipt_from_job(
                job,
                ExecutionWorkerReceiptParts::new(
                    worker_status_for_command(receipt.command),
                    worker_evidence_refs(job, receipt.evidence_refs),
                )
                .with_usecase_status(receipt.status)
                .with_run_status(receipt.run_status)
                .with_step_status(receipt.step_status)
                .with_retry_delay_seconds(receipt.retry_delay_seconds),
            ),
            ExecutionUsecaseStatus::StoreUnavailable
            | ExecutionUsecaseStatus::DispatchUnavailable
            | ExecutionUsecaseStatus::TimerUnavailable => {
                retry_or_exhaust_receipt(job, receipt.status, receipt.evidence_refs)
            }
            ExecutionUsecaseStatus::DomainDenied
            | ExecutionUsecaseStatus::DispatchDenied
            | ExecutionUsecaseStatus::IdempotencyConflict
            | ExecutionUsecaseStatus::InvalidInput
            | ExecutionUsecaseStatus::RetryPolicyRejected
            | ExecutionUsecaseStatus::StoreConflict => receipt_from_job(
                job,
                ExecutionWorkerReceiptParts::new(
                    ExecutionWorkerStatus::Denied,
                    worker_evidence_refs(job, receipt.evidence_refs),
                )
                .with_denial_kind(ExecutionWorkerDenialKind::UsecaseDenied)
                .with_usecase_status(receipt.status)
                .with_run_status(receipt.run_status)
                .with_step_status(receipt.step_status)
                .with_retry_delay_seconds(receipt.retry_delay_seconds),
            ),
        }
    }

    fn record_terminal_event(
        &mut self,
        job: &ExecutionWorkerJob,
        receipt: &ExecutionWorkerReceipt,
    ) {
        let kind = match receipt.status {
            ExecutionWorkerStatus::Applied
            | ExecutionWorkerStatus::DispatchRecorded
            | ExecutionWorkerStatus::RetryPlanned
            | ExecutionWorkerStatus::TimerArmed => ExecutionWorkerEventKind::UsecaseApplied,
            ExecutionWorkerStatus::RetryScheduled => ExecutionWorkerEventKind::RetryScheduled,
            ExecutionWorkerStatus::Exhausted => ExecutionWorkerEventKind::RetryExhausted,
            ExecutionWorkerStatus::Deferred => ExecutionWorkerEventKind::JobDeferred,
            ExecutionWorkerStatus::Denied => ExecutionWorkerEventKind::JobDenied,
        };
        self.record_event(kind, job, receipt.evidence_refs.clone());
    }

    fn record_event(
        &mut self,
        kind: ExecutionWorkerEventKind,
        job: &ExecutionWorkerJob,
        evidence_refs: Vec<String>,
    ) {
        self.events.push(ExecutionWorkerEvent {
            kind,
            job_id: safe_ref(&job.job_id, "worker-job:redacted"),
            lease_id: safe_ref(&job.lease_id, "worker-lease:redacted"),
            worker_ref: safe_ref(&job.worker_ref, "worker:redacted"),
            attempt_id: safe_ref(&job.attempt_id, "worker-attempt:redacted"),
            tenant_id: safe_tenant(&job.input.domain_request.run.tenant_id),
            run_id: safe_ref(&job.input.domain_request.run.run_id, "run:redacted"),
            evidence_refs: sorted_unique(evidence_refs),
        });
    }
}

pub fn plan_cold_start_resume(
    throttle: ExecutionWorkerResumeThrottle,
    mut candidates: Vec<ExecutionWorkerResumeCandidate>,
) -> Result<ExecutionWorkerResumePlan, String> {
    if throttle.max_resumes_per_tick == 0 {
        return Err("workflow-execution-worker:resume-throttle-zero".to_owned());
    }
    if candidates
        .iter()
        .any(|candidate| !is_safe_resume_candidate(candidate))
    {
        return Err("workflow-execution-worker:resume-candidate-unsafe".to_owned());
    }
    candidates.retain(|candidate| {
        matches!(
            candidate.run_status,
            WorkflowExecutionStatus::Pending
                | WorkflowExecutionStatus::Running
                | WorkflowExecutionStatus::Paused
        )
    });
    candidates.sort_by(|left, right| {
        left.resume_priority
            .cmp(&right.resume_priority)
            .then(left.tenant_id.cmp(&right.tenant_id))
            .then(left.run_id.cmp(&right.run_id))
    });
    let split = throttle.max_resumes_per_tick.min(candidates.len());
    let deferred = candidates.split_off(split);
    Ok(ExecutionWorkerResumePlan {
        accepted: candidates,
        deferred,
        evidence_refs: sorted_unique(vec![
            "workflow-execution-worker:resume-plan".to_owned(),
            format!(
                "workflow-execution-worker:resume-limit:{}",
                throttle.max_resumes_per_tick
            ),
        ]),
    })
}

fn validate_job(job: &ExecutionWorkerJob) -> Result<(), String> {
    if !is_safe_ref(&job.job_id)
        || !is_safe_ref(&job.lease_id)
        || !is_safe_ref(&job.worker_ref)
        || !is_safe_ref(&job.attempt_id)
    {
        return Err("workflow-execution-worker:invalid-job-metadata".to_owned());
    }
    if job.attempt_number == 0
        || job.max_attempts == 0
        || job.attempt_number > job.max_attempts
        || job.max_attempts > EXECUTION_ENGINE_WORKER_MAX_ATTEMPTS
    {
        return Err("workflow-execution-worker:invalid-attempt-bounds".to_owned());
    }
    if !is_safe_usecase_input(&job.input) {
        return Err("workflow-execution-worker:unsafe-usecase-input".to_owned());
    }
    Ok(())
}

fn is_safe_usecase_input(input: &ExecutionEngineUsecaseInput) -> bool {
    is_safe_ref(&input.request_id)
        && is_safe_ref(&input.idempotency_key)
        && is_safe_ref(&input.trace_ref)
        && input.expected_run_version > 0
        && is_safe_domain_request(&input.domain_request)
}

fn is_safe_domain_request(request: &ExecutionEngineDomainRequest) -> bool {
    is_safe_tenant(&request.expected_tenant_id)
        && is_safe_ref(&request.expected_spec_id)
        && is_safe_ref(&request.expected_version_sha)
        && is_safe_ref(&request.expected_cell_id)
        && is_safe_ref(&request.policy_evidence_ref)
        && is_safe_ref(&request.spec_integrity_ref)
        && is_safe_ref(&request.replay_epoch_ref)
        && is_safe_ref(&request.scheduler_epoch_ref)
        && is_safe_run(&request.run)
        && request.step.as_ref().is_none_or(is_safe_step)
        && request.retry_attempt.as_ref().is_none_or(is_safe_retry)
        && request.sla_timer.as_ref().is_none_or(is_safe_timer)
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

fn is_safe_resume_candidate(candidate: &ExecutionWorkerResumeCandidate) -> bool {
    is_safe_tenant(&candidate.tenant_id)
        && is_safe_ref(&candidate.run_id)
        && candidate.observed_run_version > 0
        && is_safe_ref(&candidate.resume_evidence_ref)
}

fn worker_status_for_command(command: ExecutionDomainCommandKind) -> ExecutionWorkerStatus {
    match command {
        ExecutionDomainCommandKind::StartRun => ExecutionWorkerStatus::Applied,
        ExecutionDomainCommandKind::DispatchStep => ExecutionWorkerStatus::DispatchRecorded,
        ExecutionDomainCommandKind::ScheduleRetry => ExecutionWorkerStatus::RetryPlanned,
        ExecutionDomainCommandKind::ArmSlaTimer => ExecutionWorkerStatus::TimerArmed,
    }
}

fn retry_or_exhaust_receipt(
    job: &ExecutionWorkerJob,
    usecase_status: ExecutionUsecaseStatus,
    evidence_refs: Vec<String>,
) -> ExecutionWorkerReceipt {
    if job.attempt_number < job.max_attempts {
        let delay = retry_backoff_seconds(job.attempt_number);
        return receipt_from_job(
            job,
            ExecutionWorkerReceiptParts::new(
                ExecutionWorkerStatus::RetryScheduled,
                worker_evidence_refs(
                    job,
                    [
                        evidence_refs,
                        vec!["workflow-execution-worker:retry-scheduled".to_owned()],
                    ]
                    .concat(),
                ),
            )
            .with_usecase_status(usecase_status)
            .with_next_attempt_epoch_seconds(job.now_epoch_seconds.saturating_add(delay)),
        );
    }
    receipt_from_job(
        job,
        ExecutionWorkerReceiptParts::new(
            ExecutionWorkerStatus::Exhausted,
            worker_evidence_refs(
                job,
                [
                    evidence_refs,
                    vec!["workflow-execution-worker:retry-exhausted".to_owned()],
                ]
                .concat(),
            ),
        )
        .with_denial_kind(ExecutionWorkerDenialKind::RetryExhausted)
        .with_usecase_status(usecase_status),
    )
}

fn receipt_from_job(
    job: &ExecutionWorkerJob,
    parts: ExecutionWorkerReceiptParts,
) -> ExecutionWorkerReceipt {
    ExecutionWorkerReceipt {
        job_id: safe_ref(&job.job_id, "worker-job:redacted"),
        lease_id: safe_ref(&job.lease_id, "worker-lease:redacted"),
        worker_ref: safe_ref(&job.worker_ref, "worker:redacted"),
        attempt_id: safe_ref(&job.attempt_id, "worker-attempt:redacted"),
        idempotency_key: safe_ref(&job.input.idempotency_key, "idempotency:redacted"),
        tenant_id: safe_tenant(&job.input.domain_request.run.tenant_id),
        run_id: safe_ref(&job.input.domain_request.run.run_id, "run:redacted"),
        command: job.input.domain_request.command,
        status: parts.status,
        denial_kind: parts.denial_kind,
        usecase_status: parts.usecase_status,
        run_status: parts.run_status,
        step_status: parts.step_status,
        retry_delay_seconds: parts.retry_delay_seconds,
        next_attempt_epoch_seconds: parts.next_attempt_epoch_seconds,
        evidence_refs: sorted_unique(parts.evidence_refs),
    }
}

fn worker_evidence_refs(job: &ExecutionWorkerJob, mut refs: Vec<String>) -> Vec<String> {
    refs.extend([
        job.job_id.clone(),
        job.lease_id.clone(),
        job.worker_ref.clone(),
        job.attempt_id.clone(),
        "workflow-execution-worker:processed".to_owned(),
    ]);
    sorted_unique(refs)
}

fn retry_backoff_seconds(attempt_number: u32) -> u64 {
    let shift = attempt_number.saturating_sub(1).min(8);
    let multiplier = 1_u64 << shift;
    EXECUTION_ENGINE_WORKER_BASE_BACKOFF_SECONDS
        .saturating_mul(multiplier)
        .min(EXECUTION_ENGINE_WORKER_MAX_BACKOFF_SECONDS)
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

fn safe_tenant(value: &str) -> String {
    if is_safe_tenant(value) {
        value.to_owned()
    } else {
        "ten_redacted".to_owned()
    }
}

fn safe_ref(value: &str, fallback: &str) -> String {
    if is_safe_ref(value) {
        value.to_owned()
    } else {
        fallback.to_owned()
    }
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty() && is_safe_metadata(value));
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_workflow_engine_execution_engine_adapter::WorkflowExecutionMemoryAdapterBundle;

    #[derive(Default)]
    struct UnavailableDispatcher;

    impl StepDispatcher for UnavailableDispatcher {
        fn dispatch_step(
            &mut self,
            _tenant_id: &str,
            _run_id: &str,
            _step_index: u32,
            _evidence_ref: &str,
        ) -> Result<(), ExecutionDispatchError> {
            Err(ExecutionDispatchError::Unavailable {
                evidence_ref: "dispatcher:unavailable".to_owned(),
            })
        }
    }

    fn run_with_status(status: WorkflowExecutionStatus, version: u64) -> WorkflowRun {
        let mut run = WorkflowRun::new(
            "ten_a",
            "run:worker:1",
            "workflow-spec:invoice-approval",
            "sha256:spec-v1",
            "cell:use1:a",
            vec!["workflow-execution:worker-run".to_owned()],
        )
        .unwrap();
        run.status = status;
        run.version = version;
        run.current_step_index = Some(0);
        run
    }

    fn step_with_status(status: StepExecutionStatus, attempt: u32) -> StepExecution {
        let mut step = StepExecution::new(
            "ten_a",
            "run:worker:1",
            "step:approve",
            0,
            attempt,
            "idem:worker:step",
            vec!["workflow-execution:worker-step".to_owned()],
        )
        .unwrap();
        step.status = status;
        step
    }

    fn retry_attempt(attempt: u32) -> RetryAttempt {
        RetryAttempt::new(
            "ten_a",
            "run:worker:1",
            "step:approve",
            attempt,
            "error-class:http-503",
            "retry-policy:standard",
            vec!["workflow-execution:worker-retry".to_owned()],
        )
        .unwrap()
    }

    fn timer() -> SlaTimer {
        SlaTimer::new(
            "timer:worker:1",
            "ten_a",
            "run:worker:1",
            Some(0),
            100,
            200,
            vec!["workflow-execution:worker-timer".to_owned()],
        )
        .unwrap()
    }

    fn input(command: ExecutionDomainCommandKind) -> ExecutionEngineUsecaseInput {
        let (run_status, step, retry, sla_timer) = match command {
            ExecutionDomainCommandKind::StartRun => (
                WorkflowExecutionStatus::Pending,
                Some(step_with_status(StepExecutionStatus::Pending, 1)),
                None,
                None,
            ),
            ExecutionDomainCommandKind::DispatchStep => (
                WorkflowExecutionStatus::Running,
                Some(step_with_status(StepExecutionStatus::Pending, 1)),
                None,
                None,
            ),
            ExecutionDomainCommandKind::ScheduleRetry => (
                WorkflowExecutionStatus::Running,
                Some(step_with_status(StepExecutionStatus::Failed, 1)),
                Some(retry_attempt(2)),
                None,
            ),
            ExecutionDomainCommandKind::ArmSlaTimer => {
                (WorkflowExecutionStatus::Running, None, None, Some(timer()))
            }
        };
        ExecutionEngineUsecaseInput {
            request_id: format!("req:worker:{}", command.as_wire()),
            idempotency_key: format!("idem:worker:{}", command.as_wire()),
            trace_ref: format!("trace:worker:{}", command.as_wire()),
            expected_run_version: 7,
            domain_request: ExecutionEngineDomainRequest {
                run: run_with_status(run_status, 7),
                step,
                retry_attempt: retry,
                sla_timer,
                expected_tenant_id: "ten_a".to_owned(),
                expected_spec_id: "workflow-spec:invoice-approval".to_owned(),
                expected_version_sha: "sha256:spec-v1".to_owned(),
                expected_cell_id: "cell:use1:a".to_owned(),
                policy_evidence_ref: "cedar://workflow/execution/worker/allow".to_owned(),
                spec_integrity_ref: "spec-integrity:workflow:v1".to_owned(),
                replay_epoch_ref: "replay-epoch:worker:1".to_owned(),
                scheduler_epoch_ref: "scheduler-epoch:worker:1".to_owned(),
                command,
                origin: ExecutionDomainOrigin::WorkerScheduler,
            },
        }
    }

    fn job(command: ExecutionDomainCommandKind) -> ExecutionWorkerJob {
        ExecutionWorkerJob {
            job_id: format!("worker-job:{}", command.as_wire()),
            lease_id: format!("worker-lease:{}", command.as_wire()),
            worker_ref: "worker:execution:use1:a".to_owned(),
            attempt_id: format!("worker-attempt:{}:1", command.as_wire()),
            attempt_number: 1,
            max_attempts: 3,
            now_epoch_seconds: 100,
            not_before_epoch_seconds: 90,
            lease_expires_epoch_seconds: 200,
            input: input(command),
        }
    }

    fn seeded_bundle() -> WorkflowExecutionMemoryAdapterBundle {
        let mut bundle = WorkflowExecutionMemoryAdapterBundle::default();
        bundle
            .store
            .create_run(run_with_status(WorkflowExecutionStatus::Running, 7))
            .unwrap();
        bundle
    }

    #[test]
    fn worker_type_and_defaults_exist() {
        let worker = ExecutionEngineWorker::default();
        assert_eq!(worker.recorded_event_count(), 0);
        assert_eq!(
            EXECUTION_ENGINE_WORKER_SURFACE,
            "workflow-engine.execution-engine.worker"
        );
        assert_eq!(EXECUTION_ENGINE_WORKER_MAX_ATTEMPTS, 10);
    }

    #[test]
    fn not_before_and_expired_lease_defer_or_deny_before_ports() {
        let mut worker = ExecutionEngineWorker::default();
        let mut bundle = seeded_bundle();
        let mut future = job(ExecutionDomainCommandKind::DispatchStep);
        future.not_before_epoch_seconds = 150;

        let deferred = worker.run_once(
            &mut bundle.store,
            &mut bundle.dispatcher,
            &bundle.retry_policy,
            &mut bundle.timers,
            future,
        );
        assert_eq!(deferred.status, ExecutionWorkerStatus::Deferred);
        assert_eq!(bundle.dispatcher.recorded_actions().len(), 0);

        let mut expired = job(ExecutionDomainCommandKind::DispatchStep);
        expired.lease_expires_epoch_seconds = 100;
        let denied = worker.run_once(
            &mut bundle.store,
            &mut bundle.dispatcher,
            &bundle.retry_policy,
            &mut bundle.timers,
            expired,
        );
        assert_eq!(denied.status, ExecutionWorkerStatus::Denied);
        assert_eq!(
            denied.denial_kind,
            Some(ExecutionWorkerDenialKind::LeaseExpired)
        );
        assert_eq!(bundle.dispatcher.recorded_actions().len(), 0);
    }

    #[test]
    fn dispatch_job_invokes_usecase_and_records_metadata_only_events() {
        let mut worker = ExecutionEngineWorker::default();
        let mut bundle = seeded_bundle();

        let receipt = worker.run_once(
            &mut bundle.store,
            &mut bundle.dispatcher,
            &bundle.retry_policy,
            &mut bundle.timers,
            job(ExecutionDomainCommandKind::DispatchStep),
        );

        assert_eq!(receipt.status, ExecutionWorkerStatus::DispatchRecorded);
        assert_eq!(
            receipt.usecase_status,
            Some(ExecutionUsecaseStatus::Applied)
        );
        assert_eq!(receipt.step_status, Some(StepExecutionStatus::Leased));
        assert_eq!(worker.recorded_event_count(), 2);
        assert_eq!(
            worker.events()[0].kind,
            ExecutionWorkerEventKind::JobAccepted
        );
        assert!(bundle.dispatcher.recorded_actions().iter().any(|action| {
            action.kind.as_wire() == "dispatch-step" && action.run_id == "run:worker:1"
        }));
    }

    #[test]
    fn retry_and_timer_jobs_map_to_retry_planned_and_timer_armed() {
        let mut worker = ExecutionEngineWorker::default();
        let mut retry_bundle = seeded_bundle();

        let retry = worker.run_once(
            &mut retry_bundle.store,
            &mut retry_bundle.dispatcher,
            &retry_bundle.retry_policy,
            &mut retry_bundle.timers,
            job(ExecutionDomainCommandKind::ScheduleRetry),
        );
        assert_eq!(retry.status, ExecutionWorkerStatus::RetryPlanned);
        assert_eq!(retry.retry_delay_seconds, Some(10));
        assert_eq!(retry_bundle.dispatcher.recorded_actions().len(), 0);

        let mut timer_bundle = seeded_bundle();
        let timer_receipt = worker.run_once(
            &mut timer_bundle.store,
            &mut timer_bundle.dispatcher,
            &timer_bundle.retry_policy,
            &mut timer_bundle.timers,
            job(ExecutionDomainCommandKind::ArmSlaTimer),
        );
        assert_eq!(timer_receipt.status, ExecutionWorkerStatus::TimerArmed);
        assert_eq!(timer_bundle.timers.timer_count(), 1);
    }

    #[test]
    fn retryable_dispatch_unavailable_schedules_backoff_and_exhausts_at_max_attempt() {
        let mut worker = ExecutionEngineWorker::default();
        let mut bundle = seeded_bundle();
        let mut unavailable = UnavailableDispatcher;
        let retryable = worker.run_once(
            &mut bundle.store,
            &mut unavailable,
            &bundle.retry_policy,
            &mut bundle.timers,
            job(ExecutionDomainCommandKind::DispatchStep),
        );

        assert_eq!(retryable.status, ExecutionWorkerStatus::RetryScheduled);
        assert_eq!(retryable.next_attempt_epoch_seconds, Some(130));

        let mut exhausted_bundle = seeded_bundle();
        let mut exhausted_dispatcher = UnavailableDispatcher;
        let mut exhausted_job = job(ExecutionDomainCommandKind::DispatchStep);
        exhausted_job.attempt_id = "worker-attempt:dispatch-step:3".to_owned();
        exhausted_job.input.idempotency_key = "idem:worker:dispatch-step:exhausted".to_owned();
        exhausted_job.attempt_number = 3;
        exhausted_job.max_attempts = 3;
        let exhausted = worker.run_once(
            &mut exhausted_bundle.store,
            &mut exhausted_dispatcher,
            &exhausted_bundle.retry_policy,
            &mut exhausted_bundle.timers,
            exhausted_job,
        );
        assert_eq!(exhausted.status, ExecutionWorkerStatus::Exhausted);
        assert_eq!(
            exhausted.denial_kind,
            Some(ExecutionWorkerDenialKind::RetryExhausted)
        );
    }

    #[test]
    fn cold_start_resume_throttle_caps_and_orders_candidates_without_runtime_io() {
        let worker = ExecutionEngineWorker::default();
        let candidates = vec![
            resume_candidate("run:worker:3", 30),
            resume_candidate("run:worker:1", 10),
            resume_candidate("run:worker:2", 20),
        ];

        let plan = worker
            .plan_cold_start_resume(
                ExecutionWorkerResumeThrottle {
                    max_resumes_per_tick: 2,
                },
                candidates,
            )
            .unwrap();

        assert_eq!(plan.accepted.len(), 2);
        assert_eq!(plan.accepted[0].run_id, "run:worker:1");
        assert_eq!(plan.accepted[1].run_id, "run:worker:2");
        assert_eq!(plan.deferred.len(), 1);
        assert_eq!(plan.deferred[0].run_id, "run:worker:3");
    }

    #[test]
    fn invalid_raw_metadata_denies_without_echo_and_before_ports() {
        let mut worker = ExecutionEngineWorker::default();
        let mut bundle = seeded_bundle();
        let mut raw_job = job(ExecutionDomainCommandKind::DispatchStep);
        raw_job.worker_ref = "worker raw prompt Authorization: Bearer sk-test".to_owned();

        let receipt = worker.run_once(
            &mut bundle.store,
            &mut bundle.dispatcher,
            &bundle.retry_policy,
            &mut bundle.timers,
            raw_job,
        );

        assert_eq!(receipt.status, ExecutionWorkerStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(ExecutionWorkerDenialKind::InvalidJob)
        );
        assert_eq!(bundle.dispatcher.recorded_actions().len(), 0);
        let rendered = format!("{receipt:?}{:?}", worker.events()).to_ascii_lowercase();
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("raw prompt"));
    }

    fn resume_candidate(run_id: &str, priority: u32) -> ExecutionWorkerResumeCandidate {
        ExecutionWorkerResumeCandidate {
            tenant_id: "ten_a".to_owned(),
            run_id: run_id.to_owned(),
            run_status: WorkflowExecutionStatus::Running,
            observed_run_version: 7,
            resume_priority: priority,
            resume_evidence_ref: format!("resume-evidence:{priority}"),
        }
    }
}
