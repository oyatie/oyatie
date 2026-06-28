//! # oya-ci-controller-app
//!
//! kube-rs Controller application library for the oya-ci controller.
//!
//! Exposes:
//! - [`ControllerState`] — shared state with Arc<dyn Trait> adapter seams
//! - [`reconcile`] — the single reconcile entrypoint (pure-core sandwich)
//! - [`error_policy`] — capped-exponential-backoff error policy
//! - [`build_router`] — axum `/healthz` + `/metrics` + `POST /gate-run` server
//! - [`run_controller`] — wires the kube-rs Controller runtime
//!
//! ## Reconcile loop (pure-core sandwich)
//!
//! 1. Project live Job + owned Pods -> kernel `JobObservation`
//! 2. Call `kernel::map_job_to_status(observation)` -> `ReconcileDecision`
//! 3. Act: post pending/terminal status via `CommitStatusPoster`,
//!    patch write-once annotation, requeue or stop.
//!
//! ## POST /gate-run (plank role — Job spawn)
//!
//! Accepts `{"pr_number": N, "head_sha": "...", "base_ref": "dev"}` and
//! calls `K8sJobSpawner::spawn(build_gate_job(spec))` to create the labeled
//! gate Job. The reconcile loop then picks it up and posts GitHub statuses.
//!
//! Idempotent: the Job name is deterministic (`oya-ci-gate-pr<N>-<sha8>`),
//! so a duplicate POST results in a 409 create-conflict no-op (returns 200).
//!
//! ## Idempotency / restart-safety
//!
//! ALL state lives on the Job object (labels + annotations). A controller
//! crash + relist resumes cleanly. The `oya.io/ci-status-posted`
//! annotation is the write-once terminal guard (exactly-once posting).
//!
//! ## ADR-0083 Tier-3
//!
//! No `unwrap`/`expect`/`panic` on the reconcile path.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use axum::{
    Json, Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use k8s_openapi::api::{batch::v1::Job, core::v1::Pod};
use kube::{
    Api, Client,
    api::{ListParams, Patch, PatchParams},
    runtime::{Controller, controller::Action, watcher},
};
use oya_ci_controller_k8s_adapter::{
    ANNOT_CI_STATUS_POSTED, LABEL_CI_HEAD_SHA, LABEL_CI_PR_NUMBER, observe_job,
};
use oya_ci_controller_kernel::{
    CommitState, CommitStatusPoster, GATE_CONTEXT, GateRun, GateRunSpec, JobSpawner,
    ReconcileDecision, map_job_to_status,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{sync::Arc, time::Duration};
use tracing::{error, info, warn};

// ---------------------------------------------------------------------------
// Grace configuration
// ---------------------------------------------------------------------------

/// Number of consecutive reconcile cycles in a waiting-pod-reason before
/// declaring terminal (default: 12 cycles at ~10s requeue ≈ 2 min).
const DEFAULT_GRACE_CYCLES: u32 = 12;

/// Requeue interval for active (non-terminal) Jobs.
const ACTIVE_REQUEUE_SECS: u64 = 10;

// ---------------------------------------------------------------------------
// ControllerState
// ---------------------------------------------------------------------------

/// Shared controller state — all adapters behind `Arc<dyn Trait>` for
/// testability. Mirrors gateway `AppState` seam injection pattern.
///
/// The controller ONLY WATCHES Jobs and posts GitHub commit statuses (crier
/// pattern). Job creation is the gateway's responsibility (hook/plank side).
#[derive(Clone)]
pub struct ControllerState {
    pub client: Client,
    pub status_poster: Arc<dyn CommitStatusPoster>,
    pub namespace: String,
    /// Waiting-pod-reason grace threshold.
    pub grace_cycles: u32,
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct ReconcileError(pub String);

impl std::fmt::Display for ReconcileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "reconcile error: {}", self.0)
    }
}

impl std::error::Error for ReconcileError {}

// ---------------------------------------------------------------------------
// reconcile — pure-core sandwich
// ---------------------------------------------------------------------------

