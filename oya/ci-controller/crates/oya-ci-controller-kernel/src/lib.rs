//! # oya-ci-controller-kernel
//!
//! Pure-domain kernel for the oya-ci controller (Phase 1, bespoke-Prow ADR).
//! No I/O, no async, no kube, no tokio. #![forbid(unsafe_code)].
//!
//! Owns:
//! - [`GateRun`] value object (identity + labels for the K8s Job)
//! - [`GateOutcome`] enum
//! - [`ForgejoState`] enum (Forgejo commit-status vocabulary)
//! - [`JobObservation`] — the K8s-Job-observation input type
//! - [`map_job_to_status`] — the TOTAL pure function: observation → [`ReconcileDecision`]
//! - [`ForgejoStatusPoster`] + [`JobSpawner`] trait seams (I/O boundary)
//!
//! ## Security
//!
//! - ADR-0083 Tier-3: no `unwrap`/`expect`/`panic` on the hot path.
//! - `#![forbid(unsafe_code)]`

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// All kernel-level errors. HTTP / kube mapping lives in adapter layers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelError {
    /// A downstream component (Forgejo, kube API) returned a transport failure.
    DownstreamTransport(String),
    /// A required field was missing or malformed.
    InvalidInput(String),
}

impl std::fmt::Display for KernelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KernelError::DownstreamTransport(why) => {
                write!(f, "downstream transport failure: {why}")
            }
            KernelError::InvalidInput(why) => {
                write!(f, "invalid input: {why}")
            }
        }
    }
}

impl std::error::Error for KernelError {}

pub type Result<T> = std::result::Result<T, KernelError>;

// ---------------------------------------------------------------------------
// Forgejo commit-status vocabulary
// ---------------------------------------------------------------------------

/// Forgejo commit-status state values (subset used by oya-ci-gate).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ForgejoState {
    Pending,
    Success,
    Failure,
    Error,
}

impl ForgejoState {
    pub const fn as_str(self) -> &'static str {
        match self {
            ForgejoState::Pending => "pending",
            ForgejoState::Success => "success",
            ForgejoState::Failure => "failure",
            ForgejoState::Error => "error",
        }
    }
}

impl std::fmt::Display for ForgejoState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// GateRun — identity value object
// ---------------------------------------------------------------------------

/// Immutable identity of a gate run. Lives in Job labels (trusted at creation).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GateRun {
    /// PR number (e.g. `42`).
    pub pr_number: u64, // data_class: INTERNAL_ONLY
    /// Full HEAD commit SHA (40 hex chars).
    pub head_sha: String, // data_class: INTERNAL_ONLY
    /// Forgejo delivery ID — idempotency dedup key (mirrors gateway DeliveryKey).
    pub delivery_id: String, // data_class: INTERNAL_ONLY
    /// Base branch (usually `"dev"`).
    pub base_ref: String, // data_class: INTERNAL_ONLY
    /// Repository full name, e.g. `"oya-admin/oyatie"`.
    pub repo: String, // data_class: INTERNAL_ONLY
}

impl GateRun {
    /// Deterministic K8s Job name: `oya-ci-gate-pr<N>-<sha[..8]>`.
    /// Deterministic = idempotent create-conflict dedup on re-delivery.
    pub fn job_name(&self) -> String {
        let sha_short = &self.head_sha[..self.head_sha.len().min(8)];
        format!("oya-ci-gate-pr{}-{}", self.pr_number, sha_short)
    }
}

// ---------------------------------------------------------------------------
// GateRunSpec — full specification for spawning a gate Job
// ---------------------------------------------------------------------------

/// Everything the k8s-adapter needs to build the gate Job spec.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GateRunSpec {
    pub run: GateRun, // data_class: INTERNAL_ONLY
    /// Rust-CI image (e.g. `"registry.oya-registry.svc.cluster.local:5000/rust-ci:dev"`).
    pub image: String, // data_class: INTERNAL_ONLY
    /// Forgejo clone URL (e.g. `"http://forgejo.oya-forge.svc.cluster.local:3000/oya-admin/oyatie.git"`).
    pub forge_clone_url: String, // data_class: INTERNAL_ONLY
    /// Gate deadline in seconds (mirrors Jenkinsfile 60 min timeout).
    pub active_deadline_seconds: i64, // data_class: INTERNAL_ONLY
    /// TTL after finished for GC (sinker equivalent).
    pub ttl_seconds_after_finished: i32, // data_class: INTERNAL_ONLY
    /// Namespace to spawn the Job in.
    pub namespace: String, // data_class: INTERNAL_ONLY
    /// ServiceAccount for the gate runner Pod (low-privilege, no kube API access).
    pub runner_service_account: String, // data_class: INTERNAL_ONLY
}

