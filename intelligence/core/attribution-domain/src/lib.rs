//! Intelligence attribution domain foundation.
//!
//! The domain layer binds metadata-only citation planning to tenant/principal
//! policy authority. It validates attribution surfaces, allowed audiences,
//! source kinds, data classes, citation caps, and confidence floors before
//! delegating to the attribution kernel. It performs no retrieval, citation text
//! rendering, model calls, network I/O, filesystem access, durable storage,
//! durable audit-chain emission, or policy-engine runtime execution.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use intelligence_attribution_kernel::{
    AttributionAudience, AttributionCitation, AttributionClaim, AttributionDataClass,
    AttributionDenialKind, AttributionReport, AttributionRequest, AttributionSource,
    AttributionSourceKind, AttributionStatus, plan_attribution,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributionPolicyDecision {
    pub decision_id: String,                         // data_class: INTERNAL_ONLY
    pub tenant_id: String,                           // data_class: INTERNAL_ONLY
    pub principal_id: String,                        // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>,               // data_class: INTERNAL_ONLY
    pub allowed_audiences: Vec<AttributionAudience>, // data_class: INTERNAL_ONLY
    pub allowed_source_kinds: Vec<AttributionSourceKind>, // data_class: INTERNAL_ONLY
    pub allowed_data_classes: Vec<AttributionDataClass>, // data_class: INTERNAL_ONLY
    pub max_citations: usize,                        // data_class: INTERNAL_ONLY
    pub min_confidence_bps: u32,                     // data_class: INTERNAL_ONLY
    pub evidence_ref: String,                        // data_class: INTERNAL_ONLY
    pub attribution_registry_snapshot_ref: String,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainAttributionRequest {
    pub tenant_id: String,                          // data_class: INTERNAL_ONLY
    pub principal_id: String,                       // data_class: INTERNAL_ONLY
    pub attribution_surface: String,                // data_class: INTERNAL_ONLY
    pub request_evidence_ref: String,               // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,                  // data_class: INTERNAL_ONLY
    pub policy_decision_ref: String,                // data_class: INTERNAL_ONLY
    pub policy_decision: AttributionPolicyDecision, // data_class: INTERNAL_ONLY
    pub request: AttributionRequest,                // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttributionDomainStatus {
    Denied,
    Rendered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AttributionDomainDenialKind {
    AudienceDenied,
    CitationLimitExceeded,
    ConfidenceBelowPolicy,
    DataClassDenied,
    InvalidInput,
    KernelDenied,
    PolicyDrift,
    SourceKindDenied,
    SurfaceDenied,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributionDomainReport {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub principal_id: String,            // data_class: INTERNAL_ONLY
    pub attribution_surface: String,     // data_class: INTERNAL_ONLY
    pub status: AttributionDomainStatus, // data_class: PUBLIC
    pub report: AttributionReport,       // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributionDomainDenial {
    pub tenant_id: String,                        // data_class: INTERNAL_ONLY
    pub principal_id: String,                     // data_class: INTERNAL_ONLY
    pub attribution_surface: String,              // data_class: INTERNAL_ONLY
    pub output_ref: String,                       // data_class: INTERNAL_ONLY
    pub status: AttributionDomainStatus,          // data_class: PUBLIC
    pub denial_kind: AttributionDomainDenialKind, // data_class: INTERNAL_ONLY
    pub kernel_denial_kind: Option<AttributionDenialKind>, // data_class: INTERNAL_ONLY
    pub reasons: Vec<String>,                     // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttributionDomainDecision {
    Report(AttributionDomainReport),
    Deny(AttributionDomainDenial),
}

pub fn plan_domain_attribution(input: DomainAttributionRequest) -> AttributionDomainDecision {
    let invalid = invalid_input_reasons(&input);
    if !invalid.is_empty() {
        return AttributionDomainDecision::Deny(denial_from_parts(DenialParts {
            tenant_id: safe_ref(&input.tenant_id, "redacted-invalid-tenant-id"),
            principal_id: safe_ref(&input.principal_id, "redacted-invalid-principal-id"),
            attribution_surface: safe_ref(
                &input.attribution_surface,
                "redacted-invalid-attribution-surface",
            ),
            output_ref: safe_ref(&input.request.output_ref, "redacted-invalid-output-ref"),
            denial_kind: AttributionDomainDenialKind::InvalidInput,
            kernel_denial_kind: None,
            reasons: invalid,
            evidence_refs: vec!["validation:intelligence-attribution-domain-input".to_owned()],
        }));
    }

    if input.policy_decision.tenant_id != input.tenant_id
        || input.policy_decision.principal_id != input.principal_id
        || input.policy_decision_ref != input.policy_decision.evidence_ref
        || input.request.tenant_id != input.tenant_id
        || input.request.trace_context_ref != input.trace_context_ref
    {
        return domain_denial(
            &input,
            AttributionDomainDenialKind::PolicyDrift,
            None,
            vec![
                "attribution policy decision is not bound to request tenant/principal/evidence"
                    .to_owned(),
            ],
            vec![
                input.request_evidence_ref.clone(),
                input.policy_decision_ref.clone(),
                input.policy_decision.evidence_ref.clone(),
                "validation:intelligence-attribution-policy-drift".to_owned(),
            ],
        );
    }

    if !input
        .policy_decision
        .allowed_surfaces
        .iter()
        .any(|surface| surface == &input.attribution_surface)
    {
        return domain_denial(
            &input,
            AttributionDomainDenialKind::SurfaceDenied,
            None,
            vec!["attribution policy decision does not allow this surface".to_owned()],
            policy_evidence_refs(&input),
        );
    }

    if !input
        .policy_decision
        .allowed_audiences
        .contains(&input.request.audience)
    {
        return domain_denial(
            &input,
            AttributionDomainDenialKind::AudienceDenied,
            None,
            vec!["attribution policy decision does not allow this audience".to_owned()],
            policy_evidence_refs(&input),
        );
    }

    if input.request.max_citations > input.policy_decision.max_citations {
        return domain_denial(
            &input,
            AttributionDomainDenialKind::CitationLimitExceeded,
            None,
            vec!["attribution request exceeds policy citation cap".to_owned()],
            policy_evidence_refs(&input),
        );
    }

    if input
        .request
        .claims
        .iter()
        .any(|claim| claim.confidence_bps < input.policy_decision.min_confidence_bps)
    {
        return domain_denial(
            &input,
            AttributionDomainDenialKind::ConfidenceBelowPolicy,
            None,
            vec!["attribution claim confidence is below policy floor".to_owned()],
            policy_evidence_refs(&input),
        );
    }

    if !source_kinds_are_policy_allowed(&input) {
        return domain_denial(
            &input,
            AttributionDomainDenialKind::SourceKindDenied,
            None,
            vec!["attribution sources include disallowed source kinds".to_owned()],
            policy_and_source_evidence_refs(&input),
        );
    }

    if !data_classes_are_policy_allowed(&input) {
        return domain_denial(
            &input,
            AttributionDomainDenialKind::DataClassDenied,
            None,
            vec!["attribution sources include disallowed data classes".to_owned()],
            policy_and_source_evidence_refs(&input),
        );
    }

    let report = plan_attribution(input.request.clone());
    if report.status == AttributionStatus::Denied {
        return domain_denial(
            &input,
            AttributionDomainDenialKind::KernelDenied,
            report.denial_kind,
            vec!["attribution kernel denied citation planning".to_owned()],
            sorted_unique([policy_evidence_refs(&input), report.evidence_refs].concat()),
        );
    }

    AttributionDomainDecision::Report(AttributionDomainReport {
        tenant_id: input.tenant_id,
        principal_id: input.principal_id,
        attribution_surface: input.attribution_surface,
        status: AttributionDomainStatus::Rendered,
        evidence_refs: sorted_unique(
            [
                vec![
                    input.request_evidence_ref,
                    input.policy_decision.attribution_registry_snapshot_ref,
                ],
                report.evidence_refs.clone(),
            ]
            .concat(),
        ),
        report,
    })
}

fn invalid_input_reasons(input: &DomainAttributionRequest) -> Vec<String> {
    let mut reasons = Vec::new();
    require_opaque("tenant id", &input.tenant_id, &mut reasons);
    require_opaque("principal id", &input.principal_id, &mut reasons);
    require_opaque(
        "attribution surface",
        &input.attribution_surface,
        &mut reasons,
    );
    require_opaque(
        "request evidence ref",
        &input.request_evidence_ref,
        &mut reasons,
    );
    require_opaque("trace context ref", &input.trace_context_ref, &mut reasons);
    require_opaque(
        "policy decision ref",
        &input.policy_decision_ref,
        &mut reasons,
    );
    require_metadata(
        "policy decision id",
        &input.policy_decision.decision_id,
        &mut reasons,
    );
    require_opaque(
        "policy tenant id",
        &input.policy_decision.tenant_id,
        &mut reasons,
    );
    require_opaque(
        "policy principal id",
        &input.policy_decision.principal_id,
        &mut reasons,
    );
    require_nonempty_opaque_vec(
        "policy allowed surface",
        &input.policy_decision.allowed_surfaces,
        &mut reasons,
    );
    if input.policy_decision.allowed_audiences.is_empty() {
        reasons.push("policy allowed audiences are required".to_owned());
    }
    if input.policy_decision.allowed_source_kinds.is_empty() {
        reasons.push("policy allowed source kinds are required".to_owned());
    }
    if input.policy_decision.allowed_data_classes.is_empty() {
        reasons.push("policy allowed data classes are required".to_owned());
    }
    if input.policy_decision.max_citations == 0 || input.policy_decision.max_citations > 100 {
        reasons.push("policy max citations must be 1..=100".to_owned());
    }
    if input.policy_decision.min_confidence_bps > 10_000 {
        reasons.push("policy confidence floor must be 0..=10000 basis points".to_owned());
    }
    require_opaque(
        "policy evidence ref",
        &input.policy_decision.evidence_ref,
        &mut reasons,
    );
    require_opaque(
        "attribution registry snapshot ref",
        &input.policy_decision.attribution_registry_snapshot_ref,
        &mut reasons,
    );
    require_opaque("kernel tenant id", &input.request.tenant_id, &mut reasons);
    require_opaque("kernel output ref", &input.request.output_ref, &mut reasons);
    require_opaque(
        "kernel policy evidence ref",
        &input.request.policy_evidence_ref,
        &mut reasons,
    );
    require_opaque(
        "kernel trace context ref",
        &input.request.trace_context_ref,
        &mut reasons,
    );
    sorted_unique(reasons)
}

fn source_kinds_are_policy_allowed(input: &DomainAttributionRequest) -> bool {
    input.request.sources.iter().all(|source| {
        input
            .policy_decision
            .allowed_source_kinds
            .contains(&source.source_kind)
    })
}

fn data_classes_are_policy_allowed(input: &DomainAttributionRequest) -> bool {
    input.request.sources.iter().all(|source| {
        input
            .policy_decision
            .allowed_data_classes
            .contains(&source.data_class)
    })
}

fn domain_denial(
    input: &DomainAttributionRequest,
    denial_kind: AttributionDomainDenialKind,
    kernel_denial_kind: Option<AttributionDenialKind>,
    reasons: Vec<String>,
    evidence_refs: Vec<String>,
) -> AttributionDomainDecision {
    AttributionDomainDecision::Deny(denial_from_parts(DenialParts {
        tenant_id: input.tenant_id.clone(),
        principal_id: input.principal_id.clone(),
        attribution_surface: input.attribution_surface.clone(),
        output_ref: input.request.output_ref.clone(),
        denial_kind,
        kernel_denial_kind,
        reasons,
        evidence_refs,
    }))
}

struct DenialParts {
    tenant_id: String,
    principal_id: String,
    attribution_surface: String,
    output_ref: String,
    denial_kind: AttributionDomainDenialKind,
    kernel_denial_kind: Option<AttributionDenialKind>,
    reasons: Vec<String>,
    evidence_refs: Vec<String>,
}

fn denial_from_parts(parts: DenialParts) -> AttributionDomainDenial {
    AttributionDomainDenial {
        tenant_id: parts.tenant_id,
        principal_id: parts.principal_id,
        attribution_surface: parts.attribution_surface,
        output_ref: parts.output_ref,
        status: AttributionDomainStatus::Denied,
        denial_kind: parts.denial_kind,
        kernel_denial_kind: parts.kernel_denial_kind,
        reasons: sorted_unique(parts.reasons),
        evidence_refs: sorted_unique(parts.evidence_refs),
    }
}

fn policy_evidence_refs(input: &DomainAttributionRequest) -> Vec<String> {
    sorted_unique(vec![
        input.request_evidence_ref.clone(),
        input.policy_decision.evidence_ref.clone(),
        input
            .policy_decision
            .attribution_registry_snapshot_ref
            .clone(),
    ])
}

fn policy_and_source_evidence_refs(input: &DomainAttributionRequest) -> Vec<String> {
    sorted_unique(
        [
            policy_evidence_refs(input),
            input
                .request
                .sources
                .iter()
                .map(|source| source.evidence_ref.clone())
                .collect(),
        ]
        .concat(),
    )
}

fn require_metadata(label: &str, value: &str, reasons: &mut Vec<String>) {
    if !is_safe_metadata_ref(value) {
        reasons.push(format!("{label} must be audit-safe metadata"));
    }
}

fn require_opaque(label: &str, value: &str, reasons: &mut Vec<String>) {
    if !is_safe_opaque_ref(value) {
        reasons.push(format!("{label} must be an opaque ref"));
    }
}

fn require_nonempty_opaque_vec(label: &str, values: &[String], reasons: &mut Vec<String>) {
    if values.is_empty() {
        reasons.push(format!("{label} entries are required"));
    }
    for value in values {
        require_opaque(label, value, reasons);
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
    fn authorized_domain_request_delegates_to_kernel_citation_plan() {
        let decision = plan_domain_attribution(sample_domain_request());
        let AttributionDomainDecision::Report(report) = decision else {
            panic!("expected report");
        };

        assert_eq!(report.status, AttributionDomainStatus::Rendered);
        assert_eq!(report.report.citations.len(), 2);
        assert!(
            report
                .evidence_refs
                .contains(&"attribution-registry:snapshot:1".to_owned())
        );
    }

    #[test]
    fn invalid_raw_metadata_denies_without_raw_echo() {
        let mut request = sample_domain_request();
        request.request.output_ref = "raw output model answer".to_owned();
        request.request.sources[0].resource_ref = "sk-test-secret".to_owned();

        let AttributionDomainDecision::Deny(denial) = plan_domain_attribution(request) else {
            panic!("expected denial");
        };
        let debug = format!("{denial:?}");

        assert_eq!(
            denial.denial_kind,
            AttributionDomainDenialKind::InvalidInput
        );
        assert_eq!(denial.output_ref, "redacted-invalid-output-ref");
        assert!(!debug.contains("raw output model answer"));
        assert!(!debug.contains("sk-test-secret"));
    }

    #[test]
    fn policy_drift_and_surface_denial_block_before_kernel() {
        let mut drift = sample_domain_request();
        drift.policy_decision.tenant_id = "tenant:other".to_owned();
        let AttributionDomainDecision::Deny(drift_denial) = plan_domain_attribution(drift) else {
            panic!("expected drift denial");
        };
        assert_eq!(
            drift_denial.denial_kind,
            AttributionDomainDenialKind::PolicyDrift
        );

        let mut surface = sample_domain_request();
        surface.attribution_surface = "surface:other".to_owned();
        let AttributionDomainDecision::Deny(surface_denial) = plan_domain_attribution(surface)
        else {
            panic!("expected surface denial");
        };
        assert_eq!(
            surface_denial.denial_kind,
            AttributionDomainDenialKind::SurfaceDenied
        );
    }

    #[test]
    fn audience_source_kind_and_data_class_policy_are_enforced() {
        let mut audience = sample_domain_request();
        audience.policy_decision.allowed_audiences = vec![AttributionAudience::Internal];
        let AttributionDomainDecision::Deny(audience_denial) = plan_domain_attribution(audience)
        else {
            panic!("expected audience denial");
        };
        assert_eq!(
            audience_denial.denial_kind,
            AttributionDomainDenialKind::AudienceDenied
        );

        let mut source_kind = sample_domain_request();
        source_kind.request.sources[0].source_kind = AttributionSourceKind::ToolResult;
        let AttributionDomainDecision::Deny(source_kind_denial) =
            plan_domain_attribution(source_kind)
        else {
            panic!("expected source-kind denial");
        };
        assert_eq!(
            source_kind_denial.denial_kind,
            AttributionDomainDenialKind::SourceKindDenied
        );

        let mut data_class = sample_domain_request();
        data_class.request.sources[0].data_class = AttributionDataClass::Restricted;
        let AttributionDomainDecision::Deny(data_class_denial) =
            plan_domain_attribution(data_class)
        else {
            panic!("expected data-class denial");
        };
        assert_eq!(
            data_class_denial.denial_kind,
            AttributionDomainDenialKind::DataClassDenied
        );
    }

    #[test]
    fn citation_cap_and_confidence_floor_are_enforced() {
        let mut cap = sample_domain_request();
        cap.request.max_citations = 9;
        cap.policy_decision.max_citations = 2;
        let AttributionDomainDecision::Deny(cap_denial) = plan_domain_attribution(cap) else {
            panic!("expected cap denial");
        };
        assert_eq!(
            cap_denial.denial_kind,
            AttributionDomainDenialKind::CitationLimitExceeded
        );

        let mut confidence = sample_domain_request();
        confidence.request.claims[0].confidence_bps = 6_000;
        confidence.policy_decision.min_confidence_bps = 8_000;
        let AttributionDomainDecision::Deny(confidence_denial) =
            plan_domain_attribution(confidence)
        else {
            panic!("expected confidence denial");
        };
        assert_eq!(
            confidence_denial.denial_kind,
            AttributionDomainDenialKind::ConfidenceBelowPolicy
        );
    }

    #[test]
    fn kernel_missing_source_denial_is_preserved_with_policy_evidence() {
        let mut request = sample_domain_request();
        request.request.claims[0]
            .source_ids
            .push("src-missing".to_owned());

        let AttributionDomainDecision::Deny(denial) = plan_domain_attribution(request) else {
            panic!("expected kernel denial");
        };

        assert_eq!(
            denial.denial_kind,
            AttributionDomainDenialKind::KernelDenied
        );
        assert_eq!(
            denial.kernel_denial_kind,
            Some(AttributionDenialKind::MissingSource)
        );
        assert!(
            denial
                .evidence_refs
                .contains(&"policy:evidence:attribution-domain:1".to_owned())
        );
    }

    fn sample_domain_request() -> DomainAttributionRequest {
        DomainAttributionRequest {
            tenant_id: "tenant:alpha".to_owned(),
            principal_id: "principal:attribution-owner".to_owned(),
            attribution_surface: "surface:dispatch-response".to_owned(),
            request_evidence_ref: "request:evidence:attribution-domain:1".to_owned(),
            trace_context_ref: "trace:attribution-domain:1".to_owned(),
            policy_decision_ref: "policy:evidence:attribution-domain:1".to_owned(),
            policy_decision: sample_policy(),
            request: sample_kernel_request(),
        }
    }

    fn sample_policy() -> AttributionPolicyDecision {
        AttributionPolicyDecision {
            decision_id: "attribution-policy-decision:1".to_owned(),
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
            evidence_ref: "policy:evidence:attribution-domain:1".to_owned(),
            attribution_registry_snapshot_ref: "attribution-registry:snapshot:1".to_owned(),
        }
    }

    fn sample_kernel_request() -> AttributionRequest {
        AttributionRequest {
            tenant_id: "tenant:alpha".to_owned(),
            output_ref: "answer://responses/resp-domain-1".to_owned(),
            audience: AttributionAudience::External,
            policy_evidence_ref: "policy:evidence:attribution-domain:1".to_owned(),
            trace_context_ref: "trace:attribution-domain:1".to_owned(),
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
                    answer_segment_ref: "answer-segment://resp-domain-1/2".to_owned(),
                    source_ids: vec!["src-doc-refund".to_owned()],
                    confidence_bps: 9_000,
                },
                AttributionClaim {
                    claim_id: "claim-1".to_owned(),
                    answer_segment_ref: "answer-segment://resp-domain-1/1".to_owned(),
                    source_ids: vec!["src-kg-policy".to_owned()],
                    confidence_bps: 9_200,
                },
            ],
        }
    }
}
