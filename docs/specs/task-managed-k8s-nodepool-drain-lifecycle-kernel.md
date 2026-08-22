# Spec: Managed-K8s Node-Pool Drain Lifecycle Kernel

| Field | Value |
|-------|-------|
| Task slug | `managed-k8s-nodepool-drain-lifecycle-kernel` |
| Vertical | infra |
| Crate | `managed-k8s-cluster-lifecycle-kernel` |
| Branch | `feat/task-managed-k8s-nodepool-drain-lifecycle-kernel-2026-05-28` |
| Stage | SPEC |

## Objective

Extend the pure-domain `managed-k8s-cluster-lifecycle-kernel` crate with:

1. A **NodePoolOp** request model — `NodePoolAction` enum (ScaleUp, ScaleDown,
   Cordon, Drain) and `NodePoolOpRequest` value object — with fail-closed
   validation mirroring the existing `LifecycleRequest` pattern.
2. A **drain-admission** pure function — `evaluate_drain_admission` — that
   computes an Allow / Deny decision from desired-vs-current node counts and
   tier constraints without any I/O or clock access.

Scope is pure value-object and decision logic only.  No new workspace crate,
no new dependencies, no HTTP/gRPC handlers, no provider adapters.

## Vertical and Crate Context

`managed-k8s-cluster-lifecycle-kernel` already owns:

- `DesiredTier` — `Hosted` | `Dedicated` with `parse()` + `as_str()`
- `ClusterResourceRequest` — nodes/vcpu/ram_gib with internal `validate()`
- `LifecycleRequest` — tenant_id, cluster_name, desired_tier, resources with
  public `validate()` and constructor-validates `new()`
- `LifecycleValidationError` — `EmptyTenantId`, `EmptyClusterName`,
  `ZeroResource(&'static str)`

The node-pool surface is additive.  No existing type signatures are altered.

## Module Layout (flat clean-arch, mods inside `src/lib.rs`)

```
src/lib.rs
  // --- existing surface (unchanged) ---
  pub enum DesiredTier
  pub struct ClusterResourceRequest
  pub struct LifecycleRequest
  pub enum LifecycleValidationError

  // --- new node-pool op surface (additive) ---
  pub const NODE_COUNT_CEILING: u32
  pub enum NodePoolAction          // ScaleUp | ScaleDown | Cordon | Drain
  pub struct NodePoolOpRequest     // tenant_id, cluster_name, target_node_count, action
  impl NodePoolOpRequest
    pub fn new(…) -> Result<Self, LifecycleValidationError>
    pub fn validate(&self) -> Result<(), LifecycleValidationError>

  // --- new drain-admission surface (additive) ---
  pub const HOSTED_NODE_FLOOR: u32
  pub const DEDICATED_NODE_FLOOR: u32
  pub enum DrainAdmission          // Allow | Deny { reason: String }
  pub fn evaluate_drain_admission(current_nodes: u32, drain_target: u32,
                                  desired_tier: DesiredTier) -> DrainAdmission

  // --- extended error variants (additive) ---
  // LifecycleValidationError::ZeroTargetNodeCount
  // LifecycleValidationError::TargetNodeCountExceedsFloor
```

All new code lives inside the existing single `src/lib.rs` file per the
flat-clean-arch / single-crate-per-service doctrine (ADR-0509).

## Contracts

### OpenAPI 3.2.0 fragment — node-pool op

```yaml
paths:
  /v1/clusters/{cluster_name}/node-pools/{pool_id}/ops:
    post:
      operationId: submitNodePoolOp
      summary: Submit a node-pool operation (scale/cordon/drain)
      parameters:
        - name: cluster_name
          in: path
          required: true
          schema:
            type: string
            example: "dogfood-a"
        - name: pool_id
          in: path
          required: true
          schema:
            type: string
            example: "default"
        - name: X-Tenant-Id
          in: header
          required: true
          schema:
            type: string
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/NodePoolOpRequest"
      responses:
        "202":
          description: Operation accepted
        "400":
          $ref: "#/components/responses/BadRequest"
        "403":
          $ref: "#/components/responses/Forbidden"
        "422":
          $ref: "#/components/responses/UnprocessableEntity"

components:
  schemas:
    NodePoolOpRequest:
      type: object
      required: [tenant_id, cluster_name, target_node_count, action]
      properties:
        tenant_id:
          type: string
          example: "ten_alpha"
        cluster_name:
          type: string
          example: "dogfood-a"
        target_node_count:
          type: integer
          format: int32
          minimum: 1
          example: 3
        action:
          type: string
          enum: [scale_up, scale_down, cordon, drain]
    DrainAdmissionResponse:
      type: object
      required: [decision]
      properties:
        decision:
          type: string
          enum: [allow, deny]
        reason:
          type: string
          description: Present when decision is "deny"
```

### Proto3 fragment

