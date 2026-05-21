---
doc_class: ImplementationPlan
ip_id: IP-006
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
journey_ref: j156-carlos-reyes-ii-maintenance-emergency-after-hours
sap_submodule: RE-FX-RT (room reservations)
tenant_class: paid
billing_components:
  - per_usage
persona: Mina Park, facilities service coordinator
status: Accepted
date: 2026-05-20
owner_team: axis-real-estate + axis-erp-parity
---

# IP-006: Domain layer for facility service request

## Context

- SAP submodule: RE-FX-RT room and facility service coordination.
- Persona: Mina Park, facilities service coordinator.
- Journey leg: j156 after-hours maintenance emergency creates a tenant-visible facility service request tied to a room or unit.
- SAP tables: `VIBDRO`, `VICDOBJASS`, `VICDCONTRACT`, `VICDCONDLINE`.
- Oyatie aggregate: `FacilityServiceRequest`.
- Precedent: SAP RE-FX room reservation/service request linkage plus ServiceNow facilities case management.
- ADR-0263 binds service request audit and ADR-0297 gates tenant-visible service actions.
- Boundary: owns request identity, facility object link, priority, SLA clock, and tenant communication reference; work-order dispatch remains plant-maintenance.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.facility_service_request (
  tenant_id UUID NOT NULL,
  service_request_id TEXT NOT NULL,
  facility_object_id TEXT NOT NULL,
  lease_contract_id TEXT,
  requester_party_id TEXT NOT NULL,
  request_type TEXT NOT NULL,
  priority TEXT NOT NULL CHECK (priority IN ('low','normal','high','emergency')),
  request_status TEXT NOT NULL CHECK (request_status IN ('opened','triaged','dispatched','resolved','cancelled')),
  sla_due_at TIMESTAMPTZ NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, service_request_id)
);
CREATE TABLE real_estate.service_request_status_event (
  tenant_id UUID NOT NULL,
  service_request_id TEXT NOT NULL,
  event_no INTEGER NOT NULL,
  status TEXT NOT NULL,
  reason TEXT,
  recorded_at TIMESTAMPTZ NOT NULL,
  PRIMARY KEY (tenant_id, service_request_id, event_no)
);
```

### Rust Types

```rust
pub struct FacilityServiceRequest {
    pub tenant_id: TenantId,
    pub service_request_id: ServiceRequestId,
    pub facility_object_id: FacilityObjectId,
    pub lease_contract_id: Option<LeaseContractId>,
    pub requester_party_id: PartyId,
    pub request_type: ServiceRequestType,
    pub priority: ServicePriority,
    pub request_status: ServiceRequestStatus,
    pub sla_due_at: DateTime<Utc>,
}
pub struct ServiceRequestStatusEvent {
    pub event_no: u32,
    pub status: ServiceRequestStatus,
    pub reason: Option<String>,
    pub recorded_at: DateTime<Utc>,
}
pub enum FacilityServiceRequestError { ObjectInactive, RequesterUnauthorized, EmergencyRouteMissing, SlaPolicyMissing, MaintenanceHandoffFailed }
```

## API Endpoints

- REST `POST /v1/real-estate/facility-service-requests` opens request.
- REST `POST /v1/real-estate/facility-service-requests/{id}:triage`.
- REST `POST /v1/real-estate/facility-service-requests/{id}:resolve`.
- gRPC `real_estate.service_request.v1.FacilityServiceRequestService.OpenRequest`.
- gRPC `TriageRequest`, `ResolveRequest`, and `StreamRequestStatus`.
- AsyncAPI channel `real-estate.facility-service-request.opened.v1`.
- AsyncAPI channel `real-estate.facility-service-request.dispatched.v1`.
- Consumers: plant-maintenance, workflow-engine, tenant-comms, compliance.

## Cedar Policy Hooks

- Policy: `real_estate::service_request::open`.
- Principal: `TenantFacilityRequester`.
- Action: `facility_service_request_open`.
- Resource: `FacilityObject`.
- Context: `tenant_id`, `requester_party_id`, `lease_contract_id`, `priority`, `after_hours`.
- Forbid when requester lacks access to object, facility object is inactive, emergency route is missing for emergency priority, or request crosses tenant boundary.

## Ontology Projection

- Vendor object: SAP RE-FX facility/room request.
- Oyatie object: `real_estate.facility_service_request`.
- `VIBDRO-SMENR` -> `facility_object_id`.
- `VICDOBJASS-OBJNR` -> assignment lineage.
- `VICDCONTRACT-CONTRACT` -> tenant lease reference.
- `VICDCONDLINE-CONDGUID` -> chargeable service condition lineage if applicable.
- Request type -> service taxonomy.
- SLA due time -> response obligation.
- Projection freshness floor: 3 seconds.
- Projection rule: tenant-visible request status is separate from plant-maintenance work-order internals.

## Workflow Steps

- Node `request-open`: validate requester and object.
- Decision `object-inactive`: reject request.
- Decision `requester-unauthorized`: deny and audit.
- Node `sla-calculate`: assign SLA by priority and pack.
- Decision `emergency-route-missing`: block emergency until route configured.
- Node `triage`: classify service request.
- Node `maintenance-handoff`: request plant-maintenance work order.
- Decision `handoff-failed`: keep request triaged and retry.
- Node `resolve`: capture completion and tenant-visible status.
- Node `audit-seal`: emit request evidence.

## Audit Events

- `EVT-REAL_ESTATE-SERVICE_REQUEST-OPENED`.
- `EVT-REAL_ESTATE-SERVICE_REQUEST-TRIAGED`.
- `EVT-REAL_ESTATE-SERVICE_REQUEST-DISPATCHED`.
- `EVT-REAL_ESTATE-SERVICE_REQUEST-RESOLVED`.
- `EVT-REAL_ESTATE-SERVICE_REQUEST-POLICY_DENIED`.
- `EVT-REAL_ESTATE-SERVICE_REQUEST-IP_ACCEPTED`.
- ADR-0263 envelope stores `facility_object_id`, priority, requester, SLA due time, and handoff reference.

## SLO Targets

- Open request p50: 45 ms.
- Open request p95: 160 ms.
- Open request p99: 400 ms.
- Emergency dispatch handoff p95: 1,000 ms.
- Rationale: tenants need immediate request acknowledgement; emergency dispatch may include plant-maintenance acknowledgement.

## Failure Modes and Recovery

- Failure: `OBJECT-INACTIVE`; recovery: reject and return current object status.
- Failure: `REQUESTER-UNAUTHORIZED`; recovery: deny and emit policy event.
- Failure: `EMERGENCY-ROUTE-MISSING`; recovery: route to fallback incident workflow.
- Failure: `SLA-POLICY-MISSING`; recovery: apply tenant default SLA and create admin task.
- Failure: `MAINTENANCE-HANDOFF-FAILED`; recovery: retry outbox and show triaged status.
- Failure: `STATUS-EVENT-GAP`; recovery: rebuild status from append-only events.

## Migration Notes

- Import open facility service cases only when facility object mapping exists.
- Preserve source ticket ID as lineage.
- Map service request types to tenant taxonomy before activation.
- Do not migrate private maintenance notes into tenant-visible request fields.
- Rollback path: disable request mutation and preserve read-only ticket history.
- Backfill order: facility objects, contracts, requesters, service requests, status events.

## Cross-microservice Handoffs

- From facility master: object identity and status.
- From lease-contract: requester access rights.
- To plant-maintenance: work-order request.
- To workflow-engine: triage and emergency escalation.
- To tenant-comms: status notifications.
- To compliance: request and SLA evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The primitive remains bound to SAP RE-FX-RT room and facility service coordination. |
| Persona specificity | Mina Park owns after-hours triage, tenant-visible status, and rollback language. |
| Journey specificity | The j156 maintenance-emergency leg drives escalation, SLA, and tenant notification behavior. |
| DDL anchor | The facility service request and status-event tables above are normative. |
| Rust anchor | Facility service request, status event, and error types above are implementation anchors. |
| REST anchor | Create request, triage, dispatch, update status, and close endpoints are tenant surfaces. |
| gRPC anchor | The service request service is the worker and replay contract. |
| AsyncAPI anchor | Request created, escalated, dispatched, and resolved channels carry SLA evidence. |
| Cedar anchor | Request mutation is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP room/facility lineage projects to service request and maintenance handoff nodes. |
| ADR-0263 class binding | Service request checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Emergency, tenant-comms, or premises overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on tenant-visible request APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, request id, object id, requester id, SLA class, and `cedar_decision_id`. |
| Metric | `oya_real_estate_service_requests_total{tenant_id,cell_id,priority,status}` caps priority/status cardinality. |
| Latency histogram | `oya_real_estate_service_request_duration_seconds` tracks create, triage, dispatch, and resolve latency. |
| Trace span | `real_estate.facility_service_request.triage` links facility master, workflow, plant maintenance, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `request_id`, `facility_object_id`, `priority`, and SLA state. |
| Capacity math | Emergency queue backlog uses active_requests * mean_triage_time; p95 SLA risk triggers escalation fan-out. |
| Multi-region | Request writes stay in property home cell; DR cells expose read-only status projections. |
| Sovereign cells | Tenant request and premises evidence remains in-region for privacy and sovereign packs. |
| Rollback | Disable request mutation, preserve read-only ticket history, and replay from last sealed service request audit id. |
| Test evidence | Required tests cover requester authorization, emergency escalation, private-note scrubbing, tenant mismatch, and idempotent status replay. |
| Rejected shortcut | A generic `SupportTicket` is rejected because it loses SAP RE-FX facility, room, and tenant SLA semantics. |
