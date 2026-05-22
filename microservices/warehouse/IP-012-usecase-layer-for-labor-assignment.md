---
doc_class: ImplementationPlan
ip_id: IP-012
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
sap_submodule: EWM-LR (labor resource)
tenant_class: paid
billing_components:
  - per_usage
persona: Andre Wilson, warehouse labor manager
status: Accepted
date: 2026-05-20
owner_team: axis-warehouse + axis-erp-parity
---

# IP-012: Usecase layer for labor assignment

## Context

- SAP submodule: EWM-LR labor resource execution.
- Persona: Andre Wilson, warehouse labor manager.
- Journey leg: j123 wave launch requires labor assignment acceptance, reassignment, and productivity evidence.
- SAP tables: `/SCWM/RESOURCE`, `/SCWM/RSRC`, `/SCWM/WAREHOUSEORDER`, `/SCWM/ORDIM_O`.
- Oyatie usecase: `AssignWarehouseLabor`.
- Precedent: SAP EWM labor management plus Kubernetes scheduler preemption and rebalance.
- ADR-0244 scopes resources per tenant and ADR-0263 binds assignment events.
- Boundary: orchestrates assignment proposal, worker acceptance, reassignment, completion, and analytics handoff.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.labor_command (
  tenant_id UUID NOT NULL,
  command_id TEXT NOT NULL,
  labor_assignment_id TEXT NOT NULL,
  command_kind TEXT NOT NULL CHECK (command_kind IN ('propose','accept','reject','reassign','complete')),
  idempotency_key TEXT NOT NULL,
  command_status TEXT NOT NULL,
  failure_code TEXT,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, command_id),
  UNIQUE (tenant_id, idempotency_key)
);
CREATE TABLE warehouse.labor_productivity_evidence (
  tenant_id UUID NOT NULL,
  labor_assignment_id TEXT NOT NULL,
  warehouse_order_id TEXT NOT NULL,
  completed_task_count INTEGER NOT NULL,
  actual_duration_seconds INTEGER NOT NULL,
  productivity_score NUMERIC(10,4),
  PRIMARY KEY (tenant_id, labor_assignment_id)
);
```

### Rust Types

```rust
pub struct LaborCommand {
    pub tenant_id: TenantId,
    pub command_id: CommandId,
    pub labor_assignment_id: LaborAssignmentId,
    pub command_kind: LaborCommandKind,
    pub idempotency_key: IdempotencyKey,
    pub command_status: CommandStatus,
}
pub struct LaborProductivityEvidence {
    pub warehouse_order_id: WarehouseOrderId,
    pub completed_task_count: u32,
    pub actual_duration_seconds: u32,
    pub productivity_score: Option<Decimal>,
}
pub enum AssignWarehouseLaborError { ResourceUnavailable, SkillDenied, AcceptanceExpired, ReassignConflict, ProductivitySinkFailed }
```

## API Endpoints

- REST `POST /v1/warehouse/labor-assignments/{id}:operate` executes labor command.
- REST `POST /v1/warehouse/labor-assignments/{id}:complete` closes assignment with evidence.
- REST `GET /v1/warehouse/labor-commands/{command_id}` returns command status.
- gRPC `warehouse.labor_usecase.v1.AssignWarehouseLabor`.
- gRPC `OperateLaborAssignment`, `CompleteLaborAssignment`, and `GetLaborCommand`.
- AsyncAPI channel `warehouse.labor-command.succeeded.v1`.
- AsyncAPI channel `warehouse.labor-productivity.recorded.v1`.
- Consumers: picking-wave, putaway-task, analytics, compliance.

## Cedar Policy Hooks

- Policy: `warehouse::labor_command::operate`.
- Principal: `WarehouseLaborManager`.
- Action: `labor_command_execute`.
- Resource: `LaborAssignment`.
- Context: `tenant_id`, `command_kind`, `resource_id`, `skill_code`, `shift_window`, `warehouse_order_id`.
- Forbid when resource is unavailable, skill is denied, assignment acceptance expired, or reassignment would strand an active task.

## Ontology Projection

- Vendor object: SAP EWM labor command.
- Oyatie object: `warehouse.labor_command`.
- `/SCWM/RESOURCE-RSRC` -> `resource_id`.
- `/SCWM/RSRC-SKILL` -> skill and certification evidence.
- `/SCWM/WAREHOUSEORDER-WHO` -> `warehouse_order_id`.
- `/SCWM/ORDIM_O-TANUM` -> completed task count lineage.
- Command kind -> assignment state mutation.
- Productivity evidence -> analytics projection.
- Projection freshness floor: 10 seconds.
- Projection rule: productivity score is operational analytics, not payroll compensation.

## Workflow Steps

- Node `command-accept`: dedupe labor command.
- Node `availability-check`: verify shift and open workload.
- Decision `resource-unavailable`: select alternate worker.
- Decision `skill-denied`: route to supervisor approval.
- Node `proposal-send`: send assignment to worker device.
- Decision `acceptance-expired`: expire proposal and reassign.
- Node `assignment-update`: apply accepted/rejected/reassigned state.
- Node `productivity-record`: write completion evidence.
- Decision `sink-failed`: retry analytics handoff.
- Node `audit-seal`: close command evidence.

## Audit Events

- `EVT-WAREHOUSE-LABOR_COMMAND-PROPOSED`.
- `EVT-WAREHOUSE-LABOR_COMMAND-ACCEPTED`.
- `EVT-WAREHOUSE-LABOR_COMMAND-REASSIGNED`.
- `EVT-WAREHOUSE-LABOR_PRODUCTIVITY-RECORDED`.
- `EVT-WAREHOUSE-LABOR_COMMAND-POLICY_DENIED`.
- `EVT-WAREHOUSE-LABOR_COMMAND-IP_ACCEPTED`.
- ADR-0263 envelope stores `resource_id`, `skill_code`, `warehouse_order_id`, and productivity evidence reference.

## SLO Targets

- Labor command p50: 45 ms.
- Labor command p95: 170 ms.
- Labor command p99: 480 ms.
- Completion evidence p95: 250 ms.
- Rationale: assignment is interactive during shift work; analytics can lag without blocking operators.

## Failure Modes and Recovery

- Failure: `RESOURCE-UNAVAILABLE`; recovery: re-plan assignment candidate list.
- Failure: `SKILL-DENIED`; recovery: block and request certified worker.
- Failure: `ACCEPTANCE-EXPIRED`; recovery: auto-expire proposal and reassign.
- Failure: `REASSIGN-CONFLICT`; recovery: keep current assignment and create review task.
- Failure: `PRODUCTIVITY-SINK-FAILED`; recovery: retry outbox without changing assignment state.
- Failure: `SHIFT-DATA-STALE`; recovery: refresh workforce context and repeat policy evaluation.

## Migration Notes

- Import active SAP assignments only when worker and shift identity can be resolved.
- Import historical labor productivity as read-only operational evidence.
- Map legacy skill codes to tenant-local skill catalog.
- Keep payroll-sensitive fields out of warehouse migration.
- Rollback path: disable labor command endpoint and keep task assignment manual.
- Backfill order: resources, skills, warehouse orders, assignments, commands, productivity.

## Cross-microservice Handoffs

- From identity or HR: active resource and shift state.
- From putaway/picking: task workload.
- To RF execution: assigned worker context.
- To analytics: productivity and capacity evidence.
- To workflow-engine: approval and reassignment exceptions.
- To compliance: skill and certification audit evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The usecase remains bound to SAP EWM labor-resource execution. |
| Persona specificity | Andre Wilson owns assignment acceptance, reassignment, productivity, and rollback language. |
| Journey specificity | The j123 launch-wave labor leg drives task workload, skill checks, and worker-context handoff. |
| DDL anchor | The labor command and productivity evidence tables above are the normative usecase model. |
| Rust anchor | The labor command, assignment result, and error enum above are the implementation contract. |
| REST anchor | Assign, accept, reassign, and close endpoints are the tenant command surface. |
| gRPC anchor | The labor assignment usecase service is the worker and replay contract. |
| AsyncAPI anchor | Assignment accepted, reassigned, and productivity channels carry analytics and compliance evidence. |
| Cedar anchor | Worker assignment commands are default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP labor-resource and warehouse-order lineage projects to worker-task assignment nodes. |
| ADR-0263 class binding | Labor command checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Certification or labor-law overlays emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on labor APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, worker id, skill code, assignment id, and `cedar_decision_id`. |
| Metric | `oya_warehouse_labor_usecase_commands_total{tenant_id,cell_id,command,status}` caps command/status cardinality. |
| Latency histogram | `oya_warehouse_labor_usecase_duration_seconds` tracks assignment and reassignment command latency. |
| Trace span | `warehouse.labor_assignment.assign_worker` links identity, task workload, RF execution, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `worker_ref`, `task_bundle_id`, `skill_code`, and productivity bucket. |
| Capacity math | Skill-pool assignment blocks when committed workload exceeds available certified minutes by more than 5 percent. |
| Multi-region | Facility home cell owns assignment writes; DR cells expose read-only labor projections. |
| Sovereign cells | Worker identity and certification evidence remains in-region for labor and sovereign packs. |
| Rollback | Disable labor command endpoint, keep manual task assignment, and replay from the last sealed assignment audit id. |
| Test evidence | Required tests cover certification missing, overload, reassignment race, tenant mismatch, and idempotent replay. |
| Rejected shortcut | A generic `TaskAssignee` usecase is rejected because it loses EWM labor-resource and certification semantics. |
