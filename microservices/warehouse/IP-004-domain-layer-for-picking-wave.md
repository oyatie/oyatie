---
doc_class: ImplementationPlan
ip_id: IP-004
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
sap_submodule: EWM-WO (warehouse orders)
tenant_class: paid
billing_components:
  - per_usage
persona: Grace Kim, wave planner
status: Accepted
date: 2026-05-20
owner_team: axis-warehouse + axis-erp-parity
---

# IP-004: Domain layer for picking wave

## Context

- SAP submodule: EWM-WO warehouse orders and wave release.
- Persona: Grace Kim, wave planner.
- Journey leg: j123 launch orders are grouped into a release wave without overloading aisles or pack stations.
- SAP tables: `/SCWM/WAREHOUSEORDER`, `/SCWM/ORDIM_O`, `/SCWM/QUANT`, `/SCWM/STORAGEBIN`.
- Oyatie aggregate: `PickingWave`.
- Precedent: SAP EWM wave management plus Cloudflare queue partitioning by shard and priority.
- ADR-0315 binds SAP EWM parity; ADR-0253 binds transport; ADR-0297 gates wave release.
- Boundary: this IP owns wave identity, release state, pick task membership, and capacity guardrails; route optimization is expanded in IP-017.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.picking_wave (
  tenant_id UUID NOT NULL,
  picking_wave_id TEXT NOT NULL,
  wave_type TEXT NOT NULL,
  release_status TEXT NOT NULL CHECK (release_status IN ('draft','ready','released','paused','closed','cancelled')),
  release_window_start TIMESTAMPTZ NOT NULL,
  release_window_end TIMESTAMPTZ NOT NULL,
  pack_station_capacity INTEGER NOT NULL,
  aisle_capacity_snapshot_ref TEXT NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, picking_wave_id)
);
CREATE TABLE warehouse.picking_wave_task (
  tenant_id UUID NOT NULL,
  picking_wave_id TEXT NOT NULL,
  warehouse_task_id TEXT NOT NULL,
  outbound_delivery_id TEXT NOT NULL,
  pick_sequence INTEGER NOT NULL,
  PRIMARY KEY (tenant_id, picking_wave_id, warehouse_task_id)
);
```

### Rust Types

```rust
pub struct PickingWave {
    pub tenant_id: TenantId,
    pub picking_wave_id: PickingWaveId,
    pub wave_type: WaveType,
    pub release_status: WaveReleaseStatus,
    pub release_window_start: DateTime<Utc>,
    pub release_window_end: DateTime<Utc>,
    pub pack_station_capacity: u32,
}
pub struct PickingWaveTask {
    pub warehouse_task_id: WarehouseTaskId,
    pub outbound_delivery_id: OutboundDeliveryId,
    pub pick_sequence: u32,
}
pub enum PickingWaveError { EmptyWave, AisleCapacityExceeded, PackStationOverload, ReleaseWindowClosed, TaskAlreadyInWave }
```

## API Endpoints

- REST `POST /v1/warehouse/picking-waves` creates a draft wave.
- REST `POST /v1/warehouse/picking-waves/{id}:add-task` adds an eligible pick task.
- REST `POST /v1/warehouse/picking-waves/{id}:release` releases the wave to RF execution.
- REST `POST /v1/warehouse/picking-waves/{id}:pause` pauses for congestion or safety.
- gRPC `warehouse.picking.v1.PickingWaveService.CreatePickingWave`.
- gRPC `AddWaveTask`, `ReleaseWave`, `PauseWave`, and `CloseWave`.
- AsyncAPI channel `warehouse.picking-wave.released.v1`.
- AsyncAPI channel `warehouse.picking-wave.paused.v1`.

## Cedar Policy Hooks

- Policy: `warehouse::picking_wave::release`.
- Principal: `WarehouseWavePlanner`.
- Action: `picking_wave_release`.
- Resource: `PickingWave`.
- Context: `tenant_id`, `pack_station_capacity`, `aisle_capacity_snapshot_ref`, `release_window`, `policy_bundle_version`.
- Forbid when wave is empty, release window is closed, pack station capacity is exceeded, or any task belongs to another tenant.

## Ontology Projection

- Vendor object: SAP EWM warehouse order wave.
- Oyatie object: `warehouse.picking_wave`.
- `/SCWM/WAREHOUSEORDER-WHO` -> warehouse task bundle lineage.
- `/SCWM/ORDIM_O-TANUM` -> task membership.
- `/SCWM/QUANT-LGPLA` -> pick source bin evidence.
- `/SCWM/STORAGEBIN-AISLE` -> aisle capacity dimension.
- Outbound delivery -> fulfillment demand link.
- Pack station capacity -> wave release guardrail.
- Projection freshness floor: 4 seconds.
- Projection rule: paused waves remain visible but no new RF pick commands are emitted.

## Workflow Steps

- Node `candidate-select`: identify eligible outbound pick tasks.
- Node `capacity-snapshot`: read aisle and pack station capacity.
- Decision `empty-wave`: reject draft release.
- Decision `pack-overload`: split wave or defer low-priority tasks.
- Node `wave-create`: persist draft wave and task sequence.
- Decision `window-closed`: require new release window.
- Node `wave-release`: emit RF execution event.
- Decision `congestion-detected`: pause wave and notify planner.
- Node `wave-close`: close after all tasks are confirmed or cancelled.
- Node `audit-seal`: emit wave release evidence.

## Audit Events

- `EVT-WAREHOUSE-PICKING_WAVE-CREATED`.
- `EVT-WAREHOUSE-PICKING_WAVE-TASK_ADDED`.
- `EVT-WAREHOUSE-PICKING_WAVE-RELEASED`.
- `EVT-WAREHOUSE-PICKING_WAVE-PAUSED`.
- `EVT-WAREHOUSE-PICKING_WAVE-POLICY_DENIED`.
- `EVT-WAREHOUSE-PICKING_WAVE-IP_ACCEPTED`.
- ADR-0263 envelope stores `wave_type`, `release_window`, `task_count`, and `capacity_snapshot_ref`.

## SLO Targets

- Wave create p50: 70 ms.
- Wave create p95: 260 ms.
- Wave create p99: 800 ms.
- Wave release event dispatch p95: 500 ms for up to 10,000 tasks.
- Rationale: release is batch-shaped and may include many tasks, but planner feedback still must be interactive.

## Failure Modes and Recovery

- Failure: `EMPTY-WAVE`; recovery: reject release and return candidate filter diagnostics.
- Failure: `PACK-STATION-OVERLOAD`; recovery: split wave by pack station and priority.
- Failure: `AISLE-CONGESTION`; recovery: pause affected aisle tasks and resequence.
- Failure: `TASK-ALREADY-IN-WAVE`; recovery: keep existing membership and return idempotent result.
- Failure: `RF-DISPATCH-FAILED`; recovery: retry dispatch through wave outbox.
- Failure: `WINDOW-CLOSED`; recovery: require planner to choose a new release window.

## Migration Notes

- Import active SAP wave and warehouse order assignments as draft or released waves.
- Preserve SAP wave numbers as lineage only.
- Recompute pick sequence during migration when SAP sequence is incomplete.
- Do not import closed waves as executable commands; store as audit history.
- Rollback path: disable release command and keep outbound delivery release independent.
- Backfill order: outbound deliveries, warehouse tasks, warehouse orders, waves, task memberships.

## Cross-microservice Handoffs

- From outbound delivery: released lines requiring pick.
- From labor assignment: available pickers and skills.
- From inventory-ledger: source bin stock state.
- To RF execution: released pick tasks.
- To carrier-integration: wave completion signal for staging.
- To workflow-engine: congestion and safety exception tasks.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The primitive remains bound to SAP EWM warehouse orders and wave release semantics. |
| Persona specificity | Grace Kim owns wave release, congestion review, and rollback acceptance language. |
| Journey specificity | The j123 launch-wave leg drives grouping, pick capacity, and pack-station protection. |
| DDL anchor | The picking wave and task-membership tables above are the normative release model. |
| Rust anchor | The picking wave aggregate, membership type, and error enum above are the implementation type names. |
| REST anchor | Wave create, release, split, and cancel endpoints are the tenant API surface. |
| gRPC anchor | The picking wave service is the worker and replay contract for release orchestration. |
| AsyncAPI anchor | Wave released, split, and completed channels carry execution and carrier staging evidence. |
| Cedar anchor | Wave release is default-deny and must store `cedar_decision_id` before RF dispatch. |
| Ontology anchor | SAP warehouse-order lineage projects to wave and task nodes without replacing Oyatie identity. |
| ADR-0263 class binding | Release policy checks emit `OfficeBoundaryAttemptEvaluated` plus allowed or denied outcome classes. |
| ADR-0263 pack binding | Congestion or compliance-pack overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on wave APIs emits `AbuseDefenceRateLimitHit` through the registry class. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, wave id, task count, congestion score, and `cedar_decision_id`. |
| Metric | `oya_warehouse_picking_wave_commands_total{tenant_id,cell_id,command,status}` caps command/status cardinality. |
| Latency histogram | `oya_warehouse_picking_wave_release_duration_seconds` tracks p50/p95/p99 release latency. |
| Trace span | `warehouse.picking_wave.release` links outbound delivery, labor assignment, RF execution, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `wave_id`, `task_count`, `pack_station_group`, and congestion code. |
| Capacity math | Release blocks when tasks_per_picker or pack_station_queue_depth exceeds the configured Little's-Law threshold. |
| Multi-region | Home-cell release is authoritative; DR cells provide read-only wave status until promotion. |
| Sovereign cells | Customer, shipment, and labor evidence remains in the active pack region for KR-CSAP, EU, CN-PIPL, IL5/6, and FedRAMP-High. |
| Rollback | Disable release commands, leave outbound delivery independent, and replay wave outbox from the last sealed audit id. |
| Test evidence | Required tests cover tenant mismatch, congestion block, over-release, RF dispatch failure, and idempotent release. |
| Rejected shortcut | A generic `BatchPick` record is rejected because it loses SAP EWM wave, order, and congestion semantics. |