// ---------------------------------------------------------------------------
// JobHandle — returned by JobSpawner::spawn
// ---------------------------------------------------------------------------

/// Handle to a spawned (or pre-existing) Job.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobHandle {
    pub job_name: String,    // data_class: INTERNAL_ONLY
    pub namespace: String,   // data_class: INTERNAL_ONLY
    pub already_exists: bool, // true if a Job with this name already existed (idempotent)
}

// ---------------------------------------------------------------------------
// GateOutcome
// ---------------------------------------------------------------------------

/// High-level outcome of a gate run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GateOutcome {
    /// Gate passed: `buck2 affected-gate.sh` exited 0.
    Passed,
    /// Gate failed: non-zero exit (BackoffLimitExceeded, deadline, eviction, OOM, …).
    Failed,
}

// ---------------------------------------------------------------------------
// JobObservation — K8s-Job observation input type
// ---------------------------------------------------------------------------

/// Pod-level reasons the controller inspects (container state.waiting / terminated).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PodReason {
    // Waiting reasons
    ImagePullBackOff,
    ErrImagePull,
    InvalidImageName,
    CreateContainerError,
    CreateContainerConfigError,
    RunContainerError,
    CrashLoopBackOff,
    // Terminated reasons
    OOMKilled,
    // Pod-level (status.reason)
    Evicted,
    /// Any other reason — carry the raw string for diagnostics.
    Other(String),
}

impl PodReason {
    /// Parse from a raw K8s reason string.
    pub fn from_str(s: &str) -> Self {
        match s {
            "ImagePullBackOff" => PodReason::ImagePullBackOff,
            "ErrImagePull" => PodReason::ErrImagePull,
            "InvalidImageName" => PodReason::InvalidImageName,
            "CreateContainerError" => PodReason::CreateContainerError,
            "CreateContainerConfigError" => PodReason::CreateContainerConfigError,
            "RunContainerError" => PodReason::RunContainerError,
            "CrashLoopBackOff" => PodReason::CrashLoopBackOff,
            "OOMKilled" => PodReason::OOMKilled,
            "Evicted" => PodReason::Evicted,
            other => PodReason::Other(other.to_owned()),
        }
    }

    /// True if this is a "waiting" reason that may resolve (transient).
    /// Note: `InvalidImageName` is NOT transient — it requires operator intervention;
    /// it is handled as terminal-immediately in `map_job_to_status`.
    pub fn is_pull_or_container_error(&self) -> bool {
        matches!(
            self,
            PodReason::ImagePullBackOff
                | PodReason::ErrImagePull
                | PodReason::CreateContainerError
                | PodReason::CreateContainerConfigError
                | PodReason::RunContainerError
                | PodReason::CrashLoopBackOff
        )
    }
}

/// Job condition type extracted from `status.conditions[].type`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JobConditionType {
    /// `type: Complete` (succeeded).
    Complete,
    /// `type: Failed`.
    Failed,
}

/// A single Job condition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobCondition {
    pub condition_type: JobConditionType,
    /// K8s condition reason (e.g. `"BackoffLimitExceeded"`, `"DeadlineExceeded"`).
    pub reason: Option<String>,
    pub status: bool, // true = "True"
}

