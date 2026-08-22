# Spec: ontology-query-engine-domain — traversal-bounds

**Vertical:** ontology  
**Crate:** `ontology-query-engine-domain`  
**ADR surface:** in-memory domain; no external protocol changes in this slice  
**Status:** SPEC

---

## Objective

Harden `KnowledgeGraphQueryEngine::query_graph_slice` with three
bounded-traversal guarantees:

| # | Guarantee | Mechanism |
|---|-----------|-----------|
| 1 | Result-cardinality ceilings | `MAX_QUERY_RESULT_NODES` / `MAX_QUERY_RESULT_EDGES` constants; BFS halts deterministically when either cap is reached |
| 2 | Truncation reporting | `KnowledgeGraphQueryResponse.result_truncated: bool` — set `true` when any cap fires; never silently omit results |
| 3 | Depth-ceiling validation | `KnowledgeGraphQueryError::DepthCeilingExceeded` returned when `max_depth > MAX_QUERY_DEPTH`; distinct from `InvalidMaxDepth` (zero/negative) |

The implementation is adapter-free, in-memory, and tenant-scoped.  All
logic lives as mods inside `src/lib.rs`.

---

## Vertical / Crate Context

The crate implements the preview Knowledge Graph query contract.  It is
intentionally adapter-free: no cloud storage, no query language, no
distributed execution, no authz enforcement in this slice.  The
implemented semantics are bounded, tenant-scoped, deterministic outbound
BFS traversal over validated link instances, backed by an in-memory
`BTreeMap`.

---

## New Constants

```rust
/// Hard cap on nodes returned by a single `query_graph_slice` call.
/// When the BFS frontier would push the node set beyond this ceiling,
/// traversal halts deterministically and `result_truncated` is set.
pub const MAX_QUERY_RESULT_NODES: usize = 1_000;

/// Hard cap on edges returned by a single `query_graph_slice` call.
/// When the edge set would exceed this ceiling, traversal halts
/// deterministically and `result_truncated` is set.
pub const MAX_QUERY_RESULT_EDGES: usize = 5_000;
```

Both constants are declared alongside `MAX_QUERY_DEPTH: u32 = 16` and
must appear in crate-level rustdoc.

---

## Modified Types

### `KnowledgeGraphQueryResponse` (extended)

```rust
pub struct KnowledgeGraphQueryResponse {
    pub query_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub nodes: Vec<KnowledgeGraphNode>, // data_class: INTERNAL_ONLY
    pub edges: Vec<KnowledgeGraphEdge>, // data_class: INTERNAL_ONLY
    pub observed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    /// Set to `true` when BFS was halted by `MAX_QUERY_RESULT_NODES`
    /// or `MAX_QUERY_RESULT_EDGES` before the full reachable subgraph
    /// was explored.  Callers MUST treat this as a signal to paginate
    /// or narrow their query rather than assume completeness.
    pub result_truncated: bool,         // data_class: INTERNAL_ONLY
}
```

### `KnowledgeGraphQueryError` (extended)

```rust
pub enum KnowledgeGraphQueryError {
    InvalidTenantId,
    InvalidQueryId,
    InvalidEntityId,
    InvalidEdgeTypeId,
    /// `max_depth` was zero (structurally invalid; must be ≥ 1).
    InvalidMaxDepth,
    /// `max_depth` exceeds `MAX_QUERY_DEPTH` (domain ceiling).
    /// Distinct from `InvalidMaxDepth` so callers can distinguish a
    /// malformed request from an over-specified depth.
    DepthCeilingExceeded,
    MissingRootEntity,
    DanglingLinkEndpoint { entity_id: String },
}
```

---

## Traversal Algorithm (updated BFS contract)

```
query_graph_slice(graph, request):
  1. request.validate()          // rejects DepthCeilingExceeded, InvalidMaxDepth, etc.
  2. assert root entity exists   // MissingRootEntity
  3. BFS from root_entity_id:
     for each frontier node at depth d:
       if d >= max_depth: skip (depth-bounded)
       for each outbound link:
         apply edge-type filter
         apply freshness floor
         validate link endpoints
         if nodes.len() >= MAX_QUERY_RESULT_NODES: set truncated=true, break outer
         if edges.len() >= MAX_QUERY_RESULT_EDGES: set truncated=true, break outer
         insert edge + node; enqueue if unseen
  4. return KnowledgeGraphQueryResponse { ..., result_truncated }
```

