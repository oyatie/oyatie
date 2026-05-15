//! Stuck-PR escalation — invoked by the dispatcher when a PR exhausts
//! its shared-pool retry budget (6th occurrence across CI + review).
//!
//! The actual GitHub-issue creation runs inside `.github/workflows/ci-failure-fix-loop.yml`
//! via `gh issue create` (the workflow has `permissions: issues: write` and
//! a `GITHUB_TOKEN`). The Rust side renders the canonical issue body +
//! label set and writes them to a per-PR escalation evidence file that
//! the workflow consumes. Two reasons:
//!
//! 1. Keeping `gh` out of the dispatcher binary keeps the crate
//!    dependency footprint at zero (matches the banned-primitives-app
//!    sibling pattern — no `gh` shell-out from Rust).
//! 2. The `human-escalation` label + canonical body text live in code
//!    (one source of truth), not in a YAML heredoc.

use std::fmt;

use crate::event::{FixLoopSource, json_string};

pub const HUMAN_ESCALATION_LABEL: &str = "human-escalation";
pub const STUCK_PR_LABEL: &str = "fix-loop-exhausted";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscalationRecord {
    pub pr_number: u64,
    pub final_source: FixLoopSource,
    pub attempts_used: u32,
    pub labels: Vec<&'static str>,
    pub issue_title: String,
    pub issue_body: String,
    pub emitted_at_epoch: u64,
}

impl EscalationRecord {
    /// Build the canonical stuck-PR escalation. `attempts_used` MUST equal
    /// the shared-pool maximum (5); the constructor asserts it is at least
    /// 5 so dispatcher mis-wirings (e.g. escalating at attempt 3) are caught
    /// at the boundary.
    pub fn open_stuck_pr_issue(
        pr_number: u64,
        final_source: FixLoopSource,
        attempts_used: u32,
        emitted_at_epoch: u64,
    ) -> Result<Self, EscalationError> {
        if pr_number == 0 {
            return Err(EscalationError::InvalidPrNumber);
        }
        if attempts_used < crate::retry_budget::MAX_ATTEMPTS_PER_PR {
            return Err(EscalationError::AttemptsBelowEscalationThreshold {
                attempts_used,
                threshold: crate::retry_budget::MAX_ATTEMPTS_PER_PR,
            });
        }
        if emitted_at_epoch == 0 {
            return Err(EscalationError::EpochZero);
        }
        let issue_title = format!("Stuck PR #{pr_number}: fix-loop retry budget exhausted");
        let issue_body = format!(
            "PR #{pr} exhausted the fix-loop shared retry budget of {max} attempts across BOTH CI and review sources.\n\n\
             - Final source: `{source}`\n\
             - Attempts used: {used}/{max}\n\
             - Dispatcher escalated at epoch {epoch}\n\n\
             Per the canonical state machine (`push → CI → fix-loop until green → review → fix-loop until APPROVE → merge`), \
             this PR is now parked from automated fix-loop dispatch and requires human triage. The fix-loop dispatcher will \
             NOT post further bundles for this PR until the `{escalation_label}` label is removed.\n\n\
             Evidence: `evidence/pipeline-maturity-glue/ip-005-fix-loop/{pr}/` (per-attempt bundles).\n\n\
             Related: M-CC-P10-IP-005 (dispatcher), M-CC-P10-IP-006 (merge-queue eviction on exhaustion).",
            pr = pr_number,
            max = crate::retry_budget::MAX_ATTEMPTS_PER_PR,
            source = final_source.as_wire(),
            used = attempts_used,
            epoch = emitted_at_epoch,
            escalation_label = HUMAN_ESCALATION_LABEL,
        );
        Ok(Self {
            pr_number,
            final_source,
            attempts_used,
            labels: vec![HUMAN_ESCALATION_LABEL, STUCK_PR_LABEL],
            issue_title,
            issue_body,
            emitted_at_epoch,
        })
    }

    /// JSON object representation written to the per-PR escalation
    /// evidence file `evidence/pipeline-maturity-glue/ip-005-fix-loop/<pr>/escalation.json`.
    pub fn to_json(&self) -> String {
        let labels = self
            .labels
            .iter()
            .map(|label| json_string(label))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"attempts_used\":{used},\"emitted_at_epoch\":{epoch},\"final_source\":{src},\"issue_body\":{body},\"issue_title\":{title},\"labels\":[{labels}],\"pr_number\":{pr}}}",
            used = self.attempts_used,
            epoch = self.emitted_at_epoch,
            src = json_string(self.final_source.as_wire()),
            body = json_string(&self.issue_body),
            title = json_string(&self.issue_title),
            labels = labels,
            pr = self.pr_number,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscalationError {
    InvalidPrNumber,
    EpochZero,
    AttemptsBelowEscalationThreshold { attempts_used: u32, threshold: u32 },
}

impl fmt::Display for EscalationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for EscalationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_stuck_pr_issue_requires_full_attempt_count() {
        let err = EscalationRecord::open_stuck_pr_issue(7, FixLoopSource::CiFailure, 4, 1)
            .unwrap_err();
        assert_eq!(
            err,
            EscalationError::AttemptsBelowEscalationThreshold {
                attempts_used: 4,
                threshold: crate::retry_budget::MAX_ATTEMPTS_PER_PR,
            }
        );
    }

    #[test]
    fn open_stuck_pr_issue_rejects_zero_pr_and_zero_epoch() {
        assert_eq!(
            EscalationRecord::open_stuck_pr_issue(
                0,
                FixLoopSource::CiFailure,
                5,
                1,
            )
            .unwrap_err(),
            EscalationError::InvalidPrNumber
        );
        assert_eq!(
            EscalationRecord::open_stuck_pr_issue(
                1,
                FixLoopSource::CiFailure,
                5,
                0,
            )
            .unwrap_err(),
            EscalationError::EpochZero
        );
    }

    #[test]
    fn open_stuck_pr_issue_renders_canonical_title_body_and_labels() {
        let record = EscalationRecord::open_stuck_pr_issue(
            42,
            FixLoopSource::PrReviewFixRequested,
            5,
            1_715_000_000,
        )
        .unwrap();
        assert_eq!(record.labels, vec![HUMAN_ESCALATION_LABEL, STUCK_PR_LABEL]);
        assert!(record.issue_title.contains("PR #42"));
        assert!(record.issue_body.contains("pr-review-fix-requested"));
        assert!(record.issue_body.contains("M-CC-P10-IP-005"));
        assert!(record.issue_body.contains("M-CC-P10-IP-006"));
        let json = record.to_json();
        assert!(json.contains("\"labels\":[\"human-escalation\",\"fix-loop-exhausted\"]"));
        assert!(json.contains("\"pr_number\":42"));
    }
}
