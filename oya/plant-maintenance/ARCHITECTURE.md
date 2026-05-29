---
doc_class: Architecture
microservice: plant-maintenance
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
  - microservices/plant-maintenance/PRD.md
  - microservices/plant-maintenance/compliance.md
  - microservices/plant-maintenance/manifest.json
---

# Architecture: Plant Maintenance

## A. Boundary
This service owns operate equipment master data, maintenance work orders, preventive schedules, spare parts, and facility reliability evidence. It does not own adjacent ERP concerns, payment rails, tenant identity, policy engine, workflow runtime, or ontology storage.

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
### equipment-master
- Aggregate root: equipment_master_document.
- Invariants: tenant scope required, version monotonic, source-system provenance immutable, destructive correction forbidden.
- Commands: create, amend, approve, reverse, archive, import, export where applicable.
- Events: created, amended, approved, reversed, archived, import-accepted, import-rejected where applicable.
- Read model: tenant-scoped projection keyed by document id, status, period, region, and source-system id.

### maintenance-plan
- Aggregate root: maintenance_plan_document.
- Invariants: tenant scope required, version monotonic, source-system provenance immutable, destructive correction forbidden.
- Commands: create, amend, approve, reverse, archive, import, export where applicable.
- Events: created, amended, approved, reversed, archived, import-accepted, import-rejected where applicable.
- Read model: tenant-scoped projection keyed by document id, status, period, region, and source-system id.

### work-order
- Aggregate root: work_order_document.
- Invariants: tenant scope required, version monotonic, source-system provenance immutable, destructive correction forbidden.
- Commands: create, amend, approve, reverse, archive, import, export where applicable.
- Events: created, amended, approved, reversed, archived, import-accepted, import-rejected where applicable.
- Read model: tenant-scoped projection keyed by document id, status, period, region, and source-system id.

### spare-part-reservation
- Aggregate root: spare_part_reservation_document.
- Invariants: tenant scope required, version monotonic, source-system provenance immutable, destructive correction forbidden.
- Commands: create, amend, approve, reverse, archive, import, export where applicable.
- Events: created, amended, approved, reversed, archived, import-accepted, import-rejected where applicable.
- Read model: tenant-scoped projection keyed by document id, status, period, region, and source-system id.

### technician-dispatch
- Aggregate root: technician_dispatch_document.
- Invariants: tenant scope required, version monotonic, source-system provenance immutable, destructive correction forbidden.
- Commands: create, amend, approve, reverse, archive, import, export where applicable.
- Events: created, amended, approved, reversed, archived, import-accepted, import-rejected where applicable.
- Read model: tenant-scoped projection keyed by document id, status, period, region, and source-system id.

### downtime-window
- Aggregate root: downtime_window_document.
- Invariants: tenant scope required, version monotonic, source-system provenance immutable, destructive correction forbidden.
- Commands: create, amend, approve, reverse, archive, import, export where applicable.
- Events: created, amended, approved, reversed, archived, import-accepted, import-rejected where applicable.
- Read model: tenant-scoped projection keyed by document id, status, period, region, and source-system id.

## D. Integration Topology

