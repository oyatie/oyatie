use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServingLimits {
    capacities: [usize; 3],
    pub(crate) body_deadline: Duration,
    pub(crate) drain_deadline: Duration,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidServingLimits {
    ZeroCapacity,
    JobsExceedRequests,
    ZeroDeadline,
}

impl std::fmt::Display for InvalidServingLimits {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::ZeroCapacity => "serving capacities must be positive",
            Self::JobsExceedRequests => "handler capacity exceeds request capacity",
            Self::ZeroDeadline => "serving deadlines must be positive",
        })
    }
}

impl std::error::Error for InvalidServingLimits {}

impl ServingLimits {
    pub fn new(
        connections: usize,
        requests: usize,
        jobs: usize,
        body_deadline: Duration,
        drain_deadline: Duration,
    ) -> Result<Self, InvalidServingLimits> {
        if connections == 0 || requests == 0 || jobs == 0 {
            return Err(InvalidServingLimits::ZeroCapacity);
        }
        if jobs > requests {
            return Err(InvalidServingLimits::JobsExceedRequests);
        }
        if body_deadline.is_zero() || drain_deadline.is_zero() {
            return Err(InvalidServingLimits::ZeroDeadline);
        }
        Ok(Self {
            capacities: [connections, requests, jobs],
            body_deadline,
            drain_deadline,
        })
    }

    pub(crate) fn capacity(&self, budget: Budget) -> usize {
        self.capacities[budget.index()]
    }
}

