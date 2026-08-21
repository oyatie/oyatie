//! Per-AGENT-TOKEN reserve-then-reconcile quota.
//!
//! Implements the fairness/safety property that prevents one swarm agent from
//! draining the shared provider subscription pool:
//!
//! 1. **Estimate** prompt_tokens + max_tokens for the incoming request.
//! 2. **Reserve** the estimate against the agent/tenant's remaining budget
//!    (atomic: remaining >= estimate → deduct; else → `QuotaError::BudgetExceeded`).
//! 3. **Skip reserve** when headroom is ample (> [`QUOTA_AMPLE_THRESHOLD_PCT`]
//!    of budget remaining) to avoid hot-path writes.
//! 4. **Reconcile** on response: replace estimate with actual tokens consumed;
//!    credit back over-reserve or debit extra consumption (floor at 0).
//!
//! All quota state is keyed on `(TenantId, AgentToken)` — NOT source IP — so
//! NAT-fleet agents are correctly attributed.
//!
//! The in-memory adapter ([`InMemoryAgentQuotaStore`]) is the single-node
//! bring-up reference. A Valkey-backed adapter can satisfy the same port in
//! production without any caller change.
//!
//! data_class annotations follow the Oyatie catalog:
//! - `AgentToken` → TENANT_SCOPED (identifies an agent within a tenant)
//! - budget/remaining counters → INTERNAL_ONLY
//!
//! ADR-0083 Tier 3: panic-free in production code; tests use `unwrap`/`expect`.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use crate::{RepositoryError, TenantId};

// ─────────────────────────────────────────────────────────────────────────────
// Identity
// ─────────────────────────────────────────────────────────────────────────────

/// Opaque per-agent identity token. Carries no credential material.
///
/// Keyed on `(TenantId, AgentToken)` in the quota store — two agents with the
/// same string value but different `TenantId`s are completely isolated.
///
/// data_class: TENANT_SCOPED
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct AgentToken(pub String); // data_class: TENANT_SCOPED

impl fmt::Display for AgentToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Value types
// ─────────────────────────────────────────────────────────────────────────────

/// Budget configuration for one `(TenantId, AgentToken)` pair.
///
/// data_class: INTERNAL_ONLY
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AgentQuotaBudget {
    /// Total token budget for the current window.
    pub budget_tokens: u64, // data_class: INTERNAL_ONLY
    /// Unix-epoch milliseconds at which the window resets (0 = no reset).
    pub window_reset_unix_ms: u64, // data_class: INTERNAL_ONLY
}

/// Snapshot of the current quota state for one `(TenantId, AgentToken)` pair.
///
/// data_class: INTERNAL_ONLY
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AgentQuotaSnapshot {
    /// Total budget for the window.
    pub budget_tokens: u64, // data_class: INTERNAL_ONLY
    /// Currently available tokens (budget_tokens minus in-flight reserves).
    pub remaining_tokens: u64, // data_class: INTERNAL_ONLY
    /// When the window resets (0 = not configured).
    pub window_reset_unix_ms: u64, // data_class: INTERNAL_ONLY
}

// ─────────────────────────────────────────────────────────────────────────────
// Error
// ─────────────────────────────────────────────────────────────────────────────

/// Typed failure from a quota store operation.
///
/// data_class: INTERNAL_ONLY — detail fields must never echo agent payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QuotaError {
    /// The agent's remaining budget is insufficient for the requested reserve.
    BudgetExceeded {
        /// The agent whose budget was exceeded. data_class: TENANT_SCOPED
        agent: AgentToken,
        /// The number of tokens requested. data_class: INTERNAL_ONLY
        requested: u64,
        /// The remaining budget at the time of the check. data_class: INTERNAL_ONLY
        remaining: u64,
    },
    /// A backing-store failure.
    Repository(RepositoryError),
}

impl fmt::Display for QuotaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BudgetExceeded {
                agent,
                requested,
                remaining,
            } => write!(
                f,
                "quota budget exceeded for agent {}: requested {}, remaining {}",
                agent.0, requested, remaining
            ),
            Self::Repository(e) => write!(f, "quota store error: {e}"),
        }
    }
}

impl std::error::Error for QuotaError {}

// ─────────────────────────────────────────────────────────────────────────────
// Port
// ─────────────────────────────────────────────────────────────────────────────

