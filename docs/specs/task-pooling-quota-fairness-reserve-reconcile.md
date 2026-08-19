# Spec: pooling-quota-fairness-reserve-reconcile

## Objective

Implement per-AGENT-TOKEN reserve-then-reconcile quota in the dispatch path of
`intelligence-provider-pool-app` to prevent a single swarm agent from
draining the shared provider subscription pool.

Re-key quota attribution on `(TenantId, AgentToken)` identity — NOT source IP —
because many agents share one egress IP behind NAT (the fleet correctness fix).

## Problem Context

- Multiple swarm agents dispatch through the same `ProviderAccountPool`.
- Without agent-level quota, a single agent can exhaust shared seats.
- IP-keyed quota is wrong in a NAT-fleet: all agents share one source IP.
- The fix: attribute quota to the agent's logical identity token.

## Contracts

### Identity type

```
AgentToken(pub String)   // data_class: TENANT_SCOPED
                         // Opaque per-agent identity; carries no credentials.
```

### Port: `AgentQuotaStore`

```rust
pub trait AgentQuotaStore: Send + Sync {
    /// Return the current budget snapshot for (tenant, agent).
    fn snapshot(&self, tenant_id: &TenantId, agent: &AgentToken)
        -> Result<AgentQuotaSnapshot, RepositoryError>;

    /// Atomically reserve `tokens` from the agent's remaining budget.
    /// Returns Ok(()) when budget was sufficient and deduction succeeded.
    /// Returns Err(QuotaError::BudgetExceeded { .. }) when remaining < tokens.
    fn reserve(&mut self, tenant_id: &TenantId, agent: &AgentToken, tokens: u64)
        -> Result<(), QuotaError>;

    /// Reconcile a previous reserve: replace the reserved `estimate` with
    /// `actual_used`. Credits back (estimate - actual_used) if over-reserved,
    /// or debits extra (actual_used - estimate) up to remaining = 0.
    fn reconcile(&mut self, tenant_id: &TenantId, agent: &AgentToken,
                 estimate: u64, actual_used: u64)
        -> Result<(), RepositoryError>;
}
```

### Value types

```
AgentQuotaSnapshot {
    budget_tokens: u64,      // Total budget for the window
    remaining_tokens: u64,   // Currently available tokens
    window_reset_unix_ms: u64, // When the window resets (0 = no reset configured)
}

QuotaError::BudgetExceeded {
    agent: AgentToken,
    requested: u64,
    remaining: u64,
}
QuotaError::Repository(RepositoryError)
```

### Skip-when-ample constant

```
QUOTA_AMPLE_THRESHOLD_PCT: u8 = 80
```

When `remaining_tokens * 100 / budget_tokens > QUOTA_AMPLE_THRESHOLD_PCT`,
the reserve write is skipped (hot-path write avoidance). Reconcile always runs.

### DispatchError extension

```
DispatchError::QuotaBudgetExceeded {
    agent: AgentToken,
    requested: u64,
    remaining: u64,
}
```

## Mod Layout (flat clean-arch, ADR-0509)

All code lives inside `intelligence-provider-pool-app/src/`:

```
src/
  lib.rs          — existing; pub use quota::* added
  quota.rs        — NEW: AgentToken, AgentQuotaSnapshot, QuotaError,
                        AgentQuotaStore port, InMemoryAgentQuotaStore,
                        QUOTA_AMPLE_THRESHOLD_PCT, should_skip_reserve()
```

The `dispatch_to_pool` signature gains one new parameter:

```rust
quota_store: Option<&mut (impl AgentQuotaStore + ?Sized)>
agent_token: Option<&AgentToken>
estimated_tokens: u64
```

When `quota_store` is `None`, quota is disabled and behaviour is identical to
the current implementation (zero-regression).

## Testing Strategy (hermetic unit tests)

File: `tests/quota_fairness.rs`

| Test | Assertion |
|------|-----------|
| `reserve_reduces_remaining_budget` | After reserve(100), remaining drops by 100 |
| `reserve_rejects_when_over_budget` | reserve when remaining < requested → QuotaError::BudgetExceeded |
| `reconcile_credits_back_over_reserve` | reserve(100) + reconcile(100, 60) → +40 credited back |
| `reconcile_debits_extra_consumption` | reserve(100) + reconcile(100, 150) → extra 50 debited (floor 0) |
| `skip_when_ample_returns_true_above_threshold` | remaining=90%, budget=100 → skip=true |
| `skip_when_ample_returns_false_at_threshold` | remaining=80%, budget=100 → skip=false |
| `skip_when_ample_returns_false_below_threshold` | remaining=50% → skip=false |
| `agent_isolation_separate_budgets` | AgentToken("a") and AgentToken("b") have independent state |
| `tenant_isolation_separate_budgets` | Same AgentToken in tenant_a and tenant_b have independent state |
| `dispatch_rejects_over_budget_request` | dispatch_to_pool with exhausted quota returns QuotaBudgetExceeded |
| `dispatch_reconciles_actual_after_success` | After successful dispatch, remaining reflects actual tokens |
| `dispatch_without_quota_store_unchanged` | quota_store=None → existing behaviour preserved |
| `dispatch_skip_when_ample_no_reserve_write` | ample headroom → reserve not called, reconcile still runs |

## Observability / SLO

No new OpenSLO file required for this slice (no new µservice, same crate).
The existing OtelMetricsSink records dispatch outcomes. Quota rejections surface
as `DispatchError::QuotaBudgetExceeded` — callers (and a future OTel bridge)
can record `provider_pool.quota.budget_exceeded` counter as a follow-up.

## Crate Boundary

All changes confined to `intelligence/core/provider-pool-app/`.
No changes to `Cargo.toml` (no new external deps: uses only `std`, `BTreeMap`, `Arc<Mutex>`).
No changes to root `Cargo.toml` or any other workspace member.
