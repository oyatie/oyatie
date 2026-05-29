---
doc_class: ImplementationPlan
ip_id: IP-008
microservice: warehouse
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0245
  - ADR-0253
  - ADR-0263
  - ADR-0294
  - ADR-0297
  - ADR-0314
  - ADR-0315
  - ADR-0329
  - ADR-0330
  - ADR-0331
journey_ref: j123-multi-tenant-coordinated-product-launch
sap_submodule: EWM-WIM (inventory)
tenant_class: paid
billing_components:
  - per_usage
persona: Sophie Laurent, shipping execution analyst
status: Accepted
date: 2026-05-20
owner_team: axis-warehouse + axis-erp-parity
---

# IP-008: Usecase layer for outbound delivery

## Context

- SAP submodule: EWM-WIM inventory posting for outbound goods issue.
- Persona: Sophie Laurent, shipping execution analyst.
- Journey leg: j123 launch shipments are picked, packed, staged, and made ready for goods issue.
- SAP tables: `/SCWM/PRDO`, `/SCWM/QUANT`, `/SCWM/WAREHOUSEORDER`, `/SCWM/ORDIM_O`.
- Oyatie usecase: `ReleaseOutboundDelivery`.
- Precedent: SAP EWM goods issue process plus Amazon fulfillment allocation-to-ship guardrail.
- ADR-0297 gates stock release through Cedar and ADR-0263 seals goods issue readiness evidence.
- Boundary: orchestrates release, allocation validation, pick wave request, and goods-issue-ready event; billing remains revenue-owned.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.outbound_release_command (
  tenant_id UUID NOT NULL,
  command_id TEXT NOT NULL,
  outbound_delivery_id TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  release_strategy TEXT NOT NULL,
  command_status TEXT NOT NULL CHECK (command_status IN ('accepted','executing','succeeded','failed','compensated')),
  allocation_snapshot_ref TEXT NOT NULL,
  failure_code TEXT,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, command_id),
  UNIQUE (tenant_id, idempotency_key)
);
CREATE TABLE warehouse.outbound_release_line_result (
  tenant_id UUID NOT NULL,
  command_id TEXT NOT NULL,
  line_no TEXT NOT NULL,
  material_id TEXT NOT NULL,
  allocated_qty NUMERIC(18,6) NOT NULL,
  result_state TEXT NOT NULL,
  PRIMARY KEY (tenant_id, command_id, line_no)
);
```

### Rust Types

```rust
pub struct ReleaseOutboundDeliveryCommand {
    pub tenant_id: TenantId,
    pub command_id: CommandId,
    pub outbound_delivery_id: OutboundDeliveryId,
    pub idempotency_key: IdempotencyKey,
    pub release_strategy: ReleaseStrategy,
    pub allocation_snapshot_ref: AllocationSnapshotRef,
}
pub struct OutboundReleaseLineResult {
    pub line_no: DeliveryLineNo,
    pub material_id: MaterialId,
    pub allocated_qty: Decimal,
    pub result_state: ReleaseLineState,
}
pub enum ReleaseOutboundDeliveryError { StockShort, ExportHold, AllocationStale, PickWaveCreateFailed, GoodsIssueNotReady }
```

## API Endpoints

- REST `POST /v1/warehouse/outbound-deliveries/{id}:release` executes the outbound release usecase.
- REST `POST /v1/warehouse/outbound-release-commands/{command_id}:compensate` reverses a partial release.
- REST `GET /v1/warehouse/outbound-release-commands/{command_id}` returns command result.
- gRPC `warehouse.outbound_usecase.v1.ReleaseOutboundDelivery`.
- gRPC `CompensateOutboundRelease` and `GetOutboundReleaseCommand`.
- AsyncAPI channel `warehouse.outbound-release.succeeded.v1`.
- AsyncAPI channel `warehouse.outbound-release.failed.v1`.
- Consumers: picking-wave, inventory-ledger, global-trade, carrier-integration.

## Cedar Policy Hooks

- Policy: `warehouse::outbound_release::execute`.
- Principal: `WarehouseFulfillmentSupervisor`.
- Action: `release_outbound_delivery`.
- Resource: `OutboundDelivery`.
- Context: `tenant_id`, `allocation_snapshot_ref`, `export_control_status`, `ship_to_country`, `release_strategy`.
- Forbid when export hold is active, stock allocation is stale, or release strategy is not allowed for the tenant pack.

## Ontology Projection

- Vendor object: SAP EWM outbound goods issue usecase.
- Oyatie object: `warehouse.outbound_release_command`.
- `/SCWM/PRDO-DOCID` -> `outbound_delivery_id`.
- `/SCWM/QUANT-QUAN` -> allocated quantity.
- `/SCWM/WAREHOUSEORDER-WHO` -> wave or warehouse order membership.
- `/SCWM/ORDIM_O-TANUM` -> task lineage.
- Release strategy -> `release_strategy`.
- Allocation snapshot -> stock proof.
- Projection freshness floor: 5 seconds.
- Projection rule: failed line results remain visible for backorder and shortage analysis.

## Workflow Steps

- Node `command-accept`: dedupe release command.
- Node `export-check`: call global-trade policy context.
- Decision `export-hold`: fail command and create trade review.
- Node `allocation-lock`: validate stock snapshot.
- Decision `stock-short`: split delivery or backorder lines.
- Node `domain-release`: update outbound delivery state.
- Node `pick-wave-request`: hand off eligible tasks.
- Decision `pick-wave-create-failed`: compensate release or retry outbox.
- Node `goods-issue-readiness-watch`: wait for picked and packed signal.
- Node `audit-seal`: emit release evidence.

## Audit Events

- `EVT-WAREHOUSE-OUTBOUND_RELEASE-COMMAND_ACCEPTED`.
- `EVT-WAREHOUSE-OUTBOUND_RELEASE-STOCK_ALLOCATED`.
- `EVT-WAREHOUSE-OUTBOUND_RELEASE-EXPORT_BLOCKED`.
- `EVT-WAREHOUSE-OUTBOUND_RELEASE-COMPENSATED`.
- `EVT-WAREHOUSE-OUTBOUND_RELEASE-POLICY_DENIED`.
- `EVT-WAREHOUSE-OUTBOUND_RELEASE-IP_ACCEPTED`.
- ADR-0263 envelope stores `command_id`, `allocation_snapshot_ref`, `export_control_status`, and line results.

## SLO Targets

- Release command p50: 50 ms.
- Release command p95: 180 ms.
- Release command p99: 520 ms.
- Pick wave request p95: 700 ms for 5,000 delivery lines.
- Rationale: stock allocation and trade checks are synchronous; wave creation can use durable asynchronous handoff.

## Failure Modes and Recovery

- Failure: `EXPORT-HOLD`; recovery: route to global-trade and keep delivery blocked.
- Failure: `STOCK-SHORT`; recovery: split delivery and emit backorder handoff.
- Failure: `ALLOCATION-STALE`; recovery: recompute allocation snapshot and retry command.
- Failure: `PICK-WAVE-CREATE-FAILED`; recovery: retry outbox and keep delivery released-pending-wave.
- Failure: `GOODS-ISSUE-NOT-READY`; recovery: keep release open and notify shipping lead.
- Failure: `COMPENSATION-DENIED`; recovery: require supervisor override and preserve partial state evidence.

## Migration Notes

- Import open outbound releases as command records only when they still require work.
- Closed SAP goods issue rows become non-replayable evidence.
- Preserve allocation snapshot lineage from source stock state where available.
- Recompute release line state from picked and packed quantities.
- Rollback path: disable release usecase and leave outbound delivery in draft/released domain state.
- Backfill order: outbound delivery, stock allocation, release command, line result, pick-wave request.

## Cross-microservice Handoffs

- From order-management: fulfillment demand.
- From global-trade: export hold and license status.
- From inventory-ledger: allocation snapshot.
- To picking-wave: pick task candidate lines.
- To carrier-integration: staged shipment readiness.
- To revenue: goods issue readiness evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The usecase remains bound to SAP EWM inventory posting for outbound goods issue. |
| Persona specificity | Sophie Laurent owns release orchestration, export hold handling, and rollback acceptance language. |
| Journey specificity | The j123 shipment-readiness leg drives pick, pack, stage, and goods-issue command ordering. |
| DDL anchor | The outbound release command and line-result tables above are the normative usecase persistence model. |
| Rust anchor | The outbound release command, result, and error enum above are the implementation contract. |
| REST anchor | Release, hold, unhold, and goods-issue preparation endpoints are the tenant command surface. |
| gRPC anchor | The outbound usecase service is the worker and replay contract for release orchestration. |
| AsyncAPI anchor | Released, held, and goods-issue-ready channels carry fulfillment evidence. |
| Cedar anchor | Release operation is default-deny and must persist `cedar_decision_id` before wave creation. |
| Ontology anchor | SAP EWM inventory and outbound delivery lineage projects to release command nodes. |
| ADR-0263 class binding | Release policy checks emit `OfficeBoundaryAttemptEvaluated` plus allowed or denied outcome classes. |
| ADR-0263 pack binding | Export, tax, or sovereign overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on release usecase APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, outbound delivery id, allocation id, hold reason, and `cedar_decision_id`. |
| Metric | `oya_warehouse_outbound_release_commands_total{tenant_id,cell_id,command,status}` caps cardinality. |
| Latency histogram | `oya_warehouse_outbound_release_duration_seconds` tracks release-to-wave-request latency. |
| Trace span | `warehouse.outbound_delivery.release_usecase` links order-management, global-trade, inventory-ledger, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `delivery_id`, `allocation_id`, `carrier_context`, and hold status. |
| Capacity math | Release blocks when allocation shortage probability exceeds the tenant threshold from current pickable-stock variance. |
| Multi-region | Goods-issue readiness writes stay in the home cell; DR cells serve read-only release projections. |
| Sovereign cells | Ship-to, export, and tax evidence remain in-region for active compliance-pack overlays. |
| Rollback | Disable release usecase endpoints, leave domain delivery state intact, and replay from last sealed release audit id. |
| Test evidence | Required tests cover export hold, allocation drift, tenant mismatch, downstream wave failure, and idempotent release. |
| Rejected shortcut | A generic `FulfillmentRelease` usecase is rejected because it loses SAP EWM goods-issue semantics. |
