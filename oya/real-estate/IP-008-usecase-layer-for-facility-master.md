---
doc_class: ImplementationPlan
ip_id: IP-008
microservice: real-estate
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
sap_submodule: RE-FX-AS (architectural objects)
tenant_class: paid
billing_components:
  - per_usage
persona: Tobias Klein, corporate real-estate data steward
status: Accepted
date: 2026-05-20
owner_team: axis-real-estate + axis-erp-parity
---

# IP-008: Usecase layer for facility master

## Context

- SAP submodule: RE-FX-AS architectural object orchestration.
- Persona: Tobias Klein, corporate real-estate data steward.
- Journey leg: j168 facility hierarchy updates must propagate to lease, occupancy, maintenance, and analytics views.
- SAP tables: `VIBDBU`, `VIBDRO`, `VICDOBJASS`, `VICDCONTRACT`.
- Oyatie usecase: `OperateFacilityMaster`.
- Precedent: SAP RE-FX architectural hierarchy maintenance plus digital-twin graph synchronization.
- ADR-0105 places graph mutation orchestration in usecase layer and ADR-0263 binds object changes.
- Boundary: validates hierarchy, updates object state, publishes projections, and schedules downstream re-index.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.facility_master_command (
  tenant_id UUID NOT NULL,
  command_id TEXT NOT NULL,
  facility_object_id TEXT NOT NULL,
  command_kind TEXT NOT NULL CHECK (command_kind IN ('create','amend','move','retire','record_area')),
  idempotency_key TEXT NOT NULL,
  command_status TEXT NOT NULL,
  failure_code TEXT,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, command_id),
  UNIQUE (tenant_id, idempotency_key)
);
CREATE TABLE real_estate.facility_projection_job (
  tenant_id UUID NOT NULL,
  projection_job_id TEXT NOT NULL,
  command_id TEXT NOT NULL,
  projection_kind TEXT NOT NULL,
  job_state TEXT NOT NULL,
  PRIMARY KEY (tenant_id, projection_job_id)
);
```

### Rust Types

```rust
pub struct FacilityMasterCommand {
    pub tenant_id: TenantId,
    pub command_id: CommandId,
    pub facility_object_id: FacilityObjectId,
    pub command_kind: FacilityCommandKind,
    pub idempotency_key: IdempotencyKey,
    pub command_status: CommandStatus,
}
pub struct FacilityProjectionJob {
    pub projection_job_id: ProjectionJobId,
    pub command_id: CommandId,
    pub projection_kind: FacilityProjectionKind,
    pub job_state: ProjectionJobState,
}
pub enum OperateFacilityMasterError { HierarchyInvalid, ActiveAssignmentBlocksMove, AreaMeasurementDenied, ProjectionFailed, DuplicatePayload }
```

## API Endpoints

- REST `POST /v1/real-estate/facility-objects/{id}:operate`.
- REST `GET /v1/real-estate/facility-master-commands/{command_id}`.
- REST `POST /v1/real-estate/facility-master-commands/{command_id}:retry-projection`.
- gRPC `real_estate.facility_usecase.v1.OperateFacilityMaster`.
- gRPC `GetFacilityMasterCommand` and `RetryFacilityProjection`.
- AsyncAPI channel `real-estate.facility-master.command-succeeded.v1`.
- AsyncAPI channel `real-estate.facility-master.projection-failed.v1`.
- Consumers: occupancy-allocation, lease-contract, plant-maintenance, portfolio-analytics.

## Cedar Policy Hooks

- Policy: `real_estate::facility_master_command::operate`.
- Principal: `FacilityDataSteward`.
- Action: `facility_master_command_execute`.
- Resource: `FacilityObject`.
- Context: `tenant_id`, `command_kind`, `object_type`, `parent_object_id`, `active_assignment_count`.
- Forbid when move or retire would break active assignment, hierarchy is invalid, or area measurement lacks source evidence.

## Ontology Projection

- Vendor object: SAP RE-FX architectural object mutation.
- Oyatie object: `real_estate.facility_master_command`.
- `VIBDBU-SWENR` -> building command target.
- `VIBDRO-SMENR` -> room/unit command target.
- `VICDOBJASS-OBJNR` -> active assignment blocker.
- `VICDCONTRACT-CONTRACT` -> lease blocker lineage.
- Command kind -> hierarchy state transition.
- Projection job -> ontology and analytics refresh evidence.
- Projection freshness floor: 10 seconds.
- Projection rule: graph changes emit projection jobs even if downstream read models lag.

## Workflow Steps

- Node `command-accept`: dedupe facility command.
- Node `hierarchy-validate`: check parent and cycle constraints.
- Decision `hierarchy-invalid`: reject command.
- Decision `assignment-blocks-move`: route to occupancy review.
- Node `domain-apply`: update facility object or area.
- Node `projection-job-create`: schedule ontology and analytics projections.
- Decision `projection-failed`: keep command succeeded and retry job.
- Node `maintenance-sync`: notify plant-maintenance.
- Node `contract-impact-sync`: notify lease-contract when object changes.
- Node `audit-seal`: emit command evidence.

## Audit Events

- `EVT-REAL_ESTATE-FACILITY_MASTER_COMMAND-ACCEPTED`.
- `EVT-REAL_ESTATE-FACILITY_MASTER_COMMAND-SUCCEEDED`.
- `EVT-REAL_ESTATE-FACILITY_MASTER_COMMAND-HIERARCHY_DENIED`.
- `EVT-REAL_ESTATE-FACILITY_MASTER-PROJECTION_FAILED`.
- `EVT-REAL_ESTATE-FACILITY_MASTER-POLICY_DENIED`.
- `EVT-REAL_ESTATE-FACILITY_MASTER-IP_ACCEPTED`.
- ADR-0263 envelope stores `command_kind`, `facility_object_id`, parent, and projection jobs.

## SLO Targets

- Command accept p50: 35 ms.
- Command accept p95: 130 ms.
- Command accept p99: 360 ms.
- Projection job completion p95: 2 seconds for 10,000 object graph edges.
- Rationale: steward commands are interactive; projection refresh can be asynchronous but must keep dashboards fresh.

## Failure Modes and Recovery

- Failure: `HIERARCHY-INVALID`; recovery: reject with parent/cycle diagnostic.
- Failure: `ACTIVE-ASSIGNMENT-BLOCKS-MOVE`; recovery: request occupancy reassignment workflow.
- Failure: `AREA-MEASUREMENT-DENIED`; recovery: require source measurement evidence.
- Failure: `PROJECTION-FAILED`; recovery: retry projection job.
- Failure: `DUPLICATE-PAYLOAD`; recovery: return prior command receipt.
- Failure: `MAINTENANCE-SYNC-FAILED`; recovery: retry downstream outbox.

## Migration Notes

- Convert open facility hierarchy corrections into command rows only if pending.
- Import historical object hierarchy directly as read-only baseline.
- Preserve source object move history where available.
- Do not replay historical projection jobs.
- Rollback path: disable operate endpoint and keep facility master read-only.
- Backfill order: objects, hierarchy, area measurements, commands, projection jobs.

## Cross-microservice Handoffs

- To occupancy-allocation: changed object hierarchy and area.
- To lease-contract: object assignment impact.
- To plant-maintenance: maintainable object update.
- To portfolio analytics: object graph refresh.
- To workflow-engine: blocked move review.
- To compliance: hierarchy mutation evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The usecase remains bound to SAP RE-FX-AS architectural object orchestration. |
| Persona specificity | Tobias Klein owns hierarchy mutation, projection, and rollback acceptance language. |
| Journey specificity | The j168 facility hierarchy leg drives lease, occupancy, maintenance, and analytics propagation. |
| DDL anchor | The facility command, hierarchy version, and projection job tables above are normative. |
| Rust anchor | Facility command, projection result, and error types above are implementation anchors. |
| REST anchor | Operate, move object, measure area, and publish projection endpoints are tenant surfaces. |
| gRPC anchor | The facility master usecase service is the worker and replay contract. |
| AsyncAPI anchor | Hierarchy changed and projection refreshed channels carry dependent-service evidence. |
| Cedar anchor | Facility operation is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP architectural object lineage projects to hierarchy command and projection nodes. |
| ADR-0263 class binding | Facility operation checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Office-scope or building overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on facility usecase APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, command id, object id, hierarchy version, and `cedar_decision_id`. |
| Metric | `oya_real_estate_facility_master_usecase_commands_total{tenant_id,cell_id,command,status}` caps cardinality. |
| Latency histogram | `oya_real_estate_facility_master_usecase_duration_seconds` tracks mutation-to-projection latency. |
| Trace span | `real_estate.facility_master.operate` links lease contract, occupancy, maintenance, analytics, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `facility_object_id`, `command_id`, and projection version. |
| Capacity math | Projection jobs batch descendants by hierarchy depth; fan-out above threshold moves to background worker. |
| Multi-region | Hierarchy writes stay in property home cell; DR cells expose read-only projection snapshots. |
| Sovereign cells | Site and premises evidence remains in-region for active pack overlays. |
| Rollback | Disable operate endpoint, keep facility master read-only, and replay from last sealed hierarchy audit id. |
| Test evidence | Required tests cover blocked move, area mismatch, projection failure, tenant mismatch, and idempotent replay. |
| Rejected shortcut | A generic facility update usecase is rejected because it loses SAP RE-FX object hierarchy semantics. |
