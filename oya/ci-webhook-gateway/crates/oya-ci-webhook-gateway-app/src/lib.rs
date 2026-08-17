//! # oya-ci-webhook-gateway-app
//!
//! axum application library for the CI webhook gateway (ADR-0387).
//!
//! Exposes:
//! - `POST /webhook/github` — full pipeline handler
//! - `GET  /healthz`         — liveness probe
//! - `GET  /metrics`         — metrics stub (Prometheus format, Stage-8 wires OTel)
//!
//! ## Handler pipeline
//!
//! 1. Extract `X-GitHub-Signature-256`, `X-GitHub-Event`, `X-GitHub-Delivery`,
//!    `X-GitHub-Timestamp` headers.
//! 2. ed25519 verify raw body bytes.
//! 3. Cedar authz gate.
//! 4. `route_github_event` → `CiTriggerEvent`.
//! 4.5. Replay guard: check + record the delivery key (after verify+authz+route, before Step 5).
//!      A replay within the TTL short-circuits with a benign 200 idempotent ack.
//! 5. GitHub `post_all` queued statuses (CI system dispatches asynchronously via oya-ci).
//!
//! ## ADR-0083 Tier-3
//!
//! No `unwrap`/`expect`/`panic` on the request path.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod replay;

use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use oya_ci_webhook_gateway_kernel::{
    CommitStatusPoster, JobStatus, RouteOutcome, SignatureVerifier, WebhookAuthzGate,
    WebhookAuthzRequest, WebhookSignature, route_github_event,
};
use replay::{DeliveryGuard, DeliveryKey, Verdict};
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

/// Shared application state — all adapters are behind `Arc<dyn Trait>` so the
/// router state is `Clone + Send + Sync`.
#[derive(Clone)]
pub struct AppState {
    pub verifier: Arc<dyn SignatureVerifier + Send + Sync>, // data_class: INTERNAL_ONLY
    pub authz: Arc<dyn WebhookAuthzGate + Send + Sync>,     // data_class: INTERNAL_ONLY
    pub status_poster: Arc<dyn CommitStatusPoster + Send + Sync>, // data_class: INTERNAL_ONLY
    pub target_branch: String,                              // data_class: INTERNAL_ONLY
    pub github_owner: String,                               // data_class: INTERNAL_ONLY
    pub github_repo: String,                                // data_class: INTERNAL_ONLY
    /// Delivery-replay / dedup guard.  Shared across all handler instances via
    /// `Arc<Mutex<_>>`.  The `Mutex` provides interior mutability so the guard
    /// can live behind the `Clone`-able, `Send + Sync` `AppState`.
    pub delivery_guard: Arc<Mutex<DeliveryGuard>>, // data_class: INTERNAL_ONLY
}

/// Build the axum [`Router`] with all routes and shared state.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/webhook/github", post(handle_github_webhook))
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

/// `POST /webhook/github` — main webhook handler.
///
/// Returns:
/// - `202 Accepted` on a routable event that was accepted and queued statuses posted.
/// - `200 OK` on an ignorable event (ping, non-target-branch, draft PR, etc.).
/// - `400 Bad Request` on signature / payload errors.
/// - `403 Forbidden` on Cedar authz denial.
/// - `502 Bad Gateway` on GitHub downstream failures.
async fn handle_github_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let delivery_id = header_str(&headers, "x-github-delivery").unwrap_or("unknown");
    let event_type = header_str(&headers, "x-github-event").unwrap_or("");
    let source_ip = header_str(&headers, "x-real-ip")
        .or_else(|| header_str(&headers, "x-forwarded-for"))
        .unwrap_or("0.0.0.0");

    // Step 1 — extract signature.
    let sig_header = match header_str(&headers, "x-hub-signature-256")
        .or_else(|| header_str(&headers, "x-github-signature"))
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
        header_str(&headers, "x-github-timestamp").and_then(|v| v.parse::<u64>().ok());

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
    let outcome = match route_github_event(event_type, &body, delivery_id, &state.target_branch) {
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

    // Step 4.5 — Replay / dedup guard (after verify+authz+route, before status post).
    //
    // Record-on-receipt: the key is recorded BEFORE any downstream action so that a
    // concurrent replay of the same delivery is deduped even while the first is still
    // in-flight.  See replay module docs for the retry-on-failure trade-off.
    //
    // Opportunistic prune: run on every call; cost is O(n) over the seen map
    // which stays small (TTL=5 min, typical delivery rate is low).
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let delivery_key = DeliveryKey::from_parts(
        &event.delivery_id,
        &event.head_sha,
        event.pr_number,
        event.action,
    );

    let verdict = {
        let mut guard = state
            .delivery_guard
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        guard.prune(now_ms);
        guard.record_and_check(delivery_key, now_ms)
    };

    if matches!(verdict, Verdict::Replay) {
        info!(
            delivery_id,
            "duplicate delivery, already accepted (idempotent ack)"
        );
        return (StatusCode::OK, "duplicate delivery, already accepted").into_response();
    }

    // Step 5 — post queued statuses to GitHub.
    // The oya-ci pipeline picks up the event asynchronously and posts final statuses.
    if let Err(e) = state.status_poster.post_all(
        &state.github_owner,
        &state.github_repo,
        &event.head_sha,
        JobStatus::Queued,
        None,
    ) {
        warn!(delivery_id, error = %e, "failed to post queued GitHub statuses");
        return (StatusCode::BAD_GATEWAY, format!("{e}")).into_response();
    }

    info!(delivery_id, head_sha = %event.head_sha, "webhook accepted, queued statuses posted");
    (StatusCode::ACCEPTED, "dispatched").into_response()
}

/// Extract a header value as a `&str`, returning `None` on absent / non-UTF8.
fn header_str<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name)?.to_str().ok()
}
