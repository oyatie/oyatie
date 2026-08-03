//! KR acceptance-evidence types for M07/P08 closure.
//!
//! Pure domain types — no I/O, no serde, no external deps beyond the crate's
//! existing `data-boundary-kernel` transitive surface.

/// Classifies the category of an acceptance test included in a KR evidence bundle.
///
/// Each variant maps to one of the ADR-0210 M3 closure criteria.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AcceptanceTestKind {
    /// k6 load test proving the 3 k-person payroll gross-to-net SLO (≤ 30 s).
    PayrollLoadSlo,
    /// Playwright end-to-end browser smoke test for the live B2B surface.
    BrowserSmokeE2e,
    /// Restore-drill runbook execution proving RTO ≤ target.
    RestoreDrill,
    /// Gate-validated corpus citation coverage for the KR tenant.
    CorpusCitation,
    /// Prometheus/Grafana SLO dashboard export attached to the evidence bundle.
    MonitoringSnapshot,
}

impl AcceptanceTestKind {
    /// Canonical ASCII label used in evidence-bundle JSON fields and audit rows.
    pub const fn label(self) -> &'static str {
        match self {
            Self::PayrollLoadSlo => "payroll_load_slo",
            Self::BrowserSmokeE2e => "browser_smoke_e2e",
            Self::RestoreDrill => "restore_drill",
            Self::CorpusCitation => "corpus_citation",
            Self::MonitoringSnapshot => "monitoring_snapshot",
        }
    }

    /// All variants in declaration order.
    pub const ALL: [Self; 5] = [
        Self::PayrollLoadSlo,
        Self::BrowserSmokeE2e,
        Self::RestoreDrill,
        Self::CorpusCitation,
        Self::MonitoringSnapshot,
    ];

    /// Parse from the canonical label produced by [`Self::label`].
    pub fn parse_label(label: &str) -> Option<Self> {
        match label {
            "payroll_load_slo" => Some(Self::PayrollLoadSlo),
            "browser_smoke_e2e" => Some(Self::BrowserSmokeE2e),
            "restore_drill" => Some(Self::RestoreDrill),
            "corpus_citation" => Some(Self::CorpusCitation),
            "monitoring_snapshot" => Some(Self::MonitoringSnapshot),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_variants_label_roundtrip() {
        for kind in AcceptanceTestKind::ALL {
            let label = kind.label();
            let parsed = AcceptanceTestKind::parse_label(label);
            assert_eq!(parsed, Some(kind), "roundtrip failed for {label}");
        }
    }

    #[test]
    fn unknown_label_returns_none() {
        assert_eq!(AcceptanceTestKind::parse_label("unknown"), None);
    }

    #[test]
    fn all_variants_covered() {
        assert_eq!(AcceptanceTestKind::ALL.len(), 5);
    }

    #[test]
    fn labels_are_unique() {
        let labels: Vec<&str> = AcceptanceTestKind::ALL.iter().map(|k| k.label()).collect();
        let mut deduped = labels.clone();
        deduped.sort_unstable();
        deduped.dedup();
        assert_eq!(labels.len(), deduped.len());
    }
}
