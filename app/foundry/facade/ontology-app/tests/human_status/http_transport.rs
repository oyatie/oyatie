use std::sync::{Arc, Mutex};

use crate::human_support::{TOKEN, now, view};
use axum::{
    Router,
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode},
};
use foundry_ontology_app::human_session::{HttpUserSessionAuthorizer, HumanSessionConfig};
use iam_identity_user_session::{SessionRefusal, UserSessionAuthorizer};

#[derive(Clone)]
struct Reply {
    status: StatusCode,
    body: String,
    location: Option<String>,
    delay: std::time::Duration,
}

struct StubState {
    reply: Mutex<Reply>,
    requests: Mutex<Vec<(String, axum::http::HeaderMap)>>,
}

struct Stub {
    base: String,
    state: Arc<StubState>,
    task: tokio::task::JoinHandle<()>,
}

impl Stub {
    async fn start() -> Self {
        let state = Arc::new(StubState {
            reply: Mutex::new(Reply {
                status: StatusCode::OK,
                body: serde_json::to_string(&view()).unwrap(),
                location: None,
                delay: std::time::Duration::ZERO,
            }),
            requests: Mutex::new(Vec::new()),
        });
        let router = Router::new()
            .fallback(
                async |State(state): State<Arc<StubState>>, request: Request<Body>| {
                    assert_eq!(request.method(), "GET");
                    state
                        .requests
                        .lock()
                        .unwrap()
                        .push((request.uri().to_string(), request.headers().clone()));
                    let reply = state.reply.lock().unwrap().clone();
                    tokio::time::sleep(reply.delay).await;
                    let mut response = Response::builder().status(reply.status);
                    if let Some(location) = reply.location {
                        response = response.header("location", location);
                    }
                    response.body(Body::from(reply.body)).unwrap()
                },
            )
            .with_state(state.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        Self { base, state, task }
    }
    fn authorizer(&self) -> HttpUserSessionAuthorizer {
        HttpUserSessionAuthorizer::new(HumanSessionConfig::loopback_http(&self.base).unwrap())
            .unwrap()
    }
}
impl Drop for Stub {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[tokio::test]
async fn transport_reads_fresh_exact_session_path_and_only_cookie_and_never_follows_redirects() {
    let stub = Stub::start().await;
    let authorizer = stub.authorizer();
    for _ in 0..2 {
        let membership = authorizer
            .authorize(Some(TOKEN), "ten_acme", now())
            .await
            .unwrap();
        assert_eq!(membership.user_id(), "usr_alice");
    }
    {
        let requests = stub.state.requests.lock().unwrap();
        assert_eq!(requests.len(), 2);
        for (uri, headers) in requests.iter() {
            assert_eq!(uri, "/identity/v1/ten_acme/session");
            assert_eq!(headers["cookie"], format!("__Host-oya_session={TOKEN}"));
            assert!(!headers.contains_key("authorization"));
            assert!(!headers.contains_key("origin"));
            assert!(!headers.contains_key("x-forwarded-host"));
        }
    }
    let destination = Stub::start().await;
    {
        let mut reply = stub.state.reply.lock().unwrap();
        reply.status = StatusCode::TEMPORARY_REDIRECT;
        reply.location = Some(format!("{}/identity/v1/ten_acme/session", destination.base));
    }
    assert_eq!(
        authorizer.authorize(Some(TOKEN), "ten_acme", now()).await,
        Err(SessionRefusal::MembershipUnavailable)
    );
    assert!(destination.state.requests.lock().unwrap().is_empty());
    assert_eq!(
        authorizer.authorize(None, "ten_acme", now()).await,
        Err(SessionRefusal::Unauthenticated)
    );
    assert_eq!(
        authorizer
            .authorize(Some("malformed"), "ten_acme", now())
            .await,
        Err(SessionRefusal::InvalidCredential)
    );
    assert_eq!(stub.state.requests.lock().unwrap().len(), 3);

    let mut special = view();
    special.tenant_id = "ten_a/b?c#d".into();
    {
        let mut reply = stub.state.reply.lock().unwrap();
        reply.status = StatusCode::OK;
        reply.body = serde_json::to_string(&special).unwrap();
    }
    authorizer
        .authorize(Some(TOKEN), &special.tenant_id, now())
        .await
        .unwrap();
    assert_eq!(
        stub.state.requests.lock().unwrap().last().unwrap().0,
        "/identity/v1/ten_a%2Fb%3Fc%23d/session"
    );
}

#[tokio::test]
async fn malformed_expired_wrong_tenant_roles_outage_logout_and_oversized_responses_refuse() {
    let stub = Stub::start().await;
    let authorizer = stub.authorizer();
    let mut wrong_tenant = view();
    wrong_tenant.tenant_id = "ten_other".into();
    let mut expired = view();
    expired.expires_at = now();
    let mut wrong_subject = view();
    wrong_subject.user_id = "sp_workload".into();
    let mut duplicate_role = view();
    duplicate_role.roles.push("foundry-operator".into());
    let mut invalid_role = view();
    invalid_role.roles = vec![" ".into()];
    for (body, expected) in [
        ("malformed".into(), SessionRefusal::MembershipUnavailable),
        (
            serde_json::to_string(&wrong_tenant).unwrap(),
            SessionRefusal::TenantMismatch,
        ),
        (
            serde_json::to_string(&expired).unwrap(),
            SessionRefusal::Expired,
        ),
        (
            serde_json::to_string(&wrong_subject).unwrap(),
            SessionRefusal::InvalidSubject,
        ),
        (
            serde_json::to_string(&duplicate_role).unwrap(),
            SessionRefusal::MembershipUnavailable,
        ),
        (
            serde_json::to_string(&invalid_role).unwrap(),
            SessionRefusal::MembershipUnavailable,
        ),
        (
            "x".repeat(HttpUserSessionAuthorizer::MAX_BODY_BYTES + 1),
            SessionRefusal::MembershipUnavailable,
        ),
    ] {
        stub.state.reply.lock().unwrap().body = body;
        assert_eq!(
            authorizer.authorize(Some(TOKEN), "ten_acme", now()).await,
            Err(expected)
        );
    }
    for (status, expected) in [
        (
            StatusCode::SERVICE_UNAVAILABLE,
            SessionRefusal::MembershipUnavailable,
        ),
        (StatusCode::UNAUTHORIZED, SessionRefusal::Unauthenticated),
        (StatusCode::FORBIDDEN, SessionRefusal::InactiveMembership),
    ] {
        stub.state.reply.lock().unwrap().status = status;
        assert_eq!(
            authorizer.authorize(Some(TOKEN), "ten_acme", now()).await,
            Err(expected)
        );
    }
    stub.state.reply.lock().unwrap().delay = std::time::Duration::from_secs(3);
    assert_eq!(
        authorizer.authorize(Some(TOKEN), "ten_acme", now()).await,
        Err(SessionRefusal::MembershipUnavailable)
    );
}

#[tokio::test]
async fn body_bound_is_enforced_without_content_length() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = [0u8; 8192];
        assert!(stream.read(&mut request).await.unwrap() > 0);
        let bytes = vec![b'x'; HttpUserSessionAuthorizer::MAX_BODY_BYTES + 1];
        let head = format!(
            "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n{:x}\r\n",
            bytes.len()
        );
        stream.write_all(head.as_bytes()).await.unwrap();
        stream.write_all(&bytes).await.unwrap();
        stream.write_all(b"\r\n0\r\n\r\n").await.unwrap();
    });
    let authorizer =
        HttpUserSessionAuthorizer::new(HumanSessionConfig::loopback_http(&base).unwrap()).unwrap();
    assert_eq!(
        authorizer.authorize(Some(TOKEN), "ten_acme", now()).await,
        Err(SessionRefusal::MembershipUnavailable)
    );
    task.await.unwrap();
}