Truncation is deterministic: the BFS processes frontier nodes in
`BTreeMap`-order (sorted by entity ID), so for a given graph and
request the same prefix of results is returned on every run.

---

## `validate_max_depth` contract (updated)

```
validate_max_depth(max_depth):
  if max_depth == 0:       Err(InvalidMaxDepth)
  if max_depth > MAX_QUERY_DEPTH: Err(DepthCeilingExceeded)
  else:                    Ok(())
```

The existing `(1..=MAX_QUERY_DEPTH).contains(&max_depth)` check is
split into two arms so the error kind is precise.

---

## Mod Layout (flat clean-arch)

Single-file crate; all logic in `src/lib.rs`:

| Concern | Location |
|---------|----------|
| Constants (`MAX_QUERY_DEPTH`, `MAX_QUERY_RESULT_NODES`, `MAX_QUERY_RESULT_EDGES`) | top of `lib.rs`, with rustdoc |
| Domain types (`KnowledgeGraphQueryRequest`, `KnowledgeGraphQueryResponse`, etc.) | `lib.rs` (no sub-mod split needed at this scale) |
| Error variants (`KnowledgeGraphQueryError`) | `lib.rs` |
| BFS traversal (`query_graph_slice`) | `KnowledgeGraphQueryEngine` impl block |
| Validation helpers (`validate_max_depth`, etc.) | module-private fns in `lib.rs` |
| Tests | `#[cfg(test)] mod tests` in `lib.rs` |

No new sub-modules are introduced; splitting is deferred until the file
exceeds reasonable single-screen readability.

---

## OpenAPI / proto3 / AsyncAPI Surface

This slice is adapter-free.  The types map to future REST/gRPC surfaces
as follows (informational; not normative in this slice):

### Proto3 sketch (informational)

```proto
// traversal_bounds additions
message KnowledgeGraphQueryResponse {
  // ... existing fields ...
  bool result_truncated = 6;
}

enum KnowledgeGraphQueryErrorCode {
  // ... existing ...
  DEPTH_CEILING_EXCEEDED = 8;
}
```

### OpenAPI 3.2.0 sketch (informational)

```yaml
components:
  schemas:
    KnowledgeGraphQueryResponse:
      properties:
        result_truncated:
          type: boolean
          description: >
            True when BFS was halted by MAX_QUERY_RESULT_NODES or
            MAX_QUERY_RESULT_EDGES before the full reachable subgraph
            was explored.
    KnowledgeGraphQueryErrorCode:
      enum:
        - DEPTH_CEILING_EXCEEDED
```

---

## Testing Strategy

All tests are `#[cfg(test)]` unit tests inside `src/lib.rs`.  No
integration-test crate is introduced (single-crate lane rule).

| Test | Assertion |
|------|-----------|
| `node_cap_triggers_truncation` | Build graph with > `MAX_QUERY_RESULT_NODES` entities; assert `result_truncated == true` on ≥ 2 repeated runs (determinism) |
| `edge_cap_triggers_truncation` | Build graph with > `MAX_QUERY_RESULT_EDGES` links; assert `result_truncated == true` |
| `under_cap_returns_full_results` | Graph with < caps; assert `result_truncated == false` and full node/edge sets |
| `depth_ceiling_exceeded` | `max_depth == MAX_QUERY_DEPTH + 1` → `Err(DepthCeilingExceeded)` |
| `depth_at_ceiling_accepted` | `max_depth == MAX_QUERY_DEPTH` → `Ok(...)` |
| `invalid_max_depth_zero` | `max_depth == 0` → `Err(InvalidMaxDepth)` |
| Existing tests | All remain green (tenant isolation, edge filter, freshness floor, cycle safety, upsert update) |

---

## Boundaries

- Operate ONLY in `crates/ontology-query-engine-domain/`.
- NEVER edit root `Cargo.toml` or any other crate.
- No new abstractions for single-use logic.
- `data_class` annotations preserved on every touched field.
- All logic is adapter-free and in-memory; no I/O, no async.
