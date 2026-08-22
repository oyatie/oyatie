//! # ci-webhook-gateway-kernel
//!
//! Pure-Rust kernel for the CI webhook gateway (ADR-0387).
//! No I/O, no HTTP, no async. Adapter crates provide all runtime dependencies.
//!
//! ## Deliverable coverage
//!
//! - D1: [`CiTriggerEvent`] + GitHub payload parsing types
//! - D2: [`WebhookSignature`] + [`SignatureVerifier`] trait seam
//! - D3: payload normalisation → [`CiTriggerEvent`]
//! - D4: [`JenkinsJob`] + [`JenkinsClient`] trait seam
//! - D5: [`CommitStatusContext`] + [`CommitStatusPoster`] + [`GitHubStatusRequest`]
//! - D6: [`WebhookAuthzGate`] trait seam + [`WebhookAuthzRequest`]
//!
//! ## Security invariants (ADR-0112 / ADR-0083)
//!
//! - ADR-0083 Tier-3: no `unwrap`/`expect`/`panic` on the request path.
//! - ed25519 signature is verified on the RAW body BEFORE any JSON parsing.
//! - Signature secret is never logged (`Debug` is redacted on [`WebhookSignature`]).
//! - Dogfood doctrine: `oyatie-dogfood` tenant goes through the same
//!   [`WebhookAuthzGate`] path as every external tenant — no internal bypass.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use serde::Deserialize;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// All kernel-level errors.  HTTP mapping lives in the adapter layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelError {
    /// The ed25519 signature header was absent. Fail-closed.
    MissingSignature,
    /// The signature header was present but malformed.
    MalformedSignature,
    /// The ed25519 signature did not verify against the payload.
    SignatureMismatch,
    /// The timestamp in the signature is outside the acceptable window.
    ExpiredTimestamp,
    /// The webhook secret could not be loaded from OpenBao.
    SecretUnavailable,
    /// The JSON payload was malformed or missing required fields.
    MalformedPayload(String),
    /// The (event, action) pair is not in the closed router table.
    UnroutableEvent { event: String, action: String },
    /// The request was denied by the Cedar authorization policy.
    ForbiddenByPolicy,
    /// A downstream component (Jenkins, GitHub) returned a transport failure.
    DownstreamTransport(String),
}

impl std::fmt::Display for KernelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KernelError::MissingSignature => {
                f.write_str("webhook rejected: missing ed25519 signature header (fail-closed)")
            }
            KernelError::MalformedSignature => {
                f.write_str("webhook rejected: malformed signature header")
            }
            KernelError::SignatureMismatch => {
                f.write_str("webhook rejected: ed25519 signature mismatch (fail-closed)")
            }
            KernelError::ExpiredTimestamp => {
                f.write_str("webhook rejected: timestamp outside acceptable window")
            }
            KernelError::SecretUnavailable => {
                f.write_str("webhook secret unavailable — refusing to verify (fail-closed)")
            }
            KernelError::MalformedPayload(why) => {
                write!(f, "malformed webhook payload: {why}")
            }
            KernelError::UnroutableEvent { event, action } => {
                write!(
                    f,
                    "unroutable event: ({event}, {action}) not in router table"
                )
            }
            KernelError::ForbiddenByPolicy => {
                f.write_str("webhook trigger forbidden by Cedar policy")
            }
            KernelError::DownstreamTransport(why) => {
                write!(f, "downstream transport failure: {why}")
            }
        }
    }
}

impl std::error::Error for KernelError {}

pub type Result<T> = std::result::Result<T, KernelError>;

// ---------------------------------------------------------------------------
// D2 — WebhookSignature value object
// ---------------------------------------------------------------------------

/// Raw bytes of an ed25519 signature extracted from the webhook headers.
/// The secret key material is never stored here — this is the SIGNATURE
/// (computed by GitHub using its copy of the shared secret), not the key.
///
/// `Debug` is redacted so the bytes never appear in log output.
#[derive(Clone, PartialEq, Eq)]
pub struct WebhookSignature(Vec<u8>); // data_class: INTERNAL_ONLY

impl WebhookSignature {
    /// Construct from raw bytes (e.g. decoded from a base64/hex header).
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Self {
        Self(bytes.into())
    }

