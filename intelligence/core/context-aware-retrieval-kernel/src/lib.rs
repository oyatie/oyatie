//! Intelligence context-aware retrieval kernel foundation.
//!
//! This crate produces deterministic metadata-only retrieval plans for later
//! Intelligence retrieval usecases. It does not fetch documents, call vector
//! stores, execute ontology queries, inspect raw prompts, or perform network / IO.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::cmp::Reverse;

const MAX_CONTEXT_ITEMS: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ContextRetrievalStatus {
    Planned,
    Denied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ContextRetrievalDenialReason {
    InvalidInput,
    NoEligibleContext,
    SensitiveExternalContext,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ContextSourceKind {
    AuditEvent,
    DocumentSnippet,
    KnowledgeGraphSubgraph,
    OntologyEntity,
    VectorMemory,
    WorkflowRun,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ContextDataClass {
    BehavioralTenantProduct,
    InternalOnly,
    Phi,
    PiiIdentifying,
    Public,
    SearchQuery,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ContextAudience {
    ExternalEndUser,
    InternalAutomation,
    TenantOperator,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextCandidate {
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub source_kind: ContextSourceKind, // data_class: INTERNAL_ONLY
    pub resource_ref: String,           // data_class: INTERNAL_ONLY
    pub evidence_ref: String,           // data_class: INTERNAL_ONLY
    pub data_class: ContextDataClass,   // data_class: INTERNAL_ONLY
    pub observed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub relevance_millis: u16,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextRetrievalRequest {
    pub tenant_id: String,                            // data_class: INTERNAL_ONLY
    pub query_ref: String,                            // data_class: INTERNAL_ONLY
    pub request_evidence_ref: String,                 // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,                    // data_class: INTERNAL_ONLY
    pub policy_decision_ref: String,                  // data_class: INTERNAL_ONLY
    pub audience: ContextAudience,                    // data_class: INTERNAL_ONLY
    pub allowed_source_kinds: Vec<ContextSourceKind>, // data_class: INTERNAL_ONLY
    pub max_context_items: usize,                     // data_class: INTERNAL_ONLY
    pub freshness_floor_epoch_seconds: u64,           // data_class: INTERNAL_ONLY
    pub candidates: Vec<ContextCandidate>,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextRetrievalItem {
    pub source_kind: ContextSourceKind, // data_class: INTERNAL_ONLY
    pub resource_ref: String,           // data_class: INTERNAL_ONLY
    pub evidence_ref: String,           // data_class: INTERNAL_ONLY
    pub data_class: ContextDataClass,   // data_class: INTERNAL_ONLY
    pub observed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub relevance_millis: u16,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextRetrievalPlan {
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub query_ref: String,                // data_class: INTERNAL_ONLY
    pub status: ContextRetrievalStatus,   // data_class: PUBLIC
    pub items: Vec<ContextRetrievalItem>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextRetrievalDenial {
    pub tenant_id: String,                          // data_class: INTERNAL_ONLY
    pub query_ref: String,                          // data_class: INTERNAL_ONLY
    pub status: ContextRetrievalStatus,             // data_class: PUBLIC
    pub reasons: Vec<ContextRetrievalDenialReason>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                 // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextRetrievalDecision {
    Plan(ContextRetrievalPlan),
    Deny(ContextRetrievalDenial),
}

impl ContextRetrievalDecision {
    pub fn status(&self) -> ContextRetrievalStatus {
        match self {
            Self::Plan(plan) => plan.status,
            Self::Deny(denial) => denial.status,
        }
    }

    pub fn evidence_refs(&self) -> &[String] {
        match self {
            Self::Plan(plan) => &plan.evidence_refs,
            Self::Deny(denial) => &denial.evidence_refs,
        }
    }
}

pub fn decide_context_retrieval(request: ContextRetrievalRequest) -> ContextRetrievalDecision {
    let validation_reasons = invalid_input_reasons(&request);
    if !validation_reasons.is_empty() {
        return invalid_input_denial(&request);
    }

    let mut sensitive_external_filtered = false;
    let mut eligible: Vec<ContextCandidate> = request
        .candidates
        .iter()
        .filter(|candidate| {
            if candidate.tenant_id != request.tenant_id {
                return false;
            }
            if !request
                .allowed_source_kinds
                .iter()
                .any(|kind| kind == &candidate.source_kind)
            {
                return false;
            }
            if candidate.observed_at_epoch_seconds < request.freshness_floor_epoch_seconds {
                return false;
            }
            if !data_class_allowed_for_audience(candidate.data_class, request.audience) {
                sensitive_external_filtered = true;
                return false;
            }
            true
        })
        .cloned()
        .collect();

    if eligible.is_empty() {
        let mut reasons = vec![ContextRetrievalDenialReason::NoEligibleContext];
        if sensitive_external_filtered {
            reasons.push(ContextRetrievalDenialReason::SensitiveExternalContext);
        }
        return ContextRetrievalDecision::Deny(ContextRetrievalDenial {
            tenant_id: request.tenant_id,
            query_ref: request.query_ref,
            status: ContextRetrievalStatus::Denied,
            reasons: sorted_unique_reasons(reasons),
            evidence_refs: sorted_unique(vec![
                request.request_evidence_ref,
                request.trace_context_ref,
                request.policy_decision_ref,
                "validation:intelligence-context-aware-retrieval-no-eligible-context".to_owned(),
            ]),
        });
    }

    eligible.sort_by_key(|candidate| {
        (
            Reverse(candidate.relevance_millis),
            Reverse(candidate.observed_at_epoch_seconds),
            candidate.source_kind,
            candidate.resource_ref.clone(),
        )
    });
    eligible.truncate(request.max_context_items);

    let items: Vec<ContextRetrievalItem> = eligible
        .into_iter()
        .map(|candidate| ContextRetrievalItem {
            source_kind: candidate.source_kind,
            resource_ref: candidate.resource_ref,
            evidence_ref: candidate.evidence_ref,
            data_class: candidate.data_class,
            observed_at_epoch_seconds: candidate.observed_at_epoch_seconds,
            relevance_millis: candidate.relevance_millis,
        })
        .collect();

    let mut evidence_refs = vec![
        request.request_evidence_ref.clone(),
        request.trace_context_ref.clone(),
        request.policy_decision_ref.clone(),
    ];
    evidence_refs.extend(items.iter().map(|item| item.evidence_ref.clone()));

    ContextRetrievalDecision::Plan(ContextRetrievalPlan {
        tenant_id: request.tenant_id,
        query_ref: request.query_ref,
        status: ContextRetrievalStatus::Planned,
        items,
        evidence_refs: sorted_unique(evidence_refs),
    })
}

fn invalid_input_reasons(request: &ContextRetrievalRequest) -> Vec<String> {
    let mut reasons = Vec::new();
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
        "policy decision ref",
        &request.policy_decision_ref,
        &mut reasons,
    );
    if request.allowed_source_kinds.is_empty() {
        reasons.push("allowed source kinds are required".to_owned());
    }
    if request.max_context_items == 0 {
        reasons.push("max context items must be greater than zero".to_owned());
    } else if request.max_context_items > MAX_CONTEXT_ITEMS {
        reasons.push(format!(
            "max context items must be less than or equal to {MAX_CONTEXT_ITEMS}"
        ));
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
    sorted_unique(reasons)
}

fn invalid_input_denial(request: &ContextRetrievalRequest) -> ContextRetrievalDecision {
    ContextRetrievalDecision::Deny(ContextRetrievalDenial {
        tenant_id: safe_tenant(&request.tenant_id),
        query_ref: safe_ref(&request.query_ref, "redacted-invalid-query-ref"),
        status: ContextRetrievalStatus::Denied,
        reasons: vec![ContextRetrievalDenialReason::InvalidInput],
        evidence_refs: vec!["validation:intelligence-context-aware-retrieval-input".to_owned()],
    })
}

fn data_class_allowed_for_audience(
    data_class: ContextDataClass,
    audience: ContextAudience,
) -> bool {
    if audience != ContextAudience::ExternalEndUser {
        return true;
    }
    data_class == ContextDataClass::Public
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

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn sorted_unique_reasons(
    mut values: Vec<ContextRetrievalDenialReason>,
) -> Vec<ContextRetrievalDenialReason> {
    values.sort();
    values.dedup();
    values
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

    fn sample_request(request_evidence_ref: &str) -> ContextRetrievalRequest {
        ContextRetrievalRequest {
            tenant_id: "ten_a".to_owned(),
            query_ref: "queryref://opaque/1".to_owned(),
            request_evidence_ref: request_evidence_ref.to_owned(),
            trace_context_ref: "trace:ctx:1".to_owned(),
            policy_decision_ref: "cedar:ctx:allow".to_owned(),
            audience: ContextAudience::TenantOperator,
            allowed_source_kinds: vec![
                ContextSourceKind::OntologyEntity,
                ContextSourceKind::KnowledgeGraphSubgraph,
                ContextSourceKind::DocumentSnippet,
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
                    920,
                ),
                candidate(
                    "ten_a",
                    ContextSourceKind::DocumentSnippet,
                    "docref://snippet/1",
                    "ctx:doc:1",
                    ContextDataClass::Public,
                    140,
                    850,
                ),
            ],
        }
    }

    #[test]
    fn selects_deterministic_tenant_scoped_context_plan() {
        let decision = decide_context_retrieval(sample_request("req:ctx:1"));

        let ContextRetrievalDecision::Plan(plan) = decision else {
            panic!("expected planned context retrieval");
        };
        assert_eq!(plan.status, ContextRetrievalStatus::Planned);
        assert_eq!(plan.items.len(), 2);
        assert_eq!(
            plan.items
                .iter()
                .map(|item| item.resource_ref.as_str())
                .collect::<Vec<_>>(),
            vec!["kgref://subgraph/1", "entityref://org/2"]
        );
        assert_eq!(
            plan.evidence_refs,
            vec![
                "cedar:ctx:allow".to_owned(),
                "ctx:entity:2".to_owned(),
                "ctx:kg:1".to_owned(),
                "req:ctx:1".to_owned(),
                "trace:ctx:1".to_owned(),
            ]
        );
        let debug = format!("{plan:?}");
        assert!(!debug.contains("raw query"));
        assert!(!debug.contains("customer message"));
    }

    #[test]
    fn filters_cross_tenant_stale_and_unsupported_sources_before_capping() {
        let mut request = sample_request("req:ctx:filter");
        request.max_context_items = 3;
        request.candidates.push(candidate(
            "ten_other",
            ContextSourceKind::OntologyEntity,
            "entityref://other/1",
            "ctx:other:1",
            ContextDataClass::InternalOnly,
            999,
            1000,
        ));
        request.candidates.push(candidate(
            "ten_a",
            ContextSourceKind::VectorMemory,
            "vectorref://memory/1",
            "ctx:vector:1",
            ContextDataClass::InternalOnly,
            999,
            1000,
        ));
        request.candidates.push(candidate(
            "ten_a",
            ContextSourceKind::DocumentSnippet,
            "docref://old/1",
            "ctx:old:1",
            ContextDataClass::InternalOnly,
            99,
            1000,
        ));

        let decision = decide_context_retrieval(request);

        let ContextRetrievalDecision::Plan(plan) = decision else {
            panic!("expected filtered plan");
        };
        assert_eq!(plan.items.len(), 3);
        let refs = plan
            .items
            .iter()
            .map(|item| item.resource_ref.as_str())
            .collect::<Vec<_>>();
        assert!(!refs.contains(&"entityref://other/1"));
        assert!(!refs.contains(&"vectorref://memory/1"));
        assert!(!refs.contains(&"docref://old/1"));
    }

    #[test]
    fn external_audience_denies_when_only_sensitive_context_is_available() {
        let mut request = sample_request("req:ctx:sensitive");
        request.audience = ContextAudience::ExternalEndUser;
        request.candidates = vec![candidate(
            "ten_a",
            ContextSourceKind::OntologyEntity,
            "entityref://patient/1",
            "ctx:phi:1",
            ContextDataClass::Phi,
            200,
            950,
        )];

        let decision = decide_context_retrieval(request);

        let ContextRetrievalDecision::Deny(denial) = decision else {
            panic!("expected sensitive external denial");
        };
        assert_eq!(denial.status, ContextRetrievalStatus::Denied);
        assert_eq!(
            denial.reasons,
            vec![
                ContextRetrievalDenialReason::NoEligibleContext,
                ContextRetrievalDenialReason::SensitiveExternalContext,
            ]
        );
    }

    #[test]
    fn external_audience_plans_only_public_context() {
        let mut request = sample_request("req:ctx:public");
        request.audience = ContextAudience::ExternalEndUser;
        request.max_context_items = 3;
        request.candidates = vec![
            candidate(
                "ten_a",
                ContextSourceKind::DocumentSnippet,
                "docref://public/1",
                "ctx:public:1",
                ContextDataClass::Public,
                200,
                800,
            ),
            candidate(
                "ten_a",
                ContextSourceKind::OntologyEntity,
                "entityref://internal/1",
                "ctx:internal:1",
                ContextDataClass::InternalOnly,
                201,
                999,
            ),
        ];

        let decision = decide_context_retrieval(request);

        let ContextRetrievalDecision::Plan(plan) = decision else {
            panic!("expected public-only external plan");
        };
        assert_eq!(plan.items.len(), 1);
        assert_eq!(plan.items[0].resource_ref, "docref://public/1");
        assert_eq!(plan.items[0].data_class, ContextDataClass::Public);
    }

    #[test]
    fn invalid_raw_query_material_denies_without_echoing_content() {
        let mut request = sample_request("req:ctx:raw-query");
        request.query_ref = "raw query: write an email to the customer".to_owned();
        request.tenant_id = "sk-test-tenant".to_owned();

        let decision = decide_context_retrieval(request);
        let debug = format!("{decision:?}");

        let ContextRetrievalDecision::Deny(denial) = decision else {
            panic!("expected invalid input denial");
        };
        assert_eq!(denial.tenant_id, "redacted-invalid-tenant-id");
        assert_eq!(denial.query_ref, "redacted-invalid-query-ref");
        assert_eq!(
            denial.reasons,
            vec![ContextRetrievalDenialReason::InvalidInput]
        );
        assert!(!debug.contains("write an email"));
        assert!(!debug.contains("sk-test"));
    }

    #[test]
    fn invalid_candidate_refs_deny_closed_before_selection() {
        let mut request = sample_request("req:ctx:bad-candidate");
        request.candidates[0].evidence_ref = "Bearer token".to_owned();
        request.candidates[1].relevance_millis = 1001;

        let decision = decide_context_retrieval(request);

        assert_eq!(decision.status(), ContextRetrievalStatus::Denied);
        assert_eq!(
            decision.evidence_refs(),
            &["validation:intelligence-context-aware-retrieval-input".to_owned()]
        );
    }

    #[test]
    fn unbounded_max_context_items_denies_closed() {
        let mut request = sample_request("req:ctx:too-many");
        request.max_context_items = MAX_CONTEXT_ITEMS + 1;

        let decision = decide_context_retrieval(request);

        assert_eq!(decision.status(), ContextRetrievalStatus::Denied);
        let ContextRetrievalDecision::Deny(denial) = decision else {
            panic!("expected invalid max context items denial");
        };
        assert_eq!(
            denial.reasons,
            vec![ContextRetrievalDenialReason::InvalidInput]
        );
    }

    #[test]
    fn no_eligible_context_denies_with_policy_and_trace_evidence() {
        let mut request = sample_request("req:ctx:none");
        request.allowed_source_kinds = vec![ContextSourceKind::WorkflowRun];

        let decision = decide_context_retrieval(request);

        let ContextRetrievalDecision::Deny(denial) = decision else {
            panic!("expected no eligible context denial");
        };
        assert_eq!(denial.status, ContextRetrievalStatus::Denied);
        assert_eq!(
            denial.reasons,
            vec![ContextRetrievalDenialReason::NoEligibleContext]
        );
        assert_eq!(
            denial.evidence_refs,
            vec![
                "cedar:ctx:allow".to_owned(),
                "req:ctx:none".to_owned(),
                "trace:ctx:1".to_owned(),
                "validation:intelligence-context-aware-retrieval-no-eligible-context".to_owned(),
            ]
        );
    }
}
