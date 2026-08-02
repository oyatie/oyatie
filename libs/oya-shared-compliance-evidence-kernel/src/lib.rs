//! Shared compliance evidence kernel — ADR-0209 substrate.
//!
//! Hyperscaler-grade evidence pipeline replacing commercial Drata /
//! Vanta. The kernel models:
//!
//! - **Evidence frameworks** — SOC 2 Type II, GDPR DSAR, HIPAA,
//!   PCI-DSS — as a closed enum.
//! - **Evidence artifact kinds** — CI artifact hash, deploy receipt,
//!   access-review snapshot, backup-drill receipt, vuln-scan report,
//!   pen-test report, DSAR completion record, BAA inventory entry,
//!   minimum-necessary access log — closed enum.
//! - **Coverage matrix** — per (framework × required artifact kind)
//!   the kernel tracks whether evidence has been emitted in the
//!   active window.
//! - **Audit-chain seal compliance** — evidence collectors MUST emit
//!   an audit-chain seal hash (per ADR-0145 + Bominal ADR-0028); the
//!   kernel validates the seal hash is hex-shaped (the real cosign /
//!   sigstore verification is an adapter concern).
//!
//! NO I/O. Adapters bring SeaweedFS storage + the ADR-0394 first-party portal
//! reader access.
//!
//! ADR-0083 Tier 3 test exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ComplianceFramework {
    Soc2TypeII,
    Gdpr,
    Hipaa,
    PciDss,
}

impl ComplianceFramework {
    pub const fn wire_label(self) -> &'static str {
        match self {
            Self::Soc2TypeII => "soc2-type-2",
            Self::Gdpr => "gdpr",
            Self::Hipaa => "hipaa",
            Self::PciDss => "pci-dss",
        }
    }
    pub const fn all() -> [Self; 4] {
        [Self::Soc2TypeII, Self::Gdpr, Self::Hipaa, Self::PciDss]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum EvidenceArtifactKind {
    CiArtifactHash,
    DeployReceipt,
    AccessReviewSnapshot,
    BackupRestoreDrillReceipt,
    VulnScanReport,
    PenTestReport,
    DsarCompletionRecord,
    BaaInventoryEntry,
    MinimumNecessaryAccessLog,
}

impl EvidenceArtifactKind {
    pub const fn wire_label(self) -> &'static str {
        match self {
            Self::CiArtifactHash => "ci-artifact-hash",
            Self::DeployReceipt => "deploy-receipt",
            Self::AccessReviewSnapshot => "access-review-snapshot",
            Self::BackupRestoreDrillReceipt => "backup-restore-drill-receipt",
            Self::VulnScanReport => "vuln-scan-report",
            Self::PenTestReport => "pen-test-report",
            Self::DsarCompletionRecord => "dsar-completion-record",
            Self::BaaInventoryEntry => "baa-inventory-entry",
            Self::MinimumNecessaryAccessLog => "minimum-necessary-access-log",
        }
    }
}

/// Closed required-artifact matrix per framework — the kernel's
/// authoritative declaration of "what evidence must be in place for
/// auditor X to certify framework Y."
pub fn required_artifacts_for(framework: ComplianceFramework) -> Vec<EvidenceArtifactKind> {
    use EvidenceArtifactKind::*;
    match framework {
        ComplianceFramework::Soc2TypeII => vec![
            CiArtifactHash,
            DeployReceipt,
            AccessReviewSnapshot,
            BackupRestoreDrillReceipt,
            VulnScanReport,
            PenTestReport,
        ],
        ComplianceFramework::Gdpr => {
            vec![DsarCompletionRecord, AccessReviewSnapshot, VulnScanReport]
        }
        ComplianceFramework::Hipaa => vec![
            MinimumNecessaryAccessLog,
            BaaInventoryEntry,
            AccessReviewSnapshot,
            BackupRestoreDrillReceipt,
        ],
        ComplianceFramework::PciDss => vec![
            VulnScanReport,
            PenTestReport,
            AccessReviewSnapshot,
            DeployReceipt,
        ],
    }
}

/// A single emitted evidence artifact — payload-free; the kernel
/// tracks identity + seal hash + emit time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceArtifact {
    pub artifact_id: String,            // data_class: INTERNAL_ONLY
    pub kind: EvidenceArtifactKind,     // data_class: INTERNAL_ONLY
    pub framework: ComplianceFramework, // data_class: INTERNAL_ONLY
    pub audit_chain_seal_hex: String,   // data_class: INTERNAL_ONLY
    pub emitted_unix_ms: u64,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
}

impl EvidenceArtifact {
    pub fn new(
        artifact_id: String,
        kind: EvidenceArtifactKind,
        framework: ComplianceFramework,
        audit_chain_seal_hex: String,
        emitted_unix_ms: u64,
        tenant_id: String,
    ) -> Result<Self, ComplianceError> {
        if artifact_id.is_empty() {
            return Err(ComplianceError::EmptyArtifactId);
        }
        if tenant_id.is_empty() {
            return Err(ComplianceError::EmptyTenantId);
        }
        validate_seal(&audit_chain_seal_hex)?;
        Ok(Self {
            artifact_id,
            kind,
            framework,
            audit_chain_seal_hex,
            emitted_unix_ms,
            tenant_id,
        })
    }
}

