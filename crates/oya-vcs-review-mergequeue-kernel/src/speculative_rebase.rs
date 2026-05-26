//! Speculative rebase — IP-006 §"Speculative rebase + re-CI against
//! current queue HEAD (not the original rebase target — queue may have
//! advanced)".
//!
//! When a parked PR's fix lands, the scheduler MUST re-rebase against the
//! CURRENT queue head — which may have advanced while this PR was parked.
//! The kernel side is pure: given (parked_head_sha, current_queue_head_sha,
//! pr_new_head_sha) it returns a `RebaseDecision` describing what the
//! integration crate should ask Oya VCS / git to do.

use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RebaseRequest {
    /// PR being revalidated.
    pub pr_number: u64, // data_class: INTERNAL_ONLY
    /// The SHA the fix-loop just pushed to the PR branch (pulled from the
    /// IP-005 dispatcher's bundle or the new workflow_run head_sha).
    pub pr_new_head_sha: String, // data_class: INTERNAL_ONLY
    /// What the merge-queue head pointed at when the PR was parked.
    pub queue_head_at_park: String, // data_class: INTERNAL_ONLY
    /// What the merge-queue head points at NOW (may have advanced
    /// because other PRs landed while this one was parked).
    pub current_queue_head_sha: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RebaseDecision {
    /// Queue head unchanged since park; fast-path — re-CI against the
    /// existing speculative base. Counterintuitively this is the LESS
    /// common path (PRs in queue usually advance during a fix-loop).
    FastPath {
        pr_number: u64,
        base_sha: String,
        new_head_sha: String,
    },
    /// Queue head has advanced; re-rebase against current head before
    /// re-running admission CI.
    Reproject {
        pr_number: u64,
        new_base_sha: String,
        new_head_sha: String,
        skipped_generations: u32,
    },
    /// No new head pushed yet — fix-loop hasn't converged; keep parked.
    NoOp { pr_number: u64 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RebaseError {
    InvalidPrNumber,
    InvalidSha,
}

impl fmt::Display for RebaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for RebaseError {}

/// Pure decision function.
///
/// `generations_advanced` is the number of PRs that merged since this PR
/// parked; the scheduler tracks it (see `scheduler::Scheduler::advance_head`)
/// and passes it to this function. Used only for the audit log inside
/// `Reproject::skipped_generations` so the convergence-proof tick log
/// shows how far this PR fell behind.
pub fn rebase_against_head(
    request: &RebaseRequest,
    generations_advanced: u32,
) -> Result<RebaseDecision, RebaseError> {
    if request.pr_number == 0 {
        return Err(RebaseError::InvalidPrNumber);
    }
    for sha in [
        &request.pr_new_head_sha,
        &request.queue_head_at_park,
        &request.current_queue_head_sha,
    ] {
        if !sha.is_empty() && !is_sha1_hex(sha) {
            return Err(RebaseError::InvalidSha);
        }
    }
    if request.pr_new_head_sha.is_empty() {
        return Ok(RebaseDecision::NoOp {
            pr_number: request.pr_number,
        });
    }
    if request.queue_head_at_park == request.current_queue_head_sha {
        return Ok(RebaseDecision::FastPath {
            pr_number: request.pr_number,
            base_sha: request.current_queue_head_sha.clone(),
            new_head_sha: request.pr_new_head_sha.clone(),
        });
    }
    Ok(RebaseDecision::Reproject {
        pr_number: request.pr_number,
        new_base_sha: request.current_queue_head_sha.clone(),
        new_head_sha: request.pr_new_head_sha.clone(),
        skipped_generations: generations_advanced,
    })
}

fn is_sha1_hex(value: &str) -> bool {
    value.len() == 40 && value.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(park: &str, head: &str, new: &str) -> RebaseRequest {
        RebaseRequest {
            pr_number: 7,
            pr_new_head_sha: new.to_string(),
            queue_head_at_park: park.to_string(),
            current_queue_head_sha: head.to_string(),
        }
    }

    #[test]
    fn no_op_when_no_new_head() {
        let r = req(&"1".repeat(40), &"1".repeat(40), "");
        assert_eq!(
            rebase_against_head(&r, 0).unwrap(),
            RebaseDecision::NoOp { pr_number: 7 }
        );
    }

    #[test]
    fn fast_path_when_queue_head_unchanged() {
        let r = req(&"a".repeat(40), &"a".repeat(40), &"b".repeat(40));
        let d = rebase_against_head(&r, 0).unwrap();
        assert!(matches!(d, RebaseDecision::FastPath { .. }));
    }

    #[test]
    fn reproject_when_queue_head_advanced() {
        let r = req(&"a".repeat(40), &"c".repeat(40), &"b".repeat(40));
        let d = rebase_against_head(&r, 3).unwrap();
        assert_eq!(
            d,
            RebaseDecision::Reproject {
                pr_number: 7,
                new_base_sha: "c".repeat(40),
                new_head_sha: "b".repeat(40),
                skipped_generations: 3,
            }
        );
    }

    #[test]
    fn rejects_invalid_sha() {
        let r = req("bad-sha", &"a".repeat(40), &"b".repeat(40));
        assert_eq!(
            rebase_against_head(&r, 0).unwrap_err(),
            RebaseError::InvalidSha
        );
    }
}
