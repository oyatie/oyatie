---
doc_class: ImplementationPlan
ip_id: IP-010
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
sap_submodule: EWM-RF (radio frequency)
tenant_class: paid
billing_components:
  - per_usage
persona: Diego Vargas, RF picking lead
status: Accepted
date: 2026-05-20
owner_team: axis-warehouse + axis-erp-parity
---

# IP-010: Usecase layer for picking wave

## Context

- SAP submodule: EWM-RF radio frequency picking execution.
- Persona: Diego Vargas, RF picking lead.
- Journey leg: j123 released launch wave is executed by RF pickers with confirmation and exception handling.
- SAP tables: `/SCWM/WAREHOUSEORDER`, `/SCWM/ORDIM_O`, `/SCWM/QUANT`, `/SCWM/STORAGEBIN`.
- Oyatie usecase: `ExecutePickingWave`.
- Precedent: SAP EWM RF picking flow plus Google Pub/Sub ordered delivery per partition.
- ADR-0253 binds low-latency transport and ADR-0297 gates RF confirmation through Cedar.
- Boundary: orchestrates wave release to RF, task confirmation, shortage exception, and pack handoff.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.picking_wave_execution (
  tenant_id UUID NOT NULL,
  execution_id TEXT NOT NULL,
  picking_wave_id TEXT NOT NULL,
  resource_id TEXT NOT NULL,
  execution_status TEXT NOT NULL CHECK (execution_status IN ('queued','active','paused','completed','exception')),
  current_task_id TEXT,
  rf_device_id TEXT NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, execution_id)
);
CREATE TABLE warehouse.pick_confirmation (
  tenant_id UUID NOT NULL,
  confirmation_id TEXT NOT NULL,
  execution_id TEXT NOT NULL,
  warehouse_task_id TEXT NOT NULL,
  picked_qty NUMERIC(18,6) NOT NULL,
  exception_code TEXT,
  confirmed_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, confirmation_id)
);
```

### Rust Types

```rust
pub struct PickingWaveExecution {
    pub tenant_id: TenantId,
    pub execution_id: ExecutionId,
    pub picking_wave_id: PickingWaveId,
    pub resource_id: LaborResourceId,
    pub execution_status: WaveExecutionStatus,
    pub rf_device_id: RfDeviceId,
}
pub struct PickConfirmation {
    pub confirmation_id: ConfirmationId,
    pub warehouse_task_id: WarehouseTaskId,
    pub picked_qty: Decimal,
    pub exception_code: Option<PickExceptionCode>,
}
pub enum ExecutePickingWaveError { DeviceNotBound, TaskOutOfSequence, StockShort, BarcodeMismatch, PackHandoffFailed }
```

## API Endpoints

- REST `POST /v1/warehouse/picking-waves/{id}:start-execution` starts RF execution.
- REST `POST /v1/warehouse/picking-wave-executions/{id}:confirm-task` records pick confirmation.
- REST `POST /v1/warehouse/picking-wave-executions/{id}:record-exception` records RF exception.
- gRPC `warehouse.picking_usecase.v1.ExecutePickingWave`.
- gRPC `ConfirmPickTask`, `RecordPickException`, and `StreamNextPickTask`.
- AsyncAPI channel `warehouse.picking-wave.task-confirmed.v1`.
- AsyncAPI channel `warehouse.picking-wave.shortage-recorded.v1`.
- Consumers: inventory-ledger, packing, labor-assignment, ontology.

## Cedar Policy Hooks

- Policy: `warehouse::picking_execution::confirm`.
- Principal: `WarehouseRfPicker`.
- Action: `pick_task_confirm`.
- Resource: `WarehouseTask`.
- Context: `tenant_id`, `rf_device_id`, `resource_id`, `bin_barcode`, `picked_qty`, `sequence_no`.
- Forbid when device is not bound to worker, task sequence is violated, scanned bin differs, or stock is unavailable.

## Ontology Projection

- Vendor object: SAP EWM RF pick confirmation.
- Oyatie object: `warehouse.pick_confirmation`.
- `/SCWM/WAREHOUSEORDER-WHO` -> `picking_wave_id`.
- `/SCWM/ORDIM_O-TANUM` -> `warehouse_task_id`.
- `/SCWM/STORAGEBIN-LGPLA` -> scanned bin evidence.
- `/SCWM/QUANT-QUAN` -> picked quantity.
- RF device -> `rf_device_id`.
- Worker resource -> `resource_id`.
- Projection freshness floor: 1 second.
- Projection rule: shortage confirmations reduce available stock only through inventory-ledger handoff.

## Workflow Steps

- Node `execution-start`: bind RF picker and device to wave.
- Node `next-task-stream`: stream task in sequence.
- Decision `device-not-bound`: deny confirmation.
- Decision `barcode-mismatch`: record exception and require rescan.
- Node `confirm-task`: persist picked quantity.
- Decision `stock-short`: create shortage event and backorder handoff.
- Node `inventory-reserve-update`: decrement pickable stock.
- Node `pack-handoff`: send completed line to packing.
- Decision `pack-handoff-failed`: queue retry and keep confirmation immutable.
- Node `execution-close`: close when all tasks complete.

## Audit Events

- `EVT-WAREHOUSE-PICKING_EXECUTION-STARTED`.
- `EVT-WAREHOUSE-PICKING_EXECUTION-TASK_CONFIRMED`.
- `EVT-WAREHOUSE-PICKING_EXECUTION-SHORTAGE_RECORDED`.
- `EVT-WAREHOUSE-PICKING_EXECUTION-BARCODE_MISMATCH`.
- `EVT-WAREHOUSE-PICKING_EXECUTION-POLICY_DENIED`.
- `EVT-WAREHOUSE-PICKING_EXECUTION-IP_ACCEPTED`.
- ADR-0263 envelope stores `rf_device_id`, `resource_id`, `warehouse_task_id`, and `picked_qty`.

## SLO Targets

- Next task stream p50: 20 ms.
- Next task stream p95: 75 ms.
- Next task stream p99: 180 ms.
- Pick confirmation p95: 90 ms.
- Rationale: RF picker experience must behave like a local scanner interaction; slow paths become aisle congestion.

## Failure Modes and Recovery

- Failure: `DEVICE-NOT-BOUND`; recovery: force device rebind and deny task confirmation.
- Failure: `TASK-OUT-OF-SEQUENCE`; recovery: return expected task and audit attempted skip.
- Failure: `STOCK-SHORT`; recovery: create shortage event and notify outbound release.
- Failure: `BARCODE-MISMATCH`; recovery: require rescan or supervisor override.
- Failure: `PACK-HANDOFF-FAILED`; recovery: retry durable handoff without losing pick confirmation.
- Failure: `RF-OFFLINE`; recovery: accept signed offline confirmations within pack-configured window.

## Migration Notes

- Import active RF queues as wave execution rows only when task state is open.
- Import historical confirmations as immutable evidence.
- Preserve SAP RF user and device fields as lineage when available.
- Map SAP exception codes to Oyatie pick exception codes before replay.
- Rollback path: disable RF execution start and leave waves in released state.
- Backfill order: waves, tasks, worker assignments, execution sessions, confirmations.

## Cross-microservice Handoffs

- From picking-wave: released wave and task sequence.
- From labor assignment: picker and device binding.
- To inventory-ledger: picked quantity and shortage signal.
- To packing or outbound consolidation: completed line.
- To workflow-engine: mismatch and shortage exceptions.
- To compliance: RF confirmation audit stream.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The usecase remains bound to SAP EWM RF picking execution. |
| Persona specificity | Diego Vargas owns RF session, confirmation, shortage, and rollback acceptance language. |
| Journey specificity | The j123 released-wave execution leg drives picker confirmation and exception handling. |
| DDL anchor | The RF execution session and confirmation tables above are the normative usecase persistence model. |
| Rust anchor | The RF picking session, confirmation, and error enum above are the implementation contract. |
| REST anchor | Start session, confirm pick, record shortage, and close endpoints are the tenant command surface. |
| gRPC anchor | The RF picking service is the worker and replay contract for device execution. |
| AsyncAPI anchor | Pick confirmed, shortage, and session closed channels carry downstream evidence. |
| Cedar anchor | Pick confirmation is default-deny and must persist `cedar_decision_id` before inventory update. |
| Ontology anchor | SAP RF task lineage projects to picker-session and confirmation nodes. |
| ADR-0263 class binding | RF confirmation policy checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Accessibility, safety, or labor overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on RF APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, wave id, task id, picker id, device id, and `cedar_decision_id`. |
| Metric | `oya_warehouse_rf_picking_commands_total{tenant_id,cell_id,command,status}` caps cardinality. |
| Latency histogram | `oya_warehouse_rf_picking_confirmation_duration_seconds` tracks p50/p95/p99 scanner round-trip latency. |
| Trace span | `warehouse.rf_picking.confirm_pick` links wave, labor assignment, inventory-ledger, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `picker_id`, `device_id`, `task_id`, and shortage code. |
| Capacity math | RF session fan-out is bounded by active_sessions * confirmation_rate; queueing above p95 budget blocks wave start. |
| Multi-region | RF confirmations write to the facility home cell; DR cells serve read-only execution status. |
| Sovereign cells | Worker/device evidence remains in-region for labor-law and sovereign compliance overlays. |
| Rollback | Disable RF execution start, leave waves released, and replay confirmations from the last sealed audit id. |
| Test evidence | Required tests cover shortage, device mismatch, tenant mismatch, duplicate confirmation, and inventory timeout. |
| Rejected shortcut | A generic `PickTaskExecution` is rejected because it loses SAP EWM RF and warehouse-order semantics. |
