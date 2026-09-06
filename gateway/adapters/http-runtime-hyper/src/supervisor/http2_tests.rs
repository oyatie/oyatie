use super::*;
use crate::HttpMethod;
use http_body_util::BodyExt;
use std::sync::{Mutex, atomic::AtomicUsize};
use std::time::Duration;

async fn client(
    address: std::net::SocketAddr,
) -> (
    hyper::client::conn::http2::SendRequest<Full<Bytes>>,
    tokio::task::JoinHandle<Result<(), hyper::Error>>,
) {
    let socket = TcpStream::connect(address).await.unwrap();
    let (client, connection) =
        hyper::client::conn::http2::handshake(TokioExecutor::new(), TokioIo::new(socket))
            .await
            .unwrap();
    (client, tokio::spawn(connection))
}

fn request() -> Request<Full<Bytes>> {
    Request::builder()
        .uri("http://localhost/")
        .body(Full::new(Bytes::new()))
        .unwrap()
}

#[tokio::test]
async fn global_request_budget_survives_http2_reset_and_connection_disconnect() {
    let (release, blocked) = mpsc::sync_channel(1);
    let blocked = Mutex::new(blocked);
    let calls = Arc::new(AtomicUsize::new(0));
    let handler_calls = calls.clone();
    let (started, entered) = tokio::sync::oneshot::channel();
    let started = Mutex::new(Some(started));
    let mut router: Router<SyncHandler> = Router::new();
    router
        .route(
            HttpMethod::Get,
            "/",
            Arc::new(move |_| {
                handler_calls.fetch_add(1, Ordering::SeqCst);
                if let Some(started) = started.lock().unwrap().take() {
                    let _ = started.send(());
                }
                blocked
                    .lock()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(5))
                    .unwrap();
                HttpResponse::new(200)
            }),
        )
        .unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let control = ServingControl::new(
        ServingLimits::new(4, 1, 1, Duration::from_secs(1), Duration::from_secs(1)).unwrap(),
    );
    let server = tokio::spawn(run(
        listener,
        Arc::new(router),
        Arc::new(MiddlewareChain::new()),
        ServerConfig::default(),
        control.clone(),
        None,
        None,
    ));
    let (mut first_client, mut first_driver) = client(address).await;
    let mut first = tokio::spawn(first_client.send_request(request()));
    tokio::time::timeout(Duration::from_secs(2), entered)
        .await
        .unwrap()
        .unwrap();
    let (mut second_client, second_driver) = client(address).await;
    for disconnect in [false, true] {
        if disconnect {
            first.abort();
            assert!((&mut first).await.unwrap_err().is_cancelled());
            // Cancelling a request drops its H2 stream; closing the driver also
            // exercises transport loss while the application effect remains live.
            first_driver.abort();
            assert!((&mut first_driver).await.unwrap_err().is_cancelled());
            tokio::time::timeout(Duration::from_secs(2), async {
                // One live connection plus the supervisor's reserved accept permit.
                while control.snapshot().active[0] != 2 {
                    tokio::task::yield_now().await;
                }
            })
            .await
            .unwrap();
        }
        let response = tokio::time::timeout(
            Duration::from_secs(2),
            second_client.send_request(request()),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(response.status(), 503);
        response.into_body().collect().await.unwrap();
        let snapshot = control.snapshot();
        assert_eq!(snapshot.active[1..], [1, 1]);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
    release.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(2), async {
        while control.snapshot().active[1..] != [0, 0] {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    control.request_drain();
    drop((first_client, second_client));
    let report = tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(report.outcome, ServingOutcome::Drained);
    assert_eq!(report.snapshot.high_water[1..], [1, 1]);
    assert_eq!(report.snapshot.capacity_refusals[1], 2);
    assert_eq!(report.snapshot.active, [0; 3]);
    second_driver.abort();
    let _ = second_driver.await;
}
