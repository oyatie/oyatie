//! Fairness — IP-006 §"While one PR is parked, other PRs in the queue
//! continue processing. Parked PRs don't block head; they re-enter at
//! their original position on next successful admission."
//!
//! This module is responsible for picking the next admissible PR from
//! the queue when at least one PR is parked. Pure value type; no I/O.
//!
//! Algorithm (preserves IP-006 acceptance scenario "Three test PRs (A,
//! B, C) admitted in that order; PR A fails CI; PRs B and C are NOT
//! blocked behind A and continue processing"):
//!
//! 1. Iterate the queue in admission order (FIFO).
//! 2. Skip any PR that is currently `Parked` OR `Evicted`.
//! 3. Return the first PR that is `Admissible` (no open fixups, no
//!    pending revalidation).
//! 4. If no PR is admissible, return `Idle`.
//!
//! The scheduler invokes `pick_next_pr` on every tick.

use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueueEntry {
    pub pr_number: u64,
    pub changeset_id: String,
    pub admission_position: u32,
    pub admission_state: AdmissionState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionState {
    /// PR can be admitted this tick (no open fixups, no pending re-CI).
    Admissible,
    /// PR is parked while a fix-loop runs; do not pick.
    Parked,
    /// PR was evicted after exhausting retry budget; do not pick (and
    /// should also be removed from the queue by the scheduler).
    Evicted,
    /// PR has already merged this tick; do not pick again.
    Merged,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NextPick {
    Pick(QueueEntry),
    Idle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FairnessError {
    EmptyQueue,
    DuplicatePosition(u32),
}

impl fmt::Display for FairnessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for FairnessError {}

/// Pick the next admissible PR from the queue.
///
/// The queue is iterated in `admission_position` order (FIFO); parked,
/// evicted, or merged entries are skipped. Returns `Idle` when nothing
/// is admissible (e.g. ALL PRs are parked — the scheduler then waits
/// for fix-loop output).
pub fn pick_next_pr(queue: &[QueueEntry]) -> Result<NextPick, FairnessError> {
    let mut seen_positions = std::collections::BTreeSet::new();
    for entry in queue {
        if !seen_positions.insert(entry.admission_position) {
            return Err(FairnessError::DuplicatePosition(entry.admission_position));
        }
    }
    let mut by_pos: Vec<&QueueEntry> = queue.iter().collect();
    by_pos.sort_by_key(|e| e.admission_position);
    for entry in by_pos {
        if entry.admission_state == AdmissionState::Admissible {
            return Ok(NextPick::Pick(entry.clone()));
        }
    }
    Ok(NextPick::Idle)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(pr: u64, pos: u32, state: AdmissionState) -> QueueEntry {
        QueueEntry {
            pr_number: pr,
            changeset_id: format!("cs_{pr}"),
            admission_position: pos,
            admission_state: state,
        }
    }

    #[test]
    fn fifo_pick_skips_parked_and_evicted() {
        let queue = vec![
            entry(1, 0, AdmissionState::Parked),
            entry(2, 1, AdmissionState::Evicted),
            entry(3, 2, AdmissionState::Admissible),
            entry(4, 3, AdmissionState::Admissible),
        ];
        let pick = pick_next_pr(&queue).unwrap();
        assert_eq!(
            pick,
            NextPick::Pick(QueueEntry {
                pr_number: 3,
                changeset_id: "cs_3".into(),
                admission_position: 2,
                admission_state: AdmissionState::Admissible,
            })
        );
    }

    #[test]
    fn idle_when_all_parked() {
        let queue = vec![
            entry(1, 0, AdmissionState::Parked),
            entry(2, 1, AdmissionState::Parked),
        ];
        assert_eq!(pick_next_pr(&queue).unwrap(), NextPick::Idle);
    }

    #[test]
    fn duplicate_position_rejected() {
        let queue = vec![
            entry(1, 0, AdmissionState::Admissible),
            entry(2, 0, AdmissionState::Admissible),
        ];
        assert_eq!(
            pick_next_pr(&queue).unwrap_err(),
            FairnessError::DuplicatePosition(0)
        );
    }

    #[test]
    fn acceptance_scenario_a_b_c_a_parked_b_and_c_continue() {
        // IP-006 acceptance: "Three test PRs (A, B, C) admitted in that
        // order; PR A fails CI; PRs B and C are NOT blocked behind A and
        // continue processing."
        let queue = vec![
            entry(101, 0, AdmissionState::Parked), // A
            entry(102, 1, AdmissionState::Admissible), // B
            entry(103, 2, AdmissionState::Admissible), // C
        ];
        let pick = pick_next_pr(&queue).unwrap();
        match pick {
            NextPick::Pick(e) => assert_eq!(e.pr_number, 102),
            _ => panic!("expected to pick B"),
        }
    }
}
