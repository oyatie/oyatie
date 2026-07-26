//! Child reaping — the classic PID 1 duty.
//!
//! When a process exits, its parent must `wait(2)` for it to release the
//! zombie. Orphaned processes are re-parented to PID 1, so PID 1 must reap them
//! or the process table fills with zombies. Talos' init does this in a loop
//! driven by `SIGCHLD`.
//!
//! Here we model the reaping *logic* behind a [`ChildWaiter`] trait that stands
//! in for `waitpid(-1, WNOHANG)`. The Linux binary backs it with a real
//! `waitpid`; tests back it with [`FakeWaiter`], a deterministic queue of
//! pending child exits.

use std::fmt;

/// How a child terminated, decoded from a `wait(2)` status word.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExitReason {
    /// Exited normally with this code (`WIFEXITED` / `WEXITSTATUS`).
    Exited(i32),
    /// Killed by this signal (`WIFSIGNALED` / `WTERMSIG`).
    Signaled(i32),
}

impl ExitReason {
    /// Decode a raw `wait(2)` status word the way `WIFEXITED`/`WIFSIGNALED` do.
    /// On Linux: low 7 bits are the term signal, 0 if exited normally; the next
    /// byte is the exit code when exited normally.
    pub fn from_raw_status(status: i32) -> ExitReason {
        let term_sig = status & 0x7f;
        if term_sig == 0 {
            // Exited normally: WEXITSTATUS == (status >> 8) & 0xff
            ExitReason::Exited((status >> 8) & 0xff)
        } else {
            // Signaled. (0x7f would be "stopped"; we treat it as signaled here.)
            ExitReason::Signaled(term_sig)
        }
    }

    /// True if the child exited cleanly with code 0.
    pub fn is_success(&self) -> bool {
        matches!(self, ExitReason::Exited(0))
    }
}

impl fmt::Display for ExitReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExitReason::Exited(code) => write!(f, "exited with code {code}"),
            ExitReason::Signaled(sig) => write!(f, "killed by signal {sig}"),
        }
    }
}

/// A reaped child: its PID and how it died.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Reaped {
    pub pid: i32,
    pub reason: ExitReason,
}

/// One non-blocking poll of `waitpid(-1, WNOHANG)`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WaitResult {
    /// A child was reaped.
    Reaped(Reaped),
    /// Children exist but none have exited yet (`waitpid` returned 0).
    NoneReady,
    /// No children remain (`ECHILD`).
    NoChildren,
}

/// Abstraction over `waitpid(-1, &status, WNOHANG)`.
pub trait ChildWaiter {
    /// Non-blocking reap of one child.
    fn try_wait(&mut self) -> WaitResult;
}

/// Drain all currently-exited children in a loop. Returns the list of reaped
/// children. Stops as soon as the waiter reports `NoneReady` or `NoChildren`,
/// exactly like a `SIGCHLD` handler should (reap until `WNOHANG` yields nothing).
pub fn reap_all(waiter: &mut dyn ChildWaiter) -> Vec<Reaped> {
    let mut reaped = Vec::new();
    while let WaitResult::Reaped(r) = waiter.try_wait() {
        reaped.push(r);
    }
    reaped
}

/// Running tally of reaping activity, useful for diagnostics/logging.
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct ReapStats {
    pub total: u64,
    pub clean_exits: u64,
    pub failed_exits: u64,
    pub signaled: u64,
}

impl ReapStats {
    /// Fold a batch of reaped children into the stats.
    pub fn record(&mut self, batch: &[Reaped]) {
        for r in batch {
            self.total += 1;
            match r.reason {
                ExitReason::Exited(0) => self.clean_exits += 1,
                ExitReason::Exited(_) => self.failed_exits += 1,
                ExitReason::Signaled(_) => self.signaled += 1,
            }
        }
    }

    /// True if every child reaped so far exited cleanly.
    pub fn all_clean(&self) -> bool {
        self.failed_exits == 0 && self.signaled == 0
    }
}

/// Deterministic in-memory [`ChildWaiter`] for tests. Pops queued exits until
/// empty, then reports `NoChildren`. A `none_ready_after` cursor can interleave
/// a `NoneReady` (to simulate "children still running") before the queue drains.
pub struct FakeWaiter {
    pending: std::collections::VecDeque<Reaped>,
    /// If set, return `NoneReady` once when this many have been reaped, then
    /// continue. Models a child that hasn't exited yet on this poll.
    none_ready_after: Option<usize>,
    reaped_count: usize,
    none_ready_emitted: bool,
    /// What to report when the queue is empty.
    empty_result: WaitResult,
}

impl FakeWaiter {
    pub fn new(exits: Vec<Reaped>) -> Self {
        FakeWaiter {
            pending: exits.into_iter().collect(),
            none_ready_after: None,
            reaped_count: 0,
            none_ready_emitted: false,
            empty_result: WaitResult::NoChildren,
        }
    }

    /// Build from raw `(pid, status_word)` pairs, decoding each status.
    pub fn from_statuses(pairs: &[(i32, i32)]) -> Self {
        let exits = pairs
            .iter()
            .map(|(pid, status)| Reaped {
                pid: *pid,
                reason: ExitReason::from_raw_status(*status),
            })
            .collect();
        FakeWaiter::new(exits)
    }

