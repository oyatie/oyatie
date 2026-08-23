# Spec: intel-autonomy-ceiling-tenant-tier-policy

## Objective

Extend `intelligence-autonomy-ceiling-kernel` with a per-surface tenant ceiling
policy: a `TenantCeiling` value type carrying per-surface max tiers, plus a pure
`resolve(requested_tier, surface, ceiling) -> TenantCeilingVerdict` function with
variants `Permitted | Clamped(AutonomyTier) | Denied`.

T4Actuate stays **disabled by default** unless the ceiling explicitly permits it for
a given surface.

## Crate boundary

Only `intelligence-autonomy-ceiling-kernel` is modified. No workspace-level
changes. No new crates.

## Flat clean-arch mod layout (ADR-0509)

```
src/
  lib.rs          — existing tier comparison + new tenant_ceiling mod declaration
  tenant_ceiling.rs — TenantCeiling, TenantCeilingVerdict, resolve()
```

## Types

### `TenantCeilingVerdict`

```rust
pub enum TenantCeilingVerdict {
    Permitted,
    Clamped(AutonomyTier),   // effective ceiling the request was clamped to
    Denied,                  // reserved: ceiling below T1Read (future-proof)
}
```

### `TenantCeiling`

```rust
pub struct TenantCeiling {
    global: AutonomyTier,                        // default T3PropAct
    surfaces: HashMap<String, AutonomyTier>,     // per-surface overrides
}
```

Implements `Clone, Debug, PartialEq, Default` (default = global T3PropAct, no overrides).

### `resolve`

```rust
pub fn resolve(
    requested: AutonomyTier,
    surface: &str,
    ceiling: &TenantCeiling,
) -> TenantCeilingVerdict
```

Algorithm:
1. Effective ceiling = `ceiling.surfaces.get(surface).copied().unwrap_or(ceiling.global)`.
2. If `requested <= effective` → `Permitted`.
3. Else → `Clamped(effective)` (since effective >= T1Read always, Denied is unreachable with current types but kept as variant for extensibility).

## Contracts

- Pure function; no I/O, no side effects.
- `TenantCeiling` is `Clone + Send + Sync` (no `Rc` / `Cell` inside).
- Empty surface string `""` treated same as any other key (no special-case: if not in map, falls back to global).

## Testing strategy

All tests are pure unit tests in `src/tenant_ceiling.rs` under `#[cfg(test)]`:

| Test | Description |
|---|---|
| `default_ceiling_permits_t1` | T1Read permitted by default ceiling |
| `default_ceiling_permits_t3` | T3PropAct permitted (equals global) |
| `default_ceiling_clamps_t4` | T4Actuate clamped to T3PropAct with default ceiling |
| `surface_override_permits_t4` | T4 surface override allows T4 |
| `surface_override_clamps_to_surface_tier` | surface=T2 clamps T3 to T2 |
| `unknown_surface_falls_back_to_global` | surface not in map uses global ceiling |
| `t4_global_override_permits_t4` | global ceiling set to T4 permits T4 |
| `clamped_carries_effective_tier` | Clamped variant carries the surface ceiling tier |
| `tenant_ceiling_clone_eq` | TenantCeiling implements Clone + PartialEq |
| `builder_with_surface` | with_surface() builder method works |

## Observability / SLO

This is a pure kernel crate — no runtime metrics emitted here. The calling app layer
wraps `resolve()` calls with OTel spans and records `CeilingVerdict` as a span
attribute.

## Acceptance evidence

`cargo nextest run -p intelligence-autonomy-ceiling-kernel` must pass with all
tests green, including the 10 new tests and the 7 existing tests.