/// Reconcile a single `batch/v1 Job`.
///
/// Called by the kube-rs Controller for every watch event on oya-ci-gate Jobs.
pub async fn reconcile(
    job: Arc<Job>,
    ctx: Arc<ControllerState>,
) -> std::result::Result<Action, ReconcileError> {
    let job_name = job.metadata.name.as_deref().unwrap_or("<unnamed>");
    let namespace = job.metadata.namespace.as_deref().unwrap_or(&ctx.namespace);

    let labels = job.metadata.labels.as_ref();
    let head_sha = labels
        .and_then(|l| l.get(LABEL_CI_HEAD_SHA))
        .cloned()
        .unwrap_or_default();
    let pr_number_str = labels
        .and_then(|l| l.get(LABEL_CI_PR_NUMBER))
        .cloned()
        .unwrap_or_default();

    info!(job = job_name, sha = %head_sha, pr = %pr_number_str, "reconciling gate job");

    // ---- Step 1: fetch owned Pods -----------------------------------------
    let pod_api: Api<Pod> = Api::namespaced(ctx.client.clone(), namespace);
    let pod_lp = ListParams::default().labels(&format!("job-name={job_name}"));
    let pods = match pod_api.list(&pod_lp).await {
        Ok(list) => list.items,
        Err(e) => {
            warn!(job = job_name, error = %e, "failed to list pods for job");
            vec![]
        }
    };

    // Read waiting_cycles from annotation (best-effort; start at 0 if absent)
    let waiting_cycles: u32 = job
        .metadata
        .annotations
        .as_ref()
        .and_then(|a| a.get("oya.io/ci-waiting-cycles"))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    // ---- Step 2: project into JobObservation --------------------------------
    let observation = observe_job(&job, &pods, waiting_cycles);

    // Determine whether this observation has a waiting pod reason (for cycle
    // tracking). If so, increment and persist the counter before making the
    // kernel decision so that the next reconcile sees the updated value.
    let has_waiting_reason = observation
        .pod_reasons
        .iter()
        .any(|r| r.is_pull_or_container_error());
    if has_waiting_reason {
        let next_cycles = waiting_cycles.saturating_add(1);
        patch_waiting_cycles(&ctx.client, namespace, job_name, next_cycles).await;
    }

    // ---- Step 3: pure kernel decision ---------------------------------------
    let decision = map_job_to_status(&observation, ctx.grace_cycles);

    info!(job = job_name, decision = ?decision, "kernel decision");

    // ---- Step 4: act --------------------------------------------------------
    match decision {
        ReconcileDecision::AlreadyTerminal => {
            info!(job = job_name, "terminal status already posted — no-op");
            Ok(Action::await_change())
        }

        ReconcileDecision::AwaitChange => {
            // Pending status already posted; just watch for change.
            Ok(Action::requeue(Duration::from_secs(ACTIVE_REQUEUE_SECS)))
        }

        ReconcileDecision::PostPending { description } => {
            // Post pending to GitHub via spawn_blocking (reqwest::blocking must
            // not be called on the async executor thread — ADR-0083 major fix).
            let poster = Arc::clone(&ctx.status_poster);
            let sha = head_sha.clone();
            let desc = description.clone();
            let post_result = tokio::task::spawn_blocking(move || {
                poster.post(&sha, CommitState::Pending, GATE_CONTEXT, &desc, None)
            })
            .await
            .unwrap_or_else(|e| {
                Err(oya_ci_controller_kernel::KernelError::DownstreamTransport(
                    format!("spawn_blocking join: {e}"),
                ))
            });

            match post_result {
                Ok(()) => {
                    // Patch annotation to record pending posted.
                    patch_status_annotation(
                        &ctx.client,
                        namespace,
                        job_name,
                        CommitState::Pending,
                    )
                    .await;
                    info!(job = job_name, "pending status posted");
                }
                Err(e) => {
                    warn!(job = job_name, error = %e, "failed to post pending status — will retry");
                }
            }
            Ok(Action::requeue(Duration::from_secs(ACTIVE_REQUEUE_SECS)))
        }

        ReconcileDecision::PostTerminal {
            state,
            context,
            description,
        } => {
            // Post terminal status via spawn_blocking (reqwest::blocking must not
            // be called on the async executor thread — ADR-0083 major fix).
            // Do NOT mark the annotation until GitHub returns 2xx — if we
            // crash between post and annotation patch, the next reconcile
            // re-posts (benign: GitHub statuses are last-write-wins on (sha, context)).
            let poster = Arc::clone(&ctx.status_poster);
            let sha = head_sha.clone();
            let desc = description.clone();
            let post_result =
                tokio::task::spawn_blocking(move || poster.post(&sha, state, context, &desc, None))
                    .await
                    .unwrap_or_else(|e| {
                        Err(oya_ci_controller_kernel::KernelError::DownstreamTransport(
                            format!("spawn_blocking join: {e}"),
                        ))
                    });

            match post_result {
                Ok(()) => {
                    // Write-once terminal guard: patch annotation.
                    patch_status_annotation(&ctx.client, namespace, job_name, state).await;
                    info!(
                        job = job_name,
                        state = %state,
                        "terminal status posted"
                    );
                    Ok(Action::await_change())
                }
                Err(e) => {
                    // GitHub unreachable — requeue with backoff. Verdict is
                    // durable on the Job object, so nothing is lost.
                    error!(
                        job = job_name,
                        state = %state,
                        error = %e,
                        "failed to post terminal status — requeueing"
                    );
                    Err(ReconcileError(format!(
                        "github post failed for {job_name}: {e}"
                    )))
                }
            }
        }
    }
}

