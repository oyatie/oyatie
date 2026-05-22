---
doc_class: ImplementationPlan
ip_id: IP-003
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

# IP-003: Domain layer for occupancy allocation

## Context

- SAP submodule: RE-FX-VM vacancy and occupancy management.
- Persona: Sofia Mendes, workplace occupancy planner.
- Journey leg: j168 COO review needs accurate occupied, vacant, reserved, and subleased space allocation.
- SAP tables: `VICDOBJASS`, `VIBDRO`, `VICDCONTRACT`, `VICDCONDLINE`.
- Oyatie aggregate: `OccupancyAllocation`.
- Precedent: SAP RE-FX occupancy/vacancy object assignment plus IBM TRIRIGA space allocation.
- ADR-0244 scopes assignments by tenant and ADR-0263 records occupancy changes.
- Boundary: owns space assignment, vacancy state, and allocation basis; HR seating roster and lease accounting remain separate.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.occupancy_allocation (
  tenant_id UUID NOT NULL,
  occupancy_allocation_id TEXT NOT NULL,
  facility_object_id TEXT NOT NULL,
  lease_contract_id TEXT,
  occupying_party_id TEXT,
  allocation_state TEXT NOT NULL CHECK (allocation_state IN ('vacant','reserved','occupied','subleased','blocked')),
  allocation_percent NUMERIC(8,4) NOT NULL,
  effective_from DATE NOT NULL,
  effective_to DATE,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, occupancy_allocation_id)
);
CREATE TABLE real_estate.occupancy_allocation_snapshot (
  tenant_id UUID NOT NULL,
  snapshot_id TEXT NOT NULL,
  snapshot_date DATE NOT NULL,
  vacant_area_sqm NUMERIC(14,4) NOT NULL,
  occupied_area_sqm NUMERIC(14,4) NOT NULL,
  reserved_area_sqm NUMERIC(14,4) NOT NULL,
  PRIMARY KEY (tenant_id, snapshot_id)
);
```

### Rust Types

```rust
pub struct OccupancyAllocation {
    pub tenant_id: TenantId,
    pub occupancy_allocation_id: OccupancyAllocationId,
    pub facility_object_id: FacilityObjectId,
    pub lease_contract_id: Option<LeaseContractId>,
    pub occupying_party_id: Option<PartyId>,
    pub allocation_state: OccupancyState,
    pub allocation_percent: Decimal,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
}
pub struct OccupancySnapshot {
    pub snapshot_id: SnapshotId,
    pub snapshot_date: NaiveDate,
    pub vacant_area_sqm: Decimal,
    pub occupied_area_sqm: Decimal,
    pub reserved_area_sqm: Decimal,
}
pub enum OccupancyAllocationError { OverAllocated, FacilityInactive, ContractInactive, DateOverlap, PolicyDenied }
```

## API Endpoints

- REST `POST /v1/real-estate/occupancy-allocations` creates allocation.
- REST `POST /v1/real-estate/occupancy-allocations/{id}:vacate`.
- REST `POST /v1/real-estate/occupancy-snapshots:compute`.
- gRPC `real_estate.occupancy.v1.OccupancyAllocationService.CreateAllocation`.
- gRPC `VacateAllocation`, `ComputeSnapshot`, and `ListOccupancyByFacility`.
- AsyncAPI channel `real-estate.occupancy-allocation.created.v1`.
- AsyncAPI channel `real-estate.occupancy-snapshot.computed.v1`.
- Consumers: facility-master, lease-contract, portfolio-analytics, compliance.

## Cedar Policy Hooks

- Policy: `real_estate::occupancy::allocate`.
- Principal: `OccupancyPlanner`.
- Action: `occupancy_allocate`.
- Resource: `FacilityObject`.
- Context: `tenant_id`, `allocation_percent`, `allocation_state`, `lease_contract_id`, `effective_range`.
- Forbid when object inactive, effective dates overlap beyond 100 percent allocation, lease inactive, or party belongs to another tenant.

## Ontology Projection

- Vendor object: SAP RE-FX object assignment.
- Oyatie object: `real_estate.occupancy_allocation`.
- `VICDOBJASS-OBJNR` -> `facility_object_id`.
- `VICDCONTRACT-CONTRACT` -> `lease_contract_id`.
- `VIBDRO-SMENR` -> room or unit lineage.
- `VICDCONDLINE-CONDGUID` -> allocation cost condition lineage.
- Occupancy state -> vacancy management status.
- Allocation percent -> area apportionment.
- Projection freshness floor: 5 seconds.
- Projection rule: allocation history remains temporal and never overwritten in place.

## Workflow Steps

- Node `facility-read`: validate active object and area.
- Node `contract-read`: validate active lease if present.
- Decision `facility-inactive`: reject allocation.
- Decision `contract-inactive`: route to lease admin review.
- Node `overlap-check`: calculate active allocation percent.
- Decision `over-allocated`: reject or split allocation.
- Node `allocation-create`: persist time-effective assignment.
- Node `snapshot-update`: update occupancy snapshot.
- Node `analytics-publish`: send vacancy and occupancy metrics.
- Node `audit-seal`: emit occupancy event.

## Audit Events

- `EVT-REAL_ESTATE-OCCUPANCY_ALLOCATION-CREATED`.
- `EVT-REAL_ESTATE-OCCUPANCY_ALLOCATION-VACATED`.
- `EVT-REAL_ESTATE-OCCUPANCY_ALLOCATION-OVERALLOCATED`.
- `EVT-REAL_ESTATE-OCCUPANCY_SNAPSHOT-COMPUTED`.
- `EVT-REAL_ESTATE-OCCUPANCY_ALLOCATION-POLICY_DENIED`.
- `EVT-REAL_ESTATE-OCCUPANCY_ALLOCATION-IP_ACCEPTED`.
- ADR-0263 envelope stores `facility_object_id`, allocation percent, effective range, and lease reference.

## SLO Targets

- Allocation create p50: 50 ms.
- Allocation create p95: 190 ms.
- Allocation create p99: 500 ms.
- Snapshot compute p95: 2 seconds for 100,000 allocations.
- Rationale: planner writes are interactive; portfolio snapshots are batch but need dashboard freshness.

## Failure Modes and Recovery

- Failure: `OVER-ALLOCATED`; recovery: reject and return conflicting allocations.
- Failure: `FACILITY-INACTIVE`; recovery: require facility activation.
- Failure: `CONTRACT-INACTIVE`; recovery: route to lease-contract amendment.
- Failure: `DATE-OVERLAP`; recovery: split or end existing allocation.
- Failure: `SNAPSHOT-COMPUTE-FAILED`; recovery: keep allocation and retry batch snapshot.
- Failure: `POLICY-DENIED`; recovery: preserve attempted allocation in audit stream.

## Migration Notes

- Import facility objects before allocation.
- Import SAP object assignments as temporal allocation history.
- Infer vacant periods only after full assignment timeline is loaded.
- Preserve SAP assignment numbers as lineage.
- Rollback path: disable allocation mutations and retain read-only occupancy history.
- Backfill order: facility objects, contracts, object assignments, snapshots.

## Cross-microservice Handoffs

- From facility master: object hierarchy and area.
- From lease contract: active lease state.
- To service-charge: allocation basis.
- To portfolio analytics: vacancy and occupancy metrics.
- To workflow-engine: over-allocation remediation tasks.
- To compliance: occupancy audit evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The primitive remains bound to SAP RE-FX-VM vacancy management. |
| Persona specificity | Sofia Mendes owns occupancy allocation, vacancy evidence, and rollback acceptance language. |
| Journey specificity | The j168 COO review leg drives occupied, vacant, reserved, and subleased space evidence. |
| DDL anchor | The occupancy allocation and snapshot tables above are normative. |
| Rust anchor | Occupancy allocation, snapshot, and error types above are implementation anchors. |
| REST anchor | Allocate, release, reserve, snapshot, and correct endpoints are tenant surfaces. |
| gRPC anchor | The occupancy allocation service is the worker and replay contract. |
| AsyncAPI anchor | Allocation changed and snapshot published channels carry analytics and service-charge evidence. |
| Cedar anchor | Allocation mutation is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP object assignment and vacancy lineage projects to occupancy allocation nodes. |
| ADR-0263 class binding | Allocation checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Office-scope or occupancy-pack overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on allocation APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, object id, contract id, allocation basis, and `cedar_decision_id`. |
| Metric | `oya_real_estate_occupancy_allocation_commands_total{tenant_id,cell_id,command,status}` caps cardinality. |
| Latency histogram | `oya_real_estate_occupancy_allocation_duration_seconds` tracks allocation and snapshot latency. |
| Trace span | `real_estate.occupancy_allocation.allocate` links facility master, lease contract, service charge, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `facility_object_id`, `lease_contract_id`, and occupancy status. |
| Capacity math | Allocation rejects when allocated_area exceeds rentable_area * 1.0 and routes near-threshold states for review. |
| Multi-region | Occupancy writes stay in property home cell; DR cells serve read-only occupancy snapshots. |
| Sovereign cells | Workplace occupancy and party evidence remains in-region for active compliance-pack overlays. |
| Rollback | Disable allocation mutations, retain read-only history, and replay from last sealed occupancy audit id. |
| Test evidence | Required tests cover over-allocation, inactive lease, retired object, tenant mismatch, and idempotent snapshot replay. |
| Rejected shortcut | A generic `SpaceAssignment` model is rejected because it loses SAP RE-FX vacancy and allocation-basis semantics. |
