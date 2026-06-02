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

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    /// Deterministic K8s Job name that preserves the full candidate SHA.
    ///
    /// Kubernetes Job names are RFC-1123 labels, so the decimal PR-bearing
    /// form is used only while it fits the 63-character ceiling. The fallback
    /// base-36 encodes the PR number rather than truncating the commit
    /// identity. This keeps 409 create-conflict idempotency scoped to the
    /// exact PR + candidate commit rather than to a short SHA prefix.
    pub fn job_name(&self) -> String {
        let sha = self.head_sha.to_ascii_lowercase();
        let candidate = format!("oya-ci-pr{}-{sha}", self.pr_number);
        if candidate.len() <= 63 {
            candidate
        } else {
            format!("oya-ci-pr{}-{sha}", base36_u64(self.pr_number))
        }
    }
}

fn base36_u64(mut value: u64) -> String {
    const DIGITS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    if value == 0 {
        return "0".to_owned();
    }

    let mut encoded = Vec::new();
    while value > 0 {
        encoded.push(DIGITS[(value % 36) as usize] as char);
        value /= 36;
    }
    encoded.iter().rev().collect()
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
    pub tenant_separated_surfaces: Vec<String>,
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
    TenantSurfaceSeparationIncomplete,
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
            Phase0CiPolicyViolation::TenantSurfaceSeparationIncomplete => {
                "tenant_surface_separation_incomplete"
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

fn tenant_surface_separation_is_complete(separated_surfaces: &[String]) -> bool {
    PHASE0_REQUIRED_TENANT_PIPELINE_SURFACES
        .iter()
        .all(|required| tenant_surface_is_separated(required, separated_surfaces))
}

fn tenant_surface_is_separated(required_surface: &str, separated_surfaces: &[String]) -> bool {
    separated_surfaces.iter().any(|surface| {
        let surface = surface.as_str();
        surface == required_surface
            || tenant_surface_aliases(required_surface)
                .iter()
                .any(|alias| surface == *alias)
    })
}

fn tenant_surface_aliases(required_surface: &str) -> &'static [&'static str] {
    match required_surface {
        "identity" => &["identity"],
        "secrets" => &["secret_scope", "secret_lease", "secrets"],
        "runners" => &["runner_pool", "runners"],
        "workspaces" => &["workspace_volume", "workspaces"],
        "caches" => &["cache_namespace", "caches"],
        "artifacts" => &["artifact_namespace", "artifacts"],
        "logs_evidence" => &["log_evidence_namespace", "logs_evidence"],
        "release_ledgers" => &["release_ledger", "release_ledgers"],
        "deploy_targets" => &["deploy_target", "deploy_targets"],
        "status_callbacks" => &["status_callback_identity", "status_callbacks"],
        "audit_events" => &["audit_event_stream", "audit_events"],
        _ => &[],
    }
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

    if !tenant_surface_separation_is_complete(&input.tenant_separated_surfaces) {
        violations.insert(Phase0CiPolicyViolation::TenantSurfaceSeparationIncomplete);
    }
    if !input.tenant_shared_surfaces.is_empty() {
        violations.insert(Phase0CiPolicyViolation::TenantSurfacesShared);
    }
    if input.internal_bypass_without_breakglass {
        violations.insert(Phase0CiPolicyViolation::InternalBypassWithoutBreakglass);
    }

    Phase0CiPolicyVerdict { violations }
}

/// Phase-0 automation-ratchet violation classes.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Phase0AutomationRatchetViolation {
    EmptyRegistry,
    MissingEvaluatorConfiguration,
    MissingOrEmptyRequiredField,
    DuplicateRowId,
    UnknownClassification,
    BlockingInvariantMappedToOyaCli,
    EnforceableOrAutomatableMarkedHumanJudgment,
}