    /// Construct by decoding a lowercase-hex string.
    ///
    /// Returns `Err(KernelError::MalformedSignature)` on non-hex input.
    pub fn from_hex(hex: &str) -> Result<Self> {
        let hex = hex.trim();
        if hex.is_empty() || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(KernelError::MalformedSignature);
        }
        if !hex.len().is_multiple_of(2) {
            return Err(KernelError::MalformedSignature);
        }
        let bytes = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
            .collect::<std::result::Result<Vec<u8>, _>>()
            .map_err(|_| KernelError::MalformedSignature)?;
        Ok(Self(bytes))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::fmt::Debug for WebhookSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WebhookSignature(<redacted>)")
    }
}

// ---------------------------------------------------------------------------
// D2 — SignatureVerifier trait seam
// ---------------------------------------------------------------------------

/// D2 — ed25519 signature verification seam.
///
/// The Stage-5 adapter fetches the key from
/// `sref://openbao/oya/ci/github-ed25519-secret` and implements this trait.
/// Tests use [`MockSignatureVerifier`].
///
/// Implementations MUST:
/// 1. Verify on the RAW `body` bytes BEFORE any JSON parsing.
/// 2. Return [`KernelError::MissingSignature`] when no header is present.
/// 3. Return [`KernelError::ExpiredTimestamp`] when the timestamp window
///    exceeds the configured tolerance.
/// 4. Be constant-time on the comparison to avoid timing side-channels.
pub trait SignatureVerifier {
    fn verify(
        &self,
        body: &[u8],
        signature: &WebhookSignature,
        timestamp_unix_s: Option<u64>,
    ) -> Result<()>;
}

/// Test double — always returns the configured verdict.
pub struct MockSignatureVerifier {
    /// The verdict this mock returns for every `verify` call.
    pub verdict: Result<()>, // data_class: INTERNAL_ONLY
}

impl SignatureVerifier for MockSignatureVerifier {
    fn verify(
        &self,
        _body: &[u8],
        _signature: &WebhookSignature,
        _timestamp_unix_s: Option<u64>,
    ) -> Result<()> {
        self.verdict.clone()
    }
}

// ---------------------------------------------------------------------------
// D1 / D3 — GitHub webhook payload types + CiTriggerEvent
// ---------------------------------------------------------------------------

/// The closed set of GitHub `pull_request` actions the gateway acts on.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrAction {
    /// PR opened or reopened against the target branch.
    Opened,
    /// New commits pushed to the PR head (fix-at-any-stage, ADR-0111).
    Synchronized,
    /// PR closed (merged or dismissed) — used to cancel in-flight jobs.
    Closed,
}

/// Canonical normalised trigger event produced from any supported GitHub
/// webhook payload.  This is what the [`JenkinsClient`] consumes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CiTriggerEvent {
    /// GitHub/GitHub repo full name, e.g. `"oyatie/oyatie"`.
    pub repo: String, // data_class: INTERNAL_ONLY
    /// Target base branch, e.g. `"dev"`.
    pub branch: String, // data_class: INTERNAL_ONLY
    /// HEAD commit SHA of the PR branch.
    pub head_sha: String, // data_class: INTERNAL_ONLY
    /// Base commit SHA (merge base).
    pub base_sha: String, // data_class: INTERNAL_ONLY
    /// PR number (0 for non-PR events such as a direct push).
    pub pr_number: u64, // data_class: INTERNAL_ONLY
    /// Unique delivery ID from GitHub (used for idempotency dedup).
    pub delivery_id: String, // data_class: INTERNAL_ONLY
    /// The normalised action that produced this event.
    pub action: CiAction, // data_class: INTERNAL_ONLY
}

/// The action that produced a [`CiTriggerEvent`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CiAction {
    PrOpened,
    PrSynchronized,
    PrClosed,
    /// A `ping` handshake from GitHub on webhook registration.
    Ping,
}

// ---- raw serde shapes for GitHub payloads --------------------------------

#[derive(Deserialize)]
struct RawPrPayload {
    action: String,
    #[serde(default)]
    number: Option<u64>,
    pull_request: RawPr,
}

#[derive(Deserialize)]
struct RawPr {
    #[serde(default)]
    number: Option<u64>,
    base: RawRef,
    head: RawRef,
    #[serde(default)]
    draft: bool,
}

#[derive(Deserialize)]
struct RawRef {
    #[serde(rename = "ref")]
    ref_name: String,
    #[serde(default)]
    sha: String,
}

