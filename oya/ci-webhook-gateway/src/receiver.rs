//! The axum HTTP receiver: the `/webhook/forgejo` endpoint that verifies the
//! HMAC, routes the event, and dispatches the gated pipeline.
//!
//! Order is load-bearing for security (ADR-0367): signature verification runs
//! on the RAW body BEFORE any parsing/routing/dispatch, and fails closed.

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::json;

use crate::config::GatewayConfig;
use crate::dispatch::{DispatchSubject, PipelineDispatcher};
use crate::error::GatewayError;
use crate::event::{self, RouteOutcome};
use crate::signature::{self, WebhookSecret};

/// Shared receiver state. `Arc`-cloned into every request handler.
#[derive(Clone)]
pub struct ReceiverState {
    pub config: Arc<GatewayConfig>,
    pub secret: Arc<WebhookSecret>,
    /// Optional ed25519 public key for `X-Forgejo-Signature` verification.
    /// `None` means the HMAC path is the only accepted signature scheme.
    pub ed25519_key: Option<Arc<signature::WebhookEd25519Key>>,
    pub dispatcher: Arc<dyn PipelineDispatcher>,
}

/// Canonical receiver path (per ADR-0374). Forgejo's webhook is registered to
/// post here.
pub const WEBHOOK_PATH: &str = "/webhook/forgejo";
/// Liveness path (for the k8s readiness/liveness probe).
pub const HEALTHZ_PATH: &str = "/healthz";

/// Build the axum router with all routes wired to `state`.
pub fn router(state: ReceiverState) -> Router {
    Router::new()
        .route(HEALTHZ_PATH, get(healthz))
        .route(WEBHOOK_PATH, post(handle_webhook))
        .with_state(state)
}

async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

/// Read a header as a `&str`, returning `None` if absent or non-ASCII.
fn header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

/// First present event header across Forgejo / Gitea / GitHub spellings.
fn event_name(headers: &HeaderMap) -> Option<&str> {
    header(headers, event::EVENT_HEADER_FORGEJO)
        .or_else(|| header(headers, event::EVENT_HEADER_GITEA))
        .or_else(|| header(headers, event::EVENT_HEADER_GITHUB))
}

/// First present delivery-id header (for dedup / log correlation).
fn delivery_id(headers: &HeaderMap) -> Option<&str> {
    header(headers, event::DELIVERY_HEADER_FORGEJO)
        .or_else(|| header(headers, event::DELIVERY_HEADER_GITEA))
        .or_else(|| header(headers, event::DELIVERY_HEADER_GITHUB))
}

/// The webhook handler. Pure-ish: all effects go through the injected
/// dispatcher, so this is exercised directly in tests with a fake dispatcher.
async fn handle_webhook(
    State(state): State<ReceiverState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let delivery = delivery_id(&headers).unwrap_or("unknown").to_owned();

    // 1. Verify signature on the RAW body FIRST (fail-closed before any parsing).
    //    Accepts HMAC-SHA256 (preferred) or ed25519 (when a public key is configured).
    let prefixed = header(&headers, signature::SIGNATURE_HEADER);
    let legacy = header(&headers, signature::LEGACY_SIGNATURE_HEADER);
    let ed25519_hdr = header(&headers, signature::ED25519_SIGNATURE_HEADER);
    let ed25519_key = state.ed25519_key.as_deref();
    if let Err(err) =
        signature::verify_any(&state.secret, &body, prefixed, legacy, ed25519_hdr, ed25519_key)
    {
        tracing::warn!(delivery = %delivery, error = %err, "webhook signature rejected");
        return error_response(&err);
    }

    // 2. Determine the event class.
    let Some(event) = event_name(&headers) else {
        let err = GatewayError::MalformedPayload("missing X-Forgejo-Event header".to_owned());
        tracing::warn!(delivery = %delivery, error = %err, "webhook missing event header");
        return error_response(&err);
    };
    let event = event.to_owned();

    // 3. Route to an outcome.
    let outcome = match event::route(&event, &body, &state.config.target_branch) {
        Ok(outcome) => outcome,
        Err(err) => {
            tracing::warn!(delivery = %delivery, event = %event, error = %err, "webhook unroutable");
            return error_response(&err);
        }
    };

    match outcome {
        RouteOutcome::Ignored { reason } => {
            tracing::info!(delivery = %delivery, event = %event, %reason, "webhook accepted, no dispatch");
            (
                StatusCode::OK,
                Json(json!({ "status": "ignored", "reason": reason, "delivery": delivery })),
            )
                .into_response()
        }
        RouteOutcome::Dispatch(ci_event) => {
            // 4. Dispatch the gated pipeline (kick the trusted runner).
            match state.dispatcher.dispatch(&ci_event).await {
                Ok(receipt) => {
                    let subject_kind = receipt.subject.kind();
                    let subject = subject_json(&receipt.subject);
                    tracing::info!(
                        delivery = %delivery,
                        subject_kind = %subject_kind,
                        sha = %receipt.head_sha,
                        snapshot_id = ?receipt.snapshot_id,
                        kicked_through = %receipt.kicked_through,
                        "pipeline dispatched"
                    );
                    let boundary = receipt.boundary.map(|s| s.id());
                    (
                        StatusCode::ACCEPTED,
                        Json(json!({
                            "status": "dispatched",
                            "subject_kind": subject_kind,
                            "subject": subject,
                            "head_sha": receipt.head_sha,
                            "snapshot_id": receipt.snapshot_id,
                            "kicked_through": receipt.kicked_through.id(),
                            "trusted_runner_owns_from": boundary,
                            "delivery": delivery,
                        })),
                    )
                        .into_response()
                }
                Err(err) => {
                    tracing::error!(delivery = %delivery, event = %event, error = %err, "dispatch failed");
                    error_response(&err)
                }
            }
        }
    }
}

