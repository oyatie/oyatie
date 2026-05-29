---
doc_class: ImplementationPlan
ip_id: IP-002
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
journey_ref: j101-multi-tier-supply-chain-formation
sap_submodule: QM-IM Inspection Management
tenant_class: paid
billing_components:
  - per_usage
persona: Mateo Ruiz, receiving quality technician
status: Accepted
date: 2026-05-20
owner_team: axis-quality-management + axis-erp-parity
---

# IP-002: Domain layer for inspection-lot creation rules

## Context

- SAP QM submodule: QM-IM Inspection Management.
- Topic: lot creation rules.
- Persona: Mateo Ruiz, receiving quality technician.
- Journey: j101 multi-tier supply-chain formation.
- Journey leg: goods receipt creates an inspection obligation.
- SAP precedent: inspection lot creation from goods movement and production order release.
- Oyatie aggregate: `QualityInspectionLot`.
- Boundary: domain validation and state transitions only.
- ADR-0105 keeps the aggregate in the domain ring.
- ADR-0131 keeps the implementation plan with the microservice.
- ADR-0244 requires tenant and principal continuity.
- ADR-0263 defines audit event envelopes.
- ADR-0297 makes Cedar decisions first-class evidence.
- ADR-0314 prevents settlement mutation during supplier inspection.
- ADR-0315 requires SAP QM parity.
- ADR-0329/0330/0331 requires implementation-ready ERP detail.
- Lot creation must be deterministic under duplicate goods-receipt events.
- Lot creation must not silently skip regulated materials.
- Lot creation must preserve origin because origin drives sampling and usage decision.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.inspection_lot (
  tenant_id UUID NOT NULL,
  inspection_lot_id TEXT NOT NULL,
  lot_origin TEXT NOT NULL,
  material_id TEXT NOT NULL,
  plant_code TEXT NOT NULL,
  vendor_id TEXT,
  production_order_id TEXT,
  warehouse_receipt_id TEXT,
  inspection_plan_id TEXT NOT NULL,
  plan_revision_no INTEGER NOT NULL,
  lot_quantity NUMERIC(20,6) NOT NULL,
  uom TEXT NOT NULL,
  state TEXT NOT NULL,
  creation_rule_id TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, inspection_lot_id),
  UNIQUE (tenant_id, idempotency_key)
);
CREATE INDEX inspection_lot_origin_idx
  ON quality_management.inspection_lot (tenant_id, lot_origin, material_id, state);