impl Phase0AutomationRatchetViolation {
    pub const fn as_str(self) -> &'static str {
        match self {
            Phase0AutomationRatchetViolation::EmptyRegistry => "empty_registry",
            Phase0AutomationRatchetViolation::MissingEvaluatorConfiguration => {
                "missing_evaluator_configuration"
            }
            Phase0AutomationRatchetViolation::MissingOrEmptyRequiredField => {
                "missing_or_empty_required_field"
            }
            Phase0AutomationRatchetViolation::DuplicateRowId => "duplicate_row_id",
            Phase0AutomationRatchetViolation::UnknownClassification => "unknown_classification",
            Phase0AutomationRatchetViolation::BlockingInvariantMappedToOyaCli => {
                "blocking_invariant_mapped_to_oya_cli"
            }
            Phase0AutomationRatchetViolation::EnforceableOrAutomatableMarkedHumanJudgment => {
                "enforceable_or_automatable_marked_human_judgment"
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Phase0AutomationRatchetVerdict {
    pub violations: BTreeSet<Phase0AutomationRatchetViolation>,
}

impl Phase0AutomationRatchetVerdict {
    pub fn is_green(&self) -> bool {
        self.violations.is_empty()
    }
}

/// Evaluate AC-0.16 automation-ratchet rows from a controller/gate-owned JSON
/// registry. This is intentionally pure-domain logic: callers are responsible
/// for loading the generated registry or fixture bytes from trusted
/// trunk/controller state.
pub fn evaluate_phase0_automation_rows(
    rows: &[Value],
    required_fields: &[String],
    allowed_classifications: &BTreeSet<String>,
) -> Phase0AutomationRatchetVerdict {
    let mut violations = BTreeSet::new();
    let mut ids = BTreeSet::new();

    if rows.is_empty() {
        violations.insert(Phase0AutomationRatchetViolation::EmptyRegistry);
    }
    if required_fields.is_empty() || allowed_classifications.is_empty() {
        violations.insert(Phase0AutomationRatchetViolation::MissingEvaluatorConfiguration);
    }

    for row in rows {
        for required_field in required_fields {
            if !json_field_is_present_and_non_empty(row, required_field) {
                violations.insert(Phase0AutomationRatchetViolation::MissingOrEmptyRequiredField);
            }
        }

        if let Some(id) = row["id"].as_str() {
            if !ids.insert(id.to_owned()) {
                violations.insert(Phase0AutomationRatchetViolation::DuplicateRowId);
            }
        }

        let classification = row["classification"].as_str().unwrap_or_default();
        if !allowed_classifications.contains(classification) {
            violations.insert(Phase0AutomationRatchetViolation::UnknownClassification);
        }

        if row["no_new_oya_cli_surface"].as_bool() != Some(true)
            || row["target_gate_or_controller"]
                .as_str()
                .is_some_and(contains_oya_cli_authority)
        {
            violations.insert(Phase0AutomationRatchetViolation::BlockingInvariantMappedToOyaCli);
        }

        if classification == "not_automatable_human_judgment"
            && row["enforceable_or_automatable"].as_bool() == Some(true)
        {
            violations.insert(
                Phase0AutomationRatchetViolation::EnforceableOrAutomatableMarkedHumanJudgment,
            );
        }
    }

    Phase0AutomationRatchetVerdict { violations }
}

/// Phase-0 claim-ceiling violation classes.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Phase0ClaimCeilingViolation {
    MissingOrMalformedFixture,
    EmptyRegulatedVocabulary,
    EmptyAllowedClaimTiers,
    MissingOrMalformedClaimRows,
    MissingOrMalformedRegulatedTerms,
    RegulatedVocabularyWithoutClaimRow,
    MissingOrEmptyClaimRowField,
    UnknownClaimTier,
    ForbiddenLocalOrOyaEvidenceForMechanicalClaim,
    ProductionReadinessClaimWithoutTypedEvidence,
    PerformanceClaimWithoutBudgetOrMeasuredResult,
    UnknownRegulatedTerm,
}

impl Phase0ClaimCeilingViolation {
    pub const fn as_str(self) -> &'static str {
        match self {
            Phase0ClaimCeilingViolation::MissingOrMalformedFixture => {
                "missing_or_malformed_fixture"
            }
            Phase0ClaimCeilingViolation::EmptyRegulatedVocabulary => "empty_regulated_vocabulary",
            Phase0ClaimCeilingViolation::EmptyAllowedClaimTiers => "empty_allowed_claim_tiers",
            Phase0ClaimCeilingViolation::MissingOrMalformedClaimRows => {
                "missing_or_malformed_claim_rows"
            }
            Phase0ClaimCeilingViolation::MissingOrMalformedRegulatedTerms => {
                "missing_or_malformed_regulated_terms"
            }
            Phase0ClaimCeilingViolation::RegulatedVocabularyWithoutClaimRow => {
                "regulated_vocabulary_without_claim_row"
            }
            Phase0ClaimCeilingViolation::MissingOrEmptyClaimRowField => {
                "missing_or_empty_claim_row_field"
            }
            Phase0ClaimCeilingViolation::UnknownClaimTier => "unknown_claim_tier",
            Phase0ClaimCeilingViolation::ForbiddenLocalOrOyaEvidenceForMechanicalClaim => {
                "forbidden_local_or_oya_evidence_for_mechanical_claim"
            }
            Phase0ClaimCeilingViolation::ProductionReadinessClaimWithoutTypedEvidence => {
                "production_readiness_claim_without_typed_evidence"
            }
            Phase0ClaimCeilingViolation::PerformanceClaimWithoutBudgetOrMeasuredResult => {
                "performance_claim_without_budget_or_measured_result"
            }
            Phase0ClaimCeilingViolation::UnknownRegulatedTerm => "unknown_regulated_term",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Phase0ClaimCeilingVerdict {
    pub violations: BTreeSet<Phase0ClaimCeilingViolation>,
}

impl Phase0ClaimCeilingVerdict {
    pub fn is_green(&self) -> bool {
        self.violations.is_empty()
    }
}

/// Evaluate AC-0.17 claim-ceiling fixtures or generated claim rows. The caller
/// supplies the regulated vocabulary and allowed tiers from trusted
/// controller/trunk-owned specs.
pub fn evaluate_phase0_claim_ceiling(
    fixture: &Value,
    regulated_vocabulary: &BTreeSet<String>,
    allowed_tiers: &BTreeSet<String>,
) -> Phase0ClaimCeilingVerdict {
    let mut violations = BTreeSet::new();
    if !fixture.is_object() {
        violations.insert(Phase0ClaimCeilingViolation::MissingOrMalformedFixture);
    }
    if regulated_vocabulary.is_empty() {
        violations.insert(Phase0ClaimCeilingViolation::EmptyRegulatedVocabulary);
    }
    if allowed_tiers.is_empty() {
        violations.insert(Phase0ClaimCeilingViolation::EmptyAllowedClaimTiers);
    }

    let text = fixture["text"].as_str().unwrap_or_default();
    let text_present = !text.trim().is_empty();
    let observed_terms = regulated_terms_in_text(text, regulated_vocabulary);
    let rows = match fixture.get("claim_rows").and_then(Value::as_array) {
        Some(rows) => rows.iter().collect::<Vec<_>>(),
        None => {
            violations.insert(Phase0ClaimCeilingViolation::MissingOrMalformedClaimRows);
            Vec::new()
        }
    };

    if !text_present && rows.is_empty() {
        violations.insert(Phase0ClaimCeilingViolation::MissingOrMalformedFixture);
    }

    if !observed_terms.is_empty() && rows.is_empty() {
        violations.insert(Phase0ClaimCeilingViolation::RegulatedVocabularyWithoutClaimRow);
    }

    let mut covered_terms = BTreeSet::new();
    for row in rows {
        if !row.is_object() {
            violations.insert(Phase0ClaimCeilingViolation::MissingOrMalformedClaimRows);
            continue;
        }

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
            if !json_field_is_present_and_non_empty(row, field) {
                violations.insert(Phase0ClaimCeilingViolation::MissingOrEmptyClaimRowField);
            }
        }

        let tier = row["claim_tier"].as_str().unwrap_or_default();
        if !allowed_tiers.contains(tier) {
            violations.insert(Phase0ClaimCeilingViolation::UnknownClaimTier);
        }

        let terms = match claim_row_terms(row) {
            Ok(terms) => terms,
            Err(violation) => {
                violations.insert(violation);
                BTreeSet::new()
            }
        };
        for term in &terms {
            if !regulated_vocabulary.contains(term) {
                violations.insert(Phase0ClaimCeilingViolation::UnknownRegulatedTerm);
            }
        }
        covered_terms.extend(terms.iter().cloned());
        let evidence = claim_row_evidence(row);
        if tier == "mechanically_enforced"
            && (contains_oya_cli_authority(&evidence) || contains_oya_cli_authority(text))
        {
            violations
                .insert(Phase0ClaimCeilingViolation::ForbiddenLocalOrOyaEvidenceForMechanicalClaim);
        }
        if matches!(tier, "production_ready" | "hyperscaler_grade")
            && !claim_row_has_typed_readiness_evidence(row)
        {
            violations
                .insert(Phase0ClaimCeilingViolation::ProductionReadinessClaimWithoutTypedEvidence);
        }

        let performance_claim = terms.iter().any(|term| {
            matches!(
                term.as_str(),
                "performance" | "performant" | "low-latency" | "capacity" | "capacity-ready"
            )
        });
        if performance_claim
            && matches!(tier, "production_ready" | "hyperscaler_grade")
            && !claim_row_has_typed_performance_evidence(row)
        {
            violations
                .insert(Phase0ClaimCeilingViolation::PerformanceClaimWithoutBudgetOrMeasuredResult);
        }
    }

    for observed_term in observed_terms {
        if !covered_terms.contains(&observed_term) {
            violations.insert(Phase0ClaimCeilingViolation::RegulatedVocabularyWithoutClaimRow);
        }
    }

    Phase0ClaimCeilingVerdict { violations }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Phase0AggregateExitVerdict {
    pub false_or_missing_subconditions: BTreeSet<String>,
}

impl Phase0AggregateExitVerdict {
    pub fn is_green(&self) -> bool {
        self.false_or_missing_subconditions.is_empty()
    }
}

/// Evaluate the AC-0.12 aggregate exit shape. The aggregate is green only when
/// the trusted subcondition object is non-empty and every subcondition is
/// exactly boolean `true`.
pub fn evaluate_phase0_aggregate_exit(subconditions: &Value) -> Phase0AggregateExitVerdict {
    let mut false_or_missing_subconditions = BTreeSet::new();
    let Some(conditions) = subconditions.as_object() else {
        false_or_missing_subconditions.insert("<subconditions-object>".to_owned());
        return Phase0AggregateExitVerdict {
            false_or_missing_subconditions,
        };
    };

    if conditions.is_empty() {
        false_or_missing_subconditions.insert("<non-empty-subconditions>".to_owned());
    }

    for (name, value) in conditions {
        if value.as_bool() != Some(true) {
            false_or_missing_subconditions.insert(name.clone());
        }
    }

    Phase0AggregateExitVerdict {
        false_or_missing_subconditions,
    }
}

pub fn contains_oya_cli_authority(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    if lower.contains("oya --")
        || lower.contains("`oya`")
        || lower.contains("oya cli")
        || lower.contains("legacy oya cli invocation")
        || contains_oya_executable_path(&lower)
    {
        return true;
    }

    let normalized: String = lower
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { ' ' })
        .collect();
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    tokens.windows(2).any(|window| {
        window[0] == "oya" && matches!(window[1], "gate" | "verify" | "vcs" | "git" | "cli")
    })
}

fn contains_oya_executable_path(lower: &str) -> bool {
    ["./oya", "./bin/oya", "/bin/oya"]
        .into_iter()
        .any(|needle| {
            lower.match_indices(needle).any(|(index, _)| {
                lower[index + needle.len()..]
                    .chars()
                    .next()
                    .is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '-' && ch != '_')
            })
        })
}

fn json_field_is_present_and_non_empty(row: &Value, field: &str) -> bool {
    match &row[field] {
        Value::String(value) => !value.trim().is_empty(),
        Value::Bool(_) => true,
        Value::Array(values) => !values.is_empty(),
        Value::Object(values) => !values.is_empty(),
        _ => false,
    }
}

fn regulated_terms_in_text(text: &str, vocabulary: &BTreeSet<String>) -> BTreeSet<String> {
    let lower = text.to_ascii_lowercase();
    vocabulary
        .iter()
        .filter(|term| lower.contains(&term.to_ascii_lowercase()))
        .cloned()
        .collect()
}

fn claim_row_terms(
    row: &Value,
) -> std::result::Result<BTreeSet<String>, Phase0ClaimCeilingViolation> {
    let Some(values) = row["regulated_terms"].as_array() else {
        return Err(Phase0ClaimCeilingViolation::MissingOrMalformedRegulatedTerms);
    };
    if values.is_empty() {
        return Err(Phase0ClaimCeilingViolation::MissingOrMalformedRegulatedTerms);
    }

    let mut terms = BTreeSet::new();
    for value in values {
        let Some(term) = value
            .as_str()
            .map(str::trim)
            .filter(|term| !term.is_empty())
        else {
            return Err(Phase0ClaimCeilingViolation::MissingOrMalformedRegulatedTerms);
        };
        terms.insert(term.to_owned());
    }
    Ok(terms)
}

fn claim_row_evidence(row: &Value) -> String {
    row["current_evidence"]
        .as_array()
        .map(|values| {
            values
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .map(str::to_owned)
                        .unwrap_or_else(|| value.to_string())
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default()
}

fn claim_row_has_typed_performance_evidence(row: &Value) -> bool {
    row["current_evidence"]
        .as_array()
        .is_some_and(|items| items.iter().any(is_typed_performance_evidence))
}

fn claim_row_has_typed_readiness_evidence(row: &Value) -> bool {
    row["current_evidence"]
        .as_array()
        .is_some_and(|items| items.iter().any(is_typed_readiness_evidence))
}

fn is_typed_readiness_evidence(value: &Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    let contract_result = object
        .get("claim_contract_result")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let evidence_domains = object
        .get("evidence_domains")
        .and_then(Value::as_array)
        .is_some_and(|domains| {
            !domains.is_empty()
                && domains.iter().all(|domain| {
                    domain
                        .as_str()
                        .is_some_and(|value| !value.trim().is_empty())
                })
        });
    let required_evidence =
        object
            .get("required_evidence")
            .is_some_and(|required| match required {
                Value::Array(values) => !values.is_empty(),
                Value::Object(values) => !values.is_empty(),
                _ => false,
            });
    let provenance = object
        .get("provenance")
        .is_some_and(|provenance| match provenance {
            Value::Object(values) => !values.is_empty(),
            _ => false,
        });

    contract_result && evidence_domains && required_evidence && provenance
}

fn is_typed_performance_evidence(value: &Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    let measured = object
        .get("measured_result")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let latency_budget = ["p50_ms", "p95_ms", "p99_ms"].into_iter().all(|field| {
        object
            .get(field)
            .and_then(Value::as_f64)
            .is_some_and(|v| v >= 0.0)
    });
    let has_load_profile =
        object
            .get("load_profile")
            .is_some_and(|load_profile| match load_profile {
                Value::String(value) => !value.trim().is_empty(),
                Value::Object(values) => !values.is_empty(),
                _ => false,
            });

    measured && latency_budget && has_load_profile
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
    PostPending { description: String },
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

// The protected required context for the P0.0 cloud-ci/oya-ci target. Legacy
// `oya-ci-gate` can remain bridge feedback only; it is not merge or Phase-0
// exit authority.
pub const GATE_CONTEXT: &str = "oya-ci-required";

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
            description: "oya-ci-required: run disappeared (job deleted before verdict posted)"
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
        || obs
            .conditions
            .iter()
            .any(|c| c.status && matches!(c.condition_type, JobConditionType::Complete));

    if is_complete {
        return ReconcileDecision::PostTerminal {
            state: ForgejoState::Success,
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
                    description: "oya-ci-required failed: OOMKilled — raise Job memory limit"
                        .to_owned(),
                };
            }
            PodReason::Evicted => {
                return ReconcileDecision::PostTerminal {
                    state: ForgejoState::Failure,
                    context: GATE_CONTEXT,
                    description:
                        "oya-ci-required failed: pod evicted (node-pressure or preemption)"
                            .to_owned(),
                };
            }
            // InvalidImageName is a misconfiguration — not transient, terminal immediately.
            PodReason::InvalidImageName => {
                return ReconcileDecision::PostTerminal {
                    state: ForgejoState::Error,
                    context: GATE_CONTEXT,
                    description:
                        "oya-ci-required error: InvalidImageName — operator must fix gate image config"
                            .to_owned(),
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
                        description: format!("oya-ci-required failed: {label}"),
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
            description: "oya-ci-required: running trusted required gate target".to_owned(),
        };
    }

    // Fallback: failed count > 0 but no condition yet — treat as gate failure.
    ReconcileDecision::PostTerminal {
        state: ForgejoState::Failure,
        context: GATE_CONTEXT,
        description: "oya-ci-required failed: required gate exited non-zero".to_owned(),
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
///
/// Async: the controller is fully async (tokio/kube/axum); the adapter uses
/// async `reqwest::Client`. `#[async_trait]` keeps the seam `dyn`-compatible.
#[async_trait::async_trait]
pub trait ForgejoStatusPoster: Send + Sync {
    /// POST a status to `POST /api/v1/repos/<owner>/<repo>/statuses/<sha>`.
    /// Returns `Err(KernelError::DownstreamTransport)` on non-2xx or transport error.
    async fn post(
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
            ReconcileDecision::PostTerminal { state, context, .. } => {
                assert_eq!(state, ForgejoState::Success);
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
            ReconcileDecision::PostTerminal {
                state, description, ..
            } => {
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
            ReconcileDecision::PostTerminal {
                state, description, ..
            } => {
                assert_eq!(state, ForgejoState::Failure);
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
            ReconcileDecision::PostTerminal {
                state, description, ..
            } => {
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
            ReconcileDecision::PostTerminal {
                state, description, ..
            } => {
                assert_eq!(state, ForgejoState::Error);
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
            ReconcileDecision::PostTerminal {
                state, description, ..
            } => {
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
        assert_eq!(
            map_job_to_status(&obs, 12),
            ReconcileDecision::AlreadyTerminal
        );
    }

    // ---- Already terminal idempotency guard --------------------------------

    #[test]
    fn terminal_already_posted_returns_already_terminal() {
        let obs = JobObservation {
            terminal_status_already_posted: Some(ForgejoState::Success),
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
            terminal_status_already_posted: Some(ForgejoState::Failure),
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
        assert!(
            !s.contains("do-not-log-me"),
            "debug must not leak token: {s}"
        );
    }

    // ---- GateRun job_name --------------------------------------------------

    #[test]
    fn gate_run_job_name_is_deterministic() {
        let run = GateRun {
            pr_number: 42,
            head_sha: "abcdef1234567890abcdef1234567890abcdef12".to_owned(),
            delivery_id: "d1".to_owned(),
            base_ref: "dev".to_owned(),
            repo: "oya-admin/oyatie".to_owned(),
        };
        assert_eq!(
            run.job_name(),
            "oya-ci-pr42-abcdef1234567890abcdef1234567890abcdef12"
        );
        assert!(run.job_name().len() <= 63);
    }

    #[test]
    fn gate_run_job_name_preserves_full_sha_identity() {
        let first = GateRun {
            pr_number: 42,
            head_sha: "abcdef1200000000000000000000000000000000".to_owned(),
            delivery_id: "d1".to_owned(),
            base_ref: "dev".to_owned(),
            repo: "oya-admin/oyatie".to_owned(),
        };
        let second = GateRun {
            pr_number: 42,
            head_sha: "abcdef12ffffffffffffffffffffffffffffffff".to_owned(),
            delivery_id: "d2".to_owned(),
            base_ref: "dev".to_owned(),
            repo: "oya-admin/oyatie".to_owned(),
        };

        assert_ne!(first.job_name(), second.job_name());
        assert!(first.job_name().contains(&first.head_sha));
        assert!(second.job_name().contains(&second.head_sha));
    }

    #[test]
    fn gate_run_job_name_fits_kubernetes_label_for_large_pr_numbers() {
        let run = GateRun {
            pr_number: u64::MAX,
            head_sha: "abcdef1234567890abcdef1234567890abcdef12".to_owned(),
            delivery_id: "d1".to_owned(),
            base_ref: "dev".to_owned(),
            repo: "oya-admin/oyatie".to_owned(),
        };

        let job_name = run.job_name();
        assert_eq!(
            job_name,
            "oya-ci-pr3w5e11264sgsf-abcdef1234567890abcdef1234567890abcdef12"
        );
        assert!(job_name.len() <= 63);
        assert!(job_name.ends_with(&run.head_sha));
    }

    #[test]
    fn gate_context_is_phase0_required_context() {
        assert_eq!(GATE_CONTEXT, "oya-ci-required");
        assert!(phase0_context_is_required_authority(GATE_CONTEXT));
    }
}

#[cfg(test)]
mod phase0_ci_enforcement_baseline_tests {
    use std::{collections::BTreeSet, fs, path::PathBuf};

    use serde_json::{Value, json};

    use super::{
        PHASE0_REQUIRED_TENANT_PIPELINE_SURFACES, Phase0CiPolicyInput, Phase0OverrideEvidence,
        contains_oya_cli_authority, evaluate_phase0_aggregate_exit,
        evaluate_phase0_automation_rows, evaluate_phase0_ci_policy, evaluate_phase0_claim_ceiling,
        phase0_context_is_required_authority, tenant_surface_separation_is_complete,
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
    ) -> BTreeSet<String> {
        let owned_rows: Vec<Value> = rows.iter().map(|row| (*row).clone()).collect();
        let owned_required_fields: Vec<String> = required_fields
            .iter()
            .map(|field| (*field).to_owned())
            .collect();
        evaluate_phase0_automation_rows(
            &owned_rows,
            &owned_required_fields,
            allowed_classifications,
        )
        .violations
        .into_iter()
        .map(|violation| violation.as_str().to_owned())
        .collect()
    }

    fn evaluate_claim_fixture(
        fixture: &Value,
        regulated_vocabulary: &BTreeSet<String>,
        allowed_tiers: &BTreeSet<String>,
    ) -> BTreeSet<String> {
        evaluate_phase0_claim_ceiling(fixture, regulated_vocabulary, allowed_tiers)
            .violations
            .into_iter()
            .map(|violation| violation.as_str().to_owned())
            .collect()
    }

    fn aggregate_exit_is_green(subconditions: &Value) -> bool {
        evaluate_phase0_aggregate_exit(subconditions).is_green()
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
            tenant_separated_surfaces: optional_string_array_at(
                tenant_model,
                &["separate_surfaces"],
            )
            .into_iter()
            .chain(optional_string_array_at(
                tenant_model,
                &["partitioned_surfaces"],
            ))
            .map(str::to_owned)
            .collect(),
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
    fn buck2_authority_policy_is_registered_and_forbids_legacy_cargo_contexts() {
        let root_hub = load_json("specs/root-hub-pointers.json");
        assert_eq!(
            root_hub["entry_points"]["buck2_authority_policy"]["current_path"].as_str(),
            Some("/specs/buck2-authority-policy.json")
        );

        let automation = load_json("specs/phase0-automation-matrix.json");
        let seed_ids: BTreeSet<&str> = object_array_at(&automation, &["seed_rows"])
            .into_iter()
            .filter_map(|row| row["id"].as_str())
            .collect();
        assert!(seed_ids.contains("AC-0.0-buck2-authority-no-cargo-regression"));

        let policy = load_json("specs/buck2-authority-policy.json");
        assert_eq!(
            string_at(&policy, &["target_authority", "required_context"]),
            Some("oya-ci-required")
        );
        assert_eq!(
            policy["claim_boundary"]["phase0_complete"].as_bool(),
            Some(false)
        );

        let forbidden_contexts: BTreeSet<&str> =
            string_array_at(&policy, &["forbidden_status_contexts"])
                .into_iter()
                .collect();
        for context in [
            "cargo-fmt",
            "cargo-check",
            "cargo-clippy",
            "cargo-nextest",
            "cargo-deny",
            "oya-verify",
        ] {
            assert!(
                forbidden_contexts.contains(context),
                "{context} must remain a forbidden protected-branch context"
            );
        }

        let exceptions: BTreeSet<&str> = object_array_at(&policy, &["allowed_cargo_exceptions"])
            .into_iter()
            .filter_map(|row| row["id"].as_str())
            .collect();
        assert!(exceptions.contains("production-release-image-binary-optimization"));
        assert!(exceptions.contains("buck2-graph-metadata-only"));

        let automated_chain = string_array_at(&policy, &["automated_chain"]);
        assert!(
            automated_chain
                .iter()
                .any(|entry| entry.contains("//:buck2-authority-policy-check"))
        );
    }

    #[test]
    fn phase0_baseline_is_red_gap_packet_and_not_completion_evidence() {
        let baseline = load_json("specs/phase0-ci-enforcement-baseline.json");

        assert_eq!(
            baseline["_meta"]["status"].as_str(),
            Some("p0_0_red_gap_packet")
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
            Some("P0.0_RED_blocked_until_cloud_ci_required_context_is_live")
        );

        for gap_key in [
            "required_context",
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
                "{gap_key} must remain explicit until a trusted cloud-ci context is live"
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
            !live_contexts
                .iter()
                .any(|context| phase0_context_is_required_authority(context)),
            "baseline must not claim live cloud-ci/oya-ci required status before external protection is changed"
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
    fn phase0_policy_rejects_missing_tenant_surface_separation() {
        let partial_tenant_input = Phase0CiPolicyInput {
            protected_required_contexts: vec!["oya-ci-required".to_owned()],
            producer_kind: Some("minimal_rust_bridge_adapter".to_owned()),
            producer_controller: Some("oya-ci-controller".to_owned()),
            producer_command: None,
            candidate_bytes_policy: Some("untrusted_input_only".to_owned()),
            gate_definition_source: Some("trusted_dev_or_controller_state".to_owned()),
            override_evidence: None,
            tenant_separated_surfaces: vec!["identity".to_owned(), "secret_scope".to_owned()],
            tenant_shared_surfaces: vec![],
            internal_bypass_without_breakglass: false,
        };

        let verdict = evaluate_phase0_ci_policy(&partial_tenant_input);
        assert!(
            !verdict.is_green()
                && verdict
                    .violations
                    .iter()
                    .any(|violation| violation.as_str() == "tenant_surface_separation_incomplete"),
            "partial tenant separation evidence must be RED"
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
            tenant_separated_surfaces: PHASE0_REQUIRED_TENANT_PIPELINE_SURFACES
                .iter()
                .map(|surface| (*surface).to_owned())
                .collect(),
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
            tenant_separated_surfaces: PHASE0_REQUIRED_TENANT_PIPELINE_SURFACES
                .iter()
                .map(|surface| (*surface).to_owned())
                .collect(),
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
                    let separated_surfaces: Vec<String> =
                        optional_string_array_at(fixture, &["separate_surfaces"])
                            .into_iter()
                            .map(str::to_owned)
                            .collect();
                    assert!(
                        tenant_surface_separation_is_complete(&separated_surfaces),
                        "GREEN tenant fixture must enumerate every separated P0.0 surface"
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
        assert!(
            automation_text.contains("cloud-ci-required")
                && automation_text.contains("oya-ci-required")
                && automation_text.contains("oya verify"),
            "automation matrix should preserve current gap and target-context evidence"
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
            evaluate_automation_rows(&rows, &required_fields, &allowed_classifications);
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
                evaluate_automation_rows(&rows, &required_fields, &allowed_classifications);
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
    fn phase0_automation_ratchet_public_api_fails_closed_on_empty_input() {
        let verdict = evaluate_phase0_automation_rows(&[], &[], &BTreeSet::<String>::new());
        let violations: BTreeSet<&str> = verdict
            .violations
            .iter()
            .map(|violation| violation.as_str())
            .collect();

        assert!(violations.contains("empty_registry"));
        assert!(violations.contains("missing_evaluator_configuration"));
        assert!(
            !verdict.is_green(),
            "empty or unconfigured automation input must not false-green AC-0.16"
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
            "capacity",
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
            for term in string_array_at(row, &["regulated_terms"]) {
                assert!(
                    regulated_vocabulary.contains(term),
                    "claim row {:?} uses unregistered regulated term {term}",
                    row["id"].as_str()
                );
            }
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
                    .any(|path| path.contains("bad-performance"))
                && fixture_paths
                    .iter()
                    .any(|path| path.contains("bad-unknown-regulated-term")),
            "claim fixtures must cover GOOD, ungrounded-claim, performance-budget BAD, and unknown-term BAD cases"
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

    #[test]
    fn phase0_claim_ceiling_public_api_fails_closed_on_empty_and_malformed_inputs() {
        let empty_verdict =
            evaluate_phase0_claim_ceiling(&json!({}), &BTreeSet::new(), &BTreeSet::new());
        let empty_violations: BTreeSet<&str> = empty_verdict
            .violations
            .iter()
            .map(|violation| violation.as_str())
            .collect();
        for expected in [
            "missing_or_malformed_fixture",
            "empty_regulated_vocabulary",
            "empty_allowed_claim_tiers",
            "missing_or_malformed_claim_rows",
        ] {
            assert!(
                empty_violations.contains(expected),
                "empty fixture should fail closed with {expected}, got {empty_violations:?}"
            );
        }

        let regulated_vocabulary: BTreeSet<String> = [
            "performance",
            "production-ready",
            "mechanically enforced",
            "green",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        let allowed_tiers: BTreeSet<String> = ["production_ready", "mechanically_enforced"]
            .into_iter()
            .map(str::to_owned)
            .collect();

        let malformed_terms = json!({
            "text": "The gate is production-ready and performance-verified.",
            "claim_rows": [{
                "id": "BAD-malformed-regulated-terms",
                "artifact": "inline-test",
                "claim_text": "production-ready performance",
                "claim_tier": "production_ready",
                "allowed_language_now": "not allowed",
                "regulated_terms": ["performance", 42],
                "current_evidence": [{
                    "measured_result": true,
                    "p50_ms": 1.0,
                    "p95_ms": 2.0,
                    "p99_ms": 3.0,
                    "load_profile": "synthetic"
                }],
                "owner": "platform-sre"
            }]
        });
        let malformed_violations: BTreeSet<&str> =
            evaluate_phase0_claim_ceiling(&malformed_terms, &regulated_vocabulary, &allowed_tiers)
                .violations
                .iter()
                .map(|violation| violation.as_str())
                .collect();
        assert!(
            malformed_violations.contains("missing_or_malformed_regulated_terms"),
            "non-string regulated_terms entries must fail closed, got {malformed_violations:?}"
        );

        let unknown_term = json!({
            "text": "The cloud-ci gate is magic-fast.",
            "claim_rows": [{
                "id": "BAD-unknown-regulated-term",
                "artifact": "inline-test",
                "claim_text": "magic-fast",
                "claim_tier": "production_ready",
                "allowed_language_now": "not allowed",
                "regulated_terms": ["magic-fast"],
                "current_evidence": [{
                    "claim_contract_result": true,
                    "evidence_domains": ["CI-MERGE"],
                    "required_evidence": {
                        "required_context": "inline-test"
                    },
                    "provenance": {
                        "source": "inline-test"
                    }
                }],
                "owner": "platform-sre"
            }]
        });
        let unknown_violations: BTreeSet<&str> =
            evaluate_phase0_claim_ceiling(&unknown_term, &regulated_vocabulary, &allowed_tiers)
                .violations
                .iter()
                .map(|violation| violation.as_str())
                .collect();
        assert!(
            unknown_violations.contains("unknown_regulated_term"),
            "claim rows must not introduce unregistered regulated vocabulary, got {unknown_violations:?}"
        );
    }

    #[test]
    fn phase0_claim_ceiling_requires_typed_performance_evidence_for_readiness_claims() {
        let regulated_vocabulary: BTreeSet<String> =
            ["performance", "production-ready", "low-latency"]
                .into_iter()
                .map(str::to_owned)
                .collect();
        let allowed_tiers: BTreeSet<String> = ["production_ready"]
            .into_iter()
            .map(str::to_owned)
            .collect();

        let string_only = json!({
            "text": "The cloud-ci gate is low-latency and production-ready.",
            "claim_rows": [{
                "id": "BAD-string-only-performance-evidence",
                "artifact": "inline-test",
                "claim_text": "low-latency production-ready performance",
                "claim_tier": "production_ready",
                "allowed_language_now": "not allowed",
                "regulated_terms": ["performance", "production-ready", "low-latency"],
                "current_evidence": [
                    "contains p50 p95 p99 load measured_result words, but no typed result"
                ],
                "owner": "platform-sre"
            }]
        });
        let string_only_violations: BTreeSet<&str> =
            evaluate_phase0_claim_ceiling(&string_only, &regulated_vocabulary, &allowed_tiers)
                .violations
                .iter()
                .map(|violation| violation.as_str())
                .collect();
        assert!(
            string_only_violations.contains("performance_claim_without_budget_or_measured_result")
        );

        let typed_evidence = json!({
            "text": "The cloud-ci gate is low-latency and production-ready.",
            "claim_rows": [{
                "id": "GOOD-typed-performance-evidence",
                "artifact": "inline-test",
                "claim_text": "low-latency production-ready performance",
                "claim_tier": "production_ready",
                "allowed_language_now": "allowed only with measured typed evidence",
                "regulated_terms": ["performance", "production-ready", "low-latency"],
                "current_evidence": [{
                    "claim_contract_result": true,
                    "evidence_domains": ["PERF-CAPACITY", "CI-MERGE"],
                    "required_evidence": {
                        "performance_budget_baseline": "inline-test",
                        "p95_p99_latency_by_operation": "inline-test"
                    },
                    "provenance": {
                        "source": "inline-test"
                    },
                    "measured_result": true,
                    "p50_ms": 1.0,
                    "p95_ms": 2.0,
                    "p99_ms": 3.0,
                    "load_profile": {
                        "requests_per_second": 10,
                        "duration_seconds": 60
                    }
                }],
                "owner": "platform-sre"
            }]
        });
        let typed_violations: BTreeSet<&str> =
            evaluate_phase0_claim_ceiling(&typed_evidence, &regulated_vocabulary, &allowed_tiers)
                .violations
                .iter()
                .map(|violation| violation.as_str())
                .collect();
        assert!(
            !typed_violations.contains("performance_claim_without_budget_or_measured_result"),
            "typed measured performance evidence should satisfy the performance-budget branch, got {typed_violations:?}"
        );
        assert!(
            typed_violations.is_empty(),
            "typed readiness + performance evidence should satisfy the synthetic production-ready claim row, got {typed_violations:?}"
        );
    }

    #[test]
    fn phase0_claim_ceiling_rejects_string_only_non_performance_readiness_claims() {
        let regulated_vocabulary: BTreeSet<String> =
            ["secure", "tenant-facing", "production-ready"]
                .into_iter()
                .map(str::to_owned)
                .collect();
        let allowed_tiers: BTreeSet<String> = ["production_ready"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        let fixture = json!({
            "text": "The cloud-ci gate is secure, tenant-facing, and production-ready.",
            "claim_rows": [{
                "id": "BAD-string-only-non-performance-readiness",
                "artifact": "inline-test",
                "claim_text": "secure tenant-facing production-ready",
                "claim_tier": "production_ready",
                "allowed_language_now": "not allowed",
                "regulated_terms": ["secure", "tenant-facing", "production-ready"],
                "current_evidence": [
                    "operator note says this looks ready"
                ],
                "owner": "platform-sre"
            }]
        });
        let violations: BTreeSet<&str> =
            evaluate_phase0_claim_ceiling(&fixture, &regulated_vocabulary, &allowed_tiers)
                .violations
                .iter()
                .map(|violation| violation.as_str())
                .collect();

        assert!(
            violations.contains("production_readiness_claim_without_typed_evidence"),
            "non-performance readiness claims also require typed claim-contract evidence, got {violations:?}"
        );
    }

    #[test]
    fn phase0_oya_cli_authority_detection_covers_punctuation_and_paths() {
        for forbidden in [
            "oya CLI required status",
            "./bin/oya verify --ci-required",
            "oya-gate run-all",
            "oya vcs promote",
            "oya git done",
            "legacy oya CLI invocation",
        ] {
            assert!(
                contains_oya_cli_authority(forbidden),
                "{forbidden:?} should be classified as oya CLI authority"
            );
        }

        for allowed in [
            "oya-ci-required",
            "oya-ci-controller",
            "oya-admin/oyatie",
            "cloud-ci/oya-ci required context",
        ] {
            assert!(
                !contains_oya_cli_authority(allowed),
                "{allowed:?} should remain valid cloud-ci/oya-ci service wording"
            );
        }
    }
}
