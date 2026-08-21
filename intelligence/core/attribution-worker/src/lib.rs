//! Intelligence attribution worker foundation.
//!
//! This crate provides a deterministic source-level worker seam for future
//! citation rendering jobs. It validates queued job metadata, runs the
//! attribution usecase, and hands rendered receipts to the metadata-only
//! citation renderer adapter. It performs no queue I/O, network I/O, citation
//! text rendering, retrieval execution, filesystem access, durable idempotency,
//! durable audit-chain emission, or cloud runtime scheduling.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use intelligence_attribution_adapter::{
    AttributionAudience, AttributionClaim, AttributionDataClass, AttributionDenialKind,
    AttributionDomainDenialKind, AttributionPolicyDecision, AttributionRendererAdapterConfig,
    AttributionRendererAdapterConfigError, AttributionRendererDispatchFailure,
    AttributionRendererDispatchRequest, AttributionRendererDispatchStatus,
    AttributionRendererHttpMethod, AttributionRendererRequestEnvelope, AttributionRendererStatus,
    AttributionRendererTransportMode, AttributionRequest, AttributionSource, AttributionSourceKind,
    AttributionUsecaseDenialKind, AttributionUsecaseReceipt, AttributionUsecaseStatus,
    DomainAttributionRequest, IntelligenceAttributionAdapter,
};
pub use intelligence_attribution_usecase::{
    AttributionAuditEvent, AttributionAuditEventKind, AttributionUsecaseInput,
    IntelligenceAttributionUsecase,
};

