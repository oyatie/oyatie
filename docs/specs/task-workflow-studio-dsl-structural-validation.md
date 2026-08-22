# Spec: workflow-studio-dsl-structural-validation

| Field | Value |
|---|---|
| Vertical | workflow |
| Crate | `workflow-studio-dsl-emitter-domain` |
| Stage | SPEC |
| Branch | `feat/task-workflow-studio-dsl-structural-validation-2026-05-28` |
| ADR authority | ADR-0509 (flat crate per service), ADR-0131 (flat µservice layout) |

---

## Objective

Extend `WorkflowSpec::validate` in the pure DSL emitter domain crate with two
new structural validation dimensions that are distinct from the existing
GraphCycle / DanglingEdge / DuplicateNode checks:

1. **Unreachable-node detection** (WF-STU-1) — a node not reachable from any
   entry/root node (in-degree 0) is a specification defect, surfaced as
   `UnreachableNode(String)`. This variant and its BFS reachability pass were
   introduced by the prior `studio-dsl-emitter-cycle-reachability-validate`
   task and are already present in the crate. WF-STU-1 confirms the existing
   implementation satisfies the subtask contract without further changes.

2. **Edge-condition determinism** (WF-STU-2) — two new variants detect
   non-deterministic branching:
   - `DuplicateEdgeCondition(String)`: two outgoing edges from the same source
     node carry an identical non-None condition string.
   - `AmbiguousDefaultEdge(String)`: more than one unconditional (condition =
     None) outgoing edge from the same source node.

3. **Emit contract** (WF-STU-3) — `emit_canonical_json` propagates all new
   validation errors, and canonical JSON output for previously-valid specs is
   byte-identical after this slice.

---

## Vertical & crate boundaries

- **Files changed:**
  - `crates/workflow-studio-dsl-emitter-domain/src/lib.rs` — new variants,
    PartialEq/Display arms, and edge-condition validation pass.
  - `crates/workflow-studio-dsl-emitter-domain/tests/graph_integrity.rs` —
    new integration tests for WF-STU-2 and WF-STU-3.
- Root `Cargo.toml` is not touched. No new workspace members. No new crates.
- No storage, signing, transport, HTTP, or gRPC concerns enter this crate.

---

## Mod layout (flat clean-arch, ADR-0509)

This crate is pure domain — a single `lib.rs` with no sub-modules. All
additions remain in `lib.rs` alongside the existing code. Integration tests
extend the existing `tests/graph_integrity.rs` file.

```
crates/workflow-studio-dsl-emitter-domain/
  src/
    lib.rs              ← new variants + edge-condition validation pass
  tests/
    graph_integrity.rs  ← new integration tests (WF-STU-2, WF-STU-3)
  Cargo.toml            ← unchanged
```

---

## New error variants

```rust
pub enum WorkflowSpecEmitError {
    // ... existing variants unchanged ...
    UnreachableNode(String),          // already present — WF-STU-1
    DuplicateEdgeCondition(String),   // payload: source node id — WF-STU-2
    AmbiguousDefaultEdge(String),     // payload: source node id — WF-STU-2
}
```

The hand-written `PartialEq` impl gains match arms for `DuplicateEdgeCondition`
and `AmbiguousDefaultEdge` following the existing `(UnreachableNode(l),
UnreachableNode(r)) => l == r` pattern.

The `Display` impl gains match arms for both new variants.

---

## Algorithm contracts

### Unreachable-node detection (already implemented)

Forward BFS from all entry nodes (in-degree 0). Any node ID absent from the
visited set is reported as `UnreachableNode(first_by_sorted_id)`. Runs before
cycle detection so nodes only reachable via a cycle are reported as
`UnreachableNode` rather than `GraphCycle`.

### Edge-condition determinism (WF-STU-2)

Runs after the existing reachability + cycle checks. For each source node,
accumulate:
- A `BTreeMap<&str, usize>` counting occurrences of each distinct condition
  string among outgoing edges.
- A `usize` counting unconditional (condition = None) outgoing edges.

Iteration is over source node IDs in sorted (`BTreeMap`) order to guarantee
deterministic error reporting.

