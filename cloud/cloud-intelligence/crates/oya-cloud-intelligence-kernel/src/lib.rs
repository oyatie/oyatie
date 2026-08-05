//! cloud-intelligence kernel — pure-Rust contracts for the OAuth subscription pool
//! (ADR-0384 Path B).
//!
//! Kernel scope:
//! - Identity value objects ([`TenantId`], [`AgentId`], [`SeatId`],
//!   [`SubscriptionId`], [`Provider`]).
//! - [`OAuthSubscription`] + [`SubscriptionState`] state machine
//!   (Authorized -> ActiveUntilExpiry -> RefreshingToken ->
//!   Active | Cooldown | Blacklisted).
//! - [`SubscriptionPool`] + [`SelectionStrategy`] (RoundRobin, FillFirst,
//!   TimeNormalizedQuotaPercent).
//! - [`SeatLease`] — RAII guard preventing same-seat double-allocation.
//! - [`AuthzGate`] trait — kernel-level seam for the Cedar adapter
//!   (D7 per-tenant forbid-wins isolation).
//! - [`EventSink`] trait + [`LlmGatewayEvent`] — D6 event-emission contract;
//!   ClickHouse + Valkey Stream impls live in separate adapter crates.
//!
//! Kernel does NOT take direct dependencies on cedar-policy, valkey, clickhouse,
//! reqwest, or OpenBao. Those belong to adapter crates per Oyatie's hexagonal
//! layering. The kernel only sees opaque token handles — OpenBao envelope
//! encryption (D8) is enforced in the REST/secret adapter, not here.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Identity value objects
// ---------------------------------------------------------------------------

/// Opaque tenant identifier. Per oyatie-dogfood-tenancy, Oyatie itself runs as
/// a tenant of its own cloud-intelligence; there is no internal bypass.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TenantId(String); // data_class: INTERNAL_ONLY

