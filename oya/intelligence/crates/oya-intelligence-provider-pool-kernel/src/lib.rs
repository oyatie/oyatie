//! M02-P02-IP-001 — ProviderAccountPool pure-value kernel.
//!
//! Rust counterpart to ccproxy-api `credential_balancer/manager.py` rotation +
//! health logic, refactored to pure value objects so the runtime stays
//! adapter-agnostic (MASTERPLAN Directive 4) and final-shape from day one
//! (Directive 3).
//!
//! No I/O. No async. No allocator games. Pool is a thin coordination layer —
//! it stores no account-level state, only membership + routing strategy +
//! verdict. Pool kernel sits *above* the P00 account state machine — it never
//! mutates ProviderAccount; it only reads usage snapshots, applies a routing
//! strategy, and emits a `PoolRoutingDecision { account_id, reason,
//! fallback_chain }`. This isolation is what lets the adapter crates
//! (IP-002 Anthropic-compat, IP-003 OpenAI-compat) share one rotation kernel
//! without provider-specific branching.
//!
//! Linus good-taste: eliminated the special-case branch "single-account == no
//! pool" by representing single-account as a pool of size 1 with `RoundRobin`.
//! The `pick_account` function has no `if members.len() == 1` branch — the
//! data shape removes it.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub use oya_intelligence_account_kernel::{ProviderFamily, SecretReference};

/// data_class: INTERNAL_ONLY
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct PoolId(pub String); // data_class: INTERNAL_ONLY

/// data_class: TENANT_SCOPED
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct TenantId(pub String); // data_class: TENANT_SCOPED

/// data_class: TENANT_SCOPED — identifies a ProviderAccount row in P00.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ProviderAccountId(pub String); // data_class: TENANT_SCOPED

/// data_class: INTERNAL_ONLY — opaque session anchor for `Sticky` routing.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct SessionId(pub String); // data_class: INTERNAL_ONLY

/// data_class: INTERNAL_ONLY — pool tier (paid/free/team/enterprise).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum ProviderTier {
    Free,
    Pro,
    Team,
    Enterprise,
}

/// data_class: INTERNAL_ONLY — opaque ToS-ack record id (IP-006 anchor).
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct TosAckId(pub String); // data_class: INTERNAL_ONLY

/// Routing strategy. ccproxy-api `manager.py` parity + 3 net-new strategies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PoolRoutingStrategy {
    RoundRobin,
    LeastUsed,
    LeastLatency,
    LeastRemaining,
    Sticky(SessionId),
}

/// Reason a particular account was picked.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PoolRoutingReason {
    Healthy,
    FailoverFrom(ProviderAccountId),
    Sticky,
    QuotaPreserve,
    LeastUsedTieBreak,
}

impl PoolRoutingReason {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::FailoverFrom(_) => "failover_from",
            Self::Sticky => "sticky",
            Self::QuotaPreserve => "quota_preserve",
            Self::LeastUsedTieBreak => "least_used_tie_break",
        }
    }
}

/// Membership delta event emitted when the pool changes shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PoolMembershipChange {
    Added(ProviderAccountId),       // data_class: TENANT_SCOPED
    Removed(ProviderAccountId),     // data_class: TENANT_SCOPED
    Quarantined(ProviderAccountId), // data_class: TENANT_SCOPED
}

/// data_class: INTERNAL_ONLY — duration in milliseconds (kernel std-only;
/// avoids pulling `time` 0.3 into the kernel — adapters can convert as
/// needed).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct DurationMs(pub u64); // data_class: INTERNAL_ONLY

/// data_class: INTERNAL_ONLY — pure unix-millis timestamp.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct UnixMillis(pub u64); // data_class: INTERNAL_ONLY

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderAccountPool {
    pub id: PoolId,                               // data_class: INTERNAL_ONLY
    pub provider: ProviderFamily,                 // data_class: INTERNAL_ONLY
    pub tier: ProviderTier,                       // data_class: INTERNAL_ONLY
    pub tenant_id: TenantId,                      // data_class: TENANT_SCOPED
    pub members: BTreeSet<ProviderAccountId>,     // data_class: TENANT_SCOPED
    pub routing_strategy: PoolRoutingStrategy,    // data_class: INTERNAL_ONLY
    pub anti_correlation_window_ms: DurationMs,   // data_class: INTERNAL_ONLY
    pub tos_acknowledgment_ref: Option<TosAckId>, // data_class: INTERNAL_ONLY
}

/// Request metadata supplied to `pick_account`. Carries the inbound shape and
/// (optionally) a sticky-session anchor — no provider-specific fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestMetadata {
    pub session: Option<SessionId>, // data_class: INTERNAL_ONLY
    pub previous_account: Option<ProviderAccountId>, // data_class: TENANT_SCOPED
    pub model_hint: String,         // data_class: INTERNAL_ONLY
}

impl RequestMetadata {
    pub fn new(model_hint: String) -> Self {
        Self {
            session: None,
            previous_account: None,
            model_hint,
        }
    }
}

/// Per-account usage view. Pure snapshot — kernel never mutates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UsageSnapshot {
    pub requests_in_window: u64,       // data_class: INTERNAL_ONLY
    pub remaining_quota_pct: u8,       // data_class: INTERNAL_ONLY (0..=100)
    pub last_used_unix_ms: UnixMillis, // data_class: INTERNAL_ONLY
    pub p99_latency_ms: u32,           // data_class: INTERNAL_ONLY
}

impl UsageSnapshot {
    pub const fn zero() -> Self {
        Self {
            requests_in_window: 0,
            remaining_quota_pct: 100,
            last_used_unix_ms: UnixMillis(0),
            p99_latency_ms: 0,
        }
    }
}

/// Per-account health view. Pure snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Classification of the failure that triggered a quarantine event.
///
/// Used by `CooldownPolicy::window_for` to select the per-failure-kind
/// exponential backoff window.
///
/// data_class: INTERNAL_ONLY
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum FailureKind {
    /// HTTP 429 / upstream rate-limit response.
    UpstreamRateLimit429,
    /// HTTP 5xx / upstream server error.
    UpstreamServerError5xx,
    /// TCP/TLS connection could not be established within deadline.
    ConnectionTimeout,
    /// Credential was rejected by the upstream (401/403).
    AuthFailure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccountHealth {
    pub state: HealthState,        // data_class: INTERNAL_ONLY
    pub consecutive_failures: u32, // data_class: INTERNAL_ONLY
    /// Absolute epoch (unix-millis) at which the account exits cooldown,
    /// as computed by the caller from `CooldownPolicy::window_for`.
    ///
    /// `None` means the account has not been quarantined.
    ///
    /// Routing decisions in `pick_account_with_cooldown` are driven by
    /// `QuarantineMap`, not this field; the field is informational so callers
    /// can embed the expiry directly in a health snapshot.
    ///
    /// data_class: INTERNAL_ONLY
    pub cooldown_until: Option<UnixMillis>,
}

impl AccountHealth {
    pub const fn healthy() -> Self {
        Self {
            state: HealthState::Healthy,
            consecutive_failures: 0,
            cooldown_until: None,
        }
    }
}

/// Per-account quarantine timestamps, keyed by [`ProviderAccountId`].
///
/// Carried separately from [`AccountHealth`] so that adding cooldown support
/// is a non-breaking addition: existing callers that only construct
/// `AccountHealth` literals are unaffected.
///
/// `None` (absent key) means the account was never quarantined and is therefore
/// never considered in cooldown regardless of the [`CooldownPolicy`] window.
///
/// data_class: INTERNAL_ONLY
pub type QuarantineMap = BTreeMap<ProviderAccountId, UnixMillis>;

/// data_class: INTERNAL_ONLY — encapsulates the cooldown window and the
/// evaluation instant so callers pass a single, self-describing input rather
/// than two separate scalars.
///
/// Cooldown semantics: an account is *in cooldown* when its entry in the
/// [`QuarantineMap`] satisfies
/// `now.0.saturating_sub(quarantined_at.0) < window_ms.0`.
/// An account absent from the map (never quarantined) is never in cooldown.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CooldownPolicy {
    /// Width of the anti-correlation window.  Derived from
    /// `ProviderAccountPool::anti_correlation_window_ms`.
    pub window_ms: DurationMs, // data_class: INTERNAL_ONLY
    /// The evaluation instant (caller's "now").
    pub now: UnixMillis, // data_class: INTERNAL_ONLY
}

impl CooldownPolicy {
    /// Build a `CooldownPolicy` from the pool's configured window and the
    /// current `UnixMillis`.
    pub fn from_pool(pool: &ProviderAccountPool, now: UnixMillis) -> Self {
        Self {
            window_ms: pool.anti_correlation_window_ms,
            now,
        }
    }

    /// Returns `true` when `account_id` is still inside the anti-correlation
    /// window according to `quarantines` (i.e. should be excluded from routing).
    ///
    /// An account absent from `quarantines` is never considered in cooldown.
    pub fn in_cooldown(&self, account_id: &ProviderAccountId, quarantines: &QuarantineMap) -> bool {
        match quarantines.get(account_id) {
            None => false,
            Some(quarantined_at) => self.now.0.saturating_sub(quarantined_at.0) < self.window_ms.0,
        }
    }

    /// Returns the per-`FailureKind` exponential backoff window for
    /// `consecutive_failures` (1-indexed; 0 is treated as 1).
    ///
    /// Backoff tables (values in milliseconds):
    ///
    /// | FailureKind            | f=1    | f=2     | f=3     | f=4+    |
    /// |------------------------|--------|---------|---------|---------|
    /// | UpstreamRateLimit429   | 30_000 | 60_000  | 120_000 | 300_000 |
    /// | UpstreamServerError5xx | 10_000 | 30_000  | 60_000  | 60_000  |
    /// | ConnectionTimeout      |  5_000 | 15_000  | 30_000  | 30_000  |
    /// | AuthFailure            | 60_000 | 300_000 | 900_000 | 900_000 |
    pub fn window_for(kind: FailureKind, consecutive_failures: u32) -> DurationMs {
        // Treat 0 as first failure tier.
        let tier = consecutive_failures.max(1) as usize;
        let ms = match kind {
            FailureKind::UpstreamRateLimit429 => {
                const TABLE: [u64; 4] = [30_000, 60_000, 120_000, 300_000];
                TABLE[(tier - 1).min(TABLE.len() - 1)]
            }
            FailureKind::UpstreamServerError5xx => {
                const TABLE: [u64; 3] = [10_000, 30_000, 60_000];
                TABLE[(tier - 1).min(TABLE.len() - 1)]
            }
            FailureKind::ConnectionTimeout => {
                const TABLE: [u64; 3] = [5_000, 15_000, 30_000];
                TABLE[(tier - 1).min(TABLE.len() - 1)]
            }
            FailureKind::AuthFailure => {
                const TABLE: [u64; 3] = [60_000, 300_000, 900_000];
                TABLE[(tier - 1).min(TABLE.len() - 1)]
            }
        };
        DurationMs(ms)
    }
}