const MAX_WORKER_ATTEMPTS: u32 = 10;
const BASE_RETRY_BACKOFF_SECONDS: u64 = 30;
const MAX_RETRY_BACKOFF_SECONDS: u64 = 900;
const MAX_WORKER_CITATIONS: usize = 64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributionWorkerJob {
    pub job_id: String,                 // data_class: INTERNAL_ONLY
    pub lease_id: String,               // data_class: INTERNAL_ONLY
    pub attempt_id: String,             // data_class: INTERNAL_ONLY
    pub attempt_number: u32,            // data_class: INTERNAL_ONLY
    pub max_attempts: u32,              // data_class: INTERNAL_ONLY
    pub now_epoch_seconds: u64,         // data_class: INTERNAL_ONLY
    pub not_before_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
    pub input: AttributionUsecaseInput, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttributionWorkerStatus {
    Deferred,
    Denied,
    Exhausted,
    RetryScheduled,
    RendererAccepted,
    RendererCompleted,
    RendererQueued,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttributionWorkerDenialKind {
    AttributionUsecaseDenied,
    InvalidJob,
    RendererDenied,
    RendererInvalidRequest,
    RetryExhausted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributionWorkerReceipt {
    pub job_id: String,                                   // data_class: INTERNAL_ONLY
    pub attempt_id: String,                               // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                          // data_class: INTERNAL_ONLY
    pub tenant_id: String,                                // data_class: INTERNAL_ONLY
    pub principal_id: String,                             // data_class: INTERNAL_ONLY
    pub attribution_surface: String,                      // data_class: INTERNAL_ONLY
    pub output_ref: String,                               // data_class: INTERNAL_ONLY
    pub status: AttributionWorkerStatus,                  // data_class: PUBLIC
    pub denial_kind: Option<AttributionWorkerDenialKind>, // data_class: INTERNAL_ONLY
    pub renderer_status: Option<AttributionRendererDispatchStatus>, // data_class: INTERNAL_ONLY
    pub renderer_request_ref: Option<String>,             // data_class: INTERNAL_ONLY
    pub render_ref: Option<String>,                       // data_class: INTERNAL_ONLY
    pub queue_ref: Option<String>,                        // data_class: INTERNAL_ONLY
    pub citation_bundle_ref: Option<String>,              // data_class: INTERNAL_ONLY
    pub renderer_evidence_ref: Option<String>,            // data_class: INTERNAL_ONLY
    pub next_attempt_epoch_seconds: Option<u64>,          // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttributionWorkerEventKind {
    AttributionDenied,
    JobAccepted,
    JobDenied,
    RendererAccepted,
    RendererCompleted,
    RendererQueued,
    RetryExhausted,
    RetryScheduled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributionWorkerEvent {
    pub kind: AttributionWorkerEventKind, // data_class: INTERNAL_ONLY
    pub job_id: String,                   // data_class: INTERNAL_ONLY
    pub attempt_id: String,               // data_class: INTERNAL_ONLY
    pub idempotency_key: String,          // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub attribution_surface: String,      // data_class: INTERNAL_ONLY
    pub output_ref: String,               // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,       // data_class: INTERNAL_ONLY
}

pub struct AttributionWorker {
    attribution_usecase: IntelligenceAttributionUsecase,
    adapter: IntelligenceAttributionAdapter,
    events: Vec<AttributionWorkerEvent>,
}

impl AttributionWorker {
    pub fn new(adapter: IntelligenceAttributionAdapter) -> Self {
        Self {
            attribution_usecase: IntelligenceAttributionUsecase::default(),
            adapter,
            events: Vec::new(),
        }
    }

    pub fn run_once(&mut self, job: AttributionWorkerJob) -> AttributionWorkerReceipt {
        if let Err(evidence_ref) = validate_job(&job) {
            return receipt_from_job(
                &job,
                AttributionWorkerStatus::Denied,
                Some(AttributionWorkerDenialKind::InvalidJob),
                None,
                RendererRefs::default(),
                None,
                Some(evidence_ref.clone()),
                vec![evidence_ref],
            );
        }

        if job.now_epoch_seconds < job.not_before_epoch_seconds {
            return receipt_from_job(
                &job,
                AttributionWorkerStatus::Deferred,
                None,
                None,
                RendererRefs::default(),
                Some(job.not_before_epoch_seconds),
                None,
                vec!["attribution-worker:deferred:not-before".to_owned()],
            );
        }

        self.record_event(
            AttributionWorkerEventKind::JobAccepted,
            &job,
            canonical_request_evidence_refs(&job),
        );

        let attribution_receipt = self.attribution_usecase.plan(job.input.clone());
        if attribution_receipt.status != AttributionUsecaseStatus::Rendered {
            let receipt = receipt_from_attribution_denial(&job, &attribution_receipt);
            self.record_event(
                AttributionWorkerEventKind::AttributionDenied,
                &job,
                receipt.evidence_refs.clone(),
            );
            return receipt;
        }

        match self.adapter.dispatch(AttributionRendererDispatchRequest {
            idempotency_key: job.input.idempotency_key.clone(),
            domain_request: job.input.request.clone(),
            usecase_receipt: attribution_receipt,
        }) {
            Ok(renderer_receipt) => {
                let (status, event_kind) = match renderer_receipt.status {
                    AttributionRendererDispatchStatus::Accepted => (
                        AttributionWorkerStatus::RendererAccepted,
                        AttributionWorkerEventKind::RendererAccepted,
                    ),
                    AttributionRendererDispatchStatus::Queued => (
                        AttributionWorkerStatus::RendererQueued,
                        AttributionWorkerEventKind::RendererQueued,
                    ),
                    AttributionRendererDispatchStatus::Completed => (
                        AttributionWorkerStatus::RendererCompleted,
                        AttributionWorkerEventKind::RendererCompleted,
                    ),
                };
                let evidence_ref = renderer_receipt.evidence_ref.clone();
                let receipt = receipt_from_job(
                    &job,
                    status,
                    None,
                    Some(renderer_receipt.status),
                    RendererRefs {
                        renderer_request_ref: renderer_receipt.renderer_request_ref,
                        render_ref: renderer_receipt.render_ref,
                        queue_ref: renderer_receipt.queue_ref,
                        citation_bundle_ref: renderer_receipt.citation_bundle_ref,
                    },
                    None,
                    Some(evidence_ref),
                    worker_success_evidence_refs(&job),
                );
                self.record_event(event_kind, &job, receipt.evidence_refs.clone());
                receipt
            }
            Err(failure) => self.receipt_from_renderer_failure(&job, failure),
        }
    }

    pub fn events(&self) -> &[AttributionWorkerEvent] {
        &self.events
    }

    pub fn attribution_usecase_cached_receipt_count(&self) -> usize {
        self.attribution_usecase.cached_receipt_count()
    }

    pub fn adapter_last_envelope(&self) -> Option<&AttributionRendererRequestEnvelope> {
        self.adapter.last_envelope()
    }

    pub fn set_renderer_status(&mut self, status: AttributionRendererStatus) {
        self.adapter.set_next_status(status);
    }

    fn receipt_from_renderer_failure(
        &mut self,
        job: &AttributionWorkerJob,
        failure: AttributionRendererDispatchFailure,
    ) -> AttributionWorkerReceipt {
        if is_retryable_renderer_failure(&failure.reason) {
            if job.attempt_number < job.max_attempts {
                let next_attempt = job
                    .now_epoch_seconds
                    .saturating_add(retry_backoff_seconds(job.attempt_number));
                let receipt = receipt_from_job(
                    job,
                    AttributionWorkerStatus::RetryScheduled,
                    None,
                    None,
                    RendererRefs::default(),
                    Some(next_attempt),
                    Some(failure.evidence_ref),
                    vec!["attribution-worker:renderer:retry-scheduled".to_owned()],
                );
                self.record_event(
                    AttributionWorkerEventKind::RetryScheduled,
                    job,
                    receipt.evidence_refs.clone(),
                );
                return receipt;
            }
            let receipt = receipt_from_job(
                job,
                AttributionWorkerStatus::Exhausted,
                Some(AttributionWorkerDenialKind::RetryExhausted),
                None,
                RendererRefs::default(),
                None,
                Some(failure.evidence_ref),
                vec!["attribution-worker:renderer:retry-exhausted".to_owned()],
            );
            self.record_event(
                AttributionWorkerEventKind::RetryExhausted,
                job,
                receipt.evidence_refs.clone(),
            );
            return receipt;
        }

        let denial_kind = if failure.reason == "citation-renderer:invalid_request" {
            AttributionWorkerDenialKind::RendererInvalidRequest
        } else {
            AttributionWorkerDenialKind::RendererDenied
        };
        let receipt = receipt_from_job(
            job,
            AttributionWorkerStatus::Denied,
            Some(denial_kind),
            None,
            RendererRefs::default(),
            None,
            Some(failure.evidence_ref),
            vec!["attribution-worker:renderer:denied".to_owned()],
        );
        self.record_event(
            AttributionWorkerEventKind::JobDenied,
            job,
            receipt.evidence_refs.clone(),
        );
        receipt
    }

    fn record_event(
        &mut self,
        kind: AttributionWorkerEventKind,
        job: &AttributionWorkerJob,
        evidence_refs: Vec<String>,
    ) {
        self.events.push(AttributionWorkerEvent {
            kind,
            job_id: safe_metadata(&job.job_id, "redacted-invalid-job-id"),
            attempt_id: safe_metadata(&job.attempt_id, "redacted-invalid-attempt-id"),
            idempotency_key: safe_metadata(
                &job.input.idempotency_key,
                "redacted-invalid-idempotency-key",
            ),
            tenant_id: safe_ref(&job.input.request.tenant_id, "redacted-invalid-tenant-id"),
            attribution_surface: safe_ref(
                &job.input.request.attribution_surface,
                "redacted-invalid-attribution-surface",
            ),
            output_ref: safe_ref(
                &job.input.request.request.output_ref,
                "redacted-invalid-output-ref",
            ),
            evidence_refs: sorted_unique(evidence_refs),
        });
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct RendererRefs {
    renderer_request_ref: Option<String>,
    render_ref: Option<String>,
    queue_ref: Option<String>,
    citation_bundle_ref: Option<String>,
}

fn validate_job(job: &AttributionWorkerJob) -> Result<(), String> {
    require_metadata(&job.job_id, "validation:attribution-worker-job-id")?;
    require_metadata(&job.lease_id, "validation:attribution-worker-lease-id")?;
    require_metadata(&job.attempt_id, "validation:attribution-worker-attempt-id")?;
    if job.attempt_number == 0
        || job.max_attempts == 0
        || job.max_attempts > MAX_WORKER_ATTEMPTS
        || job.attempt_number > job.max_attempts
    {
        return Err("validation:attribution-worker-attempt-bounds".to_owned());
    }
    validate_input(&job.input)
}

fn validate_input(input: &AttributionUsecaseInput) -> Result<(), String> {
    require_metadata(
        &input.idempotency_key,
        "validation:attribution-worker-idempotency-key",
    )?;
    let domain = &input.request;
    let kernel = &domain.request;
    let policy = &domain.policy_decision;

    require_opaque(&domain.tenant_id, "validation:attribution-worker-tenant")?;
    require_opaque(
        &domain.principal_id,
        "validation:attribution-worker-principal",
    )?;
    require_opaque(
        &domain.attribution_surface,
        "validation:attribution-worker-surface",
    )?;
    require_opaque(
        &domain.request_evidence_ref,
        "validation:attribution-worker-request-evidence",
    )?;
    require_opaque(
        &domain.trace_context_ref,
        "validation:attribution-worker-trace-context",
    )?;
    require_opaque(
        &domain.policy_decision_ref,
        "validation:attribution-worker-policy-decision",
    )?;

    require_metadata(
        &policy.decision_id,
        "validation:attribution-worker-policy-id",
    )?;
    require_opaque(
        &policy.tenant_id,
        "validation:attribution-worker-policy-tenant",
    )?;
    require_opaque(
        &policy.principal_id,
        "validation:attribution-worker-policy-principal",
    )?;
    if policy.allowed_surfaces.is_empty()
        || policy.allowed_audiences.is_empty()
        || policy.allowed_source_kinds.is_empty()
        || policy.allowed_data_classes.is_empty()
    {
        return Err("validation:attribution-worker-policy-allowlists".to_owned());
    }
    for surface in &policy.allowed_surfaces {
        require_opaque(surface, "validation:attribution-worker-policy-surface")?;
    }
    if policy.max_citations == 0 || policy.max_citations > MAX_WORKER_CITATIONS {
        return Err("validation:attribution-worker-policy-max-citations".to_owned());
    }
    if policy.min_confidence_bps > 10_000 {
        return Err("validation:attribution-worker-policy-confidence".to_owned());
    }
    require_opaque(
        &policy.evidence_ref,
        "validation:attribution-worker-policy-evidence",
    )?;
    require_opaque(
        &policy.attribution_registry_snapshot_ref,
        "validation:attribution-worker-registry-snapshot",
    )?;

    require_opaque(
        &kernel.tenant_id,
        "validation:attribution-worker-kernel-tenant",
    )?;
    require_opaque(
        &kernel.output_ref,
        "validation:attribution-worker-output-ref",
    )?;
    require_opaque(
        &kernel.policy_evidence_ref,
        "validation:attribution-worker-kernel-policy-evidence",
    )?;
    require_opaque(
        &kernel.trace_context_ref,
        "validation:attribution-worker-kernel-trace-context",
    )?;
    if kernel.max_citations == 0 || kernel.max_citations > MAX_WORKER_CITATIONS {
        return Err("validation:attribution-worker-max-citations".to_owned());
    }
    if kernel.sources.is_empty() || kernel.claims.is_empty() {
        return Err("validation:attribution-worker-citation-metadata".to_owned());
    }
    for source in &kernel.sources {
        require_metadata(&source.source_id, "validation:attribution-worker-source-id")?;
        require_opaque(
            &source.resource_ref,
            "validation:attribution-worker-source-resource-ref",
        )?;
        require_opaque(
            &source.title_ref,
            "validation:attribution-worker-source-title-ref",
        )?;
        require_opaque(
            &source.evidence_ref,
            "validation:attribution-worker-source-evidence-ref",
        )?;
    }
    for claim in &kernel.claims {
        require_metadata(&claim.claim_id, "validation:attribution-worker-claim-id")?;
        require_opaque(
            &claim.answer_segment_ref,
            "validation:attribution-worker-claim-segment-ref",
        )?;
        if claim.source_ids.is_empty() || claim.confidence_bps > 10_000 {
            return Err("validation:attribution-worker-claim-metadata".to_owned());
        }
        for source_id in &claim.source_ids {
            require_metadata(source_id, "validation:attribution-worker-claim-source-id")?;
        }
    }
    Ok(())
}

fn receipt_from_attribution_denial(
    job: &AttributionWorkerJob,
    attribution_receipt: &AttributionUsecaseReceipt,
) -> AttributionWorkerReceipt {
    receipt_from_job(
        job,
        AttributionWorkerStatus::Denied,
        Some(AttributionWorkerDenialKind::AttributionUsecaseDenied),
        None,
        RendererRefs::default(),
        None,
        None,
        sorted_unique(
            [
                attribution_receipt.evidence_refs.clone(),
                vec!["attribution-worker:attribution-usecase-denied".to_owned()],
            ]
            .concat(),
        ),
    )
}

#[allow(clippy::too_many_arguments)]
fn receipt_from_job(
    job: &AttributionWorkerJob,
    status: AttributionWorkerStatus,
    denial_kind: Option<AttributionWorkerDenialKind>,
    renderer_status: Option<AttributionRendererDispatchStatus>,
    renderer_refs: RendererRefs,
    next_attempt_epoch_seconds: Option<u64>,
    renderer_evidence_ref: Option<String>,
    evidence_refs: Vec<String>,
) -> AttributionWorkerReceipt {
    AttributionWorkerReceipt {
        job_id: safe_metadata(&job.job_id, "redacted-invalid-job-id"),
        attempt_id: safe_metadata(&job.attempt_id, "redacted-invalid-attempt-id"),
        idempotency_key: safe_metadata(
            &job.input.idempotency_key,
            "redacted-invalid-idempotency-key",
        ),
        tenant_id: safe_ref(&job.input.request.tenant_id, "redacted-invalid-tenant-id"),
        principal_id: safe_ref(
            &job.input.request.principal_id,
            "redacted-invalid-principal-id",
        ),
        attribution_surface: safe_ref(
            &job.input.request.attribution_surface,
            "redacted-invalid-attribution-surface",
        ),
        output_ref: safe_ref(
            &job.input.request.request.output_ref,
            "redacted-invalid-output-ref",
        ),
        status,
        denial_kind,
        renderer_status,
        renderer_request_ref: renderer_refs
            .renderer_request_ref
            .map(|value| safe_ref(&value, "attribution-worker:redacted-renderer-request-ref")),
        render_ref: renderer_refs
            .render_ref
            .map(|value| safe_ref(&value, "attribution-worker:redacted-render-ref")),
        queue_ref: renderer_refs
            .queue_ref
            .map(|value| safe_ref(&value, "attribution-worker:redacted-queue-ref")),
        citation_bundle_ref: renderer_refs
            .citation_bundle_ref
            .map(|value| safe_ref(&value, "attribution-worker:redacted-citation-bundle-ref")),
        renderer_evidence_ref: renderer_evidence_ref
            .map(|value| safe_ref(&value, "attribution-worker:redacted-renderer-evidence-ref")),
        next_attempt_epoch_seconds,
        evidence_refs: sorted_unique(evidence_refs),
    }
}

fn worker_success_evidence_refs(job: &AttributionWorkerJob) -> Vec<String> {
    let mut refs = canonical_request_evidence_refs(job);
    refs.push("attribution-worker:renderer-dispatched".to_owned());
    sorted_unique(refs)
}

fn canonical_request_evidence_refs(job: &AttributionWorkerJob) -> Vec<String> {
    sorted_unique(vec![
        job.input.request.request_evidence_ref.clone(),
        job.input.request.trace_context_ref.clone(),
        job.input.request.policy_decision_ref.clone(),
        job.input.request.policy_decision.evidence_ref.clone(),
        job.input
            .request
            .policy_decision
            .attribution_registry_snapshot_ref
            .clone(),
        job.input.request.request.policy_evidence_ref.clone(),
        job.input.request.request.trace_context_ref.clone(),
    ])
}

fn is_retryable_renderer_failure(reason: &str) -> bool {
    matches!(
        reason,
        "citation-renderer:rate_limited"
            | "citation-renderer:renderer_error"
            | "citation-renderer:timeout"
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
        || lower.contains("raw query")
        || lower.contains("customer message")
        || lower.contains("write an email")
        || lower.contains("model answer")
        || lower.contains("prompt=")
        || lower.contains("completion=")
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
    fn processes_authorized_job_and_dispatches_renderer_envelope() {
        let mut worker = valid_worker(AttributionRendererStatus::Accepted {
            renderer_request_ref: "citation-renderer://requests/req-1".to_owned(),
            render_ref: "citation-render://renders/render-1".to_owned(),
            evidence_ref: "citation-renderer:evidence:accepted".to_owned(),
        });

        let receipt = worker.run_once(valid_job());

        assert_eq!(receipt.status, AttributionWorkerStatus::RendererAccepted);
        assert_eq!(
            receipt.renderer_status,
            Some(AttributionRendererDispatchStatus::Accepted)
        );
        assert_eq!(worker.attribution_usecase_cached_receipt_count(), 1);
        assert_eq!(worker.events().len(), 2);
        assert_eq!(
            worker.events()[0].kind,
            AttributionWorkerEventKind::JobAccepted
        );
        assert_eq!(
            worker.events()[1].kind,
            AttributionWorkerEventKind::RendererAccepted
        );
        let envelope = worker.adapter_last_envelope().expect("adapter envelope");
        assert_eq!(envelope.method, AttributionRendererHttpMethod::Post);
        assert_eq!(envelope.output_ref, "answer://responses/resp-worker-1");
        assert_eq!(envelope.citation_count, 2);
    }

    #[test]
    fn defers_not_before_jobs_without_usecase_or_adapter_side_effects() {
        let mut worker = valid_worker(AttributionRendererStatus::Queued {
            renderer_request_ref: "citation-renderer://requests/req-1".to_owned(),
            queue_ref: "citation-renderer://queues/q-1".to_owned(),
            evidence_ref: "citation-renderer:evidence:queued".to_owned(),
        });
        let mut job = valid_job();
        job.now_epoch_seconds = 100;
        job.not_before_epoch_seconds = 130;

        let receipt = worker.run_once(job);

        assert_eq!(receipt.status, AttributionWorkerStatus::Deferred);
        assert_eq!(receipt.next_attempt_epoch_seconds, Some(130));
        assert_eq!(worker.attribution_usecase_cached_receipt_count(), 0);
        assert!(worker.adapter_last_envelope().is_none());
        assert!(worker.events().is_empty());
    }

    #[test]
    fn usecase_denial_does_not_call_renderer_adapter() {
        let mut worker = valid_worker(AttributionRendererStatus::Completed {
            renderer_request_ref: "citation-renderer://requests/req-1".to_owned(),
            render_ref: "citation-render://renders/render-1".to_owned(),
            citation_bundle_ref: "citation-bundle://bundles/b-1".to_owned(),
            evidence_ref: "citation-renderer:evidence:completed".to_owned(),
        });
        let mut job = valid_job();
        job.input.request.request.claims[0].source_ids = vec!["missing-source".to_owned()];

        let receipt = worker.run_once(job);

        assert_eq!(receipt.status, AttributionWorkerStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(AttributionWorkerDenialKind::AttributionUsecaseDenied)
        );
        assert!(worker.adapter_last_envelope().is_none());
        assert_eq!(worker.events().len(), 2);
        assert_eq!(
            worker.events()[1].kind,
            AttributionWorkerEventKind::AttributionDenied
        );
    }

    #[test]
    fn retryable_renderer_failure_schedules_backoff_and_exhausts_at_max_attempts() {
        let mut worker = valid_worker(AttributionRendererStatus::RateLimited {
            evidence_ref: "citation-renderer:evidence:rate-limited".to_owned(),
        });
        let mut job = valid_job();
        job.now_epoch_seconds = 1_000;
        job.attempt_number = 2;
        job.max_attempts = 4;

        let receipt = worker.run_once(job.clone());

        assert_eq!(receipt.status, AttributionWorkerStatus::RetryScheduled);
        assert_eq!(receipt.next_attempt_epoch_seconds, Some(1_060));
        assert_eq!(
            worker.events()[1].kind,
            AttributionWorkerEventKind::RetryScheduled
        );

        worker.set_renderer_status(AttributionRendererStatus::Timeout {
            evidence_ref: "citation-renderer:evidence:timeout".to_owned(),
        });
        job.attempt_number = 4;
        let exhausted = worker.run_once(job);

        assert_eq!(exhausted.status, AttributionWorkerStatus::Exhausted);
        assert_eq!(
            exhausted.denial_kind,
            Some(AttributionWorkerDenialKind::RetryExhausted)
        );
    }

    #[test]
    fn nonretryable_renderer_invalid_request_denies_without_retry() {
        let mut worker = valid_worker(AttributionRendererStatus::InvalidRequest {
            evidence_ref: "citation-renderer:evidence:invalid-request".to_owned(),
        });

        let receipt = worker.run_once(valid_job());

        assert_eq!(receipt.status, AttributionWorkerStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(AttributionWorkerDenialKind::RendererInvalidRequest)
        );
        assert_eq!(receipt.next_attempt_epoch_seconds, None);
        assert_eq!(
            worker.events()[1].kind,
            AttributionWorkerEventKind::JobDenied
        );
    }

    #[test]
    fn invalid_raw_job_metadata_denies_before_side_effects() {
        let mut worker = valid_worker(AttributionRendererStatus::Accepted {
            renderer_request_ref: "citation-renderer://requests/req-1".to_owned(),
            render_ref: "citation-render://renders/render-1".to_owned(),
            evidence_ref: "citation-renderer:evidence:accepted".to_owned(),
        });
        let mut job = valid_job();
        job.job_id = "job raw prompt: customer message".to_owned();
        job.input.request.request.output_ref = "raw output: full answer text".to_owned();
        job.input.idempotency_key = "sk-unsafe".to_owned();

        let receipt = worker.run_once(job);

        assert_eq!(receipt.status, AttributionWorkerStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(AttributionWorkerDenialKind::InvalidJob)
        );
        assert_eq!(receipt.job_id, "redacted-invalid-job-id");
        assert_eq!(receipt.idempotency_key, "redacted-invalid-idempotency-key");
        assert_eq!(worker.attribution_usecase_cached_receipt_count(), 0);
        assert!(worker.adapter_last_envelope().is_none());
        assert!(worker.events().is_empty());
    }

    #[test]
    fn worker_debug_and_receipts_never_contain_raw_prompt_output_document_or_secret_bytes() {
        let mut worker = valid_worker(AttributionRendererStatus::Denied {
            evidence_ref: "citation-renderer:evidence:denied".to_owned(),
        });
        let mut job = valid_job();
        job.input.request.request.sources[0].resource_ref =
            "document raw prompt sk-test".to_owned();

        let receipt = worker.run_once(job);

        let debug = format!("{receipt:?} {:?}", worker.events());
        assert!(!debug.contains("raw prompt"));
        assert!(!debug.contains("sk-test"));
        assert!(!debug.contains("customer message"));
        assert_eq!(receipt.status, AttributionWorkerStatus::Denied);
    }

    fn valid_worker(status: AttributionRendererStatus) -> AttributionWorker {
        AttributionWorker::new(
            IntelligenceAttributionAdapter::try_new(
                AttributionRendererAdapterConfig::new(
                    "https://citation-renderer.oyatie.internal/",
                    "secretref://ten_a/citation-renderer/byok",
                    "audit://tap/intelligence/attribution",
                    "audience://intelligence/citation-renderer",
                ),
                status,
            )
            .expect("valid adapter config"),
        )
    }

    fn valid_job() -> AttributionWorkerJob {
        AttributionWorkerJob {
            job_id: "attr-worker-job-1".to_owned(),
            lease_id: "attr-worker-lease-1".to_owned(),
            attempt_id: "attr-worker-attempt-1".to_owned(),
            attempt_number: 1,
            max_attempts: 4,
            now_epoch_seconds: 1_000,
            not_before_epoch_seconds: 900,
            input: sample_input(),
        }
    }

    fn sample_input() -> AttributionUsecaseInput {
        AttributionUsecaseInput {
            idempotency_key: "idem:attribution-worker:1".to_owned(),
            request: sample_domain_request(),
        }
    }

    fn sample_domain_request() -> DomainAttributionRequest {
        DomainAttributionRequest {
            tenant_id: "tenant:alpha".to_owned(),
            principal_id: "principal:attribution-worker".to_owned(),
            attribution_surface: "surface:dispatch-response".to_owned(),
            request_evidence_ref: "request:evidence:attribution-worker:1".to_owned(),
            trace_context_ref: "trace:attribution-worker:1".to_owned(),
            policy_decision_ref: "policy:evidence:attribution-worker:1".to_owned(),
            policy_decision: sample_policy(),
            request: sample_kernel_request(),
        }
    }

    fn sample_policy() -> AttributionPolicyDecision {
        AttributionPolicyDecision {
            decision_id: "attribution-policy-decision:worker:1".to_owned(),
            tenant_id: "tenant:alpha".to_owned(),
            principal_id: "principal:attribution-worker".to_owned(),
            allowed_surfaces: vec!["surface:dispatch-response".to_owned()],
            allowed_audiences: vec![AttributionAudience::External, AttributionAudience::Internal],
            allowed_source_kinds: vec![
                AttributionSourceKind::KnowledgeGraph,
                AttributionSourceKind::PolicyDocument,
                AttributionSourceKind::RetrievalDocument,
            ],
            allowed_data_classes: vec![
                AttributionDataClass::Public,
                AttributionDataClass::Internal,
            ],
            max_citations: 8,
            min_confidence_bps: 7_000,
            evidence_ref: "policy:evidence:attribution-worker:1".to_owned(),
            attribution_registry_snapshot_ref: "attribution-registry:snapshot:worker:1".to_owned(),
        }
    }

    fn sample_kernel_request() -> AttributionRequest {
        AttributionRequest {
            tenant_id: "tenant:alpha".to_owned(),
            output_ref: "answer://responses/resp-worker-1".to_owned(),
            audience: AttributionAudience::External,
            policy_evidence_ref: "policy:evidence:attribution-worker:1".to_owned(),
            trace_context_ref: "trace:attribution-worker:1".to_owned(),
            max_citations: 8,
            max_citations_per_claim: 8,
            sources: vec![
                AttributionSource {
                    source_id: "src-kg-policy".to_owned(),
                    resource_ref: "kg://entity/accounting-policy".to_owned(),
                    title_ref: "title://knowledge/accounting-policy".to_owned(),
                    source_kind: AttributionSourceKind::KnowledgeGraph,
                    data_class: AttributionDataClass::Public,
                    evidence_ref: "evidence:kg:accounting-policy".to_owned(),
                    freshness_epoch_seconds: 1_779_523_200,
                },
                AttributionSource {
                    source_id: "src-doc-refund".to_owned(),
                    resource_ref: "doc://help-center/refund-policy".to_owned(),
                    title_ref: "title://help/refund-policy".to_owned(),
                    source_kind: AttributionSourceKind::RetrievalDocument,
                    data_class: AttributionDataClass::Public,
                    evidence_ref: "evidence:doc:refund-policy".to_owned(),
                    freshness_epoch_seconds: 1_779_523_201,
                },
            ],
            claims: vec![
                AttributionClaim {
                    claim_id: "claim-2".to_owned(),
                    answer_segment_ref: "answer-segment://resp-worker-1/2".to_owned(),
                    source_ids: vec!["src-doc-refund".to_owned()],
                    confidence_bps: 9_000,
                },
                AttributionClaim {
                    claim_id: "claim-1".to_owned(),
                    answer_segment_ref: "answer-segment://resp-worker-1/1".to_owned(),
                    source_ids: vec!["src-kg-policy".to_owned()],
                    confidence_bps: 9_200,
                },
            ],
        }
    }
}