/// Outcome of routing a raw delivery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RouteOutcome {
    /// An event the pipeline should act on.
    Trigger(CiTriggerEvent),
    /// Authentic + parseable but deliberately ignored.
    Ignored { reason: String },
}

/// Route a raw GitHub delivery to a [`RouteOutcome`].
///
/// `event_type` is the `X-GitHub-Event` header value;
/// `body` is the already-signature-verified JSON bytes;
/// `delivery_id` is the `X-GitHub-Delivery` header value;
/// `target_branch` is the gated base branch (usually `"dev"`).
pub fn route_github_event(
    event_type: &str,
    body: &[u8],
    delivery_id: &str,
    target_branch: &str,
) -> Result<RouteOutcome> {
    match event_type {
        "pull_request" => route_pull_request(body, delivery_id, target_branch),
        "ping" => Ok(RouteOutcome::Ignored {
            reason: "ping handshake".to_owned(),
        }),
        other => Err(KernelError::UnroutableEvent {
            event: other.to_owned(),
            action: String::new(),
        }),
    }
}

fn route_pull_request(body: &[u8], delivery_id: &str, target_branch: &str) -> Result<RouteOutcome> {
    let raw: RawPrPayload = serde_json::from_slice(body)
        .map_err(|e| KernelError::MalformedPayload(format!("pull_request: {e}")))?;

    let action = match raw.action.as_str() {
        "opened" | "reopened" => PrAction::Opened,
        "synchronized" | "synchronize" => PrAction::Synchronized,
        "closed" => PrAction::Closed,
        other => {
            return Ok(RouteOutcome::Ignored {
                reason: format!("pull_request action {other:?} not gated"),
            });
        }
    };

    if raw.pull_request.base.ref_name != target_branch {
        return Ok(RouteOutcome::Ignored {
            reason: format!(
                "base ref {:?} != gated target {:?}",
                raw.pull_request.base.ref_name, target_branch
            ),
        });
    }

    if raw.pull_request.draft {
        return Ok(RouteOutcome::Ignored {
            reason: "PR is a draft".to_owned(),
        });
    }

    let pr_number =
        raw.pull_request.number.or(raw.number).ok_or_else(|| {
            KernelError::MalformedPayload("missing pull_request.number".to_owned())
        })?;

    if raw.pull_request.head.sha.trim().is_empty() {
        return Err(KernelError::MalformedPayload(
            "missing pull_request.head.sha".to_owned(),
        ));
    }

    let ci_action = match action {
        PrAction::Opened => CiAction::PrOpened,
        PrAction::Synchronized => CiAction::PrSynchronized,
        PrAction::Closed => CiAction::PrClosed,
    };

    Ok(RouteOutcome::Trigger(CiTriggerEvent {
        repo: String::new(), // populated by the adapter from repo context
        branch: raw.pull_request.base.ref_name,
        head_sha: raw.pull_request.head.sha,
        base_sha: raw.pull_request.base.sha,
        pr_number,
        delivery_id: delivery_id.to_owned(),
        action: ci_action,
    }))
}

// ---------------------------------------------------------------------------
// D4 — JenkinsJob value object + JenkinsClient trait seam
// ---------------------------------------------------------------------------

/// Status of a Jenkins build.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JobStatus {
    Queued,
    Running,
    /// Build completed successfully — all gates green.
    Success,
    /// Build completed with at least one gate failure.
    Failure,
    /// Build was aborted (e.g. a newer commit superseded it).
    Aborted,
    /// Build result is unknown (Jenkins unreachable, timeout, etc.).
    Unknown,
}

/// A Jenkins parameterized job instance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JenkinsJob {
    /// Job name in Jenkins, e.g. `"oyaCiLane"`.
    pub job_name: String, // data_class: INTERNAL_ONLY
    /// Build number assigned by Jenkins (0 = not yet assigned / queued).
    pub build_number: u64, // data_class: INTERNAL_ONLY
    /// The trigger event that produced this job.
    pub trigger: CiTriggerEvent, // data_class: INTERNAL_ONLY
    /// Current status.
    pub status: JobStatus, // data_class: INTERNAL_ONLY
    /// Jenkins build URL for human inspection.
    pub build_url: Option<String>, // data_class: INTERNAL_ONLY
}

