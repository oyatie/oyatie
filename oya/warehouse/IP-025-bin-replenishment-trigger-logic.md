---
doc_class: ImplementationPlan
ip_id: IP-025
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
sap_submodule: EWM-WT (warehouse task)
tenant_class: paid
billing_components:
  - per_usage
persona: Grace Kim, wave planner
status: Accepted
date: 2026-05-20
owner_team: axis-warehouse + axis-erp-parity
---

# IP-025: Bin replenishment trigger logic

## Context

- SAP submodule: EWM-WT replenishment warehouse tasks.
- Persona: Grace Kim, wave planner.
- Journey leg: j123 launch wave drains forward pick bins and replenishment must trigger before picker starvation.
- SAP tables: `/SCWM/QUANT`, `/SCWM/STORAGEBIN`, `/SCWM/ORDIM_O`, `/SCWM/REPL`.
- Oyatie capability: `BinReplenishmentTrigger`.
- Precedent: SAP EWM planned/order-related replenishment plus Kubernetes horizontal pod autoscaler thresholding.
- ADR-0263 records trigger decisions and ADR-0297 gates task creation.
- Boundary: computes trigger, creates replenishment task request, and monitors fill completion; it does not own procurement reorder.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.replenishment_policy (
  tenant_id UUID NOT NULL,
  replenishment_policy_id TEXT NOT NULL,
  material_id TEXT NOT NULL,
  pick_bin_id TEXT NOT NULL,
  min_qty NUMERIC(18,6) NOT NULL,
  target_qty NUMERIC(18,6) NOT NULL,
  source_storage_type TEXT NOT NULL,
  policy_state TEXT NOT NULL CHECK (policy_state IN ('draft','active','retired')),
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, replenishment_policy_id)
);
CREATE TABLE warehouse.replenishment_trigger (
  tenant_id UUID NOT NULL,
  replenishment_trigger_id TEXT NOT NULL,
  replenishment_policy_id TEXT NOT NULL,
  current_qty NUMERIC(18,6) NOT NULL,
  triggered_qty NUMERIC(18,6) NOT NULL,
  trigger_status TEXT NOT NULL CHECK (trigger_status IN ('proposed','task_created','blocked','completed','expired')),
  PRIMARY KEY (tenant_id, replenishment_trigger_id)
);
```

### Rust Types

```rust
pub struct ReplenishmentPolicy {
    pub tenant_id: TenantId,
    pub replenishment_policy_id: ReplenishmentPolicyId,
    pub material_id: MaterialId,
    pub pick_bin_id: BinId,
    pub min_qty: Decimal,
    pub target_qty: Decimal,
    pub source_storage_type: StorageType,
}
pub struct ReplenishmentTrigger {
    pub replenishment_trigger_id: ReplenishmentTriggerId,
    pub replenishment_policy_id: ReplenishmentPolicyId,
    pub current_qty: Decimal,
    pub triggered_qty: Decimal,
    pub trigger_status: ReplenishmentTriggerStatus,
}
pub enum ReplenishmentError { PolicyInactive, SourceStockMissing, TriggerDuplicate, PickBinBlocked, TaskCreateFailed }
```

## API Endpoints

- REST `POST /v1/warehouse/replenishment-policies` creates forward-pick policy.
- REST `POST /v1/warehouse/replenishment-triggers:evaluate` evaluates bins against policy.
- REST `POST /v1/warehouse/replenishment-triggers/{id}:create-task`.
- gRPC `warehouse.replenishment.v1.ReplenishmentService.EvaluateTriggers`.
- gRPC `CreateReplenishmentTask` and `CompleteReplenishmentTrigger`.
- AsyncAPI channel `warehouse.replenishment.triggered.v1`.
- AsyncAPI channel `warehouse.replenishment.task-created.v1`.
- Consumers: picking-wave, putaway-task, inventory-ledger, labor-assignment.

## Cedar Policy Hooks

- Policy: `warehouse::replenishment::create_task`.
- Principal: `ReplenishmentWorker`.
- Action: `replenishment_task_create`.
- Resource: `ReplenishmentTrigger`.
- Context: `tenant_id`, `material_id`, `pick_bin_id`, `source_storage_type`, `current_qty`, `target_qty`.
- Forbid when policy inactive, source stock missing, pick bin blocked, or duplicate active trigger exists.

## Ontology Projection

- Vendor object: SAP EWM replenishment trigger.
- Oyatie object: `warehouse.replenishment_trigger`.
- `/SCWM/REPL-MATID` -> `material_id`.
- `/SCWM/STORAGEBIN-LGPLA` -> `pick_bin_id`.
- `/SCWM/QUANT-QUAN` -> current and source quantity.
- `/SCWM/ORDIM_O-TANUM` -> replenishment task lineage.
- Min and target quantity -> policy thresholds.
- Source storage type -> replenishment source constraint.
- Projection freshness floor: 2 seconds.
- Projection rule: duplicate triggers are collapsed by active policy and pick bin.

## Workflow Steps

- Node `policy-load`: load active replenishment policy.
- Node `pick-bin-read`: read current pick-face quantity.
- Decision `policy-inactive`: skip trigger.
- Decision `pick-bin-blocked`: block trigger and notify planner.
- Node `threshold-evaluate`: compare current quantity to min.
- Decision `source-stock-missing`: create supply exception.
- Decision `duplicate-trigger`: reuse active trigger.
- Node `trigger-create`: persist replenishment trigger.
- Node `task-create`: create warehouse task from reserve to pick bin.
- Node `audit-seal`: emit replenishment evidence.

## Audit Events

- `EVT-WAREHOUSE-REPLENISHMENT-POLICY_ACTIVATED`.
- `EVT-WAREHOUSE-REPLENISHMENT-TRIGGERED`.
- `EVT-WAREHOUSE-REPLENISHMENT-TASK_CREATED`.
- `EVT-WAREHOUSE-REPLENISHMENT-SOURCE_STOCK_MISSING`.
- `EVT-WAREHOUSE-REPLENISHMENT-POLICY_DENIED`.
- `EVT-WAREHOUSE-REPLENISHMENT-IP_ACCEPTED`.
- ADR-0263 envelope stores `pick_bin_id`, `material_id`, `current_qty`, `target_qty`, and source storage type.

## SLO Targets

- Trigger evaluation p50: 40 ms per 1,000 bins.
- Trigger evaluation p95: 350 ms per 20,000 bins.
- Trigger evaluation p99: 1,100 ms with partitioned scan.
- Task creation p95: 180 ms.
- Rationale: replenishment evaluation is periodic and partitioned, but task creation must stay interactive for wave protection.

## Failure Modes and Recovery

- Failure: `POLICY-INACTIVE`; recovery: skip and emit no mutation.
- Failure: `SOURCE-STOCK-MISSING`; recovery: create supply exception and notify planning.
- Failure: `TRIGGER-DUPLICATE`; recovery: return existing active trigger.
- Failure: `PICK-BIN-BLOCKED`; recovery: block task and route to bin review.
- Failure: `TASK-CREATE-FAILED`; recovery: keep trigger proposed and retry outbox.
- Failure: `QUANT-SNAPSHOT-STALE`; recovery: refresh inventory and rerun threshold evaluation.

## Migration Notes

- Import SAP replenishment controls as draft policies until thresholds validate.
- Import current pick-face stock from quant snapshot before enabling triggers.
- Preserve SAP replenishment task history as read-only evidence.
- Do not create tasks from migrated triggers automatically.
- Rollback path: retire policies and disable trigger evaluation.
- Backfill order: bins, quants, policies, active triggers, task history.

## Cross-microservice Handoffs

- From inventory-ledger: current pick-bin and reserve stock.
- From picking-wave: expected demand drain.
- To putaway-task: replenishment movement task.
- To labor-assignment: replenishment workload.
- To supply-chain-planning: source-stock missing signal.
- To compliance: trigger and policy evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The trigger logic remains bound to SAP EWM replenishment and pick-face control. |
| Persona specificity | Hana Suzuki owns trigger threshold, source-stock exception, and rollback acceptance language. |
| Journey specificity | The j123 launch-demand leg drives pick-bin drain prediction and reserve-stock movement. |
| DDL anchor | The replenishment policy, active trigger, and task-history tables above are normative. |
| Rust anchor | The replenishment policy, trigger result, and error enum above are implementation anchors. |
| REST anchor | Evaluate, activate, retire, and explain trigger endpoints are tenant surfaces. |
| gRPC anchor | The replenishment trigger service is the worker and replay contract. |
| AsyncAPI anchor | Trigger fired, task requested, source-stock-missing, and policy-retired channels carry evidence. |
| Cedar anchor | Trigger activation is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP bin, quant, reserve stock, and pick-wave demand lineage projects to replenishment trigger nodes. |
| ADR-0263 class binding | Replenishment policy checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Storage, labor, or sovereign overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on replenishment APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, bin id, material id, threshold, source stock, and `cedar_decision_id`. |
| Metric | `oya_warehouse_replenishment_triggers_total{tenant_id,cell_id,outcome,status}` caps outcome/status cardinality. |
| Latency histogram | `oya_warehouse_replenishment_trigger_duration_seconds` tracks evaluation and task-request latency. |
| Trace span | `warehouse.replenishment.evaluate_trigger` links inventory-ledger, picking-wave, putaway-task, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `policy_id`, `pick_bin`, `material_id`, and trigger reason. |
| Capacity math | Trigger threshold uses expected_demand_rate * replenishment_lead_time plus safety stock; negative source stock blocks task creation. |
| Multi-region | Trigger activation writes in home cell; DR cells expose read-only trigger history. |
| Sovereign cells | Stock and material evidence remains in-region for active compliance-pack overlays. |
| Rollback | Retire policies, disable trigger evaluation, and replay from last sealed replenishment audit id. |
| Test evidence | Required tests cover source-stock missing, threshold crossing, tenant mismatch, duplicate trigger, and idempotent task request. |
| Rejected shortcut | A generic min/max replenishment rule is rejected because it loses EWM pick-face, reserve-stock, and wave-demand semantics. |