/// Port for per-AGENT-TOKEN token-budget accounting.
///
/// Keyed on `(TenantId, AgentToken)`. Two agents with the same token value but
/// different `TenantId`s are completely isolated — the store MUST enforce this.
///
/// Production adapter: Valkey-backed CAS (future slice).
/// Reference adapter: [`InMemoryAgentQuotaStore`].
pub trait AgentQuotaStore: Send + Sync {
    /// Return the current budget snapshot for `(tenant_id, agent)`.
    ///
    /// If no budget has been configured for this pair, return a snapshot with
    /// `budget_tokens = 0` and `remaining_tokens = 0` (treat as unlimited when
    /// the caller is the dispatch path — the caller is responsible for
    /// interpreting an absent entry as "no quota configured").
    ///
    /// # Errors
    /// Returns [`QuotaError::Repository`] on store failure.
    fn snapshot(
        &self,
        tenant_id: &TenantId,
        agent: &AgentToken,
    ) -> Result<AgentQuotaSnapshot, RepositoryError>;

    /// Atomically reserve `tokens` from the agent's remaining budget.
    ///
    /// If `remaining >= tokens`, deducts `tokens` from `remaining` and returns
    /// `Ok(())`.
    ///
    /// If `remaining < tokens`, returns
    /// `Err(QuotaError::BudgetExceeded { agent, requested, remaining })` without
    /// mutating the store.
    ///
    /// # Errors
    /// Returns [`QuotaError::BudgetExceeded`] when insufficient budget, or
    /// [`QuotaError::Repository`] on store failure.
    fn reserve(
        &mut self,
        tenant_id: &TenantId,
        agent: &AgentToken,
        tokens: u64,
    ) -> Result<(), QuotaError>;

    /// Reconcile a previous reserve: replace the reserved `estimate` with
    /// `actual_used`.
    ///
    /// Semantics:
    /// - If `actual_used < estimate`: credit back `estimate - actual_used` to
    ///   `remaining` (cap at `budget_tokens`).
    /// - If `actual_used > estimate`: debit the extra `actual_used - estimate`
    ///   from `remaining` (floor at 0).
    /// - If equal: no-op on `remaining`.
    ///
    /// When the reserve was skipped (skip-when-ample), `estimate` is 0 and
    /// `actual_used` is the real usage. The store debits `actual_used` from
    /// `remaining` (floor at 0).
    ///
    /// # Errors
    /// Returns [`RepositoryError`] on store failure.
    fn reconcile(
        &mut self,
        tenant_id: &TenantId,
        agent: &AgentToken,
        estimate: u64,
        actual_used: u64,
    ) -> Result<(), RepositoryError>;
}

// ─────────────────────────────────────────────────────────────────────────────
// Skip-when-ample
// ─────────────────────────────────────────────────────────────────────────────

/// Fraction of budget that must be remaining for the reserve write to be
/// skipped on the hot path.
///
/// When `remaining_tokens * 100 / budget_tokens > QUOTA_AMPLE_THRESHOLD_PCT`,
/// [`should_skip_reserve`] returns `true` and the caller skips the reserve
/// write. Reconcile always runs after the response.
///
/// Value: 80 (i.e. >80% remaining → skip reserve).
/// data_class: INTERNAL_ONLY
pub const QUOTA_AMPLE_THRESHOLD_PCT: u64 = 80;

/// Return `true` when the agent's remaining budget is ample enough to skip the
/// reserve write.
///
/// Guard against division by zero: when `budget_tokens == 0`, returns `false`
/// (no skip — quota is fully exhausted or unconfigured).
///
/// # Hot-path intent
/// The caller should only skip the *reserve* step, not the *reconcile* step.
/// Reconcile always runs on success so actual usage is accurately tracked.
#[must_use]
pub fn should_skip_reserve(snap: &AgentQuotaSnapshot) -> bool {
    if snap.budget_tokens == 0 {
        return false;
    }
    // remaining_pct = remaining * 100 / budget (integer, truncated).
    let remaining_pct = snap.remaining_tokens.saturating_mul(100) / snap.budget_tokens;
    remaining_pct > QUOTA_AMPLE_THRESHOLD_PCT
}

// ─────────────────────────────────────────────────────────────────────────────
// In-memory reference adapter
// ─────────────────────────────────────────────────────────────────────────────

/// Internal per-entry state.
#[derive(Clone, Debug)]
struct QuotaEntry {
    budget_tokens: u64,        // data_class: INTERNAL_ONLY
    remaining_tokens: u64,     // data_class: INTERNAL_ONLY
    window_reset_unix_ms: u64, // data_class: INTERNAL_ONLY
}

