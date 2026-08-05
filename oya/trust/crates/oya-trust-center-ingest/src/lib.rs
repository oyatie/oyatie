//! Trust Center evidence ingestion adapter boundary.
//!
//! Implements the TRUSTCENTER-INGEST-001 transport-neutral ingestion slice from
//! `specs/trust-center-compliance-evidence-portal.json#evidence_source_map`.
//! The crate normalizes source-family fixture records into `trust_center_*`
//! read-model records, applies customer-safe redaction, and emits append-only
//! publishability decision records. It deliberately does not fetch live scanner
//! output, mint external certifications, publish tenant artifacts, or expose raw
//! operator/security detail.
//!
//! non_claim: in-memory adapter/policy contract only; no storage adapter, live
//! collector, tenant-facing publish path, or external certification workflow.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;

use oya_trust_center_api::{
    TRUST_CENTER_COMPLIANCE_PACK_RECORD_TYPE, TRUST_CENTER_CONTROL_FRESHNESS_RECORD_TYPE,
    TRUST_CENTER_EVIDENCE_ITEM_RECORD_TYPE, TRUST_CENTER_PUBLISHABILITY_DECISION_RECORD_TYPE,
    TRUST_CENTER_SBOM_VEX_RECORD_TYPE, TRUST_CENTER_SCHEMA_VERSION, TrustCenterClaimTier,
    TrustCenterCommonFields, TrustCenterCompliancePackViewRecord,
    TrustCenterControlFreshnessRecord, TrustCenterDataClass, TrustCenterEvidenceItemRecord,
    TrustCenterFreshnessState, TrustCenterPublishabilityDecisionRecord,
    TrustCenterPublishabilityState, TrustCenterSbomVexViewRecord,
};

const TRUST_CENTER_INGEST_SOURCE_REF_PREFIX: &str = "trust-source://";
const TRUST_CENTER_OPERATOR_POLICY_PRINCIPAL: &str = "principal_oyatie_operator_policy";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TrustCenterEvidenceSourceFamily {
    SecurityValidationControls,
    SbomVexVulnerabilityPosture,
    CompliancePackActivation,
    SloDrStatusIncident,
    CloudQualityKits,
    ReleaseAndAuditChain,
}

