---
doc_class: ImplementationPlan
ip_id: IP-005
microservice: quality-management
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0253
  - ADR-0263
  - ADR-0294
  - ADR-0297
  - ADR-0314
  - ADR-0315
  - ADR-0329
  - ADR-0330
  - ADR-0331
  - ADR-0320
journey_ref: j118-supplier-defect-containment
sap_submodule: QM-QC Quality Control
tenant_class: paid
billing_components:
  - per_usage
persona: Elena Petrova, containment coordinator
status: Accepted
date: 2026-05-20
owner_team: axis-quality-management + axis-erp-parity
---

# IP-005: Domain layer for quality-hold containment

## Context

- SAP QM submodule: QM-QC Quality Control.
- Topic: quality hold and control-chart violation containment.
- Persona: Elena Petrova, containment coordinator.
- Journey: j118 supplier defect containment.
- Journey leg: suspect inventory is blocked before downstream consumption.
- SAP precedent: inspection stock, blocked stock, quality notification task, and usage decision block.
- Oyatie aggregate: `QualityHold`.
- Boundary: containment domain state and release invariants.
- ADR-0105 keeps hold logic inside the domain ring.
- ADR-0131 keeps the IP with this microservice.
- ADR-0244 requires tenant-scoped stock references.
- ADR-0263 binds hold audit events.
- ADR-0297 requires Cedar before hold or release.
- ADR-0314 prevents marketplace settlement mutation.
- ADR-0315 requires SAP QM parity.
- ADR-0329/0330/0331 requires implementation-ready ERP detail.
- A hold must be conservative: fail closed when evidence is incomplete.
- A release must prove disposition and authority.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.quality_hold (
  tenant_id UUID NOT NULL,
  hold_id TEXT NOT NULL,
  hold_type TEXT NOT NULL,
  material_id TEXT NOT NULL,
  batch_id TEXT,
  inventory_ref TEXT NOT NULL,
  source_notification_id TEXT,
  source_lot_id TEXT,
  reason_code TEXT NOT NULL,
  state TEXT NOT NULL,
  quantity_on_hold NUMERIC(20,6) NOT NULL,
  uom TEXT NOT NULL,
  release_requires_capa BOOLEAN NOT NULL,
  released_by_principal_id TEXT,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, hold_id)
);
CREATE TABLE quality_management.quality_hold_disposition (
  tenant_id UUID NOT NULL,
  hold_id TEXT NOT NULL,
  disposition_no INTEGER NOT NULL,
  disposition_type TEXT NOT NULL,
  quantity NUMERIC(20,6) NOT NULL,
  evidence_ref TEXT NOT NULL,
  posted_to_inventory BOOLEAN NOT NULL DEFAULT FALSE,
  PRIMARY KEY (tenant_id, hold_id, disposition_no)
);
```

### Rust Types

```rust
pub struct QualityHold {
    pub tenant_id: TenantId,
    pub hold_id: HoldId,
    pub hold_type: HoldType,
    pub material_id: MaterialId,
    pub batch_id: Option<BatchId>,
    pub inventory_ref: InventoryRef,
    pub source_notification_id: Option<NotificationId>,
    pub source_lot_id: Option<InspectionLotId>,
    pub reason_code: HoldReasonCode,
    pub state: HoldState,
    pub quantity_on_hold: Decimal,
    pub uom: UnitOfMeasure,
    pub release_requires_capa: bool,
    pub dispositions: Vec<HoldDisposition>,
}
pub enum HoldType { InspectionBlock, DefectContainment, SpcViolation, AuditFinding, Manual }
pub enum HoldState { Open, PartiallyDispositioned, PendingRelease, Released, Scrapped, Cancelled }
pub enum DispositionType { ReleaseToStock, Scrap, ReturnToSupplier, Rework, Downgrade }
pub enum HoldError {
    MissingInventoryRef,
    ReleaseWithoutDisposition,
    ReleaseRequiresCapa,
    QuantityMismatch,
    CrossTenantInventory,
    ReleasePolicyDenied,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/quality-holds`.
- Opens a hold against inventory, batch, lot, or notification.
- `POST /v1/quality-management/quality-holds/{hold_id}:disposition`.
- Adds release, scrap, return, rework, or downgrade disposition.
- `POST /v1/quality-management/quality-holds/{hold_id}:release`.
- Releases only when disposition and policy pass.
- `GET /v1/quality-management/quality-holds/{hold_id}`.
- Returns hold, inventory ref, dispositions, and policy trail.

### gRPC

- Service: `quality_management.hold.v1.QualityHoldService`.
- `rpc OpenHold(OpenHoldRequest) returns (HoldReceipt)`.
- `rpc AddDisposition(AddDispositionRequest) returns (HoldReceipt)`.
- `rpc ReleaseHold(ReleaseHoldRequest) returns (HoldReceipt)`.
- `rpc StreamHoldEvents(StreamHoldEventsRequest) returns (stream HoldEvent)`.

### AsyncAPI

- Channel: `quality-management.quality-hold.opened.v1`.
- Channel: `quality-management.quality-hold.released.v1`.
- Channel: `quality-management.quality-hold.dispositioned.v1`.
- Message: `QualityHoldOpened`.
- Message: `QualityHoldReleased`.
- Payload carries `hold_id`, `inventory_ref`, `quantity_on_hold`, `reason_code`, `disposition_type`, `audit_event_class`.
- Consumers: warehouse, production-planning, quality-notification, CAPA, ontology.

## Cedar Policy Hooks

- Policy: `quality_management::quality_hold::open`.
- Principal: `QualityEngineer`, `InspectionLotWorker`, or `SpcMonitorWorker`.
- Action: `quality_hold_open`.
- Resource: `InventoryLot::{material_id, batch_id, plant_code}`.
- Context: `tenant_id`, `reason_code`, `severity`, `source_event_id`, `pack_ids`.
- Policy: `quality_management::quality_hold::release`.
- Principal: `QualityManager`.
- Action: `quality_hold_release`.
- Resource: `QualityHold`.
- Context: `disposition_evidence`, `capa_effectiveness_state`, `authorized_plants`.
- Forbid: release without disposition.
- Forbid: release when CAPA effectiveness is required but unverified.
- Forbid: disposition quantity exceeds quantity on hold.
- Forbid: inventory ref tenant differs from hold tenant.

## Ontology Projection

- Vendor object: SAP QM inspection stock and blocked stock state.
- Oyatie object: `quality_management.quality_hold`.
- SAP stock category `Q` -> `InspectionBlock`.
- SAP blocked stock -> `DefectContainment`.
- SAP batch `CHARG` -> `batch_id`.
- SAP material `MATNR` -> `material_id`.
- SAP plant `WERKS` -> inventory plant field inside `inventory_ref`.
- SAP usage decision rejection -> `reason_code`.
- SAP return-to-vendor movement -> `ReturnToSupplier`.
- SAP scrap movement -> `Scrap`.
- SAP rework order reference -> `Rework`.
- Quality notification number -> `source_notification_id`.
- Inspection lot number -> `source_lot_id`.
- Projection freshness floor: 2 seconds.
- Projection rule: warehouse is source of inventory quantity after hold is posted.
- Projection mode: quality-management owns hold intent, warehouse owns stock movement.

## Workflow Steps

- Node `trigger-received`: lot fail, SPC violation, audit finding, or manual hold.
- Node `inventory-resolve`: warehouse inventory ref is resolved.
- Decision `inventory-not-found`: fail and notify source.
- Node `quantity-check`: quantity is positive and convertible.
- Decision `critical-defect`: set release requires CAPA.
- Node `cedar-open`: evaluate hold-open policy.
- Node `hold-open`: state `Open`.
- Node `warehouse-block`: request inventory block.
- Decision `warehouse-block-failed`: remain open and retry.
- Node `disposition-propose`: quality engineer records proposed disposition.
- Decision `return-supplier`: notify supplier quality.
- Decision `scrap`: require finance cost capture.
- Decision `rework`: notify production-planning.
- Node `release-readiness`: check disposition and CAPA.
- Node `cedar-release`: evaluate release policy.
- Node `release-post`: request warehouse movement.
- Node `audit-seal`: emit ADR-0263 class.
- Node `ontology-project`: publish hold state.
- Node `close`: hold cannot mutate except append audit evidence.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-QUALITY_HOLD-CHANGED`.
- `EVT-QUALITY_MANAGEMENT-QUALITY_HOLD-OPENED`.
- `EVT-QUALITY_MANAGEMENT-QUALITY_HOLD-DISPOSITIONED`.
- `EVT-QUALITY_MANAGEMENT-QUALITY_HOLD-RELEASED`.
- `EVT-QUALITY_MANAGEMENT-QUALITY_HOLD-IP_ACCEPTED`.
- ADR-0263 envelope stores `hold_type`.
- ADR-0263 envelope stores `inventory_ref`.
- ADR-0263 envelope stores `reason_code`.
- ADR-0263 envelope stores `disposition_type`.
- ADR-0263 envelope stores `warehouse_ack_id`.

## SLO Targets

- Hold open latency p50: 80 ms.
- Hold open latency p95: 300 ms.
- Hold open latency p99: 800 ms.
- Warehouse block ACK p95: 750 ms.
- Throughput: 120 hold opens per second per cell.
- Availability: 99.97 percent monthly for open-hold path.
- Rationale: containment is safety-critical and must outrun downstream stock movement.

## Failure Modes and Recovery

- Failure: warehouse inventory ref cannot be resolved.
- Recovery: `HOLD-INVENTORY-LOOKUP-RETRY` retries and blocks source workflow.
- Failure: warehouse block request fails after hold open.
- Recovery: `HOLD-BLOCK-RECONCILE` repeats warehouse block until ACK or manual incident.
- Failure: disposition quantity does not equal hold quantity.
- Recovery: `HOLD-DISPOSITION-BALANCE` blocks release and asks for split disposition.
- Failure: CAPA effectiveness is required but not verified.
- Recovery: `HOLD-CAPA-GATE` keeps state `PendingRelease`.
- Failure: release event reaches warehouse twice.
- Recovery: `HOLD-RELEASE-IDEMPOTENT` uses hold id as movement key.
- Failure: scrap disposition misses cost capture.
- Recovery: `HOLD-SCRAP-COST-REPLAY` sends cost event to finance ledger.

## Migration Notes

- Source vendor: SAP QM.
- Import inspection stock from lot and stock category history.
- Import blocked stock references from inventory movement history.
- Preserve SAP material document number as `source_movement_ref`.
- Source vendor: TIPQA maps nonconformance holds into `DefectContainment`.
- Source vendor: ETQ Reliance maps material review board holds into dispositions.
- Source vendor: MasterControl maps approved deviation releases into release evidence.
- Open vendor holds migrate as `Open`.
- Closed vendor holds migrate as immutable released or scrapped snapshots.
- Rollback path: preserve hold records and disable warehouse block dispatch.

## Cross-microservice Handoffs

- From inspection-lot: failed usage decision opens hold.
- From SPC monitor: control-chart violation opens hold.
- From quality-notification: defect containment opens hold.
- To warehouse: inventory block and movement requests.
- To production-planning: rework disposition request.
- To finance: scrap and failure cost event.
- To CAPA: release effectiveness gate.
- To ontology: quality hold projection.

## Verification

- Unit: release without disposition denied.
- Unit: CAPA-required hold cannot release until verified.
- Unit: disposition quantity cannot exceed hold quantity.
- Contract: REST release returns warehouse ACK placeholder.
- Contract: gRPC stream emits opened and released events.
- Event: hold opened event validates.
- Policy: Cedar denies cross-tenant inventory.
- Projection: SAP blocked stock fixture maps field-for-field.
- SLO: hold open p95 under 300 ms.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-QUALITY_HOLD-IP_ACCEPTED`.
