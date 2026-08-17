//! Intelligence attribution kernel foundation.
//!
//! This crate plans metadata-only citations for Intelligence responses. It maps
//! answer segment refs to source/resource/evidence refs, enforces external
//! audience data-class boundaries, and rejects raw prompt/output/document text
//! or secret-shaped values. It performs no retrieval, citation text rendering,
//! model calls, network I/O, filesystem access, durable storage, or policy
//! runtime execution.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

const MAX_CITATIONS: usize = 100;
const BASIS_POINTS_DENOMINATOR: u32 = 10_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AttributionAudience {
    External,
    Internal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AttributionDataClass {
    Confidential,
    Internal,
    Public,
    Restricted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AttributionSourceKind {
    KnowledgeGraph,
    PolicyDocument,
    RetrievalDocument,
    ToolResult,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributionSource {
    pub source_id: String,                  // data_class: INTERNAL_ONLY
    pub resource_ref: String,               // data_class: INTERNAL_ONLY
    pub title_ref: String,                  // data_class: INTERNAL_ONLY
    pub source_kind: AttributionSourceKind, // data_class: INTERNAL_ONLY
    pub data_class: AttributionDataClass,   // data_class: INTERNAL_ONLY
    pub evidence_ref: String,               // data_class: INTERNAL_ONLY
    pub freshness_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributionClaim {
    pub claim_id: String,           // data_class: INTERNAL_ONLY
    pub answer_segment_ref: String, // data_class: INTERNAL_ONLY
    pub source_ids: Vec<String>,    // data_class: INTERNAL_ONLY
    pub confidence_bps: u32,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributionRequest {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub output_ref: String,              // data_class: INTERNAL_ONLY
    pub audience: AttributionAudience,   // data_class: INTERNAL_ONLY
    pub policy_evidence_ref: String,     // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,       // data_class: INTERNAL_ONLY
    pub max_citations: usize,            // data_class: INTERNAL_ONLY
    pub max_citations_per_claim: usize,  // data_class: INTERNAL_ONLY
    pub sources: Vec<AttributionSource>, // data_class: INTERNAL_ONLY
    pub claims: Vec<AttributionClaim>,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AttributionStatus {
    Denied,
    Rendered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AttributionDenialKind {
    CitationLimitExceeded,
    ClaimCitationFanoutExceeded,
    InvalidInput,
    MissingSource,
    SensitiveExternalCitation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributionCitation {
    pub ordinal: u32,                       // data_class: PUBLIC
    pub claim_id: String,                   // data_class: INTERNAL_ONLY
    pub answer_segment_ref: String,         // data_class: INTERNAL_ONLY
    pub source_id: String,                  // data_class: INTERNAL_ONLY
    pub resource_ref: String,               // data_class: INTERNAL_ONLY
    pub title_ref: String,                  // data_class: INTERNAL_ONLY
    pub source_kind: AttributionSourceKind, // data_class: PUBLIC
    pub evidence_ref: String,               // data_class: INTERNAL_ONLY
    pub confidence_bps: u32,                // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributionReport {
    pub status: AttributionStatus,                  // data_class: PUBLIC
    pub denial_kind: Option<AttributionDenialKind>, // data_class: INTERNAL_ONLY
    pub citations: Vec<AttributionCitation>,        // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                 // data_class: INTERNAL_ONLY
}

pub fn plan_attribution(request: AttributionRequest) -> AttributionReport {
    let invalid = invalid_input_reasons(&request);
    if !invalid.is_empty() {
        return denied_report(
            AttributionDenialKind::InvalidInput,
            vec!["validation:intelligence-attribution-kernel-input".to_owned()],
        );
    }

    let sources_by_id = sources_by_id(&request.sources);
    if let Some(missing_evidence) = missing_source_evidence(&request, &sources_by_id) {
        return denied_report(AttributionDenialKind::MissingSource, missing_evidence);
    }

    if request.audience == AttributionAudience::External {
        let sensitive = sensitive_external_evidence(&request, &sources_by_id);
        if !sensitive.is_empty() {
            return denied_report(AttributionDenialKind::SensitiveExternalCitation, sensitive);
        }
    }

    let fanout_exceeded = request
        .claims
        .iter()
        .any(|claim| claim.source_ids.len() > request.max_citations_per_claim);
    if fanout_exceeded {
        return denied_report(
            AttributionDenialKind::ClaimCitationFanoutExceeded,
            vec![
                request.policy_evidence_ref.clone(),
                "validation:intelligence-attribution-claim-fanout".to_owned(),
            ],
        );
    }

    let planned = planned_citations(&request, &sources_by_id);
    if planned.len() > request.max_citations {
        return denied_report(
            AttributionDenialKind::CitationLimitExceeded,
            vec![
                request.policy_evidence_ref.clone(),
                "validation:intelligence-attribution-citation-limit".to_owned(),
            ],
        );
    }

    AttributionReport {
        status: AttributionStatus::Rendered,
        denial_kind: None,
        evidence_refs: sorted_unique(
            [
                vec![
                    request.policy_evidence_ref,
                    request.trace_context_ref,
                    request.output_ref,
                ],
                planned
                    .iter()
                    .map(|citation| citation.evidence_ref.clone())
                    .collect(),
            ]
            .concat(),
        ),
        citations: planned,
    }
}

fn invalid_input_reasons(request: &AttributionRequest) -> Vec<String> {
    let mut reasons = Vec::new();
    require_opaque("tenant id", &request.tenant_id, &mut reasons);
    require_opaque("output ref", &request.output_ref, &mut reasons);
    require_opaque(
        "policy evidence ref",
        &request.policy_evidence_ref,
        &mut reasons,
    );
    require_opaque(
        "trace context ref",
        &request.trace_context_ref,
        &mut reasons,
    );
    if request.max_citations == 0 || request.max_citations > MAX_CITATIONS {
        reasons.push(format!("max citations must be 1..={MAX_CITATIONS}"));
    }
    if request.max_citations_per_claim == 0
        || request.max_citations_per_claim > request.max_citations
    {
        reasons.push(format!(
            "max citations per claim must be 1..=max_citations ({})",
            request.max_citations
        ));
    }
    if request.sources.is_empty() {
        reasons.push("attribution sources are required".to_owned());
    }
    if request.claims.is_empty() {
        reasons.push("attribution claims are required".to_owned());
    }

    let mut source_ids = BTreeSet::new();
    for source in &request.sources {
        require_metadata("source id", &source.source_id, &mut reasons);
        require_opaque("source resource ref", &source.resource_ref, &mut reasons);
        require_opaque("source title ref", &source.title_ref, &mut reasons);
        require_opaque("source evidence ref", &source.evidence_ref, &mut reasons);
        if !source_ids.insert(source.source_id.clone()) {
            reasons.push("source ids must be unique".to_owned());
        }
    }
    for claim in &request.claims {
        require_metadata("claim id", &claim.claim_id, &mut reasons);
        require_opaque(
            "claim answer segment ref",
            &claim.answer_segment_ref,
            &mut reasons,
        );
        if claim.source_ids.is_empty() {
            reasons.push("claim source ids are required".to_owned());
        }
        if claim.confidence_bps > BASIS_POINTS_DENOMINATOR {
            reasons.push("claim confidence must be 0..=10000 basis points".to_owned());
        }
        let mut seen_in_claim = BTreeSet::new();
        for source_id in &claim.source_ids {
            require_metadata("claim source id", source_id, &mut reasons);
            if !seen_in_claim.insert(source_id.as_str()) {
                reasons.push(format!(
                    "claim {} contains duplicate source_id: {source_id}",
                    claim.claim_id
                ));
            }
        }
    }
    sorted_unique(reasons)
}

fn sources_by_id(sources: &[AttributionSource]) -> BTreeMap<String, AttributionSource> {
    sources
        .iter()
        .map(|source| (source.source_id.clone(), source.clone()))
        .collect()
}

fn missing_source_evidence(
    request: &AttributionRequest,
    sources_by_id: &BTreeMap<String, AttributionSource>,
) -> Option<Vec<String>> {
    let missing = request.claims.iter().any(|claim| {
        claim
            .source_ids
            .iter()
            .any(|source_id| !sources_by_id.contains_key(source_id))
    });
    missing.then(|| {
        sorted_unique(vec![
            request.policy_evidence_ref.clone(),
            "validation:intelligence-attribution-missing-source".to_owned(),
        ])
    })
}

fn sensitive_external_evidence(
    request: &AttributionRequest,
    sources_by_id: &BTreeMap<String, AttributionSource>,
) -> Vec<String> {
    let mut evidence = vec![request.policy_evidence_ref.clone()];
    for claim in &request.claims {
        for source_id in sorted_unique(claim.source_ids.clone()) {
            if let Some(source) = sources_by_id.get(&source_id)
                && source.data_class != AttributionDataClass::Public
            {
                evidence.push(source.evidence_ref.clone());
            }
        }
    }
    if evidence.len() == 1 {
        Vec::new()
    } else {
        sorted_unique(evidence)
    }
}

fn planned_citations(
    request: &AttributionRequest,
    sources_by_id: &BTreeMap<String, AttributionSource>,
) -> Vec<AttributionCitation> {
    let mut claims = request.claims.clone();
    claims.sort_by(|left, right| left.claim_id.cmp(&right.claim_id));
    let mut citations = Vec::new();
    for claim in claims {
        for source_id in sorted_unique(claim.source_ids.clone()) {
            let source = sources_by_id
                .get(&source_id)
                .expect("source existence already validated");
            citations.push(AttributionCitation {
                ordinal: (citations.len() + 1) as u32,
                claim_id: claim.claim_id.clone(),
                answer_segment_ref: claim.answer_segment_ref.clone(),
                source_id: source.source_id.clone(),
                resource_ref: source.resource_ref.clone(),
                title_ref: source.title_ref.clone(),
                source_kind: source.source_kind,
                evidence_ref: source.evidence_ref.clone(),
                confidence_bps: claim.confidence_bps,
            });
        }
    }
    citations
}

fn denied_report(kind: AttributionDenialKind, evidence_refs: Vec<String>) -> AttributionReport {
    AttributionReport {
        status: AttributionStatus::Denied,
        denial_kind: Some(kind),
        citations: Vec::new(),
        evidence_refs: sorted_unique(evidence_refs),
    }
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
    fn plans_metadata_only_citations_for_public_external_answer() {
        let report = plan_attribution(sample_request());

        assert_eq!(report.status, AttributionStatus::Rendered);
        assert_eq!(report.denial_kind, None);
        assert_eq!(report.citations.len(), 2);
        assert_eq!(report.citations[0].ordinal, 1);
        assert_eq!(
            report.citations[0].resource_ref,
            "kg://entity/accounting-policy"
        );
        assert_eq!(
            report.citations[1].resource_ref,
            "doc://help-center/refund-policy"
        );
        assert!(
            report
                .evidence_refs
                .contains(&"policy:evidence:attribution:1".to_owned())
        );
    }

    #[test]
    fn sensitive_sources_are_denied_for_external_audience() {
        let mut request = sample_request();
        request.sources[0].data_class = AttributionDataClass::Confidential;

        let report = plan_attribution(request);

        assert_eq!(report.status, AttributionStatus::Denied);
        assert_eq!(
            report.denial_kind,
            Some(AttributionDenialKind::SensitiveExternalCitation)
        );
        assert!(report.citations.is_empty());
        assert!(
            report
                .evidence_refs
                .contains(&"evidence:kg:accounting-policy".to_owned())
        );
    }

    #[test]
    fn missing_source_ids_deny_before_rendering() {
        let mut request = sample_request();
        request.claims[0].source_ids.push("src-missing".to_owned());

        let report = plan_attribution(request);

        assert_eq!(report.status, AttributionStatus::Denied);
        assert_eq!(
            report.denial_kind,
            Some(AttributionDenialKind::MissingSource)
        );
        assert!(report.citations.is_empty());
    }

    #[test]
    fn raw_output_or_secret_shaped_refs_are_rejected_without_echo() {
        let mut request = sample_request();
        request.output_ref = "raw output model answer".to_owned();
        request.sources[0].resource_ref = "sk-test-secret".to_owned();

        let report = plan_attribution(request);
        let debug = format!("{report:?}");

        assert_eq!(report.status, AttributionStatus::Denied);
        assert_eq!(
            report.denial_kind,
            Some(AttributionDenialKind::InvalidInput)
        );
        assert!(report.citations.is_empty());
        assert!(!debug.contains("raw output model answer"));
        assert!(!debug.contains("sk-test-secret"));
    }

    #[test]
    fn citation_limit_denies_deterministically() {
        let mut request = sample_request();
        request.max_citations = 1;
        request.max_citations_per_claim = 1;

        let report = plan_attribution(request);

        assert_eq!(report.status, AttributionStatus::Denied);
        assert_eq!(
            report.denial_kind,
            Some(AttributionDenialKind::CitationLimitExceeded)
        );
        assert_eq!(
            report.evidence_refs,
            vec![
                "policy:evidence:attribution:1".to_owned(),
                "validation:intelligence-attribution-citation-limit".to_owned()
            ]
        );
    }

    #[test]
    fn citation_order_is_deterministic_regardless_of_input_order() {
        // Reverse both claims and sources order; result must still be
        // claim-1 -> src-kg-policy, claim-2 -> src-doc-refund
        let mut request = sample_request();
        request.claims.reverse();
        request.sources.reverse();

        let report = plan_attribution(request);

        assert_eq!(report.status, AttributionStatus::Rendered);
        assert_eq!(report.citations.len(), 2);
        assert_eq!(report.citations[0].claim_id, "claim-1");
        assert_eq!(report.citations[0].source_id, "src-kg-policy");
        assert_eq!(report.citations[1].claim_id, "claim-2");
        assert_eq!(report.citations[1].source_id, "src-doc-refund");
    }

    #[test]
    fn per_claim_fanout_cap_exceeded_is_denied_with_correct_evidence() {
        let mut request = sample_request();
        // Set cap to 1 but claim-1 already has 1 source; add a second to exceed it
        request.max_citations_per_claim = 1;
        request.claims[1]
            .source_ids
            .push("src-doc-refund".to_owned());
        // Also make src-doc-refund available to claim-1 (it's claim_id "claim-1" after sort)
        // claim-1 has source_ids = ["src-kg-policy", "src-doc-refund"] => len 2 > cap 1
        let report = plan_attribution(request);

        assert_eq!(report.status, AttributionStatus::Denied);
        assert_eq!(
            report.denial_kind,
            Some(AttributionDenialKind::ClaimCitationFanoutExceeded)
        );
        assert!(report.citations.is_empty());
        assert!(
            report
                .evidence_refs
                .contains(&"policy:evidence:attribution:1".to_owned())
        );
        assert!(
            report
                .evidence_refs
                .contains(&"validation:intelligence-attribution-claim-fanout".to_owned())
        );
    }

    #[test]
    fn duplicate_source_ids_within_claim_is_denied_as_invalid_input() {
        let mut request = sample_request();
        // Inject duplicate source_id in claim-2 (index 0 in sample = claim-2)
        let dup_id = request.claims[0].source_ids[0].clone();
        request.claims[0].source_ids.push(dup_id);

        let report = plan_attribution(request);

        assert_eq!(report.status, AttributionStatus::Denied);
        assert_eq!(
            report.denial_kind,
            Some(AttributionDenialKind::InvalidInput)
        );
        assert!(report.citations.is_empty());
    }

    #[test]
    fn within_claim_source_ordering_is_stable_lexicographic() {
        let mut request = sample_request();
        // Give claim-1 two sources; add a second source that sorts before src-kg-policy
        request.sources.push(AttributionSource {
            source_id: "src-aaa-early".to_owned(),
            resource_ref: "doc://early/resource".to_owned(),
            title_ref: "title://early/resource".to_owned(),
            source_kind: AttributionSourceKind::RetrievalDocument,
            data_class: AttributionDataClass::Public,
            evidence_ref: "evidence:early:resource".to_owned(),
            freshness_epoch_seconds: 1_779_523_202,
        });
        // claim-1 is at index 1 in sample_request claims
        request.claims[1]
            .source_ids
            .push("src-aaa-early".to_owned());
        // max_citations_per_claim must accommodate 2 sources
        request.max_citations_per_claim = 2;
        request.max_citations = 8;

        let report = plan_attribution(request);

        assert_eq!(report.status, AttributionStatus::Rendered);
        // claim-1 should yield citations for "src-aaa-early" then "src-kg-policy"
        // (lexicographic order within claim)
        let claim_1_citations: Vec<_> = report
            .citations
            .iter()
            .filter(|c| c.claim_id == "claim-1")
            .collect();
        assert_eq!(claim_1_citations.len(), 2);
        assert_eq!(claim_1_citations[0].source_id, "src-aaa-early");
        assert_eq!(claim_1_citations[1].source_id, "src-kg-policy");
    }

    fn sample_request() -> AttributionRequest {
        AttributionRequest {
            tenant_id: "tenant:alpha".to_owned(),
            output_ref: "answer://responses/resp-1".to_owned(),
            audience: AttributionAudience::External,
            policy_evidence_ref: "policy:evidence:attribution:1".to_owned(),
            trace_context_ref: "trace:attribution:1".to_owned(),
            max_citations: 8,
            max_citations_per_claim: 4,
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
                    answer_segment_ref: "answer-segment://resp-1/2".to_owned(),
                    source_ids: vec!["src-doc-refund".to_owned()],
                    confidence_bps: 9_000,
                },
                AttributionClaim {
                    claim_id: "claim-1".to_owned(),
                    answer_segment_ref: "answer-segment://resp-1/1".to_owned(),
                    source_ids: vec!["src-kg-policy".to_owned()],
                    confidence_bps: 9_200,
                },
            ],
        }
    }
}