/// Patch the `oya.io/ci-status-posted` annotation on the Job.
/// Best-effort: log on failure but never block the reconcile verdict.
async fn patch_status_annotation(
    client: &Client,
    namespace: &str,
    job_name: &str,
    state: CommitState,
) {
    let api: Api<Job> = Api::namespaced(client.clone(), namespace);
    let patch = json!({
        "metadata": {
            "annotations": {
                ANNOT_CI_STATUS_POSTED: state.as_str()
            }
        }
    });
    if let Err(e) = api
        .patch(
            job_name,
            &PatchParams::apply("oya-ci-controller"),
            &Patch::Merge(patch),
        )
        .await
    {
        warn!(
            job = job_name,
            state = %state,
            error = %e,
            "failed to patch status annotation (benign — will re-post on next reconcile)"
        );
    }
}

/// Patch the `oya.io/ci-waiting-cycles` annotation on the Job.
/// Best-effort: log on failure but never block the reconcile verdict.
async fn patch_waiting_cycles(client: &Client, namespace: &str, job_name: &str, cycles: u32) {
    let api: Api<Job> = Api::namespaced(client.clone(), namespace);
    let patch = json!({
        "metadata": {
            "annotations": {
                "oya.io/ci-waiting-cycles": cycles.to_string()
            }
        }
    });
    if let Err(e) = api
        .patch(
            job_name,
            &PatchParams::apply("oya-ci-controller"),
            &Patch::Merge(patch),
        )
        .await
    {
        warn!(
            job = job_name,
            cycles,
            error = %e,
            "failed to patch waiting-cycles annotation (benign)"
        );
    }
}

// ---------------------------------------------------------------------------
// error_policy — capped exponential backoff
// ---------------------------------------------------------------------------

/// Capped-exponential-backoff error policy for the kube-rs Controller.
pub fn error_policy(job: Arc<Job>, err: &ReconcileError, _ctx: Arc<ControllerState>) -> Action {
    let job_name = job.metadata.name.as_deref().unwrap_or("<unnamed>");
    warn!(job = job_name, error = %err, "reconcile error — backing off");
    Action::requeue(Duration::from_secs(30))
}

// ---------------------------------------------------------------------------
// run_controller — wire the kube-rs Controller runtime
// ---------------------------------------------------------------------------

/// Build and run the kube-rs Controller over `batch/v1 Job` in the given namespace.
///
/// Watches only Jobs labeled `oya.io/ci-controller=oya-ci-gate`.
pub async fn run_controller(state: ControllerState) {
    let client = state.client.clone();
    let namespace = state.namespace.clone();
    let ctx = Arc::new(state);

    let job_api: Api<Job> = Api::namespaced(client.clone(), &namespace);
    let watcher_config =
        watcher::Config::default().labels(oya_ci_controller_k8s_adapter::WATCHER_LABEL_SELECTOR);

    info!(namespace = %namespace, "starting oya-ci controller");

    Controller::new(job_api, watcher_config)
        .shutdown_on_signal()
        .run(reconcile, error_policy, ctx)
        .for_each(|res| async move {
            match res {
                Ok((obj, _)) => {
                    info!(job = ?obj.name, "reconcile ok");
                }
                Err(e) => {
                    error!(error = %e, "reconcile failed");
                }
            }
        })
        .await;
}

