//! Workflow-engine event-bus worker seam foundation.
//!
//! This crate provides a deterministic source-level worker seam for future
//! event-bus job processing. It validates leased job metadata, honors not-before
//! scheduling and lease expiry before API side effects, invokes the event-bus API
//! boundary in-process for publish and delivery-evaluation jobs, preserves API
//! idempotent replay/conflict behavior, maps publish, delivery-accepted,
//! delivery-denied, invalid, domain-denied, and conflicting outcomes into
//! metadata-only worker receipts/events, and models capped deterministic retry
//! scheduling for retryable API unavailability. It performs no durable queue
//! polling, Valkey lease I/O, broker connection, topic creation, network I/O,
//! serialization-framework work, durable idempotency storage, durable
//! outbox/inbox writes, consumer group coordination, offset commits, payload
//! materialization, signing, Kubernetes calls, cloud deployment, or tenant
//! workload scheduling.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use workflow_event_bus_api::{
    WORKFLOW_EVENT_BUS_API_DECLARED_VERSION, WORKFLOW_EVENT_BUS_API_DELIVERY_ROUTE,
    WORKFLOW_EVENT_BUS_API_METHOD, WORKFLOW_EVENT_BUS_API_PUBLISH_ROUTE,
    WORKFLOW_EVENT_BUS_API_SURFACE, WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION,
    WORKFLOW_EVENT_BUS_MAX_SUBSCRIPTION_BATCH_SIZE, WorkflowEventBusApi,
    WorkflowEventBusApiAuthorization, WorkflowEventBusApiBoundaryContext,
    WorkflowEventBusApiDeliveryBody, WorkflowEventBusApiDeliveryRequest, WorkflowEventBusApiError,
    WorkflowEventBusApiErrorCode, WorkflowEventBusApiPrincipal, WorkflowEventBusApiProblemDetails,
    WorkflowEventBusApiPublishBody, WorkflowEventBusApiPublishRequest, WorkflowEventBusApiStatus,
    WorkflowEventBusApiSuccessResponse, WorkflowEventBusEventKind,
};

