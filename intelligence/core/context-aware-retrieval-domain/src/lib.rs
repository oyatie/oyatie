//! Intelligence context-aware retrieval domain foundation.
//!
//! The domain layer validates policy-decision metadata and request authority
//! before delegating to the deterministic retrieval kernel. It remains a
//! source-level preview seam: no vector-store calls, ontology/KG execution,
//! embedding generation, document fetch, filesystem, network, or policy-engine
//! runtime is performed here.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use intelligence_context_aware_retrieval_kernel::{
    ContextAudience, ContextCandidate, ContextDataClass, ContextRetrievalDecision,
    ContextRetrievalPlan, ContextRetrievalRequest, ContextRetrievalStatus, ContextSourceKind,
    decide_context_retrieval,
};

const DOMAIN_MAX_CONTEXT_ITEMS: usize = 32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextRetrievalPolicyDecision {
    pub decision_id: String,                          // data_class: INTERNAL_ONLY
    pub tenant_id: String,                            // data_class: INTERNAL_ONLY
    pub principal_id: String,                         // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>,                // data_class: INTERNAL_ONLY
    pub allowed_source_kinds: Vec<ContextSourceKind>, // data_class: INTERNAL_ONLY
    pub max_context_items: usize,                     // data_class: INTERNAL_ONLY
    pub freshness_floor_epoch_seconds: u64,           // data_class: INTERNAL_ONLY
    pub evidence_ref: String,                         // data_class: INTERNAL_ONLY
    pub retrieval_index_snapshot_ref: String,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainContextRetrievalRequest {
    pub principal_id: String,             // data_class: INTERNAL_ONLY
    pub query_surface: String,            // data_class: INTERNAL_ONLY
    pub request: ContextRetrievalRequest, // data_class: INTERNAL_ONLY
    pub policy_decision: ContextRetrievalPolicyDecision, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextRetrievalDomainStatus {
    Planned,
    Denied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ContextRetrievalDomainDenialKind {
    FreshnessFloorDenied,
    InvalidInput,
    KernelDenied,
    LimitExceeded,
    PolicyDrift,
    SourceKindDenied,
    SurfaceDenied,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextRetrievalDomainDenial {
    pub tenant_id: String,                             // data_class: INTERNAL_ONLY
    pub principal_id: String,                          // data_class: INTERNAL_ONLY
    pub query_surface: String,                         // data_class: INTERNAL_ONLY
    pub status: ContextRetrievalDomainStatus,          // data_class: PUBLIC
    pub denial_kind: ContextRetrievalDomainDenialKind, // data_class: INTERNAL_ONLY
    pub reasons: Vec<String>,                          // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextRetrievalDomainDecision {
    Plan(ContextRetrievalPlan),
    Deny(ContextRetrievalDomainDenial),
}

impl ContextRetrievalDomainDecision {
    pub fn status(&self) -> ContextRetrievalDomainStatus {
        match self {
            Self::Plan(_) => ContextRetrievalDomainStatus::Planned,
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

pub fn plan_domain_context_retrieval(
    input: DomainContextRetrievalRequest,
) -> ContextRetrievalDomainDecision {
    let invalid = invalid_input_reasons(&input);
    if !invalid.is_empty() {
        return ContextRetrievalDomainDecision::Deny(denial_from_parts(
            safe_tenant(&input.request.tenant_id),
            safe_metadata(&input.principal_id, "redacted-invalid-principal-id"),
            safe_metadata(&input.query_surface, "redacted-invalid-query-surface"),
            ContextRetrievalDomainDenialKind::InvalidInput,
            invalid,
            vec!["validation:intelligence-context-aware-retrieval-domain-input".to_owned()],
        ));
    }

    if input.policy_decision.tenant_id != input.request.tenant_id
        || input.policy_decision.principal_id != input.principal_id
        || input.request.policy_decision_ref != input.policy_decision.evidence_ref
    {
        return domain_denial(
            &input,
            ContextRetrievalDomainDenialKind::PolicyDrift,
            vec![
                "context retrieval policy decision is not bound to request tenant/principal/evidence"
                    .to_owned(),
            ],
            vec![
                input.request.request_evidence_ref.clone(),
                input.request.policy_decision_ref.clone(),
                input.policy_decision.evidence_ref.clone(),
                "validation:intelligence-context-aware-retrieval-policy-drift".to_owned(),
            ],
        );
    }

    if !input
        .policy_decision
        .allowed_surfaces
        .iter()
        .any(|surface| surface == &input.query_surface)
    {
        return domain_denial(
            &input,
            ContextRetrievalDomainDenialKind::SurfaceDenied,
            vec!["context retrieval policy decision does not allow this surface".to_owned()],
            policy_and_request_evidence_refs(&input),
        );
    }

    if !request_sources_are_policy_allowed(&input) {
        return domain_denial(
            &input,
            ContextRetrievalDomainDenialKind::SourceKindDenied,
            vec!["context retrieval source kinds exceed policy decision".to_owned()],
            policy_and_request_evidence_refs(&input),
        );
    }

    if input.request.max_context_items > input.policy_decision.max_context_items {
        return domain_denial(
            &input,
            ContextRetrievalDomainDenialKind::LimitExceeded,
            vec!["requested context item cap exceeds policy decision".to_owned()],
            policy_and_request_evidence_refs(&input),
        );
    }

    if input.request.freshness_floor_epoch_seconds
        < input.policy_decision.freshness_floor_epoch_seconds
    {
        return domain_denial(
            &input,
            ContextRetrievalDomainDenialKind::FreshnessFloorDenied,
            vec!["requested freshness floor is weaker than policy decision".to_owned()],
            policy_and_request_evidence_refs(&input),
        );
    }

    match decide_context_retrieval(input.request.clone()) {
        ContextRetrievalDecision::Plan(mut plan) => {
            let mut evidence_refs = plan.evidence_refs.clone();
            evidence_refs.extend(policy_evidence_refs(&input));
            plan.evidence_refs = sorted_unique(evidence_refs);
            ContextRetrievalDomainDecision::Plan(plan)
        }
        ContextRetrievalDecision::Deny(denial) => {
            let reasons = denial
                .reasons
                .iter()
                .map(|reason| format!("kernel::{reason:?}"))
                .collect::<Vec<_>>();
            let evidence_refs =
                sorted_unique([denial.evidence_refs, policy_evidence_refs(&input)].concat());
            domain_denial(
                &input,
                ContextRetrievalDomainDenialKind::KernelDenied,
                reasons,
                evidence_refs,
            )
        }
    }
}

fn invalid_input_reasons(input: &DomainContextRetrievalRequest) -> Vec<String> {
    let mut reasons = Vec::new();
    require_metadata_ref("principal id", &input.principal_id, &mut reasons);
    require_metadata_ref("query surface", &input.query_surface, &mut reasons);
    require_metadata_ref(
        "policy decision id",
        &input.policy_decision.decision_id,
        &mut reasons,
    );
    require_tenant(
        "policy tenant id",
        &input.policy_decision.tenant_id,
        &mut reasons,
    );
    require_metadata_ref(
        "policy principal id",
        &input.policy_decision.principal_id,
        &mut reasons,
    );
    if input.policy_decision.allowed_surfaces.is_empty() {
        reasons.push("policy allowed surfaces are required".to_owned());
    }
    for surface in &input.policy_decision.allowed_surfaces {
        require_metadata_ref("policy allowed surface", surface, &mut reasons);
    }
    if input.policy_decision.allowed_source_kinds.is_empty() {
        reasons.push("policy allowed source kinds are required".to_owned());
    }
    if input.policy_decision.max_context_items == 0 {
        reasons.push("policy max context items must be greater than zero".to_owned());
    } else if input.policy_decision.max_context_items > DOMAIN_MAX_CONTEXT_ITEMS {
        reasons.push(format!(
            "policy max context items must be less than or equal to {DOMAIN_MAX_CONTEXT_ITEMS}"
        ));
    }
    require_evidence_ref(
        "policy evidence ref",
        &input.policy_decision.evidence_ref,
        &mut reasons,
    );
    require_evidence_ref(
        "retrieval index snapshot ref",
        &input.policy_decision.retrieval_index_snapshot_ref,
        &mut reasons,
    );

    require_tenant("request tenant id", &input.request.tenant_id, &mut reasons);
    require_resource_ref("query ref", &input.request.query_ref, &mut reasons);
    require_evidence_ref(
        "request evidence ref",
        &input.request.request_evidence_ref,
        &mut reasons,
    );
    require_evidence_ref(
        "trace context ref",
        &input.request.trace_context_ref,
        &mut reasons,
    );
    require_evidence_ref(
        "request policy decision ref",
        &input.request.policy_decision_ref,
        &mut reasons,
    );
    if input.request.max_context_items == 0
        || input.request.max_context_items > DOMAIN_MAX_CONTEXT_ITEMS
    {
        reasons.push(format!(
            "request max context items must be in 1..={DOMAIN_MAX_CONTEXT_ITEMS}"
        ));
    }
    sorted_unique(reasons)
}

fn request_sources_are_policy_allowed(input: &DomainContextRetrievalRequest) -> bool {
    input.request.allowed_source_kinds.iter().all(|kind| {
        input
            .policy_decision
            .allowed_source_kinds
            .iter()
            .any(|allowed| allowed == kind)
    })
}

fn policy_evidence_refs(input: &DomainContextRetrievalRequest) -> Vec<String> {
    vec![
        input.policy_decision.evidence_ref.clone(),
        input.policy_decision.retrieval_index_snapshot_ref.clone(),
    ]
}

fn policy_and_request_evidence_refs(input: &DomainContextRetrievalRequest) -> Vec<String> {
    sorted_unique(vec![
        input.request.request_evidence_ref.clone(),
        input.request.trace_context_ref.clone(),
        input.request.policy_decision_ref.clone(),
        input.policy_decision.evidence_ref.clone(),
        input.policy_decision.retrieval_index_snapshot_ref.clone(),
    ])
}

fn domain_denial(
    input: &DomainContextRetrievalRequest,
    denial_kind: ContextRetrievalDomainDenialKind,
    reasons: Vec<String>,
    evidence_refs: Vec<String>,
) -> ContextRetrievalDomainDecision {
    ContextRetrievalDomainDecision::Deny(denial_from_parts(
        input.request.tenant_id.clone(),
        input.principal_id.clone(),
        input.query_surface.clone(),
        denial_kind,
        reasons,
        evidence_refs,
    ))
}

fn denial_from_parts(
    tenant_id: String,
    principal_id: String,
    query_surface: String,
    denial_kind: ContextRetrievalDomainDenialKind,
    reasons: Vec<String>,
    evidence_refs: Vec<String>,
) -> ContextRetrievalDomainDenial {
    ContextRetrievalDomainDenial {
        tenant_id,
        principal_id,
        query_surface,
        status: ContextRetrievalDomainStatus::Denied,
        denial_kind,
        reasons: sorted_unique(reasons),
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

    fn sample_domain_request(request_evidence_ref: &str) -> DomainContextRetrievalRequest {
        DomainContextRetrievalRequest {
            principal_id: "principal-ctx-1".to_owned(),
            query_surface: "intelligence.context-aware-retrieval.pre".to_owned(),
            request: ContextRetrievalRequest {
                tenant_id: "ten_a".to_owned(),
                query_ref: "queryref://opaque/1".to_owned(),
                request_evidence_ref: request_evidence_ref.to_owned(),
                trace_context_ref: "trace:ctx-domain:1".to_owned(),
                policy_decision_ref: "cedar:ctx-domain:allow".to_owned(),
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
                evidence_ref: "cedar:ctx-domain:allow".to_owned(),
                retrieval_index_snapshot_ref: "retrieval-index:snapshot:1".to_owned(),
            },
        }
    }

    #[test]
    fn authorized_domain_request_delegates_to_kernel_plan() {
        let decision = plan_domain_context_retrieval(sample_domain_request("req:ctx-domain:1"));

        let ContextRetrievalDomainDecision::Plan(plan) = decision else {
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
        assert!(
            plan.evidence_refs
                .contains(&"cedar:ctx-domain:allow".to_owned())
        );
        assert!(
            plan.evidence_refs
                .contains(&"retrieval-index:snapshot:1".to_owned())
        );
    }

    #[test]
    fn policy_drift_and_surface_denial_block_before_kernel() {
        let mut drift = sample_domain_request("req:ctx-domain:drift");
        drift.policy_decision.tenant_id = "ten_other".to_owned();
        let mut evidence_drift = sample_domain_request("req:ctx-domain:evidence-drift");
        evidence_drift.request.policy_decision_ref = "cedar:ctx-domain:stale".to_owned();
        let mut surface = sample_domain_request("req:ctx-domain:surface");
        surface.query_surface = "intelligence.dispatch.pre".to_owned();

        let drift_decision = plan_domain_context_retrieval(drift);
        let evidence_drift_decision = plan_domain_context_retrieval(evidence_drift);
        let surface_decision = plan_domain_context_retrieval(surface);

        let ContextRetrievalDomainDecision::Deny(drift_denial) = drift_decision else {
            panic!("expected policy drift denial");
        };
        let ContextRetrievalDomainDecision::Deny(evidence_drift_denial) = evidence_drift_decision
        else {
            panic!("expected policy evidence drift denial");
        };
        let ContextRetrievalDomainDecision::Deny(surface_denial) = surface_decision else {
            panic!("expected surface denial");
        };
        assert_eq!(
            drift_denial.denial_kind,
            ContextRetrievalDomainDenialKind::PolicyDrift
        );
        assert_eq!(
            evidence_drift_denial.denial_kind,
            ContextRetrievalDomainDenialKind::PolicyDrift
        );
        assert_eq!(
            surface_denial.denial_kind,
            ContextRetrievalDomainDenialKind::SurfaceDenied
        );
    }

    #[test]
    fn source_kind_limit_and_freshness_policy_are_enforced() {
        let mut source = sample_domain_request("req:ctx-domain:source");
        source
            .request
            .allowed_source_kinds
            .push(ContextSourceKind::VectorMemory);
        let mut limit = sample_domain_request("req:ctx-domain:limit");
        limit.request.max_context_items = 3;
        limit.policy_decision.max_context_items = 2;
        let mut freshness = sample_domain_request("req:ctx-domain:freshness");
        freshness.request.freshness_floor_epoch_seconds = 100;
        freshness.policy_decision.freshness_floor_epoch_seconds = 200;

        let denials = [
            plan_domain_context_retrieval(source),
            plan_domain_context_retrieval(limit),
            plan_domain_context_retrieval(freshness),
        ];

        let kinds = denials
            .iter()
            .map(|decision| match decision {
                ContextRetrievalDomainDecision::Deny(denial) => denial.denial_kind,
                ContextRetrievalDomainDecision::Plan(_) => panic!("expected denial"),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            vec![
                ContextRetrievalDomainDenialKind::SourceKindDenied,
                ContextRetrievalDomainDenialKind::LimitExceeded,
                ContextRetrievalDomainDenialKind::FreshnessFloorDenied,
            ]
        );
    }

    #[test]
    fn invalid_raw_identity_and_query_are_redacted() {
        let mut invalid = sample_domain_request("req:ctx-domain:invalid");
        invalid.principal_id = "Bearer principal token".to_owned();
        invalid.request.tenant_id = "sk-test-tenant".to_owned();
        invalid.request.query_ref = "raw query: write an email to the customer".to_owned();

        let decision = plan_domain_context_retrieval(invalid);
        let debug = format!("{decision:?}");

        let ContextRetrievalDomainDecision::Deny(denial) = decision else {
            panic!("expected invalid denial");
        };
        assert_eq!(denial.tenant_id, "redacted-invalid-tenant-id");
        assert_eq!(denial.principal_id, "redacted-invalid-principal-id");
        assert_eq!(
            denial.denial_kind,
            ContextRetrievalDomainDenialKind::InvalidInput
        );
        assert!(!debug.contains("write an email"));
        assert!(!debug.contains("sk-test"));
        assert!(!debug.contains("Bearer"));
    }

    #[test]
    fn kernel_no_eligible_context_denial_is_preserved_with_policy_evidence() {
        let mut request = sample_domain_request("req:ctx-domain:none");
        request.request.allowed_source_kinds = vec![ContextSourceKind::WorkflowRun];
        request.policy_decision.allowed_source_kinds = vec![ContextSourceKind::WorkflowRun];

        let decision = plan_domain_context_retrieval(request);

        let ContextRetrievalDomainDecision::Deny(denial) = decision else {
            panic!("expected kernel denial");
        };
        assert_eq!(
            denial.denial_kind,
            ContextRetrievalDomainDenialKind::KernelDenied
        );
        assert!(
            denial
                .reasons
                .contains(&"kernel::NoEligibleContext".to_owned())
        );
        assert!(
            denial
                .evidence_refs
                .contains(&"cedar:ctx-domain:allow".to_owned())
        );
        assert!(
            denial
                .evidence_refs
                .contains(&"retrieval-index:snapshot:1".to_owned())
        );
    }

    #[test]
    fn external_audience_sensitive_context_denies_through_kernel() {
        let mut request = sample_domain_request("req:ctx-domain:external");
        request.request.audience = ContextAudience::ExternalEndUser;
        request.request.allowed_source_kinds = vec![ContextSourceKind::OntologyEntity];
        request.policy_decision.allowed_source_kinds = vec![ContextSourceKind::OntologyEntity];
        request.request.candidates = vec![candidate(
            "ten_a",
            ContextSourceKind::OntologyEntity,
            "entityref://patient/1",
            "ctx:phi:1",
            ContextDataClass::Phi,
            300,
            950,
        )];

        let decision = plan_domain_context_retrieval(request);

        let ContextRetrievalDomainDecision::Deny(denial) = decision else {
            panic!("expected external sensitive denial");
        };
        assert_eq!(
            denial.denial_kind,
            ContextRetrievalDomainDenialKind::KernelDenied
        );
        assert!(
            denial
                .reasons
                .contains(&"kernel::SensitiveExternalContext".to_owned())
        );
    }
}
