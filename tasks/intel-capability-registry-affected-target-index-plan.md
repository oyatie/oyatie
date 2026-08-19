# Plan: intel-capability-registry-affected-target-index

## Objective

Extend `intelligence-capability-registry-app` with a pure deterministic
`affected_set` use-case: given a set of changed capability IDs, return the
transitive set of impacted capability IDs via declared `owner_capability_id`
dependency edges on `Capability`.

## Dependency model

`Capability.owner_capability_id` is the existing dependency edge:
child → parent (child depends on parent).

Reverse index (built on demand, O(n)): parent → [dependents].

When a capability changes, all direct and transitive dependents are affected.

## Edge cases

1. Capability in changed set not in registry — included in output as-is (it
   changed, so it is affected regardless of whether the registry knows it).
2. Cycles via ownership are structurally impossible (validation rejects self-
   ownership; no other ownership cycle enforcement exists — the BFS visited-set
   guards against infinite loops anyway).
3. Changed set is empty — returns empty `BTreeSet` immediately.
4. Changed set contains a leaf (no dependents) — returns just that ID.
5. Diamond: A owns B; C owns B; B changes → A and C both affected (BFS fans
   out correctly through the reverse index).
6. Multiple roots in changed set — BFS starts from all simultaneously.

## Acceptance criteria

- `affected_set({})` → `{}`
- `affected_set({A})` where A has no dependents → `{A}`
- `affected_set({parent})` → `{parent, child}` (direct edge)
- `affected_set({grandparent})` → `{grandparent, child, grandchild}` (transitive)
- `affected_set({child})` → `{child}` (changing a leaf doesn't pull in parent)
- Diamond topology returns all diamonds
- Unknown IDs pass through unchanged (no error)
- Output type: `BTreeSet<CapabilityId>` (deterministic, sorted)

## Subtasks

1. [x] Write plan (this file)
2. [x] Write spec (docs/specs/task-intel-capability-registry-affected-target-index.md)
3. [x] Write failing tests in intelligence-capability-registry-app
4. [x] Implement `CapabilityRegistry::affected_set` (minimum code to pass)
5. [x] Verify green: cargo check + cargo nextest run
6. [x] Self-review (correctness / architecture / security / performance / cloud-native)
7. [x] Simplify / cleanup
8. [x] Final green nextest + disjointness gate + push + PR
