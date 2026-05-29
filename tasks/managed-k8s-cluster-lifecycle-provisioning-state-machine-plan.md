# Plan: managed-k8s-cluster-lifecycle-provisioning-state-machine

## Objective
Add a pure, deterministic tenant-cluster provisioning lifecycle state machine to
`oya-managed-k8s-cluster-lifecycle-kernel`. The existing crate holds request/validate value
objects and `evaluate_drain_admission`; this slice adds the cluster-phase state machine layer.

## State Enum: ClusterLifecycleState
Variants (in lifecycle order):
- `Requested` — initial state; cluster creation has been accepted but work not yet started
- `Provisioning` — infrastructure is being allocated and configured
- `Ready` — cluster is healthy and serving tenant workloads
- `Updating` — in-place upgrade or config change in flight (sub-phase of serving)
- `Draining` — cluster is being drained ahead of deletion
- `Deleted` — terminal; cluster has been torn down
- `Failed` — fault-terminal; reachable from any non-terminal state

## Legal Transitions
```
Requested   -> Provisioning, Failed
Provisioning -> Ready, Failed
Ready       -> Updating, Draining, Failed
Updating    -> Ready, Failed
Draining    -> Deleted, Failed
Deleted     -> (none — terminal)
Failed      -> (none — terminal)
```

## API Surface (const-where-possible)
- `initial() -> Self` — returns `Requested`
- `is_terminal(&self) -> bool` — true for `Deleted` / `Failed`
- `is_serving(&self) -> bool` — true for `Ready` / `Updating`
- `as_str(&self) -> &'static str` — snake_case name
- `parse(s: &str) -> Option<Self>` — fail-closed; `None` on unknown
- `can_transition_to(&self, next: Self) -> bool` — pure predicate
- `transition(self, next: Self) -> Result<Self, IllegalClusterTransition>` — never panics

## Error Type
```rust
pub struct IllegalClusterTransition { pub from: ClusterLifecycleState, pub to: ClusterLifecycleState }
```
Implements `Display` + `std::error::Error`.

## Serde
`#[serde(rename_all = "snake_case")]` — all variants serialise as lowercase snake_case strings.

## Tier-awareness
`Dedicated` tier: the `Provisioning -> Ready` transition validates that the request satisfies
`DEDICATED_NODE_FLOOR` (this check lives at the call site; the state machine itself stays pure
with no resource fields — the slice spec calls out that the check is "tier-aware *where*
`DesiredTier::Dedicated` requires the same `DEDICATED_NODE_FLOOR`").

The state machine does NOT embed resource counts; tier-awareness is enforced by a separate
`validate_dedicated_readiness` const fn.

## Tasks
1. Write plan (this file)
2. Write spec doc
3. Implement `ClusterLifecycleState` + `IllegalClusterTransition` in `src/lib.rs`
4. Add hermetic unit tests (red → green)
5. `cargo check -p oya-managed-k8s-cluster-lifecycle-kernel --all-targets` — green
6. `cargo nextest run -p oya-managed-k8s-cluster-lifecycle-kernel` — green
7. Self-review + simplify
8. Commit + push + PR