/// D4 — Jenkins client trait seam.
///
/// Stage-5 adapter implements this with a `reqwest`-backed Jenkins REST client.
/// The blessed Jenkins REST endpoints are:
///   - Trigger: `POST /job/<name>/buildWithParameters`
///   - Poll:    `GET  /job/<name>/<build_number>/api/json`
pub trait JenkinsClient {
    /// Trigger a parameterized build of `job_name` for the given event.
    /// Returns a [`JenkinsJob`] in `Queued` state with the assigned build number.
    ///
    /// Returns `Err(KernelError::DownstreamTransport)` on Jenkins connectivity
    /// failure.  Never panics.
    fn trigger(&self, job_name: &str, event: &CiTriggerEvent) -> Result<JenkinsJob>;

    /// Poll the status of an in-flight build.
    fn poll_status(&self, job: &JenkinsJob) -> Result<JobStatus>;
}

// ---------------------------------------------------------------------------
// D5 — CommitStatusContext enum + CommitStatusPoster + GitHubStatusRequest
// ---------------------------------------------------------------------------

/// The 5 required commit-status contexts posted to GitHub after Jenkins
/// completes.  These must match `infra/ci/jenkins/reported-status-contexts.json`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum CommitStatusContext {
    CargoFmt,
    CargoCheck,
    CargoClippy,
    CargoNextest,
    OyaPrReview,
}

impl CommitStatusContext {
    /// The context string exactly as registered in GitHub branch protection.
    pub const fn as_str(self) -> &'static str {
        match self {
            CommitStatusContext::CargoFmt => "cargo-fmt",
            CommitStatusContext::CargoCheck => "cargo-check",
            CommitStatusContext::CargoClippy => "cargo-clippy",
            CommitStatusContext::CargoNextest => "cargo-nextest",
            CommitStatusContext::OyaPrReview => "pr-review",
        }
    }

    /// All five contexts in stable order.
    pub const ALL: [CommitStatusContext; 5] = [
        CommitStatusContext::CargoFmt,
        CommitStatusContext::CargoCheck,
        CommitStatusContext::CargoClippy,
        CommitStatusContext::CargoNextest,
        CommitStatusContext::OyaPrReview,
    ];
}

impl std::fmt::Display for CommitStatusContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// GitHub commit status state values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitStatusState {
    /// The CI check is currently running.
    Pending,
    /// The CI check passed.
    Success,
    /// The CI check failed.
    Failure,
    /// The CI check returned an error (infrastructure failure, not gate failure).
    Error,
}

impl CommitStatusState {
    pub const fn as_str(self) -> &'static str {
        match self {
            CommitStatusState::Pending => "pending",
            CommitStatusState::Success => "success",
            CommitStatusState::Failure => "failure",
            CommitStatusState::Error => "error",
        }
    }
}

impl std::fmt::Display for CommitStatusState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The body of a `POST /repos/<owner>/<repo>/statuses/<sha>` request.
/// All fields correspond directly to the GitHub statuses API schema.
///
/// Reference: https://docs.github.com/en/rest/commits/statuses#create-a-commit-status
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitHubStatusRequest {
    /// Repository owner (org or user).
    pub owner: String, // data_class: INTERNAL_ONLY
    /// Repository name.
    pub repo: String, // data_class: INTERNAL_ONLY
    /// The commit SHA to post the status against.
    pub sha: String, // data_class: INTERNAL_ONLY
    /// The status state.
    pub state: CommitStatusState, // data_class: INTERNAL_ONLY
    /// The context string (must match the branch-protection rule).
    pub context: CommitStatusContext, // data_class: INTERNAL_ONLY
    /// Human-readable description (max 140 chars per GitHub docs).
    pub description: String, // data_class: INTERNAL_ONLY
    /// URL to link from the status check (typically the Jenkins build URL).
    pub target_url: Option<String>, // data_class: INTERNAL_ONLY
}