    /// Inject a single `NoneReady` once after `n` children have been reaped,
    /// then continue draining the queue.
    pub fn with_none_ready_after(mut self, n: usize) -> Self {
        self.none_ready_after = Some(n);
        self
    }

    /// Report `NoneReady` (rather than `NoChildren`) when the queue is empty,
    /// modeling long-lived children that simply have not exited.
    pub fn empty_as_none_ready(mut self) -> Self {
        self.empty_result = WaitResult::NoneReady;
        self
    }
}

impl ChildWaiter for FakeWaiter {
    fn try_wait(&mut self) -> WaitResult {
        if let Some(n) = self.none_ready_after
            && !self.none_ready_emitted
            && self.reaped_count == n
            && !self.pending.is_empty()
        {
            self.none_ready_emitted = true;
            return WaitResult::NoneReady;
        }
        match self.pending.pop_front() {
            Some(r) => {
                self.reaped_count += 1;
                WaitResult::Reaped(r)
            }
            None => self.empty_result,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_exited_status() {
        // exit code 42, normal exit: (42 << 8) | 0
        let r = ExitReason::from_raw_status(42 << 8);
        assert_eq!(r, ExitReason::Exited(42));
        assert!(!r.is_success());
    }

    #[test]
    fn decode_clean_exit() {
        assert_eq!(ExitReason::from_raw_status(0), ExitReason::Exited(0));
        assert!(ExitReason::from_raw_status(0).is_success());
    }

    #[test]
    fn decode_signaled_status() {
        // killed by SIGKILL (9): low 7 bits = 9
        let r = ExitReason::from_raw_status(9);
        assert_eq!(r, ExitReason::Signaled(9));
    }

    #[test]
    fn exit_reason_display() {
        assert_eq!(ExitReason::Exited(3).to_string(), "exited with code 3");
        assert_eq!(ExitReason::Signaled(15).to_string(), "killed by signal 15");
    }

    #[test]
    fn reap_all_drains_queue() {
        let mut w = FakeWaiter::new(vec![
            Reaped {
                pid: 10,
                reason: ExitReason::Exited(0),
            },
            Reaped {
                pid: 11,
                reason: ExitReason::Signaled(9),
            },
        ]);
        let reaped = reap_all(&mut w);
        assert_eq!(reaped.len(), 2);
        assert_eq!(reaped[0].pid, 10);
        assert_eq!(reaped[1].reason, ExitReason::Signaled(9));
    }

    #[test]
    fn reap_all_empty_returns_nothing() {
        let mut w = FakeWaiter::new(vec![]);
        assert!(reap_all(&mut w).is_empty());
    }

    #[test]
    fn reap_stops_at_none_ready() {
        // Two queued, but NoneReady injected after the first — reap_all should
        // stop after reaping just one.
        let mut w = FakeWaiter::new(vec![
            Reaped {
                pid: 1,
                reason: ExitReason::Exited(0),
            },
            Reaped {
                pid: 2,
                reason: ExitReason::Exited(0),
            },
        ])
        .with_none_ready_after(1);
        let reaped = reap_all(&mut w);
        assert_eq!(reaped.len(), 1);
        assert_eq!(reaped[0].pid, 1);
    }

    #[test]
    fn from_statuses_decodes_each() {
        let mut w = FakeWaiter::from_statuses(&[(100, 0), (101, 5 << 8), (102, 6)]);
        let reaped = reap_all(&mut w);
        assert_eq!(reaped[0].reason, ExitReason::Exited(0));
        assert_eq!(reaped[1].reason, ExitReason::Exited(5));
        assert_eq!(reaped[2].reason, ExitReason::Signaled(6));
    }

    #[test]
    fn stats_classify_outcomes() {
        let mut w = FakeWaiter::from_statuses(&[(1, 0), (2, 7 << 8), (3, 9), (4, 0)]);
        let reaped = reap_all(&mut w);
        let mut stats = ReapStats::default();
        stats.record(&reaped);
        assert_eq!(stats.total, 4);
        assert_eq!(stats.clean_exits, 2);
        assert_eq!(stats.failed_exits, 1);
        assert_eq!(stats.signaled, 1);
        assert!(!stats.all_clean());
    }

    #[test]
    fn stats_all_clean_when_only_zero_exits() {
        let mut w = FakeWaiter::from_statuses(&[(1, 0), (2, 0)]);
        let reaped = reap_all(&mut w);
        let mut stats = ReapStats::default();
        stats.record(&reaped);
        assert!(stats.all_clean());
    }

    #[test]
    fn empty_as_none_ready_still_drains_then_stops() {
        let mut w = FakeWaiter::new(vec![Reaped {
            pid: 5,
            reason: ExitReason::Exited(0),
        }])
        .empty_as_none_ready();
        let reaped = reap_all(&mut w);
        assert_eq!(reaped.len(), 1);
        // Second poll yields NoneReady -> loop stops, no infinite loop.
    }
}
