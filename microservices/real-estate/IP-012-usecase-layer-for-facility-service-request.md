---
doc_class: ImplementationPlan
ip_id: IP-012
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

# IP-012: Usecase layer for facility service request

## Context

- SAP submodule: RE-FX-RT room/facility service execution.
- Persona: Mina Park, facilities service coordinator.
- Journey leg: j156 after-hours request is triaged, dispatched, monitored, and resolved with SLA evidence.
- SAP tables: `VIBDRO`, `VICDOBJASS`, `VICDCONTRACT`, `VICDCONDLINE`.
- Oyatie usecase: `OperateFacilityServiceRequest`.
- Precedent: SAP RE-FX facility service flow plus ServiceNow incident state machine.
- ADR-0105 keeps dispatch orchestration out of domain and ADR-0263 binds SLA events.
- Boundary: coordinates triage, maintenance dispatch, tenant status, and resolution evidence.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.service_request_command (
  tenant_id UUID NOT NULL,
  command_id TEXT NOT NULL,
  service_request_id TEXT NOT NULL,
  command_kind TEXT NOT NULL CHECK (command_kind IN ('open','triage','dispatch','resolve','cancel','escalate')),
  idempotency_key TEXT NOT NULL,
  command_status TEXT NOT NULL,
  failure_code TEXT,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, command_id),
  UNIQUE (tenant_id, idempotency_key)
);
CREATE TABLE real_estate.service_request_handoff (
  tenant_id UUID NOT NULL,
  handoff_id TEXT NOT NULL,
  command_id TEXT NOT NULL,
  target_microservice TEXT NOT NULL,
  target_ref TEXT,
  handoff_state TEXT NOT NULL,
  PRIMARY KEY (tenant_id, handoff_id)
);
```

### Rust Types

```rust
pub struct ServiceRequestCommand {
    pub tenant_id: TenantId,
    pub command_id: CommandId,
    pub service_request_id: ServiceRequestId,
    pub command_kind: ServiceRequestCommandKind,
    pub idempotency_key: IdempotencyKey,
    pub command_status: CommandStatus,
}
pub struct ServiceRequestHandoff {
    pub handoff_id: HandoffId,
    pub command_id: CommandId,
    pub target_microservice: MicroserviceName,
    pub target_ref: Option<TargetRef>,
    pub handoff_state: HandoffState,
}
pub enum OperateFacilityServiceRequestError { SlaMissing, DispatchDenied, MaintenanceHandoffFailed, TenantCommsFailed, ResolvePolicyDenied }
```

## API Endpoints

- REST `POST /v1/real-estate/facility-service-requests/{id}:operate`.
- REST `POST /v1/real-estate/service-request-commands/{command_id}:retry-handoffs`.
- REST `GET /v1/real-estate/service-request-commands/{command_id}`.
- gRPC `real_estate.service_request_usecase.v1.OperateFacilityServiceRequest`.
- gRPC `RetryServiceRequestHandoffs` and `StreamServiceRequestCommands`.
- AsyncAPI channel `real-estate.service-request.command-succeeded.v1`.
- AsyncAPI channel `real-estate.service-request.escalated.v1`.
- Consumers: plant-maintenance, workflow-engine, tenant-comms, compliance.

## Cedar Policy Hooks

- Policy: `real_estate::service_request_command::operate`.
- Principal: `FacilityServiceCoordinator`.
- Action: `service_request_command_execute`.
- Resource: `FacilityServiceRequest`.
- Context: `tenant_id`, `command_kind`, `priority`, `sla_due_at`, `facility_object_id`, `requester_party_id`.
- Forbid when dispatch route is missing, requester is unauthorized, resolution lacks evidence, or cancellation violates tenant notification policy.

## Ontology Projection

- Vendor object: SAP RE-FX facility service state change.
- Oyatie object: `real_estate.service_request_command`.
- `VIBDRO-SMENR` -> facility object lineage.
- `VICDOBJASS-OBJNR` -> occupancy assignment lineage.
- `VICDCONTRACT-CONTRACT` -> lease access lineage.
- `VICDCONDLINE-CONDGUID` -> chargeable service condition.
- Command kind -> request state transition.
- Handoff state -> maintenance and communication evidence.
- Projection freshness floor: 3 seconds.
- Projection rule: tenant-facing status never exposes internal maintenance notes.

## Workflow Steps

- Node `command-accept`: dedupe service command.
- Node `request-load`: validate request and object.
- Decision `sla-missing`: use default SLA and create admin task.
- Node `triage`: classify and assign priority.
- Decision `dispatch-denied`: route to coordinator review.
- Node `maintenance-handoff`: create work order.
- Decision `handoff-failed`: keep request triaged and retry.
- Node `tenant-notify`: publish tenant status update.
- Node `resolve`: close with completion evidence.
- Node `audit-seal`: emit command evidence.

## Audit Events

- `EVT-REAL_ESTATE-SERVICE_REQUEST_COMMAND-ACCEPTED`.
- `EVT-REAL_ESTATE-SERVICE_REQUEST_COMMAND-DISPATCHED`.
- `EVT-REAL_ESTATE-SERVICE_REQUEST_COMMAND-ESCALATED`.
- `EVT-REAL_ESTATE-SERVICE_REQUEST_COMMAND-RESOLVED`.
- `EVT-REAL_ESTATE-SERVICE_REQUEST_COMMAND-POLICY_DENIED`.
- `EVT-REAL_ESTATE-SERVICE_REQUEST_COMMAND-IP_ACCEPTED`.
- ADR-0263 envelope stores command kind, SLA due time, handoff state, and requester.

## SLO Targets

- Command accept p50: 35 ms.
- Command accept p95: 130 ms.
- Command accept p99: 350 ms.
- Emergency dispatch p95: 1,000 ms.
- Rationale: tenant request status must be immediate; emergency handoff must be bounded to protect after-hours workflows.

## Failure Modes and Recovery

- Failure: `SLA-MISSING`; recovery: apply default SLA and create setup task.
- Failure: `DISPATCH-DENIED`; recovery: hold request and notify coordinator.
- Failure: `MAINTENANCE-HANDOFF-FAILED`; recovery: retry outbox and escalate on age.
- Failure: `TENANT-COMMS-FAILED`; recovery: preserve request status and retry notification.
- Failure: `RESOLVE-POLICY-DENIED`; recovery: keep request dispatched until evidence present.
- Failure: `DUPLICATE-PAYLOAD`; recovery: return prior command receipt.

## Migration Notes

- Import pending service requests as commands only when lifecycle remains open.
- Import historical tickets as immutable request/status evidence.
- Preserve external maintenance ticket references.
- Do not migrate private notes into tenant status.
- Rollback path: disable operate endpoint and preserve existing request state.
- Backfill order: service requests, commands, status events, handoffs.

## Cross-microservice Handoffs

- From facility master: object and room identity.
- To plant-maintenance: work order creation.
- To tenant-comms: status notification.
- To workflow-engine: escalation and exception approvals.
- To compliance: SLA and evidence trail.
- To portfolio analytics: service burden metrics.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The usecase remains bound to SAP RE-FX-RT room/facility service execution. |
| Persona specificity | Mina Park owns triage, dispatch, SLA status, and rollback acceptance language. |
| Journey specificity | The j156 after-hours maintenance leg drives emergency escalation and tenant notification behavior. |
| DDL anchor | The service request command, status event, and handoff tables above are normative. |
| Rust anchor | Service request command, status result, dispatch handoff, and error types above are anchors. |
| REST anchor | Operate, triage, dispatch, update status, and resolve endpoints are tenant surfaces. |
| gRPC anchor | The facility service request usecase service is the worker and replay contract. |
| AsyncAPI anchor | Request triaged, dispatched, escalated, and resolved channels carry SLA evidence. |
| Cedar anchor | Request operations are default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP room/facility lineage projects to service request command and status nodes. |
| ADR-0263 class binding | Service request checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Emergency, tenant-comms, or premises overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on tenant-visible request APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, request id, object id, priority, SLA state, and `cedar_decision_id`. |
| Metric | `oya_real_estate_service_request_usecase_commands_total{tenant_id,cell_id,command,status}` caps cardinality. |
| Latency histogram | `oya_real_estate_service_request_usecase_duration_seconds` tracks triage, dispatch, and resolution latency. |
| Trace span | `real_estate.facility_service_request.operate` links facility master, workflow, tenant-comms, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `request_id`, `facility_object_id`, `priority`, and SLA breach flag. |
| Capacity math | Emergency workload blocks low-priority dispatch when active emergencies exceed responder capacity threshold. |
| Multi-region | Request writes stay in property home cell; DR cells expose read-only status. |
| Sovereign cells | Tenant request and premises evidence remains in-region for privacy and sovereign packs. |
| Rollback | Disable operate endpoint, preserve request state, and replay from last sealed request usecase audit id. |
| Test evidence | Required tests cover emergency escalation, private-note scrubbing, maintenance handoff failure, tenant mismatch, and replay. |
| Rejected shortcut | A generic support workflow is rejected because it loses SAP RE-FX room/facility and SLA semantics. |
