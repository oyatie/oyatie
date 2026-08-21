//! Integration test: full GitHub webhook → 200/202 + GitHub queued statuses posted.
//!
//! Uses in-memory test doubles (no blocking HTTP inside async context).
//! The ed25519 verify step uses MockSignatureVerifier from the kernel.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use bytes::Bytes;
use ci_webhook_gateway_app::{AppState, build_router, replay::DeliveryGuard};
use ci_webhook_gateway_kernel::{
    AuthzDecision, GitHubStatusRequest, KernelError, MockSignatureVerifier, Result,
    WebhookAuthzGate, WebhookAuthzRequest,
};
use std::sync::{Arc, Mutex};
use tower::util::ServiceExt;

// ---------------------------------------------------------------------------
// Test doubles
// ---------------------------------------------------------------------------

struct AlwaysAllow;
impl WebhookAuthzGate for AlwaysAllow {
    fn decide(&self, _: &WebhookAuthzRequest) -> AuthzDecision {
        AuthzDecision::Allow
    }
}

struct AlwaysDeny;
impl WebhookAuthzGate for AlwaysDeny {
    fn decide(&self, _: &WebhookAuthzRequest) -> AuthzDecision {
        AuthzDecision::Forbid
    }
}

/// In-memory status poster that records every call (no HTTP, safe inside async).
struct RecordingStatusPoster {
    calls: std::sync::Mutex<Vec<String>>,
}

impl RecordingStatusPoster {
    fn new() -> Self {
        Self {
            calls: std::sync::Mutex::new(vec![]),
        }
    }

    fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }
}

impl ci_webhook_gateway_kernel::CommitStatusPoster for RecordingStatusPoster {
    fn post(&self, req: &GitHubStatusRequest) -> Result<()> {
        let entry = format!("{}:{}", req.context.as_str(), req.state.as_str());
        self.calls.lock().unwrap().push(entry);
        Ok(())
    }
}

struct NoopStatusPoster;
impl ci_webhook_gateway_kernel::CommitStatusPoster for NoopStatusPoster {
    fn post(&self, _: &GitHubStatusRequest) -> Result<()> {
        Ok(())
    }
}

struct FailingStatusPoster;
impl ci_webhook_gateway_kernel::CommitStatusPoster for FailingStatusPoster {
    fn post(&self, _: &GitHubStatusRequest) -> Result<()> {
        Err(KernelError::DownstreamTransport(
            "github unreachable".into(),
        ))
    }
}

// ---------------------------------------------------------------------------
// Sample GitHub PR payload
// ---------------------------------------------------------------------------

fn github_pr_payload(sha: &str) -> serde_json::Value {
    serde_json::json!({
        "action": "opened",
        "number": 42,
        "pull_request": {
            "number": 42,
            "draft": false,
            "base": { "ref": "dev", "sha": "0000000000000000000000000000000000000000" },
            "head": { "ref": "feature/test", "sha": sha }
        }
    })
}

fn make_state(
    poster: Arc<dyn ci_webhook_gateway_kernel::CommitStatusPoster + Send + Sync>,
) -> AppState {
    AppState {
        verifier: Arc::new(MockSignatureVerifier { verdict: Ok(()) }),
        authz: Arc::new(AlwaysAllow),
        status_poster: poster,
        target_branch: "dev".to_owned(),
        github_owner: "oyatie".to_owned(),
        github_repo: "oyatie".to_owned(),
        delivery_guard: Arc::new(Mutex::new(DeliveryGuard::with_default_ttl())),
    }
}

// ---------------------------------------------------------------------------
// Test 1 — full happy-path: webhook → 202 Accepted + 5 queued statuses posted.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn full_webhook_flow_returns_202_and_posts_queued_statuses() {
    let sha = "cafebabe1234567890";
    let payload = serde_json::to_vec(&github_pr_payload(sha)).unwrap();

    let recorder = Arc::new(RecordingStatusPoster::new());
    let state = make_state(recorder.clone());
    let app = build_router(state);

    let request = Request::builder()
        .method("POST")
        .uri("/webhook/github")
        .header("x-github-event", "pull_request")
        .header("x-github-delivery", "del-test")
        .header("x-hub-signature-256", "sha256=aabbccdd")
        .header("content-type", "application/json")
        .body(Body::from(Bytes::from(payload)))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    // 5 queued statuses (one per CommitStatusContext).
    assert_eq!(
        recorder.call_count(),
        5,
        "expected 5 queued status posts, got {}",
        recorder.call_count()
    );
}

