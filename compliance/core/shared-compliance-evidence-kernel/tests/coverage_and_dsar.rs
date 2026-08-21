#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use shared_compliance_evidence_kernel::{
    ComplianceFramework, DsarRequest, EvidenceArtifact, EvidenceArtifactKind, coverage_gaps,
    required_artifacts_for,
};

fn seal() -> String {
    "b".repeat(64)
}

#[test]
fn hipaa_requires_minimum_necessary_log() {
    let req = required_artifacts_for(ComplianceFramework::Hipaa);
    assert!(req.contains(&EvidenceArtifactKind::MinimumNecessaryAccessLog));
    assert!(req.contains(&EvidenceArtifactKind::BaaInventoryEntry));
}

#[test]
fn pci_requires_vuln_scan_and_pen_test() {
    let req = required_artifacts_for(ComplianceFramework::PciDss);
    assert!(req.contains(&EvidenceArtifactKind::VulnScanReport));
    assert!(req.contains(&EvidenceArtifactKind::PenTestReport));
}

#[test]
fn gdpr_requires_dsar_completion_record() {
    let req = required_artifacts_for(ComplianceFramework::Gdpr);
    assert!(req.contains(&EvidenceArtifactKind::DsarCompletionRecord));
}

#[test]
fn fully_covered_tenant_has_zero_gaps() {
    let mut artifacts = Vec::new();
    for kind in required_artifacts_for(ComplianceFramework::Soc2TypeII) {
        artifacts.push(
            EvidenceArtifact::new(
                format!("a_{}", kind.wire_label()),
                kind,
                ComplianceFramework::Soc2TypeII,
                seal(),
                100,
                "tenant_x".into(),
            )
            .unwrap(),
        );
    }
    let gaps = coverage_gaps(
        "tenant_x",
        &artifacts,
        &[ComplianceFramework::Soc2TypeII],
        0,
    )
    .unwrap();
    assert_eq!(gaps.len(), 0);
}

#[test]
fn dsar_5_day_target_under_sla() {
    let day_ms = 1_000u64 * 60 * 60 * 24;
    let req = DsarRequest {
        request_id: "dsar_t1".into(),
        tenant_id: "tenant_x".into(),
        subject_id_pseudonym: "subj".into(),
        opened_unix_ms: 0,
        closed_unix_ms: Some(4 * day_ms),
    };
    assert_eq!(req.elapsed_days(40 * day_ms), 4);
    assert!(req.elapsed_days(40 * day_ms) < DsarRequest::TARGET_DAYS);
}
