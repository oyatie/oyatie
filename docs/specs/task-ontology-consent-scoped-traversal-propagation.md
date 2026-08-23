# Spec: ontology-consent-scoped-traversal-propagation

doc_class: TaskSpec
task_id: ontology-consent-scoped-traversal-propagation
vertical: ontology
crate: ontology-query-engine-domain
status: spec
date: 2026-05-29
owner_team: axis-ontology

---

## 1. Objective

Extend `KnowledgeGraphQueryEngine::query_graph_slice` so that BFS edge propagation is
consent-gated: a caller supplies a set of consented `edge_type_id` values on the request, and
the traversal prunes any outbound link whose `edge_type_id` is absent from that set. Pruned
edges are not emitted and their downstream nodes are not reached via that path. When the grant
scope is empty the existing behaviour is preserved (no additional filter applied).

This is a pure in-memory, domain-layer change. No new crate, no storage adapter, no HTTP/gRPC
surface change.

---

## 2. Vertical and Crate

| Item | Value |
|------|-------|
| Vertical | ontology |
| Crate | `ontology-query-engine-domain` |
| Crate path | `crates/ontology-query-engine-domain/` |
| Source root | `crates/ontology-query-engine-domain/src/lib.rs` |
| Upstream IP | IP-011 (query-engine 3-layer KG) |

---

## 3. Contracts

### 3.1 OpenAPI 3.2.0 delta (future REST adapter, reference only)

The domain-layer request struct maps to the existing `POST /query-engine/graph-slice` operation
body. The consent grant scope is a new optional array field `consented_edge_type_ids` added to
the request schema (empty array = no consent filter). This spec reserves that field name for the
future REST contract amendment; no OpenAPI file is modified in this task.

```yaml
# Future addition to components/schemas/KnowledgeGraphQueryRequest in ontology.yaml:
consented_edge_type_ids:
  type: array
  items:
    type: string
    pattern: '^lty_[a-z][a-z0-9_]*$'
  description: >
    Consent grant scope. When non-empty, only edges whose edge_type_id appears in
    this set are traversed and emitted. When empty (default), no consent filter is
    applied and all previously-allowed edge types are traversable.
  default: []
```

### 3.2 proto3 delta (future gRPC adapter, reference only)

```protobuf
// Future addition to KnowledgeGraphQueryRequest in ontology.proto:
repeated string consented_edge_type_ids = 8;
// When non-empty: traversal prunes links whose edge_type_id is absent.
// When empty: prior semantics (no consent filter).
```

---

## 4. Mod layout (flat-clean-arch)

The crate is currently a single `src/lib.rs` containing all domain types, validation helpers,
the BFS engine, and tests. The flat-clean-arch pattern for this crate is:

```
src/
  lib.rs     — domain types + BFS engine + validation + tests (single flat file per ADR-0509)
```

No new files or modules are introduced. All changes land in `src/lib.rs`.

---

## 5. Change surface

### 5.1 ST1: `KnowledgeGraphQueryRequest` extension

**New field:**
```rust
pub consented_edge_type_ids: Vec<String>, // data_class: INTERNAL_ONLY
```

**New error variant:**
```rust
/// An entry in `consented_edge_type_ids` is not a valid edge-type ID (must start with `lty_`).
MalformedConsentGrantId { id: String },
```

**Validation rule:** Each entry in `consented_edge_type_ids` must pass `validate_edge_type_id`.
An invalid entry returns `Err(KnowledgeGraphQueryError::MalformedConsentGrantId { id })`.

**Backward compatibility:** The field is additive. Passing an empty `Vec` produces identical
behaviour to all pre-existing requests. All existing test call-sites receive `vec![]` for the
new parameter.

**New helper:**
```rust
fn consent_filter(&self) -> BTreeSet<&str> {
    self.consented_edge_type_ids.iter().map(String::as_str).collect()
}
```

### 5.2 ST2: BFS consent gate

Insertion point inside `query_graph_slice`, in the BFS link loop, after the freshness filter
and before the edge-cap check:

```rust
// Consent gate: prune edges not in the grant scope (when scope is non-empty).
if !consent_filter.is_empty() && !consent_filter.contains(link.edge_type_id.as_str()) {
    continue;
}
```

`consent_filter` is computed once before the BFS loop:
```rust
let consent_filter = request.consent_filter();
```

---

## 6. Testing strategy

### Existing tests (must stay green, call-site update only)

All existing tests in `src/lib.rs` receive `vec![]` for the new `consented_edge_type_ids`
parameter in `KnowledgeGraphQueryRequest::new(...)`. No test logic changes.

### New tests (ST2 acceptance)

**`consent_filter_prunes_non_consented_edges`**

Graph: `ent_root --lty_partner--> ent_b --lty_partner--> ent_c` and
       `ent_root --lty_member--> ent_d`.
Consent scope: `["lty_partner"]`.
Assert: `response.nodes` contains `ent_b` and `ent_c`; does not contain `ent_d`.
Assert: `response.edges` contains `(ent_root, lty_partner, ent_b)` and
        `(ent_b, lty_partner, ent_c)`; does not contain any edge with `edge_type_id = lty_member`.

**`empty_consent_scope_preserves_prior_behavior`**

Same graph. Consent scope: `[]`.
Assert: `response.nodes` contains all four nodes.
Assert: `response.edges` contains all three edges.

**`malformed_consent_grant_id_rejected`**

`KnowledgeGraphQueryRequest::new(…, consented_edge_type_ids: vec!["bad_id"], …)` returns
`Err(KnowledgeGraphQueryError::MalformedConsentGrantId { id: "bad_id".into() })`.

**`well_formed_consent_grant_id_accepted`**

`KnowledgeGraphQueryRequest::new(…, consented_edge_type_ids: vec!["lty_partner"], …)` returns
`Ok(…)`.

---

## 7. Disjointness and boundaries

| Boundary | Rule |
|----------|------|
| Depth/cardinality caps | Unchanged; consent gate fires before cap checks so pruned links never count toward caps. |
| Freshness filter | Unchanged; consent gate fires after freshness so stale-then-non-consented links are pruned for freshness first. |
| Edge-type filter (`edge_type_ids`) | Unchanged; the consent gate is a separate, additive field. Both filters can be set simultaneously; the edge-type filter fires first. |
| Tenant isolation | Unchanged; `outbound_links` is already tenant-scoped. |
| New crates | None. |
| Root `Cargo.toml` | Not touched. |
| Other crates | Not touched. |

---

## 8. Acceptance gates

| Gate | Command | Must pass |
|------|---------|-----------|
| Compile | `cargo check -p ontology-query-engine-domain --all-targets` | Yes |
| Tests | `cargo nextest run -p ontology-query-engine-domain` | Yes |
