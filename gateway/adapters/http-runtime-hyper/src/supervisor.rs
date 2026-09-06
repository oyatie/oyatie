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

pub(crate) async fn run(
    listener: impl accept::Acceptor,
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
                if let Some(Err(error)) = completion
                    && !error.is_cancelled() {
                        control.admission.record(RuntimeEvent::ConnectionFailure);
                        failure = Some("connection supervisor task failed".into());
                        control.request_drain();
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

mod accept;
mod runtime_owner;
mod transport;
pub(crate) use runtime_owner::run_std;
use transport::connection;

#[cfg(test)]
mod http2_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod wire_tests;
