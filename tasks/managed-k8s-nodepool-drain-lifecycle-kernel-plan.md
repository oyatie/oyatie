# Plan: managed-k8s-nodepool-drain-lifecycle-kernel

Vertical: infra
Crate: oya-managed-k8s-cluster-lifecycle-kernel
Branch: feat/task-managed-k8s-nodepool-drain-lifecycle-kernel-2026-05-28

## Objective

Extend the existing `oya-managed-k8s-cluster-lifecycle-kernel` pure-domain crate
with node-pool operation value objects and a fail-closed drain-admission
decision function.  No new workspace crate, no I/O, no new dependencies beyond
the existing serde-only dep.

## Subtasks

### [np-1] NodePoolAction enum + NodePoolOpRequest + validate()

Add to `src/lib.rs`:

- `NodePoolAction` — enum with four variants: `ScaleUp`, `ScaleDown`, `Cordon`,
  `Drain`.  Derives `Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize,
  Deserialize`; `serde(rename_all = "snake_case")`.
- `NodePoolOpRequest` — struct carrying:
  - `tenant_id: String`        (data_class: TENANT_SCOPED)
  - `cluster_name: String`     (data_class: TENANT_SCOPED)
  - `target_node_count: u32`   (data_class: TENANT_SCOPED)
  - `action: NodePoolAction`   (data_class: TENANT_SCOPED)
  Derives `Clone, Debug, Eq, PartialEq, Serialize, Deserialize`.
- `NodePoolOpRequest::new(…) -> Result<Self, LifecycleValidationError>` —
  constructs + validates in one call (mirrors `LifecycleRequest::new`).
- `NodePoolOpRequest::validate(&self) -> Result<(), LifecycleValidationError>` —
  fail-closed:
  - `tenant_id.trim().is_empty()` → `LifecycleValidationError::EmptyTenantId`
  - `cluster_name.trim().is_empty()` → `LifecycleValidationError::EmptyClusterName`
  - `target_node_count == 0` → `LifecycleValidationError::ZeroTargetNodeCount`
  - `target_node_count > NODE_COUNT_CEILING` → `LifecycleValidationError::TargetNodeCountExceedsFloor`

Extend `LifecycleValidationError` with two new variants:
- `ZeroTargetNodeCount`  — "target_node_count must be > 0"
- `TargetNodeCountExceedsFloor` — "target_node_count exceeds maximum allowed"

Add `pub const NODE_COUNT_CEILING: u32 = 500;` as the configurable floor/ceiling
sentinel.

Acceptance:
- `cargo check -p oya-managed-k8s-cluster-lifecycle-kernel --all-targets` green
- `NodePoolAction` and `NodePoolOpRequest` are `pub`, serde-derived, and
  `validate()` returns `LifecycleValidationError` for empty identity and
  invalid target node counts

### [np-2] evaluate_drain_admission pure function + DrainAdmission type

Add:

- `pub const HOSTED_NODE_FLOOR: u32 = 1;` — minimum live nodes a Hosted pool
  must retain after a drain (configurable sentinel).
- `pub const DEDICATED_NODE_FLOOR: u32 = 3;` — minimum live nodes a Dedicated
  pool must retain after a drain (configurable sentinel).
- `pub enum DrainAdmission { Allow, Deny { reason: String } }` — derives
  `Clone, Debug, Eq, PartialEq`.
- `pub fn evaluate_drain_admission(current_nodes: u32, drain_target: u32,
  desired_tier: DesiredTier) -> DrainAdmission` — pure, deterministic,
  no I/O, no clocks:
  - `drain_target == 0` → `Deny { reason: "drain would reduce node count to zero" }`
  - `drain_target >= current_nodes` → `Deny { reason: "drain_target must be less than current_nodes" }`
  - post-drain remaining = `current_nodes - drain_target`; if remaining is less
    than the tier floor → `Deny { reason: "drain would drop <tier> cluster below
    node floor of <N>" }`
  - otherwise → `Allow`

Tier floors:
- `DesiredTier::Hosted` → `HOSTED_NODE_FLOOR`
- `DesiredTier::Dedicated` → `DEDICATED_NODE_FLOOR`

Acceptance:
- Function is pure (no filesystem/network/time), deterministic on identical inputs
- Returns explicit `Deny { reason }` rather than panicking
- Below-floor and drain-to-zero cases are denied
- A safe drain on a Hosted pool with headroom is allowed

### [np-3] Tests + spec doc

Add `#[cfg(test)]` inline tests in `src/lib.rs` (matching crate style):

| Test name | Coverage |
|-----------|----------|
| `nodepool_op_request_validates_happy_path` | `new()` succeeds with valid inputs |
| `nodepool_op_request_rejects_empty_tenant_id` | `EmptyTenantId` |
| `nodepool_op_request_rejects_empty_cluster_name` | `EmptyClusterName` |
| `nodepool_op_request_rejects_zero_target` | `ZeroTargetNodeCount` |
| `nodepool_op_request_rejects_over_ceiling` | `TargetNodeCountExceedsFloor` |
| `nodepool_action_serde_roundtrip` | All four `NodePoolAction` variants serialize + deserialize to/from snake_case JSON |
| `drain_admission_denies_to_zero` | `drain_target == current_nodes` |
| `drain_admission_denies_below_dedicated_floor` | Dedicated 4 nodes drain 2 → remaining 2 < floor 3 |
| `drain_admission_denies_below_hosted_floor` | Hosted 2 nodes drain 2 → zero → denied |
| `drain_admission_allows_safe_hosted_drain` | Hosted 5 nodes drain 2 → remaining 3 ≥ floor 1 → Allow |
| `drain_admission_allows_safe_dedicated_drain` | Dedicated 6 nodes drain 2 → remaining 4 ≥ floor 3 → Allow |
| `drain_admission_deterministic` | Same inputs called twice return the same variant |

Spec doc: `docs/specs/task-managed-k8s-nodepool-drain-lifecycle-kernel.md`

Acceptance:
- `cargo nextest run -p oya-managed-k8s-cluster-lifecycle-kernel` passes all
  new and pre-existing tests
- Every `NodePoolAction` has a serde round-trip test
- Drain admission covers below-floor, drain-to-zero, and safe-drain cases
- Lane-namespaced spec doc is committed

## Acceptance Summary

| Gate | Command |
|------|---------|
| Type-check | `cargo check -p oya-managed-k8s-cluster-lifecycle-kernel --all-targets` |
| Tests | `cargo nextest run -p oya-managed-k8s-cluster-lifecycle-kernel` |
| Diff scope | Additive only inside `crates/oya-managed-k8s-cluster-lifecycle-kernel/`; root `Cargo.toml` unchanged |

## Boundaries

- This task owns only `crates/oya-managed-k8s-cluster-lifecycle-kernel/` and the
  lane-namespaced docs under `docs/specs/` and `tasks/`.
- Root `Cargo.toml` is not touched.
- No new workspace crate is introduced.
- No I/O, no clocks, no new dependencies.
- All new types follow the naming pattern already established by
  `LifecycleRequest` / `DesiredTier` / `LifecycleValidationError`.
