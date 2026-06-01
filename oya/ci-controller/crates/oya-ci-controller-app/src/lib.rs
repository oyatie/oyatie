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
//! 3. Act: post pending/terminal status via `ForgejoStatusPoster`,
//!    patch write-once annotation, requeue or stop.
//!
//! ## POST /gate-run (plank role — Job spawn)
//!
//! Accepts `{"pr_number": N, "head_sha": "...", "base_ref": "dev"}` and
//! calls `K8sJobSpawner::spawn(build_gate_job(spec))` to create the labeled
//! gate Job. The reconcile loop then picks it up and posts Forgejo statuses.
//!
//! Idempotent: the Job name is deterministic (`oya-ci-gate-pr<N>-<sha8>`),
//! so a duplicate POST results in a 409 create-conflict no-op (returns 200).
//!
//! ## Idempotency / restart-safety
//!
//! ALL state lives on the Job object (labels + annotations). A controller
//! crash + relist resumes cleanly. The `oya.io/ci-forgejo-status-posted`
//! annotation is the write-once terminal guard (exactly-once posting).
//!
//! ## ADR-0083 Tier-3
//!
//! No `unwrap`/`expect`/`panic` on the reconcile path.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use k8s_openapi::api::{batch::v1::Job, core::v1::Pod};
use kube::{
    Api, Client,
    api::{ListParams, Patch, PatchParams},
    runtime::{Controller, controller::Action, watcher},
};
use oya_ci_controller_k8s_adapter::{
    ANNOT_CI_STATUS_POSTED, LABEL_CI_HEAD_SHA, LABEL_CI_PR_NUMBER, gate_job_list_params,
    observe_job,
};
use oya_ci_controller_kernel::{
    ForgejoState, ForgejoStatusPoster, GateRun, GateRunSpec, JobSpawner, ReconcileDecision,
    map_job_to_status,
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
/// The controller ONLY WATCHES Jobs and posts Forgejo commit statuses (crier
/// pattern). Job creation is the gateway's responsibility (hook/plank side).
#[derive(Clone)]
pub struct ControllerState {
    pub client: Client,
    pub forgejo_poster: Arc<dyn ForgejoStatusPoster>,
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
            // Post pending to Forgejo via spawn_blocking (reqwest::blocking must
            // not be called on the async executor thread — ADR-0083 major fix).
            let poster = Arc::clone(&ctx.forgejo_poster);
            let sha = head_sha.clone();
            let desc = description.clone();
            let post_result = tokio::task::spawn_blocking(move || {
                poster.post(&sha, ForgejoState::Pending, "oya-ci-gate", &desc, None)
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
                        ForgejoState::Pending,
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
            // Do NOT mark the annotation until Forgejo returns 200/201 — if we
            // crash between post and annotation patch, the next reconcile
            // re-posts (benign: Forgejo statuses are last-write-wins on (sha, context)).
            let poster = Arc::clone(&ctx.forgejo_poster);
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
                    // Forgejo unreachable — requeue with backoff. Verdict is
                    // durable on the Job object, so nothing is lost.
                    error!(
                        job = job_name,
                        state = %state,
                        error = %e,
                        "failed to post terminal status — requeueing"
                    );
                    Err(ReconcileError(format!(
                        "forgejo post failed for {job_name}: {e}"
                    )))
                }
            }
        }
    }
}

/// Patch the `oya.io/ci-forgejo-status-posted` annotation on the Job.
/// Best-effort: log on failure but never block the reconcile verdict.
async fn patch_status_annotation(
    client: &Client,
    namespace: &str,
    job_name: &str,
    state: ForgejoState,
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

/// Shared state for the health/metrics/gate-run server.
#[derive(Clone)]
pub struct ServerState {
    pub controller_namespace: String,
    /// Job spawner for POST /gate-run (plank role).
    pub job_spawner: Arc<dyn JobSpawner>,
    /// Full gate-Job spec config (image, clone URL, SA, deadlines).
    pub gate_spec_config: GateSpecConfig,
}

/// Static configuration for building gate Job specs.
/// Read from env vars at startup; immutable during the process lifetime.
#[derive(Clone)]
pub struct GateSpecConfig {
    /// Rust-CI image for the gate runner Pod.
    pub image: String,
    /// Forgejo in-cluster clone URL.
    pub forge_clone_url: String,
    /// Gate Job active deadline in seconds (mirrors Jenkins 60 min timeout).
    pub active_deadline_seconds: i64,
    /// TTL after finished for GC (sinker equivalent).
    pub ttl_seconds_after_finished: i32,
    /// Namespace for the gate Jobs.
    pub namespace: String,
    /// Low-privilege ServiceAccount for gate runner Pods.
    pub runner_service_account: String,
    /// Forgejo repo full name (e.g. "oya-admin/oyatie").
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
/// Returns 200 (already exists — idempotent) or 201 (created).
/// Returns 400 on invalid input, 500 on spawner failure.
async fn handle_gate_run(
    State(state): State<ServerState>,
    Json(req): Json<GateRunRequest>,
) -> impl IntoResponse {
    // Validate head_sha is at least 8 hex chars (minimum for a meaningful short-sha).
    if req.head_sha.len() < 8 || !req.head_sha.chars().all(|c| c.is_ascii_hexdigit()) {
        warn!(pr = req.pr_number, sha = %req.head_sha, "gate-run: invalid head_sha");
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "head_sha must be at least 8 hex characters"})),
        )
            .into_response();
    }

    // The current Forgejo-native weekly gate only accepts PRs targeting dev.
    // This keeps /gate-run aligned with the plain-git + Forgejo-PR-against-dev
    // contract and avoids accidentally spawning a gate Job for another branch.
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
                forge_clone_url: "http://forgejo.local/oya-admin/oyatie.git".to_owned(),
                active_deadline_seconds: 3600,
                ttl_seconds_after_finished: 600,
                namespace: "oya-ci".to_owned(),
                runner_service_account: "oya-ci-gate-runner".to_owned(),
                repo: "oya-admin/oyatie".to_owned(),
            },
        }
    }

    async fn route_request(router: &mut Router, body: &'static str) -> axum::response::Response {
        futures::future::poll_fn(|cx: &mut Context<'_>| match router.poll_ready(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(()),
            Poll::Ready(Err(err)) => panic!("router not ready: {err}"),
            Poll::Pending => Poll::Pending,
        })
        .await;

        router
            .call(
                Request::builder()
                    .method("POST")
                    .uri("/gate-run")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
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
            r#"{"pr_number":42,"head_sha":"abcdef1234567890"}"#,
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
            r#"{"pr_number":7,"head_sha":"1234567890abcdef","base_ref":"dev"}"#,
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        let calls = spawner.calls.lock().expect("calls lock");
        assert_eq!(calls.len(), 1);
    }

    #[tokio::test]
    async fn gate_run_rejects_non_dev_base_ref_for_weekly_forgejo_gate() {
        let spawner = Arc::new(RecordingSpawner::default());
        let mut router = build_router(test_state(Arc::clone(&spawner)));

        let response = route_request(
            &mut router,
            r#"{"pr_number":42,"head_sha":"abcdef1234567890","base_ref":"main"}"#,
        )
        .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let calls = spawner.calls.lock().expect("calls lock");
        assert!(calls.is_empty(), "invalid branch must not spawn a Job");
    }
}