impl GitHubStatusRequest {
    /// Serialise to the JSON body expected by `gh api repos/<owner>/<repo>/statuses/<sha>`.
    ///
    /// The caller passes this string to `gh api --input -` or as `--field` args.
    pub fn to_api_json(&self) -> String {
        let target_url = match &self.target_url {
            Some(url) => format!(r#","target_url":"{}""#, url),
            None => String::new(),
        };
        format!(
            r#"{{"state":"{}","context":"{}","description":"{}"{}}}"#,
            self.state.as_str(),
            self.context.as_str(),
            self.description,
            target_url,
        )
    }

    /// Construct a `pending` status for a context when the Jenkins job starts.
    pub fn pending(
        owner: &str,
        repo: &str,
        sha: &str,
        context: CommitStatusContext,
        build_url: Option<&str>,
    ) -> Self {
        Self {
            owner: owner.to_owned(),
            repo: repo.to_owned(),
            sha: sha.to_owned(),
            state: CommitStatusState::Pending,
            context,
            description: format!("{context} — running"),
            target_url: build_url.map(ToOwned::to_owned),
        }
    }

    /// Construct a final status for a context based on Jenkins job outcome.
    pub fn from_job_outcome(
        owner: &str,
        repo: &str,
        sha: &str,
        context: CommitStatusContext,
        job_status: JobStatus,
        build_url: Option<&str>,
    ) -> Self {
        let (state, description) = match job_status {
            JobStatus::Success => (CommitStatusState::Success, format!("{context} — passed")),
            JobStatus::Failure => (CommitStatusState::Failure, format!("{context} — failed")),
            JobStatus::Aborted => (CommitStatusState::Error, format!("{context} — aborted")),
            _ => (
                CommitStatusState::Error,
                format!("{context} — unknown outcome"),
            ),
        };
        Self {
            owner: owner.to_owned(),
            repo: repo.to_owned(),
            sha: sha.to_owned(),
            state,
            context,
            description,
            target_url: build_url.map(ToOwned::to_owned),
        }
    }
}

/// D5 — GitHub commit-status poster seam.
///
/// Stage-5 adapter wraps `gh api repos/<owner>/<repo>/statuses/<sha>`.
pub trait CommitStatusPoster {
    /// Post a single commit status.  Never panics; returns
    /// `Err(KernelError::DownstreamTransport)` on failure.
    fn post(&self, request: &GitHubStatusRequest) -> Result<()>;

