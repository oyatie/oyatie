//! Parked-PR state — M01-P17-IP-006 §"Parked-PR state".
//!
//! A parked PR is a queue position that is RESERVED but SKIPPED on each
//! cycle. Held by PR id, not branch SHA, so a new push to the PR branch
//! (typically from the fix-loop in IP-005) preserves the queue slot.
//!
//! Lifecycle:
//!
//! ```text
//!   admitted ── CI/review fail ──▶ Parked(fix-loop iterating)
//!      ▲                                  │
//!      └── fix lands (new head sha) ──────┘  (back to Admitted via speculative_rebase)
//!                                  │
//!                                  └─▶ Evicted (after MAX_ATTEMPTS_PER_PR)
//! ```
//!
//! Per IP-006 §"Concurrent fix-loops permitted": several parked PRs may
//! have fix-loop agents running in parallel. The scheduler serializes only
//! the FINAL landing (one PR merges per tick); other parked PRs stay parked.

use std::fmt;

/// One PR's queue-parked status.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParkedPr {
    pub pr_number: u64,       // data_class: INTERNAL_ONLY
    pub changeset_id: String, // data_class: INTERNAL_ONLY
    /// Original queue position when admitted. Preserved across fix-loop
    /// iterations so the PR can re-enter at its slot on the next
    /// successful admission (IP-006 §"Queue position preserved").
    pub original_queue_position: u32, // data_class: INTERNAL_ONLY
    /// Head SHA at the moment of parking (i.e. the SHA that failed
    /// admission). Used by `speculative_rebase::rebase_against_head` to
    /// detect when the fix-loop has pushed a new tip.
    pub head_sha_at_park: String, // data_class: INTERNAL_ONLY
    /// Base SHA the queue head pointed at when this PR was last admitted.
    /// May be stale once other PRs land while this one is parked — the
    /// scheduler re-rebases against current queue head, not this snapshot.
    pub queue_head_at_park: String, // data_class: INTERNAL_ONLY
    /// Source of the failure that caused parking (mirrors IP-005
    /// dual-source dispatch).
    pub parked_reason: ParkedReason, // data_class: INTERNAL_ONLY
    /// Epoch at which the PR was parked.
    pub parked_at_epoch: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParkedReason {
    CiFailure,
    ReviewChangesRequested,
    SpeculativeRebaseConflict,
}

impl ParkedReason {
    pub fn as_wire(&self) -> &'static str {
        match self {
            ParkedReason::CiFailure => "ci-failure",
            ParkedReason::ReviewChangesRequested => "review-changes-requested",
            ParkedReason::SpeculativeRebaseConflict => "speculative-rebase-conflict",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParkedStateError {
    InvalidPrNumber,
    InvalidChangesetId,
    InvalidSha,
    EpochZero,
    EmptyQueuePosition,
}

impl fmt::Display for ParkedStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for ParkedStateError {}

impl ParkedPr {
    /// Strict, fallible constructor.
    pub fn new(
        pr_number: u64,
        changeset_id: impl Into<String>,
        original_queue_position: u32,
        head_sha_at_park: impl Into<String>,
        queue_head_at_park: impl Into<String>,
        parked_reason: ParkedReason,
        parked_at_epoch: u64,
    ) -> Result<Self, ParkedStateError> {
        if pr_number == 0 {
            return Err(ParkedStateError::InvalidPrNumber);
        }
        if parked_at_epoch == 0 {
            return Err(ParkedStateError::EpochZero);
        }
        let changeset_id = changeset_id.into();
        if !changeset_id.starts_with("cs_") || changeset_id.len() <= 3 {
            return Err(ParkedStateError::InvalidChangesetId);
        }
        let head_sha = head_sha_at_park.into();
        let base_sha = queue_head_at_park.into();
        if !is_sha1_hex(&head_sha) {
            return Err(ParkedStateError::InvalidSha);
        }
        if !is_sha1_hex(&base_sha) {
            return Err(ParkedStateError::InvalidSha);
        }
        Ok(Self {
            pr_number,
            changeset_id,
            original_queue_position,
            head_sha_at_park: head_sha,
            queue_head_at_park: base_sha,
            parked_reason,
            parked_at_epoch,
        })
    }

    /// Did the agent push a new head? Used by the scheduler to decide
    /// whether the PR is ready for speculative-rebase + re-CI.
    pub fn has_new_head(&self, current_head_sha: &str) -> bool {
        !current_head_sha.is_empty() && current_head_sha != self.head_sha_at_park
    }
}

fn is_sha1_hex(value: &str) -> bool {
    value.len() == 40 && value.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_park() -> ParkedPr {
        ParkedPr::new(
            42,
            "cs_test",
            3,
            "1".repeat(40),
            "2".repeat(40),
            ParkedReason::CiFailure,
            1,
        )
        .unwrap()
    }

    #[test]
    fn parked_pr_constructor_rejects_invalid_inputs() {
        assert_eq!(
            ParkedPr::new(
                0,
                "cs_x",
                0,
                "1".repeat(40),
                "2".repeat(40),
                ParkedReason::CiFailure,
                1
            )
            .unwrap_err(),
            ParkedStateError::InvalidPrNumber
        );
        assert_eq!(
            ParkedPr::new(
                1,
                "bad",
                0,
                "1".repeat(40),
                "2".repeat(40),
                ParkedReason::CiFailure,
                1
            )
            .unwrap_err(),
            ParkedStateError::InvalidChangesetId
        );
        assert_eq!(
            ParkedPr::new(
                1,
                "cs_x",
                0,
                "short",
                "2".repeat(40),
                ParkedReason::CiFailure,
                1
            )
            .unwrap_err(),
            ParkedStateError::InvalidSha
        );
        assert_eq!(
            ParkedPr::new(
                1,
                "cs_x",
                0,
                "1".repeat(40),
                "2".repeat(40),
                ParkedReason::CiFailure,
                0
            )
            .unwrap_err(),
            ParkedStateError::EpochZero
        );
    }

    #[test]
    fn has_new_head_detects_fix_landing() {
        let park = ok_park();
        assert!(!park.has_new_head(&park.head_sha_at_park));
        assert!(park.has_new_head(&"f".repeat(40)));
        assert!(!park.has_new_head(""));
    }

    #[test]
    fn parked_reason_wire_strings_are_stable() {
        assert_eq!(ParkedReason::CiFailure.as_wire(), "ci-failure");
        assert_eq!(
            ParkedReason::ReviewChangesRequested.as_wire(),
            "review-changes-requested"
        );
        assert_eq!(
            ParkedReason::SpeculativeRebaseConflict.as_wire(),
            "speculative-rebase-conflict"
        );
    }
}