fn subject_json(subject: &DispatchSubject) -> serde_json::Value {
    match subject {
        DispatchSubject::PullRequest { pr_number } => json!({ "pr_number": pr_number }),
        DispatchSubject::Issue { issue_number } => json!({ "issue_number": issue_number }),
        DispatchSubject::Push { reference } => json!({ "reference": reference }),
    }
}

/// Map a `GatewayError` to its HTTP response. The body carries the typed
/// reason (no secrets — `WebhookSecret` is redacted in `Debug`).
fn error_response(err: &GatewayError) -> Response {
    let status = StatusCode::from_u16(err.http_status()).unwrap_or(StatusCode::BAD_REQUEST);
    let payload = match err {
        GatewayError::Unimplemented { stage, debt_token } => json!({
            "status": "unimplemented",
            "stage": stage.id(),
            "placeholder_debt": debt_token,
            "detail": err.to_string(),
        }),
        _ => json!({ "status": "rejected", "detail": err.to_string() }),
    };
    (status, Json(payload)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatch::{DispatchReceipt, DispatchSubject, PipelineKickoff};
    use crate::error::{PipelineStage, Result};
    use crate::event::CiEvent;
    use axum::body::to_bytes;
    use axum::http::Request;
    use std::future::Future;
    use std::pin::Pin;
    use tower::util::ServiceExt; // for `oneshot`

    const SECRET: &str = "test-webhook-secret";

    struct FakeDispatcher {
        boundary: Option<PipelineStage>,
    }
    impl PipelineDispatcher for FakeDispatcher {
        fn dispatch<'a>(
            &'a self,
            event: &'a CiEvent,
        ) -> Pin<Box<dyn Future<Output = Result<DispatchReceipt>> + Send + 'a>> {
            let boundary = self.boundary;
            Box::pin(async move {
                // touch the kickoff conversion so the path is exercised
                let _ = PipelineKickoff::from_event(event);
                match event {
                    CiEvent::PullRequest(event) => Ok(DispatchReceipt {
                        subject: DispatchSubject::PullRequest {
                            pr_number: event.pr_number,
                        },
                        head_sha: event.head_sha.clone(),
                        snapshot_id: None,
                        kicked_through: PipelineStage::GateRunAll,
                        boundary,
                    }),
                    CiEvent::IssueSnapshot(event) => Ok(DispatchReceipt {
                        subject: DispatchSubject::Issue {
                            issue_number: event.issue_number,
                        },
                        head_sha: event.snapshot_id.clone(),
                        snapshot_id: Some(event.snapshot_id.clone()),
                        kicked_through: PipelineStage::BoardProjection,
                        boundary: None,
                    }),
                    CiEvent::PushSnapshot(event) => Ok(DispatchReceipt {
                        subject: DispatchSubject::Push {
                            reference: event.reference.clone(),
                        },
                        head_sha: event.after_sha.clone(),
                        snapshot_id: Some(event.snapshot_id.clone()),
                        kicked_through: PipelineStage::BoardProjection,
                        boundary: None,
                    }),
                }
            })
        }
    }

    fn sign(body: &[u8]) -> String {
        let digest = crate::signature::hmac_sha256(SECRET.as_bytes(), body);
        let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
        format!("sha256={hex}")
    }

    fn test_state() -> ReceiverState {
        ReceiverState {
            config: Arc::new(GatewayConfig {
                bind_addr: "127.0.0.1:0".to_owned(),
                target_branch: "dev".to_owned(),
                jenkins_dispatch_url: Some("http://jenkins/build".to_owned()),
                secret_present: true,
                dispatcher_kind: crate::config::DispatcherKind::Jenkins,
                controller_url: None,
            }),
            secret: Arc::new(WebhookSecret::new(SECRET.as_bytes().to_vec())),
            ed25519_key: None,
            dispatcher: Arc::new(FakeDispatcher {
                boundary: Some(PipelineStage::ReviewerGate),
            }),
        }
    }

    fn pr_body() -> Vec<u8> {
        br#"{"action":"opened","number":7,
            "pull_request":{"number":7,
              "base":{"ref":"dev","sha":"b"},
              "head":{"ref":"feature/z","sha":"abc123"},"draft":false}}"#
            .to_vec()
    }

    fn issue_body() -> Vec<u8> {
        include_bytes!("../tests/fixtures/issue-label-snapshot.json").to_vec()
    }

    fn push_body() -> Vec<u8> {
        include_bytes!("../tests/fixtures/push-snapshot.json").to_vec()
    }

    async fn send(state: ReceiverState, req: Request<axum::body::Body>) -> (StatusCode, String) {
        let resp = router(state).oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        (status, String::from_utf8_lossy(&bytes).to_string())
    }

    #[tokio::test]
    async fn valid_signed_pr_dispatches_202() {
        let body = pr_body();
        let req = Request::post(WEBHOOK_PATH)
            .header("x-forgejo-event", "pull_request")
            .header("x-forgejo-delivery", "uuid-1")
            .header("x-hub-signature-256", sign(&body))
            .body(axum::body::Body::from(body.clone()))
            .unwrap();
        let (status, text) = send(test_state(), req).await;
        assert_eq!(status, StatusCode::ACCEPTED);
        assert!(text.contains("\"status\":\"dispatched\""));
        assert!(text.contains("\"subject_kind\":\"pull_request\""));
        assert!(text.contains("\"pr_number\":7"));
        assert!(text.contains("oya-pr-review")); // the honest boundary
    }

    #[tokio::test]
    async fn valid_signed_issue_dispatches_snapshot_202() {
        let body = issue_body();
        let req = Request::post(WEBHOOK_PATH)
            .header("x-forgejo-event", "issues")
            .header("x-forgejo-delivery", "uuid-issue")
            .header("x-hub-signature-256", sign(&body))
            .body(axum::body::Body::from(body.clone()))
            .unwrap();
        let (status, text) = send(test_state(), req).await;
        assert_eq!(status, StatusCode::ACCEPTED);
        assert!(text.contains("\"subject_kind\":\"issue\""));
        assert!(text.contains("\"issue_number\":108"));
        assert!(text.contains("\"snapshot_id\":\"issue:sha256:"));
    }

    #[tokio::test]
    async fn valid_signed_push_dispatches_snapshot_202() {
        let body = push_body();
        let req = Request::post(WEBHOOK_PATH)
            .header("x-forgejo-event", "push")
            .header("x-forgejo-delivery", "uuid-push")
            .header("x-hub-signature-256", sign(&body))
            .body(axum::body::Body::from(body.clone()))
            .unwrap();
        let (status, text) = send(test_state(), req).await;
        assert_eq!(status, StatusCode::ACCEPTED);
        assert!(text.contains("\"subject_kind\":\"push\""));
        assert!(text.contains("\"reference\":\"refs/heads/claims/ADR-0377-D2\""));
        assert!(text.contains("\"snapshot_id\":\"push:sha256:"));
        assert!(text.contains("\"kicked_through\":\"oya-board-projection\""));
    }

    #[tokio::test]
    async fn bad_signature_is_401_and_never_dispatches() {
        let body = pr_body();
        let req = Request::post(WEBHOOK_PATH)
            .header("x-forgejo-event", "pull_request")
            .header("x-hub-signature-256", "sha256=deadbeef")
            .body(axum::body::Body::from(body))
            .unwrap();
        let (status, text) = send(test_state(), req).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert!(text.contains("rejected"));
    }

    #[tokio::test]
    async fn missing_signature_is_401() {
        let body = pr_body();
        let req = Request::post(WEBHOOK_PATH)
            .header("x-forgejo-event", "pull_request")
            .body(axum::body::Body::from(body))
            .unwrap();
        let (status, _text) = send(test_state(), req).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn wrong_base_branch_is_200_ignored() {
        let body = br#"{"action":"opened","number":7,
            "pull_request":{"number":7,
              "base":{"ref":"main","sha":"b"},
              "head":{"ref":"f","sha":"abc"},"draft":false}}"#
            .to_vec();
        let req = Request::post(WEBHOOK_PATH)
            .header("x-forgejo-event", "pull_request")
            .header("x-hub-signature-256", sign(&body))
            .body(axum::body::Body::from(body))
            .unwrap();
        let (status, text) = send(test_state(), req).await;
        assert_eq!(status, StatusCode::OK);
        assert!(text.contains("ignored"));
    }

    #[tokio::test]
    async fn unknown_event_is_422() {
        let body = b"{}".to_vec();
        let req = Request::post(WEBHOOK_PATH)
            .header("x-forgejo-event", "wiki")
            .header("x-hub-signature-256", sign(&body))
            .body(axum::body::Body::from(body))
            .unwrap();
        let (status, _text) = send(test_state(), req).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn ping_is_200() {
        let body = b"{}".to_vec();
        let req = Request::post(WEBHOOK_PATH)
            .header("x-forgejo-event", "ping")
            .header("x-hub-signature-256", sign(&body))
            .body(axum::body::Body::from(body))
            .unwrap();
        let (status, _text) = send(test_state(), req).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn healthz_ok() {
        let req = Request::get(HEALTHZ_PATH)
            .body(axum::body::Body::empty())
            .unwrap();
        let (status, _text) = send(test_state(), req).await;
        assert_eq!(status, StatusCode::OK);
    }
}
