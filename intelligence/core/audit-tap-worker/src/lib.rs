//! Intelligence audit-tap worker foundation.
//!
//! This crate provides a deterministic source-level worker seam for future
//! audit-tap job execution. It validates queued job metadata, honors not-before
//! scheduling before side effects, runs the audit-tap usecase, and emits through
//! the metadata-only audit-chain adapter seam. It performs no durable queue I/O,
//! Ed25519 signing, Merkle-tree construction, network I/O, filesystem access,
//! durable idempotency storage, durable audit-chain writes, or cloud runtime
//! scheduling.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use intelligence_audit_tap_adapter::{
    AuditChainAdapterConfig, AuditChainAdapterConfigError, AuditChainRequestEnvelope,
    AuditChainStatus, AuditChainTransportMode, AuditTapAdapter, AuditTapEmissionFailure,
    AuditTapEmitterPort, AuditTapRecordRequest, AuditTapSealReceipt, CarbonIntensitySource,
    InfrastructureProvider, IntelligenceAuditEventClass, IntelligenceAuditTapDenialKind,
    IntelligenceAuditTapInput, IntelligenceAuditTapReceipt, IntelligenceAuditTapStatus,
    IntelligenceAuditTapUsecase,
};

const MAX_WORKER_ATTEMPTS: u32 = 10;
const BASE_RETRY_BACKOFF_SECONDS: u64 = 30;
const MAX_RETRY_BACKOFF_SECONDS: u64 = 900;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditTapWorkerJob {
    pub job_id: String,                   // data_class: INTERNAL_ONLY
    pub lease_id: String,                 // data_class: INTERNAL_ONLY
    pub attempt_id: String,               // data_class: INTERNAL_ONLY
    pub attempt_number: u32,              // data_class: INTERNAL_ONLY
    pub max_attempts: u32,                // data_class: INTERNAL_ONLY
    pub now_epoch_seconds: u64,           // data_class: INTERNAL_ONLY
    pub not_before_epoch_seconds: u64,    // data_class: INTERNAL_ONLY
    pub input: IntelligenceAuditTapInput, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditTapWorkerStatus {
    AuditChainSealed,
    Deferred,
    Denied,
    Exhausted,
    RetryScheduled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditTapWorkerDenialKind {
    AuditChainDenied,
    AuditChainInvalidRequest,
    AuditTapUsecaseDenied,
    InvalidJob,
    RetryExhausted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditTapWorkerReceipt {
    pub job_id: String,                                   // data_class: INTERNAL_ONLY
    pub attempt_id: String,                               // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                          // data_class: INTERNAL_ONLY
    pub audit_id: String,                                 // data_class: INTERNAL_ONLY
    pub event_id: String,                                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,                                // data_class: INTERNAL_ONLY
    pub status: AuditTapWorkerStatus,                     // data_class: PUBLIC
    pub denial_kind: Option<AuditTapWorkerDenialKind>,    // data_class: INTERNAL_ONLY
    pub audit_status: Option<IntelligenceAuditTapStatus>, // data_class: INTERNAL_ONLY
    pub sequence: Option<u64>,                            // data_class: INTERNAL_ONLY
    pub chain_ref: Option<String>,                        // data_class: INTERNAL_ONLY
    pub merkle_root_ref: Option<String>,                  // data_class: INTERNAL_ONLY
    pub signature_ref: Option<String>,                    // data_class: INTERNAL_ONLY
    pub outbox_ref: Option<String>,                       // data_class: INTERNAL_ONLY
    pub next_attempt_epoch_seconds: Option<u64>,          // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditTapWorkerEventKind {
    AuditChainSealed,
    AuditTapDenied,
    JobAccepted,
    JobDenied,
    RetryExhausted,
    RetryScheduled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditTapWorkerEvent {
    pub kind: AuditTapWorkerEventKind, // data_class: INTERNAL_ONLY
    pub job_id: String,                // data_class: INTERNAL_ONLY
    pub attempt_id: String,            // data_class: INTERNAL_ONLY
    pub idempotency_key: String,       // data_class: INTERNAL_ONLY
    pub audit_id: String,              // data_class: INTERNAL_ONLY
    pub event_id: String,              // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct SealRefs {
    sequence: Option<u64>,
    chain_ref: Option<String>,
    merkle_root_ref: Option<String>,
    signature_ref: Option<String>,
    outbox_ref: Option<String>,
}

pub struct AuditTapWorker {
    audit_tap_usecase: IntelligenceAuditTapUsecase<AuditTapAdapter>,
    events: Vec<AuditTapWorkerEvent>,
}

impl AuditTapWorker {
    pub fn new(adapter: AuditTapAdapter) -> Self {
        Self {
            audit_tap_usecase: IntelligenceAuditTapUsecase::new(adapter),
            events: Vec::new(),
        }
    }

    pub fn run_once(&mut self, job: AuditTapWorkerJob) -> AuditTapWorkerReceipt {
        if let Err(evidence_ref) = validate_job(&job) {
            return receipt_from_job(
                &job,
                AuditTapWorkerStatus::Denied,
                Some(AuditTapWorkerDenialKind::InvalidJob),
                None,
                SealRefs::default(),
                None,
                vec![evidence_ref],
            );
        }

        if job.now_epoch_seconds < job.not_before_epoch_seconds {
            return receipt_from_job(
                &job,
                AuditTapWorkerStatus::Deferred,
                None,
                None,
                SealRefs::default(),
                Some(job.not_before_epoch_seconds),
                vec!["audit-tap-worker:deferred:not-before".to_owned()],
            );
        }

        self.record_event(
            AuditTapWorkerEventKind::JobAccepted,
            &job,
            canonical_request_evidence_refs(&job),
        );

        let audit_receipt = self.audit_tap_usecase.emit(job.input.clone());
        if audit_receipt.status == IntelligenceAuditTapStatus::Sealed {
            let seal = audit_receipt.seal.as_ref();
            let receipt = receipt_from_job(
                &job,
                AuditTapWorkerStatus::AuditChainSealed,
                None,
                Some(audit_receipt.status),
                seal_refs_from(seal),
                None,
                sorted_unique(
                    [
                        audit_receipt.evidence_refs.clone(),
                        vec!["audit-tap-worker:audit-chain-sealed".to_owned()],
                    ]
                    .concat(),
                ),
            );
            self.record_event(
                AuditTapWorkerEventKind::AuditChainSealed,
                &job,
                receipt.evidence_refs.clone(),
            );
            return receipt;
        }

        self.receipt_from_audit_denial(&job, audit_receipt)
    }

    pub fn events(&self) -> &[AuditTapWorkerEvent] {
        &self.events
    }

    fn receipt_from_audit_denial(
        &mut self,
        job: &AuditTapWorkerJob,
        audit_receipt: IntelligenceAuditTapReceipt,
    ) -> AuditTapWorkerReceipt {
        if audit_receipt.denial_kind == Some(IntelligenceAuditTapDenialKind::EmitterFailed) {
            if is_retryable_audit_chain_evidence(&audit_receipt.evidence_refs) {
                if job.attempt_number < job.max_attempts {
                    let next_attempt = job
                        .now_epoch_seconds
                        .saturating_add(retry_backoff_seconds(job.attempt_number));
                    let receipt = receipt_from_job(
                        job,
                        AuditTapWorkerStatus::RetryScheduled,
                        None,
                        Some(audit_receipt.status),
                        SealRefs::default(),
                        Some(next_attempt),
                        sorted_unique(
                            [
                                audit_receipt.evidence_refs.clone(),
                                vec!["audit-tap-worker:audit-chain-retry-scheduled".to_owned()],
                            ]
                            .concat(),
                        ),
                    );
                    self.record_event(
                        AuditTapWorkerEventKind::RetryScheduled,
                        job,
                        receipt.evidence_refs.clone(),
                    );
                    return receipt;
                }
                let receipt = receipt_from_job(
                    job,
                    AuditTapWorkerStatus::Exhausted,
                    Some(AuditTapWorkerDenialKind::RetryExhausted),
                    Some(audit_receipt.status),
                    SealRefs::default(),
                    None,
                    sorted_unique(
                        [
                            audit_receipt.evidence_refs.clone(),
                            vec!["audit-tap-worker:audit-chain-retry-exhausted".to_owned()],
                        ]
                        .concat(),
                    ),
                );
                self.record_event(
                    AuditTapWorkerEventKind::RetryExhausted,
                    job,
                    receipt.evidence_refs.clone(),
                );
                return receipt;
            }

            let denial_kind = if is_invalid_request_evidence(&audit_receipt.evidence_refs) {
                AuditTapWorkerDenialKind::AuditChainInvalidRequest
            } else {
                AuditTapWorkerDenialKind::AuditChainDenied
            };
            let receipt = receipt_from_job(
                job,
                AuditTapWorkerStatus::Denied,
                Some(denial_kind),
                Some(audit_receipt.status),
                SealRefs::default(),
                None,
                sorted_unique(
                    [
                        audit_receipt.evidence_refs.clone(),
                        vec!["audit-tap-worker:audit-chain-denied".to_owned()],
                    ]
                    .concat(),
                ),
            );
            self.record_event(
                AuditTapWorkerEventKind::JobDenied,
                job,
                receipt.evidence_refs.clone(),
            );
            return receipt;
        }

        let receipt = receipt_from_job(
            job,
            AuditTapWorkerStatus::Denied,
            Some(AuditTapWorkerDenialKind::AuditTapUsecaseDenied),
            Some(audit_receipt.status),
            SealRefs::default(),
            None,
            sorted_unique(
                [
                    audit_receipt.evidence_refs.clone(),
                    vec!["audit-tap-worker:audit-tap-usecase-denied".to_owned()],
                ]
                .concat(),
            ),
        );
        self.record_event(
            AuditTapWorkerEventKind::AuditTapDenied,
            job,
            receipt.evidence_refs.clone(),
        );
        receipt
    }

    fn record_event(
        &mut self,
        kind: AuditTapWorkerEventKind,
        job: &AuditTapWorkerJob,
        evidence_refs: Vec<String>,
    ) {
        self.events.push(AuditTapWorkerEvent {
            kind,
            job_id: safe_metadata(&job.job_id, "redacted-invalid-job-id"),
            attempt_id: safe_metadata(&job.attempt_id, "redacted-invalid-attempt-id"),
            idempotency_key: safe_metadata(
                &job.input.idempotency_key,
                "redacted-invalid-idempotency-key",
            ),
            audit_id: safe_metadata(&job.input.audit_id, "redacted-invalid-audit-id"),
            event_id: safe_metadata(&job.input.event_id, "redacted-invalid-event-id"),
            tenant_id: safe_tenant(&job.input.tenant_id),
            evidence_refs: sorted_unique(evidence_refs),
        });
    }
}

fn validate_job(job: &AuditTapWorkerJob) -> Result<(), String> {
    require_metadata(&job.job_id, "validation:audit-tap-worker-job-id")?;
    require_metadata(&job.lease_id, "validation:audit-tap-worker-lease-id")?;
    require_metadata(&job.attempt_id, "validation:audit-tap-worker-attempt-id")?;
    if job.attempt_number == 0
        || job.max_attempts == 0
        || job.max_attempts > MAX_WORKER_ATTEMPTS
        || job.attempt_number > job.max_attempts
    {
        return Err("validation:audit-tap-worker-attempt-bounds".to_owned());
    }
    validate_input(&job.input)
}

fn validate_input(input: &IntelligenceAuditTapInput) -> Result<(), String> {
    require_metadata(
        &input.idempotency_key,
        "validation:audit-tap-worker-idempotency-key",
    )?;
    require_metadata(&input.audit_id, "validation:audit-tap-worker-audit-id")?;
    require_metadata(&input.event_id, "validation:audit-tap-worker-event-id")?;
    require_tenant(&input.tenant_id, "validation:audit-tap-worker-tenant")?;
    require_metadata(
        &input.producer_id,
        "validation:audit-tap-worker-producer-id",
    )?;
    require_metadata(&input.trace_id, "validation:audit-tap-worker-trace-id")?;
    require_metadata(&input.span_id, "validation:audit-tap-worker-span-id")?;
    require_metadata(&input.cell_id, "validation:audit-tap-worker-cell-id")?;
    require_metadata(
        &input.jurisdiction_code,
        "validation:audit-tap-worker-jurisdiction-code",
    )?;
    require_metadata(
        &input.sub_scope_path,
        "validation:audit-tap-worker-sub-scope-path",
    )?;
    require_metadata(
        &input.hlc_timestamp,
        "validation:audit-tap-worker-hlc-timestamp",
    )?;
    if !input.hlc_timestamp.contains("/lc=") {
        return Err("validation:audit-tap-worker-hlc-timestamp".to_owned());
    }
    if input.occurred_at_epoch_seconds == 0 || input.cost_usd_minor_units < 0 {
        return Err("validation:audit-tap-worker-audit-envelope".to_owned());
    }
    require_metadata(&input.action, "validation:audit-tap-worker-action")?;
    require_metadata(&input.decision, "validation:audit-tap-worker-decision")?;
    require_evidence_ref(
        &input.request_evidence_ref,
        "validation:audit-tap-worker-request-evidence",
    )?;
    require_evidence_ref(
        &input.policy_decision_ref,
        "validation:audit-tap-worker-policy-decision",
    )?;
    for value in &input.route_evidence_refs {
        require_evidence_ref(value, "validation:audit-tap-worker-route-evidence")?;
    }
    for value in &input.guardrail_evidence_refs {
        require_evidence_ref(value, "validation:audit-tap-worker-guardrail-evidence")?;
    }
    if let Some(value) = input.credential_evidence_ref.as_ref() {
        require_evidence_ref(value, "validation:audit-tap-worker-credential-evidence")?;
    }
    if let Some(value) = input.provider_evidence_ref.as_ref() {
        require_evidence_ref(value, "validation:audit-tap-worker-provider-evidence")?;
    }
    if let Some(value) = input.content_ref.as_ref() {
        require_resource_ref(value, "validation:audit-tap-worker-content-ref")?;
    }
    if let Some(value) = input.output_ref.as_ref() {
        require_resource_ref(value, "validation:audit-tap-worker-output-ref")?;
    }
    require_metadata(&input.region, "validation:audit-tap-worker-region")?;
    Ok(())
}

fn receipt_from_job(
    job: &AuditTapWorkerJob,
    status: AuditTapWorkerStatus,
    denial_kind: Option<AuditTapWorkerDenialKind>,
    audit_status: Option<IntelligenceAuditTapStatus>,
    seal_refs: SealRefs,
    next_attempt_epoch_seconds: Option<u64>,
    evidence_refs: Vec<String>,
) -> AuditTapWorkerReceipt {
    AuditTapWorkerReceipt {
        job_id: safe_metadata(&job.job_id, "redacted-invalid-job-id"),
        attempt_id: safe_metadata(&job.attempt_id, "redacted-invalid-attempt-id"),
        idempotency_key: safe_metadata(
            &job.input.idempotency_key,
            "redacted-invalid-idempotency-key",
        ),
        audit_id: safe_metadata(&job.input.audit_id, "redacted-invalid-audit-id"),
        event_id: safe_metadata(&job.input.event_id, "redacted-invalid-event-id"),
        tenant_id: safe_tenant(&job.input.tenant_id),
        status,
        denial_kind,
        audit_status,
        sequence: seal_refs.sequence,
        chain_ref: seal_refs
            .chain_ref
            .map(|value| safe_ref(&value, "audit-tap-worker:redacted-chain-ref")),
        merkle_root_ref: seal_refs
            .merkle_root_ref
            .map(|value| safe_ref(&value, "audit-tap-worker:redacted-merkle-root-ref")),
        signature_ref: seal_refs
            .signature_ref
            .map(|value| safe_ref(&value, "audit-tap-worker:redacted-signature-ref")),
        outbox_ref: seal_refs
            .outbox_ref
            .map(|value| safe_ref(&value, "audit-tap-worker:redacted-outbox-ref")),
        next_attempt_epoch_seconds,
        evidence_refs: sorted_unique(evidence_refs),
    }
}

fn seal_refs_from(seal: Option<&AuditTapSealReceipt>) -> SealRefs {
    seal.map(|seal| SealRefs {
        sequence: Some(seal.sequence),
        chain_ref: Some(seal.chain_ref.clone()),
        merkle_root_ref: Some(seal.merkle_root_ref.clone()),
        signature_ref: Some(seal.signature_ref.clone()),
        outbox_ref: Some(seal.outbox_ref.clone()),
    })
    .unwrap_or_default()
}

fn canonical_request_evidence_refs(job: &AuditTapWorkerJob) -> Vec<String> {
    sorted_unique(vec![
        job.input.request_evidence_ref.clone(),
        job.input.policy_decision_ref.clone(),
        "audit-tap-worker:job-accepted".to_owned(),
    ])
}

fn is_retryable_audit_chain_evidence(evidence_refs: &[String]) -> bool {
    evidence_refs.iter().any(|value| {
        let lower = value.to_ascii_lowercase();
        lower.contains("rate-limited") || lower.contains("timeout") || lower.contains("error")
    })
}

fn is_invalid_request_evidence(evidence_refs: &[String]) -> bool {
    evidence_refs
        .iter()
        .any(|value| value.to_ascii_lowercase().contains("invalid"))
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

fn require_tenant(value: &str, evidence_ref: &str) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.starts_with("ten_")
        && trimmed == value
        && !trimmed.contains('/')
        && !contains_whitespace(trimmed)
        && !contains_raw_secret_material(trimmed)
        && !contains_raw_content_material(trimmed)
    {
        Ok(())
    } else {
        Err(evidence_ref.to_owned())
    }
}

fn require_evidence_ref(value: &str, evidence_ref: &str) -> Result<(), String> {
    if is_safe_evidence_ref(value) {
        Ok(())
    } else {
        Err(evidence_ref.to_owned())
    }
}

fn require_resource_ref(value: &str, evidence_ref: &str) -> Result<(), String> {
    if is_safe_resource_ref(value) {
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

fn safe_tenant(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with("ten_")
        && trimmed == value
        && !trimmed.contains('/')
        && !contains_whitespace(trimmed)
        && !contains_raw_secret_material(trimmed)
        && !contains_raw_content_material(trimmed)
    {
        trimmed.to_owned()
    } else {
        "redacted-invalid-tenant-id".to_owned()
    }
}

fn safe_ref(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if is_safe_evidence_ref(trimmed) && trimmed == value {
        trimmed.to_owned()
    } else {
        fallback.to_owned()
    }
}

fn is_safe_metadata_ref(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
        && !contains_whitespace(trimmed)
        && !contains_raw_secret_material(trimmed)
        && !contains_raw_content_material(trimmed)
}

fn is_safe_evidence_ref(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
        && trimmed.contains(':')
        && !contains_whitespace(trimmed)
        && !contains_raw_secret_material(trimmed)
        && !contains_raw_content_material(trimmed)
}

fn is_safe_resource_ref(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
        && trimmed.contains("://")
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
        || lower.contains("private key")
        || lower.contains("-----begin")
        || lower.contains("ed25519_private_key")
}

fn contains_raw_content_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw model")
        || lower.contains("write an email")
        || lower.contains("customer message")
        || lower.contains("model answer")
        || lower.contains("raw output")
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

    fn config() -> AuditChainAdapterConfig {
        AuditChainAdapterConfig::new(
            "audit-chain://writer/intelligence",
            "merkle://sealer/intelligence",
            "sigkey://ed25519/intelligence/audit-tap",
            "outbox://audit-tap/intelligence",
            "audience://audit-chain/intelligence",
        )
    }

    fn sealed_status() -> AuditChainStatus {
        AuditChainStatus::Sealed {
            sequence: 77,
            chain_ref: "audit-chain://intelligence/ten_a/2026-05-24".to_owned(),
            merkle_root_ref: "merkle://intelligence/root/77".to_owned(),
            signature_ref: "sigref://ed25519/intelligence/77".to_owned(),
            outbox_ref: "outbox://audit-tap/77".to_owned(),
            evidence_ref: "seal:evidence:77".to_owned(),
        }
    }

    fn valid_worker(status: AuditChainStatus) -> AuditTapWorker {
        AuditTapWorker::new(AuditTapAdapter::try_new(config(), status).expect("valid adapter"))
    }

    fn valid_job() -> AuditTapWorkerJob {
        AuditTapWorkerJob {
            job_id: "job:audit-tap:1".to_owned(),
            lease_id: "lease:audit-tap:1".to_owned(),
            attempt_id: "attempt:audit-tap:1".to_owned(),
            attempt_number: 1,
            max_attempts: 3,
            now_epoch_seconds: 1_000,
            not_before_epoch_seconds: 900,
            input: audit_input("idem:audit-tap:1"),
        }
    }

    fn audit_input(idempotency_key: &str) -> IntelligenceAuditTapInput {
        IntelligenceAuditTapInput {
            idempotency_key: idempotency_key.to_owned(),
            audit_id: "audit_01".to_owned(),
            event_id: "evt_01".to_owned(),
            tenant_id: "ten_a".to_owned(),
            producer_id: "intelligence-dispatch".to_owned(),
            trace_id: "trace-01".to_owned(),
            span_id: "span-01".to_owned(),
            cell_id: "cell-us-east-1a".to_owned(),
            jurisdiction_code: "US".to_owned(),
            sub_scope_path: "intelligence/dispatch/openai".to_owned(),
            hlc_timestamp: "2026-05-24T21:50:00Z/lc=1".to_owned(),
            occurred_at_epoch_seconds: 1_779_647_400,
            event_class: IntelligenceAuditEventClass::DispatchCompleted,
            action: "provider_dispatch".to_owned(),
            decision: "allow".to_owned(),
            request_evidence_ref: "req:audit-tap".to_owned(),
            policy_decision_ref: "cedar:decision:1".to_owned(),
            route_evidence_refs: vec!["route:openai".to_owned()],
            guardrail_evidence_refs: vec!["guardrail:allow".to_owned()],
            credential_evidence_ref: Some("credential:handle:1".to_owned()),
            provider_evidence_ref: Some("openai:responses:resp_1".to_owned()),
            content_ref: Some("contentref://prompt/1".to_owned()),
            output_ref: Some("contentref://output/1".to_owned()),
            cost_usd_minor_units: 1,
            co2_grams: 2,
            watt_hours: 3,
            infrastructure_provider: InfrastructureProvider::OyatieOwn,
            region: "oyatie_own:iad-1".to_owned(),
            carbon_intensity_source: CarbonIntensitySource::ProviderGridAverage,
        }
    }

    #[test]
    fn processes_authorized_job_and_seals_audit_chain_record() {
        let mut worker = valid_worker(sealed_status());

        let receipt = worker.run_once(valid_job());

        assert_eq!(receipt.status, AuditTapWorkerStatus::AuditChainSealed);
        assert_eq!(receipt.denial_kind, None);
        assert_eq!(receipt.sequence, Some(77));
        assert_eq!(
            receipt.chain_ref,
            Some("audit-chain://intelligence/ten_a/2026-05-24".to_owned())
        );
        assert_eq!(
            worker.events()[0].kind,
            AuditTapWorkerEventKind::JobAccepted
        );
        assert_eq!(
            worker.events()[1].kind,
            AuditTapWorkerEventKind::AuditChainSealed
        );
        assert!(
            receipt
                .evidence_refs
                .contains(&"audit-tap-worker:audit-chain-sealed".to_owned())
        );
    }

    #[test]
    fn defers_not_before_jobs_without_usecase_or_adapter_side_effects() {
        let mut worker = valid_worker(sealed_status());
        let mut job = valid_job();
        job.now_epoch_seconds = 1_000;
        job.not_before_epoch_seconds = 1_300;

        let receipt = worker.run_once(job);

        assert_eq!(receipt.status, AuditTapWorkerStatus::Deferred);
        assert_eq!(receipt.next_attempt_epoch_seconds, Some(1_300));
        assert!(worker.events().is_empty());
    }

    #[test]
    fn invalid_raw_job_metadata_denies_before_side_effects() {
        let mut worker = valid_worker(sealed_status());
        let mut job = valid_job();
        job.input.content_ref = Some("write an email to a customer".to_owned());
        job.input.provider_evidence_ref = Some("Authorization: Bearer sk-test".to_owned());

        let receipt = worker.run_once(job);
        let debug = format!("{receipt:?}{:?}", worker.events());

        assert_eq!(receipt.status, AuditTapWorkerStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(AuditTapWorkerDenialKind::InvalidJob)
        );
        assert!(worker.events().is_empty());
        assert!(!debug.contains("sk-test"));
        assert!(!debug.contains("write an email"));
    }

    #[test]
    fn retryable_audit_chain_failure_schedules_backoff_and_exhausts_at_max_attempts() {
        let mut worker = valid_worker(AuditChainStatus::RateLimited {
            evidence_ref: "audit-chain:rate-limited".to_owned(),
        });
        let retry = worker.run_once(valid_job());

        assert_eq!(retry.status, AuditTapWorkerStatus::RetryScheduled);
        assert_eq!(retry.next_attempt_epoch_seconds, Some(1_030));
        assert_eq!(
            worker.events()[1].kind,
            AuditTapWorkerEventKind::RetryScheduled
        );

        let mut exhausted_worker = valid_worker(AuditChainStatus::Timeout {
            evidence_ref: "audit-chain:timeout".to_owned(),
        });
        let mut exhausted_job = valid_job();
        exhausted_job.attempt_number = 3;
        exhausted_job.max_attempts = 3;
        let exhausted = exhausted_worker.run_once(exhausted_job);

        assert_eq!(exhausted.status, AuditTapWorkerStatus::Exhausted);
        assert_eq!(
            exhausted.denial_kind,
            Some(AuditTapWorkerDenialKind::RetryExhausted)
        );
    }

    #[test]
    fn nonretryable_audit_chain_invalid_request_denies_without_retry() {
        let mut worker = valid_worker(AuditChainStatus::InvalidRequest {
            evidence_ref: "audit-chain:invalid".to_owned(),
        });

        let receipt = worker.run_once(valid_job());

        assert_eq!(receipt.status, AuditTapWorkerStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(AuditTapWorkerDenialKind::AuditChainInvalidRequest)
        );
        assert_eq!(receipt.next_attempt_epoch_seconds, None);
        assert_eq!(worker.events()[1].kind, AuditTapWorkerEventKind::JobDenied);
    }

    #[test]
    fn nonretryable_audit_chain_denial_does_not_retry() {
        let mut worker = valid_worker(AuditChainStatus::Denied {
            evidence_ref: "audit-chain:denied".to_owned(),
        });

        let receipt = worker.run_once(valid_job());

        assert_eq!(receipt.status, AuditTapWorkerStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(AuditTapWorkerDenialKind::AuditChainDenied)
        );
        assert_eq!(receipt.next_attempt_epoch_seconds, None);
    }

    #[test]
    fn worker_debug_and_receipts_never_contain_raw_prompt_output_document_or_secret_bytes() {
        let mut worker = valid_worker(sealed_status());

        let receipt = worker.run_once(valid_job());
        let debug = format!("{receipt:?}{:?}", worker.events());

        assert!(!debug.contains("sk-test"));
        assert!(!debug.contains("write an email"));
        assert!(!debug.contains("raw model answer"));
        assert!(!debug.contains("raw prompt"));
        assert!(!debug.contains("raw output"));
    }
}
