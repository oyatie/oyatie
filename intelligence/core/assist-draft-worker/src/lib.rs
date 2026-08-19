//! Intelligence assist-draft worker foundation.
//!
//! This crate provides a deterministic source-level worker seam for future
//! assist-draft job execution. It validates queued job metadata, honors
//! not-before scheduling before side effects, runs the assist-draft usecase,
//! and hands planned receipts to the metadata-only assist-draft executor
//! adapter. It performs no durable queue I/O, network I/O, prompt rendering,
//! model/provider calls, builder mutation, filesystem access, durable
//! idempotency storage, durable audit-chain emission, or cloud runtime
//! scheduling.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use intelligence_assist_draft_adapter::{
    AssistDraftAction, AssistDraftAudience, AssistDraftBuilderSurface, AssistDraftDataClass,
    AssistDraftDenialReason, AssistDraftDomainDecision, AssistDraftDomainDenialKind,
    AssistDraftDomainStatus, AssistDraftExecutorAdapterConfig,
    AssistDraftExecutorAdapterConfigError, AssistDraftExecutorDispatchFailure,
    AssistDraftExecutorDispatchRequest, AssistDraftExecutorDispatchStatus,
    AssistDraftExecutorHttpMethod, AssistDraftExecutorRequestEnvelope, AssistDraftExecutorStatus,
    AssistDraftExecutorTransportMode, AssistDraftInvocationMode, AssistDraftKind,
    AssistDraftPolicyDecision, AssistDraftRequest, AssistDraftReviewGate,
    AssistDraftUsecaseDenialKind, AssistDraftUsecaseInput, AssistDraftUsecaseReceipt,
    AssistDraftUsecaseStatus, DomainAssistDraftRequest, IntelligenceAssistDraftAdapter,
    IntelligenceAssistDraftUsecase, plan_domain_assist_draft,
};