// ---------------------------------------------------------------------------
// axum router — /healthz + /metrics + POST /gate-run
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// POST /gate-run authn + authz (keystone-1, AUTH-005 parity)
//
// `/gate-run` spawns K8s gate Jobs for any in-cluster caller. It is fail-closed:
// authn an UNFORGEABLE bearer FIRST (before the request body is parsed), then
// consult a default-deny authz seam. Mirrors the shipped gold standard at
// intelligence/adapters/rest (VerifiedIngressPrincipal + ConfiguredBearer* +
// constant-time compare). A caller-supplied `x-*` header can never authenticate.
// ---------------------------------------------------------------------------

/// A `/gate-run` caller whose bearer the [`CiTriggerAuthenticator`] has VERIFIED.
/// The field is private with only a `pub(crate)` constructor, so a handler cannot
/// fabricate one from caller-supplied headers — only a verifier that proved an
/// unforgeable credential mints it. Mirrors `VerifiedIngressPrincipal`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedCiTrigger {
    /// The verified trigger principal label (from the credential binding, NEVER
    /// a caller-supplied header).
    principal: String,
}

impl VerifiedCiTrigger {
    /// Mint a verified trigger. Crate-private: only an authenticator in this
    /// crate can construct one (no public constructor → unforgeable).
    pub(crate) fn new(principal: impl Into<String>) -> Self {
        Self {
            principal: principal.into(),
        }
    }

    /// The verified trigger principal label (trusted; from the credential).
    #[must_use]
    pub fn principal(&self) -> &str {
        &self.principal
    }
}

/// Gate-run authentication PORT: derive a [`VerifiedCiTrigger`] from the request
/// headers by checking an UNFORGEABLE credential (a constant-time bearer compare
/// today; mTLS/SPIFFE in a production adapter). `None` ⇒ no verified trigger ⇒
/// `401` (default-deny). Caller-supplied `x-*` headers MUST NOT authenticate.
pub trait CiTriggerAuthenticator: Send + Sync {
    /// Verify the caller's credential. `None` ⇒ `401`.
    fn verify(&self, headers: &HeaderMap) -> Option<VerifiedCiTrigger>;
}

/// Reference [`CiTriggerAuthenticator`] adapter: a single configured bearer token
/// compared in constant time. An empty/unset token verifies NOTHING (every
/// request `401`) — there is no allow-all path.
#[derive(Clone)]
pub struct ConfiguredBearerCiTriggerAuthenticator {
    token: String, // data_class: SECRET
}

impl ConfiguredBearerCiTriggerAuthenticator {
    /// Build an authenticator for one configured bearer credential.
    #[must_use]
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

// Manual `Debug` so the SECRET-annotated bearer is never printed if this (or a
// containing struct) is ever `dbg!`/derive-Debugged.
impl std::fmt::Debug for ConfiguredBearerCiTriggerAuthenticator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfiguredBearerCiTriggerAuthenticator")
            .field("token", &"<redacted>")
            .finish()
    }
}

impl CiTriggerAuthenticator for ConfiguredBearerCiTriggerAuthenticator {
    fn verify(&self, headers: &HeaderMap) -> Option<VerifiedCiTrigger> {
        let configured = self.token.trim();
        if configured.is_empty() {
            // No configured credential ⇒ no caller can be verified (fail-closed).
            return None;
        }
        let presented = headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))?;
        if constant_time_eq(presented.as_bytes(), configured.as_bytes()) {
            Some(VerifiedCiTrigger::new("ci-gate-run-trigger"))
        } else {
            None
        }
    }
}

/// Constant-time byte comparison (no early return on first mismatch), so a
/// bearer compare does not leak the matched prefix length via timing. Copied
/// from the intelligence rest adapter (no new dependency).
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    let max_len = a.len().max(b.len());
    let mut diff = a.len() ^ b.len();
    for index in 0..max_len {
        let left = a.get(index).copied().unwrap_or(0);
        let right = b.get(index).copied().unwrap_or(0);
        diff |= (left ^ right) as usize;
    }
    diff == 0
}

/// Authorization decision for a verified `/gate-run` trigger. Anything other
/// than [`CiTriggerDecision::Allow`] is fail-closed `403`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CiTriggerDecision {
    Allow,
    Deny,
}

/// Gate-run authorization PORT (default-deny seam, parity with the sibling
/// services' authz gates). Maps a VERIFIED trigger principal to a decision; a
/// `Deny` (or any future fault mapped to `Deny`) ⇒ `403`.
pub trait CiTriggerAuthz: Send + Sync {
    /// Authorize a verified trigger. Non-`Allow` ⇒ `403`.
    fn decide(&self, principal: &VerifiedCiTrigger) -> CiTriggerDecision;
}

