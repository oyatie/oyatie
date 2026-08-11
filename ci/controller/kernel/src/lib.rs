//! # oya-ci-controller-kernel
//!
//! Pure-domain kernel for the oya-ci controller (Phase 1, bespoke-Prow ADR).
//! No I/O, no async, no kube, no tokio. #![forbid(unsafe_code)].
//!
//! Owns:
//! - [`GateRun`] value object (identity + labels for the K8s Job)
//! - [`GateOutcome`] enum
//! - [`CommitState`] enum (forge-neutral commit-status vocabulary)
//! - [`JobObservation`] — the K8s-Job-observation input type
//! - [`map_job_to_status`] — the TOTAL pure function: observation → [`ReconcileDecision`]
//! - [`CommitStatusPoster`] + [`JobSpawner`] trait seams (I/O boundary)
//!
//! ## Security
//!
//! - ADR-0083 Tier-3: no `unwrap`/`expect`/`panic` on the hot path.
//! - `#![forbid(unsafe_code)]`

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// All kernel-level errors. HTTP / kube mapping lives in adapter layers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelError {
    /// A downstream component (GitHub, kube API) returned a transport failure.
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
// Commit-status vocabulary (forge-neutral)
// ---------------------------------------------------------------------------

/// Forge-neutral commit-status state values (subset used by the oya-ci gate).
/// Maps onto the GitHub status API.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommitState {
    Pending,
    Success,
    Failure,
    Error,
}

impl CommitState {
    pub const fn as_str(self) -> &'static str {
        match self {
            CommitState::Pending => "pending",
            CommitState::Success => "success",
            CommitState::Failure => "failure",
            CommitState::Error => "error",
        }
    }
}

impl std::fmt::Display for CommitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Trusted PR-review admission contract
// ---------------------------------------------------------------------------

/// Commit-status context produced by trusted review admission.
pub const REVIEW_CONTEXT: &str = "oya-pr-review";

/// Forge-neutral pull-request review verdict.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewVerdict {
    Approved,
    ChangesRequested,
    Dismissed,
    Commented,
}

impl ReviewVerdict {
    const fn is_decisive(self) -> bool {
        !matches!(self, Self::Commented)
    }
}

/// Immutable GitHub identity type returned by the forge API.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum GitHubAccountType {
    User,
    Bot,
    Organization,
}

impl GitHubAccountType {
    const fn canonical_tag(self) -> u8 {
        match self {
            Self::User => 1,
            Self::Bot => 2,
            Self::Organization => 3,
        }
    }
}

/// Immutable GitHub principal, with login retained only as display evidence.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct GitHubPrincipal {
    pub id: u64,                         // data_class: INTERNAL_ONLY
    pub account_type: GitHubAccountType, // data_class: INTERNAL_ONLY
    pub login: String,                   // data_class: INTERNAL_ONLY
}

impl GitHubPrincipal {
    fn is_valid(&self) -> bool {
        self.id != 0 && !self.login.trim().is_empty()
    }
}

/// One durable forge review observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewEvidence {
    pub review_id: u64,            // data_class: INTERNAL_ONLY
    pub head_sha: String,          // data_class: INTERNAL_ONLY
    pub reviewer: GitHubPrincipal, // data_class: INTERNAL_ONLY
    pub verdict: ReviewVerdict,    // data_class: INTERNAL_ONLY
    pub evidence_url: String,      // data_class: INTERNAL_ONLY
}

/// Versioned reviewer-eligibility policy supplied by the trusted controller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewAdmissionPolicy {
    pub policy_ref: String,                            // data_class: INTERNAL_ONLY
    pub version: String,                               // data_class: INTERNAL_ONLY
    pub sha256_digest: String,                         // data_class: INTERNAL_ONLY
    pub issuer: String,                                // data_class: INTERNAL_ONLY
    pub effective_at_unix_s: i64,                      // data_class: INTERNAL_ONLY
    pub expires_at_unix_s: i64,                        // data_class: INTERNAL_ONLY
    pub revoked: bool,                                 // data_class: INTERNAL_ONLY
    pub eligible_reviewers: BTreeSet<GitHubPrincipal>, // data_class: INTERNAL_ONLY
}

impl ReviewAdmissionPolicy {
    /// SHA-256 over every authoritative policy field in a length-delimited,
    /// versioned encoding. The receipt digest must match this exact value at
    /// the trusted admission boundary; it is not caller-asserted metadata.
    pub fn canonical_sha256(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"oya-ci/review-admission-policy/v1");
        update_canonical_string(&mut hasher, &self.policy_ref);
        update_canonical_string(&mut hasher, &self.version);
        update_canonical_string(&mut hasher, &self.issuer);
        hasher.update(self.effective_at_unix_s.to_be_bytes());
        hasher.update(self.expires_at_unix_s.to_be_bytes());
        hasher.update([u8::from(self.revoked)]);
        hasher.update((self.eligible_reviewers.len() as u64).to_be_bytes());
        for reviewer in &self.eligible_reviewers {
            hasher.update(reviewer.id.to_be_bytes());
            hasher.update([reviewer.account_type.canonical_tag()]);
            update_canonical_string(&mut hasher, &reviewer.login);
        }
        format!("{:x}", hasher.finalize())
    }

    fn admits(&self, reviewer: &GitHubPrincipal) -> bool {
        self.eligible_reviewers.iter().any(|eligible| {
            eligible.id == reviewer.id && eligible.account_type == reviewer.account_type
        })
    }
}

/// Identity of the non-live controller workload that produced this receipt.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReviewAdmissionProducer {
    pub github_app_id: u64,        // data_class: INTERNAL_ONLY
    pub workload_identity: String, // data_class: INTERNAL_ONLY
}

/// Trusted inputs used to decide review admission for one PR head.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewAdmissionInput {
    pub pr_number: u64,                    // data_class: INTERNAL_ONLY
    pub expected_head_sha: String,         // data_class: INTERNAL_ONLY
    pub observed_head_sha: String,         // data_class: INTERNAL_ONLY
    pub author: GitHubPrincipal,           // data_class: INTERNAL_ONLY
    pub policy: ReviewAdmissionPolicy,     // data_class: INTERNAL_ONLY
    pub evaluated_at_unix_s: i64,          // data_class: INTERNAL_ONLY
    pub producer: ReviewAdmissionProducer, // data_class: INTERNAL_ONLY
    pub reviews: Vec<ReviewEvidence>,      // data_class: INTERNAL_ONLY
}

/// Durable, head-bound packet emitted only after review admission succeeds.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReviewAdmissionPacket {
    pub pr_number: u64,                              // data_class: INTERNAL_ONLY
    pub head_sha: String,                            // data_class: INTERNAL_ONLY
    pub author: GitHubPrincipal,                     // data_class: INTERNAL_ONLY
    pub reviewer: GitHubPrincipal,                   // data_class: INTERNAL_ONLY
    pub reviewer_eligibility_policy_ref: String,     // data_class: INTERNAL_ONLY
    pub reviewer_eligibility_policy_version: String, // data_class: INTERNAL_ONLY
    pub reviewer_eligibility_policy_sha256: String,  // data_class: INTERNAL_ONLY
    pub reviewer_eligibility_policy_issuer: String,  // data_class: INTERNAL_ONLY
    pub policy_evaluated_at_unix_s: i64,             // data_class: INTERNAL_ONLY
    pub reviewer_eligibility_policy_effective_at_unix_s: i64, // data_class: INTERNAL_ONLY
    pub reviewer_eligibility_policy_expires_at_unix_s: i64, // data_class: INTERNAL_ONLY
    pub reviewer_eligibility_policy_revoked: bool,   // data_class: INTERNAL_ONLY
    pub producer: ReviewAdmissionProducer,           // data_class: INTERNAL_ONLY
    pub verdict: ReviewVerdict,                      // data_class: INTERNAL_ONLY
    pub evidence_url: String,                        // data_class: INTERNAL_ONLY
}

