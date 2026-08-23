# Spec: intel-archmap-reachability-cycles

## Summary
Extend `ArchitectureMap` with two pure deterministic graph query methods:
transitive reachability and DependsOn-scoped cycle detection.

## Crate
`intelligence-architecture-map-kernel`

## API

### `reachable_from`
```rust
pub fn reachable_from(&self, start: &NodeId) -> BTreeSet<NodeId>
```
Returns the set of all nodes transitively reachable from `start` via any
outgoing edge kind. `start` itself is **not** included in the result.
Returns an empty set if `start` has no outgoing edges or is unknown.
Handles self-edges and disconnected subgraphs without panicking.

### `depends_on_cycles`
```rust
pub fn depends_on_cycles(&self) -> Vec<Vec<NodeId>>
```
Finds all simple cycles in the `DependsOn`-only subgraph using iterative DFS.
Each cycle is represented as a `Vec<NodeId>` rotated to start at its
lexicographically smallest element, with no repeated node (the implicit
closing edge back to the first element is not included). Cycles are
deduplicated and the outer `Vec` is sorted for deterministic output.
Returns an empty `Vec` when the DependsOn subgraph is acyclic.
Ignores all non-`DependsOn` edges.

## Acceptance Criteria
- Both methods added with stable `BTree`-ordered output
- `reachable_from` handles disconnected nodes and self-references
- `depends_on_cycles` finds self-loops and multi-node `DependsOn` cycles
- `depends_on_cycles` returns empty for a pure DAG
- Non-`DependsOn` edges are ignored by `depends_on_cycles`
- Existing emit/walk/plane tests untouched
- 4+ new `#[cfg(test)]` cases

## Implementation Notes
- Use `BTreeMap<&NodeId, BTreeSet<&NodeId>>` for adjacency to guarantee
  deterministic traversal order.
- Cycle normalisation: rotate each cycle path so the minimum `NodeId` is
  first, then sort the outer Vec.
- No external dependencies; std only.
