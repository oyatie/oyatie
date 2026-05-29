# Plan: tenant-quota-kernel-pressure-band-classifier

## Objective
Extend `oya-managed-k8s-tenant-quota-kernel` with a quota-pressure classifier that
operates on `QuotaHeadroom` to produce band-level signals for ADR-0130 SLO/alert
inputs. Pure deterministic kernel; no I/O, no new deps.

## Scope
Crate: `oya-managed-k8s-tenant-quota-kernel`
Files modified: `src/lib.rs` only

## Tasks

### 1. Add `ConstrainedDimension` enum
- Variants: `Clusters`, `Nodes`, `Vcpu`, `Ram`
- Derive: `Clone, Debug, Eq, PartialEq, Serialize, Deserialize`
- `as_str()` -> `&'static str` (lowercase: `"clusters"`, `"nodes"`, `"vcpu"`, `"ram"`)
- `parse(s: &str) -> Option<Self>` (fail-closed: unknown string -> None)

### 2. Add `QuotaPressure` enum
- Variants: `Healthy`, `Warning`, `Critical`, `Exhausted`
- Thresholds (applied to max utilization pct across dimensions):
  - `< 70` -> `Healthy`
  - `70..< 90` -> `Warning`
  - `90..< 100` -> `Critical`
  - `== 100` -> `Exhausted`
- Derive: `Clone, Debug, Eq, PartialEq, Serialize, Deserialize`
- `as_str()` -> `&'static str` (lowercase: `"healthy"`, `"warning"`, `"critical"`, `"exhausted"`)
- `parse(s: &str) -> Option<Self>` (fail-closed)

### 3. Add `QuotaHeadroom::most_constrained_dimension()`
- Returns `ConstrainedDimension` based on highest `*_utilized_pct` field
- Tie-break: deterministic priority order `Clusters > Nodes > Vcpu > Ram`
  (first-highest-wins in that order)

### 4. Add `classify_pressure(headroom: &QuotaHeadroom) -> QuotaPressure`
- Free function (not a method) for zero-alloc hot path
- Compute `max_pct` = max of all four `*_utilized_pct` fields
- Apply threshold bands per spec
- Panic-free, total, O(1), zero-alloc

### 5. Tests (red -> green)
- Band edge tests: 0, 69, 70, 89, 90, 99, 100 pct values
- Each dimension being the constrained one
- Tie-break determinism (equal utilization across dimensions)
- Serde round-trip for `QuotaPressure` and `ConstrainedDimension`
- `classify_pressure` called on real `QuotaHeadroom` from `project_headroom`

## Tie-break Rule (documented in rustdoc)
When two or more dimensions share the same maximum utilization percentage,
`most_constrained_dimension` returns the first match in the canonical order:
`Clusters → Nodes → Vcpu → Ram`.
This is stable across platforms and Rust versions (no floating point, no hash).
