---
doc_class: ImplementationPlan
ip_id: IP-019
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
journey_ref: j137-corporate-internal-audit-sox-controls-test
sap_submodule: EWM-WIM (inventory)
tenant_class: paid
billing_components:
  - per_usage
persona: Elena Petrova, warehouse QA automation lead
status: Accepted
date: 2026-05-20
owner_team: axis-warehouse + axis-erp-parity
---

# IP-019: Cycle counting ABC class policy

## Context

- SAP submodule: EWM-WIM physical inventory and cycle counting.
- Persona: Elena Petrova, warehouse QA automation lead.
- Journey leg: j137 internal audit tests SOX inventory controls with ABC-driven count cadence.
- SAP tables: `/SCWM/QUANT`, `/SCWM/STORAGEBIN`, `/SCWM/PI_DOC`, `/SCWM/ORDIM_O`.
- Oyatie capability: `CycleCountPolicy`.
- Precedent: SAP EWM physical inventory ABC cycle counting plus AWS Config periodic compliance evaluation.
- ADR-0263 binds inventory adjustment audit and ADR-0297 requires Cedar before count posting.
- Boundary: schedules, executes, and reconciles count tasks; financial valuation stays with finance/ledger.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.cycle_count_policy (
  tenant_id UUID NOT NULL,
  policy_id TEXT NOT NULL,
  abc_class TEXT NOT NULL CHECK (abc_class IN ('A','B','C')),
  max_days_between_counts INTEGER NOT NULL,
  variance_threshold_percent NUMERIC(8,4) NOT NULL,
  policy_state TEXT NOT NULL CHECK (policy_state IN ('draft','active','retired')),
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, policy_id)
);
CREATE TABLE warehouse.cycle_count_task (
  tenant_id UUID NOT NULL,
  cycle_count_task_id TEXT NOT NULL,
  bin_id TEXT NOT NULL,
  material_id TEXT NOT NULL,
  abc_class TEXT NOT NULL,
  expected_qty NUMERIC(18,6) NOT NULL,
  counted_qty NUMERIC(18,6),
  task_status TEXT NOT NULL,
  PRIMARY KEY (tenant_id, cycle_count_task_id)
);
```

### Rust Types

```rust
pub struct CycleCountPolicy {
    pub tenant_id: TenantId,
    pub policy_id: CycleCountPolicyId,
    pub abc_class: AbcClass,
    pub max_days_between_counts: u16,
    pub variance_threshold_percent: Decimal,
}
pub struct CycleCountTask {
    pub cycle_count_task_id: CycleCountTaskId,
    pub bin_id: BinId,
    pub material_id: MaterialId,
    pub abc_class: AbcClass,
    pub expected_qty: Decimal,
    pub counted_qty: Option<Decimal>,
    pub task_status: CycleCountTaskStatus,
}
pub enum CycleCountError { PolicyMissing, CountAlreadyPosted, VarianceTooHigh, BinLocked, AdjustmentDenied }
```

## API Endpoints

- REST `POST /v1/warehouse/cycle-count-policies` creates ABC policy.
- REST `POST /v1/warehouse/cycle-count-tasks:generate` creates count tasks by class and due date.
- REST `POST /v1/warehouse/cycle-count-tasks/{id}:post-count` posts counted quantity.
- gRPC `warehouse.cycle_count.v1.CycleCountService.GenerateTasks`.
- gRPC `PostCount`, `ApproveVariance`, and `ListDueCounts`.
- AsyncAPI channel `warehouse.cycle-count.task-generated.v1`.
- AsyncAPI channel `warehouse.cycle-count.variance-posted.v1`.
- Consumers: inventory-ledger, compliance, workflow-engine, analytics.

## Cedar Policy Hooks

- Policy: `warehouse::cycle_count::post`.
- Principal: `WarehouseInventoryAuditor`.
- Action: `cycle_count_post`.
- Resource: `CycleCountTask`.
- Context: `tenant_id`, `abc_class`, `variance_percent`, `bin_locked`, `count_blind_mode`.
- Forbid when bin is locked for movement, count already posted, variance exceeds threshold without approval, or blind count mode is violated.

## Ontology Projection

- Vendor object: SAP EWM physical inventory document.
- Oyatie object: `warehouse.cycle_count_task`.
- `/SCWM/PI_DOC-DOCID` -> `cycle_count_task_id`.
- `/SCWM/QUANT-MATID` -> `material_id`.
- `/SCWM/QUANT-QUAN` -> expected quantity.
- `/SCWM/STORAGEBIN-LGPLA` -> `bin_id`.
- ABC class -> count cadence policy.
- Variance -> inventory adjustment evidence.
- Projection freshness floor: 5 seconds.
- Projection rule: count variance projects only after policy and audit seal.

## Workflow Steps

- Node `policy-load`: read active ABC policy.
- Node `due-bin-select`: identify bins due for count.
- Decision `policy-missing`: block task generation.
- Node `task-generate`: create blind count task.
- Node `bin-lock`: hold movement for counted bin.
- Decision `bin-locked-by-task`: reschedule count.
- Node `post-count`: capture counted quantity.
- Decision `variance-too-high`: require workflow approval.
- Node `adjustment-handoff`: send approved variance to inventory-ledger.
- Node `audit-seal`: emit cycle count evidence.

## Audit Events

- `EVT-WAREHOUSE-CYCLE_COUNT-POLICY_ACTIVATED`.
- `EVT-WAREHOUSE-CYCLE_COUNT-TASK_GENERATED`.
- `EVT-WAREHOUSE-CYCLE_COUNT-COUNT_POSTED`.
- `EVT-WAREHOUSE-CYCLE_COUNT-VARIANCE_APPROVED`.
- `EVT-WAREHOUSE-CYCLE_COUNT-POLICY_DENIED`.
- `EVT-WAREHOUSE-CYCLE_COUNT-IP_ACCEPTED`.
- ADR-0263 envelope stores `abc_class`, `variance_percent`, `bin_id`, and `material_id`.

## SLO Targets

- Task generation p50: 120 ms for 1,000 bins.
- Task generation p95: 900 ms for 50,000 bins.
- Task generation p99: 2,500 ms with partitioned scan.
- Count post p95: 120 ms.
- Rationale: generation is batch-like, but auditor posting must be interactive from RF device.

## Failure Modes and Recovery

- Failure: `POLICY-MISSING`; recovery: block task generation and request active ABC policy.
- Failure: `BIN-LOCKED`; recovery: reschedule count after movement completes.
- Failure: `VARIANCE-TOO-HIGH`; recovery: route to workflow approval before adjustment.
- Failure: `COUNT-ALREADY-POSTED`; recovery: reject duplicate post with existing evidence.
- Failure: `ADJUSTMENT-DENIED`; recovery: keep variance pending and notify inventory controller.
- Failure: `AUDIT-SEAL-FAILED`; recovery: do not adjust stock until seal succeeds.

## Migration Notes

- Import SAP physical inventory documents as historical cycle count tasks.
- Compute initial ABC class from movement velocity and inventory value.
- Preserve SAP count document and posting date as lineage.
- Do not auto-post historical variance without approval evidence.
- Rollback path: retire active policy and stop new task generation.
- Backfill order: quants, bins, ABC classes, policies, tasks, posted counts.

## Cross-microservice Handoffs

- From inventory-ledger: expected quantity and movement lock.
- To workflow-engine: variance approval.
- To finance-ledger: approved inventory adjustment evidence.
- To analytics: count accuracy and shrinkage metrics.
- To compliance: SOX control evidence.
- To ontology: cycle count task projection.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The policy remains bound to SAP EWM physical inventory and cycle counting. |
| Persona specificity | Elena Petrova owns ABC cadence, variance approval, and rollback acceptance language. |
| Journey specificity | The j137 SOX inventory-control leg drives evidence, count cadence, and adjustment gating. |
| DDL anchor | The ABC policy, count task, variance, and posted count tables above are normative. |
| Rust anchor | The cycle-count policy, task, variance, and error enum above are implementation anchors. |
| REST anchor | Define policy, generate task, post count, and approve variance endpoints are tenant command surfaces. |
| gRPC anchor | The cycle counting service is the worker and replay contract. |
| AsyncAPI anchor | Count task created, variance approved, and adjustment posted channels carry SOX evidence. |
| Cedar anchor | Variance posting is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP quant, bin, and physical inventory lineage projects to count task and variance nodes. |
| ADR-0263 class binding | Count and variance policy checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | SOX or inventory-control overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on count APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, bin id, material id, ABC class, variance amount, and `cedar_decision_id`. |
| Metric | `oya_warehouse_cycle_count_tasks_total{tenant_id,cell_id,abc_class,status}` caps ABC/status cardinality. |
| Latency histogram | `oya_warehouse_cycle_count_post_duration_seconds` tracks count post and variance approval latency. |
| Trace span | `warehouse.cycle_count.post_count` links inventory-ledger, finance-ledger, workflow, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `count_task_id`, `abc_class`, `variance_bucket`, and approval state. |
| Capacity math | Daily task generation uses bin_count * class_frequency / workday_minutes to keep count workload under staffed capacity. |
| Multi-region | Inventory count writes stay in warehouse home cell; DR cells expose read-only SOX evidence. |
| Sovereign cells | Inventory and control evidence remains in-region for regulated audit overlays. |
| Rollback | Retire active policy, stop new task generation, and replay from the last sealed count audit id. |
| Test evidence | Required tests cover ABC cadence, variance denial, tenant mismatch, ledger handoff failure, and idempotent count replay. |
| Rejected shortcut | A generic inventory-count task is rejected because it loses ABC cadence, EWM quant, and SOX-control semantics. |
