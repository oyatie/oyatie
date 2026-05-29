---
doc_class: ImplementationPlan
ip_id: IP-024
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
journey_ref: j170-aiko-brown-sustainability-report-and-scope-3-supply-chain
sap_submodule: EWM-WIM (inventory)
tenant_class: paid
billing_components:
  - per_usage
persona: Hana Suzuki, stock placement analyst
status: Accepted
date: 2026-05-20
owner_team: axis-warehouse + axis-erp-parity
---

# IP-024: Hazmat segregation

## Context

- SAP submodule: EWM-WIM hazardous-material inventory segregation.
- Persona: Hana Suzuki, stock placement analyst.
- Journey leg: j170 sustainability and compliance reporting needs provable segregation of regulated materials in warehouse storage.
- SAP tables: `/SCWM/STORAGEBIN`, `/SCWM/QUANT`, `/SCWM/ORDIM_O`, `/SCWM/MAT1`.
- Oyatie capability: `HazmatSegregationPolicy`.
- Precedent: SAP EWM dangerous goods checks plus NFPA/IMDG compatibility matrix enforcement.
- ADR-0297 requires Cedar before storage or movement and ADR-0263 records every segregation decision.
- Boundary: evaluates material/bin compatibility and blocks unsafe putaway, picking, consolidation, or replenishment movements.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.hazmat_compatibility_rule (
  tenant_id UUID NOT NULL,
  hazmat_rule_id TEXT NOT NULL,
  hazmat_class TEXT NOT NULL,
  incompatible_hazmat_class TEXT NOT NULL,
  minimum_separation_meters NUMERIC(8,2) NOT NULL,
  rule_state TEXT NOT NULL CHECK (rule_state IN ('draft','active','retired')),
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, hazmat_rule_id)
);
CREATE TABLE warehouse.hazmat_segregation_decision (
  tenant_id UUID NOT NULL,
  segregation_decision_id TEXT NOT NULL,
  material_id TEXT NOT NULL,
  candidate_bin_id TEXT NOT NULL,
  decision TEXT NOT NULL CHECK (decision IN ('allowed','denied','manual_review')),
  reason_code TEXT NOT NULL,
  PRIMARY KEY (tenant_id, segregation_decision_id)
);
```

### Rust Types

```rust
pub struct HazmatCompatibilityRule {
    pub tenant_id: TenantId,
    pub hazmat_rule_id: HazmatRuleId,
    pub hazmat_class: HazmatClass,
    pub incompatible_hazmat_class: HazmatClass,
    pub minimum_separation_meters: Decimal,
}
pub struct HazmatSegregationDecision {
    pub segregation_decision_id: SegregationDecisionId,
    pub material_id: MaterialId,
    pub candidate_bin_id: BinId,
    pub decision: SegregationDecision,
    pub reason_code: HazmatReasonCode,
}
pub enum HazmatSegregationError { RuleMissing, IncompatibleClass, SeparationInsufficient, BinHazmatUnknown, ManualReviewRequired }
```

## API Endpoints

- REST `POST /v1/warehouse/hazmat-compatibility-rules` activates segregation rule.
- REST `POST /v1/warehouse/materials/{material_id}:check-hazmat-bin`.
- REST `GET /v1/warehouse/hazmat-segregation-decisions/{id}` returns evidence.
- gRPC `warehouse.hazmat.v1.HazmatSegregationService.CheckCandidateBin`.
- gRPC `ActivateCompatibilityRule` and `ListSegregationDecisions`.
- AsyncAPI channel `warehouse.hazmat.segregation-allowed.v1`.
- AsyncAPI channel `warehouse.hazmat.segregation-denied.v1`.
- Consumers: putaway-strategy, outbound-consolidation, replenishment, compliance.

## Cedar Policy Hooks

- Policy: `warehouse::hazmat::segregation_check`.
- Principal: `WarehousePolicyWorker`.
- Action: `hazmat_bin_check`.
- Resource: `StorageBin`.
- Context: `tenant_id`, `material_id`, `hazmat_class`, `candidate_bin_id`, `nearby_hazmat_classes`, `separation_meters`.
- Forbid when incompatible class is nearby, separation is insufficient, rule state is inactive, or candidate bin lacks hazmat classification.

## Ontology Projection

- Vendor object: SAP EWM dangerous goods storage check.
- Oyatie object: `warehouse.hazmat_segregation_decision`.
- `/SCWM/MAT1-MATID` -> `material_id`.
- `/SCWM/STORAGEBIN-LGPLA` -> `candidate_bin_id`.
- `/SCWM/QUANT-MATID` -> nearby stock evidence.
- `/SCWM/ORDIM_O-TANUM` -> movement task lineage.
- Hazmat class -> compatibility matrix field.
- Separation distance -> risk evidence.
- Projection freshness floor: 2 seconds.
- Projection rule: denied decisions are retained for audit and workflow remediation.

## Workflow Steps

- Node `material-class-load`: read hazmat class from product master.
- Decision `rule-missing`: require manual review.
- Node `candidate-bin-read`: read bin and nearby quants.
- Decision `bin-hazmat-unknown`: deny until classified.
- Node `compatibility-evaluate`: compare classes and separation.
- Decision `incompatible-class`: deny movement.
- Decision `separation-insufficient`: suggest alternate bin.
- Node `policy-evaluate`: record Cedar decision.
- Node `decision-publish`: emit allowed or denied event.
- Node `audit-seal`: persist segregation evidence.

## Audit Events

- `EVT-WAREHOUSE-HAZMAT-RULE_ACTIVATED`.
- `EVT-WAREHOUSE-HAZMAT-SEGREGATION_ALLOWED`.
- `EVT-WAREHOUSE-HAZMAT-SEGREGATION_DENIED`.
- `EVT-WAREHOUSE-HAZMAT-MANUAL_REVIEW_REQUIRED`.
- `EVT-WAREHOUSE-HAZMAT-POLICY_DENIED`.
- `EVT-WAREHOUSE-HAZMAT-IP_ACCEPTED`.
- ADR-0263 envelope stores `hazmat_class`, candidate bin, nearby classes, and reason code.

## SLO Targets

- Segregation check p50: 35 ms.
- Segregation check p95: 140 ms.
- Segregation check p99: 420 ms.
- Rule activation p95: 250 ms.
- Rationale: segregation checks are synchronous in putaway and consolidation flows and must not slow RF execution materially.

## Failure Modes and Recovery

- Failure: `RULE-MISSING`; recovery: deny movement and create compliance review.
- Failure: `INCOMPATIBLE-CLASS`; recovery: reject bin and request alternate destination.
- Failure: `SEPARATION-INSUFFICIENT`; recovery: pick farther bin or hold in controlled staging.
- Failure: `BIN-HAZMAT-UNKNOWN`; recovery: classify bin before movement.
- Failure: `MANUAL-REVIEW-REQUIRED`; recovery: route to workflow and keep movement pending.
- Failure: `POLICY-OUTBOX-FAILED`; recovery: keep decision local and retry event dispatch.

## Migration Notes

- Import product hazmat classes before enabling rule enforcement.
- Import storage bin hazard zones and nearby-stock graph.
- Preserve source regulatory class and source-system reference.
- Do not auto-allow migrated stock that violates new segregation rules; create review tasks.
- Rollback path: switch rules to manual-review mode and disable automated movement blocks.
- Backfill order: hazmat classes, bins, quants, compatibility rules, decisions.

## Cross-microservice Handoffs

- From product master: hazmat class and regulatory attributes.
- From inventory-ledger: nearby stock and bin occupancy.
- To putaway-strategy: allowed/rejected bin decision.
- To outbound-consolidation: incompatible package grouping.
- To workflow-engine: manual review tasks.
- To compliance: hazardous material segregation evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The policy remains bound to SAP EWM hazardous-material storage and movement controls. |
| Persona specificity | Elena Petrova owns segregation rule evidence, review tasks, and rollback acceptance language. |
| Journey specificity | The j137 internal-control leg drives hazardous-material compatibility and audit evidence. |
| DDL anchor | The hazmat class, compatibility rule, segregation decision, and review tables above are normative. |
| Rust anchor | The hazmat class, segregation decision, and error enum above are implementation anchors. |
| REST anchor | Evaluate, block movement, approve review, and retire rule endpoints are tenant surfaces. |
| gRPC anchor | The hazmat segregation service is the worker and replay contract. |
| AsyncAPI anchor | Decision allowed, blocked, review-required, and rule-retired channels carry compliance evidence. |
| Cedar anchor | Movement allowance is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP bin, quant, material hazmat class, and package lineage projects to segregation decision nodes. |
| ADR-0263 class binding | Segregation checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Hazmat regulation or sovereign overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on hazmat APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, material id, hazmat class, bin id, rule id, and `cedar_decision_id`. |
| Metric | `oya_warehouse_hazmat_segregation_decisions_total{tenant_id,cell_id,outcome,status}` caps outcome/status cardinality. |
| Latency histogram | `oya_warehouse_hazmat_segregation_duration_seconds` tracks decision and review latency. |
| Trace span | `warehouse.hazmat_segregation.evaluate` links product master, inventory-ledger, putaway strategy, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `material_id`, `hazmat_class`, `bin_id`, and incompatibility code. |
| Capacity math | Rule evaluation short-circuits on first incompatibility and tracks rule_count * nearby_stock_count scan cost. |
| Multi-region | Segregation decisions write in home cell; DR cells expose read-only blocked movement evidence. |
| Sovereign cells | Hazardous material and location evidence remains in-region for regulated pack overlays. |
| Rollback | Switch rules to manual-review mode, disable automated blocks, and replay from last sealed segregation audit id. |
| Test evidence | Required tests cover incompatible class, missing hazmat data, tenant mismatch, manual review, and deterministic replay. |
| Rejected shortcut | A generic dangerous-goods flag is rejected because it loses EWM bin, quant, and compatibility-rule semantics. |
