# Spec: studio-dsl-emitter-cycle-reachability-validate

| Field | Value |
|---|---|
| Vertical | workflow |
| Crate | `workflow-studio-dsl-emitter-domain` |
| Stage | SPEC |
| Branch | `feat/task-studio-dsl-emitter-cycle-reachability-validate-2026-05-28` |
| ADR authority | ADR-0509 (flat crate per service), ADR-0131 (flat µservice layout) |

---

## Objective

Extend `WorkflowSpec::validate` in the pure DSL emitter domain crate with two
graph-integrity checks that run after the existing node/edge well-formedness
checks:

1. **Cycle detection** — directed graph built from `edges`; any cycle is a
   specification defect.
2. **Unreachable-node detection** — nodes that are structurally defined but
   never reachable from any entry node (in-degree 0) are specification defects.

Both faults surface as new `WorkflowSpecEmitError` variants. Canonical JSON
emission and all existing validations remain unchanged in shape and semantics.

---

## Vertical & crate boundaries

- **Only file changed:** `crates/workflow-studio-dsl-emitter-domain/src/lib.rs`
- Root `Cargo.toml` is not touched. No new workspace members. No new crates.
- No storage, signing, transport, HTTP, or gRPC concerns enter this crate.

---

## Mod layout (flat clean-arch, ADR-0509)

This crate is pure domain — a single `lib.rs` with no sub-modules. The
extension keeps that shape: new variants and helper functions live in
`lib.rs` alongside the existing code.

```
crates/workflow-studio-dsl-emitter-domain/
  src/
    lib.rs   ← only file modified
  Cargo.toml ← unchanged
```

---

## New error variants

```rust
pub enum WorkflowSpecEmitError {
    // ... existing variants unchanged ...
    GraphCycle(String),        // payload: offending node id (first by sorted order)
    UnreachableNode(String),   // payload: unreachable node id (first by sorted order)
}
```

The hand-written `PartialEq` impl gains two new match arms following the
existing `(DuplicateNodeId(l), DuplicateNodeId(r)) => l == r` pattern.

---

## Algorithm contracts

### Cycle detection — Kahn's BFS (topological sort)

1. Build an adjacency list and in-degree map over node IDs from `self.edges`.
2. Seed a `BTreeSet`-ordered queue with all zero-in-degree nodes (deterministic
   traversal order).
3. BFS: dequeue, decrement in-degrees of neighbours; re-enqueue any that reach
   zero. Count processed nodes.
4. If `processed < self.nodes.len()`, a cycle exists. Return
   `Err(GraphCycle(first_unprocessed_by_sorted_id))`.

Using `BTreeSet`/`BTreeMap` throughout guarantees determinism for identical
input regardless of `Vec` construction order.

### Unreachable-node detection — forward BFS from entry nodes

1. Entry nodes = nodes whose ID does not appear as a `to` endpoint in any edge
   (i.e., in-degree 0 in the directed graph).
2. Forward BFS from all entry nodes, accumulating visited IDs.
3. Any node ID not in the visited set → `Err(UnreachableNode(first_by_sorted_id))`.

**Note:** Cycle detection runs first. If a cycle is present, the unreachable
check is not reached (cycle subsumes the question of reachability for cyclic
subgraphs).

---

## Contracts

### No new HTTP/gRPC/AsyncAPI surface

This task adds no network interface. The crate remains a pure library with no
I/O. Existing `emit_canonical_json` is the only public output function and its
signature and output shape are unchanged.

### Serialisation stability

`WorkflowSpecEmitError` variants are not serialised. The new variants have no
effect on `serde` output for valid specs.

### Determinism guarantee

`validate()` is a pure function over `&self` (no global state, no randomness,
`BTreeMap`/`BTreeSet` for ordered traversal). Identical input always produces
identical output.

---

## Testing strategy

All tests live in the existing `#[cfg(test)] mod tests` block in `lib.rs`.
No separate test crate is introduced.

| Test | Scenario | Assertion |
|---|---|---|
| `validate_clean_dag_passes` | 3-node chain A→B→C | `Ok(())` |
| `validate_cyclic_graph_returns_graph_cycle` | A→B, B→A (2-cycle) | `Err(GraphCycle(_))` |
| `validate_unreachable_node_returns_unreachable_node` | A→B, C isolated | `Err(UnreachableNode("wfn_c"))` |
| `validate_is_deterministic` | `validate()` called twice | results are equal |

Existing tests (`emits_canonical_workflow_spec_v1_json_without_null_conditions`,
`rejects_duplicate_node_ids`, `rejects_dangling_edges`) must continue to pass
unchanged.

Test runner: `cargo nextest run -p workflow-studio-dsl-emitter-domain`

---

## Acceptance criteria

| # | Criterion |
|---|---|
| A1 | `cargo check -p workflow-studio-dsl-emitter-domain --all-targets` exits 0 |
| A2 | `cargo nextest run -p workflow-studio-dsl-emitter-domain` exits 0, all tests green |
| A3 | Cyclic spec → `Err(GraphCycle(..))` |
| A4 | Spec with unreachable node → `Err(UnreachableNode(..))` |
| A5 | All pre-existing valid-spec tests pass, `emit_canonical_json` output byte-identical |
| A6 | `validate()` is deterministic for identical input |
| A7 | No debug code (`dbg!`, `println!`, `eprintln!`, `todo!`) left in lib.rs |

---

## OpenSLO reference

Crate is a pure library with no runtime SLO. The owning µservice
(`workflow-studio-dsl-emitter-domain`) inherits the workflow vertical SLO
policy; no new SLO yaml is required for a pure validation extension.