const MAX_WORKER_ATTEMPTS: u32 = 10;
const BASE_RETRY_BACKOFF_SECONDS: u64 = 30;
const MAX_RETRY_BACKOFF_SECONDS: u64 = 900;
const MAX_PROMPT_CONTEXT_REFS: usize = 16;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftWorkerJob {
    pub job_id: String,                 // data_class: INTERNAL_ONLY
    pub lease_id: String,               // data_class: INTERNAL_ONLY
    pub attempt_id: String,             // data_class: INTERNAL_ONLY
    pub attempt_number: u32,            // data_class: INTERNAL_ONLY
    pub max_attempts: u32,              // data_class: INTERNAL_ONLY
    pub now_epoch_seconds: u64,         // data_class: INTERNAL_ONLY
    pub not_before_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
    pub input: AssistDraftUsecaseInput, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssistDraftWorkerStatus {
    Deferred,
    Denied,
    Exhausted,
    RetryScheduled,
    ExecutorAccepted,
    ExecutorCompleted,
    ExecutorQueued,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssistDraftWorkerDenialKind {
    AssistDraftUsecaseDenied,
    ExecutorDenied,
    ExecutorInvalidRequest,
    InvalidJob,
    RetryExhausted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftWorkerReceipt {
    pub job_id: String,                                   // data_class: INTERNAL_ONLY
    pub attempt_id: String,                               // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                          // data_class: INTERNAL_ONLY
    pub tenant_id: String,                                // data_class: INTERNAL_ONLY
    pub principal_id: String,                             // data_class: INTERNAL_ONLY
    pub brand_surface_ref: String,                        // data_class: INTERNAL_ONLY
    pub target_builder_ref: String,                       // data_class: INTERNAL_ONLY
    pub output_contract_ref: String,                      // data_class: INTERNAL_ONLY
    pub status: AssistDraftWorkerStatus,                  // data_class: PUBLIC
    pub denial_kind: Option<AssistDraftWorkerDenialKind>, // data_class: INTERNAL_ONLY
    pub executor_status: Option<AssistDraftExecutorDispatchStatus>, // data_class: INTERNAL_ONLY
    pub executor_request_ref: Option<String>,             // data_class: INTERNAL_ONLY
    pub draft_ref: Option<String>,                        // data_class: INTERNAL_ONLY
    pub queue_ref: Option<String>,                        // data_class: INTERNAL_ONLY
    pub suggested_patch_ref: Option<String>,              // data_class: INTERNAL_ONLY
    pub executor_evidence_ref: Option<String>,            // data_class: INTERNAL_ONLY
    pub next_attempt_epoch_seconds: Option<u64>,          // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssistDraftWorkerEventKind {
    AssistDraftDenied,
    ExecutorAccepted,
    ExecutorCompleted,
    ExecutorQueued,
    JobAccepted,
    JobDenied,
    RetryExhausted,
    RetryScheduled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftWorkerEvent {
    pub kind: AssistDraftWorkerEventKind, // data_class: INTERNAL_ONLY
    pub job_id: String,                   // data_class: INTERNAL_ONLY
    pub attempt_id: String,               // data_class: INTERNAL_ONLY
    pub idempotency_key: String,          // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub target_builder_ref: String,       // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ExecutorRefs {
    executor_request_ref: Option<String>,
    draft_ref: Option<String>,
    queue_ref: Option<String>,
    suggested_patch_ref: Option<String>,
}

pub struct AssistDraftWorker {
    assist_draft_usecase: IntelligenceAssistDraftUsecase,
    adapter: IntelligenceAssistDraftAdapter,
    events: Vec<AssistDraftWorkerEvent>,
}

impl AssistDraftWorker {
    pub fn new(adapter: IntelligenceAssistDraftAdapter) -> Self {
        Self {
            assist_draft_usecase: IntelligenceAssistDraftUsecase::default(),
            adapter,
            events: Vec::new(),
        }
    }

    pub fn run_once(&mut self, job: AssistDraftWorkerJob) -> AssistDraftWorkerReceipt {
        if let Err(evidence_ref) = validate_job(&job) {
            return receipt_from_job(
                &job,
                AssistDraftWorkerStatus::Denied,
                Some(AssistDraftWorkerDenialKind::InvalidJob),
                None,
                ExecutorRefs::default(),
                None,
                None,
                vec![evidence_ref],
            );
        }

        if job.now_epoch_seconds < job.not_before_epoch_seconds {
            return receipt_from_job(
                &job,
                AssistDraftWorkerStatus::Deferred,
                None,
                None,
                ExecutorRefs::default(),
                None,
                Some(job.not_before_epoch_seconds),
                vec!["assist-draft-worker:deferred:not-before".to_owned()],
            );
        }

        self.record_event(
            AssistDraftWorkerEventKind::JobAccepted,
            &job,
            canonical_request_evidence_refs(&job),
        );

        let usecase_receipt = self.assist_draft_usecase.plan(job.input.clone());
        if usecase_receipt.status != AssistDraftUsecaseStatus::Planned {
            let receipt = receipt_from_usecase_denial(&job, &usecase_receipt);
            self.record_event(
                AssistDraftWorkerEventKind::AssistDraftDenied,
                &job,
                receipt.evidence_refs.clone(),
            );
            return receipt;
        }

        match self.adapter.dispatch(AssistDraftExecutorDispatchRequest {
            idempotency_key: job.input.idempotency_key.clone(),
            domain_request: job.input.request.clone(),
            usecase_receipt,
        }) {
            Ok(executor_receipt) => {
                let (status, event_kind) = match executor_receipt.status {
                    AssistDraftExecutorDispatchStatus::Accepted => (
                        AssistDraftWorkerStatus::ExecutorAccepted,
                        AssistDraftWorkerEventKind::ExecutorAccepted,
                    ),
                    AssistDraftExecutorDispatchStatus::Queued => (
                        AssistDraftWorkerStatus::ExecutorQueued,
                        AssistDraftWorkerEventKind::ExecutorQueued,
                    ),
                    AssistDraftExecutorDispatchStatus::Completed => (
                        AssistDraftWorkerStatus::ExecutorCompleted,
                        AssistDraftWorkerEventKind::ExecutorCompleted,
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
                        draft_ref: executor_receipt.draft_ref,
                        queue_ref: executor_receipt.queue_ref,
                        suggested_patch_ref: executor_receipt.suggested_patch_ref,
                    },
                    Some(evidence_ref),
                    None,
                    worker_success_evidence_refs(&job),
                );
                self.record_event(event_kind, &job, receipt.evidence_refs.clone());
                receipt
            }
            Err(failure) => self.receipt_from_executor_failure(&job, failure),
        }
    }

    pub fn events(&self) -> &[AssistDraftWorkerEvent] {
        &self.events
    }

    pub fn assist_draft_usecase_cached_receipt_count(&self) -> usize {
        self.assist_draft_usecase.cached_receipt_count()
    }

    pub fn adapter_last_envelope(&self) -> Option<&AssistDraftExecutorRequestEnvelope> {
        self.adapter.last_envelope()
    }

    pub fn set_executor_status(&mut self, status: AssistDraftExecutorStatus) {
        self.adapter.set_next_status(status);
    }

    fn receipt_from_executor_failure(
        &mut self,
        job: &AssistDraftWorkerJob,
        failure: AssistDraftExecutorDispatchFailure,
    ) -> AssistDraftWorkerReceipt {
        if is_retryable_executor_failure(&failure.reason) {
            if job.attempt_number < job.max_attempts {
                let next_attempt = job
                    .now_epoch_seconds
                    .saturating_add(retry_backoff_seconds(job.attempt_number));
                let receipt = receipt_from_job(
                    job,
                    AssistDraftWorkerStatus::RetryScheduled,
                    None,
                    None,
                    ExecutorRefs::default(),
                    Some(failure.evidence_ref),
                    Some(next_attempt),
                    vec!["assist-draft-worker:executor:retry-scheduled".to_owned()],
                );
                self.record_event(
                    AssistDraftWorkerEventKind::RetryScheduled,
                    job,
                    receipt.evidence_refs.clone(),
                );
                return receipt;
            }
            let receipt = receipt_from_job(
                job,
                AssistDraftWorkerStatus::Exhausted,
                Some(AssistDraftWorkerDenialKind::RetryExhausted),
                None,
                ExecutorRefs::default(),
                Some(failure.evidence_ref),
                None,
                vec!["assist-draft-worker:executor:retry-exhausted".to_owned()],
            );
            self.record_event(
                AssistDraftWorkerEventKind::RetryExhausted,
                job,
                receipt.evidence_refs.clone(),
            );
            return receipt;
        }

        let denial_kind = if is_invalid_executor_failure(&failure.reason) {
            AssistDraftWorkerDenialKind::ExecutorInvalidRequest
        } else {
            AssistDraftWorkerDenialKind::ExecutorDenied
        };
        let receipt = receipt_from_job(
            job,
            AssistDraftWorkerStatus::Denied,
            Some(denial_kind),
            None,
            ExecutorRefs::default(),
            Some(failure.evidence_ref),
            None,
            vec!["assist-draft-worker:executor:denied".to_owned()],
        );
        self.record_event(
            AssistDraftWorkerEventKind::JobDenied,
            job,
            receipt.evidence_refs.clone(),
        );
        receipt
    }

    fn record_event(
        &mut self,
        kind: AssistDraftWorkerEventKind,
        job: &AssistDraftWorkerJob,
        evidence_refs: Vec<String>,
    ) {
        self.events.push(AssistDraftWorkerEvent {
            kind,
            job_id: safe_metadata(&job.job_id, "redacted-invalid-job-id"),
            attempt_id: safe_metadata(&job.attempt_id, "redacted-invalid-attempt-id"),
            idempotency_key: safe_metadata(
                &job.input.idempotency_key,
                "redacted-invalid-idempotency-key",
            ),
            tenant_id: safe_ref(
                &job.input.request.request.tenant_id,
                "redacted-invalid-tenant-id",
            ),
            target_builder_ref: safe_ref(
                &job.input.request.request.target_builder_ref,
                "redacted-invalid-target-builder-ref",
            ),
            evidence_refs: sorted_unique(evidence_refs),
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn receipt_from_job(
    job: &AssistDraftWorkerJob,
    status: AssistDraftWorkerStatus,
    denial_kind: Option<AssistDraftWorkerDenialKind>,
    executor_status: Option<AssistDraftExecutorDispatchStatus>,
    executor_refs: ExecutorRefs,
    executor_evidence_ref: Option<String>,
    next_attempt_epoch_seconds: Option<u64>,
    evidence_refs: Vec<String>,
) -> AssistDraftWorkerReceipt {
    AssistDraftWorkerReceipt {
        job_id: safe_metadata(&job.job_id, "redacted-invalid-job-id"),
        attempt_id: safe_metadata(&job.attempt_id, "redacted-invalid-attempt-id"),
        idempotency_key: safe_metadata(
            &job.input.idempotency_key,
            "redacted-invalid-idempotency-key",
        ),
        tenant_id: safe_ref(
            &job.input.request.request.tenant_id,
            "redacted-invalid-tenant-id",
        ),
        principal_id: safe_ref(
            &job.input.request.principal_id,
            "redacted-invalid-principal-id",
        ),
        brand_surface_ref: safe_ref(
            &job.input.request.brand_surface_ref,
            "redacted-invalid-brand-surface-ref",
        ),
        target_builder_ref: safe_ref(
            &job.input.request.request.target_builder_ref,
            "redacted-invalid-target-builder-ref",
        ),
        output_contract_ref: safe_ref(
            &job.input.request.request.output_contract_ref,
            "redacted-invalid-output-contract-ref",
        ),
        status,
        denial_kind,
        executor_status,
        executor_request_ref: executor_refs
            .executor_request_ref
            .map(|value| safe_ref(&value, "assist-draft-worker:redacted-executor-request-ref")),
        draft_ref: executor_refs
            .draft_ref
            .map(|value| safe_ref(&value, "assist-draft-worker:redacted-draft-ref")),
        queue_ref: executor_refs
            .queue_ref
            .map(|value| safe_ref(&value, "assist-draft-worker:redacted-queue-ref")),
        suggested_patch_ref: executor_refs
            .suggested_patch_ref
            .map(|value| safe_ref(&value, "assist-draft-worker:redacted-suggested-patch-ref")),
        executor_evidence_ref: executor_evidence_ref
            .map(|value| safe_ref(&value, "assist-draft-worker:redacted-executor-evidence-ref")),
        next_attempt_epoch_seconds,
        evidence_refs: sorted_unique(evidence_refs),
    }
}

fn receipt_from_usecase_denial(
    job: &AssistDraftWorkerJob,
    usecase_receipt: &AssistDraftUsecaseReceipt,
) -> AssistDraftWorkerReceipt {
    receipt_from_job(
        job,
        AssistDraftWorkerStatus::Denied,
        Some(AssistDraftWorkerDenialKind::AssistDraftUsecaseDenied),
        None,
        ExecutorRefs::default(),
        None,
        None,
        sorted_unique(
            [
                usecase_receipt.evidence_refs.clone(),
                vec!["assist-draft-worker:usecase-denied".to_owned()],
            ]
            .concat(),
        ),
    )
}

fn validate_job(job: &AssistDraftWorkerJob) -> Result<(), String> {
    require_metadata(&job.job_id, "validation:assist-draft-worker-job-id")?;
    require_metadata(&job.lease_id, "validation:assist-draft-worker-lease-id")?;
    require_metadata(&job.attempt_id, "validation:assist-draft-worker-attempt-id")?;
    if job.attempt_number == 0
        || job.max_attempts == 0
        || job.max_attempts > MAX_WORKER_ATTEMPTS
        || job.attempt_number > job.max_attempts
    {
        return Err("validation:assist-draft-worker-attempt-bounds".to_owned());
    }
    validate_input(&job.input)
}

fn validate_input(input: &AssistDraftUsecaseInput) -> Result<(), String> {
    require_metadata(
        &input.idempotency_key,
        "validation:assist-draft-worker-idempotency-key",
    )?;
    let domain = &input.request;
    let request = &domain.request;
    let policy = &domain.policy_decision;

    require_opaque(
        &domain.principal_id,
        "validation:assist-draft-worker-principal",
    )?;
    require_opaque(
        &domain.brand_surface_ref,
        "validation:assist-draft-worker-brand-surface",
    )?;
    require_metadata(&domain.locale, "validation:assist-draft-worker-locale")?;
    if domain.prompt_context_refs.len() > MAX_PROMPT_CONTEXT_REFS {
        return Err("validation:assist-draft-worker-prompt-context-limit".to_owned());
    }
    for prompt_context_ref in &domain.prompt_context_refs {
        require_opaque(
            prompt_context_ref,
            "validation:assist-draft-worker-prompt-context-ref",
        )?;
    }

    require_metadata(
        &policy.decision_id,
        "validation:assist-draft-worker-policy-decision-id",
    )?;
    require_opaque(
        &policy.tenant_id,
        "validation:assist-draft-worker-policy-tenant",
    )?;
    require_opaque(
        &policy.principal_id,
        "validation:assist-draft-worker-policy-principal",
    )?;
    require_opaque(
        &policy.evidence_ref,
        "validation:assist-draft-worker-policy-evidence",
    )?;
    require_opaque(
        &policy.prompt_registry_snapshot_ref,
        "validation:assist-draft-worker-prompt-registry",
    )?;
    require_opaque(
        &policy.cost_floor_disclosure_ref,
        "validation:assist-draft-worker-cost-floor",
    )?;
    require_opaque(
        &policy.builder_capability_scope_ref,
        "validation:assist-draft-worker-builder-scope",
    )?;
    if policy.allowed_builder_surfaces.is_empty()
        || policy.allowed_draft_kinds.is_empty()
        || policy.allowed_audiences.is_empty()
        || policy.allowed_data_classes.is_empty()
        || policy.allowed_actions.is_empty()
        || policy.allowed_locales.is_empty()
        || policy.max_prompt_context_refs == 0
        || policy.max_prompt_context_refs > MAX_PROMPT_CONTEXT_REFS
    {
        return Err("validation:assist-draft-worker-policy-allowlists".to_owned());
    }
    for locale in &policy.allowed_locales {
        require_metadata(locale, "validation:assist-draft-worker-policy-locale")?;
    }

    require_opaque(&request.tenant_id, "validation:assist-draft-worker-tenant")?;
    require_opaque(
        &request.principal_id,
        "validation:assist-draft-worker-request-principal",
    )?;
    require_opaque(
        &request.context_id,
        "validation:assist-draft-worker-context",
    )?;
    require_opaque(
        &request.prompt_ref,
        "validation:assist-draft-worker-prompt-ref",
    )?;
    require_opaque(
        &request.target_builder_ref,
        "validation:assist-draft-worker-target-builder",
    )?;
    require_opaque(
        &request.output_contract_ref,
        "validation:assist-draft-worker-output-contract",
    )?;
    require_opaque(
        &request.consent_grant_ref,
        "validation:assist-draft-worker-consent",
    )?;
    require_opaque(
        &request.budget_evidence_ref,
        "validation:assist-draft-worker-budget",
    )?;
    require_opaque(
        &request.policy_decision_ref,
        "validation:assist-draft-worker-policy-decision-ref",
    )?;
    require_opaque(
        &request.model_route_ref,
        "validation:assist-draft-worker-model-route",
    )?;
    require_opaque(
        &request.guardrail_evidence_ref,
        "validation:assist-draft-worker-guardrail",
    )?;
    require_opaque(
        &request.request_evidence_ref,
        "validation:assist-draft-worker-request-evidence",
    )?;
    require_opaque(
        &request.trace_context_ref,
        "validation:assist-draft-worker-trace-context",
    )?;
    if request.data_classes.is_empty() || request.requested_actions.is_empty() {
        return Err("validation:assist-draft-worker-request-shape".to_owned());
    }
    for evidence_ref in &request.additional_evidence_refs {
        require_opaque(
            evidence_ref,
            "validation:assist-draft-worker-additional-evidence-ref",
        )?;
    }
    Ok(())
}

fn worker_success_evidence_refs(job: &AssistDraftWorkerJob) -> Vec<String> {
    let mut refs = canonical_request_evidence_refs(job);
    refs.push("assist-draft-worker:executor-dispatched".to_owned());
    sorted_unique(refs)
}

fn canonical_request_evidence_refs(job: &AssistDraftWorkerJob) -> Vec<String> {
    let request = &job.input.request.request;
    let policy = &job.input.request.policy_decision;
    sorted_unique(vec![
        request.consent_grant_ref.clone(),
        request.budget_evidence_ref.clone(),
        request.request_evidence_ref.clone(),
        request.trace_context_ref.clone(),
        request.policy_decision_ref.clone(),
        request.model_route_ref.clone(),
        request.guardrail_evidence_ref.clone(),
        policy.evidence_ref.clone(),
        policy.prompt_registry_snapshot_ref.clone(),
        policy.cost_floor_disclosure_ref.clone(),
        policy.builder_capability_scope_ref.clone(),
    ])
}

fn is_retryable_executor_failure(reason: &str) -> bool {
    let lower = reason.to_ascii_lowercase();
    lower.contains("rate limited") || lower.contains("timed out") || lower.contains("failed")
}

fn is_invalid_executor_failure(reason: &str) -> bool {
    let lower = reason.to_ascii_lowercase();
    lower.contains("invalid") || lower.contains("requires") || lower.contains("planned usecase")
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
        || lower.contains("password=")
        || lower.contains("token=")
}

fn contains_raw_content_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw output")
        || lower.contains("raw model answer")
        || lower.contains("customer message")
        || lower.contains("write an email")
        || lower.contains("model answer")
        || lower.contains("document text")
        || lower.contains("document=")
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
    fn processes_authorized_job_and_dispatches_assist_draft_executor_envelope() {
        let mut worker = valid_worker(AssistDraftExecutorStatus::Accepted {
            executor_request_ref: "draft-request:worker:1".to_owned(),
            draft_ref: "draft:worker:1".to_owned(),
            evidence_ref: "executor-evidence:assist-draft:accepted".to_owned(),
        });

        let receipt = worker.run_once(valid_job());

        assert_eq!(receipt.status, AssistDraftWorkerStatus::ExecutorAccepted);
        assert_eq!(
            receipt.executor_status,
            Some(AssistDraftExecutorDispatchStatus::Accepted)
        );
        assert_eq!(worker.assist_draft_usecase_cached_receipt_count(), 1);
        assert_eq!(worker.events().len(), 2);
        assert_eq!(
            worker.events()[0].kind,
            AssistDraftWorkerEventKind::JobAccepted
        );
        assert_eq!(
            worker.events()[1].kind,
            AssistDraftWorkerEventKind::ExecutorAccepted
        );
        let envelope = worker.adapter_last_envelope().expect("adapter envelope");
        assert_eq!(envelope.method, AssistDraftExecutorHttpMethod::Post);
        assert_eq!(envelope.tenant_id, "tenant:alpha");
        assert_eq!(
            envelope.target_builder_ref,
            "builder://workflow-studio/canvas-1"
        );
    }

    #[test]
    fn defers_not_before_jobs_without_usecase_or_adapter_side_effects() {
        let mut worker = valid_worker(AssistDraftExecutorStatus::Queued {
            executor_request_ref: "draft-request:worker:1".to_owned(),
            queue_ref: "queue:assist-draft:1".to_owned(),
            evidence_ref: "executor-evidence:assist-draft:queued".to_owned(),
        });
        let mut job = valid_job();
        job.now_epoch_seconds = 100;
        job.not_before_epoch_seconds = 130;

        let receipt = worker.run_once(job);

        assert_eq!(receipt.status, AssistDraftWorkerStatus::Deferred);
        assert_eq!(receipt.next_attempt_epoch_seconds, Some(130));
        assert_eq!(worker.assist_draft_usecase_cached_receipt_count(), 0);
        assert!(worker.adapter_last_envelope().is_none());
        assert!(worker.events().is_empty());
    }

    #[test]
    fn invalid_raw_job_metadata_denies_before_side_effects() {
        let mut worker = valid_worker(AssistDraftExecutorStatus::Accepted {
            executor_request_ref: "draft-request:worker:1".to_owned(),
            draft_ref: "draft:worker:1".to_owned(),
            evidence_ref: "executor-evidence:assist-draft:accepted".to_owned(),
        });
        let mut job = valid_job();
        job.job_id = "job raw prompt: write an email".to_owned();
        job.input.request.request.prompt_ref =
            "raw prompt: write an email with sk-secret".to_owned();
        job.input.idempotency_key = "sk-unsafe".to_owned();

        let receipt = worker.run_once(job);

        assert_eq!(receipt.status, AssistDraftWorkerStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(AssistDraftWorkerDenialKind::InvalidJob)
        );
        assert_eq!(
            receipt.target_builder_ref,
            "builder://workflow-studio/canvas-1"
        );
        assert_eq!(worker.assist_draft_usecase_cached_receipt_count(), 0);
        assert!(worker.adapter_last_envelope().is_none());
        assert!(worker.events().is_empty());
    }

    #[test]
    fn usecase_denial_does_not_call_executor_adapter() {
        let mut worker = valid_worker(AssistDraftExecutorStatus::Accepted {
            executor_request_ref: "draft-request:worker:1".to_owned(),
            draft_ref: "draft:worker:1".to_owned(),
            evidence_ref: "executor-evidence:assist-draft:accepted".to_owned(),
        });
        let mut job = valid_job();
        job.input.request.policy_decision.allowed_draft_kinds = vec![AssistDraftKind::SearchQuery];

        let receipt = worker.run_once(job);

        assert_eq!(receipt.status, AssistDraftWorkerStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(AssistDraftWorkerDenialKind::AssistDraftUsecaseDenied)
        );
        assert!(worker.adapter_last_envelope().is_none());
        assert_eq!(
            worker.events()[1].kind,
            AssistDraftWorkerEventKind::AssistDraftDenied
        );
    }

    #[test]
    fn retryable_executor_failure_schedules_backoff_and_exhausts_at_max_attempts() {
        let mut worker = valid_worker(AssistDraftExecutorStatus::RateLimited {
            evidence_ref: "executor-evidence:assist-draft:rate-limited".to_owned(),
        });
        let mut job = valid_job();
        job.now_epoch_seconds = 1_000;
        job.not_before_epoch_seconds = 900;
        job.attempt_number = 2;
        job.max_attempts = 4;

        let retry = worker.run_once(job.clone());

        assert_eq!(retry.status, AssistDraftWorkerStatus::RetryScheduled);
        assert_eq!(retry.next_attempt_epoch_seconds, Some(1_060));
        assert_eq!(
            worker.events()[1].kind,
            AssistDraftWorkerEventKind::RetryScheduled
        );

        worker.set_executor_status(AssistDraftExecutorStatus::Timeout {
            evidence_ref: "executor-evidence:assist-draft:timeout".to_owned(),
        });
        job.attempt_number = 4;
        let exhausted = worker.run_once(job);

        assert_eq!(exhausted.status, AssistDraftWorkerStatus::Exhausted);
        assert_eq!(
            exhausted.denial_kind,
            Some(AssistDraftWorkerDenialKind::RetryExhausted)
        );
    }

    #[test]
    fn nonretryable_executor_invalid_request_denies_without_retry() {
        let mut worker = valid_worker(AssistDraftExecutorStatus::InvalidRequest {
            evidence_ref: "executor-evidence:assist-draft:invalid".to_owned(),
        });

        let receipt = worker.run_once(valid_job());

        assert_eq!(receipt.status, AssistDraftWorkerStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(AssistDraftWorkerDenialKind::ExecutorInvalidRequest)
        );
        assert_eq!(receipt.next_attempt_epoch_seconds, None);
        assert_eq!(
            worker.events()[1].kind,
            AssistDraftWorkerEventKind::JobDenied
        );
    }

    #[test]
    fn worker_debug_and_receipts_never_contain_raw_prompt_output_document_or_secret_bytes() {
        let mut worker = valid_worker(AssistDraftExecutorStatus::Completed {
            executor_request_ref: "draft-request:worker:1".to_owned(),
            draft_ref: "draft:worker:1".to_owned(),
            suggested_patch_ref: "suggested-patch:worker:1".to_owned(),
            evidence_ref: "executor-evidence:assist-draft:completed".to_owned(),
        });

        let receipt = worker.run_once(valid_job());
        let debug = format!(
            "{:?}{:?}{:?}",
            receipt,
            worker.events(),
            worker.adapter_last_envelope()
        );

        assert!(!debug.contains("sk-test"));
        assert!(!debug.contains("write an email"));
        assert!(!debug.contains("raw model answer"));
        assert!(!debug.contains("raw prompt"));
        assert!(!debug.contains("raw output"));
        assert!(!debug.contains("document="));
    }

    fn valid_worker(status: AssistDraftExecutorStatus) -> AssistDraftWorker {
        AssistDraftWorker::new(
            IntelligenceAssistDraftAdapter::try_new(
                AssistDraftExecutorAdapterConfig::new(
                    "https://assist-draft-executor.internal",
                    "credential-handle:assist-draft:1",
                    "audit-tap:assist-draft:1",
                    "draft-executor:assist-draft:workflow-studio",
                ),
                status,
            )
            .expect("valid adapter"),
        )
    }

    fn valid_job() -> AssistDraftWorkerJob {
        AssistDraftWorkerJob {
            job_id: "job:assist-draft-worker:1".to_owned(),
            lease_id: "lease:assist-draft-worker:1".to_owned(),
            attempt_id: "attempt:assist-draft-worker:1".to_owned(),
            attempt_number: 1,
            max_attempts: 4,
            now_epoch_seconds: 1_000,
            not_before_epoch_seconds: 900,
            input: AssistDraftUsecaseInput {
                idempotency_key: "idempotency:assist-draft-worker:1".to_owned(),
                request: valid_domain_request(),
            },
        }
    }

    fn valid_domain_request() -> DomainAssistDraftRequest {
        DomainAssistDraftRequest {
            principal_id: "principal:builder-owner".to_owned(),
            brand_surface_ref: "brand-surface:workflow-studio:assist".to_owned(),
            locale: "en-US".to_owned(),
            prompt_context_refs: vec!["context-snippet:workflow-studio:canvas-1".to_owned()],
            policy_decision: AssistDraftPolicyDecision {
                decision_id: "decision:assist-draft:1".to_owned(),
                tenant_id: "tenant:alpha".to_owned(),
                principal_id: "principal:builder-owner".to_owned(),
                ai_assist_enabled: true,
                explicit_automation_allowed: false,
                allowed_builder_surfaces: vec![AssistDraftBuilderSurface::WorkflowStudio],
                allowed_draft_kinds: vec![AssistDraftKind::WorkflowDraft],
                allowed_audiences: vec![AssistDraftAudience::TenantBuilder],
                allowed_data_classes: vec![
                    AssistDraftDataClass::Internal,
                    AssistDraftDataClass::Public,
                ],
                allowed_actions: vec![
                    AssistDraftAction::CreateDraft,
                    AssistDraftAction::ExplainDraft,
                ],
                allowed_locales: vec!["en-US".to_owned()],
                max_prompt_context_refs: 4,
                evidence_ref: "policy:assist-draft:allow".to_owned(),
                prompt_registry_snapshot_ref: "prompt-registry:assist-draft:v1".to_owned(),
                cost_floor_disclosure_ref: "cost-floor:assist-draft:workflow-studio".to_owned(),
                builder_capability_scope_ref: "builder-scope:workflow-studio:draft".to_owned(),
            },
            request: AssistDraftRequest {
                tenant_id: "tenant:alpha".to_owned(),
                principal_id: "principal:builder-owner".to_owned(),
                context_id: "context://workflow-studio/canvas-1".to_owned(),
                builder_surface: AssistDraftBuilderSurface::WorkflowStudio,
                draft_kind: AssistDraftKind::WorkflowDraft,
                audience: AssistDraftAudience::TenantBuilder,
                invocation_mode: AssistDraftInvocationMode::UserInvoked,
                review_gate: AssistDraftReviewGate::HumanReviewRequired,
                prompt_ref: "prompt://assist-draft/req-1".to_owned(),
                target_builder_ref: "builder://workflow-studio/canvas-1".to_owned(),
                output_contract_ref: "workflow-spec://contracts/v1".to_owned(),
                consent_grant_ref: "consent:assist-draft:1".to_owned(),
                budget_evidence_ref: "budget:assist-draft:1".to_owned(),
                policy_decision_ref: "policy:assist-draft:allow".to_owned(),
                model_route_ref: "model-route:assist-draft:1".to_owned(),
                guardrail_evidence_ref: "guardrail:assist-draft:allow".to_owned(),
                request_evidence_ref: "request:assist-draft:1".to_owned(),
                trace_context_ref: "trace:assist-draft:1".to_owned(),
                data_classes: vec![AssistDraftDataClass::Internal, AssistDraftDataClass::Public],
                requested_actions: vec![
                    AssistDraftAction::CreateDraft,
                    AssistDraftAction::ExplainDraft,
                ],
                additional_evidence_refs: Vec::new(),
            },
        }
    }
}