#[tokio::test]
async fn configured_http_authorizer_enters_the_same_native_gate_and_logout_is_rechecked() {
    use crate::human_support::{PATH, body, cookie, instrument, request};
    use std::sync::atomic::Ordering;
    let stub = Stub::start().await;
    let fixture = crate::facade_support::Fixture::new("human-http-composition");
    let (router, reads, pdp) = instrument(fixture.state(), Arc::new(stub.authorizer()));
    body(request(&router, PATH, &[&cookie()]).await, StatusCode::OK).await;
    let observations = reads.load(Ordering::SeqCst);
    assert!(observations > 0);
    let mut roleless = view();
    roleless.roles.clear();
    stub.state.reply.lock().unwrap().body = serde_json::to_string(&roleless).unwrap();
    body(
        request(&router, PATH, &[&cookie()]).await,
        StatusCode::FORBIDDEN,
    )
    .await;
    stub.state.reply.lock().unwrap().status = StatusCode::UNAUTHORIZED;
    body(
        request(&router, PATH, &[&cookie()]).await,
        StatusCode::UNAUTHORIZED,
    )
    .await;
    assert_eq!(stub.state.requests.lock().unwrap().len(), 3);
    assert_eq!(pdp.calls.load(Ordering::SeqCst), 2);
    assert_eq!(reads.load(Ordering::SeqCst), observations);
}

#[tokio::test]
async fn session_expiring_during_the_iam_read_is_refused() {
    let stub = Stub::start().await;
    let mut expiring = view();
    expiring.expires_at = now() + 1;
    {
        let mut reply = stub.state.reply.lock().unwrap();
        reply.body = serde_json::to_string(&expiring).unwrap();
        reply.delay = std::time::Duration::from_millis(1200);
    }
    assert_eq!(
        stub.authorizer()
            .authorize(Some(TOKEN), "ten_acme", now())
            .await,
        Err(SessionRefusal::Expired)
    );
}
