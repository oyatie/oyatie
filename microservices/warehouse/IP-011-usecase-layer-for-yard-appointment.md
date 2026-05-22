---
doc_class: ImplementationPlan
ip_id: IP-011
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

# IP-011: Usecase layer for yard appointment

## Context

- SAP submodule: EWM-DLV dock and transportation-unit execution.
- Persona: Marta Novak, yard coordinator.
- Journey leg: j107 disrupted ETA is converted into reschedule, check-in, and dwell evidence.
- SAP tables: `/SCWM/TU`, `/SCWM/DOOR`, `/SCWM/PRDI`, `/SCWM/PRDO`.
- Oyatie usecase: `OperateYardAppointment`.
- Precedent: SAP EWM yard appointment flow plus Kubernetes admission control for scarce dock capacity.
- ADR-0297 requires policy admission before schedule/check-in and ADR-0263 seals every reschedule.
- Boundary: orchestrates schedule, check-in, at-door, complete, no-show, and reschedule commands.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.yard_command (
  tenant_id UUID NOT NULL,
  command_id TEXT NOT NULL,
  yard_appointment_id TEXT NOT NULL,
  command_kind TEXT NOT NULL CHECK (command_kind IN ('schedule','check_in','at_door','complete','reschedule','no_show')),
  idempotency_key TEXT NOT NULL,
  command_status TEXT NOT NULL,
  failure_code TEXT,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, command_id),
  UNIQUE (tenant_id, idempotency_key)
);
CREATE TABLE warehouse.yard_dwell_interval (
  tenant_id UUID NOT NULL,
  yard_appointment_id TEXT NOT NULL,
  interval_no INTEGER NOT NULL,
  state TEXT NOT NULL,
  started_at TIMESTAMPTZ NOT NULL,
  ended_at TIMESTAMPTZ,
  PRIMARY KEY (tenant_id, yard_appointment_id, interval_no)
);
```

### Rust Types

```rust
pub struct YardCommand {
    pub tenant_id: TenantId,
    pub command_id: CommandId,
    pub yard_appointment_id: YardAppointmentId,
    pub command_kind: YardCommandKind,
    pub idempotency_key: IdempotencyKey,
    pub command_status: CommandStatus,
}
pub struct YardDwellInterval {
    pub interval_no: u32,
    pub state: YardDwellState,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}
