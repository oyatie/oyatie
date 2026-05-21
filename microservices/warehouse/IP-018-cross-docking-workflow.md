---
doc_class: ImplementationPlan
ip_id: IP-018
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
journey_ref: j103-just-in-time-procurement-automation
sap_submodule: EWM-DLV (deliveries)
tenant_class: paid
billing_components:
  - per_usage
persona: Priya Menon, inbound dock lead
status: Accepted
date: 2026-05-20
owner_team: axis-warehouse + axis-erp-parity
---

# IP-018: Cross-docking workflow

## Context

- SAP submodule: EWM-DLV inbound-to-outbound cross-docking.
- Persona: Priya Menon, inbound dock lead.
- Journey leg: j103 just-in-time receipt is redirected directly to outbound staging instead of storage.
- SAP tables: `/SCWM/PRDI`, `/SCWM/PRDO`, `/SCWM/ORDIM_O`, `/SCWM/QUANT`.
- Oyatie capability: `CrossDockCoordinator`.
- Precedent: SAP EWM opportunistic cross-docking plus Amazon fulfillment flow-through allocation.
- ADR-0297 gates direct stock bypass of storage and ADR-0263 seals both inbound and outbound lineage.
- Boundary: matches inbound receipt to outbound demand and creates direct movement task; it does not alter customer promise or billing.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.cross_dock_match (
  tenant_id UUID NOT NULL,
  cross_dock_match_id TEXT NOT NULL,
  inbound_delivery_id TEXT NOT NULL,
  inbound_item_no TEXT NOT NULL,
  outbound_delivery_id TEXT NOT NULL,
  outbound_line_no TEXT NOT NULL,
  matched_qty NUMERIC(18,6) NOT NULL,
  match_status TEXT NOT NULL CHECK (match_status IN ('proposed','accepted','rejected','moved','expired')),
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, cross_dock_match_id)
);
CREATE TABLE warehouse.cross_dock_movement (
  tenant_id UUID NOT NULL,
  movement_id TEXT NOT NULL,
  cross_dock_match_id TEXT NOT NULL,
  source_dock_door_id TEXT NOT NULL,
  destination_staging_bin_id TEXT NOT NULL,
  movement_status TEXT NOT NULL,
  PRIMARY KEY (tenant_id, movement_id)
);
```

### Rust Types

```rust
pub struct CrossDockMatch {
    pub tenant_id: TenantId,
    pub cross_dock_match_id: CrossDockMatchId,
    pub inbound_delivery_id: InboundDeliveryId,
    pub inbound_item_no: DeliveryItemNo,
    pub outbound_delivery_id: OutboundDeliveryId,
    pub outbound_line_no: DeliveryLineNo,
    pub matched_qty: Decimal,
    pub match_status: CrossDockMatchStatus,
}
pub struct CrossDockMovement {
    pub movement_id: MovementId,
    pub cross_dock_match_id: CrossDockMatchId,
    pub source_dock_door_id: DockDoorId,
    pub destination_staging_bin_id: BinId,
    pub movement_status: MovementState,
}
pub enum CrossDockError { DemandNotEligible, QualityHoldActive, DockDoorMismatch, MoveTaskFailed, MatchExpired }
```

## API Endpoints

- REST `POST /v1/warehouse/cross-dock-matches:propose` matches inbound item to outbound line.
- REST `POST /v1/warehouse/cross-dock-matches/{id}:accept` accepts direct flow-through.
- REST `POST /v1/warehouse/cross-dock-matches/{id}:move` creates direct movement.
- gRPC `warehouse.crossdock.v1.CrossDockService.ProposeCrossDock`.
- gRPC `AcceptCrossDock`, `MoveCrossDockStock`, and `ExpireCrossDockMatch`.
- AsyncAPI channel `warehouse.cross-dock.match-accepted.v1`.
- AsyncAPI channel `warehouse.cross-dock.moved.v1`.
- Consumers: inbound delivery, outbound delivery, inventory-ledger, picking-wave.

## Cedar Policy Hooks

- Policy: `warehouse::cross_dock::accept`.
- Principal: `WarehouseDockSupervisor`.
- Action: `cross_dock_accept`.
- Resource: `CrossDockMatch`.
- Context: `tenant_id`, `quality_gate_state`, `outbound_priority`, `source_dock_door_id`, `destination_staging_bin_id`.
- Forbid when quality hold is active, outbound demand is not eligible, or direct movement crosses incompatible dock zones.

## Ontology Projection

- Vendor object: SAP EWM cross-docking decision.
- Oyatie object: `warehouse.cross_dock_match`.
- `/SCWM/PRDI-DOCID` -> inbound delivery reference.
- `/SCWM/PRDO-DOCID` -> outbound delivery reference.
- `/SCWM/ORDIM_O-TANUM` -> movement task lineage.
- `/SCWM/QUANT-MATID` -> matched stock evidence.
- Source dock -> source movement node.
- Staging bin -> outbound staging destination.
- Projection freshness floor: 3 seconds.
- Projection rule: cross-dock match links inbound and outbound without changing either source identity.

## Workflow Steps

- Node `demand-read`: load urgent outbound demand.
- Node `receipt-read`: load inbound item eligible for cross-dock.
- Decision `quality-hold-active`: reject match and route to inspection.
- Decision `demand-not-eligible`: fall back to normal putaway.
- Node `match-score`: score quantity, timing, and dock proximity.
- Node `policy-evaluate`: confirm direct movement is allowed.
- Node `match-accept`: reserve inbound quantity for outbound line.
- Node `movement-create`: create dock-to-staging movement task.
- Decision `move-task-failed`: release match and retry normal putaway.
- Node `audit-seal`: emit cross-dock evidence.

## Audit Events

- `EVT-WAREHOUSE-CROSS_DOCK-MATCH_PROPOSED`.
- `EVT-WAREHOUSE-CROSS_DOCK-MATCH_ACCEPTED`.
- `EVT-WAREHOUSE-CROSS_DOCK-MATCH_REJECTED`.
- `EVT-WAREHOUSE-CROSS_DOCK-MOVED`.
- `EVT-WAREHOUSE-CROSS_DOCK-POLICY_DENIED`.
- `EVT-WAREHOUSE-CROSS_DOCK-IP_ACCEPTED`.
- ADR-0263 envelope stores inbound/outbound refs, `matched_qty`, quality gate state, and movement ID.

## SLO Targets

- Match proposal p50: 55 ms.
- Match proposal p95: 210 ms.
- Match proposal p99: 600 ms.
- Movement task creation p95: 300 ms.
- Rationale: cross-dock decisions happen while trucks are at dock, so match scoring must be near-real-time.

## Failure Modes and Recovery

- Failure: `QUALITY-HOLD-ACTIVE`; recovery: reject cross-dock and request quality inspection.
- Failure: `DEMAND-NOT-ELIGIBLE`; recovery: route stock to standard putaway.
- Failure: `DOCK-DOOR-MISMATCH`; recovery: propose staging transfer or reject match.
- Failure: `MOVE-TASK-FAILED`; recovery: release reservation and retry putaway.
- Failure: `MATCH-EXPIRED`; recovery: expire match and recompute demand.
- Failure: `OUTBOUND-CANCELLED`; recovery: cancel movement and restore inbound stock to staging.

## Migration Notes

- Do not infer historical cross-dock matches unless both inbound and outbound lineage are present.
- Import open SAP cross-dock tasks as proposed matches requiring validation.
- Preserve flow-through time metrics for performance baseline.
- Normalize staging bin identifiers before movement import.
- Rollback path: disable cross-dock acceptance and use normal putaway plus outbound pick.
- Backfill order: inbound delivery, outbound delivery, staging bins, matches, movements.

## Cross-microservice Handoffs

- From inbound delivery: received item and quality gate.
- From outbound delivery: urgent demand and staging target.
- To inventory-ledger: direct movement and stock reservation.
- To picking-wave: staged outbound line readiness.
- To workflow-engine: rejected or expired match review.
- To compliance: linked inbound/outbound evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The workflow remains bound to SAP EWM inbound-to-outbound cross-docking. |
| Persona specificity | Priya Menon owns cross-dock match acceptance, expiry, and rollback language. |
| Journey specificity | The j103 just-in-time receipt leg drives direct staging instead of storage. |
| DDL anchor | The cross-dock match and movement tables above are normative. |
| Rust anchor | The cross-dock match, movement result, and error enum above are implementation anchors. |
| REST anchor | Match, accept, expire, and reject endpoints are the tenant command surface. |
| gRPC anchor | The cross-docking service is the worker and replay contract. |
| AsyncAPI anchor | Match accepted, expired, and moved channels carry inbound/outbound linkage evidence. |
| Cedar anchor | Cross-dock acceptance is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP inbound, outbound, staging-bin, and movement lineage projects to cross-dock nodes. |
| ADR-0263 class binding | Cross-dock checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Perishable, controlled-material, or site overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on cross-dock APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, inbound id, outbound id, staging bin, match score, and `cedar_decision_id`. |
| Metric | `oya_warehouse_cross_dock_matches_total{tenant_id,cell_id,outcome,status}` caps outcome/status cardinality. |
| Latency histogram | `oya_warehouse_cross_dock_duration_seconds` tracks receipt-to-stage movement latency. |
| Trace span | `warehouse.cross_dock.accept_match` links inbound delivery, outbound delivery, inventory-ledger, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `match_id`, `staging_bin`, `expiry_at`, and rejection reason. |
| Capacity math | Cross-dock acceptance requires staging dwell window greater than expected movement time plus safety margin. |
| Multi-region | Home-cell cross-dock movement is authoritative; DR cells serve read-only match history. |
| Sovereign cells | Supplier, customer, and controlled-material evidence remains in-region for active packs. |
| Rollback | Disable cross-dock acceptance, use normal putaway plus outbound pick, and replay from last sealed movement audit id. |
| Test evidence | Required tests cover expired match, quality gate denial, staging conflict, tenant mismatch, and idempotent movement. |
| Rejected shortcut | A generic transfer workflow is rejected because it loses linked EWM inbound/outbound cross-dock semantics. |