pub type UsageSnapshotMap = BTreeMap<ProviderAccountId, UsageSnapshot>;
pub type AccountHealthMap = BTreeMap<ProviderAccountId, AccountHealth>;

/// data_class: TENANT_SCOPED — per-agent caller identity used for seat leases.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct AgentId(pub String); // data_class: TENANT_SCOPED

/// data_class: TENANT_SCOPED — one provider subscription seat in a pool.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct SeatId(pub String); // data_class: TENANT_SCOPED

/// data_class: INTERNAL_ONLY — stable subscription record identifier.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct SubscriptionId(pub String); // data_class: INTERNAL_ONLY

/// data_class: INTERNAL_ONLY — unique reservation marker generated by the pool.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct LeaseId(pub String); // data_class: INTERNAL_ONLY

/// Pure subscription-seat selection strategy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubscriptionPoolStrategy {
    /// Stable-order rotation across eligible seats.
    RoundRobin,
    /// Stable-order first eligible seat wins.
    FillFirst,
}

/// Terminal reasons that remove a seat from automatic routing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SeatBlacklistReason {
    RefreshTokenExhausted,
    RepeatedFailuresExceededThreshold {
        failure_count: u32, // data_class: INTERNAL_ONLY
    },
    OperatorAction,
}

/// Pure state for one provider subscription seat.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubscriptionSeatState {
    Active,
    Reserved {
        lease_id: LeaseId,         // data_class: INTERNAL_ONLY
        agent_id: AgentId,         // data_class: TENANT_SCOPED
        since_unix_ms: UnixMillis, // data_class: INTERNAL_ONLY
    },
    Cooldown {
        until_unix_ms: UnixMillis, // data_class: INTERNAL_ONLY
        reason: FailureKind,       // data_class: INTERNAL_ONLY
    },
    Blacklisted {
        reason: SeatBlacklistReason, // data_class: INTERNAL_ONLY
    },
}

/// One OAuth-backed provider account seat.
///
/// The kernel carries only an opaque [`SecretReference`]; raw OAuth material
/// remains in the credential resolver / OpenBao adapter lane.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OAuthSubscription {
    pub seat_id: SeatId,                        // data_class: TENANT_SCOPED
    pub provider_account_id: ProviderAccountId, // data_class: TENANT_SCOPED
    pub subscription_id: SubscriptionId,        // data_class: INTERNAL_ONLY
    pub provider: ProviderFamily,               // data_class: INTERNAL_ONLY
    pub refresh_token_ref: SecretReference,     // data_class: SECRET_REF
    pub state: SubscriptionSeatState,           // data_class: INTERNAL_ONLY
    pub usage: UsageSnapshot,                   // data_class: INTERNAL_ONLY
    pub failure_count: u32,                     // data_class: INTERNAL_ONLY
}

impl OAuthSubscription {
    pub fn new(
        seat_id: SeatId,
        provider_account_id: ProviderAccountId,
        subscription_id: SubscriptionId,
        provider: ProviderFamily,
        refresh_token_ref: SecretReference,
        state: SubscriptionSeatState,
        usage: UsageSnapshot,
    ) -> Self {
        Self {
            seat_id,
            provider_account_id,
            subscription_id,
            provider,
            refresh_token_ref,
            state,
            usage,
            failure_count: 0,
        }
    }
}

/// Outcome recorded by the app layer after a seat-backed request attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SeatOutcome {
    Succeeded {
        usage: UsageSnapshot, // data_class: INTERNAL_ONLY
    },
    RetryableFailure {
        kind: FailureKind, // data_class: INTERNAL_ONLY
    },
    RefreshTokenExhausted,
    OperatorBlacklisted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubscriptionPoolError {
    ProviderMismatch {
        expected: ProviderFamily, // data_class: INTERNAL_ONLY
        actual: ProviderFamily,   // data_class: INTERNAL_ONLY
    },
    DuplicateSeat(SeatId),
    SeatNotFound(SeatId),
    PoolExhausted,
    LeaseSeatMismatch {
        seat_id: SeatId,   // data_class: TENANT_SCOPED
        lease_id: LeaseId, // data_class: INTERNAL_ONLY
    },
    SeatNotReserved(SeatId),
}

impl fmt::Display for SubscriptionPoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProviderMismatch { expected, actual } => {
                write!(
                    f,
                    "subscription provider mismatch: expected {expected:?}, actual {actual:?}"
                )
            }
            Self::DuplicateSeat(SeatId(seat_id)) => {
                write!(f, "duplicate subscription seat: {seat_id}")
            }
            Self::SeatNotFound(SeatId(seat_id)) => {
                write!(f, "subscription seat not found: {seat_id}")
            }
            Self::PoolExhausted => write!(f, "subscription pool exhausted"),
            Self::LeaseSeatMismatch {
                seat_id: SeatId(seat_id),
                lease_id: LeaseId(lease_id),
            } => write!(f, "lease {lease_id} is not active on seat {seat_id}"),
            Self::SeatNotReserved(SeatId(seat_id)) => {
                write!(f, "subscription seat is not reserved: {seat_id}")
            }
        }
    }
}

/// data_class: INTERNAL_ONLY — value object returned by [`SubscriptionPool::lease`].
#[derive(Debug, Eq, PartialEq)]
pub struct SeatLease {
    pub lease_id: LeaseId,                      // data_class: INTERNAL_ONLY
    pub seat_id: SeatId,                        // data_class: TENANT_SCOPED
    pub provider_account_id: ProviderAccountId, // data_class: TENANT_SCOPED
    pub agent_id: AgentId,                      // data_class: TENANT_SCOPED
    pub leased_at_unix_ms: UnixMillis,          // data_class: INTERNAL_ONLY
}

/// Pure provider-subscription pool. One pool is scoped to one tenant/provider.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionPool {
    pub id: PoolId,                                 // data_class: INTERNAL_ONLY
    pub tenant_id: TenantId,                        // data_class: TENANT_SCOPED
    pub provider: ProviderFamily,                   // data_class: INTERNAL_ONLY
    pub strategy: SubscriptionPoolStrategy,         // data_class: INTERNAL_ONLY
    pub seats: BTreeMap<SeatId, OAuthSubscription>, // data_class: TENANT_SCOPED
    round_robin_cursor: usize,                      // data_class: INTERNAL_ONLY
    next_lease_sequence: u64,                       // data_class: INTERNAL_ONLY
}

const SUBSCRIPTION_BLACKLIST_FAILURE_THRESHOLD: u32 = 5;

impl SubscriptionPool {
    pub fn new(
        id: PoolId,
        tenant_id: TenantId,
        provider: ProviderFamily,
        strategy: SubscriptionPoolStrategy,
    ) -> Self {
        Self {
            id,
            tenant_id,
            provider,
            strategy,
            seats: BTreeMap::new(),
            round_robin_cursor: 0,
            next_lease_sequence: 0,
        }
    }

    pub fn add_subscription(
        &mut self,
        subscription: OAuthSubscription,
    ) -> Result<(), SubscriptionPoolError> {
        if subscription.provider != self.provider {
            return Err(SubscriptionPoolError::ProviderMismatch {
                expected: self.provider,
                actual: subscription.provider,
            });
        }
        if self.seats.contains_key(&subscription.seat_id) {
            return Err(SubscriptionPoolError::DuplicateSeat(
                subscription.seat_id.clone(),
            ));
        }
        self.seats
            .insert(subscription.seat_id.clone(), subscription);
        Ok(())
    }

    pub fn seat_count(&self) -> usize {
        self.seats.len()
    }

    pub fn reserved_count(&self) -> usize {
        self.seats
            .values()
            .filter(|seat| matches!(seat.state, SubscriptionSeatState::Reserved { .. }))
            .count()
    }

    pub fn free_count(&self) -> usize {
        self.seat_count().saturating_sub(self.reserved_count())
    }

    pub fn eligible_count(&self, now: UnixMillis) -> usize {
        self.seats
            .keys()
            .filter(|seat_id| self.is_eligible(seat_id, now))
            .count()
    }

    pub fn seat_state(&self, seat_id: &SeatId) -> Option<&SubscriptionSeatState> {
        self.seats.get(seat_id).map(|seat| &seat.state)
    }

    pub fn usage_snapshot(&self, seat_id: &SeatId) -> Option<UsageSnapshot> {
        self.seats.get(seat_id).map(|seat| seat.usage)
    }

    pub fn lease(
        &mut self,
        agent_id: AgentId,
        now: UnixMillis,
    ) -> Result<SeatLease, SubscriptionPoolError> {
        let seat_id = self.select_seat(now)?;
        self.next_lease_sequence = self.next_lease_sequence.saturating_add(1);
        let lease_id = LeaseId(format!(
            "{}:{}:{}",
            self.id.0, seat_id.0, self.next_lease_sequence
        ));
        let seat = self
            .seats
            .get_mut(&seat_id)
            .ok_or_else(|| SubscriptionPoolError::SeatNotFound(seat_id.clone()))?;
        seat.state = SubscriptionSeatState::Reserved {
            lease_id: lease_id.clone(),
            agent_id: agent_id.clone(),
            since_unix_ms: now,
        };

        Ok(SeatLease {
            lease_id,
            seat_id,
            provider_account_id: seat.provider_account_id.clone(),
            agent_id,
            leased_at_unix_ms: now,
        })
    }

    pub fn complete(
        &mut self,
        lease: SeatLease,
        outcome: SeatOutcome,
        now: UnixMillis,
    ) -> Result<(), SubscriptionPoolError> {
        let seat = self
            .seats
            .get_mut(&lease.seat_id)
            .ok_or_else(|| SubscriptionPoolError::SeatNotFound(lease.seat_id.clone()))?;

        match &seat.state {
            SubscriptionSeatState::Reserved { lease_id, .. } if lease_id == &lease.lease_id => {}
            SubscriptionSeatState::Reserved { .. } => {
                return Err(SubscriptionPoolError::LeaseSeatMismatch {
                    seat_id: lease.seat_id,
                    lease_id: lease.lease_id,
                });
            }
            _ => return Err(SubscriptionPoolError::SeatNotReserved(lease.seat_id)),
        }

        match outcome {
            SeatOutcome::Succeeded { usage } => {
                seat.failure_count = 0;
                seat.usage = usage;
                seat.state = SubscriptionSeatState::Active;
            }
            SeatOutcome::RetryableFailure { kind } => {
                seat.failure_count = seat.failure_count.saturating_add(1);
                if seat.failure_count > SUBSCRIPTION_BLACKLIST_FAILURE_THRESHOLD {
                    seat.state = SubscriptionSeatState::Blacklisted {
                        reason: SeatBlacklistReason::RepeatedFailuresExceededThreshold {
                            failure_count: seat.failure_count,
                        },
                    };
                } else {
                    let window = CooldownPolicy::window_for(kind, seat.failure_count);
                    seat.state = SubscriptionSeatState::Cooldown {
                        until_unix_ms: UnixMillis(now.0.saturating_add(window.0)),
                        reason: kind,
                    };
                }
            }
            SeatOutcome::RefreshTokenExhausted => {
                seat.state = SubscriptionSeatState::Blacklisted {
                    reason: SeatBlacklistReason::RefreshTokenExhausted,
                };
            }
            SeatOutcome::OperatorBlacklisted => {
                seat.state = SubscriptionSeatState::Blacklisted {
                    reason: SeatBlacklistReason::OperatorAction,
                };
            }
        }

        Ok(())
    }

