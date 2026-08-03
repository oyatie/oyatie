//! Intelligence attribution usecase foundation.
//!
//! This usecase adds idempotent, metadata-only orchestration around the
//! attribution domain layer for later cloud integration. It records in-memory
//! audit event metadata for request/rendered/denied/conflict paths, but performs
//! no retrieval, citation text rendering, model calls, filesystem/network I/O,
//! durable idempotency storage, or durable audit-chain emission.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

pub use intelligence_attribution_domain::{
    AttributionAudience, AttributionCitation, AttributionClaim, AttributionDataClass,
    AttributionDenialKind, AttributionDomainDecision, AttributionDomainDenialKind,
    AttributionDomainStatus, AttributionPolicyDecision, AttributionReport, AttributionRequest,
    AttributionSource, AttributionSourceKind, AttributionStatus, DomainAttributionRequest,
    plan_domain_attribution,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributionUsecaseInput {
    pub idempotency_key: String,           // data_class: INTERNAL_ONLY
    pub request: DomainAttributionRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttributionUsecaseStatus {
    Denied,
    Rendered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AttributionUsecaseDenialKind {
    DomainDenied,
    IdempotencyConflict,
    InvalidInput,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributionUsecaseReceipt {
    pub idempotency_key: String,          // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub principal_id: String,             // data_class: INTERNAL_ONLY
    pub attribution_surface: String,      // data_class: INTERNAL_ONLY
    pub output_ref: String,               // data_class: INTERNAL_ONLY
    pub status: AttributionUsecaseStatus, // data_class: PUBLIC
    pub denial_kind: Option<AttributionUsecaseDenialKind>, // data_class: INTERNAL_ONLY
    pub domain_denial_kind: Option<AttributionDomainDenialKind>, // data_class: INTERNAL_ONLY
    pub kernel_denial_kind: Option<AttributionDenialKind>, // data_class: INTERNAL_ONLY
    pub citation_count: usize,            // data_class: INTERNAL_ONLY
    pub citation_resource_refs: Vec<String>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AttributionAuditEventKind {
    AttributionDenied,
    AttributionRendered,
    AttributionRequested,
    IdempotencyConflict,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributionAuditEvent {
    pub kind: AttributionAuditEventKind, // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub principal_id: String,            // data_class: INTERNAL_ONLY
    pub attribution_surface: String,     // data_class: INTERNAL_ONLY
    pub output_ref: String,              // data_class: INTERNAL_ONLY
    pub idempotency_key: String,         // data_class: INTERNAL_ONLY
    pub status: Option<AttributionUsecaseStatus>, // data_class: PUBLIC
    pub citation_count: Option<usize>,   // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,      // data_class: INTERNAL_ONLY
}

#[derive(Default)]
pub struct IntelligenceAttributionUsecase {
    receipts_by_idempotency_key: BTreeMap<String, AttributionUsecaseReceipt>,
    intents_by_idempotency_key: BTreeMap<String, AttributionIntent>,
    audit_events: Vec<AttributionAuditEvent>,
}

impl IntelligenceAttributionUsecase {
    pub fn plan(&mut self, input: AttributionUsecaseInput) -> AttributionUsecaseReceipt {
        let invalid = invalid_usecase_input_reasons(&input);
        if !invalid.is_empty() {
            return invalid_receipt_from_input(
                &input,
                AttributionUsecaseDenialKind::InvalidInput,
                invalid,
                vec!["validation:intelligence-attribution-usecase-input".to_owned()],
            );
        }

        let domain_decision = plan_domain_attribution(input.request.clone());
        if let AttributionDomainDecision::Deny(denial) = &domain_decision
            && (denial.denial_kind == AttributionDomainDenialKind::InvalidInput
                || denial.kernel_denial_kind == Some(AttributionDenialKind::InvalidInput))
        {
            return receipt_from_domain_denial(
                &input.idempotency_key,
                denial,
                AttributionUsecaseDenialKind::InvalidInput,
            );
        }

        let intent = AttributionIntent::from_input(&input);
        if let Some(existing) = self.receipts_by_idempotency_key.get(&input.idempotency_key) {
            if self.intents_by_idempotency_key.get(&input.idempotency_key) == Some(&intent) {
                return existing.clone();
            }
            let receipt = invalid_receipt_from_input(
                &input,
                AttributionUsecaseDenialKind::IdempotencyConflict,
                vec!["idempotency key already used for different attribution intent".to_owned()],
                vec!["validation:intelligence-attribution-idempotency-conflict".to_owned()],
            );
            self.record_event(AttributionAuditEventKind::IdempotencyConflict, &receipt);
            return receipt;
        }

        self.record_event(
            AttributionAuditEventKind::AttributionRequested,
            &requested_receipt_for(&input),
        );

        let receipt = receipt_from_domain_decision(&input, domain_decision);
        match receipt.status {
            AttributionUsecaseStatus::Rendered => {
                self.record_event(AttributionAuditEventKind::AttributionRendered, &receipt)
            }
            AttributionUsecaseStatus::Denied => {
                self.record_event(AttributionAuditEventKind::AttributionDenied, &receipt)
            }
        }
        self.cache_receipt(&input.idempotency_key, intent, &receipt);
        receipt
    }

    pub fn audit_events(&self) -> &[AttributionAuditEvent] {
        &self.audit_events
    }

    pub fn cached_receipt_count(&self) -> usize {
        self.receipts_by_idempotency_key.len()
    }

    fn cache_receipt(
        &mut self,
        idempotency_key: &str,
        intent: AttributionIntent,
        receipt: &AttributionUsecaseReceipt,
    ) {
        self.intents_by_idempotency_key
            .insert(idempotency_key.to_owned(), intent);
        self.receipts_by_idempotency_key
            .insert(idempotency_key.to_owned(), receipt.clone());
    }

    fn record_event(
        &mut self,
        kind: AttributionAuditEventKind,
        receipt: &AttributionUsecaseReceipt,
    ) {
        self.audit_events.push(AttributionAuditEvent {
            kind,
            tenant_id: receipt.tenant_id.clone(),
            principal_id: receipt.principal_id.clone(),
            attribution_surface: receipt.attribution_surface.clone(),
            output_ref: receipt.output_ref.clone(),
            idempotency_key: receipt.idempotency_key.clone(),
            status: if kind == AttributionAuditEventKind::AttributionRequested {
                None
            } else {
                Some(receipt.status)
            },
            citation_count: if kind == AttributionAuditEventKind::AttributionRequested {
                None
            } else {
                Some(receipt.citation_count)
            },
            evidence_refs: sorted_unique(receipt.evidence_refs.clone()),
        });
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AttributionIntent {
    entries: Vec<String>,
}

impl AttributionIntent {
    fn from_input(input: &AttributionUsecaseInput) -> Self {
        let request = &input.request;
        let kernel = &request.request;
        let policy = &request.policy_decision;
        let mut entries = vec![
            canonical_entry("idempotency_key", &input.idempotency_key),
            canonical_entry("tenant_id", &request.tenant_id),
            canonical_entry("principal_id", &request.principal_id),
            canonical_entry("surface", &request.attribution_surface),
            canonical_entry("request_evidence", &request.request_evidence_ref),
            canonical_entry("trace_context", &request.trace_context_ref),
            canonical_entry("policy_decision_ref", &request.policy_decision_ref),
            canonical_entry("policy_decision_id", &policy.decision_id),
            canonical_entry("policy_tenant", &policy.tenant_id),
            canonical_entry("policy_principal", &policy.principal_id),
            canonical_vec_entry(
                "policy_surfaces",
                &sorted_unique(policy.allowed_surfaces.clone()),
            ),
            canonical_vec_entry(
                "policy_audiences",
                &audience_entries(&policy.allowed_audiences),
            ),
            canonical_vec_entry(
                "policy_source_kinds",
                &source_kind_entries(&policy.allowed_source_kinds),
            ),
            canonical_vec_entry(
                "policy_data_classes",
                &data_class_entries(&policy.allowed_data_classes),
            ),
            canonical_entry("policy_max_citations", &policy.max_citations.to_string()),
            canonical_entry(
                "policy_min_confidence",
                &policy.min_confidence_bps.to_string(),
            ),
            canonical_entry("policy_evidence", &policy.evidence_ref),
            canonical_entry(
                "attribution_registry_snapshot",
                &policy.attribution_registry_snapshot_ref,
            ),
            canonical_entry("kernel_tenant", &kernel.tenant_id),
            canonical_entry("kernel_output", &kernel.output_ref),
            canonical_entry("kernel_audience", audience_label(kernel.audience)),
            canonical_entry("kernel_policy_evidence", &kernel.policy_evidence_ref),
            canonical_entry("kernel_trace", &kernel.trace_context_ref),
            canonical_entry("kernel_max_citations", &kernel.max_citations.to_string()),
        ];
        entries.extend(source_entries(&kernel.sources));
        entries.extend(claim_entries(&kernel.claims));
        entries.sort();
        Self { entries }
    }
}

fn invalid_usecase_input_reasons(input: &AttributionUsecaseInput) -> Vec<String> {
    let mut reasons = Vec::new();
    require_metadata_ref("idempotency key", &input.idempotency_key, &mut reasons);
    let request = &input.request;
    require_opaque_ref("tenant id", &request.tenant_id, &mut reasons);
    require_opaque_ref("principal id", &request.principal_id, &mut reasons);
    require_opaque_ref(
        "attribution surface",
        &request.attribution_surface,
        &mut reasons,
    );
    require_opaque_ref(
        "request evidence ref",
        &request.request_evidence_ref,
        &mut reasons,
    );
    require_opaque_ref(
        "trace context ref",
        &request.trace_context_ref,
        &mut reasons,
    );
    require_opaque_ref(
        "policy decision ref",
        &request.policy_decision_ref,
        &mut reasons,
    );
    require_metadata_ref(
        "policy decision id",
        &request.policy_decision.decision_id,
        &mut reasons,
    );
    require_opaque_ref(
        "policy tenant id",
        &request.policy_decision.tenant_id,
        &mut reasons,
    );
    require_opaque_ref(
        "policy principal id",
        &request.policy_decision.principal_id,
        &mut reasons,
    );
    for surface in &request.policy_decision.allowed_surfaces {
        require_opaque_ref("policy allowed surface", surface, &mut reasons);
    }
    require_opaque_ref(
        "policy evidence ref",
        &request.policy_decision.evidence_ref,
        &mut reasons,
    );
    require_opaque_ref(
        "attribution registry snapshot ref",
        &request.policy_decision.attribution_registry_snapshot_ref,
        &mut reasons,
    );
    let kernel = &request.request;
    require_opaque_ref("kernel tenant id", &kernel.tenant_id, &mut reasons);
    require_opaque_ref("kernel output ref", &kernel.output_ref, &mut reasons);
    require_opaque_ref(
        "kernel policy evidence ref",
        &kernel.policy_evidence_ref,
        &mut reasons,
    );
    require_opaque_ref(
        "kernel trace context ref",
        &kernel.trace_context_ref,
        &mut reasons,
    );
    for source in &kernel.sources {
        require_metadata_ref("source id", &source.source_id, &mut reasons);
        require_opaque_ref("source resource ref", &source.resource_ref, &mut reasons);
        require_opaque_ref("source title ref", &source.title_ref, &mut reasons);
        require_opaque_ref("source evidence ref", &source.evidence_ref, &mut reasons);
    }
    for claim in &kernel.claims {
        require_metadata_ref("claim id", &claim.claim_id, &mut reasons);
        require_opaque_ref(
            "claim answer segment ref",
            &claim.answer_segment_ref,
            &mut reasons,
        );
        for source_id in &claim.source_ids {
            require_metadata_ref("claim source id", source_id, &mut reasons);
        }
    }
    sorted_unique(reasons)
}

fn requested_receipt_for(input: &AttributionUsecaseInput) -> AttributionUsecaseReceipt {
    AttributionUsecaseReceipt {
        idempotency_key: input.idempotency_key.clone(),
        tenant_id: input.request.tenant_id.clone(),
        principal_id: input.request.principal_id.clone(),
        attribution_surface: input.request.attribution_surface.clone(),
        output_ref: input.request.request.output_ref.clone(),
        status: AttributionUsecaseStatus::Rendered,
        denial_kind: None,
        domain_denial_kind: None,
        kernel_denial_kind: None,
        citation_count: 0,
        citation_resource_refs: Vec::new(),
        evidence_refs: sorted_unique(vec![
            input.request.request_evidence_ref.clone(),
            input.request.trace_context_ref.clone(),
            input.request.policy_decision_ref.clone(),
        ]),
    }
}

fn receipt_from_domain_decision(
    input: &AttributionUsecaseInput,
    decision: AttributionDomainDecision,
) -> AttributionUsecaseReceipt {
    match decision {
        AttributionDomainDecision::Report(report) => {
            let citation_resource_refs = sorted_unique(
                report
                    .report
                    .citations
                    .iter()
                    .map(|citation| citation.resource_ref.clone())
                    .collect(),
            );
            AttributionUsecaseReceipt {
                idempotency_key: input.idempotency_key.clone(),
                tenant_id: report.tenant_id,
                principal_id: report.principal_id,
                attribution_surface: report.attribution_surface,
                output_ref: input.request.request.output_ref.clone(),
                status: AttributionUsecaseStatus::Rendered,
                denial_kind: None,
                domain_denial_kind: None,
                kernel_denial_kind: None,
                citation_count: report.report.citations.len(),
                citation_resource_refs,
                evidence_refs: sorted_unique(report.evidence_refs),
            }
        }
        AttributionDomainDecision::Deny(denial) => receipt_from_domain_denial(
            &input.idempotency_key,
            &denial,
            AttributionUsecaseDenialKind::DomainDenied,
        ),
    }
}

fn receipt_from_domain_denial(
    idempotency_key: &str,
    denial: &intelligence_attribution_domain::AttributionDomainDenial,
    usecase_denial_kind: AttributionUsecaseDenialKind,
) -> AttributionUsecaseReceipt {
    AttributionUsecaseReceipt {
        idempotency_key: idempotency_key.to_owned(),
        tenant_id: denial.tenant_id.clone(),
        principal_id: denial.principal_id.clone(),
        attribution_surface: denial.attribution_surface.clone(),
        output_ref: denial.output_ref.clone(),
        status: AttributionUsecaseStatus::Denied,
        denial_kind: Some(usecase_denial_kind),
        domain_denial_kind: Some(denial.denial_kind),
        kernel_denial_kind: denial.kernel_denial_kind,
        citation_count: 0,
        citation_resource_refs: Vec::new(),
        evidence_refs: sorted_unique(denial.evidence_refs.clone()),
    }
}

fn invalid_receipt_from_input(
    input: &AttributionUsecaseInput,
    denial_kind: AttributionUsecaseDenialKind,
    reasons: Vec<String>,
    evidence_refs: Vec<String>,
) -> AttributionUsecaseReceipt {
    AttributionUsecaseReceipt {
        idempotency_key: safe_metadata(&input.idempotency_key, "redacted-invalid-idempotency-key"),
        tenant_id: safe_ref(&input.request.tenant_id, "redacted-invalid-tenant-id"),
        principal_id: safe_ref(&input.request.principal_id, "redacted-invalid-principal-id"),
        attribution_surface: safe_ref(
            &input.request.attribution_surface,
            "redacted-invalid-attribution-surface",
        ),
        output_ref: safe_ref(
            &input.request.request.output_ref,
            "redacted-invalid-output-ref",
        ),
        status: AttributionUsecaseStatus::Denied,
        denial_kind: Some(denial_kind),
        domain_denial_kind: None,
        kernel_denial_kind: None,
        citation_count: 0,
        citation_resource_refs: Vec::new(),
        evidence_refs: sorted_unique([evidence_refs, reasons].concat()),
    }
}

fn source_entries(sources: &[AttributionSource]) -> Vec<String> {
    let mut entries: Vec<String> = sources
        .iter()
        .map(|source| {
            canonical_vec_entry(
                "source",
                &[
                    canonical_entry("source_id", &source.source_id),
                    canonical_entry("resource", &source.resource_ref),
                    canonical_entry("title", &source.title_ref),
                    canonical_entry("kind", source_kind_label(source.source_kind)),
                    canonical_entry("data_class", data_class_label(source.data_class)),
                    canonical_entry("evidence", &source.evidence_ref),
                    canonical_entry("freshness", &source.freshness_epoch_seconds.to_string()),
                ],
            )
        })
        .collect();
    entries.sort();
    entries
}

fn claim_entries(claims: &[AttributionClaim]) -> Vec<String> {
    let mut entries: Vec<String> = claims
        .iter()
        .map(|claim| {
            canonical_vec_entry(
                "claim",
                &[
                    canonical_entry("claim_id", &claim.claim_id),
                    canonical_entry("segment", &claim.answer_segment_ref),
                    canonical_vec_entry("sources", &sorted_unique(claim.source_ids.clone())),
                    canonical_entry("confidence", &claim.confidence_bps.to_string()),
                ],
            )
        })
        .collect();
    entries.sort();
    entries
}

fn audience_entries(values: &[AttributionAudience]) -> Vec<String> {
    let mut entries: Vec<String> = values
        .iter()
        .map(|value| audience_label(*value).to_owned())
        .collect();
    entries.sort();
    entries.dedup();
    entries
}

fn source_kind_entries(values: &[AttributionSourceKind]) -> Vec<String> {
    let mut entries: Vec<String> = values
        .iter()
        .map(|value| source_kind_label(*value).to_owned())
        .collect();
    entries.sort();
    entries.dedup();
    entries
}

fn data_class_entries(values: &[AttributionDataClass]) -> Vec<String> {
    let mut entries: Vec<String> = values
        .iter()
        .map(|value| data_class_label(*value).to_owned())
        .collect();
    entries.sort();
    entries.dedup();
    entries
}

fn audience_label(value: AttributionAudience) -> &'static str {
    match value {
        AttributionAudience::External => "external",
        AttributionAudience::Internal => "internal",
    }
}

fn source_kind_label(value: AttributionSourceKind) -> &'static str {
    match value {
        AttributionSourceKind::KnowledgeGraph => "knowledge-graph",
        AttributionSourceKind::PolicyDocument => "policy-document",
        AttributionSourceKind::RetrievalDocument => "retrieval-document",
        AttributionSourceKind::ToolResult => "tool-result",
    }
}

fn data_class_label(value: AttributionDataClass) -> &'static str {
    match value {
        AttributionDataClass::Confidential => "confidential",
        AttributionDataClass::Internal => "internal",
        AttributionDataClass::Public => "public",
        AttributionDataClass::Restricted => "restricted",
    }
}

fn canonical_entry(label: &str, value: &str) -> String {
    format!("{}:{}:{}", label.len(), label, value.len()) + ":" + value
}

fn canonical_vec_entry(label: &str, values: &[String]) -> String {
    let joined = values
        .iter()
        .map(|value| format!("{}:{}", value.len(), value))
        .collect::<Vec<_>>()
        .join("|");
    canonical_entry(label, &joined)
}

fn require_metadata_ref(label: &str, value: &str, reasons: &mut Vec<String>) {
    if !is_safe_metadata_ref(value) {
        reasons.push(format!("{label} must be audit-safe metadata"));
    }
}

fn require_opaque_ref(label: &str, value: &str, reasons: &mut Vec<String>) {
    if !is_safe_opaque_ref(value) {
        reasons.push(format!("{label} must be an opaque ref"));
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
        || lower.contains("document text")
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
    fn renders_authorized_attribution_with_metadata_audit() {
        let mut usecase = IntelligenceAttributionUsecase::default();
        let receipt = usecase.plan(sample_input("idem:attr:1"));

        assert_eq!(receipt.status, AttributionUsecaseStatus::Rendered);
        assert_eq!(receipt.citation_count, 2);
        assert_eq!(receipt.citation_resource_refs.len(), 2);
        assert_eq!(usecase.cached_receipt_count(), 1);
        assert_eq!(usecase.audit_events().len(), 2);
        assert_eq!(
            usecase.audit_events()[0].kind,
            AttributionAuditEventKind::AttributionRequested
        );
        assert_eq!(usecase.audit_events()[0].status, None);
        assert_eq!(
            usecase.audit_events()[1].kind,
            AttributionAuditEventKind::AttributionRendered
        );
        assert_eq!(usecase.audit_events()[1].citation_count, Some(2));
    }

    #[test]
    fn invalid_raw_metadata_denies_before_audit_side_effect() {
        let mut input = sample_input("idem:attr:raw");
        input.request.request.output_ref = "raw output model answer".to_owned();
        input.request.request.sources[0].resource_ref = "sk-test-secret".to_owned();
        let mut usecase = IntelligenceAttributionUsecase::default();

        let receipt = usecase.plan(input);
        let debug = format!("{receipt:?}{:?}", usecase.audit_events());

        assert_eq!(receipt.status, AttributionUsecaseStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(AttributionUsecaseDenialKind::InvalidInput)
        );
        assert_eq!(receipt.output_ref, "redacted-invalid-output-ref");
        assert!(usecase.audit_events().is_empty());
        assert_eq!(usecase.cached_receipt_count(), 0);
        assert!(!debug.contains("raw output model answer"));
        assert!(!debug.contains("sk-test-secret"));
    }

    #[test]
    fn idempotent_replay_and_conflict_are_deterministic() {
        let mut usecase = IntelligenceAttributionUsecase::default();
        let first = usecase.plan(sample_input("idem:attr:2"));
        let replay = usecase.plan(sample_input("idem:attr:2"));
        let mut changed = sample_input("idem:attr:2");
        changed.request.request.claims[0].confidence_bps = 8_100;
        let conflict = usecase.plan(changed);

        assert_eq!(first, replay);
        assert_eq!(conflict.status, AttributionUsecaseStatus::Denied);
        assert_eq!(
            conflict.denial_kind,
            Some(AttributionUsecaseDenialKind::IdempotencyConflict)
        );
        assert_eq!(usecase.cached_receipt_count(), 1);
        assert_eq!(usecase.audit_events().len(), 3);
        assert_eq!(
            usecase.audit_events()[2].kind,
            AttributionAuditEventKind::IdempotencyConflict
        );
    }

    #[test]
    fn idempotent_replay_treats_sources_and_claims_as_order_independent_sets() {
        let mut usecase = IntelligenceAttributionUsecase::default();
        let first = usecase.plan(sample_input("idem:attr:3"));
        let mut reordered = sample_input("idem:attr:3");
        reordered.request.request.sources.reverse();
        reordered.request.request.claims.reverse();
        reordered.request.request.claims[0].source_ids.reverse();
        let replay = usecase.plan(reordered);

        assert_eq!(first, replay);
        assert_eq!(usecase.audit_events().len(), 2);
    }

    #[test]
    fn domain_policy_denial_records_fail_closed_audit() {
        let mut input = sample_input("idem:attr:4");
        input.request.policy_decision.allowed_audiences = vec![AttributionAudience::Internal];
        let mut usecase = IntelligenceAttributionUsecase::default();

        let receipt = usecase.plan(input);

        assert_eq!(receipt.status, AttributionUsecaseStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(AttributionUsecaseDenialKind::DomainDenied)
        );
        assert_eq!(
            receipt.domain_denial_kind,
            Some(AttributionDomainDenialKind::AudienceDenied)
        );
        assert_eq!(usecase.cached_receipt_count(), 1);
        assert_eq!(usecase.audit_events().len(), 2);
        assert_eq!(
            usecase.audit_events()[1].kind,
            AttributionAuditEventKind::AttributionDenied
        );
    }

    #[test]
    fn kernel_missing_source_denial_records_fail_closed_audit() {
        let mut input = sample_input("idem:attr:5");
        input.request.request.claims[0]
            .source_ids
            .push("src-missing".to_owned());
        let mut usecase = IntelligenceAttributionUsecase::default();

        let receipt = usecase.plan(input);

        assert_eq!(receipt.status, AttributionUsecaseStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(AttributionUsecaseDenialKind::DomainDenied)
        );
        assert_eq!(
            receipt.domain_denial_kind,
            Some(AttributionDomainDenialKind::KernelDenied)
        );
        assert_eq!(
            receipt.kernel_denial_kind,
            Some(AttributionDenialKind::MissingSource)
        );
        assert_eq!(
            usecase.audit_events()[1].kind,
            AttributionAuditEventKind::AttributionDenied
        );
    }

    fn sample_input(idempotency_key: &str) -> AttributionUsecaseInput {
        AttributionUsecaseInput {
            idempotency_key: idempotency_key.to_owned(),
            request: sample_domain_request(),
        }
    }

    fn sample_domain_request() -> DomainAttributionRequest {
        DomainAttributionRequest {
            tenant_id: "tenant:alpha".to_owned(),
            principal_id: "principal:attribution-owner".to_owned(),
            attribution_surface: "surface:dispatch-response".to_owned(),
            request_evidence_ref: "request:evidence:attribution-usecase:1".to_owned(),
            trace_context_ref: "trace:attribution-usecase:1".to_owned(),
            policy_decision_ref: "policy:evidence:attribution-usecase:1".to_owned(),
            policy_decision: sample_policy(),
            request: sample_kernel_request(),
        }
    }

    fn sample_policy() -> AttributionPolicyDecision {
        AttributionPolicyDecision {
            decision_id: "attribution-policy-decision:usecase:1".to_owned(),
            tenant_id: "tenant:alpha".to_owned(),
            principal_id: "principal:attribution-owner".to_owned(),
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
            evidence_ref: "policy:evidence:attribution-usecase:1".to_owned(),
            attribution_registry_snapshot_ref: "attribution-registry:snapshot:usecase:1".to_owned(),
        }
    }

    fn sample_kernel_request() -> AttributionRequest {
        AttributionRequest {
            tenant_id: "tenant:alpha".to_owned(),
            output_ref: "answer://responses/resp-usecase-1".to_owned(),
            audience: AttributionAudience::External,
            policy_evidence_ref: "policy:evidence:attribution-usecase:1".to_owned(),
            trace_context_ref: "trace:attribution-usecase:1".to_owned(),
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
                    answer_segment_ref: "answer-segment://resp-usecase-1/2".to_owned(),
                    source_ids: vec!["src-doc-refund".to_owned()],
                    confidence_bps: 9_000,
                },
                AttributionClaim {
                    claim_id: "claim-1".to_owned(),
                    answer_segment_ref: "answer-segment://resp-usecase-1/1".to_owned(),
                    source_ids: vec!["src-kg-policy".to_owned()],
                    confidence_bps: 9_200,
                },
            ],
        }
    }
}