/// Validate forge review observations and select the newest effective approval.
///
/// The decision is fail-closed: the PR number, exact candidate head, author,
/// distinct reviewer, approved verdict, and durable HTTP(S) review URL are all
/// required. Candidate-authored PR title/body text is deliberately not an
/// input to this contract.
pub fn admit_review(input: &ReviewAdmissionInput) -> Result<ReviewAdmissionPacket> {
    if input.pr_number == 0 {
        return Err(KernelError::InvalidInput(
            "review admission requires a non-zero PR number".to_owned(),
        ));
    }
    if !is_full_sha(&input.expected_head_sha) || !is_full_sha(&input.observed_head_sha) {
        return Err(KernelError::InvalidInput(
            "review admission requires a full 40-hex head SHA".to_owned(),
        ));
    }
    if input.expected_head_sha != input.observed_head_sha {
        return Err(KernelError::InvalidInput(format!(
            "review admission head SHA mismatch: expected {}, observed {}",
            input.expected_head_sha, input.observed_head_sha
        )));
    }

    if !input.author.is_valid() {
        return Err(KernelError::InvalidInput(
            "review admission requires an immutable author identity".to_owned(),
        ));
    }
    let policy_ref = input.policy.policy_ref.trim();
    if policy_ref.is_empty()
        || input.policy.version.trim().is_empty()
        || !is_sha256_hex(&input.policy.sha256_digest)
        || input.policy.issuer.trim().is_empty()
        || input.policy.eligible_reviewers.is_empty()
        || input
            .policy
            .eligible_reviewers
            .iter()
            .any(|reviewer| !reviewer.is_valid())
    {
        return Err(KernelError::InvalidInput(
            "review admission requires a complete reviewer eligibility policy receipt".to_owned(),
        ));
    }
    if !input
        .policy
        .sha256_digest
        .eq_ignore_ascii_case(&input.policy.canonical_sha256())
    {
        return Err(KernelError::InvalidInput(
            "review admission policy receipt digest does not bind its authoritative bytes"
                .to_owned(),
        ));
    }
    if input.policy.revoked
        || input.policy.effective_at_unix_s > input.evaluated_at_unix_s
        || input.policy.expires_at_unix_s <= input.evaluated_at_unix_s
        || input.policy.expires_at_unix_s <= input.policy.effective_at_unix_s
    {
        return Err(KernelError::InvalidInput(
            "review admission policy receipt is not currently valid".to_owned(),
        ));
    }
    if input.producer.github_app_id == 0 || input.producer.workload_identity.trim().is_empty() {
        return Err(KernelError::InvalidInput(
            "review admission requires an identified producer workload".to_owned(),
        ));
    }

    // GitHub returns review events in creation order. Keep the highest-id
    // decisive event per reviewer so a later request-changes or dismissal
    // cannot be bypassed by an older approval. COMMENTED is non-decisive.
    let mut latest_by_reviewer: BTreeMap<(u64, GitHubAccountType), &ReviewEvidence> =
        BTreeMap::new();
    for review in &input.reviews {
        if review.head_sha != input.expected_head_sha
            || !review.reviewer.is_valid()
            || !review.verdict.is_decisive()
        {
            continue;
        }
        let key = (review.reviewer.id, review.reviewer.account_type);
        let should_replace = latest_by_reviewer
            .get(&key)
            .is_none_or(|current| review.review_id > current.review_id);
        if should_replace {
            latest_by_reviewer.insert(key, review);
        }
    }

    let mut author_approval_present = false;
    let mut ineligible_approval_present = false;
    let mut malformed_evidence_present = false;
    let mut newest_approval: Option<&ReviewEvidence> = None;
    for (reviewer_key, review) in latest_by_reviewer {
        if review.verdict != ReviewVerdict::Approved {
            continue;
        }
        if reviewer_key == (input.author.id, input.author.account_type) {
            author_approval_present = true;
            continue;
        }
        if !input.policy.admits(&review.reviewer) {
            ineligible_approval_present = true;
            continue;
        }
        if !is_durable_http_url(&review.evidence_url) {
            malformed_evidence_present = true;
            continue;
        }
        if newest_approval.is_none_or(|current| review.review_id > current.review_id) {
            newest_approval = Some(review);
        }
    }

    let Some(review) = newest_approval else {
        let reason = if author_approval_present {
            "reviewer identity must be distinct from the PR author"
        } else if ineligible_approval_present {
            "approved reviewer is not eligible under the designated reviewer policy"
        } else if malformed_evidence_present {
            "approved review is missing a durable HTTP(S) evidence URL"
        } else {
            "no current head-bound APPROVED review evidence was found"
        };
        return Err(KernelError::InvalidInput(reason.to_owned()));
    };

    Ok(ReviewAdmissionPacket {
        pr_number: input.pr_number,
        head_sha: input.expected_head_sha.clone(),
        author: input.author.clone(),
        reviewer: review.reviewer.clone(),
        reviewer_eligibility_policy_ref: policy_ref.to_owned(),
        reviewer_eligibility_policy_version: input.policy.version.trim().to_owned(),
        reviewer_eligibility_policy_sha256: input.policy.sha256_digest.to_ascii_lowercase(),
        reviewer_eligibility_policy_issuer: input.policy.issuer.trim().to_owned(),
        policy_evaluated_at_unix_s: input.evaluated_at_unix_s,
        reviewer_eligibility_policy_effective_at_unix_s: input.policy.effective_at_unix_s,
        reviewer_eligibility_policy_expires_at_unix_s: input.policy.expires_at_unix_s,
        reviewer_eligibility_policy_revoked: input.policy.revoked,
        producer: input.producer.clone(),
        verdict: ReviewVerdict::Approved,
        evidence_url: review.evidence_url.clone(),
    })
}

fn is_full_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn update_canonical_string(hasher: &mut Sha256, value: &str) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value.as_bytes());
}

fn is_durable_http_url(value: &str) -> bool {
    let value = value.trim();
    value.starts_with("https://") || value.starts_with("http://")
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
    /// Delivery ID — idempotency dedup key (mirrors gateway DeliveryKey).
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
        let sha_short: String = self.head_sha.chars().take(8).collect();
        format!("oya-ci-gate-pr{}-{sha_short}", self.pr_number)
    }
    /// Stable cloud-native run id used by status APIs, metrics, logs, traces, and events.
    ///
    /// It intentionally matches the deterministic Job name so every debugging
    /// surface can join without a side table.
    pub fn run_id(&self) -> String {
        self.job_name()
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
    /// Git clone URL (e.g. `"https://github.com/jason931225/oyatie.git"`).
    pub forge_clone_url: String, // data_class: INTERNAL_ONLY
    /// Gate deadline in seconds (mirrors the legacy CI 60 min timeout).
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
    pub job_name: String,     // data_class: INTERNAL_ONLY
    pub namespace: String,    // data_class: INTERNAL_ONLY
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
// Phase-0 CI enforcement policy vocabulary
// ---------------------------------------------------------------------------

/// Protected-branch contexts that may satisfy the P0.0 destination authority.
///
/// Legacy `oya verify` / `oya gate` invocations can provide local migration
/// evidence only; they are not accepted by this policy as merge or Phase-0 exit
/// authority.
pub const PHASE0_REQUIRED_CI_CONTEXTS: [&str; 2] = ["cloud-ci-required", "oya-ci-required"];

/// Toolchain pipeline surfaces that must be tenant-separated before any live
/// tenant-isolation claim is allowed.
pub const PHASE0_REQUIRED_TENANT_PIPELINE_SURFACES: [&str; 11] = [
    "identity",
    "secrets",
    "runners",
    "workspaces",
    "caches",
    "artifacts",
    "logs_evidence",
    "release_ledgers",
    "deploy_targets",
    "status_callbacks",
    "audit_events",
];

/// Override evidence required when a gate is temporarily disabled or degraded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Phase0OverrideEvidence {
    pub ttl_present: bool,
    pub reviewer_acknowledgment_present: bool,
    pub audit_chain_event_present: bool,
    pub owner_present: bool,
    pub blast_radius_statement_present: bool,
    pub revert_or_fix_follow_up_present: bool,
}

impl Phase0OverrideEvidence {
    pub const fn is_complete(&self) -> bool {
        self.ttl_present
            && self.reviewer_acknowledgment_present
            && self.audit_chain_event_present
            && self.owner_present
            && self.blast_radius_statement_present
            && self.revert_or_fix_follow_up_present
    }
}

/// Pure-domain input for evaluating the P0.0 required-context policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Phase0CiPolicyInput {
    pub protected_required_contexts: Vec<String>,
    pub producer_kind: Option<String>,
    pub producer_controller: Option<String>,
    pub producer_command: Option<String>,
    pub candidate_bytes_policy: Option<String>,
    pub gate_definition_source: Option<String>,
    pub override_evidence: Option<Phase0OverrideEvidence>,
    pub tenant_shared_surfaces: Vec<String>,
    pub internal_bypass_without_breakglass: bool,
}

/// Violation classes emitted by [`evaluate_phase0_ci_policy`].
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Phase0CiPolicyViolation {
    MissingCloudCiRequiredContext,
    LegacyOyaCliRequiredContext,
    UntrustedOrLegacyStatusProducer,
    CandidateBytesCanWeakenGate,
    CandidateSourcedGateDefinition,
    OverrideMissingTtlReviewerAuditOrRevert,
    TenantSurfacesShared,
    InternalBypassWithoutBreakglass,
}

impl Phase0CiPolicyViolation {
    pub const fn as_str(self) -> &'static str {
        match self {
            Phase0CiPolicyViolation::MissingCloudCiRequiredContext => {
                "missing_cloud_ci_required_context"
            }
            Phase0CiPolicyViolation::LegacyOyaCliRequiredContext => {
                "legacy_oya_cli_required_context"
            }
            Phase0CiPolicyViolation::UntrustedOrLegacyStatusProducer => {
                "untrusted_or_legacy_status_producer"
            }
            Phase0CiPolicyViolation::CandidateBytesCanWeakenGate => {
                "candidate_bytes_can_weaken_gate"
            }
            Phase0CiPolicyViolation::CandidateSourcedGateDefinition => {
                "candidate_sourced_gate_definition"
            }
            Phase0CiPolicyViolation::OverrideMissingTtlReviewerAuditOrRevert => {
                "override_missing_ttl_reviewer_audit_or_revert"
            }
            Phase0CiPolicyViolation::TenantSurfacesShared => "tenant_surfaces_shared",
            Phase0CiPolicyViolation::InternalBypassWithoutBreakglass => {
                "internal_bypass_without_breakglass"
            }
        }
    }
}

/// Result of the pure P0.0 policy evaluation. Empty violations means the input
/// satisfies the destination *target* policy; it does not prove live branch
/// protection or external status production.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Phase0CiPolicyVerdict {
    pub violations: BTreeSet<Phase0CiPolicyViolation>,
}

impl Phase0CiPolicyVerdict {
    pub fn is_green(&self) -> bool {
        self.violations.is_empty()
    }
}

pub fn phase0_context_is_required_authority(context: &str) -> bool {
    PHASE0_REQUIRED_CI_CONTEXTS.contains(&context)
}