    /// Post all 5 required contexts in a single call (convenience wrapper).
    fn post_all(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
        job_status: JobStatus,
        build_url: Option<&str>,
    ) -> Result<()> {
        for context in CommitStatusContext::ALL {
            self.post(&GitHubStatusRequest::from_job_outcome(
                owner, repo, sha, context, job_status, build_url,
            ))?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// D6 — WebhookAuthzGate trait seam (Cedar policy)
// ---------------------------------------------------------------------------

/// Authorization principal for a webhook trigger request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WebhookAuthzRequest {
    /// Tenant identifier (e.g. `"oyatie-dogfood"`).  All tenants including
    /// the dogfood tenant go through the same Cedar policy path.
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    /// Source IP of the webhook sender (GitHub instance address).
    pub source_ip: String, // data_class: INTERNAL_ONLY
    /// The event type being triggered.
    pub event_type: String, // data_class: INTERNAL_ONLY
    /// The repository the webhook is for.
    pub repo: String, // data_class: INTERNAL_ONLY
}

/// Authorization decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthzDecision {
    Allow,
    Forbid,
}

/// D6 — Cedar authorization seam.
///
/// Stage-5 adapter evaluates the policy at
/// `microservices/ci-webhook-gateway/policy/ci-webhook-gateway.cedar`.
/// Dogfood doctrine: `oyatie-dogfood` is a regular tenant; no internal bypass.
pub trait WebhookAuthzGate {
    fn decide(&self, request: &WebhookAuthzRequest) -> AuthzDecision;
}

// ---------------------------------------------------------------------------
// D6 — Audit-chain emission seam
// ---------------------------------------------------------------------------

/// Kernel-level audit events emitted to the audit-chain (ADR-0193/ADR-0263).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WebhookAuditEvent {
    DeliveryAccepted {
        delivery_id: String, // data_class: INTERNAL_ONLY
        event_type: String,  // data_class: INTERNAL_ONLY
        head_sha: String,    // data_class: INTERNAL_ONLY
    },
    SignatureRejected {
        delivery_id: String, // data_class: INTERNAL_ONLY
        reason: String,      // data_class: INTERNAL_ONLY
    },
    DeliveryUnroutable {
        delivery_id: String, // data_class: INTERNAL_ONLY
        event_type: String,  // data_class: INTERNAL_ONLY
    },
    PipelineDispatched {
        delivery_id: String, // data_class: INTERNAL_ONLY
        job_name: String,    // data_class: INTERNAL_ONLY
        build_number: u64,   // data_class: INTERNAL_ONLY
    },
    PipelineDispatchFailed {
        delivery_id: String, // data_class: INTERNAL_ONLY
        reason: String,      // data_class: INTERNAL_ONLY
    },
}

/// D6 — audit-chain emission seam.  Synchronous from the kernel's view;
/// the adapter MAY batch.  Never panics.
pub trait WebhookEventSink {
    fn emit(&self, event: WebhookAuditEvent);
}

// ---------------------------------------------------------------------------
// Module-level tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_status_context_as_str_matches_branch_protection() {
        assert_eq!(CommitStatusContext::CargoFmt.as_str(), "cargo-fmt");
        assert_eq!(CommitStatusContext::CargoCheck.as_str(), "cargo-check");
        assert_eq!(CommitStatusContext::CargoClippy.as_str(), "cargo-clippy");
        assert_eq!(CommitStatusContext::CargoNextest.as_str(), "cargo-nextest");
        assert_eq!(CommitStatusContext::OyaPrReview.as_str(), "pr-review");
    }

    #[test]
    fn all_five_contexts_present() {
        assert_eq!(CommitStatusContext::ALL.len(), 5);
    }

    #[test]
    fn mock_signature_verifier_returns_configured_verdict() {
        let verifier = MockSignatureVerifier { verdict: Ok(()) };
        let sig = WebhookSignature::from_bytes(vec![0u8; 64]);
        assert!(verifier.verify(b"body", &sig, None).is_ok());

        let verifier_fail = MockSignatureVerifier {
            verdict: Err(KernelError::SignatureMismatch),
        };
        assert_eq!(
            verifier_fail.verify(b"body", &sig, None).unwrap_err(),
            KernelError::SignatureMismatch
        );
    }

    #[test]
    fn webhook_signature_redacted_in_debug() {
        let sig = WebhookSignature::from_bytes(vec![0xdeu8; 64]);
        assert_eq!(format!("{sig:?}"), "WebhookSignature(<redacted>)");
    }

    #[test]
    fn signature_from_hex_roundtrip() {
        let hex = "deadbeef01020304";
        let sig = WebhookSignature::from_hex(hex).unwrap();
        assert_eq!(
            sig.as_bytes(),
            &[0xde, 0xad, 0xbe, 0xef, 0x01, 0x02, 0x03, 0x04]
        );
    }

    #[test]
    fn signature_from_bad_hex_errors() {
        assert_eq!(
            WebhookSignature::from_hex("zzzz").unwrap_err(),
            KernelError::MalformedSignature
        );
    }

    #[test]
    fn ping_event_produces_ignored_outcome() {
        let outcome = route_github_event("ping", b"{}", "delivery-1", "dev").unwrap();
        assert!(matches!(outcome, RouteOutcome::Ignored { .. }));
    }

    #[test]
    fn unknown_event_produces_unroutable_error() {
        let err = route_github_event("wiki", b"{}", "delivery-1", "dev").unwrap_err();
        assert!(matches!(err, KernelError::UnroutableEvent { .. }));
    }

    #[test]
    fn job_status_maps_to_commit_status_state() {
        let req = GitHubStatusRequest::from_job_outcome(
            "oyatie",
            "oyatie",
            "abc123",
            CommitStatusContext::CargoNextest,
            JobStatus::Success,
            None,
        );
        assert_eq!(req.state, CommitStatusState::Success);
        assert_eq!(req.context, CommitStatusContext::CargoNextest);
    }

    #[test]
    fn github_status_request_json_contains_context_and_state() {
        let req = GitHubStatusRequest {
            owner: "oyatie".to_owned(),
            repo: "oyatie".to_owned(),
            sha: "abc123".to_owned(),
            state: CommitStatusState::Success,
            context: CommitStatusContext::CargoFmt,
            description: "cargo-fmt — passed".to_owned(),
            target_url: Some("https://jenkins.example.com/job/oyaCiLane/42/".to_owned()),
        };
        let json = req.to_api_json();
        assert!(json.contains(r#""state":"success""#));
        assert!(json.contains(r#""context":"cargo-fmt""#));
        assert!(json.contains("target_url"));
    }
}
