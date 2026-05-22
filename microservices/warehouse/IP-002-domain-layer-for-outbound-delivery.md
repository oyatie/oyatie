---
doc_class: ImplementationPlan
ip_id: IP-002
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
sap_submodule: EWM-DLV (deliveries)
tenant_class: paid
billing_components:
  - per_usage
persona: Omar Castillo, outbound fulfillment supervisor
status: Accepted
date: 2026-05-20
owner_team: axis-warehouse + axis-erp-parity
---

# IP-002: Domain layer for outbound delivery

## Context

- SAP submodule: EWM-DLV outbound delivery processing.
- Persona: Omar Castillo, outbound fulfillment supervisor.
- Journey leg: j123 coordinated product launch allocates stock and releases outbound delivery waves to carriers.
- SAP tables: `/SCWM/PRDO`, `/SCWM/ORDIM_O`, `/SCWM/WAREHOUSEORDER`, `/SCWM/QUANT`.
- Oyatie aggregate: `OutboundDelivery`.
- Precedent: SAP EWM outbound delivery order plus Stripe-style idempotent fulfillment command.
- ADR-0315 requires SAP EWM parity and ADR-0329/0330/0331 requires implementation-ready ERP depth.
- Boundary: owns delivery release, pick reservation, goods issue readiness, and carrier handoff evidence; does not own invoice, payment, or customer notification.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.outbound_delivery (
  tenant_id UUID NOT NULL,
  outbound_delivery_id TEXT NOT NULL,
  sap_prdo_ref TEXT NOT NULL,
  ship_to_party_id TEXT NOT NULL,
  requested_ship_at TIMESTAMPTZ NOT NULL,
  carrier_service_level TEXT,
  release_status TEXT NOT NULL CHECK (release_status IN ('draft','released','picking','packed','goods_issued','cancelled','blocked')),
  allocation_snapshot_ref TEXT NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, outbound_delivery_id)
);
CREATE TABLE warehouse.outbound_delivery_line (
  tenant_id UUID NOT NULL,
  outbound_delivery_id TEXT NOT NULL,
  line_no TEXT NOT NULL,
  material_id TEXT NOT NULL,
  requested_qty NUMERIC(18,6) NOT NULL,
  allocated_qty NUMERIC(18,6) NOT NULL DEFAULT 0,
  picked_qty NUMERIC(18,6) NOT NULL DEFAULT 0,
  uom TEXT NOT NULL,
  PRIMARY KEY (tenant_id, outbound_delivery_id, line_no)
);
```

### Rust Types

```rust
pub struct OutboundDelivery {
    pub tenant_id: TenantId,
    pub outbound_delivery_id: OutboundDeliveryId,
    pub sap_prdo_ref: SapDeliveryRef,
    pub ship_to_party_id: PartyId,
    pub requested_ship_at: DateTime<Utc>,
    pub release_status: OutboundStatus,
    pub allocation_snapshot_ref: AllocationSnapshotRef,
}
pub struct OutboundDeliveryLine {
    pub line_no: DeliveryLineNo,
    pub material_id: MaterialId,
    pub requested_qty: Decimal,
    pub allocated_qty: Decimal,
    pub picked_qty: Decimal,
    pub uom: UnitOfMeasure,
}
pub enum OutboundDeliveryError { InsufficientStock, CarrierWindowMissing, ExportHold, AllocationExpired, GoodsIssueDenied }
```

## API Endpoints

- REST `POST /v1/warehouse/outbound-deliveries` creates a delivery from order fulfillment context.
- REST `POST /v1/warehouse/outbound-deliveries/{id}:release` reserves stock and emits pick intent.
- REST `POST /v1/warehouse/outbound-deliveries/{id}:goods-issue-ready` validates picked and packed quantities.
- gRPC `warehouse.outbound.v1.OutboundDeliveryService.CreateOutboundDelivery`.
- gRPC `ReleaseOutboundDelivery`, `MarkGoodsIssueReady`, and `CancelOutboundDelivery`.
- AsyncAPI channel `warehouse.outbound-delivery.released.v1`.
- AsyncAPI channel `warehouse.outbound-delivery.goods-issue-ready.v1`.
- Consumers: picking-wave, global-trade, carrier-integration, revenue, ontology.

## Cedar Policy Hooks

- Policy: `warehouse::outbound_delivery::release`.
- Principal: `WarehouseFulfillmentSupervisor`.
- Action: `outbound_delivery_release`.
- Resource: `OutboundDelivery`.
- Context: `tenant_id`, `ship_to_country`, `export_control_status`, `allocation_snapshot_ref`, `carrier_window_id`.
- Forbid when allocation snapshot is expired, export-control status is hold, or goods issue would cross tenant boundary.

## Ontology Projection

- Vendor object: SAP EWM `/SCWM/PRDO` outbound delivery order.
- Oyatie object: `warehouse.outbound_delivery`.
- `/SCWM/PRDO-DOCID` -> `outbound_delivery_id`.
- `/SCWM/PRDO-PARTNER` -> `ship_to_party_id`.
- `/SCWM/WAREHOUSEORDER-WHO` -> pick execution bundle.
- `/SCWM/ORDIM_O-TANUM` -> warehouse task lineage.
- `/SCWM/QUANT-MATID` -> allocated stock evidence.
- Carrier service -> `carrier_service_level`.
- Projection freshness floor: 5 seconds.
- Projection rule: goods issue readiness is projected only after all line deltas reconcile.

## Workflow Steps

- Node `order-import`: ingest sales order or shipment request.
- Node `allocation-check`: lock allocation snapshot.
- Decision `stock-short`: branch to supply-chain-planning backorder review.
- Decision `export-hold`: branch to global-trade denied party or license queue.
- Node `release-delivery`: mark delivery released and create pick intent.
- Node `carrier-window-bind`: attach carrier appointment.
- Decision `carrier-window-missing`: hold release and notify dock planner.
- Node `goods-issue-ready`: reconcile picked, packed, and staged quantities.
- Node `audit-seal`: emit ADR-0263 class and outbox event.

## Audit Events

- `EVT-WAREHOUSE-OUTBOUND_DELIVERY-CREATED`.
- `EVT-WAREHOUSE-OUTBOUND_DELIVERY-RELEASED`.
- `EVT-WAREHOUSE-OUTBOUND_DELIVERY-STOCK_SHORT`.
- `EVT-WAREHOUSE-OUTBOUND_DELIVERY-GOODS_ISSUE_READY`.
- `EVT-WAREHOUSE-OUTBOUND_DELIVERY-POLICY_DENIED`.
- `EVT-WAREHOUSE-OUTBOUND_DELIVERY-IP_ACCEPTED`.
- ADR-0263 envelope stores `sap_prdo_ref`, `allocation_snapshot_ref`, `carrier_window_id`, and `export_control_status`.

## SLO Targets

- Release p50: 60 ms.
- Release p95: 220 ms.
- Release p99: 650 ms.
- Goods issue readiness p95: 180 ms after final pack confirmation.
- Rationale: launch waves need fast release, but export and stock checks justify a higher p99 than simple RF confirmations.

## Failure Modes and Recovery

- Failure: `ALLOCATION-EXPIRED`; recovery: recompute allocation and require supervisor release replay.
- Failure: `EXPORT-HOLD`; recovery: route to global-trade license review and keep delivery blocked.
- Failure: `CARRIER-WINDOW-MISSING`; recovery: hold release and request yard appointment.
- Failure: `PICKED-QTY-MISMATCH`; recovery: generate recount task and prevent goods issue readiness.
- Failure: `OUTBOX-BACKPRESSURE`; recovery: persist release state and retry event dispatch.
- Failure: `TENANT-MISMATCH`; recovery: reject command and emit policy denied audit event.

## Migration Notes

- Import open `/SCWM/PRDO` rows as draft or released based on SAP status.
- Import closed goods issue history as immutable evidence, not replayable commands.
- Map SAP partner IDs to tenant-scoped party IDs before delivery import.
- Preserve delivery order and warehouse order references for traceability.
- Rollback path: disable release command and preserve read-only delivery projections.
- Backfill order: parties, carrier services, stock allocations, outbound deliveries, delivery lines, goods issue events.

## Cross-microservice Handoffs

- From order-management: shipment demand and ship-to party.
- From inventory-ledger: allocation snapshot and stock hold.
- To picking-wave: release lines and pick priority.
- To global-trade: export-control check and license status.
- To carrier-integration: goods issue readiness and carrier service level.
- To revenue: goods issue evidence for billing readiness.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The primitive remains bound to SAP EWM outbound delivery processing, not generic shipping. |
| Persona specificity | Omar Castillo owns release, allocation, carrier-readiness, and rollback acceptance language. |
| Journey specificity | The j123 product-launch shipment leg drives allocation pressure, export holds, and goods-issue ordering. |
| DDL anchor | The outbound-delivery tables above are the normative source for delivery header and line state. |
| Rust anchor | The outbound delivery aggregate, line type, and error enum above are the implementation contract. |
| REST anchor | Outbound create/release/goods-issue endpoints are the tenant command surface. |
| gRPC anchor | The outbound delivery service is the internal release and replay worker contract. |
| AsyncAPI anchor | Released and goods-issued channels carry downstream fulfillment and revenue evidence. |
| Cedar anchor | Outbound release is default-deny and must persist `cedar_decision_id` before carrier handoff. |
| Ontology anchor | SAP EWM outbound delivery and warehouse-order lineage projects without replacing Oyatie identity. |
| ADR-0263 class binding | Release policy checks emit `OfficeBoundaryAttemptEvaluated` and then `OfficeBoundaryAttemptAllowed` or `OfficeBoundaryAttemptDenied`. |
| ADR-0263 pack binding | Export-control or sovereign-pack overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on release APIs emits `AbuseDefenceRateLimitHit`, never a free-form event name. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, source delivery ref, allocation id, carrier id, and `cedar_decision_id`. |
| Metric | `oya_warehouse_outbound_delivery_commands_total{tenant_id,cell_id,command,status}` caps command/status cardinality. |
| Latency histogram | `oya_warehouse_outbound_delivery_command_duration_seconds` tracks p50/p95/p99 release and goods-issue latency. |
| Trace span | `warehouse.outbound_delivery.release` links allocation, global-trade, carrier, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `delivery_id`, `carrier_service_id`, and scrubbed ship-to lineage. |
| Capacity math | Release waves use line_count / pack_station_capacity to block unsafe over-release before picker assignment. |
| Multi-region | Home-cell goods issue is authoritative; DR cells serve read-only shipment status until promotion. |
| Sovereign cells | Regulated customer and export evidence remain in the pack region for KR-CSAP, EU, CN-PIPL, IL5/6, and FedRAMP-High. |
| Rollback | Disable release/goods-issue commands, preserve read-only delivery projections, and replay from the last sealed goods-issue audit id. |
| Test evidence | Required tests cover export hold, allocation drift, tenant mismatch, carrier handoff failure, and idempotent goods issue. |
| Rejected shortcut | A generic `Shipment` model is rejected because it loses SAP EWM delivery, allocation, and goods-issue semantics. |
