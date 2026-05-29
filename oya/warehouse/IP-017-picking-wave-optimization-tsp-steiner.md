---
doc_class: ImplementationPlan
ip_id: IP-017
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

# IP-017: Picking wave optimization with TSP and Steiner-tree heuristics

## Context

- SAP submodule: EWM-WO warehouse order optimization.
- Persona: Grace Kim, wave planner.
- Journey leg: j123 large launch wave needs travel-minimizing pick paths without overloading aisles.
- SAP tables: `/SCWM/WAREHOUSEORDER`, `/SCWM/ORDIM_O`, `/SCWM/STORAGEBIN`, `/SCWM/QUANT`.
- Oyatie capability: `PickingRouteOptimizer`.
- Precedent: SAP EWM travel-distance optimization plus Google Maps route optimization and Steiner tree approximation for aisle graph shortcuts.
- ADR-0105 places heuristics in usecase/application boundary, ADR-0297 gates optimizer output, and ADR-0263 records explainability.
- Boundary: computes sequence and bundle shape; it does not dispatch RF confirmations.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.picking_optimization_run (
  tenant_id UUID NOT NULL,
  optimization_run_id TEXT NOT NULL,
  picking_wave_id TEXT NOT NULL,
  heuristic TEXT NOT NULL CHECK (heuristic IN ('nearest_neighbor','two_opt','steiner_aisle_graph','hybrid')),
  candidate_task_count INTEGER NOT NULL,
  estimated_distance_meters NUMERIC(14,4) NOT NULL,
  estimated_duration_seconds INTEGER NOT NULL,
  run_status TEXT NOT NULL CHECK (run_status IN ('queued','succeeded','failed','rejected')),
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, optimization_run_id)
);
CREATE TABLE warehouse.picking_optimized_stop (
  tenant_id UUID NOT NULL,
  optimization_run_id TEXT NOT NULL,
  stop_no INTEGER NOT NULL,
  warehouse_task_id TEXT NOT NULL,
  bin_id TEXT NOT NULL,
  path_segment_ref TEXT NOT NULL,
  PRIMARY KEY (tenant_id, optimization_run_id, stop_no)
);
```

### Rust Types

```rust
pub struct PickingOptimizationRun {
    pub tenant_id: TenantId,
    pub optimization_run_id: OptimizationRunId,
    pub picking_wave_id: PickingWaveId,
    pub heuristic: PickingHeuristic,
    pub candidate_task_count: u32,
    pub estimated_distance_meters: Decimal,
    pub estimated_duration_seconds: u32,
}
pub struct PickingOptimizedStop {
    pub stop_no: u32,
    pub warehouse_task_id: WarehouseTaskId,
    pub bin_id: BinId,
    pub path_segment_ref: PathSegmentRef,
}
pub enum PickingOptimizationError { AisleGraphMissing, HeuristicTimeout, UnsafeCongestion, PolicyDenied, RouteWorseThanBaseline }
```

## API Endpoints

- REST `POST /v1/warehouse/picking-waves/{id}:optimize-route`.
- REST `POST /v1/warehouse/picking-optimization-runs/{id}:accept`.
- REST `GET /v1/warehouse/picking-optimization-runs/{id}` returns route and evidence.
- gRPC `warehouse.picking_optimizer.v1.PickingOptimizerService.OptimizeWave`.
- gRPC `AcceptOptimizedRoute` and `StreamOptimizedStops`.
- AsyncAPI channel `warehouse.picking-optimizer.route-proposed.v1`.
- AsyncAPI channel `warehouse.picking-optimizer.route-accepted.v1`.
- Consumers: picking-wave, labor-assignment, RF execution, analytics.

## Cedar Policy Hooks

- Policy: `warehouse::picking_optimizer::accept`.
- Principal: `WarehouseWavePlanner`.
- Action: `picking_route_accept`.
- Resource: `PickingOptimizationRun`.
- Context: `tenant_id`, `heuristic`, `estimated_distance_meters`, `baseline_distance_meters`, `congestion_score`, `pack_ids`.
- Forbid when route is worse than baseline, congestion score exceeds safety threshold, or heuristic is not enabled for tenant class.

## Ontology Projection

- Vendor object: SAP EWM optimized warehouse order sequence.
- Oyatie object: `warehouse.picking_optimization_run`.
- `/SCWM/WAREHOUSEORDER-WHO` -> `picking_wave_id`.
- `/SCWM/ORDIM_O-TANUM` -> route stop task.
- `/SCWM/STORAGEBIN-LGPLA` -> graph node.
- `/SCWM/QUANT-MATID` -> material pick evidence.
- Aisle graph segment -> `path_segment_ref`.
- Heuristic metrics -> explainability fields.
- Projection freshness floor: 10 seconds.
- Projection rule: rejected optimization runs remain queryable for planner explainability.

## Workflow Steps

- Node `wave-task-load`: load candidate tasks.
- Node `aisle-graph-load`: load bin graph and congestion state.
- Decision `aisle-graph-missing`: fall back to baseline sequence.
- Node `baseline-distance`: compute current task order distance.
- Node `heuristic-run`: execute hybrid TSP and Steiner shortcut search.
- Decision `heuristic-timeout`: keep baseline and record timeout.
- Decision `route-worse-than-baseline`: reject candidate route.
- Node `policy-evaluate`: check safety and tier rules.
- Node `route-accept`: write optimized stops to wave.
- Node `audit-seal`: emit optimizer evidence.

## Audit Events

- `EVT-WAREHOUSE-PICKING_OPTIMIZER-RUN_STARTED`.
- `EVT-WAREHOUSE-PICKING_OPTIMIZER-ROUTE_PROPOSED`.
- `EVT-WAREHOUSE-PICKING_OPTIMIZER-ROUTE_ACCEPTED`.
- `EVT-WAREHOUSE-PICKING_OPTIMIZER-ROUTE_REJECTED`.
- `EVT-WAREHOUSE-PICKING_OPTIMIZER-POLICY_DENIED`.
- `EVT-WAREHOUSE-PICKING_OPTIMIZER-IP_ACCEPTED`.
- ADR-0263 envelope stores `heuristic`, `baseline_distance_meters`, `estimated_distance_meters`, and congestion score.

## SLO Targets

- Optimization p50: 180 ms for 500 stops.
- Optimization p95: 1,200 ms for 5,000 stops.
- Optimization p99: 3,000 ms with timeout fallback.
- Route accept p95: 200 ms.
- Rationale: route proposal can take longer than RF actions, but must finish before wave release planning stalls.

## Failure Modes and Recovery

- Failure: `AISLE-GRAPH-MISSING`; recovery: fall back to warehouse order baseline sequence.
- Failure: `HEURISTIC-TIMEOUT`; recovery: return best-so-far route only if it beats baseline.
- Failure: `UNSAFE-CONGESTION`; recovery: reject route and split wave.
- Failure: `POLICY-DENIED`; recovery: keep existing wave sequence.
- Failure: `ROUTE-WORSE-THAN-BASELINE`; recovery: reject and store evidence.
- Failure: `RF-SEQUENCE-PUBLISH-FAILED`; recovery: retry route publish through outbox.

## Migration Notes

- Build aisle graph from storage bin master before enabling optimizer.
- Import SAP warehouse order sequences as baseline routes.
- Preserve legacy route metrics as comparison evidence.
- Enable heuristic by tenant tenant class after dry-run acceptance.
- Rollback path: disable optimizer acceptance and fall back to existing picking-wave sequence.
- Backfill order: bins, aisle graph, warehouse orders, wave tasks, optimization runs.

## Cross-microservice Handoffs

- From picking-wave: candidate tasks and wave capacity.
- From labor assignment: resource speed and skill profile.
- From inventory-ledger: source bin stock state.
- To RF execution: optimized stop sequence.
- To analytics: route distance and productivity metrics.
- To compliance: optimizer explainability evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The optimizer remains bound to SAP EWM warehouse-order optimization. |
| Persona specificity | Grace Kim owns route acceptance, congestion rejection, and rollback acceptance language. |
| Journey specificity | The j123 launch-wave route leg drives travel minimization and safe aisle loading. |
| DDL anchor | The optimization run and optimized stop tables above are normative. |
| Rust anchor | `PickingOptimizationRun`, `PickingOptimizedStop`, and optimizer error enum above are implementation anchors. |
| REST anchor | Optimize, accept, and read endpoints are the tenant command and evidence surface. |
| gRPC anchor | `PickingOptimizerService` is the worker and streaming stop contract. |
| AsyncAPI anchor | Route-proposed and route-accepted channels carry RF and analytics evidence. |
| Cedar anchor | Route acceptance is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP warehouse order, task, storage-bin, and quant lineage projects to optimized route nodes. |
| ADR-0263 class binding | Route acceptance checks emit `OfficeBoundaryAttemptEvaluated` plus allowed or denied outcome classes. |
| ADR-0263 pack binding | Optimizer enablement or safety-overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on optimizer APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, heuristic, baseline distance, candidate distance, and `cedar_decision_id`. |
| Metric | `oya_warehouse_picking_optimizer_runs_total{tenant_id,cell_id,heuristic,status}` caps heuristic/status cardinality. |
| Latency histogram | `oya_warehouse_picking_optimizer_duration_seconds` tracks p50/p95/p99 optimization runtime. |
| Trace span | `warehouse.picking_optimizer.optimize_wave` links wave, labor profile, inventory state, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `optimization_run_id`, `heuristic`, `stop_count`, and rejection reason. |
| Capacity math | The optimizer compares baseline_distance - candidate_distance against runtime cost and rejects below savings threshold. |
| Multi-region | Optimization acceptance writes in home cell; DR cells expose read-only route proposals. |
| Sovereign cells | Worker, route, and material evidence remains in-region for active compliance-pack overlays. |
| Rollback | Disable optimizer acceptance, fall back to existing wave sequence, and replay from last sealed route audit id. |
| Test evidence | Required tests cover graph missing, timeout, worse-than-baseline, congestion denial, and idempotent acceptance. |
| Rejected shortcut | A generic route sorter is rejected because it loses TSP, Steiner aisle-graph, and EWM warehouse-order explainability. |
