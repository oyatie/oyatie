use super::*;
use crate::HttpMethod;
use crate::{ServingLimits, ServingPhase};
use std::collections::BTreeMap;
use std::future::poll_fn;
use std::sync::mpsc;
use std::time::{Duration, Instant};

#[tokio::test]
async fn poisoned_owner_refuses_submission_but_retains_and_reaps_active_effect() {
    let (release, blocked) = mpsc::sync_channel(1);
    let blocked = Mutex::new(blocked);
    let (started, entered) = oneshot::channel();
    let started = Mutex::new(Some(started));
    let (admission, execution, router) = fixture(Arc::new(move |_| {
        let _ = started.lock().unwrap().take().unwrap().send(());
        blocked
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(5))
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
    tokio::time::timeout(Duration::from_secs(2), entered)
        .await
        .unwrap()
        .unwrap();
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let _guard = execution.jobs.lock().unwrap();
        panic!("injected execution owner poison");
    }));
    assert!(!execution.is_healthy());
    assert!(!execution.is_empty());
    assert!(matches!(
        execution.submit(
            request(),
            router,
            Arc::new(MiddlewareChain::new()),
            admission.acquire(Budget::Request).unwrap()
        ),
        Err(ExecutionFailure::SupervisorPoisoned)
    ));
    drop(response);
    admission.request_drain(Instant::now());
    assert_eq!(admission.snapshot().active, [0, 1, 1]);
    assert!(!admission.finish_if_quiescent());
    // A poisoned owner with live work must park, not synthesize ready failures.
    let mut context = Context::from_waker(Waker::noop());
    assert!(execution.poll_reap(&mut context).is_pending());
    release.send(()).unwrap();
    assert_eq!(
        tokio::time::timeout(
            Duration::from_secs(2),
            poll_fn(|cx| execution.poll_reap(cx))
        )
        .await
        .unwrap(),
        Some(Ok(false))
    );
    assert!(execution.is_empty());
    assert!(execution.poll_reap(&mut context).is_pending());
    assert_eq!(admission.snapshot().active, [0; 3]);
    assert!(!execution.is_healthy());
}

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
    let (admission, execution, router) = fixture(Arc::new(|_| panic!("private handler detail")));
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
