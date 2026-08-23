# Spec: sla-obs-kernel-fleet-rollup-summary

## Objective

Add a pure, deterministic tenant-fleet rollup function to the
`managed-k8s-sla-observability-kernel` crate. The function aggregates many
per-cluster `SlaSummary` values into a single `FleetSlaSummary` for a tenant,
using sample-weighted arithmetic to preserve statistical correctness.

## Crate Boundary

**Only crate modified**: `managed-k8s-sla-observability-kernel`
(`microservices/managed-k8s-sla-observability/crates/managed-k8s-sla-observability-kernel`)

No root `Cargo.toml` edit. No new workspace member. All logic stays in `src/lib.rs`.

## Mod Layout (flat-clean-arch per ADR-0509)

All code is inline in `src/lib.rs` following the existing pattern:
- Public structs/enums/functions exported at crate root
- Private helpers as module-level functions
- Unit tests in `#[cfg(test)] mod tests`

## New Public Surface

### `FleetSlaSummary` struct

```rust
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FleetSlaSummary {
    pub cluster_count: usize,                        // data_class: INTERNAL_ONLY
    pub available_count: usize,                      // data_class: INTERNAL_ONLY
    pub degraded_count: usize,                       // data_class: INTERNAL_ONLY
    pub unavailable_count: usize,                    // data_class: INTERNAL_ONLY
    pub worst_availability_state: AvailabilityState, // data_class: INTERNAL_ONLY
    pub aggregate_observed_basis_points: u16,        // data_class: INTERNAL_ONLY
    pub aggregate_target_basis_points: u16,          // data_class: INTERNAL_ONLY
    pub max_burn_rate_basis_points: u32,             // data_class: INTERNAL_ONLY
    pub any_error_budget_exhausted: bool,            // data_class: INTERNAL_ONLY
    pub provisioning_latency_breach_count: usize,    // data_class: INTERNAL_ONLY
}
```

Serde: `#[serde(rename_all = "snake_case")]` on all fields (default for structs).

### `summarize_fleet_sla` function

```rust
pub fn summarize_fleet_sla(
    summaries: &[SlaSummary],
) -> Result<FleetSlaSummary, SlaKernelError>
```

**Errors**: `EmptyObservationWindow` when `summaries` is empty.

## Contracts

### OpenAPI 3.2.0
The `FleetSlaSummary` DTO is serializable; the API layer (adapter crate) maps it
to the existing `GET /tenants/{tenant_id}/sla/fleet` endpoint contract extension.
No new OpenAPI file changes required by this kernel-only slice.

### AsyncAPI 3.1.0
Fleet rollup events are emitted by the adapter layer; the kernel function is pure.

### OpenSLO (ADR-0130)
Existing SLO files at `microservices/managed-k8s-sla-observability/slos/` remain
unchanged; this slice adds no new SLO objectives.

## Algorithm

### Sample-Weighted Availability

For N clusters with `(healthy_i, total_i, target_bps_i)`:

```
weighted_healthy = Σ healthy_i            [u128]
weighted_total   = Σ total_i              [u128]
aggregate_observed_bps = floor(weighted_healthy * 10_000 / weighted_total)  → clamp u16
aggregate_target_bps   = floor(Σ(target_bps_i * total_i) / weighted_total)  → clamp u16
```

Using `u128` intermediates guarantees no overflow even when all N clusters have
`total_i = u64::MAX` (worst case: N=1 → 2^64 * 10_000 < 2^128).

### Worst Availability State

Order: `Available (0) < Degraded (1) < Unavailable (2)`.
Select maximum across all cluster states.

### Max Burn Rate

`max_burn_rate_basis_points = summaries.iter().map(|s| s.error_budget.burn_rate_basis_points).max()`

### Any Error Budget Exhausted

`any_error_budget_exhausted = summaries.iter().any(|s| s.error_budget.exhausted)`

### Provisioning Latency Breach Count

`provisioning_latency_breach_count = summaries.iter().filter(|s| s.provisioning_latency.state == ProvisioningLatencyState::Breached).count()`

## Testing Strategy

All tests are pure unit tests — no I/O, no clock, no network.

| # | Test Name | Purpose |
|---|-----------|---------|
| 1 | `single_cluster_passthrough` | Single cluster; rollup mirrors its values |
| 2 | `empty_slice_fails_closed` | `[]` → `EmptyObservationWindow` |
| 3 | `mixed_states_worst_is_unavailable` | Available+Unavailable → worst=Unavailable |
| 4 | `all_exhausted_budget` | All clusters exhausted → `any_error_budget_exhausted=true` |
| 5 | `none_exhausted_budget` | No clusters exhausted → `any_error_budget_exhausted=false` |
| 6 | `overflow_safety_u64_max` | `u64::MAX` samples → no overflow, no panic |
| 7 | `weighting_correctness` | Large+small cluster; result weighted toward large |
| 8 | `serde_snake_case_round_trip` | JSON serialize → deserialize round-trip |
| 9 | `provisioning_breach_count` | Counts only `Breached` clusters |
| 10 | `all_available_worst_is_available` | All Available → worst=Available |

## Observability / SLO

This kernel slice is pure computation — no metrics emission. The adapter layer
is responsible for emitting OTel counters/histograms from `FleetSlaSummary`
values. Existing OpenSLO files cover availability and provisioning latency
objectives for individual clusters; fleet-level SLOs are future work.

## Security

- No external input beyond `&[SlaSummary]`; all data is already validated at
  `summarize_sla` call sites
- `INTERNAL_ONLY` data class on all fields; no tenant-scoped identity in fleet rollup
- No panics; arithmetic uses saturating ops and `u128` intermediates