/// Evidence collector trait — adapters implement (Trivy, Cedar
/// snapshot, SeaweedFS-backed deploy receipt fetcher, etc.).
pub trait EvidenceCollector {
    fn kind(&self) -> EvidenceArtifactKind;
    fn framework_coverage(&self) -> Vec<ComplianceFramework>;
    /// Collect for a given tenant. Returns the artifact identifier and
    /// audit-chain seal hash (hex). The wire-level fetch is an adapter
    /// concern.
    fn collect(
        &self,
        tenant_id: &str,
        now_unix_ms: u64,
    ) -> Result<EvidenceArtifact, ComplianceError>;
}

/// Coverage gap — used by the gate to drive
/// `evidence/parent-wiring-todo-frontend-batch.json` and ADR-0209's
/// rollout schedule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverageGap {
    pub framework: ComplianceFramework, // data_class: INTERNAL_ONLY
    pub missing: EvidenceArtifactKind,  // data_class: INTERNAL_ONLY
}

/// Compute per-tenant gaps across required artifacts in the active
/// window. `window_open_unix_ms` is the lower-bound emit time below
/// which an artifact no longer counts as fresh.
pub fn coverage_gaps(
    tenant_id: &str,
    artifacts: &[EvidenceArtifact],
    frameworks: &[ComplianceFramework],
    window_open_unix_ms: u64,
) -> Result<Vec<CoverageGap>, ComplianceError> {
    if tenant_id.is_empty() {
        return Err(ComplianceError::EmptyTenantId);
    }
    let mut present: BTreeMap<ComplianceFramework, BTreeSet<EvidenceArtifactKind>> =
        BTreeMap::new();
    for a in artifacts {
        if a.tenant_id != tenant_id {
            continue;
        }
        if a.emitted_unix_ms < window_open_unix_ms {
            continue;
        }
        present.entry(a.framework).or_default().insert(a.kind);
    }
    let mut gaps = Vec::new();
    for f in frameworks {
        for required in required_artifacts_for(*f) {
            if !present.get(f).is_some_and(|s| s.contains(&required)) {
                gaps.push(CoverageGap {
                    framework: *f,
                    missing: required,
                });
            }
        }
    }
    Ok(gaps)
}

/// GDPR DSAR record — per-subject; carries SLA target so the kernel
/// can flag overdue requests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DsarRequest {
    pub request_id: String,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,            // data_class: INTERNAL_ONLY
    pub subject_id_pseudonym: String, // data_class: INTERNAL_ONLY
    pub opened_unix_ms: u64,          // data_class: INTERNAL_ONLY
    pub closed_unix_ms: Option<u64>,  // data_class: INTERNAL_ONLY
}

impl DsarRequest {
    pub const SLA_DAYS: u64 = 30;
    pub const TARGET_DAYS: u64 = 5;

    pub fn elapsed_days(&self, now_unix_ms: u64) -> u64 {
        let close = self.closed_unix_ms.unwrap_or(now_unix_ms);
        let elapsed_ms = close.saturating_sub(self.opened_unix_ms);
        elapsed_ms / (1_000 * 60 * 60 * 24)
    }

    pub fn is_overdue(&self, now_unix_ms: u64) -> bool {
        self.closed_unix_ms.is_none() && self.elapsed_days(now_unix_ms) > Self::SLA_DAYS
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComplianceError {
    EmptyArtifactId,
    EmptyTenantId,
    EmptySeal,
    MalformedSeal,
}

impl ComplianceError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyArtifactId => "artifact id is empty".to_owned(),
            Self::EmptyTenantId => "tenant id is empty".to_owned(),
            Self::EmptySeal => "audit-chain seal hex is empty".to_owned(),
            Self::MalformedSeal => "audit-chain seal hex must be 64 hex chars".to_owned(),
        }
    }
}

