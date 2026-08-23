# Spec: wf-studio-dsl-emitter-domain-node-typology-semantics

## Objective

Extend `WorkflowSpec::validate()` in `workflow-studio-dsl-emitter-domain` with
three new **pure, deterministic** node-typology semantic checks that fire strictly
after all existing structural, graph-integrity, and edge-condition checks.

## Contracts

- **No new crate dependencies.** All logic is pure Rust using `BTreeMap` / `BTreeSet`.
- **Zero external I/O.** Pure domain functions; no async, no network, no file access.
- **Byte-identical canonical JSON** for previously-valid specs is preserved.
- **Stable error ordering** (new checks strictly after `AmbiguousDefaultEdge`).

## Module / Mod Layout (flat-clean-arch per ADR-0509)

Single crate, single `src/lib.rs`. No new modules introduced; all additions are
in-place within `lib.rs`.

## New Error Variants

```
BranchNodeRequiresConditionalEdges(node_id: String)
JoinNodeRequiresMultipleInbound(node_id: String)
MissingTerminalNode
```

All three must implement `Display` (human-readable messages) and be added to the
existing hand-written `PartialEq` implementation.

## Check Semantics

### BranchNodeRequiresConditionalEdges

For every node with `kind == WorkflowSpecNodeKind::Branch`, iterate nodes in BTreeMap
(sorted) order:
- out-degree must be ≥ 2
- at least one outgoing edge must have `condition: Some(_)`

If either condition fails, return `Err(BranchNodeRequiresConditionalEdges(node_id))`.
Report the **first offending node by sorted id**.

### JoinNodeRequiresMultipleInbound

For every node with `kind == WorkflowSpecNodeKind::Join`, iterate nodes in BTreeMap
(sorted) order:
- in-degree must be ≥ 2

If this fails, return `Err(JoinNodeRequiresMultipleInbound(node_id))`.
Report the **first offending node by sorted id**.

### MissingTerminalNode

After all per-node typology checks pass, check that at least one node has out-degree 0.
A node with out-degree 0 is a sink / terminal. If no such node exists, return
`Err(MissingTerminalNode)`.

## Check Ordering (validate() control flow)

```
structural checks (schema_version, tenant_id, …, DuplicateNodeId)
edge-basic checks (EmptyEdgeEndpoint, DuplicateEdge, SelfLoop, DanglingEdge*)
graph-integrity (UnreachableNode → GraphCycle)
edge-condition (DuplicateEdgeCondition → AmbiguousDefaultEdge)
[NEW] node-typology (BranchNodeRequiresConditionalEdges → JoinNodeRequiresMultipleInbound → MissingTerminalNode)
```

## Testing Strategy

Integration test file: `crates/workflow-studio-dsl-emitter-domain/tests/node_typology.rs`

### Acceptance tests (TDD — RED then GREEN)

1. `branch_node_single_unconditional_edge_is_rejected` — Branch + 1 unconditional edge.
2. `branch_node_zero_outgoing_edges_is_rejected` — Branch + 0 outgoing edges.
3. `branch_node_multiple_conditional_edges_passes` — Branch + ≥2 distinct-condition edges.
4. `join_node_single_inbound_edge_is_rejected` — Join + 1 inbound edge.
5. `join_node_zero_inbound_edges_is_rejected` — Join + 0 inbound edges.
6. `join_node_multiple_inbound_edges_passes` — Join + ≥2 inbound edges.
7. `no_terminal_node_is_rejected` — every node has out-degree ≥1.
8. `single_node_no_edges_has_terminal_node_passes` — trivially valid.
9. `well_formed_branch_join_diamond_dag_passes_and_round_trips` — full diamond (acceptance 4).
10. `node_typology_fires_after_ambiguous_default_edge` — ordering guarantee (acceptance 5).
11. `branch_node_with_all_conditional_but_only_one_outgoing_is_rejected` — out-degree < 2.

## Observability / SLO

This crate is a pure domain library (no HTTP surface, no runtime). No new SLO file
required for this slice. The parent microservice (`microservices/workflow-studio/`)
owns runtime SLOs.

## Crate Boundary

- Only `crates/workflow-studio-dsl-emitter-domain/` is modified.
- No workspace-level `Cargo.toml` changes.
- No other crate is touched.
