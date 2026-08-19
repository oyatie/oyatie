//! Intelligence context-aware retrieval worker foundation.
//!
//! This crate provides a deterministic source-level worker seam for future
//! context-aware retrieval job execution. It validates queued job metadata,
//! runs the retrieval usecase, and hands planned receipts to the metadata-only
//! retrieval executor adapter. It performs no queue I/O, network I/O,
//! vector-store calls, embedding generation, ontology/KG execution, document
//! fetch, filesystem access, durable idempotency, durable audit-chain emission,
//! or cloud runtime scheduling.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use intelligence_context_aware_retrieval_adapter::{
    ContextAudience, ContextCandidate, ContextDataClass, ContextRetrievalDomainDenialKind,
    ContextRetrievalExecutorAdapterConfig, ContextRetrievalExecutorAdapterConfigError,
    ContextRetrievalExecutorDispatchFailure, ContextRetrievalExecutorDispatchRequest,
    ContextRetrievalExecutorDispatchStatus, ContextRetrievalExecutorHttpMethod,
    ContextRetrievalExecutorRequestEnvelope, ContextRetrievalExecutorStatus,
    ContextRetrievalExecutorTransportMode, ContextRetrievalPolicyDecision, ContextRetrievalRequest,
    ContextRetrievalUsecaseDenialKind, ContextRetrievalUsecaseReceipt,
    ContextRetrievalUsecaseStatus, ContextSourceKind, DomainContextRetrievalRequest,
    IntelligenceContextAwareRetrievalAdapter,
};
pub use intelligence_context_aware_retrieval_usecase::{
    ContextAwareRetrievalUsecase, ContextRetrievalUsecaseInput,
};

