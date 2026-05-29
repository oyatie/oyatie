# Task Plan: pooling-kernel-cooldown-window

**Vertical:** intelligence
**Crate:** `oya-intelligence-provider-pool-kernel`
**Branch:** `feat/cd-pooling-kernel-cooldown-window`

---

## Objective

Extend the pure pool kernel with time-windowed cooldown:

1. Add `cooldown_until: Option<UnixMillis>` to `AccountHealth` — records the
   epoch at which an account should be re-admitted after a failure-driven
   quarantine event.
2. Add a `FailureKind` enum modelling the failure categories that trigger
   quarantine (e.g. `UpstreamRateLimit429`, `UpstreamServerError5xx`,
   `ConnectionTimeout`, `AuthFailure`).
3. Extend `CooldownPolicy` with a per-`FailureKind` exponential backoff table:
   - `UpstreamRateLimit429`: 30 s, 60 s, 120 s, 300 s (caps at 300 s)
   - `UpstreamServerError5xx`: 10 s, 30 s, 60 s (caps at 60 s)
   - `ConnectionTimeout`: 5 s, 15 s, 30 s (caps at 30 s)
   - `AuthFailure`: 60 s, 300 s, 900 s (caps at 900 s — escalates fast)
4. Add `CooldownPolicy::window_for(kind, consecutive_failures)` to compute the
   backoff window from the table.
5. Add `populate_quarantine_from_changes` helper that scans a slice of
   `PoolMembershipChange` events and inserts `Quarantined` entries into a
   `QuarantineMap`.
6. Keep `pick_account_with_cooldown` unchanged in external signature — it already
   takes a `QuarantineMap` and `CooldownPolicy`, so callers can compose the two
   new pieces without touching the routing entry point.
7. Pure unit tests: cooldown skip, backoff escalation, quarantine population.

---

## Subtasks (ordered)

### ST1 — `FailureKind` enum + `cooldown_until` on `AccountHealth`

**Acceptance:**
- `cargo check -p oya-intelligence-provider-pool-kernel --all-targets` passes.
- `AccountHealth::healthy()` still compiles (new field has a default).
- `FailureKind` is `Copy + Eq + Hash + Debug`.

### ST2 — Per-failure-kind backoff table in `CooldownPolicy`

**Acceptance:**
- `CooldownPolicy::window_for(FailureKind, consecutive_failures) -> DurationMs`
  returns correct ms values per table.
- Table is pure const; no heap allocation.

### ST3 — `populate_quarantine_from_changes`

**Acceptance:**
- Helper maps `PoolMembershipChange::Quarantined(id)` → `QuarantineMap` entry.
- Non-quarantine variants (`Added`, `Removed`) are ignored.

### ST4 — Unit tests

**Acceptance:**
- Test: cooldown skip (in-window quarantined account excluded).
- Test: backoff escalation (each `FailureKind`, consecutive_failures 1..=4+).
- Test: quarantine population from `PoolMembershipChange` slice.
- `cargo nextest run -p oya-intelligence-provider-pool-kernel` 37+ passed, 0 failed.

---

## Edge cases

- `consecutive_failures == 0` must be safe (treat as 1st failure = lowest tier).
- Overflow in `now - quarantined_at` guarded by `saturating_sub` (already present).
- `cooldown_until` on `AccountHealth` is informational — `pick_account_with_cooldown`
  consults `QuarantineMap` (not this field) for routing decisions; the field lets
  callers embed the expiry directly in a health snapshot without a separate map lookup.

---

## Status

| ST  | State     |
|-----|-----------|
| ST1 | pending   |
| ST2 | pending   |
| ST3 | pending   |
| ST4 | pending   |