/// Snapshot of the live K8s Job + its owned Pods projected for the kernel.
///
/// The k8s-adapter builds this from the live Job object; the kernel is
/// pure-functional over it.
#[derive(Clone, Debug)]
pub struct JobObservation {
    /// `status.active` count (>= 1 means running).
    pub active: i32,
    /// `status.succeeded` count.
    pub succeeded: i32,
    /// `status.failed` count.
    pub failed: i32,
    /// Job conditions (may be empty while running).
    pub conditions: Vec<JobCondition>,
    /// Pod reasons observed across all owned Pods (waiting.reason / terminated.reason / pod.status.reason).
    pub pod_reasons: Vec<PodReason>,
    /// Number of reconcile cycles this observation has been in a "waiting"
    /// pod reason (ImagePullBackOff etc.) without transitioning.
    pub waiting_cycles: u32,
    /// Whether the Job object itself was NotFound (deleted/GC'd).
    pub job_not_found: bool,
    /// Whether a terminal Forgejo status was already posted
    /// (from annotation `oya.io/ci-forgejo-status-posted`).
    pub terminal_status_already_posted: Option<ForgejoState>,
    /// Whether the pending status was already posted
    /// (from annotation `oya.io/ci-forgejo-status-posted` == "pending").
    pub pending_status_already_posted: bool,
}

// ---------------------------------------------------------------------------
// ReconcileDecision — output of the pure state machine
// ---------------------------------------------------------------------------

/// The controller's decided action for this reconcile cycle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReconcileDecision {
    /// Job is active / pending — post pending status if not yet posted, requeue.
    PostPending {
        description: String,
    },
    /// Already posted pending; just requeue to watch for progress.
    AwaitChange,
    /// Job reached a terminal state — post this Forgejo status.
    PostTerminal {
        state: ForgejoState,
        context: &'static str,
        description: String,
    },
    /// Terminal status was already posted — nothing to do.
    AlreadyTerminal,
}

// The context label for the oya-ci-gate status (matches Jenkinsfile + branch protection rule).
pub const GATE_CONTEXT: &str = "oya-ci-gate";

/// The TOTAL pure function: K8s Job observation → reconcile decision.
///
/// This is the core of the controller. All state-machine logic lives here;
/// no I/O, no side-effects.
///
/// # Grace threshold
///
/// `grace_cycles` is the number of reconcile cycles a "waiting" pod reason
/// (ImagePullBackOff, CreateContainerError, etc.) is tolerated before the
/// controller declares it terminal. The caller supplies this from config
/// (default: 12 cycles at ~10s requeue ≈ 2 min).
pub fn map_job_to_status(obs: &JobObservation, grace_cycles: u32) -> ReconcileDecision {
    // If the Job was GC'd before a terminal status was posted, fail closed.
    if obs.job_not_found {
        if obs.terminal_status_already_posted.is_some() {
            return ReconcileDecision::AlreadyTerminal;
        }
        return ReconcileDecision::PostTerminal {
            state: ForgejoState::Failure,
            context: GATE_CONTEXT,
            description: "oya-ci-gate: run disappeared (job deleted before verdict posted)"
                .to_owned(),
        };
    }

    // If a terminal status is already posted, nothing to do.
    if let Some(posted) = obs.terminal_status_already_posted {
        if posted != ForgejoState::Pending {
            return ReconcileDecision::AlreadyTerminal;
        }
    }

    // ---- Job conditions (terminal) ----------------------------------------

    // Complete condition (succeeded >= 1 or condition Complete=True)
    let is_complete = obs.succeeded >= 1
        || obs.conditions.iter().any(|c| {
            c.status && matches!(c.condition_type, JobConditionType::Complete)
        });

    if is_complete {
        return ReconcileDecision::PostTerminal {
            state: ForgejoState::Success,
            context: GATE_CONTEXT,
            description: "buck2 affected gate passed".to_owned(),
        };
    }

    // Failed condition
    let failed_condition = obs.conditions.iter().find(|c| {
        c.status && matches!(c.condition_type, JobConditionType::Failed)
    });

    if let Some(cond) = failed_condition {
        let reason = cond.reason.as_deref().unwrap_or("");
        let description = match reason {
            "DeadlineExceeded" => {
                "oya-ci-gate failed: deadline exceeded (timeout)".to_owned()
            }
            _ => {
                // BackoffLimitExceeded or unknown — gate logic failure
                "oya-ci-gate failed: buck2 affected gate exited non-zero".to_owned()
            }
        };
        return ReconcileDecision::PostTerminal {
            state: ForgejoState::Failure,
            context: GATE_CONTEXT,
            description,
        };
    }

    // ---- Pod-level terminal reasons ----------------------------------------

    for reason in &obs.pod_reasons {
        match reason {
            PodReason::OOMKilled => {
                return ReconcileDecision::PostTerminal {
                    state: ForgejoState::Failure,
                    context: GATE_CONTEXT,
                    description: "oya-ci-gate failed: OOMKilled — raise Job memory limit"
                        .to_owned(),
                };
            }
            PodReason::Evicted => {
                return ReconcileDecision::PostTerminal {
                    state: ForgejoState::Failure,
                    context: GATE_CONTEXT,
                    description: "oya-ci-gate failed: pod evicted (node-pressure or preemption)"
                        .to_owned(),
                };
            }
            // InvalidImageName is a misconfiguration — not transient, terminal immediately.
            PodReason::InvalidImageName => {
                return ReconcileDecision::PostTerminal {
                    state: ForgejoState::Error,
                    context: GATE_CONTEXT,
                    description: "oya-ci-gate error: InvalidImageName — operator must fix gate image config".to_owned(),
                };
            }
            r if r.is_pull_or_container_error() => {
                // Apply bounded grace: past grace_cycles, declare terminal.
                if obs.waiting_cycles >= grace_cycles {
                    let label = match r {
                        PodReason::ImagePullBackOff | PodReason::ErrImagePull => {
                            "image pull failed (rust-ci:dev unavailable)"
                        }
                        _ => "container setup failed (CreateContainerError or CrashLoopBackOff)",
                    };
                    return ReconcileDecision::PostTerminal {
                        state: ForgejoState::Failure,
                        context: GATE_CONTEXT,
                        description: format!("oya-ci-gate failed: {label}"),
                    };
                }
                // Within grace — fall through to pending / await-change below
            }
            _ => {}
        }
    }

    // ---- Active / pending (non-terminal) -----------------------------------

    if obs.active >= 1 || obs.failed == 0 {
        if obs.pending_status_already_posted {
            return ReconcileDecision::AwaitChange;
        }
        return ReconcileDecision::PostPending {
            description: "oya-ci-gate: running buck2 affected gate".to_owned(),
        };
    }

    // Fallback: failed count > 0 but no condition yet — treat as gate failure.
    ReconcileDecision::PostTerminal {
        state: ForgejoState::Failure,
        context: GATE_CONTEXT,
        description: "oya-ci-gate failed: buck2 affected gate exited non-zero".to_owned(),
    }
}

