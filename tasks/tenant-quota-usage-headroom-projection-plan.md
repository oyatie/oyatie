# Plan: tenant-quota-usage-headroom-projection

## Objective
Extend `oya-managed-k8s-tenant-quota-kernel` with pure, deterministic usage-aggregation
and headroom-projection logic. No I/O, no async. Pure domain values composable with the
existing `evaluate()` / `TenantQuota` model.

## Requirements analysis

### Core domain
- **`ClusterResourceUsage`**: snapshot of one cluster's resource consumption (nodes, vCPU, RAM).
- **`aggregate_usage()`**: fold N `ClusterResourceUsage` records for one tenant into a single
  `TenantResourceSummary` (total clusters, total/max nodes, total vCPU, total RAM).
- **`QuotaHeadroom`**: derived from `TenantQuota` + `TenantResourceSummary`; carries:
  - Remaining clusters / nodes_per_cluster / vcpu_per_cluster / ram_per_cluster (saturating subtraction)
  - Percent-utilized for each dimension (0–100 u8, saturating)
- **`project_headroom()`**: `(&TenantQuota, &TenantResourceSummary) -> Result<QuotaHeadroom, HeadroomError>`:
  - Returns `Err(HeadroomError::TenantMismatch)` on cross-tenant input — same guard as `evaluate()`.
  - O(1), no allocations, panic-free.

### Edge cases
1. Empty cluster list → headroom = full quota, 0% utilized (valid, no error).
2. Tenant mismatch between quota and summary → `HeadroomError::TenantMismatch`.
3. Summary usage exceeds quota (over-provisioned legacy state) → remaining=0, utilization=100%.
4. Saturating arithmetic throughout; no wrapping or panic on over-limit inputs.
5. Zero-cluster quota is rejected at `TenantQuota::new()` already (existing invariant).

### Acceptance criteria
- AC-1: `aggregate_usage(tenant, [])` returns summary with all zero counters.
- AC-2: `aggregate_usage(tenant, [c1, c2])` sums `total_vcpu`, `total_ram_gib`, `total_nodes`;
  `total_clusters == 2`.
- AC-3: `project_headroom(quota, summary_within)` returns headroom with correct remaining
  and utilization values.
- AC-4: `project_headroom(quota, summary_at_limit)` returns remaining=0, utilized=100 on all
  dimensions that are saturated.
- AC-5: `project_headroom(quota, summary_over_limit)` returns remaining=0, utilized=100
  (no panic, no overflow).
- AC-6: Tenant mismatch returns `Err(HeadroomError::TenantMismatch)`.
- AC-7: All types are `Serialize + Deserialize`.
- AC-8: `QuotaHeadroom` is compatible as a lifecycle pre-check input (compose with `evaluate()`).

### K8s/cloud-native implications
- `ClusterResourceUsage` mirrors K8s ResourceQuota status fields (`hard`, `used`).
- Naming follows K8s convention: vCPU = milli-cores aggregated as whole units; RAM = GiB.
- `aggregate_usage()` is the domain-level equivalent of a K8s aggregated ResourceQuota roll-up.

## Ordered subtasks

1. **Write plan** (this file) ✅
2. **Write spec** (`docs/specs/task-tenant-quota-usage-headroom-projection.md`)
3. **Write red tests** in `oya-managed-k8s-tenant-quota-kernel/src/lib.rs` (AC-1 through AC-8)
4. **Implement** `ClusterResourceUsage`, `TenantResourceSummary`, `aggregate_usage()`,
   `QuotaHeadroom`, `HeadroomError`, `project_headroom()` as a new `usage_projection` mod
   in `src/lib.rs` (flat-clean-arch: mods in one crate, no new workspace member)
5. **Run `cargo nextest run`** → green
6. **Self-review** (correctness / architecture / security / performance)
7. **Simplify** (guard clauses, naming, dead-code)
8. **Commit** and push; open PR
