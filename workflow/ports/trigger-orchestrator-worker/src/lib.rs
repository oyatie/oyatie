//! Workflow-engine trigger-orchestrator worker foundation.
//!
//! This crate provides a deterministic source-level worker seam for future
//! trigger-orchestrator job processing. It validates leased job metadata, honors
//! not-before scheduling and lease expiry before API side effects, invokes the
//! trigger-orchestrator API boundary in-process, preserves API idempotent
//! replay/conflict behavior, maps accepted/deferred/suppressed/denied outcomes
//! into metadata-only worker receipts, and models capped deterministic retry
//! scheduling for retryable API unavailability. It performs no durable queue
//! polling, Valkey lease I/O, scheduler execution, webhook serving, HMAC
//! verification, event-bus consumption, run creation, network I/O, filesystem
//! access, wall-clock reads, random/UUID generation, durable idempotency storage,
//! event-bus publishing, signing, Kubernetes calls, cloud deployment, or tenant
//! workload scheduling.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use workflow_trigger_orchestrator_api::{
    TRIGGER_ORCHESTRATOR_API_DECLARED_VERSION, TRIGGER_ORCHESTRATOR_API_METHOD,
    TRIGGER_ORCHESTRATOR_API_ROUTE, TRIGGER_ORCHESTRATOR_API_SURFACE,
    TRIGGER_ORCHESTRATOR_CLOUDEVENTS_SPECVERSION, TriggerOrchestratorApiAuthorization,
    TriggerOrchestratorApiBoundaryContext, TriggerOrchestratorApiError,
    TriggerOrchestratorApiErrorCode, TriggerOrchestratorApiEventDto,
    TriggerOrchestratorApiPrincipal, TriggerOrchestratorApiProblemDetails,
    TriggerOrchestratorApiRequest, TriggerOrchestratorApiScheduleDto, TriggerOrchestratorApiStatus,
    TriggerOrchestratorApiSuccessResponse, TriggerOrchestratorApiTriggerBody,
    TriggerOrchestratorApiWebhookDto, WorkflowTriggerOrchestratorApi,
};