// ---------------------------------------------------------------------------
// GateRunToken — shared-secret auth for POST /gate-run
// ---------------------------------------------------------------------------

/// Opaque shared-secret token checked on POST /gate-run.
///
/// Callers (the ci-webhook-gateway / ControllerDispatcher) must present this
/// token in the `X-Gate-Run-Token` HTTP header. Anonymous callers are rejected
/// with HTTP 401. The token arrives in the controller via the projected
/// `gate-run-token` Kubernetes Secret (ESO-synced from OpenBao
/// `secret/oya/ci/gate-run-token`).
///
/// Comparison is constant-time (XOR fold) to resist timing side-channels.
#[derive(Clone)]
pub struct GateRunToken {
    token: Vec<u8>, // data_class: INTERNAL_ONLY — never log, never serialize
}

impl GateRunToken {
    /// Construct from raw bytes (decoded from the env var / secret).
    pub fn new(token: Vec<u8>) -> Self {
        Self { token }
    }

    /// Constant-time equality check. Returns `true` iff the supplied value
    /// matches the stored token byte-for-byte.
    ///
    /// Both length inequality AND value inequality are handled without early
    /// exit: the fold runs over `max(self, other)` length using `get`-or-zero
    /// to keep the branch count constant.
    pub fn verify(&self, candidate: &[u8]) -> bool {
        let n = self.token.len().max(candidate.len());
        // XOR-fold: accumulate all differing bits.
        let diff = (0..n).fold(0u8, |acc, i| {
            let a = self.token.get(i).copied().unwrap_or(0);
            let b = candidate.get(i).copied().unwrap_or(0);
            acc | (a ^ b)
        });
        // Length must also match to be valid.
        let len_diff = (self.token.len() ^ candidate.len()) as u8;
        (diff | len_diff) == 0
    }
}