pub fn evaluate_phase0_ci_policy(input: &Phase0CiPolicyInput) -> Phase0CiPolicyVerdict {
    let mut violations = BTreeSet::new();

    if !input
        .protected_required_contexts
        .iter()
        .any(|context| phase0_context_is_required_authority(context))
    {
        violations.insert(Phase0CiPolicyViolation::MissingCloudCiRequiredContext);
    }
    if input
        .protected_required_contexts
        .iter()
        .any(|context| context == "oya-verify")
    {
        violations.insert(Phase0CiPolicyViolation::LegacyOyaCliRequiredContext);
    }

    let producer_kind = input.producer_kind.as_deref().unwrap_or_default();
    let producer_controller = input.producer_controller.as_deref().unwrap_or_default();
    let producer_command = input.producer_command.as_deref().unwrap_or_default();
    if matches!(
        producer_kind,
        "legacy_local_cli" | "candidate_checkout_script"
    ) || producer_command.contains("oya ")
        || !matches!(
            producer_kind,
            "minimal_rust_bridge_adapter" | "oya-ci-controller"
        )
        || producer_controller != "oya-ci-controller"
    {
        violations.insert(Phase0CiPolicyViolation::UntrustedOrLegacyStatusProducer);
    }

    if input.candidate_bytes_policy.as_deref() != Some("untrusted_input_only") {
        violations.insert(Phase0CiPolicyViolation::CandidateBytesCanWeakenGate);
    }
    if input.gate_definition_source.as_deref() != Some("trusted_dev_or_controller_state") {
        violations.insert(Phase0CiPolicyViolation::CandidateSourcedGateDefinition);
    }

    if input
        .override_evidence
        .as_ref()
        .is_some_and(|evidence| !evidence.is_complete())
    {
        violations.insert(Phase0CiPolicyViolation::OverrideMissingTtlReviewerAuditOrRevert);
    }

    if !input.tenant_shared_surfaces.is_empty() {
        violations.insert(Phase0CiPolicyViolation::TenantSurfacesShared);
    }
    if input.internal_bypass_without_breakglass {
        violations.insert(Phase0CiPolicyViolation::InternalBypassWithoutBreakglass);
    }

    Phase0CiPolicyVerdict { violations }
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

impl From<&str> for PodReason {
    /// Parse from a raw K8s reason string.
    fn from(s: &str) -> Self {
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
}

impl PodReason {
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
    /// Whether a terminal commit status was already posted
    /// (from annotation `oya.io/ci-status-posted`).
    pub terminal_status_already_posted: Option<CommitState>,
    /// Whether the pending status was already posted
    /// (from annotation `oya.io/ci-status-posted` == "pending").
    pub pending_status_already_posted: bool,
}

// ---------------------------------------------------------------------------
// ReconcileDecision — output of the pure state machine
// ---------------------------------------------------------------------------

/// The controller's decided action for this reconcile cycle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReconcileDecision {
    /// Job is active / pending — post pending status if not yet posted, requeue.
    PostPending { description: String },
    /// Already posted pending; just requeue to watch for progress.
    AwaitChange,
    /// Job reached a terminal state — post this commit status.
    PostTerminal {
        state: CommitState,
        context: &'static str,
        description: String,
    },
    /// Terminal status was already posted — nothing to do.
    AlreadyTerminal,
}

// The protected required context for the P0.0 cloud-ci/oya-ci target. Legacy
// `oya-ci-gate` can remain bridge feedback only; it is not merge or Phase-0
// exit authority.
pub const GATE_CONTEXT: &str = "oya-ci-required";

// ---------------------------------------------------------------------------
// Gate-run observability contract — API-native debug packet
// ---------------------------------------------------------------------------

/// Stable schema label for the productized oya-ci run observability envelope.
pub const GATE_RUN_OBSERVABILITY_SCHEMA: &str = "oya-ci/run-observability-packet/v1";

/// Status API path prefix. Runtime routers expose run status by deterministic run id.
pub const GATE_RUN_STATUS_API_PREFIX: &str = "/gate-runs";

// Run-joined events and traces are reserved for the next telemetry adapter
// stage; this packet advertises only surfaces emitted by the controller today.

/// Metric names exposed by the controller and backing telemetry adapter.
pub const GATE_RUN_OBSERVABILITY_METRICS: [&str; 5] = [
    "oya_ci_gate_run_requests_total",
    "oya_ci_gate_status_api_requests_total",
    "oya_ci_gate_job_spawn_total",
    "oya_ci_gate_reconcile_total",
    "oya_ci_gate_status_post_total",
];

/// Log fields required on structured controller logs for API/debug correlation.
pub const GATE_RUN_OBSERVABILITY_LOG_FIELDS: [&str; 6] =
    ["run_id", "job", "namespace", "pr", "sha", "decision"];

/// Productized lifecycle phase projected onto the status API and telemetry packet.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GateRunObservabilityPhase {
    Accepted,
    Running,
    AwaitingChange,
    Passed,
    Failed,
    Errored,
    AlreadyTerminal,
}

impl GateRunObservabilityPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Running => "running",
            Self::AwaitingChange => "awaiting_change",
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Errored => "errored",
            Self::AlreadyTerminal => "already_terminal",
        }
    }
}

/// Machine-readable status/API/telemetry join packet for one oya-ci gate run.
///
/// This is not a loose evidence file. It is the API-native envelope that lets
/// cloud-native debuggers join status APIs, controller logs, metrics, and K8s
/// Job state by a single deterministic `run_id`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GateRunObservabilityPacket {
    pub schema: &'static str,
    pub run_id: String,
    pub job_name: String,
    pub namespace: String,
    pub required_context: &'static str,
    pub pr_number: u64,
    pub head_sha: String,
    pub base_ref: String,
    pub status_api_path: String,
    pub status_url: Option<String>,
    pub phase: GateRunObservabilityPhase,
    pub metrics: Vec<&'static str>,
    pub logs: Vec<&'static str>,
}

/// Structured Job condition projection exposed by the status API.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GateRunK8sConditionProjection {
    pub condition_type: &'static str,
    pub reason: Option<String>,
    pub status: bool,
}

/// Structured Pod reason projection exposed by the status API.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GateRunK8sPodReasonProjection {
    pub reason: String,
}

/// Productized K8s status projection for one gate run.
///
/// This carries the raw state-machine inputs that drove the public phase so API
/// callers can debug without reverse-engineering controller logs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GateRunK8sProjection {
    pub complete: bool,
    pub active: i32,
    pub succeeded: i32,
    pub failed: i32,
    pub waiting_cycles: u32,
    pub job_not_found: bool,
    pub terminal_status_already_posted: Option<CommitState>,
    pub pending_status_already_posted: bool,
    pub conditions: Vec<GateRunK8sConditionProjection>,
    pub pod_reasons: Vec<GateRunK8sPodReasonProjection>,
}

fn condition_type_name(condition_type: &JobConditionType) -> &'static str {
    match condition_type {
        JobConditionType::Complete => "Complete",
        JobConditionType::Failed => "Failed",
    }
}

fn pod_reason_name(reason: &PodReason) -> String {
    match reason {
        PodReason::ImagePullBackOff => "ImagePullBackOff".to_owned(),
        PodReason::ErrImagePull => "ErrImagePull".to_owned(),
        PodReason::InvalidImageName => "InvalidImageName".to_owned(),
        PodReason::CreateContainerError => "CreateContainerError".to_owned(),
        PodReason::CreateContainerConfigError => "CreateContainerConfigError".to_owned(),
        PodReason::RunContainerError => "RunContainerError".to_owned(),
        PodReason::CrashLoopBackOff => "CrashLoopBackOff".to_owned(),
        PodReason::OOMKilled => "OOMKilled".to_owned(),
        PodReason::Evicted => "Evicted".to_owned(),
        PodReason::Other(value) => value.clone(),
    }
}

/// Build a stable status API path for a deterministic oya-ci run id.
pub fn gate_run_status_api_path(run_id: &str) -> String {
    format!("{GATE_RUN_STATUS_API_PREFIX}/{run_id}")
}

/// Build an absolute status URL when the controller is configured with a public/internal base URL.
pub fn gate_run_status_url(run_id: &str, status_api_base_url: Option<&str>) -> Option<String> {
    let base = status_api_base_url?.trim();
    if base.is_empty() {
        return None;
    }
    Some(format!(
        "{}/{}",
        base.trim_end_matches('/'),
        gate_run_status_api_path(run_id).trim_start_matches('/')
    ))
}

/// Classify a reconcile decision into the public run-observability lifecycle.
pub fn observability_phase_for_decision(decision: &ReconcileDecision) -> GateRunObservabilityPhase {
    match decision {
        ReconcileDecision::PostPending { .. } => GateRunObservabilityPhase::Running,
        ReconcileDecision::AwaitChange => GateRunObservabilityPhase::AwaitingChange,
        ReconcileDecision::AlreadyTerminal => GateRunObservabilityPhase::AlreadyTerminal,
        ReconcileDecision::PostTerminal { state, .. } => match state {
            CommitState::Success => GateRunObservabilityPhase::Passed,
            CommitState::Failure => GateRunObservabilityPhase::Failed,
            CommitState::Error => GateRunObservabilityPhase::Errored,
            CommitState::Pending => GateRunObservabilityPhase::Running,
        },
    }
}

/// Build the productized observability packet returned from the gate-run API.
pub fn build_gate_run_observability_packet(
    spec: &GateRunSpec,
    handle: &JobHandle,
    phase: GateRunObservabilityPhase,
    status_api_base_url: Option<&str>,
) -> GateRunObservabilityPacket {
    let run_id = spec.run.run_id();
    GateRunObservabilityPacket {
        schema: GATE_RUN_OBSERVABILITY_SCHEMA,
        status_api_path: gate_run_status_api_path(&run_id),
        status_url: gate_run_status_url(&run_id, status_api_base_url),
        run_id,
        job_name: handle.job_name.clone(),
        namespace: handle.namespace.clone(),
        required_context: GATE_CONTEXT,
        pr_number: spec.run.pr_number,
        head_sha: spec.run.head_sha.clone(),
        base_ref: spec.run.base_ref.clone(),
        phase,
        metrics: GATE_RUN_OBSERVABILITY_METRICS.to_vec(),
        logs: GATE_RUN_OBSERVABILITY_LOG_FIELDS.to_vec(),
    }
}

