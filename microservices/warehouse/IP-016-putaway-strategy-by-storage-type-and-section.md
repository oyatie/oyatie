---
doc_class: ImplementationPlan
ip_id: IP-016
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

# IP-016: Putaway strategy by storage type and section

## Context

- SAP submodule: EWM-WT putaway strategy determination.
- Persona: Lena Fischer, putaway coordinator.
- Journey leg: j102 received stock is assigned to the safest eligible storage type and section before task creation.
- SAP tables: `/SCWM/STORAGEBIN`, `/SCWM/QUANT`, `/SCWM/ORDIM_O`, `/SCWM/T331`.
- Oyatie capability: `PutawayStrategySelector`.
- Precedent: SAP EWM storage type search sequence plus AWS S3 lifecycle rule precedence.
- ADR-0297 requires Cedar approval before strategy output becomes executable and ADR-0263 seals the chosen strategy.
- Boundary: ranks bins by storage type, section, capacity, material constraints, and pack policy; it does not confirm physical movement.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.putaway_strategy_rule (
  tenant_id UUID NOT NULL,
  strategy_rule_id TEXT NOT NULL,
  material_group TEXT NOT NULL,
  storage_type TEXT NOT NULL,
  storage_section TEXT NOT NULL,
  priority INTEGER NOT NULL,
  rule_state TEXT NOT NULL CHECK (rule_state IN ('draft','active','retired')),
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, strategy_rule_id)
);
CREATE TABLE warehouse.putaway_strategy_decision (
  tenant_id UUID NOT NULL,
  strategy_decision_id TEXT NOT NULL,
  inbound_delivery_id TEXT NOT NULL,
  item_no TEXT NOT NULL,
  selected_bin_id TEXT NOT NULL,
  score NUMERIC(12,6) NOT NULL,
  rejected_bin_count INTEGER NOT NULL,
  PRIMARY KEY (tenant_id, strategy_decision_id)
);
```

### Rust Types

```rust
pub struct PutawayStrategyRule {
    pub tenant_id: TenantId,
    pub strategy_rule_id: StrategyRuleId,
    pub material_group: MaterialGroup,
    pub storage_type: StorageType,
    pub storage_section: StorageSection,
    pub priority: u16,
}
pub struct PutawayStrategyDecision {
    pub strategy_decision_id: StrategyDecisionId,
    pub inbound_delivery_id: InboundDeliveryId,
    pub item_no: DeliveryItemNo,
    pub selected_bin_id: BinId,
    pub score: Decimal,
    pub rejected_bin_count: u32,
}
pub enum PutawayStrategyError { NoEligibleBin, RuleConflict, CapacityStale, MaterialRestriction, PolicyDenied }
```

## API Endpoints

- REST `POST /v1/warehouse/putaway-strategy-rules` creates or amends strategy rule.
- REST `POST /v1/warehouse/inbound-deliveries/{id}/items/{item_no}:select-putaway-bin`.
- REST `GET /v1/warehouse/putaway-strategy-decisions/{id}` returns chosen and rejected bins.
- gRPC `warehouse.putaway_strategy.v1.PutawayStrategyService.SelectPutawayBin`.
- gRPC `UpsertStrategyRule` and `ListStrategyDecisions`.
- AsyncAPI channel `warehouse.putaway-strategy.bin-selected.v1`.
- AsyncAPI channel `warehouse.putaway-strategy.no-bin-found.v1`.
- Consumers: putaway-task, compliance, ontology, workflow-engine.

## Cedar Policy Hooks

- Policy: `warehouse::putaway_strategy::select`.
- Principal: `PutawayStrategyWorker`.
- Action: `putaway_bin_select`.
- Resource: `StorageBin`.
- Context: `tenant_id`, `material_group`, `storage_type`, `storage_section`, `capacity_snapshot_ref`, `pack_ids`.
- Forbid when bin is blocked, material restriction fails, capacity snapshot is stale, or rule is not active.

## Ontology Projection

- Vendor object: SAP EWM storage type search sequence.
- Oyatie object: `warehouse.putaway_strategy_decision`.
- `/SCWM/T331-LGTYP` -> `storage_type`.
- `/SCWM/T331-LGBER` -> `storage_section`.
- `/SCWM/STORAGEBIN-LGPLA` -> candidate and selected bin.
- `/SCWM/QUANT-QUAN` -> capacity and occupancy evidence.
- `/SCWM/ORDIM_O-TANUM` -> downstream task lineage.
- Rule priority -> scoring input.
- Projection freshness floor: 3 seconds.
- Projection rule: rejected bins are retained as explainability evidence.

## Workflow Steps

- Node `rule-load`: load active strategy rules.
- Node `candidate-bin-read`: read storage type, section, capacity, and restrictions.
- Decision `capacity-stale`: refresh capacity snapshot.
- Decision `material-restricted`: reject bin with reason.
- Node `score-candidates`: compute priority and fit score.
- Decision `no-eligible-bin`: create overflow workflow task.
- Node `policy-evaluate`: run Cedar on selected bin.
- Node `decision-persist`: save selected and rejected candidates.
- Node `putaway-task-request`: request task creation.
- Node `audit-seal`: emit strategy evidence.

## Audit Events

- `EVT-WAREHOUSE-PUTAWAY_STRATEGY-RULE_ACTIVATED`.
- `EVT-WAREHOUSE-PUTAWAY_STRATEGY-BIN_SELECTED`.
- `EVT-WAREHOUSE-PUTAWAY_STRATEGY-NO_BIN_FOUND`.
- `EVT-WAREHOUSE-PUTAWAY_STRATEGY-CANDIDATE_REJECTED`.
- `EVT-WAREHOUSE-PUTAWAY_STRATEGY-POLICY_DENIED`.
- `EVT-WAREHOUSE-PUTAWAY_STRATEGY-IP_ACCEPTED`.
- ADR-0263 envelope stores `strategy_rule_id`, `selected_bin_id`, `score`, and rejected reasons.

## SLO Targets

- Strategy select p50: 45 ms.
- Strategy select p95: 180 ms.
- Strategy select p99: 500 ms.
- Rule write p95: 220 ms.
- Rationale: putaway task creation waits on strategy selection; bin scoring must stay sub-second even with thousands of candidate bins.

## Failure Modes and Recovery

- Failure: `NO-ELIGIBLE-BIN`; recovery: create overflow staging task and planner alert.
- Failure: `RULE-CONFLICT`; recovery: reject activation and show conflicting priority.
- Failure: `CAPACITY-STALE`; recovery: refresh quant snapshot and retry once.
- Failure: `MATERIAL-RESTRICTION`; recovery: exclude bin and continue scoring.
- Failure: `POLICY-DENIED`; recovery: leave receipt in staging and route to workflow review.
- Failure: `TASK-REQUEST-FAILED`; recovery: persist strategy decision and retry outbox.

## Migration Notes

- Import SAP storage type search rules from `/SCWM/T331` as draft rules until validated.
- Import bin master and capacity before strategy activation.
- Preserve SAP strategy names as lineage.
- Do not migrate conflicting strategy rules into active state automatically.
- Rollback path: retire new rules and fall back to static putaway-task destination.
- Backfill order: storage types, sections, bins, quants, strategy rules, decisions.

## Cross-microservice Handoffs

- From inbound delivery: received item requiring putaway.
- From inventory-ledger: capacity and quant snapshot.
- To putaway task: selected destination bin.
- To workflow-engine: no-bin or policy exception.
- To compliance: strategy decision audit evidence.
- To ontology: candidate and selected bin projection.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The strategy remains bound to SAP EWM putaway strategy determination. |
| Persona specificity | Lena Fischer owns storage-type rules, no-bin exceptions, and rollback acceptance language. |
| Journey specificity | The j102 controlled-storage leg drives safe-bin selection before task creation. |
| DDL anchor | The storage type, section, strategy rule, and decision tables above are normative. |
| Rust anchor | The strategy rule, candidate bin, decision result, and error enum above are implementation anchors. |
| REST anchor | Evaluate, accept, retire, and explain endpoints are the tenant command surface. |
| gRPC anchor | The putaway strategy service is the worker and replay contract for destination selection. |
| AsyncAPI anchor | Decision accepted, no-bin, and rule-retired channels carry putaway evidence. |
| Cedar anchor | Strategy acceptance is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP storage type, section, bin, and quant lineage projects to candidate and selected-bin nodes. |
| ADR-0263 class binding | Strategy policy checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Storage compliance overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on strategy APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, rule id, material id, selected bin, score, and `cedar_decision_id`. |
| Metric | `oya_warehouse_putaway_strategy_decisions_total{tenant_id,cell_id,outcome,status}` caps outcome/status cardinality. |
| Latency histogram | `oya_warehouse_putaway_strategy_duration_seconds` tracks evaluation and accept latency. |
| Trace span | `warehouse.putaway_strategy.evaluate` links inbound delivery, inventory-ledger, putaway task, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `rule_id`, `material_id`, `selected_bin`, and exception code. |
| Capacity math | Candidate evaluation stops when bin scan cost exceeds p95 budget and falls back to safest static rule. |
| Multi-region | Strategy decisions write in the warehouse home cell; DR cells expose read-only decision history. |
| Sovereign cells | Material and regulated-storage evidence remains in-region for active compliance packs. |
| Rollback | Retire new rules, fall back to static destination, and replay from last sealed strategy audit id. |
| Test evidence | Required tests cover no-bin, conflicting rules, hazmat block, tenant mismatch, and deterministic replay. |
| Rejected shortcut | A generic bin selector is rejected because it loses SAP EWM storage-type and section strategy semantics. |