// ---------------------------------------------------------------------------
// Test 2 — ping event returns 200 OK (ignored).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ping_event_returns_200_ignored() {
    let state = make_state(Arc::new(NoopStatusPoster));
    let app = build_router(state);

    let request = Request::builder()
        .method("POST")
        .uri("/webhook/github")
        .header("x-github-event", "ping")
        .header("x-github-delivery", "ping-001")
        .header("x-hub-signature-256", "sha256=aabbccdd")
        .header("content-type", "application/json")
        .body(Body::from("{}"))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// Test 3 — double delivery: first returns 202, replay returns 200; status
// poster called exactly 5 times (only on the first delivery, not the replay).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn replay_delivery_returns_200_and_status_not_posted_twice() {
    let sha = "deadbeef99887766";
    let payload = serde_json::to_vec(&github_pr_payload(sha)).unwrap();

    let recorder = Arc::new(RecordingStatusPoster::new());
    let state = make_state(recorder.clone());
    let app = build_router(state);

    // First delivery — must return 202 Accepted.
    let first_request = Request::builder()
        .method("POST")
        .uri("/webhook/github")
        .header("x-github-event", "pull_request")
        .header("x-github-delivery", "del-replay-test")
        .header("x-hub-signature-256", "sha256=aabbccdd")
        .header("content-type", "application/json")
        .body(Body::from(Bytes::from(payload.clone())))
        .unwrap();

    let first_response = app.clone().oneshot(first_request).await.unwrap();
    assert_eq!(
        first_response.status(),
        StatusCode::ACCEPTED,
        "first delivery must return 202 Accepted"
    );
    assert_eq!(
        recorder.call_count(),
        5,
        "first delivery must post 5 queued statuses"
    );

    // Replay — identical delivery-id, same payload: must return 200 idempotent
    // ack and NOT post statuses a second time.
    let replay_request = Request::builder()
        .method("POST")
        .uri("/webhook/github")
        .header("x-github-event", "pull_request")
        .header("x-github-delivery", "del-replay-test")
        .header("x-hub-signature-256", "sha256=aabbccdd")
        .header("content-type", "application/json")
        .body(Body::from(Bytes::from(payload)))
        .unwrap();

    let replay_response = app.oneshot(replay_request).await.unwrap();
    assert_eq!(
        replay_response.status(),
        StatusCode::OK,
        "replay delivery must return 200 OK (idempotent ack)"
    );

    let replay_body = axum::body::to_bytes(replay_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(
        replay_body.as_ref(),
        b"duplicate delivery, already accepted",
        "replay body must be the idempotent ack message"
    );

    // Status poster must still have been called exactly 5 times (first delivery only).
    assert_eq!(
        recorder.call_count(),
        5,
        "status poster must NOT be called again for a replay"
    );
}

// ---------------------------------------------------------------------------
// Test 4 — Cedar authz denial returns 403.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn authz_denied_returns_403() {
    let sha = "aaaa1111";
    let payload = serde_json::to_vec(&github_pr_payload(sha)).unwrap();

    let state = AppState {
        verifier: Arc::new(MockSignatureVerifier { verdict: Ok(()) }),
        authz: Arc::new(AlwaysDeny),
        status_poster: Arc::new(NoopStatusPoster),
        target_branch: "dev".to_owned(),
        github_owner: "oyatie".to_owned(),
        github_repo: "oyatie".to_owned(),
        delivery_guard: Arc::new(Mutex::new(DeliveryGuard::with_default_ttl())),
    };

    let app = build_router(state);

    let request = Request::builder()
        .method("POST")
        .uri("/webhook/github")
        .header("x-github-event", "pull_request")
        .header("x-github-delivery", "del-authz-test")
        .header("x-hub-signature-256", "sha256=aabbccdd")
        .header("content-type", "application/json")
        .body(Body::from(Bytes::from(payload)))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

// ---------------------------------------------------------------------------
// Test 5 — GitHub status poster failure returns 502.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn github_status_failure_returns_502() {
    let sha = "bbbb2222";
    let payload = serde_json::to_vec(&github_pr_payload(sha)).unwrap();

    let state = make_state(Arc::new(FailingStatusPoster));
    let app = build_router(state);

    let request = Request::builder()
        .method("POST")
        .uri("/webhook/github")
        .header("x-github-event", "pull_request")
        .header("x-github-delivery", "del-ghs-fail")
        .header("x-hub-signature-256", "sha256=aabbccdd")
        .header("content-type", "application/json")
        .body(Body::from(Bytes::from(payload)))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
}
