//! Intelligence eval worker foundation.
//!
//! This crate provides a deterministic source-level worker seam for future eval
//! job execution. It validates queued job metadata, runs the eval usecase, and
//! hands evaluated receipts to the metadata-only eval adapter. It performs no
//! queue I/O, network I/O, hosted runner calls, dataset fetches, filesystem
//! access, durable idempotency, durable audit-chain emission, or cloud runtime
//! scheduling.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use intelligence_eval_adapter::{
    EvalRunnerAdapterConfig, EvalRunnerAdapterConfigError, EvalRunnerDispatchFailure,
    EvalRunnerDispatchRequest, EvalRunnerDispatchStatus, EvalRunnerHttpMethod,
    EvalRunnerRequestEnvelope, EvalRunnerStatus, EvalRunnerTransportMode, IntelligenceEvalAdapter,
};
pub use intelligence_eval_usecase::{
    DomainEvalSetRequest, EvalCaseKind, EvalCaseOutcome, EvalCaseResult, EvalPolicyDecision,
    EvalSet, EvalSetStatus, EvalSetThresholds, EvalUsecaseDenialKind, EvalUsecaseInput,
    EvalUsecaseReceipt, EvalUsecaseStatus, IntelligenceEvalUsecase,
};

const MAX_WORKER_ATTEMPTS: u32 = 10;
const BASE_RETRY_BACKOFF_SECONDS: u64 = 30;
const MAX_RETRY_BACKOFF_SECONDS: u64 = 900;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalWorkerJob {
    pub job_id: String,                // data_class: INTERNAL_ONLY
    pub lease_id: String,              // data_class: INTERNAL_ONLY
    pub attempt_id: String,            // data_class: INTERNAL_ONLY
    pub attempt_number: u32,           // data_class: INTERNAL_ONLY
    pub max_attempts: u32,             // data_class: INTERNAL_ONLY
    pub now_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
    pub not_before_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub input: EvalUsecaseInput,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvalWorkerStatus {
    Deferred,
    Denied,
    Exhausted,
    RetryScheduled,
    RunnerAccepted,
    RunnerCompleted,
    RunnerQueued,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvalWorkerDenialKind {
    EvalUsecaseDenied,
    InvalidJob,
    RetryExhausted,
    RunnerDenied,
    RunnerInvalidRequest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalWorkerReceipt {
    pub job_id: String,                                  // data_class: INTERNAL_ONLY
    pub attempt_id: String,                              // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                         // data_class: INTERNAL_ONLY
    pub tenant_id: String,                               // data_class: INTERNAL_ONLY
    pub eval_surface: String,                            // data_class: INTERNAL_ONLY
    pub eval_set_id: String,                             // data_class: INTERNAL_ONLY
    pub status: EvalWorkerStatus,                        // data_class: PUBLIC
    pub denial_kind: Option<EvalWorkerDenialKind>,       // data_class: INTERNAL_ONLY
    pub runner_status: Option<EvalRunnerDispatchStatus>, // data_class: INTERNAL_ONLY
    pub runner_evidence_ref: Option<String>,             // data_class: INTERNAL_ONLY
    pub next_attempt_epoch_seconds: Option<u64>,         // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvalWorkerEventKind {
    EvalDenied,
    JobAccepted,
    JobDenied,
    RetryExhausted,
    RetryScheduled,
    RunnerAccepted,
    RunnerCompleted,
    RunnerQueued,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalWorkerEvent {
    pub kind: EvalWorkerEventKind,  // data_class: INTERNAL_ONLY
    pub job_id: String,             // data_class: INTERNAL_ONLY
    pub attempt_id: String,         // data_class: INTERNAL_ONLY
    pub idempotency_key: String,    // data_class: INTERNAL_ONLY
    pub eval_set_id: String,        // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>, // data_class: INTERNAL_ONLY
}

pub struct IntelligenceEvalWorker {
    eval_usecase: IntelligenceEvalUsecase,
    adapter: IntelligenceEvalAdapter,
    events: Vec<EvalWorkerEvent>,
}

impl IntelligenceEvalWorker {
    pub fn new(adapter: IntelligenceEvalAdapter) -> Self {
        Self {
            eval_usecase: IntelligenceEvalUsecase::default(),
            adapter,
            events: Vec::new(),
        }
    }

    pub fn run_once(&mut self, job: EvalWorkerJob) -> EvalWorkerReceipt {
        if let Err(evidence_ref) = validate_job(&job) {
            return receipt_from_job(
                &job,
                EvalWorkerStatus::Denied,
                Some(EvalWorkerDenialKind::InvalidJob),
                None,
                None,
                None,
                vec![evidence_ref],
            );
        }

        if job.now_epoch_seconds < job.not_before_epoch_seconds {
            return receipt_from_job(
                &job,
                EvalWorkerStatus::Deferred,
                None,
                None,
                None,
                Some(job.not_before_epoch_seconds),
                vec!["eval-worker:deferred:not-before".to_owned()],
            );
        }

        self.record_event(
            EvalWorkerEventKind::JobAccepted,
            &job,
            vec![job.input.request.request_evidence_ref.clone()],
        );

        let eval_receipt = self.eval_usecase.evaluate(job.input.clone());
        if eval_receipt.status != EvalUsecaseStatus::Evaluated {
            let receipt = receipt_from_eval_denial(&job, &eval_receipt);
            self.record_event(
                EvalWorkerEventKind::EvalDenied,
                &job,
                receipt.evidence_refs.clone(),
            );
            return receipt;
        }

        match self.adapter.dispatch(EvalRunnerDispatchRequest {
            idempotency_key: job.input.idempotency_key.clone(),
            domain_request: job.input.request.clone(),
            usecase_receipt: eval_receipt,
        }) {
            Ok(runner_receipt) => {
                let (status, event_kind) = match runner_receipt.status {
                    EvalRunnerDispatchStatus::Accepted => (
                        EvalWorkerStatus::RunnerAccepted,
                        EvalWorkerEventKind::RunnerAccepted,
                    ),
                    EvalRunnerDispatchStatus::Queued => (
                        EvalWorkerStatus::RunnerQueued,
                        EvalWorkerEventKind::RunnerQueued,
                    ),
                    EvalRunnerDispatchStatus::Completed => (
                        EvalWorkerStatus::RunnerCompleted,
                        EvalWorkerEventKind::RunnerCompleted,
                    ),
                };
                let receipt = receipt_from_job(
                    &job,
                    status,
                    None,
                    Some(runner_receipt.status),
                    Some(runner_receipt.evidence_ref),
                    None,
                    worker_success_evidence_refs(&job),
                );
                self.record_event(event_kind, &job, receipt.evidence_refs.clone());
                receipt
            }
            Err(failure) => self.receipt_from_runner_failure(&job, failure),
        }
    }

    pub fn events(&self) -> &[EvalWorkerEvent] {
        &self.events
    }

    pub fn eval_usecase_cached_receipt_count(&self) -> usize {
        self.eval_usecase.cached_receipt_count()
    }

    pub fn adapter_last_envelope(&self) -> Option<&EvalRunnerRequestEnvelope> {
        self.adapter.last_envelope()
    }

    fn receipt_from_runner_failure(
        &mut self,
        job: &EvalWorkerJob,
        failure: EvalRunnerDispatchFailure,
    ) -> EvalWorkerReceipt {
        if is_retryable_runner_failure(&failure.reason) {
            if job.attempt_number < job.max_attempts {
                let next_attempt = job
                    .now_epoch_seconds
                    .saturating_add(retry_backoff_seconds(job.attempt_number));
                let receipt = receipt_from_job(
                    job,
                    EvalWorkerStatus::RetryScheduled,
                    None,
                    None,
                    Some(failure.evidence_ref),
                    Some(next_attempt),
                    vec!["eval-worker:runner:retry-scheduled".to_owned()],
                );
                self.record_event(
                    EvalWorkerEventKind::RetryScheduled,
                    job,
                    receipt.evidence_refs.clone(),
                );
                return receipt;
            }
            let receipt = receipt_from_job(
                job,
                EvalWorkerStatus::Exhausted,
                Some(EvalWorkerDenialKind::RetryExhausted),
                None,
                Some(failure.evidence_ref),
                None,
                vec!["eval-worker:runner:retry-exhausted".to_owned()],
            );
            self.record_event(
                EvalWorkerEventKind::RetryExhausted,
                job,
                receipt.evidence_refs.clone(),
            );
            return receipt;
        }

        let denial_kind = if failure.reason == "eval-runner:invalid_request" {
            EvalWorkerDenialKind::RunnerInvalidRequest
        } else {
            EvalWorkerDenialKind::RunnerDenied
        };
        let receipt = receipt_from_job(
            job,
            EvalWorkerStatus::Denied,
            Some(denial_kind),
            None,
            Some(failure.evidence_ref),
            None,
            vec!["eval-worker:runner:denied".to_owned()],
        );
        self.record_event(
            EvalWorkerEventKind::JobDenied,
            job,
            receipt.evidence_refs.clone(),
        );
        receipt
    }

    fn record_event(
        &mut self,
        kind: EvalWorkerEventKind,
        job: &EvalWorkerJob,
        evidence_refs: Vec<String>,
    ) {
        self.events.push(EvalWorkerEvent {
            kind,
            job_id: safe_metadata(&job.job_id, "redacted-invalid-job-id"),
            attempt_id: safe_metadata(&job.attempt_id, "redacted-invalid-attempt-id"),
            idempotency_key: safe_metadata(
                &job.input.idempotency_key,
                "redacted-invalid-idempotency-key",
            ),
            eval_set_id: safe_ref(
                &job.input.request.eval_set.eval_set_id,
                "redacted-invalid-eval_set-id",
            ),
            evidence_refs: sorted_unique(evidence_refs),
        });
    }
}

fn validate_job(job: &EvalWorkerJob) -> Result<(), String> {
    require_metadata(&job.job_id, "validation:eval-worker-job-id")?;
    require_metadata(&job.lease_id, "validation:eval-worker-lease-id")?;
    require_metadata(&job.attempt_id, "validation:eval-worker-attempt-id")?;
    if job.attempt_number == 0
        || job.max_attempts == 0
        || job.max_attempts > MAX_WORKER_ATTEMPTS
        || job.attempt_number > job.max_attempts
    {
        return Err("validation:eval-worker-attempt-bounds".to_owned());
    }
    validate_input(&job.input)
}

fn validate_input(input: &EvalUsecaseInput) -> Result<(), String> {
    require_metadata(
        &input.idempotency_key,
        "validation:eval-worker-idempotency-key",
    )?;
    let request = &input.request;
    let eval_set = &request.eval_set;
    require_opaque(&request.tenant_id, "validation:eval-worker-tenant")?;
    require_opaque(&request.principal_id, "validation:eval-worker-principal")?;
    require_opaque(&request.eval_surface, "validation:eval-worker-surface")?;
    require_opaque(
        &request.request_evidence_ref,
        "validation:eval-worker-request-evidence",
    )?;
    require_opaque(
        &request.trace_context_ref,
        "validation:eval-worker-trace-context",
    )?;
    require_opaque(
        &request.policy_decision_ref,
        "validation:eval-worker-policy-decision",
    )?;
    require_opaque(
        &request.policy_decision.eval_registry_snapshot_ref,
        "validation:eval-worker-eval-registry-snapshot",
    )?;
    require_opaque(&eval_set.eval_set_id, "validation:eval-worker-eval_set-id")?;
    require_opaque(&eval_set.model_ref, "validation:eval-worker-model-ref")?;
    require_opaque(
        &eval_set.dataset_snapshot_ref,
        "validation:eval-worker-dataset-snapshot",
    )?;
    require_opaque(
        &eval_set.route_evidence_ref,
        "validation:eval-worker-route-evidence",
    )?;
    require_opaque(
        &eval_set.guardrail_evidence_ref,
        "validation:eval-worker-guardrail-evidence",
    )?;
    if eval_set.cases.is_empty() {
        return Err("validation:eval-worker-case-metadata".to_owned());
    }
    for case in &eval_set.cases {
        require_metadata(&case.case_id, "validation:eval-worker-case-id")?;
        require_opaque(
            &case.evaluator_evidence_ref,
            "validation:eval-worker-case-evidence",
        )?;
    }
    Ok(())
}

fn receipt_from_eval_denial(
    job: &EvalWorkerJob,
    eval_receipt: &EvalUsecaseReceipt,
) -> EvalWorkerReceipt {
    receipt_from_job(
        job,
        EvalWorkerStatus::Denied,
        Some(EvalWorkerDenialKind::EvalUsecaseDenied),
        None,
        None,
        None,
        sorted_unique(
            [
                eval_receipt.evidence_refs.clone(),
                vec!["eval-worker:eval-usecase-denied".to_owned()],
            ]
            .concat(),
        ),
    )
}

fn receipt_from_job(
    job: &EvalWorkerJob,
    status: EvalWorkerStatus,
    denial_kind: Option<EvalWorkerDenialKind>,
    runner_status: Option<EvalRunnerDispatchStatus>,
    runner_evidence_ref: Option<String>,
    next_attempt_epoch_seconds: Option<u64>,
    evidence_refs: Vec<String>,
) -> EvalWorkerReceipt {
    EvalWorkerReceipt {
        job_id: safe_metadata(&job.job_id, "redacted-invalid-job-id"),
        attempt_id: safe_metadata(&job.attempt_id, "redacted-invalid-attempt-id"),
        idempotency_key: safe_metadata(
            &job.input.idempotency_key,
            "redacted-invalid-idempotency-key",
        ),
        tenant_id: safe_ref(&job.input.request.tenant_id, "redacted-invalid-tenant-id"),
        eval_surface: safe_ref(
            &job.input.request.eval_surface,
            "redacted-invalid-eval-surface",
        ),
        eval_set_id: safe_ref(
            &job.input.request.eval_set.eval_set_id,
            "redacted-invalid-eval_set-id",
        ),
        status,
        denial_kind,
        runner_status,
        runner_evidence_ref: runner_evidence_ref
            .map(|value| safe_ref(&value, "eval-worker:redacted-invalid-runner-evidence-ref")),
        next_attempt_epoch_seconds,
        evidence_refs: sorted_unique(evidence_refs),
    }
}

fn worker_success_evidence_refs(job: &EvalWorkerJob) -> Vec<String> {
    sorted_unique(vec![
        job.input.request.request_evidence_ref.clone(),
        job.input.request.trace_context_ref.clone(),
        job.input.request.policy_decision_ref.clone(),
        job.input
            .request
            .policy_decision
            .eval_registry_snapshot_ref
            .clone(),
        "eval-worker:runner-dispatched".to_owned(),
    ])
}

fn is_retryable_runner_failure(reason: &str) -> bool {
    matches!(
        reason,
        "eval-runner:rate_limited" | "eval-runner:runner_error" | "eval-runner:timeout"
    )
}

fn retry_backoff_seconds(attempt_number: u32) -> u64 {
    let exponent = attempt_number.saturating_sub(1).min(5);
    BASE_RETRY_BACKOFF_SECONDS
        .saturating_mul(1_u64 << exponent)
        .min(MAX_RETRY_BACKOFF_SECONDS)
}

fn require_metadata(value: &str, evidence_ref: &str) -> Result<(), String> {
    if is_safe_metadata_ref(value) {
        Ok(())
    } else {
        Err(evidence_ref.to_owned())
    }
}

fn require_opaque(value: &str, evidence_ref: &str) -> Result<(), String> {
    if is_safe_opaque_ref(value) {
        Ok(())
    } else {
        Err(evidence_ref.to_owned())
    }
}

fn safe_metadata(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if is_safe_metadata_ref(trimmed) && trimmed == value {
        trimmed.to_owned()
    } else {
        fallback.to_owned()
    }
}

fn safe_ref(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if is_safe_opaque_ref(trimmed) && trimmed == value {
        trimmed.to_owned()
    } else {
        fallback.to_owned()
    }
}

fn is_safe_opaque_ref(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
        && trimmed.contains(':')
        && !contains_whitespace(trimmed)
        && !contains_raw_secret_material(trimmed)
        && !contains_raw_content_material(trimmed)
}

fn is_safe_metadata_ref(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
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
        || lower.contains("secret=")
}

fn contains_raw_content_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw output")
        || lower.contains("customer message")
        || lower.contains("write an email")
        || lower.contains("model answer")
        || lower.contains("prompt=")
        || lower.contains("completion=")
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

    #[test]
    fn processes_authorized_job_and_dispatches_runner_envelope() {
        let mut worker = valid_worker(EvalRunnerStatus::Accepted {
            runner_request_ref: "eval-runner://requests/req-1".to_owned(),
            run_ref: "eval-runner://runs/run-1".to_owned(),
            evidence_ref: "eval-runner:evidence:accepted".to_owned(),
        });

        let receipt = worker.run_once(valid_job());

        assert_eq!(receipt.status, EvalWorkerStatus::RunnerAccepted);
        assert_eq!(
            receipt.runner_status,
            Some(EvalRunnerDispatchStatus::Accepted)
        );
        assert_eq!(worker.eval_usecase_cached_receipt_count(), 1);
        assert_eq!(worker.events().len(), 2);
        assert_eq!(worker.events()[0].kind, EvalWorkerEventKind::JobAccepted);
        assert_eq!(worker.events()[1].kind, EvalWorkerEventKind::RunnerAccepted);
        let envelope = worker.adapter_last_envelope().expect("adapter envelope");
        assert_eq!(envelope.method, EvalRunnerHttpMethod::Post);
        assert_eq!(envelope.eval_set_id, "eval_set:worker-release-gate");
    }

    #[test]
    fn defers_not_before_jobs_without_usecase_or_adapter_side_effects() {
        let mut worker = valid_worker(EvalRunnerStatus::Queued {
            runner_request_ref: "eval-runner://requests/req-1".to_owned(),
            queue_ref: "eval-runner://queues/q-1".to_owned(),
            evidence_ref: "eval-runner:evidence:queued".to_owned(),
        });
        let mut job = valid_job();
        job.now_epoch_seconds = 100;
        job.not_before_epoch_seconds = 130;

        let receipt = worker.run_once(job);

        assert_eq!(receipt.status, EvalWorkerStatus::Deferred);
        assert_eq!(receipt.next_attempt_epoch_seconds, Some(130));
        assert_eq!(worker.eval_usecase_cached_receipt_count(), 0);
        assert!(worker.adapter_last_envelope().is_none());
        assert!(worker.events().is_empty());
    }

    #[test]
    fn usecase_denial_does_not_call_runner_adapter() {
        let mut worker = valid_worker(EvalRunnerStatus::Accepted {
            runner_request_ref: "eval-runner://requests/req-1".to_owned(),
            run_ref: "eval-runner://runs/run-1".to_owned(),
            evidence_ref: "eval-runner:evidence:accepted".to_owned(),
        });
        let mut job = valid_job();
        job.input.request.policy_decision.allowed_surfaces = vec!["surface:other".to_owned()];

        let receipt = worker.run_once(job);

        assert_eq!(receipt.status, EvalWorkerStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(EvalWorkerDenialKind::EvalUsecaseDenied)
        );
        assert!(worker.adapter_last_envelope().is_none());
        assert_eq!(worker.events()[1].kind, EvalWorkerEventKind::EvalDenied);
    }

    #[test]
    fn retryable_runner_failure_schedules_backoff_and_exhausts_at_max_attempts() {
        let mut retry_worker = valid_worker(EvalRunnerStatus::RateLimited {
            evidence_ref: "eval-runner:error:429".to_owned(),
        });
        let mut retry_job = valid_job();
        retry_job.now_epoch_seconds = 100;
        retry_job.not_before_epoch_seconds = 90;
        retry_job.attempt_number = 1;
        retry_job.max_attempts = 3;

        let retry = retry_worker.run_once(retry_job);

        assert_eq!(retry.status, EvalWorkerStatus::RetryScheduled);
        assert_eq!(retry.next_attempt_epoch_seconds, Some(130));
        assert_eq!(
            retry_worker.events()[1].kind,
            EvalWorkerEventKind::RetryScheduled
        );
        assert!(retry_worker.adapter_last_envelope().is_some());

        let mut exhausted_worker = valid_worker(EvalRunnerStatus::Timeout {
            evidence_ref: "eval-runner:error:timeout".to_owned(),
        });
        let mut exhausted_job = valid_job();
        exhausted_job.attempt_number = 3;
        exhausted_job.max_attempts = 3;

        let exhausted = exhausted_worker.run_once(exhausted_job);

        assert_eq!(exhausted.status, EvalWorkerStatus::Exhausted);
        assert_eq!(
            exhausted.denial_kind,
            Some(EvalWorkerDenialKind::RetryExhausted)
        );
    }

    #[test]
    fn nonretryable_runner_invalid_request_denies_without_retry() {
        let mut worker = valid_worker(EvalRunnerStatus::InvalidRequest {
            evidence_ref: "eval-runner:error:invalid".to_owned(),
        });

        let receipt = worker.run_once(valid_job());

        assert_eq!(receipt.status, EvalWorkerStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(EvalWorkerDenialKind::RunnerInvalidRequest)
        );
        assert_eq!(receipt.next_attempt_epoch_seconds, None);
        assert_eq!(worker.events()[1].kind, EvalWorkerEventKind::JobDenied);
    }

    #[test]
    fn invalid_raw_job_metadata_denies_before_side_effects() {
        let mut worker = valid_worker(EvalRunnerStatus::Accepted {
            runner_request_ref: "eval-runner://requests/req-1".to_owned(),
            run_ref: "eval-runner://runs/run-1".to_owned(),
            evidence_ref: "eval-runner:evidence:accepted".to_owned(),
        });
        let mut job = valid_job();
        job.input.request.eval_set.cases[0].evaluator_evidence_ref =
            "raw output model answer".to_owned();

        let receipt = worker.run_once(job);

        assert_eq!(receipt.status, EvalWorkerStatus::Denied);
        assert_eq!(receipt.denial_kind, Some(EvalWorkerDenialKind::InvalidJob));
        assert_eq!(receipt.eval_set_id, "eval_set:worker-release-gate");
        assert!(worker.events().is_empty());
        assert_eq!(worker.eval_usecase_cached_receipt_count(), 0);
        assert!(worker.adapter_last_envelope().is_none());
    }

    #[test]
    fn worker_debug_and_receipts_never_contain_raw_prompt_output_or_secret_bytes() {
        let mut worker = valid_worker(EvalRunnerStatus::Completed {
            runner_request_ref: "eval-runner://requests/req-1".to_owned(),
            run_ref: "eval-runner://runs/run-1".to_owned(),
            report_ref: "eval-report://runs/run-1/report".to_owned(),
            evidence_ref: "eval-runner:evidence:completed".to_owned(),
        });

        let receipt = worker.run_once(valid_job());
        let debug = format!(
            "{:?}{:?}{:?}",
            receipt,
            worker.events(),
            worker.adapter_last_envelope()
        );

        assert!(!debug.contains("sk-test"));
        assert!(!debug.contains("write an email to the customer"));
        assert!(!debug.contains("raw model answer"));
        assert!(!debug.contains("raw prompt"));
        assert!(!debug.contains("raw output"));
    }

    fn valid_worker(status: EvalRunnerStatus) -> IntelligenceEvalWorker {
        IntelligenceEvalWorker::new(
            IntelligenceEvalAdapter::try_new(
                EvalRunnerAdapterConfig::new(
                    "https://eval-runner.oyatie.internal",
                    "secretref://ten_a/eval-runner/byok",
                    "audit://tap/intelligence/eval",
                    "audience://intelligence/eval-runner",
                ),
                status,
            )
            .expect("valid adapter"),
        )
    }

    fn valid_job() -> EvalWorkerJob {
        EvalWorkerJob {
            job_id: "job:eval-worker:1".to_owned(),
            lease_id: "lease:eval-worker:1".to_owned(),
            attempt_id: "attempt:eval-worker:1".to_owned(),
            attempt_number: 1,
            max_attempts: 3,
            now_epoch_seconds: 1_000,
            not_before_epoch_seconds: 900,
            input: EvalUsecaseInput {
                idempotency_key: "idem:eval-worker:1".to_owned(),
                request: sample_domain_request("eval_set:worker-release-gate"),
            },
        }
    }

    fn sample_domain_request(eval_set_id: &str) -> DomainEvalSetRequest {
        DomainEvalSetRequest {
            tenant_id: "tenant:alpha".to_owned(),
            principal_id: "principal:eval-worker".to_owned(),
            eval_surface: "surface:release-gate".to_owned(),
            request_evidence_ref: "request:evidence:eval-worker:1".to_owned(),
            trace_context_ref: "trace:eval-worker:1".to_owned(),
            policy_decision_ref: "policy:evidence:eval-worker:1".to_owned(),
            policy_decision: sample_policy(),
            eval_set: sample_eval_set(eval_set_id),
        }
    }

    fn sample_policy() -> EvalPolicyDecision {
        EvalPolicyDecision {
            decision_id: "eval-policy-decision:worker:1".to_owned(),
            tenant_id: "tenant:alpha".to_owned(),
            principal_id: "principal:eval-worker".to_owned(),
            allowed_surfaces: vec!["surface:release-gate".to_owned()],
            allowed_model_refs: vec!["modelref://openai/gpt-preview".to_owned()],
            allowed_dataset_snapshot_refs: vec!["dataset://evals/worker/2026-05-23".to_owned()],
            allowed_case_kinds: vec![
                EvalCaseKind::Adversarial,
                EvalCaseKind::Golden,
                EvalCaseKind::Linguistic,
                EvalCaseKind::Regression,
                EvalCaseKind::Safety,
            ],
            min_case_count: 3,
            max_case_count: 10,
            min_pass_rate_bps: 8_000,
            max_safety_violation_rate_bps: 0,
            evidence_ref: "policy:evidence:eval-worker:1".to_owned(),
            eval_registry_snapshot_ref: "eval-registry:snapshot:worker:1".to_owned(),
        }
    }

    fn sample_eval_set(eval_set_id: &str) -> EvalSet {
        EvalSet {
            eval_set_id: eval_set_id.to_owned(),
            model_ref: "modelref://openai/gpt-preview".to_owned(),
            route_evidence_ref: "route:evidence:eval-worker:1".to_owned(),
            guardrail_evidence_ref: "guardrail:evidence:eval-worker:1".to_owned(),
            dataset_snapshot_ref: "dataset://evals/worker/2026-05-23".to_owned(),
            thresholds: EvalSetThresholds {
                min_pass_rate_bps: 8_000,
                max_safety_violation_rate_bps: 0,
                require_golden: true,
                require_adversarial: true,
                require_linguistic: true,
            },
            cases: vec![
                case(
                    "case-golden-worker-1",
                    EvalCaseKind::Golden,
                    EvalCaseOutcome::Passed,
                    9_500,
                    "eval:case:worker:golden:1",
                ),
                case(
                    "case-adversarial-worker-1",
                    EvalCaseKind::Adversarial,
                    EvalCaseOutcome::Passed,
                    8_800,
                    "eval:case:worker:adversarial:1",
                ),
                case(
                    "case-linguistic-worker-1",
                    EvalCaseKind::Linguistic,
                    EvalCaseOutcome::Passed,
                    8_700,
                    "eval:case:worker:linguistic:1",
                ),
            ],
        }
    }

    fn case(
        case_id: &str,
        kind: EvalCaseKind,
        outcome: EvalCaseOutcome,
        score_bps: u16,
        evidence_ref: &str,
    ) -> EvalCaseResult {
        EvalCaseResult {
            case_id: case_id.to_owned(),
            kind,
            outcome,
            score_bps,
            evaluator_evidence_ref: evidence_ref.to_owned(),
        }
    }
}