    fn select_seat(&mut self, now: UnixMillis) -> Result<SeatId, SubscriptionPoolError> {
        if self.seats.is_empty() {
            return Err(SubscriptionPoolError::PoolExhausted);
        }

        let seat_ids: Vec<SeatId> = self.seats.keys().cloned().collect();
        let count = seat_ids.len();
        match self.strategy {
            SubscriptionPoolStrategy::FillFirst => seat_ids
                .into_iter()
                .find(|seat_id| self.is_eligible(seat_id, now))
                .ok_or(SubscriptionPoolError::PoolExhausted),
            SubscriptionPoolStrategy::RoundRobin => {
                for offset in 0..count {
                    let index = (self.round_robin_cursor + offset) % count;
                    let seat_id = &seat_ids[index];
                    if self.is_eligible(seat_id, now) {
                        self.round_robin_cursor = (index + 1) % count;
                        return Ok(seat_id.clone());
                    }
                }
                Err(SubscriptionPoolError::PoolExhausted)
            }
        }
    }

    fn is_eligible(&self, seat_id: &SeatId, now: UnixMillis) -> bool {
        let Some(seat) = self.seats.get(seat_id) else {
            return false;
        };
        if seat.usage.remaining_quota_pct < LEAST_REMAINING_MIN_PCT {
            return false;
        }
        match &seat.state {
            SubscriptionSeatState::Active => true,
            SubscriptionSeatState::Cooldown { until_unix_ms, .. } => *until_unix_ms <= now,
            SubscriptionSeatState::Reserved { .. } | SubscriptionSeatState::Blacklisted { .. } => {
                false
            }
        }
    }
}

