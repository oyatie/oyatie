//! Context-bundle assembly — schema-conforming agent task payload.
//!
//! The bundle shape matches `/specs/cross-cutting/ci-fix-loop-context-bundle.json`
//! exactly (schema version 1). Construction is fallible and bounded:
//!
//! - All sha fields are validated as 40- or 64-hex.
//! - `commit_history` is truncated to last N=5 commits (most recent first).
//! - `attempt` must be in `1..=MAX_ATTEMPTS_PER_PR`; on the 6th occurrence
//!   the dispatcher escalates instead of building a bundle (see
//!   [`crate::retry_budget::Budget::register_attempt`]).
//! - `failure_surface.failed_jobs` MUST contain at least one entry — a
//!   zero-failed-jobs "failure" is a contradiction the dispatcher refuses
//!   to encode.

use std::fmt;

use crate::event::{FixLoopSource, json_string};
use crate::retry_budget::MAX_ATTEMPTS_PER_PR;

const SHA1_HEX_LEN: usize = 40;
const SHA256_HEX_LEN: usize = 64;

/// One failed job from the upstream `workflow_run` payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FailedJob {
    pub job_name: String,
    pub conclusion: String,
    pub log_excerpt_sha256: String,
    pub log_uri: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewVerdict {
    Reject,
    ChangesRequested,
}

