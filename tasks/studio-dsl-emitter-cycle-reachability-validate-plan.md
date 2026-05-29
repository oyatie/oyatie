# Plan: studio-dsl-emitter-cycle-reachability-validate

Vertical: workflow  
Crate: `oya-workflow-studio-dsl-emitter-domain`  
Branch: `feat/task-studio-dsl-emitter-cycle-reachability-validate-2026-05-28`

## Objective

Extend `WorkflowSpec::validate` with graph-integrity checks beyond existing
node/edge well-formedness: cycle detection and unreachable-node detection from
entry nodes (nodes with in-degree 0). Two new `WorkflowSpecEmitError` variants
surface these faults. Pure domain only — no storage, signing, or transport.

---

## Subtasks

### [1] Add error variants

**File:** `crates/oya-workflow-studio-dsl-emitter-domain/src/lib.rs`

Add to `WorkflowSpecEmitError`:

```
GraphCycle(String)
UnreachableNode(String)
```

Extend the hand-written `PartialEq` impl with match arms for both new variants
(String payload equality, same pattern as `DuplicateNodeId`).

Acceptance:
- `cargo check -p oya-workflow-studio-dsl-emitter-domain --all-targets` passes.
- Manual `PartialEq` arms cover both new variants.
- No existing variant semantics change.

---

### [2] Extend validate() with graph checks

**File:** `crates/oya-workflow-studio-dsl-emitter-domain/src/lib.rs`

After the existing well-formedness checks pass, append two deterministic passes:

1. **Cycle detection** — iterative Kahn's algorithm (BFS topological sort over
   the directed graph built from edges). If the processed-count after BFS is
   less than the total node count, a cycle exists. Report the first node (by
   sorted ID) that was never dequeued as `GraphCycle(node_id)`.

2. **Unreachable-node detection** — collect entry nodes (in-degree 0). BFS/DFS
   forward reachability from all entry nodes. Any node not visited is
   `UnreachableNode(node_id)` (first by sorted ID).

`canonicalized()` / `emit_canonical_json` behavior for previously-valid specs
is unchanged.

Acceptance:
- Cyclic spec → `Err(GraphCycle(..))`.
- Spec with unreachable node → `Err(UnreachableNode(..))`.
- All pre-existing valid-spec tests pass; `emit_canonical_json` output is
  byte-identical for previously-valid specs.

---

### [3] Unit tests

**File:** `crates/oya-workflow-studio-dsl-emitter-domain/src/lib.rs` — existing
`#[cfg(test)] mod tests` block.

Add:

| Test name | Scenario | Expected |
|---|---|---|
| `validate_clean_dag_passes` | 3-node linear chain (no cycle, all reachable) | `Ok(())` |
| `validate_cyclic_graph_returns_graph_cycle` | Two nodes with edges A→B and B→A | `Err(GraphCycle(..))` |
| `validate_unreachable_node_returns_unreachable_node` | 3 nodes, C has no incoming edge from A or B subgraph | `Err(UnreachableNode("wfn_c"))` |
| `validate_is_deterministic` | Call `validate()` twice on same spec | identical results |

Acceptance:
- `cargo nextest run -p oya-workflow-studio-dsl-emitter-domain` green.
- Tests assert specific new error variants.
- Determinism test confirms identical `Result` for identical input.

---

## Acceptance summary

| # | Acceptance criterion |
|---|---|
| 1 | `cargo check -p oya-workflow-studio-dsl-emitter-domain --all-targets` clean |
| 2 | Cyclic spec → `Err(GraphCycle(..))` |
| 3 | Unreachable-node spec → `Err(UnreachableNode(..))` |
| 4 | All pre-existing valid-spec tests pass unchanged |
| 5 | `emit_canonical_json` output byte-identical for previously-valid specs |
| 6 | `cargo nextest run -p oya-workflow-studio-dsl-emitter-domain` green |
| 7 | `validate()` is deterministic for identical input |

---

## Boundaries

- Only `crates/oya-workflow-studio-dsl-emitter-domain/src/lib.rs` is modified.
- Root `Cargo.toml` is NOT touched.
- No new crates, no new workspace members.
- No storage, signing, transport, or HTTP layer concerns.
