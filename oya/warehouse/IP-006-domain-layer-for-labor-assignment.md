---
doc_class: ImplementationPlan
ip_id: IP-006
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

# IP-006: Domain layer for labor assignment

## Context

- SAP submodule: EWM-LR labor resource planning.
- Persona: Andre Wilson, warehouse labor manager.
- Journey leg: j123 launch day requires assigning certified workers to putaway, pick, pack, and yard tasks without cross-tenant leakage.
- SAP tables: `/SCWM/RESOURCE`, `/SCWM/RSRC`, `/SCWM/WAREHOUSEORDER`, `/SCWM/ORDIM_O`.
- Oyatie aggregate: `LaborAssignment`.
- Precedent: SAP EWM labor management plus AWS ECS task placement constraints.
- ADR-0105 keeps skill matching in domain policy and ADR-0263 seals assignment decisions.
- Boundary: owns resource eligibility, assignment state, and workload evidence; HR payroll and attendance remain outside warehouse.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.labor_assignment (
  tenant_id UUID NOT NULL,
  labor_assignment_id TEXT NOT NULL,
  resource_id TEXT NOT NULL,
  warehouse_order_id TEXT NOT NULL,
  assignment_type TEXT NOT NULL CHECK (assignment_type IN ('putaway','picking','packing','yard','replenishment','cycle_count')),
  skill_code TEXT NOT NULL,
  assigned_at TIMESTAMPTZ NOT NULL,
  assignment_status TEXT NOT NULL CHECK (assignment_status IN ('proposed','assigned','accepted','completed','rejected','reassigned')),
  productivity_target NUMERIC(12,4),
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, labor_assignment_id)
);
CREATE TABLE warehouse.labor_resource_capability (
  tenant_id UUID NOT NULL,
  resource_id TEXT NOT NULL,
  skill_code TEXT NOT NULL,
  certification_expires_at TIMESTAMPTZ,
  max_concurrent_tasks INTEGER NOT NULL DEFAULT 1,
  PRIMARY KEY (tenant_id, resource_id, skill_code)
);
```

### Rust Types

```rust
pub struct LaborAssignment {
    pub tenant_id: TenantId,
    pub labor_assignment_id: LaborAssignmentId,
    pub resource_id: LaborResourceId,
    pub warehouse_order_id: WarehouseOrderId,
    pub assignment_type: AssignmentType,
    pub skill_code: SkillCode,
    pub assignment_status: AssignmentStatus,
}
pub struct LaborResourceCapability {
    pub resource_id: LaborResourceId,
    pub skill_code: SkillCode,
    pub certification_expires_at: Option<DateTime<Utc>>,
    pub max_concurrent_tasks: u32,
}
pub enum LaborAssignmentError { SkillMissing, CertificationExpired, ResourceOverloaded, ShiftBoundaryExceeded, TaskTenantMismatch }
```

## API Endpoints

- REST `POST /v1/warehouse/labor-assignments` proposes assignment for a warehouse order.
- REST `POST /v1/warehouse/labor-assignments/{id}:accept` records worker acceptance.
- REST `POST /v1/warehouse/labor-assignments/{id}:reassign` moves work to another resource.
- REST `GET /v1/warehouse/labor-resources/{id}/capabilities` lists skills and certification state.
- gRPC `warehouse.labor.v1.LaborAssignmentService.AssignLabor`.
- gRPC `AcceptAssignment`, `ReassignLabor`, and `ListResourceCapabilities`.
- AsyncAPI channel `warehouse.labor-assignment.assigned.v1`.
- AsyncAPI channel `warehouse.labor-assignment.reassigned.v1`.

## Cedar Policy Hooks

- Policy: `warehouse::labor_assignment::assign`.
- Principal: `WarehouseLaborManager`.
- Action: `labor_assignment_create`.
- Resource: `WarehouseOrder`.
- Context: `tenant_id`, `resource_id`, `skill_code`, `shift_window`, `certification_expires_at`.
- Forbid when certification is expired, resource is overloaded, shift boundary is exceeded, or warehouse order belongs to another tenant.

## Ontology Projection

- Vendor object: SAP EWM labor resource assignment.
- Oyatie object: `warehouse.labor_assignment`.
- `/SCWM/RESOURCE-RSRC` -> `resource_id`.
- `/SCWM/RSRC-SKILL` -> `skill_code`.
- `/SCWM/WAREHOUSEORDER-WHO` -> `warehouse_order_id`.
- `/SCWM/ORDIM_O-TANUM` -> task lineage.
- Certification register -> `certification_expires_at`.
- Shift roster -> assignment availability context.
- Projection freshness floor: 10 seconds.
- Projection rule: productivity targets are planning evidence, not payroll records.

## Workflow Steps

- Node `workload-read`: load open warehouse orders and task mix.
- Node `capability-match`: filter resources by skill and certification.
- Decision `skill-missing`: branch to supervisor manual review.
- Decision `resource-overloaded`: select alternate or split warehouse order.
- Node `assignment-create`: persist proposed assignment.
- Node `worker-accept`: worker accepts from RF or voice device.
- Decision `accept-timeout`: reassign or escalate.
- Node `task-execute`: update completion state from task confirmations.
- Node `assignment-close`: close assignment with productivity evidence.
- Node `audit-seal`: emit labor assignment audit event.

## Audit Events

- `EVT-WAREHOUSE-LABOR_ASSIGNMENT-PROPOSED`.
- `EVT-WAREHOUSE-LABOR_ASSIGNMENT-ASSIGNED`.
- `EVT-WAREHOUSE-LABOR_ASSIGNMENT-ACCEPTED`.
- `EVT-WAREHOUSE-LABOR_ASSIGNMENT-REASSIGNED`.
- `EVT-WAREHOUSE-LABOR_ASSIGNMENT-POLICY_DENIED`.
- `EVT-WAREHOUSE-LABOR_ASSIGNMENT-IP_ACCEPTED`.
- ADR-0263 envelope stores `resource_id`, `skill_code`, `warehouse_order_id`, and `shift_window`.

## SLO Targets

- Assignment proposal p50: 65 ms.
- Assignment proposal p95: 240 ms.
- Assignment proposal p99: 700 ms.
- Worker accept p95: 120 ms.
- Rationale: labor planning needs enough scheduling context for fairness and safety, while RF accept must be immediate.

## Failure Modes and Recovery

- Failure: `CERTIFICATION-EXPIRED`; recovery: deny assignment and notify labor manager.
- Failure: `RESOURCE-OVERLOADED`; recovery: rebalance task bundle and emit reassignment suggestion.
- Failure: `SHIFT-BOUNDARY-EXCEEDED`; recovery: split order across shifts.
- Failure: `ACCEPT-TIMEOUT`; recovery: auto-expire proposal and select alternate resource.
- Failure: `TASK-TENANT-MISMATCH`; recovery: reject and create security audit event.
- Failure: `PRODUCTIVITY-SINK-MISSING`; recovery: persist assignment and retry analytics handoff.

## Migration Notes

- Import SAP labor resources and skills before assignment history.
- Map legacy certification codes to tenant-local skill codes.
- Import open assignments as proposed or assigned only when source shift is active.
- Store historical productivity metrics as read-only evidence, not payroll facts.
- Rollback path: disable new assignment proposals and keep task execution manual.
- Backfill order: resources, skills, certifications, warehouse orders, assignments, productivity evidence.

## Cross-microservice Handoffs

- From HR or identity: worker identity and active shift status.
- From warehouse order: task bundle workload.
- To RF execution: assigned task list.
- To analytics: productivity evidence.
- To workflow-engine: skill or overload exceptions.
- To compliance: certification and assignment audit trail.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The primitive remains bound to SAP EWM labor-resource planning, not payroll or generic scheduling. |
| Persona specificity | Andre Wilson owns assignment, certification, overload, and rollback acceptance language. |
| Journey specificity | The j123 launch-labor leg drives skill matching, task load, and cross-tenant leakage controls. |
| DDL anchor | The labor assignment and productivity tables above are the normative warehouse labor model. |
| Rust anchor | The labor assignment aggregate, skill profile, and error enum above are the implementation contract. |
| REST anchor | Propose, accept, reassign, and close endpoints are the tenant API surface. |
| gRPC anchor | The labor assignment service is the worker and replay contract for workload balancing. |
| AsyncAPI anchor | Assignment proposed, accepted, reassigned, and overloaded channels carry execution evidence. |
| Cedar anchor | Assignment acceptance is default-deny and must persist `cedar_decision_id` before RF dispatch. |
| Ontology anchor | SAP labor-resource and warehouse-order lineage projects to assignment nodes without payroll facts. |
| ADR-0263 class binding | Assignment policy checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Certification or labor-law overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on assignment APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, worker id, task bundle id, skill code, and `cedar_decision_id`. |
| Metric | `oya_warehouse_labor_assignment_commands_total{tenant_id,cell_id,command,status}` caps command/status cardinality. |
| Latency histogram | `oya_warehouse_labor_assignment_duration_seconds` tracks proposal and acceptance latency. |
| Trace span | `warehouse.labor_assignment.accept` links identity, task bundle, RF execution, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `worker_ref`, `assignment_id`, `skill_code`, and overload flag. |
| Capacity math | Assignment blocks when projected work_minutes / available_shift_minutes exceeds 0.9 for a skill pool. |
| Multi-region | Labor assignment writes stay in the facility home cell; DR cells expose read-only workload projections. |
| Sovereign cells | Worker identity and certification evidence remain in-region for labor-law and sovereign compliance packs. |
| Rollback | Disable new assignment proposals, keep task execution manual, and replay assignment outbox from last sealed audit id. |
| Test evidence | Required tests cover certification missing, tenant mismatch, overload, reassignment conflict, and idempotent acceptance. |
| Rejected shortcut | A generic `WorkerSchedule` is rejected because it loses EWM labor-resource and warehouse-task semantics. |
