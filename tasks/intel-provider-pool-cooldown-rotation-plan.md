# Task Plan: intel-provider-pool-cooldown-rotation

**Vertical:** intelligence  
**Crate:** `oya-intelligence-provider-pool-kernel`  
**Branch:** `feat/task-intel-provider-pool-cooldown-rotation-2026-05-28`

---

## Objective

Extend the pure-value provider-pool rotation kernel with quota-aware cooldown/quarantine
rotation. A std-only function must, given pool members, an `AccountHealthMap`, a
`UsageSnapshotMap`, the pool's `anti_correlation_window_ms`, and the current `UnixMillis`,
exclude accounts that are `Unhealthy` or were quarantined within the anti-correlation
cooldown window, re-admit accounts whose cooldown has elapsed, and emit a
`PoolRoutingReason::FailoverFrom` fallback chain. No new I/O, no async, no new crate.

---

## Subtasks

### ST1 — CooldownPolicy + last_quarantined_at surface

**Goal:** Add a pure-value `CooldownPolicy` input type (window derived from
`pool.anti_correlation_window_ms` + `now: UnixMillis`) and a
`last_quarantined_at_unix_ms: Option<UnixMillis>` field on `AccountHealth` so the kernel
can compute whether an account is still inside the cooldown window.

**Acceptance criteria:**
- `cargo check -p oya-intelligence-provider-pool-kernel --all-targets` passes.
- New public types compile with zero new dependencies added to `Cargo.toml`.
- Rustdoc on new types documents `data_class` and cooldown semantics.

---

### ST2 — `pick_account_with_cooldown` pure function

**Goal:** Implement a pure function `pick_account_with_cooldown` that:
1. Filters to candidates that are healthy (`HealthState` != `Unhealthy`) AND
   out-of-cooldown (either `last_quarantined_at_unix_ms` is `None` or
   `now - last_quarantined_at >= window`).
2. Falls back through still-eligible members producing
   `PoolRoutingReason::FailoverFrom` entries in the fallback chain.
3. Returns `PoolError::NoHealthyMembers` when every member is unhealthy or in cooldown.

**Acceptance criteria:**
- `cargo nextest run -p oya-intelligence-provider-pool-kernel` green.
- Tests cover:
  - account in cooldown is excluded.
  - account whose cooldown has elapsed is re-admitted.
  - all-in-cooldown (or all-unhealthy) → `PoolError::NoHealthyMembers`.
  - fallback chain ordering is deterministic (BTree-backed iteration order).

---

### ST3 — Cooldown filtering before quota selection

**Goal:** Ensure that cooldown filtering is applied *before* quota-strategy selection
so a quarantined high-quota account is not chosen over a healthy lower-quota account.

**Acceptance criteria:**
- Test asserts a quarantined account with 100 % remaining quota is skipped in favour
  of a healthy account with lower remaining quota.
- Existing `pick_account` tests remain green (no behavioural regression on the
  non-cooldown entrypoint).

---

## Implementation notes

- `CooldownPolicy` is a pure input struct carrying `(window_ms: DurationMs, now: UnixMillis)`;
  the kernel computes `elapsed = now.0 - last_quarantined.0` without any `time` crate import.
- `pick_account_with_cooldown` reuses the existing internal strategy helpers
  (`least_remaining`, `least_used`, etc.) after the cooldown pre-filter.
- Fallback chain in the new function mirrors the existing pattern: everyone except chosen,
  in BTree order, capped at healthy-eligible.len()-1.
- The existing `pick_account` entry point is not modified; ST3 verifies the two entry
  points are compositionally consistent.

---

## Status

| ST  | State   |
|-----|---------|
| ST1 | pending |
| ST2 | pending |
| ST3 | pending |