const MAX_WORKER_ATTEMPTS: u32 = 10;
const BASE_RETRY_BACKOFF_SECONDS: u64 = 30;
const MAX_RETRY_BACKOFF_SECONDS: u64 = 900;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextRetrievalWorkerJob {
    pub job_id: String,                      // data_class: INTERNAL_ONLY
    pub lease_id: String,                    // data_class: INTERNAL_ONLY
    pub attempt_id: String,                  // data_class: INTERNAL_ONLY
    pub attempt_number: u32,                 // data_class: INTERNAL_ONLY
    pub max_attempts: u32,                   // data_class: INTERNAL_ONLY
    pub now_epoch_seconds: u64,              // data_class: INTERNAL_ONLY
    pub not_before_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
    pub input: ContextRetrievalUsecaseInput, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextRetrievalWorkerStatus {
    Deferred,
    Denied,
    Exhausted,
    RetryScheduled,
    ExecutorAccepted,
    ExecutorCompleted,
    ExecutorQueued,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextRetrievalWorkerDenialKind {
    ExecutorDenied,
    ExecutorInvalidRequest,
    InvalidJob,
    RetrievalUsecaseDenied,
    RetryExhausted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextRetrievalWorkerReceipt {
    pub job_id: String,                       // data_class: INTERNAL_ONLY
    pub attempt_id: String,                   // data_class: INTERNAL_ONLY
    pub idempotency_key: String,              // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub principal_id: String,                 // data_class: INTERNAL_ONLY
    pub query_surface: String,                // data_class: INTERNAL_ONLY
    pub query_ref: String,                    // data_class: INTERNAL_ONLY
    pub status: ContextRetrievalWorkerStatus, // data_class: PUBLIC
    pub denial_kind: Option<ContextRetrievalWorkerDenialKind>, // data_class: INTERNAL_ONLY
    pub executor_status: Option<ContextRetrievalExecutorDispatchStatus>, // data_class: INTERNAL_ONLY
    pub executor_request_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub retrieval_ref: Option<String>,        // data_class: INTERNAL_ONLY
    pub queue_ref: Option<String>,            // data_class: INTERNAL_ONLY
    pub context_bundle_ref: Option<String>,   // data_class: INTERNAL_ONLY
    pub executor_evidence_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub next_attempt_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextRetrievalWorkerEventKind {
    ExecutorAccepted,
    ExecutorCompleted,
    ExecutorQueued,
    JobAccepted,
    JobDenied,
    RetrievalDenied,
    RetryExhausted,
    RetryScheduled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextRetrievalWorkerEvent {
    pub kind: ContextRetrievalWorkerEventKind, // data_class: INTERNAL_ONLY
    pub job_id: String,                        // data_class: INTERNAL_ONLY
    pub attempt_id: String,                    // data_class: INTERNAL_ONLY
    pub idempotency_key: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub query_surface: String,                 // data_class: INTERNAL_ONLY
    pub query_ref: String,                     // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,            // data_class: INTERNAL_ONLY
}

pub struct ContextRetrievalWorker {
    retrieval_usecase: ContextAwareRetrievalUsecase,
    adapter: IntelligenceContextAwareRetrievalAdapter,
    events: Vec<ContextRetrievalWorkerEvent>,
}

impl ContextRetrievalWorker {
    pub fn new(adapter: IntelligenceContextAwareRetrievalAdapter) -> Self {
        Self {
            retrieval_usecase: ContextAwareRetrievalUsecase::default(),
            adapter,
            events: Vec::new(),
        }
    }

    pub fn run_once(&mut self, job: ContextRetrievalWorkerJob) -> ContextRetrievalWorkerReceipt {
        if let Err(evidence_ref) = validate_job(&job) {
            return receipt_from_job(
                &job,
                ContextRetrievalWorkerStatus::Denied,
                Some(ContextRetrievalWorkerDenialKind::InvalidJob),
                None,
                ExecutorRefs::default(),
                None,
                Some(evidence_ref.clone()),
                vec![evidence_ref],
            );
        }

        if job.now_epoch_seconds < job.not_before_epoch_seconds {
            return receipt_from_job(
                &job,
                ContextRetrievalWorkerStatus::Deferred,
                None,
                None,
                ExecutorRefs::default(),
                Some(job.not_before_epoch_seconds),
                None,
                vec!["context-retrieval-worker:deferred:not-before".to_owned()],
            );
        }

        self.record_event(
            ContextRetrievalWorkerEventKind::JobAccepted,
            &job,
            canonical_request_evidence_refs(&job),
        );

        let retrieval_receipt = self.retrieval_usecase.plan(job.input.clone());
        if retrieval_receipt.status != ContextRetrievalUsecaseStatus::Planned {
            let receipt = receipt_from_retrieval_denial(&job, &retrieval_receipt);
            self.record_event(
                ContextRetrievalWorkerEventKind::RetrievalDenied,
                &job,
                receipt.evidence_refs.clone(),
            );
            return receipt;
        }

        match self
            .adapter
            .dispatch(ContextRetrievalExecutorDispatchRequest {
                idempotency_key: job.input.idempotency_key.clone(),
                domain_request: job.input.request.clone(),
                usecase_receipt: retrieval_receipt,
            }) {
            Ok(executor_receipt) => {
                let (status, event_kind) = match executor_receipt.status {
                    ContextRetrievalExecutorDispatchStatus::Accepted => (
                        ContextRetrievalWorkerStatus::ExecutorAccepted,
                        ContextRetrievalWorkerEventKind::ExecutorAccepted,
                    ),
                    ContextRetrievalExecutorDispatchStatus::Queued => (
                        ContextRetrievalWorkerStatus::ExecutorQueued,
                        ContextRetrievalWorkerEventKind::ExecutorQueued,
                    ),
                    ContextRetrievalExecutorDispatchStatus::Completed => (
                        ContextRetrievalWorkerStatus::ExecutorCompleted,
                        ContextRetrievalWorkerEventKind::ExecutorCompleted,
                    ),
                };
                let evidence_ref = executor_receipt.evidence_ref.clone();
                let receipt = receipt_from_job(
                    &job,
                    status,
                    None,
                    Some(executor_receipt.status),
                    ExecutorRefs {
                        executor_request_ref: executor_receipt.executor_request_ref,
                        retrieval_ref: executor_receipt.retrieval_ref,
                        queue_ref: executor_receipt.queue_ref,
                        context_bundle_ref: executor_receipt.context_bundle_ref,
                    },
                    None,
                    Some(evidence_ref),
                    worker_success_evidence_refs(&job),
                );
                self.record_event(event_kind, &job, receipt.evidence_refs.clone());
                receipt
            }
            Err(failure) => self.receipt_from_executor_failure(&job, failure),
        }
    }

    pub fn events(&self) -> &[ContextRetrievalWorkerEvent] {
        &self.events
    }

    pub fn retrieval_usecase_receipt_count(&self) -> usize {
        self.retrieval_usecase.receipt_count()
    }

    pub fn adapter_last_envelope(&self) -> Option<&ContextRetrievalExecutorRequestEnvelope> {
        self.adapter.last_envelope()
    }

    pub fn set_executor_status(&mut self, status: ContextRetrievalExecutorStatus) {
        self.adapter.set_next_status(status);
    }

    fn receipt_from_executor_failure(
        &mut self,
        job: &ContextRetrievalWorkerJob,
        failure: ContextRetrievalExecutorDispatchFailure,
    ) -> ContextRetrievalWorkerReceipt {
        if is_retryable_executor_failure(&failure.reason) {
            if job.attempt_number < job.max_attempts {
                let next_attempt = job
                    .now_epoch_seconds
                    .saturating_add(retry_backoff_seconds(job.attempt_number));
                let receipt = receipt_from_job(
                    job,
                    ContextRetrievalWorkerStatus::RetryScheduled,
                    None,
                    None,
                    ExecutorRefs::default(),
                    Some(next_attempt),
                    Some(failure.evidence_ref),
                    vec!["context-retrieval-worker:executor:retry-scheduled".to_owned()],
                );
                self.record_event(
                    ContextRetrievalWorkerEventKind::RetryScheduled,
                    job,
                    receipt.evidence_refs.clone(),
                );
                return receipt;
            }
            let receipt = receipt_from_job(
                job,
                ContextRetrievalWorkerStatus::Exhausted,
                Some(ContextRetrievalWorkerDenialKind::RetryExhausted),
                None,
                ExecutorRefs::default(),
                None,
                Some(failure.evidence_ref),
                vec!["context-retrieval-worker:executor:retry-exhausted".to_owned()],
            );
            self.record_event(
                ContextRetrievalWorkerEventKind::RetryExhausted,
                job,
                receipt.evidence_refs.clone(),
            );
            return receipt;
        }

        let denial_kind = if failure.reason == "context-retrieval-executor:invalid_request" {
            ContextRetrievalWorkerDenialKind::ExecutorInvalidRequest
        } else {
            ContextRetrievalWorkerDenialKind::ExecutorDenied
        };
        let receipt = receipt_from_job(
            job,
            ContextRetrievalWorkerStatus::Denied,
            Some(denial_kind),
            None,
            ExecutorRefs::default(),
            None,
            Some(failure.evidence_ref),
            vec!["context-retrieval-worker:executor:denied".to_owned()],
        );
        self.record_event(
            ContextRetrievalWorkerEventKind::JobDenied,
            job,
            receipt.evidence_refs.clone(),
        );
        receipt
    }

    fn record_event(
        &mut self,
        kind: ContextRetrievalWorkerEventKind,
        job: &ContextRetrievalWorkerJob,
        evidence_refs: Vec<String>,
    ) {
        self.events.push(ContextRetrievalWorkerEvent {
            kind,
            job_id: safe_metadata(&job.job_id, "redacted-invalid-job-id"),
            attempt_id: safe_metadata(&job.attempt_id, "redacted-invalid-attempt-id"),
            idempotency_key: safe_metadata(
                &job.input.idempotency_key,
                "redacted-invalid-idempotency-key",
            ),
            tenant_id: safe_tenant(&job.input.request.request.tenant_id),
            query_surface: safe_metadata(
                &job.input.request.query_surface,
                "redacted-invalid-query-surface",
            ),
            query_ref: safe_ref(
                &job.input.request.request.query_ref,
                "redacted-invalid-query-ref",
            ),
            evidence_refs: sorted_unique(evidence_refs),
        });
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ExecutorRefs {
    executor_request_ref: Option<String>,
    retrieval_ref: Option<String>,
    queue_ref: Option<String>,
    context_bundle_ref: Option<String>,
}

fn validate_job(job: &ContextRetrievalWorkerJob) -> Result<(), String> {
    require_metadata(&job.job_id, "validation:context-retrieval-worker-job-id")?;
    require_metadata(
        &job.lease_id,
        "validation:context-retrieval-worker-lease-id",
    )?;
    require_metadata(
        &job.attempt_id,
        "validation:context-retrieval-worker-attempt-id",
    )?;
    if job.attempt_number == 0
        || job.max_attempts == 0
        || job.max_attempts > MAX_WORKER_ATTEMPTS
        || job.attempt_number > job.max_attempts
    {
        return Err("validation:context-retrieval-worker-attempt-bounds".to_owned());
    }
    validate_input(&job.input)
}

fn validate_input(input: &ContextRetrievalUsecaseInput) -> Result<(), String> {
    require_metadata(
        &input.idempotency_key,
        "validation:context-retrieval-worker-idempotency-key",
    )?;
    let domain = &input.request;
    let request = &domain.request;
    let policy = &domain.policy_decision;
    require_metadata(
        &domain.principal_id,
        "validation:context-retrieval-worker-principal",
    )?;
    require_metadata(
        &domain.query_surface,
        "validation:context-retrieval-worker-surface",
    )?;
    require_tenant(
        &request.tenant_id,
        "validation:context-retrieval-worker-tenant",
    )?;
    require_opaque(
        &request.query_ref,
        "validation:context-retrieval-worker-query-ref",
    )?;
    require_opaque(
        &request.request_evidence_ref,
        "validation:context-retrieval-worker-request-evidence",
    )?;
    require_opaque(
        &request.trace_context_ref,
        "validation:context-retrieval-worker-trace-context",
    )?;
    require_opaque(
        &request.policy_decision_ref,
        "validation:context-retrieval-worker-policy-decision",
    )?;
    if request.allowed_source_kinds.is_empty() {
        return Err("validation:context-retrieval-worker-source-kinds".to_owned());
    }
    if request.max_context_items == 0 || request.max_context_items > 32 {
        return Err("validation:context-retrieval-worker-max-context-items".to_owned());
    }
    for candidate in &request.candidates {
        require_tenant(
            &candidate.tenant_id,
            "validation:context-retrieval-worker-candidate-tenant",
        )?;
        require_opaque(
            &candidate.resource_ref,
            "validation:context-retrieval-worker-candidate-resource-ref",
        )?;
        require_opaque(
            &candidate.evidence_ref,
            "validation:context-retrieval-worker-candidate-evidence-ref",
        )?;
        if candidate.relevance_millis > 1000 {
            return Err("validation:context-retrieval-worker-candidate-relevance".to_owned());
        }
    }
    require_metadata(
        &policy.decision_id,
        "validation:context-retrieval-worker-policy-id",
    )?;
    require_tenant(
        &policy.tenant_id,
        "validation:context-retrieval-worker-policy-tenant",
    )?;
    require_metadata(
        &policy.principal_id,
        "validation:context-retrieval-worker-policy-principal",
    )?;
    if policy.allowed_surfaces.is_empty() || policy.allowed_source_kinds.is_empty() {
        return Err("validation:context-retrieval-worker-policy-allowlists".to_owned());
    }
    for surface in &policy.allowed_surfaces {
        require_metadata(
            surface,
            "validation:context-retrieval-worker-policy-surface",
        )?;
    }
    if policy.max_context_items == 0 || policy.max_context_items > 32 {
        return Err("validation:context-retrieval-worker-policy-max-context-items".to_owned());
    }
    require_opaque(
        &policy.evidence_ref,
        "validation:context-retrieval-worker-policy-evidence",
    )?;
    require_opaque(
        &policy.retrieval_index_snapshot_ref,
        "validation:context-retrieval-worker-index-snapshot",
    )?;
    Ok(())
}

fn receipt_from_retrieval_denial(
    job: &ContextRetrievalWorkerJob,
    retrieval_receipt: &ContextRetrievalUsecaseReceipt,
) -> ContextRetrievalWorkerReceipt {
    receipt_from_job(
        job,
        ContextRetrievalWorkerStatus::Denied,
        Some(ContextRetrievalWorkerDenialKind::RetrievalUsecaseDenied),
        None,
        ExecutorRefs::default(),
        None,
        None,
        sorted_unique(
            [
                retrieval_receipt.evidence_refs.clone(),
                vec!["context-retrieval-worker:retrieval-usecase-denied".to_owned()],
            ]
            .concat(),
        ),
    )
}

#[allow(clippy::too_many_arguments)]
fn receipt_from_job(
    job: &ContextRetrievalWorkerJob,
    status: ContextRetrievalWorkerStatus,
    denial_kind: Option<ContextRetrievalWorkerDenialKind>,
    executor_status: Option<ContextRetrievalExecutorDispatchStatus>,
    executor_refs: ExecutorRefs,
    next_attempt_epoch_seconds: Option<u64>,
    executor_evidence_ref: Option<String>,
    evidence_refs: Vec<String>,
) -> ContextRetrievalWorkerReceipt {
    ContextRetrievalWorkerReceipt {
        job_id: safe_metadata(&job.job_id, "redacted-invalid-job-id"),
        attempt_id: safe_metadata(&job.attempt_id, "redacted-invalid-attempt-id"),
        idempotency_key: safe_metadata(
            &job.input.idempotency_key,
            "redacted-invalid-idempotency-key",
        ),
        tenant_id: safe_tenant(&job.input.request.request.tenant_id),
        principal_id: safe_metadata(
            &job.input.request.principal_id,
            "redacted-invalid-principal-id",
        ),
        query_surface: safe_metadata(
            &job.input.request.query_surface,
            "redacted-invalid-query-surface",
        ),
        query_ref: safe_ref(
            &job.input.request.request.query_ref,
            "redacted-invalid-query-ref",
        ),
        status,
        denial_kind,
        executor_status,
        executor_request_ref: executor_refs.executor_request_ref.map(|value| {
            safe_ref(
                &value,
                "context-retrieval-worker:redacted-executor-request-ref",
            )
        }),
        retrieval_ref: executor_refs
            .retrieval_ref
            .map(|value| safe_ref(&value, "context-retrieval-worker:redacted-retrieval-ref")),
        queue_ref: executor_refs
            .queue_ref
            .map(|value| safe_ref(&value, "context-retrieval-worker:redacted-queue-ref")),
        context_bundle_ref: executor_refs.context_bundle_ref.map(|value| {
            safe_ref(
                &value,
                "context-retrieval-worker:redacted-context-bundle-ref",
            )
        }),
        executor_evidence_ref: executor_evidence_ref.map(|value| {
            safe_ref(
                &value,
                "context-retrieval-worker:redacted-executor-evidence-ref",
            )
        }),
        next_attempt_epoch_seconds,
        evidence_refs: sorted_unique(evidence_refs),
    }
}

fn worker_success_evidence_refs(job: &ContextRetrievalWorkerJob) -> Vec<String> {
    let mut refs = canonical_request_evidence_refs(job);
    refs.push("context-retrieval-worker:executor-dispatched".to_owned());
    sorted_unique(refs)
}

fn canonical_request_evidence_refs(job: &ContextRetrievalWorkerJob) -> Vec<String> {
    sorted_unique(vec![
        job.input.request.request.request_evidence_ref.clone(),
        job.input.request.request.trace_context_ref.clone(),
        job.input.request.request.policy_decision_ref.clone(),
        job.input.request.policy_decision.evidence_ref.clone(),
        job.input
            .request
            .policy_decision
            .retrieval_index_snapshot_ref
            .clone(),
    ])
}

fn is_retryable_executor_failure(reason: &str) -> bool {
    matches!(
        reason,
        "context-retrieval-executor:rate_limited"
            | "context-retrieval-executor:executor_error"
            | "context-retrieval-executor:timeout"
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

fn require_tenant(value: &str, evidence_ref: &str) -> Result<(), String> {
    if is_safe_tenant_id(value) {
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

fn safe_tenant(value: &str) -> String {
    let trimmed = value.trim();
    if is_safe_tenant_id(trimmed) && trimmed == value {
        trimmed.to_owned()
    } else {
        "redacted-invalid-tenant-id".to_owned()
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

fn is_safe_tenant_id(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
        && trimmed.starts_with("ten_")
        && !trimmed.contains('/')
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
        || lower.contains("raw query")
        || lower.contains("customer message")
        || lower.contains("write an email")
        || lower.contains("model answer")
        || lower.contains("prompt=")
        || lower.contains("query=")
        || lower.contains("document raw")
        || lower.contains("document=")
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
    fn processes_authorized_job_and_dispatches_executor_envelope() {
        let mut worker = valid_worker(ContextRetrievalExecutorStatus::Accepted {
            executor_request_ref: "ctx-executor://requests/req-1".to_owned(),
            retrieval_ref: "ctx-retrieval://plans/plan-1".to_owned(),
            evidence_ref: "ctx-executor:evidence:accepted".to_owned(),
        });

        let receipt = worker.run_once(valid_job());

        assert_eq!(
            receipt.status,
            ContextRetrievalWorkerStatus::ExecutorAccepted
        );
        assert_eq!(
            receipt.executor_status,
            Some(ContextRetrievalExecutorDispatchStatus::Accepted)
        );
        assert_eq!(worker.retrieval_usecase_receipt_count(), 1);
        assert_eq!(worker.events().len(), 2);
        assert_eq!(
            worker.events()[0].kind,
            ContextRetrievalWorkerEventKind::JobAccepted
        );
        assert_eq!(
            worker.events()[1].kind,
            ContextRetrievalWorkerEventKind::ExecutorAccepted
        );
        let envelope = worker.adapter_last_envelope().expect("adapter envelope");
        assert_eq!(envelope.method, ContextRetrievalExecutorHttpMethod::Post);
        assert_eq!(envelope.query_ref, "queryref://opaque/worker-1");
        assert_eq!(envelope.planned_context_count, 2);
    }

    #[test]
    fn defers_not_before_jobs_without_usecase_or_adapter_side_effects() {
        let mut worker = valid_worker(ContextRetrievalExecutorStatus::Queued {
            executor_request_ref: "ctx-executor://requests/req-1".to_owned(),
            queue_ref: "ctx-executor://queues/q-1".to_owned(),
            evidence_ref: "ctx-executor:evidence:queued".to_owned(),
        });
        let mut job = valid_job();
        job.now_epoch_seconds = 100;
        job.not_before_epoch_seconds = 130;

        let receipt = worker.run_once(job);

        assert_eq!(receipt.status, ContextRetrievalWorkerStatus::Deferred);
        assert_eq!(receipt.next_attempt_epoch_seconds, Some(130));
        assert_eq!(worker.retrieval_usecase_receipt_count(), 0);
        assert!(worker.adapter_last_envelope().is_none());
        assert!(worker.events().is_empty());
    }

    #[test]
    fn usecase_denial_does_not_call_executor_adapter() {
        let mut worker = valid_worker(ContextRetrievalExecutorStatus::Completed {
            executor_request_ref: "ctx-executor://requests/req-1".to_owned(),
            retrieval_ref: "ctx-retrieval://plans/plan-1".to_owned(),
            context_bundle_ref: "context-bundle://bundles/b-1".to_owned(),
            evidence_ref: "ctx-executor:evidence:completed".to_owned(),
        });
        let mut job = valid_job();
        job.input.request.request.candidates.clear();

        let receipt = worker.run_once(job);

        assert_eq!(receipt.status, ContextRetrievalWorkerStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(ContextRetrievalWorkerDenialKind::RetrievalUsecaseDenied)
        );
        assert!(worker.adapter_last_envelope().is_none());
        assert_eq!(worker.events().len(), 2);
        assert_eq!(
            worker.events()[1].kind,
            ContextRetrievalWorkerEventKind::RetrievalDenied
        );
    }

    #[test]
    fn retryable_executor_failure_schedules_backoff_and_exhausts_at_max_attempts() {
        let mut worker = valid_worker(ContextRetrievalExecutorStatus::RateLimited {
            evidence_ref: "ctx-executor:evidence:rate-limited".to_owned(),
        });
        let mut job = valid_job();
        job.now_epoch_seconds = 1_000;
        job.attempt_number = 2;
        job.max_attempts = 4;

        let receipt = worker.run_once(job.clone());

        assert_eq!(receipt.status, ContextRetrievalWorkerStatus::RetryScheduled);
        assert_eq!(receipt.next_attempt_epoch_seconds, Some(1_060));
        assert_eq!(
            worker.events()[1].kind,
            ContextRetrievalWorkerEventKind::RetryScheduled
        );

        worker.set_executor_status(ContextRetrievalExecutorStatus::Timeout {
            evidence_ref: "ctx-executor:evidence:timeout".to_owned(),
        });
        job.attempt_number = 4;
        let exhausted = worker.run_once(job);

        assert_eq!(exhausted.status, ContextRetrievalWorkerStatus::Exhausted);
        assert_eq!(
            exhausted.denial_kind,
            Some(ContextRetrievalWorkerDenialKind::RetryExhausted)
        );
    }

    #[test]
    fn nonretryable_executor_invalid_request_denies_without_retry() {
        let mut worker = valid_worker(ContextRetrievalExecutorStatus::InvalidRequest {
            evidence_ref: "ctx-executor:evidence:invalid-request".to_owned(),
        });

        let receipt = worker.run_once(valid_job());

        assert_eq!(receipt.status, ContextRetrievalWorkerStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(ContextRetrievalWorkerDenialKind::ExecutorInvalidRequest)
        );
        assert_eq!(receipt.next_attempt_epoch_seconds, None);
        assert_eq!(
            worker.events()[1].kind,
            ContextRetrievalWorkerEventKind::JobDenied
        );
    }

    #[test]
    fn invalid_raw_job_metadata_denies_before_side_effects() {
        let mut worker = valid_worker(ContextRetrievalExecutorStatus::Accepted {
            executor_request_ref: "ctx-executor://requests/req-1".to_owned(),
            retrieval_ref: "ctx-retrieval://plans/plan-1".to_owned(),
            evidence_ref: "ctx-executor:evidence:accepted".to_owned(),
        });
        let mut job = valid_job();
        job.job_id = "job raw query: customer message".to_owned();
        job.input.request.request.query_ref = "raw query: find patient data".to_owned();
        job.input.idempotency_key = "sk-unsafe".to_owned();

        let receipt = worker.run_once(job);

        assert_eq!(receipt.status, ContextRetrievalWorkerStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(ContextRetrievalWorkerDenialKind::InvalidJob)
        );
        assert_eq!(receipt.job_id, "redacted-invalid-job-id");
        assert_eq!(receipt.idempotency_key, "redacted-invalid-idempotency-key");
        assert_eq!(worker.retrieval_usecase_receipt_count(), 0);
        assert!(worker.adapter_last_envelope().is_none());
        assert!(worker.events().is_empty());
    }

    #[test]
    fn worker_debug_and_receipts_never_contain_raw_query_document_or_secret_bytes() {
        let mut worker = valid_worker(ContextRetrievalExecutorStatus::Denied {
            evidence_ref: "ctx-executor:evidence:denied".to_owned(),
        });
        let mut job = valid_job();
        job.input.request.request.candidates[0].resource_ref =
            "document raw prompt sk-test".to_owned();

        let receipt = worker.run_once(job);

        let debug = format!("{receipt:?} {:?}", worker.events());
        assert!(!debug.contains("raw prompt"));
        assert!(!debug.contains("sk-test"));
        assert!(!debug.contains("customer message"));
        assert_eq!(receipt.status, ContextRetrievalWorkerStatus::Denied);
    }

    fn valid_worker(status: ContextRetrievalExecutorStatus) -> ContextRetrievalWorker {
        ContextRetrievalWorker::new(
            IntelligenceContextAwareRetrievalAdapter::try_new(
                ContextRetrievalExecutorAdapterConfig::new(
                    "https://retrieval-executor.oyatie.internal/",
                    "secretref://ten_a/retrieval-executor/byok",
                    "audit://tap/intelligence/context-aware-retrieval",
                    "audience://intelligence/context-retrieval-executor",
                ),
                status,
            )
            .expect("valid adapter config"),
        )
    }

    fn valid_job() -> ContextRetrievalWorkerJob {
        ContextRetrievalWorkerJob {
            job_id: "ctx-worker-job-1".to_owned(),
            lease_id: "ctx-worker-lease-1".to_owned(),
            attempt_id: "ctx-worker-attempt-1".to_owned(),
            attempt_number: 1,
            max_attempts: 4,
            now_epoch_seconds: 1_000,
            not_before_epoch_seconds: 900,
            input: sample_input(),
        }
    }

    fn sample_input() -> ContextRetrievalUsecaseInput {
        ContextRetrievalUsecaseInput {
            idempotency_key: "idem-ctx-worker-1".to_owned(),
            request: DomainContextRetrievalRequest {
                principal_id: "principal-ctx-worker-1".to_owned(),
                query_surface: "intelligence.context-aware-retrieval.pre".to_owned(),
                request: ContextRetrievalRequest {
                    tenant_id: "ten_a".to_owned(),
                    query_ref: "queryref://opaque/worker-1".to_owned(),
                    request_evidence_ref: "req:ctx-worker:1".to_owned(),
                    trace_context_ref: "trace:ctx-worker:1".to_owned(),
                    policy_decision_ref: "cedar:ctx-worker:allow".to_owned(),
                    audience: ContextAudience::TenantOperator,
                    allowed_source_kinds: vec![
                        ContextSourceKind::OntologyEntity,
                        ContextSourceKind::KnowledgeGraphSubgraph,
                    ],
                    max_context_items: 2,
                    freshness_floor_epoch_seconds: 100,
                    candidates: vec![
                        ContextCandidate {
                            tenant_id: "ten_a".to_owned(),
                            source_kind: ContextSourceKind::OntologyEntity,
                            resource_ref: "entityref://org/worker-2".to_owned(),
                            evidence_ref: "ctx:entity:worker:2".to_owned(),
                            data_class: ContextDataClass::InternalOnly,
                            observed_at_epoch_seconds: 125,
                            relevance_millis: 920,
                        },
                        ContextCandidate {
                            tenant_id: "ten_a".to_owned(),
                            source_kind: ContextSourceKind::KnowledgeGraphSubgraph,
                            resource_ref: "kgref://subgraph/worker-1".to_owned(),
                            evidence_ref: "ctx:kg:worker:1".to_owned(),
                            data_class: ContextDataClass::InternalOnly,
                            observed_at_epoch_seconds: 130,
                            relevance_millis: 930,
                        },
                    ],
                },
                policy_decision: ContextRetrievalPolicyDecision {
                    decision_id: "policy-ctx-worker-1".to_owned(),
                    tenant_id: "ten_a".to_owned(),
                    principal_id: "principal-ctx-worker-1".to_owned(),
                    allowed_surfaces: vec!["intelligence.context-aware-retrieval.pre".to_owned()],
                    allowed_source_kinds: vec![
                        ContextSourceKind::OntologyEntity,
                        ContextSourceKind::KnowledgeGraphSubgraph,
                    ],
                    max_context_items: 2,
                    freshness_floor_epoch_seconds: 100,
                    evidence_ref: "cedar:ctx-worker:allow".to_owned(),
                    retrieval_index_snapshot_ref: "retrieval-index:snapshot:worker:1".to_owned(),
                },
            },
        }
    }
}
