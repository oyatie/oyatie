//! Merge-queue scheduler — IP-006 §"Convergence proof: scheduler emits
//! an admission-log entry per cycle so an external observer can verify
//! forward progress (no livelock)."
//!
//! This is the top-level state machine that wires together
//! [`parked_state`](crate::parked_state),
//! [`pr_retry_budget`](crate::pr_retry_budget),
//! [`speculative_rebase`](crate::speculative_rebase), and
//! [`fairness`](crate::fairness). The kernel is pure; the integration
//! crate (`tools/oya-foundry-vcs-merge-queue-fix-loop-app`) provides the
//! event consumer that drives ticks from the actual GitHub +
//! `pr-review-approved` / `pr-review-fix-requested` events.
//!
//! Per IP-006 §"Dependencies": this scheduler is the consumer side for
//! both IP-004's `pr-review-approved` event AND IP-005's fix-loop output.

use std::collections::BTreeMap;
use std::fmt;

use crate::fairness::{AdmissionState, NextPick, QueueEntry, pick_next_pr};
use crate::parked_state::{ParkedPr, ParkedReason};
use crate::pr_retry_budget::{BudgetVerdict, PrBudget};
use crate::speculative_rebase::{RebaseDecision, RebaseError, RebaseRequest, rebase_against_head};

pub const SCHEDULER_SCHEMA_VERSION: u32 = 1;

/// Top-level merge-queue scheduler state.
#[derive(Clone, Debug, Default)]
pub struct Scheduler {
    queue: Vec<QueueEntry>,
    parked: BTreeMap<u64, ParkedPr>,
    budget: PrBudget,
    current_head_sha: String,
    merged_generations: u32,
    parked_generation_at: BTreeMap<u64, u32>,
    tick_log: Vec<TickEntry>,
}

