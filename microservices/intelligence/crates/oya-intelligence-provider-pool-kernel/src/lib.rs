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

pub use oya_intelligence_account_kernel::ProviderFamily;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccountHealth {
    pub state: HealthState,        // data_class: INTERNAL_ONLY
    pub consecutive_failures: u32, // data_class: INTERNAL_ONLY
    /// Unix-millisecond timestamp at which this account was last quarantined.
    /// `None` means the account has never been quarantined; it is therefore
    /// never in cooldown regardless of the `CooldownPolicy` window.
    ///
    /// data_class: INTERNAL_ONLY
    pub last_quarantined_at_unix_ms: Option<UnixMillis>, // data_class: INTERNAL_ONLY
}

impl AccountHealth {
    pub const fn healthy() -> Self {
        Self {
            state: HealthState::Healthy,
            consecutive_failures: 0,
            last_quarantined_at_unix_ms: None,
        }
    }

    /// Convenience constructor for a freshly-quarantined account.
    pub const fn quarantined(at: UnixMillis) -> Self {
        Self {
            state: HealthState::Unhealthy,
            consecutive_failures: 1,
            last_quarantined_at_unix_ms: Some(at),
        }
    }
}

/// data_class: INTERNAL_ONLY — encapsulates the cooldown window and the
/// evaluation instant so callers pass a single, self-describing input rather
/// than two separate scalars.
///
/// Cooldown semantics: an account is *in cooldown* when
/// `now.0.saturating_sub(last_quarantined_at.0) < window_ms.0`.
/// An account with `last_quarantined_at = None` is never in cooldown.
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

    /// Returns `true` when `health` indicates the account is still inside the
    /// anti-correlation window (i.e. should be excluded from routing).
    pub fn in_cooldown(&self, health: &AccountHealth) -> bool {
        match health.last_quarantined_at_unix_ms {
            None => false,
            Some(quarantined_at) => {
                self.now.0.saturating_sub(quarantined_at.0) < self.window_ms.0
            }
        }
    }
}

