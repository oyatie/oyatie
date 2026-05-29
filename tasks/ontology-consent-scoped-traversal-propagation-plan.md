# Task Plan: ontology-consent-scoped-traversal-propagation

task_id: ontology-consent-scoped-traversal-propagation
vertical: ontology
crate: oya-ontology-query-engine-domain
branch: feat/task-ontology-consent-scoped-traversal-propagation-2026-05-28
base: origin/dev
stage: SPEC

---

## Objective

Add consent/grant-scoped edge propagation to `KnowledgeGraphQueryEngine::query_graph_slice`.
The BFS already enforces depth/cardinality caps and edge-type/freshness filters. This task
layers a consent grant scope on top: outbound links whose `edge_type_id` is absent from the
caller-supplied grant set are pruned at traversal time, preventing non-consented edges from
being traversed or returned. Pure in-memory; no new crate; no storage adapter.

---

## Subtasks

### ST1 — Extend request type with consent grant scope

**Goal:** Add `consented_edge_type_ids: Vec<String>` to `KnowledgeGraphQueryRequest`; add a
`MalformedConsentGrantId` error variant; preserve backward compatibility (empty vec = no
consent filter, prior behaviour preserved).

**Steps:**
1. Add `consented_edge_type_ids: Vec<String>` field to `KnowledgeGraphQueryRequest` struct.
2. Add `KnowledgeGraphQueryError::MalformedConsentGrantId { id: String }` variant.
3. Extend `KnowledgeGraphQueryRequest::validate()` to validate each entry in
   `consented_edge_type_ids` using the existing `validate_edge_type_id` helper, returning
   `MalformedConsentGrantId` on failure.
4. Extend `KnowledgeGraphQueryRequest::new()` parameter list with
   `consented_edge_type_ids: Vec<impl Into<String>>` (additive; existing callers broken if not
   updated — update all call-sites in `#[cfg(test)]` blocks accordingly).
5. Add a `consent_filter` helper method on the request (analogous to `edge_filter`) returning
   `BTreeSet<&str>`.

**Accept:**
- `cargo check -p oya-ontology-query-engine-domain --all-targets` passes.
- `KnowledgeGraphQueryRequest::new(…, vec!["bad_id"], …)` returns
  `Err(KnowledgeGraphQueryError::MalformedConsentGrantId { id: "bad_id".into() })`.
- `KnowledgeGraphQueryRequest::new(…, vec!["lty_partner"], …)` returns `Ok(…)`.
- All existing request-construction tests compile and pass (update call-sites with the new param).

---

### ST2 — Apply consent filter in BFS expansion

**Goal:** During `query_graph_slice` BFS, prune outbound links whose `edge_type_id` is absent
from the consent grant scope when the scope is non-empty. Non-consented edges are neither
emitted nor traversed; their downstream nodes are not reached via that path.

**Insertion point:** After the existing edge-type/freshness filters and before the edge/node
cap checks (i.e., inside the `for link in self.outbound_links(…)` loop, after the freshness
check, before `edges.len() >= MAX_QUERY_RESULT_EDGES`).

**Steps:**
1. Compute `let consent_filter = request.consent_filter();` before the BFS loop (same pattern as
   `edge_filter`).
2. In the BFS link loop, add: `if !consent_filter.is_empty() && !consent_filter.contains(link.edge_type_id.as_str()) { continue; }`.
3. Write test `consent_filter_prunes_non_consented_edges`: multi-hop graph with two edge types
   (`lty_partner` and `lty_member`); only `lty_partner` in grant scope; assert node C reached via
   `lty_partner` is present; assert node D reached only via `lty_member` is absent.
4. Write test `empty_consent_scope_preserves_prior_behavior`: same graph, empty
   `consented_edge_type_ids`; all reachable nodes returned (existing semantics).
5. Verify that all existing traversal-bounds/cap, freshness, and tenant-isolation tests remain green.

**Accept:**
- `cargo nextest run -p oya-ontology-query-engine-domain` green (zero failures).
- Multi-hop test: node reached via consented edge type is present; node reachable only via
  non-consented edge type is absent from `response.nodes` and `response.edges`.
- Empty-grant-scope test: full result set returned (no regression).
- No change to depth/cardinality cap logic or freshness filter logic.

---

## Acceptance Summary

| Gate | Command |
|------|---------|
| Compile + type check | `cargo check -p oya-ontology-query-engine-domain --all-targets` |
| Full test suite | `cargo nextest run -p oya-ontology-query-engine-domain` |

Both must be green before the BUILD stage PR is opened against `dev`.

---

## Boundaries

- Touch only: `crates/oya-ontology-query-engine-domain/src/lib.rs`
- Lane docs: `docs/specs/task-ontology-consent-scoped-traversal-propagation.md`, this file.
- NEVER: root `Cargo.toml`, any other crate, any microservice source file.
