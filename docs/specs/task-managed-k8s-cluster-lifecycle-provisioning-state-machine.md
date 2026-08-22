# Spec: managed-k8s-cluster-lifecycle-provisioning-state-machine

**Crate**: `managed-k8s-cluster-lifecycle-kernel`
**ADR**: ADR-0376 managed-Kubernetes product surface
**Layer**: kernel (pure value objects; no I/O, no async)

---

## Purpose

Add a pure, deterministic cluster-level provisioning lifecycle state machine to the
`managed-k8s-cluster-lifecycle-kernel` crate. The existing crate holds request/validation
value objects (`LifecycleRequest`, `NodePoolOpRequest`, `evaluate_drain_admission`). This slice
adds the cluster-phase state machine, expressed as a `ClusterLifecycleState` enum with typed
transition enforcement.

---

## State Enum: `ClusterLifecycleState`

Variants, in lifecycle order:

| Variant | Meaning |
|---------|---------|
| `Requested` | Cluster creation accepted; no infrastructure allocated yet |
| `Provisioning` | Infrastructure is being allocated and configured |
| `Ready` | Cluster is healthy and serving tenant workloads |
| `Updating` | In-place upgrade or config change in flight (sub-phase of serving) |
| `Draining` | Cluster is being drained ahead of deletion |
| `Deleted` | Terminal-success: cluster has been torn down |
| `Failed` | Fault-terminal: unrecoverable failure; reachable from any non-terminal state |

---

## Legal Transition Graph

```
Requested    -> Provisioning | Failed
Provisioning -> Ready        | Failed
Ready        -> Updating     | Draining | Failed
Updating     -> Ready        | Failed
Draining     -> Deleted      | Failed
Deleted      -> (none — terminal)
Failed       -> (none — terminal)
```

State machine invariants:
- `Failed` is reachable from every non-terminal state (fault-terminal).
- `Deleted` and `Failed` are the only terminal states; no outgoing transition is permitted.
- No skipping of states (e.g. `Requested -> Ready` is illegal).
- `Updating -> Ready` is the only "back-edge" (rolling back to the serving state after an update
  completes or is rolled back).

---

## Public API Surface

All methods are `#[must_use]` where they return a value. The state machine is `const`-friendly.

```rust
impl ClusterLifecycleState {
    /// The initial state for a freshly-accepted cluster creation request.
    pub const fn initial() -> Self;               // returns Requested

    /// True for `Deleted` and `Failed` (no outgoing transitions).
    pub const fn is_terminal(&self) -> bool;

    /// True when the cluster is serving tenant workloads: `Ready` or `Updating`.
    pub const fn is_serving(&self) -> bool;

    /// Stable wire/log slug (snake_case).
    pub const fn as_str(&self) -> &'static str;

    /// Parse from wire slug. Returns `None` for unknown values (fail-closed; no panic).
    pub fn parse(value: &str) -> Option<Self>;

    /// Pure predicate: is `next` a legal successor of `self`?
    pub const fn can_transition_to(&self, next: Self) -> bool;

    /// Attempt the transition, returning the new state or a typed error.
    /// Never panics; callers fail closed on `Err(IllegalClusterTransition)`.
    pub fn transition(self, next: Self) -> Result<Self, IllegalClusterTransition>;
}
```

---

## Error Type: `IllegalClusterTransition`

```rust
pub struct IllegalClusterTransition {
    pub from: ClusterLifecycleState,
    pub to:   ClusterLifecycleState,
}
```

- Implements `Display` (format: `"illegal cluster lifecycle transition: {from} -> {to}"`).
- Implements `std::error::Error`.
- `Copy + Clone + Debug + Eq + PartialEq`.

---

## Tier-Awareness: `validate_dedicated_readiness`

The state machine itself carries no resource fields. Tier-specific floor enforcement is a
separate pure function:

```rust
/// Validate that a `Dedicated`-tier cluster satisfies the node-floor requirement before
/// it may enter `Ready`. Pure predicate; no I/O.
///
/// Returns `Err(LifecycleValidationError::ZeroTargetNodeCount)` when `node_count == 0`,
/// or `Err(LifecycleValidationError::TargetNodeCountExceedsFloor)` when
/// `node_count < DEDICATED_NODE_FLOOR` and tier is `Dedicated`.
/// Returns `Ok(())` for `Hosted` (no additional floor beyond the resource-request check).
pub fn validate_dedicated_readiness(
    node_count: u32,
    tier: DesiredTier,
) -> Result<(), LifecycleValidationError>;
```

Callers invoke this before calling `state.transition(ClusterLifecycleState::Ready)` for
`Dedicated`-tier clusters. The state machine does not call it internally.

---

## Serde

- `#[serde(rename_all = "snake_case")]` on the enum.
- `ClusterLifecycleState::Requested` serialises as `"requested"`.
- All variants use snake_case slugs matching `as_str()`.

---

## Hermetic Test Coverage

| Test ID | Description |
|---------|-------------|
| `cls-1` | `initial()` returns `Requested` |
| `cls-2` | `is_terminal()` true only for `Deleted` and `Failed` |
| `cls-3` | `is_serving()` true only for `Ready` and `Updating` |
| `cls-4` | `parse` roundtrips all variants through `as_str()` |
| `cls-5` | `parse` returns `None` for unknown slug (fail-closed) |
| `cls-6` | Happy-path: `Requested -> Provisioning -> Ready -> Draining -> Deleted` |
| `cls-7` | Update cycle: `Ready -> Updating -> Ready` |
| `cls-8` | `Failed` reachable from every non-terminal state |
| `cls-9` | Terminal states (`Deleted`, `Failed`) have no outgoing transitions |
| `cls-10` | `transition` returns `IllegalClusterTransition` for illegal moves |
| `cls-11` | `IllegalClusterTransition` carries correct `from`/`to` pair |
| `cls-12` | Serde snake_case roundtrip for every variant |
| `cls-13` | `validate_dedicated_readiness` allows `Dedicated` at floor |
| `cls-14` | `validate_dedicated_readiness` denies `Dedicated` below floor |
| `cls-15` | `validate_dedicated_readiness` always allows `Hosted` regardless of count |
| `cls-16` | Skip transition denied (e.g. `Requested -> Ready`) |
| `cls-17` | `IllegalClusterTransition` `Display` is non-empty and well-formed |

---

## Non-Goals

- No async, no I/O, no HTTP stack, no persistence.
- No embedding of resource counts inside `ClusterLifecycleState`.
- No modification to the existing `ControlPlaneStatus` state machine in
  `managed-k8s-control-plane-host-kernel`; these are distinct kernels.
- No new workspace members or root `Cargo.toml` changes.

---

## Acceptance Criteria

1. `cargo check -p managed-k8s-cluster-lifecycle-kernel --all-targets` exits 0.
2. `cargo nextest run -p managed-k8s-cluster-lifecycle-kernel` exits 0; all 17 new tests
   plus all existing tests pass.
3. All changes confined to `microservices/managed-k8s-cluster-lifecycle/crates/managed-k8s-cluster-lifecycle-kernel/`.
4. No `unsafe`, no `unwrap`/`expect`/`panic` outside `#[cfg(test)]`.