/// Build the structured K8s status projection returned from the run-status API.
pub fn build_gate_run_k8s_projection(
    observation: &JobObservation,
    complete: bool,
) -> GateRunK8sProjection {
    GateRunK8sProjection {
        complete,
        active: observation.active,
        succeeded: observation.succeeded,
        failed: observation.failed,
        waiting_cycles: observation.waiting_cycles,
        job_not_found: observation.job_not_found,
        terminal_status_already_posted: observation.terminal_status_already_posted,
        pending_status_already_posted: observation.pending_status_already_posted,
        conditions: observation
            .conditions
            .iter()
            .map(|condition| GateRunK8sConditionProjection {
                condition_type: condition_type_name(&condition.condition_type),
                reason: condition.reason.clone(),
                status: condition.status,
            })
            .collect(),
        pod_reasons: observation
            .pod_reasons
            .iter()
            .map(|reason| GateRunK8sPodReasonProjection {
                reason: pod_reason_name(reason),
            })
            .collect(),
    }
}

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
            state: CommitState::Failure,
            context: GATE_CONTEXT,
            description: "oya-ci-required: run disappeared (job deleted before verdict posted)"
                .to_owned(),
        };
    }

    // If a terminal status is already posted, nothing to do.
    if let Some(posted) = obs.terminal_status_already_posted
        && posted != CommitState::Pending
    {
        return ReconcileDecision::AlreadyTerminal;
    }

    // ---- Job conditions (terminal) ----------------------------------------

    // Complete condition (succeeded >= 1 or condition Complete=True)
    let is_complete = obs.succeeded >= 1
        || obs
            .conditions
            .iter()
            .any(|c| c.status && matches!(c.condition_type, JobConditionType::Complete));

    if is_complete {
        return ReconcileDecision::PostTerminal {
            state: CommitState::Success,
            context: GATE_CONTEXT,
            description: "oya-ci-required full gate target passed".to_owned(),
        };
    }

    // Failed condition
    let failed_condition = obs
        .conditions
        .iter()
        .find(|c| c.status && matches!(c.condition_type, JobConditionType::Failed));

    if let Some(cond) = failed_condition {
        let reason = cond.reason.as_deref().unwrap_or("");
        let description = match reason {
            "DeadlineExceeded" => "oya-ci-required failed: deadline exceeded (timeout)".to_owned(),
            _ => {
                // BackoffLimitExceeded or unknown — gate logic failure
                "oya-ci-required failed: required gate exited non-zero".to_owned()
            }
        };
        return ReconcileDecision::PostTerminal {
            state: CommitState::Failure,
            context: GATE_CONTEXT,
            description,
        };
    }

    // ---- Pod-level terminal reasons ----------------------------------------

    for reason in &obs.pod_reasons {
        match reason {
            PodReason::OOMKilled => {
                return ReconcileDecision::PostTerminal {
                    state: CommitState::Failure,
                    context: GATE_CONTEXT,
                    description: "oya-ci-required failed: OOMKilled — raise Job memory limit"
                        .to_owned(),
                };
            }
            PodReason::Evicted => {
                return ReconcileDecision::PostTerminal {
                    state: CommitState::Failure,
                    context: GATE_CONTEXT,
                    description:
                        "oya-ci-required failed: pod evicted (node-pressure or preemption)"
                            .to_owned(),
                };
            }
            // InvalidImageName is a misconfiguration — not transient, terminal immediately.
            PodReason::InvalidImageName => {
                return ReconcileDecision::PostTerminal {
                    state: CommitState::Error,
                    context: GATE_CONTEXT,
                    description:
                        "oya-ci-required error: InvalidImageName — operator must fix gate image config"
                            .to_owned(),
                };
            }
            // Apply bounded grace: past grace_cycles, declare terminal.
            r if r.is_pull_or_container_error() && obs.waiting_cycles >= grace_cycles => {
                let label = match r {
                    PodReason::ImagePullBackOff | PodReason::ErrImagePull => {
                        "image pull failed (rust-ci:dev unavailable)"
                    }
                    _ => "container setup failed (CreateContainerError or CrashLoopBackOff)",
                };
                return ReconcileDecision::PostTerminal {
                    state: CommitState::Failure,
                    context: GATE_CONTEXT,
                    description: format!("oya-ci-required failed: {label}"),
                };
            }
            // Pull/container errors within grace fall through to pending / await-change below.
            _ => {}
        }
    }

    // ---- Active / pending (non-terminal) -----------------------------------

    if obs.active >= 1 || obs.failed == 0 {
        if obs.pending_status_already_posted {
            return ReconcileDecision::AwaitChange;
        }
        return ReconcileDecision::PostPending {
            description: "oya-ci-required: running trusted required gate target".to_owned(),
        };
    }

    // Fallback: failed count > 0 but no condition yet — treat as gate failure.
    ReconcileDecision::PostTerminal {
        state: CommitState::Failure,
        context: GATE_CONTEXT,
        description: "oya-ci-required failed: required gate exited non-zero".to_owned(),
    }
}

// ---------------------------------------------------------------------------
// Trait seams (I/O boundary — implemented by adapter crates)
// ---------------------------------------------------------------------------

