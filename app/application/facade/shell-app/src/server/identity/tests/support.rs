use super::super::{SignInConfig, router};
use axum::{
    Router,
    body::{Body, Bytes, to_bytes},
    extract::State,
    http::{HeaderMap, Method, Request, StatusCode, Uri, header},
    response::{IntoResponse, Response},
};
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

pub const ORIGIN: &str = "http://localhost:3000";

#[derive(Clone)]
pub struct Reply {
    pub status: StatusCode,
    pub body: String,
    pub headers: HeaderMap,
    pub delay_ms: u64,
}
impl Reply {
    pub fn json(body: &str) -> Self {
        Self {
            status: StatusCode::OK,
            body: body.into(),
            headers: HeaderMap::new(),
            delay_ms: 0,
        }
    }
}
pub struct Fixture {
    pub url: String,
    pub calls: Arc<Mutex<Vec<(String, HeaderMap, Bytes)>>>,
    pub reply: Arc<Mutex<Reply>>,
    server: tokio::task::JoinHandle<()>,
}
#[derive(Clone)]
struct MockState {
    calls: Arc<Mutex<Vec<(String, HeaderMap, Bytes)>>>,
    reply: Arc<Mutex<Reply>>,
}
impl Fixture {
    pub async fn new(body: &str) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let calls = Arc::new(Mutex::new(Vec::new()));
        let reply = Arc::new(Mutex::new(Reply::json(body)));
        let state = MockState {
            calls: calls.clone(),
            reply: reply.clone(),
        };
        let server = tokio::spawn(async move {
            axum::serve(listener, Router::new().fallback(mock).with_state(state))
                .await
                .unwrap();
        });
        Self {
            url,
            calls,
            reply,
            server,
        }
    }
    pub fn shell(&self, foundry: &Fixture) -> Router {
        router(Some(
            SignInConfig::for_loopback(ORIGIN, &self.url, &foundry.url).unwrap(),
        ))
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.server.abort();
    }
}
async fn mock(State(s): State<MockState>, uri: Uri, headers: HeaderMap, bytes: Bytes) -> Response {
    s.calls
        .lock()
        .unwrap()
        .push((uri.to_string(), headers, bytes));
    let reply = s.reply.lock().unwrap().clone();
    if reply.delay_ms > 0 {
        tokio::time::sleep(std::time::Duration::from_millis(reply.delay_ms)).await;
    }
    (reply.status, reply.headers, reply.body).into_response()
}

pub async fn call(
    shell: Router,
    method: Method,
    path: &str,
    headers: HeaderMap,
    body: impl Into<Body>,
) -> Response {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .body(body.into())
        .unwrap();
    *request.headers_mut() = headers;
    shell.oneshot(request).await.unwrap()
}
pub fn headers() -> HeaderMap {
    let mut h = HeaderMap::new();
    h.insert(header::ORIGIN, ORIGIN.parse().unwrap());
    h.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
    h.insert(
        header::COOKIE,
        format!("theme=dark; {}", cookie()).parse().unwrap(),
    );
    h
}
pub fn cookie() -> String {
    format!("__Host-oya_session=v1.{}", "A".repeat(43))
}
pub fn issued() -> String {
    format!(
        "{}; Path=/; Secure; HttpOnly; SameSite=Strict; Max-Age=3600",
        cookie()
    )
}
pub fn session() -> String {
    format!(
        r#"{{"tenantId":"ten_acme","userId":"usr_alice","expiresAt":{},"roles":["foundry-operator"]}}"#,
        super::super::config::now() + 3600
    )
}
pub const FACTS: &str = r#"{"policy_version":"psv-current","served_tenants":2,"projection_lag":7,"poisoned_entries":1}"#;
pub async fn json(response: Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), 128 * 1024).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}
pub fn expect_no_store(response: &Response, status: StatusCode) {
    assert_eq!(response.status(), status);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
}
