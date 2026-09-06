use super::*;
use std::io;
use std::net::SocketAddr;

pub(crate) trait Acceptor: Send {
    fn accept(&self) -> impl Future<Output = io::Result<(TcpStream, SocketAddr)>> + Send;
}

impl Acceptor for TcpListener {
    async fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        TcpListener::accept(self).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::time::Duration;

    struct Faults {
        calls: Arc<AtomicUsize>,
        first: io::ErrorKind,
    }

    impl Acceptor for Faults {
        async fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
            let call = self.calls.fetch_add(1, Ordering::SeqCst);
            Err(io::Error::from(if call == 0 {
                self.first
            } else {
                io::ErrorKind::PermissionDenied
            }))
        }
    }

    async fn exercise(first: io::ErrorKind, interrupt: bool) {
        let calls = Arc::new(AtomicUsize::new(0));
        let control = ServingControl::new(ServingLimits::default());
        let task = tokio::spawn(run(
            Faults {
                calls: calls.clone(),
                first,
            },
            Arc::new(Router::new()),
            Arc::new(MiddlewareChain::new()),
            ServerConfig::default(),
            control.clone(),
            None,
            None,
        ));
        let started = Instant::now();
        tokio::time::timeout(Duration::from_secs(2), async {
            while calls.load(Ordering::SeqCst) == 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        if interrupt {
            control.request_drain();
        }
        let report = tokio::time::timeout(Duration::from_secs(2), task)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        if interrupt {
            assert_eq!(report.outcome, ServingOutcome::Drained);
            assert_eq!(calls.load(Ordering::SeqCst), 1);
        } else {
            assert!(started.elapsed() >= Duration::from_millis(50));
            assert_eq!(calls.load(Ordering::SeqCst), 2);
            assert_eq!(report.outcome, ServingOutcome::InfrastructureFailure);
            assert!(report.failure.unwrap().contains("listener accept failed"));
        }
        assert_eq!(
            report.snapshot.events.accept_failures as usize,
            calls.load(Ordering::SeqCst)
        );
        assert_eq!(report.snapshot.active, [0; 3]);
    }

    #[tokio::test]
    async fn transient_accept_backoff_then_terminal_failure_drains_real_loop() {
        for kind in [
            io::ErrorKind::Interrupted,
            io::ErrorKind::ConnectionAborted,
            io::ErrorKind::WouldBlock,
        ] {
            exercise(kind, false).await;
        }
    }

    #[tokio::test]
    async fn drain_interrupts_accept_retry_without_another_attempt() {
        exercise(io::ErrorKind::Interrupted, true).await;
    }
}
