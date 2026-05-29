---
doc_class: Architecture
microservice: production-planning
status: reserved-wave-3-g-anchor
date: 2026-05-20
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0245
  - ADR-0315
companion_docs:
  - microservices/production-planning/PRD.md
  - microservices/production-planning/compliance.md
  - microservices/production-planning/manifest.json
---

# Architecture: Production Planning

## A. Boundary
This service owns plan and schedule manufacturing work from BOM through MRP, routing, capacity calendars, and shop-floor release. It does not own adjacent ERP concerns, payment rails, tenant identity, policy engine, workflow runtime, or ontology storage.

## B. Layer Map
| ADR-0105 layer | Planned responsibility |
|---|---|
| api | public command/query DTOs and OpenAPI 3.2.0 contract binding |
| rest | HTTP transport and idempotency enforcement |
| application | orchestration of usecases and transactions |
| usecase | command handlers and read models |
| domain | business invariants and aggregate roots |
| kernel | pure value objects and deterministic calculations |
| adapter | source-system, database, and external-system adapters |
| worker | batch migration, reconciliation, and async workflow workers |
| governance | policy, compliance, scorecards, and evidence gates |

## C. Bounded Context Architecture
### bom-revision
- Aggregate root: bom_revision_document.
- Invariants: tenant scope required, version monotonic, source-system provenance immutable, destructive correction forbidden.
- Commands: create, amend, approve, reverse, archive, import, export where applicable.
- Events: created, amended, approved, reversed, archived, import-accepted, import-rejected where applicable.
- Read model: tenant-scoped projection keyed by document id, status, period, region, and source-system id.

### mrp-run
- Aggregate root: mrp_run_document.
- Invariants: tenant scope required, version monotonic, source-system provenance immutable, destructive correction forbidden.
- Commands: create, amend, approve, reverse, archive, import, export where applicable.
- Events: created, amended, approved, reversed, archived, import-accepted, import-rejected where applicable.
- Read model: tenant-scoped projection keyed by document id, status, period, region, and source-system id.

### capacity-calendar
- Aggregate root: capacity_calendar_document.
- Invariants: tenant scope required, version monotonic, source-system provenance immutable, destructive correction forbidden.
- Commands: create, amend, approve, reverse, archive, import, export where applicable.
- Events: created, amended, approved, reversed, archived, import-accepted, import-rejected where applicable.
- Read model: tenant-scoped projection keyed by document id, status, period, region, and source-system id.

### routing-step
- Aggregate root: routing_step_document.
- Invariants: tenant scope required, version monotonic, source-system provenance immutable, destructive correction forbidden.
- Commands: create, amend, approve, reverse, archive, import, export where applicable.
- Events: created, amended, approved, reversed, archived, import-accepted, import-rejected where applicable.
- Read model: tenant-scoped projection keyed by document id, status, period, region, and source-system id.

### production-order
- Aggregate root: production_order_document.
- Invariants: tenant scope required, version monotonic, source-system provenance immutable, destructive correction forbidden.
- Commands: create, amend, approve, reverse, archive, import, export where applicable.
- Events: created, amended, approved, reversed, archived, import-accepted, import-rejected where applicable.
- Read model: tenant-scoped projection keyed by document id, status, period, region, and source-system id.

### shop-floor-release
- Aggregate root: shop_floor_release_document.
- Invariants: tenant scope required, version monotonic, source-system provenance immutable, destructive correction forbidden.
- Commands: create, amend, approve, reverse, archive, import, export where applicable.
- Events: created, amended, approved, reversed, archived, import-accepted, import-rejected where applicable.
- Read model: tenant-scoped projection keyed by document id, status, period, region, and source-system id.

## D. Integration Topology

- ontology: interaction is API/event based with tenant context, trace context, idempotency key, and audit-chain reference.
- workflow-engine: interaction is API/event based with tenant context, trace context, idempotency key, and audit-chain reference.
- warehouse: interaction is API/event based with tenant context, trace context, idempotency key, and audit-chain reference.
- quality-management: interaction is API/event based with tenant context, trace context, idempotency key, and audit-chain reference.
- finops-portal: interaction is API/event based with tenant context, trace context, idempotency key, and audit-chain reference.
- marketplace: interaction is API/event based with tenant context, trace context, idempotency key, and audit-chain reference.

## E. Failure Modes