/// One scheduler tick's audit record. Appended to
/// `registry/merge-queue-tick-log.json::entries` so an
/// external observer can verify forward progress per IP-006 §"Convergence
/// proof".
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TickEntry {
    pub tick_number: u32,         // data_class: INTERNAL_ONLY
    pub action: TickAction,       // data_class: INTERNAL_ONLY
    pub current_head_sha: String, // data_class: INTERNAL_ONLY
    pub epoch: u64,               // data_class: INTERNAL_ONLY
    pub queue_depth: u32,         // data_class: INTERNAL_ONLY
    pub parked_count: u32,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TickAction {
    AdmitPr {
        pr_number: u64,
        changeset_id: String,
    },
    MergePr {
        pr_number: u64,
    },
    ParkPr {
        pr_number: u64,
        reason: ParkedReason,
    },
    RevalidateParkedPr {
        pr_number: u64,
        rebase_decision: RebaseDecision,
        attempts_used: u32,
    },
    EvictPr {
        pr_number: u64,
        attempts_used: u32,
    },
    Idle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SchedulerError {
    PrAlreadyAdmitted(u64),
    PrNotInQueue(u64),
    PrNotParked(u64),
    DuplicateQueuePosition(u32),
    InvalidSha,
    EpochZero,
    Rebase(RebaseError),
}

impl fmt::Display for SchedulerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for SchedulerError {}

impl From<RebaseError> for SchedulerError {
    fn from(value: RebaseError) -> Self {
        SchedulerError::Rebase(value)
    }
}

impl Scheduler {
    pub fn new(initial_head_sha: impl Into<String>) -> Result<Self, SchedulerError> {
        let head = initial_head_sha.into();
        if !is_sha1_hex(&head) {
            return Err(SchedulerError::InvalidSha);
        }
        Ok(Self {
            queue: Vec::new(),
            parked: BTreeMap::new(),
            budget: PrBudget::new(),
            current_head_sha: head,
            merged_generations: 0,
            parked_generation_at: BTreeMap::new(),
            tick_log: Vec::new(),
        })
    }

    pub fn current_head_sha(&self) -> &str {
        &self.current_head_sha
    }

    pub fn queue_depth(&self) -> u32 {
        self.queue.len() as u32
    }

    pub fn parked_count(&self) -> u32 {
        self.parked.len() as u32
    }

    pub fn tick_log(&self) -> &[TickEntry] {
        &self.tick_log
    }

    pub fn is_parked(&self, pr_number: u64) -> bool {
        self.parked.contains_key(&pr_number)
    }

    pub fn is_evicted(&self, pr_number: u64) -> bool {
        self.budget.is_evicted(pr_number)
    }

    /// Append a PR onto the queue at the next admission position.
    ///
    /// Consumed by the integration crate when it receives a
    /// `pr-review-approved` event from IP-004.
    pub fn admit(
        &mut self,
        pr_number: u64,
        changeset_id: impl Into<String>,
    ) -> Result<(), SchedulerError> {
        let changeset_id = changeset_id.into();
        if self.queue.iter().any(|e| e.pr_number == pr_number) {
            return Err(SchedulerError::PrAlreadyAdmitted(pr_number));
        }
        let next_pos = self
            .queue
            .iter()
            .map(|e| e.admission_position)
            .max()
            .map(|m| m + 1)
            .unwrap_or(0);
        self.queue.push(QueueEntry {
            pr_number,
            changeset_id,
            admission_position: next_pos,
            admission_state: AdmissionState::Admissible,
        });
        Ok(())
    }

    /// Park a PR after a failed admission (CI failure or review
    /// changes-requested). Preserves queue position (PR not evicted, not
    /// removed from queue).
    pub fn park(
        &mut self,
        pr_number: u64,
        head_sha_at_park: impl Into<String>,
        reason: ParkedReason,
        now_epoch: u64,
    ) -> Result<(), SchedulerError> {
        if now_epoch == 0 {
            return Err(SchedulerError::EpochZero);
        }
        let head_sha = head_sha_at_park.into();
        if !is_sha1_hex(&head_sha) {
            return Err(SchedulerError::InvalidSha);
        }
        let entry = self
            .queue
            .iter_mut()
            .find(|e| e.pr_number == pr_number)
            .ok_or(SchedulerError::PrNotInQueue(pr_number))?;
        entry.admission_state = AdmissionState::Parked;
        let queue_position = entry.admission_position;
        let changeset_id = entry.changeset_id.clone();
        let park = ParkedPr::new(
            pr_number,
            changeset_id,
            queue_position,
            head_sha,
            self.current_head_sha.clone(),
            reason,
            now_epoch,
        )
        .map_err(|_| SchedulerError::InvalidSha)?;
        self.parked.insert(pr_number, park);
        self.parked_generation_at
            .insert(pr_number, self.merged_generations);
        Ok(())
    }

    /// React to fix-loop output (new head pushed on PR branch).
    ///
    /// Driven by the IP-005 dispatcher's bundle-emit + subsequent
    /// successful CI on the new head. Returns the speculative-rebase
    /// decision so the integration crate can ask Oya VCS / git to do the
    /// physical work.
    pub fn revalidate_parked(
        &mut self,
        pr_number: u64,
        new_head_sha: impl Into<String>,
    ) -> Result<(RebaseDecision, BudgetVerdict), SchedulerError> {
        let new_head_sha = new_head_sha.into();
        if !new_head_sha.is_empty() && !is_sha1_hex(&new_head_sha) {
            return Err(SchedulerError::InvalidSha);
        }
        let park = self
            .parked
            .get(&pr_number)
            .ok_or(SchedulerError::PrNotParked(pr_number))?
            .clone();
        let parked_at_generation = self
            .parked_generation_at
            .get(&pr_number)
            .copied()
            .unwrap_or(0);
        let generations_advanced = self.merged_generations.saturating_sub(parked_at_generation);
        let decision = rebase_against_head(
            &RebaseRequest {
                pr_number,
                pr_new_head_sha: new_head_sha,
                queue_head_at_park: park.queue_head_at_park,
                current_queue_head_sha: self.current_head_sha.clone(),
            },
            generations_advanced,
        )?;
        let verdict = self
            .budget
            .register_revalidation(pr_number)
            .map_err(|_| SchedulerError::PrNotInQueue(pr_number))?;
        match verdict {
            BudgetVerdict::EvictWithEscalation { .. } | BudgetVerdict::AlreadyEvicted { .. } => {
                self.evict_internal(pr_number);
            }
            BudgetVerdict::Proceed { .. } => {
                if let Some(entry) = self.queue.iter_mut().find(|e| e.pr_number == pr_number) {
                    entry.admission_state = AdmissionState::Admissible;
                }
                self.parked.remove(&pr_number);
                self.parked_generation_at.remove(&pr_number);
            }
        }
        Ok((decision, verdict))
    }

    /// Process one scheduler tick.
    ///
    /// Picks the next admissible PR (skipping parked + evicted), merges
    /// it (advances head), and appends a tick log entry. Returns the
    /// action taken for the convergence-proof log.
    pub fn tick(&mut self, now_epoch: u64) -> Result<TickAction, SchedulerError> {
        if now_epoch == 0 {
            return Err(SchedulerError::EpochZero);
        }
        let action = match pick_next_pr(&self.queue)
            .map_err(|_| SchedulerError::DuplicateQueuePosition(self.queue.len() as u32))?
        {
            NextPick::Pick(entry) => {
                let pr_number = entry.pr_number;
                let cs = entry.changeset_id.clone();
                self.merge_internal(pr_number);
                TickAction::MergePr { pr_number }
                    .with_admit_when_first(&entry)
                    .unwrap_or(TickAction::MergePr { pr_number });
                self.tick_log.push(TickEntry {
                    tick_number: self.tick_log.len() as u32 + 1,
                    action: TickAction::MergePr { pr_number },
                    current_head_sha: self.current_head_sha.clone(),
                    epoch: now_epoch,
                    queue_depth: self.queue_depth(),
                    parked_count: self.parked_count(),
                });
                let _ = cs; // keep cs accessible if extended later
                TickAction::MergePr { pr_number }
            }
            NextPick::Idle => {
                self.tick_log.push(TickEntry {
                    tick_number: self.tick_log.len() as u32 + 1,
                    action: TickAction::Idle,
                    current_head_sha: self.current_head_sha.clone(),
                    epoch: now_epoch,
                    queue_depth: self.queue_depth(),
                    parked_count: self.parked_count(),
                });
                TickAction::Idle
            }
        };
        Ok(action)
    }

    /// Advance the queue head after an external commit lands (e.g. main
    /// fast-forwards because some non-queue commit landed). Each call
    /// increments `merged_generations` so parked PRs know how far they
    /// fell behind.
    pub fn advance_head(&mut self, new_head_sha: impl Into<String>) -> Result<(), SchedulerError> {
        let new_head_sha = new_head_sha.into();
        if !is_sha1_hex(&new_head_sha) {
            return Err(SchedulerError::InvalidSha);
        }
        if new_head_sha != self.current_head_sha {
            self.current_head_sha = new_head_sha;
            self.merged_generations += 1;
        }
        Ok(())
    }

    fn merge_internal(&mut self, pr_number: u64) {
        // Update queue entry state + bump generations.
        if let Some(entry) = self.queue.iter_mut().find(|e| e.pr_number == pr_number) {
            entry.admission_state = AdmissionState::Merged;
        }
        self.budget.mark_merged(pr_number);
        self.merged_generations += 1;
    }

    fn evict_internal(&mut self, pr_number: u64) {
        if let Some(entry) = self.queue.iter_mut().find(|e| e.pr_number == pr_number) {
            entry.admission_state = AdmissionState::Evicted;
        }
        self.parked.remove(&pr_number);
        self.parked_generation_at.remove(&pr_number);
    }
}

impl TickAction {
    /// Stub helper retained for future composition of multi-action ticks.
    fn with_admit_when_first(self, _entry: &QueueEntry) -> Option<TickAction> {
        Some(self)
    }
}

fn is_sha1_hex(value: &str) -> bool {
    value.len() == 40 && value.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sha(byte: char) -> String {
        std::iter::repeat_n(byte, 40).collect()
    }

    #[test]
    fn admit_appends_with_incrementing_position() {
        let mut s = Scheduler::new(sha('0')).unwrap();
        s.admit(101, "cs_a").unwrap();
        s.admit(102, "cs_b").unwrap();
        assert_eq!(s.queue_depth(), 2);
        assert_eq!(s.queue[0].admission_position, 0);
        assert_eq!(s.queue[1].admission_position, 1);
    }

    #[test]
    fn admit_rejects_duplicate_pr_number() {
        let mut s = Scheduler::new(sha('0')).unwrap();
        s.admit(101, "cs_a").unwrap();
        assert_eq!(
            s.admit(101, "cs_a"),
            Err(SchedulerError::PrAlreadyAdmitted(101))
        );
    }

    #[test]
    fn park_preserves_queue_position_and_skips_in_picking() {
        let mut s = Scheduler::new(sha('0')).unwrap();
        s.admit(101, "cs_a").unwrap();
        s.admit(102, "cs_b").unwrap();
        s.admit(103, "cs_c").unwrap();
        s.park(101, sha('a'), ParkedReason::CiFailure, 5).unwrap();
        // tick: B should merge first (A is parked, B/C are admissible)
        let action = s.tick(10).unwrap();
        assert_eq!(action, TickAction::MergePr { pr_number: 102 });
        assert!(s.is_parked(101));
    }

    #[test]
    fn revalidate_parked_proceeds_until_budget_exhaustion() {
        let mut s = Scheduler::new(sha('0')).unwrap();
        s.admit(101, "cs_a").unwrap();
        s.park(101, sha('a'), ParkedReason::CiFailure, 1).unwrap();
        for i in 1..crate::pr_retry_budget::MAX_ATTEMPTS_PER_PR {
            let (_decision, verdict) = s.revalidate_parked(101, sha('b')).unwrap();
            assert_eq!(verdict, BudgetVerdict::Proceed { attempts_used: i });
            // Re-park to try again
            s.park(101, sha('a'), ParkedReason::CiFailure, i as u64 + 1)
                .unwrap();
        }
        let (_decision, verdict) = s.revalidate_parked(101, sha('b')).unwrap();
        assert_eq!(
            verdict,
            BudgetVerdict::EvictWithEscalation {
                attempts_used: crate::pr_retry_budget::MAX_ATTEMPTS_PER_PR
            }
        );
        assert!(s.is_evicted(101));
        assert!(!s.is_parked(101));
    }

    #[test]
    fn speculative_rebase_reprojects_when_head_advanced() {
        let mut s = Scheduler::new(sha('0')).unwrap();
        s.admit(101, "cs_a").unwrap();
        s.admit(102, "cs_b").unwrap();
        s.park(101, sha('a'), ParkedReason::CiFailure, 1).unwrap();
        // PR 102 merges → head advances
        s.tick(2).unwrap();
        s.advance_head(sha('c')).unwrap();
        let (decision, _verdict) = s.revalidate_parked(101, sha('d')).unwrap();
        match decision {
            RebaseDecision::Reproject {
                pr_number,
                skipped_generations,
                ..
            } => {
                assert_eq!(pr_number, 101);
                assert!(skipped_generations >= 1);
            }
            other => panic!("expected Reproject, got {other:?}"),
        }
    }

    #[test]
    fn idle_tick_logged_when_all_parked() {
        let mut s = Scheduler::new(sha('0')).unwrap();
        s.admit(101, "cs_a").unwrap();
        s.park(101, sha('a'), ParkedReason::CiFailure, 1).unwrap();
        let action = s.tick(2).unwrap();
        assert_eq!(action, TickAction::Idle);
        let log = s.tick_log();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].action, TickAction::Idle);
        assert_eq!(log[0].parked_count, 1);
    }

    #[test]
    fn concurrent_fix_loops_converge_one_merges_other_re_parks() {
        // IP-006 acceptance: "Concurrent fix-loops: PRs A and D both parked
        // simultaneously; both fix-loops run; one merges, the other re-parks.
        // Scheduler does not deadlock."
        let mut s = Scheduler::new(sha('0')).unwrap();
        s.admit(101, "cs_a").unwrap();
        s.admit(104, "cs_d").unwrap();
        s.park(101, sha('a'), ParkedReason::CiFailure, 1).unwrap();
        s.park(104, sha('a'), ParkedReason::CiFailure, 1).unwrap();
        // Both fix-loops finish; A's new head passes CI but D's still fails.
        let (_da, _va) = s.revalidate_parked(101, sha('b')).unwrap();
        let action = s.tick(2).unwrap();
        // A admissible now — merges.
        assert_eq!(action, TickAction::MergePr { pr_number: 101 });
        // D remains parked; not deadlocked
        assert!(s.is_parked(104));
        assert_eq!(s.parked_count(), 1);
    }
}
