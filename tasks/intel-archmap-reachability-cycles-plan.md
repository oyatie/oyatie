# Plan: intel-archmap-reachability-cycles

## Objective
Add two pure transitive graph query methods to `ArchitectureMap`:
1. `reachable_from(&NodeId) -> BTreeSet<NodeId>` — transitive closure over all outgoing edges
2. `depends_on_cycles() -> Vec<Vec<NodeId>>` — deterministic DFS cycle detection over `DependsOn` edges only

## Constraints
- Std-only, deterministic, no I/O
- All changes inside `oya-intelligence-architecture-map-kernel`
- `BTreeSet`/`BTreeMap` for stable ordering throughout
- Existing tests must remain untouched

## Steps
1. Add `reachable_from` to `ArchitectureMap` in `src/lib.rs`
   - BFS/DFS iterative traversal over all outgoing edge kinds
   - Exclude the start node from the result set
   - Handle disconnected nodes (return empty set) and self-edges (don't loop infinitely)
2. Add `depends_on_cycles` to `ArchitectureMap` in `src/lib.rs`
   - Filter edges to `EdgeKind::DependsOn` only
   - Iterative DFS with a path stack; canonical ordering via BTreeMap adjacency
   - Each cycle rotated so its lexicographically smallest NodeId is first; output sorted
   - Return `Vec<Vec<NodeId>>` — empty means the DependsOn subgraph is a DAG
3. Write 4+ `#[cfg(test)]` cases in `src/lib.rs` covering:
   - Reachability from disconnected node returns empty
   - Reachability over a chain (transitive)
   - Self-loop in reachability does not panic
   - `depends_on_cycles` returns empty for a pure DAG
   - `depends_on_cycles` finds a self-loop (`A -> A`)
   - `depends_on_cycles` finds a two-node cycle (`A -> B -> A`)
   - `depends_on_cycles` finds a multi-node cycle and ignores non-DependsOn edges
4. `cargo check -p oya-intelligence-architecture-map-kernel --all-targets` — green
5. `cargo nextest run -p oya-intelligence-architecture-map-kernel` — green
