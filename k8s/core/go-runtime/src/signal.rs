//! `chan struct{}` — a channel that carries no payload and whose only event is being closed.
//!
//! 179 of 400 created channels in the measured corpus (44.8%) are payload-free, so this is a large
//! and distinct shape rather than a special case of [`crate::Chan`]. It gets its own type because
//! the two have different rules: a signal is BROADCAST and idempotent — every waiter observes it,
//! and observing it twice is the same as observing it once — where a value channel delivers each
//! value to exactly one receiver.
//!
//! Modelling a signal as `Chan<()>` would compile and would be wrong in the direction that is
//! hardest to notice: `close(stopCh)` releasing only one of five waiting goroutines is a shutdown
//! that hangs, and it hangs in production rather than in a test.

use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::Duration;

struct Shared {
    fired: Mutex<bool>,
    changed: Condvar,
}

/// A `chan struct{}` used as a one-way broadcast: the `stopCh` convention, 340 parameter positions
/// in the measured corpus.
pub struct Signal;

impl Signal {
    /// Create the pair. The trigger fires it; every waiter observes it.
    ///
    /// Named `pair` rather than `new` because it does not construct a `Signal` — the type is the
    /// namespace for a two-ended thing, the way the source's `make(chan struct{})` produces one
    /// value that two goroutines hold from opposite ends.
    #[must_use]
    pub fn pair() -> (Trigger, Waiter) {
        let shared = Arc::new(Shared {
            fired: Mutex::new(false),
            changed: Condvar::new(),
        });
        (
            Trigger {
                shared: Arc::clone(&shared),
            },
            Waiter { shared },
        )
    }
}

/// The closing half of a signal channel.
pub struct Trigger {
    shared: Arc<Shared>,
}

impl Clone for Trigger {
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl Trigger {
    /// `close(stopCh)` — wakes every waiter, now and forever after.
    ///
    /// Idempotent, where the source panics on a second close. A library that aborts the process
    /// takes that decision away from the ported program; the rule pack owns how a panic translates.
    pub fn fire(&self) {
        let mut fired = self
            .shared
            .fired
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        *fired = true;
        drop(fired);
        self.shared.changed.notify_all();
    }
}

/// The receiving half of a signal channel.
pub struct Waiter {
    shared: Arc<Shared>,
}

impl Clone for Waiter {
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl Waiter {
    /// `<-stopCh` — block until the signal fires.
    pub fn wait(&self) {
        let mut fired = self
            .shared
            .fired
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        while !*fired {
            fired = self
                .shared
                .changed
                .wait(fired)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }

    /// Block until the signal fires or the timeout elapses. `true` when it fired.
    ///
    /// This is the `select { case <-stopCh: ...; case <-time.After(d): ... }` shape, which is what
    /// every background loop in the corpus is built out of — collapsed into one operation so the
    /// two-branch select does not need a translation rule per site.
    #[must_use]
    pub fn wait_timeout(&self, timeout: Duration) -> bool {
        let deadline = std::time::Instant::now() + timeout;
        let mut fired = self
            .shared
            .fired
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        while !*fired {
            let Some(remaining) = deadline.checked_duration_since(std::time::Instant::now()) else {
                return false;
            };
            let (guard, timed_out) = self
                .shared
                .changed
                .wait_timeout(fired, remaining)
                .unwrap_or_else(PoisonError::into_inner);
            fired = guard;
            if timed_out.timed_out() && !*fired {
                return false;
            }
        }
        true
    }

    /// The non-blocking check: `select { case <-stopCh: ...; default: ... }`.
    #[must_use]
    pub fn has_fired(&self) -> bool {
        *self
            .shared
            .fired
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}
