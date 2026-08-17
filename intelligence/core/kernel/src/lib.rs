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
//! - [`AuthzGate`] trait — kernel-level seam for the owned policy-engine port
//!   (D7 per-tenant forbid-wins isolation).
//! - [`EventSink`] trait + [`LlmGatewayEvent`] — D6 event-emission contract;
//!   ClickHouse + Valkey Stream impls live in separate adapter crates.
//!
//! Kernel does NOT take direct dependencies on cedar-policy, valkey, clickhouse,
//! reqwest, or concrete secret-provider engines. Those belong to adapter crates
//! per Oyatie's hexagonal layering. The kernel only sees opaque token handles —
//! envelope encryption (D8) is enforced in the REST/secret adapter, not here.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub mod cost;
pub mod model_routing;
pub mod overage_guard;
pub mod safety;
pub mod session;
pub mod xproxy_parity;

use overage_guard::{GuardDecision, HaltReason};

pub use session::{derive_sticky_key, prompt_cache_key};

/// Build a stable sticky-affinity key without storing raw prompt content.
///
/// Thin wrapper over the canonical message-fingerprint path in [`session`];
/// kept for callers that only have a first user message. New call sites with a
/// possible client wire session id should use [`session::derive_sticky_key`].
pub fn privacy_preserving_sticky_key(first_user_message: &str) -> String {
    session::message_sticky_key(first_user_message)
}

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