impl ReviewVerdict {
    fn as_wire(&self) -> &'static str {
        match self {
            ReviewVerdict::Reject => "REJECT",
            ReviewVerdict::ChangesRequested => "CHANGES_REQUESTED",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewFinding {
    pub facet_id: String,
    pub verdict: ReviewVerdict,
    pub body_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FailureSurface {
    pub failed_jobs: Vec<FailedJob>,
    pub review_findings: Vec<ReviewFinding>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiffSummary {
    pub files_changed: u32,
    pub additions: u32,
    pub deletions: u32,
    pub patch_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitHistoryEntry {
    pub sha: String,
    pub subject: String,
    pub author_epoch: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedgerCandidate {
    pub row_id: String,
    pub mistake_class: String,
    pub first_occurrence_sha: String,
}

/// Schema-conforming agent task payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextBundle {
    pub schema_version: u32,
    pub source: FixLoopSource,
    pub pr_number: u64,
    pub head_sha: String,
    pub base_sha: String,
    pub attempt: u32,
    pub attempts_used: u32,
    pub attempts_remaining: u32,
    pub failure_surface: FailureSurface,
    pub diff_summary: DiffSummary,
    pub commit_history: Vec<CommitHistoryEntry>,
    pub ledger_candidates: Vec<LedgerCandidate>,
    pub emitted_at_epoch: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextBundleError {
    InvalidPrNumber,
    InvalidSha1(&'static str),
    InvalidSha256(&'static str),
    EmptyJobName,
    EmptyJobConclusion,
    EmptyLogUri,
    EmptyFacetId,
    AttemptOutOfRange(u32),
    AttemptsExceedBudget {
        attempts_used: u32,
        max: u32,
    },
    EmptyFailureSurface,
    CommitHistoryTooLong(usize),
    EmptyCommitSubject,
    InvalidLedgerRowId,
    EmittedAtEpochZero,
}

impl fmt::Display for ContextBundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for ContextBundleError {}

impl ContextBundle {
    /// Strict, fallible constructor.
    ///
    /// All invariants enforced here are also enforced by the JSON schema
    /// in `/specs/cross-cutting/ci-fix-loop-context-bundle.json`; we duplicate
    /// the checks at the Rust boundary so the dispatcher never produces a
    /// bundle the agent-runtime would reject.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        source: FixLoopSource,
        pr_number: u64,
        head_sha: impl Into<String>,
        base_sha: impl Into<String>,
        attempt: u32,
        attempts_used: u32,
        failure_surface: FailureSurface,
        diff_summary: DiffSummary,
        commit_history: Vec<CommitHistoryEntry>,
        ledger_candidates: Vec<LedgerCandidate>,
        emitted_at_epoch: u64,
    ) -> Result<Self, ContextBundleError> {
        if pr_number == 0 {
            return Err(ContextBundleError::InvalidPrNumber);
        }
        if emitted_at_epoch == 0 {
            return Err(ContextBundleError::EmittedAtEpochZero);
        }
        let head_sha = head_sha.into();
        let base_sha = base_sha.into();
        validate_sha1(&head_sha, "head_sha")?;
        validate_sha1(&base_sha, "base_sha")?;
        if attempt == 0 || attempt > MAX_ATTEMPTS_PER_PR {
            return Err(ContextBundleError::AttemptOutOfRange(attempt));
        }
        if attempts_used > MAX_ATTEMPTS_PER_PR {
            return Err(ContextBundleError::AttemptsExceedBudget {
                attempts_used,
                max: MAX_ATTEMPTS_PER_PR,
            });
        }
        validate_failure_surface(&failure_surface)?;
        validate_diff_summary(&diff_summary)?;
        validate_commit_history(&commit_history)?;
        validate_ledger_candidates(&ledger_candidates)?;
        let attempts_remaining = MAX_ATTEMPTS_PER_PR.saturating_sub(attempts_used);
        Ok(Self {
            schema_version: crate::SCHEMA_VERSION,
            source,
            pr_number,
            head_sha,
            base_sha,
            attempt,
            attempts_used,
            attempts_remaining,
            failure_surface,
            diff_summary,
            commit_history,
            ledger_candidates,
            emitted_at_epoch,
        })
    }

    /// Stable JSON serialization without pulling serde. Keys are
    /// alphabetical for diff-friendliness; the output validates against the
    /// schema in `/specs/cross-cutting/ci-fix-loop-context-bundle.json`.
    pub fn to_json(&self) -> String {
        let mut buf = String::new();
        buf.push('{');
        push_kv_u64(&mut buf, "attempt", u64::from(self.attempt), true);
        // Nested object inserted in alphabetical key order, but we open
        // a sub-object for `retry_budget` after `failure_surface`. To
        // keep the top-level alphabetical we render keys: attempt,
        // base_sha, commit_history, diff_summary, emitted_at_epoch,
        // failure_surface, head_sha, ledger_candidates, pr_number,
        // retry_budget, schema_version, source.
        push_kv_string(&mut buf, "base_sha", &self.base_sha, false);
        push_kv_array(&mut buf, "commit_history", &self.commit_history, |entry| {
            format!(
                "{{\"author_epoch\":{epoch},\"sha\":{sha},\"subject\":{subj}}}",
                epoch = entry.author_epoch,
                sha = json_string(&entry.sha),
                subj = json_string(&entry.subject),
            )
        });
        push_kv_raw(&mut buf, "diff_summary", &diff_summary_json(&self.diff_summary));
        push_kv_u64(&mut buf, "emitted_at_epoch", self.emitted_at_epoch, false);
        push_kv_raw(
            &mut buf,
            "failure_surface",
            &failure_surface_json(&self.failure_surface),
        );
        push_kv_string(&mut buf, "head_sha", &self.head_sha, false);
        push_kv_array(
            &mut buf,
            "ledger_candidates",
            &self.ledger_candidates,
            |candidate| {
                format!(
                    "{{\"first_occurrence_sha\":{sha},\"mistake_class\":{class},\"row_id\":{row}}}",
                    sha = json_string(&candidate.first_occurrence_sha),
                    class = json_string(&candidate.mistake_class),
                    row = json_string(&candidate.row_id),
                )
            },
        );
        push_kv_u64(&mut buf, "pr_number", self.pr_number, false);
        push_kv_raw(
            &mut buf,
            "retry_budget",
            &format!(
                "{{\"attempts_remaining\":{rem},\"attempts_used\":{used},\"max_attempts\":{max}}}",
                rem = self.attempts_remaining,
                used = self.attempts_used,
                max = MAX_ATTEMPTS_PER_PR,
            ),
        );
        push_kv_u64(
            &mut buf,
            "schema_version",
            u64::from(self.schema_version),
            false,
        );
        push_kv_string(&mut buf, "source", self.source.as_wire(), false);
        buf.push('}');
        buf
    }
}

fn diff_summary_json(diff: &DiffSummary) -> String {
    format!(
        "{{\"additions\":{add},\"deletions\":{del},\"files_changed\":{files},\"patch_sha256\":{sha}}}",
        add = diff.additions,
        del = diff.deletions,
        files = diff.files_changed,
        sha = json_string(&diff.patch_sha256),
    )
}

fn failure_surface_json(surface: &FailureSurface) -> String {
    let failed_jobs = surface
        .failed_jobs
        .iter()
        .map(|job| {
            format!(
                "{{\"conclusion\":{conc},\"job_name\":{name},\"log_excerpt_sha256\":{sha},\"log_uri\":{uri}}}",
                conc = json_string(&job.conclusion),
                name = json_string(&job.job_name),
                sha = json_string(&job.log_excerpt_sha256),
                uri = json_string(&job.log_uri),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let review = surface
        .review_findings
        .iter()
        .map(|finding| {
            format!(
                "{{\"body_sha256\":{body},\"facet_id\":{facet},\"verdict\":{verdict}}}",
                body = json_string(&finding.body_sha256),
                facet = json_string(&finding.facet_id),
                verdict = json_string(finding.verdict.as_wire()),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    if surface.review_findings.is_empty() {
        format!("{{\"failed_jobs\":[{failed_jobs}]}}")
    } else {
        format!("{{\"failed_jobs\":[{failed_jobs}],\"review_findings\":[{review}]}}")
    }
}

fn push_kv_u64(buf: &mut String, key: &str, value: u64, first: bool) {
    if !first {
        buf.push(',');
    }
    buf.push_str(&format!("{}:{}", json_string(key), value));
}

fn push_kv_string(buf: &mut String, key: &str, value: &str, first: bool) {
    if !first {
        buf.push(',');
    }
    buf.push_str(&format!("{}:{}", json_string(key), json_string(value)));
}

fn push_kv_raw(buf: &mut String, key: &str, raw_value: &str) {
    buf.push(',');
    buf.push_str(&format!("{}:{}", json_string(key), raw_value));
}

fn push_kv_array<T, F>(buf: &mut String, key: &str, items: &[T], render: F)
where
    F: Fn(&T) -> String,
{
    buf.push(',');
    let body = items.iter().map(render).collect::<Vec<_>>().join(",");
    buf.push_str(&format!("{}:[{}]", json_string(key), body));
}

fn validate_sha1(value: &str, field: &'static str) -> Result<(), ContextBundleError> {
    if value.len() == SHA1_HEX_LEN && value.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(ContextBundleError::InvalidSha1(field))
    }
}

fn validate_sha256(value: &str, field: &'static str) -> Result<(), ContextBundleError> {
    if value.len() == SHA256_HEX_LEN && value.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(ContextBundleError::InvalidSha256(field))
    }
}

fn validate_failure_surface(surface: &FailureSurface) -> Result<(), ContextBundleError> {
    if surface.failed_jobs.is_empty() {
        return Err(ContextBundleError::EmptyFailureSurface);
    }
    for job in &surface.failed_jobs {
        if job.job_name.trim().is_empty() {
            return Err(ContextBundleError::EmptyJobName);
        }
        if job.conclusion.trim().is_empty() {
            return Err(ContextBundleError::EmptyJobConclusion);
        }
        if job.log_uri.trim().is_empty() {
            return Err(ContextBundleError::EmptyLogUri);
        }
        validate_sha256(&job.log_excerpt_sha256, "failed_jobs[].log_excerpt_sha256")?;
    }
    for finding in &surface.review_findings {
        if finding.facet_id.trim().is_empty() {
            return Err(ContextBundleError::EmptyFacetId);
        }
        validate_sha256(&finding.body_sha256, "review_findings[].body_sha256")?;
    }
    Ok(())
}

fn validate_diff_summary(diff: &DiffSummary) -> Result<(), ContextBundleError> {
    validate_sha256(&diff.patch_sha256, "diff_summary.patch_sha256")
}

fn validate_commit_history(history: &[CommitHistoryEntry]) -> Result<(), ContextBundleError> {
    if history.len() > 5 {
        return Err(ContextBundleError::CommitHistoryTooLong(history.len()));
    }
    for entry in history {
        validate_sha1(&entry.sha, "commit_history[].sha")?;
        if entry.subject.trim().is_empty() {
            return Err(ContextBundleError::EmptyCommitSubject);
        }
    }
    Ok(())
}

fn validate_ledger_candidates(
    candidates: &[LedgerCandidate],
) -> Result<(), ContextBundleError> {
    for candidate in candidates {
        if !candidate.row_id.starts_with("mistakes-ledger:")
            || candidate.row_id.len() == "mistakes-ledger:".len()
        {
            return Err(ContextBundleError::InvalidLedgerRowId);
        }
        validate_sha1(
            &candidate.first_occurrence_sha,
            "ledger_candidates[].first_occurrence_sha",
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_surface() -> FailureSurface {
        FailureSurface {
            failed_jobs: vec![FailedJob {
                job_name: "cargo clippy -D warnings".into(),
                conclusion: "failure".into(),
                log_excerpt_sha256: "a".repeat(64),
                log_uri: "https://example.test/log".into(),
            }],
            review_findings: Vec::new(),
        }
    }

    fn ok_diff() -> DiffSummary {
        DiffSummary {
            files_changed: 3,
            additions: 50,
            deletions: 10,
            patch_sha256: "b".repeat(64),
        }
    }

    fn build_ok(attempt: u32, attempts_used: u32) -> Result<ContextBundle, ContextBundleError> {
        ContextBundle::build(
            FixLoopSource::CiFailure,
            42,
            "1".repeat(40),
            "2".repeat(40),
            attempt,
            attempts_used,
            ok_surface(),
            ok_diff(),
            Vec::new(),
            Vec::new(),
            1_715_000_000,
        )
    }

    #[test]
    fn build_rejects_invalid_pr_number() {
        let err = ContextBundle::build(
            FixLoopSource::CiFailure,
            0,
            "1".repeat(40),
            "2".repeat(40),
            1,
            0,
            ok_surface(),
            ok_diff(),
            Vec::new(),
            Vec::new(),
            1,
        )
        .unwrap_err();
        assert_eq!(err, ContextBundleError::InvalidPrNumber);
    }

    #[test]
    fn build_rejects_bad_sha1() {
        let err = ContextBundle::build(
            FixLoopSource::CiFailure,
            1,
            "not-a-sha".to_string(),
            "2".repeat(40),
            1,
            0,
            ok_surface(),
            ok_diff(),
            Vec::new(),
            Vec::new(),
            1,
        )
        .unwrap_err();
        assert_eq!(err, ContextBundleError::InvalidSha1("head_sha"));
    }

    #[test]
    fn build_rejects_attempt_out_of_range() {
        assert_eq!(
            build_ok(0, 0).unwrap_err(),
            ContextBundleError::AttemptOutOfRange(0)
        );
        assert_eq!(
            build_ok(6, 5).unwrap_err(),
            ContextBundleError::AttemptOutOfRange(6)
        );
    }

    #[test]
    fn build_requires_at_least_one_failed_job() {
        let err = ContextBundle::build(
            FixLoopSource::CiFailure,
            1,
            "1".repeat(40),
            "2".repeat(40),
            1,
            0,
            FailureSurface {
                failed_jobs: Vec::new(),
                review_findings: Vec::new(),
            },
            ok_diff(),
            Vec::new(),
            Vec::new(),
            1,
        )
        .unwrap_err();
        assert_eq!(err, ContextBundleError::EmptyFailureSurface);
    }

    #[test]
    fn build_rejects_commit_history_longer_than_5() {
        let history = (0..6)
            .map(|i| CommitHistoryEntry {
                sha: format!("{:040x}", i),
                subject: format!("c{i}"),
                author_epoch: 1,
            })
            .collect();
        let err = ContextBundle::build(
            FixLoopSource::CiFailure,
            1,
            "1".repeat(40),
            "2".repeat(40),
            1,
            0,
            ok_surface(),
            ok_diff(),
            history,
            Vec::new(),
            1,
        )
        .unwrap_err();
        assert_eq!(err, ContextBundleError::CommitHistoryTooLong(6));
    }

    #[test]
    fn build_rejects_bad_ledger_row_id() {
        let err = ContextBundle::build(
            FixLoopSource::CiFailure,
            1,
            "1".repeat(40),
            "2".repeat(40),
            1,
            0,
            ok_surface(),
            ok_diff(),
            Vec::new(),
            vec![LedgerCandidate {
                row_id: "bad".to_string(),
                mistake_class: "ci-action-sha-rotation".to_string(),
                first_occurrence_sha: "3".repeat(40),
            }],
            1,
        )
        .unwrap_err();
        assert_eq!(err, ContextBundleError::InvalidLedgerRowId);
    }

    #[test]
    fn build_succeeds_and_computes_remaining() {
        let bundle = build_ok(2, 1).unwrap();
        assert_eq!(bundle.attempts_remaining, MAX_ATTEMPTS_PER_PR - 1);
        assert_eq!(bundle.source, FixLoopSource::CiFailure);
        assert_eq!(bundle.schema_version, crate::SCHEMA_VERSION);
    }

    #[test]
    fn to_json_is_alphabetically_ordered_and_includes_retry_budget() {
        let bundle = build_ok(1, 0).unwrap();
        let json = bundle.to_json();
        // alphabetical: attempt, base_sha, commit_history, diff_summary,
        // emitted_at_epoch, failure_surface, head_sha, ledger_candidates,
        // pr_number, retry_budget, schema_version, source
        let positions = [
            json.find("\"attempt\"").unwrap(),
            json.find("\"base_sha\"").unwrap(),
            json.find("\"commit_history\"").unwrap(),
            json.find("\"diff_summary\"").unwrap(),
            json.find("\"emitted_at_epoch\"").unwrap(),
            json.find("\"failure_surface\"").unwrap(),
            json.find("\"head_sha\"").unwrap(),
            json.find("\"ledger_candidates\"").unwrap(),
            json.find("\"pr_number\"").unwrap(),
            json.find("\"retry_budget\"").unwrap(),
            json.find("\"schema_version\"").unwrap(),
            json.find("\"source\"").unwrap(),
        ];
        for window in positions.windows(2) {
            assert!(window[0] < window[1], "key order is alphabetical");
        }
        assert!(json.contains("\"max_attempts\":5"));
        assert!(json.contains("\"schema_version\":1"));
    }

    #[test]
    fn review_source_supports_review_findings() {
        let surface = FailureSurface {
            failed_jobs: vec![FailedJob {
                job_name: "oya-pr-review".into(),
                conclusion: "failure".into(),
                log_excerpt_sha256: "c".repeat(64),
                log_uri: "https://example.test/review-rollup".into(),
            }],
            review_findings: vec![ReviewFinding {
                facet_id: "F1".into(),
                verdict: ReviewVerdict::ChangesRequested,
                body_sha256: "d".repeat(64),
            }],
        };
        let bundle = ContextBundle::build(
            FixLoopSource::PrReviewFixRequested,
            7,
            "1".repeat(40),
            "2".repeat(40),
            1,
            0,
            surface,
            ok_diff(),
            Vec::new(),
            Vec::new(),
            1,
        )
        .unwrap();
        let json = bundle.to_json();
        assert!(json.contains("\"source\":\"pr-review-fix-requested\""));
        assert!(json.contains("\"facet_id\":\"F1\""));
        assert!(json.contains("\"verdict\":\"CHANGES_REQUESTED\""));
    }
}
