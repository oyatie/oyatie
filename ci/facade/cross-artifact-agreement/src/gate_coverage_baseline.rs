//! Born-advisory frozen-baseline ratchet shared by the three gate-coverage-gap
//! advisory checks — prose⇄front-matter status agreement
//! ([`crate::prose_frontmatter_status`]), registry⇄derived-policy sync
//! ([`crate::registry_policy_sync`]), and generated-projection parity
//! ([`crate::adr_index_projection_parity`]).
//!
//! These three checks close the review class that caught six defects on #1327
//! that NO wired gate flagged: the defects lived in prose / derived-policy /
//! generated-projection surfaces the born-blocking §5.2 codes never key on.
//! Each check is BORN-ADVISORY: it does not join the born-blocking
//! [`crate::evaluate`] verdict (its findings are never in [`crate::VIOLATION_CODES`]);
//! instead it enforces NO-REGRESSION against a committed frozen baseline, exactly
//! like the tier-dependency-acyclicity ratchet
//! (`ci/facade/layer-dependency-acyclicity`): every advisory [`Finding`] is split
//! by membership in the baseline into `baselined` (known pre-existing debt,
//! advisory-only) and `regressions` (NEW — blocks). The live-corpus gate test
//! asserts the live advisory finding set equals the frozen baseline EXACTLY —
//! zero regressions (no NEW divergence) AND zero burned-down rows (a fixed
//! divergence must leave the baseline, forcing a re-freeze so a stale phantom row
//! can never silently rot the ratchet). The baseline is expected empty
//! (born-advisory-green) after #1327; a documented pre-existing divergence is the
//! only thing that ever lives in it.
//!
//! The baseline key identity is `{code}|{key}` over the shared [`Finding`] shape.
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.

use std::collections::BTreeSet;

use serde_json::Value;

use crate::Finding;

/// Validator id recorded by the gate-coverage baseline contract.
pub const GATE_COVERAGE_RATCHET_VALIDATOR: &str =
    "cloud-ci-cross-artifact-agreement/gate-coverage-born-advisory-ratchet";

/// The frozen known-debt baseline for the born-advisory gate-coverage checks,
/// loaded from `ci/facade/cross-artifact-agreement/gate-coverage-baseline.json`.
/// Keyed by `{code}|{key}`; a malformed row is dropped rather than silently
/// widening the baseline.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GateCoverageBaseline {
    keys: BTreeSet<String>,
}

impl GateCoverageBaseline {
    /// The canonical baseline identity for a finding.
    pub fn key_of(finding: &Finding) -> String {
        format!("{}|{}", finding.code, finding.key)
    }

    /// Parse the committed baseline document. Only `violations[].{code,key}` is
    /// consumed; `_comment`/`gate_id`/`frozen_at_ref` are provenance-only.
    pub fn from_value(value: &Value) -> Self {
        let keys = value
            .get("violations")
            .and_then(Value::as_array)
            .map(|rows| {
                rows.iter()
                    .filter_map(|row| {
                        let code = row.get("code").and_then(Value::as_str)?;
                        let key = row.get("key").and_then(Value::as_str)?;
                        Some(format!("{code}|{key}"))
                    })
                    .collect()
            })
            .unwrap_or_default();
        Self { keys }
    }

    /// The frozen baseline keys.
    pub fn keys(&self) -> &BTreeSet<String> {
        &self.keys
    }

    fn contains(&self, finding: &Finding) -> bool {
        self.keys.contains(&Self::key_of(finding))
    }
}

/// The two-sided ratchet split of a live advisory finding set against the frozen
/// baseline.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RatchetReport {
    /// Findings ABSENT from the baseline — NEW divergence. Any regression blocks.
    pub regressions: BTreeSet<Finding>,
    /// Findings PRESENT in the baseline — known debt, advisory-only.
    pub baselined: BTreeSet<Finding>,
    /// Baseline keys with NO live finding — a fixed divergence that must leave the
    /// baseline (re-freeze), so a stale phantom row cannot rot the ratchet.
    pub burned_down: BTreeSet<String>,
}

