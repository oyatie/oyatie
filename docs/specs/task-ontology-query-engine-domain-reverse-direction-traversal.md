# Spec: ontology-query-engine-domain-reverse-direction-traversal

## Objective

Extend `KnowledgeGraphQueryEngine` BFS traversal from outbound-only to bidirectional,
adding a `TraversalDirection { Outbound, Inbound, Both }` variant to
`KnowledgeGraphQueryRequest`. Default is `Outbound` to preserve all existing behaviour
byte-for-byte. Pure deterministic domain logic; zero I/O; no new crate dependencies.

## Crate boundary

Only `ontology-query-engine-domain` (`crates/ontology-query-engine-domain/`) is
modified. No workspace Cargo.toml changes.

## Mod layout (flat clean-arch per ADR-0509)

All code lives in `src/lib.rs` (single-file crate pattern already in use).

## Public API changes

### New type: `TraversalDirection`

```rust
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TraversalDirection {
    #[default]
    Outbound,
    Inbound,
    Both,
}
```

### Modified: `KnowledgeGraphQueryRequest`

New field `direction: TraversalDirection` (default `Outbound`).

`new()` gains `direction: TraversalDirection` as the last parameter.

### Modified: `KnowledgeGraphQueryEngine`

- Adds a secondary index `inbound: BTreeMap<KnowledgeGraphLinkInboundKey, KnowledgeGraphLinkInstance>`
  updated atomically by `upsert_link`.
- `inbound_links(tenant_id, to_entity_id) -> impl Iterator<…>` mirrors `outbound_links`.
- `query_graph_slice` BFS loop selects the correct neighbor iterator(s) from `direction`.
- All existing gates (edge-type filter, freshness floor, consent scope, node cap, edge
  cap, depth ceiling, tenant isolation) apply identically for every direction.
- Emitted edges always carry canonical `from_entity_id → to_entity_id` orientation.

### New secondary key type: `KnowledgeGraphLinkInboundKey`

```rust
struct KnowledgeGraphLinkInboundKey {
    tenant_id: String,
    to_entity_id: String,
    edge_type_id: String,
    from_entity_id: String,
}
```

## Contracts / cloud-native implications

No HTTP, gRPC, or async contracts touched (pure domain slice). No OpenAPI/proto changes
required. No SLO changes (observability substrate unchanged).

## Testing strategy

Hermetic unit tests in `#[cfg(test)]` inline mod (`src/lib.rs`).

### New tests (RED before GREEN)

| Test name | Criteria |
|-----------|----------|
| `inbound_reaches_predecessors_outbound_cannot` | Inbound returns predecessors; Outbound does not |
| `both_yields_union_of_outbound_and_inbound` | Both = union, no duplicates |
| `default_direction_reproduces_outbound_result` | Omitting explicit dir = Outbound byte-for-byte |
| `inbound_consent_prunes_non_consented_edges` | Consent gate fires under Inbound |
| `inbound_freshness_floor_prunes_stale_edges` | Freshness gate fires under Inbound |
| `inbound_node_cap_triggers_result_truncated` | Node cap honoured under Inbound |
| `inbound_tenant_isolation` | Cross-tenant links invisible under Inbound |
| `inbound_cycle_no_unbounded_revisit` | seen_nodes prevents infinite loop in cyclic inbound graph |

### Existing tests (must remain GREEN)

All existing tests in `src/lib.rs` pass without modification.

## Observability / SLO

No change. The crate is a pure domain library with no OTel instrumentation surface of
its own at this tier.
