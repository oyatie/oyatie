use super::*;

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
                    let serving = run(
                        listener,
                        router,
                        chain,
                        config,
                        control.clone(),
                        max_accepts,
                        Some(sender.clone()),
                    );
                    serve_with_termination(serving, control, || {
                        if process_signals {
                            Signals::install().map(Some)
                        } else {
                            Ok(None)
                        }
                    })
                    .await
                })
            })();
            let _ = sender.send(result);
        })
        .map_err(|error| HyperRuntimeError::Runtime(error.to_string()))?;
    receiver
        .recv()
        .map_err(|error| HyperRuntimeError::Runtime(error.to_string()))?
}

async fn serve_with_termination(
    serving: impl Future<Output = Result<ServingReport, HyperRuntimeError>>,
    control: ServingControl,
    install: impl FnOnce() -> Result<Option<Signals>, HyperRuntimeError>,
) -> Result<ServingReport, HyperRuntimeError> {
    let signals = install()?;
    tokio::pin!(serving);
    if let Some(mut signals) = signals {
        tokio::select! {
            result = &mut serving => return result,
            _ = signals.recv() => control.request_drain(),
        }
    }
    serving.await
}


#[cfg(unix)]
struct Signals {
    interrupt: tokio::signal::unix::Signal,
    terminate: tokio::signal::unix::Signal,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn signal_registration_failure_refuses_before_serving_is_polled() {
        let polled = AtomicBool::new(false);
        let control = ServingControl::new(ServingLimits::default());
        let result = serve_with_termination(
            async {
                polled.store(true, Ordering::SeqCst);
                Err(HyperRuntimeError::Runtime("unexpected serving".into()))
            },
            control.clone(),
            || {
                Err(HyperRuntimeError::Runtime(
                    "signal registration failed: injected".into(),
                ))
            },
        )
        .await;
        assert!(
            matches!(result, Err(HyperRuntimeError::Runtime(message)) if message == "signal registration failed: injected")
        );
        assert!(!polled.load(Ordering::SeqCst));
        assert_eq!(control.snapshot().active, [0; 3]);
    }
}
