// ADR-0083 Tier 3: integration tests use unwrap/expect for invariant clarity.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};

use oya_trust_center_api::{
    TRUST_CENTER_COMPLIANCE_PACK_RECORD_TYPE, TRUST_CENTER_CONTROL_FRESHNESS_RECORD_TYPE,
    TRUST_CENTER_EVIDENCE_ITEM_RECORD_TYPE, TRUST_CENTER_PUBLISHABILITY_DECISION_RECORD_TYPE,
    TRUST_CENTER_SBOM_VEX_RECORD_TYPE, TrustCenterDataClass, TrustCenterFreshnessState,
    TrustCenterPublishabilityState,
};
use oya_trust_center_ingest::*;

const TENANT: &str = "ten_alpha";
const OTHER_TENANT: &str = "ten_beta";
const NOW: &str = "2026-07-01T09:45:00Z";
const RETAIN: &str = "2027-07-01T09:45:00Z";

#[test]
fn fixture_replay_covers_every_trust_center_source_family_and_record_shape() {
    let batch = ingest_trust_center_sources(&fixture_replay()).expect("fixture replay ingests");

    assert_eq!(batch.evidence_items.len(), 6);
    assert_eq!(batch.control_freshness.len(), 1);
    assert_eq!(batch.sbom_vex.len(), 1);
    assert_eq!(batch.compliance_packs.len(), 1);
    assert_eq!(batch.publishability_decisions.len(), 6);

    let source_families = batch
        .evidence_items
        .iter()
        .map(|record| record.common.source_system.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        source_families,
        BTreeSet::from([
            "cloud_quality_kits",
            "compliance_pack_activation",
            "release_and_audit_chain",
            "sbom_vex_vulnerability_posture",
            "security_validation_controls",
            "slo_dr_status_incident",
        ])
    );

    for item in &batch.evidence_items {
        assert_eq!(
            item.common.record_type,
            TRUST_CENTER_EVIDENCE_ITEM_RECORD_TYPE
        );
        assert_eq!(item.common.schema_version, 1);
        assert_eq!(item.common.tenant_id, TENANT);
        assert!(!item.common.source_record_ref.trim().is_empty());
        assert!(!item.common.audit_event_ref.trim().is_empty());
        assert!(!item.common.redaction_policy_id.trim().is_empty());
        assert!(!item.operator_only_detail_present);
        assert!(!item.raw_operator_payload_exposed);
    }
    assert_eq!(
        batch.control_freshness[0].common.record_type,
        TRUST_CENTER_CONTROL_FRESHNESS_RECORD_TYPE
    );
    assert_eq!(
        batch.sbom_vex[0].common.record_type,
        TRUST_CENTER_SBOM_VEX_RECORD_TYPE
    );
    assert_eq!(
        batch.compliance_packs[0].common.record_type,
        TRUST_CENTER_COMPLIANCE_PACK_RECORD_TYPE
    );
    assert!(batch.sbom_vex[0].signed_sbom_ref.is_some());
    assert!(batch.sbom_vex[0].vex_ref.is_some());
}

#[test]
fn redaction_removes_operator_only_raw_detail_from_customer_safe_records() {
    let batch = ingest_trust_center_sources(&[source(
        TrustCenterEvidenceSourceFamily::SecurityValidationControls,
        "sec-redaction",
        SourceHealth::Current,
    )
    .with_customer_summary(
        "Scanner found synthetic_secret_marker, synthetic_pii_marker, synthetic_exploit_payload, tenant ten_beta impacted",
    )
    .with_operator_detail("raw_scanner_output", "synthetic scanner raw output")
    .with_operator_detail("pii", "synthetic_pii_marker")
    .with_operator_detail("exploit_payload", "synthetic_exploit_payload")
    .with_operator_detail("cross_tenant_identifier", OTHER_TENANT)])
    .expect("redaction source ingests");

    let item = &batch.evidence_items[0];
    let serialized = serde_json::to_string(item).expect("record serializes");
    for forbidden in [
        "synthetic_secret_marker",
        "synthetic_pii_marker",
        "synthetic_exploit_payload",
        "synthetic scanner raw output",
        OTHER_TENANT,
    ] {
        assert!(
            !serialized.contains(forbidden),
            "customer-safe payload leaked forbidden detail: {forbidden}"
        );
    }
    assert!(
        item.redacted_fields
            .contains(&"raw_scanner_output".to_string())
    );
    assert!(item.redacted_fields.contains(&"pii".to_string()));
    assert!(item.redacted_fields.contains(&"secret".to_string()));
    assert!(
        item.redacted_fields
            .contains(&"exploit_payload".to_string())
    );
    assert!(
        item.redacted_fields
            .contains(&"cross_tenant_identifier".to_string())
    );
}