impl TrustCenterEvidenceSourceFamily {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SecurityValidationControls => "security_validation_controls",
            Self::SbomVexVulnerabilityPosture => "sbom_vex_vulnerability_posture",
            Self::CompliancePackActivation => "compliance_pack_activation",
            Self::SloDrStatusIncident => "slo_dr_status_incident",
            Self::CloudQualityKits => "cloud_quality_kits",
            Self::ReleaseAndAuditChain => "release_and_audit_chain",
        }
    }

    pub const fn minimum_fields(self) -> &'static [&'static str] {
        match self {
            Self::SecurityValidationControls => &[
                "tenant_id",
                "lane_id",
                "subject_ref",
                "result",
                "severity_summary",
                "exception_refs",
                "audit_event_ref",
                "retention_until",
            ],
            Self::SbomVexVulnerabilityPosture => &[
                "tenant_id_or_internal_lane",
                "artifact_digest",
                "component_purl",
                "vulnerability_id",
                "sbom_refs",
                "vex_ref",
                "priority_signals",
                "exception_ref",
                "verdict",
            ],
            Self::CompliancePackActivation => &[
                "pack_id",
                "version",
                "signed_by",
                "audit_chain_requirements",
                "data_class_extensions",
                "cell_eligibility",
                "retention_rules",
                "regulator_references",
            ],
            Self::SloDrStatusIncident => &[
                "service_id",
                "tenant_or_pack_scope",
                "slo_state",
                "dr_floor_ref",
                "incident_ref",
                "postmortem_ref",
                "audit_event_ref",
            ],
            Self::CloudQualityKits => &[
                "kit_id",
                "scenario_id",
                "evidence_ref",
                "gate",
                "freshness_state",
                "non_claim_state",
            ],
            Self::ReleaseAndAuditChain => &[
                "change_id",
                "claim_tier",
                "evidence_packet_ref",
                "audit_event_ref",
                "release_note_or_governance_impact",
            ],
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceHealth {
    Current,
    AgingWarning,
    Stale,
    Missing,
    ParserError,
    ExpiredException,
    UnknownApplicability,
    NotApplicableWithPolicyReason(String),
    BlockedPendingReview,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustCenterSourceEvidence {
    pub family: TrustCenterEvidenceSourceFamily,
    pub tenant_id: String,
    pub source_id: String,
    pub observed_at_trusted: String,
    pub retention_until: String,
    pub health: SourceHealth,
    pub fields: BTreeMap<String, String>,
    pub customer_summary: String,
    pub operator_only_detail: BTreeMap<String, String>,
    pub audit_event_ref: Option<String>,
}

impl TrustCenterSourceEvidence {
    pub fn new(
        family: TrustCenterEvidenceSourceFamily,
        tenant_id: &str,
        source_id: &str,
        observed_at_trusted: &str,
        retention_until: &str,
        health: SourceHealth,
    ) -> Self {
        Self {
            family,
            tenant_id: tenant_id.to_owned(),
            source_id: source_id.to_owned(),
            observed_at_trusted: observed_at_trusted.to_owned(),
            retention_until: retention_until.to_owned(),
            health,
            fields: BTreeMap::new(),
            customer_summary: String::new(),
            operator_only_detail: BTreeMap::new(),
            audit_event_ref: None,
        }
    }

    #[must_use]
    pub fn with_field(mut self, field: &str, value: &str) -> Self {
        self.fields.insert(field.to_owned(), value.to_owned());
        self
    }

    #[must_use]
    pub fn without_minimum_field(mut self, field: &str) -> Self {
        self.fields.remove(field);
        self
    }

    #[must_use]
    pub fn with_customer_summary(mut self, summary: &str) -> Self {
        self.customer_summary = summary.to_owned();
        self
    }

    #[must_use]
    pub fn with_operator_detail(mut self, field: &str, value: &str) -> Self {
        self.operator_only_detail
            .insert(field.to_owned(), value.to_owned());
        self
    }

    #[must_use]
    pub fn with_audit_event_ref(mut self, audit_event_ref: &str) -> Self {
        self.audit_event_ref = Some(audit_event_ref.to_owned());
        self
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrustCenterIngestionBatch {
    pub evidence_items: Vec<TrustCenterEvidenceItemRecord>,
    pub control_freshness: Vec<TrustCenterControlFreshnessRecord>,
    pub sbom_vex: Vec<TrustCenterSbomVexViewRecord>,
    pub compliance_packs: Vec<TrustCenterCompliancePackViewRecord>,
    pub publishability_decisions: Vec<TrustCenterPublishabilityDecisionRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TrustCenterIngestError {
    EmptySourceSet,
    MissingMinimumField {
        family: TrustCenterEvidenceSourceFamily,
        source_id: String,
        field: &'static str,
    },
    TenantAssertionMismatch {
        source_id: String,
        trusted_tenant_id: String,
        asserted_tenant_id: String,
    },
    InvalidSourceShape {
        source_id: String,
        reason: String,
    },
}

impl fmt::Display for TrustCenterIngestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySourceSet => write!(f, "TRUST_CENTER_INGEST_EMPTY_SOURCE_SET"),
            Self::MissingMinimumField { family, field, .. } => write!(
                f,
                "TRUST_CENTER_INGEST_MISSING_MINIMUM_FIELD family={} field={}",
                family.as_str(),
                field
            ),
            Self::TenantAssertionMismatch { .. } => {
                write!(f, "TRUST_CENTER_INGEST_TENANT_ASSERTION_MISMATCH")
            }
            Self::InvalidSourceShape { reason, .. } => {
                write!(f, "TRUST_CENTER_INGEST_INVALID_SOURCE_SHAPE {reason}")
            }
        }
    }
}

impl std::error::Error for TrustCenterIngestError {}

pub fn ingest_trust_center_sources(
    sources: &[TrustCenterSourceEvidence],
) -> Result<TrustCenterIngestionBatch, TrustCenterIngestError> {
    if sources.is_empty() {
        return Err(TrustCenterIngestError::EmptySourceSet);
    }

    let mut batch = TrustCenterIngestionBatch::default();
    for (idx, source) in sources.iter().enumerate() {
        validate_source(source)?;
        let sequence = idx + 1;
        let item = evidence_item_record(source, sequence)?;
        match source.family {
            TrustCenterEvidenceSourceFamily::SecurityValidationControls => {
                batch
                    .control_freshness
                    .push(control_freshness_record(source, sequence)?);
            }
            TrustCenterEvidenceSourceFamily::SbomVexVulnerabilityPosture => {
                batch.sbom_vex.push(sbom_vex_record(source, sequence)?);
            }
            TrustCenterEvidenceSourceFamily::CompliancePackActivation => {
                batch
                    .compliance_packs
                    .push(compliance_pack_record(source, sequence)?);
            }
            TrustCenterEvidenceSourceFamily::SloDrStatusIncident
            | TrustCenterEvidenceSourceFamily::CloudQualityKits
            | TrustCenterEvidenceSourceFamily::ReleaseAndAuditChain => {}
        }
        let decision = publishability_decision_record(source, &item, sequence)?;
        batch.evidence_items.push(item);
        batch.publishability_decisions.push(decision);
    }
    Ok(batch)
}

fn validate_source(source: &TrustCenterSourceEvidence) -> Result<(), TrustCenterIngestError> {
    if source.source_id.trim().is_empty() {
        return Err(TrustCenterIngestError::InvalidSourceShape {
            source_id: source.source_id.clone(),
            reason: "source_id must be non-empty".to_owned(),
        });
    }
    if source.tenant_id.trim().is_empty() || !source.tenant_id.starts_with("ten_") {
        return Err(TrustCenterIngestError::InvalidSourceShape {
            source_id: source.source_id.clone(),
            reason: "tenant_id must be trusted ten_ scope".to_owned(),
        });
    }
    for field in source.family.minimum_fields() {
        if source
            .fields
            .get(*field)
            .is_none_or(|value| value.trim().is_empty())
        {
            return Err(TrustCenterIngestError::MissingMinimumField {
                family: source.family,
                source_id: source.source_id.clone(),
                field,
            });
        }
    }
    for field in [
        "tenant_id",
        "tenant_id_or_internal_lane",
        "tenant_or_pack_scope",
    ] {
        if let Some(asserted) = source.fields.get(field)
            && asserted.starts_with("ten_")
            && asserted != &source.tenant_id
        {
            return Err(TrustCenterIngestError::TenantAssertionMismatch {
                source_id: source.source_id.clone(),
                trusted_tenant_id: source.tenant_id.clone(),
                asserted_tenant_id: asserted.clone(),
            });
        }
    }
    Ok(())
}

fn evidence_item_record(
    source: &TrustCenterSourceEvidence,
    sequence: usize,
) -> Result<TrustCenterEvidenceItemRecord, TrustCenterIngestError> {
    let common = common_fields(
        source,
        sequence,
        "ev",
        TRUST_CENTER_EVIDENCE_ITEM_RECORD_TYPE,
        evidence_data_class(source.family),
        state_for(source).0,
        state_for(source).1,
    )?;
    Ok(TrustCenterEvidenceItemRecord {
        common,
        title: format!(
            "{} evidence {}",
            source.family.as_str().replace('_', " "),
            source.source_id
        ),
        customer_safe_summary: customer_safe_summary(source),
        source_links: vec![source_record_ref(source)],
        compliance_pack_ids: compliance_pack_ids(source),
        service_ids: service_ids(source),
        redacted_fields: redacted_fields(source),
        operator_only_detail_present: false,
        raw_operator_payload_exposed: false,
    })
}

fn control_freshness_record(
    source: &TrustCenterSourceEvidence,
    sequence: usize,
) -> Result<TrustCenterControlFreshnessRecord, TrustCenterIngestError> {
    let common = common_fields(
        source,
        sequence,
        "ctrl",
        TRUST_CENTER_CONTROL_FRESHNESS_RECORD_TYPE,
        TrustCenterDataClass::TenantTrustEvidence,
        state_for(source).0,
        state_for(source).1,
    )?;
    Ok(TrustCenterControlFreshnessRecord {
        common,
        control_id: format!("control_{}", slug(field(source, "lane_id")?)),
        lane_id: field(source, "lane_id")?.to_owned(),
        service_id: Some(field(source, "subject_ref")?.to_owned()),
        compliance_pack_ids: compliance_pack_ids(source),
        last_observed_at_trusted: source.observed_at_trusted.clone(),
        stale_after_trusted: source.retention_until.clone(),
        source_evidence_ref: record_id(source, sequence, "ev"),
    })
}

fn sbom_vex_record(
    source: &TrustCenterSourceEvidence,
    sequence: usize,
) -> Result<TrustCenterSbomVexViewRecord, TrustCenterIngestError> {
    let common = common_fields(
        source,
        sequence,
        "sbom",
        TRUST_CENTER_SBOM_VEX_RECORD_TYPE,
        TrustCenterDataClass::TenantTrustEvidence,
        state_for(source).0,
        state_for(source).1,
    )?;
    let verdict = field(source, "verdict")?.to_owned();
    Ok(TrustCenterSbomVexViewRecord {
        common,
        artifact_ref: format!(
            "{}@{}",
            field(source, "component_purl")?,
            field(source, "artifact_digest")?
        ),
        signed_sbom_ref: Some(field(source, "sbom_refs")?.to_owned()),
        vex_ref: Some(field(source, "vex_ref")?.to_owned()),
        vulnerability_status_counts: BTreeMap::from([(verdict, 1)]),
        exception_refs: vec![field(source, "exception_ref")?.to_owned()],
        remediation_sla_class: field(source, "priority_signals")?.to_owned(),
        raw_scanner_output_exposed: false,
        exploit_detail_exposed: false,
    })
}

fn compliance_pack_record(
    source: &TrustCenterSourceEvidence,
    sequence: usize,
) -> Result<TrustCenterCompliancePackViewRecord, TrustCenterIngestError> {
    let common = common_fields(
        source,
        sequence,
        "pack",
        TRUST_CENTER_COMPLIANCE_PACK_RECORD_TYPE,
        TrustCenterDataClass::RegulatedExportEvidence,
        state_for(source).0,
        state_for(source).1,
    )?;
    Ok(TrustCenterCompliancePackViewRecord {
        common,
        compliance_pack_id: field(source, "pack_id")?.to_owned(),
        version: field(source, "version")?.to_owned(),
        regulator_references: split_refs(field(source, "regulator_references")?),
        data_classes: vec![
            TrustCenterDataClass::TenantTrustEvidence,
            TrustCenterDataClass::RegulatedExportEvidence,
        ],
        residency_summary: field(source, "cell_eligibility")?.to_owned(),
        retention_days: parse_retention_days(field(source, "retention_rules")?),
        dr_floor_ref: Some(field(source, "audit_chain_requirements")?.to_owned()),
        breach_workflow_ref: Some("workflow://breach-notification-policy".to_owned()),
        activated: matches!(
            source.health,
            SourceHealth::Current | SourceHealth::AgingWarning
        ),
    })
}

fn publishability_decision_record(
    source: &TrustCenterSourceEvidence,
    item: &TrustCenterEvidenceItemRecord,
    sequence: usize,
) -> Result<TrustCenterPublishabilityDecisionRecord, TrustCenterIngestError> {
    let (_, new_state) = state_for(source);
    let record_id = record_id(source, sequence, "pub_dec");
    let common = TrustCenterCommonFields {
        record_id: record_id.clone(),
        record_type: TRUST_CENTER_PUBLISHABILITY_DECISION_RECORD_TYPE.to_owned(),
        schema_version: TRUST_CENTER_SCHEMA_VERSION,
        tenant_id: source.tenant_id.clone(),
        audience_id: "aud_customer_trust".to_owned(),
        source_system: source.family.as_str().to_owned(),
        source_record_ref: source_record_ref(source),
        evidence_class: "publishability_decision".to_owned(),
        data_class: TrustCenterDataClass::TenantTrustEvidence,
        claim_tier: TrustCenterClaimTier::SpecReady,
        freshness_state: TrustCenterFreshnessState::Current,
        publishability_state: TrustCenterPublishabilityState::TenantAdminOnly,
        redaction_policy_id: "redact_trust_center_publishability_v1".to_owned(),
        audit_event_ref: format!("audit/trust_center_publishability/{record_id}"),
        created_at_trusted: source.observed_at_trusted.clone(),
        expires_at_trusted_or_retention_until: source.retention_until.clone(),
    };
    Ok(TrustCenterPublishabilityDecisionRecord {
        common,
        decision_id: format!("decision_{}_{}", sequence, slug(&source.source_id)),
        evidence_id: item.common.record_id.clone(),
        previous_state: TrustCenterPublishabilityState::BlockedSecurityPrivacyReview,
        new_state,
        reason: decision_reason(source),
        decided_by_principal_id: TRUST_CENTER_OPERATOR_POLICY_PRINCIPAL.to_owned(),
    })
}

fn common_fields(
    source: &TrustCenterSourceEvidence,
    sequence: usize,
    prefix: &str,
    record_type: &str,
    data_class: TrustCenterDataClass,
    freshness_state: TrustCenterFreshnessState,
    publishability_state: TrustCenterPublishabilityState,
) -> Result<TrustCenterCommonFields, TrustCenterIngestError> {
    Ok(TrustCenterCommonFields {
        record_id: record_id(source, sequence, prefix),
        record_type: record_type.to_owned(),
        schema_version: TRUST_CENTER_SCHEMA_VERSION,
        tenant_id: source.tenant_id.clone(),
        audience_id: "aud_customer_trust".to_owned(),
        source_system: source.family.as_str().to_owned(),
        source_record_ref: source_record_ref(source),
        evidence_class: source.family.as_str().to_owned(),
        data_class,
        claim_tier: claim_tier(source),
        freshness_state,
        publishability_state,
        redaction_policy_id: format!("redact_{}_customer_safe_v1", source.family.as_str()),
        audit_event_ref: source
            .audit_event_ref
            .clone()
            .unwrap_or_else(|| format!("audit/source/{}", slug(&source.source_id))),
        created_at_trusted: source.observed_at_trusted.clone(),
        expires_at_trusted_or_retention_until: source.retention_until.clone(),
    })
}

fn state_for(
    source: &TrustCenterSourceEvidence,
) -> (TrustCenterFreshnessState, TrustCenterPublishabilityState) {
    match &source.health {
        SourceHealth::Current => (
            TrustCenterFreshnessState::Current,
            current_publishability(source.family),
        ),
        SourceHealth::AgingWarning => (
            TrustCenterFreshnessState::AgingWarning,
            current_publishability(source.family),
        ),
        SourceHealth::Stale => (
            TrustCenterFreshnessState::Stale,
            TrustCenterPublishabilityState::BlockedStaleEvidence,
        ),
        SourceHealth::Missing => (
            TrustCenterFreshnessState::Missing,
            TrustCenterPublishabilityState::BlockedMissingEvidence,
        ),
        SourceHealth::ParserError | SourceHealth::UnknownApplicability => (
            TrustCenterFreshnessState::BlockedPendingReview,
            TrustCenterPublishabilityState::BlockedSecurityPrivacyReview,
        ),
        SourceHealth::ExpiredException => (
            TrustCenterFreshnessState::Stale,
            TrustCenterPublishabilityState::BlockedStaleEvidence,
        ),
        SourceHealth::NotApplicableWithPolicyReason(_) => (
            TrustCenterFreshnessState::NotApplicableWithPolicyReason,
            TrustCenterPublishabilityState::NotApplicableWithPolicyReason,
        ),
        SourceHealth::BlockedPendingReview => (
            TrustCenterFreshnessState::BlockedPendingReview,
            TrustCenterPublishabilityState::BlockedSecurityPrivacyReview,
        ),
    }
}

fn current_publishability(
    family: TrustCenterEvidenceSourceFamily,
) -> TrustCenterPublishabilityState {
    match family {
        TrustCenterEvidenceSourceFamily::SecurityValidationControls => {
            TrustCenterPublishabilityState::PublishableCustomerSafe
        }
        TrustCenterEvidenceSourceFamily::CompliancePackActivation => {
            TrustCenterPublishabilityState::TenantAdminOnly
        }
        TrustCenterEvidenceSourceFamily::SbomVexVulnerabilityPosture
        | TrustCenterEvidenceSourceFamily::SloDrStatusIncident
        | TrustCenterEvidenceSourceFamily::CloudQualityKits
        | TrustCenterEvidenceSourceFamily::ReleaseAndAuditChain => {
            TrustCenterPublishabilityState::PublishableSummaryOnly
        }
    }
}

fn claim_tier(source: &TrustCenterSourceEvidence) -> TrustCenterClaimTier {
    match source.family {
        TrustCenterEvidenceSourceFamily::CloudQualityKits => TrustCenterClaimTier::TargetNonClaim,
        TrustCenterEvidenceSourceFamily::ReleaseAndAuditChain => source
            .fields
            .get("claim_tier")
            .map_or(TrustCenterClaimTier::SpecReady, |tier| {
                match tier.as_str() {
                    "mechanically_enforced" => TrustCenterClaimTier::MechanicallyEnforced,
                    "target_non_claim" => TrustCenterClaimTier::TargetNonClaim,
                    _ => TrustCenterClaimTier::SpecReady,
                }
            }),
        _ => TrustCenterClaimTier::SpecReady,
    }
}

fn evidence_data_class(family: TrustCenterEvidenceSourceFamily) -> TrustCenterDataClass {
    match family {
        TrustCenterEvidenceSourceFamily::CompliancePackActivation => {
            TrustCenterDataClass::RegulatedExportEvidence
        }
        _ => TrustCenterDataClass::TenantTrustEvidence,
    }
}

fn customer_safe_summary(source: &TrustCenterSourceEvidence) -> String {
    let health_suffix = match &source.health {
        SourceHealth::Current => "current customer-safe summary",
        SourceHealth::AgingWarning => "aging_warning customer-safe summary",
        SourceHealth::Stale => "blocked_stale_evidence",
        SourceHealth::Missing => "blocked_missing_evidence",
        SourceHealth::ParserError => "blocked_security_privacy_review parser_error",
        SourceHealth::ExpiredException => "blocked_stale_evidence expired_exception",
        SourceHealth::UnknownApplicability => {
            "blocked_security_privacy_review unknown_applicability"
        }
        SourceHealth::NotApplicableWithPolicyReason(reason) => {
            return format!(
                "not_applicable_with_policy_reason for {}: {}",
                source.family.as_str(),
                safe_policy_reason(reason)
            );
        }
        SourceHealth::BlockedPendingReview => "blocked_security_privacy_review",
    };
    format!("{} {health_suffix}", source.family.as_str())
}

fn redacted_fields(source: &TrustCenterSourceEvidence) -> Vec<String> {
    let mut redacted = Vec::new();
    for key in source.operator_only_detail.keys() {
        push_unique(&mut redacted, key);
    }
    let haystack = format!(
        "{} {}",
        source.customer_summary,
        source
            .operator_only_detail
            .values()
            .cloned()
            .collect::<Vec<_>>()
            .join(" ")
    )
    .to_ascii_lowercase();
    for (needle, field) in [
        ("secret", "secret"),
        ("token", "secret"),
        ("@", "pii"),
        ("pii", "pii"),
        ("exploit", "exploit_payload"),
        ("payload", "exploit_payload"),
        ("tenant ten_", "cross_tenant_identifier"),
    ] {
        if haystack.contains(needle) {
            push_unique(&mut redacted, field);
        }
    }
    redacted.sort();
    redacted
}

fn decision_reason(source: &TrustCenterSourceEvidence) -> String {
    match &source.health {
        SourceHealth::Current | SourceHealth::AgingWarning => {
            format!("source {} normalized and redacted", source.source_id)
        }
        SourceHealth::Stale => "blocked because source evidence is stale".to_owned(),
        SourceHealth::Missing => "blocked because source evidence is missing".to_owned(),
        SourceHealth::ParserError => "blocked because source parser reported an error".to_owned(),
        SourceHealth::ExpiredException => "blocked because exception is expired".to_owned(),
        SourceHealth::UnknownApplicability => "blocked because applicability is unknown".to_owned(),
        SourceHealth::NotApplicableWithPolicyReason(reason) => {
            format!(
                "not applicable with policy reason: {}",
                safe_policy_reason(reason)
            )
        }
        SourceHealth::BlockedPendingReview => "blocked pending security/privacy review".to_owned(),
    }
}

fn compliance_pack_ids(source: &TrustCenterSourceEvidence) -> Vec<String> {
    source
        .fields
        .get("pack_id")
        .cloned()
        .or_else(|| Some("pack_soc2_ready".to_owned()))
        .into_iter()
        .collect()
}

fn service_ids(source: &TrustCenterSourceEvidence) -> Vec<String> {
    for key in ["subject_ref", "service_id"] {
        if let Some(value) = source.fields.get(key) {
            return vec![value.clone()];
        }
    }
    vec!["svc_trust_center".to_owned()]
}

fn field<'a>(
    source: &'a TrustCenterSourceEvidence,
    field: &'static str,
) -> Result<&'a str, TrustCenterIngestError> {
    source.fields.get(field).map(String::as_str).ok_or(
        TrustCenterIngestError::MissingMinimumField {
            family: source.family,
            source_id: source.source_id.clone(),
            field,
        },
    )
}

fn split_refs(value: &str) -> Vec<String> {
    value
        .split([',', ';'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_owned)
        .collect()
}

fn parse_retention_days(value: &str) -> u32 {
    let digits = value
        .chars()
        .filter(char::is_ascii_digit)
        .collect::<String>();
    digits.parse::<u32>().unwrap_or(400)
}

fn record_id(source: &TrustCenterSourceEvidence, sequence: usize, prefix: &str) -> String {
    format!(
        "{}_{}_{}",
        prefix,
        sequence,
        slug(&format!("{}_{}", source.family.as_str(), source.source_id))
    )
}

fn source_record_ref(source: &TrustCenterSourceEvidence) -> String {
    format!(
        "{TRUST_CENTER_INGEST_SOURCE_REF_PREFIX}{}",
        source.source_id
    )
}

fn safe_policy_reason(reason: &str) -> String {
    let mut redacted_detail = false;
    let safe_tokens = reason
        .chars()
        .filter(|ch| {
            ch.is_ascii_alphanumeric() || ch.is_ascii_whitespace() || ['-', '_', ':'].contains(ch)
        })
        .collect::<String>()
        .split_whitespace()
        .filter_map(|token| {
            let lower = token.to_ascii_lowercase();
            let sensitive = [
                "secret", "token", "password", "passwd", "api_key", "apikey", "private", "exploit",
                "payload", "pii", "ten_",
            ]
            .iter()
            .any(|needle| lower.contains(needle));
            if sensitive {
                if redacted_detail {
                    None
                } else {
                    redacted_detail = true;
                    Some("redacted_policy_detail".to_owned())
                }
            } else {
                Some(token.to_owned())
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    if safe_tokens.is_empty() {
        "policy_reason_recorded".to_owned()
    } else {
        safe_tokens
    }
}

fn push_unique(fields: &mut Vec<String>, field: &str) {
    if !fields.iter().any(|existing| existing == field) {
        fields.push(field.to_owned());
    }
}

fn slug(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.ends_with('_') {
            out.push('_');
        }
    }
    out.trim_matches('_').to_owned()
}