/// Forge-neutral seam for posting commit-status updates.
pub trait CommitStatusPoster: Send + Sync {
    /// POST a commit status to the forge's statuses endpoint for `sha`.
    /// Returns `Err(KernelError::DownstreamTransport)` on non-2xx or transport error.
    fn post(
        &self,
        sha: &str,
        state: CommitState,
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
            ReconcileDecision::PostTerminal { state, context, .. } => {
                assert_eq!(state, CommitState::Success);
                assert_eq!(context, GATE_CONTEXT);
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
                assert_eq!(state, CommitState::Success);
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
            ReconcileDecision::PostTerminal {
                state, description, ..
            } => {
                assert_eq!(state, CommitState::Failure);
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
            ReconcileDecision::PostTerminal {
                state, description, ..
            } => {
                assert_eq!(state, CommitState::Failure);
                assert!(
                    description.contains("deadline exceeded"),
                    "desc: {description}"
                );
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
            ReconcileDecision::PostTerminal {
                state, description, ..
            } => {
                assert_eq!(state, CommitState::Failure);
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
            ReconcileDecision::PostTerminal {
                state, description, ..
            } => {
                assert_eq!(state, CommitState::Failure);
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
            ReconcileDecision::PostTerminal {
                state, description, ..
            } => {
                assert_eq!(state, CommitState::Error);
                assert!(
                    description.contains("InvalidImageName"),
                    "desc: {description}"
                );
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
            ReconcileDecision::PostTerminal {
                state, description, ..
            } => {
                assert_eq!(state, CommitState::Failure);
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
                assert_eq!(state, CommitState::Failure);
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
            ReconcileDecision::PostTerminal {
                state, description, ..
            } => {
                assert_eq!(state, CommitState::Failure);
                assert!(description.contains("disappeared"), "desc: {description}");
            }
            other => panic!("expected failure, got {other:?}"),
        }
    }

    #[test]
    fn job_not_found_with_prior_terminal_returns_already_terminal() {
        let obs = JobObservation {
            job_not_found: true,
            terminal_status_already_posted: Some(CommitState::Success),
            ..base_obs()
        };
        assert_eq!(
            map_job_to_status(&obs, 12),
            ReconcileDecision::AlreadyTerminal
        );
    }

    // ---- Already terminal idempotency guard --------------------------------

    #[test]
    fn terminal_already_posted_returns_already_terminal() {
        let obs = JobObservation {
            terminal_status_already_posted: Some(CommitState::Success),
            ..base_obs()
        };
        assert_eq!(
            map_job_to_status(&obs, 12),
            ReconcileDecision::AlreadyTerminal
        );
    }

    #[test]
    fn terminal_failure_already_posted_returns_already_terminal() {
        let obs = JobObservation {
            failed: 1,
            terminal_status_already_posted: Some(CommitState::Failure),
            conditions: vec![JobCondition {
                condition_type: JobConditionType::Failed,
                reason: Some("BackoffLimitExceeded".to_owned()),
                status: true,
            }],
            ..base_obs()
        };
        assert_eq!(
            map_job_to_status(&obs, 12),
            ReconcileDecision::AlreadyTerminal
        );
    }

    // ---- Commit-status state vocabulary -----------------------------------

    #[test]
    fn commit_state_as_str_matches_api() {
        assert_eq!(CommitState::Pending.as_str(), "pending");
        assert_eq!(CommitState::Success.as_str(), "success");
        assert_eq!(CommitState::Failure.as_str(), "failure");
        assert_eq!(CommitState::Error.as_str(), "error");
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

    #[test]
    fn gate_context_is_phase0_required_context() {
        assert_eq!(GATE_CONTEXT, "oya-ci-required");
        assert!(phase0_context_is_required_authority(GATE_CONTEXT));
    }

    #[test]
    fn gate_run_observability_packet_is_api_native_and_joinable() {
        let spec = GateRunSpec {
            run: GateRun {
                pr_number: 42,
                head_sha: "abcdef1234567890abcdef1234567890abcdef12".to_owned(),
                delivery_id: "d1".to_owned(),
                base_ref: "dev".to_owned(),
                repo: "jason931225/oyatie".to_owned(),
            },
            image: "registry.local/rust-ci:dev".to_owned(),
            forge_clone_url: "https://github.com/jason931225/oyatie.git".to_owned(),
            active_deadline_seconds: 3600,
            ttl_seconds_after_finished: 86400,
            namespace: "oya-ci".to_owned(),
            runner_service_account: "oya-ci-gate-runner".to_owned(),
        };
        let handle = JobHandle {
            job_name: spec.run.job_name(),
            namespace: "oya-ci".to_owned(),
            already_exists: false,
        };

        let packet = build_gate_run_observability_packet(
            &spec,
            &handle,
            GateRunObservabilityPhase::Accepted,
            Some("https://ci.example.test/"),
        );

        assert_eq!(packet.schema, GATE_RUN_OBSERVABILITY_SCHEMA);
        assert_eq!(packet.run_id, "oya-ci-gate-pr42-abcdef12");
        assert_eq!(
            packet.status_api_path,
            "/gate-runs/oya-ci-gate-pr42-abcdef12"
        );
        assert_eq!(
            packet.status_url.as_deref(),
            Some("https://ci.example.test/gate-runs/oya-ci-gate-pr42-abcdef12")
        );
        assert!(packet.metrics.contains(&"oya_ci_gate_reconcile_total"));
        assert!(packet.logs.contains(&"decision"));

        let json = serde_json::to_value(&packet).expect("packet serializes");
        assert_eq!(json["run_id"], "oya-ci-gate-pr42-abcdef12");
        assert_eq!(json["phase"], "accepted");
        assert!(
            !json.to_string().contains("multispectrum"),
            "retired loose evidence convention must not leak into run observability"
        );
    }

    #[test]
    fn gate_run_k8s_projection_carries_state_machine_inputs() {
        let projection = build_gate_run_k8s_projection(
            &JobObservation {
                active: 1,
                succeeded: 0,
                failed: 0,
                conditions: vec![JobCondition {
                    condition_type: JobConditionType::Complete,
                    reason: Some("Manual".to_owned()),
                    status: false,
                }],
                pod_reasons: vec![PodReason::ImagePullBackOff],
                waiting_cycles: 3,
                job_not_found: false,
                terminal_status_already_posted: None,
                pending_status_already_posted: true,
            },
            true,
        );
        assert!(projection.complete);
        assert_eq!(projection.active, 1);
        assert_eq!(projection.conditions[0].condition_type, "Complete");
        assert_eq!(projection.pod_reasons[0].reason, "ImagePullBackOff");
    }

    #[test]
    fn reconcile_decision_maps_to_observability_phase() {
        assert_eq!(
            observability_phase_for_decision(&ReconcileDecision::PostPending {
                description: "running".to_owned()
            }),
            GateRunObservabilityPhase::Running
        );
        assert_eq!(
            observability_phase_for_decision(&ReconcileDecision::PostTerminal {
                state: CommitState::Failure,
                context: GATE_CONTEXT,
                description: "failed".to_owned(),
            }),
            GateRunObservabilityPhase::Failed
        );
    }
}

#[cfg(test)]
mod phase0_ci_enforcement_baseline_tests {
    use std::{collections::BTreeSet, fs, path::PathBuf};

    use serde_json::Value;

    use super::{
        PHASE0_REQUIRED_TENANT_PIPELINE_SURFACES, Phase0CiPolicyInput, Phase0OverrideEvidence,
        evaluate_phase0_ci_policy, phase0_context_is_required_authority,
    };

    fn repo_root() -> PathBuf {
        let mut dir = std::env::current_dir().expect("test process should expose current_dir");
        for _ in 0..12 {
            if dir.join("specs/root-hub-pointers.json").is_file() {
                return dir;
            }
            if !dir.pop() {
                break;
            }
        }
        panic!("failed to locate repository root from test current_dir");
    }

    fn load_json(repo_relative_path: &str) -> Value {
        let path = repo_root().join(repo_relative_path);
        let contents = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        serde_json::from_str(&contents)
            .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()))
    }

    fn string_array_at<'a>(json: &'a Value, path: &[&str]) -> Vec<&'a str> {
        let mut cursor = json;
        for segment in path {
            cursor = &cursor[*segment];
        }
        cursor
            .as_array()
            .unwrap_or_else(|| panic!("expected array at {path:?}"))
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .unwrap_or_else(|| panic!("expected string entry at {path:?}"))
            })
            .collect()
    }

    fn optional_string_array_at<'a>(json: &'a Value, path: &[&str]) -> Vec<&'a str> {
        let mut cursor = json;
        for segment in path {
            cursor = &cursor[*segment];
        }
        cursor
            .as_array()
            .map(|values| {
                values
                    .iter()
                    .map(|value| {
                        value
                            .as_str()
                            .unwrap_or_else(|| panic!("expected string entry at {path:?}"))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn has_non_empty_string(json: &Value, path: &[&str]) -> bool {
        let mut cursor = json;
        for segment in path {
            cursor = &cursor[*segment];
        }
        cursor
            .as_str()
            .is_some_and(|value| !value.trim().is_empty())
    }

    fn string_at<'a>(json: &'a Value, path: &[&str]) -> Option<&'a str> {
        let mut cursor = json;
        for segment in path {
            cursor = cursor.get(*segment)?;
        }
        cursor.as_str()
    }

    fn object_array_at<'a>(json: &'a Value, path: &[&str]) -> Vec<&'a Value> {
        let mut cursor = json;
        for segment in path {
            cursor = &cursor[*segment];
        }
        cursor
            .as_array()
            .unwrap_or_else(|| panic!("expected object array at {path:?}"))
            .iter()
            .collect()
    }

    fn required_string_set(json: &Value, path: &[&str]) -> BTreeSet<String> {
        string_array_at(json, path)
            .into_iter()
            .map(str::to_owned)
            .collect()
    }

    fn expected_violation_set(json: &Value) -> BTreeSet<String> {
        optional_string_array_at(json, &["expected_violations"])
            .into_iter()
            .map(str::to_owned)
            .collect()
    }

    fn field_is_present_and_non_empty(row: &Value, field: &str) -> bool {
        match &row[field] {
            Value::String(value) => !value.trim().is_empty(),
            Value::Bool(_) => true,
            Value::Array(values) => !values.is_empty(),
            Value::Object(values) => !values.is_empty(),
            _ => false,
        }
    }

    fn contains_oya_cli_authority(value: &str) -> bool {
        let lower = value.to_ascii_lowercase();
        lower.contains("oya gate")
            || lower.contains("oya verify")
            || lower.contains("oya --")
            || lower.contains("`oya`")
            || lower.contains("legacy oya cli invocation")
    }

    fn mentions_retired_multispectrum_evidence(value: &str) -> bool {
        value
            .split([' ', '+', ',', ';', '\n', '\t'])
            .any(|part| part.trim().starts_with("evidence/multispectrum/"))
    }

    fn has_pre_merge_review_authority(row: &Value) -> bool {
        let live_authority = row["review_authority_live"].as_bool() == Some(true);
        let durable_evidence = row["has_durable_review_evidence"].as_bool() == Some(true);
        let machine_status = row["has_machine_verifiable_review_status"].as_bool() == Some(true);
        let binds_pr_number = row["binds_pr_number"].as_bool() == Some(true);
        let binds_head_sha = row["binds_head_sha"].as_bool() == Some(true);
        let binds_author = row["binds_author_identity"].as_bool() == Some(true);
        let binds_reviewer = row["binds_reviewer_identity"].as_bool() == Some(true);
        let binds_verdict = row["binds_review_verdict"].as_bool() == Some(true);
        let trusted_source = matches!(
            row["review_authority_source"].as_str().map(str::trim),
            Some(
                "trusted_runner_signed_oya_pr_review_status"
                    | "trusted_cloud_ci_review_admission_packet"
                    | "trusted_server_side_oya_pr_review_status"
            )
        );
        let blocks_merge = row["review_blocks_merge"].as_bool() == Some(true);
        let reviewer_distinct =
            row["reviewer_identity_distinct_from_author"].as_bool() == Some(true);
        live_authority
            && trusted_source
            && durable_evidence
            && machine_status
            && binds_pr_number
            && binds_head_sha
            && binds_author
            && binds_reviewer
            && binds_verdict
            && blocks_merge
            && reviewer_distinct
    }

    fn is_full_hex_sha(value: &str) -> bool {
        value.len() == 40 && value.chars().all(|c| c.is_ascii_hexdigit())
    }

    fn assert_declared_schema_keys(instance: &Value, schema: &Value, label: &str) {
        if schema["additionalProperties"].as_bool() == Some(false) {
            let allowed_keys: BTreeSet<&str> = schema["properties"]
                .as_object()
                .unwrap_or_else(|| panic!("{label} schema lacks properties"))
                .keys()
                .map(String::as_str)
                .collect();
            for key in instance
                .as_object()
                .unwrap_or_else(|| panic!("{label} instance is not an object"))
                .keys()
            {
                assert!(
                    allowed_keys.contains(key.as_str()),
                    "{label} carries undeclared schema key {key}"
                );
            }
        }

        if let (Some(properties), Some(object)) =
            (schema["properties"].as_object(), instance.as_object())
        {
            for (key, child_schema) in properties {
                if let Some(child_instance) = object.get(key) {
                    assert_declared_schema_keys(
                        child_instance,
                        child_schema,
                        &format!("{label}.{key}"),
                    );
                }
            }
        }

        if let (Some(items_schema), Some(items)) = (schema.get("items"), instance.as_array()) {
            for (index, item) in items.iter().enumerate() {
                assert_declared_schema_keys(item, items_schema, &format!("{label}[{index}]"));
            }
        }
    }

