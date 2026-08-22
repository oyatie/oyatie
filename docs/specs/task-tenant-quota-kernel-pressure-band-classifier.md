# Spec: tenant-quota-kernel-pressure-band-classifier

**Crate**: `managed-k8s-tenant-quota-kernel`  
**ADR**: ADR-0376 (Tier-3 pure kernel), ADR-0130 (SLO/alert inputs)  
**Status**: Accepted

## Purpose

Extend the quota kernel with a deterministic pressure classifier over `QuotaHeadroom`.
The classifier produces band-level signals (`QuotaPressure`) that feed ADR-0130
SLO alerting rules without any I/O, allocations on the hot path, or new dependencies.

## New Public Surface

### `ConstrainedDimension`

```rust
pub enum ConstrainedDimension { Clusters, Nodes, Vcpu, Ram }
```

- `as_str() -> &'static str`: `"clusters"`, `"nodes"`, `"vcpu"`, `"ram"`
- `parse(s: &str) -> Option<Self>`: fail-closed (unknown input returns `None`)
- Derives: `Clone, Debug, Eq, PartialEq, Serialize, Deserialize`

### `QuotaPressure`

```rust
pub enum QuotaPressure { Healthy, Warning, Critical, Exhausted }
```

**Pressure thresholds** (applied to the maximum `*_utilized_pct` across all four dimensions):

| max utilization pct | Band        |
|---------------------|-------------|
| `< 70`              | `Healthy`   |
| `70 ..< 90`         | `Warning`   |
| `90 ..< 100`        | `Critical`  |
| `== 100`            | `Exhausted` |

- `as_str() -> &'static str`: `"healthy"`, `"warning"`, `"critical"`, `"exhausted"`
- `parse(s: &str) -> Option<Self>`: fail-closed
- Derives: `Clone, Debug, Eq, PartialEq, Serialize, Deserialize`

### `QuotaHeadroom::most_constrained_dimension`

```rust
pub fn most_constrained_dimension(&self) -> ConstrainedDimension
```

Returns the dimension with the highest `*_utilized_pct`.

**Tie-break rule**: when two or more dimensions share the same maximum utilization
percentage, the canonical priority order is `Clusters → Nodes → Vcpu → Ram` (first
highest wins). This order is documented in rustdoc and is stable across platforms.

### `classify_pressure`

```rust
pub fn classify_pressure(headroom: &QuotaHeadroom) -> QuotaPressure
```

Free function (not a method) to keep the hot path zero-alloc. Computes
`max_pct = max(clusters_utilized_pct, nodes_utilized_pct, vcpu_utilized_pct, ram_utilized_pct)`
and maps to a `QuotaPressure` band using the threshold table above. Total,
deterministic, panic-free, O(1), no allocations.

## Acceptance Criteria

1. Band edge tests at pct values: 0, 69, 70, 89, 90, 99, 100.
2. Each dimension can independently be the constrained dimension.
3. Tie-break determinism: equal utilization across all dimensions returns `Clusters`.
4. Serde round-trip for `QuotaPressure` and `ConstrainedDimension`.
5. `classify_pressure` called on real `QuotaHeadroom` produced by `project_headroom`.
6. `cargo nextest run -p managed-k8s-tenant-quota-kernel` passes green.
7. `clippy::panic` and `clippy::unwrap_used` still denied in production code paths.
8. No new workspace member, no root `Cargo.toml` edit, no new dependencies.
