# Plan: ontology-query-engine-domain-traversal-bounds

Lane: ontology  
Crate: `oya-ontology-query-engine-domain`  
Branch: `feat/task-ontology-query-engine-domain-traversal-bounds-2026-05-28`

## Goal

Harden `KnowledgeGraphQueryEngine::query_graph_slice` with three
bounded-traversal guarantees that eliminate silent blast-radius risk:

1. Explicit result-cardinality ceilings (node + edge caps).
2. Deterministic cycle-safe truncation reporting (typed signal, never
   silent omission).
3. Validated `max_depth` ceiling at the domain boundary with a precise
   typed error (`DepthCeilingExceeded`).

All changes stay inside `crates/oya-ontology-query-engine-domain/src/`
as mods.  No new crate.  No root `Cargo.toml` edit.

---

## Subtasks

### ST1 — Result-cardinality ceilings + truncation reporting

**What:**

- Add `MAX_QUERY_RESULT_NODES: usize` and `MAX_QUERY_RESULT_EDGES: usize`
  constants alongside `MAX_QUERY_DEPTH`.
- In `query_graph_slice`, stop BFS deterministically once either cap is
  reached.
- Extend `KnowledgeGraphQueryResponse` with `result_truncated: bool`
  (set `true` when traversal was halted by a cap).
- Do NOT silently drop results; always surface the truncation flag.

**Acceptance:**

- `cargo check -p oya-ontology-query-engine-domain --all-targets` clean.
- `cargo nextest run -p oya-ontology-query-engine-domain` passes.
- Unit test: build a graph exceeding the node cap; assert
  `result_truncated == true` on every repeated run (determinism).
- Unit test: under-cap query returns `result_truncated == false` and
  full results.

---

### ST2 — `max_depth` ceiling validation (`DepthCeilingExceeded`)

**What:**

- Add `KnowledgeGraphQueryError::DepthCeilingExceeded` variant.
- In `KnowledgeGraphQueryRequest::validate()` (called by both `new` and
  `query_graph_slice`), reject `max_depth > MAX_QUERY_DEPTH` with
  `DepthCeilingExceeded` (distinct from the existing `InvalidMaxDepth`
  which handles `max_depth == 0`).
- Preserve all existing validation: `InvalidTenantId`, `InvalidQueryId`,
  `InvalidEntityId`, `InvalidEdgeTypeId`, `InvalidMaxDepth` (zero),
  `MissingRootEntity`, freshness-floor semantics.

**Note on existing behaviour:** `validate_max_depth` currently rejects
`max_depth > MAX_QUERY_DEPTH` with `InvalidMaxDepth`.  ST2 replaces that
arm with `DepthCeilingExceeded` so callers can distinguish "structurally
invalid depth value" from "depth exceeds the domain ceiling".  The
existing test (`MAX_QUERY_DEPTH + 1 → InvalidMaxDepth`) will be updated
to expect `DepthCeilingExceeded`.

**Acceptance:**

- `cargo nextest run -p oya-ontology-query-engine-domain` passes.
- Test (a): `max_depth == MAX_QUERY_DEPTH` → `Ok`.
- Test (b): `max_depth > MAX_QUERY_DEPTH` → `Err(DepthCeilingExceeded)`.
- Test (c): existing `MissingRootEntity` / edge-filter / freshness-floor
  tests remain green.

---

### ST3 — Rustdoc + lane docs

**What:**

- Add crate-level rustdoc block documenting `MAX_QUERY_RESULT_NODES`,
  `MAX_QUERY_RESULT_EDGES`, `MAX_QUERY_DEPTH`, `DepthCeilingExceeded`,
  and `result_truncated` semantics.
- Ensure `data_class` annotations are intact on all touched fields.
- This file (`tasks/ontology-query-engine-domain-traversal-bounds-plan.md`)
  and `docs/specs/task-ontology-query-engine-domain-traversal-bounds.md`
  are the lane-namespaced docs (written at SPEC stage).

**Acceptance:**

- `cargo check -p oya-ontology-query-engine-domain --all-targets` clean,
  no new clippy denials under workspace lints.
- `cargo doc -p oya-ontology-query-engine-domain --no-deps` compiles with
  no broken intra-doc links.
- Both lane docs exist and cover each new constant, error variant, and
  truncation-signal semantics.

---

## Implementation order

```
ST1 (cardinality caps + truncation flag)
  → ST2 (DepthCeilingExceeded)
  → ST3 (rustdoc, already covered by this SPEC commit)
```

Each subtask: implement → `cargo check` → `cargo nextest` → mark done.

---

## Boundaries

- ONLY crate `oya-ontology-query-engine-domain`.
- NO edits to root `Cargo.toml` or any other crate.
- NO new abstractions for single-use logic.
- All logic stays as `mod`s inside `src/lib.rs` (single-file crate at
  this stage; split only if the file exceeds reasonable size).