impl TenantId {
    pub fn new(value: impl Into<String>) -> Result<Self, SubscriptionPoolError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(SubscriptionPoolError::InvalidTenantId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Per-tenant agent identity — usually maps to a single human or automated
/// caller within the tenant.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AgentId(String); // data_class: INTERNAL_ONLY

impl AgentId {
    pub fn new(value: impl Into<String>) -> Result<Self, SubscriptionPoolError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(SubscriptionPoolError::InvalidAgentId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A seat is one OAuth subscription credential (one paid plan slot) belonging
/// to a tenant. SeatIds are stable across token refreshes.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SeatId(String); // data_class: INTERNAL_ONLY

impl SeatId {
    pub fn new(value: impl Into<String>) -> Result<Self, SubscriptionPoolError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(SubscriptionPoolError::InvalidSeatId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Per-token subscription instance — rotates on every refresh.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SubscriptionId(String); // data_class: INTERNAL_ONLY

impl SubscriptionId {
    pub fn new(value: impl Into<String>) -> Result<Self, SubscriptionPoolError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(SubscriptionPoolError::InvalidSubscriptionId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Provider enum — v1 scope locked to Anthropic + OpenAI Codex per
/// cloud-intelligence-reference-repo-audit memory. Gemini = v2, Cursor = v3.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Provider {
    Anthropic,
    Codex,
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Provider::Anthropic => f.write_str("anthropic"),
            Provider::Codex => f.write_str("codex"),
        }
    }
}

/// Credential transport mode attached to a seat.
///
/// The kernel stores only an opaque secret handle for either mode; provider
/// adapters decide whether that handle resolves to an OAuth refresh token,
/// provider access token, API key, or future credential material.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CredentialMode {
    OAuthSubscription,
    ApiKey,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EnvTier {
    Test,
    Staging,
    Prod,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ModelDefaultClass {
    SmallCheap,
    PromotionCandidate,
    ProductionGradeSelection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ModelProfileTag {
    CheapOrSmall,
    SandboxOk,
    NonProdOnly,
    StagingApproved,
    EvalSnapshotBound,
    ProductionGrade,
    ProdApproved,
    SloBacked,
    EvalGatePassed,
    ProductionGradeOnly,
    ProdOnly,
    CheapOrSmallOnly,
    SandboxOnly,
    ProdOnlyWithoutPromotionEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnvTierGatewayBudgetAdmission {
    pub tenant_id: String,                             // data_class: INTERNAL_ONLY
    pub env_tier: Option<EnvTier>,                     // data_class: INTERNAL_ONLY
    pub model_default_class: ModelDefaultClass,        // data_class: INTERNAL_ONLY
    pub model_profile_tags: BTreeSet<ModelProfileTag>, // data_class: INTERNAL_ONLY
    pub model_default_policy_ref: String,              // data_class: INTERNAL_ONLY
    pub tier_cost_budget_policy_ref: String,           // data_class: INTERNAL_ONLY
    pub tier_cost_budget_evidence_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub model_route_registry_snapshot_ref: String,     // data_class: INTERNAL_ONLY
    pub policy_decision_ref: String,                   // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,                     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GatewayBudgetAdmissionDenialReason {
    MissingEnvTier,
    WrongModelDefaultForTier,
    MissingPerTierCostBudgetEvidence,
    FoundryLiveAuthorityResurrection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GatewayBudgetAdmissionDenial {
    pub reasons: BTreeSet<GatewayBudgetAdmissionDenialReason>,
    pub evidence_refs: Vec<String>,
}

pub fn validate_env_tier_gateway_budget_admission(
    admission: &EnvTierGatewayBudgetAdmission,
) -> Result<(), GatewayBudgetAdmissionDenial> {
    let mut reasons = BTreeSet::new();
    let mut evidence_refs = vec![
        admission.model_default_policy_ref.clone(),
        admission.tier_cost_budget_policy_ref.clone(),
        admission.model_route_registry_snapshot_ref.clone(),
        admission.policy_decision_ref.clone(),
        admission.trace_context_ref.clone(),
    ];
    if let Some(evidence_ref) = &admission.tier_cost_budget_evidence_ref {
        evidence_refs.push(evidence_ref.clone());
    }

    if admission.env_tier.is_none() {
        reasons.insert(GatewayBudgetAdmissionDenialReason::MissingEnvTier);
        evidence_refs.push("env-tier:ENV-TIER-REQUIRED:missing_env_tier".to_owned());
    }
    if admission.tier_cost_budget_policy_ref.trim().is_empty()
        || admission
            .tier_cost_budget_evidence_ref
            .as_deref()
            .map(str::trim)
            .unwrap_or_default()
            .is_empty()
    {
        reasons.insert(GatewayBudgetAdmissionDenialReason::MissingPerTierCostBudgetEvidence);
        evidence_refs.push(
            "env-tier:TIER-BUDGET-EVIDENCE-REQUIRED:missing_per_tier_cost_budget_evidence"
                .to_owned(),
        );
    }
    if contains_retired_foundry_live_authority_ref(admission) {
        reasons.insert(GatewayBudgetAdmissionDenialReason::FoundryLiveAuthorityResurrection);
        evidence_refs.push(
            "env-tier:FOUNDRY-LIVE-AUTHORITY-FORBIDDEN:foundry_live_authority_resurrection"
                .to_owned(),
        );
    }
    if let Some(env_tier) = admission.env_tier
        && !admission_satisfies_env_tier(env_tier, admission)
    {
        reasons.insert(GatewayBudgetAdmissionDenialReason::WrongModelDefaultForTier);
        evidence_refs
            .push("env-tier:TIER-MODEL-DEFAULT-MATCH:wrong_model_default_for_tier".to_owned());
    }

    if reasons.is_empty() {
        Ok(())
    } else {
        evidence_refs.retain(|value| !value.trim().is_empty());
        evidence_refs.sort();
        evidence_refs.dedup();
        Err(GatewayBudgetAdmissionDenial {
            reasons,
            evidence_refs,
        })
    }
}

fn admission_satisfies_env_tier(
    env_tier: EnvTier,
    admission: &EnvTierGatewayBudgetAdmission,
) -> bool {
    match env_tier {
        EnvTier::Test => {
            admission.model_default_class == ModelDefaultClass::SmallCheap
                && has_required_tags(
                    &admission.model_profile_tags,
                    &[
                        ModelProfileTag::CheapOrSmall,
                        ModelProfileTag::SandboxOk,
                        ModelProfileTag::NonProdOnly,
                    ],
                )
                && has_no_forbidden_tags(
                    &admission.model_profile_tags,
                    &[
                        ModelProfileTag::ProductionGrade,
                        ModelProfileTag::ProductionGradeOnly,
                        ModelProfileTag::ProdApproved,
                        ModelProfileTag::ProdOnly,
                    ],
                )
        }
        EnvTier::Staging => {
            admission.model_default_class == ModelDefaultClass::PromotionCandidate
                && has_required_tags(
                    &admission.model_profile_tags,
                    &[
                        ModelProfileTag::StagingApproved,
                        ModelProfileTag::EvalSnapshotBound,
                    ],
                )
                && has_no_forbidden_tags(
                    &admission.model_profile_tags,
                    &[ModelProfileTag::ProdOnlyWithoutPromotionEvidence],
                )
        }
        EnvTier::Prod => {
            admission.model_default_class == ModelDefaultClass::ProductionGradeSelection
                && has_required_tags(
                    &admission.model_profile_tags,
                    &[
                        ModelProfileTag::ProductionGrade,
                        ModelProfileTag::ProdApproved,
                        ModelProfileTag::SloBacked,
                        ModelProfileTag::EvalGatePassed,
                    ],
                )
                && has_no_forbidden_tags(
                    &admission.model_profile_tags,
                    &[
                        ModelProfileTag::CheapOrSmallOnly,
                        ModelProfileTag::SandboxOnly,
                        ModelProfileTag::NonProdOnly,
                    ],
                )
        }
    }
}

fn has_required_tags(tags: &BTreeSet<ModelProfileTag>, required: &[ModelProfileTag]) -> bool {
    required.iter().all(|tag| tags.contains(tag))
}

fn has_no_forbidden_tags(tags: &BTreeSet<ModelProfileTag>, forbidden: &[ModelProfileTag]) -> bool {
    forbidden.iter().all(|tag| !tags.contains(tag))
}

fn contains_retired_foundry_live_authority_ref(admission: &EnvTierGatewayBudgetAdmission) -> bool {
    [
        admission.model_default_policy_ref.as_str(),
        admission.tier_cost_budget_policy_ref.as_str(),
        admission.model_route_registry_snapshot_ref.as_str(),
        admission
            .tier_cost_budget_evidence_ref
            .as_deref()
            .unwrap_or_default(),
    ]
    .into_iter()
    .any(is_retired_foundry_live_authority_ref)
}

fn is_retired_foundry_live_authority_ref(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("foundry:cost-budget.md")
        || lower.contains("foundry cost-budget.md")
        || lower.contains("specs/microservices/foundry.json#live-authority")
        || lower.contains("foundry.json#live-authority")
}

// ---------------------------------------------------------------------------
// SubscriptionState — the OAuth-pool state machine
// ---------------------------------------------------------------------------

/// State machine for an [`OAuthSubscription`]. Legal transitions:
///
/// ```text
///   Authorized ──(token issued)──► ActiveUntilExpiry
///   ActiveUntilExpiry ──(approaching expiry)──► RefreshingToken
///   RefreshingToken ──(refresh ok)──► Active
///   RefreshingToken ──(refresh failed/rate-limited)──► Cooldown
///   Active ──(429/upstream error)──► Cooldown
///   Cooldown ──(timer elapsed)──► Active
///   Active|Cooldown ──(repeated failures > threshold)──► Blacklisted
///   Blacklisted is terminal until operator intervention.
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubscriptionState {
    Authorized,
    ActiveUntilExpiry {
        expires_at: Instant,
    },
    RefreshingToken {
        started_at: Instant,
    },
    Active,
    Cooldown {
        until: Instant,
        reason: CooldownReason,
    },
    Blacklisted {
        reason: BlacklistReason,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CooldownReason {
    UpstreamRateLimit429,
    UpstreamServerError5xx,
    RefreshTokenTransientFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BlacklistReason {
    RefreshTokenRevoked,
    RepeatedFailuresExceededThreshold { failure_count: u32 },
    OperatorAction,
}

/// Quota window type for a provider subscription.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum QuotaWindowKind {
    FiveHour,
    Weekly,
}

/// Sliding/fixed provider quota metadata for one seat.
///
/// `used_units` deliberately represents provider-normalized quota units rather
/// than tokens so Anthropic/Codex adapters can map their provider-specific
/// accounting into the same kernel selector.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuotaWindow {
    pub kind: QuotaWindowKind, // data_class: INTERNAL_ONLY
    capacity_units: u64,       // data_class: INTERNAL_ONLY
    used_units: u64,           // data_class: INTERNAL_ONLY
    reset_at: Instant,         // data_class: INTERNAL_ONLY
    window_duration: Duration,
}

impl QuotaWindow {
    pub fn new(
        kind: QuotaWindowKind,
        capacity_units: u64,
        used_units: u64,
        reset_at: Instant,
        window_duration: Duration,
    ) -> Self {
        Self {
            kind,
            capacity_units,
            used_units,
            reset_at,
            window_duration,
        }
    }

    pub fn capacity_units(&self) -> u64 {
        self.capacity_units
    }

    pub fn reset_at(&self) -> Instant {
        self.reset_at
    }

    pub fn window_duration(&self) -> Duration {
        self.window_duration
    }

    pub fn used_units(&self, now: Instant) -> u64 {
        if self.has_reset(now) {
            0
        } else {
            self.used_units
        }
    }

    fn has_capacity_for(&self, now: Instant, inflight_units: u64, estimated_units: u64) -> bool {
        self.projected_units(now, inflight_units, estimated_units) <= self.capacity_units
    }

    fn projected_units(&self, now: Instant, inflight_units: u64, estimated_units: u64) -> u64 {
        self.used_units(now)
            .saturating_add(inflight_units)
            .saturating_add(estimated_units)
    }

    fn record_usage(&mut self, now: Instant, actual_units: u64) {
        if self.has_reset(now) {
            self.reset_at = self.effective_reset_at(now);
            self.used_units = actual_units;
        } else {
            self.used_units = self.used_units.saturating_add(actual_units);
        }
    }

    fn time_normalized_score(
        &self,
        now: Instant,
        inflight_units: u64,
        estimated_units: u64,
    ) -> f64 {
        if self.capacity_units == 0 {
            return f64::NEG_INFINITY;
        }

        let target_usage_percent = self.elapsed_fraction(now);
        let projected_usage_percent = self.projected_units(now, inflight_units, estimated_units)
            as f64
            / self.capacity_units as f64;
        target_usage_percent - projected_usage_percent
    }

    fn elapsed_fraction(&self, now: Instant) -> f64 {
        if self.window_duration.is_zero() {
            return 1.0;
        }

        let effective_reset_at = self.effective_reset_at(now);
        let Some(window_start) = effective_reset_at.checked_sub(self.window_duration) else {
            return 0.0;
        };
        let elapsed = now
            .checked_duration_since(window_start)
            .unwrap_or(Duration::ZERO);
        (elapsed.as_secs_f64() / self.window_duration.as_secs_f64()).clamp(0.0, 1.0)
    }

    fn has_reset(&self, now: Instant) -> bool {
        now >= self.reset_at
    }

    fn effective_reset_at(&self, now: Instant) -> Instant {
        if self.window_duration.is_zero() || now < self.reset_at {
            return self.reset_at;
        }

        let mut reset_at = self.reset_at;
        while now >= reset_at {
            let Some(next_reset_at) = reset_at.checked_add(self.window_duration) else {
                return reset_at;
            };
            reset_at = next_reset_at;
        }
        reset_at
    }
}

/// A single OAuth subscription (one tenant seat for one provider).
#[derive(Clone)]
pub struct OAuthSubscription {
    pub tenant_id: TenantId,             // data_class: INTERNAL_ONLY
    pub seat_id: SeatId,                 // data_class: INTERNAL_ONLY
    pub subscription_id: SubscriptionId, // data_class: INTERNAL_ONLY
    pub provider: Provider,              // data_class: INTERNAL_ONLY
    pub state: SubscriptionState,        // data_class: INTERNAL_ONLY
    pub credential_mode: CredentialMode, // data_class: INTERNAL_ONLY
    /// Opaque handle. The actual provider credential is stored
    /// envelope-encrypted in OpenBao (D8) and never enters the kernel.
    credential_secret_handle: String, // data_class: INTERNAL_ONLY
    quota_windows: Vec<QuotaWindow>,     // data_class: INTERNAL_ONLY
    inflight_units: u64,                 // data_class: INTERNAL_ONLY
    pub failure_count: u32,              // data_class: INTERNAL_ONLY
}

impl OAuthSubscription {
    /// Construct a new [`OAuthSubscription`].
    pub fn new(
        tenant_id: TenantId,
        seat_id: SeatId,
        subscription_id: SubscriptionId,
        provider: Provider,
        state: SubscriptionState,
        refresh_token_handle: impl Into<String>,
        failure_count: u32,
    ) -> Self {
        Self {
            tenant_id,
            seat_id,
            subscription_id,
            provider,
            state,
            credential_mode: CredentialMode::OAuthSubscription,
            credential_secret_handle: refresh_token_handle.into(),
            quota_windows: Vec::new(),
            inflight_units: 0,
            failure_count,
        }
    }

    pub fn with_credential_mode(mut self, credential_mode: CredentialMode) -> Self {
        self.credential_mode = credential_mode;
        self
    }

    pub fn with_quota_windows(
        mut self,
        quota_windows: impl IntoIterator<Item = QuotaWindow>,
    ) -> Self {
        self.quota_windows = quota_windows.into_iter().collect();
        self
    }

    pub fn credential_mode(&self) -> CredentialMode {
        self.credential_mode
    }

    /// Return the opaque refresh-token handle (non-plaintext; actual token
    /// lives in OpenBao envelope-encrypted storage).
    pub fn refresh_token_handle(&self) -> &str {
        &self.credential_secret_handle
    }

    /// Return the opaque credential handle. This is a secret-reference handle,
    /// not plaintext credential material.
    pub fn credential_secret_handle(&self) -> &str {
        &self.credential_secret_handle
    }

    pub fn quota_windows(&self) -> &[QuotaWindow] {
        &self.quota_windows
    }

    fn has_quota_capacity(&self, now: Instant, estimated_units: u64) -> bool {
        self.quota_windows
            .iter()
            .all(|window| window.has_capacity_for(now, self.inflight_units, estimated_units))
    }

    fn reserve_units(&mut self, estimated_units: u64) {
        self.inflight_units = self.inflight_units.saturating_add(estimated_units);
    }

    fn release_units(&mut self, reserved_units: u64) {
        self.inflight_units = self.inflight_units.saturating_sub(reserved_units);
    }

    fn record_usage(&mut self, now: Instant, actual_units: u64) {
        for window in &mut self.quota_windows {
            window.record_usage(now, actual_units);
        }
    }

    fn time_normalized_score(&self, now: Instant, estimated_units: u64) -> f64 {
        self.quota_windows
            .iter()
            .map(|window| window.time_normalized_score(now, self.inflight_units, estimated_units))
            .reduce(f64::min)
            .unwrap_or(0.0)
    }
}

impl fmt::Debug for OAuthSubscription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OAuthSubscription")
            .field("tenant_id", &self.tenant_id)
            .field("seat_id", &self.seat_id)
            .field("subscription_id", &self.subscription_id)
            .field("provider", &self.provider)
            .field("state", &self.state)
            .field("credential_mode", &self.credential_mode)
            .field("credential_secret_handle", &"<REDACTED>")
            .field("quota_windows", &self.quota_windows)
            .field("inflight_units", &self.inflight_units)
            .field("failure_count", &self.failure_count)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Authorization seam (D7)
// ---------------------------------------------------------------------------

/// Authorization decision principal: tenant + agent + the resource (target
/// subscription). Cedar adapter consumes this and returns [`AuthzDecision`].
#[derive(Clone, Debug)]
pub struct AuthzRequest<'a> {
    pub principal_tenant: &'a TenantId, // data_class: INTERNAL_ONLY
    pub principal_agent: &'a AgentId,   // data_class: INTERNAL_ONLY
    pub action: AuthzAction,            // data_class: INTERNAL_ONLY
    pub resource_tenant: &'a TenantId,  // data_class: INTERNAL_ONLY
    pub resource_provider: Provider,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthzAction {
    SelectSeat,
    RefreshToken,
    InvalidateSeat,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthzDecision {
    Allow,
    Forbid,
}

/// D7 — per-tenant forbid-wins isolation seam.
///
/// The Cedar adapter implements this. Cross-tenant requests MUST receive
/// [`AuthzDecision::Forbid`] regardless of how many `permit` rules match,
/// per the forbid-wins semantics of Cedar.
pub trait AuthzGate {
    fn decide(&self, request: &AuthzRequest<'_>) -> AuthzDecision;
}

// ---------------------------------------------------------------------------
// Event-emission seam (D6)
// ---------------------------------------------------------------------------

/// D6 event shape — every cloud-intelligence request emits one of these to the
/// configured sink. ClickHouse OLAP (ADR-0193) and Valkey Stream consume the
/// same shape via separate adapter crates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LlmGatewayEvent {
    pub request_id: String,     // data_class: INTERNAL_ONLY
    pub tenant_id: TenantId,    // data_class: INTERNAL_ONLY
    pub agent_id: AgentId,      // data_class: INTERNAL_ONLY
    pub seat_id: SeatId,        // data_class: INTERNAL_ONLY
    pub provider: Provider,     // data_class: INTERNAL_ONLY
    pub model: String,          // data_class: INTERNAL_ONLY
    pub prompt_tokens: u64,     // data_class: INTERNAL_ONLY
    pub completion_tokens: u64, // data_class: INTERNAL_ONLY
    pub ms_latency: u64,        // data_class: INTERNAL_ONLY
    pub status: EventStatus,    // data_class: INTERNAL_ONLY
    pub timestamp_unix_ms: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventStatus {
    Ok,
    UpstreamError,
    RateLimited,
    Forbidden,
    PoolExhausted,
}

/// Event sink seam. Synchronous from the kernel's view; adapters MAY batch.
pub trait EventSink {
    fn emit(&self, event: LlmGatewayEvent);
}

// ---------------------------------------------------------------------------
// SubscriptionPool kernel (D1)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionStrategy {
    /// Cycle through eligible seats in stable order. Spreads load evenly.
    RoundRobin,
    /// Always pick the first eligible seat; only fall to the next once the
    /// first becomes ineligible (cooldown/blacklist). Matches gpt-load's
    /// keypool default. Useful when one seat has higher capacity.
    FillFirst,
    /// Pick the eligible seat most behind its time-normalized quota drain.
    /// A five-hour window that is 80% elapsed but only 20% consumed should be
    /// drained faster than one that is 20% elapsed and 20% consumed.
    TimeNormalizedQuotaPercent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubscriptionPoolError {
    InvalidTenantId,
    InvalidAgentId,
    InvalidSeatId,
    InvalidSubscriptionId,
    DuplicateSeat,
    NoEligibleSeat,
    ForbiddenByPolicy,
    EnvTierBudgetContractDenied,
}

/// RAII lease on a single seat. Prevents double-allocation of the same seat
/// to two concurrent requests. Call [`SeatLease::complete`] to record the
/// outcome and release the seat. If dropped without calling `complete`, the
/// seat is released with [`SeatOutcome::Released`] and no failure penalty.
pub struct SeatLease {
    seat_id: SeatId,                    // data_class: INTERNAL_ONLY
    pool: Arc<Mutex<SubscriptionPool>>, // data_class: INTERNAL_ONLY
    completed: bool,
    reserved_units: u64,
}

impl SeatLease {
    /// Record the upstream outcome and release this lease.
    pub fn complete(self, outcome: SeatOutcome, now: Instant) -> Result<(), SubscriptionPoolError> {
        let reserved_units = self.reserved_units;
        self.complete_with_usage(outcome, now, reserved_units)
    }

    /// Record the upstream outcome, reconcile actual quota usage, and release
    /// this lease. `actual_units` is only counted for successful requests.
    pub fn complete_with_usage(
        mut self,
        outcome: SeatOutcome,
        now: Instant,
        actual_units: u64,
    ) -> Result<(), SubscriptionPoolError> {
        self.completed = true;
        let mut pool = self
            .pool
            .lock()
            .map_err(|_| SubscriptionPoolError::NoEligibleSeat)?;
        pool.release_lease(&self.seat_id, self.reserved_units);
        if outcome == SeatOutcome::Ok {
            pool.record_usage(&self.seat_id, now, actual_units)?;
        }
        pool.record_outcome(&self.seat_id, outcome, now)
    }

    pub fn seat_id(&self) -> &SeatId {
        &self.seat_id
    }
}

impl Drop for SeatLease {
    fn drop(&mut self) {
        if !self.completed
            && let Ok(mut pool) = self.pool.lock()
        {
            pool.release_lease(&self.seat_id, self.reserved_units);
            // Record Released (no-op outcome) — the seat is returned to the
            // pool without any penalty. Callers that want to record a failure
            // must call complete() explicitly before the lease is dropped.
            let now = Instant::now();
            let _ = pool.record_outcome(&self.seat_id, SeatOutcome::Released, now);
        }
    }
}

/// The kernel pool. One pool per (tenant, provider).
pub struct SubscriptionPool {
    tenant_id: TenantId,
    provider: Provider,
    strategy: SelectionStrategy,
    seats: BTreeMap<SeatId, OAuthSubscription>,
    round_robin_cursor: usize,
    cooldown_duration_429: Duration,
    /// Seats currently held by an active [`SeatLease`]. These are excluded
    /// from selection to prevent double-allocation.
    leased_seats: HashSet<SeatId>,
}

impl SubscriptionPool {
    pub fn new(tenant_id: TenantId, provider: Provider, strategy: SelectionStrategy) -> Self {
        Self {
            tenant_id,
            provider,
            strategy,
            seats: BTreeMap::new(),
            round_robin_cursor: 0,
            cooldown_duration_429: Duration::from_secs(60),
            leased_seats: HashSet::new(),
        }
    }

    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    pub fn provider(&self) -> Provider {
        self.provider
    }

    pub fn add_seat(
        &mut self,
        subscription: OAuthSubscription,
    ) -> Result<(), SubscriptionPoolError> {
        if subscription.tenant_id != self.tenant_id {
            return Err(SubscriptionPoolError::ForbiddenByPolicy);
        }
        if subscription.provider != self.provider {
            return Err(SubscriptionPoolError::ForbiddenByPolicy);
        }
        let seat_id = subscription.seat_id.clone();
        if self.seats.contains_key(&seat_id) {
            return Err(SubscriptionPoolError::DuplicateSeat);
        }
        self.seats.insert(seat_id, subscription);
        Ok(())
    }

    pub fn seat_count(&self) -> usize {
        self.seats.len()
    }

    pub fn credential_secret_handle_for_seat(&self, seat_id: &SeatId) -> Option<String> {
        self.seats
            .get(seat_id)
            .map(|seat| seat.credential_secret_handle().to_string())
    }

    /// Return the [`CredentialMode`] of the named seat, if present. Mirrors
    /// [`credential_secret_handle_for_seat`](Self::credential_secret_handle_for_seat)
    /// so proxy callers can branch on OAuth vs API-key transport per seat.
    pub fn credential_mode_for_seat(&self, seat_id: &SeatId) -> Option<CredentialMode> {
        self.seats.get(seat_id).map(|seat| seat.credential_mode())
    }

    /// Return true when at least one seat can be selected without considering
    /// per-agent authorization. Readiness uses this to avoid marking an empty,
    /// cooling, or blacklisted pool ready.
    pub fn has_eligible_seat(&self, now: Instant) -> bool {
        self.seats
            .values()
            .any(|seat| Self::is_eligible(&seat.state, now) && seat.has_quota_capacity(now, 1))
    }

    /// Acquire a [`SeatLease`] for the next eligible seat. The leased seat is
    /// marked as reserved and excluded from concurrent `lease` calls until
    /// [`SeatLease::complete`] is called (or the lease is dropped).
    ///
    /// The pool must be wrapped in an `Arc<Mutex<SubscriptionPool>>` so the
    /// lease can release itself. Pass the same `Arc` as `pool_ref`.
    pub fn lease(
        pool_ref: &Arc<Mutex<SubscriptionPool>>,
        agent_id: &AgentId,
        gate: &dyn AuthzGate,
        now: Instant,
    ) -> Result<SeatLease, SubscriptionPoolError> {
        Self::lease_with_estimate(pool_ref, agent_id, gate, now, 1)
    }

    /// Acquire a [`SeatLease`] and reserve the estimated provider quota units.
    pub fn lease_with_estimate(
        pool_ref: &Arc<Mutex<SubscriptionPool>>,
        agent_id: &AgentId,
        gate: &dyn AuthzGate,
        now: Instant,
        estimated_units: u64,
    ) -> Result<SeatLease, SubscriptionPoolError> {
        let seat_id = {
            let mut pool = pool_ref
                .lock()
                .map_err(|_| SubscriptionPoolError::NoEligibleSeat)?;
            let sid = pool.select_excluding_leased(agent_id, gate, now, estimated_units)?;
            pool.reserve_lease(&sid, estimated_units)?;
            sid
        };
        Ok(SeatLease {
            seat_id,
            pool: Arc::clone(pool_ref),
            completed: false,
            reserved_units: estimated_units,
        })
    }

    /// Internal: release a seat from the leased set.
    pub(crate) fn release_lease(&mut self, seat_id: &SeatId, reserved_units: u64) {
        self.leased_seats.remove(seat_id);
        if let Some(seat) = self.seats.get_mut(seat_id) {
            seat.release_units(reserved_units);
        }
    }

    pub fn seat_inflight_units(&self, seat_id: &SeatId) -> Option<u64> {
        self.seats.get(seat_id).map(|seat| seat.inflight_units)
    }

    pub fn seat_window_used_units(
        &self,
        seat_id: &SeatId,
        kind: QuotaWindowKind,
        now: Instant,
    ) -> Option<u64> {
        self.seats
            .get(seat_id)?
            .quota_windows
            .iter()
            .find(|window| window.kind == kind)
            .map(|window| window.used_units(now))
    }

    /// Select the next eligible seat for an inbound request (D1).
    ///
    /// D7 contract: the [`AuthzGate`] is consulted exactly once per call,
    /// before any seat is returned. Forbid wins regardless of pool capacity.
    pub fn select(
        &mut self,
        agent_id: &AgentId,
        gate: &dyn AuthzGate,
        now: Instant,
    ) -> Result<SeatId, SubscriptionPoolError> {
        self.select_with_estimate(agent_id, gate, now, 1)
    }

    pub fn select_with_estimate(
        &mut self,
        agent_id: &AgentId,
        gate: &dyn AuthzGate,
        now: Instant,
        estimated_units: u64,
    ) -> Result<SeatId, SubscriptionPoolError> {
        self.select_candidate(agent_id, gate, now, estimated_units, false)
    }

    pub fn select_with_env_tier_budget(
        &mut self,
        agent_id: &AgentId,
        gate: &dyn AuthzGate,
        now: Instant,
        estimated_units: u64,
        admission: &EnvTierGatewayBudgetAdmission,
    ) -> Result<SeatId, SubscriptionPoolError> {
        validate_env_tier_gateway_budget_admission(admission)
            .map_err(|_| SubscriptionPoolError::EnvTierBudgetContractDenied)?;
        self.select_candidate(agent_id, gate, now, estimated_units, false)
    }

    pub fn lease_with_env_tier_budget(
        pool_ref: &Arc<Mutex<SubscriptionPool>>,
        agent_id: &AgentId,
        gate: &dyn AuthzGate,
        now: Instant,
        estimated_units: u64,
        admission: &EnvTierGatewayBudgetAdmission,
    ) -> Result<SeatLease, SubscriptionPoolError> {
        validate_env_tier_gateway_budget_admission(admission)
            .map_err(|_| SubscriptionPoolError::EnvTierBudgetContractDenied)?;
        Self::lease_with_estimate(pool_ref, agent_id, gate, now, estimated_units)
    }

    fn select_candidate(
        &mut self,
        agent_id: &AgentId,
        gate: &dyn AuthzGate,
        now: Instant,
        estimated_units: u64,
        exclude_leased: bool,
    ) -> Result<SeatId, SubscriptionPoolError> {
        let request = AuthzRequest {
            principal_tenant: &self.tenant_id,
            principal_agent: agent_id,
            action: AuthzAction::SelectSeat,
            resource_tenant: &self.tenant_id,
            resource_provider: self.provider,
        };
        if gate.decide(&request) == AuthzDecision::Forbid {
            return Err(SubscriptionPoolError::ForbiddenByPolicy);
        }

        if self.seats.is_empty() {
            return Err(SubscriptionPoolError::NoEligibleSeat);
        }

        let seat_ids: Vec<SeatId> = self.seats.keys().cloned().collect();
        let n = seat_ids.len();

        match self.strategy {
            SelectionStrategy::FillFirst => {
                for sid in &seat_ids {
                    if self.is_selectable(sid, now, estimated_units, exclude_leased) {
                        return Ok(sid.clone());
                    }
                }
                Err(SubscriptionPoolError::NoEligibleSeat)
            }
            SelectionStrategy::RoundRobin => {
                for offset in 0..n {
                    let idx = (self.round_robin_cursor + offset) % n;
                    let sid = &seat_ids[idx];
                    if self.is_selectable(sid, now, estimated_units, exclude_leased) {
                        self.round_robin_cursor = (idx + 1) % n;
                        return Ok(sid.clone());
                    }
                }
                Err(SubscriptionPoolError::NoEligibleSeat)
            }
            SelectionStrategy::TimeNormalizedQuotaPercent => {
                let mut best: Option<(usize, SeatId, f64)> = None;
                for offset in 0..n {
                    let idx = (self.round_robin_cursor + offset) % n;
                    let sid = &seat_ids[idx];
                    if !self.is_selectable(sid, now, estimated_units, exclude_leased) {
                        continue;
                    }
                    let score = self.seats[sid].time_normalized_score(now, estimated_units);
                    let should_replace = best
                        .as_ref()
                        .map(|(_, _, best_score)| score > *best_score)
                        .unwrap_or(true);
                    if should_replace {
                        best = Some((idx, sid.clone(), score));
                    }
                }

                if let Some((idx, sid, _)) = best {
                    self.round_robin_cursor = (idx + 1) % n;
                    Ok(sid)
                } else {
                    Err(SubscriptionPoolError::NoEligibleSeat)
                }
            }
        }
    }

    fn reserve_lease(
        &mut self,
        seat_id: &SeatId,
        estimated_units: u64,
    ) -> Result<(), SubscriptionPoolError> {
        let Some(seat) = self.seats.get_mut(seat_id) else {
            return Err(SubscriptionPoolError::NoEligibleSeat);
        };
        self.leased_seats.insert(seat_id.clone());
        seat.reserve_units(estimated_units);
        Ok(())
    }

    fn record_usage(
        &mut self,
        seat_id: &SeatId,
        now: Instant,
        actual_units: u64,
    ) -> Result<(), SubscriptionPoolError> {
        let Some(seat) = self.seats.get_mut(seat_id) else {
            return Err(SubscriptionPoolError::NoEligibleSeat);
        };
        seat.record_usage(now, actual_units);
        Ok(())
    }

    /// Record an upstream outcome against a seat so the state machine can
    /// transition (e.g. 429 -> Cooldown, repeated failures -> Blacklisted).
    pub fn record_outcome(
        &mut self,
        seat_id: &SeatId,
        outcome: SeatOutcome,
        now: Instant,
    ) -> Result<(), SubscriptionPoolError> {
        let cooldown = self.cooldown_duration_429;
        let Some(seat) = self.seats.get_mut(seat_id) else {
            return Err(SubscriptionPoolError::NoEligibleSeat);
        };

        match outcome {
            SeatOutcome::Released => {
                // No-op: dropped without explicit complete; no penalty applied.
                return Ok(());
            }
            SeatOutcome::Ok => {
                seat.failure_count = 0;
                seat.state = SubscriptionState::Active;
            }
            SeatOutcome::RateLimited429 => {
                seat.failure_count = seat.failure_count.saturating_add(1);
                if seat.failure_count > BLACKLIST_THRESHOLD {
                    seat.state = SubscriptionState::Blacklisted {
                        reason: BlacklistReason::RepeatedFailuresExceededThreshold {
                            failure_count: seat.failure_count,
                        },
                    };
                } else {
                    seat.state = SubscriptionState::Cooldown {
                        until: now + cooldown,
                        reason: CooldownReason::UpstreamRateLimit429,
                    };
                }
            }
            SeatOutcome::ServerError5xx => {
                seat.failure_count = seat.failure_count.saturating_add(1);
                if seat.failure_count > BLACKLIST_THRESHOLD {
                    seat.state = SubscriptionState::Blacklisted {
                        reason: BlacklistReason::RepeatedFailuresExceededThreshold {
                            failure_count: seat.failure_count,
                        },
                    };
                } else {
                    seat.state = SubscriptionState::Cooldown {
                        until: now + cooldown,
                        reason: CooldownReason::UpstreamServerError5xx,
                    };
                }
            }
            SeatOutcome::RefreshTokenRevoked => {
                seat.state = SubscriptionState::Blacklisted {
                    reason: BlacklistReason::RefreshTokenRevoked,
                };
            }
            SeatOutcome::RefreshFailed => {
                seat.failure_count = seat.failure_count.saturating_add(1);
                if seat.failure_count > BLACKLIST_THRESHOLD {
                    seat.state = SubscriptionState::Blacklisted {
                        reason: BlacklistReason::RepeatedFailuresExceededThreshold {
                            failure_count: seat.failure_count,
                        },
                    };
                } else {
                    seat.state = SubscriptionState::Cooldown {
                        until: now + cooldown,
                        reason: CooldownReason::RefreshTokenTransientFailure,
                    };
                }
            }
        }
        Ok(())
    }

    /// Like [`select`] but also excludes currently-leased seats.
    fn select_excluding_leased(
        &mut self,
        agent_id: &AgentId,
        gate: &dyn AuthzGate,
        now: Instant,
        estimated_units: u64,
    ) -> Result<SeatId, SubscriptionPoolError> {
        self.select_candidate(agent_id, gate, now, estimated_units, true)
    }

    fn is_selectable(
        &self,
        seat_id: &SeatId,
        now: Instant,
        estimated_units: u64,
        exclude_leased: bool,
    ) -> bool {
        if exclude_leased && self.leased_seats.contains(seat_id) {
            return false;
        }

        let Some(seat) = self.seats.get(seat_id) else {
            return false;
        };
        Self::is_eligible(&seat.state, now) && seat.has_quota_capacity(now, estimated_units)
    }

    fn is_eligible(state: &SubscriptionState, now: Instant) -> bool {
        match state {
            SubscriptionState::Active => true,
            SubscriptionState::ActiveUntilExpiry { expires_at } => *expires_at > now,
            SubscriptionState::Cooldown { until, .. } => *until <= now,
            SubscriptionState::Authorized
            | SubscriptionState::RefreshingToken { .. }
            | SubscriptionState::Blacklisted { .. } => false,
        }
    }
}

/// Failure-count threshold above which a seat is blacklisted rather than being
/// put in repeated cooldown. Once crossed, only operator action re-enables it.
const BLACKLIST_THRESHOLD: u32 = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SeatOutcome {
    Ok,
    RateLimited429,
    ServerError5xx,
    RefreshTokenRevoked,
    /// Vault or transient OAuth refresh failure (not a permanent revocation).
    /// Seat enters Cooldown with `RefreshTokenTransientFailure` reason.
    RefreshFailed,
    /// Lease was dropped without an explicit [`SeatLease::complete`] call
    /// (e.g. a future was cancelled). Treated as a no-op by the pool —
    /// no penalty is applied and failure_count is not incremented.
    Released,
}

// ---------------------------------------------------------------------------
// Opaque secret-handle validators (shared)
// ---------------------------------------------------------------------------

/// Heuristic: does `value` look like a JWT (three dot-separated segments with a
/// `eyJ` JSON-base64 header)? Used to reject raw bearer tokens that must never
/// be stored as opaque secret handles.
pub fn looks_like_jwt(value: &str) -> bool {
    value.split('.').count() == 3 && value.starts_with("eyJ")
}

/// Validate that `handle` is an opaque secret-store reference rather than a raw
/// credential. Accepts only `vault://`, `kms://`, or `sref://openbao/` schemes
/// and rejects empty/whitespace/control/path-traversal inputs as well as obvious
/// raw secrets (`sk-`, `bearer `, `raw-secret`, `refresh-token`, JWTs).
///
/// Single source of truth for both the REST runtime-registration path and the
/// app boot-time config path.
pub fn is_secret_handle_reference(handle: &str) -> bool {
    let trimmed = handle.trim();
    if trimmed.is_empty()
        || trimmed != handle
        || trimmed.contains("..")
        || trimmed.chars().any(char::is_whitespace)
        || trimmed.chars().any(char::is_control)
    {
        return false;
    }
    let lowered = trimmed.to_ascii_lowercase();
    let has_allowed_scheme = lowered.starts_with("vault://")
        || lowered.starts_with("kms://")
        || lowered.starts_with("sref://openbao/");
    has_allowed_scheme
        && !trimmed.starts_with("sk-")
        && !lowered.starts_with("bearer ")
        && !lowered.contains("raw-secret")
        && !lowered.contains("refresh-token")
        && !looks_like_jwt(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENV_TIER_RED_FIXTURES: &str = include_str!(
        "../../../../../oya/intelligence/contracts/fixtures/env-tier-model-budget/red-fixtures.json"
    );

    struct AllowGate;

    impl AuthzGate for AllowGate {
        fn decide(&self, _request: &AuthzRequest<'_>) -> AuthzDecision {
            AuthzDecision::Allow
        }
    }

    fn tenant(value: &str) -> TenantId {
        TenantId::new(value).expect("valid tenant")
    }

    fn agent(value: &str) -> AgentId {
        AgentId::new(value).expect("valid agent")
    }

    fn seat(value: &str) -> SeatId {
        SeatId::new(value).expect("valid seat")
    }

    fn subscription_id(value: impl Into<String>) -> SubscriptionId {
        SubscriptionId::new(value).expect("valid subscription")
    }

    fn active_subscription(
        tenant_id: TenantId,
        seat_id: SeatId,
        provider: Provider,
        credential_secret_handle: &str,
    ) -> OAuthSubscription {
        OAuthSubscription::new(
            tenant_id,
            seat_id.clone(),
            subscription_id(format!("sub-{}", seat_id.as_str())),
            provider,
            SubscriptionState::Active,
            credential_secret_handle,
            0,
        )
    }

    fn gateway_admission() -> EnvTierGatewayBudgetAdmission {
        EnvTierGatewayBudgetAdmission {
            tenant_id: "tenant-a".to_owned(),
            env_tier: Some(EnvTier::Test),
            model_default_class: ModelDefaultClass::SmallCheap,
            model_profile_tags: BTreeSet::from([
                ModelProfileTag::CheapOrSmall,
                ModelProfileTag::SandboxOk,
                ModelProfileTag::NonProdOnly,
            ]),
            model_default_policy_ref: "policy:intelligence.env-tier.model-default.test.v1"
                .to_owned(),
            tier_cost_budget_policy_ref: "policy:intelligence.env-tier.cost-budget.test.v1"
                .to_owned(),
            tier_cost_budget_evidence_ref: Some("budget:intelligence:test:gateway".to_owned()),
            model_route_registry_snapshot_ref: "route-registry:intelligence:env-tier:test"
                .to_owned(),
            policy_decision_ref: "policy-decision:cloud-intelligence:test:allow".to_owned(),
            trace_context_ref: "trace:cloud-intelligence:env-tier:test".to_owned(),
        }
    }

    #[test]
    fn gateway_budget_admission_consumes_red_fixture_denials() {
        assert!(ENV_TIER_RED_FIXTURES.contains("missing_env_tier_denies_before_model_selection"));
        assert!(ENV_TIER_RED_FIXTURES.contains("test_tier_rejects_production_grade_default"));
        assert!(ENV_TIER_RED_FIXTURES.contains("prod_tier_rejects_missing_cost_budget_evidence"));
        assert!(ENV_TIER_RED_FIXTURES.contains("foundry_live_authority_resurrection_is_rejected"));
    }

    #[test]
    fn missing_env_tier_denies_before_gateway_seat_lease() {
        let now = Instant::now();
        let pool = Arc::new(Mutex::new(SubscriptionPool::new(
            tenant("tenant-a"),
            Provider::Anthropic,
            SelectionStrategy::RoundRobin,
        )));
        let sid = seat("seat-gateway");
        pool.lock()
            .expect("pool lock")
            .add_seat(active_subscription(
                tenant("tenant-a"),
                sid.clone(),
                Provider::Anthropic,
                "vault://tenant-a/gateway",
            ))
            .expect("add seat");
        let mut admission = gateway_admission();
        admission.env_tier = None;

        let denied = SubscriptionPool::lease_with_env_tier_budget(
            &pool,
            &agent("agent-a"),
            &AllowGate,
            now,
            1,
            &admission,
        );

        assert!(matches!(
            denied,
            Err(SubscriptionPoolError::EnvTierBudgetContractDenied)
        ));
        assert_eq!(
            pool.lock().expect("pool lock").seat_inflight_units(&sid),
            Some(0)
        );
    }

    #[test]
    fn prod_gateway_requires_cost_budget_evidence_before_dispatch_admission() {
        let mut admission = gateway_admission();
        admission.env_tier = Some(EnvTier::Prod);
        admission.model_default_class = ModelDefaultClass::ProductionGradeSelection;
        admission.model_profile_tags = BTreeSet::from([
            ModelProfileTag::ProductionGrade,
            ModelProfileTag::ProdApproved,
            ModelProfileTag::SloBacked,
            ModelProfileTag::EvalGatePassed,
        ]);
        admission.model_default_policy_ref =
            "policy:intelligence.env-tier.model-default.prod.v1".to_owned();
        admission.tier_cost_budget_policy_ref =
            "policy:intelligence.env-tier.cost-budget.prod.v1".to_owned();
        admission.tier_cost_budget_evidence_ref = None;

        let denial = validate_env_tier_gateway_budget_admission(&admission)
            .expect_err("missing prod budget evidence must deny");

        assert!(
            denial
                .reasons
                .contains(&GatewayBudgetAdmissionDenialReason::MissingPerTierCostBudgetEvidence)
        );
        assert!(
            denial.evidence_refs.contains(
                &"env-tier:TIER-BUDGET-EVIDENCE-REQUIRED:missing_per_tier_cost_budget_evidence"
                    .to_owned()
            )
        );
    }

    #[test]
    fn foundry_live_authority_refs_are_rejected_at_gateway_admission() {
        let mut admission = gateway_admission();
        admission.model_default_policy_ref =
            "foundry:cost-budget.md#test_tier_model_default".to_owned();
        admission.model_route_registry_snapshot_ref =
            "specs/microservices/foundry.json#live-authority".to_owned();

        let denial = validate_env_tier_gateway_budget_admission(&admission)
            .expect_err("retired foundry authority must deny");

        assert!(
            denial
                .reasons
                .contains(&GatewayBudgetAdmissionDenialReason::FoundryLiveAuthorityResurrection)
        );
    }

    #[test]
    fn credential_mode_metadata_never_leaks_secret_handles_in_debug() {
        let api_key = active_subscription(
            tenant("tenant-a"),
            seat("seat-api"),
            Provider::Anthropic,
            "sk-ant-api03-really-secret",
        )
        .with_credential_mode(CredentialMode::ApiKey);
        let oauth = active_subscription(
            tenant("tenant-a"),
            seat("seat-oauth"),
            Provider::Codex,
            "refresh-token-really-secret",
        )
        .with_credential_mode(CredentialMode::OAuthSubscription);

        assert_eq!(api_key.credential_mode(), CredentialMode::ApiKey);
        assert_eq!(oauth.credential_mode(), CredentialMode::OAuthSubscription);
        assert_eq!(
            api_key.credential_secret_handle(),
            "sk-ant-api03-really-secret"
        );

        let api_debug = format!("{api_key:?}");
        let oauth_debug = format!("{oauth:?}");
        assert!(api_debug.contains("credential_mode"));
        assert!(api_debug.contains("<REDACTED>"));
        assert!(oauth_debug.contains("<REDACTED>"));
        assert!(!api_debug.contains("sk-ant-api03-really-secret"));
        assert!(!oauth_debug.contains("refresh-token-really-secret"));
    }

    #[test]
    fn tenant_and_provider_isolation_still_rejects_foreign_seats() {
        let mut pool = SubscriptionPool::new(
            tenant("tenant-a"),
            Provider::Anthropic,
            SelectionStrategy::RoundRobin,
        );

        let wrong_tenant = active_subscription(
            tenant("tenant-b"),
            seat("seat-b"),
            Provider::Anthropic,
            "vault://tenant-b/anthropic",
        );
        let wrong_provider = active_subscription(
            tenant("tenant-a"),
            seat("seat-codex"),
            Provider::Codex,
            "vault://tenant-a/codex",
        );

        assert_eq!(
            pool.add_seat(wrong_tenant),
            Err(SubscriptionPoolError::ForbiddenByPolicy)
        );
        assert_eq!(
            pool.add_seat(wrong_provider),
            Err(SubscriptionPoolError::ForbiddenByPolicy)
        );
        assert_eq!(pool.seat_count(), 0);
    }

    #[test]
    fn time_normalized_quota_percent_prefers_near_reset_underused_seat() {
        let now = Instant::now();
        let five_hours = Duration::from_secs(5 * 60 * 60);
        let mut pool = SubscriptionPool::new(
            tenant("tenant-a"),
            Provider::Anthropic,
            SelectionStrategy::TimeNormalizedQuotaPercent,
        );

        let about_to_reset = active_subscription(
            tenant("tenant-a"),
            seat("seat-z-about-to-reset"),
            Provider::Anthropic,
            "vault://tenant-a/about-to-reset",
        )
        .with_quota_windows([QuotaWindow::new(
            QuotaWindowKind::FiveHour,
            100,
            20,
            now + Duration::from_secs(60 * 60),
            five_hours,
        )]);
        let early_window = active_subscription(
            tenant("tenant-a"),
            seat("seat-a-early-window"),
            Provider::Anthropic,
            "vault://tenant-a/early-window",
        )
        .with_quota_windows([QuotaWindow::new(
            QuotaWindowKind::FiveHour,
            100,
            20,
            now + Duration::from_secs(4 * 60 * 60),
            five_hours,
        )]);

        pool.add_seat(early_window).expect("seat accepted");
        pool.add_seat(about_to_reset).expect("seat accepted");

        assert_eq!(
            pool.select(&agent("agent-a"), &AllowGate, now)
                .expect("seat selected"),
            seat("seat-z-about-to-reset")
        );
    }

    #[test]
    fn hard_quota_window_exhaustion_makes_seat_ineligible_until_reset() {
        let now = Instant::now();
        let five_hours = Duration::from_secs(5 * 60 * 60);
        let mut pool = SubscriptionPool::new(
            tenant("tenant-a"),
            Provider::Codex,
            SelectionStrategy::TimeNormalizedQuotaPercent,
        );

        let exhausted = active_subscription(
            tenant("tenant-a"),
            seat("seat-exhausted"),
            Provider::Codex,
            "vault://tenant-a/codex-exhausted",
        )
        .with_quota_windows([QuotaWindow::new(
            QuotaWindowKind::FiveHour,
            100,
            100,
            now + Duration::from_secs(60 * 60),
            five_hours,
        )]);
        pool.add_seat(exhausted).expect("seat accepted");

        assert!(!pool.has_eligible_seat(now));
        assert_eq!(
            pool.select(&agent("agent-a"), &AllowGate, now),
            Err(SubscriptionPoolError::NoEligibleSeat)
        );
        assert!(pool.has_eligible_seat(now + Duration::from_secs(60 * 60 + 1)));
    }

    #[test]
    fn lease_estimate_reserves_inflight_units_and_completion_reconciles_actual_usage() {
        let now = Instant::now();
        let five_hours = Duration::from_secs(5 * 60 * 60);
        let pool = Arc::new(Mutex::new(SubscriptionPool::new(
            tenant("tenant-a"),
            Provider::Anthropic,
            SelectionStrategy::RoundRobin,
        )));
        for sid in ["seat-a", "seat-b"] {
            let sub = active_subscription(
                tenant("tenant-a"),
                seat(sid),
                Provider::Anthropic,
                &format!("vault://tenant-a/{sid}"),
            )
            .with_quota_windows([QuotaWindow::new(
                QuotaWindowKind::FiveHour,
                100,
                0,
                now + five_hours,
                five_hours,
            )]);
            pool.lock()
                .expect("pool lock")
                .add_seat(sub)
                .expect("add seat");
        }

        let first =
            SubscriptionPool::lease_with_estimate(&pool, &agent("agent-a"), &AllowGate, now, 90)
                .expect("first lease");
        assert_eq!(first.seat_id(), &seat("seat-a"));
        assert_eq!(
            pool.lock()
                .expect("pool lock")
                .seat_inflight_units(&seat("seat-a")),
            Some(90)
        );

        let second =
            SubscriptionPool::lease_with_estimate(&pool, &agent("agent-b"), &AllowGate, now, 90)
                .expect("second lease");
        assert_eq!(second.seat_id(), &seat("seat-b"));

        first
            .complete_with_usage(SeatOutcome::Ok, now, 70)
            .expect("complete first");
        assert_eq!(
            pool.lock()
                .expect("pool lock")
                .seat_inflight_units(&seat("seat-a")),
            Some(0)
        );
        assert_eq!(
            pool.lock().expect("pool lock").seat_window_used_units(
                &seat("seat-a"),
                QuotaWindowKind::FiveHour,
                now
            ),
            Some(70)
        );
        drop(second);
    }

    #[test]
    fn usage_recorded_after_reset_counts_against_the_new_window() {
        let now = Instant::now();
        let five_hours = Duration::from_secs(5 * 60 * 60);
        let pool = Arc::new(Mutex::new(SubscriptionPool::new(
            tenant("tenant-a"),
            Provider::Codex,
            SelectionStrategy::TimeNormalizedQuotaPercent,
        )));
        let sid = seat("seat-reset");
        let sub = active_subscription(
            tenant("tenant-a"),
            sid.clone(),
            Provider::Codex,
            "vault://tenant-a/reset",
        )
        .with_quota_windows([QuotaWindow::new(
            QuotaWindowKind::FiveHour,
            100,
            100,
            now - Duration::from_secs(1),
            five_hours,
        )]);
        pool.lock()
            .expect("pool lock")
            .add_seat(sub)
            .expect("add seat");

        assert!(pool.lock().expect("pool lock").has_eligible_seat(now));
        let lease =
            SubscriptionPool::lease_with_estimate(&pool, &agent("agent-a"), &AllowGate, now, 10)
                .expect("lease after reset");
        lease
            .complete_with_usage(SeatOutcome::Ok, now, 10)
            .expect("complete after reset");

        let mut locked = pool.lock().expect("pool lock");
        assert_eq!(
            locked.seat_window_used_units(&sid, QuotaWindowKind::FiveHour, now),
            Some(10)
        );
        assert_eq!(
            locked.select_with_estimate(&agent("agent-a"), &AllowGate, now, 91),
            Err(SubscriptionPoolError::NoEligibleSeat)
        );
    }

    #[test]
    fn cooldown_blocks_selection_until_timer_expires() {
        let now = Instant::now();
        let mut pool = SubscriptionPool::new(
            tenant("tenant-a"),
            Provider::Anthropic,
            SelectionStrategy::RoundRobin,
        );
        let sid = seat("seat-cooling");
        pool.add_seat(active_subscription(
            tenant("tenant-a"),
            sid.clone(),
            Provider::Anthropic,
            "vault://tenant-a/cooling",
        ))
        .expect("seat accepted");

        assert!(pool.has_eligible_seat(now));
        pool.record_outcome(&sid, SeatOutcome::RateLimited429, now)
            .expect("rate limit recorded");
        assert!(!pool.has_eligible_seat(now + Duration::from_secs(30)));
        assert!(pool.has_eligible_seat(now + Duration::from_secs(61)));
        assert_eq!(
            pool.select(&agent("agent-a"), &AllowGate, now + Duration::from_secs(61))
                .expect("seat selected after cooldown"),
            sid
        );
    }
}