/// Redact the token in `Debug` output — it must never appear in logs.
impl std::fmt::Debug for GateRunToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GateRunToken")
            .field("token", &"[REDACTED]")
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Trait seams (I/O boundary — implemented by adapter crates)
// ---------------------------------------------------------------------------

/// Seam for posting Forgejo commit-status updates.
pub trait ForgejoStatusPoster: Send + Sync {
    /// POST a status to `POST /api/v1/repos/<owner>/<repo>/statuses/<sha>`.
    /// Returns `Err(KernelError::DownstreamTransport)` on non-2xx or transport error.
    fn post(
        &self,
        sha: &str,
        state: ForgejoState,
        context: &str,
        description: &str,
        target_url: Option<&str>,
    ) -> Result<()>;
}

/// Seam for spawning K8s gate Jobs.
pub trait JobSpawner: Send + Sync {
    /// Create (or idempotently find) the K8s Job for a gate run.
    fn spawn(&self, spec: &GateRunSpec) -> Result<JobHandle>;
}

// ---------------------------------------------------------------------------
// Tests — full phase/status matrix
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn base_obs() -> JobObservation {
        JobObservation {
            active: 0,
            succeeded: 0,
            failed: 0,
            conditions: vec![],
            pod_reasons: vec![],
            waiting_cycles: 0,
            job_not_found: false,
            terminal_status_already_posted: None,
            pending_status_already_posted: false,
        }
    }

    // ---- Active / pending --------------------------------------------------

    #[test]
    fn active_job_no_prior_post_returns_post_pending() {
        let obs = JobObservation {
            active: 1,
            ..base_obs()
        };
        let dec = map_job_to_status(&obs, 12);
        assert!(
            matches!(dec, ReconcileDecision::PostPending { .. }),
            "expected PostPending, got {dec:?}"
        );
    }

    #[test]
    fn active_job_pending_already_posted_returns_await_change() {
        let obs = JobObservation {
            active: 1,
            pending_status_already_posted: true,
            ..base_obs()
        };
        assert_eq!(map_job_to_status(&obs, 12), ReconcileDecision::AwaitChange);
    }

    // ---- Success -----------------------------------------------------------

    #[test]
    fn succeeded_job_returns_success() {
        let obs = JobObservation {
            succeeded: 1,
            ..base_obs()
        };
        match map_job_to_status(&obs, 12) {
            ReconcileDecision::PostTerminal { state, .. } => {
                assert_eq!(state, ForgejoState::Success);
            }
            other => panic!("expected PostTerminal(Success), got {other:?}"),
        }
    }

    #[test]
    fn complete_condition_returns_success() {
        let obs = JobObservation {
            conditions: vec![JobCondition {
                condition_type: JobConditionType::Complete,
                reason: None,
                status: true,
            }],
            ..base_obs()
        };
        match map_job_to_status(&obs, 12) {
            ReconcileDecision::PostTerminal { state, .. } => {
                assert_eq!(state, ForgejoState::Success);
            }
            other => panic!("expected success, got {other:?}"),
        }
    }

    // ---- Failure (BackoffLimitExceeded) ------------------------------------

    #[test]
    fn failed_condition_backoff_limit_returns_failure() {
        let obs = JobObservation {
            failed: 1,
            conditions: vec![JobCondition {
                condition_type: JobConditionType::Failed,
                reason: Some("BackoffLimitExceeded".to_owned()),
                status: true,
            }],
            ..base_obs()
        };
        match map_job_to_status(&obs, 12) {
            ReconcileDecision::PostTerminal { state, description, .. } => {
                assert_eq!(state, ForgejoState::Failure);
                assert!(description.contains("non-zero"), "desc: {description}");
            }
            other => panic!("expected failure, got {other:?}"),
        }
    }

    // ---- DeadlineExceeded --------------------------------------------------

    #[test]
    fn deadline_exceeded_returns_failure_with_timeout_message() {
        let obs = JobObservation {
            failed: 1,
            conditions: vec![JobCondition {
                condition_type: JobConditionType::Failed,
                reason: Some("DeadlineExceeded".to_owned()),
                status: true,
            }],
            ..base_obs()
        };
        match map_job_to_status(&obs, 12) {
            ReconcileDecision::PostTerminal { state, description, .. } => {
                assert_eq!(state, ForgejoState::Failure);
                assert!(description.contains("deadline exceeded"), "desc: {description}");
            }
            other => panic!("expected failure, got {other:?}"),
        }
    }

    // ---- OOMKilled ---------------------------------------------------------

    #[test]
    fn oom_killed_pod_returns_failure_with_oom_message() {
        let obs = JobObservation {
            active: 1,
            pod_reasons: vec![PodReason::OOMKilled],
            ..base_obs()
        };
        match map_job_to_status(&obs, 12) {
            ReconcileDecision::PostTerminal { state, description, .. } => {
                assert_eq!(state, ForgejoState::Failure);
                assert!(description.contains("OOMKilled"), "desc: {description}");
            }
            other => panic!("expected failure, got {other:?}"),
        }
    }

    // ---- Evicted -----------------------------------------------------------

    #[test]
    fn evicted_pod_returns_failure_with_evicted_message() {
        let obs = JobObservation {
            pod_reasons: vec![PodReason::Evicted],
            ..base_obs()
        };
        match map_job_to_status(&obs, 12) {
            ReconcileDecision::PostTerminal { state, description, .. } => {
                assert_eq!(state, ForgejoState::Failure);
                assert!(description.contains("evicted"), "desc: {description}");
            }
            other => panic!("expected failure, got {other:?}"),
        }
    }

    // ---- InvalidImageName (terminal immediately) ---------------------------

    #[test]
    fn invalid_image_name_returns_error_immediately() {
        // InvalidImageName is a misconfiguration, not transient — terminal at cycle 0.
        let obs = JobObservation {
            active: 1,
            pod_reasons: vec![PodReason::InvalidImageName],
            waiting_cycles: 0,
            ..base_obs()
        };
        match map_job_to_status(&obs, 12) {
            ReconcileDecision::PostTerminal { state, description, .. } => {
                assert_eq!(state, ForgejoState::Error);
                assert!(description.contains("InvalidImageName"), "desc: {description}");
            }
            other => panic!("expected terminal error immediately, got {other:?}"),
        }
    }

    // ---- ImagePullBackOff within grace -------------------------------------

    #[test]
    fn image_pull_backoff_within_grace_returns_pending() {
        let obs = JobObservation {
            active: 1,
            pod_reasons: vec![PodReason::ImagePullBackOff],
            waiting_cycles: 5,
            ..base_obs()
        };
        // Grace = 12: within grace, should still show pending
        match map_job_to_status(&obs, 12) {
            ReconcileDecision::PostPending { .. } | ReconcileDecision::AwaitChange => {}
            other => panic!("expected pending/await, got {other:?}"),
        }
    }

    // ---- ImagePullBackOff past grace ---------------------------------------

    #[test]
    fn image_pull_backoff_past_grace_returns_failure() {
        let obs = JobObservation {
            active: 1,
            pod_reasons: vec![PodReason::ImagePullBackOff],
            waiting_cycles: 15,
            ..base_obs()
        };
        match map_job_to_status(&obs, 12) {
            ReconcileDecision::PostTerminal { state, description, .. } => {
                assert_eq!(state, ForgejoState::Failure);
                assert!(description.contains("image pull"), "desc: {description}");
            }
            other => panic!("expected failure past grace, got {other:?}"),
        }
    }

    // ---- CreateContainerError past grace -----------------------------------

    #[test]
    fn create_container_error_past_grace_returns_failure() {
        let obs = JobObservation {
            active: 1,
            pod_reasons: vec![PodReason::CreateContainerError],
            waiting_cycles: 13,
            ..base_obs()
        };
        match map_job_to_status(&obs, 12) {
            ReconcileDecision::PostTerminal { state, .. } => {
                assert_eq!(state, ForgejoState::Failure);
            }
            other => panic!("expected failure, got {other:?}"),
        }
    }

    // ---- Job NotFound (GC race) --------------------------------------------

    #[test]
    fn job_not_found_no_prior_terminal_returns_failure() {
        let obs = JobObservation {
            job_not_found: true,
            ..base_obs()
        };
        match map_job_to_status(&obs, 12) {
            ReconcileDecision::PostTerminal { state, description, .. } => {
                assert_eq!(state, ForgejoState::Failure);
                assert!(description.contains("disappeared"), "desc: {description}");
            }
            other => panic!("expected failure, got {other:?}"),
        }
    }

    #[test]
    fn job_not_found_with_prior_terminal_returns_already_terminal() {
        let obs = JobObservation {
            job_not_found: true,
            terminal_status_already_posted: Some(ForgejoState::Success),
            ..base_obs()
        };
        assert_eq!(map_job_to_status(&obs, 12), ReconcileDecision::AlreadyTerminal);
    }

    // ---- Already terminal idempotency guard --------------------------------

    #[test]
    fn terminal_already_posted_returns_already_terminal() {
        let obs = JobObservation {
            terminal_status_already_posted: Some(ForgejoState::Success),
            ..base_obs()
        };
        assert_eq!(map_job_to_status(&obs, 12), ReconcileDecision::AlreadyTerminal);
    }

    #[test]
    fn terminal_failure_already_posted_returns_already_terminal() {
        let obs = JobObservation {
            failed: 1,
            terminal_status_already_posted: Some(ForgejoState::Failure),
            conditions: vec![JobCondition {
                condition_type: JobConditionType::Failed,
                reason: Some("BackoffLimitExceeded".to_owned()),
                status: true,
            }],
            ..base_obs()
        };
        assert_eq!(map_job_to_status(&obs, 12), ReconcileDecision::AlreadyTerminal);
    }

    // ---- Forgejo state vocabulary -----------------------------------------

    #[test]
    fn forgejo_state_as_str_matches_api() {
        assert_eq!(ForgejoState::Pending.as_str(), "pending");
        assert_eq!(ForgejoState::Success.as_str(), "success");
        assert_eq!(ForgejoState::Failure.as_str(), "failure");
        assert_eq!(ForgejoState::Error.as_str(), "error");
    }

    // ---- GateRunToken constant-time verify ---------------------------------

    #[test]
    fn gate_run_token_correct_value_returns_true() {
        let tok = GateRunToken::new(b"super-secret-abc".to_vec());
        assert!(tok.verify(b"super-secret-abc"));
    }

    #[test]
    fn gate_run_token_wrong_value_returns_false() {
        let tok = GateRunToken::new(b"super-secret-abc".to_vec());
        assert!(!tok.verify(b"wrong-value-here"));
    }

    #[test]
    fn gate_run_token_empty_candidate_returns_false() {
        let tok = GateRunToken::new(b"secret".to_vec());
        assert!(!tok.verify(b""));
    }

    #[test]
    fn gate_run_token_prefix_match_returns_false() {
        let tok = GateRunToken::new(b"secretXYZ".to_vec());
        assert!(!tok.verify(b"secret"));
    }

    #[test]
    fn gate_run_token_suffix_match_returns_false() {
        let tok = GateRunToken::new(b"secret".to_vec());
        assert!(!tok.verify(b"secretXYZ"));
    }

    #[test]
    fn gate_run_token_off_by_one_bit_returns_false() {
        // Flip the last bit of the token value.
        let tok = GateRunToken::new(vec![0b10101010u8; 16]);
        let mut candidate = vec![0b10101010u8; 16];
        *candidate.last_mut().unwrap() ^= 0x01;
        assert!(!tok.verify(&candidate));
    }

    #[test]
    fn gate_run_token_debug_redacts_value() {
        let tok = GateRunToken::new(b"do-not-log-me".to_vec());
        let s = format!("{tok:?}");
        assert!(s.contains("[REDACTED]"), "debug should redact: {s}");
        assert!(!s.contains("do-not-log-me"), "debug must not leak token: {s}");
    }

    // ---- GateRun job_name --------------------------------------------------

    #[test]
    fn gate_run_job_name_is_deterministic() {
        let run = GateRun {
            pr_number: 42,
            head_sha: "abcdef1234567890".to_owned(),
            delivery_id: "d1".to_owned(),
            base_ref: "dev".to_owned(),
            repo: "oya-admin/oyatie".to_owned(),
        };
        assert_eq!(run.job_name(), "oya-ci-gate-pr42-abcdef12");
    }
}
