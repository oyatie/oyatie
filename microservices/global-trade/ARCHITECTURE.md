---
doc_class: Architecture
microservice: global-trade
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
  - microservices/global-trade/PRD.md
  - microservices/global-trade/compliance.md
  - microservices/global-trade/manifest.json
---

# Architecture: Global Trade

## A. Boundary
This service owns control customs declarations, sanctioned-party screening, export controls, denied-party evidence, and trade-compliance holds. It does not own adjacent ERP concerns, payment rails, tenant identity, policy engine, workflow runtime, or ontology storage.

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
### customs-declaration
- Aggregate root: customs_declaration_document.
- Invariants: tenant scope required, version monotonic, source-system provenance immutable, destructive correction forbidden.
- Commands: create, amend, approve, reverse, archive, import, export where applicable.
- Events: created, amended, approved, reversed, archived, import-accepted, import-rejected where applicable.
- Read model: tenant-scoped projection keyed by document id, status, period, region, and source-system id.

### sanctions-screening
- Aggregate root: sanctions_screening_document.
- Invariants: tenant scope required, version monotonic, source-system provenance immutable, destructive correction forbidden.
- Commands: create, amend, approve, reverse, archive, import, export where applicable.
- Events: created, amended, approved, reversed, archived, import-accepted, import-rejected where applicable.
- Read model: tenant-scoped projection keyed by document id, status, period, region, and source-system id.

### export-control-classification
- Aggregate root: export_control_classification_document.
- Invariants: tenant scope required, version monotonic, source-system provenance immutable, destructive correction forbidden.
- Commands: create, amend, approve, reverse, archive, import, export where applicable.
- Events: created, amended, approved, reversed, archived, import-accepted, import-rejected where applicable.
- Read model: tenant-scoped projection keyed by document id, status, period, region, and source-system id.

### trade-document
- Aggregate root: trade_document_document.
- Invariants: tenant scope required, version monotonic, source-system provenance immutable, destructive correction forbidden.
- Commands: create, amend, approve, reverse, archive, import, export where applicable.
- Events: created, amended, approved, reversed, archived, import-accepted, import-rejected where applicable.
- Read model: tenant-scoped projection keyed by document id, status, period, region, and source-system id.

### denied-party-hit
- Aggregate root: denied_party_hit_document.
- Invariants: tenant scope required, version monotonic, source-system provenance immutable, destructive correction forbidden.
- Commands: create, amend, approve, reverse, archive, import, export where applicable.
- Events: created, amended, approved, reversed, archived, import-accepted, import-rejected where applicable.
- Read model: tenant-scoped projection keyed by document id, status, period, region, and source-system id.

### broker-filing
- Aggregate root: broker_filing_document.
- Invariants: tenant scope required, version monotonic, source-system provenance immutable, destructive correction forbidden.
- Commands: create, amend, approve, reverse, archive, import, export where applicable.
- Events: created, amended, approved, reversed, archived, import-accepted, import-rejected where applicable.
- Read model: tenant-scoped projection keyed by document id, status, period, region, and source-system id.

## D. Integration Topology

- marketplace: interaction is API/event based with tenant context, trace context, idempotency key, and audit-chain reference.
- warehouse: interaction is API/event based with tenant context, trace context, idempotency key, and audit-chain reference.
- connect: interaction is API/event based with tenant context, trace context, idempotency key, and audit-chain reference.
- compliance: interaction is API/event based with tenant context, trace context, idempotency key, and audit-chain reference.
- payments: interaction is API/event based with tenant context, trace context, idempotency key, and audit-chain reference.
- supply-chain-planning: interaction is API/event based with tenant context, trace context, idempotency key, and audit-chain reference.

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
- Architecture trace 1: global-trade.customs-declaration must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 2: global-trade.sanctions-screening must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 3: global-trade.export-control-classification must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 4: global-trade.trade-document must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 5: global-trade.denied-party-hit must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 6: global-trade.broker-filing must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 7: global-trade.customs-declaration must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 8: global-trade.sanctions-screening must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 9: global-trade.export-control-classification must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 10: global-trade.trade-document must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 11: global-trade.denied-party-hit must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 12: global-trade.broker-filing must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 13: global-trade.customs-declaration must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 14: global-trade.sanctions-screening must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 15: global-trade.export-control-classification must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 16: global-trade.trade-document must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 17: global-trade.denied-party-hit must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 18: global-trade.broker-filing must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 19: global-trade.customs-declaration must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 20: global-trade.sanctions-screening must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 21: global-trade.export-control-classification must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 22: global-trade.trade-document must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 23: global-trade.denied-party-hit must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 24: global-trade.broker-filing must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 25: global-trade.customs-declaration must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 26: global-trade.sanctions-screening must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 27: global-trade.export-control-classification must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 28: global-trade.trade-document must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 29: global-trade.denied-party-hit must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 30: global-trade.broker-filing must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 31: global-trade.customs-declaration must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 32: global-trade.sanctions-screening must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 33: global-trade.export-control-classification must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 34: global-trade.trade-document must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 35: global-trade.denied-party-hit must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 36: global-trade.broker-filing must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 37: global-trade.customs-declaration must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 38: global-trade.sanctions-screening must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 39: global-trade.export-control-classification must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 40: global-trade.trade-document must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 41: global-trade.denied-party-hit must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 42: global-trade.broker-filing must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 43: global-trade.customs-declaration must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 44: global-trade.sanctions-screening must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 45: global-trade.export-control-classification must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 46: global-trade.trade-document must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 47: global-trade.denied-party-hit must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 48: global-trade.broker-filing must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 49: global-trade.customs-declaration must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 50: global-trade.sanctions-screening must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 51: global-trade.export-control-classification must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 52: global-trade.trade-document must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 53: global-trade.denied-party-hit must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 54: global-trade.broker-filing must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 55: global-trade.customs-declaration must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 56: global-trade.sanctions-screening must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 57: global-trade.export-control-classification must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 58: global-trade.trade-document must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 59: global-trade.denied-party-hit must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 60: global-trade.broker-filing must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 61: global-trade.customs-declaration must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 62: global-trade.sanctions-screening must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 63: global-trade.export-control-classification must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 64: global-trade.trade-document must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 65: global-trade.denied-party-hit must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 66: global-trade.broker-filing must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 67: global-trade.customs-declaration must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 68: global-trade.sanctions-screening must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 69: global-trade.export-control-classification must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 70: global-trade.trade-document must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 71: global-trade.denied-party-hit must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 72: global-trade.broker-filing must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 73: global-trade.customs-declaration must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 74: global-trade.sanctions-screening must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 75: global-trade.export-control-classification must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 76: global-trade.trade-document must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 77: global-trade.denied-party-hit must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 78: global-trade.broker-filing must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 79: global-trade.customs-declaration must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 80: global-trade.sanctions-screening must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 81: global-trade.export-control-classification must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 82: global-trade.trade-document must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 83: global-trade.denied-party-hit must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 84: global-trade.broker-filing must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 85: global-trade.customs-declaration must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 86: global-trade.sanctions-screening must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 87: global-trade.export-control-classification must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 88: global-trade.trade-document must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 89: global-trade.denied-party-hit must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
- Architecture trace 90: global-trade.broker-filing must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132.