/// In-memory [`AgentQuotaStore`] backed by a `BTreeMap` keyed by
/// `(TenantId, AgentToken)`.
///
/// Reference adapter for tests and single-node bring-up. Production swaps in
/// a Valkey-backed adapter behind the same port.
///
/// Interior mutability is `Arc<Mutex<_>>` so the store can be cloned cheaply
/// across test helper boundaries while sharing state.
#[derive(Clone, Debug, Default)]
pub struct InMemoryAgentQuotaStore {
    entries: Arc<Mutex<BTreeMap<(String, String), QuotaEntry>>>,
    // data_class: INTERNAL_ONLY
}

impl InMemoryAgentQuotaStore {
    /// Build an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed (or replace) the budget for `(tenant_id, agent)`.
    ///
    /// Must be called before [`AgentQuotaStore::reserve`] for a given pair.
    /// The initial `remaining_tokens` equals `budget.budget_tokens`.
    pub fn set_budget(&mut self, tenant_id: TenantId, agent: AgentToken, budget: AgentQuotaBudget) {
        if let Ok(mut guard) = self.entries.lock() {
            guard.insert(
                (tenant_id.0, agent.0),
                QuotaEntry {
                    budget_tokens: budget.budget_tokens,
                    remaining_tokens: budget.budget_tokens,
                    window_reset_unix_ms: budget.window_reset_unix_ms,
                },
            );
        }
    }
}

impl AgentQuotaStore for InMemoryAgentQuotaStore {
    fn snapshot(
        &self,
        tenant_id: &TenantId,
        agent: &AgentToken,
    ) -> Result<AgentQuotaSnapshot, RepositoryError> {
        let guard = self
            .entries
            .lock()
            .map_err(|_| RepositoryError::new("quota store mutex poisoned"))?;
        let key = (tenant_id.0.clone(), agent.0.clone());
        match guard.get(&key) {
            Some(entry) => Ok(AgentQuotaSnapshot {
                budget_tokens: entry.budget_tokens,
                remaining_tokens: entry.remaining_tokens,
                window_reset_unix_ms: entry.window_reset_unix_ms,
            }),
            // No entry → unlimited / unconfigured; return zeros so callers can
            // detect the absence.
            None => Ok(AgentQuotaSnapshot {
                budget_tokens: 0,
                remaining_tokens: 0,
                window_reset_unix_ms: 0,
            }),
        }
    }

    fn reserve(
        &mut self,
        tenant_id: &TenantId,
        agent: &AgentToken,
        tokens: u64,
    ) -> Result<(), QuotaError> {
        let mut guard = self.entries.lock().map_err(|_| {
            QuotaError::Repository(RepositoryError::new("quota store mutex poisoned"))
        })?;
        let key = (tenant_id.0.clone(), agent.0.clone());
        let entry = guard
            .get_mut(&key)
            .ok_or_else(|| QuotaError::Repository(RepositoryError::new("quota entry not found")))?;

        if entry.remaining_tokens < tokens {
            return Err(QuotaError::BudgetExceeded {
                agent: agent.clone(),
                requested: tokens,
                remaining: entry.remaining_tokens,
            });
        }
        entry.remaining_tokens -= tokens;
        Ok(())
    }

    fn reconcile(
        &mut self,
        tenant_id: &TenantId,
        agent: &AgentToken,
        estimate: u64,
        actual_used: u64,
    ) -> Result<(), RepositoryError> {
        let mut guard = self
            .entries
            .lock()
            .map_err(|_| RepositoryError::new("quota store mutex poisoned"))?;
        let key = (tenant_id.0.clone(), agent.0.clone());
        let entry = match guard.get_mut(&key) {
            Some(e) => e,
            // No entry → reconcile is a no-op (quota not configured for this agent).
            None => return Ok(()),
        };

        // Compute net delta: positive = credit back, negative = extra debit.
        if actual_used < estimate {
            // Over-reserved: credit back the difference, capped at budget.
            let credit = estimate - actual_used;
            entry.remaining_tokens = entry
                .remaining_tokens
                .saturating_add(credit)
                .min(entry.budget_tokens);
        } else if actual_used > estimate {
            // Under-reserved: debit the extra, floor at 0.
            let extra = actual_used - estimate;
            entry.remaining_tokens = entry.remaining_tokens.saturating_sub(extra);
        }
        // If equal: no-op.
        Ok(())
    }
}