```proto
syntax = "proto3";
package oya.managed_k8s.v1;

enum NodePoolAction {
  NODE_POOL_ACTION_UNSPECIFIED = 0;
  NODE_POOL_ACTION_SCALE_UP    = 1;
  NODE_POOL_ACTION_SCALE_DOWN  = 2;
  NODE_POOL_ACTION_CORDON      = 3;
  NODE_POOL_ACTION_DRAIN       = 4;
}

message NodePoolOpRequest {
  string tenant_id         = 1;
  string cluster_name      = 2;
  uint32 target_node_count = 3;
  NodePoolAction action     = 4;
}

enum DrainDecision {
  DRAIN_DECISION_UNSPECIFIED = 0;
  DRAIN_DECISION_ALLOW       = 1;
  DRAIN_DECISION_DENY        = 2;
}

message DrainAdmissionResponse {
  DrainDecision decision = 1;
  string reason          = 2;
}
```

## NodePoolAction Variant Semantics

| Variant | Description |
|---------|-------------|
| `ScaleUp` | Increase target_node_count beyond current pool size |
| `ScaleDown` | Reduce target_node_count (must pass drain admission if nodes must be evicted) |
| `Cordon` | Mark nodes unschedulable without eviction; target_node_count = nodes to cordon |
| `Drain` | Evict workloads from target_node_count nodes; subject to drain-admission gate |

## Drain-Admission Decision Logic

```
evaluate_drain_admission(current_nodes, drain_target, desired_tier)
  if drain_target == 0          → Deny("drain would reduce node count to zero")
  if drain_target >= current_nodes → Deny("drain_target must be less than current_nodes")
  remaining = current_nodes - drain_target
  floor = DEDICATED_NODE_FLOOR if Dedicated else HOSTED_NODE_FLOOR
  if remaining < floor          → Deny("drain would drop <tier> cluster below node floor of <N>")
  else                          → Allow
```

Tier floors:

| Tier | Constant | Default value |
|------|----------|---------------|
| `Hosted` | `HOSTED_NODE_FLOOR` | 1 |
| `Dedicated` | `DEDICATED_NODE_FLOOR` | 3 |

The function is pure: no filesystem, network, or clock access.  Identical
inputs always produce the same `DrainAdmission` variant.  The function never
panics; all boundary cases return an explicit `Deny { reason }`.

## Validation Order (NodePoolOpRequest)

1. `tenant_id.trim().is_empty()` → `LifecycleValidationError::EmptyTenantId`
2. `cluster_name.trim().is_empty()` → `LifecycleValidationError::EmptyClusterName`
3. `target_node_count == 0` → `LifecycleValidationError::ZeroTargetNodeCount`
4. `target_node_count > NODE_COUNT_CEILING` → `LifecycleValidationError::TargetNodeCountExceedsFloor`

## Error Extension (additive)

Two new `LifecycleValidationError` variants are added without altering any
existing variant or its `Display` implementation:

| Variant | Display message |
|---------|-----------------|
| `ZeroTargetNodeCount` | `"target_node_count must be > 0"` |
| `TargetNodeCountExceedsFloor` | `"target_node_count exceeds maximum allowed"` |

## Testing Strategy

All tests live in `#[cfg(test)] mod tests` inside `src/lib.rs` (matching crate
style; no separate `tests/` directory for this pure-domain crate).

| Test | Coverage |
|------|----------|
| `nodepool_op_request_validates_happy_path` | `new()` succeeds with valid inputs for each `NodePoolAction` variant |
| `nodepool_op_request_rejects_empty_tenant_id` | `EmptyTenantId` on empty/whitespace tenant |
| `nodepool_op_request_rejects_empty_cluster_name` | `EmptyClusterName` on empty/whitespace cluster name |
| `nodepool_op_request_rejects_zero_target` | `ZeroTargetNodeCount` when `target_node_count == 0` |
| `nodepool_op_request_rejects_over_ceiling` | `TargetNodeCountExceedsFloor` when count > `NODE_COUNT_CEILING` |
| `nodepool_action_serde_roundtrip` | All four variants serialize to snake_case JSON and deserialize back |
| `drain_admission_denies_to_zero` | `drain_target == current_nodes` → Deny |
| `drain_admission_denies_below_dedicated_floor` | 4 nodes, drain 2 → remaining 2 < floor 3 → Deny |
| `drain_admission_denies_below_hosted_floor` | 2 nodes, drain 2 → remaining 0 < floor 1 → Deny (zero path) |
| `drain_admission_allows_safe_hosted_drain` | 5 nodes, drain 2 → remaining 3 ≥ floor 1 → Allow |
| `drain_admission_allows_safe_dedicated_drain` | 6 nodes, drain 2 → remaining 4 ≥ floor 3 → Allow |
| `drain_admission_deterministic` | Same call twice returns equal variant |

Pre-existing tests (`request_validates_identity_and_resources`,
`tier_parse_is_fail_closed`) must continue to pass unchanged.

## Boundary Constraints

- `crates/managed-k8s-cluster-lifecycle-kernel/` is the only directory
  modified in the codebase.
- Root `Cargo.toml` is not touched; no new workspace member is introduced.
- No new `[dependencies]` are added (serde workspace dep already present).
- No I/O, no clocks, no `unsafe`, no `unwrap`/`expect` outside `#[cfg(test)]`.
- All new public items follow the naming pattern established by the existing
  surface (`LifecycleRequest`, `DesiredTier`, `LifecycleValidationError`).
- No existing public type signatures or error variants are altered.
