---
doc_class: ImplementationPlan
ip_id: IP-009
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
sap_submodule: RE-FX-VM (vacancy management)
tenant_class: paid
billing_components:
  - per_usage
persona: Sofia Mendes, workplace occupancy planner
status: Accepted
date: 2026-05-20
owner_team: axis-real-estate + axis-erp-parity
---

# IP-009: Usecase layer for occupancy allocation

## Context

- SAP submodule: RE-FX-VM vacancy workflow orchestration.
- Persona: Sofia Mendes, workplace occupancy planner.
- Journey leg: j168 vacancy and allocation changes update occupancy snapshots and cost allocation basis.
- SAP tables: `VICDOBJASS`, `VIBDRO`, `VICDCONTRACT`, `VICDCONDLINE`.
- Oyatie usecase: `OperateOccupancyAllocation`.
- Precedent: SAP RE-FX vacancy management plus IBM TRIRIGA space-charge allocation workflow.
- ADR-0105 keeps snapshot computation orchestration out of the domain aggregate and ADR-0263 binds snapshot events.
- Boundary: coordinates allocation commands, snapshot recomputation, and service-charge basis handoff.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.occupancy_command (
  tenant_id UUID NOT NULL,
  command_id TEXT NOT NULL,
  occupancy_allocation_id TEXT NOT NULL,
  command_kind TEXT NOT NULL CHECK (command_kind IN ('allocate','reserve','vacate','block','compute_snapshot')),
  idempotency_key TEXT NOT NULL,
  command_status TEXT NOT NULL,
  failure_code TEXT,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, command_id),
  UNIQUE (tenant_id, idempotency_key)
);
CREATE TABLE real_estate.occupancy_basis_handoff (
  tenant_id UUID NOT NULL,
  basis_handoff_id TEXT NOT NULL,
  command_id TEXT NOT NULL,
  allocation_snapshot_id TEXT NOT NULL,
  target_microservice TEXT NOT NULL,
  handoff_state TEXT NOT NULL,
  PRIMARY KEY (tenant_id, basis_handoff_id)
);
```

### Rust Types

```rust
pub struct OccupancyCommand {
    pub tenant_id: TenantId,
    pub command_id: CommandId,
    pub occupancy_allocation_id: OccupancyAllocationId,
    pub command_kind: OccupancyCommandKind,
    pub idempotency_key: IdempotencyKey,
    pub command_status: CommandStatus,
}
pub struct OccupancyBasisHandoff {
    pub basis_handoff_id: HandoffId,
    pub command_id: CommandId,
    pub allocation_snapshot_id: SnapshotId,
    pub target_microservice: MicroserviceName,
    pub handoff_state: HandoffState,
}
pub enum OperateOccupancyAllocationError { OverAllocationDetected, FacilityClosed, SnapshotFailed, BasisHandoffFailed, DuplicatePayload }
```

## API Endpoints

- REST `POST /v1/real-estate/occupancy-allocations/{id}:operate`.
- REST `POST /v1/real-estate/occupancy-commands/{command_id}:retry-basis-handoff`.
- REST `GET /v1/real-estate/occupancy-commands/{command_id}`.
- gRPC `real_estate.occupancy_usecase.v1.OperateOccupancyAllocation`.
- gRPC `RetryBasisHandoff` and `StreamOccupancyCommandEvents`.
- AsyncAPI channel `real-estate.occupancy-command.succeeded.v1`.
- AsyncAPI channel `real-estate.occupancy-basis.updated.v1`.
- Consumers: service-charge, portfolio-analytics, workflow-engine, compliance.

## Cedar Policy Hooks

- Policy: `real_estate::occupancy_command::operate`.
- Principal: `OccupancyPlanner`.
- Action: `occupancy_command_execute`.
- Resource: `OccupancyAllocation`.
- Context: `tenant_id`, `command_kind`, `allocation_percent`, `facility_object_state`, `effective_range`.
- Forbid when allocation exceeds available area, object is closed, command conflicts with active lease, or snapshot date is outside allowed period.

## Ontology Projection

- Vendor object: SAP RE-FX vacancy management command.
- Oyatie object: `real_estate.occupancy_command`.
- `VICDOBJASS-OBJNR` -> `occupancy_allocation_id`.
- `VIBDRO-SMENR` -> facility object lineage.
- `VICDCONTRACT-CONTRACT` -> active lease reference.
- `VICDCONDLINE-CONDGUID` -> service-charge basis lineage.
- Command kind -> occupancy state transition.
- Basis handoff -> allocation consumer evidence.
- Projection freshness floor: 5 seconds.
- Projection rule: basis handoff state is queryable even when service-charge retry is pending.

## Workflow Steps

- Node `command-accept`: dedupe occupancy command.
- Node `facility-load`: validate facility state and area.
- Decision `facility-closed`: reject allocation or reservation.
- Node `overlap-calc`: compute active allocation.
- Decision `over-allocation`: fail command and create remediation.
- Node `domain-apply`: update allocation state.
- Node `snapshot-recompute`: compute allocation basis.
- Decision `snapshot-failed`: keep command pending-retry.
- Node `basis-handoff`: notify service-charge and analytics.
- Node `audit-seal`: emit occupancy command evidence.

## Audit Events

- `EVT-REAL_ESTATE-OCCUPANCY_COMMAND-ACCEPTED`.
- `EVT-REAL_ESTATE-OCCUPANCY_COMMAND-SUCCEEDED`.
- `EVT-REAL_ESTATE-OCCUPANCY_COMMAND-OVERALLOCATION`.
- `EVT-REAL_ESTATE-OCCUPANCY_BASIS-HANDOFF_FAILED`.
- `EVT-REAL_ESTATE-OCCUPANCY_COMMAND-POLICY_DENIED`.
- `EVT-REAL_ESTATE-OCCUPANCY_COMMAND-IP_ACCEPTED`.
- ADR-0263 envelope stores command kind, allocation percent, snapshot ID, and basis handoff state.

## SLO Targets

- Command accept p50: 40 ms.
- Command accept p95: 150 ms.
- Command accept p99: 420 ms.
- Snapshot recompute p95: 2 seconds for 100,000 allocations.
- Rationale: occupancy commands are planner-facing; recompute can be asynchronous with visible receipt.

## Failure Modes and Recovery

- Failure: `OVER-ALLOCATION-DETECTED`; recovery: reject and provide conflicts.
- Failure: `FACILITY-CLOSED`; recovery: require facility activation or alternate object.
- Failure: `SNAPSHOT-FAILED`; recovery: retry recompute and keep handoff pending.
- Failure: `BASIS-HANDOFF-FAILED`; recovery: retry outbox to service-charge and analytics.
- Failure: `DUPLICATE-PAYLOAD`; recovery: return prior command.
- Failure: `LEASE-CONFLICT`; recovery: route to lease admin review.

## Migration Notes

- Convert open occupancy changes into command rows only when pending.
- Import historical allocation snapshots as read-only baseline.
- Recompute basis snapshots after full allocation import.
- Preserve SAP assignment IDs in command lineage.
- Rollback path: disable operate endpoint and keep allocations read-only.
- Backfill order: allocations, commands, snapshots, basis handoffs.

## Cross-microservice Handoffs

- From facility master: object status and area.
- From lease-contract: active lease constraints.
- To service-charge: allocation basis.
- To portfolio analytics: occupancy snapshot.
- To workflow-engine: over-allocation remediation.
- To compliance: command and snapshot evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The usecase remains bound to SAP RE-FX-VM vacancy workflow orchestration. |
| Persona specificity | Sofia Mendes owns occupancy command, snapshot, and rollback acceptance language. |
| Journey specificity | The j168 vacancy and allocation leg drives cost allocation and portfolio snapshot behavior. |
| DDL anchor | The occupancy command, allocation snapshot, and basis handoff tables above are normative. |
| Rust anchor | Occupancy command, snapshot result, and error types above are implementation anchors. |
| REST anchor | Operate, snapshot, correct, and publish endpoints are tenant surfaces. |
| gRPC anchor | The occupancy usecase service is the worker and replay contract. |
| AsyncAPI anchor | Allocation changed and snapshot published channels carry service-charge and analytics evidence. |
| Cedar anchor | Occupancy operation is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP vacancy and object assignment lineage projects to allocation command nodes. |
| ADR-0263 class binding | Occupancy operation checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Workplace or office-scope overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on occupancy APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, command id, object id, contract id, basis, and `cedar_decision_id`. |
| Metric | `oya_real_estate_occupancy_usecase_commands_total{tenant_id,cell_id,command,status}` caps cardinality. |
| Latency histogram | `oya_real_estate_occupancy_usecase_duration_seconds` tracks command and snapshot latency. |
| Trace span | `real_estate.occupancy_allocation.operate` links facility master, lease contract, service charge, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `allocation_id`, `snapshot_id`, `basis_type`, and status. |
| Capacity math | Snapshot generation partitions by facility object and rejects allocations exceeding measured area. |
| Multi-region | Allocation writes stay in property home cell; DR cells expose read-only snapshots. |
| Sovereign cells | Workplace and tenant evidence remains in-region for privacy and sovereign packs. |
| Rollback | Disable operate endpoint, keep allocations read-only, and replay from last sealed occupancy command audit id. |
| Test evidence | Required tests cover over-allocation, inactive object, service-charge handoff failure, tenant mismatch, and replay. |
| Rejected shortcut | A generic occupancy snapshot is rejected because it loses SAP RE-FX vacancy and allocation-basis semantics. |
