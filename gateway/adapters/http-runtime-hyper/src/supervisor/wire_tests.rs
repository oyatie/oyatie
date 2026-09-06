use super::*;
use crate::HttpMethod;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream as StdStream};
use std::sync::atomic::AtomicUsize;
use std::time::Duration;

struct Server {
    address: SocketAddr,
    control: ServingControl,
    report: mpsc::Receiver<Result<ServingReport, HyperRuntimeError>>,
}

impl Server {
    fn start(connections: usize, handler: SyncHandler) -> Self {
        let listener = StdTcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let control = ServingControl::new(
            ServingLimits::new(
                connections,
                2,
                1,
                Duration::from_millis(100),
                Duration::from_secs(1),
            )
            .unwrap(),
        );
        let worker_control = control.clone();
        let (sender, report) = mpsc::sync_channel(1);
        let mut router = Router::new();
        router
            .route(HttpMethod::Post, "/", handler.clone())
            .unwrap();
        router.route(HttpMethod::Get, "/", handler).unwrap();
        std::thread::spawn(move || {
            let result = run_std(
                listener,
                Arc::new(router),
                Arc::new(MiddlewareChain::new()),
                ServerConfig::default().with_max_body_bytes(3),
                worker_control,
                None,
                false,
            );
            let _ = sender.send(result);
        });
        Self {
            address,
            control,
            report,
        }
    }

    fn client(&self, request: &[u8]) -> StdStream {
        let mut stream = StdStream::connect(self.address).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        stream.write_all(request).unwrap();
        stream
    }

    fn stop(&self) -> ServingReport {
        self.control.request_drain();
        self.report
            .recv_timeout(Duration::from_secs(3))
            .unwrap()
            .unwrap()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.control.request_drain();
    }
}

fn read_closed(mut client: StdStream) -> String {
    let mut response = String::new();
    client.read_to_string(&mut response).unwrap();
    response
}

fn wait_for(mut predicate: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while !predicate() {
        assert!(Instant::now() < deadline, "serving state did not converge");
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn count_handler(count: Arc<AtomicUsize>) -> SyncHandler {
    Arc::new(move |_| {
        count.fetch_add(1, Ordering::SeqCst);
        HttpResponse::new(200).with_body(b"ok".to_vec())
    })
}

#[test]
fn body_deadline_boundary_and_cancellation_use_real_http1_admission() {
    let calls = Arc::new(AtomicUsize::new(0));
    let server = Server::start(4, count_handler(calls.clone()));
    let response = read_closed(
        server.client(b"POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 3\r\n\r\na"),
    );
    assert!(response.starts_with("HTTP/1.1 408"));
    assert!(response.to_ascii_lowercase().contains("connection: close"));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(server.control.snapshot().events.body_timeouts, 1);

    let exact = read_closed(server.client(
        b"POST / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Length: 3\r\n\r\nabc",
    ));
    assert!(exact.starts_with("HTTP/1.1 200"));
    let excess = read_closed(
        server.client(b"POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 4\r\n\r\nabcd"),
    );
    assert!(excess.starts_with("HTTP/1.1 413"));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(server.control.snapshot().events.body_limits, 1);

    let cancelled =
        server.client(b"POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 3\r\n\r\na");
    wait_for(|| server.control.snapshot().active[1] == 1);
    drop(cancelled);
    wait_for(|| server.control.snapshot().active[1] == 0);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let report = server.stop();
    assert_eq!(report.outcome, ServingOutcome::Drained);
    assert_eq!(report.snapshot.active, [0; 3]);
}

#[test]
fn saturated_connection_budget_reuses_capacity_and_drain_interrupts_paused_accept() {
    let calls = Arc::new(AtomicUsize::new(0));
    let server = Server::start(1, count_handler(calls.clone()));
    let first = server.client(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");
    wait_for(|| calls.load(Ordering::SeqCst) == 1);
    let mut second =
        server.client(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
    second
        .set_read_timeout(Some(Duration::from_millis(50)))
        .unwrap();
    let error = second.read(&mut [0; 1]).unwrap_err();
    assert!(matches!(
        error.kind(),
        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    drop(first);
    second
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    assert!(read_closed(second).starts_with("HTTP/1.1 200"));
    assert_eq!(calls.load(Ordering::SeqCst), 2);

    let occupied = server.client(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");
    wait_for(|| calls.load(Ordering::SeqCst) == 3);
    let pending = server.client(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");
    let report = server.stop();
    assert_eq!(report.outcome, ServingOutcome::Drained);
    assert_eq!(report.snapshot.high_water[0], 1);
    assert_eq!(calls.load(Ordering::SeqCst), 3);
    drop((occupied, pending));
}

#[test]
fn unread_large_response_retains_request_until_transport_disconnect() {
    let server = Server::start(
        2,
        Arc::new(|_| HttpResponse::new(200).with_body(vec![7; 16 * 1024 * 1024])),
    );
    let client = server.client(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");
    wait_for(|| {
        let snapshot = server.control.snapshot();
        snapshot.high_water[2] == 1 && snapshot.active[2] == 0
    });
    assert_eq!(server.control.snapshot().active[1], 1);
    drop(client);
    wait_for(|| server.control.snapshot().active[1] == 0);
    let report = server.stop();
    assert_eq!(report.outcome, ServingOutcome::Drained);
    assert_eq!(report.snapshot.active, [0; 3]);
}