/// Provider enum for cloud-intelligence gateway pools.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Provider {
    Anthropic,
    Codex,
    Gemini,
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Provider::Anthropic => f.write_str("anthropic"),
            Provider::Codex => f.write_str("codex"),
            Provider::Gemini => f.write_str("gemini"),
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
///   Active ──(SC8 overage guard trips, enforce mode)──► Halted
///   Halted ──(resume_at elapsed)──► eligible again (cooldown-resume)
///   Halted ──(operator admin-resume)──► Active
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
    /// SC8 overage-guard circuit-break: the seat tripped the representative-claim
    /// overage guard (enforce mode) or hit per-credential quota exhaustion.
    /// `resume_at == None` => admin-resume only; `Some(t)` => auto-resume
    /// (cooldown-resume) once `now >= t`, with operator early-resume available.
    Halted {
        resume_at: Option<Instant>,
        reason: HaltReason,
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

    fn utilization(&self, now: Instant, inflight_units: u64, estimated_units: u64) -> f64 {
        if self.capacity_units == 0 {
            return 1.0;
        }
        (self.projected_units(now, inflight_units, estimated_units) as f64
            / self.capacity_units as f64)
            .clamp(0.0, 1.0)
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
    /// Opaque handle. The actual provider credential is stored behind the owned
    /// secret-provider/KMS port (D8) and never enters the kernel.
    credential_secret_handle: String, // data_class: INTERNAL_ONLY
    quota_windows: Vec<QuotaWindow>,     // data_class: INTERNAL_ONLY
    inflight_units: u64,                 // data_class: INTERNAL_ONLY
    pub failure_count: u32,              // data_class: INTERNAL_ONLY
}

/// Account-shaped, secret-free seat status projection for admin read surfaces.
/// Carries resource identity and coarse state only — never the credential
/// secret handle, so REST/proto status responses cannot leak it.
#[derive(Clone, Debug, PartialEq)]
pub struct RedactedSeatStatus {
    pub tenant_id: String,     // data_class: INTERNAL_ONLY
    pub provider: Provider,    // data_class: INTERNAL_ONLY
    pub seat_id: String,       // data_class: INTERNAL_ONLY
    pub state: &'static str,   // data_class: INTERNAL_ONLY
    pub headroom_percent: f64, // data_class: INTERNAL_ONLY
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

    /// Return the opaque refresh-token handle (non-plaintext; actual token is
    /// resolved through the owned secret-provider/KMS port).
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

    fn headroom(&self, now: Instant, estimated_units: u64) -> f64 {
        let max_utilization = self
            .quota_windows
            .iter()
            .map(|window| window.utilization(now, self.inflight_units, estimated_units))
            .reduce(f64::max)
            .unwrap_or(0.0);
        (1.0 - max_utilization).clamp(0.0, 1.0)
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
/// subscription). The owned policy-engine adapter consumes this and returns [`AuthzDecision`].
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
/// Owned policy-engine adapters implement this. Cross-tenant requests MUST
/// receive [`AuthzDecision::Forbid`] regardless of how many allow rules match;
/// deny decisions are authoritative at the service boundary.
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
    sticky_bindings: BTreeMap<String, StickyBinding>,
}

#[derive(Clone, Debug)]
struct StickyBinding {
    seat_id: SeatId,
    expires_at: Instant,
}

/// Sticky-session constraints for a quota-aware seat lease.
#[derive(Clone, Copy, Debug)]
pub struct StickyLeaseSpec<'a> {
    sticky_key: &'a str,
    ttl: Duration,
    estimated_units: u64,
}

impl<'a> StickyLeaseSpec<'a> {
    /// Bind a privacy-preserving sticky key to its lifetime and quota estimate.
    pub const fn new(sticky_key: &'a str, ttl: Duration, estimated_units: u64) -> Self {
        Self {
            sticky_key,
            ttl,
            estimated_units,
        }
    }
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
            sticky_bindings: BTreeMap::new(),
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

    /// Project every seat into an account-shaped [`RedactedSeatStatus`] for
    /// admin status read surfaces. Secret handles never cross this boundary.
    pub fn redacted_seat_statuses(&self, now: Instant) -> Vec<RedactedSeatStatus> {
        self.seats
            .values()
            .map(|seat| RedactedSeatStatus {
                tenant_id: self.tenant_id.as_str().to_string(),
                provider: self.provider,
                seat_id: seat.seat_id.as_str().to_string(),
                state: match seat.state {
                    SubscriptionState::Authorized => "authorized",
                    SubscriptionState::ActiveUntilExpiry { .. }
                    | SubscriptionState::RefreshingToken { .. }
                    | SubscriptionState::Active => "active",
                    SubscriptionState::Cooldown { .. } => "cooldown",
                    SubscriptionState::Blacklisted { .. } => "blacklisted",
                    SubscriptionState::Halted { .. } => "halted",
                },
                headroom_percent: (seat.headroom(now, 1) * 100.0).clamp(0.0, 100.0),
            })
            .collect()
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
        principal_tenant: &TenantId,
        agent_id: &AgentId,
        gate: &dyn AuthzGate,
        now: Instant,
    ) -> Result<SeatLease, SubscriptionPoolError> {
        Self::lease_with_estimate(pool_ref, principal_tenant, agent_id, gate, now, 1)
    }

    /// Acquire a [`SeatLease`] and reserve the estimated provider quota units.
    pub fn lease_with_estimate(
        pool_ref: &Arc<Mutex<SubscriptionPool>>,
        principal_tenant: &TenantId,
        agent_id: &AgentId,
        gate: &dyn AuthzGate,
        now: Instant,
        estimated_units: u64,
    ) -> Result<SeatLease, SubscriptionPoolError> {
        let seat_id = {
            let mut pool = pool_ref
                .lock()
                .map_err(|_| SubscriptionPoolError::NoEligibleSeat)?;
            let sid = pool.select_excluding_leased(principal_tenant, agent_id, gate, now, estimated_units)?;
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

    pub fn lease_sticky_with_estimate(
        pool_ref: &Arc<Mutex<SubscriptionPool>>,
        principal_tenant: &TenantId,
        agent_id: &AgentId,
        gate: &dyn AuthzGate,
        now: Instant,
        spec: StickyLeaseSpec<'_>,
    ) -> Result<SeatLease, SubscriptionPoolError> {
        let seat_id = {
            let mut pool = pool_ref
                .lock()
                .map_err(|_| SubscriptionPoolError::NoEligibleSeat)?;
            pool.expire_sticky_bindings(now);

            let sticky_seat = pool
                .sticky_bindings
                .get(spec.sticky_key)
                .filter(|binding| binding.expires_at > now)
                .map(|binding| binding.seat_id.clone())
                .filter(|seat_id| {
                    pool.is_selectable(seat_id, now, spec.estimated_units, true)
                        && pool.authz_allows(principal_tenant, agent_id, gate).is_ok()
                });

            let sid = match sticky_seat {
                Some(seat_id) => seat_id,
                None => pool.select_excluding_leased(
                    principal_tenant,
                    agent_id,
                    gate,
                    now,
                    spec.estimated_units,
                )?,
            };
            pool.reserve_lease(&sid, spec.estimated_units)?;
            pool.sticky_bindings.insert(
                spec.sticky_key.to_string(),
                StickyBinding {
                    seat_id: sid.clone(),
                    expires_at: now + spec.ttl,
                },
            );
            sid
        };
        Ok(SeatLease {
            seat_id,
            pool: Arc::clone(pool_ref),
            completed: false,
            reserved_units: spec.estimated_units,
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

    pub fn seat_headroom(
        &self,
        seat_id: &SeatId,
        now: Instant,
        estimated_units: u64,
    ) -> Option<f64> {
        self.seats
            .get(seat_id)
            .map(|seat| seat.headroom(now, estimated_units))
    }

    /// Select the next eligible seat for an inbound request (D1).
    ///
    /// D7 contract: the [`AuthzGate`] is consulted exactly once per call,
    /// before any seat is returned. Forbid wins regardless of pool capacity.
    pub fn select(
        &mut self,
        principal_tenant: &TenantId,
        agent_id: &AgentId,
        gate: &dyn AuthzGate,
        now: Instant,
    ) -> Result<SeatId, SubscriptionPoolError> {
        self.select_with_estimate(principal_tenant, agent_id, gate, now, 1)
    }

    pub fn select_with_estimate(
        &mut self,
        principal_tenant: &TenantId,
        agent_id: &AgentId,
        gate: &dyn AuthzGate,
        now: Instant,
        estimated_units: u64,
    ) -> Result<SeatId, SubscriptionPoolError> {
        self.select_candidate(principal_tenant, agent_id, gate, now, estimated_units, false)
    }

    fn select_candidate(
        &mut self,
        principal_tenant: &TenantId,
        agent_id: &AgentId,
        gate: &dyn AuthzGate,
        now: Instant,
        estimated_units: u64,
        exclude_leased: bool,
    ) -> Result<SeatId, SubscriptionPoolError> {
        self.authz_allows(principal_tenant, agent_id, gate)?;

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
        if !matches!(outcome, SeatOutcome::Released | SeatOutcome::Ok) {
            self.remove_sticky_bindings_for_seat(seat_id);
        }
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

    /// Apply an SC8 overage [`GuardDecision`] to a seat.
    ///
    /// `Continue` / `Warn` leave seat state untouched — warn mode is
    /// observability-only and the caller emits the event. `Halt` transitions
    /// the seat to [`SubscriptionState::Halted`] and evicts its sticky bindings
    /// so traffic re-routes immediately. Returns `Ok(true)` iff the seat was
    /// halted. Overage halts do NOT touch `failure_count` — they are a billing
    /// circuit-break, not a credential-health failure, so they never escalate
    /// the seat toward [`SubscriptionState::Blacklisted`].
    pub fn apply_overage_decision(
        &mut self,
        seat_id: &SeatId,
        decision: &GuardDecision,
        _now: Instant,
    ) -> Result<bool, SubscriptionPoolError> {
        let GuardDecision::Halt { reason, resume_at } = decision else {
            return Ok(false);
        };
        self.remove_sticky_bindings_for_seat(seat_id);
        let Some(seat) = self.seats.get_mut(seat_id) else {
            return Err(SubscriptionPoolError::NoEligibleSeat);
        };
        seat.state = SubscriptionState::Halted {
            resume_at: *resume_at,
            reason: reason.clone(),
        };
        Ok(true)
    }

    /// Operator admin-resume: force a halted seat back to [`SubscriptionState::Active`]
    /// ahead of its cooldown horizon. Returns `Ok(true)` if the seat was halted
    /// and is now active, `Ok(false)` if the seat was not halted (no-op).
    pub fn admin_resume(&mut self, seat_id: &SeatId) -> Result<bool, SubscriptionPoolError> {
        let Some(seat) = self.seats.get_mut(seat_id) else {
            return Err(SubscriptionPoolError::NoEligibleSeat);
        };
        if matches!(seat.state, SubscriptionState::Halted { .. }) {
            seat.state = SubscriptionState::Active;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Like [`select`] but also excludes currently-leased seats.
    fn select_excluding_leased(
        &mut self,
        principal_tenant: &TenantId,
        agent_id: &AgentId,
        gate: &dyn AuthzGate,
        now: Instant,
        estimated_units: u64,
    ) -> Result<SeatId, SubscriptionPoolError> {
        self.select_candidate(principal_tenant, agent_id, gate, now, estimated_units, true)
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

    fn authz_allows(
        &self,
        principal_tenant: &TenantId,
        agent_id: &AgentId,
        gate: &dyn AuthzGate,
    ) -> Result<(), SubscriptionPoolError> {
        // ponytail: defense-in-depth backstop. `resource_tenant` stays the pool's
        // own tenant; `principal_tenant` is now the server-VERIFIED caller tenant
        // threaded down from the REST boundary (AUTH-005 / ADR-0573). When a
        // tenant-B caller is mis-routed to a tenant-A pool, the gate sees
        // principal=B vs resource=A and forbids — so a cross-tenant leak is
        // caught here even if the pool keying above it is wrong. The PRIMARY
        // barrier remains keying the pool by the verified principal tenant.
        let request = AuthzRequest {
            principal_tenant,
            principal_agent: agent_id,
            action: AuthzAction::SelectSeat,
            resource_tenant: &self.tenant_id,
            resource_provider: self.provider,
        };
        if gate.decide(&request) == AuthzDecision::Forbid {
            return Err(SubscriptionPoolError::ForbiddenByPolicy);
        }
        Ok(())
    }

    fn expire_sticky_bindings(&mut self, now: Instant) {
        self.sticky_bindings
            .retain(|_, binding| binding.expires_at > now);
    }

    fn remove_sticky_bindings_for_seat(&mut self, seat_id: &SeatId) {
        self.sticky_bindings
            .retain(|_, binding| binding.seat_id != *seat_id);
    }

    fn is_eligible(state: &SubscriptionState, now: Instant) -> bool {
        match state {
            SubscriptionState::Active => true,
            SubscriptionState::ActiveUntilExpiry { expires_at } => *expires_at > now,
            SubscriptionState::Cooldown { until, .. } => *until <= now,
            // Cooldown-resume: a halt with a horizon self-heals once it passes.
            // An admin-resume-only halt (resume_at None) stays ineligible.
            SubscriptionState::Halted { resume_at, .. } => {
                resume_at.is_some_and(|until| until <= now)
            }
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
    /// Secret-provider or transient OAuth refresh failure (not a permanent revocation).
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
/// credential. Accepts only `secret-ref://` or `kms-ref://` schemes
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
    let has_allowed_scheme =
        lowered.starts_with("secret-ref://") || lowered.starts_with("kms-ref://");
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
    fn redacted_seat_statuses_are_account_shaped_without_secret_handles() {
        let mut pool = SubscriptionPool::new(
            tenant("tenant-a"),
            Provider::Anthropic,
            SelectionStrategy::RoundRobin,
        );
        pool.add_seat(active_subscription(
            tenant("tenant-a"),
            seat("seat-a"),
            Provider::Anthropic,
            "secret-ref://tenant-a/anthropic",
        ))
        .expect("seat inserted");

        let statuses = pool.redacted_seat_statuses(Instant::now());
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].tenant_id, "tenant-a");
        assert_eq!(statuses[0].provider, Provider::Anthropic);
        assert_eq!(statuses[0].seat_id, "seat-a");
        assert_eq!(statuses[0].state, "active");
        assert!(statuses[0].headroom_percent >= 0.0);
        assert!(statuses[0].headroom_percent <= 100.0);
        let debug = format!("{statuses:?}");
        assert!(!debug.contains("secret-ref://tenant-a/anthropic"));
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
            "secret-ref://tenant-b/anthropic",
        );
        let wrong_provider = active_subscription(
            tenant("tenant-a"),
            seat("seat-codex"),
            Provider::Codex,
            "secret-ref://tenant-a/codex",
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
            "secret-ref://tenant-a/about-to-reset",
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
            "secret-ref://tenant-a/early-window",
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
            pool.select(&tenant("tenant-a"), &agent("agent-a"), &AllowGate, now)
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
            "secret-ref://tenant-a/codex-exhausted",
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
            pool.select(&tenant("tenant-a"), &agent("agent-a"), &AllowGate, now),
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
                &format!("secret-ref://tenant-a/{sid}"),
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
            SubscriptionPool::lease_with_estimate(
                &pool,
                &tenant("tenant-a"),
                &agent("agent-a"),
                &AllowGate,
                now,
                90,
            )
            .expect("first lease");
        assert_eq!(first.seat_id(), &seat("seat-a"));
        assert_eq!(
            pool.lock()
                .expect("pool lock")
                .seat_inflight_units(&seat("seat-a")),
            Some(90)
        );

        let second =
            SubscriptionPool::lease_with_estimate(
                &pool,
                &tenant("tenant-a"),
                &agent("agent-b"),
                &AllowGate,
                now,
                90,
            )
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
            "secret-ref://tenant-a/reset",
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
            SubscriptionPool::lease_with_estimate(
                &pool,
                &tenant("tenant-a"),
                &agent("agent-a"),
                &AllowGate,
                now,
                10,
            )
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
            locked.select_with_estimate(&tenant("tenant-a"), &agent("agent-a"), &AllowGate, now, 91),
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
            "secret-ref://tenant-a/cooling",
        ))
        .expect("seat accepted");

        assert!(pool.has_eligible_seat(now));
        pool.record_outcome(&sid, SeatOutcome::RateLimited429, now)
            .expect("rate limit recorded");
        assert!(!pool.has_eligible_seat(now + Duration::from_secs(30)));
        assert!(pool.has_eligible_seat(now + Duration::from_secs(61)));
        assert_eq!(
            pool.select(
                &tenant("tenant-a"),
                &agent("agent-a"),
                &AllowGate,
                now + Duration::from_secs(61),
            )
            .expect("seat selected after cooldown"),
            sid
        );
    }
}
