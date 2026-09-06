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
