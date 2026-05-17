---
doc_class: ImplementationPlan
ip_id: IP-011
title: query-engine (3-layer Knowledge Graph: semantic / kinetic / dynamic)
microservice: ontology
phase: P01-typed-entity-substrate
status: pending
owner_team: axis-ontology
date: 2026-05-17
depends_on: [IP-009]
acceptance_lanes:
  - cargo-check
  - cargo-clippy
  - cargo-nextest
  - oya-foundry-fitness-perf-budget
  - oya-foundry-fitness-ontology-dynamic-freshness
related_artifacts:
  - microservices/ontology/src/crates/oya-ontology-query-engine-{kernel,domain,usecase,adapter,adapter-clickhouse,worker}/
doc_status: published
---

# IP-011: query-engine (3-layer Knowledge Graph)

## Intent

Author the 3-layer Knowledge Graph query engine over the Ontology per `/specs/knowledge-graph-schema.json`:
- **Semantic layer**: typed schema graph (Object Types ↔ Link Types ↔ Action Types ↔ Function Types).
- **Kinetic layer**: event-sourced mutation history (Kafka outbox + audit-chain).
- **Dynamic layer**: live-state telemetry (OpenTelemetry signal correlated with Object Type instances).

## Scope

In-scope:
- `oya-ontology-query-engine-{kernel,domain,usecase,adapter,adapter-clickhouse,worker}` crates.
- 3-layer KG schema readers (semantic from schema registry; kinetic from audit-chain; dynamic from OpenTelemetry).
- Cross-layer joins (e.g., "every Patient Object Type that was discharged in last 30 days AND latency p99 of /discharge endpoint over the same window").
- Per-tenant scope at every layer.
- Freshness budget per `dynamic_layer_freshness_lag_seconds` ≤ 2 s p99.

## Implementation

| Step | Action |
|---|---|
| 1 | Scaffold 6 crates |
| 2 | Author 3-layer KG schema readers |
| 3 | Author cross-layer join planner |
| 4 | Wire ClickHouse adapter for OLAP analytics |
| 5 | Wire OpenTelemetry adapter for dynamic-layer reads |
| 6 | Freshness budget enforcement: refuse stale reads beyond budget |
| 7 | Tests: 3-layer join p99 ≤ 500 ms; freshness ≤ 2 s |

## Verification

- 3-layer KG join bench: p99 ≤ 500 ms at 100 QPS.
- Dynamic-layer freshness probe ≤ 2 s.
- LEAN lanes green.

## References

- `/specs/knowledge-graph-schema.json`.
- `registry/knowledge-graph-{semantic,kinetic,dynamic}.json`.
- ADR-0006; Bominal ADR-0106 + ADR-0107.
