use axum::{body::Body, http::Request};
use mail_api::{Identity, MetadataStore, Principal};
use mail_kernel::{Account, Error};
use mail_service::{MailService, OwnerPolicy};
use mail_sqlite_store::SqliteStore;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
    mpsc,
};
use std::time::Duration;
use tower::ServiceExt;

const TOKEN: &str = "0123456789abcdef0123456789abcdef";

struct BarrierIdentity {
    inner: Arc<SqliteStore>,
    entered: mpsc::Sender<()>,
    release: Mutex<mpsc::Receiver<()>>,
    remaining: AtomicUsize,
}

impl Identity for BarrierIdentity {
    fn authenticate(&self, token: &str) -> Result<Principal, Error> {
        if self.remaining.fetch_sub(1, Ordering::SeqCst) == 1 {
            self.entered.send(()).unwrap();
            self.release.lock().unwrap().recv().unwrap();
        }
        self.inner.authenticate(token)
    }
}

async fn responsive(path: &str, gate: usize) {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    let blob = db.put_blob("a", b"hello").unwrap();
    let (entered, entered_rx) = mpsc::channel();
    let (release, release_rx) = mpsc::channel();
    let (heartbeat, heartbeat_rx) = mpsc::channel();
    let identity = Arc::new(BarrierIdentity {
        inner: db.clone(),
        entered,
        release: Mutex::new(release_rx),
        remaining: AtomicUsize::new(gate),
    });
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db,
        identity,
        policy: Arc::new(OwnerPolicy),
    });
    let app = mail_protocol_imap::jmap_router(service, "https://mail.example.org".into());
    let (wake, awakened) = tokio::sync::oneshot::channel();
    let observer = std::thread::spawn(move || {
        entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        wake.send(()).unwrap();
        let responsive = heartbeat_rx.recv_timeout(Duration::from_secs(1)).is_ok();
        release.send(()).unwrap();
        responsive
    });
    let timer = tokio::spawn(async move {
        awakened.await.unwrap();
        let _ = heartbeat.send(());
    });
    let path = path.replace("{blob}", &blob);
    let (method, body) = if path == "/jmap" {
        (
            "POST",
            r#"{"using":["urn:ietf:params:jmap:core"],"methodCalls":[["Core/echo",{},"x"]]}"#,
        )
    } else if path.starts_with("/upload") {
        ("POST", "hello")
    } else {
        ("GET", "")
    };
    let response = app
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("authorization", format!("Bearer {TOKEN}"))
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    let responsive = observer.join().unwrap();
    timer.await.unwrap();
    assert_eq!(response.status(), 200);
    assert!(
        responsive,
        "synchronous identity/storage work blocked the request runtime"
    );
}

#[tokio::test]
async fn session_authentication_yields_to_unrelated_requests() {
    responsive("/.well-known/jmap", 1).await;
}
#[tokio::test]
async fn request_authentication_yields_to_unrelated_requests() {
    responsive("/jmap", 1).await;
}
#[tokio::test]
async fn upload_authorization_yields_before_accepting_body() {
    responsive("/upload/a", 2).await;
}
#[tokio::test]
async fn upload_commit_yields_after_accepting_body() {
    responsive("/upload/a", 3).await;
}
#[tokio::test]
async fn download_authorization_yields_to_unrelated_requests() {
    responsive("/download/a/{blob}/file", 2).await;
}