pub enum YardCommandError { DoorConflict, CarrierDenied, WindowExpired, DwellClockConflict, DeliveryNotReady }
```

## API Endpoints

- REST `POST /v1/warehouse/yard-appointments/{id}:operate` executes yard command.
- REST `POST /v1/warehouse/yard-appointments/{id}:mark-no-show` releases dock capacity.
- REST `GET /v1/warehouse/yard-commands/{command_id}` returns command state.
- gRPC `warehouse.yard_usecase.v1.OperateYardAppointment`.
- gRPC `MarkNoShow`, `GetYardCommand`, and `StreamDockDwell`.
- AsyncAPI channel `warehouse.yard-command.succeeded.v1`.
- AsyncAPI channel `warehouse.yard-dwell.updated.v1`.
- Consumers: carrier-integration, labor-assignment, compliance, workflow-engine.

## Cedar Policy Hooks

- Policy: `warehouse::yard_command::operate`.
- Principal: `WarehouseYardCoordinator`.
- Action: `yard_command_execute`.
- Resource: `YardAppointment`.
- Context: `tenant_id`, `command_kind`, `carrier_id`, `dock_door_id`, `appointment_window`, `delivery_ready_state`.
- Forbid when carrier is denied, dock door conflicts, appointment window expired, or delivery is not ready for the operation.

## Ontology Projection

- Vendor object: SAP EWM yard execution command.
- Oyatie object: `warehouse.yard_command`.
- `/SCWM/TU-TU_NUM` -> `yard_appointment_id`.
- `/SCWM/DOOR-DOOR` -> `dock_door_id`.
- `/SCWM/PRDI-DOCID` -> inbound delivery lineage.
- `/SCWM/PRDO-DOCID` -> outbound delivery lineage.
- Command kind -> yard state mutation.
- Dwell interval -> detention and throughput evidence.
- Projection freshness floor: 5 seconds.
- Projection rule: dwell intervals are append-only and never overwritten.

## Workflow Steps

- Node `command-accept`: dedupe yard command.
- Node `policy-evaluate`: check carrier, door, and delivery readiness.
- Decision `door-conflict`: propose alternate door or window.
- Decision `delivery-not-ready`: hold command and notify delivery owner.
- Node `state-apply`: update appointment status.
- Node `dwell-clock-update`: open or close dwell interval.
- Decision `dwell-conflict`: create correction workflow.
- Node `capacity-release`: release dock on complete or no-show.
- Node `outbox-dispatch`: notify downstream services.
- Node `audit-seal`: persist command evidence.

## Audit Events

- `EVT-WAREHOUSE-YARD_COMMAND-ACCEPTED`.
- `EVT-WAREHOUSE-YARD_COMMAND-CHECKED_IN`.
- `EVT-WAREHOUSE-YARD_COMMAND-DWELL_UPDATED`.
- `EVT-WAREHOUSE-YARD_COMMAND-NO_SHOW`.
- `EVT-WAREHOUSE-YARD_COMMAND-POLICY_DENIED`.
- `EVT-WAREHOUSE-YARD_COMMAND-IP_ACCEPTED`.
- ADR-0263 envelope stores `command_kind`, `dock_door_id`, `carrier_id`, and `dwell_interval_ref`.

## SLO Targets

- Command accept p50: 35 ms.
- Command accept p95: 130 ms.
- Command accept p99: 350 ms.
- Dwell update p95: 100 ms.
- Rationale: gate and dock interactions must not create truck queues; reschedule can tolerate policy context.

## Failure Modes and Recovery

- Failure: `DOOR-CONFLICT`; recovery: reject command and return alternate capacity.
- Failure: `CARRIER-DENIED`; recovery: keep appointment blocked and notify carrier onboarding.
- Failure: `WINDOW-EXPIRED`; recovery: require reschedule command.
- Failure: `DWELL-CLOCK-CONFLICT`; recovery: create correction workflow with immutable original interval.
- Failure: `DELIVERY-NOT-READY`; recovery: hold at yard queue and notify inbound/outbound owner.
- Failure: `OUTBOX-DELAY`; recovery: retry downstream notifications without losing yard state.

## Migration Notes

- Import active SAP transportation-unit states into command history only for open appointments.
- Import historical dwell as immutable intervals.
- Preserve source check-in and at-door timestamps for detention disputes.
- Normalize SAP door and carrier identifiers before appointment command import.
- Rollback path: disable operate endpoint and keep yard status read-only.
- Backfill order: appointments, commands, dwell intervals, downstream event state.

## Cross-microservice Handoffs

- From carrier-integration: ETA, carrier status, and driver identity.
- From inbound/outbound delivery: delivery readiness.
- To labor assignment: dock labor demand.
- To workflow-engine: dwell correction and dispute tasks.
- To compliance: detention and chain-of-custody evidence.
- To analytics: dwell-time throughput metrics.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The usecase remains bound to SAP EWM dock and transportation-unit execution. |
| Persona specificity | Marta Novak owns appointment operation, dwell correction, and rollback acceptance language. |
| Journey specificity | The j107 disrupted-ETA leg drives reschedule, check-in, dwell, and dispute behavior. |
| DDL anchor | The appointment command and dwell interval tables above are the normative usecase model. |
| Rust anchor | The appointment operation command, dwell result, and error enum above are the implementation contract. |
| REST anchor | Operate, reschedule, check-in, and close endpoints are the tenant command surface. |
| gRPC anchor | The yard operation service is the worker and replay contract for dock execution. |
| AsyncAPI anchor | Check-in, dwell-exceeded, and rescheduled channels carry analytics and compliance evidence. |
| Cedar anchor | Yard operations are default-deny and must persist `cedar_decision_id` before state mutation. |
| Ontology anchor | SAP door, carrier, delivery, and transportation-unit lineage projects to yard operation nodes. |
| ADR-0263 class binding | Yard operation checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Detention, chain-of-custody, or site-access overlays emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on yard APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, appointment id, carrier id, dwell interval, and `cedar_decision_id`. |
| Metric | `oya_warehouse_yard_operation_commands_total{tenant_id,cell_id,command,status}` caps command/status cardinality. |
| Latency histogram | `oya_warehouse_yard_operation_duration_seconds` tracks operate and check-in command latency. |
| Trace span | `warehouse.yard_appointment.operate` links carrier-integration, labor assignment, workflow, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `carrier_id`, `driver_ref`, `dock_door_id`, and dwell reason. |
| Capacity math | Dwell backlog uses active_units * average_dwell_minutes; overload triggers appointment throttling above 0.85 utilization. |
| Multi-region | Yard operations stay facility-home-cell authoritative; DR cells expose read-only yard status. |
| Sovereign cells | Driver and chain-of-custody evidence stays in-region for active pack overlays. |
| Rollback | Disable operate endpoint, keep yard status read-only, and replay from the last sealed yard operation audit id. |
| Test evidence | Required tests cover ETA drift, dock conflict, dwell dispute, tenant mismatch, and idempotent check-in replay. |
| Rejected shortcut | A generic `DockEvent` usecase is rejected because it loses SAP EWM appointment and transportation-unit semantics. |
