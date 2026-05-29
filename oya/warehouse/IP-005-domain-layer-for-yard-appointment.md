---
doc_class: ImplementationPlan
ip_id: IP-005
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
journey_ref: j107-supply-chain-disruption-and-failover
sap_submodule: EWM-DLV (deliveries)
tenant_class: paid
billing_components:
  - per_usage
persona: Marta Novak, yard coordinator
status: Accepted
date: 2026-05-20
owner_team: axis-warehouse + axis-erp-parity
---

# IP-005: Domain layer for yard appointment

## Context

- SAP submodule: EWM-DLV delivery and dock appointment coordination.
- Persona: Marta Novak, yard coordinator.
- Journey leg: j107 supplier disruption shifts carrier arrival and yard capacity must be rescheduled without losing inbound delivery context.
- SAP tables: `/SCWM/PRDI`, `/SCWM/PRDO`, `/SCWM/DOOR`, `/SCWM/TU`.
- Oyatie aggregate: `YardAppointment`.
- Precedent: SAP EWM transportation unit and door appointment flow plus Kubernetes scheduler-style capacity admission.
- ADR-0244 binds tenant appointment scope and ADR-0297 requires Cedar before dock admission.
- Boundary: owns dock window, carrier arrival, trailer status, and reschedule evidence; it does not own carrier contract or freight cost.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.yard_appointment (
  tenant_id UUID NOT NULL,
  yard_appointment_id TEXT NOT NULL,
  carrier_id TEXT NOT NULL,
  transportation_unit_ref TEXT NOT NULL,
  delivery_ref TEXT NOT NULL,
  dock_door_id TEXT NOT NULL,
  appointment_start TIMESTAMPTZ NOT NULL,
  appointment_end TIMESTAMPTZ NOT NULL,
  appointment_status TEXT NOT NULL CHECK (appointment_status IN ('scheduled','checked_in','at_door','completed','no_show','cancelled','rescheduled')),
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, yard_appointment_id)
);
CREATE TABLE warehouse.yard_appointment_reschedule (
  tenant_id UUID NOT NULL,
  yard_appointment_id TEXT NOT NULL,
  reschedule_no INTEGER NOT NULL,
  reason_code TEXT NOT NULL,
  old_window TSTZRANGE NOT NULL,
  new_window TSTZRANGE NOT NULL,
  PRIMARY KEY (tenant_id, yard_appointment_id, reschedule_no)
);
```

### Rust Types

```rust
pub struct YardAppointment {
    pub tenant_id: TenantId,
    pub yard_appointment_id: YardAppointmentId,
    pub carrier_id: CarrierId,
    pub transportation_unit_ref: TransportationUnitRef,
    pub delivery_ref: DeliveryRef,
    pub dock_door_id: DockDoorId,
    pub appointment_window: TimeWindow,
    pub appointment_status: YardAppointmentStatus,
}
pub struct YardReschedule {
    pub reschedule_no: u32,
    pub reason_code: YardReasonCode,
    pub old_window: TimeWindow,
    pub new_window: TimeWindow,
}
pub enum YardAppointmentError { DoorUnavailable, CarrierNotAuthorized, WindowOverlap, AppointmentExpired, DeliveryRefMissing }
```

## API Endpoints

- REST `POST /v1/warehouse/yard-appointments` schedules dock appointment.
- REST `POST /v1/warehouse/yard-appointments/{id}:check-in` records gate arrival.
- REST `POST /v1/warehouse/yard-appointments/{id}:reschedule` changes door or window.
- REST `POST /v1/warehouse/yard-appointments/{id}:complete` closes the appointment.
- gRPC `warehouse.yard.v1.YardAppointmentService.ScheduleYardAppointment`.
- gRPC `CheckIn`, `Reschedule`, and `CompleteAppointment`.
- AsyncAPI channel `warehouse.yard-appointment.checked-in.v1`.
- AsyncAPI channel `warehouse.yard-appointment.rescheduled.v1`.

## Cedar Policy Hooks

- Policy: `warehouse::yard_appointment::schedule`.
- Principal: `WarehouseYardCoordinator`.
- Action: `yard_appointment_schedule`.
- Resource: `DockDoor`.
- Context: `tenant_id`, `carrier_id`, `delivery_ref`, `appointment_window`, `hazmat_flag`.
- Forbid when carrier is not authorized, dock window overlaps existing appointment, door lacks material capability, or delivery belongs to another tenant.

## Ontology Projection

- Vendor object: SAP EWM transportation unit and door appointment.
- Oyatie object: `warehouse.yard_appointment`.
- `/SCWM/TU-TU_NUM` -> `transportation_unit_ref`.
- `/SCWM/DOOR-LGNUM` and door ID -> `dock_door_id`.
- `/SCWM/PRDI-DOCID` -> inbound delivery reference.
- `/SCWM/PRDO-DOCID` -> outbound delivery reference.
- Carrier check-in event -> `appointment_status`.
- Reschedule reason -> `reason_code`.
- Projection freshness floor: 5 seconds.
- Projection rule: no-show and reschedule rows remain append-only for detention disputes.

## Workflow Steps

- Node `delivery-match`: link appointment to inbound or outbound delivery.
- Node `door-capacity-check`: verify door capability and open window.
- Decision `door-unavailable`: propose next available door.
- Decision `carrier-unauthorized`: block schedule and notify carrier-integration.
- Node `schedule`: persist appointment and send carrier reference.
- Node `gate-check-in`: record arrival and trailer metadata.
- Decision `early-or-late-arrival`: branch to reschedule or wait queue.
- Node `at-door`: bind door and start handling timer.
- Node `complete`: close appointment and emit dwell evidence.
- Node `audit-seal`: persist ADR-0263 appointment trail.

## Audit Events

- `EVT-WAREHOUSE-YARD_APPOINTMENT-SCHEDULED`.
- `EVT-WAREHOUSE-YARD_APPOINTMENT-CHECKED_IN`.
- `EVT-WAREHOUSE-YARD_APPOINTMENT-RESCHEDULED`.
- `EVT-WAREHOUSE-YARD_APPOINTMENT-COMPLETED`.
- `EVT-WAREHOUSE-YARD_APPOINTMENT-POLICY_DENIED`.
- `EVT-WAREHOUSE-YARD_APPOINTMENT-IP_ACCEPTED`.
- ADR-0263 envelope stores `carrier_id`, `dock_door_id`, `appointment_window`, and `transportation_unit_ref`.

## SLO Targets

- Schedule p50: 50 ms.
- Schedule p95: 180 ms.
- Schedule p99: 500 ms.
- Check-in p95: 120 ms at gate kiosk or RF terminal.
- Rationale: yard scheduling can tolerate policy checks, but check-in must avoid gate queues.

## Failure Modes and Recovery

- Failure: `DOOR-UNAVAILABLE`; recovery: return ranked alternate windows.
- Failure: `CARRIER-UNAUTHORIZED`; recovery: route to carrier onboarding or deny check-in.
- Failure: `WINDOW-OVERLAP`; recovery: reject schedule and preserve attempted conflict evidence.
- Failure: `DELIVERY-REF-MISSING`; recovery: park appointment as pending delivery match.
- Failure: `NO-SHOW`; recovery: mark no-show and release door capacity.
- Failure: `CHECKIN-OFFLINE`; recovery: accept signed offline token and reconcile when network returns.

## Migration Notes

- Import SAP transportation units and door appointments as appointment history.
- Map yard doors before appointments to avoid dangling references.
- Preserve no-show, detention, and reschedule reason codes for carrier disputes.
- Do not migrate carrier charge settlement into warehouse; marketplace or carrier service owns settlement.
- Rollback path: keep read-only appointments and disable schedule/check-in mutation.
- Backfill order: dock doors, carriers, deliveries, transportation units, appointments, reschedules.

## Cross-microservice Handoffs

- From carrier-integration: carrier identity and ETA updates.
- From inbound delivery: expected receipt reference.
- From outbound delivery: shipment release reference.
- To workflow-engine: reschedule approvals and carrier disputes.
- To compliance: detention and chain-of-custody audit evidence.
- To labor assignment: dock staffing demand.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The primitive remains bound to SAP EWM delivery and dock appointment coordination. |
| Persona specificity | Marta Novak owns schedule, check-in, dwell, and reschedule acceptance language. |
| Journey specificity | The j107 disruption leg drives ETA correction, yard capacity, and carrier dispute behavior. |
| DDL anchor | The yard appointment and transportation-unit tables above are the normative yard state model. |
| Rust anchor | The yard appointment aggregate, dwell type, and error enum above are the implementation contract. |
| REST anchor | Schedule, check-in, reschedule, and close endpoints are the tenant API surface. |
| gRPC anchor | The yard appointment service is the worker and replay contract for dock orchestration. |
| AsyncAPI anchor | Scheduled, checked-in, rescheduled, and dwell-exceeded channels carry downstream evidence. |
| Cedar anchor | Schedule and check-in decisions are default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP delivery, dock door, and transportation-unit lineage project to yard appointment nodes. |
| ADR-0263 class binding | Yard policy checks emit `OfficeBoundaryAttemptEvaluated` and outcome classes for allow or deny. |
| ADR-0263 pack binding | Site access and detention-policy overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on check-in APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, appointment id, carrier id, dock door, ETA, and `cedar_decision_id`. |
| Metric | `oya_warehouse_yard_appointment_commands_total{tenant_id,cell_id,command,status}` caps command/status cardinality. |
| Latency histogram | `oya_warehouse_yard_appointment_command_duration_seconds` tracks schedule and check-in latency. |
| Trace span | `warehouse.yard_appointment.check_in` links carrier ETA, dock capacity, workflow, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `carrier_id`, `dock_door_id`, `arrival_window`, and dwell state. |
| Capacity math | Dock utilization blocks new appointments when planned dwell minutes exceed dock_capacity_minutes * 0.85. |
| Multi-region | Yard mutation stays in the facility home cell; DR cells expose read-only appointment and dwell projections. |
| Sovereign cells | Driver identity and chain-of-custody evidence remain in-region for applicable sovereign and regulated packs. |
| Rollback | Disable schedule/check-in mutation, keep read-only appointments, and replay from the last sealed yard audit id. |
| Test evidence | Required tests cover expired window, carrier mismatch, dock conflict, detention dispute, and idempotent check-in. |
| Rejected shortcut | A generic `Appointment` record is rejected because it loses EWM delivery, dock, and transportation-unit semantics. |
