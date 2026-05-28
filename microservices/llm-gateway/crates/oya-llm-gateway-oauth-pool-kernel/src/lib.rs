//! llm-gateway kernel — pure-Rust contracts for the OAuth subscription pool
//! (ADR-0384 Path B).
//!
//! Kernel scope:
//! - Identity value objects ([`TenantId`], [`AgentId`], [`SeatId`],
//!   [`SubscriptionId`], [`Provider`]).
//! - [`OAuthSubscription`] + [`SubscriptionState`] state machine
//!   (Authorized -> ActiveUntilExpiry -> RefreshingToken ->
//!   Active | Cooldown | Blacklisted).
//! - [`SubscriptionPool`] + [`SelectionStrategy`] (RoundRobin, FillFirst).
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

use std::collections::BTreeMap;
use std::fmt;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Identity value objects
// ---------------------------------------------------------------------------

/// Opaque tenant identifier. Per oyatie-dogfood-tenancy, Oyatie itself runs as
/// a tenant of its own llm-gateway; there is no internal bypass.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TenantId(String);

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
pub struct AgentId(String);

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
pub struct SeatId(String);

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
pub struct SubscriptionId(String);

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
/// llm-gateway-reference-repo-audit memory. Gemini = v2, Cursor = v3.
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

/// A single OAuth subscription (one tenant seat for one provider).
#[derive(Clone, Debug)]
pub struct OAuthSubscription {
    pub tenant_id: TenantId,
    pub seat_id: SeatId,
    pub subscription_id: SubscriptionId,
    pub provider: Provider,
    pub state: SubscriptionState,
    /// Opaque handle. The actual refresh token is stored envelope-encrypted in
    /// OpenBao (D8) and never enters the kernel.
    pub refresh_token_handle: String,
    pub failure_count: u32,
}

// ---------------------------------------------------------------------------
// Authorization seam (D7)
// ---------------------------------------------------------------------------

/// Authorization decision principal: tenant + agent + the resource (target
/// subscription). Cedar adapter consumes this and returns [`AuthzDecision`].
#[derive(Clone, Debug)]
pub struct AuthzRequest<'a> {
    pub principal_tenant: &'a TenantId,
    pub principal_agent: &'a AgentId,
    pub action: AuthzAction,
    pub resource_tenant: &'a TenantId,
    pub resource_provider: Provider,
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

/// D6 event shape — every llm-gateway request emits one of these to the
/// configured sink. ClickHouse OLAP (ADR-0193) and Valkey Stream consume the
/// same shape via separate adapter crates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LlmGatewayEvent {
    pub request_id: String,
    pub tenant_id: TenantId,
    pub agent_id: AgentId,
    pub seat_id: SeatId,
    pub provider: Provider,
    pub model: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub ms_latency: u64,
    pub status: EventStatus,
    pub timestamp_unix_ms: u64,
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubscriptionPoolError {
    InvalidTenantId,
    InvalidAgentId,
    InvalidSeatId,
    InvalidSubscriptionId,
    NoEligibleSeat,
    ForbiddenByPolicy,
}

/// The kernel pool. One pool per (tenant, provider).
pub struct SubscriptionPool {
    tenant_id: TenantId,
    provider: Provider,
    strategy: SelectionStrategy,
    seats: BTreeMap<SeatId, OAuthSubscription>,
    round_robin_cursor: usize,
    cooldown_duration_429: Duration,
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
        self.seats.insert(seat_id, subscription);
        Ok(())
    }

    pub fn seat_count(&self) -> usize {
        self.seats.len()
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
                    let seat = &self.seats[sid];
                    if Self::is_eligible(&seat.state, now) {
                        return Ok(sid.clone());
                    }
                }
                Err(SubscriptionPoolError::NoEligibleSeat)
            }
            SelectionStrategy::RoundRobin => {
                for offset in 0..n {
                    let idx = (self.round_robin_cursor + offset) % n;
                    let sid = &seat_ids[idx];
                    let seat = &self.seats[sid];
                    if Self::is_eligible(&seat.state, now) {
                        self.round_robin_cursor = (idx + 1) % n;
                        return Ok(sid.clone());
                    }
                }
                Err(SubscriptionPoolError::NoEligibleSeat)
            }
        }
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
        }
        Ok(())
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
}
