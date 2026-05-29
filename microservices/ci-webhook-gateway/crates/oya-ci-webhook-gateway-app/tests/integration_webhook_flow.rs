//! Integration test: full Forgejo webhook → 200/202 + GitHub statuses posted.
//!
//! Uses in-memory test doubles (no blocking HTTP inside async context).
//! The ed25519 verify step uses MockSignatureVerifier from the kernel.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use bytes::Bytes;
use oya_ci_webhook_gateway_app::{AppState, build_router, replay::DeliveryGuard};
use oya_ci_webhook_gateway_kernel::{
    AuthzDecision, CiTriggerEvent, GitHubStatusRequest, JenkinsJob, JobStatus, KernelError,
    MockSignatureVerifier, Result, WebhookAuthzGate, WebhookAuthzRequest,
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

struct MockJenkinsClient {
    trigger_result: std::sync::Mutex<Result<JenkinsJob>>,
    poll_result: std::sync::Mutex<Result<JobStatus>>,
}

impl MockJenkinsClient {
    fn new(trigger: Result<JenkinsJob>, poll: Result<JobStatus>) -> Self {
        Self {
            trigger_result: std::sync::Mutex::new(trigger),
            poll_result: std::sync::Mutex::new(poll),
        }
    }
}

impl oya_ci_webhook_gateway_kernel::JenkinsClient for MockJenkinsClient {
    fn trigger(&self, job_name: &str, event: &CiTriggerEvent) -> Result<JenkinsJob> {
        let guard = self.trigger_result.lock().unwrap();
        guard.clone().map(|mut j| {
            j.trigger = event.clone();
            j.job_name = job_name.to_owned();
            j
        })
    }

    fn poll_status(&self, _: &JenkinsJob) -> Result<JobStatus> {
        self.poll_result.lock().unwrap().clone()
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

impl oya_ci_webhook_gateway_kernel::CommitStatusPoster for RecordingStatusPoster {
    fn post(&self, req: &GitHubStatusRequest) -> Result<()> {
        let entry = format!("{}:{}", req.context.as_str(), req.state.as_str());
        self.calls.lock().unwrap().push(entry);
        Ok(())
    }
}

struct NoopStatusPoster;
impl oya_ci_webhook_gateway_kernel::CommitStatusPoster for NoopStatusPoster {
    fn post(&self, _: &GitHubStatusRequest) -> Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Sample Forgejo PR payload
// ---------------------------------------------------------------------------

fn forgejo_pr_payload(sha: &str) -> serde_json::Value {
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

// ---------------------------------------------------------------------------
// Test 1 — full happy-path: webhook -> 202 Accepted + 10 GitHub statuses posted.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn full_webhook_flow_returns_202_and_posts_statuses() {
    let sha = "cafebabe1234567890";
    let payload = serde_json::to_vec(&forgejo_pr_payload(sha)).unwrap();

    let jenkins_job = JenkinsJob {
        job_name: "oyaCiLane".to_owned(),
        build_number: 99,
        trigger: CiTriggerEvent {
            repo: "oyatie/oyatie".to_owned(),
            branch: "dev".to_owned(),
            head_sha: sha.to_owned(),
            base_sha: "0000".to_owned(),
            pr_number: 42,
            delivery_id: "del-test".to_owned(),
            action: oya_ci_webhook_gateway_kernel::CiAction::PrOpened,
        },
        status: JobStatus::Queued,
        build_url: Some("https://jenkins.example.com/job/oyaCiLane/99/".to_owned()),
    };

    let recorder = Arc::new(RecordingStatusPoster::new());

    let state = AppState {
        verifier: Arc::new(MockSignatureVerifier { verdict: Ok(()) }),
        authz: Arc::new(AlwaysAllow),
        jenkins: Arc::new(MockJenkinsClient::new(
            Ok(jenkins_job),
            Ok(JobStatus::Success),
        )),
        status_poster: recorder.clone(),
        target_branch: "dev".to_owned(),
        github_owner: "oyatie".to_owned(),
        github_repo: "oyatie".to_owned(),
        jenkins_job_name: "oyaCiLane".to_owned(),
        delivery_guard: Arc::new(Mutex::new(DeliveryGuard::with_default_ttl())),
    };

    let app = build_router(state);

    let request = Request::builder()
        .method("POST")
        .uri("/webhook/forgejo")
        .header("x-forgejo-event", "pull_request")
        .header("x-forgejo-delivery", "del-test")
        .header("x-hub-signature-256", "sha256=aabbccdd")
        .header("content-type", "application/json")
        .body(Body::from(Bytes::from(payload)))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    // 5 pending (Running) + 5 final (Success) = 10 total status posts.
    assert_eq!(
        recorder.call_count(),
        10,
        "expected 10 status posts (5 pending + 5 final), got {}",
        recorder.call_count()
    );
}

// ---------------------------------------------------------------------------
// Test 2 — ping event returns 200 OK (ignored, Jenkins not called).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ping_event_returns_200_ignored() {
    let state = AppState {
        verifier: Arc::new(MockSignatureVerifier { verdict: Ok(()) }),
        authz: Arc::new(AlwaysAllow),
        jenkins: Arc::new(MockJenkinsClient::new(
            Err(KernelError::DownstreamTransport(
                "should not be called".into(),
            )),
            Err(KernelError::DownstreamTransport(
                "should not be called".into(),
            )),
        )),
        status_poster: Arc::new(NoopStatusPoster),
        target_branch: "dev".to_owned(),
        github_owner: "oyatie".to_owned(),
        github_repo: "oyatie".to_owned(),
        jenkins_job_name: "oyaCiLane".to_owned(),
        delivery_guard: Arc::new(Mutex::new(DeliveryGuard::with_default_ttl())),
    };

    let app = build_router(state);

    let request = Request::builder()
        .method("POST")
        .uri("/webhook/forgejo")
        .header("x-forgejo-event", "ping")
        .header("x-forgejo-delivery", "ping-001")
        .header("x-hub-signature-256", "sha256=aabbccdd")
        .header("content-type", "application/json")
        .body(Body::from("{}"))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
