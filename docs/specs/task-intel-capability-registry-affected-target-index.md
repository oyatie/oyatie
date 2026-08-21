# Spec: intel-capability-registry-affected-target-index

## Objective

Add `CapabilityRegistry::affected_set` — a pure deterministic use-case that,
given a set of changed capability IDs, computes the transitive set of impacted
capability IDs by following `owner_capability_id` dependency edges in reverse.

## Crate boundary

Touches **only** `intelligence/capability-registry-app`.
No new workspace member. No kernel or domain changes required.

## Flat-clean-arch mod layout (ADR-0509)

```
intelligence-capability-registry-app/src/
  lib.rs          ← existing: Register / List / Get + new affected_set method
```

All use-case logic lives in `lib.rs` as methods on `CapabilityRegistry`.
No sub-modules needed for a single method.

## Contract

```rust
pub fn affected_set(
    &self,
    changed: &BTreeSet<CapabilityId>,
) -> BTreeSet<CapabilityId>
```

- **Input**: set of capability IDs that have changed (may include IDs not
  present in the registry).
- **Output**: `BTreeSet<CapabilityId>` containing:
  - All IDs in `changed`.
  - All capabilities in the registry whose `owner_capability_id` (directly or
    transitively) resolves to any ID in `changed`.

## Dependency edge semantics

`Capability.owner_capability_id = Some(parent_id)` means: this capability
depends on `parent_id`. When `parent_id` changes, this capability is affected.

Reverse index: `parent_id → Vec<dependent_id>` (built in O(n) from the
registry at call time).

BFS traversal starting from `changed` seeds, following reverse edges.

## Algorithm

```
fn affected_set(&self, changed: &BTreeSet<CapabilityId>) -> BTreeSet<CapabilityId> {
    // 1. Build reverse index: for each entry with owner, map owner -> child
    // 2. BFS from all ids in changed
    // 3. For each visited id, push unvisited dependents onto the queue
    // 4. Return the visited set
}
```

Time complexity: O(n) index build + O(n + e) BFS where n = registry size,
e = edges.  All structures are `BTreeMap`/`BTreeSet` for determinism.

## Testing strategy

Unit tests in `#[cfg(test)]` block of `lib.rs`:

| Test name | Scenario |
|---|---|
| `affected_set_empty_changed` | empty input → empty output |
| `affected_set_no_dependents` | leaf capability → just itself |
| `affected_set_direct_dependent` | parent changed → parent + child |
| `affected_set_transitive` | grandparent → grandparent + child + grandchild |
| `affected_set_leaf_change_no_upstream` | changing child doesn't pull parent |
| `affected_set_diamond` | two caps own same parent → both affected |
| `affected_set_unknown_id_passes_through` | unknown ID in changed → returned as-is |
| `affected_set_multiple_roots` | two changed roots propagate independently |

## Observability / SLO

This is a pure in-memory computation with no I/O. No metrics or SLO hooks
are required for this use-case. The existing SLO file for the capability
registry service applies unchanged.

## OpenAPI / AsyncAPI / proto3 implications

None — this is an in-process use-case with no HTTP surface. If an HTTP
endpoint is added later, it will expose `POST /capabilities/affected-set`
per OpenAPI 3.2.0 conventions and the http-stack-policy.
