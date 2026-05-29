# Plan: intel-autonomy-ceiling-tenant-tier-policy

## Objective

Extend `oya-intelligence-autonomy-ceiling-kernel` with a **tenant ceiling policy** that:

1. Introduces a `TenantCeiling` value type carrying **per-surface** maximum tiers (a surface is a named scope, e.g. `"write"`, `"execute"`, `"read"`).
2. Provides a pure `resolve(requested_tier, surface, ceiling) -> CeilingVerdict` function with three outcomes: `Permitted`, `Clamped(to)`, `Denied`.
3. Keeps T4Actuate **disabled by default** unless the ceiling explicitly permits it.

## Edge Cases

| Scenario | Expected behaviour |
|---|---|
| Surface not in ceiling map | Falls back to the global ceiling (T3PropAct by default) |
| Requested == surface ceiling | `Permitted` |
| Requested < surface ceiling | `Permitted` |
| Requested > surface ceiling but surface ceiling is still valid | `Clamped(surface_ceiling)` |
| Surface ceiling is None/unset + global ceiling blocks | `Denied` if requested > global, else `Permitted` |
| T4Actuate requested, no explicit T4 permission | `Clamped(T3PropAct)` |
| T4Actuate requested, ceiling has explicit T4 for that surface | `Permitted` |
| Empty surface name | Treated as "no surface override" — falls back to global |
| Denied vs Clamped distinction | `Denied` means the ceiling is *below* T1Read (impossible in current tier enum) — kept as a future-proof variant; in practice every ceiling is >= T1Read so the function returns `Permitted` or `Clamped`. |

## Acceptance Criteria

1. `TenantCeiling::default()` has global ceiling T3PropAct and no per-surface overrides.
2. `resolve(T4Actuate, "write", &default_ceiling)` returns `Clamped(T3PropAct)`.
3. `resolve(T4Actuate, "write", &ceiling_with_t4_for_write)` returns `Permitted`.
4. `resolve(T2Suggest, "read", &default_ceiling)` returns `Permitted`.
5. `resolve(T3PropAct, "execute", &ceiling_with_t2_for_execute)` returns `Clamped(T2Suggest)`.
6. `TenantCeiling` is a value type (Clone, Debug, PartialEq).
7. `CeilingVerdict` is extended (or a new parallel enum introduced) with `Permitted | Clamped(AutonomyTier) | Denied`.
8. All existing tests still pass.

## K8s / Cloud-Native Implications

- Pure value/enum logic only — no I/O, no network. Safe to use in both request-handling hot path and admission-webhook critical path.
- `TenantCeiling` is `Clone + Send + Sync` so it can be held in a `DashMap` or `Arc<RwLock<_>>` at the app layer.
- No async surface — callers gate async themselves.

## Subtasks (ordered)

1. Write plan (this file).
2. Write spec (`docs/specs/task-intel-autonomy-ceiling-tenant-tier-policy.md`).
3. Write RED tests in `oya-intelligence-autonomy-ceiling-kernel/src/lib.rs` (new `tenant_ceiling` mod).
4. Implement `TenantCeiling` value type + `resolve` function.
5. Extend or introduce `CeilingVerdict` variants `Permitted`/`Clamped`/`Denied`.
6. Verify GREEN: `cargo nextest run -p oya-intelligence-autonomy-ceiling-kernel`.
7. Self-review (correctness, security, architecture).
8. Simplify and final green check.
9. Commit + push + PR.