    fn evaluate_automation_rows(
        rows: &[&Value],
        required_fields: &[&str],
        allowed_classifications: &BTreeSet<String>,
        enforce_review_authority: bool,
    ) -> BTreeSet<String> {
        let mut violations = BTreeSet::new();
        let mut ids = BTreeSet::new();

        for row in rows {
            for required_field in required_fields {
                if !field_is_present_and_non_empty(row, required_field) {
                    violations.insert("missing_or_empty_required_field".to_owned());
                }
            }

            if let Some(id) = row["id"].as_str()
                && !ids.insert(id.to_owned())
            {
                violations.insert("duplicate_row_id".to_owned());
            }

            let classification = row["classification"].as_str().unwrap_or_default();
            if !allowed_classifications.contains(classification) {
                violations.insert("unknown_classification".to_owned());
            }

            if row["no_new_oya_cli_surface"].as_bool() != Some(true) {
                violations.insert("blocking_invariant_mapped_to_oya_cli".to_owned());
            }
            if row["target_gate_or_controller"]
                .as_str()
                .is_some_and(contains_oya_cli_authority)
            {
                violations.insert("blocking_invariant_mapped_to_oya_cli".to_owned());
            }

            if row["evidence_path"]
                .as_str()
                .is_some_and(mentions_retired_multispectrum_evidence)
            {
                violations.insert("retired_multispectrum_evidence".to_owned());
            }

            if classification == "not_automatable_human_judgment"
                && row["enforceable_or_automatable"].as_bool() == Some(true)
            {
                violations.insert("enforceable_or_automatable_marked_human_judgment".to_owned());
            }
            if enforce_review_authority
                && row["requires_pre_merge_review_authority"].as_bool() == Some(true)
                && !has_pre_merge_review_authority(row)
            {
                violations.insert("missing_pre_merge_review_authority".to_owned());
            }
        }

        violations
    }

    fn regulated_terms_in_text(text: &str, vocabulary: &BTreeSet<String>) -> BTreeSet<String> {
        let lower = text.to_ascii_lowercase();
        vocabulary
            .iter()
            .filter(|term| lower.contains(&term.to_ascii_lowercase()))
            .cloned()
            .collect()
    }

    fn claim_row_terms(row: &Value) -> BTreeSet<String> {
        optional_string_array_at(row, &["regulated_terms"])
            .into_iter()
            .map(str::to_owned)
            .collect()
    }

    fn claim_row_evidence(row: &Value) -> String {
        row["current_evidence"]
            .as_array()
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default()
    }

    fn evaluate_claim_fixture(
        fixture: &Value,
        regulated_vocabulary: &BTreeSet<String>,
        allowed_tiers: &BTreeSet<String>,
    ) -> BTreeSet<String> {
        let mut violations = BTreeSet::new();
        let text = fixture["text"].as_str().unwrap_or_default();
        let observed_terms = regulated_terms_in_text(text, regulated_vocabulary);
        let rows = object_array_at(fixture, &["claim_rows"]);

        if !observed_terms.is_empty() && rows.is_empty() {
            violations.insert("regulated_vocabulary_without_claim_row".to_owned());
        }

        let mut covered_terms = BTreeSet::new();
        for row in rows {
            for field in [
                "id",
                "artifact",
                "claim_text",
                "claim_tier",
                "allowed_language_now",
                "regulated_terms",
                "current_evidence",
                "owner",
            ] {
                if !field_is_present_and_non_empty(row, field) {
                    violations.insert("missing_or_empty_claim_row_field".to_owned());
                }
            }

            let tier = row["claim_tier"].as_str().unwrap_or_default();
            if !allowed_tiers.contains(tier) {
                violations.insert("unknown_claim_tier".to_owned());
            }

            covered_terms.extend(claim_row_terms(row));
            let evidence = claim_row_evidence(row);
            if tier == "mechanically_enforced"
                && (contains_oya_cli_authority(&evidence) || contains_oya_cli_authority(text))
            {
                violations
                    .insert("forbidden_local_or_oya_evidence_for_mechanical_claim".to_owned());
            }

            let terms = claim_row_terms(row);
            let performance_claim = terms.iter().any(|term| {
                matches!(
                    term.as_str(),
                    "performance" | "performant" | "low-latency" | "capacity" | "capacity-ready"
                )
            });
            if performance_claim
                && matches!(tier, "production_ready" | "hyperscaler_grade")
                && !(evidence.contains("p50")
                    && evidence.contains("p95")
                    && evidence.contains("p99")
                    && evidence.contains("load")
                    && evidence.contains("measured_result"))
            {
                violations.insert("performance_claim_without_budget_or_measured_result".to_owned());
            }
        }

        for observed_term in observed_terms {
            if !covered_terms.contains(&observed_term) {
                violations.insert("regulated_vocabulary_without_claim_row".to_owned());
            }
        }

        violations
    }

    fn aggregate_exit_is_green(subconditions: &Value) -> bool {
        subconditions.as_object().is_some_and(|conditions| {
            !conditions.is_empty()
                && conditions
                    .values()
                    .all(|value| value.as_bool() == Some(true))
        })
    }

    fn phase0_policy_input_for_fixture(fixture: &Value) -> Phase0CiPolicyInput {
        let required_contexts =
            optional_string_array_at(fixture, &["branch_protection", "required_contexts"])
                .into_iter()
                .map(str::to_owned)
                .collect();
        let producer_contract = &fixture["producer_contract"];

        let override_packet = &fixture["override_packet"];
        let tenant_model = &fixture["tenant_pipeline_model"];

        Phase0CiPolicyInput {
            protected_required_contexts: required_contexts,
            producer_kind: producer_contract["kind"].as_str().map(str::to_owned),
            producer_controller: producer_contract["controller"].as_str().map(str::to_owned),
            producer_command: producer_contract["command"].as_str().map(str::to_owned),
            candidate_bytes_policy: producer_contract["candidate_bytes_policy"]
                .as_str()
                .map(str::to_owned),
            gate_definition_source: producer_contract["gate_definition_source"]
                .as_str()
                .map(str::to_owned),
            override_evidence: override_packet.is_object().then(|| Phase0OverrideEvidence {
                ttl_present: has_non_empty_string(override_packet, &["ttl_expires_at"]),
                reviewer_acknowledgment_present: has_non_empty_string(
                    override_packet,
                    &["reviewer_acknowledgment"],
                ),
                audit_chain_event_present: has_non_empty_string(
                    override_packet,
                    &["audit_chain_event"],
                ),
                owner_present: has_non_empty_string(override_packet, &["owner"]),
                blast_radius_statement_present: has_non_empty_string(
                    override_packet,
                    &["blast_radius_statement"],
                ),
                revert_or_fix_follow_up_present: has_non_empty_string(
                    override_packet,
                    &["revert_or_fix_follow_up"],
                ),
            }),
            tenant_shared_surfaces: optional_string_array_at(tenant_model, &["shared_surfaces"])
                .into_iter()
                .map(str::to_owned)
                .collect(),
            internal_bypass_without_breakglass:
                tenant_model["internal_bypass"]["allowed_without_ttl_breakglass"].as_bool()
                    == Some(true),
        }
    }

    #[test]
    fn phase0_baseline_records_cutover_evidence_and_not_completion_evidence() {
        let baseline = load_json("specs/phase0-ci-enforcement-baseline.json");

        assert_eq!(
            baseline["_meta"]["status"].as_str(),
            Some("p0_0_required_context_cutover_verified")
        );
        assert_eq!(
            baseline["claim_boundary"]["p0_0_green"].as_bool(),
            Some(false)
        );
        assert_eq!(
            baseline["claim_boundary"]["phase0_complete"].as_bool(),
            Some(false)
        );
        assert_eq!(
            baseline["gap_packet"]["overall_verdict"].as_str(),
            Some("P0.0_required_context_live_remaining_gap_rows_open")
        );

        assert_eq!(
            baseline["gap_packet"]["required_context"]["status"].as_str(),
            Some("CLOSED_WITH_EVIDENCE"),
            "required_context row must carry verified cutover evidence"
        );
        assert!(
            baseline["gap_packet"]["required_context"]["evidence"]["green_post_merge_run_id"]
                .as_str()
                .is_some_and(|run_id| !run_id.is_empty()),
            "required_context closure must cite a concrete green post-merge run"
        );

        for gap_key in [
            "trusted_producer",
            "candidate_pr_untrusted",
            "no_oya_cli_authority",
            "override_kill_switch",
            "structured_result_output",
            "tenant_pipeline_isolation",
        ] {
            assert_eq!(
                baseline["gap_packet"][gap_key]["status"].as_str(),
                Some("GAP"),
                "{gap_key} must remain explicit until it carries live evidence"
            );
        }

        let checked_in_contexts = string_array_at(
            &baseline,
            &[
                "current_state_evidence",
                "checked_in_branch_protection",
                "required_contexts",
            ],
        );
        assert!(
            checked_in_contexts.contains(&"oya-ci-required")
                && !checked_in_contexts.contains(&"oya-verify"),
            "baseline should expose the local target context without legacy oya CLI authority"
        );

        let live_contexts = string_array_at(
            &baseline,
            &[
                "current_state_evidence",
                "live_github_branch_protection",
                "required_contexts",
            ],
        );
        assert!(
            live_contexts == vec!["oya-ci-required"]
                && live_contexts
                    .iter()
                    .all(|context| phase0_context_is_required_authority(context)),
            "baseline must record exactly the single live oya-ci-required protected context"
        );
    }