pub type UsageSnapshotMap = BTreeMap<ProviderAccountId, UsageSnapshot>;
pub type AccountHealthMap = BTreeMap<ProviderAccountId, AccountHealth>;

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
/// that are `HealthState::Unhealthy` **or** whose `last_quarantined_at_unix_ms`
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
            match health.get(*m) {
                Some(h) => h.state != HealthState::Unhealthy && !cooldown.in_cooldown(h),
                // No health entry → treat as healthy + not in cooldown.
                None => true,
            }
        })
        .cloned()
        .collect();

    if eligible.is_empty() {
        return Err(PoolError::NoHealthyMembers);
    }

    let (chosen, reason) = match &pool.routing_strategy {
        PoolRoutingStrategy::RoundRobin => round_robin(&eligible, request),
        PoolRoutingStrategy::LeastUsed => least_used(&eligible, usage),
        PoolRoutingStrategy::LeastLatency => least_latency(&eligible, usage),
        PoolRoutingStrategy::LeastRemaining => least_remaining(&eligible, usage)?,
        PoolRoutingStrategy::Sticky(session) => sticky(&eligible, session, request, usage)?,
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
                last_quarantined_at_unix_ms: None,
            },
        );
        health.insert(
            pid("b"),
            AccountHealth {
                state: HealthState::Unhealthy,
                consecutive_failures: 5,
                last_quarantined_at_unix_ms: None,
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
                last_quarantined_at_unix_ms: None,
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

    fn health_quarantined_at(at: u64) -> AccountHealth {
        AccountHealth {
            state: HealthState::Unhealthy,
            consecutive_failures: 1,
            last_quarantined_at_unix_ms: Some(UnixMillis(at)),
        }
    }

    fn health_healthy_quarantined_at(at: u64) -> AccountHealth {
        // Healthy state but carries a quarantine timestamp (re-admitted but still
        // within cooldown window → should be excluded).
        AccountHealth {
            state: HealthState::Healthy,
            consecutive_failures: 0,
            last_quarantined_at_unix_ms: Some(UnixMillis(at)),
        }
    }

    /// ST2: account quarantined 10 ms ago with a 60 s window is excluded.
    #[test]
    fn cooldown_excludes_in_window_account() {
        let now = 100_000u64;
        let window = 60_000u64;
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &["a", "b"]);
        let mut health: AccountHealthMap = BTreeMap::new();
        // "a" quarantined 10 ms ago — still inside 60 s window.
        health.insert(pid("a"), health_quarantined_at(now - 10));
        let d = pick_account_with_cooldown(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &health,
            cooldown(window, now),
        )
        .unwrap();
        assert_eq!(d.account_id, pid("b"), "in-cooldown account must be excluded");
        assert!(d.fallback_chain.is_empty());
    }

    /// ST2: account quarantined 90 s ago with a 60 s window is re-admitted.
    #[test]
    fn cooldown_readmits_elapsed_account() {
        let now = 100_000u64;
        let window = 60_000u64;
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &["a"]);
        let mut health: AccountHealthMap = BTreeMap::new();
        // "a" quarantined 90 s ago — cooldown has elapsed; mark as Healthy.
        health.insert(
            pid("a"),
            AccountHealth {
                state: HealthState::Healthy,
                consecutive_failures: 0,
                last_quarantined_at_unix_ms: Some(UnixMillis(now - 90_000)),
            },
        );
        let d = pick_account_with_cooldown(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &health,
            cooldown(window, now),
        )
        .unwrap();
        assert_eq!(d.account_id, pid("a"), "elapsed-cooldown account must be re-admitted");
    }

    /// ST2: all members in cooldown → NoHealthyMembers.
    #[test]
    fn all_in_cooldown_returns_no_healthy_members() {
        let now = 100_000u64;
        let window = 60_000u64;
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &["a", "b"]);
        let mut health: AccountHealthMap = BTreeMap::new();
        health.insert(pid("a"), health_quarantined_at(now - 1_000));
        health.insert(pid("b"), health_quarantined_at(now - 2_000));
        let r = pick_account_with_cooldown(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &health,
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
        let mut health: AccountHealthMap = BTreeMap::new();
        health.insert(pid("b"), health_quarantined_at(now - 500));
        let d = pick_account_with_cooldown(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &health,
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
            &health,
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
        let mut health: AccountHealthMap = BTreeMap::new();
        // "a" has 100% quota but is quarantined 5 s ago (inside 60 s window).
        health.insert(pid("a"), health_healthy_quarantined_at(now - 5_000));
        // "b" is fully healthy with 30% quota.
        health.insert(pid("b"), AccountHealth::healthy());
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
            &health,
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

    /// ST3: existing pick_account tests are unaffected — no cooldown field on
    /// `AccountHealth::healthy()` means default path is unchanged.
    #[test]
    fn pick_account_healthy_default_no_regression() {
        // Mirrors the existing unhealthy_member_is_skipped_in_fallback test
        // to confirm AccountHealth::healthy() still works identically.
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &["a", "b", "c"]);
        let mut health: AccountHealthMap = BTreeMap::new();
        health.insert(
            pid("b"),
            AccountHealth {
                state: HealthState::Unhealthy,
                consecutive_failures: 9,
                last_quarantined_at_unix_ms: None,
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

    /// in_cooldown returns false for None quarantine timestamp.
    #[test]
    fn in_cooldown_none_is_never_in_cooldown() {
        let cp = cooldown(60_000, 100_000);
        assert!(!cp.in_cooldown(&AccountHealth::healthy()));
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
        let h = AccountHealth {
            state: HealthState::Healthy,
            consecutive_failures: 0,
            last_quarantined_at_unix_ms: Some(UnixMillis(40_000)),
        };
        assert!(
            !cp.in_cooldown(&h),
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
                last_quarantined_at_unix_ms: Some(UnixMillis(now - 1_000)),
            },
        );
        health.insert(pid("b"), AccountHealth::healthy());
        let d = pick_account_with_cooldown(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &health,
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
                last_quarantined_at_unix_ms: Some(UnixMillis(now - 90_000)),
            },
        );
        let d = pick_account_with_cooldown(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &health,
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
        // "a" has no health entry; pool has one member.
        let p = pool_with(PoolRoutingStrategy::RoundRobin, &["a"]);
        let d = pick_account_with_cooldown(
            &p,
            &RequestMetadata::new("m".into()),
            &Default::default(),
            &Default::default(), // empty health map
            cooldown(60_000, now),
        )
        .unwrap();
        assert_eq!(
            d.account_id,
            pid("a"),
            "account absent from health map must be treated as eligible"
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
        let mut health: AccountHealthMap = BTreeMap::new();
        health.insert(pid("a"), health_quarantined_at(now - 1_000));
        health.insert(pid("b"), AccountHealth::healthy());
        let mut req = RequestMetadata::new("m".into());
        // previous_account = "a" (the account that was just quarantined).
        req.previous_account = Some(pid("a"));
        let d = pick_account_with_cooldown(
            &p,
            &req,
            &Default::default(),
            &health,
            cooldown(window, now),
        )
        .unwrap();
        assert_eq!(d.account_id, pid("b"), "must route away from cooldown account");
        assert_eq!(
            d.reason,
            PoolRoutingReason::FailoverFrom(pid("a")),
            "must emit FailoverFrom(prev) when previous account is in cooldown"
        );
    }
}