/// Populate `quarantines` from a slice of `PoolMembershipChange` events.
///
/// For each `PoolMembershipChange::Quarantined(id)` in `changes`, inserts
/// `(id, now)` into `quarantines`, overwriting any stale entry.
/// `Added` and `Removed` variants are ignored.
///
/// This is the canonical bridge between the membership-change event surface
/// and the `QuarantineMap` consumed by `pick_account_with_cooldown`.
///
/// data_class: INTERNAL_ONLY
pub fn populate_quarantine_from_changes(
    changes: &[PoolMembershipChange],
    now: UnixMillis,
    quarantines: &mut QuarantineMap,
) {
    for change in changes {
        if let PoolMembershipChange::Quarantined(id) = change {
            quarantines.insert(id.clone(), now);
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolRoutingDecision {
    pub account_id: ProviderAccountId, // data_class: TENANT_SCOPED
    pub reason: PoolRoutingReason,     // data_class: INTERNAL_ONLY
    pub fallback_chain: Vec<ProviderAccountId>, // data_class: TENANT_SCOPED
    pub decided_at_unix_ms: UnixMillis, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PoolError {
    EmptyMembers,
    NoHealthyMembers,
    StickySessionNotFound(SessionId),
    RemainingQuotaThresholdNotMet,
}

impl fmt::Display for PoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMembers => write!(f, "pool has no members"),
            Self::NoHealthyMembers => write!(f, "no healthy members"),
            Self::StickySessionNotFound(SessionId(s)) => {
                write!(f, "sticky session not found: {s}")
            }
            Self::RemainingQuotaThresholdNotMet => {
                write!(f, "no member meets remaining-quota threshold")
            }
        }
    }
}

/// Minimum remaining-quota percentage for `LeastRemaining` to consider a
/// member eligible. Pure constant — adapters that want a different threshold
/// should pre-filter the usage map before calling `pick_account`.
pub const LEAST_REMAINING_MIN_PCT: u8 = 5;

impl ProviderAccountPool {
    pub fn new(
        id: PoolId,
        provider: ProviderFamily,
        tier: ProviderTier,
        tenant_id: TenantId,
        members: BTreeSet<ProviderAccountId>,
        routing_strategy: PoolRoutingStrategy,
        anti_correlation_window_ms: DurationMs,
    ) -> Self {
        Self {
            id,
            provider,
            tier,
            tenant_id,
            members,
            routing_strategy,
            anti_correlation_window_ms,
            tos_acknowledgment_ref: None,
        }
    }

    pub fn with_tos_ack(mut self, ack: TosAckId) -> Self {
        self.tos_acknowledgment_ref = Some(ack);
        self
    }

    pub fn size(&self) -> usize {
        self.members.len()
    }
}

/// Pure decision function — given pool, request, usage, and health snapshots,
/// emit a `PoolRoutingDecision`. Deterministic given identical inputs.
///
/// Failure modes:
/// - `PoolError::EmptyMembers` when membership set is empty.
/// - `PoolError::NoHealthyMembers` when every member is `Unhealthy`.
/// - `PoolError::StickySessionNotFound` when `Sticky(s)` mentions a session
///   whose previous-account binding is no longer a pool member.
/// - `PoolError::RemainingQuotaThresholdNotMet` when `LeastRemaining` finds
///   every member below `LEAST_REMAINING_MIN_PCT`.
pub fn pick_account(
    pool: &ProviderAccountPool,
    request: &RequestMetadata,
    usage: &UsageSnapshotMap,
    health: &AccountHealthMap,
    now: UnixMillis,
) -> Result<PoolRoutingDecision, PoolError> {
    if pool.members.is_empty() {
        return Err(PoolError::EmptyMembers);
    }

    // Filter to healthy members preserving BTreeSet ordering (stable, deterministic).
    let healthy: Vec<ProviderAccountId> = pool
        .members
        .iter()
        .filter(|m| {
            health
                .get(*m)
                .map(|h| h.state != HealthState::Unhealthy)
                .unwrap_or(true)
        })
        .cloned()
        .collect();

    if healthy.is_empty() {
        return Err(PoolError::NoHealthyMembers);
    }

    let (chosen, reason) = match &pool.routing_strategy {
        PoolRoutingStrategy::RoundRobin => round_robin(&healthy, request),
        PoolRoutingStrategy::LeastUsed => least_used(&healthy, usage),
        PoolRoutingStrategy::LeastLatency => least_latency(&healthy, usage),
        PoolRoutingStrategy::LeastRemaining => least_remaining(&healthy, usage)?,
        PoolRoutingStrategy::Sticky(session) => sticky(&healthy, session, request, usage)?,
    };

    // Build deterministic fallback chain: everyone except chosen, in pool order,
    // capped at members.len()-1.
    let fallback_chain: Vec<ProviderAccountId> =
        healthy.into_iter().filter(|m| m != &chosen).collect();

    Ok(PoolRoutingDecision {
        account_id: chosen,
        reason,
        fallback_chain,
        decided_at_unix_ms: now,
    })
}

/// Quota-aware cooldown/quarantine rotation entry point.
///
/// Extends `pick_account` with a time-windowed cooldown pre-filter: accounts
/// that are `HealthState::Unhealthy` **or** whose entry in `quarantines`
/// falls within `cooldown.window_ms` of `cooldown.now` are excluded before any
/// routing strategy is applied.
///
/// This ensures a quarantined high-quota account cannot be chosen over a
/// healthy lower-quota account (ST3 guarantee).
///
/// Failure modes are a superset of `pick_account`:
/// - `PoolError::EmptyMembers` — membership set is empty.
/// - `PoolError::NoHealthyMembers` — every member is unhealthy or in cooldown.
/// - `PoolError::StickySessionNotFound` — Sticky strategy with no resolvable anchor.
/// - `PoolError::RemainingQuotaThresholdNotMet` — LeastRemaining, all below threshold.
pub fn pick_account_with_cooldown(
    pool: &ProviderAccountPool,
    request: &RequestMetadata,
    usage: &UsageSnapshotMap,
    health: &AccountHealthMap,
    quarantines: &QuarantineMap,
    cooldown: CooldownPolicy,
) -> Result<PoolRoutingDecision, PoolError> {
    if pool.members.is_empty() {
        return Err(PoolError::EmptyMembers);
    }

    // Filter: healthy AND out-of-cooldown, preserving BTreeSet order.
    let eligible: Vec<ProviderAccountId> = pool
        .members
        .iter()
        .filter(|m| {
            let not_unhealthy = health
                .get(*m)
                .map(|h| h.state != HealthState::Unhealthy)
                .unwrap_or(true);
            // No health entry → treat as healthy; absent from quarantines → not in cooldown.
            not_unhealthy && !cooldown.in_cooldown(m, quarantines)
        })
        .cloned()
        .collect();

    if eligible.is_empty() {
        return Err(PoolError::NoHealthyMembers);
    }

    // Detect whether previous_account was excluded by cooldown/health filtering.
    // If so, any choice we make is an anti-correlation failover and must be
    // surfaced as FailoverFrom(prev) regardless of the routing strategy.
    let prev_was_excluded = request
        .previous_account
        .as_ref()
        .map(|prev| !eligible.contains(prev))
        .unwrap_or(false);

    let (chosen, reason) = match &pool.routing_strategy {
        PoolRoutingStrategy::RoundRobin => round_robin(&eligible, request),
        PoolRoutingStrategy::LeastUsed => least_used(&eligible, usage),
        PoolRoutingStrategy::LeastLatency => least_latency(&eligible, usage),
        PoolRoutingStrategy::LeastRemaining => least_remaining(&eligible, usage)?,
        PoolRoutingStrategy::Sticky(session) => sticky(&eligible, session, request, usage)?,
    };

    // Override reason: if previous account was filtered out (quarantine/unhealthy),
    // emit FailoverFrom(prev) so callers can observe the anti-correlation handoff.
    let reason = if prev_was_excluded {
        PoolRoutingReason::FailoverFrom(request.previous_account.clone().expect("checked above"))
    } else {
        reason
    };

    let fallback_chain: Vec<ProviderAccountId> =
        eligible.into_iter().filter(|m| m != &chosen).collect();

    Ok(PoolRoutingDecision {
        account_id: chosen,
        reason,
        fallback_chain,
        decided_at_unix_ms: cooldown.now,
    })
}

fn round_robin(
    healthy: &[ProviderAccountId],
    request: &RequestMetadata,
) -> (ProviderAccountId, PoolRoutingReason) {
    // If previous account was set and is in the healthy set, pick the *next* in
    // BTreeSet order (wrap to the first). Otherwise pick the first.
    if let Some(prev) = &request.previous_account
        && let Some(idx) = healthy.iter().position(|x| x == prev)
    {
        let next_idx = (idx + 1) % healthy.len();
        let chosen = healthy[next_idx].clone();
        let reason = if next_idx == idx {
            PoolRoutingReason::Healthy
        } else {
            PoolRoutingReason::FailoverFrom(prev.clone())
        };
        return (chosen, reason);
    }
    (healthy[0].clone(), PoolRoutingReason::Healthy)
}

fn least_used(
    healthy: &[ProviderAccountId],
    usage: &UsageSnapshotMap,
) -> (ProviderAccountId, PoolRoutingReason) {
    // ADR-0083 Tier 1: replace `Option<&T>` + `.expect("healthy not empty")` with
    // first-element seeding. Caller (`route`) guarantees `healthy.is_empty()`
    // returns `Err(PoolError::NoHealthyMembers)` before this fn runs, so seeding
    // from `healthy[0]` is the canonical non-panicking encoding of that
    // invariant.
    let mut best: &ProviderAccountId = &healthy[0];
    let mut best_used = usage.get(best).map(|s| s.requests_in_window).unwrap_or(0);
    let mut tied = false;
    for m in healthy.iter().skip(1) {
        let u = usage.get(m).map(|s| s.requests_in_window).unwrap_or(0);
        if u < best_used {
            best_used = u;
            best = m;
            tied = false;
        } else if u == best_used {
            tied = true;
        }
    }
    let reason = if tied {
        PoolRoutingReason::LeastUsedTieBreak
    } else {
        PoolRoutingReason::Healthy
    };
    (best.clone(), reason)
}

fn least_latency(
    healthy: &[ProviderAccountId],
    usage: &UsageSnapshotMap,
) -> (ProviderAccountId, PoolRoutingReason) {
    // ADR-0083 Tier 1: first-element seeding (caller-enforced non-empty).
    let mut best: &ProviderAccountId = &healthy[0];
    let mut best_p99 = usage
        .get(best)
        .map(|s| s.p99_latency_ms)
        .unwrap_or(u32::MAX);
    for m in healthy.iter().skip(1) {
        let p = usage.get(m).map(|s| s.p99_latency_ms).unwrap_or(u32::MAX);
        if p < best_p99 {
            best_p99 = p;
            best = m;
        }
    }
    (best.clone(), PoolRoutingReason::Healthy)
}

fn least_remaining(
    healthy: &[ProviderAccountId],
    usage: &UsageSnapshotMap,
) -> Result<(ProviderAccountId, PoolRoutingReason), PoolError> {
    // Pick the account with the *highest* remaining quota (preserves the highest
    // headroom across the pool). Members below `LEAST_REMAINING_MIN_PCT` are
    // excluded as failed-eligibility.
    let mut best: Option<&ProviderAccountId> = None;
    let mut best_pct: u8 = 0;
    for m in healthy {
        let pct = usage.get(m).map(|s| s.remaining_quota_pct).unwrap_or(100);
        if pct < LEAST_REMAINING_MIN_PCT {
            continue;
        }
        if pct > best_pct {
            best_pct = pct;
            best = Some(m);
        }
    }
    match best {
        Some(m) => Ok((m.clone(), PoolRoutingReason::QuotaPreserve)),
        None => Err(PoolError::RemainingQuotaThresholdNotMet),
    }
}

fn sticky(
    healthy: &[ProviderAccountId],
    session: &SessionId,
    request: &RequestMetadata,
    usage: &UsageSnapshotMap,
) -> Result<(ProviderAccountId, PoolRoutingReason), PoolError> {
    // Sticky semantics: if the request carries a previous_account and that
    // account is still a healthy pool member, keep using it. Otherwise fall
    // back to LeastUsed and treat the session as freshly bound. If the
    // request session and the pool's configured sticky session both diverge
    // and there's no previous binding at all, surface a not-found.
    if let Some(prev) = &request.previous_account
        && healthy.iter().any(|m| m == prev)
    {
        return Ok((prev.clone(), PoolRoutingReason::Sticky));
    }
    if request.session.as_ref() == Some(session) {
        let (chosen, _) = least_used(healthy, usage);
        return Ok((chosen, PoolRoutingReason::Sticky));
    }
    Err(PoolError::StickySessionNotFound(session.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pid(s: &str) -> ProviderAccountId {
        ProviderAccountId(s.to_owned())
    }

    fn pool_with(strategy: PoolRoutingStrategy, members: &[&str]) -> ProviderAccountPool {
        ProviderAccountPool::new(
            PoolId("p1".into()),
            ProviderFamily::Claude,
            ProviderTier::Pro,
            TenantId("t1".into()),
            members.iter().map(|m| pid(m)).collect(),
            strategy,
            DurationMs(60_000),
        )
    }

    #[test]
    fn empty_pool_errors() {
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &[]);
        let r = pick_account(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &Default::default(),
            UnixMillis(1000),
        );
        assert_eq!(r, Err(PoolError::EmptyMembers));
    }

    #[test]
    fn single_member_no_special_case() {
        // Linus good-taste: single-account == pool of 1; no branch needed.
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &["a"]);
        let d = pick_account(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &Default::default(),
            UnixMillis(1000),
        )
        .unwrap();
        assert_eq!(d.account_id, pid("a"));
        assert!(d.fallback_chain.is_empty());
    }

    #[test]
    fn round_robin_walks_members() {
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &["a", "b", "c"]);
        let mut req = RequestMetadata::new("m".into());
        // Start: first member.
        let d = pick_account(
            &p,
            &req,
            &Default::default(),
            &Default::default(),
            UnixMillis(1),
        )
        .unwrap();
        assert_eq!(d.account_id, pid("a"));
        req.previous_account = Some(pid("a"));
        let d = pick_account(
            &p,
            &req,
            &Default::default(),
            &Default::default(),
            UnixMillis(2),
        )
        .unwrap();
        assert_eq!(d.account_id, pid("b"));
        req.previous_account = Some(pid("b"));
        let d = pick_account(
            &p,
            &req,
            &Default::default(),
            &Default::default(),
            UnixMillis(3),
        )
        .unwrap();
        assert_eq!(d.account_id, pid("c"));
        req.previous_account = Some(pid("c"));
        let d = pick_account(
            &p,
            &req,
            &Default::default(),
            &Default::default(),
            UnixMillis(4),
        )
        .unwrap();
        assert_eq!(d.account_id, pid("a")); // wraps
    }

    #[test]
    fn round_robin_visits_all_within_n_calls() {
        // Property: with N members, N successive calls (advancing previous_account)
        // visit every member.
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &["a", "b", "c", "d"]);
        let mut visited: BTreeSet<ProviderAccountId> = BTreeSet::new();
        let mut req = RequestMetadata::new("m".into());
        for i in 0..4 {
            let d = pick_account(
                &p,
                &req,
                &Default::default(),
                &Default::default(),
                UnixMillis(i),
            )
            .unwrap();
            req.previous_account = Some(d.account_id.clone());
            visited.insert(d.account_id);
        }
        assert_eq!(visited.len(), 4);
    }

    #[test]
    fn least_used_picks_strictly_lowest() {
        let p = pool_with(PoolRoutingStrategy::LeastUsed, &["a", "b", "c"]);
        let mut usage: UsageSnapshotMap = BTreeMap::new();
        usage.insert(
            pid("a"),
            UsageSnapshot {
                requests_in_window: 10,
                ..UsageSnapshot::zero()
            },
        );
        usage.insert(
            pid("b"),
            UsageSnapshot {
                requests_in_window: 2,
                ..UsageSnapshot::zero()
            },
        );
        usage.insert(
            pid("c"),
            UsageSnapshot {
                requests_in_window: 5,
                ..UsageSnapshot::zero()
            },
        );
        let d = pick_account(
            &p,
            &RequestMetadata::new("m".into()),
            &usage,
            &Default::default(),
            UnixMillis(1),
        )
        .unwrap();
        assert_eq!(d.account_id, pid("b"));
        assert_eq!(d.reason, PoolRoutingReason::Healthy);
    }

    #[test]
    fn least_used_tie_break_via_btreeset_order() {
        let p = pool_with(PoolRoutingStrategy::LeastUsed, &["b", "a"]);
        let usage: UsageSnapshotMap = BTreeMap::new(); // both 0
        let d = pick_account(
            &p,
            &RequestMetadata::new("m".into()),
            &usage,
            &Default::default(),
            UnixMillis(1),
        )
        .unwrap();
        // BTreeSet orders "a" before "b".
        assert_eq!(d.account_id, pid("a"));
        assert_eq!(d.reason, PoolRoutingReason::LeastUsedTieBreak);
    }

    #[test]
    fn least_remaining_skips_below_threshold() {
        let p = pool_with(PoolRoutingStrategy::LeastRemaining, &["a", "b"]);
        let mut usage: UsageSnapshotMap = BTreeMap::new();
        usage.insert(
            pid("a"),
            UsageSnapshot {
                remaining_quota_pct: 80,
                ..UsageSnapshot::zero()
            },
        );
        usage.insert(
            pid("b"),
            UsageSnapshot {
                remaining_quota_pct: 2,
                ..UsageSnapshot::zero()
            },
        );
        let d = pick_account(
            &p,
            &RequestMetadata::new("m".into()),
            &usage,
            &Default::default(),
            UnixMillis(1),
        )
        .unwrap();
        assert_eq!(d.account_id, pid("a"));
        assert_eq!(d.reason, PoolRoutingReason::QuotaPreserve);
    }

    #[test]
    fn least_remaining_all_below_errors() {
        let p = pool_with(PoolRoutingStrategy::LeastRemaining, &["a", "b"]);
        let mut usage: UsageSnapshotMap = BTreeMap::new();
        usage.insert(
            pid("a"),
            UsageSnapshot {
                remaining_quota_pct: 1,
                ..UsageSnapshot::zero()
            },
        );
        usage.insert(
            pid("b"),
            UsageSnapshot {
                remaining_quota_pct: 2,
                ..UsageSnapshot::zero()
            },
        );
        let r = pick_account(
            &p,
            &RequestMetadata::new("m".into()),
            &usage,
            &Default::default(),
            UnixMillis(1),
        );
        assert_eq!(r, Err(PoolError::RemainingQuotaThresholdNotMet));
    }

    #[test]
    fn least_latency_picks_lowest_p99() {
        let p = pool_with(PoolRoutingStrategy::LeastLatency, &["a", "b", "c"]);
        let mut usage: UsageSnapshotMap = BTreeMap::new();
        usage.insert(
            pid("a"),
            UsageSnapshot {
                p99_latency_ms: 900,
                ..UsageSnapshot::zero()
            },
        );
        usage.insert(
            pid("b"),
            UsageSnapshot {
                p99_latency_ms: 120,
                ..UsageSnapshot::zero()
            },
        );
        usage.insert(
            pid("c"),
            UsageSnapshot {
                p99_latency_ms: 350,
                ..UsageSnapshot::zero()
            },
        );
        let d = pick_account(
            &p,
            &RequestMetadata::new("m".into()),
            &usage,
            &Default::default(),
            UnixMillis(1),
        )
        .unwrap();
        assert_eq!(d.account_id, pid("b"));
    }

    #[test]
    fn all_unhealthy_returns_no_healthy_members() {
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &["a", "b"]);
        let mut health: AccountHealthMap = BTreeMap::new();
        health.insert(
            pid("a"),
            AccountHealth {
                state: HealthState::Unhealthy,
                consecutive_failures: 5,
                cooldown_until: None,
            },
        );
        health.insert(
            pid("b"),
            AccountHealth {
                state: HealthState::Unhealthy,
                consecutive_failures: 5,
                cooldown_until: None,
            },
        );
        let r = pick_account(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &health,
            UnixMillis(1),
        );
        assert_eq!(r, Err(PoolError::NoHealthyMembers));
    }

    #[test]
    fn unhealthy_member_is_skipped_in_fallback() {
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &["a", "b", "c"]);
        let mut health: AccountHealthMap = BTreeMap::new();
        health.insert(
            pid("b"),
            AccountHealth {
                state: HealthState::Unhealthy,
                consecutive_failures: 9,
                cooldown_until: None,
            },
        );
        let d = pick_account(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &health,
            UnixMillis(1),
        )
        .unwrap();
        assert_eq!(d.account_id, pid("a"));
        // fallback should only contain c (b filtered as unhealthy).
        assert_eq!(d.fallback_chain, vec![pid("c")]);
    }

    #[test]
    fn sticky_keeps_previous_account_if_healthy() {
        let session = SessionId("s1".into());
        let p = pool_with(
            PoolRoutingStrategy::Sticky(session.clone()),
            &["a", "b", "c"],
        );
        let mut req = RequestMetadata::new("m".into());
        req.session = Some(session.clone());
        req.previous_account = Some(pid("b"));
        let d = pick_account(
            &p,
            &req,
            &Default::default(),
            &Default::default(),
            UnixMillis(1),
        )
        .unwrap();
        assert_eq!(d.account_id, pid("b"));
        assert_eq!(d.reason, PoolRoutingReason::Sticky);
    }

    #[test]
    fn sticky_first_request_uses_least_used() {
        let session = SessionId("s1".into());
        let p = pool_with(
            PoolRoutingStrategy::Sticky(session.clone()),
            &["a", "b", "c"],
        );
        let mut req = RequestMetadata::new("m".into());
        req.session = Some(session);
        let mut usage: UsageSnapshotMap = BTreeMap::new();
        usage.insert(
            pid("c"),
            UsageSnapshot {
                requests_in_window: 0,
                ..UsageSnapshot::zero()
            },
        );
        usage.insert(
            pid("a"),
            UsageSnapshot {
                requests_in_window: 100,
                ..UsageSnapshot::zero()
            },
        );
        usage.insert(
            pid("b"),
            UsageSnapshot {
                requests_in_window: 50,
                ..UsageSnapshot::zero()
            },
        );
        let d = pick_account(&p, &req, &usage, &Default::default(), UnixMillis(1)).unwrap();
        assert_eq!(d.account_id, pid("c"));
        assert_eq!(d.reason, PoolRoutingReason::Sticky);
    }

    #[test]
    fn sticky_session_not_found_when_no_anchor() {
        let session = SessionId("s1".into());
        let p = pool_with(PoolRoutingStrategy::Sticky(session.clone()), &["a", "b"]);
        // request has neither matching session nor previous_account
        let r = pick_account(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &Default::default(),
            UnixMillis(1),
        );
        assert_eq!(r, Err(PoolError::StickySessionNotFound(session)));
    }

    #[test]
    fn deterministic_given_identical_inputs() {
        // Property: pick_account is deterministic.
        let p = pool_with(PoolRoutingStrategy::LeastUsed, &["a", "b", "c"]);
        let mut usage: UsageSnapshotMap = BTreeMap::new();
        usage.insert(
            pid("a"),
            UsageSnapshot {
                requests_in_window: 7,
                ..UsageSnapshot::zero()
            },
        );
        let req = RequestMetadata::new("m".into());
        let d1 = pick_account(&p, &req, &usage, &Default::default(), UnixMillis(1)).unwrap();
        let d2 = pick_account(&p, &req, &usage, &Default::default(), UnixMillis(1)).unwrap();
        assert_eq!(d1, d2);
    }

    #[test]
    fn tos_ack_attached_via_builder() {
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &["a"])
            .with_tos_ack(TosAckId("ack-1".into()));
        assert_eq!(p.tos_acknowledgment_ref, Some(TosAckId("ack-1".into())));
    }

    #[test]
    fn membership_change_variants_distinct() {
        let a = PoolMembershipChange::Added(pid("a"));
        let r = PoolMembershipChange::Removed(pid("a"));
        let q = PoolMembershipChange::Quarantined(pid("a"));
        assert_ne!(a, r);
        assert_ne!(r, q);
    }

    #[test]
    fn pool_error_display_distinct() {
        let m: Vec<String> = vec![
            format!("{}", PoolError::EmptyMembers),
            format!("{}", PoolError::NoHealthyMembers),
            format!(
                "{}",
                PoolError::StickySessionNotFound(SessionId("x".into()))
            ),
            format!("{}", PoolError::RemainingQuotaThresholdNotMet),
        ];
        let uniq: std::collections::HashSet<_> = m.iter().collect();
        assert_eq!(uniq.len(), m.len());
    }

    #[test]
    fn routing_reason_names_distinct() {
        let names: std::collections::HashSet<&str> = [
            PoolRoutingReason::Healthy,
            PoolRoutingReason::FailoverFrom(pid("x")),
            PoolRoutingReason::Sticky,
            PoolRoutingReason::QuotaPreserve,
            PoolRoutingReason::LeastUsedTieBreak,
        ]
        .iter()
        .map(|r| r.name())
        .collect();
        assert_eq!(names.len(), 5);
    }

    #[test]
    fn provider_tier_enumerates() {
        // Compile-time check via match coverage.
        for t in [
            ProviderTier::Free,
            ProviderTier::Pro,
            ProviderTier::Team,
            ProviderTier::Enterprise,
        ] {
            let _ = match t {
                ProviderTier::Free => "free",
                ProviderTier::Pro => "pro",
                ProviderTier::Team => "team",
                ProviderTier::Enterprise => "enterprise",
            };
        }
    }

    #[test]
    fn pool_size_reports_member_count() {
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &["a", "b", "c", "d"]);
        assert_eq!(p.size(), 4);
    }

    #[test]
    fn fallback_chain_excludes_chosen() {
        let p = pool_with(PoolRoutingStrategy::LeastUsed, &["a", "b", "c"]);
        let usage: UsageSnapshotMap = BTreeMap::new();
        let d = pick_account(
            &p,
            &RequestMetadata::new("m".into()),
            &usage,
            &Default::default(),
            UnixMillis(1),
        )
        .unwrap();
        assert!(!d.fallback_chain.contains(&d.account_id));
        // chain length == size - 1
        assert_eq!(d.fallback_chain.len(), p.size() - 1);
    }

    // ── Cooldown tests (ST2 + ST3) ────────────────────────────────────────

    fn cooldown(window_ms: u64, now_ms: u64) -> CooldownPolicy {
        CooldownPolicy {
            window_ms: DurationMs(window_ms),
            now: UnixMillis(now_ms),
        }
    }

    fn quarantine_at(account: &str, at: u64) -> QuarantineMap {
        let mut m = QuarantineMap::new();
        m.insert(pid(account), UnixMillis(at));
        m
    }

    fn quarantine_many(entries: &[(&str, u64)]) -> QuarantineMap {
        entries
            .iter()
            .map(|(a, t)| (pid(a), UnixMillis(*t)))
            .collect()
    }

    /// ST2: account quarantined 10 ms ago with a 60 s window is excluded.
    #[test]
    fn cooldown_excludes_in_window_account() {
        let now = 100_000u64;
        let window = 60_000u64;
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &["a", "b"]);
        // "a" quarantined 10 ms ago — still inside 60 s window.
        let quarantines = quarantine_at("a", now - 10);
        let d = pick_account_with_cooldown(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &Default::default(),
            &quarantines,
            cooldown(window, now),
        )
        .unwrap();
        assert_eq!(
            d.account_id,
            pid("b"),
            "in-cooldown account must be excluded"
        );
        assert!(d.fallback_chain.is_empty());
    }

    /// ST2: account quarantined 90 s ago with a 60 s window is re-admitted.
    #[test]
    fn cooldown_readmits_elapsed_account() {
        let now = 100_000u64;
        let window = 60_000u64;
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &["a"]);
        // "a" quarantined 90 s ago — cooldown has elapsed.
        let quarantines = quarantine_at("a", now - 90_000);
        let d = pick_account_with_cooldown(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &Default::default(),
            &quarantines,
            cooldown(window, now),
        )
        .unwrap();
        assert_eq!(
            d.account_id,
            pid("a"),
            "elapsed-cooldown account must be re-admitted"
        );
    }

    /// ST2: all members in cooldown → NoHealthyMembers.
    #[test]
    fn all_in_cooldown_returns_no_healthy_members() {
        let now = 100_000u64;
        let window = 60_000u64;
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &["a", "b"]);
        let quarantines = quarantine_many(&[("a", now - 1_000), ("b", now - 2_000)]);
        let r = pick_account_with_cooldown(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &Default::default(),
            &quarantines,
            cooldown(window, now),
        );
        assert_eq!(r, Err(PoolError::NoHealthyMembers));
    }

    /// ST2: fallback chain follows BTree order of eligible members.
    #[test]
    fn cooldown_fallback_chain_deterministic() {
        let now = 100_000u64;
        let window = 60_000u64;
        // Members: a, b, c, d — "b" in cooldown.
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &["a", "b", "c", "d"]);
        let quarantines = quarantine_at("b", now - 500);
        let d = pick_account_with_cooldown(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &Default::default(),
            &quarantines,
            cooldown(window, now),
        )
        .unwrap();
        // eligible = [a, c, d]; chosen = a (first in BTree order, no previous).
        assert_eq!(d.account_id, pid("a"));
        assert_eq!(d.fallback_chain, vec![pid("c"), pid("d")]);
        // Running again with same inputs must produce identical output.
        let d2 = pick_account_with_cooldown(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &Default::default(),
            &quarantines,
            cooldown(window, now),
        )
        .unwrap();
        assert_eq!(d, d2, "cooldown path must be deterministic");
    }

    /// ST3: quarantined account with 100% remaining quota is skipped in favour
    /// of a healthy account with lower remaining quota.
    #[test]
    fn quarantined_high_quota_skipped_favour_healthy_lower_quota() {
        let now = 100_000u64;
        let window = 60_000u64;
        let p = pool_with(PoolRoutingStrategy::LeastRemaining, &["a", "b"]);
        // "a" has 100% quota but is quarantined 5 s ago (inside 60 s window).
        // "b" is fully healthy with 30% quota.
        let quarantines = quarantine_at("a", now - 5_000);
        let mut usage: UsageSnapshotMap = BTreeMap::new();
        usage.insert(
            pid("a"),
            UsageSnapshot {
                remaining_quota_pct: 100,
                ..UsageSnapshot::zero()
            },
        );
        usage.insert(
            pid("b"),
            UsageSnapshot {
                remaining_quota_pct: 30,
                ..UsageSnapshot::zero()
            },
        );
        let d = pick_account_with_cooldown(
            &p,
            &RequestMetadata::new("m".into()),
            &usage,
            &Default::default(),
            &quarantines,
            cooldown(window, now),
        )
        .unwrap();
        assert_eq!(
            d.account_id,
            pid("b"),
            "quarantined high-quota account must be skipped"
        );
        assert_eq!(d.reason, PoolRoutingReason::QuotaPreserve);
    }

    /// ST3: existing pick_account tests are unaffected — AccountHealth has no
    /// cooldown field; the original 2-field struct is intact.
    #[test]
    fn pick_account_healthy_default_no_regression() {
        // Mirrors the existing unhealthy_member_is_skipped_in_fallback test
        // to confirm AccountHealth still works identically.
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &["a", "b", "c"]);
        let mut health: AccountHealthMap = BTreeMap::new();
        health.insert(
            pid("b"),
            AccountHealth {
                state: HealthState::Unhealthy,
                consecutive_failures: 9,
                cooldown_until: None,
            },
        );
        let d = pick_account(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &health,
            UnixMillis(1),
        )
        .unwrap();
        assert_eq!(d.account_id, pid("a"));
        assert_eq!(d.fallback_chain, vec![pid("c")]);
    }

    /// CooldownPolicy::from_pool convenience constructor.
    #[test]
    fn cooldown_policy_from_pool() {
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &["a"]);
        let cp = CooldownPolicy::from_pool(&p, UnixMillis(999));
        assert_eq!(cp.window_ms, DurationMs(60_000));
        assert_eq!(cp.now, UnixMillis(999));
    }

    /// in_cooldown returns false when account is absent from the QuarantineMap.
    #[test]
    fn in_cooldown_none_is_never_in_cooldown() {
        let cp = cooldown(60_000, 100_000);
        let quarantines: QuarantineMap = Default::default();
        assert!(!cp.in_cooldown(&pid("a"), &quarantines));
    }

    // ── Additional red tests: untested edge cases ────────────────────────────

    /// pick_account_with_cooldown on an empty pool returns EmptyMembers.
    #[test]
    fn cooldown_empty_pool_returns_empty_members() {
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &[]);
        let r = pick_account_with_cooldown(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &Default::default(),
            &Default::default(),
            cooldown(60_000, 100_000),
        );
        assert_eq!(r, Err(PoolError::EmptyMembers));
    }

    /// CooldownPolicy::in_cooldown at the exact boundary (elapsed == window_ms)
    /// must return false — the window uses strict `<` semantics, so expiry is
    /// inclusive on the boundary.
    #[test]
    fn in_cooldown_exact_boundary_is_not_in_cooldown() {
        // quarantined_at = 40_000, window = 60_000, now = 100_000
        // now - quarantined_at == 60_000 == window_ms → NOT in cooldown.
        let cp = cooldown(60_000, 100_000);
        let mut quarantines = QuarantineMap::new();
        quarantines.insert(pid("a"), UnixMillis(40_000));
        assert!(
            !cp.in_cooldown(&pid("a"), &quarantines),
            "elapsed == window_ms must not be in cooldown (< semantics)"
        );
    }

    /// A Degraded (not Unhealthy) account that is inside the cooldown window
    /// must be excluded — the cooldown filter applies independent of health
    /// state, provided the state is not Unhealthy.
    #[test]
    fn degraded_account_inside_cooldown_is_excluded() {
        let now = 100_000u64;
        let window = 60_000u64;
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &["a", "b"]);
        let mut health: AccountHealthMap = BTreeMap::new();
        // "a" is Degraded (not Unhealthy) but quarantined 1 s ago — still in window.
        health.insert(
            pid("a"),
            AccountHealth {
                state: HealthState::Degraded,
                consecutive_failures: 2,
                cooldown_until: None,
            },
        );
        health.insert(pid("b"), AccountHealth::healthy());
        let quarantines = quarantine_at("a", now - 1_000);
        let d = pick_account_with_cooldown(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &health,
            &quarantines,
            cooldown(window, now),
        )
        .unwrap();
        assert_eq!(
            d.account_id,
            pid("b"),
            "Degraded account inside cooldown window must be excluded"
        );
    }

    /// A Degraded account whose cooldown has elapsed must be re-admitted as an
    /// eligible candidate.
    #[test]
    fn degraded_account_outside_cooldown_is_admitted() {
        let now = 100_000u64;
        let window = 60_000u64;
        // Only "a" in pool; it is Degraded but cooldown has elapsed.
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &["a"]);
        let mut health: AccountHealthMap = BTreeMap::new();
        health.insert(
            pid("a"),
            AccountHealth {
                state: HealthState::Degraded,
                consecutive_failures: 1,
                cooldown_until: None,
            },
        );
        let quarantines = quarantine_at("a", now - 90_000);
        let d = pick_account_with_cooldown(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &health,
            &quarantines,
            cooldown(window, now),
        )
        .unwrap();
        assert_eq!(
            d.account_id,
            pid("a"),
            "Degraded account with elapsed cooldown must be admitted"
        );
    }

    /// An account with no entry in the health map must be treated as healthy
    /// and not in cooldown — eligible for routing.
    #[test]
    fn missing_health_entry_treated_as_healthy_not_in_cooldown() {
        let now = 100_000u64;
        // "a" has no health entry and no quarantine entry; pool has one member.
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &["a"]);
        let d = pick_account_with_cooldown(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &Default::default(), // empty health map
            &Default::default(), // empty quarantine map
            cooldown(60_000, now),
        )
        .unwrap();
        assert_eq!(
            d.account_id,
            pid("a"),
            "account absent from health/quarantine maps must be treated as eligible"
        );
    }

    /// decided_at_unix_ms on the cooldown decision must reflect cooldown.now,
    /// not some other timestamp.
    #[test]
    fn cooldown_decision_timestamp_equals_cooldown_now() {
        let now = 999_999u64;
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &["a"]);
        let d = pick_account_with_cooldown(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &Default::default(),
            &Default::default(),
            cooldown(60_000, now),
        )
        .unwrap();
        assert_eq!(
            d.decided_at_unix_ms,
            UnixMillis(now),
            "decided_at_unix_ms must equal cooldown.now"
        );
    }

    /// When previous_account is itself in cooldown (excluded from eligible),
    /// round-robin must signal PoolRoutingReason::FailoverFrom(prev) to
    /// indicate an anti-correlation failover, not Healthy.
    ///
    /// This is the ccproxy-api anti-correlation semantic: the caller learns
    /// *which* account was abandoned due to quarantine.
    #[test]
    fn cooldown_failover_from_previous_in_cooldown_emits_failover_reason() {
        let now = 100_000u64;
        let window = 60_000u64;
        // Pool: a (in cooldown), b (healthy).
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &["a", "b"]);
        let quarantines = quarantine_at("a", now - 1_000);
        let mut req = RequestMetadata::new("m".into());
        // previous_account = "a" (the account that was just quarantined).
        req.previous_account = Some(pid("a"));
        let d = pick_account_with_cooldown(
            &p,
            &req,
            &Default::default(),
            &Default::default(),
            &quarantines,
            cooldown(window, now),
        )
        .unwrap();
        assert_eq!(
            d.account_id,
            pid("b"),
            "must route away from cooldown account"
        );
        assert_eq!(
            d.reason,
            PoolRoutingReason::FailoverFrom(pid("a")),
            "must emit FailoverFrom(prev) when previous account is in cooldown"
        );
    }

    // ── FailureKind + backoff table tests ────────────────────────────────────

    /// AccountHealth::healthy() compiles with the new cooldown_until field
    /// defaulting to None.
    #[test]
    fn account_health_cooldown_until_field_defaults_none() {
        let h = AccountHealth::healthy();
        assert_eq!(h.cooldown_until, None);
        assert_eq!(h.state, HealthState::Healthy);
        assert_eq!(h.consecutive_failures, 0);
    }

    /// AccountHealth with an explicit cooldown_until carries the value through.
    #[test]
    fn account_health_cooldown_until_set() {
        let h = AccountHealth {
            state: HealthState::Unhealthy,
            consecutive_failures: 3,
            cooldown_until: Some(UnixMillis(999_000)),
        };
        assert_eq!(h.cooldown_until, Some(UnixMillis(999_000)));
    }

    /// UpstreamRateLimit429 backoff: escalates 30s → 60s → 120s → 300s (cap).
    #[test]
    fn failure_kind_backoff_rate_limit() {
        assert_eq!(
            CooldownPolicy::window_for(FailureKind::UpstreamRateLimit429, 1),
            DurationMs(30_000)
        );
        assert_eq!(
            CooldownPolicy::window_for(FailureKind::UpstreamRateLimit429, 2),
            DurationMs(60_000)
        );
        assert_eq!(
            CooldownPolicy::window_for(FailureKind::UpstreamRateLimit429, 3),
            DurationMs(120_000)
        );
        // tier 4 and beyond cap at 300 s
        assert_eq!(
            CooldownPolicy::window_for(FailureKind::UpstreamRateLimit429, 4),
            DurationMs(300_000)
        );
        assert_eq!(
            CooldownPolicy::window_for(FailureKind::UpstreamRateLimit429, 100),
            DurationMs(300_000),
            "must cap at max tier"
        );
    }

    /// UpstreamServerError5xx backoff: 10s → 30s → 60s (cap).
    #[test]
    fn failure_kind_backoff_server_error() {
        assert_eq!(
            CooldownPolicy::window_for(FailureKind::UpstreamServerError5xx, 1),
            DurationMs(10_000)
        );
        assert_eq!(
            CooldownPolicy::window_for(FailureKind::UpstreamServerError5xx, 2),
            DurationMs(30_000)
        );
        assert_eq!(
            CooldownPolicy::window_for(FailureKind::UpstreamServerError5xx, 3),
            DurationMs(60_000)
        );
        assert_eq!(
            CooldownPolicy::window_for(FailureKind::UpstreamServerError5xx, 4),
            DurationMs(60_000),
            "must cap at 60 s"
        );
    }

    /// ConnectionTimeout backoff: 5s → 15s → 30s (cap).
    #[test]
    fn failure_kind_backoff_timeout() {
        assert_eq!(
            CooldownPolicy::window_for(FailureKind::ConnectionTimeout, 1),
            DurationMs(5_000)
        );
        assert_eq!(
            CooldownPolicy::window_for(FailureKind::ConnectionTimeout, 2),
            DurationMs(15_000)
        );
        assert_eq!(
            CooldownPolicy::window_for(FailureKind::ConnectionTimeout, 3),
            DurationMs(30_000)
        );
        assert_eq!(
            CooldownPolicy::window_for(FailureKind::ConnectionTimeout, 99),
            DurationMs(30_000),
            "must cap at 30 s"
        );
    }

    /// AuthFailure backoff: 60s → 300s → 900s (cap).
    #[test]
    fn failure_kind_backoff_auth() {
        assert_eq!(
            CooldownPolicy::window_for(FailureKind::AuthFailure, 1),
            DurationMs(60_000)
        );
        assert_eq!(
            CooldownPolicy::window_for(FailureKind::AuthFailure, 2),
            DurationMs(300_000)
        );
        assert_eq!(
            CooldownPolicy::window_for(FailureKind::AuthFailure, 3),
            DurationMs(900_000)
        );
        assert_eq!(
            CooldownPolicy::window_for(FailureKind::AuthFailure, 10),
            DurationMs(900_000),
            "must cap at 900 s"
        );
    }

    /// consecutive_failures == 0 is treated as tier 1 (first failure).
    #[test]
    fn backoff_zero_failures_treated_as_one() {
        assert_eq!(
            CooldownPolicy::window_for(FailureKind::UpstreamRateLimit429, 0),
            CooldownPolicy::window_for(FailureKind::UpstreamRateLimit429, 1),
            "zero failures must map to first-failure tier"
        );
        assert_eq!(
            CooldownPolicy::window_for(FailureKind::AuthFailure, 0),
            CooldownPolicy::window_for(FailureKind::AuthFailure, 1)
        );
    }

    // ── populate_quarantine_from_changes tests ──────────────────────────────

    /// Quarantined entries are inserted into the map at the supplied timestamp.
    #[test]
    fn populate_quarantine_from_changes_basic() {
        let now = UnixMillis(50_000);
        let changes = vec![
            PoolMembershipChange::Quarantined(pid("a")),
            PoolMembershipChange::Quarantined(pid("b")),
        ];
        let mut quarantines = QuarantineMap::new();
        populate_quarantine_from_changes(&changes, now, &mut quarantines);
        assert_eq!(quarantines.get(&pid("a")), Some(&now));
        assert_eq!(quarantines.get(&pid("b")), Some(&now));
        assert_eq!(quarantines.len(), 2);
    }

    /// Added and Removed variants are ignored by populate_quarantine_from_changes.
    #[test]
    fn populate_quarantine_ignores_non_quarantined() {
        let now = UnixMillis(50_000);
        let changes = vec![
            PoolMembershipChange::Added(pid("x")),
            PoolMembershipChange::Removed(pid("y")),
            PoolMembershipChange::Quarantined(pid("z")),
        ];
        let mut quarantines = QuarantineMap::new();
        populate_quarantine_from_changes(&changes, now, &mut quarantines);
        // Only "z" should be in the map.
        assert!(!quarantines.contains_key(&pid("x")));
        assert!(!quarantines.contains_key(&pid("y")));
        assert_eq!(quarantines.get(&pid("z")), Some(&now));
        assert_eq!(quarantines.len(), 1);
    }

    /// A stale quarantine entry is overwritten by a fresh Quarantined event.
    #[test]
    fn populate_quarantine_overwrites_stale_entry() {
        let stale = UnixMillis(1_000);
        let fresh = UnixMillis(90_000);
        let mut quarantines = QuarantineMap::new();
        quarantines.insert(pid("a"), stale);
        let changes = vec![PoolMembershipChange::Quarantined(pid("a"))];
        populate_quarantine_from_changes(&changes, fresh, &mut quarantines);
        assert_eq!(
            quarantines.get(&pid("a")),
            Some(&fresh),
            "stale entry must be overwritten"
        );
    }

    /// Empty changes slice leaves the map unchanged.
    #[test]
    fn populate_quarantine_empty_changes_no_op() {
        let mut quarantines = QuarantineMap::new();
        quarantines.insert(pid("a"), UnixMillis(1_000));
        populate_quarantine_from_changes(&[], UnixMillis(99_000), &mut quarantines);
        assert_eq!(
            quarantines.len(),
            1,
            "map must be unchanged for empty input"
        );
    }

    /// Integration: populate then pick_account_with_cooldown excludes
    /// the newly quarantined account.
    #[test]
    fn populate_then_cooldown_excludes_quarantined() {
        let now = UnixMillis(100_000);
        let window = DurationMs(60_000);
        let p = ProviderAccountPool::new(
            PoolId("p1".into()),
            ProviderFamily::Claude,
            ProviderTier::Pro,
            TenantId("t1".into()),
            [pid("a"), pid("b")].into_iter().collect(),
            PoolRoutingStrategy::RoundRobin,
            window,
        );
        // "a" just got quarantined.
        let changes = vec![PoolMembershipChange::Quarantined(pid("a"))];
        let mut quarantines = QuarantineMap::new();
        populate_quarantine_from_changes(&changes, now, &mut quarantines);
        let cp = CooldownPolicy::from_pool(&p, now);
        let d = pick_account_with_cooldown(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &Default::default(),
            &quarantines,
            cp,
        )
        .unwrap();
        assert_eq!(
            d.account_id,
            pid("b"),
            "quarantined account must be excluded from routing"
        );
    }

    fn agent(s: &str) -> AgentId {
        AgentId(s.to_owned())
    }

    fn seat(s: &str) -> SeatId {
        SeatId(s.to_owned())
    }

    fn sub(s: &str) -> SubscriptionId {
        SubscriptionId(format!("sub-{s}"))
    }

    fn secret(s: &str) -> SecretReference {
        SecretReference::new(format!("sref://provider-subscriptions/{s}")).unwrap()
    }

    fn oauth_subscription(s: &str) -> OAuthSubscription {
        OAuthSubscription::new(
            seat(s),
            pid(s),
            sub(s),
            ProviderFamily::Claude,
            secret(s),
            SubscriptionSeatState::Active,
            UsageSnapshot::zero(),
        )
    }

    fn subscription_pool_with(
        strategy: SubscriptionPoolStrategy,
        seat_ids: &[&str],
    ) -> SubscriptionPool {
        let mut pool = SubscriptionPool::new(
            PoolId("subscription-pool".into()),
            TenantId("tenant-1".into()),
            ProviderFamily::Claude,
            strategy,
        );
        for seat_id in seat_ids {
            pool.add_subscription(oauth_subscription(seat_id)).unwrap();
        }
        pool
    }

    fn assert_subscription_pool_proof_invariants(pool: &SubscriptionPool) {
        let mut reserved_seats = BTreeSet::new();
        let mut active_leases = BTreeSet::new();

        for (seat_id, subscription) in &pool.seats {
            assert_eq!(
                seat_id, &subscription.seat_id,
                "seat map key must match the subscription value object"
            );
            if let SubscriptionSeatState::Reserved { lease_id, .. } = &subscription.state {
                assert!(
                    reserved_seats.insert(seat_id.clone()),
                    "a reserved seat must appear once in the pool map"
                );
                assert!(
                    active_leases.insert(lease_id.clone()),
                    "an active lease id must be held by one seat only"
                );
            }
        }

        assert_eq!(
            pool.reserved_count(),
            reserved_seats.len(),
            "reserved_count must match the reserved seat set"
        );
        assert_eq!(
            pool.reserved_count(),
            active_leases.len(),
            "every reserved seat must have exactly one active lease"
        );
        assert_eq!(
            pool.seat_count(),
            pool.reserved_count() + pool.free_count(),
            "ADR-0390/0391 pool invariant must hold after every state transition"
        );
    }

    #[test]
    fn subscription_pool_lease_marks_reserved_and_excludes_same_seat() {
        let mut pool = subscription_pool_with(SubscriptionPoolStrategy::RoundRobin, &["a"]);

        let lease = pool.lease(agent("agent-1"), UnixMillis(10)).unwrap();

        assert_eq!(lease.seat_id, seat("a"));
        match pool.seat_state(&seat("a")).unwrap() {
            SubscriptionSeatState::Reserved {
                lease_id,
                agent_id,
                since_unix_ms,
            } => {
                assert_eq!(lease_id, &lease.lease_id);
                assert_eq!(agent_id, &agent("agent-1"));
                assert_eq!(since_unix_ms, &UnixMillis(10));
            }
            other => panic!("expected reserved seat, got {other:?}"),
        }
        assert_eq!(
            pool.lease(agent("agent-2"), UnixMillis(11)).unwrap_err(),
            SubscriptionPoolError::PoolExhausted
        );
        assert_eq!(pool.reserved_count(), 1);
        assert_eq!(pool.free_count(), 0);
        assert_eq!(pool.seat_count(), 1);
    }

    #[test]
    fn subscription_pool_complete_success_reactivates_seat_and_updates_usage() {
        let mut pool = subscription_pool_with(SubscriptionPoolStrategy::RoundRobin, &["a"]);
        let lease = pool.lease(agent("agent-1"), UnixMillis(10)).unwrap();
        let usage = UsageSnapshot {
            requests_in_window: 7,
            remaining_quota_pct: 64,
            last_used_unix_ms: UnixMillis(20),
            p99_latency_ms: 250,
        };

        pool.complete(lease, SeatOutcome::Succeeded { usage }, UnixMillis(20))
            .unwrap();

        assert_eq!(
            pool.seat_state(&seat("a")),
            Some(&SubscriptionSeatState::Active)
        );
        assert_eq!(pool.usage_snapshot(&seat("a")), Some(usage));
        assert_eq!(pool.reserved_count(), 0);
        assert_eq!(pool.free_count(), 1);
    }

    #[test]
    fn subscription_pool_retryable_failure_moves_reserved_seat_to_cooldown() {
        let mut pool = subscription_pool_with(SubscriptionPoolStrategy::RoundRobin, &["a"]);
        let lease = pool.lease(agent("agent-1"), UnixMillis(10_000)).unwrap();

        pool.complete(
            lease,
            SeatOutcome::RetryableFailure {
                kind: FailureKind::UpstreamRateLimit429,
            },
            UnixMillis(10_000),
        )
        .unwrap();

        assert_eq!(
            pool.seat_state(&seat("a")),
            Some(&SubscriptionSeatState::Cooldown {
                until_unix_ms: UnixMillis(40_000),
                reason: FailureKind::UpstreamRateLimit429,
            })
        );
        assert_eq!(
            pool.lease(agent("agent-2"), UnixMillis(39_999))
                .unwrap_err(),
            SubscriptionPoolError::PoolExhausted
        );
        assert_eq!(
            pool.lease(agent("agent-2"), UnixMillis(40_000))
                .unwrap()
                .seat_id,
            seat("a")
        );
    }

    #[test]
    fn subscription_pool_round_robin_uses_next_eligible_after_reserved() {
        let mut pool = subscription_pool_with(SubscriptionPoolStrategy::RoundRobin, &["a", "b"]);

        let first = pool.lease(agent("agent-1"), UnixMillis(1)).unwrap();
        let second = pool.lease(agent("agent-2"), UnixMillis(2)).unwrap();

        assert_eq!(first.seat_id, seat("a"));
        assert_eq!(second.seat_id, seat("b"));
        assert_ne!(first.lease_id, second.lease_id);
        assert_eq!(
            pool.lease(agent("agent-3"), UnixMillis(3)).unwrap_err(),
            SubscriptionPoolError::PoolExhausted
        );
    }

    #[test]
    fn subscription_pool_fill_first_prefers_first_active_seat() {
        let mut pool = subscription_pool_with(SubscriptionPoolStrategy::FillFirst, &["a", "b"]);

        let first = pool.lease(agent("agent-1"), UnixMillis(1)).unwrap();
        pool.complete(
            first,
            SeatOutcome::Succeeded {
                usage: UsageSnapshot::zero(),
            },
            UnixMillis(2),
        )
        .unwrap();
        let second = pool.lease(agent("agent-2"), UnixMillis(3)).unwrap();

        assert_eq!(second.seat_id, seat("a"));
    }

    #[test]
    fn subscription_pool_blacklisted_seat_is_not_eligible() {
        let mut pool = subscription_pool_with(SubscriptionPoolStrategy::FillFirst, &["a", "b"]);
        let lease = pool.lease(agent("agent-1"), UnixMillis(1)).unwrap();

        pool.complete(lease, SeatOutcome::RefreshTokenExhausted, UnixMillis(2))
            .unwrap();
        let next = pool.lease(agent("agent-2"), UnixMillis(3)).unwrap();

        assert_eq!(
            pool.seat_state(&seat("a")),
            Some(&SubscriptionSeatState::Blacklisted {
                reason: SeatBlacklistReason::RefreshTokenExhausted,
            })
        );
        assert_eq!(next.seat_id, seat("b"));
    }

    #[test]
    fn subscription_pool_counts_preserve_total_seats() {
        let mut pool =
            subscription_pool_with(SubscriptionPoolStrategy::RoundRobin, &["a", "b", "c"]);
        let a = pool.lease(agent("agent-1"), UnixMillis(1)).unwrap();
        let b = pool.lease(agent("agent-2"), UnixMillis(2)).unwrap();

        assert_eq!(pool.reserved_count(), 2);
        assert_eq!(pool.free_count(), 1);
        assert_eq!(pool.seat_count(), pool.reserved_count() + pool.free_count());

        pool.complete(
            a,
            SeatOutcome::Succeeded {
                usage: UsageSnapshot::zero(),
            },
            UnixMillis(3),
        )
        .unwrap();
        pool.complete(
            b,
            SeatOutcome::RetryableFailure {
                kind: FailureKind::ConnectionTimeout,
            },
            UnixMillis(3),
        )
        .unwrap();

        assert_eq!(pool.reserved_count(), 0);
        assert_eq!(pool.free_count(), 3);
        assert_eq!(pool.seat_count(), pool.reserved_count() + pool.free_count());
    }

    #[test]
    fn subscription_pool_quota_floor_exhausts_pool() {
        let mut pool = SubscriptionPool::new(
            PoolId("subscription-pool".into()),
            TenantId("tenant-1".into()),
            ProviderFamily::Claude,
            SubscriptionPoolStrategy::RoundRobin,
        );
        let mut subscription = oauth_subscription("a");
        subscription.usage = UsageSnapshot {
            remaining_quota_pct: 1,
            ..UsageSnapshot::zero()
        };
        pool.add_subscription(subscription).unwrap();

        assert_eq!(
            pool.lease(agent("agent-1"), UnixMillis(1)).unwrap_err(),
            SubscriptionPoolError::PoolExhausted
        );
    }

    #[test]
    fn subscription_pool_rejects_duplicate_seat_and_provider_mismatch() {
        let mut pool = subscription_pool_with(SubscriptionPoolStrategy::RoundRobin, &["a"]);

        assert_eq!(
            pool.add_subscription(oauth_subscription("a")).unwrap_err(),
            SubscriptionPoolError::DuplicateSeat(seat("a"))
        );

        let mut gemini_subscription = oauth_subscription("b");
        gemini_subscription.provider = ProviderFamily::Gemini;

        assert_eq!(
            pool.add_subscription(gemini_subscription).unwrap_err(),
            SubscriptionPoolError::ProviderMismatch {
                expected: ProviderFamily::Claude,
                actual: ProviderFamily::Gemini,
            }
        );
        assert_subscription_pool_proof_invariants(&pool);
    }

    #[test]
    fn subscription_pool_rejects_stale_wrong_and_replayed_leases() {
        let mut pool = subscription_pool_with(SubscriptionPoolStrategy::RoundRobin, &["a", "b"]);
        let lease = pool.lease(agent("agent-1"), UnixMillis(10)).unwrap();

        let wrong_lease = SeatLease {
            lease_id: LeaseId("wrong-lease".into()),
            seat_id: lease.seat_id.clone(),
            provider_account_id: lease.provider_account_id.clone(),
            agent_id: lease.agent_id.clone(),
            leased_at_unix_ms: lease.leased_at_unix_ms,
        };

        assert_eq!(
            pool.complete(
                wrong_lease,
                SeatOutcome::Succeeded {
                    usage: UsageSnapshot::zero(),
                },
                UnixMillis(11),
            )
            .unwrap_err(),
            SubscriptionPoolError::LeaseSeatMismatch {
                seat_id: seat("a"),
                lease_id: LeaseId("wrong-lease".into()),
            }
        );
        assert_eq!(pool.reserved_count(), 1);
        assert_subscription_pool_proof_invariants(&pool);

        let active_seat_lease = SeatLease {
            lease_id: LeaseId("never-issued".into()),
            seat_id: seat("b"),
            provider_account_id: pid("b"),
            agent_id: agent("agent-2"),
            leased_at_unix_ms: UnixMillis(12),
        };

        assert_eq!(
            pool.complete(
                active_seat_lease,
                SeatOutcome::Succeeded {
                    usage: UsageSnapshot::zero(),
                },
                UnixMillis(12),
            )
            .unwrap_err(),
            SubscriptionPoolError::SeatNotReserved(seat("b"))
        );

        let replay = SeatLease {
            lease_id: lease.lease_id.clone(),
            seat_id: lease.seat_id.clone(),
            provider_account_id: lease.provider_account_id.clone(),
            agent_id: lease.agent_id.clone(),
            leased_at_unix_ms: lease.leased_at_unix_ms,
        };

        pool.complete(
            lease,
            SeatOutcome::Succeeded {
                usage: UsageSnapshot::zero(),
            },
            UnixMillis(13),
        )
        .unwrap();

        assert_eq!(
            pool.complete(
                replay,
                SeatOutcome::Succeeded {
                    usage: UsageSnapshot::zero(),
                },
                UnixMillis(14),
            )
            .unwrap_err(),
            SubscriptionPoolError::SeatNotReserved(seat("a"))
        );
        assert_subscription_pool_proof_invariants(&pool);
    }

    #[test]
    fn subscription_pool_failure_count_increases_until_success_resets_it() {
        let mut pool = subscription_pool_with(SubscriptionPoolStrategy::RoundRobin, &["a"]);

        let first = pool.lease(agent("agent-1"), UnixMillis(1_000)).unwrap();
        pool.complete(
            first,
            SeatOutcome::RetryableFailure {
                kind: FailureKind::UpstreamRateLimit429,
            },
            UnixMillis(1_000),
        )
        .unwrap();
        assert_eq!(pool.seats.get(&seat("a")).unwrap().failure_count, 1);

        let second = pool.lease(agent("agent-2"), UnixMillis(31_000)).unwrap();
        pool.complete(
            second,
            SeatOutcome::RetryableFailure {
                kind: FailureKind::UpstreamRateLimit429,
            },
            UnixMillis(31_000),
        )
        .unwrap();
        assert_eq!(pool.seats.get(&seat("a")).unwrap().failure_count, 2);

        let third = pool.lease(agent("agent-3"), UnixMillis(91_000)).unwrap();
        pool.complete(
            third,
            SeatOutcome::Succeeded {
                usage: UsageSnapshot {
                    requests_in_window: 1,
                    remaining_quota_pct: 99,
                    last_used_unix_ms: UnixMillis(91_100),
                    p99_latency_ms: 100,
                },
            },
            UnixMillis(91_100),
        )
        .unwrap();

        assert_eq!(pool.seats.get(&seat("a")).unwrap().failure_count, 0);
        assert_eq!(
            pool.seat_state(&seat("a")),
            Some(&SubscriptionSeatState::Active)
        );
        assert_subscription_pool_proof_invariants(&pool);
    }

    #[test]
    fn subscription_pool_repeated_failures_reach_terminal_blacklist() {
        let mut pool = subscription_pool_with(SubscriptionPoolStrategy::RoundRobin, &["a"]);
        let mut now = UnixMillis(1_000);

        for failure_count in 1..=6 {
            let lease = pool.lease(agent("agent-1"), now).unwrap();
            pool.complete(
                lease,
                SeatOutcome::RetryableFailure {
                    kind: FailureKind::UpstreamRateLimit429,
                },
                now,
            )
            .unwrap();
            assert_eq!(
                pool.seats.get(&seat("a")).unwrap().failure_count,
                failure_count
            );
            assert_subscription_pool_proof_invariants(&pool);

            now = UnixMillis(
                now.0
                    + CooldownPolicy::window_for(FailureKind::UpstreamRateLimit429, failure_count)
                        .0,
            );
        }

        assert_eq!(
            pool.seat_state(&seat("a")),
            Some(&SubscriptionSeatState::Blacklisted {
                reason: SeatBlacklistReason::RepeatedFailuresExceededThreshold { failure_count: 6 },
            })
        );
        assert_eq!(
            pool.lease(agent("agent-2"), UnixMillis(999_999))
                .unwrap_err(),
            SubscriptionPoolError::PoolExhausted
        );
    }

    #[test]
    fn subscription_pool_sequence_preserves_counts_and_unique_active_leases() {
        let mut pool =
            subscription_pool_with(SubscriptionPoolStrategy::RoundRobin, &["a", "b", "c"]);
        assert_subscription_pool_proof_invariants(&pool);

        let first = pool.lease(agent("agent-1"), UnixMillis(1)).unwrap();
        assert_subscription_pool_proof_invariants(&pool);
        let second = pool.lease(agent("agent-2"), UnixMillis(2)).unwrap();
        assert_ne!(first.seat_id, second.seat_id);
        assert_ne!(first.lease_id, second.lease_id);
        assert_subscription_pool_proof_invariants(&pool);

        pool.complete(
            first,
            SeatOutcome::Succeeded {
                usage: UsageSnapshot::zero(),
            },
            UnixMillis(3),
        )
        .unwrap();
        assert_subscription_pool_proof_invariants(&pool);

        let third = pool.lease(agent("agent-3"), UnixMillis(4)).unwrap();
        assert_ne!(second.seat_id, third.seat_id);
        assert_ne!(second.lease_id, third.lease_id);
        assert_subscription_pool_proof_invariants(&pool);

        pool.complete(
            second,
            SeatOutcome::RetryableFailure {
                kind: FailureKind::ConnectionTimeout,
            },
            UnixMillis(5),
        )
        .unwrap();
        assert_subscription_pool_proof_invariants(&pool);

        pool.complete(third, SeatOutcome::OperatorBlacklisted, UnixMillis(6))
            .unwrap();
        assert_subscription_pool_proof_invariants(&pool);
    }
}
