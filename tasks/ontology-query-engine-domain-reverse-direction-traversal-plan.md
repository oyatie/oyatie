# Plan: ontology-query-engine-domain-reverse-direction-traversal

## Objective

Extend `KnowledgeGraphQueryEngine` BFS traversal from outbound-only to support three
traversal directions: `Outbound` (default, preserves current behaviour), `Inbound`, and
`Both`. All existing gates (edge-type filter, freshness floor, consent scope,
result-cardinality caps, depth ceiling, tenant isolation) must fire for every direction.
Emitted edges keep canonical `from->to` orientation regardless of direction.

## Edge Cases and Acceptance Criteria

| # | Scenario | Expected |
|---|----------|----------|
| A | `Inbound` reaches predecessor nodes that `Outbound` cannot reach | Predecessor visible only under `Inbound` |
| B | `Both` yields the union of `Outbound` and `Inbound` result sets | Node/edge union, no duplicates |
| C | Omitted/default `direction` reproduces today's outbound result byte-for-byte | Zero diff vs current behaviour |
| D | Consent scope prunes correctly under `Inbound` and `Both` | Non-consented edges dropped |
| E | Freshness floor prunes correctly under `Inbound` and `Both` | Stale edges dropped |
| F | Node/edge cardinality caps truncate and set `result_truncated` under `Inbound` | Caps honoured |
| G | Tenant isolation: inbound scan only returns same-tenant links | Cross-tenant invisible |
| H | Cycle graphs: inbound traversal does not loop indefinitely | `seen_nodes` prevents revisit |

## Architecture Notes

- `TraversalDirection` is a new `enum` added as a field on `KnowledgeGraphQueryRequest`
  with default `Outbound`.
- The engine already keeps a sorted `BTreeMap<KnowledgeGraphLinkKey, _>` keyed by
  `(tenant_id, from_entity_id, edge_type_id, to_entity_id)`.  An inbound adjacency
  view is built lazily as a secondary index on `to_entity_id` — since `BTreeMap`
  range-scans by key prefix, a separate `inbound_links` method builds a small
  on-the-fly map (or iterates the full tenant slice and filters by `to_entity_id`).
  Given the in-memory pure-domain constraint, a linear scan over the tenant partition
  is acceptable; a sorted secondary index would require a second BTreeMap keyed by
  `(tenant_id, to_entity_id, ...)` — adding one is the correct approach for
  deterministic performance.
- `KnowledgeGraphQueryRequest::new` gains a `direction` parameter.  To avoid breaking
  the existing call sites (internal tests), the `new` constructor keeps the same
  signature and a `with_direction` builder or an extra parameter is added at the end.
  The simplest non-breaking approach: add `direction` as the last parameter to `new`,
  defaulting test call sites that use the helper function to `Outbound`.

## Subtasks (ordered)

1. **Add `TraversalDirection` enum** — `Outbound | Inbound | Both` with `Default = Outbound`.
2. **Add `direction` field to `KnowledgeGraphQueryRequest`** — update `new()` constructor
   (add `direction: TraversalDirection` as last parameter); update `validate()` (no new
   rules needed); update all existing test call sites to pass `TraversalDirection::Outbound`.
3. **Add secondary inbound index** — second `BTreeMap<KnowledgeGraphLinkInboundKey, …>`
   in `KnowledgeGraphQueryEngine`, updated by `upsert_link`.  `inbound_links()` mirrors
   `outbound_links()`.
4. **Extend BFS loop** — select neighbor iterator(s) based on `request.direction`;
   when `Both`, run both iterators and deduplicate via `seen_nodes`.  Emit edges in
   canonical `from->to` orientation in all cases.
5. **Write RED tests** — new test functions covering acceptance criteria A–H.
6. **Verify GREEN** — `cargo nextest run -p oya-ontology-query-engine-domain`.
7. **Self-review and simplify**.