CREATE TABLE quality_management.inspection_lot_sample (
  tenant_id UUID NOT NULL,
  inspection_lot_id TEXT NOT NULL,
  sample_no INTEGER NOT NULL,
  sample_size NUMERIC(20,6) NOT NULL,
  sample_uom TEXT NOT NULL,
  sample_state TEXT NOT NULL,
  assigned_to_principal_id TEXT,
  PRIMARY KEY (tenant_id, inspection_lot_id, sample_no)
);
```

### Rust Types

```rust
pub struct QualityInspectionLot {
    pub tenant_id: TenantId,
    pub inspection_lot_id: InspectionLotId,
    pub lot_origin: LotOrigin,
    pub material_id: MaterialId,
    pub plant_code: PlantCode,
    pub vendor_id: Option<VendorId>,
    pub production_order_id: Option<ProductionOrderId>,
    pub warehouse_receipt_id: Option<WarehouseReceiptId>,
    pub inspection_plan_ref: InspectionPlanRef,
    pub lot_quantity: Decimal,
    pub uom: UnitOfMeasure,
    pub state: InspectionLotState,
    pub samples: Vec<InspectionLotSample>,
}
pub enum LotOrigin { GoodsReceipt, ProductionRelease, CustomerReturn, StockTransfer, Manual }
pub enum InspectionLotState { Created, Sampled, ResultsRecorded, UsageDecisionPending, Accepted, Rejected, Cancelled }
pub struct LotCreationRule {
    pub rule_id: RuleId,
    pub allowed_origins: Vec<LotOrigin>,
    pub requires_plan: bool,
    pub skip_allowed_for_low_risk: bool,
}
pub enum LotCreationError {
    DuplicateIdempotencyKey,
    MissingReleasedPlan,
    OriginNotAllowed,
    QuantityOutOfRange,
    CrossTenantSource,
    RegulatedSkipDenied,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/inspection-lots`.
- Creates a lot from a domain source event.
- Requires `lot_origin`, `material_id`, `plant_code`, quantity, and idempotency key.
- Supports source references from warehouse, production-planning, and returns.
- Response includes `inspection_lot_id`, selected plan, sample roster, and state.
- `POST /v1/quality-management/inspection-lots/{inspection_lot_id}:cancel`.
- Cancels only if no result is recorded.
- `GET /v1/quality-management/inspection-lots/{inspection_lot_id}`.
- Returns lot, samples, source references, and policy trail.

### gRPC

- Service: `quality_management.inspection_lot.v1.InspectionLotService`.
- `rpc CreateInspectionLot(CreateInspectionLotRequest) returns (InspectionLotReceipt)`.
- `rpc CancelInspectionLot(CancelInspectionLotRequest) returns (InspectionLotReceipt)`.
- `rpc GetInspectionLot(GetInspectionLotRequest) returns (InspectionLotView)`.
- `rpc StreamLotState(StreamLotStateRequest) returns (stream InspectionLotEvent)`.

### AsyncAPI

- Channel: `quality-management.inspection-lot.created.v1`.
- Channel: `quality-management.inspection-lot.state-changed.v1`.
- Message: `InspectionLotCreated`.
- Message: `InspectionLotStateChanged`.
- Payload carries `tenant_id`, `lot_origin`, `source_ref`, `inspection_plan_id`, `sample_count`, `audit_event_class`.
- Dead letter: `dlq.quality-management.inspection-lot.created.v1`.

## Cedar Policy Hooks

- Policy: `quality_management::inspection_lot::create_from_goods_receipt`.
- Principal: `WarehouseReceiptWorker`.
- Action: `inspection_lot_create`.
- Resource: `InspectionLotCreation::{material_id, plant_code, lot_origin}`.
- Context: `tenant_id`, `receipt_id`, `vendor_risk_tier`, `regulatory_pack`, `plan_selection_decision_id`.
- Policy: `quality_management::inspection_lot::create_from_production_release`.
- Principal: `ProductionPlanningWorker`.
- Action: `inspection_lot_create`.
- Resource: `ProductionInspectionObligation`.
- Context: `production_order_id`, `material_criticality`, `authorized_plants`.
- Forbid: lot origin not in rule allowed origins.
- Forbid: source tenant differs from context tenant.
- Forbid: regulated material without released inspection plan.
- Permit: low-risk skip only when pack allows `inspection_skip`.

## Ontology Projection

- Vendor object: SAP QM `QALS` inspection lot.
- Oyatie object: `quality_management.inspection_lot`.
- `QALS-PRUEFLOS` -> `inspection_lot_id`.
- `QALS-HERKUNFT` -> `lot_origin`.
- `QALS-MATNR` -> `material_id`.
- `QALS-WERK` -> `plant_code`.
- `QALS-LIFNR` -> `vendor_id`.
- `QALS-AUFNR` -> `production_order_id`.
- `QALS-LOSMENGE` -> `lot_quantity`.
- `QALS-MENGENEINH` -> `uom`.
- `QALS-STAT01` -> `state`.
- `QALS-PLNNR` -> `inspection_plan_id`.
- `QALS-PLNAL` -> `plan_revision_no`.
- SAP sample rows -> `inspection_lot_sample`.
- SAP origin text -> `source_origin_label`.
- Projection freshness floor: 3 seconds.
- Projection consumers: warehouse, production-planning, compliance, ontology.

## Workflow Steps

- Node `source-event-received`: goods receipt or production release arrives.
- Node `idempotency-check`: duplicate source event returns existing lot.
- Node `origin-rule-load`: rule is found by origin and material class.
- Decision `origin-not-allowed`: reject and emit deny event.
- Node `plan-select`: call IP-001 selector.
- Decision `no-plan`: quarantine unless low-risk skip is permitted.
- Node `sample-scheme-derive`: compute sample count from plan.
- Decision `destructive-test`: allocate quarantine stock first.
- Decision `skip-permitted`: mark `Accepted` with skip evidence.
- Node `lot-create`: instantiate aggregate.
- Node `sample-create`: attach sample rows.
- Node `state-created`: lot enters `Created`.
- Node `worker-dispatch`: assign technician queue.
- Node `audit-seal`: emit creation audit event.
- Node `ontology-project`: publish lot read model.
- Node `notify-source`: ACK warehouse or production-planning.
- Decision `source-timeout`: retry with idempotency key.
- Node `close`: lot is visible to result recording.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-INSPECTION_LOT-CHANGED`.
- `EVT-QUALITY_MANAGEMENT-INSPECTION_LOT-CREATED`.
- `EVT-QUALITY_MANAGEMENT-INSPECTION_LOT-CANCELLED`.
- `EVT-QUALITY_MANAGEMENT-INSPECTION_LOT-POLICY_DENIED`.
- `EVT-QUALITY_MANAGEMENT-INSPECTION_LOT-IP_ACCEPTED`.
- ADR-0263 envelope stores `lot_origin`.
- ADR-0263 envelope stores `idempotency_key`.
- ADR-0263 envelope stores `source_ref`.
- ADR-0263 envelope stores `cedar_decision_id`.
- ADR-0263 envelope stores `sample_count`.

## SLO Targets

- Lot create latency p50: 60 ms.
- Lot create latency p95: 240 ms.
- Lot create latency p99: 650 ms.
- Source ACK p95: 500 ms.
- Throughput: 250 created lots per second per cell.
- Availability: 99.95 percent monthly.
- Rationale: warehouse receiving blocks putaway until lot creation ACK is returned.

## Failure Modes and Recovery

- Failure: duplicate warehouse event creates a repeated command.
- Recovery: `LOT-IDEMPOTENT-RETURN` returns existing lot and emits duplicate evidence.
- Failure: material is regulated and no released plan exists.
- Recovery: `LOT-PLAN-GAP-HOLD` creates quality hold and planner task.
- Failure: source event has quantity zero after unit conversion.
- Recovery: `LOT-QUANTITY-REJECT` rejects command and notifies source service.
- Failure: sample creation fails after lot row insert.
- Recovery: `LOT-SAMPLE-RECONCILE` rebuilds samples from outbox event.
- Failure: Cedar denies production release because plant scope is missing.
- Recovery: `LOT-PLANT-AUTH-RETRY` routes to identity and production supervisor.
- Failure: source ACK cannot reach warehouse.
- Recovery: `LOT-SOURCE-ACK-REPLAY` retries from outbox with same receipt id.

## Migration Notes

- Source vendor: SAP QM.
- Primary import object: `QALS`.
- Sample import object: `QASR`.
- Preserve `QALS-HERKUNFT` as lot origin.
- Preserve `QALS-ART` as inspection type when present.
- Source vendor: TIPQA maps receiving inspection lots into `GoodsReceipt`.
- Source vendor: IQS-AQM maps incoming inspection records into lot plus sample rows.
- Import staged rows must include source lot number for traceability.
- Cancelled SAP lots migrate as `Cancelled`, never deleted.
- Rollback path: disable source-event subscription and retain staged records.

## Cross-microservice Handoffs

- From warehouse: receipt event and inventory quarantine reference.
- From production-planning: production order release and material lot obligation.
- To inspection-plan: plan selection query.
- To quality-hold: containment when plan gap or reject occurs.
- To compliance: regulated inspection obligation evidence.
- To ontology: lot state projection.
- To workflow-engine: technician task routing.
- To marketplace: supplier quality visibility, no settlement ownership.

## Verification

- Unit: duplicate idempotency key returns existing lot.
- Unit: regulated skip denied.
- Unit: low-risk skip accepted only with pack flag.
- Unit: origin not allowed fails closed.
- Contract: REST create returns sample roster.
- Contract: gRPC stream emits state changes.
- Event: AsyncAPI created event validates.
- Policy: Cedar source tenant mismatch denied.
- Projection: SAP `QALS` fixture maps field-for-field.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-INSPECTION_LOT-IP_ACCEPTED`.