pub const TRIGGER_ORCHESTRATOR_WORKER_SURFACE: &str = "workflow-engine.trigger-orchestrator.worker";
pub const TRIGGER_ORCHESTRATOR_WORKER_MAX_ATTEMPTS: u32 = 10;
pub const TRIGGER_ORCHESTRATOR_WORKER_BASE_BACKOFF_SECONDS: u64 = 30;
pub const TRIGGER_ORCHESTRATOR_WORKER_MAX_BACKOFF_SECONDS: u64 = 900;
pub const TRIGGER_ORCHESTRATOR_WORKER_DEFAULT_RESUME_LIMIT: usize = 32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorWorkerJob {
    pub job_id: String,                         // data_class: INTERNAL_ONLY
    pub lease_id: String,                       // data_class: INTERNAL_ONLY
    pub worker_ref: String,                     // data_class: INTERNAL_ONLY
    pub attempt_id: String,                     // data_class: INTERNAL_ONLY
    pub attempt_number: u32,                    // data_class: INTERNAL_ONLY
    pub max_attempts: u32,                      // data_class: INTERNAL_ONLY
    pub now_epoch_seconds: u64,                 // data_class: INTERNAL_ONLY
    pub not_before_epoch_seconds: u64,          // data_class: INTERNAL_ONLY
    pub lease_expires_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
    pub request: TriggerOrchestratorApiRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TriggerOrchestratorWorkerStatus {
    Accepted,
    Deferred,
    Denied,
    DispatchPlanned,
    RetryExhausted,
    RetryScheduled,
    Suppressed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TriggerOrchestratorWorkerDenialKind {
    ApiDenied,
    IdempotencyConflict,
    InvalidJob,
    LeaseExpired,
    RetryExhausted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TriggerOrchestratorWorkerEventKind {
    ApiApplied,
    JobAccepted,
    JobDeferred,
    JobDenied,
    LeaseExpired,
    RetryExhausted,
    RetryScheduled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorWorkerReceipt {
    pub job_id: String,                          // data_class: INTERNAL_ONLY
    pub lease_id: String,                        // data_class: INTERNAL_ONLY
    pub worker_ref: String,                      // data_class: INTERNAL_ONLY
    pub attempt_id: String,                      // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,                       // data_class: INTERNAL_ONLY
    pub trigger_id: String,                      // data_class: INTERNAL_ONLY
    pub workflow_spec_id: String,                // data_class: INTERNAL_ONLY
    pub status: TriggerOrchestratorWorkerStatus, // data_class: PUBLIC
    pub denial_kind: Option<TriggerOrchestratorWorkerDenialKind>, // data_class: INTERNAL_ONLY
    pub api_status: Option<TriggerOrchestratorApiStatus>, // data_class: PUBLIC
    pub usecase_status: Option<String>,          // data_class: PUBLIC
    pub dispatch_required: bool,                 // data_class: PUBLIC
    pub start_run_command_ref: Option<String>,   // data_class: INTERNAL_ONLY
    pub retry_delay_seconds: Option<u64>,        // data_class: INTERNAL_ONLY
    pub next_attempt_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,              // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TriggerOrchestratorWorkerReceiptParts {
    status: TriggerOrchestratorWorkerStatus,
    denial_kind: Option<TriggerOrchestratorWorkerDenialKind>,
    api_status: Option<TriggerOrchestratorApiStatus>,
    usecase_status: Option<String>,
    dispatch_required: bool,
    start_run_command_ref: Option<String>,
    retry_delay_seconds: Option<u64>,
    next_attempt_epoch_seconds: Option<u64>,
    evidence_refs: Vec<String>,
}

impl TriggerOrchestratorWorkerReceiptParts {
    fn new(status: TriggerOrchestratorWorkerStatus, evidence_refs: Vec<String>) -> Self {
        Self {
            status,
            denial_kind: None,
            api_status: None,
            usecase_status: None,
            dispatch_required: false,
            start_run_command_ref: None,
            retry_delay_seconds: None,
            next_attempt_epoch_seconds: None,
            evidence_refs,
        }
    }

    fn with_denial_kind(mut self, denial_kind: TriggerOrchestratorWorkerDenialKind) -> Self {
        self.denial_kind = Some(denial_kind);
        self
    }

    fn with_api_status(mut self, api_status: TriggerOrchestratorApiStatus) -> Self {
        self.api_status = Some(api_status);
        self
    }

    fn with_usecase_status(mut self, usecase_status: impl Into<String>) -> Self {
        self.usecase_status = Some(usecase_status.into());
        self
    }

    fn with_dispatch_plan(
        mut self,
        dispatch_required: bool,
        start_run_command_ref: Option<String>,
    ) -> Self {
        self.dispatch_required = dispatch_required;
        self.start_run_command_ref = start_run_command_ref;
        self
    }

    fn with_retry(mut self, delay_seconds: u64, next_attempt_epoch_seconds: u64) -> Self {
        self.retry_delay_seconds = Some(delay_seconds);
        self.next_attempt_epoch_seconds = Some(next_attempt_epoch_seconds);
        self
    }

    fn with_next_attempt_epoch_seconds(mut self, next_attempt_epoch_seconds: u64) -> Self {
        self.next_attempt_epoch_seconds = Some(next_attempt_epoch_seconds);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorWorkerEvent {
    pub kind: TriggerOrchestratorWorkerEventKind, // data_class: INTERNAL_ONLY
    pub job_id: String,                           // data_class: INTERNAL_ONLY
    pub lease_id: String,                         // data_class: INTERNAL_ONLY
    pub worker_ref: String,                       // data_class: INTERNAL_ONLY
    pub attempt_id: String,                       // data_class: INTERNAL_ONLY
    pub tenant_id: String,                        // data_class: INTERNAL_ONLY
    pub trigger_id: String,                       // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorWorkerResumeCandidate {
    pub tenant_id: String,           // data_class: INTERNAL_ONLY
    pub trigger_id: String,          // data_class: INTERNAL_ONLY
    pub workflow_spec_id: String,    // data_class: INTERNAL_ONLY
    pub due_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
    pub resume_priority: u32,        // data_class: INTERNAL_ONLY
    pub resume_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorWorkerResumePlan {
    pub accepted: Vec<TriggerOrchestratorWorkerResumeCandidate>, // data_class: INTERNAL_ONLY
    pub deferred: Vec<TriggerOrchestratorWorkerResumeCandidate>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                              // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorWorkerResumeThrottle {
    pub max_resumes_per_tick: usize, // data_class: INTERNAL_ONLY
}

impl Default for TriggerOrchestratorWorkerResumeThrottle {
    fn default() -> Self {
        Self {
            max_resumes_per_tick: TRIGGER_ORCHESTRATOR_WORKER_DEFAULT_RESUME_LIMIT,
        }
    }
}

#[derive(Default)]
pub struct TriggerOrchestratorWorker {
    api: WorkflowTriggerOrchestratorApi,
    events: Vec<TriggerOrchestratorWorkerEvent>,
    api_apply_count: usize,
}

impl TriggerOrchestratorWorker {
    pub fn new(api: WorkflowTriggerOrchestratorApi) -> Self {
        Self {
            api,
            events: Vec::new(),
            api_apply_count: 0,
        }
    }

    pub fn run_once(
        &mut self,
        job: TriggerOrchestratorWorkerJob,
    ) -> TriggerOrchestratorWorkerReceipt {
        if let Err(evidence_ref) = validate_job(&job) {
            let receipt = receipt_from_job(
                &job,
                TriggerOrchestratorWorkerReceiptParts::new(
                    TriggerOrchestratorWorkerStatus::Denied,
                    vec![evidence_ref],
                )
                .with_denial_kind(TriggerOrchestratorWorkerDenialKind::InvalidJob),
            );
            self.record_event(
                TriggerOrchestratorWorkerEventKind::JobDenied,
                &job,
                receipt.evidence_refs.clone(),
            );
            return receipt;
        }

        if job.now_epoch_seconds < job.not_before_epoch_seconds {
            let receipt = receipt_from_job(
                &job,
                TriggerOrchestratorWorkerReceiptParts::new(
                    TriggerOrchestratorWorkerStatus::Deferred,
                    vec!["workflow-trigger-worker:deferred:not-before".to_owned()],
                )
                .with_next_attempt_epoch_seconds(job.not_before_epoch_seconds),
            );
            self.record_event(
                TriggerOrchestratorWorkerEventKind::JobDeferred,
                &job,
                receipt.evidence_refs.clone(),
            );
            return receipt;
        }

        if job.lease_expires_epoch_seconds <= job.now_epoch_seconds {
            let receipt = receipt_from_job(
                &job,
                TriggerOrchestratorWorkerReceiptParts::new(
                    TriggerOrchestratorWorkerStatus::Denied,
                    vec!["workflow-trigger-worker:lease-expired".to_owned()],
                )
                .with_denial_kind(TriggerOrchestratorWorkerDenialKind::LeaseExpired),
            );
            self.record_event(
                TriggerOrchestratorWorkerEventKind::LeaseExpired,
                &job,
                receipt.evidence_refs.clone(),
            );
            return receipt;
        }

        if job.attempt_number > job.max_attempts {
            let receipt = retry_exhausted_receipt(
                &job,
                vec!["workflow-trigger-worker:attempts-exhausted-before-api".to_owned()],
            );
            self.record_event(
                TriggerOrchestratorWorkerEventKind::RetryExhausted,
                &job,
                receipt.evidence_refs.clone(),
            );
            return receipt;
        }

        self.record_event(
            TriggerOrchestratorWorkerEventKind::JobAccepted,
            &job,
            sorted_unique(vec![
                job.job_id.clone(),
                job.lease_id.clone(),
                job.attempt_id.clone(),
                TRIGGER_ORCHESTRATOR_WORKER_SURFACE.to_owned(),
                "workflow-trigger-worker:job-accepted".to_owned(),
            ]),
        );

        self.api_apply_count += 1;
        match self.api.apply_trigger(job.request.clone()) {
            Ok(success) => {
                let receipt = receipt_from_success(&job, success);
                self.record_event(
                    TriggerOrchestratorWorkerEventKind::ApiApplied,
                    &job,
                    receipt.evidence_refs.clone(),
                );
                receipt
            }
            Err(error) => {
                let receipt = receipt_from_api_error(&job, &error);
                self.record_terminal_error_event(&job, &receipt);
                receipt
            }
        }
    }

    pub fn api_apply_count(&self) -> usize {
        self.api_apply_count
    }

    pub fn events(&self) -> &[TriggerOrchestratorWorkerEvent] {
        &self.events
    }

    fn record_terminal_error_event(
        &mut self,
        job: &TriggerOrchestratorWorkerJob,
        receipt: &TriggerOrchestratorWorkerReceipt,
    ) {
        let kind = match receipt.status {
            TriggerOrchestratorWorkerStatus::RetryExhausted => {
                TriggerOrchestratorWorkerEventKind::RetryExhausted
            }
            TriggerOrchestratorWorkerStatus::RetryScheduled => {
                TriggerOrchestratorWorkerEventKind::RetryScheduled
            }
            _ => TriggerOrchestratorWorkerEventKind::JobDenied,
        };
        self.record_event(kind, job, receipt.evidence_refs.clone());
    }

    fn record_event(
        &mut self,
        kind: TriggerOrchestratorWorkerEventKind,
        job: &TriggerOrchestratorWorkerJob,
        evidence_refs: Vec<String>,
    ) {
        self.events.push(TriggerOrchestratorWorkerEvent {
            kind,
            job_id: safe_ref_or_redacted(&job.job_id, "redacted:trigger-worker-job"),
            lease_id: safe_ref_or_redacted(&job.lease_id, "redacted:trigger-worker-lease"),
            worker_ref: safe_ref_or_redacted(&job.worker_ref, "redacted:trigger-worker"),
            attempt_id: safe_ref_or_redacted(&job.attempt_id, "redacted:trigger-worker-attempt"),
            tenant_id: safe_tenant_or_redacted(&job.request.boundary.tenant_id),
            trigger_id: safe_ref_or_redacted(
                &job.request.body.trigger_id,
                "redacted:trigger-orchestrator-trigger",
            ),
            evidence_refs: sorted_unique(evidence_refs),
        });
    }
}

pub fn plan_resume_candidates(
    mut candidates: Vec<TriggerOrchestratorWorkerResumeCandidate>,
    throttle: TriggerOrchestratorWorkerResumeThrottle,
) -> TriggerOrchestratorWorkerResumePlan {
    candidates.retain(valid_resume_candidate);
    candidates.sort_by(|left, right| {
        left.resume_priority
            .cmp(&right.resume_priority)
            .then(left.due_epoch_seconds.cmp(&right.due_epoch_seconds))
            .then(left.tenant_id.cmp(&right.tenant_id))
            .then(left.trigger_id.cmp(&right.trigger_id))
    });
    let limit = throttle.max_resumes_per_tick;
    let accepted: Vec<_> = candidates.iter().take(limit).cloned().collect();
    let deferred: Vec<_> = candidates.into_iter().skip(limit).collect();
    let accepted_len = accepted.len();
    let deferred_len = deferred.len();
    TriggerOrchestratorWorkerResumePlan {
        accepted,
        deferred,
        evidence_refs: sorted_unique(vec![
            TRIGGER_ORCHESTRATOR_WORKER_SURFACE.to_owned(),
            format!("workflow-trigger-worker:resume-accepted:{accepted_len}"),
            format!("workflow-trigger-worker:resume-deferred:{deferred_len}"),
        ]),
    }
}

fn validate_job(job: &TriggerOrchestratorWorkerJob) -> Result<(), String> {
    if !is_safe_ref(&job.job_id)
        || !is_safe_ref(&job.lease_id)
        || !is_safe_ref(&job.worker_ref)
        || !is_safe_ref(&job.attempt_id)
    {
        return Err("workflow-trigger-worker:invalid-job-metadata".to_owned());
    }
    if job.attempt_number == 0 || job.max_attempts == 0 {
        return Err("workflow-trigger-worker:invalid-attempt-bounds".to_owned());
    }
    if job.max_attempts > TRIGGER_ORCHESTRATOR_WORKER_MAX_ATTEMPTS {
        return Err("workflow-trigger-worker:max-attempts-exceeds-worker-cap".to_owned());
    }
    if job.not_before_epoch_seconds >= job.lease_expires_epoch_seconds {
        return Err("workflow-trigger-worker:not-before-after-lease-expiry".to_owned());
    }
    Ok(())
}

fn receipt_from_success(
    job: &TriggerOrchestratorWorkerJob,
    success: TriggerOrchestratorApiSuccessResponse,
) -> TriggerOrchestratorWorkerReceipt {
    let usecase_status = success.trigger.usecase_status.clone();
    let status = match usecase_status.as_str() {
        "deferred" => TriggerOrchestratorWorkerStatus::Deferred,
        "suppressed" => TriggerOrchestratorWorkerStatus::Suppressed,
        _ if success.trigger.dispatch_required => TriggerOrchestratorWorkerStatus::DispatchPlanned,
        _ => TriggerOrchestratorWorkerStatus::Accepted,
    };
    let mut evidence_refs = success.evidence_refs.clone();
    evidence_refs.extend(success.non_claim_refs.clone());
    evidence_refs.push(TRIGGER_ORCHESTRATOR_WORKER_SURFACE.to_owned());
    evidence_refs.push("workflow-trigger-worker:api-success".to_owned());
    receipt_from_job(
        job,
        TriggerOrchestratorWorkerReceiptParts::new(status, evidence_refs)
            .with_api_status(success.status)
            .with_usecase_status(usecase_status)
            .with_dispatch_plan(
                success.trigger.dispatch_required,
                success.trigger.start_run_command_ref.clone(),
            ),
    )
}

fn receipt_from_api_error(
    job: &TriggerOrchestratorWorkerJob,
    error: &TriggerOrchestratorApiError,
) -> TriggerOrchestratorWorkerReceipt {
    if error.status() == TriggerOrchestratorApiStatus::ServiceUnavailable {
        return retryable_error_receipt(job, error.problem());
    }
    let problem = error.problem();
    let denial_kind = if error.code() == TriggerOrchestratorApiErrorCode::IdempotencyKeyReused {
        TriggerOrchestratorWorkerDenialKind::IdempotencyConflict
    } else {
        TriggerOrchestratorWorkerDenialKind::ApiDenied
    };
    let mut evidence_refs = problem.evidence_refs.clone();
    evidence_refs.push(TRIGGER_ORCHESTRATOR_WORKER_SURFACE.to_owned());
    evidence_refs.push(problem.code.clone());
    receipt_from_job(
        job,
        TriggerOrchestratorWorkerReceiptParts::new(
            TriggerOrchestratorWorkerStatus::Denied,
            evidence_refs,
        )
        .with_denial_kind(denial_kind)
        .with_api_status(error.status()),
    )
}

fn retryable_error_receipt(
    job: &TriggerOrchestratorWorkerJob,
    problem: TriggerOrchestratorApiProblemDetails,
) -> TriggerOrchestratorWorkerReceipt {
    if job.attempt_number >= job.max_attempts {
        let mut evidence_refs = problem.evidence_refs.clone();
        evidence_refs.push(problem.code);
        evidence_refs.push("workflow-trigger-worker:retry-exhausted".to_owned());
        return retry_exhausted_receipt(job, evidence_refs);
    }
    let retry_delay_seconds = retry_delay_seconds(job);
    let next_attempt = job.now_epoch_seconds.saturating_add(retry_delay_seconds);
    let mut evidence_refs = problem.evidence_refs;
    evidence_refs.push(problem.code);
    evidence_refs.push(TRIGGER_ORCHESTRATOR_WORKER_SURFACE.to_owned());
    evidence_refs.push("workflow-trigger-worker:retry-scheduled".to_owned());
    receipt_from_job(
        job,
        TriggerOrchestratorWorkerReceiptParts::new(
            TriggerOrchestratorWorkerStatus::RetryScheduled,
            evidence_refs,
        )
        .with_retry(retry_delay_seconds, next_attempt)
        .with_api_status(TriggerOrchestratorApiStatus::ServiceUnavailable),
    )
}

fn retry_exhausted_receipt(
    job: &TriggerOrchestratorWorkerJob,
    evidence_refs: Vec<String>,
) -> TriggerOrchestratorWorkerReceipt {
    receipt_from_job(
        job,
        TriggerOrchestratorWorkerReceiptParts::new(
            TriggerOrchestratorWorkerStatus::RetryExhausted,
            evidence_refs,
        )
        .with_denial_kind(TriggerOrchestratorWorkerDenialKind::RetryExhausted),
    )
}

fn receipt_from_job(
    job: &TriggerOrchestratorWorkerJob,
    parts: TriggerOrchestratorWorkerReceiptParts,
) -> TriggerOrchestratorWorkerReceipt {
    TriggerOrchestratorWorkerReceipt {
        job_id: safe_ref_or_redacted(&job.job_id, "redacted:trigger-worker-job"),
        lease_id: safe_ref_or_redacted(&job.lease_id, "redacted:trigger-worker-lease"),
        worker_ref: safe_ref_or_redacted(&job.worker_ref, "redacted:trigger-worker"),
        attempt_id: safe_ref_or_redacted(&job.attempt_id, "redacted:trigger-worker-attempt"),
        idempotency_key: safe_ref_or_redacted(
            &job.request.boundary.idempotency_key,
            "redacted:trigger-worker-idempotency",
        ),
        tenant_id: safe_tenant_or_redacted(&job.request.boundary.tenant_id),
        trigger_id: safe_ref_or_redacted(
            &job.request.body.trigger_id,
            "redacted:trigger-orchestrator-trigger",
        ),
        workflow_spec_id: safe_ref_or_redacted(
            &job.request.body.workflow_spec_id,
            "redacted:trigger-orchestrator-workflow-spec",
        ),
        status: parts.status,
        denial_kind: parts.denial_kind,
        api_status: parts.api_status,
        usecase_status: parts.usecase_status,
        dispatch_required: parts.dispatch_required,
        start_run_command_ref: parts.start_run_command_ref,
        retry_delay_seconds: parts.retry_delay_seconds,
        next_attempt_epoch_seconds: parts.next_attempt_epoch_seconds,
        evidence_refs: sorted_unique(parts.evidence_refs),
    }
}

fn retry_delay_seconds(job: &TriggerOrchestratorWorkerJob) -> u64 {
    let exponent = job.attempt_number.saturating_sub(1).min(5);
    let exponential =
        TRIGGER_ORCHESTRATOR_WORKER_BASE_BACKOFF_SECONDS.saturating_mul(1 << exponent);
    let jitter = stable_jitter_seconds(&job.job_id, job.attempt_number);
    exponential
        .saturating_add(jitter)
        .min(TRIGGER_ORCHESTRATOR_WORKER_MAX_BACKOFF_SECONDS)
}

fn stable_jitter_seconds(job_id: &str, attempt_number: u32) -> u64 {
    let mut acc = u64::from(attempt_number);
    for byte in job_id.bytes() {
        acc = acc.wrapping_mul(31).wrapping_add(u64::from(byte));
    }
    acc % TRIGGER_ORCHESTRATOR_WORKER_BASE_BACKOFF_SECONDS
}

fn valid_resume_candidate(candidate: &TriggerOrchestratorWorkerResumeCandidate) -> bool {
    is_safe_tenant(&candidate.tenant_id)
        && is_safe_ref(&candidate.trigger_id)
        && is_safe_ref(&candidate.workflow_spec_id)
        && is_safe_ref(&candidate.resume_evidence_ref)
}

fn is_safe_tenant(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && value == trimmed
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
        && !contains_unsafe_debug_material(value)
}

fn is_safe_ref(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && value == trimmed
        && value.contains(':')
        && !value.chars().any(char::is_whitespace)
        && !contains_unsafe_debug_material(value)
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

fn contains_unsafe_debug_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw output")
        || lower.contains("raw model")
        || lower.contains("payload")
        || lower.contains("write an email")
        || lower.contains("customer message")
        || lower.contains("model answer")
        || lower.contains("sk-")
        || lower.contains("bearer")
        || lower.contains("authorization:")
        || lower.contains("api_key=")
        || lower.contains("openai_api_key")
        || lower.contains("secret=")
        || lower.contains("private key")
        || lower.contains("-----begin")
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty() && !contains_unsafe_debug_material(value));
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn worker_statuses_and_defaults_are_stable() {
        let statuses = [
            TriggerOrchestratorWorkerStatus::Accepted,
            TriggerOrchestratorWorkerStatus::Deferred,
            TriggerOrchestratorWorkerStatus::Denied,
            TriggerOrchestratorWorkerStatus::DispatchPlanned,
            TriggerOrchestratorWorkerStatus::RetryExhausted,
            TriggerOrchestratorWorkerStatus::RetryScheduled,
            TriggerOrchestratorWorkerStatus::Suppressed,
        ];
        let unique: BTreeSet<_> = statuses.iter().copied().collect();
        assert_eq!(unique.len(), statuses.len());
        assert_eq!(TRIGGER_ORCHESTRATOR_WORKER_MAX_ATTEMPTS, 10);
        assert_eq!(
            TriggerOrchestratorWorkerResumeThrottle::default().max_resumes_per_tick,
            TRIGGER_ORCHESTRATOR_WORKER_DEFAULT_RESUME_LIMIT
        );
    }

    #[test]
    fn accepted_scheduler_trigger_invokes_api_and_records_dispatch_plan_metadata_only() {
        let mut worker = TriggerOrchestratorWorker::default();
        let receipt = worker.run_once(valid_job("idem:worker:1", "scheduler", "cron"));

        assert_eq!(
            receipt.status,
            TriggerOrchestratorWorkerStatus::DispatchPlanned
        );
        assert_eq!(
            receipt.api_status,
            Some(TriggerOrchestratorApiStatus::Accepted)
        );
        assert_eq!(receipt.usecase_status.as_deref(), Some("accepted"));
        assert!(receipt.dispatch_required);
        assert!(receipt.start_run_command_ref.is_some());
        assert_eq!(worker.api_apply_count(), 1);
        assert_eq!(worker.events().len(), 2);
        assert_eq!(
            worker.events()[0].kind,
            TriggerOrchestratorWorkerEventKind::JobAccepted
        );
        assert_eq!(
            worker.events()[1].kind,
            TriggerOrchestratorWorkerEventKind::ApiApplied
        );
        assert!(!format!("{receipt:?}").contains("payload"));
    }

    #[test]
    fn invalid_not_before_and_expired_lease_jobs_never_invoke_api() {
        let mut worker = TriggerOrchestratorWorker::default();
        let mut invalid = valid_job("idem:worker:invalid", "scheduler", "cron");
        invalid.job_id = "job:raw prompt bearer sk-test payload".to_owned();
        let invalid_receipt = worker.run_once(invalid);
        assert_eq!(
            invalid_receipt.status,
            TriggerOrchestratorWorkerStatus::Denied
        );
        assert_eq!(
            invalid_receipt.denial_kind,
            Some(TriggerOrchestratorWorkerDenialKind::InvalidJob)
        );
        assert_eq!(worker.api_apply_count(), 0);
        assert!(!format!("{invalid_receipt:?}").contains("sk-test"));

        let mut deferred = valid_job("idem:worker:deferred", "scheduler", "cron");
        deferred.not_before_epoch_seconds = 200;
        deferred.lease_expires_epoch_seconds = 260;
        let deferred_receipt = worker.run_once(deferred);
        assert_eq!(
            deferred_receipt.status,
            TriggerOrchestratorWorkerStatus::Deferred
        );
        assert_eq!(deferred_receipt.next_attempt_epoch_seconds, Some(200));
        assert_eq!(worker.api_apply_count(), 0);

        let mut expired = valid_job("idem:worker:expired", "scheduler", "cron");
        expired.lease_expires_epoch_seconds = expired.now_epoch_seconds;
        let expired_receipt = worker.run_once(expired);
        assert_eq!(
            expired_receipt.status,
            TriggerOrchestratorWorkerStatus::Denied
        );
        assert_eq!(
            expired_receipt.denial_kind,
            Some(TriggerOrchestratorWorkerDenialKind::LeaseExpired)
        );
        assert_eq!(worker.api_apply_count(), 0);
    }

    #[test]
    fn api_domain_denial_maps_to_worker_denial_without_raw_echo() {
        let mut worker = TriggerOrchestratorWorker::default();
        let mut job = valid_job("idem:worker:domain-denied", "scheduler", "cron");
        job.request.body.scheduler_evidence_ref = None;

        let receipt = worker.run_once(job);

        assert_eq!(receipt.status, TriggerOrchestratorWorkerStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(TriggerOrchestratorWorkerDenialKind::ApiDenied)
        );
        assert_eq!(
            receipt.api_status,
            Some(TriggerOrchestratorApiStatus::Forbidden)
        );
        assert_eq!(worker.api_apply_count(), 1);
        let rendered = format!("{receipt:?}");
        assert!(rendered.contains(TriggerOrchestratorApiErrorCode::DomainDenied.as_str()));
        assert!(!rendered.contains("raw prompt"));
        assert!(!rendered.contains("payload"));
    }

    #[test]
    fn idempotent_replay_and_conflict_are_preserved_through_worker_api() {
        let mut worker = TriggerOrchestratorWorker::default();
        let job = valid_job("idem:worker:replay", "scheduler", "cron");
        let first = worker.run_once(job.clone());
        let second = worker.run_once(job);

        assert_eq!(first, second);
        assert_eq!(worker.api_apply_count(), 2);

        let mut drifted = valid_job("idem:worker:replay", "scheduler", "cron");
        drifted.request.body.workflow_spec_id = "workflow:other".to_owned();
        let conflict = worker.run_once(drifted);
        assert_eq!(conflict.status, TriggerOrchestratorWorkerStatus::Denied);
        assert_eq!(
            conflict.denial_kind,
            Some(TriggerOrchestratorWorkerDenialKind::IdempotencyConflict)
        );
        assert_eq!(
            conflict.api_status,
            Some(TriggerOrchestratorApiStatus::Conflict)
        );
    }

    #[test]
    fn retryable_service_unavailable_schedules_capped_deterministic_backoff() {
        let mut job = valid_job("idem:worker:retry", "scheduler", "cron");
        job.attempt_number = 2;
        let error = TriggerOrchestratorApiError::Boundary {
            code: TriggerOrchestratorApiErrorCode::UsecaseUnavailable,
            status: TriggerOrchestratorApiStatus::ServiceUnavailable,
            evidence_ref: "api:evidence:temporarily-unavailable".to_owned(),
        };

        let scheduled = receipt_from_api_error(&job, &error);

        assert_eq!(
            scheduled.status,
            TriggerOrchestratorWorkerStatus::RetryScheduled
        );
        assert_eq!(
            scheduled.api_status,
            Some(TriggerOrchestratorApiStatus::ServiceUnavailable)
        );
        assert!(scheduled.retry_delay_seconds.unwrap() >= 60);
        assert!(scheduled.retry_delay_seconds.unwrap() <= 90);
        assert_eq!(
            scheduled.next_attempt_epoch_seconds,
            Some(job.now_epoch_seconds + scheduled.retry_delay_seconds.unwrap())
        );

        job.attempt_number = job.max_attempts;
        let exhausted = receipt_from_api_error(&job, &error);
        assert_eq!(
            exhausted.status,
            TriggerOrchestratorWorkerStatus::RetryExhausted
        );
        assert_eq!(
            exhausted.denial_kind,
            Some(TriggerOrchestratorWorkerDenialKind::RetryExhausted)
        );
    }

    #[test]
    fn deferred_suppressed_and_webhook_sources_remain_metadata_only() {
        let mut worker = TriggerOrchestratorWorker::default();

        let mut paused = valid_job("idem:worker:paused", "scheduler", "cron");
        paused.request.body.schedule.as_mut().unwrap().paused = true;
        paused
            .request
            .body
            .schedule
            .as_mut()
            .unwrap()
            .pause_reason_ref = Some("pause:maintenance".to_owned());
        let deferred = worker.run_once(paused);
        assert_eq!(deferred.status, TriggerOrchestratorWorkerStatus::Deferred);
        assert_eq!(deferred.usecase_status.as_deref(), Some("deferred"));
        assert!(!deferred.dispatch_required);

        let mut event = valid_job("idem:worker:event", "sibling-event-bus", "event-bus");
        event.request.body.schedule = None;
        event.request.body.event = Some(event_valid());
        event.request.body.scheduler_evidence_ref = None;
        event.request.body.event_contract_ref = Some("event-contract:cloudevents-v1".to_owned());
        event.request.body.replay_mode = true;
        let suppressed = worker.run_once(event);
        assert_eq!(
            suppressed.status,
            TriggerOrchestratorWorkerStatus::Suppressed
        );
        assert_eq!(suppressed.usecase_status.as_deref(), Some("suppressed"));
        assert!(!suppressed.dispatch_required);

        let mut webhook = valid_job("idem:worker:webhook", "studio-webhook", "webhook");
        webhook.request.body.schedule = None;
        webhook.request.body.webhook = Some(webhook_valid());
        webhook.request.body.scheduler_evidence_ref = None;
        webhook.request.body.webhook_auth_evidence_ref =
            Some("webhook-auth:hmac-nonce-bound".to_owned());
        let accepted = worker.run_once(webhook);
        assert_eq!(
            accepted.status,
            TriggerOrchestratorWorkerStatus::DispatchPlanned
        );
        assert!(
            accepted
                .evidence_refs
                .contains(&"no-webhook-server".to_owned())
        );
    }

    #[test]
    fn cold_start_resume_throttle_caps_orders_and_drops_unsafe_candidates() {
        let plan = plan_resume_candidates(
            vec![
                resume_candidate("trigger:z", 30, 3),
                resume_candidate("trigger:a", 20, 1),
                resume_candidate("trigger:b", 10, 1),
                TriggerOrchestratorWorkerResumeCandidate {
                    tenant_id: "ten_foundry".to_owned(),
                    trigger_id: "trigger:raw prompt payload".to_owned(),
                    workflow_spec_id: "workflow:invoice-approval".to_owned(),
                    due_epoch_seconds: 1,
                    resume_priority: 0,
                    resume_evidence_ref: "resume:evidence".to_owned(),
                },
            ],
            TriggerOrchestratorWorkerResumeThrottle {
                max_resumes_per_tick: 2,
            },
        );

        assert_eq!(plan.accepted.len(), 2);
        assert_eq!(plan.deferred.len(), 1);
        assert_eq!(plan.accepted[0].trigger_id, "trigger:b");
        assert_eq!(plan.accepted[1].trigger_id, "trigger:a");
        assert!(!format!("{plan:?}").contains("raw prompt"));
    }

    fn valid_job(idempotency_key: &str, source: &str, kind: &str) -> TriggerOrchestratorWorkerJob {
        TriggerOrchestratorWorkerJob {
            job_id: format!("job:trigger-worker:{idempotency_key}"),
            lease_id: format!("lease:trigger-worker:{idempotency_key}"),
            worker_ref: "worker:trigger-orchestrator:1".to_owned(),
            attempt_id: format!("attempt:trigger-worker:{idempotency_key}"),
            attempt_number: 1,
            max_attempts: 5,
            now_epoch_seconds: 100,
            not_before_epoch_seconds: 90,
            lease_expires_epoch_seconds: 160,
            request: authorized_request(idempotency_key, source, kind),
        }
    }

    fn authorized_request(
        idempotency_key: &str,
        source: &str,
        kind: &str,
    ) -> TriggerOrchestratorApiRequest {
        TriggerOrchestratorApiRequest {
            boundary: TriggerOrchestratorApiBoundaryContext {
                request_id: format!("request:trigger-api:{idempotency_key}"),
                tenant_id: "ten_foundry".to_owned(),
                idempotency_key: idempotency_key.to_owned(),
                trace_context_ref: "trace:trigger-api".to_owned(),
                oyatie_version: TRIGGER_ORCHESTRATOR_API_DECLARED_VERSION.to_owned(),
            },
            principal: TriggerOrchestratorApiPrincipal {
                tenant_id: "ten_foundry".to_owned(),
                principal_id: "principal:workflow-operator".to_owned(),
            },
            authorization: TriggerOrchestratorApiAuthorization {
                tenant_id: "ten_foundry".to_owned(),
                principal_id: "principal:workflow-operator".to_owned(),
                decision_id: "policy-decision:allow-trigger".to_owned(),
                evidence_ref: "policy-evidence:cedar-allow".to_owned(),
                policy_bundle_ref: "policy-bundle:trigger-v1".to_owned(),
                allowed_surfaces: vec![TRIGGER_ORCHESTRATOR_API_SURFACE.to_owned()],
            },
            method: TRIGGER_ORCHESTRATOR_API_METHOD.to_owned(),
            route: TRIGGER_ORCHESTRATOR_API_ROUTE.to_owned(),
            body: TriggerOrchestratorApiTriggerBody {
                source: source.to_owned(),
                trigger_kind: kind.to_owned(),
                trigger_id: "trigger:daily-invoice".to_owned(),
                workflow_spec_id: "workflow:invoice-approval".to_owned(),
                version_sha: "sha:abc123".to_owned(),
                active_cell_id: "cell:use1-a".to_owned(),
                trigger_lineage_ref: "lineage:trigger-parent".to_owned(),
                run_idempotency_key: "idem:trigger-run".to_owned(),
                authorization_surface_ref: "authz-surface:trigger-admission".to_owned(),
                source_evidence_ref: "source-evidence:trigger-admission".to_owned(),
                scheduler_evidence_ref: Some("scheduler:durable-clock-window".to_owned()),
                webhook_auth_evidence_ref: None,
                event_contract_ref: None,
                replay_epoch_ref: "replay-epoch:2026-05-25T000000Z".to_owned(),
                audit_chain_ref: "audit-chain:trigger-api".to_owned(),
                correlation_ref: "corr:trigger-api".to_owned(),
                idempotency_scope_ref: "idem-scope:tenant-trigger".to_owned(),
                dry_run_reason_ref: None,
                replay_mode: false,
                dry_run: false,
                schedule: Some(schedule_due()),
                webhook: None,
                event: None,
                evidence_refs: vec!["evidence:worker-unit-test".to_owned()],
            },
        }
    }

    fn schedule_due() -> TriggerOrchestratorApiScheduleDto {
        TriggerOrchestratorApiScheduleDto {
            cron_expr_ref: "cron:every-hour".to_owned(),
            timezone_ref: "tz:America-New_York".to_owned(),
            due_epoch_seconds: 1_750_000_000,
            observed_epoch_seconds: 1_750_000_008,
            catchup_window_seconds: 10,
            overlap_policy: "buffer-one".to_owned(),
            paused: false,
            pause_reason_ref: None,
            last_fired_epoch_seconds: Some(1_749_996_400),
        }
    }

    fn webhook_valid() -> TriggerOrchestratorApiWebhookDto {
        TriggerOrchestratorApiWebhookDto {
            endpoint_ref: "endpoint:webhook-invoice".to_owned(),
            signature_ref: "signature:webhook-headers".to_owned(),
            nonce_ref: "nonce:webhook-001".to_owned(),
            hmac_key_ref: "hmac-key:webhook-signing".to_owned(),
            received_epoch_seconds: 1_750_000_001,
            expires_epoch_seconds: 1_750_000_061,
        }
    }

    fn event_valid() -> TriggerOrchestratorApiEventDto {
        TriggerOrchestratorApiEventDto {
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

    fn resume_candidate(
        trigger_id: &str,
        due_epoch_seconds: u64,
        resume_priority: u32,
    ) -> TriggerOrchestratorWorkerResumeCandidate {
        TriggerOrchestratorWorkerResumeCandidate {
            tenant_id: "ten_foundry".to_owned(),
            trigger_id: trigger_id.to_owned(),
            workflow_spec_id: "workflow:invoice-approval".to_owned(),
            due_epoch_seconds,
            resume_priority,
            resume_evidence_ref: "resume:evidence".to_owned(),
        }
    }
}
