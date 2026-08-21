//! Intelligence credential resolver usecase foundation.
//!
//! This crate orchestrates metadata-only resolution of provider credential
//! references into short-lived opaque `CredentialHandle`s. It owns idempotency,
//! in-process handle cache reuse, and rotation-triggered cache invalidation, but
//! deliberately has no OpenBao client, Unix socket, provider SDK, filesystem, or
//! raw credential material.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

use intelligence_credential_resolver_domain::{
    CredentialAudience, CredentialHandle, CredentialProvider, SecretReference,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialResolutionInput {
    pub idempotency_key: String,       // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub provider: CredentialProvider,  // data_class: INTERNAL_ONLY
    pub audience: CredentialAudience,  // data_class: INTERNAL_ONLY
    pub secret_reference_text: String, // data_class: INTERNAL_ONLY
    pub request_evidence_ref: String,  // data_class: INTERNAL_ONLY
    pub now_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialHandleRequest {
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub provider: CredentialProvider,      // data_class: INTERNAL_ONLY
    pub audience: CredentialAudience,      // data_class: INTERNAL_ONLY
    pub secret_reference: SecretReference, // data_class: INTERNAL_ONLY
    pub request_evidence_ref: String,      // data_class: INTERNAL_ONLY
    pub now_epoch_seconds: u64,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialHandleIssueFailure {
    pub reason: String,       // data_class: INTERNAL_ONLY
    pub evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialResolutionStatus {
    Resolved,
    Denied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialResolutionDenialKind {
    CachedHandleExpired,
    IdempotencyConflict,
    InvalidInput,
    SecretReferenceInvalid,
    SidecarFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialResolutionCacheStatus {
    Hit,
    MissIssued,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialResolutionReceipt {
    pub idempotency_key: String,            // data_class: INTERNAL_ONLY
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub provider: CredentialProvider,       // data_class: INTERNAL_ONLY
    pub audience: CredentialAudience,       // data_class: INTERNAL_ONLY
    pub status: CredentialResolutionStatus, // data_class: PUBLIC
    pub denial_kind: Option<CredentialResolutionDenialKind>, // data_class: INTERNAL_ONLY
    pub denial_reasons: Vec<String>,        // data_class: INTERNAL_ONLY
    pub handle: Option<CredentialHandle>,   // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,         // data_class: INTERNAL_ONLY
    pub cache_status: Option<CredentialResolutionCacheStatus>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialRotationEvent {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub provider: CredentialProvider,  // data_class: INTERNAL_ONLY
    pub secret_reference_text: String, // data_class: INTERNAL_ONLY
    pub old_generation: u64,           // data_class: INTERNAL_ONLY
    pub new_generation: u64,           // data_class: INTERNAL_ONLY
    pub rotated_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub evidence_ref: String,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialRotationStatus {
    Recorded,
    Denied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialRotationDenialKind {
    InvalidInput,
    SecretReferenceInvalid,
    GenerationNotAdvanced,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialRotationReceipt {
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub provider: CredentialProvider,     // data_class: INTERNAL_ONLY
    pub status: CredentialRotationStatus, // data_class: PUBLIC
    pub denial_kind: Option<CredentialRotationDenialKind>, // data_class: INTERNAL_ONLY
    pub denial_reasons: Vec<String>,      // data_class: INTERNAL_ONLY
    pub old_generation: u64,              // data_class: INTERNAL_ONLY
    pub new_generation: u64,              // data_class: INTERNAL_ONLY
    pub evicted_handles: usize,           // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialResolverAuditEventKind {
    ResolutionRequested,
    CacheHit,
    HandleIssued,
    ResolutionDenied,
    ByokCredentialRotated,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialResolverAuditEvent {
    pub kind: CredentialResolverAuditEventKind, // data_class: INTERNAL_ONLY
    pub tenant_id: String,                      // data_class: INTERNAL_ONLY
    pub provider: CredentialProvider,           // data_class: INTERNAL_ONLY
    pub audience: Option<CredentialAudience>,   // data_class: INTERNAL_ONLY
    pub idempotency_key: Option<String>,        // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,             // data_class: INTERNAL_ONLY
}

pub trait CredentialHandleIssuerPort {
    fn issue_handle(
        &mut self,
        request: CredentialHandleRequest,
    ) -> Result<CredentialHandle, CredentialHandleIssueFailure>;
}

pub trait CredentialResolverAuditSink {
    fn record(&mut self, event: CredentialResolverAuditEvent);
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct ResolutionIntent {
    tenant_id: String,
    provider: CredentialProvider,
    audience: CredentialAudience,
    secret_reference_canonical: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CacheKey {
    tenant_id: String,
    provider: CredentialProvider,
    audience: CredentialAudience,
    secret_reference_canonical: String,
}

#[derive(Debug)]
pub struct CredentialResolverUsecase<P, A> {
    issuer: P,
    audit_sink: A,
    receipts_by_idempotency_key: BTreeMap<String, (ResolutionIntent, CredentialResolutionReceipt)>,
    handles_by_key: BTreeMap<CacheKey, CredentialHandle>,
}

impl<P, A> CredentialResolverUsecase<P, A>
where
    P: CredentialHandleIssuerPort,
    A: CredentialResolverAuditSink,
{
    pub fn new(issuer: P, audit_sink: A) -> Self {
        Self {
            issuer,
            audit_sink,
            receipts_by_idempotency_key: BTreeMap::new(),
            handles_by_key: BTreeMap::new(),
        }
    }

    pub fn into_parts(self) -> (P, A) {
        (self.issuer, self.audit_sink)
    }

    pub fn resolve(&mut self, input: CredentialResolutionInput) -> CredentialResolutionReceipt {
        let invalid = invalid_input_reasons(&input);
        if !invalid.is_empty() {
            let receipt = denied_resolution_receipt(
                &input,
                CredentialResolutionDenialKind::InvalidInput,
                invalid,
                vec!["validation:credential-resolution-input".to_owned()],
            );
            self.record_resolution_denied(&receipt);
            return receipt;
        }

        let secret_reference = match SecretReference::parse(
            &input.secret_reference_text,
            &input.tenant_id,
            input.provider,
        ) {
            Ok(reference) => reference,
            Err(error) => {
                let receipt = denied_resolution_receipt(
                    &input,
                    CredentialResolutionDenialKind::SecretReferenceInvalid,
                    vec![format!("secret reference invalid: {error:?}")],
                    vec![
                        input.request_evidence_ref.clone(),
                        "validation:secret-reference".to_owned(),
                    ],
                );
                self.record_resolution_denied(&receipt);
                return receipt;
            }
        };

        let intent = ResolutionIntent::from_input(&input, secret_reference.canonical_ref());
        if let Some((existing_intent, existing_receipt)) =
            self.receipts_by_idempotency_key.get(&input.idempotency_key)
        {
            if existing_intent == &intent {
                if let Some(handle) = &existing_receipt.handle {
                    let cached_handle_still_authoritative = self
                        .handles_by_key
                        .get(&CacheKey::from_intent(existing_intent))
                        == Some(handle);
                    if handle.is_valid_for(
                        &input.tenant_id,
                        input.provider,
                        input.audience,
                        input.now_epoch_seconds,
                    ) && cached_handle_still_authoritative
                    {
                        return existing_receipt.clone();
                    }
                    let receipt = denied_resolution_receipt(
                        &input,
                        CredentialResolutionDenialKind::CachedHandleExpired,
                        vec!["cached idempotent credential handle is expired, rotated, or no longer bound to this request".to_owned()],
                        vec![input.request_evidence_ref.clone(), "validation:credential-idempotency-handle-not-authoritative".to_owned()],
                    );
                    self.record_resolution_denied(&receipt);
                    return receipt;
                }
                return existing_receipt.clone();
            }
            let receipt = denied_resolution_receipt(
                &input,
                CredentialResolutionDenialKind::IdempotencyConflict,
                vec![
                    "idempotency key already used for different credential resolution intent"
                        .to_owned(),
                ],
                vec![
                    input.request_evidence_ref.clone(),
                    "validation:credential-idempotency-conflict".to_owned(),
                ],
            );
            self.record_resolution_denied(&receipt);
            return receipt;
        }

        self.record_event(
            CredentialResolverAuditEventKind::ResolutionRequested,
            input.tenant_id.clone(),
            input.provider,
            Some(input.audience),
            Some(input.idempotency_key.clone()),
            vec![
                input.request_evidence_ref.clone(),
                secret_reference.canonical_ref().to_owned(),
            ],
        );

        let key = CacheKey::from_intent(&intent);
        if let Some(handle) = self.handles_by_key.get(&key) {
            if handle.is_valid_for(
                &input.tenant_id,
                input.provider,
                input.audience,
                input.now_epoch_seconds,
            ) {
                let receipt = resolved_receipt(
                    &input,
                    handle.clone(),
                    vec![
                        input.request_evidence_ref.clone(),
                        secret_reference.canonical_ref().to_owned(),
                    ],
                    CredentialResolutionCacheStatus::Hit,
                );
                self.receipts_by_idempotency_key
                    .insert(input.idempotency_key.clone(), (intent, receipt.clone()));
                self.record_event(
                    CredentialResolverAuditEventKind::CacheHit,
                    receipt.tenant_id.clone(),
                    receipt.provider,
                    Some(receipt.audience),
                    Some(receipt.idempotency_key.clone()),
                    receipt.evidence_refs.clone(),
                );
                return receipt;
            }
            self.handles_by_key.remove(&key);
        }

        let request = CredentialHandleRequest {
            tenant_id: input.tenant_id.clone(),
            provider: input.provider,
            audience: input.audience,
            secret_reference: secret_reference.clone(),
            request_evidence_ref: input.request_evidence_ref.clone(),
            now_epoch_seconds: input.now_epoch_seconds,
        };
        let handle = match self.issuer.issue_handle(request) {
            Ok(handle) => handle,
            Err(failure) => {
                let receipt = denied_resolution_receipt(
                    &input,
                    CredentialResolutionDenialKind::SidecarFailed,
                    vec![safe_failure_reason(&failure.reason)],
                    vec![
                        input.request_evidence_ref.clone(),
                        safe_evidence_ref(&failure.evidence_ref),
                    ],
                );
                self.record_resolution_denied(&receipt);
                return receipt;
            }
        };
        if !handle.is_valid_for(
            &input.tenant_id,
            input.provider,
            input.audience,
            input.now_epoch_seconds,
        ) {
            let receipt = denied_resolution_receipt(
                &input,
                CredentialResolutionDenialKind::SidecarFailed,
                vec!["sidecar returned a handle outside the requested tenant/provider/audience/time binding".to_owned()],
                vec![
                    input.request_evidence_ref.clone(),
                    "validation:credential-handle-binding".to_owned(),
                ],
            );
            self.record_resolution_denied(&receipt);
            return receipt;
        }

        self.handles_by_key.insert(key, handle.clone());
        let receipt = resolved_receipt(
            &input,
            handle,
            vec![
                input.request_evidence_ref.clone(),
                secret_reference.canonical_ref().to_owned(),
            ],
            CredentialResolutionCacheStatus::MissIssued,
        );
        self.receipts_by_idempotency_key
            .insert(input.idempotency_key.clone(), (intent, receipt.clone()));
        self.record_event(
            CredentialResolverAuditEventKind::HandleIssued,
            receipt.tenant_id.clone(),
            receipt.provider,
            Some(receipt.audience),
            Some(receipt.idempotency_key.clone()),
            receipt.evidence_refs.clone(),
        );
        receipt
    }

    pub fn record_rotation(&mut self, event: CredentialRotationEvent) -> CredentialRotationReceipt {
        let invalid = invalid_rotation_reasons(&event);
        if !invalid.is_empty() {
            return denied_rotation_receipt(
                event,
                CredentialRotationDenialKind::InvalidInput,
                invalid,
                vec!["validation:credential-rotation-input".to_owned()],
            );
        }
        if event.new_generation <= event.old_generation {
            return denied_rotation_receipt(
                event,
                CredentialRotationDenialKind::GenerationNotAdvanced,
                vec!["rotation generation must advance".to_owned()],
                vec!["validation:credential-rotation-generation".to_owned()],
            );
        }

        let secret_reference = match SecretReference::parse(
            &event.secret_reference_text,
            &event.tenant_id,
            event.provider,
        ) {
            Ok(reference) => reference,
            Err(error) => {
                return denied_rotation_receipt(
                    event,
                    CredentialRotationDenialKind::SecretReferenceInvalid,
                    vec![format!("secret reference invalid: {error:?}")],
                    vec!["validation:credential-rotation-secret-reference".to_owned()],
                );
            }
        };

        let canonical_ref = secret_reference.canonical_ref().to_owned();
        let before = self.handles_by_key.len();
        self.handles_by_key.retain(|key, _| {
            !(key.tenant_id == event.tenant_id
                && key.provider == event.provider
                && key.secret_reference_canonical == canonical_ref)
        });
        let evicted = before - self.handles_by_key.len();
        let evidence_refs = sorted_unique(vec![event.evidence_ref.clone(), canonical_ref]);
        let receipt = CredentialRotationReceipt {
            tenant_id: event.tenant_id.clone(),
            provider: event.provider,
            status: CredentialRotationStatus::Recorded,
            denial_kind: None,
            denial_reasons: Vec::new(),
            old_generation: event.old_generation,
            new_generation: event.new_generation,
            evicted_handles: evicted,
            evidence_refs: evidence_refs.clone(),
        };
        self.record_event(
            CredentialResolverAuditEventKind::ByokCredentialRotated,
            receipt.tenant_id.clone(),
            receipt.provider,
            None,
            None,
            evidence_refs,
        );
        receipt
    }

    fn record_resolution_denied(&mut self, receipt: &CredentialResolutionReceipt) {
        self.record_event(
            CredentialResolverAuditEventKind::ResolutionDenied,
            receipt.tenant_id.clone(),
            receipt.provider,
            Some(receipt.audience),
            Some(receipt.idempotency_key.clone()),
            receipt.evidence_refs.clone(),
        );
    }

    fn record_event(
        &mut self,
        kind: CredentialResolverAuditEventKind,
        tenant_id: String,
        provider: CredentialProvider,
        audience: Option<CredentialAudience>,
        idempotency_key: Option<String>,
        evidence_refs: Vec<String>,
    ) {
        self.audit_sink.record(CredentialResolverAuditEvent {
            kind,
            tenant_id,
            provider,
            audience,
            idempotency_key,
            evidence_refs: sorted_unique(evidence_refs),
        });
    }
}

impl ResolutionIntent {
    fn from_input(input: &CredentialResolutionInput, secret_reference_canonical: &str) -> Self {
        Self {
            tenant_id: input.tenant_id.clone(),
            provider: input.provider,
            audience: input.audience,
            secret_reference_canonical: secret_reference_canonical.to_owned(),
        }
    }
}

impl CacheKey {
    fn from_intent(intent: &ResolutionIntent) -> Self {
        Self {
            tenant_id: intent.tenant_id.clone(),
            provider: intent.provider,
            audience: intent.audience,
            secret_reference_canonical: intent.secret_reference_canonical.clone(),
        }
    }
}

fn invalid_input_reasons(input: &CredentialResolutionInput) -> Vec<String> {
    let mut reasons = Vec::new();
    if input.idempotency_key.trim().is_empty() {
        reasons.push("idempotency key is required".to_owned());
    }
    if input.tenant_id.trim().is_empty() {
        reasons.push("tenant id is required".to_owned());
    }
    if input.request_evidence_ref.trim().is_empty() {
        reasons.push("request evidence ref is required".to_owned());
    }
    if input.secret_reference_text.trim().is_empty() {
        reasons.push("secret reference is required".to_owned());
    }
    sorted_unique(reasons)
}

fn invalid_rotation_reasons(event: &CredentialRotationEvent) -> Vec<String> {
    let mut reasons = Vec::new();
    if event.tenant_id.trim().is_empty() {
        reasons.push("tenant id is required".to_owned());
    }
    if event.secret_reference_text.trim().is_empty() {
        reasons.push("secret reference is required".to_owned());
    }
    if event.evidence_ref.trim().is_empty() {
        reasons.push("rotation evidence ref is required".to_owned());
    }
    sorted_unique(reasons)
}

fn resolved_receipt(
    input: &CredentialResolutionInput,
    handle: CredentialHandle,
    evidence_refs: Vec<String>,
    cache_status: CredentialResolutionCacheStatus,
) -> CredentialResolutionReceipt {
    CredentialResolutionReceipt {
        idempotency_key: input.idempotency_key.clone(),
        tenant_id: input.tenant_id.clone(),
        provider: input.provider,
        audience: input.audience,
        status: CredentialResolutionStatus::Resolved,
        denial_kind: None,
        denial_reasons: Vec::new(),
        handle: Some(handle),
        evidence_refs: sorted_unique(evidence_refs),
        cache_status: Some(cache_status),
    }
}

fn denied_resolution_receipt(
    input: &CredentialResolutionInput,
    denial_kind: CredentialResolutionDenialKind,
    denial_reasons: Vec<String>,
    evidence_refs: Vec<String>,
) -> CredentialResolutionReceipt {
    CredentialResolutionReceipt {
        idempotency_key: input.idempotency_key.clone(),
        tenant_id: input.tenant_id.clone(),
        provider: input.provider,
        audience: input.audience,
        status: CredentialResolutionStatus::Denied,
        denial_kind: Some(denial_kind),
        denial_reasons: sorted_unique(denial_reasons),
        handle: None,
        evidence_refs: sorted_unique(evidence_refs),
        cache_status: None,
    }
}

fn denied_rotation_receipt(
    event: CredentialRotationEvent,
    denial_kind: CredentialRotationDenialKind,
    denial_reasons: Vec<String>,
    evidence_refs: Vec<String>,
) -> CredentialRotationReceipt {
    CredentialRotationReceipt {
        tenant_id: event.tenant_id,
        provider: event.provider,
        status: CredentialRotationStatus::Denied,
        denial_kind: Some(denial_kind),
        denial_reasons: sorted_unique(denial_reasons),
        old_generation: event.old_generation,
        new_generation: event.new_generation,
        evicted_handles: 0,
        evidence_refs: sorted_unique(evidence_refs),
    }
}

fn safe_evidence_ref(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "sidecar:missing-evidence-ref".to_owned()
    } else if value != trimmed
        || contains_raw_secret_material(trimmed)
        || contains_whitespace(trimmed)
    {
        "sidecar:unsafe-evidence-ref".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn safe_failure_reason(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() || contains_raw_secret_material(trimmed) {
        "sidecar failed".to_owned()
    } else {
        trimmed.to_owned()
    }
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
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use intelligence_credential_resolver_domain::{
        CredentialHandleIssueRequest, CredentialProvider,
    };

    #[derive(Default)]
    struct FakeSidecar {
        calls: usize,
        generation: u64,
    }

    impl CredentialHandleIssuerPort for FakeSidecar {
        fn issue_handle(
            &mut self,
            request: CredentialHandleRequest,
        ) -> Result<CredentialHandle, CredentialHandleIssueFailure> {
            self.calls += 1;
            self.generation += 1;
            CredentialHandle::issue(CredentialHandleIssueRequest {
                handle_id: format!(
                    "handle://{}/{}/gen-{}",
                    request.tenant_id, request.provider, self.generation
                ),
                tenant_id: request.tenant_id,
                provider: request.provider,
                audience: request.audience,
                issued_at_epoch_seconds: request.now_epoch_seconds,
                expires_at_epoch_seconds: request.now_epoch_seconds + 60,
                generation: self.generation,
                sidecar_signature_ref: format!("sigref://openbao/handle/{}", self.generation),
            })
            .map_err(|error| CredentialHandleIssueFailure {
                reason: format!("domain:{error:?}"),
                evidence_ref: "sidecar:domain-error".to_owned(),
            })
        }
    }

    #[derive(Default)]
    struct AuditCollector {
        events: Vec<CredentialResolverAuditEvent>,
    }

    impl CredentialResolverAuditSink for AuditCollector {
        fn record(&mut self, event: CredentialResolverAuditEvent) {
            self.events.push(event);
        }
    }

    fn input(idempotency_key: &str, now: u64) -> CredentialResolutionInput {
        CredentialResolutionInput {
            idempotency_key: idempotency_key.to_owned(),
            tenant_id: "ten_a".to_owned(),
            provider: CredentialProvider::OpenAi,
            audience: CredentialAudience::ProviderDispatch,
            secret_reference_text: "${openbao:secret/ten_a/intelligence/provider/openai}"
                .to_owned(),
            request_evidence_ref: "req:credential".to_owned(),
            now_epoch_seconds: now,
        }
    }

    #[test]
    fn resolves_openbao_reference_through_sidecar_and_audits_metadata_only() {
        let mut usecase =
            CredentialResolverUsecase::new(FakeSidecar::default(), AuditCollector::default());

        let receipt = usecase.resolve(input("idem-1", 100));
        let (sidecar, audit) = usecase.into_parts();

        assert_eq!(receipt.status, CredentialResolutionStatus::Resolved);
        assert_eq!(
            receipt.cache_status,
            Some(CredentialResolutionCacheStatus::MissIssued)
        );
        let handle = receipt.handle.clone().expect("handle returned");
        assert_eq!(handle.bound_tenant(), "ten_a");
        assert_eq!(handle.generation(), 1);
        assert_eq!(sidecar.calls, 1);
        let debug = format!("{receipt:?}{:?}", audit.events);
        assert!(!debug.contains("sk-"));
        assert!(!debug.contains("raw provider key"));
    }

    #[test]
    fn idempotent_replay_returns_original_receipt_and_conflict_denies_without_sidecar_call() {
        let mut usecase =
            CredentialResolverUsecase::new(FakeSidecar::default(), AuditCollector::default());

        let first = usecase.resolve(input("idem-1", 100));
        let replay = usecase.resolve(input("idem-1", 101));
        let mut drifted = input("idem-1", 101);
        drifted.provider = CredentialProvider::Anthropic;
        drifted.secret_reference_text =
            "${openbao:secret/ten_a/intelligence/provider/anthropic}".to_owned();
        let conflict = usecase.resolve(drifted);
        let (sidecar, _) = usecase.into_parts();

        assert_eq!(replay, first);
        assert_eq!(conflict.status, CredentialResolutionStatus::Denied);
        assert_eq!(
            conflict.denial_kind,
            Some(CredentialResolutionDenialKind::IdempotencyConflict)
        );
        assert_eq!(sidecar.calls, 1);
    }

    #[test]
    fn cache_reuses_unexpired_handle_and_refreshes_after_expiry() {
        let mut usecase =
            CredentialResolverUsecase::new(FakeSidecar::default(), AuditCollector::default());

        let first = usecase.resolve(input("idem-1", 100));
        let cached = usecase.resolve(input("idem-2", 120));
        let refreshed = usecase.resolve(input("idem-3", 160));
        let (sidecar, _) = usecase.into_parts();

        assert_eq!(first.handle.as_ref().unwrap().generation(), 1);
        assert_eq!(cached.handle.as_ref().unwrap().generation(), 1);
        assert_eq!(
            cached.cache_status,
            Some(CredentialResolutionCacheStatus::Hit)
        );
        assert_eq!(refreshed.handle.as_ref().unwrap().generation(), 2);
        assert_eq!(sidecar.calls, 2);
    }

    #[test]
    fn rotation_evicts_matching_tenant_provider_cache_and_records_audit() {
        let mut usecase =
            CredentialResolverUsecase::new(FakeSidecar::default(), AuditCollector::default());

        let before = usecase.resolve(input("idem-1", 100));
        let rotation = usecase.record_rotation(CredentialRotationEvent {
            tenant_id: "ten_a".to_owned(),
            provider: CredentialProvider::OpenAi,
            secret_reference_text: "${openbao:secret/ten_a/intelligence/provider/openai}"
                .to_owned(),
            old_generation: before.handle.as_ref().unwrap().generation(),
            new_generation: 2,
            rotated_at_epoch_seconds: 110,
            evidence_ref: "rotation:openbao:2".to_owned(),
        });
        let after = usecase.resolve(input("idem-2", 111));
        let (_, audit) = usecase.into_parts();

        assert_eq!(rotation.status, CredentialRotationStatus::Recorded);
        assert_eq!(rotation.evicted_handles, 1);
        assert_eq!(after.handle.as_ref().unwrap().generation(), 2);
        assert!(
            audit
                .events
                .iter()
                .any(|event| event.kind == CredentialResolverAuditEventKind::ByokCredentialRotated)
        );
    }

    #[test]
    fn rotation_invalidates_idempotent_replay_of_previous_handle_without_sidecar_call() {
        let mut usecase =
            CredentialResolverUsecase::new(FakeSidecar::default(), AuditCollector::default());

        let first = usecase.resolve(input("idem-rotated", 100));
        let rotation = usecase.record_rotation(CredentialRotationEvent {
            tenant_id: "ten_a".to_owned(),
            provider: CredentialProvider::OpenAi,
            secret_reference_text: "${openbao:secret/ten_a/intelligence/provider/openai}"
                .to_owned(),
            old_generation: first.handle.as_ref().unwrap().generation(),
            new_generation: 2,
            rotated_at_epoch_seconds: 110,
            evidence_ref: "rotation:openbao:2".to_owned(),
        });
        let replay = usecase.resolve(input("idem-rotated", 111));
        let (sidecar, _) = usecase.into_parts();

        assert_eq!(rotation.status, CredentialRotationStatus::Recorded);
        assert_eq!(replay.status, CredentialResolutionStatus::Denied);
        assert_eq!(
            replay.denial_kind,
            Some(CredentialResolutionDenialKind::CachedHandleExpired)
        );
        assert_eq!(sidecar.calls, 1);
    }

    #[test]
    fn idempotent_replay_after_handle_expiry_denies_stale_handle() {
        let mut usecase =
            CredentialResolverUsecase::new(FakeSidecar::default(), AuditCollector::default());

        let first = usecase.resolve(input("idem-expiring", 100));
        let stale_replay = usecase.resolve(input("idem-expiring", 161));
        let (sidecar, _) = usecase.into_parts();

        assert_eq!(first.status, CredentialResolutionStatus::Resolved);
        assert_eq!(stale_replay.status, CredentialResolutionStatus::Denied);
        assert_eq!(
            stale_replay.denial_kind,
            Some(CredentialResolutionDenialKind::CachedHandleExpired)
        );
        assert_eq!(sidecar.calls, 1);
    }

    struct WrongProviderSidecar;

    impl CredentialHandleIssuerPort for WrongProviderSidecar {
        fn issue_handle(
            &mut self,
            request: CredentialHandleRequest,
        ) -> Result<CredentialHandle, CredentialHandleIssueFailure> {
            CredentialHandle::issue(CredentialHandleIssueRequest {
                handle_id: "handle://ten_a/anthropic/gen-1".to_owned(),
                tenant_id: request.tenant_id,
                provider: CredentialProvider::Anthropic,
                audience: request.audience,
                issued_at_epoch_seconds: request.now_epoch_seconds,
                expires_at_epoch_seconds: request.now_epoch_seconds + 60,
                generation: 1,
                sidecar_signature_ref: "sigref://openbao/handle/1".to_owned(),
            })
            .map_err(|error| CredentialHandleIssueFailure {
                reason: format!("domain:{error:?}"),
                evidence_ref: "sidecar:domain-error".to_owned(),
            })
        }
    }

    #[test]
    fn sidecar_handle_binding_drift_denies_without_cache() {
        let mut usecase =
            CredentialResolverUsecase::new(WrongProviderSidecar, AuditCollector::default());

        let receipt = usecase.resolve(input("idem-wrong-handle", 100));
        let (_, audit) = usecase.into_parts();
        let debug = format!("{receipt:?}{:?}", audit.events);

        assert_eq!(receipt.status, CredentialResolutionStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(CredentialResolutionDenialKind::SidecarFailed)
        );
        assert!(receipt.handle.is_none());
        assert!(!debug.contains("handle://ten_a/anthropic/gen-1"));
    }

    #[test]
    fn invalid_secret_reference_denies_before_sidecar() {
        let mut usecase =
            CredentialResolverUsecase::new(FakeSidecar::default(), AuditCollector::default());
        let mut bad = input("idem-bad", 100);
        bad.secret_reference_text = "sk-test-raw".to_owned();

        let receipt = usecase.resolve(bad);
        let (sidecar, _) = usecase.into_parts();

        assert_eq!(receipt.status, CredentialResolutionStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(CredentialResolutionDenialKind::SecretReferenceInvalid)
        );
        assert_eq!(sidecar.calls, 0);
    }
}