#[test]
fn freshness_and_exception_states_fail_closed_or_render_policy_na() {
    let batch = ingest_trust_center_sources(&[
        source(
            TrustCenterEvidenceSourceFamily::SecurityValidationControls,
            "current",
            SourceHealth::Current,
        ),
        source(
            TrustCenterEvidenceSourceFamily::CloudQualityKits,
            "warning",
            SourceHealth::AgingWarning,
        ),
        source(
            TrustCenterEvidenceSourceFamily::ReleaseAndAuditChain,
            "stale",
            SourceHealth::Stale,
        ),
        source(
            TrustCenterEvidenceSourceFamily::SbomVexVulnerabilityPosture,
            "missing",
            SourceHealth::Missing,
        ),
        source(
            TrustCenterEvidenceSourceFamily::CompliancePackActivation,
            "parser",
            SourceHealth::ParserError,
        ),
        source(
            TrustCenterEvidenceSourceFamily::SloDrStatusIncident,
            "expired-exception",
            SourceHealth::ExpiredException,
        ),
        source(
            TrustCenterEvidenceSourceFamily::SecurityValidationControls,
            "unknown-applicability",
            SourceHealth::UnknownApplicability,
        ),
        source(
            TrustCenterEvidenceSourceFamily::CompliancePackActivation,
            "na-with-policy",
            SourceHealth::NotApplicableWithPolicyReason(
                "pack not activated for tenant".to_string(),
            ),
        ),
        source(
            TrustCenterEvidenceSourceFamily::CloudQualityKits,
            "blocked-review",
            SourceHealth::BlockedPendingReview,
        ),
    ])
    .expect("freshness replay ingests");

    let by_source_ref = batch
        .evidence_items
        .iter()
        .map(|record| {
            (
                record.common.source_record_ref.as_str(),
                (
                    record.common.freshness_state,
                    record.common.publishability_state,
                    record.common.customer_safe_state_for_assertion(),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();

    assert_eq!(
        by_source_ref["trust-source://current"].0,
        TrustCenterFreshnessState::Current
    );
    assert_eq!(
        by_source_ref["trust-source://warning"].0,
        TrustCenterFreshnessState::AgingWarning
    );
    for source_ref in [
        "trust-source://stale",
        "trust-source://missing",
        "trust-source://parser",
        "trust-source://expired-exception",
        "trust-source://unknown-applicability",
    ] {
        let (_, publishability, summary) = by_source_ref[source_ref];
        assert!(matches!(
            publishability,
            TrustCenterPublishabilityState::BlockedMissingEvidence
                | TrustCenterPublishabilityState::BlockedStaleEvidence
                | TrustCenterPublishabilityState::BlockedSecurityPrivacyReview
        ));
        assert!(
            summary.starts_with("blocked_"),
            "{source_ref} did not fail closed"
        );
    }
    assert_eq!(
        by_source_ref["trust-source://na-with-policy"].0,
        TrustCenterFreshnessState::NotApplicableWithPolicyReason
    );
    assert_eq!(
        by_source_ref["trust-source://na-with-policy"].1,
        TrustCenterPublishabilityState::NotApplicableWithPolicyReason
    );
    let policy_na = ingest_trust_center_sources(&[source(
        TrustCenterEvidenceSourceFamily::CompliancePackActivation,
        "na-sensitive-policy-reason",
        SourceHealth::NotApplicableWithPolicyReason(
            "pack not activated for tenant ten_beta synthetic_secret_marker user@example.com exploit_payload"
                .to_string(),
        ),
    )])
    .expect("policy N/A with sensitive operator wording ingests");
    let serialized = serde_json::to_string(&(
        &policy_na.evidence_items,
        &policy_na.publishability_decisions,
    ))
    .expect("policy N/A serializes");
    for forbidden in [
        "ten_beta",
        "synthetic_secret_marker",
        "user@example.com",
        "exploit_payload",
    ] {
        assert!(
            !serialized.contains(forbidden),
            "policy N/A leaked sensitive operator wording: {forbidden}"
        );
    }
    assert!(serialized.contains("redacted_policy_detail"));
    assert_eq!(
        by_source_ref["trust-source://blocked-review"].0,
        TrustCenterFreshnessState::BlockedPendingReview
    );
}

#[test]
fn missing_minimum_fields_and_cross_tenant_source_assertions_are_rejected() {
    let missing = ingest_trust_center_sources(&[source(
        TrustCenterEvidenceSourceFamily::SecurityValidationControls,
        "missing-field",
        SourceHealth::Current,
    )
    .without_minimum_field("lane_id")])
    .expect_err("security validation source without lane_id must fail closed");
    assert!(matches!(
        missing,
        TrustCenterIngestError::MissingMinimumField {
            family: TrustCenterEvidenceSourceFamily::SecurityValidationControls,
            field: "lane_id",
            ..
        }
    ));

    let missing_tenant_assertion = ingest_trust_center_sources(&[source(
        TrustCenterEvidenceSourceFamily::SecurityValidationControls,
        "missing-tenant-assertion",
        SourceHealth::Current,
    )
    .without_minimum_field("tenant_id")])
    .expect_err("security validation source without tenant_id assertion must fail closed");
    assert!(matches!(
        missing_tenant_assertion,
        TrustCenterIngestError::MissingMinimumField {
            family: TrustCenterEvidenceSourceFamily::SecurityValidationControls,
            field: "tenant_id",
            ..
        }
    ));

    let cross_tenant = ingest_trust_center_sources(&[source(
        TrustCenterEvidenceSourceFamily::SecurityValidationControls,
        "cross-tenant",
        SourceHealth::Current,
    )
    .with_field("tenant_id", OTHER_TENANT)])
    .expect_err("source tenant assertion mismatch must fail closed");
    assert!(matches!(
        cross_tenant,
        TrustCenterIngestError::TenantAssertionMismatch { .. }
    ));
}

#[test]
fn publishability_decisions_are_append_only_auditable_records() {
    let batch = ingest_trust_center_sources(&[
        source(
            TrustCenterEvidenceSourceFamily::SecurityValidationControls,
            "decision-a",
            SourceHealth::Current,
        ),
        source(
            TrustCenterEvidenceSourceFamily::SecurityValidationControls,
            "decision-a",
            SourceHealth::Stale,
        ),
    ])
    .expect("decision replay ingests");

    assert_eq!(batch.publishability_decisions.len(), 2);
    let ids = batch
        .publishability_decisions
        .iter()
        .map(|record| record.decision_id.as_str())
        .collect::<Vec<_>>();
    assert_ne!(
        ids[0], ids[1],
        "decisions must append, not overwrite by evidence id"
    );
    for decision in &batch.publishability_decisions {
        assert_eq!(
            decision.common.record_type,
            TRUST_CENTER_PUBLISHABILITY_DECISION_RECORD_TYPE
        );
        assert_eq!(decision.common.tenant_id, TENANT);
        assert!(
            decision
                .common
                .audit_event_ref
                .starts_with("audit/trust_center_publishability/")
        );
        assert_eq!(
            decision.decided_by_principal_id,
            "principal_oyatie_operator_policy"
        );
    }
    assert_eq!(
        batch.publishability_decisions[1].new_state,
        TrustCenterPublishabilityState::BlockedStaleEvidence
    );
}

trait AssertionState {
    fn customer_safe_state_for_assertion(&self) -> &'static str;
}

impl AssertionState for oya_trust_center_api::TrustCenterCommonFields {
    fn customer_safe_state_for_assertion(&self) -> &'static str {
        match self.publishability_state {
            TrustCenterPublishabilityState::PublishableCustomerSafe => "publishable_customer_safe",
            TrustCenterPublishabilityState::PublishableSummaryOnly => "publishable_summary_only",
            TrustCenterPublishabilityState::TenantAdminOnly => "tenant_admin_only",
            TrustCenterPublishabilityState::OperatorOnly => "blocked_operator_only",
            TrustCenterPublishabilityState::BlockedMissingEvidence => "blocked_missing_evidence",
            TrustCenterPublishabilityState::BlockedStaleEvidence => "blocked_stale_evidence",
            TrustCenterPublishabilityState::BlockedSecurityPrivacyReview => {
                "blocked_security_privacy_review"
            }
            TrustCenterPublishabilityState::NotApplicableWithPolicyReason => {
                "not_applicable_with_policy_reason"
            }
        }
    }
}

fn fixture_replay() -> Vec<TrustCenterSourceEvidence> {
    vec![
        source(
            TrustCenterEvidenceSourceFamily::SecurityValidationControls,
            "security-validation-controls",
            SourceHealth::Current,
        ),
        source(
            TrustCenterEvidenceSourceFamily::SbomVexVulnerabilityPosture,
            "sbom-vex-vulnerability-posture",
            SourceHealth::Current,
        ),
        source(
            TrustCenterEvidenceSourceFamily::CompliancePackActivation,
            "compliance-pack-activation",
            SourceHealth::Current,
        ),
        source(
            TrustCenterEvidenceSourceFamily::SloDrStatusIncident,
            "slo-dr-status-incident",
            SourceHealth::Current,
        ),
        source(
            TrustCenterEvidenceSourceFamily::CloudQualityKits,
            "cloud-quality-kits",
            SourceHealth::Current,
        ),
        source(
            TrustCenterEvidenceSourceFamily::ReleaseAndAuditChain,
            "release-and-audit-chain",
            SourceHealth::Current,
        ),
    ]
}

fn source(
    family: TrustCenterEvidenceSourceFamily,
    source_id: &str,
    health: SourceHealth,
) -> TrustCenterSourceEvidence {
    let mut source = TrustCenterSourceEvidence::new(family, TENANT, source_id, NOW, RETAIN, health);
    for (field, value) in required_fields_for(family) {
        source = source.with_field(field, value);
    }
    source
        .with_customer_summary("Customer-safe Trust Center evidence summary")
        .with_audit_event_ref(&format!("audit/source/{source_id}"))
}

fn required_fields_for(
    family: TrustCenterEvidenceSourceFamily,
) -> Vec<(&'static str, &'static str)> {
    match family {
        TrustCenterEvidenceSourceFamily::SecurityValidationControls => vec![
            ("tenant_id", TENANT),
            ("lane_id", "security_validation_controls"),
            ("subject_ref", "svc_trust_center"),
            ("result", "pass"),
            ("severity_summary", "no critical findings"),
            ("exception_refs", "vex_exception_1"),
            ("audit_event_ref", "audit/source/security"),
            ("retention_until", RETAIN),
        ],
        TrustCenterEvidenceSourceFamily::SbomVexVulnerabilityPosture => vec![
            ("tenant_id_or_internal_lane", TENANT),
            ("artifact_digest", "sha256:abc"),
            ("component_purl", "pkg:cargo/oya-trust-center-api"),
            ("vulnerability_id", "CVE-2026-0001"),
            ("sbom_refs", "sbom://trust-center-api/sha256-abc"),
            ("vex_ref", "vex://trust-center-api/sha256-abc"),
            ("priority_signals", "kev=false"),
            ("exception_ref", "vex_exception_1"),
            ("verdict", "not_affected"),
        ],
        TrustCenterEvidenceSourceFamily::CompliancePackActivation => vec![
            ("pack_id", "pack_soc2_ready"),
            ("version", "2026.07"),
            ("signed_by", "compliance-owner"),
            ("audit_chain_requirements", "audit-chain-required"),
            ("data_class_extensions", "TENANT_TRUST_EVIDENCE"),
            ("cell_eligibility", "home-region"),
            ("retention_rules", "P400D"),
            ("regulator_references", "SOC2-readiness-non-certification"),
        ],
        TrustCenterEvidenceSourceFamily::SloDrStatusIncident => vec![
            ("service_id", "svc_trust_center"),
            ("tenant_or_pack_scope", TENANT),
            ("slo_state", "healthy"),
            ("dr_floor_ref", "dr-floor://soc2-ready"),
            ("incident_ref", "status://none-open"),
            ("postmortem_ref", "postmortem://none-required"),
            ("audit_event_ref", "audit/source/status"),
        ],
        TrustCenterEvidenceSourceFamily::CloudQualityKits => vec![
            ("kit_id", "privacy-data-governance"),
            ("scenario_id", "privacy-qk-01"),
            ("evidence_ref", "quality-kit://privacy-qk-01"),
            ("gate", "privacy_data_governance"),
            ("freshness_state", "current"),
            ("non_claim_state", "target_non_claim"),
        ],
        TrustCenterEvidenceSourceFamily::ReleaseAndAuditChain => vec![
            ("change_id", "change-trust-center-api"),
            ("claim_tier", "spec_ready"),
            ("evidence_packet_ref", "evidence://release/trust-center-api"),
            ("audit_event_ref", "audit/source/release"),
            (
                "release_note_or_governance_impact",
                "release governance impact recorded; no certification claim",
            ),
        ],
    }
}