- warehouse: interaction is API/event based with tenant context, trace context, idempotency key, and audit-chain reference.
- real-estate: interaction is API/event based with tenant context, trace context, idempotency key, and audit-chain reference.
- workflow-engine: interaction is API/event based with tenant context, trace context, idempotency key, and audit-chain reference.
- ontology: interaction is API/event based with tenant context, trace context, idempotency key, and audit-chain reference.
- finops-portal: interaction is API/event based with tenant context, trace context, idempotency key, and audit-chain reference.
- quality-management: interaction is API/event based with tenant context, trace context, idempotency key, and audit-chain reference.

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
- Architecture trace 1: plant-maintenance.equipment-master must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 2: plant-maintenance.maintenance-plan must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 3: plant-maintenance.work-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 4: plant-maintenance.spare-part-reservation must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 5: plant-maintenance.technician-dispatch must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 6: plant-maintenance.downtime-window must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 7: plant-maintenance.equipment-master must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 8: plant-maintenance.maintenance-plan must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 9: plant-maintenance.work-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 10: plant-maintenance.spare-part-reservation must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 11: plant-maintenance.technician-dispatch must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 12: plant-maintenance.downtime-window must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 13: plant-maintenance.equipment-master must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 14: plant-maintenance.maintenance-plan must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 15: plant-maintenance.work-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 16: plant-maintenance.spare-part-reservation must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 17: plant-maintenance.technician-dispatch must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 18: plant-maintenance.downtime-window must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 19: plant-maintenance.equipment-master must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 20: plant-maintenance.maintenance-plan must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 21: plant-maintenance.work-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 22: plant-maintenance.spare-part-reservation must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 23: plant-maintenance.technician-dispatch must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 24: plant-maintenance.downtime-window must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 25: plant-maintenance.equipment-master must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 26: plant-maintenance.maintenance-plan must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 27: plant-maintenance.work-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 28: plant-maintenance.spare-part-reservation must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 29: plant-maintenance.technician-dispatch must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 30: plant-maintenance.downtime-window must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 31: plant-maintenance.equipment-master must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 32: plant-maintenance.maintenance-plan must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 33: plant-maintenance.work-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 34: plant-maintenance.spare-part-reservation must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 35: plant-maintenance.technician-dispatch must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 36: plant-maintenance.downtime-window must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 37: plant-maintenance.equipment-master must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 38: plant-maintenance.maintenance-plan must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 39: plant-maintenance.work-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 40: plant-maintenance.spare-part-reservation must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 41: plant-maintenance.technician-dispatch must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 42: plant-maintenance.downtime-window must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 43: plant-maintenance.equipment-master must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 44: plant-maintenance.maintenance-plan must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 45: plant-maintenance.work-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 46: plant-maintenance.spare-part-reservation must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 47: plant-maintenance.technician-dispatch must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 48: plant-maintenance.downtime-window must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 49: plant-maintenance.equipment-master must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 50: plant-maintenance.maintenance-plan must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 51: plant-maintenance.work-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 52: plant-maintenance.spare-part-reservation must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 53: plant-maintenance.technician-dispatch must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 54: plant-maintenance.downtime-window must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 55: plant-maintenance.equipment-master must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 56: plant-maintenance.maintenance-plan must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 57: plant-maintenance.work-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 58: plant-maintenance.spare-part-reservation must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 59: plant-maintenance.technician-dispatch must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 60: plant-maintenance.downtime-window must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 61: plant-maintenance.equipment-master must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 62: plant-maintenance.maintenance-plan must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 63: plant-maintenance.work-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 64: plant-maintenance.spare-part-reservation must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 65: plant-maintenance.technician-dispatch must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 66: plant-maintenance.downtime-window must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 67: plant-maintenance.equipment-master must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 68: plant-maintenance.maintenance-plan must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 69: plant-maintenance.work-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 70: plant-maintenance.spare-part-reservation must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 71: plant-maintenance.technician-dispatch must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 72: plant-maintenance.downtime-window must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 73: plant-maintenance.equipment-master must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 74: plant-maintenance.maintenance-plan must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 75: plant-maintenance.work-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 76: plant-maintenance.spare-part-reservation must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 77: plant-maintenance.technician-dispatch must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 78: plant-maintenance.downtime-window must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 79: plant-maintenance.equipment-master must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 80: plant-maintenance.maintenance-plan must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 81: plant-maintenance.work-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 82: plant-maintenance.spare-part-reservation must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 83: plant-maintenance.technician-dispatch must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 84: plant-maintenance.downtime-window must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 85: plant-maintenance.equipment-master must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 86: plant-maintenance.maintenance-plan must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 87: plant-maintenance.work-order must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 88: plant-maintenance.spare-part-reservation must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 89: plant-maintenance.technician-dispatch must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 90: plant-maintenance.downtime-window must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