Rules (checked in this order per node):
1. Any condition string with count ≥ 2 → `Err(DuplicateEdgeCondition(node_id))`.
2. Unconditional count ≥ 2 → `Err(AmbiguousDefaultEdge(node_id))`.

A single conditional edge + a single default (unconditional) edge from the same
node is explicitly valid and must not be rejected.

---

## Contracts

### No new HTTP/gRPC/AsyncAPI surface

This task adds no network interface. The crate remains a pure library with no
I/O. `emit_canonical_json` is the only public output function; its signature and
output shape are unchanged.

### Serialisation stability

`WorkflowSpecEmitError` variants are not serialised. The new variants have no
effect on `serde` output for valid specs. Canonical JSON of a valid spec is
byte-identical before and after this slice (verified by a pinned regression
test).

### Determinism guarantee

`validate()` is a pure function over `&self` (no global state, no randomness,
`BTreeMap`/`BTreeSet` throughout for ordered traversal). Identical input always
produces identical output.

---

## Testing strategy

### Unit tests (`src/lib.rs` — `#[cfg(test)] mod tests`)

Existing tests (`emits_canonical_workflow_spec_v1_json_without_null_conditions`,
`rejects_duplicate_node_ids`, `rejects_dangling_edges`, `validate_clean_dag_passes`,
`validate_cyclic_graph_returns_graph_cycle`,
`validate_unreachable_node_returns_unreachable_node`,
`validate_is_deterministic`) must continue to pass unchanged.

New unit tests added for WF-STU-2:

| Test name | Scenario | Assertion |
|---|---|---|
| `validate_duplicate_edge_condition_returns_error` | two outgoing edges from `wfn_branch` with condition `"ok"` | `Err(DuplicateEdgeCondition("wfn_branch"))` |
| `validate_ambiguous_default_edge_returns_error` | two unconditional outgoing edges from `wfn_split` | `Err(AmbiguousDefaultEdge("wfn_split"))` |
| `validate_single_conditional_and_single_default_passes` | one conditional + one unconditional edge from same source | `Ok(())` |

### Integration tests (`tests/graph_integrity.rs`)

Existing integration tests must remain green.

New integration tests added for WF-STU-2 and WF-STU-3:

| Test name | Scenario | Assertion |
|---|---|---|
| `emit_rejects_duplicate_edge_condition_spec` | spec with duplicate condition siblings | `Err(DuplicateEdgeCondition(_))` from `emit_canonical_json` |
| `emit_rejects_ambiguous_default_edge_spec` | spec with two unconditional siblings | `Err(AmbiguousDefaultEdge(_))` from `emit_canonical_json` |
| `canonical_json_of_valid_spec_is_stable` | pin byte-exact JSON of the linear helper spec | byte-identical to hardcoded expected string |

Test runner: `cargo nextest run -p workflow-studio-dsl-emitter-domain`

---

## Acceptance criteria

| # | Criterion |
|---|---|
| A1 | `cargo check -p workflow-studio-dsl-emitter-domain --all-targets` exits 0 |
| A2 | `cargo nextest run -p workflow-studio-dsl-emitter-domain` exits 0, all tests green |
| A3 | Disconnected node → `Err(UnreachableNode(node_id))` ordered by sorted id |
| A4 | Duplicate-condition siblings → `Err(DuplicateEdgeCondition(node_id))` |
| A5 | Two unconditional siblings → `Err(AmbiguousDefaultEdge(node_id))` |
| A6 | Single conditional + single default sibling set → `Ok(())` |
| A7 | Invalid spec (any new check) rejected by `emit_canonical_json` |
| A8 | Canonical JSON of valid spec byte-identical (pinned regression test green) |
| A9 | No new non-allowlisted deps; crate remains source-level only |
| A10 | No debug code (`dbg!`, `println!`, `eprintln!`, `todo!`) in modified files |

---

## OpenSLO reference

Crate is a pure library with no runtime SLO. The owning µservice
(`microservices/workflow-studio`) inherits the workflow vertical SLO policy; no
new SLO yaml is required for a pure validation extension.