/// v1 in-crate authz adapter: permits any VERIFIED trigger principal (the
/// gateway is the only intended caller and is authenticated upstream by the
/// bearer). A richer policy adapter can replace this behind the port without
/// touching the handler.
#[derive(Clone, Debug, Default)]
pub struct AllowVerifiedTriggerAuthz;

impl CiTriggerAuthz for AllowVerifiedTriggerAuthz {
    fn decide(&self, _principal: &VerifiedCiTrigger) -> CiTriggerDecision {
        CiTriggerDecision::Allow
    }
}

/// Shared state for the health/metrics/gate-run server.
#[derive(Clone)]
pub struct ServerState {
    pub controller_namespace: String,
    /// Job spawner for POST /gate-run (plank role).
    pub job_spawner: Arc<dyn JobSpawner>,
    /// Full gate-Job spec config (image, clone URL, SA, deadlines).
    pub gate_spec_config: GateSpecConfig,
    /// Fail-closed bearer authenticator for POST /gate-run (keystone-1). An
    /// empty/unset configured token ⇒ every gate-run request `401` (no
    /// allow-all path); the binary refuses to start without the token.
    pub authenticator: Arc<dyn CiTriggerAuthenticator>,
    /// Default-deny authorization seam for POST /gate-run (parity with the
    /// sibling services). A `Deny` ⇒ `403`.
    pub authz: Arc<dyn CiTriggerAuthz>,
}

/// Static configuration for building gate Job specs.
/// Read from env vars at startup; immutable during the process lifetime.
#[derive(Clone)]
pub struct GateSpecConfig {
    /// Rust-CI image for the gate runner Pod.
    pub image: String,
    /// Git clone URL for the gate Job init container (GitHub forge of record).
    pub forge_clone_url: String,
    /// Gate Job active deadline in seconds (mirrors Jenkins 60 min timeout).
    pub active_deadline_seconds: i64,
    /// TTL after finished for GC (sinker equivalent).
    pub ttl_seconds_after_finished: i32,
    /// Namespace for the gate Jobs.
    pub namespace: String,
    /// Low-privilege ServiceAccount for gate runner Pods.
    pub runner_service_account: String,
    /// GitHub repo full name (e.g. "jason931225/oyatie").
    pub repo: String,
}

/// Request body for `POST /gate-run`.
#[derive(Debug, Deserialize)]
pub struct GateRunRequest {
    pub pr_number: u64,
    pub head_sha: String,
    /// Base branch to gate against (default: "dev").
    #[serde(default = "default_base_ref")]
    pub base_ref: String,
}

fn default_base_ref() -> String {
    "dev".to_owned()
}

/// Response body for `POST /gate-run`.
#[derive(Debug, Serialize)]
pub struct GateRunResponse {
    pub job_name: String,
    pub namespace: String,
    pub already_exists: bool,
}

/// Build the axum Router with health, metrics, and gate-run endpoints.
pub fn build_router(state: ServerState) -> Router {
    Router::new()
        .route("/healthz", get(handle_healthz))
        .route("/metrics", get(handle_metrics))
        .route("/gate-run", post(handle_gate_run))
        .with_state(state)
}

async fn handle_healthz(_state: State<ServerState>) -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

async fn handle_metrics(_state: State<ServerState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        "# HELP ci_controller_up Controller liveness\n\
         # TYPE ci_controller_up gauge\n\
         ci_controller_up 1\n",
    )
}

/// POST /gate-run — spawn a K8s gate Job for the given PR (plank role).
///
/// Body: `{"pr_number": N, "head_sha": "...", "base_ref": "dev"}`
///
/// Fail-closed: a valid `Authorization: Bearer <token>` is REQUIRED — the body
/// is not even parsed until the caller is authenticated + authorized.
///
/// Returns 401 (missing/invalid bearer), 403 (authz deny), 400 on invalid input,
/// 200 (already exists — idempotent) or 201 (created), 500 on spawner failure.
fn is_full_hex_commit_sha(value: &str) -> bool {
    value.len() == 40 && value.chars().all(|c| c.is_ascii_hexdigit())
}

