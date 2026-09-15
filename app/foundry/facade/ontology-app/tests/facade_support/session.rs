//! One booted process driven across requests, and the per-request helpers
//! that compose a fresh process each call.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use foundry_ontology_app::{AppState, router, router_from};
use http_body_util::BodyExt;
use tower::ServiceExt;

use super::Fixture;

pub async fn post(fixture: &Fixture, token: Option<&str>, body: &str) -> (StatusCode, String) {
    let mut request = Request::builder()
        .method("POST")
        .uri("/v1/actions")
        .header("content-type", "application/json");
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let response = router(fixture.state())
        .oneshot(
            request
                .body(Body::from(body.to_owned()))
                .expect("a well-formed request"),
        )
        .await
        .expect("the router answers");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("a readable body")
        .to_bytes();
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

pub async fn get(fixture: &Fixture, token: Option<&str>, path: &str) -> (StatusCode, String) {
    let mut request = Request::builder().method("GET").uri(path);
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let response = router(fixture.state())
        .oneshot(request.body(Body::empty()).expect("a well-formed request"))
        .await
        .expect("the router answers");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("a readable body")
        .to_bytes();
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

pub async fn write_a_record(fixture: &Fixture, object_ref: &str, key: &str) {
    let body = format!(
        r#"{{"object_ref":"{object_ref}","action_type":"aty_record_write","idempotency_key":"{key}","occurred_at_epoch_seconds":1700000000,"properties":{{"name":"Ada"}}}}"#
    );
    let (status, reply) = post(fixture, Some(fixture.operator_token()), &body).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "the fixture write must land: {reply}"
    );
}

/// One booted process, driven across several requests.
///
/// The per-request helpers above compose a FRESH `AppState` each call, which
/// is right for isolation but makes process-lifetime counters unobservable —
/// every request would start from zero. A session builds the router once and
/// clones it per request, so what one request counted the next one can see.
pub struct Session {
    pub(super) router: axum::Router,
}

impl Fixture {
    pub fn session(&self) -> Session {
        Session {
            router: router(self.state()),
        }
    }
}

impl Session {
    pub fn from_shared(state: std::sync::Arc<AppState>) -> Self {
        Session {
            router: router_from(state),
        }
    }

    pub fn from_state(state: AppState) -> Self {
        Session {
            router: router(state),
        }
    }

    pub async fn post(&self, token: Option<&str>, body: &str) -> (StatusCode, String) {
        let mut request = Request::builder()
            .method("POST")
            .uri("/v1/actions")
            .header("content-type", "application/json");
        if let Some(token) = token {
            request = request.header("authorization", format!("Bearer {token}"));
        }
        self.send(
            request
                .body(Body::from(body.to_owned()))
                .expect("a request"),
        )
        .await
    }

    /// A POST to a path other than `/v1/actions`.
    pub async fn post_to(
        &self,
        token: Option<&str>,
        path: &str,
        body: &str,
    ) -> (StatusCode, String) {
        let mut request = Request::builder()
            .method("POST")
            .uri(path)
            .header("content-type", "application/json");
        if let Some(token) = token {
            request = request.header("authorization", format!("Bearer {token}"));
        }
        self.send(
            request
                .body(Body::from(body.to_owned()))
                .expect("a request"),
        )
        .await
    }

    pub async fn get(&self, token: Option<&str>, path: &str) -> (StatusCode, String) {
        let mut request = Request::builder().method("GET").uri(path);
        if let Some(token) = token {
            request = request.header("authorization", format!("Bearer {token}"));
        }
        self.send(request.body(Body::empty()).expect("a request"))
            .await
    }

    pub async fn send(&self, request: Request<Body>) -> (StatusCode, String) {
        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .expect("the router answers");
        let status = response.status();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("a readable body")
            .to_bytes();
        (status, String::from_utf8_lossy(&bytes).into_owned())
    }
}

pub const WRITE_BODY: &str = r#"{"object_ref":"ent_alpha","action_type":"aty_record_write","idempotency_key":"idem_1","occurred_at_epoch_seconds":1700000000,"properties":{"name":"Ada"}}"#;

/// Scrape the exposition. UNAUTHENTICATED on purpose: were `/metrics` behind
/// the refusal counter, every scrape would inflate the very number the
/// delta assertions read.
pub async fn scrape(session: &Session) -> String {
    let (status, body) = session.get(None, "/metrics").await;
    assert_eq!(status, StatusCode::OK);
    body
}

pub fn value_of(body: &str, metric: &str) -> u64 {
    body.lines()
        .find_map(|line| line.strip_prefix(&format!("{metric} ")))
        .unwrap_or_else(|| panic!("{metric} has no value line in:\n{body}"))
        .trim()
        .parse()
        .expect("a metric value is a number")
}
