# Plan: wf-studio-dsl-emitter-domain-node-typology-semantics

## Objective

Extend `WorkflowSpec::validate()` with pure semantic node-typology checks layered
**after** all existing structural / graph-integrity / edge-condition checks (preserving
current error ordering and byte-identical canonical JSON for already-valid specs).

## Requirements Analysis

### New error variants

Three new variants must be added to `WorkflowSpecEmitError`:

1. `BranchNodeRequiresConditionalEdges(node_id)` — a `Branch` node must have ≥ 2
   outgoing edges and at least one edge carrying a condition expression.
2. `JoinNodeRequiresMultipleInbound(node_id)` — a `Join` node must have ≥ 2
   inbound edges.
3. `MissingTerminalNode` — at least one node in the graph must have out-degree 0
   (a sink / terminal node); if every node has at least one outgoing edge this error
   fires.

### Ordering constraint

New checks run **strictly after** `AmbiguousDefaultEdge` (the last current check).
Existing error ordering: structural → node-id/name → edge basic → graph-integrity
(UnreachableNode → GraphCycle) → edge-condition (DuplicateEdgeCondition →
AmbiguousDefaultEdge) → **[NEW]** node-typology (BranchNodeRequiresConditionalEdges →
JoinNodeRequiresMultipleInbound → MissingTerminalNode).

### Determinism

- BTreeMap / sorted IDs throughout so the first offending node by lexicographic
  (sorted) id is always reported.
- The function is pure: no I/O, no side effects.

### Acceptance criteria

1. A Branch node with a single unconditional outgoing edge → `BranchNodeRequiresConditionalEdges(first sorted offender)`.
2. A Join node with one inbound edge → `JoinNodeRequiresMultipleInbound`.
3. Every node has an outgoing edge (no sink) → `MissingTerminalNode`.
4. A well-formed branch/join diamond DAG with a terminal node → passes and
   round-trips byte-identically through `canonicalized()` + `emit_canonical_json`.
5. New checks fire strictly **after** `UnreachableNode` / `GraphCycle` /
   `DuplicateEdgeCondition` / `AmbiguousDefaultEdge`.
6. All existing `graph_integrity.rs` and `structural_validation.rs` tests still pass.

### Edge cases

- A Branch node with 0 outgoing edges: out-degree check (≥2) fires before the
  conditional-edge check; same error variant covers it.
- A Branch node with ≥2 outgoing edges all unconditional: fires
  `BranchNodeRequiresConditionalEdges` (existing `AmbiguousDefaultEdge` also fires
  if exactly two unconditional — but `AmbiguousDefaultEdge` is checked **before**
  node-typology, so `AmbiguousDefaultEdge` fires first in that case).
- A Join node with 0 or 1 inbound edges: fires `JoinNodeRequiresMultipleInbound`.
- `MissingTerminalNode` is reported only if the graph is otherwise valid: if a
  Branch/Join violation fires first, `MissingTerminalNode` is not reached.
- A single-node spec (no edges) has out-degree 0 → it is its own terminal → passes.

### k8s / cloud-native implications

Pure domain logic; no network calls, no k8s API surface changes, no OpenAPI/proto
changes in this slice. The crate is already published with `publish = false`.

## Ordered Subtasks

1. **Write plan** (this file) — done.
2. **Write spec** (`docs/specs/task-wf-studio-dsl-emitter-domain-node-typology-semantics.md`).
3. **Write RED tests** — `tests/node_typology.rs` in the crate.
4. **Confirm RED** — `cargo check --all-targets` shows tests compile, `cargo nextest run` shows failures.
5. **Implement** — add three new `WorkflowSpecEmitError` variants + `Display` + `PartialEq` arms
   + the three check blocks in `validate()`.
6. **Confirm GREEN** — `cargo nextest run -p oya-workflow-studio-dsl-emitter-domain` all pass.
7. **Self-review** — correctness / architecture / security / performance / cloud-native.
8. **Simplify** — guard clauses, dead code, naming; re-run nextest after each.
9. **Commit** — conventional commit per phase; strict path allowlist.
10. **Ship** — push branch, open PR.
