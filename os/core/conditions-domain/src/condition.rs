//! The core [`Condition`] abstraction and the polling driver.
//!
//! Mirrors Talos's `conditions.Condition` interface from `pkg/conditions`:
//!
//! ```go
//! type Condition interface {
//!     fmt.Stringer
//!     Wait(ctx context.Context) error
//! }
//! ```
//!
//! In Talos a condition is a predicate the boot sequencer (and service
//! dependency graph) blocks on: "wait until `/var/run/foo.sock` exists",
//! "wait until the `etcd` service is healthy", "wait until the network is
//! ready". `Wait` blocks until the predicate holds or the context is
//! cancelled / times out.
//!
//! Because this crate has no async runtime and no real OS, we model `Wait`
//! as a *polling loop* driven by a [`Clock`] (so tests are deterministic) and
//! a [`Poller`] budget. Each poll asks the condition whether it is currently
//! satisfied via [`Condition::poll`]. The loop is the analogue of Talos's
//! `conditions.PollingCondition` / the `retry` package used under the hood.

use os_kernel::os::Clock;
use os_kernel::{Error, Result};

/// A simulated, advanceable clock used to drive polling loops deterministically.
///
/// `talos-core`'s [`Clock`] is read-only (`now_unix_nanos`). The polling driver
/// needs to *advance* simulated time between attempts (Talos sleeps a backoff
/// interval between polls), so this crate models the OS time boundary with a
/// small advanceable clock. Tests use it directly; production code would back
/// it with a real monotonic clock + sleep.
pub trait WaitClock: Clock {
    /// Advance simulated time by `delta` nanoseconds (a "sleep").
    fn sleep(&self, delta_nanos: u64);
}

/// In-memory advanceable clock for the polling driver and tests.
#[derive(Debug, Default)]
pub struct SimClock {
    nanos: core::cell::Cell<u64>,
}

impl SimClock {
    /// Create a clock pinned at `start_nanos`.
    pub fn new(start_nanos: u64) -> Self {
        SimClock {
            nanos: core::cell::Cell::new(start_nanos),
        }
    }
}

impl Clock for SimClock {
    fn now_unix_nanos(&self) -> u64 {
        self.nanos.get()
    }
}

impl WaitClock for SimClock {
    fn sleep(&self, delta_nanos: u64) {
        self.nanos.set(self.nanos.get().saturating_add(delta_nanos));
    }
}

/// The outcome of a single, non-blocking check of a condition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Poll {
    /// The predicate currently holds; waiting can stop successfully.
    Ready,
    /// The predicate does not hold yet; the driver should poll again later.
    /// The string is the current human-readable status (what we are still
    /// waiting on), matching Talos's `String()` output.
    Pending(String),
    /// The predicate can never become true (permanent failure). Waiting must
    /// abort immediately rather than spin until timeout.
    Failed(Error),
}

impl Poll {
    /// True when the poll reported [`Poll::Ready`].
    pub fn is_ready(&self) -> bool {
        matches!(self, Poll::Ready)
    }
}

/// A predicate the boot sequencer / services wait on.
///
/// Implementations must be cheap to [`poll`](Condition::poll); the driving loop
/// calls it repeatedly. `describe` returns the same kind of string Talos's
/// `Condition.String()` returns and is what the user sees while a service is
/// blocked (e.g. `"service \"etcd\" to be \"up\""`).
pub trait Condition {
    /// Perform a single, non-blocking check.
    fn poll(&self) -> Poll;

    /// Human-readable description of *what is being waited for*.
    fn describe(&self) -> String;

    /// Block until the condition is satisfied, fails, or the budget is
    /// exhausted, using `clock` for deterministic time accounting.
    ///
    /// This is the Rust analogue of Talos's `Condition.Wait(ctx)`.
    fn wait(&self, clock: &dyn WaitClock, budget: Poller) -> Result<WaitReport> {
        budget.run(clock, self)
    }
}

/// Budget / cadence for [`Condition::wait`].
///
/// Talos uses `context.WithTimeout` plus a backoff ticker. We capture the same
/// idea concretely: a maximum number of polls, the simulated interval between
/// polls (advanced on the [`Clock`]), and an overall deadline in nanoseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Poller {
    /// Maximum number of poll attempts before giving up with a timeout.
    pub max_attempts: u32,
    /// Simulated wait between attempts, in nanoseconds. Advances the clock.
    pub interval_nanos: u64,
    /// Overall deadline relative to the clock at `run` start, in nanoseconds.
    /// `None` means "bounded only by `max_attempts`".
    pub timeout_nanos: Option<u64>,
}

impl Poller {
    /// A poller that checks `max_attempts` times, sleeping `interval_nanos`
    /// between attempts, with no separate wall-clock deadline.
    pub fn new(max_attempts: u32, interval_nanos: u64) -> Self {
        Poller {
            max_attempts,
            interval_nanos,
            timeout_nanos: Option::None,
        }
    }

    /// Add an overall wall-clock deadline (in nanoseconds from start).
    pub fn with_timeout(mut self, timeout_nanos: u64) -> Self {
        self.timeout_nanos = Some(timeout_nanos);
        self
    }

