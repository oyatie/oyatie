---
doc_class: ImplementationPlan
ip_id: IP-001
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
journey_ref: j102-raw-material-purchase-with-quality-attestation
sap_submodule: EWM-DLV (deliveries)
tenant_class: paid
billing_components:
  - per_usage
persona: Priya Menon, inbound dock lead
status: Accepted
date: 2026-05-20
owner_team: axis-warehouse + axis-erp-parity
---

# IP-001: Domain layer for inbound delivery

## Context

- SAP submodule: EWM-DLV inbound deliveries.
- Persona: Priya Menon, inbound dock lead.
- Journey leg: j102 raw material purchase arrives with supplier quality attestation before goods receipt.
- SAP tables: `/SCWM/PRDI`, `/SCWM/ORDIM_O`, `/SCWM/QUANT`, `/SCWM/STORAGEBIN`.
- Oyatie aggregate: `InboundDelivery`.
- Precedent: SAP EWM inbound delivery item validation plus AWS EventBridge outbox projection.
- ADR-0105 keeps validation in the domain layer, ADR-0244 scopes every row by tenant, and ADR-0263 binds audit classes.
- Boundary: this IP validates ASN, delivery item, dock, and first stock evidence; it does not own payment, supplier score, or quality inspection result.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.inbound_delivery (
  tenant_id UUID NOT NULL,
  inbound_delivery_id TEXT NOT NULL,
  sap_prdi_ref TEXT NOT NULL,
  supplier_id TEXT NOT NULL,
  expected_arrival_at TIMESTAMPTZ NOT NULL,
  dock_door_id TEXT,
  receiving_status TEXT NOT NULL CHECK (receiving_status IN ('expected','arrived','receiving','blocked','received','reversed')),
  quality_attestation_ref TEXT,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, inbound_delivery_id)
);
CREATE TABLE warehouse.inbound_delivery_item (
  tenant_id UUID NOT NULL,
  inbound_delivery_id TEXT NOT NULL,
  item_no TEXT NOT NULL,
  material_id TEXT NOT NULL,
  expected_qty NUMERIC(18,6) NOT NULL,
  received_qty NUMERIC(18,6) NOT NULL DEFAULT 0,
  uom TEXT NOT NULL,
  storage_type TEXT,
  inspection_lot_ref TEXT,
  PRIMARY KEY (tenant_id, inbound_delivery_id, item_no)
);
```

### Rust Types

```rust
pub struct InboundDelivery {
    pub tenant_id: TenantId,
    pub inbound_delivery_id: InboundDeliveryId,
    pub sap_prdi_ref: SapDeliveryRef,
    pub supplier_id: SupplierId,
    pub expected_arrival_at: DateTime<Utc>,
    pub receiving_status: ReceivingStatus,
    pub quality_attestation_ref: Option<QualityAttestationRef>,
}
pub struct InboundDeliveryItem {
    pub item_no: DeliveryItemNo,
    pub material_id: MaterialId,
    pub expected_qty: Decimal,
    pub received_qty: Decimal,
    pub uom: UnitOfMeasure,
    pub storage_type: Option<StorageType>,
}
pub enum InboundDeliveryError { AsnMismatch, TenantMismatch, QuantityOverReceipt, QualityHoldRequired, AuditSealFailed }
```

## API Endpoints

- REST `POST /v1/warehouse/inbound-deliveries` creates an expected inbound delivery from ASN or SAP extract.
- REST `POST /v1/warehouse/inbound-deliveries/{id}:receive-item` records item-level receipt.
- REST `POST /v1/warehouse/inbound-deliveries/{id}:reverse-receipt` reverses a received item with reason code.
- gRPC `warehouse.inbound.v1.InboundDeliveryService.CreateInboundDelivery`.
- gRPC `ReceiveInboundItem`, `ReverseReceipt`, and `GetInboundDelivery`.
- AsyncAPI channel `warehouse.inbound-delivery.received.v1`.
- AsyncAPI channel `warehouse.inbound-delivery.blocked.v1`.
- Consumers: quality-management, inventory-ledger, workflow-engine, ontology, compliance.

## Cedar Policy Hooks

- Policy: `warehouse::inbound_delivery::receive`.
- Principal: `WarehouseDockOperator`.
- Action: `inbound_delivery_receive`.
- Resource: `InboundDelivery`.
- Context: `tenant_id`, `dock_door_id`, `supplier_id`, `quality_attestation_ref`, `policy_bundle_version`.
- Forbid when tenant mismatches, quality attestation is missing for controlled material, or the dock is not assigned to the principal.

## Ontology Projection

- Vendor object: SAP EWM `/SCWM/PRDI` inbound delivery item.
- Oyatie object: `warehouse.inbound_delivery`.
- `/SCWM/PRDI-DOCID` -> `inbound_delivery_id`.
- `/SCWM/PRDI-ITEMID` -> `item_no`.
- `/SCWM/ORDIM_O-TANUM` -> putaway task lineage.
- `/SCWM/QUANT-LGPLA` -> current bin evidence after receipt.
- `/SCWM/STORAGEBIN-LGTYP` -> storage type candidate.
- Supplier ASN -> `quality_attestation_ref`.
- Projection freshness floor: 3 seconds.
- Projection rule: raw SAP references are lineage fields, not primary identity.

## Workflow Steps

- Node `asn-import`: load supplier ASN and SAP `/SCWM/PRDI` rows.
- Node `delivery-validate`: verify tenant, supplier, material, and expected quantity.
- Decision `quality-attestation-missing`: branch to `quality-management.attestation-review`.
- Node `dock-assign`: bind dock door and arrival window.
- Decision `dock-capacity-exceeded`: route to yard appointment reschedule.
- Node `receive-item`: record item receipt and first stock evidence.
- Decision `quantity-over-receipt`: block and require supervisor override.
- Node `emit-received`: publish received event and audit seal.
- Node `ontology-project`: publish inbound delivery projection.

## Audit Events

- `EVT-WAREHOUSE-INBOUND_DELIVERY-CREATED`.
- `EVT-WAREHOUSE-INBOUND_DELIVERY-ITEM_RECEIVED`.
- `EVT-WAREHOUSE-INBOUND_DELIVERY-QUALITY_BLOCKED`.
- `EVT-WAREHOUSE-INBOUND_DELIVERY-REVERSED`.
- `EVT-WAREHOUSE-INBOUND_DELIVERY-POLICY_DENIED`.
- `EVT-WAREHOUSE-INBOUND_DELIVERY-IP_ACCEPTED`.
- ADR-0263 envelope stores `sap_prdi_ref`, `supplier_id`, `quality_attestation_ref`, and `cedar_decision_id`.

## SLO Targets

- Create delivery p50: 45 ms.
- Create delivery p95: 160 ms.
- Create delivery p99: 420 ms.
- Item receipt p95: 120 ms for RF scanner round trip.
- Rationale: dock operators need sub-second acknowledgement to avoid truck queueing, but quality attestation checks may add bounded policy latency.

## Failure Modes and Recovery

- Failure: `ASN-MISMATCH` expected item differs from SAP delivery item; recovery: block delivery and open supplier discrepancy workflow.
- Failure: `QUALITY-ATTESTATION-MISSING`; recovery: park receipt in blocked status and request quality-management review.
- Failure: `OVER-RECEIPT`; recovery: require supervisor Cedar permit and create variance audit event.
- Failure: `AUDIT-SEAL-FAILED`; recovery: keep receipt pending and retry seal through audit outbox.
- Failure: `DOCK-WINDOW-EXPIRED`; recovery: hand off to yard appointment reschedule.
- Failure: `TENANT-SCOPE-DRIFT`; recovery: reject command and emit security event.

## Migration Notes

- Import `/SCWM/PRDI` open inbound deliveries as `expected`.
- Import goods receipt history as immutable receipt events when reversal period is closed.
- Map supplier ASN numbers to `quality_attestation_ref` only after source-system provenance is verified.
- Preserve SAP document IDs as lineage, not Oyatie primary keys.
- Rollback path: disable create and receive commands while keeping read-only migrated projections.
- Backfill order: suppliers, materials, storage bins, inbound deliveries, delivery items, receipt history.

## Cross-microservice Handoffs

- From procurement: purchase order and supplier ASN.
- From quality-management: attestation validity and inspection lot status.
- To inventory-ledger: first stock quantity evidence.
- To putaway task: candidate storage type and item quantity.
- To workflow-engine: exception approvals.
- To compliance: ADR-0263 receipt evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The primitive remains bound to SAP EWM delivery handling, not a generic warehouse receipt abstraction. |
| Persona specificity | Priya Menon is the accountable dock lead persona for acceptance, exception, and rollback language. |
| Journey specificity | The j102 supplier-quality-attested raw-material receipt leg drives the command ordering and failure modes. |
| DDL anchor | The `warehouse.inbound_delivery` and item tables above are the normative data start point. |
| Rust anchor | `InboundDelivery`, `InboundDeliveryItem`, and `InboundDeliveryError` are the implementation type names. |
| REST anchor | `POST /v1/warehouse/inbound-deliveries` and receive/reverse commands are the tenant API surface. |
| gRPC anchor | `warehouse.inbound.v1.InboundDeliveryService` is the internal worker and replay contract. |
| AsyncAPI anchor | `warehouse.inbound-delivery.received.v1` and blocked channels carry downstream receipt evidence. |
| Cedar anchor | `warehouse::inbound_delivery::receive` remains default-deny and must store `cedar_decision_id`. |
| Ontology anchor | SAP `/SCWM/PRDI` lineage projects to `warehouse.inbound_delivery` without becoming primary identity. |
| ADR-0263 class binding | Policy checks emit `OfficeBoundaryAttemptEvaluated` and then `OfficeBoundaryAttemptAllowed` or `OfficeBoundaryAttemptDenied`. |
| ADR-0263 pack binding | Compliance-pack activation or receipt overlay drift emits `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on receipt APIs emits `AbuseDefenceRateLimitHit`, never a free-form event name. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, `sap_prdi_ref`, `supplier_id`, `quality_attestation_ref`, and `cedar_decision_id`. |
| Metric | `oya_warehouse_inbound_delivery_commands_total{tenant_id,cell_id,command,status}` has cardinality capped by command/status. |
| Latency histogram | `oya_warehouse_inbound_delivery_command_duration_seconds` tracks p50/p95/p99 receipt latency per cell. |
| Trace span | `warehouse.inbound_delivery.receive_item` is child of API gateway span and parent of audit-chain seal span. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `dock_door_id`, `material_id`, and scrubbed supplier lineage. |
| Capacity math | Dock queue sizing uses arrival_rate * receipt_service_time; crossing 0.8 utilization blocks auto dock assignment. |
| Multi-region | Home-cell receipt writes are authoritative; DR cells serve read-only inbound projections until promotion. |
| Sovereign cells | KR-CSAP, EU-sovereign, CN-PIPL, IL5/6, and FedRAMP-High overlays keep supplier attestation evidence in-region. |
| Rollback | Disable create/receive commands, keep read-only migrated projections, and replay outbox from last sealed audit id. |
| Test evidence | Required tests cover tenant mismatch, missing attestation, over-receipt, audit seal failure, and replay idempotency. |
| Rejected shortcut | A generic `WarehouseReceipt` record is rejected because it would lose SAP `/SCWM/PRDI` lineage and quality gate evidence. |