impl Default for ServingLimits {
    fn default() -> Self {
        Self {
            capacities: [256, 256, 32],
            body_deadline: Duration::from_secs(30),
            drain_deadline: Duration::from_secs(30),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Budget {
    Connection,
    Request,
    Job,
}

impl Budget {
    fn index(self) -> usize {
        match self {
            Self::Connection => 0,
            Self::Request => 1,
            Self::Job => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServingPhase {
    Running,
    Draining,
    Stopped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServingSnapshot {
    pub limits: ServingLimits,
    pub phase: ServingPhase,
    /// Counts are ordered as connections, requests, and submitted jobs.
    pub active: [usize; 3],
    pub high_water: [usize; 3],
    pub capacity_refusals: [u64; 3],
    pub events: ServingEvents,
    pub admission_healthy: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ServingEvents {
    pub request_refusals: u64,
    pub body_timeouts: u64,
    pub body_limits: u64,
    pub handler_panics: u64,
    pub connection_failures: u64,
    pub accept_failures: u64,
    pub drain_deadlines: u64,
}

pub(crate) enum RuntimeEvent {
    RequestRefused,
    BodyTimeout,
    BodyLimit,
    HandlerPanic,
    ConnectionFailure,
    AcceptFailure,
    DrainDeadline,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AdmissionRefusal {
    Capacity(Budget),
    Draining,
    Poisoned,
}

#[derive(Debug)]
struct State {
    snapshot: ServingSnapshot,
    drain_started: Option<Instant>,
    quiescence: Option<Waker>,
}

#[derive(Debug)]
pub(crate) struct Admission {
    state: Mutex<State>,
}

impl Admission {
    #[cfg(test)]
    pub(crate) fn poison_for_test(&self) {
        let _ = std::panic::catch_unwind(|| {
            let _guard = self.state.lock().unwrap();
            panic!("injected admission poison");
        });
    }

    pub(crate) fn new(limits: ServingLimits) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(State {
                snapshot: ServingSnapshot {
                    limits,
                    phase: ServingPhase::Running,
                    active: [0; 3],
                    high_water: [0; 3],
                    capacity_refusals: [0; 3],
                    events: ServingEvents::default(),
                    admission_healthy: true,
                },
                drain_started: None,
                quiescence: None,
            }),
        })
    }

    pub(crate) fn acquire(
        self: &Arc<Self>,
        budget: Budget,
    ) -> Result<Arc<Permit>, AdmissionRefusal> {
        let mut state = self.state.lock().map_err(|_| AdmissionRefusal::Poisoned)?;
        let snapshot = &mut state.snapshot;
        if snapshot.phase != ServingPhase::Running {
            return Err(AdmissionRefusal::Draining);
        }
        let index = budget.index();
        if snapshot.active[index] == snapshot.limits.capacity(budget) {
            snapshot.capacity_refusals[index] = snapshot.capacity_refusals[index].saturating_add(1);
            return Err(AdmissionRefusal::Capacity(budget));
        }
        snapshot.active[index] += 1;
        snapshot.high_water[index] = snapshot.high_water[index].max(snapshot.active[index]);
        Ok(Arc::new(Permit {
            admission: Arc::clone(self),
            budget,
        }))
    }

    pub(crate) fn request_drain(&self, now: Instant) -> Instant {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.snapshot.phase == ServingPhase::Running {
            state.snapshot.phase = ServingPhase::Draining;
        }
        *state.drain_started.get_or_insert(now)
    }

    /// Called by the supervisor only after it has reaped its owned task handles.
    pub(crate) fn finish_if_quiescent(&self) -> bool {
        let Ok(mut state) = self.state.lock() else {
            return false;
        };
        if state.snapshot.phase == ServingPhase::Running || state.snapshot.active != [0; 3] {
            return false;
        }
        state.snapshot.phase = ServingPhase::Stopped;
        true
    }

    pub(crate) fn snapshot(&self) -> ServingSnapshot {
        match self.state.lock() {
            Ok(state) => state.snapshot.clone(),
            Err(poisoned) => {
                let mut snapshot = poisoned.into_inner().snapshot.clone();
                snapshot.admission_healthy = false;
                snapshot
            }
        }
    }

    pub(crate) fn poll_quiescent(&self, context: &mut Context<'_>) -> Poll<()> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.quiescence = Some(context.waker().clone());
        if state.snapshot.active == [0; 3] {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }

    pub(crate) fn record(&self, event: RuntimeEvent) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let events = &mut state.snapshot.events;
        let counter = match event {
            RuntimeEvent::RequestRefused => &mut events.request_refusals,
            RuntimeEvent::BodyTimeout => &mut events.body_timeouts,
            RuntimeEvent::BodyLimit => &mut events.body_limits,
            RuntimeEvent::HandlerPanic => &mut events.handler_panics,
            RuntimeEvent::ConnectionFailure => &mut events.connection_failures,
            RuntimeEvent::AcceptFailure => &mut events.accept_failures,
            RuntimeEvent::DrainDeadline => &mut events.drain_deadlines,
        };
        *counter = counter.saturating_add(1);
    }
}

/// Shared by every remaining owner of one admitted unit, not one permit per clone.
#[derive(Debug)]
pub(crate) struct Permit {
    admission: Arc<Admission>,
    budget: Budget,
}

impl Drop for Permit {
    fn drop(&mut self) {
        let mut state = self
            .admission
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.snapshot.active[self.budget.index()] -= 1;
        let quiescence = state.quiescence.take();
        drop(state);
        if let Some(waker) = quiescence {
            waker.wake();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;

    fn limits() -> ServingLimits {
        ServingLimits::new(2, 2, 1, Duration::from_secs(1), Duration::from_secs(1)).unwrap()
    }

    #[test]
    fn refuses_zero_limits_and_jobs_exceeding_requests() {
        let second = Duration::from_secs(1);
        for (connections, requests, jobs) in [(0, 1, 1), (1, 0, 1), (1, 1, 0), (1, 1, 2)] {
            assert!(ServingLimits::new(connections, requests, jobs, second, second).is_err());
        }
        assert!(ServingLimits::new(1, 1, 1, Duration::ZERO, second).is_err());
        assert!(ServingLimits::new(1, 1, 1, second, Duration::ZERO).is_err());
    }

    #[test]
    fn every_budget_refuses_at_capacity_and_returns_on_drop() {
        for budget in [Budget::Connection, Budget::Request, Budget::Job] {
            let ledger = Admission::new(limits());
            let maximum = ledger.snapshot().limits.capacity(budget);
            let mut permits = Vec::new();
            for _ in 0..maximum {
                permits.push(ledger.acquire(budget).unwrap());
            }
            assert_eq!(
                ledger.acquire(budget).unwrap_err(),
                AdmissionRefusal::Capacity(budget)
            );
            assert_eq!(ledger.snapshot().active[budget.index()], maximum);
            permits.pop();
            let replacement = ledger.acquire(budget).unwrap();
            drop(replacement);
            drop(permits);
            let snapshot = ledger.snapshot();
            assert_eq!(snapshot.active, [0; 3]);
            assert_eq!(snapshot.high_water[budget.index()], maximum);
            assert_eq!(snapshot.capacity_refusals[budget.index()], 1);
        }
    }

    #[test]
    fn response_cancellation_does_not_refund_job_owned_request() {
        let ledger = Admission::new(limits());
        let response = ledger.acquire(Budget::Request).unwrap();
        let job_request = Arc::clone(&response);
        let job = ledger.acquire(Budget::Job).unwrap();
        drop(response);
        assert_eq!(ledger.snapshot().active, [0, 1, 1]);
        drop(job);
        assert_eq!(ledger.snapshot().active, [0, 1, 0]);
        drop(job_request);
        assert_eq!(ledger.snapshot().active, [0; 3]);
    }

    #[test]
    fn drain_is_idempotent_and_cannot_report_stopped_with_owned_work() {
        let ledger = Admission::new(limits());
        let request = ledger.acquire(Budget::Request).unwrap();
        let started = Instant::now();
        assert_eq!(ledger.request_drain(started), started);
        assert_eq!(
            ledger.request_drain(started + Duration::from_secs(1)),
            started
        );
        assert!(!ledger.finish_if_quiescent());
        assert_eq!(ledger.snapshot().phase, ServingPhase::Draining);
        for budget in [Budget::Connection, Budget::Request, Budget::Job] {
            assert_eq!(
                ledger.acquire(budget).unwrap_err(),
                AdmissionRefusal::Draining
            );
        }
        drop(request);
        assert!(ledger.finish_if_quiescent());
        assert_eq!(ledger.snapshot().phase, ServingPhase::Stopped);
        assert_eq!(
            ledger.acquire(Budget::Request).unwrap_err(),
            AdmissionRefusal::Draining
        );
    }

    #[test]
    fn concurrent_admission_and_drain_have_one_serial_order() {
        for _ in 0..64 {
            let ledger = Admission::new(limits());
            let start = Arc::new(Barrier::new(2));
            let worker_ledger = Arc::clone(&ledger);
            let worker_start = Arc::clone(&start);
            let worker = thread::spawn(move || {
                worker_start.wait();
                worker_ledger.acquire(Budget::Request)
            });
            start.wait();
            ledger.request_drain(Instant::now());
            let result = worker.join().unwrap();
            match &result {
                Ok(_) => assert_eq!(ledger.snapshot().active[1], 1),
                Err(error) => assert_eq!(*error, AdmissionRefusal::Draining),
            }
            assert_eq!(
                ledger.acquire(Budget::Request).unwrap_err(),
                AdmissionRefusal::Draining
            );
            drop(result);
            assert!(ledger.finish_if_quiescent());
        }
    }
}