- Source-system import drift: dry-run evidence identifies row, table, transform, and rejection reason.
- Cross-tenant reference attempt: Cedar denies before domain command execution and emits refusal evidence.
- Duplicate command submission: idempotency key returns previous result and increments duplicate metric.
- Regional outage: writes queue in the tenant home cell and reads expose stale-region metadata.
- Audit-chain outage: critical state transitions pause; non-critical queries continue with degraded banner.

## F. Data Integrity

Commands own local transactions. Cross-service work uses workflow-engine sagas and compensating transitions. Financial, inventory, trade, quality, and compliance documents reverse through explicit domain events rather than row deletion.

## G. Contracts

- REST: OpenAPI 3.2.0.
- Events: AsyncAPI 3.1.0.
- Internal RPC: proto3.
- Naming: BNF v4.1.
- Layers: ADR-0105 13-layer enum.

## H. Wave-3-G Follow-Up
The anchor architecture reserves the boundary. Wave-3-G adds contracts, Cedar policies, runbooks, SLOs, dashboards, catalog records, IaC, threat model, DPIA, capacity model, cost budget, failure modes, and implementation plans.
- Architecture trace 1: production-planning.bom-revision must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 2: production-planning.mrp-run must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 3: production-planning.capacity-calendar must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 4: production-planning.routing-step must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 5: production-planning.production-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 6: production-planning.shop-floor-release must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 7: production-planning.bom-revision must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 8: production-planning.mrp-run must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 9: production-planning.capacity-calendar must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 10: production-planning.routing-step must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 11: production-planning.production-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 12: production-planning.shop-floor-release must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 13: production-planning.bom-revision must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 14: production-planning.mrp-run must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 15: production-planning.capacity-calendar must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 16: production-planning.routing-step must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 17: production-planning.production-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 18: production-planning.shop-floor-release must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 19: production-planning.bom-revision must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 20: production-planning.mrp-run must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 21: production-planning.capacity-calendar must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 22: production-planning.routing-step must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 23: production-planning.production-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 24: production-planning.shop-floor-release must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 25: production-planning.bom-revision must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 26: production-planning.mrp-run must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 27: production-planning.capacity-calendar must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 28: production-planning.routing-step must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 29: production-planning.production-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 30: production-planning.shop-floor-release must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 31: production-planning.bom-revision must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 32: production-planning.mrp-run must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 33: production-planning.capacity-calendar must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 34: production-planning.routing-step must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 35: production-planning.production-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 36: production-planning.shop-floor-release must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 37: production-planning.bom-revision must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 38: production-planning.mrp-run must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 39: production-planning.capacity-calendar must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 40: production-planning.routing-step must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 41: production-planning.production-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 42: production-planning.shop-floor-release must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 43: production-planning.bom-revision must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 44: production-planning.mrp-run must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 45: production-planning.capacity-calendar must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 46: production-planning.routing-step must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 47: production-planning.production-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 48: production-planning.shop-floor-release must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 49: production-planning.bom-revision must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 50: production-planning.mrp-run must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 51: production-planning.capacity-calendar must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 52: production-planning.routing-step must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 53: production-planning.production-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 54: production-planning.shop-floor-release must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 55: production-planning.bom-revision must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 56: production-planning.mrp-run must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 57: production-planning.capacity-calendar must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 58: production-planning.routing-step must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 59: production-planning.production-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 60: production-planning.shop-floor-release must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 61: production-planning.bom-revision must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 62: production-planning.mrp-run must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 63: production-planning.capacity-calendar must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 64: production-planning.routing-step must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 65: production-planning.production-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 66: production-planning.shop-floor-release must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 67: production-planning.bom-revision must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 68: production-planning.mrp-run must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 69: production-planning.capacity-calendar must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 70: production-planning.routing-step must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 71: production-planning.production-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 72: production-planning.shop-floor-release must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 73: production-planning.bom-revision must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 74: production-planning.mrp-run must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 75: production-planning.capacity-calendar must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 76: production-planning.routing-step must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 77: production-planning.production-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 78: production-planning.shop-floor-release must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 79: production-planning.bom-revision must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 80: production-planning.mrp-run must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 81: production-planning.capacity-calendar must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 82: production-planning.routing-step must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 83: production-planning.production-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 84: production-planning.shop-floor-release must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 85: production-planning.bom-revision must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 86: production-planning.mrp-run must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 87: production-planning.capacity-calendar must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 88: production-planning.routing-step must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 89: production-planning.production-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 90: production-planning.shop-floor-release must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
