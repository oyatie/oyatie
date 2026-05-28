//! # oya-ci-webhook-gateway-app
//!
//! axum application library for the CI webhook gateway (ADR-0387).
//!
//! Exposes:
//! - `POST /webhook/forgejo` — full pipeline handler
//! - `GET  /healthz`         — liveness probe
//! - `GET  /metrics`         — metrics stub (Prometheus format, Stage-8 wires OTel)
//!
//! ## Handler pipeline
//!
//! 1. Extract `X-Forgejo-Signature-256`, `X-Forgejo-Event`, `X-Forgejo-Delivery`,
//!    `X-Forgejo-Timestamp` headers.
//! 2. ed25519 verify raw body bytes.
//! 3. Cedar authz gate.
//! 4. `route_forgejo_event` → `CiTriggerEvent`.
//! 5. Jenkins `trigger` → `JenkinsJob`.
//! 6. GitHub `post_all` pending statuses.
//! 7. Jenkins `poll_status` loop.
//! 8. GitHub `post_all` final statuses.
//!
//! ## ADR-0083 Tier-3
//!
//! No `unwrap`/`expect`/`panic` on the request path.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use oya_ci_webhook_gateway_kernel::{
    CommitStatusPoster, JenkinsClient, RouteOutcome, SignatureVerifier, WebhookAuthzGate,
    WebhookAuthzRequest, WebhookSignature, route_forgejo_event,
};
use std::sync::Arc;
use tracing::{error, info, warn};

/// Shared application state — all adapters are behind `Arc<dyn Trait>` so the
/// router state is `Clone + Send + Sync`.
#[derive(Clone)]
pub struct AppState {
    pub verifier: Arc<dyn SignatureVerifier + Send + Sync>, // data_class: INTERNAL_ONLY
    pub authz: Arc<dyn WebhookAuthzGate + Send + Sync>,     // data_class: INTERNAL_ONLY
    pub jenkins: Arc<dyn JenkinsClient + Send + Sync>,      // data_class: INTERNAL_ONLY
    pub status_poster: Arc<dyn CommitStatusPoster + Send + Sync>, // data_class: INTERNAL_ONLY
    pub target_branch: String,                              // data_class: INTERNAL_ONLY
    pub github_owner: String,                               // data_class: INTERNAL_ONLY
    pub github_repo: String,                                // data_class: INTERNAL_ONLY
    pub jenkins_job_name: String,                           // data_class: INTERNAL_ONLY
}

/// Build the axum [`Router`] with all routes and shared state.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/webhook/forgejo", post(handle_forgejo_webhook))
        .route("/healthz", get(handle_healthz))
        .route("/metrics", get(handle_metrics))
        .with_state(state)
}

/// `GET /healthz` — liveness probe.
async fn handle_healthz() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

/// `GET /metrics` — Prometheus text format stub.
/// Stage-8 wires the real OTel/Prometheus exporter here.
async fn handle_metrics() -> impl IntoResponse {
    (
        StatusCode::OK,
        "# HELP ci_webhook_gateway_up Gateway liveness\n# TYPE ci_webhook_gateway_up gauge\nci_webhook_gateway_up 1\n",
    )
}