    #[test]
    fn phase0_fixture_corpus_executes_red_green_policy() {
        let baseline = load_json("specs/phase0-ci-enforcement-baseline.json");
        let fixture_paths = string_array_at(&baseline, &["fixture_set", "all_fixture_paths"]);

        let mut seen_green = false;
        let mut seen_red = false;
        for fixture_path in fixture_paths {
            assert!(
                repo_root().join(fixture_path).is_file(),
                "{fixture_path} is referenced by the baseline but missing"
            );
            let fixture = load_json(fixture_path);
            let expected_verdict = fixture["expected_verdict"]
                .as_str()
                .unwrap_or_else(|| panic!("{fixture_path} lacks expected_verdict"));
            let policy_verdict =
                evaluate_phase0_ci_policy(&phase0_policy_input_for_fixture(&fixture));

            match expected_verdict {
                "GREEN" => {
                    seen_green = true;
                    assert!(
                        policy_verdict.is_green(),
                        "{fixture_path} should satisfy the P0.0 policy, got {:?}",
                        policy_verdict
                            .violations
                            .iter()
                            .map(|violation| violation.as_str())
                            .collect::<Vec<_>>()
                    );
                }
                "RED" => {
                    seen_red = true;
                    assert!(
                        !policy_verdict.is_green(),
                        "{fixture_path} should violate the P0.0 policy"
                    );
                    let observed_violations: BTreeSet<&str> = policy_verdict
                        .violations
                        .iter()
                        .map(|violation| violation.as_str())
                        .collect();
                    for expected_violation in string_array_at(&fixture, &["expected_violations"]) {
                        assert!(
                            observed_violations.contains(expected_violation),
                            "{fixture_path} expected {expected_violation}, got {observed_violations:?}"
                        );
                    }
                }
                other => panic!("{fixture_path} has unsupported expected_verdict {other}"),
            }
        }

        assert!(
            seen_green,
            "fixture corpus must include a GREEN target fixture"
        );
        assert!(
            seen_red,
            "fixture corpus must include RED negative fixtures"
        );
    }

    #[test]
    fn phase0_policy_rejects_empty_contexts_and_missing_trusted_producer() {
        let empty_context_input = Phase0CiPolicyInput {
            protected_required_contexts: vec![],
            producer_kind: Some("minimal_rust_bridge_adapter".to_owned()),
            producer_controller: Some("oya-ci-controller".to_owned()),
            producer_command: None,
            candidate_bytes_policy: Some("untrusted_input_only".to_owned()),
            gate_definition_source: Some("trusted_dev_or_controller_state".to_owned()),
            override_evidence: None,
            tenant_shared_surfaces: vec![],
            internal_bypass_without_breakglass: false,
        };
        let empty_context_verdict = evaluate_phase0_ci_policy(&empty_context_input);
        assert!(
            !empty_context_verdict.is_green()
                && empty_context_verdict
                    .violations
                    .iter()
                    .any(|violation| violation.as_str() == "missing_cloud_ci_required_context"),
            "empty required contexts must be RED"
        );

        let missing_producer_input = Phase0CiPolicyInput {
            protected_required_contexts: vec!["oya-ci-required".to_owned()],
            producer_kind: None,
            producer_controller: None,
            producer_command: None,
            candidate_bytes_policy: None,
            gate_definition_source: None,
            override_evidence: None,
            tenant_shared_surfaces: vec![],
            internal_bypass_without_breakglass: false,
        };
        let missing_producer_verdict = evaluate_phase0_ci_policy(&missing_producer_input);
        assert!(
            !missing_producer_verdict.is_green()
                && missing_producer_verdict
                    .violations
                    .iter()
                    .any(|violation| violation.as_str() == "untrusted_or_legacy_status_producer")
                && missing_producer_verdict
                    .violations
                    .iter()
                    .any(|violation| violation.as_str() == "candidate_bytes_can_weaken_gate")
                && missing_producer_verdict
                    .violations
                    .iter()
                    .any(|violation| violation.as_str() == "candidate_sourced_gate_definition"),
            "required context without trusted producer evidence must be RED"
        );
    }

    #[test]
    fn tenant_isolation_fixture_contract_executes_negative_and_target_cases() {
        let contract = load_json("specs/toolchain-tenant-isolation-fixtures.json");
        let required_surfaces = string_array_at(&contract, &["required_separation_surfaces"]);
        assert!(
            PHASE0_REQUIRED_TENANT_PIPELINE_SURFACES
                .iter()
                .all(|surface| required_surfaces.contains(surface)),
            "tenant fixture contract must enumerate every P0.0 pipeline separation surface"
        );

        let fixtures = contract["fixtures"]
            .as_array()
            .expect("tenant fixture contract should contain fixtures");
        let mut seen_green = false;
        let mut seen_red = false;

        for fixture in fixtures {
            match fixture["expected_verdict"].as_str() {
                Some("GREEN") => {
                    seen_green = true;
                    assert!(
                        string_array_at(fixture, &["expected_violations"]).is_empty(),
                        "GREEN tenant fixture must not list expected violations"
                    );
                    assert!(
                        has_non_empty_string(fixture, &["breakglass"])
                            && has_non_empty_string(fixture, &["separation_model"]),
                        "GREEN tenant fixture must specify breakglass and separation model"
                    );
                }
                Some("RED") => {
                    seen_red = true;
                    assert!(
                        !string_array_at(fixture, &["expected_violations"]).is_empty()
                            && !string_array_at(fixture, &["shared_surfaces"]).is_empty(),
                        "RED tenant fixture must expose shared-surface violations"
                    );
                    assert_eq!(
                        fixture["internal_bypass_without_breakglass"].as_bool(),
                        Some(true),
                        "RED tenant fixture must cover internal bypass without breakglass"
                    );
                }
                other => panic!("unsupported tenant fixture verdict {other:?}"),
            }
        }

        assert!(
            seen_green,
            "tenant contract must include a GREEN target fixture"
        );
        assert!(
            seen_red,
            "tenant contract must include a RED negative fixture"
        );
    }

    #[test]
    fn phase0_automation_and_claim_maps_reference_the_executable_baseline() {
        let automation_matrix = load_json("specs/phase0-automation-matrix.json");
        let claim_map = load_json("specs/phase0-claim-evidence-map.json");
        let root_hub = load_json("specs/root-hub-pointers.json");
        let result_schema = load_json("specs/phase0-ci-enforcement-result-schema.json");
        let red_result_fixture = load_json(
            "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-current-red-gap-result.json",
        );

        let automation_text = automation_matrix.to_string();
        assert!(
            automation_text.contains("specs/phase0-ci-enforcement-baseline.json")
                && automation_text.contains("specs/fixtures/phase0-ci-enforcement-baseline"),
            "automation matrix should map AC-0.0 to the baseline and fixture corpus"
        );
        let required_context_row = automation_matrix["seed_rows"]
            .as_array()
            .and_then(|rows| {
                rows.iter()
                    .find(|row| row["id"] == "AC-0.0-cloud-ci-required-context")
            })
            .expect("AC-0.0 required-context row");
        assert_eq!(
            required_context_row["target_gate_or_controller"].as_str(),
            Some("oya-ci-required branch-protection context"),
            "automation matrix should name the exact live required context, not retain an obsolete alternative"
        );
        assert!(
            automation_text
                .contains("Candidate-independent trusted gate definitions remain required")
                && automation_text.contains("F-PR5-06"),
            "automation matrix should preserve the trusted-producer and review-admission gaps after context cutover"
        );

        let claim_text = claim_map.to_string();
        assert!(
            claim_text.contains("gap-packet")
                && claim_text.contains("specs/phase0-ci-enforcement-baseline.json")
                && claim_text.contains("specs/toolchain-tenant-isolation-fixtures.json"),
            "claim map should restrict current wording to gap/target language backed by fixtures"
        );

        assert!(
            root_hub["entry_points"]["phase0_ci_enforcement_baseline"].is_object()
                && root_hub["entry_points"]["toolchain_tenant_isolation_fixtures"].is_object(),
            "root hub should expose the new P0.0 executable pointers without requiring backlog loading"
        );
        assert!(
            root_hub["entry_points"]["phase0_ci_enforcement_result_schema"].is_object(),
            "root hub should expose the P0.0 structured result schema"
        );
        assert!(
            string_array_at(&result_schema, &["required"]).contains(&"candidate_sha")
                && string_array_at(&result_schema, &["required"]).contains(&"producer")
                && string_array_at(&result_schema, &["required"]).contains(&"fixture_results")
                && string_array_at(&result_schema, &["required"]).contains(&"observed_verdict")
                && string_array_at(&result_schema, &["required"]).contains(&"provenance"),
            "result schema must require candidate, producer, fixture, verdict, and provenance fields"
        );
        assert_eq!(
            red_result_fixture["observed_verdict"].as_str(),
            Some("RED"),
            "current gap result fixture must not claim green"
        );
        assert_eq!(
            string_at(&result_schema, &["properties", "candidate_sha", "pattern"]),
            Some("^[0-9a-fA-F]{40}$"),
            "result schema must require a full candidate commit SHA"
        );
        assert!(
            red_result_fixture["candidate_sha"]
                .as_str()
                .is_some_and(is_full_hex_sha),
            "current gap result fixture should use a full synthetic hex SHA"
        );
        assert_eq!(
            red_result_fixture["claim_boundary"]["phase0_complete"].as_bool(),
            Some(false),
            "current gap result fixture must not claim Phase 0 completion"
        );

        for field in string_array_at(&result_schema, &["required"]) {
            assert!(
                !red_result_fixture[field].is_null(),
                "current red result fixture must satisfy required result-schema field {field}"
            );
        }
        assert_declared_schema_keys(
            &red_result_fixture,
            &result_schema,
            "tc-0.0-current-red-gap-result",
        );
        assert!(
            red_result_fixture["fixture_results"].is_array()
                && red_result_fixture["producer"].is_object()
                && red_result_fixture["provenance"].is_object(),
            "current red result fixture must match the declared structured-result object/array shape"
        );
    }