/// 32-byte SHA-256 hex (sha2 family) is the canonical seal shape.
fn validate_seal(seal_hex: &str) -> Result<(), ComplianceError> {
    if seal_hex.is_empty() {
        return Err(ComplianceError::EmptySeal);
    }
    if seal_hex.len() != 64 || !seal_hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ComplianceError::MalformedSeal);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seal() -> String {
        "a".repeat(64)
    }

    fn art(
        kind: EvidenceArtifactKind,
        framework: ComplianceFramework,
        tenant: &str,
        ts: u64,
    ) -> EvidenceArtifact {
        EvidenceArtifact::new(
            format!("evt_{}", kind.wire_label()),
            kind,
            framework,
            seal(),
            ts,
            tenant.into(),
        )
        .unwrap()
    }

    #[test]
    fn frameworks_have_distinct_labels() {
        use std::collections::HashSet;
        let labels: HashSet<_> = ComplianceFramework::all()
            .iter()
            .map(|f| f.wire_label())
            .collect();
        assert_eq!(labels.len(), 4);
    }

    #[test]
    fn soc2_requires_six_artifact_kinds() {
        let req = required_artifacts_for(ComplianceFramework::Soc2TypeII);
        assert_eq!(req.len(), 6);
        assert!(req.contains(&EvidenceArtifactKind::PenTestReport));
        assert!(req.contains(&EvidenceArtifactKind::BackupRestoreDrillReceipt));
    }

    #[test]
    fn coverage_gaps_lists_missing_artifacts_per_framework() {
        let now = 100_000;
        let window = 0;
        // SOC2 needs 6 kinds; we'll provide 2 → 4 gaps for SOC2.
        let artifacts = vec![
            art(
                EvidenceArtifactKind::CiArtifactHash,
                ComplianceFramework::Soc2TypeII,
                "tenant_x",
                10,
            ),
            art(
                EvidenceArtifactKind::DeployReceipt,
                ComplianceFramework::Soc2TypeII,
                "tenant_x",
                20,
            ),
        ];
        let gaps = coverage_gaps(
            "tenant_x",
            &artifacts,
            &[ComplianceFramework::Soc2TypeII],
            window,
        )
        .unwrap();
        assert_eq!(gaps.len(), 4);
        assert!(
            gaps.iter()
                .all(|g| g.framework == ComplianceFramework::Soc2TypeII)
        );
        // Sanity: window cutoff
        let stale_gaps = coverage_gaps(
            "tenant_x",
            &artifacts,
            &[ComplianceFramework::Soc2TypeII],
            now, // window opens NOW; all old artifacts excluded
        )
        .unwrap();
        assert_eq!(stale_gaps.len(), 6);
    }

    #[test]
    fn cross_tenant_artifacts_excluded_from_coverage() {
        let other = art(
            EvidenceArtifactKind::CiArtifactHash,
            ComplianceFramework::Soc2TypeII,
            "tenant_other",
            10,
        );
        let gaps =
            coverage_gaps("tenant_x", &[other], &[ComplianceFramework::Soc2TypeII], 0).unwrap();
        // None of tenant_other's artifacts count for tenant_x.
        assert_eq!(gaps.len(), 6);
    }

    #[test]
    fn artifact_rejects_malformed_seal() {
        assert!(matches!(
            EvidenceArtifact::new(
                "evt_x".into(),
                EvidenceArtifactKind::CiArtifactHash,
                ComplianceFramework::Soc2TypeII,
                "shortseal".into(),
                10,
                "tenant_x".into(),
            ),
            Err(ComplianceError::MalformedSeal)
        ));
        assert!(matches!(
            EvidenceArtifact::new(
                "evt_x".into(),
                EvidenceArtifactKind::CiArtifactHash,
                ComplianceFramework::Soc2TypeII,
                String::new(),
                10,
                "tenant_x".into(),
            ),
            Err(ComplianceError::EmptySeal)
        ));
        // Wrong char outside hex
        assert!(matches!(
            EvidenceArtifact::new(
                "evt_x".into(),
                EvidenceArtifactKind::CiArtifactHash,
                ComplianceFramework::Soc2TypeII,
                "z".repeat(64),
                10,
                "tenant_x".into(),
            ),
            Err(ComplianceError::MalformedSeal)
        ));
    }

    #[test]
    fn dsar_overdue_after_thirty_days() {
        let day_ms = 1_000u64 * 60 * 60 * 24;
        let req = DsarRequest {
            request_id: "dsar_1".into(),
            tenant_id: "tenant_x".into(),
            subject_id_pseudonym: "subj_pseudonym".into(),
            opened_unix_ms: 0,
            closed_unix_ms: None,
        };
        assert!(!req.is_overdue(20 * day_ms));
        assert!(req.is_overdue(31 * day_ms));

        let closed_req = DsarRequest {
            closed_unix_ms: Some(3 * day_ms),
            ..req
        };
        assert!(!closed_req.is_overdue(40 * day_ms));
    }

    #[test]
    fn empty_artifact_id_or_tenant_rejected() {
        assert!(matches!(
            EvidenceArtifact::new(
                String::new(),
                EvidenceArtifactKind::CiArtifactHash,
                ComplianceFramework::Soc2TypeII,
                seal(),
                10,
                "tenant_x".into(),
            ),
            Err(ComplianceError::EmptyArtifactId)
        ));
        assert!(matches!(
            EvidenceArtifact::new(
                "evt_x".into(),
                EvidenceArtifactKind::CiArtifactHash,
                ComplianceFramework::Soc2TypeII,
                seal(),
                10,
                String::new(),
            ),
            Err(ComplianceError::EmptyTenantId)
        ));
    }
}
