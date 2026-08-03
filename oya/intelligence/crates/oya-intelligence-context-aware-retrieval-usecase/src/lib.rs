//! Intelligence context-aware retrieval usecase foundation.
//!
//! This crate orchestrates context-aware retrieval planning for later cloud
//! integration. It validates metadata before side effects, preserves idempotent
//! receipts, emits metadata-only audit events, and delegates only to the domain
//! policy/metadata layer. It deliberately has no vector-store, embedding,
//! ontology/KG runtime, document-fetch, filesystem, network, durable store, or
//! policy-engine runtime integration.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

pub use intelligence_context_aware_retrieval_domain::{
    ContextAudience, ContextCandidate, ContextDataClass, ContextRetrievalDomainDecision,
    ContextRetrievalDomainDenialKind, ContextRetrievalPolicyDecision, ContextRetrievalRequest,
    ContextRetrievalStatus, ContextSourceKind, DomainContextRetrievalRequest,
    plan_domain_context_retrieval,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextRetrievalUsecaseInput {
    pub idempotency_key: String,                // data_class: INTERNAL_ONLY
    pub request: DomainContextRetrievalRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextRetrievalUsecaseStatus {
    Planned,
    Denied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextRetrievalUsecaseDenialKind {
    DomainDenied,
    IdempotencyConflict,
    InvalidInput,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextRetrievalUsecaseReceipt {
    pub idempotency_key: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub principal_id: String,                  // data_class: INTERNAL_ONLY
    pub query_surface: String,                 // data_class: INTERNAL_ONLY
    pub query_ref: String,                     // data_class: INTERNAL_ONLY
    pub status: ContextRetrievalUsecaseStatus, // data_class: PUBLIC
    pub denial_kind: Option<ContextRetrievalUsecaseDenialKind>, // data_class: INTERNAL_ONLY
    pub denial_reasons: Vec<String>,           // data_class: INTERNAL_ONLY
    pub plan_resource_refs: Vec<String>,       // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextRetrievalAuditEventKind {
    RetrievalRequested,
    RetrievalPlanned,
    RetrievalDenied,
    IdempotencyConflict,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextRetrievalAuditEvent {
    pub kind: ContextRetrievalAuditEventKind, // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub principal_id: String,                 // data_class: INTERNAL_ONLY
    pub idempotency_key: String,              // data_class: INTERNAL_ONLY
    pub query_surface: String,                // data_class: INTERNAL_ONLY
    pub query_ref: String,                    // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ContextAwareRetrievalUsecase {
    receipts_by_idempotency_key: BTreeMap<String, ContextRetrievalUsecaseReceipt>,
    intents_by_idempotency_key: BTreeMap<String, RetrievalIntent>,
    audit_events: Vec<ContextRetrievalAuditEvent>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RetrievalIntent {
    canonical: String,
}

impl ContextAwareRetrievalUsecase {
    pub fn plan(&mut self, input: ContextRetrievalUsecaseInput) -> ContextRetrievalUsecaseReceipt {
        let invalid = invalid_input_reasons(&input);
        if !invalid.is_empty() {
            return denied_receipt(
                safe_metadata(&input.idempotency_key, "redacted-invalid-idempotency-key"),
                safe_tenant(&input.request.request.tenant_id),
                safe_metadata(&input.request.principal_id, "redacted-invalid-principal-id"),
                safe_metadata(
                    &input.request.query_surface,
                    "redacted-invalid-query-surface",
                ),
                safe_ref(
                    &input.request.request.query_ref,
                    "redacted-invalid-query-ref",
                ),
                ContextRetrievalUsecaseDenialKind::InvalidInput,
                invalid,
                vec!["validation:intelligence-context-aware-retrieval-usecase-input".to_owned()],
                Vec::new(),
            );
        }

        let intent = RetrievalIntent {
            canonical: canonical_intent(&input),
        };
        if let Some(existing) = self.receipts_by_idempotency_key.get(&input.idempotency_key) {
            if self.intents_by_idempotency_key.get(&input.idempotency_key) == Some(&intent) {
                return existing.clone();
            }
            let receipt = denied_receipt_from_valid_input(
                &input,
                ContextRetrievalUsecaseDenialKind::IdempotencyConflict,
                vec![
                    "idempotency key already used for different context retrieval intent"
                        .to_owned(),
                ],
                vec![
                    input.request.request.request_evidence_ref.clone(),
                    "validation:intelligence-context-aware-retrieval-idempotency-conflict"
                        .to_owned(),
                ],
                Vec::new(),
            );
            self.record_event(
                ContextRetrievalAuditEventKind::IdempotencyConflict,
                &input,
                &receipt,
            );
            return receipt;
        }

        let request_evidence = canonical_request_evidence_refs(&input);
        self.record_audit_event(ContextRetrievalAuditEvent {
            kind: ContextRetrievalAuditEventKind::RetrievalRequested,
            tenant_id: input.request.request.tenant_id.clone(),
            principal_id: input.request.principal_id.clone(),
            idempotency_key: input.idempotency_key.clone(),
            query_surface: input.request.query_surface.clone(),
            query_ref: input.request.request.query_ref.clone(),
            evidence_refs: request_evidence,
        });

        let receipt = match plan_domain_context_retrieval(input.request.clone()) {
            ContextRetrievalDomainDecision::Plan(plan) => ContextRetrievalUsecaseReceipt {
                idempotency_key: input.idempotency_key.clone(),
                tenant_id: plan.tenant_id,
                principal_id: input.request.principal_id.clone(),
                query_surface: input.request.query_surface.clone(),
                query_ref: plan.query_ref,
                status: ContextRetrievalUsecaseStatus::Planned,
                denial_kind: None,
                denial_reasons: Vec::new(),
                plan_resource_refs: unique_preserve_order(
                    plan.items
                        .iter()
                        .map(|item| item.resource_ref.clone())
                        .collect(),
                ),
                evidence_refs: sorted_unique(plan.evidence_refs),
            },
            ContextRetrievalDomainDecision::Deny(denial) => denied_receipt_from_valid_input(
                &input,
                ContextRetrievalUsecaseDenialKind::DomainDenied,
                vec![format!("domain::{:?}", denial.denial_kind)],
                denial.evidence_refs,
                Vec::new(),
            ),
        };

        let event_kind = match receipt.status {
            ContextRetrievalUsecaseStatus::Planned => {
                ContextRetrievalAuditEventKind::RetrievalPlanned
            }
            ContextRetrievalUsecaseStatus::Denied => {
                ContextRetrievalAuditEventKind::RetrievalDenied
            }
        };
        self.record_event(event_kind, &input, &receipt);
        self.cache_receipt(&input.idempotency_key, intent, &receipt);
        receipt
    }

    pub fn audit_events(&self) -> &[ContextRetrievalAuditEvent] {
        &self.audit_events
    }

    pub fn receipt_count(&self) -> usize {
        self.receipts_by_idempotency_key.len()
    }

    fn cache_receipt(
        &mut self,
        idempotency_key: &str,
        intent: RetrievalIntent,
        receipt: &ContextRetrievalUsecaseReceipt,
    ) {
        self.intents_by_idempotency_key
            .insert(idempotency_key.to_owned(), intent);
        self.receipts_by_idempotency_key
            .insert(idempotency_key.to_owned(), receipt.clone());
    }

    fn record_event(
        &mut self,
        kind: ContextRetrievalAuditEventKind,
        input: &ContextRetrievalUsecaseInput,
        receipt: &ContextRetrievalUsecaseReceipt,
    ) {
        self.record_audit_event(ContextRetrievalAuditEvent {
            kind,
            tenant_id: receipt.tenant_id.clone(),
            principal_id: input.request.principal_id.clone(),
            idempotency_key: input.idempotency_key.clone(),
            query_surface: input.request.query_surface.clone(),
            query_ref: input.request.request.query_ref.clone(),
            evidence_refs: receipt.evidence_refs.clone(),
        });
    }

    fn record_audit_event(&mut self, mut event: ContextRetrievalAuditEvent) {
        event.evidence_refs = sorted_unique(event.evidence_refs);
        self.audit_events.push(event);
    }
}

fn invalid_input_reasons(input: &ContextRetrievalUsecaseInput) -> Vec<String> {
    let mut reasons = Vec::new();
    require_metadata_ref("idempotency key", &input.idempotency_key, &mut reasons);
    require_metadata_ref("principal id", &input.request.principal_id, &mut reasons);
    require_metadata_ref("query surface", &input.request.query_surface, &mut reasons);

    let request = &input.request.request;
    require_tenant("request tenant id", &request.tenant_id, &mut reasons);
    require_resource_ref("query ref", &request.query_ref, &mut reasons);
    require_evidence_ref(
        "request evidence ref",
        &request.request_evidence_ref,
        &mut reasons,
    );
    require_evidence_ref(
        "trace context ref",
        &request.trace_context_ref,
        &mut reasons,
    );
    require_evidence_ref(
        "request policy decision ref",
        &request.policy_decision_ref,
        &mut reasons,
    );
    if request.allowed_source_kinds.is_empty() {
        reasons.push("allowed source kinds are required".to_owned());
    }
    if request.max_context_items == 0 || request.max_context_items > 32 {
        reasons.push("max context items must be in 1..=32".to_owned());
    }
    if request.candidates.is_empty() {
        reasons.push("retrieval candidates are required".to_owned());
    }
    for candidate in &request.candidates {
        require_tenant("candidate tenant id", &candidate.tenant_id, &mut reasons);
        require_resource_ref(
            "candidate resource ref",
            &candidate.resource_ref,
            &mut reasons,
        );
        require_evidence_ref(
            "candidate evidence ref",
            &candidate.evidence_ref,
            &mut reasons,
        );
        if candidate.relevance_millis > 1000 {
            reasons.push("candidate relevance must be 0..=1000 millis".to_owned());
        }
    }

    let policy = &input.request.policy_decision;
    require_metadata_ref("policy decision id", &policy.decision_id, &mut reasons);
    require_tenant("policy tenant id", &policy.tenant_id, &mut reasons);
    require_metadata_ref("policy principal id", &policy.principal_id, &mut reasons);
    if policy.allowed_surfaces.is_empty() {
        reasons.push("policy allowed surfaces are required".to_owned());
    }
    for surface in &policy.allowed_surfaces {
        require_metadata_ref("policy allowed surface", surface, &mut reasons);
    }
    if policy.allowed_source_kinds.is_empty() {
        reasons.push("policy allowed source kinds are required".to_owned());
    }
    if policy.max_context_items == 0 || policy.max_context_items > 32 {
        reasons.push("policy max context items must be in 1..=32".to_owned());
    }
    require_evidence_ref("policy evidence ref", &policy.evidence_ref, &mut reasons);
    require_evidence_ref(
        "retrieval index snapshot ref",
        &policy.retrieval_index_snapshot_ref,
        &mut reasons,
    );
    sorted_unique(reasons)
}

fn canonical_request_evidence_refs(input: &ContextRetrievalUsecaseInput) -> Vec<String> {
    sorted_unique(vec![
        input.request.request.request_evidence_ref.clone(),
        input.request.request.trace_context_ref.clone(),
        input.request.request.policy_decision_ref.clone(),
        input.request.policy_decision.evidence_ref.clone(),
        input
            .request
            .policy_decision
            .retrieval_index_snapshot_ref
            .clone(),
    ])
}

fn canonical_intent(input: &ContextRetrievalUsecaseInput) -> String {
    let mut candidate_entries: Vec<String> = input
        .request
        .request
        .candidates
        .iter()
        .map(|candidate| {
            [
                canonical_entry("tenant", &candidate.tenant_id),
                canonical_entry("source", &format!("{:?}", candidate.source_kind)),
                canonical_entry("resource", &candidate.resource_ref),
                canonical_entry("evidence", &candidate.evidence_ref),
                canonical_entry("data_class", &format!("{:?}", candidate.data_class)),
                canonical_entry("observed", &candidate.observed_at_epoch_seconds.to_string()),
                canonical_entry("relevance", &candidate.relevance_millis.to_string()),
            ]
            .concat()
        })
        .collect();
    candidate_entries.sort();

    [
        canonical_entry("idempotency_key", &input.idempotency_key),
        canonical_entry("principal_id", &input.request.principal_id),
        canonical_entry("query_surface", &input.request.query_surface),
        canonical_entry("tenant_id", &input.request.request.tenant_id),
        canonical_entry("query_ref", &input.request.request.query_ref),
        canonical_entry(
            "request_evidence",
            &input.request.request.request_evidence_ref,
        ),
        canonical_entry("trace_context", &input.request.request.trace_context_ref),
        canonical_entry("request_policy", &input.request.request.policy_decision_ref),
        canonical_entry("audience", &format!("{:?}", input.request.request.audience)),
        canonical_vec_entry(
            "request_sources",
            &source_kind_entries(&input.request.request.allowed_source_kinds),
        ),
        canonical_entry(
            "max_items",
            &input.request.request.max_context_items.to_string(),
        ),
        canonical_entry(
            "freshness_floor",
            &input
                .request
                .request
                .freshness_floor_epoch_seconds
                .to_string(),
        ),
        canonical_entry(
            "policy_decision_id",
            &input.request.policy_decision.decision_id,
        ),
        canonical_entry("policy_tenant", &input.request.policy_decision.tenant_id),
        canonical_entry(
            "policy_principal",
            &input.request.policy_decision.principal_id,
        ),
        canonical_vec_entry(
            "policy_surfaces",
            &input.request.policy_decision.allowed_surfaces,
        ),
        canonical_vec_entry(
            "policy_sources",
            &source_kind_entries(&input.request.policy_decision.allowed_source_kinds),
        ),
        canonical_entry(
            "policy_max_items",
            &input.request.policy_decision.max_context_items.to_string(),
        ),
        canonical_entry(
            "policy_freshness_floor",
            &input
                .request
                .policy_decision
                .freshness_floor_epoch_seconds
                .to_string(),
        ),
        canonical_entry(
            "policy_evidence",
            &input.request.policy_decision.evidence_ref,
        ),
        canonical_entry(
            "retrieval_index_snapshot",
            &input.request.policy_decision.retrieval_index_snapshot_ref,
        ),
        canonical_vec_entry("candidates", &candidate_entries),
    ]
    .concat()
}

fn source_kind_entries(source_kinds: &[ContextSourceKind]) -> Vec<String> {
    let mut entries = source_kinds
        .iter()
        .map(|kind| format!("{kind:?}"))
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

fn denied_receipt_from_valid_input(
    input: &ContextRetrievalUsecaseInput,
    denial_kind: ContextRetrievalUsecaseDenialKind,
    denial_reasons: Vec<String>,
    evidence_refs: Vec<String>,
    plan_resource_refs: Vec<String>,
) -> ContextRetrievalUsecaseReceipt {
    denied_receipt(
        input.idempotency_key.clone(),
        input.request.request.tenant_id.clone(),
        input.request.principal_id.clone(),
        input.request.query_surface.clone(),
        input.request.request.query_ref.clone(),
        denial_kind,
        denial_reasons,
        evidence_refs,
        plan_resource_refs,
    )
}

#[allow(clippy::too_many_arguments)]
fn denied_receipt(
    idempotency_key: String,
    tenant_id: String,
    principal_id: String,
    query_surface: String,
    query_ref: String,
    denial_kind: ContextRetrievalUsecaseDenialKind,
    denial_reasons: Vec<String>,
    evidence_refs: Vec<String>,
    plan_resource_refs: Vec<String>,
) -> ContextRetrievalUsecaseReceipt {
    ContextRetrievalUsecaseReceipt {
        idempotency_key,
        tenant_id,
        principal_id,
        query_surface,
        query_ref,
        status: ContextRetrievalUsecaseStatus::Denied,
        denial_kind: Some(denial_kind),
        denial_reasons: sorted_unique(denial_reasons),
        plan_resource_refs: unique_preserve_order(plan_resource_refs),
        evidence_refs: sorted_unique(evidence_refs),
    }
}

fn require_tenant(label: &str, value: &str, reasons: &mut Vec<String>) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        reasons.push(format!("{label} is required"));
    } else if value != trimmed
        || !trimmed.starts_with("ten_")
        || contains_whitespace(trimmed)
        || trimmed.contains('/')
        || contains_raw_secret_material(trimmed)
        || contains_raw_content_material(trimmed)
    {
        reasons.push(format!("{label} is invalid"));
    }
}

fn require_metadata_ref(label: &str, value: &str, reasons: &mut Vec<String>) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        reasons.push(format!("{label} is required"));
    } else if value != trimmed
        || contains_whitespace(trimmed)
        || contains_raw_secret_material(trimmed)
        || contains_raw_content_material(trimmed)
    {
        reasons.push(format!("{label} must be audit-safe metadata"));
    }
}

fn require_evidence_ref(label: &str, value: &str, reasons: &mut Vec<String>) {
    require_opaque_ref(label, value, "opaque evidence ref", reasons);
}

fn require_resource_ref(label: &str, value: &str, reasons: &mut Vec<String>) {
    require_opaque_ref(label, value, "opaque resource ref", reasons);
}

fn require_opaque_ref(label: &str, value: &str, kind: &str, reasons: &mut Vec<String>) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        reasons.push(format!("{label} is required"));
    } else if value != trimmed
        || contains_whitespace(trimmed)
        || contains_raw_secret_material(trimmed)
        || contains_raw_content_material(trimmed)
        || !trimmed.contains(':')
    {
        reasons.push(format!("{label} must be an {kind}"));
    }
}

fn safe_tenant(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || value != trimmed
        || !trimmed.starts_with("ten_")
        || contains_whitespace(trimmed)
        || trimmed.contains('/')
        || contains_raw_secret_material(trimmed)
        || contains_raw_content_material(trimmed)
    {
        "redacted-invalid-tenant-id".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn safe_metadata(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || value != trimmed
        || contains_whitespace(trimmed)
        || contains_raw_secret_material(trimmed)
        || contains_raw_content_material(trimmed)
    {
        fallback.to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn safe_ref(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || value != trimmed
        || contains_whitespace(trimmed)
        || contains_raw_secret_material(trimmed)
        || contains_raw_content_material(trimmed)
        || !trimmed.contains(':')
    {
        fallback.to_owned()
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

fn contains_raw_content_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw query")
        || lower.contains("customer message")
        || lower.contains("write an email")
        || lower.contains("model answer")
}

fn canonical_entry(label: &str, value: &str) -> String {
    format!("{}:{}{}:{}", label.len(), label, value.len(), value)
}

fn canonical_vec_entry(label: &str, values: &[String]) -> String {
    let mut encoded = canonical_entry(label, &values.len().to_string());
    for value in values {
        encoded.push_str(&canonical_entry("item", value));
    }
    encoded
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn unique_preserve_order(values: Vec<String>) -> Vec<String> {
    let mut unique = Vec::new();
    for value in values {
        if !value.trim().is_empty() && !unique.contains(&value) {
            unique.push(value);
        }
    }
    unique
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(
        tenant_id: &str,
        source_kind: ContextSourceKind,
        resource_ref: &str,
        evidence_ref: &str,
        data_class: ContextDataClass,
        observed_at_epoch_seconds: u64,
        relevance_millis: u16,
    ) -> ContextCandidate {
        ContextCandidate {
            tenant_id: tenant_id.to_owned(),
            source_kind,
            resource_ref: resource_ref.to_owned(),
            evidence_ref: evidence_ref.to_owned(),
            data_class,
            observed_at_epoch_seconds,
            relevance_millis,
        }
    }

    fn sample_input(idempotency_key: &str) -> ContextRetrievalUsecaseInput {
        ContextRetrievalUsecaseInput {
            idempotency_key: idempotency_key.to_owned(),
            request: DomainContextRetrievalRequest {
                principal_id: "principal-ctx-1".to_owned(),
                query_surface: "intelligence.context-aware-retrieval.pre".to_owned(),
                request: ContextRetrievalRequest {
                    tenant_id: "ten_a".to_owned(),
                    query_ref: "queryref://opaque/1".to_owned(),
                    request_evidence_ref: "req:ctx-usecase:1".to_owned(),
                    trace_context_ref: "trace:ctx-usecase:1".to_owned(),
                    policy_decision_ref: "cedar:ctx-usecase:allow".to_owned(),
                    audience: ContextAudience::TenantOperator,
                    allowed_source_kinds: vec![
                        ContextSourceKind::OntologyEntity,
                        ContextSourceKind::KnowledgeGraphSubgraph,
                    ],
                    max_context_items: 2,
                    freshness_floor_epoch_seconds: 100,
                    candidates: vec![
                        candidate(
                            "ten_a",
                            ContextSourceKind::OntologyEntity,
                            "entityref://org/2",
                            "ctx:entity:2",
                            ContextDataClass::InternalOnly,
                            125,
                            920,
                        ),
                        candidate(
                            "ten_a",
                            ContextSourceKind::KnowledgeGraphSubgraph,
                            "kgref://subgraph/1",
                            "ctx:kg:1",
                            ContextDataClass::InternalOnly,
                            130,
                            930,
                        ),
                    ],
                },
                policy_decision: ContextRetrievalPolicyDecision {
                    decision_id: "policy-ctx-1".to_owned(),
                    tenant_id: "ten_a".to_owned(),
                    principal_id: "principal-ctx-1".to_owned(),
                    allowed_surfaces: vec!["intelligence.context-aware-retrieval.pre".to_owned()],
                    allowed_source_kinds: vec![
                        ContextSourceKind::OntologyEntity,
                        ContextSourceKind::KnowledgeGraphSubgraph,
                    ],
                    max_context_items: 2,
                    freshness_floor_epoch_seconds: 100,
                    evidence_ref: "cedar:ctx-usecase:allow".to_owned(),
                    retrieval_index_snapshot_ref: "retrieval-index:snapshot:1".to_owned(),
                },
            },
        }
    }

    #[test]
    fn plans_authorized_context_with_metadata_audit() {
        let mut usecase = ContextAwareRetrievalUsecase::default();

        let receipt = usecase.plan(sample_input("idem-ctx-1"));

        assert_eq!(receipt.status, ContextRetrievalUsecaseStatus::Planned);
        assert_eq!(receipt.denial_kind, None);
        assert_eq!(
            receipt.plan_resource_refs,
            vec![
                "kgref://subgraph/1".to_owned(),
                "entityref://org/2".to_owned()
            ]
        );
        assert!(
            receipt
                .evidence_refs
                .contains(&"cedar:ctx-usecase:allow".to_owned())
        );
        assert!(
            receipt
                .evidence_refs
                .contains(&"retrieval-index:snapshot:1".to_owned())
        );
        assert_eq!(usecase.audit_events().len(), 2);
        assert_eq!(
            usecase.audit_events()[0].kind,
            ContextRetrievalAuditEventKind::RetrievalRequested
        );
        assert_eq!(
            usecase.audit_events()[1].kind,
            ContextRetrievalAuditEventKind::RetrievalPlanned
        );
        let debug = format!("{receipt:?}{:?}", usecase.audit_events());
        assert!(!debug.contains("raw query"));
        assert!(!debug.contains("customer message"));
        assert!(!debug.contains("sk-"));
    }

    #[test]
    fn invalid_raw_query_and_identity_deny_before_audit_side_effect() {
        let mut usecase = ContextAwareRetrievalUsecase::default();
        let mut input = sample_input("sk-test-idem");
        input.request.principal_id = "Bearer principal token".to_owned();
        input.request.request.tenant_id = "sk-test-tenant".to_owned();
        input.request.request.query_ref = "raw query: write an email to customer".to_owned();

        let receipt = usecase.plan(input);
        let debug = format!("{receipt:?}{:?}", usecase.audit_events());

        assert_eq!(receipt.status, ContextRetrievalUsecaseStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(ContextRetrievalUsecaseDenialKind::InvalidInput)
        );
        assert_eq!(receipt.idempotency_key, "redacted-invalid-idempotency-key");
        assert_eq!(receipt.tenant_id, "redacted-invalid-tenant-id");
        assert_eq!(receipt.principal_id, "redacted-invalid-principal-id");
        assert_eq!(receipt.query_ref, "redacted-invalid-query-ref");
        assert!(usecase.audit_events().is_empty());
        assert!(!debug.contains("write an email"));
        assert!(!debug.contains("sk-test"));
        assert!(!debug.contains("Bearer"));
    }

    #[test]
    fn idempotent_replay_and_conflict_are_deterministic() {
        let mut usecase = ContextAwareRetrievalUsecase::default();

        let first = usecase.plan(sample_input("idem-ctx-replay"));
        let replay = usecase.plan(sample_input("idem-ctx-replay"));
        let mut changed = sample_input("idem-ctx-replay");
        changed.request.request.max_context_items = 1;
        let conflict = usecase.plan(changed);

        assert_eq!(first, replay);
        assert_eq!(conflict.status, ContextRetrievalUsecaseStatus::Denied);
        assert_eq!(
            conflict.denial_kind,
            Some(ContextRetrievalUsecaseDenialKind::IdempotencyConflict)
        );
        assert_eq!(usecase.receipt_count(), 1);
        assert!(
            usecase
                .audit_events()
                .iter()
                .any(|event| event.kind == ContextRetrievalAuditEventKind::IdempotencyConflict)
        );
    }

    #[test]
    fn idempotent_replay_treats_candidates_as_order_independent_set() {
        let mut usecase = ContextAwareRetrievalUsecase::default();
        let first = sample_input("idem-ctx-order");
        let mut reordered = sample_input("idem-ctx-order");
        reordered.request.request.candidates.reverse();

        let first_receipt = usecase.plan(first);
        let replay_receipt = usecase.plan(reordered);

        assert_eq!(first_receipt, replay_receipt);
        assert_eq!(usecase.receipt_count(), 1);
    }

    #[test]
    fn domain_policy_denial_records_fail_closed_audit() {
        let mut usecase = ContextAwareRetrievalUsecase::default();
        let mut input = sample_input("idem-ctx-domain-deny");
        input.request.policy_decision.tenant_id = "ten_other".to_owned();

        let receipt = usecase.plan(input);

        assert_eq!(receipt.status, ContextRetrievalUsecaseStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(ContextRetrievalUsecaseDenialKind::DomainDenied)
        );
        assert!(
            receipt
                .denial_reasons
                .contains(&"domain::PolicyDrift".to_owned())
        );
        assert!(receipt.plan_resource_refs.is_empty());
        assert_eq!(usecase.audit_events().len(), 2);
        assert_eq!(
            usecase.audit_events()[1].kind,
            ContextRetrievalAuditEventKind::RetrievalDenied
        );
    }

    #[test]
    fn external_sensitive_context_denies_without_resource_refs() {
        let mut usecase = ContextAwareRetrievalUsecase::default();
        let mut input = sample_input("idem-ctx-external");
        input.request.request.audience = ContextAudience::ExternalEndUser;
        input.request.request.allowed_source_kinds = vec![ContextSourceKind::OntologyEntity];
        input.request.policy_decision.allowed_source_kinds =
            vec![ContextSourceKind::OntologyEntity];
        input.request.request.candidates = vec![candidate(
            "ten_a",
            ContextSourceKind::OntologyEntity,
            "entityref://patient/1",
            "ctx:phi:1",
            ContextDataClass::Phi,
            200,
            950,
        )];

        let receipt = usecase.plan(input);

        assert_eq!(receipt.status, ContextRetrievalUsecaseStatus::Denied);
        assert_eq!(receipt.plan_resource_refs, Vec::<String>::new());
        assert!(
            receipt
                .evidence_refs
                .contains(&"cedar:ctx-usecase:allow".to_owned())
        );
    }
}
