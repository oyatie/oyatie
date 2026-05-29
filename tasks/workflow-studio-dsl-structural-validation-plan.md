# Plan: workflow-studio-dsl-structural-validation

Vertical: workflow  
Crate: `oya-workflow-studio-dsl-emitter-domain`  
Branch: `feat/task-workflow-studio-dsl-structural-validation-2026-05-28`

## Objective

Extend `WorkflowSpec::validate` with two new structural validation dimensions
distinct from the existing GraphCycle/DanglingEdge/DuplicateNode/UnreachableNode
checks:

1. **Unreachable-node detection** (WF-STU-1) — any node not reachable from the
   spec's entry node(s) (in-degree 0 nodes) is a specification defect. Surfaced
   as `UnreachableNode(String)`. NOTE: `UnreachableNode` and its reachability
   BFS pass are already present in the crate from the prior
   `studio-dsl-emitter-cycle-reachability-validate` task. WF-STU-1 acceptance
   is therefore satisfied by confirming existing tests remain green and the
   existing implementation matches the subtask contract.

2. **Edge-condition determinism** (WF-STU-2) — two new error variants:
   - `DuplicateEdgeCondition(String)` — two outgoing edges from the same node
     share an identical non-None condition string.
   - `AmbiguousDefaultEdge(String)` — more than one unconditional (condition =
     None) outgoing edge from the same source node.

3. **Round-trip / emit contract** (WF-STU-3) — a spec that fails any new check
   is rejected at `emit_canonical_json`; canonical JSON of a valid spec is
   byte-identical before and after this slice.

---

## Subtasks

### [WF-STU-1] UnreachableNode variant + reachability pass

**Status:** Already implemented in `src/lib.rs` (variant + PartialEq + Display
arm + BFS pass all present). Acceptance is confirmed by running existing tests.

Acceptance:
- `cargo check -p oya-workflow-studio-dsl-emitter-domain --all-targets` passes.
- Tests prove a disconnected node yields `UnreachableNode` with the expected id.
- A fully-connected spec still validates.
- Existing `graph_integrity.rs` tests remain green.

---

### [WF-STU-2] Edge-condition determinism validation

**File:** `crates/oya-workflow-studio-dsl-emitter-domain/src/lib.rs`

Add to `WorkflowSpecEmitError`:

```rust
DuplicateEdgeCondition(String),  // payload: source node id
AmbiguousDefaultEdge(String),    // payload: source node id
```

Extend `PartialEq` impl with match arms for both variants (String payload
equality, same pattern as existing `DuplicateNodeId` / `UnreachableNode` arms).

Extend `Display` impl with match arms for both variants.

After the existing graph-integrity checks (UnreachableNode + GraphCycle) in
`validate()`, add a deterministic edge-condition pass:

1. Build a `BTreeMap<&str, (BTreeMap<&str, usize>, usize)>` keyed by source
   node id, tracking per-condition counts and unconditional-edge count.
2. Iterate edges in insertion order (determinism comes from checking in sorted
   source-node order so report is reproducible).
3. For each source node: if any condition string appears on ≥ 2 outgoing edges,
   return `Err(DuplicateEdgeCondition(node_id))` (first node id by sorted order).
4. For each source node: if unconditional-edge count ≥ 2, return
   `Err(AmbiguousDefaultEdge(node_id))` (first node id by sorted order).

Ordering: report `DuplicateEdgeCondition` before `AmbiguousDefaultEdge` (check
condition duplicates first in the loop).

Acceptance:
- `cargo nextest run -p oya-workflow-studio-dsl-emitter-domain` green.
- Test: duplicate-condition siblings → `Err(DuplicateEdgeCondition(node_id))`.
- Test: two unconditional siblings → `Err(AmbiguousDefaultEdge(node_id))`.
- Test: single conditional + single default sibling set → `Ok(())`.

---

### [WF-STU-3] Emit contract + round-trip regression

**File:** `crates/oya-workflow-studio-dsl-emitter-domain/tests/graph_integrity.rs`

Add tests:

| Test name | Scenario | Expected |
|---|---|---|
| `emit_rejects_duplicate_edge_condition_spec` | spec with duplicate condition siblings | `Err(DuplicateEdgeCondition(_))` from `emit_canonical_json` |
| `emit_rejects_ambiguous_default_edge_spec` | spec with two unconditional siblings | `Err(AmbiguousDefaultEdge(_))` from `emit_canonical_json` |
| `canonical_json_of_valid_spec_is_stable` | pin byte-exact JSON of the linear helper spec | byte-identical to hardcoded string |

Acceptance:
- `cargo nextest run -p oya-workflow-studio-dsl-emitter-domain` green.
- Invalid specs (per WF-STU-1/WF-STU-2) return the new error from
  `emit_canonical_json`.
- Regression test pins canonical JSON of a representative valid spec unchanged.
- No new non-allowlisted dependencies.
- Crate remains source-level only (no build.rs, no proc-macro).

---

## Acceptance summary

| # | Acceptance criterion |
|---|---|
| A1 | `cargo check -p oya-workflow-studio-dsl-emitter-domain --all-targets` exits 0 |
| A2 | `cargo nextest run -p oya-workflow-studio-dsl-emitter-domain` exits 0, all tests green |
| A3 | Disconnected node → `Err(UnreachableNode(node_id))` (ordered by id) |
| A4 | Duplicate-condition siblings → `Err(DuplicateEdgeCondition(node_id))` |
| A5 | Two unconditional siblings → `Err(AmbiguousDefaultEdge(node_id))` |
| A6 | Single conditional + single default sibling → `Ok(())` |
| A7 | Invalid spec rejected by `emit_canonical_json` |
| A8 | Canonical JSON of valid spec byte-identical (regression pinned) |
| A9 | No new non-allowlisted deps; crate source-level only |
| A10 | No debug code (`dbg!`, `println!`, `eprintln!`, `todo!`) in modified files |

---

## Boundaries

- Only `crates/oya-workflow-studio-dsl-emitter-domain/src/lib.rs` and
  `crates/oya-workflow-studio-dsl-emitter-domain/tests/graph_integrity.rs` are
  modified.
- Root `Cargo.toml` is NOT touched.
- No new crates, no new workspace members.
- No storage, signing, transport, HTTP, or gRPC concerns enter this crate.
