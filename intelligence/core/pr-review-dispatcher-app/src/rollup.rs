//! Rollup-verdict aggregation for the reviewer panel.
//!
//! Inputs: zero-or-more per-facet finding JSON blobs from the subagent
//! panel. Output: a single APPROVE / CHANGES_REQUESTED / REJECT verdict
//! per [`Verdict`] + a rollup JSON that the GitHub Check Run + the
//! merge-queue admission log consume.

use std::collections::BTreeMap;

use crate::fanout::FacetId;

/// Per-facet recommendation from one subagent. Contains only the
/// fields the rollup needs to make a verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FacetFinding {
    pub facet: FacetId,
    pub reviewer_id: String,
    pub recommendation: FacetRecommendation,
}

/// What one facet's subagent recommends. Matches the
/// `final_recommendation` enum in the reviewer-panel spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FacetRecommendation {
    Approve,
    /// Reviewer requests specific changes; fix-loop should pick them up.
    ChangesRequested,
    /// Reviewer rejects outright; PR should not be re-tried automatically.
    Reject,
}

/// PR-level verdict. Order matters — `Reject` dominates
/// `ChangesRequested` dominates `Approve` per the conservative-merge
/// posture (`feedback_quality_performance_scalability_bar`: industry-leader
/// quality bar requires real review).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Approve,
    ChangesRequested,
    Reject,
}

impl Verdict {
    /// Canonical kebab-case label for the GitHub Check Run conclusion
    /// and the merge-queue admission event payload.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Approve => "APPROVE",
            Self::ChangesRequested => "CHANGES_REQUESTED",
            Self::Reject => "REJECT",
        }
    }

    /// Event name emitted into
    /// `registry/merge-queue-admission-log.json`.
    /// APPROVE → admission; ChangesRequested / Reject → fix-requested
    /// (consumed by IP-005 fix-loop).
    #[must_use]
    pub const fn admission_event(self) -> &'static str {
        match self {
            Self::Approve => "pr-review-approved",
            Self::ChangesRequested | Self::Reject => "pr-review-fix-requested",
        }
    }
}

/// Roll a set of per-facet findings up to one PR-level verdict.
///
/// Rules:
/// 1. Any `Reject` ⇒ `Reject`.
/// 2. Otherwise, any `ChangesRequested` ⇒ `ChangesRequested`.
/// 3. Otherwise, all `Approve` ⇒ `Approve`.
/// 4. Empty findings input ⇒ deliberate-scaffold-pending `Approve` with
///    `subagent_runtime_pending = true` (caller MUST surface this flag
///    in the rollup JSON; the verdict alone is insufficient signal).
///
/// The dispatcher's caller is responsible for verifying every required
/// facet has produced a finding before invoking this rollup — that
/// completeness check is `audit_panel_completeness` below.
#[must_use]
pub fn rollup_verdict(findings: &[FacetFinding]) -> Verdict {
    let mut has_change_request = false;
    for finding in findings {
        match finding.recommendation {
            FacetRecommendation::Reject => return Verdict::Reject,
            FacetRecommendation::ChangesRequested => has_change_request = true,
            FacetRecommendation::Approve => {}
        }
    }
    if has_change_request {
        Verdict::ChangesRequested
    } else {
        Verdict::Approve
    }
}

/// Result of comparing the realized panel against a required set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelCompletenessReport {
    pub required: Vec<FacetId>,
    pub present: Vec<FacetId>,
    pub missing: Vec<FacetId>,
    pub duplicate_reviewer_ids: Vec<String>,
}

impl PanelCompletenessReport {
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.missing.is_empty() && self.duplicate_reviewer_ids.is_empty()
    }
}