/// `POST /webhook/forgejo` — main webhook handler.
///
/// Returns:
/// - `202 Accepted` on a routable event that was dispatched to Jenkins.
/// - `200 OK` on an ignorable event (ping, non-target-branch, draft PR, etc.).
/// - `400 Bad Request` on signature / payload errors.
/// - `403 Forbidden` on Cedar authz denial.
/// - `502 Bad Gateway` on Jenkins / GitHub downstream failures.
async fn handle_forgejo_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let delivery_id = header_str(&headers, "x-forgejo-delivery").unwrap_or("unknown");
    let event_type = header_str(&headers, "x-forgejo-event").unwrap_or("");
    let source_ip = header_str(&headers, "x-real-ip")
        .or_else(|| header_str(&headers, "x-forwarded-for"))
        .unwrap_or("0.0.0.0");

    // Step 1 — extract signature.
    let sig_header = match header_str(&headers, "x-hub-signature-256")
        .or_else(|| header_str(&headers, "x-forgejo-signature"))
    {
        Some(h) => h.to_owned(),
        None => {
            warn!(delivery_id, "missing signature header");
            return (StatusCode::BAD_REQUEST, "missing signature").into_response();
        }
    };

    let sig_hex = sig_header
        .strip_prefix("sha256=")
        .unwrap_or(sig_header.as_str());

    let signature = match WebhookSignature::from_hex(sig_hex) {
        Ok(s) => s,
        Err(_) => {
            warn!(delivery_id, "malformed signature header");
            return (StatusCode::BAD_REQUEST, "malformed signature").into_response();
        }
    };

    let timestamp_s =
        header_str(&headers, "x-forgejo-timestamp").and_then(|v| v.parse::<u64>().ok());

    // Step 2 — ed25519 verify (raw bytes, before JSON parse).
    if let Err(e) = state.verifier.verify(&body, &signature, timestamp_s) {
        warn!(delivery_id, error = %e, "signature verification failed");
        return (StatusCode::BAD_REQUEST, "signature verification failed").into_response();
    }

    // Step 3 — Cedar authz gate.
    let authz_req = WebhookAuthzRequest {
        tenant_id: "oyatie-dogfood".to_owned(), // populated from JWT in Stage-7
        source_ip: source_ip.to_owned(),
        event_type: event_type.to_owned(),
        repo: format!("{}/{}", state.github_owner, state.github_repo),
    };
    if matches!(
        state.authz.decide(&authz_req),
        oya_ci_webhook_gateway_kernel::AuthzDecision::Forbid
    ) {
        warn!(delivery_id, "Cedar authz denied webhook trigger");
        return (StatusCode::FORBIDDEN, "forbidden by policy").into_response();
    }

    // Step 4 — route event.
    let outcome = match route_forgejo_event(event_type, &body, delivery_id, &state.target_branch) {
        Ok(o) => o,
        Err(e) => {
            warn!(delivery_id, error = %e, "event routing failed");
            return (StatusCode::BAD_REQUEST, format!("{e}")).into_response();
        }
    };

    let mut event = match outcome {
        RouteOutcome::Ignored { reason } => {
            info!(delivery_id, reason, "event ignored");
            return (StatusCode::OK, format!("ignored: {reason}")).into_response();
        }
        RouteOutcome::Trigger(ev) => ev,
    };

    // Populate repo from app config.
    event.repo = format!("{}/{}", state.github_owner, state.github_repo);

    // Step 5 — Jenkins trigger.
    let job = match state.jenkins.trigger(&state.jenkins_job_name, &event) {
        Ok(j) => j,
        Err(e) => {
            error!(delivery_id, error = %e, "jenkins trigger failed");
            return (StatusCode::BAD_GATEWAY, format!("{e}")).into_response();
        }
    };

    info!(
        delivery_id,
        build_number = job.build_number,
        "jenkins build triggered"
    );

    // Step 6 — post pending statuses to GitHub.
    let build_url = job.build_url.as_deref();
    if let Err(e) = state.status_poster.post_all(
        &state.github_owner,
        &state.github_repo,
        &event.head_sha,
        oya_ci_webhook_gateway_kernel::JobStatus::Running,
        build_url,
    ) {
        warn!(delivery_id, error = %e, "failed to post pending GitHub statuses");
        // Non-fatal: continue with Jenkins polling.
    }

    // Step 7+8 — poll + post final statuses.
    // This is done synchronously in the handler.  Stage-9 will move this to a
    // background task queue so the HTTP response returns before the build
    // completes.
    let final_status = match state.jenkins.poll_status(&job) {
        Ok(s) => s,
        Err(e) => {
            error!(delivery_id, error = %e, "jenkins poll failed");
            oya_ci_webhook_gateway_kernel::JobStatus::Unknown
        }
    };

    if let Err(e) = state.status_poster.post_all(
        &state.github_owner,
        &state.github_repo,
        &event.head_sha,
        final_status,
        build_url,
    ) {
        error!(delivery_id, error = %e, "failed to post final GitHub statuses");
    }

    info!(delivery_id, status = ?final_status, "pipeline complete");
    (StatusCode::ACCEPTED, "dispatched").into_response()
}

/// Extract a header value as a `&str`, returning `None` on absent / non-UTF8.
fn header_str<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name)?.to_str().ok()
}
