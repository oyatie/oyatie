# pooling-quota-fairness-reserve-reconcile — Task Plan

## Problem

The provider-pool dispatch path routes requests through shared subscription
accounts (seats). A single swarm agent can exhaust the shared pool by issuing
many requests in parallel, starving other agents/tenants of quota.

The root NAT-fleet correctness issue: keying quota on the source IP is wrong
because many agents share one egress IP behind a NAT. Re-keying on
AGENT/TENANT identity (the `TenantId` + a logical `AgentToken` identity) fixes
the attribution.

## Solution: reserve-then-reconcile

Pattern lifted from one-api per-token reserve-then-reconcile, re-keyed on
AGENT/TENANT identity (not IP):

1. **Estimate** prompt_tokens + max_tokens for the incoming request.
2. **Reserve** that token estimate against the agent/tenant's budget in the
   quota store (atomic CAS: remaining >= estimate → deduct estimate).
3. **Reject** immediately with `DispatchError::QuotaBudgetExceeded` if the
   agent's remaining budget is < estimate (no transport call, no health mutation).
4. **Dispatch** the request (existing pipeline unchanged).
5. **Reconcile** on response: replace the estimate with actual tokens consumed,
   crediting back any over-reserve (actual_used < estimate) or charging the
   extra (capped at remaining = 0).

### Hot-path write avoidance (skip-when-ample)

When an agent has ample headroom (> `QUOTA_AMPLE_THRESHOLD_PCT` of their
budget remaining), skip the reserve step entirely to avoid a write on the
hot path. The reconcile step always runs so actual usage is accurately tracked.

Threshold: if `remaining_tokens > budget_tokens * QUOTA_AMPLE_THRESHOLD_PCT / 100`
→ skip reserve (only reconcile after response).

## Ordered Subtasks

1. **PLAN**: Write this file.
2. **SPEC**: Write `docs/specs/task-pooling-quota-fairness-reserve-reconcile.md`.
3. **Port**: Add `AgentQuotaStore` port and `AgentToken` identity type in
   `src/quota.rs` inside `oya-intelligence-provider-pool-app`.
4. **In-memory adapter**: `InMemoryAgentQuotaStore` with `Arc<Mutex<BTreeMap>>`.
5. **Logic**: `reserve_tokens`, `reconcile_tokens`, `should_skip_reserve` functions.
6. **Integration**: Thread quota store into `dispatch_to_pool` as an optional
   parameter (None = quota disabled for backwards compat).
7. **RED tests**: Write tests in `tests/quota_fairness.rs` — confirm they fail.
8. **GREEN**: Implement minimum code to pass all tests.
9. **REVIEW**: Self-review correctness / security / performance.
10. **SIMPLIFY**: Behavior-preserving cleanup.

## Acceptance Criteria

- `reserve_tokens` reduces remaining budget by estimate; returns
  `QuotaBudgetExceeded` if insufficient.
- `reconcile_tokens` adjusts the reservation to actual usage (credit back
  over-reserve or debit extra consumption), flooring at 0.
- `skip_when_ample` returns `true` when remaining > threshold; the reserve
  write is skipped.
- Agent isolation: two different `AgentToken` values share no budget state.
- Tenant isolation: same `AgentToken` in different tenants shares no state.
- `dispatch_to_pool` with a `QuotaStore` configured: rejects over-budget
  requests before transport; reconciles after success.
- `dispatch_to_pool` with `quota_store = None`: existing behaviour unchanged
  (zero-regression on existing acceptance tests).
- All tests hermetic: no real upstream, no network.
- `cargo check -p oya-intelligence-provider-pool-app --all-targets` clean.
- `cargo nextest run -p oya-intelligence-provider-pool-app` green.
