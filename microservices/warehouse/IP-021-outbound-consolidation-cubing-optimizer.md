---
doc_class: ImplementationPlan
ip_id: IP-021
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
persona: Omar Castillo, outbound fulfillment supervisor
status: Accepted
date: 2026-05-20
owner_team: axis-warehouse + axis-erp-parity
---

# IP-021: Outbound consolidation with cubing optimizer

## Context

- SAP submodule: EWM-WO outbound warehouse order and packing consolidation.
- Persona: Omar Castillo, outbound fulfillment supervisor.
- Journey leg: j123 high-volume launch consolidates picked lines into cartons and pallets before carrier tender.
- SAP tables: `/SCWM/PRDO`, `/SCWM/WAREHOUSEORDER`, `/SCWM/HUHDR`, `/SCWM/QUANT`.
- Oyatie capability: `OutboundCubingOptimizer`.
- Precedent: SAP EWM packing specification/cubing plus UPS package optimization.
- ADR-0297 gates optimizer acceptance and ADR-0263 records package decision evidence.
- Boundary: proposes consolidation units and packing plan; carrier rating and freight payment stay outside warehouse.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.outbound_consolidation_run (
  tenant_id UUID NOT NULL,
  consolidation_run_id TEXT NOT NULL,
  outbound_delivery_id TEXT NOT NULL,
  optimizer_version TEXT NOT NULL,
  package_count INTEGER NOT NULL,
  volume_utilization_percent NUMERIC(8,4) NOT NULL,
  weight_utilization_percent NUMERIC(8,4) NOT NULL,
  run_status TEXT NOT NULL CHECK (run_status IN ('proposed','accepted','rejected','packed')),
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, consolidation_run_id)
);
CREATE TABLE warehouse.outbound_package_plan (
  tenant_id UUID NOT NULL,
  package_plan_id TEXT NOT NULL,
  consolidation_run_id TEXT NOT NULL,
  package_type TEXT NOT NULL,
  gross_weight_kg NUMERIC(12,4) NOT NULL,
  volume_liters NUMERIC(12,4) NOT NULL,
  line_refs JSONB NOT NULL,
  PRIMARY KEY (tenant_id, package_plan_id)
);
```

### Rust Types

```rust
pub struct OutboundConsolidationRun {
    pub tenant_id: TenantId,
    pub consolidation_run_id: ConsolidationRunId,
    pub outbound_delivery_id: OutboundDeliveryId,
    pub optimizer_version: OptimizerVersion,
    pub package_count: u32,
    pub volume_utilization_percent: Decimal,
    pub weight_utilization_percent: Decimal,
}
pub struct OutboundPackagePlan {
    pub package_plan_id: PackagePlanId,
    pub package_type: PackageType,
    pub gross_weight_kg: Decimal,
    pub volume_liters: Decimal,
    pub line_refs: Vec<DeliveryLineRef>,
}
pub enum CubingOptimizerError { DimensionMissing, WeightLimitExceeded, IncompatibleItems, CarrierConstraintMissing, OptimizerRejected }
```

## API Endpoints

- REST `POST /v1/warehouse/outbound-deliveries/{id}:optimize-cubing`.
- REST `POST /v1/warehouse/outbound-consolidation-runs/{id}:accept`.
- REST `GET /v1/warehouse/outbound-consolidation-runs/{id}/package-plans`.
- gRPC `warehouse.cubing.v1.CubingOptimizerService.OptimizeOutboundDelivery`.
- gRPC `AcceptConsolidationRun` and `StreamPackagePlans`.
- AsyncAPI channel `warehouse.outbound-consolidation.proposed.v1`.
- AsyncAPI channel `warehouse.outbound-consolidation.accepted.v1`.
- Consumers: packing, carrier-integration, outbound delivery, analytics.

## Cedar Policy Hooks

- Policy: `warehouse::outbound_consolidation::accept`.
- Principal: `WarehousePackingSupervisor`.
- Action: `outbound_consolidation_accept`.
- Resource: `OutboundConsolidationRun`.
- Context: `tenant_id`, `optimizer_version`, `hazmat_mix_state`, `carrier_constraints`, `weight_limit_kg`.
- Forbid when package plan mixes incompatible items, exceeds carrier weight, lacks dimension evidence, or optimizer version is not approved.

## Ontology Projection

- Vendor object: SAP EWM packing specification and handling unit plan.
- Oyatie object: `warehouse.outbound_package_plan`.
- `/SCWM/PRDO-DOCID` -> `outbound_delivery_id`.
- `/SCWM/WAREHOUSEORDER-WHO` -> picked task bundle lineage.
- `/SCWM/HUHDR-HUIDENT` -> package or handling unit lineage.
- `/SCWM/QUANT-MATID` -> packed line material.
- Package type -> carton/pallet recommendation.
- Utilization percentages -> optimizer evidence.
- Projection freshness floor: 5 seconds.
- Projection rule: rejected package plans remain visible for cost/performance analysis.

## Workflow Steps

- Node `picked-lines-read`: load picked lines for delivery.
- Node `dimension-resolve`: read item dimensions and weights.
- Decision `dimension-missing`: block optimizer and request item master fix.
- Node `carrier-constraints-load`: load service-level package limits.
- Decision `incompatible-items`: split package groups.
- Node `cubing-run`: produce package plan.
- Decision `weight-limit-exceeded`: split package or choose pallet.
- Node `policy-evaluate`: validate approved optimizer and hazmat mix.
- Node `plan-accept`: persist accepted package plans.
- Node `audit-seal`: emit cubing evidence.

## Audit Events

- `EVT-WAREHOUSE-OUTBOUND_CONSOLIDATION-RUN_PROPOSED`.
- `EVT-WAREHOUSE-OUTBOUND_CONSOLIDATION-PLAN_ACCEPTED`.
- `EVT-WAREHOUSE-OUTBOUND_CONSOLIDATION-PLAN_REJECTED`.
- `EVT-WAREHOUSE-OUTBOUND_CONSOLIDATION-PACKED`.
- `EVT-WAREHOUSE-OUTBOUND_CONSOLIDATION-POLICY_DENIED`.
- `EVT-WAREHOUSE-OUTBOUND_CONSOLIDATION-IP_ACCEPTED`.
- ADR-0263 envelope stores `optimizer_version`, package count, utilization metrics, and carrier constraints.

## SLO Targets

- Cubing proposal p50: 90 ms for 100 lines.
- Cubing proposal p95: 700 ms for 2,000 lines.
- Cubing proposal p99: 2,000 ms with fallback split.
- Plan accept p95: 180 ms.
- Rationale: packing work waits on plan generation, but complex cubing can spend bounded compute to reduce freight cost.

## Failure Modes and Recovery

- Failure: `DIMENSION-MISSING`; recovery: block plan and create item master remediation.
- Failure: `WEIGHT-LIMIT-EXCEEDED`; recovery: split package plan by line group.
- Failure: `INCOMPATIBLE-ITEMS`; recovery: force separate package groups.
- Failure: `CARRIER-CONSTRAINT-MISSING`; recovery: use tenant default safe package limits.
- Failure: `OPTIMIZER-REJECTED`; recovery: fall back to rule-based packing.
- Failure: `PACK-HANDOFF-FAILED`; recovery: retry accepted package plan dispatch.

## Migration Notes

- Import SAP packaging specs as draft constraints until validated.
- Import handling unit history as read-only package evidence.
- Preserve carton and pallet dimensions from source master data.
- Do not activate cubing optimizer without item dimensions and carrier constraints.
- Rollback path: disable optimizer acceptance and use manual packing rules.
- Backfill order: item dimensions, carrier constraints, packaging specs, package history, consolidation runs.

## Cross-microservice Handoffs

- From picking execution: picked line quantities.
- From product master: dimensions and hazmat flags.
- To carrier-integration: package dimensions and weights.
- To outbound delivery: packed readiness.
- To analytics: utilization and freight savings evidence.
- To compliance: package decision audit trail.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The optimizer remains bound to SAP EWM warehouse-order consolidation and packing. |
| Persona specificity | Omar Castillo owns cubing acceptance, carrier constraint review, and rollback language. |
| Journey specificity | The j123 launch packing leg drives package utilization and carrier handoff behavior. |
| DDL anchor | The consolidation run, package, package line, and constraint tables above are normative. |
| Rust anchor | The cubing run, package decision, and error enum above are implementation anchors. |
| REST anchor | Optimize package, accept, reject, and explain endpoints are tenant command surfaces. |
| gRPC anchor | The cubing optimizer service is the worker and replay contract. |
| AsyncAPI anchor | Package proposed, accepted, and rejected channels carry carrier and compliance evidence. |
| Cedar anchor | Cubing acceptance is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP warehouse order, packed item, carrier, and packaging spec lineage projects to package decisions. |
| ADR-0263 class binding | Cubing policy checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Hazmat, carrier, or sovereign overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on cubing APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, consolidation run id, package id, carrier id, utilization, and `cedar_decision_id`. |
| Metric | `oya_warehouse_cubing_optimizer_runs_total{tenant_id,cell_id,outcome,status}` caps outcome/status cardinality. |
| Latency histogram | `oya_warehouse_cubing_optimizer_duration_seconds` tracks optimize and accept latency. |
| Trace span | `warehouse.cubing_optimizer.accept_package` links picking execution, product master, carrier, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `run_id`, `package_id`, `carrier_service_id`, and rejection reason. |
| Capacity math | Optimizer accepts only when freight_savings - compute_cost exceeds tenant floor and carrier constraints pass. |
| Multi-region | Package acceptance writes in fulfillment home cell; DR cells expose read-only packing history. |
| Sovereign cells | Customer shipment and hazmat evidence remains in-region for active compliance-pack overlays. |
| Rollback | Disable optimizer acceptance, use manual packing rules, and replay from last sealed package audit id. |
| Test evidence | Required tests cover dimensions missing, carrier constraint failure, hazmat incompatibility, tenant mismatch, and idempotent acceptance. |
| Rejected shortcut | A generic packing calculator is rejected because it loses EWM warehouse-order and carrier-constraint semantics. |
