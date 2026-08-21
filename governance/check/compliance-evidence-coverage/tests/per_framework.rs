#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use check_compliance_evidence_coverage::{MicroserviceEvidenceInput, check};
use oya_shared_compliance_evidence_kernel::{
    ComplianceFramework, EvidenceArtifact, EvidenceArtifactKind, required_artifacts_for,
};

fn seal() -> String {
    "d".repeat(64)
}

#[test]
fn pci_scope_yields_four_gaps_when_empty() {
    let input = MicroserviceEvidenceInput {
        microservice: "payments".into(),
        tenant_id: "tenant_x".into(),
        frameworks: vec![ComplianceFramework::PciDss],
        artifacts: vec![],
        window_open_unix_ms: 0,
    };
    let r = check(&[input]).unwrap();
    assert_eq!(r.gaps.len(), 4);
}

#[test]
fn hipaa_minimum_necessary_log_gap_is_present_when_absent() {
    let input = MicroserviceEvidenceInput {
        microservice: "healthcare-portal".into(),
        tenant_id: "tenant_x".into(),
        frameworks: vec![ComplianceFramework::Hipaa],
        artifacts: vec![],
        window_open_unix_ms: 0,
    };
    let r = check(&[input]).unwrap();
    assert!(
        r.gaps
            .iter()
            .any(|g| g.missing_artifact_label == "minimum-necessary-access-log")
    );
    assert!(
        r.gaps
            .iter()
            .any(|g| g.missing_artifact_label == "baa-inventory-entry")
    );
}

#[test]
fn covered_then_gapped_per_microservice_count() {
    let mut covered = Vec::new();
    for k in required_artifacts_for(ComplianceFramework::PciDss) {
        covered.push(
            EvidenceArtifact::new(
                format!("a_{}", k.wire_label()),
                k,
                ComplianceFramework::PciDss,
                seal(),
                100,
                "tenant_x".into(),
            )
            .unwrap(),
        );
    }
    let input_a = MicroserviceEvidenceInput {
        microservice: "ms-a".into(),
        tenant_id: "tenant_x".into(),
        frameworks: vec![ComplianceFramework::PciDss],
        artifacts: covered,
        window_open_unix_ms: 0,
    };
    let input_b = MicroserviceEvidenceInput {
        microservice: "ms-b".into(),
        tenant_id: "tenant_x".into(),
        frameworks: vec![ComplianceFramework::PciDss],
        artifacts: vec![],
        window_open_unix_ms: 0,
    };
    let r = check(&[input_a, input_b]).unwrap();
    assert_eq!(r.microservices_checked, 2);
    let ms_a_gaps = r.gaps.iter().filter(|g| g.microservice == "ms-a").count();
    let ms_b_gaps = r.gaps.iter().filter(|g| g.microservice == "ms-b").count();
    assert_eq!(ms_a_gaps, 0);
    assert_eq!(ms_b_gaps, 4);
}

#[test]
fn cross_tenant_artifact_does_not_close_gap() {
    let artifact = EvidenceArtifact::new(
        "a_other".into(),
        EvidenceArtifactKind::DsarCompletionRecord,
        ComplianceFramework::Gdpr,
        seal(),
        100,
        "other_tenant".into(),
    )
    .unwrap();
    let input = MicroserviceEvidenceInput {
        microservice: "identity".into(),
        tenant_id: "tenant_x".into(),
        frameworks: vec![ComplianceFramework::Gdpr],
        artifacts: vec![artifact],
        window_open_unix_ms: 0,
    };
    let r = check(&[input]).unwrap();
    assert!(
        r.gaps
            .iter()
            .any(|g| g.missing_artifact_label == "dsar-completion-record")
    );
}

#[test]
fn empty_inputs_yields_empty_report() {
    let r = check(&[]).unwrap();
    assert_eq!(r.microservices_checked, 0);
    assert!(r.gaps.is_empty());
}