    #[test]
    fn phase0_aggregate_exit_gate_fixtures_prevent_vacuous_green() {
        let good =
            load_json("specs/fixtures/phase0-exit-gate/tc-0.12-good-all-subconditions-green.json");
        assert_eq!(good["expected_verdict"].as_str(), Some("GREEN"));
        assert!(
            aggregate_exit_is_green(&good["subconditions"]),
            "all-true aggregate fixture should be GREEN"
        );

        let bad = load_json(
            "specs/fixtures/phase0-exit-gate/tc-0.12-bad-single-false-subconditions.json",
        );
        let subcondition_names = string_array_at(&bad, &["subcondition_names"]);
        let mut seen_forced_false = BTreeSet::new();
        for case in object_array_at(&bad, &["cases"]) {
            let forced_false = string_at(case, &["forced_false"])
                .unwrap_or_else(|| panic!("case lacks forced_false: {case:?}"));
            seen_forced_false.insert(forced_false.to_owned());
            assert_eq!(case["expected_verdict"].as_str(), Some("RED"));
            assert!(
                !aggregate_exit_is_green(&case["subconditions"]),
                "{forced_false} false must make AC-0.12 aggregate gate RED"
            );

            let false_count = case["subconditions"]
                .as_object()
                .expect("case subconditions object")
                .values()
                .filter(|value| value.as_bool() == Some(false))
                .count();
            assert_eq!(
                false_count, 1,
                "{forced_false} case should force exactly one subcondition false"
            );
        }

        for subcondition in subcondition_names {
            assert!(
                seen_forced_false.contains(subcondition),
                "aggregate exit negative fixture must force {subcondition} false"
            );
        }

        let current_red = load_json(
            "specs/fixtures/phase0-exit-gate/tc-0.12-current-red-p0-0-live-context-missing.json",
        );
        assert_eq!(current_red["expected_verdict"].as_str(), Some("RED"));
        assert!(
            !aggregate_exit_is_green(&current_red["subconditions"]),
            "current aggregate fixture must remain RED while P0.0 live context evidence is missing"
        );
        assert_eq!(
            current_red["claim_boundary"]["phase0_complete"].as_bool(),
            Some(false),
            "current aggregate fixture must not claim Phase-0 completion"
        );
    }

    #[test]
    fn phase0_automation_matrix_rows_are_shape_checked_and_fixture_backed() {
        let automation_matrix = load_json("specs/phase0-automation-matrix.json");
        let required_fields = string_array_at(&automation_matrix, &["required_row_fields"]);
        let allowed_classifications = required_string_set(&automation_matrix, &["classifications"]);
        let rows = object_array_at(&automation_matrix, &["seed_rows"]);

        let row_violations =
            evaluate_automation_rows(&rows, &required_fields, &allowed_classifications, false);
        assert!(
            row_violations.is_empty(),
            "automation matrix seed rows should satisfy the executable row contract, got {row_violations:?}"
        );

        let row_ids: BTreeSet<_> = rows.iter().filter_map(|row| row["id"].as_str()).collect();
        for required_id in [
            "AC-0.0-cloud-ci-required-context",
            "AC-0.0-tenant-pipeline-isolation",
            "AC-0.12-aggregate-exit-gate",
            "AC-0.16-automation-ratchet",
            "AC-0.17-claim-ceiling",
            "AC-0.17-performance-budget-claim",
        ] {
            assert!(
                row_ids.contains(required_id),
                "automation matrix must carry an explicit row for {required_id}"
            );
        }

        let fixture_paths =
            string_array_at(&automation_matrix, &["fixture_set", "all_fixture_paths"]);
        assert!(
            fixture_paths
                .iter()
                .all(|fixture_path| repo_root().join(fixture_path).is_file()),
            "automation matrix fixture_set must reference only committed fixture files"
        );
        assert!(
            fixture_paths.iter().any(|path| path.contains("good-"))
                && fixture_paths
                    .iter()
                    .any(|path| path.contains("bad-oya-cli"))
                && fixture_paths
                    .iter()
                    .any(|path| path.contains("bad-missing-field")),
            "automation fixtures must cover GOOD, missing/unknown/duplicate, and oya-CLI BAD cases"
        );
    }

    #[test]
    fn phase0_automation_ratchet_fixtures_execute_red_green_cases() {
        let automation_matrix = load_json("specs/phase0-automation-matrix.json");
        let required_fields = string_array_at(&automation_matrix, &["required_row_fields"]);
        let allowed_classifications = required_string_set(&automation_matrix, &["classifications"]);
        let fixture_paths =
            string_array_at(&automation_matrix, &["fixture_set", "all_fixture_paths"]);

        let mut seen_green = false;
        let mut seen_red = false;
        for fixture_path in fixture_paths {
            let fixture = load_json(fixture_path);
            let rows = object_array_at(&fixture, &["rows"]);
            let observed_violations =
                evaluate_automation_rows(&rows, &required_fields, &allowed_classifications, true);
            let expected_violations = expected_violation_set(&fixture);

            match fixture["expected_verdict"].as_str() {
                Some("GREEN") => {
                    seen_green = true;
                    assert!(
                        observed_violations.is_empty(),
                        "{fixture_path} should be GREEN, got {observed_violations:?}"
                    );
                }
                Some("RED") => {
                    seen_red = true;
                    assert!(
                        !observed_violations.is_empty(),
                        "{fixture_path} should be RED"
                    );
                    for expected in expected_violations {
                        assert!(
                            observed_violations.contains(&expected),
                            "{fixture_path} expected violation {expected}, got {observed_violations:?}"
                        );
                    }
                }
                other => panic!("{fixture_path} has unsupported expected_verdict {other:?}"),
            }
        }

        assert!(
            seen_green && seen_red,
            "automation fixtures must include RED and GREEN cases"
        );
    }

    #[test]
    fn phase0_claim_evidence_map_rows_are_normalized_and_fixture_backed() {
        let claim_contract =
            load_json("specs/hyperscaler-production-readiness-claim-contract.json");
        let claim_map = load_json("specs/phase0-claim-evidence-map.json");
        let allowed_tiers: BTreeSet<String> = object_array_at(&claim_contract, &["claim_tiers"])
            .into_iter()
            .filter_map(|tier| tier["tier"].as_str())
            .map(str::to_owned)
            .collect();
        let regulated_vocabulary = required_string_set(&claim_map, &["regulated_vocabulary"]);

        for term in [
            "performance",
            "performant",
            "low-latency",
            "scalable",
            "capacity-ready",
        ] {
            assert!(
                regulated_vocabulary.contains(term),
                "claim map regulated vocabulary must include {term}"
            );
        }

        for row in object_array_at(&claim_map, &["seed_claim_rows"]) {
            for field in [
                "id",
                "artifact",
                "claim_text",
                "claim_tier",
                "allowed_language_now",
                "regulated_terms",
                "current_evidence",
                "missing_for_next_tier",
                "owner",
            ] {
                assert!(
                    field_is_present_and_non_empty(row, field),
                    "claim row {:?} is missing normalized field {field}",
                    row["id"].as_str()
                );
            }
            let tier = row["claim_tier"].as_str().unwrap_or_default();
            assert!(allowed_tiers.contains(tier), "unknown claim tier {tier}");
            assert!(
                row.get("current_allowed_tier").is_none()
                    && row.get("evidence_present").is_none()
                    && row.get("missing_for_higher_tier").is_none(),
                "claim row {:?} must not use stale/non-normalized keys",
                row["id"].as_str()
            );
        }

        let fixture_paths = string_array_at(&claim_map, &["fixture_set", "all_fixture_paths"]);
        assert!(
            fixture_paths
                .iter()
                .all(|fixture_path| repo_root().join(fixture_path).is_file()),
            "claim fixture_set must reference only committed fixture files"
        );
        assert!(
            fixture_paths.iter().any(|path| path.contains("good-"))
                && fixture_paths
                    .iter()
                    .any(|path| path.contains("bad-ungrounded"))
                && fixture_paths
                    .iter()
                    .any(|path| path.contains("bad-performance")),
            "claim fixtures must cover GOOD, ungrounded-claim, and performance-budget BAD cases"
        );
    }

    #[test]
    fn phase0_claim_ceiling_fixtures_execute_red_green_cases() {
        let claim_contract =
            load_json("specs/hyperscaler-production-readiness-claim-contract.json");
        let claim_map = load_json("specs/phase0-claim-evidence-map.json");
        let regulated_vocabulary = required_string_set(&claim_map, &["regulated_vocabulary"]);
        let allowed_tiers: BTreeSet<String> = object_array_at(&claim_contract, &["claim_tiers"])
            .into_iter()
            .filter_map(|tier| tier["tier"].as_str())
            .map(str::to_owned)
            .collect();
        let fixture_paths = string_array_at(&claim_map, &["fixture_set", "all_fixture_paths"]);

        let mut seen_green = false;
        let mut seen_red = false;
        for fixture_path in fixture_paths {
            let fixture = load_json(fixture_path);
            let observed_violations =
                evaluate_claim_fixture(&fixture, &regulated_vocabulary, &allowed_tiers);
            let expected_violations = expected_violation_set(&fixture);

            match fixture["expected_verdict"].as_str() {
                Some("GREEN") => {
                    seen_green = true;
                    assert!(
                        observed_violations.is_empty(),
                        "{fixture_path} should be GREEN, got {observed_violations:?}"
                    );
                }
                Some("RED") => {
                    seen_red = true;
                    assert!(
                        !observed_violations.is_empty(),
                        "{fixture_path} should be RED"
                    );
                    for expected in expected_violations {
                        assert!(
                            observed_violations.contains(&expected),
                            "{fixture_path} expected violation {expected}, got {observed_violations:?}"
                        );
                    }
                }
                other => panic!("{fixture_path} has unsupported expected_verdict {other:?}"),
            }
        }

        assert!(
            seen_green && seen_red,
            "claim fixtures must include RED and GREEN cases"
        );
    }
}
