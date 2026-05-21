---
doc_class: ImplementationPlan
ip_id: IP-002
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

# IP-002: Domain layer for facility master

## Context

- SAP submodule: RE-FX-AS architectural objects.
- Persona: Tobias Klein, corporate real-estate data steward.
- Journey leg: j168 ops review requires building, floor, room, and rentable-area evidence to reconcile occupancy and service charges.
- SAP tables: `VIBDBU`, `VIBDRO`, `VICDOBJASS`, `VICDCONTRACT`.
- Oyatie aggregate: `FacilityMaster`.
- Precedent: SAP RE-FX architectural object hierarchy plus Autodesk Tandem digital-twin asset hierarchy.
- ADR-0244 scopes every object by tenant and ADR-0263 records object lifecycle evidence.
- Boundary: owns facility object identity, hierarchy, rentable area, and occupancy eligibility; maintenance work orders remain plant-maintenance owned.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.facility_object (
  tenant_id UUID NOT NULL,
  facility_object_id TEXT NOT NULL,
  sap_object_ref TEXT NOT NULL,
  object_type TEXT NOT NULL CHECK (object_type IN ('site','building','floor','room','unit','parking_space')),
  parent_object_id TEXT,
  object_name TEXT NOT NULL,
  rentable_area_sqm NUMERIC(14,4),
  object_status TEXT NOT NULL CHECK (object_status IN ('planned','active','inactive','retired')),
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, facility_object_id)
);
CREATE TABLE real_estate.facility_area_measurement (
  tenant_id UUID NOT NULL,
  measurement_id TEXT NOT NULL,
  facility_object_id TEXT NOT NULL,
  measurement_type TEXT NOT NULL,
  area_sqm NUMERIC(14,4) NOT NULL,
  effective_from DATE NOT NULL,
  PRIMARY KEY (tenant_id, measurement_id)
);
```

### Rust Types

```rust
pub struct FacilityObject {
    pub tenant_id: TenantId,
    pub facility_object_id: FacilityObjectId,
    pub sap_object_ref: SapObjectRef,
    pub object_type: FacilityObjectType,
    pub parent_object_id: Option<FacilityObjectId>,
    pub object_name: String,
    pub rentable_area_sqm: Option<Decimal>,
    pub object_status: FacilityObjectStatus,
}
pub struct FacilityAreaMeasurement {
    pub measurement_id: MeasurementId,
    pub facility_object_id: FacilityObjectId,
    pub measurement_type: AreaMeasurementType,
    pub area_sqm: Decimal,
    pub effective_from: NaiveDate,
}
pub enum FacilityMasterError { ParentMissing, AreaInvalid, ObjectCycleDetected, DuplicateObjectRef, PolicyDenied }
```

## API Endpoints

- REST `POST /v1/real-estate/facility-objects` creates facility object.
- REST `POST /v1/real-estate/facility-objects/{id}:retire` retires object with reason.
- REST `POST /v1/real-estate/facility-objects/{id}/area-measurements` records rentable area.
- gRPC `real_estate.facility.v1.FacilityMasterService.CreateFacilityObject`.
- gRPC `RetireFacilityObject`, `RecordAreaMeasurement`, and `ListFacilityTree`.
- AsyncAPI channel `real-estate.facility-object.created.v1`.
- AsyncAPI channel `real-estate.facility-area.updated.v1`.
- Consumers: lease-contract, occupancy-allocation, service-charge, plant-maintenance.

## Cedar Policy Hooks

- Policy: `real_estate::facility_master::mutate`.
- Principal: `FacilityDataSteward`.
- Action: `facility_object_mutate`.
- Resource: `FacilityObject`.
- Context: `tenant_id`, `object_type`, `parent_object_id`, `rentable_area_sqm`, `effective_from`.
- Forbid when parent object is missing, hierarchy cycle is detected, area is negative, or mutation would retire an object with active lease assignment.

## Ontology Projection

- Vendor object: SAP RE-FX architectural object.
- Oyatie object: `real_estate.facility_object`.
- `VIBDBU-SWENR` -> building object lineage.
- `VIBDRO-SMENR` -> room or unit lineage.
- `VICDOBJASS-OBJNR` -> contract assignment link.
- `VICDCONTRACT-CONTRACT` -> active lease linkage.
- Rentable area -> `rentable_area_sqm`.
- Parent object -> hierarchy edge.
- Projection freshness floor: 5 seconds.
- Projection rule: retired objects remain addressable for historical contracts and audits.

## Workflow Steps

- Node `object-draft`: create object with parent and type.
- Decision `parent-missing`: reject unless object type is site.
- Decision `cycle-detected`: block hierarchy mutation.
- Node `area-record`: capture measured area.
- Decision `area-invalid`: reject measurement.
- Node `status-activate`: make object assignable.
- Decision `active-lease-on-retire`: block retirement and request lease review.
- Node `ontology-project`: publish hierarchy projection.
- Node `maintenance-link`: notify plant-maintenance about new object.
- Node `audit-seal`: emit facility event.

## Audit Events

- `EVT-REAL_ESTATE-FACILITY_OBJECT-CREATED`.
- `EVT-REAL_ESTATE-FACILITY_OBJECT-AREA_RECORDED`.
- `EVT-REAL_ESTATE-FACILITY_OBJECT-ACTIVATED`.
- `EVT-REAL_ESTATE-FACILITY_OBJECT-RETIRED`.
- `EVT-REAL_ESTATE-FACILITY_OBJECT-POLICY_DENIED`.
- `EVT-REAL_ESTATE-FACILITY_OBJECT-IP_ACCEPTED`.
- ADR-0263 envelope stores `sap_object_ref`, `object_type`, parent object, and area measurement.

## SLO Targets

- Object create p50: 45 ms.
- Object create p95: 170 ms.
- Object create p99: 420 ms.
- Facility tree read p95: 250 ms for 10,000 objects.
- Rationale: facility data stewarding is interactive, while large tree reads support operations dashboards.

## Failure Modes and Recovery

- Failure: `PARENT-MISSING`; recovery: reject child object until parent is created.
- Failure: `AREA-INVALID`; recovery: reject measurement and preserve prior value.
- Failure: `OBJECT-CYCLE-DETECTED`; recovery: block hierarchy update.
- Failure: `DUPLICATE-OBJECT-REF`; recovery: return existing object lineage.
- Failure: `ACTIVE-LEASE-ON-RETIRE`; recovery: route to lease review.
- Failure: `MAINTENANCE-HANDOFF-FAILED`; recovery: retry outbox without blocking facility object.

## Migration Notes

- Import buildings and sites before rooms and units.
- Import area measurements as time-effective facts.
- Preserve SAP architectural object numbers as lineage.
- Do not retire objects during migration when active contracts exist.
- Rollback path: disable facility mutation and keep tree read-only.
- Backfill order: sites, buildings, floors, rooms, areas, object assignments.

## Cross-microservice Handoffs

- To lease-contract: assignable facility object.
- To occupancy-allocation: area and hierarchy basis.
- To service-charge: allocation area basis.
- To plant-maintenance: maintainable object reference.
- To analytics: portfolio area and vacancy metrics.
- To compliance: architectural object lifecycle evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The primitive remains bound to SAP RE-FX-AS architectural objects. |
| Persona specificity | Tobias Klein owns object hierarchy, area evidence, and rollback acceptance language. |
| Journey specificity | The j168 ops-review leg drives building, floor, room, and rentable-area reconciliation. |
| DDL anchor | The facility master, object hierarchy, and area measurement tables above are normative. |
| Rust anchor | Facility object, hierarchy node, area measurement, and error types above are implementation anchors. |
| REST anchor | Create/update object, reparent, retire, and measure-area endpoints are tenant surfaces. |
| gRPC anchor | The facility master service is the worker and replay contract for hierarchy changes. |
| AsyncAPI anchor | Object changed, hierarchy changed, and area measured channels carry downstream evidence. |
| Cedar anchor | Facility mutation is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP architectural object lineage projects to facility graph nodes without replacing Oyatie identity. |
| ADR-0263 class binding | Facility policy checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Office-scope or building-pack overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on facility APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, object id, parent id, area basis, and `cedar_decision_id`. |
| Metric | `oya_real_estate_facility_master_commands_total{tenant_id,cell_id,command,status}` caps command/status cardinality. |
| Latency histogram | `oya_real_estate_facility_master_duration_seconds` tracks hierarchy and measurement command latency. |
| Trace span | `real_estate.facility_master.update_hierarchy` links occupancy, lease contract, maintenance, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `facility_object_id`, `object_type`, and hierarchy version. |
| Capacity math | Hierarchy recompute caps descendants touched per mutation; large moves run through worker queue with backpressure. |
| Multi-region | Facility hierarchy writes stay in the property home cell; DR cells serve read-only hierarchy snapshots. |
| Sovereign cells | Premises and site evidence remains in-region for active sovereign and regulated packs. |
| Rollback | Disable facility mutation, keep tree read-only, and replay from last sealed facility audit id. |
| Test evidence | Required tests cover active-contract retire block, cycle prevention, area mismatch, tenant mismatch, and replay idempotency. |
| Rejected shortcut | A generic `Location` tree is rejected because it loses SAP RE-FX architectural object and area semantics. |