    /// A single-shot poller: check exactly once, never sleep. Useful when the
    /// caller just wants the current status without blocking.
    pub fn once() -> Self {
        Poller::new(1, 0)
    }

    /// Drive `cond` to completion under this budget.
    pub fn run(
        &self,
        clock: &dyn WaitClock,
        cond: &(impl Condition + ?Sized),
    ) -> Result<WaitReport> {
        if self.max_attempts == 0 {
            return Err(Error::invalid("poller max_attempts must be >= 1"));
        }
        let start = clock.now_unix_nanos();
        let mut last_status = cond.describe();

        for attempt in 1..=self.max_attempts {
            match cond.poll() {
                Poll::Ready => {
                    return Ok(WaitReport {
                        attempts: attempt,
                        elapsed_nanos: clock.now_unix_nanos().saturating_sub(start),
                    });
                }
                Poll::Pending(status) => last_status = status,
                Poll::Failed(err) => return Err(err),
            }

            // Deadline check happens before we sleep the next interval.
            if let Some(timeout) = self.timeout_nanos {
                let elapsed = clock.now_unix_nanos().saturating_sub(start);
                if elapsed >= timeout {
                    return Err(Error::Timeout);
                }
            }

            if attempt < self.max_attempts {
                clock.sleep(self.interval_nanos);
            }
        }

        // Exhausted all attempts without becoming ready.
        let _ = last_status;
        Err(Error::Timeout)
    }
}

/// Bookkeeping returned by a successful [`Condition::wait`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaitReport {
    /// Number of poll attempts that were made (>= 1).
    pub attempts: u32,
    /// Simulated elapsed time, in nanoseconds.
    pub elapsed_nanos: u64,
}

/// A condition that is always immediately satisfied.
///
/// Equivalent to Talos's `conditions.None()`, used as a placeholder dependency.
#[derive(Debug, Clone, Copy, Default)]
pub struct None;

impl Condition for None {
    fn poll(&self) -> Poll {
        Poll::Ready
    }

    fn describe(&self) -> String {
        "no condition".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    /// A condition that becomes ready once the *shared* clock reaches `ready_at`.
    struct ReadyAfter {
        clock: Rc<SimClock>,
        ready_at: u64,
    }

    impl Condition for ReadyAfter {
        fn poll(&self) -> Poll {
            if self.clock.now_unix_nanos() >= self.ready_at {
                Poll::Ready
            } else {
                Poll::Pending("not yet".to_string())
            }
        }
        fn describe(&self) -> String {
            "ready-after".to_string()
        }
    }

    struct AlwaysFails;
    impl Condition for AlwaysFails {
        fn poll(&self) -> Poll {
            Poll::Failed(Error::not_found("gone"))
        }
        fn describe(&self) -> String {
            "always-fails".to_string()
        }
    }

    #[test]
    fn none_is_always_ready() {
        let clock = SimClock::new(0);
        let report = None.wait(&clock, Poller::once()).unwrap();
        assert_eq!(report.attempts, 1);
        assert!(None.poll().is_ready());
        assert_eq!(None.describe(), "no condition");
    }

    #[test]
    fn pending_then_ready_after_sleeps() {
        let clock = Rc::new(SimClock::new(0));
        let cond = ReadyAfter {
            clock: clock.clone(),
            ready_at: 30,
        };
        // interval 10: attempt1@0 pending, sleep->10, attempt2@10 pending,
        // sleep->20, attempt3@20 pending, sleep->30, attempt4@30 ready.
        let report = cond.wait(clock.as_ref(), Poller::new(5, 10)).unwrap();
        assert_eq!(report.attempts, 4);
        assert_eq!(report.elapsed_nanos, 30);
    }

    #[test]
    fn exhausting_attempts_times_out() {
        let clock = Rc::new(SimClock::new(0));
        let cond = ReadyAfter {
            clock: clock.clone(),
            ready_at: u64::MAX,
        };
        let err = cond.wait(clock.as_ref(), Poller::new(3, 5)).unwrap_err();
        assert_eq!(err, Error::Timeout);
        // Two intervals advanced between three attempts.
        assert_eq!(clock.now_unix_nanos(), 10);
    }

    #[test]
    fn permanent_failure_aborts_immediately() {
        let clock = SimClock::new(0);
        let err = AlwaysFails.wait(&clock, Poller::new(100, 5)).unwrap_err();
        assert_eq!(err.kind(), "not_found");
        // No time should have been spent sleeping.
        assert_eq!(clock.now_unix_nanos(), 0);
    }

    #[test]
    fn zero_attempts_is_invalid() {
        let clock = SimClock::new(0);
        let err = None.wait(&clock, Poller::new(0, 1)).unwrap_err();
        assert_eq!(err.kind(), "invalid");
    }

    #[test]
    fn deadline_trips_before_attempts() {
        let clock = Rc::new(SimClock::new(0));
        let cond = ReadyAfter {
            clock: clock.clone(),
            ready_at: u64::MAX,
        };
        // interval 100, timeout 50 -> after first pending poll elapsed 0,
        // sleep 100; second attempt elapsed 100 >= 50 -> Timeout.
        let err = cond
            .wait(clock.as_ref(), Poller::new(10, 100).with_timeout(50))
            .unwrap_err();
        assert_eq!(err, Error::Timeout);
    }
}