pub const WORKFLOW_EVENT_BUS_WORKER_SURFACE: &str = "workflow-engine.event-bus.worker";
pub const WORKFLOW_EVENT_BUS_WORKER_MAX_ATTEMPTS: u32 = 10;
pub const WORKFLOW_EVENT_BUS_WORKER_BASE_BACKOFF_SECONDS: u64 = 30;
pub const WORKFLOW_EVENT_BUS_WORKER_MAX_BACKOFF_SECONDS: u64 = 900;
pub const WORKFLOW_EVENT_BUS_WORKER_DEFAULT_RESUME_LIMIT: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowEventBusWorkerJobKind {
    Publish,
    EvaluateDelivery,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowEventBusWorkerJobBody {
    Publish(Box<WorkflowEventBusApiPublishRequest>),
    Delivery(Box<WorkflowEventBusApiDeliveryRequest>),
}

impl WorkflowEventBusWorkerJobBody {
    pub const fn kind(&self) -> WorkflowEventBusWorkerJobKind {
        match self {
            Self::Publish(_) => WorkflowEventBusWorkerJobKind::Publish,
            Self::Delivery(_) => WorkflowEventBusWorkerJobKind::EvaluateDelivery,
        }
    }

    pub fn tenant_id(&self) -> &str {
        match self {
            Self::Publish(request) => &request.boundary.tenant_id,
            Self::Delivery(request) => &request.boundary.tenant_id,
        }
    }

    pub fn idempotency_key(&self) -> &str {
        match self {
            Self::Publish(request) => &request.boundary.idempotency_key,
            Self::Delivery(request) => &request.boundary.idempotency_key,
        }
    }

    pub fn event_type_hint(&self) -> &str {
        match self {
            Self::Publish(request) => &request.body.event_kind,
            Self::Delivery(request) => &request.body.candidate_event_type,
        }
    }

    pub fn channel_hint(&self) -> Option<&str> {
        match self {
            Self::Publish(_) => None,
            Self::Delivery(request) => Some(&request.body.candidate_channel),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusWorkerJob {
    pub job_id: String,                      // data_class: INTERNAL_ONLY
    pub lease_id: String,                    // data_class: INTERNAL_ONLY
    pub worker_ref: String,                  // data_class: INTERNAL_ONLY
    pub attempt_id: String,                  // data_class: INTERNAL_ONLY
    pub attempt_number: u32,                 // data_class: INTERNAL_ONLY
    pub max_attempts: u32,                   // data_class: INTERNAL_ONLY
    pub now_epoch_seconds: u64,              // data_class: INTERNAL_ONLY
    pub not_before_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
    pub lease_expires_epoch_seconds: u64,    // data_class: INTERNAL_ONLY
    pub body: WorkflowEventBusWorkerJobBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowEventBusWorkerStatus {
    DeliveryAccepted,
    DeliveryDenied,
    Denied,
    Deferred,
    Published,
    RetryExhausted,
    RetryScheduled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowEventBusWorkerDenialKind {
    ApiDenied,
    IdempotencyConflict,
    InvalidJob,
    LeaseExpired,
    RetryExhausted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowEventBusWorkerEventKind {
    ApiApplied,
    JobAccepted,
    JobDeferred,
    JobDenied,
    LeaseExpired,
    RetryExhausted,
    RetryScheduled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusWorkerReceipt {
    pub job_id: String,                          // data_class: INTERNAL_ONLY
    pub lease_id: String,                        // data_class: INTERNAL_ONLY
    pub worker_ref: String,                      // data_class: INTERNAL_ONLY
    pub attempt_id: String,                      // data_class: INTERNAL_ONLY
    pub tenant_id: String,                       // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                 // data_class: INTERNAL_ONLY
    pub job_kind: WorkflowEventBusWorkerJobKind, // data_class: PUBLIC
    pub status: WorkflowEventBusWorkerStatus,    // data_class: PUBLIC
    pub denial_kind: Option<WorkflowEventBusWorkerDenialKind>, // data_class: INTERNAL_ONLY
    pub api_status: Option<WorkflowEventBusApiStatus>, // data_class: PUBLIC
    pub api_usecase_status: Option<String>,      // data_class: PUBLIC
    pub event_type: String,                      // data_class: PUBLIC
    pub channel_address: Option<String>,         // data_class: PUBLIC
    pub consumer_ref: Option<String>,            // data_class: INTERNAL_ONLY
    pub offset_ref: Option<String>,              // data_class: INTERNAL_ONLY
    pub offset_commit_planned: bool,             // data_class: PUBLIC
    pub retry_delay_seconds: Option<u64>,        // data_class: INTERNAL_ONLY
    pub next_attempt_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,              // data_class: INTERNAL_ONLY
    pub non_claim_refs: Vec<String>,             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkflowEventBusWorkerReceiptParts {
    status: WorkflowEventBusWorkerStatus,
    denial_kind: Option<WorkflowEventBusWorkerDenialKind>,
    api_status: Option<WorkflowEventBusApiStatus>,
    api_usecase_status: Option<String>,
    channel_address: Option<String>,
    consumer_ref: Option<String>,
    offset_ref: Option<String>,
    offset_commit_planned: bool,
    retry_delay_seconds: Option<u64>,
    next_attempt_epoch_seconds: Option<u64>,
    evidence_refs: Vec<String>,
    non_claim_refs: Vec<String>,
}

impl WorkflowEventBusWorkerReceiptParts {
    fn new(status: WorkflowEventBusWorkerStatus, evidence_refs: Vec<String>) -> Self {
        Self {
            status,
            denial_kind: None,
            api_status: None,
            api_usecase_status: None,
            channel_address: None,
            consumer_ref: None,
            offset_ref: None,
            offset_commit_planned: false,
            retry_delay_seconds: None,
            next_attempt_epoch_seconds: None,
            evidence_refs,
            non_claim_refs: worker_non_claim_refs(Vec::new()),
        }
    }

    fn with_denial_kind(mut self, denial_kind: WorkflowEventBusWorkerDenialKind) -> Self {
        self.denial_kind = Some(denial_kind);
        self
    }

    fn with_api_status(mut self, api_status: WorkflowEventBusApiStatus) -> Self {
        self.api_status = Some(api_status);
        self
    }

    fn with_api_usecase_status(mut self, status: impl Into<String>) -> Self {
        self.api_usecase_status = Some(status.into());
        self
    }

    fn with_delivery_refs(
        mut self,
        channel_address: Option<String>,
        consumer_ref: Option<String>,
        offset_ref: Option<String>,
    ) -> Self {
        self.channel_address = channel_address;
        self.consumer_ref = consumer_ref;
        self.offset_ref = offset_ref;
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

    fn with_non_claims(mut self, non_claim_refs: Vec<String>) -> Self {
        self.non_claim_refs = worker_non_claim_refs(non_claim_refs);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusWorkerEvent {
    pub kind: WorkflowEventBusWorkerEventKind, // data_class: INTERNAL_ONLY
    pub job_id: String,                        // data_class: INTERNAL_ONLY
    pub lease_id: String,                      // data_class: INTERNAL_ONLY
    pub worker_ref: String,                    // data_class: INTERNAL_ONLY
    pub attempt_id: String,                    // data_class: INTERNAL_ONLY
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub idempotency_key: String,               // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusWorkerResumeCandidate {
    pub tenant_id: String,           // data_class: INTERNAL_ONLY
    pub channel_address: String,     // data_class: PUBLIC
    pub event_type: String,          // data_class: PUBLIC
    pub event_id: String,            // data_class: INTERNAL_ONLY
    pub due_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
    pub resume_priority: u32,        // data_class: INTERNAL_ONLY
    pub resume_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusWorkerResumePlan {
    pub accepted: Vec<WorkflowEventBusWorkerResumeCandidate>, // data_class: INTERNAL_ONLY
    pub deferred: Vec<WorkflowEventBusWorkerResumeCandidate>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusWorkerResumeThrottle {
    pub max_resumes_per_tick: usize, // data_class: INTERNAL_ONLY
}

impl Default for WorkflowEventBusWorkerResumeThrottle {
    fn default() -> Self {
        Self {
            max_resumes_per_tick: WORKFLOW_EVENT_BUS_WORKER_DEFAULT_RESUME_LIMIT,
        }
    }
}

#[derive(Default)]
pub struct WorkflowEventBusWorker {
    api: WorkflowEventBusApi,
    events: Vec<WorkflowEventBusWorkerEvent>,
    api_apply_count: usize,
}

impl WorkflowEventBusWorker {
    pub fn new(api: WorkflowEventBusApi) -> Self {
        Self {
            api,
            events: Vec::new(),
            api_apply_count: 0,
        }
    }

    pub fn run_once(&mut self, job: WorkflowEventBusWorkerJob) -> WorkflowEventBusWorkerReceipt {
        if let Err(evidence_ref) = validate_job(&job) {
            let receipt = receipt_from_job(
                &job,
                WorkflowEventBusWorkerReceiptParts::new(
                    WorkflowEventBusWorkerStatus::Denied,
                    vec![evidence_ref],
                )
                .with_denial_kind(WorkflowEventBusWorkerDenialKind::InvalidJob),
            );
            self.record_event(
                WorkflowEventBusWorkerEventKind::JobDenied,
                &job,
                receipt.evidence_refs.clone(),
            );
            return receipt;
        }

        if job.now_epoch_seconds < job.not_before_epoch_seconds {
            let receipt = receipt_from_job(
                &job,
                WorkflowEventBusWorkerReceiptParts::new(
                    WorkflowEventBusWorkerStatus::Deferred,
                    vec!["workflow-event-bus-worker:deferred:not-before".to_owned()],
                )
                .with_next_attempt_epoch_seconds(job.not_before_epoch_seconds),
            );
            self.record_event(
                WorkflowEventBusWorkerEventKind::JobDeferred,
                &job,
                receipt.evidence_refs.clone(),
            );
            return receipt;
        }

        if job.lease_expires_epoch_seconds <= job.now_epoch_seconds {
            let receipt = receipt_from_job(
                &job,
                WorkflowEventBusWorkerReceiptParts::new(
                    WorkflowEventBusWorkerStatus::Denied,
                    vec!["workflow-event-bus-worker:lease-expired".to_owned()],
                )
                .with_denial_kind(WorkflowEventBusWorkerDenialKind::LeaseExpired),
            );
            self.record_event(
                WorkflowEventBusWorkerEventKind::LeaseExpired,
                &job,
                receipt.evidence_refs.clone(),
            );
            return receipt;
        }

        if job.attempt_number > job.max_attempts {
            let receipt = retry_exhausted_receipt(
                &job,
                vec!["workflow-event-bus-worker:attempts-exhausted-before-api".to_owned()],
            );
            self.record_event(
                WorkflowEventBusWorkerEventKind::RetryExhausted,
                &job,
                receipt.evidence_refs.clone(),
            );
            return receipt;
        }

        self.record_event(
            WorkflowEventBusWorkerEventKind::JobAccepted,
            &job,
            sorted_unique(vec![
                job.job_id.clone(),
                job.lease_id.clone(),
                job.attempt_id.clone(),
                WORKFLOW_EVENT_BUS_WORKER_SURFACE.to_owned(),
                "workflow-event-bus-worker:job-accepted".to_owned(),
            ]),
        );

        self.api_apply_count += 1;
        match apply_job_body(&mut self.api, job.body.clone()) {
            Ok(success) => {
                let receipt = receipt_from_success(&job, success);
                self.record_event(
                    WorkflowEventBusWorkerEventKind::ApiApplied,
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

    pub fn events(&self) -> &[WorkflowEventBusWorkerEvent] {
        &self.events
    }

    fn record_terminal_error_event(
        &mut self,
        job: &WorkflowEventBusWorkerJob,
        receipt: &WorkflowEventBusWorkerReceipt,
    ) {
        let kind = match receipt.status {
            WorkflowEventBusWorkerStatus::RetryExhausted => {
                WorkflowEventBusWorkerEventKind::RetryExhausted
            }
            WorkflowEventBusWorkerStatus::RetryScheduled => {
                WorkflowEventBusWorkerEventKind::RetryScheduled
            }
            _ => WorkflowEventBusWorkerEventKind::JobDenied,
        };
        self.record_event(kind, job, receipt.evidence_refs.clone());
    }

    fn record_event(
        &mut self,
        kind: WorkflowEventBusWorkerEventKind,
        job: &WorkflowEventBusWorkerJob,
        evidence_refs: Vec<String>,
    ) {
        self.events.push(WorkflowEventBusWorkerEvent {
            kind,
            job_id: safe_ref_or_redacted(&job.job_id, "redacted:event-bus-worker-job"),
            lease_id: safe_ref_or_redacted(&job.lease_id, "redacted:event-bus-worker-lease"),
            worker_ref: safe_ref_or_redacted(&job.worker_ref, "redacted:event-bus-worker"),
            attempt_id: safe_ref_or_redacted(&job.attempt_id, "redacted:event-bus-worker-attempt"),
            tenant_id: safe_tenant_or_redacted(job.body.tenant_id()),
            idempotency_key: safe_ref_or_redacted(
                job.body.idempotency_key(),
                "redacted:event-bus-worker-idempotency",
            ),
            evidence_refs: sorted_unique(evidence_refs),
        });
    }
}

pub fn plan_resume_candidates(
    mut candidates: Vec<WorkflowEventBusWorkerResumeCandidate>,
    throttle: WorkflowEventBusWorkerResumeThrottle,
) -> WorkflowEventBusWorkerResumePlan {
    candidates.retain(valid_resume_candidate);
    candidates.sort_by(|left, right| {
        left.resume_priority
            .cmp(&right.resume_priority)
            .then(left.due_epoch_seconds.cmp(&right.due_epoch_seconds))
            .then(left.tenant_id.cmp(&right.tenant_id))
            .then(left.channel_address.cmp(&right.channel_address))
            .then(left.event_id.cmp(&right.event_id))
    });
    let limit = throttle.max_resumes_per_tick;
    let accepted: Vec<_> = candidates.iter().take(limit).cloned().collect();
    let deferred: Vec<_> = candidates.into_iter().skip(limit).collect();
    let accepted_len = accepted.len();
    let deferred_len = deferred.len();
    WorkflowEventBusWorkerResumePlan {
        accepted,
        deferred,
        evidence_refs: sorted_unique(vec![
            WORKFLOW_EVENT_BUS_WORKER_SURFACE.to_owned(),
            format!("workflow-event-bus-worker:resume-accepted:{accepted_len}"),
            format!("workflow-event-bus-worker:resume-deferred:{deferred_len}"),
        ]),
    }
}

fn apply_job_body(
    api: &mut WorkflowEventBusApi,
    body: WorkflowEventBusWorkerJobBody,
) -> Result<WorkflowEventBusApiSuccessResponse, WorkflowEventBusApiError> {
    match body {
        WorkflowEventBusWorkerJobBody::Publish(request) => api.publish_event(*request),
        WorkflowEventBusWorkerJobBody::Delivery(request) => api.evaluate_delivery(*request),
    }
}

fn validate_job(job: &WorkflowEventBusWorkerJob) -> Result<(), String> {
    if !is_safe_ref(&job.job_id)
        || !is_safe_ref(&job.lease_id)
        || !is_safe_ref(&job.worker_ref)
        || !is_safe_ref(&job.attempt_id)
    {
        return Err("workflow-event-bus-worker:invalid-job-metadata".to_owned());
    }
    if job.attempt_number == 0 || job.max_attempts == 0 {
        return Err("workflow-event-bus-worker:invalid-attempt-bounds".to_owned());
    }
    if job.max_attempts > WORKFLOW_EVENT_BUS_WORKER_MAX_ATTEMPTS {
        return Err("workflow-event-bus-worker:max-attempts-exceeds-worker-cap".to_owned());
    }
    if job.not_before_epoch_seconds >= job.lease_expires_epoch_seconds {
        return Err("workflow-event-bus-worker:not-before-after-lease-expiry".to_owned());
    }
    Ok(())
}

fn receipt_from_success(
    job: &WorkflowEventBusWorkerJob,
    success: WorkflowEventBusApiSuccessResponse,
) -> WorkflowEventBusWorkerReceipt {
    let status = match success.event.usecase_status.as_str() {
        "published" => WorkflowEventBusWorkerStatus::Published,
        "delivery-accepted" => WorkflowEventBusWorkerStatus::DeliveryAccepted,
        "delivery-denied" => WorkflowEventBusWorkerStatus::DeliveryDenied,
        _ => WorkflowEventBusWorkerStatus::Denied,
    };
    let mut evidence_refs = success.evidence_refs.clone();
    evidence_refs.push(WORKFLOW_EVENT_BUS_WORKER_SURFACE.to_owned());
    evidence_refs.push("workflow-event-bus-worker:api-success".to_owned());
    receipt_from_job(
        job,
        WorkflowEventBusWorkerReceiptParts::new(status, evidence_refs)
            .with_api_status(success.status)
            .with_api_usecase_status(success.event.usecase_status)
            .with_delivery_refs(
                success.event.channel_address,
                success.event.consumer_ref,
                success.event.offset_ref,
            )
            .with_non_claims(success.non_claim_refs),
    )
}

fn receipt_from_api_error(
    job: &WorkflowEventBusWorkerJob,
    error: &WorkflowEventBusApiError,
) -> WorkflowEventBusWorkerReceipt {
    if error.status() == WorkflowEventBusApiStatus::ServiceUnavailable {
        return retryable_error_receipt(job, error.problem());
    }
    let problem = error.problem();
    let denial_kind = if error.code() == WorkflowEventBusApiErrorCode::IdempotencyKeyReused {
        WorkflowEventBusWorkerDenialKind::IdempotencyConflict
    } else {
        WorkflowEventBusWorkerDenialKind::ApiDenied
    };
    let mut evidence_refs = problem.evidence_refs.clone();
    evidence_refs.push(WORKFLOW_EVENT_BUS_WORKER_SURFACE.to_owned());
    evidence_refs.push(problem.code.clone());
    receipt_from_job(
        job,
        WorkflowEventBusWorkerReceiptParts::new(
            WorkflowEventBusWorkerStatus::Denied,
            evidence_refs,
        )
        .with_denial_kind(denial_kind)
        .with_api_status(error.status()),
    )
}

fn retryable_error_receipt(
    job: &WorkflowEventBusWorkerJob,
    problem: WorkflowEventBusApiProblemDetails,
) -> WorkflowEventBusWorkerReceipt {
    if job.attempt_number >= job.max_attempts {
        let mut evidence_refs = problem.evidence_refs.clone();
        evidence_refs.push(problem.code);
        evidence_refs.push("workflow-event-bus-worker:retry-exhausted".to_owned());
        return retry_exhausted_receipt(job, evidence_refs);
    }
    let retry_delay_seconds = retry_delay_seconds(job);
    let next_attempt = job.now_epoch_seconds.saturating_add(retry_delay_seconds);
    let mut evidence_refs = problem.evidence_refs;
    evidence_refs.push(problem.code);
    evidence_refs.push(WORKFLOW_EVENT_BUS_WORKER_SURFACE.to_owned());
    evidence_refs.push("workflow-event-bus-worker:retry-scheduled".to_owned());
    receipt_from_job(
        job,
        WorkflowEventBusWorkerReceiptParts::new(
            WorkflowEventBusWorkerStatus::RetryScheduled,
            evidence_refs,
        )
        .with_retry(retry_delay_seconds, next_attempt)
        .with_api_status(WorkflowEventBusApiStatus::ServiceUnavailable),
    )
}

fn retry_exhausted_receipt(
    job: &WorkflowEventBusWorkerJob,
    evidence_refs: Vec<String>,
) -> WorkflowEventBusWorkerReceipt {
    receipt_from_job(
        job,
        WorkflowEventBusWorkerReceiptParts::new(
            WorkflowEventBusWorkerStatus::RetryExhausted,
            evidence_refs,
        )
        .with_denial_kind(WorkflowEventBusWorkerDenialKind::RetryExhausted),
    )
}

fn receipt_from_job(
    job: &WorkflowEventBusWorkerJob,
    parts: WorkflowEventBusWorkerReceiptParts,
) -> WorkflowEventBusWorkerReceipt {
    WorkflowEventBusWorkerReceipt {
        job_id: safe_ref_or_redacted(&job.job_id, "redacted:event-bus-worker-job"),
        lease_id: safe_ref_or_redacted(&job.lease_id, "redacted:event-bus-worker-lease"),
        worker_ref: safe_ref_or_redacted(&job.worker_ref, "redacted:event-bus-worker"),
        attempt_id: safe_ref_or_redacted(&job.attempt_id, "redacted:event-bus-worker-attempt"),
        tenant_id: safe_tenant_or_redacted(job.body.tenant_id()),
        idempotency_key: safe_ref_or_redacted(
            job.body.idempotency_key(),
            "redacted:event-bus-worker-idempotency",
        ),
        job_kind: job.body.kind(),
        status: parts.status,
        denial_kind: parts.denial_kind,
        api_status: parts.api_status,
        api_usecase_status: parts.api_usecase_status,
        event_type: safe_metadata_or_redacted(
            job.body.event_type_hint(),
            "redacted:event-bus-event-type",
        ),
        channel_address: parts
            .channel_address
            .or_else(|| job.body.channel_hint().map(str::to_owned))
            .map(|value| safe_metadata_or_redacted(&value, "redacted:event-bus-channel")),
        consumer_ref: parts
            .consumer_ref
            .map(|value| safe_ref_or_redacted(&value, "redacted:event-bus-consumer")),
        offset_ref: parts
            .offset_ref
            .map(|value| safe_ref_or_redacted(&value, "redacted:event-bus-offset")),
        offset_commit_planned: parts.offset_commit_planned,
        retry_delay_seconds: parts.retry_delay_seconds,
        next_attempt_epoch_seconds: parts.next_attempt_epoch_seconds,
        evidence_refs: sorted_unique(parts.evidence_refs),
        non_claim_refs: parts.non_claim_refs,
    }
}

fn retry_delay_seconds(job: &WorkflowEventBusWorkerJob) -> u64 {
    let exponent = job.attempt_number.saturating_sub(1).min(5);
    let exponential = WORKFLOW_EVENT_BUS_WORKER_BASE_BACKOFF_SECONDS.saturating_mul(1 << exponent);
    let jitter = stable_jitter_seconds(&job.job_id, job.attempt_number);
    exponential
        .saturating_add(jitter)
        .min(WORKFLOW_EVENT_BUS_WORKER_MAX_BACKOFF_SECONDS)
}

fn stable_jitter_seconds(job_id: &str, attempt_number: u32) -> u64 {
    let mut acc = u64::from(attempt_number);
    for byte in job_id.bytes() {
        acc = acc.wrapping_mul(31).wrapping_add(u64::from(byte));
    }
    acc % WORKFLOW_EVENT_BUS_WORKER_BASE_BACKOFF_SECONDS
}

fn valid_resume_candidate(candidate: &WorkflowEventBusWorkerResumeCandidate) -> bool {
    is_safe_tenant(&candidate.tenant_id)
        && is_safe_metadata(&candidate.channel_address)
        && is_safe_metadata(&candidate.event_type)
        && is_safe_ref(&candidate.event_id)
        && is_safe_ref(&candidate.resume_evidence_ref)
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

fn safe_metadata_or_redacted(value: &str, redacted: &str) -> String {
    if is_safe_metadata(value) {
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

fn worker_non_claim_refs(mut upstream_non_claims: Vec<String>) -> Vec<String> {
    upstream_non_claims.extend([
        "workflow-event-bus-worker:no-durable-queue-polling".to_owned(),
        "workflow-event-bus-worker:no-valkey-lease-io".to_owned(),
        "workflow-event-bus-worker:no-broker-runtime".to_owned(),
        "workflow-event-bus-worker:no-consumer-group-runtime".to_owned(),
        "workflow-event-bus-worker:no-offset-commit-runtime".to_owned(),
        "workflow-event-bus-worker:no-cloud-deployment".to_owned(),
        "workflow-event-bus-worker:no-tenant-workload-scheduling".to_owned(),
        "workflow-event-bus-worker:no-hyperscaler-claim".to_owned(),
    ]);
    sorted_unique(upstream_non_claims)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn worker_statuses_defaults_and_retry_delay_are_stable() {
        let statuses = [
            WorkflowEventBusWorkerStatus::DeliveryAccepted,
            WorkflowEventBusWorkerStatus::DeliveryDenied,
            WorkflowEventBusWorkerStatus::Denied,
            WorkflowEventBusWorkerStatus::Deferred,
            WorkflowEventBusWorkerStatus::Published,
            WorkflowEventBusWorkerStatus::RetryExhausted,
            WorkflowEventBusWorkerStatus::RetryScheduled,
        ];
        let unique: BTreeSet<_> = statuses.iter().copied().collect();
        assert_eq!(unique.len(), statuses.len());
        assert_eq!(WORKFLOW_EVENT_BUS_WORKER_MAX_ATTEMPTS, 10);
        assert_eq!(
            WorkflowEventBusWorkerResumeThrottle::default().max_resumes_per_tick,
            WORKFLOW_EVENT_BUS_WORKER_DEFAULT_RESUME_LIMIT
        );
        let job = publish_job("idem:event-bus-worker:retry-delay");
        assert_eq!(retry_delay_seconds(&job), retry_delay_seconds(&job));
        assert!(retry_delay_seconds(&job) <= WORKFLOW_EVENT_BUS_WORKER_MAX_BACKOFF_SECONDS);
    }

    #[test]
    fn publish_job_invokes_api_and_records_metadata_only_receipt() {
        let mut worker = WorkflowEventBusWorker::default();
        let receipt = worker.run_once(publish_job("idem:event-bus-worker:publish:1"));

        assert_eq!(receipt.status, WorkflowEventBusWorkerStatus::Published);
        assert_eq!(
            receipt.api_status,
            Some(WorkflowEventBusApiStatus::Accepted)
        );
        assert_eq!(receipt.api_usecase_status.as_deref(), Some("published"));
        assert_eq!(receipt.job_kind, WorkflowEventBusWorkerJobKind::Publish);
        assert_eq!(
            receipt.channel_address.as_deref(),
            Some("workflow.runs.events.v1")
        );
        assert_eq!(worker.api_apply_count(), 1);
        assert_eq!(worker.events().len(), 2);
        assert_eq!(
            worker.events()[0].kind,
            WorkflowEventBusWorkerEventKind::JobAccepted
        );
        assert_eq!(
            worker.events()[1].kind,
            WorkflowEventBusWorkerEventKind::ApiApplied
        );
        assert!(
            receipt
                .non_claim_refs
                .contains(&"workflow-event-bus-worker:no-broker-runtime".to_owned())
        );
        assert!(!format!("{receipt:?}").contains("payload"));
    }

    #[test]
    fn delivery_jobs_preserve_accepted_and_denied_outcomes_without_offset_commit_claim() {
        let mut worker = WorkflowEventBusWorker::default();
        let accepted = worker.run_once(delivery_job("idem:event-bus-worker:delivery:1"));

        assert_eq!(
            accepted.status,
            WorkflowEventBusWorkerStatus::DeliveryAccepted
        );
        assert_eq!(
            accepted.api_usecase_status.as_deref(),
            Some("delivery-accepted")
        );
        assert_eq!(
            accepted.consumer_ref.as_deref(),
            Some("consumer:workflow-state-machine")
        );
        assert_eq!(
            accepted.offset_ref.as_deref(),
            Some("offset:partition-0:42")
        );
        assert!(!accepted.offset_commit_planned);

        let mut denied_job = delivery_job("idem:event-bus-worker:delivery-denied");
        let WorkflowEventBusWorkerJobBody::Delivery(request) = &mut denied_job.body else {
            unreachable!("expected delivery job");
        };
        request.body.candidate_channel = "workflow-runs".to_owned();
        request.body.candidate_event_type = WorkflowEventBusEventKind::WorkflowRunStarted
            .event_type()
            .to_owned();
        let denied = worker.run_once(denied_job);
        assert_eq!(denied.status, WorkflowEventBusWorkerStatus::DeliveryDenied);
        assert_eq!(
            denied.api_usecase_status.as_deref(),
            Some("delivery-denied")
        );
        assert!(!denied.offset_commit_planned);
        assert!(
            denied
                .non_claim_refs
                .contains(&"workflow-event-bus-worker:no-offset-commit-runtime".to_owned())
        );
    }

    #[test]
    fn invalid_not_before_expired_and_exhausted_jobs_never_invoke_api() {
        let mut worker = WorkflowEventBusWorker::default();
        let mut invalid = publish_job("idem:event-bus-worker:invalid");
        invalid.job_id = "job:raw prompt bearer sk-test payload".to_owned();
        let invalid_receipt = worker.run_once(invalid);
        assert_eq!(invalid_receipt.status, WorkflowEventBusWorkerStatus::Denied);
        assert_eq!(
            invalid_receipt.denial_kind,
            Some(WorkflowEventBusWorkerDenialKind::InvalidJob)
        );
        assert_eq!(worker.api_apply_count(), 0);
        assert!(!format!("{invalid_receipt:?}").contains("sk-test"));

        let mut deferred = publish_job("idem:event-bus-worker:deferred");
        deferred.not_before_epoch_seconds = 200;
        deferred.lease_expires_epoch_seconds = 260;
        let deferred_receipt = worker.run_once(deferred);
        assert_eq!(
            deferred_receipt.status,
            WorkflowEventBusWorkerStatus::Deferred
        );
        assert_eq!(deferred_receipt.next_attempt_epoch_seconds, Some(200));
        assert_eq!(worker.api_apply_count(), 0);

        let mut expired = publish_job("idem:event-bus-worker:expired");
        expired.lease_expires_epoch_seconds = expired.now_epoch_seconds;
        let expired_receipt = worker.run_once(expired);
        assert_eq!(expired_receipt.status, WorkflowEventBusWorkerStatus::Denied);
        assert_eq!(
            expired_receipt.denial_kind,
            Some(WorkflowEventBusWorkerDenialKind::LeaseExpired)
        );
        assert_eq!(worker.api_apply_count(), 0);

        let mut exhausted = publish_job("idem:event-bus-worker:exhausted");
        exhausted.attempt_number = 11;
        exhausted.max_attempts = 10;
        let exhausted_receipt = worker.run_once(exhausted);
        assert_eq!(
            exhausted_receipt.status,
            WorkflowEventBusWorkerStatus::RetryExhausted
        );
        assert_eq!(worker.api_apply_count(), 0);
    }

    #[test]
    fn api_domain_denial_maps_to_worker_denial_without_raw_echo() {
        let mut worker = WorkflowEventBusWorker::default();
        let mut job = publish_job("idem:event-bus-worker:domain-denied");
        let WorkflowEventBusWorkerJobBody::Publish(request) = &mut job.body else {
            unreachable!("expected publish job");
        };
        request.authorization.allowed_event_types = vec![
            WorkflowEventBusEventKind::WorkflowStepDispatched
                .event_type()
                .to_owned(),
        ];

        let receipt = worker.run_once(job);

        assert_eq!(receipt.status, WorkflowEventBusWorkerStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(WorkflowEventBusWorkerDenialKind::ApiDenied)
        );
        assert_eq!(
            receipt.api_status,
            Some(WorkflowEventBusApiStatus::Forbidden)
        );
        assert_eq!(worker.api_apply_count(), 1);
        let rendered = format!("{receipt:?}");
        assert!(rendered.contains(WorkflowEventBusApiErrorCode::DomainDenied.as_str()));
        assert!(!rendered.contains("raw prompt"));
        assert!(!rendered.contains("payload"));
    }

    #[test]
    fn idempotent_replay_and_conflict_are_preserved_through_worker_api() {
        let mut worker = WorkflowEventBusWorker::default();
        let job = publish_job("idem:event-bus-worker:replay");
        let first = worker.run_once(job.clone());
        let second = worker.run_once(job);

        assert_eq!(first, second);
        assert_eq!(worker.api_apply_count(), 2);

        let mut drifted = publish_job("idem:event-bus-worker:replay");
        let WorkflowEventBusWorkerJobBody::Publish(request) = &mut drifted.body else {
            unreachable!("expected publish job");
        };
        request.body.event_id = "event:workflow-run-started:drift".to_owned();
        let conflict = worker.run_once(drifted);

        assert_eq!(conflict.status, WorkflowEventBusWorkerStatus::Denied);
        assert_eq!(
            conflict.denial_kind,
            Some(WorkflowEventBusWorkerDenialKind::IdempotencyConflict)
        );
        assert_eq!(
            conflict.api_status,
            Some(WorkflowEventBusApiStatus::Conflict)
        );
        assert!(
            conflict.evidence_refs.contains(
                &WorkflowEventBusApiErrorCode::IdempotencyKeyReused
                    .as_str()
                    .to_owned()
            )
        );
    }

    #[test]
    fn unsafe_api_body_is_rejected_after_api_boundary_without_raw_echo() {
        let mut worker = WorkflowEventBusWorker::default();
        let mut job = publish_job("idem:event-bus-worker:unsafe-body");
        let WorkflowEventBusWorkerJobBody::Publish(request) = &mut job.body else {
            unreachable!("expected publish job");
        };
        request.body.payload_ref =
            "raw payload Authorization: Bearer sk-test customer message".to_owned();

        let receipt = worker.run_once(job);

        assert_eq!(receipt.status, WorkflowEventBusWorkerStatus::Denied);
        assert_eq!(
            receipt.api_status,
            Some(WorkflowEventBusApiStatus::BadRequest)
        );
        assert_eq!(worker.api_apply_count(), 1);
        let rendered = format!("{receipt:?}");
        assert!(rendered.contains(WorkflowEventBusApiErrorCode::UnsafeMetadata.as_str()));
        assert!(!rendered.contains("raw payload"));
        assert!(!rendered.contains("Authorization"));
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("customer message"));
    }

    #[test]
    fn cold_start_resume_throttle_caps_orders_and_drops_unsafe_candidates() {
        let plan = plan_resume_candidates(
            vec![
                resume_candidate("ten_b", "workflow.runs.events.v1", "event:3", 30, 2),
                resume_candidate("ten_a", "workflow.state.events.v1", "event:1", 10, 1),
                resume_candidate("ten_a", "workflow.runs.events.v1", "event:2", 20, 1),
                WorkflowEventBusWorkerResumeCandidate {
                    tenant_id: "tenant raw prompt".to_owned(),
                    channel_address: "workflow.runs.events.v1".to_owned(),
                    event_type: WorkflowEventBusEventKind::WorkflowRunStarted
                        .event_type()
                        .to_owned(),
                    event_id: "event:bad".to_owned(),
                    due_epoch_seconds: 1,
                    resume_priority: 0,
                    resume_evidence_ref: "resume:bad".to_owned(),
                },
            ],
            WorkflowEventBusWorkerResumeThrottle {
                max_resumes_per_tick: 2,
            },
        );

        assert_eq!(plan.accepted.len(), 2);
        assert_eq!(plan.deferred.len(), 1);
        assert_eq!(plan.accepted[0].event_id, "event:1");
        assert_eq!(plan.accepted[1].event_id, "event:2");
        assert!(
            plan.evidence_refs
                .contains(&"workflow-event-bus-worker:resume-accepted:2".to_owned())
        );
    }

    fn publish_job(idempotency_key: &str) -> WorkflowEventBusWorkerJob {
        WorkflowEventBusWorkerJob {
            job_id: format!("job:event-bus-worker:{idempotency_key}"),
            lease_id: format!("lease:event-bus-worker:{idempotency_key}"),
            worker_ref: "worker:event-bus:cell-a:001".to_owned(),
            attempt_id: format!("attempt:event-bus-worker:{idempotency_key}:1"),
            attempt_number: 1,
            max_attempts: WORKFLOW_EVENT_BUS_WORKER_MAX_ATTEMPTS,
            now_epoch_seconds: 100,
            not_before_epoch_seconds: 90,
            lease_expires_epoch_seconds: 180,
            body: WorkflowEventBusWorkerJobBody::Publish(Box::new(publish_request(
                idempotency_key,
            ))),
        }
    }

    fn delivery_job(idempotency_key: &str) -> WorkflowEventBusWorkerJob {
        WorkflowEventBusWorkerJob {
            job_id: format!("job:event-bus-worker:{idempotency_key}"),
            lease_id: format!("lease:event-bus-worker:{idempotency_key}"),
            worker_ref: "worker:event-bus:cell-a:001".to_owned(),
            attempt_id: format!("attempt:event-bus-worker:{idempotency_key}:1"),
            attempt_number: 1,
            max_attempts: WORKFLOW_EVENT_BUS_WORKER_MAX_ATTEMPTS,
            now_epoch_seconds: 100,
            not_before_epoch_seconds: 90,
            lease_expires_epoch_seconds: 180,
            body: WorkflowEventBusWorkerJobBody::Delivery(Box::new(delivery_request(
                idempotency_key,
            ))),
        }
    }

    fn publish_request(idempotency_key: &str) -> WorkflowEventBusApiPublishRequest {
        WorkflowEventBusApiPublishRequest {
            boundary: boundary(idempotency_key),
            principal: principal(),
            authorization: authorization(),
            method: WORKFLOW_EVENT_BUS_API_METHOD.to_owned(),
            route: WORKFLOW_EVENT_BUS_API_PUBLISH_ROUTE.to_owned(),
            body: WorkflowEventBusApiPublishBody {
                cell_id: "cell:us-east-1a".to_owned(),
                residency_ref: "residency:us:data-plane".to_owned(),
                audit_chain_ref: "audit-chain:event-bus-worker".to_owned(),
                event_kind: "workflow-run-started".to_owned(),
                producer_ref: "producer:workflow-engine:execution".to_owned(),
                event_id: "event:workflow-run-started:001".to_owned(),
                source_ref: "urn:oyatie:workflow-engine:execution".to_owned(),
                subject_ref: Some("subject:workflow-run:001".to_owned()),
                time_ref: Some("time:2026-05-25T00:00:00Z".to_owned()),
                dataschema_ref: Some("schema:workflow-event-run-started".to_owned()),
                partition_key_ref: "partition:tenant-workflow-run".to_owned(),
                publish_idempotency_key: "idem:event-bus-domain:publish:001".to_owned(),
                causation_ref: "cause:execution-engine:start-run".to_owned(),
                correlation_ref: "corr:workflow-run:001".to_owned(),
                payload_ref: "body-ref:workflow-run-started".to_owned(),
                evidence_refs: vec!["evidence:event-bus-worker:publish".to_owned()],
            },
        }
    }

    fn delivery_request(idempotency_key: &str) -> WorkflowEventBusApiDeliveryRequest {
        WorkflowEventBusApiDeliveryRequest {
            boundary: boundary(idempotency_key),
            principal: principal(),
            authorization: authorization(),
            method: WORKFLOW_EVENT_BUS_API_METHOD.to_owned(),
            route: WORKFLOW_EVENT_BUS_API_DELIVERY_ROUTE.to_owned(),
            body: WorkflowEventBusApiDeliveryBody {
                cell_id: "cell:us-east-1a".to_owned(),
                residency_ref: "residency:us:data-plane".to_owned(),
                audit_chain_ref: "audit-chain:event-bus-worker".to_owned(),
                subscription_channel: "workflow-state".to_owned(),
                consumer_ref: "consumer:workflow-state-machine".to_owned(),
                subscription_event_types: vec![
                    WorkflowEventBusEventKind::WorkflowStateTransitioned
                        .event_type()
                        .to_owned(),
                ],
                replay_cursor_ref: Some("cursor:event-bus-worker:state".to_owned()),
                max_batch_size: 100,
                subscription_authorization_evidence_ref: "authz:event-bus-worker:consume"
                    .to_owned(),
                candidate_channel: "workflow-state".to_owned(),
                candidate_event_id: "event:workflow-state:001".to_owned(),
                candidate_event_type: WorkflowEventBusEventKind::WorkflowStateTransitioned
                    .event_type()
                    .to_owned(),
                candidate_idempotency_key: "idem:event-bus-domain:delivery:001".to_owned(),
                candidate_payload_ref: "body-ref:workflow-state-transitioned".to_owned(),
                candidate_offset_ref: "offset:partition-0:42".to_owned(),
                candidate_evidence_refs: vec!["evidence:event-bus-worker:delivery".to_owned()],
            },
        }
    }

    fn boundary(idempotency_key: &str) -> WorkflowEventBusApiBoundaryContext {
        WorkflowEventBusApiBoundaryContext {
            request_id: format!("request:event-bus-api:{idempotency_key}"),
            tenant_id: "ten_workflow_event_bus".to_owned(),
            idempotency_key: idempotency_key.to_owned(),
            trace_context_ref: "trace:event-bus-worker".to_owned(),
            oyatie_version: WORKFLOW_EVENT_BUS_API_DECLARED_VERSION.to_owned(),
        }
    }

    fn principal() -> WorkflowEventBusApiPrincipal {
        WorkflowEventBusApiPrincipal {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            principal_id: "principal:workflow-operator".to_owned(),
        }
    }

    fn authorization() -> WorkflowEventBusApiAuthorization {
        WorkflowEventBusApiAuthorization {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            principal_id: "principal:workflow-operator".to_owned(),
            decision_id: "policy-decision:event-bus-allow".to_owned(),
            evidence_ref: "policy-evidence:event-bus-allow".to_owned(),
            policy_bundle_ref: "policy-bundle:event-bus-v1".to_owned(),
            allowed_surfaces: vec![WORKFLOW_EVENT_BUS_API_SURFACE.to_owned()],
            allowed_channels: vec![
                "workflow-runs".to_owned(),
                "workflow-state".to_owned(),
                "trigger-events".to_owned(),
                "intelligence-requests".to_owned(),
                "ontology-projections".to_owned(),
            ],
            allowed_event_types: vec![
                WorkflowEventBusEventKind::WorkflowRunStarted
                    .event_type()
                    .to_owned(),
                WorkflowEventBusEventKind::WorkflowStateTransitioned
                    .event_type()
                    .to_owned(),
                WorkflowEventBusEventKind::TriggerEvaluated
                    .event_type()
                    .to_owned(),
                WorkflowEventBusEventKind::IntelligenceDraftRequested
                    .event_type()
                    .to_owned(),
                WorkflowEventBusEventKind::OntologyProjectionUpdated
                    .event_type()
                    .to_owned(),
            ],
        }
    }

    fn resume_candidate(
        tenant_id: &str,
        channel_address: &str,
        event_id: &str,
        due_epoch_seconds: u64,
        resume_priority: u32,
    ) -> WorkflowEventBusWorkerResumeCandidate {
        WorkflowEventBusWorkerResumeCandidate {
            tenant_id: tenant_id.to_owned(),
            channel_address: channel_address.to_owned(),
            event_type: WorkflowEventBusEventKind::WorkflowRunStarted
                .event_type()
                .to_owned(),
            event_id: event_id.to_owned(),
            due_epoch_seconds,
            resume_priority,
            resume_evidence_ref: format!("resume:{event_id}"),
        }
    }
}
