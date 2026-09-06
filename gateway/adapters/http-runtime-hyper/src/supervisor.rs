use std::future::poll_fn;
use std::net::TcpListener as StdTcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::time::Instant;

use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo, TokioTimer};
use hyper_util::server::conn::auto::Builder;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use tokio::task::JoinSet;

use crate::admission::{Admission, AdmissionRefusal, Budget, Permit, RuntimeEvent};
use crate::execution::{Execution, ExecutionFailure};
use crate::{
    HttpRequest, HttpResponse, HyperRuntimeError, MiddlewareChain, Router, ServerConfig,
    ServingLimits, ServingPhase, ServingSnapshot, SyncHandler, collect_hyper_request,
};

#[derive(Clone, Debug)]
pub struct ServingControl {
    admission: Arc<Admission>,
    drain: watch::Sender<bool>,
    claimed: Arc<AtomicBool>,
}

impl ServingControl {
    pub fn new(limits: ServingLimits) -> Self {
        Self {
            admission: Admission::new(limits),
            drain: watch::channel(false).0,
            claimed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn request_drain(&self) {
        self.admission.request_drain(Instant::now());
        self.drain.send_replace(true);
    }

    pub fn snapshot(&self) -> ServingSnapshot {
        self.admission.snapshot()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServingOutcome {
    Drained,
    DeadlineExceeded,
    InfrastructureFailure,
}

#[derive(Clone, Debug)]
pub struct ServingReport {
    pub outcome: ServingOutcome,
    pub snapshot: ServingSnapshot,
    /// Retains observation of residual work; dropping this does not cancel effects.
    pub completion: ServingControl,
    pub failure: Option<String>,
}

impl ServingReport {
    pub fn into_result(self) -> Result<(), HyperRuntimeError> {
        match self.outcome {
            ServingOutcome::Drained => Ok(()),
            outcome => Err(HyperRuntimeError::Runtime(format!(
                "serving {outcome:?}; outstanding {:?}; failure {:?}",
                self.snapshot.active, self.failure,
            ))),
        }
    }
}

struct Context {
    control: ServingControl,
    execution: Execution,
    router: Arc<Router<SyncHandler>>,
    chain: Arc<MiddlewareChain<HttpRequest, HttpResponse>>,
    config: ServerConfig,
}

impl Context {
    async fn request(&self, request: Request<Incoming>) -> Response<Full<Bytes>> {
        let is_http1 = request.version() != hyper::Version::HTTP_2;
        let permit = match self.control.admission.acquire(Budget::Request) {
            Ok(permit) => permit,
            Err(error) => {
                self.control.admission.record(RuntimeEvent::RequestRefused);
                if error == AdmissionRefusal::Poisoned {
                    self.control.request_drain();
                }
                return refusal(503, is_http1, None);
            }
        };
        let deadline = self.control.snapshot().limits.body_deadline;
        let parsed = match tokio::time::timeout(
            deadline,
            collect_hyper_request(request, self.config.max_body_bytes),
        )
        .await
        {
            Ok(Ok(parsed)) => parsed,
            Ok(Err(error)) => {
                if matches!(error, HyperRuntimeError::BodyTooLarge { .. }) {
                    self.control.admission.record(RuntimeEvent::BodyLimit);
                }
                return refusal(error.status_code(), is_http1, Some(permit));
            }
            Err(_) => {
                self.control.admission.record(RuntimeEvent::BodyTimeout);
                return refusal(408, is_http1, Some(permit));
            }
        };
        let response = match self.execution.submit(
            parsed,
            self.router.clone(),
            self.chain.clone(),
            permit.clone(),
        ) {
            Ok(response) => response,
            Err(error) => {
                self.control.admission.record(RuntimeEvent::RequestRefused);
                if !matches!(
                    error,
                    ExecutionFailure::Admission(
                        AdmissionRefusal::Capacity(_) | AdmissionRefusal::Draining
                    )
                ) {
                    self.control.request_drain();
                    return refusal(500, is_http1, Some(permit));
                }
                return refusal(503, is_http1, Some(permit));
            }
        };
        match response.await {
            Ok(response) => crate::response::convert(response.response, Some(response.request)),
            Err(_) => refusal(500, is_http1, Some(permit)),
        }
    }
}

fn refusal(status: u16, close: bool, permit: Option<Arc<Permit>>) -> Response<Full<Bytes>> {
    let mut response = HttpResponse::new(status).with_body(b"request refused".to_vec());
    if close {
        response = response.with_header("connection", "close");
    }
    crate::response::convert(response, permit)
}

async fn connection(stream: TcpStream, context: Arc<Context>) -> Result<(), String> {
    let mut drain = context.control.drain.subscribe();
    let service_context = context.clone();
    let service = service_fn(move |request| {
        let context = service_context.clone();
        async move { Ok::<_, std::convert::Infallible>(context.request(request).await) }
    });
    let mut builder = Builder::new(TokioExecutor::new());
    builder
        .http1()
        .header_read_timeout(context.config.header_read_timeout)
        .keep_alive(true)
        .timer(TokioTimer::new());
    let requests = context.control.snapshot().limits.capacity(Budget::Request);
    let streams = u32::try_from(requests).unwrap_or(u32::MAX);
    builder
        .http2()
        .max_concurrent_streams(streams)
        .keep_alive_interval(Some(context.config.keepalive_timeout / 2))
        .keep_alive_timeout(context.config.keepalive_timeout)
        .timer(TokioTimer::new());
    let connection = builder.serve_connection(TokioIo::new(stream), service);
    tokio::pin!(connection);
    if !*drain.borrow_and_update() {
        tokio::select! {
            result = &mut connection => return result.map_err(|error| error.to_string()),
            _ = drain.changed() => {},
        }
    }
    connection.as_mut().graceful_shutdown();
    connection.await.map_err(|error| error.to_string())
}

pub(crate) async fn run(
    listener: TcpListener,
    router: Arc<Router<SyncHandler>>,
    chain: Arc<MiddlewareChain<HttpRequest, HttpResponse>>,
    config: ServerConfig,
    control: ServingControl,
    max_accepts: Option<usize>,
    early_report: Option<mpsc::SyncSender<Result<ServingReport, HyperRuntimeError>>>,
) -> Result<ServingReport, HyperRuntimeError> {
    if max_accepts == Some(0)
        || config.header_read_timeout.is_zero()
        || config.keepalive_timeout.is_zero()
    {
        return Err(HyperRuntimeError::Config(
            "serving count and transport deadlines must be positive".into(),
        ));
    }
    if control.claimed.swap(true, Ordering::AcqRel) {
        return Err(HyperRuntimeError::Config(
            "serving control already consumed".into(),
        ));
    }
    let context = Arc::new(Context {
        execution: Execution::new(control.admission.clone()),
        control: control.clone(),
        router,
        chain,
        config,
    });
    let mut connections = JoinSet::new();
    let mut drain = control.drain.subscribe();
    let mut accepted = 0usize;
    let mut reserved = None;
    let mut deadline = None;
    let mut reported = None;
    let mut failure = None;
    let mut listener = Some(listener);
    let mut retry_at = None;
    loop {
        let snapshot = control.snapshot();
        if !snapshot.admission_healthy || !context.execution.is_healthy() {
            failure = Some("serving supervision poisoned".into());
            control.request_drain();
            if connections.is_empty() && context.execution.is_empty() && snapshot.active == [0; 3] {
                return Ok(ServingReport {
                    outcome: ServingOutcome::InfrastructureFailure,
                    snapshot: control.snapshot(),
                    completion: control.clone(),
                    failure,
                });
            }
        }
        if control.snapshot().phase != ServingPhase::Running && deadline.is_none() {
            let started = control.admission.request_drain(Instant::now());
            deadline = started.checked_add(control.snapshot().limits.drain_deadline);
            if deadline.is_none() {
                deadline = Some(Instant::now());
            }
            reserved = None;
            listener = None;
        }
        if max_accepts.is_some_and(|limit| accepted >= limit) {
            reserved = None;
            listener = None;
            if connections.is_empty() && control.snapshot().phase == ServingPhase::Running {
                control.request_drain();
                continue;
            }
        }
        if deadline.is_some()
            && connections.is_empty()
            && context.execution.is_empty()
            && control.admission.finish_if_quiescent()
        {
            return Ok(reported.unwrap_or_else(|| ServingReport {
                outcome: if failure.is_some() {
                    ServingOutcome::InfrastructureFailure
                } else {
                    ServingOutcome::Drained
                },
                snapshot: control.snapshot(),
                completion: control.clone(),
                failure,
            }));
        }
        if listener.is_some() && reserved.is_none() {
            match control.admission.acquire(Budget::Connection) {
                Ok(permit) => reserved = Some(permit),
                Err(AdmissionRefusal::Poisoned) => {
                    failure = Some("connection admission poisoned".into());
                    control.request_drain();
                    continue;
                }
                Err(_) => {}
            }
        }
        tokio::select! {
            _ = drain.changed(), if deadline.is_none() => {},
            _ = poll_fn(|cx| control.admission.poll_quiescent(cx)), if deadline.is_some() && connections.is_empty() && context.execution.is_empty() => {},
            completion = poll_fn(|cx| context.execution.poll_reap(cx)) => {
                if matches!(completion, Some(Ok(true))) {
                    control.admission.record(RuntimeEvent::HandlerPanic);
                }
                if let Some(Err(error)) = completion {
                    failure = Some(format!("execution supervision failed: {error:?}"));
                    control.request_drain();
                }
            },
            completion = connections.join_next(), if !connections.is_empty() => {
                if matches!(completion, Some(Ok((Err(_), _)))) {
                    control.admission.record(RuntimeEvent::ConnectionFailure);
                }
                if let Some(Err(error)) = completion {
                    if !error.is_cancelled() {
                        control.admission.record(RuntimeEvent::ConnectionFailure);
                        failure = Some("connection supervisor task failed".into());
                        control.request_drain();
                    }
                }
            },
            result = async {
                if let Some(when) = retry_at { tokio::time::sleep_until(when).await; }
                match &listener { Some(listener) => listener.accept().await, None => std::future::pending().await }
            }, if reserved.is_some() => {
                match result {
                    Ok((stream, _)) => {
                        retry_at = None;
                        accepted = accepted.saturating_add(1);
                        if let Some(permit) = reserved.take() {
                            let context = context.clone();
                            connections.spawn(async move {
                                let result = connection(stream, context).await;
                                (result, permit)
                            });
                        }
                    }
                    Err(error) => {
                        control.admission.record(RuntimeEvent::AcceptFailure);
                        if transient_accept(error.kind()) {
                            retry_at = Some(tokio::time::Instant::now() + std::time::Duration::from_millis(50));
                        } else {
                            failure = Some(format!("listener accept failed: {error}"));
                            control.request_drain();
                        }
                    }
                }
            },
            _ = async { match deadline { Some(deadline) => tokio::time::sleep_until(deadline.into()).await, None => std::future::pending().await } }, if reported.is_none() => {
                connections.abort_all();
                control.admission.record(RuntimeEvent::DrainDeadline);
                let report = ServingReport { outcome: ServingOutcome::DeadlineExceeded,
                    snapshot: control.snapshot(), completion: control.clone(), failure: failure.clone() };
                if let Some(sender) = &early_report { let _ = sender.send(Ok(report.clone())); }
                reported = Some(report);
            },
        }
    }
}

fn transient_accept(kind: std::io::ErrorKind) -> bool {
    matches!(
        kind,
        std::io::ErrorKind::Interrupted
            | std::io::ErrorKind::ConnectionAborted
            | std::io::ErrorKind::WouldBlock
    )
}

pub(crate) fn run_std(
    listener: StdTcpListener,
    router: Arc<Router<SyncHandler>>,
    chain: Arc<MiddlewareChain<HttpRequest, HttpResponse>>,
    config: ServerConfig,
    control: ServingControl,
    max_accepts: Option<usize>,
    process_signals: bool,
) -> Result<ServingReport, HyperRuntimeError> {
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("gateway-serving-owner".into())
        .spawn(move || {
            let result = (|| {
                listener
                    .set_nonblocking(true)
                    .map_err(|error| HyperRuntimeError::Bind(error.to_string()))?;
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|error| HyperRuntimeError::Runtime(error.to_string()))?;
                runtime.block_on(async {
                    let listener = TcpListener::from_std(listener)
                        .map_err(|error| HyperRuntimeError::Bind(error.to_string()))?;
                    let signals = if process_signals {
                        Some(Signals::install()?)
                    } else {
                        None
                    };
                    let serving = run(
                        listener,
                        router,
                        chain,
                        config,
                        control.clone(),
                        max_accepts,
                        Some(sender.clone()),
                    );
                    tokio::pin!(serving);
                    if let Some(mut signals) = signals {
                        tokio::select! {
                            result = &mut serving => return result,
                            _ = signals.recv() => control.request_drain(),
                        }
                    }
                    serving.await
                })
            })();
            let _ = sender.send(result);
        })
        .map_err(|error| HyperRuntimeError::Runtime(error.to_string()))?;
    receiver
        .recv()
        .map_err(|error| HyperRuntimeError::Runtime(error.to_string()))?
}

#[cfg(unix)]
struct Signals {
    interrupt: tokio::signal::unix::Signal,
    terminate: tokio::signal::unix::Signal,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HttpMethod;
    use http_body_util::BodyExt;
    use std::time::Duration;

    fn limits(requests: usize, jobs: usize) -> ServingLimits {
        ServingLimits::new(
            4,
            requests,
            jobs,
            Duration::from_millis(50),
            Duration::from_millis(100),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn poisoned_empty_admission_reports_closed_failure_without_spinning() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let control = ServingControl::new(limits(2, 1));
        control.admission.poison_for_test();
        let report = tokio::time::timeout(
            Duration::from_secs(2),
            run(
                listener,
                Arc::new(Router::new()),
                Arc::new(MiddlewareChain::new()),
                ServerConfig::default(),
                control,
                None,
                None,
            ),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(report.outcome, ServingOutcome::InfrastructureFailure);
        assert!(!report.snapshot.admission_healthy);
        assert_eq!(report.snapshot.phase, ServingPhase::Draining);
        assert_eq!(report.snapshot, report.completion.snapshot());
    }

    #[tokio::test]
    async fn idle_server_drains_without_a_connection_event() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let control = ServingControl::new(limits(2, 1));
        let task = tokio::spawn(run(
            listener,
            Arc::new(Router::new()),
            Arc::new(MiddlewareChain::new()),
            ServerConfig::default(),
            control.clone(),
            None,
            None,
        ));
        tokio::task::yield_now().await;
        control.request_drain();
        let report = tokio::time::timeout(Duration::from_secs(2), task)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(report.outcome, ServingOutcome::Drained);
        assert_eq!(report.snapshot.active, [0; 3]);
        assert_eq!(report.snapshot.phase, ServingPhase::Stopped);
    }

    #[tokio::test]
    async fn http2_global_job_budget_refuses_without_invoking_another_handler() {
        use std::sync::Mutex;
        let (release, blocked) = mpsc::sync_channel(1);
        let blocked = Mutex::new(blocked);
        let (started, entered) = tokio::sync::oneshot::channel();
        let started = Mutex::new(Some(started));
        let mut router: Router<SyncHandler> = Router::new();
        router
            .route(
                HttpMethod::Get,
                "/",
                Arc::new(move |_| {
                    if let Some(started) = started.lock().unwrap().take() {
                        let _ = started.send(());
                    }
                    blocked
                        .lock()
                        .unwrap()
                        .recv_timeout(Duration::from_secs(5))
                        .unwrap();
                    HttpResponse::new(200).with_body(b"ok".to_vec())
                }),
            )
            .unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let control = ServingControl::new(limits(2, 1));
        let server = tokio::spawn(run(
            listener,
            Arc::new(router),
            Arc::new(MiddlewareChain::new()),
            ServerConfig::default(),
            control.clone(),
            None,
            None,
        ));
        let socket = TcpStream::connect(address).await.unwrap();
        let (mut client, connection) =
            hyper::client::conn::http2::handshake(TokioExecutor::new(), TokioIo::new(socket))
                .await
                .unwrap();
        let driver = tokio::spawn(connection);
        let first = client.send_request(
            Request::builder()
                .uri("http://localhost/")
                .body(Full::new(Bytes::new()))
                .unwrap(),
        );
        let first = tokio::spawn(first);
        tokio::time::timeout(Duration::from_secs(2), entered)
            .await
            .unwrap()
            .unwrap();
        let second = client.send_request(
            Request::builder()
                .uri("http://localhost/")
                .body(Full::new(Bytes::new()))
                .unwrap(),
        );
        let second = tokio::time::timeout(Duration::from_secs(2), second)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(second.status(), 503);
        second.into_body().collect().await.unwrap();
        assert_eq!(control.snapshot().active[2], 1);
        release.send(()).unwrap();
        let first = first.await.unwrap().unwrap();
        assert_eq!(first.status(), 200);
        first.into_body().collect().await.unwrap();
        control.request_drain();
        drop(client);
        let report = tokio::time::timeout(Duration::from_secs(2), server)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(report.outcome, ServingOutcome::Drained);
        driver.abort();
        let _ = driver.await;
    }

    #[test]
    fn std_deadline_reports_residual_work_before_handler_completes() {
        use std::io::{Read, Write};
        use std::sync::Mutex;
        let (release, blocked) = mpsc::sync_channel(1);
        let blocked = Mutex::new(blocked);
        let (entered, started) = mpsc::sync_channel(1);
        let mut router: Router<SyncHandler> = Router::new();
        router
            .route(
                HttpMethod::Get,
                "/",
                Arc::new(move |_| {
                    entered.send(()).unwrap();
                    blocked
                        .lock()
                        .unwrap()
                        .recv_timeout(Duration::from_secs(5))
                        .unwrap();
                    HttpResponse::new(200)
                }),
            )
            .unwrap();
        let listener = StdTcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let control = ServingControl::new(limits(2, 1));
        let owner_control = control.clone();
        let (reports, report) = mpsc::sync_channel(1);
        let caller = std::thread::spawn(move || {
            let result = run_std(
                listener,
                Arc::new(router),
                Arc::new(MiddlewareChain::new()),
                ServerConfig::default(),
                owner_control,
                None,
                false,
            );
            reports.send(result).unwrap();
        });
        let mut client = std::net::TcpStream::connect(address).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        client
            .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .unwrap();
        started.recv_timeout(Duration::from_secs(2)).unwrap();
        control.request_drain();
        let report = report
            .recv_timeout(Duration::from_secs(2))
            .unwrap()
            .unwrap();
        assert_eq!(report.outcome, ServingOutcome::DeadlineExceeded);
        assert_eq!(report.snapshot.active[2], 1);
        assert_ne!(report.completion.snapshot().phase, ServingPhase::Stopped);
        release.send(()).unwrap();
        let mut response = Vec::new();
        let _ = client.read_to_end(&mut response);
        caller.join().unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while control.snapshot().phase != ServingPhase::Stopped && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(control.snapshot().phase, ServingPhase::Stopped);
        assert_eq!(control.snapshot().active, [0; 3]);
    }
}

#[cfg(unix)]
impl Signals {
    fn install() -> Result<Self, HyperRuntimeError> {
        use tokio::signal::unix::{SignalKind, signal};
        let map = |error: std::io::Error| {
            HyperRuntimeError::Runtime(format!("signal registration failed: {error}"))
        };
        Ok(Self {
            interrupt: signal(SignalKind::interrupt()).map_err(map)?,
            terminate: signal(SignalKind::terminate()).map_err(map)?,
        })
    }
    async fn recv(&mut self) {
        tokio::select! { _ = self.interrupt.recv() => {}, _ = self.terminate.recv() => {} }
    }
}

#[cfg(windows)]
struct Signals(tokio::signal::windows::CtrlC);
#[cfg(windows)]
impl Signals {
    fn install() -> Result<Self, HyperRuntimeError> {
        tokio::signal::windows::ctrl_c().map(Self).map_err(|error| {
            HyperRuntimeError::Runtime(format!("signal registration failed: {error}"))
        })
    }
    async fn recv(&mut self) {
        self.0.recv().await;
    }
}
