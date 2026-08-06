---
id: ADR-0480
title: "oya-cost: bespoke Rust K8s cost allocation substrate"
status: Superseded
date: 2026-05-28
authority: founder
owner: council-cost
planning_impact: true
supersedes: [ADR-0443]
superseded_by: [ADR-701]
amended_by: [ADR-0632]
milestone: M-COST-ALLOCATION-V2
related: [ADR-0443, ADR-0479, ADR-0193, ADR-0407, ADR-0083, ADR-0411, ADR-0423, ADR-0509]
---
# ADR-0480 — oya-cost: bespoke Rust K8s cost allocation substrate

## Status

Accepted — 2026-05-28. Supersedes ADR-0443 (OpenCost Phase-1).

## ADR-0632 product-protocol reconciliation

The tenant cost API is public HTTPS REST documented by OpenAPI 3.2.0, with signed/versioned webhooks, AsyncAPI/CloudEvents events, SSE, or WebSocket used where their semantics apply. Public GraphQL, gRPC, gRPC-Web, and Connect are forbidden. Cost-to-meter and other sibling-service calls may use internal-only gRPC/proto3 over HTTP/2.

## Context

AWS and GCP operate bespoke cost allocation substrates internally (AWS Cost Allocation Tags + Cost Explorer engine; GCP Cost Recommendations + Billing BigQuery export pipeline). Neither ships a general-purpose open-source cost engine as their internal tool. OpenCost (ADR-0443 Phase-1) provided a bootstrapping shortcut but introduces an external operator dependency, a Go runtime, and a Prometheus-scrape-only data model incompatible with ClickHouse time-series storage (ADR-0193) and Cedar tenant scoping (ADR-0083).

Bounded complexity: K8s cost allocation reduces to three primitives — (1) node unit price from Karpenter provisioner annotations (ADR-0423), (2) pod resource fraction from Mimir metrics, (3) attribution labels (tenant, region, runtime-tier). A bespoke Rust service implementing these three primitives is smaller than the OpenCost operator surface and aligns with the Rust-native doctrine.

## Decision

Ship `microservices/oya-cost/` as a bespoke Rust µservice (Axum public HTTPS REST plus internal-only gRPC/proto3 over HTTP/2). ClickHouse for time-series cost data; PostgreSQL for catalog. Cedar (ADR-0083) gates per-tenant cost-API access. OpenCost is retired.

## Deliverables

| ID | Deliverable | Exit criteria |
|---|---|---|
| D1 | `microservices/oya-cost/` Rust workspace | `cargo check` passes; Axum serves `/healthz`; internal gRPC/proto3 skeleton wired |
| D2 | K8s resource cost computation | Ingest Mimir resource-usage metrics + Karpenter node-pricing → per-pod/per-tenant/per-µservice allocation; labels: tenant, region, runtime-tier |
| D3 | Cost-as-SLI | Per-tenant cost-per-request metrics; OpenSLO (ADR-0441) cost-bound objectives; Polars (ADR-0420) materialized streams for trend analysis |
| D4 | oya-meter → oya-billing feed | Feed oya-meter (ADR-0479) → oya-billing (ADR-0478) for tenant invoicing; Cedar gates per-tenant cost-API access |
| D5 | Multi-region cost allocation | Per-region cost allocation (ADR-0418); tenant region-budget enforcement via Cedar gate |

## Hyperscaler-lens

| Criterion | Result |
|---|---|
| Active upstream | ✅ all deps (Axum, ClickHouse, Cedar) have active upstreams |
| Clean license | ✅ Apache-2 / MIT throughout |
| Fully self-hostable | ✅ no managed-service dependency |
| Hyperscaler-internal equivalent | ✅ matches AWS Cost Allocation + GCP Cost Recommendations bespoke pattern |

## Alternatives Considered

| Alternative | Reason rejected |
|---|---|
| OpenCost (ADR-0443 Phase-1) | Go runtime; Prometheus-scrape-only model; external operator dep; incompatible with ClickHouse + Cedar scoping |
| Kubecost commercial | Commercial license; managed-service dependency; violates self-hostable criterion |

## Integration

- **Karpenter (ADR-0423)**: node unit price sourced from Karpenter provisioner node-class annotations.
- **Mimir (ADR-0383)**: pod CPU/memory usage metrics ingested via Mimir remote-read.
- **ClickHouse (ADR-0193)**: time-series cost data store; Polars streams for trend materialization.
- **Cedar (ADR-0083)**: per-tenant cost-API authorization; region-budget enforcement.
- **oya-meter (ADR-0479)**: oya-cost publishes per-request cost signals consumed by oya-meter.
- **oya-billing (ADR-0478)**: oya-meter feeds oya-billing; oya-cost is the upstream cost source.

## Promotion Rationale

OpenCost Phase-1 unblocked cost visibility. oya-cost Phase-2 closes the integration gap with Cedar scoping, ClickHouse storage, Polars trend analysis, and the oya-meter/oya-billing invoicing chain. Bounded implementation: three computational primitives, one Rust workspace, zero new external operator dependencies.

## Consequences

- OpenCost operator uninstalled from cluster on D1 ship.
- oya-cost owns the cost-allocation SLI surface; SLO authored at `microservices/oya-cost/slos/` before promotion past dev.
- ADR-0443 status updated to Superseded.

## Implementation pattern (ADR-0509 alignment)

Per ADR-0509 (Hyperscaler service decomposition pattern), `oya-cost` ships as **single-crate-per-service with mod-based subsystems**. Per-use-case crate sprawl is superseded. Use cases remain valid as domain concepts (subsystem boundaries inside `src/<subsystem>/`).
