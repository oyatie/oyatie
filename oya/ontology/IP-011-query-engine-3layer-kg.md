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
  - oya-governance-perf-budget
  - oya-governance-ontology-dynamic-freshness
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


## A. Problem
`IP-011: query-engine (3-layer Knowledge Graph)` is not a generic implementation packet; it closes the `011 query engine 3layer kg` gap for `ontology` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Object Type, Link Type, Action Type, Function Type, tenant-scoped entity store, Cedar fragment, read-path library, Merkle audit chain.

## B. Approach
A split OLTP/OLAP read path using Postgres/Citus for fresh typed rows and ClickHouse for history projections, selected by freshness floor and query shape. The implementation must keep the µservice boundary intact: contracts remain under `microservices/ontology/contracts/openapi/ontology.yaml` / `microservices/ontology/contracts/proto/ontology.proto`, policy decisions remain in `microservices/ontology/policy/tenant-scope.cedar`, operational proof remains in `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`, and the parity claim is checked against `microservices/ontology/competitor-parity-matrix.md`.

## C. Deliverables
- `microservices/ontology/PRD.md` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/ARCHITECTURE.md` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/contracts/openapi/ontology.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/contracts/proto/ontology.proto` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/contracts/asyncapi/ontology-events.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/policy/tenant-scope.cedar` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/slos/read-path-library-freshness.openslo.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/runbooks/type-registry-migration.md` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/catalog/oya-ontology-object-type-registry-kernel.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/competitor-parity-matrix.md` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/catalog/oya-ontology-query-engine-adapter-clickhouse.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/slos/function-read-latency.openslo.yaml` — verify/update as the authoritative artifact for this IP.
- Named code targets declared by this IP and `manifest.json` must be created only when the implementation PR actually adds the crates/types; this scrub does not pretend source files exist.

## D. Implementation Steps
1. Read `microservices/ontology/PRD.md` and `microservices/ontology/ARCHITECTURE.md` to confirm the bounded context, tenant class, and first-ship milestone for `ontology`.
2. Diff the declared contract in `microservices/ontology/contracts/openapi/ontology.yaml` and `microservices/ontology/contracts/proto/ontology.proto` against the IP title so every endpoint/message has a matching domain type or explicit backlog gap.
3. Check `microservices/ontology/policy/tenant-scope.cedar` plus adjacent Cedar/policy files before adding any mutation, share, webhook, agent, AI, or cross-tenant path.
4. Wire observability to `microservices/ontology/slos/read-path-library-freshness.openslo.yaml` and the relevant dashboard/runbook; no acceptance claim counts without a metric or sealed evidence path.
5. Update the catalog/capability record such as `microservices/ontology/catalog/oya-ontology-object-type-registry-kernel.yaml` so the service registry can discover the new boundary.
6. Run the IP-specific test/gate commands listed above; if a source crate is absent, record the absent crate as implementation debt rather than faking a green result.

## E. Acceptance
- Local artifact links resolve for `microservices/ontology/PRD.md`, `microservices/ontology/ARCHITECTURE.md`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/policy/tenant-scope.cedar`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`, and `microservices/ontology/competitor-parity-matrix.md`.
- The implementation exposes no cross-tenant, cross-pack, credential, E2E, or vendor-call path without the policy file cited in this IP.
- At least one targeted unit/contract/gate command verifies the named behavior, and any skipped command is documented with the missing artifact.
- The final PR includes evidence that counterpart parity is improved or explicitly marks the remaining gap.

## F. Evidence
- `microservices/ontology/PRD.md`
- `microservices/ontology/ARCHITECTURE.md`
- `microservices/ontology/contracts/openapi/ontology.yaml`
- `microservices/ontology/contracts/proto/ontology.proto`
- `microservices/ontology/contracts/asyncapi/ontology-events.yaml`
- `microservices/ontology/policy/tenant-scope.cedar`
- `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`
- `microservices/ontology/runbooks/type-registry-migration.md`
- `microservices/ontology/catalog/oya-ontology-object-type-registry-kernel.yaml`
- `microservices/ontology/competitor-parity-matrix.md`
- `microservices/ontology/competitor-parity-matrix.md` — counterpart gap table used for the comparison below.

## G. Counterparts
| Counterpart pressure | Oyatie closure for this IP |
|---|---|
| Palantir Foundry Ontology / Palantir AIP, AWS Cedar, Neo4j, AWS Neptune, Apache TinkerPop, Stardog, and Salesforce object model | Palantir Foundry Ontology supplies the product bar for object/link/action/function types; AWS Cedar supplies the policy bar; Neo4j/AWS Neptune/Stardog supply graph traversal and virtual graph pressure; Salesforce object model supplies admin-facing object semantics. This IP closes the relevant gap by binding `011 query engine 3layer kg` to concrete `ontology` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
