# Plan: sla-obs-kernel-fleet-rollup-summary

## Objective

Add a pure, deterministic tenant-fleet rollup that aggregates many per-cluster
`SlaSummary` values into one `FleetSlaSummary` for a tenant.

## Requirements Analysis

### Core Function
`summarize_fleet_sla(summaries: &[SlaSummary]) -> Result<FleetSlaSummary, SlaKernelError>`

### Output Fields
- `cluster_count: usize` — total clusters in the rollup
- `available_count: usize` — clusters in `AvailabilityState::Available`
- `degraded_count: usize` — clusters in `AvailabilityState::Degraded`
- `unavailable_count: usize` — clusters in `AvailabilityState::Unavailable`
- `worst_availability_state: AvailabilityState` — max severity across all clusters
- `aggregate_observed_basis_points: u16` — sample-weighted observed availability
- `aggregate_target_basis_points: u16` — sample-weighted target availability
- `max_burn_rate_basis_points: u32` — max burn rate across all clusters
- `any_error_budget_exhausted: bool` — true if any cluster has exhausted budget
- `provisioning_latency_breach_count: usize` — count of clusters with `ProvisioningLatencyState::Breached`

### Edge Cases
1. **Empty slice**: fails closed with `SlaKernelError::EmptyObservationWindow`
2. **Overflow safety**: use `u128` intermediates for sample-weighted aggregation
   since individual `total_status_samples` can be `u64::MAX`
3. **Worst-case state ordering**: `Unavailable > Degraded > Available`
4. **All exhausted**: `any_error_budget_exhausted = true`
5. **No provisioning breaches**: `provisioning_latency_breach_count = 0`

### Aggregation Algorithm (sample-weighted availability)
```
aggregate_observed = sum(healthy * 10_000 / total, weighted) =
  floor(sum(healthy_samples_i * 10_000) / sum(total_samples_i))

using u128 accumulators:
  weighted_healthy = sum_i(healthy_status_samples_i)  [u128]
  weighted_total   = sum_i(total_status_samples_i)     [u128]
  aggregate_observed_bps = (weighted_healthy * 10_000) / weighted_total
  clamped to u16 (max 10_000)
```

### Worst Availability State
AvailabilityState has no Ord. Define ordering:
`Available (0) < Degraded (1) < Unavailable (2)`
Pick maximum across all clusters.

### Target Aggregation
Sample-weighted target mirrors the observed calculation:
```
weighted_target_sum = sum_i(target_basis_points_i * total_samples_i)  [u128]
aggregate_target_bps = weighted_target_sum / weighted_total
clamped to u16
```

## Subtasks (ordered)

1. Write plan (this file) ✓
2. Write spec doc
3. Write tests (RED) — 8+ unit tests covering acceptance criteria
4. Implement `FleetSlaSummary` struct + `summarize_fleet_sla` function (GREEN)
5. Self-review (correctness/security/perf/cloud-native)
6. Simplify (guard clauses, naming, dead code)
7. Verify: `cargo nextest run -p oya-managed-k8s-sla-observability-kernel`
8. Commit + push + PR

## Acceptance Criteria

- Empty slice returns `Err(SlaKernelError::EmptyObservationWindow)`
- Sample-weighted aggregation uses `u128` intermediates
- `u64::MAX` sample counts do not overflow
- Serde snake_case round-trip works
- `worst_availability_state` is `Unavailable` when any cluster is unavailable
- `any_error_budget_exhausted` true when any cluster exhausted
- `provisioning_latency_breach_count` counts only `Breached` clusters
- At least 8 unit tests pass
- No root `Cargo.toml` edit
- No changes outside `oya-managed-k8s-sla-observability-kernel`