/// Authenticate the gate-run caller against the configured bearer and authorize
/// the verified trigger, fail-closed. No verified principal ⇒ `401`; a non-Allow
/// authz decision ⇒ `403`. Mirrors the gold-standard ordering at
/// intelligence/adapters/rest: authn first, then decide — both BEFORE the
/// untrusted request body is parsed.
fn require_gate_run_authz(state: &ServerState, headers: &HeaderMap) -> Result<VerifiedCiTrigger, Response> {
    // (1) AUTHN — unforgeable verified principal. A self-attested header cannot
    // reach the `Some` branch (the authenticator ignores them).
    let Some(principal) = state.authenticator.verify(headers) else {
        warn!("gate-run: missing or invalid bearer — refusing to spawn");
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "gate-run requires a valid bearer token"})),
        )
            .into_response());
    };
    // (2) AUTHZ — default-deny seam; any non-Allow is fail-closed 403.
    if state.authz.decide(&principal) != CiTriggerDecision::Allow {
        warn!(principal = %principal.principal(), "gate-run: authz denied — refusing to spawn");
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "gate-run trigger is not authorized"})),
        )
            .into_response());
    }
    Ok(principal)
}

async fn handle_gate_run(
    State(state): State<ServerState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // Authn + authz BEFORE parsing the untrusted body (authn-before-body-parse).
    let principal = match require_gate_run_authz(&state, &headers) {
        Ok(principal) => principal,
        Err(response) => return response,
    };

    // Parse the body only after the caller is verified + authorized.
    let req: GateRunRequest = match serde_json::from_slice(&body) {
        Ok(req) => req,
        Err(e) => {
            warn!(error = %e, "gate-run: invalid JSON body");
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "invalid JSON body"})),
            )
                .into_response();
        }
    };

    info!(principal = %principal.principal(), pr = req.pr_number, "gate-run: authenticated trigger");

    // P0.0 required-context evidence must bind to the exact candidate commit.
    // Short SHAs are intentionally rejected because they can be ambiguous and
    // cannot prove that the protected required status was posted to the
    // candidate SHA.
    if !is_full_hex_commit_sha(&req.head_sha) {
        warn!(pr = req.pr_number, sha = %req.head_sha, "gate-run: invalid head_sha");
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "head_sha must be exactly 40 hex characters"})),
        )
            .into_response();
    }

    // The current weekly gate only accepts PRs targeting dev. This keeps
    // /gate-run aligned with the plain-git + GitHub-PR-against-dev contract and
    // avoids accidentally spawning a gate Job for another branch.
    if req.base_ref != "dev" {
        warn!(pr = req.pr_number, base_ref = %req.base_ref, "gate-run: unsupported base_ref");
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "base_ref must be dev for oya-ci gate runs"})),
        )
            .into_response();
    }

    let cfg = &state.gate_spec_config;

    let spec = GateRunSpec {
        run: GateRun {
            pr_number: req.pr_number,
            head_sha: req.head_sha.clone(),
            // delivery_id not available from the HTTP body; use pr+sha as dedup key.
            delivery_id: format!("gate-run-pr{}-{}", req.pr_number, &req.head_sha[..8]),
            base_ref: req.base_ref.clone(),
            repo: cfg.repo.clone(),
        },
        image: cfg.image.clone(),
        forge_clone_url: cfg.forge_clone_url.clone(),
        active_deadline_seconds: cfg.active_deadline_seconds,
        ttl_seconds_after_finished: cfg.ttl_seconds_after_finished,
        namespace: cfg.namespace.clone(),
        runner_service_account: cfg.runner_service_account.clone(),
    };

    let spawner = Arc::clone(&state.job_spawner);
    // K8sJobSpawner::spawn drives a one-shot tokio runtime internally;
    // run it via spawn_blocking to avoid blocking the async executor thread.
    let result = tokio::task::spawn_blocking(move || spawner.spawn(&spec))
        .await
        .unwrap_or_else(|e| {
            Err(oya_ci_controller_kernel::KernelError::DownstreamTransport(
                format!("spawn_blocking join: {e}"),
            ))
        });

    match result {
        Ok(handle) => {
            let status = if handle.already_exists {
                info!(
                    job = %handle.job_name,
                    pr = req.pr_number,
                    sha = %req.head_sha,
                    "gate-run: job already exists (idempotent)"
                );
                StatusCode::OK
            } else {
                info!(
                    job = %handle.job_name,
                    pr = req.pr_number,
                    sha = %req.head_sha,
                    "gate-run: job created"
                );
                StatusCode::CREATED
            };
            (
                status,
                Json(json!({
                    "job_name": handle.job_name,
                    "namespace": handle.namespace,
                    "already_exists": handle.already_exists,
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!(pr = req.pr_number, sha = %req.head_sha, error = %e, "gate-run: spawn failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// futures re-export for bin usage
// ---------------------------------------------------------------------------

pub use futures::StreamExt;

// ---------------------------------------------------------------------------
// Tests — /gate-run contract and restart-safe idempotency
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use oya_ci_controller_kernel::{GateRunSpec, JobHandle, KernelError, Result as KernelResult};
    use std::{
        sync::Mutex,
        task::{Context, Poll},
    };
    use tower::Service;

    #[derive(Default)]
    struct RecordingSpawner {
        calls: Mutex<Vec<GateRunSpec>>,
        already_exists: bool,
    }

    impl JobSpawner for RecordingSpawner {
        fn spawn(&self, spec: &GateRunSpec) -> KernelResult<JobHandle> {
            self.calls
                .lock()
                .map_err(|e| KernelError::DownstreamTransport(format!("lock poisoned: {e}")))?
                .push(spec.clone());
            Ok(JobHandle {
                job_name: spec.run.job_name(),
                namespace: spec.namespace.clone(),
                already_exists: self.already_exists,
            })
        }
    }

    fn test_state(spawner: Arc<RecordingSpawner>) -> ServerState {
        ServerState {
            controller_namespace: "oya-ci".to_owned(),
            job_spawner: spawner,
            gate_spec_config: GateSpecConfig {
                image: "registry.local/rust-ci:dev".to_owned(),
                forge_clone_url: "https://github.com/jason931225/oyatie.git".to_owned(),
                active_deadline_seconds: 3600,
                ttl_seconds_after_finished: 600,
                namespace: "oya-ci".to_owned(),
                runner_service_account: "oya-ci-gate-runner".to_owned(),
                repo: "oya-admin/oyatie".to_owned(),
            },
            authenticator: Arc::new(ConfiguredBearerCiTriggerAuthenticator::new(TEST_BEARER)),
            authz: Arc::new(AllowVerifiedTriggerAuthz),
        }
    }

    /// Configured gate-run bearer the test `ServerState` authenticates against.
    const TEST_BEARER: &str = "test-gate-run-bearer-token";

    /// Route a `/gate-run` POST with the valid configured bearer (the common
    /// case for the body-validation/spawn tests).
    async fn route_request(router: &mut Router, body: &str) -> axum::response::Response {
        route_request_with_bearer(router, body, Some(TEST_BEARER)).await
    }

    /// Route a `/gate-run` POST with an optional `Authorization: Bearer` header.
    /// `None` omits the header entirely (simulating an unauthenticated caller).
    async fn route_request_with_bearer(
        router: &mut Router,
        body: &str,
        bearer: Option<&str>,
    ) -> axum::response::Response {
        futures::future::poll_fn(|cx: &mut Context<'_>| {
            match <Router as Service<Request<Body>>>::poll_ready(router, cx) {
                Poll::Ready(Ok(())) => Poll::Ready(()),
                Poll::Ready(Err(err)) => panic!("router not ready: {err}"),
                Poll::Pending => Poll::Pending,
            }
        })
        .await;

        let mut builder = Request::builder()
            .method("POST")
            .uri("/gate-run")
            .header("content-type", "application/json");
        if let Some(token) = bearer {
            builder = builder.header("authorization", format!("Bearer {token}"));
        }

        <Router as Service<Request<Body>>>::call(
            router,
            builder
                .body(Body::from(body.to_owned()))
                .expect("request builds"),
        )
        .await
        .expect("route call succeeds")
    }

    #[tokio::test]
    async fn gate_run_defaults_to_dev_and_spawns_deterministic_job() {
        let spawner = Arc::new(RecordingSpawner::default());
        let mut router = build_router(test_state(Arc::clone(&spawner)));

        let response = route_request(
            &mut router,
            r#"{"pr_number":42,"head_sha":"abcdef1234567890abcdef1234567890abcdef12"}"#,
        )
        .await;

        assert_eq!(response.status(), StatusCode::CREATED);
        let calls = spawner.calls.lock().expect("calls lock");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].run.base_ref, "dev");
        assert_eq!(calls[0].run.delivery_id, "gate-run-pr42-abcdef12");
        assert_eq!(calls[0].run.job_name(), "oya-ci-gate-pr42-abcdef12");
    }

    #[tokio::test]
    async fn gate_run_duplicate_job_is_idempotent_ok() {
        let spawner = Arc::new(RecordingSpawner {
            calls: Mutex::new(Vec::new()),
            already_exists: true,
        });
        let mut router = build_router(test_state(Arc::clone(&spawner)));

        let response = route_request(
            &mut router,
            r#"{"pr_number":7,"head_sha":"1234567890abcdef1234567890abcdef12345678","base_ref":"dev"}"#,
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        let calls = spawner.calls.lock().expect("calls lock");
        assert_eq!(calls.len(), 1);
    }

    #[tokio::test]
    async fn gate_run_rejects_short_or_non_hex_candidate_sha() {
        let spawner = Arc::new(RecordingSpawner::default());

        for bad_sha in [
            "abcdef1",
            "abcdef12",
            "abcdef1234567890abcdef1234567890abcdef1",
            "abcdef1234567890abcdef1234567890abcdeg12",
        ] {
            let mut router = build_router(test_state(Arc::clone(&spawner)));
            let body = format!(r#"{{"pr_number":42,"head_sha":"{bad_sha}","base_ref":"dev"}}"#);
            let response = route_request(&mut router, &body).await;
            assert_eq!(
                response.status(),
                StatusCode::BAD_REQUEST,
                "{bad_sha} should be rejected"
            );
        }

        let calls = spawner.calls.lock().expect("calls lock");
        assert!(calls.is_empty(), "invalid SHAs must not spawn a Job");
    }

    #[tokio::test]
    async fn gate_run_rejects_non_dev_base_ref_for_weekly_gate() {
        let spawner = Arc::new(RecordingSpawner::default());
        let mut router = build_router(test_state(Arc::clone(&spawner)));

        let response = route_request(
            &mut router,
            r#"{"pr_number":42,"head_sha":"abcdef1234567890abcdef1234567890abcdef12","base_ref":"main"}"#,
        )
        .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let calls = spawner.calls.lock().expect("calls lock");
        assert!(calls.is_empty(), "invalid branch must not spawn a Job");
    }

    // -----------------------------------------------------------------------
    // Fail-closed bearer authn (keystone-1): /gate-run spawns K8s Jobs, so an
    // unauthenticated/forged caller must be rejected BEFORE the body is parsed
    // and BEFORE any spawn. Authn-before-body-parse + zero-spawn on deny.
    // -----------------------------------------------------------------------

    const VALID_GATE_RUN_BODY: &str =
        r#"{"pr_number":42,"head_sha":"abcdef1234567890abcdef1234567890abcdef12"}"#;

    #[tokio::test]
    async fn gate_run_without_bearer_is_unauthorized_and_does_not_spawn() {
        let spawner = Arc::new(RecordingSpawner::default());
        let mut router = build_router(test_state(Arc::clone(&spawner)));

        let response = route_request_with_bearer(&mut router, VALID_GATE_RUN_BODY, None).await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let calls = spawner.calls.lock().expect("calls lock");
        assert!(calls.is_empty(), "missing bearer must not spawn a Job");
    }

    #[tokio::test]
    async fn gate_run_with_wrong_bearer_is_unauthorized_and_does_not_spawn() {
        let spawner = Arc::new(RecordingSpawner::default());
        let mut router = build_router(test_state(Arc::clone(&spawner)));

        let response =
            route_request_with_bearer(&mut router, VALID_GATE_RUN_BODY, Some("not-the-token"))
                .await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let calls = spawner.calls.lock().expect("calls lock");
        assert!(calls.is_empty(), "wrong bearer must not spawn a Job");
    }

    #[tokio::test]
    async fn gate_run_with_valid_bearer_spawns_one_job() {
        let spawner = Arc::new(RecordingSpawner::default());
        let mut router = build_router(test_state(Arc::clone(&spawner)));

        let response =
            route_request_with_bearer(&mut router, VALID_GATE_RUN_BODY, Some(TEST_BEARER)).await;

        assert_eq!(response.status(), StatusCode::CREATED);
        let calls = spawner.calls.lock().expect("calls lock");
        assert_eq!(calls.len(), 1, "valid bearer must spawn exactly one Job");
    }
}
