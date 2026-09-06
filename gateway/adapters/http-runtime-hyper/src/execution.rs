use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use tokio::sync::oneshot;
use tokio::task::JoinSet;

use crate::admission::{Admission, AdmissionRefusal, Budget, Permit};
use crate::{HttpRequest, HttpResponse, MiddlewareChain, Router, SyncHandler, dispatch};

pub(crate) struct ExecutedResponse {
    pub response: HttpResponse,
    pub request: Arc<Permit>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ExecutionFailure {
    Admission(AdmissionRefusal),
    SupervisorPoisoned,
    TaskFailed,
}

struct CompletedJob {
    // Keep capacity charged until the supervisor consumes the completion.
    _job: Arc<Permit>,
    _request: Arc<Permit>,
    panicked: bool,
}

/// One task reaps this set; request services may only submit bounded work.
pub(crate) struct Execution {
    admission: Arc<Admission>,
    jobs: Mutex<Jobs>,
}

struct Jobs {
    set: JoinSet<CompletedJob>,
    reaper: Option<Waker>,
}

impl Execution {
    pub(crate) fn new(admission: Arc<Admission>) -> Self {
        Self {
            admission,
            jobs: Mutex::new(Jobs {
                set: JoinSet::new(),
                reaper: None,
            }),
        }
    }

    pub(crate) fn submit(
        &self,
        request: HttpRequest,
        router: Arc<Router<SyncHandler>>,
        chain: Arc<MiddlewareChain<HttpRequest, HttpResponse>>,
        request_permit: Arc<Permit>,
    ) -> Result<oneshot::Receiver<ExecutedResponse>, ExecutionFailure> {
        let mut jobs = self
            .jobs
            .lock()
            .map_err(|_| ExecutionFailure::SupervisorPoisoned)?;
        let job = self
            .admission
            .acquire(Budget::Job)
            .map_err(ExecutionFailure::Admission)?;
        let (sender, receiver) = oneshot::channel();
        jobs.set.spawn_blocking(move || {
            // Release builds use panic=abort; recovery here applies only to unwind builds.
            let outcome = catch_unwind(AssertUnwindSafe(|| dispatch(request, &router, &chain)));
            let panicked = outcome.is_err();
            let response = outcome.unwrap_or_else(|_| {
                HttpResponse::new(500).with_body(b"internal server error".to_vec())
            });
            let _ = sender.send(ExecutedResponse {
                response,
                request: Arc::clone(&request_permit),
            });
            CompletedJob {
                _job: job,
                _request: request_permit,
                panicked,
            }
        });
        let reaper = jobs.reaper.take();
        drop(jobs);
        if let Some(reaper) = reaper {
            reaper.wake();
        }
        Ok(receiver)
    }

    /// Never hold the set lock across an await or while running application code.
    /// The boolean reports an unwind-caught handler panic, not arbitrary panic recovery.
    pub(crate) fn poll_reap(
        &self,
        context: &mut Context<'_>,
    ) -> Poll<Option<Result<bool, ExecutionFailure>>> {
        // Poison refuses new submissions, but existing handles still need a single reaper.
        let mut jobs = self
            .jobs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        jobs.reaper = Some(context.waker().clone());
        let completion = jobs.set.poll_join_next(context);
        if matches!(completion, Poll::Ready(None)) {
            return Poll::Pending;
        }
        completion.map(|completion| {
            completion.map(|result| {
                result
                    .map(|completed| completed.panicked)
                    .map_err(|_| ExecutionFailure::TaskFailed)
            })
        })
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.jobs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .set
            .is_empty()
    }

    pub(crate) fn is_healthy(&self) -> bool {
        !self.jobs.is_poisoned()
    }
}

#[cfg(test)]
mod tests;
