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
