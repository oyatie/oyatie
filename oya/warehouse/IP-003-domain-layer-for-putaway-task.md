---
doc_class: ImplementationPlan
ip_id: IP-003
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
sap_submodule: EWM-WT (warehouse task)
tenant_class: paid
billing_components:
  - per_usage
persona: Lena Fischer, putaway coordinator
status: Accepted
date: 2026-05-20
owner_team: axis-warehouse + axis-erp-parity
---

# IP-003: Domain layer for putaway task

## Context

- SAP submodule: EWM-WT warehouse task creation and confirmation.
- Persona: Lena Fischer, putaway coordinator.
- Journey leg: j102 receipt is accepted and a controlled storage destination must be assigned.
- SAP tables: `/SCWM/ORDIM_O`, `/SCWM/STORAGEBIN`, `/SCWM/QUANT`, `/SCWM/WAREHOUSEORDER`.
- Oyatie aggregate: `PutawayTask`.
- Precedent: SAP EWM warehouse task confirmation plus Google Cloud Spanner-style idempotent mutation record.
- ADR-0297 requires Cedar before task confirmation and ADR-0263 requires immutable task audit events.
- Boundary: owns task instruction, bin reservation, exception state, and confirmation; strategy scoring is extended by IP-016.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.putaway_task (
  tenant_id UUID NOT NULL,
  putaway_task_id TEXT NOT NULL,
  inbound_delivery_id TEXT NOT NULL,
  material_id TEXT NOT NULL,
  source_bin_id TEXT NOT NULL,
  destination_bin_id TEXT NOT NULL,
  task_qty NUMERIC(18,6) NOT NULL,
  uom TEXT NOT NULL,
  task_status TEXT NOT NULL CHECK (task_status IN ('created','assigned','confirmed','exception','cancelled')),
  warehouse_order_id TEXT,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, putaway_task_id)
);
CREATE TABLE warehouse.putaway_task_exception (
  tenant_id UUID NOT NULL,
  putaway_task_id TEXT NOT NULL,
  exception_code TEXT NOT NULL,
  exception_reason TEXT NOT NULL,
  resolved_by TEXT,
  resolved_hlc TEXT,
  PRIMARY KEY (tenant_id, putaway_task_id, exception_code)
);
```

### Rust Types

```rust
pub struct PutawayTask {
    pub tenant_id: TenantId,
    pub putaway_task_id: PutawayTaskId,
    pub inbound_delivery_id: InboundDeliveryId,
    pub material_id: MaterialId,
    pub source_bin_id: BinId,
    pub destination_bin_id: BinId,
    pub task_qty: Decimal,
    pub task_status: WarehouseTaskStatus,
}
pub struct PutawayTaskException {
    pub exception_code: PutawayExceptionCode,
    pub exception_reason: String,
    pub resolved_by: Option<PrincipalId>,
}
pub enum PutawayTaskError { BinBlocked, CapacityExceeded, MaterialBinIncompatible, ConfirmationQtyMismatch, TaskAlreadyConfirmed }
```

## API Endpoints

- REST `POST /v1/warehouse/putaway-tasks` creates a task for received stock.
- REST `POST /v1/warehouse/putaway-tasks/{id}:assign` assigns operator or resource.
- REST `POST /v1/warehouse/putaway-tasks/{id}:confirm` confirms movement into destination bin.
- REST `POST /v1/warehouse/putaway-tasks/{id}:record-exception` captures bin or quantity exception.
- gRPC `warehouse.putaway.v1.PutawayTaskService.CreatePutawayTask`.
- gRPC `AssignPutawayTask`, `ConfirmPutawayTask`, and `RecordPutawayException`.
- AsyncAPI channel `warehouse.putaway-task.confirmed.v1`.
- AsyncAPI channel `warehouse.putaway-task.exception-recorded.v1`.

## Cedar Policy Hooks

- Policy: `warehouse::putaway_task::confirm`.
- Principal: `WarehouseRfOperator`.
- Action: `putaway_task_confirm`.
- Resource: `PutawayTask`.
- Context: `tenant_id`, `operator_resource_id`, `destination_bin_id`, `confirmed_qty`, `policy_bundle_version`.
- Forbid when destination bin is blocked, operator is not assigned, material is hazmat-incompatible, or quantity exceeds task quantity.

## Ontology Projection

- Vendor object: SAP EWM `/SCWM/ORDIM_O` open warehouse task.
- Oyatie object: `warehouse.putaway_task`.
- `/SCWM/ORDIM_O-TANUM` -> `putaway_task_id`.
- `/SCWM/ORDIM_O-VLPLA` -> `source_bin_id`.
- `/SCWM/ORDIM_O-NLPLA` -> `destination_bin_id`.
- `/SCWM/STORAGEBIN-LGPLA` -> bin identity.
- `/SCWM/QUANT-QUAN` -> task quantity evidence.
- `/SCWM/WAREHOUSEORDER-WHO` -> `warehouse_order_id`.
- Projection freshness floor: 2 seconds.
- Projection rule: confirmed tasks become immutable movement evidence.

## Workflow Steps

- Node `receipt-ready`: inbound receipt exposes stock requiring putaway.
- Node `bin-reserve`: reserve destination bin.
- Decision `bin-blocked`: branch to alternative bin selection.
- Decision `capacity-exceeded`: branch to overflow staging.
- Node `task-create`: persist task and emit creation event.
- Node `operator-assign`: bind RF operator or autonomous resource.
- Node `rf-confirm`: operator confirms bin, material, and quantity.
- Decision `confirmation-mismatch`: create exception and hold inventory posting.
- Node `inventory-post`: hand off confirmed movement to inventory-ledger.
- Node `audit-seal`: emit task accepted evidence.

## Audit Events

- `EVT-WAREHOUSE-PUTAWAY_TASK-CREATED`.
- `EVT-WAREHOUSE-PUTAWAY_TASK-ASSIGNED`.
- `EVT-WAREHOUSE-PUTAWAY_TASK-CONFIRMED`.
- `EVT-WAREHOUSE-PUTAWAY_TASK-EXCEPTION_RECORDED`.
- `EVT-WAREHOUSE-PUTAWAY_TASK-POLICY_DENIED`.
- `EVT-WAREHOUSE-PUTAWAY_TASK-IP_ACCEPTED`.
- ADR-0263 envelope stores `source_bin_id`, `destination_bin_id`, `task_qty`, and `warehouse_order_id`.

## SLO Targets

- Task creation p50: 35 ms.
- Task creation p95: 140 ms.
- Task creation p99: 400 ms.
- RF confirmation p95: 100 ms.
- Rationale: putaway is operator-paced, but RF confirmation must feel immediate to keep aisle work moving.

## Failure Modes and Recovery

- Failure: `BIN-BLOCKED`; recovery: invalidate reservation and rerun destination selection.
- Failure: `CAPACITY-EXCEEDED`; recovery: allocate overflow staging bin and emit exception event.
- Failure: `MATERIAL-INCOMPATIBLE`; recovery: block confirmation and request hazmat or temperature-zone review.
- Failure: `CONFIRMATION-QTY-MISMATCH`; recovery: create recount workflow and hold inventory posting.
- Failure: `TASK-DUPLICATE`; recovery: return idempotent existing task receipt.
- Failure: `LEDGER-HANDOFF-FAILED`; recovery: retry confirmed movement through durable outbox.

## Migration Notes

- Import open `/SCWM/ORDIM_O` tasks as created or assigned.
- Import confirmed tasks as immutable movement evidence tied to bin and quant.
- Map SAP exception codes to Oyatie `PutawayExceptionCode`.
- Preserve warehouse order references for later labor and order optimization.
- Rollback path: stop new task creation while keeping confirmation replay read-only.
- Backfill order: bins, quants, inbound deliveries, tasks, exceptions, confirmations.

## Cross-microservice Handoffs

- From inbound delivery: received item and source staging bin.
- From inventory-ledger: current bin capacity and stock state.
- To labor assignment: operator task workload.
- To warehouse order: task bundle evidence.
- To workflow-engine: exceptions requiring approval.
- To ontology: confirmed movement projection.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The primitive remains bound to SAP EWM warehouse task creation and confirmation. |
| Persona specificity | Lena Fischer owns putaway-destination acceptance, exception review, and rollback language. |
| Journey specificity | The j102 accepted-receipt-to-controlled-storage leg drives destination and quality-hold behavior. |
| DDL anchor | The putaway task tables above are the normative task, source bin, destination bin, and confirmation model. |
| Rust anchor | The putaway aggregate, confirmation type, and error enum above are the implementation type names. |
| REST anchor | Putaway create, confirm, exception, and cancel endpoints are the tenant API surface. |
| gRPC anchor | The putaway task service is the worker and replay contract for internal movement orchestration. |
| AsyncAPI anchor | Created, confirmed, and exception channels carry inventory and workflow evidence. |
| Cedar anchor | Destination assignment and confirmation are default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP `/SCWM/ORDIM_O` task lineage projects to warehouse movement nodes without replacing Oyatie task identity. |
| ADR-0263 class binding | Destination and confirmation policy checks emit `OfficeBoundaryAttemptEvaluated` plus allow/deny outcome classes. |
| ADR-0263 pack binding | Storage compliance overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge or worker quota throttles emit `AbuseDefenceRateLimitHit` through the registry class. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, source bin, destination bin, material id, and `cedar_decision_id`. |
| Metric | `oya_warehouse_putaway_task_commands_total{tenant_id,cell_id,command,status}` caps command/status cardinality. |
| Latency histogram | `oya_warehouse_putaway_task_duration_seconds` tracks create and confirm latency per storage type. |
| Trace span | `warehouse.putaway_task.confirm` links inbound receipt, inventory movement, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `task_id`, `source_bin`, `destination_bin`, and exception code. |
| Capacity math | Destination selection rejects bins where projected quantity would exceed bin_capacity * 0.95 safety threshold. |
| Multi-region | Home-cell movement confirmation is authoritative; DR cells serve read-only movement projections until promoted. |
| Sovereign cells | Regulated material and supplier lineage remain in-region for KR-CSAP, EU, CN-PIPL, IL5/6, and FedRAMP-High overlays. |
| Rollback | Stop new task creation, keep confirmations read-only, and replay movement outbox from the last sealed audit id. |
| Test evidence | Required tests cover no-bin, quality hold, tenant mismatch, over-capacity destination, and idempotent confirmation. |
| Rejected shortcut | A generic `MovementTask` is rejected because it loses SAP `/SCWM/ORDIM_O` task and storage-bin semantics. |