/// Audit the realized panel against the required facet set.
///
/// Per `feedback_consensus_debate_spectrum_lens_subagents` no single
/// `reviewer_id` may appear across multiple facets — that would indicate
/// a single agent wearing all facets, which is the bias-collapse failure
/// mode the lane refuses.
#[must_use]
pub fn audit_panel_completeness(
    required: &[FacetId],
    findings: &[FacetFinding],
) -> PanelCompletenessReport {
    let present_set: std::collections::BTreeSet<FacetId> =
        findings.iter().map(|f| f.facet).collect();

    let missing: Vec<FacetId> = required
        .iter()
        .copied()
        .filter(|facet| !present_set.contains(facet))
        .collect();

    // Detect any reviewer_id used across more than one facet.
    let mut reviewer_id_to_facets: BTreeMap<String, std::collections::BTreeSet<FacetId>> =
        BTreeMap::new();
    for finding in findings {
        reviewer_id_to_facets
            .entry(finding.reviewer_id.clone())
            .or_default()
            .insert(finding.facet);
    }
    let duplicate_reviewer_ids: Vec<String> = reviewer_id_to_facets
        .into_iter()
        .filter_map(|(id, facets)| if facets.len() > 1 { Some(id) } else { None })
        .collect();

    PanelCompletenessReport {
        required: required.to_vec(),
        present: present_set.into_iter().collect(),
        missing,
        duplicate_reviewer_ids,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approve(facet: FacetId) -> FacetFinding {
        FacetFinding {
            facet,
            reviewer_id: format!("claude-{}-changeX", facet.slug()),
            recommendation: FacetRecommendation::Approve,
        }
    }

    fn changes_requested(facet: FacetId) -> FacetFinding {
        FacetFinding {
            facet,
            reviewer_id: format!("claude-{}-changeX", facet.slug()),
            recommendation: FacetRecommendation::ChangesRequested,
        }
    }

    fn reject(facet: FacetId) -> FacetFinding {
        FacetFinding {
            facet,
            reviewer_id: format!("claude-{}-changeX", facet.slug()),
            recommendation: FacetRecommendation::Reject,
        }
    }

    #[test]
    fn empty_findings_default_to_approve() {
        // Deliberate-scaffold posture: until the subagent runtime lands,
        // zero findings = APPROVE with the pending flag surfaced by the
        // caller. The verdict alone is correct; the pending-flag is the
        // caller's responsibility.
        assert_eq!(rollup_verdict(&[]), Verdict::Approve);
    }

    #[test]
    fn all_approve_rolls_up_to_approve() {
        let findings = vec![approve(FacetId::F1Linus), approve(FacetId::F2Hyperscaler)];
        assert_eq!(rollup_verdict(&findings), Verdict::Approve);
    }

    #[test]
    fn any_changes_requested_rolls_up_to_changes_requested() {
        let findings = vec![
            approve(FacetId::F1Linus),
            changes_requested(FacetId::F7Security),
            approve(FacetId::F2Hyperscaler),
        ];
        assert_eq!(rollup_verdict(&findings), Verdict::ChangesRequested);
    }

    #[test]
    fn any_reject_dominates_changes_requested() {
        let findings = vec![
            approve(FacetId::F1Linus),
            changes_requested(FacetId::F7Security),
            reject(FacetId::F9Compliance),
            approve(FacetId::F2Hyperscaler),
        ];
        assert_eq!(rollup_verdict(&findings), Verdict::Reject);
    }

    #[test]
    fn verdict_labels_match_check_run_contract() {
        assert_eq!(Verdict::Approve.label(), "APPROVE");
        assert_eq!(Verdict::ChangesRequested.label(), "CHANGES_REQUESTED");
        assert_eq!(Verdict::Reject.label(), "REJECT");
    }

    #[test]
    fn approve_emits_admission_event_others_emit_fix_requested() {
        assert_eq!(Verdict::Approve.admission_event(), "pr-review-approved");
        assert_eq!(
            Verdict::ChangesRequested.admission_event(),
            "pr-review-fix-requested"
        );
        assert_eq!(Verdict::Reject.admission_event(), "pr-review-fix-requested");
    }

    #[test]
    fn completeness_report_detects_missing_facet() {
        let required = vec![FacetId::F1Linus, FacetId::F7Security];
        let findings = vec![approve(FacetId::F1Linus)];
        let report = audit_panel_completeness(&required, &findings);
        assert_eq!(report.missing, vec![FacetId::F7Security]);
        assert!(!report.is_complete());
    }

    #[test]
    fn completeness_report_detects_reviewer_id_duplicated_across_facets() {
        // Bias-collapse: same reviewer running F1 and F2. The lane rejects.
        let findings = vec![
            FacetFinding {
                facet: FacetId::F1Linus,
                reviewer_id: "claude-omnibus-changeX".into(),
                recommendation: FacetRecommendation::Approve,
            },
            FacetFinding {
                facet: FacetId::F2Hyperscaler,
                reviewer_id: "claude-omnibus-changeX".into(),
                recommendation: FacetRecommendation::Approve,
            },
        ];
        let required = vec![FacetId::F1Linus, FacetId::F2Hyperscaler];
        let report = audit_panel_completeness(&required, &findings);
        assert!(report.missing.is_empty());
        assert_eq!(
            report.duplicate_reviewer_ids,
            vec!["claude-omnibus-changeX"]
        );
        assert!(!report.is_complete());
    }

    #[test]
    fn complete_panel_no_duplicates_reports_complete() {
        let required = vec![FacetId::F1Linus, FacetId::F2Hyperscaler];
        let findings = vec![approve(FacetId::F1Linus), approve(FacetId::F2Hyperscaler)];
        let report = audit_panel_completeness(&required, &findings);
        assert!(report.is_complete());
    }
}
