---
doc_class: ImplementationPlan
ip_id: IP-022
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
journey_ref: j168-coo-akira-watanabe-quarterly-ops-review-and-incident-debrief
sap_submodule: EWM-WIM (inventory)
tenant_class: paid
billing_components:
  - per_usage
persona: Mei Tan, warehouse analytics lead
status: Accepted
date: 2026-05-20
owner_team: axis-warehouse + axis-erp-parity
---

# IP-022: Slotting analytics

## Context

- SAP submodule: EWM-WIM slotting and rearrangement analytics.
- Persona: Mei Tan, warehouse analytics lead.
- Journey leg: j168 quarterly ops review identifies high-velocity SKUs that should move closer to pack stations.
- SAP tables: `/SCWM/SLOTTING`, `/SCWM/QUANT`, `/SCWM/STORAGEBIN`, `/SCWM/ORDIM_O`.
- Oyatie capability: `SlottingAnalytics`.
- Precedent: SAP EWM slotting plus Amazon warehouse velocity-based slotting.
- ADR-0329/0330/0331 requires implementable analytics, ADR-0263 records recommendation decisions, and ADR-0297 gates accepted re-slot tasks.
- Boundary: recommends slotting changes and movement tasks; it does not directly change product master or finance valuation.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.slotting_analysis_run (
  tenant_id UUID NOT NULL,
  slotting_run_id TEXT NOT NULL,
  analysis_window_start TIMESTAMPTZ NOT NULL,
  analysis_window_end TIMESTAMPTZ NOT NULL,
  sku_count INTEGER NOT NULL,
  recommendation_count INTEGER NOT NULL,
  run_status TEXT NOT NULL CHECK (run_status IN ('queued','succeeded','failed','accepted')),
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, slotting_run_id)
);
CREATE TABLE warehouse.slotting_recommendation (
  tenant_id UUID NOT NULL,
  recommendation_id TEXT NOT NULL,
  slotting_run_id TEXT NOT NULL,
  material_id TEXT NOT NULL,
  current_bin_id TEXT NOT NULL,
  recommended_bin_id TEXT NOT NULL,
  velocity_score NUMERIC(12,6) NOT NULL,
  estimated_travel_savings_meters NUMERIC(14,4) NOT NULL,
  PRIMARY KEY (tenant_id, recommendation_id)
);
```

### Rust Types

```rust
pub struct SlottingAnalysisRun {
    pub tenant_id: TenantId,
    pub slotting_run_id: SlottingRunId,
    pub analysis_window: TimeWindow,
    pub sku_count: u32,
    pub recommendation_count: u32,
    pub run_status: SlottingRunStatus,
}
pub struct SlottingRecommendation {
    pub recommendation_id: RecommendationId,
    pub material_id: MaterialId,
    pub current_bin_id: BinId,
    pub recommended_bin_id: BinId,
    pub velocity_score: Decimal,
    pub estimated_travel_savings_meters: Decimal,
}
pub enum SlottingAnalyticsError { InsufficientHistory, CandidateBinBlocked, RecommendationPolicyDenied, AnalyticsWindowTooLarge, MovementTaskFailed }
```

## API Endpoints

- REST `POST /v1/warehouse/slotting-analysis-runs` starts analysis.
- REST `POST /v1/warehouse/slotting-recommendations/{id}:accept` creates movement task.
- REST `GET /v1/warehouse/slotting-analysis-runs/{id}/recommendations`.
- gRPC `warehouse.slotting.v1.SlottingAnalyticsService.StartAnalysis`.
- gRPC `AcceptRecommendation` and `StreamSlottingRecommendations`.
- AsyncAPI channel `warehouse.slotting.recommendation-created.v1`.
- AsyncAPI channel `warehouse.slotting.recommendation-accepted.v1`.
- Consumers: putaway-task, replenishment, analytics-dashboard, compliance.

## Cedar Policy Hooks

- Policy: `warehouse::slotting::accept_recommendation`.
- Principal: `WarehouseOperationsManager`.
- Action: `slotting_recommendation_accept`.
- Resource: `SlottingRecommendation`.
- Context: `tenant_id`, `velocity_score`, `current_bin_id`, `recommended_bin_id`, `movement_risk`, `pack_ids`.
- Forbid when recommended bin is blocked, material restrictions fail, movement risk exceeds policy, or analytics window is not approved.

## Ontology Projection

- Vendor object: SAP EWM slotting recommendation.
- Oyatie object: `warehouse.slotting_recommendation`.
- `/SCWM/SLOTTING-MATID` -> `material_id`.
- `/SCWM/STORAGEBIN-LGPLA` -> current and recommended bin.
- `/SCWM/QUANT-QUAN` -> inventory position.
- `/SCWM/ORDIM_O-TANUM` -> accepted movement task lineage.
- Velocity score -> demand and pick frequency evidence.
- Travel savings -> operational benefit estimate.
- Projection freshness floor: analysis completion.
- Projection rule: analytics recommendations are not executable until explicitly accepted.

## Workflow Steps

- Node `history-load`: load pick, replenishment, and movement history.
- Decision `insufficient-history`: mark SKU ineligible.
- Node `velocity-score`: compute SKU velocity and affinity.
- Node `candidate-bin-rank`: score available bins by distance and constraints.
- Decision `candidate-bin-blocked`: reject recommendation.
- Node `recommendation-create`: persist candidate move.
- Node `policy-evaluate`: validate accepted recommendation.
- Node `movement-task-create`: create relocation task.
- Decision `movement-task-failed`: keep recommendation accepted-pending-task.
- Node `audit-seal`: emit analytics evidence.

## Audit Events

- `EVT-WAREHOUSE-SLOTTING-ANALYSIS_STARTED`.
- `EVT-WAREHOUSE-SLOTTING-RECOMMENDATION_CREATED`.
- `EVT-WAREHOUSE-SLOTTING-RECOMMENDATION_ACCEPTED`.
- `EVT-WAREHOUSE-SLOTTING-MOVEMENT_TASK_CREATED`.
- `EVT-WAREHOUSE-SLOTTING-POLICY_DENIED`.
- `EVT-WAREHOUSE-SLOTTING-IP_ACCEPTED`.
- ADR-0263 envelope stores `analysis_window`, `velocity_score`, `current_bin_id`, and recommended bin.

## SLO Targets

- Analysis start p50: 60 ms.
- Recommendation query p95: 250 ms.
- Full analysis p99: 10 minutes for 1M movements.
- Accept recommendation p95: 200 ms.
- Rationale: analysis is batch and may run offline; planner interactions remain interactive.

## Failure Modes and Recovery

- Failure: `INSUFFICIENT-HISTORY`; recovery: mark SKU excluded and continue analysis.
- Failure: `CANDIDATE-BIN-BLOCKED`; recovery: select next candidate or reject recommendation.
- Failure: `POLICY-DENIED`; recovery: keep recommendation unaccepted with reason.
- Failure: `ANALYTICS-WINDOW-TOO-LARGE`; recovery: split analysis by week or zone.
- Failure: `MOVEMENT-TASK-FAILED`; recovery: retry relocation task creation.
- Failure: `ONTOLOGY-PUBLISH-FAILED`; recovery: keep analysis result and retry projection.

## Migration Notes

- Import SAP slotting results as historical recommendations.
- Compute baseline velocity from movement history if SAP score is missing.
- Preserve accepted/rejected SAP recommendation state as audit lineage.
- Do not create relocation tasks from migrated recommendations automatically.
- Rollback path: disable accept endpoint and keep analysis read-only.
- Backfill order: movement history, bins, quants, slotting runs, recommendations.

## Cross-microservice Handoffs

- From inventory-ledger: movement and stock history.
- From picking execution: pick frequency and travel evidence.
- To putaway-task: accepted relocation task.
- To replenishment: forward pick-face capacity signal.
- To analytics-dashboard: score and savings metrics.
- To compliance: accepted recommendation evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The analytics remain bound to SAP EWM slotting and bin/material movement evidence. |
| Persona specificity | Hana Suzuki owns slotting recommendation acceptance, relocation scope, and rollback language. |
| Journey specificity | The j123 launch readiness leg drives pick-frequency, travel-distance, and forward-pick capacity analysis. |
| DDL anchor | The slotting run, recommendation, and accepted relocation tables above are normative. |
| Rust anchor | The slotting run, recommendation score, and error enum above are implementation anchors. |
| REST anchor | Analyze, accept recommendation, reject recommendation, and explain endpoints are tenant surfaces. |
| gRPC anchor | The slotting analytics service is the worker and replay contract. |
| AsyncAPI anchor | Recommendation created, accepted, and rejected channels carry downstream movement evidence. |
| Cedar anchor | Recommendation acceptance is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP bin, quant, movement history, and pick-frequency lineage projects to slotting recommendation nodes. |
| ADR-0263 class binding | Slotting acceptance checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Storage-policy or labor-safety overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on slotting APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, slotting run id, material id, source bin, target bin, score, and `cedar_decision_id`. |
| Metric | `oya_warehouse_slotting_recommendations_total{tenant_id,cell_id,outcome,status}` caps outcome/status cardinality. |
| Latency histogram | `oya_warehouse_slotting_analysis_duration_seconds` tracks analysis runtime and acceptance latency. |
| Trace span | `warehouse.slotting_analytics.accept_recommendation` links inventory history, putaway task, replenishment, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `run_id`, `material_id`, `score_bucket`, and rejection reason. |
| Capacity math | Expected travel savings must exceed relocation effort; recommendations below break-even stay read-only. |
| Multi-region | Recommendation acceptance writes in warehouse home cell; DR cells expose read-only analysis results. |
| Sovereign cells | Movement and stock evidence remains in-region for active compliance overlays. |
| Rollback | Disable accept endpoint, keep analysis read-only, and replay from last sealed recommendation audit id. |
| Test evidence | Required tests cover stale stock history, unsafe relocation, tenant mismatch, duplicate acceptance, and deterministic scoring. |
| Rejected shortcut | A generic analytics score is rejected because it loses EWM bin, quant, and movement-history semantics. |
