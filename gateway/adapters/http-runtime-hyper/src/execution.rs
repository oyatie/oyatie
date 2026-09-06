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
        let Ok(mut jobs) = self.jobs.lock() else {
            return Poll::Ready(Some(Err(ExecutionFailure::SupervisorPoisoned)));
        };
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
        self.jobs.lock().is_ok_and(|jobs| jobs.set.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HttpMethod;
    use crate::{ServingLimits, ServingPhase};
    use std::collections::BTreeMap;
    use std::future::poll_fn;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    #[tokio::test]
    async fn idle_reaper_is_woken_by_first_submission() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::task::{Wake, Waker};
        struct Counter(AtomicUsize);
        impl Wake for Counter {
            fn wake(self: Arc<Self>) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }
        let (admission, execution, router) = fixture(Arc::new(|_| HttpResponse::new(200)));
        let counter = Arc::new(Counter(AtomicUsize::new(0)));
        let waker = Waker::from(counter.clone());
        let mut context = Context::from_waker(&waker);
        assert!(execution.poll_reap(&mut context).is_pending());
        let response = execution
            .submit(
                request(),
                router,
                Arc::new(MiddlewareChain::new()),
                admission.acquire(Budget::Request).unwrap(),
            )
            .unwrap();
        assert!(counter.0.load(Ordering::SeqCst) > 0);
        drop(response.await.unwrap());
        assert_eq!(poll_fn(|cx| execution.poll_reap(cx)).await, Some(Ok(false)));
        assert!(execution.is_empty());
        assert_eq!(admission.snapshot().active, [0; 3]);
    }

    fn request() -> HttpRequest {
        HttpRequest {
            method: HttpMethod::Get,
            path: "/".into(),
            headers: BTreeMap::new(),
            body: Vec::new(),
            path_captures: BTreeMap::new(),
            matched_template: None,
        }
    }

    fn fixture(handler: SyncHandler) -> (Arc<Admission>, Execution, Arc<Router<SyncHandler>>) {
        let limits =
            ServingLimits::new(1, 2, 1, Duration::from_secs(1), Duration::from_secs(1)).unwrap();
        let admission = Admission::new(limits);
        let execution = Execution::new(Arc::clone(&admission));
        let mut router = Router::new();
        router.route(HttpMethod::Get, "/", handler).unwrap();
        (admission, execution, Arc::new(router))
    }

    #[tokio::test]
    async fn cancelled_waiter_retains_job_until_supervisor_reaps() {
        let (release, blocked) = mpsc::sync_channel(1);
        let blocked = Mutex::new(blocked);
        let (started, entered) = oneshot::channel();
        let started = Mutex::new(Some(started));
        let (admission, execution, router) = fixture(Arc::new(move |_| {
            if let Some(started) = started.lock().unwrap().take() {
                let _ = started.send(());
            }
            blocked
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(10))
                .unwrap();
            HttpResponse::new(200)
        }));
        let response = execution
            .submit(
                request(),
                router.clone(),
                Arc::new(MiddlewareChain::new()),
                admission.acquire(Budget::Request).unwrap(),
            )
            .unwrap();
        entered.await.unwrap();
        drop(response);
        assert_eq!(admission.snapshot().active, [0, 1, 1]);
        let second_request = admission.acquire(Budget::Request).unwrap();
        assert!(matches!(
            execution.submit(
                request(),
                router,
                Arc::new(MiddlewareChain::new()),
                second_request
            ),
            Err(ExecutionFailure::Admission(AdmissionRefusal::Capacity(
                Budget::Job
            )))
        ));
        admission.request_drain(Instant::now());
        assert!(!admission.finish_if_quiescent());
        release.send(()).unwrap();
        assert_eq!(poll_fn(|cx| execution.poll_reap(cx)).await, Some(Ok(false)));
        assert_eq!(admission.snapshot().active, [0; 3]);
        assert!(admission.finish_if_quiescent());
        assert_eq!(admission.snapshot().phase, ServingPhase::Stopped);
    }

    #[tokio::test]
    async fn completed_response_and_unreaped_job_keep_distinct_ownership() {
        let (admission, execution, router) = fixture(Arc::new(|_| HttpResponse::new(200)));
        let response = execution
            .submit(
                request(),
                router,
                Arc::new(MiddlewareChain::new()),
                admission.acquire(Budget::Request).unwrap(),
            )
            .unwrap()
            .await
            .unwrap();
        assert_eq!(response.response.status, 200);
        assert_eq!(admission.snapshot().active, [0, 1, 1]);
        assert_eq!(poll_fn(|cx| execution.poll_reap(cx)).await, Some(Ok(false)));
        assert_eq!(admission.snapshot().active, [0, 1, 0]);
        drop(response);
        assert_eq!(admission.snapshot().active, [0; 3]);
    }

    #[tokio::test]
    async fn unwind_panic_is_fixed_response_and_reaped_failure() {
        let (admission, execution, router) =
            fixture(Arc::new(|_| panic!("private handler detail")));
        let response = execution
            .submit(
                request(),
                router,
                Arc::new(MiddlewareChain::new()),
                admission.acquire(Budget::Request).unwrap(),
            )
            .unwrap()
            .await
            .unwrap();
        assert_eq!(response.response.status, 500);
        assert_eq!(response.response.body, b"internal server error");
        assert_eq!(poll_fn(|cx| execution.poll_reap(cx)).await, Some(Ok(true)));
        drop(response);
        assert_eq!(admission.snapshot().active, [0; 3]);
    }
}
