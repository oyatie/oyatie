//! ADR-0209 compliance evidence coverage gate.
//!
//! Advisory lane that audits per-µservice evidence emission against
//! the required artifact set declared in
//! `shared_compliance_evidence_kernel::required_artifacts_for`.
//!
//! The kernel emits a `CoverageReport` listing gaps per (microservice,
//! tenant, framework) tuple. Mode is advisory; downstream gate runner
//! decides whether to fail-closed (production rollout) or warn-only
//! (dev/staging).
//!
//! Pure model; no I/O.
//!
//! ADR-0083 Tier 3 test exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use shared_compliance_evidence_kernel::{
    ComplianceError, ComplianceFramework, CoverageGap, EvidenceArtifact, coverage_gaps,
};

#[derive(Clone, Debug)]
pub struct MicroserviceEvidenceInput {
    pub microservice: String,
    pub tenant_id: String,
    pub frameworks: Vec<ComplianceFramework>,
    pub artifacts: Vec<EvidenceArtifact>,
    pub window_open_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceCoverageGap {
    pub microservice: String,
    pub tenant_id: String,
    pub framework: ComplianceFramework,
    pub missing_artifact_label: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverageReport {
    pub microservices_checked: usize,
    pub gaps: Vec<EvidenceCoverageGap>,
}

pub fn check(inputs: &[MicroserviceEvidenceInput]) -> Result<CoverageReport, ComplianceError> {
    let mut out_gaps = Vec::new();
    for input in inputs {
        let gaps: Vec<CoverageGap> = coverage_gaps(
            &input.tenant_id,
            &input.artifacts,
            &input.frameworks,
            input.window_open_unix_ms,
        )?;
        for g in gaps {
            out_gaps.push(EvidenceCoverageGap {
                microservice: input.microservice.clone(),
                tenant_id: input.tenant_id.clone(),
                framework: g.framework,
                missing_artifact_label: g.missing.wire_label().to_owned(),
            });
        }
    }
    Ok(CoverageReport {
        microservices_checked: inputs.len(),
        gaps: out_gaps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_compliance_evidence_kernel::{
        ComplianceFramework, EvidenceArtifactKind, required_artifacts_for,
    };

    fn seal() -> String {
        "c".repeat(64)
    }

    fn covered_inputs(ms: &str, tenant: &str) -> MicroserviceEvidenceInput {
        let mut artifacts = Vec::new();
        for k in required_artifacts_for(ComplianceFramework::Soc2TypeII) {
            artifacts.push(
                EvidenceArtifact::new(
                    format!("a_{}", k.wire_label()),
                    k,
                    ComplianceFramework::Soc2TypeII,
                    seal(),
                    100,
                    tenant.into(),
                )
                .unwrap(),
            );
        }
        MicroserviceEvidenceInput {
            microservice: ms.into(),
            tenant_id: tenant.into(),
            frameworks: vec![ComplianceFramework::Soc2TypeII],
            artifacts,
            window_open_unix_ms: 0,
        }
    }

    #[test]
    fn fully_covered_microservice_yields_no_gaps() {
        let r = check(&[covered_inputs("identity", "tenant_x")]).unwrap();
        assert_eq!(r.microservices_checked, 1);
        assert!(r.gaps.is_empty());
    }

    #[test]
    fn partial_evidence_emits_gaps_per_required_kind() {
        let input = MicroserviceEvidenceInput {
            microservice: "identity".into(),
            tenant_id: "tenant_x".into(),
            frameworks: vec![ComplianceFramework::Gdpr],
            artifacts: vec![],
            window_open_unix_ms: 0,
        };
        let r = check(&[input]).unwrap();
        // GDPR requires 3 artifacts; none provided.
        assert_eq!(r.gaps.len(), 3);
        assert!(
            r.gaps
                .iter()
                .all(|g| g.framework == ComplianceFramework::Gdpr)
        );
    }

    #[test]
    fn empty_tenant_id_rejected() {
        let input = MicroserviceEvidenceInput {
            microservice: "identity".into(),
            tenant_id: String::new(),
            frameworks: vec![ComplianceFramework::Gdpr],
            artifacts: vec![],
            window_open_unix_ms: 0,
        };
        assert!(matches!(
            check(&[input]),
            Err(ComplianceError::EmptyTenantId)
        ));
    }

    #[test]
    fn multi_framework_gaps_carry_correct_framework_tag() {
        let input = MicroserviceEvidenceInput {
            microservice: "ops-portal".into(),
            tenant_id: "tenant_x".into(),
            frameworks: vec![ComplianceFramework::Gdpr, ComplianceFramework::Hipaa],
            artifacts: vec![],
            window_open_unix_ms: 0,
        };
        let r = check(&[input]).unwrap();
        let gdpr_gaps = r
            .gaps
            .iter()
            .filter(|g| g.framework == ComplianceFramework::Gdpr)
            .count();
        let hipaa_gaps = r
            .gaps
            .iter()
            .filter(|g| g.framework == ComplianceFramework::Hipaa)
            .count();
        assert_eq!(gdpr_gaps, 3);
        assert_eq!(hipaa_gaps, 4);
    }

    #[test]
    fn stale_artifact_excluded_by_window() {
        let stale = EvidenceArtifact::new(
            "a_ci".into(),
            EvidenceArtifactKind::CiArtifactHash,
            ComplianceFramework::Soc2TypeII,
            seal(),
            10,
            "tenant_x".into(),
        )
        .unwrap();
        let input = MicroserviceEvidenceInput {
            microservice: "identity".into(),
            tenant_id: "tenant_x".into(),
            frameworks: vec![ComplianceFramework::Soc2TypeII],
            artifacts: vec![stale],
            window_open_unix_ms: 1_000, // newer than the artifact
        };
        let r = check(&[input]).unwrap();
        // The stale CI artifact gap is still listed because the window
        // closed it out.
        assert!(
            r.gaps
                .iter()
                .any(|g| g.missing_artifact_label == "ci-artifact-hash")
        );
    }
}
