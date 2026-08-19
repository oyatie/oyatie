//! `sync.WaitGroup` and `sync.Once`.
//!
//! 218 `WaitGroup` declarations and 241 `Once` declarations in the measured corpus, against 2
//! `errgroup` call sites total — so these two are the join and the singleton-init vocabulary of
//! this corpus, and `errgroup` is vestigial.
//!
//! Neither is a thin wrapper over a standard type. `WaitGroup` is a counter any goroutine may add
//! to and any goroutine may wait on, which `JoinHandle` does not model — a fan-out that reports on
//! a channel never joins its handles at all. `Once` is `std::sync::Once` in shape, and is wrapped
//! only so that a ported call site names the type the source named.

use std::sync::{Arc, Condvar, Mutex, PoisonError};

struct Counter {
    outstanding: Mutex<usize>,
    drained: Condvar,
}

/// `sync.WaitGroup`: a counter that goroutines add to and wait on.
#[derive(Clone)]
pub struct WaitGroup {
    counter: Arc<Counter>,
}

impl Default for WaitGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl WaitGroup {
    /// A group with nothing outstanding.
    #[must_use]
    pub fn new() -> Self {
        Self {
            counter: Arc::new(Counter {
                outstanding: Mutex::new(0),
                drained: Condvar::new(),
            }),
        }
    }

    /// `wg.Add(n)`.
    pub fn add(&self, n: usize) {
        let mut outstanding = self
            .counter
            .outstanding
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        *outstanding += n;
    }

    /// `wg.Done()`.
    ///
    /// Saturating rather than wrapping: the source panics on a negative counter, and a library that
    /// aborts removes a decision the ported program should make. A group that goes negative here
    /// stays at zero, so a waiter is released rather than hung — which is the failure mode a reader
    /// can see.
    pub fn done(&self) {
        let mut outstanding = self
            .counter
            .outstanding
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        *outstanding = outstanding.saturating_sub(1);
        let released = *outstanding == 0;
        drop(outstanding);
        if released {
            self.counter.drained.notify_all();
        }
    }

    /// `wg.Wait()` — block until the counter reaches zero. 265 call sites in the measured corpus.
    pub fn wait(&self) {
        let mut outstanding = self
            .counter
            .outstanding
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        while *outstanding > 0 {
            outstanding = self
                .counter
                .drained
                .wait(outstanding)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }

    /// How many `Done` calls the group is still waiting for.
    #[must_use]
    pub fn outstanding(&self) -> usize {
        *self
            .counter
            .outstanding
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

/// `sync.Once`: run a closure exactly once, however many goroutines reach it.
#[derive(Clone)]
pub struct Once {
    done: Arc<std::sync::Once>,
}

impl Default for Once {
    fn default() -> Self {
        Self::new()
    }
}

impl Once {
    /// A `Once` that has not run.
    #[must_use]
    pub fn new() -> Self {
        Self {
            done: Arc::new(std::sync::Once::new()),
        }
    }

    /// `once.Do(f)` — 141 call sites in the measured corpus.
    ///
    /// Every caller returns only after the closure has completed, including the callers that did
    /// not run it. That is the source's guarantee and the reason `Once` is not a boolean flag.
    pub fn call<F: FnOnce()>(&self, f: F) {
        self.done.call_once(f);
    }

    /// Whether the closure has run to completion.
    #[must_use]
    pub fn has_run(&self) -> bool {
        self.done.is_completed()
    }
}
