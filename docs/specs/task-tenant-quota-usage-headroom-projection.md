# Spec: tenant-quota-usage-headroom-projection

## Objective
Add pure, deterministic usage-aggregation and headroom-projection to the
`managed-k8s-tenant-quota-kernel` crate (ADR-0083 Tier-3: no I/O, no async).
These primitives feed lifecycle pre-checks and future billing/alerting pipelines.

## Crate boundary
Crate: `managed-k8s-tenant-quota-kernel` (sole modified crate).
No new workspace members. No edits to root `Cargo.toml`.

## Mod layout (flat-clean-arch, ADR-0509)

```
src/lib.rs
  mod usage_projection;   // NEW: ClusterResourceUsage, TenantResourceSummary,
                          //      QuotaHeadroom, HeadroomError,
                          //      aggregate_usage(), project_headroom()
  // existing: TenantQuota, TenantUsage, ProvisionRequest, QuotaDecision,
  //           DenyReason, QuotaModelError, RbacRole, RbacBinding, evaluate()
```

All new types live in `mod usage_projection` inside `src/lib.rs` (single-file crate).
They are re-exported at crate root so callers use `managed_k8s_tenant_quota_kernel::QuotaHeadroom`.

## Contracts

### New types

#### `ClusterResourceUsage`
```rust
pub struct ClusterResourceUsage {
    pub cluster_id: String,     // data_class: INTERNAL_ONLY
    pub node_count: u32,        // data_class: INTERNAL_ONLY
    pub vcpu_total: u32,        // data_class: INTERNAL_ONLY
    pub ram_gib_total: u32,     // data_class: INTERNAL_ONLY
}
```
Mirrors K8s ResourceQuota `status.used` fields for one cluster.

#### `TenantResourceSummary`
```rust
pub struct TenantResourceSummary {
    pub tenant_id: TenantId,    // data_class: INTERNAL_ONLY
    pub total_clusters: u32,    // data_class: INTERNAL_ONLY
    pub total_nodes: u32,       // data_class: INTERNAL_ONLY
    pub total_vcpu: u32,        // data_class: INTERNAL_ONLY
    pub total_ram_gib: u32,     // data_class: INTERNAL_ONLY
    pub max_nodes_per_cluster: u32,   // data_class: INTERNAL_ONLY
    pub max_vcpu_per_cluster: u32,    // data_class: INTERNAL_ONLY
    pub max_ram_gib_per_cluster: u32, // data_class: INTERNAL_ONLY
}
```
Aggregated view across all clusters for one tenant; feeds `project_headroom()`.

#### `QuotaHeadroom`
```rust
pub struct QuotaHeadroom {
    pub tenant_id: TenantId,               // data_class: INTERNAL_ONLY
    pub remaining_clusters: u32,           // data_class: INTERNAL_ONLY
    pub remaining_nodes_per_cluster: u32,  // data_class: INTERNAL_ONLY
    pub remaining_vcpu_per_cluster: u32,   // data_class: INTERNAL_ONLY
    pub remaining_ram_gib_per_cluster: u32, // data_class: INTERNAL_ONLY
    pub clusters_utilized_pct: u8,         // 0–100, saturating
    pub nodes_utilized_pct: u8,            // 0–100, saturating, vs max_nodes_per_cluster
    pub vcpu_utilized_pct: u8,             // 0–100, saturating, vs max_vcpu_per_cluster
    pub ram_utilized_pct: u8,              // 0–100, saturating, vs max_ram_gib_per_cluster
}
```

#### `HeadroomError`
```rust
pub enum HeadroomError {
    TenantMismatch { quota_tenant: String, summary_tenant: String },
}
```

### New functions

#### `aggregate_usage(tenant_id, usages) -> TenantResourceSummary`
- Pure fold over `&[ClusterResourceUsage]`.
- Returns summary with zeros when slice is empty.
- Saturating arithmetic on all aggregation; no overflow, no panic.
- O(n) in cluster count; no allocation beyond the returned struct.

#### `project_headroom(quota, summary) -> Result<QuotaHeadroom, HeadroomError>`
- Returns `Err(HeadroomError::TenantMismatch)` if `quota.tenant_id != summary.tenant_id`.
- `remaining_X = quota.max_X.saturating_sub(summary.X)`.
- `X_utilized_pct = min(100, summary.X * 100 / quota.max_X)` (integer, no float).
- O(1), no allocations.

## Testing strategy

All tests in `#[cfg(test)]` mod inside `src/lib.rs` (matching existing pattern).

| Test | Acceptance criterion |
|------|---------------------|
| `aggregate_empty_is_zero` | AC-1 |
| `aggregate_multiple_clusters` | AC-2 |
| `headroom_within_quota` | AC-3 |
| `headroom_at_limit` | AC-4 |
| `headroom_over_limit_saturates` | AC-5 |
| `headroom_tenant_mismatch` | AC-6 |
| `headroom_types_serde_roundtrip` | AC-7 |
| `headroom_compose_with_evaluate` | AC-8 |

## Observability / SLO
- `project_headroom()` is O(1) with no allocations on the hot path; sub-microsecond.
- Existing SLO file: `slos/quota-decision-latency.openslo.yaml` covers the evaluate path.
- No new SLO file required for this slice (no new I/O surface introduced).

## Security
- Cross-tenant guard in `project_headroom()` mirrors `evaluate()` — same `TenantMismatch` pattern.
- All types `#[forbid(unsafe_code)]` inherited from crate attribute.
- No floating-point arithmetic (avoids NaN/Inf in utilization calc).