impl RatchetReport {
    /// The born-advisory pass condition: the live finding set equals the frozen
    /// baseline exactly — no NEW regression AND no stale burned-down row.
    pub fn is_clean(&self) -> bool {
        self.regressions.is_empty() && self.burned_down.is_empty()
    }
}

/// Split `findings` against the frozen `baseline` into the born-advisory ratchet
/// report. Pure and deterministic.
pub fn ratchet(findings: &BTreeSet<Finding>, baseline: &GateCoverageBaseline) -> RatchetReport {
    let mut regressions = BTreeSet::new();
    let mut baselined = BTreeSet::new();
    for finding in findings {
        if baseline.contains(finding) {
            baselined.insert(finding.clone());
        } else {
            regressions.insert(finding.clone());
        }
    }
    let live_keys: BTreeSet<String> = findings.iter().map(GateCoverageBaseline::key_of).collect();
    let burned_down = baseline.keys.difference(&live_keys).cloned().collect();
    RatchetReport {
        regressions,
        baselined,
        burned_down,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use serde_json::json;

    use super::*;

    fn finding(code: &str, key: &str) -> Finding {
        Finding::new(code, key)
    }

    #[test]
    fn empty_baseline_makes_every_finding_a_regression() {
        let baseline = GateCoverageBaseline::from_value(&json!({ "violations": [] }));
        let findings: BTreeSet<Finding> = [finding("c1", "k1"), finding("c2", "k2")]
            .into_iter()
            .collect();
        let report = ratchet(&findings, &baseline);
        assert_eq!(report.regressions, findings);
        assert!(report.baselined.is_empty());
        assert!(report.burned_down.is_empty());
        assert!(!report.is_clean(), "regressions must block");
    }

    #[test]
    fn baselined_finding_is_advisory_not_a_regression() {
        let baseline = GateCoverageBaseline::from_value(&json!({
            "violations": [{ "code": "c1", "key": "k1" }]
        }));
        let findings: BTreeSet<Finding> = [finding("c1", "k1"), finding("c2", "k2")]
            .into_iter()
            .collect();
        let report = ratchet(&findings, &baseline);
        assert_eq!(
            report.baselined,
            [finding("c1", "k1")].into_iter().collect()
        );
        assert_eq!(
            report.regressions,
            [finding("c2", "k2")].into_iter().collect()
        );
        assert!(report.burned_down.is_empty());
    }

    #[test]
    fn fixed_divergence_burns_down_and_forces_a_refreeze() {
        let baseline = GateCoverageBaseline::from_value(&json!({
            "violations": [{ "code": "c1", "key": "k1" }]
        }));
        let report = ratchet(&BTreeSet::new(), &baseline);
        assert_eq!(
            report.burned_down,
            ["c1|k1".to_owned()].into_iter().collect()
        );
        assert!(
            !report.is_clean(),
            "a stale baseline row that no longer reproduces must force a re-freeze"
        );
    }

    #[test]
    fn exact_match_is_clean() {
        let baseline = GateCoverageBaseline::from_value(&json!({
            "violations": [{ "code": "c1", "key": "k1" }]
        }));
        let findings: BTreeSet<Finding> = [finding("c1", "k1")].into_iter().collect();
        let report = ratchet(&findings, &baseline);
        assert!(report.is_clean());
        assert_eq!(report.baselined, findings);
    }

    #[test]
    fn malformed_baseline_rows_are_dropped_not_widened() {
        let baseline = GateCoverageBaseline::from_value(&json!({
            "violations": [{ "code": "c1" }, { "key": "k2" }, { "code": "c3", "key": "k3" }]
        }));
        assert_eq!(baseline.keys(), &["c3|k3".to_owned()].into_iter().collect());
    }
}
